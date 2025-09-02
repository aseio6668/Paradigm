use super::{ContributionType, ValidationResult};
use crate::Address;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

/// Autonomous treasury that funds projects based on community voting and AI-curated impact metrics
#[derive(Debug)]
pub struct TreasuryManager {
    /// Current treasury balance in PAR tokens
    balance: u64,
    /// Active funding proposals
    active_proposals: HashMap<Uuid, FundingProposal>,
    /// Completed grants and their outcomes
    grant_history: Vec<CompletedGrant>,
    /// AI curator for proposal evaluation
    ai_curator: AICurator,
    /// Community voting system
    voting_system: CommunityVoting,
    /// Impact tracking for funded projects
    impact_tracker: ImpactTracker,
    /// Treasury allocation rules
    allocation_rules: AllocationRules,
}

impl TreasuryManager {
    pub fn new() -> Self {
        TreasuryManager {
            balance: 0,
            active_proposals: HashMap::new(),
            grant_history: Vec::new(),
            ai_curator: AICurator::new(),
            voting_system: CommunityVoting::new(),
            impact_tracker: ImpactTracker::new(),
            allocation_rules: AllocationRules::default(),
        }
    }

    pub async fn initialize(&mut self) -> anyhow::Result<()> {
        tracing::info!("Initializing Treasury Manager");

        // Set initial treasury balance (could come from token genesis)
        self.balance = 100_000_000_000_000; // 1M PAR initial treasury

        // Initialize AI curator
        self.ai_curator.initialize().await?;

        // Initialize voting system
        self.voting_system.initialize().await?;

        // Initialize impact tracker
        self.impact_tracker.initialize().await?;

        tracing::info!(
            "Treasury initialized with {} PAR",
            self.balance as f64 / 100_000_000.0
        );
        Ok(())
    }

    /// Record a contribution for treasury accounting
    pub async fn record_contribution(
        &mut self,
        contributor: &Address,
        validation_result: &ValidationResult,
        tokens_minted: u64,
    ) -> anyhow::Result<()> {
        // Treasury receives a small percentage of minted tokens for sustainability
        let treasury_fee = tokens_minted / 100; // 1% fee
        self.balance += treasury_fee;

        tracing::debug!(
            "Treasury fee collected: {} PAR from contribution",
            treasury_fee as f64 / 100_000_000.0
        );
        Ok(())
    }

    /// Submit a funding proposal to the treasury
    pub async fn submit_funding_proposal(
        &mut self,
        proposer: Address,
        proposal: FundingProposalRequest,
    ) -> anyhow::Result<Uuid> {
        let proposal_id = Uuid::new_v4();

        // Create full proposal with AI curation
        let mut funding_proposal = FundingProposal {
            id: proposal_id,
            proposer,
            title: proposal.title,
            description: proposal.description,
            requested_amount: proposal.requested_amount,
            category: proposal.category,
            milestones: proposal.milestones,
            submitted_at: Utc::now(),
            voting_deadline: Utc::now() + chrono::Duration::days(14),
            ai_evaluation: None,
            community_votes: CommunityVotes::default(),
            status: ProposalStatus::UnderReview,
            impact_metrics: ImpactMetrics::default(),
        };

        // Get AI evaluation
        funding_proposal.ai_evaluation =
            Some(self.ai_curator.evaluate_proposal(&funding_proposal).await?);

        // Add to active proposals
        self.active_proposals.insert(proposal_id, funding_proposal);

        tracing::info!(
            "New funding proposal submitted: {} - {} PAR requested",
            proposal_id,
            proposal.requested_amount as f64 / 100_000_000.0
        );

        Ok(proposal_id)
    }

    /// Vote on a funding proposal
    pub async fn vote_on_proposal(
        &mut self,
        proposal_id: Uuid,
        voter: Address,
        vote: Vote,
        voting_power: u64, // Based on staked tokens or reputation
    ) -> anyhow::Result<()> {
        if let Some(proposal) = self.active_proposals.get_mut(&proposal_id) {
            if proposal.status != ProposalStatus::Active {
                return Err(anyhow::anyhow!("Proposal not active for voting"));
            }

            if Utc::now() > proposal.voting_deadline {
                return Err(anyhow::anyhow!("Voting deadline has passed"));
            }

            // Record vote
            self.voting_system
                .record_vote(proposal_id, voter, vote, voting_power)
                .await?;

            // Update proposal votes
            match vote {
                Vote::For => proposal.community_votes.for_votes += voting_power,
                Vote::Against => proposal.community_votes.against_votes += voting_power,
                Vote::Abstain => proposal.community_votes.abstain_votes += voting_power,
            }

            tracing::debug!(
                "Vote recorded for proposal {}: {:?} with power {}",
                proposal_id,
                vote,
                voting_power
            );
        } else {
            return Err(anyhow::anyhow!("Proposal not found"));
        }

        Ok(())
    }

    /// Process proposals that have reached their voting deadline
    pub async fn process_completed_votes(&mut self) -> anyhow::Result<Vec<Uuid>> {
        let mut processed_proposals = Vec::new();
        let now = Utc::now();

        // Collect proposal IDs that need processing
        let mut proposal_ids_to_process = Vec::new();
        for (proposal_id, proposal) in self.active_proposals.iter() {
            if proposal.status == ProposalStatus::Active && now > proposal.voting_deadline {
                proposal_ids_to_process.push(*proposal_id);
            }
        }

        // Process each proposal
        for proposal_id in proposal_ids_to_process {
            // Get proposal clone for decision calculation
            let proposal_clone = if let Some(proposal) = self.active_proposals.get(&proposal_id) {
                proposal.clone()
            } else {
                continue;
            };

            // Calculate final decision
            let decision = self.calculate_funding_decision(&proposal_clone).await?;

            // Update the proposal in the active_proposals map
            if let Some(stored_proposal) = self.active_proposals.get_mut(&proposal_id) {
                match decision {
                    FundingDecision::Approved { amount } => {
                        if self.balance >= amount {
                            stored_proposal.status = ProposalStatus::Approved;
                            self.balance -= amount;

                            tracing::info!(
                                "Proposal {} approved for {} PAR",
                                proposal_id,
                                amount as f64 / 100_000_000.0
                            );
                        } else {
                            stored_proposal.status = ProposalStatus::RejectedInsufficientFunds;
                            tracing::warn!(
                                "Proposal {} rejected - insufficient treasury funds",
                                proposal_id
                            );
                        }
                    }
                    FundingDecision::Rejected => {
                        stored_proposal.status = ProposalStatus::RejectedByVote;
                        tracing::info!("Proposal {} rejected by community vote", proposal_id);
                    }
                }
            }

            processed_proposals.push(proposal_id);
        }

        Ok(processed_proposals)
    }

    /// Calculate funding decision based on AI evaluation and community votes
    async fn calculate_funding_decision(
        &self,
        proposal: &FundingProposal,
    ) -> anyhow::Result<FundingDecision> {
        let total_votes = proposal.community_votes.for_votes
            + proposal.community_votes.against_votes
            + proposal.community_votes.abstain_votes;

        if total_votes == 0 {
            return Ok(FundingDecision::Rejected);
        }

        // Community voting threshold (60% for approval)
        let community_approval_rate = proposal.community_votes.for_votes as f64
            / (proposal.community_votes.for_votes + proposal.community_votes.against_votes) as f64;

        // AI evaluation weight (30% of decision)
        let ai_score = proposal
            .ai_evaluation
            .as_ref()
            .map(|eval| eval.overall_score)
            .unwrap_or(0.5);

        // Combined score: 70% community + 30% AI
        let combined_score = (community_approval_rate * 0.7) + (ai_score * 0.3);

        // Apply treasury allocation rules
        let max_allocation = self.calculate_max_allocation(&proposal.category);
        let final_amount = proposal.requested_amount.min(max_allocation);

        if combined_score >= 0.6 && self.balance >= final_amount {
            Ok(FundingDecision::Approved {
                amount: final_amount,
            })
        } else {
            Ok(FundingDecision::Rejected)
        }
    }

    /// Calculate maximum allocation for a funding category
    fn calculate_max_allocation(&self, category: &FundingCategory) -> u64 {
        match category {
            FundingCategory::Research => self.balance / 20, // 5% max for research
            FundingCategory::Infrastructure => self.balance / 10, // 10% max for infrastructure
            FundingCategory::Community => self.balance / 50, // 2% max for community
            FundingCategory::Security => self.balance / 5,  // 20% max for security
            FundingCategory::Innovation => self.balance / 25, // 4% max for innovation
        }
    }

    /// Track impact of funded projects
    pub async fn update_project_impact(
        &mut self,
        proposal_id: Uuid,
        impact_update: ImpactUpdate,
    ) -> anyhow::Result<()> {
        self.impact_tracker
            .update_impact(proposal_id, impact_update)
            .await?;
        Ok(())
    }

    /// Get treasury statistics
    pub fn get_treasury_stats(&self) -> TreasuryStats {
        TreasuryStats {
            balance: self.balance,
            active_proposals: self.active_proposals.len(),
            total_grants_awarded: self.grant_history.len(),
            total_amount_granted: self.grant_history.iter().map(|g| g.amount).sum(),
        }
    }

    /// Get all active proposals
    pub fn get_active_proposals(&self) -> Vec<&FundingProposal> {
        self.active_proposals.values().collect()
    }

    /// Set treasury balance (used for genesis initialization)
    pub fn set_balance(&mut self, balance: u64) {
        self.balance = balance;
    }
}

/// AI curator for evaluating funding proposals
#[derive(Debug)]
pub struct AICurator {
    evaluation_models: Vec<String>,
}

impl AICurator {
    pub fn new() -> Self {
        AICurator {
            evaluation_models: vec![
                "impact_prediction".to_string(),
                "feasibility_analysis".to_string(),
                "risk_assessment".to_string(),
            ],
        }
    }

    pub async fn initialize(&mut self) -> anyhow::Result<()> {
        tracing::debug!(
            "Initializing AI Curator with {} models",
            self.evaluation_models.len()
        );
        Ok(())
    }

    pub async fn evaluate_proposal(
        &self,
        proposal: &FundingProposal,
    ) -> anyhow::Result<AIEvaluation> {
        // In a real implementation, this would use ML models to evaluate proposals

        // Simulate AI evaluation based on proposal characteristics
        let impact_score = self.evaluate_impact_potential(proposal).await?;
        let feasibility_score = self.evaluate_feasibility(proposal).await?;
        let risk_score = self.evaluate_risk(proposal).await?;

        let overall_score = (impact_score + feasibility_score + (1.0 - risk_score)) / 3.0;

        Ok(AIEvaluation {
            overall_score,
            impact_score,
            feasibility_score,
            risk_score,
            confidence: 0.8, // AI confidence in evaluation
            reasoning: format!(
                "Impact: {:.2}, Feasibility: {:.2}, Risk: {:.2}",
                impact_score, feasibility_score, risk_score
            ),
            evaluated_at: Utc::now(),
        })
    }

    async fn evaluate_impact_potential(&self, proposal: &FundingProposal) -> anyhow::Result<f64> {
        // Analyze potential impact based on category and description
        let base_score = match proposal.category {
            FundingCategory::Research => 0.7,
            FundingCategory::Infrastructure => 0.8,
            FundingCategory::Security => 0.9,
            FundingCategory::Innovation => 0.6,
            FundingCategory::Community => 0.5,
        };

        // Adjust based on proposal size
        let size_multiplier = if proposal.requested_amount > 10_000_000_000 {
            0.9
        } else {
            1.0
        };

        Ok(base_score * size_multiplier)
    }

    async fn evaluate_feasibility(&self, proposal: &FundingProposal) -> anyhow::Result<f64> {
        // Evaluate feasibility based on milestones and timeline
        let milestone_score = if proposal.milestones.len() >= 3 {
            0.8
        } else {
            0.6
        };

        // Consider funding amount vs market rates
        let funding_reasonableness = if proposal.requested_amount < 100_000_000_000 {
            0.9
        } else {
            0.7
        };

        Ok((milestone_score + funding_reasonableness) / 2.0)
    }

    async fn evaluate_risk(&self, _proposal: &FundingProposal) -> anyhow::Result<f64> {
        // Evaluate risks associated with the proposal
        // Higher risk score means more risky
        Ok(0.3) // Simulated low risk
    }
}

/// Community voting system
#[derive(Debug)]
pub struct CommunityVoting {
    votes: HashMap<Uuid, HashMap<Address, (Vote, u64)>>, // proposal_id -> (voter -> (vote, power))
}

impl CommunityVoting {
    pub fn new() -> Self {
        CommunityVoting {
            votes: HashMap::new(),
        }
    }

    pub async fn initialize(&mut self) -> anyhow::Result<()> {
        tracing::debug!("Initializing Community Voting system");
        Ok(())
    }

    pub async fn record_vote(
        &mut self,
        proposal_id: Uuid,
        voter: Address,
        vote: Vote,
        voting_power: u64,
    ) -> anyhow::Result<()> {
        self.votes
            .entry(proposal_id)
            .or_insert_with(HashMap::new)
            .insert(voter, (vote, voting_power));
        Ok(())
    }
}

/// Impact tracking system
#[derive(Debug)]
pub struct ImpactTracker {
    impact_records: HashMap<Uuid, Vec<ImpactUpdate>>,
}

impl ImpactTracker {
    pub fn new() -> Self {
        ImpactTracker {
            impact_records: HashMap::new(),
        }
    }

    pub async fn initialize(&mut self) -> anyhow::Result<()> {
        tracing::debug!("Initializing Impact Tracker");
        Ok(())
    }

    pub async fn update_impact(
        &mut self,
        proposal_id: Uuid,
        impact_update: ImpactUpdate,
    ) -> anyhow::Result<()> {
        self.impact_records
            .entry(proposal_id)
            .or_insert_with(Vec::new)
            .push(impact_update);
        Ok(())
    }
}

/// Treasury allocation rules
#[derive(Debug)]
pub struct AllocationRules {
    pub max_single_grant_percentage: f64,
    pub category_limits: HashMap<FundingCategory, f64>,
    pub monthly_spending_cap: u64,
}

impl Default for AllocationRules {
    fn default() -> Self {
        let mut category_limits = HashMap::new();
        category_limits.insert(FundingCategory::Research, 0.05);
        category_limits.insert(FundingCategory::Infrastructure, 0.10);
        category_limits.insert(FundingCategory::Security, 0.20);
        category_limits.insert(FundingCategory::Innovation, 0.04);
        category_limits.insert(FundingCategory::Community, 0.02);

        AllocationRules {
            max_single_grant_percentage: 0.05, // 5% of treasury max
            category_limits,
            monthly_spending_cap: 50_000_000_000, // 500 PAR per month
        }
    }
}

// Data structures

#[derive(Debug, Serialize, Deserialize)]
pub struct FundingProposal {
    pub id: Uuid,
    pub proposer: Address,
    pub title: String,
    pub description: String,
    pub requested_amount: u64,
    pub category: FundingCategory,
    pub milestones: Vec<Milestone>,
    pub submitted_at: DateTime<Utc>,
    pub voting_deadline: DateTime<Utc>,
    pub ai_evaluation: Option<AIEvaluation>,
    pub community_votes: CommunityVotes,
    pub status: ProposalStatus,
    pub impact_metrics: ImpactMetrics,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FundingProposalRequest {
    pub title: String,
    pub description: String,
    pub requested_amount: u64,
    pub category: FundingCategory,
    pub milestones: Vec<Milestone>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum FundingCategory {
    Research,
    Infrastructure,
    Community,
    Security,
    Innovation,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Milestone {
    pub description: String,
    pub deadline: DateTime<Utc>,
    pub funding_percentage: f64,
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub enum ProposalStatus {
    UnderReview,
    Active,
    Approved,
    RejectedByVote,
    RejectedInsufficientFunds,
    Completed,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AIEvaluation {
    pub overall_score: f64,
    pub impact_score: f64,
    pub feasibility_score: f64,
    pub risk_score: f64,
    pub confidence: f64,
    pub reasoning: String,
    pub evaluated_at: DateTime<Utc>,
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct CommunityVotes {
    pub for_votes: u64,
    pub against_votes: u64,
    pub abstain_votes: u64,
}

#[derive(Debug, Copy, Clone, Serialize, Deserialize)]
pub enum Vote {
    For,
    Against,
    Abstain,
}

#[derive(Debug)]
pub enum FundingDecision {
    Approved { amount: u64 },
    Rejected,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CompletedGrant {
    pub proposal_id: Uuid,
    pub amount: u64,
    pub completed_at: DateTime<Utc>,
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct ImpactMetrics {
    pub users_affected: u64,
    pub performance_improvement: f64,
    pub cost_savings: u64,
    pub network_growth: f64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ImpactUpdate {
    pub metric_type: String,
    pub value: f64,
    pub timestamp: DateTime<Utc>,
    pub description: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TreasuryStats {
    pub balance: u64,
    pub active_proposals: usize,
    pub total_grants_awarded: usize,
    pub total_amount_granted: u64,
}

#[cfg(test)]
mod tests {
    use super::*;
    use ed25519_dalek::SigningKey;
    use rand::thread_rng;

    #[tokio::test]
    async fn test_treasury_proposal_flow() {
        let mut treasury = TreasuryManager::new();
        treasury.initialize().await.unwrap();

        let keypair = SigningKey::from_bytes(&rand::random());
        let proposer = Address::from_public_key(&keypair.verifying_key());

        let request = FundingProposalRequest {
            title: "Network Optimization Research".to_string(),
            description: "Research into advanced network optimization techniques".to_string(),
            requested_amount: 10_000_000_000, // 100 PAR
            category: FundingCategory::Research,
            milestones: vec![
                Milestone {
                    description: "Literature review".to_string(),
                    deadline: Utc::now() + chrono::Duration::days(30),
                    funding_percentage: 0.3,
                },
                Milestone {
                    description: "Implementation".to_string(),
                    deadline: Utc::now() + chrono::Duration::days(60),
                    funding_percentage: 0.7,
                },
            ],
        };

        let proposal_id = treasury
            .submit_funding_proposal(proposer, request)
            .await
            .unwrap();
        assert!(treasury.active_proposals.contains_key(&proposal_id));

        let stats = treasury.get_treasury_stats();
        assert_eq!(stats.active_proposals, 1);
    }
}
