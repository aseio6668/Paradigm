// Paradigm SDK - Developer Tools and Client Libraries
// Comprehensive toolkit for building applications on the Paradigm network

#![warn(missing_docs)]
#![warn(clippy::all)]
#![deny(unsafe_code)]

//! # Paradigm SDK
//!
//! The Paradigm SDK provides a comprehensive set of tools and libraries for developers
//! building applications on the Paradigm cryptocurrency network. It includes:
//!
//! - **Client Libraries**: High-level APIs for interacting with the Paradigm network
//! - **Wallet Management**: Secure wallet creation, management, and transaction signing
//! - **Smart Contract Integration**: Tools for deploying and interacting with smart contracts
//! - **Cross-Chain Operations**: Utilities for cross-chain transactions and governance
//! - **Network Monitoring**: Real-time network status and analytics
//! - **Testing Framework**: Comprehensive testing tools for dApp development
//!
//! ## Quick Start
//!
//! ```rust,no_run
//! use paradigm_sdk::{ParadigmClient, NetworkConfig};
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     // Connect to Paradigm network
//!     let config = NetworkConfig::mainnet();
//!     let client = ParadigmClient::new(config).await?;
//!     
//!     // Get network status
//!     let status = client.get_network_status().await?;
//!     println!("Connected to Paradigm network: {:?}", status);
//!     
//!     Ok(())
//! }
//! ```

// Core modules
pub mod client;
pub mod contracts;
pub mod enterprise;
pub mod error;
pub mod network;
pub mod types;
pub mod utils;
pub mod wallet;

// Advanced security modules
pub mod privacy;
pub mod security;
pub mod threshold;
pub mod zkp;

// Monitoring and observability modules
pub mod monitoring;
pub mod observability;
pub mod telemetry;

// Feature-gated modules
#[cfg(feature = "testing")]
// pub mod testing; // TODO: Implement testing module

// Note: CLI and server modules will be implemented later
// #[cfg(feature = "cli")]
// pub mod cli;

// #[cfg(feature = "server")]
// pub mod server;

// Re-exports for convenience
pub use client::{ClientConfig, ParadigmClient};
pub use contracts::{Contract, ContractBuilder, ContractCall};
pub use enterprise::{
    ApiKey, ComplianceMonitor, ComplianceRule, EnterpriseApiManager, EnterpriseWalletManager,
    MultisigWallet,
};
pub use error::{ParadigmError, Result};
pub use network::{NetworkConfig, NetworkStatus, PeerInfo};
pub use types::{Address, Amount, Balance, Block, Fee, Hash, Signature, TokenInfo, Transaction};
pub use wallet::{Wallet, WalletManager};

// Advanced security features
pub use privacy::{ConfidentialTransaction, PrivacyCoin, RingSignature, StealthAddress};
pub use security::{AnomalyDetector, SecurityAudit, SecurityMonitor, ThreatIntelligence};
pub use threshold::{MultiSigWallet, SecretShare, ThresholdCrypto, ThresholdSignature};
pub use zkp::{MerkleTree, PrivateTransaction, RangeProof, ZKProof};

// Monitoring and observability features
pub use monitoring::{HealthMonitor, MetricsCollector, MonitoringConfig, MonitoringSystem};
pub use observability::{
    AnalyticsEngine, ObservabilityConfig, ObservabilityPlatform, SystemInsights,
};
pub use telemetry::{DistributedTracer, StructuredLog, TelemetryConfig, TelemetrySystem};

// Version information
/// SDK version
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Git commit hash (if available)
pub const GIT_HASH: Option<&str> = option_env!("GIT_HASH");

/// Build timestamp (fallback if not available)
pub const BUILD_TIME: &str = "unknown";

/// Mainnet chain ID (re-exported for convenience)
pub const MAINNET_CHAIN_ID: u64 = constants::MAINNET_CHAIN_ID;

/// Paradigm network constants
pub mod constants {
    /// Mainnet chain ID
    pub const MAINNET_CHAIN_ID: u64 = 1;

    /// Testnet chain ID
    pub const TESTNET_CHAIN_ID: u64 = 2;

    /// Devnet chain ID
    pub const DEVNET_CHAIN_ID: u64 = 3;

    /// Default RPC port
    pub const DEFAULT_RPC_PORT: u16 = 8545;

    /// Default P2P port
    pub const DEFAULT_P2P_PORT: u16 = 30303;

    /// Default WebSocket port
    pub const DEFAULT_WS_PORT: u16 = 8546;

    /// Native token symbol
    pub const NATIVE_TOKEN_SYMBOL: &str = "PARADIGM";

    /// Native token decimals
    pub const NATIVE_TOKEN_DECIMALS: u8 = 18;

    /// Block time in seconds
    pub const BLOCK_TIME: u64 = 12;

    /// Max transaction size
    pub const MAX_TRANSACTION_SIZE: usize = 1024 * 1024; // 1MB

    /// Gas limit per block
    pub const BLOCK_GAS_LIMIT: u64 = 30_000_000;

    /// Base fee per gas (in wei)
    pub const BASE_FEE_PER_GAS: u64 = 1_000_000_000; // 1 Gwei
}

/// SDK configuration and environment
pub mod config {
    use serde::{Deserialize, Serialize};
    use std::collections::HashMap;
    use url::Url;

    /// SDK configuration
    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct SdkConfig {
        /// Network configuration
        pub network: super::NetworkConfig,

        /// API endpoints
        pub endpoints: EndpointConfig,

        /// Timeout settings
        pub timeouts: TimeoutConfig,

        /// Retry settings
        pub retry: RetryConfig,

        /// Logging configuration
        pub logging: LoggingConfig,

        /// Feature flags
        pub features: FeatureFlags,
    }

    /// API endpoint configuration
    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct EndpointConfig {
        /// Primary RPC endpoint
        pub rpc_url: Url,

        /// WebSocket endpoint
        pub ws_url: Option<Url>,

        /// GraphQL endpoint
        pub graphql_url: Option<Url>,

        /// REST API endpoint
        pub rest_url: Option<Url>,

        /// Fallback endpoints
        pub fallback_urls: Vec<Url>,

        /// API key (if required)
        pub api_key: Option<String>,

        /// Rate limiting
        pub rate_limit: Option<RateLimit>,
    }

    /// Rate limiting configuration
    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct RateLimit {
        /// Requests per second
        pub requests_per_second: u32,

        /// Burst capacity
        pub burst_capacity: u32,
    }

    /// Timeout configuration
    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct TimeoutConfig {
        /// Connection timeout
        pub connection_timeout: std::time::Duration,

        /// Request timeout
        pub request_timeout: std::time::Duration,

        /// WebSocket ping interval
        pub ws_ping_interval: std::time::Duration,

        /// Transaction confirmation timeout
        pub confirmation_timeout: std::time::Duration,
    }

    /// Retry configuration
    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct RetryConfig {
        /// Maximum number of retries
        pub max_retries: u32,

        /// Base delay between retries
        pub base_delay: std::time::Duration,

        /// Backoff multiplier
        pub backoff_multiplier: f64,

        /// Maximum delay
        pub max_delay: std::time::Duration,

        /// Retryable error codes
        pub retryable_errors: Vec<String>,
    }

    /// Logging configuration
    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct LoggingConfig {
        /// Log level
        pub level: String,

        /// Log format (json, pretty, compact)
        pub format: String,

        /// Output destination (stdout, file)
        pub output: String,

        /// Log file path (if output is file)
        pub file_path: Option<String>,

        /// Enable tracing
        pub tracing_enabled: bool,

        /// Custom log targets
        pub targets: HashMap<String, String>,
    }

    /// Feature flags
    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct FeatureFlags {
        /// Enable metrics collection
        pub metrics_enabled: bool,

        /// Enable caching
        pub caching_enabled: bool,

        /// Enable compression
        pub compression_enabled: bool,

        /// Enable experimental features
        pub experimental_features: bool,

        /// Custom feature flags
        pub custom_flags: HashMap<String, bool>,
    }

    impl Default for SdkConfig {
        fn default() -> Self {
            Self {
                network: super::NetworkConfig::mainnet(),
                endpoints: EndpointConfig {
                    rpc_url: "http://localhost:8545".parse().unwrap(),
                    ws_url: Some("ws://localhost:8546".parse().unwrap()),
                    graphql_url: None,
                    rest_url: None,
                    fallback_urls: vec![],
                    api_key: None,
                    rate_limit: Some(RateLimit {
                        requests_per_second: 10,
                        burst_capacity: 50,
                    }),
                },
                timeouts: TimeoutConfig {
                    connection_timeout: std::time::Duration::from_secs(30),
                    request_timeout: std::time::Duration::from_secs(60),
                    ws_ping_interval: std::time::Duration::from_secs(30),
                    confirmation_timeout: std::time::Duration::from_secs(300),
                },
                retry: RetryConfig {
                    max_retries: 3,
                    base_delay: std::time::Duration::from_millis(1000),
                    backoff_multiplier: 2.0,
                    max_delay: std::time::Duration::from_secs(30),
                    retryable_errors: vec![
                        "timeout".to_string(),
                        "connection_error".to_string(),
                        "rate_limited".to_string(),
                    ],
                },
                logging: LoggingConfig {
                    level: "info".to_string(),
                    format: "pretty".to_string(),
                    output: "stdout".to_string(),
                    file_path: None,
                    tracing_enabled: true,
                    targets: HashMap::new(),
                },
                features: FeatureFlags {
                    metrics_enabled: true,
                    caching_enabled: true,
                    compression_enabled: true,
                    experimental_features: false,
                    custom_flags: HashMap::new(),
                },
            }
        }
    }
}

/// SDK utilities and helpers
pub mod helpers {
    use crate::error::{ParadigmError, Result};
    use crate::types::{Address, Amount, Hash};

    /// Convert hex string to bytes
    pub fn hex_to_bytes(hex: &str) -> Result<Vec<u8>> {
        let hex = hex.strip_prefix("0x").unwrap_or(hex);
        hex::decode(hex).map_err(|e| ParadigmError::InvalidHex(e.to_string()))
    }

    /// Convert bytes to hex string
    pub fn bytes_to_hex(bytes: &[u8]) -> String {
        format!("0x{}", hex::encode(bytes))
    }

    /// Validate Paradigm address format
    pub fn validate_address(address: &str) -> Result<()> {
        if !address.starts_with("0x") {
            return Err(ParadigmError::InvalidAddress(
                "Missing 0x prefix".to_string(),
            ));
        }

        if address.len() != 42 {
            return Err(ParadigmError::InvalidAddress("Invalid length".to_string()));
        }

        hex_to_bytes(address)?;
        Ok(())
    }

    /// Generate random address (for testing)
    pub fn generate_random_address() -> Address {
        use rand::Rng;
        let mut rng = rand::thread_rng();
        let bytes: [u8; 20] = rng.gen();
        Address::from_bytes(bytes)
    }

    /// Convert wei to Paradigm tokens
    pub fn wei_to_paradigm(wei: u64) -> f64 {
        wei as f64 / 10_f64.powi(crate::constants::NATIVE_TOKEN_DECIMALS as i32)
    }

    /// Convert Paradigm tokens to wei
    pub fn paradigm_to_wei(paradigm: f64) -> u64 {
        (paradigm * 10_f64.powi(crate::constants::NATIVE_TOKEN_DECIMALS as i32)) as u64
    }

    /// Format amount for display
    pub fn format_amount(amount: Amount, decimals: u8) -> String {
        let divisor = 10_f64.powi(decimals as i32);
        let value = amount.value() as f64 / divisor;
        format!("{:.6}", value)
    }

    /// Parse amount from string
    pub fn parse_amount(amount_str: &str, decimals: u8) -> Result<Amount> {
        let value: f64 = amount_str
            .parse()
            .map_err(|_| ParadigmError::InvalidAmount("Invalid number format".to_string()))?;

        let multiplier = 10_f64.powi(decimals as i32);
        let wei_value = (value * multiplier) as u64;

        Ok(Amount::from_wei(wei_value))
    }

    /// Calculate transaction hash
    pub fn calculate_tx_hash(tx_data: &[u8]) -> Hash {
        use sha2::{Digest, Sha256};
        let hash = Sha256::digest(tx_data);
        Hash::from_bytes(hash.into())
    }

    /// Estimate gas for transaction
    pub fn estimate_gas_simple(data_size: usize) -> u64 {
        let base_gas = 21_000u64; // Base transaction cost
        let data_gas = data_size as u64 * 16; // 16 gas per byte
        base_gas + data_gas
    }

    /// Check if string is valid hex
    pub fn is_valid_hex(s: &str) -> bool {
        let s = s.strip_prefix("0x").unwrap_or(s);
        s.chars().all(|c| c.is_ascii_hexdigit())
    }

    /// Normalize address to checksum format
    pub fn to_checksum_address(address: &str) -> Result<String> {
        validate_address(address)?;
        // Simplified checksum - in real implementation would use EIP-55
        Ok(address.to_lowercase())
    }

    /// Generate unique identifier
    pub fn generate_id() -> String {
        uuid::Uuid::new_v4().to_string()
    }

    /// Current timestamp in seconds
    pub fn current_timestamp() -> u64 {
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs()
    }

    /// Format duration as human readable string
    pub fn format_duration(duration: std::time::Duration) -> String {
        let secs = duration.as_secs();
        if secs < 60 {
            format!("{}s", secs)
        } else if secs < 3600 {
            format!("{}m {}s", secs / 60, secs % 60)
        } else {
            format!("{}h {}m", secs / 3600, (secs % 3600) / 60)
        }
    }
}

/// Macros for common operations
pub mod macros {
    /// Create a new address from hex string
    #[macro_export]
    macro_rules! address {
        ($hex:expr) => {
            $crate::types::Address::from_hex($hex).expect("Invalid address hex")
        };
    }

    /// Create a new hash from hex string
    #[macro_export]
    macro_rules! hash {
        ($hex:expr) => {
            $crate::types::Hash::from_hex($hex).expect("Invalid hash hex")
        };
    }

    /// Create an amount from Paradigm tokens
    #[macro_export]
    macro_rules! paradigm {
        ($amount:expr) => {
            $crate::types::Amount::from_paradigm($amount)
        };
    }

    /// Create an amount from wei
    #[macro_export]
    macro_rules! wei {
        ($amount:expr) => {
            $crate::types::Amount::from_wei($amount)
        };
    }

    /// Assert transaction success
    #[macro_export]
    macro_rules! assert_tx_success {
        ($tx_result:expr) => {
            match $tx_result {
                Ok(receipt) => {
                    assert!(receipt.success, "Transaction failed: {:?}", receipt.error);
                }
                Err(e) => panic!("Transaction error: {:?}", e),
            }
        };
    }
}

// Re-export commonly used external types
pub use serde_json::Value as JsonValue;
pub use url::Url;
pub use uuid::Uuid;

/// Prelude module for convenient imports
pub mod prelude {
    pub use crate::{
        address, hash, paradigm, wei, Address, Amount, Balance, Block, ClientConfig,
        ComplianceMonitor, Contract, ContractBuilder, ContractCall, EnterpriseApiManager,
        EnterpriseWalletManager, Fee, Hash, NetworkConfig, NetworkStatus, ParadigmClient,
        ParadigmError, Result, Signature, TokenInfo, Transaction, Wallet, WalletManager,
    };

    pub use crate::constants::*;
    pub use crate::helpers::*;

    // Common external types
    pub use serde_json::Value as JsonValue;
    pub use url::Url;
    pub use uuid::Uuid;

    // Async runtime
    pub use tokio;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_version_constants() {
        assert!(!VERSION.is_empty());
        assert!(!BUILD_TIME.is_empty());
    }

    #[test]
    fn test_network_constants() {
        assert_eq!(constants::MAINNET_CHAIN_ID, 1);
        assert_eq!(constants::TESTNET_CHAIN_ID, 2);
        assert_eq!(constants::NATIVE_TOKEN_SYMBOL, "PARADIGM");
        assert_eq!(constants::NATIVE_TOKEN_DECIMALS, 18);
    }

    #[test]
    fn test_helpers() {
        // Test hex conversion
        let bytes = vec![0x12, 0x34, 0x56];
        let hex = helpers::bytes_to_hex(&bytes);
        assert_eq!(hex, "0x123456");

        let decoded = helpers::hex_to_bytes(&hex).unwrap();
        assert_eq!(decoded, bytes);

        // Test amount conversion
        let paradigm_amount = 1.5;
        let wei_amount = helpers::paradigm_to_wei(paradigm_amount);
        let converted_back = helpers::wei_to_paradigm(wei_amount);
        assert!((converted_back - paradigm_amount).abs() < 0.0001);
    }

    #[test]
    fn test_address_validation() {
        // Valid address
        assert!(helpers::validate_address("0x1234567890123456789012345678901234567890").is_ok());

        // Invalid addresses
        assert!(helpers::validate_address("1234567890123456789012345678901234567890").is_err()); // No 0x
        assert!(helpers::validate_address("0x12345").is_err()); // Too short
        assert!(helpers::validate_address("0x123456789012345678901234567890123456789g").is_err());
        // Invalid hex
    }

    #[test]
    fn test_macros() {
        let addr = address!("0x1234567890123456789012345678901234567890");
        assert_eq!(addr.to_hex(), "0x1234567890123456789012345678901234567890");

        let amount = paradigm!(1.5);
        assert_eq!(amount.value(), helpers::paradigm_to_wei(1.5));

        let wei_amount = wei!(1000);
        assert_eq!(wei_amount.value(), 1000);
    }
}
