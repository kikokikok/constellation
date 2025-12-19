//! Monitoring endpoints for agent metrics

use std::net::SocketAddr;
use std::sync::Arc;
use tokio::sync::Mutex;
use tracing::info;

use crate::error::AgentError;

#[cfg(feature = "monitoring")]
use serde_json::json;

#[cfg(feature = "monitoring")]
use tracing::error;

#[cfg(feature = "monitoring")]
use axum::{Router, http::StatusCode, response::Json, routing::get};

use crate::client::AgentClient;
use crate::error::AgentResult;

/// Monitoring server configuration
#[derive(Debug, Clone)]
#[cfg_attr(feature = "monitoring", derive(serde::Serialize, serde::Deserialize))]
pub struct MonitoringConfig {
    /// Server bind address
    pub bind_address: SocketAddr,
    /// Enable Prometheus metrics endpoint
    pub enable_prometheus: bool,
    /// Enable health check endpoint
    pub enable_health_check: bool,
    /// Enable metrics endpoint
    pub enable_metrics: bool,
    /// Enable connection pool stats endpoint
    pub enable_pool_stats: bool,
}

impl Default for MonitoringConfig {
    fn default() -> Self {
        Self {
            bind_address: "127.0.0.1:8080".parse().unwrap(),
            enable_prometheus: true,
            enable_health_check: true,
            enable_metrics: true,
            enable_pool_stats: true,
        }
    }
}

/// Monitoring server
pub struct MonitoringServer {
    /// Agent client
    client: Arc<Mutex<AgentClient>>,
    /// Server configuration
    config: MonitoringConfig,
    /// Server handle
    server_handle: Option<tokio::task::JoinHandle<()>>,
}

impl MonitoringServer {
    /// Create a new monitoring server
    pub fn new(client: AgentClient, config: Option<MonitoringConfig>) -> Self {
        Self {
            client: Arc::new(Mutex::new(client)),
            config: config.unwrap_or_default(),
            server_handle: None,
        }
    }

    /// Start the monitoring server
    #[cfg(feature = "monitoring")]
    pub async fn start(&mut self) -> crate::AgentResult<()> {
        info!("Starting monitoring server on {}", self.config.bind_address);

        let app = self.create_app().await?;

        let listener = tokio::net::TcpListener::bind(&self.config.bind_address)
            .await
            .map_err(|e| {
                AgentError::Configuration(format!(
                    "Failed to bind to {}: {}",
                    self.config.bind_address, e
                ))
            })?;

        let handle = tokio::spawn(async move {
            if let Err(e) = axum::serve(listener, app).await {
                error!("Monitoring server error: {}", e);
            }
        });

        self.server_handle = Some(handle);
        info!("Monitoring server started successfully");

        Ok(())
    }

    /// Start the monitoring server (no-op without monitoring feature)
    #[cfg(not(feature = "monitoring"))]
    pub async fn start(&mut self) -> crate::AgentResult<()> {
        info!("Monitoring feature not enabled - server not started");
        Ok(())
    }

    /// Create the Axum application
    #[cfg(feature = "monitoring")]
    async fn create_app(&self) -> crate::AgentResult<axum::Router> {
        let mut router = Router::new();

        // Health check endpoint
        if self.config.enable_health_check {
            router = router.route("/health", get(health_check));
        }

        // Metrics endpoint
        if self.config.enable_metrics {
            let client = self.client.clone();
            router = router.route("/metrics", get(move || get_metrics(client.clone())));
        }

        // Pattern-specific metrics
        if self.config.enable_metrics {
            let client = self.client.clone();
            router = router.route(
                "/metrics/request-response",
                get(move || get_request_response_metrics(client.clone())),
            );

            let client = self.client.clone();
            router = router.route(
                "/metrics/publish-subscribe",
                get(move || get_publish_subscribe_metrics(client.clone())),
            );

            let client = self.client.clone();
            router = router.route(
                "/metrics/fire-and-forget",
                get(move || get_fire_and_forget_metrics(client.clone())),
            );
        }

        // Connection pool stats
        if self.config.enable_pool_stats {
            let client = self.client.clone();
            router = router.route("/pool-stats", get(move || get_pool_stats(client.clone())));
        }

        // Prometheus metrics endpoint
        if self.config.enable_prometheus {
            let client = self.client.clone();
            router = router.route(
                "/prometheus",
                get(move || get_prometheus_metrics(client.clone())),
            );
        }

        Ok(router)
    }

    /// Create the Axum application (no-op without monitoring feature)
    #[cfg(not(feature = "monitoring"))]
    async fn create_app(&self) -> crate::AgentResult<()> {
        Err(crate::error::AgentError::Configuration(
            "Monitoring feature not enabled".to_string(),
        ))
    }

    /// Stop the monitoring server
    pub async fn stop(&mut self) -> AgentResult<()> {
        info!("Stopping monitoring server");

        if let Some(handle) = self.server_handle.take() {
            handle.abort();
            let _ = handle.await;
        }

        info!("Monitoring server stopped");
        Ok(())
    }
}

/// Health check endpoint
#[cfg(feature = "monitoring")]
async fn health_check() -> (StatusCode, Json<serde_json::Value>) {
    (
        StatusCode::OK,
        Json(json!({
            "status": "healthy",
            "timestamp": chrono::Utc::now().to_rfc3339(),
        })),
    )
}

/// Get all metrics endpoint
#[cfg(feature = "monitoring")]
async fn get_metrics(client: Arc<Mutex<AgentClient>>) -> (StatusCode, Json<serde_json::Value>) {
    let client = client.lock().await;

    match client.get_metrics().await {
        Ok(metrics) => {
            let json_value = serde_json::to_value(&metrics).unwrap_or_else(|_| json!({}));
            (
                StatusCode::OK,
                Json(json!({
                    "status": "success",
                    "data": json_value,
                    "timestamp": chrono::Utc::now().to_rfc3339(),
                })),
            )
        }
        Err(e) => {
            error!("Failed to get metrics: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({
                    "status": "error",
                    "error": e.to_string(),
                    "timestamp": chrono::Utc::now().to_rfc3339(),
                })),
            )
        }
    }
}

/// Get request-response metrics endpoint
#[cfg(feature = "monitoring")]
async fn get_request_response_metrics(
    client: Arc<Mutex<AgentClient>>,
) -> (StatusCode, Json<serde_json::Value>) {
    let client = client.lock().await;

    match client.get_pattern_metrics("request-response").await {
        Ok(metrics) => {
            let json_value = serde_json::to_value(&metrics).unwrap_or_else(|_| json!({}));
            (
                StatusCode::OK,
                Json(json!({
                    "status": "success",
                    "data": json_value,
                    "timestamp": chrono::Utc::now().to_rfc3339(),
                })),
            )
        }
        Err(e) => {
            error!("Failed to get request-response metrics: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({
                    "status": "error",
                    "error": e.to_string(),
                    "timestamp": chrono::Utc::now().to_rfc3339(),
                })),
            )
        }
    }
}

/// Get publish-subscribe metrics endpoint
#[cfg(feature = "monitoring")]
async fn get_publish_subscribe_metrics(
    client: Arc<Mutex<AgentClient>>,
) -> (StatusCode, Json<serde_json::Value>) {
    let client = client.lock().await;

    match client.get_pattern_metrics("publish-subscribe").await {
        Ok(metrics) => {
            let json_value = serde_json::to_value(&metrics).unwrap_or_else(|_| json!({}));
            (
                StatusCode::OK,
                Json(json!({
                    "status": "success",
                    "data": json_value,
                    "timestamp": chrono::Utc::now().to_rfc3339(),
                })),
            )
        }
        Err(e) => {
            error!("Failed to get publish-subscribe metrics: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({
                    "status": "error",
                    "error": e.to_string(),
                    "timestamp": chrono::Utc::now().to_rfc3339(),
                })),
            )
        }
    }
}

/// Get fire-and-forget metrics endpoint
#[cfg(feature = "monitoring")]
async fn get_fire_and_forget_metrics(
    client: Arc<Mutex<AgentClient>>,
) -> (StatusCode, Json<serde_json::Value>) {
    let client = client.lock().await;

    match client.get_pattern_metrics("fire-and-forget").await {
        Ok(metrics) => {
            let json_value = serde_json::to_value(&metrics).unwrap_or_else(|_| json!({}));
            (
                StatusCode::OK,
                Json(json!({
                    "status": "success",
                    "data": json_value,
                    "timestamp": chrono::Utc::now().to_rfc3339(),
                })),
            )
        }
        Err(e) => {
            error!("Failed to get fire-and-forget metrics: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({
                    "status": "error",
                    "error": e.to_string(),
                    "timestamp": chrono::Utc::now().to_rfc3339(),
                })),
            )
        }
    }
}

/// Get connection pool statistics endpoint
#[cfg(feature = "monitoring")]
async fn get_pool_stats(client: Arc<Mutex<AgentClient>>) -> (StatusCode, Json<serde_json::Value>) {
    let client = client.lock().await;

    let stats = client.get_connection_pool_stats().await;
    let json_value = serde_json::to_value(&stats).unwrap_or_else(|_| json!({}));
    (
        StatusCode::OK,
        Json(json!({
            "status": "success",
            "data": json_value,
            "timestamp": chrono::Utc::now().to_rfc3339(),
        })),
    )
}

/// Get Prometheus metrics endpoint
#[cfg(feature = "monitoring")]
async fn get_prometheus_metrics(client: Arc<Mutex<AgentClient>>) -> (StatusCode, String) {
    let client = client.lock().await;

    match client.get_metrics().await {
        Ok(metrics) => {
            let prometheus_output = format_prometheus_metrics(&metrics);
            (StatusCode::OK, prometheus_output)
        }
        Err(e) => {
            error!("Failed to get metrics for Prometheus: {}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("# ERROR: {}\n", e),
            )
        }
    }
}

/// Format metrics as Prometheus text format
fn format_prometheus_metrics(
    metrics: &constellation_core::communication::CommunicationMetricsSnapshot,
) -> String {
    let mut output = String::new();

    // Request-response metrics
    output.push_str("# HELP constellation_request_response_requests_sent Total requests sent\n");
    output.push_str("# TYPE constellation_request_response_requests_sent counter\n");
    output.push_str(&format!(
        "constellation_request_response_requests_sent {}\n",
        metrics.request_response.requests_sent
    ));

    output.push_str(
        "# HELP constellation_request_response_responses_received Total responses received\n",
    );
    output.push_str("# TYPE constellation_request_response_responses_received counter\n");
    output.push_str(&format!(
        "constellation_request_response_responses_received {}\n",
        metrics.request_response.responses_received
    ));

    output.push_str("# HELP constellation_request_response_timeouts Total request timeouts\n");
    output.push_str("# TYPE constellation_request_response_timeouts counter\n");
    output.push_str(&format!(
        "constellation_request_response_timeouts {}\n",
        metrics.request_response.request_timeouts
    ));

    output.push_str("# HELP constellation_request_response_avg_response_time_ms Average response time in milliseconds\n");
    output.push_str("# TYPE constellation_request_response_avg_response_time_ms gauge\n");
    output.push_str(&format!(
        "constellation_request_response_avg_response_time_ms {}\n",
        metrics.request_response.avg_response_time_ms
    ));

    // Publish-subscribe metrics
    output.push_str(
        "# HELP constellation_publish_subscribe_messages_published Total messages published\n",
    );
    output.push_str("# TYPE constellation_publish_subscribe_messages_published counter\n");
    output.push_str(&format!(
        "constellation_publish_subscribe_messages_published {}\n",
        metrics.publish_subscribe.messages_published
    ));

    output.push_str("# HELP constellation_publish_subscribe_messages_delivered Total messages delivered to subscribers\n");
    output.push_str("# TYPE constellation_publish_subscribe_messages_delivered counter\n");
    output.push_str(&format!(
        "constellation_publish_subscribe_messages_delivered {}\n",
        metrics.publish_subscribe.messages_delivered
    ));

    output.push_str(
        "# HELP constellation_publish_subscribe_active_subscriptions Active subscriptions\n",
    );
    output.push_str("# TYPE constellation_publish_subscribe_active_subscriptions gauge\n");
    output.push_str(&format!(
        "constellation_publish_subscribe_active_subscriptions {}\n",
        metrics.publish_subscribe.active_subscriptions
    ));

    // Fire-and-forget metrics
    output.push_str(
        "# HELP constellation_fire_and_forget_notifications_sent Total notifications sent\n",
    );
    output.push_str("# TYPE constellation_fire_and_forget_notifications_sent counter\n");
    output.push_str(&format!(
        "constellation_fire_and_forget_notifications_sent {}\n",
        metrics.fire_and_forget.notifications_sent
    ));

    // Delivery guarantee metrics
    output.push_str(
        "# HELP constellation_delivery_guarantees_best_effort Messages with BestEffort guarantee\n",
    );
    output.push_str("# TYPE constellation_delivery_guarantees_best_effort counter\n");
    output.push_str(&format!(
        "constellation_delivery_guarantees_best_effort {}\n",
        metrics.delivery_guarantees.best_effort
    ));

    output.push_str("# HELP constellation_delivery_guarantees_at_least_once Messages with AtLeastOnce guarantee\n");
    output.push_str("# TYPE constellation_delivery_guarantees_at_least_once counter\n");
    output.push_str(&format!(
        "constellation_delivery_guarantees_at_least_once {}\n",
        metrics.delivery_guarantees.at_least_once
    ));

    output.push_str("# HELP constellation_delivery_guarantees_at_most_once Messages with AtMostOnce guarantee\n");
    output.push_str("# TYPE constellation_delivery_guarantees_at_most_once counter\n");
    output.push_str(&format!(
        "constellation_delivery_guarantees_at_most_once {}\n",
        metrics.delivery_guarantees.at_most_once
    ));

    output.push_str("# HELP constellation_delivery_guarantees_exactly_once Messages with ExactlyOnce guarantee\n");
    output.push_str("# TYPE constellation_delivery_guarantees_exactly_once counter\n");
    output.push_str(&format!(
        "constellation_delivery_guarantees_exactly_once {}\n",
        metrics.delivery_guarantees.exactly_once
    ));

    // Error metrics
    output.push_str("# HELP constellation_errors_total Total errors\n");
    output.push_str("# TYPE constellation_errors_total counter\n");
    output.push_str(&format!(
        "constellation_errors_total {}\n",
        metrics.errors.total_errors
    ));

    output.push_str("# HELP constellation_errors_request Request errors\n");
    output.push_str("# TYPE constellation_errors_request counter\n");
    output.push_str(&format!(
        "constellation_errors_request {}\n",
        metrics.errors.request_errors
    ));

    output.push_str("# HELP constellation_errors_timeout Timeout errors\n");
    output.push_str("# TYPE constellation_errors_timeout counter\n");
    output.push_str(&format!(
        "constellation_errors_timeout {}\n",
        metrics.errors.timeout_errors
    ));

    output.push_str("# HELP constellation_errors_network Network errors\n");
    output.push_str("# TYPE constellation_errors_network counter\n");
    output.push_str(&format!(
        "constellation_errors_network {}\n",
        metrics.errors.network_errors
    ));

    output.push_str("# HELP constellation_errors_subscription Subscription errors\n");
    output.push_str("# TYPE constellation_errors_subscription counter\n");
    output.push_str(&format!(
        "constellation_errors_subscription {}\n",
        metrics.errors.subscription_errors
    ));

    output
}
