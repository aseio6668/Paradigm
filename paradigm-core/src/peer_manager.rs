use anyhow::Result;
use chrono::{DateTime, Duration as ChronoDuration, Utc};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet, VecDeque};
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

/// Manages peer connections, discovery, and persistence for network resilience
#[derive(Debug)]
pub struct PeerManager {
    /// Currently connected peers
    active_peers: Arc<RwLock<HashMap<String, ActivePeer>>>,
    /// Known peers database (persisted to disk)
    known_peers: Arc<RwLock<HashMap<String, KnownPeer>>>,
    /// Recently failed peer attempts
    failed_peers: Arc<RwLock<HashMap<String, FailureInfo>>>,
    /// Peer discovery queue
    discovery_queue: Arc<RwLock<VecDeque<String>>>,
    /// Configuration
    config: PeerManagerConfig,
    /// Storage file path
    peers_file: PathBuf,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PeerManagerConfig {
    /// Maximum number of active connections
    pub max_active_peers: usize,
    /// Maximum number of known peers to remember
    pub max_known_peers: usize,
    /// How often to attempt reconnection to known peers (seconds)
    pub reconnection_interval: u64,
    /// How often to save peer database to disk (seconds)  
    pub save_interval: u64,
    /// How often to discover new peers (seconds)
    pub discovery_interval: u64,
    /// Minimum reputation score to attempt connection
    pub min_reputation_threshold: f64,
    /// Time to wait before retrying failed peers (minutes)
    pub failure_retry_delay: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActivePeer {
    pub address: String,
    pub peer_id: Option<String>,
    pub connected_at: DateTime<Utc>,
    pub last_seen: DateTime<Utc>,
    pub bytes_sent: u64,
    pub bytes_received: u64,
    pub latency_ms: u32,
    pub version: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KnownPeer {
    pub address: String,
    pub first_seen: DateTime<Utc>,
    pub last_successful_connection: Option<DateTime<Utc>>,
    pub connection_attempts: u32,
    pub successful_connections: u32,
    pub reputation_score: f64, // 0.0 to 10.0
    pub average_latency: f64,
    pub last_version: Option<String>,
    pub features: HashSet<String>,
    pub is_bootstrap: bool,
    pub quality_metrics: PeerQualityMetrics,
    pub reputation_history: Vec<ReputationEvent>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FailureInfo {
    pub address: String,
    pub failure_count: u32,
    pub last_failure: DateTime<Utc>,
    pub failure_reason: String,
    pub next_retry: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PeerDatabase {
    pub version: String,
    pub updated_at: DateTime<Utc>,
    pub known_peers: HashMap<String, KnownPeer>,
    pub bootstrap_peers: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PeerQualityMetrics {
    pub reliability_score: f64,     // Connection success rate
    pub task_success_rate: f64,     // Task completion success rate
    pub average_response_time: f64, // Average response time in ms
    pub uptime_percentage: f64,     // How often the peer is available
    pub data_quality_score: f64,    // Quality of data provided
    pub bandwidth_score: f64,       // Network performance
    pub consistency_score: f64,     // Consistent behavior over time
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReputationEvent {
    pub event_type: ReputationEventType,
    pub impact: f64, // Positive or negative impact
    pub timestamp: DateTime<Utc>,
    pub description: String,
    pub details: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ReputationEventType {
    TaskCompleted,
    TaskFailed,
    FastResponse,
    SlowResponse,
    ConnectionSuccess,
    ConnectionFailure,
    DataCorruption,
    MaliciousBehavior,
    HelpfulBehavior,
    HighQualityData,
    LowQualityData,
    ConsistentBehavior,
    InconsistentBehavior,
}

impl Default for PeerManagerConfig {
    fn default() -> Self {
        Self {
            max_active_peers: 50,
            max_known_peers: 1000,
            reconnection_interval: 300, // 5 minutes
            save_interval: 60,          // 1 minute
            discovery_interval: 120,    // 2 minutes
            min_reputation_threshold: 1.0,
            failure_retry_delay: 15, // 15 minutes
        }
    }
}

impl PeerManager {
    pub async fn new(data_dir: &str) -> Result<Self> {
        let peers_file = PathBuf::from(data_dir).join("peers.json");
        let config = PeerManagerConfig::default();

        let manager = Self {
            active_peers: Arc::new(RwLock::new(HashMap::new())),
            known_peers: Arc::new(RwLock::new(HashMap::new())),
            failed_peers: Arc::new(RwLock::new(HashMap::new())),
            discovery_queue: Arc::new(RwLock::new(VecDeque::new())),
            config,
            peers_file,
        };

        // Load existing peer database
        manager.load_peers().await?;

        tracing::info!(
            "PeerManager initialized with {} known peers",
            manager.known_peers.read().await.len()
        );

        Ok(manager)
    }

    /// Add a bootstrap peer that's always trusted
    pub async fn add_bootstrap_peer(&self, address: String) -> Result<()> {
        let mut known_peers = self.known_peers.write().await;

        let peer = KnownPeer {
            address: address.clone(),
            first_seen: Utc::now(),
            last_successful_connection: None,
            connection_attempts: 0,
            successful_connections: 0,
            reputation_score: 10.0, // Maximum reputation for bootstrap
            average_latency: 0.0,
            last_version: None,
            features: HashSet::new(),
            is_bootstrap: true,
            quality_metrics: PeerQualityMetrics::default(),
            reputation_history: Vec::new(),
        };

        known_peers.insert(address.clone(), peer);
        tracing::info!("Added bootstrap peer: {}", address);
        Ok(())
    }

    /// Connect to a peer and track the connection
    pub async fn connect_to_peer(&self, address: String) -> Result<bool> {
        // Check if already connected
        if self.is_peer_connected(&address).await {
            return Ok(true);
        }

        // Check if peer is in failure cooldown
        if self.is_peer_in_cooldown(&address).await {
            return Ok(false);
        }

        // Attempt connection (simplified - in real implementation would use networking layer)
        let connection_successful = self.attempt_peer_connection(&address).await?;

        if connection_successful {
            self.on_peer_connected(&address).await?;
            tracing::info!("Successfully connected to peer: {}", address);
            Ok(true)
        } else {
            self.on_peer_connection_failed(&address, "Connection timeout")
                .await?;
            Ok(false)
        }
    }

    /// Handle successful peer connection
    async fn on_peer_connected(&self, address: &str) -> Result<()> {
        let now = Utc::now();

        // Add to active peers
        let mut active_peers = self.active_peers.write().await;
        let active_peer = ActivePeer {
            address: address.to_string(),
            peer_id: Some(format!("peer_{}", Uuid::new_v4())),
            connected_at: now,
            last_seen: now,
            bytes_sent: 0,
            bytes_received: 0,
            latency_ms: 50, // TODO: measure actual latency
            version: Some("1.0.0".to_string()),
        };
        active_peers.insert(address.to_string(), active_peer);

        // Update known peers database
        let mut known_peers = self.known_peers.write().await;
        if let Some(known_peer) = known_peers.get_mut(address) {
            known_peer.last_successful_connection = Some(now);
            known_peer.successful_connections += 1;
            known_peer.reputation_score = (known_peer.reputation_score + 0.5).min(10.0);
        } else {
            // New peer discovered
            let new_peer = KnownPeer {
                address: address.to_string(),
                first_seen: now,
                last_successful_connection: Some(now),
                connection_attempts: 1,
                successful_connections: 1,
                reputation_score: 5.0, // Starting reputation
                average_latency: 50.0,
                last_version: Some("1.0.0".to_string()),
                features: HashSet::new(),
                is_bootstrap: false,
                quality_metrics: PeerQualityMetrics::default(),
                reputation_history: Vec::new(),
            };
            known_peers.insert(address.to_string(), new_peer);
        }

        // Remove from failed peers if present
        self.failed_peers.write().await.remove(address);

        Ok(())
    }

    /// Handle failed peer connection
    async fn on_peer_connection_failed(&self, address: &str, reason: &str) -> Result<()> {
        let now = Utc::now();

        // Update known peer failure count
        let mut known_peers = self.known_peers.write().await;
        if let Some(known_peer) = known_peers.get_mut(address) {
            known_peer.connection_attempts += 1;
            known_peer.reputation_score = (known_peer.reputation_score - 0.2).max(0.0);
        }

        // Add to failed peers with cooldown
        let mut failed_peers = self.failed_peers.write().await;
        let failure_info = failed_peers
            .get(address)
            .cloned()
            .unwrap_or_else(|| FailureInfo {
                address: address.to_string(),
                failure_count: 0,
                last_failure: now,
                failure_reason: reason.to_string(),
                next_retry: now,
            });

        let updated_failure = FailureInfo {
            failure_count: failure_info.failure_count + 1,
            last_failure: now,
            failure_reason: reason.to_string(),
            next_retry: now + ChronoDuration::minutes(self.config.failure_retry_delay),
            ..failure_info
        };

        failed_peers.insert(address.to_string(), updated_failure);
        tracing::warn!("Peer connection failed: {} ({})", address, reason);

        Ok(())
    }

    /// Get peers ready for connection attempts
    pub async fn get_peers_for_connection(&self) -> Vec<String> {
        let known_peers = self.known_peers.read().await;
        let active_peers = self.active_peers.read().await;
        let failed_peers = self.failed_peers.read().await;
        let now = Utc::now();

        let mut candidates = Vec::new();

        for (address, peer) in known_peers.iter() {
            // Skip if already connected
            if active_peers.contains_key(address) {
                continue;
            }

            // Skip if in failure cooldown
            if let Some(failure) = failed_peers.get(address) {
                if failure.next_retry > now {
                    continue;
                }
            }

            // Skip if reputation too low (unless bootstrap)
            if !peer.is_bootstrap && peer.reputation_score < self.config.min_reputation_threshold {
                continue;
            }

            candidates.push((address.clone(), peer.reputation_score));
        }

        // Sort by reputation (highest first)
        candidates.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

        // Return up to max_active_peers candidates
        candidates
            .into_iter()
            .take(self.config.max_active_peers)
            .map(|(addr, _)| addr)
            .collect()
    }

    /// Add a newly discovered peer to the known peers list
    pub async fn add_discovered_peer(&self, address: String) -> Result<()> {
        let mut known_peers = self.known_peers.write().await;

        if known_peers.contains_key(&address) {
            return Ok(()); // Already known
        }

        if known_peers.len() >= self.config.max_known_peers {
            // Remove lowest reputation peer to make space
            if let Some((lowest_addr, _)) = known_peers
                .iter()
                .filter(|(_, peer)| !peer.is_bootstrap) // Don't remove bootstrap peers
                .min_by(|(_, a), (_, b)| {
                    a.reputation_score.partial_cmp(&b.reputation_score).unwrap()
                })
                .map(|(addr, peer)| (addr.clone(), peer.clone()))
            {
                known_peers.remove(&lowest_addr);
                tracing::debug!("Removed low reputation peer: {}", lowest_addr);
            }
        }

        let new_peer = KnownPeer {
            address: address.clone(),
            first_seen: Utc::now(),
            last_successful_connection: None,
            connection_attempts: 0,
            successful_connections: 0,
            reputation_score: 3.0, // Default reputation for discovered peers
            average_latency: 100.0,
            last_version: None,
            features: HashSet::new(),
            is_bootstrap: false,
            quality_metrics: PeerQualityMetrics::default(),
            reputation_history: Vec::new(),
        };

        known_peers.insert(address.clone(), new_peer);
        tracing::info!("Added discovered peer: {}", address);

        Ok(())
    }

    /// Get list of peers to share with other nodes
    pub async fn get_shareable_peers(&self, max_count: usize) -> Vec<String> {
        let known_peers = self.known_peers.read().await;

        known_peers
            .values()
            .filter(|peer| peer.reputation_score >= 5.0) // Only share good peers
            .filter(|peer| peer.last_successful_connection.is_some()) // Only peers we've connected to
            .map(|peer| peer.address.clone())
            .take(max_count)
            .collect()
    }

    /// Start automatic peer management background tasks
    pub async fn start_background_tasks(&self) -> Result<()> {
        self.start_reconnection_task().await;
        self.start_discovery_task().await;
        self.start_save_task().await;
        tracing::info!("Started peer manager background tasks");
        Ok(())
    }

    /// Start automatic reconnection to known peers
    async fn start_reconnection_task(&self) {
        let known_peers = self.known_peers.clone();
        let active_peers = self.active_peers.clone();
        let failed_peers = self.failed_peers.clone();
        let config = self.config.clone();

        tokio::spawn(async move {
            let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(
                config.reconnection_interval,
            ));

            loop {
                interval.tick().await;

                let peers_to_connect = {
                    let known = known_peers.read().await;
                    let active = active_peers.read().await;
                    let failed = failed_peers.read().await;
                    let now = Utc::now();

                    known
                        .values()
                        .filter(|peer| !active.contains_key(&peer.address))
                        .filter(|peer| {
                            peer.reputation_score >= config.min_reputation_threshold
                                || peer.is_bootstrap
                        })
                        .filter(|peer| {
                            if let Some(failure) = failed.get(&peer.address) {
                                failure.next_retry <= now
                            } else {
                                true
                            }
                        })
                        .take(5) // Limit connection attempts per interval
                        .map(|peer| peer.address.clone())
                        .collect::<Vec<_>>()
                };

                for address in peers_to_connect {
                    tracing::debug!("Attempting automatic reconnection to: {}", address);
                    // TODO: Implement actual connection attempt
                }
            }
        });
    }

    /// Start peer discovery task
    async fn start_discovery_task(&self) {
        let discovery_queue = self.discovery_queue.clone();
        let config = self.config.clone();

        tokio::spawn(async move {
            let mut interval =
                tokio::time::interval(tokio::time::Duration::from_secs(config.discovery_interval));

            loop {
                interval.tick().await;

                let queue_size = discovery_queue.read().await.len();
                if queue_size > 0 {
                    tracing::debug!("Processing {} peers in discovery queue", queue_size);
                    // TODO: Implement peer discovery logic
                }
            }
        });
    }

    /// Start periodic save task
    async fn start_save_task(&self) {
        let known_peers = self.known_peers.clone();
        let peers_file = self.peers_file.clone();
        let config = self.config.clone();

        tokio::spawn(async move {
            let mut interval =
                tokio::time::interval(tokio::time::Duration::from_secs(config.save_interval));

            loop {
                interval.tick().await;

                let database = {
                    let peers = known_peers.read().await;
                    PeerDatabase {
                        version: "1.0.0".to_string(),
                        updated_at: Utc::now(),
                        known_peers: peers.clone(),
                        bootstrap_peers: peers
                            .values()
                            .filter(|p| p.is_bootstrap)
                            .map(|p| p.address.clone())
                            .collect(),
                    }
                };

                if let Err(e) = tokio::fs::write(
                    &peers_file,
                    serde_json::to_string_pretty(&database).unwrap_or_default(),
                )
                .await
                {
                    tracing::error!("Failed to save peer database: {}", e);
                } else {
                    tracing::debug!(
                        "Saved peer database with {} peers",
                        database.known_peers.len()
                    );
                }
            }
        });
    }

    // Helper methods

    async fn is_peer_connected(&self, address: &str) -> bool {
        self.active_peers.read().await.contains_key(address)
    }

    async fn is_peer_in_cooldown(&self, address: &str) -> bool {
        if let Some(failure) = self.failed_peers.read().await.get(address) {
            failure.next_retry > Utc::now()
        } else {
            false
        }
    }

    async fn attempt_peer_connection(&self, _address: &str) -> Result<bool> {
        // TODO: Implement actual network connection
        // For now, simulate success/failure
        Ok(rand::random::<f32>() > 0.3) // 70% success rate
    }

    async fn load_peers(&self) -> Result<()> {
        if !self.peers_file.exists() {
            tracing::info!("No existing peer database found, starting fresh");
            return Ok(());
        }

        let content = tokio::fs::read_to_string(&self.peers_file).await?;
        let database: PeerDatabase = serde_json::from_str(&content)?;

        let mut known_peers = self.known_peers.write().await;
        *known_peers = database.known_peers;

        tracing::info!("Loaded {} peers from database", known_peers.len());
        Ok(())
    }

    /// Record a reputation event for a peer
    pub async fn record_reputation_event(
        &self,
        peer_address: &str,
        event_type: ReputationEventType,
        impact: f64,
        description: String,
    ) -> Result<()> {
        let mut known_peers = self.known_peers.write().await;

        if let Some(peer) = known_peers.get_mut(peer_address) {
            // Add to reputation history
            let event = ReputationEvent {
                event_type: event_type.clone(),
                impact,
                timestamp: Utc::now(),
                description: description.clone(),
                details: HashMap::new(),
            };

            peer.reputation_history.push(event);

            // Limit history size to prevent unbounded growth
            if peer.reputation_history.len() > 100 {
                peer.reputation_history.remove(0);
            }

            // Update reputation score
            peer.reputation_score = (peer.reputation_score + impact).clamp(0.0, 10.0);

            // Update quality metrics based on event type
            self.update_quality_metrics(peer, &event_type, impact).await;

            tracing::debug!(
                "Recorded reputation event for {}: {:?} (impact: {})",
                peer_address,
                event_type,
                impact
            );
        }

        Ok(())
    }

    /// Update quality metrics based on reputation events
    async fn update_quality_metrics(
        &self,
        peer: &mut KnownPeer,
        event_type: &ReputationEventType,
        impact: f64,
    ) {
        let metrics = &mut peer.quality_metrics;

        match event_type {
            ReputationEventType::TaskCompleted => {
                metrics.task_success_rate = (metrics.task_success_rate * 0.9 + 1.0 * 0.1).min(1.0);
                metrics.consistency_score = (metrics.consistency_score + 0.1).min(10.0);
            }
            ReputationEventType::TaskFailed => {
                metrics.task_success_rate = (metrics.task_success_rate * 0.9).max(0.0);
                metrics.consistency_score = (metrics.consistency_score - 0.2).max(0.0);
            }
            ReputationEventType::FastResponse => {
                metrics.average_response_time = metrics.average_response_time * 0.8;
                // Improve response time
            }
            ReputationEventType::SlowResponse => {
                metrics.average_response_time = metrics.average_response_time * 1.2;
                // Worsen response time
            }
            ReputationEventType::ConnectionSuccess => {
                metrics.reliability_score =
                    (metrics.reliability_score * 0.95 + 1.0 * 0.05).min(1.0);
                metrics.uptime_percentage =
                    (metrics.uptime_percentage * 0.95 + 1.0 * 0.05).min(1.0);
            }
            ReputationEventType::ConnectionFailure => {
                metrics.reliability_score = (metrics.reliability_score * 0.95).max(0.0);
                metrics.uptime_percentage = (metrics.uptime_percentage * 0.95).max(0.0);
            }
            ReputationEventType::HighQualityData => {
                metrics.data_quality_score = (metrics.data_quality_score + 0.2).min(10.0);
            }
            ReputationEventType::LowQualityData => {
                metrics.data_quality_score = (metrics.data_quality_score - 0.3).max(0.0);
            }
            ReputationEventType::ConsistentBehavior => {
                metrics.consistency_score = (metrics.consistency_score + 0.1).min(10.0);
            }
            ReputationEventType::InconsistentBehavior => {
                metrics.consistency_score = (metrics.consistency_score - 0.2).max(0.0);
            }
            _ => {} // Other events don't directly affect specific metrics
        }
    }

    /// Get detailed peer quality assessment
    pub async fn get_peer_quality_assessment(
        &self,
        peer_address: &str,
    ) -> Option<PeerQualityAssessment> {
        let known_peers = self.known_peers.read().await;

        if let Some(peer) = known_peers.get(peer_address) {
            let metrics = &peer.quality_metrics;

            // Calculate overall quality score
            let overall_score = (metrics.reliability_score * 0.25
                + metrics.task_success_rate * 0.25
                + metrics.uptime_percentage * 0.15
                + (10.0 - metrics.average_response_time.min(10.0) / 100.0) * 0.15
                + metrics.data_quality_score * 0.1
                + metrics.consistency_score * 0.1)
                .clamp(0.0, 10.0);

            Some(PeerQualityAssessment {
                address: peer_address.to_string(),
                overall_score,
                reputation_score: peer.reputation_score,
                metrics: metrics.clone(),
                last_assessment: Utc::now(),
                recent_events: peer
                    .reputation_history
                    .iter()
                    .rev()
                    .take(10)
                    .cloned()
                    .collect(),
                recommendation: if overall_score >= 8.0 {
                    PeerRecommendation::HighlyRecommended
                } else if overall_score >= 6.0 {
                    PeerRecommendation::Recommended
                } else if overall_score >= 4.0 {
                    PeerRecommendation::Neutral
                } else if overall_score >= 2.0 {
                    PeerRecommendation::NotRecommended
                } else {
                    PeerRecommendation::Avoid
                },
            })
        } else {
            None
        }
    }

    /// Get peers ranked by quality
    pub async fn get_peers_by_quality(&self, limit: usize) -> Vec<PeerQualityAssessment> {
        let known_peers = self.known_peers.read().await;

        let mut assessments: Vec<PeerQualityAssessment> = Vec::new();

        for (address, peer) in known_peers.iter() {
            if let Some(assessment) = self.get_peer_quality_assessment(address).await {
                assessments.push(assessment);
            }
        }

        // Sort by overall quality score descending
        assessments.sort_by(|a, b| {
            b.overall_score
                .partial_cmp(&a.overall_score)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        assessments.into_iter().take(limit).collect()
    }

    pub async fn get_stats(&self) -> PeerStats {
        let active_peers = self.active_peers.read().await;
        let known_peers = self.known_peers.read().await;
        let failed_peers = self.failed_peers.read().await;

        PeerStats {
            active_peers: active_peers.len(),
            known_peers: known_peers.len(),
            failed_peers: failed_peers.len(),
            bootstrap_peers: known_peers.values().filter(|p| p.is_bootstrap).count(),
            average_reputation: {
                let scores: Vec<f64> = known_peers.values().map(|p| p.reputation_score).collect();
                if scores.is_empty() {
                    0.0
                } else {
                    scores.iter().sum::<f64>() / scores.len() as f64
                }
            },
            high_quality_peers: known_peers
                .values()
                .filter(|p| p.reputation_score >= 7.0 && p.quality_metrics.reliability_score >= 0.8)
                .count(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PeerStats {
    pub active_peers: usize,
    pub known_peers: usize,
    pub failed_peers: usize,
    pub bootstrap_peers: usize,
    pub average_reputation: f64,
    pub high_quality_peers: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PeerQualityAssessment {
    pub address: String,
    pub overall_score: f64,
    pub reputation_score: f64,
    pub metrics: PeerQualityMetrics,
    pub last_assessment: DateTime<Utc>,
    pub recent_events: Vec<ReputationEvent>,
    pub recommendation: PeerRecommendation,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PeerRecommendation {
    HighlyRecommended,
    Recommended,
    Neutral,
    NotRecommended,
    Avoid,
}
