use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

use crate::storage::ParadigmStorage;
use crate::transaction::Transaction;
use crate::Address;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkSyncData {
    pub total_transactions: u64,
    pub total_peers: u64,
    pub latest_block_height: u64,
    pub sync_timestamp: i64,
    pub data_hash: Vec<u8>,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SyncStatus {
    NotStarted,
    Syncing,
    Synced,
    Failed,
}

#[derive(Debug)]
pub struct NetworkSynchronizer {
    storage: Arc<RwLock<ParadigmStorage>>,
    sync_status: SyncStatus,
    sync_progress: f32, // 0.0 to 100.0
    peer_sync_data: HashMap<String, NetworkSyncData>,
    local_sync_data: Option<NetworkSyncData>,
    last_sync_attempt: Option<std::time::Instant>,
}

impl NetworkSynchronizer {
    pub fn new(storage: Arc<RwLock<ParadigmStorage>>) -> Self {
        Self {
            storage,
            sync_status: SyncStatus::NotStarted,
            sync_progress: 0.0,
            peer_sync_data: HashMap::new(),
            local_sync_data: None,
            last_sync_attempt: None,
        }
    }

    pub async fn start_sync(&mut self) -> Result<()> {
        self.sync_status = SyncStatus::Syncing;
        self.sync_progress = 0.0;
        self.last_sync_attempt = Some(std::time::Instant::now());

        // Generate local sync data
        self.local_sync_data = Some(self.generate_local_sync_data().await?);

        tracing::info!("Network synchronization started");
        Ok(())
    }

    pub async fn generate_local_sync_data(&self) -> Result<NetworkSyncData> {
        let storage = self.storage.read().await;

        // Get local network state
        let total_transactions = storage.get_transaction_count().await.unwrap_or(0);
        let latest_block_height = 0; // TODO: implement block height tracking
        let sync_timestamp = chrono::Utc::now().timestamp();

        // Generate data hash for integrity checking
        let mut hasher = blake3::Hasher::new();
        hasher.update(&total_transactions.to_be_bytes());
        hasher.update(&(latest_block_height as u64).to_be_bytes());
        hasher.update(&sync_timestamp.to_be_bytes());
        let data_hash = hasher.finalize().as_bytes().to_vec();

        Ok(NetworkSyncData {
            total_transactions,
            total_peers: 0, // Will be updated by network layer
            latest_block_height,
            sync_timestamp,
            data_hash,
        })
    }

    pub fn update_peer_sync_data(&mut self, peer_id: String, sync_data: NetworkSyncData) {
        self.peer_sync_data.insert(peer_id, sync_data);
        self.calculate_sync_progress();
    }

    pub fn calculate_sync_progress(&mut self) {
        if self.peer_sync_data.is_empty() {
            self.sync_progress = 100.0; // Assume synced if no peers
            self.sync_status = SyncStatus::Synced;
            return;
        }

        let local_data = match &self.local_sync_data {
            Some(data) => data,
            None => {
                self.sync_progress = 0.0;
                return;
            }
        };

        // Calculate average peer state
        let total_peers = self.peer_sync_data.len() as f32;
        let avg_transactions: f32 = self
            .peer_sync_data
            .values()
            .map(|data| data.total_transactions as f32)
            .sum::<f32>()
            / total_peers;

        let avg_block_height: f32 = self
            .peer_sync_data
            .values()
            .map(|data| data.latest_block_height as f32)
            .sum::<f32>()
            / total_peers;

        // Calculate sync percentage based on transaction count and block height
        let transaction_sync = if avg_transactions == 0.0 {
            100.0
        } else {
            (local_data.total_transactions as f32 / avg_transactions * 100.0).min(100.0)
        };

        let block_sync = if avg_block_height == 0.0 {
            100.0
        } else {
            (local_data.latest_block_height as f32 / avg_block_height * 100.0).min(100.0)
        };

        // Overall sync progress (weighted average)
        self.sync_progress = (transaction_sync * 0.7 + block_sync * 0.3).min(100.0);

        // Update sync status
        if self.sync_progress >= 99.5 {
            self.sync_status = SyncStatus::Synced;
        } else {
            self.sync_status = SyncStatus::Syncing;
        }

        tracing::debug!(
            "Sync progress: {:.1}% (Transactions: {:.1}%, Blocks: {:.1}%)",
            self.sync_progress,
            transaction_sync,
            block_sync
        );
    }

    pub fn get_sync_progress(&self) -> f32 {
        self.sync_progress
    }

    pub fn get_sync_status(&self) -> SyncStatus {
        self.sync_status
    }

    pub fn get_sync_percentage(&self) -> u8 {
        self.sync_progress.round() as u8
    }

    pub fn is_synced(&self) -> bool {
        matches!(self.sync_status, SyncStatus::Synced)
    }

    pub fn get_peer_count(&self) -> usize {
        self.peer_sync_data.len()
    }

    pub fn get_local_sync_data(&self) -> Option<&NetworkSyncData> {
        self.local_sync_data.as_ref()
    }

    pub fn get_sync_info(&self) -> SyncInfo {
        SyncInfo {
            status: self.sync_status,
            progress_percentage: self.get_sync_percentage(),
            peer_count: self.get_peer_count(),
            last_sync_attempt: self.last_sync_attempt,
            local_transactions: self
                .local_sync_data
                .as_ref()
                .map(|data| data.total_transactions)
                .unwrap_or(0),
        }
    }

    pub async fn force_resync(&mut self) -> Result<()> {
        tracing::info!("Forcing network resync...");
        self.sync_status = SyncStatus::Syncing;
        self.sync_progress = 0.0;
        self.peer_sync_data.clear();
        self.start_sync().await?;
        Ok(())
    }

    pub fn mark_sync_failed(&mut self) {
        self.sync_status = SyncStatus::Failed;
        tracing::warn!("Network synchronization failed");
    }
}

#[derive(Debug, Clone)]
pub struct SyncInfo {
    pub status: SyncStatus,
    pub progress_percentage: u8,
    pub peer_count: usize,
    pub last_sync_attempt: Option<std::time::Instant>,
    pub local_transactions: u64,
}

impl SyncInfo {
    pub fn status_string(&self) -> &'static str {
        match self.status {
            SyncStatus::NotStarted => "Not Started",
            SyncStatus::Syncing => "Syncing",
            SyncStatus::Synced => "Synced",
            SyncStatus::Failed => "Failed",
        }
    }

    pub fn progress_bar(&self) -> String {
        let filled = (self.progress_percentage as f32 / 100.0 * 20.0) as usize;
        let empty = 20 - filled;
        format!(
            "[{}{}] {}%",
            "█".repeat(filled),
            "░".repeat(empty),
            self.progress_percentage
        )
    }
}
