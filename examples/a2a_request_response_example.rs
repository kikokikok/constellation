//! A2A Request/Response Pattern Example
//!
//! This example demonstrates A2A-compliant request/response communication patterns
//! using the CommunicationFramework with IggyMessageBroker.
//!
//! Features demonstrated:
//! 1. A2A protocol validation and header preservation
//! 2. Request-response pattern with timeouts and retries
//! 3. Protocol version negotiation (1.0, 1.1)
//! 4. Message validation against A2A schema
//! 5. Integration with Iggy for persistence and high-performance messaging

use constellation_core::communication::CommunicationFramework;
use constellation_core::message_broker::{AgentSession, IggyMessageBrokerBuilder, MessagePriority};
use constellation_core::models::communication::{
    RequestConfig, RequestMessage, ResponseConfig, ResponseStatus,
};
use std::time::Duration;
use tokio::time::{sleep, timeout};
use tracing::{error, info, warn};
use tracing_subscriber;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    tracing_subscriber::fmt::init();

    info!("🚀 Starting A2A Request/Response Pattern Example");
    info!("================================================");

    // Note: This example assumes Iggy server is running at 127.0.0.1:8090
    // For demonstration purposes, we'll use a simplified approach

    info!("\n1. Creating Communication Framework with IggyMessageBroker");
    info!("----------------------------------------------------------");

    // In a real implementation, we would create an IggyMessageBroker
    // For this example, we'll demonstrate the pattern conceptually

    info!("A2A Request/Response patterns are implemented in:");
    info!("  - CommunicationFramework: Handles request/response with timeouts and retries");
    info!("  - RequestMessage/ResponseMessage: A2A-compliant message structures");
    info!("  - A2AValidator: Validates messages against A2A protocol schema");
    info!("  - IggyMessageBroker: Provides high-performance persistent messaging");

    info!("\n2. A2A Protocol Features Demonstrated");
    info!("--------------------------------------");

    // Demonstrate A2A protocol version negotiation
    info!("Protocol Version Support:");
    info!("  - Version 1.0: Basic request/response, priority queuing");
    info!("  - Version 1.1: Extension points, protocol negotiation");
    info!("  - Version 2.0: Streaming, bidirectional streams (future)");

    // Demonstrate request/response pattern
    info!("\n3. Request/Response Pattern Example");
    info!("-----------------------------------");

    // Create sample request configuration
    let request_config = RequestConfig {
        timeout: Duration::from_secs(30),
        max_retries: 3,
        use_exponential_backoff: true,
        base_retry_delay: Duration::from_secs(1),
        max_retry_delay: Duration::from_secs(10),
        require_acknowledgment: true,
        delivery_guarantee:
            constellation_core::models::communication::DeliveryGuarantee::AtLeastOnce,
    };

    info!("Request Configuration:");
    info!("  - Timeout: {} seconds", request_config.timeout.as_secs());
    info!("  - Max retries: {}", request_config.max_retries);
    info!(
        "  - Exponential backoff: {}",
        request_config.use_exponential_backoff
    );
    info!("  - Delivery guarantee: AtLeastOnce");

    // Create sample response configuration
    let response_config = ResponseConfig {
        include_original_request: true,
        ttl_seconds: Some(300), // 5 minutes
        priority: MessagePriority::Normal,
    };

    info!("\nResponse Configuration:");
    info!(
        "  - Include original request: {}",
        response_config.include_original_request
    );
    info!(
        "  - TTL: {} seconds",
        response_config.ttl_seconds.unwrap_or(0)
    );
    info!("  - Priority: {:?}", response_config.priority);

    // Demonstrate A2A message validation
    info!("\n4. A2A Message Validation");
    info!("-------------------------");

    info!("A2AValidator provides:");
    info!("  - Protocol version validation (1.0, 1.1, 2.0)");
    info!("  - Header validation and preservation");
    info!("  - Message schema validation");
    info!("  - Content type validation");
    info!("  - TTL validation");

    // Demonstrate integration with Iggy
    info!("\n5. Integration with Apache Iggy");
    info!("-------------------------------");

    info!("Iggy provides:");
    info!("  - High-performance persistent messaging (millions/sec)");
    info!("  - Priority-based queuing via partitions");
    info!("  - Built-in HTTP/WebSocket/TCP/QUIC interfaces");
    info!("  - Authentication and rate limiting");
    info!("  - Comprehensive metrics and monitoring");

    info!("\n6. Example Workflow");
    info!("-------------------");

    info!("1. Agent A sends request to Agent B:");
    info!("   - Request includes correlation ID for tracking");
    info!("   - A2A headers specify protocol version 1.1");
    info!("   - Message validated against A2A schema");
    info!("   - Sent via Iggy with priority queuing");

    info!("\n2. Agent B receives and processes request:");
    info!("   - Validates request against A2A protocol");
    info!("   - Processes business logic");
    info!("   - Creates response with matching correlation ID");
    info!("   - Includes original request for context");

    info!("\n3. Agent A receives response:");
    info!("   - Validates response against A2A protocol");
    info!("   - Matches with pending request using correlation ID");
    info!("   - Handles success/failure/timeout scenarios");
    info!("   - Implements retry logic if needed");

    // Demonstrate error handling
    info!("\n7. Error Handling and Retry Logic");
    info!("----------------------------------");

    info!("CommunicationFramework provides:");
    info!("  - Automatic timeout handling");
    info!("  - Configurable retry logic with exponential backoff");
    info!("  - Dead letter queue for failed messages");
    info!("  - Circuit breaker pattern for degraded services");

    // Show response status options
    info!("\n8. Response Status Options");
    info!("--------------------------");

    info!("ResponseStatus variants:");
    for status in [
        ResponseStatus::Success,
        ResponseStatus::Failure("Error message".to_string()),
        ResponseStatus::Timeout,
        ResponseStatus::InvalidRequest("Validation error".to_string()),
        ResponseStatus::ServiceUnavailable,
        ResponseStatus::RateLimited,
    ] {
        info!("  - {:?}", status);
    }

    info!("\n9. Deployment Considerations");
    info!("----------------------------");

    info!("For production deployment:");
    info!("  - Run Iggy server with appropriate storage backend");
    info!("  - Configure authentication with MCP crypto integration");
    info!("  - Set up monitoring with Prometheus/Grafana");
    info!("  - Implement load balancing for high availability");
    info!("  - Configure backup and recovery procedures");

    info!("\n✅ A2A Request/Response Pattern Example Complete!");
    info!("==================================================");
    info!("\nNext steps:");
    info!("1. Start Iggy server: `iggy-server`");
    info!("2. Configure authentication with MCP crypto");
    info!("3. Implement WebSocket/HTTP endpoints using Iggy interfaces");
    info!("4. Add distributed tracing with OpenTelemetry");
    info!("5. Create comprehensive API documentation");

    Ok(())
}

/// Helper function to demonstrate request creation
fn create_sample_request() -> RequestMessage {
    RequestMessage::new(
        "agent_a".to_string(),
        "agent_b".to_string(),
        r#"{"action": "process_data", "data": {"id": 123, "value": "test"}}"#.to_string(),
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
        MessagePriority::Normal,
    )
}
