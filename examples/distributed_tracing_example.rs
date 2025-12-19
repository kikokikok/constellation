//! Distributed Tracing Example
//!
//! This example demonstrates tracing capabilities in Constellation
//! with structured logging and span instrumentation.
//!
//! Features demonstrated:
//! 1. Tracing setup with structured logging
//! 2. Span instrumentation for A2A message processing
//! 3. Request/response tracing
//! 4. Custom tracing macros for different components

use constellation_core::models::communication::{RequestConfig, RequestMessage, ResponseStatus};
use constellation_core::models::message_broker::Message;
use constellation_core::tracing::{
    TracingConfig, generate_trace_id, init_tracing, log_with_trace_context, shutdown_tracing,
};
use std::time::Duration;
use tokio::time::sleep;
use tracing::{debug, error, info, warn};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🚀 Starting Distributed Tracing Example");
    println!("=======================================");

    // Configure tracing
    let tracing_config = TracingConfig {
        service_name: "constellation-example".to_string(),
        enable_console: true,
        log_level: "debug".to_string(),
    };

    println!("\n1. Initializing Tracing");
    println!("------------------------");
    println!("Service: {}", tracing_config.service_name);
    println!("Log Level: {}", tracing_config.log_level);

    // Initialize tracing
    init_tracing(tracing_config).await?;

    info!("Distributed tracing initialized successfully");

    println!("\n2. Demonstrating Trace Context");
    println!("-------------------------------");

    // Log with trace context
    log_with_trace_context("This log includes trace context");

    // Generate and show a trace ID
    let trace_id = generate_trace_id();
    println!("Generated Trace ID: {}", trace_id);

    println!("\n3. Instrumented A2A Message Processing");
    println!("--------------------------------------");

    // Create a sample A2A message
    let message = Message {
        message_id: "msg_123".to_string(),
        sender_id: "agent_a".to_string(),
        recipient_id: "agent_b".to_string(),
        payload: r#"{"action": "process", "data": "test"}"#.to_string(),
        message_type: "command".to_string(),
        content_type: "application/json".to_string(),
        protocol_version: "1.1".to_string(),
        priority: constellation_core::models::message_broker::MessagePriority::Normal,
        ttl_seconds: Some(300),
        correlation_id: Some("corr_123".to_string()),
        conversation_id: Some("conv_123".to_string()),
        metadata: None,
        created_at: chrono::Utc::now(),
        expires_at: None,
        delivery_status: constellation_core::models::message_broker::DeliveryStatus::Pending,
        retry_count: 0,
        last_retry_at: None,
        dead_letter_reason: None,
    };

    // Process the message with tracing instrumentation
    constellation_core::tracing::process_a2a_message(&message).await?;

    println!("\n4. Request/Response Pattern Tracing");
    println!("-----------------------------------");

    // Create a sample request
    let request = RequestMessage::new(
        "agent_a".to_string(),
        "agent_b".to_string(),
        r#"{"query": "get_data", "params": {"id": 123}}"#.to_string(),
        RequestConfig {
            timeout: Duration::from_secs(30),
            max_retries: 3,
            use_exponential_backoff: true,
            base_retry_delay: Duration::from_secs(1),
            max_retry_delay: Duration::from_secs(10),
            require_acknowledgment: true,
            delivery_guarantee:
                constellation_core::models::communication::DeliveryGuarantee::AtLeastOnce,
        },
        constellation_core::models::message_broker::MessagePriority::Normal,
    );

    // Handle request/response with tracing
    let response = constellation_core::tracing::handle_request_response(&request).await?;

    info!("Response created: {:?}", response.status);

    println!("\n5. Tracing Macros Demonstration");
    println!("-------------------------------");

    // Use the a2a_span macro
    {
        let span = constellation_core::a2a_span!(
            "custom_a2a_operation",
            "operation" => "transform",
            "input_size" => 1024,
            "output_size" => 2048
        );
        let _guard = span.enter();

        info!("Processing inside custom A2A span");
        sleep(Duration::from_millis(20)).await;
        info!("Custom A2A operation complete");
    }

    // Use the broker_span macro
    {
        let span = constellation_core::broker_span!("send_message", "agent_c", "msg_456");
        let _guard = span.enter();

        info!("Sending message via broker");
        sleep(Duration::from_millis(15)).await;
        info!("Message sent successfully");
    }

    // Use the auth_span macro
    {
        let span = constellation_core::auth_span!("validate_token", "agent_d");
        let _guard = span.enter();

        info!("Validating JWT token");
        sleep(Duration::from_millis(25)).await;
        info!("Token validation complete");
    }

    println!("\n6. Error Tracing");
    println!("----------------");

    // Demonstrate error tracing
    {
        let span = tracing::span!(tracing::Level::ERROR, "error_operation");
        let _guard = span.enter();

        error!("This is an error with trace context");
        warn!("This is a warning with trace context");
        debug!("This is a debug message with trace context");
    }

    println!("\n7. Trace Context Propagation");
    println!("---------------------------");
    println!("Tracing enables:");
    println!("- Structured logging with context");
    println!("- Performance analysis and bottleneck identification");
    println!("- Debugging complex distributed systems");
    println!("- Monitoring performance metrics");

    println!("\n8. Extending to Full Distributed Tracing");
    println!("----------------------------------------");
    println!("To add full distributed tracing with OpenTelemetry:");
    println!("1. Add OpenTelemetry dependencies to Cargo.toml");
    println!("2. Configure Jaeger or other exporters");
    println!("3. Add context propagation across service boundaries");
    println!("4. Set up trace sampling and filtering");

    // Shutdown tracing gracefully
    println!("\n9. Shutting Down Tracing");
    println!("------------------------");
    shutdown_tracing().await;

    println!("\n✅ Distributed Tracing Example Complete!");
    println!("========================================");

    Ok(())
}
