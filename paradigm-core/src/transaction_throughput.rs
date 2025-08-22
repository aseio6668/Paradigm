use anyhow::Result;
use dashmap::DashMap;
use rayon::prelude::*;
use serde::{Deserialize, Serialize};
/// High-throughput transaction processing optimization for Paradigm
use std::collections::{BTreeMap, HashMap, VecDeque};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::{mpsc, oneshot, RwLock, Semaphore};
use tokio::time::timeout;
use uuid::Uuid;

use crate::crypto_optimization::OptimizedSignatureEngine;
use crate::storage::ParadigmStorage;
use crate::transaction::Transaction;
use crate::Address;

/// Transaction processing pipeline stages
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ProcessingStage {
    Received,
    Validated,
    Queued,
    Processing,
    Completed,
    Failed,
}

/// Transaction processing result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransactionResult {
    pub transaction_id: Uuid,
    pub success: bool,
    pub stage: ProcessingStage,
    pub processing_time_ms: u64,
    pub gas_used: u64,
    pub error_message: Option<String>,
    pub block_number: Option<u64>,
}

/// Transaction pool for managing pending transactions
#[derive(Debug)]
pub struct TransactionPool {
    pending: Arc<DashMap<Uuid, PendingTransaction>>,
    by_nonce: Arc<DashMap<Address, BTreeMap<u64, Uuid>>>, // Address -> nonce -> tx_id
    by_gas_price: Arc<RwLock<BTreeMap<u64, Vec<Uuid>>>>,  // gas_price -> tx_ids (sorted)
    max_pool_size: usize,
    max_per_account: usize,
    stats: Arc<RwLock<PoolStats>>,
}

#[derive(Debug, Clone)]
pub struct PendingTransaction {
    pub transaction: Transaction,
    pub received_at: Instant,
    pub gas_price: u64,
    pub retries: u32,
    pub stage: ProcessingStage,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PoolStats {
    pub total_received: u64,
    pub total_processed: u64,
    pub total_rejected: u64,
    pub current_pool_size: usize,
    pub average_processing_time_ms: f64,
    pub throughput_tps: f64,
}

impl TransactionPool {
    pub fn new(max_pool_size: usize, max_per_account: usize) -> Self {
        Self {
            pending: Arc::new(DashMap::new()),
            by_nonce: Arc::new(DashMap::new()),
            by_gas_price: Arc::new(RwLock::new(BTreeMap::new())),
            max_pool_size,
            max_per_account,
            stats: Arc::new(RwLock::new(PoolStats::default())),
        }
    }

    /// Add transaction to pool with validation
    pub async fn add_transaction(&self, transaction: Transaction) -> Result<bool> {
        // Check pool capacity
        if self.pending.len() >= self.max_pool_size {
            return Ok(false);
        }

        // Check per-account limit
        if let Some(account_nonces) = self.by_nonce.get(&transaction.from) {
            if account_nonces.len() >= self.max_per_account {
                return Ok(false);
            }
        }

        // Check for duplicate nonce (replace-by-fee logic)
        let should_replace =
            if let Some(mut account_nonces) = self.by_nonce.get_mut(&transaction.from) {
                if let Some(existing_tx_id) = account_nonces.get(&transaction.nonce) {
                    // Check if new transaction has higher gas price
                    if let Some(existing_tx) = self.pending.get(existing_tx_id) {
                        transaction.fee > existing_tx.gas_price
                    } else {
                        true
                    }
                } else {
                    false
                }
            } else {
                false
            };

        if should_replace {
            // Remove existing transaction
            if let Some(account_nonces) = self.by_nonce.get(&transaction.from) {
                if let Some(old_tx_id) = account_nonces.get(&transaction.nonce).cloned() {
                    self.remove_transaction_internal(&old_tx_id).await;
                }
            }
        }

        // Add new transaction
        let pending_tx = PendingTransaction {
            gas_price: transaction.fee, // Using fee as gas price for simplicity
            received_at: Instant::now(),
            retries: 0,
            stage: ProcessingStage::Received,
            transaction: transaction.clone(),
        };

        self.pending.insert(transaction.id, pending_tx);

        // Update nonce mapping
        self.by_nonce
            .entry(transaction.from.clone())
            .or_insert_with(BTreeMap::new)
            .insert(transaction.nonce, transaction.id);

        // Update gas price index
        let mut gas_price_index = self.by_gas_price.write().await;
        gas_price_index
            .entry(transaction.fee)
            .or_insert_with(Vec::new)
            .push(transaction.id);

        // Update stats
        let mut stats = self.stats.write().await;
        stats.total_received += 1;
        stats.current_pool_size = self.pending.len();

        Ok(true)
    }

    /// Get transactions ready for processing (sorted by gas price and nonce)
    pub async fn get_ready_transactions(&self, limit: usize) -> Vec<Transaction> {
        let mut ready_transactions = Vec::new();
        let gas_price_index = self.by_gas_price.read().await;

        // Iterate from highest gas price to lowest
        for (_, tx_ids) in gas_price_index.iter().rev() {
            for tx_id in tx_ids {
                if ready_transactions.len() >= limit {
                    return ready_transactions;
                }

                if let Some(pending_tx) = self.pending.get(tx_id) {
                    if pending_tx.stage == ProcessingStage::Received {
                        // Check if this is the next nonce for the account
                        if self.is_next_nonce(&pending_tx.transaction).await {
                            ready_transactions.push(pending_tx.transaction.clone());
                        }
                    }
                }
            }
        }

        ready_transactions
    }

    /// Check if transaction has the next expected nonce for the account
    async fn is_next_nonce(&self, transaction: &Transaction) -> bool {
        // In a real implementation, you'd check against the account's current nonce
        // For simplicity, we'll just check if it's the lowest nonce in the pool for this account
        if let Some(account_nonces) = self.by_nonce.get(&transaction.from) {
            if let Some((&lowest_nonce, _)) = account_nonces.iter().next() {
                return transaction.nonce == lowest_nonce;
            }
        }
        true
    }

    /// Remove transaction from pool
    pub async fn remove_transaction(&self, tx_id: &Uuid) -> Option<Transaction> {
        self.remove_transaction_internal(tx_id).await
    }

    async fn remove_transaction_internal(&self, tx_id: &Uuid) -> Option<Transaction> {
        if let Some((_, pending_tx)) = self.pending.remove(tx_id) {
            let transaction = pending_tx.transaction.clone();

            // Remove from nonce mapping
            if let Some(mut account_nonces) = self.by_nonce.get_mut(&transaction.from) {
                account_nonces.remove(&transaction.nonce);
                if account_nonces.is_empty() {
                    drop(account_nonces);
                    self.by_nonce.remove(&transaction.from);
                }
            }

            // Remove from gas price index
            let mut gas_price_index = self.by_gas_price.write().await;
            if let Some(tx_list) = gas_price_index.get_mut(&transaction.fee) {
                tx_list.retain(|id| id != tx_id);
                if tx_list.is_empty() {
                    gas_price_index.remove(&transaction.fee);
                }
            }

            // Update stats
            let mut stats = self.stats.write().await;
            stats.current_pool_size = self.pending.len();

            Some(transaction)
        } else {
            None
        }
    }

    /// Update transaction stage
    pub async fn update_transaction_stage(&self, tx_id: &Uuid, stage: ProcessingStage) -> bool {
        if let Some(mut pending_tx) = self.pending.get_mut(tx_id) {
            pending_tx.stage = stage;
            true
        } else {
            false
        }
    }

    /// Get pool statistics
    pub async fn get_stats(&self) -> PoolStats {
        let mut stats = self.stats.read().await.clone();
        stats.current_pool_size = self.pending.len();
        stats
    }

    /// Clean up expired transactions
    pub async fn cleanup_expired(&self, max_age: Duration) -> usize {
        let now = Instant::now();
        let mut expired_ids = Vec::new();

        for entry in self.pending.iter() {
            if now.duration_since(entry.received_at) > max_age {
                expired_ids.push(*entry.key());
            }
        }

        let count = expired_ids.len();
        for tx_id in expired_ids {
            self.remove_transaction_internal(&tx_id).await;
        }

        count
    }
}

/// Parallel transaction processor with optimized pipeline
#[derive(Debug)]
pub struct ParallelTransactionProcessor {
    pool: Arc<TransactionPool>,
    storage: Arc<ParadigmStorage>,
    signature_engine: Arc<OptimizedSignatureEngine>,
    processing_semaphore: Arc<Semaphore>,
    result_sender: mpsc::UnboundedSender<TransactionResult>,
    result_receiver: Arc<RwLock<mpsc::UnboundedReceiver<TransactionResult>>>,
    processor_stats: Arc<RwLock<ProcessorStats>>,
    config: ProcessorConfig,
    is_running: Arc<RwLock<bool>>,
}

#[derive(Debug, Clone)]
pub struct ProcessorConfig {
    pub max_concurrent_transactions: usize,
    pub batch_size: usize,
    pub processing_timeout_ms: u64,
    pub validation_workers: usize,
    pub execution_workers: usize,
    pub enable_parallel_validation: bool,
    pub enable_speculative_execution: bool,
}

impl Default for ProcessorConfig {
    fn default() -> Self {
        Self {
            max_concurrent_transactions: 1000,
            batch_size: 100,
            processing_timeout_ms: 5000,
            validation_workers: num_cpus::get(),
            execution_workers: num_cpus::get() / 2,
            enable_parallel_validation: true,
            enable_speculative_execution: true,
        }
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ProcessorStats {
    pub transactions_processed: u64,
    pub transactions_failed: u64,
    pub average_processing_time_ms: f64,
    pub current_tps: f64,
    pub peak_tps: f64,
    pub validation_rate: f64,
    pub execution_rate: f64,
    pub parallelism_efficiency: f64,
}

impl ParallelTransactionProcessor {
    pub async fn new(
        storage: Arc<ParadigmStorage>,
        signature_engine: Arc<OptimizedSignatureEngine>,
        config: ProcessorConfig,
    ) -> Result<Self> {
        let pool = Arc::new(TransactionPool::new(10000, 100));
        let (result_sender, result_receiver) = mpsc::unbounded_channel();

        Ok(Self {
            pool,
            storage,
            signature_engine,
            processing_semaphore: Arc::new(Semaphore::new(config.max_concurrent_transactions)),
            result_sender,
            result_receiver: Arc::new(RwLock::new(result_receiver)),
            processor_stats: Arc::new(RwLock::new(ProcessorStats::default())),
            config,
            is_running: Arc::new(RwLock::new(false)),
        })
    }

    /// Start the parallel processing pipeline
    pub async fn start(&self) -> Result<()> {
        *self.is_running.write().await = true;

        // Start transaction processing pipeline
        self.start_validation_pipeline().await?;
        self.start_execution_pipeline().await?;
        self.start_cleanup_task().await?;

        tracing::info!("Parallel transaction processor started with {} validation workers and {} execution workers",
            self.config.validation_workers, self.config.execution_workers);

        Ok(())
    }

    /// Stop the processor
    pub async fn stop(&self) {
        *self.is_running.write().await = false;
        tracing::info!("Parallel transaction processor stopped");
    }

    /// Submit transaction for processing
    pub async fn submit_transaction(&self, transaction: Transaction) -> Result<bool> {
        self.pool.add_transaction(transaction).await
    }

    /// Get next processed transaction result
    pub async fn get_result(&self) -> Option<TransactionResult> {
        let mut receiver = self.result_receiver.write().await;
        receiver.recv().await
    }

    /// Start validation pipeline
    async fn start_validation_pipeline(&self) -> Result<()> {
        let pool = self.pool.clone();
        let signature_engine = self.signature_engine.clone();
        let is_running = self.is_running.clone();
        let config = self.config.clone();
        let stats = self.processor_stats.clone();

        for worker_id in 0..config.validation_workers {
            let pool_worker = pool.clone();
            let sig_engine_worker = signature_engine.clone();
            let is_running_worker = is_running.clone();
            let stats_worker = stats.clone();
            let batch_size = config.batch_size;

            tokio::spawn(async move {
                tracing::debug!("Validation worker {} started", worker_id);

                while *is_running_worker.read().await {
                    // Get batch of ready transactions
                    let transactions = pool_worker.get_ready_transactions(batch_size).await;

                    if transactions.is_empty() {
                        tokio::time::sleep(Duration::from_millis(10)).await;
                        continue;
                    }

                    // Parallel validation
                    let validation_start = Instant::now();
                    let validation_results =
                        Self::validate_transactions_batch(&transactions, &sig_engine_worker).await;

                    let validation_time = validation_start.elapsed();

                    // Update transaction stages based on validation results
                    for (i, result) in validation_results.iter().enumerate() {
                        let tx = &transactions[i];
                        let new_stage = if *result {
                            ProcessingStage::Validated
                        } else {
                            ProcessingStage::Failed
                        };
                        pool_worker
                            .update_transaction_stage(&tx.id, new_stage)
                            .await;
                    }

                    // Update stats
                    {
                        let mut stats_guard = stats_worker.write().await;
                        stats_guard.validation_rate =
                            transactions.len() as f64 / validation_time.as_secs_f64();
                    }
                }

                tracing::debug!("Validation worker {} stopped", worker_id);
            });
        }

        Ok(())
    }

    /// Start execution pipeline
    async fn start_execution_pipeline(&self) -> Result<()> {
        let pool = self.pool.clone();
        let storage = self.storage.clone();
        let semaphore = self.processing_semaphore.clone();
        let result_sender = self.result_sender.clone();
        let is_running = self.is_running.clone();
        let config = self.config.clone();
        let stats = self.processor_stats.clone();

        for worker_id in 0..config.execution_workers {
            let pool_worker = pool.clone();
            let storage_worker = storage.clone();
            let semaphore_worker = semaphore.clone();
            let result_sender_worker = result_sender.clone();
            let is_running_worker = is_running.clone();
            let stats_worker = stats.clone();
            let timeout_duration = Duration::from_millis(config.processing_timeout_ms);

            tokio::spawn(async move {
                tracing::debug!("Execution worker {} started", worker_id);

                while *is_running_worker.read().await {
                    // Get validated transactions
                    let transactions = pool_worker.get_ready_transactions(10).await; // Smaller batches for execution

                    if transactions.is_empty() {
                        tokio::time::sleep(Duration::from_millis(10)).await;
                        continue;
                    }

                    for transaction in transactions {
                        // Check if transaction is validated
                        if let Some(pending) = pool_worker.pending.get(&transaction.id) {
                            if pending.stage != ProcessingStage::Validated {
                                continue;
                            }
                        } else {
                            continue;
                        }

                        // Acquire semaphore permit
                        let permit = match semaphore_worker.clone().try_acquire_owned() {
                            Ok(permit) => permit,
                            Err(_) => continue, // No permits available
                        };

                        let pool_exec = pool_worker.clone();
                        let storage_exec = storage_worker.clone();
                        let result_sender_exec = result_sender_worker.clone();
                        let stats_exec = stats_worker.clone();
                        let tx_id = transaction.id;

                        tokio::spawn(async move {
                            let execution_start = Instant::now();

                            // Update stage to processing
                            pool_exec
                                .update_transaction_stage(&tx_id, ProcessingStage::Processing)
                                .await;

                            // Execute transaction with timeout
                            let result = timeout(
                                timeout_duration,
                                Self::execute_transaction(transaction.clone(), &storage_exec),
                            )
                            .await;

                            let execution_time = execution_start.elapsed();

                            let tx_result = match result {
                                Ok(Ok(_)) => {
                                    pool_exec
                                        .update_transaction_stage(
                                            &tx_id,
                                            ProcessingStage::Completed,
                                        )
                                        .await;
                                    TransactionResult {
                                        transaction_id: tx_id,
                                        success: true,
                                        stage: ProcessingStage::Completed,
                                        processing_time_ms: execution_time.as_millis() as u64,
                                        gas_used: transaction.fee, // Simplified
                                        error_message: None,
                                        block_number: Some(1), // Simplified
                                    }
                                }
                                Ok(Err(e)) => {
                                    pool_exec
                                        .update_transaction_stage(&tx_id, ProcessingStage::Failed)
                                        .await;
                                    TransactionResult {
                                        transaction_id: tx_id,
                                        success: false,
                                        stage: ProcessingStage::Failed,
                                        processing_time_ms: execution_time.as_millis() as u64,
                                        gas_used: 0,
                                        error_message: Some(e.to_string()),
                                        block_number: None,
                                    }
                                }
                                Err(_) => {
                                    pool_exec
                                        .update_transaction_stage(&tx_id, ProcessingStage::Failed)
                                        .await;
                                    TransactionResult {
                                        transaction_id: tx_id,
                                        success: false,
                                        stage: ProcessingStage::Failed,
                                        processing_time_ms: execution_time.as_millis() as u64,
                                        gas_used: 0,
                                        error_message: Some("Execution timeout".to_string()),
                                        block_number: None,
                                    }
                                }
                            };

                            // Remove from pool
                            pool_exec.remove_transaction(&tx_id).await;

                            // Update stats
                            {
                                let mut stats_guard = stats_exec.write().await;
                                if tx_result.success {
                                    stats_guard.transactions_processed += 1;
                                } else {
                                    stats_guard.transactions_failed += 1;
                                }

                                let total_processed = stats_guard.transactions_processed
                                    + stats_guard.transactions_failed;
                                if total_processed > 0 {
                                    stats_guard.average_processing_time_ms = (stats_guard
                                        .average_processing_time_ms
                                        * (total_processed - 1) as f64
                                        + execution_time.as_millis() as f64)
                                        / total_processed as f64;
                                }
                            }

                            // Send result
                            let _ = result_sender_exec.send(tx_result);

                            // Release permit
                            drop(permit);
                        });
                    }
                }

                tracing::debug!("Execution worker {} stopped", worker_id);
            });
        }

        Ok(())
    }

    /// Start cleanup task for expired transactions
    async fn start_cleanup_task(&self) -> Result<()> {
        let pool = self.pool.clone();
        let is_running = self.is_running.clone();

        tokio::spawn(async move {
            while *is_running.read().await {
                // Clean up transactions older than 5 minutes
                let expired_count = pool.cleanup_expired(Duration::from_secs(300)).await;
                if expired_count > 0 {
                    tracing::debug!("Cleaned up {} expired transactions", expired_count);
                }

                tokio::time::sleep(Duration::from_secs(60)).await; // Run every minute
            }
        });

        Ok(())
    }

    /// Validate transactions in parallel
    async fn validate_transactions_batch(
        transactions: &[Transaction],
        signature_engine: &OptimizedSignatureEngine,
    ) -> Vec<bool> {
        // Create verification data for batch processing
        let verifications: Vec<_> = transactions
            .iter()
            .map(|tx| {
                // Create message to verify (simplified)
                let message =
                    format!("{}:{}:{}:{}", tx.from, tx.to, tx.amount, tx.nonce).into_bytes();
                (
                    message,
                    tx.signature.clone(),
                    get_public_key_for_address(&tx.from),
                )
            })
            .collect();

        // Batch verify signatures
        signature_engine
            .batch_verify_signatures(&verifications)
            .await
            .unwrap_or_else(|_| vec![false; transactions.len()])
    }

    /// Execute transaction (simplified implementation)
    async fn execute_transaction(
        transaction: Transaction,
        storage: &ParadigmStorage,
    ) -> Result<()> {
        // Simulate transaction execution
        tokio::time::sleep(Duration::from_millis(1)).await;

        // Store transaction in database
        storage.store_transaction_optimized(&transaction).await?;

        // Update balances (simplified)
        let from_balance = storage.get_balance(&transaction.from).await?;
        let to_balance = storage.get_balance(&transaction.to).await?;

        if from_balance >= transaction.amount + transaction.fee {
            storage
                .update_balance(
                    &transaction.from,
                    from_balance - transaction.amount - transaction.fee,
                )
                .await?;
            storage
                .update_balance(&transaction.to, to_balance + transaction.amount)
                .await?;
            Ok(())
        } else {
            Err(anyhow::anyhow!("Insufficient balance"))
        }
    }

    /// Get comprehensive statistics
    pub async fn get_stats(&self) -> ProcessorStats {
        self.processor_stats.read().await.clone()
    }

    /// Get pool statistics
    pub async fn get_pool_stats(&self) -> PoolStats {
        self.pool.get_stats().await
    }

    /// Force process all pending transactions (for testing/emergency)
    pub async fn force_process_all(&self) -> Result<usize> {
        let transactions = self.pool.get_ready_transactions(usize::MAX).await;
        let count = transactions.len();

        for transaction in transactions {
            self.submit_transaction(transaction).await?;
        }

        Ok(count)
    }
}

// Helper function to get public key for address (simplified)
fn get_public_key_for_address(_address: &Address) -> ed25519_dalek::VerifyingKey {
    // In a real implementation, this would look up the public key
    // For now, return a dummy key
    use rand::rngs::OsRng;
    ed25519_dalek::SigningKey::from_bytes(&[0u8; 32]).verifying_key()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::crypto_optimization::CryptoEngine;
    use crate::storage::{ParadigmStorage, StorageConfig};

    #[tokio::test]
    async fn test_transaction_pool() {
        let pool = TransactionPool::new(10, 5);

        let tx1 = create_test_transaction(1);
        let tx2 = create_test_transaction(2);

        assert!(pool.add_transaction(tx1.clone()).await.unwrap());
        assert!(pool.add_transaction(tx2.clone()).await.unwrap());

        let ready = pool.get_ready_transactions(10).await;
        assert_eq!(ready.len(), 2);

        let stats = pool.get_stats().await;
        assert_eq!(stats.total_received, 2);
        assert_eq!(stats.current_pool_size, 2);
    }

    #[tokio::test]
    async fn test_transaction_pool_nonce_ordering() {
        let pool = TransactionPool::new(10, 5);
        let address = Address([1u8; 32]);

        // Add transactions with nonces 2, 1, 3 (out of order)
        let tx2 = create_test_transaction_with_nonce(address.clone(), 2);
        let tx1 = create_test_transaction_with_nonce(address.clone(), 1);
        let tx3 = create_test_transaction_with_nonce(address.clone(), 3);

        pool.add_transaction(tx2).await.unwrap();
        pool.add_transaction(tx1).await.unwrap();
        pool.add_transaction(tx3).await.unwrap();

        let ready = pool.get_ready_transactions(1).await;
        assert_eq!(ready.len(), 1);
        assert_eq!(ready[0].nonce, 1); // Should get the lowest nonce first
    }

    #[tokio::test]
    async fn test_parallel_processor() {
        let storage = Arc::new(ParadigmStorage::new("sqlite::memory:").await.unwrap());
        let crypto_engine = Arc::new(CryptoEngine::new(2).unwrap());
        let config = ProcessorConfig::default();

        let processor =
            ParallelTransactionProcessor::new(storage, crypto_engine.signatures.clone(), config)
                .await
                .unwrap();

        processor.start().await.unwrap();

        // Submit test transactions
        let tx1 = create_test_transaction(1);
        let tx2 = create_test_transaction(2);

        assert!(processor.submit_transaction(tx1).await.unwrap());
        assert!(processor.submit_transaction(tx2).await.unwrap());

        // Wait a bit for processing
        tokio::time::sleep(Duration::from_millis(100)).await;

        let stats = processor.get_stats().await;
        let pool_stats = processor.get_pool_stats().await;

        assert!(pool_stats.total_received >= 2);

        processor.stop().await;
    }

    fn create_test_transaction(index: u64) -> Transaction {
        Transaction {
            id: Uuid::new_v4(),
            from: Address([index as u8; 32]),
            to: Address([(index + 1) as u8; 32]),
            amount: 100,
            fee: 10,
            timestamp: chrono::Utc::now(),
            signature: vec![index as u8; 64],
            nonce: index,
        }
    }

    fn create_test_transaction_with_nonce(from: Address, nonce: u64) -> Transaction {
        Transaction {
            id: Uuid::new_v4(),
            from,
            to: Address([255u8; 32]),
            amount: 100,
            fee: 10,
            timestamp: chrono::Utc::now(),
            signature: vec![nonce as u8; 64],
            nonce,
        }
    }
}
