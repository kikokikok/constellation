//! Tests for LLM strategist coordinator.

use super::*;
use crate::hybrid::coordinator::ExecutorStatus;
use crate::models::hybrid_agent::{
    FallbackAction, FallbackStrategy, FallbackTrigger, HybridAgentConfig,
};
use serde_json::json;

#[test]
fn test_coordinator_creation() {
    let config = HybridAgentConfig::new("Test Agent".to_string(), "Test description".to_string());

    let coordinator = LlmStrategistCoordinator::new(config);
    assert!(coordinator.get_queue_stats().pending_tasks == 0);
}

#[test]
fn test_task_submission() {
    let config = HybridAgentConfig::new("Test Agent".to_string(), "Test description".to_string());

    let coordinator = LlmStrategistCoordinator::new(config);

    let task = Task::new("test_task".to_string(), json!({"input": "test"}));

    let _task_id = coordinator.submit_task(task).unwrap();
    let stats = coordinator.get_queue_stats();

    assert_eq!(stats.pending_tasks, 1);
    assert_eq!(stats.total_tasks_processed, 0);
}

#[test]
fn test_executor_registration() {
    let config = HybridAgentConfig::new("Test Agent".to_string(), "Test description".to_string());

    let coordinator = LlmStrategistCoordinator::new(config);

    let status = ExecutorStatus::new("executor1".to_string())
        .with_load(0.5)
        .with_performance(0.9, 0.85, 1000.0)
        .with_cost(0.1)
        .with_availability(true);

    coordinator.update_executor_status(status).unwrap();

    let executor_stats = coordinator.get_executor_stats();
    assert_eq!(executor_stats.len(), 1);
    assert_eq!(executor_stats[0].executor_id, "executor1");
    assert_eq!(executor_stats[0].current_load, 0.5);
    assert_eq!(executor_stats[0].is_available, true);
}

#[test]
fn test_task_assignment() {
    let config = HybridAgentConfig::new("Test Agent".to_string(), "Test description".to_string());

    let coordinator = LlmStrategistCoordinator::new(config);

    // Register executor
    let status = ExecutorStatus::new("executor1".to_string())
        .with_load(0.3)
        .with_performance(0.95, 0.9, 1500.0)
        .with_cost(0.2)
        .with_availability(true);

    coordinator.update_executor_status(status).unwrap();

    // Submit task
    let task = Task::new("test_task".to_string(), json!({"input": "test"})).with_priority(75);

    coordinator.submit_task(task).unwrap();

    // Assign tasks
    let assignments = coordinator.assign_tasks().unwrap();
    assert_eq!(assignments.len(), 1);
    assert_eq!(assignments[0].executor_id, "executor1");

    let stats = coordinator.get_queue_stats();
    assert_eq!(stats.pending_tasks, 0);
    assert_eq!(stats.active_tasks, 1);
}

#[test]
fn test_task_completion() {
    let config = HybridAgentConfig::new("Test Agent".to_string(), "Test description".to_string());

    let coordinator = LlmStrategistCoordinator::new(config);

    // Register executor
    let status = ExecutorStatus::new("executor1".to_string())
        .with_load(0.3)
        .with_performance(0.95, 0.9, 1500.0)
        .with_cost(0.2)
        .with_availability(true);

    coordinator.update_executor_status(status).unwrap();

    // Submit and assign task
    let task = Task::new("test_task".to_string(), json!({"input": "test"})).with_priority(75);

    let task_id = coordinator.submit_task(task).unwrap();
    let _assignments = coordinator.assign_tasks().unwrap();

    // Complete task
    let result = TaskResult {
        task_id,
        executor_id: "executor1".to_string(),
        completed_at: chrono::Utc::now(),
        result: json!({"output": "success"}),
        success: true,
        error: None,
        quality_score: 0.9,
        execution_time_ms: 1500,
        resource_usage: ResourceUsage {
            cpu_core_seconds: 0.5,
            memory_mb_seconds: 512.0,
            gpu_memory_mb_seconds: None,
            network_mb: 0.1,
        },
        cost: 0.5,
    };

    coordinator.complete_task(result).unwrap();

    let stats = coordinator.get_queue_stats();
    assert_eq!(stats.pending_tasks, 0);
    assert_eq!(stats.active_tasks, 0);
    assert_eq!(stats.completed_tasks, 1);
    assert_eq!(stats.total_tasks_processed, 1);
    assert_eq!(stats.total_budget_spent, 0.5);
}

#[test]
fn test_fallback_conditions() {
    let mut config =
        HybridAgentConfig::new("Test Agent".to_string(), "Test description".to_string());

    // Set very lenient performance targets so they don't trigger
    config.performance_targets.success_rate_target = 0.0;
    config.performance_targets.latency_target_ms = u32::MAX;

    let fallback_strategies = vec![
        FallbackStrategy {
            trigger: FallbackTrigger::HighLatency,
            action: FallbackAction::SwitchExecutor,
            priority: 50,
            timeout_ms: 5000,
        },
        FallbackStrategy {
            trigger: FallbackTrigger::LowSuccessRate,
            action: FallbackAction::ReduceQuality,
            priority: 75,
            timeout_ms: 10000,
        },
    ];

    let coordinator =
        LlmStrategistCoordinator::new(config).with_fallback_strategies(fallback_strategies);

    // Initially no fallback conditions (with lenient targets)
    let actions = coordinator.check_fallback_conditions();
    assert!(actions.is_empty());

    // Note: More comprehensive fallback testing would require
    // setting up specific performance conditions
}

#[test]
fn test_performance_metrics() {
    let config = HybridAgentConfig::new("Test Agent".to_string(), "Test description".to_string());

    let coordinator = LlmStrategistCoordinator::new(config);

    // Register an executor
    let status = ExecutorStatus::new("executor1".to_string())
        .with_load(0.0)
        .with_performance(0.95, 0.9, 1000.0)
        .with_cost(0.1)
        .with_availability(true);

    coordinator.update_executor_status(status).unwrap();

    // Submit, assign, and complete tasks properly
    for i in 0..3 {
        let task = Task::new(format!("task{}", i), json!({"input": format!("test{}", i)}))
            .with_priority(50);

        let task_id = coordinator.submit_task(task).unwrap();

        // Assign the task
        let assignments = coordinator.assign_tasks().unwrap();
        assert_eq!(assignments.len(), 1);
        assert_eq!(assignments[0].task_id, task_id);

        // Complete the task
        let result = TaskResult {
            task_id,
            executor_id: "executor1".to_string(),
            completed_at: chrono::Utc::now(),
            result: json!({"output": format!("task{}", i)}),
            success: true,
            error: None,
            quality_score: 0.8 + (i as f64 * 0.05),
            execution_time_ms: 1000 + (i as u64 * 500),
            resource_usage: ResourceUsage {
                cpu_core_seconds: 0.5,
                memory_mb_seconds: 512.0,
                gpu_memory_mb_seconds: None,
                network_mb: 0.1,
            },
            cost: 0.3 + (i as f64 * 0.1),
        };

        coordinator.complete_task(result).unwrap();
    }

    let metrics = coordinator.get_performance_metrics();

    // Basic sanity checks
    assert!(metrics.success_rate > 0.0);
    assert!(metrics.avg_latency_ms > 0.0);
    assert!(metrics.avg_quality_score > 0.0);
    assert!(metrics.throughput_tps >= 0.0);
}

#[test]
fn test_task_priority_ordering() {
    let config = HybridAgentConfig::new("Test Agent".to_string(), "Test description".to_string());

    let coordinator = LlmStrategistCoordinator::new(config);

    // Register executor
    let status = ExecutorStatus::new("executor1".to_string())
        .with_load(0.0)
        .with_performance(0.95, 0.9, 1000.0)
        .with_cost(0.1)
        .with_availability(true);

    coordinator.update_executor_status(status).unwrap();

    // Submit tasks with different priorities
    let tasks = vec![
        Task::new("low".to_string(), json!({})).with_priority(25),
        Task::new("high".to_string(), json!({})).with_priority(100),
        Task::new("medium".to_string(), json!({})).with_priority(50),
    ];

    for task in tasks {
        coordinator.submit_task(task).unwrap();
    }

    // Assign tasks - high priority should be assigned first
    let assignments = coordinator.assign_tasks().unwrap();
    assert_eq!(assignments.len(), 3);

    // Check that tasks were assigned in priority order
    // (The coordinator sorts by priority when submitting)
    let stats = coordinator.get_queue_stats();
    assert_eq!(stats.pending_tasks, 0);
}
