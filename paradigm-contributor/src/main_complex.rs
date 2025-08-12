use paradigm_core::{ParadigmNode, ml_tasks::MLTaskEngine, consensus::MLTask};
use clap::{Arg, Command};
use tracing_subscriber;
use anyhow::Result;
use std::time::Duration;
use tokio::time::interval;

mod task_manager;
mod performance_monitor;
mod hardware_detector;

use task_manager::TaskManager;
use performance_monitor::PerformanceMonitor;
use hardware_detector::HardwareDetector;

/// Paradigm Contributor Application
/// 
/// This application connects to the Paradigm network and contributes computational
/// power for machine learning tasks in exchange for PAR rewards.
#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();

    let matches = Command::new("paradigm-contributor")
        .version("0.1.0")
        .about("Paradigm ML Contributor Client")
        .arg(
            Arg::new("node-address")
                .long("node-address")
                .value_name("ADDRESS")
                .help("Address of the Paradigm node to connect to")
                .default_value("127.0.0.1:8080"),
        )
        .arg(
            Arg::new("wallet-address")
                .long("wallet-address")
                .value_name("ADDRESS")
                .help("Wallet address to receive rewards")
                .required(true),
        )
        .arg(
            Arg::new("max-tasks")
                .long("max-tasks")
                .value_name("NUMBER")
                .help("Maximum number of concurrent tasks")
                .default_value("4"),
        )
        .arg(
            Arg::new("cpu-threads")
                .long("cpu-threads")
                .value_name("NUMBER")
                .help("Number of CPU threads to use")
                .default_value("0"), // 0 = auto-detect
        )
        .arg(
            Arg::new("use-gpu")
                .long("use-gpu")
                .help("Enable GPU acceleration if available")
                .action(clap::ArgAction::SetTrue),
        )
        .arg(
            Arg::new("task-types")
                .long("task-types")
                .value_name("TYPES")
                .help("Comma-separated list of task types to accept")
                .default_value("all"),
        )
        .get_matches();

    let node_address = matches.get_one::<String>("node-address").unwrap();
    let wallet_address = matches.get_one::<String>("wallet-address").unwrap();
    let max_tasks: usize = matches.get_one::<String>("max-tasks").unwrap().parse()?;
    let cpu_threads: usize = matches.get_one::<String>("cpu-threads").unwrap().parse()?;
    let use_gpu = matches.get_flag("use-gpu");
    let task_types_str = matches.get_one::<String>("task-types").unwrap();

    tracing::info!("Starting Paradigm Contributor");
    tracing::info!("Node address: {}", node_address);
    tracing::info!("Wallet address: {}", wallet_address);
    tracing::info!("Max concurrent tasks: {}", max_tasks);
    tracing::info!("GPU acceleration: {}", use_gpu);

    // Detect hardware capabilities
    let hardware = HardwareDetector::new().await?;
    hardware.log_capabilities();

    // Parse task types
    let task_types = if task_types_str == "all" {
        vec![] // Empty means all types
    } else {
        task_types_str.split(',').map(|s| s.trim().to_string()).collect()
    };

    // Create ML task engine
    let mut ml_engine = MLTaskEngine::new()?;
    tracing::info!("Supported task types: {:?}", ml_engine.get_supported_tasks());

    // Run benchmark
    tracing::info!("Running initial benchmark...");
    let benchmark_results = ml_engine.run_benchmark().await?;
    tracing::info!("Benchmark completed. Score: {:.2}", benchmark_results.total_score);

    // Create Paradigm node in contributor mode
    let node = ParadigmNode::new(true).await?;
    tracing::info!("Contributor node created with address: {}", node.address.to_string());

    // Start the node
    node.start().await?;

    // Create task manager
    let mut task_manager = TaskManager::new(
        node,
        ml_engine,
        wallet_address.clone(),
        max_tasks,
    ).await?;

    // Create performance monitor
    let mut performance_monitor = PerformanceMonitor::new();
    performance_monitor.start().await?;

    tracing::info!("Paradigm Contributor is now running...");
    tracing::info!("Waiting for ML tasks from the network...");

    // Main contributor loop
    let mut task_interval = interval(Duration::from_secs(10)); // Check for tasks every 10 seconds
    let mut stats_interval = interval(Duration::from_secs(60)); // Log stats every minute

    loop {
        tokio::select! {
            _ = task_interval.tick() => {
                // Check for new tasks
                if let Err(e) = task_manager.check_for_tasks().await {
                    tracing::error!("Error checking for tasks: {}", e);
                }
            }
            
            _ = stats_interval.tick() => {
                // Log performance statistics
                let stats = task_manager.get_statistics().await;
                let perf_stats = performance_monitor.get_current_stats().await;
                
                tracing::info!("=== Contributor Statistics ===");
                tracing::info!("Tasks completed: {}", stats.tasks_completed);
                tracing::info!("Total rewards earned: {:.8} PAR", stats.total_rewards as f64 / 100_000_000.0);
                tracing::info!("CPU usage: {:.1}%", perf_stats.cpu_usage);
                tracing::info!("Memory usage: {:.1}%", perf_stats.memory_usage);
                if use_gpu {
                    tracing::info!("GPU usage: {:.1}%", perf_stats.gpu_usage);
                }
                tracing::info!("Active tasks: {}", stats.active_tasks);
                tracing::info!("Average task time: {:.2}s", stats.average_task_time);
                tracing::info!("===============================");
            }
            
            _ = tokio::signal::ctrl_c() => {
                tracing::info!("Received shutdown signal");
                break;
            }
        }
    }

    // Graceful shutdown
    tracing::info!("Shutting down contributor...");
    task_manager.shutdown().await?;
    performance_monitor.shutdown().await?;
    
    let final_stats = task_manager.get_statistics().await;
    tracing::info!("Final statistics:");
    tracing::info!("Total tasks completed: {}", final_stats.tasks_completed);
    tracing::info!("Total rewards earned: {:.8} PAR", final_stats.total_rewards as f64 / 100_000_000.0);
    tracing::info!("Thanks for contributing to the Paradigm network!");

    Ok(())
}

/// Contributor statistics
#[derive(Debug, Clone)]
pub struct ContributorStats {
    pub tasks_completed: u64,
    pub total_rewards: u64,
    pub active_tasks: usize,
    pub average_task_time: f64,
    pub uptime_seconds: u64,
}

impl Default for ContributorStats {
    fn default() -> Self {
        ContributorStats {
            tasks_completed: 0,
            total_rewards: 0,
            active_tasks: 0,
            average_task_time: 0.0,
            uptime_seconds: 0,
        }
    }
}
