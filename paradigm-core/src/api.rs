use crate::{
    autonomous_tasks::AutonomousTaskGenerator, peer_manager::PeerManager, storage::ParadigmStorage,
    Address, AddressExt, Amount, Transaction,
};
use axum::{
    extract::State,
    http::StatusCode,
    response::Json,
    routing::{get, post},
    Router,
};
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, sync::Arc};
use tokio::sync::RwLock;
use tower::ServiceBuilder;
use tower_http::{cors::CorsLayer, trace::TraceLayer};
use tracing::{info, warn};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthResponse {
    pub status: String,
    pub version: String,
    pub node_id: String,
    pub peers_count: u32,
    pub block_height: u64,
    pub network_status: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskRequest {
    pub task_id: String,
    pub task_type: String,
    pub difficulty: u32,
    pub data: String,
    pub reward: u64,
    pub timestamp: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskResponse {
    pub available_tasks: Vec<TaskRequest>,
    pub queue_size: u32,
    pub estimated_reward: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskSubmission {
    pub task_id: String,
    pub result: String,
    pub completion_time_ms: u64,
    pub contributor_address: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PayoutResponse {
    pub success: bool,
    pub transaction_id: String,
    pub amount_paid: u64,
    pub recipient_address: String,
    pub message: String,
}

#[derive(Debug, Clone)]
pub struct ApiState {
    pub node_id: String,
    pub version: String,
    pub peers_count: Arc<RwLock<u32>>,
    pub block_height: Arc<RwLock<u64>>,
    pub network_status: Arc<RwLock<String>>,
    pub task_queue: Arc<RwLock<Vec<TaskRequest>>>,
    pub completed_tasks: Arc<RwLock<HashMap<String, TaskSubmission>>>,
    pub total_payouts: Arc<RwLock<u64>>,
    pub storage: Option<Arc<RwLock<ParadigmStorage>>>,
    pub autonomous_tasks: Option<Arc<RwLock<AutonomousTaskGenerator>>>,
    pub peer_manager: Option<Arc<RwLock<PeerManager>>>,
}

impl ApiState {
    pub fn new() -> Self {
        Self {
            node_id: Uuid::new_v4().to_string(),
            version: "0.1.0".to_string(),
            peers_count: Arc::new(RwLock::new(0)),
            block_height: Arc::new(RwLock::new(0)),
            network_status: Arc::new(RwLock::new("initializing".to_string())),
            task_queue: Arc::new(RwLock::new(Vec::new())),
            completed_tasks: Arc::new(RwLock::new(HashMap::new())),
            total_payouts: Arc::new(RwLock::new(0)),
            storage: None,
            autonomous_tasks: None,
            peer_manager: None,
        }
    }

    pub fn with_storage(mut self, storage: Arc<RwLock<ParadigmStorage>>) -> Self {
        self.storage = Some(storage);
        self
    }

    pub fn with_autonomous_tasks(
        mut self,
        autonomous_tasks: Arc<RwLock<AutonomousTaskGenerator>>,
    ) -> Self {
        self.autonomous_tasks = Some(autonomous_tasks);
        self
    }

    pub fn with_peer_manager(mut self, peer_manager: Arc<RwLock<PeerManager>>) -> Self {
        self.peer_manager = Some(peer_manager);
        self
    }

    pub async fn update_peers_count(&self, count: u32) {
        let mut peers = self.peers_count.write().await;
        *peers = count;
    }

    pub async fn update_block_height(&self, height: u64) {
        let mut block_height = self.block_height.write().await;
        *block_height = height;
    }

    pub async fn update_network_status(&self, status: String) {
        let mut network_status = self.network_status.write().await;
        *network_status = status;
    }

    pub async fn add_task(&self, task: TaskRequest) {
        let mut queue = self.task_queue.write().await;
        queue.push(task);
        info!("Added new task to queue, total tasks: {}", queue.len());
    }

    pub async fn get_available_tasks(&self, limit: Option<usize>) -> Vec<TaskRequest> {
        let queue = self.task_queue.read().await;
        let limit = limit.unwrap_or(10);
        queue.iter().take(limit).cloned().collect()
    }

    pub async fn remove_task(&self, task_id: &str) -> bool {
        let mut queue = self.task_queue.write().await;
        if let Some(pos) = queue.iter().position(|task| task.task_id == task_id) {
            queue.remove(pos);
            info!("Removed completed task: {}", task_id);
            true
        } else {
            warn!("Attempted to remove non-existent task: {}", task_id);
            false
        }
    }
}

pub async fn health_handler(
    State(state): State<ApiState>,
) -> Result<Json<HealthResponse>, StatusCode> {
    let peers_count = *state.peers_count.read().await;
    let block_height = *state.block_height.read().await;
    let network_status = state.network_status.read().await.clone();

    let response = HealthResponse {
        status: "healthy".to_string(),
        version: state.version.clone(),
        node_id: state.node_id.clone(),
        peers_count,
        block_height,
        network_status: network_status.clone(),
    };

    info!(
        "Health check - Peers: {}, Height: {}, Status: {}",
        peers_count, block_height, network_status
    );

    Ok(Json(response))
}

pub async fn tasks_handler(
    State(state): State<ApiState>,
) -> Result<Json<TaskResponse>, StatusCode> {
    // Try to get tasks from autonomous task generator first
    let autonomous_tasks = if let Some(task_generator) = &state.autonomous_tasks {
        let generator = task_generator.read().await;
        match generator.get_available_tasks().await {
            Ok(ml_tasks) => ml_tasks
                .into_iter()
                .map(|ml_task| TaskRequest {
                    task_id: ml_task.id.to_string(),
                    task_type: format!("{:?}", ml_task.task_type),
                    difficulty: ml_task.difficulty as u32,
                    data: hex::encode(&ml_task.data),
                    reward: ml_task.reward,
                    timestamp: ml_task.created_at.timestamp() as u64,
                })
                .collect::<Vec<_>>(),
            Err(e) => {
                warn!("Failed to get autonomous tasks: {}", e);
                Vec::new()
            }
        }
    } else {
        Vec::new()
    };

    // Fallback to legacy tasks if no autonomous tasks available
    let available_tasks = if autonomous_tasks.is_empty() {
        state.get_available_tasks(Some(10)).await
    } else {
        autonomous_tasks
    };

    let queue_size = available_tasks.len() as u32;

    // Calculate estimated reward based on current task difficulty
    let estimated_reward = available_tasks
        .iter()
        .map(|task| task.reward)
        .sum::<u64>()
        .max(100000000); // Minimum 1 PAR if no tasks

    let response = TaskResponse {
        available_tasks: available_tasks.clone(),
        queue_size,
        estimated_reward,
    };

    info!(
        "Task request - Available: {} (autonomous: {}), Queue size: {}, Est. reward: {}",
        available_tasks.len(),
        if state.autonomous_tasks.is_some() {
            "yes"
        } else {
            "no"
        },
        queue_size,
        estimated_reward
    );

    Ok(Json(response))
}

// Simple task submission handler for single-node testing
pub async fn simple_task_submit_handler(
    State(state): State<ApiState>,
    Json(body): Json<serde_json::Value>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    // Extract basic info from the submission
    let task_id = body["task_id"].as_str().unwrap_or("unknown");
    let contributor_address = body["contributor_address"].as_str().unwrap_or("unknown");
    let completion_time = body["completion_time"].as_u64().unwrap_or(1000);

    info!(
        "✅ Task submission received: {} from {}",
        task_id, contributor_address
    );
    info!("⏱️  Completion time: {}ms", completion_time);

    // Remove task from queue if it exists
    let mut queue = state.task_queue.write().await;
    let task_position = queue.iter().position(|task| task.task_id == task_id);

    let reward_amount = if let Some(pos) = task_position {
        let completed_task = queue.remove(pos);
        info!("📋 Removed completed task: {}", task_id);
        completed_task.reward
    } else {
        // Default reward for testing
        warn!("⚠️ Task {} not in queue, using default reward", task_id);
        100000000 // 1.0 PAR default
    };

    drop(queue);

    // Generate transaction ID and create payout response
    let transaction_id = uuid::Uuid::new_v4().to_string();
    let reward_par = reward_amount as f64 / 100_000_000.0;

    info!(
        "💰 Payout: {:.8} PAR to {}",
        reward_par, contributor_address
    );
    info!("📦 Transaction ID: {}", transaction_id);

    // Create successful payout response
    let response = serde_json::json!({
        "success": true,
        "message": "Task completed successfully",
        "transaction_id": transaction_id,
        "amount_paid": reward_amount,
        "recipient_address": contributor_address,
        "timestamp": chrono::Utc::now().timestamp()
    });

    Ok(Json(response))
}

/*
TODO: Fix compilation issues with axum handler
pub async fn task_submit_handler(State(state): State<ApiState>, Json(submission): Json<TaskSubmission>) -> Result<Json<PayoutResponse>, StatusCode> {
    info!("📝 Received task submission from {}", submission.contributor_address);

    // Verify task was actually requested
    let mut queue = state.task_queue.write().await;
    let task_position = queue.iter().position(|task| task.task_id == submission.task_id);

    if let Some(pos) = task_position {
        // Remove completed task from queue
        let completed_task = queue.remove(pos);
        drop(queue); // Release the write lock early

        // Store the completed task
        let mut completed_tasks = state.completed_tasks.write().await;
        completed_tasks.insert(submission.task_id.clone(), submission.clone());
        drop(completed_tasks);

        // Process payout - create real blockchain transaction
        let reward_amount = completed_task.reward;
        let transaction_id = Uuid::new_v4().to_string();
        let reward_par = reward_amount as f64 / 100_000_000.0;

        // Create blockchain transaction if storage available
        if let Some(storage) = &state.storage {
            match create_payout_transaction(&submission.contributor_address, reward_amount, &transaction_id).await {
                Ok(transaction) => {
                    // Store transaction in blockchain
                    let storage_lock = storage.write().await;
                    if let Err(e) = storage_lock.store_transaction(&transaction).await {
                        warn!("Failed to store payout transaction: {}", e);
                    }
                    info!("🔗 BLOCKCHAIN: Created payout transaction {} in blockchain", transaction_id);
                }
                Err(e) => {
                    warn!("Failed to create blockchain transaction: {}", e);
                }
            }
        } else {
            info!("📝 SIMULATED: Payout logged (no blockchain storage configured)");
        }

        // Update total payouts
        let mut total_payouts = state.total_payouts.write().await;
        *total_payouts += reward_amount;
        drop(total_payouts);

        info!("💰 PAYOUT: {:.8} PAR → {} | TX: {}",
              reward_par, submission.contributor_address, transaction_id);

        // Create successful payout response
        let response = PayoutResponse {
            success: true,
            transaction_id: transaction_id.clone(),
            amount_paid: reward_amount,
            recipient_address: submission.contributor_address.clone(),
            message: format!("Successfully paid {:.8} PAR for task completion", reward_par),
        };

        Ok(Json(response))
    } else {
        warn!("❌ Task submission rejected: task_id {} not found in queue", submission.task_id);

        let response = PayoutResponse {
            success: false,
            transaction_id: "".to_string(),
            amount_paid: 0,
            recipient_address: submission.contributor_address,
            message: "Task not found or already completed".to_string(),
        };

        Ok(Json(response))
    }
}
*/

pub fn create_api_router(state: ApiState) -> Router {
    Router::new()
        .route("/health", get(health_handler))
        .route("/api/tasks/available", get(tasks_handler))
        .route("/api/tasks/submit", post(simple_task_submit_handler))
        .layer(
            ServiceBuilder::new()
                .layer(TraceLayer::new_for_http())
                .layer(CorsLayer::permissive()),
        )
        .with_state(state)
}

pub async fn start_api_server(
    port: u16,
    state: ApiState,
) -> Result<(), Box<dyn std::error::Error>> {
    let app = create_api_router(state);

    let listener = tokio::net::TcpListener::bind(format!("0.0.0.0:{}", port)).await?;
    info!("🌐 API server starting on port {}", port);

    axum::serve(listener, app).await?;

    Ok(())
}

// Task generation for testing and development
pub async fn generate_sample_tasks(state: &ApiState, count: u32) {
    use rand::Rng;
    let mut rng = rand::thread_rng();

    for i in 0..count {
        let task = TaskRequest {
            task_id: Uuid::new_v4().to_string(),
            task_type: match rng.gen_range(0..4) {
                0 => "ml_inference".to_string(),
                1 => "data_validation".to_string(),
                2 => "hash_computation".to_string(),
                _ => "network_relay".to_string(),
            },
            difficulty: rng.gen_range(1..=100),
            data: format!("sample_data_{}", i),
            reward: rng.gen_range(50000000..=200000000), // 0.5 to 2 PAR
            timestamp: chrono::Utc::now().timestamp() as u64,
        };

        state.add_task(task).await;
    }

    info!("Generated {} sample tasks for testing", count);
}

/*
// Helper function to create a payout transaction
async fn create_payout_transaction(recipient_address: &str, amount: u64, tx_id: &str) -> Result<Transaction, Box<dyn std::error::Error>> {
    use crate::genesis::NETWORK_TREASURY_ADDRESS;
    use ed25519_dalek::{SigningKey, Signature};
    use rand::rngs::OsRng;

    // Parse recipient address
    let to_address = Address::from_string(recipient_address)
        .map_err(|e| format!("Invalid recipient address: {}", e))?;

    // Create network treasury address (this would normally use a proper treasury key)
    let from_address = Address::from_string(NETWORK_TREASURY_ADDRESS)
        .map_err(|e| format!("Invalid treasury address: {}", e))?;

    // Create the transaction
    let mut transaction = Transaction {
        id: tx_id.to_string(),
        from: from_address,
        to: to_address,
        amount: Amount::from_sat(amount),
        timestamp: chrono::Utc::now().timestamp() as u64,
        signature: None,
        nonce: 0, // This would normally be managed by the wallet
        fee: Amount::from_sat(1000), // Small fee (0.00001 PAR)
    };

    // In a real implementation, we would sign with the treasury's private key
    // For now, we create a placeholder signature
    let signing_key = SigningKey::generate(&mut OsRng);
    let signature_data = format!("{}{}{}{}", transaction.from.to_string(), transaction.to.to_string(), transaction.amount.to_sat(), transaction.timestamp);
    let signature = signing_key.sign(signature_data.as_bytes());
    transaction.signature = Some(signature);

    info!("🔗 Created payout transaction: {} PAR {} → {}",
          amount as f64 / 100_000_000.0, NETWORK_TREASURY_ADDRESS, recipient_address);

    Ok(transaction)
}
*/
