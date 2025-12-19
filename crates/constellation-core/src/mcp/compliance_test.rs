//! Tests for the compliance module.

use crate::mcp::compliance::*;
use crate::mcp::crypto::KeyStore;
use chrono::{TimeZone, Utc};

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
    assert!(matches!(requirement.verification, VerificationMethod::Automated));
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
    assert_eq!(
        format!("{:?}", combined),
        "Combined([Automated, Manual])"
    );
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
    assert!(matches!(policy.access_control.min_auth_strength, AuthStrength::Mfa));
    assert!(policy.access_control.mfa_required);
    assert_eq!(policy.access_control.access_review_frequency_days, 90);
    assert!(policy.audit_logging);
    assert!(matches!(policy.deletion_method, DeletionMethod::Cryptographic));
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
    let frameworks = vec![
        ComplianceFramework::Gdpr,
        ComplianceFramework::Hipaa,
    ];
    
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
    let gdpr_requirements: Vec<_> = manager.requirements.values()
        .filter(|req| matches!(req.framework, ComplianceFramework::Gdpr))
        .collect();
    
    let hipaa_requirements: Vec<_> = manager.requirements.values()
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
    
    // Create a mock key store
    let key_store = KeyStore::new();
    
    // Test checking a GDPR requirement
    let result = manager.check_requirement(
        "GDPR-ART-5",
        &key_store,
        &DataClassification::Pii,
    );
    
    assert_eq!(result.requirement_id, "GDPR-ART-5");
    assert!(result.satisfied); // Should pass basic checks
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
    let result = manager.check_requirement(
        "NONEXISTENT-001",
        &key_store,
        &DataClassification::Pii,
    );
    
    assert_eq!(result.requirement_id, "NONEXISTENT-001");
    // Should create a custom framework requirement
    assert!(matches!(
        manager.requirements.get("NONEXISTENT-001").unwrap().framework,
        ComplianceFramework::Custom(_)
    ));
}

#[test]
fn test_check_requirement_wrong_data_classification() {
    let frameworks = vec![ComplianceFramework::Gdpr];
    let mut manager = ComplianceManager::new(frameworks);
    
    let key_store = KeyStore::new();
    
    // GDPR requirements apply to PII, not Public data
    let result = manager.check_requirement(
        "GDPR-ART-5",
        &key_store,
        &DataClassification::Public,
    );
    
    // Should have an informational issue about wrong classification
    assert!(!result.issues.is_empty());
    assert!(result.issues[0].description.contains("does not apply"));
    assert!(matches!(result.issues[0].severity, IssueSeverity::Informational));
}

#[test]
fn test_run_comprehensive_audit() {
    let frameworks = vec![ComplianceFramework::Gdpr, ComplianceFramework::Hipaa];
    let mut manager = ComplianceManager::new(frameworks);
    
    let key_store = KeyStore::new();
    let data_classifications = vec![
        DataClassification::Pii,
        DataClassification::Phi,
    ];
    
    let results = manager.run_comprehensive_audit(&key_store, &data_classifications);
    
    // Should have results for both GDPR and HIPAA requirements
    assert!(results.len() > 0);
    
    let gdpr_results: Vec<_> = results.iter()
        .filter(|r| r.requirement_id.starts_with("GDPR"))
        .collect();
    
    let hipaa_results: Vec<_> = results.iter()
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
    
    let key_store = KeyStore::new();
    
    // Run some checks first
    let _ = manager.check_requirement(
        "GDPR-ART-5",
        &key_store,
        &DataClassification::Pii,
    );
    
    let _ = manager.check_requirement(
        "GDPR-ART-17",
        &key_store,
        &DataClassification::Pii,
    );
    
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
    assert!(report.contains("GDPR"));
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
    let gdpr_count = manager.requirements.values()
        .filter(|req| matches!(req.framework, ComplianceFramework::Gdpr))
        .count();
    
    let hipaa_count = manager.requirements.values()
        .filter(|req| matches!(req.framework, ComplianceFramework::Hipaa))
        .count();
    
    let ccpa_count = manager.requirements.values()
        .filter(|req| matches!(req.framework, ComplianceFramework::Ccpa))
        .count();
    
    let pci_dss_count = manager.requirements.values()
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
    
    let high_issues: Vec<_> = result.issues.iter()
        .filter(|issue| matches!(issue.severity, IssueSeverity::High))
        .collect();
    
    let medium_issues: Vec<_> = result.issues.iter()
        .filter(|issue| matches!(issue.severity, IssueSeverity::Medium))
        .collect();
    
    assert_eq!(high_issues.len(), 1);
    assert_eq!(medium_issues.len(), 1);
}