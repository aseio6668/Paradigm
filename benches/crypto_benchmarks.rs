use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId, Throughput};
use paradigm_sdk::prelude::*;
use ed25519_dalek::{SigningKey, Signer, Verifier};
use rand::{Rng, RngCore};
use std::time::Duration;

fn benchmark_signature_operations(c: &mut Criterion) {
    let mut group = c.benchmark_group("signature_operations");
    
    // Prepare test data
    let signing_key = SigningKey::generate(&mut rand::thread_rng());
    let verifying_key = signing_key.verifying_key();
    let message = b"benchmark message for signing performance test";
    let signature = signing_key.sign(message);
    
    // Benchmark signing
    group.bench_function("ed25519_sign", |b| {
        b.iter(|| {
            let key = SigningKey::generate(&mut rand::thread_rng());
            let msg = black_box(message);
            key.sign(msg)
        });
    });
    
    // Benchmark verification
    group.bench_function("ed25519_verify", |b| {
        b.iter(|| {
            let key = black_box(&verifying_key);
            let msg = black_box(message);
            let sig = black_box(&signature);
            key.verify(msg, sig)
        });
    });
    
    group.finish();
}

fn benchmark_hash_functions(c: &mut Criterion) {
    let mut group = c.benchmark_group("hash_functions");
    group.throughput(Throughput::Bytes(1024));
    
    let data = vec![0u8; 1024];
    
    // Benchmark SHA-256
    group.bench_function("sha256", |b| {
        use sha2::{Sha256, Digest};
        b.iter(|| {
            let data = black_box(&data);
            Sha256::digest(data)
        });
    });
    
    // Benchmark SHA-3
    group.bench_function("sha3_256", |b| {
        use sha3::{Sha3_256, Digest};
        b.iter(|| {
            let data = black_box(&data);
            Sha3_256::digest(data)
        });
    });
    
    // Benchmark BLAKE3
    group.bench_function("blake3", |b| {
        b.iter(|| {
            let data = black_box(&data);
            blake3::hash(data)
        });
    });
    
    group.finish();
}

fn benchmark_zero_knowledge_proofs(c: &mut Criterion) {
    let mut group = c.benchmark_group("zero_knowledge_proofs");
    
    let circuit_hash = Hash::from_hex("deadbeef000000000000000000000000000000000000000000000000deadbeef").unwrap();
    let private_inputs = b"secret_witness_data_for_benchmark_testing";
    let public_inputs = b"public_verification_data_for_benchmarks".to_vec();
    
    // Benchmark ZK proof generation
    group.bench_function("zk_proof_generation", |b| {
        b.iter(|| {
            let circuit = black_box(circuit_hash);
            let private = black_box(private_inputs);
            let public = black_box(public_inputs.clone());
            ZKProof::new(circuit, private, public)
        });
    });
    
    // Benchmark ZK proof verification
    let proof = ZKProof::new(circuit_hash, private_inputs, public_inputs).unwrap();
    group.bench_function("zk_proof_verification", |b| {
        b.iter(|| {
            let p = black_box(&proof);
            let circuit = black_box(&circuit_hash);
            p.verify(circuit)
        });
    });
    
    group.finish();
}

fn benchmark_threshold_signatures(c: &mut Criterion) {
    let mut group = c.benchmark_group("threshold_signatures");
    
    // Setup threshold signature test data
    let message_hash = Hash::from_hex("deadbeef000000000000000000000000000000000000000000000000deadbeef").unwrap();
    let participants = vec![1, 2, 3, 4, 5];
    let threshold = 3;
    
    // Benchmark threshold signature creation
    group.bench_function("threshold_sig_create", |b| {
        b.iter(|| {
            let hash = black_box(message_hash);
            let parts = black_box(participants.clone());
            let thresh = black_box(threshold);
            ThresholdSignature::new(thresh, parts, hash)
        });
    });
    
    // Setup for combination benchmark
    let mut threshold_sig = ThresholdSignature::new(threshold, participants.clone(), message_hash);
    for &party_id in &participants[..threshold as usize] {
        let partial_sig = vec![party_id as u8; 64]; // Mock partial signature
        threshold_sig.add_partial_signature(party_id, partial_sig).unwrap();
    }
    
    // Benchmark signature combination
    group.bench_function("threshold_sig_combine", |b| {
        b.iter(|| {
            let sig = black_box(&threshold_sig);
            sig.combine()
        });
    });
    
    group.finish();
}

fn benchmark_secret_sharing(c: &mut Criterion) {
    let mut group = c.benchmark_group("secret_sharing");
    
    let secret = b"super_secret_key_32_bytes_long_benchmark_test";
    let threshold = 5;
    let total_parties = 10;
    
    // Benchmark secret splitting
    group.bench_function("secret_split", |b| {
        b.iter(|| {
            let s = black_box(secret);
            let t = black_box(threshold);
            let n = black_box(total_parties);
            SecretShare::split_secret(s, t, n)
        });
    });
    
    // Setup shares for reconstruction benchmark
    let shares = SecretShare::split_secret(secret, threshold, total_parties).unwrap();
    let min_shares = &shares[..threshold as usize];
    
    // Benchmark secret reconstruction
    group.bench_function("secret_reconstruct", |b| {
        b.iter(|| {
            let s = black_box(min_shares);
            SecretShare::reconstruct_secret(s)
        });
    });
    
    group.finish();
}

fn benchmark_privacy_features(c: &mut Criterion) {
    let mut group = c.benchmark_group("privacy_features");
    
    // Ring signature benchmarks
    let message = b"privacy benchmark message";
    let private_key = b"signer_private_key_32_bytes_benchmark";
    let ring = vec![
        Address::from_hex("1111111111111111111111111111111111111111").unwrap(),
        Address::from_hex("2222222222222222222222222222222222222222").unwrap(),
        Address::from_hex("3333333333333333333333333333333333333333").unwrap(),
        Address::from_hex("4444444444444444444444444444444444444444").unwrap(),
    ];
    let signer_index = 1;
    
    // Benchmark ring signature creation
    group.bench_function("ring_signature_create", |b| {
        b.iter(|| {
            let msg = black_box(message);
            let key = black_box(private_key);
            let r = black_box(ring.clone());
            let idx = black_box(signer_index);
            RingSignature::create(msg, idx, r, key)
        });
    });
    
    // Setup for verification benchmark
    let ring_sig = RingSignature::create(message, signer_index, ring, private_key).unwrap();
    
    // Benchmark ring signature verification
    group.bench_function("ring_signature_verify", |b| {
        b.iter(|| {
            let sig = black_box(&ring_sig);
            let msg = black_box(message);
            sig.verify(msg)
        });
    });
    
    // Stealth address benchmarks
    let view_key = b"recipient_view_key_32_bytes_bench";
    let spend_key = b"recipient_spend_key_32_bytes_ben";
    let tx_private = b"tx_private_key_32_bytes_benchmark";
    
    // Benchmark stealth address generation
    group.bench_function("stealth_address_generate", |b| {
        b.iter(|| {
            let v = black_box(view_key);
            let s = black_box(spend_key);  
            let t = black_box(tx_private);
            StealthAddress::generate(v, s, t)
        });
    });
    
    group.finish();
}

fn benchmark_merkle_operations(c: &mut Criterion) {
    let mut group = c.benchmark_group("merkle_operations");
    
    // Test different tree sizes
    for size in [16, 64, 256, 1024].iter() {
        let leaves: Vec<Hash> = (0..*size)
            .map(|i| Hash::from_hex(&format!("{:064x}", i)).unwrap())
            .collect();
        
        group.bench_with_input(
            BenchmarkId::new("merkle_tree_construction", size),
            size,
            |b, _| {
                b.iter(|| {
                    let l = black_box(leaves.clone());
                    MerkleTree::new(l)
                });
            }
        );
    }
    
    // Benchmark proof generation and verification
    let leaves: Vec<Hash> = (0..256)
        .map(|i| Hash::from_hex(&format!("{:064x}", i)).unwrap())
        .collect();
    let tree = MerkleTree::new(leaves.clone());
    let proof = tree.generate_proof(128).unwrap();
    
    group.bench_function("merkle_proof_generation", |b| {
        b.iter(|| {
            let t = black_box(&tree);
            let idx = black_box(128);
            t.generate_proof(idx)
        });
    });
    
    group.bench_function("merkle_proof_verification", |b| {
        b.iter(|| {
            let t = black_box(&tree);
            let leaf = black_box(leaves[128]);
            let p = black_box(&proof);
            let idx = black_box(128);
            t.verify_proof(leaf, p, idx)
        });
    });
    
    group.finish();
}

fn benchmark_security_monitoring(c: &mut Criterion) {
    let mut group = c.benchmark_group("security_monitoring");
    
    let mut monitor = SecurityMonitor::new();
    
    // Add some compliance rules
    let aml_rule = ComplianceRule {
        rule_id: Hash::default(),
        name: "Benchmark AML Rule".to_string(),
        description: "AML rule for benchmarking".to_string(),
        rule_type: ComplianceType::AML,
        parameters: [("amount_threshold".to_string(), "1000000000000000000".to_string())].into(),
        enabled: true,
    };
    monitor.add_compliance_rule(aml_rule);
    
    // Create test transaction
    let test_tx = Transaction {
        hash: Hash::default(),
        from: Address::from_hex("1111111111111111111111111111111111111111").unwrap(),
        to: Address::from_hex("2222222222222222222222222222222222222222").unwrap(),
        amount: Amount::from_paradigm(1.5),
        gas: 21000,
        gas_price: Amount::from_paradigm(0.00001),
        nonce: 1,
        block_hash: None,
        block_number: None,
        transaction_index: None,
        input: vec![],
    };
    
    // Benchmark transaction monitoring
    group.bench_function("transaction_monitoring", |b| {
        b.iter(|| {
            let m = black_box(&mut monitor);
            let tx = black_box(&test_tx);
            m.monitor_transaction(tx)
        });
    });
    
    // Benchmark anomaly detection
    let mut detector = AnomalyDetector::new();
    
    group.bench_function("anomaly_detection", |b| {
        b.iter(|| {
            let d = black_box(&detector);
            let tx = black_box(&test_tx);
            d.detect_anomalies(tx)
        });
    });
    
    group.finish();
}

fn benchmark_encryption_decryption(c: &mut Criterion) {
    let mut group = c.benchmark_group("encryption");
    group.throughput(Throughput::Bytes(1024));
    
    let message = "This is a confidential message for encryption benchmarking. ".repeat(16); // ~1KB
    let key = b"encryption_key_32_bytes_benchmark";
    
    // Benchmark memo encryption
    group.bench_function("memo_encryption", |b| {
        b.iter(|| {
            let msg = black_box(&message);
            let k = black_box(key);
            EncryptedMemo::encrypt(msg, k)
        });
    });
    
    // Setup for decryption benchmark
    let encrypted = EncryptedMemo::encrypt(&message, key).unwrap();
    
    // Benchmark memo decryption
    group.bench_function("memo_decryption", |b| {
        b.iter(|| {
            let enc = black_box(&encrypted);
            let k = black_box(key);
            enc.decrypt(k)
        });
    });
    
    group.finish();
}

fn benchmark_address_operations(c: &mut Criterion) {
    let mut group = c.benchmark_group("address_operations");
    
    let hex_address = "0x1234567890123456789012345678901234567890";
    let address_bytes = [0x12, 0x34, 0x56, 0x78, 0x90, 0x12, 0x34, 0x56, 0x78, 0x90,
                        0x12, 0x34, 0x56, 0x78, 0x90, 0x12, 0x34, 0x56, 0x78, 0x90];
    
    // Benchmark address creation from hex
    group.bench_function("address_from_hex", |b| {
        b.iter(|| {
            let hex = black_box(hex_address);
            Address::from_hex(hex)
        });
    });
    
    // Benchmark address creation from bytes
    group.bench_function("address_from_bytes", |b| {
        b.iter(|| {
            let bytes = black_box(&address_bytes);
            Address::from_bytes(bytes)
        });
    });
    
    // Benchmark address to hex conversion
    let address = Address::from_hex(hex_address).unwrap();
    group.bench_function("address_to_hex", |b| {
        b.iter(|| {
            let addr = black_box(&address);
            addr.to_hex()
        });
    });
    
    // Benchmark address validation
    group.bench_function("address_validation", |b| {
        use paradigm_sdk::helpers::validate_address;
        b.iter(|| {
            let hex = black_box(hex_address);
            validate_address(hex)
        });
    });
    
    group.finish();
}

fn benchmark_amount_operations(c: &mut Criterion) {
    let mut group = c.benchmark_group("amount_operations");
    
    // Benchmark amount conversions
    group.bench_function("paradigm_to_wei", |b| {
        b.iter(|| {
            let amount = black_box(123.456789);
            Amount::from_paradigm(amount)
        });
    });
    
    group.bench_function("wei_to_paradigm", |b| {
        b.iter(|| {
            let wei = black_box(1234567890123456789u64);
            Amount::from_wei(wei)
        });
    });
    
    // Benchmark amount arithmetic
    let amount1 = Amount::from_paradigm(100.0);
    let amount2 = Amount::from_paradigm(50.0);
    
    group.bench_function("amount_addition", |b| {
        b.iter(|| {
            let a1 = black_box(amount1);
            let a2 = black_box(amount2);
            a1.checked_add(a2)
        });
    });
    
    group.bench_function("amount_subtraction", |b| {
        b.iter(|| {
            let a1 = black_box(amount1);
            let a2 = black_box(amount2);
            a1.checked_sub(a2)
        });
    });
    
    group.finish();
}

// Comprehensive benchmark combining multiple operations
fn benchmark_transaction_processing_pipeline(c: &mut Criterion) {
    let mut group = c.benchmark_group("transaction_pipeline");
    
    // Setup components
    let wallet = Wallet::create_random().unwrap();
    let mut monitor = SecurityMonitor::new();
    let aml_rule = ComplianceRule {
        rule_id: Hash::default(),
        name: "Pipeline AML Rule".to_string(),
        description: "AML rule for pipeline benchmarking".to_string(),
        rule_type: ComplianceType::AML,
        parameters: [("amount_threshold".to_string(), "1000000000000000000".to_string())].into(),
        enabled: true,
    };
    monitor.add_compliance_rule(aml_rule);
    
    // Benchmark complete transaction processing pipeline
    group.bench_function("full_transaction_pipeline", |b| {
        b.iter(|| {
            // Create transaction
            let tx = Transaction {
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
            
            // Sign transaction
            let signature = wallet.sign_transaction(&tx).unwrap();
            
            // Verify signature
            wallet.verify_signature(&tx, &signature).unwrap();
            
            // Monitor for security issues
            monitor.monitor_transaction(&tx);
            
            // Return transaction hash (to prevent optimization)
            tx.hash
        });
    });
    
    group.finish();
}

// Memory usage benchmarks
fn benchmark_memory_usage(c: &mut Criterion) {
    let mut group = c.benchmark_group("memory_usage");
    
    // Benchmark wallet creation memory usage
    group.bench_function("wallet_creation_memory", |b| {
        b.iter(|| {
            let wallets: Vec<_> = (0..100)
                .map(|_| Wallet::create_random().unwrap())
                .collect();
            black_box(wallets)
        });
    });
    
    // Benchmark large merkle tree memory usage
    group.bench_function("large_merkle_tree_memory", |b| {
        b.iter(|| {
            let leaves: Vec<Hash> = (0..10000)
                .map(|i| Hash::from_hex(&format!("{:064x}", i)).unwrap())
                .collect();
            let tree = MerkleTree::new(leaves);
            black_box(tree)
        });
    });
    
    group.finish();
}

criterion_group!(
    benches,
    benchmark_signature_operations,
    benchmark_hash_functions,
    benchmark_zero_knowledge_proofs,
    benchmark_threshold_signatures,
    benchmark_secret_sharing,
    benchmark_privacy_features,
    benchmark_merkle_operations,
    benchmark_security_monitoring,
    benchmark_encryption_decryption,
    benchmark_address_operations,
    benchmark_amount_operations,
    benchmark_transaction_processing_pipeline,
    benchmark_memory_usage
);

criterion_main!(benches);