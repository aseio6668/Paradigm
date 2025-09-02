use anyhow::Result;
use paradigm_core::{transaction::Transaction, Address};
use reqwest::Client;
use serde_json::json;

/// Network client for communicating with Paradigm nodes
pub struct NetworkClient {
    client: Client,
    node_url: String,
    connected: bool,
}

impl NetworkClient {
    pub async fn new(node_address: &str) -> Result<Self> {
        let client = Client::new();
        let node_url = if node_address.starts_with("http") {
            node_address.to_string()
        } else {
            format!("http://{}", node_address)
        };

        Ok(NetworkClient {
            client,
            node_url,
            connected: false,
        })
    }

    pub async fn connect(&mut self) -> Result<()> {
        // Try to ping the node
        let response = self
            .client
            .get(&format!("{}/health", self.node_url))
            .send()
            .await?;

        if response.status().is_success() {
            self.connected = true;
            Ok(())
        } else {
            Err(anyhow::anyhow!("Failed to connect to node"))
        }
    }

    pub async fn disconnect(&mut self) -> Result<()> {
        self.connected = false;
        Ok(())
    }

    pub fn is_connected(&self) -> bool {
        self.connected
    }

    pub async fn get_balance(&self, address: &Address) -> Result<u64> {
        if !self.connected {
            return Err(anyhow::anyhow!("Not connected to network"));
        }

        let response = self
            .client
            .get(&format!(
                "{}/api/v1/balance/{}",
                self.node_url,
                address.to_string()
            ))
            .send()
            .await?;

        if response.status().is_success() {
            let balance_response: serde_json::Value = response.json().await?;
            Ok(balance_response["balance"].as_u64().unwrap_or(0))
        } else {
            Err(anyhow::anyhow!("Failed to get balance"))
        }
    }

    pub async fn get_transactions(&self, address: &Address) -> Result<Vec<Transaction>> {
        if !self.connected {
            return Err(anyhow::anyhow!("Not connected to network"));
        }

        let response = self
            .client
            .get(&format!(
                "{}/api/v1/transactions/{}",
                self.node_url,
                address.to_string()
            ))
            .send()
            .await?;

        if response.status().is_success() {
            let tx_response: serde_json::Value = response.json().await?;
            // In a real implementation, we'd deserialize the actual transactions
            // For now, return empty vector
            Ok(Vec::new())
        } else {
            Err(anyhow::anyhow!("Failed to get transactions"))
        }
    }

    pub async fn broadcast_transaction(&self, transaction: &Transaction) -> Result<String> {
        if !self.connected {
            return Err(anyhow::anyhow!("Not connected to network"));
        }

        let tx_data = json!({
            "from": transaction.from.to_string(),
            "to": transaction.to.to_string(),
            "amount": transaction.amount,
            "fee": transaction.fee,
            "signature": hex::encode(&transaction.signature),
            "nonce": transaction.nonce
        });

        let response = self
            .client
            .post(&format!("{}/api/v1/transaction", self.node_url))
            .json(&tx_data)
            .send()
            .await?;

        if response.status().is_success() {
            let result: serde_json::Value = response.json().await?;
            Ok(result["transaction_id"]
                .as_str()
                .unwrap_or("unknown")
                .to_string())
        } else {
            let error_text = response.text().await.unwrap_or_default();
            Err(anyhow::anyhow!(
                "Failed to broadcast transaction: {}",
                error_text
            ))
        }
    }

    pub async fn get_network_stats(&self) -> Result<serde_json::Value> {
        if !self.connected {
            return Err(anyhow::anyhow!("Not connected to network"));
        }

        let response = self
            .client
            .get(&format!("{}/api/v1/stats", self.node_url))
            .send()
            .await?;

        if response.status().is_success() {
            Ok(response.json().await?)
        } else {
            Err(anyhow::anyhow!("Failed to get network stats"))
        }
    }

    pub async fn get_comprehensive_network_status(&self) -> Result<serde_json::Value> {
        if !self.connected {
            return Err(anyhow::anyhow!("Not connected to network"));
        }

        // Get health status
        let health_response = self
            .client
            .get(&format!("{}/health", self.node_url))
            .send()
            .await?;

        let health_data: serde_json::Value = if health_response.status().is_success() {
            health_response.json().await?
        } else {
            serde_json::json!({"status": "unhealthy", "error": "Health check failed"})
        };

        // Try to get additional network information
        let tasks_response = self
            .client
            .get(&format!("{}/api/tasks/available", self.node_url))
            .send()
            .await;

        let network_active = tasks_response.is_ok();

        let tasks_data = if let Ok(response) = tasks_response {
            if response.status().is_success() {
                response.json().await.unwrap_or(serde_json::json!({}))
            } else {
                serde_json::json!({"error": "Tasks endpoint unavailable"})
            }
        } else {
            serde_json::json!({"error": "Cannot reach tasks endpoint"})
        };

        // Compile comprehensive status
        Ok(serde_json::json!({
            "timestamp": chrono::Utc::now().to_rfc3339(),
            "node_url": self.node_url,
            "health": health_data,
            "network_active": network_active,
            "tasks": {
                "available_count": tasks_data.get("available_tasks").and_then(|t| t.as_array()).map(|a| a.len()).unwrap_or(0),
                "queue_size": tasks_data.get("queue_size").and_then(|v| v.as_u64()).unwrap_or(0),
                "estimated_reward": tasks_data.get("estimated_reward").and_then(|v| v.as_u64()).unwrap_or(0)
            },
            "peer_info": {
                "peer_count": health_data.get("peers_count").and_then(|v| v.as_u64()).unwrap_or(0),
                "block_height": health_data.get("block_height").and_then(|v| v.as_u64()).unwrap_or(0),
                "network_status": health_data.get("network_status").and_then(|v| v.as_str()).unwrap_or("unknown"),
                "is_synchronized": health_data.get("peers_count").and_then(|v| v.as_u64()).unwrap_or(0) > 0
            }
        }))
    }
}
