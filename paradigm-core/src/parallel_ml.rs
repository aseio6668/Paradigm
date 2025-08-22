use anyhow::Result;
use dashmap::DashMap;
use rayon::prelude::*;
use serde::{Deserialize, Serialize};
/// High-performance parallel ML task processing for Paradigm
use std::collections::{HashMap, VecDeque};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::{mpsc, RwLock, Semaphore};
use tokio::time::timeout;
use uuid::Uuid;

use crate::consensus::{MLTask, MLTaskType, TaskCapabilities};

/// Task priority levels for scheduling
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum TaskPriority {
    Low = 1,
    Normal = 2,
    High = 3,
    Critical = 4,
}

/// Task execution result with performance metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskExecutionResult {
    pub task_id: Uuid,
    pub success: bool,
    pub result_data: Option<Vec<u8>>,
    pub execution_time_ms: u64,
    pub memory_used_bytes: u64,
    pub cpu_usage_percent: f64,
    pub error_message: Option<String>,
    pub worker_id: String,
}

/// Worker node capabilities and current status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkerNode {
    pub id: String,
    pub capabilities: TaskCapabilities,
    pub current_load: f64,
    pub active_tasks: usize,
    pub max_concurrent_tasks: usize,
    pub total_completed: u64,
    pub total_failed: u64,
    pub average_execution_time_ms: f64,
    #[serde(skip, default = "Instant::now")]
    pub last_heartbeat: Instant,
    pub is_active: bool,
    pub gpu_available: bool,
    pub memory_available_mb: u64,
}

impl WorkerNode {
    pub fn new(id: String, capabilities: TaskCapabilities, max_concurrent: usize) -> Self {
        Self {
            id,
            capabilities,
            current_load: 0.0,
            active_tasks: 0,
            max_concurrent_tasks: max_concurrent,
            total_completed: 0,
            total_failed: 0,
            average_execution_time_ms: 0.0,
            last_heartbeat: Instant::now(),
            is_active: true,
            gpu_available: false,
            memory_available_mb: 1024,
        }
    }

    pub fn can_accept_task(&self, task: &MLTask) -> bool {
        self.is_active
            && self.active_tasks < self.max_concurrent_tasks
            && self.current_load < 0.9
            && self.capabilities.max_difficulty >= task.difficulty
            && (task.data.len() as u64) < self.memory_available_mb * 1024 * 1024
    }

    pub fn update_heartbeat(&mut self) {
        self.last_heartbeat = Instant::now();
    }

    pub fn is_healthy(&self) -> bool {
        self.is_active && self.last_heartbeat.elapsed() < Duration::from_secs(30)
    }

    pub fn success_rate(&self) -> f64 {
        if self.total_completed + self.total_failed == 0 {
            1.0
        } else {
            self.total_completed as f64 / (self.total_completed + self.total_failed) as f64
        }
    }
}

/// Priority queue for task scheduling
#[derive(Debug)]
pub struct TaskQueue {
    high_priority: VecDeque<(MLTask, TaskPriority)>,
    normal_priority: VecDeque<(MLTask, TaskPriority)>,
    low_priority: VecDeque<(MLTask, TaskPriority)>,
    task_lookup: HashMap<Uuid, TaskPriority>,
}

impl TaskQueue {
    pub fn new() -> Self {
        Self {
            high_priority: VecDeque::new(),
            normal_priority: VecDeque::new(),
            low_priority: VecDeque::new(),
            task_lookup: HashMap::new(),
        }
    }

    pub fn enqueue(&mut self, task: MLTask, priority: TaskPriority) {
        self.task_lookup.insert(task.id, priority);

        match priority {
            TaskPriority::Critical | TaskPriority::High => {
                self.high_priority.push_back((task, priority));
            }
            TaskPriority::Normal => {
                self.normal_priority.push_back((task, priority));
            }
            TaskPriority::Low => {
                self.low_priority.push_back((task, priority));
            }
        }
    }

    pub fn dequeue(&mut self) -> Option<(MLTask, TaskPriority)> {
        // Process in priority order
        if let Some(task) = self.high_priority.pop_front() {
            self.task_lookup.remove(&task.0.id);
            Some(task)
        } else if let Some(task) = self.normal_priority.pop_front() {
            self.task_lookup.remove(&task.0.id);
            Some(task)
        } else if let Some(task) = self.low_priority.pop_front() {
            self.task_lookup.remove(&task.0.id);
            Some(task)
        } else {
            None
        }
    }

    pub fn len(&self) -> usize {
        self.high_priority.len() + self.normal_priority.len() + self.low_priority.len()
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub fn remove_task(&mut self, task_id: &Uuid) -> bool {
        if let Some(priority) = self.task_lookup.remove(task_id) {
            let queue = match priority {
                TaskPriority::Critical | TaskPriority::High => &mut self.high_priority,
                TaskPriority::Normal => &mut self.normal_priority,
                TaskPriority::Low => &mut self.low_priority,
            };

            if let Some(pos) = queue.iter().position(|(task, _)| task.id == *task_id) {
                queue.remove(pos);
                return true;
            }
        }
        false
    }
}

/// Load balancer for optimal task distribution
#[derive(Debug)]
pub struct LoadBalancer {
    workers: Arc<DashMap<String, WorkerNode>>,
    task_assignments: Arc<DashMap<Uuid, String>>, // task_id -> worker_id
}

impl LoadBalancer {
    pub fn new() -> Self {
        Self {
            workers: Arc::new(DashMap::new()),
            task_assignments: Arc::new(DashMap::new()),
        }
    }

    pub fn register_worker(&self, worker: WorkerNode) {
        self.workers.insert(worker.id.clone(), worker);
    }

    pub fn unregister_worker(&self, worker_id: &str) {
        self.workers.remove(worker_id);
    }

    /// Find the best worker for a given task using weighted scoring
    pub fn find_best_worker(&self, task: &MLTask) -> Option<String> {
        let mut best_worker: Option<(String, f64)> = None;

        for worker_ref in self.workers.iter() {
            let worker = worker_ref.value();

            if !worker.can_accept_task(task) {
                continue;
            }

            let score = self.calculate_worker_score(worker, task);

            if let Some((_, best_score)) = &best_worker {
                if score > *best_score {
                    best_worker = Some((worker.id.clone(), score));
                }
            } else {
                best_worker = Some((worker.id.clone(), score));
            }
        }

        best_worker.map(|(id, _)| id)
    }

    /// Calculate worker suitability score for a task
    fn calculate_worker_score(&self, worker: &WorkerNode, task: &MLTask) -> f64 {
        let mut score = 0.0;

        // Load factor (lower load = higher score)
        score += (1.0 - worker.current_load) * 30.0;

        // Success rate
        score += worker.success_rate() * 25.0;

        // Available capacity
        let capacity_ratio = (worker.max_concurrent_tasks - worker.active_tasks) as f64
            / worker.max_concurrent_tasks as f64;
        score += capacity_ratio * 20.0;

        // Task-specific capabilities
        if worker.capabilities.max_difficulty >= task.difficulty {
            score += 15.0;
        }

        // Performance history
        if worker.average_execution_time_ms > 0.0 {
            // Prefer faster workers (inverse relationship)
            score += 10.0 / (worker.average_execution_time_ms / 1000.0).max(0.1);
        }

        // GPU availability bonus for compute-intensive tasks
        if worker.gpu_available && self.is_compute_intensive_task(&task.task_type) {
            score += 20.0;
        }

        // Health check
        if worker.is_healthy() {
            score += 10.0;
        }

        score
    }

    fn is_compute_intensive_task(&self, task_type: &MLTaskType) -> bool {
        matches!(
            task_type,
            MLTaskType::ImageClassification
                | MLTaskType::ReinforcementLearning
                | MLTaskType::DistributedTraining
        )
    }

    pub fn assign_task(&self, task_id: Uuid, worker_id: String) {
        self.task_assignments.insert(task_id, worker_id.clone());

        // Update worker load
        if let Some(mut worker) = self.workers.get_mut(&worker_id) {
            worker.active_tasks += 1;
            worker.current_load = worker.active_tasks as f64 / worker.max_concurrent_tasks as f64;
        }
    }

    pub fn complete_task(&self, task_id: Uuid, success: bool, execution_time_ms: u64) {
        if let Some((_, worker_id)) = self.task_assignments.remove(&task_id) {
            if let Some(mut worker) = self.workers.get_mut(&worker_id) {
                worker.active_tasks = worker.active_tasks.saturating_sub(1);
                worker.current_load =
                    worker.active_tasks as f64 / worker.max_concurrent_tasks as f64;

                if success {
                    worker.total_completed += 1;
                } else {
                    worker.total_failed += 1;
                }

                // Update average execution time
                let total_tasks = worker.total_completed + worker.total_failed;
                if total_tasks > 1 {
                    worker.average_execution_time_ms = (worker.average_execution_time_ms
                        * (total_tasks - 1) as f64
                        + execution_time_ms as f64)
                        / total_tasks as f64;
                } else {
                    worker.average_execution_time_ms = execution_time_ms as f64;
                }
            }
        }
    }

    pub fn get_worker_stats(&self) -> Vec<WorkerNode> {
        self.workers
            .iter()
            .map(|entry| entry.value().clone())
            .collect()
    }

    pub fn cleanup_dead_workers(&self) {
        let dead_workers: Vec<String> = self
            .workers
            .iter()
            .filter(|entry| !entry.value().is_healthy())
            .map(|entry| entry.key().clone())
            .collect();

        for worker_id in dead_workers {
            self.workers.remove(&worker_id);
        }
    }
}

/// High-performance parallel ML task processor
#[derive(Debug)]
pub struct ParallelMLProcessor {
    task_queue: Arc<RwLock<TaskQueue>>,
    load_balancer: Arc<LoadBalancer>,
    execution_semaphore: Arc<Semaphore>,
    result_sender: mpsc::UnboundedSender<TaskExecutionResult>,
    result_receiver: Arc<RwLock<mpsc::UnboundedReceiver<TaskExecutionResult>>>,
    performance_stats: Arc<RwLock<ProcessorStats>>,
    max_concurrent_tasks: usize,
    task_timeout: Duration,
}

impl ParallelMLProcessor {
    pub fn new(max_concurrent_tasks: usize, task_timeout_secs: u64) -> Self {
        let (result_sender, result_receiver) = mpsc::unbounded_channel();

        Self {
            task_queue: Arc::new(RwLock::new(TaskQueue::new())),
            load_balancer: Arc::new(LoadBalancer::new()),
            execution_semaphore: Arc::new(Semaphore::new(max_concurrent_tasks)),
            result_sender,
            result_receiver: Arc::new(RwLock::new(result_receiver)),
            performance_stats: Arc::new(RwLock::new(ProcessorStats::default())),
            max_concurrent_tasks,
            task_timeout: Duration::from_secs(task_timeout_secs),
        }
    }

    /// Submit a task for processing
    pub async fn submit_task(&self, task: MLTask, priority: TaskPriority) -> Result<()> {
        let mut queue = self.task_queue.write().await;
        queue.enqueue(task, priority);

        // Update stats
        let mut stats = self.performance_stats.write().await;
        stats.total_tasks_submitted += 1;
        stats.queue_size = queue.len();

        Ok(())
    }

    /// Register a worker node
    pub fn register_worker(&self, worker: WorkerNode) {
        self.load_balancer.register_worker(worker);
    }

    /// Start the task processing loop
    pub async fn start_processing(&self) -> Result<()> {
        let queue = self.task_queue.clone();
        let load_balancer = self.load_balancer.clone();
        let semaphore = self.execution_semaphore.clone();
        let result_sender = self.result_sender.clone();
        let stats = self.performance_stats.clone();
        let timeout_duration = self.task_timeout;

        tokio::spawn(async move {
            loop {
                // Check for available tasks
                let task_info = {
                    let mut queue_guard = queue.write().await;
                    queue_guard.dequeue()
                };

                if let Some((task, priority)) = task_info {
                    // Find best worker
                    if let Some(worker_id) = load_balancer.find_best_worker(&task) {
                        // Acquire semaphore permit
                        let permit = semaphore.clone().acquire_owned().await.unwrap();

                        // Assign task to worker
                        load_balancer.assign_task(task.id, worker_id.clone());

                        let task_clone = task.clone();
                        let worker_id_clone = worker_id.clone();
                        let load_balancer_clone = load_balancer.clone();
                        let result_sender_clone = result_sender.clone();
                        let stats_clone = stats.clone();

                        // Execute task in background
                        tokio::spawn(async move {
                            let start_time = Instant::now();

                            let result = timeout(
                                timeout_duration,
                                Self::execute_task_on_worker(
                                    task_clone.clone(),
                                    worker_id_clone.clone(),
                                ),
                            )
                            .await;

                            let execution_time_ms = start_time.elapsed().as_millis() as u64;

                            let task_result = match result {
                                Ok(Ok(result_data)) => TaskExecutionResult {
                                    task_id: task_clone.id,
                                    success: true,
                                    result_data: Some(result_data),
                                    execution_time_ms,
                                    memory_used_bytes: task_clone.data.len() as u64,
                                    cpu_usage_percent: 0.0, // Would be measured in real implementation
                                    error_message: None,
                                    worker_id: worker_id_clone.clone(),
                                },
                                Ok(Err(e)) => TaskExecutionResult {
                                    task_id: task_clone.id,
                                    success: false,
                                    result_data: None,
                                    execution_time_ms,
                                    memory_used_bytes: task_clone.data.len() as u64,
                                    cpu_usage_percent: 0.0,
                                    error_message: Some(e.to_string()),
                                    worker_id: worker_id_clone.clone(),
                                },
                                Err(_) => TaskExecutionResult {
                                    task_id: task_clone.id,
                                    success: false,
                                    result_data: None,
                                    execution_time_ms,
                                    memory_used_bytes: task_clone.data.len() as u64,
                                    cpu_usage_percent: 0.0,
                                    error_message: Some("Task timeout".to_string()),
                                    worker_id: worker_id_clone.clone(),
                                },
                            };

                            // Update load balancer
                            load_balancer_clone.complete_task(
                                task_clone.id,
                                task_result.success,
                                execution_time_ms,
                            );

                            // Update stats
                            {
                                let mut stats_guard = stats_clone.write().await;
                                if task_result.success {
                                    stats_guard.total_tasks_completed += 1;
                                } else {
                                    stats_guard.total_tasks_failed += 1;
                                }
                                stats_guard.average_execution_time_ms = (stats_guard
                                    .average_execution_time_ms
                                    * stats_guard.total_tasks_completed as f64
                                    + execution_time_ms as f64)
                                    / (stats_guard.total_tasks_completed + 1) as f64;
                            }

                            // Send result
                            let _ = result_sender_clone.send(task_result);

                            // Release permit
                            drop(permit);
                        });
                    } else {
                        // No available workers, put task back in queue
                        let mut queue_guard = queue.write().await;
                        queue_guard.enqueue(task, priority);
                    }
                } else {
                    // No tasks available, sleep briefly
                    tokio::time::sleep(Duration::from_millis(100)).await;
                }

                // Cleanup dead workers periodically
                load_balancer.cleanup_dead_workers();
            }
        });

        Ok(())
    }

    /// Execute a task on a specific worker (simulated)
    async fn execute_task_on_worker(task: MLTask, worker_id: String) -> Result<Vec<u8>> {
        // This is a simplified simulation of task execution
        // In a real implementation, this would communicate with actual worker nodes

        // Simulate processing time based on task difficulty and data size
        let processing_time =
            Duration::from_millis((task.difficulty as u64 * 100) + (task.data.len() as u64 / 1000));

        tokio::time::sleep(processing_time).await;

        // Simulate different task types
        match task.task_type {
            MLTaskType::ImageClassification => {
                // Simulate image classification result
                Ok(vec![0, 1, 0, 1]) // Dummy classification result
            }
            MLTaskType::NaturalLanguageProcessing => {
                // Simulate NLP processing
                Ok(format!("Processed text: {}", String::from_utf8_lossy(&task.data)).into_bytes())
            }
            MLTaskType::TimeSeriesAnalysis => {
                // Simulate time series analysis
                Ok(vec![1, 2, 3, 4, 5]) // Dummy predictions
            }
            MLTaskType::ReinforcementLearning => {
                // Simulate RL training step
                Ok(vec![0xFF, 0x00, 0xFF, 0x00]) // Dummy policy update
            }
            MLTaskType::AutoML => {
                // Simulate AutoML pipeline
                Ok(b"optimal_model_config".to_vec())
            }
            MLTaskType::DistributedTraining => {
                // Simulate distributed training
                Ok(vec![0xAA, 0xBB, 0xCC, 0xDD]) // Dummy gradient update
            }
            MLTaskType::Oracle => {
                // Simulate oracle query
                Ok(b"oracle_response".to_vec())
            }
            MLTaskType::SmartContractOptimization => {
                // Simulate smart contract optimization
                Ok(b"optimized_contract_bytecode".to_vec())
            }
            MLTaskType::NetworkOptimization => {
                // Simulate network optimization
                Ok(vec![0x01, 0x02, 0x03]) // Dummy optimization parameters
            }
        }
    }

    /// Get the next completed task result
    pub async fn get_result(&self) -> Option<TaskExecutionResult> {
        let mut receiver = self.result_receiver.write().await;
        receiver.recv().await
    }

    /// Get performance statistics
    pub async fn get_stats(&self) -> ProcessorStats {
        self.performance_stats.read().await.clone()
    }

    /// Get current queue size
    pub async fn get_queue_size(&self) -> usize {
        self.task_queue.read().await.len()
    }

    /// Get worker statistics
    pub fn get_worker_stats(&self) -> Vec<WorkerNode> {
        self.load_balancer.get_worker_stats()
    }

    /// Cancel a pending task
    pub async fn cancel_task(&self, task_id: Uuid) -> bool {
        let mut queue = self.task_queue.write().await;
        queue.remove_task(&task_id)
    }

    /// Batch process multiple tasks with optimal parallelization
    pub async fn batch_process_tasks(
        &self,
        tasks: Vec<MLTask>,
        priority: TaskPriority,
    ) -> Result<Vec<TaskExecutionResult>> {
        let task_count = tasks.len();

        // Submit all tasks
        for task in tasks {
            self.submit_task(task, priority).await?;
        }

        // Collect results (in a real implementation, you'd want better coordination)
        let mut results = Vec::new();
        let start_time = Instant::now();
        let batch_timeout = Duration::from_secs(300); // 5 minutes for batch processing

        while start_time.elapsed() < batch_timeout && results.len() < task_count {
            if let Some(result) = self.get_result().await {
                results.push(result);
            } else {
                tokio::time::sleep(Duration::from_millis(100)).await;
            }
        }

        Ok(results)
    }
}

/// Performance statistics for the processor
#[derive(Debug, Clone, Default)]
pub struct ProcessorStats {
    pub total_tasks_submitted: u64,
    pub total_tasks_completed: u64,
    pub total_tasks_failed: u64,
    pub queue_size: usize,
    pub active_workers: usize,
    pub average_execution_time_ms: f64,
    pub throughput_tasks_per_sec: f64,
    pub total_processing_time_ms: u64,
}

impl ProcessorStats {
    pub fn success_rate(&self) -> f64 {
        if self.total_tasks_completed + self.total_tasks_failed == 0 {
            0.0
        } else {
            self.total_tasks_completed as f64
                / (self.total_tasks_completed + self.total_tasks_failed) as f64
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::consensus::MLTaskType;
    use chrono::Utc;

    #[tokio::test]
    async fn test_task_queue() {
        let mut queue = TaskQueue::new();

        let task1 = create_test_task(MLTaskType::ImageClassification, 5);
        let task2 = create_test_task(MLTaskType::NaturalLanguageProcessing, 3);

        queue.enqueue(task1.clone(), TaskPriority::Low);
        queue.enqueue(task2.clone(), TaskPriority::High);

        // High priority task should come first
        let (dequeued, priority) = queue.dequeue().unwrap();
        assert_eq!(dequeued.id, task2.id);
        assert_eq!(priority, TaskPriority::High);

        // Then low priority task
        let (dequeued, priority) = queue.dequeue().unwrap();
        assert_eq!(dequeued.id, task1.id);
        assert_eq!(priority, TaskPriority::Low);
    }

    #[tokio::test]
    async fn test_load_balancer() {
        let balancer = LoadBalancer::new();

        let worker1 = WorkerNode::new(
            "worker1".to_string(),
            TaskCapabilities {
                max_difficulty: 10,
                estimated_time_per_unit: Duration::from_millis(100),
                memory_requirement: 1024 * 1024,
                gpu_required: false,
            },
            4,
        );

        let worker2 = WorkerNode::new(
            "worker2".to_string(),
            TaskCapabilities {
                max_difficulty: 5,
                estimated_time_per_unit: Duration::from_millis(200),
                memory_requirement: 512 * 1024,
                gpu_required: false,
            },
            2,
        );

        balancer.register_worker(worker1);
        balancer.register_worker(worker2);

        let task = create_test_task(MLTaskType::ImageClassification, 7);

        // Should select worker1 as it has higher max_difficulty
        let selected = balancer.find_best_worker(&task);
        assert_eq!(selected, Some("worker1".to_string()));
    }

    #[tokio::test]
    async fn test_parallel_processor() {
        let processor = ParallelMLProcessor::new(4, 30);

        // Register a test worker
        let worker = WorkerNode::new(
            "test_worker".to_string(),
            TaskCapabilities {
                max_difficulty: 10,
                estimated_time_per_unit: Duration::from_millis(50),
                memory_requirement: 1024 * 1024,
                gpu_required: false,
            },
            2,
        );

        processor.register_worker(worker);

        // Start processing
        processor.start_processing().await.unwrap();

        // Submit test tasks
        let task1 = create_test_task(MLTaskType::ImageClassification, 3);
        let task2 = create_test_task(MLTaskType::NaturalLanguageProcessing, 2);

        processor
            .submit_task(task1.clone(), TaskPriority::Normal)
            .await
            .unwrap();
        processor
            .submit_task(task2.clone(), TaskPriority::High)
            .await
            .unwrap();

        // Wait for results
        tokio::time::sleep(Duration::from_millis(500)).await;

        let stats = processor.get_stats().await;
        assert!(stats.total_tasks_submitted >= 2);
    }

    fn create_test_task(task_type: MLTaskType, difficulty: u8) -> MLTask {
        MLTask {
            id: Uuid::new_v4(),
            task_type,
            data: vec![0u8; 1024], // 1KB test data
            difficulty,
            reward: 100,
            deadline: Utc::now() + chrono::Duration::minutes(10),
            created_at: Utc::now(),
            assigned_to: None,
            completed: false,
            result: None,
        }
    }
}
