// Paradigm Contributor - ML Task Processing Client with GPU Acceleration
use paradigm_core::{MLTask, ParadigmError};
use clap::Parser;
use anyhow::Result;
use std::time::Duration;
use tracing::{info, warn, error};
use uuid::Uuid;
use chrono::{DateTime, Utc};

mod task_manager;
mod performance_monitor;
//mod gpu_compute;

use task_manager::TaskManager;
use performance_monitor::PerformanceMonitor;
//use gpu_compute::{GpuComputeEngine, GpuBackend};

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
    let mut task_manager = TaskManager::new(num_workers).await?;
    let mut performance_monitor = PerformanceMonitor::new();

    info!("✅ Contributor initialized successfully");
    info!("🔄 Starting main processing loop...");

    // Main processing loop with enhanced GPU support
    let mut task_count = 0u64;
    loop {
        tokio::select! {
            // Check for new tasks and process them with GPU acceleration
            _ = tokio::time::sleep(Duration::from_secs(5)) => {
                // In a real implementation, this would connect to the Paradigm network
                // and fetch actual ML tasks. For now, we simulate task processing.
                
                // Generate a mock ML task
                let mock_task = MLTask {
                    id: Uuid::new_v4(),
                    task_type: paradigm_core::consensus::MLTaskType::ImageClassification,
                    data: vec![0u8; 1024 * (1 + task_count % 10) as usize], // Variable size
                    difficulty: 1 + (task_count % 5) as u8,
                    reward: 1000000 * (1 + task_count % 3), // Variable reward
                    deadline: chrono::Utc::now() + chrono::Duration::minutes(30),
                    created_at: chrono::Utc::now(),
                    assigned_to: None,
                    completed: false,
                    result: None,
                };

                // Process the task using CPU processing (for now)
                let start_time = std::time::Instant::now();
                // Simulate task processing - in real implementation would fetch actual tasks
                tokio::time::sleep(Duration::from_millis(500)).await;
                
                task_count += 1;
                let completion_time = start_time.elapsed();
                let reward_par = mock_task.reward as f64 / 100_000_000.0;
                
                info!("✅ Completed task #{} in {:?} - earned {:.8} PAR", 
                      task_count, completion_time, reward_par);
                
                // Update performance stats
                performance_monitor.record_task_completion(completion_time);
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
