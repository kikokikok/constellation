//! A2A protocol validation and header preservation.
//!
//! This module provides validation for A2A protocol messages according to the
//! OpenAPI specification, including protocol version negotiation and
//! header preservation.

use serde::{Deserialize, Serialize};
use std::{borrow::Cow, collections::HashMap, str::FromStr};
use tracing::{debug, error, info, warn};
use validator::{Validate, ValidationError, ValidationErrors};

use crate::models::message_broker::{
    Message as ConstellationMessage, MessageBrokerError, MessageBrokerResult,
};

/// A2A protocol version
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, Default)]
pub enum A2AProtocolVersion {
    /// Version 1.0 - Initial release
    #[default]
    V1_0,
    /// Version 1.1 - Added extension points
    V1_1,
    /// Version 2.0 - Future major version
    V2_0,
}

impl A2AProtocolVersion {
    /// Convert to string representation
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::V1_0 => "1.0",
            Self::V1_1 => "1.1",
            Self::V2_0 => "2.0",
        }
    }

    /// Check if this version is compatible with another version
    pub fn is_compatible_with(&self, other: &Self) -> bool {
        // For now, all versions are backward compatible within major version
        match (self, other) {
            (Self::V1_0, Self::V1_0) => true,
            (Self::V1_1, Self::V1_0) => true, // 1.1 can talk to 1.0
            (Self::V1_1, Self::V1_1) => true,
            (Self::V2_0, Self::V2_0) => true,
            _ => false, // Different major versions not compatible
        }
    }

    /// Get supported features for this version
    pub fn supported_features(&self) -> Vec<A2AFeature> {
        match self {
            Self::V1_0 => vec![
                A2AFeature::BasicMessaging,
                A2AFeature::PriorityQueuing,
                A2AFeature::DeliveryGuarantees,
            ],
            Self::V1_1 => vec![
                A2AFeature::BasicMessaging,
                A2AFeature::PriorityQueuing,
                A2AFeature::DeliveryGuarantees,
                A2AFeature::ExtensionPoints,
                A2AFeature::ProtocolNegotiation,
            ],
            Self::V2_0 => vec![
                A2AFeature::BasicMessaging,
                A2AFeature::PriorityQueuing,
                A2AFeature::DeliveryGuarantees,
                A2AFeature::ExtensionPoints,
                A2AFeature::ProtocolNegotiation,
                A2AFeature::Streaming,
                A2AFeature::BidirectionalStreams,
            ],
        }
    }
}

impl FromStr for A2AProtocolVersion {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "1.0" => Ok(Self::V1_0),
            "1.1" => Ok(Self::V1_1),
            "2.0" => Ok(Self::V2_0),
            _ => Err(format!("Invalid A2A protocol version: {}", s)),
        }
    }
}

/// A2A protocol features
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum A2AFeature {
    /// Basic message sending/receiving
    BasicMessaging,
    /// Priority-based message queuing
    PriorityQueuing,
    /// Delivery guarantees (at-least-once, at-most-once, exactly-once)
    DeliveryGuarantees,
    /// Extension points for custom protocols
    ExtensionPoints,
    /// Protocol version negotiation
    ProtocolNegotiation,
    /// Streaming messages
    Streaming,
    /// Bidirectional streams
    BidirectionalStreams,
}

/// A2A message headers
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct A2AHeaders {
    /// Protocol version
    pub protocol_version: String,
    /// Message type (command, query, event, response)
    pub message_type: String,
    /// Content type
    pub content_type: String,
    /// Priority (0-10)
    pub priority: u8,
    /// Time-to-live in seconds
    pub ttl: Option<u32>,
    /// Correlation ID for request/response tracking
    pub correlation_id: Option<String>,
    /// Conversation ID for multi-message conversations
    pub conversation_id: Option<String>,
    /// Custom headers
    pub custom: HashMap<String, String>,
}

impl A2AHeaders {
    /// Create new headers with default values
    pub fn new(protocol_version: String, message_type: String) -> Self {
        Self {
            protocol_version,
            message_type,
            content_type: "application/json".to_string(),
            priority: 5,
            ttl: None,
            correlation_id: None,
            conversation_id: None,
            custom: HashMap::new(),
        }
    }

    /// Validate headers
    pub fn validate(&self) -> Result<(), ValidationErrors> {
        let mut errors = ValidationErrors::new();

        // Validate protocol version
        if self.protocol_version.parse::<A2AProtocolVersion>().is_err() {
            errors.add(
                "protocol_version",
                ValidationError::new("invalid_protocol_version")
                    .with_message(Cow::Borrowed("Invalid protocol version")),
            );
        }

        // Validate message type
        let valid_types = ["command", "query", "event", "response"];
        if !valid_types.contains(&self.message_type.as_str()) {
            errors.add(
                "message_type",
                ValidationError::new("invalid_message_type").with_message(Cow::Borrowed(
                    "Message type must be one of: command, query, event, response",
                )),
            );
        }

        // Validate priority
        if self.priority > 10 {
            errors.add(
                "priority",
                ValidationError::new("invalid_priority")
                    .with_message(Cow::Borrowed("Priority must be between 0 and 10")),
            );
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }

    /// Extract headers from Constellation message
    pub fn from_constellation_message(message: &ConstellationMessage) -> Self {
        let mut headers = Self::new(
            message.protocol_version.clone(),
            message.message_type.clone(),
        );

        headers.content_type = message.content_type.clone();
        headers.priority = match message.priority {
            crate::models::message_broker::MessagePriority::Critical => 10,
            crate::models::message_broker::MessagePriority::High => 7,
            crate::models::message_broker::MessagePriority::Normal => 5,
            crate::models::message_broker::MessagePriority::Low => 2,
        };
        headers.ttl = message.ttl_seconds.map(|s| s as u32);
        headers.correlation_id = message.correlation_id.clone();
        headers.conversation_id = message.conversation_id.clone();

        // Extract custom headers from metadata
        if let Some(metadata) = &message.metadata
            && let Some(custom_obj) = metadata.get("headers")
            && let Some(custom_map) = custom_obj.as_object()
        {
            for (key, value) in custom_map {
                if let Some(str_value) = value.as_str() {
                    headers.custom.insert(key.clone(), str_value.to_string());
                }
            }
        }

        headers
    }

    /// Apply headers to Constellation message
    pub fn apply_to_constellation_message(&self, message: &mut ConstellationMessage) {
        message.protocol_version = self.protocol_version.clone();
        message.message_type = self.message_type.clone();
        message.content_type = self.content_type.clone();

        // Map priority
        message.priority = match self.priority {
            0..=3 => crate::models::message_broker::MessagePriority::Low,
            4..=6 => crate::models::message_broker::MessagePriority::Normal,
            7..=9 => crate::models::message_broker::MessagePriority::High,
            10 => crate::models::message_broker::MessagePriority::Critical,
            _ => crate::models::message_broker::MessagePriority::Normal,
        };

        message.ttl_seconds = self.ttl.map(|t| t as i32);
        message.correlation_id = self.correlation_id.clone();
        message.conversation_id = self.conversation_id.clone();

        // Store custom headers in metadata
        if !self.custom.is_empty() {
            let mut metadata = message.metadata.take().unwrap_or_default();
            let headers_obj = serde_json::json!(self.custom);
            metadata["headers"] = headers_obj;
            message.metadata = Some(metadata);
        }
    }
}

/// A2A protocol validator
#[derive(Debug, Clone)]
pub struct A2AValidator {
    /// Supported protocol versions
    supported_versions: Vec<A2AProtocolVersion>,
    /// Strict validation mode
    strict_mode: bool,
}

impl A2AValidator {
    /// Create a new validator with default supported versions
    pub fn new() -> Self {
        Self {
            supported_versions: vec![A2AProtocolVersion::V1_0, A2AProtocolVersion::V1_1],
            strict_mode: false,
        }
    }

    /// Create a validator with custom supported versions
    pub fn with_versions(versions: Vec<A2AProtocolVersion>) -> Self {
        Self {
            supported_versions: versions,
            strict_mode: false,
        }
    }

    /// Enable strict validation mode
    pub fn strict(mut self) -> Self {
        self.strict_mode = true;
        self
    }

    /// Validate A2A headers
    pub fn validate_headers(&self, headers: &A2AHeaders) -> MessageBrokerResult<()> {
        // Validate header structure
        if let Err(errors) = headers.validate() {
            let error_msg = format!("Header validation failed: {:?}", errors);
            error!("{}", error_msg);
            return Err(MessageBrokerError::ValidationError(error_msg));
        }

        // Validate protocol version
        let version = headers
            .protocol_version
            .parse::<A2AProtocolVersion>()
            .map_err(|_| {
                MessageBrokerError::ValidationError(format!(
                    "Unsupported protocol version: {}",
                    headers.protocol_version
                ))
            })?;

        // Check if version is supported
        if !self.supported_versions.contains(&version) {
            return Err(MessageBrokerError::ValidationError(format!(
                "Protocol version {} not supported. Supported: {:?}",
                version.as_str(),
                self.supported_versions
                    .iter()
                    .map(|v| v.as_str())
                    .collect::<Vec<_>>()
            )));
        }

        debug!("A2A headers validated successfully: {:?}", headers);
        Ok(())
    }

    /// Validate Constellation message against A2A protocol
    pub fn validate_message(&self, message: &ConstellationMessage) -> MessageBrokerResult<()> {
        // Extract and validate headers
        let headers = A2AHeaders::from_constellation_message(message);
        self.validate_headers(&headers)?;

        // Validate payload (basic checks)
        if message.payload.is_empty() {
            return Err(MessageBrokerError::ValidationError(
                "Message payload cannot be empty".to_string(),
            ));
        }

        // Validate content type
        if !message.content_type.starts_with("application/") {
            warn!("Non-standard content type: {}", message.content_type);
            if self.strict_mode {
                return Err(MessageBrokerError::ValidationError(format!(
                    "Invalid content type: {}",
                    message.content_type
                )));
            }
        }

        // Validate TTL if present
        if let Some(ttl) = message.ttl_seconds
            && ttl < 0
        {
            return Err(MessageBrokerError::ValidationError(
                "TTL cannot be negative".to_string(),
            ));
        }

        info!("A2A message validated successfully: {}", message.message_id);
        Ok(())
    }

    /// Negotiate protocol version between client and server
    pub fn negotiate_version(
        &self,
        client_versions: &[String],
    ) -> MessageBrokerResult<A2AProtocolVersion> {
        // Parse client versions
        let mut parsed_versions = Vec::new();
        for version_str in client_versions {
            if let Ok(version) = version_str.parse::<A2AProtocolVersion>() {
                parsed_versions.push(version);
            }
        }

        // Sort by version (highest first)
        parsed_versions.sort_by(|a, b| b.cmp(a));

        // Find highest compatible version
        // Sort server versions highest first to prefer higher versions
        let mut sorted_server_versions = self.supported_versions.clone();
        sorted_server_versions.sort_by(|a, b| b.cmp(a));

        for client_version in &parsed_versions {
            for server_version in &sorted_server_versions {
                if client_version.is_compatible_with(server_version) {
                    let negotiated = if client_version > server_version {
                        // Use server version (downgrade)
                        *server_version
                    } else {
                        // Use client version
                        *client_version
                    };

                    info!(
                        "Protocol version negotiated: {} (client wanted: {:?}, server supports: {:?})",
                        negotiated.as_str(),
                        parsed_versions
                            .iter()
                            .map(|v| v.as_str())
                            .collect::<Vec<_>>(),
                        self.supported_versions
                            .iter()
                            .map(|v| v.as_str())
                            .collect::<Vec<_>>()
                    );

                    return Ok(negotiated);
                }
            }
        }

        Err(MessageBrokerError::ValidationError(format!(
            "No compatible protocol version found. Client supports: {:?}, Server supports: {:?}",
            parsed_versions
                .iter()
                .map(|v| v.as_str())
                .collect::<Vec<_>>(),
            self.supported_versions
                .iter()
                .map(|v| v.as_str())
                .collect::<Vec<_>>()
        )))
    }

    /// Get supported protocol versions
    pub fn supported_versions(&self) -> Vec<&'static str> {
        self.supported_versions.iter().map(|v| v.as_str()).collect()
    }

    /// Check if a feature is supported for a given version
    pub fn is_feature_supported(&self, version: &A2AProtocolVersion, feature: A2AFeature) -> bool {
        version.supported_features().contains(&feature)
    }

    /// Preserve headers during message transformation
    pub fn preserve_headers(
        &self,
        source: &ConstellationMessage,
        target: &mut ConstellationMessage,
    ) -> MessageBrokerResult<()> {
        // Extract headers from source
        let headers = A2AHeaders::from_constellation_message(source);

        // Validate headers
        self.validate_headers(&headers)?;

        // Apply headers to target
        headers.apply_to_constellation_message(target);

        debug!(
            "Headers preserved from message {} to {}",
            source.message_id, target.message_id
        );
        Ok(())
    }
}

impl Default for A2AValidator {
    fn default() -> Self {
        Self::new()
    }
}

/// Extension points for A2A protocol
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct A2AExtensionPoint {
    /// Extension identifier
    pub id: String,
    /// Extension type (gossip, toon, custom)
    pub extension_type: String,
    /// Protocol version required
    pub min_version: String,
    /// Configuration data
    pub config: serde_json::Value,
    /// Is extension enabled
    pub enabled: bool,
}

impl A2AExtensionPoint {
    /// Create a new extension point
    pub fn new(
        id: String,
        extension_type: String,
        min_version: String,
        config: serde_json::Value,
    ) -> Self {
        Self {
            id,
            extension_type,
            min_version,
            config,
            enabled: true,
        }
    }

    /// Check if extension is compatible with protocol version
    pub fn is_compatible(&self, version: &A2AProtocolVersion) -> bool {
        if let Ok(min_version) = self.min_version.parse::<A2AProtocolVersion>() {
            version >= &min_version
        } else {
            false
        }
    }
}

/// Extension point manager
#[derive(Debug, Clone)]
pub struct ExtensionPointManager {
    /// Registered extension points
    extensions: HashMap<String, A2AExtensionPoint>,
    /// Protocol validator
    validator: A2AValidator,
}

impl ExtensionPointManager {
    /// Create a new extension point manager
    pub fn new(validator: A2AValidator) -> Self {
        Self {
            extensions: HashMap::new(),
            validator,
        }
    }

    /// Register an extension point
    pub fn register_extension(&mut self, extension: A2AExtensionPoint) -> MessageBrokerResult<()> {
        // Validate extension
        if extension.id.is_empty() {
            return Err(MessageBrokerError::ValidationError(
                "Extension ID cannot be empty".to_string(),
            ));
        }

        if extension.extension_type.is_empty() {
            return Err(MessageBrokerError::ValidationError(
                "Extension type cannot be empty".to_string(),
            ));
        }

        // Check if extension ID already exists
        if self.extensions.contains_key(&extension.id) {
            return Err(MessageBrokerError::ValidationError(format!(
                "Extension with ID '{}' already registered",
                extension.id
            )));
        }

        let extension_id = extension.id.clone();
        self.extensions.insert(extension_id.clone(), extension);
        info!("Extension point registered: {}", extension_id);
        Ok(())
    }

    /// Get extension point by ID
    pub fn get_extension(&self, id: &str) -> Option<&A2AExtensionPoint> {
        self.extensions.get(id)
    }

    /// Get all enabled extensions for a protocol version
    pub fn get_enabled_extensions(&self, version: &A2AProtocolVersion) -> Vec<&A2AExtensionPoint> {
        self.extensions
            .values()
            .filter(|ext| ext.enabled && ext.is_compatible(version))
            .collect()
    }

    /// Check if an extension is available for a protocol version
    pub fn is_extension_available(&self, id: &str, version: &A2AProtocolVersion) -> bool {
        self.extensions
            .get(id)
            .map(|ext| ext.enabled && ext.is_compatible(version))
            .unwrap_or(false)
    }

    /// Enable/disable an extension
    pub fn set_extension_enabled(&mut self, id: &str, enabled: bool) -> MessageBrokerResult<()> {
        if let Some(extension) = self.extensions.get_mut(id) {
            extension.enabled = enabled;
            info!(
                "Extension {} {}",
                id,
                if enabled { "enabled" } else { "disabled" }
            );
            Ok(())
        } else {
            Err(MessageBrokerError::ValidationError(format!(
                "Extension '{}' not found",
                id
            )))
        }
    }
}
