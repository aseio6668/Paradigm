pub mod advanced_governance;
pub mod ai_agent_governance;
pub mod ai_optimizer;
pub mod analytics_api;
pub mod bridge_adapter;
pub mod contribution_validator;
/// Modular tokenomics system for Paradigm network
/// Implements advanced features like ZK proofs, reputation weighting,
/// temporal dynamics, and cross-platform interoperability
pub mod core_token;
pub mod decay_mechanism;
pub mod governance_module;
pub mod model_hosting;
pub mod network_analytics;
pub mod privacy_preserving;
pub mod quantum_resistant;
pub mod reputation_ledger;
pub mod reward_engine;
pub mod staking_module;
pub mod temporal_evolution;
pub mod treasury_manager;

#[allow(ambiguous_glob_reexports)]
pub use advanced_governance::*;
#[allow(ambiguous_glob_reexports)]
pub use ai_agent_governance::*;
#[allow(ambiguous_glob_reexports)]
pub use ai_optimizer::*;
#[allow(ambiguous_glob_reexports)]
pub use analytics_api::*;
#[allow(ambiguous_glob_reexports)]
pub use bridge_adapter::*;
#[allow(ambiguous_glob_reexports)]
pub use contribution_validator::*;
#[allow(ambiguous_glob_reexports)]
pub use core_token::*;
#[allow(ambiguous_glob_reexports)]
pub use decay_mechanism::*;
#[allow(ambiguous_glob_reexports)]
pub use governance_module::*;
#[allow(ambiguous_glob_reexports)]
pub use model_hosting::*;
#[allow(ambiguous_glob_reexports)]
pub use network_analytics::*;
#[allow(ambiguous_glob_reexports)]
pub use privacy_preserving::*;
#[allow(ambiguous_glob_reexports)]
pub use quantum_resistant::*;
pub use reputation_ledger::{ReputationLedger, ReputationMetrics as LedgerReputationMetrics};
pub use reward_engine::{NetworkConditions, RewardEngine, RewardStats};
#[allow(ambiguous_glob_reexports)]
pub use staking_module::*;
#[allow(ambiguous_glob_reexports)]
pub use temporal_evolution::*;
#[allow(ambiguous_glob_reexports)]
pub use treasury_manager::*;

use crate::Address;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Central tokenomics coordinator that manages all modules
#[derive(Debug)]
pub struct TokenomicsSystem {
    pub core_token: CoreToken,
    pub contribution_validator: ContributionValidator,
    pub reward_engine: RewardEngine,
    pub staking_module: StakingModule,
    pub governance_module: GovernanceModule,
    pub reputation_ledger: ReputationLedger,
    pub bridge_adapter: BridgeAdapter,
    pub treasury_manager: TreasuryManager,
    pub decay_mechanism: DecayMechanism,
    pub privacy_preserving: PrivacyPreserving,
    pub model_hosting: ModelHosting,
    pub ai_optimizer: AIOptimizer,
    pub quantum_resistant: QuantumResistantCrypto,
    pub advanced_governance: AdvancedGovernance,
    pub network_analytics: NetworkAnalyticsDashboard,
    pub ai_agent_governance: AIAgentGovernanceSystem,
    pub temporal_evolution: TemporalTokenEvolution,
}

impl TokenomicsSystem {
    pub fn new() -> Self {
        TokenomicsSystem {
            core_token: CoreToken::new(),
            contribution_validator: ContributionValidator::new(),
            reward_engine: RewardEngine::new(),
            staking_module: StakingModule::new(),
            governance_module: GovernanceModule::new(),
            reputation_ledger: ReputationLedger::new(),
            bridge_adapter: BridgeAdapter::new(),
            treasury_manager: TreasuryManager::new(),
            decay_mechanism: DecayMechanism::new(),
            privacy_preserving: PrivacyPreserving::new(),
            model_hosting: ModelHosting::new(),
            ai_optimizer: AIOptimizer::new(),
            quantum_resistant: QuantumResistantCrypto::new(),
            advanced_governance: AdvancedGovernance::new(),
            network_analytics: NetworkAnalyticsDashboard::new(),
            ai_agent_governance: AIAgentGovernanceSystem::new(),
            temporal_evolution: TemporalTokenEvolution::new(),
        }
    }

    pub async fn start(&mut self) -> anyhow::Result<()> {
        tracing::info!("Starting advanced tokenomics system");

        // Initialize all modules
        self.core_token.initialize().await?;
        self.contribution_validator.initialize().await?;
        self.reward_engine.initialize().await?;
        self.staking_module.initialize().await?;
        self.governance_module.initialize().await?;
        self.reputation_ledger.initialize().await?;
        self.bridge_adapter.initialize().await?;
        self.treasury_manager.initialize().await?;
        self.decay_mechanism.initialize().await?;
        self.privacy_preserving.initialize().await?;
        self.model_hosting.initialize().await?;
        self.ai_optimizer.initialize().await?;
        self.quantum_resistant.initialize().await?;
        self.advanced_governance.initialize().await?;
        self.network_analytics.initialize().await?;
        self.ai_agent_governance.initialize().await?;
        self.temporal_evolution.initialize().await?;

        tracing::info!("Advanced tokenomics system started successfully");
        Ok(())
    }

    /// Process a contribution and calculate rewards using all modules
    pub async fn process_contribution(
        &mut self,
        contributor: &Address,
        workload_proof: ContributionProof,
    ) -> anyhow::Result<ContributionResult> {
        // 1. Validate contribution using ZK proofs
        let validation_result = self
            .contribution_validator
            .validate_contribution(contributor, &workload_proof)
            .await?;

        // 2. Check reputation and calculate multipliers
        let ledger_reputation = self.reputation_ledger.get_reputation(contributor).await?;

        // Convert to reward engine format
        let reputation = reward_engine::ReputationMetrics {
            consistency_score: ledger_reputation.consistency_score,
            expertise_score: ledger_reputation.expertise_score,
            trust_score: ledger_reputation.trust_score,
            contribution_count: ledger_reputation.contribution_count,
            average_quality: ledger_reputation.average_quality,
        };

        // 3. Calculate reward using advanced engine
        let reward = self
            .reward_engine
            .calculate_reward(&validation_result, reputation)
            .await?;

        // 4. Apply temporal dynamics (decay/evolution)
        let adjusted_reward = self
            .decay_mechanism
            .apply_temporal_dynamics(contributor, reward)
            .await?;

        // 5. Mint tokens through core token system
        let tokens_minted = self
            .core_token
            .mint_tokens(contributor, adjusted_reward)
            .await?;

        // 6. Update reputation based on contribution quality
        self.reputation_ledger
            .update_reputation(contributor, &validation_result)
            .await?;

        // 7. Record for governance and treasury management
        self.treasury_manager
            .record_contribution(contributor, &validation_result, tokens_minted)
            .await?;

        // 8. Record contribution event in analytics
        self.network_analytics
            .record_contribution(
                contributor,
                workload_proof.contribution_type.clone(),
                tokens_minted,
            )
            .await?;

        // 9. Record contribution for temporal evolution
        self.temporal_evolution
            .record_contribution_for_evolution(contributor, &workload_proof, tokens_minted)
            .await
            .map_err(|e| anyhow::anyhow!("Failed to record contribution for evolution: {:?}", e))?;

        Ok(ContributionResult {
            contributor: contributor.clone(),
            tokens_earned: tokens_minted,
            reputation_change: validation_result.quality_score,
            timestamp: Utc::now(),
            contribution_type: workload_proof.contribution_type,
        })
    }

    /// Generate quantum-resistant keys for a contributor
    pub async fn generate_quantum_resistant_keys(
        &mut self,
        contributor: &Address,
    ) -> anyhow::Result<ContributorKeys> {
        self.quantum_resistant
            .generate_contributor_keys(contributor)
            .await
    }

    /// Create quantum-resistant contribution proof
    pub async fn create_quantum_resistant_proof(
        &mut self,
        contributor: &Address,
        contribution_type: ContributionType,
        workload_data: &[u8],
        metadata: serde_json::Value,
    ) -> anyhow::Result<ContributionProof> {
        let proof_id = Uuid::new_v4();
        let workload_hash = self.compute_workload_hash(workload_data);

        // Generate quantum-resistant ZK proof
        let proof_type = self.get_proof_type_for_contribution(&contribution_type);
        let private_inputs = QRPrivateInputs {
            witness: workload_data.to_vec(),
            randomness: self.generate_secure_randomness().await?,
        };
        let public_inputs = QRPublicInputs {
            statement: workload_hash.clone(),
            challenge: self.generate_challenge(&workload_hash).await?,
        };

        let qr_zk_proof = self
            .quantum_resistant
            .generate_qr_zk_proof(&proof_type, &private_inputs, &public_inputs)
            .await?;

        // Generate quantum-resistant signature
        let proof_data = serde_json::to_vec(&qr_zk_proof)?;
        let qr_signature = self
            .quantum_resistant
            .sign_contribution_proof(contributor, &proof_data, QRSignatureType::Lattice)
            .await?;

        Ok(ContributionProof {
            id: proof_id,
            contributor: contributor.clone(),
            contribution_type,
            workload_hash,
            zk_proof: vec![], // Empty for quantum-resistant mode
            qr_zk_proof: Some(qr_zk_proof),
            qr_signature: Some(qr_signature),
            metadata,
            timestamp: Utc::now(),
        })
    }

    /// Verify quantum-resistant contribution proof
    pub async fn verify_quantum_resistant_proof(
        &self,
        proof: &ContributionProof,
    ) -> anyhow::Result<bool> {
        // Verify quantum-resistant ZK proof
        if let Some(qr_zk_proof) = &proof.qr_zk_proof {
            let zk_valid = self
                .quantum_resistant
                .verify_qr_zk_proof(qr_zk_proof)
                .await?;
            if !zk_valid {
                return Ok(false);
            }
        }

        // Verify quantum-resistant signature
        if let Some(qr_signature) = &proof.qr_signature {
            if let Some(qr_zk_proof) = &proof.qr_zk_proof {
                let proof_data = serde_json::to_vec(qr_zk_proof)?;
                let sig_valid = self
                    .quantum_resistant
                    .verify_signature(&proof.contributor, &proof_data, qr_signature)
                    .await?;
                if !sig_valid {
                    return Ok(false);
                }
            }
        }

        Ok(true)
    }

    /// Perform quantum-safe key exchange with peer
    pub async fn quantum_safe_key_exchange(
        &mut self,
        peer: &Address,
    ) -> anyhow::Result<SharedSecret> {
        self.quantum_resistant.quantum_safe_key_exchange(peer).await
    }

    /// Generate quantum random value for governance
    pub async fn generate_quantum_governance_random(
        &mut self,
        proposal_id: &str,
    ) -> anyhow::Result<QuantumRandom> {
        let entropy_sources = vec![
            format!("proposal_{}", proposal_id),
            format!("timestamp_{}", Utc::now().timestamp()),
            "network_entropy".to_string(),
            "block_hash_entropy".to_string(),
        ];

        self.quantum_resistant
            .get_quantum_random(entropy_sources)
            .await
    }

    // Private helper methods
    fn compute_workload_hash(&self, data: &[u8]) -> Vec<u8> {
        // Simplified hash computation
        data.iter()
            .map(|&b| b.wrapping_mul(31).wrapping_add(17))
            .collect()
    }

    async fn generate_secure_randomness(&self) -> anyhow::Result<Vec<u8>> {
        // Generate cryptographically secure randomness
        Ok(vec![42u8; 32]) // Simplified for demonstration
    }

    async fn generate_challenge(&self, statement: &[u8]) -> anyhow::Result<Vec<u8>> {
        // Generate Fiat-Shamir challenge
        let mut challenge = Vec::new();
        for &byte in statement.iter().take(16) {
            challenge.push(byte.wrapping_add(127));
        }
        Ok(challenge)
    }

    fn get_proof_type_for_contribution(&self, contribution_type: &ContributionType) -> String {
        match contribution_type {
            ContributionType::MLTraining => "ml_training",
            ContributionType::InferenceServing => "inference_serving",
            ContributionType::DataValidation => "data_validation",
            ContributionType::ModelOptimization => "model_optimization",
            ContributionType::NetworkMaintenance => "network_maintenance",
            ContributionType::GovernanceParticipation => "governance_participation",
            ContributionType::CrossPlatformCompute => "ml_training", // Use ML training proof
            ContributionType::StorageProvision => "data_validation", // Use data validation proof
            ContributionType::GenerativeMedia => "ml_training",      // Use ML training proof
            ContributionType::SymbolicMath => "model_optimization",  // Use optimization proof
            ContributionType::Simulation => "ml_training",           // Use ML training proof
            ContributionType::MediaGeneration => "ml_training",      // Use ML training proof
        }
        .to_string()
    }

    /// Create quadratic voting proposal with quantum randomness
    pub async fn create_governance_proposal(
        &mut self,
        proposer: &Address,
        title: String,
        description: String,
        proposal_type: ProposalType,
        expected_impact: ExpectedImpact,
    ) -> anyhow::Result<Uuid> {
        // Generate quantum randomness for the proposal
        let quantum_random = self.generate_quantum_governance_random(&title).await?;

        let proposal_data = QuadraticProposalData {
            title,
            description,
            proposal_type,
            implementation_details: "Implementation details to be provided".to_string(),
            expected_impact,
            required_quorum: 0.1, // 10% participation required
        };

        self.advanced_governance
            .create_quadratic_proposal(proposer.clone(), proposal_data, quantum_random)
            .await
    }

    /// Cast quadratic vote on governance proposal
    pub async fn cast_governance_vote(
        &mut self,
        voter: &Address,
        proposal_id: Uuid,
        vote_strength: f64,
        max_cost: u64,
    ) -> anyhow::Result<QuadraticVoteResult> {
        self.advanced_governance
            .cast_quadratic_vote(voter.clone(), proposal_id, vote_strength, max_cost)
            .await
    }

    /// Create futarchy prediction market for complex decisions
    pub async fn create_prediction_market(
        &mut self,
        proposer: &Address,
        proposal_title: String,
        proposal_description: String,
        success_metrics: Vec<SuccessMetric>,
    ) -> anyhow::Result<Uuid> {
        let proposal_data = FutarchyProposalData {
            title: proposal_title,
            description: proposal_description,
            implementation_plan: "Detailed implementation plan".to_string(),
            success_criteria: vec!["Network performance improvement".to_string()],
            risk_assessment: RiskAssessment {
                technical_risk: advanced_governance::RiskLevel::Medium,
                economic_risk: advanced_governance::RiskLevel::Low,
                adoption_risk: advanced_governance::RiskLevel::Medium,
                timeline_risk: advanced_governance::RiskLevel::Low,
            },
        };

        self.advanced_governance
            .create_futarchy_market(proposer.clone(), proposal_data, success_metrics)
            .await
    }

    /// Start conviction voting for funding proposals
    pub async fn start_conviction_funding(
        &mut self,
        proposer: &Address,
        title: String,
        description: String,
        deliverables: Vec<String>,
        funding_requested: u64,
    ) -> anyhow::Result<Uuid> {
        let proposal_data = ConvictionProposalData {
            title,
            description,
            deliverables,
            timeline: "3-6 months".to_string(),
        };

        self.advanced_governance
            .start_conviction_voting(proposer.clone(), proposal_data, funding_requested)
            .await
    }

    /// Signal conviction for funding proposal
    pub async fn signal_funding_conviction(
        &mut self,
        supporter: &Address,
        proposal_id: Uuid,
        token_amount: u64,
    ) -> anyhow::Result<ConvictionSignalResult> {
        self.advanced_governance
            .signal_conviction(supporter.clone(), proposal_id, token_amount)
            .await
    }

    /// Delegate voting power to another participant
    pub async fn delegate_voting_power(
        &mut self,
        delegator: &Address,
        delegatee: &Address,
        delegation_type: DelegationType,
        duration_days: u32,
    ) -> anyhow::Result<DelegationResult> {
        let expiry = Utc::now() + chrono::Duration::days(duration_days as i64);

        self.advanced_governance
            .delegate_voting_power(
                delegator.clone(),
                delegatee.clone(),
                delegation_type,
                expiry,
            )
            .await
    }

    /// Get comprehensive governance statistics
    pub async fn get_governance_statistics(&self) -> anyhow::Result<GovernanceStats> {
        self.advanced_governance.get_governance_stats().await
    }

    /// Update network analytics with current system state
    pub async fn update_network_analytics(&mut self) -> anyhow::Result<()> {
        let network_state = self.get_current_network_state().await?;
        self.network_analytics
            .update_network_state(&network_state)
            .await
            .map_err(|e| anyhow::anyhow!("Failed to update network analytics: {:?}", e))?;
        Ok(())
    }

    /// Get real-time dashboard data
    pub async fn get_dashboard_data(&self) -> anyhow::Result<DashboardData> {
        self.network_analytics
            .get_dashboard_data()
            .await
            .map_err(|e| anyhow::anyhow!("Failed to get dashboard data: {:?}", e))
    }

    /// Generate analytics report for specified timeframe
    pub async fn generate_analytics_report(
        &self,
        timeframe: TimeFrame,
    ) -> anyhow::Result<AnalyticsReport> {
        self.network_analytics
            .generate_report(timeframe)
            .await
            .map_err(|e| anyhow::anyhow!("Failed to generate analytics report: {:?}", e))
    }

    /// Get current network state for analytics
    async fn get_current_network_state(&self) -> anyhow::Result<NetworkState> {
        // This would typically gather data from various system components
        // For now, we'll create a representative state
        Ok(NetworkState {
            total_supply: 1000000,                   // This would come from core_token
            active_participants: 100,                // This would come from reputation_ledger
            transaction_volume: 50000,               // This would come from transaction data
            transaction_throughput: 100.0,           // Calculated from recent performance
            uptime_percentage: 0.99,                 // System uptime
            avg_consensus_time: 2.5,                 // Average block time
            error_rate: 0.01,                        // Error rate from monitoring
            resource_utilization: 0.75,              // System resource usage
            token_velocity: 3.2,                     // Economic metric
            network_growth: 0.15,                    // Growth rate
            inflation_rate: 0.05,                    // Current inflation
            top_10_validator_stake_percentage: 0.35, // Decentralization metric
            geographic_diversity_index: 0.8,         // Geographic distribution
            wealth_concentration_gini: 0.4,          // Wealth distribution
            treasury_balance: 500000,                // Treasury holdings
            mint_rate: 0.02,                         // Token minting rate
            burn_rate: 0.01,                         // Token burning rate
            total_rewards: 10000,                    // Total rewards distributed
            productive_work_rewards: 8000,           // Rewards for productive work
            avg_transaction_fee: 0.01,               // Average transaction cost
            contributor_satisfaction_score: 0.85,    // User satisfaction
            governance_participation_rate: 0.6,      // Governance engagement
            monthly_active_user_retention: 0.8,      // User retention
        })
    }

    /// Get AI-generated governance proposals
    pub async fn get_ai_generated_proposals(&mut self) -> anyhow::Result<Vec<AIGeneratedProposal>> {
        let network_state = self.get_current_network_state().await?;
        self.ai_agent_governance
            .generate_ai_proposals(&network_state)
            .await
            .map_err(|e| anyhow::anyhow!("Failed to generate AI proposals: {:?}", e))
    }

    /// Get AI agent analysis of a proposal
    pub async fn get_ai_agent_analysis(
        &mut self,
        proposal_id: Uuid,
        proposal_data: &ProposalData,
    ) -> anyhow::Result<Vec<ai_agent_governance::AIAgentVote>> {
        self.ai_agent_governance
            .agents_analyze_proposal(proposal_id, proposal_data)
            .await
            .map_err(|e| anyhow::anyhow!("Failed to get AI agent analysis: {:?}", e))
    }

    /// Predict consensus likelihood using AI
    pub async fn predict_proposal_consensus(
        &self,
        proposal_data: &ProposalData,
        current_votes: &[HumanVote],
    ) -> anyhow::Result<ConsensusPrediction> {
        self.ai_agent_governance
            .predict_consensus(proposal_data, current_votes)
            .await
            .map_err(|e| anyhow::anyhow!("Failed to predict consensus: {:?}", e))
    }

    /// Submit human feedback for AI agents
    pub async fn submit_agent_feedback(
        &mut self,
        agent_id: Uuid,
        feedback: HumanFeedback,
    ) -> anyhow::Result<()> {
        self.ai_agent_governance
            .process_human_feedback(agent_id, feedback)
            .await
            .map_err(|e| anyhow::anyhow!("Failed to process agent feedback: {:?}", e))
    }

    /// Get AI agent governance statistics
    pub async fn get_ai_agent_statistics(&self) -> anyhow::Result<AgentGovernanceStats> {
        self.ai_agent_governance
            .get_agent_statistics()
            .await
            .map_err(|e| anyhow::anyhow!("Failed to get agent statistics: {:?}", e))
    }

    /// Evolve AI agents based on performance
    pub async fn evolve_ai_agents(&mut self) -> anyhow::Result<EvolutionReport> {
        self.ai_agent_governance
            .evolve_agents()
            .await
            .map_err(|e| anyhow::anyhow!("Failed to evolve AI agents: {:?}", e))
    }

    /// Register a new AI governance agent
    pub async fn register_ai_agent(&mut self, agent: AIGovernanceAgent) -> anyhow::Result<Uuid> {
        self.ai_agent_governance
            .register_agent(agent)
            .await
            .map_err(|e| anyhow::anyhow!("Failed to register AI agent: {:?}", e))
    }

    /// Get all AI governance agents
    pub async fn get_all_ai_agents(&self) -> anyhow::Result<Vec<AIGovernanceAgent>> {
        self.ai_agent_governance
            .get_all_agents()
            .await
            .map_err(|e| anyhow::anyhow!("Failed to get AI agents: {:?}", e))
    }

    /// Evolve tokens for a specific address
    pub async fn evolve_tokens(&mut self, address: &Address) -> anyhow::Result<EvolutionResult> {
        let network_state = self.get_current_network_state().await?;
        self.temporal_evolution
            .evolve_tokens(address, &network_state)
            .await
            .map_err(|e| anyhow::anyhow!("Failed to evolve tokens: {:?}", e))
    }

    /// Stake tokens with temporal evolution benefits
    pub async fn stake_tokens_temporal(
        &mut self,
        address: &Address,
        amount: u64,
        staking_type: TemporalStakingType,
    ) -> anyhow::Result<StakingResult> {
        self.temporal_evolution
            .stake_tokens(address, amount, staking_type)
            .await
            .map_err(|e| anyhow::anyhow!("Failed to stake tokens: {:?}", e))
    }

    /// Unstake tokens from temporal staking
    pub async fn unstake_tokens_temporal(
        &mut self,
        address: &Address,
        stake_id: Uuid,
    ) -> anyhow::Result<UnstakingResult> {
        self.temporal_evolution
            .unstake_tokens(address, stake_id)
            .await
            .map_err(|e| anyhow::anyhow!("Failed to unstake tokens: {:?}", e))
    }

    /// Get token evolution state for an address
    pub async fn get_token_evolution_state(
        &self,
        address: &Address,
    ) -> anyhow::Result<Option<TokenState>> {
        self.temporal_evolution
            .get_token_state(address)
            .await
            .map_err(|e| anyhow::anyhow!("Failed to get token state: {:?}", e))
    }

    /// Get temporal staking information
    pub async fn get_temporal_staking_info(
        &self,
        address: &Address,
    ) -> anyhow::Result<Vec<TemporalStake>> {
        self.temporal_evolution
            .get_staking_info(address)
            .await
            .map_err(|e| anyhow::anyhow!("Failed to get staking info: {:?}", e))
    }

    /// Predict token evolution for future timeframe
    pub async fn predict_token_evolution(
        &self,
        address: &Address,
        time_horizon: chrono::Duration,
    ) -> anyhow::Result<EvolutionPrediction> {
        self.temporal_evolution
            .predict_evolution(address, time_horizon)
            .await
            .map_err(|e| anyhow::anyhow!("Failed to predict evolution: {:?}", e))
    }

    /// Process global token evolution for all addresses
    pub async fn process_global_token_evolution(
        &mut self,
    ) -> anyhow::Result<GlobalEvolutionResult> {
        let network_state = self.get_current_network_state().await?;
        self.temporal_evolution
            .process_global_evolution(&network_state)
            .await
            .map_err(|e| anyhow::anyhow!("Failed to process global evolution: {:?}", e))
    }

    /// Get temporal evolution metrics
    pub async fn get_evolution_metrics(
        &self,
    ) -> anyhow::Result<temporal_evolution::EvolutionMetrics> {
        self.temporal_evolution
            .get_evolution_metrics()
            .await
            .map_err(|e| anyhow::anyhow!("Failed to get evolution metrics: {:?}", e))
    }
}

/// Result of processing a contribution
#[derive(Debug, Serialize, Deserialize)]
pub struct ContributionResult {
    pub contributor: Address,
    pub tokens_earned: u64,
    pub reputation_change: f64,
    pub timestamp: DateTime<Utc>,
    pub contribution_type: ContributionType,
}

/// Types of contributions supported
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum ContributionType {
    MLTraining,
    InferenceServing,
    DataValidation,
    ModelOptimization,
    NetworkMaintenance,
    GovernanceParticipation,
    CrossPlatformCompute,
    StorageProvision,
    GenerativeMedia,
    SymbolicMath,
    Simulation,
    MediaGeneration,
}

/// Proof of contribution with quantum-resistant ZK verification
#[derive(Debug, Serialize, Deserialize)]
pub struct ContributionProof {
    pub id: Uuid,
    pub contributor: Address,
    pub contribution_type: ContributionType,
    pub workload_hash: Vec<u8>,
    pub zk_proof: Vec<u8>, // Traditional ZK proof for backward compatibility
    pub qr_zk_proof: Option<QuantumResistantZKProof>, // Quantum-resistant ZK proof
    pub qr_signature: Option<QuantumResistantSignature>, // Quantum-resistant signature
    pub metadata: serde_json::Value,
    pub timestamp: DateTime<Utc>,
}

/// Validation result from contribution validator
#[derive(Debug)]
pub struct ValidationResult {
    pub valid: bool,
    pub compute_units: u64,
    pub quality_score: f64,         // 0.0 to 1.0
    pub novelty_score: f64,         // 0.0 to 1.0
    pub peer_validation_score: f64, // 0.0 to 1.0
}
