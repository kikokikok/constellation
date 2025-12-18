//! Tests for fallback strategies and graceful degradation.

use crate::hybrid::fallback_manager::{
    Bulkhead, BulkheadConfig, CircuitBreaker, CircuitBreakerConfig, CircuitBreakerState,
    DegradationLevel, FallbackManager, GracefulDegradation, RetryConfig, RetryMechanism,
};
use crate::hybrid::performance_monitor::{
    Alert, AlertLevel, AlertType, PerformanceMetric, PerformanceMonitor,
};
use crate::models::hybrid_agent::{FallbackAction, FallbackStrategy, FallbackTrigger};
use std::sync::Arc;
use std::time::{Duration, Instant};
use uuid::Uuid;

#[test]
fn test_circuit_breaker_initial_state() {
    let circuit_breaker = CircuitBreaker::new("test-service".to_string());

    assert_eq!(circuit_breaker.state, CircuitBreakerState::Closed);
    assert_eq!(circuit_breaker.failure_count, 0);
    assert_eq!(circuit_breaker.success_count, 0);
    assert!(circuit_breaker.is_available());
}

#[test]
fn test_circuit_breaker_record_success() {
    let mut circuit_breaker = CircuitBreaker::new("test-service".to_string());

    // Initial state
    assert_eq!(circuit_breaker.state, CircuitBreakerState::Closed);

    // Record success in closed state
    circuit_breaker.record_success();
    assert_eq!(circuit_breaker.state, CircuitBreakerState::Closed);
    assert_eq!(circuit_breaker.failure_count, 0);

    // Record failures to open circuit
    for _ in 0..circuit_breaker.config.failure_threshold {
        circuit_breaker.record_failure();
    }
    assert_eq!(circuit_breaker.state, CircuitBreakerState::Open);
    assert!(!circuit_breaker.is_available());

    // Wait for reset timeout (simulate by setting last_failure_time in the past)
    // In real test, we would use mock time
}

#[test]
fn test_circuit_breaker_record_failure() {
    let mut circuit_breaker = CircuitBreaker::new("test-service".to_string());

    // Record failures up to threshold
    for i in 0..circuit_breaker.config.failure_threshold - 1 {
        circuit_breaker.record_failure();
        assert_eq!(circuit_breaker.failure_count, i + 1);
        assert_eq!(circuit_breaker.state, CircuitBreakerState::Closed);
        assert!(circuit_breaker.is_available());
    }

    // Record final failure to open circuit
    circuit_breaker.record_failure();
    assert_eq!(circuit_breaker.state, CircuitBreakerState::Open);
    assert!(!circuit_breaker.is_available());
    assert!(circuit_breaker.last_failure_time.is_some());
}

#[test]
fn test_circuit_breaker_half_open_recovery() {
    let mut circuit_breaker = CircuitBreaker::new("test-service".to_string());

    // Open circuit
    for _ in 0..circuit_breaker.config.failure_threshold {
        circuit_breaker.record_failure();
    }
    assert_eq!(circuit_breaker.state, CircuitBreakerState::Open);

    // Simulate reset timeout passed (in real test, use mock time)
    // For now, manually set state to half-open and reset counters
    circuit_breaker.state = CircuitBreakerState::HalfOpen;
    circuit_breaker.success_count = 0;
    circuit_breaker.failure_count = 0;

    // Record successes to close circuit
    for i in 0..circuit_breaker.config.success_threshold {
        circuit_breaker.record_success();

        if i < circuit_breaker.config.success_threshold - 1 {
            // Still in half-open state
            assert_eq!(circuit_breaker.state, CircuitBreakerState::HalfOpen);
            assert_eq!(circuit_breaker.success_count, i + 1);
        } else {
            // After last success, circuit should close
            assert_eq!(circuit_breaker.state, CircuitBreakerState::Closed);
            assert_eq!(circuit_breaker.success_count, 0);
        }
    }

    // After enough successes, circuit should be closed with counters reset
    assert_eq!(circuit_breaker.state, CircuitBreakerState::Closed);
    assert_eq!(circuit_breaker.success_count, 0);
    assert_eq!(circuit_breaker.failure_count, 0);
}

#[test]
fn test_retry_mechanism_calculate_delay() {
    let config = RetryConfig {
        max_attempts: 5,
        initial_delay: Duration::from_millis(100),
        max_delay: Duration::from_secs(10),
        backoff_factor: 2.0,
        jitter: false,
    };

    let retry_mechanism = RetryMechanism::new(config);

    // Test exponential backoff
    let delay1 = retry_mechanism.calculate_delay(1);
    let delay2 = retry_mechanism.calculate_delay(2);
    let delay3 = retry_mechanism.calculate_delay(3);

    assert_eq!(delay1, Duration::from_millis(100));
    assert_eq!(delay2, Duration::from_millis(200)); // 100 * 2^1
    assert_eq!(delay3, Duration::from_millis(400)); // 100 * 2^2

    // Test max delay
    let delay_large = retry_mechanism.calculate_delay(20); // Would be huge without max
    assert!(delay_large <= Duration::from_secs(10));
}

#[test]
fn test_graceful_degradation_initial_state() {
    let degradation = GracefulDegradation::new();

    assert_eq!(degradation.current_level, DegradationLevel::FullService);
    assert!(!degradation.triggers.is_empty());
    assert!(
        degradation
            .actions
            .contains_key(&DegradationLevel::FullService)
    );
    assert!(
        degradation
            .actions
            .contains_key(&DegradationLevel::ReducedQuality)
    );
    assert!(
        degradation
            .actions
            .contains_key(&DegradationLevel::EssentialOnly)
    );
    assert!(
        degradation
            .actions
            .contains_key(&DegradationLevel::ReadOnly)
    );
    assert!(
        degradation
            .actions
            .contains_key(&DegradationLevel::EmergencyMode)
    );
}

#[test]
fn test_graceful_degradation_escalation() {
    let mut degradation = GracefulDegradation::new();

    // Start at FullService
    assert_eq!(degradation.current_level, DegradationLevel::FullService);

    // Escalate to ReducedQuality
    let actions1 = degradation.escalate();
    assert_eq!(degradation.current_level, DegradationLevel::ReducedQuality);
    assert!(actions1.contains(&FallbackAction::ReduceQuality));
    assert!(actions1.contains(&FallbackAction::SwitchExecutor));

    // Escalate to EssentialOnly
    let actions2 = degradation.escalate();
    assert_eq!(degradation.current_level, DegradationLevel::EssentialOnly);
    assert!(actions2.contains(&FallbackAction::ReduceQuality));
    assert!(actions2.contains(&FallbackAction::SwitchExecutor));
    assert!(actions2.contains(&FallbackAction::ScaleResources));

    // Escalate to ReadOnly
    let actions3 = degradation.escalate();
    assert_eq!(degradation.current_level, DegradationLevel::ReadOnly);
    assert!(actions3.contains(&FallbackAction::AbortTask));
    assert!(actions3.contains(&FallbackAction::NotifyHuman));

    // Escalate to EmergencyMode
    let actions4 = degradation.escalate();
    assert_eq!(degradation.current_level, DegradationLevel::EmergencyMode);
    assert!(actions4.contains(&FallbackAction::AbortTask));
    assert!(actions4.contains(&FallbackAction::NotifyHuman));
    assert!(actions4.contains(&FallbackAction::UseAlternativeStrategy));

    // Cannot escalate further
    let actions5 = degradation.escalate();
    assert_eq!(degradation.current_level, DegradationLevel::EmergencyMode);
    assert_eq!(actions5, actions4);
}

#[test]
fn test_graceful_degradation_deescalation() {
    let mut degradation = GracefulDegradation::new();

    // Escalate to EmergencyMode first
    degradation.escalate(); // ReducedQuality
    degradation.escalate(); // EssentialOnly
    degradation.escalate(); // ReadOnly
    degradation.escalate(); // EmergencyMode
    assert_eq!(degradation.current_level, DegradationLevel::EmergencyMode);

    // Deescalate to ReadOnly
    let actions1 = degradation.deescalate();
    assert_eq!(degradation.current_level, DegradationLevel::ReadOnly);
    assert!(actions1.contains(&FallbackAction::AbortTask));
    assert!(actions1.contains(&FallbackAction::NotifyHuman));

    // Deescalate to EssentialOnly
    let actions2 = degradation.deescalate();
    assert_eq!(degradation.current_level, DegradationLevel::EssentialOnly);
    assert!(actions2.contains(&FallbackAction::ReduceQuality));
    assert!(actions2.contains(&FallbackAction::SwitchExecutor));
    assert!(actions2.contains(&FallbackAction::ScaleResources));

    // Deescalate to ReducedQuality
    let actions3 = degradation.deescalate();
    assert_eq!(degradation.current_level, DegradationLevel::ReducedQuality);
    assert!(actions3.contains(&FallbackAction::ReduceQuality));
    assert!(actions3.contains(&FallbackAction::SwitchExecutor));

    // Deescalate to FullService
    let actions4 = degradation.deescalate();
    assert_eq!(degradation.current_level, DegradationLevel::FullService);
    assert!(actions4.is_empty());

    // Cannot deescalate further
    let actions5 = degradation.deescalate();
    assert_eq!(degradation.current_level, DegradationLevel::FullService);
    assert!(actions5.is_empty());
}

#[test]
fn test_bulkhead_initial_state() {
    let bulkhead = Bulkhead::new("test-bulkhead".to_string());

    assert_eq!(bulkhead.id, "test-bulkhead");
    assert_eq!(bulkhead.concurrent_calls, 0);
    assert!(bulkhead.can_execute());
}

#[test]
fn test_bulkhead_acquire_release() {
    let mut bulkhead = Bulkhead::new("test-bulkhead".to_string());

    // Acquire slots up to max
    for i in 0..bulkhead.config.max_concurrent_calls {
        assert!(bulkhead.acquire());
        assert_eq!(bulkhead.concurrent_calls, i + 1);
    }

    // Should not be able to acquire more
    assert!(!bulkhead.acquire());
    assert_eq!(
        bulkhead.concurrent_calls,
        bulkhead.config.max_concurrent_calls
    );
    assert!(!bulkhead.can_execute());

    // Release slots
    for i in (0..bulkhead.config.max_concurrent_calls).rev() {
        bulkhead.release();
        assert_eq!(bulkhead.concurrent_calls, i);
    }

    // Should not go below 0
    bulkhead.release();
    assert_eq!(bulkhead.concurrent_calls, 0);
}

#[tokio::test]
async fn test_fallback_manager_initialization() {
    let fallback_manager = FallbackManager::new();

    // Should be able to create without performance monitor
    assert!(fallback_manager.performance_monitor.is_none());

    // Should have empty strategies by default
    assert!(fallback_manager.strategies.is_empty());
}

#[tokio::test]
async fn test_fallback_manager_with_performance_monitor() {
    let performance_monitor = Arc::new(PerformanceMonitor::new(
        crate::models::hybrid_agent::PerformanceTargets::default(),
    ));

    let fallback_manager =
        FallbackManager::new().with_performance_monitor(performance_monitor.clone());

    assert!(fallback_manager.performance_monitor.is_some());
}

#[tokio::test]
async fn test_fallback_manager_with_strategies() {
    let strategies = vec![
        FallbackStrategy {
            trigger: FallbackTrigger::HighLatency,
            action: FallbackAction::SwitchExecutor,
            priority: 1,
            timeout_ms: 5000,
        },
        FallbackStrategy {
            trigger: FallbackTrigger::LowSuccessRate,
            action: FallbackAction::ReduceQuality,
            priority: 2,
            timeout_ms: 10000,
        },
    ];

    let fallback_manager = FallbackManager::new().with_strategies(strategies.clone());

    assert_eq!(fallback_manager.strategies.len(), 2);
    assert_eq!(
        fallback_manager.strategies[0].trigger,
        FallbackTrigger::HighLatency
    );
    assert_eq!(
        fallback_manager.strategies[1].trigger,
        FallbackTrigger::LowSuccessRate
    );
}

#[tokio::test]
async fn test_fallback_manager_circuit_breaker_operations() {
    let fallback_manager = FallbackManager::new();

    // Add circuit breaker
    fallback_manager
        .add_circuit_breaker("test-service".to_string(), None)
        .await;

    // Should be available initially
    assert!(
        fallback_manager
            .is_circuit_breaker_available("test-service")
            .await
    );

    // Record failures
    for _ in 0..4 {
        fallback_manager.record_failure("test-service").await;
        assert!(
            fallback_manager
                .is_circuit_breaker_available("test-service")
                .await
        );
    }

    // 5th failure should open circuit (default threshold is 5)
    fallback_manager.record_failure("test-service").await;
    assert!(
        !fallback_manager
            .is_circuit_breaker_available("test-service")
            .await
    );

    // Record success (should not affect open circuit without timeout)
    fallback_manager.record_success("test-service").await;
    assert!(
        !fallback_manager
            .is_circuit_breaker_available("test-service")
            .await
    );
}

#[tokio::test]
async fn test_fallback_manager_bulkhead_operations() {
    let fallback_manager = FallbackManager::new();

    // Add bulkhead
    fallback_manager
        .add_bulkhead("test-bulkhead".to_string(), None)
        .await;

    // Acquire slots
    for i in 0..10 {
        let acquired = fallback_manager.acquire_bulkhead("test-bulkhead").await;
        assert!(acquired, "Should acquire slot {}", i + 1);
    }

    // Should not acquire more (default max is 10)
    let acquired = fallback_manager.acquire_bulkhead("test-bulkhead").await;
    assert!(!acquired, "Should not acquire beyond max");

    // Release slots
    for _ in 0..10 {
        fallback_manager.release_bulkhead("test-bulkhead").await;
    }

    // Should be able to acquire again
    let acquired = fallback_manager.acquire_bulkhead("test-bulkhead").await;
    assert!(acquired, "Should acquire after release");
}

#[tokio::test]
async fn test_fallback_manager_handle_alert() {
    let strategies = vec![FallbackStrategy {
        trigger: FallbackTrigger::HighLatency,
        action: FallbackAction::SwitchExecutor,
        priority: 1,
        timeout_ms: 5000,
    }];

    let fallback_manager = FallbackManager::new().with_strategies(strategies);

    // Create alert matching strategy
    let alert = Alert {
        id: Uuid::new_v4(),
        alert_type: AlertType::Performance,
        level: AlertLevel::Warning,
        metric: PerformanceMetric::Latency,
        threshold: 1000.0,
        actual_value: 2000.0,
        duration_above_threshold: Duration::from_secs(60),
        message: "High latency detected".to_string(),
        timestamp: Instant::now(),
        recommendations: vec!["Switch executor".to_string()],
        acknowledged: false,
        resolved: false,
    };

    let actions = fallback_manager.handle_alert(&alert).await;

    assert_eq!(actions.len(), 1);
    assert_eq!(actions[0], FallbackAction::SwitchExecutor);
}

#[tokio::test]
async fn test_fallback_manager_handle_alert_no_match() {
    let strategies = vec![FallbackStrategy {
        trigger: FallbackTrigger::HighLatency,
        action: FallbackAction::SwitchExecutor,
        priority: 1,
        timeout_ms: 5000,
    }];

    let fallback_manager = FallbackManager::new().with_strategies(strategies);

    // Create alert not matching any strategy
    let alert = Alert {
        id: Uuid::new_v4(),
        alert_type: AlertType::Performance,
        level: AlertLevel::Warning,
        metric: PerformanceMetric::SuccessRate,
        threshold: 0.05,
        actual_value: 0.15,
        duration_above_threshold: Duration::from_secs(60),
        message: "High error rate detected".to_string(),
        timestamp: Instant::now(),
        recommendations: vec!["Check executor health".to_string()],
        acknowledged: false,
        resolved: false,
    };

    let actions = fallback_manager.handle_alert(&alert).await;

    // Should return empty vector for non-matching alert
    assert!(actions.is_empty());
}

#[tokio::test]
async fn test_fallback_manager_handle_alert_critical_without_strategy() {
    let fallback_manager = FallbackManager::new();

    // Create critical alert without matching strategy
    let alert = Alert {
        id: Uuid::new_v4(),
        alert_type: AlertType::Performance,
        level: AlertLevel::Critical,
        metric: PerformanceMetric::Latency,
        threshold: 1000.0,
        actual_value: 5000.0,
        duration_above_threshold: Duration::from_secs(300),
        message: "Critical high latency detected".to_string(),
        timestamp: Instant::now(),
        recommendations: vec!["Immediate action required".to_string()],
        acknowledged: false,
        resolved: false,
    };

    let actions = fallback_manager.handle_alert(&alert).await;

    // Critical alert without strategy should trigger graceful degradation escalation
    // Since we start at FullService, first escalation is to ReducedQuality
    assert!(actions.contains(&FallbackAction::ReduceQuality));
    assert!(actions.contains(&FallbackAction::SwitchExecutor));
}

#[tokio::test]
async fn test_fallback_manager_get_recommended_actions() {
    let strategies = vec![FallbackStrategy {
        trigger: FallbackTrigger::HighLatency,
        action: FallbackAction::SwitchExecutor,
        priority: 1,
        timeout_ms: 5000,
    }];

    let fallback_manager = FallbackManager::new().with_strategies(strategies);

    // Get recommended actions for trigger with strategy
    let actions = fallback_manager
        .get_recommended_actions(FallbackTrigger::HighLatency)
        .await;

    assert_eq!(actions.len(), 1);
    assert_eq!(actions[0], FallbackAction::SwitchExecutor);

    // Get recommended actions for trigger without strategy
    let actions = fallback_manager
        .get_recommended_actions(FallbackTrigger::ResourceExhaustion)
        .await;

    // Should return default recommendations
    assert!(!actions.is_empty());
    assert!(actions.contains(&FallbackAction::ScaleResources));
    assert!(actions.contains(&FallbackAction::SwitchExecutor));
}

#[tokio::test]
async fn test_fallback_manager_execute_with_retry() {
    let fallback_manager = FallbackManager::new();

    let attempt_count = std::sync::Arc::new(std::sync::atomic::AtomicUsize::new(0));
    let attempt_count_clone = attempt_count.clone();

    let result = fallback_manager
        .execute_with_retry(move || {
            let current = attempt_count_clone.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
            if current < 2 {
                Err("Temporary failure".to_string())
            } else {
                Ok("Success".to_string())
            }
        })
        .await;

    assert_eq!(result, Ok("Success".to_string()));
    assert_eq!(attempt_count.load(std::sync::atomic::Ordering::SeqCst), 3);
}

#[tokio::test]
async fn test_fallback_manager_execute_with_retry_max_attempts() {
    let fallback_manager = FallbackManager::new();

    let attempt_count = std::sync::Arc::new(std::sync::atomic::AtomicUsize::new(0));
    let attempt_count_clone = attempt_count.clone();

    let result = fallback_manager
        .execute_with_retry(move || {
            attempt_count_clone.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
            Err::<String, String>("Always fails".to_string())
        })
        .await;

    assert_eq!(result, Err("Always fails".to_string()));
    assert_eq!(attempt_count.load(std::sync::atomic::Ordering::SeqCst), 3); // Default max attempts is 3
}

#[tokio::test]
async fn test_fallback_manager_execute_with_timeout_success() {
    let fallback_manager = FallbackManager::new();

    let result = fallback_manager
        .execute_with_timeout(
            async {
                tokio::time::sleep(Duration::from_millis(50)).await;
                "Success".to_string()
            },
            Duration::from_millis(100),
        )
        .await;

    assert_eq!(result, Ok("Success".to_string()));
}

#[tokio::test]
async fn test_fallback_manager_execute_with_timeout_failure() {
    let fallback_manager = FallbackManager::new();

    let result = fallback_manager
        .execute_with_timeout(
            async {
                tokio::time::sleep(Duration::from_millis(200)).await;
                "Success".to_string()
            },
            Duration::from_millis(100),
        )
        .await;

    assert!(result.is_err());
    assert!(result.unwrap_err().contains("timed out"));
}

#[tokio::test]
async fn test_fallback_manager_get_current_degradation_level() {
    let fallback_manager = FallbackManager::new();

    let level = fallback_manager.get_current_degradation_level().await;

    assert_eq!(level, DegradationLevel::FullService);
}

#[test]
fn test_fallback_manager_default() {
    let fallback_manager = FallbackManager::default();

    // Should be same as new()
    assert!(fallback_manager.performance_monitor.is_none());
    assert!(fallback_manager.strategies.is_empty());
}
