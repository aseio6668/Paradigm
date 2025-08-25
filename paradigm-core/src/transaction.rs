use crate::{Address, Keypair};
use blake3::Hasher;
use chrono::{DateTime, Utc};
use ed25519_dalek::{Signature, Signer, SigningKey, Verifier, VerifyingKey};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

/// Transaction structure for Paradigm network
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Transaction {
    pub id: Uuid,
    pub from: Address,
    pub to: Address,
    pub amount: u64, // Amount in smallest unit (8 decimal places)
    pub fee: u64,    // Transaction fee
    pub timestamp: DateTime<Utc>,
    pub signature: Vec<u8>,
    pub nonce: u64,
    pub message: Option<String>, // Optional 10-character message
}

impl Transaction {
    /// Create a new transaction
    pub fn new(
        from: Address,
        to: Address,
        amount: u64,
        fee: u64,
        timestamp: DateTime<Utc>,
        keypair: &Keypair,
    ) -> anyhow::Result<Self> {
        Self::new_with_message(from, to, amount, fee, timestamp, keypair, None)
    }

    /// Create a new transaction with optional message
    pub fn new_with_message(
        from: Address,
        to: Address,
        amount: u64,
        fee: u64,
        timestamp: DateTime<Utc>,
        keypair: &Keypair,
        message: Option<String>,
    ) -> anyhow::Result<Self> {
        let id = Uuid::new_v4();
        let nonce = timestamp.timestamp_nanos_opt().unwrap_or(0) as u64;

        // Validate message length (10 characters max)
        if let Some(ref msg) = message {
            if msg.len() > 10 {
                return Err(anyhow::anyhow!("Transaction message cannot exceed 10 characters"));
            }
            // Ensure message contains only printable ASCII characters
            if !msg.chars().all(|c| c.is_ascii() && !c.is_control()) {
                return Err(anyhow::anyhow!("Transaction message must contain only printable ASCII characters"));
            }
        }

        // Create transaction without signature first
        let mut transaction = Transaction {
            id,
            from,
            to,
            amount,
            fee,
            timestamp,
            signature: Vec::new(),
            nonce,
            message,
        };

        // Sign the transaction
        let signature = transaction.sign(keypair)?;
        transaction.signature = signature.to_bytes().to_vec();

        Ok(transaction)
    }

    /// Sign the transaction
    fn sign(&self, keypair: &Keypair) -> anyhow::Result<Signature> {
        let message = self.get_signing_data();
        Ok(keypair.sign(&message))
    }

    /// Verify transaction signature
    pub fn verify_signature(&self, public_key: &VerifyingKey) -> bool {
        if self.signature.is_empty() {
            return false;
        }

        let signature_bytes: [u8; 64] = match self.signature.as_slice().try_into() {
            Ok(bytes) => bytes,
            Err(_) => return false,
        };

        let signature = Signature::from_bytes(&signature_bytes);

        let message = self.get_signing_data();
        public_key.verify(&message, &signature).is_ok()
    }

    /// Get data to be signed/verified
    fn get_signing_data(&self) -> Vec<u8> {
        let mut hasher = Hasher::new();
        hasher.update(self.id.as_bytes());
        hasher.update(self.from.as_bytes());
        hasher.update(self.to.as_bytes());
        hasher.update(&self.amount.to_le_bytes());
        hasher.update(&self.fee.to_le_bytes());
        hasher.update(
            &self
                .timestamp
                .timestamp_nanos_opt()
                .unwrap_or(0)
                .to_le_bytes(),
        );
        hasher.update(&self.nonce.to_le_bytes());
        
        // Include message in signing data
        if let Some(ref message) = self.message {
            hasher.update(message.as_bytes());
        }
        
        hasher.finalize().as_bytes().to_vec()
    }

    /// Calculate transaction hash
    pub fn hash(&self) -> Vec<u8> {
        let mut hasher = Hasher::new();
        hasher.update(&self.get_signing_data());
        hasher.update(&self.signature);
        hasher.finalize().as_bytes().to_vec()
    }

    /// Validate transaction
    pub fn validate(&self, public_key: &VerifyingKey) -> anyhow::Result<()> {
        // Check amounts
        if self.amount == 0 {
            return Err(anyhow::anyhow!("Transaction amount cannot be zero"));
        }

        if self.fee == 0 {
            return Err(anyhow::anyhow!("Transaction fee cannot be zero"));
        }

        // Check timestamp (not too far in future)
        let now = Utc::now();
        if self.timestamp > now + chrono::Duration::minutes(10) {
            return Err(anyhow::anyhow!("Transaction timestamp too far in future"));
        }

        // Verify signature
        if !self.verify_signature(public_key) {
            return Err(anyhow::anyhow!("Invalid transaction signature"));
        }

        Ok(())
    }
}

/// Transaction pool for managing pending transactions
#[derive(Debug)]
pub struct TransactionPool {
    transactions: HashMap<Uuid, Transaction>,
    by_address: HashMap<Address, Vec<Uuid>>,
}

impl TransactionPool {
    pub fn new() -> Self {
        TransactionPool {
            transactions: HashMap::new(),
            by_address: HashMap::new(),
        }
    }

    /// Add transaction to pool
    pub async fn add_transaction(&mut self, transaction: Transaction) -> anyhow::Result<()> {
        let tx_id = transaction.id;
        let from_address = transaction.from.clone();

        // Check if transaction already exists
        if self.transactions.contains_key(&tx_id) {
            return Err(anyhow::anyhow!("Transaction already in pool"));
        }

        // Add to transactions map
        self.transactions.insert(tx_id, transaction);

        // Add to address index
        self.by_address
            .entry(from_address)
            .or_insert_with(Vec::new)
            .push(tx_id);

        Ok(())
    }

    /// Remove transaction from pool
    pub fn remove_transaction(&mut self, tx_id: &Uuid) -> Option<Transaction> {
        if let Some(transaction) = self.transactions.remove(tx_id) {
            // Remove from address index
            if let Some(tx_list) = self.by_address.get_mut(&transaction.from) {
                tx_list.retain(|id| id != tx_id);
                if tx_list.is_empty() {
                    self.by_address.remove(&transaction.from);
                }
            }
            Some(transaction)
        } else {
            None
        }
    }

    /// Get all transactions for processing
    pub fn get_all_transactions(&self) -> Vec<&Transaction> {
        self.transactions.values().collect()
    }

    /// Get transactions by address
    pub fn get_transactions_by_address(&self, address: &Address) -> Vec<&Transaction> {
        if let Some(tx_ids) = self.by_address.get(address) {
            tx_ids
                .iter()
                .filter_map(|id| self.transactions.get(id))
                .collect()
        } else {
            Vec::new()
        }
    }

    /// Get pool size
    pub fn size(&self) -> usize {
        self.transactions.len()
    }

    /// Clear all transactions
    pub fn clear(&mut self) {
        self.transactions.clear();
        self.by_address.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Keypair;
    use rand::thread_rng;

    #[test]
    fn test_transaction_creation() {
        let keypair = Keypair::generate(&mut thread_rng());
        let from = Address::from_public_key(&keypair.public);
        let to = Address::from_public_key(&keypair.public);

        let transaction = Transaction::new(
            from,
            to,
            1000000000, // 10 PAR
            10000000,   // 0.1 PAR fee
            Utc::now(),
            &keypair,
        )
        .unwrap();

        assert_eq!(transaction.amount, 1000000000);
        assert_eq!(transaction.fee, 10000000);
        assert!(!transaction.signature.is_empty());
    }

    #[test]
    fn test_transaction_verification() {
        let keypair = Keypair::generate(&mut thread_rng());
        let from = Address::from_public_key(&keypair.public);
        let to = Address::from_public_key(&keypair.public);

        let transaction =
            Transaction::new(from, to, 1000000000, 10000000, Utc::now(), &keypair).unwrap();

        assert!(transaction.verify_signature(&keypair.public));
        assert!(transaction.validate(&keypair.public).is_ok());
    }

    #[tokio::test]
    async fn test_transaction_pool() {
        let mut pool = TransactionPool::new();
        let keypair = Keypair::generate(&mut thread_rng());
        let from = Address::from_public_key(&keypair.public);
        let to = Address::from_public_key(&keypair.public);

        let transaction =
            Transaction::new(from.clone(), to, 1000000000, 10000000, Utc::now(), &keypair).unwrap();

        pool.add_transaction(transaction.clone()).await.unwrap();
        assert_eq!(pool.size(), 1);

        let transactions = pool.get_transactions_by_address(&from);
        assert_eq!(transactions.len(), 1);
        assert_eq!(transactions[0].id, transaction.id);
    }
}
