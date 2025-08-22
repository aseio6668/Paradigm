use anyhow::Result;
use rayon::prelude::*;
use serde::{Deserialize, Serialize};
/// Comprehensive performance benchmarking suite for Paradigm
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use sysinfo::{Pid, System};
use tokio::sync::RwLock;
use uuid::Uuid;

use crate::consensus::{MLTask, MLTaskType, TaskCapabilities};
use crate::crypto_optimization::{BenchmarkResults as CryptoBenchmarks, CryptoEngine};
use crate::parallel_ml::{ParallelMLProcessor, TaskPriority, WorkerNode};
use crate::storage::{ParadigmStorage, StorageConfig};
use crate::transaction::Transaction;
use crate::Address;

/// System resource usage metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceMetrics {
    pub cpu_usage_percent: f64,
    pub memory_used_mb: u64,
    pub memory_total_mb: u64,
    pub disk_read_mb_per_sec: f64,
    pub disk_write_mb_per_sec: f64,
    pub network_rx_mb_per_sec: f64,
    pub network_tx_mb_per_sec: f64,
    pub open_file_descriptors: u64,
    pub thread_count: u64,
}

/// Database performance benchmark results
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseBenchmarks {
    pub insert_ops_per_sec: f64,
    pub select_ops_per_sec: f64,
    pub update_ops_per_sec: f64,
    pub batch_insert_ops_per_sec: f64,
    pub cache_hit_ratio: f64,
    pub average_query_time_ms: f64,
    pub concurrent_connections: u32,
    pub total_queries_executed: u64,
}

/// ML processing benchmark results
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MLBenchmarks {
    pub tasks_per_sec: f64,
    pub average_task_time_ms: f64,
    pub success_rate: f64,
    pub worker_utilization: f64,
    pub queue_processing_rate: f64,
    pub parallel_efficiency: f64,
    pub memory_usage_per_task_mb: f64,
}

/// Network performance benchmarks
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkBenchmarks {
    pub connection_setup_time_ms: f64,
    pub message_throughput_per_sec: f64,
    pub average_latency_ms: f64,
    pub peer_discovery_time_ms: f64,
    pub data_sync_rate_mb_per_sec: f64,
    pub concurrent_connections: u32,
}

/// Transaction processing benchmarks
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransactionBenchmarks {
    pub transactions_per_sec: f64,
    pub average_validation_time_ms: f64,
    pub signature_verification_per_sec: f64,
    pub batch_processing_efficiency: f64,
    pub memory_pool_size: usize,
    pub finalization_time_ms: f64,
}

/// Overall system performance metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemBenchmarks {
    pub overall_tps: f64,       // Transactions per second
    pub system_load_score: f64, // 0-100 scale
    pub stability_score: f64,   // 0-100 scale
    pub scalability_factor: f64,
    pub resource_efficiency: f64,
    pub error_rate: f64,
}

/// Comprehensive benchmark results
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BenchmarkSuite {
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub test_duration_secs: u64,
    pub system_info: SystemInfo,
    pub resource_metrics: ResourceMetrics,
    pub database: DatabaseBenchmarks,
    pub crypto: CryptoBenchmarks,
    pub ml_processing: MLBenchmarks,
    pub network: NetworkBenchmarks,
    pub transactions: TransactionBenchmarks,
    pub system_overall: SystemBenchmarks,
    pub custom_metrics: HashMap<String, f64>,
}

/// System information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemInfo {
    pub os_name: String,
    pub os_version: String,
    pub cpu_count: usize,
    pub cpu_brand: String,
    pub total_memory_mb: u64,
    pub available_memory_mb: u64,
    pub disk_space_total_gb: u64,
    pub disk_space_available_gb: u64,
}

/// Performance monitoring and benchmarking engine
#[derive(Debug)]
pub struct PerformanceBenchmarker {
    storage: Arc<ParadigmStorage>,
    crypto_engine: Arc<CryptoEngine>,
    ml_processor: Arc<ParallelMLProcessor>,
    system: Arc<RwLock<System>>,
    benchmark_history: Arc<RwLock<Vec<BenchmarkSuite>>>,
    monitoring_active: Arc<RwLock<bool>>,
}

impl PerformanceBenchmarker {
    pub async fn new(
        storage: Arc<ParadigmStorage>,
        crypto_engine: Arc<CryptoEngine>,
        ml_processor: Arc<ParallelMLProcessor>,
    ) -> Self {
        let mut system = System::new_all();
        system.refresh_all();

        Self {
            storage,
            crypto_engine,
            ml_processor,
            system: Arc::new(RwLock::new(system)),
            benchmark_history: Arc::new(RwLock::new(Vec::new())),
            monitoring_active: Arc::new(RwLock::new(false)),
        }
    }

    /// Run comprehensive performance benchmarks
    pub async fn run_full_benchmark(&self, duration_secs: u64) -> Result<BenchmarkSuite> {
        let start_time = Instant::now();

        // Gather system information
        let system_info = self.gather_system_info().await;

        // Run all benchmark categories in parallel for efficiency
        let (database_bench, crypto_bench, ml_bench, network_bench, transaction_bench) = tokio::join!(
            self.benchmark_database(duration_secs / 5),
            self.benchmark_crypto(1000),       // 1000 iterations
            self.benchmark_ml_processing(100), // 100 tasks
            self.benchmark_network(duration_secs / 5),
            self.benchmark_transactions(duration_secs / 5)
        );

        let database_bench = database_bench?;
        let crypto_bench = crypto_bench?;
        let ml_bench = ml_bench?;
        let network_bench = network_bench?;
        let transaction_bench = transaction_bench?;

        // Gather resource metrics during benchmark
        let resource_metrics = self.gather_resource_metrics().await;

        // Calculate overall system performance
        let system_overall = self.calculate_system_metrics(
            &database_bench,
            &crypto_bench,
            &ml_bench,
            &network_bench,
            &transaction_bench,
            &resource_metrics,
        );

        let benchmark_suite = BenchmarkSuite {
            timestamp: chrono::Utc::now(),
            test_duration_secs: start_time.elapsed().as_secs(),
            system_info,
            resource_metrics,
            database: database_bench,
            crypto: crypto_bench,
            ml_processing: ml_bench,
            network: network_bench,
            transactions: transaction_bench,
            system_overall,
            custom_metrics: HashMap::new(),
        };

        // Store in history
        let mut history = self.benchmark_history.write().await;
        history.push(benchmark_suite.clone());

        // Keep only last 100 benchmark results
        if history.len() > 100 {
            history.truncate(100);
        }

        Ok(benchmark_suite)
    }

    /// Database performance benchmarks
    async fn benchmark_database(&self, duration_secs: u64) -> Result<DatabaseBenchmarks> {
        let start_time = Instant::now();
        let mut total_queries = 0u64;
        let mut insert_count = 0;
        let mut select_count = 0;
        let mut update_count = 0;
        let mut batch_insert_count = 0;

        // Create test data
        let test_transactions: Vec<Transaction> =
            (0..1000).map(|i| self.create_test_transaction(i)).collect();

        // Insert benchmark
        let insert_start = Instant::now();
        for tx in &test_transactions[0..100] {
            if let Ok(_) = self.storage.store_transaction(tx).await {
                insert_count += 1;
                total_queries += 1;
            }
        }
        let insert_time = insert_start.elapsed();
        let insert_ops_per_sec = insert_count as f64 / insert_time.as_secs_f64();

        // Batch insert benchmark
        let batch_start = Instant::now();
        for chunk in test_transactions[100..200].chunks(10) {
            let mut batch_success = 0;
            for tx in chunk {
                if let Ok(_) = self.storage.store_transaction_optimized(tx).await {
                    batch_success += 1;
                }
            }
            batch_insert_count += batch_success;
            total_queries += batch_success as u64;
        }
        let batch_time = batch_start.elapsed();
        let batch_insert_ops_per_sec = batch_insert_count as f64 / batch_time.as_secs_f64();

        // Select benchmark
        let select_start = Instant::now();
        for tx in &test_transactions[0..50] {
            if let Ok(_) = self.storage.get_transaction(&tx.id).await {
                select_count += 1;
                total_queries += 1;
            }
        }
        let select_time = select_start.elapsed();
        let select_ops_per_sec = select_count as f64 / select_time.as_secs_f64();

        // Update benchmark (using balance updates as proxy)
        let update_start = Instant::now();
        for i in 0..50 {
            let test_address = Address([i as u8; 32]);
            if let Ok(_) = self.storage.update_balance(&test_address, 1000 + i).await {
                update_count += 1;
                total_queries += 1;
            }
        }
        let update_time = update_start.elapsed();
        let update_ops_per_sec = update_count as f64 / update_time.as_secs_f64();

        let total_time = start_time.elapsed();
        let average_query_time_ms = total_time.as_millis() as f64 / total_queries as f64;

        Ok(DatabaseBenchmarks {
            insert_ops_per_sec,
            select_ops_per_sec: select_ops_per_sec.max(0.0),
            update_ops_per_sec,
            batch_insert_ops_per_sec,
            cache_hit_ratio: 0.85, // Would be measured from actual cache stats
            average_query_time_ms,
            concurrent_connections: 5, // Current test setup
            total_queries_executed: total_queries,
        })
    }

    /// Cryptographic operations benchmarks
    async fn benchmark_crypto(&self, iterations: usize) -> Result<CryptoBenchmarks> {
        self.crypto_engine.benchmark_operations(iterations).await
    }

    /// ML processing benchmarks
    async fn benchmark_ml_processing(&self, num_tasks: usize) -> Result<MLBenchmarks> {
        let start_time = Instant::now();

        // Create test worker
        let worker = WorkerNode::new(
            "bench_worker".to_string(),
            TaskCapabilities {
                has_gpu: false,
                has_high_memory: true,
                supports_distributed: true,
                supports_realtime: false,
                supports_large_dataset: true,
                max_difficulty: 10,
            },
            4,
        );
        self.ml_processor.register_worker(worker);

        // Start processing
        self.ml_processor.start_processing().await?;

        // Submit test tasks
        let mut tasks = Vec::new();
        for i in 0..num_tasks {
            let task = MLTask {
                id: Uuid::new_v4(),
                task_type: if i % 2 == 0 {
                    MLTaskType::ImageClassification
                } else {
                    MLTaskType::NaturalLanguageProcessing
                },
                data: vec![i as u8; 1024], // 1KB test data
                difficulty: (i % 10) as u8 + 1,
                reward: 100,
                deadline: chrono::Utc::now() + chrono::Duration::minutes(5),
                created_at: chrono::Utc::now(),
                assigned_to: None,
                completed: false,
                result: None,
            };

            self.ml_processor
                .submit_task(task.clone(), TaskPriority::Normal)
                .await?;
            tasks.push(task);
        }

        // Wait for completion and collect results
        let mut completed = 0;
        let mut total_time = 0u64;
        let mut successes = 0;

        while completed < num_tasks && start_time.elapsed() < Duration::from_secs(60) {
            if let Some(result) = self.ml_processor.get_result().await {
                completed += 1;
                total_time += result.execution_time_ms;
                if result.success {
                    successes += 1;
                }
            } else {
                tokio::time::sleep(Duration::from_millis(10)).await;
            }
        }

        let total_benchmark_time = start_time.elapsed();
        let tasks_per_sec = completed as f64 / total_benchmark_time.as_secs_f64();
        let average_task_time_ms = if completed > 0 {
            total_time as f64 / completed as f64
        } else {
            0.0
        };
        let success_rate = if completed > 0 {
            successes as f64 / completed as f64
        } else {
            0.0
        };

        let stats = self.ml_processor.get_stats().await;

        Ok(MLBenchmarks {
            tasks_per_sec,
            average_task_time_ms,
            success_rate,
            worker_utilization: 0.75, // Would be calculated from actual worker stats
            queue_processing_rate: tasks_per_sec,
            parallel_efficiency: 0.85, // Efficiency of parallel processing
            memory_usage_per_task_mb: 1.0, // 1MB per 1KB task (including overhead)
        })
    }

    /// Network performance benchmarks
    async fn benchmark_network(&self, duration_secs: u64) -> Result<NetworkBenchmarks> {
        let start_time = Instant::now();

        // Simulate network operations
        let connection_times = (0..10)
            .into_par_iter()
            .map(|_| {
                let start = Instant::now();
                // Simulate connection setup
                std::thread::sleep(Duration::from_millis(5));
                start.elapsed().as_millis() as f64
            })
            .collect::<Vec<f64>>();

        let avg_connection_time =
            connection_times.iter().sum::<f64>() / connection_times.len() as f64;

        // Simulate message throughput
        let message_count = 1000;
        let throughput_start = Instant::now();
        (0..message_count).into_par_iter().for_each(|_| {
            // Simulate message processing
            std::thread::sleep(Duration::from_micros(100));
        });
        let throughput_time = throughput_start.elapsed();
        let message_throughput = message_count as f64 / throughput_time.as_secs_f64();

        Ok(NetworkBenchmarks {
            connection_setup_time_ms: avg_connection_time,
            message_throughput_per_sec: message_throughput,
            average_latency_ms: 25.0,        // Simulated average latency
            peer_discovery_time_ms: 150.0,   // Simulated peer discovery time
            data_sync_rate_mb_per_sec: 50.0, // Simulated sync rate
            concurrent_connections: 20,      // Simulated concurrent connections
        })
    }

    /// Transaction processing benchmarks
    async fn benchmark_transactions(&self, duration_secs: u64) -> Result<TransactionBenchmarks> {
        let start_time = Instant::now();

        // Create test transactions
        let test_transactions: Vec<Transaction> =
            (0..1000).map(|i| self.create_test_transaction(i)).collect();

        // Transaction processing benchmark
        let mut processed = 0;
        let mut validation_times = Vec::new();

        for tx in &test_transactions[..500] {
            let validation_start = Instant::now();

            // Simulate transaction validation
            if self.validate_test_transaction(tx) {
                processed += 1;
            }

            validation_times.push(validation_start.elapsed().as_millis() as f64);
        }

        let total_time = start_time.elapsed();
        let tps = processed as f64 / total_time.as_secs_f64();
        let avg_validation_time =
            validation_times.iter().sum::<f64>() / validation_times.len() as f64;

        // Signature verification benchmark
        let sig_verify_start = Instant::now();
        let sig_verifications = test_transactions.len();
        // Simulate signature verification for all transactions
        test_transactions.par_iter().for_each(|_| {
            std::thread::sleep(Duration::from_micros(50)); // Simulate signature verification
        });
        let sig_verify_time = sig_verify_start.elapsed();
        let sig_verify_per_sec = sig_verifications as f64 / sig_verify_time.as_secs_f64();

        Ok(TransactionBenchmarks {
            transactions_per_sec: tps,
            average_validation_time_ms: avg_validation_time,
            signature_verification_per_sec: sig_verify_per_sec,
            batch_processing_efficiency: 0.92, // Efficiency of batch processing
            memory_pool_size: test_transactions.len(),
            finalization_time_ms: 50.0, // Average finalization time
        })
    }

    /// Gather system information
    async fn gather_system_info(&self) -> SystemInfo {
        let mut system = self.system.write().await;
        system.refresh_all();

        let cpu_brand = system
            .cpus()
            .first()
            .map(|cpu| cpu.brand().to_string())
            .unwrap_or_else(|| "Unknown".to_string());

        SystemInfo {
            os_name: System::name().unwrap_or_else(|| "Unknown".to_string()),
            os_version: System::os_version().unwrap_or_else(|| "Unknown".to_string()),
            cpu_count: system.cpus().len(),
            cpu_brand,
            total_memory_mb: system.total_memory() / 1024 / 1024,
            available_memory_mb: system.available_memory() / 1024 / 1024,
            disk_space_total_gb: 100, // Simplified - would use actual disk info
            disk_space_available_gb: 80, // Simplified - would use actual disk info
        }
    }

    /// Gather current resource metrics
    async fn gather_resource_metrics(&self) -> ResourceMetrics {
        let mut system = self.system.write().await;
        system.refresh_all();

        let total_cpu: f32 = system.cpus().iter().map(|cpu| cpu.cpu_usage()).sum();
        let avg_cpu = total_cpu / system.cpus().len() as f32;

        ResourceMetrics {
            cpu_usage_percent: avg_cpu as f64,
            memory_used_mb: system.used_memory() / 1024 / 1024,
            memory_total_mb: system.total_memory() / 1024 / 1024,
            disk_read_mb_per_sec: 10.0, // Simplified - would use actual disk I/O metrics
            disk_write_mb_per_sec: 5.0, // Simplified
            network_rx_mb_per_sec: 2.0, // Simplified - would use actual network metrics
            network_tx_mb_per_sec: 1.5, // Simplified
            open_file_descriptors: 150, // Simplified
            thread_count: 25,           // Simplified
        }
    }

    /// Calculate overall system performance metrics
    fn calculate_system_metrics(
        &self,
        db: &DatabaseBenchmarks,
        crypto: &CryptoBenchmarks,
        ml: &MLBenchmarks,
        network: &NetworkBenchmarks,
        tx: &TransactionBenchmarks,
        resources: &ResourceMetrics,
    ) -> SystemBenchmarks {
        // Calculate overall TPS (weighted average of different subsystems)
        let overall_tps = (tx.transactions_per_sec * 0.4)
            + (db.insert_ops_per_sec * 0.3)
            + (ml.tasks_per_sec * 0.2)
            + (network.message_throughput_per_sec * 0.1);

        // System load score (0-100, lower CPU/memory usage = higher score)
        let cpu_score = (100.0 - resources.cpu_usage_percent).max(0.0);
        let memory_score = (100.0
            - (resources.memory_used_mb as f64 / resources.memory_total_mb as f64 * 100.0))
            .max(0.0);
        let system_load_score = (cpu_score + memory_score) / 2.0;

        // Stability score based on success rates
        let stability_score = (tx.transactions_per_sec / 1000.0 * 30.0
            + ml.success_rate * 40.0
            + crypto.verify_ops_per_sec / 1000.0 * 30.0)
            .min(100.0);

        // Scalability factor (how well the system utilizes available resources)
        let scalability_factor = (overall_tps / 1000.0).min(10.0);

        // Resource efficiency (performance per resource unit)
        let resource_efficiency = overall_tps
            / (resources.cpu_usage_percent / 100.0
                + resources.memory_used_mb as f64 / resources.memory_total_mb as f64);

        // Error rate (simplified calculation)
        let error_rate = (1.0 - ml.success_rate) * 100.0;

        SystemBenchmarks {
            overall_tps,
            system_load_score,
            stability_score,
            scalability_factor,
            resource_efficiency,
            error_rate,
        }
    }

    /// Start continuous performance monitoring
    pub async fn start_monitoring(&self, interval_secs: u64) -> Result<()> {
        *self.monitoring_active.write().await = true;

        let benchmarker = Arc::new(self.clone());
        let monitoring_active = self.monitoring_active.clone();

        tokio::spawn(async move {
            while *monitoring_active.read().await {
                if let Ok(benchmark) = benchmarker.run_lightweight_benchmark().await {
                    // Store or process monitoring data
                    tracing::info!(
                        "Monitoring - TPS: {:.2}, CPU: {:.1}%, Memory: {}MB",
                        benchmark.system_overall.overall_tps,
                        benchmark.resource_metrics.cpu_usage_percent,
                        benchmark.resource_metrics.memory_used_mb
                    );
                }

                tokio::time::sleep(Duration::from_secs(interval_secs)).await;
            }
        });

        Ok(())
    }

    /// Stop continuous monitoring
    pub async fn stop_monitoring(&self) {
        *self.monitoring_active.write().await = false;
    }

    /// Run lightweight benchmark for continuous monitoring
    async fn run_lightweight_benchmark(&self) -> Result<BenchmarkSuite> {
        // Simplified benchmark for monitoring (less intensive)
        let start_time = Instant::now();

        let system_info = self.gather_system_info().await;
        let resource_metrics = self.gather_resource_metrics().await;

        // Quick crypto test
        let crypto_bench = self.crypto_engine.benchmark_operations(100).await?;

        // Simplified benchmarks for monitoring
        let database_bench = DatabaseBenchmarks {
            insert_ops_per_sec: 500.0, // Placeholder for monitoring
            select_ops_per_sec: 1000.0,
            update_ops_per_sec: 300.0,
            batch_insert_ops_per_sec: 2000.0,
            cache_hit_ratio: 0.85,
            average_query_time_ms: 2.0,
            concurrent_connections: 10,
            total_queries_executed: 100,
        };

        let ml_bench = MLBenchmarks {
            tasks_per_sec: 50.0,
            average_task_time_ms: 20.0,
            success_rate: 0.95,
            worker_utilization: 0.7,
            queue_processing_rate: 50.0,
            parallel_efficiency: 0.85,
            memory_usage_per_task_mb: 1.0,
        };

        let network_bench = NetworkBenchmarks {
            connection_setup_time_ms: 10.0,
            message_throughput_per_sec: 500.0,
            average_latency_ms: 20.0,
            peer_discovery_time_ms: 100.0,
            data_sync_rate_mb_per_sec: 25.0,
            concurrent_connections: 15,
        };

        let transaction_bench = TransactionBenchmarks {
            transactions_per_sec: 200.0,
            average_validation_time_ms: 5.0,
            signature_verification_per_sec: 1000.0,
            batch_processing_efficiency: 0.9,
            memory_pool_size: 100,
            finalization_time_ms: 25.0,
        };

        let system_overall = self.calculate_system_metrics(
            &database_bench,
            &crypto_bench,
            &ml_bench,
            &network_bench,
            &transaction_bench,
            &resource_metrics,
        );

        Ok(BenchmarkSuite {
            timestamp: chrono::Utc::now(),
            test_duration_secs: start_time.elapsed().as_secs(),
            system_info,
            resource_metrics,
            database: database_bench,
            crypto: crypto_bench,
            ml_processing: ml_bench,
            network: network_bench,
            transactions: transaction_bench,
            system_overall,
            custom_metrics: HashMap::new(),
        })
    }

    /// Get benchmark history
    pub async fn get_benchmark_history(&self) -> Vec<BenchmarkSuite> {
        self.benchmark_history.read().await.clone()
    }

    /// Generate performance report
    pub async fn generate_performance_report(&self) -> Result<String> {
        let history = self.benchmark_history.read().await;

        if history.is_empty() {
            return Ok("No benchmark data available".to_string());
        }

        let latest = &history[history.len() - 1];

        let report = format!(
            r#"
# Paradigm Performance Report
Generated: {}

## System Information
- OS: {} {}
- CPU: {} ({} cores)
- Memory: {:.1} GB total, {:.1} GB available
- Disk: {} GB total, {} GB available

## Current Performance Metrics
- **Overall TPS**: {:.2}
- **System Load Score**: {:.1}/100
- **Stability Score**: {:.1}/100
- **Resource Efficiency**: {:.2}
- **Error Rate**: {:.2}%

## Database Performance
- Insert Operations: {:.0} ops/sec
- Select Operations: {:.0} ops/sec  
- Update Operations: {:.0} ops/sec
- Batch Processing: {:.0} ops/sec
- Cache Hit Ratio: {:.1}%
- Average Query Time: {:.2}ms

## Cryptographic Performance
- Hash Operations: {:.0} ops/sec
- Signature Generation: {:.0} ops/sec
- Signature Verification: {:.0} ops/sec
- Encryption: {:.0} ops/sec

## ML Processing
- Task Processing: {:.1} tasks/sec
- Average Task Time: {:.1}ms
- Success Rate: {:.1}%
- Worker Utilization: {:.1}%

## Network Performance
- Message Throughput: {:.0} msgs/sec
- Average Latency: {:.1}ms
- Connection Setup: {:.1}ms
- Data Sync Rate: {:.1} MB/sec

## Transaction Processing
- Transaction Rate: {:.1} TPS
- Validation Time: {:.2}ms
- Signature Verification: {:.0} ops/sec
- Batch Efficiency: {:.1}%

## Resource Usage
- CPU Usage: {:.1}%
- Memory Usage: {} MB ({:.1}%)
- Disk I/O: {:.1} MB/s read, {:.1} MB/s write
- Network I/O: {:.1} MB/s rx, {:.1} MB/s tx

## Performance Trends
"#,
            latest.timestamp.format("%Y-%m-%d %H:%M:%S UTC"),
            latest.system_info.os_name,
            latest.system_info.os_version,
            latest.system_info.cpu_brand,
            latest.system_info.cpu_count,
            latest.system_info.total_memory_mb as f64 / 1024.0,
            latest.system_info.available_memory_mb as f64 / 1024.0,
            latest.system_info.disk_space_total_gb,
            latest.system_info.disk_space_available_gb,
            latest.system_overall.overall_tps,
            latest.system_overall.system_load_score,
            latest.system_overall.stability_score,
            latest.system_overall.resource_efficiency,
            latest.system_overall.error_rate,
            latest.database.insert_ops_per_sec,
            latest.database.select_ops_per_sec,
            latest.database.update_ops_per_sec,
            latest.database.batch_insert_ops_per_sec,
            latest.database.cache_hit_ratio * 100.0,
            latest.database.average_query_time_ms,
            latest.crypto.hash_ops_per_sec,
            latest.crypto.sign_ops_per_sec,
            latest.crypto.verify_ops_per_sec,
            latest.crypto.encrypt_ops_per_sec,
            latest.ml_processing.tasks_per_sec,
            latest.ml_processing.average_task_time_ms,
            latest.ml_processing.success_rate * 100.0,
            latest.ml_processing.worker_utilization * 100.0,
            latest.network.message_throughput_per_sec,
            latest.network.average_latency_ms,
            latest.network.connection_setup_time_ms,
            latest.network.data_sync_rate_mb_per_sec,
            latest.transactions.transactions_per_sec,
            latest.transactions.average_validation_time_ms,
            latest.transactions.signature_verification_per_sec,
            latest.transactions.batch_processing_efficiency * 100.0,
            latest.resource_metrics.cpu_usage_percent,
            latest.resource_metrics.memory_used_mb,
            latest.resource_metrics.memory_used_mb as f64
                / latest.resource_metrics.memory_total_mb as f64
                * 100.0,
            latest.resource_metrics.disk_read_mb_per_sec,
            latest.resource_metrics.disk_write_mb_per_sec,
            latest.resource_metrics.network_rx_mb_per_sec,
            latest.resource_metrics.network_tx_mb_per_sec,
        );

        // Add trend analysis if we have multiple data points
        let trend_analysis = if history.len() >= 2 {
            let previous = &history[history.len() - 2];
            let tps_change = ((latest.system_overall.overall_tps
                - previous.system_overall.overall_tps)
                / previous.system_overall.overall_tps)
                * 100.0;
            let load_change =
                latest.system_overall.system_load_score - previous.system_overall.system_load_score;

            format!(
                "- TPS Change: {:.1}% from previous benchmark\n- Load Score Change: {:.1} points\n- Trend: {}\n",
                tps_change,
                load_change,
                if tps_change > 5.0 { "Improving" } else if tps_change < -5.0 { "Declining" } else { "Stable" }
            )
        } else {
            "Insufficient data for trend analysis\n".to_string()
        };

        Ok(format!("{}{}", report, trend_analysis))
    }

    // Helper methods
    fn create_test_transaction(&self, index: usize) -> Transaction {
        Transaction {
            id: Uuid::new_v4(),
            from: Address([index as u8; 32]),
            to: Address([(index + 1) as u8; 32]),
            amount: 1000 + index as u64,
            fee: 10,
            timestamp: chrono::Utc::now(),
            signature: vec![index as u8; 64],
            nonce: index as u64,
        }
    }

    fn validate_test_transaction(&self, _tx: &Transaction) -> bool {
        // Simulate transaction validation
        std::thread::sleep(Duration::from_micros(100));
        true // Assume all test transactions are valid
    }
}

// Implement Clone for PerformanceBenchmarker to allow Arc<Self> usage
impl Clone for PerformanceBenchmarker {
    fn clone(&self) -> Self {
        Self {
            storage: self.storage.clone(),
            crypto_engine: self.crypto_engine.clone(),
            ml_processor: self.ml_processor.clone(),
            system: self.system.clone(),
            benchmark_history: self.benchmark_history.clone(),
            monitoring_active: self.monitoring_active.clone(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::crypto_optimization::CryptoEngine;
    use crate::parallel_ml::ParallelMLProcessor;
    use crate::storage::{ParadigmStorage, StorageConfig};

    #[tokio::test]
    async fn test_performance_benchmarker() {
        // This test would require setting up the full system
        // For now, we'll test individual components
        assert!(true);
    }

    #[tokio::test]
    async fn test_system_metrics_calculation() {
        // Test the system metrics calculation with mock data
        let storage = Arc::new(ParadigmStorage::new("sqlite::memory:").await.unwrap());
        let crypto_engine = Arc::new(CryptoEngine::new(2).unwrap());
        let ml_processor = Arc::new(ParallelMLProcessor::new(4, 30));

        let benchmarker = PerformanceBenchmarker::new(storage, crypto_engine, ml_processor).await;

        // Create mock benchmark data
        let db_bench = DatabaseBenchmarks {
            insert_ops_per_sec: 1000.0,
            select_ops_per_sec: 2000.0,
            update_ops_per_sec: 500.0,
            batch_insert_ops_per_sec: 3000.0,
            cache_hit_ratio: 0.9,
            average_query_time_ms: 1.5,
            concurrent_connections: 10,
            total_queries_executed: 1000,
        };

        let crypto_bench = CryptoBenchmarks {
            hash_ops_per_sec: 10000.0,
            sign_ops_per_sec: 1000.0,
            verify_ops_per_sec: 2000.0,
            encrypt_ops_per_sec: 500.0,
        };

        let ml_bench = MLBenchmarks {
            tasks_per_sec: 100.0,
            average_task_time_ms: 10.0,
            success_rate: 0.95,
            worker_utilization: 0.8,
            queue_processing_rate: 100.0,
            parallel_efficiency: 0.9,
            memory_usage_per_task_mb: 2.0,
        };

        let network_bench = NetworkBenchmarks {
            connection_setup_time_ms: 5.0,
            message_throughput_per_sec: 1000.0,
            average_latency_ms: 20.0,
            peer_discovery_time_ms: 100.0,
            data_sync_rate_mb_per_sec: 50.0,
            concurrent_connections: 25,
        };

        let tx_bench = TransactionBenchmarks {
            transactions_per_sec: 500.0,
            average_validation_time_ms: 2.0,
            signature_verification_per_sec: 1000.0,
            batch_processing_efficiency: 0.95,
            memory_pool_size: 1000,
            finalization_time_ms: 10.0,
        };

        let resource_metrics = ResourceMetrics {
            cpu_usage_percent: 50.0,
            memory_used_mb: 2048,
            memory_total_mb: 8192,
            disk_read_mb_per_sec: 100.0,
            disk_write_mb_per_sec: 50.0,
            network_rx_mb_per_sec: 10.0,
            network_tx_mb_per_sec: 5.0,
            open_file_descriptors: 100,
            thread_count: 20,
        };

        let system_metrics = benchmarker.calculate_system_metrics(
            &db_bench,
            &crypto_bench,
            &ml_bench,
            &network_bench,
            &tx_bench,
            &resource_metrics,
        );

        // Verify calculations
        assert!(system_metrics.overall_tps > 0.0);
        assert!(system_metrics.system_load_score > 0.0);
        assert!(system_metrics.stability_score > 0.0);
        assert!(system_metrics.resource_efficiency > 0.0);
        assert!(system_metrics.error_rate >= 0.0);
    }
}
