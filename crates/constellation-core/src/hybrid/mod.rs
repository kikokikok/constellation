//! Hybrid agent architecture implementation.
//!
//! Combines LLM strategists (large language models for planning) with
//! SLM executors (smaller, specialized models for execution).

pub mod coordinator;
pub mod executor_manager;
pub mod fallback_manager;
pub mod integrated_coordinator;
pub mod performance_monitor;
pub mod resource_manager;

#[cfg(test)]
mod coordinator_test;

#[cfg(test)]
mod executor_manager_test;

#[cfg(test)]
mod integrated_coordinator_test;

#[cfg(test)]
mod resource_manager_test;

#[cfg(test)]
mod performance_monitor_test;

#[cfg(test)]
mod fallback_manager_test;

pub use coordinator::{
    ExecutorStats, LlmStrategistCoordinator, PerformanceMetrics, QueueStats, ResourceAllocation,
    ResourceRequirements, ResourceUsage, Task, TaskAssignment, TaskResult, TaskStatus,
};

pub use executor_manager::{
    ExecutorHealth, ExecutorHealthUpdate, ExecutorLifecycleState, ExecutorMatch,
    ExecutorSpecialization, LoadBalancingStrategy, ManagedExecutor, MatchingCriteria,
    PerformanceSnapshot, ResourceConstraints, ScalingRecommendation, ScalingThresholds,
    SlmExecutorManager,
};

pub use integrated_coordinator::IntegratedCoordinator;

pub use performance_monitor::{
    Alert, AlertLevel, AlertSubscriber, AlertThresholds, AlertType, CostMetric, CostRecord,
    CostTrigger, NotificationChannel, OptimizationAlgorithm, OptimizationEngine,
    OptimizationResult, PerformanceHistory, PerformanceImpact, PerformanceImprovement,
    PerformanceMetric, PerformanceMonitor, PerformanceTrends, PerformanceTrigger, PredictionModel,
    PredictiveScaler, RecurrencePattern, ResourceCost, ResourceUtilization, ScalingEvent,
    ScalingTrigger, ScalingType, ScheduleTrigger, TrainingSample,
};

pub use resource_manager::{
    AllocationAlternative, AllocationResult, BudgetTracker, EstimatedPerformance,
    HorizontalScalingConfig, HybridScalingConfig, PerformanceHistory as ResourcePerformanceHistory,
    QualityRequirements, ResourceCosts, ResourceManager, ResourcePool, ResourceRequest,
    ScalingController, ScalingOperation, ScalingOperationType, ScalingResult, UtilizationBreakdown,
    UtilizationStats, VerticalScalingConfig,
};

pub use fallback_manager::{
    Bulkhead, BulkheadConfig, CircuitBreaker, CircuitBreakerConfig, CircuitBreakerState,
    DegradationLevel, FallbackManager, GracefulDegradation, RetryConfig, RetryMechanism,
};
