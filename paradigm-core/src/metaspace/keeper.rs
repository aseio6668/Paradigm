use anyhow::Result;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::{Duration, Instant};
use tokio::fs;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use uuid::Uuid;

use super::sigil::Sigil;
use super::proofs::StorageProof;

/// Status and role of a keeper in the network
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum KeeperStatus {
    /// New keeper, building reputation
    Apprentice,
    
    /// Established keeper with good reputation
    Guardian,
    
    /// Senior keeper with excellent track record
    Archivist,
    
    /// Master keeper with legendary reputation
    Loremaster,
    
    /// Offline or inactive keeper
    Dormant,
    
    /// Keeper with poor performance, restricted access
    Probation,
}

/// A Keeper is a storage node that hosts sigils and earns rewards
/// Keepers can be regular contributors or dedicated storage providers
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Keeper {
    /// Unique identifier for this keeper
    pub keeper_id: String,
    
    /// Network address (IP:port) for communication
    pub network_address: String,
    
    /// Total storage capacity pledged in bytes
    pub capacity: u64,
    
    /// Currently used storage in bytes
    pub used_storage: u64,
    
    /// List of sigil hashes currently stored
    pub sigils_held: Vec<String>,
    
    /// Uptime tracking
    pub uptime_start: DateTime<Utc>,
    pub last_heartbeat: DateTime<Utc>,
    
    /// Reputation score (0.0 to 1.0)
    pub reputation: f64,
    
    /// Performance metrics
    pub metrics: KeeperMetrics,
    
    /// Reward earnings in PAR tokens
    pub total_earned: u64,
    
    /// Storage policies and preferences
    pub policies: KeeperPolicies,
    
    /// Keeper status and role
    pub status: KeeperStatus,
}

/// Performance metrics for a keeper
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeeperMetrics {
    /// Total number of successful retrievals
    pub successful_retrievals: u64,
    
    /// Total number of failed retrievals
    pub failed_retrievals: u64,
    
    /// Average response time in milliseconds
    pub avg_response_time_ms: f64,
    
    /// Number of storage proofs provided
    pub proofs_provided: u64,
    
    /// Number of failed proof challenges
    pub proof_failures: u64,
    
    /// Total data transferred (uploaded + downloaded)
    pub total_data_transferred: u64,
}

/// Storage policies and preferences for a keeper
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeeperPolicies {
    /// Maximum size of individual sigils to accept
    pub max_sigil_size: u64,
    
    /// Preferred glyph types (empty = accept all)
    pub preferred_glyphs: Vec<String>,
    
    /// Minimum reputation required for originators
    pub min_originator_reputation: f64,
    
    /// Whether to accept paid storage requests
    pub accept_paid_storage: bool,
    
    /// Custom storage rate (PAR per GB per month)
    pub storage_rate: Option<u64>,
}


impl Keeper {
    /// Create a new keeper
    pub fn new(network_address: String, capacity: u64) -> Self {
        let keeper_id = format!("keeper_{}", Uuid::new_v4().to_string()[0..8].to_string());
        let now = Utc::now();
        
        Self {
            keeper_id,
            network_address,
            capacity,
            used_storage: 0,
            sigils_held: Vec::new(),
            uptime_start: now,
            last_heartbeat: now,
            reputation: 0.5, // Start with neutral reputation
            metrics: KeeperMetrics::default(),
            total_earned: 0,
            policies: KeeperPolicies::default(),
            status: KeeperStatus::Apprentice,
        }
    }
    
    /// Check if keeper has capacity for a new sigil
    pub fn has_capacity_for_sigil(&self) -> bool {
        self.capacity > self.used_storage + 1024 * 1024 // Keep 1MB buffer
    }
    
    /// Store a sigil (saves encrypted data to disk)
    pub async fn store_sigil(&mut self, sigil: Sigil) -> Result<()> {
        // Check capacity
        if !self.has_capacity_for_sigil() {
            return Err(anyhow::anyhow!("Insufficient storage capacity"));
        }
        
        // Check policies
        if !self.should_accept_sigil(&sigil) {
            return Err(anyhow::anyhow!("Sigil rejected by keeper policies"));
        }
        
        // Create storage directory if it doesn't exist
        let storage_dir = format!("storage/keeper_{}", self.keeper_id);
        fs::create_dir_all(&storage_dir).await?;
        
        // Store sigil metadata
        let sigil_path = format!("{}/{}.json", storage_dir, sigil.content_hash);
        let sigil_data = serde_json::to_string_pretty(&sigil)?;
        let mut file = fs::File::create(&sigil_path).await?;
        file.write_all(sigil_data.as_bytes()).await?;
        
        // Update keeper state
        self.sigils_held.push(sigil.content_hash.clone());
        self.used_storage += sigil.size as u64;
        
        Ok(())
    }
    
    /// Retrieve a sigil's data
    pub async fn retrieve_sigil(&self, sigil_hash: &str) -> Result<Vec<u8>> {
        let start_time = Instant::now();
        
        // Load sigil metadata
        let storage_dir = format!("storage/keeper_{}", self.keeper_id);
        let sigil_path = format!("{}/{}.json", storage_dir, sigil_hash);
        
        // Check if we have this sigil
        if !self.sigils_held.contains(&sigil_hash.to_string()) {
            return Err(anyhow::anyhow!("Sigil not found on this keeper"));
        }
        
        // Read sigil metadata
        let mut file = fs::File::open(&sigil_path).await?;
        let mut contents = String::new();
        file.read_to_string(&mut contents).await?;
        let _sigil: Sigil = serde_json::from_str(&contents)?;
        
        // For now, return mock data - in production this would be encrypted chunks
        let response_time = start_time.elapsed();
        println!("Keeper {} retrieved sigil {} in {:?}", 
                self.keeper_id, sigil_hash, response_time);
                
        // Return placeholder data - in production this would be the actual encrypted chunk
        Ok(vec![42; 1024]) // Mock 1KB of data
    }
    
    /// Generate a proof of storage for a sigil
    pub async fn generate_storage_proof(&self, sigil_hash: &str, challenge: &[u8]) -> Result<StorageProof> {
        // Verify we have the sigil
        if !self.sigils_held.contains(&sigil_hash.to_string()) {
            return Err(anyhow::anyhow!("Cannot generate proof for sigil not held"));
        }
        
        // In a real implementation, this would:
        // 1. Read the stored data
        // 2. Compute a proof based on the challenge
        // 3. Return cryptographic evidence of storage
        
        use sha2::{Sha256, Digest};
        let mut hasher = Sha256::new();
        hasher.update(sigil_hash.as_bytes());
        hasher.update(challenge);
        hasher.update(self.keeper_id.as_bytes());
        
        let proof_hash = hex::encode(hasher.finalize());
        
        Ok(StorageProof {
            keeper_id: self.keeper_id.clone(),
            sigil_hash: sigil_hash.to_string(),
            challenge: challenge.to_vec(),
            proof_data: proof_hash.into_bytes(),
            timestamp: Utc::now(),
        })
    }
    
    /// Update keeper metrics after a retrieval
    pub fn record_retrieval(&mut self, success: bool, response_time_ms: u64) {
        if success {
            self.metrics.successful_retrievals += 1;
        } else {
            self.metrics.failed_retrievals += 1;
        }
        
        // Update average response time
        let total_retrievals = self.metrics.successful_retrievals + self.metrics.failed_retrievals;
        self.metrics.avg_response_time_ms = 
            ((self.metrics.avg_response_time_ms * (total_retrievals - 1) as f64) + response_time_ms as f64) 
            / total_retrievals as f64;
    }
    
    /// Update reputation based on performance
    pub fn update_reputation(&mut self) {
        let total_retrievals = self.metrics.successful_retrievals + self.metrics.failed_retrievals;
        
        if total_retrievals == 0 {
            return; // No data yet
        }
        
        // Success rate component (0.0 to 1.0)
        let success_rate = self.metrics.successful_retrievals as f64 / total_retrievals as f64;
        
        // Response time component (faster = better)
        let response_score = if self.metrics.avg_response_time_ms < 100.0 {
            1.0
        } else if self.metrics.avg_response_time_ms < 1000.0 {
            0.8
        } else if self.metrics.avg_response_time_ms < 5000.0 {
            0.5
        } else {
            0.2
        };
        
        // Proof reliability component
        let total_proofs = self.metrics.proofs_provided + self.metrics.proof_failures;
        let proof_score = if total_proofs > 0 {
            self.metrics.proofs_provided as f64 / total_proofs as f64
        } else {
            0.5 // Neutral if no proofs yet
        };
        
        // Weighted average
        self.reputation = (success_rate * 0.5) + (response_score * 0.3) + (proof_score * 0.2);
        
        // Update status based on reputation
        self.status = match self.reputation {
            r if r >= 0.9 => KeeperStatus::Loremaster,
            r if r >= 0.8 => KeeperStatus::Archivist,
            r if r >= 0.6 => KeeperStatus::Guardian,
            r if r >= 0.3 => KeeperStatus::Apprentice,
            _ => KeeperStatus::Probation,
        };
    }
    
    /// Check if this keeper should accept a sigil based on policies
    fn should_accept_sigil(&self, sigil: &Sigil) -> bool {
        // Check size limit
        if sigil.size as u64 > self.policies.max_sigil_size {
            return false;
        }
        
        // Check preferred glyphs
        if !self.policies.preferred_glyphs.is_empty() 
            && !self.policies.preferred_glyphs.contains(&sigil.glyph.to_string()) {
            return false;
        }
        
        // TODO: Check originator reputation when that system is implemented
        
        true
    }
    
    /// Send heartbeat to update last seen time
    pub fn heartbeat(&mut self) {
        self.last_heartbeat = Utc::now();
    }
    
    /// Check if keeper is considered online
    pub fn is_online(&self) -> bool {
        let now = Utc::now();
        let time_since_heartbeat = now.signed_duration_since(self.last_heartbeat);
        time_since_heartbeat < chrono::Duration::minutes(5)
    }
    
    /// Get storage utilization as percentage
    pub fn storage_utilization(&self) -> f64 {
        if self.capacity == 0 {
            0.0
        } else {
            (self.used_storage as f64 / self.capacity as f64) * 100.0
        }
    }
    
    /// Calculate uptime in hours
    pub fn uptime_hours(&self) -> f64 {
        let now = Utc::now();
        let duration = now.signed_duration_since(self.uptime_start);
        duration.num_seconds() as f64 / 3600.0
    }
}

impl Default for KeeperMetrics {
    fn default() -> Self {
        Self {
            successful_retrievals: 0,
            failed_retrievals: 0,
            avg_response_time_ms: 0.0,
            proofs_provided: 0,
            proof_failures: 0,
            total_data_transferred: 0,
        }
    }
}

impl Default for KeeperPolicies {
    fn default() -> Self {
        Self {
            max_sigil_size: 10 * 1024 * 1024, // 10MB default
            preferred_glyphs: Vec::new(),
            min_originator_reputation: 0.0,
            accept_paid_storage: true,
            storage_rate: None,
        }
    }
}

impl std::fmt::Display for KeeperStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            KeeperStatus::Apprentice => write!(f, "🔰 Apprentice"),
            KeeperStatus::Guardian => write!(f, "🛡️ Guardian"), 
            KeeperStatus::Archivist => write!(f, "📚 Archivist"),
            KeeperStatus::Loremaster => write!(f, "🎭 Loremaster"),
            KeeperStatus::Dormant => write!(f, "😴 Dormant"),
            KeeperStatus::Probation => write!(f, "⚠️ Probation"),
        }
    }
}