use anyhow::Result;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::HashMap;

use super::glyph::Glyph;

/// A Sigil represents a content-addressed piece of data with symbolic meaning
/// Each sigil is an encrypted shard of a larger file, distributed across keepers
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Sigil {
    /// Content hash (SHA-256) that uniquely identifies this sigil
    pub content_hash: String,

    /// Size of the encrypted data in bytes
    pub size: usize,

    /// Symbolic glyph that represents the nature/purpose of this data
    pub glyph: Glyph,

    /// The contributor who originally uploaded this data
    pub originator: String,

    /// List of keeper IDs currently storing this sigil
    pub keepers: Vec<String>,

    /// Shard index within the original file (for erasure coded data)
    pub shard_index: usize,

    /// Timestamp when this sigil was created
    pub created_at: DateTime<Utc>,

    /// Access log (optional, can be anonymized)
    pub retrievals: Vec<RetrievalRecord>,

    /// Contribution weight for reward calculation
    pub value: u64,

    /// Custom metadata that can be attached to the sigil
    pub metadata: HashMap<String, String>,
}

/// Record of a sigil retrieval for auditing and rewards
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetrievalRecord {
    pub timestamp: DateTime<Utc>,
    pub requester: Option<String>, // Can be anonymized
    pub keeper_id: String,
    pub success: bool,
}

impl Sigil {
    /// Create a new sigil from data
    pub fn new(
        data: Vec<u8>,
        glyph: Glyph,
        originator: String,
        shard_index: usize,
    ) -> Result<Self> {
        // Calculate content hash
        let mut hasher = Sha256::new();
        hasher.update(&data);
        let content_hash = hex::encode(hasher.finalize());

        // Calculate value based on data size and glyph importance
        let base_value = data.len() as u64;
        let glyph_multiplier = glyph.importance_multiplier();
        let value = base_value * glyph_multiplier;

        Ok(Self {
            content_hash,
            size: data.len(),
            glyph,
            originator,
            keepers: Vec::new(),
            shard_index,
            created_at: Utc::now(),
            retrievals: Vec::new(),
            value,
            metadata: HashMap::new(),
        })
    }

    /// Add a keeper to the list of nodes storing this sigil
    pub fn add_keeper(&mut self, keeper_id: String) {
        if !self.keepers.contains(&keeper_id) {
            self.keepers.push(keeper_id);
        }
    }

    /// Remove a keeper from the list
    pub fn remove_keeper(&mut self, keeper_id: &str) {
        self.keepers.retain(|id| id != keeper_id);
    }

    /// Record a retrieval attempt
    pub fn record_retrieval(
        &mut self,
        requester: Option<String>,
        keeper_id: String,
        success: bool,
    ) {
        let record = RetrievalRecord {
            timestamp: Utc::now(),
            requester,
            keeper_id,
            success,
        };

        self.retrievals.push(record);

        // Keep only recent retrievals to prevent unbounded growth
        const MAX_RETRIEVALS: usize = 1000;
        if self.retrievals.len() > MAX_RETRIEVALS {
            let cutoff = self.retrievals.len() - MAX_RETRIEVALS;
            self.retrievals.drain(0..cutoff);
        }
    }

    /// Get the "DNA" string - a unique identifier encoding content + metadata
    pub fn get_dna_string(&self) -> String {
        let mut dna_components = [
            self.content_hash.clone(),
            self.glyph.to_string(),
            self.originator.clone(),
            self.created_at.timestamp().to_string(),
        ];

        // Sort for consistent ordering
        dna_components.sort();

        let combined = dna_components.join(":");
        let mut hasher = Sha256::new();
        hasher.update(combined.as_bytes());

        format!("dna_{}", hex::encode(hasher.finalize())[0..16].to_string())
    }

    /// Check if this sigil needs replication (too few keepers)
    pub fn needs_replication(&self, min_keepers: usize) -> bool {
        self.keepers.len() < min_keepers
    }

    /// Get active keepers (those that have responded recently)
    pub fn get_active_keepers(&self) -> Vec<String> {
        // In a full implementation, this would check keeper liveness
        // For now, return all keepers
        self.keepers.clone()
    }

    /// Calculate storage rewards owed to keepers
    pub fn calculate_keeper_rewards(&self, total_reward_pool: u64) -> HashMap<String, u64> {
        let mut rewards = HashMap::new();

        if self.keepers.is_empty() {
            return rewards;
        }

        // Base reward per keeper
        let base_reward = total_reward_pool / self.keepers.len() as u64;

        // Distribute rewards based on successful retrievals
        let successful_retrievals: HashMap<String, u64> = self
            .retrievals
            .iter()
            .filter(|r| r.success)
            .fold(HashMap::new(), |mut acc, r| {
                *acc.entry(r.keeper_id.clone()).or_insert(0) += 1;
                acc
            });

        let total_successful = successful_retrievals.values().sum::<u64>();

        for keeper_id in &self.keepers {
            let successful_count = successful_retrievals.get(keeper_id).unwrap_or(&0);

            let performance_bonus = if total_successful > 0 {
                (base_reward * successful_count) / total_successful
            } else {
                0
            };

            rewards.insert(keeper_id.clone(), base_reward + performance_bonus);
        }

        rewards
    }
}

/// A Tome represents a collection of sigils that together form a complete file
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Tome {
    /// Unique identifier for this tome
    pub tome_hash: String,

    /// List of sigil hashes that comprise this tome
    pub sigil_hashes: Vec<String>,

    /// Original filename (optional)
    pub filename: Option<String>,

    /// MIME type of the original file
    pub mime_type: Option<String>,

    /// Total size of the original file
    pub total_size: usize,

    /// Glyph representing the overall purpose of this tome
    pub glyph: Glyph,

    /// Who created this tome
    pub originator: String,

    /// When this tome was created
    pub created_at: u64,

    /// Access permissions and policies
    pub access_policy: AccessPolicy,

    /// Fusion-specific metadata (if this tome was created via fusion)
    pub fusion_metadata: Option<crate::metaspace::fusion_forge::FusionMetadata>,
}

/// Defines who can access a tome and under what conditions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AccessPolicy {
    /// Anyone can access freely
    Open,
    /// Anyone can access
    Public,
    /// Only the originator can access
    Private,
    /// Specific list of allowed users
    Restricted(Vec<String>),
    /// Requires payment in PAR tokens
    Paid { price_per_access: u64 },
    /// Requires staking PAR tokens
    Staked { required_stake: u64 },
}

impl Tome {
    pub fn new(
        sigil_hashes: Vec<String>,
        glyph: Glyph,
        originator: String,
        filename: Option<String>,
        mime_type: Option<String>,
        total_size: usize,
    ) -> Self {
        // Create tome hash from sigil hashes
        let mut hasher = Sha256::new();
        for hash in &sigil_hashes {
            hasher.update(hash.as_bytes());
        }
        let tome_hash = format!("tome_{}", hex::encode(hasher.finalize()));

        Self {
            tome_hash,
            sigil_hashes,
            filename,
            mime_type,
            total_size,
            glyph,
            originator,
            created_at: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            access_policy: AccessPolicy::Public,
            fusion_metadata: None,
        }
    }

    /// Check if a user can access this tome
    pub fn can_access(&self, user_id: &str, user_balance: u64, user_stake: u64) -> bool {
        match &self.access_policy {
            AccessPolicy::Open => true,
            AccessPolicy::Public => true,
            AccessPolicy::Private => user_id == self.originator,
            AccessPolicy::Restricted(allowed) => allowed.contains(&user_id.to_string()),
            AccessPolicy::Paid { price_per_access } => user_balance >= *price_per_access,
            AccessPolicy::Staked { required_stake } => user_stake >= *required_stake,
        }
    }
}
