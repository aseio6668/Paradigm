use anyhow::Result;
use chrono::{DateTime, Duration as ChronoDuration, Utc};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, VecDeque};
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

use crate::transaction::Transaction;
use crate::Address;

/// Ephemeral storage system that automatically forgets transaction history
/// Maintains only current balances and recent activity for privacy
#[derive(Debug)]
pub struct EphemeralStorage {
    /// Current balances - these persist
    current_balances: Arc<RwLock<HashMap<String, u64>>>,
    /// Recent transactions - automatically expire
    recent_transactions: Arc<RwLock<VecDeque<EphemeralTransaction>>>,
    /// Transaction lookup index
    transaction_index: Arc<RwLock<HashMap<Uuid, usize>>>,
    /// Configuration
    config: EphemeralConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EphemeralConfig {
    /// How long to keep recent transactions (in minutes)
    pub transaction_memory_minutes: i64,
    /// Maximum number of recent transactions to keep
    pub max_recent_transactions: usize,
    /// How often to clean up expired transactions (in seconds)
    pub cleanup_interval_seconds: u64,
    /// Enable transaction mixing for privacy
    pub enable_mixing: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EphemeralTransaction {
    pub id: Uuid,
    pub from_hash: String, // Hash of sender address for privacy
    pub to_hash: String,   // Hash of receiver address for privacy
    pub amount: u64,
    pub timestamp: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
    pub message_hash: Option<String>,
    pub mixed: bool, // Whether this transaction was mixed with others
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BalanceState {
    pub address_hash: String,
    pub balance: u64,
    pub last_updated: DateTime<Utc>,
    pub transaction_count: u64, // Total transactions processed (for stats only)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EphemeralStats {
    pub total_addresses: usize,
    pub recent_transactions: usize,
    pub expired_transactions_cleaned: u64,
    pub oldest_transaction: Option<DateTime<Utc>>,
    pub newest_transaction: Option<DateTime<Utc>>,
    pub privacy_level: PrivacyLevel,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PrivacyLevel {
    Standard, // Recent transactions visible, addresses hashed
    Enhanced, // Transactions mixed, minimal history
    Maximum,  // Ultra-short memory, full anonymization
}

impl Default for EphemeralConfig {
    fn default() -> Self {
        Self {
            transaction_memory_minutes: 30, // Keep transactions for 30 minutes
            max_recent_transactions: 1000,  // Max 1000 recent transactions
            cleanup_interval_seconds: 300,  // Clean up every 5 minutes
            enable_mixing: true,
        }
    }
}

impl EphemeralStorage {
    pub fn new() -> Self {
        Self::with_config(EphemeralConfig::default())
    }

    pub fn with_config(config: EphemeralConfig) -> Self {
        Self {
            current_balances: Arc::new(RwLock::new(HashMap::new())),
            recent_transactions: Arc::new(RwLock::new(VecDeque::new())),
            transaction_index: Arc::new(RwLock::new(HashMap::new())),
            config,
        }
    }

    /// Store a transaction with automatic expiry
    pub async fn store_transaction(&self, transaction: &Transaction) -> Result<()> {
        // Validate transaction signature before storing
        if !transaction.signature.is_empty() {
            let from_address_bytes = transaction.from.as_bytes();
            if from_address_bytes.len() >= 32 {
                let mut key_bytes = [0u8; 32];
                key_bytes.copy_from_slice(&from_address_bytes[..32]);

                if let Ok(public_key) = ed25519_dalek::VerifyingKey::from_bytes(&key_bytes) {
                    if let Err(e) = transaction.validate(&public_key) {
                        tracing::warn!("Ephemeral storage: Transaction validation failed: {}", e);
                        return Err(anyhow::anyhow!(
                            "Transaction signature validation failed: {}",
                            e
                        ));
                    }
                    tracing::debug!(
                        "✅ Ephemeral transaction {} signature validated",
                        transaction.id
                    );
                }
            }
        }

        let expires_at =
            Utc::now() + ChronoDuration::minutes(self.config.transaction_memory_minutes);

        let ephemeral_tx = EphemeralTransaction {
            id: transaction.id,
            from_hash: self.hash_address(&transaction.from),
            to_hash: self.hash_address(&transaction.to),
            amount: transaction.amount,
            timestamp: Utc::now(),
            expires_at,
            message_hash: transaction.message.as_ref().map(|m| self.hash_string(m)),
            mixed: false, // Will be set during mixing process
        };

        // Update balances
        self.update_balances(&transaction.from, &transaction.to, transaction.amount)
            .await?;

        // Store recent transaction
        let mut recent_txs = self.recent_transactions.write().await;
        let mut tx_index = self.transaction_index.write().await;

        // Remove oldest transactions if we exceed the limit
        while recent_txs.len() >= self.config.max_recent_transactions {
            if let Some(old_tx) = recent_txs.pop_front() {
                tx_index.remove(&old_tx.id);
            }
        }

        // Add new transaction
        let tx_position = recent_txs.len();
        recent_txs.push_back(ephemeral_tx);
        tx_index.insert(transaction.id, tx_position);

        tracing::debug!(
            "Stored ephemeral transaction {} (expires: {})",
            transaction.id,
            expires_at
        );

        Ok(())
    }

    /// Get current balance without revealing transaction history
    pub async fn get_balance(&self, address: &Address) -> Result<u64> {
        let address_hash = self.hash_address(address);
        let balances = self.current_balances.read().await;

        Ok(balances.get(&address_hash).copied().unwrap_or(0))
    }

    /// Get recent transactions (limited and hashed for privacy)
    pub async fn get_recent_transactions(
        &self,
        limit: Option<usize>,
    ) -> Result<Vec<EphemeralTransaction>> {
        let recent_txs = self.recent_transactions.read().await;
        let max_return = limit.unwrap_or(50).min(100); // Cap at 100 for performance

        let transactions: Vec<EphemeralTransaction> = recent_txs
            .iter()
            .rev() // Most recent first
            .take(max_return)
            .filter(|tx| tx.expires_at > Utc::now()) // Only non-expired
            .cloned()
            .collect();

        Ok(transactions)
    }

    /// Get transactions for a specific address (hashed for privacy)
    pub async fn get_address_transactions(
        &self,
        address: &Address,
        limit: Option<usize>,
    ) -> Result<Vec<EphemeralTransaction>> {
        let address_hash = self.hash_address(address);
        let recent_txs = self.recent_transactions.read().await;
        let max_return = limit.unwrap_or(20).min(50); // Lower limit for address-specific queries

        let transactions: Vec<EphemeralTransaction> = recent_txs
            .iter()
            .filter(|tx| {
                tx.expires_at > Utc::now()
                    && (tx.from_hash == address_hash || tx.to_hash == address_hash)
            })
            .rev() // Most recent first
            .take(max_return)
            .cloned()
            .collect();

        Ok(transactions)
    }

    /// Clean up expired transactions
    pub async fn cleanup_expired(&self) -> Result<u64> {
        let now = Utc::now();
        let mut recent_txs = self.recent_transactions.write().await;
        let mut tx_index = self.transaction_index.write().await;
        let mut removed_count = 0u64;

        // Remove expired transactions from the front of the queue
        while let Some(front_tx) = recent_txs.front() {
            if front_tx.expires_at <= now {
                let expired_tx = recent_txs.pop_front().unwrap();
                tx_index.remove(&expired_tx.id);
                removed_count += 1;
            } else {
                break; // Since we add in order, once we hit a non-expired tx, we're done
            }
        }

        // Rebuild index to fix positions after removals
        tx_index.clear();
        for (pos, tx) in recent_txs.iter().enumerate() {
            tx_index.insert(tx.id, pos);
        }

        if removed_count > 0 {
            tracing::debug!("Cleaned up {} expired transactions", removed_count);
        }

        Ok(removed_count)
    }

    /// Start automatic cleanup process
    pub async fn start_auto_cleanup(&self) -> Result<()> {
        let recent_txs = self.recent_transactions.clone();
        let tx_index = self.transaction_index.clone();
        let cleanup_interval = self.config.cleanup_interval_seconds;

        tokio::spawn(async move {
            let mut interval =
                tokio::time::interval(tokio::time::Duration::from_secs(cleanup_interval));

            loop {
                interval.tick().await;

                let now = Utc::now();
                let mut recent = recent_txs.write().await;
                let mut index = tx_index.write().await;
                let mut removed_count = 0u64;

                // Clean up expired transactions
                while let Some(front_tx) = recent.front() {
                    if front_tx.expires_at <= now {
                        let expired_tx = recent.pop_front().unwrap();
                        index.remove(&expired_tx.id);
                        removed_count += 1;
                    } else {
                        break;
                    }
                }

                // Rebuild index
                if removed_count > 0 {
                    index.clear();
                    for (pos, tx) in recent.iter().enumerate() {
                        index.insert(tx.id, pos);
                    }
                    tracing::debug!(
                        "Auto-cleanup removed {} expired transactions",
                        removed_count
                    );
                }
            }
        });

        tracing::info!(
            "Started ephemeral storage auto-cleanup (interval: {}s)",
            cleanup_interval
        );
        Ok(())
    }

    /// Get storage statistics
    pub async fn get_stats(&self) -> Result<EphemeralStats> {
        let balances = self.current_balances.read().await;
        let recent_txs = self.recent_transactions.read().await;

        let oldest_tx = recent_txs.front().map(|tx| tx.timestamp);
        let newest_tx = recent_txs.back().map(|tx| tx.timestamp);

        let stats = EphemeralStats {
            total_addresses: balances.len(),
            recent_transactions: recent_txs.len(),
            expired_transactions_cleaned: 0, // TODO: Track this in a counter
            oldest_transaction: oldest_tx,
            newest_transaction: newest_tx,
            privacy_level: self.calculate_privacy_level(),
        };

        Ok(stats)
    }

    /// Mix transactions for enhanced privacy
    pub async fn mix_transactions(&self, _count: usize) -> Result<u64> {
        if !self.config.enable_mixing {
            return Ok(0);
        }

        // TODO: Implement transaction mixing algorithm
        // This would group transactions together and obscure their individual details
        tracing::debug!("Transaction mixing not yet implemented");
        Ok(0)
    }

    /// Force cleanup of all expired data
    pub async fn force_cleanup(&self) -> Result<u64> {
        let removed = self.cleanup_expired().await?;
        tracing::info!("Force cleanup completed: {} transactions removed", removed);
        Ok(removed)
    }

    // Private helper methods

    async fn update_balances(&self, from: &Address, to: &Address, amount: u64) -> Result<()> {
        let from_hash = self.hash_address(from);
        let to_hash = self.hash_address(to);

        let mut balances = self.current_balances.write().await;

        // Update sender balance
        let from_balance = balances.get(&from_hash).copied().unwrap_or(0);
        if from_balance >= amount {
            balances.insert(from_hash, from_balance - amount);
        } else {
            return Err(anyhow::anyhow!("Insufficient balance"));
        }

        // Update receiver balance
        let to_balance = balances.get(&to_hash).copied().unwrap_or(0);
        balances.insert(to_hash, to_balance + amount);

        Ok(())
    }

    fn hash_address(&self, address: &Address) -> String {
        let mut hasher = blake3::Hasher::new();
        hasher.update(address.as_bytes());
        hex::encode(hasher.finalize().as_bytes())
    }

    fn hash_string(&self, text: &str) -> String {
        let mut hasher = blake3::Hasher::new();
        hasher.update(text.as_bytes());
        hex::encode(hasher.finalize().as_bytes())
    }

    fn calculate_privacy_level(&self) -> PrivacyLevel {
        match self.config.transaction_memory_minutes {
            0..=15 => PrivacyLevel::Maximum,   // 15 minutes or less
            16..=60 => PrivacyLevel::Enhanced, // 16-60 minutes
            _ => PrivacyLevel::Standard,       // More than 1 hour
        }
    }
}

impl Default for EphemeralStorage {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{transaction::Transaction, Address, AddressExt};
    use ed25519_dalek::SigningKey;

    #[tokio::test]
    async fn test_ephemeral_storage_basic() {
        let storage = EphemeralStorage::new();

        // Create test addresses
        let keypair1 = SigningKey::from_bytes(&rand::random());
        let keypair2 = SigningKey::from_bytes(&rand::random());
        let addr1 = Address::from_public_key(&keypair1.verifying_key());
        let addr2 = Address::from_public_key(&keypair2.verifying_key());

        // Initial balances should be 0
        assert_eq!(storage.get_balance(&addr1).await.unwrap(), 0);
        assert_eq!(storage.get_balance(&addr2).await.unwrap(), 0);

        // TODO: Add more comprehensive tests
    }

    #[tokio::test]
    async fn test_transaction_expiry() {
        let config = EphemeralConfig {
            transaction_memory_minutes: 1, // Very short memory for testing
            ..Default::default()
        };

        let storage = EphemeralStorage::with_config(config);

        // Store a transaction
        let tx = Transaction {
            id: Uuid::new_v4(),
            from: Address([0u8; 32]),
            to: Address([1u8; 32]),
            amount: 100,
            fee: 1,
            message: Some("test".to_string()),
            timestamp: chrono::Utc::now().timestamp(),
        };

        storage.store_transaction(&tx).await.unwrap();

        // Should have 1 recent transaction
        let recent = storage.get_recent_transactions(None).await.unwrap();
        assert_eq!(recent.len(), 1);

        // TODO: Test expiry after waiting
    }
}
