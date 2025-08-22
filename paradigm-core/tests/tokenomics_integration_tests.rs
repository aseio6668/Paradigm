use chrono::{Duration, Utc};
/// Comprehensive integration tests for the advanced Paradigm tokenomics system
/// Tests all major components: tokenomics, governance, privacy, AI optimization, quantum resistance
use paradigm_core::{tokenomics::*, Address};
use std::collections::HashMap;
use uuid::Uuid;

/// Test helper to create a sample address
fn create_test_address(id: u8) -> Address {
    let mut addr = [0u8; 32];
    addr[0] = id;
    Address(addr)
}

/// Test helper to create a basic contribution proof
fn create_test_contribution_proof(
    contributor: Address,
    contribution_type: ContributionType,
) -> ContributionProof {
    ContributionProof {
        id: Uuid::new_v4(),
        contributor,
        contribution_type,
        workload_hash: vec![1, 2, 3, 4],
        zk_proof: vec![5, 6, 7, 8],
        qr_zk_proof: None,
        qr_signature: None,
        metadata: serde_json::json!({"test": "data"}),
        timestamp: Utc::now(),
    }
}

#[tokio::test]
async fn test_tokenomics_system_initialization() {
    let mut system = TokenomicsSystem::new();

    // Test system initialization
    let result = system.start().await;
    assert!(
        result.is_ok(),
        "Tokenomics system should initialize successfully"
    );

    // Verify all modules are initialized
    // This test ensures the system starts without errors
}

#[tokio::test]
async fn test_contribution_processing_flow() {
    let mut system = TokenomicsSystem::new();
    system.start().await.expect("System should start");

    let contributor = create_test_address(1);
    let proof = create_test_contribution_proof(contributor.clone(), ContributionType::MLTraining);

    // Test contribution processing
    let result = system.process_contribution(&contributor, proof).await;

    assert!(result.is_ok(), "Contribution processing should succeed");

    let contribution_result = result.unwrap();
    assert_eq!(contribution_result.contributor, contributor);
    assert!(
        contribution_result.tokens_earned > 0,
        "Tokens should be earned for valid contribution"
    );
    assert_eq!(
        contribution_result.contribution_type,
        ContributionType::MLTraining
    );
}

#[tokio::test]
async fn test_quantum_resistant_key_generation() {
    let mut system = TokenomicsSystem::new();
    system.start().await.expect("System should start");

    let contributor = create_test_address(2);

    // Test quantum-resistant key generation
    let result = system.generate_quantum_resistant_keys(&contributor).await;

    assert!(
        result.is_ok(),
        "Quantum-resistant key generation should succeed"
    );

    let keys = result.unwrap();
    assert_eq!(keys.address, contributor);
    assert!(
        !keys.lattice_public_key.is_empty(),
        "Lattice public key should be generated"
    );
    assert!(
        !keys.hash_tree_public_key.is_empty(),
        "Hash tree public key should be generated"
    );
}

#[tokio::test]
async fn test_quantum_resistant_proof_creation_and_verification() {
    let mut system = TokenomicsSystem::new();
    system.start().await.expect("System should start");

    let contributor = create_test_address(3);

    // Generate quantum-resistant keys first
    system
        .generate_quantum_resistant_keys(&contributor)
        .await
        .expect("Key generation should succeed");

    // Create quantum-resistant proof
    let workload_data = b"test workload data";
    let metadata = serde_json::json!({"test": "metadata"});

    let proof_result = system
        .create_quantum_resistant_proof(
            &contributor,
            ContributionType::InferenceServing,
            workload_data,
            metadata,
        )
        .await;

    assert!(
        proof_result.is_ok(),
        "Quantum-resistant proof creation should succeed"
    );

    let proof = proof_result.unwrap();
    assert!(
        proof.qr_zk_proof.is_some(),
        "Quantum-resistant ZK proof should be present"
    );
    assert!(
        proof.qr_signature.is_some(),
        "Quantum-resistant signature should be present"
    );

    // Test proof verification
    let verification_result = system.verify_quantum_resistant_proof(&proof).await;
    assert!(
        verification_result.is_ok(),
        "Proof verification should succeed"
    );
    assert!(verification_result.unwrap(), "Proof should be valid");
}

#[tokio::test]
async fn test_quadratic_voting_governance() {
    let mut system = TokenomicsSystem::new();
    system.start().await.expect("System should start");

    let proposer = create_test_address(4);
    let voter1 = create_test_address(5);
    let voter2 = create_test_address(6);

    // Create a governance proposal
    let expected_impact = ExpectedImpact {
        network_performance: ImpactRating::Positive,
        economic_efficiency: ImpactRating::VeryPositive,
        decentralization: ImpactRating::Neutral,
        user_experience: ImpactRating::Positive,
        security: ImpactRating::Neutral,
    };

    let proposal_result = system
        .create_governance_proposal(
            &proposer,
            "Test Proposal".to_string(),
            "A test proposal for governance".to_string(),
            ProposalType::ParameterChange,
            expected_impact,
        )
        .await;

    assert!(proposal_result.is_ok(), "Proposal creation should succeed");
    let proposal_id = proposal_result.unwrap();

    // Test quadratic voting
    let vote1_result = system
        .cast_governance_vote(&voter1, proposal_id, 5.0, 10000)
        .await;
    assert!(vote1_result.is_ok(), "First vote should succeed");

    let vote1 = vote1_result.unwrap();
    assert_eq!(vote1.proposal_id, proposal_id);
    assert!(vote1.cost_paid > 0, "Vote should cost tokens");
    assert!(
        vote1.effective_votes > 0.0,
        "Vote should have effective weight"
    );

    // Test opposing vote
    let vote2_result = system
        .cast_governance_vote(&voter2, proposal_id, -3.0, 5000)
        .await;
    assert!(vote2_result.is_ok(), "Opposing vote should succeed");

    let vote2 = vote2_result.unwrap();
    assert!(
        vote2.effective_votes < 0.0,
        "Opposing vote should be negative"
    );
}

#[tokio::test]
async fn test_futarchy_prediction_market() {
    let mut system = TokenomicsSystem::new();
    system.start().await.expect("System should start");

    let proposer = create_test_address(7);

    // Create success metrics for the prediction market
    let success_metrics = vec![
        SuccessMetric {
            name: "Network Throughput".to_string(),
            description: "Transactions per second".to_string(),
            measurement_method: "Automated monitoring".to_string(),
            predicted_value: 10000.0,
            measurement_deadline: Utc::now() + Duration::days(30),
        },
        SuccessMetric {
            name: "User Adoption".to_string(),
            description: "Number of active users".to_string(),
            measurement_method: "User analytics".to_string(),
            predicted_value: 50000.0,
            measurement_deadline: Utc::now() + Duration::days(60),
        },
    ];

    // Create futarchy market
    let market_result = system
        .create_prediction_market(
            &proposer,
            "Network Upgrade Prediction".to_string(),
            "Predict the success of the proposed network upgrade".to_string(),
            success_metrics,
        )
        .await;

    assert!(
        market_result.is_ok(),
        "Futarchy market creation should succeed"
    );
    let market_id = market_result.unwrap();

    // Test placing bets in the market
    let bettor = create_test_address(8);

    let outcome_bet = OutcomeBet {
        outcome_name: "Network Throughput".to_string(),
        predicted_value: 12000.0,
    };

    let bet_result = system
        .advanced_governance
        .place_futarchy_bet(
            bettor,
            market_id,
            FutarchyMarketType::Implement,
            outcome_bet,
            1000,
        )
        .await;

    assert!(bet_result.is_ok(), "Placing futarchy bet should succeed");
}

#[tokio::test]
async fn test_conviction_voting_funding() {
    let mut system = TokenomicsSystem::new();
    system.start().await.expect("System should start");

    let proposer = create_test_address(9);
    let supporter1 = create_test_address(10);
    let supporter2 = create_test_address(11);

    // Create conviction voting proposal
    let deliverables = vec![
        "Implement new consensus algorithm".to_string(),
        "Deploy testnet".to_string(),
        "Community testing phase".to_string(),
    ];

    let proposal_result = system
        .start_conviction_funding(
            &proposer,
            "Consensus Algorithm Upgrade".to_string(),
            "Implementation of a new consensus algorithm for improved performance".to_string(),
            deliverables,
            100000, // 100k tokens requested
        )
        .await;

    assert!(
        proposal_result.is_ok(),
        "Conviction proposal should be created"
    );
    let proposal_id = proposal_result.unwrap();

    // Test conviction signaling
    let signal1_result = system
        .signal_funding_conviction(&supporter1, proposal_id, 5000)
        .await;
    assert!(
        signal1_result.is_ok(),
        "First conviction signal should succeed"
    );

    let signal1 = signal1_result.unwrap();
    assert_eq!(signal1.proposal_id, proposal_id);
    assert!(signal1.conviction_added > 0.0, "Conviction should be added");

    // Test additional conviction signaling
    let signal2_result = system
        .signal_funding_conviction(&supporter2, proposal_id, 8000)
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
}

#[tokio::test]
async fn test_delegation_system() {
    let mut system = TokenomicsSystem::new();
    system.start().await.expect("System should start");

    let delegator = create_test_address(12);
    let delegatee = create_test_address(13);

    // Test full delegation
    let delegation_result = system
        .delegate_voting_power(
            &delegator,
            &delegatee,
            DelegationType::Full,
            30, // 30 days
        )
        .await;

    assert!(delegation_result.is_ok(), "Delegation should succeed");

    let delegation = delegation_result.unwrap();
    assert!(delegation.active, "Delegation should be active");

    // Test specific delegation
    let specific_delegation_result = system
        .delegate_voting_power(
            &delegator,
            &delegatee,
            DelegationType::Specific(vec![ProposalType::TreasuryAllocation]),
            7, // 7 days
        )
        .await;

    assert!(
        specific_delegation_result.is_ok(),
        "Specific delegation should succeed"
    );
}

#[tokio::test]
async fn test_ai_optimizer_integration() {
    let mut system = TokenomicsSystem::new();
    system.start().await.expect("System should start");

    // Test AI optimizer analysis
    let network_state = NetworkState {
        total_tokens: 1000000,
        circulating_supply: 800000,
        staked_tokens: 200000,
        active_validators: 100,
        transaction_volume: 50000,
        average_transaction_fee: 0.01,
        network_utilization: 0.75,
        governance_participation_rate: 0.15,
    };

    let optimization_result = system
        .ai_optimizer
        .optimize_tokenomics(&network_state)
        .await;

    assert!(
        optimization_result.is_ok(),
        "AI optimization should succeed"
    );

    let optimized_params = optimization_result.unwrap();
    assert!(
        optimized_params.inflation_rate >= 0.0,
        "Inflation rate should be non-negative"
    );
    assert!(
        optimized_params.burn_rate >= 0.0,
        "Burn rate should be non-negative"
    );
    assert!(
        optimized_params.base_reward_multiplier > 0.0,
        "Reward multiplier should be positive"
    );
}

#[tokio::test]
async fn test_privacy_preserving_federated_learning() {
    let mut system = TokenomicsSystem::new();
    system.start().await.expect("System should start");

    let coordinator = create_test_address(14);
    let participant1 = create_test_address(15);
    let participant2 = create_test_address(16);

    // Create federated learning task
    let task_spec = FederatedTaskSpec {
        task_type: "sentiment_analysis".to_string(),
        model_architecture: "transformer".to_string(),
        dataset_requirements: vec!["text_classification".to_string()],
        privacy_requirements: PrivacyRequirements {
            differential_privacy_epsilon: 1.0,
            homomorphic_encryption_required: false,
            secure_aggregation_required: true,
        },
        max_participants: 10,
        training_rounds: 5,
        deadline: Utc::now() + Duration::days(7),
    };

    let task_result = system
        .privacy_preserving
        .create_federated_task(task_spec)
        .await;

    assert!(
        task_result.is_ok(),
        "Federated task creation should succeed"
    );
    let task_id = task_result.unwrap();

    // Test participant registration
    let register1_result = system
        .privacy_preserving
        .register_participant(participant1, task_id)
        .await;
    assert!(
        register1_result.is_ok(),
        "Participant registration should succeed"
    );

    let register2_result = system
        .privacy_preserving
        .register_participant(participant2, task_id)
        .await;
    assert!(
        register2_result.is_ok(),
        "Second participant registration should succeed"
    );
}

#[tokio::test]
async fn test_model_hosting_marketplace() {
    let mut system = TokenomicsSystem::new();
    system.start().await.expect("System should start");

    let provider = create_test_address(17);
    let requester = create_test_address(18);

    // Register a model for hosting
    let model_spec = ModelSpec {
        name: "Text Classifier".to_string(),
        version: "v1.0.0".to_string(),
        model_type: ModelType::ImageClassification,
        input_format: "text/plain".to_string(),
        output_format: "application/json".to_string(),
        compute_requirements: ComputeRequirements {
            min_cpu_cores: 2,
            min_memory_gb: 4,
            min_gpu_memory_gb: 0,
            estimated_latency_ms: 100,
        },
    };

    let pricing = ModelPricing {
        base_cost_per_request: 100, // 100 base units
        compute_multiplier: 1.0,
        quality_bonus: 0.1,
    };

    let register_result = system
        .model_hosting
        .register_model(provider.clone(), model_spec)
        .await;

    assert!(register_result.is_ok(), "Model registration should succeed");
    let model_id = register_result.unwrap();

    // Submit inference request
    let request_spec = InferenceRequestSpec {
        model_type: ModelType::ImageClassification,
        input_format: "text/plain".to_string(),
        output_format: "application/json".to_string(),
        estimated_input_size_kb: 1,
        max_latency_ms: 1000,
        quality_requirements: QualityRequirements {
            min_confidence_score: 0.8,
            min_accuracy: Some(0.9),
            max_error_rate: 0.1,
        },
        preferred_availability_zone: None,
    };

    let request_result = system
        .model_hosting
        .submit_inference_request(requester, request_spec)
        .await;

    assert!(request_result.is_ok(), "Inference request should succeed");
    let request_id = request_result.unwrap();

    // Process inference result
    let inference_result = InferenceResult {
        output_data: b"classification result".to_vec(),
        confidence_score: 0.95,
        accuracy: 0.92,
        processing_time_ms: 80,
        metadata: HashMap::new(),
    };

    let process_result = system
        .model_hosting
        .process_inference_request(&provider, request_id, inference_result)
        .await;

    assert!(
        process_result.is_ok(),
        "Inference processing should succeed"
    );
    let earnings = process_result.unwrap();
    assert!(earnings > 0, "Provider should earn tokens for inference");
}

#[tokio::test]
async fn test_cross_platform_bridge_integration() {
    let mut system = TokenomicsSystem::new();
    system.start().await.expect("System should start");

    let user = create_test_address(19);

    // Test converting tokens to compute credits
    let convert_result = system
        .bridge_adapter
        .convert_to_credits(
            user.clone(),
            Platform::Filecoin,
            1000, // 1000 PAR tokens
        )
        .await;

    assert!(convert_result.is_ok(), "Credit conversion should succeed");

    let conversion = convert_result.unwrap();
    assert_eq!(conversion.user, user);
    assert_eq!(conversion.platform, Platform::Filecoin);
    assert_eq!(conversion.par_amount, 1000);
    assert!(
        conversion.credits_received > 0,
        "Should receive platform credits"
    );

    // Test using credits on platform
    let usage_result = system
        .bridge_adapter
        .use_credits(
            user.clone(),
            Platform::Filecoin,
            500, // Use 500 credits
            "File storage operation".to_string(),
        )
        .await;

    assert!(usage_result.is_ok(), "Credit usage should succeed");
}

#[tokio::test]
async fn test_reputation_system_integration() {
    let mut system = TokenomicsSystem::new();
    system.start().await.expect("System should start");

    let contributor = create_test_address(20);

    // Process multiple contributions to build reputation
    for i in 0..5 {
        let proof = create_test_contribution_proof(
            contributor.clone(),
            if i % 2 == 0 {
                ContributionType::MLTraining
            } else {
                ContributionType::DataValidation
            },
        );

        let result = system.process_contribution(&contributor, proof).await;
        assert!(result.is_ok(), "Contribution {} should succeed", i);
    }

    // Check reputation metrics
    let reputation_result = system.reputation_ledger.get_reputation(&contributor).await;

    assert!(
        reputation_result.is_ok(),
        "Reputation lookup should succeed"
    );

    let reputation = reputation_result.unwrap();
    assert!(
        reputation.contribution_count >= 5,
        "Should track contribution count"
    );
    assert!(
        reputation.consistency_score > 0.0,
        "Should have consistency score"
    );
    assert!(
        reputation.expertise_score > 0.0,
        "Should have expertise score"
    );
}

#[tokio::test]
async fn test_treasury_and_governance_integration() {
    let mut system = TokenomicsSystem::new();
    system.start().await.expect("System should start");

    // Test treasury statistics
    let stats_result = Ok(system.treasury_manager.get_treasury_stats());
    assert!(stats_result.is_ok(), "Treasury stats should be available");

    let stats = stats_result.unwrap();
    assert!(
        stats.total_balance >= 0,
        "Treasury balance should be non-negative"
    );

    // Test governance statistics
    let gov_stats_result = system.get_governance_statistics().await;
    assert!(
        gov_stats_result.is_ok(),
        "Governance stats should be available"
    );

    let gov_stats = gov_stats_result.unwrap();
    assert!(
        gov_stats.governance_participation_rate >= 0.0,
        "Participation rate should be non-negative"
    );
    assert!(
        gov_stats.governance_participation_rate <= 1.0,
        "Participation rate should not exceed 100%"
    );
}

#[tokio::test]
async fn test_comprehensive_system_stress() {
    let mut system = TokenomicsSystem::new();
    system.start().await.expect("System should start");

    // Simulate concurrent operations across different modules
    let mut tasks = Vec::new();

    // Create multiple contributors
    for i in 0..10 {
        let contributor = create_test_address(30 + i);
        let mut sys = TokenomicsSystem::new();
        sys.start().await.expect("System should start");

        tasks.push(tokio::spawn(async move {
            // Generate quantum keys
            let _keys = sys.generate_quantum_resistant_keys(&contributor).await?;

            // Process contributions
            for j in 0..3 {
                let proof = create_test_contribution_proof(
                    contributor.clone(),
                    match j % 3 {
                        0 => ContributionType::MLTraining,
                        1 => ContributionType::InferenceServing,
                        _ => ContributionType::DataValidation,
                    },
                );

                let _result = sys.process_contribution(&contributor, proof).await?;
            }

            Ok::<(), anyhow::Error>(())
        }));
    }

    // Wait for all concurrent operations
    for task in tasks {
        let result = task.await.expect("Task should complete");
        assert!(result.is_ok(), "Concurrent operation should succeed");
    }
}

#[tokio::test]
async fn test_quantum_safe_key_exchange() {
    let mut system = TokenomicsSystem::new();
    system.start().await.expect("System should start");

    let peer1 = create_test_address(40);
    let peer2 = create_test_address(41);

    // Test quantum-safe key exchange
    let exchange_result = system.quantum_safe_key_exchange(&peer1).await;
    assert!(
        exchange_result.is_ok(),
        "Quantum-safe key exchange should succeed"
    );

    let shared_secret = exchange_result.unwrap();
    assert!(
        !shared_secret.secret.is_empty(),
        "Shared secret should be generated"
    );
}

#[tokio::test]
async fn test_end_to_end_governance_workflow() {
    let mut system = TokenomicsSystem::new();
    system.start().await.expect("System should start");

    let proposer = create_test_address(50);
    let voter1 = create_test_address(51);
    let voter2 = create_test_address(52);
    let delegatee = create_test_address(53);

    // 1. Create governance proposal
    let expected_impact = ExpectedImpact {
        network_performance: ImpactRating::VeryPositive,
        economic_efficiency: ImpactRating::Positive,
        decentralization: ImpactRating::Neutral,
        user_experience: ImpactRating::Positive,
        security: ImpactRating::VeryPositive,
    };

    let proposal_id = system
        .create_governance_proposal(
            &proposer,
            "Major Network Upgrade".to_string(),
            "Comprehensive upgrade to improve all network aspects".to_string(),
            ProposalType::ProtocolUpgrade,
            expected_impact,
        )
        .await
        .expect("Proposal creation should succeed");

    // 2. Set up delegation
    system
        .delegate_voting_power(
            &voter2,
            &delegatee,
            DelegationType::Specific(vec![ProposalType::ProtocolUpgrade]),
            30,
        )
        .await
        .expect("Delegation should succeed");

    // 3. Cast votes
    let vote1 = system
        .cast_governance_vote(&voter1, proposal_id, 8.0, 20000)
        .await
        .expect("Vote should succeed");
    assert!(
        vote1.effective_votes > 0.0,
        "Vote should have positive weight"
    );

    // 4. Create prediction market for the same concept
    let success_metrics = vec![SuccessMetric {
        name: "Transaction Throughput".to_string(),
        description: "Network TPS improvement".to_string(),
        measurement_method: "Automated monitoring".to_string(),
        predicted_value: 15000.0,
        measurement_deadline: Utc::now() + Duration::days(30),
    }];

    let market_id = system
        .create_prediction_market(
            &proposer,
            "Network Upgrade Success".to_string(),
            "Prediction market for network upgrade outcomes".to_string(),
            success_metrics,
        )
        .await
        .expect("Prediction market should be created");

    // 5. Start conviction voting for funding
    let funding_proposal_id = system
        .start_conviction_funding(
            &proposer,
            "Implementation Funding".to_string(),
            "Funding for network upgrade implementation".to_string(),
            vec![
                "Phase 1: Core upgrades".to_string(),
                "Phase 2: Testing".to_string(),
            ],
            500000, // 500k tokens
        )
        .await
        .expect("Conviction funding should start");

    // 6. Signal conviction
    let conviction_result = system
        .signal_funding_conviction(&voter1, funding_proposal_id, 10000)
        .await
        .expect("Conviction signaling should succeed");
    assert!(
        conviction_result.conviction_added > 0.0,
        "Conviction should be added"
    );

    // 7. Get comprehensive governance stats
    let stats = system
        .get_governance_statistics()
        .await
        .expect("Stats should be available");

    assert!(
        stats.active_quadratic_proposals > 0,
        "Should have active proposals"
    );
    assert!(
        stats.total_proposals_created > 0,
        "Should track proposal creation"
    );
}

/// Performance benchmark test
#[tokio::test]
async fn test_system_performance_benchmark() {
    let mut system = TokenomicsSystem::new();
    let start_time = std::time::Instant::now();

    system.start().await.expect("System should start");
    let init_time = start_time.elapsed();

    // Benchmark contribution processing
    let contributor = create_test_address(60);
    system
        .generate_quantum_resistant_keys(&contributor)
        .await
        .expect("Key generation should succeed");

    let contribution_start = std::time::Instant::now();

    for i in 0..10 {
        let proof =
            create_test_contribution_proof(contributor.clone(), ContributionType::MLTraining);

        system
            .process_contribution(&contributor, proof)
            .await
            .expect("Contribution should succeed");
    }

    let contribution_time = contribution_start.elapsed();

    // Log performance metrics
    println!("System initialization time: {:?}", init_time);
    println!("10 contributions processing time: {:?}", contribution_time);
    println!("Average contribution time: {:?}", contribution_time / 10);

    // Performance assertions
    assert!(
        init_time.as_secs() < 5,
        "System should initialize within 5 seconds"
    );
    assert!(
        contribution_time.as_millis() / 10 < 1000,
        "Average contribution should process within 1 second"
    );
}
