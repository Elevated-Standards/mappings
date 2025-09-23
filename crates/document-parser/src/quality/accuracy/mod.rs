//! Modified: 2025-01-22
//! 
//! Data Accuracy Validation Module
//! 
//! This module provides comprehensive accuracy validation for POA&M data including
//! format validation, value constraints, field analysis, and quality assessment.
//! 
//! ## Features
//! 
//! - **Format Validation**: UUID, date, email, and other format validations
//! - **Value Constraints**: Status, severity, and custom value validations
//! - **Field Analysis**: Detailed field-level accuracy statistics and error patterns
//! - **Quality Assessment**: Overall accuracy scoring and quality findings generation
//! - **Configurable Rules**: Customizable validation rules and thresholds
//! - **Comprehensive Reporting**: Detailed accuracy reports with remediation guidance
//! 
//! ## Usage
//! 
//! ```rust
//! use crate::quality::accuracy::{AccuracyValidator, AccuracyConfig};
//! use crate::poam::PoamItem;
//! 
//! // Create validator with default configuration
//! let validator = AccuracyValidator::new();
//! 
//! // Or with custom configuration
//! let config = AccuracyConfig::default();
//! let validator = AccuracyValidator::with_accuracy_config(config);
//! 
//! // Validate POA&M items
//! let poam_items: Vec<PoamItem> = vec![/* ... */];
//! let result = validator.validate(&poam_items)?;
//! 
//! println!("Accuracy Score: {:.2}", result.score);
//! println!("Findings: {}", result.findings.len());
//! ```
//! 
//! ## Module Structure
//! 
//! - `types` - Type definitions for accuracy validation results and configuration
//! - `validator` - Main AccuracyValidator implementation and coordination logic
//! - `rules` - Individual validation rule implementations (UUID, date, status, etc.)
//! - `analyzers` - Field analysis and statistics calculation functionality

pub mod types;
pub mod validator;
pub mod rules;
pub mod analyzers;

// Re-export main types for backward compatibility and convenience
pub use types::{
    AccuracyResult,
    FieldAccuracyStats,
    ValidationRuleResult,
    AccuracyConfig,
    AccuracyThresholds,
};

pub use validator::{
    AccuracyValidator,
    AccuracySummary,
};

pub use rules::{
    ValidationRuleExecutor,
    uuid_regex,
    email_regex,
};

pub use analyzers::{
    FieldAccuracyAnalyzer,
};

#[cfg(test)]
mod tests {
    use super::*;
    use crate::poam::PoamItem;
    use std::collections::HashMap;
    
    /// Create a test POA&M item with valid data
    fn create_valid_poam_item() -> PoamItem {
        PoamItem {
            uuid: "550e8400-e29b-41d4-a716-446655440000".to_string(),
            title: "Test POA&M Item".to_string(),
            description: "This is a test POA&M item for accuracy validation".to_string(),
            status: "Open".to_string(),
            severity: Some("High".to_string()),
            scheduled_completion_date: Some("2024-12-31".to_string()),
            actual_completion_date: None,
            responsible_entity: Some("Test Team".to_string()),
            resources_required: Some("Test resources".to_string()),
            milestones: Vec::new(),
            vendor_dependency: None,
            source_identifier: Some("TEST-001".to_string()),
            weakness_detection_source: Some("Manual Review".to_string()),
            weakness_source_identifier: Some("WS-001".to_string()),
            remediation_plan: Some("Test remediation plan".to_string()),
            risk_rating: Some("High".to_string()),
            threat_relevance: Some("High".to_string()),
            likelihood: Some("Medium".to_string()),
            impact: Some("High".to_string()),
            impact_description: Some("Test impact description".to_string()),
            residual_risk_level: Some("Medium".to_string()),
            recommendations: Some("Test recommendations".to_string()),
            metadata: HashMap::new(),
        }
    }
    
    /// Create a test POA&M item with invalid data
    fn create_invalid_poam_item() -> PoamItem {
        PoamItem {
            uuid: "invalid-uuid".to_string(),
            title: "".to_string(), // Too short
            description: "Short".to_string(), // Too short
            status: "InvalidStatus".to_string(),
            severity: Some("InvalidSeverity".to_string()),
            scheduled_completion_date: Some("invalid-date".to_string()),
            actual_completion_date: Some("2023-01-01".to_string()), // Before scheduled
            responsible_entity: Some("Test Team".to_string()),
            resources_required: Some("Test resources".to_string()),
            milestones: Vec::new(),
            vendor_dependency: None,
            source_identifier: Some("TEST-002".to_string()),
            weakness_detection_source: Some("Manual Review".to_string()),
            weakness_source_identifier: Some("WS-002".to_string()),
            remediation_plan: Some("Test remediation plan".to_string()),
            risk_rating: Some("High".to_string()),
            threat_relevance: Some("High".to_string()),
            likelihood: Some("Medium".to_string()),
            impact: Some("High".to_string()),
            impact_description: Some("Test impact description".to_string()),
            residual_risk_level: Some("Medium".to_string()),
            recommendations: Some("Test recommendations".to_string()),
            metadata: HashMap::new(),
        }
    }
    
    #[test]
    fn test_accuracy_validator_creation() {
        let validator = AccuracyValidator::new();
        let summary = validator.get_accuracy_summary();
        
        assert!(summary.total_validation_rules > 0);
        assert!(!summary.analyzed_fields.is_empty());
        assert!(summary.strict_uuid_validation);
        assert!(summary.date_logic_validation);
    }
    
    #[test]
    fn test_accuracy_validation_empty_items() {
        let validator = AccuracyValidator::new();
        let result = validator.validate(&[]).unwrap();
        
        assert_eq!(result.score, 1.0);
        assert!(result.findings.is_empty());
        assert!(result.field_accuracy.is_empty());
        assert!(result.rule_results.is_empty());
    }
    
    #[test]
    fn test_accuracy_validation_valid_items() {
        let validator = AccuracyValidator::new();
        let items = vec![create_valid_poam_item()];
        let result = validator.validate(&items).unwrap();
        
        assert!(result.score > 0.8); // Should have high accuracy
        assert!(!result.field_accuracy.is_empty());
        assert!(!result.rule_results.is_empty());
        
        // Check that all validation rules passed
        for rule_result in &result.rule_results {
            assert!(rule_result.success_rate > 0.0);
        }
    }
    
    #[test]
    fn test_accuracy_validation_invalid_items() {
        let validator = AccuracyValidator::new();
        let items = vec![create_invalid_poam_item()];
        let result = validator.validate(&items).unwrap();
        
        assert!(result.score < 0.8); // Should have low accuracy
        assert!(!result.findings.is_empty()); // Should have findings
        assert!(!result.field_accuracy.is_empty());
        assert!(!result.rule_results.is_empty());
        
        // Check that some validation rules failed
        let failed_rules = result.rule_results.iter()
            .filter(|r| r.failed_items > 0)
            .count();
        assert!(failed_rules > 0);
    }
    
    #[test]
    fn test_accuracy_validation_mixed_items() {
        let validator = AccuracyValidator::new();
        let items = vec![
            create_valid_poam_item(),
            create_invalid_poam_item(),
        ];
        let result = validator.validate(&items).unwrap();
        
        assert!(result.score > 0.0 && result.score < 1.0); // Should be between 0 and 1
        assert!(!result.field_accuracy.is_empty());
        assert!(!result.rule_results.is_empty());
        
        // Check that we have both passed and failed items
        let total_passed: usize = result.rule_results.iter()
            .map(|r| r.passed_items)
            .sum();
        let total_failed: usize = result.rule_results.iter()
            .map(|r| r.failed_items)
            .sum();
        
        assert!(total_passed > 0);
        assert!(total_failed > 0);
    }
    
    #[test]
    fn test_field_accuracy_stats() {
        let validator = AccuracyValidator::new();
        let items = vec![create_valid_poam_item(), create_invalid_poam_item()];
        let result = validator.validate(&items).unwrap();
        
        // Check that we have field accuracy stats for expected fields
        assert!(result.field_accuracy.contains_key("uuid"));
        assert!(result.field_accuracy.contains_key("status"));
        assert!(result.field_accuracy.contains_key("title"));
        assert!(result.field_accuracy.contains_key("description"));
        
        // Check UUID field stats
        let uuid_stats = &result.field_accuracy["uuid"];
        assert_eq!(uuid_stats.populated_items, 2);
        assert_eq!(uuid_stats.valid_items, 1); // Only one valid UUID
        assert_eq!(uuid_stats.invalid_items, 1);
        assert_eq!(uuid_stats.accuracy_percentage, 50.0);
    }
    
    #[test]
    fn test_validation_rule_results() {
        let validator = AccuracyValidator::new();
        let items = vec![create_invalid_poam_item()];
        let result = validator.validate(&items).unwrap();
        
        // Find the UUID format validation rule
        let uuid_rule = result.rule_results.iter()
            .find(|r| r.rule_name == "uuid_format")
            .expect("UUID format rule should be present");
        
        assert_eq!(uuid_rule.passed_items, 0);
        assert_eq!(uuid_rule.failed_items, 1);
        assert_eq!(uuid_rule.success_rate, 0.0);
        assert_eq!(uuid_rule.failed_item_uuids.len(), 1);
        assert_eq!(uuid_rule.failed_item_uuids[0], "invalid-uuid");
    }
    
    #[test]
    fn test_accuracy_config_customization() {
        let mut config = AccuracyConfig::default();
        config.valid_statuses = vec!["Custom Status".to_string()];
        config.strict_uuid_validation = false;
        
        let validator = AccuracyValidator::with_accuracy_config(config);
        let summary = validator.get_accuracy_summary();
        
        assert!(!summary.strict_uuid_validation);
        assert_eq!(summary.valid_status_count, 1);
    }
    
    #[test]
    fn test_single_item_validation() {
        let validator = AccuracyValidator::new();
        let item = create_valid_poam_item();
        let result = validator.validate_single_item(&item).unwrap();
        
        assert!(result.score > 0.8);
        assert!(!result.field_accuracy.is_empty());
        assert!(!result.rule_results.is_empty());
    }
    
    #[test]
    fn test_accuracy_result_methods() {
        let mut result = AccuracyResult::new();
        assert!(result.is_high_quality());
        assert_eq!(result.critical_issues_count(), 0);
        assert_eq!(result.total_failed_items(), 0);
        
        // Add a critical finding
        result.findings.push(crate::quality::QualityFinding {
            id: "test".to_string(),
            severity: crate::quality::QualitySeverity::Critical,
            category: crate::quality::QualityCategory::Accuracy,
            description: "Test critical finding".to_string(),
            affected_items: vec!["item1".to_string()],
            impact_assessment: "Test impact".to_string(),
            recommendation: "Test recommendation".to_string(),
            location: None,
            metadata: HashMap::new(),
        });
        
        assert!(!result.is_high_quality());
        assert_eq!(result.critical_issues_count(), 1);
    }
}
