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

/// Key management configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KeyManagement {
    pub rotation_policy: RotationPolicy,
    pub storage: KeyStorage,
    pub backup_policy: BackupPolicy,
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
    pub fn add_compliance(&mut self, standard: ComplianceStandard, requirement_id: String, description: String) {
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
            req.standard == standard && 
            req.requirement_id == requirement_id && 
            matches!(req.status, ComplianceStatus::Implemented | ComplianceStatus::Verified)
        })
    }
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
                    permissions: vec![
                        Permission {
                            resource: "*".to_string(),
                            action: "*".to_string(),
                        },
                    ],
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
        }
    }
}

impl McpSecureEnvelope {
    /// Create a new secure envelope.
    pub fn new(sender: String, recipient: String, message_type: String, payload: McpEncryptedMessage, signature: McpSignature) -> Self {
        Self {
            message_id: Uuid::new_v4(),
            sender,
            recipient,
            message_type,
            payload,
            signature,
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