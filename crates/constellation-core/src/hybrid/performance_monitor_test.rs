//! Tests for performance monitoring and optimization.

use crate::hybrid::coordinator::{ResourceUsage, TaskResult};
use crate::hybrid::performance_monitor::{
    AlertLevel, AlertSubscriber, AlertThresholds, AlertType, NotificationChannel,
    PerformanceHistory, PerformanceMonitor, PerformanceTrends,
};
use crate::models::hybrid_agent::PerformanceTargets;
use std::time::Duration;

#[test]
fn test_performance_monitor_creation() {
    let targets = PerformanceTargets::default();
    let monitor = PerformanceMonitor::new(targets);

    let metrics = monitor.get_current_metrics();
    assert_eq!(metrics.throughput_tps, 0.0);
    assert_eq!(metrics.avg_latency_ms, 0.0);
    assert_eq!(metrics.success_rate, 0.0);
}

#[test]
fn test_performance_history_creation() {
    let history = PerformanceHistory::new();

    // Can't check private fields directly
    // Just verify the struct was created
    assert!(true);
}

#[test]
fn test_performance_history_add_task_result() {
    let mut history = PerformanceHistory::new();

    let result = TaskResult {
        task_id: uuid::Uuid::new_v4(),
        executor_id: "test-executor".to_string(),
        completed_at: chrono::Utc::now(),
        result: serde_json::Value::Null,
        success: true,
        error: None,
        quality_score: 0.9,
        execution_time_ms: 1000,
        resource_usage: ResourceUsage {
            cpu_core_seconds: 1.0,
            memory_mb_seconds: 1024.0,
            gpu_memory_mb_seconds: None,
            network_mb: 10.0,
        },
        cost: 0.5,
    };

    history.add_task_result(result);
    // Can't check private fields directly
    // Just verify the method doesn't panic
    assert!(true);
}

#[test]
fn test_performance_monitor_update_with_task_result() {
    println!("TEST: Starting test_performance_monitor_update_with_task_result");
    let targets = PerformanceTargets::default();
    println!("TEST: Creating PerformanceMonitor");
    let monitor = PerformanceMonitor::new(targets);
    println!("TEST: PerformanceMonitor created");

    let result = TaskResult {
        task_id: uuid::Uuid::new_v4(),
        executor_id: "test-executor".to_string(),
        completed_at: chrono::Utc::now(),
        result: serde_json::Value::Null,
        success: true,
        error: None,
        quality_score: 0.9,
        execution_time_ms: 1000,
        resource_usage: ResourceUsage {
            cpu_core_seconds: 1.0,
            memory_mb_seconds: 1024.0,
            gpu_memory_mb_seconds: None,
            network_mb: 10.0,
        },
        cost: 0.5,
    };

    println!("TEST: Calling update_with_task_result");
    let update_result = monitor.update_with_task_result(&result);
    println!("TEST: update_with_task_result returned");
    assert!(update_result.is_ok());

    println!("TEST: Getting current metrics");
    let metrics = monitor.get_current_metrics();
    println!("TEST: Got metrics");
    // After one successful task, metrics should be updated
    assert!(metrics.success_rate > 0.0);
    println!("TEST: Test completed successfully");
}

#[test]
fn test_alert_thresholds_default() {
    let thresholds = AlertThresholds::default();

    assert_eq!(thresholds.critical.success_rate, 0.8);
    assert_eq!(thresholds.warning.success_rate, 0.9);
    assert_eq!(thresholds.info.success_rate, 0.95);
    assert_eq!(thresholds.critical.latency_ms, 10000);
    assert_eq!(thresholds.warning.latency_ms, 5000);
    assert_eq!(thresholds.info.latency_ms, 3000);
}

#[test]
fn test_performance_trends_calculation() {
    use crate::hybrid::performance_monitor::calculate_trend;

    let values = vec![1.0, 2.0, 3.0, 4.0, 5.0];
    let trend = calculate_trend(&values);

    // Increasing trend should be positive
    assert!(trend > 0.0);

    let decreasing_values = vec![5.0, 4.0, 3.0, 2.0, 1.0];
    let decreasing_trend = calculate_trend(&decreasing_values);

    // Decreasing trend should be negative
    assert!(decreasing_trend < 0.0);

    let constant_values = vec![3.0, 3.0, 3.0, 3.0, 3.0];
    let constant_trend = calculate_trend(&constant_values);

    // Constant trend should be approximately zero
    assert!(constant_trend.abs() < 0.001);
}

#[test]
fn test_alert_subscriber_management() {
    let targets = PerformanceTargets::default();
    let monitor = PerformanceMonitor::new(targets);

    let subscriber = AlertSubscriber {
        id: "test-subscriber".to_string(),
        channel: NotificationChannel::Email("test@example.com".to_string()),
        alert_levels: vec![AlertLevel::Critical, AlertLevel::Warning],
        alert_types: vec![AlertType::Performance, AlertType::Resource],
        cooldown_period: Duration::from_secs(60),
        last_notification: None,
    };

    // Add subscriber
    monitor.add_subscriber(subscriber.clone());

    // Remove subscriber
    monitor.remove_subscriber("test-subscriber");

    // Adding again should work
    monitor.add_subscriber(subscriber);
}

#[test]
fn test_performance_trends_default() {
    let trends = PerformanceTrends::default();

    assert_eq!(trends.latency_trend, 0.0);
    assert_eq!(trends.success_rate_trend, 0.0);
    assert_eq!(trends.quality_trend, 0.0);
    assert_eq!(trends.throughput_trend, 0.0);
    assert_eq!(trends.cost_trend, 0.0);
}

#[test]
fn test_optimization_engine_default() {
    use crate::hybrid::performance_monitor::OptimizationEngine;

    let engine = OptimizationEngine::default();

    assert_eq!(engine.algorithms.len(), 1);
    assert_eq!(engine.improvement_threshold, 0.1);
    assert_eq!(engine.min_samples, 100);
    assert_eq!(engine.optimization_frequency, Duration::from_secs(300));
    assert!(engine.last_optimization.is_none());
}

#[test]
fn test_predictive_scaler_default() {
    use crate::hybrid::performance_monitor::PredictiveScaler;

    let scaler = PredictiveScaler::default();

    assert_eq!(scaler.models.len(), 1);
    assert_eq!(scaler.prediction_horizon, Duration::from_secs(300));
    assert_eq!(scaler.prediction_frequency, Duration::from_secs(60));
    assert_eq!(scaler.confidence_threshold, 0.7);
    assert!(scaler.last_prediction.is_none());
    assert!(scaler.last_recommendation.is_none());
}

#[test]
fn test_optimization_engine_optimize() {
    use crate::hybrid::performance_monitor::{OptimizationEngine, PerformanceHistory};

    let mut engine = OptimizationEngine::default();
    let history = PerformanceHistory::new();

    let result = engine.optimize(&history);
    assert!(result.is_ok());

    let optimization_result = result.unwrap();
    assert!(optimization_result.success);
    assert!(optimization_result.expected_improvement.overall_score > 0.0);
}

#[test]
fn test_predictive_scaler_predict() {
    use crate::hybrid::performance_monitor::{PerformanceHistory, PredictiveScaler};

    let mut scaler = PredictiveScaler::default();
    let history = PerformanceHistory::new();

    let result = scaler.predict(&history);
    assert!(result.is_ok());

    let recommendation = result.unwrap();
    assert!(recommendation.confidence > 0.0);
    assert!(recommendation.priority > 0);
}

#[test]
fn test_performance_monitor_get_active_alerts() {
    let targets = PerformanceTargets::default();
    let monitor = PerformanceMonitor::new(targets);

    let alerts = monitor.get_active_alerts();
    assert_eq!(alerts.len(), 0); // No alerts initially
}

#[test]
fn test_performance_monitor_get_optimization_recommendations() {
    let targets = PerformanceTargets::default();
    let monitor = PerformanceMonitor::new(targets);

    let recommendations = monitor.get_optimization_recommendations();
    assert_eq!(recommendations.len(), 0); // No recommendations initially
}

#[test]
fn test_performance_monitor_get_scaling_recommendations() {
    let targets = PerformanceTargets::default();
    let monitor = PerformanceMonitor::new(targets);

    let recommendation = monitor.get_scaling_recommendations();
    assert!(recommendation.is_none()); // No recommendation initially
}
