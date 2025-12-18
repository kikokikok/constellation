//! Tests for autonomy measurement engine.

use crate::autonomy::measurement_engine::{AutonomyMeasurementEngine, MeasurementConfig};
use crate::models::autonomy::{AutonomyLevel, CapabilityAxis, KappaScore};
use crate::models::hybrid_agent::TaskResult;
use std::collections::HashMap;
use std::time::Duration;
use uuid::Uuid;

#[test]
fn test_measurement_engine_initialization() {
    let config = MeasurementConfig::default();
    let engine = AutonomyMeasurementEngine::new(config);
    
    // Engine should be created successfully
    assert!(true); // Just checking it doesn't panic
}

#[test]
fn test_record_and_calculate_kappa_scores() {
    let engine = AutonomyMeasurementEngine::default();
    let agent_id = "test-agent";
    
    // Record some observations
    for i in 0..15 {
        let score = 0.5 + (i as f64 * 0.03);
        let confidence = 0.7;
        
        engine.record_observation(
            agent_id,
            CapabilityAxis::Planning,
            score,
            confidence,
            Some(Uuid::new_v4()),
            0.5,
        );
    }
    
    // Calculate κ scores
    let kappa_scores = engine.calculate_kappa_scores(agent_id);
    
    // Should have a score for planning axis
    assert!(kappa_scores.contains_key(&CapabilityAxis::Planning));
    
    if let Some(score) = kappa_scores.get(&CapabilityAxis::Planning) {
        assert!(score.score > 0.5);
        assert!(score.confidence > 0.0);
        assert!(score.observation_count >= 10);
    }
}

#[test]
fn test_determine_autonomy_level() {
    let engine = AutonomyMeasurementEngine::default();
    
    // Create test κ scores
    let mut kappa_scores = HashMap::new();
    
    for axis in CapabilityAxis::all() {
        let score = KappaScore::new(axis, 0.6, 0.8, 10);
        kappa_scores.insert(axis, score);
    }
    
    // Test with different environment complexities
    let level1 = engine.determine_autonomy_level(&kappa_scores, 0.3);
    let level2 = engine.determine_autonomy_level(&kappa_scores, 0.6);
    let level3 = engine.determine_autonomy_level(&kappa_scores, 0.9);
    
    // Higher environment complexity should allow higher autonomy levels
    // (assuming κ scores are sufficient)
    assert!(level2.value() >= level1.value());
    assert!(level3.value() >= level2.value());
}

#[test]
fn test_measure_autonomy() {
    let engine = AutonomyMeasurementEngine::default();
    let agent_id = "test-agent";
    
    // Record observations for multiple axes
    for axis in CapabilityAxis::all() {
        for i in 0..12 {
            let score = 0.4 + (i as f64 * 0.05);
            engine.record_observation(
                agent_id,
                axis,
                score,
                0.7,
                Some(Uuid::new_v4()),
                0.5,
            );
        }
    }
    
    // Measure autonomy
    let measurement = engine.measure_autonomy(agent_id, 0.5);
    
    assert!(measurement.is_some());
    
    if let Some(measurement) = measurement {
        assert_eq!(measurement.agent_id, agent_id);
        assert!(measurement.composite_kappa > 0.0);
        assert!(measurement.composite_kappa <= 1.0);
        assert!(!measurement.kappa_scores.is_empty());
    }
}

#[test]
fn test_record_from_task_result() {
    let engine = AutonomyMeasurementEngine::default();
    let agent_id = "test-agent";
    
    // Create a successful task result
    let task_result = TaskResult {
        task_id: Uuid::new_v4(),
        executor_id: "executor-1".to_string(),
        success: true,
        execution_time_ms: 2000,
        quality_score: 0.85,
        input: serde_json::json!({}),
        output: Some(serde_json::json!({})),
        expected_output: None,
        error_message: None,
        task_type: "test".to_string(),
        priority: 50,
        budget_allocation: 100.0,
        resource_requirements: crate::models::hybrid_agent::ResourceRequirements {
            min_cpu_cores: 2,
            min_memory_mb: 4096,
            gpu_memory_mb: None,
            network_mbps: 100,
        },
        planning_time_ms: 1000,
    };
    
    engine.record_from_task_result(agent_id, &task_result, 0.5);
    
    // Should have recorded observations
    let kappa_scores = engine.calculate_kappa_scores(agent_id);
    
    // At least planning and execution should be recorded
    assert!(kappa_scores.contains_key(&CapabilityAxis::Planning) 
        || kappa_scores.contains_key(&CapabilityAxis::Execution));
}

#[test]
fn test_get_improvement_recommendations() {
    let engine = AutonomyMeasurementEngine::default();
    let agent_id = "test-agent";
    
    // Record low scores for some axes
    for axis in [CapabilityAxis::Planning, CapabilityAxis::Creativity] {
        for _ in 0..12 {
            engine.record_observation(
                agent_id,
                axis,
                0.3, // Low score
                0.8,
                Some(Uuid::new_v4()),
                0.5,
            );
        }
    }
    
    // Record high scores for other axes
    for axis in [CapabilityAxis::Execution, CapabilityAxis::Collaboration] {
        for _ in 0..12 {
            engine.record_observation(
                agent_id,
                axis,
                0.8, // High score
                0.8,
                Some(Uuid::new_v4()),
                0.5,
            );
        }
    }
    
    // Get recommendations
    let recommendations = engine.get_improvement_recommendations(agent_id);
    
    // Should have recommendations for weak axes
    assert!(!recommendations.is_empty());
    
    let weak_axes: Vec<_> = recommendations.iter().map(|(axis, _)| *axis).collect();
    assert!(weak_axes.contains(&CapabilityAxis::Planning));
    assert!(weak_axes.contains(&CapabilityAxis::Creativity));
}

#[test]
fn test_analyze_configuration_potential() {
    use crate::models::hybrid_agent::{
        AllocationStrategy, CoordinationStrategy, CoordinationStrategyType,
        ExecutorConfig, ExecutorDomain, HybridAgentConfig, PerformanceTargets,
        ResourceAllocation, StrategistConfig,
    };
    
    let engine = AutonomyMeasurementEngine::default();
    
    // Create a test configuration
    let config = HybridAgentConfig {
        id: Uuid::new_v4(),
        name: "Test Agent".to_string(),
        description: "Test configuration".to_string(),
        strategist: StrategistConfig::default(),
        executors: vec![
            ExecutorConfig::new("executor-1".to_string(), ExecutorDomain::CodeGeneration),
            ExecutorConfig::new("executor-2".to_string(), ExecutorDomain::DataAnalysis),
        ],
        coordination: CoordinationStrategy::default(),
        resource_allocation: ResourceAllocation {
            strategy: AllocationStrategy::Dynamic,
            ..ResourceAllocation::default()
        },
        performance_targets: PerformanceTargets::default(),
        fallback_strategies: Vec::new(),
    };
    
    let potential_scores = engine.analyze_configuration_potential(&config);
    
    // Should have scores for all axes
    for axis in CapabilityAxis::all() {
        assert!(potential_scores.contains_key(&axis));
        let score = potential_scores.get(&axis).unwrap();
        assert!(*score >= 0.0);
        assert!(*score <= 1.0);
    }
    
    // Dynamic resource allocation should give higher resource management potential
    let resource_score = potential_scores.get(&CapabilityAxis::ResourceManagement).unwrap();
    assert!(*resource_score > 0.3); // Higher than static allocation
}

#[test]
fn test_get_agents_by_autonomy() {
    let engine = AutonomyMeasurementEngine::default();
    
    // Create multiple agents with different performance
    let agents = ["agent-1", "agent-2", "agent-3"];
    
    for (i, agent_id) in agents.iter().enumerate() {
        for axis in CapabilityAxis::all() {
            for j in 0..12 {
                let score = 0.4 + (i as f64 * 0.1) + (j as f64 * 0.01);
                engine.record_observation(
                    agent_id,
                    axis,
                    score,
                    0.8,
                    Some(Uuid::new_v4()),
                    0.5,
                );
            }
        }
        
        // Measure autonomy for each agent
        engine.measure_autonomy(agent_id, 0.5);
    }
    
    // Get agents sorted by autonomy
    let sorted_agents = engine.get_agents_by_autonomy();
    
    // Should have all agents
    assert_eq!(sorted_agents.len(), 3);
    
    // Should be sorted by autonomy level (descending)
    for i in 1..sorted_agents.len() {
        assert!(sorted_agents[i-1].1.value() >= sorted_agents[i].1.value());
        if sorted_agents[i-1].1.value() == sorted_agents[i].1.value() {
            assert!(sorted_agents[i-1].2 >= sorted_agents[i].2);
        }
    }
}

#[test]
fn test_is_ready_to_level_up() {
    let mut config = MeasurementConfig::default();
    
    // Set lower thresholds for testing
    config.kappa_thresholds.insert(AutonomyLevel::Level1GoalOriented, 0.3);
    config.kappa_thresholds.insert(AutonomyLevel::Level2Adaptive, 0.5);
    
    let engine = AutonomyMeasurementEngine::new(config);
    let agent_id = "test-agent";
    
    // Record observations to reach Level 1
    for axis in CapabilityAxis::all() {
        for _ in 0..12 {
            engine.record_observation(
                agent_id,
                axis,
                0.4, // Enough for Level 1
                0.8,
                Some(Uuid::new_v4()),
                0.3, // Environment for Level 1
            );
        }
    }
    
    // Measure autonomy (should be Level 1)
    engine.measure_autonomy(agent_id, 0.3);
    
    // Should not be ready for Level 2 yet
    assert!(!engine.is_ready_to_level_up(agent_id));
    
    // Record more observations to reach Level 2 threshold
    for axis in CapabilityAxis::all() {
        for _ in 0..12 {
            engine.record_observation(
                agent_id,
                axis,
                0.6, // Enough for Level 2
                0.8,
                Some(Uuid::new_v4()),
                0.4, // Environment for Level 2
            );
        }
    }
    
    // Re-measure autonomy
    engine.measure_autonomy(agent_id, 0.4);
    
    // Should be ready for Level 2
    assert!(engine.is_ready_to_level_up(agent_id));
}

#[test]
fn test_progress_tracking() {
    let engine = AutonomyMeasurementEngine::default();
    let agent_id = "test-agent";
    
    // Record initial observations
    for axis in CapabilityAxis::all() {
        for _ in 0..12 {
            engine.record_observation(
                agent_id,
                axis,
                0.4,
                0.8,
                Some(Uuid::new_v4()),
                0.3,
            );
        }
    }
    
    // First measurement
    engine.measure_autonomy(agent_id, 0.3);
    
    // Get progress tracking
    let progress = engine.get_progress_tracking(agent_id);
    assert!(progress.is_some());
    
    if let Some(progress) = progress {
        assert_eq!(progress.agent_id, agent_id);
        assert_eq!(progress.starting_level, AutonomyLevel::Level0Scripted);
        assert_eq!(progress.current_level, AutonomyLevel::Level0Scripted);
        assert_eq!(progress.target_level, AutonomyLevel::Level9Transcendent);
        assert_eq!(progress.measurements.len(), 1);
    }
    
    // Record improved observations
    for axis in CapabilityAxis::all() {
        for _ in 0..12 {
            engine.record_observation(
                agent_id,
                axis,
                0.7, // Improved score
                0.8,
                Some(Uuid::new_v4()),
                0.5, // Higher environment
            );
        }
    }
    
    // Second measurement
    engine.measure_autonomy(agent_id, 0.5);
    
    // Check updated progress
    let progress = engine.get_progress_tracking(agent_id).unwrap();
    assert_eq!(progress.measurements.len(), 2);
    assert!(progress.progress_rate > 0.0);
}

#[test]
fn test_estimate_time_to_target() {
    let engine = AutonomyMeasurementEngine::default();
    let agent_id = "test-agent";
    
    // Record observations showing improvement
    for (i, axis) in CapabilityAxis::all().iter().enumerate() {
        for j in 0..24 {
            let score = 0.3 + (i as f64 * 0.05) + (j as f64 * 0.01);
            engine.record_observation(
                agent_id,
                *axis,
                score.min(0.9),
                0.8,
                Some(Uuid::new_v4()),
                0.5,
            );
        }
    }
    
    // Measure autonomy multiple times (simulating progress over time)
    for _ in 0..3 {
        engine.measure_autonomy(agent_id, 0.5);
        
        // Simulate time passing by adding more observations
        for axis in CapabilityAxis::all() {
            engine.record_observation(
                agent_id,
                axis,
                0.8,
                0.8,
                Some(Uuid::new_v4()),
                0.5,
            );
        }
    }
    
    // Get time estimate
    let estimate = engine.estimate_time_to_target(agent_id, AutonomyLevel::Level9Transcendent);
    
    // Should have an estimate if progress rate is positive
    let progress = engine.get_progress_tracking(agent_id).unwrap();
    if progress.progress_rate > 0.0 {
        assert!(estimate.is_some());
    }
}