use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

use crate::transaction::Transaction;
use crate::Address;

/// Task capabilities for ML workers
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskCapabilities {
    pub has_gpu: bool,
    pub has_high_memory: bool,
    pub supports_distributed: bool,
    pub supports_realtime: bool,
    pub supports_large_dataset: bool,
    pub max_difficulty: u8,
}

/// ML task types that contributors can work on
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MLTaskType {
    ImageClassification,
    NaturalLanguageProcessing,
    TimeSeriesAnalysis,
    ReinforcementLearning,
    AutoML,
    DistributedTraining,
    Oracle,
    SmartContractOptimization,
    NetworkOptimization,
}

/// ML task structure for contributors
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MLTask {
    pub id: Uuid,
    pub task_type: MLTaskType,
    pub data: Vec<u8>,
    pub difficulty: u8, // 1-10 scale
    pub reward: u64,    // PAR reward for completion
    pub deadline: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
    pub assigned_to: Option<Address>,
    pub completed: bool,
    pub result: Option<Vec<u8>>,
}

impl MLTask {
    pub fn new(
        task_type: MLTaskType,
        data: Vec<u8>,
        difficulty: u8,
        reward: u64,
        deadline: DateTime<Utc>,
    ) -> Self {
        MLTask {
            id: Uuid::new_v4(),
            task_type,
            data,
            difficulty,
            reward,
            deadline,
            created_at: Utc::now(),
            assigned_to: None,
            completed: false,
            result: None,
        }
    }

    pub fn assign_to(&mut self, contributor: Address) {
        self.assigned_to = Some(contributor);
    }

    pub fn complete(&mut self, result: Vec<u8>) -> anyhow::Result<()> {
        if self.completed {
            return Err(anyhow::anyhow!("Task already completed"));
        }

        self.result = Some(result);
        self.completed = true;
        Ok(())
    }

    pub fn is_expired(&self) -> bool {
        Utc::now() > self.deadline
    }
}

/// Consensus engine using ML-based contribution rewards
#[derive(Debug)]
pub struct ConsensusEngine {
    active_tasks: HashMap<Uuid, MLTask>,
    contributor_scores: HashMap<Address, f64>,
    pending_rewards: HashMap<Address, u64>,
    network_difficulty: u8,
}

impl ConsensusEngine {
    pub fn new() -> Self {
        ConsensusEngine {
            active_tasks: HashMap::new(),
            contributor_scores: HashMap::new(),
            pending_rewards: HashMap::new(),
            network_difficulty: 1,
        }
    }

    pub async fn start(&mut self) -> anyhow::Result<()> {
        tracing::info!("Starting ML-based consensus engine");

        // Initialize with some basic tasks
        self.generate_initial_tasks().await?;

        Ok(())
    }

    /// Generate initial ML tasks for the network
    async fn generate_initial_tasks(&mut self) -> anyhow::Result<()> {
        let initial_tasks = vec![
            MLTask::new(
                MLTaskType::NetworkOptimization,
                b"optimize_transaction_routing".to_vec(),
                3,
                50000000, // 0.5 PAR
                Utc::now() + chrono::Duration::hours(24),
            ),
            MLTask::new(
                MLTaskType::Oracle,
                b"price_feed_btc_usd".to_vec(),
                2,
                25000000, // 0.25 PAR
                Utc::now() + chrono::Duration::hours(1),
            ),
            MLTask::new(
                MLTaskType::SmartContractOptimization,
                b"gas_optimization_analysis".to_vec(),
                5,
                100000000, // 1 PAR
                Utc::now() + chrono::Duration::hours(48),
            ),
        ];

        for task in initial_tasks {
            self.active_tasks.insert(task.id, task);
        }

        tracing::info!("Generated {} initial ML tasks", self.active_tasks.len());
        Ok(())
    }

    /// Submit a new ML task to the network
    pub async fn submit_task(&mut self, task: MLTask) -> anyhow::Result<()> {
        tracing::info!("Submitting new ML task: {:?}", task.task_type);
        self.active_tasks.insert(task.id, task);
        Ok(())
    }

    /// Get available tasks for a contributor
    pub async fn get_available_tasks(&self, contributor: &Address) -> Vec<&MLTask> {
        self.active_tasks
            .values()
            .filter(|task| !task.completed && task.assigned_to.is_none() && !task.is_expired())
            .collect()
    }

    /// Assign a task to a contributor
    pub async fn assign_task(&mut self, task_id: Uuid, contributor: Address) -> anyhow::Result<()> {
        if let Some(task) = self.active_tasks.get_mut(&task_id) {
            if task.assigned_to.is_some() {
                return Err(anyhow::anyhow!("Task already assigned"));
            }
            if task.completed {
                return Err(anyhow::anyhow!("Task already completed"));
            }
            if task.is_expired() {
                return Err(anyhow::anyhow!("Task expired"));
            }

            task.assign_to(contributor.clone());
            tracing::info!(
                "Assigned task {} to contributor {}",
                task_id,
                contributor.to_string()
            );
            Ok(())
        } else {
            Err(anyhow::anyhow!("Task not found"))
        }
    }

    /// Submit task result from contributor
    pub async fn submit_task_result(
        &mut self,
        task_id: Uuid,
        contributor: Address,
        result: Vec<u8>,
    ) -> anyhow::Result<u64> {
        if let Some(task) = self.active_tasks.get_mut(&task_id) {
            // Verify contributor is assigned to this task
            if task.assigned_to != Some(contributor.clone()) {
                return Err(anyhow::anyhow!("Task not assigned to this contributor"));
            }

            if task.completed {
                return Err(anyhow::anyhow!("Task already completed"));
            }

            // Complete the task
            task.complete(result)?;

            // Calculate reward based on task difficulty and contributor score
            let base_reward = task.reward;
            let contributor_score = self.contributor_scores.get(&contributor).unwrap_or(&1.0);
            let final_reward = (base_reward as f64 * contributor_score) as u64;
            let task_difficulty = task.difficulty;

            // Add to pending rewards
            *self.pending_rewards.entry(contributor.clone()).or_insert(0) += final_reward;

            // Update contributor score (moved outside the borrow scope)
            self.update_contributor_score(&contributor, task_difficulty)
                .await;

            tracing::info!(
                "Task {} completed by {}, reward: {} PAR",
                task_id,
                contributor.to_string(),
                final_reward as f64 / 100_000_000.0
            );

            Ok(final_reward)
        } else {
            Err(anyhow::anyhow!("Task not found"))
        }
    }

    /// Update contributor score based on performance
    async fn update_contributor_score(&mut self, contributor: &Address, task_difficulty: u8) {
        let current_score = self.contributor_scores.get(contributor).unwrap_or(&1.0);

        // Increase score based on task difficulty
        let score_increase = (task_difficulty as f64) * 0.1;
        let new_score = (current_score + score_increase).min(2.0); // Cap at 2.0x multiplier

        self.contributor_scores
            .insert(contributor.clone(), new_score);
    }

    /// Process pending rewards and create reward transactions
    pub async fn process_pending_rewards(&mut self) -> Vec<Transaction> {
        let mut reward_transactions = Vec::new();

        for (contributor, reward) in self.pending_rewards.drain() {
            // Create a reward transaction from the network treasury
            // Note: In a real implementation, this would come from the AI governance system
            tracing::info!(
                "Processing reward of {} PAR for contributor {}",
                reward as f64 / 100_000_000.0,
                contributor.to_string()
            );
        }

        reward_transactions
    }

    /// Get network statistics
    pub async fn get_network_stats(&self) -> NetworkStats {
        let total_tasks = self.active_tasks.len();
        let completed_tasks = self.active_tasks.values().filter(|t| t.completed).count();
        let active_contributors = self.contributor_scores.len();
        let total_rewards_pending: u64 = self.pending_rewards.values().sum();

        NetworkStats {
            total_tasks,
            completed_tasks,
            active_contributors,
            total_rewards_pending,
            network_difficulty: self.network_difficulty,
        }
    }

    /// Adjust network difficulty based on task completion rate
    pub async fn adjust_difficulty(&mut self) {
        let stats = self.get_network_stats().await;
        let completion_rate = if stats.total_tasks > 0 {
            stats.completed_tasks as f64 / stats.total_tasks as f64
        } else {
            0.0
        };

        // Adjust difficulty to maintain ~75% completion rate
        if completion_rate > 0.8 && self.network_difficulty < 10 {
            self.network_difficulty += 1;
        } else if completion_rate < 0.5 && self.network_difficulty > 1 {
            self.network_difficulty -= 1;
        }
    }
}

/// Network statistics structure
#[derive(Debug, Serialize, Deserialize)]
pub struct NetworkStats {
    pub total_tasks: usize,
    pub completed_tasks: usize,
    pub active_contributors: usize,
    pub total_rewards_pending: u64,
    pub network_difficulty: u8,
}

#[cfg(test)]
mod tests {
    use super::*;
    use ed25519_dalek::SigningKey;
    use rand::thread_rng;

    #[tokio::test]
    async fn test_consensus_engine() {
        let mut engine = ConsensusEngine::new();
        engine.start().await.unwrap();

        let keypair = SigningKey::from_bytes(&rand::random());
        let contributor = Address::from_public_key(&keypair.verifying_key());

        // Get available tasks
        let tasks = engine.get_available_tasks(&contributor).await;
        assert!(!tasks.is_empty());

        // Assign a task
        let task_id = tasks[0].id;
        engine
            .assign_task(task_id, contributor.clone())
            .await
            .unwrap();

        // Submit result
        let result = b"task_result_data".to_vec();
        let reward = engine
            .submit_task_result(task_id, contributor, result)
            .await
            .unwrap();
        assert!(reward > 0);
    }

    #[test]
    fn test_ml_task_creation() {
        let task = MLTask::new(
            MLTaskType::Oracle,
            b"test_data".to_vec(),
            5,
            100000000,
            Utc::now() + chrono::Duration::hours(1),
        );

        assert!(!task.completed);
        assert!(task.assigned_to.is_none());
        assert_eq!(task.difficulty, 5);
        assert_eq!(task.reward, 100000000);
    }
}
