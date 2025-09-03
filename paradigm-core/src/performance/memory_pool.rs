// Advanced Memory Pool (Mempool) with Intelligent Transaction Management
// Optimizes transaction storage and retrieval for maximum throughput

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet, HashMap, VecDeque};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;
use tracing::{debug, error, info, warn};
use uuid::Uuid;

use crate::{Address, Transaction};

/// Memory pool configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MempoolConfig {
    pub max_transactions: usize,
    pub max_size_bytes: usize,
    pub transaction_ttl: Duration,
    pub enable_priority_sorting: bool,
    pub enable_fee_estimation: bool,
    pub enable_replacement: bool,
    pub max_transactions_per_account: usize,
    pub cleanup_interval: Duration,
}

impl Default for MempoolConfig {
    fn default() -> Self {
        Self {
            max_transactions: 100_000,
            max_size_bytes: 256 * 1024 * 1024,         // 256MB
            transaction_ttl: Duration::from_secs(300), // 5 minutes
            enable_priority_sorting: true,
            enable_fee_estimation: true,
            enable_replacement: true,
            max_transactions_per_account: 1000,
            cleanup_interval: Duration::from_secs(30),
        }
    }
}

/// Transaction priority information
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct TransactionPriority {
    pub fee_per_byte: u64,
    pub timestamp: Instant,
    pub is_system: bool,
}

/// Memory pool entry with metadata
#[derive(Debug, Clone)]
pub struct MempoolEntry {
    pub transaction: Transaction,
    pub priority: TransactionPriority,
    pub size_bytes: usize,
    pub added_at: Instant,
    pub dependencies: Vec<Uuid>, // Transaction dependencies
    pub dependents: Vec<Uuid>,   // Transactions that depend on this one
}

/// Advanced memory pool with intelligent transaction management
pub struct AdvancedMempool {
    config: MempoolConfig,

    // Core storage
    transactions: Arc<RwLock<HashMap<Uuid, MempoolEntry>>>,

    // Indexing structures for fast lookup
    by_priority: Arc<RwLock<BTreeMap<TransactionPriority, BTreeSet<Uuid>>>>,
    by_sender: Arc<RwLock<HashMap<Address, BTreeSet<Uuid>>>>,
    by_recipient: Arc<RwLock<HashMap<Address, BTreeSet<Uuid>>>>,

    // Dependency tracking
    dependency_graph: Arc<RwLock<HashMap<Uuid, Vec<Uuid>>>>,

    // Statistics and metrics
    metrics: Arc<RwLock<MempoolMetrics>>,

    // Fee estimation
    fee_estimator: Arc<RwLock<FeeEstimator>>,
}

#[derive(Debug, Default, Clone)]
pub struct MempoolMetrics {
    pub total_transactions: usize,
    pub total_size_bytes: usize,
    pub average_fee: u64,
    pub throughput_tps: f64,
    pub hit_rate: f64,
    pub eviction_count: u64,
    pub replacement_count: u64,
    pub dependency_resolution_time: Duration,
}

/// Dynamic fee estimation based on mempool state
#[derive(Debug)]
pub struct FeeEstimator {
    fee_history: VecDeque<u64>,
    confirmation_times: HashMap<u64, Duration>,
    network_congestion: f64,
}

impl Default for FeeEstimator {
    fn default() -> Self {
        Self {
            fee_history: VecDeque::with_capacity(1000),
            confirmation_times: HashMap::new(),
            network_congestion: 0.0,
        }
    }
}

impl AdvancedMempool {
    pub fn new(config: MempoolConfig) -> Self {
        let mempool = Self {
            config: config.clone(),
            transactions: Arc::new(RwLock::new(HashMap::new())),
            by_priority: Arc::new(RwLock::new(BTreeMap::new())),
            by_sender: Arc::new(RwLock::new(HashMap::new())),
            by_recipient: Arc::new(RwLock::new(HashMap::new())),
            dependency_graph: Arc::new(RwLock::new(HashMap::new())),
            metrics: Arc::new(RwLock::new(MempoolMetrics::default())),
            fee_estimator: Arc::new(RwLock::new(FeeEstimator::default())),
        };

        // Start background cleanup task
        let cleanup_mempool = mempool.clone();
        tokio::spawn(async move {
            cleanup_mempool.cleanup_loop().await;
        });

        mempool
    }

    /// Add transaction to mempool with intelligent placement
    pub async fn add_transaction(&self, transaction: Transaction) -> Result<bool> {
        let tx_id = transaction.id;
        let sender = transaction.from.clone();
        let recipient = transaction.to.clone();

        // Check if transaction already exists
        {
            let transactions = self.transactions.read().await;
            if transactions.contains_key(&tx_id) {
                debug!("Transaction {} already in mempool", tx_id);
                return Ok(false);
            }
        }

        // Check mempool limits
        if !self.check_capacity_limits(&transaction).await? {
            debug!("Mempool capacity limits reached");
            self.evict_low_priority_transactions().await?;
        }

        // Check per-account limits
        if !self.check_account_limits(&sender).await? {
            warn!("Account {} has too many pending transactions", sender);
            return Ok(false);
        }

        // Calculate transaction priority
        let priority = self.calculate_priority(&transaction).await;
        let size_bytes = self.estimate_transaction_size(&transaction);

        // Create mempool entry
        let entry = MempoolEntry {
            transaction: transaction.clone(),
            priority: priority.clone(),
            size_bytes,
            added_at: Instant::now(),
            dependencies: self.find_dependencies(&transaction).await,
            dependents: Vec::new(),
        };

        // Add to all index structures
        {
            let mut transactions = self.transactions.write().await;
            let mut by_priority = self.by_priority.write().await;
            let mut by_sender = self.by_sender.write().await;
            let mut by_recipient = self.by_recipient.write().await;

            // Insert transaction
            transactions.insert(tx_id, entry);

            // Update priority index
            by_priority
                .entry(priority.clone())
                .or_insert_with(BTreeSet::new)
                .insert(tx_id);

            // Update sender index
            by_sender
                .entry(sender)
                .or_insert_with(BTreeSet::new)
                .insert(tx_id);

            // Update recipient index
            by_recipient
                .entry(recipient)
                .or_insert_with(BTreeSet::new)
                .insert(tx_id);
        }

        // Update dependency graph
        self.update_dependency_graph(&transaction).await?;

        // Update metrics
        self.update_metrics_on_add(&transaction).await;

        // Update fee estimator
        self.update_fee_estimation(&transaction).await;

        info!(
            "Added transaction {} to mempool (priority: {:?})",
            tx_id, priority
        );
        Ok(true)
    }

    /// Get transactions for batch processing (optimized selection)
    pub async fn get_transactions_for_batch(
        &self,
        max_count: usize,
        max_size_bytes: usize,
    ) -> Vec<Transaction> {
        let start_time = Instant::now();

        if self.config.enable_priority_sorting {
            self.get_priority_ordered_transactions(max_count, max_size_bytes)
                .await
        } else {
            self.get_fifo_transactions(max_count, max_size_bytes).await
        }
    }

    /// Get transactions ordered by priority (highest first)
    async fn get_priority_ordered_transactions(
        &self,
        max_count: usize,
        max_size_bytes: usize,
    ) -> Vec<Transaction> {
        let by_priority = self.by_priority.read().await;
        let transactions = self.transactions.read().await;

        let mut selected = Vec::new();
        let mut total_size = 0;

        // Iterate through priority levels (highest to lowest)
        for (_, tx_ids) in by_priority.iter().rev() {
            for tx_id in tx_ids.iter() {
                if selected.len() >= max_count || total_size >= max_size_bytes {
                    break;
                }

                if let Some(entry) = transactions.get(tx_id) {
                    // Check dependencies are satisfied
                    if self
                        .are_dependencies_satisfied(&entry.dependencies, &selected)
                        .await
                    {
                        total_size += entry.size_bytes;
                        selected.push(entry.transaction.clone());
                    }
                }
            }

            if selected.len() >= max_count || total_size >= max_size_bytes {
                break;
            }
        }

        debug!(
            "Selected {} transactions for batch ({} bytes)",
            selected.len(),
            total_size
        );
        selected
    }

    /// Get transactions in FIFO order
    async fn get_fifo_transactions(
        &self,
        max_count: usize,
        max_size_bytes: usize,
    ) -> Vec<Transaction> {
        let transactions = self.transactions.read().await;

        let mut entries: Vec<_> = transactions.values().collect();
        entries.sort_by_key(|entry| entry.added_at);

        let mut selected = Vec::new();
        let mut total_size = 0;

        for entry in entries {
            if selected.len() >= max_count || total_size >= max_size_bytes {
                break;
            }

            total_size += entry.size_bytes;
            selected.push(entry.transaction.clone());
        }

        selected
    }

    /// Remove transaction from mempool
    pub async fn remove_transaction(&self, tx_id: &Uuid) -> Result<Option<Transaction>> {
        let removed_entry = {
            let mut transactions = self.transactions.write().await;
            transactions.remove(tx_id)
        };

        if let Some(entry) = removed_entry {
            // Remove from all indices
            self.remove_from_indices(tx_id, &entry).await;

            // Update dependency graph
            self.remove_from_dependency_graph(tx_id).await;

            // Update metrics
            self.update_metrics_on_remove(&entry.transaction).await;

            debug!("Removed transaction {} from mempool", tx_id);
            Ok(Some(entry.transaction))
        } else {
            Ok(None)
        }
    }

    /// Check if dependencies are satisfied
    async fn are_dependencies_satisfied(
        &self,
        dependencies: &[Uuid],
        selected: &[Transaction],
    ) -> bool {
        let transactions = self.transactions.read().await;

        for dep_id in dependencies {
            // Check if dependency is in selected transactions
            if !selected.iter().any(|tx| tx.id == *dep_id) {
                // Check if dependency is still in mempool (not processed yet)
                if transactions.contains_key(dep_id) {
                    return false;
                }
                // If not in mempool, assume it's already processed
            }
        }

        true
    }

    /// Calculate transaction priority
    async fn calculate_priority(&self, transaction: &Transaction) -> TransactionPriority {
        let size_bytes = self.estimate_transaction_size(transaction);
        let fee_per_byte = if size_bytes > 0 {
            transaction.amount / size_bytes as u64 // Simplified fee calculation
        } else {
            0
        };

        TransactionPriority {
            fee_per_byte,
            timestamp: Instant::now(),
            is_system: self.is_system_transaction(transaction).await,
        }
    }

    async fn is_system_transaction(&self, _transaction: &Transaction) -> bool {
        // Check if transaction is system-related (governance, rewards, etc.)
        false // Simplified for now
    }

    fn estimate_transaction_size(&self, _transaction: &Transaction) -> usize {
        // Estimate serialized transaction size
        256 // Simplified estimate
    }

    /// Find transaction dependencies
    async fn find_dependencies(&self, transaction: &Transaction) -> Vec<Uuid> {
        let transactions = self.transactions.read().await;
        let mut dependencies = Vec::new();

        // Find transactions from the same sender with lower nonce
        for entry in transactions.values() {
            if entry.transaction.from == transaction.from {
                // In a real implementation, we'd check nonce ordering
                // For now, simplified dependency detection
                if entry.transaction.amount < transaction.amount {
                    dependencies.push(entry.transaction.id);
                }
            }
        }

        dependencies
    }

    /// Update dependency graph
    async fn update_dependency_graph(&self, transaction: &Transaction) -> Result<()> {
        let mut graph = self.dependency_graph.write().await;

        // Add dependencies for this transaction
        let deps = self.find_dependencies(transaction).await;
        if !deps.is_empty() {
            graph.insert(transaction.id, deps.clone());

            // Update dependents for each dependency
            for dep_id in deps {
                graph
                    .entry(dep_id)
                    .or_insert_with(Vec::new)
                    .push(transaction.id);
            }
        }

        Ok(())
    }

    /// Remove transaction from dependency graph
    async fn remove_from_dependency_graph(&self, tx_id: &Uuid) {
        let mut graph = self.dependency_graph.write().await;

        // Remove the transaction's dependencies
        if let Some(deps) = graph.remove(tx_id) {
            // Remove this transaction from dependents lists
            for dep_id in deps {
                if let Some(dependents) = graph.get_mut(&dep_id) {
                    dependents.retain(|id| id != tx_id);
                }
            }
        }

        // Remove from other transaction's dependent lists
        for dependents in graph.values_mut() {
            dependents.retain(|id| id != tx_id);
        }
    }

    /// Remove transaction from all indices
    async fn remove_from_indices(&self, tx_id: &Uuid, entry: &MempoolEntry) {
        let mut by_priority = self.by_priority.write().await;
        let mut by_sender = self.by_sender.write().await;
        let mut by_recipient = self.by_recipient.write().await;

        // Remove from priority index
        if let Some(tx_set) = by_priority.get_mut(&entry.priority) {
            tx_set.remove(tx_id);
            if tx_set.is_empty() {
                by_priority.remove(&entry.priority);
            }
        }

        // Remove from sender index
        if let Some(tx_set) = by_sender.get_mut(&entry.transaction.from) {
            tx_set.remove(tx_id);
            if tx_set.is_empty() {
                by_sender.remove(&entry.transaction.from);
            }
        }

        // Remove from recipient index
        if let Some(tx_set) = by_recipient.get_mut(&entry.transaction.to) {
            tx_set.remove(tx_id);
            if tx_set.is_empty() {
                by_recipient.remove(&entry.transaction.to);
            }
        }
    }

    /// Check capacity limits
    async fn check_capacity_limits(&self, _transaction: &Transaction) -> Result<bool> {
        let transactions = self.transactions.read().await;
        let metrics = self.metrics.read().await;

        Ok(transactions.len() < self.config.max_transactions
            && metrics.total_size_bytes < self.config.max_size_bytes)
    }

    /// Check per-account transaction limits
    async fn check_account_limits(&self, sender: &Address) -> Result<bool> {
        let by_sender = self.by_sender.read().await;

        if let Some(tx_set) = by_sender.get(sender) {
            Ok(tx_set.len() < self.config.max_transactions_per_account)
        } else {
            Ok(true)
        }
    }

    /// Evict low priority transactions to make space
    async fn evict_low_priority_transactions(&self) -> Result<()> {
        let by_priority = self.by_priority.read().await;

        // Find lowest priority transactions to evict
        #[allow(clippy::never_loop)]
        for (priority, tx_ids) in by_priority.iter() {
            for tx_id in tx_ids.iter().take(10) {
                // Evict up to 10 transactions
                let _ = self.remove_transaction(tx_id).await;

                let mut metrics = self.metrics.write().await;
                metrics.eviction_count += 1;
            }
            break; // Only evict from lowest priority level
        }

        Ok(())
    }

    /// Update metrics when adding transaction
    async fn update_metrics_on_add(&self, transaction: &Transaction) {
        let mut metrics = self.metrics.write().await;
        let size = self.estimate_transaction_size(transaction);

        metrics.total_transactions += 1;
        metrics.total_size_bytes += size;

        // Update average fee
        let total_fee =
            metrics.average_fee * (metrics.total_transactions - 1) as u64 + transaction.amount;
        metrics.average_fee = total_fee / metrics.total_transactions as u64;
    }

    /// Update metrics when removing transaction
    async fn update_metrics_on_remove(&self, transaction: &Transaction) {
        let mut metrics = self.metrics.write().await;
        let size = self.estimate_transaction_size(transaction);

        if metrics.total_transactions > 0 {
            metrics.total_transactions -= 1;
            metrics.total_size_bytes = metrics.total_size_bytes.saturating_sub(size);
        }
    }

    /// Update fee estimation
    async fn update_fee_estimation(&self, transaction: &Transaction) {
        let mut estimator = self.fee_estimator.write().await;
        let size = self.estimate_transaction_size(transaction);
        let fee_per_byte = if size > 0 {
            transaction.amount / size as u64
        } else {
            0
        };

        estimator.fee_history.push_back(fee_per_byte);
        if estimator.fee_history.len() > 1000 {
            estimator.fee_history.pop_front();
        }

        // Update network congestion based on mempool size
        let metrics = self.metrics.read().await;
        estimator.network_congestion =
            metrics.total_transactions as f64 / self.config.max_transactions as f64;
    }

    /// Estimate optimal fee for fast confirmation
    pub async fn estimate_fee(&self, target_confirmations: u32) -> u64 {
        let estimator = self.fee_estimator.read().await;

        if estimator.fee_history.is_empty() {
            return 1000; // Default fee
        }

        // Calculate percentile based on target confirmations
        let percentile = match target_confirmations {
            1 => 0.9, // 90th percentile for fast confirmation
            3 => 0.7, // 70th percentile for medium confirmation
            6 => 0.5, // 50th percentile for slow confirmation
            _ => 0.8,
        };

        let mut fees: Vec<u64> = estimator.fee_history.iter().cloned().collect();
        fees.sort_unstable();

        let index = ((fees.len() - 1) as f64 * percentile) as usize;
        let base_fee = fees.get(index).cloned().unwrap_or(1000);

        // Apply congestion multiplier
        let congestion_multiplier = 1.0 + estimator.network_congestion;
        (base_fee as f64 * congestion_multiplier) as u64
    }

    /// Get mempool statistics
    pub async fn get_metrics(&self) -> MempoolMetrics {
        self.metrics.read().await.clone()
    }

    /// Get transactions by sender
    pub async fn get_transactions_by_sender(&self, sender: &Address) -> Vec<Transaction> {
        let by_sender = self.by_sender.read().await;
        let transactions = self.transactions.read().await;

        if let Some(tx_ids) = by_sender.get(sender) {
            tx_ids
                .iter()
                .filter_map(|id| transactions.get(id))
                .map(|entry| entry.transaction.clone())
                .collect()
        } else {
            Vec::new()
        }
    }

    /// Background cleanup loop
    async fn cleanup_loop(&self) {
        let mut interval = tokio::time::interval(self.config.cleanup_interval);

        loop {
            interval.tick().await;

            if let Err(e) = self.cleanup_expired_transactions().await {
                error!("Error during mempool cleanup: {}", e);
            }
        }
    }

    /// Remove expired transactions
    async fn cleanup_expired_transactions(&self) -> Result<()> {
        let now = Instant::now();
        let mut expired_tx_ids = Vec::new();

        {
            let transactions = self.transactions.read().await;
            for (tx_id, entry) in transactions.iter() {
                if now.duration_since(entry.added_at) > self.config.transaction_ttl {
                    expired_tx_ids.push(*tx_id);
                }
            }
        }

        for tx_id in expired_tx_ids {
            let _ = self.remove_transaction(&tx_id).await;
            debug!("Removed expired transaction: {}", tx_id);
        }

        Ok(())
    }
}

impl Clone for AdvancedMempool {
    fn clone(&self) -> Self {
        Self {
            config: self.config.clone(),
            transactions: Arc::clone(&self.transactions),
            by_priority: Arc::clone(&self.by_priority),
            by_sender: Arc::clone(&self.by_sender),
            by_recipient: Arc::clone(&self.by_recipient),
            dependency_graph: Arc::clone(&self.dependency_graph),
            metrics: Arc::clone(&self.metrics),
            fee_estimator: Arc::clone(&self.fee_estimator),
        }
    }
}
