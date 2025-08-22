use paradigm_sdk::prelude::*;
use std::collections::HashMap;

/// Security-focused tests to verify protection against common attacks
/// and ensure cryptographic security properties are maintained

#[test]
fn test_signature_malleability_protection() {
    let wallet = Wallet::create_random().unwrap();
    let transaction = Transaction {
        hash: Hash::default(),
        from: wallet.address(),
        to: Address::from_hex("2222222222222222222222222222222222222222").unwrap(),
        amount: Amount::from_paradigm(1.0),
        gas: 21000,
        gas_price: Amount::from_paradigm(0.00001),
        nonce: 1,
        block_hash: None,
        block_number: None,
        transaction_index: None,
        input: vec![],
    };
    
    let signature = wallet.sign_transaction(&transaction).unwrap();
    
    // Original signature should verify
    assert!(wallet.verify_signature(&transaction, &signature).unwrap());
    
    // Malleable signature (flipped s value) should not verify
    let mut malleable_sig = signature.clone();
    if let Some(last_byte) = malleable_sig.s.last_mut() {
        *last_byte = last_byte.wrapping_add(1);
    }
    
    // Malleable signature should be rejected
    assert!(!wallet.verify_signature(&transaction, &malleable_sig).unwrap());
}

#[test]
fn test_replay_attack_protection() {
    let wallet = Wallet::create_random().unwrap();
    let recipient = Address::from_hex("2222222222222222222222222222222222222222").unwrap();
    
    // Create transaction with specific nonce
    let transaction = Transaction {
        hash: Hash::default(),
        from: wallet.address(),
        to: recipient,
        amount: Amount::from_paradigm(1.0),
        gas: 21000,
        gas_price: Amount::from_paradigm(0.00001),
        nonce: 1,
        block_hash: None,
        block_number: None,
        transaction_index: None,
        input: vec![],
    };
    
    let signature = wallet.sign_transaction(&transaction).unwrap();
    
    // Same transaction should not be replayable with different nonce
    let replayed_transaction = Transaction {
        nonce: 2, // Different nonce
        ..transaction.clone()
    };
    
    // Signature should not verify for replayed transaction
    assert!(!wallet.verify_signature(&replayed_transaction, &signature).unwrap());
}

#[test]
fn test_double_spending_protection_ring_signatures() {
    let message = b"transaction to prevent double spending";
    let private_key = b"signer_private_key_for_double_test";
    
    let ring = vec![
        Address::from_hex("1111111111111111111111111111111111111111").unwrap(),
        Address::from_hex("2222222222222222222222222222222222222222").unwrap(),
        Address::from_hex("3333333333333333333333333333333333333333").unwrap(),
    ];
    
    // Create first ring signature
    let ring_sig1 = RingSignature::create(message, 1, ring.clone(), private_key).unwrap();
    
    // Create second ring signature with same private key
    let ring_sig2 = RingSignature::create(message, 1, ring, private_key).unwrap();
    
    // Key images should be the same (preventing double spending)
    assert_eq!(ring_sig1.key_image, ring_sig2.key_image);
    
    // Simulate spent key image tracking
    let mut spent_images = HashMap::new();
    spent_images.insert(ring_sig1.key_image.clone(), Hash::default());
    
    // Second signature should be detected as double spend
    assert!(ring_sig2.is_key_image_spent(&spent_images));
}

#[test]
fn test_private_key_exposure_protection() {
    let wallet = Wallet::create_random().unwrap();
    
    // Signing should not expose private key
    let transaction = Transaction {
        hash: Hash::default(),
        from: wallet.address(),
        to: Address::from_hex("2222222222222222222222222222222222222222").unwrap(),
        amount: Amount::from_paradigm(1.0),
        gas: 21000,
        gas_price: Amount::from_paradigm(0.00001),
        nonce: 1,
        block_hash: None,
        block_number: None,
        transaction_index: None,
        input: vec![],
    };
    
    let signature = wallet.sign_transaction(&transaction).unwrap();
    
    // Signature should not contain the private key directly
    let private_key_bytes = wallet.private_key_bytes().unwrap();
    assert!(!signature.r.contains(&private_key_bytes[0]));
    assert!(!signature.s.contains(&private_key_bytes[0]));
    
    // Multiple signatures should not reveal private key through correlation
    let mut signatures = Vec::new();
    for i in 1..=10 {
        let tx = Transaction {
            nonce: i,
            ..transaction.clone()
        };
        signatures.push(wallet.sign_transaction(&tx).unwrap());
    }
    
    // All signatures should be different
    for (i, sig1) in signatures.iter().enumerate() {
        for (j, sig2) in signatures.iter().enumerate() {
            if i != j {
                assert_ne!(sig1.r, sig2.r);
                assert_ne!(sig1.s, sig2.s);
            }
        }
    }
}

#[test]
fn test_timing_attack_resistance() {
    let wallet1 = Wallet::create_random().unwrap();
    let wallet2 = Wallet::create_random().unwrap();
    
    let transaction = Transaction {
        hash: Hash::default(),
        from: wallet1.address(),
        to: Address::from_hex("2222222222222222222222222222222222222222").unwrap(),
        amount: Amount::from_paradigm(1.0),
        gas: 21000,
        gas_price: Amount::from_paradigm(0.00001),
        nonce: 1,
        block_hash: None,
        block_number: None,
        transaction_index: None,
        input: vec![],
    };
    
    let valid_signature = wallet1.sign_transaction(&transaction).unwrap();
    let invalid_signature = wallet2.sign_transaction(&transaction).unwrap();
    
    // Time verification of valid signature
    let start = std::time::Instant::now();
    let _result1 = wallet1.verify_signature(&transaction, &valid_signature);
    let valid_duration = start.elapsed();
    
    // Time verification of invalid signature
    let start = std::time::Instant::now();
    let _result2 = wallet1.verify_signature(&transaction, &invalid_signature);
    let invalid_duration = start.elapsed();
    
    // Timing should be similar to prevent timing attacks
    let timing_ratio = valid_duration.as_nanos() as f64 / invalid_duration.as_nanos() as f64;
    assert!(timing_ratio > 0.5 && timing_ratio < 2.0, "Timing difference too large: {:?} vs {:?}", valid_duration, invalid_duration);
}

#[test]
fn test_multisig_security_properties() {
    let owner1 = ed25519_dalek::SigningKey::generate(&mut rand::thread_rng());
    let owner2 = ed25519_dalek::SigningKey::generate(&mut rand::thread_rng());
    let owner3 = ed25519_dalek::SigningKey::generate(&mut rand::thread_rng());
    let attacker = ed25519_dalek::SigningKey::generate(&mut rand::thread_rng());
    
    let public_keys = vec![
        owner1.verifying_key(),
        owner2.verifying_key(),
        owner3.verifying_key(),
    ];
    
    let wallet = MultiSigWallet::new(public_keys, 2).unwrap();
    let message = b"multisig transaction message";
    
    // Valid 2-of-3 signature should work
    let valid_signers = vec![(0, &owner1), (1, &owner2)];
    let valid_sig = wallet.create_transaction_signature(message, &valid_signers).unwrap();
    assert!(wallet.verify_transaction(message, &valid_sig).unwrap());
    
    // Attacker cannot forge signature
    let attack_signers = vec![(0, &attacker), (1, &owner2)];
    if let Ok(attack_sig) = wallet.create_transaction_signature(message, &attack_signers) {
        // Should not verify because attacker doesn't have valid key for position 0
        assert!(!wallet.verify_transaction(message, &attack_sig).unwrap());
    }
    
    // Single signature should not meet threshold
    let single_signer = vec![(0, &owner1)];
    let single_sig = wallet.create_transaction_signature(message, &single_signer).unwrap();
    assert!(!single_sig.is_complete());
}

#[test]
fn test_zero_knowledge_proof_security() {
    let circuit_hash = Hash::from_hex("deadbeef000000000000000000000000000000000000000000000000deadbeef").unwrap();
    let secret_witness = b"secret_witness_that_should_stay_hidden";
    let public_input = b"public_verification_data".to_vec();
    
    // Valid proof should verify
    let proof = ZKProof::new(circuit_hash, secret_witness, public_input.clone()).unwrap();
    assert!(proof.verify(&circuit_hash).unwrap());
    
    // Proof should not reveal the secret witness
    assert!(!proof.proof_data.contains(&secret_witness[0]));
    assert!(!proof.verification_key.contains(&secret_witness[0]));
    
    // Proof with wrong circuit should fail
    let wrong_circuit = Hash::from_hex("beefdead000000000000000000000000000000000000000000000000beefdead").unwrap();
    assert!(!proof.verify(&wrong_circuit).unwrap());
    
    // Proof should be deterministic for same inputs
    let proof2 = ZKProof::new(circuit_hash, secret_witness, public_input).unwrap();
    assert_eq!(proof.circuit_hash, proof2.circuit_hash);
    assert_eq!(proof.public_inputs, proof2.public_inputs);
}

#[test]
fn test_stealth_address_unlinkability() {
    let recipient_view_key = b"recipient_view_key_32_bytes_long";
    let recipient_spend_key = b"recipient_spend_key_32_bytes_lon";
    
    // Generate multiple stealth addresses for same recipient
    let mut stealth_addresses = Vec::new();
    for i in 0..10 {
        let tx_private_key = format!("tx_private_key_{}_{:016x}", i, rand::random::<u64>());
        let tx_key_bytes = tx_private_key.as_bytes();
        let padded_key = if tx_key_bytes.len() >= 32 {
            &tx_key_bytes[..32]
        } else {
            // Pad with zeros if needed
            let mut padded = [0u8; 32];
            padded[..tx_key_bytes.len()].copy_from_slice(tx_key_bytes);
            &padded[..]
        };
        
        if let Ok(stealth) = StealthAddress::generate(recipient_view_key, recipient_spend_key, padded_key) {
            stealth_addresses.push(stealth);
        }
    }
    
    // All stealth addresses should be different (unlinkable)
    for (i, addr1) in stealth_addresses.iter().enumerate() {
        for (j, addr2) in stealth_addresses.iter().enumerate() {
            if i != j {
                assert_ne!(addr1.stealth_address, addr2.stealth_address);
                assert_ne!(addr1.tx_public_key, addr2.tx_public_key);
            }
        }
    }
    
    // All should belong to the same recipient
    for stealth in &stealth_addresses {
        assert!(stealth.is_ours(recipient_view_key, recipient_spend_key).unwrap());
    }
}

#[test]
fn test_range_proof_soundness() {
    let blinding = b"blinding_factor_32_bytes_long!!!";
    
    // Valid range proof should verify
    let valid_proof = RangeProof::create(500, 0, 1000, blinding).unwrap();
    assert!(valid_proof.verify().unwrap());
    
    // Values outside range should fail to create proof
    assert!(RangeProof::create(1001, 0, 1000, blinding).is_err()); // Above max
    
    // Edge cases should work
    let min_proof = RangeProof::create(0, 0, 1000, blinding).unwrap();
    assert!(min_proof.verify().unwrap());
    
    let max_proof = RangeProof::create(1000, 0, 1000, blinding).unwrap();
    assert!(max_proof.verify().unwrap());
    
    // Invalid range (min > max) should fail
    assert!(RangeProof::create(500, 1000, 0, blinding).is_err());
}

#[test] 
fn test_confidential_transaction_balance_proof() {
    // Create confidential transaction
    let input_commitment = vec![1u8; 32];
    let input_key_image = vec![2u8; 32];
    let input = ConfidentialInput::new(input_commitment, input_key_image);
    
    let view_key = b"view_key_32_bytes_long_for_testing";
    let spend_key = b"spend_key_32_bytes_long_for_test";
    let tx_private = b"tx_private_key_32_bytes_for_test";
    
    let stealth = StealthAddress::generate(view_key, spend_key, tx_private).unwrap();
    let amount = Amount::from_paradigm(100.0);
    let output = ConfidentialOutput::new(amount, stealth, view_key).unwrap();
    
    let mut tx = ConfidentialTransaction::new();
    tx.add_input(input);
    tx.add_output(output);
    tx.set_fee(Amount::from_paradigm(1.0));
    
    // Generate proofs
    tx.generate_range_proofs().unwrap();
    tx.generate_balance_proof().unwrap();
    
    // Valid transaction should verify
    assert!(tx.verify().unwrap());
    
    // Tampered balance proof should fail
    let mut tampered_tx = tx.clone();
    tampered_tx.balance_proof[0] = tampered_tx.balance_proof[0].wrapping_add(1);
    assert!(!tampered_tx.verify().unwrap());
}

#[test]
fn test_threshold_signature_robustness() {
    let message_hash = Hash::from_hex("deadbeef000000000000000000000000000000000000000000000000deadbeef").unwrap();
    let participants = vec![1, 2, 3, 4, 5];
    let threshold = 3;
    
    let mut threshold_sig = ThresholdSignature::new(threshold, participants.clone(), message_hash);
    
    // Add valid partial signatures
    for &party_id in &participants[..threshold as usize] {
        let partial_sig = vec![party_id as u8; 64];
        threshold_sig.add_partial_signature(party_id, partial_sig).unwrap();
    }
    
    // Should be complete and combinable
    assert!(threshold_sig.is_complete());
    assert!(threshold_sig.combine().is_ok());
    
    // Duplicate signature should be rejected
    assert!(threshold_sig.add_partial_signature(1, vec![1u8; 64]).is_err());
    
    // Non-participant signature should be rejected  
    assert!(threshold_sig.add_partial_signature(99, vec![99u8; 64]).is_err());
}

#[test]
fn test_secret_sharing_security_properties() {
    let secret = b"ultra_secret_data_32_bytes_long!";
    let threshold = 3;
    let total_parties = 5;
    
    let shares = SecretShare::split_secret(secret, threshold, total_parties).unwrap();
    
    // Individual shares should not reveal the secret
    for share in &shares {
        assert!(!share.share_data.contains(&secret[0]));
        assert!(!share.share_data.contains(&secret[10]));
        assert!(!share.share_data.contains(&secret[20]));
    }
    
    // Insufficient shares should not allow reconstruction
    let insufficient_shares = &shares[..threshold as usize - 1];
    assert!(SecretShare::reconstruct_secret(insufficient_shares).is_err());
    
    // Corrupted share should fail verification
    let mut corrupted_share = shares[0].clone();
    corrupted_share.share_data[0] = corrupted_share.share_data[0].wrapping_add(1);
    
    let mut mixed_shares = vec![corrupted_share];
    mixed_shares.extend_from_slice(&shares[1..threshold as usize]);
    
    // Mixed shares with corruption should still attempt reconstruction
    // but result should be different from original secret
    if let Ok(reconstructed) = SecretShare::reconstruct_secret(&mixed_shares) {
        assert_ne!(reconstructed, secret.to_vec());
    }
}

#[test]
fn test_anomaly_detection_false_positive_resistance() {
    let mut detector = AnomalyDetector::new();
    let from_addr = Address::from_hex("1111111111111111111111111111111111111111").unwrap();
    let to_addr = Address::from_hex("2222222222222222222222222222222222222222").unwrap();
    
    // Train with consistent pattern
    let base_amount = 1000u64;
    for i in 1..=50 {
        let amount = base_amount + (i % 10) * 10; // Small variance
        let tx = Transaction {
            hash: Hash::default(),
            from: from_addr,
            to: to_addr,
            amount: Amount::from_wei(amount),
            gas: 21000,
            gas_price: Amount::from_paradigm(0.00001),
            nonce: i,
            block_hash: None,
            block_number: None,
            transaction_index: None,
            input: vec![],
        };
        detector.update_pattern(from_addr, &tx);
    }
    
    // Normal variation should not trigger anomaly
    let normal_tx = Transaction {
        hash: Hash::default(),
        from: from_addr,
        to: to_addr,
        amount: Amount::from_wei(base_amount + 15), // Within normal range
        gas: 21000,
        gas_price: Amount::from_paradigm(0.00001),
        nonce: 51,
        block_hash: None,
        block_number: None,
        transaction_index: None,
        input: vec![],
    };
    
    let anomalies = detector.detect_anomalies(&normal_tx);
    assert!(anomalies.is_empty() || anomalies.iter().all(|a| a.severity != SecuritySeverity::High));
}

#[test]
fn test_compliance_bypass_prevention() {
    let mut monitor = SecurityMonitor::new();
    
    // Add strict AML rule
    let aml_rule = ComplianceRule {
        rule_id: Hash::default(),
        name: "Strict AML Rule".to_string(),
        description: "Low threshold for testing".to_string(),
        rule_type: ComplianceType::AML,
        parameters: [("amount_threshold".to_string(), "1000000000000000000".to_string())].into(), // 1 ETH
        enabled: true,
    };
    monitor.add_compliance_rule(aml_rule);
    
    // Transaction just above threshold should be flagged
    let flagged_tx = Transaction {
        hash: Hash::default(),
        from: Address::from_hex("1111111111111111111111111111111111111111").unwrap(),
        to: Address::from_hex("2222222222222222222222222222222222222222").unwrap(),
        amount: Amount::from_wei(1000000000000000001), // 1 ETH + 1 wei
        gas: 21000,
        gas_price: Amount::from_paradigm(0.00001),
        nonce: 1,
        block_hash: None,
        block_number: None,
        transaction_index: None,
        input: vec![],
    };
    
    let audits = monitor.monitor_transaction(&flagged_tx);
    assert!(!audits.is_empty());
    let has_aml_violation = audits.iter().any(|a| a.category == SecurityCategory::ComplianceViolation);
    assert!(has_aml_violation);
    
    // Transaction just below threshold should not be flagged
    let unflagged_tx = Transaction {
        amount: Amount::from_wei(999999999999999999), // 1 ETH - 1 wei
        ..flagged_tx.clone()
    };
    
    let no_audits = monitor.monitor_transaction(&unflagged_tx);
    let has_aml_violation = no_audits.iter().any(|a| a.category == SecurityCategory::ComplianceViolation);
    assert!(!has_aml_violation);
}

#[test]
fn test_side_channel_resistance() {
    // Test that cryptographic operations don't leak information through side channels
    let message = b"side channel test message";
    
    // Test with different message lengths
    let messages = vec![
        vec![0u8; 16],
        vec![1u8; 32], 
        vec![2u8; 64],
        vec![3u8; 128],
        vec![4u8; 256],
    ];
    
    let signing_key = ed25519_dalek::SigningKey::generate(&mut rand::thread_rng());
    let mut durations = Vec::new();
    
    for msg in &messages {
        let start = std::time::Instant::now();
        let _signature = signing_key.sign(msg);
        durations.push(start.elapsed());
    }
    
    // Signing times should be relatively consistent regardless of message length
    let avg_duration = durations.iter().sum::<std::time::Duration>() / durations.len() as u32;
    let max_deviation = durations.iter()
        .map(|d| if d > &avg_duration { *d - avg_duration } else { avg_duration - *d })
        .max()
        .unwrap();
    
    // Deviation should be less than 50% of average (allowing for some variance)
    let deviation_ratio = max_deviation.as_nanos() as f64 / avg_duration.as_nanos() as f64;
    assert!(deviation_ratio < 0.5, "Timing variance too high: {:.2}%", deviation_ratio * 100.0);
}

#[test]
fn test_memory_safety_cryptographic_operations() {
    // Test that cryptographic operations don't cause memory corruption
    let mut wallets = Vec::new();
    
    // Create many wallets to test memory safety
    for _ in 0..1000 {
        let wallet = Wallet::create_random().unwrap();
        wallets.push(wallet);
    }
    
    // Perform operations on all wallets
    for (i, wallet) in wallets.iter().enumerate() {
        let tx = Transaction {
            hash: Hash::default(),
            from: wallet.address(),
            to: Address::from_hex("2222222222222222222222222222222222222222").unwrap(),
            amount: Amount::from_paradigm(1.0),
            gas: 21000,
            gas_price: Amount::from_paradigm(0.00001),
            nonce: i as u64 + 1,
            block_hash: None,
            block_number: None,
            transaction_index: None,
            input: vec![],
        };
        
        let signature = wallet.sign_transaction(&tx).unwrap();
        assert!(wallet.verify_signature(&tx, &signature).unwrap());
    }
    
    // All wallets should still be valid
    assert_eq!(wallets.len(), 1000);
}

#[test]
fn test_entropy_quality() {
    // Test that random generation produces good entropy
    let mut addresses = std::collections::HashSet::new();
    let mut hashes = std::collections::HashSet::new();
    
    // Generate many random values
    for _ in 0..1000 {
        let wallet = Wallet::create_random().unwrap();
        addresses.insert(wallet.address());
        
        let random_hash = Hash::from_bytes(&rand::random::<[u8; 32]>());
        hashes.insert(random_hash);
    }
    
    // Should have high uniqueness (no duplicates expected)
    assert_eq!(addresses.len(), 1000);
    assert_eq!(hashes.len(), 1000);
    
    // Test byte distribution in random data
    let random_data = rand::random::<[u8; 10000]>();
    let mut byte_counts = [0usize; 256];
    
    for &byte in &random_data {
        byte_counts[byte as usize] += 1;
    }
    
    // Each byte value should appear roughly equally (within reasonable variance)
    let expected_count = 10000 / 256; // ~39
    let max_deviation = byte_counts.iter()
        .map(|&count| if count > expected_count { count - expected_count } else { expected_count - count })
        .max()
        .unwrap();
    
    // Deviation should be reasonable for random data
    assert!(max_deviation < expected_count / 2, "Random distribution deviation too high: {}", max_deviation);
}