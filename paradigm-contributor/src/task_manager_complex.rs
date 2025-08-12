use paradigm_core::{ParadigmNode, ml_tasks::MLTaskEngine, consensus::MLTask};
use anyhow::Result;
use std::sync::{Arc, Mutex};
use std::collections::HashMap;
use tokio::time::{Duration, Instant};
use uuid::Uuid;

use crate::ContributorStats;

/// Manages ML task assignment and execution for contributors
pub struct TaskManager {
    node: Arc<ParadigmNode>,
    ml_engine: Arc<Mutex<MLTaskEngine>>,
    wallet_address: String,
    max_concurrent_tasks: usize,
    active_tasks: HashMap<Uuid, TaskExecution>,
    stats: ContributorStats,
    start_time: Instant,
}

#[derive(Debug)]
struct TaskExecution {
    task: MLTask,
    start_time: Instant,
    handle: tokio::task::JoinHandle<Result<Vec<u8>>>,
}

impl TaskManager {
    pub async fn new(
        node: ParadigmNode,
        ml_engine: MLTaskEngine,
        wallet_address: String,
        max_concurrent_tasks: usize,
    ) -> Result<Self> {
        Ok(TaskManager {
            node: Arc::new(node),
            ml_engine: Arc::new(Mutex::new(ml_engine)),
            wallet_address,
            max_concurrent_tasks,
            active_tasks: HashMap::new(),
            stats: ContributorStats::default(),
            start_time: Instant::now(),
        })
    }

    /// Check for new tasks and start processing them
    pub async fn check_for_tasks(&mut self) -> Result<()> {
        // Don't get new tasks if we're at capacity
        if self.active_tasks.len() >= self.max_concurrent_tasks {
            self.check_completed_tasks().await?;
            return Ok();
        }

        // Get available tasks from the consensus engine
        let consensus = self.node.consensus_engine.read().await;
        let available_tasks = consensus.get_available_tasks(&self.node.address).await;

        if available_tasks.is_empty() {
            return Ok();
        }

        // Select a task to work on (for now, just take the first one)
        if let Some(task) = available_tasks.first() {
            let task_id = task.id;
            tracing::info!("Found available task: {} (type: {:?}, difficulty: {})", 
                          task_id, task.task_type, task.difficulty);

            // Try to assign the task to ourselves
            drop(consensus); // Release the read lock
            let mut consensus = self.node.consensus_engine.write().await;
            
            if let Ok(()) = consensus.assign_task(task_id, self.node.address.clone()).await {
                // Get the task again (now assigned to us)
                if let Some(assigned_task) = consensus.active_tasks.get(&task_id) {
                    self.start_task_execution(assigned_task.clone()).await?;
                }
            }
        }

        Ok(())
    }

    /// Start executing a task
    async fn start_task_execution(&mut self, task: MLTask) -> Result<()> {
        tracing::info!("Starting execution of task: {} (difficulty: {})", task.id, task.difficulty);

        let ml_engine = self.ml_engine.clone();
        let task_clone = task.clone();

        // Spawn the task execution
        let handle = tokio::spawn(async move {
            let mut engine = ml_engine.lock().unwrap();
            engine.execute_task(&task_clone).await
        });

        let execution = TaskExecution {
            task: task.clone(),
            start_time: Instant::now(),
            handle,
        };

        self.active_tasks.insert(task.id, execution);
        Ok(())
    }

    /// Check for completed tasks and submit results
    async fn check_completed_tasks(&mut self) -> Result<()> {
        let mut completed_tasks = Vec::new();

        // Check which tasks have completed
        for (task_id, execution) in &mut self.active_tasks {
            if execution.handle.is_finished() {
                completed_tasks.push(*task_id);
            }
        }

        // Process completed tasks
        for task_id in completed_tasks {
            if let Some(execution) = self.active_tasks.remove(&task_id) {
                let execution_time = execution.start_time.elapsed();
                
                match execution.handle.await {
                    Ok(Ok(result)) => {
                        tracing::info!("Task {} completed successfully in {:?}", task_id, execution_time);
                        
                        // Submit result to the network
                        let mut consensus = self.node.consensus_engine.write().await;
                        match consensus.submit_task_result(
                            task_id,
                            self.node.address.clone(),
                            result,
                        ).await {
                            Ok(reward) => {
                                tracing::info!("Received reward: {:.8} PAR", reward as f64 / 100_000_000.0);
                                self.stats.tasks_completed += 1;
                                self.stats.total_rewards += reward;
                                self.update_average_task_time(execution_time);
                            }
                            Err(e) => {
                                tracing::error!("Failed to submit task result: {}", e);
                            }
                        }
                    }
                    Ok(Err(e)) => {
                        tracing::error!("Task {} execution failed: {}", task_id, e);
                    }
                    Err(e) => {
                        tracing::error!("Task {} panicked: {}", task_id, e);
                    }
                }
            }
        }

        Ok(())
    }

    fn update_average_task_time(&mut self, execution_time: Duration) {
        let total_time = self.stats.average_task_time * (self.stats.tasks_completed - 1) as f64;
        let new_time = execution_time.as_secs_f64();
        self.stats.average_task_time = (total_time + new_time) / self.stats.tasks_completed as f64;
    }

    /// Get current statistics
    pub async fn get_statistics(&mut self) -> ContributorStats {
        self.stats.active_tasks = self.active_tasks.len();
        self.stats.uptime_seconds = self.start_time.elapsed().as_secs();
        self.stats.clone()
    }

    /// Get current performance metrics
    pub async fn get_performance_metrics(&self) -> Result<serde_json::Value> {
        let ml_engine = self.ml_engine.lock().unwrap();
        let metrics = ml_engine.get_performance_metrics();
        
        Ok(serde_json::json!({
            "tasks_completed": metrics.tasks_completed,
            "average_execution_time_ms": metrics.get_average_execution_time().as_millis(),
            "active_tasks": self.active_tasks.len(),
            "max_concurrent_tasks": self.max_concurrent_tasks
        }))
    }

    /// Graceful shutdown
    pub async fn shutdown(&mut self) -> Result<()> {
        tracing::info!("Shutting down task manager...");
        
        // Wait for all active tasks to complete or timeout
        let timeout = Duration::from_secs(30);
        let start = Instant::now();
        
        while !self.active_tasks.is_empty() && start.elapsed() < timeout {
            self.check_completed_tasks().await?;
            tokio::time::sleep(Duration::from_millis(100)).await;
        }
        
        // Cancel remaining tasks
        for (task_id, execution) in self.active_tasks.drain() {
            tracing::warn!("Cancelling task {} due to shutdown", task_id);
            execution.handle.abort();
        }
        
        tracing::info!("Task manager shutdown complete");
        Ok(())
    }

    /// Force refresh tasks (for manual trigger)
    pub async fn refresh_tasks(&mut self) -> Result<()> {
        tracing::info!("Manually refreshing tasks...");
        self.check_completed_tasks().await?;
        self.check_for_tasks().await?;
        Ok(())
    }

    /// Get detailed task information
    pub async fn get_task_details(&self) -> Vec<serde_json::Value> {
        self.active_tasks
            .values()
            .map(|execution| {
                serde_json::json!({
                    "task_id": execution.task.id,
                    "task_type": format!("{:?}", execution.task.task_type),
                    "difficulty": execution.task.difficulty,
                    "reward": execution.task.reward,
                    "running_time_seconds": execution.start_time.elapsed().as_secs(),
                    "deadline": execution.task.deadline.to_rfc3339()
                })
            })
            .collect()
    }
}
