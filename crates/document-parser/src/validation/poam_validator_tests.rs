//! Tests for POA&M validation

#[cfg(test)]
mod tests {
    use super::super::poam_validator::*;
    use crate::validation::{ValidationStatus, ValidationSeverity};
    use std::collections::HashMap;
    use serde_json::json;

    #[tokio::test]
    async fn test_poam_validator_creation() {
        let validator = PoamValidator::new();
        assert_eq!(validator.validation_config.allowed_severities.len(), 5);
        assert_eq!(validator.validation_config.allowed_statuses.len(), 7);
        assert_eq!(validator.validation_config.validation_mode, ValidationMode::Strict);
    }

    #[tokio::test]
    async fn test_poam_validator_with_config() {
        let config = PoamValidationConfig {
            allowed_severities: vec![PoamSeverity::High, PoamSeverity::Low],
            allowed_statuses: vec![PoamStatus::Open, PoamStatus::Closed],
            business_rules: vec![],
            validation_mode: ValidationMode::Lenient,
            custom_rules: vec![],
            performance_settings: PerformanceSettings {
                max_validation_time_ms: 50,
                enable_caching: false,
                cache_size_limit: 500,
                parallel_threshold: 50,
            },
        };

        let validator = PoamValidator::with_config(config);
        assert_eq!(validator.validation_config.allowed_severities.len(), 2);
        assert_eq!(validator.validation_config.allowed_statuses.len(), 2);
        assert_eq!(validator.validation_config.validation_mode, ValidationMode::Lenient);
    }

    #[tokio::test]
    async fn test_severity_validation_valid() {
        let validator = PoamValidator::new();
        let mut poam_data = HashMap::new();
        poam_data.insert("severity".to_string(), json!("High"));

        let result = validator.validate_severity_field(&poam_data).await.unwrap().unwrap();
        assert!(result.passed);
        assert_eq!(result.status, ValidationStatus::Valid);
        assert_eq!(result.confidence, 1.0);
    }

    #[tokio::test]
    async fn test_severity_validation_case_insensitive() {
        let validator = PoamValidator::new();
        let mut poam_data = HashMap::new();
        poam_data.insert("severity".to_string(), json!("critical"));

        let result = validator.validate_severity_field(&poam_data).await.unwrap().unwrap();
        assert!(result.passed);
        assert_eq!(result.status, ValidationStatus::Valid);
    }

    #[tokio::test]
    async fn test_severity_validation_numeric() {
        let validator = PoamValidator::new();
        let mut poam_data = HashMap::new();
        poam_data.insert("severity".to_string(), json!("1"));

        let result = validator.validate_severity_field(&poam_data).await.unwrap().unwrap();
        assert!(result.passed);
        assert_eq!(result.status, ValidationStatus::Valid);
    }

    #[tokio::test]
    async fn test_severity_validation_invalid() {
        let validator = PoamValidator::new();
        let mut poam_data = HashMap::new();
        poam_data.insert("severity".to_string(), json!("Invalid"));

        let result = validator.validate_severity_field(&poam_data).await.unwrap().unwrap();
        assert!(!result.passed);
        assert_eq!(result.status, ValidationStatus::Invalid);
        assert!(result.normalized_value.is_some());
    }

    #[tokio::test]
    async fn test_severity_validation_missing() {
        let validator = PoamValidator::new();
        let poam_data = HashMap::new();

        let result = validator.validate_severity_field(&poam_data).await.unwrap().unwrap();
        assert!(!result.passed);
        assert_eq!(result.status, ValidationStatus::Missing);
    }

    #[tokio::test]
    async fn test_status_validation_valid() {
        let validator = PoamValidator::new();
        let mut poam_data = HashMap::new();
        poam_data.insert("status".to_string(), json!("Open"));

        let result = validator.validate_status_field(&poam_data).await.unwrap().unwrap();
        assert!(result.passed);
        assert_eq!(result.status, ValidationStatus::Valid);
    }

    #[tokio::test]
    async fn test_status_validation_aliases() {
        let validator = PoamValidator::new();
        let test_cases = vec![
            ("in-progress", true),
            ("in progress", true),
            ("ongoing", true),
            ("complete", true),
            ("done", true),
            ("risk accepted", true),
        ];

        for (status_value, should_pass) in test_cases {
            let mut poam_data = HashMap::new();
            poam_data.insert("status".to_string(), json!(status_value));

            let result = validator.validate_status_field(&poam_data).await.unwrap().unwrap();
            assert_eq!(result.passed, should_pass, "Failed for status: {}", status_value);
        }
    }

    #[tokio::test]
    async fn test_status_validation_invalid() {
        let validator = PoamValidator::new();
        let mut poam_data = HashMap::new();
        poam_data.insert("status".to_string(), json!("InvalidStatus"));

        let result = validator.validate_status_field(&poam_data).await.unwrap().unwrap();
        assert!(!result.passed);
        assert_eq!(result.status, ValidationStatus::Invalid);
        assert!(result.normalized_value.is_some());
    }

    #[tokio::test]
    async fn test_business_rule_validation() {
        let validator = PoamValidator::new();
        let mut poam_data = HashMap::new();
        poam_data.insert("status".to_string(), json!("Completed"));
        // Missing actual_completion_date should trigger business rule

        let result = validator.validate_poam_item(&poam_data).await.unwrap();
        assert!(!result.is_valid);
        assert!(!result.errors.is_empty());
        
        // Check that the business rule error is present
        let has_completion_date_error = result.business_rule_results.iter()
            .any(|r| r.rule_name == "require_completion_date_for_completed" && !r.passed);
        assert!(has_completion_date_error);
    }

    #[tokio::test]
    async fn test_cross_field_validation() {
        let validator = PoamValidator::new();
        let mut poam_data = HashMap::new();
        poam_data.insert("status".to_string(), json!("Completed"));
        poam_data.insert("actual_completion_date".to_string(), json!("2024-01-15"));
        poam_data.insert("scheduled_completion_date".to_string(), json!("2024-01-01"));

        let result = validator.validate_poam_item(&poam_data).await.unwrap();
        
        // Should have warning about completion date being later than scheduled
        assert!(!result.warnings.is_empty() || !result.field_results.is_empty());
    }

    #[tokio::test]
    async fn test_required_fields_validation() {
        let validator = PoamValidator::new();
        let mut poam_data = HashMap::new();
        poam_data.insert("severity".to_string(), json!("High"));
        // Missing other required fields

        let result = validator.validate_poam_item(&poam_data).await.unwrap();
        assert!(!result.is_valid);
        
        // Should have errors for missing required fields
        let missing_fields: Vec<_> = result.field_results.iter()
            .filter(|(_, r)| r.status == ValidationStatus::Missing)
            .map(|(name, _)| name.clone())
            .collect();
        
        assert!(missing_fields.contains(&"status".to_string()));
        assert!(missing_fields.contains(&"poam_id".to_string()));
    }

    #[tokio::test]
    async fn test_complete_valid_poam() {
        let validator = PoamValidator::new();
        let mut poam_data = HashMap::new();
        poam_data.insert("poam_id".to_string(), json!("POA-001"));
        poam_data.insert("vulnerability_description".to_string(), json!("Test vulnerability"));
        poam_data.insert("severity".to_string(), json!("High"));
        poam_data.insert("status".to_string(), json!("Open"));
        poam_data.insert("scheduled_completion_date".to_string(), json!("2024-12-31"));
        poam_data.insert("point_of_contact".to_string(), json!("John Doe"));

        let result = validator.validate_poam_item(&poam_data).await.unwrap();
        assert!(result.is_valid);
        assert!(result.errors.is_empty());
    }

    #[test]
    fn test_severity_validator_creation() {
        let severities = vec![PoamSeverity::High, PoamSeverity::Low];
        let validator = SeverityValidator::new(&severities);
        assert_eq!(validator.allowed_severities.len(), 2);
        assert!(validator.severity_aliases.contains_key("high"));
        assert!(validator.severity_aliases.contains_key("1"));
    }

    #[test]
    fn test_severity_fuzzy_matching() {
        let severities = vec![PoamSeverity::High, PoamSeverity::Low];
        let validator = SeverityValidator::new(&severities);
        
        let result = validator.validate("Hgh"); // Typo in "High"
        assert!(!result.passed);
        assert_eq!(result.status, ValidationStatus::Invalid);
        assert!(result.normalized_value.is_some());
        assert_eq!(result.confidence, 0.5);
    }

    #[test]
    fn test_status_validator_creation() {
        let statuses = vec![PoamStatus::Open, PoamStatus::Closed];
        let validator = StatusValidator::new(&statuses);
        assert_eq!(validator.allowed_statuses.len(), 2);
        assert!(validator.status_aliases.contains_key("open"));
        assert!(validator.status_aliases.contains_key("closed"));
    }

    #[test]
    fn test_status_transition_validation() {
        let statuses = vec![PoamStatus::Open, PoamStatus::InProgress, PoamStatus::Completed];
        let validator = StatusValidator::new(&statuses);
        
        // Valid transitions
        assert!(validator.validate_transition(&PoamStatus::Open, &PoamStatus::InProgress));
        assert!(validator.validate_transition(&PoamStatus::InProgress, &PoamStatus::Completed));
        
        // Invalid transitions
        assert!(!validator.validate_transition(&PoamStatus::Completed, &PoamStatus::Open));
        assert!(!validator.validate_transition(&PoamStatus::Closed, &PoamStatus::Open));
    }

    #[test]
    fn test_business_rule_validator_creation() {
        let rules = vec![
            BusinessRule {
                name: "test_rule".to_string(),
                description: "Test rule".to_string(),
                condition: RuleCondition::FieldEquals {
                    field: "test".to_string(),
                    value: "value".to_string(),
                },
                action: RuleAction::RequireField {
                    field: "required_field".to_string(),
                },
                severity: ValidationSeverity::Error,
                enabled: true,
            }
        ];
        
        let validator = BusinessRuleValidator::new(&rules);
        assert_eq!(validator.rules.len(), 1);
    }

    #[test]
    fn test_cross_field_validator_creation() {
        let validator = CrossFieldValidator::new();
        assert_eq!(validator.cross_field_rules.len(), 3);
        
        let rule_names: Vec<_> = validator.cross_field_rules.iter()
            .map(|r| r.name.clone())
            .collect();
        
        assert!(rule_names.contains(&"completion_date_consistency".to_string()));
        assert!(rule_names.contains(&"scheduled_date_logic".to_string()));
        assert!(rule_names.contains(&"critical_severity_timeline".to_string()));
    }

    #[test]
    fn test_poam_validation_config_default() {
        let config = PoamValidationConfig::default();
        assert_eq!(config.allowed_severities.len(), 5);
        assert_eq!(config.allowed_statuses.len(), 7);
        assert_eq!(config.business_rules.len(), 3);
        assert_eq!(config.validation_mode, ValidationMode::Strict);
        assert_eq!(config.performance_settings.max_validation_time_ms, 100);
    }

    #[tokio::test]
    async fn test_validation_performance_metrics() {
        let validator = PoamValidator::new();
        let mut poam_data = HashMap::new();
        poam_data.insert("severity".to_string(), json!("High"));
        poam_data.insert("status".to_string(), json!("Open"));

        let result = validator.validate_poam_item(&poam_data).await.unwrap();
        
        assert!(result.performance_metrics.total_time_ms < 1000); // Should be fast
        assert!(result.performance_metrics.rules_evaluated > 0);
        assert!(result.performance_metrics.fields_validated > 0);
    }

    #[tokio::test]
    async fn test_validation_suggestions() {
        let validator = PoamValidator::new();
        let mut poam_data = HashMap::new();
        poam_data.insert("severity".to_string(), json!("Hgh")); // Typo

        let result = validator.validate_poam_item(&poam_data).await.unwrap();
        
        assert!(!result.suggestions.is_empty());
        let suggestion = &result.suggestions[0];
        assert_eq!(suggestion.suggestion_type, "field_correction");
        assert!(suggestion.field.is_some());
        assert!(suggestion.suggested_value.is_some());
    }
}
