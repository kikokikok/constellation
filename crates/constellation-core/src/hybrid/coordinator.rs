//! LLM strategist coordination layer.
//!
//! Implements coordination between LLM strategists (large language models for planning)
//! and SLM executors (smaller, specialized models for execution).

use crate::models::hybrid_agent::{
    FallbackAction, FallbackStrategy, FallbackTrigger, HybridAgentConfig,
};
use dashmap::DashMap;
use serde_json::Value;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use uuid::Uuid;

/// Task status.
#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub enum TaskStatus {
    Pending,
    Assigned,
    InProgress,
    Completed,
    Failed,
    Cancelled,
}

/// Task assignment.
#[derive(Debug, Clone)]
pub struct TaskAssignment {
    /// Task ID.
    pub task_id: Uuid,

    /// Executor ID.
    pub executor_id: String,

    /// Assignment time.
    pub assigned_at: chrono::DateTime<chrono::Utc>,

    /// Priority.
    pub priority: u32,

    /// Estimated completion time.
    pub estimated_completion: chrono::DateTime<chrono::Utc>,

    /// Resource allocation.
    pub resource_allocation: ResourceAllocation,
}

/// Resource allocation for a task.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ResourceAllocation {
    /// CPU cores.
    pub cpu_cores: u32,

    /// Memory in MB.
    pub memory_mb: u32,

    /// GPU memory in MB.
    pub gpu_memory_mb: Option<u32>,

    /// Budget allocation.
    pub budget_allocation: f64,
}

/// Task result.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct TaskResult {
    /// Task ID.
    pub task_id: Uuid,

    /// Executor ID.
    pub executor_id: String,

    /// Completion time.
    pub completed_at: chrono::DateTime<chrono::Utc>,

    /// Result data.
    pub result: Value,

    /// Success flag.
    pub success: bool,

    /// Error message if failed.
    pub error: Option<String>,

    /// Quality score (0.0 to 1.0).
    pub quality_score: f64,

    /// Execution time in milliseconds.
    pub execution_time_ms: u64,

    /// Resource usage.
    pub resource_usage: ResourceUsage,

    /// Cost incurred.
    pub cost: f64,
}

/// Resource usage for a task.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ResourceUsage {
    /// CPU usage in core-seconds.
    pub cpu_core_seconds: f64,

    /// Memory usage in MB-seconds.
    pub memory_mb_seconds: f64,

    /// GPU memory usage in MB-seconds.
    pub gpu_memory_mb_seconds: Option<f64>,

    /// Network usage in MB.
    pub network_mb: f64,
}

impl Default for ResourceUsage {
    fn default() -> Self {
        Self {
            cpu_core_seconds: 0.0,
            memory_mb_seconds: 0.0,
            gpu_memory_mb_seconds: None,
            network_mb: 0.0,
        }
    }
}

/// Performance metrics.
#[derive(Debug, Clone)]
pub struct PerformanceMetrics {
    /// Throughput in tasks per second.
    pub throughput_tps: f64,

    /// Average latency in milliseconds.
    pub avg_latency_ms: f64,

    /// Success rate (0.0 to 1.0).
    pub success_rate: f64,

    /// Average quality score (0.0 to 1.0).
    pub avg_quality_score: f64,

    /// Resource utilization (0.0 to 1.0).
    pub resource_utilization: f64,

    /// Cost efficiency (0.0 to 1.0).
    pub cost_efficiency: f64,

    /// Availability (0.0 to 1.0).
    pub availability: f64,
}

/// LLM strategist coordinator.
#[derive(Debug)]
pub struct LlmStrategistCoordinator {
    /// Agent configuration.
    config: HybridAgentConfig,

    /// Task queue.
    task_queue: Arc<Mutex<Vec<Task>>>,

    /// Active tasks.
    active_tasks: DashMap<Uuid, TaskAssignment>,

    /// Completed tasks.
    completed_tasks: Arc<Mutex<Vec<TaskResult>>>,

    /// Performance metrics.
    performance_metrics: Arc<Mutex<PerformanceMetrics>>,

    /// Executor status.
    executor_status: DashMap<String, ExecutorStatus>,

    /// Fallback strategies.
    fallback_strategies: Vec<FallbackStrategy>,

    /// Total budget spent.
    total_budget_spent: Arc<Mutex<f64>>,

    /// Total tasks processed.
    total_tasks_processed: Arc<Mutex<u64>>,
}

/// Task definition.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Task {
    /// Task ID.
    pub id: Uuid,

    /// Task type.
    pub task_type: String,

    /// Input data.
    pub input: Value,

    /// Expected output (optional).
    pub expected_output: Option<Value>,

    /// Assigned agent (optional).
    pub assigned_to: Option<String>,

    /// Priority.
    pub priority: u32,

    /// Timeout in milliseconds.
    pub timeout_ms: u32,

    /// Created time.
    pub created_at: chrono::DateTime<chrono::Utc>,

    /// Task status.
    pub status: TaskStatus,

    /// Task metadata.
    pub metadata: HashMap<String, Value>,

    /// Deadline.
    pub deadline: Option<chrono::DateTime<chrono::Utc>>,

    /// Quality requirements.
    pub quality_requirement: f64,

    /// Budget allocation.
    pub budget_allocation: f64,

    /// Resource requirements.
    pub resource_requirements: ResourceRequirements,
}

/// Resource requirements for a task.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ResourceRequirements {
    /// Minimum CPU cores.
    pub min_cpu_cores: u32,

    /// Minimum memory in MB.
    pub min_memory_mb: u32,

    /// GPU memory requirement in MB.
    pub gpu_memory_mb: Option<u32>,

    /// Network bandwidth in Mbps.
    pub network_mbps: u32,
}

impl Default for ResourceRequirements {
    fn default() -> Self {
        Self {
            min_cpu_cores: 1,
            min_memory_mb: 1024,
            gpu_memory_mb: None,
            network_mbps: 10,
        }
    }
}

/// Executor status.
#[derive(Debug, Clone)]
pub struct ExecutorStatus {
    /// Executor ID.
    pub executor_id: String,

    /// Current load (0.0 to 1.0).
    pub current_load: f64,

    /// Available capacity.
    pub available_capacity: u32,

    /// Average latency in milliseconds.
    pub avg_latency_ms: f64,

    /// Success rate (0.0 to 1.0).
    pub success_rate: f64,

    /// Quality score (0.0 to 1.0).
    pub quality_score: f64,

    /// Cost per task.
    pub cost_per_task: f64,

    /// Last heartbeat.
    pub last_heartbeat: chrono::DateTime<chrono::Utc>,

    /// Is available.
    pub is_available: bool,
}

impl LlmStrategistCoordinator {
    /// Create a new coordinator.
    pub fn new(config: HybridAgentConfig) -> Self {
        let performance_metrics = PerformanceMetrics {
            throughput_tps: 0.0,
            avg_latency_ms: 0.0,
            success_rate: 0.0,
            avg_quality_score: 0.0,
            resource_utilization: 0.0,
            cost_efficiency: 0.0,
            availability: 1.0,
        };

        Self {
            config,
            task_queue: Arc::new(Mutex::new(Vec::new())),
            active_tasks: DashMap::new(),
            completed_tasks: Arc::new(Mutex::new(Vec::new())),
            performance_metrics: Arc::new(Mutex::new(performance_metrics)),
            executor_status: DashMap::new(),
            fallback_strategies: Vec::new(),
            total_budget_spent: Arc::new(Mutex::new(0.0)),
            total_tasks_processed: Arc::new(Mutex::new(0)),
        }
    }

    /// Initialize the coordinator with fallback strategies.
    pub fn with_fallback_strategies(mut self, strategies: Vec<FallbackStrategy>) -> Self {
        self.fallback_strategies = strategies;
        self
    }

    /// Submit a task for execution.
    pub fn submit_task(&self, task: Task) -> Result<Uuid, String> {
        let mut queue = self.task_queue.lock().unwrap();
        queue.push(task.clone());

        // Sort by priority (higher priority first)
        queue.sort_by(|a, b| b.priority.cmp(&a.priority));

        Ok(task.id)
    }

    /// Assign tasks to executors.
    pub fn assign_tasks(&self) -> Result<Vec<TaskAssignment>, String> {
        let mut queue = self.task_queue.lock().unwrap();
        let mut assignments = Vec::new();

        // Get available executors
        let available_executors: Vec<ExecutorStatus> = self
            .executor_status
            .iter()
            .map(|entry| entry.value().clone())
            .collect();

        if available_executors.is_empty() {
            return Ok(assignments);
        }

        // Assign tasks based on strategy
        match self.config.coordination.strategy_type {
            crate::models::hybrid_agent::CoordinationStrategyType::Hierarchical => {
                assignments = self.assign_hierarchical(&mut queue, &available_executors)?;
            }
            crate::models::hybrid_agent::CoordinationStrategyType::Collaborative => {
                assignments = self.assign_collaborative(&mut queue, &available_executors)?;
            }
            crate::models::hybrid_agent::CoordinationStrategyType::MarketBased => {
                assignments = self.assign_market_based(&mut queue, &available_executors)?;
            }
            _ => {
                // Default to hierarchical
                assignments = self.assign_hierarchical(&mut queue, &available_executors)?;
            }
        }

        Ok(assignments)
    }

    /// Assign tasks using hierarchical strategy.
    fn assign_hierarchical(
        &self,
        queue: &mut Vec<Task>,
        available_executors: &[ExecutorStatus],
    ) -> Result<Vec<TaskAssignment>, String> {
        let mut assignments = Vec::new();
        let now = chrono::Utc::now();

        // Simple round-robin assignment
        let mut executor_index = 0;

        for task in queue.iter_mut() {
            if executor_index >= available_executors.len() {
                break;
            }

            let executor = &available_executors[executor_index];
            let assignment = TaskAssignment {
                task_id: task.id,
                executor_id: executor.executor_id.clone(),
                assigned_at: now,
                priority: task.priority,
                estimated_completion: now
                    + chrono::Duration::milliseconds(executor.avg_latency_ms as i64),
                resource_allocation: ResourceAllocation {
                    cpu_cores: 1,
                    memory_mb: 1024,
                    gpu_memory_mb: None,
                    budget_allocation: task.budget_allocation,
                },
            };

            assignments.push(assignment.clone());
            self.active_tasks.insert(task.id, assignment);

            // Remove from queue
            // We'll mark it for removal and collect indices
            executor_index = (executor_index + 1) % available_executors.len();
        }

        // Remove assigned tasks from queue
        let assigned_ids: Vec<Uuid> = assignments.iter().map(|a| a.task_id).collect();
        queue.retain(|task| !assigned_ids.contains(&task.id));

        Ok(assignments)
    }

    /// Assign tasks using collaborative strategy.
    fn assign_collaborative(
        &self,
        queue: &mut Vec<Task>,
        available_executors: &[ExecutorStatus],
    ) -> Result<Vec<TaskAssignment>, String> {
        let mut assignments = Vec::new();
        let now = chrono::Utc::now();

        // Assign based on executor capabilities and load
        for task in queue.iter_mut() {
            // Find best executor for this task
            let best_executor = available_executors
                .iter()
                .filter(|executor| {
                    // Check if executor has capacity
                    executor.available_capacity > 0
                })
                .min_by(|a, b| {
                    // Compare by load, then by success rate, then by cost
                    a.current_load
                        .partial_cmp(&b.current_load)
                        .unwrap()
                        .then(b.success_rate.partial_cmp(&a.success_rate).unwrap())
                        .then(a.cost_per_task.partial_cmp(&b.cost_per_task).unwrap())
                });

            if let Some(executor) = best_executor {
                let assignment = TaskAssignment {
                    task_id: task.id,
                    executor_id: executor.executor_id.clone(),
                    assigned_at: now,
                    priority: task.priority,
                    estimated_completion: now
                        + chrono::Duration::milliseconds(executor.avg_latency_ms as i64),
                    resource_allocation: ResourceAllocation {
                        cpu_cores: 1,
                        memory_mb: 1024,
                        gpu_memory_mb: None,
                        budget_allocation: task.budget_allocation,
                    },
                };

                assignments.push(assignment.clone());
                self.active_tasks.insert(task.id, assignment);
            }
        }

        // Remove assigned tasks from queue
        let assigned_ids: Vec<Uuid> = assignments.iter().map(|a| a.task_id).collect();
        queue.retain(|task| !assigned_ids.contains(&task.id));

        Ok(assignments)
    }

    /// Assign tasks using market-based strategy.
    fn assign_market_based(
        &self,
        queue: &mut Vec<Task>,
        available_executors: &[ExecutorStatus],
    ) -> Result<Vec<TaskAssignment>, String> {
        let mut assignments = Vec::new();
        let now = chrono::Utc::now();

        // Simple auction-based assignment
        for task in queue.iter_mut() {
            // Find executor with best "bid" (lowest cost with sufficient quality)
            let best_executor = available_executors
                .iter()
                .filter(|executor| {
                    executor.available_capacity > 0
                        && executor.quality_score >= task.quality_requirement
                })
                .min_by(|a, b| {
                    // Compare by cost, then by success rate
                    a.cost_per_task
                        .partial_cmp(&b.cost_per_task)
                        .unwrap()
                        .then(b.success_rate.partial_cmp(&a.success_rate).unwrap())
                });

            if let Some(executor) = best_executor {
                let assignment = TaskAssignment {
                    task_id: task.id,
                    executor_id: executor.executor_id.clone(),
                    assigned_at: now,
                    priority: task.priority,
                    estimated_completion: now
                        + chrono::Duration::milliseconds(executor.avg_latency_ms as i64),
                    resource_allocation: ResourceAllocation {
                        cpu_cores: 1,
                        memory_mb: 1024,
                        gpu_memory_mb: None,
                        budget_allocation: task.budget_allocation,
                    },
                };

                assignments.push(assignment.clone());
                self.active_tasks.insert(task.id, assignment);
            }
        }

        // Remove assigned tasks from queue
        let assigned_ids: Vec<Uuid> = assignments.iter().map(|a| a.task_id).collect();
        queue.retain(|task| !assigned_ids.contains(&task.id));

        Ok(assignments)
    }

    /// Update executor status.
    pub fn update_executor_status(&self, status: ExecutorStatus) -> Result<(), String> {
        self.executor_status
            .insert(status.executor_id.clone(), status);
        Ok(())
    }

    /// Complete a task.
    pub fn complete_task(&self, result: TaskResult) -> Result<(), String> {
        let mut completed_tasks = self.completed_tasks.lock().unwrap();
        let mut total_budget_spent = self.total_budget_spent.lock().unwrap();
        let mut total_tasks_processed = self.total_tasks_processed.lock().unwrap();
        let mut performance_metrics = self.performance_metrics.lock().unwrap();

        // Remove from active tasks
        self.active_tasks.remove(&result.task_id);

        // Add to completed tasks
        completed_tasks.push(result.clone());

        // Update budget and task count
        *total_budget_spent += result.cost;
        *total_tasks_processed += 1;

        // Update performance metrics
        self.update_performance_metrics(&mut performance_metrics, &result, &completed_tasks);

        Ok(())
    }

    /// Update performance metrics.
    fn update_performance_metrics(
        &self,
        metrics: &mut PerformanceMetrics,
        _result: &TaskResult,
        completed_tasks: &[TaskResult],
    ) {
        let total_tasks = completed_tasks.len() as f64;

        if total_tasks == 0.0 {
            return;
        }

        // Calculate new metrics
        let total_latency: f64 = completed_tasks
            .iter()
            .map(|r| r.execution_time_ms as f64)
            .sum();
        let total_success: f64 = completed_tasks
            .iter()
            .map(|r| if r.success { 1.0 } else { 0.0 })
            .sum();
        let total_quality: f64 = completed_tasks.iter().map(|r| r.quality_score).sum();

        metrics.avg_latency_ms = total_latency / total_tasks;
        metrics.success_rate = total_success / total_tasks;
        metrics.avg_quality_score = total_quality / total_tasks;

        // Simple throughput calculation (tasks per second over last minute)
        let now = chrono::Utc::now();
        let one_minute_ago = now - chrono::Duration::minutes(1);
        let recent_tasks: f64 = completed_tasks
            .iter()
            .filter(|r| r.completed_at > one_minute_ago)
            .count() as f64;

        metrics.throughput_tps = recent_tasks / 60.0;

        // Update other metrics (simplified)
        metrics.resource_utilization = 0.7; // Placeholder
        metrics.cost_efficiency = 0.8; // Placeholder
        metrics.availability = 0.99; // Placeholder
    }

    /// Get performance metrics.
    pub fn get_performance_metrics(&self) -> PerformanceMetrics {
        let metrics = self.performance_metrics.lock().unwrap();
        metrics.clone()
    }

    /// Check for fallback conditions.
    pub fn check_fallback_conditions(&self) -> Vec<FallbackAction> {
        let metrics = self.performance_metrics.lock().unwrap();
        let mut actions = Vec::new();

        for strategy in &self.fallback_strategies {
            let trigger = match strategy.trigger {
                FallbackTrigger::HighLatency => {
                    metrics.avg_latency_ms
                        > self.config.performance_targets.latency_target_ms as f64
                }
                FallbackTrigger::LowSuccessRate => {
                    metrics.success_rate < self.config.performance_targets.success_rate_target
                }
                FallbackTrigger::HighErrorRate => {
                    1.0 - metrics.success_rate > 0.1 // More than 10% error rate
                }
                FallbackTrigger::ResourceExhaustion => {
                    metrics.resource_utilization > 0.9 // More than 90% utilization
                }
                FallbackTrigger::BudgetExceeded => {
                    let budget_spent = *self.total_budget_spent.lock().unwrap();
                    let total_budget = self
                        .config
                        .resource_allocation
                        .budget_allocation
                        .total_budget;
                    budget_spent > total_budget * 0.8 // More than 80% of budget spent
                }
                FallbackTrigger::QualityBelowThreshold => {
                    metrics.avg_quality_score < self.config.performance_targets.quality_score_target
                }
                FallbackTrigger::AvailabilityBelowThreshold => {
                    metrics.availability < self.config.performance_targets.availability_target
                }
                FallbackTrigger::Timeout => {
                    // Check for timed out tasks
                    let now = chrono::Utc::now();
                    self.active_tasks
                        .iter()
                        .any(|entry| now > entry.estimated_completion)
                }
            };

            if trigger {
                actions.push(strategy.action.clone());
            }
        }

        actions
    }

    /// Get queue statistics.
    pub fn get_queue_stats(&self) -> QueueStats {
        let queue = self.task_queue.lock().unwrap();
        let completed_tasks = self.completed_tasks.lock().unwrap();

        QueueStats {
            pending_tasks: queue.len(),
            active_tasks: self.active_tasks.len(),
            completed_tasks: completed_tasks.len(),
            total_tasks_processed: *self.total_tasks_processed.lock().unwrap(),
            total_budget_spent: *self.total_budget_spent.lock().unwrap(),
        }
    }

    /// Get executor statistics.
    pub fn get_executor_stats(&self) -> Vec<ExecutorStats> {
        self.executor_status
            .iter()
            .map(|entry| {
                let status = entry.value();
                ExecutorStats {
                    executor_id: entry.key().clone(),
                    current_load: status.current_load,
                    available_capacity: status.available_capacity,
                    success_rate: status.success_rate,
                    quality_score: status.quality_score,
                    cost_per_task: status.cost_per_task,
                    is_available: status.is_available,
                }
            })
            .collect()
    }
}

/// Queue statistics.
#[derive(Debug, Clone)]
pub struct QueueStats {
    /// Number of pending tasks.
    pub pending_tasks: usize,

    /// Number of active tasks.
    pub active_tasks: usize,

    /// Number of completed tasks.
    pub completed_tasks: usize,

    /// Total tasks processed.
    pub total_tasks_processed: u64,

    /// Total budget spent.
    pub total_budget_spent: f64,
}

/// Executor statistics.
#[derive(Debug, Clone)]
pub struct ExecutorStats {
    /// Executor ID.
    pub executor_id: String,

    /// Current load (0.0 to 1.0).
    pub current_load: f64,

    /// Available capacity.
    pub available_capacity: u32,

    /// Success rate (0.0 to 1.0).
    pub success_rate: f64,

    /// Quality score (0.0 to 1.0).
    pub quality_score: f64,

    /// Cost per task.
    pub cost_per_task: f64,

    /// Is available.
    pub is_available: bool,
}

impl Default for Task {
    fn default() -> Self {
        Self {
            id: Uuid::new_v4(),
            task_type: "default".to_string(),
            input: Value::Null,
            expected_output: None,
            assigned_to: None,
            priority: 50,
            timeout_ms: 30000,
            created_at: chrono::Utc::now(),
            status: TaskStatus::Pending,
            metadata: HashMap::new(),
            deadline: None,
            quality_requirement: 0.8,
            budget_allocation: 1.0,
            resource_requirements: ResourceRequirements {
                min_cpu_cores: 1,
                min_memory_mb: 1024,
                gpu_memory_mb: None,
                network_mbps: 10,
            },
        }
    }
}

impl Task {
    /// Create a new task.
    pub fn new(task_type: String, input: Value) -> Self {
        Self {
            id: Uuid::new_v4(),
            task_type,
            input,
            ..Default::default()
        }
    }

    /// Set priority.
    pub fn with_priority(mut self, priority: u32) -> Self {
        self.priority = priority;
        self
    }

    /// Set deadline.
    pub fn with_deadline(mut self, deadline: chrono::DateTime<chrono::Utc>) -> Self {
        self.deadline = Some(deadline);
        self
    }

    /// Set quality requirement.
    pub fn with_quality_requirement(mut self, quality: f64) -> Self {
        self.quality_requirement = quality;
        self
    }

    /// Set budget allocation.
    pub fn with_budget_allocation(mut self, budget: f64) -> Self {
        self.budget_allocation = budget;
        self
    }
}

impl Default for ExecutorStatus {
    fn default() -> Self {
        Self {
            executor_id: "default".to_string(),
            current_load: 0.0,
            available_capacity: 1,
            avg_latency_ms: 1000.0,
            success_rate: 0.95,
            quality_score: 0.9,
            cost_per_task: 0.1,
            last_heartbeat: chrono::Utc::now(),
            is_available: true,
        }
    }
}

impl ExecutorStatus {
    /// Create a new executor status.
    pub fn new(executor_id: String) -> Self {
        Self {
            executor_id,
            ..Default::default()
        }
    }

    /// Update with current load.
    pub fn with_load(mut self, load: f64) -> Self {
        self.current_load = load.clamp(0.0, 1.0);
        self.available_capacity = if load < 0.8 { 1 } else { 0 };
        self
    }

    /// Update with performance metrics.
    pub fn with_performance(
        mut self,
        success_rate: f64,
        quality_score: f64,
        avg_latency_ms: f64,
    ) -> Self {
        self.success_rate = success_rate.clamp(0.0, 1.0);
        self.quality_score = quality_score.clamp(0.0, 1.0);
        self.avg_latency_ms = avg_latency_ms.max(0.0);
        self
    }

    /// Update with cost.
    pub fn with_cost(mut self, cost_per_task: f64) -> Self {
        self.cost_per_task = cost_per_task.max(0.0);
        self
    }

    /// Mark as available/unavailable.
    pub fn with_availability(mut self, available: bool) -> Self {
        self.is_available = available;
        self
    }
}

impl Default for ExecutorStats {
    fn default() -> Self {
        Self {
            executor_id: String::new(),
            current_load: 0.0,
            available_capacity: 0,
            success_rate: 0.0,
            quality_score: 0.0,
            cost_per_task: 0.0,
            is_available: false,
        }
    }
}

impl Default for PerformanceMetrics {
    fn default() -> Self {
        Self {
            throughput_tps: 0.0,
            avg_latency_ms: 0.0,
            success_rate: 0.0,
            avg_quality_score: 0.0,
            resource_utilization: 0.0,
            cost_efficiency: 0.0,
            availability: 1.0,
        }
    }
}

impl Default for QueueStats {
    fn default() -> Self {
        Self {
            pending_tasks: 0,
            active_tasks: 0,
            completed_tasks: 0,
            total_tasks_processed: 0,
            total_budget_spent: 0.0,
        }
    }
}
