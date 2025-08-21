// Simplified Performance Monitor for Paradigm Contributor
use std::time::{Duration, Instant};
use std::collections::VecDeque;
use tracing::info;

#[derive(Debug, Clone)]
pub struct PerformanceStats {
    pub total_tasks: usize,
    pub average_time: Duration,
    pub tasks_per_minute: f64,
    pub uptime: Duration,
}

pub struct PerformanceMonitor {
    start_time: Instant,
    task_completion_times: VecDeque<Duration>,
    max_history: usize,
    last_task_count: usize,
    last_check_time: Instant,
}

impl PerformanceMonitor {
    pub fn new() -> Self {
        let now = Instant::now();
        Self {
            start_time: now,
            task_completion_times: VecDeque::new(),
            max_history: 100,
            last_task_count: 0,
            last_check_time: now,
        }
    }

    pub async fn update_metrics(&mut self) {
        // This would be called when tasks complete
        // For now, we'll simulate some metrics
        
        // Simulate task completion
        let simulated_time = Duration::from_millis(1000 + (rand::random::<u64>() % 3000));
        self.record_task_completion(simulated_time);
    }

    pub fn record_task_completion(&mut self, completion_time: Duration) {
        self.task_completion_times.push_back(completion_time);
        
        // Keep only recent history
        if self.task_completion_times.len() > self.max_history {
            self.task_completion_times.pop_front();
        }
    }

    pub fn get_stats(&self) -> PerformanceStats {
        let total_tasks = self.task_completion_times.len();
        
        let average_time = if total_tasks > 0 {
            let total_time: Duration = self.task_completion_times.iter().sum();
            total_time / total_tasks as u32
        } else {
            Duration::from_secs(0)
        };

        let uptime = self.start_time.elapsed();
        let tasks_per_minute = if uptime.as_secs() > 0 {
            (total_tasks as f64 * 60.0) / uptime.as_secs() as f64
        } else {
            0.0
        };

        PerformanceStats {
            total_tasks,
            average_time,
            tasks_per_minute,
            uptime,
        }
    }

    pub async fn log_performance(&self) {
        let stats = self.get_stats();
        info!(
            "Performance - Tasks: {}, Avg Time: {:?}, Rate: {:.2}/min, Uptime: {:?}",
            stats.total_tasks,
            stats.average_time,
            stats.tasks_per_minute,
            stats.uptime
        );
    }
}
