use std::sync::Arc;
use tokio::sync::mpsc;
use uuid::Uuid;

use crate::{
    models::{WebSocketMessage, SubscriptionRequest},
    services::ApiServices,
};
use super::manager::WebSocketManager;

/// WebSocket handler for processing messages
pub struct WebSocketHandler {
    manager: Arc<WebSocketManager>,
    services: Arc<ApiServices>,
}

impl WebSocketHandler {
    pub fn new(manager: Arc<WebSocketManager>, services: Arc<ApiServices>) -> Self {
        Self { manager, services }
    }

    /// Handle a new WebSocket connection
    pub async fn handle_connection(
        &self,
        connection_id: Uuid,
        sender: mpsc::UnboundedSender<WebSocketMessage>,
    ) {
        self.manager.register_connection(connection_id, sender).await;

        // Send welcome message
        let welcome_message = WebSocketMessage {
            message_type: crate::models::WebSocketMessageType::SystemAlert,
            data: serde_json::json!({
                "type": "welcome",
                "connection_id": connection_id,
                "message": "Connected to Paradigm API WebSocket",
                "api_version": crate::API_VERSION,
                "supported_subscriptions": [
                    "blocks",
                    "transactions",
                    "balances",
                    "ml_tasks",
                    "proposals",
                    "prices"
                ]
            }),
            timestamp: chrono::Utc::now(),
        };

        let _ = self.manager.send_to_connection(connection_id, welcome_message).await;
    }

    /// Handle connection disconnect
    pub async fn handle_disconnect(&self, connection_id: Uuid) {
        self.manager.unregister_connection(&connection_id).await;
    }

    /// Handle subscription request
    pub async fn handle_subscription(
        &self,
        connection_id: Uuid,
        request: SubscriptionRequest,
    ) -> Result<(), String> {
        // Validate subscription request
        self.validate_subscription_request(&request)?;

        // Handle the subscription
        self.manager.handle_subscription(connection_id, request).await?;

        tracing::info!("Processed subscription for connection {}", connection_id);
        Ok(())
    }

    /// Handle ping message
    pub async fn handle_ping(
        &self,
        connection_id: Uuid,
    ) -> Result<(), String> {
        let pong_message = WebSocketMessage {
            message_type: crate::models::WebSocketMessageType::SystemAlert,
            data: serde_json::json!({
                "type": "pong",
                "timestamp": chrono::Utc::now()
            }),
            timestamp: chrono::Utc::now(),
        };

        self.manager.send_to_connection(connection_id, pong_message).await?;
        Ok(())
    }

    /// Validate subscription request
    fn validate_subscription_request(&self, request: &SubscriptionRequest) -> Result<(), String> {
        match &request.subscription_type {
            crate::models::SubscriptionType::Transactions { address } => {
                if let Some(addr) = address {
                    // Validate address format
                    crate::utils::parse_address(&crate::utils::format_address(addr))
                        .map_err(|e| format!("Invalid address: {}", e))?;
                }
            }
            crate::models::SubscriptionType::Balances { address } => {
                // Validate address format
                crate::utils::parse_address(&crate::utils::format_address(address))
                    .map_err(|e| format!("Invalid address: {}", e))?;
            }
            crate::models::SubscriptionType::Prices { assets } => {
                if assets.is_empty() {
                    return Err("At least one asset must be specified for price subscription".to_string());
                }
                
                // Validate asset symbols
                for asset in assets {
                    if asset.is_empty() || asset.len() > 10 {
                        return Err(format!("Invalid asset symbol: {}", asset));
                    }
                }
            }
            _ => {} // Other types don't need validation
        }

        Ok(())
    }

    /// Get connection statistics
    pub async fn get_stats(&self) -> serde_json::Value {
        serde_json::json!({
            "connections": self.manager.connection_count().await,
            "subscriptions": self.manager.subscription_count().await,
            "uptime_seconds": std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs()
        })
    }
}