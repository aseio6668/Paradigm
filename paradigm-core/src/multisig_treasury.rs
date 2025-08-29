use crate::Transaction;
use anyhow::{anyhow, Result};
use ed25519_dalek::{Signature, Signer, SigningKey, VerifyingKey};
use serde::{Deserialize, Serialize};
use sqlx::{Row, SqlitePool};
use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};
use tracing;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MultiSigWallet {
    pub wallet_id: Uuid,
    pub name: String,
    pub threshold: u32,
    pub required_signatures: u32,
    pub signers: Vec<WalletSigner>,
    pub balance: u64,
    pub created_at: u64,
    pub is_active: bool,
    pub wallet_type: TreasuryWalletType,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WalletSigner {
    pub signer_id: Uuid,
    pub public_key: Vec<u8>,
    pub name: String,
    pub role: SignerRole,
    pub weight: u32,
    pub is_active: bool,
    pub last_signed: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SignerRole {
    TreasuryManager,
    NetworkGovernor,
    EmergencyRecovery,
    AuditOversight,
    TechnicalLead,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TreasuryWalletType {
    MainTreasury,
    DevelopmentFund,
    MarketingFund,
    EmergencyReserve,
    GovernanceFund,
    StakingRewards,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PendingTransaction {
    pub transaction_id: Uuid,
    pub wallet_id: Uuid,
    pub transaction: Transaction,
    pub signatures: HashMap<Uuid, TransactionSignature>,
    pub required_signatures: u32,
    pub created_at: u64,
    pub expires_at: u64,
    pub status: TransactionStatus,
    pub created_by: Uuid,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransactionSignature {
    pub signer_id: Uuid,
    pub signature: Vec<u8>,
    pub signed_at: u64,
    pub signature_hash: Vec<u8>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum TransactionStatus {
    Pending,
    AwaitingSignatures,
    ReadyToExecute,
    Executed,
    Rejected,
    Expired,
}

#[derive(Debug)]
pub struct MultiSigTreasuryManager {
    pub db_pool: SqlitePool,
    pub wallets: HashMap<Uuid, MultiSigWallet>,
    pub pending_transactions: HashMap<Uuid, PendingTransaction>,
    pub signature_timeout: u64, // seconds
}

impl MultiSigTreasuryManager {
    pub fn new(db_pool: SqlitePool) -> Self {
        Self {
            db_pool,
            wallets: HashMap::new(),
            pending_transactions: HashMap::new(),
            signature_timeout: 24 * 60 * 60, // 24 hours default
        }
    }

    pub async fn initialize(&mut self) -> Result<()> {
        self.create_tables().await?;
        self.load_wallets().await?;
        self.load_pending_transactions().await?;
        tracing::info!(
            "🔐 Multi-signature treasury manager initialized with {} wallets",
            self.wallets.len()
        );
        Ok(())
    }

    async fn create_tables(&self) -> Result<()> {
        // Create multi-sig wallets table
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS multisig_wallets (
                wallet_id TEXT PRIMARY KEY,
                name TEXT NOT NULL,
                threshold INTEGER NOT NULL,
                required_signatures INTEGER NOT NULL,
                signers TEXT NOT NULL,
                balance INTEGER NOT NULL DEFAULT 0,
                created_at INTEGER NOT NULL,
                is_active BOOLEAN NOT NULL DEFAULT true,
                wallet_type TEXT NOT NULL
            )
        "#,
        )
        .execute(&self.db_pool)
        .await?;

        // Create pending transactions table
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS multisig_pending_transactions (
                transaction_id TEXT PRIMARY KEY,
                wallet_id TEXT NOT NULL,
                transaction_data TEXT NOT NULL,
                signatures TEXT NOT NULL,
                required_signatures INTEGER NOT NULL,
                created_at INTEGER NOT NULL,
                expires_at INTEGER NOT NULL,
                status TEXT NOT NULL,
                created_by TEXT NOT NULL,
                FOREIGN KEY (wallet_id) REFERENCES multisig_wallets (wallet_id)
            )
        "#,
        )
        .execute(&self.db_pool)
        .await?;

        tracing::info!("✅ Multi-signature treasury database tables initialized");
        Ok(())
    }

    pub async fn create_treasury_wallet(
        &mut self,
        name: String,
        wallet_type: TreasuryWalletType,
        threshold: u32,
        signers: Vec<WalletSigner>,
    ) -> Result<Uuid> {
        if threshold == 0 || threshold > signers.len() as u32 {
            return Err(anyhow!(
                "Invalid threshold: must be between 1 and {}",
                signers.len()
            ));
        }

        // Calculate required signatures based on weighted threshold
        let total_weight: u32 = signers.iter().map(|s| s.weight).sum();
        let required_signatures = std::cmp::min(threshold, total_weight);

        let wallet_id = Uuid::new_v4();
        let wallet = MultiSigWallet {
            wallet_id,
            name: name.clone(),
            threshold,
            required_signatures,
            signers: signers.clone(),
            balance: 0,
            created_at: SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs(),
            is_active: true,
            wallet_type,
        };

        // Store in database
        let signers_json = serde_json::to_string(&signers)?;
        let wallet_type_str = format!("{:?}", wallet.wallet_type);

        sqlx::query(r#"
            INSERT INTO multisig_wallets 
            (wallet_id, name, threshold, required_signatures, signers, balance, created_at, is_active, wallet_type)
            VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)
        "#)
        .bind(wallet_id.to_string())
        .bind(&wallet.name)
        .bind(wallet.threshold as i64)
        .bind(wallet.required_signatures as i64)
        .bind(signers_json)
        .bind(wallet.balance as i64)
        .bind(wallet.created_at as i64)
        .bind(wallet.is_active)
        .bind(wallet_type_str)
        .execute(&self.db_pool).await?;

        // Store in memory
        self.wallets.insert(wallet_id, wallet);

        tracing::info!(
            "🏦 Created multi-sig treasury wallet '{}' (ID: {}) with {}/{} threshold",
            name,
            wallet_id,
            threshold,
            signers.len()
        );
        Ok(wallet_id)
    }

    pub async fn propose_transaction(
        &mut self,
        wallet_id: Uuid,
        transaction: Transaction,
        proposer_id: Uuid,
    ) -> Result<Uuid> {
        let wallet = self
            .wallets
            .get(&wallet_id)
            .ok_or_else(|| anyhow!("Treasury wallet not found: {}", wallet_id))?;

        if !wallet.is_active {
            return Err(anyhow!("Treasury wallet is inactive"));
        }

        // Verify proposer is authorized signer
        let _proposer = wallet
            .signers
            .iter()
            .find(|s| s.signer_id == proposer_id && s.is_active)
            .ok_or_else(|| anyhow!("Proposer is not an authorized signer"))?;

        let transaction_id = Uuid::new_v4();
        let current_time = SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs();
        let expires_at = current_time + self.signature_timeout;

        let pending_tx = PendingTransaction {
            transaction_id,
            wallet_id,
            transaction: transaction.clone(),
            signatures: HashMap::new(),
            required_signatures: wallet.required_signatures,
            created_at: current_time,
            expires_at,
            status: TransactionStatus::AwaitingSignatures,
            created_by: proposer_id,
        };

        // Store in database
        let transaction_json = serde_json::to_string(&transaction)?;
        let signatures_json = serde_json::to_string(&pending_tx.signatures)?;
        let status_str = format!("{:?}", pending_tx.status);

        sqlx::query(r#"
            INSERT INTO multisig_pending_transactions 
            (transaction_id, wallet_id, transaction_data, signatures, required_signatures, created_at, expires_at, status, created_by)
            VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)
        "#)
        .bind(transaction_id.to_string())
        .bind(wallet_id.to_string())
        .bind(transaction_json)
        .bind(signatures_json)
        .bind(pending_tx.required_signatures as i64)
        .bind(current_time as i64)
        .bind(expires_at as i64)
        .bind(status_str)
        .bind(proposer_id.to_string())
        .execute(&self.db_pool).await?;

        // Store in memory
        self.pending_transactions.insert(transaction_id, pending_tx);

        tracing::info!(
            "📝 Transaction proposed for wallet {} by signer {} (TX: {})",
            wallet_id,
            proposer_id,
            transaction_id
        );
        Ok(transaction_id)
    }

    pub async fn sign_transaction(
        &mut self,
        transaction_id: Uuid,
        signer_id: Uuid,
        signing_key: &SigningKey,
    ) -> Result<bool> {
        let pending_tx = self
            .pending_transactions
            .get_mut(&transaction_id)
            .ok_or_else(|| anyhow!("Pending transaction not found: {}", transaction_id))?;

        if pending_tx.status != TransactionStatus::AwaitingSignatures {
            return Err(anyhow!("Transaction is not awaiting signatures"));
        }

        let current_time = SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs();
        if current_time > pending_tx.expires_at {
            pending_tx.status = TransactionStatus::Expired;
            return Err(anyhow!("Transaction has expired"));
        }

        let wallet = self
            .wallets
            .get(&pending_tx.wallet_id)
            .ok_or_else(|| anyhow!("Treasury wallet not found"))?;

        // Verify signer is authorized
        let signer = wallet
            .signers
            .iter()
            .find(|s| s.signer_id == signer_id && s.is_active)
            .ok_or_else(|| anyhow!("Unauthorized signer"))?;

        // Check if already signed
        if pending_tx.signatures.contains_key(&signer_id) {
            return Err(anyhow!("Transaction already signed by this signer"));
        }

        // Create signature hash from transaction data
        let transaction_bytes = serde_json::to_vec(&pending_tx.transaction)?;
        let signature = signing_key.sign(&transaction_bytes);
        let signature_hash = blake3::hash(&transaction_bytes);

        let tx_signature = TransactionSignature {
            signer_id,
            signature: signature.to_bytes().to_vec(),
            signed_at: current_time,
            signature_hash: signature_hash.as_bytes().to_vec(),
        };

        // Add signature
        pending_tx.signatures.insert(signer_id, tx_signature);

        // Check if we have enough signatures (weighted by signer weight)
        let signature_weight: u32 = pending_tx
            .signatures
            .keys()
            .filter_map(|id| wallet.signers.iter().find(|s| s.signer_id == *id))
            .map(|s| s.weight)
            .sum();

        let is_ready = signature_weight >= wallet.threshold;
        if is_ready {
            pending_tx.status = TransactionStatus::ReadyToExecute;
            tracing::info!(
                "✅ Transaction {} ready to execute ({}/{} weight threshold met)",
                transaction_id,
                signature_weight,
                wallet.threshold
            );
        }

        // Update database
        let signatures_json = serde_json::to_string(&pending_tx.signatures)?;
        let status_str = format!("{:?}", pending_tx.status);

        sqlx::query(
            r#"
            UPDATE multisig_pending_transactions 
            SET signatures = ?, status = ? 
            WHERE transaction_id = ?
        "#,
        )
        .bind(signatures_json)
        .bind(status_str)
        .bind(transaction_id.to_string())
        .execute(&self.db_pool)
        .await?;

        tracing::info!(
            "🖊️ Transaction {} signed by {} (weight: {}, total: {}/{})",
            transaction_id,
            signer.name,
            signer.weight,
            signature_weight,
            wallet.threshold
        );

        Ok(is_ready)
    }

    pub async fn execute_transaction(&mut self, transaction_id: Uuid) -> Result<Transaction> {
        let pending_tx = self
            .pending_transactions
            .get_mut(&transaction_id)
            .ok_or_else(|| anyhow!("Pending transaction not found: {}", transaction_id))?;

        if pending_tx.status != TransactionStatus::ReadyToExecute {
            return Err(anyhow!("Transaction is not ready to execute"));
        }

        // Final verification of signatures
        let wallet = self
            .wallets
            .get(&pending_tx.wallet_id)
            .ok_or_else(|| anyhow!("Treasury wallet not found"))?;

        let signature_weight: u32 = pending_tx
            .signatures
            .keys()
            .filter_map(|id| wallet.signers.iter().find(|s| s.signer_id == *id))
            .map(|s| s.weight)
            .sum();

        if signature_weight < wallet.threshold {
            return Err(anyhow!(
                "Insufficient signature weight: {}/{}",
                signature_weight,
                wallet.threshold
            ));
        }

        // Verify each signature
        let transaction_bytes = serde_json::to_vec(&pending_tx.transaction)?;
        for (signer_id, tx_sig) in &pending_tx.signatures {
            let signer = wallet
                .signers
                .iter()
                .find(|s| s.signer_id == *signer_id)
                .ok_or_else(|| anyhow!("Signer not found: {}", signer_id))?;

            let public_key_bytes: [u8; 32] = signer
                .public_key
                .clone()
                .try_into()
                .map_err(|_| anyhow!("Invalid public key length"))?;
            let verifying_key = VerifyingKey::from_bytes(&public_key_bytes)?;

            let signature_bytes: [u8; 64] = tx_sig
                .signature
                .clone()
                .try_into()
                .map_err(|_| anyhow!("Invalid signature length"))?;
            let signature = Signature::from_bytes(&signature_bytes);

            verifying_key
                .verify_strict(&transaction_bytes, &signature)
                .map_err(|_| anyhow!("Signature verification failed for signer: {}", signer_id))?;
        }

        // Mark as executed
        pending_tx.status = TransactionStatus::Executed;

        // Update database
        let status_str = format!("{:?}", pending_tx.status);
        sqlx::query(
            r#"
            UPDATE multisig_pending_transactions 
            SET status = ? 
            WHERE transaction_id = ?
        "#,
        )
        .bind(status_str)
        .bind(transaction_id.to_string())
        .execute(&self.db_pool)
        .await?;

        let executed_transaction = pending_tx.transaction.clone();
        tracing::info!(
            "🚀 Multi-sig transaction {} executed successfully from wallet {}",
            transaction_id,
            wallet.name
        );

        Ok(executed_transaction)
    }

    pub async fn get_wallet_info(&self, wallet_id: Uuid) -> Result<&MultiSigWallet> {
        self.wallets
            .get(&wallet_id)
            .ok_or_else(|| anyhow!("Treasury wallet not found: {}", wallet_id))
    }

    pub async fn get_pending_transactions(
        &self,
        wallet_id: Option<Uuid>,
    ) -> Vec<&PendingTransaction> {
        self.pending_transactions
            .values()
            .filter(|tx| wallet_id.map_or(true, |id| tx.wallet_id == id))
            .filter(|tx| {
                matches!(
                    tx.status,
                    TransactionStatus::AwaitingSignatures | TransactionStatus::ReadyToExecute
                )
            })
            .collect()
    }

    pub async fn cleanup_expired_transactions(&mut self) -> Result<u32> {
        let current_time = SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs();
        let mut expired_count = 0;

        let expired_ids: Vec<Uuid> = self
            .pending_transactions
            .iter()
            .filter(|(_, tx)| {
                current_time > tx.expires_at
                    && matches!(tx.status, TransactionStatus::AwaitingSignatures)
            })
            .map(|(id, _)| *id)
            .collect();

        for tx_id in expired_ids {
            if let Some(pending_tx) = self.pending_transactions.get_mut(&tx_id) {
                pending_tx.status = TransactionStatus::Expired;

                // Update database
                sqlx::query(
                    r#"
                    UPDATE multisig_pending_transactions 
                    SET status = 'Expired' 
                    WHERE transaction_id = ?
                "#,
                )
                .bind(tx_id.to_string())
                .execute(&self.db_pool)
                .await?;

                expired_count += 1;
            }
        }

        if expired_count > 0 {
            tracing::info!(
                "🧹 Cleaned up {} expired multi-sig transactions",
                expired_count
            );
        }

        Ok(expired_count)
    }

    async fn load_wallets(&mut self) -> Result<()> {
        let rows = sqlx::query(r#"
            SELECT wallet_id, name, threshold, required_signatures, signers, balance, created_at, is_active, wallet_type
            FROM multisig_wallets
        "#).fetch_all(&self.db_pool).await?;

        for row in rows {
            let wallet_id: String = row.get("wallet_id");
            let wallet_id = Uuid::parse_str(&wallet_id)?;

            let signers_json: String = row.get("signers");
            let signers: Vec<WalletSigner> = serde_json::from_str(&signers_json)?;

            let wallet_type_str: String = row.get("wallet_type");
            let wallet_type = match wallet_type_str.as_str() {
                "MainTreasury" => TreasuryWalletType::MainTreasury,
                "DevelopmentFund" => TreasuryWalletType::DevelopmentFund,
                "MarketingFund" => TreasuryWalletType::MarketingFund,
                "EmergencyReserve" => TreasuryWalletType::EmergencyReserve,
                "GovernanceFund" => TreasuryWalletType::GovernanceFund,
                "StakingRewards" => TreasuryWalletType::StakingRewards,
                _ => TreasuryWalletType::MainTreasury,
            };

            let wallet = MultiSigWallet {
                wallet_id,
                name: row.get("name"),
                threshold: row.get::<i64, _>("threshold") as u32,
                required_signatures: row.get::<i64, _>("required_signatures") as u32,
                signers,
                balance: row.get::<i64, _>("balance") as u64,
                created_at: row.get::<i64, _>("created_at") as u64,
                is_active: row.get("is_active"),
                wallet_type,
            };

            self.wallets.insert(wallet_id, wallet);
        }

        Ok(())
    }

    async fn load_pending_transactions(&mut self) -> Result<()> {
        let rows = sqlx::query(r#"
            SELECT transaction_id, wallet_id, transaction_data, signatures, required_signatures, created_at, expires_at, status, created_by
            FROM multisig_pending_transactions
            WHERE status IN ('AwaitingSignatures', 'ReadyToExecute')
        "#).fetch_all(&self.db_pool).await?;

        for row in rows {
            let transaction_id: String = row.get("transaction_id");
            let transaction_id = Uuid::parse_str(&transaction_id)?;

            let wallet_id: String = row.get("wallet_id");
            let wallet_id = Uuid::parse_str(&wallet_id)?;

            let created_by: String = row.get("created_by");
            let created_by = Uuid::parse_str(&created_by)?;

            let transaction_json: String = row.get("transaction_data");
            let transaction: Transaction = serde_json::from_str(&transaction_json)?;

            let signatures_json: String = row.get("signatures");
            let signatures: HashMap<Uuid, TransactionSignature> =
                serde_json::from_str(&signatures_json)?;

            let status_str: String = row.get("status");
            let status = match status_str.as_str() {
                "AwaitingSignatures" => TransactionStatus::AwaitingSignatures,
                "ReadyToExecute" => TransactionStatus::ReadyToExecute,
                _ => TransactionStatus::Pending,
            };

            let pending_tx = PendingTransaction {
                transaction_id,
                wallet_id,
                transaction,
                signatures,
                required_signatures: row.get::<i64, _>("required_signatures") as u32,
                created_at: row.get::<i64, _>("created_at") as u64,
                expires_at: row.get::<i64, _>("expires_at") as u64,
                status,
                created_by,
            };

            self.pending_transactions.insert(transaction_id, pending_tx);
        }

        Ok(())
    }
}
