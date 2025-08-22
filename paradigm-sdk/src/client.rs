//! Paradigm network client implementation
//!
//! This module provides the main client interface for interacting with the Paradigm blockchain.
//! It handles connection management, transaction submission, and data retrieval.

use crate::error::{ErrorExt, ParadigmError, Result};
use crate::types::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, SystemTime};
use tokio::sync::RwLock;
use url::Url;

/// Configuration for the Paradigm client
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClientConfig {
    /// RPC endpoint URLs
    pub rpc_endpoints: Vec<String>,

    /// WebSocket endpoint URLs
    pub ws_endpoints: Vec<String>,

    /// Chain ID to connect to
    pub chain_id: u64,

    /// Request timeout in milliseconds
    pub timeout_ms: u64,

    /// Maximum number of retries for failed requests
    pub max_retries: u32,

    /// Connection pool size
    pub connection_pool_size: usize,

    /// Enable request caching
    pub enable_caching: bool,

    /// Cache TTL in seconds
    pub cache_ttl_seconds: u64,

    /// API key for authenticated endpoints
    pub api_key: Option<String>,

    /// Custom headers for requests
    pub custom_headers: HashMap<String, String>,
}

impl Default for ClientConfig {
    fn default() -> Self {
        Self {
            rpc_endpoints: vec!["https://rpc.paradigm.network".to_string()],
            ws_endpoints: vec!["wss://ws.paradigm.network".to_string()],
            chain_id: crate::MAINNET_CHAIN_ID,
            timeout_ms: 30000,
            max_retries: 3,
            connection_pool_size: 10,
            enable_caching: true,
            cache_ttl_seconds: 60,
            api_key: None,
            custom_headers: HashMap::new(),
        }
    }
}

/// Connection status for RPC endpoints
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ConnectionStatus {
    Connected,
    Connecting,
    Disconnected,
    Error(String),
}

/// RPC endpoint information
#[derive(Debug, Clone)]
pub struct RpcEndpoint {
    pub url: String,
    pub status: ConnectionStatus,
    pub last_seen: SystemTime,
    pub latency_ms: Option<u64>,
    pub error_count: u64,
    pub request_count: u64,
}

/// Request cache entry
#[derive(Debug, Clone)]
struct CacheEntry {
    data: serde_json::Value,
    expires_at: SystemTime,
}

/// Main Paradigm client for interacting with the blockchain
#[derive(Debug)]
pub struct ParadigmClient {
    config: ClientConfig,
    http_client: reqwest::Client,
    endpoints: Arc<RwLock<Vec<RpcEndpoint>>>,
    cache: Arc<RwLock<HashMap<String, CacheEntry>>>,
    current_endpoint_index: Arc<RwLock<usize>>,
}

impl ParadigmClient {
    /// Create a new Paradigm client with default configuration
    pub fn new() -> Result<Self> {
        Self::with_config(ClientConfig::default())
    }

    /// Create a new Paradigm client with custom configuration
    pub fn with_config(config: ClientConfig) -> Result<Self> {
        let mut http_client_builder = reqwest::Client::builder()
            .timeout(Duration::from_millis(config.timeout_ms))
            .pool_max_idle_per_host(config.connection_pool_size);

        // Add custom headers
        let mut headers = reqwest::header::HeaderMap::new();
        for (key, value) in &config.custom_headers {
            headers.insert(
                reqwest::header::HeaderName::from_bytes(key.as_bytes()).map_err(|e| {
                    ParadigmError::Config(format!("Invalid header name '{}': {}", key, e))
                })?,
                reqwest::header::HeaderValue::from_str(value).map_err(|e| {
                    ParadigmError::Config(format!("Invalid header value '{}': {}", value, e))
                })?,
            );
        }

        if let Some(api_key) = &config.api_key {
            headers.insert(
                "Authorization",
                reqwest::header::HeaderValue::from_str(&format!("Bearer {}", api_key))
                    .map_err(|e| ParadigmError::Config(format!("Invalid API key: {}", e)))?,
            );
        }

        http_client_builder = http_client_builder.default_headers(headers);

        let http_client = http_client_builder
            .build()
            .map_err(|e| ParadigmError::Config(format!("Failed to create HTTP client: {}", e)))?;

        // Initialize endpoints
        let endpoints = config
            .rpc_endpoints
            .iter()
            .map(|url| RpcEndpoint {
                url: url.clone(),
                status: ConnectionStatus::Disconnected,
                last_seen: SystemTime::now(),
                latency_ms: None,
                error_count: 0,
                request_count: 0,
            })
            .collect();

        Ok(Self {
            config,
            http_client,
            endpoints: Arc::new(RwLock::new(endpoints)),
            cache: Arc::new(RwLock::new(HashMap::new())),
            current_endpoint_index: Arc::new(RwLock::new(0)),
        })
    }

    /// Get current chain ID
    pub fn chain_id(&self) -> u64 {
        self.config.chain_id
    }

    /// Get client configuration
    pub fn config(&self) -> &ClientConfig {
        &self.config
    }

    /// Check connection status to all endpoints
    pub async fn check_connection(&self) -> Result<Vec<RpcEndpoint>> {
        let mut endpoints = self.endpoints.write().await;

        for endpoint in endpoints.iter_mut() {
            let start_time = SystemTime::now();

            match self.make_request(&endpoint.url, "eth_chainId", &[]).await {
                Ok(_) => {
                    endpoint.status = ConnectionStatus::Connected;
                    endpoint.last_seen = SystemTime::now();
                    if let Ok(elapsed) = start_time.elapsed() {
                        endpoint.latency_ms = Some(elapsed.as_millis() as u64);
                    }
                }
                Err(e) => {
                    endpoint.status = ConnectionStatus::Error(e.to_string());
                    endpoint.error_count += 1;
                }
            }
        }

        Ok(endpoints.clone())
    }

    /// Get the best available endpoint based on latency and error rate
    pub async fn get_best_endpoint(&self) -> Result<String> {
        let endpoints = self.endpoints.read().await;

        let connected_endpoints: Vec<&RpcEndpoint> = endpoints
            .iter()
            .filter(|e| e.status == ConnectionStatus::Connected)
            .collect();

        if connected_endpoints.is_empty() {
            return Err(ParadigmError::Network(
                "No connected endpoints available".to_string(),
            ));
        }

        // Find endpoint with lowest latency and error rate
        let best_endpoint = connected_endpoints
            .into_iter()
            .min_by(|a, b| {
                let a_score = a.latency_ms.unwrap_or(u64::MAX) + (a.error_count * 1000);
                let b_score = b.latency_ms.unwrap_or(u64::MAX) + (b.error_count * 1000);
                a_score.cmp(&b_score)
            })
            .ok_or_else(|| ParadigmError::Network("No suitable endpoint found".to_string()))?;

        Ok(best_endpoint.url.clone())
    }

    /// Make a JSON-RPC request
    async fn make_request(
        &self,
        url: &str,
        method: &str,
        params: &[serde_json::Value],
    ) -> Result<serde_json::Value> {
        let request_body = serde_json::json!({
            "jsonrpc": "2.0",
            "method": method,
            "params": params,
            "id": 1
        });

        let response = self
            .http_client
            .post(url)
            .json(&request_body)
            .send()
            .await
            .map_err(|e| ParadigmError::Network(format!("Request failed: {}", e)))?;

        if !response.status().is_success() {
            return Err(ParadigmError::Rpc(format!(
                "HTTP error: {}",
                response.status()
            )));
        }

        let response_body: serde_json::Value = response
            .json()
            .await
            .map_err(|e| ParadigmError::Rpc(format!("Failed to parse response: {}", e)))?;

        if let Some(error) = response_body.get("error") {
            return Err(ParadigmError::Rpc(format!("RPC error: {}", error)));
        }

        response_body
            .get("result")
            .cloned()
            .ok_or_else(|| ParadigmError::Rpc("Missing result field".to_string()))
    }

    /// Make a cached request
    async fn make_cached_request(
        &self,
        method: &str,
        params: &[serde_json::Value],
    ) -> Result<serde_json::Value> {
        let cache_key = format!(
            "{}:{}",
            method,
            serde_json::to_string(params).unwrap_or_default()
        );

        // Check cache first
        if self.config.enable_caching {
            let cache = self.cache.read().await;
            if let Some(entry) = cache.get(&cache_key) {
                if entry.expires_at > SystemTime::now() {
                    return Ok(entry.data.clone());
                }
            }
        }

        // Make request to best endpoint
        let endpoint_url = self.get_best_endpoint().await?;
        let result = self.make_request(&endpoint_url, method, params).await?;

        // Cache the result
        if self.config.enable_caching {
            let mut cache = self.cache.write().await;
            cache.insert(
                cache_key,
                CacheEntry {
                    data: result.clone(),
                    expires_at: SystemTime::now()
                        + Duration::from_secs(self.config.cache_ttl_seconds),
                },
            );
        }

        Ok(result)
    }

    /// Get current block number
    pub async fn get_block_number(&self) -> Result<u64> {
        let result = self.make_cached_request("eth_blockNumber", &[]).await?;
        let block_number_str = result
            .as_str()
            .ok_or_else(|| ParadigmError::Rpc("Invalid block number format".to_string()))?;

        u64::from_str_radix(block_number_str.trim_start_matches("0x"), 16)
            .map_err(|e| ParadigmError::Rpc(format!("Failed to parse block number: {}", e)))
    }

    /// Get block by number
    pub async fn get_block(&self, block_number: BlockNumber) -> Result<Option<Block>> {
        let block_param = match block_number {
            BlockNumber::Number(n) => serde_json::Value::String(format!("0x{:x}", n)),
            BlockNumber::Latest => serde_json::Value::String("latest".to_string()),
            BlockNumber::Earliest => serde_json::Value::String("earliest".to_string()),
            BlockNumber::Pending => serde_json::Value::String("pending".to_string()),
        };

        let result = self
            .make_cached_request(
                "eth_getBlockByNumber",
                &[block_param, serde_json::Value::Bool(true)],
            )
            .await?;

        if result.is_null() {
            return Ok(None);
        }

        let block: Block = serde_json::from_value(result).map_err(|e| {
            ParadigmError::Serialization(format!("Failed to deserialize block: {}", e))
        })?;

        Ok(Some(block))
    }

    /// Get block by hash
    pub async fn get_block_by_hash(&self, hash: &Hash) -> Result<Option<Block>> {
        let result = self
            .make_cached_request(
                "eth_getBlockByHash",
                &[
                    serde_json::Value::String(hash.to_string()),
                    serde_json::Value::Bool(true),
                ],
            )
            .await?;

        if result.is_null() {
            return Ok(None);
        }

        let block: Block = serde_json::from_value(result).map_err(|e| {
            ParadigmError::Serialization(format!("Failed to deserialize block: {}", e))
        })?;

        Ok(Some(block))
    }

    /// Get transaction by hash
    pub async fn get_transaction(&self, hash: &Hash) -> Result<Option<Transaction>> {
        let result = self
            .make_cached_request(
                "eth_getTransactionByHash",
                &[serde_json::Value::String(hash.to_string())],
            )
            .await?;

        if result.is_null() {
            return Ok(None);
        }

        let transaction: Transaction = serde_json::from_value(result).map_err(|e| {
            ParadigmError::Serialization(format!("Failed to deserialize transaction: {}", e))
        })?;

        Ok(Some(transaction))
    }

    /// Get transaction receipt
    pub async fn get_transaction_receipt(&self, hash: &Hash) -> Result<Option<TransactionReceipt>> {
        let result = self
            .make_cached_request(
                "eth_getTransactionReceipt",
                &[serde_json::Value::String(hash.to_string())],
            )
            .await?;

        if result.is_null() {
            return Ok(None);
        }

        let receipt: TransactionReceipt = serde_json::from_value(result).map_err(|e| {
            ParadigmError::Serialization(format!("Failed to deserialize receipt: {}", e))
        })?;

        Ok(Some(receipt))
    }

    /// Send raw transaction
    pub async fn send_raw_transaction(&self, data: &[u8]) -> Result<Hash> {
        let hex_data = format!("0x{}", hex::encode(data));
        let result = self
            .make_request(
                &self.get_best_endpoint().await?,
                "eth_sendRawTransaction",
                &[serde_json::Value::String(hex_data)],
            )
            .await?;

        let hash_str = result
            .as_str()
            .ok_or_else(|| ParadigmError::Rpc("Invalid transaction hash format".to_string()))?;

        Hash::from_hex(hash_str)
            .map_err(|e| ParadigmError::Rpc(format!("Failed to parse transaction hash: {}", e)))
    }

    /// Get account balance
    pub async fn get_balance(&self, address: &Address, block: BlockNumber) -> Result<Amount> {
        let block_param = match block {
            BlockNumber::Number(n) => serde_json::Value::String(format!("0x{:x}", n)),
            BlockNumber::Latest => serde_json::Value::String("latest".to_string()),
            BlockNumber::Earliest => serde_json::Value::String("earliest".to_string()),
            BlockNumber::Pending => serde_json::Value::String("pending".to_string()),
        };

        let result = self
            .make_cached_request(
                "eth_getBalance",
                &[serde_json::Value::String(address.to_string()), block_param],
            )
            .await?;

        let balance_str = result
            .as_str()
            .ok_or_else(|| ParadigmError::Rpc("Invalid balance format".to_string()))?;

        let balance_u64 = u64::from_str_radix(balance_str.trim_start_matches("0x"), 16)
            .map_err(|e| ParadigmError::Rpc(format!("Failed to parse balance: {}", e)))?;

        Ok(Amount::from_wei(balance_u64))
    }

    /// Get account nonce
    pub async fn get_nonce(&self, address: &Address, block: BlockNumber) -> Result<u64> {
        let block_param = match block {
            BlockNumber::Number(n) => serde_json::Value::String(format!("0x{:x}", n)),
            BlockNumber::Latest => serde_json::Value::String("latest".to_string()),
            BlockNumber::Earliest => serde_json::Value::String("earliest".to_string()),
            BlockNumber::Pending => serde_json::Value::String("pending".to_string()),
        };

        let result = self
            .make_cached_request(
                "eth_getTransactionCount",
                &[serde_json::Value::String(address.to_string()), block_param],
            )
            .await?;

        let nonce_str = result
            .as_str()
            .ok_or_else(|| ParadigmError::Rpc("Invalid nonce format".to_string()))?;

        u64::from_str_radix(nonce_str.trim_start_matches("0x"), 16)
            .map_err(|e| ParadigmError::Rpc(format!("Failed to parse nonce: {}", e)))
    }

    /// Estimate gas for transaction
    pub async fn estimate_gas(&self, transaction: &Transaction) -> Result<u64> {
        let tx_object = serde_json::json!({
            "from": transaction.from.to_string(),
            "to": transaction.to.as_ref().map(|addr| addr.to_string()),
            "value": format!("0x{:x}", transaction.value.wei()),
            "data": format!("0x{}", hex::encode(&transaction.input))
        });

        let result = self
            .make_request(
                &self.get_best_endpoint().await?,
                "eth_estimateGas",
                &[tx_object],
            )
            .await?;

        let gas_str = result.as_str().ok_or_else(|| {
            ParadigmError::GasEstimation("Invalid gas estimate format".to_string())
        })?;

        u64::from_str_radix(gas_str.trim_start_matches("0x"), 16).map_err(|e| {
            ParadigmError::GasEstimation(format!("Failed to parse gas estimate: {}", e))
        })
    }

    /// Get current gas price
    pub async fn get_gas_price(&self) -> Result<Amount> {
        let result = self.make_cached_request("eth_gasPrice", &[]).await?;

        let price_str = result
            .as_str()
            .ok_or_else(|| ParadigmError::Rpc("Invalid gas price format".to_string()))?;

        let price_u64 = u64::from_str_radix(price_str.trim_start_matches("0x"), 16)
            .map_err(|e| ParadigmError::Rpc(format!("Failed to parse gas price: {}", e)))?;

        Ok(Amount::from_wei(price_u64))
    }

    /// Get network information
    pub async fn get_network_info(&self) -> Result<NetworkInfo> {
        let chain_id = self.make_cached_request("eth_chainId", &[]).await?;
        let block_number = self.get_block_number().await?;
        let gas_price = self.get_gas_price().await?;

        let chain_id_str = chain_id
            .as_str()
            .ok_or_else(|| ParadigmError::Rpc("Invalid chain ID format".to_string()))?;

        let chain_id_u64 = u64::from_str_radix(chain_id_str.trim_start_matches("0x"), 16)
            .map_err(|e| ParadigmError::Rpc(format!("Failed to parse chain ID: {}", e)))?;

        Ok(NetworkInfo {
            chain_id: chain_id_u64,
            block_number,
            gas_price,
            network_name: match chain_id_u64 {
                1 => "Mainnet".to_string(),
                11155111 => "Sepolia".to_string(),
                _ => format!("Unknown ({})", chain_id_u64),
            },
        })
    }

    /// Wait for transaction confirmation
    pub async fn wait_for_confirmation(
        &self,
        hash: &Hash,
        confirmations: u64,
    ) -> Result<TransactionReceipt> {
        let mut attempts = 0;
        let max_attempts = 60; // 5 minutes with 5-second intervals

        loop {
            if attempts >= max_attempts {
                return Err(ParadigmError::Timeout(format!(
                    "Transaction {} not confirmed after {} attempts",
                    hash, max_attempts
                )));
            }

            if let Some(receipt) = self.get_transaction_receipt(hash).await? {
                let current_block = self.get_block_number().await?;
                let confirmations_received = current_block.saturating_sub(receipt.block_number);

                if confirmations_received >= confirmations {
                    return Ok(receipt);
                }
            }

            attempts += 1;
            tokio::time::sleep(Duration::from_secs(5)).await;
        }
    }

    /// Clear request cache
    pub async fn clear_cache(&self) {
        let mut cache = self.cache.write().await;
        cache.clear();
    }

    /// Get cache statistics
    pub async fn get_cache_stats(&self) -> CacheStats {
        let cache = self.cache.read().await;
        let now = SystemTime::now();

        let expired_entries = cache
            .values()
            .filter(|entry| entry.expires_at <= now)
            .count();

        CacheStats {
            total_entries: cache.len(),
            expired_entries,
            valid_entries: cache.len() - expired_entries,
        }
    }
}

impl Default for ParadigmClient {
    fn default() -> Self {
        Self::new().expect("Failed to create default client")
    }
}

/// Network information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkInfo {
    pub chain_id: u64,
    pub block_number: u64,
    pub gas_price: Amount,
    pub network_name: String,
}

/// Cache statistics
#[derive(Debug, Clone)]
pub struct CacheStats {
    pub total_entries: usize,
    pub expired_entries: usize,
    pub valid_entries: usize,
}

/// Block number specification
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BlockNumber {
    Number(u64),
    Latest,
    Earliest,
    Pending,
}

impl Default for BlockNumber {
    fn default() -> Self {
        BlockNumber::Latest
    }
}

/// Client builder for advanced configuration
#[derive(Debug, Default)]
pub struct ClientBuilder {
    config: ClientConfig,
}

impl ClientBuilder {
    /// Create a new client builder
    pub fn new() -> Self {
        Self::default()
    }

    /// Set RPC endpoints
    pub fn rpc_endpoints(mut self, endpoints: Vec<String>) -> Self {
        self.config.rpc_endpoints = endpoints;
        self
    }

    /// Add RPC endpoint
    pub fn add_rpc_endpoint(mut self, endpoint: String) -> Self {
        self.config.rpc_endpoints.push(endpoint);
        self
    }

    /// Set WebSocket endpoints
    pub fn ws_endpoints(mut self, endpoints: Vec<String>) -> Self {
        self.config.ws_endpoints = endpoints;
        self
    }

    /// Set chain ID
    pub fn chain_id(mut self, chain_id: u64) -> Self {
        self.config.chain_id = chain_id;
        self
    }

    /// Set request timeout
    pub fn timeout(mut self, timeout: Duration) -> Self {
        self.config.timeout_ms = timeout.as_millis() as u64;
        self
    }

    /// Set maximum retries
    pub fn max_retries(mut self, retries: u32) -> Self {
        self.config.max_retries = retries;
        self
    }

    /// Enable or disable caching
    pub fn caching(mut self, enabled: bool) -> Self {
        self.config.enable_caching = enabled;
        self
    }

    /// Set cache TTL
    pub fn cache_ttl(mut self, ttl: Duration) -> Self {
        self.config.cache_ttl_seconds = ttl.as_secs();
        self
    }

    /// Set API key
    pub fn api_key(mut self, key: String) -> Self {
        self.config.api_key = Some(key);
        self
    }

    /// Add custom header
    pub fn header(mut self, name: String, value: String) -> Self {
        self.config.custom_headers.insert(name, value);
        self
    }

    /// Build the client
    pub fn build(self) -> Result<ParadigmClient> {
        ParadigmClient::with_config(self.config)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_client_config_default() {
        let config = ClientConfig::default();
        assert_eq!(config.chain_id, crate::MAINNET_CHAIN_ID);
        assert_eq!(config.timeout_ms, 30000);
        assert_eq!(config.max_retries, 3);
        assert!(config.enable_caching);
    }

    #[test]
    fn test_client_builder() {
        let client = ClientBuilder::new()
            .chain_id(1337)
            .timeout(Duration::from_secs(10))
            .max_retries(5)
            .caching(false)
            .add_rpc_endpoint("http://localhost:8545".to_string())
            .build();

        assert!(client.is_ok());
        let client = client.unwrap();
        assert_eq!(client.chain_id(), 1337);
    }

    #[test]
    fn test_block_number_enum() {
        assert_eq!(BlockNumber::default(), BlockNumber::Latest);
    }

    #[tokio::test]
    async fn test_client_creation() {
        let client = ParadigmClient::new();
        assert!(client.is_ok());
    }
}
