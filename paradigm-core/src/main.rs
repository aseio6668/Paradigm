use paradigm_core::{ParadigmNode, NodeConfig, PARADIGM_VERSION};
use clap::{Arg, Command};
use tracing_subscriber;
use anyhow::Result;
use std::path::PathBuf;
use uuid;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    tracing_subscriber::fmt::init();

    let matches = Command::new("paradigm-node")
        .version(PARADIGM_VERSION)
        .about("Paradigm Cryptocurrency Node")
        .arg(
            Arg::new("data-dir")
                .long("data-dir")
                .value_name("PATH")
                .help("Directory to store blockchain data")
                .default_value("./paradigm-data"),
        )
        .arg(
            Arg::new("port")
                .long("port")
                .value_name("PORT")
                .help("Network port to listen on")
                .default_value("8080"),
        )
        .arg(
            Arg::new("bootstrap-peers")
                .long("bootstrap-peers")
                .value_name("PEERS")
                .help("Comma-separated list of bootstrap peer addresses")
                .default_value(""),
        )
        .get_matches();

    let data_dir = PathBuf::from(matches.get_one::<String>("data-dir").unwrap());
    let port: u16 = matches.get_one::<String>("port").unwrap().parse()?;
    let bootstrap_peers: Vec<String> = matches
        .get_one::<String>("bootstrap-peers")
        .unwrap()
        .split(',')
        .filter(|s| !s.is_empty())
        .map(|s| s.to_string())
        .collect();

    tracing::info!("Starting Paradigm node v{}", PARADIGM_VERSION);
    tracing::info!("Data directory: {}", data_dir.display());
    tracing::info!("Network port: {}", port);

    // Ensure data directory exists
    std::fs::create_dir_all(&data_dir)?;

    // Create configuration
    let config = NodeConfig {
        node_id: uuid::Uuid::new_v4(),
        network_port: port,
        data_dir: data_dir.to_string_lossy().to_string(),
        enable_ml_tasks: true,
        max_peers: 50,
    };

    // Create and start the node
    let mut node = ParadigmNode::new(config).await?;
    
    tracing::info!("Node created successfully");
    
    // Start the node
    node.start().await?;

    // Keep the node running
    tracing::info!("Paradigm node is running...");
    tracing::info!("Press Ctrl+C to stop");

    // Wait for shutdown signal
    tokio::signal::ctrl_c().await?;
    tracing::info!("Received shutdown signal, stopping node...");

    Ok(())
}
