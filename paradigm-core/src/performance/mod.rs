// Performance optimization modules for Paradigm cryptocurrency
// Phase 3: Optimization and Scaling

pub mod auto_scaling;
pub mod cache_manager;
pub mod consensus_optimization;
pub mod database_optimization;
pub mod memory_pool;
pub mod metrics_collector;
pub mod network_scaling;
pub mod parallel_processing;
pub mod transaction_batching;

#[allow(ambiguous_glob_reexports)]
pub use auto_scaling::*;
#[allow(ambiguous_glob_reexports)]
pub use cache_manager::*;
#[allow(ambiguous_glob_reexports)]
pub use consensus_optimization::*;
#[allow(ambiguous_glob_reexports)]
pub use database_optimization::*;
#[allow(ambiguous_glob_reexports)]
pub use memory_pool::*;
#[allow(ambiguous_glob_reexports)]
pub use metrics_collector::*;
#[allow(ambiguous_glob_reexports)]
pub use network_scaling::*;
#[allow(ambiguous_glob_reexports)]
pub use parallel_processing::*;
#[allow(ambiguous_glob_reexports)]
pub use transaction_batching::*;

use serde::{Deserialize, Serialize};
use std::time::Duration;

/// Performance metrics and monitoring
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceMetrics {
    pub transaction_throughput: f64, // TPS
    pub average_latency_ms: u64,     // milliseconds
    pub memory_usage: u64,           // bytes
    pub cpu_usage: f64,              // percentage
    pub disk_io: u64,                // bytes/sec
    pub network_io: u64,             // bytes/sec
    pub cache_hit_rate: f64,         // percentage
    pub consensus_time_ms: u64,      // milliseconds
    pub last_updated_timestamp: u64, // unix timestamp
}

impl Default for PerformanceMetrics {
    fn default() -> Self {
        Self {
            transaction_throughput: 0.0,
            average_latency_ms: 0,
            memory_usage: 0,
            cpu_usage: 0.0,
            disk_io: 0,
            network_io: 0,
            cache_hit_rate: 0.0,
            consensus_time_ms: 0,
            last_updated_timestamp: 0,
        }
    }
}

/// Performance optimization configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceConfig {
    pub enable_transaction_batching: bool,
    pub batch_size: usize,
    pub batch_timeout: Duration,
    pub enable_parallel_processing: bool,
    pub worker_threads: usize,
    pub enable_caching: bool,
    pub cache_size_mb: usize,
    pub enable_compression: bool,
    pub enable_prefetching: bool,
}

impl Default for PerformanceConfig {
    fn default() -> Self {
        Self {
            enable_transaction_batching: true,
            batch_size: 1000,
            batch_timeout: Duration::from_millis(100),
            enable_parallel_processing: true,
            worker_threads: num_cpus::get(),
            enable_caching: true,
            cache_size_mb: 512,
            enable_compression: true,
            enable_prefetching: true,
        }
    }
}
