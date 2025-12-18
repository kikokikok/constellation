//! Model Context Protocol (MCP) module.
//!
//! Provides cryptographic security for agent communications with
//! signing, verification, encryption, and decryption capabilities.

pub mod compliance;
pub mod crypto;
pub mod key_management;
pub mod threat_detection;

pub use compliance::{
    AccessControlRequirements, AuthStrength, ComplianceCheckResult, ComplianceFramework,
    ComplianceIssue, ComplianceManager, ComplianceRequirement, DataClassification,
    DataRetentionPolicy, DeletionMethod, IssueSeverity, PrivacyImpactAssessment, RiskLevel,
    VerificationMethod,
};
pub use crypto::{
    CryptoError, KeyMetadata, KeyStore, KeyUsage, McpCrypto, McpPublicKey, PrivateKey,
};
pub use key_management::{
    DeactivationReason, KeyExport, KeyManager, KeyRotationPolicy, KeyUsageStats, KeyVersion,
    RotationNeeded, RotationReason, RotationStrategy, RotationUrgency,
};
pub use threat_detection::{
    DetectedThreat, DetectionPattern, DetectionRule, FeedType, IndicatorOfCompromise,
    IndicatorType, MitigationStatus, ResponseAction, ThreatDetectionEngine, ThreatIntelligence,
    ThreatSeverity, ThreatSource, ThreatType,
};
