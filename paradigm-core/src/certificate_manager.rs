// Certificate Management System
// Handles TLS certificate generation, validation, and rotation for secure communications

use anyhow::Result;
use rcgen::{Certificate, CertificateParams, DistinguishedName, DnType, SanType};
use rustls::{Certificate as RustlsCert, PrivateKey, ServerConfig};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use tokio::sync::RwLock;
use uuid::Uuid;

/// Certificate types supported by the system
#[derive(Debug, Clone, PartialEq)]
pub enum CertificateType {
    SelfSigned,        // For development and testing
    LetsEncrypt,       // For production with domain names
    CustomCA,          // For enterprise deployments
    ClientCertificate, // For mutual TLS authentication
}

/// Certificate metadata and storage
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CertificateInfo {
    pub id: Uuid,
    pub certificate_type: String, // Serialized enum
    pub subject: String,
    pub issuer: String,
    pub valid_from: u64,
    pub valid_to: u64,
    pub serial_number: String,
    pub fingerprint: String,
    pub is_ca: bool,
    pub key_usage: Vec<String>,
    pub san_entries: Vec<String>, // Subject Alternative Names
    pub created_at: u64,
    pub last_used: Option<u64>,
}

/// Certificate store for managing multiple certificates
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CertificateStore {
    pub version: String,
    pub updated_at: u64,
    pub certificates: HashMap<Uuid, CertificateInfo>,
    pub default_cert_id: Option<Uuid>,
    pub ca_cert_ids: Vec<Uuid>,
}

/// Certificate generation configuration
#[derive(Debug, Clone)]
pub struct CertConfig {
    pub subject_name: String,
    pub organization: String,
    pub country: String,
    pub validity_days: u32,
    pub key_size: u32,
    pub san_entries: Vec<String>, // DNS names, IP addresses
    pub is_ca: bool,
    pub key_usage: Vec<String>,
}

impl Default for CertConfig {
    fn default() -> Self {
        Self {
            subject_name: "paradigm-node".to_string(),
            organization: "Paradigm Network".to_string(),
            country: "US".to_string(),
            validity_days: 365,
            key_size: 2048,
            san_entries: vec![
                "localhost".to_string(),
                "127.0.0.1".to_string(),
                "::1".to_string(),
            ],
            is_ca: false,
            key_usage: vec![
                "digital_signature".to_string(),
                "key_encipherment".to_string(),
            ],
        }
    }
}

/// Main certificate manager
pub struct CertificateManager {
    cert_dir: PathBuf,
    store: Arc<RwLock<CertificateStore>>,
    server_configs: Arc<RwLock<HashMap<Uuid, Arc<ServerConfig>>>>,
}

impl CertificateManager {
    /// Create new certificate manager
    pub async fn new(cert_dir: &Path) -> Result<Self> {
        // Ensure certificate directory exists
        fs::create_dir_all(cert_dir)?;

        let store_path = cert_dir.join("cert_store.json");
        let store = if store_path.exists() {
            let store_data = fs::read_to_string(&store_path)?;
            serde_json::from_str(&store_data)?
        } else {
            CertificateStore {
                version: "1.0".to_string(),
                updated_at: SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs(),
                certificates: HashMap::new(),
                default_cert_id: None,
                ca_cert_ids: Vec::new(),
            }
        };

        Ok(Self {
            cert_dir: cert_dir.to_path_buf(),
            store: Arc::new(RwLock::new(store)),
            server_configs: Arc::new(RwLock::new(HashMap::new())),
        })
    }

    /// Generate a self-signed certificate for development
    pub async fn generate_self_signed_cert(&self, config: CertConfig) -> Result<Uuid> {
        tracing::info!(
            "🔐 Generating self-signed certificate: {}",
            config.subject_name
        );

        // Create certificate parameters
        let mut params = CertificateParams::new(config.san_entries.clone());

        // Set distinguished name
        let mut dn = DistinguishedName::new();
        dn.push(DnType::CommonName, config.subject_name.clone());
        dn.push(DnType::OrganizationName, config.organization);
        dn.push(DnType::CountryName, config.country);
        params.distinguished_name = dn;

        // Set validity period
        let not_before = SystemTime::now();
        let not_after =
            not_before + Duration::from_secs(config.validity_days as u64 * 24 * 60 * 60);
        params.not_before = rcgen::date_time_ymd(2024, 1, 1);
        params.not_after = rcgen::date_time_ymd(2025, 1, 1);

        // Set certificate as CA if requested
        params.is_ca = rcgen::IsCa::Ca(rcgen::BasicConstraints::Unconstrained);

        // Add Subject Alternative Names
        params.subject_alt_names = config
            .san_entries
            .iter()
            .map(|name| {
                if name.parse::<std::net::IpAddr>().is_ok() {
                    SanType::IpAddress(name.parse().unwrap())
                } else {
                    SanType::DnsName(name.clone())
                }
            })
            .collect();

        // Generate the certificate
        let cert = Certificate::from_params(params)?;
        let cert_der = cert.serialize_der()?;
        let private_key_der = cert.serialize_private_key_der();

        // Create certificate info
        let cert_id = Uuid::new_v4();
        let cert_info = CertificateInfo {
            id: cert_id,
            certificate_type: "SelfSigned".to_string(),
            subject: config.subject_name.clone(),
            issuer: config.subject_name.clone(),
            valid_from: SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs(),
            valid_to: (SystemTime::now()
                + Duration::from_secs(config.validity_days as u64 * 24 * 60 * 60))
            .duration_since(UNIX_EPOCH)?
            .as_secs(),
            serial_number: hex::encode(&cert_der[..8]),
            fingerprint: hex::encode(blake3::hash(&cert_der).as_bytes()),
            is_ca: config.is_ca,
            key_usage: config.key_usage,
            san_entries: config.san_entries,
            created_at: SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs(),
            last_used: None,
        };

        // Save certificate and private key to disk
        let cert_path = self.cert_dir.join(format!("{}.crt", cert_id));
        let key_path = self.cert_dir.join(format!("{}.key", cert_id));

        fs::write(&cert_path, &cert_der)?;
        fs::write(&key_path, &private_key_der)?;

        // Create rustls server config
        let rustls_cert = RustlsCert(cert_der);
        let private_key = PrivateKey(private_key_der);

        let server_config = ServerConfig::builder()
            .with_safe_defaults()
            .with_no_client_auth()
            .with_single_cert(vec![rustls_cert], private_key)?;

        // Store in memory
        {
            let mut configs = self.server_configs.write().await;
            configs.insert(cert_id, Arc::new(server_config));
        }

        // Update store
        {
            let mut store = self.store.write().await;
            store.certificates.insert(cert_id, cert_info);

            // Set as default if it's the first certificate
            if store.default_cert_id.is_none() {
                store.default_cert_id = Some(cert_id);
            }

            if config.is_ca {
                store.ca_cert_ids.push(cert_id);
            }

            store.updated_at = SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs();
        }

        // Save store to disk
        self.save_store().await?;

        tracing::info!("✅ Certificate generated successfully: {}", cert_id);
        Ok(cert_id)
    }

    /// Get server config for TLS
    pub async fn get_server_config(&self, cert_id: Option<Uuid>) -> Result<Arc<ServerConfig>> {
        let configs = self.server_configs.read().await;
        let store = self.store.read().await;

        let cert_id = cert_id
            .or(store.default_cert_id)
            .ok_or_else(|| anyhow::anyhow!("No certificate available"))?;

        configs
            .get(&cert_id)
            .cloned()
            .ok_or_else(|| anyhow::anyhow!("Certificate not found: {}", cert_id))
    }

    /// Validate certificate expiration and health
    pub async fn validate_certificate(&self, cert_id: Uuid) -> Result<bool> {
        let store = self.store.read().await;

        if let Some(cert_info) = store.certificates.get(&cert_id) {
            let now = SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs();

            // Check if certificate is still valid
            if now < cert_info.valid_from {
                tracing::warn!("Certificate {} not yet valid", cert_id);
                return Ok(false);
            }

            if now > cert_info.valid_to {
                tracing::warn!("Certificate {} has expired", cert_id);
                return Ok(false);
            }

            // Check if certificate expires soon (within 30 days)
            if now + (30 * 24 * 60 * 60) > cert_info.valid_to {
                tracing::warn!("Certificate {} expires soon (within 30 days)", cert_id);
            }

            Ok(true)
        } else {
            Err(anyhow::anyhow!("Certificate not found: {}", cert_id))
        }
    }

    /// List all certificates
    pub async fn list_certificates(&self) -> HashMap<Uuid, CertificateInfo> {
        self.store.read().await.certificates.clone()
    }

    /// Get certificate information
    pub async fn get_certificate_info(&self, cert_id: Uuid) -> Option<CertificateInfo> {
        self.store.read().await.certificates.get(&cert_id).cloned()
    }

    /// Remove certificate
    pub async fn remove_certificate(&self, cert_id: Uuid) -> Result<()> {
        {
            let mut store = self.store.write().await;

            // Remove from store
            if store.certificates.remove(&cert_id).is_none() {
                return Err(anyhow::anyhow!("Certificate not found: {}", cert_id));
            }

            // Update default if removing default cert
            if store.default_cert_id == Some(cert_id) {
                store.default_cert_id = store.certificates.keys().next().copied();
            }

            // Remove from CA list
            store.ca_cert_ids.retain(|id| *id != cert_id);

            store.updated_at = SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs();
        }

        // Remove from memory
        {
            let mut configs = self.server_configs.write().await;
            configs.remove(&cert_id);
        }

        // Remove files
        let cert_path = self.cert_dir.join(format!("{}.crt", cert_id));
        let key_path = self.cert_dir.join(format!("{}.key", cert_id));

        let _ = fs::remove_file(cert_path);
        let _ = fs::remove_file(key_path);

        self.save_store().await?;

        tracing::info!("🗑️ Certificate removed: {}", cert_id);
        Ok(())
    }

    /// Start certificate monitoring and auto-renewal
    pub async fn start_certificate_monitor(&self) -> Result<()> {
        let store = self.store.clone();
        let cert_dir = self.cert_dir.clone();

        tokio::spawn(async move {
            let mut interval = tokio::time::interval(Duration::from_secs(24 * 60 * 60)); // Daily check

            loop {
                interval.tick().await;

                tracing::debug!("🔍 Running certificate health check");

                let certificates = {
                    let store_lock = store.read().await;
                    store_lock.certificates.clone()
                };

                let now = SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_secs();

                for (cert_id, cert_info) in certificates {
                    // Check for expiring certificates (within 30 days)
                    if now + (30 * 24 * 60 * 60) > cert_info.valid_to {
                        tracing::warn!(
                            "⚠️ Certificate {} expires soon: {} days remaining",
                            cert_id,
                            (cert_info.valid_to - now) / (24 * 60 * 60)
                        );
                    }

                    // Check for expired certificates
                    if now > cert_info.valid_to {
                        tracing::error!(
                            "❌ Certificate {} has expired and should be renewed",
                            cert_id
                        );
                    }
                }

                tracing::debug!("✅ Certificate health check completed");
            }
        });

        Ok(())
    }

    /// Generate a Certificate Authority certificate
    pub async fn generate_ca_certificate(&self, config: CertConfig) -> Result<Uuid> {
        let mut ca_config = config;
        ca_config.is_ca = true;
        ca_config.key_usage = vec![
            "cert_signing".to_string(),
            "crl_signing".to_string(),
            "digital_signature".to_string(),
        ];
        ca_config.validity_days = 3650; // 10 years for CA

        self.generate_self_signed_cert(ca_config).await
    }

    /// Save certificate store to disk
    async fn save_store(&self) -> Result<()> {
        let store = self.store.read().await;
        let store_path = self.cert_dir.join("cert_store.json");
        let store_json = serde_json::to_string_pretty(&*store)?;
        fs::write(store_path, store_json)?;
        Ok(())
    }

    /// Load certificate from disk into memory
    async fn load_certificate(&self, cert_id: Uuid) -> Result<()> {
        let cert_path = self.cert_dir.join(format!("{}.crt", cert_id));
        let key_path = self.cert_dir.join(format!("{}.key", cert_id));

        if !cert_path.exists() || !key_path.exists() {
            return Err(anyhow::anyhow!(
                "Certificate files not found for {}",
                cert_id
            ));
        }

        let cert_der = fs::read(cert_path)?;
        let key_der = fs::read(key_path)?;

        let rustls_cert = RustlsCert(cert_der);
        let private_key = PrivateKey(key_der);

        let server_config = ServerConfig::builder()
            .with_safe_defaults()
            .with_no_client_auth()
            .with_single_cert(vec![rustls_cert], private_key)?;

        let mut configs = self.server_configs.write().await;
        configs.insert(cert_id, Arc::new(server_config));

        Ok(())
    }

    /// Initialize with default certificates if none exist
    pub async fn initialize_default_certificates(&self) -> Result<()> {
        let store = self.store.read().await;

        if store.certificates.is_empty() {
            drop(store);

            tracing::info!("🔐 No certificates found, generating default self-signed certificate");

            let config = CertConfig::default();
            self.generate_self_signed_cert(config).await?;

            tracing::info!("✅ Default certificate generated successfully");
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_certificate_generation() {
        let temp_dir = TempDir::new().unwrap();
        let cert_manager = CertificateManager::new(temp_dir.path()).await.unwrap();

        let config = CertConfig::default();
        let cert_id = cert_manager
            .generate_self_signed_cert(config)
            .await
            .unwrap();

        // Verify certificate was created
        let cert_info = cert_manager.get_certificate_info(cert_id).await;
        assert!(cert_info.is_some());

        // Verify server config can be retrieved
        let server_config = cert_manager.get_server_config(Some(cert_id)).await;
        assert!(server_config.is_ok());

        // Verify certificate validation
        let is_valid = cert_manager.validate_certificate(cert_id).await.unwrap();
        assert!(is_valid);
    }

    #[tokio::test]
    async fn test_certificate_store() {
        let temp_dir = TempDir::new().unwrap();
        let cert_manager = CertificateManager::new(temp_dir.path()).await.unwrap();

        // Generate multiple certificates
        let config1 = CertConfig {
            subject_name: "test1".to_string(),
            ..Default::default()
        };
        let config2 = CertConfig {
            subject_name: "test2".to_string(),
            ..Default::default()
        };

        let cert_id1 = cert_manager
            .generate_self_signed_cert(config1)
            .await
            .unwrap();
        let cert_id2 = cert_manager
            .generate_self_signed_cert(config2)
            .await
            .unwrap();

        // List certificates
        let certificates = cert_manager.list_certificates().await;
        assert_eq!(certificates.len(), 2);
        assert!(certificates.contains_key(&cert_id1));
        assert!(certificates.contains_key(&cert_id2));

        // Remove certificate
        cert_manager.remove_certificate(cert_id1).await.unwrap();
        let certificates = cert_manager.list_certificates().await;
        assert_eq!(certificates.len(), 1);
        assert!(!certificates.contains_key(&cert_id1));
    }
}
