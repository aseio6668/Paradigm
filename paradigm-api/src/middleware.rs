use axum::{
    extract::{Request, State},
    http::{HeaderMap, StatusCode},
    middleware::Next,
    response::Response,
};
use governor::{
    clock::DefaultClock,
    state::{DirectStateStore, InMemoryState},
    Quota, RateLimiter,
};
use nonzero_ext::*;
use std::{
    net::SocketAddr,
    sync::Arc,
    time::Duration,
};
use tower::Layer;
use tracing::{info, warn};

use crate::{
    auth::AuthService,
    error::{ApiErrorType, ApiResult},
    services::ApiServices,
};

/// Authentication middleware
pub async fn auth_middleware(
    State(services): State<Arc<ApiServices>>,
    headers: HeaderMap,
    mut request: Request,
    next: Next,
) -> Result<Response, ApiErrorType> {
    // Skip authentication for certain paths
    let path = request.uri().path();
    if should_skip_auth(path) {
        return Ok(next.run(request).await);
    }

    // Extract token from Authorization header
    let token = extract_bearer_token(&headers)
        .ok_or(ApiErrorType::Unauthorized)?;

    // Validate token and get user info
    let user_claims = services.auth_service()
        .validate_token(&token)
        .await
        .map_err(|_| ApiErrorType::Unauthorized)?;

    // Add user info to request extensions
    request.extensions_mut().insert(user_claims);

    Ok(next.run(request).await)
}

/// Rate limiting middleware
pub fn rate_limit_middleware() -> impl Layer<axum::routing::Router> {
    // Create rate limiter: 60 requests per minute with burst of 10
    let quota = Quota::per_minute(nonzero!(60u32))
        .allow_burst(nonzero!(10u32));
    let limiter = Arc::new(RateLimiter::direct(quota));

    axum::middleware::from_fn(move |request: Request, next: Next| {
        let limiter = limiter.clone();
        async move {
            // Extract client IP
            let client_ip = extract_client_ip(&request);
            
            // Check rate limit
            match limiter.check_key(&client_ip) {
                Ok(_) => {
                    // Rate limit not exceeded
                    Ok(next.run(request).await)
                }
                Err(_) => {
                    warn!("Rate limit exceeded for IP: {}", client_ip);
                    Err(ApiErrorType::RateLimitExceeded)
                }
            }
        }
    })
}

/// CORS middleware configuration
pub fn cors_middleware() -> tower_http::cors::CorsLayer {
    tower_http::cors::CorsLayer::new()
        .allow_origin(tower_http::cors::Any)
        .allow_methods([
            axum::http::Method::GET,
            axum::http::Method::POST,
            axum::http::Method::PUT,
            axum::http::Method::DELETE,
            axum::http::Method::PATCH,
            axum::http::Method::OPTIONS,
        ])
        .allow_headers([
            axum::http::header::AUTHORIZATION,
            axum::http::header::CONTENT_TYPE,
            axum::http::header::ACCEPT,
        ])
        .expose_headers([
            axum::http::header::CONTENT_LENGTH,
            axum::http::header::CONTENT_TYPE,
        ])
        .max_age(Duration::from_secs(86400)) // 24 hours
}

/// Request ID middleware
pub async fn request_id_middleware(
    mut request: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    let request_id = uuid::Uuid::new_v4();
    request.extensions_mut().insert(request_id);
    
    let mut response = next.run(request).await;
    response.headers_mut().insert(
        "X-Request-ID",
        request_id.to_string().parse().unwrap(),
    );
    
    Ok(response)
}

/// Security headers middleware
pub async fn security_headers_middleware(
    request: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    let mut response = next.run(request).await;
    let headers = response.headers_mut();
    
    // Security headers
    headers.insert("X-Content-Type-Options", "nosniff".parse().unwrap());
    headers.insert("X-Frame-Options", "DENY".parse().unwrap());
    headers.insert("X-XSS-Protection", "1; mode=block".parse().unwrap());
    headers.insert(
        "Strict-Transport-Security",
        "max-age=31536000; includeSubDomains".parse().unwrap(),
    );
    headers.insert(
        "Content-Security-Policy",
        "default-src 'self'".parse().unwrap(),
    );
    headers.insert("Referrer-Policy", "strict-origin-when-cross-origin".parse().unwrap());
    
    Ok(response)
}

/// API versioning middleware
pub async fn api_version_middleware(
    request: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    let path = request.uri().path();
    
    // Check if path starts with API version
    if !path.starts_with("/api/v") && !path.starts_with("/health") && !path.starts_with("/docs") {
        return Err(StatusCode::NOT_FOUND);
    }
    
    Ok(next.run(request).await)
}

/// Logging middleware
pub async fn logging_middleware(
    request: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    let method = request.method().clone();
    let uri = request.uri().clone();
    let start = std::time::Instant::now();
    
    let response = next.run(request).await;
    
    let duration = start.elapsed();
    let status = response.status();
    
    info!(
        method = %method,
        uri = %uri,
        status = %status.as_u16(),
        duration_ms = duration.as_millis(),
        "HTTP request completed"
    );
    
    Ok(response)
}

/// Compression middleware
pub fn compression_middleware() -> tower_http::compression::CompressionLayer {
    tower_http::compression::CompressionLayer::new()
        .gzip(true)
        .deflate(true)
        .br(true)
}

/// Request timeout middleware
pub fn timeout_middleware() -> tower_http::timeout::TimeoutLayer {
    tower_http::timeout::TimeoutLayer::new(Duration::from_secs(30))
}

/// Body size limit middleware
pub fn body_limit_middleware() -> tower_http::limit::RequestBodyLimitLayer {
    tower_http::limit::RequestBodyLimitLayer::new(1024 * 1024 * 10) // 10MB
}

// Helper functions

fn should_skip_auth(path: &str) -> bool {
    let public_paths = [
        "/health",
        "/api/v1/auth/login",
        "/api/v1/auth/register", 
        "/api/v1/auth/refresh",
        "/docs",
        "/metrics",
    ];
    
    public_paths.iter().any(|&p| path.starts_with(p))
}

fn extract_bearer_token(headers: &HeaderMap) -> Option<String> {
    let auth_header = headers.get("Authorization")?;
    let auth_str = auth_header.to_str().ok()?;
    
    if auth_str.starts_with("Bearer ") {
        Some(auth_str.strip_prefix("Bearer ")?.to_string())
    } else {
        None
    }
}

fn extract_client_ip(request: &Request) -> String {
    // Try to get real IP from headers (behind proxy)
    if let Some(forwarded_for) = request.headers().get("X-Forwarded-For") {
        if let Ok(forwarded_str) = forwarded_for.to_str() {
            if let Some(first_ip) = forwarded_str.split(',').next() {
                return first_ip.trim().to_string();
            }
        }
    }
    
    if let Some(real_ip) = request.headers().get("X-Real-IP") {
        if let Ok(ip_str) = real_ip.to_str() {
            return ip_str.to_string();
        }
    }
    
    // Fallback to connection info
    request
        .extensions()
        .get::<axum::extract::ConnectInfo<SocketAddr>>()
        .map(|connect_info| connect_info.0.ip().to_string())
        .unwrap_or_else(|| "unknown".to_string())
}

/// Enterprise API key validation middleware
pub async fn api_key_middleware(
    State(services): State<Arc<ApiServices>>,
    headers: HeaderMap,
    mut request: Request,
    next: Next,
) -> Result<Response, ApiErrorType> {
    // Check for API key in header
    let api_key = headers
        .get("X-API-Key")
        .and_then(|h| h.to_str().ok())
        .ok_or(ApiErrorType::Unauthorized)?;

    // Validate API key
    let api_key_info = services.auth_service()
        .validate_api_key(api_key)
        .await
        .map_err(|_| ApiErrorType::Unauthorized)?;

    // Add API key info to request
    request.extensions_mut().insert(api_key_info);

    Ok(next.run(request).await)
}

/// Quota checking middleware for enterprise users
pub async fn quota_middleware(
    State(services): State<Arc<ApiServices>>,
    request: Request,
    next: Next,
) -> Result<Response, ApiErrorType> {
    // Get user from request extensions
    if let Some(user_claims) = request.extensions().get::<crate::auth::UserClaims>() {
        // Check if user has exceeded their quota
        let quota_status = services.auth_service()
            .check_quota(&user_claims.user_id)
            .await
            .map_err(|_| ApiErrorType::InternalServerError)?;

        if quota_status.exceeded {
            return Err(ApiErrorType::QuotaExceeded);
        }

        // Increment usage counter
        services.auth_service()
            .increment_usage(&user_claims.user_id)
            .await
            .map_err(|_| ApiErrorType::InternalServerError)?;
    }

    Ok(next.run(request).await)
}

/// Health check middleware - bypass all other middleware for health checks
pub async fn health_check_middleware(
    request: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    if request.uri().path() == "/health" {
        // Skip all middleware for health checks
        return Ok(axum::response::Json(serde_json::json!({
            "status": "ok",
            "timestamp": chrono::Utc::now(),
            "version": env!("CARGO_PKG_VERSION")
        })).into_response());
    }
    
    Ok(next.run(request).await)
}