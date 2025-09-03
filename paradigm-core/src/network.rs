use anyhow::Result;
use libp2p::{
    gossipsub, identity::Keypair as LibP2PKeypair, kad, mdns, noise, swarm::Swarm, tcp, yamux,
    PeerId, Transport,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
// Removed unused Arc and RwLock imports
use uuid::Uuid;

use crate::ml_tasks::{MLTask, MLTaskResult};
use crate::transaction::Transaction;

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

/// Data chunk types for network synchronization
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ChunkType {
    Transactions,
    MLTasks,
    Balances,
    NetworkState,
}

/// P2P Network behavior combining Gossipsub, mDNS, and Kademlia
#[derive(libp2p::swarm::NetworkBehaviour)]
pub struct P2PBehaviour {
    pub gossipsub: gossipsub::Behaviour,
    pub mdns: mdns::tokio::Behaviour,
    pub kademlia: kad::Behaviour<kad::store::MemoryStore>,
}

/// P2P Network struct
pub struct P2PNetwork {
    swarm: Swarm<P2PBehaviour>,
    connected_peers: HashMap<PeerId, String>,
    transaction_topic: gossipsub::IdentTopic,
    ml_task_topic: gossipsub::IdentTopic,
    sync_topic: gossipsub::IdentTopic,
    node_id: Uuid,
}

impl P2PNetwork {
    pub async fn new(node_id: Uuid) -> Result<Self> {
        // Generate a random PeerId
        let local_key = LibP2PKeypair::generate_ed25519();
        let local_peer_id = PeerId::from(local_key.public());

        // Create a TCP transport
        let transport = tcp::tokio::Transport::default()
            .upgrade(libp2p::core::upgrade::Version::V1)
            .authenticate(noise::Config::new(&local_key)?)
            .multiplex(yamux::Config::default())
            .boxed();

        // Create gossipsub configuration
        let gossipsub_config = gossipsub::ConfigBuilder::default()
            .heartbeat_interval(std::time::Duration::from_secs(10))
            .validation_mode(gossipsub::ValidationMode::Strict)
            .build()
            .map_err(|msg| anyhow::anyhow!("Gossipsub config error: {}", msg))?;

        let mut gossipsub = gossipsub::Behaviour::new(
            gossipsub::MessageAuthenticity::Signed(local_key.clone()),
            gossipsub_config,
        )
        .map_err(|msg| anyhow::anyhow!("Gossipsub creation error: {}", msg))?;

        // Create and subscribe to topics
        let transaction_topic = gossipsub::IdentTopic::new("paradigm-transactions");
        let ml_task_topic = gossipsub::IdentTopic::new("paradigm-ml-tasks");
        let sync_topic = gossipsub::IdentTopic::new("paradigm-sync");

        gossipsub.subscribe(&transaction_topic)?;
        gossipsub.subscribe(&ml_task_topic)?;
        gossipsub.subscribe(&sync_topic)?;

        // Create mDNS behaviour
        let mdns = mdns::tokio::Behaviour::new(mdns::Config::default(), local_peer_id)?;

        // Create Kademlia behaviour
        let store = kad::store::MemoryStore::new(local_peer_id);
        let kademlia = kad::Behaviour::new(local_peer_id, store);

        // Create behaviour
        let behaviour = P2PBehaviour {
            gossipsub,
            mdns,
            kademlia,
        };

        // Create swarm
        let swarm = Swarm::new(
            transport,
            behaviour,
            local_peer_id,
            libp2p::swarm::Config::with_tokio_executor(),
        );

        Ok(Self {
            swarm,
            connected_peers: HashMap::new(),
            transaction_topic,
            ml_task_topic,
            sync_topic,
            node_id,
        })
    }

    pub async fn start_listening(&mut self) -> Result<()> {
        self.swarm.listen_on("/ip4/0.0.0.0/tcp/0".parse()?)?;
        Ok(())
    }

    pub async fn broadcast_transaction(&mut self, transaction: &Transaction) -> Result<()> {
        let message = NetworkMessage::Transaction(transaction.clone());
        let serialized = serde_json::to_vec(&message)?;

        if let Err(e) = self
            .swarm
            .behaviour_mut()
            .gossipsub
            .publish(self.transaction_topic.clone(), serialized)
        {
            tracing::warn!("Failed to broadcast transaction: {:?}", e);
        }

        Ok(())
    }

    pub async fn broadcast_ml_task(&mut self, task: &MLTask) -> Result<()> {
        let message = NetworkMessage::MLTask(task.clone());
        let serialized = serde_json::to_vec(&message)?;

        if let Err(e) = self
            .swarm
            .behaviour_mut()
            .gossipsub
            .publish(self.ml_task_topic.clone(), serialized)
        {
            tracing::warn!("Failed to broadcast ML task: {:?}", e);
        }

        Ok(())
    }

    pub async fn broadcast_ml_result(
        &mut self,
        task_id: Uuid,
        result: &MLTaskResult,
    ) -> Result<()> {
        let message = NetworkMessage::MLTaskResult {
            task_id,
            result: result.result.clone(),
        };
        let serialized = serde_json::to_vec(&message)?;

        if let Err(e) = self
            .swarm
            .behaviour_mut()
            .gossipsub
            .publish(self.ml_task_topic.clone(), serialized)
        {
            tracing::warn!("Failed to broadcast ML result: {:?}", e);
        }

        Ok(())
    }

    pub async fn sync_network(&mut self, timestamp: i64, data_hash: Vec<u8>) -> Result<()> {
        let message = NetworkMessage::NetworkSync {
            timestamp,
            data_hash,
        };
        let serialized = serde_json::to_vec(&message)?;

        if let Err(e) = self
            .swarm
            .behaviour_mut()
            .gossipsub
            .publish(self.sync_topic.clone(), serialized)
        {
            tracing::warn!("Failed to sync network: {:?}", e);
        }

        Ok(())
    }

    pub fn add_peer(&mut self, peer_id: PeerId, address: String) {
        let address_clone = address.clone();
        self.connected_peers.insert(peer_id, address);
        tracing::info!("Added peer {} at {}", peer_id, address_clone);
    }

    pub fn get_connected_peers(&self) -> &HashMap<PeerId, String> {
        &self.connected_peers
    }

    // Temporarily disabled while fixing libp2p event handling
    // pub async fn handle_network_event(
    //     &mut self,
    //     event: SwarmEvent<<P2PBehaviour as libp2p::swarm::NetworkBehaviour>::ToSwarm>,
    // ) -> Result<Option<NetworkMessage>> {
    //     match event {
    //         SwarmEvent::Behaviour(_) => {
    //             tracing::debug!("Received network behavior event");
    //             Ok(None)
    //         },
    //         SwarmEvent::NewListenAddr { address, .. } => {
    //             tracing::info!("Listening on {}", address);
    //             Ok(None)
    //         }
    //         _ => Ok(None),
    //     }
    // }

    pub async fn process_network_message(&mut self, message: NetworkMessage) -> Result<()> {
        match message {
            NetworkMessage::Transaction(transaction) => {
                tracing::info!("Received transaction: {:?}", transaction.id);
                // Forward to transaction processor
                Ok(())
            }
            NetworkMessage::MLTask(task) => {
                tracing::info!("Received ML task: {:?}", task.id);
                // Forward to ML task engine
                Ok(())
            }
            NetworkMessage::MLTaskResult { task_id, result: _ } => {
                tracing::info!("Received ML task result for: {:?}", task_id);
                // Forward to consensus engine
                Ok(())
            }
            NetworkMessage::PeerDiscovery {
                peer_id: _,
                address: _,
            } => {
                tracing::debug!("Peer discovery message received");
                Ok(())
            }
            NetworkMessage::NetworkSync {
                timestamp: _,
                data_hash: _,
            } => {
                tracing::debug!("Network sync message received");
                Ok(())
            }
            NetworkMessage::Heartbeat {
                peer_id: _,
                timestamp: _,
            } => {
                tracing::debug!("Heartbeat received");
                Ok(())
            }
        }
    }

    pub async fn tick(&mut self) -> Result<()> {
        // Simplified network tick - just process events without message handling for now
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
        Ok(())
    }
}
