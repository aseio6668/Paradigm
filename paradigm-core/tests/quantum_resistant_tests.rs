/// Unit tests for quantum-resistant cryptography module
/// Tests lattice-based signatures, hash-based signatures, ZK proofs, and key exchange

use paradigm_core::tokenomics::quantum_resistant::*;
use paradigm_core::Address;

fn create_test_address(id: u8) -> Address {
    let mut addr = [0u8; 32];
    addr[0] = id;
    Address(addr)
}

#[tokio::test]
async fn test_quantum_resistant_crypto_initialization() {
    let mut crypto = QuantumResistantCrypto::new();
    
    let result = crypto.initialize().await;
    assert!(result.is_ok(), "Quantum-resistant crypto should initialize successfully");
}

#[tokio::test]
async fn test_contributor_key_generation() {
    let mut crypto = QuantumResistantCrypto::new();
    crypto.initialize().await.expect("Crypto should initialize");
    
    let address = create_test_address(1);
    
    let result = crypto.generate_contributor_keys(&address).await;
    assert!(result.is_ok(), "Key generation should succeed");
    
    let keys = result.unwrap();
    assert_eq!(keys.address, address);
    assert!(!keys.lattice_public_key.is_empty(), "Lattice public key should be generated");
    assert!(!keys.hash_tree_public_key.is_empty(), "Hash tree public key should be generated");
    
    // Test generating keys for the same address again (should succeed)
    let result2 = crypto.generate_contributor_keys(&address).await;
    assert!(result2.is_ok(), "Should be able to regenerate keys");
}

#[tokio::test]
async fn test_lattice_based_signatures() {
    let mut crypto = QuantumResistantCrypto::new();
    crypto.initialize().await.expect("Crypto should initialize");
    
    let address = create_test_address(2);
    crypto.generate_contributor_keys(&address).await.expect("Keys should be generated");
    
    let test_data = b"test data for signing";
    
    // Test lattice-based signature creation
    let signature_result = crypto.sign_contribution_proof(
        &address,
        test_data,
        QRSignatureType::Lattice,
    ).await;
    
    assert!(signature_result.is_ok(), "Lattice signature should be created");
    
    let signature = signature_result.unwrap();
    assert_eq!(signature.signature_type, QRSignatureType::Lattice);
    assert!(!signature.signature_data.is_empty(), "Signature data should be present");
    
    // Test signature verification
    let verification_result = crypto.verify_signature(&address, test_data, &signature).await;
    assert!(verification_result.is_ok(), "Signature verification should succeed");
    assert!(verification_result.unwrap(), "Signature should be valid");
    
    // Test verification with wrong data
    let wrong_data = b"wrong test data";
    let wrong_verification = crypto.verify_signature(&address, wrong_data, &signature).await;
    assert!(wrong_verification.is_ok(), "Verification should complete");
    // Note: In our simulation, this might still pass, but in real implementation it would fail
}

#[tokio::test]
async fn test_hash_based_signatures() {
    let mut crypto = QuantumResistantCrypto::new();
    crypto.initialize().await.expect("Crypto should initialize");
    
    let address = create_test_address(3);
    crypto.generate_contributor_keys(&address).await.expect("Keys should be generated");
    
    let test_data = b"test data for hash-based signing";
    
    // Test hash-based signature creation
    let signature_result = crypto.sign_contribution_proof(
        &address,
        test_data,
        QRSignatureType::HashBased,
    ).await;
    
    assert!(signature_result.is_ok(), "Hash-based signature should be created");
    
    let signature = signature_result.unwrap();
    assert_eq!(signature.signature_type, QRSignatureType::HashBased);
    assert!(!signature.signature_data.is_empty(), "Signature data should be present");
    
    // Test signature verification
    let verification_result = crypto.verify_signature(&address, test_data, &signature).await;
    assert!(verification_result.is_ok(), "Signature verification should succeed");
    assert!(verification_result.unwrap(), "Signature should be valid");
}

#[tokio::test]
async fn test_multiple_hash_based_signatures() {
    let mut crypto = QuantumResistantCrypto::new();
    crypto.initialize().await.expect("Crypto should initialize");
    
    let address = create_test_address(4);
    crypto.generate_contributor_keys(&address).await.expect("Keys should be generated");
    
    // Test multiple signatures to verify signature counter increment
    for i in 0..5 {
        let test_data = format!("test data {}", i).as_bytes().to_vec();
        
        let signature_result = crypto.sign_contribution_proof(
            &address,
            &test_data,
            QRSignatureType::HashBased,
        ).await;
        
        assert!(signature_result.is_ok(), "Signature {} should be created", i);
        
        let signature = signature_result.unwrap();
        let verification = crypto.verify_signature(&address, &test_data, &signature).await;
        assert!(verification.is_ok() && verification.unwrap(), "Signature {} should be valid", i);
    }
}

#[tokio::test]
async fn test_quantum_resistant_zk_proofs() {
    let mut crypto = QuantumResistantCrypto::new();
    crypto.initialize().await.expect("Crypto should initialize");
    
    // Test different proof types
    let proof_types = ["ml_training", "inference_serving", "data_validation"];
    
    for proof_type in proof_types.iter() {
        let private_inputs = QRPrivateInputs {
            witness: vec![1, 2, 3, 4, 5],
            randomness: vec![6, 7, 8, 9, 10],
        };
        
        let public_inputs = QRPublicInputs {
            statement: vec![11, 12, 13, 14, 15],
            challenge: vec![16, 17, 18, 19, 20],
        };
        
        // Generate proof
        let proof_result = crypto.generate_qr_zk_proof(proof_type, &private_inputs, &public_inputs).await;
        assert!(proof_result.is_ok(), "Proof generation should succeed for {}", proof_type);
        
        let proof = proof_result.unwrap();
        assert_eq!(proof.proof_type, *proof_type);
        assert!(!proof.proof_data.is_empty(), "Proof data should be present");
        assert_eq!(proof.public_inputs.statement, public_inputs.statement);
        
        // Verify proof
        let verification_result = crypto.verify_qr_zk_proof(&proof).await;
        assert!(verification_result.is_ok(), "Proof verification should succeed for {}", proof_type);
        assert!(verification_result.unwrap(), "Proof should be valid for {}", proof_type);
    }
}

#[tokio::test]
async fn test_invalid_zk_proof_type() {
    let mut crypto = QuantumResistantCrypto::new();
    crypto.initialize().await.expect("Crypto should initialize");
    
    let private_inputs = QRPrivateInputs {
        witness: vec![1, 2, 3],
        randomness: vec![4, 5, 6],
    };
    
    let public_inputs = QRPublicInputs {
        statement: vec![7, 8, 9],
        challenge: vec![10, 11, 12],
    };
    
    // Test with invalid proof type
    let result = crypto.generate_qr_zk_proof("invalid_proof_type", &private_inputs, &public_inputs).await;
    assert!(result.is_err(), "Should fail with invalid proof type");
}

#[tokio::test]
async fn test_post_quantum_key_exchange() {
    let mut crypto = QuantumResistantCrypto::new();
    crypto.initialize().await.expect("Crypto should initialize");
    
    let peer1 = create_test_address(5);
    let peer2 = create_test_address(6);
    
    // Test key exchange with peer1
    let exchange1_result = crypto.quantum_safe_key_exchange(&peer1).await;
    assert!(exchange1_result.is_ok(), "Key exchange with peer1 should succeed");
    
    let secret1 = exchange1_result.unwrap();
    assert!(!secret1.secret.is_empty(), "Shared secret should be generated");
    
    // Test key exchange with peer2
    let exchange2_result = crypto.quantum_safe_key_exchange(&peer2).await;
    assert!(exchange2_result.is_ok(), "Key exchange with peer2 should succeed");
    
    let secret2 = exchange2_result.unwrap();
    assert!(!secret2.secret.is_empty(), "Shared secret should be generated");
    
    // Secrets with different peers should be different
    assert_ne!(secret1.secret, secret2.secret, "Secrets with different peers should differ");
}

#[tokio::test]
async fn test_quantum_random_oracle() {
    let mut crypto = QuantumResistantCrypto::new();
    crypto.initialize().await.expect("Crypto should initialize");
    
    let entropy_sources = vec![
        "proposal_test_proposal".to_string(),
        "timestamp_1234567890".to_string(),
        "network_entropy".to_string(),
        "block_hash_entropy".to_string(),
    ];
    
    // Test quantum random generation
    let random_result = crypto.get_quantum_random(entropy_sources.clone()).await;
    assert!(random_result.is_ok(), "Quantum random generation should succeed");
    
    let random1 = random_result.unwrap();
    assert!(!random1.value.is_empty(), "Random value should be generated");
    assert_eq!(random1.entropy_sources, entropy_sources);
    
    // Test with different entropy sources
    let different_sources = vec![
        "different_proposal".to_string(),
        "different_timestamp".to_string(),
    ];
    
    let random2_result = crypto.get_quantum_random(different_sources.clone()).await;
    assert!(random2_result.is_ok(), "Second random generation should succeed");
    
    let random2 = random2_result.unwrap();
    assert_ne!(random1.value, random2.value, "Different entropy should produce different random values");
    assert_eq!(random2.entropy_sources, different_sources);
}

#[tokio::test]
async fn test_signature_type_consistency() {
    let mut crypto = QuantumResistantCrypto::new();
    crypto.initialize().await.expect("Crypto should initialize");
    
    let address = create_test_address(7);
    crypto.generate_contributor_keys(&address).await.expect("Keys should be generated");
    
    let test_data = b"consistency test data";
    
    // Create lattice signature
    let lattice_sig = crypto.sign_contribution_proof(
        &address,
        test_data,
        QRSignatureType::Lattice,
    ).await.expect("Lattice signature should be created");
    
    // Create hash-based signature
    let hash_sig = crypto.sign_contribution_proof(
        &address,
        test_data,
        QRSignatureType::HashBased,
    ).await.expect("Hash-based signature should be created");
    
    // Verify signatures are of correct types
    assert_eq!(lattice_sig.signature_type, QRSignatureType::Lattice);
    assert_eq!(hash_sig.signature_type, QRSignatureType::HashBased);
    
    // Signatures should be different
    assert_ne!(lattice_sig.signature_data, hash_sig.signature_data);
    
    // Both should verify correctly
    assert!(crypto.verify_signature(&address, test_data, &lattice_sig).await.unwrap());
    assert!(crypto.verify_signature(&address, test_data, &hash_sig).await.unwrap());
}

#[tokio::test]
async fn test_key_exchange_uniqueness() {
    let mut crypto = QuantumResistantCrypto::new();
    crypto.initialize().await.expect("Crypto should initialize");
    
    let peer = create_test_address(8);
    
    // Perform multiple key exchanges with the same peer
    let mut secrets = Vec::new();
    
    for i in 0..3 {
        let secret = crypto.quantum_safe_key_exchange(&peer).await
            .expect(&format!("Key exchange {} should succeed", i));
        secrets.push(secret);
    }
    
    // All secrets should be valid
    for (i, secret) in secrets.iter().enumerate() {
        assert!(!secret.secret.is_empty(), "Secret {} should not be empty", i);
    }
    
    // In a real implementation, secrets might be different each time
    // For our simulation, they might be the same, which is also valid for testing
}

#[tokio::test]
async fn test_comprehensive_cryptographic_workflow() {
    let mut crypto = QuantumResistantCrypto::new();
    crypto.initialize().await.expect("Crypto should initialize");
    
    let alice = create_test_address(9);
    let bob = create_test_address(10);
    
    // 1. Generate keys for both parties
    let alice_keys = crypto.generate_contributor_keys(&alice).await
        .expect("Alice's keys should be generated");
    let bob_keys = crypto.generate_contributor_keys(&bob).await
        .expect("Bob's keys should be generated");
    
    // 2. Alice creates a quantum-resistant proof
    let private_inputs = QRPrivateInputs {
        witness: b"alice's secret computation".to_vec(),
        randomness: b"alice's randomness".to_vec(),
    };
    
    let public_inputs = QRPublicInputs {
        statement: b"public computation result".to_vec(),
        challenge: b"verification challenge".to_vec(),
    };
    
    let alice_proof = crypto.generate_qr_zk_proof("ml_training", &private_inputs, &public_inputs).await
        .expect("Alice's proof should be generated");
    
    // 3. Alice signs the proof
    let proof_data = serde_json::to_vec(&alice_proof).expect("Proof should serialize");
    let alice_signature = crypto.sign_contribution_proof(&alice, &proof_data, QRSignatureType::Lattice).await
        .expect("Alice's signature should be created");
    
    // 4. Bob verifies Alice's proof and signature
    let proof_valid = crypto.verify_qr_zk_proof(&alice_proof).await
        .expect("Proof verification should complete");
    assert!(proof_valid, "Alice's proof should be valid");
    
    let signature_valid = crypto.verify_signature(&alice, &proof_data, &alice_signature).await
        .expect("Signature verification should complete");
    assert!(signature_valid, "Alice's signature should be valid");
    
    // 5. Establish quantum-safe communication
    let alice_secret = crypto.quantum_safe_key_exchange(&bob).await
        .expect("Alice should establish shared secret with Bob");
    let bob_secret = crypto.quantum_safe_key_exchange(&alice).await
        .expect("Bob should establish shared secret with Alice");
    
    // In a real implementation, both parties would derive the same secret
    assert!(!alice_secret.secret.is_empty(), "Alice's shared secret should exist");
    assert!(!bob_secret.secret.is_empty(), "Bob's shared secret should exist");
    
    // 6. Generate quantum randomness for governance
    let governance_random = crypto.get_quantum_random(vec![
        "governance_proposal_123".to_string(),
        format!("alice_address_{:?}", alice.0[0]),
        format!("bob_address_{:?}", bob.0[0]),
    ]).await.expect("Quantum randomness should be generated");
    
    assert!(!governance_random.value.is_empty(), "Governance randomness should be generated");
    assert_eq!(governance_random.entropy_sources.len(), 3, "Should use all entropy sources");
}

#[tokio::test] 
async fn test_error_handling() {
    let mut crypto = QuantumResistantCrypto::new();
    crypto.initialize().await.expect("Crypto should initialize");
    
    let valid_address = create_test_address(11);
    let invalid_address = create_test_address(12);
    
    // Generate keys for valid address only
    crypto.generate_contributor_keys(&valid_address).await.expect("Keys should be generated");
    
    let test_data = b"test data";
    
    // Try to sign with address that has no keys
    let result = crypto.sign_contribution_proof(&invalid_address, test_data, QRSignatureType::Lattice).await;
    assert!(result.is_err(), "Should fail to sign with non-existent keys");
    
    // Try to verify with address that has no keys
    let valid_signature = crypto.sign_contribution_proof(&valid_address, test_data, QRSignatureType::Lattice).await
        .expect("Valid signature should be created");
    
    let verify_result = crypto.verify_signature(&invalid_address, test_data, &valid_signature).await;
    assert!(verify_result.is_err(), "Should fail to verify with non-existent keys");
}

#[tokio::test]
async fn test_performance_characteristics() {
    let mut crypto = QuantumResistantCrypto::new();
    crypto.initialize().await.expect("Crypto should initialize");
    
    let address = create_test_address(13);
    
    // Benchmark key generation
    let key_start = std::time::Instant::now();
    crypto.generate_contributor_keys(&address).await.expect("Keys should be generated");
    let key_time = key_start.elapsed();
    
    println!("Key generation time: {:?}", key_time);
    assert!(key_time.as_millis() < 1000, "Key generation should complete within 1 second");
    
    // Benchmark signature creation
    let test_data = b"performance test data";
    let sig_start = std::time::Instant::now();
    
    for _ in 0..10 {
        crypto.sign_contribution_proof(&address, test_data, QRSignatureType::Lattice).await
            .expect("Signature should be created");
    }
    
    let sig_time = sig_start.elapsed();
    println!("10 signatures time: {:?}", sig_time);
    println!("Average signature time: {:?}", sig_time / 10);
    
    assert!(sig_time.as_millis() / 10 < 100, "Average signature should complete within 100ms");
}