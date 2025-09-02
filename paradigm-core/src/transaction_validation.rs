// Formal Transaction Validation Rules
// Provides comprehensive validation for all transaction types with security and business logic enforcement

use anyhow::Result;
use chrono::{DateTime, Duration as ChronoDuration, Utc};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use tokio::sync::RwLock;

use crate::{Address, Amount, PublicKey, Transaction};

/// Validation error types
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ValidationError {
    InvalidSignature,
    InsufficientBalance,
    InvalidAmount,
    InvalidFee,
    TransactionTooOld,
    TransactionInFuture,
    DuplicateTransaction,
    InvalidNonce,
    ExceedsRateLimit,
    BlockedAddress,
    InvalidMessageFormat,
    ExceedsMaxTransactionSize,
    UnsupportedTransactionType,
    InvalidTimestamp,
    NetworkMismatch,
    PolicyViolation(String),
}

/// Transaction validation result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationResult {
    pub is_valid: bool,
    pub errors: Vec<ValidationError>,
    pub warnings: Vec<String>,
    pub validation_time_ms: u64,
    pub risk_score: u32, // 0-100, higher = riskier
    pub validation_version: String,
}

/// Transaction validation rule types
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ValidationRuleType {
    Signature,
    Balance,
    Format,
    Temporal,
    RateLimit,
    Policy,
    Network,
    Risk,
}

/// Individual validation rule
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationRule {
    pub id: String,
    pub name: String,
    pub rule_type: ValidationRuleType,
    pub enabled: bool,
    pub severity: ValidationSeverity,
    pub description: String,
    pub parameters: HashMap<String, String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Validation severity levels
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ValidationSeverity {
    Critical, // Transaction must be rejected
    Warning,  // Log warning but allow transaction
    Info,     // Informational only
}

/// Network configuration for validation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkValidationConfig {
    pub network_id: String,
    pub chain_id: u64,
    pub min_fee: Amount,
    pub max_transaction_size: usize,
    pub max_message_size: usize,
    pub transaction_timeout_hours: u32,
    pub rate_limit_per_address_per_hour: u32,
    pub require_memo_for_large_amounts: bool,
    pub large_amount_threshold: Amount,
    pub enable_address_whitelist: bool,
    pub enable_address_blacklist: bool,
    pub enable_risk_scoring: bool,
}

impl Default for NetworkValidationConfig {
    fn default() -> Self {
        Self {
            network_id: "paradigm-mainnet".to_string(),
            chain_id: 1,
            min_fee: 1_000_000,                // 0.01 PAR
            max_transaction_size: 1024 * 1024, // 1MB
            max_message_size: 1024,            // 1KB
            transaction_timeout_hours: 24,
            rate_limit_per_address_per_hour: 1000,
            require_memo_for_large_amounts: true,
            large_amount_threshold: 1_000_000_000_000, // 10,000 PAR
            enable_address_whitelist: false,
            enable_address_blacklist: true,
            enable_risk_scoring: true,
        }
    }
}

/// Address tracking for rate limiting and risk scoring
#[derive(Debug, Clone)]
struct AddressTracker {
    address: Address,
    transaction_count: u32,
    last_transaction: DateTime<Utc>,
    total_volume: Amount,
    risk_flags: HashSet<String>,
    first_seen: DateTime<Utc>,
}

/// Transaction validation engine
pub struct TransactionValidator {
    config: NetworkValidationConfig,
    rules: Arc<RwLock<HashMap<String, ValidationRule>>>,
    address_trackers: Arc<RwLock<HashMap<String, AddressTracker>>>,
    processed_transactions: Arc<RwLock<HashSet<String>>>, // Track duplicates
    whitelisted_addresses: Arc<RwLock<HashSet<String>>>,
    blacklisted_addresses: Arc<RwLock<HashSet<String>>>,
    validation_stats: Arc<RwLock<ValidationStats>>,
}

/// Validation statistics
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ValidationStats {
    pub total_validations: u64,
    pub successful_validations: u64,
    pub failed_validations: u64,
    pub average_validation_time_ms: f64,
    pub rule_trigger_counts: HashMap<String, u64>,
    pub error_counts: HashMap<String, u64>,
    pub risk_score_distribution: [u64; 10], // Buckets for risk scores 0-9, 10-19, ..., 90-100
}

impl TransactionValidator {
    pub async fn new(config: NetworkValidationConfig) -> Result<Self> {
        let mut validator = Self {
            config,
            rules: Arc::new(RwLock::new(HashMap::new())),
            address_trackers: Arc::new(RwLock::new(HashMap::new())),
            processed_transactions: Arc::new(RwLock::new(HashSet::new())),
            whitelisted_addresses: Arc::new(RwLock::new(HashSet::new())),
            blacklisted_addresses: Arc::new(RwLock::new(HashSet::new())),
            validation_stats: Arc::new(RwLock::new(ValidationStats::default())),
        };

        // Initialize default validation rules
        validator.initialize_default_rules().await?;

        tracing::info!(
            "🔍 Transaction validator initialized with {} rules",
            validator.rules.read().await.len()
        );

        Ok(validator)
    }

    /// Validate a transaction against all active rules
    pub async fn validate_transaction(
        &self,
        transaction: &Transaction,
        sender_balance: Amount,
        sender_public_key: &PublicKey,
    ) -> Result<ValidationResult> {
        let start_time = std::time::Instant::now();
        let mut errors = Vec::new();
        let mut warnings = Vec::new();
        let mut risk_score = 0u32;

        tracing::debug!("🔍 Validating transaction: {}", transaction.id);

        // Get active rules
        let rules = self.rules.read().await;
        let active_rules: Vec<ValidationRule> = rules
            .values()
            .filter(|rule| rule.enabled)
            .cloned()
            .collect();

        // Track validation
        self.track_validation_start().await;

        // Apply validation rules in order of severity
        for rule in active_rules.iter() {
            match rule.rule_type {
                ValidationRuleType::Signature => {
                    if let Err(e) = self
                        .validate_signature(transaction, sender_public_key)
                        .await
                    {
                        errors.push(e);
                        self.increment_rule_trigger(&rule.id).await;
                    }
                }
                ValidationRuleType::Balance => {
                    if let Err(e) = self.validate_balance(transaction, sender_balance).await {
                        errors.push(e);
                        self.increment_rule_trigger(&rule.id).await;
                    }
                }
                ValidationRuleType::Format => {
                    if let Err(e) = self.validate_format(transaction).await {
                        errors.push(e);
                        self.increment_rule_trigger(&rule.id).await;
                    }
                }
                ValidationRuleType::Temporal => {
                    if let Err(e) = self.validate_temporal(transaction).await {
                        errors.push(e);
                        self.increment_rule_trigger(&rule.id).await;
                    }
                }
                ValidationRuleType::RateLimit => {
                    if let Err(e) = self.validate_rate_limit(transaction).await {
                        errors.push(e);
                        self.increment_rule_trigger(&rule.id).await;
                    }
                }
                ValidationRuleType::Policy => {
                    if let Err(e) = self.validate_policy(transaction).await {
                        if rule.severity == ValidationSeverity::Critical {
                            errors.push(e);
                        } else {
                            warnings.push(format!("Policy warning: {:?}", e));
                        }
                        self.increment_rule_trigger(&rule.id).await;
                    }
                }
                ValidationRuleType::Network => {
                    if let Err(e) = self.validate_network(transaction).await {
                        errors.push(e);
                        self.increment_rule_trigger(&rule.id).await;
                    }
                }
                ValidationRuleType::Risk => {
                    let score = self.calculate_risk_score(transaction).await;
                    risk_score = risk_score.max(score);
                    if score > 80 {
                        warnings.push(format!("High risk transaction detected (score: {})", score));
                    }
                }
            }
        }

        // Check for duplicate transactions
        if let Err(e) = self.validate_duplicate(transaction).await {
            errors.push(e);
        }

        let validation_time_ms = start_time.elapsed().as_millis() as u64;
        let is_valid = errors.is_empty();

        let result = ValidationResult {
            is_valid,
            errors,
            warnings,
            validation_time_ms,
            risk_score,
            validation_version: "1.0.0".to_string(),
        };

        // Update statistics
        self.update_validation_stats(&result).await;

        // Track transaction if valid
        if result.is_valid {
            self.track_valid_transaction(transaction).await?;
        }

        tracing::debug!(
            "✅ Validation completed: {} ({}ms, risk: {})",
            if result.is_valid { "VALID" } else { "INVALID" },
            validation_time_ms,
            risk_score
        );

        Ok(result)
    }

    /// Add a custom validation rule
    pub async fn add_validation_rule(&self, rule: ValidationRule) -> Result<()> {
        let mut rules = self.rules.write().await;
        rules.insert(rule.id.clone(), rule.clone());

        tracing::info!("➕ Added validation rule: {} ({})", rule.name, rule.id);
        Ok(())
    }

    /// Remove a validation rule
    pub async fn remove_validation_rule(&self, rule_id: &str) -> Result<bool> {
        let mut rules = self.rules.write().await;
        let removed = rules.remove(rule_id).is_some();

        if removed {
            tracing::info!("➖ Removed validation rule: {}", rule_id);
        }

        Ok(removed)
    }

    /// Enable/disable a validation rule
    pub async fn toggle_validation_rule(&self, rule_id: &str, enabled: bool) -> Result<bool> {
        let mut rules = self.rules.write().await;

        if let Some(rule) = rules.get_mut(rule_id) {
            rule.enabled = enabled;
            rule.updated_at = Utc::now();
            tracing::info!(
                "🔄 Toggled rule {} to {}",
                rule_id,
                if enabled { "enabled" } else { "disabled" }
            );
            Ok(true)
        } else {
            Ok(false)
        }
    }

    /// Add address to whitelist
    pub async fn whitelist_address(&self, address: &Address) -> Result<()> {
        let mut whitelist = self.whitelisted_addresses.write().await;
        whitelist.insert(address.to_string());

        tracing::info!("✅ Added address to whitelist: {}", address.to_string());
        Ok(())
    }

    /// Add address to blacklist
    pub async fn blacklist_address(&self, address: &Address) -> Result<()> {
        let mut blacklist = self.blacklisted_addresses.write().await;
        blacklist.insert(address.to_string());

        tracing::info!("🚫 Added address to blacklist: {}", address.to_string());
        Ok(())
    }

    /// Get validation statistics
    pub async fn get_validation_stats(&self) -> ValidationStats {
        self.validation_stats.read().await.clone()
    }

    /// Get all validation rules
    pub async fn get_validation_rules(&self) -> Vec<ValidationRule> {
        self.rules.read().await.values().cloned().collect()
    }

    // Private validation methods

    async fn validate_signature(
        &self,
        transaction: &Transaction,
        public_key: &PublicKey,
    ) -> Result<(), ValidationError> {
        if transaction.signature.is_empty() {
            return Err(ValidationError::InvalidSignature);
        }

        // Validate signature using the public key
        match transaction.validate(public_key) {
            Ok(_) => Ok(()),
            Err(_) => Err(ValidationError::InvalidSignature),
        }
    }

    async fn validate_balance(
        &self,
        transaction: &Transaction,
        sender_balance: Amount,
    ) -> Result<(), ValidationError> {
        let total_cost = transaction.amount + transaction.fee;

        if sender_balance < total_cost {
            return Err(ValidationError::InsufficientBalance);
        }

        if transaction.amount == 0 {
            return Err(ValidationError::InvalidAmount);
        }

        if transaction.fee < self.config.min_fee {
            return Err(ValidationError::InvalidFee);
        }

        Ok(())
    }

    async fn validate_format(&self, transaction: &Transaction) -> Result<(), ValidationError> {
        // Check message size
        if let Some(ref message) = transaction.message {
            if message.len() > self.config.max_message_size {
                return Err(ValidationError::InvalidMessageFormat);
            }

            // Check for invalid characters or suspicious content
            if message.contains('\0') || message.len() > 1000 {
                return Err(ValidationError::InvalidMessageFormat);
            }
        }

        // Validate addresses are not the same (prevent self-sends in certain contexts)
        if transaction.from == transaction.to {
            // Allow self-sends but flag for risk scoring
        }

        Ok(())
    }

    async fn validate_temporal(&self, transaction: &Transaction) -> Result<(), ValidationError> {
        let now = Utc::now().timestamp();
        let tx_time = transaction.timestamp.timestamp();

        // Check if transaction is too old
        let max_age_seconds = (self.config.transaction_timeout_hours as i64) * 3600;
        if now - tx_time > max_age_seconds {
            return Err(ValidationError::TransactionTooOld);
        }

        // Check if transaction is from the future (allow small clock skew)
        if tx_time > now + 300 {
            // 5 minutes tolerance
            return Err(ValidationError::TransactionInFuture);
        }

        Ok(())
    }

    async fn validate_rate_limit(&self, transaction: &Transaction) -> Result<(), ValidationError> {
        let address_key = transaction.from.to_string();
        let mut trackers = self.address_trackers.write().await;

        let now = Utc::now();
        let hour_ago = now - ChronoDuration::hours(1);

        match trackers.get_mut(&address_key) {
            Some(tracker) => {
                // Reset counter if it's been more than an hour
                if tracker.last_transaction < hour_ago {
                    tracker.transaction_count = 0;
                }

                if tracker.transaction_count >= self.config.rate_limit_per_address_per_hour {
                    return Err(ValidationError::ExceedsRateLimit);
                }

                tracker.transaction_count += 1;
                tracker.last_transaction = now;
                tracker.total_volume += transaction.amount;
            }
            None => {
                // First transaction from this address
                trackers.insert(
                    address_key,
                    AddressTracker {
                        address: transaction.from.clone(),
                        transaction_count: 1,
                        last_transaction: now,
                        total_volume: transaction.amount,
                        risk_flags: HashSet::new(),
                        first_seen: now,
                    },
                );
            }
        }

        Ok(())
    }

    async fn validate_policy(&self, transaction: &Transaction) -> Result<(), ValidationError> {
        // Check blacklist
        if self.config.enable_address_blacklist {
            let blacklist = self.blacklisted_addresses.read().await;
            if blacklist.contains(&transaction.from.to_string())
                || blacklist.contains(&transaction.to.to_string())
            {
                return Err(ValidationError::BlockedAddress);
            }
        }

        // Check whitelist (if enabled, only whitelisted addresses can transact)
        if self.config.enable_address_whitelist {
            let whitelist = self.whitelisted_addresses.read().await;
            if !whitelist.contains(&transaction.from.to_string())
                || !whitelist.contains(&transaction.to.to_string())
            {
                return Err(ValidationError::PolicyViolation(
                    "Address not whitelisted".to_string(),
                ));
            }
        }

        // Require memo for large transactions
        if self.config.require_memo_for_large_amounts
            && transaction.amount >= self.config.large_amount_threshold
            && transaction.message.is_none()
        {
            return Err(ValidationError::PolicyViolation(
                "Large transactions require memo".to_string(),
            ));
        }

        Ok(())
    }

    async fn validate_network(&self, _transaction: &Transaction) -> Result<(), ValidationError> {
        // Network-specific validations (chain ID, network ID, etc.)
        // For now, assume all transactions are for the correct network
        Ok(())
    }

    async fn validate_duplicate(&self, transaction: &Transaction) -> Result<(), ValidationError> {
        let tx_id = transaction.id.to_string();
        let mut processed = self.processed_transactions.write().await;

        if processed.contains(&tx_id) {
            return Err(ValidationError::DuplicateTransaction);
        }

        // Keep only recent transaction IDs (last 10,000)
        if processed.len() > 10_000 {
            // In a real implementation, we'd use a more sophisticated approach
            // like a time-based cleanup or LRU cache
            processed.clear();
        }

        Ok(())
    }

    async fn calculate_risk_score(&self, transaction: &Transaction) -> u32 {
        let mut score = 0u32;

        // Large amount = higher risk
        if transaction.amount > self.config.large_amount_threshold {
            score += 20;
        }

        // Check address history
        let trackers = self.address_trackers.read().await;
        if let Some(tracker) = trackers.get(&transaction.from.to_string()) {
            // New address = higher risk
            if tracker.first_seen > Utc::now() - ChronoDuration::days(7) {
                score += 15;
            }

            // High velocity = higher risk
            if tracker.transaction_count > 100 {
                score += 10;
            }

            // Risk flags
            score += tracker.risk_flags.len() as u32 * 5;
        } else {
            // Unknown address = moderate risk
            score += 10;
        }

        // Self-send = slight risk increase
        if transaction.from == transaction.to {
            score += 5;
        }

        // No memo on large transaction = risk increase
        if transaction.amount > self.config.large_amount_threshold && transaction.message.is_none()
        {
            score += 10;
        }

        score.min(100) // Cap at 100
    }

    async fn track_valid_transaction(&self, transaction: &Transaction) -> Result<()> {
        let tx_id = transaction.id.to_string();
        let mut processed = self.processed_transactions.write().await;
        processed.insert(tx_id);
        Ok(())
    }

    async fn track_validation_start(&self) {
        let mut stats = self.validation_stats.write().await;
        stats.total_validations += 1;
    }

    async fn update_validation_stats(&self, result: &ValidationResult) {
        let mut stats = self.validation_stats.write().await;

        if result.is_valid {
            stats.successful_validations += 1;
        } else {
            stats.failed_validations += 1;
        }

        // Update average validation time
        let total_time = stats.average_validation_time_ms * (stats.total_validations - 1) as f64
            + result.validation_time_ms as f64;
        stats.average_validation_time_ms = total_time / stats.total_validations as f64;

        // Update risk score distribution
        let bucket = (result.risk_score / 10).min(9) as usize;
        stats.risk_score_distribution[bucket] += 1;

        // Count errors
        for error in &result.errors {
            let error_name = format!("{:?}", error);
            *stats.error_counts.entry(error_name).or_insert(0) += 1;
        }
    }

    async fn increment_rule_trigger(&self, rule_id: &str) {
        let mut stats = self.validation_stats.write().await;
        *stats
            .rule_trigger_counts
            .entry(rule_id.to_string())
            .or_insert(0) += 1;
    }

    async fn initialize_default_rules(&mut self) -> Result<()> {
        let now = Utc::now();

        let default_rules = vec![
            ValidationRule {
                id: "signature_required".to_string(),
                name: "Signature Required".to_string(),
                rule_type: ValidationRuleType::Signature,
                enabled: true,
                severity: ValidationSeverity::Critical,
                description: "All transactions must have valid signatures".to_string(),
                parameters: HashMap::new(),
                created_at: now,
                updated_at: now,
            },
            ValidationRule {
                id: "sufficient_balance".to_string(),
                name: "Sufficient Balance".to_string(),
                rule_type: ValidationRuleType::Balance,
                enabled: true,
                severity: ValidationSeverity::Critical,
                description: "Sender must have sufficient balance for transaction + fee"
                    .to_string(),
                parameters: HashMap::new(),
                created_at: now,
                updated_at: now,
            },
            ValidationRule {
                id: "minimum_fee".to_string(),
                name: "Minimum Fee".to_string(),
                rule_type: ValidationRuleType::Balance,
                enabled: true,
                severity: ValidationSeverity::Critical,
                description: "Transaction fee must meet minimum requirements".to_string(),
                parameters: HashMap::new(),
                created_at: now,
                updated_at: now,
            },
            ValidationRule {
                id: "valid_format".to_string(),
                name: "Valid Format".to_string(),
                rule_type: ValidationRuleType::Format,
                enabled: true,
                severity: ValidationSeverity::Critical,
                description: "Transaction must have valid format and structure".to_string(),
                parameters: HashMap::new(),
                created_at: now,
                updated_at: now,
            },
            ValidationRule {
                id: "temporal_validity".to_string(),
                name: "Temporal Validity".to_string(),
                rule_type: ValidationRuleType::Temporal,
                enabled: true,
                severity: ValidationSeverity::Critical,
                description: "Transaction timestamp must be within acceptable range".to_string(),
                parameters: HashMap::new(),
                created_at: now,
                updated_at: now,
            },
            ValidationRule {
                id: "rate_limiting".to_string(),
                name: "Rate Limiting".to_string(),
                rule_type: ValidationRuleType::RateLimit,
                enabled: true,
                severity: ValidationSeverity::Critical,
                description: "Addresses must not exceed transaction rate limits".to_string(),
                parameters: HashMap::new(),
                created_at: now,
                updated_at: now,
            },
            ValidationRule {
                id: "blacklist_check".to_string(),
                name: "Blacklist Check".to_string(),
                rule_type: ValidationRuleType::Policy,
                enabled: true,
                severity: ValidationSeverity::Critical,
                description: "Transactions involving blacklisted addresses are rejected"
                    .to_string(),
                parameters: HashMap::new(),
                created_at: now,
                updated_at: now,
            },
            ValidationRule {
                id: "risk_assessment".to_string(),
                name: "Risk Assessment".to_string(),
                rule_type: ValidationRuleType::Risk,
                enabled: true,
                severity: ValidationSeverity::Warning,
                description: "Calculate and assess transaction risk score".to_string(),
                parameters: HashMap::new(),
                created_at: now,
                updated_at: now,
            },
        ];

        let mut rules = self.rules.write().await;
        for rule in default_rules {
            rules.insert(rule.id.clone(), rule);
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{transaction::Transaction, AddressExt};
    use ed25519_dalek::SigningKey;

    #[tokio::test]
    async fn test_transaction_validator() {
        let config = NetworkValidationConfig::default();
        let validator = TransactionValidator::new(config).await.unwrap();

        // Test with default rules
        let rules = validator.get_validation_rules().await;
        assert!(!rules.is_empty());
        assert!(rules
            .iter()
            .any(|r| r.rule_type == ValidationRuleType::Signature));
    }

    #[tokio::test]
    async fn test_signature_validation() {
        let config = NetworkValidationConfig::default();
        let validator = TransactionValidator::new(config).await.unwrap();

        // Create test transaction
        let keypair = SigningKey::from_bytes(&rand::random());
        let public_key = keypair.verifying_key();
        let address = Address::from_public_key(&public_key);

        let transaction = Transaction {
            id: uuid::Uuid::new_v4(),
            from: address.clone(),
            to: Address::from_public_key(&SigningKey::from_bytes(&rand::random()).verifying_key()),
            amount: 1000_00000000,
            fee: 1_000_000,
            message: None,
            timestamp: Utc::now().timestamp(),
            signature: vec![1, 2, 3], // Invalid signature for testing
        };

        // Should fail due to invalid signature
        let result = validator
            .validate_transaction(&transaction, 2000_00000000, &public_key)
            .await
            .unwrap();
        assert!(!result.is_valid);
        assert!(result.errors.contains(&ValidationError::InvalidSignature));
    }

    #[tokio::test]
    async fn test_balance_validation() {
        let config = NetworkValidationConfig::default();
        let validator = TransactionValidator::new(config).await.unwrap();

        let keypair = SigningKey::from_bytes(&rand::random());
        let public_key = keypair.verifying_key();
        let address = Address::from_public_key(&public_key);

        let transaction = Transaction {
            id: uuid::Uuid::new_v4(),
            from: address.clone(),
            to: Address::from_public_key(&SigningKey::from_bytes(&rand::random()).verifying_key()),
            amount: 1000_00000000,
            fee: 1_000_000,
            message: None,
            timestamp: Utc::now().timestamp(),
            signature: vec![],
        };

        // Should fail due to insufficient balance
        let result = validator
            .validate_transaction(&transaction, 500_00000000, &public_key)
            .await
            .unwrap();
        assert!(!result.is_valid);
        assert!(result
            .errors
            .contains(&ValidationError::InsufficientBalance));
    }
}
