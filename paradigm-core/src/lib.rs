use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use chrono::Utc;
use serde::{Deserialize, Serialize};
use blake3::Hasher;
use uuid::Uuid;

/// The core Paradigm network node implementation
pub mod error;
pub mod network;
pub mod consensus;
pub mod transaction;
pub mod wallet;
pub mod ml_tasks;
pub mod storage;
pub mod governance;

// Type aliases for easier use
pub type Address = String;
pub type Hash = [u8; 32];
pub type Amount = u64;
pub type PublicKey = ed25519_dalek::VerifyingKey;
pub type SecretKey = ed25519_dalek::SigningKey;
pub type Keypair = ed25519_dalek::SigningKey; // In the new API, SigningKey is the keypair

// Re-export commonly used types
pub use error::ParadigmError;
pub use transaction::Transaction;
pub use consensus::MLTask;
pub use network::{P2PNetwork, NetworkMessage};
pub use wallet::Wallet;
pub use governance::{ProposalStatus, ProposalType};

pub const PARADIGM_VERSION: &str = "0.1.0";

/// Extension trait for Address operations
pub trait AddressExt {
    fn from_public_key(public_key: &PublicKey) -> Self;
}

impl AddressExt for Address {
    fn from_public_key(public_key: &PublicKey) -> Self {
        let mut hasher = Hasher::new();
        hasher.update(public_key.as_bytes());
        let hash = hasher.finalize();
        format!("PAR{}", hex::encode(&hash.as_bytes()[..20]))
    }
}

/// Network configuration constants
pub const PARADIGM_PROTOCOL_VERSION: &str = "paradigm/1.0.0";
pub const DEFAULT_PORT: u16 = 8080;
pub const MAX_PEERS: usize = 50;

/// Economic constants
pub const TOTAL_SUPPLY: u64 = 8_000_000_000_00000000; // 8 billion PAR with 8 decimal places
pub const DECIMALS: u8 = 8;
pub const FIRST_YEAR_DISTRIBUTION: u64 = 100_000_000_00000000; // 100 million PAR in first year

/// ML Task and Consensus Constants
pub const MIN_TASK_DIFFICULTY: u32 = 1;
pub const MAX_TASK_DIFFICULTY: u32 = 10;
pub const BASE_REWARD: u64 = 100_00000000; // 100 PAR base reward
pub const CONSENSUS_TIMEOUT_SECS: u64 = 30;

/// Core Paradigm Node
pub struct ParadigmNode {
    pub config: NodeConfig,
    pub network: Arc<RwLock<network::P2PNetwork>>,
    pub consensus: Arc<RwLock<consensus::ConsensusEngine>>,
    pub storage: Arc<RwLock<storage::ParadigmStorage>>,
    pub governance: Arc<RwLock<governance::AIGovernance>>,
    pub keypair: Keypair,
}

#[derive(Debug, Clone)]
pub struct NodeConfig {
    pub node_id: uuid::Uuid,
    pub network_port: u16,
    pub data_dir: String,
    pub enable_ml_tasks: bool,
    pub max_peers: usize,
}

impl Default for NodeConfig {
    fn default() -> Self {
        Self {
            node_id: uuid::Uuid::new_v4(),
            network_port: DEFAULT_PORT,
            data_dir: "./paradigm_data".to_string(),
            enable_ml_tasks: true,
            max_peers: MAX_PEERS,
        }
    }
}

impl ParadigmNode {
    pub async fn new(config: NodeConfig) -> anyhow::Result<Self> {
        use rand::rngs::OsRng;
        use rand::RngCore;
        
        // Generate random bytes for the keypair
        let mut secret_bytes = [0u8; 32];
        OsRng.fill_bytes(&mut secret_bytes);
        let keypair = ed25519_dalek::SigningKey::from_bytes(&secret_bytes);
        let node_id = uuid::Uuid::new_v4();
        
        // Construct proper database URL with absolute path
        let data_path = std::path::Path::new(&config.data_dir);
        let absolute_data_path = if data_path.is_absolute() {
            data_path.to_path_buf()
        } else {
            std::env::current_dir()?.join(data_path)
        };
        let db_path = absolute_data_path.join("paradigm.db");
        let db_url = format!("sqlite://{}", db_path.to_string_lossy().replace('\\', "/"));
        
        let storage = Arc::new(RwLock::new(
            storage::ParadigmStorage::new(&db_url).await?
        ));

        let network = Arc::new(RwLock::new(
            network::P2PNetwork::new(node_id).await?
        ));

        let consensus = Arc::new(RwLock::new(
            consensus::ConsensusEngine::new()
        ));

        let governance = Arc::new(RwLock::new(
            governance::AIGovernance::new()
        ));

        Ok(ParadigmNode {
            config,
            network,
            consensus,
            storage,
            governance,
            keypair,
        })
    }

    pub async fn start(&mut self) -> anyhow::Result<()> {
        tracing::info!("Starting Paradigm node with ID: {}", self.config.node_id);
        
        // Start network layer
        {
            let mut network = self.network.write().await;
            network.start_listening().await?;
        }

        tracing::info!("Paradigm node started successfully");
        Ok(())
    }

    pub async fn stop(&mut self) -> anyhow::Result<()> {
        tracing::info!("Stopping Paradigm node");
        
        // Graceful shutdown
        {
            let mut network = self.network.write().await;
            // network.stop().await?; // TODO: implement stop method
        }

        tracing::info!("Paradigm node stopped");
        Ok(())
    }

    pub fn get_address(&self) -> Address {
        AddressExt::from_public_key(&self.keypair.verifying_key())
    }

    pub async fn submit_transaction(&self, transaction: transaction::Transaction) -> anyhow::Result<()> {
        // Store the transaction in storage (simplified approach)
        let mut storage = self.storage.write().await;
        storage.store_transaction(&transaction).await?;
        
        // Broadcast to network
        let mut network = self.network.write().await;
        network.broadcast_transaction(&transaction).await?;
        
        Ok(())
    }

    pub async fn get_balance(&self, address: &Address) -> anyhow::Result<Amount> {
        let storage = self.storage.read().await;
        storage.get_balance(address).await
    }

    pub async fn get_transaction_history(&self, address: &Address) -> anyhow::Result<Vec<transaction::Transaction>> {
        let storage = self.storage.read().await;
        storage.get_transactions_for_address(address).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_node_creation() {
        let config = NodeConfig::default();
        let node = ParadigmNode::new(config).await;
        assert!(node.is_ok());
    }

    #[test]
    fn test_address_generation() {
        let keypair = ed25519_dalek::SigningKey::generate(&mut rand::rngs::OsRng);
        let address = Address::from_public_key(&keypair.verifying_key());
        assert!(address.starts_with("PAR"));
        assert_eq!(address.len(), 43); // "PAR" + 40 hex chars
    }

    #[test]
    fn test_constants() {
        assert_eq!(TOTAL_SUPPLY, 8_000_000_000_00000000);
        assert_eq!(DECIMALS, 8);
        assert_eq!(DEFAULT_PORT, 8080);
    }
}
