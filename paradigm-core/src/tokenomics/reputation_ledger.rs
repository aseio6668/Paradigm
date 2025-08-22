use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use uuid::Uuid;
use crate::Address;
use super::{ValidationResult, ContributionType};

/// Reputation ledger that tracks contributor trust and history
/// Implements Sybil-resistant, decay-based, peer validation system
#[derive(Debug)]
pub struct ReputationLedger {
    /// Reputation scores for each contributor
    reputation_scores: HashMap<Address, ReputationMetrics>,
    /// Historical reputation data
    reputation_history: HashMap<Address, Vec<ReputationEvent>>,
    /// Peer validation relationships
    peer_validations: HashMap<Address, HashMap<Address, PeerValidation>>,
    /// Reputation decay configuration
    decay_config: ReputationDecayConfig,
    /// Anti-Sybil detection system
    sybil_detector: SybilDetector,
}

impl ReputationLedger {
    pub fn new() -> Self {
        ReputationLedger {
            reputation_scores: HashMap::new(),
            reputation_history: HashMap::new(),
            peer_validations: HashMap::new(),
            decay_config: ReputationDecayConfig::default(),
            sybil_detector: SybilDetector::new(),
        }
    }

    pub async fn initialize(&mut self) -> anyhow::Result<()> {
        tracing::info!("Initializing Reputation Ledger");
        
        // Initialize Sybil detection
        self.sybil_detector.initialize().await?;
        
        tracing::info!("Reputation Ledger initialized with decay system");
        Ok(())
    }

    /// Get reputation metrics for a contributor
    pub async fn get_reputation(&self, contributor: &Address) -> anyhow::Result<ReputationMetrics> {
        if let Some(metrics) = self.reputation_scores.get(contributor) {
            // Apply decay to get current reputation
            let current_metrics = self.apply_reputation_decay(metrics).await?;
            Ok(current_metrics)
        } else {
            // New contributor gets default reputation
            Ok(ReputationMetrics::default())
        }
    }

    /// Update reputation based on contribution validation result
    pub async fn update_reputation(
        &mut self,
        contributor: &Address,
        validation_result: &ValidationResult,
    ) -> anyhow::Result<()> {
        // Get current reputation or create new
        let mut current_reputation = self.get_reputation(contributor).await?;

        // Calculate reputation changes based on contribution
        let contribution_impact = self.calculate_contribution_impact(validation_result).await?;

        // Update consistency score
        current_reputation.consistency_score = self.update_consistency_score(
            current_reputation.consistency_score,
            validation_result.quality_score,
            current_reputation.contribution_count,
        ).await?;

        // Update expertise score based on contribution type and quality
        current_reputation.expertise_score = self.update_expertise_score(
            current_reputation.expertise_score,
            validation_result,
        ).await?;

        // Update contribution count and average quality
        current_reputation.contribution_count += 1;
        current_reputation.average_quality = (
            (current_reputation.average_quality * (current_reputation.contribution_count - 1) as f64) +
            validation_result.quality_score
        ) / current_reputation.contribution_count as f64;

        // Update last activity
        current_reputation.last_activity = Utc::now();

        // Detect potential Sybil behavior
        let sybil_risk = self.sybil_detector.analyze_contributor(contributor, &current_reputation).await?;
        current_reputation.sybil_risk_score = sybil_risk;

        // Record reputation event
        let event = ReputationEvent {
            timestamp: Utc::now(),
            event_type: ReputationEventType::ContributionUpdate,
            impact: contribution_impact,
            quality_score: validation_result.quality_score,
            peer_validation_score: validation_result.peer_validation_score,
        };

        self.reputation_history
            .entry(contributor.clone())
            .or_insert_with(Vec::new)
            .push(event);

        // Store updated reputation  
        let consistency = current_reputation.consistency_score;
        let expertise = current_reputation.expertise_score;
        let trust = current_reputation.trust_score;
        
        self.reputation_scores.insert(contributor.clone(), current_reputation);

        tracing::debug!("Updated reputation for {}: consistency={:.3}, expertise={:.3}, trust={:.3}",
                       contributor.to_string(),
                       consistency,
                       expertise,
                       trust);

        Ok(())
    }

    /// Record peer validation
    pub async fn record_peer_validation(
        &mut self,
        validator: &Address,
        validated: &Address,
        validation: PeerValidation,
    ) -> anyhow::Result<()> {
        // Store peer validation
        let trust_score = validation.trust_score;
        self.peer_validations
            .entry(validator.clone())
            .or_insert_with(HashMap::new)
            .insert(validated.clone(), validation);

        // Update trust score for validated contributor
        self.update_trust_score(validated).await?;

        tracing::debug!("Recorded peer validation from {} to {} with score {:.3}",
                       validator.to_string(),
                       validated.to_string(),
                       trust_score);

        Ok(())
    }

    /// Apply reputation decay over time
    async fn apply_reputation_decay(&self, metrics: &ReputationMetrics) -> anyhow::Result<ReputationMetrics> {
        let now = Utc::now();
        let time_since_activity = now.signed_duration_since(metrics.last_activity);
        let days_inactive = time_since_activity.num_days() as f64;

        // Calculate decay factor
        let decay_factor = if days_inactive > 0.0 {
            (1.0 - self.decay_config.daily_decay_rate).powf(days_inactive)
        } else {
            1.0
        };

        let mut decayed_metrics = metrics.clone();
        
        // Apply decay to scores
        decayed_metrics.consistency_score *= decay_factor;
        decayed_metrics.expertise_score *= decay_factor;
        
        // Trust score decays more slowly
        let trust_decay_factor = (1.0 - self.decay_config.trust_decay_rate).powf(days_inactive);
        decayed_metrics.trust_score *= trust_decay_factor;

        // Ensure minimum scores
        decayed_metrics.consistency_score = decayed_metrics.consistency_score.max(0.1);
        decayed_metrics.expertise_score = decayed_metrics.expertise_score.max(0.1);
        decayed_metrics.trust_score = decayed_metrics.trust_score.max(0.1);

        Ok(decayed_metrics)
    }

    async fn calculate_contribution_impact(&self, validation_result: &ValidationResult) -> anyhow::Result<f64> {
        // Calculate positive impact based on quality and novelty
        let quality_impact = validation_result.quality_score * 0.6;
        let novelty_impact = validation_result.novelty_score * 0.3;
        let peer_impact = validation_result.peer_validation_score * 0.1;

        Ok(quality_impact + novelty_impact + peer_impact)
    }

    async fn update_consistency_score(
        &self,
        current_score: f64,
        new_quality: f64,
        contribution_count: u64,
    ) -> anyhow::Result<f64> {
        // Consistency measures how reliable a contributor is
        let weight = 1.0 / (contribution_count as f64 + 1.0);
        let quality_deviation = (new_quality - current_score).abs();
        
        // Lower deviation = higher consistency
        let consistency_bonus = (1.0 - quality_deviation).max(0.0);
        let new_score = current_score * (1.0 - weight) + consistency_bonus * weight;
        
        Ok(new_score.min(1.0))
    }

    async fn update_expertise_score(
        &self,
        current_score: f64,
        validation_result: &ValidationResult,
    ) -> anyhow::Result<f64> {
        // Expertise grows with high-quality contributions
        let expertise_gain = validation_result.quality_score * 0.1;
        let new_score = current_score + expertise_gain;
        
        Ok(new_score.min(1.0))
    }

    async fn update_trust_score(&mut self, contributor: &Address) -> anyhow::Result<()> {
        // Calculate trust score based on peer validations
        let mut total_trust = 0.0;
        let mut validation_count = 0;

        // Aggregate peer validations for this contributor
        for validations in self.peer_validations.values() {
            if let Some(validation) = validations.get(contributor) {
                total_trust += validation.trust_score;
                validation_count += 1;
            }
        }

        // Calculate average trust score
        let trust_score = if validation_count > 0 {
            total_trust / validation_count as f64
        } else {
            0.5 // Default neutral trust
        };

        // Update reputation metrics
        if let Some(metrics) = self.reputation_scores.get_mut(contributor) {
            metrics.trust_score = trust_score;
        }

        Ok(())
    }

    /// Get reputation ranking of contributors
    pub fn get_reputation_ranking(&self, limit: usize) -> Vec<(Address, ReputationMetrics)> {
        let mut contributors: Vec<_> = self.reputation_scores.iter()
            .map(|(addr, metrics)| (addr.clone(), metrics.clone()))
            .collect();

        // Sort by combined reputation score
        contributors.sort_by(|a, b| {
            let score_a = (a.1.consistency_score + a.1.expertise_score + a.1.trust_score) / 3.0;
            let score_b = (b.1.consistency_score + b.1.expertise_score + b.1.trust_score) / 3.0;
            score_b.partial_cmp(&score_a).unwrap_or(std::cmp::Ordering::Equal)
        });

        contributors.into_iter().take(limit).collect()
    }

    /// Get reputation statistics
    pub fn get_reputation_stats(&self) -> ReputationStats {
        if self.reputation_scores.is_empty() {
            return ReputationStats::default();
        }

        let total_contributors = self.reputation_scores.len();
        let avg_consistency = self.reputation_scores.values()
            .map(|m| m.consistency_score)
            .sum::<f64>() / total_contributors as f64;
        let avg_expertise = self.reputation_scores.values()
            .map(|m| m.expertise_score)
            .sum::<f64>() / total_contributors as f64;
        let avg_trust = self.reputation_scores.values()
            .map(|m| m.trust_score)
            .sum::<f64>() / total_contributors as f64;

        let high_reputation_count = self.reputation_scores.values()
            .filter(|m| (m.consistency_score + m.expertise_score + m.trust_score) / 3.0 > 0.8)
            .count();

        ReputationStats {
            total_contributors,
            average_consistency_score: avg_consistency,
            average_expertise_score: avg_expertise,
            average_trust_score: avg_trust,
            high_reputation_contributors: high_reputation_count,
        }
    }

    /// Detect and flag potential Sybil attacks
    pub async fn detect_sybil_attacks(&mut self) -> anyhow::Result<Vec<Address>> {
        self.sybil_detector.detect_sybil_networks(&self.reputation_scores).await
    }
}

/// Anti-Sybil detection system
#[derive(Debug)]
pub struct SybilDetector {
    /// Behavioral patterns that indicate Sybil attacks
    suspicious_patterns: Vec<SybilPattern>,
    /// Network analysis for detecting coordinated behavior
    network_analyzer: NetworkAnalyzer,
}

impl SybilDetector {
    pub fn new() -> Self {
        SybilDetector {
            suspicious_patterns: Vec::new(),
            network_analyzer: NetworkAnalyzer::new(),
        }
    }

    pub async fn initialize(&mut self) -> anyhow::Result<()> {
        // Initialize suspicious patterns
        self.suspicious_patterns = vec![
            SybilPattern::IdenticalBehavior,
            SybilPattern::CoordinatedTiming,
            SybilPattern::SimilarPerformance,
            SybilPattern::NetworkClustering,
        ];

        self.network_analyzer.initialize().await?;
        
        tracing::debug!("Sybil detector initialized with {} patterns", 
                       self.suspicious_patterns.len());
        Ok(())
    }

    pub async fn analyze_contributor(
        &self,
        _contributor: &Address,
        _reputation: &ReputationMetrics,
    ) -> anyhow::Result<f64> {
        // In a real implementation, this would analyze behavioral patterns
        // For now, return a low risk score
        Ok(0.1)
    }

    pub async fn detect_sybil_networks(
        &self,
        _reputation_scores: &HashMap<Address, ReputationMetrics>,
    ) -> anyhow::Result<Vec<Address>> {
        // In a real implementation, this would use graph analysis and ML
        // to detect coordinated Sybil networks
        Ok(Vec::new())
    }
}

/// Network analyzer for detecting coordinated behavior
#[derive(Debug)]
pub struct NetworkAnalyzer {
    // Graph-based analysis for detecting Sybil networks
}

impl NetworkAnalyzer {
    pub fn new() -> Self {
        NetworkAnalyzer {}
    }

    pub async fn initialize(&mut self) -> anyhow::Result<()> {
        Ok(())
    }
}

// Data structures

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReputationMetrics {
    pub consistency_score: f64,   // How consistent are their contributions
    pub expertise_score: f64,     // Level of expertise in their domain
    pub trust_score: f64,         // Peer trust and validation
    pub contribution_count: u64,  // Total number of contributions
    pub average_quality: f64,     // Average quality of contributions
    pub last_activity: DateTime<Utc>, // Last contribution timestamp
    pub sybil_risk_score: f64,    // Risk of being a Sybil identity
}

impl Default for ReputationMetrics {
    fn default() -> Self {
        ReputationMetrics {
            consistency_score: 0.5,
            expertise_score: 0.3,
            trust_score: 0.5,
            contribution_count: 0,
            average_quality: 0.0,
            last_activity: Utc::now(),
            sybil_risk_score: 0.1,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ReputationEvent {
    pub timestamp: DateTime<Utc>,
    pub event_type: ReputationEventType,
    pub impact: f64,
    pub quality_score: f64,
    pub peer_validation_score: f64,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum ReputationEventType {
    ContributionUpdate,
    PeerValidation,
    SlashingPenalty,
    ReputationDecay,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PeerValidation {
    pub validator: Address,
    pub validated: Address,
    pub trust_score: f64,
    pub validation_type: ValidationType,
    pub timestamp: DateTime<Utc>,
    pub evidence: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum ValidationType {
    WorkQuality,
    Expertise,
    Reliability,
    Honesty,
}

#[derive(Debug)]
pub struct ReputationDecayConfig {
    pub daily_decay_rate: f64,    // Daily decay rate for reputation scores
    pub trust_decay_rate: f64,    // Decay rate for trust score (slower)
    pub minimum_score: f64,       // Minimum reputation score
}

impl Default for ReputationDecayConfig {
    fn default() -> Self {
        ReputationDecayConfig {
            daily_decay_rate: 0.001,  // 0.1% per day
            trust_decay_rate: 0.0005, // 0.05% per day
            minimum_score: 0.1,       // 10% minimum
        }
    }
}

#[derive(Debug)]
pub enum SybilPattern {
    IdenticalBehavior,    // Multiple accounts with identical behavior
    CoordinatedTiming,    // Coordinated submission timing
    SimilarPerformance,   // Suspiciously similar performance metrics
    NetworkClustering,    // Clustering in social/trust networks
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct ReputationStats {
    pub total_contributors: usize,
    pub average_consistency_score: f64,
    pub average_expertise_score: f64,
    pub average_trust_score: f64,
    pub high_reputation_contributors: usize,
}

#[cfg(test)]
mod tests {
    use super::*;
    use ed25519_dalek::Keypair;
    use rand::thread_rng;

    #[tokio::test]
    async fn test_reputation_initialization() {
        let mut ledger = ReputationLedger::new();
        ledger.initialize().await.unwrap();

        let keypair = Keypair::generate(&mut thread_rng());
        let contributor = Address::from_public_key(&keypair.public);

        // New contributor should have default reputation
        let reputation = ledger.get_reputation(&contributor).await.unwrap();
        assert_eq!(reputation.consistency_score, 0.5);
        assert_eq!(reputation.expertise_score, 0.3);
        assert_eq!(reputation.trust_score, 0.5);
        assert_eq!(reputation.contribution_count, 0);
    }

    #[tokio::test]
    async fn test_reputation_update() {
        let mut ledger = ReputationLedger::new();
        ledger.initialize().await.unwrap();

        let keypair = Keypair::generate(&mut thread_rng());
        let contributor = Address::from_public_key(&keypair.public);

        let validation_result = ValidationResult {
            valid: true,
            compute_units: 1000,
            quality_score: 0.9,
            novelty_score: 0.7,
            peer_validation_score: 0.8,
        };

        // Update reputation
        ledger.update_reputation(&contributor, &validation_result).await.unwrap();

        // Check updated reputation
        let reputation = ledger.get_reputation(&contributor).await.unwrap();
        assert_eq!(reputation.contribution_count, 1);
        assert_eq!(reputation.average_quality, 0.9);
        assert!(reputation.expertise_score > 0.3); // Should have increased
    }

    #[tokio::test]
    async fn test_peer_validation() {
        let mut ledger = ReputationLedger::new();
        ledger.initialize().await.unwrap();

        let keypair1 = Keypair::generate(&mut thread_rng());
        let keypair2 = Keypair::generate(&mut thread_rng());
        let validator = Address::from_public_key(&keypair1.public);
        let validated = Address::from_public_key(&keypair2.public);

        let peer_validation = PeerValidation {
            validator,
            validated,
            trust_score: 0.9,
            validation_type: ValidationType::WorkQuality,
            timestamp: Utc::now(),
            evidence: Some("High quality work verified".to_string()),
        };

        ledger.record_peer_validation(&validator, &validated, peer_validation).await.unwrap();

        // Check that trust score was updated
        let reputation = ledger.get_reputation(&validated).await.unwrap();
        assert!(reputation.trust_score > 0.5); // Should have increased from default
    }

    #[tokio::test]
    async fn test_reputation_stats() {
        let mut ledger = ReputationLedger::new();
        ledger.initialize().await.unwrap();

        // Add some test data
        for i in 0..5 {
            let keypair = Keypair::generate(&mut thread_rng());
            let contributor = Address::from_public_key(&keypair.public);
            
            let validation_result = ValidationResult {
                valid: true,
                compute_units: 1000,
                quality_score: 0.8,
                novelty_score: 0.6,
                peer_validation_score: 0.7,
            };

            ledger.update_reputation(&contributor, &validation_result).await.unwrap();
        }

        let stats = ledger.get_reputation_stats();
        assert_eq!(stats.total_contributors, 5);
        assert!(stats.average_consistency_score > 0.0);
        assert!(stats.average_expertise_score > 0.0);
        assert!(stats.average_trust_score > 0.0);
    }
}