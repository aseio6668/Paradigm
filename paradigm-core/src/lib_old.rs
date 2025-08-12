use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use chrono::Utc;
use serde::{Deserialize, Serialize};
use ed25519_dalek::{SigningKey, VerifyingKey, Signature, Signer, Verifier};
use blake3::Hasher;
use uuid::Uuid;

/// The core Paradigm network node implementation
pub mod error;
pub mod network;
pub mod consensus;
pub mod transaction;
pub mod wallet;
pub mod ml_tasks;
pub mod governance;
pub mod storage;

use crate::transaction::{Transaction, TransactionPool};
use crate::consensus::ConsensusEngine;
use crate::network::P2PNetwork;
use crate::governance::AIGovernance;

/// Core constants for Paradigm
pub const PARADIGM_VERSION: &str = "0.1.0";
pub const INITIAL_SUPPLY: u64 = 8_000_000_000_00000000; // 8B PAR with 8 decimals
pub const FIRST_YEAR_DISTRIBUTION: u64 = 1_000_000_000_00000000; // 1B PAR first year
pub const DECIMAL_PLACES: u8 = 8;
pub const NETWORK_ID: &str = "paradigm-mainnet";

/// Paradigm address format (32-byte public key + checksum)
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Address(pub [u8; 32]);

impl Address {
    // Type aliases for easier use
pub type Address = String;
pub type Hash = [u8; 32];
pub type Amount = u64;
pub type PublicKey = ed25519_dalek::VerifyingKey;
pub type SecretKey = ed25519_dalek::SigningKey;
pub type Keypair = ed25519_dalek::SigningKey; // In the new API, SigningKey is the keypair

// Re-export commonly used types
pub use error::ParadigmError;

impl Address {
    pub fn from_public_key(public_key: &PublicKey) -> Self {
        let mut hasher = Hasher::new();
        hasher.update(public_key.as_bytes());
        let hash = hasher.finalize();
        format!("PAR{}", hex::encode(&hash.as_bytes()[..20]))
    }
}
}

/// Main Paradigm node structure
#[derive(Debug)]
pub struct ParadigmNode {
    pub id: Uuid,
    pub address: Address,
    pub keypair: Keypair,
    pub transaction_pool: Arc<RwLock<TransactionPool>>,
    pub consensus_engine: Arc<RwLock<ConsensusEngine>>,
    pub network: Arc<RwLock<P2PNetwork>>,
    pub ai_governance: Arc<RwLock<AIGovernance>>,
    pub balances: Arc<RwLock<HashMap<Address, u64>>>,
    pub is_contributor: bool,
}

impl ParadigmNode {
    /// Create a new Paradigm node
    pub async fn new(is_contributor: bool) -> anyhow::Result<Self> {
        let keypair = Keypair::generate(&mut rand::thread_rng());
        let address = Address::from_public_key(&keypair.public);
        let id = Uuid::new_v4();

        let transaction_pool = Arc::new(RwLock::new(TransactionPool::new()));
        let consensus_engine = Arc::new(RwLock::new(ConsensusEngine::new()));
        let network = Arc::new(RwLock::new(P2PNetwork::new(id).await?));
        let ai_governance = Arc::new(RwLock::new(AIGovernance::new()));
        let balances = Arc::new(RwLock::new(HashMap::new()));

        Ok(ParadigmNode {
            id,
            address,
            keypair,
            transaction_pool,
            consensus_engine,
            network,
            ai_governance,
            balances,
            is_contributor,
        })
    }

    /// Start the node and begin network operations
    pub async fn start(&self) -> anyhow::Result<()> {
        tracing::info!("Starting Paradigm node {} at address {}", self.id, self.address.to_string());

        // Initialize network
        let mut network = self.network.write().await;
        network.start().await?;
        drop(network);

        // Start consensus engine
        let mut consensus = self.consensus_engine.write().await;
        consensus.start().await?;
        drop(consensus);

        // Start AI governance if this is a contributor node
        if self.is_contributor {
            let mut governance = self.ai_governance.write().await;
            governance.start().await?;
        }

        tracing::info!("Paradigm node started successfully");
        Ok(())
    }

    /// Send a transaction
    pub async fn send_transaction(
        &self,
        to: Address,
        amount: u64,
        fee: u64,
    ) -> anyhow::Result<Transaction> {
        let transaction = Transaction::new(
            self.address.clone(),
            to,
            amount,
            fee,
            Utc::now(),
            &self.keypair,
        )?;

        let mut pool = self.transaction_pool.write().await;
        pool.add_transaction(transaction.clone()).await?;

        // Broadcast to network
        let network = self.network.read().await;
        network.broadcast_transaction(&transaction).await?;

        Ok(transaction)
    }

    /// Get balance for an address
    pub async fn get_balance(&self, address: &Address) -> u64 {
        let balances = self.balances.read().await;
        *balances.get(address).unwrap_or(&0)
    }

    /// Process incoming ML task (for contributor nodes)
    pub async fn process_ml_task(&self, task_data: Vec<u8>) -> anyhow::Result<Vec<u8>> {
        if !self.is_contributor {
            return Err(anyhow::anyhow!("Node is not configured as a contributor"));
        }

        // Process ML task and return result
        let governance = self.ai_governance.read().await;
        governance.process_task(task_data).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_node_creation() {
        let node = ParadigmNode::new(false).await.unwrap();
        assert!(!node.is_contributor);
        assert_eq!(node.get_balance(&node.address).await, 0);
    }

    #[tokio::test]
    async fn test_address_generation() {
        let keypair = Keypair::generate(&mut rand::thread_rng());
        let address = Address::from_public_key(&keypair.public);
        let address_str = address.to_string();
        assert!(address_str.starts_with("PAR"));
        assert_eq!(address_str.len(), 67); // "PAR" + 64 hex chars
    }
}
