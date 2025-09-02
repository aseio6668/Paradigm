// Proof of Work implementation for Paradigm network
// Provides security against double-spending and 51% attacks

use anyhow::Result;
use blake3::Hasher;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::time::Instant;
use tokio::sync::RwLock;
use uuid::Uuid;

use crate::transaction::Transaction;

/// Block header containing proof-of-work data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlockHeader {
    pub block_id: Uuid,
    pub previous_hash: Vec<u8>,
    pub merkle_root: Vec<u8>,
    pub timestamp: DateTime<Utc>,
    pub difficulty: u32,
    pub nonce: u64,
    pub hash: Vec<u8>,
}

/// Complete block with transactions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Block {
    pub header: BlockHeader,
    pub transactions: Vec<Transaction>,
    pub transaction_count: u32,
}

/// Mining statistics and performance metrics
#[derive(Debug, Clone)]
pub struct MiningStats {
    pub blocks_mined: u64,
    pub total_hash_attempts: u64,
    pub average_time_per_block: f64,
    pub current_difficulty: u32,
    pub network_hashrate: f64,
}

/// Proof of Work mining engine
pub struct ProofOfWorkMiner {
    difficulty: Arc<RwLock<u32>>,
    stats: Arc<RwLock<MiningStats>>,
    target_block_time: u64,              // seconds
    difficulty_adjustment_interval: u32, // blocks
}

impl ProofOfWorkMiner {
    pub fn new(initial_difficulty: u32, target_block_time: u64) -> Self {
        Self {
            difficulty: Arc::new(RwLock::new(initial_difficulty)),
            stats: Arc::new(RwLock::new(MiningStats {
                blocks_mined: 0,
                total_hash_attempts: 0,
                average_time_per_block: 0.0,
                current_difficulty: initial_difficulty,
                network_hashrate: 0.0,
            })),
            target_block_time,
            difficulty_adjustment_interval: 10, // Adjust every 10 blocks
        }
    }

    /// Mine a new block with proof-of-work
    pub async fn mine_block(
        &self,
        transactions: Vec<Transaction>,
        previous_hash: Vec<u8>,
    ) -> Result<Block> {
        let start_time = Instant::now();
        let current_difficulty = *self.difficulty.read().await;
        let target = Self::calculate_target(current_difficulty);

        tracing::info!(
            "🔨 Starting to mine block with {} transactions (difficulty: {})",
            transactions.len(),
            current_difficulty
        );

        // Calculate merkle root
        let merkle_root = self.calculate_merkle_root(&transactions);

        // Create block header
        let mut header = BlockHeader {
            block_id: Uuid::new_v4(),
            previous_hash,
            merkle_root,
            timestamp: Utc::now(),
            difficulty: current_difficulty,
            nonce: 0,
            hash: Vec::new(),
        };

        // Mine: find nonce that produces hash below target
        let mut attempts = 0u64;
        loop {
            header.nonce += 1;
            attempts += 1;

            let hash = self.calculate_block_hash(&header);

            // Check if hash meets difficulty target
            if Self::hash_meets_target(&hash, &target) {
                header.hash = hash;
                break;
            }

            // Progress logging every 100k attempts
            if attempts % 100_000 == 0 {
                tracing::debug!(
                    "Mining progress: {} attempts, nonce: {}",
                    attempts,
                    header.nonce
                );
            }

            // Prevent infinite loops in development
            if attempts > 10_000_000 {
                tracing::warn!("Mining took over 10M attempts, reducing difficulty");
                let mut diff = self.difficulty.write().await;
                *diff = (*diff).saturating_sub(1).max(1);
                break;
            }
        }

        let mining_time = start_time.elapsed().as_secs_f64();

        // Update statistics
        let mut stats = self.stats.write().await;
        stats.blocks_mined += 1;
        stats.total_hash_attempts += attempts;
        stats.current_difficulty = current_difficulty;
        stats.average_time_per_block =
            (stats.average_time_per_block * (stats.blocks_mined - 1) as f64 + mining_time)
                / stats.blocks_mined as f64;
        stats.network_hashrate = attempts as f64 / mining_time;

        let tx_len = transactions.len() as u32;
        let block = Block {
            header,
            transactions,
            transaction_count: tx_len,
        };

        tracing::info!(
            "⚡ Block {} mined! Time: {:.2}s, Attempts: {}, Hash: {}",
            block.header.block_id,
            mining_time,
            attempts,
            hex::encode(&block.header.hash[..8])
        );

        // Adjust difficulty if needed
        if stats.blocks_mined % self.difficulty_adjustment_interval as u64 == 0 {
            self.adjust_difficulty(&stats).await;
        }

        Ok(block)
    }

    /// Validate a block's proof-of-work
    pub async fn validate_block(&self, block: &Block) -> Result<bool> {
        let current_difficulty = *self.difficulty.read().await;

        // Check if block difficulty matches current network difficulty
        if block.header.difficulty != current_difficulty {
            tracing::warn!(
                "Block difficulty {} doesn't match network difficulty {}",
                block.header.difficulty,
                current_difficulty
            );
            return Ok(false);
        }

        // Verify merkle root
        let calculated_merkle = self.calculate_merkle_root(&block.transactions);
        if calculated_merkle != block.header.merkle_root {
            tracing::error!("Block merkle root validation failed");
            return Ok(false);
        }

        // Verify proof-of-work hash
        let calculated_hash = self.calculate_block_hash(&block.header);
        if calculated_hash != block.header.hash {
            tracing::error!("Block hash validation failed");
            return Ok(false);
        }

        // Check if hash meets difficulty target
        let target = Self::calculate_target(block.header.difficulty);
        if !Self::hash_meets_target(&calculated_hash, &target) {
            tracing::error!("Block hash doesn't meet difficulty target");
            return Ok(false);
        }

        tracing::info!(
            "✅ Block {} proof-of-work validated successfully",
            block.header.block_id
        );
        Ok(true)
    }

    /// Calculate block hash for proof-of-work
    fn calculate_block_hash(&self, header: &BlockHeader) -> Vec<u8> {
        let mut hasher = Hasher::new();
        hasher.update(header.block_id.as_bytes());
        hasher.update(&header.previous_hash);
        hasher.update(&header.merkle_root);
        hasher.update(
            &header
                .timestamp
                .timestamp_nanos_opt()
                .unwrap_or(0)
                .to_le_bytes(),
        );
        hasher.update(&header.difficulty.to_le_bytes());
        hasher.update(&header.nonce.to_le_bytes());
        hasher.finalize().as_bytes().to_vec()
    }

    /// Calculate merkle root of transactions
    fn calculate_merkle_root(&self, transactions: &[Transaction]) -> Vec<u8> {
        if transactions.is_empty() {
            return vec![0; 32]; // Empty merkle root
        }

        // Calculate transaction hashes
        let mut hashes: Vec<Vec<u8>> = transactions.iter().map(|tx| tx.hash()).collect();

        // Build merkle tree
        while hashes.len() > 1 {
            let mut next_level = Vec::new();

            for chunk in hashes.chunks(2) {
                let mut hasher = Hasher::new();
                hasher.update(&chunk[0]);
                if chunk.len() > 1 {
                    hasher.update(&chunk[1]);
                } else {
                    hasher.update(&chunk[0]); // Duplicate if odd number
                }
                next_level.push(hasher.finalize().as_bytes().to_vec());
            }

            hashes = next_level;
        }

        hashes.into_iter().next().unwrap_or_else(|| vec![0; 32])
    }

    /// Calculate difficulty target (lower = harder)
    fn calculate_target(difficulty: u32) -> Vec<u8> {
        let leading_zeros = difficulty.min(31); // Max 31 zero bytes
        let mut target = vec![0u8; leading_zeros as usize];
        target.extend_from_slice(&vec![255u8; 32 - leading_zeros as usize]);
        target.truncate(32);
        target
    }

    /// Check if hash meets difficulty target
    fn hash_meets_target(hash: &[u8], target: &[u8]) -> bool {
        hash.iter().zip(target.iter()).all(|(&h, &t)| h <= t)
    }

    /// Adjust mining difficulty based on block times
    async fn adjust_difficulty(&self, stats: &MiningStats) {
        let current_difficulty = *self.difficulty.read().await;
        let target_time = self.target_block_time as f64;
        let actual_time = stats.average_time_per_block;

        let new_difficulty = if actual_time > target_time * 1.5 {
            // Blocks taking too long, reduce difficulty
            current_difficulty.saturating_sub(1).max(1)
        } else if actual_time < target_time * 0.75 {
            // Blocks too fast, increase difficulty
            current_difficulty.saturating_add(1).min(20) // Max difficulty cap
        } else {
            current_difficulty
        };

        if new_difficulty != current_difficulty {
            let mut difficulty = self.difficulty.write().await;
            *difficulty = new_difficulty;
            tracing::info!(
                "⚖️ Difficulty adjusted: {} → {} (avg block time: {:.1}s)",
                current_difficulty,
                new_difficulty,
                actual_time
            );
        }
    }

    /// Get current mining statistics
    pub async fn get_stats(&self) -> MiningStats {
        self.stats.read().await.clone()
    }

    /// Get current difficulty
    pub async fn get_difficulty(&self) -> u32 {
        *self.difficulty.read().await
    }
}

/// Block validation results
#[derive(Debug, Clone)]
pub struct BlockValidationResult {
    pub is_valid: bool,
    pub errors: Vec<String>,
    pub hash_rate_estimate: f64,
}

impl Block {
    /// Calculate the total fees in this block
    pub fn total_fees(&self) -> u64 {
        self.transactions.iter().map(|tx| tx.fee).sum()
    }

    /// Calculate block size in bytes (estimated)
    pub fn estimated_size(&self) -> usize {
        // Rough estimate: 64 bytes per transaction + header overhead
        self.transactions.len() * 64 + 256
    }

    /// Get block hash
    pub fn hash(&self) -> Vec<u8> {
        self.header.hash.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Address, AddressExt, Keypair};
    use ed25519_dalek::SigningKey;

    #[tokio::test]
    async fn test_proof_of_work_mining() {
        let miner = ProofOfWorkMiner::new(1, 30); // Low difficulty for testing

        // Create test transactions
        let keypair = SigningKey::from_bytes(&rand::random());
        let from_addr = AddressExt::from_public_key(&keypair.verifying_key());
        let to_addr = AddressExt::from_public_key(&keypair.verifying_key());

        let tx = Transaction::new(from_addr, to_addr, 1000, 10, Utc::now(), &keypair).unwrap();

        let transactions = vec![tx];
        let previous_hash = vec![0; 32];

        // Mine a block
        let block = miner.mine_block(transactions, previous_hash).await.unwrap();

        // Validate the block
        let is_valid = miner.validate_block(&block).await.unwrap();
        assert!(is_valid);

        // Check stats
        let stats = miner.get_stats().await;
        assert_eq!(stats.blocks_mined, 1);
    }
}
