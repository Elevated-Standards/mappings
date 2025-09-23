//! Modified: 2025-09-23

//! Validation helper functions and utilities
//!
//! This module contains helper functions for validation operations including
//! column matching, data validation, regex compilation, and suggestion generation.

use fedramp_core::{Result, Error};
use regex::Regex;
use tracing::warn;
use crate::fuzzy::FuzzyMatcher;

use super::types::*;
use super::super::types::*;

impl ColumnValidator {
    /// Find a matching column for a validation rule
    pub(crate) fn find_matching_column(&self, rule: &ColumnValidationRule, headers: &[String]) -> Option<String> {
        // Try exact matches first
        for column_name in &rule.column_names {
            if headers.contains(column_name) {
                return Some(column_name.clone());
            }
        }
        
        // Try case-insensitive matches
        for column_name in &rule.column_names {
            for header in headers {
                if header.to_lowercase() == column_name.to_lowercase() {
                    return Some(header.clone());
                }
            }
        }
        
        // Try fuzzy matching as last resort
        let fuzzy_matcher = FuzzyMatcher::new();
        for column_name in &rule.column_names {
            for header in headers {
                if fuzzy_matcher.similarity(column_name, header) > 0.8 {
                    return Some(header.clone());
                }
            }
        }
        
        None
    }

    /// Validate data in a column according to the rule
    pub(crate) fn validate_column_data(
        &mut self,
        rule: &ColumnValidationRule,
        data: &[&serde_json::Value],
    ) -> Result<ValidationResult> {
        let mut passed = true;
        let mut messages = Vec::new();

        // Check for empty data
        if data.is_empty() {
            return Ok(ValidationResult {
                field_name: rule.field_id.clone(),
                rule_type: "presence".to_string(),
                passed: !rule.required,
                message: if rule.required {
                    Some("Required field has no data".to_string())
                } else {
                    None
                },
                severity: if rule.required { "error".to_string() } else { "info".to_string() },
            });
        }

        // Validate based on validation type
        if let Some(validation_type) = &rule.validation_type {
            match validation_type.as_str() {
                "enumeration" => {
                    if let Some(allowed_values) = &rule.allowed_values {
                        for value in data {
                            if let Some(str_value) = value.as_str() {
                                if !allowed_values.contains(&str_value.to_string()) {
                                    passed = false;
                                    messages.push(format!("Value '{}' not in allowed values", str_value));
                                }
                            }
                        }
                    }
                }
                "format" => {
                    if let Some(pattern) = &rule.pattern {
                        let regex = self.get_or_compile_regex(pattern)?;
                        for value in data {
                            if let Some(str_value) = value.as_str() {
                                if !regex.is_match(str_value) {
                                    passed = false;
                                    messages.push(format!("Value '{}' does not match required format", str_value));
                                }
                            }
                        }
                    }
                }
                _ => {
                    warn!("Unknown validation type: {}", validation_type);
                }
            }
        }

        Ok(ValidationResult {
            field_name: rule.field_id.clone(),
            rule_type: rule.validation_type.clone().unwrap_or_else(|| "general".to_string()),
            passed,
            message: if messages.is_empty() { None } else { Some(messages.join("; ")) },
            severity: if passed { "info".to_string() } else { "error".to_string() },
        })
    }

    /// Get or compile a regex pattern
    pub(crate) fn get_or_compile_regex(&mut self, pattern: &str) -> Result<&Regex> {
        if !self.regex_cache.contains_key(pattern) {
            let regex = Regex::new(pattern)
                .map_err(|e| Error::ValidationError(format!("Invalid regex pattern '{}': {}", pattern, e)))?;
            self.regex_cache.insert(pattern.to_string(), regex);
        }
        
        Ok(self.regex_cache.get(pattern).unwrap())
    }

    /// Suggest alternative column names for a missing field
    pub(crate) fn suggest_alternatives(&self, field_id: &str, headers: &[String]) -> Vec<String> {
        let fuzzy_matcher = FuzzyMatcher::new();
        let mut suggestions = Vec::new();
        
        for header in headers {
            let similarity = fuzzy_matcher.similarity(field_id, header);
            if similarity > 0.5 {
                suggestions.push(header.clone());
            }
        }
        
        suggestions.sort_by(|a, b| {
            let sim_a = fuzzy_matcher.similarity(field_id, a);
            let sim_b = fuzzy_matcher.similarity(field_id, b);
            sim_b.partial_cmp(&sim_a).unwrap_or(std::cmp::Ordering::Equal)
        });
        
        suggestions.truncate(3); // Limit to top 3 suggestions
        suggestions
    }

    /// Perform cross-field validation
    pub(crate) fn validate_cross_fields(
        &self,
        _field_results: &[ColumnValidationResult],
        _data: &[Vec<serde_json::Value>],
    ) -> Result<Vec<CrossFieldValidationResult>> {
        // Placeholder for cross-field validation logic
        // This would implement business rules that span multiple fields
        Ok(Vec::new())
    }

    /// Calculate validation metrics
    pub(crate) fn calculate_validation_metrics(
        &self,
        field_results: &[ColumnValidationResult],
        missing_required: &[RequiredFieldInfo],
    ) -> ValidationMetrics {
        let total_fields = field_results.len();
        let valid_fields = field_results.iter().filter(|r| r.passed).count();
        let invalid_fields = total_fields - valid_fields;
        let warning_count = field_results.iter()
            .filter(|r| r.severity == ValidationSeverity::Warning)
            .count();
        let error_count = field_results.iter()
            .filter(|r| r.severity == ValidationSeverity::Error)
            .count() + missing_required.len();
        
        let validation_score = if total_fields > 0 {
            valid_fields as f64 / total_fields as f64
        } else {
            1.0
        };

        ValidationMetrics {
            total_fields,
            required_fields: field_results.iter()
                .filter(|r| matches!(r.status, ValidationStatus::MissingRequired | ValidationStatus::Valid))
                .count(),
            optional_fields: field_results.iter()
                .filter(|r| matches!(r.status, ValidationStatus::MissingOptional))
                .count(),
            valid_fields,
            missing_required_count: missing_required.len(),
            invalid_fields,
            warning_count,
            error_count,
            validation_score,
        }
    }

    /// Add a custom validator function
    pub fn add_custom_validator(
        &mut self,
        name: String,
        validator: fn(&[serde_json::Value]) -> Result<(ValidationResult, f64)>,
    ) {
        self.custom_validators.insert(name, validator);
    }

    /// Remove a custom validator function
    pub fn remove_custom_validator(&mut self, name: &str) -> Option<fn(&[serde_json::Value]) -> Result<(ValidationResult, f64)>> {
        self.custom_validators.remove(name)
    }

    /// Get all custom validator names
    pub fn get_custom_validator_names(&self) -> Vec<&String> {
        self.custom_validators.keys().collect()
    }

    /// Set minimum quality threshold
    pub fn set_min_quality_threshold(&mut self, threshold: f64) {
        self.min_quality_threshold = threshold.clamp(0.0, 1.0);
    }

    /// Get minimum quality threshold
    pub fn get_min_quality_threshold(&self) -> f64 {
        self.min_quality_threshold
    }

    /// Set performance target in milliseconds
    pub fn set_performance_target_ms(&mut self, target_ms: u64) {
        self.performance_target_ms = target_ms;
    }

    /// Get performance target in milliseconds
    pub fn get_performance_target_ms(&self) -> u64 {
        self.performance_target_ms
    }

    /// Clear regex cache
    pub fn clear_regex_cache(&mut self) {
        self.regex_cache.clear();
    }

    /// Get regex cache size
    pub fn get_regex_cache_size(&self) -> usize {
        self.regex_cache.len()
    }
}

/// Utility functions for validation operations
pub mod validation_utils {
    use super::*;

    /// Check if a value matches a data type
    pub fn validate_data_type(value: &serde_json::Value, expected_type: &str) -> bool {
        match expected_type.to_lowercase().as_str() {
            "string" => value.is_string(),
            "number" | "integer" => value.is_number(),
            "boolean" => value.is_boolean(),
            "array" => value.is_array(),
            "object" => value.is_object(),
            "null" => value.is_null(),
            _ => false,
        }
    }

    /// Extract string value from JSON value
    pub fn extract_string_value(value: &serde_json::Value) -> Option<String> {
        match value {
            serde_json::Value::String(s) => Some(s.clone()),
            serde_json::Value::Number(n) => Some(n.to_string()),
            serde_json::Value::Bool(b) => Some(b.to_string()),
            _ => None,
        }
    }

    /// Check if a string value is empty or whitespace only
    pub fn is_empty_or_whitespace(value: &str) -> bool {
        value.trim().is_empty()
    }

    /// Normalize string value for comparison
    pub fn normalize_string(value: &str) -> String {
        value.trim().to_lowercase()
    }

    /// Calculate similarity score between two strings
    pub fn calculate_similarity(a: &str, b: &str) -> f64 {
        let fuzzy_matcher = FuzzyMatcher::new();
        fuzzy_matcher.similarity(a, b)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mapping::MappingConfiguration;

    #[test]
    fn test_column_validator_creation() {
        let config = MappingConfiguration::default();
        let validator = ColumnValidator::new(config);
        assert_eq!(validator.min_quality_threshold, 0.7);
        assert_eq!(validator.performance_target_ms, 100);
    }

    #[test]
    fn test_validation_utils_data_type() {
        use validation_utils::*;
        
        assert!(validate_data_type(&serde_json::Value::String("test".to_string()), "string"));
        assert!(validate_data_type(&serde_json::Value::Number(serde_json::Number::from(42)), "number"));
        assert!(validate_data_type(&serde_json::Value::Bool(true), "boolean"));
        assert!(!validate_data_type(&serde_json::Value::String("test".to_string()), "number"));
    }

    #[test]
    fn test_validation_utils_string_extraction() {
        use validation_utils::*;
        
        assert_eq!(extract_string_value(&serde_json::Value::String("test".to_string())), Some("test".to_string()));
        assert_eq!(extract_string_value(&serde_json::Value::Number(serde_json::Number::from(42))), Some("42".to_string()));
        assert_eq!(extract_string_value(&serde_json::Value::Bool(true)), Some("true".to_string()));
        assert_eq!(extract_string_value(&serde_json::Value::Null), None);
    }

    #[test]
    fn test_validation_utils_empty_check() {
        use validation_utils::*;
        
        assert!(is_empty_or_whitespace(""));
        assert!(is_empty_or_whitespace("   "));
        assert!(is_empty_or_whitespace("\t\n"));
        assert!(!is_empty_or_whitespace("test"));
        assert!(!is_empty_or_whitespace("  test  "));
    }
}
