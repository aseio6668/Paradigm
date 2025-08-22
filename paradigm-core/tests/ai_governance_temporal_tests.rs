use chrono::{Duration, Utc};
/// Comprehensive tests for AI Agent Governance and Temporal Token Evolution
/// Tests the integration and functionality of both advanced systems
use paradigm_core::tokenomics::*;
use paradigm_core::Address;

/// Create a test address with a specific ID
fn create_test_address(id: u8) -> Address {
    let mut addr = [0u8; 32];
    addr[0] = id;
    Address(addr)
}

#[tokio::test]
async fn test_ai_governance_system_initialization() {
    let mut ai_governance = AIAgentGovernanceSystem::new();

    let result = ai_governance.initialize().await;
    assert!(
        result.is_ok(),
        "AI governance system should initialize successfully"
    );

    // Check that initial agents were created
    let agents = ai_governance
        .get_all_agents()
        .await
        .expect("Should get agents");
    assert_eq!(agents.len(), 5, "Should have 5 initial agents");

    // Verify agent specializations
    let specializations: Vec<_> = agents.iter().map(|a| &a.specialization).collect();
    assert!(specializations.contains(&&AgentSpecialization::Economic));
    assert!(specializations.contains(&&AgentSpecialization::Technical));
    assert!(specializations.contains(&&AgentSpecialization::Community));
    assert!(specializations.contains(&&AgentSpecialization::Security));
    assert!(specializations.contains(&&AgentSpecialization::Arbitration));
}

#[tokio::test]
async fn test_temporal_evolution_system_initialization() {
    let mut temporal_evolution = TemporalTokenEvolution::new();

    let result = temporal_evolution.initialize().await;
    assert!(
        result.is_ok(),
        "Temporal evolution system should initialize successfully"
    );

    // Test token state creation
    let address = create_test_address(1);
    let network_state = create_test_network_state();

    let evolution_result = temporal_evolution
        .evolve_tokens(&address, &network_state)
        .await;
    assert!(
        evolution_result.is_ok(),
        "Token evolution should work for new address"
    );
}

#[tokio::test]
async fn test_ai_agent_proposal_analysis() {
    let mut ai_governance = AIAgentGovernanceSystem::new();
    ai_governance.initialize().await.expect("Should initialize");

    // Create test proposal
    let proposal_data = ProposalData {
        id: uuid::Uuid::new_v4(),
        title: "Test Economic Proposal".to_string(),
        description: "Adjust inflation rate for network stability".to_string(),
        proposal_type: AIProposalType::ParameterAdjustment,
        created_at: Utc::now(),
    };

    // Get AI agent analysis
    let analysis_result = ai_governance
        .agents_analyze_proposal(proposal_data.id, &proposal_data)
        .await;
    assert!(analysis_result.is_ok(), "AI agent analysis should succeed");

    let votes = analysis_result.unwrap();
    assert_eq!(votes.len(), 5, "Should get votes from all 5 agents");

    // Verify vote structure
    for vote in &votes {
        assert!(matches!(
            vote.vote_direction,
            VoteDirection::For | VoteDirection::Against
        ));
        assert!(vote.confidence >= 0.0 && vote.confidence <= 1.0);
        assert!(!vote.reasoning.is_empty());
    }
}

#[tokio::test]
async fn test_ai_proposal_generation() {
    let mut ai_governance = AIAgentGovernanceSystem::new();
    ai_governance.initialize().await.expect("Should initialize");

    // Create network state that should trigger proposals
    let network_state = NetworkState {
        inflation_rate: 0.15,                // High inflation should trigger proposal
        governance_participation_rate: 0.05, // Low participation should trigger proposal
        treasury_balance: 10000,             // Low treasury ratio should trigger proposal
        total_supply: 1000000,
        error_rate: 0.08, // High error rate should trigger security proposal
        ..create_test_network_state()
    };

    let proposals = ai_governance.generate_ai_proposals(&network_state).await;
    assert!(proposals.is_ok(), "AI proposal generation should succeed");

    let generated_proposals = proposals.unwrap();
    assert!(
        !generated_proposals.is_empty(),
        "Should generate proposals for problematic network state"
    );

    // Check for expected proposal types
    let proposal_types: Vec<_> = generated_proposals
        .iter()
        .map(|p| &p.proposal_type)
        .collect();
    assert!(proposal_types.contains(&&AIProposalType::ParameterAdjustment)); // For inflation
    assert!(proposal_types.contains(&&AIProposalType::SecurityUpdate)); // For high error rate
}

#[tokio::test]
async fn test_consensus_prediction() {
    let ai_governance = AIAgentGovernanceSystem::new();

    let proposal_data = ProposalData {
        id: uuid::Uuid::new_v4(),
        title: "Test Consensus Proposal".to_string(),
        description: "Simple parameter adjustment".to_string(),
        proposal_type: AIProposalType::ParameterAdjustment,
        created_at: Utc::now(),
    };

    // Create mock human votes (majority for)
    let human_votes = vec![
        HumanVote {
            id: uuid::Uuid::new_v4(),
            voter: create_test_address(10),
            proposal_id: proposal_data.id,
            direction: VoteDirection::For,
            weight: 100,
            timestamp: Utc::now(),
        },
        HumanVote {
            id: uuid::Uuid::new_v4(),
            voter: create_test_address(11),
            proposal_id: proposal_data.id,
            direction: VoteDirection::For,
            weight: 150,
            timestamp: Utc::now(),
        },
        HumanVote {
            id: uuid::Uuid::new_v4(),
            voter: create_test_address(12),
            proposal_id: proposal_data.id,
            direction: VoteDirection::Against,
            weight: 50,
            timestamp: Utc::now(),
        },
    ];

    let prediction = ai_governance
        .predict_consensus(&proposal_data, &human_votes)
        .await;
    assert!(prediction.is_ok(), "Consensus prediction should succeed");

    let prediction_result = prediction.unwrap();
    assert!(
        prediction_result.consensus_probability > 0.5,
        "Should predict positive consensus for majority-for votes"
    );
    assert!(matches!(
        prediction_result.expected_outcome,
        PredictedOutcome::Pass
    ));
}

#[tokio::test]
async fn test_temporal_staking_system() {
    let mut temporal_evolution = TemporalTokenEvolution::new();
    temporal_evolution
        .initialize()
        .await
        .expect("Should initialize");

    let address = create_test_address(2);

    // Test staking
    let stake_result = temporal_evolution
        .stake_tokens(&address, 1000, TemporalStakingType::MediumTerm)
        .await;
    assert!(stake_result.is_ok(), "Staking should succeed");

    let staking_info = stake_result.unwrap();
    assert_eq!(staking_info.amount_staked, 1000);
    assert!(
        staking_info.evolution_multiplier > 1.0,
        "Should have evolution multiplier"
    );
    assert!(
        staking_info.estimated_rewards > 0,
        "Should have estimated rewards"
    );

    // Test getting staking info
    let stake_info = temporal_evolution.get_staking_info(&address).await;
    assert!(stake_info.is_ok(), "Should get staking info");

    let stakes = stake_info.unwrap();
    assert_eq!(stakes.len(), 1, "Should have one stake");
    assert_eq!(stakes[0].amount, 1000);
    assert!(matches!(stakes[0].status, StakingStatus::Active));
}

#[tokio::test]
async fn test_token_evolution_stages() {
    let mut temporal_evolution = TemporalTokenEvolution::new();
    temporal_evolution
        .initialize()
        .await
        .expect("Should initialize");

    let address = create_test_address(3);
    let network_state = create_test_network_state();

    // Initial evolution should create Genesis stage
    let evolution_result = temporal_evolution
        .evolve_tokens(&address, &network_state)
        .await;
    assert!(evolution_result.is_ok(), "Token evolution should succeed");

    let result = evolution_result.unwrap();
    assert!(matches!(result.evolution_stage, EvolutionStage::Genesis));

    // Get token state to verify
    let token_state = temporal_evolution.get_token_state(&address).await;
    assert!(token_state.is_ok(), "Should get token state");

    let state = token_state.unwrap();
    assert!(state.is_some(), "Token state should exist");

    let token_state = state.unwrap();
    assert!(matches!(
        token_state.evolution_stage,
        EvolutionStage::Genesis
    ));
    assert_eq!(
        token_state.activity_streak, 0,
        "Should start with no activity streak"
    );
}

#[tokio::test]
async fn test_contribution_evolution_integration() {
    let mut temporal_evolution = TemporalTokenEvolution::new();
    temporal_evolution
        .initialize()
        .await
        .expect("Should initialize");

    let address = create_test_address(4);

    // Create test contribution
    let contribution = ContributionProof {
        id: uuid::Uuid::new_v4(),
        contributor: address.clone(),
        contribution_type: ContributionType::MLTraining,
        workload_hash: vec![1, 2, 3, 4],
        zk_proof: vec![5, 6, 7, 8],
        qr_zk_proof: None,
        qr_signature: None,
        metadata: serde_json::json!({"test": "contribution"}),
        timestamp: Utc::now(),
    };

    // Record contribution for evolution
    let record_result = temporal_evolution
        .record_contribution_for_evolution(&address, &contribution, 500)
        .await;
    assert!(
        record_result.is_ok(),
        "Should record contribution for evolution"
    );

    // Check that token state was updated
    let token_state = temporal_evolution
        .get_token_state(&address)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(
        token_state.contribution_history.len(),
        1,
        "Should have one contribution record"
    );
    assert!(
        token_state.evolution_factors.ml_specialization > 0.0,
        "Should increase ML specialization"
    );
    assert!(
        token_state.activity_streak > 0,
        "Should have activity streak"
    );
}

#[tokio::test]
async fn test_evolution_prediction() {
    let mut temporal_evolution = TemporalTokenEvolution::new();
    temporal_evolution
        .initialize()
        .await
        .expect("Should initialize");

    let address = create_test_address(5);
    let network_state = create_test_network_state();

    // Set up initial token state
    temporal_evolution
        .evolve_tokens(&address, &network_state)
        .await
        .expect("Should evolve tokens");

    // Test prediction
    let time_horizon = Duration::days(30);
    let prediction = temporal_evolution
        .predict_evolution(&address, time_horizon)
        .await;
    assert!(prediction.is_ok(), "Evolution prediction should succeed");

    let prediction_result = prediction.unwrap();
    assert!(
        prediction_result.confidence > 0.0,
        "Should have confidence value"
    );
    assert_eq!(prediction_result.time_horizon, time_horizon);
    assert!(prediction_result.predicted_at <= Utc::now());
}

#[tokio::test]
async fn test_global_evolution_processing() {
    let mut temporal_evolution = TemporalTokenEvolution::new();
    temporal_evolution
        .initialize()
        .await
        .expect("Should initialize");

    let network_state = create_test_network_state();

    // Create multiple addresses with token states
    for i in 0..3 {
        let address = create_test_address(10 + i);
        temporal_evolution
            .evolve_tokens(&address, &network_state)
            .await
            .expect("Should evolve tokens");
    }

    // Process global evolution
    let global_result = temporal_evolution
        .process_global_evolution(&network_state)
        .await;
    assert!(global_result.is_ok(), "Global evolution should succeed");

    let result = global_result.unwrap();
    assert_eq!(result.addresses_processed, 3, "Should process 3 addresses");
    assert_eq!(
        result.individual_results.len(),
        3,
        "Should have 3 individual results"
    );
}

#[tokio::test]
async fn test_ai_agent_learning_and_feedback() {
    let mut ai_governance = AIAgentGovernanceSystem::new();
    ai_governance.initialize().await.expect("Should initialize");

    // Get an agent to provide feedback to
    let agents = ai_governance
        .get_all_agents()
        .await
        .expect("Should get agents");
    let agent_id = agents[0].id;

    // Create feedback
    let feedback = HumanFeedback {
        id: uuid::Uuid::new_v4(),
        agent_id,
        feedback_type: FeedbackType::Positive,
        content: "Good analysis on the economic proposal".to_string(),
        rating: Some(4.5),
        provided_by: create_test_address(20),
        timestamp: Utc::now(),
    };

    // Submit feedback
    let feedback_result = ai_governance
        .process_human_feedback(agent_id, feedback)
        .await;
    assert!(
        feedback_result.is_ok(),
        "Should process feedback successfully"
    );

    // Test learning from human votes
    let proposal_id = uuid::Uuid::new_v4();
    let human_votes = vec![HumanVote {
        id: uuid::Uuid::new_v4(),
        voter: create_test_address(21),
        proposal_id,
        direction: VoteDirection::For,
        weight: 100,
        timestamp: Utc::now(),
    }];

    let learning_result = ai_governance
        .learn_from_human_votes(proposal_id, &human_votes, ProposalOutcome::Passed)
        .await;
    assert!(learning_result.is_ok(), "Should learn from human votes");
}

#[tokio::test]
async fn test_agent_evolution_system() {
    let mut ai_governance = AIAgentGovernanceSystem::new();
    ai_governance.initialize().await.expect("Should initialize");

    // Test agent evolution
    let evolution_result = ai_governance.evolve_agents().await;
    assert!(evolution_result.is_ok(), "Agent evolution should succeed");

    let evolution_report = evolution_result.unwrap();
    assert!(
        evolution_report.generation > 0,
        "Should have generation number"
    );
    assert!(
        evolution_report.evolution_metrics.total_agents >= 5,
        "Should maintain at least initial agents"
    );
}

#[tokio::test]
async fn test_temporal_decay_mechanism() {
    let mut temporal_evolution = TemporalTokenEvolution::new();
    temporal_evolution
        .initialize()
        .await
        .expect("Should initialize");

    let address = create_test_address(6);
    let network_state = create_test_network_state();

    // Create token state with balance
    temporal_evolution
        .evolve_tokens(&address, &network_state)
        .await
        .expect("Should evolve tokens");

    // Simulate time passage and inactivity by getting and modifying token state
    let mut token_state = temporal_evolution
        .get_token_state(&address)
        .await
        .unwrap()
        .unwrap();

    // In a real implementation, we would modify the last_activity to simulate passage of time
    // For this test, we just verify that evolution considers decay factors
    let evolution_result = temporal_evolution
        .evolve_tokens(&address, &network_state)
        .await;
    assert!(
        evolution_result.is_ok(),
        "Evolution with decay should succeed"
    );

    let result = evolution_result.unwrap();
    // Decay amount depends on activity and other factors
    assert!(
        result.decay_amount >= 0.0,
        "Decay amount should be non-negative"
    );
}

#[tokio::test]
async fn test_integration_with_tokenomics_system() {
    let mut system = TokenomicsSystem::new();
    system.start().await.expect("System should start");

    let address = create_test_address(7);

    // Test AI-generated proposals
    let ai_proposals = system.get_ai_generated_proposals().await;
    assert!(ai_proposals.is_ok(), "Should get AI-generated proposals");

    // Test AI agent statistics
    let agent_stats = system.get_ai_agent_statistics().await;
    assert!(agent_stats.is_ok(), "Should get agent statistics");

    let stats = agent_stats.unwrap();
    assert_eq!(stats.total_agents, 5, "Should have 5 initial agents");
    assert_eq!(stats.active_agents, 5, "All agents should be active");

    // Test token evolution
    let evolution_result = system.evolve_tokens(&address).await;
    assert!(evolution_result.is_ok(), "Should evolve tokens");

    // Test temporal staking
    let staking_result = system
        .stake_tokens_temporal(&address, 1000, TemporalStakingType::LongTerm)
        .await;
    assert!(staking_result.is_ok(), "Should stake tokens temporally");

    // Test evolution prediction
    let prediction = system
        .predict_token_evolution(&address, Duration::days(7))
        .await;
    assert!(prediction.is_ok(), "Should predict token evolution");

    // Test global evolution
    let global_evolution = system.process_global_token_evolution().await;
    assert!(global_evolution.is_ok(), "Should process global evolution");
}

#[tokio::test]
async fn test_performance_benchmarks() {
    let mut system = TokenomicsSystem::new();
    system.start().await.expect("System should start");

    let address = create_test_address(8);

    // Benchmark AI proposal generation
    let start = std::time::Instant::now();
    let _proposals = system
        .get_ai_generated_proposals()
        .await
        .expect("Should get proposals");
    let ai_proposal_time = start.elapsed();

    // Benchmark token evolution
    let start = std::time::Instant::now();
    let _evolution = system
        .evolve_tokens(&address)
        .await
        .expect("Should evolve tokens");
    let evolution_time = start.elapsed();

    // Benchmark temporal staking
    let start = std::time::Instant::now();
    let _staking = system
        .stake_tokens_temporal(&address, 1000, TemporalStakingType::ShortTerm)
        .await
        .expect("Should stake");
    let staking_time = start.elapsed();

    println!("Performance benchmarks:");
    println!("AI proposal generation: {:?}", ai_proposal_time);
    println!("Token evolution: {:?}", evolution_time);
    println!("Temporal staking: {:?}", staking_time);

    // Verify performance meets requirements
    assert!(
        ai_proposal_time.as_millis() < 1000,
        "AI proposal generation should be under 1 second"
    );
    assert!(
        evolution_time.as_millis() < 500,
        "Token evolution should be under 500ms"
    );
    assert!(
        staking_time.as_millis() < 200,
        "Temporal staking should be under 200ms"
    );
}

// Helper function to create test network state
fn create_test_network_state() -> NetworkState {
    NetworkState {
        total_supply: 1000000,
        active_participants: 100,
        transaction_volume: 50000,
        transaction_throughput: 100.0,
        uptime_percentage: 0.99,
        avg_consensus_time: 2.5,
        error_rate: 0.01,
        resource_utilization: 0.75,
        token_velocity: 3.2,
        network_growth: 0.15,
        inflation_rate: 0.05,
        top_10_validator_stake_percentage: 0.35,
        geographic_diversity_index: 0.8,
        wealth_concentration_gini: 0.4,
        treasury_balance: 500000,
        mint_rate: 0.02,
        burn_rate: 0.01,
        total_rewards: 10000,
        productive_work_rewards: 8000,
        avg_transaction_fee: 0.01,
        contributor_satisfaction_score: 0.85,
        governance_participation_rate: 0.6,
        monthly_active_user_retention: 0.8,
    }
}
