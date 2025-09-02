use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::HashMap;

/// Proof that a keeper is storing data correctly
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageProof {
    /// ID of the keeper providing the proof
    pub keeper_id: String,

    /// Hash of the sigil being proven
    pub sigil_hash: String,

    /// Challenge data that was used to generate this proof
    pub challenge: Vec<u8>,

    /// The proof response data
    pub proof_data: Vec<u8>,

    /// When this proof was generated
    pub timestamp: DateTime<Utc>,
}

/// Proof that a keeper can retrieve data on demand
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetrievabilityProof {
    /// ID of the keeper providing the proof
    pub keeper_id: String,

    /// Hash of the sigil being retrieved
    pub sigil_hash: String,

    /// Requested byte ranges to retrieve
    pub requested_ranges: Vec<(usize, usize)>,

    /// The retrieved data for verification
    pub retrieved_data: Vec<u8>,

    /// Response time in milliseconds
    pub response_time_ms: u64,

    /// When this proof was generated
    pub timestamp: DateTime<Utc>,
}

/// Challenge issued to a keeper to prove storage
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageChallenge {
    /// Unique challenge ID
    pub challenge_id: String,

    /// Target keeper ID
    pub keeper_id: String,

    /// Sigil hash to prove
    pub sigil_hash: String,

    /// Random challenge data
    pub challenge_data: Vec<u8>,

    /// When this challenge was issued
    pub issued_at: DateTime<Utc>,

    /// When this challenge expires
    pub expires_at: DateTime<Utc>,
}

/// Result of verifying a storage proof
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProofVerificationResult {
    pub valid: bool,
    pub verification_time_ms: u64,
    pub error_message: Option<String>,
    pub score: f64, // 0.0 to 1.0
}

/// Engine for managing storage proofs and challenges
pub struct ProofEngine {
    /// Active challenges by challenge ID
    active_challenges: HashMap<String, StorageChallenge>,

    /// Proof history for reputation tracking
    proof_history: HashMap<String, Vec<StorageProof>>,
}

impl ProofEngine {
    pub fn new() -> Self {
        Self {
            active_challenges: HashMap::new(),
            proof_history: HashMap::new(),
        }
    }

    /// Issue a storage challenge to a keeper
    pub fn issue_storage_challenge(
        &mut self,
        keeper_id: &str,
        sigil_hash: &str,
    ) -> StorageChallenge {
        let challenge_id = format!("challenge_{}_{}", keeper_id, chrono::Utc::now().timestamp());

        // Generate deterministic challenge data based on keeper and sigil
        let mut hasher = Sha256::new();
        hasher.update(keeper_id);
        hasher.update(sigil_hash);
        hasher.update(challenge_id.as_bytes());
        let challenge_data = hasher.finalize().to_vec();

        let now = Utc::now();
        let challenge = StorageChallenge {
            challenge_id: challenge_id.clone(),
            keeper_id: keeper_id.to_string(),
            sigil_hash: sigil_hash.to_string(),
            challenge_data,
            issued_at: now,
            expires_at: now + chrono::Duration::minutes(5), // 5 minute timeout
        };

        self.active_challenges
            .insert(challenge_id, challenge.clone());
        challenge
    }

    /// Verify a storage proof submitted by a keeper
    pub fn verify_storage_proof(&mut self, proof: &StorageProof) -> ProofVerificationResult {
        let start_time = std::time::Instant::now();

        // Find the corresponding challenge
        let challenge = self
            .active_challenges
            .values()
            .find(|c| c.keeper_id == proof.keeper_id && c.sigil_hash == proof.sigil_hash);

        let challenge = match challenge {
            Some(c) => c,
            None => {
                return ProofVerificationResult {
                    valid: false,
                    verification_time_ms: start_time.elapsed().as_millis() as u64,
                    error_message: Some("No matching challenge found".to_string()),
                    score: 0.0,
                };
            }
        };

        // Check if challenge is still valid
        if Utc::now() > challenge.expires_at {
            return ProofVerificationResult {
                valid: false,
                verification_time_ms: start_time.elapsed().as_millis() as u64,
                error_message: Some("Challenge expired".to_string()),
                score: 0.0,
            };
        }

        // Verify proof data
        let expected_proof = self.generate_expected_proof(
            &challenge.sigil_hash,
            &challenge.challenge_data,
            &proof.keeper_id,
        );

        let valid = proof.proof_data == expected_proof;
        let score = if valid { 1.0 } else { 0.0 };

        // Record proof in history
        self.proof_history
            .entry(proof.keeper_id.clone())
            .or_insert_with(Vec::new)
            .push(proof.clone());

        // Clean up expired challenges
        self.cleanup_expired_challenges();

        ProofVerificationResult {
            valid,
            verification_time_ms: start_time.elapsed().as_millis() as u64,
            error_message: if valid {
                None
            } else {
                Some("Proof verification failed".to_string())
            },
            score,
        }
    }

    /// Generate the expected proof for a challenge
    fn generate_expected_proof(
        &self,
        sigil_hash: &str,
        challenge_data: &[u8],
        keeper_id: &str,
    ) -> Vec<u8> {
        let mut hasher = Sha256::new();
        hasher.update(sigil_hash.as_bytes());
        hasher.update(challenge_data);
        hasher.update(keeper_id.as_bytes());
        hasher.update(b"storage_proof");

        hasher.finalize().to_vec()
    }

    /// Issue a retrievability challenge
    pub fn issue_retrievability_challenge(
        &mut self,
        keeper_id: &str,
        sigil_hash: &str,
        data_size: usize,
    ) -> Vec<(usize, usize)> {
        // Request random byte ranges for proof of retrievability
        let mut ranges = Vec::new();
        let num_ranges = std::cmp::min(3, data_size / 1024); // Up to 3 ranges

        for i in 0..num_ranges {
            if data_size > 100 {
                // Use deterministic ranges based on sigil hash and iteration
                let hash_seed = sigil_hash
                    .chars()
                    .fold(0u32, |acc, c| acc.wrapping_mul(31).wrapping_add(c as u32));
                let start = (hash_seed.wrapping_add(i as u32 * 17) as usize) % (data_size - 100);
                let end = std::cmp::min(start + 100, data_size); // 100 byte samples
                ranges.push((start, end));
            }
        }

        if ranges.is_empty() && data_size > 0 {
            ranges.push((0, std::cmp::min(data_size, 100)));
        }

        ranges
    }

    /// Verify retrievability proof
    pub fn verify_retrievability_proof(
        &self,
        proof: &RetrievabilityProof,
        expected_data: &[u8],
    ) -> ProofVerificationResult {
        let start_time = std::time::Instant::now();
        let mut score = 0.0;
        let mut errors = Vec::new();

        // Verify each requested range
        let mut offset = 0;
        for (range_start, range_end) in &proof.requested_ranges {
            if *range_end > expected_data.len() {
                errors.push(format!(
                    "Range {}..{} exceeds data size",
                    range_start, range_end
                ));
                continue;
            }

            let expected_range = &expected_data[*range_start..*range_end];
            let proof_range_end = offset + (range_end - range_start);

            if proof_range_end > proof.retrieved_data.len() {
                errors.push(format!(
                    "Insufficient proof data for range {}..{}",
                    range_start, range_end
                ));
                continue;
            }

            let proof_range = &proof.retrieved_data[offset..proof_range_end];

            if expected_range == proof_range {
                score += 1.0;
            } else {
                errors.push(format!(
                    "Data mismatch in range {}..{}",
                    range_start, range_end
                ));
            }

            offset = proof_range_end;
        }

        // Normalize score
        if !proof.requested_ranges.is_empty() {
            score /= proof.requested_ranges.len() as f64;
        }

        // Penalize slow responses
        if proof.response_time_ms > 5000 {
            // > 5 seconds
            score *= 0.5;
        } else if proof.response_time_ms > 1000 {
            // > 1 second
            score *= 0.8;
        }

        let valid = score > 0.8 && errors.is_empty();

        ProofVerificationResult {
            valid,
            verification_time_ms: start_time.elapsed().as_millis() as u64,
            error_message: if errors.is_empty() {
                None
            } else {
                Some(errors.join("; "))
            },
            score,
        }
    }

    /// Get proof history for a keeper
    pub fn get_proof_history(&self, keeper_id: &str) -> Vec<StorageProof> {
        self.proof_history
            .get(keeper_id)
            .map(|proofs| proofs.clone())
            .unwrap_or_default()
    }

    /// Calculate keeper reliability score based on proof history
    pub fn calculate_reliability_score(&self, keeper_id: &str) -> f64 {
        let proofs = self.get_proof_history(keeper_id);

        if proofs.is_empty() {
            return 0.5; // Neutral score for new keepers
        }

        // For now, assume all proofs in history are valid (verified ones)
        // In production, store verification results too
        let recent_proofs = if proofs.len() > 100 {
            &proofs[proofs.len() - 100..] // Last 100 proofs
        } else {
            &proofs[..]
        };

        // Simple reliability based on number of recent proofs
        let score = if recent_proofs.len() >= 50 {
            1.0
        } else if recent_proofs.len() >= 20 {
            0.8
        } else if recent_proofs.len() >= 10 {
            0.6
        } else {
            0.5
        };

        score
    }

    /// Clean up expired challenges
    fn cleanup_expired_challenges(&mut self) {
        let now = Utc::now();
        self.active_challenges
            .retain(|_, challenge| now <= challenge.expires_at);
    }

    /// Get statistics about proof system
    pub fn get_proof_statistics(&self) -> ProofStatistics {
        let total_keepers = self.proof_history.len();
        let total_proofs: usize = self.proof_history.values().map(|v| v.len()).sum();
        let active_challenges = self.active_challenges.len();

        let avg_proofs_per_keeper = if total_keepers > 0 {
            total_proofs as f64 / total_keepers as f64
        } else {
            0.0
        };

        ProofStatistics {
            total_keepers,
            total_proofs,
            active_challenges,
            avg_proofs_per_keeper,
        }
    }
}

/// Statistics about the proof system
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProofStatistics {
    pub total_keepers: usize,
    pub total_proofs: usize,
    pub active_challenges: usize,
    pub avg_proofs_per_keeper: f64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_storage_challenge_and_proof() {
        let mut engine = ProofEngine::new();

        // Issue challenge
        let challenge = engine.issue_storage_challenge("keeper_1", "sigil_abc123");
        assert_eq!(challenge.keeper_id, "keeper_1");
        assert_eq!(challenge.sigil_hash, "sigil_abc123");
        assert!(!challenge.challenge_data.is_empty());

        // Generate valid proof
        let expected_proof = engine.generate_expected_proof(
            &challenge.sigil_hash,
            &challenge.challenge_data,
            &challenge.keeper_id,
        );

        let proof = StorageProof {
            keeper_id: challenge.keeper_id.clone(),
            sigil_hash: challenge.sigil_hash.clone(),
            challenge: challenge.challenge_data.clone(),
            proof_data: expected_proof,
            timestamp: Utc::now(),
        };

        // Verify proof
        let result = engine.verify_storage_proof(&proof);
        assert!(result.valid);
        assert_eq!(result.score, 1.0);
    }

    #[test]
    fn test_retrievability_proof() {
        let mut engine = ProofEngine::new();
        let test_data = b"Hello, World! This is test data for retrievability proof.";

        // Issue retrievability challenge
        let ranges =
            engine.issue_retrievability_challenge("keeper_1", "sigil_123", test_data.len());
        assert!(!ranges.is_empty());

        // Generate proof data
        let mut proof_data = Vec::new();
        for (start, end) in &ranges {
            proof_data.extend_from_slice(&test_data[*start..*end]);
        }

        let proof = RetrievabilityProof {
            keeper_id: "keeper_1".to_string(),
            sigil_hash: "sigil_123".to_string(),
            requested_ranges: ranges,
            retrieved_data: proof_data,
            response_time_ms: 150,
            timestamp: Utc::now(),
        };

        // Verify proof
        let result = engine.verify_retrievability_proof(&proof, test_data);
        assert!(result.valid);
        assert!(result.score > 0.8);
    }

    #[test]
    fn test_expired_challenge() {
        let mut engine = ProofEngine::new();

        // Create expired challenge manually
        let expired_challenge = StorageChallenge {
            challenge_id: "test_challenge".to_string(),
            keeper_id: "keeper_1".to_string(),
            sigil_hash: "sigil_123".to_string(),
            challenge_data: vec![1, 2, 3, 4],
            issued_at: Utc::now() - chrono::Duration::minutes(10),
            expires_at: Utc::now() - chrono::Duration::minutes(5),
        };

        engine
            .active_challenges
            .insert("test_challenge".to_string(), expired_challenge);

        let proof = StorageProof {
            keeper_id: "keeper_1".to_string(),
            sigil_hash: "sigil_123".to_string(),
            challenge: vec![1, 2, 3, 4],
            proof_data: vec![5, 6, 7, 8],
            timestamp: Utc::now(),
        };

        let result = engine.verify_storage_proof(&proof);
        assert!(!result.valid);
        assert!(result.error_message.is_some());
    }
}
