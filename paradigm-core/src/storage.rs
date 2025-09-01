use anyhow::Result;
use chrono::{DateTime, Utc};
use dashmap::DashMap;
use serde::{Deserialize, Serialize};
use sqlx::{Row, SqlitePool};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;
use uuid::Uuid;

use crate::consensus::{MLTask, NetworkStats};
use crate::{transaction::Transaction, Address};


/// High-performance cache for frequently accessed data
#[derive(Debug, Clone)]
pub struct PerformanceCache {
    balances: DashMap<String, (u64, Instant)>,
    transactions: DashMap<String, (Transaction, Instant)>,
    ml_tasks: DashMap<String, (MLTask, Instant)>,
    cache_ttl: Duration,
}

impl PerformanceCache {
    pub fn new(cache_ttl_seconds: u64) -> Self {
        Self {
            balances: DashMap::new(),
            transactions: DashMap::new(),
            ml_tasks: DashMap::new(),
            cache_ttl: Duration::from_secs(cache_ttl_seconds),
        }
    }

    pub fn get_balance(&self, address: &str) -> Option<u64> {
        if let Some(entry) = self.balances.get(address) {
            if entry.1.elapsed() < self.cache_ttl {
                return Some(entry.0);
            } else {
                self.balances.remove(address);
            }
        }
        None
    }

    pub fn set_balance(&self, address: String, balance: u64) {
        self.balances.insert(address, (balance, Instant::now()));
    }

    pub fn get_transaction(&self, tx_id: &str) -> Option<Transaction> {
        if let Some(entry) = self.transactions.get(tx_id) {
            if entry.1.elapsed() < self.cache_ttl {
                return Some(entry.0.clone());
            } else {
                self.transactions.remove(tx_id);
            }
        }
        None
    }

    pub fn set_transaction(&self, tx_id: String, transaction: Transaction) {
        self.transactions
            .insert(tx_id, (transaction, Instant::now()));
    }

    pub fn cleanup_expired(&self) {
        let now = Instant::now();

        // Clean expired balances
        self.balances
            .retain(|_, v| now.duration_since(v.1) < self.cache_ttl);

        // Clean expired transactions
        self.transactions
            .retain(|_, v| now.duration_since(v.1) < self.cache_ttl);

        // Clean expired ML tasks
        self.ml_tasks
            .retain(|_, v| now.duration_since(v.1) < self.cache_ttl);
    }
}

/// Optimized batch operation manager
#[derive(Debug)]
pub struct BatchOperationManager {
    pending_transactions: Arc<RwLock<Vec<Transaction>>>,
    pending_ml_tasks: Arc<RwLock<Vec<MLTask>>>,
    pending_balance_updates: Arc<RwLock<HashMap<String, u64>>>,
    batch_size: usize,
    batch_timeout: Duration,
}

impl BatchOperationManager {
    pub fn new(batch_size: usize, batch_timeout_ms: u64) -> Self {
        Self {
            pending_transactions: Arc::new(RwLock::new(Vec::new())),
            pending_ml_tasks: Arc::new(RwLock::new(Vec::new())),
            pending_balance_updates: Arc::new(RwLock::new(HashMap::new())),
            batch_size,
            batch_timeout: Duration::from_millis(batch_timeout_ms),
        }
    }

    pub async fn add_transaction(&self, transaction: Transaction) -> bool {
        let mut pending = self.pending_transactions.write().await;
        pending.push(transaction);
        pending.len() >= self.batch_size
    }

    pub async fn add_ml_task(&self, task: MLTask) -> bool {
        let mut pending = self.pending_ml_tasks.write().await;
        pending.push(task);
        pending.len() >= self.batch_size
    }

    pub async fn add_balance_update(&self, address: String, balance: u64) -> bool {
        let mut pending = self.pending_balance_updates.write().await;
        pending.insert(address, balance);
        pending.len() >= self.batch_size
    }

    pub async fn drain_transactions(&self) -> Vec<Transaction> {
        let mut pending = self.pending_transactions.write().await;
        std::mem::take(&mut *pending)
    }

    pub async fn drain_ml_tasks(&self) -> Vec<MLTask> {
        let mut pending = self.pending_ml_tasks.write().await;
        std::mem::take(&mut *pending)
    }

    pub async fn drain_balance_updates(&self) -> HashMap<String, u64> {
        let mut pending = self.pending_balance_updates.write().await;
        std::mem::take(&mut *pending)
    }
}

/// Storage layer for Paradigm blockchain data with performance optimizations
#[derive(Debug)]
pub struct ParadigmStorage {
    db_pool: SqlitePool,
    cache: PerformanceCache,
    batch_manager: BatchOperationManager,
    connection_stats: Arc<RwLock<ConnectionStats>>,
}

impl ParadigmStorage {
    /// Create a new high-performance storage instance
    pub async fn new(database_url: &str) -> Result<Self> {
        Self::new_with_config(database_url, StorageConfig::default()).await
    }

    /// Create storage with custom performance configuration
    pub async fn new_with_config(database_url: &str, config: StorageConfig) -> Result<Self> {
        // Extract the actual file path from the database URL
        let file_path = if database_url.starts_with("sqlite://") {
            database_url
                .strip_prefix("sqlite://")
                .unwrap()
                .split('?')
                .next()
                .unwrap()
        } else {
            database_url
        };

        // Ensure the directory exists
        if let Some(parent) = std::path::Path::new(file_path).parent() {
            std::fs::create_dir_all(parent)?;
        }

        tracing::info!(
            "Connecting to high-performance database: sqlite://{}",
            file_path
        );

        // Create optimized connection pool
        let pool = sqlx::sqlite::SqlitePoolOptions::new()
            .max_connections(config.max_connections)
            .min_connections(config.min_connections)
            .acquire_timeout(Duration::from_millis(config.acquire_timeout_ms))
            .idle_timeout(Duration::from_secs(config.idle_timeout_secs))
            .connect_with(
                sqlx::sqlite::SqliteConnectOptions::new()
                    .filename(file_path)
                    .create_if_missing(true) // This should create the database if it doesn't exist
                    .pragma("cache_size", "-64000") // 64MB cache
                    .pragma("temp_store", "memory")
                    .pragma("mmap_size", "268435456") // 256MB mmap
                    .pragma("synchronous", "NORMAL")
                    .pragma("journal_mode", "WAL")
                    .pragma("foreign_keys", "ON")
                    .busy_timeout(Duration::from_millis(config.busy_timeout_ms)),
            )
            .await?;

        let storage = ParadigmStorage {
            db_pool: pool,
            cache: PerformanceCache::new(config.cache_ttl_secs),
            batch_manager: BatchOperationManager::new(config.batch_size, config.batch_timeout_ms),
            connection_stats: Arc::new(RwLock::new(ConnectionStats::default())),
        };

        storage.initialize_optimized_tables().await?;
        storage.optimize_database().await?;

        // Start background maintenance tasks
        storage.start_maintenance_tasks().await?;

        Ok(storage)
    }

    /// Initialize database tables with performance optimizations
    async fn initialize_optimized_tables(&self) -> Result<()> {
        // Enable performance optimizations at connection level
        sqlx::query("PRAGMA cache_size = -64000")
            .execute(&self.db_pool)
            .await?;
        sqlx::query("PRAGMA temp_store = memory")
            .execute(&self.db_pool)
            .await?;
        sqlx::query("PRAGMA mmap_size = 268435456")
            .execute(&self.db_pool)
            .await?;
        sqlx::query("PRAGMA synchronous = NORMAL")
            .execute(&self.db_pool)
            .await?;
        sqlx::query("PRAGMA journal_mode = WAL")
            .execute(&self.db_pool)
            .await?;

        // Optimized transactions table with better indexing
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS transactions (
                id TEXT PRIMARY KEY,
                from_address TEXT NOT NULL,
                to_address TEXT NOT NULL,
                amount INTEGER NOT NULL,
                fee INTEGER NOT NULL,
                timestamp TEXT NOT NULL,
                signature BLOB NOT NULL,
                nonce INTEGER NOT NULL,
                block_timestamp INTEGER,
                confirmed BOOLEAN DEFAULT FALSE,
                UNIQUE(id)
            ) WITHOUT ROWID
            "#,
        )
        .execute(&self.db_pool)
        .await?;

        // Optimized balances table
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS balances (
                address TEXT PRIMARY KEY,
                balance INTEGER NOT NULL DEFAULT 0,
                last_updated TEXT NOT NULL,
                nonce INTEGER NOT NULL DEFAULT 0
            ) WITHOUT ROWID
            "#,
        )
        .execute(&self.db_pool)
        .await?;

        // Optimized ML tasks table
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS ml_tasks (
                id TEXT PRIMARY KEY,
                task_type TEXT NOT NULL,
                data BLOB NOT NULL,
                difficulty INTEGER NOT NULL,
                reward INTEGER NOT NULL,
                deadline TEXT NOT NULL,
                created_at TEXT NOT NULL,
                assigned_to TEXT,
                completed BOOLEAN DEFAULT FALSE,
                result BLOB,
                priority INTEGER DEFAULT 0,
                UNIQUE(id)
            ) WITHOUT ROWID
            "#,
        )
        .execute(&self.db_pool)
        .await?;

        // Network metrics table (partitioned by time for better performance)
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS network_metrics (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                timestamp TEXT NOT NULL,
                total_tasks INTEGER NOT NULL,
                completed_tasks INTEGER NOT NULL,
                active_contributors INTEGER NOT NULL,
                total_rewards_pending INTEGER NOT NULL,
                network_difficulty INTEGER NOT NULL,
                hash_rate REAL DEFAULT 0.0,
                block_time REAL DEFAULT 0.0
            )
            "#,
        )
        .execute(&self.db_pool)
        .await?;

        // Data chunks for fast sync with compression
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS data_chunks (
                timestamp INTEGER PRIMARY KEY,
                hash BLOB NOT NULL,
                data BLOB NOT NULL,
                chunk_type TEXT NOT NULL,
                compressed BOOLEAN DEFAULT FALSE,
                original_size INTEGER DEFAULT 0
            ) WITHOUT ROWID
            "#,
        )
        .execute(&self.db_pool)
        .await?;

        // Peer information with performance metrics
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS peers (
                peer_id TEXT PRIMARY KEY,
                address TEXT NOT NULL,
                last_seen TEXT NOT NULL,
                reputation_score REAL DEFAULT 1.0,
                is_active BOOLEAN DEFAULT TRUE,
                latency_ms INTEGER DEFAULT 0,
                bandwidth_score REAL DEFAULT 1.0,
                contribution_score REAL DEFAULT 0.0
            ) WITHOUT ROWID
            "#,
        )
        .execute(&self.db_pool)
        .await?;

        // Performance indexes for optimal query speed
        self.create_performance_indexes().await?;

        tracing::info!("Optimized database tables initialized successfully");
        Ok(())
    }

    /// Create performance-optimized indexes
    async fn create_performance_indexes(&self) -> Result<()> {
        // Transaction indexes
        sqlx::query("CREATE INDEX IF NOT EXISTS idx_transactions_from ON transactions(from_address, timestamp DESC)")
            .execute(&self.db_pool).await?;
        sqlx::query("CREATE INDEX IF NOT EXISTS idx_transactions_to ON transactions(to_address, timestamp DESC)")
            .execute(&self.db_pool).await?;
        sqlx::query(
            "CREATE INDEX IF NOT EXISTS idx_transactions_timestamp ON transactions(timestamp DESC)",
        )
        .execute(&self.db_pool)
        .await?;
        sqlx::query("CREATE INDEX IF NOT EXISTS idx_transactions_confirmed ON transactions(confirmed, timestamp DESC)")
            .execute(&self.db_pool).await?;

        // ML task indexes
        sqlx::query(
            "CREATE INDEX IF NOT EXISTS idx_ml_tasks_type ON ml_tasks(task_type, created_at DESC)",
        )
        .execute(&self.db_pool)
        .await?;
        sqlx::query(
            "CREATE INDEX IF NOT EXISTS idx_ml_tasks_assigned ON ml_tasks(assigned_to, completed)",
        )
        .execute(&self.db_pool)
        .await?;
        sqlx::query("CREATE INDEX IF NOT EXISTS idx_ml_tasks_deadline ON ml_tasks(deadline ASC)")
            .execute(&self.db_pool)
            .await?;
        sqlx::query("CREATE INDEX IF NOT EXISTS idx_ml_tasks_priority ON ml_tasks(priority DESC, created_at ASC)")
            .execute(&self.db_pool).await?;

        // Network metrics indexes
        sqlx::query("CREATE INDEX IF NOT EXISTS idx_network_metrics_timestamp ON network_metrics(timestamp DESC)")
            .execute(&self.db_pool).await?;

        // Data chunks indexes
        sqlx::query("CREATE INDEX IF NOT EXISTS idx_data_chunks_type ON data_chunks(chunk_type, timestamp DESC)")
            .execute(&self.db_pool).await?;

        // Peer indexes
        sqlx::query(
            "CREATE INDEX IF NOT EXISTS idx_peers_active ON peers(is_active, last_seen DESC)",
        )
        .execute(&self.db_pool)
        .await?;
        sqlx::query("CREATE INDEX IF NOT EXISTS idx_peers_reputation ON peers(reputation_score DESC, contribution_score DESC)")
            .execute(&self.db_pool).await?;

        tracing::info!("Performance indexes created successfully");
        Ok(())
    }

    /// Optimize database with maintenance operations
    async fn optimize_database(&self) -> Result<()> {
        // Analyze tables for query optimization
        sqlx::query("ANALYZE").execute(&self.db_pool).await?;

        // Optimize database file
        sqlx::query("PRAGMA optimize")
            .execute(&self.db_pool)
            .await?;

        tracing::info!("Database optimization completed");
        Ok(())
    }

    /// Start background maintenance tasks
    async fn start_maintenance_tasks(&self) -> Result<()> {
        let cache = Arc::new(self.cache.clone());
        let db_pool = self.db_pool.clone();
        let stats = self.connection_stats.clone();

        // Cache cleanup task
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(Duration::from_secs(300)); // 5 minutes
            loop {
                interval.tick().await;
                cache.cleanup_expired();

                // Update connection stats
                {
                    let mut conn_stats = stats.write().await;
                    conn_stats.cache_cleanups += 1;
                    conn_stats.last_cleanup = Utc::now();
                }
            }
        });

        // WAL checkpoint task
        let db_pool_wal = self.db_pool.clone();
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(Duration::from_secs(1800)); // 30 minutes
            loop {
                interval.tick().await;
                if let Err(e) = sqlx::query("PRAGMA wal_checkpoint(TRUNCATE)")
                    .execute(&db_pool_wal)
                    .await
                {
                    tracing::warn!("WAL checkpoint failed: {}", e);
                } else {
                    tracing::debug!("WAL checkpoint completed");
                }
            }
        });

        // Database vacuum task (weekly)
        let db_pool_vacuum = self.db_pool.clone();
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(Duration::from_secs(604800)); // 1 week
            loop {
                interval.tick().await;
                if let Err(e) = sqlx::query("VACUUM").execute(&db_pool_vacuum).await {
                    tracing::warn!("Database vacuum failed: {}", e);
                } else {
                    tracing::info!("Database vacuum completed");
                }
            }
        });

        tracing::info!("Background maintenance tasks started");
        Ok(())
    }

    /// High-performance store transaction with caching and batching
    pub async fn store_transaction_optimized(&self, transaction: &Transaction) -> Result<()> {
        // Check cache first
        let tx_id = transaction.id.to_string();
        self.cache
            .set_transaction(tx_id.clone(), transaction.clone());

        // Add to batch or store immediately if batch is full
        if self
            .batch_manager
            .add_transaction(transaction.clone())
            .await
        {
            self.flush_transaction_batch().await?;
        }

        Ok(())
    }

    /// Flush pending transaction batch
    async fn flush_transaction_batch(&self) -> Result<()> {
        let transactions = self.batch_manager.drain_transactions().await;
        if transactions.is_empty() {
            return Ok(());
        }

        // Batch insert for optimal performance
        let mut tx = self.db_pool.begin().await?;

        for transaction in &transactions {
            sqlx::query(
                r#"
                INSERT OR REPLACE INTO transactions 
                (id, from_address, to_address, amount, fee, timestamp, signature, nonce, confirmed)
                VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)
                "#,
            )
            .bind(transaction.id.to_string())
            .bind(transaction.from.to_string())
            .bind(transaction.to.to_string())
            .bind(transaction.amount as i64)
            .bind(transaction.fee as i64)
            .bind(transaction.timestamp.to_rfc3339())
            .bind(&transaction.signature)
            .bind(transaction.nonce as i64)
            .bind(false)
            .execute(&mut *tx)
            .await?;
        }

        tx.commit().await?;

        // Update stats
        {
            let mut stats = self.connection_stats.write().await;
            stats.total_queries += transactions.len() as u64;
            stats.batch_operations += 1;
        }

        tracing::debug!("Flushed {} transactions in batch", transactions.len());
        Ok(())
    }

    /// Store a transaction
    pub async fn store_transaction(&self, transaction: &Transaction) -> Result<()> {
        sqlx::query(
            r#"
            INSERT OR REPLACE INTO transactions 
            (id, from_address, to_address, amount, fee, timestamp, signature, nonce, confirmed)
            VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)
            "#,
        )
        .bind(transaction.id.to_string())
        .bind(transaction.from.to_string())
        .bind(transaction.to.to_string())
        .bind(transaction.amount as i64)
        .bind(transaction.fee as i64)
        .bind(transaction.timestamp.to_rfc3339())
        .bind(&transaction.signature)
        .bind(transaction.nonce as i64)
        .bind(false) // Not confirmed by default
        .execute(&self.db_pool)
        .await?;

        Ok(())
    }

    /// Get transaction by ID
    pub async fn get_transaction(&self, transaction_id: &Uuid) -> Result<Option<Transaction>> {
        let row = sqlx::query("SELECT * FROM transactions WHERE id = ?")
            .bind(transaction_id.to_string())
            .fetch_optional(&self.db_pool)
            .await?;

        if let Some(row) = row {
            // Reconstruct transaction from database row
            // This is simplified - in reality we'd need to handle all fields properly
            Ok(Some(self.row_to_transaction(row)?))
        } else {
            Ok(None)
        }
    }

    /// Get transactions for an address
    pub async fn get_transactions_for_address(
        &self,
        address: &Address,
    ) -> Result<Vec<Transaction>> {
        let rows = sqlx::query(
            "SELECT * FROM transactions WHERE from_address = ? OR to_address = ? ORDER BY timestamp DESC"
        )
        .bind(address.to_string())
        .bind(address.to_string())
        .fetch_all(&self.db_pool)
        .await?;

        let mut transactions = Vec::new();
        for row in rows {
            transactions.push(self.row_to_transaction(row)?);
        }

        Ok(transactions)
    }

    /// Convert database row to Transaction
    fn row_to_transaction(&self, row: sqlx::sqlite::SqliteRow) -> Result<Transaction> {
        let id_str: String = row.get("id");
        let from_str: String = row.get("from_address");
        let to_str: String = row.get("to_address");
        let timestamp_str: String = row.get("timestamp");

        // Parse the data
        let id = Uuid::parse_str(&id_str)?;
        let from = self.parse_address(&from_str)?;
        let to = self.parse_address(&to_str)?;
        let amount = row.get::<i64, _>("amount") as u64;
        let fee = row.get::<i64, _>("fee") as u64;
        let timestamp = DateTime::parse_from_rfc3339(&timestamp_str)?.with_timezone(&Utc);
        let signature: Vec<u8> = row.get("signature");
        let nonce = row.get::<i64, _>("nonce") as u64;

        Ok(Transaction {
            id,
            from,
            to,
            amount,
            fee,
            timestamp,
            signature,
            nonce,
            message: None, // TODO: Add message column to database schema
        })
    }

    /// Parse address string to Address
    fn parse_address(&self, addr_str: &str) -> Result<Address> {
        if addr_str.starts_with("PAR") && addr_str.len() >= 43 {
            // Extract hex part and convert to bytes
            let hex_part = &addr_str[3..];
            if let Ok(bytes) = hex::decode(hex_part) {
                if bytes.len() >= 20 {
                    let mut addr_bytes = [0u8; 32];
                    addr_bytes[..20].copy_from_slice(&bytes[..20]);
                    Ok(Address(addr_bytes))
                } else {
                    Err(anyhow::anyhow!(
                        "Invalid address format - insufficient bytes"
                    ))
                }
            } else {
                Err(anyhow::anyhow!("Invalid address format - not hex"))
            }
        } else {
            Err(anyhow::anyhow!("Invalid address format"))
        }
    }

    /// Update balance for an address
    pub async fn update_balance(&self, address: &Address, balance: u64) -> Result<()> {
        sqlx::query(
            "INSERT OR REPLACE INTO balances (address, balance, last_updated) VALUES (?, ?, ?)",
        )
        .bind(address.to_string())
        .bind(balance as i64)
        .bind(Utc::now().to_rfc3339())
        .execute(&self.db_pool)
        .await?;

        Ok(())
    }

    /// Get balance for an address
    pub async fn get_balance(&self, address: &Address) -> Result<u64> {
        let row = sqlx::query("SELECT balance FROM balances WHERE address = ?")
            .bind(address.to_string())
            .fetch_optional(&self.db_pool)
            .await?;

        if let Some(row) = row {
            Ok(row.get::<i64, _>("balance") as u64)
        } else {
            Ok(0)
        }
    }

    /// Store ML task
    pub async fn store_ml_task(&self, task: &MLTask) -> Result<()> {
        sqlx::query(
            r#"
            INSERT OR REPLACE INTO ml_tasks 
            (id, task_type, data, difficulty, reward, deadline, created_at, assigned_to, completed, result)
            VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
            "#,
        )
        .bind(task.id.to_string())
        .bind(format!("{:?}", task.task_type))
        .bind(&task.data)
        .bind(task.difficulty as i64)
        .bind(task.reward as i64)
        .bind(task.deadline.to_rfc3339())
        .bind(task.created_at.to_rfc3339())
        .bind(task.assigned_to.as_ref().map(|a| a.to_string()))
        .bind(task.completed)
        .bind(&task.result)
        .execute(&self.db_pool)
        .await?;

        Ok(())
    }

    /// Get ML task by ID
    pub async fn get_ml_task(&self, task_id: &Uuid) -> Result<Option<MLTask>> {
        let row = sqlx::query("SELECT * FROM ml_tasks WHERE id = ?")
            .bind(task_id.to_string())
            .fetch_optional(&self.db_pool)
            .await?;

        if let Some(row) = row {
            Ok(Some(self.row_to_ml_task(row)?))
        } else {
            Ok(None)
        }
    }

    /// Convert database row to MLTask
    fn row_to_ml_task(&self, row: sqlx::sqlite::SqliteRow) -> Result<MLTask> {
        let id_str: String = row.get("id");
        let task_type_str: String = row.get("task_type");
        let deadline_str: String = row.get("deadline");
        let created_at_str: String = row.get("created_at");

        let id = Uuid::parse_str(&id_str)?;
        let task_type = self.parse_task_type(&task_type_str)?;
        let data: Vec<u8> = row.get("data");
        let difficulty = row.get::<i64, _>("difficulty") as u8;
        let reward = row.get::<i64, _>("reward") as u64;
        let deadline = DateTime::parse_from_rfc3339(&deadline_str)?.with_timezone(&Utc);
        let created_at = DateTime::parse_from_rfc3339(&created_at_str)?.with_timezone(&Utc);
        let assigned_to_str: Option<String> = row.get("assigned_to");
        let assigned_to = if let Some(addr_str) = assigned_to_str {
            Some(self.parse_address(&addr_str)?)
        } else {
            None
        };
        let completed: bool = row.get("completed");
        let result: Option<Vec<u8>> = row.get("result");

        Ok(MLTask {
            id,
            task_type,
            data,
            difficulty,
            reward,
            deadline,
            created_at,
            assigned_to,
            completed,
            result,
        })
    }

    /// Parse task type string
    fn parse_task_type(&self, type_str: &str) -> Result<crate::consensus::MLTaskType> {
        match type_str {
            "ImageClassification" => Ok(crate::consensus::MLTaskType::ImageClassification),
            "NaturalLanguageProcessing" => {
                Ok(crate::consensus::MLTaskType::NaturalLanguageProcessing)
            }
            "TimeSeriesAnalysis" => Ok(crate::consensus::MLTaskType::TimeSeriesAnalysis),
            "ReinforcementLearning" => Ok(crate::consensus::MLTaskType::ReinforcementLearning),
            "AutoML" => Ok(crate::consensus::MLTaskType::AutoML),
            "DistributedTraining" => Ok(crate::consensus::MLTaskType::DistributedTraining),
            "Oracle" => Ok(crate::consensus::MLTaskType::Oracle),
            "SmartContractOptimization" => {
                Ok(crate::consensus::MLTaskType::SmartContractOptimization)
            }
            "NetworkOptimization" => Ok(crate::consensus::MLTaskType::NetworkOptimization),
            _ => Err(anyhow::anyhow!("Unknown task type: {}", type_str)),
        }
    }

    /// Store network metrics
    pub async fn store_network_metrics(&self, metrics: &NetworkStats) -> Result<()> {
        sqlx::query(
            r#"
            INSERT INTO network_metrics 
            (timestamp, total_tasks, completed_tasks, active_contributors, total_rewards_pending, network_difficulty)
            VALUES (?, ?, ?, ?, ?, ?)
            "#,
        )
        .bind(Utc::now().to_rfc3339())
        .bind(metrics.total_tasks as i64)
        .bind(metrics.completed_tasks as i64)
        .bind(metrics.active_contributors as i64)
        .bind(metrics.total_rewards_pending as i64)
        .bind(metrics.network_difficulty as i64)
        .execute(&self.db_pool)
        .await?;

        Ok(())
    }

    /// Get recent network metrics
    pub async fn get_recent_metrics(&self, limit: i64) -> Result<Vec<NetworkStats>> {
        let rows = sqlx::query("SELECT * FROM network_metrics ORDER BY timestamp DESC LIMIT ?")
            .bind(limit)
            .fetch_all(&self.db_pool)
            .await?;

        let mut metrics = Vec::new();
        for row in rows {
            metrics.push(NetworkStats {
                total_tasks: row.get::<i64, _>("total_tasks") as usize,
                completed_tasks: row.get::<i64, _>("completed_tasks") as usize,
                active_contributors: row.get::<i64, _>("active_contributors") as usize,
                total_rewards_pending: row.get::<i64, _>("total_rewards_pending") as u64,
                network_difficulty: row.get::<i64, _>("network_difficulty") as u8,
            });
        }

        Ok(metrics)
    }

    /// Store data chunk for fast sync
    pub async fn store_data_chunk(
        &self,
        timestamp: i64,
        hash: &[u8],
        data: &[u8],
        chunk_type: &str,
    ) -> Result<()> {
        sqlx::query(
            "INSERT OR REPLACE INTO data_chunks (timestamp, hash, data, chunk_type) VALUES (?, ?, ?, ?)"
        )
        .bind(timestamp)
        .bind(hash)
        .bind(data)
        .bind(chunk_type)
        .execute(&self.db_pool)
        .await?;

        Ok(())
    }

    /// Get data chunks after timestamp
    pub async fn get_data_chunks_after(&self, timestamp: i64) -> Result<Vec<DataChunk>> {
        let rows =
            sqlx::query("SELECT * FROM data_chunks WHERE timestamp > ? ORDER BY timestamp ASC")
                .bind(timestamp)
                .fetch_all(&self.db_pool)
                .await?;

        let mut chunks = Vec::new();
        for row in rows {
            chunks.push(DataChunk {
                timestamp: row.get("timestamp"),
                hash: row.get("hash"),
                data: row.get("data"),
                chunk_type: self.parse_chunk_type(&row.get::<String, _>("chunk_type"))?,
            });
        }

        Ok(chunks)
    }

    /// Parse chunk type
    fn parse_chunk_type(&self, type_str: &str) -> Result<crate::network::ChunkType> {
        match type_str {
            "Transactions" => Ok(crate::network::ChunkType::Transactions),
            "MLTasks" => Ok(crate::network::ChunkType::MLTasks),
            "Balances" => Ok(crate::network::ChunkType::Balances),
            "NetworkState" => Ok(crate::network::ChunkType::NetworkState),
            _ => Err(anyhow::anyhow!("Unknown chunk type: {}", type_str)),
        }
    }

    /// Get database statistics
    pub async fn get_storage_stats(&self) -> Result<StorageStats> {
        let transaction_count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM transactions")
            .fetch_one(&self.db_pool)
            .await?;

        let task_count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM ml_tasks")
            .fetch_one(&self.db_pool)
            .await?;

        let balance_count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM balances")
            .fetch_one(&self.db_pool)
            .await?;

        let chunk_count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM data_chunks")
            .fetch_one(&self.db_pool)
            .await?;

        Ok(StorageStats {
            transaction_count: transaction_count as u64,
            task_count: task_count as u64,
            balance_count: balance_count as u64,
            chunk_count: chunk_count as u64,
        })
    }

    /// Get total transaction count for network sync
    pub async fn get_transaction_count(&self) -> Result<u64> {
        let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM transactions")
            .fetch_one(&self.db_pool)
            .await?;
        Ok(count as u64)
    }

    /// Cleanup old data (for maintenance)
    pub async fn cleanup_old_data(&self, days_to_keep: i64) -> Result<u64> {
        let cutoff_date = Utc::now() - chrono::Duration::days(days_to_keep);

        let result = sqlx::query("DELETE FROM network_metrics WHERE timestamp < ?")
            .bind(cutoff_date.to_rfc3339())
            .execute(&self.db_pool)
            .await?;

        Ok(result.rows_affected())
    }

    // Genesis-related storage methods

    /// Store genesis block
    pub async fn store_genesis_block(
        &self,
        genesis_block: &crate::genesis::GenesisBlock,
    ) -> Result<()> {
        let genesis_data = serde_json::to_string(genesis_block)?;

        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS genesis_blocks (
                block_number INTEGER PRIMARY KEY,
                hash BLOB NOT NULL,
                timestamp TEXT NOT NULL,
                config_data TEXT NOT NULL,
                created_at TEXT DEFAULT CURRENT_TIMESTAMP
            )
            "#,
        )
        .execute(&self.db_pool)
        .await?;

        sqlx::query(
            "INSERT OR REPLACE INTO genesis_blocks (block_number, hash, timestamp, config_data) VALUES (?, ?, ?, ?)"
        )
        .bind(genesis_block.block_number as i64)
        .bind(&genesis_block.hash[..])
        .bind(genesis_block.timestamp.to_rfc3339())
        .bind(genesis_data)
        .execute(&self.db_pool)
        .await?;

        Ok(())
    }

    /// Store AI governance parameters
    pub async fn store_ai_governance_params(
        &self,
        params: &crate::genesis::AIGovernanceParams,
    ) -> Result<()> {
        let params_data = serde_json::to_string(params)?;

        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS ai_governance_params (
                id INTEGER PRIMARY KEY DEFAULT 1,
                params_data TEXT NOT NULL,
                updated_at TEXT DEFAULT CURRENT_TIMESTAMP
            )
            "#,
        )
        .execute(&self.db_pool)
        .await?;

        sqlx::query("INSERT OR REPLACE INTO ai_governance_params (id, params_data) VALUES (1, ?)")
            .bind(params_data)
            .execute(&self.db_pool)
            .await?;

        Ok(())
    }

    /// Get AI governance parameters
    pub async fn get_ai_governance_params(&self) -> Result<crate::genesis::AIGovernanceParams> {
        let row = sqlx::query("SELECT params_data FROM ai_governance_params WHERE id = 1")
            .fetch_optional(&self.db_pool)
            .await?;

        if let Some(row) = row {
            let params_data: String = row.get("params_data");
            Ok(serde_json::from_str(&params_data)?)
        } else {
            Ok(crate::genesis::AIGovernanceParams::default())
        }
    }

    /// Store network genesis configuration
    pub async fn store_network_genesis_config(
        &self,
        config: &crate::genesis::NetworkGenesisConfig,
    ) -> Result<()> {
        let config_data = serde_json::to_string(config)?;

        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS network_genesis_config (
                id INTEGER PRIMARY KEY DEFAULT 1,
                config_data TEXT NOT NULL,
                updated_at TEXT DEFAULT CURRENT_TIMESTAMP
            )
            "#,
        )
        .execute(&self.db_pool)
        .await?;

        sqlx::query(
            "INSERT OR REPLACE INTO network_genesis_config (id, config_data) VALUES (1, ?)",
        )
        .bind(config_data)
        .execute(&self.db_pool)
        .await?;

        Ok(())
    }

    /// Store genesis features
    pub async fn store_genesis_features(
        &self,
        features: &crate::genesis::GenesisFeatures,
    ) -> Result<()> {
        let features_data = serde_json::to_string(features)?;

        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS genesis_features (
                id INTEGER PRIMARY KEY DEFAULT 1,
                features_data TEXT NOT NULL,
                updated_at TEXT DEFAULT CURRENT_TIMESTAMP
            )
            "#,
        )
        .execute(&self.db_pool)
        .await?;

        sqlx::query("INSERT OR REPLACE INTO genesis_features (id, features_data) VALUES (1, ?)")
            .bind(features_data)
            .execute(&self.db_pool)
            .await?;

        Ok(())
    }

    /// Set balance for an address (used for genesis initialization)
    pub async fn set_balance(&self, address: &Address, balance: u64) -> Result<()> {
        sqlx::query(
            "INSERT OR REPLACE INTO balances (address, balance, updated_at) VALUES (?, ?, CURRENT_TIMESTAMP)"
        )
        .bind(address.to_string())
        .bind(balance as i64)
        .execute(&self.db_pool)
        .await?;

        // Update cache
        self.cache.set_balance(address.to_string(), balance);

        Ok(())
    }

    /// Get a reference to the database pool for advanced operations
    pub fn get_db_pool(&self) -> &SqlitePool {
        &self.db_pool
    }
}

/// Data chunk for storage
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataChunk {
    pub timestamp: i64,
    pub hash: Vec<u8>,
    pub data: Vec<u8>,
    pub chunk_type: crate::network::ChunkType,
}

/// Storage statistics
#[derive(Debug, Serialize, Deserialize)]
pub struct StorageStats {
    pub transaction_count: u64,
    pub task_count: u64,
    pub balance_count: u64,
    pub chunk_count: u64,
}

/// Storage configuration for performance tuning
#[derive(Debug, Clone)]
pub struct StorageConfig {
    pub max_connections: u32,
    pub min_connections: u32,
    pub acquire_timeout_ms: u64,
    pub idle_timeout_secs: u64,
    pub busy_timeout_ms: u64,
    pub cache_ttl_secs: u64,
    pub batch_size: usize,
    pub batch_timeout_ms: u64,
}

impl Default for StorageConfig {
    fn default() -> Self {
        Self {
            max_connections: 20,
            min_connections: 5,
            acquire_timeout_ms: 30000,
            idle_timeout_secs: 600,
            busy_timeout_ms: 30000,
            cache_ttl_secs: 300,
            batch_size: 100,
            batch_timeout_ms: 1000,
        }
    }
}

/// Connection and performance statistics
#[derive(Debug, Default)]
pub struct ConnectionStats {
    pub total_queries: u64,
    pub cache_hits: u64,
    pub cache_misses: u64,
    pub batch_operations: u64,
    pub cache_cleanups: u64,
    pub last_cleanup: DateTime<Utc>,
    pub connection_pool_size: u32,
    pub active_connections: u32,
}

impl ConnectionStats {
    pub fn cache_hit_ratio(&self) -> f64 {
        if self.cache_hits + self.cache_misses == 0 {
            0.0
        } else {
            self.cache_hits as f64 / (self.cache_hits + self.cache_misses) as f64
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[tokio::test]
    async fn test_storage_creation() {
        let temp_dir = tempdir().unwrap();
        let db_path = temp_dir.path().join("test.db");

        let storage = ParadigmStorage::new(db_path.to_str().unwrap()).await;
        assert!(storage.is_ok());
    }

    #[tokio::test]
    async fn test_cache_functionality() {
        let cache = PerformanceCache::new(60);

        // Test balance cache
        cache.set_balance("test_address".to_string(), 1000);
        assert_eq!(cache.get_balance("test_address"), Some(1000));
        assert_eq!(cache.get_balance("non_existent"), None);

        // Test cache expiration (would need to wait for TTL in real scenario)
        cache.cleanup_expired();
    }

    #[tokio::test]
    async fn test_batch_operations() {
        let batch_manager = BatchOperationManager::new(3, 1000);

        // Test that batch doesn't trigger until size is reached
        let tx1 = create_test_transaction();
        let tx2 = create_test_transaction();

        assert!(!batch_manager.add_transaction(tx1).await);
        assert!(!batch_manager.add_transaction(tx2).await);

        let tx3 = create_test_transaction();
        assert!(batch_manager.add_transaction(tx3).await); // Should trigger batch

        let drained = batch_manager.drain_transactions().await;
        assert_eq!(drained.len(), 3);
    }

    fn create_test_transaction() -> Transaction {
        Transaction {
            id: uuid::Uuid::new_v4(),
            from: Address([0u8; 32]),
            to: Address([1u8; 32]),
            amount: 100,
            fee: 10,
            timestamp: chrono::Utc::now(),
            signature: vec![0u8; 64],
            nonce: 1,
        }
    }
}
