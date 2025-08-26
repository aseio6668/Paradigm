// Error types for the Paradigm SDK

use thiserror::Error;

/// Result type alias for convenience
pub type Result<T> = std::result::Result<T, ParadigmError>;

/// Main error type for the Paradigm SDK
#[derive(Error, Debug)]
pub enum ParadigmError {
    /// Network connection errors
    #[error("Network error: {0}")]
    Network(String),

    /// RPC errors
    #[error("RPC error: {0}")]
    Rpc(String),

    /// Invalid address format
    #[error("Invalid address: {0}")]
    InvalidAddress(String),

    /// Invalid hash format
    #[error("Invalid hash: {0}")]
    InvalidHash(String),

    /// Invalid hash length
    #[error("Invalid hash length - expected 32 bytes")]
    InvalidHashLength,

    /// Invalid hex string
    #[error("Invalid hex: {0}")]
    InvalidHex(String),

    /// Invalid amount format
    #[error("Invalid amount: {0}")]
    InvalidAmount(String),

    /// Invalid signature
    #[error("Invalid signature: {0}")]
    InvalidSignature(String),

    /// Invalid key format
    #[error("Invalid key: {0}")]
    InvalidKey(String),

    /// Unsupported key type
    #[error("Unsupported key type: {0}")]
    UnsupportedKeyType(String),

    /// Transaction errors
    #[error("Transaction error: {0}")]
    Transaction(String),

    /// Insufficient funds
    #[error("Insufficient funds: required {required}, available {available}")]
    InsufficientFunds { required: u64, available: u64 },

    /// Gas estimation errors
    #[error("Gas estimation failed: {0}")]
    GasEstimation(String),

    /// Contract errors
    #[error("Contract error: {0}")]
    Contract(String),

    /// ABI encoding/decoding errors
    #[error("ABI error: {0}")]
    Abi(String),

    /// Wallet errors
    #[error("Wallet error: {0}")]
    Wallet(String),

    /// Keystore errors
    #[error("Keystore error: {0}")]
    Keystore(String),

    /// Configuration errors
    #[error("Configuration error: {0}")]
    Config(String),

    /// Serialization errors
    #[error("Serialization error: {0}")]
    Serialization(String),

    /// Authentication errors
    #[error("Authentication error: {0}")]
    Authentication(String),

    /// Authorization errors
    #[error("Authorization error: {0}")]
    Authorization(String),

    /// Rate limiting errors
    #[error("Rate limited: {0}")]
    RateLimit(String),

    /// Timeout errors
    #[error("Timeout: {0}")]
    Timeout(String),

    /// Not found errors
    #[error("Not found: {0}")]
    NotFound(String),

    /// Already exists errors
    #[error("Already exists: {0}")]
    AlreadyExists(String),

    /// Validation errors
    #[error("Validation error: {0}")]
    Validation(String),

    /// Generic I/O errors
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    /// JSON parsing errors
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    /// HTTP request errors
    #[error("HTTP error: {0}")]
    Http(#[from] reqwest::Error),

    /// URL parsing errors
    #[error("URL error: {0}")]
    Url(#[from] url::ParseError),

    /// Database errors
    #[error("Database error: {0}")]
    Database(String),

    /// Cryptographic errors
    #[error("Crypto error: {0}")]
    Crypto(String),

    /// Generic errors
    #[error("Error: {0}")]
    Generic(String),

    /// Multiple errors combined
    #[error("Multiple errors: {0:?}")]
    Multiple(Vec<ParadigmError>),
}

/// Error context for providing additional information
#[derive(Debug, Clone)]
pub struct ErrorContext {
    /// Error operation context
    pub operation: String,

    /// Additional details
    pub details: std::collections::HashMap<String, String>,

    /// Timestamp when error occurred
    pub timestamp: std::time::SystemTime,
}

impl ErrorContext {
    /// Create new error context
    pub fn new(operation: impl Into<String>) -> Self {
        Self {
            operation: operation.into(),
            details: std::collections::HashMap::new(),
            timestamp: std::time::SystemTime::now(),
        }
    }

    /// Add detail to context
    pub fn with_detail(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.details.insert(key.into(), value.into());
        self
    }

    /// Add multiple details
    pub fn with_details(mut self, details: std::collections::HashMap<String, String>) -> Self {
        self.details.extend(details);
        self
    }
}

/// Extension trait for adding context to errors
pub trait ErrorExt<T> {
    /// Add context to an error
    fn with_context(self, context: ErrorContext) -> Result<T>;

    /// Add operation context
    fn with_operation(self, operation: impl Into<String>) -> Result<T>;
}

impl<T> ErrorExt<T> for Result<T> {
    fn with_context(self, context: ErrorContext) -> Result<T> {
        self.map_err(|e| {
            let details = context
                .details
                .iter()
                .map(|(k, v)| format!("{}={}", k, v))
                .collect::<Vec<_>>()
                .join(", ");

            ParadigmError::Generic(format!(
                "{} (operation: {}, details: {}, timestamp: {:?})",
                e, context.operation, details, context.timestamp
            ))
        })
    }

    fn with_operation(self, operation: impl Into<String>) -> Result<T> {
        self.with_context(ErrorContext::new(operation))
    }
}

/// Error recovery strategies
#[derive(Debug, Clone)]
pub enum RecoveryStrategy {
    /// Retry the operation
    Retry {
        max_attempts: u32,
        delay: std::time::Duration,
        backoff_multiplier: f64,
    },

    /// Use fallback value/method
    Fallback { description: String },

    /// Graceful degradation
    Degrade { reduced_functionality: String },

    /// Fail fast - no recovery possible
    FailFast,
}

/// Error recovery information
#[derive(Debug, Clone)]
pub struct ErrorRecovery {
    /// Suggested recovery strategy
    pub strategy: RecoveryStrategy,

    /// Human-readable description
    pub description: String,

    /// Whether automatic recovery is possible
    pub automatic: bool,
}

impl ParadigmError {
    /// Get suggested recovery strategy for this error
    pub fn recovery_strategy(&self) -> ErrorRecovery {
        match self {
            ParadigmError::Network(_) | ParadigmError::Rpc(_) | ParadigmError::Http(_) => {
                ErrorRecovery {
                    strategy: RecoveryStrategy::Retry {
                        max_attempts: 3,
                        delay: std::time::Duration::from_millis(1000),
                        backoff_multiplier: 2.0,
                    },
                    description: "Network errors can often be resolved by retrying".to_string(),
                    automatic: true,
                }
            }

            ParadigmError::RateLimit(_) => ErrorRecovery {
                strategy: RecoveryStrategy::Retry {
                    max_attempts: 5,
                    delay: std::time::Duration::from_secs(60),
                    backoff_multiplier: 1.5,
                },
                description: "Rate limits usually clear after waiting".to_string(),
                automatic: true,
            },

            ParadigmError::Timeout(_) => ErrorRecovery {
                strategy: RecoveryStrategy::Retry {
                    max_attempts: 2,
                    delay: std::time::Duration::from_millis(500),
                    backoff_multiplier: 1.0,
                },
                description: "Timeouts may be resolved by retrying".to_string(),
                automatic: true,
            },

            ParadigmError::NotFound(_) => ErrorRecovery {
                strategy: RecoveryStrategy::Fallback {
                    description: "Use default values or alternative data source".to_string(),
                },
                description: "Missing data might be available from other sources".to_string(),
                automatic: false,
            },

            ParadigmError::InsufficientFunds { .. } => ErrorRecovery {
                strategy: RecoveryStrategy::FailFast,
                description: "User needs to add funds before retrying".to_string(),
                automatic: false,
            },

            ParadigmError::InvalidAddress(_)
            | ParadigmError::InvalidHash(_)
            | ParadigmError::InvalidAmount(_) => ErrorRecovery {
                strategy: RecoveryStrategy::FailFast,
                description: "Input validation errors require user correction".to_string(),
                automatic: false,
            },

            _ => ErrorRecovery {
                strategy: RecoveryStrategy::FailFast,
                description: "Error requires manual intervention".to_string(),
                automatic: false,
            },
        }
    }

    /// Check if error is retryable
    pub fn is_retryable(&self) -> bool {
        matches!(
            self.recovery_strategy().strategy,
            RecoveryStrategy::Retry { .. }
        )
    }

    /// Check if error indicates a temporary issue
    pub fn is_temporary(&self) -> bool {
        matches!(
            self,
            ParadigmError::Network(_)
                | ParadigmError::Rpc(_)
                | ParadigmError::Http(_)
                | ParadigmError::RateLimit(_)
                | ParadigmError::Timeout(_)
        )
    }

    /// Check if error indicates a permanent failure
    pub fn is_permanent(&self) -> bool {
        !self.is_temporary()
    }

    /// Get error category
    pub fn category(&self) -> ErrorCategory {
        match self {
            ParadigmError::Network(_) | ParadigmError::Rpc(_) | ParadigmError::Http(_) => {
                ErrorCategory::Network
            }

            ParadigmError::InvalidAddress(_)
            | ParadigmError::InvalidHash(_)
            | ParadigmError::InvalidAmount(_)
            | ParadigmError::InvalidSignature(_)
            | ParadigmError::InvalidKey(_)
            | ParadigmError::Validation(_) => ErrorCategory::Validation,

            ParadigmError::Authentication(_) | ParadigmError::Authorization(_) => {
                ErrorCategory::Security
            }

            ParadigmError::InsufficientFunds { .. } => ErrorCategory::Business,

            ParadigmError::Wallet(_) | ParadigmError::Keystore(_) => ErrorCategory::Wallet,

            ParadigmError::Contract(_) | ParadigmError::Abi(_) => ErrorCategory::SmartContract,

            ParadigmError::Transaction(_) | ParadigmError::GasEstimation(_) => {
                ErrorCategory::Transaction
            }

            _ => ErrorCategory::Internal,
        }
    }
}

/// Error categories for grouping related errors
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ErrorCategory {
    /// Network and connectivity errors
    Network,

    /// Input validation errors
    Validation,

    /// Security-related errors
    Security,

    /// Business logic errors
    Business,

    /// Wallet-related errors
    Wallet,

    /// Smart contract errors
    SmartContract,

    /// Transaction errors
    Transaction,

    /// Internal system errors
    Internal,
}

/// Error metrics for monitoring and debugging
#[derive(Debug, Clone)]
pub struct ErrorMetrics {
    /// Error count by category
    pub error_counts: std::collections::HashMap<ErrorCategory, u64>,

    /// Most common errors
    pub common_errors: Vec<(String, u64)>,

    /// Error rate over time
    pub error_rate: f64,

    /// Recovery success rate
    pub recovery_rate: f64,
}

impl ErrorMetrics {
    /// Create new error metrics
    pub fn new() -> Self {
        Self {
            error_counts: std::collections::HashMap::new(),
            common_errors: Vec::new(),
            error_rate: 0.0,
            recovery_rate: 0.0,
        }
    }

    /// Record an error occurrence
    pub fn record_error(&mut self, error: &ParadigmError) {
        let category = error.category();
        *self.error_counts.entry(category).or_insert(0) += 1;

        let error_string = error.to_string();
        if let Some((_, count)) = self
            .common_errors
            .iter_mut()
            .find(|(err, _)| err == &error_string)
        {
            *count += 1;
        } else {
            self.common_errors.push((error_string, 1));
        }

        // Sort by count and keep top 10
        self.common_errors.sort_by(|a, b| b.1.cmp(&a.1));
        self.common_errors.truncate(10);
    }

    /// Get total error count
    pub fn total_errors(&self) -> u64 {
        self.error_counts.values().sum()
    }

    /// Get error count for category
    pub fn errors_for_category(&self, category: &ErrorCategory) -> u64 {
        self.error_counts.get(category).copied().unwrap_or(0)
    }
}

impl Default for ErrorMetrics {
    fn default() -> Self {
        Self::new()
    }
}

/// Custom error for specific use cases
pub trait CustomError: std::error::Error + Send + Sync + 'static {
    /// Convert to ParadigmError
    fn into_paradigm_error(self) -> ParadigmError;
}

/// Macro for creating custom errors quickly
#[macro_export]
macro_rules! paradigm_error {
    ($variant:ident, $msg:expr) => {
        $crate::error::ParadigmError::$variant($msg.to_string())
    };
    ($variant:ident, $fmt:expr, $($arg:tt)*) => {
        $crate::error::ParadigmError::$variant(format!($fmt, $($arg)*))
    };
}

/// Macro for early return with context
#[macro_export]
macro_rules! ensure {
    ($condition:expr, $error:expr) => {
        if !($condition) {
            return Err($error);
        }
    };
}

/// Macro for converting Option to Result
#[macro_export]
macro_rules! ok_or {
    ($option:expr, $error:expr) => {
        match $option {
            Some(val) => val,
            None => return Err($error),
        }
    };
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_categories() {
        let network_error = ParadigmError::Network("Connection failed".to_string());
        assert_eq!(network_error.category(), ErrorCategory::Network);
        assert!(network_error.is_temporary());
        assert!(network_error.is_retryable());

        let validation_error = ParadigmError::InvalidAddress("Bad format".to_string());
        assert_eq!(validation_error.category(), ErrorCategory::Validation);
        assert!(validation_error.is_permanent());
        assert!(!validation_error.is_retryable());
    }

    #[test]
    fn test_recovery_strategies() {
        let network_error = ParadigmError::Network("Connection failed".to_string());
        let recovery = network_error.recovery_strategy();
        assert!(matches!(recovery.strategy, RecoveryStrategy::Retry { .. }));
        assert!(recovery.automatic);

        let validation_error = ParadigmError::InvalidAddress("Bad format".to_string());
        let recovery = validation_error.recovery_strategy();
        assert!(matches!(recovery.strategy, RecoveryStrategy::FailFast));
        assert!(!recovery.automatic);
    }

    #[test]
    fn test_error_metrics() {
        let mut metrics = ErrorMetrics::new();

        let error1 = ParadigmError::Network("Connection failed".to_string());
        let error2 = ParadigmError::InvalidAddress("Bad format".to_string());

        metrics.record_error(&error1);
        metrics.record_error(&error2);
        metrics.record_error(&error1); // Same error again

        assert_eq!(metrics.total_errors(), 3);
        assert_eq!(metrics.errors_for_category(&ErrorCategory::Network), 2);
        assert_eq!(metrics.errors_for_category(&ErrorCategory::Validation), 1);
    }

    #[test]
    fn test_error_context() {
        let context = ErrorContext::new("test_operation")
            .with_detail("user_id", "123")
            .with_detail("action", "transfer");

        assert_eq!(context.operation, "test_operation");
        assert_eq!(context.details.get("user_id"), Some(&"123".to_string()));
        assert_eq!(context.details.get("action"), Some(&"transfer".to_string()));
    }

    #[test]
    fn test_macros() {
        let error = paradigm_error!(Network, "Connection failed");
        assert!(matches!(error, ParadigmError::Network(_)));

        let formatted_error = paradigm_error!(Network, "Port {} unavailable", 8545);
        assert!(matches!(formatted_error, ParadigmError::Network(_)));
    }
}
