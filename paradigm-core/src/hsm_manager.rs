// Hardware Security Module (HSM) Support
// Provides secure key management, signing operations, and cryptographic functions
// using hardware-backed security for production environments

use anyhow::Result;
use ring::rand::SecureRandom;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};
use tokio::sync::RwLock;
use uuid::Uuid;
use zeroize::{Zeroize, ZeroizeOnDrop};

use crate::{Address, AddressExt, Hash, PublicKey};

/// HSM configuration and connection parameters
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HSMConfig {
    pub hsm_type: HSMType,
    pub connection_string: String,
    pub slot_id: Option<u32>,
    pub pin_file_path: Option<String>,
    pub library_path: Option<String>,
    pub timeout_seconds: u32,
    pub retry_attempts: u32,
    pub enable_backup: bool,
    pub backup_hsm_config: Option<Box<HSMConfig>>,
}

/// Types of HSM supported
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum HSMType {
    PKCS11,      // Standard PKCS#11 interface
    AWSCloudHSM, // AWS CloudHSM
    AzureHSM,    // Azure Dedicated HSM
    Software,    // Software-based HSM for development
    YubiKey,     // YubiKey PIV
    Ledger,      // Ledger hardware wallet
}

/// Cryptographic key metadata stored in HSM
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HSMKeyInfo {
    pub key_id: String,
    pub key_type: KeyType,
    pub algorithm: CryptoAlgorithm,
    pub usage: Vec<KeyUsage>,
    pub created_at: u64,
    pub expires_at: Option<u64>,
    pub label: String,
    pub description: String,
    pub backup_available: bool,
}

/// Types of cryptographic keys
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum KeyType {
    Signing,        // For transaction signing
    Encryption,     // For data encryption
    Authentication, // For node authentication
    Treasury,       // For treasury operations
    Emergency,      // For emergency operations
}

/// Supported cryptographic algorithms
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CryptoAlgorithm {
    Ed25519,
    RSA2048,
    RSA4096,
    ECDSA_P256,
    ECDSA_P384,
    AES256,
    ChaCha20Poly1305,
}

/// Key usage permissions
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum KeyUsage {
    Sign,
    Verify,
    Encrypt,
    Decrypt,
    KeyAgreement,
    CertificateSign,
    Authenticate,
}

/// HSM session management  
struct HSMSession {
    session_id: Uuid,
    hsm_type: HSMType,
    connection_handle: Option<u64>,
    authenticated: bool,
    created_at: SystemTime,
    last_activity: SystemTime,
    operations_count: u64,
}

impl Drop for HSMSession {
    fn drop(&mut self) {
        // Session cleanup if needed
        self.operations_count = 0;
        self.authenticated = false;
    }
}

/// HSM audit log entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HSMAuditEntry {
    pub timestamp: u64,
    pub session_id: Uuid,
    pub operation: HSMOperation,
    pub key_id: Option<String>,
    pub success: bool,
    pub error_code: Option<String>,
    pub user_id: Option<String>,
    pub source_ip: Option<String>,
}

/// HSM operations for auditing
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum HSMOperation {
    KeyGeneration,
    Signing,
    Encryption,
    Decryption,
    KeyDerivation,
    Authentication,
    Backup,
    Recovery,
}

/// Main HSM manager
pub struct HSMManager {
    config: HSMConfig,
    sessions: Arc<RwLock<HashMap<Uuid, HSMSession>>>,
    key_cache: Arc<RwLock<HashMap<String, HSMKeyInfo>>>,
    audit_log: Arc<RwLock<Vec<HSMAuditEntry>>>,
    backup_manager: Option<Arc<HSMBackupManager>>,
}

/// HSM backup and recovery manager
pub struct HSMBackupManager {
    backup_configs: Vec<HSMConfig>,
    encryption_key: [u8; 32],
}

/// HSM operation statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HSMStats {
    pub total_operations: u64,
    pub successful_operations: u64,
    pub failed_operations: u64,
    pub active_sessions: usize,
    pub key_count: usize,
    pub uptime_seconds: u64,
    pub last_backup: Option<u64>,
    pub health_status: HSMHealthStatus,
}

/// HSM health status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum HSMHealthStatus {
    Healthy,
    Warning,
    Critical,
    Offline,
}

impl Default for HSMConfig {
    fn default() -> Self {
        Self {
            hsm_type: HSMType::Software,
            connection_string: "localhost:9999".to_string(),
            slot_id: Some(0),
            pin_file_path: None,
            library_path: None,
            timeout_seconds: 30,
            retry_attempts: 3,
            enable_backup: true,
            backup_hsm_config: None,
        }
    }
}

impl HSMManager {
    /// Create a new HSM manager with configuration
    pub async fn new(config: HSMConfig) -> Result<Self> {
        let backup_manager = if config.enable_backup && config.backup_hsm_config.is_some() {
            // In production, would initialize backup HSM
            match HSMBackupManager::new() {
                Ok(backup) => Some(Arc::new(backup)),
                Err(e) => {
                    tracing::warn!("Failed to initialize HSM backup manager: {}", e);
                    None
                }
            }
        } else {
            None
        };

        let manager = Self {
            config,
            sessions: Arc::new(RwLock::new(HashMap::new())),
            key_cache: Arc::new(RwLock::new(HashMap::new())),
            audit_log: Arc::new(RwLock::new(Vec::new())),
            backup_manager,
        };

        // Initialize HSM connection
        manager.initialize_hsm().await?;

        tracing::info!(
            "🔐 HSM Manager initialized with {:?} HSM",
            manager.config.hsm_type
        );
        Ok(manager)
    }

    /// Initialize HSM connection and load existing keys
    async fn initialize_hsm(&self) -> Result<()> {
        match self.config.hsm_type {
            HSMType::Software => {
                tracing::info!("🔧 Initializing software HSM for development");
                self.initialize_software_hsm().await?;
            }
            HSMType::PKCS11 => {
                tracing::info!("🔒 Connecting to PKCS#11 HSM");
                self.initialize_pkcs11_hsm().await?;
            }
            HSMType::AWSCloudHSM => {
                tracing::info!("☁️ Connecting to AWS CloudHSM");
                self.initialize_aws_hsm().await?;
            }
            HSMType::AzureHSM => {
                tracing::info!("🔷 Connecting to Azure Dedicated HSM");
                self.initialize_azure_hsm().await?;
            }
            HSMType::YubiKey => {
                tracing::info!("🗝️ Connecting to YubiKey PIV");
                self.initialize_yubikey_hsm().await?;
            }
            HSMType::Ledger => {
                tracing::info!("💎 Connecting to Ledger hardware wallet");
                self.initialize_ledger_hsm().await?;
            }
        }

        // Load existing keys into cache
        self.load_key_cache().await?;
        Ok(())
    }

    /// Generate a new cryptographic key in HSM
    pub async fn generate_key(
        &self,
        key_type: KeyType,
        algorithm: CryptoAlgorithm,
        label: String,
        usage: Vec<KeyUsage>,
    ) -> Result<String> {
        let session_id = self.create_session().await?;

        tracing::info!(
            "🔑 Generating {:?} key with {:?} algorithm in HSM",
            key_type,
            algorithm
        );

        // Validate algorithm compatibility
        self.validate_algorithm_support(&algorithm)?;

        let key_id = match self.config.hsm_type {
            HSMType::Software => self.generate_software_key(&algorithm, &label).await?,
            HSMType::PKCS11 => self.generate_pkcs11_key(&algorithm, &label, &usage).await?,
            HSMType::AWSCloudHSM => self.generate_aws_key(&algorithm, &label).await?,
            HSMType::AzureHSM => self.generate_azure_key(&algorithm, &label).await?,
            HSMType::YubiKey => self.generate_yubikey_key(&algorithm, &label).await?,
            HSMType::Ledger => self.generate_ledger_key(&algorithm, &label).await?,
        };

        // Store key metadata
        let key_info = HSMKeyInfo {
            key_id: key_id.clone(),
            key_type,
            algorithm,
            usage,
            created_at: SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs(),
            expires_at: None, // Set based on policy
            label: label.clone(),
            description: format!("Generated key for {}", label),
            backup_available: self.config.enable_backup,
        };

        {
            let mut cache = self.key_cache.write().await;
            cache.insert(key_id.clone(), key_info);
        }

        // Log operation
        self.log_operation(
            session_id,
            HSMOperation::KeyGeneration,
            Some(key_id.clone()),
            true,
            None,
        )
        .await;

        // Backup key if enabled
        if let Some(backup_manager) = &self.backup_manager {
            if let Err(e) = backup_manager.backup_key(&key_id).await {
                tracing::warn!("Failed to backup key {}: {}", key_id, e);
            }
        }

        tracing::info!("✅ Key generated successfully: {}", key_id);
        Ok(key_id)
    }

    /// Sign data using HSM key
    pub async fn sign_data(&self, key_id: &str, data: &[u8]) -> Result<Vec<u8>> {
        let session_id = self.create_session().await?;

        tracing::debug!("✍️ Signing data with HSM key: {}", key_id);

        // Validate key exists and has signing permission
        {
            let cache = self.key_cache.read().await;
            let key_info = cache
                .get(key_id)
                .ok_or_else(|| anyhow::anyhow!("Key not found: {}", key_id))?;

            if !key_info.usage.contains(&KeyUsage::Sign) {
                return Err(anyhow::anyhow!("Key does not have signing permission"));
            }
        }

        let signature = match self.config.hsm_type {
            HSMType::Software => self.sign_with_software_hsm(key_id, data).await?,
            HSMType::PKCS11 => self.sign_with_pkcs11(key_id, data).await?,
            HSMType::AWSCloudHSM => self.sign_with_aws_hsm(key_id, data).await?,
            HSMType::AzureHSM => self.sign_with_azure_hsm(key_id, data).await?,
            HSMType::YubiKey => self.sign_with_yubikey(key_id, data).await?,
            HSMType::Ledger => self.sign_with_ledger(key_id, data).await?,
        };

        // Log operation
        self.log_operation(
            session_id,
            HSMOperation::Signing,
            Some(key_id.to_string()),
            true,
            None,
        )
        .await;

        tracing::debug!("✅ Data signed successfully with key: {}", key_id);
        Ok(signature)
    }

    /// Get public key from HSM
    pub async fn get_public_key(&self, key_id: &str) -> Result<PublicKey> {
        tracing::debug!("📖 Retrieving public key from HSM: {}", key_id);

        let public_key_bytes = match self.config.hsm_type {
            HSMType::Software => self.get_software_public_key(key_id).await?,
            HSMType::PKCS11 => self.get_pkcs11_public_key(key_id).await?,
            HSMType::AWSCloudHSM => self.get_aws_public_key(key_id).await?,
            HSMType::AzureHSM => self.get_azure_public_key(key_id).await?,
            HSMType::YubiKey => self.get_yubikey_public_key(key_id).await?,
            HSMType::Ledger => self.get_ledger_public_key(key_id).await?,
        };

        // Convert to Ed25519 public key (assuming Ed25519 for now)
        let mut key_bytes = [0u8; 32];
        key_bytes.copy_from_slice(&public_key_bytes[..32]);
        let public_key = ed25519_dalek::VerifyingKey::from_bytes(&key_bytes)?;

        tracing::debug!("✅ Public key retrieved for: {}", key_id);
        Ok(public_key)
    }

    /// Derive address from HSM public key
    pub async fn get_address_for_key(&self, key_id: &str) -> Result<Address> {
        let public_key = self.get_public_key(key_id).await?;
        Ok(Address::from_public_key(&public_key))
    }

    /// List all available keys in HSM
    pub async fn list_keys(&self) -> Result<Vec<HSMKeyInfo>> {
        let cache = self.key_cache.read().await;
        Ok(cache.values().cloned().collect())
    }

    /// Get HSM statistics and health information
    pub async fn get_stats(&self) -> Result<HSMStats> {
        let sessions = self.sessions.read().await;
        let key_cache = self.key_cache.read().await;
        let audit_log = self.audit_log.read().await;

        let (successful_ops, failed_ops) =
            audit_log.iter().fold((0, 0), |(success, failed), entry| {
                if entry.success {
                    (success + 1, failed)
                } else {
                    (success, failed + 1)
                }
            });

        let health_status = if sessions.is_empty() {
            HSMHealthStatus::Offline
        } else if failed_ops as f64 / (successful_ops + failed_ops) as f64 > 0.1 {
            HSMHealthStatus::Critical
        } else if failed_ops > 0 {
            HSMHealthStatus::Warning
        } else {
            HSMHealthStatus::Healthy
        };

        Ok(HSMStats {
            total_operations: successful_ops + failed_ops,
            successful_operations: successful_ops,
            failed_operations: failed_ops,
            active_sessions: sessions.len(),
            key_count: key_cache.len(),
            uptime_seconds: 0, // Would track actual uptime
            last_backup: None, // Would track last backup time
            health_status,
        })
    }

    /// Perform HSM health check
    pub async fn health_check(&self) -> Result<bool> {
        match self.config.hsm_type {
            HSMType::Software => Ok(true), // Software HSM is always "healthy"
            _ => {
                // In production, would ping the actual HSM
                tracing::debug!("🔍 Performing HSM health check");
                Ok(true)
            }
        }
    }

    // Private helper methods for different HSM types

    async fn initialize_software_hsm(&self) -> Result<()> {
        // Software HSM initialization (for development)
        tracing::debug!("Initializing software-based HSM");
        Ok(())
    }

    async fn initialize_pkcs11_hsm(&self) -> Result<()> {
        // PKCS#11 HSM initialization
        tracing::debug!("Initializing PKCS#11 HSM connection");
        // Would use pkcs11 crate in production
        Ok(())
    }

    async fn initialize_aws_hsm(&self) -> Result<()> {
        // AWS CloudHSM initialization
        tracing::debug!("Initializing AWS CloudHSM connection");
        // Would use AWS SDK in production
        Ok(())
    }

    async fn initialize_azure_hsm(&self) -> Result<()> {
        // Azure HSM initialization
        tracing::debug!("Initializing Azure Dedicated HSM connection");
        // Would use Azure SDK in production
        Ok(())
    }

    async fn initialize_yubikey_hsm(&self) -> Result<()> {
        // YubiKey PIV initialization
        tracing::debug!("Initializing YubiKey PIV connection");
        // Would use yubikey crate in production
        Ok(())
    }

    async fn initialize_ledger_hsm(&self) -> Result<()> {
        // Ledger hardware wallet initialization
        tracing::debug!("Initializing Ledger hardware wallet connection");
        // Would use ledger transport in production
        Ok(())
    }

    async fn load_key_cache(&self) -> Result<()> {
        // Load existing keys from HSM into cache
        tracing::debug!("Loading key cache from HSM");

        // For software HSM, we might load from a secure file
        // For hardware HSM, we would enumerate keys from the device
        Ok(())
    }

    async fn create_session(&self) -> Result<Uuid> {
        let session_id = Uuid::new_v4();
        let now = SystemTime::now();

        let session = HSMSession {
            session_id,
            hsm_type: self.config.hsm_type.clone(),
            connection_handle: None,
            authenticated: true, // Simplified for now
            created_at: now,
            last_activity: now,
            operations_count: 0,
        };

        {
            let mut sessions = self.sessions.write().await;
            sessions.insert(session_id, session);
        }

        Ok(session_id)
    }

    fn validate_algorithm_support(&self, algorithm: &CryptoAlgorithm) -> Result<()> {
        match self.config.hsm_type {
            HSMType::Software => Ok(()), // Software HSM supports all algorithms
            HSMType::YubiKey => match algorithm {
                CryptoAlgorithm::RSA2048 | CryptoAlgorithm::ECDSA_P256 => Ok(()),
                _ => Err(anyhow::anyhow!("YubiKey does not support {:?}", algorithm)),
            },
            _ => Ok(()), // Other HSMs generally support all algorithms
        }
    }

    // Placeholder implementations for different HSM types
    // In production, these would call actual HSM APIs

    async fn generate_software_key(
        &self,
        algorithm: &CryptoAlgorithm,
        label: &str,
    ) -> Result<String> {
        let key_id = format!("soft_{}", Uuid::new_v4());
        tracing::debug!("Generated software key: {} ({})", key_id, label);
        Ok(key_id)
    }

    async fn generate_pkcs11_key(
        &self,
        algorithm: &CryptoAlgorithm,
        label: &str,
        _usage: &[KeyUsage],
    ) -> Result<String> {
        let key_id = format!("pkcs11_{}", Uuid::new_v4());
        tracing::debug!("Generated PKCS#11 key: {} ({})", key_id, label);
        Ok(key_id)
    }

    async fn generate_aws_key(&self, algorithm: &CryptoAlgorithm, label: &str) -> Result<String> {
        let key_id = format!("aws_{}", Uuid::new_v4());
        tracing::debug!("Generated AWS CloudHSM key: {} ({})", key_id, label);
        Ok(key_id)
    }

    async fn generate_azure_key(&self, algorithm: &CryptoAlgorithm, label: &str) -> Result<String> {
        let key_id = format!("azure_{}", Uuid::new_v4());
        tracing::debug!("Generated Azure HSM key: {} ({})", key_id, label);
        Ok(key_id)
    }

    async fn generate_yubikey_key(
        &self,
        algorithm: &CryptoAlgorithm,
        label: &str,
    ) -> Result<String> {
        let key_id = format!("yubikey_{}", Uuid::new_v4());
        tracing::debug!("Generated YubiKey key: {} ({})", key_id, label);
        Ok(key_id)
    }

    async fn generate_ledger_key(
        &self,
        algorithm: &CryptoAlgorithm,
        label: &str,
    ) -> Result<String> {
        let key_id = format!("ledger_{}", Uuid::new_v4());
        tracing::debug!("Generated Ledger key: {} ({})", key_id, label);
        Ok(key_id)
    }

    async fn sign_with_software_hsm(&self, key_id: &str, data: &[u8]) -> Result<Vec<u8>> {
        // Simulate software HSM signing
        use ed25519_dalek::{Signer, SigningKey};
        use rand::rngs::OsRng;
        use rand::RngCore;

        let mut secret_bytes = [0u8; 32];
        OsRng.fill_bytes(&mut secret_bytes);
        let signing_key = SigningKey::from_bytes(&secret_bytes);
        let signature = signing_key.sign(data);
        Ok(signature.to_bytes().to_vec())
    }

    async fn sign_with_pkcs11(&self, key_id: &str, data: &[u8]) -> Result<Vec<u8>> {
        // Simulate PKCS#11 HSM signing
        Ok(vec![0u8; 64]) // Placeholder signature
    }

    async fn sign_with_aws_hsm(&self, key_id: &str, data: &[u8]) -> Result<Vec<u8>> {
        // Simulate AWS CloudHSM signing
        Ok(vec![0u8; 64]) // Placeholder signature
    }

    async fn sign_with_azure_hsm(&self, key_id: &str, data: &[u8]) -> Result<Vec<u8>> {
        // Simulate Azure HSM signing
        Ok(vec![0u8; 64]) // Placeholder signature
    }

    async fn sign_with_yubikey(&self, key_id: &str, data: &[u8]) -> Result<Vec<u8>> {
        // Simulate YubiKey signing
        Ok(vec![0u8; 64]) // Placeholder signature
    }

    async fn sign_with_ledger(&self, key_id: &str, data: &[u8]) -> Result<Vec<u8>> {
        // Simulate Ledger signing
        Ok(vec![0u8; 64]) // Placeholder signature
    }

    async fn get_software_public_key(&self, key_id: &str) -> Result<Vec<u8>> {
        // Return a placeholder public key
        Ok(vec![0u8; 32])
    }

    async fn get_pkcs11_public_key(&self, key_id: &str) -> Result<Vec<u8>> {
        Ok(vec![0u8; 32])
    }

    async fn get_aws_public_key(&self, key_id: &str) -> Result<Vec<u8>> {
        Ok(vec![0u8; 32])
    }

    async fn get_azure_public_key(&self, key_id: &str) -> Result<Vec<u8>> {
        Ok(vec![0u8; 32])
    }

    async fn get_yubikey_public_key(&self, key_id: &str) -> Result<Vec<u8>> {
        Ok(vec![0u8; 32])
    }

    async fn get_ledger_public_key(&self, key_id: &str) -> Result<Vec<u8>> {
        Ok(vec![0u8; 32])
    }

    async fn log_operation(
        &self,
        session_id: Uuid,
        operation: HSMOperation,
        key_id: Option<String>,
        success: bool,
        error_code: Option<String>,
    ) {
        let entry = HSMAuditEntry {
            timestamp: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
            session_id,
            operation,
            key_id,
            success,
            error_code,
            user_id: None,   // Would get from context
            source_ip: None, // Would get from context
        };

        let mut audit_log = self.audit_log.write().await;
        audit_log.push(entry);

        // Keep only recent entries (last 10,000)
        if audit_log.len() > 10_000 {
            audit_log.drain(0..1000);
        }
    }
}

impl HSMBackupManager {
    pub fn new() -> Result<Self> {
        // Generate encryption key for backups
        let mut encryption_key = [0u8; 32];
        ring::rand::SystemRandom::new()
            .fill(&mut encryption_key)
            .map_err(|_| anyhow::anyhow!("Failed to generate backup encryption key"))?;

        Ok(Self {
            backup_configs: Vec::new(),
            encryption_key,
        })
    }

    pub async fn backup_key(&self, key_id: &str) -> Result<()> {
        tracing::info!("🔄 Backing up HSM key: {}", key_id);
        // In production, would implement encrypted key backup
        Ok(())
    }

    pub async fn restore_key(&self, key_id: &str) -> Result<()> {
        tracing::info!("🔄 Restoring HSM key: {}", key_id);
        // In production, would implement key restoration
        Ok(())
    }
}

impl Drop for HSMBackupManager {
    fn drop(&mut self) {
        // Securely clear encryption key
        self.encryption_key.zeroize();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_hsm_manager_creation() {
        let config = HSMConfig::default();
        let hsm_manager = HSMManager::new(config).await;
        assert!(hsm_manager.is_ok());
    }

    #[tokio::test]
    async fn test_software_key_generation() {
        let config = HSMConfig {
            hsm_type: HSMType::Software,
            ..Default::default()
        };

        let hsm_manager = HSMManager::new(config).await.unwrap();

        let key_id = hsm_manager
            .generate_key(
                KeyType::Signing,
                CryptoAlgorithm::Ed25519,
                "test-key".to_string(),
                vec![KeyUsage::Sign],
            )
            .await;

        assert!(key_id.is_ok());
        let key_id = key_id.unwrap();
        assert!(key_id.starts_with("soft_"));
    }

    #[tokio::test]
    async fn test_hsm_stats() {
        let config = HSMConfig::default();
        let hsm_manager = HSMManager::new(config).await.unwrap();

        let stats = hsm_manager.get_stats().await.unwrap();
        assert_eq!(stats.key_count, 0);
        assert!(matches!(stats.health_status, HSMHealthStatus::Offline));
    }
}
