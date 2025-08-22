use std::collections::HashMap;
use tokio::sync::RwLock;
use uuid::Uuid;
use chrono::{DateTime, Utc};

use crate::models::SubscriptionType;

/// Represents a WebSocket subscription
#[derive(Debug, Clone)]
pub struct Subscription {
    pub id: Uuid,
    pub connection_id: Uuid,
    pub subscription_type: SubscriptionType,
    pub filters: Option<serde_json::Value>,
    pub created_at: DateTime<Utc>,
}

/// Manages subscriptions for WebSocket connections
pub struct SubscriptionManager {
    subscriptions: RwLock<HashMap<Uuid, Subscription>>,
    connection_subscriptions: RwLock<HashMap<Uuid, Vec<Uuid>>>, // connection_id -> subscription_ids
    type_subscriptions: RwLock<HashMap<SubscriptionType, Vec<Uuid>>>, // subscription_type -> subscription_ids
}

impl SubscriptionManager {
    pub fn new() -> Self {
        Self {
            subscriptions: RwLock::new(HashMap::new()),
            connection_subscriptions: RwLock::new(HashMap::new()),
            type_subscriptions: RwLock::new(HashMap::new()),
        }
    }

    /// Add a new subscription
    pub async fn add_subscription(&self, subscription: Subscription) {
        let subscription_id = subscription.id;
        let connection_id = subscription.connection_id;
        let subscription_type = subscription.subscription_type.clone();

        // Store the subscription
        {
            let mut subscriptions = self.subscriptions.write().await;
            subscriptions.insert(subscription_id, subscription);
        }

        // Index by connection
        {
            let mut connection_subs = self.connection_subscriptions.write().await;
            connection_subs
                .entry(connection_id)
                .or_insert_with(Vec::new)
                .push(subscription_id);
        }

        // Index by type
        {
            let mut type_subs = self.type_subscriptions.write().await;
            type_subs
                .entry(subscription_type)
                .or_insert_with(Vec::new)
                .push(subscription_id);
        }

        tracing::info!("Added subscription {} for connection {}", subscription_id, connection_id);
    }

    /// Remove a subscription
    pub async fn remove_subscription(&self, subscription_id: &Uuid) {
        let subscription = {
            let mut subscriptions = self.subscriptions.write().await;
            subscriptions.remove(subscription_id)
        };

        if let Some(subscription) = subscription {
            // Remove from connection index
            {
                let mut connection_subs = self.connection_subscriptions.write().await;
                if let Some(subs) = connection_subs.get_mut(&subscription.connection_id) {
                    subs.retain(|id| id != subscription_id);
                    if subs.is_empty() {
                        connection_subs.remove(&subscription.connection_id);
                    }
                }
            }

            // Remove from type index
            {
                let mut type_subs = self.type_subscriptions.write().await;
                if let Some(subs) = type_subs.get_mut(&subscription.subscription_type) {
                    subs.retain(|id| id != subscription_id);
                    if subs.is_empty() {
                        type_subs.remove(&subscription.subscription_type);
                    }
                }
            }

            tracing::info!("Removed subscription {}", subscription_id);
        }
    }

    /// Remove all subscriptions for a connection
    pub async fn remove_connection_subscriptions(&self, connection_id: &Uuid) {
        let subscription_ids = {
            let mut connection_subs = self.connection_subscriptions.write().await;
            connection_subs.remove(connection_id).unwrap_or_default()
        };

        for subscription_id in subscription_ids {
            self.remove_subscription(&subscription_id).await;
        }

        tracing::info!("Removed all subscriptions for connection {}", connection_id);
    }

    /// Get all subscribers for a subscription type
    pub async fn get_subscribers(&self, subscription_type: &SubscriptionType) -> Vec<Uuid> {
        let type_subs = self.type_subscriptions.read().await;
        let subscription_ids = type_subs.get(subscription_type).cloned().unwrap_or_default();

        let subscriptions = self.subscriptions.read().await;
        subscription_ids
            .into_iter()
            .filter_map(|id| subscriptions.get(&id).map(|s| s.connection_id))
            .collect()
    }

    /// Get transaction subscribers (with optional address filter)
    pub async fn get_transaction_subscribers(&self) -> Vec<Uuid> {
        let mut subscribers = Vec::new();

        // Get all transaction subscriptions
        let type_subs = self.type_subscriptions.read().await;
        let subscription_ids = type_subs
            .iter()
            .filter_map(|(sub_type, ids)| {
                match sub_type {
                    SubscriptionType::Transactions { .. } => Some(ids.clone()),
                    SubscriptionType::All => Some(ids.clone()),
                    _ => None,
                }
            })
            .flatten()
            .collect::<Vec<_>>();

        let subscriptions = self.subscriptions.read().await;
        for subscription_id in subscription_ids {
            if let Some(subscription) = subscriptions.get(&subscription_id) {
                subscribers.push(subscription.connection_id);
            }
        }

        subscribers
    }

    /// Get balance subscribers for a specific address
    pub async fn get_balance_subscribers(&self, address: &paradigm_core::Address) -> Vec<Uuid> {
        let mut subscribers = Vec::new();

        let subscriptions = self.subscriptions.read().await;
        for subscription in subscriptions.values() {
            match &subscription.subscription_type {
                SubscriptionType::Balances { address: sub_address } => {
                    if sub_address == address {
                        subscribers.push(subscription.connection_id);
                    }
                }
                SubscriptionType::All => {
                    subscribers.push(subscription.connection_id);
                }
                _ => {}
            }
        }

        subscribers
    }

    /// Get price subscribers
    pub async fn get_price_subscribers(&self) -> Vec<Uuid> {
        let mut subscribers = Vec::new();

        let subscriptions = self.subscriptions.read().await;
        for subscription in subscriptions.values() {
            match &subscription.subscription_type {
                SubscriptionType::Prices { .. } => {
                    subscribers.push(subscription.connection_id);
                }
                SubscriptionType::All => {
                    subscribers.push(subscription.connection_id);
                }
                _ => {}
            }
        }

        subscribers
    }

    /// Get subscription count
    pub async fn subscription_count(&self) -> usize {
        self.subscriptions.read().await.len()
    }

    /// Get subscriptions for a connection
    pub async fn get_connection_subscriptions(&self, connection_id: &Uuid) -> Vec<Subscription> {
        let connection_subs = self.connection_subscriptions.read().await;
        let subscription_ids = connection_subs.get(connection_id).cloned().unwrap_or_default();

        let subscriptions = self.subscriptions.read().await;
        subscription_ids
            .into_iter()
            .filter_map(|id| subscriptions.get(&id).cloned())
            .collect()
    }

    /// Check if a subscription matches filters
    pub fn matches_filters(
        subscription: &Subscription,
        data: &serde_json::Value,
    ) -> bool {
        if let Some(filters) = &subscription.filters {
            // Implement filter matching logic based on subscription type
            match &subscription.subscription_type {
                SubscriptionType::Transactions { address } => {
                    if let Some(addr) = address {
                        // Check if transaction involves this address
                        if let Some(from) = data.get("from") {
                            if from.as_str() == Some(&crate::utils::format_address(addr)) {
                                return true;
                            }
                        }
                        if let Some(to) = data.get("to") {
                            if to.as_str() == Some(&crate::utils::format_address(addr)) {
                                return true;
                            }
                        }
                        return false;
                    }
                }
                SubscriptionType::Prices { assets } => {
                    if let Some(asset) = data.get("asset") {
                        if let Some(asset_str) = asset.as_str() {
                            return assets.contains(&asset_str.to_string());
                        }
                    }
                    return false;
                }
                _ => {}
            }
        }

        // If no filters or filters don't apply, allow the message
        true
    }
}

impl Default for SubscriptionManager {
    fn default() -> Self {
        Self::new()
    }
}