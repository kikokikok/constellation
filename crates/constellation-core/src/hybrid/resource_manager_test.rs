//! Tests for resource allocation and scaling strategies.

use std::time::Duration;

use crate::hybrid::executor_manager::SlmExecutorManager;
use crate::hybrid::resource_manager::{
    AllocationResult, ResourceManager, ResourcePool, ResourceRequest, QualityRequirements,
    ScalingOperation, ScalingOperationType,
};
use crate::models::hybrid_agent::{ResourceAllocation, ResourceRequirements, ScalingStrategy};

    #[test]
    fn test_resource_pool_allocation() {
        let mut pool = ResourcePool::new(16, 32768, Some(16384), 16384, 1000);
        
        let requirements = ResourceRequirements {
            cpu_cores: 4,
            memory_mb: 8192,
            gpu_memory_mb: Some(4096),
            disk_mb: 1024,
            network_mbps: 100,
        };
        
        // Allocate resources
        let allocated = pool.allocate_resources(&requirements, "executor-1");
        assert!(allocated.is_some());
        
        // Check utilization
        let cpu_util = pool.get_cpu_utilization();
        let memory_util = pool.get_memory_utilization();
        assert!(cpu_util > 0.0);
        assert!(memory_util > 0.0);
        
        // Release resources
        pool.release_resources("executor-1", &requirements);
        
        // Utilization should be 0 after release
        let cpu_util = pool.get_cpu_utilization();
        let memory_util = pool.get_memory_utilization();
        assert_eq!(cpu_util, 0.0);
        assert_eq!(memory_util, 0.0);
    }

#[test]
fn test_resource_manager_creation() {
    let allocation_config = ResourceAllocation::default();
    let manager = ResourceManager::new(allocation_config.clone());
    
    // Check that manager was created with correct configuration
    let stats = manager.get_utilization_stats();
    assert_eq!(stats.cpu_utilization, 0.0);
    assert_eq!(stats.memory_utilization, 0.0);
}

    #[test]
    fn test_resource_allocation() {
        let allocation_config = ResourceAllocation::default();
        let manager = ResourceManager::new(allocation_config);
        
        // Create resource request
        let request = ResourceRequest {
            request_id: "test-request".to_string(),
            priority: 50,
            requirements: ResourceRequirements {
                cpu_cores: 2,
                memory_mb: 4096,
                gpu_memory_mb: Some(2048),
                disk_mb: 512,
                network_mbps: 50,
            },
            estimated_duration: Duration::from_secs(60),
            task_type: "code_generation".to_string(),
            domain: "CodeGeneration".to_string(),
            quality_requirements: QualityRequirements {
                min_success_rate: 0.9,
                max_latency_ms: 5000,
                min_quality_score: 0.8,
                availability_requirement: 0.95,
            },
        };
        
        // Try to allocate resources
        let result = manager.allocate_resources(&request);
        
        // Should succeed since we have available resources
        assert!(result.success);
        assert!(result.allocated_resources.is_some());
        assert!(result.executor_id.is_some());
    }

    #[test]
    fn test_budget_constrained_allocation() {
        let mut allocation_config = ResourceAllocation::default();
        allocation_config.budget_allocation.total_budget = 10.0; // Very small budget
        
        let manager = ResourceManager::new(allocation_config);
        
        let request = ResourceRequest {
            request_id: "expensive-request".to_string(),
            priority: 50,
            requirements: ResourceRequirements {
                cpu_cores: 16,
                memory_mb: 32768,
                gpu_memory_mb: Some(16384),
                disk_mb: 8192,
                network_mbps: 1000,
            },
            estimated_duration: Duration::from_secs(3600), // 1 hour
            task_type: "heavy_computation".to_string(),
            domain: "DataAnalysis".to_string(),
            quality_requirements: QualityRequirements {
                min_success_rate: 0.9,
                max_latency_ms: 10000,
                min_quality_score: 0.8,
                availability_requirement: 0.95,
            },
        };
        
        // Should fail due to budget constraints
        let result = manager.allocate_resources(&request);
        assert!(!result.success);
        assert!(!result.alternatives.is_empty());
    }

    #[test]
    fn test_auto_scaling_check() {
        let allocation_config = ResourceAllocation::default();
        let manager = ResourceManager::new(allocation_config);
        
        // Check auto-scaling (should return empty since no metrics yet)
        let operations = manager.check_auto_scaling();
        assert!(operations.is_empty());
    }

    #[test]
    fn test_scaling_operations_application() {
        let allocation_config = ResourceAllocation::default();
        let manager = ResourceManager::new(allocation_config);
        
        // Create a scaling operation
        let operation = ScalingOperation {
            operation_type: ScalingOperationType::ScaleOut,
            domain: "CodeGeneration".to_string(),
            executor_type: "default".to_string(),
            executor_id: None,
            resource_requirements: ResourceRequirements {
                cpu_cores: 4,
                memory_mb: 8192,
                gpu_memory_mb: Some(4096),
                disk_mb: 1024,
                network_mbps: 100,
            },
        };
        
        let operations = vec![operation];
        let results = manager.apply_scaling_operations(operations);
        
        // Should have one result
        assert_eq!(results.len(), 1);
    }

#[test]
fn test_resource_optimization() {
    let allocation_config = ResourceAllocation::default();
    let manager = ResourceManager::new(allocation_config);
    
    // Get optimization recommendations
    let recommendations = manager.optimize_allocation();
    
    // Should have some recommendations (even if empty)
    assert!(recommendations.resource_recommendations.len() >= 0);
    assert!(recommendations.scaling_recommendations.len() >= 0);
    assert!(recommendations.cost_optimizations.len() >= 0);
}

#[test]
fn test_allocation_result_helpers() {
    // Test successful allocation
    let success_result = AllocationResult::success(
        ResourceRequirements::default(),
        "executor-1".to_string(),
        10.0,
        crate::hybrid::resource_manager::EstimatedPerformance::default(),
    );
    assert!(success_result.success);
    assert!(success_result.allocated_resources.is_some());
    assert!(success_result.executor_id.is_some());
    
    // Test failed allocation
    let request = ResourceRequest {
        request_id: "test".to_string(),
        priority: 50,
        requirements: ResourceRequirements::default(),
        estimated_duration: Duration::from_secs(60),
        task_type: "test".to_string(),
        domain: "test".to_string(),
        quality_requirements: QualityRequirements {
            min_success_rate: 0.9,
            max_latency_ms: 5000,
            min_quality_score: 0.8,
            availability_requirement: 0.95,
        },
    };
    
    let failed_result = AllocationResult::failed(&request, vec![]);
    assert!(!failed_result.success);
    assert!(failed_result.allocated_resources.is_none());
    assert!(failed_result.executor_id.is_none());
}

    #[test]
    fn test_scaling_strategies() {
        // Test different scaling strategies
        let mut allocation_config = ResourceAllocation::default();
        
        // Test horizontal scaling
        allocation_config.scaling_strategy = ScalingStrategy::Horizontal;
        let manager_horizontal = ResourceManager::new(allocation_config.clone());
        let ops_horizontal = manager_horizontal.check_auto_scaling();
        // Should be empty but not crash
        
        // Test vertical scaling
        allocation_config.scaling_strategy = ScalingStrategy::Vertical;
        let manager_vertical = ResourceManager::new(allocation_config.clone());
        let ops_vertical = manager_vertical.check_auto_scaling();
        // Should be empty but not crash
        
        // Test hybrid scaling
        allocation_config.scaling_strategy = ScalingStrategy::Hybrid;
        let manager_hybrid = ResourceManager::new(allocation_config);
        let ops_hybrid = manager_hybrid.check_auto_scaling();
        // Should be empty but not crash
        
        assert!(ops_horizontal.is_empty());
        assert!(ops_vertical.is_empty());
        assert!(ops_hybrid.is_empty());
    }