use anyhow::Result;
use clap::{Arg, Command};
use paradigm_core::api::{generate_sample_tasks, start_api_server, ApiState};
use paradigm_core::genesis::{GenesisConfig, GenesisManager};
use paradigm_core::{NodeConfig, ParadigmNode, PARADIGM_VERSION};
use std::path::PathBuf;
use tracing_subscriber;
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
        .arg(
            Arg::new("config")
                .long("config")
                .value_name("FILE")
                .help("Path to network configuration file")
                .default_value("network-config.toml"),
        )
        .arg(
            Arg::new("genesis")
                .long("genesis")
                .value_name("FILE")
                .help("Path to genesis configuration file (for new chains)")
                .default_value(""),
        )
        .arg(
            Arg::new("addnode")
                .long("addnode")
                .value_name("NODES")
                .help("Manual peer connections: IP:PORT;IP2:PORT2 or IP;IP2 (uses default port)")
                .default_value(""),
        )
        .arg(
            Arg::new("addnodefile")
                .long("addnodefile")
                .value_name("FILE")
                .help("Load peer connections from text file (one IP[:PORT] per line)")
                .default_value(""),
        )
        .arg(
            Arg::new("api-port")
                .long("api-port")
                .value_name("PORT")
                .help("HTTP API server port")
                .default_value("8080"),
        )
        .arg(
            Arg::new("enable-api")
                .long("enable-api")
                .help("Enable HTTP API server for task distribution")
                .action(clap::ArgAction::SetTrue),
        )
        .get_matches();

    let data_dir = PathBuf::from(matches.get_one::<String>("data-dir").unwrap());
    let port: u16 = matches.get_one::<String>("port").unwrap().parse()?;
    let api_port: u16 = matches.get_one::<String>("api-port").unwrap().parse()?;
    let enable_api = matches.get_flag("enable-api");
    let _bootstrap_peers: Vec<String> = matches
        .get_one::<String>("bootstrap-peers")
        .unwrap()
        .split(',')
        .filter(|s| !s.is_empty())
        .map(|s| s.to_string())
        .collect();
    let config_file = matches.get_one::<String>("config").unwrap();
    let genesis_file = matches.get_one::<String>("genesis").unwrap();
    let addnode_string = matches.get_one::<String>("addnode").unwrap();
    let addnodefile_path = matches.get_one::<String>("addnodefile").unwrap();

    // Parse manual peer connections
    let mut manual_peers = parse_addnode_peers(addnode_string, port)?;

    // Load peers from file if specified
    if !addnodefile_path.is_empty() {
        match load_peers_from_file(addnodefile_path, port) {
            Ok(file_peers) => {
                manual_peers.extend(file_peers);
                tracing::info!(
                    "Loaded {} additional peers from file: {}",
                    manual_peers.len() - parse_addnode_peers(addnode_string, port)?.len(),
                    addnodefile_path
                );
            }
            Err(e) => {
                tracing::warn!("Failed to load peers from file {}: {}", addnodefile_path, e);
            }
        }
    }

    tracing::info!("Starting Paradigm node v{}", PARADIGM_VERSION);
    tracing::info!("Data directory: {}", data_dir.display());
    tracing::info!("Network port: {}", port);

    // Check for genesis configuration
    let is_genesis = !genesis_file.is_empty() && std::path::Path::new(genesis_file).exists();

    if is_genesis {
        tracing::info!("Genesis configuration found: {}", genesis_file);
        tracing::info!("Starting new blockchain from block 0");
    }

    // Ensure data directory exists
    std::fs::create_dir_all(&data_dir)?;

    // Create configuration
    let config = NodeConfig {
        node_id: uuid::Uuid::new_v4(),
        network_port: port,
        data_dir: data_dir.to_string_lossy().to_string(),
        enable_ml_tasks: true,
        max_peers: 50,
        enable_hsm: false,
        hsm_config: None,
    };

    // Create and start the node
    let mut node = ParadigmNode::new(config).await?;

    // If this is a genesis node, initialize the blockchain
    if is_genesis {
        tracing::info!("Initializing genesis blockchain...");

        // Create genesis manager
        let db_path = data_dir.join("paradigm.db");
        let storage = paradigm_core::storage::ParadigmStorage::new(&format!(
            "sqlite://{}",
            db_path.display()
        ))
        .await?;

        let mut genesis_manager = GenesisManager::new(storage);

        // Load genesis configuration
        let genesis_config = if std::path::Path::new(genesis_file).exists() {
            // For now, use default config - in production, parse the TOML file
            GenesisConfig::default()
        } else {
            GenesisConfig::default()
        };

        // Create and initialize genesis block
        let genesis_block = genesis_manager.create_genesis_block(genesis_config).await?;
        genesis_manager
            .initialize_chain_from_genesis(&genesis_block)
            .await?;

        tracing::info!("✅ Genesis blockchain initialized successfully!");
        tracing::info!(
            "Network treasury holds {} PAR tokens",
            genesis_block.config.initial_supply as f64 / 100_000_000.0
        );
        tracing::info!("AI governance is active and controlling distribution");
    }

    tracing::info!("Node created successfully");

    // Start the node
    node.start().await?;

    // Add bootstrap peers to peer manager after node is started
    if !manual_peers.is_empty() {
        tracing::info!(
            "Adding {} bootstrap peers to peer manager",
            manual_peers.len()
        );
        let peer_manager = node.peer_manager.read().await;
        for peer in &manual_peers {
            if let Err(e) = peer_manager.add_bootstrap_peer(peer.clone()).await {
                tracing::warn!("Failed to add bootstrap peer {}: {}", peer, e);
            }
        }
        tracing::info!("Bootstrap peers configured: {:?}", manual_peers);
    }

    // Print initial network sync status
    let sync_info = node.get_sync_info().await;
    tracing::info!(
        "Network sync: {} ({}%)",
        sync_info.status_string(),
        sync_info.progress_percentage
    );

    // Start API server if enabled
    if enable_api {
        tracing::info!("Starting HTTP API server on port {}", api_port);

        // Create API state with storage, autonomous task integration, and peer manager
        let api_state = ApiState::new()
            .with_storage(node.storage.clone())
            .with_autonomous_tasks(node.autonomous_tasks.clone())
            .with_peer_manager(node.peer_manager.clone());

        // Set initial node status
        api_state.update_network_status("running".to_string()).await;
        api_state
            .update_peers_count(manual_peers.len() as u32)
            .await;

        // No need to generate sample tasks - autonomous system will handle this
        tracing::info!("API configured with autonomous task generation");

        // Clone the API state for the server task
        let server_state = api_state.clone();
        let server_port = api_port;

        // Start API server in a separate task
        tokio::spawn(async move {
            if let Err(e) = start_api_server(server_port, server_state).await {
                tracing::error!("API server error: {}", e);
            }
        });

        tracing::info!("✅ HTTP API server started on http://0.0.0.0:{}", api_port);
        tracing::info!("Health endpoint: http://localhost:{}/health", api_port);
        tracing::info!(
            "Tasks endpoint: http://localhost:{}/api/tasks/available",
            api_port
        );
    }

    // Keep the node running and show periodic sync status
    tracing::info!("Paradigm node is running...");
    tracing::info!("Press Ctrl+C to stop");

    // Create periodic status update task
    let node_sync = node.network_sync.clone();
    let status_task = tokio::spawn(async move {
        let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(30));
        loop {
            interval.tick().await;
            let sync = node_sync.read().await;
            let sync_info = sync.get_sync_info();
            tracing::info!(
                "Network sync: {} {}",
                sync_info.status_string(),
                sync_info.progress_bar()
            );
        }
    });

    // Wait for shutdown signal
    tokio::signal::ctrl_c().await?;
    tracing::info!("Received shutdown signal, stopping node...");

    // Cancel status task
    status_task.abort();

    Ok(())
}

/// Parse --addnode peer connections from semicolon-separated string
/// Format: "IP:PORT;IP2:PORT2" or "IP;IP2" (uses default port)
fn parse_addnode_peers(addnode_string: &str, default_port: u16) -> Result<Vec<String>> {
    if addnode_string.is_empty() {
        return Ok(Vec::new());
    }

    let mut peers = Vec::new();

    // Split by semicolon and process each peer
    for peer_str in addnode_string.split(';') {
        let peer_str = peer_str.trim();
        if peer_str.is_empty() {
            continue;
        }

        let peer_address = if peer_str.contains(':') {
            // Already has port specified
            peer_str.to_string()
        } else {
            // No port specified, use default
            format!("{}:{}", peer_str, default_port)
        };

        // Validate the address format
        if let Err(_) = peer_address.parse::<std::net::SocketAddr>() {
            tracing::warn!("Invalid peer address format: {}, skipping", peer_address);
            continue;
        }

        peers.push(peer_address);
    }

    Ok(peers)
}

/// Load peer connections from a text file
/// Format: One IP[:PORT] per line, lines starting with # are ignored
fn load_peers_from_file(file_path: &str, default_port: u16) -> Result<Vec<String>> {
    let content = std::fs::read_to_string(file_path)
        .map_err(|e| anyhow::anyhow!("Failed to read peer file {}: {}", file_path, e))?;

    let mut peers = Vec::new();

    for (line_num, line) in content.lines().enumerate() {
        let line = line.trim();

        // Skip empty lines and comments
        if line.is_empty() || line.starts_with('#') {
            continue;
        }

        let peer_address = if line.contains(':') {
            // Already has port specified
            line.to_string()
        } else {
            // No port specified, use default
            format!("{}:{}", line, default_port)
        };

        // Validate the address format
        if let Err(_) = peer_address.parse::<std::net::SocketAddr>() {
            tracing::warn!(
                "Invalid peer address format at line {}: {}, skipping",
                line_num + 1,
                peer_address
            );
            continue;
        }

        peers.push(peer_address);
    }

    tracing::info!("Loaded {} peers from file: {}", peers.len(), file_path);
    Ok(peers)
}
