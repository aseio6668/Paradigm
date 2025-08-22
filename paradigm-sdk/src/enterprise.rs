//! Enterprise integration capabilities
//! 
//! This module provides enterprise-grade features including API management,
//! compliance tools, monitoring, and enterprise wallet management.

use crate::types::*;
use crate::error::{Result, ParadigmError, ErrorExt};
use crate::client::ParadigmClient;
use crate::wallet::{Wallet, WalletManager};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, SystemTime};
use tokio::sync::RwLock;
use uuid::Uuid;

/// Enterprise API management
#[derive(Debug)]
pub struct EnterpriseApiManager {
    /// API key management
    api_keys: Arc<RwLock<HashMap<String, ApiKey>>>,
    /// Rate limiting per API key
    rate_limiters: Arc<RwLock<HashMap<String, RateLimiter>>>,
    /// Usage analytics
    analytics: Arc<RwLock<ApiAnalytics>>,
    /// Configuration
    config: EnterpriseConfig,
}

/// API key information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiKey {
    /// Unique key ID
    pub id: Uuid,
    /// API key string
    pub key: String,
    /// Associated organization
    pub organization: String,
    /// Key permissions
    pub permissions: Vec<ApiPermission>,
    /// Rate limit tier
    pub rate_limit_tier: RateLimitTier,
    /// Creation time
    pub created_at: SystemTime,
    /// Expiration time
    pub expires_at: Option<SystemTime>,
    /// Is active
    pub active: bool,
    /// Usage statistics
    pub usage_stats: ApiUsageStats,
}

/// API permissions
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum ApiPermission {
    ReadBlocks,
    ReadTransactions,
    ReadAccounts,
    SubmitTransactions,
    ManageWallets,
    AccessAnalytics,
    AdminAccess,
}

/// Rate limit tiers
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum RateLimitTier {
    Basic { requests_per_minute: u32 },
    Professional { requests_per_minute: u32 },
    Enterprise { requests_per_minute: u32 },
    Unlimited,
}

impl RateLimitTier {
    pub fn requests_per_minute(&self) -> Option<u32> {
        match self {
            RateLimitTier::Basic { requests_per_minute } => Some(*requests_per_minute),
            RateLimitTier::Professional { requests_per_minute } => Some(*requests_per_minute),
            RateLimitTier::Enterprise { requests_per_minute } => Some(*requests_per_minute),
            RateLimitTier::Unlimited => None,
        }
    }
}

/// API usage statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiUsageStats {
    /// Total requests made
    pub total_requests: u64,
    /// Requests in current period
    pub current_period_requests: u64,
    /// Last request time
    pub last_request_at: Option<SystemTime>,
    /// Error count
    pub error_count: u64,
    /// Rate limit violations
    pub rate_limit_violations: u64,
}

impl Default for ApiUsageStats {
    fn default() -> Self {
        Self {
            total_requests: 0,
            current_period_requests: 0,
            last_request_at: None,
            error_count: 0,
            rate_limit_violations: 0,
        }
    }
}

/// Rate limiter implementation
#[derive(Debug)]
pub struct RateLimiter {
    tier: RateLimitTier,
    window_start: SystemTime,
    requests_in_window: u32,
}

impl RateLimiter {
    pub fn new(tier: RateLimitTier) -> Self {
        Self {
            tier,
            window_start: SystemTime::now(),
            requests_in_window: 0,
        }
    }
    
    pub fn allow_request(&mut self) -> bool {
        let now = SystemTime::now();
        
        // Reset window if a minute has passed
        if now.duration_since(self.window_start).unwrap_or_default() >= Duration::from_secs(60) {
            self.window_start = now;
            self.requests_in_window = 0;
        }
        
        match self.tier.requests_per_minute() {
            Some(limit) => {
                if self.requests_in_window < limit {
                    self.requests_in_window += 1;
                    true
                } else {
                    false
                }
            }
            None => true, // Unlimited
        }
    }
}

/// API analytics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiAnalytics {
    /// Total API calls across all keys
    pub total_calls: u64,
    /// Calls by endpoint
    pub calls_by_endpoint: HashMap<String, u64>,
    /// Calls by organization
    pub calls_by_organization: HashMap<String, u64>,
    /// Error rates
    pub error_rates: HashMap<String, f64>,
    /// Response times
    pub average_response_times: HashMap<String, u64>,
    /// Peak usage times
    pub peak_usage_stats: PeakUsageStats,
}

impl Default for ApiAnalytics {
    fn default() -> Self {
        Self {
            total_calls: 0,
            calls_by_endpoint: HashMap::new(),
            calls_by_organization: HashMap::new(),
            error_rates: HashMap::new(),
            average_response_times: HashMap::new(),
            peak_usage_stats: PeakUsageStats::default(),
        }
    }
}

/// Peak usage statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PeakUsageStats {
    /// Peak requests per minute
    pub peak_rpm: u32,
    /// When peak occurred
    pub peak_time: Option<SystemTime>,
    /// Peak concurrent users
    pub peak_concurrent_users: u32,
    /// Busiest hour of day (0-23)
    pub busiest_hour: Option<u8>,
}

impl Default for PeakUsageStats {
    fn default() -> Self {
        Self {
            peak_rpm: 0,
            peak_time: None,
            peak_concurrent_users: 0,
            busiest_hour: None,
        }
    }
}

/// Enterprise configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnterpriseConfig {
    /// Default rate limit tier for new keys
    pub default_rate_limit: RateLimitTier,
    /// Enable audit logging
    pub enable_audit_logging: bool,
    /// Audit log retention period
    pub audit_retention_days: u32,
    /// Enable compliance features
    pub enable_compliance: bool,
    /// KYC requirements
    pub kyc_required: bool,
    /// AML monitoring
    pub enable_aml_monitoring: bool,
    /// Multi-signature requirements
    pub multisig_config: MultisigConfig,
    /// Backup and recovery settings
    pub backup_config: BackupConfig,
}

impl Default for EnterpriseConfig {
    fn default() -> Self {
        Self {
            default_rate_limit: RateLimitTier::Professional { requests_per_minute: 1000 },
            enable_audit_logging: true,
            audit_retention_days: 365,
            enable_compliance: true,
            kyc_required: false,
            enable_aml_monitoring: true,
            multisig_config: MultisigConfig::default(),
            backup_config: BackupConfig::default(),
        }
    }
}

/// Multi-signature configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MultisigConfig {
    /// Require multisig for transactions above threshold
    pub threshold_amount: Amount,
    /// Minimum number of signatures required
    pub min_signatures: u32,
    /// Maximum number of signers
    pub max_signers: u32,
    /// Timeout for signature collection
    pub signature_timeout_hours: u32,
}

impl Default for MultisigConfig {
    fn default() -> Self {
        Self {
            threshold_amount: Amount::from_paradigm(10000.0),
            min_signatures: 2,
            max_signers: 10,
            signature_timeout_hours: 24,
        }
    }
}

/// Backup configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackupConfig {
    /// Enable automatic backups
    pub auto_backup: bool,
    /// Backup interval in hours
    pub backup_interval_hours: u32,
    /// Number of backups to retain
    pub backup_retention_count: u32,
    /// Backup encryption enabled
    pub encrypt_backups: bool,
    /// Remote backup destinations
    pub remote_destinations: Vec<BackupDestination>,
}

impl Default for BackupConfig {
    fn default() -> Self {
        Self {
            auto_backup: true,
            backup_interval_hours: 24,
            backup_retention_count: 30,
            encrypt_backups: true,
            remote_destinations: vec![],
        }
    }
}

/// Backup destination
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackupDestination {
    /// Destination type
    pub destination_type: BackupDestinationType,
    /// Destination URL or path
    pub location: String,
    /// Authentication credentials
    pub credentials: Option<String>,
    /// Encryption key
    pub encryption_key: Option<String>,
}

/// Backup destination types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BackupDestinationType {
    LocalFilesystem,
    S3Compatible,
    SFTP,
    WebDAV,
}

impl EnterpriseApiManager {
    /// Create new enterprise API manager
    pub fn new(config: EnterpriseConfig) -> Self {
        Self {
            api_keys: Arc::new(RwLock::new(HashMap::new())),
            rate_limiters: Arc::new(RwLock::new(HashMap::new())),
            analytics: Arc::new(RwLock::new(ApiAnalytics::default())),
            config,
        }
    }
    
    /// Create new API key
    pub async fn create_api_key(
        &self,
        organization: String,
        permissions: Vec<ApiPermission>,
        rate_limit_tier: Option<RateLimitTier>,
    ) -> Result<ApiKey> {
        let api_key = ApiKey {
            id: Uuid::new_v4(),
            key: self.generate_api_key(),
            organization,
            permissions,
            rate_limit_tier: rate_limit_tier.unwrap_or(self.config.default_rate_limit.clone()),
            created_at: SystemTime::now(),
            expires_at: None,
            active: true,
            usage_stats: ApiUsageStats::default(),
        };
        
        // Store API key
        self.api_keys.write().await.insert(api_key.key.clone(), api_key.clone());
        
        // Create rate limiter
        self.rate_limiters.write().await.insert(
            api_key.key.clone(),
            RateLimiter::new(api_key.rate_limit_tier.clone()),
        );
        
        Ok(api_key)
    }
    
    /// Validate API key and check permissions
    pub async fn validate_api_key(&self, key: &str, required_permission: ApiPermission) -> Result<bool> {
        let api_keys = self.api_keys.read().await;
        
        if let Some(api_key) = api_keys.get(key) {
            if !api_key.active {
                return Ok(false);
            }
            
            if let Some(expires_at) = api_key.expires_at {
                if SystemTime::now() > expires_at {
                    return Ok(false);
                }
            }
            
            Ok(api_key.permissions.contains(&required_permission) || 
               api_key.permissions.contains(&ApiPermission::AdminAccess))
        } else {
            Ok(false)
        }
    }
    
    /// Check rate limit for API key
    pub async fn check_rate_limit(&self, key: &str) -> Result<bool> {
        let mut rate_limiters = self.rate_limiters.write().await;
        
        if let Some(rate_limiter) = rate_limiters.get_mut(key) {
            if rate_limiter.allow_request() {
                // Update usage stats
                self.update_usage_stats(key, false).await;
                Ok(true)
            } else {
                // Update rate limit violation
                self.update_usage_stats(key, true).await;
                Ok(false)
            }
        } else {
            Err(ParadigmError::Authentication("Invalid API key".to_string()))
        }
    }
    
    /// Update usage statistics
    async fn update_usage_stats(&self, key: &str, rate_limited: bool) {
        let mut api_keys = self.api_keys.write().await;
        if let Some(api_key) = api_keys.get_mut(key) {
            api_key.usage_stats.total_requests += 1;
            api_key.usage_stats.current_period_requests += 1;
            api_key.usage_stats.last_request_at = Some(SystemTime::now());
            
            if rate_limited {
                api_key.usage_stats.rate_limit_violations += 1;
            }
        }
        
        // Update global analytics
        let mut analytics = self.analytics.write().await;
        analytics.total_calls += 1;
    }
    
    /// Generate secure API key
    fn generate_api_key(&self) -> String {
        use rand::RngCore;
        let mut bytes = [0u8; 32];
        rand::rngs::OsRng.fill_bytes(&mut bytes);
        format!("pk_{}", hex::encode(bytes))
    }
    
    /// Get API analytics
    pub async fn get_analytics(&self) -> ApiAnalytics {
        self.analytics.read().await.clone()
    }
    
    /// List API keys for organization
    pub async fn list_api_keys(&self, organization: &str) -> Vec<ApiKey> {
        self.api_keys.read().await
            .values()
            .filter(|key| key.organization == organization)
            .cloned()
            .collect()
    }
    
    /// Revoke API key
    pub async fn revoke_api_key(&self, key: &str) -> Result<()> {
        let mut api_keys = self.api_keys.write().await;
        if let Some(api_key) = api_keys.get_mut(key) {
            api_key.active = false;
            Ok(())
        } else {
            Err(ParadigmError::NotFound("API key not found".to_string()))
        }
    }
}

/// Compliance monitoring system
#[derive(Debug)]
pub struct ComplianceMonitor {
    /// Compliance rules
    rules: Arc<RwLock<Vec<ComplianceRule>>>,
    /// Transaction alerts
    alerts: Arc<RwLock<Vec<ComplianceAlert>>>,
    /// Watchlist addresses
    watchlist: Arc<RwLock<HashMap<Address, WatchlistEntry>>>,
    /// Configuration
    config: ComplianceConfig,
}

/// Compliance rule
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComplianceRule {
    /// Rule ID
    pub id: Uuid,
    /// Rule name
    pub name: String,
    /// Rule description
    pub description: String,
    /// Rule type
    pub rule_type: ComplianceRuleType,
    /// Is active
    pub active: bool,
    /// Severity level
    pub severity: ComplianceSeverity,
    /// Actions to take when triggered
    pub actions: Vec<ComplianceAction>,
}

/// Compliance rule types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ComplianceRuleType {
    /// Large transaction threshold
    LargeTransaction { threshold: Amount },
    /// Rapid succession of transactions
    RapidTransactions { count: u32, time_window_minutes: u32 },
    /// Unusual transaction patterns
    UnusualPattern { pattern_type: String },
    /// Transactions with watchlist addresses
    WatchlistInteraction,
    /// Cross-border transactions
    CrossBorderTransaction,
    /// High-risk jurisdiction
    HighRiskJurisdiction { jurisdictions: Vec<String> },
}

/// Compliance severity levels
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub enum ComplianceSeverity {
    Low,
    Medium,
    High,
    Critical,
}

/// Compliance actions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ComplianceAction {
    /// Log the event
    Log,
    /// Send alert to compliance team
    Alert,
    /// Temporarily freeze account
    FreezeAccount { duration_hours: u32 },
    /// Require manual review
    RequireManualReview,
    /// Block transaction
    BlockTransaction,
    /// Report to authorities
    ReportToAuthorities,
}

/// Compliance alert
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComplianceAlert {
    /// Alert ID
    pub id: Uuid,
    /// Associated rule
    pub rule_id: Uuid,
    /// Transaction that triggered alert
    pub transaction_hash: Option<Hash>,
    /// Account involved
    pub account: Address,
    /// Alert severity
    pub severity: ComplianceSeverity,
    /// Alert message
    pub message: String,
    /// When alert was created
    pub created_at: SystemTime,
    /// Alert status
    pub status: AlertStatus,
    /// Assigned investigator
    pub assigned_to: Option<String>,
    /// Resolution notes
    pub resolution_notes: Option<String>,
}

/// Alert status
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum AlertStatus {
    Open,
    InProgress,
    Resolved,
    FalsePositive,
    Escalated,
}

/// Watchlist entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WatchlistEntry {
    /// Address on watchlist
    pub address: Address,
    /// Reason for watchlisting
    pub reason: String,
    /// Risk level
    pub risk_level: RiskLevel,
    /// Added by
    pub added_by: String,
    /// When added
    pub added_at: SystemTime,
    /// Additional metadata
    pub metadata: HashMap<String, String>,
}

/// Risk levels
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub enum RiskLevel {
    Low,
    Medium,
    High,
    Prohibited,
}

/// Compliance configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComplianceConfig {
    /// Enable real-time monitoring
    pub realtime_monitoring: bool,
    /// Enable automated responses
    pub automated_responses: bool,
    /// Reporting requirements
    pub reporting_config: ReportingConfig,
    /// Data retention policies
    pub retention_config: RetentionConfig,
}

impl Default for ComplianceConfig {
    fn default() -> Self {
        Self {
            realtime_monitoring: true,
            automated_responses: true,
            reporting_config: ReportingConfig::default(),
            retention_config: RetentionConfig::default(),
        }
    }
}

/// Reporting configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReportingConfig {
    /// Generate daily reports
    pub daily_reports: bool,
    /// Generate weekly reports
    pub weekly_reports: bool,
    /// Generate monthly reports
    pub monthly_reports: bool,
    /// Report recipients
    pub recipients: Vec<String>,
    /// Report format
    pub format: ReportFormat,
}

impl Default for ReportingConfig {
    fn default() -> Self {
        Self {
            daily_reports: true,
            weekly_reports: true,
            monthly_reports: true,
            recipients: vec![],
            format: ReportFormat::PDF,
        }
    }
}

/// Report formats
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ReportFormat {
    PDF,
    CSV,
    JSON,
    XML,
}

/// Data retention configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetentionConfig {
    /// Transaction data retention in days
    pub transaction_retention_days: u32,
    /// Alert data retention in days
    pub alert_retention_days: u32,
    /// Audit log retention in days
    pub audit_retention_days: u32,
    /// Archive old data instead of deleting
    pub archive_instead_of_delete: bool,
}

impl Default for RetentionConfig {
    fn default() -> Self {
        Self {
            transaction_retention_days: 2555, // 7 years
            alert_retention_days: 1825,       // 5 years
            audit_retention_days: 2555,       // 7 years
            archive_instead_of_delete: true,
        }
    }
}

impl ComplianceMonitor {
    /// Create new compliance monitor
    pub fn new(config: ComplianceConfig) -> Self {
        Self {
            rules: Arc::new(RwLock::new(Vec::new())),
            alerts: Arc::new(RwLock::new(Vec::new())),
            watchlist: Arc::new(RwLock::new(HashMap::new())),
            config,
        }
    }
    
    /// Add compliance rule
    pub async fn add_rule(&self, rule: ComplianceRule) {
        self.rules.write().await.push(rule);
    }
    
    /// Monitor transaction for compliance
    pub async fn monitor_transaction(&self, transaction: &Transaction) -> Result<Vec<ComplianceAlert>> {
        let rules = self.rules.read().await;
        let mut triggered_alerts = Vec::new();
        
        for rule in rules.iter() {
            if !rule.active {
                continue;
            }
            
            if self.check_rule_against_transaction(rule, transaction).await? {
                let alert = ComplianceAlert {
                    id: Uuid::new_v4(),
                    rule_id: rule.id,
                    transaction_hash: Some(transaction.hash.clone()),
                    account: transaction.from.clone(),
                    severity: rule.severity.clone(),
                    message: format!("Rule '{}' triggered by transaction", rule.name),
                    created_at: SystemTime::now(),
                    status: AlertStatus::Open,
                    assigned_to: None,
                    resolution_notes: None,
                };
                
                triggered_alerts.push(alert.clone());
                self.alerts.write().await.push(alert);
                
                // Execute compliance actions
                self.execute_compliance_actions(&rule.actions, transaction).await?;
            }
        }
        
        Ok(triggered_alerts)
    }
    
    /// Check if rule applies to transaction
    async fn check_rule_against_transaction(&self, rule: &ComplianceRule, transaction: &Transaction) -> Result<bool> {
        match &rule.rule_type {
            ComplianceRuleType::LargeTransaction { threshold } => {
                Ok(transaction.value >= *threshold)
            }
            ComplianceRuleType::WatchlistInteraction => {
                let watchlist = self.watchlist.read().await;
                Ok(watchlist.contains_key(&transaction.from) || 
                   transaction.to.as_ref().map_or(false, |to| watchlist.contains_key(to)))
            }
            ComplianceRuleType::RapidTransactions { count: _, time_window_minutes: _ } => {
                // Would need access to transaction history to implement
                Ok(false)
            }
            ComplianceRuleType::UnusualPattern { pattern_type: _ } => {
                // Would need ML models to detect patterns
                Ok(false)
            }
            ComplianceRuleType::CrossBorderTransaction => {
                // Would need geolocation data
                Ok(false)
            }
            ComplianceRuleType::HighRiskJurisdiction { jurisdictions: _ } => {
                // Would need jurisdiction mapping
                Ok(false)
            }
        }
    }
    
    /// Execute compliance actions
    async fn execute_compliance_actions(&self, actions: &[ComplianceAction], _transaction: &Transaction) -> Result<()> {
        for action in actions {
            match action {
                ComplianceAction::Log => {
                    tracing::info!("Compliance rule triggered for transaction");
                }
                ComplianceAction::Alert => {
                    tracing::warn!("Compliance alert generated");
                }
                ComplianceAction::FreezeAccount { duration_hours: _ } => {
                    tracing::warn!("Account freeze action would be executed");
                }
                ComplianceAction::RequireManualReview => {
                    tracing::info!("Manual review required");
                }
                ComplianceAction::BlockTransaction => {
                    tracing::warn!("Transaction would be blocked");
                }
                ComplianceAction::ReportToAuthorities => {
                    tracing::warn!("Authorities would be notified");
                }
            }
        }
        Ok(())
    }
    
    /// Add address to watchlist
    pub async fn add_to_watchlist(&self, entry: WatchlistEntry) {
        self.watchlist.write().await.insert(entry.address.clone(), entry);
    }
    
    /// Remove address from watchlist
    pub async fn remove_from_watchlist(&self, address: &Address) -> bool {
        self.watchlist.write().await.remove(address).is_some()
    }
    
    /// Get all alerts
    pub async fn get_alerts(&self, status_filter: Option<AlertStatus>) -> Vec<ComplianceAlert> {
        let alerts = self.alerts.read().await;
        match status_filter {
            Some(status) => alerts.iter().filter(|alert| alert.status == status).cloned().collect(),
            None => alerts.clone(),
        }
    }
    
    /// Update alert status
    pub async fn update_alert_status(&self, alert_id: Uuid, status: AlertStatus, assigned_to: Option<String>) -> Result<()> {
        let mut alerts = self.alerts.write().await;
        if let Some(alert) = alerts.iter_mut().find(|a| a.id == alert_id) {
            alert.status = status;
            alert.assigned_to = assigned_to;
            Ok(())
        } else {
            Err(ParadigmError::NotFound("Alert not found".to_string()))
        }
    }
}

/// Enterprise wallet management
#[derive(Debug)]
pub struct EnterpriseWalletManager {
    /// Wallet manager
    wallet_manager: WalletManager,
    /// Multisig configurations
    multisig_configs: Arc<RwLock<HashMap<Uuid, MultisigWallet>>>,
    /// Approval workflows
    approval_workflows: Arc<RwLock<HashMap<Uuid, ApprovalWorkflow>>>,
    /// Configuration
    config: EnterpriseConfig,
}

/// Multisig wallet
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MultisigWallet {
    /// Wallet ID
    pub id: Uuid,
    /// Wallet name
    pub name: String,
    /// Required signatures
    pub required_signatures: u32,
    /// Authorized signers
    pub signers: Vec<Address>,
    /// Pending transactions
    pub pending_transactions: Vec<PendingMultisigTransaction>,
    /// Created at
    pub created_at: SystemTime,
}

/// Pending multisig transaction
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PendingMultisigTransaction {
    /// Transaction ID
    pub id: Uuid,
    /// Transaction to be signed
    pub transaction: Transaction,
    /// Signatures collected
    pub signatures: Vec<MultisigSignature>,
    /// Created at
    pub created_at: SystemTime,
    /// Expires at
    pub expires_at: SystemTime,
    /// Status
    pub status: MultisigTransactionStatus,
}

/// Multisig signature
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MultisigSignature {
    /// Signer address
    pub signer: Address,
    /// Signature data
    pub signature: Vec<u8>,
    /// Signed at
    pub signed_at: SystemTime,
}

/// Multisig transaction status
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum MultisigTransactionStatus {
    Pending,
    Approved,
    Executed,
    Rejected,
    Expired,
}

/// Approval workflow
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApprovalWorkflow {
    /// Workflow ID
    pub id: Uuid,
    /// Workflow name
    pub name: String,
    /// Approval steps
    pub steps: Vec<ApprovalStep>,
    /// Current step
    pub current_step: usize,
    /// Associated transaction
    pub transaction_id: Option<Uuid>,
    /// Status
    pub status: WorkflowStatus,
    /// Created at
    pub created_at: SystemTime,
}

/// Approval step
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApprovalStep {
    /// Step name
    pub name: String,
    /// Required approvers
    pub required_approvers: Vec<String>,
    /// Current approvals
    pub approvals: Vec<Approval>,
    /// Minimum approvals needed
    pub min_approvals: u32,
    /// Step status
    pub status: StepStatus,
}

/// Individual approval
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Approval {
    /// Approver identity
    pub approver: String,
    /// Approval decision
    pub decision: ApprovalDecision,
    /// Comments
    pub comments: Option<String>,
    /// Approved at
    pub approved_at: SystemTime,
}

/// Approval decision
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum ApprovalDecision {
    Approved,
    Rejected,
    RequestMoreInfo,
}

/// Step status
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum StepStatus {
    Pending,
    InProgress,
    Completed,
    Skipped,
}

/// Workflow status
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum WorkflowStatus {
    Pending,
    InProgress,
    Approved,
    Rejected,
    Cancelled,
}

impl EnterpriseWalletManager {
    /// Create new enterprise wallet manager
    pub fn new(config: EnterpriseConfig) -> Self {
        Self {
            wallet_manager: WalletManager::with_config(crate::wallet::WalletConfig::default()),
            multisig_configs: Arc::new(RwLock::new(HashMap::new())),
            approval_workflows: Arc::new(RwLock::new(HashMap::new())),
            config,
        }
    }
    
    /// Create multisig wallet
    pub async fn create_multisig_wallet(
        &self,
        name: String,
        required_signatures: u32,
        signers: Vec<Address>,
    ) -> Result<Uuid> {
        if required_signatures == 0 || required_signatures > signers.len() as u32 {
            return Err(ParadigmError::Validation("Invalid signature requirements".to_string()));
        }
        
        let multisig_wallet = MultisigWallet {
            id: Uuid::new_v4(),
            name,
            required_signatures,
            signers,
            pending_transactions: Vec::new(),
            created_at: SystemTime::now(),
        };
        
        let id = multisig_wallet.id;
        self.multisig_configs.write().await.insert(id, multisig_wallet);
        
        Ok(id)
    }
    
    /// Submit transaction for multisig approval
    pub async fn submit_multisig_transaction(
        &self,
        wallet_id: Uuid,
        transaction: Transaction,
    ) -> Result<Uuid> {
        let mut multisig_configs = self.multisig_configs.write().await;
        
        if let Some(wallet) = multisig_configs.get_mut(&wallet_id) {
            let pending_tx = PendingMultisigTransaction {
                id: Uuid::new_v4(),
                transaction,
                signatures: Vec::new(),
                created_at: SystemTime::now(),
                expires_at: SystemTime::now() + Duration::from_secs(
                    self.config.multisig_config.signature_timeout_hours as u64 * 3600
                ),
                status: MultisigTransactionStatus::Pending,
            };
            
            let tx_id = pending_tx.id;
            wallet.pending_transactions.push(pending_tx);
            
            Ok(tx_id)
        } else {
            Err(ParadigmError::NotFound("Multisig wallet not found".to_string()))
        }
    }
    
    /// Sign multisig transaction
    pub async fn sign_multisig_transaction(
        &self,
        wallet_id: Uuid,
        transaction_id: Uuid,
        signer: Address,
        signature: Vec<u8>,
    ) -> Result<()> {
        let mut multisig_configs = self.multisig_configs.write().await;
        
        if let Some(wallet) = multisig_configs.get_mut(&wallet_id) {
            if !wallet.signers.contains(&signer) {
                return Err(ParadigmError::Authorization("Signer not authorized".to_string()));
            }
            
            if let Some(pending_tx) = wallet.pending_transactions.iter_mut().find(|tx| tx.id == transaction_id) {
                if pending_tx.status != MultisigTransactionStatus::Pending {
                    return Err(ParadigmError::Validation("Transaction not in pending state".to_string()));
                }
                
                if SystemTime::now() > pending_tx.expires_at {
                    pending_tx.status = MultisigTransactionStatus::Expired;
                    return Err(ParadigmError::Timeout("Transaction expired".to_string()));
                }
                
                // Check if already signed
                if pending_tx.signatures.iter().any(|sig| sig.signer == signer) {
                    return Err(ParadigmError::Validation("Already signed by this signer".to_string()));
                }
                
                // Add signature
                pending_tx.signatures.push(MultisigSignature {
                    signer,
                    signature,
                    signed_at: SystemTime::now(),
                });
                
                // Check if enough signatures
                if pending_tx.signatures.len() >= wallet.required_signatures as usize {
                    pending_tx.status = MultisigTransactionStatus::Approved;
                }
                
                Ok(())
            } else {
                Err(ParadigmError::NotFound("Pending transaction not found".to_string()))
            }
        } else {
            Err(ParadigmError::NotFound("Multisig wallet not found".to_string()))
        }
    }
    
    /// Get pending multisig transactions
    pub async fn get_pending_transactions(&self, wallet_id: Uuid) -> Result<Vec<PendingMultisigTransaction>> {
        let multisig_configs = self.multisig_configs.read().await;
        
        if let Some(wallet) = multisig_configs.get(&wallet_id) {
            Ok(wallet.pending_transactions.clone())
        } else {
            Err(ParadigmError::NotFound("Multisig wallet not found".to_string()))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_enterprise_api_manager() {
        let config = EnterpriseConfig::default();
        let api_manager = EnterpriseApiManager::new(config);
        
        let api_key = api_manager.create_api_key(
            "TestOrg".to_string(),
            vec![ApiPermission::ReadBlocks, ApiPermission::ReadTransactions],
            None,
        ).await.unwrap();
        
        assert!(api_manager.validate_api_key(&api_key.key, ApiPermission::ReadBlocks).await.unwrap());
        assert!(!api_manager.validate_api_key(&api_key.key, ApiPermission::AdminAccess).await.unwrap());
        
        assert!(api_manager.check_rate_limit(&api_key.key).await.unwrap());
    }
    
    #[tokio::test]
    async fn test_compliance_monitor() {
        let config = ComplianceConfig::default();
        let monitor = ComplianceMonitor::new(config);
        
        let rule = ComplianceRule {
            id: Uuid::new_v4(),
            name: "Large Transaction".to_string(),
            description: "Detect large transactions".to_string(),
            rule_type: ComplianceRuleType::LargeTransaction { 
                threshold: Amount::from_paradigm(1000) 
            },
            active: true,
            severity: ComplianceSeverity::High,
            actions: vec![ComplianceAction::Alert],
        };
        
        monitor.add_rule(rule).await;
        
        let transaction = Transaction {
            value: Amount::from_paradigm(2000),
            from: crate::utils::crypto::random_address(),
            ..crate::utils::debug::create_debug_transaction()
        };
        
        let alerts = monitor.monitor_transaction(&transaction).await.unwrap();
        assert_eq!(alerts.len(), 1);
        assert_eq!(alerts[0].severity, ComplianceSeverity::High);
    }
    
    #[tokio::test]
    async fn test_multisig_wallet() {
        let config = EnterpriseConfig::default();
        let wallet_manager = EnterpriseWalletManager::new(config);
        
        let signers = vec![
            crate::utils::crypto::random_address(),
            crate::utils::crypto::random_address(),
            crate::utils::crypto::random_address(),
        ];
        
        let wallet_id = wallet_manager.create_multisig_wallet(
            "Test Multisig".to_string(),
            2,
            signers.clone(),
        ).await.unwrap();
        
        let transaction = crate::utils::debug::create_debug_transaction();
        let tx_id = wallet_manager.submit_multisig_transaction(wallet_id, transaction).await.unwrap();
        
        // First signature
        wallet_manager.sign_multisig_transaction(
            wallet_id,
            tx_id,
            signers[0].clone(),
            vec![0; 65],
        ).await.unwrap();
        
        // Second signature
        wallet_manager.sign_multisig_transaction(
            wallet_id,
            tx_id,
            signers[1].clone(),
            vec![0; 65],
        ).await.unwrap();
        
        let pending_txs = wallet_manager.get_pending_transactions(wallet_id).await.unwrap();
        assert_eq!(pending_txs[0].status, MultisigTransactionStatus::Approved);
    }
}