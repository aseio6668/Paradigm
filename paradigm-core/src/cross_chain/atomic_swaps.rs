use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use tokio::sync::RwLock;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::{DateTime, Utc};
use anyhow::{Result, anyhow};
use blake3::Hasher;

use crate::{Hash, Amount, Address, transaction::Transaction};
use super::{ChainId, CrossChainConfig};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SwapState {
    Initiated,
    SecretLocked,
    SecretRevealed,
    Redeemed,
    Refunded,
    Expired,
    Failed,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AtomicSwap {
    pub swap_id: Uuid,
    pub chain_a: ChainId,
    pub chain_b: ChainId,
    pub participant_a: Address,
    pub participant_b: Address,
    pub asset_a: String,
    pub asset_b: String,
    pub amount_a: Amount,
    pub amount_b: Amount,
    pub secret_hash: Hash,
    pub secret: Option<Vec<u8>>,
    pub lock_time_a: u64,
    pub lock_time_b: u64,
    pub refund_time_a: u64,
    pub refund_time_b: u64,
    pub state: SwapState,
    pub contract_address_a: Option<String>,
    pub contract_address_b: Option<String>,
    pub lock_tx_hash_a: Option<String>,
    pub lock_tx_hash_b: Option<String>,
    pub redeem_tx_hash_a: Option<String>,
    pub redeem_tx_hash_b: Option<String>,
    pub refund_tx_hash_a: Option<String>,
    pub refund_tx_hash_b: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub timeout_height_a: u64,
    pub timeout_height_b: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SwapProposal {
    pub proposal_id: Uuid,
    pub proposer: Address,
    pub counterparty: Address,
    pub offered_asset: String,
    pub offered_amount: Amount,
    pub requested_asset: String,
    pub requested_amount: Amount,
    pub offered_chain: ChainId,
    pub requested_chain: ChainId,
    pub expiry_time: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HashTimeLockContract {
    pub contract_id: Uuid,
    pub chain_id: ChainId,
    pub sender: Address,
    pub receiver: Address,
    pub amount: Amount,
    pub asset: String,
    pub secret_hash: Hash,
    pub lock_time: u64,
    pub refund_time: u64,
    pub is_locked: bool,
    pub is_redeemed: bool,
    pub is_refunded: bool,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SwapSecret {
    pub secret: Vec<u8>,
    pub hash: Hash,
    pub revealed_at: Option<DateTime<Utc>>,
}

impl SwapSecret {
    pub fn new() -> Self {
        let mut secret = vec![0u8; 32];
        use rand::RngCore;
        rand::rngs::OsRng.fill_bytes(&mut secret);
        
        let mut hasher = Hasher::new();
        hasher.update(&secret);
        let hash_bytes = hasher.finalize();
        let mut hash = [0u8; 32];
        hash.copy_from_slice(hash_bytes.as_bytes());
        
        Self {
            secret,
            hash,
            revealed_at: None,
        }
    }
    
    pub fn from_secret(secret: Vec<u8>) -> Self {
        let mut hasher = Hasher::new();
        hasher.update(&secret);
        let hash_bytes = hasher.finalize();
        let mut hash = [0u8; 32];
        hash.copy_from_slice(hash_bytes.as_bytes());
        
        Self {
            secret,
            hash,
            revealed_at: None,
        }
    }
    
    pub fn verify(&self, hash: &Hash) -> bool {
        &self.hash == hash
    }
}

pub struct AtomicSwapEngine {
    active_swaps: Arc<RwLock<HashMap<Uuid, AtomicSwap>>>,
    pending_proposals: Arc<RwLock<HashMap<Uuid, SwapProposal>>>,
    htlc_contracts: Arc<RwLock<HashMap<Uuid, HashTimeLockContract>>>,
    swap_secrets: Arc<RwLock<HashMap<Hash, SwapSecret>>>,
    config: CrossChainConfig,
    chain_handlers: Arc<RwLock<HashMap<ChainId, Box<dyn ChainHandler + Send + Sync>>>>,
}

#[async_trait::async_trait]
pub trait ChainHandler {
    async fn create_htlc(&self, contract: &HashTimeLockContract) -> Result<String>;
    async fn redeem_htlc(&self, contract_id: &str, secret: &[u8]) -> Result<String>;
    async fn refund_htlc(&self, contract_id: &str) -> Result<String>;
    async fn check_htlc_status(&self, contract_id: &str) -> Result<HTLCStatus>;
    async fn get_current_block_height(&self) -> Result<u64>;
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum HTLCStatus {
    Created,
    Locked,
    Redeemed,
    Refunded,
    Expired,
}

impl AtomicSwapEngine {
    pub async fn new(config: &CrossChainConfig) -> Result<Self> {
        Ok(Self {
            active_swaps: Arc::new(RwLock::new(HashMap::new())),
            pending_proposals: Arc::new(RwLock::new(HashMap::new())),
            htlc_contracts: Arc::new(RwLock::new(HashMap::new())),
            swap_secrets: Arc::new(RwLock::new(HashMap::new())),
            config: config.clone(),
            chain_handlers: Arc::new(RwLock::new(HashMap::new())),
        })
    }

    pub async fn initialize(&self) -> Result<()> {
        tracing::info!("Initializing Atomic Swap Engine...");
        
        // Start monitoring tasks
        self.start_monitoring_tasks().await?;
        
        tracing::info!("Atomic Swap Engine initialized successfully");
        Ok(())
    }

    pub async fn create_swap_proposal(
        &self,
        proposer: Address,
        counterparty: Address,
        offered_asset: String,
        offered_amount: Amount,
        requested_asset: String,
        requested_amount: Amount,
        offered_chain: ChainId,
        requested_chain: ChainId,
        expiry_hours: u64,
    ) -> Result<Uuid> {
        let proposal_id = Uuid::new_v4();
        let expiry_time = Utc::now() + chrono::Duration::hours(expiry_hours as i64);
        
        let proposal = SwapProposal {
            proposal_id,
            proposer,
            counterparty,
            offered_asset,
            offered_amount,
            requested_asset,
            requested_amount,
            offered_chain,
            requested_chain,
            expiry_time,
            created_at: Utc::now(),
        };
        
        let mut proposals = self.pending_proposals.write().await;
        proposals.insert(proposal_id, proposal);
        
        tracing::info!("Created swap proposal: {}", proposal_id);
        Ok(proposal_id)
    }

    pub async fn accept_swap_proposal(&self, proposal_id: Uuid, acceptor: Address) -> Result<Uuid> {
        let proposal = {
            let mut proposals = self.pending_proposals.write().await;
            proposals.remove(&proposal_id)
                .ok_or_else(|| anyhow!("Swap proposal not found"))?
        };
        
        if proposal.counterparty != acceptor {
            return Err(anyhow!("Only the specified counterparty can accept this proposal"));
        }
        
        if Utc::now() > proposal.expiry_time {
            return Err(anyhow!("Swap proposal has expired"));
        }
        
        let swap_id = self.initiate_atomic_swap(
            proposal.offered_chain,
            proposal.requested_chain,
            proposal.proposer,
            proposal.counterparty,
            proposal.offered_asset,
            proposal.requested_asset,
            proposal.offered_amount,
            proposal.requested_amount,
        ).await?;
        
        tracing::info!("Accepted swap proposal {} -> swap {}", proposal_id, swap_id);
        Ok(swap_id)
    }

    pub async fn initiate_atomic_swap(
        &self,
        chain_a: ChainId,
        chain_b: ChainId,
        participant_a: Address,
        participant_b: Address,
        asset_a: String,
        asset_b: String,
        amount_a: Amount,
        amount_b: Amount,
    ) -> Result<Uuid> {
        let swap_id = Uuid::new_v4();
        let secret = SwapSecret::new();
        let secret_hash = secret.hash;
        
        // Calculate timeouts
        let current_time = SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs();
        let lock_time_a = current_time + 3600; // 1 hour
        let lock_time_b = current_time + 1800; // 30 minutes
        let refund_time_a = current_time + 7200; // 2 hours
        let refund_time_b = current_time + 5400; // 1.5 hours
        
        let swap = AtomicSwap {
            swap_id,
            chain_a,
            chain_b,
            participant_a,
            participant_b,
            asset_a,
            asset_b,
            amount_a,
            amount_b,
            secret_hash,
            secret: None,
            lock_time_a,
            lock_time_b,
            refund_time_a,
            refund_time_b,
            state: SwapState::Initiated,
            contract_address_a: None,
            contract_address_b: None,
            lock_tx_hash_a: None,
            lock_tx_hash_b: None,
            redeem_tx_hash_a: None,
            redeem_tx_hash_b: None,
            refund_tx_hash_a: None,
            refund_tx_hash_b: None,
            created_at: Utc::now(),
            updated_at: Utc::now(),
            timeout_height_a: 0,
            timeout_height_b: 0,
        };
        
        // Store secret
        {
            let mut secrets = self.swap_secrets.write().await;
            secrets.insert(secret_hash, secret);
        }
        
        // Store swap
        {
            let mut swaps = self.active_swaps.write().await;
            swaps.insert(swap_id, swap);
        }
        
        tracing::info!("Initiated atomic swap: {}", swap_id);
        Ok(swap_id)
    }

    pub async fn lock_funds(
        &self,
        swap_id: Uuid,
        chain_id: ChainId,
        participant: Address,
    ) -> Result<String> {
        let mut swap = {
            let swaps = self.active_swaps.read().await;
            swaps.get(&swap_id).cloned()
                .ok_or_else(|| anyhow!("Swap not found"))?
        };
        
        if swap.state != SwapState::Initiated {
            return Err(anyhow!("Swap is not in initiated state"));
        }
        
        let (amount, asset) = if chain_id == swap.chain_a {
            (swap.amount_a, swap.asset_a.clone())
        } else if chain_id == swap.chain_b {
            (swap.amount_b, swap.asset_b.clone())
        } else {
            return Err(anyhow!("Invalid chain for this swap"));
        };
        
        let contract_id = Uuid::new_v4();
        let htlc = HashTimeLockContract {
            contract_id,
            chain_id,
            sender: participant,
            receiver: if chain_id == swap.chain_a { swap.participant_b } else { swap.participant_a },
            amount,
            asset,
            secret_hash: swap.secret_hash,
            lock_time: if chain_id == swap.chain_a { swap.lock_time_a } else { swap.lock_time_b },
            refund_time: if chain_id == swap.chain_a { swap.refund_time_a } else { swap.refund_time_b },
            is_locked: false,
            is_redeemed: false,
            is_refunded: false,
            created_at: Utc::now(),
        };
        
        // Store HTLC
        {
            let mut contracts = self.htlc_contracts.write().await;
            contracts.insert(contract_id, htlc.clone());
        }
        
        // Create HTLC on chain
        let tx_hash = self.create_htlc_on_chain(&htlc).await?;
        
        // Update swap
        {
            let mut swaps = self.active_swaps.write().await;
            if let Some(swap) = swaps.get_mut(&swap_id) {
                if chain_id == swap.chain_a {
                    swap.contract_address_a = Some(contract_id.to_string());
                    swap.lock_tx_hash_a = Some(tx_hash.clone());
                } else {
                    swap.contract_address_b = Some(contract_id.to_string());
                    swap.lock_tx_hash_b = Some(tx_hash.clone());
                }
                swap.state = SwapState::SecretLocked;
                swap.updated_at = Utc::now();
            }
        }
        
        tracing::info!("Locked funds for swap {} on chain {:?}", swap_id, chain_id);
        Ok(tx_hash)
    }

    pub async fn redeem_funds(
        &self,
        swap_id: Uuid,
        chain_id: ChainId,
        secret: Vec<u8>,
    ) -> Result<String> {
        let swap = {
            let swaps = self.active_swaps.read().await;
            swaps.get(&swap_id).cloned()
                .ok_or_else(|| anyhow!("Swap not found"))?
        };
        
        // Verify secret
        let mut hasher = Hasher::new();
        hasher.update(&secret);
        let computed_hash_bytes = hasher.finalize();
        let mut computed_hash = [0u8; 32];
        computed_hash.copy_from_slice(computed_hash_bytes.as_bytes());
        
        if computed_hash != swap.secret_hash {
            return Err(anyhow!("Invalid secret"));
        }
        
        let contract_address = if chain_id == swap.chain_a {
            swap.contract_address_a.as_ref()
        } else {
            swap.contract_address_b.as_ref()
        }.ok_or_else(|| anyhow!("No contract found for this chain"))?;
        
        // Redeem HTLC on chain
        let tx_hash = self.redeem_htlc_on_chain(contract_address, &secret).await?;
        
        // Update swap
        {
            let mut swaps = self.active_swaps.write().await;
            if let Some(swap) = swaps.get_mut(&swap_id) {
                swap.secret = Some(secret.clone());
                if chain_id == swap.chain_a {
                    swap.redeem_tx_hash_a = Some(tx_hash.clone());
                } else {
                    swap.redeem_tx_hash_b = Some(tx_hash.clone());
                }
                swap.state = SwapState::SecretRevealed;
                swap.updated_at = Utc::now();
            }
        }
        
        // Reveal secret
        {
            let mut secrets = self.swap_secrets.write().await;
            if let Some(stored_secret) = secrets.get_mut(&swap.secret_hash) {
                stored_secret.revealed_at = Some(Utc::now());
            }
        }
        
        tracing::info!("Redeemed funds for swap {} on chain {:?}", swap_id, chain_id);
        Ok(tx_hash)
    }

    pub async fn refund_funds(
        &self,
        swap_id: Uuid,
        chain_id: ChainId,
    ) -> Result<String> {
        let swap = {
            let swaps = self.active_swaps.read().await;
            swaps.get(&swap_id).cloned()
                .ok_or_else(|| anyhow!("Swap not found"))?
        };
        
        let contract_address = if chain_id == swap.chain_a {
            swap.contract_address_a.as_ref()
        } else {
            swap.contract_address_b.as_ref()
        }.ok_or_else(|| anyhow!("No contract found for this chain"))?;
        
        // Check if refund time has passed
        let current_time = SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs();
        let refund_time = if chain_id == swap.chain_a { swap.refund_time_a } else { swap.refund_time_b };
        
        if current_time < refund_time {
            return Err(anyhow!("Refund time has not yet passed"));
        }
        
        // Refund HTLC on chain
        let tx_hash = self.refund_htlc_on_chain(contract_address).await?;
        
        // Update swap
        {
            let mut swaps = self.active_swaps.write().await;
            if let Some(swap) = swaps.get_mut(&swap_id) {
                if chain_id == swap.chain_a {
                    swap.refund_tx_hash_a = Some(tx_hash.clone());
                } else {
                    swap.refund_tx_hash_b = Some(tx_hash.clone());
                }
                swap.state = SwapState::Refunded;
                swap.updated_at = Utc::now();
            }
        }
        
        tracing::info!("Refunded funds for swap {} on chain {:?}", swap_id, chain_id);
        Ok(tx_hash)
    }

    pub async fn get_swap(&self, swap_id: &Uuid) -> Option<AtomicSwap> {
        let swaps = self.active_swaps.read().await;
        swaps.get(swap_id).cloned()
    }

    pub async fn get_swap_proposal(&self, proposal_id: &Uuid) -> Option<SwapProposal> {
        let proposals = self.pending_proposals.read().await;
        proposals.get(proposal_id).cloned()
    }

    pub async fn list_active_swaps(&self) -> Vec<AtomicSwap> {
        let swaps = self.active_swaps.read().await;
        swaps.values().cloned().collect()
    }

    pub async fn list_pending_proposals(&self) -> Vec<SwapProposal> {
        let proposals = self.pending_proposals.read().await;
        proposals.values().cloned().collect()
    }

    pub async fn handle_swap(&self, transaction_id: Uuid) -> Result<()> {
        tracing::info!("Handling atomic swap transaction: {}", transaction_id);
        Ok(())
    }

    // Private methods
    
    async fn start_monitoring_tasks(&self) -> Result<()> {
        // Monitor swap timeouts
        let active_swaps = self.active_swaps.clone();
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(Duration::from_secs(60));
            loop {
                interval.tick().await;
                
                let current_time = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();
                let mut expired_swaps = Vec::new();
                
                {
                    let swaps = active_swaps.read().await;
                    for (swap_id, swap) in swaps.iter() {
                        if current_time > swap.refund_time_a.max(swap.refund_time_b) {
                            expired_swaps.push(*swap_id);
                        }
                    }
                }
                
                for swap_id in expired_swaps {
                    let mut swaps = active_swaps.write().await;
                    if let Some(swap) = swaps.get_mut(&swap_id) {
                        swap.state = SwapState::Expired;
                        swap.updated_at = Utc::now();
                        tracing::warn!("Atomic swap {} expired", swap_id);
                    }
                }
            }
        });
        
        // Monitor proposal expiry
        let pending_proposals = self.pending_proposals.clone();
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(Duration::from_secs(300)); // 5 minutes
            loop {
                interval.tick().await;
                
                let now = Utc::now();
                let mut expired_proposals = Vec::new();
                
                {
                    let proposals = pending_proposals.read().await;
                    for (proposal_id, proposal) in proposals.iter() {
                        if now > proposal.expiry_time {
                            expired_proposals.push(*proposal_id);
                        }
                    }
                }
                
                {
                    let mut proposals = pending_proposals.write().await;
                    for proposal_id in expired_proposals {
                        proposals.remove(&proposal_id);
                        tracing::info!("Removed expired swap proposal: {}", proposal_id);
                    }
                }
            }
        });
        
        Ok(())
    }
    
    async fn create_htlc_on_chain(&self, htlc: &HashTimeLockContract) -> Result<String> {
        // This would interact with the actual blockchain
        // For now, we'll simulate a successful transaction
        tokio::time::sleep(Duration::from_millis(100)).await;
        Ok(format!("0x{}", hex::encode(&htlc.contract_id.as_bytes()[..16])))
    }
    
    async fn redeem_htlc_on_chain(&self, contract_address: &str, secret: &[u8]) -> Result<String> {
        // This would interact with the actual blockchain
        // For now, we'll simulate a successful transaction
        tokio::time::sleep(Duration::from_millis(100)).await;
        Ok(format!("0x{}", hex::encode(&secret[..16])))
    }
    
    async fn refund_htlc_on_chain(&self, contract_address: &str) -> Result<String> {
        // This would interact with the actual blockchain
        // For now, we'll simulate a successful transaction
        tokio::time::sleep(Duration::from_millis(100)).await;
        Ok(format!("0x{}", hex::encode(contract_address.as_bytes())))
    }
}

impl Default for AtomicSwapEngine {
    fn default() -> Self {
        let config = CrossChainConfig::default();
        // This is a synchronous implementation that should only be used for testing
        Self {
            active_swaps: Arc::new(RwLock::new(HashMap::new())),
            pending_proposals: Arc::new(RwLock::new(HashMap::new())),
            htlc_contracts: Arc::new(RwLock::new(HashMap::new())),
            swap_secrets: Arc::new(RwLock::new(HashMap::new())),
            config,
            chain_handlers: Arc::new(RwLock::new(HashMap::new())),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_swap_secret_creation() {
        let secret = SwapSecret::new();
        assert_eq!(secret.secret.len(), 32);
        assert!(secret.verify(&secret.hash));
    }
    
    #[tokio::test]
    async fn test_atomic_swap_engine_creation() {
        let config = CrossChainConfig::default();
        let engine = AtomicSwapEngine::new(&config).await;
        assert!(engine.is_ok());
    }
    
    #[tokio::test]
    async fn test_swap_proposal_creation() {
        let config = CrossChainConfig::default();
        let engine = AtomicSwapEngine::new(&config).await.unwrap();
        
        let proposer = Address([1u8; 32]);
        let counterparty = Address([2u8; 32]);
        
        let proposal_id = engine.create_swap_proposal(
            proposer,
            counterparty,
            "PAR".to_string(),
            1000,
            "ETH".to_string(),
            500,
            ChainId::Paradigm,
            ChainId::Ethereum,
            24,
        ).await.unwrap();
        
        let proposal = engine.get_swap_proposal(&proposal_id).await;
        assert!(proposal.is_some());
        assert_eq!(proposal.unwrap().offered_amount, 1000);
    }
    
    #[tokio::test]
    async fn test_atomic_swap_initiation() {
        let config = CrossChainConfig::default();
        let engine = AtomicSwapEngine::new(&config).await.unwrap();
        
        let participant_a = Address([1u8; 32]);
        let participant_b = Address([2u8; 32]);
        
        let swap_id = engine.initiate_atomic_swap(
            ChainId::Paradigm,
            ChainId::Ethereum,
            participant_a,
            participant_b,
            "PAR".to_string(),
            "ETH".to_string(),
            1000,
            500,
        ).await.unwrap();
        
        let swap = engine.get_swap(&swap_id).await;
        assert!(swap.is_some());
        assert_eq!(swap.unwrap().state, SwapState::Initiated);
    }
}