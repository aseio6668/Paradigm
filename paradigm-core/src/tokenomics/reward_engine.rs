use super::{ContributionType, ValidationResult};
use crate::Address;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Advanced reward calculation engine that considers multiple factors
/// including computational value, social trust, reputation, and network needs
#[derive(Debug)]
pub struct RewardEngine {
    /// Base reward rates for different contribution types
    base_rates: HashMap<ContributionType, u64>,
    /// Network demand multipliers
    demand_multipliers: HashMap<ContributionType, f64>,
    /// Reputation weight factors
    reputation_weights: ReputationWeights,
    /// Social trust network
    trust_network: TrustNetwork,
    /// Dynamic pricing model
    pricing_model: DynamicPricing,
    /// Reward history for analysis
    reward_history: Vec<RewardRecord>,
}

impl RewardEngine {
    pub fn new() -> Self {
        let mut base_rates = HashMap::new();
        base_rates.insert(ContributionType::MLTraining, 100_000_000); // 1 PAR base
        base_rates.insert(ContributionType::InferenceServing, 50_000_000); // 0.5 PAR base
        base_rates.insert(ContributionType::DataValidation, 25_000_000); // 0.25 PAR base
        base_rates.insert(ContributionType::ModelOptimization, 150_000_000); // 1.5 PAR base
        base_rates.insert(ContributionType::NetworkMaintenance, 75_000_000); // 0.75 PAR base
        base_rates.insert(ContributionType::GovernanceParticipation, 10_000_000); // 0.1 PAR base
        base_rates.insert(ContributionType::CrossPlatformCompute, 200_000_000); // 2 PAR base
        base_rates.insert(ContributionType::StorageProvision, 30_000_000); // 0.3 PAR base
        base_rates.insert(ContributionType::GenerativeMedia, 80_000_000); // 0.8 PAR base
        base_rates.insert(ContributionType::SymbolicMath, 120_000_000); // 1.2 PAR base

        RewardEngine {
            base_rates,
            demand_multipliers: HashMap::new(),
            reputation_weights: ReputationWeights::default(),
            trust_network: TrustNetwork::new(),
            pricing_model: DynamicPricing::new(),
            reward_history: Vec::new(),
        }
    }

    pub async fn initialize(&mut self) -> anyhow::Result<()> {
        tracing::info!("Initializing Reward Engine");

        // Initialize demand multipliers
        self.initialize_demand_multipliers().await?;

        // Initialize trust network
        self.trust_network.initialize().await?;

        // Initialize dynamic pricing
        self.pricing_model.initialize().await?;

        tracing::info!(
            "Reward Engine initialized with {} contribution types",
            self.base_rates.len()
        );
        Ok(())
    }

    /// Calculate reward using advanced multi-factor system
    pub async fn calculate_reward(
        &mut self,
        validation_result: &ValidationResult,
        reputation: ReputationMetrics,
    ) -> anyhow::Result<u64> {
        // Start with base reward based on compute units
        let base_reward = self.calculate_base_reward(validation_result).await?;

        // Apply quality multiplier
        let quality_multiplier = self.calculate_quality_multiplier(validation_result);

        // Apply reputation multiplier (meritocracy factor)
        let reputation_multiplier = self.calculate_reputation_multiplier(&reputation);

        // Apply novelty bonus
        let novelty_multiplier = self.calculate_novelty_multiplier(validation_result);

        // Apply peer validation multiplier (social trust factor)
        let peer_multiplier = self.calculate_peer_multiplier(validation_result);

        // Apply network demand multiplier
        let demand_multiplier = self.get_current_demand_multiplier().await?;

        // Apply dynamic pricing adjustments
        let pricing_multiplier = self.pricing_model.get_current_multiplier().await?;

        // Calculate final reward
        let final_reward = (base_reward as f64
            * quality_multiplier
            * reputation_multiplier
            * novelty_multiplier
            * peer_multiplier
            * demand_multiplier
            * pricing_multiplier) as u64;

        // Record reward for analysis
        self.record_reward(validation_result, final_reward).await?;

        tracing::debug!(
            "Reward calculated: base={} PAR, quality={:.2}x, reputation={:.2}x, novelty={:.2}x, peer={:.2}x, demand={:.2}x, pricing={:.2}x, final={} PAR",
            base_reward as f64 / 100_000_000.0,
            quality_multiplier,
            reputation_multiplier,
            novelty_multiplier,
            peer_multiplier,
            demand_multiplier,
            pricing_multiplier,
            final_reward as f64 / 100_000_000.0
        );

        Ok(final_reward)
    }

    async fn calculate_base_reward(
        &self,
        validation_result: &ValidationResult,
    ) -> anyhow::Result<u64> {
        // Calculate based on compute units with a scaling factor
        let compute_reward = validation_result.compute_units * 1000; // 1000 base units per compute unit
        Ok(compute_reward.min(1_000_000_000)) // Cap at 10 PAR base
    }

    fn calculate_quality_multiplier(&self, validation_result: &ValidationResult) -> f64 {
        // Quality score between 0.5x and 2.0x multiplier
        let quality_score = validation_result.quality_score.max(0.0).min(1.0);
        0.5 + (quality_score * 1.5) // Range: 0.5 to 2.0
    }

    fn calculate_reputation_multiplier(&self, reputation: &ReputationMetrics) -> f64 {
        // Reputation multiplier based on consistency, expertise, and trust
        let consistency_factor = reputation.consistency_score.max(0.0).min(1.0);
        let expertise_factor = reputation.expertise_score.max(0.0).min(1.0);
        let trust_factor = reputation.trust_score.max(0.0).min(1.0);

        // Weighted combination
        let reputation_score = (consistency_factor * self.reputation_weights.consistency)
            + (expertise_factor * self.reputation_weights.expertise)
            + (trust_factor * self.reputation_weights.trust);

        // Convert to multiplier range: 0.8x to 2.5x
        0.8 + (reputation_score * 1.7)
    }

    fn calculate_novelty_multiplier(&self, validation_result: &ValidationResult) -> f64 {
        // Novelty bonus for original work
        let novelty_score = validation_result.novelty_score.max(0.0).min(1.0);
        1.0 + (novelty_score * 0.5) // Range: 1.0x to 1.5x
    }

    fn calculate_peer_multiplier(&self, validation_result: &ValidationResult) -> f64 {
        // Peer validation multiplier for social consensus
        let peer_score = validation_result.peer_validation_score.max(0.0).min(1.0);
        0.9 + (peer_score * 0.3) // Range: 0.9x to 1.2x
    }

    async fn get_current_demand_multiplier(&self) -> anyhow::Result<f64> {
        // Get network demand multiplier based on current needs
        Ok(1.2) // Simplified: assume 20% demand bonus
    }

    async fn initialize_demand_multipliers(&mut self) -> anyhow::Result<()> {
        // Initialize demand multipliers based on network analysis
        self.demand_multipliers
            .insert(ContributionType::MLTraining, 1.5);
        self.demand_multipliers
            .insert(ContributionType::InferenceServing, 1.8);
        self.demand_multipliers
            .insert(ContributionType::DataValidation, 1.2);
        self.demand_multipliers
            .insert(ContributionType::ModelOptimization, 1.6);
        self.demand_multipliers
            .insert(ContributionType::NetworkMaintenance, 2.0);
        self.demand_multipliers
            .insert(ContributionType::GovernanceParticipation, 1.1);
        self.demand_multipliers
            .insert(ContributionType::CrossPlatformCompute, 1.9);
        self.demand_multipliers
            .insert(ContributionType::StorageProvision, 1.3);
        self.demand_multipliers
            .insert(ContributionType::GenerativeMedia, 1.4);
        self.demand_multipliers
            .insert(ContributionType::SymbolicMath, 1.7);

        Ok(())
    }

    async fn record_reward(
        &mut self,
        validation_result: &ValidationResult,
        reward: u64,
    ) -> anyhow::Result<()> {
        let record = RewardRecord {
            timestamp: Utc::now(),
            compute_units: validation_result.compute_units,
            quality_score: validation_result.quality_score,
            novelty_score: validation_result.novelty_score,
            peer_validation_score: validation_result.peer_validation_score,
            final_reward: reward,
        };

        self.reward_history.push(record);

        // Keep only recent history (last 10000 records)
        if self.reward_history.len() > 10000 {
            self.reward_history.drain(0..1000);
        }

        Ok(())
    }

    /// Get reward statistics for analysis
    pub fn get_reward_stats(&self) -> RewardStats {
        if self.reward_history.is_empty() {
            return RewardStats::default();
        }

        let total_rewards: u64 = self.reward_history.iter().map(|r| r.final_reward).sum();
        let avg_reward = total_rewards / self.reward_history.len() as u64;
        let avg_quality = self
            .reward_history
            .iter()
            .map(|r| r.quality_score)
            .sum::<f64>()
            / self.reward_history.len() as f64;

        RewardStats {
            total_rewards_distributed: total_rewards,
            average_reward: avg_reward,
            total_contributions: self.reward_history.len(),
            average_quality_score: avg_quality,
        }
    }

    /// Update demand multipliers based on network conditions
    pub async fn update_demand_multipliers(
        &mut self,
        network_analysis: NetworkDemandAnalysis,
    ) -> anyhow::Result<()> {
        for (contribution_type, multiplier) in network_analysis.demand_multipliers {
            self.demand_multipliers
                .insert(contribution_type, multiplier);
        }
        Ok(())
    }
}

/// Reputation-based weighting system
#[derive(Debug)]
pub struct ReputationWeights {
    pub consistency: f64, // Weight for consistency in contributions
    pub expertise: f64,   // Weight for domain expertise
    pub trust: f64,       // Weight for peer trust
}

impl Default for ReputationWeights {
    fn default() -> Self {
        ReputationWeights {
            consistency: 0.4, // 40% weight on consistency
            expertise: 0.35,  // 35% weight on expertise
            trust: 0.25,      // 25% weight on trust
        }
    }
}

/// Social trust network for peer validation
#[derive(Debug)]
pub struct TrustNetwork {
    trust_relationships: HashMap<Address, HashMap<Address, f64>>, // from -> to -> trust_score
    trust_scores: HashMap<Address, f64>,                          // aggregated trust scores
}

impl TrustNetwork {
    pub fn new() -> Self {
        TrustNetwork {
            trust_relationships: HashMap::new(),
            trust_scores: HashMap::new(),
        }
    }

    pub async fn initialize(&mut self) -> anyhow::Result<()> {
        tracing::debug!("Initializing Trust Network");
        Ok(())
    }

    pub async fn update_trust(
        &mut self,
        from: &Address,
        to: &Address,
        score: f64,
    ) -> anyhow::Result<()> {
        self.trust_relationships
            .entry(from.clone())
            .or_insert_with(HashMap::new)
            .insert(to.clone(), score);

        // Recalculate aggregated trust score for 'to' address
        self.recalculate_trust_score(to).await?;
        Ok(())
    }

    async fn recalculate_trust_score(&mut self, address: &Address) -> anyhow::Result<()> {
        let mut total_trust = 0.0;
        let mut trust_count = 0;

        for relationships in self.trust_relationships.values() {
            if let Some(trust) = relationships.get(address) {
                total_trust += trust;
                trust_count += 1;
            }
        }

        let avg_trust = if trust_count > 0 {
            total_trust / trust_count as f64
        } else {
            0.5 // Default neutral trust
        };

        self.trust_scores.insert(address.clone(), avg_trust);
        Ok(())
    }

    pub fn get_trust_score(&self, address: &Address) -> f64 {
        *self.trust_scores.get(address).unwrap_or(&0.5)
    }
}

/// Dynamic pricing model for reward adjustments
#[derive(Debug)]
pub struct DynamicPricing {
    current_multiplier: f64,
    supply_factors: SupplyFactors,
    demand_factors: DemandFactors,
}

impl DynamicPricing {
    pub fn new() -> Self {
        DynamicPricing {
            current_multiplier: 1.0,
            supply_factors: SupplyFactors::default(),
            demand_factors: DemandFactors::default(),
        }
    }

    pub async fn initialize(&mut self) -> anyhow::Result<()> {
        tracing::debug!("Initializing Dynamic Pricing model");
        Ok(())
    }

    pub async fn get_current_multiplier(&self) -> anyhow::Result<f64> {
        Ok(self.current_multiplier)
    }

    pub async fn update_pricing(
        &mut self,
        network_conditions: NetworkConditions,
    ) -> anyhow::Result<()> {
        // Calculate new multiplier based on supply and demand
        let supply_factor = self.calculate_supply_factor(&network_conditions);
        let demand_factor = self.calculate_demand_factor(&network_conditions);

        // Combine factors with weights
        self.current_multiplier = (supply_factor * 0.6) + (demand_factor * 0.4);

        // Clamp to reasonable range
        self.current_multiplier = self.current_multiplier.max(0.5).min(3.0);

        Ok(())
    }

    fn calculate_supply_factor(&self, conditions: &NetworkConditions) -> f64 {
        // Higher supply of contributors = lower multiplier
        let contributor_ratio =
            conditions.active_contributors as f64 / conditions.target_contributors as f64;
        if contributor_ratio > 1.2 {
            0.8 // Oversupply
        } else if contributor_ratio < 0.8 {
            1.5 // Undersupply
        } else {
            1.0 // Balanced
        }
    }

    fn calculate_demand_factor(&self, conditions: &NetworkConditions) -> f64 {
        // Higher demand for compute = higher multiplier
        let utilization = conditions.network_utilization;
        if utilization > 0.8 {
            1.3 // High demand
        } else if utilization < 0.4 {
            0.9 // Low demand
        } else {
            1.0 // Normal demand
        }
    }
}

// Data structures

#[derive(Debug, Serialize, Deserialize)]
pub struct ReputationMetrics {
    pub consistency_score: f64, // 0.0 to 1.0
    pub expertise_score: f64,   // 0.0 to 1.0
    pub trust_score: f64,       // 0.0 to 1.0
    pub contribution_count: u64,
    pub average_quality: f64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RewardRecord {
    pub timestamp: DateTime<Utc>,
    pub compute_units: u64,
    pub quality_score: f64,
    pub novelty_score: f64,
    pub peer_validation_score: f64,
    pub final_reward: u64,
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct RewardStats {
    pub total_rewards_distributed: u64,
    pub average_reward: u64,
    pub total_contributions: usize,
    pub average_quality_score: f64,
}

#[derive(Debug)]
pub struct NetworkDemandAnalysis {
    pub demand_multipliers: HashMap<ContributionType, f64>,
}

#[derive(Debug, Default)]
pub struct SupplyFactors {
    pub contributor_availability: f64,
    pub compute_capacity: f64,
}

#[derive(Debug, Default)]
pub struct DemandFactors {
    pub task_queue_size: f64,
    pub priority_tasks: f64,
}

#[derive(Debug)]
pub struct NetworkConditions {
    pub active_contributors: u64,
    pub target_contributors: u64,
    pub network_utilization: f64,
    pub average_task_completion_time: f64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_reward_calculation() {
        let mut engine = RewardEngine::new();
        engine.initialize().await.unwrap();

        let validation_result = ValidationResult {
            valid: true,
            compute_units: 1000,
            quality_score: 0.8,
            novelty_score: 0.6,
            peer_validation_score: 0.9,
        };

        let reputation = ReputationMetrics {
            consistency_score: 0.7,
            expertise_score: 0.8,
            trust_score: 0.9,
            contribution_count: 50,
            average_quality: 0.75,
        };

        let reward = engine
            .calculate_reward(&validation_result, reputation)
            .await
            .unwrap();
        assert!(reward > 0);
        assert!(reward < 10_000_000_000); // Should be less than 100 PAR

        let stats = engine.get_reward_stats();
        assert_eq!(stats.total_contributions, 1);
        assert_eq!(stats.total_rewards_distributed, reward);
    }

    #[tokio::test]
    async fn test_trust_network() {
        let mut trust_network = TrustNetwork::new();
        trust_network.initialize().await.unwrap();

        use ed25519_dalek::Keypair;
        use rand::thread_rng;

        let keypair1 = Keypair::generate(&mut thread_rng());
        let keypair2 = Keypair::generate(&mut thread_rng());
        let addr1 = Address::from_public_key(&keypair1.public);
        let addr2 = Address::from_public_key(&keypair2.public);

        trust_network
            .update_trust(&addr1, &addr2, 0.8)
            .await
            .unwrap();
        let trust_score = trust_network.get_trust_score(&addr2);
        assert!((trust_score - 0.8).abs() < 0.001);
    }
}
