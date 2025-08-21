use anyhow::Result;
use serde::{Deserialize, Serialize};

/// Detects and reports hardware capabilities
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HardwareDetector {
    pub cpu_info: CpuInfo,
    pub memory_info: MemoryInfo,
    pub gpu_info: Option<GpuInfo>,
    pub storage_info: StorageInfo,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CpuInfo {
    pub model: String,
    pub cores: u32,
    pub threads: u32,
    pub base_frequency: f64, // GHz
    pub cache_size: u64,     // MB
    pub architecture: String,
    pub features: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryInfo {
    pub total_ram: u64,      // GB
    pub available_ram: u64,  // GB
    pub memory_type: String, // DDR4, DDR5, etc.
    pub memory_speed: u32,   // MHz
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GpuInfo {
    pub model: String,
    pub vendor: String,      // NVIDIA, AMD, Intel
    pub memory: u64,         // GB
    pub compute_units: u32,
    pub base_clock: u32,     // MHz
    pub memory_clock: u32,   // MHz
    pub driver_version: String,
    pub cuda_cores: Option<u32>,
    pub opencl_support: bool,
    pub cuda_support: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageInfo {
    pub total_space: u64,    // GB
    pub available_space: u64, // GB
    pub storage_type: String, // SSD, HDD
    pub read_speed: u32,     // MB/s
    pub write_speed: u32,    // MB/s
}

impl HardwareDetector {
    pub async fn new() -> Result<Self> {
        let cpu_info = Self::detect_cpu().await?;
        let memory_info = Self::detect_memory().await?;
        let gpu_info = Self::detect_gpu().await;
        let storage_info = Self::detect_storage().await?;

        Ok(HardwareDetector {
            cpu_info,
            memory_info,
            gpu_info,
            storage_info,
        })
    }

    async fn detect_cpu() -> Result<CpuInfo> {
        // Simplified CPU detection - in reality, you'd use platform-specific APIs
        #[cfg(target_os = "windows")]
        {
            Self::detect_cpu_windows().await
        }
        
        #[cfg(target_os = "linux")]
        {
            Self::detect_cpu_linux().await
        }
        
        #[cfg(target_os = "macos")]
        {
            Self::detect_cpu_macos().await
        }
        
        #[cfg(not(any(target_os = "windows", target_os = "linux", target_os = "macos")))]
        {
            Ok(Self::default_cpu_info())
        }
    }

    #[cfg(target_os = "windows")]
    async fn detect_cpu_windows() -> Result<CpuInfo> {
        // Would use WMI queries or Windows APIs
        Ok(CpuInfo {
            model: "Detected Windows CPU".to_string(),
            cores: num_cpus::get_physical() as u32,
            threads: num_cpus::get() as u32,
            base_frequency: 3.2,
            cache_size: 16,
            architecture: std::env::consts::ARCH.to_string(),
            features: vec!["SSE4.2".to_string(), "AVX2".to_string()],
        })
    }

    #[cfg(target_os = "linux")]
    async fn detect_cpu_linux() -> Result<CpuInfo> {
        // Would read /proc/cpuinfo
        Ok(CpuInfo {
            model: "Detected Linux CPU".to_string(),
            cores: num_cpus::get_physical() as u32,
            threads: num_cpus::get() as u32,
            base_frequency: 3.2,
            cache_size: 16,
            architecture: std::env::consts::ARCH.to_string(),
            features: vec!["SSE4.2".to_string(), "AVX2".to_string()],
        })
    }

    #[cfg(target_os = "macos")]
    async fn detect_cpu_macos() -> Result<CpuInfo> {
        // Would use system_profiler or sysctl
        Ok(CpuInfo {
            model: "Detected macOS CPU".to_string(),
            cores: num_cpus::get_physical() as u32,
            threads: num_cpus::get() as u32,
            base_frequency: 3.2,
            cache_size: 16,
            architecture: std::env::consts::ARCH.to_string(),
            features: vec!["SSE4.2".to_string(), "AVX2".to_string()],
        })
    }

    fn default_cpu_info() -> CpuInfo {
        CpuInfo {
            model: "Unknown CPU".to_string(),
            cores: num_cpus::get_physical() as u32,
            threads: num_cpus::get() as u32,
            base_frequency: 2.4,
            cache_size: 8,
            architecture: std::env::consts::ARCH.to_string(),
            features: vec![],
        }
    }

    async fn detect_memory() -> Result<MemoryInfo> {
        // Simplified memory detection
        Ok(MemoryInfo {
            total_ram: 16, // Assume 16GB for demo
            available_ram: 12,
            memory_type: "DDR4".to_string(),
            memory_speed: 3200,
        })
    }

    async fn detect_gpu() -> Option<GpuInfo> {
        // Simplified GPU detection
        // In reality, you'd use platform-specific APIs or libraries like wgpu
        Some(GpuInfo {
            model: "Detected GPU".to_string(),
            vendor: "Unknown".to_string(),
            memory: 8,
            compute_units: 2048,
            base_clock: 1500,
            memory_clock: 7000,
            driver_version: "1.0.0".to_string(),
            cuda_cores: Some(2048),
            opencl_support: true,
            cuda_support: true,
        })
    }

    async fn detect_storage() -> Result<StorageInfo> {
        // Simplified storage detection
        Ok(StorageInfo {
            total_space: 1000, // 1TB
            available_space: 500, // 500GB available
            storage_type: "SSD".to_string(),
            read_speed: 3500, // MB/s
            write_speed: 2000, // MB/s
        })
    }

    /// Log hardware capabilities
    pub fn log_capabilities(&self) {
        tracing::info!("=== Hardware Capabilities ===");
        tracing::info!("CPU: {} ({} cores, {} threads)", 
                      self.cpu_info.model, self.cpu_info.cores, self.cpu_info.threads);
        tracing::info!("Architecture: {}", self.cpu_info.architecture);
        tracing::info!("CPU Features: {:?}", self.cpu_info.features);
        
        tracing::info!("Memory: {} GB total, {} GB available ({})", 
                      self.memory_info.total_ram, self.memory_info.available_ram, self.memory_info.memory_type);
        
        if let Some(gpu) = &self.gpu_info {
            tracing::info!("GPU: {} {} ({} GB VRAM)", gpu.vendor, gpu.model, gpu.memory);
            tracing::info!("CUDA Support: {}, OpenCL Support: {}", gpu.cuda_support, gpu.opencl_support);
        } else {
            tracing::info!("GPU: None detected");
        }
        
        tracing::info!("Storage: {} GB total, {} GB available ({})", 
                      self.storage_info.total_space, self.storage_info.available_space, self.storage_info.storage_type);
        tracing::info!("Storage Speed: {} MB/s read, {} MB/s write", 
                      self.storage_info.read_speed, self.storage_info.write_speed);
        tracing::info!("=============================");
    }

    /// Check if hardware is suitable for ML tasks
    pub fn is_ml_capable(&self) -> bool {
        // Basic requirements for ML tasks
        self.cpu_info.cores >= 2 && 
        self.memory_info.total_ram >= 4 && 
        self.storage_info.available_space >= 10
    }

    /// Get ML performance estimate (0-100 score)
    pub fn get_ml_performance_score(&self) -> u32 {
        let mut score = 0u32;

        // CPU score (0-30 points)
        score += (self.cpu_info.cores * 3).min(30);
        
        // Memory score (0-25 points)
        score += ((self.memory_info.total_ram / 2) as u32).min(25);
        
        // GPU score (0-35 points)
        if let Some(gpu) = &self.gpu_info {
            if gpu.cuda_support {
                score += 35;
            } else if gpu.opencl_support {
                score += 20;
            } else {
                score += 10;
            }
        }
        
        // Storage score (0-10 points)
        if self.storage_info.storage_type == "SSD" {
            score += 10;
        } else {
            score += 5;
        }

        score.min(100)
    }

    /// Get recommended task types based on hardware
    pub fn get_recommended_task_types(&self) -> Vec<String> {
        let mut recommendations = Vec::new();

        // Basic tasks that all systems can handle
        recommendations.push("Oracle".to_string());
        recommendations.push("NetworkOptimization".to_string());

        // CPU-intensive tasks
        if self.cpu_info.cores >= 4 {
            recommendations.push("NaturalLanguageProcessing".to_string());
            recommendations.push("TimeSeriesAnalysis".to_string());
        }

        // Memory-intensive tasks
        if self.memory_info.total_ram >= 8 {
            recommendations.push("DistributedTraining".to_string());
        }

        // GPU-accelerated tasks
        if let Some(gpu) = &self.gpu_info {
            if gpu.cuda_support && gpu.memory >= 4 {
                recommendations.push("ImageClassification".to_string());
                recommendations.push("ReinforcementLearning".to_string());
                recommendations.push("AutoML".to_string());
            }
        }

        // High-end tasks
        if self.cpu_info.cores >= 8 && self.memory_info.total_ram >= 16 {
            recommendations.push("SmartContractOptimization".to_string());
        }

        recommendations
    }

    /// Get optimal task concurrency
    pub fn get_optimal_concurrency(&self) -> usize {
        let base_concurrency = self.cpu_info.threads as usize / 2;
        
        // Adjust based on memory
        let memory_limit = (self.memory_info.available_ram / 2) as usize;
        
        // Adjust based on GPU
        let gpu_boost = if self.gpu_info.is_some() { 2 } else { 0 };
        
        (base_concurrency + gpu_boost).min(memory_limit).max(1)
    }

    /// Export hardware report
    pub fn export_report(&self) -> Result<String> {
        Ok(serde_json::to_string_pretty(self)?)
    }
}
