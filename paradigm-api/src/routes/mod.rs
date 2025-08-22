/// API route modules
pub mod auth;
pub mod accounts;
pub mod analytics;
pub mod blockchain;
pub mod cross_chain;
pub mod docs;
pub mod governance;
pub mod health;
pub mod ml_tasks;
pub mod transactions;
pub mod webhooks;
pub mod websocket;

use axum::Router;
use std::sync::Arc;
use crate::services::ApiServices;

/// Create the complete API router
pub fn create_api_router(services: Arc<ApiServices>) -> Router {
    Router::new()
        .nest("/api/v1", api_v1_router(services))
}

/// API v1 routes
fn api_v1_router(services: Arc<ApiServices>) -> Router {
    Router::new()
        .merge(auth::router(services.clone()))
        .merge(accounts::router(services.clone()))
        .merge(analytics::router(services.clone()))
        .merge(blockchain::router(services.clone()))
        .merge(cross_chain::router(services.clone()))
        .merge(governance::router(services.clone()))
        .merge(ml_tasks::router(services.clone()))
        .merge(transactions::router(services.clone()))
        .merge(webhooks::router(services.clone()))
        .merge(websocket::router(services.clone()))
}