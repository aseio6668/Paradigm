/// Service layer modules
pub mod auth;
pub mod blockchain;
pub mod cross_chain;
pub mod governance;
pub mod ml_tasks;
pub mod analytics;
pub mod webhooks;
pub mod websocket;

use anyhow::Result;
use std::sync::Arc;

use crate::config::ApiConfig;

pub use auth::AuthService;
pub use blockchain::BlockchainService;
pub use cross_chain::CrossChainService;
pub use governance::GovernanceService;
pub use ml_tasks::MLTaskService;
pub use analytics::AnalyticsService;
pub use webhooks::WebhookService;
pub use websocket::WebSocketService;

/// Container for all API services
#[derive(Clone)]
pub struct ApiServices {
    auth_service: Arc<AuthService>,
    blockchain_service: Arc<BlockchainService>,
    cross_chain_service: Arc<CrossChainService>,
    governance_service: Arc<GovernanceService>,
    ml_task_service: Arc<MLTaskService>,
    analytics_service: Arc<AnalyticsService>,
    webhook_service: Arc<WebhookService>,
    websocket_service: Arc<WebSocketService>,
}

impl ApiServices {
    pub async fn new(config: &ApiConfig) -> Result<Self> {
        let auth_service = Arc::new(AuthService::new(config).await?);
        let blockchain_service = Arc::new(BlockchainService::new(config).await?);
        let cross_chain_service = Arc::new(CrossChainService::new(config).await?);
        let governance_service = Arc::new(GovernanceService::new(config).await?);
        let ml_task_service = Arc::new(MLTaskService::new(config).await?);
        let analytics_service = Arc::new(AnalyticsService::new(config).await?);
        let webhook_service = Arc::new(WebhookService::new(config).await?);
        let websocket_service = Arc::new(WebSocketService::new(config).await?);

        Ok(Self {
            auth_service,
            blockchain_service,
            cross_chain_service,
            governance_service,
            ml_task_service,
            analytics_service,
            webhook_service,
            websocket_service,
        })
    }

    pub fn auth_service(&self) -> &AuthService {
        &self.auth_service
    }

    pub fn blockchain_service(&self) -> &BlockchainService {
        &self.blockchain_service
    }

    pub fn cross_chain_service(&self) -> &CrossChainService {
        &self.cross_chain_service
    }

    pub fn governance_service(&self) -> &GovernanceService {
        &self.governance_service
    }

    pub fn ml_task_service(&self) -> &MLTaskService {
        &self.ml_task_service
    }

    pub fn analytics_service(&self) -> &AnalyticsService {
        &self.analytics_service
    }

    pub fn webhook_service(&self) -> &WebhookService {
        &self.webhook_service
    }

    pub fn websocket_service(&self) -> &WebSocketService {
        &self.websocket_service
    }
}