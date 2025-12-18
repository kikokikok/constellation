//! Data Transformation Graph (DTG) module.
//!
//! Provides models and execution engine for tracking agent skill execution
//! as data transformations with full provenance tracking.

pub mod engine;
pub mod error;
pub mod metrics;
pub mod provenance;
pub mod visualization;

pub use engine::{DtgExecutionEngine, ExecutionStats};
pub use error::{DtgError, DtgResult, to_dtg_error, validation_errors};
pub use metrics::{
    DtgMetricsCollector, PerformanceAnalysis, PerformanceStatus, QualityReport,
    QualityScoringConfig,
};
pub use provenance::{
    AuditOperation, AuditTrailIntegrityResult, IntegrityCheckResult, ProvenanceError,
    ProvenanceManager, SignatureVerificationResult,
};
pub use visualization::{DtgVisualizationEngine, GraphAnalysis, VisualizationFormat};
