// Modified: 2025-01-22

//! POA&M validation type definitions and data structures
//!
//! This module contains all the core types, enums, and data structures
//! used throughout the POA&M validation system.

use crate::validation::types::{ValidationSeverity, ValidationStatus};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use chrono::NaiveDate;
use fedramp_core::Result;

/// POA&M validation configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PoamValidationConfig {
    /// Allowed severity levels
    pub allowed_severities: Vec<PoamSeverity>,
    /// Allowed status values
    pub allowed_statuses: Vec<PoamStatus>,
    /// Business rules
    pub business_rules: Vec<BusinessRule>,
    /// Validation mode
    pub validation_mode: ValidationMode,
    /// Custom validation rules
    pub custom_rules: Vec<CustomValidationRule>,
    /// Performance settings
    pub performance_settings: PerformanceSettings,
}

/// POA&M severity levels
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum PoamSeverity {
    Critical,
    High,
    Moderate,
    Low,
    Informational,
}

/// POA&M status values
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum PoamStatus {
    Open,
    InProgress,
    Completed,
    RiskAccepted,
    Deferred,
    Rejected,
    Closed,
}

/// Validation mode
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ValidationMode {
    /// Strict validation - all rules must pass
    Strict,
    /// Lenient validation - warnings allowed
    Lenient,
    /// Custom validation with specific rules
    Custom,
}

/// Business rule definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BusinessRule {
    /// Rule name
    pub name: String,
    /// Rule description
    pub description: String,
    /// Rule condition
    pub condition: RuleCondition,
    /// Rule action
    pub action: RuleAction,
    /// Validation severity
    pub severity: ValidationSeverity,
    /// Rule enabled flag
    pub enabled: bool,
}

/// Rule condition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RuleCondition {
    /// Field equals value
    FieldEquals { field: String, value: String },
    /// Field not equals value
    FieldNotEquals { field: String, value: String },
    /// Field is empty
    FieldEmpty { field: String },
    /// Field is not empty
    FieldNotEmpty { field: String },
    /// Field matches regex
    FieldMatches { field: String, pattern: String },
    /// Complex condition with multiple sub-conditions
    Complex { conditions: Vec<RuleCondition>, operator: LogicalOperator },
}

/// Logical operator for complex conditions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LogicalOperator {
    And,
    Or,
    Not,
}

/// Rule action
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RuleAction {
    /// Require field
    RequireField { field: String },
    /// Validate field value
    ValidateValue { field: String, validation: String },
    /// Set default value
    SetDefault { field: String, value: String },
    /// Generate warning
    GenerateWarning { message: String },
    /// Generate error
    GenerateError { message: String },
}

/// Custom validation rule
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CustomValidationRule {
    /// Rule name
    pub name: String,
    /// Target field
    pub field: String,
    /// Validation function name
    pub validation_function: String,
    /// Rule parameters
    pub parameters: HashMap<String, String>,
    /// Rule severity
    pub severity: ValidationSeverity,
}

/// Performance settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceSettings {
    /// Maximum validation time per item (ms)
    pub max_validation_time_ms: u64,
    /// Enable validation caching
    pub enable_caching: bool,
    /// Cache size limit
    pub cache_size_limit: usize,
    /// Parallel processing threshold
    pub parallel_threshold: usize,
}

/// Comprehensive validation result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PoamValidationResult {
    /// Overall validation status
    pub is_valid: bool,
    /// Validation errors
    pub errors: Vec<ValidationError>,
    /// Validation warnings
    pub warnings: Vec<ValidationWarning>,
    /// Validation suggestions
    pub suggestions: Vec<ValidationSuggestion>,
    /// Field validation results
    pub field_results: Vec<FieldValidationResult>,
    /// Business rule results
    pub business_rule_results: Vec<BusinessRuleResult>,
    /// Performance metrics
    pub performance_metrics: ValidationPerformanceMetrics,
}

/// Validation error
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationError {
    /// Error code
    pub code: String,
    /// Error message
    pub message: String,
    /// Field name
    pub field: Option<String>,
    /// Severity level
    pub severity: ValidationSeverity,
    /// Suggested fix
    pub suggested_fix: Option<String>,
}

/// Validation warning
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationWarning {
    /// Warning code
    pub code: String,
    /// Warning message
    pub message: String,
    /// Field name
    pub field: Option<String>,
    /// Recommendation
    pub recommendation: Option<String>,
}

/// Validation suggestion
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationSuggestion {
    /// Suggestion type
    pub suggestion_type: String,
    /// Suggestion message
    pub message: String,
    /// Field name
    pub field: Option<String>,
    /// Suggested value
    pub suggested_value: Option<String>,
}

/// Field validation result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FieldValidationResult {
    /// Field name
    pub field_name: String,
    /// Validation passed
    pub passed: bool,
    /// Validation status
    pub status: ValidationStatus,
    /// Error message if validation failed
    pub error_message: Option<String>,
    /// Warning message if applicable
    pub warning_message: Option<String>,
    /// Suggested value if applicable
    pub suggested_value: Option<String>,
    /// Validation confidence score
    pub confidence: f64,
}

/// Business rule result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BusinessRuleResult {
    /// Rule name
    pub rule_name: String,
    /// Rule passed
    pub passed: bool,
    /// Rule message
    pub message: Option<String>,
    /// Rule action taken
    pub action_taken: Option<String>,
    /// Rule severity
    pub severity: ValidationSeverity,
}

/// Validation performance metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationPerformanceMetrics {
    /// Total validation time (ms)
    pub total_time_ms: u64,
    /// Number of rules evaluated
    pub rules_evaluated: usize,
    /// Number of fields validated
    pub fields_validated: usize,
    /// Cache hit rate
    pub cache_hit_rate: f64,
}

/// Cross-field validation rule
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CrossFieldRule {
    /// Rule name
    pub name: String,
    /// Primary field
    pub primary_field: String,
    /// Related fields
    pub related_fields: Vec<String>,
    /// Validation logic description
    pub validation_logic: String,
    /// Error message template
    pub error_message: String,
    /// Rule severity
    pub severity: ValidationSeverity,
}

impl Default for PoamValidationConfig {
    fn default() -> Self {
        Self {
            allowed_severities: vec![
                PoamSeverity::Critical,
                PoamSeverity::High,
                PoamSeverity::Moderate,
                PoamSeverity::Low,
                PoamSeverity::Informational,
            ],
            allowed_statuses: vec![
                PoamStatus::Open,
                PoamStatus::InProgress,
                PoamStatus::Completed,
                PoamStatus::RiskAccepted,
                PoamStatus::Deferred,
                PoamStatus::Rejected,
                PoamStatus::Closed,
            ],
            business_rules: vec![],
            validation_mode: ValidationMode::Strict,
            custom_rules: vec![],
            performance_settings: PerformanceSettings {
                max_validation_time_ms: 5000,
                enable_caching: true,
                cache_size_limit: 1000,
                parallel_threshold: 100,
            },
        }
    }
}
