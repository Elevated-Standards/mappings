// Modified: 2025-09-22

//! Core validation types and data structures
//!
//! This module contains all the fundamental types used throughout the validation system,
//! including validation rules, results, and status enumerations.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::Duration;
use uuid::Uuid;

/// Validation rule configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationRule {
    /// Field name to validate
    pub field_name: String,
    /// Rule type (required, format, range, etc.)
    pub rule_type: String,
    /// Rule parameters
    pub parameters: serde_json::Value,
    /// Error message template
    pub error_message: String,
    /// Severity level (error, warning, info)
    pub severity: String,
}

/// Enhanced validation rule for column validation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ColumnValidationRule {
    /// Field identifier
    pub field_id: String,
    /// Column names that map to this field
    pub column_names: Vec<String>,
    /// Target OSCAL field path
    pub oscal_field: String,
    /// Whether this field is required
    pub required: bool,
    /// Validation type (enumeration, format, etc.)
    pub validation_type: Option<String>,
    /// Allowed values for enumeration validation
    pub allowed_values: Option<Vec<String>>,
    /// Regex pattern for format validation
    pub pattern: Option<String>,
    /// Data type expected
    pub data_type: Option<String>,
    /// Conditional requirements
    pub conditional: Option<ConditionalRequirement>,
}

/// Conditional requirement for field validation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConditionalRequirement {
    /// Field that determines the condition
    pub depends_on: String,
    /// Values that trigger the requirement
    pub trigger_values: Vec<String>,
    /// Whether the condition is inverted (NOT logic)
    pub inverted: bool,
}

/// Validation result for a single field
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationResult {
    /// Field name that was validated
    pub field_name: String,
    /// Rule that was applied
    pub rule_type: String,
    /// Whether validation passed
    pub passed: bool,
    /// Error message if validation failed
    pub message: Option<String>,
    /// Severity level
    pub severity: String,
}

/// Enhanced validation result for column validation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ColumnValidationResult {
    /// Field identifier
    pub field_id: String,
    /// Source column name (if found)
    pub source_column: Option<String>,
    /// Target OSCAL field path
    pub oscal_field: String,
    /// Whether validation passed
    pub passed: bool,
    /// Validation status
    pub status: ValidationStatus,
    /// Error or warning message
    pub message: String,
    /// Severity level
    pub severity: ValidationSeverity,
    /// Suggested fixes
    pub suggestions: Vec<String>,
    /// Execution time for this validation
    pub execution_time: Duration,
}

/// Validation status enumeration
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ValidationStatus {
    /// Field is present and valid
    Valid,
    /// Field is missing but required
    MissingRequired,
    /// Field is present but invalid
    Invalid,
    /// Field has warnings but is acceptable
    Warning,
    /// Field is missing but optional
    MissingOptional,
    /// Field validation was skipped
    Skipped,
}

/// Validation severity levels
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, PartialOrd)]
pub enum ValidationSeverity {
    /// Informational message
    Info,
    /// Warning that should be addressed
    Warning,
    /// Error that must be fixed
    Error,
    /// Critical error that prevents processing
    Critical,
}

/// Comprehensive column validation report
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ColumnValidationReport {
    /// Document type being validated
    pub document_type: String,
    /// Overall validation status
    pub is_valid: bool,
    /// Individual field validation results
    pub field_results: Vec<ColumnValidationResult>,
    /// Missing required fields
    pub missing_required: Vec<RequiredFieldInfo>,
    /// Data type mismatches
    pub type_mismatches: Vec<TypeMismatchInfo>,
    /// Enumeration validation failures
    pub enumeration_failures: Vec<EnumerationFailureInfo>,
    /// Cross-field validation results
    pub cross_field_results: Vec<CrossFieldValidationResult>,
    /// Overall validation metrics
    pub metrics: ValidationMetrics,
    /// Total validation time
    pub total_execution_time: Duration,
    /// Validation timestamp
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

/// Information about missing required fields
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RequiredFieldInfo {
    /// Field identifier
    pub field_id: String,
    /// Expected column names
    pub expected_columns: Vec<String>,
    /// Target OSCAL field
    pub oscal_field: String,
    /// Description of the field
    pub description: String,
    /// Suggested alternatives
    pub alternatives: Vec<String>,
}

/// Information about data type mismatches
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TypeMismatchInfo {
    /// Field identifier
    pub field_id: String,
    /// Source column name
    pub column_name: String,
    /// Expected data type
    pub expected_type: String,
    /// Actual data type detected
    pub actual_type: String,
    /// Sample values that caused the mismatch
    pub sample_values: Vec<String>,
    /// Suggested conversion
    pub suggested_conversion: Option<String>,
}

/// Information about enumeration validation failures
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnumerationFailureInfo {
    /// Field identifier
    pub field_id: String,
    /// Source column name
    pub column_name: String,
    /// Invalid values found
    pub invalid_values: Vec<String>,
    /// Allowed values
    pub allowed_values: Vec<String>,
    /// Suggested mappings for invalid values
    pub suggested_mappings: HashMap<String, String>,
}

/// Cross-field validation result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CrossFieldValidationResult {
    /// Validation rule name
    pub rule_name: String,
    /// Fields involved in the validation
    pub involved_fields: Vec<String>,
    /// Whether validation passed
    pub passed: bool,
    /// Validation message
    pub message: String,
    /// Severity level
    pub severity: ValidationSeverity,
}

/// Validation metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationMetrics {
    /// Total number of fields validated
    pub total_fields: usize,
    /// Number of required fields
    pub required_fields: usize,
    /// Number of optional fields
    pub optional_fields: usize,
    /// Number of fields that passed validation
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

/// Validation summary
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationSummary {
    /// Total validations performed
    pub total_validations: usize,
    /// Number of validations passed
    pub passed_validations: usize,
    /// Number of validations failed
    pub failed_validations: usize,
    /// Number of warnings generated
    pub warning_count: usize,
    /// Number of errors generated
    pub error_count: usize,
    /// Overall validation score (0.0-1.0)
    pub overall_score: f64,
    /// Most common validation failures
    pub common_failures: Vec<ValidationFailureInfo>,
    /// Performance metrics
    pub performance_metrics: ValidationPerformanceMetrics,
}

/// Validation failure information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationFailureInfo {
    /// Type of validation failure
    pub failure_type: String,
    /// Number of occurrences
    pub occurrence_count: usize,
    /// Percentage of total failures
    pub failure_percentage: f64,
    /// Sample error messages
    pub sample_messages: Vec<String>,
    /// Suggested remediation
    pub suggested_remediation: String,
}

/// Validation performance metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationPerformanceMetrics {
    /// Average validation time per field (microseconds)
    pub avg_validation_time_us: f64,
    /// Total validation time
    pub total_validation_time: Duration,
    /// Slowest validations
    pub slow_validations: Vec<SlowValidationInfo>,
}

/// Information about slow validations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SlowValidationInfo {
    /// Field that was slow to validate
    pub field_id: String,
    /// Validation time in microseconds
    pub validation_time_us: u64,
    /// Validation rule that was slow
    pub slow_rule: String,
}
