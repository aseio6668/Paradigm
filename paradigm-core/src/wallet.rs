use std::collections::HashMap;
use std::path::Path;
use serde::{Deserialize, Serialize};
use ed25519_dalek::{SigningKey, VerifyingKey, SecretKey};
use crate::{Keypair, PublicKey, Address, AddressExt, transaction::Transaction};
use rand::thread_rng;
use blake3::Hasher;
use anyhow::Result;
use sqlx::{SqlitePool, Row};

/// Paradigm wallet for managing keys and transactions
#[derive(Debug)]
pub struct Wallet {
    keypair: Keypair,
    address: Address,
    balance: u64,
    transaction_history: Vec<Transaction>,
    db_pool: Option<SqlitePool>,
}

impl Wallet {
    /// Create a new wallet with random keypair
    pub fn new() -> Self {
        use rand::rngs::OsRng;
        use rand::RngCore;
        
        // Generate random bytes for the keypair  
        let mut secret_bytes = [0u8; 32];
        OsRng.fill_bytes(&mut secret_bytes);
        let keypair = Keypair::from_bytes(&secret_bytes);
        let address = AddressExt::from_public_key(&keypair.verifying_key());
        
        Wallet {
            keypair,
            address,
            balance: 0,
            transaction_history: Vec::new(),
            db_pool: None,
        }
    }

    /// Create wallet from existing private key
    pub fn from_private_key(private_key: &[u8; 32]) -> Result<Self> {
        let keypair = SigningKey::from_bytes(private_key);
        let address = AddressExt::from_public_key(&keypair.verifying_key());
        
        Ok(Wallet {
            keypair,
            address,
            balance: 0,
            transaction_history: Vec::new(),
            db_pool: None,
        })
    }

    /// Initialize wallet with database storage
    pub async fn initialize_storage<P: AsRef<Path>>(&mut self, db_path: P) -> Result<()> {
        let db_url = format!("sqlite:{}", db_path.as_ref().display());
        let pool = SqlitePool::connect(&db_url).await?;
        
        // Create tables
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
                status TEXT NOT NULL DEFAULT 'pending'
            )
            "#,
        )
        .execute(&pool)
        .await?;

        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS wallet_info (
                address TEXT PRIMARY KEY,
                balance INTEGER NOT NULL DEFAULT 0,
                last_updated TEXT NOT NULL
            )
            "#,
        )
        .execute(&pool)
        .await?;

        self.db_pool = Some(pool);
        self.load_from_storage().await?;
        
        Ok(())
    }

    /// Load wallet data from storage
    async fn load_from_storage(&mut self) -> Result<()> {
        if let Some(pool) = &self.db_pool {
            // Load balance
            let row = sqlx::query("SELECT balance FROM wallet_info WHERE address = ?")
                .bind(self.address.to_string())
                .fetch_optional(pool)
                .await?;

            if let Some(row) = row {
                self.balance = row.get::<i64, _>("balance") as u64;
            }

            // Load transaction history
            let rows = sqlx::query("SELECT * FROM transactions WHERE from_address = ? OR to_address = ? ORDER BY timestamp DESC")
                .bind(self.address.to_string())
                .bind(self.address.to_string())
                .fetch_all(pool)
                .await?;

            self.transaction_history.clear();
            for row in rows {
                // In a real implementation, we'd deserialize the full transaction
                // For now, we'll skip this for simplicity
            }
        }
        Ok(())
    }

    /// Save wallet state to storage
    async fn save_to_storage(&self) -> Result<()> {
        if let Some(pool) = &self.db_pool {
            sqlx::query(
                "INSERT OR REPLACE INTO wallet_info (address, balance, last_updated) VALUES (?, ?, ?)"
            )
            .bind(self.address.to_string())
            .bind(self.balance as i64)
            .bind(chrono::Utc::now().to_rfc3339())
            .execute(pool)
            .await?;
        }
        Ok(())
    }

    /// Get wallet address
    pub fn get_address(&self) -> &Address {
        &self.address
    }

    /// Get current balance
    pub fn get_balance(&self) -> u64 {
        self.balance
    }

    /// Get balance in PAR (with decimals)
    pub fn get_balance_par(&self) -> f64 {
        self.balance as f64 / 100_000_000.0
    }

    /// Update balance
    pub async fn update_balance(&mut self, new_balance: u64) -> Result<()> {
        self.balance = new_balance;
        self.save_to_storage().await?;
        Ok(())
    }

    /// Create a new transaction
    pub fn create_transaction(
        &self,
        to: Address,
        amount: u64,
        fee: u64,
    ) -> Result<Transaction> {
        if amount + fee > self.balance {
            return Err(anyhow::anyhow!("Insufficient balance"));
        }

        Transaction::new(
            self.address.clone(),
            to,
            amount,
            fee,
            chrono::Utc::now(),
            &self.keypair,
        )
    }

    /// Add transaction to history
    pub async fn add_transaction(&mut self, transaction: Transaction) -> Result<()> {
        // Update balance based on transaction
        if transaction.from == self.address {
            // Outgoing transaction
            self.balance = self.balance.saturating_sub(transaction.amount + transaction.fee);
        } else if transaction.to == self.address {
            // Incoming transaction
            self.balance = self.balance.saturating_add(transaction.amount);
        }

        self.transaction_history.push(transaction.clone());
        
        // Save to database
        if let Some(pool) = &self.db_pool {
            sqlx::query(
                r#"
                INSERT INTO transactions (id, from_address, to_address, amount, fee, timestamp, signature, nonce, status)
                VALUES (?, ?, ?, ?, ?, ?, ?, ?, 'confirmed')
                "#
            )
            .bind(transaction.id.to_string())
            .bind(transaction.from.to_string())
            .bind(transaction.to.to_string())
            .bind(transaction.amount as i64)
            .bind(transaction.fee as i64)
            .bind(transaction.timestamp.to_rfc3339())
            .bind(&transaction.signature)
            .bind(transaction.nonce as i64)
            .execute(pool)
            .await?;
        }

        self.save_to_storage().await?;
        Ok(())
    }

    /// Get transaction history
    pub fn get_transaction_history(&self) -> &[Transaction] {
        &self.transaction_history
    }

    /// Export private key (be very careful with this!)
    pub fn export_private_key(&self) -> [u8; 32] {
        self.keypair.to_bytes()
    }

    /// Export public key
    pub fn export_public_key(&self) -> [u8; 32] {
        self.keypair.verifying_key().to_bytes()
    }

    /// Generate a seed phrase (simplified - in reality use BIP39)
    pub fn generate_seed_phrase() -> Vec<String> {
        // Simplified seed phrase generation
        // In a real implementation, use proper BIP39 wordlist
        vec![
            "paradigm".to_string(), "crypto".to_string(), "machine".to_string(), 
            "learning".to_string(), "network".to_string(), "autonomous".to_string(),
            "secure".to_string(), "instant".to_string(), "transaction".to_string(), 
            "reward".to_string(), "contribution".to_string(), "future".to_string()
        ]
    }

    /// Create wallet from seed phrase (simplified)
    pub fn from_seed_phrase(words: &[String]) -> Result<Self> {
        // Simplified seed to key derivation
        // In a real implementation, use proper BIP39/BIP44
        let mut hasher = Hasher::new();
        for word in words {
            hasher.update(word.as_bytes());
        }
        let hash = hasher.finalize();
        let private_key: [u8; 32] = hash.as_bytes()[..32].try_into()?;
        
        Self::from_private_key(&private_key)
    }

    /// Sign a message with the wallet's private key
    pub fn sign_message(&self, message: &[u8]) -> Vec<u8> {
        use ed25519_dalek::Signer;
        self.keypair.sign(message).to_bytes().to_vec()
    }

    /// Verify a signature
    pub fn verify_signature(&self, message: &[u8], signature: &[u8]) -> bool {
        use ed25519_dalek::Verifier;
        if signature.len() != 64 {
            return false;
        }
        
        let sig_bytes: [u8; 64] = signature.try_into().unwrap_or([0; 64]);
        let signature = ed25519_dalek::Signature::from_bytes(&sig_bytes);
        self.keypair.verifying_key().verify(message, &signature).is_ok()
    }

    /// Get wallet statistics
    pub fn get_stats(&self) -> WalletStats {
        let total_sent = self.transaction_history
            .iter()
            .filter(|tx| tx.from == self.address)
            .map(|tx| tx.amount + tx.fee)
            .sum();

        let total_received = self.transaction_history
            .iter()
            .filter(|tx| tx.to == self.address)
            .map(|tx| tx.amount)
            .sum();

        let transaction_count = self.transaction_history.len();

        WalletStats {
            balance: self.balance,
            total_sent,
            total_received,
            transaction_count,
        }
    }
}

/// Wallet statistics
#[derive(Debug, Serialize, Deserialize)]
pub struct WalletStats {
    pub balance: u64,
    pub total_sent: u64,
    pub total_received: u64,
    pub transaction_count: usize,
}

/// Multi-signature wallet functionality
#[derive(Debug)]
pub struct MultiSigWallet {
    required_signatures: u8,
    signers: Vec<PublicKey>,
    pending_transactions: HashMap<uuid::Uuid, (Transaction, Vec<Vec<u8>>)>,
}

impl MultiSigWallet {
    pub fn new(required_signatures: u8, signers: Vec<PublicKey>) -> Result<Self> {
        if required_signatures == 0 || required_signatures > signers.len() as u8 {
            return Err(anyhow::anyhow!("Invalid signature requirements"));
        }

        Ok(MultiSigWallet {
            required_signatures,
            signers,
            pending_transactions: HashMap::new(),
        })
    }

    pub fn add_signature(
        &mut self,
        transaction_id: uuid::Uuid,
        signature: Vec<u8>,
    ) -> Result<bool> {
        if let Some((_, signatures)) = self.pending_transactions.get_mut(&transaction_id) {
            signatures.push(signature);
            Ok(signatures.len() >= self.required_signatures as usize)
        } else {
            Err(anyhow::anyhow!("Transaction not found"))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_wallet_creation() {
        let wallet = Wallet::new();
        assert_eq!(wallet.get_balance(), 0);
        assert!(!wallet.get_address().to_string().is_empty());
    }

    #[test]
    fn test_private_key_wallet() {
        let private_key = [1u8; 32];
        let wallet1 = Wallet::from_private_key(&private_key).unwrap();
        let wallet2 = Wallet::from_private_key(&private_key).unwrap();
        
        assert_eq!(wallet1.get_address(), wallet2.get_address());
    }

    #[test]
    fn test_seed_phrase() {
        let words = Wallet::generate_seed_phrase();
        let wallet = Wallet::from_seed_phrase(&words).unwrap();
        assert!(!wallet.get_address().to_string().is_empty());
    }

    #[tokio::test]
    async fn test_wallet_storage() {
        let temp_dir = tempdir().unwrap();
        let db_path = temp_dir.path().join("test_wallet.db");
        
        let mut wallet = Wallet::new();
        wallet.initialize_storage(&db_path).await.unwrap();
        wallet.update_balance(1000000000).await.unwrap(); // 10 PAR
        
        // Create new wallet instance with same storage
        let mut wallet2 = Wallet::from_private_key(&wallet.export_private_key()).unwrap();
        wallet2.initialize_storage(&db_path).await.unwrap();
        
        assert_eq!(wallet2.get_balance(), 1000000000);
    }

    #[test]
    fn test_transaction_creation() {
        let mut wallet = Wallet::new();
        wallet.balance = 2000000000; // 20 PAR
        
        let to_wallet = Wallet::new();
        let transaction = wallet.create_transaction(
            to_wallet.get_address().clone(),
            1000000000, // 10 PAR
            10000000,   // 0.1 PAR fee
        ).unwrap();
        
        assert_eq!(transaction.amount, 1000000000);
        assert_eq!(transaction.fee, 10000000);
        assert_eq!(transaction.from, *wallet.get_address());
        assert_eq!(transaction.to, *to_wallet.get_address());
    }

    #[test]
    fn test_signature_verification() {
        let wallet = Wallet::new();
        let message = b"test message";
        let signature = wallet.sign_message(message);
        
        assert!(wallet.verify_signature(message, &signature));
        assert!(!wallet.verify_signature(b"different message", &signature));
    }
}
