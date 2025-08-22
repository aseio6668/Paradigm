use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde_json::json;
use thiserror::Error;

use crate::models::{ApiError, ApiResponse};

#[derive(Error, Debug)]
pub enum ApiErrorType {
    // Authentication errors
    #[error("Invalid credentials")]
    InvalidCredentials,
    #[error("Token expired")]
    TokenExpired,
    #[error("Unauthorized")]
    Unauthorized,
    #[error("Forbidden")]
    Forbidden,
    
    // Validation errors
    #[error("Invalid request: {message}")]
    InvalidRequest { message: String },
    #[error("Validation failed: {field}")]
    ValidationFailed { field: String },
    #[error("Missing required field: {field}")]
    MissingField { field: String },
    
    // Resource errors
    #[error("Resource not found: {resource}")]
    NotFound { resource: String },
    #[error("Resource already exists: {resource}")]
    AlreadyExists { resource: String },
    #[error("Resource conflict: {message}")]
    Conflict { message: String },
    
    // Blockchain errors
    #[error("Transaction failed: {reason}")]
    TransactionFailed { reason: String },
    #[error("Insufficient balance")]
    InsufficientBalance,
    #[error("Invalid address: {address}")]
    InvalidAddress { address: String },
    #[error("Network error: {message}")]
    NetworkError { message: String },
    
    // Rate limiting
    #[error("Rate limit exceeded")]
    RateLimitExceeded,
    #[error("Quota exceeded")]
    QuotaExceeded,
    
    // Server errors
    #[error("Internal server error")]
    InternalServerError,
    #[error("Service unavailable")]
    ServiceUnavailable,
    #[error("Database error: {message}")]
    DatabaseError { message: String },
    #[error("External service error: {service}")]
    ExternalServiceError { service: String },
    
    // Custom business logic errors
    #[error("ML task failed: {reason}")]
    MLTaskFailed { reason: String },
    #[error("Cross-chain transfer failed: {reason}")]
    CrossChainFailed { reason: String },
    #[error("Governance proposal invalid: {reason}")]
    ProposalInvalid { reason: String },
}

impl ApiErrorType {
    pub fn to_api_error(&self) -> ApiError {
        match self {
            ApiErrorType::InvalidCredentials => ApiError {
                code: "INVALID_CREDENTIALS".to_string(),
                message: self.to_string(),
                details: None,
            },
            ApiErrorType::TokenExpired => ApiError {
                code: "TOKEN_EXPIRED".to_string(),
                message: self.to_string(),
                details: None,
            },
            ApiErrorType::Unauthorized => ApiError {
                code: "UNAUTHORIZED".to_string(),
                message: self.to_string(),
                details: None,
            },
            ApiErrorType::Forbidden => ApiError {
                code: "FORBIDDEN".to_string(),
                message: self.to_string(),
                details: None,
            },
            ApiErrorType::InvalidRequest { message } => ApiError {
                code: "INVALID_REQUEST".to_string(),
                message: self.to_string(),
                details: Some(json!({ "details": message })),
            },
            ApiErrorType::ValidationFailed { field } => ApiError {
                code: "VALIDATION_FAILED".to_string(),
                message: self.to_string(),
                details: Some(json!({ "field": field })),
            },
            ApiErrorType::MissingField { field } => ApiError {
                code: "MISSING_FIELD".to_string(),
                message: self.to_string(),
                details: Some(json!({ "field": field })),
            },
            ApiErrorType::NotFound { resource } => ApiError {
                code: "NOT_FOUND".to_string(),
                message: self.to_string(),
                details: Some(json!({ "resource": resource })),
            },
            ApiErrorType::AlreadyExists { resource } => ApiError {
                code: "ALREADY_EXISTS".to_string(),
                message: self.to_string(),
                details: Some(json!({ "resource": resource })),
            },
            ApiErrorType::Conflict { message } => ApiError {
                code: "CONFLICT".to_string(),
                message: self.to_string(),
                details: Some(json!({ "details": message })),
            },
            ApiErrorType::TransactionFailed { reason } => ApiError {
                code: "TRANSACTION_FAILED".to_string(),
                message: self.to_string(),
                details: Some(json!({ "reason": reason })),
            },
            ApiErrorType::InsufficientBalance => ApiError {
                code: "INSUFFICIENT_BALANCE".to_string(),
                message: self.to_string(),
                details: None,
            },
            ApiErrorType::InvalidAddress { address } => ApiError {
                code: "INVALID_ADDRESS".to_string(),
                message: self.to_string(),
                details: Some(json!({ "address": address })),
            },
            ApiErrorType::NetworkError { message } => ApiError {
                code: "NETWORK_ERROR".to_string(),
                message: self.to_string(),
                details: Some(json!({ "details": message })),
            },
            ApiErrorType::RateLimitExceeded => ApiError {
                code: "RATE_LIMIT_EXCEEDED".to_string(),
                message: self.to_string(),
                details: None,
            },
            ApiErrorType::QuotaExceeded => ApiError {
                code: "QUOTA_EXCEEDED".to_string(),
                message: self.to_string(),
                details: None,
            },
            ApiErrorType::InternalServerError => ApiError {
                code: "INTERNAL_SERVER_ERROR".to_string(),
                message: self.to_string(),
                details: None,
            },
            ApiErrorType::ServiceUnavailable => ApiError {
                code: "SERVICE_UNAVAILABLE".to_string(),
                message: self.to_string(),
                details: None,
            },
            ApiErrorType::DatabaseError { message } => ApiError {
                code: "DATABASE_ERROR".to_string(),
                message: self.to_string(),
                details: Some(json!({ "details": message })),
            },
            ApiErrorType::ExternalServiceError { service } => ApiError {
                code: "EXTERNAL_SERVICE_ERROR".to_string(),
                message: self.to_string(),
                details: Some(json!({ "service": service })),
            },
            ApiErrorType::MLTaskFailed { reason } => ApiError {
                code: "ML_TASK_FAILED".to_string(),
                message: self.to_string(),
                details: Some(json!({ "reason": reason })),
            },
            ApiErrorType::CrossChainFailed { reason } => ApiError {
                code: "CROSS_CHAIN_FAILED".to_string(),
                message: self.to_string(),
                details: Some(json!({ "reason": reason })),
            },
            ApiErrorType::ProposalInvalid { reason } => ApiError {
                code: "PROPOSAL_INVALID".to_string(),
                message: self.to_string(),
                details: Some(json!({ "reason": reason })),
            },
        }
    }
    
    pub fn status_code(&self) -> StatusCode {
        match self {
            ApiErrorType::InvalidCredentials 
            | ApiErrorType::TokenExpired 
            | ApiErrorType::Unauthorized => StatusCode::UNAUTHORIZED,
            
            ApiErrorType::Forbidden => StatusCode::FORBIDDEN,
            
            ApiErrorType::InvalidRequest { .. }
            | ApiErrorType::ValidationFailed { .. }
            | ApiErrorType::MissingField { .. }
            | ApiErrorType::InvalidAddress { .. }
            | ApiErrorType::InsufficientBalance
            | ApiErrorType::ProposalInvalid { .. } => StatusCode::BAD_REQUEST,
            
            ApiErrorType::NotFound { .. } => StatusCode::NOT_FOUND,
            
            ApiErrorType::AlreadyExists { .. }
            | ApiErrorType::Conflict { .. } => StatusCode::CONFLICT,
            
            ApiErrorType::RateLimitExceeded => StatusCode::TOO_MANY_REQUESTS,
            
            ApiErrorType::QuotaExceeded => StatusCode::PAYMENT_REQUIRED,
            
            ApiErrorType::ServiceUnavailable => StatusCode::SERVICE_UNAVAILABLE,
            
            ApiErrorType::TransactionFailed { .. }
            | ApiErrorType::NetworkError { .. }
            | ApiErrorType::InternalServerError
            | ApiErrorType::DatabaseError { .. }
            | ApiErrorType::ExternalServiceError { .. }
            | ApiErrorType::MLTaskFailed { .. }
            | ApiErrorType::CrossChainFailed { .. } => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}

impl IntoResponse for ApiErrorType {
    fn into_response(self) -> Response {
        let status = self.status_code();
        let api_error = self.to_api_error();
        let response: ApiResponse<()> = ApiResponse::error(api_error);
        
        tracing::error!("API Error: {} - {}", status, self);
        
        (status, Json(response)).into_response()
    }
}

// Convenience type alias
pub type ApiResult<T> = Result<T, ApiErrorType>;

// Helper functions for common errors
pub fn validation_error(field: &str) -> ApiErrorType {
    ApiErrorType::ValidationFailed {
        field: field.to_string(),
    }
}

pub fn not_found(resource: &str) -> ApiErrorType {
    ApiErrorType::NotFound {
        resource: resource.to_string(),
    }
}

pub fn invalid_request(message: &str) -> ApiErrorType {
    ApiErrorType::InvalidRequest {
        message: message.to_string(),
    }
}

pub fn transaction_failed(reason: &str) -> ApiErrorType {
    ApiErrorType::TransactionFailed {
        reason: reason.to_string(),
    }
}

pub fn network_error(message: &str) -> ApiErrorType {
    ApiErrorType::NetworkError {
        message: message.to_string(),
    }
}

pub fn database_error(message: &str) -> ApiErrorType {
    ApiErrorType::DatabaseError {
        message: message.to_string(),
    }
}

// Convert from common error types
impl From<sqlx::Error> for ApiErrorType {
    fn from(err: sqlx::Error) -> Self {
        ApiErrorType::DatabaseError {
            message: err.to_string(),
        }
    }
}

impl From<serde_json::Error> for ApiErrorType {
    fn from(err: serde_json::Error) -> Self {
        ApiErrorType::InvalidRequest {
            message: format!("JSON parsing error: {}", err),
        }
    }
}

impl From<uuid::Error> for ApiErrorType {
    fn from(err: uuid::Error) -> Self {
        ApiErrorType::InvalidRequest {
            message: format!("Invalid UUID: {}", err),
        }
    }
}

impl From<validator::ValidationErrors> for ApiErrorType {
    fn from(err: validator::ValidationErrors) -> Self {
        let field = err.field_errors()
            .keys()
            .next()
            .unwrap_or(&"unknown")
            .to_string();
            
        ApiErrorType::ValidationFailed { field }
    }
}

impl From<anyhow::Error> for ApiErrorType {
    fn from(err: anyhow::Error) -> Self {
        ApiErrorType::InternalServerError
    }
}