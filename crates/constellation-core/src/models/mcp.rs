//! Model Context Protocol (MCP) security models.
//!
//! MCP provides cryptographic provenance and security for agent communications,
//! ensuring data integrity, authenticity, and non-repudiation.

use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// MCP security context for agent communications.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpSecurityContext {
    /// Unique identifier for this security context.
    pub id: Uuid,

    /// Protocol version.
    pub protocol_version: String,

    /// Security level.
    pub security_level: SecurityLevel,

    /// Cryptographic algorithms in use.
    pub algorithms: McpAlgorithms,

    /// Key management configuration.
    pub key_management: KeyManagement,

    /// Access control policies.
    pub access_control: AccessControl,

    /// Audit logging configuration.
    pub audit_logging: AuditLogging,

    /// Compliance requirements.
    pub compliance: Vec<ComplianceRequirement>,
}

/// Security level for MCP communications.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum SecurityLevel {
    Low,
    Medium,
    High,
    Critical,
}

/// Cryptographic algorithms used in MCP.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpAlgorithms {
    pub signature: String,
    pub encryption: String,
    pub key_exchange: String,
    pub hash: String,
}

impl Default for McpAlgorithms {
    fn default() -> Self {
        Self {
            signature: "Ed25519".to_string(),
            encryption: "AES-256-GCM".to_string(),
            key_exchange: "X25519".to_string(),
            hash: "SHA-256".to_string(),
        }
    }
}

/// Key management configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeyManagement {
    pub rotation_policy: RotationPolicy,
    pub storage: KeyStorage,
    pub backup_policy: BackupPolicy,
    /// Metadata for key management
    #[serde(default)]
    pub metadata: std::collections::HashMap<String, serde_json::Value>,
}

/// Key rotation policy.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RotationPolicy {
    pub interval_days: u32,
    pub rotate_on_compromise: bool,
    pub max_lifetime_days: u32,
}

/// Key storage method.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum KeyStorage {
    Memory,
    EncryptedDisk,
    Hsm,
    CloudKms,
}

/// Key backup policy.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackupPolicy {
    pub enabled: bool,
    pub frequency_days: u32,
    pub encrypted: bool,
}

/// Access control policies.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccessControl {
    pub authentication: AuthenticationMethod,
    pub authorization: AuthorizationModel,
    pub roles: Vec<Role>,
    /// Metadata for storing access rules and other configuration
    #[serde(default)]
    pub metadata: std::collections::HashMap<String, serde_json::Value>,
}

/// Authentication method.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AuthenticationMethod {
    PublicKey,
    Certificate,
    OAuth2,
    MutualTls,
}

/// Authorization model.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AuthorizationModel {
    Rbac,
    Abac,
    Capabilities,
}

/// Role definition.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Role {
    pub name: String,
    pub description: String,
    pub permissions: Vec<Permission>,
}

/// Permission definition.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Permission {
    pub resource: String,
    pub action: String,
}

/// Audit logging configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditLogging {
    pub enabled: bool,
    pub retention_days: u32,
    pub events_to_log: Vec<AuditEvent>,
    /// Metadata for storing audit logs
    #[serde(default)]
    pub metadata: std::collections::HashMap<String, serde_json::Value>,
}

/// Audit event to log.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditEvent {
    pub event_type: AuditEventType,
    pub severity: AuditSeverity,
    pub log_success: bool,
    pub log_failure: bool,
}

/// Audit event type.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AuditEventType {
    Authentication,
    Authorization,
    DataAccess,
    SecurityEvent,
}

/// Audit severity level.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AuditSeverity {
    Informational,
    Warning,
    Error,
    Critical,
}

/// Compliance requirement.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComplianceRequirement {
    pub standard: ComplianceStandard,
    pub requirement_id: String,
    pub description: String,
    pub status: ComplianceStatus,
}

/// Compliance standard.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum ComplianceStandard {
    Gdpr,
    Hipaa,
    PciDss,
    Soc2,
    Iso27001,
}

/// Compliance status.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum ComplianceStatus {
    NotImplemented,
    PartiallyImplemented,
    Implemented,
    Verified,
}

/// MCP cryptographic signature for messages.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpSignature {
    pub signer: String,
    pub algorithm: String,
    pub signature: String,
    pub signed_at: chrono::DateTime<chrono::Utc>,
    pub nonce: String,
    pub key_id: String,
}

/// MCP encrypted message.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpEncryptedMessage {
    pub ciphertext: String,
    pub algorithm: String,
    pub iv: Option<String>,
    pub key_id: String,
}

/// MCP secure message envelope.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpSecureEnvelope {
    pub message_id: Uuid,
    pub sender: String,
    pub recipient: String,
    pub message_type: String,
    pub payload: McpEncryptedMessage,
    pub signature: McpSignature,
    pub security_level: SecurityLevel,
    pub sent_at: chrono::DateTime<chrono::Utc>,
    pub expires_at: Option<chrono::DateTime<chrono::Utc>>,
}

impl McpSecurityContext {
    /// Create a new MCP security context with default settings.
    pub fn new(security_level: SecurityLevel) -> Self {
        Self {
            id: Uuid::new_v4(),
            protocol_version: "1.0.0".to_string(),
            security_level,
            algorithms: McpAlgorithms::default(),
            key_management: KeyManagement::default(),
            access_control: AccessControl::default(),
            audit_logging: AuditLogging::default(),
            compliance: Vec::new(),
        }
    }

    /// Create a high-security context for critical operations.
    pub fn high_security() -> Self {
        let mut context = Self::new(SecurityLevel::High);

        context.algorithms.signature = "Ed25519".to_string();
        context.algorithms.encryption = "AES-256-GCM".to_string();
        context.key_management.storage = KeyStorage::Hsm;
        context.access_control.authentication = AuthenticationMethod::MutualTls;

        context
    }

    /// Add a compliance requirement.
    pub fn add_compliance(
        &mut self,
        standard: ComplianceStandard,
        requirement_id: String,
        description: String,
    ) {
        let requirement = ComplianceRequirement {
            standard,
            requirement_id,
            description,
            status: ComplianceStatus::NotImplemented,
        };
        self.compliance.push(requirement);
    }

    /// Check if the context meets a specific compliance requirement.
    pub fn is_compliant(&self, standard: ComplianceStandard, requirement_id: &str) -> bool {
        self.compliance.iter().any(|req| {
            req.standard == standard
                && req.requirement_id == requirement_id
                && matches!(
                    req.status,
                    ComplianceStatus::Implemented | ComplianceStatus::Verified
                )
        })
    }
}

impl Default for KeyManagement {
    fn default() -> Self {
        Self {
            rotation_policy: RotationPolicy {
                interval_days: 90,
                rotate_on_compromise: true,
                max_lifetime_days: 365,
            },
            storage: KeyStorage::EncryptedDisk,
            backup_policy: BackupPolicy {
                enabled: true,
                frequency_days: 7,
                encrypted: true,
            },
            metadata: std::collections::HashMap::new(),
        }
    }
}

impl KeyManagement {
    /// Create a new KeyManagement instance
    pub fn new() -> Self {
        Self::default()
    }

    /// Register a key with metadata
    pub fn register_key(&mut self, key_id: &str, metadata: &crate::mcp::crypto::KeyMetadata) {
        self.metadata.insert(
            key_id.to_string(),
            serde_json::json!({
                "key_id": key_id,
                "created_at": metadata.created_at.to_rfc3339(),
                "expires_at": metadata.expires_at.map(|dt| dt.to_rfc3339()),
                "owner": metadata.owner,
                "usage": format!("{:?}", metadata.usage),
                "active": metadata.active,
            }),
        );
    }

    /// Rotate a key (mark old as inactive, new as active)
    pub fn rotate_key(&mut self, old_key_id: &str, new_key_id: &str) {
        if let Some(old_metadata) = self.metadata.get_mut(old_key_id)
            && let Some(obj) = old_metadata.as_object_mut()
        {
            obj.insert("active".to_string(), serde_json::Value::Bool(false));
            obj.insert(
                "rotated_to".to_string(),
                serde_json::Value::String(new_key_id.to_string()),
            );
            obj.insert(
                "rotated_at".to_string(),
                serde_json::Value::String(chrono::Utc::now().to_rfc3339()),
            );
        }
    }
}

impl Default for AccessControl {
    fn default() -> Self {
        Self {
            authentication: AuthenticationMethod::PublicKey,
            authorization: AuthorizationModel::Rbac,
            roles: vec![
                Role {
                    name: "admin".to_string(),
                    description: "System administrator".to_string(),
                    permissions: vec![Permission {
                        resource: "*".to_string(),
                        action: "*".to_string(),
                    }],
                },
                Role {
                    name: "user".to_string(),
                    description: "Regular user".to_string(),
                    permissions: vec![
                        Permission {
                            resource: "data".to_string(),
                            action: "read".to_string(),
                        },
                        Permission {
                            resource: "data".to_string(),
                            action: "write".to_string(),
                        },
                    ],
                },
            ],
            metadata: std::collections::HashMap::new(),
        }
    }
}

impl Default for AuditLogging {
    fn default() -> Self {
        Self {
            enabled: true,
            retention_days: 365,
            events_to_log: vec![
                AuditEvent {
                    event_type: AuditEventType::Authentication,
                    severity: AuditSeverity::Informational,
                    log_success: true,
                    log_failure: true,
                },
                AuditEvent {
                    event_type: AuditEventType::Authorization,
                    severity: AuditSeverity::Warning,
                    log_success: false,
                    log_failure: true,
                },
                AuditEvent {
                    event_type: AuditEventType::SecurityEvent,
                    severity: AuditSeverity::Critical,
                    log_success: true,
                    log_failure: true,
                },
            ],
            metadata: std::collections::HashMap::new(),
        }
    }
}

impl AccessControl {
    /// Create a new AccessControl instance
    pub fn new() -> Self {
        Self::default()
    }

    /// Check if a subject is authorized to perform an action on a resource
    pub fn is_authorized(&self, subject_id: &str, resource_id: &str, action: &str) -> bool {
        // Check metadata for specific rules first
        let rule_key = format!("rule_{subject_id}_{resource_id}_{action}");
        if self.metadata.contains_key(&rule_key) {
            return true;
        }

        // Check role-based permissions
        for role in &self.roles {
            // In a real implementation, we would check if the subject has this role
            // For now, we'll check if any role has the required permission
            for permission in &role.permissions {
                if (permission.resource == "*" || permission.resource == resource_id)
                    && (permission.action == "*" || permission.action == action)
                {
                    return true;
                }
            }
        }

        false
    }

    /// Add an access rule
    pub fn add_rule(
        &mut self,
        subject_id: &str,
        resource_id: &str,
        action: &str,
        security_level: SecurityLevel,
    ) {
        let rule_key = format!("rule_{subject_id}_{resource_id}_{action}");
        self.metadata.insert(
            rule_key,
            serde_json::json!({
                "subject_id": subject_id,
                "resource_id": resource_id,
                "action": action,
                "security_level": format!("{:?}", security_level),
                "created_at": chrono::Utc::now().to_rfc3339(),
            }),
        );
    }
}

impl AuditLogging {
    /// Create a new AuditLogging instance
    pub fn new() -> Self {
        Self::default()
    }

    /// Log an audit event
    pub fn log_event(
        &mut self,
        event_type: &str,
        message: &str,
        resource_id: Option<&str>,
        security_level: SecurityLevel,
    ) {
        if !self.enabled {
            return;
        }

        let timestamp = chrono::Utc::now();
        let log_key = format!("audit_{}", timestamp.timestamp());

        self.metadata.insert(
            log_key,
            serde_json::json!({
                "event_type": event_type,
                "message": message,
                "resource_id": resource_id,
                "security_level": format!("{:?}", security_level),
                "timestamp": timestamp.to_rfc3339(),
            }),
        );
    }

    /// Get audit logs within a time range
    pub fn get_logs(
        &self,
        start_time: Option<chrono::DateTime<chrono::Utc>>,
        end_time: Option<chrono::DateTime<chrono::Utc>>,
        security_level: Option<SecurityLevel>,
    ) -> Vec<String> {
        let mut logs = Vec::new();

        for (key, value) in &self.metadata {
            if !key.starts_with("audit_") {
                continue;
            }

            if let Some(log_data) = value.as_object()
                && let Some(timestamp_str) = log_data.get("timestamp").and_then(|v| v.as_str())
                && let Ok(timestamp) = chrono::DateTime::parse_from_rfc3339(timestamp_str)
            {
                let timestamp_utc = timestamp.with_timezone(&chrono::Utc);

                // Filter by time range
                if let Some(start) = start_time
                    && timestamp_utc < start
                {
                    continue;
                }

                if let Some(end) = end_time
                    && timestamp_utc > end
                {
                    continue;
                }

                // Filter by security level
                if let Some(level) = &security_level
                    && let Some(log_level_str) =
                        log_data.get("security_level").and_then(|v| v.as_str())
                {
                    let log_level = format!("{level:?}");
                    if log_level_str != log_level {
                        continue;
                    }
                }

                if let Some(message) = log_data.get("message").and_then(|v| v.as_str()) {
                    logs.push(message.to_string());
                }
            }
        }

        logs.sort();
        logs
    }
}

impl McpSecureEnvelope {
    /// Create a new secure envelope.
    pub fn new(
        sender: String,
        recipient: String,
        message_type: String,
        payload: McpEncryptedMessage,
        signature: McpSignature,
        security_level: SecurityLevel,
    ) -> Self {
        Self {
            message_id: Uuid::new_v4(),
            sender,
            recipient,
            message_type,
            payload,
            signature,
            security_level,
            sent_at: chrono::Utc::now(),
            expires_at: None,
        }
    }

    /// Check if the envelope has expired.
    pub fn is_expired(&self) -> bool {
        if let Some(expires_at) = self.expires_at {
            chrono::Utc::now() > expires_at
        } else {
            false
        }
    }

    /// Set expiration time.
    pub fn set_expiration(&mut self, hours_from_now: u32) {
        self.expires_at = Some(chrono::Utc::now() + chrono::Duration::hours(hours_from_now as i64));
    }
}
