//! Integrated coordinator combining LLM strategist coordination with SLM executor management.

use crate::hybrid::coordinator::{
    ExecutorStats, LlmStrategistCoordinator, PerformanceMetrics, QueueStats, Task, TaskAssignment,
    TaskResult,
};
use crate::hybrid::executor_manager::{
    ExecutorHealthUpdate, ExecutorLifecycleState, ExecutorMatch, LoadBalancingStrategy,
    MatchingCriteria, ResourceConstraints, ScalingRecommendation, ScalingThresholds,
    SlmExecutorManager,
};
use crate::hybrid::fallback_manager::FallbackManager;
use crate::hybrid::performance_monitor::PerformanceMonitor;
use crate::hybrid::resource_manager::{
    AllocationResult, QualityRequirements, ResourceManager, ResourceRequest, ScalingResult,
};
use crate::models::hybrid_agent::{ExecutorConfig, ExecutorDomain, HybridAgentConfig};
use serde_json::Value;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use uuid::Uuid;

/// Integrated coordinator combining LLM strategist and SLM executor management.
#[derive(Debug)]
pub struct IntegratedCoordinator {
    /// Base LLM strategist coordinator.
    base_coordinator: LlmStrategistCoordinator,

    /// SLM executor manager.
    executor_manager: SlmExecutorManager,

    /// Resource manager for dynamic allocation and scaling.
    resource_manager: ResourceManager,

    /// Performance monitor for real-time monitoring and optimization.
    performance_monitor: PerformanceMonitor,

    /// Fallback manager for fault tolerance and graceful degradation.
    fallback_manager: FallbackManager,

    /// Mapping between executor IDs and their configurations.
    executor_configs: Arc<Mutex<Vec<ExecutorConfig>>>,

    /// Whether to use advanced executor matching.
    use_advanced_matching: bool,

    /// Whether to use resource-aware task assignment.
    use_resource_aware_assignment: bool,
}

impl IntegratedCoordinator {
    /// Create a new integrated coordinator.
    pub fn new(config: HybridAgentConfig) -> Self {
        let base_coordinator = LlmStrategistCoordinator::new(config.clone());

        // Create executor manager with configuration from agent config
        let scaling_thresholds = ScalingThresholds {
            scale_up_load_threshold: 0.8,
            scale_down_load_threshold: 0.3,
            scale_up_success_threshold: 0.9,
            scale_up_latency_threshold_ms: 5000,
            min_executors: 1,
            max_executors: 100,
            scaling_cooldown_secs: 300,
        };

        let executor_manager = SlmExecutorManager::with_config(
            30, // health_monitoring_interval_secs
            3,  // max_consecutive_failures
            5,  // max_recovery_attempts
            LoadBalancingStrategy::LeastLoaded,
            scaling_thresholds,
        );

        // Create resource manager with allocation config from agent
        let resource_manager = ResourceManager::new(config.resource_allocation.clone());

        // Create performance monitor with performance targets from agent config
        let performance_monitor = PerformanceMonitor::new(config.performance_targets.clone());

        // Create fallback manager with strategies from agent config
        let fallback_manager = FallbackManager::new()
            .with_strategies(config.fallback_strategies.clone())
            .with_performance_monitor(Arc::new(performance_monitor.clone()));

        Self {
            base_coordinator,
            executor_manager,
            resource_manager,
            performance_monitor,
            fallback_manager,
            executor_configs: Arc::new(Mutex::new(Vec::new())),
            use_advanced_matching: true,
            use_resource_aware_assignment: true,
        }
    }

    /// Create with custom executor manager configuration.
    pub fn with_executor_config(
        config: HybridAgentConfig,
        health_monitoring_interval_secs: u64,
        max_consecutive_failures: u32,
        max_recovery_attempts: u32,
        load_balancing_strategy: LoadBalancingStrategy,
        scaling_thresholds: ScalingThresholds,
    ) -> Self {
        let base_coordinator = LlmStrategistCoordinator::new(config.clone());

        let executor_manager = SlmExecutorManager::with_config(
            health_monitoring_interval_secs,
            max_consecutive_failures,
            max_recovery_attempts,
            load_balancing_strategy,
            scaling_thresholds,
        );

        // Create resource manager with allocation config from agent
        let resource_manager = ResourceManager::new(config.resource_allocation.clone());

        // Create performance monitor with performance targets from agent config
        let performance_monitor = PerformanceMonitor::new(config.performance_targets.clone());

        // Create fallback manager with strategies from agent config
        let fallback_manager = FallbackManager::new()
            .with_strategies(config.fallback_strategies.clone())
            .with_performance_monitor(Arc::new(performance_monitor.clone()));

        Self {
            base_coordinator,
            executor_manager,
            resource_manager,
            performance_monitor,
            fallback_manager,
            executor_configs: Arc::new(Mutex::new(Vec::new())),
            use_advanced_matching: true,
            use_resource_aware_assignment: true,
        }
    }

    /// Register an executor with both systems.
    pub fn register_executor(&self, config: ExecutorConfig) -> Result<String, String> {
        // Register with executor manager
        let executor_id = self.executor_manager.register_executor(config.clone())?;

        // Update executor configs
        let mut configs = self.executor_configs.lock().unwrap();
        configs.push(config);

        // Set executor to ready state
        self.executor_manager
            .update_executor_state(&executor_id, ExecutorLifecycleState::Ready)?;

        Ok(executor_id)
    }

    /// Update executor status (compatible with base coordinator interface).
    pub fn update_executor_status(
        &self,
        status: crate::hybrid::coordinator::ExecutorStatus,
    ) -> Result<(), String> {
        // Clone status since we need it after passing to base coordinator
        let executor_id = status.executor_id.clone();
        let avg_latency_ms = status.avg_latency_ms;
        let success_rate = status.success_rate;

        // Update base coordinator
        self.base_coordinator.update_executor_status(status)?;

        // Also update executor manager health
        let health_update = ExecutorHealthUpdate {
            cpu_utilization: None, // Would need to extract from status if available
            memory_utilization: None,
            gpu_utilization: None,
            network_latency_ms: Some(avg_latency_ms as u32),
            error_rate: Some(1.0 - success_rate),
            task_succeeded: false,
            task_failed: false,
        };

        // Try to update health if executor exists in manager
        let _ = self
            .executor_manager
            .update_executor_health(&executor_id, health_update);

        Ok(())
    }

    /// Submit a task for execution.
    pub fn submit_task(&self, task: Task) -> Result<Uuid, String> {
        self.base_coordinator.submit_task(task)
    }

    /// Assign tasks to executors using advanced matching if enabled.
    pub fn assign_tasks(&self) -> Result<Vec<TaskAssignment>, String> {
        if !self.use_advanced_matching {
            return self.base_coordinator.assign_tasks();
        }

        // Get tasks from queue
        let queue_stats = self.base_coordinator.get_queue_stats();
        if queue_stats.pending_tasks == 0 {
            return Ok(Vec::new());
        }

        // Use resource-aware assignment if enabled
        if self.use_resource_aware_assignment {
            self.assign_tasks_with_resource_management()
        } else {
            // Fall back to base coordinator
            self.base_coordinator.assign_tasks()
        }
    }

    /// Assign tasks with resource management.
    fn assign_tasks_with_resource_management(&self) -> Result<Vec<TaskAssignment>, String> {
        // Simplified implementation for now
        // In production, this would integrate with the base coordinator's task queue
        // and use resource manager for allocation decisions
        self.base_coordinator.assign_tasks()
    }

    /// Convert task to resource request.
    fn task_to_resource_request(&self, task: &Task) -> Result<ResourceRequest, String> {
        // Simplified conversion
        let domain = self
            .infer_domain_from_task_type(&task.task_type)
            .map(|d| format!("{d:?}"))
            .unwrap_or_else(|| "general".to_string());

        Ok(ResourceRequest {
            request_id: task.id.to_string(),
            priority: task.priority,
            requirements: crate::models::hybrid_agent::ResourceRequirements {
                cpu_cores: 2,        // Default
                memory_mb: 4096,     // Default
                gpu_memory_mb: None, // Default
                disk_mb: 1024,       // Default
                network_mbps: 100,   // Default
            },
            estimated_duration: Duration::from_secs(60), // Default
            task_type: task.task_type.clone(),
            domain,
            quality_requirements: QualityRequirements {
                min_success_rate: 0.9,          // Default
                max_latency_ms: 5000,           // Default
                min_quality_score: 0.8,         // Default
                availability_requirement: 0.95, // Default
            },
        })
    }

    /// Complete a task and update executor metrics.
    pub fn complete_task(&self, result: TaskResult) -> Result<(), String> {
        // Update base coordinator
        self.base_coordinator.complete_task(result.clone())?;

        // Update executor manager
        let task_completed = result.success;
        let execution_time_ms = result.execution_time_ms;

        // Update executor load and health
        let health_update = ExecutorHealthUpdate {
            cpu_utilization: None,
            memory_utilization: None,
            gpu_utilization: None,
            network_latency_ms: None,
            error_rate: None,
            task_succeeded: result.success,
            task_failed: !result.success,
        };

        // Update executor health
        let _ = self
            .executor_manager
            .update_executor_health(&result.executor_id, health_update);

        // Update executor load
        self.executor_manager.update_executor_load(
            &result.executor_id,
            task_completed,
            execution_time_ms,
        )?;

        Ok(())
    }

    /// Find the best executor for a task using advanced matching.
    pub fn find_best_executor_for_task(&self, task: &Task) -> Result<ExecutorMatch, String> {
        // Convert task requirements to matching criteria
        let criteria = self.task_to_matching_criteria(task);

        // Find best executor
        self.executor_manager.find_best_executor(&criteria)
    }

    /// Convert task to matching criteria.
    fn task_to_matching_criteria(&self, task: &Task) -> MatchingCriteria {
        // Extract domain from task type (simplified)
        let required_domain = self.infer_domain_from_task_type(&task.task_type);

        // Extract skills from task input (simplified)
        let required_skills = self.extract_skills_from_task(&task.input);

        MatchingCriteria {
            required_domain,
            required_skills,
            min_skill_proficiency: task.quality_requirement,
            max_latency_ms: 10000, // Default
            min_success_rate: 0.9, // Default
            min_quality_score: task.quality_requirement,
            max_cost_per_task: task.budget_allocation,
            resource_constraints: ResourceConstraints {
                max_cpu_cores: task.resource_requirements.min_cpu_cores * 2,
                max_memory_mb: task.resource_requirements.min_memory_mb * 2,
                max_gpu_memory_mb: task.resource_requirements.gpu_memory_mb.map(|v| v * 2),
                max_network_mbps: task.resource_requirements.network_mbps * 2,
            },
            priority: task.priority,
        }
    }

    /// Infer domain from task type (simplified implementation).
    fn infer_domain_from_task_type(&self, task_type: &str) -> Option<ExecutorDomain> {
        let task_type_lower = task_type.to_lowercase();

        if task_type_lower.contains("code") || task_type_lower.contains("program") {
            Some(ExecutorDomain::CodeGeneration)
        } else if task_type_lower.contains("data") || task_type_lower.contains("analy") {
            Some(ExecutorDomain::DataAnalysis)
        } else if task_type_lower.contains("research") {
            Some(ExecutorDomain::Research)
        } else if task_type_lower.contains("write") || task_type_lower.contains("content") {
            Some(ExecutorDomain::Writing)
        } else if task_type_lower.contains("math") {
            Some(ExecutorDomain::Mathematics)
        } else {
            None
        }
    }

    /// Extract skills from task input (simplified implementation).
    fn extract_skills_from_task(&self, input: &Value) -> Vec<String> {
        let mut skills = Vec::new();

        // Simplified: look for skill hints in input
        if let Some(input_str) = input.as_str() {
            let input_lower = input_str.to_lowercase();

            if input_lower.contains("python") {
                skills.push("python".to_string());
            }
            if input_lower.contains("rust") {
                skills.push("rust".to_string());
            }
            if input_lower.contains("javascript") || input_lower.contains("js") {
                skills.push("javascript".to_string());
            }
            if input_lower.contains("sql") || input_lower.contains("database") {
                skills.push("sql".to_string());
            }
            if input_lower.contains("api") {
                skills.push("api".to_string());
            }
        }

        skills
    }

    /// Get executor scaling recommendations.
    pub fn get_executor_scaling_recommendations(&self) -> ScalingRecommendation {
        self.executor_manager.check_scaling_recommendations()
    }

    /// Record performance snapshot.
    pub fn record_performance_snapshot(&self) {
        self.executor_manager.record_performance_snapshot();
    }

    /// Get combined performance metrics from both systems.
    pub fn get_combined_performance_metrics(
        &self,
    ) -> (
        PerformanceMetrics,
        Vec<crate::hybrid::executor_manager::PerformanceSnapshot>,
    ) {
        let base_metrics = self.base_coordinator.get_performance_metrics();
        let executor_snapshots = self.executor_manager.get_performance_history(Some(100));

        (base_metrics, executor_snapshots)
    }

    /// Get queue statistics.
    pub fn get_queue_stats(&self) -> QueueStats {
        self.base_coordinator.get_queue_stats()
    }

    /// Get executor statistics from both systems.
    pub fn get_executor_stats(
        &self,
    ) -> (
        Vec<ExecutorStats>,
        Vec<crate::hybrid::executor_manager::ManagedExecutor>,
    ) {
        let base_stats = self.base_coordinator.get_executor_stats();
        let manager_executors = self.executor_manager.get_all_executors();

        (base_stats, manager_executors)
    }

    /// Check fallback conditions.
    pub fn check_fallback_conditions(&self) -> Vec<crate::models::hybrid_agent::FallbackAction> {
        self.base_coordinator.check_fallback_conditions()
    }

    /// Enable or disable advanced executor matching.
    pub fn set_advanced_matching(&mut self, enabled: bool) {
        self.use_advanced_matching = enabled;
    }

    /// Get the base coordinator for direct access if needed.
    pub fn base_coordinator(&self) -> &LlmStrategistCoordinator {
        &self.base_coordinator
    }

    /// Get the executor manager for direct access if needed.
    pub fn executor_manager(&self) -> &SlmExecutorManager {
        &self.executor_manager
    }

    /// Get the resource manager for direct access if needed.
    pub fn resource_manager(&self) -> &ResourceManager {
        &self.resource_manager
    }

    /// Get the fallback manager for direct access if needed.
    pub fn fallback_manager(&self) -> &FallbackManager {
        &self.fallback_manager
    }

    /// Check and apply auto-scaling.
    pub fn check_and_apply_auto_scaling(&self) -> Vec<ScalingResult> {
        let scaling_operations = self.resource_manager.check_auto_scaling();

        if !scaling_operations.is_empty() {
            self.resource_manager
                .apply_scaling_operations(scaling_operations)
        } else {
            Vec::new()
        }
    }

    /// Get resource utilization statistics.
    pub fn get_resource_utilization_stats(
        &self,
    ) -> crate::hybrid::resource_manager::UtilizationStats {
        self.resource_manager.get_utilization_stats()
    }

    /// Optimize resource allocation.
    pub fn optimize_resource_allocation(
        &self,
    ) -> crate::hybrid::resource_manager::OptimizationRecommendations {
        self.resource_manager.optimize_allocation()
    }

    /// Update resource allocation configuration.
    pub fn update_resource_allocation_config(
        &mut self,
        config: crate::models::hybrid_agent::ResourceAllocation,
    ) {
        self.resource_manager.update_allocation_config(config);
    }

    /// Enable or disable resource-aware task assignment.
    pub fn set_resource_aware_assignment(&mut self, enabled: bool) {
        self.use_resource_aware_assignment = enabled;
    }

    /// Allocate resources for a specific task.
    pub fn allocate_resources_for_task(&self, task_id: Uuid) -> Result<AllocationResult, String> {
        // Simplified implementation
        // In production, we would get the actual task from the coordinator
        let resource_request = ResourceRequest {
            request_id: task_id.to_string(),
            priority: 50,
            requirements: crate::models::hybrid_agent::ResourceRequirements::default(),
            estimated_duration: Duration::from_secs(60),
            task_type: "general".to_string(),
            domain: "general".to_string(),
            quality_requirements: QualityRequirements {
                min_success_rate: 0.9,
                max_latency_ms: 5000,
                min_quality_score: 0.8,
                availability_requirement: 0.95,
            },
        };

        let allocation_result = self.resource_manager.allocate_resources(&resource_request);

        Ok(allocation_result)
    }

    /// Release resources for a completed task.
    pub fn release_resources_for_task(
        &self,
        task_id: Uuid,
        executor_id: &str,
    ) -> Result<(), String> {
        // Simplified implementation
        let requirements = crate::models::hybrid_agent::ResourceRequirements::default();
        self.resource_manager
            .release_resources(executor_id, &requirements);

        Ok(())
    }

    // Performance monitoring methods

    /// Update performance metrics with a completed task result.
    pub fn update_performance_metrics(&self, result: &TaskResult) -> Result<(), String> {
        self.performance_monitor.update_with_task_result(result)
    }

    /// Get current performance metrics.
    pub fn get_performance_metrics(&self) -> PerformanceMetrics {
        self.performance_monitor.get_current_metrics()
    }

    /// Get performance history.
    pub fn get_performance_history(
        &self,
    ) -> crate::hybrid::performance_monitor::PerformanceHistory {
        self.performance_monitor.get_performance_history()
    }

    /// Get active alerts.
    pub fn get_active_alerts(&self) -> Vec<crate::hybrid::performance_monitor::Alert> {
        self.performance_monitor.get_active_alerts()
    }

    /// Get performance trends over a time window.
    pub fn get_performance_trends(
        &self,
        window: Duration,
    ) -> crate::hybrid::performance_monitor::PerformanceTrends {
        self.performance_monitor.get_performance_trends(window)
    }

    /// Get optimization recommendations.
    pub fn get_optimization_recommendations(
        &self,
    ) -> Vec<crate::hybrid::performance_monitor::OptimizationResult> {
        self.performance_monitor.get_optimization_recommendations()
    }

    /// Get scaling recommendations.
    pub fn get_scaling_recommendations(
        &self,
    ) -> Option<crate::hybrid::performance_monitor::ScalingRecommendation> {
        self.performance_monitor.get_scaling_recommendations()
    }

    /// Add an alert subscriber.
    pub fn add_alert_subscriber(
        &self,
        subscriber: crate::hybrid::performance_monitor::AlertSubscriber,
    ) {
        self.performance_monitor.add_subscriber(subscriber);
    }

    /// Remove an alert subscriber.
    pub fn remove_alert_subscriber(&self, subscriber_id: &str) {
        self.performance_monitor.remove_subscriber(subscriber_id);
    }

    /// Acknowledge an alert.
    pub fn acknowledge_alert(&self, alert_id: Uuid) -> Result<(), String> {
        self.performance_monitor.acknowledge_alert(alert_id)
    }

    /// Resolve an alert.
    pub fn resolve_alert(&self, alert_id: Uuid) -> Result<(), String> {
        self.performance_monitor.resolve_alert(alert_id)
    }

    /// Get combined performance dashboard data.
    pub fn get_performance_dashboard(&self) -> PerformanceDashboard {
        let (base_metrics, _) = self.get_combined_performance_metrics();
        let (executor_stats, _) = self.get_executor_stats();

        PerformanceDashboard {
            current_metrics: base_metrics,
            active_alerts: self.get_active_alerts(),
            optimization_recommendations: self.get_optimization_recommendations(),
            scaling_recommendations: self.get_scaling_recommendations(),
            queue_stats: self.get_queue_stats(),
            executor_stats,
            resource_utilization: self.resource_manager.get_utilization_stats(),
            performance_trends: self.get_performance_trends(Duration::from_secs(300)), // Last 5 minutes
        }
    }

    // Fallback management methods

    /// Handle an alert and execute appropriate fallback actions.
    pub async fn handle_alert_with_fallback(
        &self,
        alert: &crate::hybrid::performance_monitor::Alert,
    ) -> Vec<crate::models::hybrid_agent::FallbackAction> {
        self.fallback_manager.handle_alert(alert).await
    }

    /// Execute a task with retry mechanism.
    pub async fn execute_with_retry<F, T, E>(&self, operation: F) -> Result<T, E>
    where
        F: Fn() -> Result<T, E> + Clone,
        E: std::fmt::Debug,
    {
        self.fallback_manager.execute_with_retry(operation).await
    }

    /// Execute a task with timeout.
    pub async fn execute_with_timeout<F, T>(
        &self,
        operation: F,
        timeout: Duration,
    ) -> Result<T, String>
    where
        F: std::future::Future<Output = T> + Send + 'static,
        T: Send + 'static,
    {
        self.fallback_manager
            .execute_with_timeout(operation, timeout)
            .await
    }

    /// Add a circuit breaker for an executor or service.
    pub async fn add_circuit_breaker(
        &self,
        id: String,
        config: Option<crate::hybrid::fallback_manager::CircuitBreakerConfig>,
    ) {
        self.fallback_manager.add_circuit_breaker(id, config).await;
    }

    /// Record success for a circuit breaker.
    pub async fn record_circuit_breaker_success(&self, circuit_breaker_id: &str) {
        self.fallback_manager
            .record_success(circuit_breaker_id)
            .await;
    }

    /// Record failure for a circuit breaker.
    pub async fn record_circuit_breaker_failure(&self, circuit_breaker_id: &str) {
        self.fallback_manager
            .record_failure(circuit_breaker_id)
            .await;
    }

    /// Check if a circuit breaker is available.
    pub async fn is_circuit_breaker_available(&self, circuit_breaker_id: &str) -> bool {
        self.fallback_manager
            .is_circuit_breaker_available(circuit_breaker_id)
            .await
    }

    /// Add a bulkhead for failure isolation.
    pub async fn add_bulkhead(
        &self,
        id: String,
        config: Option<crate::hybrid::fallback_manager::BulkheadConfig>,
    ) {
        self.fallback_manager.add_bulkhead(id, config).await;
    }

    /// Acquire a bulkhead slot.
    pub async fn acquire_bulkhead(&self, bulkhead_id: &str) -> bool {
        self.fallback_manager.acquire_bulkhead(bulkhead_id).await
    }

    /// Release a bulkhead slot.
    pub async fn release_bulkhead(&self, bulkhead_id: &str) {
        self.fallback_manager.release_bulkhead(bulkhead_id).await;
    }

    /// Get current graceful degradation level.
    pub async fn get_current_degradation_level(
        &self,
    ) -> crate::hybrid::fallback_manager::DegradationLevel {
        self.fallback_manager.get_current_degradation_level().await
    }

    /// Get recommended fallback actions for a trigger.
    pub async fn get_recommended_fallback_actions(
        &self,
        trigger: crate::models::hybrid_agent::FallbackTrigger,
    ) -> Vec<crate::models::hybrid_agent::FallbackAction> {
        self.fallback_manager.get_recommended_actions(trigger).await
    }

    /// Check and handle fallback conditions based on current performance.
    pub async fn check_and_handle_fallback_conditions(
        &self,
    ) -> Vec<crate::models::hybrid_agent::FallbackAction> {
        let active_alerts = self.get_active_alerts();
        let mut all_actions = Vec::new();

        for alert in &active_alerts {
            let actions = self.fallback_manager.handle_alert(alert).await;
            all_actions.extend(actions);
        }

        all_actions
    }
}

/// Performance dashboard combining all monitoring data.
#[derive(Debug, Clone)]
pub struct PerformanceDashboard {
    /// Current performance metrics.
    pub current_metrics: PerformanceMetrics,

    /// Active alerts.
    pub active_alerts: Vec<crate::hybrid::performance_monitor::Alert>,

    /// Optimization recommendations.
    pub optimization_recommendations: Vec<crate::hybrid::performance_monitor::OptimizationResult>,

    /// Scaling recommendations.
    pub scaling_recommendations: Option<crate::hybrid::performance_monitor::ScalingRecommendation>,

    /// Queue statistics.
    pub queue_stats: QueueStats,

    /// Executor statistics.
    pub executor_stats: Vec<ExecutorStats>,

    /// Resource utilization statistics.
    pub resource_utilization: crate::hybrid::resource_manager::UtilizationStats,

    /// Performance trends over last 5 minutes.
    pub performance_trends: crate::hybrid::performance_monitor::PerformanceTrends,
}
