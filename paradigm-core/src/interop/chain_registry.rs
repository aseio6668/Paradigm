// Chain Registry and Management
// Comprehensive blockchain network registry and management system

use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;
use anyhow::Result;
use uuid::Uuid;
use serde::{Serialize, Deserialize};

use super::{SupportedChain, SecurityLevel};

#[derive(Debug, Clone)]
pub struct ChainRegistry {
    registered_chains: Arc<RwLock<HashMap<SupportedChain, ChainInfo>>>,
    network_monitors: Arc<RwLock<HashMap<SupportedChain, NetworkMonitor>>>,
    compatibility_matrix: Arc<RwLock<CompatibilityMatrix>>,
    registry_config: ChainRegistryConfig,
}

#[derive(Debug, Clone)]
pub struct ChainInfo {
    pub chain_id: SupportedChain,
    pub chain_name: String,
    pub network_type: NetworkType,
    pub consensus_mechanism: ConsensusMechanism,
    pub native_token: TokenInfo,
    pub rpc_endpoints: Vec<RPCEndpoint>,
    pub block_explorers: Vec<String>,
    pub smart_contract_support: SmartContractSupport,
    pub interop_capabilities: InteropCapabilities,
    pub security_features: Vec<SecurityFeature>,
    pub performance_metrics: PerformanceMetrics,
    pub governance_model: GovernanceModel,
    pub status: ChainStatus,
    pub registration_date: u64,
    pub last_updated: u64,
}

#[derive(Debug, Clone)]
pub enum NetworkType {
    Mainnet,
    Testnet,
    Devnet,
    Private,
    Consortium,
}

#[derive(Debug, Clone)]
pub enum ConsensusMechanism {
    ProofOfWork,
    ProofOfStake,
    DelegatedProofOfStake,
    ProofOfAuthority,
    ByzantineFaultTolerance,
    PracticalByzantineFaultTolerance,
    HoneyBadgerBFT,
    Tendermint,
    Ouroboros,
    Algorand,
    Avalanche,
    Custom(String),
}

#[derive(Debug, Clone)]
pub struct TokenInfo {
    pub symbol: String,
    pub name: String,
    pub decimals: u8,
    pub total_supply: Option<u64>,
    pub contract_address: Option<String>,
}

#[derive(Debug, Clone)]
pub struct RPCEndpoint {
    pub url: String,
    pub endpoint_type: EndpointType,
    pub authentication: Option<AuthenticationMethod>,
    pub rate_limit: Option<RateLimit>,
    pub reliability_score: f64,
    pub latency_ms: u64,
    pub active: bool,
}

#[derive(Debug, Clone)]
pub enum EndpointType {
    HTTP,
    HTTPS,
    WebSocket,
    WebSocketSecure,
    GRPC,
    GraphQL,
}

#[derive(Debug, Clone)]
pub enum AuthenticationMethod {
    ApiKey(String),
    BearerToken(String),
    BasicAuth { username: String, password: String },
    Custom(String),
}

#[derive(Debug, Clone)]
pub struct RateLimit {
    pub requests_per_second: u32,
    pub burst_limit: u32,
    pub window_size: Duration,
}

#[derive(Debug, Clone)]
pub struct SmartContractSupport {
    pub supported: bool,
    pub vm_type: Option<VirtualMachineType>,
    pub programming_languages: Vec<String>,
    pub gas_model: GasModel,
    pub max_contract_size: Option<u64>,
    pub deployment_cost: Option<u64>,
}

#[derive(Debug, Clone)]
pub enum VirtualMachineType {
    EVM, // Ethereum Virtual Machine
    WASM, // WebAssembly
    MoveVM, // Move Virtual Machine
    ActorModel,
    Custom(String),
}

#[derive(Debug, Clone)]
pub struct GasModel {
    pub has_gas_concept: bool,
    pub gas_unit_name: String,
    pub base_fee: Option<u64>,
    pub priority_fee_support: bool,
}

#[derive(Debug, Clone)]
pub struct InteropCapabilities {
    pub bridge_protocols: Vec<BridgeProtocolType>,
    pub atomic_swaps: bool,
    pub cross_chain_messaging: bool,
    pub ibc_support: bool, // Inter-Blockchain Communication
    pub xcmp_support: bool, // Cross-Consensus Message Passing
    pub relay_chain_support: bool,
    pub state_channels: bool,
    pub sidechains: bool,
}

#[derive(Debug, Clone)]
pub enum BridgeProtocolType {
    LockAndMint,
    BurnAndMint,
    Atomic,
    Optimistic,
    ZkProof,
    Multisig,
    Federation,
    Relay,
}

#[derive(Debug, Clone)]
pub enum SecurityFeature {
    MultiSignature,
    TimeLocks,
    ZeroKnowledgeProofs,
    RingSignatures,
    HomomorphicEncryption,
    QuantumResistance,
    FormalVerification,
    BugBountyProgram,
}

#[derive(Debug, Clone)]
pub struct PerformanceMetrics {
    pub avg_block_time: Duration,
    pub max_tps: u32,
    pub finality_time: Duration,
    pub network_latency: Duration,
    pub storage_efficiency: f64,
    pub energy_consumption: EnergyConsumption,
}

#[derive(Debug, Clone)]
pub struct EnergyConsumption {
    pub watts_per_transaction: Option<f64>,
    pub carbon_footprint_rating: CarbonFootprintRating,
    pub renewable_energy_percentage: Option<f64>,
}

#[derive(Debug, Clone)]
pub enum CarbonFootprintRating {
    VeryLow,
    Low,
    Medium,
    High,
    VeryHigh,
}

#[derive(Debug, Clone)]
pub enum GovernanceModel {
    OnChain,
    OffChain,
    Hybrid,
    Foundation,
    DAO, // Decentralized Autonomous Organization
    Council,
    Democracy,
    Autocracy,
}

#[derive(Debug, Clone)]
pub enum ChainStatus {
    Active,
    Inactive,
    Deprecated,
    Maintenance,
    Experimental,
}

#[derive(Debug, Clone)]
pub struct NetworkMonitor {
    pub chain_id: SupportedChain,
    pub health_metrics: Arc<RwLock<HealthMetrics>>,
    pub connection_pool: Arc<ConnectionPool>,
    pub monitoring_config: MonitoringConfig,
    pub alert_system: Arc<AlertSystem>,
}

#[derive(Debug, Clone)]
pub struct HealthMetrics {
    pub is_online: bool,
    pub block_height: u64,
    pub peer_count: u32,
    pub sync_status: SyncStatus,
    pub mempool_size: u32,
    pub network_hashrate: Option<u64>,
    pub validator_count: Option<u32>,
    pub last_block_time: u64,
    pub avg_response_time: Duration,
    pub error_rate: f64,
    pub uptime_percentage: f64,
}

#[derive(Debug, Clone)]
pub enum SyncStatus {
    Synced,
    Syncing { current: u64, target: u64 },
    NotSynced,
    Unknown,
}

#[derive(Debug, Clone)]
pub struct ConnectionPool {
    pub active_connections: u32,
    pub max_connections: u32,
    pub connection_timeout: Duration,
    pub retry_policy: RetryPolicy,
}

#[derive(Debug, Clone)]
pub struct RetryPolicy {
    pub max_retries: u32,
    pub base_delay: Duration,
    pub backoff_multiplier: f64,
    pub max_delay: Duration,
}

#[derive(Debug, Clone)]
pub struct MonitoringConfig {
    pub check_interval: Duration,
    pub health_check_timeout: Duration,
    pub metrics_retention_period: Duration,
    pub alert_thresholds: AlertThresholds,
}

#[derive(Debug, Clone)]
pub struct AlertThresholds {
    pub max_response_time: Duration,
    pub min_uptime_percentage: f64,
    pub max_error_rate: f64,
    pub max_sync_lag: u64,
}

#[derive(Debug, Clone)]
pub struct AlertSystem {
    pub alert_channels: Vec<AlertChannel>,
    pub alert_rules: Vec<AlertRule>,
    pub alert_history: Arc<RwLock<Vec<Alert>>>,
}

#[derive(Debug, Clone)]
pub enum AlertChannel {
    Email(String),
    Webhook(String),
    SMS(String),
    Slack(String),
    Discord(String),
}

#[derive(Debug, Clone)]
pub struct AlertRule {
    pub rule_id: Uuid,
    pub metric: String,
    pub condition: AlertCondition,
    pub threshold: f64,
    pub duration: Duration,
    pub severity: AlertSeverity,
    pub enabled: bool,
}

#[derive(Debug, Clone)]
pub enum AlertCondition {
    GreaterThan,
    LessThan,
    Equal,
    NotEqual,
    PercentageChange,
}

#[derive(Debug, Clone)]
pub enum AlertSeverity {
    Low,
    Medium,
    High,
    Critical,
}

#[derive(Debug, Clone)]
pub struct Alert {
    pub alert_id: Uuid,
    pub chain_id: SupportedChain,
    pub rule_id: Uuid,
    pub message: String,
    pub severity: AlertSeverity,
    pub timestamp: u64,
    pub resolved: bool,
    pub resolved_at: Option<u64>,
}

#[derive(Debug, Clone)]
pub struct CompatibilityMatrix {
    pub chain_pairs: HashMap<(SupportedChain, SupportedChain), CompatibilityInfo>,
    pub protocol_support: HashMap<SupportedChain, Vec<InteropProtocol>>,
}

#[derive(Debug, Clone)]
pub struct CompatibilityInfo {
    pub compatible: bool,
    pub supported_protocols: Vec<InteropProtocol>,
    pub limitations: Vec<String>,
    pub estimated_transfer_time: Duration,
    pub estimated_cost: u64,
    pub security_level: SecurityLevel,
}

#[derive(Debug, Clone)]
pub enum InteropProtocol {
    DirectBridge,
    AtomicSwap,
    IBC, // Inter-Blockchain Communication
    XCMP, // Cross-Consensus Message Passing
    StateChannel,
    Sidechain,
    RelayChain,
    LightClient,
}

#[derive(Debug, Clone)]
pub struct ChainRegistryConfig {
    pub auto_discovery_enabled: bool,
    pub health_check_interval: Duration,
    pub compatibility_update_interval: Duration,
    pub max_inactive_period: Duration,
    pub security_validation_enabled: bool,
}

impl Default for ChainRegistryConfig {
    fn default() -> Self {
        Self {
            auto_discovery_enabled: true,
            health_check_interval: Duration::from_secs(30),
            compatibility_update_interval: Duration::from_secs(3600), // 1 hour
            max_inactive_period: Duration::from_secs(86400), // 24 hours
            security_validation_enabled: true,
        }
    }
}

impl ChainRegistry {
    pub fn new(config: ChainRegistryConfig) -> Self {
        Self {
            registered_chains: Arc::new(RwLock::new(HashMap::new())),
            network_monitors: Arc::new(RwLock::new(HashMap::new())),
            compatibility_matrix: Arc::new(RwLock::new(CompatibilityMatrix {
                chain_pairs: HashMap::new(),
                protocol_support: HashMap::new(),
            })),
            registry_config: config,
        }
    }

    pub async fn initialize(&self) -> Result<()> {
        self.register_default_chains().await?;
        self.setup_network_monitors().await?;
        self.build_compatibility_matrix().await?;
        Ok(())
    }

    async fn register_default_chains(&self) -> Result<()> {
        let mut chains = self.registered_chains.write().await;

        // Register Ethereum
        chains.insert(SupportedChain::Ethereum, ChainInfo {
            chain_id: SupportedChain::Ethereum,
            chain_name: "Ethereum".to_string(),
            network_type: NetworkType::Mainnet,
            consensus_mechanism: ConsensusMechanism::ProofOfStake,
            native_token: TokenInfo {
                symbol: "ETH".to_string(),
                name: "Ether".to_string(),
                decimals: 18,
                total_supply: None,
                contract_address: None,
            },
            rpc_endpoints: vec![
                RPCEndpoint {
                    url: "https://mainnet.infura.io/v3/".to_string(),
                    endpoint_type: EndpointType::HTTPS,
                    authentication: None,
                    rate_limit: Some(RateLimit {
                        requests_per_second: 10,
                        burst_limit: 50,
                        window_size: Duration::from_secs(1),
                    }),
                    reliability_score: 0.95,
                    latency_ms: 200,
                    active: true,
                },
            ],
            block_explorers: vec!["https://etherscan.io".to_string()],
            smart_contract_support: SmartContractSupport {
                supported: true,
                vm_type: Some(VirtualMachineType::EVM),
                programming_languages: vec!["Solidity".to_string(), "Vyper".to_string()],
                gas_model: GasModel {
                    has_gas_concept: true,
                    gas_unit_name: "gwei".to_string(),
                    base_fee: Some(20),
                    priority_fee_support: true,
                },
                max_contract_size: Some(24576), // 24KB
                deployment_cost: Some(1000000), // 1M gas
            },
            interop_capabilities: InteropCapabilities {
                bridge_protocols: vec![
                    BridgeProtocolType::LockAndMint,
                    BridgeProtocolType::Atomic,
                    BridgeProtocolType::Multisig,
                ],
                atomic_swaps: true,
                cross_chain_messaging: true,
                ibc_support: false,
                xcmp_support: false,
                relay_chain_support: false,
                state_channels: true,
                sidechains: true,
            },
            security_features: vec![
                SecurityFeature::MultiSignature,
                SecurityFeature::TimeLocks,
                SecurityFeature::ZeroKnowledgeProofs,
                SecurityFeature::FormalVerification,
            ],
            performance_metrics: PerformanceMetrics {
                avg_block_time: Duration::from_secs(12),
                max_tps: 15,
                finality_time: Duration::from_secs(180),
                network_latency: Duration::from_millis(200),
                storage_efficiency: 0.7,
                energy_consumption: EnergyConsumption {
                    watts_per_transaction: Some(62.56),
                    carbon_footprint_rating: CarbonFootprintRating::Medium,
                    renewable_energy_percentage: Some(25.0),
                },
            },
            governance_model: GovernanceModel::OffChain,
            status: ChainStatus::Active,
            registration_date: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            last_updated: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
        });

        // Register Bitcoin
        chains.insert(SupportedChain::Bitcoin, ChainInfo {
            chain_id: SupportedChain::Bitcoin,
            chain_name: "Bitcoin".to_string(),
            network_type: NetworkType::Mainnet,
            consensus_mechanism: ConsensusMechanism::ProofOfWork,
            native_token: TokenInfo {
                symbol: "BTC".to_string(),
                name: "Bitcoin".to_string(),
                decimals: 8,
                total_supply: Some(21_000_000_00_000_000), // 21M BTC in satoshis
                contract_address: None,
            },
            rpc_endpoints: vec![
                RPCEndpoint {
                    url: "https://bitcoincore.org/rpc".to_string(),
                    endpoint_type: EndpointType::HTTPS,
                    authentication: None,
                    rate_limit: Some(RateLimit {
                        requests_per_second: 5,
                        burst_limit: 20,
                        window_size: Duration::from_secs(1),
                    }),
                    reliability_score: 0.98,
                    latency_ms: 500,
                    active: true,
                },
            ],
            block_explorers: vec!["https://blockstream.info".to_string()],
            smart_contract_support: SmartContractSupport {
                supported: false,
                vm_type: None,
                programming_languages: vec!["Script".to_string()],
                gas_model: GasModel {
                    has_gas_concept: false,
                    gas_unit_name: "satoshis/byte".to_string(),
                    base_fee: Some(1),
                    priority_fee_support: true,
                },
                max_contract_size: None,
                deployment_cost: None,
            },
            interop_capabilities: InteropCapabilities {
                bridge_protocols: vec![
                    BridgeProtocolType::LockAndMint,
                    BridgeProtocolType::Atomic,
                ],
                atomic_swaps: true,
                cross_chain_messaging: false,
                ibc_support: false,
                xcmp_support: false,
                relay_chain_support: false,
                state_channels: true,
                sidechains: true,
            },
            security_features: vec![
                SecurityFeature::MultiSignature,
                SecurityFeature::TimeLocks,
            ],
            performance_metrics: PerformanceMetrics {
                avg_block_time: Duration::from_secs(600), // 10 minutes
                max_tps: 7,
                finality_time: Duration::from_secs(3600), // 1 hour
                network_latency: Duration::from_millis(500),
                storage_efficiency: 0.9,
                energy_consumption: EnergyConsumption {
                    watts_per_transaction: Some(741000.0), // Very high
                    carbon_footprint_rating: CarbonFootprintRating::VeryHigh,
                    renewable_energy_percentage: Some(39.0),
                },
            },
            governance_model: GovernanceModel::OffChain,
            status: ChainStatus::Active,
            registration_date: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            last_updated: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
        });

        // Register Polkadot
        chains.insert(SupportedChain::Polkadot, ChainInfo {
            chain_id: SupportedChain::Polkadot,
            chain_name: "Polkadot".to_string(),
            network_type: NetworkType::Mainnet,
            consensus_mechanism: ConsensusMechanism::ProofOfStake,
            native_token: TokenInfo {
                symbol: "DOT".to_string(),
                name: "Polkadot".to_string(),
                decimals: 10,
                total_supply: Some(1_070_000_000_0000000000), // ~1.07B DOT
                contract_address: None,
            },
            rpc_endpoints: vec![
                RPCEndpoint {
                    url: "wss://rpc.polkadot.io".to_string(),
                    endpoint_type: EndpointType::WebSocketSecure,
                    authentication: None,
                    rate_limit: Some(RateLimit {
                        requests_per_second: 50,
                        burst_limit: 200,
                        window_size: Duration::from_secs(1),
                    }),
                    reliability_score: 0.92,
                    latency_ms: 150,
                    active: true,
                },
            ],
            block_explorers: vec!["https://polkascan.io".to_string()],
            smart_contract_support: SmartContractSupport {
                supported: true,
                vm_type: Some(VirtualMachineType::WASM),
                programming_languages: vec!["Rust".to_string(), "AssemblyScript".to_string()],
                gas_model: GasModel {
                    has_gas_concept: true,
                    gas_unit_name: "weight".to_string(),
                    base_fee: Some(125000000), // 0.125 DOT
                    priority_fee_support: false,
                },
                max_contract_size: Some(128 * 1024), // 128KB
                deployment_cost: Some(1000000),
            },
            interop_capabilities: InteropCapabilities {
                bridge_protocols: vec![
                    BridgeProtocolType::XCMP,
                    BridgeProtocolType::Relay,
                    BridgeProtocolType::LockAndMint,
                ],
                atomic_swaps: true,
                cross_chain_messaging: true,
                ibc_support: false,
                xcmp_support: true,
                relay_chain_support: true,
                state_channels: false,
                sidechains: true,
            },
            security_features: vec![
                SecurityFeature::MultiSignature,
                SecurityFeature::FormalVerification,
                SecurityFeature::QuantumResistance,
            ],
            performance_metrics: PerformanceMetrics {
                avg_block_time: Duration::from_secs(6),
                max_tps: 1000,
                finality_time: Duration::from_secs(60),
                network_latency: Duration::from_millis(150),
                storage_efficiency: 0.85,
                energy_consumption: EnergyConsumption {
                    watts_per_transaction: Some(0.126),
                    carbon_footprint_rating: CarbonFootprintRating::VeryLow,
                    renewable_energy_percentage: Some(60.0),
                },
            },
            governance_model: GovernanceModel::OnChain,
            status: ChainStatus::Active,
            registration_date: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            last_updated: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
        });

        Ok(())
    }

    async fn setup_network_monitors(&self) -> Result<()> {
        let chains = self.registered_chains.read().await;
        let mut monitors = self.network_monitors.write().await;

        for (chain_id, _chain_info) in chains.iter() {
            let monitor = NetworkMonitor {
                chain_id: chain_id.clone(),
                health_metrics: Arc::new(RwLock::new(HealthMetrics {
                    is_online: true,
                    block_height: 0,
                    peer_count: 0,
                    sync_status: SyncStatus::Unknown,
                    mempool_size: 0,
                    network_hashrate: None,
                    validator_count: None,
                    last_block_time: 0,
                    avg_response_time: Duration::from_millis(0),
                    error_rate: 0.0,
                    uptime_percentage: 100.0,
                })),
                connection_pool: Arc::new(ConnectionPool {
                    active_connections: 0,
                    max_connections: 100,
                    connection_timeout: Duration::from_secs(30),
                    retry_policy: RetryPolicy {
                        max_retries: 3,
                        base_delay: Duration::from_millis(1000),
                        backoff_multiplier: 2.0,
                        max_delay: Duration::from_secs(30),
                    },
                }),
                monitoring_config: MonitoringConfig {
                    check_interval: Duration::from_secs(30),
                    health_check_timeout: Duration::from_secs(10),
                    metrics_retention_period: Duration::from_secs(86400 * 7), // 7 days
                    alert_thresholds: AlertThresholds {
                        max_response_time: Duration::from_secs(5),
                        min_uptime_percentage: 95.0,
                        max_error_rate: 5.0,
                        max_sync_lag: 10,
                    },
                },
                alert_system: Arc::new(AlertSystem {
                    alert_channels: vec![],
                    alert_rules: vec![],
                    alert_history: Arc::new(RwLock::new(vec![])),
                }),
            };

            monitors.insert(chain_id.clone(), monitor);
        }

        Ok(())
    }

    async fn build_compatibility_matrix(&self) -> Result<()> {
        let chains = self.registered_chains.read().await;
        let mut matrix = self.compatibility_matrix.write().await;

        // Build compatibility information for all chain pairs
        for (chain_a, info_a) in chains.iter() {
            for (chain_b, info_b) in chains.iter() {
                if chain_a != chain_b {
                    let supported_protocols = self.find_common_protocols(
                        &info_a.interop_capabilities,
                        &info_b.interop_capabilities
                    );

                    let compatibility = CompatibilityInfo {
                        compatible: !supported_protocols.is_empty(),
                        supported_protocols: supported_protocols.clone(),
                        limitations: self.identify_limitations(chain_a, chain_b),
                        estimated_transfer_time: self.estimate_transfer_time(chain_a, chain_b),
                        estimated_cost: self.estimate_transfer_cost(chain_a, chain_b),
                        security_level: self.determine_security_level(&supported_protocols),
                    };

                    matrix.chain_pairs.insert((chain_a.clone(), chain_b.clone()), compatibility);
                }
            }
        }

        Ok(())
    }

    fn find_common_protocols(
        &self,
        cap_a: &InteropCapabilities,
        cap_b: &InteropCapabilities,
    ) -> Vec<InteropProtocol> {
        let mut common = Vec::new();

        if cap_a.atomic_swaps && cap_b.atomic_swaps {
            common.push(InteropProtocol::AtomicSwap);
        }

        if cap_a.ibc_support && cap_b.ibc_support {
            common.push(InteropProtocol::IBC);
        }

        if cap_a.xcmp_support && cap_b.xcmp_support {
            common.push(InteropProtocol::XCMP);
        }

        if cap_a.state_channels && cap_b.state_channels {
            common.push(InteropProtocol::StateChannel);
        }

        // Always include direct bridge as fallback
        common.push(InteropProtocol::DirectBridge);

        common
    }

    fn identify_limitations(&self, _chain_a: &SupportedChain, _chain_b: &SupportedChain) -> Vec<String> {
        // Placeholder implementation
        vec![]
    }

    fn estimate_transfer_time(&self, chain_a: &SupportedChain, chain_b: &SupportedChain) -> Duration {
        // Simplified estimation based on chain types
        match (chain_a, chain_b) {
            (SupportedChain::Bitcoin, _) | (_, SupportedChain::Bitcoin) => Duration::from_secs(3600),
            (SupportedChain::Ethereum, SupportedChain::Polkadot) => Duration::from_secs(300),
            _ => Duration::from_secs(600),
        }
    }

    fn estimate_transfer_cost(&self, chain_a: &SupportedChain, chain_b: &SupportedChain) -> u64 {
        // Simplified cost estimation
        match (chain_a, chain_b) {
            (SupportedChain::Ethereum, _) | (_, SupportedChain::Ethereum) => 50_000,
            (SupportedChain::Bitcoin, _) | (_, SupportedChain::Bitcoin) => 10_000,
            _ => 20_000,
        }
    }

    fn determine_security_level(&self, protocols: &[InteropProtocol]) -> SecurityLevel {
        if protocols.contains(&InteropProtocol::AtomicSwap) {
            SecurityLevel::High
        } else if protocols.contains(&InteropProtocol::IBC) || protocols.contains(&InteropProtocol::XCMP) {
            SecurityLevel::Medium
        } else {
            SecurityLevel::Low
        }
    }

    pub async fn register_chain(&self, chain_info: ChainInfo) -> Result<()> {
        let mut chains = self.registered_chains.write().await;
        chains.insert(chain_info.chain_id.clone(), chain_info);
        Ok(())
    }

    pub async fn get_chain_info(&self, chain_id: &SupportedChain) -> Result<Option<ChainInfo>> {
        let chains = self.registered_chains.read().await;
        Ok(chains.get(chain_id).cloned())
    }

    pub async fn update_chain_status(&self, chain_id: &SupportedChain, status: ChainStatus) -> Result<()> {
        let mut chains = self.registered_chains.write().await;
        if let Some(chain_info) = chains.get_mut(chain_id) {
            chain_info.status = status;
            chain_info.last_updated = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs();
        }
        Ok(())
    }

    pub async fn get_compatible_chains(&self, chain_id: &SupportedChain) -> Result<Vec<SupportedChain>> {
        let matrix = self.compatibility_matrix.read().await;
        let mut compatible = Vec::new();

        for ((chain_a, chain_b), compatibility) in matrix.chain_pairs.iter() {
            if chain_a == chain_id && compatibility.compatible {
                compatible.push(chain_b.clone());
            }
        }

        Ok(compatible)
    }

    pub async fn get_compatibility_info(
        &self,
        chain_a: &SupportedChain,
        chain_b: &SupportedChain,
    ) -> Result<Option<CompatibilityInfo>> {
        let matrix = self.compatibility_matrix.read().await;
        Ok(matrix.chain_pairs.get(&(chain_a.clone(), chain_b.clone())).cloned())
    }

    pub async fn get_health_metrics(&self, chain_id: &SupportedChain) -> Result<Option<HealthMetrics>> {
        let monitors = self.network_monitors.read().await;
        if let Some(monitor) = monitors.get(chain_id) {
            Ok(Some(monitor.health_metrics.read().await.clone()))
        } else {
            Ok(None)
        }
    }

    pub async fn update_health_metrics(&self, chain_id: &SupportedChain, metrics: HealthMetrics) -> Result<()> {
        let monitors = self.network_monitors.read().await;
        if let Some(monitor) = monitors.get(chain_id) {
            *monitor.health_metrics.write().await = metrics;
        }
        Ok(())
    }

    pub async fn get_all_chains(&self) -> Result<Vec<ChainInfo>> {
        let chains = self.registered_chains.read().await;
        Ok(chains.values().cloned().collect())
    }

    pub async fn get_active_chains(&self) -> Result<Vec<ChainInfo>> {
        let chains = self.registered_chains.read().await;
        Ok(chains.values()
            .filter(|chain| chain.status == ChainStatus::Active)
            .cloned()
            .collect())
    }
}