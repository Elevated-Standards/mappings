//! Modified: 2025-09-23

//! Core validation types and structures
//!
//! This module contains the type definitions for validation components including
//! validators, rules, and configuration structures.

use fedramp_core::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::Duration;
use regex::Regex;
use crate::mapping::MappingConfiguration;

use super::super::types::*;

/// Column validator for field-level validation
#[derive(Debug)]
pub struct ColumnValidator {
    /// Mapping configuration for validation rules
    pub(crate) mapping_config: MappingConfiguration,
    /// Compiled regex patterns for format validation
    pub(crate) regex_cache: HashMap<String, Regex>,
    /// Minimum quality threshold for acceptance
    pub(crate) min_quality_threshold: f64,
    /// Performance target in milliseconds
    pub(crate) performance_target_ms: u64,
    /// Custom validation functions
    pub(crate) custom_validators: HashMap<String, fn(&[serde_json::Value]) -> Result<(ValidationResult, f64)>>,
}

/// Document validator for comprehensive document validation
#[derive(Debug, Clone)]
pub struct DocumentValidator {
    /// Validation rules loaded from configuration
    pub(crate) rules: HashMap<String, Vec<ValidationRule>>,
    /// Column validator for field-level validation
    pub(crate) column_validator: Option<ColumnValidator>,
}

/// Column validation rule for field-level validation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ColumnValidationRule {
    /// Unique identifier for the field
    pub field_id: String,
    /// Possible column names that map to this field
    pub column_names: Vec<String>,
    /// OSCAL field path this maps to
    pub oscal_field: String,
    /// Whether this field is required
    pub required: bool,
    /// Type of validation to perform
    pub validation_type: Option<String>,
    /// Allowed values for enumeration validation
    pub allowed_values: Option<Vec<String>>,
    /// Regex pattern for format validation
    pub pattern: Option<String>,
    /// Expected data type
    pub data_type: Option<String>,
    /// Conditional validation rules
    pub conditional: Option<ConditionalValidation>,
}

/// Conditional validation configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConditionalValidation {
    /// Field that this validation depends on
    pub depends_on: String,
    /// Value that the dependency field must have
    pub required_value: String,
    /// Alternative validation rule if condition is not met
    pub alternative_rule: Option<Box<ColumnValidationRule>>,
}

/// Column validation result for a single field
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ColumnValidationResult {
    /// Field identifier
    pub field_id: String,
    /// Source column name that was validated
    pub source_column: Option<String>,
    /// OSCAL field path
    pub oscal_field: String,
    /// Whether validation passed
    pub passed: bool,
    /// Validation status
    pub status: ValidationStatus,
    /// Validation message
    pub message: String,
    /// Severity level
    pub severity: ValidationSeverity,
    /// Suggestions for improvement
    pub suggestions: Vec<String>,
    /// Time taken for validation
    pub execution_time: Duration,
}

/// Column validation report containing all validation results
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ColumnValidationReport {
    /// Document type that was validated
    pub document_type: String,
    /// Overall validation status
    pub is_valid: bool,
    /// Individual field validation results
    pub field_results: Vec<ColumnValidationResult>,
    /// Missing required fields
    pub missing_required: Vec<RequiredFieldInfo>,
    /// Type mismatch information
    pub type_mismatches: Vec<TypeMismatchInfo>,
    /// Enumeration validation failures
    pub enumeration_failures: Vec<EnumerationFailureInfo>,
    /// Cross-field validation results
    pub cross_field_results: Vec<CrossFieldValidationResult>,
    /// Validation metrics
    pub metrics: ValidationMetrics,
    /// Total execution time
    pub total_execution_time: Duration,
    /// Timestamp of validation
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

/// Information about missing required fields
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RequiredFieldInfo {
    /// Field identifier
    pub field_id: String,
    /// Expected column names
    pub expected_columns: Vec<String>,
    /// OSCAL field path
    pub oscal_field: String,
    /// Description of the requirement
    pub description: String,
    /// Alternative column suggestions
    pub alternatives: Vec<String>,
}

/// Information about type mismatches
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TypeMismatchInfo {
    /// Field identifier
    pub field_id: String,
    /// Column name with type mismatch
    pub column_name: String,
    /// Expected data type
    pub expected_type: String,
    /// Actual data type found
    pub actual_type: String,
    /// Sample values that caused the mismatch
    pub sample_values: Vec<String>,
    /// Suggested conversion method
    pub suggested_conversion: Option<String>,
}

/// Information about enumeration validation failures
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnumerationFailureInfo {
    /// Field identifier
    pub field_id: String,
    /// Column name with enumeration failure
    pub column_name: String,
    /// Invalid values found
    pub invalid_values: Vec<String>,
    /// Valid values that are allowed
    pub valid_values: Vec<String>,
    /// Suggested corrections
    pub suggestions: Vec<String>,
}

/// Cross-field validation result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CrossFieldValidationResult {
    /// Rule identifier
    pub rule_id: String,
    /// Fields involved in the validation
    pub fields: Vec<String>,
    /// Whether validation passed
    pub passed: bool,
    /// Validation message
    pub message: String,
    /// Severity level
    pub severity: ValidationSeverity,
    /// Execution time
    pub execution_time: Duration,
}

/// Validation metrics for performance tracking
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationMetrics {
    /// Total number of fields validated
    pub total_fields: usize,
    /// Number of required fields
    pub required_fields: usize,
    /// Number of optional fields
    pub optional_fields: usize,
    /// Number of valid fields
    pub valid_fields: usize,
    /// Number of missing required fields
    pub missing_required_count: usize,
    /// Number of invalid fields
    pub invalid_fields: usize,
    /// Number of warnings
    pub warning_count: usize,
    /// Number of errors
    pub error_count: usize,
    /// Overall validation score (0.0 to 1.0)
    pub validation_score: f64,
}

impl ColumnValidator {
    /// Create a new column validator with mapping configuration
    pub fn new(mapping_config: MappingConfiguration) -> Self {
        Self {
            mapping_config,
            regex_cache: HashMap::new(),
            min_quality_threshold: 0.7,
            performance_target_ms: 100,
            custom_validators: HashMap::new(),
        }
    }
}

impl DocumentValidator {
    /// Create a new document validator
    #[must_use]
    pub fn new() -> Self {
        Self {
            rules: HashMap::new(),
            column_validator: None,
        }
    }

    /// Add validation rules for a document type
    pub fn add_rules(&mut self, document_type: String, rules: Vec<ValidationRule>) {
        self.rules.insert(document_type, rules);
    }

    /// Set the column validator
    pub fn set_column_validator(&mut self, validator: ColumnValidator) {
        self.column_validator = Some(validator);
    }

    /// Get validation rules for a document type
    pub fn get_rules(&self, document_type: &str) -> Option<&Vec<ValidationRule>> {
        self.rules.get(document_type)
    }

    /// Remove validation rules for a document type
    pub fn remove_rules(&mut self, document_type: &str) -> Option<Vec<ValidationRule>> {
        self.rules.remove(document_type)
    }

    /// Clear all validation rules
    pub fn clear_rules(&mut self) {
        self.rules.clear();
    }

    /// Get all document types with validation rules
    pub fn get_document_types(&self) -> Vec<&String> {
        self.rules.keys().collect()
    }
}

impl Default for DocumentValidator {
    fn default() -> Self {
        Self::new()
    }
}

impl Default for ValidationMetrics {
    fn default() -> Self {
        Self {
            total_fields: 0,
            required_fields: 0,
            optional_fields: 0,
            valid_fields: 0,
            missing_required_count: 0,
            invalid_fields: 0,
            warning_count: 0,
            error_count: 0,
            validation_score: 0.0,
        }
    }
}
