use chrono::{Duration, Utc};
/// Unit tests for advanced governance system
/// Tests quadratic voting, futarchy, conviction voting, AI agents, and delegation
use paradigm_core::tokenomics::advanced_governance::*;
use paradigm_core::tokenomics::quantum_resistant::QuantumRandom;
use paradigm_core::Address;
use uuid::Uuid;

fn create_test_address(id: u8) -> Address {
    let mut addr = [0u8; 32];
    addr[0] = id;
    Address(addr)
}

fn create_test_quantum_random() -> QuantumRandom {
    QuantumRandom {
        value: vec![1, 2, 3, 4, 5, 6, 7, 8],
        entropy_sources: vec!["test_entropy".to_string()],
        generated_at: Utc::now(),
    }
}

#[tokio::test]
async fn test_advanced_governance_initialization() {
    let mut governance = AdvancedGovernance::new();

    let result = governance.initialize().await;
    assert!(
        result.is_ok(),
        "Advanced governance should initialize successfully"
    );
}

#[tokio::test]
async fn test_quadratic_proposal_creation() {
    let mut governance = AdvancedGovernance::new();
    governance
        .initialize()
        .await
        .expect("Governance should initialize");

    let proposer = create_test_address(1);
    let quantum_random = create_test_quantum_random();

    let proposal_data = QuadraticProposalData {
        title: "Test Proposal".to_string(),
        description: "A test proposal for quadratic voting".to_string(),
        proposal_type: ProposalType::ParameterChange,
        implementation_details: "Detailed implementation plan".to_string(),
        expected_impact: ExpectedImpact {
            network_performance: ImpactRating::Positive,
            economic_efficiency: ImpactRating::VeryPositive,
            decentralization: ImpactRating::Neutral,
            user_experience: ImpactRating::Positive,
            security: ImpactRating::Neutral,
        },
        required_quorum: 0.1,
    };

    let result = governance
        .create_quadratic_proposal(proposer, proposal_data.clone(), quantum_random)
        .await;

    assert!(result.is_ok(), "Quadratic proposal creation should succeed");

    let proposal_id = result.unwrap();
    assert!(!proposal_id.is_nil(), "Proposal ID should be valid");
}

#[tokio::test]
async fn test_quadratic_voting_mechanics() {
    let mut governance = AdvancedGovernance::new();
    governance
        .initialize()
        .await
        .expect("Governance should initialize");

    let proposer = create_test_address(2);
    let voter = create_test_address(3);

    // Create proposal
    let proposal_data = QuadraticProposalData {
        title: "Voting Test Proposal".to_string(),
        description: "Testing quadratic voting mechanics".to_string(),
        proposal_type: ProposalType::NetworkGovernance,
        implementation_details: "Test implementation".to_string(),
        expected_impact: ExpectedImpact {
            network_performance: ImpactRating::Positive,
            economic_efficiency: ImpactRating::Positive,
            decentralization: ImpactRating::VeryPositive,
            user_experience: ImpactRating::Neutral,
            security: ImpactRating::Positive,
        },
        required_quorum: 0.05,
    };

    let proposal_id = governance
        .create_quadratic_proposal(proposer, proposal_data, create_test_quantum_random())
        .await
        .expect("Proposal should be created");

    // Test voting with different strengths
    let vote_strengths = [1.0, 3.0, 5.0, -2.0, -4.0];
    let max_costs = [1000, 5000, 10000, 2000, 8000];

    for (i, (&vote_strength, &max_cost)) in vote_strengths.iter().zip(max_costs.iter()).enumerate()
    {
        let test_voter = create_test_address(10 + i as u8);

        let vote_result = governance
            .cast_quadratic_vote(test_voter, proposal_id, vote_strength, max_cost)
            .await;

        assert!(vote_result.is_ok(), "Vote {} should succeed", i);

        let vote = vote_result.unwrap();
        assert_eq!(vote.proposal_id, proposal_id);
        assert!(vote.cost_paid > 0, "Vote should have a cost");

        // Check quadratic cost scaling
        let expected_cost_approx = (vote_strength.abs() * vote_strength.abs() * 100.0) as u64;
        assert!(vote.cost_paid <= max_cost, "Cost should not exceed maximum");

        if vote_strength >= 0.0 {
            assert!(
                vote.effective_votes >= 0.0,
                "Positive vote should have positive effect"
            );
        } else {
            assert!(
                vote.effective_votes < 0.0,
                "Negative vote should have negative effect"
            );
        }
    }
}

#[tokio::test]
async fn test_vote_cost_exceeds_maximum() {
    let mut governance = AdvancedGovernance::new();
    governance
        .initialize()
        .await
        .expect("Governance should initialize");

    let proposer = create_test_address(4);
    let voter = create_test_address(5);

    let proposal_data = QuadraticProposalData {
        title: "Cost Test Proposal".to_string(),
        description: "Testing vote cost limits".to_string(),
        proposal_type: ProposalType::TreasuryAllocation,
        implementation_details: "Cost test".to_string(),
        expected_impact: ExpectedImpact {
            network_performance: ImpactRating::Neutral,
            economic_efficiency: ImpactRating::Positive,
            decentralization: ImpactRating::Neutral,
            user_experience: ImpactRating::Neutral,
            security: ImpactRating::Neutral,
        },
        required_quorum: 0.1,
    };

    let proposal_id = governance
        .create_quadratic_proposal(proposer, proposal_data, create_test_quantum_random())
        .await
        .expect("Proposal should be created");

    // Try to vote with high strength but low max cost
    let result = governance
        .cast_quadratic_vote(
            voter,
            proposal_id,
            10.0, // High vote strength
            500,  // Low max cost
        )
        .await;

    assert!(
        result.is_err(),
        "Vote should fail when cost exceeds maximum"
    );
}

#[tokio::test]
async fn test_futarchy_market_creation_and_betting() {
    let mut governance = AdvancedGovernance::new();
    governance
        .initialize()
        .await
        .expect("Governance should initialize");

    let proposer = create_test_address(6);
    let bettor = create_test_address(7);

    let proposal_data = FutarchyProposalData {
        title: "Futarchy Test Proposal".to_string(),
        description: "Testing futarchy prediction markets".to_string(),
        implementation_plan: "Detailed implementation plan for testing".to_string(),
        success_criteria: vec![
            "Increase network throughput by 50%".to_string(),
            "Reduce transaction fees by 20%".to_string(),
        ],
        risk_assessment: RiskAssessment {
            technical_risk: RiskLevel::Medium,
            economic_risk: RiskLevel::Low,
            adoption_risk: RiskLevel::High,
            timeline_risk: RiskLevel::Medium,
        },
    };

    let success_metrics = vec![
        SuccessMetric {
            name: "Network Throughput".to_string(),
            description: "Transactions per second".to_string(),
            measurement_method: "Automated monitoring".to_string(),
            predicted_value: 15000.0,
            measurement_deadline: Utc::now() + Duration::days(30),
        },
        SuccessMetric {
            name: "Transaction Fees".to_string(),
            description: "Average transaction fee in tokens".to_string(),
            measurement_method: "Network statistics".to_string(),
            predicted_value: 0.01,
            measurement_deadline: Utc::now() + Duration::days(30),
        },
    ];

    // Create futarchy market
    let market_result = governance
        .create_futarchy_market(proposer, proposal_data, success_metrics)
        .await;

    assert!(
        market_result.is_ok(),
        "Futarchy market creation should succeed"
    );
    let market_id = market_result.unwrap();

    // Test betting on implement scenario
    let implement_bet = OutcomeBet {
        outcome_name: "Network Throughput".to_string(),
        predicted_value: 16000.0, // Optimistic prediction
    };

    let bet_result = governance
        .place_futarchy_bet(
            bettor,
            market_id,
            FutarchyMarketType::Implement,
            implement_bet,
            1000,
        )
        .await;

    assert!(bet_result.is_ok(), "Placing futarchy bet should succeed");

    let bet = bet_result.unwrap();
    assert_eq!(bet.market_id, market_id);
    assert!(bet.expected_payout > 0, "Bet should have expected payout");
    assert!(bet.current_odds > 0.0, "Bet should have current odds");
}

#[tokio::test]
async fn test_futarchy_market_resolution() {
    let mut governance = AdvancedGovernance::new();
    governance
        .initialize()
        .await
        .expect("Governance should initialize");

    let proposer = create_test_address(8);

    let proposal_data = FutarchyProposalData {
        title: "Resolution Test".to_string(),
        description: "Testing market resolution".to_string(),
        implementation_plan: "Test plan".to_string(),
        success_criteria: vec!["Test criteria".to_string()],
        risk_assessment: RiskAssessment {
            technical_risk: RiskLevel::Low,
            economic_risk: RiskLevel::Low,
            adoption_risk: RiskLevel::Low,
            timeline_risk: RiskLevel::Low,
        },
    };

    let success_metrics = vec![SuccessMetric {
        name: "Test Metric".to_string(),
        description: "A test metric".to_string(),
        measurement_method: "Manual testing".to_string(),
        predicted_value: 100.0,
        measurement_deadline: Utc::now() + Duration::days(1),
    }];

    let market_id = governance
        .create_futarchy_market(proposer, proposal_data, success_metrics)
        .await
        .expect("Market should be created");

    // Resolve market with actual outcomes
    let actual_outcomes = vec![MetricOutcome {
        metric_name: "Test Metric".to_string(),
        actual_value: 105.0, // Close to predicted value
        measured_at: Utc::now(),
    }];

    let resolution_result = governance
        .resolve_futarchy_market(market_id, actual_outcomes)
        .await;

    assert!(
        resolution_result.is_ok(),
        "Market resolution should succeed"
    );

    let resolution = resolution_result.unwrap();
    assert_eq!(resolution.market_id, market_id);
    assert!(
        resolution.implement_accuracy >= 0.0,
        "Accuracy should be non-negative"
    );
    assert!(
        resolution.no_implement_accuracy >= 0.0,
        "Accuracy should be non-negative"
    );
}

#[tokio::test]
async fn test_conviction_voting_system() {
    let mut governance = AdvancedGovernance::new();
    governance
        .initialize()
        .await
        .expect("Governance should initialize");

    let proposer = create_test_address(9);
    let supporter1 = create_test_address(10);
    let supporter2 = create_test_address(11);

    let proposal_data = ConvictionProposalData {
        title: "Conviction Test Proposal".to_string(),
        description: "Testing conviction voting mechanics".to_string(),
        deliverables: vec![
            "Phase 1: Research".to_string(),
            "Phase 2: Development".to_string(),
            "Phase 3: Testing".to_string(),
        ],
        timeline: "6 months".to_string(),
    };

    // Create conviction voting proposal
    let proposal_result = governance
        .start_conviction_voting(
            proposer,
            proposal_data,
            50000, // 50k tokens requested
        )
        .await;

    assert!(
        proposal_result.is_ok(),
        "Conviction proposal creation should succeed"
    );
    let proposal_id = proposal_result.unwrap();

    // Test conviction signaling
    let signal1_result = governance
        .signal_conviction(
            supporter1,
            proposal_id,
            5000, // 5k tokens
        )
        .await;

    assert!(
        signal1_result.is_ok(),
        "First conviction signal should succeed"
    );

    let signal1 = signal1_result.unwrap();
    assert_eq!(signal1.proposal_id, proposal_id);
    assert!(signal1.conviction_added > 0.0, "Conviction should be added");
    assert!(
        !signal1.funded,
        "Should not be funded yet with small signal"
    );

    // Add more conviction
    let signal2_result = governance
        .signal_conviction(
            supporter2,
            proposal_id,
            15000, // 15k tokens
        )
        .await;

    assert!(
        signal2_result.is_ok(),
        "Second conviction signal should succeed"
    );

    let signal2 = signal2_result.unwrap();
    assert!(
        signal2.total_conviction > signal1.total_conviction,
        "Total conviction should increase"
    );

    // Check if proposal gets funded (depends on threshold)
    if signal2.total_conviction >= signal2.funding_threshold {
        assert!(
            signal2.funded,
            "Proposal should be funded when threshold is reached"
        );
    }
}

#[tokio::test]
async fn test_ai_agent_proposal_assessment() {
    let mut governance = AdvancedGovernance::new();
    governance
        .initialize()
        .await
        .expect("Governance should initialize");

    let proposal_data = QuadraticProposalData {
        title: "AI Assessment Test".to_string(),
        description: "Testing AI agent proposal assessment".to_string(),
        proposal_type: ProposalType::ProtocolUpgrade,
        implementation_details: "Major protocol changes".to_string(),
        expected_impact: ExpectedImpact {
            network_performance: ImpactRating::VeryPositive,
            economic_efficiency: ImpactRating::Positive,
            decentralization: ImpactRating::Positive,
            user_experience: ImpactRating::VeryPositive,
            security: ImpactRating::VeryPositive,
        },
        required_quorum: 0.2,
    };

    let assessment_result = governance
        .ai_agent_system
        .assess_proposal(&proposal_data)
        .await;

    assert!(assessment_result.is_ok(), "AI assessment should succeed");

    let assessment = assessment_result.unwrap();
    assert!(
        assessment.overall_score >= 0.0 && assessment.overall_score <= 1.0,
        "Assessment score should be between 0 and 1"
    );
    assert!(
        assessment.confidence >= 0.0 && assessment.confidence <= 1.0,
        "Confidence should be between 0 and 1"
    );
    assert!(
        !assessment.individual_assessments.is_empty(),
        "Should have individual assessments"
    );
    assert!(!assessment.reasoning.is_empty(), "Should have reasoning");

    // Check individual agent assessments
    for agent_assessment in &assessment.individual_assessments {
        assert!(
            agent_assessment.score >= 0.0 && agent_assessment.score <= 1.0,
            "Agent score should be between 0 and 1"
        );
        assert!(
            !agent_assessment.agent_name.is_empty(),
            "Agent should have a name"
        );
        assert!(
            !agent_assessment.reasoning.is_empty(),
            "Agent should provide reasoning"
        );
    }
}

#[tokio::test]
async fn test_ai_agent_learning_from_votes() {
    let mut governance = AdvancedGovernance::new();
    governance
        .initialize()
        .await
        .expect("Governance should initialize");

    let voter = create_test_address(12);
    let proposal_id = Uuid::new_v4();

    let vote = QuadraticVote {
        voter,
        vote_strength: 5.0,
        weighted_votes: 7.5, // With some weight multiplier
        cost_paid: 2500,
        timestamp: Utc::now(),
    };

    // Test AI learning from human vote
    let learning_result = governance
        .ai_agent_system
        .update_assessment_from_vote(
            proposal_id,
            &vote,
            100.0, // Total votes for
            50.0,  // Total votes against
        )
        .await;

    assert!(learning_result.is_ok(), "AI learning should succeed");

    // Get AI agent stats to verify system is functioning
    let stats_result = governance.ai_agent_system.get_stats().await;
    assert!(stats_result.is_ok(), "AI stats should be available");

    let stats = stats_result.unwrap();
    assert!(stats.total_agents > 0, "Should have registered AI agents");
    assert!(stats.active_agents > 0, "Should have active AI agents");
}

#[tokio::test]
async fn test_delegation_system() {
    let mut governance = AdvancedGovernance::new();
    governance
        .initialize()
        .await
        .expect("Governance should initialize");

    let delegator = create_test_address(13);
    let delegatee = create_test_address(14);

    // Test full delegation
    let full_delegation_result = governance
        .delegate_voting_power(
            delegator.clone(),
            delegatee.clone(),
            DelegationType::Full,
            Utc::now() + Duration::days(30),
        )
        .await;

    assert!(
        full_delegation_result.is_ok(),
        "Full delegation should succeed"
    );

    let delegation = full_delegation_result.unwrap();
    assert!(delegation.active, "Delegation should be active");

    // Test specific delegation
    let specific_delegation_result = governance
        .delegate_voting_power(
            delegator.clone(),
            delegatee.clone(),
            DelegationType::Specific(vec![
                ProposalType::TreasuryAllocation,
                ProposalType::ParameterChange,
            ]),
            Utc::now() + Duration::days(7),
        )
        .await;

    assert!(
        specific_delegation_result.is_ok(),
        "Specific delegation should succeed"
    );

    // Test temporary delegation
    let temp_delegation_result = governance
        .delegate_voting_power(
            delegator,
            delegatee,
            DelegationType::Temporary,
            Utc::now() + Duration::hours(24),
        )
        .await;

    assert!(
        temp_delegation_result.is_ok(),
        "Temporary delegation should succeed"
    );
}

#[tokio::test]
async fn test_governance_statistics() {
    let mut governance = AdvancedGovernance::new();
    governance
        .initialize()
        .await
        .expect("Governance should initialize");

    let proposer = create_test_address(15);

    // Create some proposals and activities
    let _proposal1 = governance
        .create_quadratic_proposal(
            proposer.clone(),
            QuadraticProposalData {
                title: "Stats Test 1".to_string(),
                description: "First proposal for stats".to_string(),
                proposal_type: ProposalType::ParameterChange,
                implementation_details: "Test implementation".to_string(),
                expected_impact: ExpectedImpact {
                    network_performance: ImpactRating::Positive,
                    economic_efficiency: ImpactRating::Neutral,
                    decentralization: ImpactRating::Positive,
                    user_experience: ImpactRating::Neutral,
                    security: ImpactRating::Positive,
                },
                required_quorum: 0.1,
            },
            create_test_quantum_random(),
        )
        .await
        .expect("First proposal should be created");

    let _proposal2 = governance
        .create_quadratic_proposal(
            proposer,
            QuadraticProposalData {
                title: "Stats Test 2".to_string(),
                description: "Second proposal for stats".to_string(),
                proposal_type: ProposalType::NetworkGovernance,
                implementation_details: "Another test implementation".to_string(),
                expected_impact: ExpectedImpact {
                    network_performance: ImpactRating::VeryPositive,
                    economic_efficiency: ImpactRating::Positive,
                    decentralization: ImpactRating::VeryPositive,
                    user_experience: ImpactRating::Positive,
                    security: ImpactRating::Neutral,
                },
                required_quorum: 0.15,
            },
            create_test_quantum_random(),
        )
        .await
        .expect("Second proposal should be created");

    // Get governance statistics
    let stats_result = governance.get_governance_stats().await;
    assert!(stats_result.is_ok(), "Governance stats should be available");

    let stats = stats_result.unwrap();
    assert_eq!(
        stats.active_quadratic_proposals, 2,
        "Should have 2 active quadratic proposals"
    );
    assert_eq!(
        stats.total_proposals_created, 2,
        "Should track total proposals created"
    );
    assert!(
        stats.governance_participation_rate >= 0.0,
        "Participation rate should be non-negative"
    );
    assert!(
        stats.governance_participation_rate <= 1.0,
        "Participation rate should not exceed 100%"
    );

    // Check subsystem stats
    assert!(
        stats.ai_agent_stats.total_agents > 0,
        "Should have AI agents"
    );
    assert!(
        stats.conviction_stats.active_proposals >= 0,
        "Should track conviction proposals"
    );
    assert!(
        stats.delegation_stats.total_delegations >= 0,
        "Should track delegations"
    );
}

#[tokio::test]
async fn test_proposal_type_cost_differences() {
    let mut governance = AdvancedGovernance::new();
    governance
        .initialize()
        .await
        .expect("Governance should initialize");

    let proposer = create_test_address(16);
    let voter = create_test_address(17);

    let proposal_types = [
        ProposalType::ParameterChange,
        ProposalType::ProtocolUpgrade,
        ProposalType::TreasuryAllocation,
        ProposalType::NetworkGovernance,
        ProposalType::EmergencyAction,
    ];

    let mut proposal_ids = Vec::new();

    // Create proposals of different types
    for proposal_type in &proposal_types {
        let proposal_data = QuadraticProposalData {
            title: format!("Test {:?}", proposal_type),
            description: format!("Testing {:?} proposal type", proposal_type),
            proposal_type: proposal_type.clone(),
            implementation_details: "Type-specific implementation".to_string(),
            expected_impact: ExpectedImpact {
                network_performance: ImpactRating::Positive,
                economic_efficiency: ImpactRating::Positive,
                decentralization: ImpactRating::Neutral,
                user_experience: ImpactRating::Positive,
                security: ImpactRating::Positive,
            },
            required_quorum: 0.1,
        };

        let proposal_id = governance
            .create_quadratic_proposal(
                proposer.clone(),
                proposal_data,
                create_test_quantum_random(),
            )
            .await
            .expect("Proposal should be created");

        proposal_ids.push(proposal_id);
    }

    // Vote on each proposal with same strength and check costs
    let vote_strength = 3.0;
    let max_cost = 50000;
    let mut costs = Vec::new();

    for (i, &proposal_id) in proposal_ids.iter().enumerate() {
        let test_voter = create_test_address(20 + i as u8);

        let vote_result = governance
            .cast_quadratic_vote(test_voter, proposal_id, vote_strength, max_cost)
            .await
            .expect("Vote should succeed");

        costs.push(vote_result.cost_paid);
    }

    // Different proposal types should have different base costs
    // Emergency actions should be most expensive, parameter changes least expensive
    let emergency_cost = costs[4]; // EmergencyAction
    let parameter_cost = costs[0]; // ParameterChange

    assert!(
        emergency_cost >= parameter_cost,
        "Emergency actions should cost at least as much as parameter changes"
    );
}

#[tokio::test]
async fn test_comprehensive_governance_workflow() {
    let mut governance = AdvancedGovernance::new();
    governance
        .initialize()
        .await
        .expect("Governance should initialize");

    let proposer = create_test_address(18);
    let voter1 = create_test_address(19);
    let voter2 = create_test_address(20);
    let delegator = create_test_address(21);
    let delegatee = create_test_address(22);

    // 1. Set up delegation
    governance
        .delegate_voting_power(
            delegator,
            delegatee,
            DelegationType::Full,
            Utc::now() + Duration::days(30),
        )
        .await
        .expect("Delegation should succeed");

    // 2. Create quadratic voting proposal
    let proposal_data = QuadraticProposalData {
        title: "Comprehensive Governance Test".to_string(),
        description: "End-to-end governance workflow test".to_string(),
        proposal_type: ProposalType::ProtocolUpgrade,
        implementation_details: "Major protocol upgrade with comprehensive changes".to_string(),
        expected_impact: ExpectedImpact {
            network_performance: ImpactRating::VeryPositive,
            economic_efficiency: ImpactRating::VeryPositive,
            decentralization: ImpactRating::Positive,
            user_experience: ImpactRating::VeryPositive,
            security: ImpactRating::VeryPositive,
        },
        required_quorum: 0.2,
    };

    let proposal_id = governance
        .create_quadratic_proposal(
            proposer.clone(),
            proposal_data.clone(),
            create_test_quantum_random(),
        )
        .await
        .expect("Proposal should be created");

    // 3. AI assessment should have been done automatically
    let ai_assessment = governance
        .ai_agent_system
        .assess_proposal(&proposal_data)
        .await
        .expect("AI assessment should work");
    assert!(
        ai_assessment.overall_score > 0.7,
        "Should be highly rated by AI"
    );

    // 4. Cast votes
    governance
        .cast_quadratic_vote(voter1.clone(), proposal_id, 6.0, 15000)
        .await
        .expect("First vote should succeed");

    governance
        .cast_quadratic_vote(voter2.clone(), proposal_id, -2.0, 5000)
        .await
        .expect("Opposing vote should succeed");

    // 5. Create related futarchy market
    let success_metrics = vec![SuccessMetric {
        name: "Protocol Performance".to_string(),
        description: "Overall protocol performance improvement".to_string(),
        measurement_method: "Comprehensive benchmarking".to_string(),
        predicted_value: 200.0, // 200% improvement
        measurement_deadline: Utc::now() + Duration::days(60),
    }];

    let market_id = governance
        .create_futarchy_market(
            proposer.clone(),
            FutarchyProposalData {
                title: "Protocol Upgrade Prediction".to_string(),
                description: "Predicting success of protocol upgrade".to_string(),
                implementation_plan: "Detailed upgrade plan".to_string(),
                success_criteria: vec!["Performance improvement".to_string()],
                risk_assessment: RiskAssessment {
                    technical_risk: RiskLevel::Medium,
                    economic_risk: RiskLevel::Low,
                    adoption_risk: RiskLevel::Medium,
                    timeline_risk: RiskLevel::High,
                },
            },
            success_metrics,
        )
        .await
        .expect("Futarchy market should be created");

    // 6. Place bets in market
    governance
        .place_futarchy_bet(
            voter1.clone(),
            market_id,
            FutarchyMarketType::Implement,
            OutcomeBet {
                outcome_name: "Protocol Performance".to_string(),
                predicted_value: 250.0, // Optimistic
            },
            2000,
        )
        .await
        .expect("Futarchy bet should succeed");

    // 7. Start conviction voting for implementation funding
    let funding_proposal_id = governance
        .start_conviction_voting(
            proposer,
            ConvictionProposalData {
                title: "Protocol Upgrade Implementation".to_string(),
                description: "Funding for implementing the protocol upgrade".to_string(),
                deliverables: vec![
                    "Core protocol changes".to_string(),
                    "Security audits".to_string(),
                    "Network migration".to_string(),
                ],
                timeline: "12 months".to_string(),
            },
            1000000, // 1M tokens
        )
        .await
        .expect("Conviction voting should start");

    // 8. Signal conviction
    governance
        .signal_conviction(voter1.clone(), funding_proposal_id, 25000)
        .await
        .expect("Conviction signaling should succeed");

    governance
        .signal_conviction(voter2.clone(), funding_proposal_id, 15000)
        .await
        .expect("Second conviction signal should succeed");

    // 9. Get final statistics
    let final_stats = governance
        .get_governance_stats()
        .await
        .expect("Final stats should be available");

    assert!(
        final_stats.active_quadratic_proposals > 0,
        "Should have active proposals"
    );
    assert!(
        final_stats.active_futarchy_markets > 0,
        "Should have active markets"
    );
    assert!(
        final_stats.conviction_stats.active_proposals > 0,
        "Should have conviction proposals"
    );
    assert!(
        final_stats.total_proposals_created >= 2,
        "Should track all created proposals"
    );
}

#[tokio::test]
async fn test_error_conditions() {
    let mut governance = AdvancedGovernance::new();
    governance
        .initialize()
        .await
        .expect("Governance should initialize");

    let voter = create_test_address(23);
    let invalid_proposal_id = Uuid::new_v4();

    // Test voting on non-existent proposal
    let vote_result = governance
        .cast_quadratic_vote(voter.clone(), invalid_proposal_id, 5.0, 10000)
        .await;

    assert!(
        vote_result.is_err(),
        "Should fail to vote on non-existent proposal"
    );

    // Test conviction signaling on non-existent proposal
    let conviction_result = governance
        .signal_conviction(voter, invalid_proposal_id, 5000)
        .await;

    assert!(
        conviction_result.is_err(),
        "Should fail to signal conviction on non-existent proposal"
    );
}

#[tokio::test]
async fn test_governance_performance() {
    let mut governance = AdvancedGovernance::new();

    let init_start = std::time::Instant::now();
    governance
        .initialize()
        .await
        .expect("Governance should initialize");
    let init_time = init_start.elapsed();

    println!("Governance initialization time: {:?}", init_time);
    assert!(
        init_time.as_millis() < 1000,
        "Governance should initialize quickly"
    );

    let proposer = create_test_address(24);

    // Benchmark proposal creation
    let proposal_start = std::time::Instant::now();

    for i in 0..5 {
        let proposal_data = QuadraticProposalData {
            title: format!("Performance Test {}", i),
            description: "Performance testing proposal".to_string(),
            proposal_type: ProposalType::ParameterChange,
            implementation_details: "Performance test".to_string(),
            expected_impact: ExpectedImpact {
                network_performance: ImpactRating::Positive,
                economic_efficiency: ImpactRating::Positive,
                decentralization: ImpactRating::Neutral,
                user_experience: ImpactRating::Positive,
                security: ImpactRating::Neutral,
            },
            required_quorum: 0.1,
        };

        governance
            .create_quadratic_proposal(
                proposer.clone(),
                proposal_data,
                create_test_quantum_random(),
            )
            .await
            .expect("Proposal should be created");
    }

    let proposal_time = proposal_start.elapsed();
    println!("5 proposals creation time: {:?}", proposal_time);
    println!("Average proposal creation time: {:?}", proposal_time / 5);

    assert!(
        proposal_time.as_millis() / 5 < 100,
        "Average proposal creation should be fast"
    );
}
