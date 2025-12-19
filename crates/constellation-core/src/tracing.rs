//! Distributed tracing foundation for Constellation.
//!
//! This module provides the foundation for distributed tracing capabilities
//! with structured logging and trace context.
//!
//! Features:
//! - Structured logging with trace context
//! - Span instrumentation for A2A operations
//! - Trace context propagation foundation
//! - Configurable logging levels
//!
//! Note: Full OpenTelemetry integration can be added when needed.

use std::time::Duration;
use tracing::{Level, debug, error, info, span, warn};

/// Configuration for tracing
#[derive(Debug, Clone)]
pub struct TracingConfig {
    /// Service name for traces
    pub service_name: String,
    /// Enable console logging
    pub enable_console: bool,
    /// Log level filter
    pub log_level: String,
}

impl Default for TracingConfig {
    fn default() -> Self {
        Self {
            service_name: "constellation".to_string(),
            enable_console: true,
            log_level: "info".to_string(),
        }
    }
}

/// Initialize tracing
///
/// # Arguments
/// * `config` - Tracing configuration
///
/// # Returns
/// * `Result<(), Box<dyn std::error::Error>>` - Success or error
///
/// # Example
/// ```rust
/// use constellation_core::tracing::{init_tracing, TracingConfig};
///
/// #[tokio::main]
/// async fn main() -> Result<(), Box<dyn std::error::Error>> {
///     let config = TracingConfig {
///         service_name: "my-service".to_string(),
///         enable_console: true,
///         log_level: "debug".to_string(),
///     };
///
///     init_tracing(config).await?;
///     Ok(())
/// }
/// ```
pub async fn init_tracing(config: TracingConfig) -> Result<(), Box<dyn std::error::Error>> {
    use tracing_subscriber::fmt;
    use tracing_subscriber::prelude::*;

    info!("Initializing tracing for {}", config.service_name);

    // Create console layer if enabled
    if config.enable_console {
        let fmt_layer = fmt::layer()
            .with_target(true)
            .with_thread_ids(true)
            .with_thread_names(true)
            .compact();

        let filter_layer =
            tracing_subscriber::filter::LevelFilter::from_level(match config.log_level.as_str() {
                "trace" => tracing::Level::TRACE,
                "debug" => tracing::Level::DEBUG,
                "info" => tracing::Level::INFO,
                "warn" => tracing::Level::WARN,
                "error" => tracing::Level::ERROR,
                _ => tracing::Level::INFO,
            });

        tracing_subscriber::registry()
            .with(fmt_layer)
            .with(filter_layer)
            .init();
    }

    info!("Tracing initialized successfully");
    debug!("Service: {}", config.service_name);
    debug!("Log level: {}", config.log_level);

    Ok(())
}

/// Shutdown tracing gracefully
pub async fn shutdown_tracing() {
    info!("Shutting down tracing");
    info!("Tracing shutdown complete");
}

/// Create a span for A2A message processing
#[macro_export]
macro_rules! a2a_span {
    ($name:expr, $($key:expr => $value:expr),*) => {{
        use tracing::span;

        span!(
            tracing::Level::INFO,
            $name,
            $(
                $key = $value,
            )*
            component = "a2a",
            protocol.version = "1.1",
        )
    }};
}

/// Create a span for message broker operations
#[macro_export]
macro_rules! broker_span {
    ($name:expr, $agent_id:expr, $message_id:expr) => {{
        use tracing::span;

        span!(
            tracing::Level::INFO,
            $name,
            component = "message_broker",
            agent.id = $agent_id,
            message.id = $message_id,
            broker.type = "iggy",
        )
    }};
}

/// Create a span for authentication operations
#[macro_export]
macro_rules! auth_span {
    ($name:expr, $agent_id:expr) => {{
        use tracing::span;

        span!(
            tracing::Level::INFO,
            $name,
            component = "authentication",
            agent.id = $agent_id,
            auth.type = "jwt",
            crypto.type = "ed25519",
        )
    }};
}

/// Example of instrumented function with tracing
#[tracing::instrument(
    name = "process_a2a_message",
    skip(message),
    fields(
        message_id = %message.message_id,
        sender_id = %message.sender_id,
        recipient_id = %message.recipient_id,
        message_type = %message.message_type,
        protocol_version = %message.protocol_version,
    )
)]
pub async fn process_a2a_message(
    message: &crate::models::message_broker::Message,
) -> Result<(), Box<dyn std::error::Error>> {
    // This is an example function showing how to instrument with tracing
    info!("Processing A2A message");

    // Add custom fields to the span
    tracing::Span::current().record("content_type", &message.content_type);

    if let Some(ttl) = message.ttl_seconds {
        tracing::Span::current().record("ttl_seconds", ttl);
    }

    // Simulate processing
    tokio::time::sleep(Duration::from_millis(10)).await;

    info!("A2A message processed successfully");
    Ok(())
}

/// Example of creating a trace for request/response pattern
#[tracing::instrument(
    name = "handle_request_response",
    skip(request),
    fields(
        request_id = %request.request_id,
        correlation_id = %request.correlation_id,
        sender_id = %request.sender_id,
        recipient = %request.recipient,
    )
)]
pub async fn handle_request_response(
    request: &crate::models::communication::RequestMessage,
) -> Result<crate::models::communication::ResponseMessage, Box<dyn std::error::Error>> {
    info!("Handling request/response pattern");

    // Record request details
    tracing::Span::current().record("payload_size", request.payload.len());
    tracing::Span::current().record("priority", format!("{:?}", request.priority));

    // Simulate processing
    tokio::time::sleep(Duration::from_millis(50)).await;

    // Create response
    let response = crate::models::communication::ResponseMessage {
        response_id: uuid::Uuid::new_v4().to_string(),
        correlation_id: request.correlation_id.clone(),
        sender_id: request.recipient.clone(),
        recipient_id: request.sender_id.clone(),
        payload: r#"{"status": "success", "processed": true}"#.to_string(),
        status: crate::models::communication::ResponseStatus::Success,
        config: crate::models::communication::ResponseConfig::default(),
        created_at: chrono::Utc::now(),
        original_request: Some(Box::new(request.clone())),
        priority: request.priority,
        ttl_seconds: Some(300),
    };

    info!("Request/response handled successfully");
    Ok(response)
}

/// Get a unique trace ID for logging context
pub fn generate_trace_id() -> String {
    uuid::Uuid::new_v4().to_string()
}

/// Get a unique span ID for logging context
pub fn generate_span_id() -> String {
    uuid::Uuid::new_v4().to_string()[..8].to_string()
}

/// Add trace context to log messages
pub fn log_with_trace_context(message: &str) {
    let trace_id = generate_trace_id();
    let span_id = generate_span_id();
    info!(trace_id, span_id, message);
}
