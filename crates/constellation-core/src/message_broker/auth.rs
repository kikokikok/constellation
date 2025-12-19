//! Authentication service for A2A message broker with MCP crypto integration.
//!
//! This module provides JWT-based authentication for agent communications
//! using MCP crypto for signing and verification.

use base64::Engine;
use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tracing::{debug, error, info, warn};

use crate::mcp::compliance::{
    ComplianceCheckResult, ComplianceFramework, ComplianceManager, DataClassification,
};
use crate::mcp::crypto::{CryptoError, KeyUsage, McpCrypto};
use crate::models::message_broker::{MessageBrokerError, MessageBrokerResult};

/// JWT claims for agent authentication
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JwtClaims {
    /// Agent ID
    pub agent_id: String,
    /// Key ID used for signing
    pub key_id: String,
    /// Expiration time (Unix timestamp)
    pub exp: i64,
    /// Issued at time (Unix timestamp)
    pub iat: i64,
    /// Not before time (Unix timestamp)
    pub nbf: Option<i64>,
    /// Issuer
    pub iss: Option<String>,
    /// Audience
    pub aud: Option<String>,
    /// Subject
    pub sub: Option<String>,
}

/// Authentication service for A2A message broker
#[derive(Clone)]
pub struct AuthService {
    /// MCP crypto for signing and verification
    crypto: Arc<McpCrypto>,
    /// JWT issuer
    issuer: String,
    /// Token expiration in seconds
    token_expiration_seconds: i64,
    /// Compliance manager for audit logging
    compliance_manager: Option<Arc<ComplianceManager>>,
}

impl AuthService {
    /// Create a new authentication service
    pub fn new(crypto: McpCrypto, issuer: String) -> Self {
        Self {
            crypto: Arc::new(crypto),
            issuer,
            token_expiration_seconds: 3600, // 1 hour default
            compliance_manager: None,
        }
    }

    /// Create a new authentication service with custom expiration
    pub fn new_with_expiration(crypto: McpCrypto, issuer: String, expiration_seconds: i64) -> Self {
        Self {
            crypto: Arc::new(crypto),
            issuer,
            token_expiration_seconds: expiration_seconds,
            compliance_manager: None,
        }
    }

    /// Create a new authentication service with compliance manager
    pub fn new_with_compliance(
        crypto: McpCrypto,
        issuer: String,
        compliance_manager: ComplianceManager,
    ) -> Self {
        Self {
            crypto: Arc::new(crypto),
            issuer,
            token_expiration_seconds: 3600,
            compliance_manager: Some(Arc::new(compliance_manager)),
        }
    }

    /// Set compliance manager
    pub fn set_compliance_manager(&mut self, compliance_manager: ComplianceManager) {
        self.compliance_manager = Some(Arc::new(compliance_manager));
    }

    /// Generate JWT token for agent
    pub fn generate_token(&self, agent_id: &str, key_id: &str) -> MessageBrokerResult<String> {
        let now = chrono::Utc::now().timestamp();

        let claims = JwtClaims {
            agent_id: agent_id.to_string(),
            key_id: key_id.to_string(),
            exp: now + self.token_expiration_seconds,
            iat: now,
            nbf: Some(now),
            iss: Some(self.issuer.clone()),
            aud: Some("constellation-a2a".to_string()),
            sub: Some(agent_id.to_string()),
        };

        // Serialize claims
        let claims_json = serde_json::to_string(&claims).map_err(|e| {
            MessageBrokerError::AuthenticationFailed(format!("Failed to serialize claims: {}", e))
        })?;

        // Create JWT header
        let header = serde_json::json!({
            "alg": "Ed25519",
            "typ": "JWT",
            "kid": key_id
        });
        let header_json = serde_json::to_string(&header).map_err(|e| {
            MessageBrokerError::AuthenticationFailed(format!("Failed to serialize header: {}", e))
        })?;

        // Encode header and payload
        let header_b64 = URL_SAFE_NO_PAD.encode(&header_json);
        let payload_b64 = URL_SAFE_NO_PAD.encode(&claims_json);

        // Create data to sign
        let data_to_sign = format!("{}.{}", header_b64, payload_b64);

        // Sign with MCP crypto
        let signature = self
            .crypto
            .sign(key_id, data_to_sign.as_bytes())
            .map_err(|e| {
                MessageBrokerError::AuthenticationFailed(format!("Failed to sign token: {}", e))
            })?;

        // Encode signature
        let signature_b64 = URL_SAFE_NO_PAD.encode(&signature);

        // Combine into JWT token
        let token = format!("{}.{}.{}", header_b64, payload_b64, signature_b64);

        // Log audit trail if compliance manager is available
        if let Some(compliance_manager) = &self.compliance_manager {
            self.log_audit_event(
                compliance_manager,
                "token_generated",
                &format!("JWT token generated for agent: {}", agent_id),
                DataClassification::Pii,
            );
        }

        info!("Generated JWT token for agent: {}", agent_id);
        Ok(token)
    }

    /// Validate JWT token
    pub fn validate_token(&self, token: &str) -> MessageBrokerResult<JwtClaims> {
        // Parse JWT token
        let parts: Vec<&str> = token.split('.').collect();
        if parts.len() != 3 {
            return Err(MessageBrokerError::AuthenticationFailed(
                "Invalid JWT format".to_string(),
            ));
        }

        let header_b64 = parts[0];
        let payload_b64 = parts[1];
        let signature_b64 = parts[2];

        // Decode and parse header to get key ID
        let header_json = URL_SAFE_NO_PAD.decode(header_b64).map_err(|e| {
            MessageBrokerError::AuthenticationFailed(format!("Failed to decode header: {}", e))
        })?;

        let header: serde_json::Value = serde_json::from_slice(&header_json).map_err(|e| {
            MessageBrokerError::AuthenticationFailed(format!("Failed to parse header: {}", e))
        })?;

        let key_id = header["kid"].as_str().ok_or_else(|| {
            MessageBrokerError::AuthenticationFailed("Missing key ID in JWT header".to_string())
        })?;

        // Decode and parse claims
        let payload_json = URL_SAFE_NO_PAD.decode(payload_b64).map_err(|e| {
            MessageBrokerError::AuthenticationFailed(format!("Failed to decode payload: {}", e))
        })?;

        let claims: JwtClaims = serde_json::from_slice(&payload_json).map_err(|e| {
            MessageBrokerError::AuthenticationFailed(format!("Failed to parse claims: {}", e))
        })?;

        // Verify signature
        let data_to_verify = format!("{}.{}", header_b64, payload_b64);
        let signature = URL_SAFE_NO_PAD.decode(signature_b64).map_err(|e| {
            MessageBrokerError::AuthenticationFailed(format!("Failed to decode signature: {}", e))
        })?;

        let is_valid = self
            .crypto
            .verify(key_id, data_to_verify.as_bytes(), &signature)
            .map_err(|e| {
                MessageBrokerError::AuthenticationFailed(format!(
                    "Signature verification failed: {}",
                    e
                ))
            })?;

        if !is_valid {
            return Err(MessageBrokerError::AuthenticationFailed(
                "Invalid signature".to_string(),
            ));
        }

        // Validate claims
        self.validate_claims(&claims)?;

        // Log audit trail if compliance manager is available
        if let Some(compliance_manager) = &self.compliance_manager {
            self.log_audit_event(
                compliance_manager,
                "token_validated",
                &format!("JWT token validated for agent: {}", claims.agent_id),
                DataClassification::Pii,
            );
        }

        debug!("Validated JWT token for agent: {}", claims.agent_id);
        Ok(claims)
    }

    /// Validate JWT claims
    fn validate_claims(&self, claims: &JwtClaims) -> MessageBrokerResult<()> {
        let now = chrono::Utc::now().timestamp();

        // Check expiration
        if claims.exp < now {
            return Err(MessageBrokerError::AuthenticationFailed(
                "Token expired".to_string(),
            ));
        }

        // Check not before
        if let Some(nbf) = claims.nbf
            && nbf > now
        {
            return Err(MessageBrokerError::AuthenticationFailed(
                "Token not yet valid".to_string(),
            ));
        }

        // Check issuer
        if let Some(iss) = &claims.iss
            && iss != &self.issuer
        {
            return Err(MessageBrokerError::AuthenticationFailed(
                "Invalid issuer".to_string(),
            ));
        }

        // Check audience
        if let Some(aud) = &claims.aud
            && aud != "constellation-a2a"
        {
            return Err(MessageBrokerError::AuthenticationFailed(
                "Invalid audience".to_string(),
            ));
        }

        Ok(())
    }

    /// Log audit event to compliance manager
    fn log_audit_event(
        &self,
        compliance_manager: &ComplianceManager,
        event_type: &str,
        description: &str,
        data_classification: DataClassification,
    ) {
        // Create compliance check result for audit logging
        let check_result = ComplianceCheckResult {
            requirement_id: format!("AUDIT-{}", event_type),
            satisfied: true,
            evidence: description.to_string(),
            checked_at: chrono::Utc::now(),
            issues: Vec::new(),
            recommendations: Vec::new(),
        };

        // In a real implementation, this would log to the compliance manager's audit trail
        debug!(
            "Audit event: {} - {} (data classification: {:?})",
            event_type, description, data_classification
        );
    }

    /// Refresh JWT token
    pub fn refresh_token(&self, token: &str) -> MessageBrokerResult<String> {
        let claims = self.validate_token(token)?;

        // Generate new token with same agent ID and key ID
        self.generate_token(&claims.agent_id, &claims.key_id)
    }

    /// Get token expiration time
    pub fn get_token_expiration(&self) -> i64 {
        self.token_expiration_seconds
    }

    /// Set token expiration time
    pub fn set_token_expiration(&mut self, expiration_seconds: i64) {
        self.token_expiration_seconds = expiration_seconds;
    }
}

/// Agent registration service
#[derive(Clone)]
pub struct AgentRegistrationService {
    /// Authentication service
    auth_service: AuthService,
    /// Registered agents
    registered_agents:
        Arc<tokio::sync::RwLock<std::collections::HashMap<String, AgentRegistration>>>,
    /// Compliance manager for audit logging
    compliance_manager: Option<Arc<ComplianceManager>>,
}

/// Agent registration information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentRegistration {
    /// Agent ID
    pub agent_id: String,
    /// Key ID used for signing
    pub key_id: String,
    /// Registration timestamp
    pub registered_at: chrono::DateTime<chrono::Utc>,
    /// Last activity timestamp
    pub last_activity_at: chrono::DateTime<chrono::Utc>,
    /// Is active
    pub active: bool,
}

impl AgentRegistrationService {
    /// Create a new agent registration service
    pub fn new(auth_service: AuthService) -> Self {
        Self {
            auth_service,
            registered_agents: Arc::new(tokio::sync::RwLock::new(std::collections::HashMap::new())),
            compliance_manager: None,
        }
    }

    /// Create a new agent registration service with compliance manager
    pub fn new_with_compliance(
        auth_service: AuthService,
        compliance_manager: ComplianceManager,
    ) -> Self {
        Self {
            auth_service,
            registered_agents: Arc::new(tokio::sync::RwLock::new(std::collections::HashMap::new())),
            compliance_manager: Some(Arc::new(compliance_manager)),
        }
    }

    /// Set compliance manager
    pub fn set_compliance_manager(&mut self, compliance_manager: ComplianceManager) {
        self.compliance_manager = Some(Arc::new(compliance_manager));
    }

    /// Log audit event to compliance manager
    fn log_audit_event(
        &self,
        compliance_manager: &ComplianceManager,
        event_type: &str,
        description: &str,
        data_classification: DataClassification,
    ) {
        // Create compliance check result for audit logging
        let check_result = ComplianceCheckResult {
            requirement_id: format!("AUDIT-{}", event_type),
            satisfied: true,
            evidence: description.to_string(),
            checked_at: chrono::Utc::now(),
            issues: Vec::new(),
            recommendations: Vec::new(),
        };

        // In a real implementation, this would log to the compliance manager's audit trail
        debug!(
            "Audit event: {} - {} (data classification: {:?})",
            event_type, description, data_classification
        );
    }

    /// Register a new agent
    pub async fn register_agent(&self, agent_id: &str) -> MessageBrokerResult<(String, String)> {
        // Generate key pair for agent
        // Note: In a real implementation, we would need access to the crypto module's key generation
        // For now, we'll use a placeholder key ID
        let key_id = format!("key-{}-{}", agent_id, uuid::Uuid::new_v4());
        let public_key_id = format!("pub-{}", key_id);

        // Generate JWT token
        let token = self.auth_service.generate_token(agent_id, &key_id)?;

        // Store registration
        let registration = AgentRegistration {
            agent_id: agent_id.to_string(),
            key_id: key_id.to_string(),
            registered_at: chrono::Utc::now(),
            last_activity_at: chrono::Utc::now(),
            active: true,
        };

        let mut agents = self.registered_agents.write().await;
        agents.insert(agent_id.to_string(), registration);

        // Log audit trail if compliance manager is available
        if let Some(compliance_manager) = &self.compliance_manager {
            self.log_audit_event(
                compliance_manager,
                "agent_registered",
                &format!("Agent registered: {}", agent_id),
                DataClassification::Pii,
            );
        }

        info!("Registered agent: {}", agent_id);
        Ok((token, public_key_id))
    }

    /// Validate agent token and update activity
    pub async fn validate_agent(&self, token: &str) -> MessageBrokerResult<String> {
        let claims = self.auth_service.validate_token(token)?;

        // Update last activity
        let mut agents = self.registered_agents.write().await;
        if let Some(registration) = agents.get_mut(&claims.agent_id) {
            registration.last_activity_at = chrono::Utc::now();
            if !registration.active {
                // Log audit trail for failed authentication
                if let Some(compliance_manager) = &self.compliance_manager {
                    self.log_audit_event(
                        compliance_manager,
                        "authentication_failed",
                        &format!(
                            "Inactive agent attempted authentication: {}",
                            claims.agent_id
                        ),
                        DataClassification::Pii,
                    );
                }

                return Err(MessageBrokerError::AuthenticationFailed(
                    "Agent is inactive".to_string(),
                ));
            }
        } else {
            // Log audit trail for unknown agent
            if let Some(compliance_manager) = &self.compliance_manager {
                self.log_audit_event(
                    compliance_manager,
                    "authentication_failed",
                    &format!(
                        "Unknown agent attempted authentication: {}",
                        claims.agent_id
                    ),
                    DataClassification::Pii,
                );
            }

            return Err(MessageBrokerError::AuthenticationFailed(
                "Agent not registered".to_string(),
            ));
        }

        // Log audit trail for successful authentication
        if let Some(compliance_manager) = &self.compliance_manager {
            self.log_audit_event(
                compliance_manager,
                "authentication_success",
                &format!("Agent authenticated: {}", claims.agent_id),
                DataClassification::Pii,
            );
        }

        Ok(claims.agent_id)
    }

    /// Deactivate agent
    pub async fn deactivate_agent(&self, agent_id: &str) -> MessageBrokerResult<()> {
        let mut agents = self.registered_agents.write().await;
        if let Some(registration) = agents.get_mut(agent_id) {
            registration.active = false;

            // Log audit trail if compliance manager is available
            if let Some(compliance_manager) = &self.compliance_manager {
                self.log_audit_event(
                    compliance_manager,
                    "agent_deactivated",
                    &format!("Agent deactivated: {}", agent_id),
                    DataClassification::Pii,
                );
            }

            info!("Deactivated agent: {}", agent_id);
            Ok(())
        } else {
            Err(MessageBrokerError::AuthenticationFailed(
                "Agent not found".to_string(),
            ))
        }
    }

    /// Get agent registration
    pub async fn get_agent_registration(&self, agent_id: &str) -> Option<AgentRegistration> {
        let agents = self.registered_agents.read().await;
        agents.get(agent_id).cloned()
    }

    /// Clean up inactive agents
    pub async fn cleanup_inactive_agents(
        &self,
        max_inactive_days: i64,
    ) -> MessageBrokerResult<usize> {
        let mut agents = self.registered_agents.write().await;
        let now = chrono::Utc::now();
        let cutoff = now - chrono::Duration::days(max_inactive_days);

        let inactive: Vec<String> = agents
            .iter()
            .filter(|(_, reg)| reg.last_activity_at < cutoff)
            .map(|(id, _)| id.clone())
            .collect();

        for agent_id in &inactive {
            agents.remove(agent_id);

            // Log audit trail if compliance manager is available
            if let Some(compliance_manager) = &self.compliance_manager {
                self.log_audit_event(
                    compliance_manager,
                    "agent_removed",
                    &format!("Removed inactive agent: {}", agent_id),
                    DataClassification::Pii,
                );
            }

            info!("Removed inactive agent: {}", agent_id);
        }

        Ok(inactive.len())
    }
}
