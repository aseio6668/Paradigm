use anyhow::Result;
use paradigm_api::{ApiServer, init_observability};
use paradigm_api::config::ApiConfig;
use tracing::info;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize observability (logging and metrics)
    init_observability()?;

    // Load configuration
    let config = ApiConfig::from_env()
        .or_else(|_| ApiConfig::from_file("config.json"))
        .unwrap_or_else(|_| {
            info!("Using default configuration");
            ApiConfig::default()
        });

    // Validate configuration
    config.validate()?;

    info!("Starting Paradigm API server...");
    info!("Configuration loaded successfully");

    // Create and start the API server
    let server = ApiServer::new(config).await?;
    server.start().await?;

    Ok(())
}