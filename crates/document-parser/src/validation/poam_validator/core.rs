// Modified: 2025-01-22

//! Core POA&M validator implementation
//!
//! This module provides the main PoamValidator that orchestrates all
//! validation components including field validation, business rules,
//! and cross-field validation.

use super::types::{
    PoamValidationConfig, PoamValidationResult, ValidationError, ValidationWarning,
    ValidationSuggestion, FieldValidationResult, BusinessRuleResult, ValidationPerformanceMetrics
};
use super::field_validation::{SeverityValidator, StatusValidator};
use super::business_rules::BusinessRuleValidator;
use super::cross_field::CrossFieldValidator;
use crate::validation::types::ValidationSeverity;
use fedramp_core::Result;
use serde_json::Value;
use std::collections::HashMap;
use std::time::Instant;
use tracing::{info, warn};

/// Comprehensive POA&M validator
#[derive(Debug, Clone)]
pub struct PoamValidator {
    /// Severity validator
    severity_validator: SeverityValidator,
    /// Status validator
    status_validator: StatusValidator,
    /// Business rule validator
    business_rule_validator: BusinessRuleValidator,
    /// Cross-field validator
    cross_field_validator: CrossFieldValidator,
    /// Validation configuration
    pub validation_config: PoamValidationConfig,
}

impl PoamValidator {
    /// Create a new POA&M validator with default configuration
    pub fn new() -> Self {
        let config = PoamValidationConfig::default();
        Self::with_config(config)
    }

    /// Create a new POA&M validator with custom configuration
    pub fn with_config(config: PoamValidationConfig) -> Self {
        let severity_validator = SeverityValidator::new(&config.allowed_severities);
        let status_validator = StatusValidator::new(&config.allowed_statuses);
        let business_rule_validator = BusinessRuleValidator::new(&config.business_rules);
        let cross_field_validator = CrossFieldValidator::new();

        Self {
            severity_validator,
            status_validator,
            business_rule_validator,
            cross_field_validator,
            validation_config: config,
        }
    }

    /// Validate a complete POA&M document
    pub fn validate_poam(&mut self, poam_data: &HashMap<String, Value>) -> Result<PoamValidationResult> {
        let start_time = Instant::now();
        let mut errors = Vec::new();
        let mut warnings = Vec::new();
        let mut suggestions = Vec::new();
        let mut field_results = Vec::new();

        info!("Starting POA&M validation for {} fields", poam_data.len());

        // 1. Validate individual fields
        let field_validation_results = self.validate_fields(poam_data)?;
        for result in field_validation_results {
            if !result.passed {
                match result.status {
                    crate::validation::types::ValidationStatus::Invalid => {
                        if let Some(error_msg) = &result.error_message {
                            errors.push(ValidationError {
                                code: format!("FIELD_{}", result.field_name.to_uppercase()),
                                message: error_msg.clone(),
                                field: Some(result.field_name.clone()),
                                severity: ValidationSeverity::Error,
                                suggested_fix: result.suggested_value.clone(),
                            });
                        }
                    }
                    crate::validation::types::ValidationStatus::TypeMismatch | crate::validation::types::ValidationStatus::ConditionallyMissing => {
                        if let Some(warning_msg) = &result.warning_message {
                            warnings.push(ValidationWarning {
                                code: format!("FIELD_{}_WARNING", result.field_name.to_uppercase()),
                                message: warning_msg.clone(),
                                field: Some(result.field_name.clone()),
                                recommendation: result.suggested_value.clone(),
                            });
                        }
                    }
                    _ => {}
                }
            }
            field_results.push(result);
        }

        // 2. Validate business rules
        let business_rule_results = self.business_rule_validator.validate_business_rules(poam_data)?;
        for result in &business_rule_results {
            if !result.passed {
                if let Some(message) = &result.message {
                    match result.severity {
                        ValidationSeverity::Error => {
                            errors.push(ValidationError {
                                code: format!("RULE_{}", result.rule_name.to_uppercase()),
                                message: message.clone(),
                                field: None,
                                severity: result.severity.clone(),
                                suggested_fix: None,
                            });
                        }
                        ValidationSeverity::Warning => {
                            warnings.push(ValidationWarning {
                                code: format!("RULE_{}", result.rule_name.to_uppercase()),
                                message: message.clone(),
                                field: None,
                                recommendation: None,
                            });
                        }
                        _ => {}
                    }
                }
            }
        }

        // 3. Validate cross-field relationships
        let cross_field_results = self.cross_field_validator.validate_cross_fields(poam_data)?;
        for result in cross_field_results {
            if !result.passed {
                match result.status {
                    crate::validation::types::ValidationStatus::Invalid => {
                        if let Some(error_msg) = &result.error_message {
                            errors.push(ValidationError {
                                code: format!("CROSS_FIELD_{}", result.field_name.to_uppercase()),
                                message: error_msg.clone(),
                                field: Some(result.field_name.clone()),
                                severity: ValidationSeverity::Error,
                                suggested_fix: result.suggested_value.clone(),
                            });
                        }
                    }
                    crate::validation::types::ValidationStatus::TypeMismatch | crate::validation::types::ValidationStatus::ConditionallyMissing => {
                        if let Some(warning_msg) = &result.warning_message {
                            warnings.push(ValidationWarning {
                                code: format!("CROSS_FIELD_{}_WARNING", result.field_name.to_uppercase()),
                                message: warning_msg.clone(),
                                field: Some(result.field_name.clone()),
                                recommendation: result.suggested_value.clone(),
                            });
                        }
                    }
                    _ => {}
                }
            }
            field_results.push(result);
        }

        // 4. Generate suggestions
        suggestions.extend(self.generate_suggestions(poam_data, &field_results)?);

        let elapsed_time = start_time.elapsed();
        let is_valid = errors.is_empty() && (self.validation_config.validation_mode != super::types::ValidationMode::Strict || warnings.is_empty());

        info!("POA&M validation completed in {:.2}ms: {} errors, {} warnings", 
              elapsed_time.as_millis(), errors.len(), warnings.len());

        Ok(PoamValidationResult {
            is_valid,
            errors,
            warnings,
            suggestions,
            field_results,
            business_rule_results,
            performance_metrics: ValidationPerformanceMetrics {
                total_time_ms: elapsed_time.as_millis() as u64,
                rules_evaluated: self.business_rule_validator.rules.len(),
                fields_validated: poam_data.len(),
                cache_hit_rate: self.calculate_cache_hit_rate(),
            },
        })
    }

    /// Validate individual fields
    fn validate_fields(&self, poam_data: &HashMap<String, Value>) -> Result<Vec<FieldValidationResult>> {
        let mut results = Vec::new();

        // Validate severity field
        if let Some(severity_value) = poam_data.get("severity") {
            if let Some(severity_str) = severity_value.as_str() {
                results.push(self.severity_validator.validate_severity(severity_str)?);
            }
        }

        // Validate status field
        if let Some(status_value) = poam_data.get("status") {
            if let Some(status_str) = status_value.as_str() {
                results.push(self.status_validator.validate_status(status_str)?);
            }
        }

        // Validate required fields
        let required_fields = vec![
            "poam_id", "vulnerability_id", "description", "severity", "status",
            "scheduled_completion_date", "responsible_entity"
        ];

        for field in required_fields {
            if !poam_data.contains_key(field) || poam_data[field].is_null() {
                results.push(FieldValidationResult {
                    field_name: field.to_string(),
                    passed: false,
                    status: crate::validation::types::ValidationStatus::Invalid,
                    error_message: Some(format!("Required field '{}' is missing", field)),
                    warning_message: None,
                    suggested_value: None,
                    confidence: 0.0,
                });
            }
        }

        Ok(results)
    }

    /// Generate validation suggestions
    fn generate_suggestions(
        &self,
        poam_data: &HashMap<String, Value>,
        field_results: &[FieldValidationResult],
    ) -> Result<Vec<ValidationSuggestion>> {
        let mut suggestions = Vec::new();

        // Suggest improvements based on field validation results
        for result in field_results {
            if let Some(suggested_value) = &result.suggested_value {
                suggestions.push(ValidationSuggestion {
                    suggestion_type: "field_improvement".to_string(),
                    message: format!("Consider using '{}' for field '{}'", suggested_value, result.field_name),
                    field: Some(result.field_name.clone()),
                    suggested_value: Some(suggested_value.clone()),
                });
            }
        }

        // Suggest missing optional but recommended fields
        let recommended_fields = vec![
            ("milestone_date", "Consider adding milestone dates for better tracking"),
            ("resources_required", "Consider specifying resources required for completion"),
            ("comments", "Consider adding comments for additional context"),
        ];

        for (field, message) in recommended_fields {
            if !poam_data.contains_key(field) || poam_data[field].is_null() {
                suggestions.push(ValidationSuggestion {
                    suggestion_type: "optional_field".to_string(),
                    message: message.to_string(),
                    field: Some(field.to_string()),
                    suggested_value: None,
                });
            }
        }

        Ok(suggestions)
    }

    /// Calculate cache hit rate for performance metrics
    fn calculate_cache_hit_rate(&self) -> f64 {
        let (cached_items, total_rules) = self.business_rule_validator.get_cache_stats();
        if total_rules > 0 {
            cached_items as f64 / total_rules as f64
        } else {
            0.0
        }
    }

    /// Update validation configuration
    pub fn update_config(&mut self, config: PoamValidationConfig) {
        self.validation_config = config.clone();
        self.severity_validator = SeverityValidator::new(&config.allowed_severities);
        self.status_validator = StatusValidator::new(&config.allowed_statuses);
        self.business_rule_validator = BusinessRuleValidator::new(&config.business_rules);
    }

    /// Get validation statistics
    pub fn get_validation_stats(&self) -> HashMap<String, usize> {
        let mut stats = HashMap::new();
        stats.insert("allowed_severities".to_string(), self.validation_config.allowed_severities.len());
        stats.insert("allowed_statuses".to_string(), self.validation_config.allowed_statuses.len());
        stats.insert("business_rules".to_string(), self.validation_config.business_rules.len());
        stats.insert("cross_field_rules".to_string(), self.cross_field_validator.cross_field_rules.len());
        stats
    }

    /// Clear all validation caches
    pub fn clear_caches(&mut self) {
        self.business_rule_validator.clear_cache();
    }
}

impl Default for PoamValidator {
    fn default() -> Self {
        Self::new()
    }
}
