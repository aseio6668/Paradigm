use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::{DateTime, Utc, Duration};
use anyhow::{Result, anyhow};

use crate::{Hash, Amount, Address};
use super::{ChainId, CrossChainConfig};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ProposalType {
    ParameterChange {
        parameter: String,
        old_value: String,
        new_value: String,
    },
    ProtocolUpgrade {
        upgrade_version: String,
        upgrade_hash: Hash,
        upgrade_data: Vec<u8>,
    },
    AssetListing {
        asset_symbol: String,
        asset_contract: String,
        chain_id: ChainId,
    },
    BridgeConfiguration {
        chain_id: ChainId,
        bridge_contract: String,
        fee_rate: f64,
    },
    TreasurySpending {
        recipient: Address,
        amount: Amount,
        purpose: String,
    },
    ValidatorUpdate {
        validator_address: Address,
        action: ValidatorAction,
    },
    EmergencyAction {
        action_type: EmergencyActionType,
        target_chain: Option<ChainId>,
        additional_data: Vec<u8>,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ValidatorAction {
    Add,
    Remove,
    UpdateStake(Amount),
    Slash(f64),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EmergencyActionType {
    PauseBridge,
    ResumeBridge,
    FreezeAsset,
    UnfreezeAsset,
    EmergencyWithdraw,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ProposalStatus {
    Draft,
    Active,
    Passed,
    Rejected,
    Executed,
    Cancelled,
    Expired,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum VoteOption {
    Yes,
    No,
    Abstain,
    NoWithVeto,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CrossChainProposal {
    pub proposal_id: Uuid,
    pub title: String,
    pub description: String,
    pub proposer: Address,
    pub proposal_type: ProposalType,
    pub target_chains: Vec<ChainId>,
    pub voting_start_time: DateTime<Utc>,
    pub voting_end_time: DateTime<Utc>,
    pub execution_delay: Duration,
    pub minimum_deposit: Amount,
    pub current_deposit: Amount,
    pub quorum_threshold: f64,
    pub pass_threshold: f64,
    pub veto_threshold: f64,
    pub status: ProposalStatus,
    pub votes: HashMap<ChainId, ChainVotingResults>,
    pub execution_data: Option<Vec<u8>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChainVotingResults {
    pub chain_id: ChainId,
    pub total_voting_power: Amount,
    pub yes_votes: Amount,
    pub no_votes: Amount,
    pub abstain_votes: Amount,
    pub no_with_veto_votes: Amount,
    pub participation_rate: f64,
    pub votes: HashMap<Address, Vote>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Vote {
    pub voter: Address,
    pub option: VoteOption,
    pub voting_power: Amount,
    pub timestamp: DateTime<Utc>,
    pub chain_id: ChainId,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Validator {
    pub address: Address,
    pub chain_id: ChainId,
    pub voting_power: Amount,
    pub commission_rate: f64,
    pub is_active: bool,
    pub delegated_stake: Amount,
    pub self_stake: Amount,
    pub reputation_score: f64,
    pub uptime_percentage: f64,
    pub last_signed_block: u64,
    pub jailed_until: Option<DateTime<Utc>>,
    pub slashing_history: Vec<SlashingEvent>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SlashingEvent {
    pub event_id: Uuid,
    pub validator: Address,
    pub slash_percentage: f64,
    pub reason: String,
    pub block_height: u64,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Delegation {
    pub delegator: Address,
    pub validator: Address,
    pub chain_id: ChainId,
    pub amount: Amount,
    pub rewards_accumulated: Amount,
    pub created_at: DateTime<Utc>,
    pub last_claim: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GovernanceConfig {
    pub min_deposit_amount: Amount,
    pub voting_period_hours: u64,
    pub execution_delay_hours: u64,
    pub quorum_threshold: f64,
    pub pass_threshold: f64,
    pub veto_threshold: f64,
    pub validator_min_stake: Amount,
    pub max_validators_per_chain: usize,
    pub slashing_percentage: f64,
}

impl Default for GovernanceConfig {
    fn default() -> Self {
        Self {
            min_deposit_amount: 1000_00000000, // 1000 PAR
            voting_period_hours: 168, // 7 days
            execution_delay_hours: 48, // 2 days
            quorum_threshold: 0.334, // 33.4%
            pass_threshold: 0.5, // 50%
            veto_threshold: 0.334, // 33.4%
            validator_min_stake: 10000_00000000, // 10,000 PAR
            max_validators_per_chain: 100,
            slashing_percentage: 0.05, // 5%
        }
    }
}

pub struct CrossChainGovernance {
    proposals: Arc<RwLock<HashMap<Uuid, CrossChainProposal>>>,
    validators: Arc<RwLock<HashMap<(Address, ChainId), Validator>>>,
    delegations: Arc<RwLock<HashMap<(Address, Address, ChainId), Delegation>>>,
    voting_powers: Arc<RwLock<HashMap<(Address, ChainId), Amount>>>,
    config: GovernanceConfig,
    cross_chain_config: CrossChainConfig,
}

impl CrossChainGovernance {
    pub async fn new(cross_chain_config: CrossChainConfig) -> Result<Self> {
        Ok(Self {
            proposals: Arc::new(RwLock::new(HashMap::new())),
            validators: Arc::new(RwLock::new(HashMap::new())),
            delegations: Arc::new(RwLock::new(HashMap::new())),
            voting_powers: Arc::new(RwLock::new(HashMap::new())),
            config: GovernanceConfig::default(),
            cross_chain_config,
        })
    }

    pub async fn initialize(&self) -> Result<()> {
        tracing::info!("Initializing Cross-Chain Governance...");
        
        // Start monitoring tasks
        self.start_monitoring_tasks().await?;
        
        tracing::info!("Cross-Chain Governance initialized successfully");
        Ok(())
    }

    pub async fn create_proposal(
        &self,
        title: String,
        description: String,
        proposer: Address,
        proposal_type: ProposalType,
        target_chains: Vec<ChainId>,
        initial_deposit: Amount,
    ) -> Result<Uuid> {
        if initial_deposit < self.config.min_deposit_amount {
            return Err(anyhow!("Insufficient deposit amount"));
        }
        
        let proposal_id = Uuid::new_v4();
        let now = Utc::now();
        let voting_start_time = now + Duration::hours(1); // 1 hour delay
        let voting_end_time = voting_start_time + Duration::hours(self.config.voting_period_hours as i64);
        
        let mut votes = HashMap::new();
        for &chain_id in &target_chains {
            votes.insert(chain_id, ChainVotingResults {
                chain_id,
                total_voting_power: 0,
                yes_votes: 0,
                no_votes: 0,
                abstain_votes: 0,
                no_with_veto_votes: 0,
                participation_rate: 0.0,
                votes: HashMap::new(),
            });
        }
        
        let proposal = CrossChainProposal {
            proposal_id,
            title,
            description,
            proposer,
            proposal_type,
            target_chains,
            voting_start_time,
            voting_end_time,
            execution_delay: Duration::hours(self.config.execution_delay_hours as i64),
            minimum_deposit: self.config.min_deposit_amount,
            current_deposit: initial_deposit,
            quorum_threshold: self.config.quorum_threshold,
            pass_threshold: self.config.pass_threshold,
            veto_threshold: self.config.veto_threshold,
            status: ProposalStatus::Active,
            votes,
            execution_data: None,
            created_at: now,
            updated_at: now,
        };
        
        let mut proposals = self.proposals.write().await;
        proposals.insert(proposal_id, proposal);
        
        tracing::info!("Created cross-chain proposal: {}", proposal_id);
        Ok(proposal_id)
    }

    pub async fn cast_vote(
        &self,
        proposal_id: Uuid,
        voter: Address,
        option: VoteOption,
        chain_id: ChainId,
    ) -> Result<()> {
        let voting_power = self.get_voting_power(&voter, chain_id).await?;
        if voting_power == 0 {
            return Err(anyhow!("No voting power"));
        }
        
        let mut proposals = self.proposals.write().await;
        let proposal = proposals.get_mut(&proposal_id)
            .ok_or_else(|| anyhow!("Proposal not found"))?;
        
        // Check if voting is active
        let now = Utc::now();
        if now < proposal.voting_start_time || now > proposal.voting_end_time {
            return Err(anyhow!("Voting period is not active"));
        }
        
        // Check if chain is included in proposal
        if !proposal.target_chains.contains(&chain_id) {
            return Err(anyhow!("Chain not included in this proposal"));
        }
        
        let vote = Vote {
            voter,
            option: option.clone(),
            voting_power,
            timestamp: now,
            chain_id,
        };
        
        // Update chain voting results
        if let Some(chain_results) = proposal.votes.get_mut(&chain_id) {
            // Remove previous vote if exists
            if let Some(prev_vote) = chain_results.votes.remove(&voter) {
                match prev_vote.option {
                    VoteOption::Yes => chain_results.yes_votes -= prev_vote.voting_power,
                    VoteOption::No => chain_results.no_votes -= prev_vote.voting_power,
                    VoteOption::Abstain => chain_results.abstain_votes -= prev_vote.voting_power,
                    VoteOption::NoWithVeto => chain_results.no_with_veto_votes -= prev_vote.voting_power,
                }
            }
            
            // Add new vote
            match option {
                VoteOption::Yes => chain_results.yes_votes += voting_power,
                VoteOption::No => chain_results.no_votes += voting_power,
                VoteOption::Abstain => chain_results.abstain_votes += voting_power,
                VoteOption::NoWithVeto => chain_results.no_with_veto_votes += voting_power,
            }
            
            chain_results.votes.insert(voter, vote);
            
            // Recalculate participation rate
            let total_votes = chain_results.yes_votes + chain_results.no_votes + 
                            chain_results.abstain_votes + chain_results.no_with_veto_votes;
            chain_results.participation_rate = if chain_results.total_voting_power > 0 {
                total_votes as f64 / chain_results.total_voting_power as f64
            } else {
                0.0
            };
        }
        
        proposal.updated_at = now;
        
        tracing::info!("Vote cast for proposal {} by {} on {:?}", proposal_id, voter, chain_id);
        Ok(())
    }

    pub async fn execute_proposal(&self, proposal_id: Uuid) -> Result<()> {
        let mut proposals = self.proposals.write().await;
        let proposal = proposals.get_mut(&proposal_id)
            .ok_or_else(|| anyhow!("Proposal not found"))?;
        
        if proposal.status != ProposalStatus::Passed {
            return Err(anyhow!("Proposal has not passed"));
        }
        
        let now = Utc::now();
        let execution_time = proposal.voting_end_time + proposal.execution_delay;
        if now < execution_time {
            return Err(anyhow!("Execution delay has not passed"));
        }
        
        // Execute the proposal based on its type
        match &proposal.proposal_type {
            ProposalType::ParameterChange { parameter, new_value, .. } => {
                self.execute_parameter_change(parameter, new_value).await?;
            },
            ProposalType::ProtocolUpgrade { upgrade_version, upgrade_data, .. } => {
                self.execute_protocol_upgrade(upgrade_version, upgrade_data).await?;
            },
            ProposalType::AssetListing { asset_symbol, asset_contract, chain_id } => {
                self.execute_asset_listing(asset_symbol, asset_contract, *chain_id).await?;
            },
            ProposalType::BridgeConfiguration { chain_id, bridge_contract, fee_rate } => {
                self.execute_bridge_configuration(*chain_id, bridge_contract, *fee_rate).await?;
            },
            ProposalType::TreasurySpending { recipient, amount, purpose } => {
                self.execute_treasury_spending(*recipient, *amount, purpose).await?;
            },
            ProposalType::ValidatorUpdate { validator_address, action } => {
                self.execute_validator_update(*validator_address, action).await?;
            },
            ProposalType::EmergencyAction { action_type, target_chain, additional_data } => {
                self.execute_emergency_action(action_type, *target_chain, additional_data).await?;
            },
        }
        
        proposal.status = ProposalStatus::Executed;
        proposal.updated_at = now;
        
        tracing::info!("Executed proposal: {}", proposal_id);
        Ok(())
    }

    pub async fn register_validator(
        &self,
        address: Address,
        chain_id: ChainId,
        self_stake: Amount,
        commission_rate: f64,
    ) -> Result<()> {
        if self_stake < self.config.validator_min_stake {
            return Err(anyhow!("Insufficient stake amount"));
        }
        
        let validator = Validator {
            address,
            chain_id,
            voting_power: self_stake,
            commission_rate,
            is_active: true,
            delegated_stake: 0,
            self_stake,
            reputation_score: 1.0,
            uptime_percentage: 100.0,
            last_signed_block: 0,
            jailed_until: None,
            slashing_history: Vec::new(),
        };
        
        let mut validators = self.validators.write().await;
        validators.insert((address, chain_id), validator);
        
        // Update voting power
        self.update_voting_power(address, chain_id, self_stake).await?;
        
        tracing::info!("Registered validator {} on {:?}", address, chain_id);
        Ok(())
    }

    pub async fn delegate_stake(
        &self,
        delegator: Address,
        validator: Address,
        chain_id: ChainId,
        amount: Amount,
    ) -> Result<()> {
        // Check if validator exists
        {
            let validators = self.validators.read().await;
            if !validators.contains_key(&(validator, chain_id)) {
                return Err(anyhow!("Validator not found"));
            }
        }
        
        let delegation = Delegation {
            delegator,
            validator,
            chain_id,
            amount,
            rewards_accumulated: 0,
            created_at: Utc::now(),
            last_claim: Utc::now(),
        };
        
        let mut delegations = self.delegations.write().await;
        delegations.insert((delegator, validator, chain_id), delegation);
        
        // Update validator's delegated stake
        {
            let mut validators = self.validators.write().await;
            if let Some(val) = validators.get_mut(&(validator, chain_id)) {
                val.delegated_stake += amount;
                val.voting_power = val.self_stake + val.delegated_stake;
            }
        }
        
        // Update delegator's voting power
        let current_power = self.get_voting_power(&delegator, chain_id).await.unwrap_or(0);
        self.update_voting_power(delegator, chain_id, current_power + amount).await?;
        
        tracing::info!("Delegated {} from {} to {} on {:?}", amount, delegator, validator, chain_id);
        Ok(())
    }

    pub async fn get_proposal(&self, proposal_id: &Uuid) -> Option<CrossChainProposal> {
        let proposals = self.proposals.read().await;
        proposals.get(proposal_id).cloned()
    }

    pub async fn list_active_proposals(&self) -> Vec<CrossChainProposal> {
        let proposals = self.proposals.read().await;
        proposals.values()
            .filter(|p| matches!(p.status, ProposalStatus::Active))
            .cloned()
            .collect()
    }

    pub async fn get_voting_power(&self, address: &Address, chain_id: ChainId) -> Result<Amount> {
        let voting_powers = self.voting_powers.read().await;
        Ok(voting_powers.get(&(*address, chain_id)).cloned().unwrap_or(0))
    }

    // Private methods

    async fn start_monitoring_tasks(&self) -> Result<()> {
        // Monitor proposal status
        let proposals = self.proposals.clone();
        let config = self.config.clone();
        
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(std::time::Duration::from_secs(300)); // 5 minutes
            loop {
                interval.tick().await;
                
                let now = Utc::now();
                let mut proposals_to_update = Vec::new();
                
                {
                    let proposals_read = proposals.read().await;
                    for (proposal_id, proposal) in proposals_read.iter() {
                        if proposal.status == ProposalStatus::Active && now > proposal.voting_end_time {
                            proposals_to_update.push((*proposal_id, Self::calculate_proposal_result(proposal, &config)));
                        }
                    }
                }
                
                {
                    let mut proposals_write = proposals.write().await;
                    for (proposal_id, new_status) in proposals_to_update {
                        if let Some(proposal) = proposals_write.get_mut(&proposal_id) {
                            proposal.status = new_status;
                            proposal.updated_at = now;
                            tracing::info!("Updated proposal {} status to {:?}", proposal_id, proposal.status);
                        }
                    }
                }
            }
        });
        
        Ok(())
    }

    fn calculate_proposal_result(proposal: &CrossChainProposal, config: &GovernanceConfig) -> ProposalStatus {
        let mut total_yes = 0u64;
        let mut total_no = 0u64;
        let mut total_abstain = 0u64;
        let mut total_veto = 0u64;
        let mut total_voting_power = 0u64;
        
        for chain_results in proposal.votes.values() {
            total_yes += chain_results.yes_votes;
            total_no += chain_results.no_votes;
            total_abstain += chain_results.abstain_votes;
            total_veto += chain_results.no_with_veto_votes;
            total_voting_power += chain_results.total_voting_power;
        }
        
        let total_votes = total_yes + total_no + total_abstain + total_veto;
        let participation_rate = if total_voting_power > 0 {
            total_votes as f64 / total_voting_power as f64
        } else {
            0.0
        };
        
        // Check quorum
        if participation_rate < config.quorum_threshold {
            return ProposalStatus::Rejected;
        }
        
        // Check veto
        let veto_rate = total_veto as f64 / total_votes as f64;
        if veto_rate >= config.veto_threshold {
            return ProposalStatus::Rejected;
        }
        
        // Check pass threshold
        let yes_rate = total_yes as f64 / (total_yes + total_no) as f64;
        if yes_rate >= config.pass_threshold {
            ProposalStatus::Passed
        } else {
            ProposalStatus::Rejected
        }
    }

    async fn update_voting_power(&self, address: Address, chain_id: ChainId, power: Amount) -> Result<()> {
        let mut voting_powers = self.voting_powers.write().await;
        voting_powers.insert((address, chain_id), power);
        Ok(())
    }

    // Execution methods
    
    async fn execute_parameter_change(&self, parameter: &str, new_value: &str) -> Result<()> {
        tracing::info!("Executing parameter change: {} = {}", parameter, new_value);
        Ok(())
    }
    
    async fn execute_protocol_upgrade(&self, version: &str, upgrade_data: &[u8]) -> Result<()> {
        tracing::info!("Executing protocol upgrade to version: {}", version);
        Ok(())
    }
    
    async fn execute_asset_listing(&self, symbol: &str, contract: &str, chain_id: ChainId) -> Result<()> {
        tracing::info!("Executing asset listing: {} on {:?}", symbol, chain_id);
        Ok(())
    }
    
    async fn execute_bridge_configuration(&self, chain_id: ChainId, contract: &str, fee_rate: f64) -> Result<()> {
        tracing::info!("Executing bridge configuration for {:?}: {} ({}%)", chain_id, contract, fee_rate * 100.0);
        Ok(())
    }
    
    async fn execute_treasury_spending(&self, recipient: Address, amount: Amount, purpose: &str) -> Result<()> {
        tracing::info!("Executing treasury spending: {} PAR to {} for {}", amount, recipient, purpose);
        Ok(())
    }
    
    async fn execute_validator_update(&self, validator: Address, action: &ValidatorAction) -> Result<()> {
        tracing::info!("Executing validator update for {}: {:?}", validator, action);
        Ok(())
    }
    
    async fn execute_emergency_action(&self, action_type: &EmergencyActionType, target_chain: Option<ChainId>, _data: &[u8]) -> Result<()> {
        tracing::warn!("Executing emergency action: {:?} on {:?}", action_type, target_chain);
        Ok(())
    }
}

pub async fn handle_cross_chain_proposal(transaction_id: Uuid) -> Result<()> {
    tracing::info!("Handling cross-chain governance proposal: {}", transaction_id);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_governance_creation() {
        let config = CrossChainConfig::default();
        let governance = CrossChainGovernance::new(config).await;
        assert!(governance.is_ok());
    }
    
    #[tokio::test]
    async fn test_proposal_creation() {
        let config = CrossChainConfig::default();
        let governance = CrossChainGovernance::new(config).await.unwrap();
        
        let proposer = Address([1u8; 32]);
        let proposal_type = ProposalType::ParameterChange {
            parameter: "bridge_fee".to_string(),
            old_value: "0.001".to_string(),
            new_value: "0.002".to_string(),
        };
        
        let proposal_id = governance.create_proposal(
            "Increase Bridge Fee".to_string(),
            "Proposal to increase bridge fee from 0.1% to 0.2%".to_string(),
            proposer,
            proposal_type,
            vec![ChainId::Paradigm, ChainId::Ethereum],
            2000_00000000, // 2000 PAR
        ).await.unwrap();
        
        let proposal = governance.get_proposal(&proposal_id).await;
        assert!(proposal.is_some());
        assert_eq!(proposal.unwrap().current_deposit, 2000_00000000);
    }
    
    #[tokio::test]
    async fn test_validator_registration() {
        let config = CrossChainConfig::default();
        let governance = CrossChainGovernance::new(config).await.unwrap();
        
        let validator_address = Address([1u8; 32]);
        let result = governance.register_validator(
            validator_address,
            ChainId::Paradigm,
            15000_00000000, // 15,000 PAR
            0.05, // 5% commission
        ).await;
        
        assert!(result.is_ok());
        
        let voting_power = governance.get_voting_power(&validator_address, ChainId::Paradigm).await.unwrap();
        assert_eq!(voting_power, 15000_00000000);
    }
}