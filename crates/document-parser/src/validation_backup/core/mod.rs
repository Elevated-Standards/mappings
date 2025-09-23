//! Modified: 2025-09-23

//! Core validation module
//!
//! This module provides the core validation functionality including column validation,
//! document validation, and validation helper utilities. It contains the main validation
//! logic and document validator implementation.

// Module declarations
pub mod types;
pub mod column_validator;
pub mod document_validator;
pub mod validation_helpers;

// Re-export all public types and functions for backward compatibility
pub use types::*;
pub use column_validator::*;
pub use document_validator::*;
pub use validation_helpers::*;

// Re-export validation utilities
pub use validation_helpers::validation_utils;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mapping::MappingConfiguration;

    #[test]
    fn test_column_validator_integration() {
        let config = MappingConfiguration::default();
        let mut validator = ColumnValidator::new(config);
        
        // Test basic functionality
        assert_eq!(validator.get_min_quality_threshold(), 0.7);
        assert_eq!(validator.get_performance_target_ms(), 100);
        assert_eq!(validator.get_regex_cache_size(), 0);
        
        // Test threshold setting
        validator.set_min_quality_threshold(0.8);
        assert_eq!(validator.get_min_quality_threshold(), 0.8);
        
        // Test performance target setting
        validator.set_performance_target_ms(200);
        assert_eq!(validator.get_performance_target_ms(), 200);
    }

    #[test]
    fn test_document_validator_integration() {
        let mut validator = DocumentValidator::new();
        
        // Test basic functionality
        assert!(validator.get_document_types().is_empty());
        assert!(!validator.is_document_type_supported("test"));
        
        // Add some rules
        let rules = vec![
            crate::validation_backup::types::ValidationRule {
                field_id: Some("test_field".to_string()),
                rule_type: Some("required".to_string()),
                required: Some(true),
                ..Default::default()
            }
        ];
        
        validator.add_rules("test_doc".to_string(), rules);
        assert_eq!(validator.get_document_types().len(), 1);
        assert!(validator.is_document_type_supported("test_doc"));
        
        // Test validation summary
        let summary = validator.get_validation_summary("test_doc").unwrap();
        assert_eq!(summary.document_type, "test_doc");
        assert_eq!(summary.total_rules, 1);
        assert_eq!(summary.required_rules, 1);
        assert_eq!(summary.optional_rules, 0);
    }

    #[test]
    fn test_validation_helpers_integration() {
        use validation_utils::*;
        
        // Test data type validation
        let string_value = serde_json::Value::String("test".to_string());
        let number_value = serde_json::Value::Number(serde_json::Number::from(42));
        let bool_value = serde_json::Value::Bool(true);
        
        assert!(validate_data_type(&string_value, "string"));
        assert!(validate_data_type(&number_value, "number"));
        assert!(validate_data_type(&bool_value, "boolean"));
        assert!(!validate_data_type(&string_value, "number"));
        
        // Test string extraction
        assert_eq!(extract_string_value(&string_value), Some("test".to_string()));
        assert_eq!(extract_string_value(&number_value), Some("42".to_string()));
        assert_eq!(extract_string_value(&bool_value), Some("true".to_string()));
        
        // Test empty/whitespace checking
        assert!(is_empty_or_whitespace(""));
        assert!(is_empty_or_whitespace("   "));
        assert!(!is_empty_or_whitespace("test"));
        
        // Test string normalization
        assert_eq!(normalize_string("  Test  "), "test");
        assert_eq!(normalize_string("UPPER"), "upper");
    }

    #[test]
    fn test_column_validation_rule_creation() {
        let rule = ColumnValidationRule {
            field_id: "test_field".to_string(),
            column_names: vec!["Test Column".to_string()],
            oscal_field: "test.field".to_string(),
            required: true,
            validation_type: Some("format".to_string()),
            allowed_values: None,
            pattern: Some(r"^[A-Za-z]+$".to_string()),
            data_type: Some("string".to_string()),
            conditional: None,
        };
        
        assert_eq!(rule.field_id, "test_field");
        assert_eq!(rule.column_names.len(), 1);
        assert!(rule.required);
        assert_eq!(rule.validation_type, Some("format".to_string()));
        assert!(rule.pattern.is_some());
    }

    #[test]
    fn test_validation_metrics_default() {
        let metrics = ValidationMetrics::default();
        
        assert_eq!(metrics.total_fields, 0);
        assert_eq!(metrics.valid_fields, 0);
        assert_eq!(metrics.error_count, 0);
        assert_eq!(metrics.validation_score, 0.0);
    }

    #[test]
    fn test_conditional_validation() {
        let conditional = ConditionalValidation {
            depends_on: "status".to_string(),
            required_value: "active".to_string(),
            alternative_rule: None,
        };
        
        assert_eq!(conditional.depends_on, "status");
        assert_eq!(conditional.required_value, "active");
        assert!(conditional.alternative_rule.is_none());
    }

    #[test]
    fn test_cross_field_validation_result() {
        let result = CrossFieldValidationResult {
            rule_id: "cross_field_rule".to_string(),
            fields: vec!["field1".to_string(), "field2".to_string()],
            passed: true,
            message: "Validation passed".to_string(),
            severity: crate::validation_backup::types::ValidationSeverity::Info,
            execution_time: std::time::Duration::from_millis(10),
        };
        
        assert_eq!(result.rule_id, "cross_field_rule");
        assert_eq!(result.fields.len(), 2);
        assert!(result.passed);
    }

    #[test]
    fn test_required_field_info() {
        let info = RequiredFieldInfo {
            field_id: "required_field".to_string(),
            expected_columns: vec!["Column A".to_string(), "Column B".to_string()],
            oscal_field: "required.field".to_string(),
            description: "This field is required".to_string(),
            alternatives: vec!["Similar Column".to_string()],
        };
        
        assert_eq!(info.field_id, "required_field");
        assert_eq!(info.expected_columns.len(), 2);
        assert_eq!(info.alternatives.len(), 1);
    }

    #[test]
    fn test_type_mismatch_info() {
        let info = TypeMismatchInfo {
            field_id: "numeric_field".to_string(),
            column_name: "Number Column".to_string(),
            expected_type: "number".to_string(),
            actual_type: "string".to_string(),
            sample_values: vec!["abc".to_string(), "def".to_string()],
            suggested_conversion: Some("Parse as number".to_string()),
        };
        
        assert_eq!(info.field_id, "numeric_field");
        assert_eq!(info.expected_type, "number");
        assert_eq!(info.actual_type, "string");
        assert_eq!(info.sample_values.len(), 2);
        assert!(info.suggested_conversion.is_some());
    }

    #[test]
    fn test_enumeration_failure_info() {
        let info = EnumerationFailureInfo {
            field_id: "status_field".to_string(),
            column_name: "Status".to_string(),
            invalid_values: vec!["invalid_status".to_string()],
            valid_values: vec!["active".to_string(), "inactive".to_string()],
            suggestions: vec!["Did you mean 'active'?".to_string()],
        };
        
        assert_eq!(info.field_id, "status_field");
        assert_eq!(info.invalid_values.len(), 1);
        assert_eq!(info.valid_values.len(), 2);
        assert_eq!(info.suggestions.len(), 1);
    }

    #[test]
    fn test_validation_summary() {
        let summary = ValidationSummary {
            document_type: "test_document".to_string(),
            total_rules: 5,
            required_rules: 3,
            optional_rules: 2,
            rule_types: vec!["required".to_string(), "format".to_string()],
        };
        
        assert_eq!(summary.document_type, "test_document");
        assert_eq!(summary.total_rules, 5);
        assert_eq!(summary.required_rules, 3);
        assert_eq!(summary.optional_rules, 2);
        assert_eq!(summary.rule_types.len(), 2);
    }

    #[test]
    fn test_validation_rules_statistics() {
        let stats = ValidationRulesStatistics {
            total_document_types: 3,
            total_rules: 15,
            total_required_rules: 10,
            total_optional_rules: 5,
            unique_rule_types: 4,
            rule_types: vec!["required".to_string(), "format".to_string(), "enumeration".to_string(), "custom".to_string()],
        };
        
        assert_eq!(stats.total_document_types, 3);
        assert_eq!(stats.total_rules, 15);
        assert_eq!(stats.total_required_rules, 10);
        assert_eq!(stats.total_optional_rules, 5);
        assert_eq!(stats.unique_rule_types, 4);
        assert_eq!(stats.rule_types.len(), 4);
    }
}
