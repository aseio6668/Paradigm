use anyhow::Result;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, info, warn};
use uuid::Uuid;


/// Minimum PAR amount that can be distributed (1 satoshi = 0.00000001 PAR)
pub const MIN_PAR_AMOUNT: u64 = 1;

/// Autopool participant information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AutopoolParticipant {
    pub address: String,
    pub computing_power: f64,   // Relative computing power (hash rate, etc.)
    pub contribution_time: u64, // Time spent contributing in seconds
    pub joined_at: DateTime<Utc>,
    pub last_active: DateTime<Utc>,
    pub work_units_completed: u64,
    pub opt_in_timestamp: DateTime<Utc>,
}

/// Autopool work aggregation session
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AutopoolSession {
    pub session_id: Uuid,
    pub created_at: DateTime<Utc>,
    pub target_amount: u64,  // Target PAR amount to reach before distribution
    pub current_amount: u64, // Current accumulated PAR amount
    pub participants: HashMap<String, AutopoolParticipant>,
    pub work_threshold: f64, // Minimum work required to join pool
    pub distribution_pending: bool,
    pub estimated_completion: Option<DateTime<Utc>>,
}

/// Autopool payout distribution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AutopoolPayout {
    pub session_id: Uuid,
    pub participant_address: String,
    pub amount: u64,
    pub share_percentage: f64,
    pub work_contribution: f64,
    pub time_contribution: u64,
    pub calculated_at: DateTime<Utc>,
}

/// Autopool system for aggregating small contributions
#[derive(Debug)]
pub struct AutopoolManager {
    active_sessions: Arc<RwLock<HashMap<Uuid, AutopoolSession>>>,
    completed_sessions: Arc<RwLock<Vec<AutopoolSession>>>,
    participant_history: Arc<RwLock<HashMap<String, Vec<Uuid>>>>, // address -> session_ids
    network_difficulty: Arc<RwLock<f64>>,
    min_payout_threshold: u64, // Minimum amount before triggering autopool
}

impl AutopoolManager {
    pub fn new(min_payout_threshold: u64) -> Self {
        Self {
            active_sessions: Arc::new(RwLock::new(HashMap::new())),
            completed_sessions: Arc::new(RwLock::new(Vec::new())),
            participant_history: Arc::new(RwLock::new(HashMap::new())),
            network_difficulty: Arc::new(RwLock::new(1.0)),
            min_payout_threshold,
        }
    }

    /// Check if a contributor should be offered autopool participation
    pub async fn should_offer_autopool(
        &self,
        estimated_earning: u64,
        computing_power: f64,
    ) -> bool {
        if estimated_earning >= self.min_payout_threshold {
            return false; // Can earn enough individually
        }

        let difficulty = *self.network_difficulty.read().await;

        // Offer autopool if:
        // 1. Estimated earning is below threshold
        // 2. Network difficulty is high relative to computing power
        // 3. Computing power is above minimum threshold to avoid spam

        estimated_earning < self.min_payout_threshold
            && difficulty > computing_power * 10.0
            && computing_power > 0.001 // Minimum computing power
    }

    /// Opt a contributor into autopool system
    pub async fn opt_into_autopool(&self, address: &str, computing_power: f64) -> Result<Uuid> {
        info!("🔄 {} opting into autopool system", address);

        // Find or create suitable session
        let session_id = self.find_or_create_session(computing_power).await?;

        // Add participant to session
        let participant = AutopoolParticipant {
            address: address.to_string(),
            computing_power,
            contribution_time: 0,
            joined_at: Utc::now(),
            last_active: Utc::now(),
            work_units_completed: 0,
            opt_in_timestamp: Utc::now(),
        };

        let mut sessions = self.active_sessions.write().await;
        if let Some(session) = sessions.get_mut(&session_id) {
            session
                .participants
                .insert(address.to_string(), participant);
            info!("✅ Added {} to autopool session {}", address, session_id);
        }

        // Update participant history
        let mut history = self.participant_history.write().await;
        history
            .entry(address.to_string())
            .or_insert_with(Vec::new)
            .push(session_id);

        Ok(session_id)
    }

    /// Record work contribution from a participant
    pub async fn record_work_contribution(
        &self,
        session_id: Uuid,
        address: &str,
        work_amount: f64,
        time_spent: u64,
    ) -> Result<()> {
        debug!(
            "📊 Recording work: {} from {} ({}s)",
            work_amount, address, time_spent
        );

        let mut sessions = self.active_sessions.write().await;
        if let Some(session) = sessions.get_mut(&session_id) {
            if let Some(participant) = session.participants.get_mut(address) {
                participant.work_units_completed += work_amount as u64;
                participant.contribution_time += time_spent;
                participant.last_active = Utc::now();

                debug!(
                    "📈 {} total work: {}, time: {}s",
                    address, participant.work_units_completed, participant.contribution_time
                );
            }
        }

        Ok(())
    }

    /// Add earnings to a session (called when network rewards are earned)
    pub async fn add_session_earnings(&self, session_id: Uuid, amount: u64) -> Result<()> {
        info!(
            "💰 Adding {} PAR to session {}",
            amount as f64 / 100_000_000.0,
            session_id
        );

        let mut sessions = self.active_sessions.write().await;
        if let Some(session) = sessions.get_mut(&session_id) {
            session.current_amount += amount;

            // Check if session is ready for distribution
            if session.current_amount >= session.target_amount {
                info!("🎯 Session {} ready for payout distribution", session_id);
                session.distribution_pending = true;
            }
        }

        Ok(())
    }

    /// Distribute accumulated rewards to participants
    pub async fn distribute_session_rewards(
        &self,
        session_id: Uuid,
    ) -> Result<Vec<AutopoolPayout>> {
        info!("💸 Distributing rewards for session {}", session_id);

        let mut sessions = self.active_sessions.write().await;
        let mut payouts = Vec::new();

        if let Some(session) = sessions.get(&session_id) {
            if !session.distribution_pending {
                return Err(anyhow::anyhow!("Session not ready for distribution"));
            }

            // Calculate total work and time contributions
            let total_work: f64 = session
                .participants
                .values()
                .map(|p| p.work_units_completed as f64)
                .sum();

            let total_time: u64 = session
                .participants
                .values()
                .map(|p| p.contribution_time)
                .sum();

            if total_work == 0.0 {
                warn!("⚠️ No work contributions found in session {}", session_id);
                return Ok(payouts);
            }

            // Distribute rewards proportionally (70% by work, 30% by time)
            for (address, participant) in &session.participants {
                let work_share = if total_work > 0.0 {
                    participant.work_units_completed as f64 / total_work
                } else {
                    0.0
                };

                let time_share = if total_time > 0 {
                    participant.contribution_time as f64 / total_time as f64
                } else {
                    0.0
                };

                // Weighted contribution: 70% work, 30% time
                let total_share = (work_share * 0.7) + (time_share * 0.3);
                let payout_amount = (session.current_amount as f64 * total_share) as u64;

                if payout_amount >= MIN_PAR_AMOUNT {
                    let payout = AutopoolPayout {
                        session_id,
                        participant_address: address.clone(),
                        amount: payout_amount,
                        share_percentage: total_share * 100.0,
                        work_contribution: participant.work_units_completed as f64,
                        time_contribution: participant.contribution_time,
                        calculated_at: Utc::now(),
                    };

                    payouts.push(payout);
                    info!(
                        "💳 {} receives {:.8} PAR ({:.2}% share)",
                        address,
                        payout_amount as f64 / 100_000_000.0,
                        total_share * 100.0
                    );
                }
            }

            // Move session to completed
            if let Some(completed_session) = sessions.remove(&session_id) {
                let mut completed = self.completed_sessions.write().await;
                completed.push(completed_session);
                info!("✅ Session {} completed and archived", session_id);
            }
        }

        Ok(payouts)
    }

    /// Find existing session or create new one
    async fn find_or_create_session(&self, computing_power: f64) -> Result<Uuid> {
        let sessions = self.active_sessions.read().await;

        // Look for session with available spots and similar difficulty level
        for (session_id, session) in sessions.iter() {
            if session.participants.len() < 50 && // Max 50 participants per session
               session.current_amount < session.target_amount &&
               !session.distribution_pending
            {
                return Ok(*session_id);
            }
        }

        // Create new session if none found
        drop(sessions);
        self.create_new_session(computing_power).await
    }

    /// Create a new autopool session
    async fn create_new_session(&self, _computing_power: f64) -> Result<Uuid> {
        let session_id = Uuid::new_v4();
        let target_amount = self.calculate_target_amount().await;

        let session = AutopoolSession {
            session_id,
            created_at: Utc::now(),
            target_amount,
            current_amount: 0,
            participants: HashMap::new(),
            work_threshold: 0.001, // Minimum work threshold
            distribution_pending: false,
            estimated_completion: None,
        };

        let mut sessions = self.active_sessions.write().await;
        sessions.insert(session_id, session);

        info!(
            "🆕 Created new autopool session {} (target: {:.8} PAR)",
            session_id,
            target_amount as f64 / 100_000_000.0
        );

        Ok(session_id)
    }

    /// Calculate target amount based on network conditions
    async fn calculate_target_amount(&self) -> u64 {
        // Target should be high enough to make meaningful payouts
        // but not so high that sessions take forever to complete
        let base_target = self.min_payout_threshold * 10; // 10x minimum threshold
        let difficulty = *self.network_difficulty.read().await;

        // Adjust target based on difficulty - higher difficulty = higher target
        (base_target as f64 * (1.0 + difficulty / 100.0)) as u64
    }

    /// Update network difficulty (called by network module)
    pub async fn update_network_difficulty(&self, new_difficulty: f64) {
        let mut difficulty = self.network_difficulty.write().await;
        *difficulty = new_difficulty;
        debug!("🔧 Updated network difficulty to {:.2}", new_difficulty);
    }

    /// Get active sessions count
    pub async fn get_active_sessions_count(&self) -> usize {
        self.active_sessions.read().await.len()
    }

    /// Get participant count across all active sessions
    pub async fn get_total_participants_count(&self) -> usize {
        let sessions = self.active_sessions.read().await;
        sessions.values().map(|s| s.participants.len()).sum()
    }

    /// Check if address is currently in autopool
    pub async fn is_participant_active(&self, address: &str) -> bool {
        let sessions = self.active_sessions.read().await;
        sessions
            .values()
            .any(|s| s.participants.contains_key(address))
    }

    /// Get session info for a participant
    pub async fn get_participant_session(&self, address: &str) -> Option<Uuid> {
        let sessions = self.active_sessions.read().await;
        for (session_id, session) in sessions.iter() {
            if session.participants.contains_key(address) {
                return Some(*session_id);
            }
        }
        None
    }

    /// Opt out of autopool system
    pub async fn opt_out_of_autopool(&self, address: &str) -> Result<()> {
        info!("🚫 {} opting out of autopool system", address);

        let mut sessions = self.active_sessions.write().await;
        let mut session_to_remove = None;

        for (session_id, session) in sessions.iter_mut() {
            if session.participants.remove(address).is_some() {
                info!("➖ Removed {} from session {}", address, session_id);

                // If session becomes empty, mark it for removal
                if session.participants.is_empty() {
                    session_to_remove = Some(*session_id);
                }
                break;
            }
        }

        // Remove empty session
        if let Some(session_id) = session_to_remove {
            sessions.remove(&session_id);
            info!("🗑️ Removed empty session {}", session_id);
        }

        Ok(())
    }

    /// Get autopool statistics
    pub async fn get_autopool_stats(&self) -> AutopoolStats {
        let sessions = self.active_sessions.read().await;
        let completed = self.completed_sessions.read().await;

        let total_participants = sessions.values().map(|s| s.participants.len()).sum();
        let total_accumulated = sessions.values().map(|s| s.current_amount).sum();
        let total_target = sessions.values().map(|s| s.target_amount).sum();

        AutopoolStats {
            active_sessions: sessions.len(),
            total_participants,
            completed_sessions: completed.len(),
            total_accumulated_par: total_accumulated,
            total_target_par: total_target,
            network_difficulty: *self.network_difficulty.read().await,
            avg_session_progress: if total_target > 0 {
                (total_accumulated as f64 / total_target as f64) * 100.0
            } else {
                0.0
            },
        }
    }
}

/// Autopool system statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AutopoolStats {
    pub active_sessions: usize,
    pub total_participants: usize,
    pub completed_sessions: usize,
    pub total_accumulated_par: u64,
    pub total_target_par: u64,
    pub network_difficulty: f64,
    pub avg_session_progress: f64,
}

impl AutopoolStats {
    pub fn print_summary(&self) {
        println!("🔄 Autopool System Status");
        println!("==========================");
        println!("🎯 Active Sessions: {}", self.active_sessions);
        println!("👥 Total Participants: {}", self.total_participants);
        println!("✅ Completed Sessions: {}", self.completed_sessions);
        println!(
            "💰 Accumulated: {:.8} PAR",
            self.total_accumulated_par as f64 / 100_000_000.0
        );
        println!(
            "🎯 Target Total: {:.8} PAR",
            self.total_target_par as f64 / 100_000_000.0
        );
        println!("⚡ Network Difficulty: {:.2}", self.network_difficulty);
        println!("📊 Avg Progress: {:.1}%", self.avg_session_progress);
    }
}
