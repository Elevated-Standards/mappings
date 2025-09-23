// Modified: 2025-09-22

//! Validator implementations for field-level and document-level validation
//!
//! This module provides comprehensive validation capabilities including:
//! - Field-level validation for different data types
//! - Document-level validation with quality metrics
//! - Custom validation rules and patterns
//! - Performance tracking and metrics
//! - Validation helper utilities

pub mod types;
pub mod field_validators;
pub mod document_validator;
pub mod validation_helpers;

// Re-export main types for backward compatibility
pub use types::{
    ColumnValidator,
    DocumentValidator,
    DocumentValidationResult,
    ColumnValidationConfig,
    DocumentValidationConfig,
    ValidationPatterns,
    ValidityThresholds,
    ValidatorRegistry,
    ValidatorMetadata,
    ValidatorPerformanceInfo,
    MemoryUsage,
    CustomValidatorFn,
};

// Re-export validation helpers
pub use validation_helpers::ValidationHelpers;

// Re-export implementations (these are implemented via impl blocks in separate files)
// The actual struct definitions are in types.rs, implementations are in field_validators.rs and document_validator.rs

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mapping::MappingConfiguration;
    use crate::validation::types::DataType;
    use serde_json::Value;
    use std::collections::HashMap;

    #[test]
    fn test_column_validator_creation() {
        let mapping_config = MappingConfiguration::default();
        let validator = ColumnValidator::new(mapping_config);
        
        assert_eq!(validator.get_min_quality_threshold(), 0.8);
        assert_eq!(validator.get_performance_target_ms(), 50);
    }

    #[test]
    fn test_document_validator_creation() {
        let validator = DocumentValidator::new();
        
        assert_eq!(validator.get_min_quality_threshold(), 0.8);
        assert!(!validator.has_rules());
        assert_eq!(validator.get_rule_count(), 0);
    }

    #[test]
    fn test_validation_helpers_type_detection() {
        let values = vec![
            Value::String("test".to_string()),
            Value::String("another".to_string()),
            Value::Number(serde_json::Number::from(42)),
        ];
        
        let detected_type = ValidationHelpers::detect_data_type(&values);
        assert_eq!(detected_type, "String");
    }

    #[test]
    fn test_validation_helpers_date_validation() {
        assert!(ValidationHelpers::is_valid_date("2023-12-25"));
        assert!(ValidationHelpers::is_valid_date("12/25/2023"));
        assert!(!ValidationHelpers::is_valid_date("invalid-date"));
    }

    #[test]
    fn test_validation_helpers_email_validation() {
        assert!(ValidationHelpers::is_valid_email("test@example.com"));
        assert!(ValidationHelpers::is_valid_email("user.name+tag@domain.co.uk"));
        assert!(!ValidationHelpers::is_valid_email("invalid-email"));
        assert!(!ValidationHelpers::is_valid_email("@domain.com"));
    }

    #[test]
    fn test_validation_helpers_url_validation() {
        assert!(ValidationHelpers::is_valid_url("https://example.com"));
        assert!(ValidationHelpers::is_valid_url("http://test.org"));
        assert!(ValidationHelpers::is_valid_url("ftp://files.example.com"));
        assert!(!ValidationHelpers::is_valid_url("invalid-url"));
        assert!(!ValidationHelpers::is_valid_url("example.com"));
    }

    #[test]
    fn test_validation_helpers_ip_validation() {
        assert!(ValidationHelpers::is_valid_ip_address("192.168.1.1"));
        assert!(ValidationHelpers::is_valid_ip_address("10.0.0.1"));
        assert!(ValidationHelpers::is_valid_ip_address("255.255.255.255"));
        assert!(!ValidationHelpers::is_valid_ip_address("256.1.1.1"));
        assert!(!ValidationHelpers::is_valid_ip_address("192.168.1"));
        assert!(!ValidationHelpers::is_valid_ip_address("invalid-ip"));
    }

    #[test]
    fn test_validation_helpers_uuid_validation() {
        assert!(ValidationHelpers::is_valid_uuid("550e8400-e29b-41d4-a716-446655440000"));
        assert!(ValidationHelpers::is_valid_uuid("6ba7b810-9dad-11d1-80b4-00c04fd430c8"));
        assert!(!ValidationHelpers::is_valid_uuid("invalid-uuid"));
        assert!(!ValidationHelpers::is_valid_uuid("550e8400-e29b-41d4-a716"));
    }

    #[test]
    fn test_validation_helpers_empty_value_detection() {
        assert!(ValidationHelpers::is_empty_value(&Value::Null));
        assert!(ValidationHelpers::is_empty_value(&Value::String("".to_string())));
        assert!(ValidationHelpers::is_empty_value(&Value::String("   ".to_string())));
        assert!(ValidationHelpers::is_empty_value(&Value::Array(vec![])));
        assert!(ValidationHelpers::is_empty_value(&Value::Object(serde_json::Map::new())));
        
        assert!(!ValidationHelpers::is_empty_value(&Value::String("test".to_string())));
        assert!(!ValidationHelpers::is_empty_value(&Value::Number(serde_json::Number::from(42))));
        assert!(!ValidationHelpers::is_empty_value(&Value::Bool(true)));
    }

    #[test]
    fn test_validation_helpers_boolean_detection() {
        assert!(ValidationHelpers::is_boolean_string("true"));
        assert!(ValidationHelpers::is_boolean_string("false"));
        assert!(ValidationHelpers::is_boolean_string("yes"));
        assert!(ValidationHelpers::is_boolean_string("no"));
        assert!(ValidationHelpers::is_boolean_string("1"));
        assert!(ValidationHelpers::is_boolean_string("0"));
        assert!(ValidationHelpers::is_boolean_string("on"));
        assert!(ValidationHelpers::is_boolean_string("off"));
        assert!(ValidationHelpers::is_boolean_string("TRUE"));
        assert!(ValidationHelpers::is_boolean_string("False"));
        
        assert!(!ValidationHelpers::is_boolean_string("maybe"));
        assert!(!ValidationHelpers::is_boolean_string("2"));
        assert!(!ValidationHelpers::is_boolean_string(""));
    }

    #[test]
    fn test_validation_helpers_numeric_detection() {
        assert!(ValidationHelpers::is_numeric_string("42"));
        assert!(ValidationHelpers::is_numeric_string("3.14"));
        assert!(ValidationHelpers::is_numeric_string("-123"));
        assert!(ValidationHelpers::is_numeric_string("0.0"));
        
        assert!(!ValidationHelpers::is_numeric_string("abc"));
        assert!(!ValidationHelpers::is_numeric_string(""));
        assert!(!ValidationHelpers::is_numeric_string("12.34.56"));
    }

    #[test]
    fn test_validation_helpers_integer_detection() {
        assert!(ValidationHelpers::is_integer_string("42"));
        assert!(ValidationHelpers::is_integer_string("-123"));
        assert!(ValidationHelpers::is_integer_string("0"));
        
        assert!(!ValidationHelpers::is_integer_string("3.14"));
        assert!(!ValidationHelpers::is_integer_string("abc"));
        assert!(!ValidationHelpers::is_integer_string(""));
    }

    #[test]
    fn test_validation_helpers_length_validation() {
        assert!(ValidationHelpers::is_length_valid("test", Some(1), Some(10)));
        assert!(ValidationHelpers::is_length_valid("test", Some(4), Some(4)));
        assert!(!ValidationHelpers::is_length_valid("test", Some(5), None));
        assert!(!ValidationHelpers::is_length_valid("test", None, Some(3)));
        assert!(ValidationHelpers::is_length_valid("test", None, None));
    }

    #[test]
    fn test_validation_helpers_number_range_validation() {
        assert!(ValidationHelpers::is_number_in_range(5.0, Some(1.0), Some(10.0)));
        assert!(ValidationHelpers::is_number_in_range(5.0, Some(5.0), Some(5.0)));
        assert!(!ValidationHelpers::is_number_in_range(5.0, Some(6.0), None));
        assert!(!ValidationHelpers::is_number_in_range(5.0, None, Some(4.0)));
        assert!(ValidationHelpers::is_number_in_range(5.0, None, None));
    }

    #[test]
    fn test_validator_registry() {
        let mut registry = ValidatorRegistry::new();
        assert_eq!(registry.count(), 0);
        assert!(!registry.has_validator("test"));
        
        let metadata = ValidatorMetadata::new(
            "Test Validator".to_string(),
            "A test validator".to_string(),
            vec!["String".to_string()],
        );
        
        fn test_validator(_values: &[Value]) -> crate::Result<(crate::validation::types::ValidationStatus, String)> {
            Ok((crate::validation::types::ValidationStatus::Valid, "Test passed".to_string()))
        }
        
        registry.register_validator("test".to_string(), test_validator, metadata);
        assert_eq!(registry.count(), 1);
        assert!(registry.has_validator("test"));
        assert!(registry.get_validator("test").is_some());
        assert!(registry.get_metadata("test").is_some());
        
        let validators = registry.list_validators();
        assert_eq!(validators.len(), 1);
        assert!(validators.contains(&"test"));
        
        assert!(registry.remove_validator("test"));
        assert_eq!(registry.count(), 0);
        assert!(!registry.has_validator("test"));
    }

    #[test]
    fn test_column_validation_config_defaults() {
        let config = ColumnValidationConfig::default();
        assert_eq!(config.min_quality_threshold, 0.8);
        assert_eq!(config.performance_target_ms, 50);
        assert!(config.collect_samples);
        assert_eq!(config.max_sample_size, 5);
        assert_eq!(config.validity_thresholds.valid_threshold, 0.9);
        assert_eq!(config.validity_thresholds.invalid_threshold, 0.7);
    }

    #[test]
    fn test_document_validation_config_defaults() {
        let config = DocumentValidationConfig::default();
        assert_eq!(config.min_quality_threshold, 0.8);
        assert!(config.track_performance);
        assert_eq!(config.max_validation_time_ms, 5000);
        assert!(!config.fail_fast);
    }

    #[test]
    fn test_validation_patterns_defaults() {
        let patterns = ValidationPatterns::default();
        assert!(!patterns.date_patterns.is_empty());
        assert!(!patterns.email_pattern.is_empty());
        assert!(!patterns.url_patterns.is_empty());
        assert!(!patterns.ip_pattern.is_empty());
        assert!(!patterns.uuid_pattern.is_empty());
    }
}
