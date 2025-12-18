//! Fallback strategies and graceful degradation manager.
//!
//! Implements fault tolerance mechanisms including:
//! - Circuit breaker patterns
//! - Retry mechanisms with exponential backoff
//! - Graceful degradation strategies
//! - Health check integration
//! - Timeout handling and deadline management
//! - Bulkhead patterns for failure isolation

use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::{Mutex, RwLock};

use crate::hybrid::performance_monitor::{
    Alert, AlertLevel, AlertType, PerformanceMetric, PerformanceMonitor,
};
use crate::models::hybrid_agent::{FallbackAction, FallbackStrategy, FallbackTrigger};

/// Circuit breaker state.
#[derive(Debug, Clone, PartialEq)]
pub enum CircuitBreakerState {
    Closed,
    Open,
    HalfOpen,
}

/// Circuit breaker configuration.
#[derive(Debug, Clone)]
pub struct CircuitBreakerConfig {
    pub failure_threshold: u32,
    pub reset_timeout: Duration,
    pub half_open_max_attempts: u32,
    pub success_threshold: u32,
}

impl Default for CircuitBreakerConfig {
    fn default() -> Self {
        Self {
            failure_threshold: 5,
            reset_timeout: Duration::from_secs(30),
            half_open_max_attempts: 3,
            success_threshold: 3,
        }
    }
}

/// Circuit breaker for an executor or service.
#[derive(Debug)]
pub struct CircuitBreaker {
    pub id: String,
    pub state: CircuitBreakerState,
    pub failure_count: u32,
    pub success_count: u32,
    pub last_failure_time: Option<Instant>,
    pub config: CircuitBreakerConfig,
}

impl CircuitBreaker {
    pub fn new(id: String) -> Self {
        Self {
            id,
            state: CircuitBreakerState::Closed,
            failure_count: 0,
            success_count: 0,
            last_failure_time: None,
            config: CircuitBreakerConfig::default(),
        }
    }

    pub fn with_config(id: String, config: CircuitBreakerConfig) -> Self {
        Self {
            id,
            state: CircuitBreakerState::Closed,
            failure_count: 0,
            success_count: 0,
            last_failure_time: None,
            config,
        }
    }

    pub fn record_success(&mut self) {
        match self.state {
            CircuitBreakerState::Closed => {
                self.failure_count = 0;
            }
            CircuitBreakerState::HalfOpen => {
                self.success_count += 1;
                if self.success_count >= self.config.success_threshold {
                    self.state = CircuitBreakerState::Closed;
                    self.success_count = 0;
                    self.failure_count = 0;
                }
            }
            CircuitBreakerState::Open => {
                // Check if reset timeout has passed
                if let Some(last_failure) = self.last_failure_time
                    && last_failure.elapsed() >= self.config.reset_timeout
                {
                    self.state = CircuitBreakerState::HalfOpen;
                    self.success_count = 0;
                    self.failure_count = 0;
                }
            }
        }
    }

    pub fn record_failure(&mut self) {
        match self.state {
            CircuitBreakerState::Closed => {
                self.failure_count += 1;
                if self.failure_count >= self.config.failure_threshold {
                    self.state = CircuitBreakerState::Open;
                    self.last_failure_time = Some(Instant::now());
                }
            }
            CircuitBreakerState::HalfOpen => {
                self.state = CircuitBreakerState::Open;
                self.last_failure_time = Some(Instant::now());
                self.success_count = 0;
            }
            CircuitBreakerState::Open => {
                // Already open, just update timestamp
                self.last_failure_time = Some(Instant::now());
            }
        }
    }

    pub fn is_available(&self) -> bool {
        match self.state {
            CircuitBreakerState::Closed => true,
            CircuitBreakerState::HalfOpen => true,
            CircuitBreakerState::Open => {
                if let Some(last_failure) = self.last_failure_time {
                    last_failure.elapsed() >= self.config.reset_timeout
                } else {
                    false
                }
            }
        }
    }
}

/// Retry configuration.
#[derive(Debug, Clone)]
pub struct RetryConfig {
    pub max_attempts: u32,
    pub initial_delay: Duration,
    pub max_delay: Duration,
    pub backoff_factor: f64,
    pub jitter: bool,
}

impl Default for RetryConfig {
    fn default() -> Self {
        Self {
            max_attempts: 3,
            initial_delay: Duration::from_millis(100),
            max_delay: Duration::from_secs(10),
            backoff_factor: 2.0,
            jitter: true,
        }
    }
}

/// Retry mechanism with exponential backoff.
#[derive(Debug)]
pub struct RetryMechanism {
    pub config: RetryConfig,
}

impl RetryMechanism {
    pub fn new(config: RetryConfig) -> Self {
        Self { config }
    }

    pub fn calculate_delay(&self, attempt: u32) -> Duration {
        let base_delay = self.config.initial_delay.as_millis() as f64;
        let delay_ms = base_delay * self.config.backoff_factor.powi(attempt as i32 - 1);

        let mut delay =
            Duration::from_millis(delay_ms.min(self.config.max_delay.as_millis() as f64) as u64);

        if self.config.jitter {
            use rand::Rng;
            let mut rng = rand::rng();
            let jitter = rng.random_range(0.8..1.2);
            delay = Duration::from_millis((delay.as_millis() as f64 * jitter) as u64);
        }

        delay
    }
}

/// Graceful degradation level.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum DegradationLevel {
    FullService,
    ReducedQuality,
    EssentialOnly,
    ReadOnly,
    EmergencyMode,
}

/// Graceful degradation strategy.
#[derive(Debug, Clone)]
pub struct GracefulDegradation {
    pub current_level: DegradationLevel,
    pub triggers: Vec<FallbackTrigger>,
    pub actions: HashMap<DegradationLevel, Vec<FallbackAction>>,
}

impl Default for GracefulDegradation {
    fn default() -> Self {
        Self::new()
    }
}

impl GracefulDegradation {
    pub fn new() -> Self {
        let mut actions = HashMap::new();

        actions.insert(DegradationLevel::FullService, vec![]);
        actions.insert(
            DegradationLevel::ReducedQuality,
            vec![
                FallbackAction::ReduceQuality,
                FallbackAction::SwitchExecutor,
            ],
        );
        actions.insert(
            DegradationLevel::EssentialOnly,
            vec![
                FallbackAction::ReduceQuality,
                FallbackAction::SwitchExecutor,
                FallbackAction::ScaleResources,
            ],
        );
        actions.insert(
            DegradationLevel::ReadOnly,
            vec![FallbackAction::AbortTask, FallbackAction::NotifyHuman],
        );
        actions.insert(
            DegradationLevel::EmergencyMode,
            vec![
                FallbackAction::AbortTask,
                FallbackAction::NotifyHuman,
                FallbackAction::UseAlternativeStrategy,
            ],
        );

        Self {
            current_level: DegradationLevel::FullService,
            triggers: vec![
                FallbackTrigger::HighLatency,
                FallbackTrigger::LowSuccessRate,
                FallbackTrigger::ResourceExhaustion,
                FallbackTrigger::BudgetExceeded,
            ],
            actions,
        }
    }

    pub fn escalate(&mut self) -> Vec<FallbackAction> {
        match self.current_level {
            DegradationLevel::FullService => {
                self.current_level = DegradationLevel::ReducedQuality;
            }
            DegradationLevel::ReducedQuality => {
                self.current_level = DegradationLevel::EssentialOnly;
            }
            DegradationLevel::EssentialOnly => {
                self.current_level = DegradationLevel::ReadOnly;
            }
            DegradationLevel::ReadOnly => {
                self.current_level = DegradationLevel::EmergencyMode;
            }
            DegradationLevel::EmergencyMode => {
                // Already at highest level
            }
        }

        self.actions
            .get(&self.current_level)
            .cloned()
            .unwrap_or_default()
    }

    pub fn deescalate(&mut self) -> Vec<FallbackAction> {
        match self.current_level {
            DegradationLevel::EmergencyMode => {
                self.current_level = DegradationLevel::ReadOnly;
            }
            DegradationLevel::ReadOnly => {
                self.current_level = DegradationLevel::EssentialOnly;
            }
            DegradationLevel::EssentialOnly => {
                self.current_level = DegradationLevel::ReducedQuality;
            }
            DegradationLevel::ReducedQuality => {
                self.current_level = DegradationLevel::FullService;
            }
            DegradationLevel::FullService => {
                // Already at lowest level
            }
        }

        self.actions
            .get(&self.current_level)
            .cloned()
            .unwrap_or_default()
    }
}

/// Bulkhead configuration for failure isolation.
#[derive(Debug, Clone)]
pub struct BulkheadConfig {
    pub max_concurrent_calls: u32,
    pub max_wait_time: Duration,
    pub isolation_groups: Vec<String>,
}

impl Default for BulkheadConfig {
    fn default() -> Self {
        Self {
            max_concurrent_calls: 10,
            max_wait_time: Duration::from_secs(5),
            isolation_groups: vec!["default".to_string()],
        }
    }
}

/// Bulkhead for isolating failures.
#[derive(Debug)]
pub struct Bulkhead {
    pub id: String,
    pub concurrent_calls: u32,
    pub config: BulkheadConfig,
}

impl Bulkhead {
    pub fn new(id: String) -> Self {
        Self {
            id,
            concurrent_calls: 0,
            config: BulkheadConfig::default(),
        }
    }

    pub fn can_execute(&self) -> bool {
        self.concurrent_calls < self.config.max_concurrent_calls
    }

    pub fn acquire(&mut self) -> bool {
        if self.can_execute() {
            self.concurrent_calls += 1;
            true
        } else {
            false
        }
    }

    pub fn release(&mut self) {
        if self.concurrent_calls > 0 {
            self.concurrent_calls -= 1;
        }
    }
}

/// Fallback manager orchestrating all fault tolerance mechanisms.
#[derive(Debug)]
pub struct FallbackManager {
    pub circuit_breakers: Arc<RwLock<HashMap<String, CircuitBreaker>>>,
    pub retry_mechanism: RetryMechanism,
    pub graceful_degradation: Arc<Mutex<GracefulDegradation>>,
    pub bulkheads: Arc<RwLock<HashMap<String, Bulkhead>>>,
    pub performance_monitor: Option<Arc<PerformanceMonitor>>,
    pub strategies: Vec<FallbackStrategy>,
}

impl FallbackManager {
    pub fn new() -> Self {
        Self {
            circuit_breakers: Arc::new(RwLock::new(HashMap::new())),
            retry_mechanism: RetryMechanism::new(RetryConfig::default()),
            graceful_degradation: Arc::new(Mutex::new(GracefulDegradation::new())),
            bulkheads: Arc::new(RwLock::new(HashMap::new())),
            performance_monitor: None,
            strategies: Vec::new(),
        }
    }

    pub fn with_performance_monitor(
        mut self,
        performance_monitor: Arc<PerformanceMonitor>,
    ) -> Self {
        self.performance_monitor = Some(performance_monitor);
        self
    }

    pub fn with_strategies(mut self, strategies: Vec<FallbackStrategy>) -> Self {
        self.strategies = strategies;
        self
    }

    pub async fn add_circuit_breaker(&self, id: String, config: Option<CircuitBreakerConfig>) {
        let mut circuit_breakers = self.circuit_breakers.write().await;
        let circuit_breaker = match config {
            Some(config) => CircuitBreaker::with_config(id.clone(), config),
            None => CircuitBreaker::new(id.clone()),
        };
        circuit_breakers.insert(id, circuit_breaker);
    }

    pub async fn add_bulkhead(&self, id: String, config: Option<BulkheadConfig>) {
        let mut bulkheads = self.bulkheads.write().await;
        let mut bulkhead = Bulkhead::new(id.clone());
        if let Some(config) = config {
            bulkhead.config = config;
        }
        bulkheads.insert(id, bulkhead);
    }

    pub async fn record_success(&self, circuit_breaker_id: &str) {
        let mut circuit_breakers = self.circuit_breakers.write().await;
        if let Some(circuit_breaker) = circuit_breakers.get_mut(circuit_breaker_id) {
            circuit_breaker.record_success();
        }
    }

    pub async fn record_failure(&self, circuit_breaker_id: &str) {
        let mut circuit_breakers = self.circuit_breakers.write().await;
        if let Some(circuit_breaker) = circuit_breakers.get_mut(circuit_breaker_id) {
            circuit_breaker.record_failure();
        }
    }

    pub async fn is_circuit_breaker_available(&self, circuit_breaker_id: &str) -> bool {
        let circuit_breakers = self.circuit_breakers.read().await;
        circuit_breakers
            .get(circuit_breaker_id)
            .map(|cb| cb.is_available())
            .unwrap_or(true)
    }

    pub async fn acquire_bulkhead(&self, bulkhead_id: &str) -> bool {
        let mut bulkheads = self.bulkheads.write().await;
        if let Some(bulkhead) = bulkheads.get_mut(bulkhead_id) {
            bulkhead.acquire()
        } else {
            // Create default bulkhead if it doesn't exist
            let mut bulkhead = Bulkhead::new(bulkhead_id.to_string());
            let result = bulkhead.acquire();
            bulkheads.insert(bulkhead_id.to_string(), bulkhead);
            result
        }
    }

    pub async fn release_bulkhead(&self, bulkhead_id: &str) {
        let mut bulkheads = self.bulkheads.write().await;
        if let Some(bulkhead) = bulkheads.get_mut(bulkhead_id) {
            bulkhead.release();
        }
    }

    pub async fn handle_alert(&self, alert: &Alert) -> Vec<FallbackAction> {
        let mut actions = Vec::new();

        // Map alert type to fallback trigger based on alert metadata
        let trigger = match alert.alert_type {
            AlertType::Performance => {
                // Check alert metric to determine specific trigger
                match alert.metric {
                    PerformanceMetric::Latency => Some(FallbackTrigger::HighLatency),
                    PerformanceMetric::SuccessRate => Some(FallbackTrigger::LowSuccessRate),
                    _ => None,
                }
            }
            AlertType::Resource => Some(FallbackTrigger::ResourceExhaustion),
            AlertType::Cost => Some(FallbackTrigger::BudgetExceeded),
            AlertType::Quality => Some(FallbackTrigger::QualityBelowThreshold),
            AlertType::Availability => Some(FallbackTrigger::AvailabilityBelowThreshold),
            _ => None,
        };

        if let Some(trigger) = trigger {
            // Find matching strategies
            for strategy in &self.strategies {
                if strategy.trigger == trigger {
                    actions.push(strategy.action.clone());
                }
            }

            // If no specific strategy, use graceful degradation
            if actions.is_empty() && alert.level == AlertLevel::Critical {
                let mut degradation = self.graceful_degradation.lock().await;
                actions = degradation.escalate();
            }
        }

        actions
    }

    pub async fn execute_with_retry<F, T, E>(&self, operation: F) -> Result<T, E>
    where
        F: Fn() -> Result<T, E> + Clone,
        E: std::fmt::Debug,
    {
        let mut last_error = None;

        for attempt in 1..=self.retry_mechanism.config.max_attempts {
            match operation() {
                Ok(result) => return Ok(result),
                Err(err) => {
                    last_error = Some(err);

                    if attempt < self.retry_mechanism.config.max_attempts {
                        let delay = self.retry_mechanism.calculate_delay(attempt);
                        tokio::time::sleep(delay).await;
                    }
                }
            }
        }

        Err(last_error.unwrap())
    }

    pub async fn execute_with_timeout<F, T>(
        &self,
        operation: F,
        timeout: Duration,
    ) -> Result<T, String>
    where
        F: std::future::Future<Output = T> + Send + 'static,
        T: Send + 'static,
    {
        tokio::time::timeout(timeout, operation)
            .await
            .map_err(|_| format!("Operation timed out after {timeout:?}"))
    }

    pub async fn get_current_degradation_level(&self) -> DegradationLevel {
        let degradation = self.graceful_degradation.lock().await;
        degradation.current_level.clone()
    }

    pub async fn get_recommended_actions(&self, trigger: FallbackTrigger) -> Vec<FallbackAction> {
        let mut actions = Vec::new();

        // Check configured strategies
        for strategy in &self.strategies {
            if strategy.trigger == trigger {
                actions.push(strategy.action.clone());
            }
        }

        // If no specific strategy, provide default recommendations
        if actions.is_empty() {
            actions = match trigger {
                FallbackTrigger::HighLatency => vec![
                    FallbackAction::SwitchExecutor,
                    FallbackAction::RetryWithBackoff,
                ],
                FallbackTrigger::LowSuccessRate => vec![
                    FallbackAction::SwitchExecutor,
                    FallbackAction::ReduceQuality,
                ],
                FallbackTrigger::HighErrorRate => vec![
                    FallbackAction::SwitchExecutor,
                    FallbackAction::RetryWithBackoff,
                    FallbackAction::NotifyHuman,
                ],
                FallbackTrigger::ResourceExhaustion => vec![
                    FallbackAction::ScaleResources,
                    FallbackAction::SwitchExecutor,
                ],
                FallbackTrigger::BudgetExceeded => vec![
                    FallbackAction::ReduceQuality,
                    FallbackAction::SwitchExecutor,
                ],
                FallbackTrigger::QualityBelowThreshold => vec![
                    FallbackAction::SwitchExecutor,
                    FallbackAction::UseAlternativeStrategy,
                ],
                FallbackTrigger::AvailabilityBelowThreshold => vec![
                    FallbackAction::SwitchExecutor,
                    FallbackAction::ScaleResources,
                    FallbackAction::NotifyHuman,
                ],
                FallbackTrigger::Timeout => vec![
                    FallbackAction::RetryWithBackoff,
                    FallbackAction::SwitchExecutor,
                    FallbackAction::AbortTask,
                ],
            };
        }

        actions
    }
}

impl Default for FallbackManager {
    fn default() -> Self {
        Self::new()
    }
}
