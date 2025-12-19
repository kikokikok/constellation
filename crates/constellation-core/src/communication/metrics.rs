//! Metrics collection for communication patterns

use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{Duration, Instant};

/// Metrics for communication patterns
#[derive(Debug, Clone, Default)]
pub struct CommunicationMetrics {
    /// Request-response metrics
    pub request_response: RequestResponseMetrics,

    /// Publish-subscribe metrics
    pub publish_subscribe: PublishSubscribeMetrics,

    /// Fire-and-forget metrics
    pub fire_and_forget: FireAndForgetMetrics,

    /// Delivery guarantee metrics
    pub delivery_guarantees: DeliveryGuaranteeMetrics,

    /// Error metrics
    pub errors: ErrorMetrics,
}

/// Request-response pattern metrics
#[derive(Debug, Clone)]
pub struct RequestResponseMetrics {
    /// Total requests sent
    pub requests_sent: Arc<AtomicU64>,

    /// Total responses received
    pub responses_received: Arc<AtomicU64>,

    /// Total request timeouts
    pub request_timeouts: Arc<AtomicU64>,

    /// Total request retries
    pub request_retries: Arc<AtomicU64>,

    /// Average response time in milliseconds
    pub avg_response_time_ms: Arc<AtomicU64>,

    /// Total response time accumulator for calculating average
    pub total_response_time_ms: Arc<AtomicU64>,

    /// Response count for calculating average
    pub response_count: Arc<AtomicU64>,

    /// Requests by priority
    pub requests_by_priority: PriorityMetrics,
}

impl Default for RequestResponseMetrics {
    fn default() -> Self {
        Self {
            requests_sent: Arc::new(AtomicU64::new(0)),
            responses_received: Arc::new(AtomicU64::new(0)),
            request_timeouts: Arc::new(AtomicU64::new(0)),
            request_retries: Arc::new(AtomicU64::new(0)),
            avg_response_time_ms: Arc::new(AtomicU64::new(0)),
            total_response_time_ms: Arc::new(AtomicU64::new(0)),
            response_count: Arc::new(AtomicU64::new(0)),
            requests_by_priority: PriorityMetrics::default(),
        }
    }
}

impl RequestResponseMetrics {
    /// Record a request sent
    pub fn record_request_sent(&self, priority: crate::models::message_broker::MessagePriority) {
        self.requests_sent.fetch_add(1, Ordering::Relaxed);
        self.requests_by_priority.record(priority);
    }

    /// Record a response received
    pub fn record_response_received(&self, response_time: Duration) {
        self.responses_received.fetch_add(1, Ordering::Relaxed);

        // Update average response time
        let response_time_ms = response_time.as_millis() as u64;
        let total = self
            .total_response_time_ms
            .fetch_add(response_time_ms, Ordering::Relaxed);
        let count = self.response_count.fetch_add(1, Ordering::Relaxed);

        // Calculate new average
        let new_avg = (total + response_time_ms) / (count + 1);
        self.avg_response_time_ms.store(new_avg, Ordering::Relaxed);
    }

    /// Record a request timeout
    pub fn record_request_timeout(&self) {
        self.request_timeouts.fetch_add(1, Ordering::Relaxed);
    }

    /// Record a request retry
    pub fn record_request_retry(&self) {
        self.request_retries.fetch_add(1, Ordering::Relaxed);
    }

    /// Get current metrics snapshot
    pub fn snapshot(&self) -> RequestResponseMetricsSnapshot {
        RequestResponseMetricsSnapshot {
            requests_sent: self.requests_sent.load(Ordering::Relaxed),
            responses_received: self.responses_received.load(Ordering::Relaxed),
            request_timeouts: self.request_timeouts.load(Ordering::Relaxed),
            request_retries: self.request_retries.load(Ordering::Relaxed),
            avg_response_time_ms: self.avg_response_time_ms.load(Ordering::Relaxed),
            requests_by_priority: self.requests_by_priority.snapshot(),
        }
    }
}

/// Snapshot of request-response metrics
#[derive(Debug, Clone, serde::Serialize)]
pub struct RequestResponseMetricsSnapshot {
    pub requests_sent: u64,
    pub responses_received: u64,
    pub request_timeouts: u64,
    pub request_retries: u64,
    pub avg_response_time_ms: u64,
    pub requests_by_priority: PriorityMetricsSnapshot,
}

/// Publish-subscribe pattern metrics
#[derive(Debug, Clone)]
pub struct PublishSubscribeMetrics {
    /// Total messages published
    pub messages_published: Arc<AtomicU64>,

    /// Total messages delivered to subscribers
    pub messages_delivered: Arc<AtomicU64>,

    /// Total active subscriptions
    pub active_subscriptions: Arc<AtomicU64>,

    /// Total subscription events (subscribe/unsubscribe)
    pub subscription_events: Arc<AtomicU64>,

    /// Messages published by topic pattern
    pub messages_by_topic: TopicMetrics,
}

impl Default for PublishSubscribeMetrics {
    fn default() -> Self {
        Self {
            messages_published: Arc::new(AtomicU64::new(0)),
            messages_delivered: Arc::new(AtomicU64::new(0)),
            active_subscriptions: Arc::new(AtomicU64::new(0)),
            subscription_events: Arc::new(AtomicU64::new(0)),
            messages_by_topic: TopicMetrics::default(),
        }
    }
}

impl PublishSubscribeMetrics {
    /// Record a message published
    pub fn record_message_published(&self, topic: &str) {
        self.messages_published.fetch_add(1, Ordering::Relaxed);
        self.messages_by_topic.record_publish(topic);
    }

    /// Record a message delivered to subscribers
    pub fn record_message_delivered(&self, subscriber_count: usize) {
        self.messages_delivered
            .fetch_add(subscriber_count as u64, Ordering::Relaxed);
    }

    /// Record a subscription event
    pub fn record_subscription_event(&self, is_subscribe: bool) {
        self.subscription_events.fetch_add(1, Ordering::Relaxed);

        if is_subscribe {
            self.active_subscriptions.fetch_add(1, Ordering::Relaxed);
        } else {
            self.active_subscriptions.fetch_sub(1, Ordering::Relaxed);
        }
    }

    /// Get current metrics snapshot
    pub fn snapshot(&self) -> PublishSubscribeMetricsSnapshot {
        PublishSubscribeMetricsSnapshot {
            messages_published: self.messages_published.load(Ordering::Relaxed),
            messages_delivered: self.messages_delivered.load(Ordering::Relaxed),
            active_subscriptions: self.active_subscriptions.load(Ordering::Relaxed),
            subscription_events: self.subscription_events.load(Ordering::Relaxed),
            messages_by_topic: self.messages_by_topic.snapshot(),
        }
    }
}

/// Snapshot of publish-subscribe metrics
#[derive(Debug, Clone, serde::Serialize)]
pub struct PublishSubscribeMetricsSnapshot {
    pub messages_published: u64,
    pub messages_delivered: u64,
    pub active_subscriptions: u64,
    pub subscription_events: u64,
    pub messages_by_topic: TopicMetricsSnapshot,
}

/// Fire-and-forget pattern metrics
#[derive(Debug, Clone)]
pub struct FireAndForgetMetrics {
    /// Total notifications sent
    pub notifications_sent: Arc<AtomicU64>,

    /// Notifications by delivery guarantee
    pub notifications_by_guarantee: DeliveryGuaranteeCounts,

    /// Notifications by priority
    pub notifications_by_priority: PriorityMetrics,
}

impl Default for FireAndForgetMetrics {
    fn default() -> Self {
        Self {
            notifications_sent: Arc::new(AtomicU64::new(0)),
            notifications_by_guarantee: DeliveryGuaranteeCounts::default(),
            notifications_by_priority: PriorityMetrics::default(),
        }
    }
}

impl FireAndForgetMetrics {
    /// Record a notification sent
    pub fn record_notification_sent(
        &self,
        guarantee: crate::models::communication::DeliveryGuarantee,
        priority: crate::models::message_broker::MessagePriority,
    ) {
        self.notifications_sent.fetch_add(1, Ordering::Relaxed);
        self.notifications_by_guarantee.record(guarantee);
        self.notifications_by_priority.record(priority);
    }

    /// Get current metrics snapshot
    pub fn snapshot(&self) -> FireAndForgetMetricsSnapshot {
        FireAndForgetMetricsSnapshot {
            notifications_sent: self.notifications_sent.load(Ordering::Relaxed),
            notifications_by_guarantee: self.notifications_by_guarantee.snapshot(),
            notifications_by_priority: self.notifications_by_priority.snapshot(),
        }
    }
}

/// Snapshot of fire-and-forget metrics
#[derive(Debug, Clone, serde::Serialize)]
pub struct FireAndForgetMetricsSnapshot {
    pub notifications_sent: u64,
    pub notifications_by_guarantee: DeliveryGuaranteeCountsSnapshot,
    pub notifications_by_priority: PriorityMetricsSnapshot,
}

/// Delivery guarantee metrics
#[derive(Debug, Clone)]
pub struct DeliveryGuaranteeMetrics {
    /// Messages with BestEffort guarantee
    pub best_effort: Arc<AtomicU64>,

    /// Messages with AtLeastOnce guarantee
    pub at_least_once: Arc<AtomicU64>,

    /// Messages with AtMostOnce guarantee
    pub at_most_once: Arc<AtomicU64>,

    /// Messages with ExactlyOnce guarantee
    pub exactly_once: Arc<AtomicU64>,

    /// Delivery failures by guarantee type
    pub delivery_failures: DeliveryGuaranteeCounts,

    /// Successful deliveries by guarantee type
    pub successful_deliveries: DeliveryGuaranteeCounts,
}

impl Default for DeliveryGuaranteeMetrics {
    fn default() -> Self {
        Self {
            best_effort: Arc::new(AtomicU64::new(0)),
            at_least_once: Arc::new(AtomicU64::new(0)),
            at_most_once: Arc::new(AtomicU64::new(0)),
            exactly_once: Arc::new(AtomicU64::new(0)),
            delivery_failures: DeliveryGuaranteeCounts::default(),
            successful_deliveries: DeliveryGuaranteeCounts::default(),
        }
    }
}

impl DeliveryGuaranteeMetrics {
    /// Record a message with specific delivery guarantee
    pub fn record_message(&self, guarantee: crate::models::communication::DeliveryGuarantee) {
        match guarantee {
            crate::models::communication::DeliveryGuarantee::BestEffort => {
                self.best_effort.fetch_add(1, Ordering::Relaxed);
            }
            crate::models::communication::DeliveryGuarantee::AtLeastOnce => {
                self.at_least_once.fetch_add(1, Ordering::Relaxed);
            }
            crate::models::communication::DeliveryGuarantee::AtMostOnce => {
                self.at_most_once.fetch_add(1, Ordering::Relaxed);
            }
            crate::models::communication::DeliveryGuarantee::ExactlyOnce => {
                self.exactly_once.fetch_add(1, Ordering::Relaxed);
            }
        }
    }

    /// Record a delivery failure
    pub fn record_delivery_failure(
        &self,
        guarantee: crate::models::communication::DeliveryGuarantee,
    ) {
        self.delivery_failures.record(guarantee);
    }

    /// Record a successful delivery
    pub fn record_successful_delivery(
        &self,
        guarantee: crate::models::communication::DeliveryGuarantee,
    ) {
        self.successful_deliveries.record(guarantee);
    }

    /// Get current metrics snapshot
    pub fn snapshot(&self) -> DeliveryGuaranteeMetricsSnapshot {
        DeliveryGuaranteeMetricsSnapshot {
            best_effort: self.best_effort.load(Ordering::Relaxed),
            at_least_once: self.at_least_once.load(Ordering::Relaxed),
            at_most_once: self.at_most_once.load(Ordering::Relaxed),
            exactly_once: self.exactly_once.load(Ordering::Relaxed),
            delivery_failures: self.delivery_failures.snapshot(),
            successful_deliveries: self.successful_deliveries.snapshot(),
        }
    }
}

/// Snapshot of delivery guarantee metrics
#[derive(Debug, Clone, serde::Serialize)]
pub struct DeliveryGuaranteeMetricsSnapshot {
    pub best_effort: u64,
    pub at_least_once: u64,
    pub at_most_once: u64,
    pub exactly_once: u64,
    pub delivery_failures: DeliveryGuaranteeCountsSnapshot,
    pub successful_deliveries: DeliveryGuaranteeCountsSnapshot,
}

/// Error metrics
#[derive(Debug, Clone)]
pub struct ErrorMetrics {
    /// Total communication errors
    pub total_errors: Arc<AtomicU64>,

    /// Authentication errors
    pub auth_errors: Arc<AtomicU64>,

    /// Timeout errors
    pub timeout_errors: Arc<AtomicU64>,

    /// Network errors
    pub network_errors: Arc<AtomicU64>,

    /// Serialization errors
    pub serialization_errors: Arc<AtomicU64>,

    /// Subscription errors
    pub subscription_errors: Arc<AtomicU64>,

    /// Request errors
    pub request_errors: Arc<AtomicU64>,
}

impl Default for ErrorMetrics {
    fn default() -> Self {
        Self {
            total_errors: Arc::new(AtomicU64::new(0)),
            auth_errors: Arc::new(AtomicU64::new(0)),
            timeout_errors: Arc::new(AtomicU64::new(0)),
            network_errors: Arc::new(AtomicU64::new(0)),
            serialization_errors: Arc::new(AtomicU64::new(0)),
            subscription_errors: Arc::new(AtomicU64::new(0)),
            request_errors: Arc::new(AtomicU64::new(0)),
        }
    }
}

impl ErrorMetrics {
    /// Record an error
    pub fn record_error(&self, error_type: ErrorType) {
        self.total_errors.fetch_add(1, Ordering::Relaxed);

        match error_type {
            ErrorType::Auth => self.auth_errors.fetch_add(1, Ordering::Relaxed),
            ErrorType::Timeout => self.timeout_errors.fetch_add(1, Ordering::Relaxed),
            ErrorType::Network => self.network_errors.fetch_add(1, Ordering::Relaxed),
            ErrorType::Serialization => self.serialization_errors.fetch_add(1, Ordering::Relaxed),
            ErrorType::Subscription => self.subscription_errors.fetch_add(1, Ordering::Relaxed),
            ErrorType::Request => self.request_errors.fetch_add(1, Ordering::Relaxed),
        };
    }

    /// Get current metrics snapshot
    pub fn snapshot(&self) -> ErrorMetricsSnapshot {
        ErrorMetricsSnapshot {
            total_errors: self.total_errors.load(Ordering::Relaxed),
            auth_errors: self.auth_errors.load(Ordering::Relaxed),
            timeout_errors: self.timeout_errors.load(Ordering::Relaxed),
            network_errors: self.network_errors.load(Ordering::Relaxed),
            serialization_errors: self.serialization_errors.load(Ordering::Relaxed),
            subscription_errors: self.subscription_errors.load(Ordering::Relaxed),
            request_errors: self.request_errors.load(Ordering::Relaxed),
        }
    }
}

/// Error types for metrics tracking
#[derive(Debug, Clone, Copy)]
pub enum ErrorType {
    Auth,
    Timeout,
    Network,
    Serialization,
    Subscription,
    Request,
}

/// Snapshot of error metrics
#[derive(Debug, Clone, serde::Serialize)]
pub struct ErrorMetricsSnapshot {
    pub total_errors: u64,
    pub auth_errors: u64,
    pub timeout_errors: u64,
    pub network_errors: u64,
    pub serialization_errors: u64,
    pub subscription_errors: u64,
    pub request_errors: u64,
}

/// Priority-based metrics
#[derive(Debug, Clone)]
pub struct PriorityMetrics {
    pub critical: Arc<AtomicU64>,
    pub high: Arc<AtomicU64>,
    pub normal: Arc<AtomicU64>,
    pub low: Arc<AtomicU64>,
}

impl Default for PriorityMetrics {
    fn default() -> Self {
        Self {
            critical: Arc::new(AtomicU64::new(0)),
            high: Arc::new(AtomicU64::new(0)),
            normal: Arc::new(AtomicU64::new(0)),
            low: Arc::new(AtomicU64::new(0)),
        }
    }
}

impl PriorityMetrics {
    pub fn record(&self, priority: crate::models::message_broker::MessagePriority) {
        match priority {
            crate::models::message_broker::MessagePriority::Critical => {
                self.critical.fetch_add(1, Ordering::Relaxed);
            }
            crate::models::message_broker::MessagePriority::High => {
                self.high.fetch_add(1, Ordering::Relaxed);
            }
            crate::models::message_broker::MessagePriority::Normal => {
                self.normal.fetch_add(1, Ordering::Relaxed);
            }
            crate::models::message_broker::MessagePriority::Low => {
                self.low.fetch_add(1, Ordering::Relaxed);
            }
        }
    }

    pub fn snapshot(&self) -> PriorityMetricsSnapshot {
        PriorityMetricsSnapshot {
            critical: self.critical.load(Ordering::Relaxed),
            high: self.high.load(Ordering::Relaxed),
            normal: self.normal.load(Ordering::Relaxed),
            low: self.low.load(Ordering::Relaxed),
        }
    }
}

/// Snapshot of priority metrics
#[derive(Debug, Clone, serde::Serialize)]
pub struct PriorityMetricsSnapshot {
    pub critical: u64,
    pub high: u64,
    pub normal: u64,
    pub low: u64,
}

/// Topic-based metrics
#[derive(Debug, Clone)]
pub struct TopicMetrics {
    pub topics: Arc<std::sync::RwLock<std::collections::HashMap<String, u64>>>,
}

impl Default for TopicMetrics {
    fn default() -> Self {
        Self {
            topics: Arc::new(std::sync::RwLock::new(std::collections::HashMap::new())),
        }
    }
}

impl TopicMetrics {
    pub fn record_publish(&self, topic: &str) {
        let mut topics = self.topics.write().unwrap();
        *topics.entry(topic.to_string()).or_insert(0) += 1;
    }

    pub fn snapshot(&self) -> TopicMetricsSnapshot {
        let topics = self.topics.read().unwrap();
        TopicMetricsSnapshot {
            topics: topics.clone(),
        }
    }
}

/// Snapshot of topic metrics
#[derive(Debug, Clone, serde::Serialize)]
pub struct TopicMetricsSnapshot {
    pub topics: std::collections::HashMap<String, u64>,
}

/// Delivery guarantee counts
#[derive(Debug, Clone)]
pub struct DeliveryGuaranteeCounts {
    pub best_effort: Arc<AtomicU64>,
    pub at_least_once: Arc<AtomicU64>,
    pub at_most_once: Arc<AtomicU64>,
    pub exactly_once: Arc<AtomicU64>,
}

impl Default for DeliveryGuaranteeCounts {
    fn default() -> Self {
        Self {
            best_effort: Arc::new(AtomicU64::new(0)),
            at_least_once: Arc::new(AtomicU64::new(0)),
            at_most_once: Arc::new(AtomicU64::new(0)),
            exactly_once: Arc::new(AtomicU64::new(0)),
        }
    }
}

impl DeliveryGuaranteeCounts {
    pub fn record(&self, guarantee: crate::models::communication::DeliveryGuarantee) {
        match guarantee {
            crate::models::communication::DeliveryGuarantee::BestEffort => {
                self.best_effort.fetch_add(1, Ordering::Relaxed);
            }
            crate::models::communication::DeliveryGuarantee::AtLeastOnce => {
                self.at_least_once.fetch_add(1, Ordering::Relaxed);
            }
            crate::models::communication::DeliveryGuarantee::AtMostOnce => {
                self.at_most_once.fetch_add(1, Ordering::Relaxed);
            }
            crate::models::communication::DeliveryGuarantee::ExactlyOnce => {
                self.exactly_once.fetch_add(1, Ordering::Relaxed);
            }
        }
    }

    pub fn snapshot(&self) -> DeliveryGuaranteeCountsSnapshot {
        DeliveryGuaranteeCountsSnapshot {
            best_effort: self.best_effort.load(Ordering::Relaxed),
            at_least_once: self.at_least_once.load(Ordering::Relaxed),
            at_most_once: self.at_most_once.load(Ordering::Relaxed),
            exactly_once: self.exactly_once.load(Ordering::Relaxed),
        }
    }
}

/// Snapshot of delivery guarantee counts
#[derive(Debug, Clone, serde::Serialize)]
pub struct DeliveryGuaranteeCountsSnapshot {
    pub best_effort: u64,
    pub at_least_once: u64,
    pub at_most_once: u64,
    pub exactly_once: u64,
}

/// Complete metrics snapshot
#[derive(Debug, Clone, serde::Serialize)]
pub struct CommunicationMetricsSnapshot {
    pub request_response: RequestResponseMetricsSnapshot,
    pub publish_subscribe: PublishSubscribeMetricsSnapshot,
    pub fire_and_forget: FireAndForgetMetricsSnapshot,
    pub delivery_guarantees: DeliveryGuaranteeMetricsSnapshot,
    pub errors: ErrorMetricsSnapshot,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

impl CommunicationMetrics {
    /// Get a complete snapshot of all metrics
    pub fn snapshot(&self) -> CommunicationMetricsSnapshot {
        CommunicationMetricsSnapshot {
            request_response: self.request_response.snapshot(),
            publish_subscribe: self.publish_subscribe.snapshot(),
            fire_and_forget: self.fire_and_forget.snapshot(),
            delivery_guarantees: self.delivery_guarantees.snapshot(),
            errors: self.errors.snapshot(),
            timestamp: chrono::Utc::now(),
        }
    }

    /// Reset all metrics to zero
    pub fn reset(&mut self) {
        // Note: This is a simplified reset - in a real implementation,
        // we would need to reset all atomic counters
        // For now, we'll create new instances
        *self = Self::default();
    }
}
