// DDoS Protection and Rate Limiting System
// Protects the network against distributed denial of service attacks

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, VecDeque};
use std::net::IpAddr;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;
use uuid::Uuid;

/// Request types for rate limiting
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum RequestType {
    Connection,
    Transaction,
    BlockRequest,
    PeerRequest,
    APICall,
    WebSocketMessage,
}

/// Rate limiting configuration per request type
#[derive(Debug, Clone)]
pub struct RateLimitRule {
    pub max_requests: u32,
    pub window_duration: Duration,
    pub burst_limit: u32,
    pub ban_duration: Duration,
}

impl Default for RateLimitRule {
    fn default() -> Self {
        Self {
            max_requests: 100,
            window_duration: Duration::from_secs(60),
            burst_limit: 10,
            ban_duration: Duration::from_secs(300), // 5 minutes
        }
    }
}

/// Request tracking data
#[derive(Debug, Clone)]
struct RequestTracker {
    requests: VecDeque<Instant>,
    burst_count: u32,
    last_burst_reset: Instant,
    total_violations: u32,
    is_banned: bool,
    ban_expires: Option<Instant>,
}

impl RequestTracker {
    fn new() -> Self {
        Self {
            requests: VecDeque::new(),
            burst_count: 0,
            last_burst_reset: Instant::now(),
            total_violations: 0,
            is_banned: false,
            ban_expires: None,
        }
    }
}

/// Threat detection patterns
#[derive(Debug, Clone)]
pub struct ThreatPattern {
    pub name: String,
    pub description: String,
    pub detection_threshold: u32,
    pub action: ThreatAction,
}

#[derive(Debug, Clone)]
pub enum ThreatAction {
    LogOnly,
    RateLimit,
    TempBan { duration: Duration },
    PermaBan,
    Alert,
}

/// DDoS protection statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DDoSStats {
    pub total_requests: u64,
    pub blocked_requests: u64,
    pub active_bans: u32,
    pub threat_detections: u32,
    pub protection_effectiveness: f64,
    pub top_blocked_ips: Vec<(IpAddr, u32)>,
}

/// Main DDoS protection engine
pub struct DDoSProtection {
    /// Rate limiting rules per request type
    rules: HashMap<RequestType, RateLimitRule>,
    /// Per-IP request tracking
    ip_trackers: Arc<RwLock<HashMap<IpAddr, HashMap<RequestType, RequestTracker>>>>,
    /// Global statistics
    stats: Arc<RwLock<DDoSStats>>,
    /// Threat detection patterns
    threat_patterns: Vec<ThreatPattern>,
    /// Whitelist of trusted IPs
    whitelist: Arc<RwLock<Vec<IpAddr>>>,
    /// Blacklist of permanently banned IPs
    blacklist: Arc<RwLock<Vec<IpAddr>>>,
}

impl DDoSProtection {
    pub fn new() -> Self {
        let mut rules = HashMap::new();
        
        // Configure rate limits for different request types
        rules.insert(RequestType::Connection, RateLimitRule {
            max_requests: 10,
            window_duration: Duration::from_secs(60),
            burst_limit: 3,
            ban_duration: Duration::from_secs(300),
        });
        
        rules.insert(RequestType::Transaction, RateLimitRule {
            max_requests: 100,
            window_duration: Duration::from_secs(60),
            burst_limit: 20,
            ban_duration: Duration::from_secs(600),
        });
        
        rules.insert(RequestType::APICall, RateLimitRule {
            max_requests: 1000,
            window_duration: Duration::from_secs(60),
            burst_limit: 50,
            ban_duration: Duration::from_secs(180),
        });

        rules.insert(RequestType::BlockRequest, RateLimitRule {
            max_requests: 50,
            window_duration: Duration::from_secs(60),
            burst_limit: 10,
            ban_duration: Duration::from_secs(300),
        });

        rules.insert(RequestType::PeerRequest, RateLimitRule {
            max_requests: 20,
            window_duration: Duration::from_secs(60),
            burst_limit: 5,
            ban_duration: Duration::from_secs(300),
        });

        Self {
            rules,
            ip_trackers: Arc::new(RwLock::new(HashMap::new())),
            stats: Arc::new(RwLock::new(DDoSStats {
                total_requests: 0,
                blocked_requests: 0,
                active_bans: 0,
                threat_detections: 0,
                protection_effectiveness: 0.0,
                top_blocked_ips: Vec::new(),
            })),
            threat_patterns: Self::default_threat_patterns(),
            whitelist: Arc::new(RwLock::new(vec![
                "127.0.0.1".parse().unwrap(), // Localhost
                "::1".parse().unwrap(),       // IPv6 localhost
            ])),
            blacklist: Arc::new(RwLock::new(Vec::new())),
        }
    }

    /// Check if request should be allowed
    pub async fn check_request(&self, ip: IpAddr, request_type: RequestType) -> Result<bool> {
        // Update total request count
        {
            let mut stats = self.stats.write().await;
            stats.total_requests += 1;
        }

        // Check whitelist
        if self.is_whitelisted(ip).await {
            return Ok(true);
        }

        // Check blacklist
        if self.is_blacklisted(ip).await {
            self.increment_blocked_count().await;
            tracing::warn!("🚫 Blocked request from blacklisted IP: {}", ip);
            return Ok(false);
        }

        // Get rate limit rule for request type
        let rule = self.rules.get(&request_type).unwrap_or(&RateLimitRule::default()).clone();

        // Check rate limits
        let mut trackers = self.ip_trackers.write().await;
        let ip_map = trackers.entry(ip).or_insert_with(HashMap::new);
        let tracker = ip_map.entry(request_type.clone()).or_insert_with(RequestTracker::new);

        let now = Instant::now();

        // Check if currently banned
        if tracker.is_banned {
            if let Some(ban_expires) = tracker.ban_expires {
                if now < ban_expires {
                    self.increment_blocked_count().await;
                    tracing::debug!("🚫 Request blocked - IP {} still banned", ip);
                    return Ok(false);
                } else {
                    // Ban expired, reset
                    tracker.is_banned = false;
                    tracker.ban_expires = None;
                    tracing::info!("✅ Ban expired for IP {}", ip);
                }
            }
        }

        // Clean old requests outside window
        while let Some(&front_time) = tracker.requests.front() {
            if now.duration_since(front_time) > rule.window_duration {
                tracker.requests.pop_front();
            } else {
                break;
            }
        }

        // Reset burst counter if window expired
        if now.duration_since(tracker.last_burst_reset) > Duration::from_secs(1) {
            tracker.burst_count = 0;
            tracker.last_burst_reset = now;
        }

        // Check burst limit
        if tracker.burst_count >= rule.burst_limit {
            tracker.total_violations += 1;
            self.ban_ip(ip, request_type, rule.ban_duration).await;
            self.increment_blocked_count().await;
            tracing::warn!("🚫 Burst limit exceeded for {} from {}", 
                          self.request_type_name(&request_type), ip);
            return Ok(false);
        }

        // Check rate limit
        if tracker.requests.len() >= rule.max_requests as usize {
            tracker.total_violations += 1;
            self.ban_ip(ip, request_type, rule.ban_duration).await;
            self.increment_blocked_count().await;
            tracing::warn!("🚫 Rate limit exceeded for {} from {} ({} requests in {}s)", 
                          self.request_type_name(&request_type), ip, 
                          tracker.requests.len(), rule.window_duration.as_secs());
            return Ok(false);
        }

        // Record request
        tracker.requests.push_back(now);
        tracker.burst_count += 1;

        // Check for threat patterns
        self.detect_threats(ip, &request_type, tracker).await;

        Ok(true)
    }

    /// Add IP to whitelist
    pub async fn whitelist_ip(&self, ip: IpAddr) {
        let mut whitelist = self.whitelist.write().await;
        if !whitelist.contains(&ip) {
            whitelist.push(ip);
            tracing::info!("✅ Added {} to whitelist", ip);
        }
    }

    /// Add IP to blacklist
    pub async fn blacklist_ip(&self, ip: IpAddr) {
        let mut blacklist = self.blacklist.write().await;
        if !blacklist.contains(&ip) {
            blacklist.push(ip);
            tracing::warn!("🚫 Added {} to permanent blacklist", ip);
        }
    }

    /// Temporarily ban an IP
    async fn ban_ip(&self, ip: IpAddr, request_type: RequestType, duration: Duration) {
        let mut trackers = self.ip_trackers.write().await;
        if let Some(ip_map) = trackers.get_mut(&ip) {
            if let Some(tracker) = ip_map.get_mut(&request_type) {
                tracker.is_banned = true;
                tracker.ban_expires = Some(Instant::now() + duration);
                
                // Update ban count
                let mut stats = self.stats.write().await;
                stats.active_bans += 1;
                
                tracing::warn!("⏰ Temporarily banned {} for {} ({}s)", 
                              ip, self.request_type_name(&request_type), duration.as_secs());
            }
        }
    }

    /// Check if IP is whitelisted
    async fn is_whitelisted(&self, ip: IpAddr) -> bool {
        self.whitelist.read().await.contains(&ip)
    }

    /// Check if IP is blacklisted
    async fn is_blacklisted(&self, ip: IpAddr) -> bool {
        self.blacklist.read().await.contains(&ip)
    }

    /// Increment blocked request counter
    async fn increment_blocked_count(&self) {
        let mut stats = self.stats.write().await;
        stats.blocked_requests += 1;
        stats.protection_effectiveness = 
            (stats.blocked_requests as f64 / stats.total_requests as f64) * 100.0;
    }

    /// Detect threat patterns
    async fn detect_threats(&self, ip: IpAddr, request_type: &RequestType, tracker: &RequestTracker) {
        // Simple threat detection based on violation patterns
        if tracker.total_violations > 5 {
            tracing::warn!("🔍 Threat pattern detected: {} has {} violations", 
                          ip, tracker.total_violations);
            
            let mut stats = self.stats.write().await;
            stats.threat_detections += 1;
        }
        
        // Detect rapid-fire requests (more than 50 requests in 10 seconds)
        if tracker.requests.len() > 50 {
            let recent_requests = tracker.requests.iter()
                .filter(|&&time| Instant::now().duration_since(time) < Duration::from_secs(10))
                .count();
                
            if recent_requests > 50 {
                tracing::error!("🚨 ALERT: Potential DDoS attack from {} - {} requests in 10s", 
                               ip, recent_requests);
            }
        }
    }

    /// Get human-readable request type name
    fn request_type_name(&self, request_type: &RequestType) -> &'static str {
        match request_type {
            RequestType::Connection => "Connection",
            RequestType::Transaction => "Transaction",
            RequestType::BlockRequest => "Block Request",
            RequestType::PeerRequest => "Peer Request",
            RequestType::APICall => "API Call",
            RequestType::WebSocketMessage => "WebSocket Message",
        }
    }

    /// Get current protection statistics
    pub async fn get_stats(&self) -> DDoSStats {
        let mut stats = self.stats.read().await.clone();
        
        // Update active bans count
        let trackers = self.ip_trackers.read().await;
        let mut active_bans = 0;
        let mut ip_block_counts: HashMap<IpAddr, u32> = HashMap::new();
        
        for (ip, ip_map) in trackers.iter() {
            for tracker in ip_map.values() {
                if tracker.is_banned {
                    if let Some(ban_expires) = tracker.ban_expires {
                        if Instant::now() < ban_expires {
                            active_bans += 1;
                            *ip_block_counts.entry(*ip).or_insert(0) += 1;
                        }
                    }
                }
            }
        }
        
        stats.active_bans = active_bans;
        
        // Sort top blocked IPs
        let mut ip_blocks: Vec<_> = ip_block_counts.into_iter().collect();
        ip_blocks.sort_by(|a, b| b.1.cmp(&a.1));
        stats.top_blocked_ips = ip_blocks.into_iter().take(10).collect();
        
        stats
    }

    /// Start cleanup task to remove expired data
    pub async fn start_cleanup_task(&self) {
        let ip_trackers = self.ip_trackers.clone();
        
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(Duration::from_secs(60));
            
            loop {
                interval.tick().await;
                
                let mut trackers = ip_trackers.write().await;
                let now = Instant::now();
                
                // Remove expired bans and old data
                trackers.retain(|_ip, ip_map| {
                    ip_map.retain(|_req_type, tracker| {
                        // Remove expired bans
                        if tracker.is_banned {
                            if let Some(ban_expires) = tracker.ban_expires {
                                if now >= ban_expires {
                                    tracker.is_banned = false;
                                    tracker.ban_expires = None;
                                }
                            }
                        }
                        
                        // Keep tracker if it has recent activity or active ban
                        !tracker.requests.is_empty() || tracker.is_banned
                    });
                    
                    !ip_map.is_empty()
                });
                
                tracing::debug!("🧹 DDoS protection cleanup completed - tracking {} IPs", 
                               trackers.len());
            }
        });
    }

    /// Default threat detection patterns
    fn default_threat_patterns() -> Vec<ThreatPattern> {
        vec![
            ThreatPattern {
                name: "Rapid Connection Attempts".to_string(),
                description: "Multiple connection attempts in short time".to_string(),
                detection_threshold: 20,
                action: ThreatAction::TempBan { 
                    duration: Duration::from_secs(600) 
                },
            },
            ThreatPattern {
                name: "Transaction Spam".to_string(),
                description: "Excessive transaction submissions".to_string(),
                detection_threshold: 100,
                action: ThreatAction::RateLimit,
            },
            ThreatPattern {
                name: "API Abuse".to_string(),
                description: "Excessive API calls".to_string(),
                detection_threshold: 1000,
                action: ThreatAction::TempBan { 
                    duration: Duration::from_secs(300) 
                },
            },
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_rate_limiting() {
        let protection = DDoSProtection::new();
        let test_ip = "192.168.1.100".parse().unwrap();

        // First few requests should be allowed
        for _ in 0..5 {
            let allowed = protection.check_request(test_ip, RequestType::APICall).await.unwrap();
            assert!(allowed);
        }

        // Get stats
        let stats = protection.get_stats().await;
        assert_eq!(stats.total_requests, 5);
        assert_eq!(stats.blocked_requests, 0);
    }

    #[tokio::test]
    async fn test_whitelist() {
        let protection = DDoSProtection::new();
        let test_ip = "10.0.0.1".parse().unwrap();

        // Add to whitelist
        protection.whitelist_ip(test_ip).await;

        // Should allow even excessive requests
        for _ in 0..1000 {
            let allowed = protection.check_request(test_ip, RequestType::APICall).await.unwrap();
            assert!(allowed);
        }
    }
}