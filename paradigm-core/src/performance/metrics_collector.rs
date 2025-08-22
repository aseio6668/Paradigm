// Performance Metrics Collection and Analysis
// Collects and analyzes performance data for optimization insights

use std::collections::{HashMap, VecDeque};
use std::sync::Arc;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};
use tokio::sync::RwLock;
use serde::{Serialize, Deserialize};
use tracing::{info, debug};

/// Comprehensive performance metrics collector
pub struct MetricsCollector {
    metrics: Arc<RwLock<PerformanceMetrics>>,
    historical_data: Arc<RwLock<HistoricalMetrics>>,
    real_time_stats: Arc<RwLock<RealTimeStats>>,
    collection_config: MetricsConfig,
}

/// Performance metrics configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricsConfig {
    pub collection_interval: Duration,
    pub history_retention_days: u32,
    pub enable_real_time_analysis: bool,
    pub enable_trend_detection: bool,
    pub alert_thresholds: AlertThresholds,
}

/// Alert threshold configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlertThresholds {
    pub max_latency_ms: u64,
    pub min_throughput_tps: f64,
    pub max_memory_usage_mb: u64,
    pub max_cpu_usage_percent: f64,
    pub min_cache_hit_rate: f64,
}

impl Default for MetricsConfig {
    fn default() -> Self {
        Self {
            collection_interval: Duration::from_secs(10),
            history_retention_days: 30,
            enable_real_time_analysis: true,
            enable_trend_detection: true,
            alert_thresholds: AlertThresholds {
                max_latency_ms: 1000,
                min_throughput_tps: 1000.0,
                max_memory_usage_mb: 2048,
                max_cpu_usage_percent: 80.0,
                min_cache_hit_rate: 0.8,
            },
        }
    }
}

/// Current performance metrics
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct PerformanceMetrics {
    pub timestamp: u64,
    pub transaction_throughput: f64,
    pub average_latency: Duration,
    pub memory_usage_mb: f64,
    pub cpu_usage_percent: f64,
    pub network_io_mbps: f64,
    pub disk_io_mbps: f64,
    pub cache_hit_rate: f64,
    pub active_connections: u32,
    pub queue_depths: HashMap<String, usize>,
    pub error_rates: HashMap<String, f64>,
}

/// Historical metrics storage
#[derive(Debug)]
pub struct HistoricalMetrics {
    pub hourly_snapshots: VecDeque<PerformanceMetrics>,
    pub daily_summaries: VecDeque<DailySummary>,
    pub trend_analysis: TrendAnalysis,
}

/// Daily performance summary
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DailySummary {
    pub date: String,
    pub avg_throughput: f64,
    pub peak_throughput: f64,
    pub avg_latency: Duration,
    pub p95_latency: Duration,
    pub total_transactions: u64,
    pub uptime_percentage: f64,
    pub major_incidents: u32,
}

/// Trend analysis data
#[derive(Debug, Default, Clone)]
pub struct TrendAnalysis {
    pub throughput_trend: TrendDirection,
    pub latency_trend: TrendDirection,
    pub memory_trend: TrendDirection,
    pub predicted_bottlenecks: Vec<BottleneckPrediction>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum TrendDirection {
    Increasing,
    Decreasing,
    Stable,
    Volatile,
}

impl Default for TrendDirection {
    fn default() -> Self {
        TrendDirection::Stable
    }
}

/// Bottleneck prediction
#[derive(Debug, Clone)]
pub struct BottleneckPrediction {
    pub component: String,
    pub severity: AlertSeverity,
    pub estimated_time_to_impact: Duration,
    pub recommended_actions: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum AlertSeverity {
    Info,
    Warning,
    Error,
    Critical,
}

/// Real-time statistics
#[derive(Debug, Default, Clone)]
pub struct RealTimeStats {
    pub current_tps: f64,
    pub last_minute_avg: f64,
    pub last_hour_avg: f64,
    pub peak_tps_today: f64,
    pub bottleneck_indicators: HashMap<String, f64>,
}

impl MetricsCollector {
    pub fn new(config: MetricsConfig) -> Self {
        let collector = Self {
            metrics: Arc::new(RwLock::new(PerformanceMetrics::default())),
            historical_data: Arc::new(RwLock::new(HistoricalMetrics::new())),
            real_time_stats: Arc::new(RwLock::new(RealTimeStats::default())),
            collection_config: config,
        };
        
        // Start background collection
        collector.start_collection_loop();
        
        collector
    }

    /// Record transaction processing metrics
    pub async fn record_transaction_batch(&self, transaction_count: usize, processing_time: Duration) {
        let tps = transaction_count as f64 / processing_time.as_secs_f64();
        
        let mut metrics = self.metrics.write().await;
        metrics.transaction_throughput = tps;
        metrics.average_latency = processing_time / transaction_count as u32;
        metrics.timestamp = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();
        
        // Update real-time stats
        if self.collection_config.enable_real_time_analysis {
            let mut real_time = self.real_time_stats.write().await;
            real_time.current_tps = tps;
            
            // Update peak if necessary
            if tps > real_time.peak_tps_today {
                real_time.peak_tps_today = tps;
            }
        }
    }

    /// Record cache performance metrics
    pub async fn record_cache_metrics(&self, hit_rate: f64, memory_usage_mb: f64) {
        let mut metrics = self.metrics.write().await;
        metrics.cache_hit_rate = hit_rate;
        metrics.memory_usage_mb = memory_usage_mb;
    }

    /// Record system resource usage
    pub async fn record_system_metrics(&self, cpu_percent: f64, network_mbps: f64, disk_mbps: f64) {
        let mut metrics = self.metrics.write().await;
        metrics.cpu_usage_percent = cpu_percent;
        metrics.network_io_mbps = network_mbps;
        metrics.disk_io_mbps = disk_mbps;
    }

    /// Record queue depths for different components
    pub async fn record_queue_depth(&self, component: String, depth: usize) {
        let mut metrics = self.metrics.write().await;
        metrics.queue_depths.insert(component, depth);
    }

    /// Record error rates
    pub async fn record_error_rate(&self, component: String, error_rate: f64) {
        let mut metrics = self.metrics.write().await;
        metrics.error_rates.insert(component, error_rate);
    }

    /// Get current performance snapshot
    pub async fn get_current_metrics(&self) -> PerformanceMetrics {
        self.metrics.read().await.clone()
    }

    /// Get real-time statistics
    pub async fn get_real_time_stats(&self) -> RealTimeStats {
        self.real_time_stats.read().await.clone()
    }

    /// Get trend analysis
    pub async fn get_trend_analysis(&self) -> TrendAnalysis {
        let historical = self.historical_data.read().await;
        historical.trend_analysis.clone()
    }

    /// Get daily summary for a specific date
    pub async fn get_daily_summary(&self, date: &str) -> Option<DailySummary> {
        let historical = self.historical_data.read().await;
        historical.daily_summaries.iter()
            .find(|summary| summary.date == date)
            .cloned()
    }

    /// Analyze performance bottlenecks
    pub async fn analyze_bottlenecks(&self) -> Vec<BottleneckPrediction> {
        let metrics = self.metrics.read().await;
        let mut predictions = Vec::new();
        
        // Analyze memory usage
        if metrics.memory_usage_mb > self.collection_config.alert_thresholds.max_memory_usage_mb as f64 * 0.8 {
            predictions.push(BottleneckPrediction {
                component: "Memory".to_string(),
                severity: if metrics.memory_usage_mb > self.collection_config.alert_thresholds.max_memory_usage_mb as f64 {
                    AlertSeverity::Critical
                } else {
                    AlertSeverity::Warning
                },
                estimated_time_to_impact: Duration::from_secs(1800), // 30 minutes
                recommended_actions: vec![
                    "Increase cache eviction rate".to_string(),
                    "Scale up memory allocation".to_string(),
                    "Enable memory compression".to_string(),
                ],
            });
        }
        
        // Analyze CPU usage
        if metrics.cpu_usage_percent > self.collection_config.alert_thresholds.max_cpu_usage_percent * 0.8 {
            predictions.push(BottleneckPrediction {
                component: "CPU".to_string(),
                severity: if metrics.cpu_usage_percent > self.collection_config.alert_thresholds.max_cpu_usage_percent {
                    AlertSeverity::Critical
                } else {
                    AlertSeverity::Warning
                },
                estimated_time_to_impact: Duration::from_secs(900), // 15 minutes
                recommended_actions: vec![
                    "Increase parallel processing".to_string(),
                    "Scale out to more nodes".to_string(),
                    "Optimize algorithm efficiency".to_string(),
                ],
            });
        }
        
        // Analyze throughput
        if metrics.transaction_throughput < self.collection_config.alert_thresholds.min_throughput_tps {
            predictions.push(BottleneckPrediction {
                component: "Transaction Processing".to_string(),
                severity: AlertSeverity::Warning,
                estimated_time_to_impact: Duration::from_secs(600), // 10 minutes
                recommended_actions: vec![
                    "Enable transaction batching".to_string(),
                    "Optimize database queries".to_string(),
                    "Increase worker thread count".to_string(),
                ],
            });
        }
        
        // Analyze cache hit rate
        if metrics.cache_hit_rate < self.collection_config.alert_thresholds.min_cache_hit_rate {
            predictions.push(BottleneckPrediction {
                component: "Cache System".to_string(),
                severity: AlertSeverity::Warning,
                estimated_time_to_impact: Duration::from_secs(300), // 5 minutes
                recommended_actions: vec![
                    "Increase cache size".to_string(),
                    "Improve cache warming strategies".to_string(),
                    "Optimize cache eviction policy".to_string(),
                ],
            });
        }
        
        predictions
    }

    /// Generate performance optimization recommendations
    pub async fn get_optimization_recommendations(&self) -> Vec<OptimizationRecommendation> {
        let metrics = self.metrics.read().await;
        let historical = self.historical_data.read().await;
        let mut recommendations = Vec::new();
        
        // Memory optimization
        if metrics.memory_usage_mb > 1024.0 {
            recommendations.push(OptimizationRecommendation {
                category: "Memory".to_string(),
                priority: OptimizationPriority::High,
                description: "High memory usage detected".to_string(),
                actions: vec![
                    "Enable data compression".to_string(),
                    "Implement more aggressive cache eviction".to_string(),
                    "Consider memory-mapped files for large datasets".to_string(),
                ],
                expected_improvement: "20-30% memory reduction".to_string(),
            });
        }
        
        // Throughput optimization
        if metrics.transaction_throughput < 5000.0 {
            recommendations.push(OptimizationRecommendation {
                category: "Throughput".to_string(),
                priority: OptimizationPriority::High,
                description: "Transaction throughput below optimal".to_string(),
                actions: vec![
                    "Implement parallel transaction processing".to_string(),
                    "Optimize database write batching".to_string(),
                    "Enable speculative execution".to_string(),
                ],
                expected_improvement: "50-100% throughput increase".to_string(),
            });
        }
        
        // Latency optimization
        if metrics.average_latency > Duration::from_millis(100) {
            recommendations.push(OptimizationRecommendation {
                category: "Latency".to_string(),
                priority: OptimizationPriority::Medium,
                description: "Average latency higher than target".to_string(),
                actions: vec![
                    "Implement request pipelining".to_string(),
                    "Optimize critical path algorithms".to_string(),
                    "Add more aggressive caching".to_string(),
                ],
                expected_improvement: "30-50% latency reduction".to_string(),
            });
        }
        
        recommendations
    }

    /// Start background metrics collection
    fn start_collection_loop(&self) {
        let collector = self.clone();
        tokio::spawn(async move {
            collector.collection_loop().await;
        });
        
        if self.collection_config.enable_trend_detection {
            let collector = self.clone();
            tokio::spawn(async move {
                collector.trend_analysis_loop().await;
            });
        }
    }

    /// Main collection loop
    async fn collection_loop(&self) {
        let mut interval = tokio::time::interval(self.collection_config.collection_interval);
        
        loop {
            interval.tick().await;
            
            // Collect system metrics
            if let Err(e) = self.collect_system_metrics().await {
                debug!("Failed to collect system metrics: {}", e);
            }
            
            // Update historical data
            if let Err(e) = self.update_historical_data().await {
                debug!("Failed to update historical data: {}", e);
            }
            
            // Check for alerts
            if let Err(e) = self.check_alert_conditions().await {
                debug!("Failed to check alert conditions: {}", e);
            }
        }
    }

    /// Trend analysis loop
    async fn trend_analysis_loop(&self) {
        let mut interval = tokio::time::interval(Duration::from_secs(300)); // 5 minutes
        
        loop {
            interval.tick().await;
            
            if let Err(e) = self.analyze_trends().await {
                debug!("Failed to analyze trends: {}", e);
            }
        }
    }

    /// Collect current system metrics
    async fn collect_system_metrics(&self) -> Result<(), Box<dyn std::error::Error>> {
        // Mock system metrics collection
        // In real implementation, this would use system APIs
        
        let cpu_usage = 45.0; // Mock CPU usage
        let memory_usage = 1024.0; // Mock memory usage in MB
        let network_io = 100.0; // Mock network I/O in Mbps
        let disk_io = 50.0; // Mock disk I/O in Mbps
        
        self.record_system_metrics(cpu_usage, network_io, disk_io).await;
        
        let mut metrics = self.metrics.write().await;
        metrics.memory_usage_mb = memory_usage;
        metrics.active_connections = 150; // Mock connection count
        
        Ok(())
    }

    /// Update historical data with current metrics
    async fn update_historical_data(&self) -> Result<(), Box<dyn std::error::Error>> {
        let current_metrics = self.metrics.read().await.clone();
        let mut historical = self.historical_data.write().await;
        
        // Add to hourly snapshots
        historical.hourly_snapshots.push_back(current_metrics);
        
        // Keep only last 24 hours of snapshots
        while historical.hourly_snapshots.len() > 24 {
            historical.hourly_snapshots.pop_front();
        }
        
        // Generate daily summary if it's a new day
        self.maybe_generate_daily_summary(&mut historical).await;
        
        Ok(())
    }

    /// Generate daily summary if needed
    async fn maybe_generate_daily_summary(&self, historical: &mut HistoricalMetrics) {
        // Check if we need to generate a new daily summary
        let today = chrono::Utc::now().format("%Y-%m-%d").to_string();
        
        if historical.daily_summaries.is_empty() || 
           historical.daily_summaries.back().unwrap().date != today {
            
            if !historical.hourly_snapshots.is_empty() {
                let summary = self.calculate_daily_summary(&historical.hourly_snapshots).await;
                historical.daily_summaries.push_back(summary);
                
                // Keep only last 30 days
                while historical.daily_summaries.len() > self.collection_config.history_retention_days as usize {
                    historical.daily_summaries.pop_front();
                }
            }
        }
    }

    /// Calculate daily summary from hourly snapshots
    async fn calculate_daily_summary(&self, snapshots: &VecDeque<PerformanceMetrics>) -> DailySummary {
        let total_throughput: f64 = snapshots.iter().map(|s| s.transaction_throughput).sum();
        let avg_throughput = total_throughput / snapshots.len() as f64;
        let peak_throughput = snapshots.iter().map(|s| s.transaction_throughput).fold(0.0, f64::max);
        
        let total_latency_ms: u64 = snapshots.iter().map(|s| s.average_latency.as_millis() as u64).sum();
        let avg_latency = Duration::from_millis(total_latency_ms / snapshots.len() as u64);
        
        // Calculate P95 latency (simplified)
        let mut latencies: Vec<_> = snapshots.iter().map(|s| s.average_latency).collect();
        latencies.sort();
        let p95_index = (latencies.len() as f64 * 0.95) as usize;
        let p95_latency = latencies.get(p95_index).cloned().unwrap_or(Duration::from_millis(0));
        
        DailySummary {
            date: chrono::Utc::now().format("%Y-%m-%d").to_string(),
            avg_throughput,
            peak_throughput,
            avg_latency,
            p95_latency,
            total_transactions: (avg_throughput * 24.0 * 3600.0) as u64, // Estimated
            uptime_percentage: 99.9, // Mock uptime
            major_incidents: 0, // Mock incident count
        }
    }

    /// Analyze performance trends
    async fn analyze_trends(&self) -> Result<(), Box<dyn std::error::Error>> {
        let historical = self.historical_data.read().await;
        
        if historical.hourly_snapshots.len() < 2 {
            return Ok(());
        }
        
        let mut trend_analysis = TrendAnalysis::default();
        
        // Analyze throughput trend
        trend_analysis.throughput_trend = self.analyze_metric_trend(
            &historical.hourly_snapshots.iter().map(|s| s.transaction_throughput).collect::<Vec<_>>()
        );
        
        // Analyze latency trend (convert to f64 for analysis)
        let latency_values: Vec<f64> = historical.hourly_snapshots.iter()
            .map(|s| s.average_latency.as_millis() as f64)
            .collect();
        trend_analysis.latency_trend = self.analyze_metric_trend(&latency_values);
        
        // Analyze memory trend
        trend_analysis.memory_trend = self.analyze_metric_trend(
            &historical.hourly_snapshots.iter().map(|s| s.memory_usage_mb).collect::<Vec<_>>()
        );
        
        // Generate bottleneck predictions based on trends
        trend_analysis.predicted_bottlenecks = self.analyze_bottlenecks().await;
        
        // Update stored trend analysis
        let mut historical_mut = self.historical_data.write().await;
        historical_mut.trend_analysis = trend_analysis;
        
        Ok(())
    }

    /// Analyze trend direction for a metric
    fn analyze_metric_trend(&self, values: &[f64]) -> TrendDirection {
        if values.len() < 3 {
            return TrendDirection::Stable;
        }
        
        let recent_values = &values[values.len().saturating_sub(6)..]; // Last 6 data points
        let mut increasing_count = 0;
        let mut decreasing_count = 0;
        
        for window in recent_values.windows(2) {
            if window[1] > window[0] * 1.05 { // 5% increase threshold
                increasing_count += 1;
            } else if window[1] < window[0] * 0.95 { // 5% decrease threshold
                decreasing_count += 1;
            }
        }
        
        let total_comparisons = recent_values.len() - 1;
        let volatility_threshold = total_comparisons / 2;
        
        if increasing_count >= volatility_threshold && decreasing_count >= volatility_threshold {
            TrendDirection::Volatile
        } else if increasing_count > decreasing_count {
            TrendDirection::Increasing
        } else if decreasing_count > increasing_count {
            TrendDirection::Decreasing
        } else {
            TrendDirection::Stable
        }
    }

    /// Check alert conditions
    async fn check_alert_conditions(&self) -> Result<(), Box<dyn std::error::Error>> {
        let metrics = self.metrics.read().await;
        let thresholds = &self.collection_config.alert_thresholds;
        
        // Check latency threshold
        if metrics.average_latency.as_millis() > thresholds.max_latency_ms as u128 {
            info!("ALERT: High latency detected: {:?}", metrics.average_latency);
        }
        
        // Check throughput threshold
        if metrics.transaction_throughput < thresholds.min_throughput_tps {
            info!("ALERT: Low throughput detected: {:.2} TPS", metrics.transaction_throughput);
        }
        
        // Check memory threshold
        if metrics.memory_usage_mb > thresholds.max_memory_usage_mb as f64 {
            info!("ALERT: High memory usage detected: {:.2} MB", metrics.memory_usage_mb);
        }
        
        // Check CPU threshold
        if metrics.cpu_usage_percent > thresholds.max_cpu_usage_percent {
            info!("ALERT: High CPU usage detected: {:.2}%", metrics.cpu_usage_percent);
        }
        
        // Check cache hit rate threshold
        if metrics.cache_hit_rate < thresholds.min_cache_hit_rate {
            info!("ALERT: Low cache hit rate detected: {:.2}", metrics.cache_hit_rate);
        }
        
        Ok(())
    }
}

impl Clone for MetricsCollector {
    fn clone(&self) -> Self {
        Self {
            metrics: Arc::clone(&self.metrics),
            historical_data: Arc::clone(&self.historical_data),
            real_time_stats: Arc::clone(&self.real_time_stats),
            collection_config: self.collection_config.clone(),
        }
    }
}

impl HistoricalMetrics {
    fn new() -> Self {
        Self {
            hourly_snapshots: VecDeque::new(),
            daily_summaries: VecDeque::new(),
            trend_analysis: TrendAnalysis::default(),
        }
    }
}

/// Optimization recommendation
#[derive(Debug, Clone)]
pub struct OptimizationRecommendation {
    pub category: String,
    pub priority: OptimizationPriority,
    pub description: String,
    pub actions: Vec<String>,
    pub expected_improvement: String,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum OptimizationPriority {
    Low,
    Medium,
    High,
    Critical,
}