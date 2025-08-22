use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiConfig {
    pub host: String,
    pub port: u16,
    pub database_url: String,
    pub jwt_secret: String,
    pub jwt_expiration_hours: u64,
    pub rate_limits: RateLimits,
    pub cors: CorsConfig,
    pub auth: AuthConfig,
    pub webhooks: WebhookConfig,
    pub monitoring: MonitoringConfig,
    pub paradigm_node: NodeConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RateLimits {
    pub requests_per_minute: u32,
    pub burst_size: u32,
    pub authenticated_multiplier: f64,
    pub enterprise_multiplier: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CorsConfig {
    pub allowed_origins: Vec<String>,
    pub allowed_methods: Vec<String>,
    pub allowed_headers: Vec<String>,
    pub max_age: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthConfig {
    pub require_api_key: bool,
    pub enable_jwt: bool,
    pub enable_oauth: bool,
    pub oauth_providers: HashMap<String, OAuthProvider>,
    pub password_requirements: PasswordRequirements,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OAuthProvider {
    pub client_id: String,
    pub client_secret: String,
    pub redirect_uri: String,
    pub scope: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PasswordRequirements {
    pub min_length: usize,
    pub require_uppercase: bool,
    pub require_lowercase: bool,
    pub require_numbers: bool,
    pub require_symbols: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebhookConfig {
    pub enabled: bool,
    pub max_retries: u32,
    pub retry_delay_seconds: u64,
    pub timeout_seconds: u64,
    pub signature_secret: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MonitoringConfig {
    pub metrics_enabled: bool,
    pub metrics_port: u16,
    pub tracing_enabled: bool,
    pub health_check_interval_seconds: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeConfig {
    pub rpc_endpoint: String,
    pub websocket_endpoint: Option<String>,
    pub chain_id: String,
    pub network: String,
}

impl Default for ApiConfig {
    fn default() -> Self {
        let mut oauth_providers = HashMap::new();
        oauth_providers.insert("google".to_string(), OAuthProvider {
            client_id: "".to_string(),
            client_secret: "".to_string(),
            redirect_uri: "".to_string(),
            scope: "openid email profile".to_string(),
        });

        Self {
            host: "0.0.0.0".to_string(),
            port: 8080,
            database_url: "sqlite://paradigm_api.db".to_string(),
            jwt_secret: "your-secret-key-change-in-production".to_string(),
            jwt_expiration_hours: 24,
            rate_limits: RateLimits {
                requests_per_minute: 60,
                burst_size: 10,
                authenticated_multiplier: 5.0,
                enterprise_multiplier: 50.0,
            },
            cors: CorsConfig {
                allowed_origins: vec!["*".to_string()],
                allowed_methods: vec!["GET".to_string(), "POST".to_string(), "PUT".to_string(), "DELETE".to_string()],
                allowed_headers: vec!["*".to_string()],
                max_age: 3600,
            },
            auth: AuthConfig {
                require_api_key: false,
                enable_jwt: true,
                enable_oauth: false,
                oauth_providers,
                password_requirements: PasswordRequirements {
                    min_length: 8,
                    require_uppercase: true,
                    require_lowercase: true,
                    require_numbers: true,
                    require_symbols: false,
                },
            },
            webhooks: WebhookConfig {
                enabled: true,
                max_retries: 3,
                retry_delay_seconds: 5,
                timeout_seconds: 30,
                signature_secret: "webhook-secret-change-in-production".to_string(),
            },
            monitoring: MonitoringConfig {
                metrics_enabled: true,
                metrics_port: 9090,
                tracing_enabled: true,
                health_check_interval_seconds: 30,
            },
            paradigm_node: NodeConfig {
                rpc_endpoint: "http://localhost:8545".to_string(),
                websocket_endpoint: Some("ws://localhost:8546".to_string()),
                chain_id: "paradigm-1".to_string(),
                network: "mainnet".to_string(),
            },
        }
    }
}

impl ApiConfig {
    pub fn from_env() -> anyhow::Result<Self> {
        dotenvy::dotenv().ok();
        
        let mut config = Self::default();
        
        if let Ok(host) = std::env::var("API_HOST") {
            config.host = host;
        }
        
        if let Ok(port) = std::env::var("API_PORT") {
            config.port = port.parse()?;
        }
        
        if let Ok(database_url) = std::env::var("DATABASE_URL") {
            config.database_url = database_url;
        }
        
        if let Ok(jwt_secret) = std::env::var("JWT_SECRET") {
            config.jwt_secret = jwt_secret;
        }
        
        if let Ok(paradigm_rpc) = std::env::var("PARADIGM_RPC_ENDPOINT") {
            config.paradigm_node.rpc_endpoint = paradigm_rpc;
        }
        
        if let Ok(paradigm_ws) = std::env::var("PARADIGM_WS_ENDPOINT") {
            config.paradigm_node.websocket_endpoint = Some(paradigm_ws);
        }
        
        Ok(config)
    }
    
    pub fn from_file(path: &str) -> anyhow::Result<Self> {
        let config_str = std::fs::read_to_string(path)?;
        let config: Self = serde_json::from_str(&config_str)?;
        Ok(config)
    }
    
    pub fn validate(&self) -> anyhow::Result<()> {
        if self.jwt_secret.len() < 32 {
            return Err(anyhow::anyhow!("JWT secret must be at least 32 characters"));
        }
        
        if self.port == 0 {
            return Err(anyhow::anyhow!("Port must be specified"));
        }
        
        if self.database_url.is_empty() {
            return Err(anyhow::anyhow!("Database URL must be specified"));
        }
        
        if self.paradigm_node.rpc_endpoint.is_empty() {
            return Err(anyhow::anyhow!("Paradigm node RPC endpoint must be specified"));
        }
        
        Ok(())
    }
}