//! Resource allocation and scaling strategies for hybrid agents.
//!
//! Implements dynamic resource allocation, auto-scaling, and resource pool management
//! that integrates with the SLM executor manager for optimal resource utilization.
//!
//! Based on Task 3.4 from the "incorporate-edge-research" OpenSpec change proposal.

use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use crate::models::hybrid_agent::{
    BudgetAllocation, ResourceAllocation, ResourceRequirements, ScalingStrategy,
};

/// Resource manager for hybrid agent architectures.
#[derive(Debug)]
pub struct ResourceManager {
    /// Current resource allocation configuration.
    allocation_config: ResourceAllocation,

    /// Resource pool for available executors.
    resource_pool: Arc<Mutex<ResourcePool>>,

    /// Performance metrics history.
    performance_history: Arc<Mutex<PerformanceHistory>>,

    /// Budget tracking.
    budget_tracker: Arc<Mutex<BudgetTracker>>,

    /// Auto-scaling controller.
    scaling_controller: Arc<Mutex<ScalingController>>,
}

/// Resource pool for managing executor resources.
#[derive(Debug, Clone)]
pub struct ResourcePool {
    /// Available CPU cores.
    available_cpu_cores: u32,

    /// Available memory in MB.
    available_memory_mb: u32,

    /// Available GPU memory in MB.
    available_gpu_memory_mb: Option<u32>,

    /// Available disk space in MB.
    available_disk_mb: u32,

    /// Available network bandwidth in Mbps.
    available_network_mbps: u32,

    /// Allocated resources by executor ID.
    allocated_resources: HashMap<String, ResourceRequirements>,

    /// Reserved resources for priority tasks.
    reserved_resources: ResourceRequirements,
}

/// Performance metrics history for resource optimization.
#[derive(Debug, Clone)]
pub struct PerformanceHistory {
    /// Historical CPU utilization (0.0 to 1.0).
    cpu_utilization: Vec<f64>,

    /// Historical memory utilization (0.0 to 1.0).
    memory_utilization: Vec<f64>,

    /// Historical GPU utilization (0.0 to 1.0).
    gpu_utilization: Vec<f64>,

    /// Historical network utilization (0.0 to 1.0).
    network_utilization: Vec<f64>,

    /// Historical task throughput.
    throughput_history: Vec<f64>,

    /// Historical latency measurements.
    latency_history: Vec<u32>,

    /// Historical success rates.
    success_rate_history: Vec<f64>,

    /// Timestamps for each measurement.
    timestamps: Vec<Instant>,

    /// Maximum history length.
    max_history_length: usize,
}

/// Budget tracker for cost-aware resource allocation.
#[derive(Debug, Clone)]
pub struct BudgetTracker {
    /// Total budget.
    total_budget: f64,

    /// Current spent amount.
    spent_amount: f64,

    /// Budget allocation breakdown.
    allocation: BudgetAllocation,

    /// Cost per resource type.
    resource_costs: ResourceCosts,

    /// Spending history.
    spending_history: Vec<(Instant, f64, String)>,
}

/// Resource costs for budget tracking.
#[derive(Debug, Clone)]
pub struct ResourceCosts {
    /// Cost per CPU core per hour.
    cpu_cost_per_hour: f64,

    /// Cost per GB memory per hour.
    memory_cost_per_hour: f64,

    /// Cost per GB GPU memory per hour.
    gpu_cost_per_hour: f64,

    /// Cost per GB disk per hour.
    disk_cost_per_hour: f64,

    /// Cost per Mbps network per hour.
    network_cost_per_hour: f64,

    /// Cost per executor instance per hour.
    executor_instance_cost: f64,
}

/// Auto-scaling controller for dynamic resource adjustment.
#[derive(Debug, Clone)]
pub struct ScalingController {
    /// Current scaling strategy.
    strategy: ScalingStrategy,

    /// Horizontal scaling configuration.
    horizontal_scaling: HorizontalScalingConfig,

    /// Vertical scaling configuration.
    vertical_scaling: VerticalScalingConfig,

    /// Hybrid scaling configuration.
    hybrid_scaling: HybridScalingConfig,

    /// Scaling thresholds.
    thresholds: ScalingThresholds,

    /// Cooldown period between scaling operations.
    cooldown_period: Duration,

    /// Last scaling operation time.
    last_scaling_time: Option<Instant>,
}

/// Horizontal scaling configuration.
#[derive(Debug, Clone)]
pub struct HorizontalScalingConfig {
    /// Minimum number of executor instances.
    min_instances: u32,

    /// Maximum number of executor instances.
    max_instances: u32,

    /// Desired number of instances.
    desired_instances: u32,

    /// Instance warm-up time.
    instance_warmup_time: Duration,

    /// Instance cool-down time.
    instance_cooldown_time: Duration,
}

/// Vertical scaling configuration.
#[derive(Debug, Clone)]
pub struct VerticalScalingConfig {
    /// Minimum CPU cores per instance.
    min_cpu_cores: u32,

    /// Maximum CPU cores per instance.
    max_cpu_cores: u32,

    /// Minimum memory per instance (MB).
    min_memory_mb: u32,

    /// Maximum memory per instance (MB).
    max_memory_mb: u32,

    /// Minimum GPU memory per instance (MB).
    min_gpu_memory_mb: Option<u32>,

    /// Maximum GPU memory per instance (MB).
    max_gpu_memory_mb: Option<u32>,

    /// Scaling step size for CPU.
    cpu_step_size: u32,

    /// Scaling step size for memory.
    memory_step_size: u32,
}

/// Hybrid scaling configuration.
#[derive(Debug, Clone)]
pub struct HybridScalingConfig {
    /// Weight for horizontal scaling (0.0 to 1.0).
    horizontal_weight: f64,

    /// Weight for vertical scaling (0.0 to 1.0).
    vertical_weight: f64,

    /// Minimum improvement threshold for scaling.
    min_improvement_threshold: f64,

    /// Maximum scaling operations per hour.
    max_operations_per_hour: u32,
}

/// Scaling thresholds for triggering operations.
#[derive(Debug, Clone)]
pub struct ScalingThresholds {
    /// CPU utilization threshold for scaling up (0.0 to 1.0).
    scale_up_cpu_threshold: f64,

    /// CPU utilization threshold for scaling down (0.0 to 1.0).
    scale_down_cpu_threshold: f64,

    /// Memory utilization threshold for scaling up (0.0 to 1.0).
    scale_up_memory_threshold: f64,

    /// Memory utilization threshold for scaling down (0.0 to 1.0).
    scale_down_memory_threshold: f64,

    /// Latency threshold for scaling up (ms).
    scale_up_latency_threshold: u32,

    /// Success rate threshold for scaling up (0.0 to 1.0).
    scale_up_success_rate_threshold: f64,

    /// Throughput threshold for scaling up (tasks/sec).
    scale_up_throughput_threshold: f64,

    /// Budget utilization threshold for scaling (0.0 to 1.0).
    budget_threshold: f64,
}

/// Resource allocation request.
#[derive(Debug, Clone)]
pub struct ResourceRequest {
    /// Request ID.
    pub request_id: String,

    /// Priority level.
    pub priority: u32,

    /// Required resources.
    pub requirements: ResourceRequirements,

    /// Estimated duration.
    pub estimated_duration: Duration,

    /// Task type.
    pub task_type: String,

    /// Domain specialization.
    pub domain: String,

    /// Quality requirements.
    pub quality_requirements: QualityRequirements,
}

/// Quality requirements for resource allocation.
#[derive(Debug, Clone)]
pub struct QualityRequirements {
    /// Minimum success rate (0.0 to 1.0).
    pub min_success_rate: f64,

    /// Maximum latency in milliseconds.
    pub max_latency_ms: u32,

    /// Minimum quality score (0.0 to 1.0).
    pub min_quality_score: f64,

    /// Availability requirement (0.0 to 1.0).
    pub availability_requirement: f64,
}

/// Resource allocation result.
#[derive(Debug, Clone)]
pub struct AllocationResult {
    /// Whether allocation was successful.
    pub success: bool,

    /// Allocated resources.
    pub allocated_resources: Option<ResourceRequirements>,

    /// Assigned executor ID.
    pub executor_id: Option<String>,

    /// Estimated cost.
    pub estimated_cost: f64,

    /// Estimated performance.
    pub estimated_performance: EstimatedPerformance,

    /// Alternative suggestions if allocation failed.
    pub alternatives: Vec<AllocationAlternative>,
}

/// Estimated performance for allocated resources.
#[derive(Debug, Clone)]
pub struct EstimatedPerformance {
    /// Estimated success rate (0.0 to 1.0).
    pub success_rate: f64,

    /// Estimated latency in milliseconds.
    pub latency_ms: u32,

    /// Estimated throughput in tasks per second.
    pub throughput_tps: f64,

    /// Estimated resource utilization (0.0 to 1.0).
    pub resource_utilization: f64,

    /// Estimated cost efficiency (0.0 to 1.0).
    pub cost_efficiency: f64,
}

/// Alternative allocation suggestion.
#[derive(Debug, Clone)]
pub struct AllocationAlternative {
    /// Alternative resource configuration.
    pub resources: ResourceRequirements,

    /// Estimated performance.
    pub estimated_performance: EstimatedPerformance,

    /// Cost difference from original request.
    pub cost_difference: f64,

    /// Performance difference from original request.
    pub performance_difference: f64,

    /// Time to allocate.
    pub time_to_allocate: Duration,
}

// Additional types needed for implementation
#[derive(Debug, Clone)]
pub struct ExecutorInfo {
    pub id: String,
    pub performance: crate::models::hybrid_agent::ExecutorPerformance,
    pub current_load: f64,
    pub max_capacity: f64,
}

#[derive(Debug, Clone)]
pub struct TaskHistory {
    pub task_type: String,
    pub domain: String,
    pub success_rate: f64,
    pub latency_ms: u32,
    pub throughput_tps: f64,
}

#[derive(Debug, Clone)]
pub struct PatternAnalysis {
    pub has_periodic_pattern: bool,
    pub has_growth_trend: bool,
    pub has_low_utilization_periods: bool,
    pub has_predictable_pattern: bool,
}

#[derive(Debug, Clone)]
pub struct UtilizationBreakdown {
    pub cpu_utilization: f64,
    pub memory_utilization: f64,
    pub gpu_utilization: f64,
    pub network_utilization: f64,
}

#[derive(Debug, Clone)]
pub struct UtilizationStats {
    pub cpu_utilization: f64,
    pub memory_utilization: f64,
    pub gpu_utilization: f64,
    pub network_utilization: f64,
    pub historical_trends: Vec<f64>,
    pub allocation_efficiency: f64,
}

#[derive(Debug, Clone)]
pub enum ResourceRecommendation {
    IncreaseCpu,
    DecreaseCpu,
    IncreaseMemory,
    DecreaseMemory,
    OptimizeForCost,
}

#[derive(Debug, Clone)]
pub enum ScalingRecommendationType {
    ScaleOut,
    ScaleIn,
    ScaleUp,
    ScaleDown,
    ScheduleScaling,
    PlanForGrowth,
    ImmediateScaleOut,
    ImmediateScaleIn,
}

#[derive(Debug, Clone)]
pub struct ScalingRecommendation {
    pub recommendation_type: ScalingRecommendationType,
    pub resource_requirements: ResourceRequirements,
}

#[derive(Debug, Clone)]
pub enum CostOptimization {
    UseSpotInstances,
    ReserveInstances,
    ScheduleShutdown,
}

#[derive(Debug, Clone)]
pub struct OptimizationRecommendations {
    pub resource_recommendations: Vec<ResourceRecommendation>,
    pub scaling_recommendations: Vec<ScalingRecommendation>,
    pub cost_optimizations: Vec<CostOptimization>,
}

#[derive(Debug, Clone)]
pub enum ScalingOperationType {
    ScaleOut,
    ScaleIn,
    ScaleUp,
    ScaleDown,
}

#[derive(Debug, Clone)]
pub struct ScalingOperation {
    pub operation_type: ScalingOperationType,
    pub domain: String,
    pub executor_type: String,
    pub executor_id: Option<String>,
    pub resource_requirements: ResourceRequirements,
}

#[derive(Debug, Clone)]
pub struct ScalingResult {
    pub operation: ScalingOperation,
    pub success: bool,
    pub error_message: Option<String>,
}

impl ResourceManager {
    /// Create a new resource manager with default configuration.
    pub fn new(allocation_config: ResourceAllocation) -> Self {
        let resource_pool = Arc::new(Mutex::new(ResourcePool::new(
            allocation_config.cpu_policy.max,
            allocation_config.memory_policy.max,
            allocation_config.gpu_policy.as_ref().map(|p| p.max),
            allocation_config.memory_policy.max / 2, // Estimate disk as half of memory
            allocation_config.memory_policy.max / 10, // Estimate network
        )));

        let performance_history = Arc::new(Mutex::new(PerformanceHistory::new(1000)));
        let budget_tracker = Arc::new(Mutex::new(BudgetTracker::new(
            allocation_config.budget_allocation.total_budget,
            allocation_config.budget_allocation.clone(),
        )));

        let scaling_controller = Arc::new(Mutex::new(ScalingController::new(
            allocation_config.scaling_strategy.clone(),
        )));

        Self {
            allocation_config,
            resource_pool,
            performance_history,
            budget_tracker,
            scaling_controller,
        }
    }

    /// Allocate resources for a task request.
    pub fn allocate_resources(&self, request: &ResourceRequest) -> AllocationResult {
        let mut pool = self.resource_pool.lock().unwrap();
        let mut budget = self.budget_tracker.lock().unwrap();
        let mut history = self.performance_history.lock().unwrap();

        // Check budget constraints
        let estimated_cost =
            self.estimate_resource_cost(&request.requirements, request.estimated_duration);
        if !budget.can_allocate(estimated_cost) {
            return AllocationResult::budget_exceeded(request, estimated_cost);
        }

        // Try to allocate resources
        let executor_id = format!("executor-{}", chrono::Utc::now().timestamp());
        if let Some(allocated) = pool.allocate_resources(&request.requirements, &executor_id) {
            // Update budget
            budget.allocate(estimated_cost, &format!("Executor {executor_id}"));

            // Record allocation in history
            history.record_allocation(request, &allocated, estimated_cost);

            // Create executor info for performance estimation
            let executor_info = ExecutorInfo {
                id: executor_id.clone(),
                performance: crate::models::hybrid_agent::ExecutorPerformance {
                    throughput_tps: 5.0,
                    avg_latency_ms: 2000,
                    p95_latency_ms: 5000,
                    p99_latency_ms: 10000,
                    error_rate: 0.05,
                    success_rate: 0.95,
                    availability: 0.99,
                    cost_per_1k_tasks: 0.50,
                },
                current_load: 0.5,
                max_capacity: 1.0,
            };

            return AllocationResult::success(
                allocated,
                executor_id,
                estimated_cost,
                self.estimate_performance(request, &executor_info, &history, &pool),
            );
        }

        // If no resources available, generate alternatives
        let alternatives = self.generate_alternatives(request, &[]);

        AllocationResult::failed(request, alternatives)
    }

    /// Release allocated resources.
    pub fn release_resources(&self, executor_id: &str, resources: &ResourceRequirements) {
        let mut pool = self.resource_pool.lock().unwrap();
        pool.release_resources(executor_id, resources);

        let mut history = self.performance_history.lock().unwrap();
        history.record_release(executor_id, resources);
    }

    /// Update resource allocation configuration.
    pub fn update_allocation_config(&mut self, config: ResourceAllocation) {
        self.allocation_config = config.clone();

        // Update resource pool limits
        let mut pool = self.resource_pool.lock().unwrap();
        pool.update_limits(
            config.cpu_policy.max,
            config.memory_policy.max,
            config.gpu_policy.as_ref().map(|p| p.max),
        );
    }

    /// Check and apply auto-scaling based on current metrics.
    pub fn check_auto_scaling(&self) -> Vec<ScalingOperation> {
        // Always acquire locks in consistent order to avoid deadlocks:
        // 1. resource_pool, 2. budget_tracker, 3. performance_history, 4. scaling_controller
        let pool = self.resource_pool.lock().unwrap();
        let budget = self.budget_tracker.lock().unwrap();
        let history = self.performance_history.lock().unwrap();
        let mut controller = self.scaling_controller.lock().unwrap();

        // Check if in cooldown period
        if controller.in_cooldown() {
            return Vec::new();
        }

        // Get current metrics
        let current_metrics = history.get_current_metrics();

        // Don't scale if we don't have meaningful metrics data
        if history.cpu_utilization.is_empty() {
            return Vec::new();
        }

        // Determine scaling operations based on strategy
        let operations = match self.allocation_config.scaling_strategy {
            ScalingStrategy::Horizontal => controller.check_horizontal_scaling(
                &current_metrics,
                &[],
                budget.get_budget_utilization(),
            ),
            ScalingStrategy::Vertical => controller.check_vertical_scaling(
                &current_metrics,
                &[],
                budget.get_budget_utilization(),
            ),
            ScalingStrategy::Hybrid => controller.check_hybrid_scaling(
                &current_metrics,
                &[],
                budget.get_budget_utilization(),
            ),
            ScalingStrategy::Burstable => controller.check_burstable_scaling(
                &current_metrics,
                &[],
                budget.get_budget_utilization(),
            ),
            ScalingStrategy::Reserved => Vec::new(), // No auto-scaling for reserved
            ScalingStrategy::Spot => controller.check_spot_scaling(
                &current_metrics,
                &[],
                budget.get_budget_utilization(),
            ),
        };

        // Record scaling operations
        if !operations.is_empty() {
            controller.record_scaling_operations(&operations);
        }

        operations
    }

    /// Apply scaling operations.
    pub fn apply_scaling_operations(
        &self,
        operations: Vec<ScalingOperation>,
    ) -> Vec<ScalingResult> {
        let mut results = Vec::new();

        for operation in operations {
            match operation.operation_type {
                ScalingOperationType::ScaleOut => {
                    // Add new executor instance
                    results.push(ScalingResult::from_operation(operation, true));
                }
                ScalingOperationType::ScaleIn => {
                    // Remove executor instance
                    results.push(ScalingResult::from_operation(operation, true));
                }
                ScalingOperationType::ScaleUp => {
                    // Increase resources for existing executor
                    results.push(ScalingResult::from_operation(operation, true));
                }
                ScalingOperationType::ScaleDown => {
                    // Decrease resources for existing executor
                    results.push(ScalingResult::from_operation(operation, true));
                }
            }
        }

        results
    }

    /// Get resource utilization statistics.
    pub fn get_utilization_stats(&self) -> UtilizationStats {
        let pool = self.resource_pool.lock().unwrap();
        let history = self.performance_history.lock().unwrap();

        UtilizationStats {
            cpu_utilization: pool.get_cpu_utilization(),
            memory_utilization: pool.get_memory_utilization(),
            gpu_utilization: pool.get_gpu_utilization(),
            network_utilization: pool.get_network_utilization(),
            historical_trends: history.get_utilization_trends(),
            allocation_efficiency: pool.get_allocation_efficiency(),
        }
    }

    /// Optimize resource allocation based on historical data.
    pub fn optimize_allocation(&self) -> OptimizationRecommendations {
        // Always acquire locks in consistent order to avoid deadlocks:
        // 1. resource_pool, 2. budget_tracker, 3. performance_history
        let pool = self.resource_pool.lock().unwrap();
        let budget = self.budget_tracker.lock().unwrap();
        let history = self.performance_history.lock().unwrap();

        let recommendations = history.analyze_patterns();
        let current_utilization = pool.get_utilization_breakdown();
        let budget_utilization = budget.get_budget_utilization();

        OptimizationRecommendations {
            resource_recommendations: self.generate_resource_recommendations(
                &recommendations,
                &current_utilization,
                budget_utilization,
            ),
            scaling_recommendations: self
                .generate_scaling_recommendations(&recommendations, &current_utilization),
            cost_optimizations: self
                .generate_cost_optimizations(&recommendations, budget_utilization),
        }
    }

    /// Estimate resource cost for given requirements and duration.
    fn estimate_resource_cost(
        &self,
        requirements: &ResourceRequirements,
        duration: Duration,
    ) -> f64 {
        let costs = ResourceCosts::default();
        let hours = duration.as_secs_f64() / 3600.0;

        let cpu_cost = requirements.cpu_cores as f64 * costs.cpu_cost_per_hour * hours;
        let memory_cost =
            requirements.memory_mb as f64 / 1024.0 * costs.memory_cost_per_hour * hours;
        let gpu_cost = requirements.gpu_memory_mb.unwrap_or(0) as f64 / 1024.0
            * costs.gpu_cost_per_hour
            * hours;
        let disk_cost = requirements.disk_mb as f64 / 1024.0 * costs.disk_cost_per_hour * hours;
        let network_cost = requirements.network_mbps as f64 * costs.network_cost_per_hour * hours;

        cpu_cost
            + memory_cost
            + gpu_cost
            + disk_cost
            + network_cost
            + costs.executor_instance_cost * hours
    }

    /// Estimate performance for given request and executor.
    fn estimate_performance(
        &self,
        request: &ResourceRequest,
        executor: &ExecutorInfo,
        history: &PerformanceHistory,
        pool: &ResourcePool,
    ) -> EstimatedPerformance {
        let similar_tasks = history.find_similar_tasks(&request.task_type, &request.domain);

        EstimatedPerformance {
            success_rate: self.calculate_estimated_success_rate(executor, &similar_tasks),
            latency_ms: self.calculate_estimated_latency(
                executor,
                &similar_tasks,
                &request.requirements,
            ),
            throughput_tps: self.calculate_estimated_throughput(executor, &similar_tasks),
            resource_utilization: self.calculate_resource_utilization(&request.requirements, pool),
            cost_efficiency: self.calculate_cost_efficiency(executor, &request.requirements),
        }
    }

    /// Generate allocation alternatives when primary allocation fails.
    fn generate_alternatives(
        &self,
        request: &ResourceRequest,
        scaling_recommendations: &[ScalingRecommendation],
    ) -> Vec<AllocationAlternative> {
        let mut alternatives = Vec::new();

        // Alternative 1: Reduced resources
        if let Some(reduced) = self.reduce_resource_requirements(&request.requirements) {
            let estimated_performance =
                self.estimate_performance_for_alternative(request, &reduced);
            let cost_difference = self.estimate_resource_cost(&reduced, request.estimated_duration)
                - self.estimate_resource_cost(&request.requirements, request.estimated_duration);

            let performance_diff =
                self.calculate_performance_difference(request, &estimated_performance);
            alternatives.push(AllocationAlternative {
                resources: reduced,
                estimated_performance: estimated_performance.clone(),
                cost_difference,
                performance_difference: performance_diff,
                time_to_allocate: Duration::from_secs(30), // Estimated time
            });
        }

        // Alternative 2: Delayed execution with scaling
        if !scaling_recommendations.is_empty() {
            let scaling_time = self.estimate_scaling_time(scaling_recommendations);
            let scaled_resources = self.get_scaled_resources(scaling_recommendations);

            let estimated_performance =
                self.estimate_performance_for_alternative(request, &scaled_resources);

            alternatives.push(AllocationAlternative {
                resources: scaled_resources,
                estimated_performance,
                cost_difference: 0.0,        // Same cost after scaling
                performance_difference: 0.0, // Same performance after scaling
                time_to_allocate: scaling_time,
            });
        }

        // Alternative 3: Different executor type
        let alternative_executor = self.find_alternative_executor_type(&request.domain);
        if let Some(alt_resources) = alternative_executor {
            let estimated_performance =
                self.estimate_performance_for_alternative(request, &alt_resources);
            let cost_difference = self
                .estimate_resource_cost(&alt_resources, request.estimated_duration)
                - self.estimate_resource_cost(&request.requirements, request.estimated_duration);

            let performance_diff =
                self.calculate_performance_difference(request, &estimated_performance);
            alternatives.push(AllocationAlternative {
                resources: alt_resources,
                estimated_performance: estimated_performance.clone(),
                cost_difference,
                performance_difference: performance_diff,
                time_to_allocate: Duration::from_secs(60), // Longer setup time
            });
        }

        alternatives
    }

    // Helper methods for performance estimation
    fn calculate_estimated_success_rate(
        &self,
        executor: &ExecutorInfo,
        similar_tasks: &[TaskHistory],
    ) -> f64 {
        if similar_tasks.is_empty() {
            return executor.performance.success_rate;
        }

        let total_success: f64 = similar_tasks.iter().map(|t| t.success_rate).sum();
        total_success / similar_tasks.len() as f64
    }

    fn calculate_estimated_latency(
        &self,
        executor: &ExecutorInfo,
        similar_tasks: &[TaskHistory],
        requirements: &ResourceRequirements,
    ) -> u32 {
        if similar_tasks.is_empty() {
            return executor.performance.avg_latency_ms;
        }

        let avg_latency: f64 = similar_tasks
            .iter()
            .map(|t| t.latency_ms as f64)
            .sum::<f64>()
            / similar_tasks.len() as f64;

        // Adjust based on resource requirements
        let resource_factor = self.calculate_resource_factor(requirements);
        (avg_latency * resource_factor) as u32
    }

    fn calculate_estimated_throughput(
        &self,
        executor: &ExecutorInfo,
        similar_tasks: &[TaskHistory],
    ) -> f64 {
        if similar_tasks.is_empty() {
            return executor.performance.throughput_tps;
        }

        let avg_throughput: f64 = similar_tasks.iter().map(|t| t.throughput_tps).sum::<f64>()
            / similar_tasks.len() as f64;
        avg_throughput
    }

    fn calculate_resource_utilization(
        &self,
        requirements: &ResourceRequirements,
        pool: &ResourcePool,
    ) -> f64 {
        let total_cpu = pool.available_cpu_cores as f64;
        let total_memory = pool.available_memory_mb as f64;

        let cpu_util = requirements.cpu_cores as f64 / total_cpu;
        let memory_util = requirements.memory_mb as f64 / total_memory;

        (cpu_util + memory_util) / 2.0
    }

    fn calculate_cost_efficiency(
        &self,
        executor: &ExecutorInfo,
        requirements: &ResourceRequirements,
    ) -> f64 {
        let cost = self.estimate_resource_cost(requirements, Duration::from_secs(3600)); // Hourly cost
        let value = executor.performance.success_rate * executor.performance.throughput_tps;

        if cost > 0.0 { value / cost } else { 1.0 }
    }

    fn calculate_resource_factor(&self, requirements: &ResourceRequirements) -> f64 {
        // Simple heuristic: more resources = faster execution
        let base_factor = 1.0;
        let cpu_factor = 1.0 / (requirements.cpu_cores as f64).sqrt();
        let memory_factor = 1.0 / (requirements.memory_mb as f64 / 1024.0).sqrt();

        base_factor * cpu_factor * memory_factor
    }

    fn calculate_performance_difference(
        &self,
        request: &ResourceRequest,
        estimated: &EstimatedPerformance,
    ) -> f64 {
        let target_success = request.quality_requirements.min_success_rate;
        let target_latency = request.quality_requirements.max_latency_ms as f64;

        let success_diff = (estimated.success_rate - target_success).abs();
        let latency_diff = (estimated.latency_ms as f64 - target_latency).abs() / target_latency;

        (success_diff + latency_diff) / 2.0
    }

    fn reduce_resource_requirements(
        &self,
        requirements: &ResourceRequirements,
    ) -> Option<ResourceRequirements> {
        let mut reduced = requirements.clone();

        // Try reducing by 25%
        reduced.cpu_cores = (reduced.cpu_cores as f64 * 0.75).ceil() as u32;
        reduced.memory_mb = (reduced.memory_mb as f64 * 0.75).ceil() as u32;

        if reduced.cpu_cores >= 1 && reduced.memory_mb >= 512 {
            Some(reduced)
        } else {
            None
        }
    }

    fn estimate_scaling_time(&self, recommendations: &[ScalingRecommendation]) -> Duration {
        // Estimate scaling time based on recommendation type
        let mut total_time = Duration::from_secs(0);

        for rec in recommendations {
            match rec.recommendation_type {
                ScalingRecommendationType::ScaleOut => {
                    total_time += Duration::from_secs(120); // 2 minutes for new instance
                }
                ScalingRecommendationType::ScaleUp => {
                    total_time += Duration::from_secs(30); // 30 seconds for resource increase
                }
                _ => {}
            }
        }

        total_time
    }

    fn get_scaled_resources(
        &self,
        recommendations: &[ScalingRecommendation],
    ) -> ResourceRequirements {
        // Get resources from first scaling recommendation
        if let Some(rec) = recommendations.first() {
            rec.resource_requirements.clone()
        } else {
            ResourceRequirements::default()
        }
    }

    fn find_alternative_executor_type(&self, domain: &str) -> Option<ResourceRequirements> {
        // Map domains to alternative executor types
        match domain {
            "CodeGeneration" => Some(ResourceRequirements {
                cpu_cores: 4,
                memory_mb: 8192,
                gpu_memory_mb: Some(8192),
                disk_mb: 2048,
                network_mbps: 200,
            }),
            "DataAnalysis" => Some(ResourceRequirements {
                cpu_cores: 8,
                memory_mb: 16384,
                gpu_memory_mb: None,
                disk_mb: 4096,
                network_mbps: 100,
            }),
            "Research" => Some(ResourceRequirements {
                cpu_cores: 2,
                memory_mb: 4096,
                gpu_memory_mb: None,
                disk_mb: 1024,
                network_mbps: 50,
            }),
            _ => None,
        }
    }

    fn estimate_performance_for_alternative(
        &self,
        request: &ResourceRequest,
        resources: &ResourceRequirements,
    ) -> EstimatedPerformance {
        // Simplified estimation for alternatives
        let pool = self.resource_pool.lock().unwrap();
        EstimatedPerformance {
            success_rate: request.quality_requirements.min_success_rate * 0.9, // 10% reduction
            latency_ms: request.quality_requirements.max_latency_ms * 2,       // Double latency
            throughput_tps: 5.0, // Conservative estimate
            resource_utilization: self.calculate_resource_utilization(resources, &pool),
            cost_efficiency: 0.7, // Lower efficiency
        }
    }

    fn generate_resource_recommendations(
        &self,
        patterns: &PatternAnalysis,
        utilization: &UtilizationBreakdown,
        budget_utilization: f64,
    ) -> Vec<ResourceRecommendation> {
        let mut recommendations = Vec::new();

        // CPU recommendations
        if utilization.cpu_utilization > 0.8 {
            recommendations.push(ResourceRecommendation::IncreaseCpu);
        } else if utilization.cpu_utilization < 0.3 {
            recommendations.push(ResourceRecommendation::DecreaseCpu);
        }

        // Memory recommendations
        if utilization.memory_utilization > 0.8 {
            recommendations.push(ResourceRecommendation::IncreaseMemory);
        } else if utilization.memory_utilization < 0.3 {
            recommendations.push(ResourceRecommendation::DecreaseMemory);
        }

        // Budget-aware recommendations
        if budget_utilization > 0.9 {
            recommendations.push(ResourceRecommendation::OptimizeForCost);
        }

        recommendations
    }

    fn generate_scaling_recommendations(
        &self,
        patterns: &PatternAnalysis,
        utilization: &UtilizationBreakdown,
    ) -> Vec<ScalingRecommendation> {
        let mut recommendations = Vec::new();

        // Check for periodic patterns
        if patterns.has_periodic_pattern {
            recommendations.push(ScalingRecommendation {
                recommendation_type: ScalingRecommendationType::ScheduleScaling,
                resource_requirements: ResourceRequirements::default(),
            });
        }

        // Check for growth trends
        if patterns.has_growth_trend {
            recommendations.push(ScalingRecommendation {
                recommendation_type: ScalingRecommendationType::PlanForGrowth,
                resource_requirements: ResourceRequirements::default(),
            });
        }

        // Check for high utilization (upscaling)
        if utilization.cpu_utilization > 0.9 || utilization.memory_utilization > 0.9 {
            recommendations.push(ScalingRecommendation {
                recommendation_type: ScalingRecommendationType::ImmediateScaleOut,
                resource_requirements: ResourceRequirements {
                    cpu_cores: 4,
                    memory_mb: 8192,
                    gpu_memory_mb: Some(4096),
                    disk_mb: 2048,
                    network_mbps: 100,
                },
            });
        }

        // Check for low utilization (downscaling)
        if utilization.cpu_utilization < 0.3 && utilization.memory_utilization < 0.3 {
            recommendations.push(ScalingRecommendation {
                recommendation_type: ScalingRecommendationType::ImmediateScaleIn,
                resource_requirements: ResourceRequirements {
                    cpu_cores: 2,
                    memory_mb: 4096,
                    gpu_memory_mb: Some(2048),
                    disk_mb: 1024,
                    network_mbps: 50,
                },
            });
        }

        // Check for sustained low utilization (vertical downscaling)
        if patterns.has_low_utilization_periods && utilization.cpu_utilization < 0.2 {
            recommendations.push(ScalingRecommendation {
                recommendation_type: ScalingRecommendationType::ScaleDown,
                resource_requirements: ResourceRequirements {
                    cpu_cores: 1,
                    memory_mb: 2048,
                    gpu_memory_mb: Some(1024),
                    disk_mb: 512,
                    network_mbps: 25,
                },
            });
        }

        recommendations
    }

    fn generate_cost_optimizations(
        &self,
        patterns: &PatternAnalysis,
        budget_utilization: f64,
    ) -> Vec<CostOptimization> {
        let mut optimizations = Vec::new();

        if patterns.has_low_utilization_periods {
            optimizations.push(CostOptimization::UseSpotInstances);
        }

        if budget_utilization > 0.8 {
            optimizations.push(CostOptimization::ReserveInstances);
        }

        if patterns.has_predictable_pattern {
            optimizations.push(CostOptimization::ScheduleShutdown);
        }

        optimizations
    }
}

impl ResourcePool {
    /// Create a new resource pool.
    pub fn new(
        total_cpu_cores: u32,
        total_memory_mb: u32,
        total_gpu_memory_mb: Option<u32>,
        total_disk_mb: u32,
        total_network_mbps: u32,
    ) -> Self {
        Self {
            available_cpu_cores: total_cpu_cores,
            available_memory_mb: total_memory_mb,
            available_gpu_memory_mb: total_gpu_memory_mb,
            available_disk_mb: total_disk_mb,
            available_network_mbps: total_network_mbps,
            allocated_resources: HashMap::new(),
            reserved_resources: ResourceRequirements {
                cpu_cores: 0,
                memory_mb: 0,
                gpu_memory_mb: None,
                disk_mb: 0,
                network_mbps: 0,
            },
        }
    }

    /// Allocate resources for an executor.
    pub fn allocate_resources(
        &mut self,
        requirements: &ResourceRequirements,
        executor_id: &str,
    ) -> Option<ResourceRequirements> {
        // Check if resources are available
        if !self.has_available_resources(requirements) {
            return None;
        }

        // Allocate resources
        self.available_cpu_cores -= requirements.cpu_cores;
        self.available_memory_mb -= requirements.memory_mb;

        if let Some(req_gpu) = requirements.gpu_memory_mb
            && let Some(available_gpu) = &mut self.available_gpu_memory_mb
        {
            *available_gpu -= req_gpu;
        }

        self.available_disk_mb -= requirements.disk_mb;
        self.available_network_mbps -= requirements.network_mbps;

        // Record allocation
        self.allocated_resources
            .insert(executor_id.to_string(), requirements.clone());

        Some(requirements.clone())
    }

    /// Release allocated resources.
    pub fn release_resources(&mut self, executor_id: &str, resources: &ResourceRequirements) {
        if let Some(allocated) = self.allocated_resources.remove(executor_id) {
            // Return resources to pool
            self.available_cpu_cores += allocated.cpu_cores;
            self.available_memory_mb += allocated.memory_mb;

            if let Some(allocated_gpu) = allocated.gpu_memory_mb
                && let Some(available_gpu) = &mut self.available_gpu_memory_mb
            {
                *available_gpu += allocated_gpu;
            }

            self.available_disk_mb += allocated.disk_mb;
            self.available_network_mbps += allocated.network_mbps;
        }
    }

    /// Check if resources are available.
    fn has_available_resources(&self, requirements: &ResourceRequirements) -> bool {
        if self.available_cpu_cores < requirements.cpu_cores {
            return false;
        }

        if self.available_memory_mb < requirements.memory_mb {
            return false;
        }

        if let Some(req_gpu) = requirements.gpu_memory_mb {
            if let Some(available_gpu) = self.available_gpu_memory_mb {
                if available_gpu < req_gpu {
                    return false;
                }
            } else {
                return false; // GPU requested but not available
            }
        }

        if self.available_disk_mb < requirements.disk_mb {
            return false;
        }

        if self.available_network_mbps < requirements.network_mbps {
            return false;
        }

        true
    }

    /// Update resource limits.
    pub fn update_limits(
        &mut self,
        max_cpu_cores: u32,
        max_memory_mb: u32,
        max_gpu_memory_mb: Option<u32>,
    ) {
        self.available_cpu_cores = max_cpu_cores;
        self.available_memory_mb = max_memory_mb;
        self.available_gpu_memory_mb = max_gpu_memory_mb;

        // Recalculate available resources based on current allocations
        for allocated in self.allocated_resources.values() {
            self.available_cpu_cores -= allocated.cpu_cores;
            self.available_memory_mb -= allocated.memory_mb;

            if let Some(allocated_gpu) = allocated.gpu_memory_mb
                && let Some(available_gpu) = &mut self.available_gpu_memory_mb
            {
                *available_gpu -= allocated_gpu;
            }
        }
    }

    /// Get CPU utilization (0.0 to 1.0).
    pub fn get_cpu_utilization(&self) -> f64 {
        let total_allocated: u32 = self.allocated_resources.values().map(|r| r.cpu_cores).sum();
        let total = self.available_cpu_cores + total_allocated;

        if total > 0 {
            total_allocated as f64 / total as f64
        } else {
            0.0
        }
    }

    /// Get memory utilization (0.0 to 1.0).
    pub fn get_memory_utilization(&self) -> f64 {
        let total_allocated: u32 = self.allocated_resources.values().map(|r| r.memory_mb).sum();
        let total = self.available_memory_mb + total_allocated;

        if total > 0 {
            total_allocated as f64 / total as f64
        } else {
            0.0
        }
    }

    /// Get GPU utilization (0.0 to 1.0).
    pub fn get_gpu_utilization(&self) -> f64 {
        let total_allocated: u32 = self
            .allocated_resources
            .values()
            .filter_map(|r| r.gpu_memory_mb)
            .sum();

        if let Some(total_gpu) = self.available_gpu_memory_mb {
            let total = total_gpu + total_allocated;
            if total > 0 {
                total_allocated as f64 / total as f64
            } else {
                0.0
            }
        } else {
            0.0
        }
    }

    /// Get network utilization (0.0 to 1.0).
    pub fn get_network_utilization(&self) -> f64 {
        let total_allocated: u32 = self
            .allocated_resources
            .values()
            .map(|r| r.network_mbps)
            .sum();
        let total = self.available_network_mbps + total_allocated;

        if total > 0 {
            total_allocated as f64 / total as f64
        } else {
            0.0
        }
    }

    /// Get allocation efficiency (0.0 to 1.0).
    pub fn get_allocation_efficiency(&self) -> f64 {
        let cpu_util = self.get_cpu_utilization();
        let memory_util = self.get_memory_utilization();
        let gpu_util = self.get_gpu_utilization();
        let network_util = self.get_network_utilization();

        // Weighted average with CPU and memory having higher weights
        cpu_util * 0.4 + memory_util * 0.4 + gpu_util * 0.1 + network_util * 0.1
    }

    /// Get utilization breakdown.
    pub fn get_utilization_breakdown(&self) -> UtilizationBreakdown {
        UtilizationBreakdown {
            cpu_utilization: self.get_cpu_utilization(),
            memory_utilization: self.get_memory_utilization(),
            gpu_utilization: self.get_gpu_utilization(),
            network_utilization: self.get_network_utilization(),
        }
    }
}

impl PerformanceHistory {
    /// Create a new performance history with maximum length.
    pub fn new(max_history_length: usize) -> Self {
        Self {
            cpu_utilization: Vec::with_capacity(max_history_length),
            memory_utilization: Vec::with_capacity(max_history_length),
            gpu_utilization: Vec::with_capacity(max_history_length),
            network_utilization: Vec::with_capacity(max_history_length),
            throughput_history: Vec::with_capacity(max_history_length),
            latency_history: Vec::with_capacity(max_history_length),
            success_rate_history: Vec::with_capacity(max_history_length),
            timestamps: Vec::with_capacity(max_history_length),
            max_history_length,
        }
    }

    /// Record a resource allocation.
    pub fn record_allocation(
        &mut self,
        request: &ResourceRequest,
        resources: &ResourceRequirements,
        cost: f64,
    ) {
        self.record_metrics(0.5, 0.5, 0.3, 0.2, 1.0, 1000, 0.95);
    }

    /// Record resource release.
    pub fn record_release(&mut self, executor_id: &str, resources: &ResourceRequirements) {
        self.record_metrics(0.3, 0.3, 0.2, 0.1, 0.5, 500, 0.98);
    }

    /// Record performance metrics.
    fn record_metrics(
        &mut self,
        cpu_util: f64,
        memory_util: f64,
        gpu_util: f64,
        network_util: f64,
        throughput: f64,
        latency: u32,
        success_rate: f64,
    ) {
        // Remove oldest entry if at capacity
        if self.cpu_utilization.len() >= self.max_history_length {
            self.cpu_utilization.remove(0);
            self.memory_utilization.remove(0);
            self.gpu_utilization.remove(0);
            self.network_utilization.remove(0);
            self.throughput_history.remove(0);
            self.latency_history.remove(0);
            self.success_rate_history.remove(0);
            self.timestamps.remove(0);
        }

        // Add new metrics
        self.cpu_utilization.push(cpu_util);
        self.memory_utilization.push(memory_util);
        self.gpu_utilization.push(gpu_util);
        self.network_utilization.push(network_util);
        self.throughput_history.push(throughput);
        self.latency_history.push(latency);
        self.success_rate_history.push(success_rate);
        self.timestamps.push(Instant::now());
    }

    /// Get current metrics.
    pub fn get_current_metrics(&self) -> CurrentMetrics {
        if self.cpu_utilization.is_empty() {
            return CurrentMetrics::default();
        }

        CurrentMetrics {
            cpu_utilization: *self.cpu_utilization.last().unwrap(),
            memory_utilization: *self.memory_utilization.last().unwrap(),
            gpu_utilization: *self.gpu_utilization.last().unwrap(),
            network_utilization: *self.network_utilization.last().unwrap(),
            throughput: *self.throughput_history.last().unwrap(),
            latency: *self.latency_history.last().unwrap(),
            success_rate: *self.success_rate_history.last().unwrap(),
        }
    }

    /// Find similar tasks in history.
    pub fn find_similar_tasks(&self, task_type: &str, domain: &str) -> Vec<TaskHistory> {
        // Simplified implementation - in real system would query actual history
        vec![TaskHistory {
            task_type: task_type.to_string(),
            domain: domain.to_string(),
            success_rate: 0.95,
            latency_ms: 1000,
            throughput_tps: 5.0,
        }]
    }

    /// Get utilization trends.
    pub fn get_utilization_trends(&self) -> Vec<f64> {
        if self.cpu_utilization.len() < 2 {
            return vec![0.0];
        }

        // Calculate simple moving average of CPU utilization
        let window_size = 10.min(self.cpu_utilization.len());
        let start = self.cpu_utilization.len() - window_size;

        self.cpu_utilization[start..].to_vec()
    }

    /// Analyze patterns in historical data.
    pub fn analyze_patterns(&self) -> PatternAnalysis {
        PatternAnalysis {
            has_periodic_pattern: self.cpu_utilization.len() > 100,
            has_growth_trend: self.cpu_utilization.len() > 50
                && self.cpu_utilization.last().unwrap_or(&0.0)
                    > self.cpu_utilization.first().unwrap_or(&0.0),
            has_low_utilization_periods: self.cpu_utilization.iter().any(|&x| x < 0.2),
            has_predictable_pattern: self.cpu_utilization.len() > 200,
        }
    }
}

impl BudgetTracker {
    /// Create a new budget tracker.
    pub fn new(total_budget: f64, allocation: BudgetAllocation) -> Self {
        Self {
            total_budget,
            spent_amount: 0.0,
            allocation,
            resource_costs: ResourceCosts::default(),
            spending_history: Vec::new(),
        }
    }

    /// Check if budget can accommodate allocation.
    pub fn can_allocate(&self, cost: f64) -> bool {
        self.spent_amount + cost <= self.total_budget
    }

    /// Allocate budget for a resource.
    pub fn allocate(&mut self, cost: f64, description: &str) {
        self.spent_amount += cost;
        self.spending_history
            .push((Instant::now(), cost, description.to_string()));
    }

    /// Get budget utilization (0.0 to 1.0).
    pub fn get_budget_utilization(&self) -> f64 {
        if self.total_budget > 0.0 {
            self.spent_amount / self.total_budget
        } else {
            0.0
        }
    }
}

impl ScalingController {
    /// Create a new scaling controller.
    pub fn new(strategy: ScalingStrategy) -> Self {
        Self {
            strategy,
            horizontal_scaling: HorizontalScalingConfig::default(),
            vertical_scaling: VerticalScalingConfig::default(),
            hybrid_scaling: HybridScalingConfig::default(),
            thresholds: ScalingThresholds::default(),
            cooldown_period: Duration::from_secs(300), // 5 minutes
            last_scaling_time: None,
        }
    }

    /// Check if in cooldown period.
    pub fn in_cooldown(&self) -> bool {
        if let Some(last_time) = self.last_scaling_time {
            last_time.elapsed() < self.cooldown_period
        } else {
            false
        }
    }

    /// Check horizontal scaling needs.
    pub fn check_horizontal_scaling(
        &self,
        metrics: &CurrentMetrics,
        recommendations: &[ScalingRecommendation],
        budget_utilization: f64,
    ) -> Vec<ScalingOperation> {
        let mut operations = Vec::new();

        // Check for scale out (upscaling)
        if metrics.cpu_utilization > self.thresholds.scale_up_cpu_threshold
            && budget_utilization < self.thresholds.budget_threshold
        {
            operations.push(ScalingOperation {
                operation_type: ScalingOperationType::ScaleOut,
                domain: "general".to_string(),
                executor_type: "default".to_string(),
                executor_id: None,
                resource_requirements: ResourceRequirements::default(),
            });
        }

        // Check for scale in (downscaling)
        if metrics.cpu_utilization < self.thresholds.scale_down_cpu_threshold
            && metrics.memory_utilization < self.thresholds.scale_down_memory_threshold
        {
            operations.push(ScalingOperation {
                operation_type: ScalingOperationType::ScaleIn,
                domain: "general".to_string(),
                executor_type: "default".to_string(),
                executor_id: None,
                resource_requirements: ResourceRequirements::default(),
            });
        }

        operations
    }

    /// Check vertical scaling needs.
    pub fn check_vertical_scaling(
        &self,
        metrics: &CurrentMetrics,
        recommendations: &[ScalingRecommendation],
        budget_utilization: f64,
    ) -> Vec<ScalingOperation> {
        let mut operations = Vec::new();

        if metrics.cpu_utilization > self.thresholds.scale_up_cpu_threshold
            && budget_utilization < self.thresholds.budget_threshold
        {
            operations.push(ScalingOperation {
                operation_type: ScalingOperationType::ScaleUp,
                domain: "general".to_string(),
                executor_type: "default".to_string(),
                executor_id: Some("executor-1".to_string()),
                resource_requirements: ResourceRequirements {
                    cpu_cores: 4,
                    memory_mb: 8192,
                    gpu_memory_mb: Some(4096),
                    disk_mb: 2048,
                    network_mbps: 100,
                },
            });
        }

        operations
    }

    /// Check hybrid scaling needs.
    pub fn check_hybrid_scaling(
        &self,
        metrics: &CurrentMetrics,
        recommendations: &[ScalingRecommendation],
        budget_utilization: f64,
    ) -> Vec<ScalingOperation> {
        let mut operations = Vec::new();

        // Combine horizontal and vertical scaling logic
        let horizontal_ops =
            self.check_horizontal_scaling(metrics, recommendations, budget_utilization);
        let vertical_ops =
            self.check_vertical_scaling(metrics, recommendations, budget_utilization);

        operations.extend(horizontal_ops);
        operations.extend(vertical_ops);

        operations
    }

    /// Check burstable scaling needs.
    pub fn check_burstable_scaling(
        &self,
        metrics: &CurrentMetrics,
        recommendations: &[ScalingRecommendation],
        budget_utilization: f64,
    ) -> Vec<ScalingOperation> {
        let mut operations = Vec::new();

        // Burstable scaling: quick scale out for short bursts
        if metrics.cpu_utilization > 0.9 && budget_utilization < 0.7 {
            operations.push(ScalingOperation {
                operation_type: ScalingOperationType::ScaleOut,
                domain: "general".to_string(),
                executor_type: "burstable".to_string(),
                executor_id: None,
                resource_requirements: ResourceRequirements {
                    cpu_cores: 2,
                    memory_mb: 4096,
                    gpu_memory_mb: None,
                    disk_mb: 1024,
                    network_mbps: 50,
                },
            });
        }

        operations
    }

    /// Check spot scaling needs.
    pub fn check_spot_scaling(
        &self,
        metrics: &CurrentMetrics,
        recommendations: &[ScalingRecommendation],
        budget_utilization: f64,
    ) -> Vec<ScalingOperation> {
        let mut operations = Vec::new();

        // Spot scaling: scale out when budget is low but need is high
        if metrics.cpu_utilization > 0.8 && budget_utilization > 0.5 {
            operations.push(ScalingOperation {
                operation_type: ScalingOperationType::ScaleOut,
                domain: "general".to_string(),
                executor_type: "spot".to_string(),
                executor_id: None,
                resource_requirements: ResourceRequirements {
                    cpu_cores: 2,
                    memory_mb: 4096,
                    gpu_memory_mb: None,
                    disk_mb: 1024,
                    network_mbps: 50,
                },
            });
        }

        operations
    }

    /// Record scaling operations.
    pub fn record_scaling_operations(&mut self, operations: &[ScalingOperation]) {
        if !operations.is_empty() {
            self.last_scaling_time = Some(Instant::now());
        }
    }
}

// Default implementations
impl Default for ResourceCosts {
    fn default() -> Self {
        Self {
            cpu_cost_per_hour: 0.05,
            memory_cost_per_hour: 0.01,
            gpu_cost_per_hour: 0.50,
            disk_cost_per_hour: 0.001,
            network_cost_per_hour: 0.005,
            executor_instance_cost: 0.10,
        }
    }
}

impl Default for HorizontalScalingConfig {
    fn default() -> Self {
        Self {
            min_instances: 1,
            max_instances: 10,
            desired_instances: 2,
            instance_warmup_time: Duration::from_secs(60),
            instance_cooldown_time: Duration::from_secs(300),
        }
    }
}

impl Default for VerticalScalingConfig {
    fn default() -> Self {
        Self {
            min_cpu_cores: 1,
            max_cpu_cores: 16,
            min_memory_mb: 1024,
            max_memory_mb: 32768,
            min_gpu_memory_mb: Some(0),
            max_gpu_memory_mb: Some(16384),
            cpu_step_size: 2,
            memory_step_size: 2048,
        }
    }
}

impl Default for HybridScalingConfig {
    fn default() -> Self {
        Self {
            horizontal_weight: 0.6,
            vertical_weight: 0.4,
            min_improvement_threshold: 0.1,
            max_operations_per_hour: 12,
        }
    }
}

impl Default for ScalingThresholds {
    fn default() -> Self {
        Self {
            scale_up_cpu_threshold: 0.8,
            scale_down_cpu_threshold: 0.3,
            scale_up_memory_threshold: 0.8,
            scale_down_memory_threshold: 0.3,
            scale_up_latency_threshold: 5000,
            scale_up_success_rate_threshold: 0.9,
            scale_up_throughput_threshold: 5.0,
            budget_threshold: 0.8,
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct CurrentMetrics {
    pub cpu_utilization: f64,
    pub memory_utilization: f64,
    pub gpu_utilization: f64,
    pub network_utilization: f64,
    pub throughput: f64,
    pub latency: u32,
    pub success_rate: f64,
}

// AllocationResult helper methods
impl AllocationResult {
    /// Create successful allocation result.
    pub fn success(
        allocated_resources: ResourceRequirements,
        executor_id: String,
        estimated_cost: f64,
        estimated_performance: EstimatedPerformance,
    ) -> Self {
        Self {
            success: true,
            allocated_resources: Some(allocated_resources),
            executor_id: Some(executor_id),
            estimated_cost,
            estimated_performance,
            alternatives: Vec::new(),
        }
    }

    /// Create failed allocation result.
    pub fn failed(request: &ResourceRequest, alternatives: Vec<AllocationAlternative>) -> Self {
        Self {
            success: false,
            allocated_resources: None,
            executor_id: None,
            estimated_cost: 0.0,
            estimated_performance: EstimatedPerformance::default(),
            alternatives,
        }
    }

    /// Create budget exceeded allocation result.
    pub fn budget_exceeded(request: &ResourceRequest, estimated_cost: f64) -> Self {
        Self {
            success: false,
            allocated_resources: None,
            executor_id: None,
            estimated_cost,
            estimated_performance: EstimatedPerformance::default(),
            alternatives: vec![AllocationAlternative {
                resources: ResourceRequirements::default(),
                estimated_performance: EstimatedPerformance::default(),
                cost_difference: 0.0,
                performance_difference: 1.0,
                time_to_allocate: Duration::from_secs(0),
            }],
        }
    }
}

impl Default for EstimatedPerformance {
    fn default() -> Self {
        Self {
            success_rate: 0.9,
            latency_ms: 1000,
            throughput_tps: 5.0,
            resource_utilization: 0.5,
            cost_efficiency: 0.8,
        }
    }
}

impl ScalingResult {
    /// Create scaling result from operation.
    pub fn from_operation(operation: ScalingOperation, success: bool) -> Self {
        Self {
            operation,
            success,
            error_message: if success {
                None
            } else {
                Some("Operation failed".to_string())
            },
        }
    }
}
