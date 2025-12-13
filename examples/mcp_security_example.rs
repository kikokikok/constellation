//! Example demonstrating MCP (Model Context Protocol) security usage.
//!
//! This example shows how to create secure agent communications with
//! cryptographic provenance and security layers.

use constellation_core::{
    McpSecurityContext, SecurityLevel, McpSecureEnvelope, McpEncryptedMessage, McpSignature,
};
use constellation_core::models::mcp::ComplianceStandard;

fn main() {
    println!("=== MCP Security Example ===\n");
    
    // Create different security contexts
    let medium_context = McpSecurityContext::new(SecurityLevel::Medium);
    let high_context = McpSecurityContext::high_security();
    
    println!("Medium Security Context:");
    println!("  ID: {}", medium_context.id);
    println!("  Security Level: {:?}", medium_context.security_level);
    println!("  Signature Algorithm: {}", medium_context.algorithms.signature);
    println!("  Encryption Algorithm: {}", medium_context.algorithms.encryption);
    println!("  Key Storage: {:?}", medium_context.key_management.storage);
    println!("  Authentication: {:?}", medium_context.access_control.authentication);
    
    println!("\nHigh Security Context:");
    println!("  ID: {}", high_context.id);
    println!("  Security Level: {:?}", high_context.security_level);
    println!("  Signature Algorithm: {}", high_context.algorithms.signature);
    println!("  Encryption Algorithm: {}", high_context.algorithms.encryption);
    println!("  Key Storage: {:?}", high_context.key_management.storage);
    println!("  Authentication: {:?}", high_context.access_control.authentication);
    
    // Add compliance requirements
    let mut compliance_context = McpSecurityContext::new(SecurityLevel::High);
    compliance_context.add_compliance(
        ComplianceStandard::Gdpr,
        "GDPR-25".to_string(),
        "Data protection by design and by default".to_string(),
    );
    compliance_context.add_compliance(
        ComplianceStandard::Iso27001,
        "A.9.2.1".to_string(),
        "User registration and de-registration".to_string(),
    );
    
    println!("\nCompliance Context:");
    println!("  GDPR compliant: {}", compliance_context.is_compliant(ComplianceStandard::Gdpr, "GDPR-25"));
    println!("  ISO27001 compliant: {}", compliance_context.is_compliant(ComplianceStandard::Iso27001, "A.9.2.1"));
    
    // Create a secure message envelope
    let encrypted_payload = McpEncryptedMessage {
        ciphertext: "encrypted_data_here".to_string(),
        algorithm: "AES-256-GCM".to_string(),
        iv: Some("initialization_vector".to_string()),
        key_id: "key_123".to_string(),
    };
    
    let signature = McpSignature {
        signer: "agent_1".to_string(),
        algorithm: "Ed25519".to_string(),
        signature: "signature_value".to_string(),
        signed_at: chrono::Utc::now(),
        nonce: "nonce_123".to_string(),
        key_id: "key_456".to_string(),
    };
    
    let mut envelope = McpSecureEnvelope::new(
        "agent_1".to_string(),
        "agent_2".to_string(),
        "data_request".to_string(),
        encrypted_payload,
        signature,
    );
    
    // Set expiration (1 hour from now)
    envelope.set_expiration(1);
    
    println!("\nSecure Message Envelope:");
    println!("  Message ID: {}", envelope.message_id);
    println!("  Sender: {}", envelope.sender);
    println!("  Recipient: {}", envelope.recipient);
    println!("  Message Type: {}", envelope.message_type);
    println!("  Sent At: {}", envelope.sent_at);
    println!("  Expires At: {:?}", envelope.expires_at);
    println!("  Is Expired: {}", envelope.is_expired());
    
    // Show audit logging configuration
    println!("\nAudit Logging Configuration:");
    for event in &high_context.audit_logging.events_to_log {
        println!("  Event: {:?}, Severity: {:?}", event.event_type, event.severity);
        println!("    Log Success: {}, Log Failure: {}", event.log_success, event.log_failure);
    }
    
    // Show access control roles
    println!("\nAccess Control Roles:");
    for role in &high_context.access_control.roles {
        println!("  Role: {} - {}", role.name, role.description);
        for permission in &role.permissions {
            println!("    Permission: {} -> {}", permission.resource, permission.action);
        }
    }
    
    // Serialize to JSON
    let json = serde_json::to_string_pretty(&high_context).unwrap();
    println!("\n=== MCP Context JSON Representation (first 500 chars) ===\n");
    println!("{}...", &json[..500.min(json.len())]);
}