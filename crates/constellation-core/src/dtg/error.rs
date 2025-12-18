//! DTG error types and utilities.

use thiserror::Error;
use uuid::Uuid;

/// DTG execution and validation errors.
#[derive(Debug, Error)]
pub enum DtgError {
    /// Graph contains cycles.
    #[error("DTG contains cycles: {0}")]
    CyclicGraph(String),

    /// Node not found.
    #[error("Node not found: {0}")]
    NodeNotFound(Uuid),

    /// Edge not found.
    #[error("Edge not found: {0}")]
    EdgeNotFound(Uuid),

    /// Invalid edge references.
    #[error("Invalid edge references: source={0}, target={1}")]
    InvalidEdgeReferences(Uuid, Uuid),

    /// Duplicate node IDs.
    #[error("Duplicate node IDs detected")]
    DuplicateNodeIds,

    /// Orphaned node.
    #[error("Orphaned node: {0}")]
    OrphanedNode(Uuid),

    /// Execution failed.
    #[error("Execution failed: {0}")]
    ExecutionFailed(String),

    /// Node execution failed.
    #[error("Node execution failed: {0}")]
    NodeExecutionFailed(Uuid, String),

    /// Validation errors.
    #[error("Validation errors: {0:?}")]
    ValidationErrors(Vec<String>),

    /// Graph not ready for execution.
    #[error("Graph not ready for execution: {0}")]
    GraphNotReady(String),

    /// Timeout during execution.
    #[error("Execution timeout")]
    Timeout,

    /// Resource allocation failed.
    #[error("Resource allocation failed: {0}")]
    ResourceAllocationFailed(String),

    /// Provenance error.
    #[error("Provenance error: {0}")]
    ProvenanceError(#[from] crate::dtg::provenance::ProvenanceError),

    /// Serialization error.
    #[error("Serialization error: {0}")]
    SerializationError(#[from] serde_json::Error),

    /// IO error.
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
}

/// Result type for DTG operations.
pub type DtgResult<T> = Result<T, DtgError>;

/// Utility to convert string errors to DtgError.
pub fn to_dtg_error<T: ToString>(error: T) -> DtgError {
    DtgError::ExecutionFailed(error.to_string())
}

/// Utility to convert validation errors.
pub fn validation_errors(errors: Vec<String>) -> DtgError {
    DtgError::ValidationErrors(errors)
}
