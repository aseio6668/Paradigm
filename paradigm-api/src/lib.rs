/// Enterprise REST API for Paradigm blockchain network
pub mod api;
pub mod auth;
pub mod config;
pub mod error;
pub mod middleware;
pub mod models;
pub mod routes;
pub mod services;
pub mod utils;
pub mod websocket;

use anyhow::Result;
use axum::Router;
use std::sync::Arc;
use tokio::net::TcpListener;
use tower::ServiceBuilder;
use tower_http::{
    cors::{Any, CorsLayer},
    trace::TraceLayer,
    timeout::TimeoutLayer,
    limit::RequestBodyLimitLayer,
};
use tracing::info;

use crate::config::ApiConfig;
use crate::middleware::{auth_middleware, rate_limit_middleware};
use crate::services::ApiServices;

pub const API_VERSION: &str = "v1";

/// Main API server
pub struct ApiServer {
    config: ApiConfig,
    services: Arc<ApiServices>,
}

impl ApiServer {
    pub async fn new(config: ApiConfig) -> Result<Self> {
        let services = Arc::new(ApiServices::new(&config).await?);
        
        Ok(Self {
            config,
            services,
        })
    }

    pub async fn start(&self) -> Result<()> {
        let app = self.create_router().await?;
        
        let addr = format!("{}:{}", self.config.host, self.config.port);
        let listener = TcpListener::bind(&addr).await?;
        
        info!("🚀 Paradigm API server starting on http://{}", addr);
        info!("📖 API documentation available at http://{}/docs", addr);
        info!("🔍 Health check available at http://{}/health", addr);
        
        axum::serve(listener, app).await?;
        
        Ok(())
    }

    async fn create_router(&self) -> Result<Router> {
        let cors = CorsLayer::new()
            .allow_origin(Any)
            .allow_methods(Any)
            .allow_headers(Any);

        let middleware_stack = ServiceBuilder::new()
            .layer(TraceLayer::new_for_http())
            .layer(cors)
            .layer(TimeoutLayer::new(std::time::Duration::from_secs(30)))
            .layer(RequestBodyLimitLayer::new(1024 * 1024 * 10)) // 10MB limit
            .layer(rate_limit_middleware());

        let app = Router::new()
            // Health and status endpoints
            .merge(routes::health::router())
            
            // Authentication endpoints
            .merge(routes::auth::router(self.services.clone()))
            
            // Core blockchain endpoints
            .merge(routes::blockchain::router(self.services.clone()))
            .merge(routes::transactions::router(self.services.clone()))
            .merge(routes::accounts::router(self.services.clone()))
            
            // AI and ML endpoints
            .merge(routes::ml_tasks::router(self.services.clone()))
            .merge(routes::governance::router(self.services.clone()))
            
            // Cross-chain endpoints
            .merge(routes::cross_chain::router(self.services.clone()))
            
            // Enterprise features
            .merge(routes::analytics::router(self.services.clone()))
            .merge(routes::webhooks::router(self.services.clone()))
            
            // WebSocket endpoints
            .merge(routes::websocket::router(self.services.clone()))
            
            // API documentation
            .merge(routes::docs::router())
            
            .layer(middleware_stack)
            .layer(axum::middleware::from_fn_with_state(
                self.services.clone(),
                auth_middleware,
            ));

        Ok(app)
    }
}

/// Initialize tracing and metrics
pub fn init_observability() -> Result<()> {
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "paradigm_api=info,tower_http=debug".into())
        )
        .init();

    // Initialize metrics
    metrics_exporter_prometheus::PrometheusBuilder::new()
        .install()?;

    Ok(())
}

/// API response types
pub use models::{
    ApiResponse, ApiError, PaginatedResponse, 
    CreateTransactionRequest, TransactionResponse,
    AccountResponse, BalanceResponse,
    MLTaskRequest, MLTaskResponse,
    CrossChainTransferRequest, CrossChainResponse,
};

/// Re-export commonly used types
pub use paradigm_core::{Address, Hash, Amount};
pub use paradigm_sdk::ParadigmClient;