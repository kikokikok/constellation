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

#[cfg(test)]
mod tests {
    use super::*;
    use uuid::Uuid;

    #[test]
    fn test_dtg_error_variants() {
        let node_id = Uuid::new_v4();
        let edge_id = Uuid::new_v4();

        // Test CyclicGraph
        let cyclic_error = DtgError::CyclicGraph("Test cycle".to_string());
        assert!(format!("{:?}", cyclic_error).contains("CyclicGraph"));
        assert!(format!("{}", cyclic_error).contains("DTG contains cycles"));

        // Test NodeNotFound
        let node_error = DtgError::NodeNotFound(node_id);
        assert!(format!("{:?}", node_error).contains("NodeNotFound"));
        assert!(format!("{}", node_error).contains(&node_id.to_string()));

        // Test EdgeNotFound
        let edge_error = DtgError::EdgeNotFound(edge_id);
        assert!(format!("{:?}", edge_error).contains("EdgeNotFound"));
        assert!(format!("{}", edge_error).contains(&edge_id.to_string()));

        // Test InvalidEdgeReferences
        let invalid_edge_error = DtgError::InvalidEdgeReferences(node_id, edge_id);
        assert!(format!("{:?}", invalid_edge_error).contains("InvalidEdgeReferences"));
        assert!(format!("{}", invalid_edge_error).contains(&node_id.to_string()));
        assert!(format!("{}", invalid_edge_error).contains(&edge_id.to_string()));

        // Test DuplicateNodeIds
        let duplicate_error = DtgError::DuplicateNodeIds;
        assert!(format!("{:?}", duplicate_error).contains("DuplicateNodeIds"));
        assert!(format!("{}", duplicate_error).contains("Duplicate node IDs"));

        // Test OrphanedNode
        let orphaned_error = DtgError::OrphanedNode(node_id);
        assert!(format!("{:?}", orphaned_error).contains("OrphanedNode"));
        assert!(format!("{}", orphaned_error).contains(&node_id.to_string()));

        // Test ExecutionFailed
        let exec_error = DtgError::ExecutionFailed("Test failure".to_string());
        assert!(format!("{:?}", exec_error).contains("ExecutionFailed"));
        assert!(format!("{}", exec_error).contains("Test failure"));

        // Test NodeExecutionFailed
        let node_exec_error = DtgError::NodeExecutionFailed(node_id, "Node failed".to_string());
        assert!(format!("{:?}", node_exec_error).contains("NodeExecutionFailed"));
        let display_str = format!("{}", node_exec_error);
        // The display format is "Node execution failed: {node_id}" (only shows UUID)
        // based on the #[error] annotation: "Node execution failed: {0}"
        assert!(display_str.contains(&node_id.to_string()));
        // Note: The error message "Node failed" is not in the display string
        // because the #[error] annotation only includes {0} (the UUID)

        // Test ValidationErrors
        let validation_error =
            DtgError::ValidationErrors(vec!["Error 1".to_string(), "Error 2".to_string()]);
        assert!(format!("{:?}", validation_error).contains("ValidationErrors"));
        assert!(format!("{}", validation_error).contains("Error 1"));

        // Test GraphNotReady
        let not_ready_error = DtgError::GraphNotReady("Not ready".to_string());
        assert!(format!("{:?}", not_ready_error).contains("GraphNotReady"));
        assert!(format!("{}", not_ready_error).contains("Not ready"));

        // Test Timeout
        let timeout_error = DtgError::Timeout;
        assert!(format!("{:?}", timeout_error).contains("Timeout"));
        assert!(format!("{}", timeout_error).contains("Execution timeout"));

        // Test ResourceAllocationFailed
        let resource_error = DtgError::ResourceAllocationFailed("No memory".to_string());
        assert!(format!("{:?}", resource_error).contains("ResourceAllocationFailed"));
        assert!(format!("{}", resource_error).contains("No memory"));
    }

    #[test]
    fn test_to_dtg_error() {
        let error = to_dtg_error("Test error message");

        match error {
            DtgError::ExecutionFailed(msg) => {
                assert_eq!(msg, "Test error message");
            }
            _ => panic!("Expected ExecutionFailed variant"),
        }

        // Test with different types that implement ToString
        let error_int = to_dtg_error(42);
        match error_int {
            DtgError::ExecutionFailed(msg) => {
                assert_eq!(msg, "42");
            }
            _ => panic!("Expected ExecutionFailed variant"),
        }

        let error_string = to_dtg_error(String::from("String error"));
        match error_string {
            DtgError::ExecutionFailed(msg) => {
                assert_eq!(msg, "String error");
            }
            _ => panic!("Expected ExecutionFailed variant"),
        }
    }

    #[test]
    fn test_validation_errors() {
        let errors = vec![
            "Error 1".to_string(),
            "Error 2".to_string(),
            "Error 3".to_string(),
        ];

        let validation_error = validation_errors(errors.clone());

        match validation_error {
            DtgError::ValidationErrors(errs) => {
                assert_eq!(errs.len(), 3);
                assert_eq!(errs[0], "Error 1");
                assert_eq!(errs[1], "Error 2");
                assert_eq!(errs[2], "Error 3");
            }
            _ => panic!("Expected ValidationErrors variant"),
        }
    }

    #[test]
    fn test_dtg_result_type() {
        // Test successful result
        let success: DtgResult<i32> = Ok(42);
        assert!(success.is_ok());
        assert_eq!(success.unwrap(), 42);

        // Test error result
        let error: DtgResult<i32> = Err(DtgError::ExecutionFailed("Failed".to_string()));
        assert!(error.is_err());

        match error {
            Err(DtgError::ExecutionFailed(msg)) => {
                assert_eq!(msg, "Failed");
            }
            _ => panic!("Expected ExecutionFailed error"),
        }
    }

    #[test]
    fn test_error_conversions() {
        // Test serde_json error conversion
        let json_error = serde_json::from_str::<i32>("invalid").unwrap_err();
        let dtg_error: DtgError = json_error.into();

        match dtg_error {
            DtgError::SerializationError(_) => {
                // Successfully converted
            }
            _ => panic!("Expected SerializationError variant"),
        }

        // Test std::io error conversion
        let io_error = std::io::Error::new(std::io::ErrorKind::NotFound, "File not found");
        let dtg_error: DtgError = io_error.into();

        match dtg_error {
            DtgError::IoError(_) => {
                // Successfully converted
            }
            _ => panic!("Expected IoError variant"),
        }
    }
}
