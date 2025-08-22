// Advanced Cache Manager with Intelligent Policies
// Manages multi-tier caching for optimal performance

use std::collections::{HashMap, VecDeque};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;
use anyhow::Result;
use serde::{Serialize, Deserialize};
use tracing::{info, debug};

/// Cache management configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheConfig {
    pub max_memory_mb: usize,
    pub eviction_policy: EvictionPolicy,
    pub enable_adaptive_sizing: bool,
    pub enable_prefetching: bool,
    pub prefetch_window: usize,
    pub compression_threshold: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EvictionPolicy {
    LRU,
    LFU,
    TTL,
    Adaptive,
}

impl Default for CacheConfig {
    fn default() -> Self {
        Self {
            max_memory_mb: 256,
            eviction_policy: EvictionPolicy::Adaptive,
            enable_adaptive_sizing: true,
            enable_prefetching: true,
            prefetch_window: 100,
            compression_threshold: 1024,
        }
    }
}

/// Intelligent cache manager
pub struct CacheManager {
    config: CacheConfig,
    cache_entries: Arc<RwLock<HashMap<String, CacheEntry>>>,
    access_patterns: Arc<RwLock<AccessPatternAnalyzer>>,
    memory_usage: Arc<RwLock<usize>>,
    metrics: Arc<RwLock<CacheMetrics>>,
}

#[derive(Debug, Clone)]
pub struct CacheEntry {
    pub data: Vec<u8>,
    pub created_at: Instant,
    pub last_accessed: Instant,
    pub access_count: u64,
    pub size_bytes: usize,
    pub priority: CachePriority,
    pub ttl: Option<Duration>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum CachePriority {
    Low = 0,
    Normal = 1,
    High = 2,
    Critical = 3,
}

#[derive(Debug)]
pub struct AccessPatternAnalyzer {
    recent_accesses: VecDeque<String>,
    frequency_map: HashMap<String, u64>,
    sequential_patterns: HashMap<String, Vec<String>>,
}

#[derive(Debug, Default, Clone)]
pub struct CacheMetrics {
    pub hits: u64,
    pub misses: u64,
    pub evictions: u64,
    pub prefetch_hits: u64,
    pub memory_usage_mb: f64,
    pub average_access_time: Duration,
}

impl CacheManager {
    pub fn new(config: CacheConfig) -> Self {
        Self {
            config,
            cache_entries: Arc::new(RwLock::new(HashMap::new())),
            access_patterns: Arc::new(RwLock::new(AccessPatternAnalyzer::new())),
            memory_usage: Arc::new(RwLock::new(0)),
            metrics: Arc::new(RwLock::new(CacheMetrics::default())),
        }
    }

    /// Get item from cache with intelligent prefetching
    pub async fn get(&self, key: &str) -> Option<Vec<u8>> {
        let start_time = Instant::now();
        
        // Record access pattern
        self.record_access(key).await;
        
        let result = {
            let mut cache = self.cache_entries.write().await;
            if let Some(entry) = cache.get_mut(key) {
                entry.last_accessed = Instant::now();
                entry.access_count += 1;
                Some(entry.data.clone())
            } else {
                None
            }
        };
        
        // Update metrics
        let mut metrics = self.metrics.write().await;
        let access_time = start_time.elapsed();
        
        if result.is_some() {
            metrics.hits += 1;
            
            // Trigger prefetching based on patterns
            if self.config.enable_prefetching {
                let cache_manager = self.clone();
                let key_owned = key.to_string();
                tokio::spawn(async move {
                    cache_manager.prefetch_related(&key_owned).await;
                });
            }
        } else {
            metrics.misses += 1;
        }
        
        metrics.average_access_time = self.update_average_time(
            metrics.average_access_time, 
            access_time, 
            metrics.hits + metrics.misses
        );
        
        result
    }

    /// Put item in cache with intelligent placement
    pub async fn put(&self, key: String, data: Vec<u8>, priority: CachePriority) -> Result<()> {
        let size_bytes = data.len();
        
        // Check if we need to evict entries
        if self.should_evict(size_bytes).await {
            self.evict_entries(size_bytes).await?;
        }
        
        // Compress large entries if configured
        let final_data = if size_bytes > self.config.compression_threshold {
            self.compress_data(&data)?
        } else {
            data
        };
        
        let entry = CacheEntry {
            data: final_data,
            created_at: Instant::now(),
            last_accessed: Instant::now(),
            access_count: 1,
            size_bytes,
            priority,
            ttl: self.calculate_ttl(&priority),
        };
        
        {
            let mut cache = self.cache_entries.write().await;
            cache.insert(key, entry);
        }
        
        {
            let mut memory_usage = self.memory_usage.write().await;
            *memory_usage += size_bytes;
        }
        
        Ok(())
    }

    /// Record access for pattern analysis
    async fn record_access(&self, key: &str) {
        if !self.config.enable_prefetching {
            return;
        }
        
        let mut analyzer = self.access_patterns.write().await;
        
        // Record recent access
        analyzer.recent_accesses.push_back(key.to_string());
        if analyzer.recent_accesses.len() > 1000 {
            analyzer.recent_accesses.pop_front();
        }
        
        // Update frequency
        *analyzer.frequency_map.entry(key.to_string()).or_insert(0) += 1;
        
        // Analyze sequential patterns
        if analyzer.recent_accesses.len() >= 2 {
            let prev_key = analyzer.recent_accesses[analyzer.recent_accesses.len() - 2].clone();
            analyzer.sequential_patterns
                .entry(prev_key)
                .or_insert_with(Vec::new)
                .push(key.to_string());
        }
    }

    /// Prefetch related items based on access patterns
    async fn prefetch_related(&self, key: &str) {
        let related_keys = {
            let analyzer = self.access_patterns.read().await;
            analyzer.sequential_patterns
                .get(key)
                .cloned()
                .unwrap_or_default()
        };
        
        for related_key in related_keys.into_iter().take(self.config.prefetch_window) {
            // Check if already cached
            let cache = self.cache_entries.read().await;
            if !cache.contains_key(&related_key) {
                drop(cache);
                
                // Simulate prefetch from storage
                if let Ok(data) = self.fetch_from_storage(&related_key).await {
                    let _ = self.put(related_key, data, CachePriority::Low).await;
                    
                    let mut metrics = self.metrics.write().await;
                    metrics.prefetch_hits += 1;
                }
            }
        }
    }

    /// Check if eviction is needed
    async fn should_evict(&self, new_entry_size: usize) -> bool {
        let memory_usage = self.memory_usage.read().await;
        let max_memory_bytes = self.config.max_memory_mb * 1024 * 1024;
        
        *memory_usage + new_entry_size > max_memory_bytes
    }

    /// Evict entries based on configured policy
    async fn evict_entries(&self, space_needed: usize) -> Result<()> {
        let mut space_freed = 0;
        let mut to_remove = Vec::new();
        
        match self.config.eviction_policy {
            EvictionPolicy::LRU => {
                to_remove = self.get_lru_candidates().await;
            },
            EvictionPolicy::LFU => {
                to_remove = self.get_lfu_candidates().await;
            },
            EvictionPolicy::TTL => {
                to_remove = self.get_expired_candidates().await;
            },
            EvictionPolicy::Adaptive => {
                to_remove = self.get_adaptive_candidates().await;
            },
        }
        
        // Remove candidates until we have enough space
        {
            let mut cache = self.cache_entries.write().await;
            let mut memory_usage = self.memory_usage.write().await;
            
            for key in to_remove {
                if space_freed >= space_needed {
                    break;
                }
                
                if let Some(entry) = cache.remove(&key) {
                    space_freed += entry.size_bytes;
                    *memory_usage = memory_usage.saturating_sub(entry.size_bytes);
                    
                    let mut metrics = self.metrics.write().await;
                    metrics.evictions += 1;
                }
            }
        }
        
        debug!("Evicted entries freeing {} bytes", space_freed);
        Ok(())
    }

    /// Get LRU eviction candidates
    async fn get_lru_candidates(&self) -> Vec<String> {
        let cache = self.cache_entries.read().await;
        let mut candidates: Vec<_> = cache.iter().collect();
        
        candidates.sort_by_key(|(_, entry)| entry.last_accessed);
        candidates.into_iter()
            .take(10) // Evict up to 10 entries
            .map(|(key, _)| key.clone())
            .collect()
    }

    /// Get LFU eviction candidates
    async fn get_lfu_candidates(&self) -> Vec<String> {
        let cache = self.cache_entries.read().await;
        let mut candidates: Vec<_> = cache.iter().collect();
        
        candidates.sort_by_key(|(_, entry)| entry.access_count);
        candidates.into_iter()
            .take(10)
            .map(|(key, _)| key.clone())
            .collect()
    }

    /// Get expired TTL candidates
    async fn get_expired_candidates(&self) -> Vec<String> {
        let cache = self.cache_entries.read().await;
        let now = Instant::now();
        
        cache.iter()
            .filter(|(_, entry)| {
                if let Some(ttl) = entry.ttl {
                    now.duration_since(entry.created_at) > ttl
                } else {
                    false
                }
            })
            .map(|(key, _)| key.clone())
            .collect()
    }

    /// Get adaptive eviction candidates
    async fn get_adaptive_candidates(&self) -> Vec<String> {
        let cache = self.cache_entries.read().await;
        let analyzer = self.access_patterns.read().await;
        
        let mut candidates: Vec<_> = cache.iter().collect();
        
        // Score based on multiple factors
        candidates.sort_by_key(|(key, entry)| {
            let frequency_score = analyzer.frequency_map.get(*key).unwrap_or(&0);
            let recency_score = entry.last_accessed.elapsed().as_secs();
            let priority_score = entry.priority as u64;
            
            // Lower score = better candidate for eviction
            frequency_score * 1000 + priority_score * 10000 - recency_score
        });
        
        candidates.into_iter()
            .take(10)
            .map(|(key, _)| key.clone())
            .collect()
    }

    /// Calculate TTL based on priority
    fn calculate_ttl(&self, priority: &CachePriority) -> Option<Duration> {
        match priority {
            CachePriority::Critical => None, // Never expire
            CachePriority::High => Some(Duration::from_secs(3600)), // 1 hour
            CachePriority::Normal => Some(Duration::from_secs(1800)), // 30 minutes
            CachePriority::Low => Some(Duration::from_secs(300)), // 5 minutes
        }
    }

    /// Compress data for storage efficiency
    fn compress_data(&self, data: &[u8]) -> Result<Vec<u8>> {
        // Mock compression - in real implementation would use actual compression
        Ok(data.to_vec())
    }

    /// Mock storage fetch
    async fn fetch_from_storage(&self, _key: &str) -> Result<Vec<u8>> {
        // Simulate storage latency
        tokio::time::sleep(Duration::from_millis(1)).await;
        Ok(vec![1, 2, 3, 4, 5])
    }

    /// Update average time metric
    fn update_average_time(&self, current_avg: Duration, new_time: Duration, count: u64) -> Duration {
        let total_nanos = current_avg.as_nanos() as f64 * (count - 1) as f64 + new_time.as_nanos() as f64;
        Duration::from_nanos((total_nanos / count as f64) as u64)
    }

    /// Get cache metrics
    pub async fn get_metrics(&self) -> CacheMetrics {
        let mut metrics = self.metrics.read().await.clone();
        let memory_usage = *self.memory_usage.read().await;
        metrics.memory_usage_mb = memory_usage as f64 / (1024.0 * 1024.0);
        metrics
    }

    /// Clear cache
    pub async fn clear(&self) {
        let mut cache = self.cache_entries.write().await;
        cache.clear();
        
        let mut memory_usage = self.memory_usage.write().await;
        *memory_usage = 0;
        
        info!("Cache cleared");
    }
}

impl Clone for CacheManager {
    fn clone(&self) -> Self {
        Self {
            config: self.config.clone(),
            cache_entries: Arc::clone(&self.cache_entries),
            access_patterns: Arc::clone(&self.access_patterns),
            memory_usage: Arc::clone(&self.memory_usage),
            metrics: Arc::clone(&self.metrics),
        }
    }
}

impl AccessPatternAnalyzer {
    fn new() -> Self {
        Self {
            recent_accesses: VecDeque::new(),
            frequency_map: HashMap::new(),
            sequential_patterns: HashMap::new(),
        }
    }
}