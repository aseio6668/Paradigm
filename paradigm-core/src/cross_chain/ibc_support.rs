use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::{DateTime, Utc};
use anyhow::{Result, anyhow};

use crate::{Hash, Amount, Address, transaction::Transaction};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum IBCPacketType {
    Transfer,
    Acknowledgement,
    Timeout,
    Error,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IBCChannel {
    pub id: String,
    pub port_id: String,
    pub counterparty_port_id: String,
    pub counterparty_channel_id: String,
    pub state: IBCChannelState,
    pub ordering: IBCOrdering,
    pub version: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum IBCChannelState {
    Init,
    TryOpen,
    Open,
    Closed,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum IBCOrdering {
    Ordered,
    Unordered,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IBCConnection {
    pub id: String,
    pub client_id: String,
    pub counterparty_client_id: String,
    pub state: IBCConnectionState,
    pub prefix: String,
    pub delay_period: u64,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum IBCConnectionState {
    Init,
    TryOpen,
    Open,
    Closed,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IBCClient {
    pub id: String,
    pub client_type: String,
    pub consensus_height: u64,
    pub frozen_height: Option<u64>,
    pub trust_level: f64,
    pub trusting_period: u64,
    pub unbonding_period: u64,
    pub max_clock_drift: u64,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IBCPacket {
    pub sequence: u64,
    pub source_port: String,
    pub source_channel: String,
    pub destination_port: String,
    pub destination_channel: String,
    pub data: Vec<u8>,
    pub timeout_height: u64,
    pub timeout_timestamp: u64,
    pub packet_type: IBCPacketType,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IBCTransferData {
    pub denom: String,
    pub amount: Amount,
    pub sender: Address,
    pub receiver: Address,
    pub memo: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IBCAcknowledgement {
    pub packet_sequence: u64,
    pub success: bool,
    pub error: Option<String>,
    pub result: Option<Vec<u8>>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProofData {
    pub key: Vec<u8>,
    pub value: Vec<u8>,
    pub proof: Vec<u8>,
    pub height: u64,
}

pub struct IBCModule {
    clients: Arc<RwLock<HashMap<String, IBCClient>>>,
    connections: Arc<RwLock<HashMap<String, IBCConnection>>>,
    channels: Arc<RwLock<HashMap<String, IBCChannel>>>,
    packets: Arc<RwLock<HashMap<u64, IBCPacket>>>,
    acknowledgements: Arc<RwLock<HashMap<u64, IBCAcknowledgement>>>,
    sequence_counter: Arc<RwLock<u64>>,
    port_bindings: Arc<RwLock<HashMap<String, String>>>,
}

impl IBCModule {
    pub fn new() -> Self {
        Self {
            clients: Arc::new(RwLock::new(HashMap::new())),
            connections: Arc::new(RwLock::new(HashMap::new())),
            channels: Arc::new(RwLock::new(HashMap::new())),
            packets: Arc::new(RwLock::new(HashMap::new())),
            acknowledgements: Arc::new(RwLock::new(HashMap::new())),
            sequence_counter: Arc::new(RwLock::new(1)),
            port_bindings: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub async fn create_client(
        &self,
        client_id: String,
        client_type: String,
        consensus_height: u64,
        trust_level: f64,
        trusting_period: u64,
        unbonding_period: u64,
        max_clock_drift: u64,
    ) -> Result<()> {
        let client = IBCClient {
            id: client_id.clone(),
            client_type,
            consensus_height,
            frozen_height: None,
            trust_level,
            trusting_period,
            unbonding_period,
            max_clock_drift,
            created_at: Utc::now(),
        };

        let mut clients = self.clients.write().await;
        clients.insert(client_id, client);
        
        tracing::info!("Created IBC client: {}", client.id);
        Ok(())
    }

    pub async fn create_connection(
        &self,
        connection_id: String,
        client_id: String,
        counterparty_client_id: String,
        prefix: String,
        delay_period: u64,
    ) -> Result<()> {
        let clients = self.clients.read().await;
        if !clients.contains_key(&client_id) {
            return Err(anyhow!("Client {} not found", client_id));
        }

        let connection = IBCConnection {
            id: connection_id.clone(),
            client_id,
            counterparty_client_id,
            state: IBCConnectionState::Init,
            prefix,
            delay_period,
            created_at: Utc::now(),
        };

        let mut connections = self.connections.write().await;
        connections.insert(connection_id, connection);
        
        tracing::info!("Created IBC connection: {}", connection.id);
        Ok(())
    }

    pub async fn create_channel(
        &self,
        channel_id: String,
        port_id: String,
        counterparty_port_id: String,
        counterparty_channel_id: String,
        ordering: IBCOrdering,
        version: String,
    ) -> Result<()> {
        let channel = IBCChannel {
            id: channel_id.clone(),
            port_id: port_id.clone(),
            counterparty_port_id,
            counterparty_channel_id,
            state: IBCChannelState::Init,
            ordering,
            version,
            created_at: Utc::now(),
        };

        let mut channels = self.channels.write().await;
        channels.insert(channel_id.clone(), channel);

        let mut port_bindings = self.port_bindings.write().await;
        port_bindings.insert(port_id, channel_id);
        
        tracing::info!("Created IBC channel: {}", channel.id);
        Ok(())
    }

    pub async fn send_packet(
        &self,
        source_port: String,
        source_channel: String,
        destination_port: String,
        destination_channel: String,
        data: Vec<u8>,
        timeout_height: u64,
        timeout_timestamp: u64,
        packet_type: IBCPacketType,
    ) -> Result<u64> {
        let mut sequence_counter = self.sequence_counter.write().await;
        let sequence = *sequence_counter;
        *sequence_counter += 1;

        let packet = IBCPacket {
            sequence,
            source_port,
            source_channel,
            destination_port,
            destination_channel,
            data,
            timeout_height,
            timeout_timestamp,
            packet_type,
            created_at: Utc::now(),
        };

        let mut packets = self.packets.write().await;
        packets.insert(sequence, packet);
        
        tracing::info!("Sent IBC packet with sequence: {}", sequence);
        Ok(sequence)
    }

    pub async fn receive_packet(&self, packet: IBCPacket) -> Result<()> {
        let channels = self.channels.read().await;
        let channel = channels.get(&packet.destination_channel)
            .ok_or_else(|| anyhow!("Channel {} not found", packet.destination_channel))?;

        if channel.state != IBCChannelState::Open {
            return Err(anyhow!("Channel {} is not open", packet.destination_channel));
        }

        let mut packets = self.packets.write().await;
        packets.insert(packet.sequence, packet.clone());

        self.process_packet_data(&packet).await?;
        
        tracing::info!("Received IBC packet with sequence: {}", packet.sequence);
        Ok(())
    }

    async fn process_packet_data(&self, packet: &IBCPacket) -> Result<()> {
        match packet.packet_type {
            IBCPacketType::Transfer => {
                let transfer_data: IBCTransferData = serde_json::from_slice(&packet.data)?;
                self.handle_transfer(transfer_data).await?;
            }
            IBCPacketType::Acknowledgement => {
                let ack: IBCAcknowledgement = serde_json::from_slice(&packet.data)?;
                self.handle_acknowledgement(ack).await?;
            }
            IBCPacketType::Timeout => {
                self.handle_timeout(packet.sequence).await?;
            }
            IBCPacketType::Error => {
                tracing::error!("Received error packet: {}", packet.sequence);
            }
        }
        Ok(())
    }

    async fn handle_transfer(&self, transfer: IBCTransferData) -> Result<()> {
        tracing::info!(
            "Processing IBC transfer: {} {} from {} to {}",
            transfer.amount,
            transfer.denom,
            transfer.sender,
            transfer.receiver
        );
        Ok(())
    }

    async fn handle_acknowledgement(&self, ack: IBCAcknowledgement) -> Result<()> {
        let mut acknowledgements = self.acknowledgements.write().await;
        acknowledgements.insert(ack.packet_sequence, ack.clone());
        
        tracing::info!("Processed acknowledgement for packet: {}", ack.packet_sequence);
        Ok(())
    }

    async fn handle_timeout(&self, sequence: u64) -> Result<()> {
        let mut packets = self.packets.write().await;
        if let Some(packet) = packets.remove(&sequence) {
            tracing::warn!("Packet {} timed out", sequence);
        }
        Ok(())
    }

    pub async fn acknowledge_packet(
        &self,
        packet_sequence: u64,
        success: bool,
        error: Option<String>,
        result: Option<Vec<u8>>,
    ) -> Result<()> {
        let ack = IBCAcknowledgement {
            packet_sequence,
            success,
            error,
            result,
            created_at: Utc::now(),
        };

        let mut acknowledgements = self.acknowledgements.write().await;
        acknowledgements.insert(packet_sequence, ack);
        
        tracing::info!("Created acknowledgement for packet: {}", packet_sequence);
        Ok(())
    }

    pub async fn verify_proof(&self, proof: &ProofData) -> Result<bool> {
        Ok(true)
    }

    pub async fn get_client(&self, client_id: &str) -> Option<IBCClient> {
        let clients = self.clients.read().await;
        clients.get(client_id).cloned()
    }

    pub async fn get_connection(&self, connection_id: &str) -> Option<IBCConnection> {
        let connections = self.connections.read().await;
        connections.get(connection_id).cloned()
    }

    pub async fn get_channel(&self, channel_id: &str) -> Option<IBCChannel> {
        let channels = self.channels.read().await;
        channels.get(channel_id).cloned()
    }

    pub async fn get_packet(&self, sequence: u64) -> Option<IBCPacket> {
        let packets = self.packets.read().await;
        packets.get(&sequence).cloned()
    }

    pub async fn list_channels(&self) -> Vec<IBCChannel> {
        let channels = self.channels.read().await;
        channels.values().cloned().collect()
    }

    pub async fn update_client_state(&self, client_id: &str, consensus_height: u64) -> Result<()> {
        let mut clients = self.clients.write().await;
        if let Some(client) = clients.get_mut(client_id) {
            client.consensus_height = consensus_height;
            tracing::info!("Updated client {} to height {}", client_id, consensus_height);
        }
        Ok(())
    }

    pub async fn freeze_client(&self, client_id: &str, height: u64) -> Result<()> {
        let mut clients = self.clients.write().await;
        if let Some(client) = clients.get_mut(client_id) {
            client.frozen_height = Some(height);
            tracing::warn!("Froze client {} at height {}", client_id, height);
        }
        Ok(())
    }

    pub async fn open_connection(&self, connection_id: &str) -> Result<()> {
        let mut connections = self.connections.write().await;
        if let Some(connection) = connections.get_mut(connection_id) {
            connection.state = IBCConnectionState::Open;
            tracing::info!("Opened IBC connection: {}", connection_id);
        }
        Ok(())
    }

    pub async fn open_channel(&self, channel_id: &str) -> Result<()> {
        let mut channels = self.channels.write().await;
        if let Some(channel) = channels.get_mut(channel_id) {
            channel.state = IBCChannelState::Open;
            tracing::info!("Opened IBC channel: {}", channel_id);
        }
        Ok(())
    }

    pub async fn close_channel(&self, channel_id: &str) -> Result<()> {
        let mut channels = self.channels.write().await;
        if let Some(channel) = channels.get_mut(channel_id) {
            channel.state = IBCChannelState::Closed;
            tracing::info!("Closed IBC channel: {}", channel_id);
        }
        Ok(())
    }

    pub async fn get_pending_packets(&self) -> Vec<IBCPacket> {
        let packets = self.packets.read().await;
        let acknowledgements = self.acknowledgements.read().await;
        
        packets.values()
            .filter(|packet| !acknowledgements.contains_key(&packet.sequence))
            .cloned()
            .collect()
    }

    pub async fn cleanup_timed_out_packets(&self, current_height: u64, current_timestamp: u64) -> Result<usize> {
        let mut packets = self.packets.write().await;
        let mut removed_count = 0;
        
        packets.retain(|_, packet| {
            let is_timed_out = packet.timeout_height > 0 && current_height >= packet.timeout_height ||
                              packet.timeout_timestamp > 0 && current_timestamp >= packet.timeout_timestamp;
            
            if is_timed_out {
                removed_count += 1;
                tracing::warn!("Removed timed out packet: {}", packet.sequence);
                false
            } else {
                true
            }
        });
        
        Ok(removed_count)
    }
}

impl Default for IBCModule {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_create_client() {
        let ibc = IBCModule::new();
        let result = ibc.create_client(
            "client-1".to_string(),
            "tendermint".to_string(),
            100,
            0.67,
            86400,
            172800,
            3000,
        ).await;
        assert!(result.is_ok());
        
        let client = ibc.get_client("client-1").await;
        assert!(client.is_some());
        assert_eq!(client.unwrap().consensus_height, 100);
    }

    #[tokio::test]
    async fn test_create_connection() {
        let ibc = IBCModule::new();
        
        ibc.create_client(
            "client-1".to_string(),
            "tendermint".to_string(),
            100,
            0.67,
            86400,
            172800,
            3000,
        ).await.unwrap();
        
        let result = ibc.create_connection(
            "connection-1".to_string(),
            "client-1".to_string(),
            "client-2".to_string(),
            "ibc".to_string(),
            0,
        ).await;
        assert!(result.is_ok());
        
        let connection = ibc.get_connection("connection-1").await;
        assert!(connection.is_some());
    }

    #[tokio::test]
    async fn test_create_channel() {
        let ibc = IBCModule::new();
        let result = ibc.create_channel(
            "channel-1".to_string(),
            "transfer".to_string(),
            "transfer".to_string(),
            "channel-2".to_string(),
            IBCOrdering::Unordered,
            "ics20-1".to_string(),
        ).await;
        assert!(result.is_ok());
        
        let channel = ibc.get_channel("channel-1").await;
        assert!(channel.is_some());
    }

    #[tokio::test]
    async fn test_send_packet() {
        let ibc = IBCModule::new();
        let data = b"test packet data".to_vec();
        
        let sequence = ibc.send_packet(
            "transfer".to_string(),
            "channel-1".to_string(),
            "transfer".to_string(),
            "channel-2".to_string(),
            data,
            1000,
            0,
            IBCPacketType::Transfer,
        ).await.unwrap();
        
        assert_eq!(sequence, 1);
        
        let packet = ibc.get_packet(sequence).await;
        assert!(packet.is_some());
    }

    #[tokio::test]
    async fn test_acknowledge_packet() {
        let ibc = IBCModule::new();
        let result = ibc.acknowledge_packet(1, true, None, None).await;
        assert!(result.is_ok());
    }
}