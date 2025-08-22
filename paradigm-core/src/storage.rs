use std::collections::HashMap;
use sqlx::{SqlitePool, Row};
use anyhow::Result;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::{DateTime, Utc};

use crate::{Address, transaction::Transaction};
use crate::consensus::{MLTask, NetworkStats};

/// Storage layer for Paradigm blockchain data
#[derive(Debug)]
pub struct ParadigmStorage {
    db_pool: SqlitePool,
}

impl ParadigmStorage {
    /// Create a new storage instance
    pub async fn new(database_url: &str) -> Result<Self> {
        // Handle both file paths and SQLite URLs
        let connection_string = if database_url.starts_with("sqlite://") {
            format!("{}?mode=rwc", database_url)
        } else {
            // Ensure the directory exists
            if let Some(parent) = std::path::Path::new(database_url).parent() {
                std::fs::create_dir_all(parent)?;
            }
            // Convert path separators to forward slashes for SQLite URL
            let normalized_path = database_url.replace('\\', "/");
            format!("sqlite://{}?mode=rwc", normalized_path)
        };
        
        tracing::info!("Connecting to database: {}", connection_string);
        let pool = SqlitePool::connect(&connection_string).await?;
        
        let storage = ParadigmStorage { db_pool: pool };
        storage.initialize_tables().await?;
        
        Ok(storage)
    }

    /// Initialize database tables
    async fn initialize_tables(&self) -> Result<()> {
        // Transactions table
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
            )
            "#,
        )
        .execute(&self.db_pool)
        .await?;

        // Balances table
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS balances (
                address TEXT PRIMARY KEY,
                balance INTEGER NOT NULL DEFAULT 0,
                last_updated TEXT NOT NULL
            )
            "#,
        )
        .execute(&self.db_pool)
        .await?;

        // ML tasks table
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
                UNIQUE(id)
            )
            "#,
        )
        .execute(&self.db_pool)
        .await?;

        // Network metrics table
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS network_metrics (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                timestamp TEXT NOT NULL,
                total_tasks INTEGER NOT NULL,
                completed_tasks INTEGER NOT NULL,
                active_contributors INTEGER NOT NULL,
                total_rewards_pending INTEGER NOT NULL,
                network_difficulty INTEGER NOT NULL
            )
            "#,
        )
        .execute(&self.db_pool)
        .await?;

        // Data chunks for fast sync
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS data_chunks (
                timestamp INTEGER PRIMARY KEY,
                hash BLOB NOT NULL,
                data BLOB NOT NULL,
                chunk_type TEXT NOT NULL
            )
            "#,
        )
        .execute(&self.db_pool)
        .await?;

        // Peer information
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS peers (
                peer_id TEXT PRIMARY KEY,
                address TEXT NOT NULL,
                last_seen TEXT NOT NULL,
                reputation_score REAL DEFAULT 1.0,
                is_active BOOLEAN DEFAULT TRUE
            )
            "#,
        )
        .execute(&self.db_pool)
        .await?;

        tracing::info!("Database tables initialized successfully");
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
        let row = sqlx::query(
            "SELECT * FROM transactions WHERE id = ?"
        )
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
    pub async fn get_transactions_for_address(&self, address: &Address) -> Result<Vec<Transaction>> {
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
                    Err(anyhow::anyhow!("Invalid address format - insufficient bytes"))
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
            "INSERT OR REPLACE INTO balances (address, balance, last_updated) VALUES (?, ?, ?)"
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
        let row = sqlx::query(
            "SELECT balance FROM balances WHERE address = ?"
        )
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
        let row = sqlx::query(
            "SELECT * FROM ml_tasks WHERE id = ?"
        )
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
            "NaturalLanguageProcessing" => Ok(crate::consensus::MLTaskType::NaturalLanguageProcessing),
            "TimeSeriesAnalysis" => Ok(crate::consensus::MLTaskType::TimeSeriesAnalysis),
            "ReinforcementLearning" => Ok(crate::consensus::MLTaskType::ReinforcementLearning),
            "AutoML" => Ok(crate::consensus::MLTaskType::AutoML),
            "DistributedTraining" => Ok(crate::consensus::MLTaskType::DistributedTraining),
            "Oracle" => Ok(crate::consensus::MLTaskType::Oracle),
            "SmartContractOptimization" => Ok(crate::consensus::MLTaskType::SmartContractOptimization),
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
        let rows = sqlx::query(
            "SELECT * FROM network_metrics ORDER BY timestamp DESC LIMIT ?"
        )
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
        let rows = sqlx::query(
            "SELECT * FROM data_chunks WHERE timestamp > ? ORDER BY timestamp ASC"
        )
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

    /// Cleanup old data (for maintenance)
    pub async fn cleanup_old_data(&self, days_to_keep: i64) -> Result<u64> {
        let cutoff_date = Utc::now() - chrono::Duration::days(days_to_keep);
        
        let result = sqlx::query(
            "DELETE FROM network_metrics WHERE timestamp < ?"
        )
        .bind(cutoff_date.to_rfc3339())
        .execute(&self.db_pool)
        .await?;

        Ok(result.rows_affected())
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

#[cfg(test)]
mod tests {
    // Tests disabled temporarily
    // TODO: Add tempfile dependency for testing
}
