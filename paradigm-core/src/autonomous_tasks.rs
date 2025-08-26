use anyhow::Result;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, VecDeque};
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

use crate::consensus::{MLTask, MLTaskType};
use crate::network_sync::NetworkSynchronizer;
use crate::peer_manager::PeerManager;
use crate::storage::ParadigmStorage;
use crate::Address;

/// Autonomous task generation system for network maintenance and ML workloads
#[derive(Debug)]
pub struct AutonomousTaskGenerator {
    /// Task queue
    pending_tasks: Arc<RwLock<VecDeque<GeneratedTask>>>,
    /// Task history for analysis
    completed_tasks: Arc<RwLock<HashMap<Uuid, CompletedTask>>>,
    /// Configuration
    config: TaskGenerationConfig,
    /// Network state dependencies
    storage: Arc<RwLock<ParadigmStorage>>,
    peer_manager: Arc<RwLock<PeerManager>>,
    network_sync: Arc<RwLock<NetworkSynchronizer>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskGenerationConfig {
    /// How often to generate new tasks (seconds)
    pub generation_interval: u64,
    /// Maximum pending tasks in queue
    pub max_pending_tasks: usize,
    /// Base task rewards
    pub base_network_task_reward: u64,
    pub base_validation_task_reward: u64,
    pub base_ml_task_reward: u64,
    /// Task priority weights
    pub network_health_priority: f64,
    pub validation_priority: f64,
    pub ml_workload_priority: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeneratedTask {
    pub id: Uuid,
    pub task_type: AutonomousTaskType,
    pub description: String,
    pub reward: u64,
    pub difficulty: u32,
    pub estimated_duration: u32, // seconds
    pub created_at: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
    pub requirements: TaskRequirements,
    pub ml_task: MLTask, // Converted to MLTask for processing
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompletedTask {
    pub id: Uuid,
    pub task_type: AutonomousTaskType,
    pub completed_at: DateTime<Utc>,
    pub contributor: Option<Address>,
    pub success: bool,
    pub execution_time: u32,
    pub reward_paid: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum AutonomousTaskType {
    /// Network health monitoring
    NetworkHeartbeat,
    PeerConnectivityCheck,
    NetworkLatencyMeasurement,
    
    /// Transaction and blockchain validation
    TransactionValidation,
    BlockchainIntegrityCheck,
    BalanceConsistencyVerification,
    
    /// Network synchronization
    DataSyncVerification,
    PeerStateComparison,
    ChainConsistencyCheck,
    
    /// Machine learning workloads
    ModelInference,
    DataProcessing,
    NetworkOptimization,
    
    /// System maintenance
    DatabaseCleanup,
    PeerReputationUpdate,
    MetricsCollection,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskRequirements {
    pub min_reputation: f64,
    pub required_features: Vec<String>,
    pub max_execution_time: u32,
    pub resource_requirements: ResourceRequirements,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceRequirements {
    pub cpu_cores: u32,
    pub memory_mb: u32,
    pub network_bandwidth: u32, // KB/s
    pub storage_mb: u32,
}

impl Default for TaskGenerationConfig {
    fn default() -> Self {
        Self {
            generation_interval: 60,     // Generate tasks every minute
            max_pending_tasks: 100,
            base_network_task_reward: 10_000_000,    // 0.1 PAR
            base_validation_task_reward: 50_000_000, // 0.5 PAR
            base_ml_task_reward: 100_000_000,        // 1.0 PAR
            network_health_priority: 1.0,
            validation_priority: 0.8,
            ml_workload_priority: 0.6,
        }
    }
}

impl AutonomousTaskGenerator {
    pub fn new(
        storage: Arc<RwLock<ParadigmStorage>>,
        peer_manager: Arc<RwLock<PeerManager>>,
        network_sync: Arc<RwLock<NetworkSynchronizer>>,
    ) -> Self {
        Self {
            pending_tasks: Arc::new(RwLock::new(VecDeque::new())),
            completed_tasks: Arc::new(RwLock::new(HashMap::new())),
            config: TaskGenerationConfig::default(),
            storage,
            peer_manager,
            network_sync,
        }
    }

    /// Start the autonomous task generation system
    pub async fn start(&self) -> Result<()> {
        self.start_task_generation_loop().await;
        self.start_task_cleanup_loop().await;
        tracing::info!("Started autonomous task generation system");
        Ok(())
    }

    /// Generate tasks based on current network state
    pub async fn generate_tasks(&self) -> Result<Vec<GeneratedTask>> {
        let mut new_tasks = Vec::new();

        // Check current network state
        let network_health = self.assess_network_health().await?;
        let validation_needs = self.assess_validation_needs().await?;
        let ml_workload = self.assess_ml_workload().await?;

        // Generate network health tasks
        if network_health.needs_heartbeat {
            new_tasks.push(self.create_network_heartbeat_task().await?);
        }
        if network_health.needs_connectivity_check {
            new_tasks.push(self.create_peer_connectivity_task().await?);
        }
        if network_health.needs_latency_check {
            new_tasks.push(self.create_latency_measurement_task().await?);
        }

        // Generate validation tasks
        if validation_needs.pending_transactions > 10 {
            new_tasks.push(self.create_transaction_validation_task().await?);
        }
        if validation_needs.needs_integrity_check {
            new_tasks.push(self.create_blockchain_integrity_task().await?);
        }

        // Generate ML tasks based on workload
        if ml_workload.has_inference_requests {
            new_tasks.push(self.create_model_inference_task().await?);
        }
        if ml_workload.needs_optimization {
            new_tasks.push(self.create_network_optimization_task().await?);
        }

        // Generate maintenance tasks periodically
        if self.should_generate_maintenance_tasks().await {
            new_tasks.push(self.create_metrics_collection_task().await?);
            new_tasks.push(self.create_peer_reputation_update_task().await?);
        }

        tracing::info!("Generated {} autonomous tasks", new_tasks.len());
        Ok(new_tasks)
    }

    /// Get available tasks for contributors
    pub async fn get_available_tasks(&self) -> Result<Vec<MLTask>> {
        let pending_tasks = self.pending_tasks.read().await;
        
        let ml_tasks: Vec<MLTask> = pending_tasks.iter()
            .filter(|task| task.expires_at > Utc::now())
            .map(|task| task.ml_task.clone())
            .collect();

        Ok(ml_tasks)
    }

    /// Mark task as completed
    pub async fn complete_task(&self, task_id: Uuid, contributor: Address, success: bool, execution_time: u32) -> Result<()> {
        // Remove from pending tasks
        let mut pending_tasks = self.pending_tasks.write().await;
        let task = pending_tasks.iter().find(|t| t.id == task_id).cloned();
        
        if let Some(completed_task_data) = task {
            pending_tasks.retain(|t| t.id != task_id);
            
            // Add to completed tasks
            let mut completed_tasks = self.completed_tasks.write().await;
            let completed_task = CompletedTask {
                id: task_id,
                task_type: completed_task_data.task_type,
                completed_at: Utc::now(),
                contributor: Some(contributor.clone()),
                success,
                execution_time,
                reward_paid: if success { completed_task_data.reward } else { 0 },
            };
            
            completed_tasks.insert(task_id, completed_task);
            tracing::info!("Task {} completed by {} (success: {})", task_id, contributor, success);
        }

        Ok(())
    }

    // Task generation methods

    async fn create_network_heartbeat_task(&self) -> Result<GeneratedTask> {
        let task_id = Uuid::new_v4();
        let now = Utc::now();

        let ml_task = MLTask {
            id: task_id,
            task_type: MLTaskType::NetworkOptimization,
            data: vec![1, 2, 3, 4], // Minimal data for heartbeat
            difficulty: 1,
            reward: self.config.base_network_task_reward,
            deadline: now + chrono::Duration::minutes(5),
            created_at: now,
            assigned_to: None,
            completed: false,
            result: None,
        };

        Ok(GeneratedTask {
            id: task_id,
            task_type: AutonomousTaskType::NetworkHeartbeat,
            description: "Network heartbeat check".to_string(),
            reward: self.config.base_network_task_reward,
            difficulty: 1,
            estimated_duration: 30,
            created_at: now,
            expires_at: now + chrono::Duration::minutes(5),
            requirements: TaskRequirements {
                min_reputation: 1.0,
                required_features: vec!["networking".to_string()],
                max_execution_time: 60,
                resource_requirements: ResourceRequirements {
                    cpu_cores: 1,
                    memory_mb: 100,
                    network_bandwidth: 50,
                    storage_mb: 10,
                },
            },
            ml_task,
        })
    }

    async fn create_peer_connectivity_task(&self) -> Result<GeneratedTask> {
        let task_id = Uuid::new_v4();
        let now = Utc::now();

        let peer_stats = self.peer_manager.read().await.get_stats().await;
        let task_data = format!("check_peers:{}", peer_stats.known_peers).into_bytes();

        let ml_task = MLTask {
            id: task_id,
            task_type: MLTaskType::NetworkOptimization,
            data: task_data,
            difficulty: 2,
            reward: self.config.base_network_task_reward * 2,
            deadline: now + chrono::Duration::minutes(10),
            created_at: now,
            assigned_to: None,
            completed: false,
            result: None,
        };

        Ok(GeneratedTask {
            id: task_id,
            task_type: AutonomousTaskType::PeerConnectivityCheck,
            description: "Peer connectivity verification".to_string(),
            reward: self.config.base_network_task_reward * 2,
            difficulty: 2,
            estimated_duration: 120,
            created_at: now,
            expires_at: now + chrono::Duration::minutes(10),
            requirements: TaskRequirements {
                min_reputation: 2.0,
                required_features: vec!["networking".to_string(), "peer_management".to_string()],
                max_execution_time: 180,
                resource_requirements: ResourceRequirements {
                    cpu_cores: 1,
                    memory_mb: 200,
                    network_bandwidth: 100,
                    storage_mb: 50,
                },
            },
            ml_task,
        })
    }

    async fn create_latency_measurement_task(&self) -> Result<GeneratedTask> {
        let task_id = Uuid::new_v4();
        let now = Utc::now();

        let ml_task = MLTask {
            id: task_id,
            task_type: MLTaskType::NetworkOptimization,
            data: vec![0; 64], // Ping payload
            difficulty: 1,
            reward: self.config.base_network_task_reward,
            deadline: now + chrono::Duration::minutes(3),
            created_at: now,
            assigned_to: None,
            completed: false,
            result: None,
        };

        Ok(GeneratedTask {
            id: task_id,
            task_type: AutonomousTaskType::NetworkLatencyMeasurement,
            description: "Network latency measurement".to_string(),
            reward: self.config.base_network_task_reward,
            difficulty: 1,
            estimated_duration: 60,
            created_at: now,
            expires_at: now + chrono::Duration::minutes(3),
            requirements: TaskRequirements {
                min_reputation: 1.0,
                required_features: vec!["networking".to_string()],
                max_execution_time: 90,
                resource_requirements: ResourceRequirements {
                    cpu_cores: 1,
                    memory_mb: 50,
                    network_bandwidth: 200,
                    storage_mb: 10,
                },
            },
            ml_task,
        })
    }

    async fn create_transaction_validation_task(&self) -> Result<GeneratedTask> {
        let task_id = Uuid::new_v4();
        let now = Utc::now();

        let ml_task = MLTask {
            id: task_id,
            task_type: MLTaskType::SmartContractOptimization,
            data: vec![1; 256], // Transaction data placeholder
            difficulty: 3,
            reward: self.config.base_validation_task_reward,
            deadline: now + chrono::Duration::minutes(15),
            created_at: now,
            assigned_to: None,
            completed: false,
            result: None,
        };

        Ok(GeneratedTask {
            id: task_id,
            task_type: AutonomousTaskType::TransactionValidation,
            description: "Transaction validation and verification".to_string(),
            reward: self.config.base_validation_task_reward,
            difficulty: 3,
            estimated_duration: 300,
            created_at: now,
            expires_at: now + chrono::Duration::minutes(15),
            requirements: TaskRequirements {
                min_reputation: 3.0,
                required_features: vec!["validation".to_string(), "cryptography".to_string()],
                max_execution_time: 600,
                resource_requirements: ResourceRequirements {
                    cpu_cores: 2,
                    memory_mb: 500,
                    network_bandwidth: 100,
                    storage_mb: 100,
                },
            },
            ml_task,
        })
    }

    async fn create_blockchain_integrity_task(&self) -> Result<GeneratedTask> {
        let task_id = Uuid::new_v4();
        let now = Utc::now();

        let ml_task = MLTask {
            id: task_id,
            task_type: MLTaskType::SmartContractOptimization,
            data: vec![2; 512], // Blockchain state data
            difficulty: 4,
            reward: self.config.base_validation_task_reward * 2,
            deadline: now + chrono::Duration::minutes(30),
            created_at: now,
            assigned_to: None,
            completed: false,
            result: None,
        };

        Ok(GeneratedTask {
            id: task_id,
            task_type: AutonomousTaskType::BlockchainIntegrityCheck,
            description: "Blockchain integrity verification".to_string(),
            reward: self.config.base_validation_task_reward * 2,
            difficulty: 4,
            estimated_duration: 600,
            created_at: now,
            expires_at: now + chrono::Duration::minutes(30),
            requirements: TaskRequirements {
                min_reputation: 5.0,
                required_features: vec!["validation".to_string(), "blockchain".to_string()],
                max_execution_time: 1200,
                resource_requirements: ResourceRequirements {
                    cpu_cores: 2,
                    memory_mb: 1000,
                    network_bandwidth: 200,
                    storage_mb: 500,
                },
            },
            ml_task,
        })
    }

    async fn create_model_inference_task(&self) -> Result<GeneratedTask> {
        let task_id = Uuid::new_v4();
        let now = Utc::now();

        let ml_task = MLTask {
            id: task_id,
            task_type: MLTaskType::ImageClassification,
            data: vec![3; 1024], // ML model data
            difficulty: 5,
            reward: self.config.base_ml_task_reward,
            deadline: now + chrono::Duration::hours(1),
            created_at: now,
            assigned_to: None,
            completed: false,
            result: None,
        };

        Ok(GeneratedTask {
            id: task_id,
            task_type: AutonomousTaskType::ModelInference,
            description: "Machine learning model inference".to_string(),
            reward: self.config.base_ml_task_reward,
            difficulty: 5,
            estimated_duration: 1800,
            created_at: now,
            expires_at: now + chrono::Duration::hours(1),
            requirements: TaskRequirements {
                min_reputation: 4.0,
                required_features: vec!["ml".to_string(), "gpu".to_string()],
                max_execution_time: 3600,
                resource_requirements: ResourceRequirements {
                    cpu_cores: 4,
                    memory_mb: 2048,
                    network_bandwidth: 500,
                    storage_mb: 1024,
                },
            },
            ml_task,
        })
    }

    async fn create_network_optimization_task(&self) -> Result<GeneratedTask> {
        let task_id = Uuid::new_v4();
        let now = Utc::now();

        let ml_task = MLTask {
            id: task_id,
            task_type: MLTaskType::NetworkOptimization,
            data: vec![4; 768],
            difficulty: 4,
            reward: self.config.base_ml_task_reward / 2,
            deadline: now + chrono::Duration::minutes(45),
            created_at: now,
            assigned_to: None,
            completed: false,
            result: None,
        };

        Ok(GeneratedTask {
            id: task_id,
            task_type: AutonomousTaskType::NetworkOptimization,
            description: "Network performance optimization".to_string(),
            reward: self.config.base_ml_task_reward / 2,
            difficulty: 4,
            estimated_duration: 1200,
            created_at: now,
            expires_at: now + chrono::Duration::minutes(45),
            requirements: TaskRequirements {
                min_reputation: 3.5,
                required_features: vec!["ml".to_string(), "networking".to_string()],
                max_execution_time: 2400,
                resource_requirements: ResourceRequirements {
                    cpu_cores: 2,
                    memory_mb: 1024,
                    network_bandwidth: 300,
                    storage_mb: 200,
                },
            },
            ml_task,
        })
    }

    async fn create_metrics_collection_task(&self) -> Result<GeneratedTask> {
        let task_id = Uuid::new_v4();
        let now = Utc::now();

        let ml_task = MLTask {
            id: task_id,
            task_type: MLTaskType::TimeSeriesAnalysis,
            data: vec![5; 128],
            difficulty: 2,
            reward: self.config.base_network_task_reward,
            deadline: now + chrono::Duration::minutes(20),
            created_at: now,
            assigned_to: None,
            completed: false,
            result: None,
        };

        Ok(GeneratedTask {
            id: task_id,
            task_type: AutonomousTaskType::MetricsCollection,
            description: "Network metrics collection and analysis".to_string(),
            reward: self.config.base_network_task_reward,
            difficulty: 2,
            estimated_duration: 300,
            created_at: now,
            expires_at: now + chrono::Duration::minutes(20),
            requirements: TaskRequirements {
                min_reputation: 2.0,
                required_features: vec!["metrics".to_string()],
                max_execution_time: 600,
                resource_requirements: ResourceRequirements {
                    cpu_cores: 1,
                    memory_mb: 300,
                    network_bandwidth: 150,
                    storage_mb: 100,
                },
            },
            ml_task,
        })
    }

    async fn create_peer_reputation_update_task(&self) -> Result<GeneratedTask> {
        let task_id = Uuid::new_v4();
        let now = Utc::now();

        let ml_task = MLTask {
            id: task_id,
            task_type: MLTaskType::TimeSeriesAnalysis,
            data: vec![6; 256],
            difficulty: 3,
            reward: self.config.base_network_task_reward * 2,
            deadline: now + chrono::Duration::minutes(30),
            created_at: now,
            assigned_to: None,
            completed: false,
            result: None,
        };

        Ok(GeneratedTask {
            id: task_id,
            task_type: AutonomousTaskType::PeerReputationUpdate,
            description: "Peer reputation scoring update".to_string(),
            reward: self.config.base_network_task_reward * 2,
            difficulty: 3,
            estimated_duration: 600,
            created_at: now,
            expires_at: now + chrono::Duration::minutes(30),
            requirements: TaskRequirements {
                min_reputation: 4.0,
                required_features: vec!["analytics".to_string(), "peer_management".to_string()],
                max_execution_time: 900,
                resource_requirements: ResourceRequirements {
                    cpu_cores: 1,
                    memory_mb: 400,
                    network_bandwidth: 100,
                    storage_mb: 200,
                },
            },
            ml_task,
        })
    }

    // Assessment methods

    async fn assess_network_health(&self) -> Result<NetworkHealthAssessment> {
        let peer_stats = self.peer_manager.read().await.get_stats().await;
        let sync_info = self.network_sync.read().await.get_sync_info();

        Ok(NetworkHealthAssessment {
            needs_heartbeat: peer_stats.active_peers < 3,
            needs_connectivity_check: peer_stats.failed_peers > peer_stats.active_peers,
            needs_latency_check: sync_info.progress_percentage < 90,
        })
    }

    async fn assess_validation_needs(&self) -> Result<ValidationAssessment> {
        // TODO: Get actual pending transaction count from storage
        Ok(ValidationAssessment {
            pending_transactions: 15, // Placeholder
            needs_integrity_check: true, // Periodic integrity checks
        })
    }

    async fn assess_ml_workload(&self) -> Result<MLWorkloadAssessment> {
        let completed_count = self.completed_tasks.read().await.len();
        
        Ok(MLWorkloadAssessment {
            has_inference_requests: completed_count < 10, // Generate if we haven't done much
            needs_optimization: completed_count % 50 == 0, // Optimize every 50 tasks
        })
    }

    async fn should_generate_maintenance_tasks(&self) -> bool {
        let completed_tasks = self.completed_tasks.read().await;
        let last_maintenance = completed_tasks.values()
            .filter(|task| matches!(task.task_type, AutonomousTaskType::MetricsCollection | AutonomousTaskType::PeerReputationUpdate))
            .map(|task| task.completed_at)
            .max();

        if let Some(last_time) = last_maintenance {
            Utc::now() - last_time > chrono::Duration::hours(1)
        } else {
            true // No maintenance tasks yet
        }
    }

    // Background task loops

    async fn start_task_generation_loop(&self) {
        let pending_tasks = self.pending_tasks.clone();
        let config = self.config.clone();
        let storage = self.storage.clone();
        let peer_manager = self.peer_manager.clone();
        let network_sync = self.network_sync.clone();

        tokio::spawn(async move {
            let mut interval = tokio::time::interval(
                tokio::time::Duration::from_secs(config.generation_interval)
            );

            loop {
                interval.tick().await;

                let generator = AutonomousTaskGenerator {
                    pending_tasks: pending_tasks.clone(),
                    completed_tasks: Arc::new(RwLock::new(HashMap::new())),
                    config: config.clone(),
                    storage: storage.clone(),
                    peer_manager: peer_manager.clone(),
                    network_sync: network_sync.clone(),
                };

                match generator.generate_tasks().await {
                    Ok(new_tasks) => {
                        let mut pending = pending_tasks.write().await;
                        
                        // Add new tasks to queue
                        for task in new_tasks {
                            if pending.len() < config.max_pending_tasks {
                                pending.push_back(task);
                            }
                        }
                    }
                    Err(e) => {
                        tracing::error!("Failed to generate autonomous tasks: {}", e);
                    }
                }
            }
        });
    }

    async fn start_task_cleanup_loop(&self) {
        let pending_tasks = self.pending_tasks.clone();

        tokio::spawn(async move {
            let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(300)); // 5 minutes

            loop {
                interval.tick().await;

                let mut pending = pending_tasks.write().await;
                let now = Utc::now();
                let original_len = pending.len();

                // Remove expired tasks
                pending.retain(|task| task.expires_at > now);

                let removed = original_len - pending.len();
                if removed > 0 {
                    tracing::debug!("Cleaned up {} expired tasks", removed);
                }
            }
        });
    }

    pub async fn get_task_stats(&self) -> TaskStats {
        let pending_tasks = self.pending_tasks.read().await;
        let completed_tasks = self.completed_tasks.read().await;

        let pending_by_type: HashMap<String, usize> = pending_tasks.iter()
            .map(|task| format!("{:?}", task.task_type))
            .fold(HashMap::new(), |mut acc, task_type| {
                *acc.entry(task_type).or_insert(0) += 1;
                acc
            });

        TaskStats {
            pending_tasks: pending_tasks.len(),
            completed_tasks: completed_tasks.len(),
            pending_by_type,
            total_rewards_paid: completed_tasks.values().map(|task| task.reward_paid).sum(),
        }
    }
}

// Assessment structs

#[derive(Debug)]
struct NetworkHealthAssessment {
    needs_heartbeat: bool,
    needs_connectivity_check: bool,
    needs_latency_check: bool,
}

#[derive(Debug)]
struct ValidationAssessment {
    pending_transactions: usize,
    needs_integrity_check: bool,
}

#[derive(Debug)]
struct MLWorkloadAssessment {
    has_inference_requests: bool,
    needs_optimization: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskStats {
    pub pending_tasks: usize,
    pub completed_tasks: usize,
    pub pending_by_type: HashMap<String, usize>,
    pub total_rewards_paid: u64,
}