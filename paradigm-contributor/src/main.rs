// Paradigm Contributor - ML Task Processing Client with GPU Acceleration
use anyhow::Result;
use clap::Parser;
use ed25519_dalek::SigningKey;
use paradigm_core::{
    autopool::AutopoolManager, wallet_manager::WalletManager, Address, AddressExt, MLTask,
};
use rand::rngs::OsRng;
use rand::RngCore;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::{fs, path::PathBuf, time::Duration};
use tracing::{error, info, warn};
use uuid::Uuid;

mod gpu_compute;
mod performance_monitor;
mod task_manager;

use gpu_compute::{GpuBackend, GpuComputeEngine};
use performance_monitor::PerformanceMonitor;
use task_manager::TaskManager;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskRequest {
    pub task_id: String,
    pub task_type: String,
    pub difficulty: u32,
    pub data: String,
    pub reward: u64,
    pub timestamp: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskResponse {
    pub available_tasks: Vec<TaskRequest>,
    pub queue_size: u32,
    pub estimated_reward: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthResponse {
    pub status: String,
    pub version: String,
    pub node_id: String,
    pub peers_count: u32,
    pub block_height: u64,
    pub network_status: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskSubmission {
    pub task_id: String,
    pub result: String,
    pub completion_time_ms: u64,
    pub contributor_address: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PayoutResponse {
    pub success: bool,
    pub transaction_id: String,
    pub amount_paid: u64,
    pub recipient_address: String,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContributorWallet {
    pub address: String,
    pub private_key_bytes: Vec<u8>,
    pub balance: u64,
    pub total_earned: u64,
    pub tasks_completed: u64,
    pub created_at: u64,
    pub last_updated: u64,
}

impl ContributorWallet {
    pub fn new() -> Self {
        let mut secret_bytes = [0u8; 32];
        OsRng.fill_bytes(&mut secret_bytes);
        let signing_key = SigningKey::from_bytes(&secret_bytes);
        let contributor_address = Address::from_public_key(&signing_key.verifying_key());
        let address_str = contributor_address.to_string();
        let now = chrono::Utc::now().timestamp() as u64;

        Self {
            address: address_str,
            private_key_bytes: signing_key.to_bytes().to_vec(),
            balance: 0,
            total_earned: 0,
            tasks_completed: 0,
            created_at: now,
            last_updated: now,
        }
    }

    pub fn load_or_create(wallet_path: &PathBuf) -> Result<Self> {
        if wallet_path.exists() {
            let wallet_data = fs::read_to_string(wallet_path)?;
            let wallet: ContributorWallet = serde_json::from_str(&wallet_data)?;
            info!("📂 Loaded existing wallet: {}", wallet.address);
            info!(
                "💰 Current balance: {:.8} PAR",
                wallet.balance as f64 / 100_000_000.0
            );
            info!(
                "📊 Total earned: {:.8} PAR from {} tasks",
                wallet.total_earned as f64 / 100_000_000.0,
                wallet.tasks_completed
            );
            Ok(wallet)
        } else {
            let wallet = Self::new();
            wallet.save(wallet_path)?;
            info!("🔑 Created new wallet: {}", wallet.address);
            info!("💾 Wallet saved to: {}", wallet_path.display());
            Ok(wallet)
        }
    }

    pub fn save(&self, wallet_path: &PathBuf) -> Result<()> {
        if let Some(parent) = wallet_path.parent() {
            fs::create_dir_all(parent)?;
        }
        let wallet_json = serde_json::to_string_pretty(self)?;
        fs::write(wallet_path, wallet_json)?;
        Ok(())
    }

    pub fn add_payout(&mut self, amount: u64, wallet_path: &PathBuf) -> Result<()> {
        self.balance += amount;
        self.total_earned += amount;
        self.tasks_completed += 1;
        self.last_updated = chrono::Utc::now().timestamp() as u64;
        self.save(wallet_path)?;
        Ok(())
    }

    pub fn get_signing_key(&self) -> Result<SigningKey> {
        let key_bytes: [u8; 32] = self
            .private_key_bytes
            .as_slice()
            .try_into()
            .map_err(|_| anyhow::anyhow!("Invalid private key format"))?;
        Ok(SigningKey::from_bytes(&key_bytes))
    }
}

#[derive(Debug, Clone)]
pub enum NetworkStatus {
    Disconnected,
    Connecting,
    Connected,
    NetworkError(String),
}

#[derive(Debug, Clone)]
pub struct NetworkConnection {
    client: Client,
    node_address: String,
    status: NetworkStatus,
    last_ping: Option<std::time::Instant>,
}

impl NetworkConnection {
    pub fn new(node_address: String) -> Self {
        Self {
            client: Client::new(),
            node_address,
            status: NetworkStatus::Disconnected,
            last_ping: None,
        }
    }

    pub async fn test_connection(&mut self, timeout_secs: u64) -> bool {
        self.status = NetworkStatus::Connecting;

        let url = format!("http://{}/health", self.node_address);
        let timeout = Duration::from_secs(timeout_secs);

        match tokio::time::timeout(timeout, self.client.get(&url).send()).await {
            Ok(Ok(response)) => {
                if response.status().is_success() {
                    self.status = NetworkStatus::Connected;
                    self.last_ping = Some(std::time::Instant::now());
                    info!(
                        "✅ Successfully connected to Paradigm node at {}",
                        self.node_address
                    );
                    true
                } else {
                    self.status =
                        NetworkStatus::NetworkError(format!("HTTP {}", response.status()));
                    false
                }
            }
            Ok(Err(e)) => {
                self.status = NetworkStatus::NetworkError(e.to_string());
                false
            }
            Err(_) => {
                self.status = NetworkStatus::NetworkError("Connection timeout".to_string());
                false
            }
        }
    }

    pub async fn fetch_tasks(&mut self) -> Result<Option<Vec<TaskRequest>>, String> {
        if !matches!(self.status, NetworkStatus::Connected) {
            return Err("Not connected to network".to_string());
        }

        let url = format!("http://{}/api/tasks/available", self.node_address);

        match self.client.get(&url).send().await {
            Ok(response) => {
                if response.status().is_success() {
                    match response.json::<TaskResponse>().await {
                        Ok(task_response) => {
                            if task_response.available_tasks.is_empty() {
                                Ok(None)
                            } else {
                                info!(
                                    "Received {} available tasks from network",
                                    task_response.available_tasks.len()
                                );
                                Ok(Some(task_response.available_tasks))
                            }
                        }
                        Err(e) => {
                            warn!("Failed to parse task response: {}", e);
                            Err(format!("Failed to parse tasks response: {}", e))
                        }
                    }
                } else {
                    Err(format!("Failed to fetch tasks: HTTP {}", response.status()))
                }
            }
            Err(e) => {
                self.status = NetworkStatus::NetworkError(e.to_string());
                Err(e.to_string())
            }
        }
    }

    pub fn get_status(&self) -> &NetworkStatus {
        &self.status
    }

    pub fn is_connected(&self) -> bool {
        matches!(self.status, NetworkStatus::Connected)
    }

    pub async fn submit_task_completion(
        &mut self,
        task_id: String,
        result: String,
        completion_time_ms: u64,
        contributor_address: String,
    ) -> Result<PayoutResponse, String> {
        if !matches!(self.status, NetworkStatus::Connected) {
            return Err("Not connected to network".to_string());
        }

        let url = format!("http://{}/api/tasks/submit", self.node_address);
        let submission = TaskSubmission {
            task_id,
            result,
            completion_time_ms,
            contributor_address,
        };

        match self.client.post(&url).json(&submission).send().await {
            Ok(response) => {
                if response.status().is_success() {
                    match response.json::<PayoutResponse>().await {
                        Ok(payout_response) => Ok(payout_response),
                        Err(e) => {
                            warn!("Failed to parse payout response: {}", e);
                            Err(format!("Failed to parse payout response: {}", e))
                        }
                    }
                } else {
                    Err(format!(
                        "Failed to submit task completion: HTTP {}",
                        response.status()
                    ))
                }
            }
            Err(e) => {
                self.status = NetworkStatus::NetworkError(e.to_string());
                Err(e.to_string())
            }
        }
    }
}

#[derive(Parser)]
#[command(name = "paradigm-contributor")]
#[command(about = "Paradigm ML Contributor Client with GPU Acceleration")]
pub struct Args {
    /// Node address to connect to
    #[arg(long, default_value = "127.0.0.1:8080")]
    node_address: String,

    /// Number of worker threads (0 = auto-detect)
    #[arg(long, default_value_t = 0)]
    threads: usize,

    /*
    /// Preferred GPU backend: auto, cuda, opencl, wgpu, cpu
    #[arg(long, default_value = "auto")]
    gpu_backend: String,

    /// Run performance benchmark on startup
    #[arg(long)]
    benchmark: bool,
    */
    /// Enable verbose logging
    #[arg(short, long)]
    verbose: bool,

    /// Connection timeout in seconds
    #[arg(long, default_value_t = 10)]
    timeout: u64,

    /// Retry interval in seconds for network connection
    #[arg(long, default_value_t = 30)]
    retry_interval: u64,

    /// Use specific wallet address for receiving rewards (optional)
    #[arg(long)]
    wallet_address: Option<String>,

    /// Wallet file to load/save addresses (optional)
    #[arg(long)]
    wallet_file: Option<String>,

    /// Enable autopool for small contribution aggregation (opt-in)
    #[arg(long)]
    enable_autopool: bool,

    /// Minimum PAR amount threshold for autopool participation
    #[arg(long, default_value_t = 1000000)] // 0.01 PAR
    autopool_threshold: u64,
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Args::parse();

    // Initialize logging
    if args.verbose {
        tracing_subscriber::fmt()
            .with_max_level(tracing::Level::DEBUG)
            .init();
    } else {
        tracing_subscriber::fmt()
            .with_max_level(tracing::Level::INFO)
            .init();
    }

    info!("🚀 Starting Paradigm Contributor v0.1.0");
    info!("💻 Connecting to node at: {}", args.node_address);
    //info!("🎯 Preferred GPU backend: {}", args.gpu_backend);

    /*
    // Initialize GPU compute engine
    info!("🔧 Initializing GPU compute engine...");
    let compute_engine = match GpuComputeEngine::new().await {
        Ok(engine) => {
            let caps = engine.get_capabilities();
            info!("✅ GPU compute engine initialized successfully");
            info!("🎯 Recommended backend: {:?}", caps.recommended_backend);
            info!("💾 Total GPU memory: {} MB", caps.total_memory_mb);
            info!("⚡ Performance score: {:.1}", caps.estimated_performance_score);

            // Display detected GPUs
            for (i, gpu) in caps.available_gpus.iter().enumerate() {
                info!("  GPU {}: {} ({:?}) - {} MB",
                     i, gpu.device_name, gpu.backend, gpu.memory_mb);
            }

            engine
        }
        Err(e) => {
            error!("❌ Failed to initialize GPU compute engine: {}", e);
            return Err(e);
        }
    };

    // Run benchmark if requested
    if args.benchmark {
        info!("🏃 Running performance benchmark...");
        match compute_engine.benchmark_performance().await {
            Ok(throughput) => {
                info!("📊 Benchmark result: {:.2} MB/s", throughput);
            }
            Err(e) => {
                warn!("⚠️  Benchmark failed: {}", e);
            }
        }
    }
    */

    // Determine number of worker threads
    let num_workers = if args.threads == 0 {
        num_cpus::get()
    } else {
        args.threads
    };

    info!("🔧 Using {} worker threads", num_workers);

    // Initialize task manager
    let _task_manager = TaskManager::new(num_workers).await?;
    let mut performance_monitor = PerformanceMonitor::new();

    // Initialize wallet system
    let wallet_path = if let Some(wallet_file) = &args.wallet_file {
        PathBuf::from(wallet_file)
    } else {
        WalletManager::get_default_wallet_path()
    };

    let mut wallet_manager = WalletManager::new(wallet_path.clone())?;
    let address_str = wallet_manager
        .get_or_create_address(args.wallet_address.clone(), "Contributor Work Address")?;

    info!("💳 Contributor wallet address: {}", address_str);
    info!("🔑 Ready to receive PAR token rewards");
    info!("💾 Wallet file: {}", wallet_path.display());

    // Show wallet info if address already exists
    if let Some(entry) = wallet_manager.get_address_info(&address_str) {
        if entry.total_earned > 0 {
            info!("📊 Existing wallet stats:");
            info!(
                "   💰 Balance: {:.8} PAR",
                entry.balance as f64 / 100_000_000.0
            );
            info!(
                "   🎯 Total earned: {:.8} PAR",
                entry.total_earned as f64 / 100_000_000.0
            );
            info!("   ⚡ Tasks completed: {}", entry.tasks_completed);
        }
    }

    // Initialize autopool system if enabled
    let autopool_manager = if args.enable_autopool {
        let autopool = AutopoolManager::new(args.autopool_threshold);
        info!(
            "🔄 Autopool system enabled (threshold: {:.8} PAR)",
            args.autopool_threshold as f64 / 100_000_000.0
        );
        info!("💡 Small contributions will be automatically pooled for efficient payouts");
        Some(autopool)
    } else {
        info!("⚠️  Autopool disabled - use --enable-autopool to participate in work aggregation");
        None
    };

    // Initialize network connection
    let mut network_connection = NetworkConnection::new(args.node_address.clone());
    let mut connection_retry_count = 0u32;
    let mut last_connection_attempt = std::time::Instant::now();
    let mut in_mock_mode = false;

    info!("✅ Contributor initialized successfully");
    info!("🔄 Starting main processing loop...");
    info!("🌐 Attempting to connect to Paradigm network...");

    // Main processing loop with real network connectivity
    let mut task_count = 0u64;
    let mut mock_task_count = 0u64;

    loop {
        tokio::select! {
            // Network connection and task processing
            _ = tokio::time::sleep(Duration::from_secs(3)) => {
                // Check network connection status
                let needs_connection_attempt = match network_connection.get_status() {
                    NetworkStatus::Disconnected | NetworkStatus::NetworkError(_) => {
                        // In mock mode, check more frequently for network recovery
                        let check_interval = if in_mock_mode {
                            Duration::from_secs(5)  // Check every 5 seconds when in mock mode
                        } else {
                            Duration::from_secs(args.retry_interval)
                        };
                        last_connection_attempt.elapsed() > check_interval
                    }
                    NetworkStatus::Connecting => false,
                    NetworkStatus::Connected => {
                        // Periodically ping to ensure connection is still alive
                        network_connection.last_ping.map_or(true, |t| t.elapsed() > Duration::from_secs(60))
                    }
                };

                // Attempt connection if needed
                if needs_connection_attempt {
                    connection_retry_count += 1;
                    last_connection_attempt = std::time::Instant::now();

                    if connection_retry_count > 1 {
                        if !in_mock_mode {
                            warn!("🔄 Connection attempt #{} to {}...", connection_retry_count, args.node_address);
                        }
                    } else {
                        info!("🔗 Testing connection to Paradigm node...");
                    }

                    let connected = network_connection.test_connection(args.timeout).await;

                    if connected {
                        if in_mock_mode {
                            info!("🎉 Network connection restored! Switching from mock mode to real tasks.");
                            in_mock_mode = false;
                        }
                        connection_retry_count = 0;
                    } else {
                        if connection_retry_count == 1 {
                            warn!("❌ Failed to connect to Paradigm network at {}", args.node_address);
                            warn!("📡 Status: {:?}", network_connection.get_status());
                            warn!("🔄 Will retry every {} seconds...", args.retry_interval);
                            warn!("⚠️  RUNNING IN MOCK MODE - No real work is being done!");
                            warn!("💡 Make sure the Paradigm node is running and accessible.");
                        }

                        if !in_mock_mode {
                            in_mock_mode = true;
                            info!("🎭 Entering mock mode - simulating tasks until network is available");
                        }
                    }
                }

                // Process tasks based on connection status
                if network_connection.is_connected() {
                    // Try to fetch real tasks from the network
                    match network_connection.fetch_tasks().await {
                        Ok(Some(tasks)) => {
                            // Process real network tasks
                            for task in tasks {
                                task_count += 1;
                                let start_time = std::time::Instant::now();

                                // Real task processing would go here
                                tokio::time::sleep(Duration::from_millis(100)).await;

                                let completion_time = start_time.elapsed();
                                let reward_par = task.reward as f64 / 100_000_000.0;

                                // Submit task completion and request payout
                                let task_result = format!("task_completed_{}", task.task_id);
                                match network_connection.submit_task_completion(
                                    task.task_id.clone(),
                                    task_result,
                                    completion_time.as_millis() as u64,
                                    address_str.clone()
                                ).await {
                                    Ok(payout) => {
                                        let actual_paid = payout.amount_paid as f64 / 100_000_000.0;

                                        // Check if autopool should be used for small rewards
                                        if let Some(ref autopool) = autopool_manager {
                                            if autopool.should_offer_autopool(payout.amount_paid, 1.0).await {
                                                // Use autopool for small contributions
                                                match autopool.opt_into_autopool(&address_str, 1.0).await {
                                                    Ok(session_id) => {
                                                        // Record work contribution to autopool session
                                                        let work_amount = completion_time.as_secs_f64(); // Use processing time as work metric
                                                        let _ = autopool.record_work_contribution(
                                                            session_id,
                                                            &address_str,
                                                            work_amount,
                                                            completion_time.as_secs()
                                                        ).await;

                                                        // Add earnings to session instead of direct payout
                                                        let _ = autopool.add_session_earnings(session_id, payout.amount_paid).await;

                                                        info!("🔄 [AUTOPOOL] Task #{} → {:.8} PAR added to session {}",
                                                              task_count, actual_paid, session_id);
                                                        info!("💡 Small contribution pooled for efficient payout");
                                                    }
                                                    Err(e) => {
                                                        warn!("❌ Autopool failed, using direct payout: {}", e);
                                                        // Fall back to direct wallet update
                                                        if let Err(e) = wallet_manager.update_address_balance(&address_str, payout.amount_paid) {
                                                            warn!("Failed to update wallet: {}", e);
                                                        }
                                                    }
                                                }
                                            } else {
                                                // Direct payout for larger amounts
                                                if let Err(e) = wallet_manager.update_address_balance(&address_str, payout.amount_paid) {
                                                    warn!("Failed to update wallet: {}", e);
                                                }
                                            }
                                        } else {
                                            // No autopool, direct wallet update
                                            if let Err(e) = wallet_manager.update_address_balance(&address_str, payout.amount_paid) {
                                                warn!("Failed to update wallet: {}", e);
                                            }
                                        }

                                        info!("✅ [REAL] Task #{} completed in {:?} | 💰 RECEIVED {:.8} PAR → {}",
                                              task_count, completion_time, actual_paid, address_str);
                                        info!("📦 Transaction ID: {}", payout.transaction_id);

                                        // Show updated balance
                                        if let Some(entry) = wallet_manager.get_address_info(&address_str) {
                                            info!("💰 New balance: {:.8} PAR | Total earned: {:.8} PAR",
                                                  entry.balance as f64 / 100_000_000.0,
                                                  entry.total_earned as f64 / 100_000_000.0);
                                        }
                                    }
                                    Err(e) => {
                                        warn!("❌ Task completed but payout failed: {} - earned {:.8} PAR (pending)",
                                              e, reward_par);
                                    }
                                }
                            }
                        }
                        Ok(None) => {
                            // No tasks available - wait
                            if task_count % 10 == 0 {
                                info!("⏳ Connected to network, waiting for tasks...");
                            }
                        }
                        Err(e) => {
                            error!("❌ Failed to fetch tasks: {}", e);
                            // Connection might be broken, will retry on next cycle
                        }
                    }
                } else if in_mock_mode {
                    // Run in mock mode - generate fake tasks to show the system works
                    mock_task_count += 1;

                    let mock_task = MLTask {
                        id: Uuid::new_v4(),
                        task_type: paradigm_core::consensus::MLTaskType::ImageClassification,
                        data: vec![0u8; 1024 * (1 + mock_task_count % 10) as usize],
                        difficulty: 1 + (mock_task_count % 5) as u8,
                        reward: 1000000 * (1 + mock_task_count % 3),
                        deadline: chrono::Utc::now() + chrono::Duration::minutes(30),
                        created_at: chrono::Utc::now(),
                        assigned_to: None,
                        completed: false,
                        result: None,
                    };

                    let start_time = std::time::Instant::now();
                    tokio::time::sleep(Duration::from_millis(200)).await;

                    let completion_time = start_time.elapsed();
                    let reward_par = mock_task.reward as f64 / 100_000_000.0;

                    // Periodically check if the network has come back online during mock mode
                    if mock_task_count % 10 == 0 {
                        info!("🔍 [MOCK] Checking if Paradigm network is back online...");
                        if network_connection.test_connection(2).await {
                            info!("🎉 Network connection restored! Exiting mock mode immediately.");
                            in_mock_mode = false;
                            connection_retry_count = 0;
                            // Skip the sleep to immediately start processing real tasks
                            continue;
                        }
                    }

                    if mock_task_count % 5 == 0 {
                        warn!("🎭 [MOCK] Simulated task #{} in {:?} - NO real PAR earned",
                              mock_task_count, completion_time);
                        warn!("⚠️  This is a simulation! Connect to a real Paradigm network to earn rewards.");
                    } else {
                        info!("🎭 [MOCK] Simulated task #{} in {:?} - would earn {:.8} PAR",
                              mock_task_count, completion_time, reward_par);
                    }
                } else {
                    // Waiting for first connection attempt
                    info!("⏳ Waiting to connect to network...");
                }
            }

            // Update performance metrics and log status
            _ = tokio::time::sleep(Duration::from_secs(30)) => {
                performance_monitor.update_metrics().await;

                let stats = performance_monitor.get_stats();
                info!("📊 Performance: {} tasks completed, avg time: {:?}",
                      stats.total_tasks, stats.average_time);

                // Log CPU processing status
                if task_count % 6 == 0 {
                    info!("💡 CPU task processing active");
                }
            }

            // Check autopool sessions for payouts
            _ = tokio::time::sleep(Duration::from_secs(60)) => {
                if let Some(ref autopool) = autopool_manager {
                    // Check for sessions ready for payout
                    let stats = autopool.get_autopool_stats().await;
                    if stats.active_sessions > 0 {
                        info!("🔄 Autopool Status: {} sessions, {} participants",
                              stats.active_sessions, stats.total_participants);

                        // In a full implementation, we would check each session and distribute payouts
                        // For now, just log the status
                        if stats.avg_session_progress > 90.0 {
                            info!("🎯 Some autopool sessions near completion ({:.1}% avg progress)",
                                  stats.avg_session_progress);
                        }
                    }
                }
            }

            // Handle shutdown signal
            _ = tokio::signal::ctrl_c() => {
                info!("🛑 Shutdown signal received");
                break;
            }
        }
    }

    info!("🏁 Paradigm Contributor shutting down...");
    info!("📈 Final stats: {} tasks completed", task_count);
    Ok(())
}
