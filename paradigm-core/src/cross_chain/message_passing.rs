use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use tokio::sync::RwLock;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::{DateTime, Utc};
use anyhow::{Result, anyhow};
use blake3::Hasher;

use crate::{Hash, Address};
use super::{ChainId, CrossChainConfig};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MessageType {
    DataTransfer,
    FunctionCall {
        contract_address: String,
        function_signature: String,
        parameters: Vec<u8>,
    },
    Event {
        event_name: String,
        event_data: Vec<u8>,
    },
    Oracle {
        oracle_type: OracleType,
        query: String,
        response_callback: String,
    },
    Governance {
        proposal_id: Uuid,
        action: String,
    },
    Emergency {
        alert_type: String,
        severity: u8,
        action_required: bool,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum OracleType {
    PriceFeed,
    RandomNumber,
    ExternalAPI,
    WeatherData,
    SportData,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MessageStatus {
    Pending,
    Sent,
    Delivered,
    Acknowledged,
    Failed,
    Expired,
    Retrying,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DeliveryGuarantee {
    AtMostOnce,
    AtLeastOnce,
    ExactlyOnce,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CrossChainMessage {
    pub message_id: Uuid,
    pub from_chain: ChainId,
    pub to_chain: ChainId,
    pub sender: Address,
    pub recipient: Address,
    pub message_type: MessageType,
    pub payload: Vec<u8>,
    pub nonce: u64,
    pub timestamp: DateTime<Utc>,
    pub expiry: DateTime<Utc>,
    pub status: MessageStatus,
    pub delivery_guarantee: DeliveryGuarantee,
    pub retry_count: u32,
    pub max_retries: u32,
    pub gas_limit: Option<u64>,
    pub gas_price: Option<u64>,
    pub fee_paid: u64,
    pub acknowledgment_data: Option<Vec<u8>>,
    pub error_message: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessageRoute {
    pub route_id: Uuid,
    pub from_chain: ChainId,
    pub to_chain: ChainId,
    pub relay_chains: Vec<ChainId>,
    pub total_hops: u32,
    pub estimated_time_seconds: u64,
    pub total_fee: u64,
    pub reliability_score: f64,
    pub is_active: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessageRelay {
    pub relay_id: Uuid,
    pub relay_address: Address,
    pub supported_chains: Vec<ChainId>,
    pub fee_per_message: u64,
    pub success_rate: f64,
    pub average_delivery_time: Duration,
    pub stake_amount: u64,
    pub is_active: bool,
    pub reputation_score: f64,
    pub last_activity: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessageQueue {
    pub queue_id: Uuid,
    pub chain_id: ChainId,
    pub messages: Vec<Uuid>,
    pub max_size: usize,
    pub processing_rate: f64,
    pub priority_levels: HashMap<MessagePriority, Vec<Uuid>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Hash, Eq, PartialEq)]
pub enum MessagePriority {
    Low,
    Normal,
    High,
    Emergency,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessageReceipt {
    pub receipt_id: Uuid,
    pub message_id: Uuid,
    pub recipient_signature: Vec<u8>,
    pub execution_result: Option<Vec<u8>>,
    pub gas_used: Option<u64>,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessageFilter {
    pub filter_id: Uuid,
    pub chain_id: ChainId,
    pub sender_filter: Option<Address>,
    pub recipient_filter: Option<Address>,
    pub message_type_filter: Option<MessageType>,
    pub payload_pattern: Option<String>,
    pub action: FilterAction,
    pub is_active: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FilterAction {
    Allow,
    Block,
    Quarantine,
    RateLimit(u32), // messages per minute
}

pub struct MessagePassingProtocol {
    messages: Arc<RwLock<HashMap<Uuid, CrossChainMessage>>>,
    message_queues: Arc<RwLock<HashMap<ChainId, MessageQueue>>>,
    relays: Arc<RwLock<HashMap<Uuid, MessageRelay>>>,
    routes: Arc<RwLock<HashMap<(ChainId, ChainId), Vec<MessageRoute>>>>,
    receipts: Arc<RwLock<HashMap<Uuid, MessageReceipt>>>,
    filters: Arc<RwLock<HashMap<ChainId, Vec<MessageFilter>>>>,
    nonce_counters: Arc<RwLock<HashMap<Address, u64>>>,
    config: CrossChainConfig,
    protocol_stats: Arc<RwLock<ProtocolStats>>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ProtocolStats {
    pub total_messages_sent: u64,
    pub total_messages_delivered: u64,
    pub total_messages_failed: u64,
    pub average_delivery_time_ms: u64,
    pub active_relays: usize,
    pub active_routes: usize,
    pub pending_messages: usize,
}

impl MessagePassingProtocol {
    pub async fn new(config: &CrossChainConfig) -> Result<Self> {
        Ok(Self {
            messages: Arc::new(RwLock::new(HashMap::new())),
            message_queues: Arc::new(RwLock::new(HashMap::new())),
            relays: Arc::new(RwLock::new(HashMap::new())),
            routes: Arc::new(RwLock::new(HashMap::new())),
            receipts: Arc::new(RwLock::new(HashMap::new())),
            filters: Arc::new(RwLock::new(HashMap::new())),
            nonce_counters: Arc::new(RwLock::new(HashMap::new())),
            config: config.clone(),
            protocol_stats: Arc::new(RwLock::new(ProtocolStats::default())),
        })
    }

    pub async fn initialize(&self) -> Result<()> {
        tracing::info!("Initializing Cross-Chain Message Passing Protocol...");
        
        // Initialize message queues for each supported chain
        for &chain_id in &self.config.enabled_chains {
            self.create_message_queue(chain_id).await?;
        }
        
        // Start monitoring and processing tasks
        self.start_processing_tasks().await?;
        
        tracing::info!("Cross-Chain Message Passing Protocol initialized successfully");
        Ok(())
    }

    pub async fn send_message(
        &self,
        from_chain: ChainId,
        to_chain: ChainId,
        sender: Address,
        recipient: Address,
        message_type: MessageType,
        payload: Vec<u8>,
        delivery_guarantee: DeliveryGuarantee,
        expiry_hours: u64,
        gas_limit: Option<u64>,
        gas_price: Option<u64>,
    ) -> Result<Uuid> {
        // Validate chains are supported
        if !self.config.enabled_chains.contains(&from_chain) || 
           !self.config.enabled_chains.contains(&to_chain) {
            return Err(anyhow!("Unsupported chain"));
        }
        
        // Check message filters
        if !self.passes_filters(&from_chain, &to_chain, &sender, &recipient, &message_type).await? {
            return Err(anyhow!("Message blocked by filter"));
        }
        
        let message_id = Uuid::new_v4();
        let nonce = self.get_next_nonce(&sender).await?;
        let now = Utc::now();
        let expiry = now + chrono::Duration::hours(expiry_hours as i64);
        
        // Calculate fee
        let fee = self.calculate_message_fee(&from_chain, &to_chain, &payload, gas_limit).await?;
        
        let message = CrossChainMessage {
            message_id,
            from_chain,
            to_chain,
            sender,
            recipient,
            message_type,
            payload,
            nonce,
            timestamp: now,
            expiry,
            status: MessageStatus::Pending,
            delivery_guarantee,
            retry_count: 0,
            max_retries: match delivery_guarantee {
                DeliveryGuarantee::AtMostOnce => 0,
                DeliveryGuarantee::AtLeastOnce => 3,
                DeliveryGuarantee::ExactlyOnce => 5,
            },
            gas_limit,
            gas_price,
            fee_paid: fee,
            acknowledgment_data: None,
            error_message: None,
            created_at: now,
            updated_at: now,
        };
        
        // Store message
        {
            let mut messages = self.messages.write().await;
            messages.insert(message_id, message.clone());
        }
        
        // Add to queue
        self.enqueue_message(from_chain, message_id, MessagePriority::Normal).await?;
        
        // Update stats
        {
            let mut stats = self.protocol_stats.write().await;
            stats.total_messages_sent += 1;
            stats.pending_messages += 1;
        }
        
        tracing::info!("Sent cross-chain message: {} from {:?} to {:?}", message_id, from_chain, to_chain);
        Ok(message_id)
    }

    pub async fn register_relay(
        &self,
        relay_address: Address,
        supported_chains: Vec<ChainId>,
        fee_per_message: u64,
        stake_amount: u64,
    ) -> Result<Uuid> {
        let relay_id = Uuid::new_v4();
        
        let relay = MessageRelay {
            relay_id,
            relay_address,
            supported_chains,
            fee_per_message,
            success_rate: 0.95, // Start with good rating
            average_delivery_time: Duration::from_secs(30),
            stake_amount,
            is_active: true,
            reputation_score: 1.0,
            last_activity: Utc::now(),
        };
        
        let mut relays = self.relays.write().await;
        relays.insert(relay_id, relay);
        
        tracing::info!("Registered message relay: {}", relay_id);
        Ok(relay_id)
    }

    pub async fn create_route(
        &self,
        from_chain: ChainId,
        to_chain: ChainId,
        relay_chains: Vec<ChainId>,
        estimated_time_seconds: u64,
        total_fee: u64,
    ) -> Result<Uuid> {
        let route_id = Uuid::new_v4();
        
        let route = MessageRoute {
            route_id,
            from_chain,
            to_chain,
            relay_chains: relay_chains.clone(),
            total_hops: relay_chains.len() as u32 + 1,
            estimated_time_seconds,
            total_fee,
            reliability_score: 0.9, // Start with good score
            is_active: true,
        };
        
        let mut routes = self.routes.write().await;
        routes.entry((from_chain, to_chain))
            .or_insert_with(Vec::new)
            .push(route);
        
        tracing::info!("Created message route: {} ({:?} -> {:?})", route_id, from_chain, to_chain);
        Ok(route_id)
    }

    pub async fn acknowledge_message(
        &self,
        message_id: Uuid,
        recipient_signature: Vec<u8>,
        execution_result: Option<Vec<u8>>,
        gas_used: Option<u64>,
    ) -> Result<()> {
        let receipt_id = Uuid::new_v4();
        
        let receipt = MessageReceipt {
            receipt_id,
            message_id,
            recipient_signature,
            execution_result,
            gas_used,
            timestamp: Utc::now(),
        };
        
        // Store receipt
        {
            let mut receipts = self.receipts.write().await;
            receipts.insert(receipt_id, receipt);
        }
        
        // Update message status
        {
            let mut messages = self.messages.write().await;
            if let Some(message) = messages.get_mut(&message_id) {
                message.status = MessageStatus::Acknowledged;
                message.updated_at = Utc::now();
            }
        }
        
        tracing::info!("Acknowledged message: {}", message_id);
        Ok(())
    }

    pub async fn get_message(&self, message_id: &Uuid) -> Option<CrossChainMessage> {
        let messages = self.messages.read().await;
        messages.get(message_id).cloned()
    }

    pub async fn get_messages_for_chain(&self, chain_id: ChainId) -> Vec<CrossChainMessage> {
        let messages = self.messages.read().await;
        messages.values()
            .filter(|m| m.to_chain == chain_id && m.status == MessageStatus::Delivered)
            .cloned()
            .collect()
    }

    pub async fn get_pending_messages(&self) -> Vec<CrossChainMessage> {
        let messages = self.messages.read().await;
        messages.values()
            .filter(|m| matches!(m.status, MessageStatus::Pending | MessageStatus::Retrying))
            .cloned()
            .collect()
    }

    pub async fn get_protocol_stats(&self) -> ProtocolStats {
        let mut stats = self.protocol_stats.read().await.clone();
        
        // Update real-time stats
        stats.active_relays = self.relays.read().await.len();
        stats.active_routes = self.routes.read().await.values().map(|v| v.len()).sum();
        stats.pending_messages = self.messages.read().await.values()
            .filter(|m| matches!(m.status, MessageStatus::Pending | MessageStatus::Retrying))
            .count();
        
        stats
    }

    pub async fn handle_message(
        &self,
        transaction_id: Uuid,
        from_chain: ChainId,
        to_chain: ChainId,
    ) -> Result<()> {
        tracing::info!("Handling cross-chain message: {} from {:?} to {:?}", transaction_id, from_chain, to_chain);
        
        // This would implement the actual message handling logic
        // For now, we'll simulate successful handling
        tokio::time::sleep(Duration::from_millis(100)).await;
        
        Ok(())
    }

    // Private methods

    async fn create_message_queue(&self, chain_id: ChainId) -> Result<()> {
        let queue_id = Uuid::new_v4();
        let mut priority_levels = HashMap::new();
        priority_levels.insert(MessagePriority::Low, Vec::new());
        priority_levels.insert(MessagePriority::Normal, Vec::new());
        priority_levels.insert(MessagePriority::High, Vec::new());
        priority_levels.insert(MessagePriority::Emergency, Vec::new());
        
        let queue = MessageQueue {
            queue_id,
            chain_id,
            messages: Vec::new(),
            max_size: 10000,
            processing_rate: 100.0, // messages per second
            priority_levels,
        };
        
        let mut queues = self.message_queues.write().await;
        queues.insert(chain_id, queue);
        
        Ok(())
    }

    async fn enqueue_message(
        &self,
        chain_id: ChainId,
        message_id: Uuid,
        priority: MessagePriority,
    ) -> Result<()> {
        let mut queues = self.message_queues.write().await;
        if let Some(queue) = queues.get_mut(&chain_id) {
            if queue.messages.len() >= queue.max_size {
                return Err(anyhow!("Message queue is full"));
            }
            
            queue.messages.push(message_id);
            queue.priority_levels.entry(priority)
                .or_insert_with(Vec::new)
                .push(message_id);
        }
        
        Ok(())
    }

    async fn passes_filters(
        &self,
        from_chain: &ChainId,
        to_chain: &ChainId,
        sender: &Address,
        recipient: &Address,
        message_type: &MessageType,
    ) -> Result<bool> {
        let filters = self.filters.read().await;
        
        // Check filters for destination chain
        if let Some(chain_filters) = filters.get(to_chain) {
            for filter in chain_filters {
                if !filter.is_active {
                    continue;
                }
                
                let mut matches = true;
                
                if let Some(sender_filter) = &filter.sender_filter {
                    if sender_filter != sender {
                        matches = false;
                    }
                }
                
                if let Some(recipient_filter) = &filter.recipient_filter {
                    if recipient_filter != recipient {
                        matches = false;
                    }
                }
                
                if matches {
                    match &filter.action {
                        FilterAction::Allow => continue,
                        FilterAction::Block => return Ok(false),
                        FilterAction::Quarantine => {
                            tracing::warn!("Message quarantined by filter: {}", filter.filter_id);
                            return Ok(false);
                        },
                        FilterAction::RateLimit(_limit) => {
                            // TODO: Implement rate limiting logic
                            continue;
                        },
                    }
                }
            }
        }
        
        Ok(true)
    }

    async fn get_next_nonce(&self, sender: &Address) -> Result<u64> {
        let mut nonce_counters = self.nonce_counters.write().await;
        let nonce = nonce_counters.entry(*sender).or_insert(0);
        *nonce += 1;
        Ok(*nonce)
    }

    async fn calculate_message_fee(
        &self,
        from_chain: &ChainId,
        to_chain: &ChainId,
        payload: &[u8],
        gas_limit: Option<u64>,
    ) -> Result<u64> {
        let base_fee = 1000; // Base fee in smallest unit
        let size_fee = payload.len() as u64 * 10; // Per byte fee
        let gas_fee = gas_limit.unwrap_or(21000) / 1000; // Gas cost
        
        // Chain-specific multipliers
        let chain_multiplier = match to_chain {
            ChainId::Ethereum => 5,
            ChainId::Bitcoin => 3,
            ChainId::Cosmos => 2,
            _ => 1,
        };
        
        Ok((base_fee + size_fee + gas_fee) * chain_multiplier)
    }

    async fn start_processing_tasks(&self) -> Result<()> {
        // Message processing task
        let messages = self.messages.clone();
        let queues = self.message_queues.clone();
        let stats = self.protocol_stats.clone();
        
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(Duration::from_secs(1));
            loop {
                interval.tick().await;
                
                // Process messages from queues
                let queue_map = queues.read().await.clone();
                for (chain_id, queue) in queue_map {
                    // Process emergency messages first
                    if let Some(emergency_messages) = queue.priority_levels.get(&MessagePriority::Emergency) {
                        for &message_id in emergency_messages.iter().take(5) { // Process up to 5 per cycle
                            if let Some(mut message) = {
                                let messages_read = messages.read().await;
                                messages_read.get(&message_id).cloned()
                            } {
                                if message.status == MessageStatus::Pending {
                                    message.status = MessageStatus::Sent;
                                    message.updated_at = Utc::now();
                                    
                                    let mut messages_write = messages.write().await;
                                    messages_write.insert(message_id, message);
                                    
                                    tracing::info!("Processed emergency message: {}", message_id);
                                }
                            }
                        }
                    }
                }
                
                // Update delivery stats
                tokio::time::sleep(Duration::from_millis(100)).await;
            }
        });
        
        // Message timeout task
        let messages_timeout = self.messages.clone();
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(Duration::from_secs(60));
            loop {
                interval.tick().await;
                
                let now = Utc::now();
                let mut expired_messages = Vec::new();
                
                {
                    let messages = messages_timeout.read().await;
                    for (message_id, message) in messages.iter() {
                        if now > message.expiry && !matches!(message.status, MessageStatus::Delivered | MessageStatus::Acknowledged) {
                            expired_messages.push(*message_id);
                        }
                    }
                }
                
                {
                    let mut messages = messages_timeout.write().await;
                    for message_id in expired_messages {
                        if let Some(message) = messages.get_mut(&message_id) {
                            message.status = MessageStatus::Expired;
                            message.updated_at = now;
                            tracing::warn!("Message expired: {}", message_id);
                        }
                    }
                }
            }
        });
        
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_message_protocol_creation() {
        let config = CrossChainConfig::default();
        let protocol = MessagePassingProtocol::new(&config).await;
        assert!(protocol.is_ok());
    }
    
    #[tokio::test]
    async fn test_send_message() {
        let config = CrossChainConfig::default();
        let protocol = MessagePassingProtocol::new(&config).await.unwrap();
        protocol.initialize().await.unwrap();
        
        let sender = Address([1u8; 32]);
        let recipient = Address([2u8; 32]);
        let payload = b"Hello, cross-chain world!".to_vec();
        
        let message_id = protocol.send_message(
            ChainId::Paradigm,
            ChainId::Ethereum,
            sender,
            recipient,
            MessageType::DataTransfer,
            payload,
            DeliveryGuarantee::AtLeastOnce,
            24, // 24 hours expiry
            Some(50000),
            Some(20),
        ).await.unwrap();
        
        let message = protocol.get_message(&message_id).await;
        assert!(message.is_some());
        assert_eq!(message.unwrap().from_chain, ChainId::Paradigm);
    }
    
    #[tokio::test]
    async fn test_relay_registration() {
        let config = CrossChainConfig::default();
        let protocol = MessagePassingProtocol::new(&config).await.unwrap();
        
        let relay_address = Address([1u8; 32]);
        let supported_chains = vec![ChainId::Paradigm, ChainId::Ethereum];
        
        let relay_id = protocol.register_relay(
            relay_address,
            supported_chains,
            1000, // fee per message
            100000, // stake amount
        ).await.unwrap();
        
        // Test that relay was registered
        let relays = protocol.relays.read().await;
        assert!(relays.contains_key(&relay_id));
    }
}