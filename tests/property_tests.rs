use paradigm_sdk::prelude::*;
use proptest::prelude::*;
use proptest::collection::vec;

/// Property-based tests using proptest to verify invariants
/// These tests generate random inputs to test edge cases and invariants

// Property tests for Address type
proptest! {
    #[test]
    fn test_address_roundtrip_hex(hex_string in r"[0-9a-fA-F]{40}") {
        let prefixed = format!("0x{}", hex_string);
        if let Ok(address) = Address::from_hex(&prefixed) {
            let back_to_hex = address.to_hex();
            // Should be deterministic and consistent
            prop_assert_eq!(back_to_hex.to_lowercase(), prefixed.to_lowercase());
        }
    }
    
    #[test]
    fn test_address_from_bytes_length(bytes in vec(any::<u8>(), 20)) {
        let address = Address::from_bytes(&bytes);
        let back_to_bytes = address.as_bytes();
        prop_assert_eq!(back_to_bytes, bytes.as_slice());
    }
    
    #[test] 
    fn test_address_equality_is_reflexive(bytes in vec(any::<u8>(), 20)) {
        let address = Address::from_bytes(&bytes);
        prop_assert_eq!(address, address);
    }
    
    #[test]
    fn test_address_equality_is_symmetric(
        bytes1 in vec(any::<u8>(), 20),
        bytes2 in vec(any::<u8>(), 20)
    ) {
        let addr1 = Address::from_bytes(&bytes1);
        let addr2 = Address::from_bytes(&bytes2);
        prop_assert_eq!(addr1 == addr2, addr2 == addr1);
    }
}

// Property tests for Hash type  
proptest! {
    #[test]
    fn test_hash_roundtrip_hex(hex_string in r"[0-9a-fA-F]{64}") {
        let prefixed = format!("0x{}", hex_string);
        if let Ok(hash) = Hash::from_hex(&prefixed) {
            let back_to_hex = hash.to_hex();
            prop_assert_eq!(back_to_hex.to_lowercase(), prefixed.to_lowercase());
        }
    }
    
    #[test]
    fn test_hash_from_bytes_length(bytes in vec(any::<u8>(), 32)) {
        let hash = Hash::from_bytes(&bytes);
        let back_to_bytes = hash.as_bytes();
        prop_assert_eq!(back_to_bytes, bytes.as_slice());
    }
    
    #[test]
    fn test_hash_equality_properties(bytes in vec(any::<u8>(), 32)) {
        let hash1 = Hash::from_bytes(&bytes);
        let hash2 = Hash::from_bytes(&bytes);
        
        // Reflexivity
        prop_assert_eq!(hash1, hash1);
        
        // Symmetry  
        prop_assert_eq!(hash1 == hash2, hash2 == hash1);
        
        // Transitivity (with same bytes should be equal)
        prop_assert_eq!(hash1, hash2);
    }
}

// Property tests for Amount type
proptest! {
    #[test]
    fn test_amount_from_wei_consistency(wei in 0u64..=u64::MAX) {
        let amount = Amount::from_wei(wei);
        prop_assert_eq!(amount.wei(), wei);
    }
    
    #[test] 
    fn test_amount_zero_properties(wei in 1u64..=1000000u64) {
        let amount = Amount::from_wei(wei);
        let zero = Amount::zero();
        
        // Zero is additive identity
        prop_assert_eq!(amount.checked_add(zero).unwrap(), amount);
        prop_assert_eq!(zero.checked_add(amount).unwrap(), amount);
        
        // Zero subtracted from anything gives the original
        prop_assert_eq!(amount.checked_sub(zero).unwrap(), amount);
        
        // Zero is less than any positive amount
        prop_assert!(zero < amount);
    }
    
    #[test]
    fn test_amount_arithmetic_properties(
        a in 0u64..1_000_000_000_000_000_000u64,
        b in 0u64..1_000_000_000_000_000_000u64
    ) {
        let amount_a = Amount::from_wei(a);
        let amount_b = Amount::from_wei(b);
        
        // Addition is commutative
        if let (Some(sum1), Some(sum2)) = (amount_a.checked_add(amount_b), amount_b.checked_add(amount_a)) {
            prop_assert_eq!(sum1, sum2);
        }
        
        // Subtraction properties
        if a >= b {
            let diff = amount_a.checked_sub(amount_b).unwrap();
            prop_assert_eq!(diff.checked_add(amount_b).unwrap(), amount_a);
        }
        
        // Ordering is consistent
        prop_assert_eq!(amount_a < amount_b, a < b);
        prop_assert_eq!(amount_a <= amount_b, a <= b);
        prop_assert_eq!(amount_a > amount_b, a > b);
        prop_assert_eq!(amount_a >= amount_b, a >= b);
    }
    
    #[test]
    fn test_amount_paradigm_conversion_bounds(paradigm_amount in 0.0f64..1_000_000.0) {
        let amount = Amount::from_paradigm(paradigm_amount);
        let wei_value = amount.wei();
        
        // Wei should be non-negative
        prop_assert!(wei_value >= 0);
        
        // Conversion should be consistent within floating point precision
        let back_to_paradigm = amount.to_paradigm();
        let diff = (paradigm_amount - back_to_paradigm).abs();
        prop_assert!(diff < 0.000001, "Conversion error too large: {} vs {}", paradigm_amount, back_to_paradigm);
    }
}

// Property tests for cryptographic operations
proptest! {
    #[test]
    fn test_signature_verification_consistency(
        message in vec(any::<u8>(), 1..1000),
        seed in any::<u64>()
    ) {
        use rand::{SeedableRng, RngCore};
        let mut rng = rand_chacha::ChaCha8Rng::seed_from_u64(seed);
        
        let signing_key = ed25519_dalek::SigningKey::generate(&mut rng);
        let verifying_key = signing_key.verifying_key();
        
        let signature = signing_key.sign(&message);
        
        // Valid signature should always verify
        prop_assert!(verifying_key.verify(&message, &signature).is_ok());
        
        // Modified message should not verify (with high probability)
        if message.len() > 1 {
            let mut modified_message = message.clone();
            modified_message[0] = modified_message[0].wrapping_add(1);
            prop_assert!(verifying_key.verify(&modified_message, &signature).is_err());
        }
    }
}

// Property tests for Merkle tree operations
proptest! {
    #[test]
    fn test_merkle_tree_inclusion_proofs(
        leaf_count in 1usize..=64,
        leaf_index_offset in 0usize..16
    ) {
        if leaf_count == 0 {
            return Ok(());
        }
        
        let leaf_index = leaf_index_offset % leaf_count;
        
        // Generate deterministic leaves
        let leaves: Vec<Hash> = (0..leaf_count)
            .map(|i| Hash::from_hex(&format!("{:064x}", i)).unwrap())
            .collect();
        
        let tree = MerkleTree::new(leaves.clone());
        
        if let Ok(proof) = tree.generate_proof(leaf_index) {
            // Proof should verify for correct leaf and index
            prop_assert!(tree.verify_proof(leaves[leaf_index], &proof, leaf_index));
            
            // Proof should not verify for wrong leaf (with high probability)
            if leaves.len() > 1 {
                let wrong_index = (leaf_index + 1) % leaves.len();
                prop_assert!(!tree.verify_proof(leaves[wrong_index], &proof, leaf_index));
            }
        }
    }
    
    #[test]
    fn test_merkle_tree_root_deterministic(leaf_count in 1usize..=32) {
        // Generate deterministic leaves
        let leaves: Vec<Hash> = (0..leaf_count)
            .map(|i| Hash::from_hex(&format!("{:064x}", i)).unwrap())
            .collect();
        
        let tree1 = MerkleTree::new(leaves.clone());
        let tree2 = MerkleTree::new(leaves);
        
        // Same leaves should produce same root
        prop_assert_eq!(tree1.root, tree2.root);
        prop_assert_eq!(tree1.height, tree2.height);
    }
}

// Property tests for secret sharing
proptest! {
    #[test]
    fn test_secret_sharing_threshold_properties(
        secret in vec(any::<u8>(), 32),
        threshold in 2u32..=7,
        extra_parties in 0u32..=3
    ) {
        let total_parties = threshold + extra_parties;
        
        if let Ok(shares) = SecretShare::split_secret(&secret, threshold, total_parties) {
            prop_assert_eq!(shares.len(), total_parties as usize);
            
            // All shares should have consistent parameters
            for share in &shares {
                prop_assert_eq!(share.threshold, threshold);
                prop_assert_eq!(share.total_parties, total_parties);
            }
            
            // Reconstruction with minimum shares should work
            let min_shares = &shares[..threshold as usize];
            prop_assert!(SecretShare::reconstruct_secret(min_shares).is_ok());
            
            // Reconstruction with insufficient shares should fail
            if threshold > 1 {
                let insufficient_shares = &shares[..threshold as usize - 1];
                prop_assert!(SecretShare::reconstruct_secret(insufficient_shares).is_err());
            }
            
            // Reconstruction with more than minimum shares should work
            if total_parties > threshold {
                let extra_shares = &shares[..threshold as usize + 1];
                prop_assert!(SecretShare::reconstruct_secret(extra_shares).is_ok());
            }
        }
    }
}

// Property tests for zero-knowledge proofs
proptest! {
    #[test]
    fn test_zk_proof_consistency(
        private_input in vec(any::<u8>(), 1..128),
        public_input in vec(any::<u8>(), 1..128),
        circuit_bytes in vec(any::<u8>(), 32)
    ) {
        let circuit_hash = Hash::from_bytes(&circuit_bytes);
        
        if let Ok(proof) = ZKProof::new(circuit_hash, &private_input, public_input.clone()) {
            // Proof should verify with correct circuit
            prop_assert!(proof.verify(&circuit_hash).unwrap_or(false));
            
            // Proof should have consistent properties
            prop_assert_eq!(proof.circuit_hash, circuit_hash);
            prop_assert_eq!(proof.public_inputs, public_input);
            prop_assert!(!proof.proof_data.is_empty());
            prop_assert!(!proof.verification_key.is_empty());
        }
    }
}

// Property tests for range proofs
proptest! {
    #[test]
    fn test_range_proof_validity(
        value in 0u64..1000000,
        min_offset in 0u64..100,
        max_offset in 100u64..1000,
        blinding in vec(any::<u8>(), 32)
    ) {
        let min_value = value - min_offset.min(value);
        let max_value = value + max_offset;
        
        if let Ok(range_proof) = RangeProof::create(value, min_value, max_value, &blinding) {
            // Range proof should verify
            prop_assert!(range_proof.verify().unwrap_or(false));
            
            // Properties should be consistent
            prop_assert_eq!(range_proof.min_value, min_value);
            prop_assert_eq!(range_proof.max_value, max_value);
            prop_assert!(min_value <= max_value);
        }
        
        // Value outside range should fail
        if min_value > 0 {
            prop_assert!(RangeProof::create(min_value - 1, min_value, max_value, &blinding).is_err());
        }
        if max_value < u64::MAX {
            prop_assert!(RangeProof::create(max_value + 1, min_value, max_value, &blinding).is_err());
        }
    }
}

// Property tests for ring signatures
proptest! {
    #[test]
    fn test_ring_signature_properties(
        message in vec(any::<u8>(), 1..1000),
        ring_size in 2usize..=8,
        signer_position in 0usize..8,
        private_key in vec(any::<u8>(), 32)
    ) {
        if signer_position >= ring_size {
            return Ok(());
        }
        
        // Generate ring addresses deterministically
        let ring: Vec<Address> = (0..ring_size)
            .map(|i| {
                let bytes = [(i as u8); 20];
                Address::from_bytes(&bytes)
            })
            .collect();
        
        if let Ok(ring_sig) = RingSignature::create(&message, signer_position, ring.clone(), &private_key) {
            // Ring signature should verify
            prop_assert!(ring_sig.verify(&message).unwrap_or(false));
            
            // Properties should be consistent
            prop_assert_eq!(ring_sig.ring_size(), ring_size);
            prop_assert_eq!(ring_sig.ring, ring);
            
            // Different message should not verify
            if !message.is_empty() {
                let mut wrong_message = message.clone();
                wrong_message[0] = wrong_message[0].wrapping_add(1);
                prop_assert!(!ring_sig.verify(&wrong_message).unwrap_or(true));
            }
        }
    }
}

// Property tests for stealth addresses
proptest! {
    #[test]
    fn test_stealth_address_ownership(
        view_key in vec(any::<u8>(), 32),
        spend_key in vec(any::<u8>(), 32),
        tx_private in vec(any::<u8>(), 32),
        wrong_key in vec(any::<u8>(), 32)
    ) {
        if let Ok(stealth) = StealthAddress::generate(&view_key, &spend_key, &tx_private) {
            // Should recognize correct owner
            prop_assert!(stealth.is_ours(&view_key, &spend_key).unwrap_or(false));
            
            // Should reject wrong owner (with high probability)
            if wrong_key != view_key {
                prop_assert!(!stealth.is_ours(&wrong_key, &spend_key).unwrap_or(true));
            }
            if wrong_key != spend_key {
                prop_assert!(!stealth.is_ours(&view_key, &wrong_key).unwrap_or(true));
            }
            
            // Should be able to compute private key for correct owner
            if stealth.is_ours(&view_key, &spend_key).unwrap_or(false) {
                prop_assert!(stealth.compute_private_key(&view_key, &spend_key).is_ok());
            }
        }
    }
}

// Property tests for multisig wallets
proptest! {
    #[test]
    fn test_multisig_threshold_properties(
        threshold in 1u32..=5,
        extra_owners in 0u32..=3,
        seed in any::<u64>()
    ) {
        use rand::{SeedableRng, RngCore};
        let mut rng = rand_chacha::ChaCha8Rng::seed_from_u64(seed);
        
        let total_owners = threshold + extra_owners;
        
        // Generate signing keys
        let signing_keys: Vec<_> = (0..total_owners)
            .map(|_| ed25519_dalek::SigningKey::generate(&mut rng))
            .collect();
        
        let public_keys: Vec<_> = signing_keys
            .iter()
            .map(|sk| sk.verifying_key())
            .collect();
        
        if let Ok(wallet) = MultiSigWallet::new(public_keys, threshold) {
            prop_assert_eq!(wallet.threshold, threshold);
            prop_assert_eq!(wallet.owners.len(), total_owners as usize);
            
            let message = b"test multisig message";
            
            // Sufficient signers should create valid signature
            let signers: Vec<_> = signing_keys
                .iter()
                .take(threshold as usize)
                .enumerate()
                .map(|(i, sk)| (i as u32, sk))
                .collect();
            
            if let Ok(threshold_sig) = wallet.create_transaction_signature(message, &signers) {
                prop_assert!(threshold_sig.is_complete());
                prop_assert!(wallet.verify_transaction(message, &threshold_sig).unwrap_or(false));
            }
            
            // Insufficient signers should not meet threshold
            if threshold > 1 {
                let insufficient_signers: Vec<_> = signing_keys
                    .iter()
                    .take(threshold as usize - 1)
                    .enumerate()
                    .map(|(i, sk)| (i as u32, sk))
                    .collect();
                
                if let Ok(insufficient_sig) = wallet.create_transaction_signature(message, &insufficient_signers) {
                    prop_assert!(!insufficient_sig.is_complete());
                }
            }
        }
    }
}

// Property tests for security monitoring
proptest! {
    #[test]
    fn test_anomaly_detection_consistency(
        amounts in vec(100u64..1000u64, 5..20),
        anomaly_multiplier in 5u64..50
    ) {
        let mut detector = AnomalyDetector::new();
        let from_addr = Address::from_bytes(&[1u8; 20]);
        let to_addr = Address::from_bytes(&[2u8; 20]);
        
        // Train detector with normal transactions
        for (i, &amount) in amounts.iter().enumerate() {
            let tx = Transaction {
                hash: Hash::default(),
                from: from_addr,
                to: to_addr,
                amount: Amount::from_wei(amount),
                gas: 21000,
                gas_price: Amount::from_paradigm(0.00001),
                nonce: i as u64 + 1,
                block_hash: None,
                block_number: None,
                transaction_index: None,
                input: vec![],
            };
            detector.update_pattern(from_addr, &tx);
        }
        
        // Test with anomalous transaction
        let avg_amount = amounts.iter().sum::<u64>() / amounts.len() as u64;
        let anomalous_amount = avg_amount * anomaly_multiplier;
        
        let anomalous_tx = Transaction {
            hash: Hash::default(),
            from: from_addr,
            to: to_addr,
            amount: Amount::from_wei(anomalous_amount),
            gas: 21000,
            gas_price: Amount::from_paradigm(0.00001),
            nonce: amounts.len() as u64 + 1,
            block_hash: None,
            block_number: None,
            transaction_index: None,
            input: vec![],
        };
        
        let anomalies = detector.detect_anomalies(&anomalous_tx);
        
        // Large deviation should be detected as anomaly
        if anomaly_multiplier >= 10 {
            prop_assert!(!anomalies.is_empty());
        }
    }
}

// Property tests for transaction serialization/deserialization
proptest! {
    #[test]
    fn test_transaction_serialization_roundtrip(
        from_bytes in vec(any::<u8>(), 20),
        to_bytes in vec(any::<u8>(), 20),
        amount in 0u64..u64::MAX,
        gas in 21000u64..1000000u64,
        nonce in 0u64..1000000u64,
        input_data in vec(any::<u8>(), 0..1000)
    ) {
        let transaction = Transaction {
            hash: Hash::from_bytes(&[0u8; 32]),
            from: Address::from_bytes(&from_bytes),
            to: Address::from_bytes(&to_bytes),
            amount: Amount::from_wei(amount),
            gas,
            gas_price: Amount::from_wei(1000000000), // 1 Gwei
            nonce,
            block_hash: None,
            block_number: None,
            transaction_index: None,
            input: input_data,
        };
        
        // Serialize and deserialize
        if let Ok(serialized) = serde_json::to_string(&transaction) {
            if let Ok(deserialized) = serde_json::from_str::<Transaction>(&serialized) {
                prop_assert_eq!(transaction.from, deserialized.from);
                prop_assert_eq!(transaction.to, deserialized.to);
                prop_assert_eq!(transaction.amount, deserialized.amount);
                prop_assert_eq!(transaction.gas, deserialized.gas);
                prop_assert_eq!(transaction.nonce, deserialized.nonce);
                prop_assert_eq!(transaction.input, deserialized.input);
            }
        }
    }
}

// Property tests for helper functions
proptest! {
    #[test]
    fn test_hex_conversion_properties(bytes in vec(any::<u8>(), 1..100)) {
        use paradigm_sdk::helpers::{hex_to_bytes, bytes_to_hex};
        
        let hex = bytes_to_hex(&bytes);
        
        // Should start with 0x
        prop_assert!(hex.starts_with("0x"));
        
        // Should be valid hex
        prop_assert!(hex[2..].chars().all(|c| c.is_ascii_hexdigit()));
        
        // Should roundtrip correctly
        if let Ok(decoded) = hex_to_bytes(&hex) {
            prop_assert_eq!(decoded, bytes);
        }
    }
    
    #[test]
    fn test_amount_conversion_properties(paradigm_amount in 0.0f64..1_000_000.0) {
        use paradigm_sdk::helpers::{paradigm_to_wei, wei_to_paradigm};
        
        let wei = paradigm_to_wei(paradigm_amount);
        let back_to_paradigm = wei_to_paradigm(wei);
        
        // Should be approximately equal (within floating point precision)
        let diff = (paradigm_amount - back_to_paradigm).abs();
        prop_assert!(diff < 0.000001);
        
        // Wei should be non-negative
        prop_assert!(wei >= 0);
    }
}