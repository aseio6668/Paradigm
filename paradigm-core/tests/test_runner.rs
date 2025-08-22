use chrono::Utc;
/// Test runner configuration and basic test utilities
/// Provides common setup and utilities for all tests
use paradigm_core::tokenomics::*;
use paradigm_core::Address;
use uuid::Uuid;

/// Create a test address with a specific ID
pub fn create_test_address(id: u8) -> Address {
    let mut addr = [0u8; 32];
    addr[0] = id;
    Address(addr)
}

/// Create a basic contribution proof for testing
pub fn create_test_contribution_proof(
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
async fn test_basic_system_functionality() {
    let mut system = TokenomicsSystem::new();

    // Test basic initialization
    let result = system.start().await;
    assert!(result.is_ok(), "System should initialize");

    // Test basic contribution processing
    let contributor = create_test_address(1);
    let proof = create_test_contribution_proof(contributor.clone(), ContributionType::MLTraining);

    let contribution_result = system.process_contribution(&contributor, proof).await;
    assert!(
        contribution_result.is_ok(),
        "Basic contribution should work"
    );
}

#[tokio::test]
async fn test_quantum_resistant_basic() {
    let mut system = TokenomicsSystem::new();
    system.start().await.expect("System should start");

    let contributor = create_test_address(2);

    // Test basic quantum-resistant key generation
    let keys_result = system.generate_quantum_resistant_keys(&contributor).await;
    assert!(keys_result.is_ok(), "QR key generation should work");
}

#[tokio::test]
async fn test_governance_basic() {
    let mut system = TokenomicsSystem::new();
    system.start().await.expect("System should start");

    let proposer = create_test_address(3);

    // Test basic governance proposal
    let expected_impact = ExpectedImpact {
        network_performance: ImpactRating::Positive,
        economic_efficiency: ImpactRating::Positive,
        decentralization: ImpactRating::Neutral,
        user_experience: ImpactRating::Positive,
        security: ImpactRating::Positive,
    };

    let proposal_result = system
        .create_governance_proposal(
            &proposer,
            "Test Proposal".to_string(),
            "Basic test proposal".to_string(),
            ProposalType::ParameterChange,
            expected_impact,
        )
        .await;

    assert!(
        proposal_result.is_ok(),
        "Basic governance proposal should work"
    );
}

#[tokio::test]
async fn test_system_statistics() {
    let mut system = TokenomicsSystem::new();
    system.start().await.expect("System should start");

    // Test governance statistics
    let gov_stats = system.get_governance_statistics().await;
    assert!(gov_stats.is_ok(), "Governance stats should be available");

    let stats = gov_stats.unwrap();
    assert!(
        stats.governance_participation_rate >= 0.0,
        "Participation rate should be valid"
    );
}
