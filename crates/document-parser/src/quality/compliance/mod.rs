//! Modified: 2025-01-22

//! Compliance Assessment Module
//! 
//! Comprehensive compliance assessment for POA&M items against FedRAMP, OSCAL, and regulatory standards

pub mod types;
pub mod fedramp_compliance;
pub mod oscal_compliance;
pub mod regulatory_compliance;
pub mod assessor;

// Re-export main types for backward compatibility
pub use types::*;
pub use fedramp_compliance::FedRampComplianceChecker;
pub use oscal_compliance::OscalComplianceChecker;
pub use regulatory_compliance::{RegulatoryComplianceChecker, RegulatoryRule};
pub use assessor::ComplianceAssessor;

// Re-export for backward compatibility with the original single file
pub use assessor::ComplianceAssessor as ComplianceAssessor_Legacy;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::poam::PoamItem;

    fn create_test_poam_item() -> PoamItem {
        PoamItem {
            uuid: "550e8400-e29b-41d4-a716-446655440000".to_string(),
            title: "Test vulnerability requiring remediation".to_string(),
            description: "This is a detailed description of a security vulnerability that needs to be addressed according to compliance requirements.".to_string(),
            status: "Open".to_string(),
            scheduled_completion_date: Some("2024-12-31T23:59:59Z".to_string()),
            actual_completion_date: None,
            responsible_entity: Some("Security Team".to_string()),
            resources_required: Some("2 FTE, security tools".to_string()),
            severity: Some("High".to_string()),
            risk_assessment: Some("High risk vulnerability that could lead to data breach if not addressed promptly.".to_string()),
            ..Default::default()
        }
    }

    fn create_non_compliant_poam_item() -> PoamItem {
        PoamItem {
            uuid: "invalid-uuid".to_string(),
            title: "Short".to_string(),
            description: "Too short".to_string(),
            status: "Unknown".to_string(),
            scheduled_completion_date: None,
            actual_completion_date: None,
            responsible_entity: None,
            resources_required: None,
            severity: None,
            risk_assessment: None,
            ..Default::default()
        }
    }

    #[test]
    fn test_compliance_assessor_creation() {
        let assessor = ComplianceAssessor::new();
        assert!(assessor.get_config().enable_fedramp_checks);
        assert!(assessor.get_config().enable_oscal_checks);
        assert!(assessor.get_config().enable_regulatory_checks);
    }

    #[test]
    fn test_compliance_assessment_empty_items() {
        let assessor = ComplianceAssessor::new();
        let result = assessor.assess(&[]).unwrap();
        
        assert_eq!(result.score, 1.0);
        assert!(result.findings.is_empty());
        assert!(result.fedramp_results.is_empty());
        assert!(result.oscal_results.is_empty());
        assert!(result.regulatory_results.is_empty());
    }

    #[test]
    fn test_compliance_assessment_compliant_item() {
        let assessor = ComplianceAssessor::new();
        let items = vec![create_test_poam_item()];
        let result = assessor.assess(&items).unwrap();
        
        assert!(result.score > 0.8); // Should have high compliance
        assert!(!result.fedramp_results.is_empty());
        assert!(!result.oscal_results.is_empty());
    }

    #[test]
    fn test_compliance_assessment_non_compliant_item() {
        let assessor = ComplianceAssessor::new();
        let items = vec![create_non_compliant_poam_item()];
        let result = assessor.assess(&items).unwrap();
        
        assert!(result.score < 0.5); // Should have low compliance
        assert!(!result.findings.is_empty()); // Should have findings
    }

    #[test]
    fn test_fedramp_compliance_checker() {
        let checker = FedRampComplianceChecker::new();
        let items = vec![create_test_poam_item()];
        let results = checker.assess_compliance(&items).unwrap();
        
        assert!(!results.is_empty());
        assert!(results.iter().all(|r| r.standard == "FedRAMP"));
    }

    #[test]
    fn test_oscal_compliance_checker() {
        let checker = OscalComplianceChecker::new();
        let items = vec![create_test_poam_item()];
        let results = checker.assess_compliance(&items).unwrap();
        
        assert!(!results.is_empty());
        assert!(results.iter().all(|r| r.standard == "OSCAL"));
    }

    #[test]
    fn test_regulatory_compliance_checker() {
        let checker = RegulatoryComplianceChecker::new();
        let items = vec![create_test_poam_item()];
        let results = checker.assess_compliance(&items).unwrap();
        
        // Regulatory checks might be empty by default
        assert!(results.iter().all(|r| r.standard == "Regulatory"));
    }

    #[test]
    fn test_compliance_config_default() {
        let config = ComplianceConfig::default();
        assert!(config.enable_fedramp_checks);
        assert!(config.enable_oscal_checks);
        assert!(config.enable_regulatory_checks);
        assert_eq!(config.min_compliance_rate, 0.9);
        assert!(!config.fedramp_required_fields.is_empty());
    }

    #[test]
    fn test_compliance_check_type_methods() {
        let check_type = ComplianceCheckType::FedRampRequiredFields;
        assert_eq!(check_type.standard(), "FedRAMP");
        assert_eq!(check_type.check_name(), "fedramp_required_fields");
        assert_eq!(check_type.default_severity(), QualitySeverity::High);
    }

    #[test]
    fn test_compliance_statistics() {
        let mut stats = ComplianceStatistics::new();
        assert_eq!(stats.total_items, 0);
        assert_eq!(stats.total_checks, 0);
        assert_eq!(stats.overall_compliance_rate, 0.0);
        
        // Test threshold checking
        stats.overall_compliance_rate = 0.95;
        assert!(stats.meets_threshold(0.9));
        assert!(!stats.meets_threshold(0.99));
    }

    #[test]
    fn test_oscal_uuid_validation() {
        assert!(OscalComplianceChecker::validate_uuid_format("550e8400-e29b-41d4-a716-446655440000"));
        assert!(!OscalComplianceChecker::validate_uuid_format("invalid-uuid"));
        assert!(!OscalComplianceChecker::validate_uuid_format(""));
    }

    #[test]
    fn test_oscal_date_validation() {
        assert!(OscalComplianceChecker::validate_iso8601_date("2024-12-31T23:59:59Z"));
        assert!(OscalComplianceChecker::validate_iso8601_date("2024-01-01T00:00:00+00:00"));
        assert!(!OscalComplianceChecker::validate_iso8601_date("2024-12-31"));
        assert!(!OscalComplianceChecker::validate_iso8601_date("invalid-date"));
    }

    #[test]
    fn test_oscal_status_normalization() {
        assert_eq!(OscalComplianceChecker::normalize_status("Open"), "open");
        assert_eq!(OscalComplianceChecker::normalize_status("In Progress"), "in-progress");
        assert_eq!(OscalComplianceChecker::normalize_status("Complete"), "completed");
        assert_eq!(OscalComplianceChecker::normalize_status("Unknown"), "Unknown");
    }

    #[test]
    fn test_regulatory_rule_creation() {
        let fisma_rule = RegulatoryComplianceChecker::create_fisma_rule();
        assert_eq!(fisma_rule.standard, "FISMA");
        assert_eq!(fisma_rule.severity, QualitySeverity::High);
        assert!(!fisma_rule.required_fields.is_empty());

        let nist_rule = RegulatoryComplianceChecker::create_nist_rule();
        assert_eq!(nist_rule.standard, "NIST");
        assert_eq!(nist_rule.severity, QualitySeverity::High);
    }

    #[test]
    fn test_compliance_assessor_with_statistics() {
        let assessor = ComplianceAssessor::new();
        let items = vec![create_test_poam_item()];
        let (result, stats) = assessor.assess_with_statistics(&items).unwrap();
        
        assert!(result.score >= 0.0 && result.score <= 1.0);
        assert!(stats.total_checks > 0);
        assert!(stats.assessment_duration_ms > 0);
    }

    #[test]
    fn test_compliance_summary_generation() {
        let assessor = ComplianceAssessor::new();
        let items = vec![create_test_poam_item()];
        let result = assessor.assess(&items).unwrap();
        let summary = assessor.get_compliance_summary(&result);
        
        assert!(summary.contains_key("FedRAMP"));
        assert!(summary.contains_key("OSCAL"));
    }

    #[test]
    fn test_compliance_threshold_checking() {
        let assessor = ComplianceAssessor::new();
        let items = vec![create_test_poam_item()];
        let result = assessor.assess(&items).unwrap();
        
        // Test threshold checking
        let meets_threshold = assessor.meets_compliance_threshold(&result);
        assert_eq!(meets_threshold, result.score >= 0.9);
    }
}
