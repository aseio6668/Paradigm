use anyhow::Result;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use super::keeper::Keeper;
use super::proofs::ProofVerificationResult;
use super::sigil::Sigil;
use crate::Amount;

/// Reward calculation and distribution system for storage contributors
pub struct StorageRewardEngine {
    /// Current reward pool balance in PAR tokens
    pub reward_pool_balance: Amount,

    /// Reward rates for different activities
    pub reward_rates: RewardRates,

    /// Pending rewards to be distributed
    pending_rewards: HashMap<String, Amount>,

    /// Reward history for auditing
    reward_history: Vec<RewardTransaction>,
}

/// Reward rates for different storage activities
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RewardRates {
    /// PAR per GB per month for storage
    pub storage_rate_per_gb_month: Amount,

    /// PAR bonus for successful storage proof
    pub proof_success_bonus: Amount,

    /// PAR bonus for successful data retrieval
    pub retrieval_success_bonus: Amount,

    /// PAR bonus for high uptime (per day)
    pub uptime_bonus_per_day: Amount,

    /// Multiplier for critical importance data
    pub critical_data_multiplier: f64,

    /// Multiplier for high-reputation keepers
    pub reputation_multiplier_max: f64,

    /// Penalty for failed proofs (deducted from rewards)
    pub proof_failure_penalty: Amount,
}

impl Default for RewardRates {
    fn default() -> Self {
        Self {
            storage_rate_per_gb_month: 1_00000000, // 1 PAR per GB per month
            proof_success_bonus: 10_0000000,       // 0.1 PAR per successful proof
            retrieval_success_bonus: 5_0000000,    // 0.05 PAR per successful retrieval
            uptime_bonus_per_day: 10_0000000,      // 0.1 PAR per day of uptime
            critical_data_multiplier: 3.0,
            reputation_multiplier_max: 2.0,
            proof_failure_penalty: 50_0000000, // 0.5 PAR penalty
        }
    }
}

/// A reward transaction record
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RewardTransaction {
    /// Recipient keeper ID
    pub keeper_id: String,

    /// Amount of PAR rewarded
    pub amount: Amount,

    /// Type of reward
    pub reward_type: RewardType,

    /// Related sigil hash (if applicable)
    pub sigil_hash: Option<String>,

    /// When the reward was earned
    pub timestamp: DateTime<Utc>,

    /// Additional metadata
    pub metadata: HashMap<String, String>,
}

/// Types of rewards that can be earned
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum RewardType {
    /// Reward for storing data over time
    StorageReward,

    /// Bonus for successful storage proof
    ProofBonus,

    /// Bonus for successful data retrieval
    RetrievalBonus,

    /// Bonus for high uptime
    UptimeBonus,

    /// Penalty for failed operations
    Penalty,

    /// One-time bonus for joining network
    RegistrationBonus,

    /// Bonus for contributing to network health
    NetworkHealthBonus,
}

impl StorageRewardEngine {
    pub fn new(initial_pool_balance: Amount) -> Self {
        Self {
            reward_pool_balance: initial_pool_balance,
            reward_rates: RewardRates::default(),
            pending_rewards: HashMap::new(),
            reward_history: Vec::new(),
        }
    }

    /// Calculate storage reward for a keeper based on their stored sigils
    pub fn calculate_storage_reward(
        &self,
        keeper: &Keeper,
        sigils: &[Sigil],
        days_since_last_reward: u32,
    ) -> Amount {
        let mut total_reward = 0;

        for sigil in sigils {
            // Calculate base reward based on data size
            let size_gb = sigil.size as f64 / (1024.0 * 1024.0 * 1024.0);
            let days_fraction = days_since_last_reward as f64 / 30.0; // Convert to monthly fraction
            let base_reward = (size_gb
                * self.reward_rates.storage_rate_per_gb_month as f64
                * days_fraction) as Amount;

            // Apply importance multiplier
            let importance_multiplier = match sigil.glyph.importance {
                super::glyph::Importance::Critical => self.reward_rates.critical_data_multiplier,
                super::glyph::Importance::Major => 2.0,
                super::glyph::Importance::Standard => 1.0,
                super::glyph::Importance::Minor => 0.5,
                super::glyph::Importance::Trivial => 0.1,
                super::glyph::Importance::Legendary => 5.0,
            };

            // Apply reputation multiplier
            let reputation_multiplier =
                1.0 + (keeper.reputation * (self.reward_rates.reputation_multiplier_max - 1.0));

            let adjusted_reward =
                (base_reward as f64 * importance_multiplier * reputation_multiplier) as Amount;
            total_reward += adjusted_reward;
        }

        total_reward
    }

    /// Award a proof bonus for successful storage verification
    pub fn award_proof_bonus(
        &mut self,
        keeper_id: String,
        proof_result: &ProofVerificationResult,
        sigil_hash: String,
    ) -> Result<Amount> {
        let reward = if proof_result.valid {
            // Scale reward based on proof score
            (self.reward_rates.proof_success_bonus as f64 * proof_result.score) as Amount
        } else {
            // Apply penalty for failed proof
            return self.apply_penalty(
                keeper_id,
                self.reward_rates.proof_failure_penalty,
                sigil_hash,
            );
        };

        self.add_pending_reward(keeper_id.clone(), reward);

        // Record transaction
        self.reward_history.push(RewardTransaction {
            keeper_id: keeper_id.clone(),
            amount: reward,
            reward_type: RewardType::ProofBonus,
            sigil_hash: Some(sigil_hash),
            timestamp: Utc::now(),
            metadata: {
                let mut meta = HashMap::new();
                meta.insert("proof_score".to_string(), proof_result.score.to_string());
                meta.insert(
                    "verification_time_ms".to_string(),
                    proof_result.verification_time_ms.to_string(),
                );
                meta
            },
        });

        tracing::info!(
            "💰 Awarded {} PAR proof bonus to keeper {} (score: {})",
            reward as f64 / 1_00000000.0,
            keeper_id,
            proof_result.score
        );

        Ok(reward)
    }

    /// Award a retrieval bonus for successful data retrieval
    pub fn award_retrieval_bonus(
        &mut self,
        keeper_id: String,
        response_time_ms: u64,
        sigil_hash: String,
    ) -> Result<Amount> {
        // Scale reward based on response time (faster = more reward)
        let time_multiplier = if response_time_ms < 1000 {
            1.5 // 1.5x for sub-second response
        } else if response_time_ms < 5000 {
            1.0 // Normal reward for 1-5 second response
        } else if response_time_ms < 30000 {
            0.5 // Half reward for 5-30 second response
        } else {
            0.1 // Minimal reward for slow response
        };

        let reward = (self.reward_rates.retrieval_success_bonus as f64 * time_multiplier) as Amount;

        self.add_pending_reward(keeper_id.clone(), reward);

        // Record transaction
        self.reward_history.push(RewardTransaction {
            keeper_id: keeper_id.clone(),
            amount: reward,
            reward_type: RewardType::RetrievalBonus,
            sigil_hash: Some(sigil_hash),
            timestamp: Utc::now(),
            metadata: {
                let mut meta = HashMap::new();
                meta.insert("response_time_ms".to_string(), response_time_ms.to_string());
                meta.insert("time_multiplier".to_string(), time_multiplier.to_string());
                meta
            },
        });

        tracing::info!(
            "💰 Awarded {} PAR retrieval bonus to keeper {} ({}ms)",
            reward as f64 / 1_00000000.0,
            keeper_id,
            response_time_ms
        );

        Ok(reward)
    }

    /// Award uptime bonus for keepers with high availability
    pub fn award_uptime_bonus(&mut self, keeper_id: String, uptime_hours: f64) -> Result<Amount> {
        if uptime_hours < 20.0 {
            return Ok(0); // No bonus for less than 20 hours uptime
        }

        let days = uptime_hours / 24.0;
        let reward = (self.reward_rates.uptime_bonus_per_day as f64 * days) as Amount;

        self.add_pending_reward(keeper_id.clone(), reward);

        // Record transaction
        self.reward_history.push(RewardTransaction {
            keeper_id: keeper_id.clone(),
            amount: reward,
            reward_type: RewardType::UptimeBonus,
            sigil_hash: None,
            timestamp: Utc::now(),
            metadata: {
                let mut meta = HashMap::new();
                meta.insert("uptime_hours".to_string(), uptime_hours.to_string());
                meta
            },
        });

        tracing::info!(
            "💰 Awarded {} PAR uptime bonus to keeper {} ({:.1}h uptime)",
            reward as f64 / 1_00000000.0,
            keeper_id,
            uptime_hours
        );

        Ok(reward)
    }

    /// Apply a penalty (negative reward)
    fn apply_penalty(
        &mut self,
        keeper_id: String,
        penalty_amount: Amount,
        sigil_hash: String,
    ) -> Result<Amount> {
        // Reduce pending rewards or add negative balance
        let current_pending = self.pending_rewards.get(&keeper_id).copied().unwrap_or(0);
        let new_pending = if current_pending >= penalty_amount {
            current_pending - penalty_amount
        } else {
            0 // Don't go negative, just reduce to zero
        };

        self.pending_rewards.insert(keeper_id.clone(), new_pending);

        // Record penalty transaction
        self.reward_history.push(RewardTransaction {
            keeper_id: keeper_id.clone(),
            amount: penalty_amount,
            reward_type: RewardType::Penalty,
            sigil_hash: Some(sigil_hash),
            timestamp: Utc::now(),
            metadata: HashMap::new(),
        });

        tracing::warn!(
            "⚠️ Applied {} PAR penalty to keeper {}",
            penalty_amount as f64 / 1_00000000.0,
            keeper_id
        );

        Ok(penalty_amount)
    }

    /// Add reward to pending rewards for a keeper
    pub fn add_pending_reward(&mut self, keeper_id: String, amount: Amount) {
        let current = self.pending_rewards.get(&keeper_id).copied().unwrap_or(0);
        self.pending_rewards.insert(keeper_id, current + amount);
    }

    /// Get pending rewards for a keeper
    pub fn get_pending_rewards(&self, keeper_id: &str) -> Amount {
        self.pending_rewards.get(keeper_id).copied().unwrap_or(0)
    }

    /// Get all pending rewards
    pub fn get_all_pending_rewards(&self) -> &HashMap<String, Amount> {
        &self.pending_rewards
    }

    /// Distribute pending rewards (this would integrate with the main tokenomics system)
    pub async fn distribute_rewards(&mut self) -> Result<Vec<RewardDistribution>> {
        let mut distributions = Vec::new();
        let mut remaining_rewards = HashMap::new();

        for (keeper_id, amount) in self.pending_rewards.drain() {
            if amount > 0 && amount <= self.reward_pool_balance {
                distributions.push(RewardDistribution {
                    keeper_id: keeper_id.clone(),
                    amount,
                    timestamp: Utc::now(),
                });

                self.reward_pool_balance -= amount;

                tracing::info!(
                    "💸 Distributed {} PAR to keeper {}",
                    amount as f64 / 1_00000000.0,
                    keeper_id
                );
            } else if amount > self.reward_pool_balance {
                tracing::warn!(
                    "💰 Insufficient reward pool balance for keeper {} (need: {}, have: {})",
                    keeper_id,
                    amount as f64 / 1_00000000.0,
                    self.reward_pool_balance as f64 / 1_00000000.0
                );

                // Keep reward for later distribution
                remaining_rewards.insert(keeper_id, amount);
            }
        }

        // Restore undistributed rewards
        self.pending_rewards = remaining_rewards;

        Ok(distributions)
    }

    /// Add funds to the reward pool
    pub fn add_to_reward_pool(&mut self, amount: Amount) {
        self.reward_pool_balance += amount;
        tracing::info!(
            "💰 Added {} PAR to storage reward pool (new balance: {} PAR)",
            amount as f64 / 1_00000000.0,
            self.reward_pool_balance as f64 / 1_00000000.0
        );
    }

    /// Get reward history for a keeper
    pub fn get_keeper_reward_history(
        &self,
        keeper_id: &str,
        limit: Option<usize>,
    ) -> Vec<RewardTransaction> {
        let mut history: Vec<_> = self
            .reward_history
            .iter()
            .filter(|tx| tx.keeper_id == keeper_id)
            .cloned()
            .collect();

        // Sort by timestamp (newest first)
        history.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));

        if let Some(limit) = limit {
            history.truncate(limit);
        }

        history
    }

    /// Get reward statistics
    pub fn get_reward_statistics(&self) -> RewardStatistics {
        let total_rewards_paid: Amount = self
            .reward_history
            .iter()
            .filter(|tx| tx.reward_type != RewardType::Penalty)
            .map(|tx| tx.amount)
            .sum();

        let total_penalties: Amount = self
            .reward_history
            .iter()
            .filter(|tx| tx.reward_type == RewardType::Penalty)
            .map(|tx| tx.amount)
            .sum();

        let pending_total: Amount = self.pending_rewards.values().sum();

        let unique_keepers = {
            let mut keepers = std::collections::HashSet::new();
            for tx in &self.reward_history {
                keepers.insert(tx.keeper_id.clone());
            }
            keepers.len()
        };

        RewardStatistics {
            total_rewards_paid,
            total_penalties,
            pending_rewards_total: pending_total,
            reward_pool_balance: self.reward_pool_balance,
            total_transactions: self.reward_history.len(),
            unique_rewarded_keepers: unique_keepers,
        }
    }

    /// Update reward rates (governance function)
    pub fn update_reward_rates(&mut self, new_rates: RewardRates) {
        self.reward_rates = new_rates;
        tracing::info!("📊 Storage reward rates updated");
    }
}

/// Record of a reward distribution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RewardDistribution {
    pub keeper_id: String,
    pub amount: Amount,
    pub timestamp: DateTime<Utc>,
}

/// Overall reward system statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RewardStatistics {
    pub total_rewards_paid: Amount,
    pub total_penalties: Amount,
    pub pending_rewards_total: Amount,
    pub reward_pool_balance: Amount,
    pub total_transactions: usize,
    pub unique_rewarded_keepers: usize,
}

#[cfg(test)]
mod tests {
    use super::super::glyph::{DataCategory, Element, Importance};
    use super::*;

    #[test]
    fn test_storage_reward_calculation() {
        let engine = StorageRewardEngine::new(1000_00000000); // 1000 PAR pool

        // Create test keeper
        let mut keeper =
            super::super::keeper::Keeper::new("127.0.0.1:8080".to_string(), 1024 * 1024 * 1024); // 1GB capacity
        keeper.reputation = 0.8; // High reputation

        // Create test sigil
        let glyph = super::super::glyph::Glyph::new(
            Element::Earth,
            DataCategory::Archive,
            Importance::Normal,
        );
        let sigil = super::super::sigil::Sigil::new(
            vec![0u8; 100 * 1024 * 1024], // 100MB
            glyph,
            "test_originator".to_string(),
            0,
        )
        .unwrap();

        let reward = engine.calculate_storage_reward(&keeper, &[sigil], 30); // 30 days

        assert!(reward > 0);
        println!("Calculated reward: {} PAR", reward as f64 / 1_00000000.0);
    }

    #[test]
    fn test_proof_bonus() {
        let mut engine = StorageRewardEngine::new(1000_00000000);

        let proof_result = super::super::proofs::ProofVerificationResult {
            valid: true,
            verification_time_ms: 50,
            error_message: None,
            score: 1.0,
        };

        let reward = engine
            .award_proof_bonus(
                "test_keeper".to_string(),
                &proof_result,
                "test_sigil".to_string(),
            )
            .unwrap();

        assert_eq!(reward, engine.reward_rates.proof_success_bonus);
        assert_eq!(engine.get_pending_rewards("test_keeper"), reward);
    }

    #[test]
    fn test_penalty_system() {
        let mut engine = StorageRewardEngine::new(1000_00000000);

        // Add some pending rewards first
        engine.add_pending_reward("test_keeper".to_string(), 100_00000000);

        // Apply penalty
        let penalty = engine
            .apply_penalty(
                "test_keeper".to_string(),
                50_00000000,
                "test_sigil".to_string(),
            )
            .unwrap();

        assert_eq!(penalty, 50_00000000);
        assert_eq!(engine.get_pending_rewards("test_keeper"), 50_00000000); // Reduced by penalty
    }
}
