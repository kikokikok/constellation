//! Performance monitoring and optimization for hybrid agents.
//!
//! Implements real-time performance monitoring, predictive scaling, and optimization
//! algorithms that integrate with the resource manager for continuous optimization.
//!
//! Based on Task 3.5 from the "incorporate-edge-research" OpenSpec change proposal.

use std::collections::{HashMap, VecDeque};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use crate::hybrid::coordinator::{PerformanceMetrics, TaskResult};
use crate::models::hybrid_agent::{PerformanceTargets, ResourceAllocation};

/// Performance monitor for real-time monitoring and optimization.
#[derive(Debug, Clone)]
pub struct PerformanceMonitor {
    /// Current performance metrics.
    current_metrics: Arc<Mutex<PerformanceMetrics>>,

    /// Historical performance data.
    performance_history: Arc<Mutex<PerformanceHistory>>,

    /// Performance targets.
    targets: PerformanceTargets,

    /// Alert thresholds.
    alert_thresholds: AlertThresholds,

    /// Optimization engine.
    optimization_engine: Arc<Mutex<OptimizationEngine>>,

    /// Predictive scaling model.
    predictive_scaler: Arc<Mutex<PredictiveScaler>>,

    /// Alert subscribers.
    alert_subscribers: Arc<Mutex<Vec<AlertSubscriber>>>,
}

/// Performance history with detailed metrics.
#[derive(Debug, Clone)]
pub struct PerformanceHistory {
    /// Task results history.
    task_results: VecDeque<TaskResult>,

    /// Metrics history.
    metrics_history: VecDeque<PerformanceMetrics>,

    /// Resource utilization history.
    resource_utilization: VecDeque<ResourceUtilization>,

    /// Cost history.
    cost_history: VecDeque<CostRecord>,

    /// Scaling events.
    scaling_events: VecDeque<ScalingEvent>,

    /// Alert history.
    alert_history: VecDeque<Alert>,

    /// Maximum history length.
    max_history_length: usize,

    /// Window size for moving averages.
    window_size: usize,
}

/// Resource utilization metrics.
#[derive(Debug, Clone)]
pub struct ResourceUtilization {
    /// Timestamp.
    pub timestamp: Instant,

    /// CPU utilization (0.0 to 1.0).
    pub cpu_utilization: f64,

    /// Memory utilization (0.0 to 1.0).
    pub memory_utilization: f64,

    /// GPU utilization (0.0 to 1.0).
    pub gpu_utilization: Option<f64>,

    /// Network utilization (0.0 to 1.0).
    pub network_utilization: f64,

    /// Disk utilization (0.0 to 1.0).
    pub disk_utilization: f64,

    /// Executor utilization (0.0 to 1.0).
    pub executor_utilization: f64,
}

/// Cost record for budget tracking.
#[derive(Debug, Clone)]
pub struct CostRecord {
    /// Timestamp.
    pub timestamp: Instant,

    /// Task ID.
    pub task_id: uuid::Uuid,

    /// Executor ID.
    pub executor_id: String,

    /// Cost incurred.
    pub cost: f64,

    /// Resource cost breakdown.
    pub resource_cost: ResourceCost,

    /// Quality-adjusted cost.
    pub quality_adjusted_cost: f64,
}

/// Resource cost breakdown.
#[derive(Debug, Clone)]
pub struct ResourceCost {
    /// CPU cost.
    pub cpu_cost: f64,

    /// Memory cost.
    pub memory_cost: f64,

    /// GPU cost.
    pub gpu_cost: Option<f64>,

    /// Network cost.
    pub network_cost: f64,

    /// Storage cost.
    pub storage_cost: f64,
}

/// Scaling event record.
#[derive(Debug, Clone)]
pub struct ScalingEvent {
    /// Timestamp.
    pub timestamp: Instant,

    /// Scaling type.
    pub scaling_type: ScalingType,

    /// Previous resource allocation.
    pub previous_allocation: ResourceAllocation,

    /// New resource allocation.
    pub new_allocation: ResourceAllocation,

    /// Trigger reason.
    pub trigger: ScalingTrigger,

    /// Performance impact.
    pub performance_impact: Option<PerformanceImpact>,
}

/// Scaling type.
#[derive(Debug, Clone)]
pub enum ScalingType {
    /// Horizontal scaling (add/remove instances).
    Horizontal,

    /// Vertical scaling (increase/decrease resources per instance).
    Vertical,

    /// Hybrid scaling (combination).
    Hybrid,

    /// Burstable scaling (quick scale-out).
    Burstable,

    /// Spot scaling (cost-optimized).
    Spot,
}

/// Scaling trigger.
#[derive(Debug, Clone)]
pub enum ScalingTrigger {
    /// Performance-based trigger.
    Performance(PerformanceTrigger),

    /// Predictive trigger.
    Predictive(PredictiveTrigger),

    /// Schedule-based trigger.
    Scheduled(ScheduleTrigger),

    /// Manual trigger.
    Manual,

    /// Cost-based trigger.
    Cost(CostTrigger),
}

/// Performance trigger.
#[derive(Debug, Clone)]
pub struct PerformanceTrigger {
    /// Metric that triggered scaling.
    pub metric: PerformanceMetric,

    /// Threshold value.
    pub threshold: f64,

    /// Actual value.
    pub actual_value: f64,

    /// Duration above threshold.
    pub duration_above_threshold: Duration,
}

/// Performance metric.
#[derive(Debug, Clone)]
pub enum PerformanceMetric {
    /// CPU utilization.
    CpuUtilization,

    /// Memory utilization.
    MemoryUtilization,

    /// GPU utilization.
    GpuUtilization,

    /// Network utilization.
    NetworkUtilization,

    /// Latency.
    Latency,

    /// Throughput.
    Throughput,

    /// Success rate.
    SuccessRate,

    /// Quality score.
    QualityScore,

    /// Cost efficiency.
    CostEfficiency,
}

/// Predictive trigger.
#[derive(Debug, Clone)]
pub struct PredictiveTrigger {
    /// Predicted metric.
    pub predicted_metric: PerformanceMetric,

    /// Predicted value.
    pub predicted_value: f64,

    /// Prediction confidence.
    pub confidence: f64,

    /// Time horizon.
    pub time_horizon: Duration,
}

/// Schedule trigger.
#[derive(Debug, Clone)]
pub struct ScheduleTrigger {
    /// Schedule name.
    pub schedule_name: String,

    /// Scheduled time.
    pub scheduled_time: Instant,

    /// Recurrence pattern.
    pub recurrence: Option<RecurrencePattern>,
}

/// Recurrence pattern.
#[derive(Debug, Clone)]
pub enum RecurrencePattern {
    /// Daily recurrence.
    Daily,

    /// Weekly recurrence.
    Weekly(u8), // Day of week (0-6, Sunday=0)

    /// Monthly recurrence.
    Monthly(u8), // Day of month (1-31)

    /// Custom cron expression.
    Cron(String),
}

/// Cost trigger.
#[derive(Debug, Clone)]
pub struct CostTrigger {
    /// Cost metric.
    pub cost_metric: CostMetric,

    /// Threshold value.
    pub threshold: f64,

    /// Actual value.
    pub actual_value: f64,

    /// Budget remaining.
    pub budget_remaining: f64,
}

/// Cost metric.
#[derive(Debug, Clone)]
pub enum CostMetric {
    /// Total cost.
    TotalCost,

    /// Cost per task.
    CostPerTask,

    /// Resource cost.
    ResourceCost,

    /// Quality-adjusted cost.
    QualityAdjustedCost,

    /// Budget utilization.
    BudgetUtilization,
}

/// Performance impact of scaling.
#[derive(Debug, Clone)]
pub struct PerformanceImpact {
    /// Impact on latency.
    pub latency_impact: f64,

    /// Impact on throughput.
    pub throughput_impact: f64,

    /// Impact on success rate.
    pub success_rate_impact: f64,

    /// Impact on quality.
    pub quality_impact: f64,

    /// Impact on cost.
    pub cost_impact: f64,

    /// Impact on resource utilization.
    pub resource_utilization_impact: f64,
}

/// Alert thresholds for performance monitoring.
#[derive(Debug, Clone)]
pub struct AlertThresholds {
    /// Critical alert thresholds.
    pub critical: AlertLevelThresholds,

    /// Warning alert thresholds.
    pub warning: AlertLevelThresholds,

    /// Info alert thresholds.
    pub info: AlertLevelThresholds,

    /// Minimum duration before alert.
    pub min_duration_before_alert: Duration,

    /// Cooldown period between alerts.
    pub cooldown_period: Duration,
}

/// Alert level thresholds.
#[derive(Debug, Clone)]
pub struct AlertLevelThresholds {
    /// CPU utilization threshold.
    pub cpu_utilization: f64,

    /// Memory utilization threshold.
    pub memory_utilization: f64,

    /// GPU utilization threshold.
    pub gpu_utilization: Option<f64>,

    /// Latency threshold.
    pub latency_ms: u32,

    /// Success rate threshold.
    pub success_rate: f64,

    /// Quality score threshold.
    pub quality_score: f64,

    /// Cost efficiency threshold.
    pub cost_efficiency: f64,

    /// Availability threshold.
    pub availability: f64,
}

/// Alert for performance issues.
#[derive(Debug, Clone)]
pub struct Alert {
    /// Alert ID.
    pub id: uuid::Uuid,

    /// Timestamp.
    pub timestamp: Instant,

    /// Alert level.
    pub level: AlertLevel,

    /// Alert type.
    pub alert_type: AlertType,

    /// Metric that triggered alert.
    pub metric: PerformanceMetric,

    /// Threshold value.
    pub threshold: f64,

    /// Actual value.
    pub actual_value: f64,

    /// Duration above threshold.
    pub duration_above_threshold: Duration,

    /// Message.
    pub message: String,

    /// Recommendations.
    pub recommendations: Vec<String>,

    /// Acknowledged flag.
    pub acknowledged: bool,

    /// Resolved flag.
    pub resolved: bool,
}

/// Alert level.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AlertLevel {
    /// Critical alert.
    Critical,

    /// Warning alert.
    Warning,

    /// Info alert.
    Info,
}

/// Alert type.
#[derive(Debug, Clone, PartialEq)]
pub enum AlertType {
    /// Performance alert.
    Performance,

    /// Resource alert.
    Resource,

    /// Cost alert.
    Cost,

    /// Availability alert.
    Availability,

    /// Quality alert.
    Quality,

    /// Security alert.
    Security,
}

/// Alert subscriber for notifications.
#[derive(Debug, Clone)]
pub struct AlertSubscriber {
    /// Subscriber ID.
    pub id: String,

    /// Notification channel.
    pub channel: NotificationChannel,

    /// Alert levels to receive.
    pub alert_levels: Vec<AlertLevel>,

    /// Alert types to receive.
    pub alert_types: Vec<AlertType>,

    /// Cooldown period between notifications.
    pub cooldown_period: Duration,

    /// Last notification time.
    pub last_notification: Option<Instant>,
}

/// Notification channel.
#[derive(Debug, Clone)]
pub enum NotificationChannel {
    /// Email notification.
    Email(String),

    /// Webhook notification.
    Webhook(String),

    /// Slack notification.
    Slack(String),

    /// PagerDuty notification.
    PagerDuty(String),

    /// Custom webhook.
    CustomWebhook {
        /// Webhook URL.
        url: String,

        /// HTTP method.
        method: String,

        /// Headers.
        headers: HashMap<String, String>,
    },
}

impl PerformanceMonitor {
    /// Create a new performance monitor.
    pub fn new(targets: PerformanceTargets) -> Self {
        let current_metrics = PerformanceMetrics {
            throughput_tps: 0.0,
            avg_latency_ms: 0.0,
            success_rate: 0.0,
            avg_quality_score: 0.0,
            resource_utilization: 0.0,
            cost_efficiency: 0.0,
            availability: 1.0,
        };

        let alert_thresholds = AlertThresholds::default();
        let optimization_engine = OptimizationEngine::default();
        let predictive_scaler = PredictiveScaler::default();

        Self {
            current_metrics: Arc::new(Mutex::new(current_metrics)),
            performance_history: Arc::new(Mutex::new(PerformanceHistory::new())),
            targets,
            alert_thresholds,
            optimization_engine: Arc::new(Mutex::new(optimization_engine)),
            predictive_scaler: Arc::new(Mutex::new(predictive_scaler)),
            alert_subscribers: Arc::new(Mutex::new(Vec::new())),
        }
    }

    /// Update performance metrics with a new task result.
    pub fn update_with_task_result(&self, result: &TaskResult) -> Result<(), String> {
        let mut history = self.performance_history.lock().unwrap();
        let mut current_metrics = self.current_metrics.lock().unwrap();

        // Add to history
        history.add_task_result(result.clone());

        // Update current metrics
        self.update_current_metrics(&mut current_metrics, &history);

        // Check for alerts
        self.check_alerts(&current_metrics, &history);

        // Run optimization if needed
        self.run_optimization_if_needed(&history);

        // Run predictive scaling if needed
        self.run_predictive_scaling_if_needed(&history);

        Ok(())
    }

    /// Update current metrics based on history.
    fn update_current_metrics(
        &self,
        current_metrics: &mut PerformanceMetrics,
        history: &PerformanceHistory,
    ) {
        if history.task_results.is_empty() {
            return;
        }

        // Calculate metrics from recent history
        let recent_results = history.get_recent_results(Duration::from_secs(60)); // Last 60 seconds

        if recent_results.is_empty() {
            return;
        }

        // Calculate throughput
        let time_window = 60.0; // seconds
        current_metrics.throughput_tps = recent_results.len() as f64 / time_window;

        // Calculate average latency
        let total_latency: f64 = recent_results
            .iter()
            .map(|r| r.execution_time_ms as f64)
            .sum();
        current_metrics.avg_latency_ms = total_latency / recent_results.len() as f64;

        // Calculate success rate
        let success_count = recent_results.iter().filter(|r| r.success).count();
        current_metrics.success_rate = success_count as f64 / recent_results.len() as f64;

        // Calculate average quality score
        let total_quality: f64 = recent_results.iter().map(|r| r.quality_score).sum();
        current_metrics.avg_quality_score = total_quality / recent_results.len() as f64;

        // Update other metrics (simplified for now)
        current_metrics.resource_utilization = 0.7; // Placeholder
        current_metrics.cost_efficiency = 0.8; // Placeholder
        current_metrics.availability = 0.99; // Placeholder
    }

    /// Check for performance alerts.
    fn check_alerts(
        &self,
        current_metrics: &PerformanceMetrics,
        history: &PerformanceHistory,
    ) -> Vec<Alert> {
        let mut alerts = Vec::new();

        // Check critical thresholds
        if current_metrics.success_rate < self.alert_thresholds.critical.success_rate {
            alerts.push(self.create_alert(
                AlertLevel::Critical,
                AlertType::Performance,
                PerformanceMetric::SuccessRate,
                self.alert_thresholds.critical.success_rate,
                current_metrics.success_rate,
                Duration::from_secs(0), // Simplified
                format!(
                    "Critical: Success rate {:.2}% below threshold {:.2}%",
                    current_metrics.success_rate * 100.0,
                    self.alert_thresholds.critical.success_rate * 100.0
                ),
            ));
        }

        if current_metrics.avg_latency_ms > self.alert_thresholds.critical.latency_ms as f64 {
            alerts.push(self.create_alert(
                AlertLevel::Critical,
                AlertType::Performance,
                PerformanceMetric::Latency,
                self.alert_thresholds.critical.latency_ms as f64,
                current_metrics.avg_latency_ms,
                Duration::from_secs(0),
                format!(
                    "Critical: Latency {:.0}ms above threshold {}ms",
                    current_metrics.avg_latency_ms, self.alert_thresholds.critical.latency_ms
                ),
            ));
        }

        // Check warning thresholds
        if current_metrics.success_rate < self.alert_thresholds.warning.success_rate {
            alerts.push(self.create_alert(
                AlertLevel::Warning,
                AlertType::Performance,
                PerformanceMetric::SuccessRate,
                self.alert_thresholds.warning.success_rate,
                current_metrics.success_rate,
                Duration::from_secs(0),
                format!(
                    "Warning: Success rate {:.2}% below threshold {:.2}%",
                    current_metrics.success_rate * 100.0,
                    self.alert_thresholds.warning.success_rate * 100.0
                ),
            ));
        }

        // Add more threshold checks as needed

        // Notify subscribers
        self.notify_subscribers(&alerts);

        // Add to history
        let mut history_lock = self.performance_history.lock().unwrap();
        for alert in &alerts {
            history_lock.add_alert(alert.clone());
        }

        alerts
    }

    /// Create an alert.
    fn create_alert(
        &self,
        level: AlertLevel,
        alert_type: AlertType,
        metric: PerformanceMetric,
        threshold: f64,
        actual_value: f64,
        duration: Duration,
        message: String,
    ) -> Alert {
        Alert {
            id: uuid::Uuid::new_v4(),
            timestamp: Instant::now(),
            level,
            alert_type,
            metric: metric.clone(),
            threshold,
            actual_value,
            duration_above_threshold: duration,
            message,
            recommendations: self.generate_recommendations(&metric, actual_value, threshold),
            acknowledged: false,
            resolved: false,
        }
    }

    /// Generate recommendations for an alert.
    fn generate_recommendations(
        &self,
        metric: &PerformanceMetric,
        actual_value: f64,
        threshold: f64,
    ) -> Vec<String> {
        match metric {
            PerformanceMetric::SuccessRate => {
                vec![
                    "Check executor health and availability".to_string(),
                    "Review task complexity and resource allocation".to_string(),
                    "Consider scaling up resources for critical tasks".to_string(),
                ]
            }
            PerformanceMetric::Latency => {
                vec![
                    "Optimize task assignment to lower-latency executors".to_string(),
                    "Consider horizontal scaling to distribute load".to_string(),
                    "Review network latency and connectivity".to_string(),
                ]
            }
            PerformanceMetric::CpuUtilization => {
                vec![
                    "Scale CPU resources vertically or horizontally".to_string(),
                    "Optimize task scheduling to reduce peak load".to_string(),
                    "Consider moving CPU-intensive tasks to dedicated executors".to_string(),
                ]
            }
            _ => vec!["Review system performance and configuration".to_string()],
        }
    }

    /// Notify alert subscribers.
    fn notify_subscribers(&self, alerts: &[Alert]) {
        let mut subscribers = self.alert_subscribers.lock().unwrap();
        let now = Instant::now();

        for subscriber in subscribers.iter_mut() {
            // Check cooldown period
            if let Some(last_notification) = subscriber.last_notification {
                if now.duration_since(last_notification) < subscriber.cooldown_period {
                    continue;
                }
            }

            // Filter alerts for this subscriber
            let relevant_alerts: Vec<&Alert> = alerts
                .iter()
                .filter(|alert| {
                    subscriber.alert_levels.contains(&alert.level)
                        && subscriber.alert_types.contains(&alert.alert_type)
                })
                .collect();

            if !relevant_alerts.is_empty() {
                // Send notification (simplified for now)
                subscriber.last_notification = Some(now);
                // In a real implementation, this would send via the appropriate channel
            }
        }
    }

    /// Run optimization if needed.
    fn run_optimization_if_needed(&self, history: &PerformanceHistory) {
        let mut engine = self.optimization_engine.lock().unwrap();
        let now = Instant::now();

        // Check if it's time for optimization
        if let Some(last_optimization) = engine.last_optimization {
            if now.duration_since(last_optimization) < engine.optimization_frequency {
                return;
            }
        }

        // Check if we have enough data
        if history.task_results.len() < engine.min_samples {
            return;
        }

        // Run optimization
        match engine.optimize(history) {
            Ok(result) => {
                if result.success {
                    // Apply optimization if improvement is significant
                    if result.expected_improvement.overall_score > engine.improvement_threshold {
                        engine.current_best_allocation = Some(result.new_allocation.clone());
                        // In a real implementation, this would trigger resource reallocation
                    }
                }
            }
            Err(err) => {
                // Log optimization error
                eprintln!("Optimization failed: {}", err);
            }
        }

        engine.last_optimization = Some(now);
    }

    /// Run predictive scaling if needed.
    fn run_predictive_scaling_if_needed(&self, history: &PerformanceHistory) {
        let mut scaler = self.predictive_scaler.lock().unwrap();
        let now = Instant::now();

        // Check if it's time for prediction
        if let Some(last_prediction) = scaler.last_prediction {
            if now.duration_since(last_prediction) < scaler.prediction_frequency {
                return;
            }
        }

        // Make prediction
        match scaler.predict(history) {
            Ok(recommendation) => {
                if recommendation.confidence > scaler.confidence_threshold {
                    scaler.last_recommendation = Some(recommendation);
                    // In a real implementation, this would trigger scaling
                }
            }
            Err(err) => {
                // Log prediction error
                eprintln!("Prediction failed: {}", err);
            }
        }

        scaler.last_prediction = Some(now);
    }

    /// Get current performance metrics.
    pub fn get_current_metrics(&self) -> PerformanceMetrics {
        let metrics = self.current_metrics.lock().unwrap();
        metrics.clone()
    }

    /// Get performance history.
    pub fn get_performance_history(&self) -> PerformanceHistory {
        let history = self.performance_history.lock().unwrap();
        history.clone()
    }

    /// Get active alerts.
    pub fn get_active_alerts(&self) -> Vec<Alert> {
        let history = self.performance_history.lock().unwrap();
        history
            .alert_history
            .iter()
            .filter(|alert| !alert.resolved)
            .cloned()
            .collect()
    }

    /// Add an alert subscriber.
    pub fn add_subscriber(&self, subscriber: AlertSubscriber) {
        let mut subscribers = self.alert_subscribers.lock().unwrap();
        subscribers.push(subscriber);
    }

    /// Remove an alert subscriber.
    pub fn remove_subscriber(&self, subscriber_id: &str) {
        let mut subscribers = self.alert_subscribers.lock().unwrap();
        subscribers.retain(|s| s.id != subscriber_id);
    }

    /// Acknowledge an alert.
    pub fn acknowledge_alert(&self, alert_id: uuid::Uuid) -> Result<(), String> {
        let mut history = self.performance_history.lock().unwrap();
        for alert in history.alert_history.iter_mut() {
            if alert.id == alert_id {
                alert.acknowledged = true;
                return Ok(());
            }
        }
        Err("Alert not found".to_string())
    }

    /// Resolve an alert.
    pub fn resolve_alert(&self, alert_id: uuid::Uuid) -> Result<(), String> {
        let mut history = self.performance_history.lock().unwrap();
        for alert in history.alert_history.iter_mut() {
            if alert.id == alert_id {
                alert.resolved = true;
                return Ok(());
            }
        }
        Err("Alert not found".to_string())
    }

    /// Get optimization recommendations.
    pub fn get_optimization_recommendations(&self) -> Vec<OptimizationResult> {
        let engine = self.optimization_engine.lock().unwrap();
        engine.optimization_history.iter().cloned().collect()
    }

    /// Get scaling recommendations.
    pub fn get_scaling_recommendations(&self) -> Option<ScalingRecommendation> {
        let scaler = self.predictive_scaler.lock().unwrap();
        scaler.last_recommendation.clone()
    }

    /// Get performance trends.
    pub fn get_performance_trends(&self, window: Duration) -> PerformanceTrends {
        let history = self.performance_history.lock().unwrap();
        history.calculate_trends(window)
    }
}

// Implementation for PerformanceHistory
impl PerformanceHistory {
    /// Create a new performance history.
    pub fn new() -> Self {
        Self {
            task_results: VecDeque::new(),
            metrics_history: VecDeque::new(),
            resource_utilization: VecDeque::new(),
            cost_history: VecDeque::new(),
            scaling_events: VecDeque::new(),
            alert_history: VecDeque::new(),
            max_history_length: 1000,
            window_size: 60,
        }
    }

    /// Add a task result to history.
    pub fn add_task_result(&mut self, result: TaskResult) {
        self.task_results.push_back(result);
        if self.task_results.len() > self.max_history_length {
            self.task_results.pop_front();
        }
    }

    /// Add a metrics snapshot to history.
    pub fn add_metrics(&mut self, metrics: PerformanceMetrics) {
        self.metrics_history.push_back(metrics);
        if self.metrics_history.len() > self.max_history_length {
            self.metrics_history.pop_front();
        }
    }

    /// Add resource utilization to history.
    pub fn add_resource_utilization(&mut self, utilization: ResourceUtilization) {
        self.resource_utilization.push_back(utilization);
        if self.resource_utilization.len() > self.max_history_length {
            self.resource_utilization.pop_front();
        }
    }

    /// Add a cost record to history.
    pub fn add_cost_record(&mut self, record: CostRecord) {
        self.cost_history.push_back(record);
        if self.cost_history.len() > self.max_history_length {
            self.cost_history.pop_front();
        }
    }

    /// Add a scaling event to history.
    pub fn add_scaling_event(&mut self, event: ScalingEvent) {
        self.scaling_events.push_back(event);
        if self.scaling_events.len() > self.max_history_length {
            self.scaling_events.pop_front();
        }
    }

    /// Add an alert to history.
    pub fn add_alert(&mut self, alert: Alert) {
        self.alert_history.push_back(alert);
        if self.alert_history.len() > self.max_history_length {
            self.alert_history.pop_front();
        }
    }

    /// Get recent task results within a time window.
    pub fn get_recent_results(&self, window: Duration) -> Vec<TaskResult> {
        let now = Instant::now();
        self.task_results
            .iter()
            .filter(|result| {
                // Convert chrono::DateTime to Instant (simplified)
                let result_time = Instant::now(); // Placeholder
                now.duration_since(result_time) <= window
            })
            .cloned()
            .collect()
    }

    /// Calculate performance trends over a window.
    pub fn calculate_trends(&self, window: Duration) -> PerformanceTrends {
        let recent_results = self.get_recent_results(window);
        
        if recent_results.is_empty() {
            return PerformanceTrends::default();
        }

        // Calculate basic statistics
        let latencies: Vec<f64> = recent_results
            .iter()
            .map(|r| r.execution_time_ms as f64)
            .collect();
        let success_rates: Vec<f64> = recent_results
            .iter()
            .map(|r| if r.success { 1.0 } else { 0.0 })
            .collect();
        let quality_scores: Vec<f64> = recent_results
            .iter()
            .map(|r| r.quality_score)
            .collect();

        PerformanceTrends {
            latency_trend: calculate_trend(&latencies),
            success_rate_trend: calculate_trend(&success_rates),
            quality_trend: calculate_trend(&quality_scores),
            throughput_trend: recent_results.len() as f64 / window.as_secs_f64(),
            cost_trend: recent_results.iter().map(|r| r.cost).sum::<f64>() / recent_results.len() as f64,
        }
    }
}

/// Performance trends.
#[derive(Debug, Clone)]
pub struct PerformanceTrends {
    /// Latency trend.
    pub latency_trend: f64,

    /// Success rate trend.
    pub success_rate_trend: f64,

    /// Quality trend.
    pub quality_trend: f64,

    /// Throughput trend.
    pub throughput_trend: f64,

    /// Cost trend.
    pub cost_trend: f64,
}

impl Default for PerformanceTrends {
    fn default() -> Self {
        Self {
            latency_trend: 0.0,
            success_rate_trend: 0.0,
            quality_trend: 0.0,
            throughput_trend: 0.0,
            cost_trend: 0.0,
        }
    }
}

/// Calculate trend (slope) of a time series.
pub(crate) fn calculate_trend(values: &[f64]) -> f64 {
    if values.len() < 2 {
        return 0.0;
    }

    let n = values.len() as f64;
    let sum_x: f64 = (0..values.len()).map(|i| i as f64).sum();
    let sum_y: f64 = values.iter().sum();
    let sum_xy: f64 = values.iter().enumerate().map(|(i, &y)| i as f64 * y).sum();
    let sum_x2: f64 = (0..values.len()).map(|i| (i as f64).powi(2)).sum();

    let numerator = n * sum_xy - sum_x * sum_y;
    let denominator = n * sum_x2 - sum_x.powi(2);

    if denominator.abs() < 1e-10 {
        0.0
    } else {
        numerator / denominator
    }
}

// Default implementations
impl Default for AlertThresholds {
    fn default() -> Self {
        Self {
            critical: AlertLevelThresholds {
                cpu_utilization: 0.9,
                memory_utilization: 0.9,
                gpu_utilization: Some(0.9),
                latency_ms: 10000,
                success_rate: 0.8,
                quality_score: 0.7,
                cost_efficiency: 0.5,
                availability: 0.95,
            },
            warning: AlertLevelThresholds {
                cpu_utilization: 0.8,
                memory_utilization: 0.8,
                gpu_utilization: Some(0.8),
                latency_ms: 5000,
                success_rate: 0.9,
                quality_score: 0.8,
                cost_efficiency: 0.7,
                availability: 0.98,
            },
            info: AlertLevelThresholds {
                cpu_utilization: 0.7,
                memory_utilization: 0.7,
                gpu_utilization: Some(0.7),
                latency_ms: 3000,
                success_rate: 0.95,
                quality_score: 0.9,
                cost_efficiency: 0.8,
                availability: 0.99,
            },
            min_duration_before_alert: Duration::from_secs(30),
            cooldown_period: Duration::from_secs(300),
        }
    }
}

// Placeholder structs for optimization and prediction
#[derive(Debug, Clone)]
pub struct OptimizationEngine {
    pub algorithms: Vec<OptimizationAlgorithm>,
    pub optimization_history: VecDeque<OptimizationResult>,
    pub current_best_allocation: Option<ResourceAllocation>,
    pub improvement_threshold: f64,
    pub min_samples: usize,
    pub optimization_frequency: Duration,
    pub last_optimization: Option<Instant>,
}

#[derive(Debug, Clone)]
pub enum OptimizationAlgorithm {
    GradientDescent,
    Bayesian,
    Genetic,
    RuleBased,
}

#[derive(Debug, Clone)]
pub struct OptimizationResult {
    pub timestamp: Instant,
    pub algorithm: OptimizationAlgorithm,
    pub previous_allocation: ResourceAllocation,
    pub new_allocation: ResourceAllocation,
    pub expected_improvement: PerformanceImprovement,
    pub actual_improvement: Option<PerformanceImprovement>,
    pub optimization_duration: Duration,
    pub success: bool,
    pub error: Option<String>,
}

#[derive(Debug, Clone)]
pub struct PerformanceImprovement {
    pub latency_improvement: f64,
    pub throughput_improvement: f64,
    pub success_rate_improvement: f64,
    pub quality_improvement: f64,
    pub cost_improvement: f64,
    pub resource_utilization_improvement: f64,
    pub overall_score: f64,
}

#[derive(Debug, Clone)]
pub struct PredictiveScaler {
    pub models: Vec<PredictionModel>,
    pub prediction_horizon: Duration,
    pub prediction_frequency: Duration,
    pub confidence_threshold: f64,
    pub training_data: VecDeque<TrainingSample>,
    pub last_prediction: Option<Instant>,
    pub last_recommendation: Option<ScalingRecommendation>,
}

#[derive(Debug, Clone)]
pub enum PredictionModel {
    TimeSeries,
    MachineLearning,
    RuleBased,
}

#[derive(Debug, Clone)]
pub struct TrainingSample {
    pub timestamp: Instant,
    pub features: HashMap<String, f64>,
    pub target: f64,
    pub weight: f64,
}

#[derive(Debug, Clone)]
pub struct ScalingRecommendation {
    pub timestamp: Instant,
    pub recommendation_type: ScalingType,
    pub recommended_allocation: ResourceAllocation,
    pub expected_improvement: PerformanceImprovement,
    pub confidence: f64,
    pub time_to_act: Duration,
    pub priority: u32,
}

impl Default for OptimizationEngine {
    fn default() -> Self {
        Self {
            algorithms: vec![OptimizationAlgorithm::RuleBased],
            optimization_history: VecDeque::new(),
            current_best_allocation: None,
            improvement_threshold: 0.1,
            min_samples: 100,
            optimization_frequency: Duration::from_secs(300),
            last_optimization: None,
        }
    }
}

impl Default for PredictiveScaler {
    fn default() -> Self {
        Self {
            models: vec![PredictionModel::RuleBased],
            prediction_horizon: Duration::from_secs(300),
            prediction_frequency: Duration::from_secs(60),
            confidence_threshold: 0.7,
            training_data: VecDeque::new(),
            last_prediction: None,
            last_recommendation: None,
        }
    }
}

impl OptimizationEngine {
    /// Optimize resource allocation based on performance history.
    pub fn optimize(&mut self, _history: &PerformanceHistory) -> Result<OptimizationResult, String> {
        // Simplified optimization logic
        let result = OptimizationResult {
            timestamp: Instant::now(),
            algorithm: OptimizationAlgorithm::RuleBased,
            previous_allocation: ResourceAllocation::default(),
            new_allocation: ResourceAllocation::default(),
            expected_improvement: PerformanceImprovement {
                latency_improvement: 0.1,
                throughput_improvement: 0.1,
                success_rate_improvement: 0.05,
                quality_improvement: 0.05,
                cost_improvement: 0.1,
                resource_utilization_improvement: 0.1,
                overall_score: 0.08,
            },
            actual_improvement: None,
            optimization_duration: Duration::from_millis(100),
            success: true,
            error: None,
        };

        self.optimization_history.push_back(result.clone());
        if self.optimization_history.len() > 100 {
            self.optimization_history.pop_front();
        }

        Ok(result)
    }
}

impl PredictiveScaler {
    /// Predict future scaling needs.
    pub fn predict(&mut self, _history: &PerformanceHistory) -> Result<ScalingRecommendation, String> {
        // Simplified prediction logic
        let recommendation = ScalingRecommendation {
            timestamp: Instant::now(),
            recommendation_type: ScalingType::Horizontal,
            recommended_allocation: ResourceAllocation::default(),
            expected_improvement: PerformanceImprovement {
                latency_improvement: 0.05,
                throughput_improvement: 0.15,
                success_rate_improvement: 0.03,
                quality_improvement: 0.02,
                cost_improvement: 0.08,
                resource_utilization_improvement: 0.12,
                overall_score: 0.07,
            },
            confidence: 0.75,
            time_to_act: Duration::from_secs(60),
            priority: 50,
        };

        self.last_recommendation = Some(recommendation.clone());
        Ok(recommendation)
    }
}