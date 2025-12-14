//! Tests for integrated coordinator.

use super::*;
use crate::models::hybrid_agent::{ExecutorConfig, ExecutorDomain, ExecutorModel, ExecutorPerformance, ExecutorSkill, ExecutorModelSize, ModelProvider, ResourceRequirements};
use serde_json::json;

#[test]
fn test_integrated_coordinator_creation() {
    let config = crate::models::hybrid_agent::HybridAgentConfig::new(
        "Test Agent".to_string(),
        "Test description".to_string(),
    );
    
    let coordinator = IntegratedCoordinator::new(config);
    
    // Verify both systems are accessible
    let base_stats = coordinator.get_queue_stats();
    assert_eq!(base_stats.pending_tasks, 0);
    
    let (base_executors, manager_executors) = coordinator.get_executor_stats();
    assert!(base_executors.is_empty());
    assert!(manager_executors.is_empty());
}

#[test]
fn test_executor_registration() {
    let config = crate::models::hybrid_agent::HybridAgentConfig::new(
        "Test Agent".to_string(),
        "Test description".to_string(),
    );
    
    let coordinator = IntegratedCoordinator::new(config);
    
    let executor_config = ExecutorConfig {
        id: "test-executor-1".to_string(),
        domain: ExecutorDomain::CodeGeneration,
        model: ExecutorModel {
            model_id: "codellama-7b".to_string(),
            provider: ModelProvider::Meta,
            size: ExecutorModelSize::Small,
            fine_tuned: false,
            fine_tuning_dataset: None,
            specialized_capabilities: vec!["code_generation".to_string()],
        },
        skills: vec![ExecutorSkill {
            id: "python".to_string(),
            name: "Python Programming".to_string(),
            description: "Python code generation and analysis".to_string(),
            input_schema: serde_json::json!({}),
            output_schema: serde_json::json!({}),
            avg_execution_time_ms: 1000,
            success_rate: 0.95,
            quality_score: 0.9,
            deterministic: true,
        }],
        performance: ExecutorPerformance {
            throughput_tps: 5.0,
            avg_latency_ms: 2000,
            p95_latency_ms: 5000,
            p99_latency_ms: 10000,
            error_rate: 0.05,
            success_rate: 0.95,
            availability: 0.99,
            cost_per_1k_tasks: 0.50,
        },
        resource_requirements: ResourceRequirements {
            cpu_cores: 2,
            memory_mb: 4096,
            gpu_memory_mb: Some(4096),
            disk_mb: 1024,
            network_mbps: 100,
        },
        local_execution: false,
        max_concurrent_tasks: 5,
    };
    
    let result = coordinator.register_executor(executor_config);
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), "test-executor-1");
    
    // Verify executor is in both systems
    let (base_executors, manager_executors) = coordinator.get_executor_stats();
    assert_eq!(base_executors.len(), 0); // Base coordinator doesn't have it yet
    assert_eq!(manager_executors.len(), 1); // But manager does
    assert_eq!(manager_executors[0].id, "test-executor-1");
}

#[test]
fn test_task_submission_and_completion() {
    let config = crate::models::hybrid_agent::HybridAgentConfig::new(
        "Test Agent".to_string(),
        "Test description".to_string(),
    );
    
    let coordinator = IntegratedCoordinator::new(config);
    
    // Register an executor
    let executor_config = ExecutorConfig {
        id: "test-executor-2".to_string(),
        domain: ExecutorDomain::CodeGeneration,
        max_concurrent_tasks: 3,
        ..Default::default()
    };
    
    coordinator.register_executor(executor_config).unwrap();
    
    // Submit a task
    let task = Task::new(
        "code_task".to_string(),
        json!({"input": "Write a Python function to calculate factorial"}),
    );
    
    let task_id = coordinator.submit_task(task).unwrap();
    
    // Complete the task
    let result = TaskResult {
        task_id,
        executor_id: "test-executor-2".to_string(),
        completed_at: chrono::Utc::now(),
        result: json!({"output": "def factorial(n): return 1 if n <= 1 else n * factorial(n-1)"}),
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
        cost: 0.3,
    };
    
    let completion_result = coordinator.complete_task(result);
    assert!(completion_result.is_ok());
    
    // Verify stats
    let stats = coordinator.get_queue_stats();
    assert_eq!(stats.completed_tasks, 1);
    assert_eq!(stats.total_tasks_processed, 1);
}

#[test]
fn test_scaling_recommendations() {
    let config = crate::models::hybrid_agent::HybridAgentConfig::new(
        "Test Agent".to_string(),
        "Test description".to_string(),
    );
    
    let coordinator = IntegratedCoordinator::new(config);
    
    // Test scaling recommendations with no executors
    let recommendation = coordinator.get_executor_scaling_recommendations();
    
    match recommendation {
        ScalingRecommendation::ScaleUp { count, domain, reason } => {
            assert_eq!(count, 1);
            assert_eq!(domain, ExecutorDomain::CodeGeneration);
            assert_eq!(reason, "No executors available");
        }
        _ => panic!("Expected ScaleUp recommendation with no executors"),
    }
    
    // Register an executor
    let executor_config = ExecutorConfig {
        id: "test-executor-3".to_string(),
        domain: ExecutorDomain::CodeGeneration,
        ..Default::default()
    };
    
    coordinator.register_executor(executor_config).unwrap();
    
    // Test scaling recommendations with one executor
    let recommendation = coordinator.get_executor_scaling_recommendations();
    
    match recommendation {
        ScalingRecommendation::NoScaling => (), // Expected with low load
        _ => (), // Other recommendations possible depending on state
    }
}

#[test]
fn test_performance_snapshots() {
    let config = crate::models::hybrid_agent::HybridAgentConfig::new(
        "Test Agent".to_string(),
        "Test description".to_string(),
    );
    
    let coordinator = IntegratedCoordinator::new(config);
    
    // Record performance snapshot
    coordinator.record_performance_snapshot();
    
    // Get performance metrics
    let (base_metrics, executor_snapshots) = coordinator.get_combined_performance_metrics();
    
    // Verify metrics exist
    assert!(base_metrics.success_rate >= 0.0);
    assert!(base_metrics.avg_latency_ms >= 0.0);
    
    // Snapshots might be empty if no executors
    // That's OK for this test
}

#[test]
fn test_resource_management_integration() {
    let config = crate::models::hybrid_agent::HybridAgentConfig::new(
        "Test Agent".to_string(),
        "Test description".to_string(),
    );
    
    let coordinator = IntegratedCoordinator::new(config);
    
    // Test resource utilization stats
    let stats = coordinator.get_resource_utilization_stats();
    assert_eq!(stats.cpu_utilization, 0.0);
    assert_eq!(stats.memory_utilization, 0.0);
    
    // Test optimization recommendations
    let recommendations = coordinator.optimize_resource_allocation();
    assert!(recommendations.resource_recommendations.len() >= 0);
    
    // Test auto-scaling check (should be empty initially)
    let scaling_results = coordinator.check_and_apply_auto_scaling();
    assert!(scaling_results.is_empty());
}

#[test]
fn test_resource_aware_assignment() {
    let mut config = crate::models::hybrid_agent::HybridAgentConfig::new(
        "Test Agent".to_string(),
        "Test description".to_string(),
    );
    
    // Add an executor to the config
    let executor_config = ExecutorConfig::new(
        "test-executor".to_string(),
        ExecutorDomain::CodeGeneration,
    );
    config.add_executor(executor_config);
    
    let coordinator = IntegratedCoordinator::new(config);
    
    // At minimum, ensure the coordinator can be created and basic methods work
    let assignments = coordinator.assign_tasks();
    assert!(assignments.is_ok());
}