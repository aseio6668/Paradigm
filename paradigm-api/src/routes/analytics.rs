use axum::{routing::get, Router};
use std::sync::Arc;
use crate::services::ApiServices;

pub fn router(services: Arc<ApiServices>) -> Router {
    Router::new()
        .route("/analytics", get(|| async { "analytics endpoint" }))
        .with_state(services)
}