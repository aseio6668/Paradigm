// Core imports - only include what's actually used
use blake3::Hasher;
use std::sync::Arc;
use tokio::sync::RwLock;

pub mod ai;
pub mod api;
pub mod consensus;
/// The core Paradigm network node implementation
pub mod error;
pub mod genesis;
pub mod governance;
pub mod ml_tasks;
pub mod network;
pub mod performance;
pub mod storage;
pub mod tokenomics;
pub mod transaction;
pub mod wallet;
pub mod wallet_manager;
pub mod transaction_tester;
pub mod autopool;
pub mod network_sync;
pub mod privacy_blockchain;
pub mod ephemeral_storage;
pub mod peer_manager;
pub mod autonomous_tasks;

// Performance optimization modules
pub mod crypto_optimization;
pub mod memory_optimization;
pub mod parallel_ml;
pub mod performance_benchmarks;
pub mod transaction_throughput;

// Type aliases for easier use
#[derive(Debug, Clone, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub struct Address(pub [u8; 32]);

impl Address {
    pub fn as_bytes(&self) -> &[u8; 32] {
        &self.0
    }

    pub fn to_string(&self) -> String {
        format!("PAR{}", hex::encode(&self.0[..20]))
    }

    pub fn from_string(addr_str: &str) -> anyhow::Result<Self> {
        if !addr_str.starts_with("PAR") {
            return Err(anyhow::anyhow!("Invalid address format: must start with PAR"));
        }
        
        let hex_part = &addr_str[3..]; // Remove "PAR" prefix
        if hex_part.len() != 40 { // 20 bytes * 2 hex chars
            return Err(anyhow::anyhow!("Invalid address length"));
        }
        
        let hex_bytes = hex::decode(hex_part)?;
        let mut addr = [0u8; 32];
        addr[..20].copy_from_slice(&hex_bytes);
        Ok(Address(addr))
    }
}

impl std::fmt::Display for Address {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.to_string())
    }
}
pub type Hash = [u8; 32];
pub type Amount = u64;
pub type PublicKey = ed25519_dalek::VerifyingKey;
pub type SecretKey = ed25519_dalek::SigningKey;
pub type Keypair = ed25519_dalek::SigningKey; // In the new API, SigningKey is the keypair

// Re-export commonly used types
pub use consensus::MLTask;
pub use error::ParadigmError;
pub use governance::{ProposalStatus, ProposalType};
pub use network::{NetworkMessage, P2PNetwork};
pub use tokenomics::{ContributionProof, ContributionType, TokenomicsSystem};
pub use transaction::Transaction;
pub use wallet::Wallet;

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
        let mut addr = [0u8; 32];
        addr.copy_from_slice(hash.as_bytes());
        Address(addr)
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
    pub tokenomics: Arc<RwLock<tokenomics::TokenomicsSystem>>,
    pub network_sync: Arc<RwLock<network_sync::NetworkSynchronizer>>,
    pub privacy_blockchain: Arc<RwLock<privacy_blockchain::PrivacyBlockchain>>,
    pub ephemeral_storage: Arc<RwLock<ephemeral_storage::EphemeralStorage>>,
    pub peer_manager: Arc<RwLock<peer_manager::PeerManager>>,
    pub autonomous_tasks: Arc<RwLock<autonomous_tasks::AutonomousTaskGenerator>>,
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
        // Ensure the data directory exists before creating database
        std::fs::create_dir_all(&absolute_data_path)?;

        let db_path = absolute_data_path.join("paradigm.db");
        let db_url = format!("sqlite://{}", db_path.to_string_lossy().replace('\\', "/"));

        let storage = Arc::new(RwLock::new(storage::ParadigmStorage::new(&db_url).await?));

        let network = Arc::new(RwLock::new(network::P2PNetwork::new(node_id).await?));

        let consensus = Arc::new(RwLock::new(consensus::ConsensusEngine::new()));

        let governance = Arc::new(RwLock::new(governance::AIGovernance::new()));

        let tokenomics = Arc::new(RwLock::new(tokenomics::TokenomicsSystem::new()));

        let network_sync = Arc::new(RwLock::new(
            network_sync::NetworkSynchronizer::new(storage.clone())
        ));

        let privacy_blockchain = Arc::new(RwLock::new(
            privacy_blockchain::PrivacyBlockchain::new(storage.clone())
        ));

        let ephemeral_storage = Arc::new(RwLock::new(
            ephemeral_storage::EphemeralStorage::new()
        ));

        let peer_manager = Arc::new(RwLock::new(
            peer_manager::PeerManager::new(&config.data_dir).await?
        ));

        let autonomous_tasks = Arc::new(RwLock::new(
            autonomous_tasks::AutonomousTaskGenerator::new(
                storage.clone(),
                peer_manager.clone(),
                network_sync.clone()
            )
        ));

        Ok(ParadigmNode {
            config,
            network,
            consensus,
            storage,
            governance,
            tokenomics,
            network_sync,
            privacy_blockchain,
            ephemeral_storage,
            peer_manager,
            autonomous_tasks,
            keypair,
        })
    }

    pub async fn start(&mut self) -> anyhow::Result<()> {
        tracing::info!("Starting Paradigm node with ID: {}", self.config.node_id);

        // Start tokenomics system
        {
            let mut tokenomics = self.tokenomics.write().await;
            tokenomics.start().await?;
        }

        // Start network layer
        {
            let mut network = self.network.write().await;
            network.start_listening().await?;
        }

        // Start network synchronization
        {
            let mut network_sync = self.network_sync.write().await;
            network_sync.start_sync().await?;
        }

        // Start privacy blockchain auto-cleanup
        {
            let privacy_blockchain = self.privacy_blockchain.read().await;
            privacy_blockchain.start_auto_cleanup().await?;
        }

        // Start ephemeral storage auto-cleanup
        {
            let ephemeral_storage = self.ephemeral_storage.read().await;
            ephemeral_storage.start_auto_cleanup().await?;
        }

        // Start peer manager background tasks
        {
            let peer_manager = self.peer_manager.read().await;
            peer_manager.start_background_tasks().await?;
        }

        // Start autonomous task generation
        {
            let autonomous_tasks = self.autonomous_tasks.read().await;
            autonomous_tasks.start().await?;
        }

        tracing::info!("Paradigm node started successfully with autonomous features");
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

    pub async fn submit_transaction(
        &self,
        transaction: transaction::Transaction,
    ) -> anyhow::Result<()> {
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

    pub async fn get_transaction_history(
        &self,
        address: &Address,
    ) -> anyhow::Result<Vec<transaction::Transaction>> {
        let storage = self.storage.read().await;
        storage.get_transactions_for_address(address).await
    }

    pub async fn get_sync_info(&self) -> network_sync::SyncInfo {
        let network_sync = self.network_sync.read().await;
        network_sync.get_sync_info()
    }

    pub async fn get_sync_percentage(&self) -> u8 {
        let network_sync = self.network_sync.read().await;
        network_sync.get_sync_percentage()
    }

    pub async fn get_privacy_stats(&self) -> anyhow::Result<privacy_blockchain::PrivacyStats> {
        let privacy_blockchain = self.privacy_blockchain.read().await;
        privacy_blockchain.get_privacy_stats().await
    }

    pub async fn store_private_transaction(&self, transaction: &transaction::Transaction) -> anyhow::Result<()> {
        let privacy_blockchain = self.privacy_blockchain.read().await;
        privacy_blockchain.store_private_transaction(transaction).await
    }

    pub async fn get_private_balance(&self, address: &Address) -> anyhow::Result<u64> {
        let privacy_blockchain = self.privacy_blockchain.read().await;
        privacy_blockchain.get_private_balance(address).await
    }

    pub async fn get_ephemeral_balance(&self, address: &Address) -> anyhow::Result<u64> {
        let ephemeral_storage = self.ephemeral_storage.read().await;
        ephemeral_storage.get_balance(address).await
    }

    pub async fn get_ephemeral_transactions(&self, address: &Address, limit: Option<usize>) -> anyhow::Result<Vec<ephemeral_storage::EphemeralTransaction>> {
        let ephemeral_storage = self.ephemeral_storage.read().await;
        ephemeral_storage.get_address_transactions(address, limit).await
    }

    pub async fn store_ephemeral_transaction(&self, transaction: &transaction::Transaction) -> anyhow::Result<()> {
        let ephemeral_storage = self.ephemeral_storage.read().await;
        ephemeral_storage.store_transaction(transaction).await
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
        let address_str = address.to_string();
        assert!(address_str.starts_with("PAR"));
        assert_eq!(address_str.len(), 43); // "PAR" + 40 hex chars
    }

    #[test]
    fn test_constants() {
        assert_eq!(TOTAL_SUPPLY, 8_000_000_000_00000000);
        assert_eq!(DECIMALS, 8);
        assert_eq!(DEFAULT_PORT, 8080);
    }
}
