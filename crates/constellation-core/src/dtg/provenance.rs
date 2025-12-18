//! Cryptographic provenance and audit trails for Data Transformation Graphs.
//!
//! Provides cryptographic signing and verification of DTG execution chains
//! for auditability, integrity, and non-repudiation.

use crate::mcp::crypto::{CryptoError, McpCrypto};
use crate::models::dtg::{
    CryptographicSignature, DataTransformationGraph, DtgProvenance, TransformationRecord,
};
use base64::Engine;
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use uuid::Uuid;

/// Provenance manager for DTG cryptographic audit trails.
#[derive(Debug)]
pub struct ProvenanceManager {
    /// Cryptographic operations.
    crypto: McpCrypto,

    /// Audit log storage.
    audit_logs: HashMap<Uuid, Vec<AuditLogEntry>>,

    /// Provenance records.
    provenance_records: HashMap<Uuid, DtgProvenance>,
}

/// Audit log entry for DTG operations.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditLogEntry {
    /// Operation type.
    pub operation: AuditOperation,

    /// DTG ID.
    pub dtg_id: Uuid,

    /// Agent ID (if applicable).
    pub agent_id: Option<String>,

    /// Node ID (if applicable).
    pub node_id: Option<Uuid>,

    /// Operation details.
    pub details: Value,

    /// Timestamp.
    pub timestamp: chrono::DateTime<chrono::Utc>,

    /// Cryptographic hash of the operation.
    pub operation_hash: String,

    /// Previous hash for chain integrity.
    pub previous_hash: Option<String>,
}

/// Audit operation types.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum AuditOperation {
    /// DTG creation.
    DtgCreated,

    /// Node added.
    NodeAdded,

    /// Edge added.
    EdgeAdded,

    /// Node execution started.
    NodeExecutionStarted,

    /// Node execution completed.
    NodeExecutionCompleted,

    /// Node execution failed.
    NodeExecutionFailed,

    /// Data transformation.
    DataTransformed,

    /// Provenance signed.
    ProvenanceSigned,

    /// Provenance verified.
    ProvenanceVerified,

    /// Audit trail accessed.
    AuditAccessed,

    /// Integrity check performed.
    IntegrityChecked,
}

impl ProvenanceManager {
    /// Create a new provenance manager.
    pub fn new() -> Result<Self, CryptoError> {
        let crypto = McpCrypto::new()?;

        Ok(Self {
            crypto,
            audit_logs: HashMap::new(),
            provenance_records: HashMap::new(),
        })
    }

    /// Create cryptographic provenance for a DTG.
    pub fn create_provenance(
        &mut self,
        dtg: &DataTransformationGraph,
        transformation_chain: Vec<TransformationRecord>,
        input_lineage: Vec<crate::models::dtg::DataLineage>,
        output_lineage: Vec<crate::models::dtg::DataLineage>,
        agent_records: Vec<crate::models::dtg::AgentExecutionRecord>,
    ) -> Result<DtgProvenance, ProvenanceError> {
        // Create the provenance structure
        let provenance = DtgProvenance {
            dtg_id: dtg.id,
            transformation_chain,
            input_lineage,
            output_lineage,
            agent_records,
            signatures: Vec::new(),
            recorded_at: chrono::Utc::now(),
        };

        // Generate hash for the provenance
        let provenance_hash = self.hash_provenance(&provenance)?;

        // Log the provenance creation
        self.log_audit(
            dtg.id,
            AuditOperation::DtgCreated,
            None,
            None,
            json!({
                "provenance_hash": provenance_hash,
                "node_count": dtg.nodes.len(),
                "edge_count": dtg.edges.len(),
            }),
        )?;

        // Store the provenance
        self.provenance_records.insert(dtg.id, provenance.clone());

        Ok(provenance)
    }

    /// Sign DTG provenance with a cryptographic signature.
    pub fn sign_provenance(
        &mut self,
        dtg_id: Uuid,
        signer_key_id: &str,
        signer: &str,
        algorithm: &str,
    ) -> Result<CryptographicSignature, ProvenanceError> {
        // Get the provenance
        let provenance = self
            .provenance_records
            .get(&dtg_id)
            .ok_or(ProvenanceError::ProvenanceNotFound(dtg_id))?;

        // Serialize the provenance for signing
        let provenance_json = serde_json::to_vec(provenance)
            .map_err(|e| ProvenanceError::SerializationError(e.to_string()))?;

        // Create signature using MCP crypto
        let mcp_signature =
            self.crypto
                .create_signature(signer_key_id, signer, algorithm, &provenance_json)?;

        // Convert to DTG cryptographic signature
        let signature = CryptographicSignature {
            signer: mcp_signature.signer,
            algorithm: mcp_signature.algorithm,
            signature: mcp_signature.signature,
            public_key: Some(mcp_signature.key_id),
            signed_at: mcp_signature.signed_at,
        };

        // Add signature to provenance
        if let Some(provenance) = self.provenance_records.get_mut(&dtg_id) {
            provenance.signatures.push(signature.clone());
        }

        // Log the signing operation
        self.log_audit(
            dtg_id,
            AuditOperation::ProvenanceSigned,
            Some(signer.to_string()),
            None,
            json!({
                "algorithm": algorithm,
                "key_id": signer_key_id,
            }),
        )?;

        Ok(signature)
    }

    /// Verify cryptographic signatures on DTG provenance.
    pub fn verify_provenance_signatures(
        &self,
        dtg_id: Uuid,
    ) -> Result<Vec<SignatureVerificationResult>, ProvenanceError> {
        let provenance = self
            .provenance_records
            .get(&dtg_id)
            .ok_or(ProvenanceError::ProvenanceNotFound(dtg_id))?;

        let mut results = Vec::new();

        for signature in &provenance.signatures {
            // Get the public key ID
            let public_key_id = signature.public_key.as_ref().ok_or_else(|| {
                ProvenanceError::VerificationError("No public key provided".to_string())
            })?;

            // Serialize provenance for verification
            let provenance_json = serde_json::to_vec(provenance)
                .map_err(|e| ProvenanceError::SerializationError(e.to_string()))?;

            // Decode signature
            let signature_bytes = base64::engine::general_purpose::STANDARD
                .decode(&signature.signature)
                .map_err(|e| ProvenanceError::VerificationError(e.to_string()))?;

            // Verify signature
            let is_valid = self
                .crypto
                .verify(public_key_id, &provenance_json, &signature_bytes)?;

            results.push(SignatureVerificationResult {
                signer: signature.signer.clone(),
                algorithm: signature.algorithm.clone(),
                is_valid,
                signed_at: signature.signed_at,
                public_key: signature.public_key.clone(),
            });
        }

        // Note: We can't log here because this is a read-only operation
        // Logging would require mutable access

        Ok(results)
    }

    /// Generate cryptographic hash for provenance integrity.
    pub fn hash_provenance(&self, provenance: &DtgProvenance) -> Result<String, ProvenanceError> {
        // Serialize the provenance
        let provenance_json = serde_json::to_vec(provenance)
            .map_err(|e| ProvenanceError::SerializationError(e.to_string()))?;

        // Compute SHA-256 hash
        let mut hasher = Sha256::new();
        hasher.update(&provenance_json);
        let hash = hasher.finalize();

        // Convert to hex string
        Ok(format!("{hash:x}"))
    }

    /// Verify provenance integrity using cryptographic hashes.
    pub fn verify_provenance_integrity(
        &self,
        dtg_id: Uuid,
    ) -> Result<IntegrityCheckResult, ProvenanceError> {
        let provenance = self
            .provenance_records
            .get(&dtg_id)
            .ok_or(ProvenanceError::ProvenanceNotFound(dtg_id))?;

        // Compute current hash
        let current_hash = self.hash_provenance(provenance)?;

        // Check transformation chain integrity
        let mut chain_integrity = true;
        let mut chain_errors = Vec::new();

        for (i, transformation) in provenance.transformation_chain.iter().enumerate() {
            if transformation.transformation_hash.is_empty() {
                chain_integrity = false;
                chain_errors.push(format!("Transformation {i} has no hash"));
            }
        }

        // Note: We can't log here because this is a read-only operation
        // Logging would require mutable access

        Ok(IntegrityCheckResult {
            dtg_id,
            current_hash,
            chain_integrity,
            chain_errors,
            signature_count: provenance.signatures.len(),
            transformation_count: provenance.transformation_chain.len(),
        })
    }

    /// Log an audit entry with cryptographic chain integrity.
    pub fn log_audit(
        &mut self,
        dtg_id: Uuid,
        operation: AuditOperation,
        agent_id: Option<String>,
        node_id: Option<Uuid>,
        details: Value,
    ) -> Result<(), ProvenanceError> {
        let timestamp = chrono::Utc::now();

        // Get previous hash for this DTG's audit chain
        let previous_hash = self
            .audit_logs
            .get(&dtg_id)
            .and_then(|logs| logs.last())
            .map(|entry| entry.operation_hash.clone());

        // Create audit entry
        let entry = AuditLogEntry {
            operation: operation.clone(),
            dtg_id,
            agent_id,
            node_id,
            details,
            timestamp,
            operation_hash: String::new(), // Will be set after hash computation
            previous_hash,
        };

        // Compute hash for the entry
        let entry_json = serde_json::to_vec(&entry)
            .map_err(|e| ProvenanceError::SerializationError(e.to_string()))?;
        let mut hasher = Sha256::new();
        hasher.update(&entry_json);
        let hash = format!("{:x}", hasher.finalize());

        // Update entry with hash
        let mut entry = entry;
        entry.operation_hash = hash;

        // Store in audit log
        self.audit_logs.entry(dtg_id).or_default().push(entry);

        Ok(())
    }

    /// Get audit trail for a DTG.
    pub fn get_audit_trail(&self, dtg_id: Uuid) -> Result<&[AuditLogEntry], ProvenanceError> {
        self.audit_logs
            .get(&dtg_id)
            .map(|logs| logs.as_slice())
            .ok_or(ProvenanceError::ProvenanceNotFound(dtg_id))
    }

    /// Verify audit trail integrity.
    pub fn verify_audit_trail_integrity(
        &self,
        dtg_id: Uuid,
    ) -> Result<AuditTrailIntegrityResult, ProvenanceError> {
        let logs = self
            .audit_logs
            .get(&dtg_id)
            .ok_or(ProvenanceError::ProvenanceNotFound(dtg_id))?;

        let mut integrity = true;
        let mut errors = Vec::new();

        // Check chain integrity
        for i in 1..logs.len() {
            let current = &logs[i];
            let previous = &logs[i - 1];

            if current.previous_hash != Some(previous.operation_hash.clone()) {
                integrity = false;
                errors.push(format!(
                    "Audit chain broken at entry {i}: previous hash mismatch"
                ));
            }

            // Verify entry hash
            let mut entry_for_hash = current.clone();
            entry_for_hash.operation_hash = String::new();
            let entry_json = serde_json::to_vec(&entry_for_hash)
                .map_err(|e| ProvenanceError::SerializationError(e.to_string()))?;
            let mut hasher = Sha256::new();
            hasher.update(&entry_json);
            let computed_hash = format!("{:x}", hasher.finalize());

            if computed_hash != current.operation_hash {
                integrity = false;
                errors.push(format!("Entry {i} hash mismatch"));
            }
        }

        Ok(AuditTrailIntegrityResult {
            dtg_id,
            entry_count: logs.len(),
            integrity,
            errors,
            first_timestamp: logs.first().map(|e| e.timestamp),
            last_timestamp: logs.last().map(|e| e.timestamp),
        })
    }

    /// Get provenance record.
    pub fn get_provenance(&self, dtg_id: Uuid) -> Option<&DtgProvenance> {
        self.provenance_records.get(&dtg_id)
    }

    /// Get all provenance records.
    pub fn get_all_provenance(&self) -> Vec<&DtgProvenance> {
        self.provenance_records.values().collect()
    }
}

/// Signature verification result.
#[derive(Debug, Clone)]
pub struct SignatureVerificationResult {
    /// Signer identifier.
    pub signer: String,

    /// Signature algorithm.
    pub algorithm: String,

    /// Whether the signature is valid.
    pub is_valid: bool,

    /// When the signature was created.
    pub signed_at: chrono::DateTime<chrono::Utc>,

    /// Public key or certificate.
    pub public_key: Option<String>,
}

/// Integrity check result.
#[derive(Debug, Clone)]
pub struct IntegrityCheckResult {
    /// DTG ID.
    pub dtg_id: Uuid,

    /// Current cryptographic hash.
    pub current_hash: String,

    /// Whether the transformation chain is intact.
    pub chain_integrity: bool,

    /// Chain integrity errors.
    pub chain_errors: Vec<String>,

    /// Number of cryptographic signatures.
    pub signature_count: usize,

    /// Number of transformations.
    pub transformation_count: usize,
}

/// Audit trail integrity result.
#[derive(Debug, Clone)]
pub struct AuditTrailIntegrityResult {
    /// DTG ID.
    pub dtg_id: Uuid,

    /// Number of audit entries.
    pub entry_count: usize,

    /// Whether the audit trail is intact.
    pub integrity: bool,

    /// Integrity errors.
    pub errors: Vec<String>,

    /// First entry timestamp.
    pub first_timestamp: Option<chrono::DateTime<chrono::Utc>>,

    /// Last entry timestamp.
    pub last_timestamp: Option<chrono::DateTime<chrono::Utc>>,
}

/// Provenance error.
#[derive(Debug, thiserror::Error)]
pub enum ProvenanceError {
    /// Cryptographic error.
    #[error("Cryptographic error: {0}")]
    CryptoError(#[from] CryptoError),

    /// Provenance not found.
    #[error("Provenance not found for DTG: {0}")]
    ProvenanceNotFound(Uuid),

    /// Serialization error.
    #[error("Serialization error: {0}")]
    SerializationError(String),

    /// Verification error.
    #[error("Verification error: {0}")]
    VerificationError(String),

    /// Audit trail error.
    #[error("Audit trail error: {0}")]
    AuditTrailError(String),
}

impl Default for ProvenanceManager {
    fn default() -> Self {
        Self::new().expect("Failed to create provenance manager")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::dtg::{DtgDataRef, DtgMetrics, DtgNode, DtgNodeStatus};
    use serde_json::json;

    #[test]
    fn test_provenance_creation() -> Result<(), ProvenanceError> {
        let mut manager = ProvenanceManager::new()?;

        // Create a simple DTG
        let dtg = DataTransformationGraph {
            id: Uuid::new_v4(),
            name: "Test DTG".to_string(),
            root_nodes: vec![],
            nodes: HashMap::new(),
            edges: vec![],
            graph_inputs: vec![],
            graph_outputs: vec![],
            metadata: HashMap::new(),
            started_at: chrono::Utc::now(),
            completed_at: None,
            status: crate::models::dtg::DtgGraphStatus::Constructing,
            tags: vec![],
        };

        // Create provenance
        let provenance = manager.create_provenance(&dtg, vec![], vec![], vec![], vec![])?;

        assert_eq!(provenance.dtg_id, dtg.id);
        assert!(provenance.signatures.is_empty());

        Ok(())
    }

    #[test]
    fn test_provenance_hashing() -> Result<(), ProvenanceError> {
        let manager = ProvenanceManager::new()?;

        let provenance = DtgProvenance {
            dtg_id: Uuid::new_v4(),
            transformation_chain: vec![],
            input_lineage: vec![],
            output_lineage: vec![],
            agent_records: vec![],
            signatures: vec![],
            recorded_at: chrono::Utc::now(),
        };

        let hash = manager.hash_provenance(&provenance)?;
        assert!(!hash.is_empty());
        assert_eq!(hash.len(), 64); // SHA-256 hex string length

        Ok(())
    }

    #[test]
    fn test_audit_logging() -> Result<(), ProvenanceError> {
        let mut manager = ProvenanceManager::new()?;
        let dtg_id = Uuid::new_v4();

        // Log some audit entries
        manager.log_audit(
            dtg_id,
            AuditOperation::DtgCreated,
            Some("system".to_string()),
            None,
            json!({"action": "create"}),
        )?;

        manager.log_audit(
            dtg_id,
            AuditOperation::NodeAdded,
            Some("user1".to_string()),
            Some(Uuid::new_v4()),
            json!({"node_name": "test_node"}),
        )?;

        // Get audit trail
        let trail = manager.get_audit_trail(dtg_id)?;
        assert_eq!(trail.len(), 2);
        assert_eq!(trail[0].operation, AuditOperation::DtgCreated);
        assert_eq!(trail[1].operation, AuditOperation::NodeAdded);

        // Verify integrity
        let integrity = manager.verify_audit_trail_integrity(dtg_id)?;
        assert!(integrity.integrity);
        assert_eq!(integrity.entry_count, 2);

        Ok(())
    }
}
