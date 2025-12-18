//! Tests for SLM executor management system.

use crate::hybrid::executor_manager::*;
use crate::models::hybrid_agent::{
    ExecutorConfig, ExecutorDomain, ExecutorModel, ExecutorModelSize, ExecutorPerformance,
    ExecutorSkill, ModelProvider, ResourceRequirements,
};

#[test]
fn test_executor_registration() {
    let manager = SlmExecutorManager::new();

    let config = ExecutorConfig {
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

    let result = manager.register_executor(config);
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), "test-executor-1");

    // Test duplicate registration
    let config2 = ExecutorConfig {
        id: "test-executor-1".to_string(),
        domain: ExecutorDomain::DataAnalysis,
        model: ExecutorModel {
            model_id: "llama-3-8b".to_string(),
            provider: ModelProvider::Meta,
            size: ExecutorModelSize::Small,
            fine_tuned: false,
            fine_tuning_dataset: None,
            specialized_capabilities: vec!["data_analysis".to_string()],
        },
        skills: vec![],
        performance: ExecutorPerformance::default(),
        resource_requirements: ResourceRequirements::default(),
        local_execution: false,
        max_concurrent_tasks: 3,
    };

    let result2 = manager.register_executor(config2);
    assert!(result2.is_err());
    assert!(result2.unwrap_err().contains("already registered"));
}

#[test]
fn test_executor_state_management() {
    let manager = SlmExecutorManager::new();

    let config = ExecutorConfig {
        id: "test-executor-2".to_string(),
        domain: ExecutorDomain::DataAnalysis,
        ..Default::default()
    };

    manager.register_executor(config).unwrap();

    // Test state transitions
    assert!(
        manager
            .update_executor_state("test-executor-2", ExecutorLifecycleState::Ready)
            .is_ok()
    );
    assert!(
        manager
            .update_executor_state("test-executor-2", ExecutorLifecycleState::Busy)
            .is_ok()
    );
    assert!(
        manager
            .update_executor_state("test-executor-2", ExecutorLifecycleState::Draining)
            .is_ok()
    );

    // Test invalid executor
    let result = manager.update_executor_state("nonexistent", ExecutorLifecycleState::Ready);
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("not found"));
}

#[test]
fn test_executor_health_monitoring() {
    let manager = SlmExecutorManager::new();

    let config = ExecutorConfig {
        id: "test-executor-3".to_string(),
        domain: ExecutorDomain::Research,
        ..Default::default()
    };

    manager.register_executor(config).unwrap();
    manager
        .update_executor_state("test-executor-3", ExecutorLifecycleState::Ready)
        .unwrap();

    // Update health with successful task
    let health_update = ExecutorHealthUpdate {
        cpu_utilization: Some(0.5),
        memory_utilization: Some(0.6),
        gpu_utilization: Some(0.7),
        network_latency_ms: Some(100),
        error_rate: Some(0.05),
        task_succeeded: true,
        task_failed: false,
    };

    assert!(
        manager
            .update_executor_health("test-executor-3", health_update)
            .is_ok()
    );

    let executor = manager.get_executor("test-executor-3").unwrap();
    assert_eq!(executor.health.cpu_utilization, 0.5);
    assert_eq!(executor.health.memory_utilization, 0.6);
    assert_eq!(executor.health.gpu_utilization, Some(0.7));
    assert_eq!(executor.health.network_latency_ms, 100);
    assert_eq!(executor.health.error_rate, 0.05);
    assert_eq!(executor.health.consecutive_failures, 0);

    // Update health with failed tasks to trigger failure state
    for _ in 0..3 {
        let health_update = ExecutorHealthUpdate {
            cpu_utilization: None,
            memory_utilization: None,
            gpu_utilization: None,
            network_latency_ms: None,
            error_rate: None,
            task_succeeded: false,
            task_failed: true,
        };
        manager
            .update_executor_health("test-executor-3", health_update)
            .unwrap();
    }

    let executor = manager.get_executor("test-executor-3").unwrap();
    assert_eq!(executor.lifecycle_state, ExecutorLifecycleState::Failed);
}

#[test]
fn test_executor_load_management() {
    let manager = SlmExecutorManager::new();

    let config = ExecutorConfig {
        id: "test-executor-4".to_string(),
        domain: ExecutorDomain::Writing,
        max_concurrent_tasks: 3,
        ..Default::default()
    };

    manager.register_executor(config).unwrap();
    manager
        .update_executor_state("test-executor-4", ExecutorLifecycleState::Ready)
        .unwrap();

    // Assign tasks
    for i in 0..3 {
        assert!(manager.assign_task("test-executor-4").is_ok());

        let executor = manager.get_executor("test-executor-4").unwrap();
        assert_eq!(executor.active_task_count, i + 1);
        assert_eq!(executor.current_load, (i + 1) as f64 / 3.0);
    }

    // Test capacity limit
    let result = manager.assign_task("test-executor-4");
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("maximum capacity"));

    // Complete tasks
    assert!(
        manager
            .update_executor_load("test-executor-4", true, 1000)
            .is_ok()
    );
    let executor = manager.get_executor("test-executor-4").unwrap();
    assert_eq!(executor.active_task_count, 2);
    assert_eq!(executor.total_tasks_completed, 1);
    assert_eq!(executor.total_execution_time_ms, 1000);
}

#[test]
fn test_executor_matching() {
    let manager = SlmExecutorManager::new();

    // Register multiple executors with different specializations
    let code_executor = ExecutorConfig {
        id: "code-executor".to_string(),
        domain: ExecutorDomain::CodeGeneration,
        skills: vec![
            ExecutorSkill {
                id: "python".to_string(),
                name: "Python".to_string(),
                description: "Python programming".to_string(),
                input_schema: serde_json::json!({}),
                output_schema: serde_json::json!({}),
                avg_execution_time_ms: 1000,
                success_rate: 0.95,
                quality_score: 0.9,
                deterministic: true,
            },
            ExecutorSkill {
                id: "rust".to_string(),
                name: "Rust".to_string(),
                description: "Rust programming".to_string(),
                input_schema: serde_json::json!({}),
                output_schema: serde_json::json!({}),
                avg_execution_time_ms: 1500,
                success_rate: 0.9,
                quality_score: 0.85,
                deterministic: true,
            },
        ],
        performance: ExecutorPerformance {
            avg_latency_ms: 2000,
            success_rate: 0.95,
            availability: 0.99,
            cost_per_1k_tasks: 0.50,
            ..Default::default()
        },
        max_concurrent_tasks: 5,
        ..Default::default()
    };

    let data_executor = ExecutorConfig {
        id: "data-executor".to_string(),
        domain: ExecutorDomain::DataAnalysis,
        skills: vec![ExecutorSkill {
            id: "data-analysis".to_string(),
            name: "Data Analysis".to_string(),
            description: "Data analysis and visualization".to_string(),
            input_schema: serde_json::json!({}),
            output_schema: serde_json::json!({}),
            avg_execution_time_ms: 2000,
            success_rate: 0.9,
            quality_score: 0.85,
            deterministic: true,
        }],
        performance: ExecutorPerformance {
            avg_latency_ms: 3000,
            success_rate: 0.9,
            availability: 0.98,
            cost_per_1k_tasks: 0.40,
            ..Default::default()
        },
        max_concurrent_tasks: 3,
        ..Default::default()
    };

    manager.register_executor(code_executor).unwrap();
    manager.register_executor(data_executor).unwrap();

    manager
        .update_executor_state("code-executor", ExecutorLifecycleState::Ready)
        .unwrap();
    manager
        .update_executor_state("data-executor", ExecutorLifecycleState::Ready)
        .unwrap();

    // Test matching for code generation task
    let criteria = MatchingCriteria {
        required_domain: Some(ExecutorDomain::CodeGeneration),
        required_skills: vec!["python".to_string()],
        min_skill_proficiency: 0.8,
        max_latency_ms: 5000,
        min_success_rate: 0.9,
        min_quality_score: 0.8,
        max_cost_per_task: 1.0,
        resource_constraints: ResourceConstraints {
            max_cpu_cores: 4,
            max_memory_mb: 8192,
            max_gpu_memory_mb: Some(8192),
            max_network_mbps: 100,
        },
        priority: 50,
    };

    let result = manager.find_best_executor(&criteria);
    assert!(result.is_ok());
    let best_match = result.unwrap();
    assert_eq!(best_match.executor_id, "code-executor");
    assert!(best_match.match_score > 0.0);
    assert!(best_match.domain_match_score > 0.0);
    assert!(best_match.skill_match_score > 0.0);

    // Test matching with no suitable executor
    let criteria = MatchingCriteria {
        required_domain: Some(ExecutorDomain::Medical),
        required_skills: vec!["medical-diagnosis".to_string()],
        min_skill_proficiency: 0.9,
        max_latency_ms: 1000,
        min_success_rate: 0.99,
        min_quality_score: 0.95,
        max_cost_per_task: 0.1,
        resource_constraints: ResourceConstraints::default(),
        priority: 100,
    };

    let result = manager.find_best_executor(&criteria);
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("No suitable executor"));
}

#[test]
fn test_scaling_recommendations() {
    let manager = SlmExecutorManager::new();

    // Test no executors
    let recommendation = manager.check_scaling_recommendations();
    match recommendation {
        ScalingRecommendation::ScaleUp {
            count,
            domain,
            reason,
        } => {
            assert_eq!(count, 1);
            assert_eq!(domain, ExecutorDomain::CodeGeneration);
            assert_eq!(reason, "No executors available");
        }
        _ => panic!("Expected ScaleUp recommendation"),
    }

    // Register an executor
    let config = ExecutorConfig {
        id: "test-executor-5".to_string(),
        domain: ExecutorDomain::CodeGeneration,
        max_concurrent_tasks: 5,
        ..Default::default()
    };

    manager.register_executor(config).unwrap();
    manager
        .update_executor_state("test-executor-5", ExecutorLifecycleState::Ready)
        .unwrap();

    // Test no scaling needed
    let recommendation = manager.check_scaling_recommendations();
    match recommendation {
        ScalingRecommendation::NoScaling => (),
        _ => panic!("Expected NoScaling recommendation"),
    }
}

#[test]
fn test_performance_snapshots() {
    let manager = SlmExecutorManager::new();

    let config = ExecutorConfig {
        id: "test-executor-6".to_string(),
        domain: ExecutorDomain::CodeGeneration,
        max_concurrent_tasks: 5,
        ..Default::default()
    };

    manager.register_executor(config).unwrap();
    manager
        .update_executor_state("test-executor-6", ExecutorLifecycleState::Ready)
        .unwrap();

    // Record performance snapshot
    manager.record_performance_snapshot();

    // Get performance history
    let history = manager.get_performance_history(None);
    assert_eq!(history.len(), 1);

    let snapshot = &history[0];
    assert!(snapshot.avg_load >= 0.0);
    assert!(snapshot.avg_success_rate >= 0.0);
    assert!(snapshot.avg_quality_score >= 0.0);
    assert!(snapshot.avg_latency_ms >= 0.0);

    // Test with limit
    for _ in 0..5 {
        manager.record_performance_snapshot();
    }

    let limited_history = manager.get_performance_history(Some(3));
    assert_eq!(limited_history.len(), 3);
}
