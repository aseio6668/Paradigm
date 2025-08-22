use axum::{
    extract::{Query, State},
    routing::{get, post},
    Json, Router,
};
use std::sync::Arc;
use validator::Validate;

use crate::{
    error::{ApiErrorType, ApiResult},
    models::{
        ApiResponse, LoginRequest, LoginResponse, RegisterRequest, 
        UserProfile, PaginationParams,
    },
    services::ApiServices,
};

/// Authentication routes
pub fn router(services: Arc<ApiServices>) -> Router {
    Router::new()
        .route("/auth/login", post(login))
        .route("/auth/register", post(register))
        .route("/auth/refresh", post(refresh_token))
        .route("/auth/logout", post(logout))
        .route("/auth/profile", get(get_profile))
        .route("/auth/profile", post(update_profile))
        .route("/auth/api-keys", get(list_api_keys))
        .route("/auth/api-keys", post(create_api_key))
        .route("/auth/api-keys/:key_id", post(revoke_api_key))
        .with_state(services)
}

/// User login
async fn login(
    State(services): State<Arc<ApiServices>>,
    Json(request): Json<LoginRequest>,
) -> ApiResult<Json<ApiResponse<LoginResponse>>> {
    request.validate()
        .map_err(|e| ApiErrorType::ValidationFailed { 
            field: e.field_errors().keys().next().unwrap_or(&"unknown").to_string() 
        })?;

    let response = services.auth_service()
        .login(&request.email, &request.password)
        .await?;

    Ok(Json(ApiResponse::success(response)))
}

/// User registration
async fn register(
    State(services): State<Arc<ApiServices>>,
    Json(request): Json<RegisterRequest>,
) -> ApiResult<Json<ApiResponse<UserProfile>>> {
    request.validate()
        .map_err(|e| ApiErrorType::ValidationFailed { 
            field: e.field_errors().keys().next().unwrap_or(&"unknown").to_string() 
        })?;

    let user = services.auth_service()
        .register(&request.email, &request.password, &request.name, request.organization.as_deref())
        .await?;

    Ok(Json(ApiResponse::success(user)))
}

/// Refresh JWT token
async fn refresh_token(
    State(services): State<Arc<ApiServices>>,
    Json(refresh_token): Json<serde_json::Value>,
) -> ApiResult<Json<ApiResponse<LoginResponse>>> {
    let token = refresh_token["refresh_token"]
        .as_str()
        .ok_or(ApiErrorType::InvalidRequest { 
            message: "Missing refresh_token".to_string() 
        })?;

    let response = services.auth_service()
        .refresh_token(token)
        .await?;

    Ok(Json(ApiResponse::success(response)))
}

/// User logout
async fn logout(
    State(services): State<Arc<ApiServices>>,
    Json(token_data): Json<serde_json::Value>,
) -> ApiResult<Json<ApiResponse<()>>> {
    let token = token_data["token"]
        .as_str()
        .ok_or(ApiErrorType::InvalidRequest { 
            message: "Missing token".to_string() 
        })?;

    services.auth_service()
        .logout(token)
        .await?;

    Ok(Json(ApiResponse::success(())))
}

/// Get user profile
async fn get_profile(
    State(services): State<Arc<ApiServices>>,
    // user_claims: UserClaims, // Would come from auth middleware
) -> ApiResult<Json<ApiResponse<UserProfile>>> {
    // For now, return a mock user profile
    let profile = UserProfile {
        id: uuid::Uuid::new_v4(),
        email: "user@example.com".to_string(),
        name: "John Doe".to_string(),
        organization: Some("Acme Corp".to_string()),
        role: crate::models::UserRole::Developer,
        api_key: Some("par_live_1234567890abcdef".to_string()),
        created_at: chrono::Utc::now(),
        last_login: Some(chrono::Utc::now()),
    };

    Ok(Json(ApiResponse::success(profile)))
}

/// Update user profile
async fn update_profile(
    State(services): State<Arc<ApiServices>>,
    Json(update_data): Json<serde_json::Value>,
) -> ApiResult<Json<ApiResponse<UserProfile>>> {
    // Mock implementation
    let profile = UserProfile {
        id: uuid::Uuid::new_v4(),
        email: "user@example.com".to_string(),
        name: update_data["name"].as_str().unwrap_or("John Doe").to_string(),
        organization: update_data["organization"].as_str().map(|s| s.to_string()),
        role: crate::models::UserRole::Developer,
        api_key: Some("par_live_1234567890abcdef".to_string()),
        created_at: chrono::Utc::now(),
        last_login: Some(chrono::Utc::now()),
    };

    Ok(Json(ApiResponse::success(profile)))
}

/// List user's API keys
async fn list_api_keys(
    State(services): State<Arc<ApiServices>>,
    Query(pagination): Query<PaginationParams>,
) -> ApiResult<Json<ApiResponse<Vec<ApiKeyInfo>>>> {
    pagination.validate()
        .map_err(|e| ApiErrorType::ValidationFailed { 
            field: e.field_errors().keys().next().unwrap_or(&"unknown").to_string() 
        })?;

    // Mock API keys
    let api_keys = vec![
        ApiKeyInfo {
            id: uuid::Uuid::new_v4(),
            name: "Production API Key".to_string(),
            key_prefix: "par_live_1234".to_string(),
            permissions: vec!["read".to_string(), "write".to_string()],
            last_used: Some(chrono::Utc::now()),
            expires_at: None,
            created_at: chrono::Utc::now(),
        },
        ApiKeyInfo {
            id: uuid::Uuid::new_v4(),
            name: "Development API Key".to_string(),
            key_prefix: "par_test_5678".to_string(),
            permissions: vec!["read".to_string()],
            last_used: Some(chrono::Utc::now() - chrono::Duration::hours(2)),
            expires_at: Some(chrono::Utc::now() + chrono::Duration::days(90)),
            created_at: chrono::Utc::now() - chrono::Duration::days(7),
        },
    ];

    Ok(Json(ApiResponse::success(api_keys)))
}

/// Create new API key
async fn create_api_key(
    State(services): State<Arc<ApiServices>>,
    Json(request): Json<CreateApiKeyRequest>,
) -> ApiResult<Json<ApiResponse<ApiKeyResponse>>> {
    request.validate()
        .map_err(|e| ApiErrorType::ValidationFailed { 
            field: e.field_errors().keys().next().unwrap_or(&"unknown").to_string() 
        })?;

    let api_key = services.auth_service()
        .create_api_key(&request.name, &request.permissions, request.expires_in_days)
        .await?;

    Ok(Json(ApiResponse::success(api_key)))
}

/// Revoke API key
async fn revoke_api_key(
    State(services): State<Arc<ApiServices>>,
    axum::extract::Path(key_id): axum::extract::Path<uuid::Uuid>,
) -> ApiResult<Json<ApiResponse<()>>> {
    services.auth_service()
        .revoke_api_key(&key_id)
        .await?;

    Ok(Json(ApiResponse::success(())))
}

// Supporting types
#[derive(serde::Serialize, serde::Deserialize)]
pub struct ApiKeyInfo {
    pub id: uuid::Uuid,
    pub name: String,
    pub key_prefix: String,
    pub permissions: Vec<String>,
    pub last_used: Option<chrono::DateTime<chrono::Utc>>,
    pub expires_at: Option<chrono::DateTime<chrono::Utc>>,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

#[derive(serde::Serialize, serde::Deserialize, validator::Validate)]
pub struct CreateApiKeyRequest {
    #[validate(length(min = 1, max = 100))]
    pub name: String,
    pub permissions: Vec<String>,
    pub expires_in_days: Option<u32>,
}

#[derive(serde::Serialize, serde::Deserialize)]
pub struct ApiKeyResponse {
    pub id: uuid::Uuid,
    pub name: String,
    pub api_key: String, // Full key returned only once
    pub permissions: Vec<String>,
    pub expires_at: Option<chrono::DateTime<chrono::Utc>>,
    pub created_at: chrono::DateTime<chrono::Utc>,
}