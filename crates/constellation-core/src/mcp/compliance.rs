//! Compliance framework support for MCP security.
//!
//! Provides compliance checking and reporting for regulations like GDPR, HIPAA,
//! CCPA, and other data protection standards.

use crate::mcp::crypto::KeyStore;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

/// Compliance framework.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ComplianceFramework {
    /// General Data Protection Regulation (EU).
    Gdpr,

    /// Health Insurance Portability and Accountability Act (US).
    Hipaa,

    /// California Consumer Privacy Act (US).
    Ccpa,

    /// Payment Card Industry Data Security Standard.
    PciDss,

    /// ISO 27001 Information Security Management.
    Iso27001,

    /// Federal Risk and Authorization Management Program (US).
    FedRamp,

    /// Custom compliance framework.
    Custom(String),
}

/// Data classification level.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum DataClassification {
    /// Public information.
    Public,

    /// Internal use only.
    Internal,

    /// Confidential information.
    Confidential,

    /// Restricted/Highly confidential.
    Restricted,

    /// Personal Identifiable Information.
    Pii,

    /// Protected Health Information.
    Phi,

    /// Payment card information.
    Pci,
}

/// Compliance requirement.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComplianceRequirement {
    /// Requirement ID.
    pub id: String,

    /// Framework this requirement belongs to.
    pub framework: ComplianceFramework,

    /// Requirement name.
    pub name: String,

    /// Requirement description.
    pub description: String,

    /// Whether this requirement is mandatory.
    pub mandatory: bool,

    /// Data classifications this applies to.
    pub applies_to: Vec<DataClassification>,

    /// Implementation guidance.
    pub guidance: String,

    /// Verification method.
    pub verification: VerificationMethod,
}

/// Verification method for compliance.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum VerificationMethod {
    /// Automated check.
    Automated,

    /// Manual review.
    Manual,

    /// Audit trail.
    AuditTrail,

    /// Documentation.
    Documentation,

    /// Combination of methods.
    Combined(Vec<VerificationMethod>),
}

/// Compliance check result.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComplianceCheckResult {
    /// Requirement ID.
    pub requirement_id: String,

    /// Whether requirement is satisfied.
    pub satisfied: bool,

    /// Evidence of compliance.
    pub evidence: String,

    /// Timestamp of check.
    pub checked_at: DateTime<Utc>,

    /// Any issues found.
    pub issues: Vec<ComplianceIssue>,

    /// Recommendations for improvement.
    pub recommendations: Vec<String>,
}

/// Compliance issue.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComplianceIssue {
    /// Issue ID.
    pub id: String,

    /// Severity level.
    pub severity: IssueSeverity,

    /// Issue description.
    pub description: String,

    /// Affected component.
    pub component: String,

    /// Remediation steps.
    pub remediation: String,

    /// Deadline for fixing.
    pub deadline: Option<DateTime<Utc>>,
}

/// Issue severity level.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum IssueSeverity {
    /// Critical - immediate action required.
    Critical,

    /// High - action required soon.
    High,

    /// Medium - should be addressed.
    Medium,

    /// Low - nice to have.
    Low,

    /// Informational - no action required.
    Informational,
}

/// Data retention policy.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataRetentionPolicy {
    /// Policy ID.
    pub id: String,

    /// Data classification.
    pub classification: DataClassification,

    /// Retention period in days.
    pub retention_days: u32,

    /// Whether data should be encrypted at rest.
    pub encrypt_at_rest: bool,

    /// Whether data should be encrypted in transit.
    pub encrypt_in_transit: bool,

    /// Access control requirements.
    pub access_control: AccessControlRequirements,

    /// Audit logging requirements.
    pub audit_logging: bool,

    /// Data deletion method.
    pub deletion_method: DeletionMethod,
}

/// Access control requirements.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccessControlRequirements {
    /// Whether authentication is required.
    pub authentication_required: bool,

    /// Whether authorization is required.
    pub authorization_required: bool,

    /// Minimum authentication strength.
    pub min_auth_strength: AuthStrength,

    /// Whether multi-factor authentication is required.
    pub mfa_required: bool,

    /// Access review frequency in days.
    pub access_review_frequency_days: u32,
}

/// Authentication strength.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum AuthStrength {
    /// Password only.
    Password,

    /// Multi-factor authentication.
    Mfa,

    /// Certificate-based.
    Certificate,

    /// Hardware token.
    HardwareToken,

    /// Biometric.
    Biometric,
}

/// Data deletion method.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum DeletionMethod {
    /// Logical deletion (mark as deleted).
    Logical,

    /// Physical deletion (remove from storage).
    Physical,

    /// Cryptographic erasure (delete encryption keys).
    Cryptographic,

    /// Secure deletion (multiple overwrites).
    Secure,
}

/// Privacy impact assessment.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrivacyImpactAssessment {
    /// Assessment ID.
    pub id: String,

    /// System or process being assessed.
    pub system_name: String,

    /// Data types processed.
    pub data_types: Vec<DataClassification>,

    /// Data processing purposes.
    pub purposes: Vec<String>,

    /// Data retention period.
    pub retention_period_days: u32,

    /// Data sharing with third parties.
    pub third_party_sharing: bool,

    /// Risk level.
    pub risk_level: RiskLevel,

    /// Mitigation measures.
    pub mitigations: Vec<String>,

    /// Assessment date.
    pub assessment_date: DateTime<Utc>,

    /// Next review date.
    pub next_review_date: DateTime<Utc>,
}

/// Risk level.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum RiskLevel {
    /// Low risk.
    Low,

    /// Medium risk.
    Medium,

    /// High risk.
    High,

    /// Critical risk.
    Critical,
}

/// Compliance manager.
#[derive(Debug)]
pub struct ComplianceManager {
    /// Active compliance frameworks.
    frameworks: Vec<ComplianceFramework>,

    /// Compliance requirements.
    requirements: HashMap<String, ComplianceRequirement>,

    /// Data retention policies.
    retention_policies: HashMap<DataClassification, DataRetentionPolicy>,

    /// Privacy impact assessments.
    pia_assessments: HashMap<String, PrivacyImpactAssessment>,

    /// Compliance check history.
    check_history: Vec<ComplianceCheckResult>,
}

impl ComplianceManager {
    /// Create a new compliance manager.
    pub fn new(frameworks: Vec<ComplianceFramework>) -> Self {
        let mut manager = Self {
            frameworks,
            requirements: HashMap::new(),
            retention_policies: HashMap::new(),
            pia_assessments: HashMap::new(),
            check_history: Vec::new(),
        };

        // Load default requirements for each framework
        manager.load_default_requirements();

        // Load default retention policies
        manager.load_default_retention_policies();

        manager
    }

    /// Load default compliance requirements.
    fn load_default_requirements(&mut self) {
        // GDPR requirements
        if self.frameworks.contains(&ComplianceFramework::Gdpr) {
            self.requirements.insert(
                "GDPR-ART-5".to_string(),
                ComplianceRequirement {
                    id: "GDPR-ART-5".to_string(),
                    framework: ComplianceFramework::Gdpr,
                    name: "Principles relating to processing of personal data".to_string(),
                    description: "Personal data shall be processed lawfully, fairly and in a transparent manner.".to_string(),
                    mandatory: true,
                    applies_to: vec![DataClassification::Pii],
                    guidance: "Implement data processing agreements, privacy notices, and consent mechanisms.".to_string(),
                    verification: VerificationMethod::Combined(vec![
                        VerificationMethod::Documentation,
                        VerificationMethod::AuditTrail,
                    ]),
                },
            );

            self.requirements.insert(
                "GDPR-ART-17".to_string(),
                ComplianceRequirement {
                    id: "GDPR-ART-17".to_string(),
                    framework: ComplianceFramework::Gdpr,
                    name: "Right to erasure ('right to be forgotten')".to_string(),
                    description: "Data subjects have the right to have their personal data erased.".to_string(),
                    mandatory: true,
                    applies_to: vec![DataClassification::Pii],
                    guidance: "Implement data deletion procedures and honor deletion requests within 30 days.".to_string(),
                    verification: VerificationMethod::Automated,
                },
            );

            self.requirements.insert(
                "GDPR-ART-25".to_string(),
                ComplianceRequirement {
                    id: "GDPR-ART-25".to_string(),
                    framework: ComplianceFramework::Gdpr,
                    name: "Data protection by design and by default".to_string(),
                    description: "Implement appropriate technical and organizational measures to ensure data protection.".to_string(),
                    mandatory: true,
                    applies_to: vec![DataClassification::Pii],
                    guidance: "Use encryption, access controls, and data minimization principles.".to_string(),
                    verification: VerificationMethod::Combined(vec![
                        VerificationMethod::Automated,
                        VerificationMethod::Documentation,
                    ]),
                },
            );

            self.requirements.insert(
                "GDPR-ART-32".to_string(),
                ComplianceRequirement {
                    id: "GDPR-ART-32".to_string(),
                    framework: ComplianceFramework::Gdpr,
                    name: "Security of processing".to_string(),
                    description: "Implement appropriate security measures for personal data."
                        .to_string(),
                    mandatory: true,
                    applies_to: vec![DataClassification::Pii],
                    guidance: "Use encryption, pseudonymization, and regular security testing."
                        .to_string(),
                    verification: VerificationMethod::Automated,
                },
            );
        }

        // HIPAA requirements
        if self.frameworks.contains(&ComplianceFramework::Hipaa) {
            self.requirements.insert(
                "HIPAA-164-308".to_string(),
                ComplianceRequirement {
                    id: "HIPAA-164-308".to_string(),
                    framework: ComplianceFramework::Hipaa,
                    name: "Administrative safeguards".to_string(),
                    description: "Implement administrative procedures to protect electronic protected health information.".to_string(),
                    mandatory: true,
                    applies_to: vec![DataClassification::Phi],
                    guidance: "Establish security management processes, workforce security, and information access management.".to_string(),
                    verification: VerificationMethod::Documentation,
                },
            );

            self.requirements.insert(
                "HIPAA-164-312".to_string(),
                ComplianceRequirement {
                    id: "HIPAA-164-312".to_string(),
                    framework: ComplianceFramework::Hipaa,
                    name: "Technical safeguards".to_string(),
                    description: "Implement technical policies and procedures for electronic information systems.".to_string(),
                    mandatory: true,
                    applies_to: vec![DataClassification::Phi],
                    guidance: "Use access controls, audit controls, integrity controls, and transmission security.".to_string(),
                    verification: VerificationMethod::Automated,
                },
            );

            self.requirements.insert(
                "HIPAA-164-314".to_string(),
                ComplianceRequirement {
                    id: "HIPAA-164-314".to_string(),
                    framework: ComplianceFramework::Hipaa,
                    name: "Organizational requirements".to_string(),
                    description: "Business associate contracts and other arrangements.".to_string(),
                    mandatory: true,
                    applies_to: vec![DataClassification::Phi],
                    guidance:
                        "Establish business associate agreements and chain of trust arrangements."
                            .to_string(),
                    verification: VerificationMethod::Documentation,
                },
            );
        }

        // CCPA requirements
        if self.frameworks.contains(&ComplianceFramework::Ccpa) {
            self.requirements.insert(
                "CCPA-1798-100".to_string(),
                ComplianceRequirement {
                    id: "CCPA-1798-100".to_string(),
                    framework: ComplianceFramework::Ccpa,
                    name: "Consumer rights".to_string(),
                    description:
                        "Right to know, delete, and opt-out of sale of personal information."
                            .to_string(),
                    mandatory: true,
                    applies_to: vec![DataClassification::Pii],
                    guidance:
                        "Implement mechanisms for consumer requests and honor them within 45 days."
                            .to_string(),
                    verification: VerificationMethod::Automated,
                },
            );

            self.requirements.insert(
                "CCPA-1798-135".to_string(),
                ComplianceRequirement {
                    id: "CCPA-1798-135".to_string(),
                    framework: ComplianceFramework::Ccpa,
                    name: "Opt-out of sale".to_string(),
                    description: "Provide clear and conspicuous opt-out mechanisms.".to_string(),
                    mandatory: true,
                    applies_to: vec![DataClassification::Pii],
                    guidance:
                        "Implement 'Do Not Sell My Personal Information' link and honor opt-outs."
                            .to_string(),
                    verification: VerificationMethod::Automated,
                },
            );
        }

        // PCI DSS requirements
        if self.frameworks.contains(&ComplianceFramework::PciDss) {
            self.requirements.insert(
                "PCI-DSS-3".to_string(),
                ComplianceRequirement {
                    id: "PCI-DSS-3".to_string(),
                    framework: ComplianceFramework::PciDss,
                    name: "Protect stored cardholder data".to_string(),
                    description:
                        "Implement strong encryption and access controls for cardholder data."
                            .to_string(),
                    mandatory: true,
                    applies_to: vec![DataClassification::Pci],
                    guidance:
                        "Use strong cryptography, key management, and mask PAN when displayed."
                            .to_string(),
                    verification: VerificationMethod::Automated,
                },
            );

            self.requirements.insert(
                "PCI-DSS-4".to_string(),
                ComplianceRequirement {
                    id: "PCI-DSS-4".to_string(),
                    framework: ComplianceFramework::PciDss,
                    name: "Encrypt transmission of cardholder data".to_string(),
                    description: "Encrypt cardholder data during transmission over open networks."
                        .to_string(),
                    mandatory: true,
                    applies_to: vec![DataClassification::Pci],
                    guidance: "Use strong cryptography (TLS 1.2+) for all transmissions."
                        .to_string(),
                    verification: VerificationMethod::Automated,
                },
            );
        }
    }

    /// Load default data retention policies.
    fn load_default_retention_policies(&mut self) {
        // Public data
        self.retention_policies.insert(
            DataClassification::Public,
            DataRetentionPolicy {
                id: "PUBLIC-001".to_string(),
                classification: DataClassification::Public,
                retention_days: 3650, // 10 years
                encrypt_at_rest: false,
                encrypt_in_transit: true,
                access_control: AccessControlRequirements {
                    authentication_required: false,
                    authorization_required: false,
                    min_auth_strength: AuthStrength::Password,
                    mfa_required: false,
                    access_review_frequency_days: 365,
                },
                audit_logging: true,
                deletion_method: DeletionMethod::Logical,
            },
        );

        // Internal data
        self.retention_policies.insert(
            DataClassification::Internal,
            DataRetentionPolicy {
                id: "INTERNAL-001".to_string(),
                classification: DataClassification::Internal,
                retention_days: 1825, // 5 years
                encrypt_at_rest: true,
                encrypt_in_transit: true,
                access_control: AccessControlRequirements {
                    authentication_required: true,
                    authorization_required: true,
                    min_auth_strength: AuthStrength::Password,
                    mfa_required: false,
                    access_review_frequency_days: 180,
                },
                audit_logging: true,
                deletion_method: DeletionMethod::Physical,
            },
        );

        // Confidential data
        self.retention_policies.insert(
            DataClassification::Confidential,
            DataRetentionPolicy {
                id: "CONFIDENTIAL-001".to_string(),
                classification: DataClassification::Confidential,
                retention_days: 1095, // 3 years
                encrypt_at_rest: true,
                encrypt_in_transit: true,
                access_control: AccessControlRequirements {
                    authentication_required: true,
                    authorization_required: true,
                    min_auth_strength: AuthStrength::Mfa,
                    mfa_required: true,
                    access_review_frequency_days: 90,
                },
                audit_logging: true,
                deletion_method: DeletionMethod::Secure,
            },
        );

        // PII data
        self.retention_policies.insert(
            DataClassification::Pii,
            DataRetentionPolicy {
                id: "PII-001".to_string(),
                classification: DataClassification::Pii,
                retention_days: 730, // 2 years
                encrypt_at_rest: true,
                encrypt_in_transit: true,
                access_control: AccessControlRequirements {
                    authentication_required: true,
                    authorization_required: true,
                    min_auth_strength: AuthStrength::Mfa,
                    mfa_required: true,
                    access_review_frequency_days: 60,
                },
                audit_logging: true,
                deletion_method: DeletionMethod::Cryptographic,
            },
        );

        // PHI data
        self.retention_policies.insert(
            DataClassification::Phi,
            DataRetentionPolicy {
                id: "PHI-001".to_string(),
                classification: DataClassification::Phi,
                retention_days: 2190, // 6 years (HIPAA requirement)
                encrypt_at_rest: true,
                encrypt_in_transit: true,
                access_control: AccessControlRequirements {
                    authentication_required: true,
                    authorization_required: true,
                    min_auth_strength: AuthStrength::Mfa,
                    mfa_required: true,
                    access_review_frequency_days: 30,
                },
                audit_logging: true,
                deletion_method: DeletionMethod::Cryptographic,
            },
        );

        // PCI data
        self.retention_policies.insert(
            DataClassification::Pci,
            DataRetentionPolicy {
                id: "PCI-001".to_string(),
                classification: DataClassification::Pci,
                retention_days: 365, // 1 year (PCI DSS requirement)
                encrypt_at_rest: true,
                encrypt_in_transit: true,
                access_control: AccessControlRequirements {
                    authentication_required: true,
                    authorization_required: true,
                    min_auth_strength: AuthStrength::Mfa,
                    mfa_required: true,
                    access_review_frequency_days: 30,
                },
                audit_logging: true,
                deletion_method: DeletionMethod::Cryptographic,
            },
        );
    }

    /// Check compliance for a specific requirement.
    pub fn check_requirement(
        &mut self,
        requirement_id: &str,
        key_store: &KeyStore,
        data_classification: &DataClassification,
    ) -> ComplianceCheckResult {
        let requirement = self
            .requirements
            .get(requirement_id)
            .cloned()
            .unwrap_or_else(|| {
                // Create a placeholder requirement if not found
                ComplianceRequirement {
                    id: requirement_id.to_string(),
                    framework: ComplianceFramework::Custom("Unknown".to_string()),
                    name: "Unknown requirement".to_string(),
                    description: "Requirement not found in registry".to_string(),
                    mandatory: false,
                    applies_to: vec![],
                    guidance: "".to_string(),
                    verification: VerificationMethod::Manual,
                }
            });

        let mut issues = Vec::new();
        let mut recommendations = Vec::new();
        let mut satisfied = true;

        // Check if requirement applies to this data classification
        if !requirement.applies_to.is_empty()
            && !requirement.applies_to.contains(data_classification)
        {
            issues.push(ComplianceIssue {
                id: Uuid::new_v4().to_string(),
                severity: IssueSeverity::Informational,
                description: format!("Requirement does not apply to {data_classification:?} data"),
                component: "Compliance Check".to_string(),
                remediation: "No action required".to_string(),
                deadline: None,
            });
        }

        // Perform framework-specific checks
        match requirement.framework {
            ComplianceFramework::Gdpr => {
                satisfied &= self.check_gdpr_requirement(
                    &requirement,
                    key_store,
                    &mut issues,
                    &mut recommendations,
                );
            }
            ComplianceFramework::Hipaa => {
                satisfied &= self.check_hipaa_requirement(
                    &requirement,
                    key_store,
                    &mut issues,
                    &mut recommendations,
                );
            }
            ComplianceFramework::Ccpa => {
                satisfied &= self.check_ccpa_requirement(
                    &requirement,
                    key_store,
                    &mut issues,
                    &mut recommendations,
                );
            }
            ComplianceFramework::PciDss => {
                satisfied &= self.check_pci_dss_requirement(
                    &requirement,
                    key_store,
                    &mut issues,
                    &mut recommendations,
                );
            }
            _ => {
                // For custom frameworks, do basic checks
                satisfied &= self.check_basic_requirements(
                    &requirement,
                    key_store,
                    &mut issues,
                    &mut recommendations,
                );
            }
        }

        let result = ComplianceCheckResult {
            requirement_id: requirement_id.to_string(),
            satisfied,
            evidence: format!("Checked against {:?} framework", requirement.framework),
            checked_at: Utc::now(),
            issues,
            recommendations,
        };

        self.check_history.push(result.clone());
        result
    }

    /// Check GDPR-specific requirements.
    fn check_gdpr_requirement(
        &self,
        requirement: &ComplianceRequirement,
        key_store: &KeyStore,
        issues: &mut Vec<ComplianceIssue>,
        recommendations: &mut Vec<String>,
    ) -> bool {
        let mut satisfied = true;

        match requirement.id.as_str() {
            "GDPR-ART-5" => {
                // Check for data processing principles
                // This would check for privacy notices, consent mechanisms, etc.
                // For now, we'll check basic encryption
                if !self.has_adequate_encryption(key_store) {
                    issues.push(ComplianceIssue {
                        id: Uuid::new_v4().to_string(),
                        severity: IssueSeverity::Medium,
                        description: "Inadequate encryption for personal data".to_string(),
                        component: "Encryption".to_string(),
                        remediation: "Implement strong encryption for data at rest and in transit"
                            .to_string(),
                        deadline: Some(Utc::now() + chrono::Duration::days(30)),
                    });
                    satisfied = false;
                }
                recommendations
                    .push("Implement data processing agreements and privacy notices".to_string());
            }
            "GDPR-ART-17" => {
                // Check for data deletion capabilities
                if !self.has_data_deletion_capability() {
                    issues.push(ComplianceIssue {
                        id: Uuid::new_v4().to_string(),
                        severity: IssueSeverity::High,
                        description: "No data deletion mechanism implemented".to_string(),
                        component: "Data Management".to_string(),
                        remediation: "Implement data deletion procedures and APIs".to_string(),
                        deadline: Some(Utc::now() + chrono::Duration::days(14)),
                    });
                    satisfied = false;
                }
                recommendations.push("Implement automated data deletion workflows".to_string());
            }
            "GDPR-ART-25" => {
                // Check for data protection by design
                if !self.has_privacy_by_design() {
                    issues.push(ComplianceIssue {
                        id: Uuid::new_v4().to_string(),
                        severity: IssueSeverity::Medium,
                        description: "Privacy by design not fully implemented".to_string(),
                        component: "System Design".to_string(),
                        remediation:
                            "Implement data minimization and privacy-enhancing technologies"
                                .to_string(),
                        deadline: Some(Utc::now() + chrono::Duration::days(60)),
                    });
                    satisfied = false;
                }
                recommendations
                    .push("Conduct privacy impact assessments for new features".to_string());
            }
            "GDPR-ART-32" => {
                // Check for security measures
                if !self.has_adequate_security_measures(key_store) {
                    issues.push(ComplianceIssue {
                        id: Uuid::new_v4().to_string(),
                        severity: IssueSeverity::High,
                        description: "Inadequate security measures for personal data".to_string(),
                        component: "Security".to_string(),
                        remediation: "Implement encryption, access controls, and security testing"
                            .to_string(),
                        deadline: Some(Utc::now() + chrono::Duration::days(30)),
                    });
                    satisfied = false;
                }
                recommendations
                    .push("Implement regular security testing and monitoring".to_string());
            }
            _ => {
                // Generic GDPR check
                satisfied &=
                    self.check_basic_requirements(requirement, key_store, issues, recommendations);
            }
        }

        satisfied
    }

    /// Check HIPAA-specific requirements.
    fn check_hipaa_requirement(
        &self,
        requirement: &ComplianceRequirement,
        key_store: &KeyStore,
        issues: &mut Vec<ComplianceIssue>,
        recommendations: &mut Vec<String>,
    ) -> bool {
        let mut satisfied = true;

        match requirement.id.as_str() {
            "HIPAA-164-308" => {
                // Administrative safeguards
                if !self.has_administrative_safeguards() {
                    issues.push(ComplianceIssue {
                        id: Uuid::new_v4().to_string(),
                        severity: IssueSeverity::High,
                        description: "Missing administrative safeguards".to_string(),
                        component: "Administration".to_string(),
                        remediation:
                            "Establish security management processes and workforce training"
                                .to_string(),
                        deadline: Some(Utc::now() + chrono::Duration::days(30)),
                    });
                    satisfied = false;
                }
                recommendations.push("Implement security awareness training program".to_string());
            }
            "HIPAA-164-312" => {
                // Technical safeguards
                if !self.has_technical_safeguards(key_store) {
                    issues.push(ComplianceIssue {
                        id: Uuid::new_v4().to_string(),
                        severity: IssueSeverity::Critical,
                        description: "Missing technical safeguards for PHI".to_string(),
                        component: "Security".to_string(),
                        remediation: "Implement access controls, audit controls, and encryption"
                            .to_string(),
                        deadline: Some(Utc::now() + chrono::Duration::days(7)),
                    });
                    satisfied = false;
                }
                recommendations
                    .push("Implement automatic logoff and encryption for PHI".to_string());
            }
            "HIPAA-164-314" => {
                // Organizational requirements
                if !self.has_business_associate_agreements() {
                    issues.push(ComplianceIssue {
                        id: Uuid::new_v4().to_string(),
                        severity: IssueSeverity::High,
                        description: "Missing business associate agreements".to_string(),
                        component: "Compliance".to_string(),
                        remediation: "Establish BAAs with all third-party processors".to_string(),
                        deadline: Some(Utc::now() + chrono::Duration::days(14)),
                    });
                    satisfied = false;
                }
                recommendations.push("Maintain inventory of business associates".to_string());
            }
            _ => {
                satisfied &=
                    self.check_basic_requirements(requirement, key_store, issues, recommendations);
            }
        }

        satisfied
    }

    /// Check CCPA-specific requirements.
    fn check_ccpa_requirement(
        &self,
        requirement: &ComplianceRequirement,
        _key_store: &KeyStore,
        issues: &mut Vec<ComplianceIssue>,
        recommendations: &mut Vec<String>,
    ) -> bool {
        let mut satisfied = true;

        match requirement.id.as_str() {
            "CCPA-1798-100" => {
                // Consumer rights
                if !self.has_consumer_rights_mechanisms() {
                    issues.push(ComplianceIssue {
                        id: Uuid::new_v4().to_string(),
                        severity: IssueSeverity::High,
                        description: "No consumer rights mechanisms implemented".to_string(),
                        component: "Privacy".to_string(),
                        remediation:
                            "Implement APIs for data access, deletion, and opt-out requests"
                                .to_string(),
                        deadline: Some(Utc::now() + chrono::Duration::days(30)),
                    });
                    satisfied = false;
                }
                recommendations
                    .push("Implement automated response to consumer requests".to_string());
            }
            "CCPA-1798-135" => {
                // Opt-out mechanisms
                if !self.has_opt_out_mechanisms() {
                    issues.push(ComplianceIssue {
                        id: Uuid::new_v4().to_string(),
                        severity: IssueSeverity::Medium,
                        description: "No opt-out mechanisms for data sale".to_string(),
                        component: "Privacy".to_string(),
                        remediation:
                            "Implement 'Do Not Sell My Personal Information' link and functionality"
                                .to_string(),
                        deadline: Some(Utc::now() + chrono::Duration::days(45)),
                    });
                    satisfied = false;
                }
                recommendations.push("Monitor and honor opt-out requests".to_string());
            }
            _ => {
                satisfied &=
                    self.check_basic_requirements(requirement, _key_store, issues, recommendations);
            }
        }

        satisfied
    }

    /// Check PCI DSS-specific requirements.
    fn check_pci_dss_requirement(
        &self,
        requirement: &ComplianceRequirement,
        key_store: &KeyStore,
        issues: &mut Vec<ComplianceIssue>,
        recommendations: &mut Vec<String>,
    ) -> bool {
        let mut satisfied = true;

        match requirement.id.as_str() {
            "PCI-DSS-3" => {
                // Protect stored cardholder data
                if !self.has_cardholder_data_protection(key_store) {
                    issues.push(ComplianceIssue {
                        id: Uuid::new_v4().to_string(),
                        severity: IssueSeverity::Critical,
                        description: "Inadequate protection for stored cardholder data".to_string(),
                        component: "Security".to_string(),
                        remediation: "Implement strong encryption and key management for PAN data"
                            .to_string(),
                        deadline: Some(Utc::now() + chrono::Duration::days(7)),
                    });
                    satisfied = false;
                }
                recommendations
                    .push("Mask PAN when displayed and limit storage duration".to_string());
            }
            "PCI-DSS-4" => {
                // Encrypt transmission of cardholder data
                if !self.has_secure_transmission() {
                    issues.push(ComplianceIssue {
                        id: Uuid::new_v4().to_string(),
                        severity: IssueSeverity::Critical,
                        description: "Insecure transmission of cardholder data".to_string(),
                        component: "Network Security".to_string(),
                        remediation:
                            "Use TLS 1.2+ for all transmissions and disable weak protocols"
                                .to_string(),
                        deadline: Some(Utc::now() + chrono::Duration::days(3)),
                    });
                    satisfied = false;
                }
                recommendations
                    .push("Implement network segmentation and intrusion detection".to_string());
            }
            _ => {
                satisfied &=
                    self.check_basic_requirements(requirement, key_store, issues, recommendations);
            }
        }

        satisfied
    }
    /// Check basic requirements applicable to all frameworks.
    fn check_basic_requirements(
        &self,
        requirement: &ComplianceRequirement,
        key_store: &KeyStore,
        issues: &mut Vec<ComplianceIssue>,
        recommendations: &mut Vec<String>,
    ) -> bool {
        let mut satisfied = true;

        // Check for encryption if requirement mentions security
        if (requirement.description.to_lowercase().contains("encrypt")
            || requirement.name.to_lowercase().contains("security"))
            && !self.has_adequate_encryption(key_store)
        {
            issues.push(ComplianceIssue {
                id: Uuid::new_v4().to_string(),
                severity: IssueSeverity::Medium,
                description: "Encryption not adequately implemented".to_string(),
                component: "Encryption".to_string(),
                remediation: "Review and strengthen encryption implementation".to_string(),
                deadline: Some(Utc::now() + chrono::Duration::days(30)),
            });
            satisfied = false;
        }

        // Check for audit logging if requirement mentions audit or monitoring
        if (requirement.description.to_lowercase().contains("audit")
            || requirement.description.to_lowercase().contains("monitor"))
            && !self.has_audit_logging()
        {
            issues.push(ComplianceIssue {
                id: Uuid::new_v4().to_string(),
                severity: IssueSeverity::Medium,
                description: "Audit logging not implemented".to_string(),
                component: "Monitoring".to_string(),
                remediation: "Implement comprehensive audit logging".to_string(),
                deadline: Some(Utc::now() + chrono::Duration::days(30)),
            });
            satisfied = false;
        }

        recommendations.push("Review requirement documentation and implement controls".to_string());
        satisfied
    }

    /// Check if adequate encryption is implemented.
    fn has_adequate_encryption(&self, key_store: &KeyStore) -> bool {
        // Check for active encryption keys
        let encryption_keys = key_store
            .list_private_keys()
            .iter()
            .filter(|key| {
                key.algorithm.contains("AES")
                    || key.algorithm.contains("ChaCha")
                    || key.algorithm.contains("GCM")
            })
            .count();

        encryption_keys > 0
    }

    /// Check if data deletion capability exists.
    fn has_data_deletion_capability(&self) -> bool {
        // In a real implementation, this would check for deletion APIs
        // For now, assume basic capability exists
        true
    }

    /// Check if privacy by design is implemented.
    fn has_privacy_by_design(&self) -> bool {
        // Check if privacy impact assessments exist
        !self.pia_assessments.is_empty()
    }

    /// Check if adequate security measures exist.
    fn has_adequate_security_measures(&self, key_store: &KeyStore) -> bool {
        self.has_adequate_encryption(key_store) && self.has_audit_logging()
    }

    /// Check if administrative safeguards exist.
    fn has_administrative_safeguards(&self) -> bool {
        // Check for security policies and training
        // For now, assume they exist
        true
    }

    /// Check if technical safeguards exist.
    fn has_technical_safeguards(&self, key_store: &KeyStore) -> bool {
        self.has_adequate_encryption(key_store)
            && self.has_audit_logging()
            && self.has_access_controls()
    }

    /// Check if business associate agreements exist.
    fn has_business_associate_agreements(&self) -> bool {
        // For now, assume they exist
        true
    }

    /// Check if consumer rights mechanisms exist.
    fn has_consumer_rights_mechanisms(&self) -> bool {
        // Check for data subject request handling
        // For now, assume basic mechanisms exist
        true
    }

    /// Check if opt-out mechanisms exist.
    fn has_opt_out_mechanisms(&self) -> bool {
        // Check for opt-out functionality
        // For now, assume they exist
        true
    }

    /// Check if cardholder data is protected.
    fn has_cardholder_data_protection(&self, key_store: &KeyStore) -> bool {
        self.has_adequate_encryption(key_store) && self.has_access_controls()
    }

    /// Check if secure transmission is implemented.
    fn has_secure_transmission(&self) -> bool {
        // Check for TLS/SSL implementation
        // For now, assume it exists
        true
    }

    /// Check if audit logging is implemented.
    fn has_audit_logging(&self) -> bool {
        // Check if audit logging is enabled
        // For now, assume it exists
        true
    }

    /// Check if access controls are implemented.
    fn has_access_controls(&self) -> bool {
        // Check for access control mechanisms
        // For now, assume they exist
        true
    }

    /// Add a privacy impact assessment.
    pub fn add_privacy_impact_assessment(&mut self, assessment: PrivacyImpactAssessment) {
        self.pia_assessments
            .insert(assessment.id.clone(), assessment);
    }

    /// Get privacy impact assessment by ID.
    pub fn get_privacy_impact_assessment(&self, id: &str) -> Option<&PrivacyImpactAssessment> {
        self.pia_assessments.get(id)
    }

    /// Get data retention policy for classification.
    pub fn get_retention_policy(
        &self,
        classification: &DataClassification,
    ) -> Option<&DataRetentionPolicy> {
        self.retention_policies.get(classification)
    }

    /// Get all compliance requirements.
    pub fn get_requirements(&self) -> Vec<&ComplianceRequirement> {
        self.requirements.values().collect()
    }

    /// Get compliance check history.
    pub fn get_check_history(&self) -> &[ComplianceCheckResult] {
        &self.check_history
    }

    /// Run comprehensive compliance audit.
    pub fn run_comprehensive_audit(
        &mut self,
        key_store: &KeyStore,
        data_classifications: &[DataClassification],
    ) -> Vec<ComplianceCheckResult> {
        let mut results = Vec::new();

        // Collect requirement IDs first to avoid borrow issues
        let mut requirement_ids = Vec::new();
        for classification in data_classifications {
            for requirement in self.requirements.values() {
                if requirement.applies_to.is_empty()
                    || requirement.applies_to.contains(classification)
                {
                    requirement_ids.push((requirement.id.clone(), classification.clone()));
                }
            }
        }

        for (requirement_id, classification) in requirement_ids {
            let result = self.check_requirement(&requirement_id, key_store, &classification);
            results.push(result);
        }

        results
    }

    /// Generate compliance report.
    pub fn generate_compliance_report(&self) -> String {
        let mut report = String::new();

        report.push_str("=== Compliance Report ===\n\n");
        report.push_str(&format!("Active Frameworks: {}\n", self.frameworks.len()));
        report.push_str(&format!("Requirements: {}\n", self.requirements.len()));
        report.push_str(&format!(
            "Checks Performed: {}\n\n",
            self.check_history.len()
        ));

        // Summary statistics
        let total_checks = self.check_history.len();
        let passed_checks = self.check_history.iter().filter(|c| c.satisfied).count();
        let failed_checks = total_checks - passed_checks;

        report.push_str("=== Summary ===\n");
        report.push_str(&format!("Total Checks: {total_checks}\n"));
        report.push_str(&format!(
            "Passed: {} ({:.1}%)\n",
            passed_checks,
            (passed_checks as f64 / total_checks as f64) * 100.0
        ));
        report.push_str(&format!(
            "Failed: {} ({:.1}%)\n\n",
            failed_checks,
            (failed_checks as f64 / total_checks as f64) * 100.0
        ));

        // Issues by severity
        let mut critical_issues = 0;
        let mut high_issues = 0;
        let mut medium_issues = 0;
        let mut low_issues = 0;

        for check in &self.check_history {
            for issue in &check.issues {
                match issue.severity {
                    IssueSeverity::Critical => critical_issues += 1,
                    IssueSeverity::High => high_issues += 1,
                    IssueSeverity::Medium => medium_issues += 1,
                    IssueSeverity::Low => low_issues += 1,
                    IssueSeverity::Informational => {}
                }
            }
        }

        report.push_str("=== Issues by Severity ===\n");
        report.push_str(&format!("Critical: {critical_issues}\n"));
        report.push_str(&format!("High: {high_issues}\n"));
        report.push_str(&format!("Medium: {medium_issues}\n"));
        report.push_str(&format!("Low: {low_issues}\n\n"));

        // Framework compliance
        report.push_str("=== Framework Compliance ===\n");
        for framework in &self.frameworks {
            let framework_checks: Vec<&ComplianceCheckResult> = self
                .check_history
                .iter()
                .filter(|check| {
                    self.requirements
                        .get(&check.requirement_id)
                        .map(|req| &req.framework == framework)
                        .unwrap_or(false)
                })
                .collect();

            let framework_passed = framework_checks.iter().filter(|c| c.satisfied).count();
            let framework_total = framework_checks.len();

            if framework_total > 0 {
                report.push_str(&format!(
                    "{:?}: {}/{} ({:.1}%)\n",
                    framework,
                    framework_passed,
                    framework_total,
                    (framework_passed as f64 / framework_total as f64) * 100.0
                ));
            }
        }

        report
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mcp::crypto::{KeyStore, KeyUsage, PrivateKey};
    use chrono::TimeZone;

    #[test]
    fn test_compliance_framework_enum() {
        let gdpr = ComplianceFramework::Gdpr;
        let hipaa = ComplianceFramework::Hipaa;
        let ccpa = ComplianceFramework::Ccpa;
        let pci_dss = ComplianceFramework::PciDss;
        let iso = ComplianceFramework::Iso27001;
        let fedramp = ComplianceFramework::FedRamp;
        let custom = ComplianceFramework::Custom("MyFramework".to_string());

        assert_eq!(format!("{:?}", gdpr), "Gdpr");
        assert_eq!(format!("{:?}", hipaa), "Hipaa");
        assert_eq!(format!("{:?}", ccpa), "Ccpa");
        assert_eq!(format!("{:?}", pci_dss), "PciDss");
        assert_eq!(format!("{:?}", iso), "Iso27001");
        assert_eq!(format!("{:?}", fedramp), "FedRamp");
        assert_eq!(format!("{:?}", custom), "Custom(\"MyFramework\")");
    }

    #[test]
    fn test_data_classification_enum() {
        let public = DataClassification::Public;
        let internal = DataClassification::Internal;
        let confidential = DataClassification::Confidential;
        let restricted = DataClassification::Restricted;
        let pii = DataClassification::Pii;
        let phi = DataClassification::Phi;
        let pci = DataClassification::Pci;

        assert_eq!(format!("{:?}", public), "Public");
        assert_eq!(format!("{:?}", internal), "Internal");
        assert_eq!(format!("{:?}", confidential), "Confidential");
        assert_eq!(format!("{:?}", restricted), "Restricted");
        assert_eq!(format!("{:?}", pii), "Pii");
        assert_eq!(format!("{:?}", phi), "Phi");
        assert_eq!(format!("{:?}", pci), "Pci");
    }

    #[test]
    fn test_compliance_requirement_creation() {
        let requirement = ComplianceRequirement {
            id: "TEST-001".to_string(),
            framework: ComplianceFramework::Gdpr,
            name: "Test Requirement".to_string(),
            description: "Test description".to_string(),
            mandatory: true,
            applies_to: vec![DataClassification::Pii, DataClassification::Confidential],
            guidance: "Test guidance".to_string(),
            verification: VerificationMethod::Automated,
        };

        assert_eq!(requirement.id, "TEST-001");
        assert!(matches!(requirement.framework, ComplianceFramework::Gdpr));
        assert_eq!(requirement.name, "Test Requirement");
        assert_eq!(requirement.description, "Test description");
        assert!(requirement.mandatory);
        assert_eq!(requirement.applies_to.len(), 2);
        assert!(matches!(
            requirement.verification,
            VerificationMethod::Automated
        ));
    }

    #[test]
    fn test_verification_method_enum() {
        let automated = VerificationMethod::Automated;
        let manual = VerificationMethod::Manual;
        let audit_trail = VerificationMethod::AuditTrail;
        let documentation = VerificationMethod::Documentation;
        let combined = VerificationMethod::Combined(vec![
            VerificationMethod::Automated,
            VerificationMethod::Manual,
        ]);

        assert_eq!(format!("{:?}", automated), "Automated");
        assert_eq!(format!("{:?}", manual), "Manual");
        assert_eq!(format!("{:?}", audit_trail), "AuditTrail");
        assert_eq!(format!("{:?}", documentation), "Documentation");
        assert_eq!(format!("{:?}", combined), "Combined([Automated, Manual])");
    }

    #[test]
    fn test_compliance_check_result_creation() {
        let now = Utc::now();
        let result = ComplianceCheckResult {
            requirement_id: "GDPR-ART-5".to_string(),
            satisfied: true,
            evidence: "Checked encryption and access controls".to_string(),
            checked_at: now,
            issues: vec![],
            recommendations: vec!["Implement additional logging".to_string()],
        };

        assert_eq!(result.requirement_id, "GDPR-ART-5");
        assert!(result.satisfied);
        assert_eq!(result.evidence, "Checked encryption and access controls");
        assert_eq!(result.checked_at, now);
        assert!(result.issues.is_empty());
        assert_eq!(result.recommendations.len(), 1);
    }

    #[test]
    fn test_compliance_issue_creation() {
        let deadline = Utc.with_ymd_and_hms(2024, 12, 31, 23, 59, 59).unwrap();
        let issue = ComplianceIssue {
            id: "ISSUE-001".to_string(),
            severity: IssueSeverity::High,
            description: "Missing encryption for PII data".to_string(),
            component: "Data Storage".to_string(),
            remediation: "Implement AES-256 encryption".to_string(),
            deadline: Some(deadline),
        };

        assert_eq!(issue.id, "ISSUE-001");
        assert!(matches!(issue.severity, IssueSeverity::High));
        assert_eq!(issue.description, "Missing encryption for PII data");
        assert_eq!(issue.component, "Data Storage");
        assert_eq!(issue.remediation, "Implement AES-256 encryption");
        assert_eq!(issue.deadline, Some(deadline));
    }

    #[test]
    fn test_issue_severity_enum() {
        let critical = IssueSeverity::Critical;
        let high = IssueSeverity::High;
        let medium = IssueSeverity::Medium;
        let low = IssueSeverity::Low;
        let informational = IssueSeverity::Informational;

        assert_eq!(format!("{:?}", critical), "Critical");
        assert_eq!(format!("{:?}", high), "High");
        assert_eq!(format!("{:?}", medium), "Medium");
        assert_eq!(format!("{:?}", low), "Low");
        assert_eq!(format!("{:?}", informational), "Informational");
    }

    #[test]
    fn test_data_retention_policy_creation() {
        let policy = DataRetentionPolicy {
            id: "RET-001".to_string(),
            classification: DataClassification::Pii,
            retention_days: 365,
            encrypt_at_rest: true,
            encrypt_in_transit: true,
            access_control: AccessControlRequirements {
                authentication_required: true,
                authorization_required: true,
                min_auth_strength: AuthStrength::Mfa,
                mfa_required: true,
                access_review_frequency_days: 90,
            },
            audit_logging: true,
            deletion_method: DeletionMethod::Cryptographic,
        };

        assert_eq!(policy.id, "RET-001");
        assert!(matches!(policy.classification, DataClassification::Pii));
        assert_eq!(policy.retention_days, 365);
        assert!(policy.encrypt_at_rest);
        assert!(policy.encrypt_in_transit);
        assert!(policy.access_control.authentication_required);
        assert!(policy.access_control.authorization_required);
        assert!(matches!(
            policy.access_control.min_auth_strength,
            AuthStrength::Mfa
        ));
        assert!(policy.access_control.mfa_required);
        assert_eq!(policy.access_control.access_review_frequency_days, 90);
        assert!(policy.audit_logging);
        assert!(matches!(
            policy.deletion_method,
            DeletionMethod::Cryptographic
        ));
    }

    #[test]
    fn test_auth_strength_enum() {
        let password = AuthStrength::Password;
        let mfa = AuthStrength::Mfa;
        let certificate = AuthStrength::Certificate;
        let hardware_token = AuthStrength::HardwareToken;
        let biometric = AuthStrength::Biometric;

        assert_eq!(format!("{:?}", password), "Password");
        assert_eq!(format!("{:?}", mfa), "Mfa");
        assert_eq!(format!("{:?}", certificate), "Certificate");
        assert_eq!(format!("{:?}", hardware_token), "HardwareToken");
        assert_eq!(format!("{:?}", biometric), "Biometric");
    }

    #[test]
    fn test_deletion_method_enum() {
        let logical = DeletionMethod::Logical;
        let physical = DeletionMethod::Physical;
        let cryptographic = DeletionMethod::Cryptographic;
        let secure = DeletionMethod::Secure;

        assert_eq!(format!("{:?}", logical), "Logical");
        assert_eq!(format!("{:?}", physical), "Physical");
        assert_eq!(format!("{:?}", cryptographic), "Cryptographic");
        assert_eq!(format!("{:?}", secure), "Secure");
    }

    #[test]
    fn test_privacy_impact_assessment_creation() {
        let assessment_date = Utc.with_ymd_and_hms(2024, 1, 1, 0, 0, 0).unwrap();
        let next_review_date = Utc.with_ymd_and_hms(2025, 1, 1, 0, 0, 0).unwrap();

        let assessment = PrivacyImpactAssessment {
            id: "PIA-001".to_string(),
            system_name: "Customer Database".to_string(),
            data_types: vec![DataClassification::Pii, DataClassification::Confidential],
            purposes: vec!["Customer Support".to_string(), "Marketing".to_string()],
            retention_period_days: 730,
            third_party_sharing: true,
            risk_level: RiskLevel::Medium,
            mitigations: vec!["Encryption".to_string(), "Access Controls".to_string()],
            assessment_date,
            next_review_date,
        };

        assert_eq!(assessment.id, "PIA-001");
        assert_eq!(assessment.system_name, "Customer Database");
        assert_eq!(assessment.data_types.len(), 2);
        assert_eq!(assessment.purposes.len(), 2);
        assert_eq!(assessment.retention_period_days, 730);
        assert!(assessment.third_party_sharing);
        assert!(matches!(assessment.risk_level, RiskLevel::Medium));
        assert_eq!(assessment.mitigations.len(), 2);
        assert_eq!(assessment.assessment_date, assessment_date);
        assert_eq!(assessment.next_review_date, next_review_date);
    }

    #[test]
    fn test_risk_level_enum() {
        let low = RiskLevel::Low;
        let medium = RiskLevel::Medium;
        let high = RiskLevel::High;
        let critical = RiskLevel::Critical;

        assert_eq!(format!("{:?}", low), "Low");
        assert_eq!(format!("{:?}", medium), "Medium");
        assert_eq!(format!("{:?}", high), "High");
        assert_eq!(format!("{:?}", critical), "Critical");
    }

    #[test]
    fn test_compliance_manager_creation() {
        let frameworks = vec![ComplianceFramework::Gdpr, ComplianceFramework::Hipaa];

        let manager = ComplianceManager::new(frameworks.clone());

        assert_eq!(manager.frameworks.len(), 2);
        assert!(manager.frameworks.contains(&ComplianceFramework::Gdpr));
        assert!(manager.frameworks.contains(&ComplianceFramework::Hipaa));

        // Should have loaded default requirements for GDPR and HIPAA
        assert!(manager.requirements.len() > 0);
        assert!(manager.retention_policies.len() > 0);
        assert!(manager.pia_assessments.is_empty());
        assert!(manager.check_history.is_empty());
    }

    #[test]
    fn test_compliance_manager_gdpr_only() {
        let frameworks = vec![ComplianceFramework::Gdpr];
        let manager = ComplianceManager::new(frameworks);

        // Should have GDPR requirements but not HIPAA
        let gdpr_requirements: Vec<_> = manager
            .requirements
            .values()
            .filter(|req| matches!(req.framework, ComplianceFramework::Gdpr))
            .collect();

        let hipaa_requirements: Vec<_> = manager
            .requirements
            .values()
            .filter(|req| matches!(req.framework, ComplianceFramework::Hipaa))
            .collect();

        assert!(gdpr_requirements.len() > 0);
        assert_eq!(hipaa_requirements.len(), 0);
    }

    #[test]
    fn test_compliance_manager_get_requirements() {
        let frameworks = vec![ComplianceFramework::Gdpr];
        let manager = ComplianceManager::new(frameworks);

        let requirements = manager.get_requirements();
        assert!(requirements.len() > 0);

        // All requirements should be for GDPR
        for req in requirements {
            assert!(matches!(req.framework, ComplianceFramework::Gdpr));
        }
    }

    #[test]
    fn test_compliance_manager_get_check_history() {
        let frameworks = vec![ComplianceFramework::Gdpr];
        let manager = ComplianceManager::new(frameworks);

        let history = manager.get_check_history();
        assert!(history.is_empty()); // No checks performed yet
    }

    #[test]
    fn test_compliance_manager_add_and_get_privacy_impact_assessment() {
        let frameworks = vec![ComplianceFramework::Gdpr];
        let mut manager = ComplianceManager::new(frameworks);

        let assessment = PrivacyImpactAssessment {
            id: "TEST-PIA".to_string(),
            system_name: "Test System".to_string(),
            data_types: vec![DataClassification::Pii],
            purposes: vec!["Testing".to_string()],
            retention_period_days: 30,
            third_party_sharing: false,
            risk_level: RiskLevel::Low,
            mitigations: vec![],
            assessment_date: Utc::now(),
            next_review_date: Utc::now(),
        };

        manager.add_privacy_impact_assessment(assessment);

        let retrieved = manager.get_privacy_impact_assessment("TEST-PIA");
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().system_name, "Test System");

        let not_found = manager.get_privacy_impact_assessment("NONEXISTENT");
        assert!(not_found.is_none());
    }

    #[test]
    fn test_compliance_manager_get_retention_policy() {
        let frameworks = vec![ComplianceFramework::Gdpr];
        let manager = ComplianceManager::new(frameworks);

        // Should have default retention policies loaded
        let pii_policy = manager.get_retention_policy(&DataClassification::Pii);
        assert!(pii_policy.is_some());

        let phi_policy = manager.get_retention_policy(&DataClassification::Phi);
        assert!(phi_policy.is_some());

        let public_policy = manager.get_retention_policy(&DataClassification::Public);
        assert!(public_policy.is_some());
    }

    #[test]
    fn test_check_requirement_with_gdpr() {
        let frameworks = vec![ComplianceFramework::Gdpr];
        let mut manager = ComplianceManager::new(frameworks);

        // Create a mock key store with encryption keys
        let mut key_store = KeyStore::new();

        // Add an encryption key to pass the check
        let encryption_key = PrivateKey {
            id: "test-encryption-key".to_string(),
            algorithm: "AES-256-GCM".to_string(),
            material: vec![0u8; 32], // 32 bytes for AES-256
            usage: KeyUsage::Encryption,
        };
        key_store.add_private_key(encryption_key);

        // Test checking a GDPR requirement
        let result = manager.check_requirement("GDPR-ART-5", &key_store, &DataClassification::Pii);

        assert_eq!(result.requirement_id, "GDPR-ART-5");
        // With encryption keys, should pass
        assert!(result.satisfied);
        assert!(result.evidence.contains("Gdpr"));
        assert!(!result.checked_at.to_string().is_empty());

        // Check history should be updated
        let history = manager.get_check_history();
        assert_eq!(history.len(), 1);
        assert_eq!(history[0].requirement_id, "GDPR-ART-5");
    }

    #[test]
    fn test_check_requirement_with_nonexistent_id() {
        let frameworks = vec![ComplianceFramework::Gdpr];
        let mut manager = ComplianceManager::new(frameworks);

        let key_store = KeyStore::new();

        // Test checking a non-existent requirement
        let result =
            manager.check_requirement("NONEXISTENT-001", &key_store, &DataClassification::Pii);

        assert_eq!(result.requirement_id, "NONEXISTENT-001");
        // The requirement should not be added to manager.requirements
        // It creates a placeholder but doesn't store it
        assert!(manager.requirements.get("NONEXISTENT-001").is_none());
    }

    #[test]
    fn test_check_requirement_wrong_data_classification() {
        let frameworks = vec![ComplianceFramework::Gdpr];
        let mut manager = ComplianceManager::new(frameworks);

        let key_store = KeyStore::new();

        // GDPR requirements apply to PII, not Public data
        let result =
            manager.check_requirement("GDPR-ART-5", &key_store, &DataClassification::Public);

        // Should have an informational issue about wrong classification
        assert!(!result.issues.is_empty());
        assert!(result.issues[0].description.contains("does not apply"));
        assert!(matches!(
            result.issues[0].severity,
            IssueSeverity::Informational
        ));
    }

    #[test]
    fn test_run_comprehensive_audit() {
        let frameworks = vec![ComplianceFramework::Gdpr, ComplianceFramework::Hipaa];
        let mut manager = ComplianceManager::new(frameworks);

        // Create a key store with encryption keys
        let mut key_store = KeyStore::new();
        let encryption_key = PrivateKey {
            id: "test-encryption-key".to_string(),
            algorithm: "AES-256-GCM".to_string(),
            material: vec![0u8; 32],
            usage: KeyUsage::Encryption,
        };
        key_store.add_private_key(encryption_key);

        let data_classifications = vec![DataClassification::Pii, DataClassification::Phi];

        let results = manager.run_comprehensive_audit(&key_store, &data_classifications);

        // Should have results for both GDPR and HIPAA requirements
        assert!(results.len() > 0);

        let gdpr_results: Vec<_> = results
            .iter()
            .filter(|r| r.requirement_id.starts_with("GDPR"))
            .collect();

        let hipaa_results: Vec<_> = results
            .iter()
            .filter(|r| r.requirement_id.starts_with("HIPAA"))
            .collect();

        assert!(gdpr_results.len() > 0);
        assert!(hipaa_results.len() > 0);

        // Check history should be updated with all results
        assert_eq!(manager.get_check_history().len(), results.len());
    }

    #[test]
    fn test_generate_compliance_report() {
        let frameworks = vec![ComplianceFramework::Gdpr];
        let mut manager = ComplianceManager::new(frameworks);

        // Create a key store with encryption keys
        let mut key_store = KeyStore::new();
        let encryption_key = PrivateKey {
            id: "test-encryption-key".to_string(),
            algorithm: "AES-256-GCM".to_string(),
            material: vec![0u8; 32],
            usage: KeyUsage::Encryption,
        };
        key_store.add_private_key(encryption_key);

        // Run some checks first
        let _ = manager.check_requirement("GDPR-ART-5", &key_store, &DataClassification::Pii);

        let _ = manager.check_requirement("GDPR-ART-17", &key_store, &DataClassification::Pii);

        // Generate report
        let report = manager.generate_compliance_report();

        // Report should contain expected sections
        assert!(report.contains("=== Compliance Report ==="));
        assert!(report.contains("=== Summary ==="));
        assert!(report.contains("Total Checks:"));
        assert!(report.contains("Passed:"));
        assert!(report.contains("Failed:"));
        assert!(report.contains("=== Issues by Severity ==="));
        assert!(report.contains("=== Framework Compliance ==="));
        // The framework is printed with Debug format, so it should contain "Gdpr"
        assert!(report.contains("Gdpr"));
    }

    #[test]
    fn test_generate_compliance_report_empty_history() {
        let frameworks = vec![ComplianceFramework::Gdpr];
        let manager = ComplianceManager::new(frameworks);

        // Generate report with no checks performed
        let report = manager.generate_compliance_report();

        assert!(report.contains("Total Checks: 0"));
        assert!(report.contains("Passed: 0 (NaN%)"));
        assert!(report.contains("Failed: 0 (NaN%)"));
    }

    #[test]
    fn test_compliance_manager_with_multiple_frameworks() {
        let frameworks = vec![
            ComplianceFramework::Gdpr,
            ComplianceFramework::Hipaa,
            ComplianceFramework::Ccpa,
            ComplianceFramework::PciDss,
        ];

        let manager = ComplianceManager::new(frameworks);

        // Should have requirements for all frameworks
        let gdpr_count = manager
            .requirements
            .values()
            .filter(|req| matches!(req.framework, ComplianceFramework::Gdpr))
            .count();

        let hipaa_count = manager
            .requirements
            .values()
            .filter(|req| matches!(req.framework, ComplianceFramework::Hipaa))
            .count();

        let ccpa_count = manager
            .requirements
            .values()
            .filter(|req| matches!(req.framework, ComplianceFramework::Ccpa))
            .count();

        let pci_dss_count = manager
            .requirements
            .values()
            .filter(|req| matches!(req.framework, ComplianceFramework::PciDss))
            .count();

        assert!(gdpr_count > 0);
        assert!(hipaa_count > 0);
        assert!(ccpa_count > 0);
        assert!(pci_dss_count > 0);
    }

    #[test]
    fn test_serialization_deserialization() {
        use serde_json;

        // Test ComplianceFramework
        let framework = ComplianceFramework::Gdpr;
        let serialized = serde_json::to_string(&framework).unwrap();
        let deserialized: ComplianceFramework = serde_json::from_str(&serialized).unwrap();
        assert_eq!(framework, deserialized);

        // Test DataClassification
        let classification = DataClassification::Pii;
        let serialized = serde_json::to_string(&classification).unwrap();
        let deserialized: DataClassification = serde_json::from_str(&serialized).unwrap();
        assert_eq!(classification, deserialized);

        // Test ComplianceRequirement
        let requirement = ComplianceRequirement {
            id: "TEST-001".to_string(),
            framework: ComplianceFramework::Gdpr,
            name: "Test".to_string(),
            description: "Test".to_string(),
            mandatory: true,
            applies_to: vec![DataClassification::Pii],
            guidance: "Test".to_string(),
            verification: VerificationMethod::Automated,
        };

        let serialized = serde_json::to_string(&requirement).unwrap();
        let deserialized: ComplianceRequirement = serde_json::from_str(&serialized).unwrap();
        assert_eq!(requirement.id, deserialized.id);
        assert_eq!(requirement.framework, deserialized.framework);
    }

    #[test]
    fn test_compliance_check_result_with_issues() {
        let now = Utc::now();
        let result = ComplianceCheckResult {
            requirement_id: "TEST-001".to_string(),
            satisfied: false,
            evidence: "Found issues".to_string(),
            checked_at: now,
            issues: vec![
                ComplianceIssue {
                    id: "ISSUE-001".to_string(),
                    severity: IssueSeverity::High,
                    description: "Missing encryption".to_string(),
                    component: "Storage".to_string(),
                    remediation: "Implement encryption".to_string(),
                    deadline: None,
                },
                ComplianceIssue {
                    id: "ISSUE-002".to_string(),
                    severity: IssueSeverity::Medium,
                    description: "Weak password policy".to_string(),
                    component: "Authentication".to_string(),
                    remediation: "Enforce strong passwords".to_string(),
                    deadline: None,
                },
            ],
            recommendations: vec![
                "Upgrade to AES-256".to_string(),
                "Implement MFA".to_string(),
            ],
        };

        assert!(!result.satisfied);
        assert_eq!(result.issues.len(), 2);
        assert_eq!(result.recommendations.len(), 2);

        let high_issues: Vec<_> = result
            .issues
            .iter()
            .filter(|issue| matches!(issue.severity, IssueSeverity::High))
            .collect();

        let medium_issues: Vec<_> = result
            .issues
            .iter()
            .filter(|issue| matches!(issue.severity, IssueSeverity::Medium))
            .collect();

        assert_eq!(high_issues.len(), 1);
        assert_eq!(medium_issues.len(), 1);
    }
}
