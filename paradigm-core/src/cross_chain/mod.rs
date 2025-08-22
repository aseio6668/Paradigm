/// Cross-chain interoperability framework for Paradigm
pub mod bridge_protocol;
pub mod lightning_integration;
pub mod ibc_support;
pub mod atomic_swaps;
pub mod asset_management;
pub mod governance;
pub mod message_passing;

use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use anyhow::Result;
use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;
use uuid::Uuid;

use crate::{Address, Hash, Amount};

/// Supported blockchain networks for cross-chain operations
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ChainId {
    Paradigm,
    Ethereum,
    Bitcoin,
    Cosmos,
    Polkadot,
    Binance,
    Polygon,
    Avalanche,
    Solana,
    Cardano,
}

impl ChainId {
    pub fn chain_name(&self) -> &'static str {
        match self {
            ChainId::Paradigm => "Paradigm",
            ChainId::Ethereum => "Ethereum",
            ChainId::Bitcoin => "Bitcoin",
            ChainId::Cosmos => "Cosmos Hub",
            ChainId::Polkadot => "Polkadot",
            ChainId::Binance => "Binance Smart Chain",
            ChainId::Polygon => "Polygon",
            ChainId::Avalanche => "Avalanche",
            ChainId::Solana => "Solana",
            ChainId::Cardano => "Cardano",
        }
    }

    pub fn native_token(&self) -> &'static str {
        match self {
            ChainId::Paradigm => "PAR",
            ChainId::Ethereum => "ETH",
            ChainId::Bitcoin => "BTC",
            ChainId::Cosmos => "ATOM",
            ChainId::Polkadot => "DOT",
            ChainId::Binance => "BNB",
            ChainId::Polygon => "MATIC",
            ChainId::Avalanche => "AVAX",
            ChainId::Solana => "SOL",
            ChainId::Cardano => "ADA",
        }
    }

    pub fn block_time_seconds(&self) -> u64 {
        match self {
            ChainId::Paradigm => 12,
            ChainId::Ethereum => 12,
            ChainId::Bitcoin => 600,
            ChainId::Cosmos => 6,
            ChainId::Polkadot => 6,
            ChainId::Binance => 3,
            ChainId::Polygon => 2,
            ChainId::Avalanche => 2,
            ChainId::Solana => 1,
            ChainId::Cardano => 20,
        }
    }
}

/// Cross-chain asset representation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CrossChainAsset {
    pub asset_id: Uuid,
    pub origin_chain: ChainId,
    pub origin_address: String,
    pub symbol: String,
    pub name: String,
    pub decimals: u8,
    pub total_supply: Option<u128>,
    pub is_native: bool,
    pub supported_chains: Vec<ChainId>,
}

/// Cross-chain transaction types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CrossChainOperation {
    AssetTransfer {
        from_chain: ChainId,
        to_chain: ChainId,
        asset: CrossChainAsset,
        amount: u128,
        recipient: String,
        fee: u128,
    },
    AtomicSwap {
        chain_a: ChainId,
        chain_b: ChainId,
        asset_a: CrossChainAsset,
        asset_b: CrossChainAsset,
        amount_a: u128,
        amount_b: u128,
        timeout_height: u64,
        secret_hash: Hash,
    },
    MessagePassing {
        from_chain: ChainId,
        to_chain: ChainId,
        message: Vec<u8>,
        callback_address: Option<String>,
    },
    GovernanceProposal {
        proposal_id: Uuid,
        origin_chain: ChainId,
        target_chains: Vec<ChainId>,
        proposal_data: Vec<u8>,
        voting_period: Duration,
    },
}

/// Cross-chain transaction status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CrossChainStatus {
    Initiated,
    Pending,
    Confirmed,
    Executed,
    Failed,
    Cancelled,
    TimedOut,
}

/// Cross-chain transaction record
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CrossChainTransaction {
    pub id: Uuid,
    pub operation: CrossChainOperation,
    pub status: CrossChainStatus,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub updated_at: chrono::DateTime<chrono::Utc>,
    pub source_tx_hash: Option<String>,
    pub destination_tx_hash: Option<String>,
    pub confirmations: HashMap<ChainId, u64>,
    pub required_confirmations: HashMap<ChainId, u64>,
    pub error_message: Option<String>,
    pub fee_paid: HashMap<ChainId, u128>,
}

/// Chain connection information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChainConnection {
    pub chain_id: ChainId,
    pub rpc_endpoint: String,
    pub websocket_endpoint: Option<String>,
    pub bridge_contract: Option<String>,
    pub is_connected: bool,
    pub last_block_height: u64,
    pub last_heartbeat: chrono::DateTime<chrono::Utc>,
    pub connection_quality: f64, // 0.0 to 1.0
}

/// Cross-chain interoperability manager
#[derive(Debug)]
pub struct CrossChainManager {
    connections: Arc<RwLock<HashMap<ChainId, ChainConnection>>>,
    pending_transactions: Arc<RwLock<HashMap<Uuid, CrossChainTransaction>>>,
    asset_registry: Arc<RwLock<HashMap<Uuid, CrossChainAsset>>>,
    config: CrossChainConfig,
    bridge_protocol: Arc<bridge_protocol::BridgeProtocol>,
    lightning_integration: Arc<lightning_integration::LightningIntegration>,
    ibc_support: Arc<ibc_support::IBCModule>,
    atomic_swaps: Arc<atomic_swaps::AtomicSwapEngine>,
    message_passing: Arc<message_passing::MessagePassingProtocol>,
    stats: Arc<RwLock<CrossChainStats>>,
}

#[derive(Debug, Clone)]
pub struct CrossChainConfig {
    pub enabled_chains: Vec<ChainId>,
    pub confirmation_requirements: HashMap<ChainId, u64>,
    pub bridge_fee_rates: HashMap<ChainId, f64>,
    pub max_transaction_size: u128,
    pub transaction_timeout: Duration,
    pub heartbeat_interval: Duration,
    pub enable_atomic_swaps: bool,
    pub enable_message_passing: bool,
    pub enable_governance_bridging: bool,
}

impl Default for CrossChainConfig {
    fn default() -> Self {
        let mut confirmation_requirements = HashMap::new();
        confirmation_requirements.insert(ChainId::Paradigm, 6);
        confirmation_requirements.insert(ChainId::Ethereum, 12);
        confirmation_requirements.insert(ChainId::Bitcoin, 6);
        confirmation_requirements.insert(ChainId::Cosmos, 1);

        let mut bridge_fee_rates = HashMap::new();
        bridge_fee_rates.insert(ChainId::Ethereum, 0.001); // 0.1%
        bridge_fee_rates.insert(ChainId::Bitcoin, 0.002);  // 0.2%
        bridge_fee_rates.insert(ChainId::Cosmos, 0.0005);  // 0.05%

        Self {
            enabled_chains: vec![
                ChainId::Paradigm,
                ChainId::Ethereum,
                ChainId::Bitcoin,
                ChainId::Cosmos,
            ],
            confirmation_requirements,
            bridge_fee_rates,
            max_transaction_size: 1_000_000_000_000_000_000, // 1B tokens max
            transaction_timeout: Duration::from_hours(24),
            heartbeat_interval: Duration::from_secs(30),
            enable_atomic_swaps: true,
            enable_message_passing: true,
            enable_governance_bridging: true,
        }
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CrossChainStats {
    pub total_transactions: u64,
    pub successful_transactions: u64,
    pub failed_transactions: u64,
    pub total_volume_usd: f64,
    pub average_confirmation_time: Duration,
    pub active_connections: usize,
    pub supported_assets: usize,
    pub pending_operations: usize,
}

impl CrossChainManager {
    pub async fn new(config: CrossChainConfig) -> Result<Self> {
        let bridge_protocol = Arc::new(bridge_protocol::BridgeProtocol::new(&config).await?);
        let lightning_integration = Arc::new(lightning_integration::LightningIntegration::new(&config).await?);
        let ibc_support = Arc::new(ibc_support::IBCModule::new());
        let atomic_swaps = Arc::new(atomic_swaps::AtomicSwapEngine::new(&config).await?);
        let message_passing = Arc::new(message_passing::MessagePassingProtocol::new(&config).await?);

        Ok(Self {
            connections: Arc::new(RwLock::new(HashMap::new())),
            pending_transactions: Arc::new(RwLock::new(HashMap::new())),
            asset_registry: Arc::new(RwLock::new(HashMap::new())),
            config,
            bridge_protocol,
            lightning_integration,
            ibc_support,
            atomic_swaps,
            message_passing,
            stats: Arc::new(RwLock::new(CrossChainStats::default())),
        })
    }

    /// Initialize cross-chain connections
    pub async fn initialize(&self) -> Result<()> {
        tracing::info!("Initializing cross-chain interoperability...");

        // Initialize bridge protocol
        self.bridge_protocol.initialize().await?;

        // Initialize Lightning Network integration
        self.lightning_integration.initialize().await?;

        // Initialize IBC support - no initialize method needed for basic IBCModule

        // Initialize atomic swap engine
        self.atomic_swaps.initialize().await?;

        // Initialize message passing protocol
        self.message_passing.initialize().await?;

        // Start monitoring and heartbeat tasks
        self.start_monitoring_tasks().await?;

        // Register default assets
        self.register_default_assets().await?;

        tracing::info!("Cross-chain interoperability initialized successfully");
        Ok(())
    }

    /// Add a chain connection
    pub async fn add_chain_connection(&self, connection: ChainConnection) -> Result<()> {
        let chain_id = connection.chain_id;
        
        // Validate connection
        if !self.config.enabled_chains.contains(&chain_id) {
            return Err(anyhow::anyhow!("Chain {:?} is not enabled in configuration", chain_id));
        }

        // Test connection
        let is_healthy = self.test_chain_connection(&connection).await?;
        let mut updated_connection = connection;
        updated_connection.is_connected = is_healthy;
        updated_connection.last_heartbeat = chrono::Utc::now();

        // Store connection
        {
            let mut connections = self.connections.write().await;
            connections.insert(chain_id, updated_connection);
        }

        tracing::info!("Added connection for chain: {}", chain_id.chain_name());
        Ok(())
    }

    /// Execute cross-chain operation
    pub async fn execute_cross_chain_operation(&self, operation: CrossChainOperation) -> Result<Uuid> {
        let transaction_id = Uuid::new_v4();
        
        let cross_chain_tx = CrossChainTransaction {
            id: transaction_id,
            operation: operation.clone(),
            status: CrossChainStatus::Initiated,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
            source_tx_hash: None,
            destination_tx_hash: None,
            confirmations: HashMap::new(),
            required_confirmations: HashMap::new(),
            error_message: None,
            fee_paid: HashMap::new(),
        };

        // Store pending transaction
        {
            let mut pending = self.pending_transactions.write().await;
            pending.insert(transaction_id, cross_chain_tx);
        }

        // Route to appropriate handler
        match operation {
            CrossChainOperation::AssetTransfer { from_chain, to_chain, .. } => {
                self.handle_asset_transfer(transaction_id, from_chain, to_chain).await?;
            },
            CrossChainOperation::AtomicSwap { .. } => {
                self.handle_atomic_swap(transaction_id).await?;
            },
            CrossChainOperation::MessagePassing { from_chain, to_chain, .. } => {
                self.handle_message_passing(transaction_id, from_chain, to_chain).await?;
            },
            CrossChainOperation::GovernanceProposal { .. } => {
                self.handle_governance_proposal(transaction_id).await?;
            },
        }

        // Update stats
        {
            let mut stats = self.stats.write().await;
            stats.total_transactions += 1;
            stats.pending_operations += 1;
        }

        Ok(transaction_id)
    }

    /// Get transaction status
    pub async fn get_transaction_status(&self, transaction_id: &Uuid) -> Option<CrossChainTransaction> {
        let pending = self.pending_transactions.read().await;
        pending.get(transaction_id).cloned()
    }

    /// Register a cross-chain asset
    pub async fn register_asset(&self, asset: CrossChainAsset) -> Result<()> {
        let mut registry = self.asset_registry.write().await;
        registry.insert(asset.asset_id, asset.clone());
        
        tracing::info!("Registered cross-chain asset: {} ({})", asset.name, asset.symbol);
        Ok(())
    }

    /// Get supported assets
    pub async fn get_supported_assets(&self) -> Vec<CrossChainAsset> {
        let registry = self.asset_registry.read().await;
        registry.values().cloned().collect()
    }

    /// Get cross-chain statistics
    pub async fn get_stats(&self) -> CrossChainStats {
        let mut stats = self.stats.read().await.clone();
        
        // Update real-time stats
        stats.active_connections = self.connections.read().await.len();
        stats.supported_assets = self.asset_registry.read().await.len();
        stats.pending_operations = self.pending_transactions.read().await.len();
        
        stats
    }

    // Private methods

    async fn start_monitoring_tasks(&self) -> Result<()> {
        // Heartbeat monitoring
        let connections = self.connections.clone();
        let heartbeat_interval = self.config.heartbeat_interval;
        
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(heartbeat_interval);
            loop {
                interval.tick().await;
                
                let connection_map = connections.read().await.clone();
                for (chain_id, connection) in connection_map {
                    // Test connection health
                    // In a real implementation, this would ping the chain
                    tracing::debug!("Heartbeat check for {}", chain_id.chain_name());
                }
            }
        });

        // Transaction monitoring
        let pending_transactions = self.pending_transactions.clone();
        let transaction_timeout = self.config.transaction_timeout;
        
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(Duration::from_secs(60));
            loop {
                interval.tick().await;
                
                let mut to_timeout = Vec::new();
                {
                    let pending = pending_transactions.read().await;
                    let now = chrono::Utc::now();
                    
                    for (id, tx) in pending.iter() {
                        if now.signed_duration_since(tx.created_at).to_std().unwrap_or_default() > transaction_timeout {
                            to_timeout.push(*id);
                        }
                    }
                }
                
                // Timeout expired transactions
                for tx_id in to_timeout {
                    let mut pending = pending_transactions.write().await;
                    if let Some(mut tx) = pending.get_mut(&tx_id) {
                        tx.status = CrossChainStatus::TimedOut;
                        tx.updated_at = chrono::Utc::now();
                        tracing::warn!("Cross-chain transaction {} timed out", tx_id);
                    }
                }
            }
        });

        Ok(())
    }

    async fn register_default_assets(&self) -> Result<()> {
        // Register PAR token
        let par_asset = CrossChainAsset {
            asset_id: Uuid::new_v4(),
            origin_chain: ChainId::Paradigm,
            origin_address: "native".to_string(),
            symbol: "PAR".to_string(),
            name: "Paradigm".to_string(),
            decimals: 8,
            total_supply: Some(8_000_000_000_00000000),
            is_native: true,
            supported_chains: vec![ChainId::Paradigm, ChainId::Ethereum, ChainId::Cosmos],
        };
        self.register_asset(par_asset).await?;

        // Register wrapped versions of major assets
        let weth_asset = CrossChainAsset {
            asset_id: Uuid::new_v4(),
            origin_chain: ChainId::Ethereum,
            origin_address: "0xC02aaA39b223FE8D0A0e5C4F27eAD9083C756Cc2".to_string(),
            symbol: "WETH".to_string(),
            name: "Wrapped Ethereum".to_string(),
            decimals: 18,
            total_supply: None,
            is_native: false,
            supported_chains: vec![ChainId::Ethereum, ChainId::Paradigm],
        };
        self.register_asset(weth_asset).await?;

        let wbtc_asset = CrossChainAsset {
            asset_id: Uuid::new_v4(),
            origin_chain: ChainId::Bitcoin,
            origin_address: "native".to_string(),
            symbol: "WBTC".to_string(),
            name: "Wrapped Bitcoin".to_string(),
            decimals: 8,
            total_supply: Some(21_000_000_00000000),
            is_native: false,
            supported_chains: vec![ChainId::Bitcoin, ChainId::Ethereum, ChainId::Paradigm],
        };
        self.register_asset(wbtc_asset).await?;

        Ok(())
    }

    async fn test_chain_connection(&self, connection: &ChainConnection) -> Result<bool> {
        // This would test the actual connection to the chain
        // For now, we'll simulate a successful connection test
        tokio::time::sleep(Duration::from_millis(100)).await;
        Ok(true)
    }

    async fn handle_asset_transfer(&self, transaction_id: Uuid, from_chain: ChainId, to_chain: ChainId) -> Result<()> {
        // Route to bridge protocol
        self.bridge_protocol.handle_asset_transfer(transaction_id, from_chain, to_chain).await
    }

    async fn handle_atomic_swap(&self, transaction_id: Uuid) -> Result<()> {
        // Route to atomic swap engine
        self.atomic_swaps.handle_swap(transaction_id).await
    }

    async fn handle_message_passing(&self, transaction_id: Uuid, from_chain: ChainId, to_chain: ChainId) -> Result<()> {
        // Route to message passing protocol
        self.message_passing.handle_message(transaction_id, from_chain, to_chain).await
    }

    async fn handle_governance_proposal(&self, transaction_id: Uuid) -> Result<()> {
        // Route to cross-chain governance
        governance::handle_cross_chain_proposal(transaction_id).await
    }
}

/// Utility functions for cross-chain operations
pub mod utils {
    use super::*;

    /// Calculate bridge fee for a transaction
    pub fn calculate_bridge_fee(
        amount: u128,
        from_chain: ChainId,
        to_chain: ChainId,
        config: &CrossChainConfig,
    ) -> u128 {
        let base_fee_rate = config.bridge_fee_rates.get(&to_chain).unwrap_or(&0.001);
        let fee = (amount as f64 * base_fee_rate) as u128;
        
        // Minimum fee based on chain characteristics
        let min_fee = match to_chain {
            ChainId::Ethereum => 50_000, // ~$0.50 equivalent
            ChainId::Bitcoin => 10_000,  // ~$0.10 equivalent
            ChainId::Cosmos => 1_000,    // ~$0.01 equivalent
            _ => 5_000,                  // Default minimum
        };
        
        fee.max(min_fee)
    }

    /// Validate cross-chain address format
    pub fn validate_cross_chain_address(address: &str, chain_id: ChainId) -> bool {
        match chain_id {
            ChainId::Paradigm => address.starts_with("PAR") && address.len() == 43,
            ChainId::Ethereum => address.starts_with("0x") && address.len() == 42,
            ChainId::Bitcoin => address.len() >= 26 && address.len() <= 35,
            ChainId::Cosmos => address.starts_with("cosmos") && address.len() == 45,
            _ => true, // Basic validation for other chains
        }
    }

    /// Convert amount between different chain decimals
    pub fn convert_amount_decimals(amount: u128, from_decimals: u8, to_decimals: u8) -> u128 {
        if from_decimals == to_decimals {
            return amount;
        }
        
        if from_decimals > to_decimals {
            let divisor = 10_u128.pow((from_decimals - to_decimals) as u32);
            amount / divisor
        } else {
            let multiplier = 10_u128.pow((to_decimals - from_decimals) as u32);
            amount * multiplier
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_cross_chain_manager_creation() {
        let config = CrossChainConfig::default();
        let manager = CrossChainManager::new(config).await;
        assert!(manager.is_ok());
    }

    #[test]
    fn test_utils_calculate_bridge_fee() {
        let config = CrossChainConfig::default();
        let fee = utils::calculate_bridge_fee(
            1_000_000,
            ChainId::Paradigm,
            ChainId::Ethereum,
            &config,
        );
        assert!(fee > 0);
    }

    #[test]
    fn test_utils_validate_address() {
        assert!(utils::validate_cross_chain_address("PAR1234567890abcdef1234567890abcdef12345678", ChainId::Paradigm));
        assert!(utils::validate_cross_chain_address("0x742d35Cc6635C0532925a3b8D8434d8975c64d27", ChainId::Ethereum));
        assert!(!utils::validate_cross_chain_address("invalid", ChainId::Bitcoin));
    }

    #[test]
    fn test_utils_convert_decimals() {
        // 18 decimals to 8 decimals
        let result = utils::convert_amount_decimals(1_000_000_000_000_000_000, 18, 8);
        assert_eq!(result, 100_000_000);
        
        // 8 decimals to 18 decimals
        let result = utils::convert_amount_decimals(100_000_000, 8, 18);
        assert_eq!(result, 1_000_000_000_000_000_000);
    }
}