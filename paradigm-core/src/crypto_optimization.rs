use anyhow::Result;
use blake3::Hasher as Blake3Hasher;
use dashmap::DashMap;
use ed25519_dalek::{Signature, Signer, SigningKey, Verifier, VerifyingKey};
use rand::{rngs::OsRng, RngCore};
use rayon::prelude::*;
use ring::aead::{Aad, LessSafeKey, Nonce, UnboundKey, AES_256_GCM, NONCE_LEN};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use sha3::Keccak256;
/// High-performance cryptographic operations for Paradigm
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Instant;
use tokio::sync::RwLock;

/// High-performance signature cache for avoiding redundant verifications
#[derive(Debug, Clone)]
pub struct SignatureCache {
    cache: DashMap<Vec<u8>, (bool, Instant)>,
    ttl_seconds: u64,
    max_entries: usize,
}

impl SignatureCache {
    pub fn new(ttl_seconds: u64, max_entries: usize) -> Self {
        Self {
            cache: DashMap::new(),
            ttl_seconds,
            max_entries,
        }
    }

    pub fn verify_cached(
        &self,
        message: &[u8],
        signature: &[u8],
        public_key: &[u8],
    ) -> Option<bool> {
        let cache_key = self.create_cache_key(message, signature, public_key);

        if let Some(entry) = self.cache.get(&cache_key) {
            if entry.1.elapsed().as_secs() < self.ttl_seconds {
                return Some(entry.0);
            } else {
                self.cache.remove(&cache_key);
            }
        }
        None
    }

    pub fn cache_result(&self, message: &[u8], signature: &[u8], public_key: &[u8], valid: bool) {
        if self.cache.len() >= self.max_entries {
            self.cleanup_expired();

            // If still at capacity, remove oldest entries
            if self.cache.len() >= self.max_entries {
                let mut to_remove = Vec::new();
                for entry in self.cache.iter().take(self.max_entries / 4) {
                    to_remove.push(entry.key().clone());
                }
                for key in to_remove {
                    self.cache.remove(&key);
                }
            }
        }

        let cache_key = self.create_cache_key(message, signature, public_key);
        self.cache.insert(cache_key, (valid, Instant::now()));
    }

    fn create_cache_key(&self, message: &[u8], signature: &[u8], public_key: &[u8]) -> Vec<u8> {
        let mut hasher = Blake3Hasher::new();
        hasher.update(message);
        hasher.update(signature);
        hasher.update(public_key);
        hasher.finalize().as_bytes().to_vec()
    }

    fn cleanup_expired(&self) {
        let now = Instant::now();
        self.cache
            .retain(|_, v| now.duration_since(v.1).as_secs() < self.ttl_seconds);
    }

    pub fn stats(&self) -> CacheStats {
        CacheStats {
            entries: self.cache.len(),
            max_entries: self.max_entries,
        }
    }
}

/// Hash computation pool for parallel processing
#[derive(Debug)]
pub struct HashPool {
    thread_pool: rayon::ThreadPool,
}

impl Clone for HashPool {
    fn clone(&self) -> Self {
        // Create a new thread pool with the same configuration
        Self::new(self.thread_pool.current_num_threads()).unwrap_or_else(|_| {
            Self::new(4).unwrap() // fallback to 4 threads
        })
    }
}

impl HashPool {
    pub fn new(num_threads: usize) -> Result<Self> {
        let thread_pool = rayon::ThreadPoolBuilder::new()
            .num_threads(num_threads)
            .build()?;

        Ok(Self { thread_pool })
    }

    /// Compute multiple hashes in parallel using different algorithms
    pub fn parallel_multi_hash(&self, data: &[u8]) -> ParallelHashResult {
        let data_arc = Arc::new(data.to_vec());

        let (sha256_tx, sha256_rx) = std::sync::mpsc::channel();
        let (sha3_tx, sha3_rx) = std::sync::mpsc::channel();
        let (blake3_tx, blake3_rx) = std::sync::mpsc::channel();

        // SHA256 computation
        let data_sha256 = data_arc.clone();
        self.thread_pool.spawn(move || {
            let mut hasher = Sha256::new();
            hasher.update(&*data_sha256);
            let _ = sha256_tx.send(hasher.finalize().to_vec());
        });

        // SHA3/Keccak256 computation
        let data_sha3 = data_arc.clone();
        self.thread_pool.spawn(move || {
            let mut hasher = Keccak256::new();
            hasher.update(&*data_sha3);
            let _ = sha3_tx.send(hasher.finalize().to_vec());
        });

        // BLAKE3 computation
        let data_blake3 = data_arc.clone();
        self.thread_pool.spawn(move || {
            let mut hasher = Blake3Hasher::new();
            hasher.update(&*data_blake3);
            let _ = blake3_tx.send(hasher.finalize().as_bytes().to_vec());
        });

        ParallelHashResult {
            sha256: sha256_rx.recv().unwrap_or_default(),
            sha3: sha3_rx.recv().unwrap_or_default(),
            blake3: blake3_rx.recv().unwrap_or_default(),
        }
    }

    /// Batch hash computation for multiple inputs
    pub fn batch_hash_sha256(&self, inputs: &[Vec<u8>]) -> Vec<Vec<u8>> {
        inputs
            .par_iter()
            .map(|data| {
                let mut hasher = Sha256::new();
                hasher.update(data);
                hasher.finalize().to_vec()
            })
            .collect()
    }

    /// Batch hash computation with BLAKE3 (fastest)
    pub fn batch_hash_blake3(&self, inputs: &[Vec<u8>]) -> Vec<Vec<u8>> {
        inputs
            .par_iter()
            .map(|data| {
                let mut hasher = Blake3Hasher::new();
                hasher.update(data);
                hasher.finalize().as_bytes().to_vec()
            })
            .collect()
    }
}

/// Optimized signature operations with batching and caching
#[derive(Debug, Clone)]
pub struct OptimizedSignatureEngine {
    cache: SignatureCache,
    hash_pool: HashPool,
    signing_keys: Arc<RwLock<HashMap<String, SigningKey>>>,
}

impl OptimizedSignatureEngine {
    pub fn new(num_hash_threads: usize) -> Result<Self> {
        Ok(Self {
            cache: SignatureCache::new(300, 10000), // 5 min TTL, 10k entries
            hash_pool: HashPool::new(num_hash_threads)?,
            signing_keys: Arc::new(RwLock::new(HashMap::new())),
        })
    }

    /// High-performance signature verification with caching
    pub async fn verify_signature_cached(
        &self,
        message: &[u8],
        signature: &[u8],
        public_key: &VerifyingKey,
    ) -> Result<bool> {
        let public_key_bytes = public_key.as_bytes();

        // Check cache first
        if let Some(cached_result) = self
            .cache
            .verify_cached(message, signature, public_key_bytes)
        {
            return Ok(cached_result);
        }

        // Perform verification
        let sig = Signature::from_bytes(
            signature
                .try_into()
                .map_err(|_| anyhow::anyhow!("Invalid signature length"))?,
        );
        let valid = public_key.verify(message, &sig).is_ok();

        // Cache the result
        self.cache
            .cache_result(message, signature, public_key_bytes, valid);

        Ok(valid)
    }

    /// Batch signature verification for multiple signatures
    pub async fn batch_verify_signatures(
        &self,
        verifications: &[(Vec<u8>, Vec<u8>, VerifyingKey)], // (message, signature, public_key)
    ) -> Result<Vec<bool>> {
        let results: Vec<bool> = verifications
            .par_iter()
            .map(|(message, signature, public_key)| {
                // Check cache first
                if let Some(cached) =
                    self.cache
                        .verify_cached(message, signature, public_key.as_bytes())
                {
                    return cached;
                }

                // Perform verification
                if signature.len() == 64 {
                    if let Ok(sig_bytes) = signature.as_slice().try_into() {
                        let sig = Signature::from_bytes(&sig_bytes);
                        let valid = public_key.verify(message, &sig).is_ok();
                        self.cache
                            .cache_result(message, signature, public_key.as_bytes(), valid);
                        return valid;
                    }
                }
                false
            })
            .collect();

        Ok(results)
    }

    /// Generate signatures with optimized key management
    pub async fn sign_message(&self, message: &[u8], key_id: &str) -> Result<Vec<u8>> {
        let signing_keys = self.signing_keys.read().await;

        if let Some(signing_key) = signing_keys.get(key_id) {
            let signature = signing_key.sign(message);
            Ok(signature.to_bytes().to_vec())
        } else {
            Err(anyhow::anyhow!("Signing key not found: {}", key_id))
        }
    }

    /// Add a signing key to the key pool
    pub async fn add_signing_key(&self, key_id: String, signing_key: SigningKey) {
        let mut signing_keys = self.signing_keys.write().await;
        signing_keys.insert(key_id, signing_key);
    }

    /// Batch sign multiple messages
    pub async fn batch_sign_messages(
        &self,
        messages: &[Vec<u8>],
        key_id: &str,
    ) -> Result<Vec<Vec<u8>>> {
        let signing_keys = self.signing_keys.read().await;

        if let Some(signing_key) = signing_keys.get(key_id) {
            let signatures: Vec<Vec<u8>> = messages
                .par_iter()
                .map(|message| {
                    let signature = signing_key.sign(message);
                    signature.to_bytes().to_vec()
                })
                .collect();
            Ok(signatures)
        } else {
            Err(anyhow::anyhow!("Signing key not found: {}", key_id))
        }
    }

    /// Get cache statistics
    pub fn get_cache_stats(&self) -> CacheStats {
        self.cache.stats()
    }
}

/// High-performance encryption engine with AES-256-GCM
#[derive(Debug)]
pub struct OptimizedEncryptionEngine {
    keys: Arc<RwLock<HashMap<String, LessSafeKey>>>,
}

impl OptimizedEncryptionEngine {
    pub fn new() -> Self {
        Self {
            keys: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Add an encryption key
    pub async fn add_key(&self, key_id: String, key_data: &[u8]) -> Result<()> {
        let unbound_key = UnboundKey::new(&AES_256_GCM, key_data)?;
        let key = LessSafeKey::new(unbound_key);

        let mut keys = self.keys.write().await;
        keys.insert(key_id, key);
        Ok(())
    }

    /// High-performance encryption
    pub async fn encrypt(
        &self,
        key_id: &str,
        plaintext: &[u8],
        associated_data: &[u8],
    ) -> Result<EncryptionResult> {
        let keys = self.keys.read().await;

        if let Some(key) = keys.get(key_id) {
            let mut nonce_bytes = [0u8; NONCE_LEN];
            OsRng.fill_bytes(&mut nonce_bytes);
            let nonce = Nonce::assume_unique_for_key(nonce_bytes);

            let mut ciphertext = plaintext.to_vec();
            key.seal_in_place_append_tag(nonce, Aad::from(associated_data), &mut ciphertext)?;

            Ok(EncryptionResult {
                ciphertext,
                nonce: nonce_bytes.to_vec(),
            })
        } else {
            Err(anyhow::anyhow!("Encryption key not found: {}", key_id))
        }
    }

    /// High-performance decryption
    pub async fn decrypt(
        &self,
        key_id: &str,
        ciphertext: &[u8],
        nonce: &[u8],
        associated_data: &[u8],
    ) -> Result<Vec<u8>> {
        let keys = self.keys.read().await;

        if let Some(key) = keys.get(key_id) {
            let nonce_array: [u8; NONCE_LEN] = nonce
                .try_into()
                .map_err(|_| anyhow::anyhow!("Invalid nonce length"))?;
            let nonce = Nonce::assume_unique_for_key(nonce_array);

            let mut ciphertext_with_tag = ciphertext.to_vec();
            let plaintext =
                key.open_in_place(nonce, Aad::from(associated_data), &mut ciphertext_with_tag)?;

            Ok(plaintext.to_vec())
        } else {
            Err(anyhow::anyhow!("Decryption key not found: {}", key_id))
        }
    }

    /// Batch encryption for multiple plaintexts
    pub async fn batch_encrypt(
        &self,
        key_id: &str,
        plaintexts: &[Vec<u8>],
        associated_data: &[u8],
    ) -> Result<Vec<EncryptionResult>> {
        let keys = self.keys.read().await;

        if let Some(key) = keys.get(key_id) {
            let results: Result<Vec<EncryptionResult>, _> = plaintexts
                .par_iter()
                .map(|plaintext| {
                    let mut nonce_bytes = [0u8; NONCE_LEN];
                    OsRng.fill_bytes(&mut nonce_bytes);
                    let nonce = Nonce::assume_unique_for_key(nonce_bytes);

                    let mut ciphertext = plaintext.clone();
                    key.seal_in_place_append_tag(
                        nonce,
                        Aad::from(associated_data),
                        &mut ciphertext,
                    )?;

                    Ok(EncryptionResult {
                        ciphertext,
                        nonce: nonce_bytes.to_vec(),
                    })
                })
                .collect();

            results
        } else {
            Err(anyhow::anyhow!("Encryption key not found: {}", key_id))
        }
    }
}

/// Combined high-performance crypto engine
#[derive(Debug)]
pub struct CryptoEngine {
    pub signatures: OptimizedSignatureEngine,
    pub encryption: OptimizedEncryptionEngine,
    pub hashing: HashPool,
    performance_stats: Arc<RwLock<CryptoPerformanceStats>>,
}

impl CryptoEngine {
    pub fn new(num_hash_threads: usize) -> Result<Self> {
        Ok(Self {
            signatures: OptimizedSignatureEngine::new(num_hash_threads)?,
            encryption: OptimizedEncryptionEngine::new(),
            hashing: HashPool::new(num_hash_threads)?,
            performance_stats: Arc::new(RwLock::new(CryptoPerformanceStats::default())),
        })
    }

    /// Comprehensive performance test
    pub async fn benchmark_operations(&self, iterations: usize) -> Result<BenchmarkResults> {
        let test_data = vec![0u8; 1024]; // 1KB test data
        let mut results = BenchmarkResults::default();

        // Hash benchmarks
        let start = Instant::now();
        for _ in 0..iterations {
            let _ = self.hashing.parallel_multi_hash(&test_data);
        }
        results.hash_ops_per_sec = iterations as f64 / start.elapsed().as_secs_f64();

        // Signature benchmarks
        let csprng = OsRng;
        let signing_key = SigningKey::from_bytes(&[0u8; 32]);
        let verifying_key = signing_key.verifying_key();

        self.signatures
            .add_signing_key("test".to_string(), signing_key)
            .await;

        let start = Instant::now();
        for _ in 0..iterations {
            let _ = self.signatures.sign_message(&test_data, "test").await?;
        }
        results.sign_ops_per_sec = iterations as f64 / start.elapsed().as_secs_f64();

        // Verification benchmark
        let signature = self.signatures.sign_message(&test_data, "test").await?;
        let start = Instant::now();
        for _ in 0..iterations {
            let _ = self
                .signatures
                .verify_signature_cached(&test_data, &signature, &verifying_key)
                .await?;
        }
        results.verify_ops_per_sec = iterations as f64 / start.elapsed().as_secs_f64();

        // Encryption benchmark
        let key_data = [0u8; 32];
        self.encryption
            .add_key("test".to_string(), &key_data)
            .await?;

        let start = Instant::now();
        for _ in 0..iterations {
            let _ = self.encryption.encrypt("test", &test_data, b"").await?;
        }
        results.encrypt_ops_per_sec = iterations as f64 / start.elapsed().as_secs_f64();

        Ok(results)
    }

    /// Get comprehensive performance statistics
    pub async fn get_performance_stats(&self) -> CryptoPerformanceStats {
        self.performance_stats.read().await.clone()
    }
}

/// Results from parallel hash computation
#[derive(Debug, Clone)]
pub struct ParallelHashResult {
    pub sha256: Vec<u8>,
    pub sha3: Vec<u8>,
    pub blake3: Vec<u8>,
}

/// Encryption result with nonce
#[derive(Debug, Clone)]
pub struct EncryptionResult {
    pub ciphertext: Vec<u8>,
    pub nonce: Vec<u8>,
}

/// Cache statistics
#[derive(Debug, Clone)]
pub struct CacheStats {
    pub entries: usize,
    pub max_entries: usize,
}

/// Performance statistics for crypto operations
#[derive(Debug, Clone, Default)]
pub struct CryptoPerformanceStats {
    pub total_signatures_generated: u64,
    pub total_signatures_verified: u64,
    pub total_encryptions: u64,
    pub total_decryptions: u64,
    pub total_hashes_computed: u64,
    pub cache_hit_rate: f64,
    pub average_sign_time_ms: f64,
    pub average_verify_time_ms: f64,
    pub average_encrypt_time_ms: f64,
    pub average_hash_time_ms: f64,
}

/// Benchmark results
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct BenchmarkResults {
    pub hash_ops_per_sec: f64,
    pub sign_ops_per_sec: f64,
    pub verify_ops_per_sec: f64,
    pub encrypt_ops_per_sec: f64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_signature_cache() {
        let cache = SignatureCache::new(60, 1000);
        let message = b"test message";
        let signature = [0u8; 64];
        let public_key = [1u8; 32];

        // No cache hit initially
        assert!(cache
            .verify_cached(message, &signature, &public_key)
            .is_none());

        // Cache result
        cache.cache_result(message, &signature, &public_key, true);

        // Should hit cache now
        assert_eq!(
            cache.verify_cached(message, &signature, &public_key),
            Some(true)
        );
    }

    #[tokio::test]
    async fn test_parallel_hashing() {
        let hash_pool = HashPool::new(4).unwrap();
        let test_data = b"test data for hashing";

        let result = hash_pool.parallel_multi_hash(test_data);

        // All hashes should be computed
        assert!(!result.sha256.is_empty());
        assert!(!result.sha3.is_empty());
        assert!(!result.blake3.is_empty());
    }

    #[tokio::test]
    async fn test_crypto_engine() {
        let engine = CryptoEngine::new(2).unwrap();

        // Run a small benchmark
        let results = engine.benchmark_operations(10).await.unwrap();

        // Should have positive performance numbers
        assert!(results.hash_ops_per_sec > 0.0);
        assert!(results.sign_ops_per_sec > 0.0);
        assert!(results.verify_ops_per_sec > 0.0);
        assert!(results.encrypt_ops_per_sec > 0.0);
    }
}
