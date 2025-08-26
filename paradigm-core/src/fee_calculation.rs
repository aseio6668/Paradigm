use crate::{Address, Amount, genesis::AIGovernanceParams, storage::ParadigmStorage};
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};
use tokio::sync::RwLock;
use tracing;

/// AI-driven dynamic fee calculation system
#[derive(Debug)]
pub struct DynamicFeeCalculator {
    storage: Arc<RwLock<ParadigmStorage>>,
    network_metrics: Arc<RwLock<NetworkMetrics>>,
    contributor_rewards: Arc<RwLock<ContributorRewardSystem>>,
}

/// Network congestion and performance metrics
#[derive(Debug, Clone)]
pub struct NetworkMetrics {
    pub transaction_volume_24h: Amount,
    pub pending_transaction_count: usize,
    pub average_confirmation_time: f64, // seconds
    pub network_congestion: f64, // 0.0 to 1.0
    pub active_contributors: usize,
    pub last_updated: u64,
}

/// Contributor reward balancing system
#[derive(Debug)]
pub struct ContributorRewardSystem {
    pub total_contributors: usize,
    pub active_contributors: usize,
    pub reward_pool_balance: Amount,
    pub min_contribution_threshold: Amount,
    pub reward_multiplier: f64,
}

/// Fee calculation result with breakdown
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeeCalculationResult {
    pub base_fee: Amount,
    pub congestion_adjustment: Amount,
    pub contributor_incentive: i64, // Can be negative for fee reductions
    pub total_fee: Amount,
    pub fee_percentage: f64,
    pub calculation_factors: FeeFactors,
}

/// Factors that influence fee calculation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeeFactors {
    pub transaction_amount: Amount,
    pub network_congestion: f64,
    pub contributor_load: f64,
    pub reward_pool_health: f64,
    pub governance_multiplier: f64,
}

impl Default for NetworkMetrics {
    fn default() -> Self {
        Self {
            transaction_volume_24h: 0,
            pending_transaction_count: 0,
            average_confirmation_time: 10.0, // 10 seconds default
            network_congestion: 0.0,
            active_contributors: 1,
            last_updated: SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs(),
        }
    }
}

impl Default for ContributorRewardSystem {
    fn default() -> Self {
        Self {
            total_contributors: 1,
            active_contributors: 1,
            reward_pool_balance: 1000_00000000, // 1000 PAR initial pool
            min_contribution_threshold: 10_00000000, // 10 PAR minimum
            reward_multiplier: 1.0,
        }
    }
}

impl DynamicFeeCalculator {
    pub fn new(storage: Arc<RwLock<ParadigmStorage>>) -> Self {
        Self {
            storage,
            network_metrics: Arc::new(RwLock::new(NetworkMetrics::default())),
            contributor_rewards: Arc::new(RwLock::new(ContributorRewardSystem::default())),
        }
    }

    /// Calculate dynamic fee for a transaction using AI governance
    pub async fn calculate_transaction_fee(
        &self,
        transaction_amount: Amount,
        sender: &Address,
        urgent: bool,
    ) -> Result<FeeCalculationResult> {
        // Get AI governance parameters
        let governance_params = self.storage.read().await.get_ai_governance_params().await?;
        let network_metrics = self.network_metrics.read().await.clone();
        let reward_system = self.contributor_rewards.read().await;

        // Calculate base fee using AI parameters
        let base_fee_percentage = self.calculate_base_fee_percentage(
            &governance_params,
            &network_metrics,
            transaction_amount,
        ).await;

        let base_fee = ((transaction_amount as f64) * base_fee_percentage) as Amount;

        // Apply congestion adjustments
        let congestion_adjustment = self.calculate_congestion_adjustment(
            &governance_params,
            &network_metrics,
            base_fee,
            urgent,
        ).await;

        // Calculate contributor incentive (can be negative to reduce fees)
        let contributor_incentive_signed = self.calculate_contributor_incentive(
            &reward_system,
            &network_metrics,
            transaction_amount,
        ).await;

        // Convert incentive to amount, handling negative values
        let contributor_incentive = if contributor_incentive_signed < 0 {
            0 // We'll subtract the absolute value from the total
        } else {
            contributor_incentive_signed as Amount
        };

        // Total fee calculation - apply reduction if incentive was negative
        let base_total = base_fee + congestion_adjustment + contributor_incentive;
        let reduction = if contributor_incentive_signed < 0 {
            (-contributor_incentive_signed) as Amount
        } else {
            0
        };
        let total_fee = base_total.saturating_sub(reduction).max(1); // Minimum 1 unit

        // Ensure fee is within governance bounds
        let min_fee = ((transaction_amount as f64) * governance_params.min_fee_percentage) as Amount;
        let max_fee = ((transaction_amount as f64) * governance_params.max_fee_percentage) as Amount;
        let final_fee = total_fee.max(min_fee).min(max_fee);

        let result = FeeCalculationResult {
            base_fee,
            congestion_adjustment,
            contributor_incentive: contributor_incentive_signed, // Keep original signed value for reporting
            total_fee: final_fee,
            fee_percentage: (final_fee as f64) / (transaction_amount as f64),
            calculation_factors: FeeFactors {
                transaction_amount,
                network_congestion: network_metrics.network_congestion,
                contributor_load: reward_system.active_contributors as f64 / reward_system.total_contributors as f64,
                reward_pool_health: (reward_system.reward_pool_balance as f64) / (100_000_00000000.0), // Health as % of 100k PAR
                governance_multiplier: governance_params.fee_sensitivity,
            },
        };

        tracing::debug!(
            "Dynamic fee calculated: {:.8} PAR ({:.4}%) for transaction of {:.8} PAR",
            final_fee as f64 / 100_000_000.0,
            result.fee_percentage * 100.0,
            transaction_amount as f64 / 100_000_000.0
        );

        Ok(result)
    }

    /// Calculate base fee percentage using AI governance
    async fn calculate_base_fee_percentage(
        &self,
        params: &AIGovernanceParams,
        metrics: &NetworkMetrics,
        amount: Amount,
    ) -> f64 {
        let mut base_percentage = params.min_fee_percentage;

        // AI-driven adjustments based on transaction patterns
        if amount > 1000_00000000 {
            // Large transactions (>1000 PAR) get slightly higher base fee
            base_percentage *= 1.2;
        } else if amount < 1_00000000 {
            // Small transactions (<1 PAR) get reduced base fee
            base_percentage *= 0.8;
        }

        // Network health adjustment
        let health_factor = 1.0 - (metrics.network_congestion * 0.3);
        base_percentage *= health_factor;

        base_percentage.max(0.0001).min(params.max_fee_percentage) // At least 0.01%
    }

    /// Calculate congestion-based fee adjustment
    async fn calculate_congestion_adjustment(
        &self,
        params: &AIGovernanceParams,
        metrics: &NetworkMetrics,
        base_fee: Amount,
        urgent: bool,
    ) -> Amount {
        let mut adjustment_factor = metrics.network_congestion * params.fee_sensitivity;

        // Urgent transactions pay premium
        if urgent {
            adjustment_factor *= 2.0;
        }

        // Consider pending transaction backlog
        let backlog_factor = (metrics.pending_transaction_count as f64 / 1000.0).min(1.0);
        adjustment_factor += backlog_factor * 0.5;

        // Consider confirmation time pressure
        if metrics.average_confirmation_time > 30.0 {
            adjustment_factor *= 1.5;
        }

        ((base_fee as f64) * adjustment_factor) as Amount
    }

    /// Calculate contributor incentive (can reduce fees when network is healthy)
    async fn calculate_contributor_incentive(
        &self,
        reward_system: &ContributorRewardSystem,
        metrics: &NetworkMetrics,
        amount: Amount,
    ) -> i64 {
        // When we have many active contributors and healthy reward pool, reduce fees
        let contributor_health = reward_system.active_contributors as f64 / 10.0; // Ideal: 10+ contributors
        let pool_health = (reward_system.reward_pool_balance as f64) / (1000_00000000.0); // Health relative to 1k PAR

        // Calculate incentive (negative = fee reduction)
        let health_score = (contributor_health * pool_health).min(1.0);
        let fee_reduction = (amount as f64) * 0.001 * health_score; // Up to 0.1% reduction

        // Apply fee reduction more aggressively for smaller transactions
        let size_multiplier = if amount < 10_00000000 { 2.0 } else { 1.0 }; // 2x reduction for <10 PAR

        // Return negative for fee reduction
        -((fee_reduction * size_multiplier) as i64)
    }

    /// Update network metrics (called by consensus engine)
    pub async fn update_network_metrics(
        &self,
        transaction_volume_24h: Amount,
        pending_count: usize,
        avg_confirmation_time: f64,
        active_contributors: usize,
    ) -> Result<()> {
        let mut metrics = self.network_metrics.write().await;
        
        // Calculate congestion based on various factors
        let congestion = self.calculate_congestion_score(
            pending_count,
            avg_confirmation_time,
            transaction_volume_24h,
        ).await;

        metrics.transaction_volume_24h = transaction_volume_24h;
        metrics.pending_transaction_count = pending_count;
        metrics.average_confirmation_time = avg_confirmation_time;
        metrics.network_congestion = congestion;
        metrics.active_contributors = active_contributors;
        metrics.last_updated = SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs();

        tracing::debug!(
            "Network metrics updated: congestion {:.2}, pending {}, contributors {}",
            congestion, pending_count, active_contributors
        );

        Ok(())
    }

    /// Calculate network congestion score (0.0 = no congestion, 1.0 = maximum congestion)
    async fn calculate_congestion_score(
        &self,
        pending_count: usize,
        avg_confirmation_time: f64,
        transaction_volume: Amount,
    ) -> f64 {
        // Pending transactions factor (0-1)
        let pending_factor = (pending_count as f64 / 1000.0).min(1.0); // Max at 1000 pending

        // Confirmation time factor (0-1) - ideal is 10 seconds
        let time_factor = ((avg_confirmation_time - 10.0) / 60.0).max(0.0).min(1.0); // Max penalty at 70 seconds

        // Volume factor - high volume can indicate congestion or healthy adoption
        let volume_factor = if transaction_volume > 10000_00000000 {
            // Very high volume (>10k PAR/day) adds slight congestion
            0.2
        } else {
            0.0
        };

        // Weighted average
        (pending_factor * 0.5 + time_factor * 0.4 + volume_factor * 0.1).min(1.0)
    }

    /// Update contributor reward system metrics
    pub async fn update_contributor_metrics(
        &self,
        total_contributors: usize,
        active_contributors: usize,
        reward_pool_balance: Amount,
    ) -> Result<()> {
        let mut rewards = self.contributor_rewards.write().await;
        rewards.total_contributors = total_contributors;
        rewards.active_contributors = active_contributors;
        rewards.reward_pool_balance = reward_pool_balance;
        
        // Adjust reward multiplier based on pool health
        rewards.reward_multiplier = if reward_pool_balance > 10000_00000000 {
            1.5 // Healthy pool allows more generous rewards
        } else if reward_pool_balance > 1000_00000000 {
            1.0 // Normal rewards
        } else {
            0.5 // Conservative rewards when pool is low
        };

        Ok(())
    }

    /// Get current fee estimation for UI display
    pub async fn estimate_fee_range(&self, amount: Amount) -> Result<(Amount, Amount, Amount)> {
        // Calculate fees for different scenarios
        let low_fee = self.calculate_transaction_fee(amount, &Address([0; 32]), false).await?.total_fee;
        let normal_fee = {
            // Simulate moderate congestion
            self.update_network_metrics(1000_00000000, 50, 15.0, 5).await?;
            self.calculate_transaction_fee(amount, &Address([0; 32]), false).await?.total_fee
        };
        let high_fee = self.calculate_transaction_fee(amount, &Address([0; 32]), true).await?.total_fee;

        Ok((low_fee, normal_fee, high_fee))
    }

    /// Get network health score for monitoring
    pub async fn get_network_health_score(&self) -> f64 {
        let metrics = self.network_metrics.read().await;
        let rewards = self.contributor_rewards.read().await;

        let congestion_health = 1.0 - metrics.network_congestion;
        let contributor_health = (rewards.active_contributors as f64 / 10.0).min(1.0);
        let pool_health = (rewards.reward_pool_balance as f64 / 1000_00000000.0).min(1.0);

        (congestion_health * 0.4 + contributor_health * 0.3 + pool_health * 0.3).max(0.0).min(1.0)
    }
}