/// Bitcoin Lightning Network integration for Paradigm
use std::collections::{HashMap, VecDeque};
use std::sync::Arc;
use std::time::{Duration, Instant};
use anyhow::Result;
use serde::{Deserialize, Serialize};
use tokio::sync::{RwLock, mpsc};
use uuid::Uuid;
use sha2::{Sha256, Digest};

use super::{ChainId, CrossChainConfig};
use crate::{Hash, crypto_optimization::OptimizedSignatureEngine};

/// Lightning Network integration for Bitcoin cross-chain operations
#[derive(Debug)]
pub struct LightningIntegration {
    config: CrossChainConfig,
    channels: Arc<RwLock<HashMap<Uuid, LightningChannel>>>,
    payment_routes: Arc<RwLock<HashMap<Uuid, PaymentRoute>>>,
    invoices: Arc<RwLock<HashMap<String, LightningInvoice>>>,
    node_info: Arc<RwLock<LightningNodeInfo>>,
    signature_engine: Arc<OptimizedSignatureEngine>,
    stats: Arc<RwLock<LightningStats>>,
    event_processor: Arc<RwLock<LightningEventProcessor>>,
}

/// Lightning Network channel information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LightningChannel {
    pub channel_id: Uuid,
    pub short_channel_id: Option<String>,
    pub remote_node_id: String,
    pub capacity_sats: u64,
    pub local_balance_sats: u64,
    pub remote_balance_sats: u64,
    pub is_active: bool,
    pub is_public: bool,
    pub fee_rate_milli_msat: u64,
    pub min_htlc_msat: u64,
    pub max_htlc_msat: u64,
    pub channel_reserve_sats: u64,
    pub funding_tx_id: String,
    pub funding_output_index: u32,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub last_update: chrono::DateTime<chrono::Utc>,
    pub channel_state: ChannelState,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ChannelState {
    Pending,
    Active,
    Inactive,
    Closing,
    Closed,
    ForceClosing,
}

/// Payment route through Lightning Network
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaymentRoute {
    pub route_id: Uuid,
    pub source_node: String,
    pub destination_node: String,
    pub amount_msat: u64,
    pub fee_msat: u64,
    pub hops: Vec<RouteHop>,
    pub timeout_height: u64,
    pub payment_hash: Hash,
    pub payment_secret: Vec<u8>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub status: RouteStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RouteHop {
    pub node_id: String,
    pub channel_id: String,
    pub amount_msat: u64,
    pub fee_msat: u64,
    pub cltv_expiry_delta: u16,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RouteStatus {
    Created,
    InProgress,
    Succeeded,
    Failed,
    Timeout,
}

/// Lightning invoice for payments
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LightningInvoice {
    pub payment_request: String,
    pub payment_hash: Hash,
    pub amount_msat: Option<u64>,
    pub description: String,
    pub destination: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub expires_at: chrono::DateTime<chrono::Utc>,
    pub is_paid: bool,
    pub settled_at: Option<chrono::DateTime<chrono::Utc>>,
    pub memo: Option<String>,
}

/// Lightning node information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LightningNodeInfo {
    pub node_id: String,
    pub alias: String,
    pub color: String,
    pub version: String,
    pub num_channels: usize,
    pub num_peers: usize,
    pub total_capacity_sats: u64,
    pub local_balance_sats: u64,
    pub remote_balance_sats: u64,
    pub pending_balance_sats: u64,
    pub is_synced: bool,
    pub block_height: u64,
    pub network: String, // mainnet, testnet, regtest
}

/// Lightning Network event processor
#[derive(Debug)]
pub struct LightningEventProcessor {
    event_queue: VecDeque<LightningEvent>,
    processed_events: HashMap<String, chrono::DateTime<chrono::Utc>>,
    subscription_handlers: HashMap<LightningEventType, Vec<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LightningEvent {
    pub event_id: String,
    pub event_type: LightningEventType,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub data: LightningEventData,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LightningEventType {
    ChannelOpened,
    ChannelClosed,
    PaymentSent,
    PaymentReceived,
    InvoiceSettled,
    NodeConnected,
    NodeDisconnected,
    ForwardingEvent,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LightningEventData {
    ChannelEvent {
        channel_id: Uuid,
        node_id: String,
        capacity_sats: u64,
    },
    PaymentEvent {
        payment_hash: Hash,
        amount_msat: u64,
        fee_msat: u64,
        destination: String,
    },
    InvoiceEvent {
        payment_request: String,
        amount_msat: u64,
        description: String,
    },
    NodeEvent {
        node_id: String,
        address: String,
    },
    ForwardingEvent {
        incoming_channel: String,
        outgoing_channel: String,
        amount_msat: u64,
        fee_msat: u64,
    },
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct LightningStats {
    pub total_channels: usize,
    pub active_channels: usize,
    pub total_capacity_sats: u64,
    pub local_balance_sats: u64,
    pub remote_balance_sats: u64,
    pub total_payments_sent: u64,
    pub total_payments_received: u64,
    pub total_volume_sent_sats: u64,
    pub total_volume_received_sats: u64,
    pub total_fees_paid_sats: u64,
    pub total_fees_earned_sats: u64,
    pub average_payment_size_sats: u64,
    pub success_rate: f64,
    pub uptime_percentage: f64,
}

/// Lightning Network payment request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaymentRequest {
    pub destination: String,
    pub amount_sats: u64,
    pub description: String,
    pub timeout_seconds: u64,
    pub max_fee_sats: u64,
    pub final_cltv_delta: u16,
}

/// Lightning Network payment response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaymentResponse {
    pub payment_hash: Hash,
    pub payment_preimage: Option<Vec<u8>>,
    pub route: Option<PaymentRoute>,
    pub fee_paid_sats: u64,
    pub status: PaymentStatus,
    pub error_message: Option<String>,
    pub sent_at: chrono::DateTime<chrono::Utc>,
    pub settled_at: Option<chrono::DateTime<chrono::Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PaymentStatus {
    Pending,
    InFlight,
    Succeeded,
    Failed,
    Unknown,
}

impl LightningIntegration {
    pub async fn new(config: &CrossChainConfig) -> Result<Self> {
        let signature_engine = Arc::new(OptimizedSignatureEngine::new(2)?);
        
        Ok(Self {
            config: config.clone(),
            channels: Arc::new(RwLock::new(HashMap::new())),
            payment_routes: Arc::new(RwLock::new(HashMap::new())),
            invoices: Arc::new(RwLock::new(HashMap::new())),
            node_info: Arc::new(RwLock::new(LightningNodeInfo::default())),
            signature_engine,
            stats: Arc::new(RwLock::new(LightningStats::default())),
            event_processor: Arc::new(RwLock::new(LightningEventProcessor::new())),
        })
    }

    pub async fn initialize(&self) -> Result<()> {
        tracing::info!("Initializing Lightning Network integration...");

        // Initialize node information
        self.initialize_node_info().await?;

        // Start Lightning daemon connection
        self.connect_to_lightning_daemon().await?;

        // Start event monitoring
        self.start_event_monitoring().await?;

        // Start background tasks
        self.start_background_tasks().await?;

        tracing::info!("Lightning Network integration initialized successfully");
        Ok(())
    }

    /// Connect to Lightning Network daemon (LND, c-lightning, etc.)
    pub async fn connect_to_lightning_daemon(&self) -> Result<()> {
        tracing::info!("Connecting to Lightning Network daemon...");

        // Simulate connection to Lightning daemon
        tokio::time::sleep(Duration::from_millis(100)).await;

        // Update node info
        let mut node_info = self.node_info.write().await;
        node_info.node_id = "0302d48972ba7eef8b40696102ad114090fd4c146e381f18c7932a02d533b4bcbd".to_string();
        node_info.alias = "paradigm-lightning-node".to_string();
        node_info.version = "0.15.5-beta".to_string();
        node_info.network = "mainnet".to_string();
        node_info.is_synced = true;

        tracing::info!("Connected to Lightning daemon: {}", node_info.alias);
        Ok(())
    }

    /// Open a Lightning channel
    pub async fn open_channel(
        &self,
        remote_node_id: String,
        capacity_sats: u64,
        push_amount_sats: u64,
    ) -> Result<Uuid> {
        tracing::info!("Opening Lightning channel to {} with capacity {} sats", 
            remote_node_id, capacity_sats);

        let channel_id = Uuid::new_v4();
        
        // Simulate channel opening
        let funding_tx_id = self.simulate_funding_transaction(capacity_sats).await?;
        
        let channel = LightningChannel {
            channel_id,
            short_channel_id: None, // Will be set when channel is confirmed
            remote_node_id: remote_node_id.clone(),
            capacity_sats,
            local_balance_sats: capacity_sats - push_amount_sats,
            remote_balance_sats: push_amount_sats,
            is_active: false, // Will become active after confirmations
            is_public: true,
            fee_rate_milli_msat: 1000, // 1 msat per sat
            min_htlc_msat: 1000,
            max_htlc_msat: (capacity_sats * 1000) / 10, // 10% of capacity
            channel_reserve_sats: capacity_sats / 100, // 1% reserve
            funding_tx_id,
            funding_output_index: 0,
            created_at: chrono::Utc::now(),
            last_update: chrono::Utc::now(),
            channel_state: ChannelState::Pending,
        };

        // Store channel
        {
            let mut channels = self.channels.write().await;
            channels.insert(channel_id, channel);
        }

        // Emit event
        self.emit_event(LightningEvent {
            event_id: Uuid::new_v4().to_string(),
            event_type: LightningEventType::ChannelOpened,
            timestamp: chrono::Utc::now(),
            data: LightningEventData::ChannelEvent {
                channel_id,
                node_id: remote_node_id,
                capacity_sats,
            },
        }).await;

        tracing::info!("Lightning channel {} opened successfully", channel_id);
        Ok(channel_id)
    }

    /// Send Lightning payment
    pub async fn send_payment(&self, payment_request: PaymentRequest) -> Result<PaymentResponse> {
        tracing::info!("Sending Lightning payment of {} sats to {}", 
            payment_request.amount_sats, payment_request.destination);

        // Find route to destination
        let route = self.find_payment_route(&payment_request).await?;
        
        // Create payment hash
        let payment_hash = self.generate_payment_hash(&payment_request).await?;
        
        // Simulate payment execution
        let payment_response = self.execute_payment(payment_hash, route, &payment_request).await?;

        // Update stats
        {
            let mut stats = self.stats.write().await;
            stats.total_payments_sent += 1;
            stats.total_volume_sent_sats += payment_request.amount_sats;
            stats.total_fees_paid_sats += payment_response.fee_paid_sats;
        }

        // Emit event
        self.emit_event(LightningEvent {
            event_id: Uuid::new_v4().to_string(),
            event_type: LightningEventType::PaymentSent,
            timestamp: chrono::Utc::now(),
            data: LightningEventData::PaymentEvent {
                payment_hash,
                amount_msat: payment_request.amount_sats * 1000,
                fee_msat: payment_response.fee_paid_sats * 1000,
                destination: payment_request.destination.clone(),
            },
        }).await;

        tracing::info!("Lightning payment sent successfully: {:?}", payment_hash);
        Ok(payment_response)
    }

    /// Create Lightning invoice
    pub async fn create_invoice(
        &self,
        amount_sats: Option<u64>,
        description: String,
        expiry_seconds: u64,
    ) -> Result<LightningInvoice> {
        tracing::info!("Creating Lightning invoice for {} sats", 
            amount_sats.unwrap_or(0));

        let payment_hash = self.generate_random_hash().await;
        let payment_request = self.encode_payment_request(
            payment_hash,
            amount_sats,
            &description,
            expiry_seconds,
        ).await?;

        let invoice = LightningInvoice {
            payment_request: payment_request.clone(),
            payment_hash,
            amount_msat: amount_sats.map(|a| a * 1000),
            description: description.clone(),
            destination: {
                let node_info = self.node_info.read().await;
                node_info.node_id.clone()
            },
            created_at: chrono::Utc::now(),
            expires_at: chrono::Utc::now() + chrono::Duration::seconds(expiry_seconds as i64),
            is_paid: false,
            settled_at: None,
            memo: Some(description.clone()),
        };

        // Store invoice
        {
            let mut invoices = self.invoices.write().await;
            invoices.insert(payment_request.clone(), invoice.clone());
        }

        tracing::info!("Lightning invoice created: {}", payment_request);
        Ok(invoice)
    }

    /// Close Lightning channel
    pub async fn close_channel(&self, channel_id: Uuid, force_close: bool) -> Result<String> {
        tracing::info!("Closing Lightning channel: {} (force: {})", channel_id, force_close);

        let mut channels = self.channels.write().await;
        if let Some(channel) = channels.get_mut(&channel_id) {
            channel.channel_state = if force_close {
                ChannelState::ForceClosing
            } else {
                ChannelState::Closing
            };
            channel.last_update = chrono::Utc::now();

            // Simulate closing transaction
            let closing_tx_id = self.simulate_closing_transaction(channel).await?;

            // Emit event
            self.emit_event(LightningEvent {
                event_id: Uuid::new_v4().to_string(),
                event_type: LightningEventType::ChannelClosed,
                timestamp: chrono::Utc::now(),
                data: LightningEventData::ChannelEvent {
                    channel_id,
                    node_id: channel.remote_node_id.clone(),
                    capacity_sats: channel.capacity_sats,
                },
            }).await;

            tracing::info!("Lightning channel {} closed: {}", channel_id, closing_tx_id);
            Ok(closing_tx_id)
        } else {
            Err(anyhow::anyhow!("Channel not found: {}", channel_id))
        }
    }

    /// Get Lightning Network statistics
    pub async fn get_stats(&self) -> LightningStats {
        let mut stats = self.stats.read().await.clone();
        
        // Update real-time stats
        let channels = self.channels.read().await;
        stats.total_channels = channels.len();
        stats.active_channels = channels.values()
            .filter(|c| matches!(c.channel_state, ChannelState::Active))
            .count();
        
        stats.total_capacity_sats = channels.values()
            .map(|c| c.capacity_sats)
            .sum();
        
        stats.local_balance_sats = channels.values()
            .map(|c| c.local_balance_sats)
            .sum();
        
        stats.remote_balance_sats = channels.values()
            .map(|c| c.remote_balance_sats)
            .sum();

        if stats.total_payments_sent > 0 {
            stats.average_payment_size_sats = stats.total_volume_sent_sats / stats.total_payments_sent;
            stats.success_rate = 0.95; // Simulated success rate
        }

        stats.uptime_percentage = 99.5; // Simulated uptime

        stats
    }

    /// Get channel information
    pub async fn get_channels(&self) -> Vec<LightningChannel> {
        self.channels.read().await.values().cloned().collect()
    }

    /// Get node information
    pub async fn get_node_info(&self) -> LightningNodeInfo {
        self.node_info.read().await.clone()
    }

    /// Check if Lightning payment is possible
    pub async fn can_route_payment(&self, destination: &str, amount_sats: u64) -> Result<bool> {
        // Check if we have sufficient local balance
        let channels = self.channels.read().await;
        let total_local_balance: u64 = channels.values()
            .filter(|c| matches!(c.channel_state, ChannelState::Active))
            .map(|c| c.local_balance_sats)
            .sum();

        if total_local_balance < amount_sats {
            return Ok(false);
        }

        // Simulate route finding
        let route_exists = self.simulate_route_finding(destination, amount_sats).await?;
        Ok(route_exists)
    }

    // Private helper methods

    async fn initialize_node_info(&self) -> Result<()> {
        let node_info = LightningNodeInfo {
            node_id: "".to_string(), // Will be set when connecting to daemon
            alias: "paradigm-lightning-node".to_string(),
            color: "#FF6B35".to_string(),
            version: "0.15.5-beta".to_string(),
            num_channels: 0,
            num_peers: 0,
            total_capacity_sats: 0,
            local_balance_sats: 0,
            remote_balance_sats: 0,
            pending_balance_sats: 0,
            is_synced: false,
            block_height: 0,
            network: "mainnet".to_string(),
        };

        *self.node_info.write().await = node_info;
        Ok(())
    }

    async fn start_event_monitoring(&self) -> Result<()> {
        let event_processor = self.event_processor.clone();
        
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(Duration::from_secs(5));
            loop {
                interval.tick().await;
                
                let mut processor = event_processor.write().await;
                processor.process_events().await;
            }
        });

        Ok(())
    }

    async fn start_background_tasks(&self) -> Result<()> {
        // Channel state monitoring
        let channels = self.channels.clone();
        
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(Duration::from_secs(30));
            loop {
                interval.tick().await;
                
                // Simulate channel state updates
                let mut channels_guard = channels.write().await;
                for channel in channels_guard.values_mut() {
                    if matches!(channel.channel_state, ChannelState::Pending) {
                        // Simulate channel becoming active after some time
                        let age = chrono::Utc::now().signed_duration_since(channel.created_at);
                        if age > chrono::Duration::minutes(10) {
                            channel.channel_state = ChannelState::Active;
                            channel.is_active = true;
                            channel.short_channel_id = Some(format!("{}x{}x{}", 
                                700000, // Block height
                                channel.funding_output_index,
                                0 // Output index within the transaction
                            ));
                            tracing::info!("Channel {} became active", channel.channel_id);
                        }
                    }
                }
            }
        });

        Ok(())
    }

    async fn find_payment_route(&self, payment_request: &PaymentRequest) -> Result<PaymentRoute> {
        // Simulate route finding algorithm
        tokio::time::sleep(Duration::from_millis(100)).await;

        let channels = self.channels.read().await;
        let active_channels: Vec<_> = channels.values()
            .filter(|c| matches!(c.channel_state, ChannelState::Active))
            .collect();

        if active_channels.is_empty() {
            return Err(anyhow::anyhow!("No active channels available"));
        }

        // Use first active channel as simple routing
        let channel = active_channels[0];
        let route_hop = RouteHop {
            node_id: channel.remote_node_id.clone(),
            channel_id: channel.short_channel_id.clone().unwrap_or_default(),
            amount_msat: payment_request.amount_sats * 1000,
            fee_msat: channel.fee_rate_milli_msat,
            cltv_expiry_delta: 144, // ~24 hours
        };

        let payment_hash = self.generate_payment_hash(payment_request).await?;

        Ok(PaymentRoute {
            route_id: Uuid::new_v4(),
            source_node: {
                let node_info = self.node_info.read().await;
                node_info.node_id.clone()
            },
            destination_node: payment_request.destination.clone(),
            amount_msat: payment_request.amount_sats * 1000,
            fee_msat: channel.fee_rate_milli_msat,
            hops: vec![route_hop],
            timeout_height: 750000, // Current block height + delta
            payment_hash,
            payment_secret: vec![42; 32], // Random payment secret
            created_at: chrono::Utc::now(),
            status: RouteStatus::Created,
        })
    }

    async fn execute_payment(
        &self,
        payment_hash: Hash,
        route: PaymentRoute,
        payment_request: &PaymentRequest,
    ) -> Result<PaymentResponse> {
        // Simulate payment execution
        tokio::time::sleep(Duration::from_millis(500)).await;

        // Simulate successful payment (95% success rate)
        let success = rand::random::<f64>() < 0.95;

        if success {
            Ok(PaymentResponse {
                payment_hash,
                payment_preimage: Some(vec![42; 32]), // Random preimage
                route: Some(route),
                fee_paid_sats: payment_request.max_fee_sats.min(100), // Simulate reasonable fee
                status: PaymentStatus::Succeeded,
                error_message: None,
                sent_at: chrono::Utc::now(),
                settled_at: Some(chrono::Utc::now()),
            })
        } else {
            Ok(PaymentResponse {
                payment_hash,
                payment_preimage: None,
                route: Some(route),
                fee_paid_sats: 0,
                status: PaymentStatus::Failed,
                error_message: Some("Route not found".to_string()),
                sent_at: chrono::Utc::now(),
                settled_at: None,
            })
        }
    }

    async fn generate_payment_hash(&self, payment_request: &PaymentRequest) -> Result<Hash> {
        let mut hasher = Sha256::new();
        hasher.update(payment_request.destination.as_bytes());
        hasher.update(&payment_request.amount_sats.to_le_bytes());
        hasher.update(payment_request.description.as_bytes());
        hasher.update(&chrono::Utc::now().timestamp().to_le_bytes());
        
        let result = hasher.finalize();
        let mut hash = [0u8; 32];
        hash.copy_from_slice(&result);
        Ok(hash)
    }

    async fn generate_random_hash(&self) -> Hash {
        let mut hash = [0u8; 32];
        for i in 0..32 {
            hash[i] = rand::random::<u8>();
        }
        hash
    }

    async fn encode_payment_request(
        &self,
        payment_hash: Hash,
        amount_sats: Option<u64>,
        description: &str,
        expiry_seconds: u64,
    ) -> Result<String> {
        // Simulate BOLT11 invoice encoding
        let node_info = self.node_info.read().await;
        let invoice = format!("lnbc{}1pvjluezpp5{}{}{}{}",
            amount_sats.map(|a| a.to_string()).unwrap_or_default(),
            hex::encode(&payment_hash[..10]), // Shortened for example
            if description.is_empty() { "" } else { "d" },
            description.len(),
            node_info.node_id[..20].to_string() // Shortened node ID
        );
        Ok(invoice)
    }

    async fn simulate_funding_transaction(&self, capacity_sats: u64) -> Result<String> {
        // Simulate Bitcoin transaction creation for channel funding
        tokio::time::sleep(Duration::from_millis(200)).await;
        
        let tx_id = format!("{}:{}:funding", 
            hex::encode(&self.generate_random_hash().await[..16]),
            capacity_sats
        );
        Ok(tx_id)
    }

    async fn simulate_closing_transaction(&self, channel: &LightningChannel) -> Result<String> {
        // Simulate Bitcoin transaction creation for channel closing
        tokio::time::sleep(Duration::from_millis(200)).await;
        
        let tx_id = format!("{}:{}:closing", 
            channel.funding_tx_id,
            channel.capacity_sats
        );
        Ok(tx_id)
    }

    async fn simulate_route_finding(&self, _destination: &str, _amount_sats: u64) -> Result<bool> {
        // Simulate route finding with 90% success rate
        tokio::time::sleep(Duration::from_millis(50)).await;
        Ok(rand::random::<f64>() < 0.9)
    }

    async fn emit_event(&self, event: LightningEvent) {
        let mut processor = self.event_processor.write().await;
        processor.event_queue.push_back(event);
    }
}

impl LightningNodeInfo {
    fn default() -> Self {
        Self {
            node_id: "".to_string(),
            alias: "".to_string(),
            color: "#000000".to_string(),
            version: "".to_string(),
            num_channels: 0,
            num_peers: 0,
            total_capacity_sats: 0,
            local_balance_sats: 0,
            remote_balance_sats: 0,
            pending_balance_sats: 0,
            is_synced: false,
            block_height: 0,
            network: "mainnet".to_string(),
        }
    }
}

impl LightningEventProcessor {
    fn new() -> Self {
        Self {
            event_queue: VecDeque::new(),
            processed_events: HashMap::new(),
            subscription_handlers: HashMap::new(),
        }
    }

    async fn process_events(&mut self) {
        while let Some(event) = self.event_queue.pop_front() {
            match event.event_type {
                LightningEventType::ChannelOpened => {
                    tracing::info!("Processing channel opened event: {}", event.event_id);
                },
                LightningEventType::ChannelClosed => {
                    tracing::info!("Processing channel closed event: {}", event.event_id);
                },
                LightningEventType::PaymentSent => {
                    tracing::info!("Processing payment sent event: {}", event.event_id);
                },
                LightningEventType::PaymentReceived => {
                    tracing::info!("Processing payment received event: {}", event.event_id);
                },
                LightningEventType::InvoiceSettled => {
                    tracing::info!("Processing invoice settled event: {}", event.event_id);
                },
                LightningEventType::NodeConnected => {
                    tracing::info!("Processing node connected event: {}", event.event_id);
                },
                LightningEventType::NodeDisconnected => {
                    tracing::info!("Processing node disconnected event: {}", event.event_id);
                },
                LightningEventType::ForwardingEvent => {
                    tracing::debug!("Processing forwarding event: {}", event.event_id);
                },
            }
            
            self.processed_events.insert(event.event_id, chrono::Utc::now());
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_lightning_integration_creation() {
        let config = CrossChainConfig::default();
        let lightning = LightningIntegration::new(&config).await;
        assert!(lightning.is_ok());
    }

    #[tokio::test]
    async fn test_channel_operations() {
        let config = CrossChainConfig::default();
        let lightning = LightningIntegration::new(&config).await.unwrap();
        
        // Open channel
        let channel_id = lightning.open_channel(
            "0302d48972ba7eef8b40696102ad114090fd4c146e381f18c7932a02d533b4bcbd".to_string(),
            1000000, // 1M sats
            100000,  // 100K sats push
        ).await.unwrap();

        // Check channel exists
        let channels = lightning.get_channels().await;
        assert_eq!(channels.len(), 1);
        assert_eq!(channels[0].channel_id, channel_id);
        assert_eq!(channels[0].capacity_sats, 1000000);

        // Close channel
        let closing_tx = lightning.close_channel(channel_id, false).await;
        assert!(closing_tx.is_ok());
    }

    #[tokio::test]
    async fn test_invoice_creation() {
        let config = CrossChainConfig::default();
        let lightning = LightningIntegration::new(&config).await.unwrap();
        
        let invoice = lightning.create_invoice(
            Some(50000), // 50K sats
            "Test invoice".to_string(),
            3600, // 1 hour expiry
        ).await.unwrap();

        assert!(invoice.payment_request.starts_with("lnbc"));
        assert_eq!(invoice.amount_msat, Some(50000000)); // 50K sats in msat
        assert_eq!(invoice.description, "Test invoice");
    }

    #[tokio::test]
    async fn test_payment_routing() {
        let config = CrossChainConfig::default();
        let lightning = LightningIntegration::new(&config).await.unwrap();
        
        // First open a channel to enable routing
        let _channel_id = lightning.open_channel(
            "0302d48972ba7eef8b40696102ad114090fd4c146e381f18c7932a02d533b4bcbd".to_string(),
            1000000,
            0,
        ).await.unwrap();

        // Check if payment is possible
        let can_route = lightning.can_route_payment(
            "0302d48972ba7eef8b40696102ad114090fd4c146e381f18c7932a02d533b4bcbd",
            10000,
        ).await.unwrap();

        assert!(can_route);
    }
}