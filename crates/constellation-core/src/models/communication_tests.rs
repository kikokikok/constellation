//! Comprehensive tests for communication models

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::communication::*;
    use crate::models::message_broker::MessagePriority;
    use std::time::Duration;

    #[test]
    fn test_communication_pattern_enum() {
        // Test enum variants
        let patterns = vec![
            CommunicationPattern::RequestResponse,
            CommunicationPattern::PublishSubscribe,
            CommunicationPattern::FireAndForget,
            CommunicationPattern::Broadcast,
        ];

        for pattern in patterns {
            match pattern {
                CommunicationPattern::RequestResponse => assert!(true),
                CommunicationPattern::PublishSubscribe => assert!(true),
                CommunicationPattern::FireAndForget => assert!(true),
                CommunicationPattern::Broadcast => assert!(true),
            }
        }

        // Test equality
        assert_eq!(
            CommunicationPattern::RequestResponse,
            CommunicationPattern::RequestResponse
        );
        assert_ne!(
            CommunicationPattern::RequestResponse,
            CommunicationPattern::PublishSubscribe
        );
    }

    #[test]
    fn test_delivery_guarantee_enum() {
        // Test enum variants
        let guarantees = vec![
            DeliveryGuarantee::BestEffort,
            DeliveryGuarantee::AtLeastOnce,
            DeliveryGuarantee::AtMostOnce,
            DeliveryGuarantee::ExactlyOnce,
        ];

        for guarantee in guarantees {
            match guarantee {
                DeliveryGuarantee::BestEffort => assert!(true),
                DeliveryGuarantee::AtLeastOnce => assert!(true),
                DeliveryGuarantee::AtMostOnce => assert!(true),
                DeliveryGuarantee::ExactlyOnce => assert!(true),
            }
        }

        // Test equality
        assert_eq!(DeliveryGuarantee::BestEffort, DeliveryGuarantee::BestEffort);
        assert_ne!(
            DeliveryGuarantee::BestEffort,
            DeliveryGuarantee::AtLeastOnce
        );
    }

    #[test]
    fn test_request_config_default() {
        let config = RequestConfig::default();

        assert_eq!(config.timeout, Duration::from_secs(30));
        assert_eq!(config.max_retries, 3);
        assert!(config.use_exponential_backoff);
        assert_eq!(config.retry_base_delay, Duration::from_secs(1));
        assert!(config.require_response);
    }

    #[test]
    fn test_response_config_default() {
        let config = ResponseConfig::default();

        assert!(config.include_request);
        assert!(config.validate_response);
        assert_eq!(config.max_size_bytes, Some(10 * 1024 * 1024)); // 10 MB default
    }

    #[test]
    fn test_topic_pattern_matching() {
        // Test exact matching
        let exact = TopicPattern::Exact("system.alerts".to_string());
        assert!(exact.matches("system.alerts"));
        assert!(!exact.matches("system.metrics"));
        assert!(!exact.matches("system.alerts.critical"));

        // Test wildcard matching
        let wildcard = TopicPattern::Wildcard("system.*".to_string());
        assert!(wildcard.matches("system.alerts"));
        assert!(wildcard.matches("system.metrics"));
        assert!(!wildcard.matches("system"));
        assert!(!wildcard.matches("system.alerts.critical"));

        // Test multi-level wildcard
        let multi_wildcard = TopicPattern::Wildcard("system.*.critical".to_string());
        assert!(multi_wildcard.matches("system.alerts.critical"));
        assert!(multi_wildcard.matches("system.metrics.critical"));
        assert!(!multi_wildcard.matches("system.alerts"));
        assert!(!multi_wildcard.matches("system.critical"));

        // Test regex matching
        let regex = TopicPattern::Regex("^system\\.(alerts|metrics)$".to_string());
        assert!(regex.matches("system.alerts"));
        assert!(regex.matches("system.metrics"));
        assert!(!regex.matches("system.logs"));
        assert!(!regex.matches("system.alerts.critical"));

        // Test invalid regex (should not panic)
        let invalid_regex = TopicPattern::Regex("[invalid".to_string());
        assert!(!invalid_regex.matches("anything"));
    }

    #[test]
    fn test_subscription_creation() {
        let subscription = Subscription::new(
            "agent_123".to_string(),
            TopicPattern::Exact("test.topic".to_string()),
        );

        assert!(!subscription.id.is_empty());
        assert_eq!(subscription.agent_id, "agent_123");
        assert_eq!(
            subscription.topic_pattern,
            TopicPattern::Exact("test.topic".to_string())
        );
        assert!(subscription.active);
        assert!(subscription.created_at <= chrono::Utc::now());
        assert_eq!(subscription.last_message_at, None);
    }

    #[test]
    fn test_subscription_update_last_message() {
        let mut subscription = Subscription::new(
            "agent_123".to_string(),
            TopicPattern::Exact("test.topic".to_string()),
        );

        assert_eq!(subscription.last_message_at, None);

        subscription.update_last_message();

        assert!(subscription.last_message_at.is_some());
        let timestamp = subscription.last_message_at.unwrap();
        assert!(timestamp <= chrono::Utc::now());
    }

    #[test]
    fn test_request_message_creation() {
        let config = RequestConfig {
            timeout: Duration::from_secs(10),
            max_retries: 2,
            use_exponential_backoff: false,
            retry_base_delay: Duration::from_millis(100),
            require_response: true,
        };

        let request = RequestMessage::new(
            "sender_123".to_string(),
            "recipient_456".to_string(),
            "{\"action\": \"test\"}".to_string(),
            config.clone(),
            MessagePriority::Normal,
        );

        assert!(!request.request_id.is_empty());
        assert!(!request.correlation_id.is_empty());
        assert_eq!(request.sender_id, "sender_123");
        assert_eq!(request.recipient, "recipient_456");
        assert_eq!(request.payload, "{\"action\": \"test\"}");
        assert_eq!(request.config.timeout, config.timeout);
        assert_eq!(request.config.max_retries, config.max_retries);
        assert_eq!(
            request.config.use_exponential_backoff,
            config.use_exponential_backoff
        );
        assert_eq!(request.config.retry_base_delay, config.retry_base_delay);
        assert_eq!(request.config.require_response, config.require_response);
        assert_eq!(request.retry_count, 0);
        assert_eq!(request.priority, MessagePriority::Normal);

        // Check timestamps
        assert!(request.created_at <= chrono::Utc::now());
        assert!(request.expires_at > request.created_at);
    }

    #[test]
    fn test_request_message_expiration() {
        let config = RequestConfig {
            timeout: Duration::from_secs(0), // Zero timeout
            max_retries: 0,
            use_exponential_backoff: false,
            retry_base_delay: Duration::from_secs(0),
            require_response: true,
        };

        let request = RequestMessage::new(
            "sender".to_string(),
            "recipient".to_string(),
            "test".to_string(),
            config,
            MessagePriority::Normal,
        );

        // With zero timeout, expires_at equals created_at
        // The request is considered expired if now > expires_at
        // Since they might be equal, we need to check the logic
        // For zero timeout, the request should not be retryable
        assert!(!request.can_retry());

        // Wait a tiny bit to ensure expiration
        std::thread::sleep(Duration::from_millis(1));

        // Now it should be expired
        assert!(request.is_expired());
    }

    #[test]
    fn test_request_message_retry_logic() {
        let config = RequestConfig {
            timeout: Duration::from_secs(60),
            max_retries: 3,
            use_exponential_backoff: true,
            retry_base_delay: Duration::from_secs(1),
            require_response: true,
        };

        let mut request = RequestMessage::new(
            "sender".to_string(),
            "recipient".to_string(),
            "test".to_string(),
            config,
            MessagePriority::Normal,
        );

        // Initially can retry
        assert!(request.can_retry());

        // Test exponential backoff
        assert_eq!(request.next_retry_delay(), Duration::from_secs(1)); // 2^0 = 1

        // Simulate retries
        request.retry_count = 1;
        assert_eq!(request.next_retry_delay(), Duration::from_secs(2)); // 2^1 = 2

        request.retry_count = 2;
        assert_eq!(request.next_retry_delay(), Duration::from_secs(4)); // 2^2 = 4

        request.retry_count = 3;
        assert_eq!(request.next_retry_delay(), Duration::from_secs(8)); // 2^3 = 8

        // After max retries, can't retry anymore
        request.retry_count = 3; // max_retries = 3
        assert!(!request.can_retry());
    }

    #[test]
    fn test_request_message_to_a2a_message() {
        let config = RequestConfig::default();
        let request = RequestMessage::new(
            "sender".to_string(),
            "recipient".to_string(),
            "{\"test\": \"data\"}".to_string(),
            config,
            MessagePriority::High,
        );

        let a2a_message = request.to_a2a_message();

        assert_eq!(a2a_message.message_id, request.request_id);
        assert_eq!(a2a_message.sender_id, request.sender_id);
        assert_eq!(a2a_message.recipient_id, request.recipient);
        assert_eq!(a2a_message.message_type, "request");
        assert_eq!(a2a_message.payload, request.payload);
        assert_eq!(a2a_message.correlation_id, Some(request.correlation_id));
        assert_eq!(a2a_message.priority, MessagePriority::High);
    }

    #[test]
    fn test_response_message_creation() {
        // Test success response
        let success_response = ResponseMessage::success(
            "corr_123".to_string(),
            "responder".to_string(),
            "requester".to_string(),
            "{\"result\": \"success\"}".to_string(),
            None,
        );

        assert!(!success_response.response_id.is_empty());
        assert_eq!(success_response.correlation_id, "corr_123");
        assert_eq!(success_response.sender_id, "responder");
        assert_eq!(success_response.recipient_id, "requester");
        assert_eq!(success_response.payload, "{\"result\": \"success\"}");
        assert_eq!(success_response.status, ResponseStatus::Success);
        assert!(success_response.created_at <= chrono::Utc::now());
        assert_eq!(success_response.original_request, None);
        assert_eq!(success_response.priority, MessagePriority::Normal);
        assert_eq!(success_response.ttl_seconds, None);

        // Test error response
        let error_response = ResponseMessage::error(
            "corr_456".to_string(),
            "responder".to_string(),
            "requester".to_string(),
            "Something went wrong".to_string(),
            None,
        );

        assert_eq!(error_response.correlation_id, "corr_456");
        match error_response.status {
            ResponseStatus::Error(msg) => assert_eq!(msg, "Something went wrong"),
            _ => panic!("Expected error status"),
        }
    }

    #[test]
    fn test_response_message_to_a2a_message() {
        let response = ResponseMessage::success(
            "corr_123".to_string(),
            "responder".to_string(),
            "requester".to_string(),
            "{\"result\": \"ok\"}".to_string(),
            None,
        );

        let a2a_message = response.to_a2a_message();

        assert_eq!(a2a_message.message_id, response.response_id);
        assert_eq!(a2a_message.sender_id, response.sender_id);
        assert_eq!(a2a_message.recipient_id, response.recipient_id);
        assert_eq!(a2a_message.message_type, "response");
        assert_eq!(a2a_message.payload, response.payload);
        assert_eq!(a2a_message.correlation_id, Some(response.correlation_id));
        assert_eq!(a2a_message.priority, MessagePriority::Normal);
    }

    #[test]
    fn test_response_status_equality() {
        assert_eq!(ResponseStatus::Success, ResponseStatus::Success);
        assert_eq!(
            ResponseStatus::Error("msg1".to_string()),
            ResponseStatus::Error("msg1".to_string())
        );
        assert_eq!(ResponseStatus::Timeout, ResponseStatus::Timeout);
        assert_eq!(
            ResponseStatus::Rejected("reason1".to_string()),
            ResponseStatus::Rejected("reason1".to_string())
        );

        assert_ne!(
            ResponseStatus::Success,
            ResponseStatus::Error("msg".to_string())
        );
        assert_ne!(
            ResponseStatus::Error("msg1".to_string()),
            ResponseStatus::Error("msg2".to_string())
        );
    }

    #[test]
    fn test_notification_message_creation() {
        let notification = NotificationMessage::new(
            "sender".to_string(),
            "recipient".to_string(),
            "{\"event\": \"update\"}".to_string(),
            DeliveryGuarantee::AtLeastOnce,
            MessagePriority::High,
        );

        assert!(!notification.message_id.is_empty());
        assert_eq!(notification.sender_id, "sender");
        assert_eq!(notification.recipient, "recipient");
        assert_eq!(notification.payload, "{\"event\": \"update\"}");
        assert_eq!(
            notification.delivery_guarantee,
            DeliveryGuarantee::AtLeastOnce
        );
        assert_eq!(notification.priority, MessagePriority::High);
        assert!(notification.created_at <= chrono::Utc::now());
    }

    #[test]
    fn test_notification_message_to_a2a_message() {
        let notification = NotificationMessage::new(
            "sender".to_string(),
            "recipient".to_string(),
            "test".to_string(),
            DeliveryGuarantee::ExactlyOnce,
            MessagePriority::Critical,
        );

        let a2a_message = notification.to_a2a_message();

        assert_eq!(a2a_message.message_id, notification.message_id);
        assert_eq!(a2a_message.sender_id, notification.sender_id);
        assert_eq!(a2a_message.recipient_id, notification.recipient);
        assert_eq!(a2a_message.message_type, "notification");
        assert_eq!(a2a_message.payload, notification.payload);
        assert_eq!(a2a_message.priority, MessagePriority::Critical);
    }

    #[test]
    fn test_publish_message_creation() {
        let publish = PublishMessage::new(
            "publisher".to_string(),
            "system.alerts".to_string(),
            "{\"alert\": \"critical\"}".to_string(),
            DeliveryGuarantee::AtLeastOnce,
            MessagePriority::High,
            Some(60),
        );

        assert!(!publish.message_id.is_empty());
        assert_eq!(publish.publisher_id, "publisher");
        assert_eq!(publish.topic, "system.alerts");
        assert_eq!(publish.payload, "{\"alert\": \"critical\"}");
        assert_eq!(publish.delivery_guarantee, DeliveryGuarantee::AtLeastOnce);
        assert_eq!(publish.priority, MessagePriority::High);
        assert_eq!(publish.ttl_seconds, Some(60));
        assert!(publish.created_at <= chrono::Utc::now());
    }

    #[test]
    fn test_publish_message_to_a2a_message() {
        let publish = PublishMessage::new(
            "publisher".to_string(),
            "topic.name".to_string(),
            "data".to_string(),
            DeliveryGuarantee::BestEffort,
            MessagePriority::Normal,
            Some(30),
        );

        let a2a_message = publish.to_a2a_message();

        assert_eq!(a2a_message.message_id, publish.message_id);
        assert_eq!(a2a_message.sender_id, publish.publisher_id);
        assert_eq!(a2a_message.recipient_id, publish.topic);
        assert_eq!(a2a_message.message_type, "publish");
        assert_eq!(a2a_message.payload, publish.payload);
        assert_eq!(a2a_message.priority, MessagePriority::Normal);
        assert_eq!(a2a_message.ttl_seconds, Some(30));
    }

    #[test]
    fn test_communication_error_display() {
        let errors = vec![
            CommunicationError::Timeout("Request timed out".to_string()),
            CommunicationError::MaxRetriesExceeded("Max retries exceeded".to_string()),
            CommunicationError::InvalidResponse("Invalid response".to_string()),
            CommunicationError::SubscriptionError("Subscription error".to_string()),
            CommunicationError::PatternNotSupported("Pattern not supported".to_string()),
        ];

        for error in errors {
            let display = format!("{}", error);
            assert!(!display.is_empty());
            // Check that the error message contains the error type (timeout, retries, etc.)
            assert!(
                display.contains("timeout")
                    || display.contains("retries")
                    || display.contains("response")
                    || display.contains("Subscription")
                    || display.contains("Pattern")
                    || display.contains("broker")
                    || display.contains("Serialization")
            );
        }
    }
}
