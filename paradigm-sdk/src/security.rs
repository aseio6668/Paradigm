use crate::{Hash, Address, Amount, Transaction, Error, Result};
use serde::{Deserialize, Serialize};
use sha3::{Digest, Sha3_256, Sha3_512};
use std::collections::{HashMap, HashSet, VecDeque};
use std::time::{SystemTime, UNIX_EPOCH, Duration};

/// Security audit result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityAudit {
    pub audit_id: Hash,
    pub timestamp: u64,
    pub severity: SecuritySeverity,
    pub category: SecurityCategory,
    pub description: String,
    pub affected_addresses: Vec<Address>,
    pub recommended_actions: Vec<String>,
}

/// Security severity levels
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum SecuritySeverity {
    Low,
    Medium,
    High,
    Critical,
}

/// Security issue categories
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum SecurityCategory {
    SuspiciousTransaction,
    AbnormalBehavior,
    PotentialAttack,
    ComplianceViolation,
    TechnicalVulnerability,
    PrivacyBreach,
}

/// Transaction anomaly detector
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnomalyDetector {
    pub baseline_patterns: HashMap<Address, TransactionPattern>,
    pub anomaly_threshold: f64,
    pub detection_window: Duration,
    pub max_pattern_history: usize,
}

/// Transaction pattern for anomaly detection
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransactionPattern {
    pub address: Address,
    pub avg_amount: Amount,
    pub avg_frequency: f64, // transactions per hour
    pub common_recipients: HashSet<Address>,
    pub time_patterns: Vec<u8>, // 24-hour pattern
    pub gas_usage_pattern: Vec<u64>,
}

/// Real-time security monitor
#[derive(Debug, Clone)]
pub struct SecurityMonitor {
    anomaly_detector: AnomalyDetector,
    compliance_rules: Vec<ComplianceRule>,
    threat_intelligence: ThreatIntelligence,
    security_alerts: VecDeque<SecurityAudit>,
    max_alerts: usize,
}

/// Compliance rule for regulatory requirements
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComplianceRule {
    pub rule_id: Hash,
    pub name: String,
    pub description: String,
    pub rule_type: ComplianceType,
    pub parameters: HashMap<String, String>,
    pub enabled: bool,
}

/// Compliance rule types
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ComplianceType {
    AML,        // Anti-Money Laundering
    KYC,        // Know Your Customer
    CTF,        // Counter-Terrorism Financing
    OFAC,       // Office of Foreign Assets Control
    GDPR,       // General Data Protection Regulation
    Custom,
}

/// Threat intelligence data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThreatIntelligence {
    pub blacklisted_addresses: HashSet<Address>,
    pub suspicious_patterns: Vec<ThreatPattern>,
    pub known_attack_vectors: Vec<AttackVector>,
    pub updated_at: u64,
}

/// Known threat pattern
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThreatPattern {
    pub pattern_id: Hash,
    pub name: String,
    pub description: String,
    pub indicators: Vec<ThreatIndicator>,
    pub confidence_score: f64,
}

/// Threat indicator
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThreatIndicator {
    pub indicator_type: IndicatorType,
    pub value: String,
    pub weight: f64,
}

/// Types of threat indicators
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum IndicatorType {
    Address,
    AmountRange,
    TransactionFrequency,
    TimePattern,
    GasUsage,
    ContractInteraction,
}

/// Known attack vectors
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AttackVector {
    pub vector_id: Hash,
    pub name: String,
    pub description: String,
    pub mitigation_strategies: Vec<String>,
    pub severity: SecuritySeverity,
}

/// Security scanner for smart contracts
#[derive(Debug, Clone)]
pub struct SecurityScanner {
    vulnerability_db: HashMap<Hash, Vulnerability>,
    scan_rules: Vec<ScanRule>,
}

/// Smart contract vulnerability
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Vulnerability {
    pub vuln_id: Hash,
    pub name: String,
    pub description: String,
    pub severity: SecuritySeverity,
    pub cwe_id: Option<u32>, // Common Weakness Enumeration
    pub fix_recommendations: Vec<String>,
}

/// Security scan rule
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScanRule {
    pub rule_id: Hash,
    pub name: String,
    pub pattern: String,
    pub severity: SecuritySeverity,
    pub description: String,
}

impl SecurityAudit {
    /// Create a new security audit result
    pub fn new(
        severity: SecuritySeverity,
        category: SecurityCategory,
        description: String,
        affected_addresses: Vec<Address>,
    ) -> Self {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or(Duration::from_secs(0))
            .as_secs();
        
        let mut hasher = Sha3_256::new();
        hasher.update(&timestamp.to_be_bytes());
        hasher.update(description.as_bytes());
        for addr in &affected_addresses {
            hasher.update(addr.as_bytes());
        }
        let audit_id = Hash::from_bytes(hasher.finalize().as_slice());
        
        SecurityAudit {
            audit_id,
            timestamp,
            severity,
            category,
            description,
            affected_addresses,
            recommended_actions: Vec::new(),
        }
    }
    
    /// Add recommended action
    pub fn add_recommendation(&mut self, action: String) {
        self.recommended_actions.push(action);
    }
    
    /// Get audit age in seconds
    pub fn age_seconds(&self) -> u64 {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or(Duration::from_secs(0))
            .as_secs();
        
        now.saturating_sub(self.timestamp)
    }
    
    /// Check if audit is expired
    pub fn is_expired(&self, max_age: Duration) -> bool {
        self.age_seconds() > max_age.as_secs()
    }
}

impl AnomalyDetector {
    /// Create new anomaly detector
    pub fn new() -> Self {
        AnomalyDetector {
            baseline_patterns: HashMap::new(),
            anomaly_threshold: 2.5, // Standard deviations from normal
            detection_window: Duration::from_secs(3600), // 1 hour
            max_pattern_history: 1000,
        }
    }
    
    /// Update baseline pattern for an address
    pub fn update_pattern(&mut self, address: Address, transaction: &Transaction) {
        let pattern = self.baseline_patterns.entry(address.clone())
            .or_insert_with(|| TransactionPattern::new(address));
        
        pattern.update_with_transaction(transaction);
        
        // Trim history if needed
        if self.baseline_patterns.len() > self.max_pattern_history {
            let oldest_addr = self.baseline_patterns.keys().next().cloned();
            if let Some(addr) = oldest_addr {
                self.baseline_patterns.remove(&addr);
            }
        }
    }
    
    /// Detect anomalies in a transaction
    pub fn detect_anomalies(&self, transaction: &Transaction) -> Vec<SecurityAudit> {
        let mut audits = Vec::new();
        
        // Check sender pattern
        if let Some(pattern) = self.baseline_patterns.get(&transaction.from) {
            if let Some(audit) = self.check_amount_anomaly(transaction, pattern) {
                audits.push(audit);
            }
            
            if let Some(audit) = self.check_frequency_anomaly(transaction, pattern) {
                audits.push(audit);
            }
            
            if let Some(audit) = self.check_recipient_anomaly(transaction, pattern) {
                audits.push(audit);
            }
        }
        
        audits
    }
    
    fn check_amount_anomaly(&self, transaction: &Transaction, pattern: &TransactionPattern) -> Option<SecurityAudit> {
        let amount_ratio = transaction.value.wei() as f64 / pattern.avg_amount.wei() as f64;
        
        if amount_ratio > self.anomaly_threshold || amount_ratio < (1.0 / self.anomaly_threshold) {
            let mut audit = SecurityAudit::new(
                SecuritySeverity::Medium,
                SecurityCategory::AbnormalBehavior,
                format!("Unusual transaction amount detected: {}x normal pattern", amount_ratio),
                vec![transaction.from.clone()],
            );
            audit.add_recommendation("Monitor this address for additional suspicious activity".to_string());
            Some(audit)
        } else {
            None
        }
    }
    
    fn check_frequency_anomaly(&self, _transaction: &Transaction, _pattern: &TransactionPattern) -> Option<SecurityAudit> {
        // Mock frequency check - in reality would track transaction timing
        None
    }
    
    fn check_recipient_anomaly(&self, transaction: &Transaction, pattern: &TransactionPattern) -> Option<SecurityAudit> {
        if !pattern.common_recipients.contains(&transaction.to) && pattern.common_recipients.len() > 0 {
            let mut audit = SecurityAudit::new(
                SecuritySeverity::Low,
                SecurityCategory::AbnormalBehavior,
                "Transaction to unusual recipient address".to_string(),
                vec![transaction.from, transaction.to],
            );
            audit.add_recommendation("Verify recipient address legitimacy".to_string());
            Some(audit)
        } else {
            None
        }
    }
}

impl TransactionPattern {
    /// Create new transaction pattern
    pub fn new(address: Address) -> Self {
        TransactionPattern {
            address,
            avg_amount: Amount::zero(),
            avg_frequency: 0.0,
            common_recipients: HashSet::new(),
            time_patterns: vec![0u8; 24], // 24-hour pattern
            gas_usage_pattern: Vec::new(),
        }
    }
    
    /// Update pattern with new transaction
    pub fn update_with_transaction(&mut self, transaction: &Transaction) {
        // Update average amount (simple moving average)
        let current_wei = self.avg_amount.wei();
        let new_wei = transaction.value.wei();
        let updated_wei = (current_wei + new_wei) / 2;
        self.avg_amount = Amount::from_wei(updated_wei);
        
        // Add recipient to common recipients (limit to top 10)
        if let Some(to_addr) = &transaction.to {
            self.common_recipients.insert(to_addr.clone());
        }
        if self.common_recipients.len() > 10 {
            // Remove random recipient to maintain size
            let to_remove = self.common_recipients.iter().next().cloned();
            if let Some(addr) = to_remove {
                self.common_recipients.remove(&addr);
            }
        }
        
        // Update gas usage pattern
        self.gas_usage_pattern.push(transaction.gas);
        if self.gas_usage_pattern.len() > 100 {
            self.gas_usage_pattern.remove(0);
        }
        
        // Update time pattern (mock - would use actual timestamp)
        let hour = (SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or(Duration::from_secs(0))
            .as_secs() / 3600) % 24;
        
        if hour < 24 {
            self.time_patterns[hour as usize] = self.time_patterns[hour as usize].saturating_add(1);
        }
    }
}

impl SecurityMonitor {
    /// Create new security monitor
    pub fn new() -> Self {
        SecurityMonitor {
            anomaly_detector: AnomalyDetector::new(),
            compliance_rules: Vec::new(),
            threat_intelligence: ThreatIntelligence::new(),
            security_alerts: VecDeque::new(),
            max_alerts: 1000,
        }
    }
    
    /// Monitor a transaction for security issues
    pub fn monitor_transaction(&mut self, transaction: &Transaction) -> Vec<SecurityAudit> {
        let mut audits = Vec::new();
        
        // Check for anomalies
        let mut anomaly_audits = self.anomaly_detector.detect_anomalies(transaction);
        audits.append(&mut anomaly_audits);
        
        // Check compliance rules
        let mut compliance_audits = self.check_compliance(transaction);
        audits.append(&mut compliance_audits);
        
        // Check threat intelligence
        let mut threat_audits = self.check_threats(transaction);
        audits.append(&mut threat_audits);
        
        // Store alerts
        for audit in &audits {
            self.add_alert(audit.clone());
        }
        
        // Update patterns
        self.anomaly_detector.update_pattern(transaction.from.clone(), transaction);
        
        audits
    }
    
    /// Add security alert
    pub fn add_alert(&mut self, alert: SecurityAudit) {
        self.security_alerts.push_back(alert);
        
        // Trim old alerts
        while self.security_alerts.len() > self.max_alerts {
            self.security_alerts.pop_front();
        }
    }
    
    /// Get recent alerts
    pub fn get_recent_alerts(&self, limit: usize) -> Vec<&SecurityAudit> {
        self.security_alerts.iter().rev().take(limit).collect()
    }
    
    /// Add compliance rule
    pub fn add_compliance_rule(&mut self, rule: ComplianceRule) {
        self.compliance_rules.push(rule);
    }
    
    /// Check compliance rules
    fn check_compliance(&self, transaction: &Transaction) -> Vec<SecurityAudit> {
        let mut audits = Vec::new();
        
        for rule in &self.compliance_rules {
            if !rule.enabled {
                continue;
            }
            
            match rule.rule_type {
                ComplianceType::AML => {
                    if let Some(audit) = self.check_aml_rule(transaction, rule) {
                        audits.push(audit);
                    }
                }
                ComplianceType::OFAC => {
                    if let Some(audit) = self.check_ofac_rule(transaction, rule) {
                        audits.push(audit);
                    }
                }
                _ => {} // Other compliance types
            }
        }
        
        audits
    }
    
    fn check_aml_rule(&self, transaction: &Transaction, rule: &ComplianceRule) -> Option<SecurityAudit> {
        // Check for large transactions that might indicate money laundering
        if let Some(threshold_str) = rule.parameters.get("amount_threshold") {
            if let Ok(threshold_wei) = threshold_str.parse::<u64>() {
                if transaction.value.wei() > threshold_wei {
                    let mut audit = SecurityAudit::new(
                        SecuritySeverity::High,
                        SecurityCategory::ComplianceViolation,
                        format!("Large transaction exceeding AML threshold: {} wei", transaction.value.wei()),
                        {
                            let mut addresses = vec![transaction.from.clone()];
                            if let Some(to_addr) = &transaction.to {
                                addresses.push(to_addr.clone());
                            }
                            addresses
                        },
                    );
                    audit.add_recommendation("Conduct enhanced due diligence on transaction parties".to_string());
                    audit.add_recommendation("File suspicious activity report if required".to_string());
                    return Some(audit);
                }
            }
        }
        
        None
    }
    
    fn check_ofac_rule(&self, transaction: &Transaction, _rule: &ComplianceRule) -> Option<SecurityAudit> {
        // Check against OFAC sanctions list
        let from_blacklisted = self.threat_intelligence.blacklisted_addresses.contains(&transaction.from);
        let to_blacklisted = transaction.to
            .as_ref()
            .map(|addr| self.threat_intelligence.blacklisted_addresses.contains(addr))
            .unwrap_or(false);
        
        if from_blacklisted || to_blacklisted {
            let mut addresses = vec![transaction.from.clone()];
            if let Some(to_addr) = &transaction.to {
                addresses.push(to_addr.clone());
            }
            let mut audit = SecurityAudit::new(
                SecuritySeverity::Critical,
                SecurityCategory::ComplianceViolation,
                "Transaction involving OFAC sanctioned address".to_string(),
                addresses,
            );
            audit.add_recommendation("Block transaction immediately".to_string());
            audit.add_recommendation("Report to regulatory authorities".to_string());
            Some(audit)
        } else {
            None
        }
    }
    
    fn check_threats(&self, transaction: &Transaction) -> Vec<SecurityAudit> {
        let mut audits = Vec::new();
        
        // Check blacklisted addresses
        let from_blacklisted = self.threat_intelligence.blacklisted_addresses.contains(&transaction.from);
        let to_blacklisted = transaction.to
            .as_ref()
            .map(|addr| self.threat_intelligence.blacklisted_addresses.contains(addr))
            .unwrap_or(false);
            
        if from_blacklisted || to_blacklisted {
            let mut addresses = vec![transaction.from.clone()];
            if let Some(to_addr) = &transaction.to {
                addresses.push(to_addr.clone());
            }
            let mut audit = SecurityAudit::new(
                SecuritySeverity::High,
                SecurityCategory::PotentialAttack,
                "Transaction involving blacklisted address".to_string(),
                addresses,
            );
            audit.add_recommendation("Block transaction".to_string());
            audit.add_recommendation("Investigate address activity".to_string());
            audits.push(audit);
        }
        
        // Check threat patterns
        for pattern in &self.threat_intelligence.suspicious_patterns {
            if self.matches_threat_pattern(transaction, pattern) {
                let mut addresses = vec![transaction.from.clone()];
                if let Some(to_addr) = &transaction.to {
                    addresses.push(to_addr.clone());
                }
                let mut audit = SecurityAudit::new(
                    SecuritySeverity::Medium,
                    SecurityCategory::PotentialAttack,
                    format!("Transaction matches threat pattern: {}", pattern.name),
                    addresses,
                );
                audit.add_recommendation("Enhanced monitoring recommended".to_string());
                audits.push(audit);
            }
        }
        
        audits
    }
    
    fn matches_threat_pattern(&self, transaction: &Transaction, pattern: &ThreatPattern) -> bool {
        let mut score = 0.0;
        
        for indicator in &pattern.indicators {
            match indicator.indicator_type {
                IndicatorType::Address => {
                    let from_match = transaction.from.to_string() == indicator.value;
                    let to_match = transaction.to
                        .as_ref()
                        .map(|addr| addr.to_string() == indicator.value)
                        .unwrap_or(false);
                    
                    if from_match || to_match {
                        score += indicator.weight;
                    }
                }
                IndicatorType::AmountRange => {
                    // Parse amount range and check if transaction amount falls within it
                    if indicator.value.contains('-') {
                        let parts: Vec<&str> = indicator.value.split('-').collect();
                        if parts.len() == 2 {
                            if let (Ok(min), Ok(max)) = (parts[0].parse::<u64>(), parts[1].parse::<u64>()) {
                                let amount = transaction.value.wei();
                                if amount >= min && amount <= max {
                                    score += indicator.weight;
                                }
                            }
                        }
                    }
                }
                IndicatorType::GasUsage => {
                    if let Ok(gas_threshold) = indicator.value.parse::<u64>() {
                        if transaction.gas > gas_threshold {
                            score += indicator.weight;
                        }
                    }
                }
                _ => {} // Other indicator types
            }
        }
        
        score >= pattern.confidence_score
    }
}

impl ThreatIntelligence {
    /// Create new threat intelligence
    pub fn new() -> Self {
        ThreatIntelligence {
            blacklisted_addresses: HashSet::new(),
            suspicious_patterns: Vec::new(),
            known_attack_vectors: Vec::new(),
            updated_at: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or(Duration::from_secs(0))
                .as_secs(),
        }
    }
    
    /// Add blacklisted address
    pub fn add_blacklisted_address(&mut self, address: Address) {
        self.blacklisted_addresses.insert(address);
        self.update_timestamp();
    }
    
    /// Remove blacklisted address
    pub fn remove_blacklisted_address(&mut self, address: &Address) {
        self.blacklisted_addresses.remove(address);
        self.update_timestamp();
    }
    
    /// Add threat pattern
    pub fn add_threat_pattern(&mut self, pattern: ThreatPattern) {
        self.suspicious_patterns.push(pattern);
        self.update_timestamp();
    }
    
    /// Add attack vector
    pub fn add_attack_vector(&mut self, vector: AttackVector) {
        self.known_attack_vectors.push(vector);
        self.update_timestamp();
    }
    
    fn update_timestamp(&mut self) {
        self.updated_at = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or(Duration::from_secs(0))
            .as_secs();
    }
}

impl Default for AnomalyDetector {
    fn default() -> Self {
        Self::new()
    }
}

impl Default for SecurityMonitor {
    fn default() -> Self {
        Self::new()
    }
}

impl Default for ThreatIntelligence {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_security_audit() {
        let audit = SecurityAudit::new(
            SecuritySeverity::High,
            SecurityCategory::SuspiciousTransaction,
            "Test audit".to_string(),
            vec![Address::from_hex("1111111111111111111111111111111111111111").unwrap()],
        );
        
        assert_eq!(audit.severity, SecuritySeverity::High);
        assert_eq!(audit.category, SecurityCategory::SuspiciousTransaction);
        assert!(audit.age_seconds() < 5); // Should be very recent
    }
    
    #[test]
    fn test_anomaly_detector() {
        let mut detector = AnomalyDetector::new();
        let from = Address::from_hex("1111111111111111111111111111111111111111").unwrap();
        let to = Address::from_hex("2222222222222222222222222222222222222222").unwrap();
        
        let normal_tx = Transaction {
            hash: Hash::default(),
            from,
            to,
            amount: Amount::from_paradigm(100.0),
            gas: 21000,
            gas_price: Amount::from_paradigm(0.00001),
            nonce: 1,
            block_hash: Some(Hash::default()),
            block_number: Some(1),
            transaction_index: Some(0),
            input: vec![],
        };
        
        // Train with normal transaction
        detector.update_pattern(from, &normal_tx);
        
        let anomalous_tx = Transaction {
            hash: Hash::default(),
            from,
            to,
            amount: Amount::from_paradigm(10000.0), // 100x normal
            gas: 21000,
            gas_price: Amount::from_paradigm(0.00001),
            nonce: 2,
            block_hash: Some(Hash::default()),
            block_number: Some(2),
            transaction_index: Some(0),
            input: vec![],
        };
        
        let anomalies = detector.detect_anomalies(&anomalous_tx);
        assert!(!anomalies.is_empty());
    }
    
    #[test]
    fn test_threat_intelligence() {
        let mut threat_intel = ThreatIntelligence::new();
        let bad_address = Address::from_hex("deadbeefdeadbeefdeadbeefdeadbeefdeadbeef").unwrap();
        
        threat_intel.add_blacklisted_address(bad_address);
        assert!(threat_intel.blacklisted_addresses.contains(&bad_address));
        
        threat_intel.remove_blacklisted_address(&bad_address);
        assert!(!threat_intel.blacklisted_addresses.contains(&bad_address));
    }
    
    #[test]
    fn test_security_monitor() {
        let mut monitor = SecurityMonitor::new();
        
        // Add AML compliance rule
        let aml_rule = ComplianceRule {
            rule_id: Hash::default(),
            name: "Large Transaction AML".to_string(),
            description: "Flag large transactions for AML review".to_string(),
            rule_type: ComplianceType::AML,
            parameters: [("amount_threshold".to_string(), "1000000000000000000".to_string())].into(),
            enabled: true,
        };
        monitor.add_compliance_rule(aml_rule);
        
        let large_tx = Transaction {
            hash: Hash::default(),
            from: Address::from_hex("1111111111111111111111111111111111111111").unwrap(),
            to: Address::from_hex("2222222222222222222222222222222222222222").unwrap(),
            amount: Amount::from_paradigm(2.0), // 2 ETH = 2e18 wei
            gas: 21000,
            gas_price: Amount::from_paradigm(0.00001),
            nonce: 1,
            block_hash: Some(Hash::default()),
            block_number: Some(1),
            transaction_index: Some(0),
            input: vec![],
        };
        
        let audits = monitor.monitor_transaction(&large_tx);
        assert!(!audits.is_empty());
        
        let recent_alerts = monitor.get_recent_alerts(10);
        assert!(!recent_alerts.is_empty());
    }
}