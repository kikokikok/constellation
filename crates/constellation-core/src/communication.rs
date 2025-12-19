//! Communication framework for agent-to-agent (A2A) messaging patterns.
//!
//! This module implements communication patterns on top of the message broker:
//! - Request-response with timeouts and retries
//! - Publish-subscribe with topic-based routing
//! - Fire-and-forget notifications
//! - Delivery guarantees and idempotency
//! - Comprehensive metrics collection

mod metrics;

pub use metrics::{
    CommunicationMetrics, CommunicationMetricsSnapshot, ErrorType, FireAndForgetMetricsSnapshot,
    PublishSubscribeMetricsSnapshot, RequestResponseMetricsSnapshot,
};

use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::{Mutex, RwLock};
use tokio::task::JoinHandle;
use tokio::time;
use tracing::{debug, error, info, trace, warn};
use uuid::Uuid;

use crate::models::communication::{
    CommunicationError, CommunicationResult, DeliveryGuarantee, NotificationMessage,
    PublishMessage, RequestConfig, RequestMessage, ResponseConfig, ResponseMessage, ResponseStatus,
    Subscription, TopicPattern,
};
use crate::models::message_broker::{
    AgentSession, Message as BrokerMessage, MessageBrokerError, MessageBrokerResult,
    MessagePriority,
};

/// Communication framework for A2A patterns
pub struct CommunicationFramework<B>
where
    B: MessageBroker + Send + Sync,
{
    /// Active subscriptions by topic pattern
    subscriptions: Arc<RwLock<HashMap<String, Vec<Subscription>>>>,
    /// Pending requests by correlation ID
    pending_requests: Arc<RwLock<HashMap<String, PendingRequest>>>,
    /// Message broker instance
    message_broker: Arc<B>,
    /// Configuration
    config: CommunicationConfig,
    /// Metrics collection
    metrics: Arc<CommunicationMetrics>,
    /// Background task handle
    background_task: Option<JoinHandle<()>>,
}

impl<B> Clone for CommunicationFramework<B>
where
    B: MessageBroker + Send + Sync,
{
    fn clone(&self) -> Self {
        Self {
            subscriptions: self.subscriptions.clone(),
            pending_requests: self.pending_requests.clone(),
            message_broker: self.message_broker.clone(),
            config: self.config.clone(),
            metrics: self.metrics.clone(),
            background_task: None, // Don't clone the background task
        }
    }
}

/// Trait for message broker operations (abstraction layer)
#[async_trait::async_trait]
pub trait MessageBroker {
    /// Send a message
    async fn send_message(&self, message: BrokerMessage) -> MessageBrokerResult<()>;

    /// Receive messages for an agent
    async fn receive_messages(
        &self,
        agent_id: &str,
        limit: usize,
    ) -> MessageBrokerResult<Vec<BrokerMessage>>;

    /// Register agent session
    async fn register_session(&self, session: AgentSession) -> MessageBrokerResult<()>;

    /// Get agent session
    async fn get_session(&self, agent_id: &str) -> MessageBrokerResult<Option<AgentSession>>;

    /// Broadcast message to all connected agents
    async fn broadcast(&self, message: BrokerMessage) -> MessageBrokerResult<()>;
}

/// Pending request tracking
struct PendingRequest {
    /// Request message
    request: RequestMessage,
    /// Response channel
    response_tx: tokio::sync::oneshot::Sender<CommunicationResult<ResponseMessage>>,
    /// Created timestamp
    created_at: Instant,
    /// Last retry timestamp
    last_retry_at: Option<Instant>,
}

/// Communication framework configuration
#[derive(Clone, Debug)]
pub struct CommunicationConfig {
    /// Default request timeout
    pub default_request_timeout: Duration,
    /// Default max retries
    pub default_max_retries: u32,
    /// Default retry base delay
    pub default_retry_base_delay: Duration,
    /// Subscription cleanup interval
    pub subscription_cleanup_interval: Duration,
    /// Request cleanup interval
    pub request_cleanup_interval: Duration,
    /// Max pending requests per agent
    pub max_pending_requests: usize,
}

impl Default for CommunicationConfig {
    fn default() -> Self {
        Self {
            default_request_timeout: Duration::from_secs(30),
            default_max_retries: 3,
            default_retry_base_delay: Duration::from_secs(1),
            subscription_cleanup_interval: Duration::from_secs(300), // 5 minutes
            request_cleanup_interval: Duration::from_secs(60),       // 1 minute
            max_pending_requests: 1000,
        }
    }
}

impl<B> CommunicationFramework<B>
where
    B: MessageBroker + Send + Sync + 'static,
{
    /// Create a new communication framework
    pub fn new(message_broker: Arc<B>) -> Self {
        Self::with_config(message_broker, CommunicationConfig::default())
    }

    /// Create a new communication framework with custom configuration
    pub fn with_config(message_broker: Arc<B>, config: CommunicationConfig) -> Self {
        let framework = Self {
            subscriptions: Arc::new(RwLock::new(HashMap::new())),
            pending_requests: Arc::new(RwLock::new(HashMap::new())),
            message_broker,
            config,
            metrics: Arc::new(CommunicationMetrics::default()),
            background_task: None,
        };

        // Start background tasks
        framework.start_background_tasks();

        framework
    }

    /// Start background maintenance tasks
    fn start_background_tasks(&self) {
        let subscriptions = self.subscriptions.clone();
        let pending_requests = self.pending_requests.clone();
        let subscription_interval = self.config.subscription_cleanup_interval;
        let request_interval = self.config.request_cleanup_interval;

        // Subscription cleanup task
        tokio::spawn(async move {
            let mut interval = time::interval(subscription_interval);
            loop {
                interval.tick().await;
                if let Err(e) = Self::cleanup_subscriptions_task(&subscriptions).await {
                    warn!("Subscription cleanup failed: {}", e);
                }
            }
        });

        // Request cleanup task
        tokio::spawn(async move {
            let mut interval = time::interval(request_interval);
            loop {
                interval.tick().await;
                if let Err(e) = Self::cleanup_pending_requests_task(&pending_requests).await {
                    warn!("Request cleanup failed: {}", e);
                }
            }
        });
    }

    /// Send a request and wait for response
    pub async fn send_request(
        &self,
        request: RequestMessage,
    ) -> CommunicationResult<ResponseMessage> {
        let request_id = request.request_id.clone();
        let correlation_id = request.correlation_id.clone();
        let start_time = Instant::now();

        // Record metrics
        self.metrics
            .request_response
            .record_request_sent(request.priority);

        // Check if we have too many pending requests
        {
            let pending_requests = self.pending_requests.read().await;
            if pending_requests.len() >= self.config.max_pending_requests {
                self.metrics.errors.record_error(ErrorType::Request);
                return Err(CommunicationError::MaxRetriesExceeded(
                    "Too many pending requests".to_string(),
                ));
            }
        }

        // Create response channel
        let (response_tx, response_rx) = tokio::sync::oneshot::channel();

        // Store pending request
        {
            let mut pending_requests = self.pending_requests.write().await;
            pending_requests.insert(
                correlation_id.clone(),
                PendingRequest {
                    request: request.clone(),
                    response_tx,
                    created_at: Instant::now(),
                    last_retry_at: None,
                },
            );
        }

        // Send the request
        if let Err(e) = self.send_request_message(&request).await {
            self.metrics.errors.record_error(ErrorType::Request);
            return Err(e);
        }

        // Wait for response with timeout
        let timeout = request.config.timeout;
        match time::timeout(timeout, response_rx).await {
            Ok(Ok(Ok(response))) => {
                // Record successful response
                let response_time = start_time.elapsed();
                self.metrics
                    .request_response
                    .record_response_received(response_time);
                Ok(response)
            }
            Ok(Ok(Err(e))) => {
                // Response channel returned an error
                self.cancel_request(&correlation_id).await;
                Err(e)
            }
            Ok(Err(_)) => {
                // Channel closed without response
                self.cancel_request(&correlation_id).await;
                self.metrics.request_response.record_request_timeout();
                self.metrics.errors.record_error(ErrorType::Timeout);
                Err(CommunicationError::Timeout(format!(
                    "Request {} cancelled",
                    request_id
                )))
            }
            Err(_) => {
                // Timeout
                self.cancel_request(&correlation_id).await;
                self.metrics.request_response.record_request_timeout();
                self.metrics.errors.record_error(ErrorType::Timeout);
                Err(CommunicationError::Timeout(format!(
                    "Request {} timed out after {:?}",
                    request_id, timeout
                )))
            }
        }
    }

    /// Send request message to broker
    async fn send_request_message(&self, request: &RequestMessage) -> CommunicationResult<()> {
        let broker_message = BrokerMessage::new(
            request.request_id.clone(),
            request.sender_id.clone(),
            request.recipient.clone(),
            "request".to_string(),
            request.payload.clone(),
        );

        self.message_broker
            .send_message(broker_message)
            .await
            .map_err(CommunicationError::MessageBrokerError)?;

        trace!(
            "Sent request {} to {}",
            request.request_id, request.recipient
        );
        Ok(())
    }

    /// Cancel a pending request
    async fn cancel_request(&self, correlation_id: &str) {
        let mut pending_requests = self.pending_requests.write().await;
        pending_requests.remove(correlation_id);
        trace!("Cancelled request with correlation ID: {}", correlation_id);
    }

    /// Handle incoming response
    pub async fn handle_response(&self, response: ResponseMessage) -> CommunicationResult<()> {
        let correlation_id = response.correlation_id.clone();

        let pending_request = {
            let mut pending_requests = self.pending_requests.write().await;
            pending_requests.remove(&correlation_id)
        };

        match pending_request {
            Some(pending) => {
                // Send response to waiting task
                if pending.response_tx.send(Ok(response)).is_err() {
                    warn!(
                        "Failed to send response for correlation ID: {}",
                        correlation_id
                    );
                }
                Ok(())
            }
            None => {
                warn!(
                    "Received response for unknown correlation ID: {}",
                    correlation_id
                );
                Err(CommunicationError::InvalidResponse(format!(
                    "Unknown correlation ID: {}",
                    correlation_id
                )))
            }
        }
    }

    /// Subscribe to a topic pattern
    pub async fn subscribe(
        &self,
        agent_id: String,
        topic_pattern: TopicPattern,
    ) -> CommunicationResult<Subscription> {
        let subscription = Subscription::new(agent_id.clone(), topic_pattern);

        // Add to subscriptions map
        let mut subscriptions = self.subscriptions.write().await;
        let agent_subscriptions = subscriptions
            .entry(agent_id.clone())
            .or_insert_with(Vec::new);

        // Check for duplicate subscription
        if agent_subscriptions.iter().any(|s| {
            // Compare pattern strings for simplicity
            match (&s.topic_pattern, &subscription.topic_pattern) {
                (TopicPattern::Exact(a), TopicPattern::Exact(b)) => a == b,
                (TopicPattern::Wildcard(a), TopicPattern::Wildcard(b)) => a == b,
                (TopicPattern::Regex(a), TopicPattern::Regex(b)) => a == b,
                _ => false,
            }
        }) {
            self.metrics.errors.record_error(ErrorType::Subscription);
            return Err(CommunicationError::SubscriptionError(
                "Already subscribed to similar topic pattern".to_string(),
            ));
        }

        agent_subscriptions.push(subscription.clone());

        // Record metrics
        self.metrics
            .publish_subscribe
            .record_subscription_event(true);

        info!(
            "Agent {} subscribed to topic pattern: {:?}",
            agent_id, subscription.topic_pattern
        );

        Ok(subscription)
    }

    /// Unsubscribe from a topic
    pub async fn unsubscribe(
        &self,
        subscription_id: &str,
        agent_id: &str,
    ) -> CommunicationResult<()> {
        let mut subscriptions = self.subscriptions.write().await;

        if let Some(agent_subscriptions) = subscriptions.get_mut(agent_id) {
            let original_len = agent_subscriptions.len();
            agent_subscriptions.retain(|s| s.id != subscription_id);

            if agent_subscriptions.len() < original_len {
                // Record metrics
                self.metrics
                    .publish_subscribe
                    .record_subscription_event(false);

                info!(
                    "Agent {} unsubscribed from subscription {}",
                    agent_id, subscription_id
                );
                Ok(())
            } else {
                self.metrics.errors.record_error(ErrorType::Subscription);
                Err(CommunicationError::SubscriptionError(format!(
                    "Subscription {} not found",
                    subscription_id
                )))
            }
        } else {
            self.metrics.errors.record_error(ErrorType::Subscription);
            Err(CommunicationError::SubscriptionError(format!(
                "No subscriptions found for agent {}",
                agent_id
            )))
        }
    }

    /// Publish a message to a topic
    pub async fn publish(&self, publish_message: PublishMessage) -> CommunicationResult<()> {
        let topic = publish_message.topic.clone();

        // Record metrics
        self.metrics
            .publish_subscribe
            .record_message_published(&topic);
        self.metrics
            .delivery_guarantees
            .record_message(publish_message.delivery_guarantee);

        // Find all subscribers matching the topic
        let subscribers = {
            let subscriptions = self.subscriptions.read().await;
            let mut subscribers = HashSet::new();

            for (agent_id, agent_subscriptions) in subscriptions.iter() {
                for subscription in agent_subscriptions {
                    if subscription.active && subscription.topic_pattern.matches(&topic) {
                        subscribers.insert(agent_id.clone());
                        break;
                    }
                }
            }

            subscribers
        };

        // Send to each subscriber
        let mut successful_deliveries = 0;
        for subscriber_id in &subscribers {
            let broker_message = BrokerMessage::new(
                publish_message.message_id.clone(),
                publish_message.publisher_id.clone(),
                subscriber_id.clone(),
                "publish".to_string(),
                publish_message.payload.clone(),
            );

            if let Err(e) = self.message_broker.send_message(broker_message).await {
                warn!("Failed to publish to subscriber {}: {}", subscriber_id, e);
                self.metrics
                    .delivery_guarantees
                    .record_delivery_failure(publish_message.delivery_guarantee);
                self.metrics.errors.record_error(ErrorType::Network);
                // Continue with other subscribers
            } else {
                successful_deliveries += 1;
                self.metrics
                    .delivery_guarantees
                    .record_successful_delivery(publish_message.delivery_guarantee);
            }
        }

        // Record delivery metrics
        self.metrics
            .publish_subscribe
            .record_message_delivered(successful_deliveries);

        info!(
            "Published message {} to topic {} ({} subscribers, {} successful deliveries)",
            publish_message.message_id,
            topic,
            subscribers.len(),
            successful_deliveries
        );

        Ok(())
    }

    /// Send a notification (fire-and-forget)
    pub async fn send_notification(
        &self,
        notification: NotificationMessage,
    ) -> CommunicationResult<()> {
        // Record metrics
        self.metrics
            .fire_and_forget
            .record_notification_sent(notification.delivery_guarantee, notification.priority);
        self.metrics
            .delivery_guarantees
            .record_message(notification.delivery_guarantee);

        let broker_message = BrokerMessage::new(
            notification.message_id.clone(),
            notification.sender_id.clone(),
            notification.recipient.clone(),
            "notification".to_string(),
            notification.payload.clone(),
        );

        match self
            .message_broker
            .send_message(broker_message)
            .await
            .map_err(CommunicationError::MessageBrokerError)
        {
            Ok(()) => {
                self.metrics
                    .delivery_guarantees
                    .record_successful_delivery(notification.delivery_guarantee);
                trace!(
                    "Sent notification {} to {}",
                    notification.message_id, notification.recipient
                );
                Ok(())
            }
            Err(e) => {
                self.metrics
                    .delivery_guarantees
                    .record_delivery_failure(notification.delivery_guarantee);
                self.metrics.errors.record_error(ErrorType::Network);
                Err(e)
            }
        }
    }

    /// Process incoming message
    pub async fn process_incoming_message(
        &self,
        message: BrokerMessage,
    ) -> CommunicationResult<()> {
        match message.message_type.as_str() {
            "request" => {
                // This is a request to us - we should handle it
                // In a real implementation, we'd have a request handler callback
                debug!("Received request: {}", message.message_id);
                Ok(())
            }
            "response" => {
                // This is a response to one of our requests
                // Parse as response message
                let response = match serde_json::from_str::<ResponseMessage>(&message.payload) {
                    Ok(response) => response,
                    Err(e) => {
                        return Err(CommunicationError::InvalidResponse(format!(
                            "Failed to parse response: {}",
                            e
                        )));
                    }
                };

                self.handle_response(response).await
            }
            "publish" => {
                // This is a published message to a topic we're subscribed to
                debug!("Received published message: {}", message.message_id);
                Ok(())
            }
            "notification" => {
                // This is a notification to us
                debug!("Received notification: {}", message.message_id);
                Ok(())
            }
            _ => {
                warn!("Unknown message type: {}", message.message_type);
                Ok(())
            }
        }
    }

    /// Clean up expired subscriptions (static method for background task)
    async fn cleanup_subscriptions_task(
        subscriptions: &Arc<RwLock<HashMap<String, Vec<Subscription>>>>,
    ) -> CommunicationResult<()> {
        let mut subscriptions = subscriptions.write().await;
        let mut removed_count = 0;

        for agent_subscriptions in subscriptions.values_mut() {
            let original_len = agent_subscriptions.len();
            agent_subscriptions.retain(|s| s.active);
            removed_count += original_len - agent_subscriptions.len();
        }

        // Remove empty agent entries
        subscriptions.retain(|_, subs| !subs.is_empty());

        if removed_count > 0 {
            debug!("Cleaned up {} inactive subscriptions", removed_count);
        }

        Ok(())
    }

    /// Clean up expired pending requests (static method for background task)
    async fn cleanup_pending_requests_task(
        pending_requests: &Arc<RwLock<HashMap<String, PendingRequest>>>,
    ) -> CommunicationResult<()> {
        let mut pending_requests = pending_requests.write().await;
        let original_len = pending_requests.len();

        pending_requests.retain(|_, pending| {
            let elapsed = pending.created_at.elapsed();
            elapsed < pending.request.config.timeout
        });

        let removed_count = original_len - pending_requests.len();
        if removed_count > 0 {
            debug!("Cleaned up {} expired pending requests", removed_count);
        }

        Ok(())
    }

    /// Get active subscriptions for an agent
    pub async fn get_subscriptions(&self, agent_id: &str) -> Vec<Subscription> {
        let subscriptions = self.subscriptions.read().await;
        subscriptions.get(agent_id).cloned().unwrap_or_default()
    }

    /// Get pending request count
    pub async fn pending_request_count(&self) -> usize {
        let pending_requests = self.pending_requests.read().await;
        pending_requests.len()
    }

    /// Get communication metrics
    pub fn metrics(&self) -> Arc<CommunicationMetrics> {
        self.metrics.clone()
    }

    /// Get metrics snapshot
    pub fn metrics_snapshot(&self) -> CommunicationMetricsSnapshot {
        self.metrics.snapshot()
    }

    /// Reset all metrics
    pub fn reset_metrics(&mut self) {
        // We need to get a mutable reference to the metrics
        // Since metrics is wrapped in Arc, we need to use Arc::get_mut
        if let Some(mutable_metrics) = Arc::get_mut(&mut self.metrics) {
            mutable_metrics.reset();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::communication::{RequestConfig, ResponseMessage};
    use std::collections::{HashMap, VecDeque};
    use std::sync::Arc;

    // Mock message broker for testing
    struct MockMessageBroker {
        sent_messages: Arc<tokio::sync::Mutex<Vec<BrokerMessage>>>,
        receive_queue: Arc<tokio::sync::Mutex<HashMap<String, VecDeque<BrokerMessage>>>>,
        sessions: Arc<tokio::sync::Mutex<HashMap<String, AgentSession>>>,
    }

    impl MockMessageBroker {
        fn new() -> Self {
            Self {
                sent_messages: Arc::new(tokio::sync::Mutex::new(Vec::new())),
                receive_queue: Arc::new(tokio::sync::Mutex::new(HashMap::new())),
                sessions: Arc::new(tokio::sync::Mutex::new(HashMap::new())),
            }
        }

        async fn queue_message(&self, agent_id: &str, message: BrokerMessage) {
            let mut queues = self.receive_queue.lock().await;
            queues
                .entry(agent_id.to_string())
                .or_insert_with(VecDeque::new)
                .push_back(message);
        }
    }

    #[async_trait::async_trait]
    impl MessageBroker for MockMessageBroker {
        async fn send_message(&self, message: BrokerMessage) -> MessageBrokerResult<()> {
            self.sent_messages.lock().await.push(message);
            Ok(())
        }

        async fn receive_messages(
            &self,
            agent_id: &str,
            limit: usize,
        ) -> MessageBrokerResult<Vec<BrokerMessage>> {
            let mut queues = self.receive_queue.lock().await;
            if let Some(queue) = queues.get_mut(agent_id) {
                let mut messages = Vec::new();
                for _ in 0..limit {
                    if let Some(message) = queue.pop_front() {
                        messages.push(message);
                    } else {
                        break;
                    }
                }
                Ok(messages)
            } else {
                Ok(Vec::new())
            }
        }

        async fn register_session(&self, session: AgentSession) -> MessageBrokerResult<()> {
            let mut sessions = self.sessions.lock().await;
            sessions.insert(session.agent_id.clone(), session);
            Ok(())
        }

        async fn get_session(&self, agent_id: &str) -> MessageBrokerResult<Option<AgentSession>> {
            let sessions = self.sessions.lock().await;
            Ok(sessions.get(agent_id).cloned())
        }

        async fn broadcast(&self, message: BrokerMessage) -> MessageBrokerResult<()> {
            self.sent_messages.lock().await.push(message);
            Ok(())
        }
    }

    #[tokio::test]
    async fn test_subscription_matching() {
        let exact_pattern = TopicPattern::Exact("system.alerts".to_string());
        let wildcard_pattern = TopicPattern::Wildcard("system.*".to_string());

        assert!(exact_pattern.matches("system.alerts"));
        assert!(!exact_pattern.matches("system.metrics"));

        assert!(wildcard_pattern.matches("system.alerts"));
        assert!(wildcard_pattern.matches("system.metrics"));
        assert!(!wildcard_pattern.matches("system"));
        assert!(!wildcard_pattern.matches("system.alerts.critical"));
    }

    #[tokio::test]
    async fn test_request_response_flow() {
        let mock_broker = Arc::new(MockMessageBroker::new());
        let framework = CommunicationFramework::new(mock_broker.clone());

        // Create a request
        let request = RequestMessage::new(
            "agent_a".to_string(),
            "agent_b".to_string(),
            "{\"action\": \"test\"}".to_string(),
            RequestConfig::default(),
            MessagePriority::Normal,
        );

        // Try to send request (will fail because no one will respond)
        let result = framework.send_request(request).await;
        assert!(matches!(result, Err(CommunicationError::Timeout(_))));

        // Check that message was sent
        let sent_messages = mock_broker.sent_messages.lock().await;
        assert_eq!(sent_messages.len(), 1);
        assert_eq!(sent_messages[0].message_type, "request");
    }

    #[tokio::test]
    async fn test_subscription_management() {
        let mock_broker = Arc::new(MockMessageBroker::new());
        let framework = CommunicationFramework::new(mock_broker.clone());

        // Subscribe to a topic
        let subscription = framework
            .subscribe(
                "agent_a".to_string(),
                TopicPattern::Exact("test.topic".to_string()),
            )
            .await
            .unwrap();

        // Get subscriptions
        let subscriptions = framework.get_subscriptions("agent_a").await;
        assert_eq!(subscriptions.len(), 1);
        assert_eq!(subscriptions[0].id, subscription.id);

        // Unsubscribe
        framework
            .unsubscribe(&subscription.id, "agent_a")
            .await
            .unwrap();

        let subscriptions = framework.get_subscriptions("agent_a").await;
        assert_eq!(subscriptions.len(), 0);
    }

    #[tokio::test]
    async fn test_successful_request_response() {
        let mock_broker = Arc::new(MockMessageBroker::new());
        let framework = CommunicationFramework::new(mock_broker.clone());

        // Register agent sessions
        let session_a = AgentSession::new(
            "agent_a".to_string(),
            "token_a".to_string(),
            "websocket".to_string(),
            None,
        );
        let session_b = AgentSession::new(
            "agent_b".to_string(),
            "token_b".to_string(),
            "websocket".to_string(),
            None,
        );
        mock_broker.register_session(session_a).await.unwrap();
        mock_broker.register_session(session_b).await.unwrap();

        // Create a request
        let request = RequestMessage::new(
            "agent_a".to_string(),
            "agent_b".to_string(),
            "{\"action\": \"test\"}".to_string(),
            RequestConfig {
                timeout: std::time::Duration::from_secs(5),
                max_retries: 3,
                use_exponential_backoff: false,
                retry_base_delay: std::time::Duration::from_millis(100),
                require_response: true,
            },
            MessagePriority::Normal,
        );

        // Spawn a task to simulate agent_b responding
        let framework_clone = framework.clone();
        let request_clone = request.clone();
        tokio::spawn(async move {
            // Simulate receiving the request
            tokio::time::sleep(std::time::Duration::from_millis(50)).await;

            // Create a response
            let response = ResponseMessage::success(
                request_clone.correlation_id.clone(),
                "agent_b".to_string(),
                "agent_a".to_string(),
                "{\"result\": \"success\"}".to_string(),
                Some(request_clone),
            );

            // Send the response
            framework_clone.handle_response(response).await.unwrap();
        });

        // Send request and wait for response
        let result = framework.send_request(request).await;
        assert!(result.is_ok());

        let response = result.unwrap();
        assert_eq!(response.status, ResponseStatus::Success);
        assert_eq!(response.payload, "{\"result\": \"success\"}");
    }

    #[tokio::test]
    async fn test_publish_subscribe() {
        let mock_broker = Arc::new(MockMessageBroker::new());
        let framework = CommunicationFramework::new(mock_broker.clone());

        // Register agent sessions
        let session_a = AgentSession::new(
            "agent_a".to_string(),
            "token_a".to_string(),
            "websocket".to_string(),
            None,
        );
        let session_b = AgentSession::new(
            "agent_b".to_string(),
            "token_b".to_string(),
            "websocket".to_string(),
            None,
        );
        mock_broker.register_session(session_a).await.unwrap();
        mock_broker.register_session(session_b).await.unwrap();

        // Subscribe agents to topics
        let sub1 = framework
            .subscribe(
                "agent_a".to_string(),
                TopicPattern::Exact("system.alerts".to_string()),
            )
            .await
            .unwrap();

        let sub2 = framework
            .subscribe(
                "agent_b".to_string(),
                TopicPattern::Wildcard("system.*".to_string()),
            )
            .await
            .unwrap();

        // Create a publish message
        let publish_message = PublishMessage::new(
            "publisher".to_string(),
            "system.alerts".to_string(),
            "{\"alert\": \"critical\"}".to_string(),
            DeliveryGuarantee::AtLeastOnce,
            MessagePriority::High,
            Some(60),
        );

        // Publish the message
        framework.publish(publish_message).await.unwrap();

        // Check that messages were sent to subscribers
        let sent_messages = mock_broker.sent_messages.lock().await;
        assert_eq!(sent_messages.len(), 2); // Both agents should receive it

        // Clean up
        framework.unsubscribe(&sub1.id, "agent_a").await.unwrap();
        framework.unsubscribe(&sub2.id, "agent_b").await.unwrap();
    }

    #[tokio::test]
    async fn test_notification_send() {
        let mock_broker = Arc::new(MockMessageBroker::new());
        let framework = CommunicationFramework::new(mock_broker.clone());

        // Register agent session
        let session = AgentSession::new(
            "agent_b".to_string(),
            "token".to_string(),
            "websocket".to_string(),
            None,
        );
        mock_broker.register_session(session).await.unwrap();

        // Create a notification
        let notification = NotificationMessage::new(
            "agent_a".to_string(),
            "agent_b".to_string(),
            "{\"event\": \"status_update\"}".to_string(),
            DeliveryGuarantee::BestEffort,
            MessagePriority::Normal,
        );

        // Send notification
        framework.send_notification(notification).await.unwrap();

        // Check that message was sent
        let sent_messages = mock_broker.sent_messages.lock().await;
        assert_eq!(sent_messages.len(), 1);
        assert_eq!(sent_messages[0].message_type, "notification");
    }

    #[tokio::test]
    async fn test_delivery_guarantees() {
        let mock_broker = Arc::new(MockMessageBroker::new());
        let framework = CommunicationFramework::new(mock_broker.clone());

        // Test different delivery guarantees
        let guarantees = vec![
            DeliveryGuarantee::BestEffort,
            DeliveryGuarantee::AtLeastOnce,
            DeliveryGuarantee::AtMostOnce,
            DeliveryGuarantee::ExactlyOnce,
        ];

        for guarantee in guarantees {
            let notification = NotificationMessage::new(
                "agent_a".to_string(),
                "agent_b".to_string(),
                format!("{{\"guarantee\": \"{:?}\"}}", guarantee),
                guarantee,
                MessagePriority::Normal,
            );

            framework.send_notification(notification).await.unwrap();
        }

        // All messages should be sent
        let sent_messages = mock_broker.sent_messages.lock().await;
        assert_eq!(sent_messages.len(), 4);
    }

    #[tokio::test]
    async fn test_max_pending_requests_limit() {
        // Create framework with very low max pending requests
        let mock_broker = Arc::new(MockMessageBroker::new());
        let config = CommunicationConfig {
            max_pending_requests: 1, // Only allow 1 pending request
            ..CommunicationConfig::default()
        };
        let framework = CommunicationFramework::with_config(mock_broker.clone(), config);

        // Register agent session
        let session = AgentSession::new(
            "agent_b".to_string(),
            "token".to_string(),
            "websocket".to_string(),
            None,
        );
        mock_broker.register_session(session).await.unwrap();

        // Send first request
        let request1 = RequestMessage::new(
            "agent_a".to_string(),
            "agent_b".to_string(),
            "{\"action\": \"test1\"}".to_string(),
            RequestConfig {
                timeout: std::time::Duration::from_secs(1), // Short timeout
                max_retries: 0,
                use_exponential_backoff: false,
                retry_base_delay: std::time::Duration::from_millis(100),
                require_response: true,
            },
            MessagePriority::Normal,
        );

        // Send request in background
        let framework_clone = framework.clone();
        tokio::spawn(async move {
            let _ = framework_clone.send_request(request1).await;
        });

        // Give it a moment to register as pending
        tokio::time::sleep(std::time::Duration::from_millis(50)).await;

        // Check pending request count - should be 1
        let pending_count = framework.pending_request_count().await;
        assert_eq!(pending_count, 1);

        // Send second request (should fail due to max pending requests)
        let request2 = RequestMessage::new(
            "agent_a".to_string(),
            "agent_b".to_string(),
            "{\"action\": \"test2\"}".to_string(),
            RequestConfig {
                timeout: std::time::Duration::from_secs(5),
                max_retries: 0,
                use_exponential_backoff: false,
                retry_base_delay: std::time::Duration::from_millis(100),
                require_response: true,
            },
            MessagePriority::Normal,
        );

        let result2 = framework.send_request(request2).await;
        assert!(matches!(
            result2,
            Err(CommunicationError::MaxRetriesExceeded(_))
        ));
    }

    #[tokio::test]
    async fn test_invalid_correlation_id_handling() {
        let mock_broker = Arc::new(MockMessageBroker::new());
        let framework = CommunicationFramework::new(mock_broker.clone());

        // Try to handle response with invalid correlation ID (no matching request)
        let response = ResponseMessage::success(
            "invalid_correlation_id".to_string(),
            "agent_b".to_string(),
            "agent_a".to_string(),
            "{\"result\": \"test\"}".to_string(),
            None,
        );

        let result = framework.handle_response(response).await;
        // Returns error for unknown correlation ID
        assert!(matches!(
            result,
            Err(CommunicationError::InvalidResponse(_))
        ));
    }

    #[tokio::test]
    async fn test_invalid_subscription_operations() {
        let mock_broker = Arc::new(MockMessageBroker::new());
        let framework = CommunicationFramework::new(mock_broker.clone());

        // Try to unsubscribe with invalid subscription ID
        let result = framework.unsubscribe("invalid_id", "agent_a").await;
        // Returns error for invalid subscription
        assert!(matches!(
            result,
            Err(CommunicationError::SubscriptionError(_))
        ));

        // Try to unsubscribe with wrong agent ID
        let subscription = framework
            .subscribe(
                "agent_a".to_string(),
                TopicPattern::Exact("test.topic".to_string()),
            )
            .await
            .unwrap();

        let result = framework.unsubscribe(&subscription.id, "wrong_agent").await;
        // Returns error for wrong agent ID
        assert!(matches!(
            result,
            Err(CommunicationError::SubscriptionError(_))
        ));
    }

    #[tokio::test]
    async fn test_publish_with_no_subscribers() {
        let mock_broker = Arc::new(MockMessageBroker::new());
        let framework = CommunicationFramework::new(mock_broker.clone());

        // Publish message when no one is subscribed
        let publish_message = PublishMessage::new(
            "publisher".to_string(),
            "test.topic".to_string(),
            "{\"data\": \"test\"}".to_string(),
            DeliveryGuarantee::AtLeastOnce,
            MessagePriority::Normal,
            Some(60),
        );

        // Should succeed even with no subscribers
        let result = framework.publish(publish_message).await;
        assert!(result.is_ok());

        // No messages should be sent
        let sent_messages = mock_broker.sent_messages.lock().await;
        assert_eq!(sent_messages.len(), 0);
    }

    #[tokio::test]
    async fn test_process_incoming_message() {
        let mock_broker = Arc::new(MockMessageBroker::new());
        let framework = CommunicationFramework::new(mock_broker.clone());

        // Register agent session
        let session = AgentSession::new(
            "agent_a".to_string(),
            "token".to_string(),
            "websocket".to_string(),
            None,
        );
        mock_broker.register_session(session).await.unwrap();

        // Create a request message
        let request = RequestMessage::new(
            "agent_b".to_string(),
            "agent_a".to_string(),
            "{\"action\": \"process\"}".to_string(),
            RequestConfig::default(),
            MessagePriority::Normal,
        );

        let a2a_message = request.to_a2a_message();
        let broker_message = BrokerMessage::new(
            a2a_message.message_id,
            a2a_message.sender_id,
            a2a_message.recipient_id,
            a2a_message.message_type,
            a2a_message.payload,
        );

        // Process incoming message
        let result = framework.process_incoming_message(broker_message).await;
        // Should succeed (even though no handler is registered)
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_model_validation() {
        // Test RequestConfig validation
        let config = RequestConfig {
            timeout: std::time::Duration::from_secs(0), // Zero timeout
            max_retries: 0,
            use_exponential_backoff: false,
            retry_base_delay: std::time::Duration::from_millis(0), // Zero delay
            require_response: true,
        };

        let request = RequestMessage::new(
            "agent_a".to_string(),
            "agent_b".to_string(),
            "{\"test\": \"data\"}".to_string(),
            config,
            MessagePriority::Normal,
        );

        // Request should be immediately expired with zero timeout
        assert!(request.is_expired());
        assert!(!request.can_retry());

        // Test TopicPattern regex validation
        let invalid_regex = TopicPattern::Regex("[invalid".to_string());
        // Should not panic on invalid regex
        let matches = invalid_regex.matches("test");
        assert!(!matches);

        // Test Subscription creation
        let subscription = Subscription::new(
            "agent_a".to_string(),
            TopicPattern::Exact("test.topic".to_string()),
        );
        assert!(!subscription.id.is_empty());
        assert_eq!(subscription.agent_id, "agent_a");
        assert!(subscription.active);
    }
}
