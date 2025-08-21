use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::time::{Duration, Instant};
use tokio::time::interval;

/// Monitors system performance and resource usage
#[derive(Debug)]
pub struct PerformanceMonitor {
    start_time: Instant,
    current_stats: PerformanceStats,
    history: Vec<PerformanceSnapshot>,
    monitoring_active: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceStats {
    pub cpu_usage: f64,
    pub memory_usage: f64,
    pub gpu_usage: f64,
    pub gpu_memory_usage: f64,
    pub disk_usage: f64,
    pub network_usage: f64,
    pub temperature: f64,
    pub uptime_seconds: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceSnapshot {
    pub timestamp: u64,
    pub stats: PerformanceStats,
}

impl Default for PerformanceStats {
    fn default() -> Self {
        PerformanceStats {
            cpu_usage: 0.0,
            memory_usage: 0.0,
            gpu_usage: 0.0,
            gpu_memory_usage: 0.0,
            disk_usage: 0.0,
            network_usage: 0.0,
            temperature: 0.0,
            uptime_seconds: 0,
        }
    }
}

impl PerformanceMonitor {
    pub fn new() -> Self {
        PerformanceMonitor {
            start_time: Instant::now(),
            current_stats: PerformanceStats::default(),
            history: Vec::new(),
            monitoring_active: false,
        }
    }

    /// Start performance monitoring
    pub async fn start(&mut self) -> Result<()> {
        if self.monitoring_active {
            return Ok(());
        }

        self.monitoring_active = true;
        tracing::info!("Starting performance monitoring");

        // Spawn monitoring task
        let monitor = self.clone_for_monitoring();
        tokio::spawn(async move {
            let mut interval = interval(Duration::from_secs(5));
            
            loop {
                interval.tick().await;
                if let Err(e) = monitor.update_stats().await {
                    tracing::error!("Failed to update performance stats: {}", e);
                }
            }
        });

        Ok(())
    }

    fn clone_for_monitoring(&self) -> PerformanceMonitor {
        PerformanceMonitor {
            start_time: self.start_time,
            current_stats: self.current_stats.clone(),
            history: Vec::new(), // Don't clone history to save memory
            monitoring_active: true,
        }
    }

    /// Update performance statistics
    async fn update_stats(&self) -> Result<()> {
        // Update CPU usage
        let cpu_usage = self.get_cpu_usage().await?;
        
        // Update memory usage
        let memory_usage = self.get_memory_usage().await?;
        
        // Update GPU usage (if available)
        let (gpu_usage, gpu_memory_usage) = self.get_gpu_usage().await.unwrap_or((0.0, 0.0));
        
        // Update disk usage
        let disk_usage = self.get_disk_usage().await?;
        
        // Update network usage
        let network_usage = self.get_network_usage().await?;
        
        // Update temperature (if available)
        let temperature = self.get_temperature().await.unwrap_or(0.0);
        
        let uptime_seconds = self.start_time.elapsed().as_secs();

        // This would normally update a shared state, but for now we'll just log
        tracing::debug!("Performance stats - CPU: {:.1}%, Memory: {:.1}%, GPU: {:.1}%, Uptime: {}s",
                       cpu_usage, memory_usage, gpu_usage, uptime_seconds);

        Ok(())
    }

    /// Get current CPU usage percentage
    async fn get_cpu_usage(&self) -> Result<f64> {
        // Simplified CPU usage calculation
        // In a real implementation, you'd use system APIs or libraries like `sysinfo`
        
        #[cfg(target_os = "windows")]
        {
            // Windows implementation would use WMI or performance counters
            Ok(self.simulate_cpu_usage())
        }
        
        #[cfg(target_os = "linux")]
        {
            // Linux implementation would read /proc/stat
            Ok(self.simulate_cpu_usage())
        }
        
        #[cfg(target_os = "macos")]
        {
            // macOS implementation would use host_processor_info
            Ok(self.simulate_cpu_usage())
        }
        
        #[cfg(not(any(target_os = "windows", target_os = "linux", target_os = "macos")))]
        {
            Ok(0.0)
        }
    }

    fn simulate_cpu_usage(&self) -> f64 {
        // Simulate CPU usage for demo purposes
        use std::time::{SystemTime, UNIX_EPOCH};
        let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();
        ((now % 100) as f64) * 0.8 // Simulate 0-80% usage
    }

    /// Get current memory usage percentage
    async fn get_memory_usage(&self) -> Result<f64> {
        // Simplified memory usage calculation
        #[cfg(target_os = "windows")]
        {
            Ok(self.simulate_memory_usage())
        }
        
        #[cfg(target_os = "linux")]
        {
            // Read /proc/meminfo
            Ok(self.simulate_memory_usage())
        }
        
        #[cfg(target_os = "macos")]
        {
            Ok(self.simulate_memory_usage())
        }
        
        #[cfg(not(any(target_os = "windows", target_os = "linux", target_os = "macos")))]
        {
            Ok(0.0)
        }
    }

    fn simulate_memory_usage(&self) -> f64 {
        // Simulate memory usage for demo purposes
        use std::time::{SystemTime, UNIX_EPOCH};
        let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();
        ((now % 80) as f64) + 20.0 // Simulate 20-100% usage
    }

    /// Get GPU usage and memory usage
    async fn get_gpu_usage(&self) -> Result<(f64, f64)> {
        // In a real implementation, you'd use NVIDIA ML or similar APIs
        Ok(self.simulate_gpu_usage())
    }

    fn simulate_gpu_usage(&self) -> (f64, f64) {
        // Simulate GPU usage for demo purposes
        use std::time::{SystemTime, UNIX_EPOCH};
        let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();
        let gpu_usage = ((now % 90) as f64) * 0.7; // 0-63% usage
        let gpu_memory = ((now % 70) as f64) + 30.0; // 30-100% memory
        (gpu_usage, gpu_memory)
    }

    /// Get disk usage percentage
    async fn get_disk_usage(&self) -> Result<f64> {
        // Simplified disk usage
        Ok(45.0) // Simulate 45% disk usage
    }

    /// Get network usage in MB/s
    async fn get_network_usage(&self) -> Result<f64> {
        // Simplified network usage
        use std::time::{SystemTime, UNIX_EPOCH};
        let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();
        Ok(((now % 50) as f64) * 0.1) // 0-5 MB/s
    }

    /// Get system temperature in Celsius
    async fn get_temperature(&self) -> Result<f64> {
        // In a real implementation, you'd read hardware sensors
        use std::time::{SystemTime, UNIX_EPOCH};
        let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();
        Ok(((now % 30) as f64) + 40.0) // 40-70°C
    }

    /// Get current performance statistics
    pub async fn get_current_stats(&mut self) -> PerformanceStats {
        // Update stats if monitoring is active
        if self.monitoring_active {
            self.current_stats.cpu_usage = self.get_cpu_usage().await.unwrap_or(0.0);
            self.current_stats.memory_usage = self.get_memory_usage().await.unwrap_or(0.0);
            let (gpu_usage, gpu_memory) = self.get_gpu_usage().await.unwrap_or((0.0, 0.0));
            self.current_stats.gpu_usage = gpu_usage;
            self.current_stats.gpu_memory_usage = gpu_memory;
            self.current_stats.disk_usage = self.get_disk_usage().await.unwrap_or(0.0);
            self.current_stats.network_usage = self.get_network_usage().await.unwrap_or(0.0);
            self.current_stats.temperature = self.get_temperature().await.unwrap_or(0.0);
            self.current_stats.uptime_seconds = self.start_time.elapsed().as_secs();
        }

        self.current_stats.clone()
    }

    /// Get performance history
    pub fn get_history(&self) -> &[PerformanceSnapshot] {
        &self.history
    }

    /// Add current stats to history
    pub async fn snapshot(&mut self) -> Result<()> {
        let stats = self.get_current_stats().await;
        let snapshot = PerformanceSnapshot {
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)?
                .as_secs(),
            stats,
        };

        self.history.push(snapshot);

        // Keep only last 100 snapshots
        if self.history.len() > 100 {
            self.history.remove(0);
        }

        Ok(())
    }

    /// Check if system is under high load
    pub async fn is_high_load(&mut self) -> bool {
        let stats = self.get_current_stats().await;
        stats.cpu_usage > 80.0 || stats.memory_usage > 90.0 || stats.temperature > 80.0
    }

    /// Get system health score (0-100)
    pub async fn get_health_score(&mut self) -> f64 {
        let stats = self.get_current_stats().await;
        
        let cpu_score = (100.0 - stats.cpu_usage).max(0.0);
        let memory_score = (100.0 - stats.memory_usage).max(0.0);
        let temp_score = if stats.temperature > 0.0 {
            (100.0 - (stats.temperature - 20.0) * 2.0).max(0.0).min(100.0)
        } else {
            100.0
        };

        (cpu_score + memory_score + temp_score) / 3.0
    }

    /// Shutdown monitoring
    pub async fn shutdown(&mut self) -> Result<()> {
        self.monitoring_active = false;
        tracing::info!("Performance monitoring stopped");
        Ok(())
    }

    /// Export performance data
    pub fn export_data(&self) -> Result<String> {
        Ok(serde_json::to_string_pretty(&self.history)?)
    }
}
