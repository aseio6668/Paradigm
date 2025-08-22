//! Network configuration and monitoring
//!
//! This module provides network-related functionality including network configuration,
//! peer management, and network monitoring for the Paradigm blockchain.

use crate::error::{ErrorExt, ParadigmError, Result};
use crate::types::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::net::{IpAddr, SocketAddr};
use std::sync::Arc;
use std::time::{Duration, SystemTime};
use tokio::sync::RwLock;
use uuid::Uuid;

/// Network configuration for Paradigm blockchain
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkConfig {
    /// Network name
    pub name: String,
    /// Chain ID
    pub chain_id: u64,
    /// Network type
    pub network_type: NetworkType,
    /// RPC endpoints
    pub rpc_endpoints: Vec<RpcEndpoint>,
    /// WebSocket endpoints
    pub ws_endpoints: Vec<WsEndpoint>,
    /// Bootstrap nodes for P2P networking
    pub bootstrap_nodes: Vec<PeerInfo>,
    /// Network consensus configuration
    pub consensus_config: ConsensusConfig,
    /// P2P networking configuration
    pub p2p_config: P2pConfig,
    /// Security configuration
    pub security_config: SecurityConfig,
    /// Performance settings
    pub performance_config: PerformanceConfig,
}

impl NetworkConfig {
    /// Create mainnet configuration
    pub fn mainnet() -> Self {
        Self {
            name: "Paradigm Mainnet".to_string(),
            chain_id: crate::MAINNET_CHAIN_ID,
            network_type: NetworkType::Mainnet,
            rpc_endpoints: vec![
                RpcEndpoint {
                    url: "https://rpc.paradigm.network".to_string(),
                    priority: 1,
                    rate_limit: Some(1000),
                    auth_required: false,
                },
                RpcEndpoint {
                    url: "https://rpc-backup.paradigm.network".to_string(),
                    priority: 2,
                    rate_limit: Some(500),
                    auth_required: false,
                },
            ],
            ws_endpoints: vec![WsEndpoint {
                url: "wss://ws.paradigm.network".to_string(),
                priority: 1,
                max_connections: 100,
                auth_required: false,
            }],
            bootstrap_nodes: Self::mainnet_bootstrap_nodes(),
            consensus_config: ConsensusConfig::mainnet(),
            p2p_config: P2pConfig::mainnet(),
            security_config: SecurityConfig::mainnet(),
            performance_config: PerformanceConfig::mainnet(),
        }
    }

    /// Create testnet configuration
    pub fn testnet() -> Self {
        Self {
            name: "Paradigm Testnet".to_string(),
            chain_id: 11155111,
            network_type: NetworkType::Testnet,
            rpc_endpoints: vec![RpcEndpoint {
                url: "https://testnet-rpc.paradigm.network".to_string(),
                priority: 1,
                rate_limit: Some(2000),
                auth_required: false,
            }],
            ws_endpoints: vec![WsEndpoint {
                url: "wss://testnet-ws.paradigm.network".to_string(),
                priority: 1,
                max_connections: 50,
                auth_required: false,
            }],
            bootstrap_nodes: Self::testnet_bootstrap_nodes(),
            consensus_config: ConsensusConfig::testnet(),
            p2p_config: P2pConfig::testnet(),
            security_config: SecurityConfig::testnet(),
            performance_config: PerformanceConfig::testnet(),
        }
    }

    /// Create local development configuration
    pub fn local() -> Self {
        Self {
            name: "Paradigm Local".to_string(),
            chain_id: 1337,
            network_type: NetworkType::Local,
            rpc_endpoints: vec![RpcEndpoint {
                url: "http://localhost:8545".to_string(),
                priority: 1,
                rate_limit: None,
                auth_required: false,
            }],
            ws_endpoints: vec![WsEndpoint {
                url: "ws://localhost:8546".to_string(),
                priority: 1,
                max_connections: 10,
                auth_required: false,
            }],
            bootstrap_nodes: vec![],
            consensus_config: ConsensusConfig::local(),
            p2p_config: P2pConfig::local(),
            security_config: SecurityConfig::local(),
            performance_config: PerformanceConfig::local(),
        }
    }

    /// Get mainnet bootstrap nodes
    fn mainnet_bootstrap_nodes() -> Vec<PeerInfo> {
        vec![
            PeerInfo {
                id: "mainnet-bootstrap-1".to_string(),
                address: "34.234.123.45:30303".parse().unwrap(),
                public_key: None,
                node_type: NodeType::Bootstrap,
                version: "1.0.0".to_string(),
                capabilities: vec!["paradigm/1".to_string(), "snap/1".to_string()],
                last_seen: SystemTime::now(),
                reputation: 100,
                connection_status: ConnectionStatus::Unknown,
            },
            PeerInfo {
                id: "mainnet-bootstrap-2".to_string(),
                address: "52.123.45.67:30303".parse().unwrap(),
                public_key: None,
                node_type: NodeType::Bootstrap,
                version: "1.0.0".to_string(),
                capabilities: vec!["paradigm/1".to_string(), "snap/1".to_string()],
                last_seen: SystemTime::now(),
                reputation: 100,
                connection_status: ConnectionStatus::Unknown,
            },
        ]
    }

    /// Get testnet bootstrap nodes
    fn testnet_bootstrap_nodes() -> Vec<PeerInfo> {
        vec![PeerInfo {
            id: "testnet-bootstrap-1".to_string(),
            address: "testnet1.paradigm.network:30303".parse().unwrap(),
            public_key: None,
            node_type: NodeType::Bootstrap,
            version: "1.0.0".to_string(),
            capabilities: vec!["paradigm/1".to_string()],
            last_seen: SystemTime::now(),
            reputation: 100,
            connection_status: ConnectionStatus::Unknown,
        }]
    }
}

impl Default for NetworkConfig {
    fn default() -> Self {
        Self::mainnet()
    }
}

/// Network type enumeration
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum NetworkType {
    Mainnet,
    Testnet,
    Local,
    Custom(String),
}

/// RPC endpoint configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RpcEndpoint {
    /// Endpoint URL
    pub url: String,
    /// Priority (lower numbers = higher priority)
    pub priority: u32,
    /// Rate limit (requests per minute)
    pub rate_limit: Option<u32>,
    /// Whether authentication is required
    pub auth_required: bool,
}

/// WebSocket endpoint configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WsEndpoint {
    /// Endpoint URL
    pub url: String,
    /// Priority (lower numbers = higher priority)
    pub priority: u32,
    /// Maximum concurrent connections
    pub max_connections: u32,
    /// Whether authentication is required
    pub auth_required: bool,
}

/// Peer information and status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PeerInfo {
    /// Peer ID
    pub id: String,
    /// Network address
    pub address: SocketAddr,
    /// Public key (if known)
    pub public_key: Option<Vec<u8>>,
    /// Node type
    pub node_type: NodeType,
    /// Node version
    pub version: String,
    /// Supported capabilities/protocols
    pub capabilities: Vec<String>,
    /// Last seen timestamp
    pub last_seen: SystemTime,
    /// Reputation score (0-100)
    pub reputation: u32,
    /// Current connection status
    pub connection_status: ConnectionStatus,
}

/// Node type enumeration
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum NodeType {
    Full,
    Light,
    Archive,
    Bootstrap,
    Validator,
    Relay,
}

/// Connection status enumeration
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum ConnectionStatus {
    Connected,
    Connecting,
    Disconnected,
    Banned,
    Unknown,
}

/// Consensus configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConsensusConfig {
    /// Consensus algorithm
    pub algorithm: ConsensusAlgorithm,
    /// Block time in seconds
    pub block_time_seconds: u64,
    /// Epoch length in blocks
    pub epoch_length: u64,
    /// Minimum validator stake
    pub min_validator_stake: Amount,
    /// Maximum validators
    pub max_validators: u32,
    /// Slash conditions
    pub slash_conditions: Vec<SlashCondition>,
    /// Finality confirmations
    pub finality_confirmations: u32,
}

impl ConsensusConfig {
    /// Mainnet consensus configuration
    pub fn mainnet() -> Self {
        Self {
            algorithm: ConsensusAlgorithm::ProofOfStake,
            block_time_seconds: 12,
            epoch_length: 32,
            min_validator_stake: Amount::from_paradigm(32000.0),
            max_validators: 1000,
            slash_conditions: vec![
                SlashCondition::DoubleSign { penalty_percent: 5 },
                SlashCondition::Downtime {
                    penalty_percent: 1,
                    max_downtime_blocks: 8192,
                },
            ],
            finality_confirmations: 64,
        }
    }

    /// Testnet consensus configuration
    pub fn testnet() -> Self {
        Self {
            algorithm: ConsensusAlgorithm::ProofOfStake,
            block_time_seconds: 6,
            epoch_length: 16,
            min_validator_stake: Amount::from_paradigm(1000.0),
            max_validators: 100,
            slash_conditions: vec![
                SlashCondition::DoubleSign { penalty_percent: 1 },
                SlashCondition::Downtime {
                    penalty_percent: 0,
                    max_downtime_blocks: 1024,
                },
            ],
            finality_confirmations: 16,
        }
    }

    /// Local development consensus configuration
    pub fn local() -> Self {
        Self {
            algorithm: ConsensusAlgorithm::ProofOfAuthority,
            block_time_seconds: 2,
            epoch_length: 8,
            min_validator_stake: Amount::from_paradigm(100.0),
            max_validators: 10,
            slash_conditions: vec![],
            finality_confirmations: 1,
        }
    }
}

/// Consensus algorithm enumeration
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum ConsensusAlgorithm {
    ProofOfWork,
    ProofOfStake,
    ProofOfAuthority,
    DelegatedProofOfStake,
    PracticalByzantineFaultTolerance,
}

/// Slash condition for validators
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SlashCondition {
    DoubleSign {
        penalty_percent: u32,
    },
    Downtime {
        penalty_percent: u32,
        max_downtime_blocks: u64,
    },
    InvalidBlock {
        penalty_percent: u32,
    },
    Censorship {
        penalty_percent: u32,
    },
}

/// P2P networking configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct P2pConfig {
    /// Listen address
    pub listen_address: SocketAddr,
    /// Maximum inbound connections
    pub max_inbound_peers: u32,
    /// Maximum outbound connections
    pub max_outbound_peers: u32,
    /// Connection timeout
    pub connection_timeout_seconds: u64,
    /// Heartbeat interval
    pub heartbeat_interval_seconds: u64,
    /// Message propagation settings
    pub propagation_config: PropagationConfig,
    /// Discovery settings
    pub discovery_config: DiscoveryConfig,
}

impl P2pConfig {
    /// Mainnet P2P configuration
    pub fn mainnet() -> Self {
        Self {
            listen_address: "0.0.0.0:30303".parse().unwrap(),
            max_inbound_peers: 50,
            max_outbound_peers: 25,
            connection_timeout_seconds: 30,
            heartbeat_interval_seconds: 15,
            propagation_config: PropagationConfig::mainnet(),
            discovery_config: DiscoveryConfig::mainnet(),
        }
    }

    /// Testnet P2P configuration
    pub fn testnet() -> Self {
        Self {
            listen_address: "0.0.0.0:30303".parse().unwrap(),
            max_inbound_peers: 25,
            max_outbound_peers: 15,
            connection_timeout_seconds: 20,
            heartbeat_interval_seconds: 10,
            propagation_config: PropagationConfig::testnet(),
            discovery_config: DiscoveryConfig::testnet(),
        }
    }

    /// Local development P2P configuration
    pub fn local() -> Self {
        Self {
            listen_address: "127.0.0.1:30303".parse().unwrap(),
            max_inbound_peers: 5,
            max_outbound_peers: 5,
            connection_timeout_seconds: 10,
            heartbeat_interval_seconds: 5,
            propagation_config: PropagationConfig::local(),
            discovery_config: DiscoveryConfig::local(),
        }
    }
}

/// Message propagation configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PropagationConfig {
    /// Maximum hops for message propagation
    pub max_hops: u32,
    /// Fanout factor for gossip
    pub gossip_fanout: u32,
    /// Message TTL in seconds
    pub message_ttl_seconds: u64,
    /// Duplicate detection cache size
    pub duplicate_cache_size: usize,
}

impl PropagationConfig {
    pub fn mainnet() -> Self {
        Self {
            max_hops: 8,
            gossip_fanout: 8,
            message_ttl_seconds: 300,
            duplicate_cache_size: 10000,
        }
    }

    pub fn testnet() -> Self {
        Self {
            max_hops: 6,
            gossip_fanout: 6,
            message_ttl_seconds: 180,
            duplicate_cache_size: 5000,
        }
    }

    pub fn local() -> Self {
        Self {
            max_hops: 3,
            gossip_fanout: 3,
            message_ttl_seconds: 60,
            duplicate_cache_size: 1000,
        }
    }
}

/// Peer discovery configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiscoveryConfig {
    /// Enable mDNS discovery
    pub enable_mdns: bool,
    /// Enable DHT-based discovery
    pub enable_dht: bool,
    /// Bootstrap discovery interval
    pub discovery_interval_seconds: u64,
    /// Peer exchange enabled
    pub enable_peer_exchange: bool,
    /// Maximum discovered peers to remember
    pub max_discovered_peers: usize,
}

impl DiscoveryConfig {
    pub fn mainnet() -> Self {
        Self {
            enable_mdns: false,
            enable_dht: true,
            discovery_interval_seconds: 60,
            enable_peer_exchange: true,
            max_discovered_peers: 1000,
        }
    }

    pub fn testnet() -> Self {
        Self {
            enable_mdns: true,
            enable_dht: true,
            discovery_interval_seconds: 30,
            enable_peer_exchange: true,
            max_discovered_peers: 500,
        }
    }

    pub fn local() -> Self {
        Self {
            enable_mdns: true,
            enable_dht: false,
            discovery_interval_seconds: 10,
            enable_peer_exchange: false,
            max_discovered_peers: 50,
        }
    }
}

/// Security configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityConfig {
    /// Enable TLS for connections
    pub enable_tls: bool,
    /// Require authentication
    pub require_auth: bool,
    /// Rate limiting settings
    pub rate_limits: RateLimits,
    /// Firewall rules
    pub firewall_rules: Vec<FirewallRule>,
    /// DDoS protection settings
    pub ddos_protection: DdosProtection,
}

impl SecurityConfig {
    pub fn mainnet() -> Self {
        Self {
            enable_tls: true,
            require_auth: false,
            rate_limits: RateLimits::mainnet(),
            firewall_rules: vec![
                FirewallRule::AllowPort { port: 30303 },
                FirewallRule::BlockIpRange {
                    start: "192.168.0.0".parse().unwrap(),
                    end: "192.168.255.255".parse().unwrap(),
                },
            ],
            ddos_protection: DdosProtection::mainnet(),
        }
    }

    pub fn testnet() -> Self {
        Self {
            enable_tls: false,
            require_auth: false,
            rate_limits: RateLimits::testnet(),
            firewall_rules: vec![FirewallRule::AllowPort { port: 30303 }],
            ddos_protection: DdosProtection::testnet(),
        }
    }

    pub fn local() -> Self {
        Self {
            enable_tls: false,
            require_auth: false,
            rate_limits: RateLimits::local(),
            firewall_rules: vec![],
            ddos_protection: DdosProtection::local(),
        }
    }
}

/// Rate limiting configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RateLimits {
    /// Requests per second per peer
    pub requests_per_second: u32,
    /// Burst size
    pub burst_size: u32,
    /// Bandwidth limit in bytes per second
    pub bandwidth_limit_bps: u64,
}

impl RateLimits {
    pub fn mainnet() -> Self {
        Self {
            requests_per_second: 100,
            burst_size: 200,
            bandwidth_limit_bps: 10_000_000, // 10 MB/s
        }
    }

    pub fn testnet() -> Self {
        Self {
            requests_per_second: 200,
            burst_size: 400,
            bandwidth_limit_bps: 50_000_000, // 50 MB/s
        }
    }

    pub fn local() -> Self {
        Self {
            requests_per_second: 1000,
            burst_size: 2000,
            bandwidth_limit_bps: 100_000_000, // 100 MB/s
        }
    }
}

/// Firewall rule
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FirewallRule {
    AllowPort { port: u16 },
    BlockPort { port: u16 },
    AllowIp { ip: IpAddr },
    BlockIp { ip: IpAddr },
    AllowIpRange { start: IpAddr, end: IpAddr },
    BlockIpRange { start: IpAddr, end: IpAddr },
}

/// DDoS protection configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DdosProtection {
    /// Enable DDoS protection
    pub enabled: bool,
    /// Connection limit per IP
    pub max_connections_per_ip: u32,
    /// Request rate limit per IP
    pub max_requests_per_ip_per_minute: u32,
    /// Blacklist duration in seconds
    pub blacklist_duration_seconds: u64,
}

impl DdosProtection {
    pub fn mainnet() -> Self {
        Self {
            enabled: true,
            max_connections_per_ip: 10,
            max_requests_per_ip_per_minute: 1000,
            blacklist_duration_seconds: 3600, // 1 hour
        }
    }

    pub fn testnet() -> Self {
        Self {
            enabled: true,
            max_connections_per_ip: 20,
            max_requests_per_ip_per_minute: 2000,
            blacklist_duration_seconds: 1800, // 30 minutes
        }
    }

    pub fn local() -> Self {
        Self {
            enabled: false,
            max_connections_per_ip: 100,
            max_requests_per_ip_per_minute: 10000,
            blacklist_duration_seconds: 60,
        }
    }
}

/// Performance configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceConfig {
    /// Buffer sizes
    pub buffer_sizes: BufferSizes,
    /// Thread pool settings
    pub thread_pools: ThreadPoolConfig,
    /// Cache settings
    pub cache_config: CacheConfig,
    /// Batch processing settings
    pub batch_config: BatchConfig,
}

impl PerformanceConfig {
    pub fn mainnet() -> Self {
        Self {
            buffer_sizes: BufferSizes::mainnet(),
            thread_pools: ThreadPoolConfig::mainnet(),
            cache_config: CacheConfig::mainnet(),
            batch_config: BatchConfig::mainnet(),
        }
    }

    pub fn testnet() -> Self {
        Self {
            buffer_sizes: BufferSizes::testnet(),
            thread_pools: ThreadPoolConfig::testnet(),
            cache_config: CacheConfig::testnet(),
            batch_config: BatchConfig::testnet(),
        }
    }

    pub fn local() -> Self {
        Self {
            buffer_sizes: BufferSizes::local(),
            thread_pools: ThreadPoolConfig::local(),
            cache_config: CacheConfig::local(),
            batch_config: BatchConfig::local(),
        }
    }
}

/// Buffer size configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BufferSizes {
    /// Network receive buffer size
    pub network_receive_buffer: usize,
    /// Network send buffer size
    pub network_send_buffer: usize,
    /// Message queue buffer size
    pub message_queue_buffer: usize,
    /// Transaction pool buffer size
    pub transaction_pool_buffer: usize,
}

impl BufferSizes {
    pub fn mainnet() -> Self {
        Self {
            network_receive_buffer: 1024 * 1024, // 1 MB
            network_send_buffer: 1024 * 1024,    // 1 MB
            message_queue_buffer: 10000,
            transaction_pool_buffer: 50000,
        }
    }

    pub fn testnet() -> Self {
        Self {
            network_receive_buffer: 512 * 1024, // 512 KB
            network_send_buffer: 512 * 1024,    // 512 KB
            message_queue_buffer: 5000,
            transaction_pool_buffer: 25000,
        }
    }

    pub fn local() -> Self {
        Self {
            network_receive_buffer: 64 * 1024, // 64 KB
            network_send_buffer: 64 * 1024,    // 64 KB
            message_queue_buffer: 1000,
            transaction_pool_buffer: 5000,
        }
    }
}

/// Thread pool configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThreadPoolConfig {
    /// Network I/O threads
    pub network_threads: usize,
    /// Transaction processing threads
    pub transaction_threads: usize,
    /// Block processing threads
    pub block_processing_threads: usize,
    /// Validation threads
    pub validation_threads: usize,
}

impl ThreadPoolConfig {
    pub fn mainnet() -> Self {
        Self {
            network_threads: 8,
            transaction_threads: 16,
            block_processing_threads: 4,
            validation_threads: 8,
        }
    }

    pub fn testnet() -> Self {
        Self {
            network_threads: 4,
            transaction_threads: 8,
            block_processing_threads: 2,
            validation_threads: 4,
        }
    }

    pub fn local() -> Self {
        Self {
            network_threads: 2,
            transaction_threads: 4,
            block_processing_threads: 1,
            validation_threads: 2,
        }
    }
}

/// Cache configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheConfig {
    /// Block cache size (number of blocks)
    pub block_cache_size: usize,
    /// Transaction cache size (number of transactions)
    pub transaction_cache_size: usize,
    /// State cache size in MB
    pub state_cache_size_mb: usize,
    /// Peer cache size (number of peers)
    pub peer_cache_size: usize,
}

impl CacheConfig {
    pub fn mainnet() -> Self {
        Self {
            block_cache_size: 1000,
            transaction_cache_size: 100000,
            state_cache_size_mb: 512,
            peer_cache_size: 10000,
        }
    }

    pub fn testnet() -> Self {
        Self {
            block_cache_size: 500,
            transaction_cache_size: 50000,
            state_cache_size_mb: 256,
            peer_cache_size: 5000,
        }
    }

    pub fn local() -> Self {
        Self {
            block_cache_size: 100,
            transaction_cache_size: 10000,
            state_cache_size_mb: 64,
            peer_cache_size: 1000,
        }
    }
}

/// Batch processing configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchConfig {
    /// Transaction batch size
    pub transaction_batch_size: usize,
    /// Message batch size
    pub message_batch_size: usize,
    /// Batch timeout in milliseconds
    pub batch_timeout_ms: u64,
    /// Maximum batch processing time
    pub max_batch_processing_time_ms: u64,
}

impl BatchConfig {
    pub fn mainnet() -> Self {
        Self {
            transaction_batch_size: 1000,
            message_batch_size: 100,
            batch_timeout_ms: 100,
            max_batch_processing_time_ms: 1000,
        }
    }

    pub fn testnet() -> Self {
        Self {
            transaction_batch_size: 500,
            message_batch_size: 50,
            batch_timeout_ms: 50,
            max_batch_processing_time_ms: 500,
        }
    }

    pub fn local() -> Self {
        Self {
            transaction_batch_size: 100,
            message_batch_size: 10,
            batch_timeout_ms: 10,
            max_batch_processing_time_ms: 100,
        }
    }
}

/// Network status and monitoring
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkStatus {
    /// Current network configuration
    pub config: NetworkConfig,
    /// Connected peers
    pub connected_peers: u32,
    /// Total discovered peers
    pub total_peers: u32,
    /// Current block height
    pub block_height: u64,
    /// Sync status
    pub sync_status: SyncStatus,
    /// Network health metrics
    pub health_metrics: NetworkHealthMetrics,
    /// Network statistics
    pub statistics: NetworkStatistics,
    /// Last updated timestamp
    pub last_updated: SystemTime,
}

/// Synchronization status
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum SyncStatus {
    Syncing {
        current_block: u64,
        target_block: u64,
    },
    Synced,
    NotSynced,
    Failed(String),
}

/// Network health metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkHealthMetrics {
    /// Peer connectivity score (0-100)
    pub peer_connectivity: u32,
    /// Network latency in milliseconds
    pub average_latency_ms: u64,
    /// Message delivery rate (0-100)
    pub message_delivery_rate: u32,
    /// Block propagation time in milliseconds
    pub block_propagation_time_ms: u64,
    /// Overall network health score (0-100)
    pub overall_health: u32,
}

/// Network statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkStatistics {
    /// Total bytes sent
    pub bytes_sent: u64,
    /// Total bytes received
    pub bytes_received: u64,
    /// Total messages sent
    pub messages_sent: u64,
    /// Total messages received
    pub messages_received: u64,
    /// Connection attempts
    pub connection_attempts: u64,
    /// Successful connections
    pub successful_connections: u64,
    /// Failed connections
    pub failed_connections: u64,
    /// Peer ban count
    pub banned_peers: u32,
}

/// Network manager for monitoring and configuration
#[derive(Debug)]
pub struct NetworkManager {
    /// Current network configuration
    config: Arc<RwLock<NetworkConfig>>,
    /// Known peers
    peers: Arc<RwLock<HashMap<String, PeerInfo>>>,
    /// Network statistics
    statistics: Arc<RwLock<NetworkStatistics>>,
    /// Network status
    status: Arc<RwLock<NetworkStatus>>,
}

impl NetworkManager {
    /// Create a new network manager
    pub fn new(config: NetworkConfig) -> Self {
        let status = NetworkStatus {
            config: config.clone(),
            connected_peers: 0,
            total_peers: 0,
            block_height: 0,
            sync_status: SyncStatus::NotSynced,
            health_metrics: NetworkHealthMetrics {
                peer_connectivity: 0,
                average_latency_ms: 0,
                message_delivery_rate: 0,
                block_propagation_time_ms: 0,
                overall_health: 0,
            },
            statistics: NetworkStatistics {
                bytes_sent: 0,
                bytes_received: 0,
                messages_sent: 0,
                messages_received: 0,
                connection_attempts: 0,
                successful_connections: 0,
                failed_connections: 0,
                banned_peers: 0,
            },
            last_updated: SystemTime::now(),
        };

        Self {
            config: Arc::new(RwLock::new(config)),
            peers: Arc::new(RwLock::new(HashMap::new())),
            statistics: Arc::new(RwLock::new(status.statistics.clone())),
            status: Arc::new(RwLock::new(status)),
        }
    }

    /// Get current network configuration
    pub async fn get_config(&self) -> NetworkConfig {
        self.config.read().await.clone()
    }

    /// Update network configuration
    pub async fn update_config(&self, config: NetworkConfig) {
        *self.config.write().await = config.clone();

        let mut status = self.status.write().await;
        status.config = config;
        status.last_updated = SystemTime::now();
    }

    /// Get network status
    pub async fn get_status(&self) -> NetworkStatus {
        self.status.read().await.clone()
    }

    /// Add or update peer information
    pub async fn update_peer(&self, peer: PeerInfo) {
        self.peers.write().await.insert(peer.id.clone(), peer);
        self.update_status().await;
    }

    /// Remove peer
    pub async fn remove_peer(&self, peer_id: &str) {
        self.peers.write().await.remove(peer_id);
        self.update_status().await;
    }

    /// Get all known peers
    pub async fn get_peers(&self) -> Vec<PeerInfo> {
        self.peers.read().await.values().cloned().collect()
    }

    /// Get connected peers
    pub async fn get_connected_peers(&self) -> Vec<PeerInfo> {
        self.peers
            .read()
            .await
            .values()
            .filter(|peer| peer.connection_status == ConnectionStatus::Connected)
            .cloned()
            .collect()
    }

    /// Get peer by ID
    pub async fn get_peer(&self, peer_id: &str) -> Option<PeerInfo> {
        self.peers.read().await.get(peer_id).cloned()
    }

    /// Ban peer
    pub async fn ban_peer(&self, peer_id: &str, reason: String) -> Result<()> {
        let mut peers = self.peers.write().await;
        if let Some(peer) = peers.get_mut(peer_id) {
            peer.connection_status = ConnectionStatus::Banned;
            peer.reputation = 0;

            // Update statistics
            let mut stats = self.statistics.write().await;
            stats.banned_peers += 1;

            self.update_status().await;
            Ok(())
        } else {
            Err(ParadigmError::NotFound(format!(
                "Peer {} not found",
                peer_id
            )))
        }
    }

    /// Update network statistics
    pub async fn update_statistics(&self, stats_update: NetworkStatisticsUpdate) {
        let mut stats = self.statistics.write().await;

        match stats_update {
            NetworkStatisticsUpdate::BytesSent(bytes) => stats.bytes_sent += bytes,
            NetworkStatisticsUpdate::BytesReceived(bytes) => stats.bytes_received += bytes,
            NetworkStatisticsUpdate::MessageSent => stats.messages_sent += 1,
            NetworkStatisticsUpdate::MessageReceived => stats.messages_received += 1,
            NetworkStatisticsUpdate::ConnectionAttempt => stats.connection_attempts += 1,
            NetworkStatisticsUpdate::SuccessfulConnection => stats.successful_connections += 1,
            NetworkStatisticsUpdate::FailedConnection => stats.failed_connections += 1,
        }

        self.update_status().await;
    }

    /// Update network status (internal)
    async fn update_status(&self) {
        let peers = self.peers.read().await;
        let connected_peers = peers
            .values()
            .filter(|peer| peer.connection_status == ConnectionStatus::Connected)
            .count() as u32;

        let total_peers = peers.len() as u32;
        let statistics = self.statistics.read().await.clone();

        // Calculate health metrics
        let peer_connectivity = if total_peers > 0 {
            (connected_peers * 100) / total_peers
        } else {
            0
        };

        let average_latency_ms = 50; // Placeholder - would be calculated from actual measurements
        let message_delivery_rate = 95; // Placeholder - would be calculated from statistics
        let block_propagation_time_ms = 500; // Placeholder

        let overall_health = (peer_connectivity
            + (100 - (average_latency_ms / 10).min(100))
            + message_delivery_rate)
            / 3;

        let health_metrics = NetworkHealthMetrics {
            peer_connectivity,
            average_latency_ms,
            message_delivery_rate,
            block_propagation_time_ms,
            overall_health,
        };

        let mut status = self.status.write().await;
        status.connected_peers = connected_peers;
        status.total_peers = total_peers;
        status.health_metrics = health_metrics;
        status.statistics = statistics;
        status.last_updated = SystemTime::now();
    }

    /// Check network health
    pub async fn check_health(&self) -> NetworkHealthReport {
        let status = self.get_status().await;
        let mut issues = Vec::new();
        let mut recommendations = Vec::new();

        // Check peer connectivity
        if status.health_metrics.peer_connectivity < 50 {
            issues.push("Low peer connectivity".to_string());
            recommendations.push("Add more bootstrap nodes or check firewall settings".to_string());
        }

        // Check latency
        if status.health_metrics.average_latency_ms > 1000 {
            issues.push("High network latency".to_string());
            recommendations.push("Consider using closer RPC endpoints".to_string());
        }

        // Check sync status
        if status.sync_status == SyncStatus::NotSynced {
            issues.push("Node not synchronized".to_string());
            recommendations.push("Wait for initial synchronization to complete".to_string());
        }

        let overall_status = if issues.is_empty() {
            HealthStatus::Healthy
        } else if status.health_metrics.overall_health > 50 {
            HealthStatus::Warning
        } else {
            HealthStatus::Critical
        };

        NetworkHealthReport {
            status: overall_status,
            overall_score: status.health_metrics.overall_health,
            issues,
            recommendations,
            checked_at: SystemTime::now(),
        }
    }
}

/// Network statistics update enum
#[derive(Debug, Clone)]
pub enum NetworkStatisticsUpdate {
    BytesSent(u64),
    BytesReceived(u64),
    MessageSent,
    MessageReceived,
    ConnectionAttempt,
    SuccessfulConnection,
    FailedConnection,
}

/// Network health report
#[derive(Debug, Clone)]
pub struct NetworkHealthReport {
    pub status: HealthStatus,
    pub overall_score: u32,
    pub issues: Vec<String>,
    pub recommendations: Vec<String>,
    pub checked_at: SystemTime,
}

/// Health status enumeration
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HealthStatus {
    Healthy,
    Warning,
    Critical,
}

/// Network utilities
pub mod utils {
    use super::*;

    /// Validate network configuration
    pub fn validate_config(config: &NetworkConfig) -> Result<()> {
        if config.rpc_endpoints.is_empty() {
            return Err(ParadigmError::Config(
                "No RPC endpoints configured".to_string(),
            ));
        }

        if config.chain_id == 0 {
            return Err(ParadigmError::Config("Invalid chain ID".to_string()));
        }

        if config.p2p_config.max_inbound_peers == 0 && config.p2p_config.max_outbound_peers == 0 {
            return Err(ParadigmError::Config(
                "No peer connections allowed".to_string(),
            ));
        }

        Ok(())
    }

    /// Get network name by chain ID
    pub fn get_network_name(chain_id: u64) -> String {
        match chain_id {
            1 => "Paradigm Mainnet".to_string(),
            11155111 => "Paradigm Testnet".to_string(),
            1337 => "Paradigm Local".to_string(),
            _ => format!("Unknown Network ({})", chain_id),
        }
    }

    /// Calculate network score based on metrics
    pub fn calculate_network_score(metrics: &NetworkHealthMetrics) -> u32 {
        let connectivity_weight = 0.3;
        let latency_weight = 0.2;
        let delivery_weight = 0.3;
        let propagation_weight = 0.2;

        let latency_score = 100_u32.saturating_sub((metrics.average_latency_ms / 10) as u32);
        let propagation_score =
            100_u32.saturating_sub((metrics.block_propagation_time_ms / 50) as u32);

        let weighted_score = (metrics.peer_connectivity as f64 * connectivity_weight)
            + (latency_score as f64 * latency_weight)
            + (metrics.message_delivery_rate as f64 * delivery_weight)
            + (propagation_score as f64 * propagation_weight);

        weighted_score.round() as u32
    }

    /// Parse peer address string
    pub fn parse_peer_address(addr_str: &str) -> Result<SocketAddr> {
        addr_str
            .parse()
            .map_err(|e| ParadigmError::Network(format!("Invalid peer address: {}", e)))
    }

    /// Generate peer ID
    pub fn generate_peer_id() -> String {
        format!("peer-{}", Uuid::new_v4())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_network_config_creation() {
        let mainnet = NetworkConfig::mainnet();
        assert_eq!(mainnet.chain_id, crate::MAINNET_CHAIN_ID);
        assert_eq!(mainnet.network_type, NetworkType::Mainnet);

        let testnet = NetworkConfig::testnet();
        assert_eq!(testnet.chain_id, 11155111);
        assert_eq!(testnet.network_type, NetworkType::Testnet);

        let local = NetworkConfig::local();
        assert_eq!(local.chain_id, 1337);
        assert_eq!(local.network_type, NetworkType::Local);
    }

    #[test]
    fn test_consensus_config() {
        let mainnet_consensus = ConsensusConfig::mainnet();
        assert_eq!(
            mainnet_consensus.algorithm,
            ConsensusAlgorithm::ProofOfStake
        );
        assert_eq!(mainnet_consensus.block_time_seconds, 12);

        let local_consensus = ConsensusConfig::local();
        assert_eq!(
            local_consensus.algorithm,
            ConsensusAlgorithm::ProofOfAuthority
        );
        assert_eq!(local_consensus.block_time_seconds, 2);
    }

    #[tokio::test]
    async fn test_network_manager() {
        let config = NetworkConfig::local();
        let manager = NetworkManager::new(config.clone());

        let status = manager.get_status().await;
        assert_eq!(status.config.chain_id, config.chain_id);
        assert_eq!(status.connected_peers, 0);

        let peer = PeerInfo {
            id: "test-peer".to_string(),
            address: "127.0.0.1:30303".parse().unwrap(),
            public_key: None,
            node_type: NodeType::Full,
            version: "1.0.0".to_string(),
            capabilities: vec!["paradigm/1".to_string()],
            last_seen: SystemTime::now(),
            reputation: 100,
            connection_status: ConnectionStatus::Connected,
        };

        manager.update_peer(peer.clone()).await;
        let retrieved_peer = manager.get_peer("test-peer").await;
        assert!(retrieved_peer.is_some());
        assert_eq!(retrieved_peer.unwrap().id, peer.id);

        let connected_peers = manager.get_connected_peers().await;
        assert_eq!(connected_peers.len(), 1);
    }

    #[test]
    fn test_config_validation() {
        let valid_config = NetworkConfig::mainnet();
        assert!(utils::validate_config(&valid_config).is_ok());

        let mut invalid_config = valid_config.clone();
        invalid_config.rpc_endpoints.clear();
        assert!(utils::validate_config(&invalid_config).is_err());

        invalid_config.rpc_endpoints = valid_config.rpc_endpoints.clone();
        invalid_config.chain_id = 0;
        assert!(utils::validate_config(&invalid_config).is_err());
    }

    #[test]
    fn test_network_score_calculation() {
        let metrics = NetworkHealthMetrics {
            peer_connectivity: 80,
            average_latency_ms: 100,
            message_delivery_rate: 95,
            block_propagation_time_ms: 500,
            overall_health: 0,
        };

        let score = utils::calculate_network_score(&metrics);
        assert!(score > 0 && score <= 100);
    }

    #[test]
    fn test_peer_address_parsing() {
        let valid_addr = "192.168.1.1:30303";
        let parsed = utils::parse_peer_address(valid_addr);
        assert!(parsed.is_ok());

        let invalid_addr = "invalid-address";
        let parsed = utils::parse_peer_address(invalid_addr);
        assert!(parsed.is_err());
    }
}
