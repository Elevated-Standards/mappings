//! Basic validation types and enums

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use chrono::{DateTime, Duration};

/// Validation result for a single field
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationResult {
    /// Field name that was validated
    pub field_name: String,
    /// Whether the validation passed
    pub passed: bool,
    /// Validation status
    pub status: ValidationStatus,
    /// Severity level of any issues found
    pub severity: ValidationSeverity,
    /// Detailed message about the validation result
    pub message: String,
    /// Suggested fix if validation failed
    pub suggested_fix: Option<String>,
}

/// Column validation result with additional metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ColumnValidationResult {
    /// Field identifier
    pub field_id: String,
    /// Source column name
    pub source_column: String,
    /// Whether the validation passed
    pub passed: bool,
    /// Validation status
    pub status: ValidationStatus,
    /// Severity level
    pub severity: ValidationSeverity,
    /// Validation message
    pub message: String,
    /// Expected data type
    pub expected_type: Option<String>,
    /// Actual data type found
    pub actual_type: Option<String>,
    /// Sample invalid values
    pub sample_invalid_values: Vec<String>,
    /// Validation time in microseconds
    pub validation_time_us: u64,
}

/// Status of a validation check
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum ValidationStatus {
    /// Field is present and valid
    Valid,
    /// Field is present but invalid
    Invalid,
    /// Field is missing but required
    Missing,
    /// Field is present but has type mismatch
    TypeMismatch,
    /// Field failed enumeration validation
    EnumerationFailure,
    /// Field is conditionally required but missing
    ConditionallyMissing,
}

/// Severity level of validation issues
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub enum ValidationSeverity {
    /// Informational message
    Info,
    /// Warning that should be reviewed
    Warning,
    /// Error that affects functionality
    Error,
    /// Critical error that prevents processing
    Critical,
}

/// Comprehensive validation report for a document
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ColumnValidationReport {
    /// Document type being validated
    pub document_type: String,
    /// Overall validation success
    pub overall_success: bool,
    /// Individual field validation results
    pub field_results: Vec<ColumnValidationResult>,
    /// Summary of required fields
    pub required_fields: Vec<RequiredFieldInfo>,
    /// Summary of type mismatches
    pub type_mismatches: Vec<TypeMismatchInfo>,
    /// Summary of enumeration failures
    pub enumeration_failures: Vec<EnumerationFailureInfo>,
    /// Cross-field validation results
    pub cross_field_results: Vec<CrossFieldValidationResult>,
    /// Performance and timing metrics
    pub metrics: ValidationMetrics,
    /// Timestamp when validation was performed
    pub validated_at: DateTime<chrono::Utc>,
}

/// Information about required fields
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RequiredFieldInfo {
    /// Field identifier
    pub field_id: String,
    /// Human-readable field name
    pub field_name: String,
    /// Whether the field was found
    pub found: bool,
    /// Source column if found
    pub source_column: Option<String>,
    /// Reason why field is required
    pub requirement_reason: String,
}

/// Information about type mismatches
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TypeMismatchInfo {
    /// Field identifier
    pub field_id: String,
    /// Source column name
    pub source_column: String,
    /// Expected data type
    pub expected_type: String,
    /// Detected data type
    pub detected_type: String,
    /// Sample values that caused the mismatch
    pub sample_values: Vec<String>,
    /// Confidence in the type detection
    pub detection_confidence: f64,
}

/// Information about enumeration validation failures
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnumerationFailureInfo {
    /// Field identifier
    pub field_id: String,
    /// Source column name
    pub source_column: String,
    /// Expected enumeration values
    pub expected_values: Vec<String>,
    /// Invalid values found
    pub invalid_values: Vec<String>,
    /// Percentage of invalid values
    pub invalid_percentage: f64,
}

/// Cross-field validation result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CrossFieldValidationResult {
    /// Validation rule name
    pub rule_name: String,
    /// Fields involved in the validation
    pub involved_fields: Vec<String>,
    /// Whether the validation passed
    pub passed: bool,
    /// Validation message
    pub message: String,
    /// Severity of the issue
    pub severity: ValidationSeverity,
}

/// Performance and timing metrics for validation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationMetrics {
    /// Total number of fields validated
    pub total_fields: usize,
    /// Number of fields that passed validation
    pub passed_fields: usize,
    /// Number of fields that failed validation
    pub failed_fields: usize,
    /// Total validation time in milliseconds
    pub total_time_ms: u64,
    /// Average validation time per field in microseconds
    pub avg_time_per_field_us: f64,
    /// Memory usage during validation in bytes
    pub memory_usage_bytes: Option<u64>,
    /// Number of validation rules applied
    pub rules_applied: usize,
    /// Performance warnings
    pub performance_warnings: Vec<String>,
}

/// Issue severity levels for mapping reports
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, PartialOrd)]
pub enum IssueSeverity {
    /// Informational message
    Info,
    /// Warning that should be reviewed
    Warning,
    /// Error that affects functionality
    Error,
    /// Critical error that prevents processing
    Critical,
}

/// Category of mapping issue
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum IssueCategory {
    /// Missing required field
    MissingRequired,
    /// Low confidence mapping
    LowConfidence,
    /// Data type mismatch
    DataTypeMismatch,
    /// Validation failure
    ValidationFailure,
    /// Override conflict
    OverrideConflict,
    /// Performance issue
    Performance,
    /// Data quality issue
    DataQuality,
}

/// Risk level assessment
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, PartialOrd)]
pub enum RiskLevel {
    /// Very low risk
    VeryLow,
    /// Low risk
    Low,
    /// Medium risk
    Medium,
    /// High risk
    High,
    /// Very high risk
    VeryHigh,
}

/// Quality grade classification
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum QualityGrade {
    /// Excellent quality (90-100%)
    A,
    /// Good quality (80-89%)
    B,
    /// Fair quality (70-79%)
    C,
    /// Poor quality (60-69%)
    D,
    /// Failing quality (<60%)
    F,
}

impl PartialOrd for QualityGrade {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        use std::cmp::Ordering;
        use QualityGrade::*;
        
        let self_rank = match self {
            A => 5,
            B => 4,
            C => 3,
            D => 2,
            F => 1,
        };
        
        let other_rank = match other {
            A => 5,
            B => 4,
            C => 3,
            D => 2,
            F => 1,
        };
        
        self_rank.partial_cmp(&other_rank)
    }
}

/// Trend direction for analysis
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum TrendDirection {
    /// Improving trend
    Improving,
    /// Stable trend
    Stable,
    /// Declining trend
    Declining,
    /// Insufficient data
    InsufficientData,
}

/// Effort level required for implementation
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, PartialOrd)]
pub enum EffortLevel {
    /// Minimal effort (< 1 hour)
    Minimal,
    /// Low effort (1-4 hours)
    Low,
    /// Medium effort (1-2 days)
    Medium,
    /// High effort (3-5 days)
    High,
    /// Very high effort (> 1 week)
    VeryHigh,
}

/// Priority level for recommendations
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, PartialOrd)]
pub enum RecommendationPriority {
    /// Low priority
    Low,
    /// Medium priority
    Medium,
    /// High priority
    High,
    /// Critical priority
    Critical,
}

/// Category of recommendation
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum RecommendationCategory {
    /// Data quality improvement
    DataQuality,
    /// Mapping accuracy improvement
    MappingAccuracy,
    /// Performance optimization
    Performance,
    /// Compliance enhancement
    Compliance,
    /// Process improvement
    Process,
    /// Configuration optimization
    Configuration,
}

/// Overall quality metrics for the mapping
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QualityMetrics {
    /// Data completeness score (0.0-1.0)
    pub completeness_score: f64,
    /// Mapping accuracy score (0.0-1.0)
    pub accuracy_score: f64,
    /// Data consistency score (0.0-1.0)
    pub consistency_score: f64,
    /// Overall quality score (0.0-1.0)
    pub overall_quality_score: f64,
    /// Risk level assessment
    pub risk_level: RiskLevel,
    /// Quality grade (A-F)
    pub quality_grade: QualityGrade,
    /// Compliance percentage
    pub compliance_percentage: f64,
    /// Number of critical issues
    pub critical_issues: usize,
    /// Number of warnings
    pub warnings: usize,
}
