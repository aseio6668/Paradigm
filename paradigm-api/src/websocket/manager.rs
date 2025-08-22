use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{mpsc, RwLock};
use uuid::Uuid;

use crate::models::{WebSocketMessage, SubscriptionRequest, SubscriptionType};
use super::subscription::{Subscription, SubscriptionManager};

/// Manages WebSocket connections and subscriptions
#[derive(Clone)]
pub struct WebSocketManager {
    connections: Arc<RwLock<HashMap<Uuid, mpsc::UnboundedSender<WebSocketMessage>>>>,
    subscription_manager: Arc<SubscriptionManager>,
}

impl WebSocketManager {
    pub fn new() -> Self {
        Self {
            connections: Arc::new(RwLock::new(HashMap::new())),
            subscription_manager: Arc::new(SubscriptionManager::new()),
        }
    }

    /// Register a new WebSocket connection
    pub async fn register_connection(
        &self,
        connection_id: Uuid,
        sender: mpsc::UnboundedSender<WebSocketMessage>,
    ) {
        let mut connections = self.connections.write().await;
        connections.insert(connection_id, sender);
        tracing::info!("Registered WebSocket connection: {}", connection_id);
    }

    /// Unregister a WebSocket connection
    pub async fn unregister_connection(&self, connection_id: &Uuid) {
        let mut connections = self.connections.write().await;
        connections.remove(connection_id);
        
        // Remove all subscriptions for this connection
        self.subscription_manager.remove_connection_subscriptions(connection_id).await;
        
        tracing::info!("Unregistered WebSocket connection: {}", connection_id);
    }

    /// Handle a subscription request
    pub async fn handle_subscription(
        &self,
        connection_id: Uuid,
        request: SubscriptionRequest,
    ) -> Result<(), String> {
        let subscription = Subscription {
            id: Uuid::new_v4(),
            connection_id,
            subscription_type: request.subscription_type,
            filters: request.filters,
            created_at: chrono::Utc::now(),
        };

        self.subscription_manager.add_subscription(subscription).await;
        
        // Send confirmation message
        let confirmation = WebSocketMessage {
            message_type: crate::models::WebSocketMessageType::SystemAlert,
            data: serde_json::json!({
                "type": "subscription_confirmed",
                "subscription_id": subscription.id,
                "message": "Subscription created successfully"
            }),
            timestamp: chrono::Utc::now(),
        };

        self.send_to_connection(connection_id, confirmation).await?;

        Ok(())
    }

    /// Send message to a specific connection
    pub async fn send_to_connection(
        &self,
        connection_id: Uuid,
        message: WebSocketMessage,
    ) -> Result<(), String> {
        let connections = self.connections.read().await;
        if let Some(sender) = connections.get(&connection_id) {
            sender.send(message)
                .map_err(|e| format!("Failed to send message: {}", e))?;
        }
        Ok(())
    }

    /// Broadcast message to all connections
    pub async fn broadcast(&self, message: WebSocketMessage) -> Result<(), String> {
        let connections = self.connections.read().await;
        let mut failed_connections = Vec::new();

        for (connection_id, sender) in connections.iter() {
            if sender.send(message.clone()).is_err() {
                failed_connections.push(*connection_id);
            }
        }

        // Clean up failed connections
        drop(connections);
        for connection_id in failed_connections {
            self.unregister_connection(&connection_id).await;
        }

        Ok(())
    }

    /// Broadcast to subscribers of a specific type
    pub async fn broadcast_to_subscribers(
        &self,
        subscription_type: &SubscriptionType,
        message: WebSocketMessage,
    ) -> Result<(), String> {
        let subscribers = self.subscription_manager
            .get_subscribers(subscription_type)
            .await;

        for connection_id in subscribers {
            let _ = self.send_to_connection(connection_id, message.clone()).await;
        }

        Ok(())
    }

    /// Get connection count
    pub async fn connection_count(&self) -> usize {
        self.connections.read().await.len()
    }

    /// Get subscription count
    pub async fn subscription_count(&self) -> usize {
        self.subscription_manager.subscription_count().await
    }

    /// Notify about new block
    pub async fn notify_new_block(&self, block_data: serde_json::Value) -> Result<(), String> {
        let message = WebSocketMessage {
            message_type: crate::models::WebSocketMessageType::BlockUpdate,
            data: block_data,
            timestamp: chrono::Utc::now(),
        };

        self.broadcast_to_subscribers(&SubscriptionType::Blocks, message).await
    }

    /// Notify about new transaction
    pub async fn notify_new_transaction(
        &self,
        transaction_data: serde_json::Value,
    ) -> Result<(), String> {
        let message = WebSocketMessage {
            message_type: crate::models::WebSocketMessageType::TransactionUpdate,
            data: transaction_data,
            timestamp: chrono::Utc::now(),
        };

        // Broadcast to all transaction subscribers
        let subscribers = self.subscription_manager
            .get_transaction_subscribers()
            .await;

        for connection_id in subscribers {
            let _ = self.send_to_connection(connection_id, message.clone()).await;
        }

        Ok(())
    }

    /// Notify about balance update
    pub async fn notify_balance_update(
        &self,
        address: &paradigm_core::Address,
        balance_data: serde_json::Value,
    ) -> Result<(), String> {
        let message = WebSocketMessage {
            message_type: crate::models::WebSocketMessageType::BalanceUpdate,
            data: balance_data,
            timestamp: chrono::Utc::now(),
        };

        // Get subscribers for this specific address
        let subscribers = self.subscription_manager
            .get_balance_subscribers(address)
            .await;

        for connection_id in subscribers {
            let _ = self.send_to_connection(connection_id, message.clone()).await;
        }

        Ok(())
    }

    /// Notify about ML task update
    pub async fn notify_ml_task_update(&self, task_data: serde_json::Value) -> Result<(), String> {
        let message = WebSocketMessage {
            message_type: crate::models::WebSocketMessageType::MLTaskUpdate,
            data: task_data,
            timestamp: chrono::Utc::now(),
        };

        self.broadcast_to_subscribers(&SubscriptionType::MLTasks, message).await
    }

    /// Notify about price update
    pub async fn notify_price_update(&self, price_data: serde_json::Value) -> Result<(), String> {
        let message = WebSocketMessage {
            message_type: crate::models::WebSocketMessageType::PriceUpdate,
            data: price_data,
            timestamp: chrono::Utc::now(),
        };

        // Get price subscribers
        let subscribers = self.subscription_manager
            .get_price_subscribers()
            .await;

        for connection_id in subscribers {
            let _ = self.send_to_connection(connection_id, message.clone()).await;
        }

        Ok(())
    }

    /// Send system alert
    pub async fn send_system_alert(&self, alert_data: serde_json::Value) -> Result<(), String> {
        let message = WebSocketMessage {
            message_type: crate::models::WebSocketMessageType::SystemAlert,
            data: alert_data,
            timestamp: chrono::Utc::now(),
        };

        self.broadcast(message).await
    }
}

impl Default for WebSocketManager {
    fn default() -> Self {
        Self::new()
    }
}