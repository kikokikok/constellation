//! SLM executor management and specialization system.
//!
//! Manages the lifecycle, registration, specialization, and health monitoring
//! of SLM (Small Language Model) executors for hybrid agent architectures.

use crate::models::hybrid_agent::{ExecutorConfig, ExecutorDomain, ExecutorPerformance};
use serde_json::Value;
use std::collections::{HashMap, HashSet};
use std::sync::{Arc, Mutex};

/// Executor lifecycle state.
#[derive(Debug, Clone, PartialEq)]
pub enum ExecutorLifecycleState {
    /// Executor is being initialized.
    Initializing,

    /// Executor is ready to accept tasks.
    Ready,

    /// Executor is currently executing tasks.
    Busy,

    /// Executor is being drained (finishing current tasks, not accepting new ones).
    Draining,

    /// Executor is stopped.
    Stopped,

    /// Executor has failed and needs recovery.
    Failed,

    /// Executor is being scaled up/down.
    Scaling,
}

/// Executor health status.
#[derive(Debug, Clone)]
pub struct ExecutorHealth {
    /// Overall health score (0.0 to 1.0).
    pub health_score: f64,

    /// CPU utilization (0.0 to 1.0).
    pub cpu_utilization: f64,

    /// Memory utilization (0.0 to 1.0).
    pub memory_utilization: f64,

    /// GPU utilization if applicable (0.0 to 1.0).
    pub gpu_utilization: Option<f64>,

    /// Network latency in milliseconds.
    pub network_latency_ms: u32,

    /// Error rate (0.0 to 1.0).
    pub error_rate: f64,

    /// Last heartbeat timestamp.
    pub last_heartbeat: chrono::DateTime<chrono::Utc>,

    /// Consecutive failures.
    pub consecutive_failures: u32,

    /// Recovery attempts.
    pub recovery_attempts: u32,
}

/// Executor specialization profile.
#[derive(Debug, Clone)]
pub struct ExecutorSpecialization {
    /// Primary domain specialization.
    pub primary_domain: ExecutorDomain,

    /// Secondary domains.
    pub secondary_domains: Vec<ExecutorDomain>,

    /// Skills with proficiency scores (0.0 to 1.0).
    pub skills: HashMap<String, f64>,

    /// Model capabilities.
    pub model_capabilities: HashSet<String>,

    /// Average task complexity this executor can handle.
    pub avg_task_complexity: f64,

    /// Maximum concurrent tasks.
    pub max_concurrent_tasks: u32,

    /// Preferred task types.
    pub preferred_task_types: HashSet<String>,

    /// Task type performance scores.
    pub task_type_performance: HashMap<String, f64>,
}

/// Managed executor instance.
#[derive(Debug, Clone)]
pub struct ManagedExecutor {
    /// Unique executor ID.
    pub id: String,

    /// Configuration.
    pub config: ExecutorConfig,

    /// Current lifecycle state.
    pub lifecycle_state: ExecutorLifecycleState,

    /// Health status.
    pub health: ExecutorHealth,

    /// Specialization profile.
    pub specialization: ExecutorSpecialization,

    /// Current load (0.0 to 1.0).
    pub current_load: f64,

    /// Active task count.
    pub active_task_count: u32,

    /// Total tasks completed.
    pub total_tasks_completed: u64,

    /// Total tasks failed.
    pub total_tasks_failed: u64,

    /// Total execution time in milliseconds.
    pub total_execution_time_ms: u64,

    /// Registration timestamp.
    pub registered_at: chrono::DateTime<chrono::Utc>,

    /// Last activity timestamp.
    pub last_activity: chrono::DateTime<chrono::Utc>,

    /// Metadata.
    pub metadata: HashMap<String, Value>,
}

/// Task-executor matching criteria.
#[derive(Debug, Clone)]
pub struct MatchingCriteria {
    /// Required domain.
    pub required_domain: Option<ExecutorDomain>,

    /// Required skills.
    pub required_skills: Vec<String>,

    /// Minimum skill proficiency (0.0 to 1.0).
    pub min_skill_proficiency: f64,

    /// Maximum latency in milliseconds.
    pub max_latency_ms: u32,

    /// Minimum success rate (0.0 to 1.0).
    pub min_success_rate: f64,

    /// Minimum quality score (0.0 to 1.0).
    pub min_quality_score: f64,

    /// Maximum cost per task.
    pub max_cost_per_task: f64,

    /// Resource constraints.
    pub resource_constraints: ResourceConstraints,

    /// Priority level.
    pub priority: u32,
}

/// Resource constraints for task matching.
#[derive(Debug, Clone)]
pub struct ResourceConstraints {
    /// Maximum CPU cores.
    pub max_cpu_cores: u32,

    /// Maximum memory in MB.
    pub max_memory_mb: u32,

    /// Maximum GPU memory in MB.
    pub max_gpu_memory_mb: Option<u32>,

    /// Maximum network bandwidth in Mbps.
    pub max_network_mbps: u32,
}

impl Default for ResourceConstraints {
    fn default() -> Self {
        Self {
            max_cpu_cores: 4,
            max_memory_mb: 8192,
            max_gpu_memory_mb: Some(8192),
            max_network_mbps: 100,
        }
    }
}

/// Executor matching result.
#[derive(Debug, Clone)]
pub struct ExecutorMatch {
    /// Executor ID.
    pub executor_id: String,

    /// Match score (0.0 to 1.0).
    pub match_score: f64,

    /// Domain match score.
    pub domain_match_score: f64,

    /// Skill match score.
    pub skill_match_score: f64,

    /// Performance match score.
    pub performance_match_score: f64,

    /// Cost match score.
    pub cost_match_score: f64,

    /// Estimated latency in milliseconds.
    pub estimated_latency_ms: u32,

    /// Estimated success rate (0.0 to 1.0).
    pub estimated_success_rate: f64,

    /// Estimated quality score (0.0 to 1.0).
    pub estimated_quality_score: f64,

    /// Estimated cost per task.
    pub estimated_cost_per_task: f64,
}

/// Scaling recommendation.
#[derive(Debug, Clone)]
pub enum ScalingRecommendation {
    /// Scale up (add more executors).
    ScaleUp {
        /// Number of executors to add.
        count: u32,

        /// Recommended domain specialization.
        domain: ExecutorDomain,

        /// Reason for scaling.
        reason: String,
    },

    /// Scale down (remove executors).
    ScaleDown {
        /// Number of executors to remove.
        count: u32,

        /// Executor IDs to remove.
        executor_ids: Vec<String>,

        /// Reason for scaling.
        reason: String,
    },

    /// No scaling needed.
    NoScaling,
}

/// SLM executor manager.
#[derive(Debug)]
pub struct SlmExecutorManager {
    /// Registered executors.
    executors: Arc<Mutex<HashMap<String, ManagedExecutor>>>,

    /// Executor groups by domain.
    executor_groups: Arc<Mutex<HashMap<ExecutorDomain, Vec<String>>>>,

    /// Executor groups by skill.
    executor_skills: Arc<Mutex<HashMap<String, Vec<String>>>>,

    /// Health monitoring interval in seconds.
    health_monitoring_interval_secs: u64,

    /// Maximum consecutive failures before marking as failed.
    max_consecutive_failures: u32,

    /// Maximum recovery attempts.
    max_recovery_attempts: u32,

    /// Load balancing strategy.
    load_balancing_strategy: LoadBalancingStrategy,

    /// Scaling thresholds.
    scaling_thresholds: ScalingThresholds,

    /// Performance history.
    performance_history: Arc<Mutex<Vec<PerformanceSnapshot>>>,
}

/// Load balancing strategy.
#[derive(Debug, Clone, PartialEq)]
pub enum LoadBalancingStrategy {
    /// Round-robin distribution.
    RoundRobin,

    /// Least loaded first.
    LeastLoaded,

    /// Performance-based (highest success rate/quality).
    PerformanceBased,

    /// Cost-based (lowest cost).
    CostBased,

    /// Latency-based (lowest latency).
    LatencyBased,

    /// Hybrid (weighted combination).
    Hybrid {
        /// Weight for load (0.0 to 1.0).
        load_weight: f64,

        /// Weight for performance (0.0 to 1.0).
        performance_weight: f64,

        /// Weight for cost (0.0 to 1.0).
        cost_weight: f64,

        /// Weight for latency (0.0 to 1.0).
        latency_weight: f64,
    },
}

/// Scaling thresholds.
#[derive(Debug, Clone)]
pub struct ScalingThresholds {
    /// Scale up when average load exceeds this threshold (0.0 to 1.0).
    pub scale_up_load_threshold: f64,

    /// Scale down when average load falls below this threshold (0.0 to 1.0).
    pub scale_down_load_threshold: f64,

    /// Scale up when success rate falls below this threshold (0.0 to 1.0).
    pub scale_up_success_threshold: f64,

    /// Scale up when latency exceeds this threshold in milliseconds.
    pub scale_up_latency_threshold_ms: u32,

    /// Minimum executors to maintain.
    pub min_executors: u32,

    /// Maximum executors allowed.
    pub max_executors: u32,

    /// Cooldown period between scaling operations in seconds.
    pub scaling_cooldown_secs: u64,
}

/// Performance snapshot.
#[derive(Debug, Clone)]
pub struct PerformanceSnapshot {
    /// Timestamp.
    pub timestamp: chrono::DateTime<chrono::Utc>,

    /// Average load across all executors.
    pub avg_load: f64,

    /// Average success rate.
    pub avg_success_rate: f64,

    /// Average quality score.
    pub avg_quality_score: f64,

    /// Average latency in milliseconds.
    pub avg_latency_ms: f64,

    /// Total active tasks.
    pub total_active_tasks: u32,

    /// Total available capacity.
    pub total_available_capacity: u32,

    /// Health status distribution.
    pub health_distribution: HashMap<String, u32>, // health_score range -> count
}

/// Health update for executor.
#[derive(Debug, Clone)]
pub struct ExecutorHealthUpdate {
    /// CPU utilization (0.0 to 1.0).
    pub cpu_utilization: Option<f64>,

    /// Memory utilization (0.0 to 1.0).
    pub memory_utilization: Option<f64>,

    /// GPU utilization (0.0 to 1.0).
    pub gpu_utilization: Option<f64>,

    /// Network latency in milliseconds.
    pub network_latency_ms: Option<u32>,

    /// Error rate (0.0 to 1.0).
    pub error_rate: Option<f64>,

    /// Task succeeded.
    pub task_succeeded: bool,

    /// Task failed.
    pub task_failed: bool,
}

impl ExecutorSpecialization {
    /// Create specialization from executor config.
    pub fn from_config(config: &ExecutorConfig) -> Self {
        let mut skills = HashMap::new();
        for skill in &config.skills {
            skills.insert(skill.id.clone(), skill.quality_score);
        }

        let mut model_capabilities = HashSet::new();
        for capability in &config.model.specialized_capabilities {
            model_capabilities.insert(capability.clone());
        }

        Self {
            primary_domain: config.domain.clone(),
            secondary_domains: Vec::new(),
            skills,
            model_capabilities,
            avg_task_complexity: 0.5,
            max_concurrent_tasks: config.max_concurrent_tasks,
            preferred_task_types: HashSet::new(),
            task_type_performance: HashMap::new(),
        }
    }
}

impl Default for SlmExecutorManager {
    fn default() -> Self {
        Self::new()
    }
}

impl SlmExecutorManager {
    /// Create a new SLM executor manager.
    pub fn new() -> Self {
        let scaling_thresholds = ScalingThresholds {
            scale_up_load_threshold: 0.8,
            scale_down_load_threshold: 0.3,
            scale_up_success_threshold: 0.9,
            scale_up_latency_threshold_ms: 5000,
            min_executors: 1,
            max_executors: 100,
            scaling_cooldown_secs: 300, // 5 minutes
        };

        Self {
            executors: Arc::new(Mutex::new(HashMap::new())),
            executor_groups: Arc::new(Mutex::new(HashMap::new())),
            executor_skills: Arc::new(Mutex::new(HashMap::new())),
            health_monitoring_interval_secs: 30,
            max_consecutive_failures: 3,
            max_recovery_attempts: 5,
            load_balancing_strategy: LoadBalancingStrategy::LeastLoaded,
            scaling_thresholds,
            performance_history: Arc::new(Mutex::new(Vec::new())),
        }
    }

    /// Create with custom configuration.
    pub fn with_config(
        health_monitoring_interval_secs: u64,
        max_consecutive_failures: u32,
        max_recovery_attempts: u32,
        load_balancing_strategy: LoadBalancingStrategy,
        scaling_thresholds: ScalingThresholds,
    ) -> Self {
        Self {
            executors: Arc::new(Mutex::new(HashMap::new())),
            executor_groups: Arc::new(Mutex::new(HashMap::new())),
            executor_skills: Arc::new(Mutex::new(HashMap::new())),
            health_monitoring_interval_secs,
            max_consecutive_failures,
            max_recovery_attempts,
            load_balancing_strategy,
            scaling_thresholds,
            performance_history: Arc::new(Mutex::new(Vec::new())),
        }
    }

    /// Register a new executor.
    pub fn register_executor(&self, config: ExecutorConfig) -> Result<String, String> {
        let executor_id = config.id.clone();
        let now = chrono::Utc::now();

        // Create specialization profile from config
        let specialization = ExecutorSpecialization::from_config(&config);

        // Create health status
        let health = ExecutorHealth {
            health_score: 1.0,
            cpu_utilization: 0.0,
            memory_utilization: 0.0,
            gpu_utilization: None,
            network_latency_ms: 0,
            error_rate: 0.0,
            last_heartbeat: now,
            consecutive_failures: 0,
            recovery_attempts: 0,
        };

        // Create managed executor
        let executor = ManagedExecutor {
            id: executor_id.clone(),
            config: config.clone(),
            lifecycle_state: ExecutorLifecycleState::Initializing,
            health,
            specialization,
            current_load: 0.0,
            active_task_count: 0,
            total_tasks_completed: 0,
            total_tasks_failed: 0,
            total_execution_time_ms: 0,
            registered_at: now,
            last_activity: now,
            metadata: HashMap::new(),
        };

        // Add to executors map
        let mut executors = self.executors.lock().unwrap();
        if executors.contains_key(&executor_id) {
            return Err(format!(
                "Executor with ID '{executor_id}' already registered"
            ));
        }
        executors.insert(executor_id.clone(), executor);

        // Update domain groups
        self.update_domain_groups(&config.domain, &executor_id);

        // Update skill groups
        for skill in &config.skills {
            self.update_skill_groups(&skill.id, &executor_id);
        }

        Ok(executor_id)
    }

    /// Update executor lifecycle state.
    pub fn update_executor_state(
        &self,
        executor_id: &str,
        state: ExecutorLifecycleState,
    ) -> Result<(), String> {
        let mut executors = self.executors.lock().unwrap();

        match executors.get_mut(executor_id) {
            Some(executor) => {
                executor.lifecycle_state = state;
                executor.last_activity = chrono::Utc::now();
                Ok(())
            }
            None => Err(format!("Executor with ID '{executor_id}' not found")),
        }
    }

    /// Update executor health status.
    pub fn update_executor_health(
        &self,
        executor_id: &str,
        health_update: ExecutorHealthUpdate,
    ) -> Result<(), String> {
        let mut executors = self.executors.lock().unwrap();

        match executors.get_mut(executor_id) {
            Some(executor) => {
                let now = chrono::Utc::now();

                // Update health metrics
                if let Some(cpu) = health_update.cpu_utilization {
                    executor.health.cpu_utilization = cpu.clamp(0.0, 1.0);
                }

                if let Some(memory) = health_update.memory_utilization {
                    executor.health.memory_utilization = memory.clamp(0.0, 1.0);
                }

                if let Some(gpu) = health_update.gpu_utilization {
                    executor.health.gpu_utilization = Some(gpu.clamp(0.0, 1.0));
                }

                if let Some(latency) = health_update.network_latency_ms {
                    executor.health.network_latency_ms = latency;
                }

                if let Some(error_rate) = health_update.error_rate {
                    executor.health.error_rate = error_rate.clamp(0.0, 1.0);
                }

                // Update heartbeat
                executor.health.last_heartbeat = now;
                executor.last_activity = now;

                // Update consecutive failures
                if health_update.task_failed {
                    executor.health.consecutive_failures += 1;

                    if executor.health.consecutive_failures >= self.max_consecutive_failures {
                        executor.lifecycle_state = ExecutorLifecycleState::Failed;
                    }
                } else if health_update.task_succeeded {
                    executor.health.consecutive_failures = 0;
                }

                // Calculate overall health score
                executor.health.health_score = self.calculate_health_score(&executor.health);

                Ok(())
            }
            None => Err(format!("Executor with ID '{executor_id}' not found")),
        }
    }

    /// Update executor load.
    pub fn update_executor_load(
        &self,
        executor_id: &str,
        task_completed: bool,
        execution_time_ms: u64,
    ) -> Result<(), String> {
        let mut executors = self.executors.lock().unwrap();

        match executors.get_mut(executor_id) {
            Some(executor) => {
                let now = chrono::Utc::now();

                if task_completed {
                    executor.total_tasks_completed += 1;
                    executor.total_execution_time_ms += execution_time_ms;
                } else {
                    executor.total_tasks_failed += 1;
                }

                // Update active task count and load
                if task_completed {
                    executor.active_task_count = executor.active_task_count.saturating_sub(1);
                }

                executor.current_load = if executor.config.max_concurrent_tasks > 0 {
                    executor.active_task_count as f64 / executor.config.max_concurrent_tasks as f64
                } else {
                    0.0
                };

                executor.last_activity = now;

                Ok(())
            }
            None => Err(format!("Executor with ID '{executor_id}' not found")),
        }
    }

    /// Assign task to executor.
    pub fn assign_task(&self, executor_id: &str) -> Result<(), String> {
        let mut executors = self.executors.lock().unwrap();

        match executors.get_mut(executor_id) {
            Some(executor) => {
                // Check if executor can accept more tasks
                if executor.active_task_count >= executor.config.max_concurrent_tasks {
                    return Err(format!("Executor '{executor_id}' at maximum capacity"));
                }

                // Check if executor is in a state that can accept tasks
                match executor.lifecycle_state {
                    ExecutorLifecycleState::Ready | ExecutorLifecycleState::Busy => {
                        executor.active_task_count += 1;
                        executor.current_load = if executor.config.max_concurrent_tasks > 0 {
                            executor.active_task_count as f64
                                / executor.config.max_concurrent_tasks as f64
                        } else {
                            0.0
                        };

                        if executor.lifecycle_state == ExecutorLifecycleState::Ready {
                            executor.lifecycle_state = ExecutorLifecycleState::Busy;
                        }

                        executor.last_activity = chrono::Utc::now();
                        Ok(())
                    }
                    _ => Err(format!(
                        "Executor '{}' not in ready state: {:?}",
                        executor_id, executor.lifecycle_state
                    )),
                }
            }
            None => Err(format!("Executor with ID '{executor_id}' not found")),
        }
    }

    /// Find best executor for a task based on matching criteria.
    pub fn find_best_executor(&self, criteria: &MatchingCriteria) -> Result<ExecutorMatch, String> {
        let executors = self.executors.lock().unwrap();

        if executors.is_empty() {
            return Err("No executors available".to_string());
        }

        let mut matches = Vec::new();

        for executor in executors.values() {
            // Skip executors that can't accept tasks
            if !self.can_accept_tasks(executor) {
                continue;
            }

            // Calculate match score
            let match_score = self.calculate_match_score(executor, criteria);

            if match_score > 0.0 {
                let executor_match = ExecutorMatch {
                    executor_id: executor.id.clone(),
                    match_score,
                    domain_match_score: self
                        .calculate_domain_match_score(&executor.specialization, criteria),
                    skill_match_score: self
                        .calculate_skill_match_score(&executor.specialization, criteria),
                    performance_match_score: self
                        .calculate_performance_match_score(&executor.config.performance, criteria),
                    cost_match_score: self
                        .calculate_cost_match_score(&executor.config.performance, criteria),
                    estimated_latency_ms: executor.config.performance.avg_latency_ms,
                    estimated_success_rate: executor.config.performance.success_rate,
                    estimated_quality_score: executor.config.performance.availability, // Using availability as proxy for quality
                    estimated_cost_per_task: executor.config.performance.cost_per_1k_tasks / 1000.0,
                };

                matches.push(executor_match);
            }
        }

        if matches.is_empty() {
            return Err("No suitable executor found".to_string());
        }

        // Sort by match score (descending)
        matches.sort_by(|a, b| b.match_score.partial_cmp(&a.match_score).unwrap());

        Ok(matches[0].clone())
    }

    /// Get executor by ID.
    pub fn get_executor(&self, executor_id: &str) -> Result<ManagedExecutor, String> {
        let executors = self.executors.lock().unwrap();

        match executors.get(executor_id) {
            Some(executor) => Ok(executor.clone()),
            None => Err(format!("Executor with ID '{executor_id}' not found")),
        }
    }

    /// Get all executors.
    pub fn get_all_executors(&self) -> Vec<ManagedExecutor> {
        let executors = self.executors.lock().unwrap();
        executors.values().cloned().collect()
    }

    /// Get executors by domain.
    pub fn get_executors_by_domain(&self, domain: &ExecutorDomain) -> Vec<ManagedExecutor> {
        let executors = self.executors.lock().unwrap();
        let groups = self.executor_groups.lock().unwrap();

        match groups.get(domain) {
            Some(executor_ids) => executor_ids
                .iter()
                .filter_map(|id| executors.get(id).cloned())
                .collect(),
            None => Vec::new(),
        }
    }

    /// Get executors by skill.
    pub fn get_executors_by_skill(&self, skill_id: &str) -> Vec<ManagedExecutor> {
        let executors = self.executors.lock().unwrap();
        let skills = self.executor_skills.lock().unwrap();

        match skills.get(skill_id) {
            Some(executor_ids) => executor_ids
                .iter()
                .filter_map(|id| executors.get(id).cloned())
                .collect(),
            None => Vec::new(),
        }
    }

    /// Check if executor can accept tasks.
    fn can_accept_tasks(&self, executor: &ManagedExecutor) -> bool {
        match executor.lifecycle_state {
            ExecutorLifecycleState::Ready | ExecutorLifecycleState::Busy => {
                executor.active_task_count < executor.config.max_concurrent_tasks
                    && executor.health.health_score > 0.5
            }
            _ => false,
        }
    }

    /// Calculate match score between executor and criteria.
    fn calculate_match_score(
        &self,
        executor: &ManagedExecutor,
        criteria: &MatchingCriteria,
    ) -> f64 {
        let domain_score = self.calculate_domain_match_score(&executor.specialization, criteria);

        // If domain is required and doesn't match, return 0.0 immediately
        if criteria.required_domain.is_some() && domain_score == 0.0 {
            return 0.0;
        }

        let skill_score = self.calculate_skill_match_score(&executor.specialization, criteria);
        let performance_score =
            self.calculate_performance_match_score(&executor.config.performance, criteria);
        let cost_score = self.calculate_cost_match_score(&executor.config.performance, criteria);
        let load_score = 1.0 - executor.current_load; // Prefer less loaded executors

        // Weighted combination
        let weights = match &self.load_balancing_strategy {
            LoadBalancingStrategy::RoundRobin => (0.2, 0.2, 0.2, 0.2, 0.2),
            LoadBalancingStrategy::LeastLoaded => (0.1, 0.1, 0.1, 0.1, 0.6),
            LoadBalancingStrategy::PerformanceBased => (0.2, 0.2, 0.4, 0.1, 0.1),
            LoadBalancingStrategy::CostBased => (0.1, 0.1, 0.1, 0.6, 0.1),
            LoadBalancingStrategy::LatencyBased => (0.1, 0.1, 0.6, 0.1, 0.1),
            LoadBalancingStrategy::Hybrid {
                load_weight,
                performance_weight,
                cost_weight,
                latency_weight,
            } => {
                let total = load_weight + performance_weight + cost_weight + latency_weight;
                let normalized_load = load_weight / total;
                let normalized_perf = performance_weight / total;
                let normalized_cost = cost_weight / total;
                let _normalized_latency = latency_weight / total;
                (0.1, 0.1, normalized_perf, normalized_cost, normalized_load)
            }
        };

        domain_score * weights.0
            + skill_score * weights.1
            + performance_score * weights.2
            + cost_score * weights.3
            + load_score * weights.4
    }

    /// Calculate domain match score.
    fn calculate_domain_match_score(
        &self,
        specialization: &ExecutorSpecialization,
        criteria: &MatchingCriteria,
    ) -> f64 {
        match &criteria.required_domain {
            Some(required_domain) => {
                if specialization.primary_domain == *required_domain {
                    1.0
                } else if specialization.secondary_domains.contains(required_domain) {
                    0.7
                } else {
                    0.0
                }
            }
            None => 1.0, // No domain requirement means perfect match
        }
    }

    /// Calculate skill match score.
    fn calculate_skill_match_score(
        &self,
        specialization: &ExecutorSpecialization,
        criteria: &MatchingCriteria,
    ) -> f64 {
        if criteria.required_skills.is_empty() {
            return 1.0;
        }

        let mut total_score = 0.0;
        let mut matched_skills = 0;

        for required_skill in &criteria.required_skills {
            if let Some(proficiency) = specialization.skills.get(required_skill)
                && *proficiency >= criteria.min_skill_proficiency
            {
                total_score += *proficiency;
                matched_skills += 1;
            }
        }

        if matched_skills == 0 {
            return 0.0;
        }

        // Average proficiency for matched skills, weighted by percentage of required skills matched
        let proficiency_score = total_score / matched_skills as f64;
        let coverage_score = matched_skills as f64 / criteria.required_skills.len() as f64;

        proficiency_score * coverage_score
    }

    /// Calculate performance match score.
    fn calculate_performance_match_score(
        &self,
        performance: &ExecutorPerformance,
        criteria: &MatchingCriteria,
    ) -> f64 {
        let mut score = 1.0;

        // Check success rate
        if performance.success_rate < criteria.min_success_rate {
            score *= 0.5;
        }

        // Check quality (using availability as proxy)
        if performance.availability < criteria.min_quality_score {
            score *= 0.5;
        }

        // Check latency
        if performance.avg_latency_ms > criteria.max_latency_ms {
            score *= 0.3;
        }

        score
    }

    /// Calculate cost match score.
    fn calculate_cost_match_score(
        &self,
        performance: &ExecutorPerformance,
        criteria: &MatchingCriteria,
    ) -> f64 {
        let cost_per_task = performance.cost_per_1k_tasks / 1000.0;

        if cost_per_task <= criteria.max_cost_per_task {
            1.0
        } else {
            // Exponential decay for costs above threshold
            let excess = cost_per_task - criteria.max_cost_per_task;
            let decay_factor = (-excess / criteria.max_cost_per_task).exp();
            decay_factor.max(0.1)
        }
    }

    /// Calculate health score.
    fn calculate_health_score(&self, health: &ExecutorHealth) -> f64 {
        let mut score = 1.0;

        // Penalize high resource utilization
        if health.cpu_utilization > 0.9 {
            score *= 0.7;
        }
        if health.memory_utilization > 0.9 {
            score *= 0.7;
        }
        if let Some(gpu) = health.gpu_utilization
            && gpu > 0.9
        {
            score *= 0.7;
        }

        // Penalize high error rate
        if health.error_rate > 0.1 {
            score *= 0.5;
        }

        // Penalize consecutive failures
        if health.consecutive_failures > 0 {
            score *= 0.8_f64.powi(health.consecutive_failures as i32);
        }

        // Penalize stale heartbeat (more than 2 intervals)
        let now = chrono::Utc::now();
        let max_stale_secs = self.health_monitoring_interval_secs * 2;
        let stale_secs = now
            .signed_duration_since(health.last_heartbeat)
            .num_seconds();

        if stale_secs > max_stale_secs as i64 {
            score *= 0.3;
        }

        score.clamp(0.0, 1.0)
    }

    /// Update domain groups.
    fn update_domain_groups(&self, domain: &ExecutorDomain, executor_id: &str) {
        let mut groups = self.executor_groups.lock().unwrap();
        groups
            .entry(domain.clone())
            .or_default()
            .push(executor_id.to_string());
    }

    /// Update skill groups.
    fn update_skill_groups(&self, skill_id: &str, executor_id: &str) {
        let mut skills = self.executor_skills.lock().unwrap();
        skills
            .entry(skill_id.to_string())
            .or_default()
            .push(executor_id.to_string());
    }

    /// Check scaling recommendations.
    pub fn check_scaling_recommendations(&self) -> ScalingRecommendation {
        let executors = self.executors.lock().unwrap();

        if executors.is_empty() {
            return ScalingRecommendation::ScaleUp {
                count: 1,
                domain: ExecutorDomain::CodeGeneration, // Default domain
                reason: "No executors available".to_string(),
            };
        }

        // Calculate average metrics
        let mut total_load = 0.0;
        let mut total_success_rate = 0.0;
        let mut total_latency = 0.0;
        let mut count = 0;

        for executor in executors.values() {
            if self.can_accept_tasks(executor) {
                total_load += executor.current_load;
                total_success_rate += executor.config.performance.success_rate;
                total_latency += executor.config.performance.avg_latency_ms as f64;
                count += 1;
            }
        }

        if count == 0 {
            return ScalingRecommendation::NoScaling;
        }

        let avg_load = total_load / count as f64;
        let avg_success_rate = total_success_rate / count as f64;
        let avg_latency = total_latency / count as f64;

        // Check scaling thresholds
        let total_executors = executors.len() as u32;

        if avg_load > self.scaling_thresholds.scale_up_load_threshold
            && total_executors < self.scaling_thresholds.max_executors
        {
            let count_to_add = ((avg_load - self.scaling_thresholds.scale_up_load_threshold)
                * total_executors as f64)
                .ceil() as u32;

            // Determine which domain needs scaling (simplified - use most common domain)
            let domain = self.get_most_loaded_domain();

            return ScalingRecommendation::ScaleUp {
                count: count_to_add.min(self.scaling_thresholds.max_executors - total_executors),
                domain,
                reason: format!("High average load: {avg_load:.2}"),
            };
        }

        if avg_load < self.scaling_thresholds.scale_down_load_threshold
            && total_executors > self.scaling_thresholds.min_executors
        {
            let count_to_remove = ((self.scaling_thresholds.scale_down_load_threshold - avg_load)
                * total_executors as f64)
                .ceil() as u32;
            let executor_ids = self.get_least_performing_executors(count_to_remove);

            return ScalingRecommendation::ScaleDown {
                count: count_to_remove.min(total_executors - self.scaling_thresholds.min_executors),
                executor_ids,
                reason: format!("Low average load: {avg_load:.2}"),
            };
        }

        if avg_success_rate < self.scaling_thresholds.scale_up_success_threshold
            && total_executors < self.scaling_thresholds.max_executors
        {
            let domain = self.get_most_loaded_domain();

            return ScalingRecommendation::ScaleUp {
                count: 1,
                domain,
                reason: format!("Low success rate: {avg_success_rate:.2}"),
            };
        }

        if avg_latency > self.scaling_thresholds.scale_up_latency_threshold_ms as f64
            && total_executors < self.scaling_thresholds.max_executors
        {
            let domain = self.get_most_loaded_domain();

            return ScalingRecommendation::ScaleUp {
                count: 1,
                domain,
                reason: format!("High latency: {avg_latency:.0}ms"),
            };
        }

        ScalingRecommendation::NoScaling
    }

    /// Get most loaded domain.
    fn get_most_loaded_domain(&self) -> ExecutorDomain {
        let executors = self.executors.lock().unwrap();
        let mut domain_loads: HashMap<ExecutorDomain, (f64, u32)> = HashMap::new();

        for executor in executors.values() {
            if self.can_accept_tasks(executor) {
                let entry = domain_loads
                    .entry(executor.specialization.primary_domain.clone())
                    .or_insert((0.0, 0));
                entry.0 += executor.current_load;
                entry.1 += 1;
            }
        }

        // Find domain with highest average load
        domain_loads
            .into_iter()
            .map(|(domain, (total_load, count))| (domain, total_load / count as f64))
            .max_by(|(_, load_a), (_, load_b)| load_a.partial_cmp(load_b).unwrap())
            .map(|(domain, _)| domain)
            .unwrap_or(ExecutorDomain::CodeGeneration)
    }

    /// Get least performing executors.
    fn get_least_performing_executors(&self, count: u32) -> Vec<String> {
        let executors = self.executors.lock().unwrap();
        let mut executor_scores: Vec<(String, f64)> = Vec::new();

        for executor in executors.values() {
            if self.can_accept_tasks(executor) {
                let score = executor.config.performance.success_rate * 0.4
                    + executor.config.performance.availability * 0.3
                    + (1.0 - executor.current_load) * 0.3;
                executor_scores.push((executor.id.clone(), score));
            }
        }

        // Sort by score (ascending)
        executor_scores.sort_by(|(_, score_a), (_, score_b)| score_a.partial_cmp(score_b).unwrap());

        // Take the specified number of executors
        executor_scores
            .iter()
            .take(count as usize)
            .map(|(id, _)| id.clone())
            .collect()
    }

    /// Record performance snapshot.
    pub fn record_performance_snapshot(&self) {
        let executors = self.executors.lock().unwrap();
        let now = chrono::Utc::now();

        let mut total_load = 0.0;
        let mut total_success_rate = 0.0;
        let mut total_quality_score = 0.0;
        let mut total_latency = 0.0;
        let mut total_active_tasks = 0;
        let mut total_available_capacity = 0;
        let mut health_distribution: HashMap<String, u32> = HashMap::new();

        let mut count = 0;
        for executor in executors.values() {
            if self.can_accept_tasks(executor) {
                total_load += executor.current_load;
                total_success_rate += executor.config.performance.success_rate;
                total_quality_score += executor.config.performance.availability; // Using availability as proxy
                total_latency += executor.config.performance.avg_latency_ms as f64;
                total_active_tasks += executor.active_task_count;
                total_available_capacity +=
                    executor.config.max_concurrent_tasks - executor.active_task_count;
                count += 1;

                // Categorize health score
                let health_category = if executor.health.health_score >= 0.8 {
                    "excellent".to_string()
                } else if executor.health.health_score >= 0.6 {
                    "good".to_string()
                } else if executor.health.health_score >= 0.4 {
                    "fair".to_string()
                } else {
                    "poor".to_string()
                };

                *health_distribution.entry(health_category).or_insert(0) += 1;
            }
        }

        if count > 0 {
            let snapshot = PerformanceSnapshot {
                timestamp: now,
                avg_load: total_load / count as f64,
                avg_success_rate: total_success_rate / count as f64,
                avg_quality_score: total_quality_score / count as f64,
                avg_latency_ms: total_latency / count as f64,
                total_active_tasks,
                total_available_capacity,
                health_distribution,
            };

            let mut history = self.performance_history.lock().unwrap();
            history.push(snapshot);

            // Keep only last 1000 snapshots
            if history.len() > 1000 {
                history.remove(0);
            }
        }
    }

    /// Get performance history.
    pub fn get_performance_history(&self, limit: Option<usize>) -> Vec<PerformanceSnapshot> {
        let history = self.performance_history.lock().unwrap();
        let limit = limit.unwrap_or(100);
        let start = if history.len() > limit {
            history.len() - limit
        } else {
            0
        };
        history[start..].to_vec()
    }
}
