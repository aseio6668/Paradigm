use paradigm_sdk::prelude::*;
use tokio_test;
use tempfile::TempDir;
use std::time::Duration;

/// Integration tests for the Paradigm SDK
/// These tests verify end-to-end functionality across multiple modules

#[tokio::test]
async fn test_full_transaction_lifecycle() {
    let temp_dir = TempDir::new().expect("Failed to create temp directory");
    
    // Initialize client
    let config = NetworkConfig::testnet();
    let client = ParadigmClient::new(config).await.unwrap();
    
    // Create wallets
    let sender_wallet = Wallet::create_random().unwrap();
    let recipient_wallet = Wallet::create_random().unwrap();
    
    // Test transaction creation
    let transaction = Transaction {
        hash: Hash::default(),
        from: sender_wallet.address(),
        to: recipient_wallet.address(),
        amount: Amount::from_paradigm(10.0),
        gas: 21000,
        gas_price: Amount::from_paradigm(0.00001),
        nonce: 1,
        block_hash: None,
        block_number: None,
        transaction_index: None,
        input: vec![],
    };
    
    // Sign transaction
    let signature = sender_wallet.sign_transaction(&transaction).unwrap();
    assert!(signature.r.len() == 32);
    assert!(signature.s.len() == 32);
    
    // Verify signature
    assert!(sender_wallet.verify_signature(&transaction, &signature).unwrap());
}

#[tokio::test]
async fn test_multi_signature_workflow() {
    // Create multi-sig wallet with 2-of-3 threshold
    let owner1 = ed25519_dalek::SigningKey::generate(&mut rand::thread_rng());
    let owner2 = ed25519_dalek::SigningKey::generate(&mut rand::thread_rng());
    let owner3 = ed25519_dalek::SigningKey::generate(&mut rand::thread_rng());
    
    let public_keys = vec![
        owner1.verifying_key(),
        owner2.verifying_key(), 
        owner3.verifying_key(),
    ];
    
    let multisig_wallet = MultiSigWallet::new(public_keys, 2).unwrap();
    
    // Create transaction to sign
    let message = b"test multisig transaction";
    let signers = vec![(0, &owner1), (1, &owner2)]; // 2 signers meet threshold
    
    let threshold_sig = multisig_wallet.create_transaction_signature(message, &signers).unwrap();
    assert!(threshold_sig.is_complete());
    
    // Verify multi-signature
    assert!(multisig_wallet.verify_transaction(message, &threshold_sig).unwrap());
}

#[tokio::test]
async fn test_privacy_transaction_flow() {
    let sender_secret = b"sender_private_key_32_bytes_long";
    let recipient = Address::from_hex("1234567890123456789012345678901234567890").unwrap();
    let amount = Amount::from_paradigm(50.0);
    let memo = Some(b"confidential payment".to_vec());
    
    // Create private transaction
    let private_tx = PrivateTransaction::new(sender_secret, recipient, amount, memo).unwrap();
    
    // Verify private transaction
    let circuit_hash = Hash::from_hex("deadbeef000000000000000000000000000000000000000000000000deadbeef").unwrap();
    assert!(private_tx.verify(&circuit_hash).unwrap());
    
    // Test memo decryption
    if let Some(decrypted_memo) = private_tx.decrypt_memo(sender_secret) {
        assert_eq!(decrypted_memo, b"confidential payment");
    }
}

#[tokio::test]
async fn test_security_monitoring() {
    let mut monitor = SecurityMonitor::new();
    
    // Add compliance rule
    let aml_rule = ComplianceRule {
        rule_id: Hash::default(),
        name: "Large Transaction AML".to_string(),
        description: "Flag large transactions for AML review".to_string(),
        rule_type: ComplianceType::AML,
        parameters: [("amount_threshold".to_string(), "1000000000000000000".to_string())].into(),
        enabled: true,
    };
    monitor.add_compliance_rule(aml_rule);
    
    // Create suspicious transaction
    let suspicious_tx = Transaction {
        hash: Hash::default(),
        from: Address::from_hex("1111111111111111111111111111111111111111").unwrap(),
        to: Address::from_hex("2222222222222222222222222222222222222222").unwrap(),
        amount: Amount::from_paradigm(5.0), // 5 ETH > threshold
        gas: 21000,
        gas_price: Amount::from_paradigm(0.00001),
        nonce: 1,
        block_hash: None,
        block_number: None,
        transaction_index: None,
        input: vec![],
    };
    
    // Monitor transaction
    let audits = monitor.monitor_transaction(&suspicious_tx);
    assert!(!audits.is_empty());
    
    // Verify security audit was generated
    let recent_alerts = monitor.get_recent_alerts(10);
    assert!(!recent_alerts.is_empty());
    assert_eq!(recent_alerts[0].category, SecurityCategory::ComplianceViolation);
}

#[tokio::test]
async fn test_zero_knowledge_proofs() {
    let circuit_hash = Hash::from_hex("deadbeef000000000000000000000000000000000000000000000000deadbeef").unwrap();
    let private_inputs = b"secret_witness_data";
    let public_inputs = b"public_verification_data".to_vec();
    
    // Create ZK proof
    let proof = ZKProof::new(circuit_hash, private_inputs, public_inputs).unwrap();
    
    // Verify proof
    assert!(proof.verify(&circuit_hash).unwrap());
    
    // Test with wrong circuit hash
    let wrong_circuit = Hash::from_hex("beefdead000000000000000000000000000000000000000000000000beefdead").unwrap();
    assert!(!proof.verify(&wrong_circuit).unwrap());
}

#[tokio::test]
async fn test_stealth_addresses() {
    let recipient_view_key = b"recipient_view_key_32_bytes_long";
    let recipient_spend_key = b"recipient_spend_key_32_bytes_lon";
    let tx_private_key = b"tx_private_key_32_bytes_long_key";
    
    // Generate stealth address
    let stealth = StealthAddress::generate(recipient_view_key, recipient_spend_key, tx_private_key).unwrap();
    
    // Verify ownership
    assert!(stealth.is_ours(recipient_view_key, recipient_spend_key).unwrap());
    
    // Wrong keys should not match
    let wrong_view_key = b"wrong_view_key_32_bytes_long_key";
    assert!(!stealth.is_ours(wrong_view_key, recipient_spend_key).unwrap());
    
    // Compute private key for spending
    let private_key = stealth.compute_private_key(recipient_view_key, recipient_spend_key).unwrap();
    assert_eq!(private_key.len(), 32);
}

#[tokio::test]
async fn test_merkle_tree_operations() {
    let leaves = vec![
        Hash::from_hex("0000000000000000000000000000000000000000000000000000000000000001").unwrap(),
        Hash::from_hex("0000000000000000000000000000000000000000000000000000000000000002").unwrap(),
        Hash::from_hex("0000000000000000000000000000000000000000000000000000000000000003").unwrap(),
        Hash::from_hex("0000000000000000000000000000000000000000000000000000000000000004").unwrap(),
    ];
    
    // Build merkle tree
    let tree = MerkleTree::new(leaves.clone());
    assert_eq!(tree.height, 2); // log2(4) = 2
    
    // Generate inclusion proof for second leaf
    let proof = tree.generate_proof(1).unwrap();
    assert!(!proof.is_empty());
    
    // Verify proof
    assert!(tree.verify_proof(leaves[1], &proof, 1));
    
    // Invalid proof should fail
    assert!(!tree.verify_proof(leaves[0], &proof, 1)); // Wrong leaf
}

#[tokio::test]
async fn test_threshold_secret_sharing() {
    let secret = b"super_secret_key_32_bytes_long!!";
    let threshold = 3;
    let total_parties = 5;
    
    // Split secret into shares
    let shares = SecretShare::split_secret(secret, threshold, total_parties).unwrap();
    assert_eq!(shares.len(), total_parties as usize);
    
    // Verify each share has correct parameters
    for share in &shares {
        assert_eq!(share.threshold, threshold);
        assert_eq!(share.total_parties, total_parties);
        assert_eq!(share.share_data.len(), 32);
    }
    
    // Reconstruct with minimum shares
    let min_shares = &shares[..threshold as usize];
    let reconstructed = SecretShare::reconstruct_secret(min_shares).unwrap();
    assert_eq!(reconstructed.len(), 32);
    
    // Too few shares should fail
    let insufficient_shares = &shares[..2];
    assert!(SecretShare::reconstruct_secret(insufficient_shares).is_err());
}

#[tokio::test]
async fn test_ring_signature_anonymity() {
    let message = b"anonymous transaction message";
    let private_key = b"signer_private_key_32_bytes_long";
    
    // Create ring with decoy addresses
    let ring = vec![
        Address::from_hex("1111111111111111111111111111111111111111").unwrap(),
        Address::from_hex("2222222222222222222222222222222222222222").unwrap(), // Real signer position
        Address::from_hex("3333333333333333333333333333333333333333").unwrap(),
        Address::from_hex("4444444444444444444444444444444444444444").unwrap(),
    ];
    
    let signer_index = 1; // Position of real signer in ring
    
    // Create ring signature
    let ring_sig = RingSignature::create(message, signer_index, ring.clone(), private_key).unwrap();
    
    // Verify signature
    assert!(ring_sig.verify(message).unwrap());
    assert_eq!(ring_sig.ring_size(), 4);
    assert_eq!(ring_sig.ring, ring);
    
    // Wrong message should fail verification
    assert!(!ring_sig.verify(b"different message").unwrap());
}

#[tokio::test]
async fn test_confidential_transaction_balance() {
    // Create confidential inputs and outputs
    let input_commitment = vec![0u8; 32];
    let input_key_image = vec![1u8; 32];
    let input = ConfidentialInput::new(input_commitment, input_key_image);
    
    let view_key = b"view_key_32_bytes_long_for_encrypt";
    let spend_key = b"spend_key_32_bytes_long_for_spend";
    let tx_private = b"tx_private_key_32_bytes_long_key";
    
    let stealth = StealthAddress::generate(view_key, spend_key, tx_private).unwrap();
    let amount = Amount::from_paradigm(100.0);
    let output = ConfidentialOutput::new(amount, stealth, view_key).unwrap();
    
    // Build confidential transaction
    let mut tx = ConfidentialTransaction::new();
    tx.add_input(input);
    tx.add_output(output);
    tx.set_fee(Amount::from_paradigm(1.0));
    
    // Generate proofs
    tx.generate_range_proofs().unwrap();
    tx.generate_balance_proof().unwrap();
    
    // Verify transaction
    assert!(tx.verify().unwrap());
}

#[tokio::test]
async fn test_anomaly_detection() {
    let mut detector = AnomalyDetector::new();
    let from_addr = Address::from_hex("1111111111111111111111111111111111111111").unwrap();
    let to_addr = Address::from_hex("2222222222222222222222222222222222222222").unwrap();
    
    // Train detector with normal transactions
    for i in 1..=10 {
        let normal_tx = Transaction {
            hash: Hash::default(),
            from: from_addr,
            to: to_addr,
            amount: Amount::from_paradigm(100.0 + i as f64), // Normal range
            gas: 21000,
            gas_price: Amount::from_paradigm(0.00001),
            nonce: i,
            block_hash: None,
            block_number: None,
            transaction_index: None,
            input: vec![],
        };
        detector.update_pattern(from_addr, &normal_tx);
    }
    
    // Test with anomalous transaction
    let anomalous_tx = Transaction {
        hash: Hash::default(),
        from: from_addr,
        to: to_addr,
        amount: Amount::from_paradigm(10000.0), // 100x normal amount
        gas: 21000,
        gas_price: Amount::from_paradigm(0.00001),
        nonce: 11,
        block_hash: None,
        block_number: None,
        transaction_index: None,
        input: vec![],
    };
    
    let anomalies = detector.detect_anomalies(&anomalous_tx);
    assert!(!anomalies.is_empty());
    assert_eq!(anomalies[0].category, SecurityCategory::AbnormalBehavior);
}

#[tokio::test]
async fn test_enterprise_compliance_workflow() {
    let mut monitor = SecurityMonitor::new();
    
    // Configure multiple compliance rules
    let rules = vec![
        ComplianceRule {
            rule_id: Hash::default(),
            name: "AML Large Transaction".to_string(),
            description: "Monitor large transactions for AML compliance".to_string(),
            rule_type: ComplianceType::AML,
            parameters: [("amount_threshold".to_string(), "1000000000000000000".to_string())].into(),
            enabled: true,
        },
        ComplianceRule {
            rule_id: Hash::default(),
            name: "OFAC Sanctions Check".to_string(),
            description: "Check transactions against OFAC sanctions list".to_string(),
            rule_type: ComplianceType::OFAC,
            parameters: HashMap::new(),
            enabled: true,
        },
    ];
    
    for rule in rules {
        monitor.add_compliance_rule(rule);
    }
    
    // Add blacklisted address for OFAC test
    let blacklisted_addr = Address::from_hex("deadbeefdeadbeefdeadbeefdeadbeefdeadbeef").unwrap();
    monitor.threat_intelligence.add_blacklisted_address(blacklisted_addr);
    
    // Test transaction with blacklisted address
    let sanctioned_tx = Transaction {
        hash: Hash::default(),
        from: blacklisted_addr,
        to: Address::from_hex("1111111111111111111111111111111111111111").unwrap(),
        amount: Amount::from_paradigm(1.0),
        gas: 21000,
        gas_price: Amount::from_paradigm(0.00001),
        nonce: 1,
        block_hash: None,
        block_number: None,
        transaction_index: None,
        input: vec![],
    };
    
    let audits = monitor.monitor_transaction(&sanctioned_tx);
    
    // Should trigger both OFAC and threat intelligence alerts
    assert!(audits.len() >= 2);
    let has_ofac = audits.iter().any(|a| a.category == SecurityCategory::ComplianceViolation);
    let has_threat = audits.iter().any(|a| a.category == SecurityCategory::PotentialAttack);
    assert!(has_ofac || has_threat);
}

// Performance and stress tests

#[tokio::test] 
async fn test_high_volume_transaction_processing() {
    let mut monitor = SecurityMonitor::new();
    let start_time = std::time::Instant::now();
    
    // Process 1000 transactions
    for i in 0..1000 {
        let tx = Transaction {
            hash: Hash::default(),
            from: Address::from_hex("1111111111111111111111111111111111111111").unwrap(),
            to: Address::from_hex("2222222222222222222222222222222222222222").unwrap(),
            amount: Amount::from_paradigm(1.0 + (i % 100) as f64),
            gas: 21000,
            gas_price: Amount::from_paradigm(0.00001),
            nonce: i + 1,
            block_hash: None,
            block_number: None,
            transaction_index: None,
            input: vec![],
        };
        
        monitor.monitor_transaction(&tx);
    }
    
    let elapsed = start_time.elapsed();
    println!("Processed 1000 transactions in {:?} ({:.2} tx/s)", elapsed, 1000.0 / elapsed.as_secs_f64());
    
    // Should complete within reasonable time (less than 1 second for 1000 tx)
    assert!(elapsed < Duration::from_secs(1));
}

#[tokio::test]
async fn test_concurrent_wallet_operations() {
    use std::sync::Arc;
    use tokio::sync::Mutex;
    
    let wallet = Arc::new(Mutex::new(Wallet::create_random().unwrap()));
    
    // Spawn multiple concurrent signing operations
    let mut handles = vec![];
    
    for i in 0..10 {
        let wallet = Arc::clone(&wallet);
        let handle = tokio::spawn(async move {
            let tx = Transaction {
                hash: Hash::default(),
                from: Address::from_hex("1111111111111111111111111111111111111111").unwrap(),
                to: Address::from_hex("2222222222222222222222222222222222222222").unwrap(),
                amount: Amount::from_paradigm(1.0),
                gas: 21000,
                gas_price: Amount::from_paradigm(0.00001),
                nonce: i + 1,
                block_hash: None,
                block_number: None,
                transaction_index: None,
                input: vec![],
            };
            
            let wallet = wallet.lock().await;
            wallet.sign_transaction(&tx)
        });
        handles.push(handle);
    }
    
    // Wait for all signing operations to complete
    for handle in handles {
        let result = handle.await.unwrap();
        assert!(result.is_ok());
    }
}

// Edge case and error handling tests

#[tokio::test]
async fn test_invalid_transaction_handling() {
    let wallet = Wallet::create_random().unwrap();
    
    // Test transaction with zero amount
    let zero_tx = Transaction {
        hash: Hash::default(),
        from: wallet.address(),
        to: Address::from_hex("2222222222222222222222222222222222222222").unwrap(),
        amount: Amount::zero(),
        gas: 21000,
        gas_price: Amount::from_paradigm(0.00001),
        nonce: 1,
        block_hash: None,
        block_number: None,
        transaction_index: None,
        input: vec![],
    };
    
    // Should still be able to sign zero-amount transactions (for contract calls)
    let signature = wallet.sign_transaction(&zero_tx).unwrap();
    assert!(wallet.verify_signature(&zero_tx, &signature).unwrap());
}

#[tokio::test]
async fn test_malformed_address_handling() {
    // Test various malformed addresses
    let invalid_addresses = vec![
        "", 
        "0x", 
        "0x123", 
        "0x123g567890123456789012345678901234567890", // Invalid hex
        "123456789012345678901234567890123456789012", // No 0x prefix  
        "0x12345678901234567890123456789012345678900123", // Too long
    ];
    
    for invalid_addr in invalid_addresses {
        assert!(Address::from_hex(invalid_addr).is_err());
    }
}

#[tokio::test]
async fn test_edge_case_multisig_scenarios() {
    // Test 1-of-1 multisig (should work)
    let owner = ed25519_dalek::SigningKey::generate(&mut rand::thread_rng());
    let single_owner_wallet = MultiSigWallet::new(vec![owner.verifying_key()], 1).unwrap();
    
    let message = b"single owner test";
    let signers = vec![(0, &owner)];
    let sig = single_owner_wallet.create_transaction_signature(message, &signers).unwrap();
    assert!(single_owner_wallet.verify_transaction(message, &sig).unwrap());
    
    // Test edge case: threshold equals number of owners
    let owner1 = ed25519_dalek::SigningKey::generate(&mut rand::thread_rng());
    let owner2 = ed25519_dalek::SigningKey::generate(&mut rand::thread_rng());
    let all_required = MultiSigWallet::new(
        vec![owner1.verifying_key(), owner2.verifying_key()], 
        2
    ).unwrap();
    
    let message = b"all required test";
    let all_signers = vec![(0, &owner1), (1, &owner2)];
    let all_sig = all_required.create_transaction_signature(message, &all_signers).unwrap();
    assert!(all_required.verify_transaction(message, &all_sig).unwrap());
}

#[tokio::test]
async fn test_security_audit_lifecycle() {
    let mut audit = SecurityAudit::new(
        SecuritySeverity::High,
        SecurityCategory::SuspiciousTransaction,
        "Test security audit".to_string(),
        vec![Address::from_hex("1111111111111111111111111111111111111111").unwrap()],
    );
    
    // Test audit properties
    assert_eq!(audit.severity, SecuritySeverity::High);
    assert!(audit.age_seconds() < 5); // Should be very recent
    assert!(!audit.is_expired(Duration::from_secs(3600))); // Should not be expired
    
    // Add recommendations
    audit.add_recommendation("Block suspicious address".to_string());
    audit.add_recommendation("Notify compliance team".to_string());
    assert_eq!(audit.recommended_actions.len(), 2);
    
    // Test audit serialization
    let serialized = serde_json::to_string(&audit).unwrap();
    let deserialized: SecurityAudit = serde_json::from_str(&serialized).unwrap();
    assert_eq!(audit.audit_id, deserialized.audit_id);
}