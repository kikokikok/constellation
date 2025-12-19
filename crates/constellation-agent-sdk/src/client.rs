//! Agent client for A2A communication

use std::sync::Arc;
use std::time::Duration;
use tokio::sync::mpsc;
use tokio::task::JoinHandle;
use tracing::{debug, error, info, warn};

use crate::config::AgentConfig;
use crate::connection_pool::{ConnectionPool, ConnectionPoolConfig, PooledConnectionHandle};
use crate::error::{AgentError, AgentResult};
use crate::handler::{MessageHandler, RequestHandler};

use constellation_core::communication::CommunicationFramework;
use constellation_core::message_broker::IggyMessageBroker;
use constellation_core::models::communication::{
    DeliveryGuarantee, NotificationMessage, PublishMessage, RequestConfig, RequestMessage,
    ResponseMessage, TopicPattern,
};
use constellation_core::models::message_broker::{Message, MessagePriority};

/// Agent client for A2A communication
pub struct AgentClient {
    /// Agent configuration
    config: AgentConfig,

    /// Connection pool
    connection_pool: ConnectionPool,

    /// Message handler
    message_handler: Option<Arc<dyn MessageHandler>>,

    /// Request handler
    request_handler: Option<Arc<dyn RequestHandler>>,

    /// Background task handles
    background_tasks: Vec<JoinHandle<()>>,

    /// Shutdown signal sender
    shutdown_tx: Option<mpsc::Sender<()>>,
}

impl AgentClient {
    /// Connect to the message broker and create a new agent client
    pub async fn connect(config: AgentConfig) -> AgentResult<Self> {
        info!("Connecting agent '{}' to broker...", config.agent_id);

        // Create connection pool
        let mut connection_pool = ConnectionPool::new(config.clone(), None).await?;

        // Start connection pool maintenance
        connection_pool.start_maintenance().await;

        info!("Agent '{}' connected successfully", config.agent_id);

        Ok(Self {
            config,
            connection_pool,
            message_handler: None,
            request_handler: None,
            background_tasks: Vec::new(),
            shutdown_tx: None,
        })
    }

    /// Connect with custom connection pool configuration
    pub async fn connect_with_pool_config(
        config: AgentConfig,
        pool_config: ConnectionPoolConfig,
    ) -> AgentResult<Self> {
        info!(
            "Connecting agent '{}' to broker with custom pool config...",
            config.agent_id
        );

        // Create connection pool with custom configuration
        let mut connection_pool = ConnectionPool::new(config.clone(), Some(pool_config)).await?;

        // Start connection pool maintenance
        connection_pool.start_maintenance().await;

        info!(
            "Agent '{}' connected successfully with custom pool",
            config.agent_id
        );

        Ok(Self {
            config,
            connection_pool,
            message_handler: None,
            request_handler: None,
            background_tasks: Vec::new(),
            shutdown_tx: None,
        })
    }

    /// Set the message handler
    pub fn with_message_handler<H: MessageHandler + 'static>(mut self, handler: H) -> Self {
        self.message_handler = Some(Arc::new(handler));
        self
    }

    /// Set the request handler
    pub fn with_request_handler<H: RequestHandler + 'static>(mut self, handler: H) -> Self {
        self.request_handler = Some(Arc::new(handler));
        self
    }

    /// Start background message processing
    pub async fn start(&mut self) -> AgentResult<()> {
        info!(
            "Starting background message processing for agent '{}'",
            self.config.agent_id
        );

        let (shutdown_tx, mut shutdown_rx) = mpsc::channel(1);
        self.shutdown_tx = Some(shutdown_tx);

        let connection_pool = self.connection_pool.clone();
        let agent_id = self.config.agent_id.clone();
        let message_handler = self.message_handler.clone();
        let request_handler = self.request_handler.clone();

        // Start message processing task
        let task = tokio::spawn(async move {
            info!("Message processing started for agent '{}'", agent_id);

            loop {
                tokio::select! {
                    _ = shutdown_rx.recv() => {
                        info!("Shutdown signal received for agent '{}'", agent_id);
                        break;
                    }
                    _ = tokio::time::sleep(Duration::from_millis(100)) => {
                        // Get a connection from the pool
                        match connection_pool.get_connection().await {
                            Ok(connection_handle) => {
                                // Process messages using the pooled connection
                                if let Err(e) = Self::process_messages_with_connection(
                                    &connection_handle,
                                    &agent_id,
                                    &message_handler,
                                    &request_handler,
                                ).await {
                                    error!("Failed to process messages: {}", e);
                                }
                                // Connection handle will be automatically returned to pool when dropped
                            }
                            Err(e) => {
                                error!("Failed to get connection from pool: {}", e);
                                tokio::time::sleep(Duration::from_secs(1)).await;
                            }
                        }
                    }
                }
            }

            info!("Message processing stopped for agent '{}'", agent_id);
        });

        self.background_tasks.push(task);
        Ok(())
    }

    /// Process messages using a pooled connection
    async fn process_messages_with_connection(
        connection_handle: &PooledConnectionHandle,
        agent_id: &str,
        message_handler: &Option<Arc<dyn MessageHandler>>,
        request_handler: &Option<Arc<dyn RequestHandler>>,
    ) -> AgentResult<()> {
        // Receive messages using the pooled connection
        match connection_handle
            .broker
            .receive_messages(agent_id, 10)
            .await
        {
            Ok(messages) => {
                for message in messages {
                    // Process the message
                    if let Err(e) = Self::process_message(
                        &connection_handle.framework,
                        message_handler,
                        request_handler,
                        message,
                    )
                    .await
                    {
                        error!("Failed to process message: {}", e);
                    }
                }
                Ok(())
            }
            Err(e) => Err(AgentError::MessageProcessing(format!(
                "Failed to receive messages: {}",
                e
            ))),
        }
    }

    /// Process a single message
    async fn process_message(
        framework: &CommunicationFramework<IggyMessageBroker>,
        message_handler: &Option<Arc<dyn MessageHandler>>,
        request_handler: &Option<Arc<dyn RequestHandler>>,
        message: Message,
    ) -> AgentResult<()> {
        debug!("Processing message: {}", message.message_id);

        match message.message_type.as_str() {
            "request" => {
                // Parse as request message
                let request: RequestMessage =
                    serde_json::from_str(&message.payload).map_err(|e| {
                        AgentError::MessageProcessing(format!("Failed to parse request: {}", e))
                    })?;

                // Handle the request if we have a handler
                if let Some(handler) = request_handler {
                    let _response = handler.handle_request(request).await;

                    // Send the response
                    // Note: The response is sent automatically by handle_response
                    // framework.handle_response(response).await
                    //     .map_err(|e| AgentError::MessageProcessing(format!("Failed to send response: {}", e)))?;
                } else {
                    warn!("Received request but no request handler is configured");
                }
            }
            "response" => {
                // Parse as response message
                let response: ResponseMessage =
                    serde_json::from_str(&message.payload).map_err(|e| {
                        AgentError::MessageProcessing(format!("Failed to parse response: {}", e))
                    })?;

                // Handle the response
                framework.handle_response(response).await.map_err(|e| {
                    AgentError::MessageProcessing(format!("Failed to handle response: {}", e))
                })?;
            }
            "publish" | "notification" => {
                // Handle regular messages
                if let Some(handler) = message_handler
                    && let Some(_response) = handler.handle_message(message).await
                {
                    // Send response if handler produced one
                    // Note: In a real implementation, we would need to get the broker from framework
                    // For now, we'll just log that we would send a response
                    debug!(
                        "Message handler produced response, but sending is not implemented in this version"
                    );
                }
            }
            _ => {
                warn!("Unknown message type: {}", message.message_type);
            }
        }

        Ok(())
    }

    /// Send a request and wait for response
    pub async fn request(
        &self,
        recipient: impl Into<String>,
        payload: impl Into<String>,
    ) -> AgentResult<ResponseMessage> {
        // Get a connection from the pool
        let connection_handle = self.connection_pool.get_connection().await?;

        let request_config = RequestConfig {
            timeout: self.config.default_request_timeout,
            max_retries: self.config.default_max_retries,
            use_exponential_backoff: true,
            retry_base_delay: self.config.default_retry_base_delay,
            require_response: true,
        };

        let request = RequestMessage::new(
            self.config.agent_id.clone(),
            recipient.into(),
            payload.into(),
            request_config,
            MessagePriority::Normal,
        );

        connection_handle
            .framework
            .send_request(request)
            .await
            .map_err(|e| AgentError::Request(format!("Failed to send request: {}", e)))
    }

    /// Send a notification (fire-and-forget)
    pub async fn notify(
        &self,
        recipient: impl Into<String>,
        payload: impl Into<String>,
        guarantee: DeliveryGuarantee,
    ) -> AgentResult<()> {
        // Get a connection from the pool
        let connection_handle = self.connection_pool.get_connection().await?;

        let notification = NotificationMessage::new(
            self.config.agent_id.clone(),
            recipient.into(),
            payload.into(),
            guarantee,
            MessagePriority::Normal,
        );

        connection_handle
            .framework
            .send_notification(notification)
            .await
            .map_err(|e| AgentError::Communication(format!("Failed to send notification: {}", e)))
    }

    /// Publish a message to a topic
    pub async fn publish(
        &self,
        topic: impl Into<String>,
        payload: impl Into<String>,
        guarantee: DeliveryGuarantee,
    ) -> AgentResult<()> {
        // Get a connection from the pool
        let connection_handle = self.connection_pool.get_connection().await?;

        let publish_message = PublishMessage::new(
            self.config.agent_id.clone(),
            topic.into(),
            payload.into(),
            guarantee,
            MessagePriority::Normal,
            Some(300), // 5 minute TTL
        );

        connection_handle
            .framework
            .publish(publish_message)
            .await
            .map_err(|e| AgentError::Communication(format!("Failed to publish: {}", e)))
    }

    /// Subscribe to a topic pattern
    pub async fn subscribe(&self, pattern: impl Into<String>) -> AgentResult<()> {
        // Get a connection from the pool
        let connection_handle = self.connection_pool.get_connection().await?;

        let pattern_str = pattern.into();
        let topic_pattern = if pattern_str.contains('*') {
            TopicPattern::Wildcard(pattern_str)
        } else {
            TopicPattern::Exact(pattern_str)
        };

        connection_handle
            .framework
            .subscribe(self.config.agent_id.clone(), topic_pattern)
            .await
            .map_err(|e| AgentError::Subscription(format!("Failed to subscribe: {}", e)))?;

        Ok(())
    }

    /// Unsubscribe from a subscription
    pub async fn unsubscribe(&self, subscription_id: &str) -> AgentResult<()> {
        // Get a connection from the pool
        let connection_handle = self.connection_pool.get_connection().await?;

        connection_handle
            .framework
            .unsubscribe(subscription_id, &self.config.agent_id)
            .await
            .map_err(|e| AgentError::Subscription(format!("Failed to unsubscribe: {}", e)))?;

        Ok(())
    }

    /// Get active subscriptions
    pub async fn get_subscriptions(
        &self,
    ) -> Vec<constellation_core::models::communication::Subscription> {
        // Get a connection from the pool
        match self.connection_pool.get_connection().await {
            Ok(connection_handle) => {
                connection_handle
                    .framework
                    .get_subscriptions(&self.config.agent_id)
                    .await
            }
            Err(_) => Vec::new(), // Return empty vector if we can't get a connection
        }
    }

    /// Send a message directly
    pub async fn send_message(
        &self,
        recipient: impl Into<String>,
        message_type: impl Into<String>,
        payload: impl Into<String>,
        priority: MessagePriority,
    ) -> AgentResult<()> {
        // Get a connection from the pool
        let connection_handle = self.connection_pool.get_connection().await?;

        let message = Message::new(
            uuid::Uuid::new_v4().to_string(),
            self.config.agent_id.clone(),
            recipient.into(),
            message_type.into(),
            payload.into(),
        )
        .with_priority(priority);

        connection_handle
            .broker
            .send_message(message)
            .await
            .map_err(|e| AgentError::Communication(format!("Failed to send message: {}", e)))
    }

    /// Receive messages (blocking)
    pub async fn receive(&self, limit: usize) -> AgentResult<Vec<Message>> {
        // Get a connection from the pool
        let connection_handle = self.connection_pool.get_connection().await?;

        connection_handle
            .broker
            .receive_messages(&self.config.agent_id, limit)
            .await
            .map_err(|e| AgentError::Communication(format!("Failed to receive messages: {}", e)))
    }

    /// Broadcast a message to all connected agents
    pub async fn broadcast(
        &self,
        message_type: impl Into<String>,
        payload: impl Into<String>,
        priority: MessagePriority,
    ) -> AgentResult<()> {
        // Get a connection from the pool
        let connection_handle = self.connection_pool.get_connection().await?;

        let message = Message::new(
            uuid::Uuid::new_v4().to_string(),
            self.config.agent_id.clone(),
            "broadcast".to_string(),
            message_type.into(),
            payload.into(),
        )
        .with_priority(priority);

        connection_handle
            .broker
            .broadcast(message)
            .await
            .map_err(|e| AgentError::Communication(format!("Failed to broadcast: {}", e)))
    }

    /// Get connected agents
    pub async fn get_connected_agents(&self) -> Vec<String> {
        // Get a connection from the pool
        match self.connection_pool.get_connection().await {
            Ok(connection_handle) => connection_handle.broker.get_connected_agents().await,
            Err(_) => Vec::new(), // Return empty vector if we can't get a connection
        }
    }

    /// Shutdown the agent client
    pub async fn shutdown(&mut self) -> AgentResult<()> {
        info!("Shutting down agent '{}'", self.config.agent_id);

        // Send shutdown signal
        if let Some(tx) = self.shutdown_tx.take() {
            let _ = tx.send(()).await;
        }

        // Wait for background tasks
        let tasks = std::mem::take(&mut self.background_tasks);
        for task in tasks {
            let _ = task.await;
        }

        info!("Agent '{}' shutdown complete", self.config.agent_id);
        Ok(())
    }

    /// Get connection pool statistics
    pub async fn get_connection_pool_stats(&self) -> crate::connection_pool::PoolStats {
        self.connection_pool.get_stats().await
    }

    /// Get a connection handle for advanced usage
    pub async fn get_connection_handle(&self) -> AgentResult<PooledConnectionHandle> {
        self.connection_pool.get_connection().await
    }

    /// Get communication metrics snapshot
    pub async fn get_metrics(
        &self,
    ) -> AgentResult<constellation_core::communication::CommunicationMetricsSnapshot> {
        // Get a connection from the pool
        let connection_handle = self.connection_pool.get_connection().await?;

        Ok(connection_handle.framework.metrics_snapshot())
    }

    /// Get communication metrics with detailed breakdown
    pub async fn get_detailed_metrics(
        &self,
    ) -> AgentResult<Arc<constellation_core::communication::CommunicationMetrics>> {
        // Get a connection from the pool
        let connection_handle = self.connection_pool.get_connection().await?;

        Ok(connection_handle.framework.metrics())
    }

    /// Reset all communication metrics
    pub async fn reset_metrics(&self) -> AgentResult<()> {
        // Get a connection from the pool
        let _connection_handle = self.connection_pool.get_connection().await?;

        // We need a mutable reference to the framework
        // This is tricky since we have an Arc - for now, we'll skip this method
        // or implement it differently
        Err(AgentError::Metrics(
            "Reset metrics requires mutable access to framework".to_string(),
        ))
    }

    /// Get metrics for a specific pattern
    pub async fn get_pattern_metrics(&self, pattern: &str) -> AgentResult<crate::PatternMetrics> {
        // Get a connection from the pool
        let connection_handle = self.connection_pool.get_connection().await?;

        let snapshot = connection_handle.framework.metrics_snapshot();

        match pattern {
            "request-response" => Ok(crate::PatternMetrics::RequestResponse(
                snapshot.request_response,
            )),
            "publish-subscribe" => Ok(crate::PatternMetrics::PublishSubscribe(
                snapshot.publish_subscribe,
            )),
            "fire-and-forget" => Ok(crate::PatternMetrics::FireAndForget(
                snapshot.fire_and_forget,
            )),
            _ => Err(AgentError::Metrics(format!("Unknown pattern: {}", pattern))),
        }
    }
}

impl Drop for AgentClient {
    fn drop(&mut self) {
        if self.shutdown_tx.is_some() {
            error!(
                "AgentClient dropped without calling shutdown() - this may cause resource leaks"
            );
        }
    }
}
