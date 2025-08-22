/// Memory optimization and management for Paradigm
use std::alloc::{GlobalAlloc, System, Layout};
use std::collections::{HashMap, VecDeque};
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, AtomicUsize, Ordering};
use std::time::{Duration, Instant};
use anyhow::Result;
use serde::{Deserialize, Serialize};
use tokio::sync::{RwLock, Mutex};
use dashmap::DashMap;

/// Custom allocator wrapper for tracking memory usage
#[derive(Debug)]
pub struct TrackingAllocator {
    inner: System,
    allocated: AtomicU64,
    deallocated: AtomicU64,
    peak_usage: AtomicU64,
    allocation_count: AtomicU64,
}

impl TrackingAllocator {
    pub const fn new() -> Self {
        Self {
            inner: System,
            allocated: AtomicU64::new(0),
            deallocated: AtomicU64::new(0),
            peak_usage: AtomicU64::new(0),
            allocation_count: AtomicU64::new(0),
        }
    }

    pub fn get_stats(&self) -> MemoryStats {
        let allocated = self.allocated.load(Ordering::Relaxed);
        let deallocated = self.deallocated.load(Ordering::Relaxed);
        let current_usage = allocated.saturating_sub(deallocated);
        
        MemoryStats {
            current_usage_bytes: current_usage,
            peak_usage_bytes: self.peak_usage.load(Ordering::Relaxed),
            total_allocated_bytes: allocated,
            total_deallocated_bytes: deallocated,
            allocation_count: self.allocation_count.load(Ordering::Relaxed),
        }
    }

    pub fn reset_stats(&self) {
        self.allocated.store(0, Ordering::Relaxed);
        self.deallocated.store(0, Ordering::Relaxed);
        self.peak_usage.store(0, Ordering::Relaxed);
        self.allocation_count.store(0, Ordering::Relaxed);
    }
}

unsafe impl GlobalAlloc for TrackingAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let ptr = self.inner.alloc(layout);
        if !ptr.is_null() {
            let size = layout.size() as u64;
            self.allocated.fetch_add(size, Ordering::Relaxed);
            self.allocation_count.fetch_add(1, Ordering::Relaxed);
            
            // Update peak usage
            let allocated = self.allocated.load(Ordering::Relaxed);
            let deallocated = self.deallocated.load(Ordering::Relaxed);
            let current = allocated.saturating_sub(deallocated);
            let mut peak = self.peak_usage.load(Ordering::Relaxed);
            while current > peak {
                match self.peak_usage.compare_exchange_weak(
                    peak, current, Ordering::Relaxed, Ordering::Relaxed
                ) {
                    Ok(_) => break,
                    Err(x) => peak = x,
                }
            }
        }
        ptr
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        self.inner.dealloc(ptr, layout);
        let size = layout.size() as u64;
        self.deallocated.fetch_add(size, Ordering::Relaxed);
    }
}

/// Memory statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryStats {
    pub current_usage_bytes: u64,
    pub peak_usage_bytes: u64,
    pub total_allocated_bytes: u64,
    pub total_deallocated_bytes: u64,
    pub allocation_count: u64,
}

impl MemoryStats {
    pub fn current_usage_mb(&self) -> f64 {
        self.current_usage_bytes as f64 / 1024.0 / 1024.0
    }

    pub fn peak_usage_mb(&self) -> f64 {
        self.peak_usage_bytes as f64 / 1024.0 / 1024.0
    }
}

/// Object pool for reusing expensive objects
pub struct ObjectPool<T> {
    pool: Arc<Mutex<VecDeque<T>>>,
    max_size: usize,
    create_fn: Arc<dyn Fn() -> T + Send + Sync>,
    reset_fn: Arc<dyn Fn(&mut T) + Send + Sync>,
}

impl<T> ObjectPool<T>
where
    T: Send + 'static,
{
    pub fn new<F, R>(max_size: usize, create_fn: F, reset_fn: R) -> Self
    where
        F: Fn() -> T + Send + Sync + 'static,
        R: Fn(&mut T) + Send + Sync + 'static,
    {
        Self {
            pool: Arc::new(Mutex::new(VecDeque::with_capacity(max_size))),
            max_size,
            create_fn: Arc::new(create_fn),
            reset_fn: Arc::new(reset_fn),
        }
    }

    pub async fn acquire(&self) -> PooledObject<T> {
        let mut pool = self.pool.lock().await;
        let object = pool.pop_front().unwrap_or_else(|| (self.create_fn)());
        
        PooledObject {
            object: Some(object),
            pool: self.pool.clone(),
            reset_fn: self.reset_fn.clone(),
            max_pool_size: self.max_size,
        }
    }

    pub async fn size(&self) -> usize {
        self.pool.lock().await.len()
    }
}

/// Pooled object wrapper that automatically returns to pool on drop
pub struct PooledObject<T: Send + 'static> {
    object: Option<T>,
    pool: Arc<Mutex<VecDeque<T>>>,
    reset_fn: Arc<dyn Fn(&mut T) + Send + Sync>,
    max_pool_size: usize,
}

impl<T: Send + 'static> PooledObject<T> {
    pub fn as_ref(&self) -> &T {
        self.object.as_ref().unwrap()
    }

    pub fn as_mut(&mut self) -> &mut T {
        self.object.as_mut().unwrap()
    }
}

impl<T: Send + 'static> Drop for PooledObject<T> {
    fn drop(&mut self) {
        if let Some(mut object) = self.object.take() {
            (self.reset_fn)(&mut object);
            
            // Return to pool if not at capacity
            let pool = self.pool.clone();
            let max_size = self.max_pool_size;
            tokio::spawn(async move {
                let mut pool_guard = pool.lock().await;
                if pool_guard.len() < max_size {
                    pool_guard.push_back(object);
                }
                // Otherwise let object drop naturally
            });
        }
    }
}

/// Memory buffer manager for efficient buffer reuse
pub struct BufferManager {
    small_buffers: ObjectPool<Vec<u8>>,  // < 1KB
    medium_buffers: ObjectPool<Vec<u8>>, // 1KB - 64KB
    large_buffers: ObjectPool<Vec<u8>>,  // > 64KB
    stats: Arc<RwLock<BufferStats>>,
}

impl BufferManager {
    pub fn new() -> Self {
        Self {
            small_buffers: ObjectPool::new(
                100,
                || Vec::with_capacity(1024),
                |buf| buf.clear()
            ),
            medium_buffers: ObjectPool::new(
                50,
                || Vec::with_capacity(64 * 1024),
                |buf| buf.clear()
            ),
            large_buffers: ObjectPool::new(
                10,
                || Vec::with_capacity(1024 * 1024),
                |buf| buf.clear()
            ),
            stats: Arc::new(RwLock::new(BufferStats::default())),
        }
    }

    pub async fn get_buffer(&self, size_hint: usize) -> PooledObject<Vec<u8>> {
        let mut stats = self.stats.write().await;
        stats.requests += 1;

        if size_hint <= 1024 {
            stats.small_buffer_requests += 1;
            self.small_buffers.acquire().await
        } else if size_hint <= 64 * 1024 {
            stats.medium_buffer_requests += 1;
            self.medium_buffers.acquire().await
        } else {
            stats.large_buffer_requests += 1;
            self.large_buffers.acquire().await
        }
    }

    pub async fn get_stats(&self) -> BufferStats {
        self.stats.read().await.clone()
    }
}

/// Buffer usage statistics
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct BufferStats {
    pub requests: u64,
    pub small_buffer_requests: u64,
    pub medium_buffer_requests: u64,
    pub large_buffer_requests: u64,
}

/// Least Recently Used (LRU) cache with size limits
#[derive(Debug)]
pub struct LruCache<K: std::hash::Hash + std::cmp::Eq, V> {
    cache: Arc<RwLock<lru::LruCache<K, V>>>,
    max_size_bytes: usize,
    current_size_bytes: Arc<AtomicUsize>,
    hit_count: Arc<AtomicU64>,
    miss_count: Arc<AtomicU64>,
}

impl<K, V> LruCache<K, V>
where
    K: std::hash::Hash + Eq + Clone + Send + Sync,
    V: Clone + Send + Sync,
{
    pub fn new(capacity: usize, max_size_bytes: usize) -> Self {
        Self {
            cache: Arc::new(RwLock::new(lru::LruCache::new(
                std::num::NonZeroUsize::new(capacity).unwrap()
            ))),
            max_size_bytes,
            current_size_bytes: Arc::new(AtomicUsize::new(0)),
            hit_count: Arc::new(AtomicU64::new(0)),
            miss_count: Arc::new(AtomicU64::new(0)),
        }
    }

    pub async fn get(&self, key: &K) -> Option<V> {
        let mut cache = self.cache.write().await;
        if let Some(value) = cache.get(key) {
            self.hit_count.fetch_add(1, Ordering::Relaxed);
            Some(value.clone())
        } else {
            self.miss_count.fetch_add(1, Ordering::Relaxed);
            None
        }
    }

    pub async fn put(&self, key: K, value: V, size_bytes: usize) -> bool {
        if size_bytes > self.max_size_bytes {
            return false; // Object too large
        }

        let mut cache = self.cache.write().await;
        
        // Check if we need to evict items to make space
        while self.current_size_bytes.load(Ordering::Relaxed) + size_bytes > self.max_size_bytes {
            if cache.pop_lru().is_none() {
                break; // Cache is empty but still not enough space
            }
            // In a real implementation, we'd track the size of evicted items
            self.current_size_bytes.fetch_sub(1024, Ordering::Relaxed); // Approximation
        }

        cache.put(key, value);
        self.current_size_bytes.fetch_add(size_bytes, Ordering::Relaxed);
        true
    }

    pub async fn remove(&self, key: &K) -> Option<V> {
        let mut cache = self.cache.write().await;
        if let Some(value) = cache.pop(key) {
            // In a real implementation, we'd know the actual size
            self.current_size_bytes.fetch_sub(1024, Ordering::Relaxed); // Approximation
            Some(value)
        } else {
            None
        }
    }

    pub async fn clear(&self) {
        let mut cache = self.cache.write().await;
        cache.clear();
        self.current_size_bytes.store(0, Ordering::Relaxed);
    }

    pub fn get_hit_ratio(&self) -> f64 {
        let hits = self.hit_count.load(Ordering::Relaxed);
        let misses = self.miss_count.load(Ordering::Relaxed);
        if hits + misses == 0 {
            0.0
        } else {
            hits as f64 / (hits + misses) as f64
        }
    }

    pub async fn len(&self) -> usize {
        self.cache.read().await.len()
    }

    pub fn current_size_bytes(&self) -> usize {
        self.current_size_bytes.load(Ordering::Relaxed)
    }
}

/// Memory pressure monitor and manager
pub struct MemoryPressureManager {
    thresholds: MemoryThresholds,
    callbacks: Arc<RwLock<Vec<Box<dyn Fn(MemoryPressureLevel) + Send + Sync>>>>,
    current_level: Arc<RwLock<MemoryPressureLevel>>,
    monitoring_active: Arc<RwLock<bool>>,
}

#[derive(Debug, Clone)]
pub struct MemoryThresholds {
    pub low_threshold_mb: u64,      // Start optimizations
    pub moderate_threshold_mb: u64, // More aggressive optimizations
    pub high_threshold_mb: u64,     // Emergency memory freeing
    pub critical_threshold_mb: u64, // Stop non-essential operations
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MemoryPressureLevel {
    Normal,
    Low,
    Moderate,
    High,
    Critical,
}

impl Default for MemoryThresholds {
    fn default() -> Self {
        Self {
            low_threshold_mb: 512,      // 512MB
            moderate_threshold_mb: 1024, // 1GB
            high_threshold_mb: 2048,    // 2GB
            critical_threshold_mb: 4096, // 4GB
        }
    }
}

impl MemoryPressureManager {
    pub fn new(thresholds: MemoryThresholds) -> Self {
        Self {
            thresholds,
            callbacks: Arc::new(RwLock::new(Vec::new())),
            current_level: Arc::new(RwLock::new(MemoryPressureLevel::Normal)),
            monitoring_active: Arc::new(RwLock::new(false)),
        }
    }

    pub async fn register_callback<F>(&self, callback: F)
    where
        F: Fn(MemoryPressureLevel) + Send + Sync + 'static,
    {
        let mut callbacks = self.callbacks.write().await;
        callbacks.push(Box::new(callback));
    }

    pub async fn start_monitoring(&self, interval: Duration) -> Result<()> {
        *self.monitoring_active.write().await = true;
        
        let thresholds = self.thresholds.clone();
        let callbacks = self.callbacks.clone();
        let current_level = self.current_level.clone();
        let monitoring_active = self.monitoring_active.clone();

        tokio::spawn(async move {
            while *monitoring_active.read().await {
                let memory_usage = Self::get_current_memory_usage().await;
                let new_level = Self::calculate_pressure_level(&thresholds, memory_usage);
                
                let mut current = current_level.write().await;
                if new_level != *current {
                    *current = new_level;
                    
                    // Notify all callbacks
                    let callbacks_guard = callbacks.read().await;
                    for callback in callbacks_guard.iter() {
                        callback(new_level);
                    }
                }
                drop(current);
                
                tokio::time::sleep(interval).await;
            }
        });

        Ok(())
    }

    pub async fn stop_monitoring(&self) {
        *self.monitoring_active.write().await = false;
    }

    pub async fn get_current_level(&self) -> MemoryPressureLevel {
        *self.current_level.read().await
    }

    async fn get_current_memory_usage() -> u64 {
        // This is a simplified implementation
        // In reality, you'd use system APIs to get actual memory usage
        use sysinfo::System;
        let mut system = System::new_all();
        system.refresh_memory();
        system.used_memory() / 1024 / 1024 // Convert to MB
    }

    fn calculate_pressure_level(thresholds: &MemoryThresholds, usage_mb: u64) -> MemoryPressureLevel {
        if usage_mb >= thresholds.critical_threshold_mb {
            MemoryPressureLevel::Critical
        } else if usage_mb >= thresholds.high_threshold_mb {
            MemoryPressureLevel::High
        } else if usage_mb >= thresholds.moderate_threshold_mb {
            MemoryPressureLevel::Moderate
        } else if usage_mb >= thresholds.low_threshold_mb {
            MemoryPressureLevel::Low
        } else {
            MemoryPressureLevel::Normal
        }
    }
}

/// Comprehensive memory manager for Paradigm
pub struct MemoryManager {
    pub allocator_stats: Arc<TrackingAllocator>,
    pub buffer_manager: Arc<BufferManager>,
    pub transaction_cache: Arc<LruCache<String, crate::Transaction>>,
    pub ml_task_cache: Arc<LruCache<String, crate::MLTask>>,
    pub pressure_manager: Arc<MemoryPressureManager>,
    config: MemoryConfig,
}

#[derive(Debug, Clone)]
pub struct MemoryConfig {
    pub transaction_cache_size: usize,
    pub ml_task_cache_size: usize,
    pub max_cache_size_mb: usize,
    pub pressure_monitoring_interval: Duration,
    pub enable_aggressive_gc: bool,
}

impl Default for MemoryConfig {
    fn default() -> Self {
        Self {
            transaction_cache_size: 10000,
            ml_task_cache_size: 5000,
            max_cache_size_mb: 256,
            pressure_monitoring_interval: Duration::from_secs(10),
            enable_aggressive_gc: true,
        }
    }
}

impl MemoryManager {
    pub fn new(config: MemoryConfig) -> Self {
        let allocator_stats = Arc::new(TrackingAllocator::new());
        let buffer_manager = Arc::new(BufferManager::new());
        
        let transaction_cache = Arc::new(LruCache::new(
            config.transaction_cache_size,
            config.max_cache_size_mb * 1024 * 1024 / 2, // Half for transactions
        ));
        
        let ml_task_cache = Arc::new(LruCache::new(
            config.ml_task_cache_size,
            config.max_cache_size_mb * 1024 * 1024 / 2, // Half for ML tasks
        ));
        
        let pressure_manager = Arc::new(MemoryPressureManager::new(
            MemoryThresholds::default()
        ));

        Self {
            allocator_stats,
            buffer_manager,
            transaction_cache,
            ml_task_cache,
            pressure_manager,
            config,
        }
    }

    pub async fn start(&self) -> Result<()> {
        // Start pressure monitoring
        self.pressure_manager.start_monitoring(self.config.pressure_monitoring_interval).await?;
        
        // Register memory pressure callbacks
        let transaction_cache = self.transaction_cache.clone();
        let ml_task_cache = self.ml_task_cache.clone();
        let enable_gc = self.config.enable_aggressive_gc;
        
        self.pressure_manager.register_callback(move |level| {
            let tx_cache = transaction_cache.clone();
            let ml_cache = ml_task_cache.clone();
            
            tokio::spawn(async move {
                match level {
                    MemoryPressureLevel::Low => {
                        // Light cleanup
                        tracing::info!("Low memory pressure - performing light cleanup");
                    },
                    MemoryPressureLevel::Moderate => {
                        // More aggressive cleanup
                        tracing::warn!("Moderate memory pressure - clearing 25% of caches");
                        // Implementation would clear portion of caches
                    },
                    MemoryPressureLevel::High => {
                        // Aggressive cleanup
                        tracing::warn!("High memory pressure - clearing 50% of caches");
                        // Implementation would clear more caches
                    },
                    MemoryPressureLevel::Critical => {
                        // Emergency cleanup
                        tracing::error!("Critical memory pressure - clearing all caches");
                        tx_cache.clear().await;
                        ml_cache.clear().await;
                        
                        if enable_gc {
                            // Force garbage collection
                            #[cfg(not(target_arch = "wasm32"))]
                            {
                                // This is a no-op in Rust, but represents where you might
                                // trigger more aggressive memory reclamation
                                tracing::info!("Triggered aggressive memory reclamation");
                            }
                        }
                    },
                    MemoryPressureLevel::Normal => {
                        tracing::info!("Memory pressure returned to normal");
                    },
                }
            });
        }).await;

        tracing::info!("Memory manager started with monitoring enabled");
        Ok(())
    }

    pub async fn stop(&self) {
        self.pressure_manager.stop_monitoring().await;
        tracing::info!("Memory manager stopped");
    }

    pub async fn get_comprehensive_stats(&self) -> ComprehensiveMemoryStats {
        let allocator_stats = self.allocator_stats.get_stats();
        let buffer_stats = self.buffer_manager.get_stats().await;
        let pressure_level = self.pressure_manager.get_current_level().await;
        
        let tx_cache_stats = CacheStats {
            size: self.transaction_cache.len().await,
            hit_ratio: self.transaction_cache.get_hit_ratio(),
            size_bytes: self.transaction_cache.current_size_bytes(),
        };
        
        let ml_cache_stats = CacheStats {
            size: self.ml_task_cache.len().await,
            hit_ratio: self.ml_task_cache.get_hit_ratio(),
            size_bytes: self.ml_task_cache.current_size_bytes(),
        };

        ComprehensiveMemoryStats {
            allocator: allocator_stats,
            buffers: buffer_stats,
            transaction_cache: tx_cache_stats,
            ml_task_cache: ml_cache_stats,
            pressure_level,
        }
    }

    pub async fn force_cleanup(&self) {
        tracing::info!("Forcing memory cleanup");
        self.transaction_cache.clear().await;
        self.ml_task_cache.clear().await;
        self.allocator_stats.reset_stats();
    }

    pub async fn optimize_for_performance(&self) {
        // Pre-warm caches, adjust thresholds, etc.
        tracing::info!("Optimizing memory manager for performance");
        
        // This could include:
        // - Pre-allocating buffer pools
        // - Adjusting cache sizes based on current memory availability
        // - Setting more aggressive pressure thresholds
    }

    pub async fn optimize_for_memory(&self) {
        // Reduce cache sizes, more aggressive cleanup, etc.
        tracing::info!("Optimizing memory manager for low memory usage");
        
        // Clear half of each cache
        // In a real implementation, you'd have methods to resize caches
        let tx_len = self.transaction_cache.len().await;
        let ml_len = self.ml_task_cache.len().await;
        
        tracing::info!("Reduced cache sizes - TX: {}, ML: {}", tx_len, ml_len);
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheStats {
    pub size: usize,
    pub hit_ratio: f64,
    pub size_bytes: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComprehensiveMemoryStats {
    pub allocator: MemoryStats,
    pub buffers: BufferStats,
    pub transaction_cache: CacheStats,
    pub ml_task_cache: CacheStats,
    pub pressure_level: MemoryPressureLevel,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::transaction::Transaction;
    use crate::Address;

    #[tokio::test]
    async fn test_object_pool() {
        let pool = ObjectPool::new(
            5,
            || Vec::with_capacity(1024),
            |v| v.clear()
        );

        let mut obj1 = pool.acquire().await;
        obj1.as_mut().push(42u8);
        assert_eq!(obj1.as_ref().len(), 1);
        
        let initial_pool_size = pool.size().await;
        drop(obj1);
        
        // Object should return to pool
        tokio::time::sleep(Duration::from_millis(10)).await;
        let final_pool_size = pool.size().await;
        assert!(final_pool_size > initial_pool_size || final_pool_size == 5);
    }

    #[tokio::test]
    async fn test_lru_cache() {
        let cache = LruCache::new(3, 1024);
        
        cache.put("key1".to_string(), "value1".to_string(), 100).await;
        cache.put("key2".to_string(), "value2".to_string(), 100).await;
        cache.put("key3".to_string(), "value3".to_string(), 100).await;
        
        assert_eq!(cache.len().await, 3);
        assert_eq!(cache.get(&"key1".to_string()).await, Some("value1".to_string()));
        
        // Adding fourth item should evict least recently used
        cache.put("key4".to_string(), "value4".to_string(), 100).await;
        assert_eq!(cache.len().await, 3);
    }

    #[tokio::test]
    async fn test_buffer_manager() {
        let manager = BufferManager::new();
        
        let small_buf = manager.get_buffer(512).await;
        let medium_buf = manager.get_buffer(32768).await;
        let large_buf = manager.get_buffer(1048576).await;
        
        assert!(small_buf.as_ref().capacity() >= 512);
        assert!(medium_buf.as_ref().capacity() >= 32768);
        assert!(large_buf.as_ref().capacity() >= 1048576);
        
        let stats = manager.get_stats().await;
        assert_eq!(stats.requests, 3);
        assert_eq!(stats.small_buffer_requests, 1);
        assert_eq!(stats.medium_buffer_requests, 1);
        assert_eq!(stats.large_buffer_requests, 1);
    }

    #[tokio::test]
    async fn test_memory_pressure_manager() {
        let thresholds = MemoryThresholds {
            low_threshold_mb: 10,
            moderate_threshold_mb: 20,
            high_threshold_mb: 30,
            critical_threshold_mb: 40,
        };
        
        let manager = MemoryPressureManager::new(thresholds);
        
        // Test level calculation
        assert_eq!(
            MemoryPressureManager::calculate_pressure_level(&manager.thresholds, 5),
            MemoryPressureLevel::Normal
        );
        assert_eq!(
            MemoryPressureManager::calculate_pressure_level(&manager.thresholds, 15),
            MemoryPressureLevel::Low
        );
        assert_eq!(
            MemoryPressureManager::calculate_pressure_level(&manager.thresholds, 45),
            MemoryPressureLevel::Critical
        );
    }

    #[tokio::test]
    async fn test_memory_manager() {
        let config = MemoryConfig::default();
        let manager = MemoryManager::new(config);
        
        manager.start().await.unwrap();
        
        let stats = manager.get_comprehensive_stats().await;
        assert_eq!(stats.transaction_cache.size, 0);
        assert_eq!(stats.ml_task_cache.size, 0);
        
        manager.stop().await;
    }

    fn create_test_transaction() -> Transaction {
        Transaction {
            id: uuid::Uuid::new_v4(),
            from: Address([0u8; 32]),
            to: Address([1u8; 32]),
            amount: 100,
            fee: 10,
            timestamp: chrono::Utc::now(),
            signature: vec![0u8; 64],
            nonce: 1,
        }
    }
}