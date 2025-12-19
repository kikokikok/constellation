//! Tests for A2A protocol validation

use std::str::FromStr;

use crate::message_broker::a2a_validation::{
    A2AExtensionPoint, A2AFeature, A2AHeaders, A2AProtocolVersion, A2AValidator,
    ExtensionPointManager,
};
use crate::models::message_broker::{Message, MessagePriority};

#[test]
fn test_a2a_protocol_version_parsing() {
    // Test parsing from string
    assert_eq!(
        "1.0".parse::<A2AProtocolVersion>(),
        Ok(A2AProtocolVersion::V1_0)
    );
    assert_eq!(
        "1.1".parse::<A2AProtocolVersion>(),
        Ok(A2AProtocolVersion::V1_1)
    );
    assert_eq!(
        "2.0".parse::<A2AProtocolVersion>(),
        Ok(A2AProtocolVersion::V2_0)
    );
    assert!("3.0".parse::<A2AProtocolVersion>().is_err());
    assert!("invalid".parse::<A2AProtocolVersion>().is_err());

    // Test string representation
    assert_eq!(A2AProtocolVersion::V1_0.as_str(), "1.0");
    assert_eq!(A2AProtocolVersion::V1_1.as_str(), "1.1");
    assert_eq!(A2AProtocolVersion::V2_0.as_str(), "2.0");

    // Test default
    assert_eq!(A2AProtocolVersion::default(), A2AProtocolVersion::V1_0);
}

#[test]
fn test_a2a_protocol_version_compatibility() {
    // Same versions are compatible
    assert!(A2AProtocolVersion::V1_0.is_compatible_with(&A2AProtocolVersion::V1_0));
    assert!(A2AProtocolVersion::V1_1.is_compatible_with(&A2AProtocolVersion::V1_1));
    assert!(A2AProtocolVersion::V2_0.is_compatible_with(&A2AProtocolVersion::V2_0));

    // 1.1 can talk to 1.0 (backward compatible)
    assert!(A2AProtocolVersion::V1_1.is_compatible_with(&A2AProtocolVersion::V1_0));

    // 1.0 cannot talk to 1.1 (not forward compatible)
    assert!(!A2AProtocolVersion::V1_0.is_compatible_with(&A2AProtocolVersion::V1_1));

    // Different major versions not compatible
    assert!(!A2AProtocolVersion::V1_0.is_compatible_with(&A2AProtocolVersion::V2_0));
    assert!(!A2AProtocolVersion::V2_0.is_compatible_with(&A2AProtocolVersion::V1_0));
}

#[test]
fn test_a2a_headers_validation() {
    // Valid headers
    let valid_headers = A2AHeaders {
        protocol_version: "1.0".to_string(),
        message_type: "command".to_string(),
        content_type: "application/json".to_string(),
        priority: 5,
        ttl: Some(3600),
        correlation_id: Some("test-correlation".to_string()),
        conversation_id: Some("test-conversation".to_string()),
        custom: std::collections::HashMap::new(),
    };

    assert!(valid_headers.validate().is_ok());

    // Invalid protocol version
    let invalid_version = A2AHeaders {
        protocol_version: "3.0".to_string(), // Invalid version
        message_type: "command".to_string(),
        content_type: "application/json".to_string(),
        priority: 5,
        ttl: None,
        correlation_id: None,
        conversation_id: None,
        custom: std::collections::HashMap::new(),
    };

    assert!(invalid_version.validate().is_err());

    // Invalid message type
    let invalid_type = A2AHeaders {
        protocol_version: "1.0".to_string(),
        message_type: "invalid".to_string(), // Invalid type
        content_type: "application/json".to_string(),
        priority: 5,
        ttl: None,
        correlation_id: None,
        conversation_id: None,
        custom: std::collections::HashMap::new(),
    };

    assert!(invalid_type.validate().is_err());

    // Invalid priority (too high)
    let invalid_priority = A2AHeaders {
        protocol_version: "1.0".to_string(),
        message_type: "command".to_string(),
        content_type: "application/json".to_string(),
        priority: 15, // Invalid: > 10
        ttl: None,
        correlation_id: None,
        conversation_id: None,
        custom: std::collections::HashMap::new(),
    };

    assert!(invalid_priority.validate().is_err());
}

#[test]
fn test_a2a_validator_creation() {
    let validator = A2AValidator::new();
    let supported = validator.supported_versions();

    // Default validator should support 1.0 and 1.1
    assert!(supported.contains(&"1.0"));
    assert!(supported.contains(&"1.1"));
    assert!(!supported.contains(&"2.0"));
}

#[test]
fn test_a2a_validator_version_negotiation() {
    let validator = A2AValidator::new();

    // Client wants 1.1, server supports 1.0 and 1.1 -> negotiate to 1.1
    let client_versions = vec!["1.1".to_string()];
    let negotiated = validator.negotiate_version(&client_versions).unwrap();
    assert_eq!(negotiated, A2AProtocolVersion::V1_1);

    // Client wants 2.0, server only supports 1.0 and 1.1 -> error
    let client_versions = vec!["2.0".to_string()];
    let result = validator.negotiate_version(&client_versions);
    assert!(result.is_err());

    // Client wants 1.0 and 1.1, server supports both -> negotiate to highest compatible (1.1)
    let client_versions = vec!["1.0".to_string(), "1.1".to_string()];
    let negotiated = validator.negotiate_version(&client_versions).unwrap();
    assert_eq!(negotiated, A2AProtocolVersion::V1_1);

    // Client wants 1.1 and 2.0, server supports 1.0 and 1.1 -> negotiate to 1.1
    let client_versions = vec!["1.1".to_string(), "2.0".to_string()];
    let negotiated = validator.negotiate_version(&client_versions).unwrap();
    assert_eq!(negotiated, A2AProtocolVersion::V1_1);
}

#[test]
fn test_a2a_headers_from_constellation_message() {
    let message = Message::new(
        "test-message".to_string(),
        "sender".to_string(),
        "recipient".to_string(),
        "command".to_string(),
        "{\"action\": \"test\"}".to_string(),
    );

    let headers = A2AHeaders::from_constellation_message(&message);

    assert_eq!(headers.protocol_version, "1.0");
    assert_eq!(headers.message_type, "command");
    assert_eq!(headers.content_type, "application/json");
    assert_eq!(headers.priority, 5); // Normal priority
    assert!(headers.ttl.is_none());
    assert!(headers.correlation_id.is_none());
    assert!(headers.conversation_id.is_none());
}

#[test]
fn test_a2a_headers_apply_to_constellation_message() {
    let mut message = Message::new(
        "test-message".to_string(),
        "sender".to_string(),
        "recipient".to_string(),
        "query".to_string(), // Will be overwritten
        "{}".to_string(),
    );

    let mut headers = A2AHeaders::new("1.1".to_string(), "command".to_string());
    headers.priority = 10; // Critical
    headers.ttl = Some(1800);
    headers.correlation_id = Some("test-correlation".to_string());

    // Add custom headers
    let mut custom = std::collections::HashMap::new();
    custom.insert("X-Custom-Header".to_string(), "custom-value".to_string());
    headers.custom = custom;

    headers.apply_to_constellation_message(&mut message);

    assert_eq!(message.protocol_version, "1.1");
    assert_eq!(message.message_type, "command");
    assert_eq!(message.content_type, "application/json");
    assert_eq!(message.priority, MessagePriority::Critical);
    assert_eq!(message.ttl_seconds, Some(1800));
    assert_eq!(message.correlation_id, Some("test-correlation".to_string()));

    // Check custom headers in metadata
    assert!(message.metadata.is_some());
    let metadata = message.metadata.unwrap();
    assert!(metadata.get("headers").is_some());
    let headers_obj = metadata.get("headers").unwrap();
    assert_eq!(headers_obj["X-Custom-Header"], "custom-value");
}

#[test]
fn test_a2a_validator_validate_message() {
    let validator = A2AValidator::new();

    // Valid message
    let valid_message = Message::new(
        "test-message".to_string(),
        "sender".to_string(),
        "recipient".to_string(),
        "event".to_string(),
        "{\"data\": \"test\"}".to_string(),
    );

    assert!(validator.validate_message(&valid_message).is_ok());

    // Message with empty payload
    let empty_payload = Message::new(
        "test-message".to_string(),
        "sender".to_string(),
        "recipient".to_string(),
        "event".to_string(),
        "".to_string(), // Empty payload
    );

    assert!(validator.validate_message(&empty_payload).is_err());

    // Message with invalid protocol version
    let mut invalid_version = Message::new(
        "test-message".to_string(),
        "sender".to_string(),
        "recipient".to_string(),
        "event".to_string(),
        "{}".to_string(),
    );
    invalid_version.protocol_version = "3.0".to_string(); // Invalid version

    assert!(validator.validate_message(&invalid_version).is_err());

    // Message with negative TTL
    let mut negative_ttl = Message::new(
        "test-message".to_string(),
        "sender".to_string(),
        "recipient".to_string(),
        "event".to_string(),
        "{}".to_string(),
    );
    negative_ttl.ttl_seconds = Some(-1);

    assert!(validator.validate_message(&negative_ttl).is_err());
}

#[test]
fn test_a2a_validator_preserve_headers() {
    let validator = A2AValidator::new();

    let source = Message::new(
        "source-message".to_string(),
        "sender".to_string(),
        "recipient".to_string(),
        "command".to_string(),
        "{\"action\": \"test\"}".to_string(),
    );

    let mut target = Message::new(
        "target-message".to_string(),
        "different-sender".to_string(),
        "different-recipient".to_string(),
        "query".to_string(), // Will be overwritten
        "{}".to_string(),
    );

    // Preserve headers from source to target
    assert!(validator.preserve_headers(&source, &mut target).is_ok());

    // Check that headers were preserved
    assert_eq!(target.protocol_version, source.protocol_version);
    assert_eq!(target.message_type, source.message_type);
    assert_eq!(target.content_type, source.content_type);
    assert_eq!(target.priority, source.priority);
}

#[test]
fn test_extension_point_manager() {
    let validator = A2AValidator::new();
    let mut manager = ExtensionPointManager::new(validator);

    // Create a gossip extension
    let gossip_extension = A2AExtensionPoint::new(
        "gossip-v1".to_string(),
        "gossip".to_string(),
        "1.1".to_string(), // Requires 1.1 or higher
        serde_json::json!({
            "protocol": "epidemic",
            "fanout": 3
        }),
    );

    // Register extension
    assert!(manager.register_extension(gossip_extension).is_ok());

    // Try to register duplicate ID
    let duplicate = A2AExtensionPoint::new(
        "gossip-v1".to_string(), // Same ID
        "gossip".to_string(),
        "1.0".to_string(),
        serde_json::json!({}),
    );
    assert!(manager.register_extension(duplicate).is_err());

    // Get extension
    let extension = manager.get_extension("gossip-v1");
    assert!(extension.is_some());
    let extension = extension.unwrap();
    assert_eq!(extension.id, "gossip-v1");
    assert_eq!(extension.extension_type, "gossip");
    assert_eq!(extension.min_version, "1.1");
    assert!(extension.enabled);

    // Check compatibility
    assert!(extension.is_compatible(&A2AProtocolVersion::V1_1));
    assert!(extension.is_compatible(&A2AProtocolVersion::V2_0));
    assert!(!extension.is_compatible(&A2AProtocolVersion::V1_0)); // Requires 1.1+

    // Get enabled extensions for version
    let enabled_v1_0 = manager.get_enabled_extensions(&A2AProtocolVersion::V1_0);
    assert!(enabled_v1_0.is_empty()); // No extensions compatible with 1.0

    let enabled_v1_1 = manager.get_enabled_extensions(&A2AProtocolVersion::V1_1);
    assert_eq!(enabled_v1_1.len(), 1);

    // Check availability
    assert!(manager.is_extension_available("gossip-v1", &A2AProtocolVersion::V1_1));
    assert!(!manager.is_extension_available("gossip-v1", &A2AProtocolVersion::V1_0));
    assert!(!manager.is_extension_available("nonexistent", &A2AProtocolVersion::V1_1));

    // Disable extension
    assert!(manager.set_extension_enabled("gossip-v1", false).is_ok());
    assert!(!manager.is_extension_available("gossip-v1", &A2AProtocolVersion::V1_1));

    let enabled_v1_1 = manager.get_enabled_extensions(&A2AProtocolVersion::V1_1);
    assert!(enabled_v1_1.is_empty()); // Extension is disabled
}

#[test]
fn test_a2a_feature_support() {
    let validator = A2AValidator::new();

    // Check feature support for different versions
    assert!(validator.is_feature_supported(&A2AProtocolVersion::V1_0, A2AFeature::BasicMessaging));
    assert!(validator.is_feature_supported(&A2AProtocolVersion::V1_0, A2AFeature::PriorityQueuing));
    assert!(
        validator.is_feature_supported(&A2AProtocolVersion::V1_0, A2AFeature::DeliveryGuarantees)
    );
    assert!(
        !validator.is_feature_supported(&A2AProtocolVersion::V1_0, A2AFeature::ExtensionPoints)
    );
    assert!(
        !validator.is_feature_supported(&A2AProtocolVersion::V1_0, A2AFeature::ProtocolNegotiation)
    );

    assert!(validator.is_feature_supported(&A2AProtocolVersion::V1_1, A2AFeature::BasicMessaging));
    assert!(validator.is_feature_supported(&A2AProtocolVersion::V1_1, A2AFeature::PriorityQueuing));
    assert!(
        validator.is_feature_supported(&A2AProtocolVersion::V1_1, A2AFeature::DeliveryGuarantees)
    );
    assert!(validator.is_feature_supported(&A2AProtocolVersion::V1_1, A2AFeature::ExtensionPoints));
    assert!(
        validator.is_feature_supported(&A2AProtocolVersion::V1_1, A2AFeature::ProtocolNegotiation)
    );
    assert!(!validator.is_feature_supported(&A2AProtocolVersion::V1_1, A2AFeature::Streaming));

    assert!(validator.is_feature_supported(&A2AProtocolVersion::V2_0, A2AFeature::BasicMessaging));
    assert!(validator.is_feature_supported(&A2AProtocolVersion::V2_0, A2AFeature::PriorityQueuing));
    assert!(
        validator.is_feature_supported(&A2AProtocolVersion::V2_0, A2AFeature::DeliveryGuarantees)
    );
    assert!(validator.is_feature_supported(&A2AProtocolVersion::V2_0, A2AFeature::ExtensionPoints));
    assert!(
        validator.is_feature_supported(&A2AProtocolVersion::V2_0, A2AFeature::ProtocolNegotiation)
    );
    assert!(validator.is_feature_supported(&A2AProtocolVersion::V2_0, A2AFeature::Streaming));
    assert!(
        validator.is_feature_supported(&A2AProtocolVersion::V2_0, A2AFeature::BidirectionalStreams)
    );
}
