// Paradigm Contributor - ML Task Processing Client (Simplified)
use paradigm_core::ParadigmNode;
use clap::Parser;
use anyhow::Result;
use std::time::Duration;
use tracing::{info, warn};

mod task_manager;
mod performance_monitor;

use task_manager::TaskManager;
use performance_monitor::PerformanceMonitor;

#[derive(Parser)]
#[command(name = "paradigm-contributor")]
#[command(about = "Paradigm ML Contributor Client")]
pub struct Args {
    /// Node address to connect to
    #[arg(long, default_value = "127.0.0.1:8080")]
    node_address: String,
    
    /// Number of worker threads
    #[arg(long, default_value_t = num_cpus::get())]
    threads: usize,
    
    /// Enable verbose logging
    #[arg(short, long)]
    verbose: bool,
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

    info!("Starting Paradigm Contributor v0.1.0");
    info!("Connecting to node at: {}", args.node_address);
    info!("Using {} worker threads", args.threads);

    // Initialize components
    let mut task_manager = TaskManager::new(args.threads).await?;
    let mut performance_monitor = PerformanceMonitor::new();

    info!("Contributor initialized successfully");
    info!("Starting main processing loop...");

    // Main processing loop
    loop {
        tokio::select! {
            // Check for new tasks periodically
            _ = tokio::time::sleep(Duration::from_secs(10)) => {
                if let Err(e) = task_manager.fetch_new_tasks(&args.node_address).await {
                    warn!("Failed to fetch new tasks: {}", e);
                }
            }
            
            // Process pending tasks
            _ = task_manager.process_tasks() => {
                // Task processing completed
            }
            
            // Update performance metrics
            _ = tokio::time::sleep(Duration::from_secs(30)) => {
                performance_monitor.update_metrics().await;
                
                let stats = performance_monitor.get_stats();
                info!("Performance: {} tasks completed, avg time: {:?}", 
                      stats.total_tasks, stats.average_time);
            }
            
            // Handle shutdown signal
            _ = tokio::signal::ctrl_c() => {
                info!("Shutdown signal received");
                break;
            }
        }
    }

    info!("Paradigm Contributor shutting down...");
    Ok(())
}
