use crate::storage::ParadigmStorage;
use crate::tokenomics::treasury_manager::TreasuryManager;
use crate::{Address, Amount};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Special network address that holds the initial supply for decentralized distribution
pub const NETWORK_TREASURY_ADDRESS: &str = "PAR0000000000000000000000000000000000000000";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenesisConfig {
    /// Total initial supply to be held by network
    pub initial_supply: Amount,
    /// Genesis block timestamp
    pub genesis_time: DateTime<Utc>,
    /// Initial AI governance parameters
    pub ai_governance_params: AIGovernanceParams,
    /// Network configuration for P2P
    pub network_config: NetworkGenesisConfig,
    /// Enable features from genesis
    pub features: GenesisFeatures,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AIGovernanceParams {
    /// Initial distribution rate (tokens per block)
    pub initial_distribution_rate: Amount,
    /// AI learning rate for dynamic adjustments
    pub ai_learning_rate: f64,
    /// Minimum fee percentage
    pub min_fee_percentage: f64,
    /// Maximum fee percentage
    pub max_fee_percentage: f64,
    /// Fee adjustment sensitivity
    pub fee_sensitivity: f64,
    /// Allow AI to exceed max supply if needed
    pub allow_supply_expansion: bool,
    /// Supply expansion threshold (percentage of max supply)
    pub expansion_threshold: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkGenesisConfig {
    /// Genesis node peer ID
    pub genesis_peer_id: String,
    /// Genesis node listening port
    pub genesis_port: u16,
    /// Genesis node IP address
    pub genesis_ip: String,
    /// Chain ID for network identification
    pub chain_id: String,
    /// Network protocol version
    pub protocol_version: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenesisFeatures {
    /// Enable AI-controlled token distribution
    pub ai_distribution: bool,
    /// Enable quantum-resistant cryptography
    pub quantum_resistance: bool,
    /// Enable ML task rewards
    pub ml_task_rewards: bool,
    /// Enable cross-chain functionality
    pub cross_chain: bool,
    /// Enable governance proposals
    pub governance: bool,
    /// Enable privacy features
    pub privacy: bool,
}

impl Default for GenesisConfig {
    fn default() -> Self {
        Self {
            initial_supply: 21_000_000_00000000, // 21M PAR with 8 decimals
            genesis_time: Utc::now(),
            ai_governance_params: AIGovernanceParams::default(),
            network_config: NetworkGenesisConfig::default(),
            features: GenesisFeatures::default(),
        }
    }
}

impl Default for AIGovernanceParams {
    fn default() -> Self {
        Self {
            initial_distribution_rate: 1000_00000000, // 1000 PAR per block initially
            ai_learning_rate: 0.001,
            min_fee_percentage: 0.001, // 0.1%
            max_fee_percentage: 0.05,  // 5%
            fee_sensitivity: 0.1,
            allow_supply_expansion: true,
            expansion_threshold: 0.95, // When 95% of supply is distributed
        }
    }
}

impl Default for NetworkGenesisConfig {
    fn default() -> Self {
        Self {
            genesis_peer_id: String::new(), // Will be generated
            genesis_port: 8080,
            genesis_ip: "0.0.0.0".to_string(), // Bind to all interfaces
            chain_id: format!("paradigm-genesis-{}", Utc::now().timestamp()),
            protocol_version: "paradigm/1.0.0".to_string(),
        }
    }
}

impl Default for GenesisFeatures {
    fn default() -> Self {
        Self {
            ai_distribution: true,
            quantum_resistance: true,
            ml_task_rewards: true,
            cross_chain: false, // Can be enabled later
            governance: true,
            privacy: true,
        }
    }
}

/// Genesis block structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenesisBlock {
    /// Block number (always 0)
    pub block_number: u64,
    /// Previous block hash (all zeros for genesis)
    pub previous_hash: [u8; 32],
    /// Genesis block hash
    pub hash: [u8; 32],
    /// Genesis timestamp
    pub timestamp: DateTime<Utc>,
    /// Genesis transactions (initial supply allocation)
    pub transactions: Vec<GenesisTransaction>,
    /// Genesis configuration
    pub config: GenesisConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenesisTransaction {
    /// Transaction type
    pub tx_type: GenesisTransactionType,
    /// Recipient address (network treasury for initial supply)
    pub to: Address,
    /// Amount
    pub amount: Amount,
    /// Transaction timestamp
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum GenesisTransactionType {
    /// Initial supply allocation to network treasury
    InitialSupplyAllocation,
    /// Network treasury setup
    TreasurySetup,
    /// AI governance initialization
    AIGovernanceSetup,
}

pub struct GenesisManager {
    storage: ParadigmStorage,
    treasury_manager: TreasuryManager,
}

impl GenesisManager {
    pub fn new(storage: ParadigmStorage) -> Self {
        Self {
            storage,
            treasury_manager: TreasuryManager::new(),
        }
    }

    /// Create a new genesis block with network-held initial supply
    pub async fn create_genesis_block(
        &mut self,
        config: GenesisConfig,
    ) -> anyhow::Result<GenesisBlock> {
        tracing::info!("Creating genesis block for new Paradigm chain");

        // Create network treasury address (special address that network controls)
        let network_treasury = self.create_network_treasury_address();

        let genesis_time = config.genesis_time;
        let initial_supply = config.initial_supply;

        // Create genesis transactions
        let mut transactions = Vec::new();

        // 1. Allocate initial supply to network treasury
        transactions.push(GenesisTransaction {
            tx_type: GenesisTransactionType::InitialSupplyAllocation,
            to: network_treasury.clone(),
            amount: initial_supply,
            timestamp: genesis_time,
        });

        // 2. Setup treasury management
        transactions.push(GenesisTransaction {
            tx_type: GenesisTransactionType::TreasurySetup,
            to: network_treasury.clone(),
            amount: 0, // No additional tokens, just setup
            timestamp: genesis_time,
        });

        // 3. Initialize AI governance
        transactions.push(GenesisTransaction {
            tx_type: GenesisTransactionType::AIGovernanceSetup,
            to: network_treasury.clone(),
            amount: 0,
            timestamp: genesis_time,
        });

        // Calculate genesis block hash
        let genesis_hash = self.calculate_genesis_hash(&transactions, &config)?;

        let genesis_block = GenesisBlock {
            block_number: 0,
            previous_hash: [0u8; 32], // Genesis has no previous block
            hash: genesis_hash,
            timestamp: genesis_time,
            transactions,
            config,
        };

        // Initialize treasury with the initial supply
        self.treasury_manager
            .initialize_with_supply(initial_supply)
            .await?;

        tracing::info!(
            "Genesis block created with {} PAR allocated to network treasury",
            initial_supply as f64 / 100_000_000.0
        );

        Ok(genesis_block)
    }

    /// Initialize a new blockchain from genesis block
    pub async fn initialize_chain_from_genesis(
        &mut self,
        genesis_block: &GenesisBlock,
    ) -> anyhow::Result<()> {
        tracing::info!("Initializing new blockchain from genesis block");

        // Store genesis block
        self.storage.store_genesis_block(genesis_block).await?;

        // Initialize network treasury with initial supply
        let network_treasury = self.create_network_treasury_address();
        self.storage
            .set_balance(&network_treasury, genesis_block.config.initial_supply)
            .await?;

        // Initialize AI governance parameters
        self.storage
            .store_ai_governance_params(&genesis_block.config.ai_governance_params)
            .await?;

        // Initialize network configuration
        self.storage
            .store_network_genesis_config(&genesis_block.config.network_config)
            .await?;

        // Enable genesis features
        self.storage
            .store_genesis_features(&genesis_block.config.features)
            .await?;

        tracing::info!(
            "Chain initialized successfully. Network treasury holds {} PAR",
            genesis_block.config.initial_supply as f64 / 100_000_000.0
        );

        Ok(())
    }

    /// Create the special network treasury address
    fn create_network_treasury_address(&self) -> Address {
        // Use a deterministic address for the network treasury
        // This ensures the same address across all nodes
        let treasury_bytes = [0u8; 32]; // All zeros for network treasury
        Address(treasury_bytes)
    }

    /// AI-controlled token distribution from network treasury
    pub async fn ai_distribute_tokens(
        &mut self,
        current_block_height: u64,
        network_demand: f64,
        contribution_quality: f64,
    ) -> anyhow::Result<Amount> {
        let network_treasury = self.create_network_treasury_address();
        let current_balance = self.storage.get_balance(&network_treasury).await?;

        if current_balance == 0 {
            return Ok(0);
        }

        // AI calculates optimal distribution amount
        let base_rate = 1000_00000000u64; // 1000 PAR base

        // Dynamic adjustment based on network conditions
        let demand_multiplier = 1.0 + (network_demand - 0.5) * 0.5; // ±25% based on demand
        let quality_multiplier = 0.5 + contribution_quality; // 0.5x to 1.5x based on quality

        let distribution_amount =
            (base_rate as f64 * demand_multiplier * quality_multiplier) as Amount;

        // Ensure we don't exceed available treasury
        let actual_distribution = distribution_amount.min(current_balance);

        tracing::info!(
            "AI distributing {} PAR from network treasury (block {})",
            actual_distribution as f64 / 100_000_000.0,
            current_block_height
        );

        Ok(actual_distribution)
    }

    /// AI-controlled fee calculation
    pub async fn ai_calculate_fees(
        &self,
        transaction_volume: Amount,
        network_congestion: f64,
    ) -> anyhow::Result<f64> {
        // Load AI governance parameters
        let params = self.storage.get_ai_governance_params().await?;

        // Base fee calculation
        let base_fee = params.min_fee_percentage;

        // Congestion adjustment
        let congestion_adjustment = network_congestion * params.fee_sensitivity;

        // Final fee percentage
        let final_fee = (base_fee + congestion_adjustment)
            .max(params.min_fee_percentage)
            .min(params.max_fee_percentage);

        tracing::debug!(
            "AI calculated fee: {:.4}% (congestion: {:.2}, volume: {} PAR)",
            final_fee * 100.0,
            network_congestion,
            transaction_volume as f64 / 100_000_000.0
        );

        Ok(final_fee)
    }

    /// Check if AI should expand supply beyond max cap
    pub async fn ai_should_expand_supply(
        &self,
        current_circulating: Amount,
        max_supply: Amount,
        economic_indicators: &EconomicIndicators,
    ) -> anyhow::Result<bool> {
        let params = self.storage.get_ai_governance_params().await?;

        if !params.allow_supply_expansion {
            return Ok(false);
        }

        let supply_ratio = current_circulating as f64 / max_supply as f64;

        // Only consider expansion if we're near the threshold
        if supply_ratio < params.expansion_threshold {
            return Ok(false);
        }

        // AI decision based on economic indicators
        let expansion_score = self.calculate_expansion_score(economic_indicators);

        // Expand if score is high enough (threshold of 0.8)
        let should_expand = expansion_score > 0.8;

        if should_expand {
            tracing::warn!(
                "AI recommends supply expansion: score={:.2}, ratio={:.2}",
                expansion_score,
                supply_ratio
            );
        }

        Ok(should_expand)
    }

    /// Calculate AI expansion score based on economic indicators
    fn calculate_expansion_score(&self, indicators: &EconomicIndicators) -> f64 {
        // Weighted scoring algorithm
        let demand_weight = 0.3;
        let growth_weight = 0.3;
        let utility_weight = 0.2;
        let adoption_weight = 0.2;

        let demand_score = (indicators.demand_pressure / 100.0).min(1.0);
        let growth_score = (indicators.network_growth / 50.0).min(1.0);
        let utility_score = (indicators.utility_usage / 80.0).min(1.0);
        let adoption_score = (indicators.adoption_rate / 100.0).min(1.0);

        (demand_score * demand_weight)
            + (growth_score * growth_weight)
            + (utility_score * utility_weight)
            + (adoption_score * adoption_weight)
    }

    /// Calculate genesis block hash
    fn calculate_genesis_hash(
        &self,
        transactions: &[GenesisTransaction],
        config: &GenesisConfig,
    ) -> anyhow::Result<[u8; 32]> {
        use blake3::Hasher;

        let mut hasher = Hasher::new();

        // Hash transactions
        for tx in transactions {
            let tx_data = serde_json::to_vec(tx)?;
            hasher.update(&tx_data);
        }

        // Hash config
        let config_data = serde_json::to_vec(config)?;
        hasher.update(&config_data);

        let hash_result = hasher.finalize();
        let mut hash = [0u8; 32];
        hash.copy_from_slice(hash_result.as_bytes());

        Ok(hash)
    }
}

/// Economic indicators for AI decision making
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EconomicIndicators {
    /// Demand pressure (0-100)
    pub demand_pressure: f64,
    /// Network growth rate (percentage)
    pub network_growth: f64,
    /// Utility usage rate (0-100)
    pub utility_usage: f64,
    /// Adoption rate (0-100)
    pub adoption_rate: f64,
}

// Extensions for TreasuryManager to support genesis
impl TreasuryManager {
    /// Initialize treasury with network-held initial supply
    pub async fn initialize_with_supply(&mut self, initial_supply: Amount) -> anyhow::Result<()> {
        self.initialize().await?;

        // Override balance with initial supply from genesis
        self.set_balance(initial_supply);

        tracing::info!(
            "Treasury initialized with network-held supply: {} PAR",
            initial_supply as f64 / 100_000_000.0
        );

        Ok(())
    }
}
