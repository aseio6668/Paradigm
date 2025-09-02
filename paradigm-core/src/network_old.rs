use std::collections::HashMap#[derive(libp2p::swarm::NetworkBehaviour)]
pub struct P2PBehaviour {
    pub gossipsub: gossipsub::Behaviour,
    pub mdns: mdns::async_io::Behaviour,
    pub kademlia: Kademlia<MemoryStore>,
}std::sync::Arc;
use tokio::sync::RwLock;
use libp2p::{
    gossipsub, mdns, noise,
    swarm::{NetworkBehaviour, Swarm, SwarmEvent},
    tcp, yamux, PeerId, Transport,
    kad::{Kademlia, KademliaEvent, store::MemoryStore},
    identity::Keypair as LibP2PKeypair,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use anyhow::Result;

use crate::transaction::Transaction;
use crate::ml_tasks::{MLTask, MLTaskResult};

/// Network message types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum NetworkMessage {
    Transaction(Transaction),
    MLTask(MLTask),
    MLTaskResult { task_id: Uuid, result: Vec<u8> },
    PeerDiscovery { peer_id: String, address: String },
    NetworkSync { timestamp: i64, data_hash: Vec<u8> },
    Heartbeat { peer_id: String, timestamp: i64 },
}

/// P2P Network behavior combining Gossipsub, mDNS, and Kademlia
#[derive(libp2p::swarm::NetworkBehaviour)]
pub struct P2PBehaviour {
    pub gossipsub: gossipsub::Behaviour,
    pub mdns: mdns::async_io::Behaviour,
    pub kademlia: Kademlia<MemoryStore>,
}

#[derive(Debug)]
pub enum P2PEvent {
    Gossipsub(gossipsub::Event),
    Mdns(mdns::Event),
    Kademlia(KademliaEvent),
}

impl From<gossipsub::Event> for P2PEvent {
    fn from(event: gossipsub::Event) -> Self {
        P2PEvent::Gossipsub(event)
    }
}

impl From<mdns::Event> for P2PEvent {
    fn from(event: mdns::Event) -> Self {
        P2PEvent::Mdns(event)
    }
}

impl From<KademliaEvent> for P2PEvent {
    fn from(event: KademliaEvent) -> Self {
        P2PEvent::Kademlia(event)
    }
}

/// Main P2P network structure
pub struct P2PNetwork {
    swarm: Swarm<P2PBehaviour>,
    node_id: Uuid,
    connected_peers: HashMap<PeerId, String>,
    transaction_topic: gossipsub::IdentTopic,
    ml_task_topic: gossipsub::IdentTopic,
    sync_topic: gossipsub::IdentTopic,
}

impl P2PNetwork {
    /// Create a new P2P network instance
    pub async fn new(node_id: Uuid) -> Result<Self> {
        // Generate a random keypair for libp2p
        let local_key = LibP2PSigningKey::generate_ed25519();
        let local_peer_id = PeerId::from(local_key.public());

        tracing::info!("Local peer id: {}", local_peer_id);

        // Create transport
        let transport = tcp::async_io::Transport::default()
            .upgrade(yamux::Config::default())
            .multiplex(yamux::Config::default())
            .authenticate(noise::Config::new(&local_key)?)
            .boxed();

        // Create gossipsub configuration
        let gossipsub_config = gossipsub::ConfigBuilder::default()
            .heartbeat_interval(std::time::Duration::from_secs(10))
            .validation_mode(gossipsub::ValidationMode::Strict)
            .build()
            .expect("Valid gossipsub config");

        // Create gossipsub behavior
        let mut gossipsub = gossipsub::Behaviour::new(
            gossipsub::MessageAuthenticity::Signed(local_key.clone()),
            gossipsub_config,
        )?;

        // Create topics
        let transaction_topic = gossipsub::IdentTopic::new("paradigm-transactions");
        let ml_task_topic = gossipsub::IdentTopic::new("paradigm-ml-tasks");
        let sync_topic = gossipsub::IdentTopic::new("paradigm-sync");

        // Subscribe to topics
        gossipsub.subscribe(&transaction_topic)?;
        gossipsub.subscribe(&ml_task_topic)?;
        gossipsub.subscribe(&sync_topic)?;

        // Create mDNS behavior
        let mdns = mdns::async_io::Behaviour::new(mdns::Config::default(), local_peer_id)?;

        // Create Kademlia behavior
        let store = libp2p::kad::store::MemoryStore::new(local_peer_id);
        let kademlia = Kademlia::new(local_peer_id, store);

        // Create combined behavior
        let behaviour = P2PBehaviour {
            gossipsub,
            mdns,
            kademlia,
        };

        // Create swarm
        let swarm = Swarm::with_async_std_executor(transport, behaviour, local_peer_id);

        Ok(P2PNetwork {
            swarm,
            node_id,
            connected_peers: HashMap::new(),
            transaction_topic,
            ml_task_topic,
            sync_topic,
        })
    }

    /// Start the P2P network
    pub async fn start(&mut self) -> Result<()> {
        // Listen on all interfaces
        self.swarm.listen_on("/ip4/0.0.0.0/tcp/0".parse()?)?;

        tracing::info!("P2P network started for node {}", self.node_id);
        
        // Start the event loop
        tokio::spawn(async move {
            // This would be the main event loop, but for now we'll just log
            tracing::info!("P2P event loop started");
        });

        Ok(())
    }

    /// Broadcast a transaction to the network
    pub async fn broadcast_transaction(&self, transaction: &Transaction) -> Result<()> {
        let message = NetworkMessage::Transaction(transaction.clone());
        let serialized = serde_json::to_vec(&message)?;
        
        if let Err(e) = self.swarm.behaviour().gossipsub.publish(self.transaction_topic.clone(), serialized) {
            tracing::error!("Failed to broadcast transaction: {}", e);
            return Err(anyhow::anyhow!("Failed to broadcast transaction"));
        }

        tracing::debug!("Broadcasted transaction {}", transaction.id);
        Ok(())
    }

    /// Broadcast an ML task to the network
    pub async fn broadcast_ml_task(&self, task: &MLTask) -> Result<()> {
        let message = NetworkMessage::MLTask(task.clone());
        let serialized = serde_json::to_vec(&message)?;
        
        if let Err(e) = self.swarm.behaviour().gossipsub.publish(self.ml_task_topic.clone(), serialized) {
            tracing::error!("Failed to broadcast ML task: {}", e);
            return Err(anyhow::anyhow!("Failed to broadcast ML task"));
        }

        tracing::debug!("Broadcasted ML task {}", task.id);
        Ok(())
    }

    /// Broadcast ML task result
    pub async fn broadcast_ml_result(&self, task_id: Uuid, result: Vec<u8>) -> Result<()> {
        let message = NetworkMessage::MLTaskResult { task_id, result };
        let serialized = serde_json::to_vec(&message)?;
        
        if let Err(e) = self.swarm.behaviour().gossipsub.publish(self.ml_task_topic.clone(), serialized) {
            tracing::error!("Failed to broadcast ML result: {}", e);
            return Err(anyhow::anyhow!("Failed to broadcast ML result"));
        }

        tracing::debug!("Broadcasted ML task result for {}", task_id);
        Ok(())
    }

    /// Sync with network using timestamp-based chunks
    pub async fn sync_with_network(&self, timestamp: i64, data_hash: Vec<u8>) -> Result<()> {
        let message = NetworkMessage::NetworkSync { timestamp, data_hash };
        let serialized = serde_json::to_vec(&message)?;
        
        if let Err(e) = self.swarm.behaviour().gossipsub.publish(self.sync_topic.clone(), serialized) {
            tracing::error!("Failed to sync with network: {}", e);
            return Err(anyhow::anyhow!("Failed to sync with network"));
        }

        tracing::debug!("Sent network sync message");
        Ok(())
    }

    /// Get connected peers
    pub fn get_connected_peers(&self) -> &HashMap<PeerId, String> {
        &self.connected_peers
    }

    /// Add a peer to the connected peers list
    pub fn add_peer(&mut self, peer_id: PeerId, address: String) {
        self.connected_peers.insert(peer_id, address);
        tracing::info!("Added peer {} at {}", peer_id, address);
    }

    /// Remove a peer from the connected peers list
    pub fn remove_peer(&mut self, peer_id: &PeerId) {
        if let Some(address) = self.connected_peers.remove(peer_id) {
            tracing::info!("Removed peer {} at {}", peer_id, address);
        }
    }

    /// Handle incoming network messages
    pub async fn handle_message(&mut self, message: NetworkMessage) -> Result<()> {
        match message {
            NetworkMessage::Transaction(transaction) => {
                tracing::debug!("Received transaction: {}", transaction.id);
                // Process transaction
            }
            NetworkMessage::MLTask(task) => {
                tracing::debug!("Received ML task: {}", task.id);
                // Process ML task
            }
            NetworkMessage::MLTaskResult { task_id, result } => {
                tracing::debug!("Received ML task result for: {}", task_id);
                // Process ML task result
            }
            NetworkMessage::PeerDiscovery { peer_id, address } => {
                tracing::debug!("Peer discovery: {} at {}", peer_id, address);
                // Handle peer discovery
            }
            NetworkMessage::NetworkSync { timestamp, data_hash } => {
                tracing::debug!("Network sync message: timestamp {}", timestamp);
                // Handle network synchronization
            }
            NetworkMessage::Heartbeat { peer_id, timestamp } => {
                tracing::debug!("Heartbeat from peer: {} at {}", peer_id, timestamp);
                // Handle heartbeat
            }
        }
        Ok(())
    }

    /// Get network statistics
    pub fn get_network_stats(&self) -> NetworkStats {
        NetworkStats {
            connected_peers: self.connected_peers.len(),
            node_id: self.node_id,
        }
    }
}

/// Network statistics
#[derive(Debug, Serialize, Deserialize)]
pub struct NetworkStats {
    pub connected_peers: usize,
    pub node_id: Uuid,
}

/// Fast synchronization using timestamp-based data chunks
pub struct FastSync {
    chunks: HashMap<i64, DataChunk>,
    latest_timestamp: i64,
}

impl FastSync {
    pub fn new() -> Self {
        FastSync {
            chunks: HashMap::new(),
            latest_timestamp: 0,
        }
    }

    /// Add a data chunk
    pub fn add_chunk(&mut self, chunk: DataChunk) {
        if chunk.timestamp > self.latest_timestamp {
            self.latest_timestamp = chunk.timestamp;
        }
        self.chunks.insert(chunk.timestamp, chunk);
    }

    /// Get chunks after a specific timestamp
    pub fn get_chunks_after(&self, timestamp: i64) -> Vec<&DataChunk> {
        self.chunks
            .values()
            .filter(|chunk| chunk.timestamp > timestamp)
            .collect()
    }

    /// Check if client is synced
    pub fn is_synced(&self, client_timestamp: i64) -> bool {
        (self.latest_timestamp - client_timestamp) < 60000 // Within 1 minute
    }
}

/// Data chunk for fast synchronization
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataChunk {
    pub timestamp: i64,
    pub hash: Vec<u8>,
    pub data: Vec<u8>,
    pub chunk_type: ChunkType,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ChunkType {
    Transactions,
    MLTasks,
    Balances,
    NetworkState,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_network_creation() {
        let node_id = Uuid::new_v4();
        let network = P2PNetwork::new(node_id).await.unwrap();
        assert_eq!(network.node_id, node_id);
        assert_eq!(network.connected_peers.len(), 0);
    }

    #[test]
    fn test_fast_sync() {
        let mut sync = FastSync::new();
        
        let chunk = DataChunk {
            timestamp: 1000,
            hash: vec![1, 2, 3],
            data: vec![4, 5, 6],
            chunk_type: ChunkType::Transactions,
        };

        sync.add_chunk(chunk);
        assert_eq!(sync.latest_timestamp, 1000);
        
        let chunks = sync.get_chunks_after(500);
        assert_eq!(chunks.len(), 1);
        
        assert!(sync.is_synced(950)); // Within 1 minute
        assert!(!sync.is_synced(500)); // More than 1 minute behind
    }
}
