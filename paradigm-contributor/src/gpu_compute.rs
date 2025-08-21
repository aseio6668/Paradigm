use std::sync::Arc;
use anyhow::{Result, anyhow};
use serde::{Deserialize, Serialize};
use tracing::{info, warn, error, debug};
use tokio::sync::RwLock;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum GpuBackend {
    Wgpu,
    #[cfg(feature = "cuda")]
    Cuda,
    #[cfg(feature = "opencl")]
    OpenCl,
    #[cfg(feature = "metal")]
    Metal,
    Cpu,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GpuInfo {
    pub backend: GpuBackend,
    pub device_name: String,
    pub memory_mb: u64,
    pub compute_units: u32,
    pub max_threads: u32,
    pub supports_fp16: bool,
    pub supports_bf16: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComputeCapabilities {
    pub available_gpus: Vec<GpuInfo>,
    pub recommended_backend: GpuBackend,
    pub total_memory_mb: u64,
    pub estimated_performance_score: f32,
}

pub struct GpuComputeEngine {
    capabilities: ComputeCapabilities,
    #[cfg(feature = "cuda")]
    cuda_context: Option<Arc<cudarc::driver::CudaContext>>,
    #[cfg(feature = "opencl")]
    opencl_context: Option<ocl::ProQue>,
    wgpu_device: Option<Arc<wgpu::Device>>,
    wgpu_queue: Option<Arc<wgpu::Queue>>,
}

impl GpuComputeEngine {
    pub async fn new() -> Result<Self> {
        info!("Initializing GPU compute engine...");
        
        let capabilities = Self::detect_gpu_capabilities().await?;
        info!("Detected GPU capabilities: {:?}", capabilities);
        
        let mut engine = Self {
            capabilities,
            #[cfg(feature = "cuda")]
            cuda_context: None,
            #[cfg(feature = "opencl")]
            opencl_context: None,
            wgpu_device: None,
            wgpu_queue: None,
        };
        
        engine.initialize_backend().await?;
        Ok(engine)
    }
    
    async fn detect_gpu_capabilities() -> Result<ComputeCapabilities> {
        let mut available_gpus = Vec::new();
        let mut total_memory_mb = 0;
        let mut performance_score = 0.0;
        
        // Detect WGPU devices (cross-platform)
        if let Ok((device_infos, _)) = Self::detect_wgpu_devices().await {
            for info in device_infos {
                total_memory_mb += info.memory_mb;
                performance_score += Self::calculate_gpu_score(&info);
                available_gpus.push(info);
            }
        }
        
        #[cfg(feature = "cuda")]
        {
            if let Ok(cuda_infos) = Self::detect_cuda_devices().await {
                for info in cuda_infos {
                    total_memory_mb += info.memory_mb;
                    performance_score += Self::calculate_gpu_score(&info);
                    available_gpus.push(info);
                }
            }
        }
        
        #[cfg(feature = "opencl")]
        {
            if let Ok(opencl_infos) = Self::detect_opencl_devices().await {
                for info in opencl_infos {
                    total_memory_mb += info.memory_mb;
                    performance_score += Self::calculate_gpu_score(&info);
                    available_gpus.push(info);
                }
            }
        }
        
        let recommended_backend = Self::choose_best_backend(&available_gpus);
        
        Ok(ComputeCapabilities {
            available_gpus,
            recommended_backend,
            total_memory_mb,
            estimated_performance_score: performance_score,
        })
    }
    
    async fn detect_wgpu_devices() -> Result<(Vec<GpuInfo>, wgpu::Instance)> {
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends: wgpu::Backends::all(),
            ..Default::default()
        });
        
        let adapters = instance.enumerate_adapters(wgpu::Backends::all());
        let mut gpu_infos = Vec::new();
        
        for adapter in adapters {
            let info = adapter.get_info();
            let limits = adapter.limits();
            
            // Estimate memory (WGPU doesn't expose this directly)
            let estimated_memory_mb = match info.device_type {
                wgpu::DeviceType::DiscreteGpu => 8192, // Assume 8GB for discrete GPUs
                wgpu::DeviceType::IntegratedGpu => 2048, // 2GB for integrated
                wgpu::DeviceType::VirtualGpu => 1024,
                _ => 512,
            };
            
            let gpu_info = GpuInfo {
                backend: GpuBackend::Wgpu,
                device_name: info.name.clone(),
                memory_mb: estimated_memory_mb,
                compute_units: 32, // Default estimate
                max_threads: limits.max_compute_workgroup_size_x,
                supports_fp16: true, // Most modern GPUs support this
                supports_bf16: false, // Less common
            };
            
            gpu_infos.push(gpu_info);
        }
        
        Ok((gpu_infos, instance))
    }
    
    #[cfg(feature = "cuda")]
    async fn detect_cuda_devices() -> Result<Vec<GpuInfo>> {
        use cudarc::driver::CudaDevice;
        
        let mut gpu_infos = Vec::new();
        
        match CudaDevice::new(0) {
            Ok(device) => {
                let name = device.name()?;
                let memory_info = device.memory_info()?;
                let attributes = device.get_device_attributes()?;
                
                let gpu_info = GpuInfo {
                    backend: GpuBackend::Cuda,
                    device_name: name,
                    memory_mb: memory_info.total / (1024 * 1024),
                    compute_units: attributes.multiprocessor_count as u32,
                    max_threads: attributes.max_threads_per_multiprocessor as u32,
                    supports_fp16: true,
                    supports_bf16: true, // Modern CUDA devices support BF16
                };
                
                gpu_infos.push(gpu_info);
            }
            Err(e) => {
                warn!("CUDA device detection failed: {}", e);
            }
        }
        
        Ok(gpu_infos)
    }
    
    #[cfg(feature = "opencl")]
    async fn detect_opencl_devices() -> Result<Vec<GpuInfo>> {
        use ocl::{Platform, Device};
        
        let mut gpu_infos = Vec::new();
        
        if let Ok(platforms) = Platform::list() {
            for platform in platforms {
                if let Ok(devices) = Device::list(platform, Some(ocl::flags::DEVICE_TYPE_GPU)) {
                    for device in devices {
                        let name = device.name().unwrap_or_else(|_| "Unknown OpenCL Device".to_string());
                        let memory = device.global_mem_size().unwrap_or(0) / (1024 * 1024);
                        let compute_units = device.max_compute_units().unwrap_or(1);
                        
                        let gpu_info = GpuInfo {
                            backend: GpuBackend::OpenCl,
                            device_name: name,
                            memory_mb: memory,
                            compute_units,
                            max_threads: 256, // Conservative estimate
                            supports_fp16: false, // Depends on extensions
                            supports_bf16: false,
                        };
                        
                        gpu_infos.push(gpu_info);
                    }
                }
            }
        }
        
        Ok(gpu_infos)
    }
    
    fn calculate_gpu_score(info: &GpuInfo) -> f32 {
        let memory_score = (info.memory_mb as f32) / 1024.0; // GB
        let compute_score = info.compute_units as f32;
        let backend_multiplier = match info.backend {
            GpuBackend::Cuda => 1.5,    // CUDA is often fastest
            GpuBackend::Metal => 1.3,   // Metal is very efficient on Apple
            GpuBackend::Wgpu => 1.0,    // Good cross-platform baseline
            GpuBackend::OpenCl => 0.8,  // Often slower than native APIs
            GpuBackend::Cpu => 0.1,     // Much slower than GPU
        };
        
        (memory_score + compute_score) * backend_multiplier
    }
    
    fn choose_best_backend(gpus: &[GpuInfo]) -> GpuBackend {
        if gpus.is_empty() {
            return GpuBackend::Cpu;
        }
        
        // Find GPU with highest performance score
        gpus.iter()
            .max_by(|a, b| {
                Self::calculate_gpu_score(a)
                    .partial_cmp(&Self::calculate_gpu_score(b))
                    .unwrap_or(std::cmp::Ordering::Equal)
            })
            .map(|gpu| gpu.backend.clone())
            .unwrap_or(GpuBackend::Cpu)
    }
    
    async fn initialize_backend(&mut self) -> Result<()> {
        match self.capabilities.recommended_backend {
            GpuBackend::Wgpu => {
                self.initialize_wgpu().await?;
                info!("Initialized WGPU backend");
            }
            #[cfg(feature = "cuda")]
            GpuBackend::Cuda => {
                self.initialize_cuda().await?;
                info!("Initialized CUDA backend");
            }
            #[cfg(feature = "opencl")]
            GpuBackend::OpenCl => {
                self.initialize_opencl().await?;
                info!("Initialized OpenCL backend");
            }
            GpuBackend::Cpu => {
                info!("Using CPU backend (no suitable GPU found)");
            }
            _ => {
                warn!("Requested backend not available, falling back to CPU");
            }
        }
        
        Ok(())
    }
    
    async fn initialize_wgpu(&mut self) -> Result<()> {
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends: wgpu::Backends::all(),
            ..Default::default()
        });
        
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::HighPerformance,
                compatible_surface: None,
                force_fallback_adapter: false,
            })
            .await
            .ok_or_else(|| anyhow!("Failed to find suitable GPU adapter"))?;
        
        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    label: Some("Paradigm Compute Device"),
                    features: wgpu::Features::empty(),
                    limits: wgpu::Limits::default(),
                },
                None,
            )
            .await?;
        
        self.wgpu_device = Some(Arc::new(device));
        self.wgpu_queue = Some(Arc::new(queue));
        
        Ok(())
    }
    
    #[cfg(feature = "cuda")]
    async fn initialize_cuda(&mut self) -> Result<()> {
        use cudarc::driver::CudaDevice;
        
        let device = CudaDevice::new(0)?;
        self.cuda_context = Some(Arc::new(device));
        
        Ok(())
    }
    
    #[cfg(feature = "opencl")]
    async fn initialize_opencl(&mut self) -> Result<()> {
        let pro_que = ocl::ProQue::builder()
            .src("")
            .dims(1024)
            .build()?;
        
        self.opencl_context = Some(pro_que);
        
        Ok(())
    }
    
    pub fn get_capabilities(&self) -> &ComputeCapabilities {
        &self.capabilities
    }
    
    pub async fn run_ml_task(&self, task_data: &[u8]) -> Result<Vec<u8>> {
        debug!("Running ML task with {} bytes of data", task_data.len());
        
        match self.capabilities.recommended_backend {
            GpuBackend::Wgpu => self.run_wgpu_task(task_data).await,
            #[cfg(feature = "cuda")]
            GpuBackend::Cuda => self.run_cuda_task(task_data).await,
            #[cfg(feature = "opencl")]
            GpuBackend::OpenCl => self.run_opencl_task(task_data).await,
            _ => self.run_cpu_task(task_data).await,
        }
    }
    
    async fn run_wgpu_task(&self, task_data: &[u8]) -> Result<Vec<u8>> {
        // Placeholder for WGPU compute shader execution
        // In a real implementation, this would:
        // 1. Create compute buffers
        // 2. Load and execute compute shaders
        // 3. Process ML workloads (matrix operations, neural networks, etc.)
        
        info!("Processing ML task on WGPU backend");
        
        // Simulate computation time based on data size
        let computation_time = std::cmp::max(100, task_data.len() / 1000);
        tokio::time::sleep(tokio::time::Duration::from_millis(computation_time as u64)).await;
        
        // Return processed data (placeholder)
        let mut result = task_data.to_vec();
        result.reverse(); // Simple transformation for demonstration
        Ok(result)
    }
    
    #[cfg(feature = "cuda")]
    async fn run_cuda_task(&self, task_data: &[u8]) -> Result<Vec<u8>> {
        info!("Processing ML task on CUDA backend");
        
        if let Some(context) = &self.cuda_context {
            // Placeholder for CUDA kernel execution
            // Real implementation would:
            // 1. Allocate GPU memory
            // 2. Copy data to GPU
            // 3. Launch CUDA kernels for ML computations
            // 4. Copy results back to CPU
            
            let computation_time = std::cmp::max(50, task_data.len() / 2000); // CUDA is faster
            tokio::time::sleep(tokio::time::Duration::from_millis(computation_time as u64)).await;
            
            let mut result = task_data.to_vec();
            result.reverse();
            Ok(result)
        } else {
            Err(anyhow!("CUDA context not initialized"))
        }
    }
    
    #[cfg(feature = "opencl")]
    async fn run_opencl_task(&self, task_data: &[u8]) -> Result<Vec<u8>> {
        info!("Processing ML task on OpenCL backend");
        
        if let Some(context) = &self.opencl_context {
            // Placeholder for OpenCL kernel execution
            let computation_time = std::cmp::max(75, task_data.len() / 1500);
            tokio::time::sleep(tokio::time::Duration::from_millis(computation_time as u64)).await;
            
            let mut result = task_data.to_vec();
            result.reverse();
            Ok(result)
        } else {
            Err(anyhow!("OpenCL context not initialized"))
        }
    }
    
    async fn run_cpu_task(&self, task_data: &[u8]) -> Result<Vec<u8>> {
        info!("Processing ML task on CPU backend");
        
        // CPU is slower, so longer computation time
        let computation_time = std::cmp::max(200, task_data.len() / 500);
        tokio::time::sleep(tokio::time::Duration::from_millis(computation_time as u64)).await;
        
        let mut result = task_data.to_vec();
        result.reverse();
        Ok(result)
    }
    
    pub async fn benchmark_performance(&self) -> Result<f32> {
        info!("Running performance benchmark...");
        
        let test_data = vec![0u8; 1024 * 1024]; // 1MB test data
        let start_time = std::time::Instant::now();
        
        // Run multiple iterations
        for _ in 0..10 {
            self.run_ml_task(&test_data).await?;
        }
        
        let elapsed = start_time.elapsed();
        let throughput_mbps = (10.0 * 1024.0 * 1024.0) / elapsed.as_secs_f32(); // MB/s
        
        info!("Benchmark completed: {:.2} MB/s throughput", throughput_mbps);
        Ok(throughput_mbps)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_gpu_detection() {
        let capabilities = GpuComputeEngine::detect_gpu_capabilities().await;
        assert!(capabilities.is_ok());
        
        let caps = capabilities.unwrap();
        println!("Detected GPUs: {:?}", caps);
    }
    
    #[tokio::test]
    async fn test_compute_engine_creation() {
        let engine = GpuComputeEngine::new().await;
        assert!(engine.is_ok());
    }
}
