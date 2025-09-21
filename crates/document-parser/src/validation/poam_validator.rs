//! POA&M-specific validation implementation
//!
//! This module provides comprehensive validation for POA&M documents including
//! severity levels, status values, business rules, and cross-field validation.

use crate::validation::types::{ValidationStatus, ValidationSeverity};
use fedramp_core::{Result, Error};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tracing::{info, warn};
use regex::Regex;
use chrono::NaiveDate;

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
    /// Date comparison
    DateComparison { field1: String, operator: String, field2: String },
    /// Complex condition with multiple criteria
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
    pub function: String,
    /// Rule parameters
    pub parameters: HashMap<String, serde_json::Value>,
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
    /// Parallel validation threshold
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
    /// Field-specific results
    pub field_results: HashMap<String, FieldValidationResult>,
    /// Business rule results
    pub business_rule_results: Vec<BusinessRuleResult>,
    /// Validation performance metrics
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
    /// Error severity
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
    /// Validation message
    pub message: String,
    /// Original value
    pub original_value: Option<String>,
    /// Normalized value
    pub normalized_value: Option<String>,
    /// Confidence score
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
    pub message: String,
    /// Affected fields
    pub affected_fields: Vec<String>,
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

/// Severity validator
#[derive(Debug, Clone)]
pub struct SeverityValidator {
    /// Allowed severities
    pub allowed_severities: Vec<PoamSeverity>,
    /// Severity aliases
    pub severity_aliases: HashMap<String, PoamSeverity>,
}

/// Status validator
#[derive(Debug, Clone)]
pub struct StatusValidator {
    /// Allowed statuses
    pub allowed_statuses: Vec<PoamStatus>,
    /// Status aliases
    pub status_aliases: HashMap<String, PoamStatus>,
    /// Status transitions
    pub status_transitions: HashMap<PoamStatus, Vec<PoamStatus>>,
}

/// Business rule validator
#[derive(Debug, Clone)]
pub struct BusinessRuleValidator {
    /// Business rules
    pub rules: Vec<BusinessRule>,
    /// Rule cache
    pub rule_cache: HashMap<String, BusinessRuleResult>,
}

/// Cross-field validator
#[derive(Debug, Clone)]
pub struct CrossFieldValidator {
    /// Cross-field rules
    pub cross_field_rules: Vec<CrossFieldRule>,
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
    /// Validation logic
    pub validation_logic: String,
    /// Error message
    pub error_message: String,
    /// Rule severity
    pub severity: ValidationSeverity,
}

impl PoamValidator {
    /// Create a new POA&M validator with default configuration
    pub fn new() -> Self {
        let config = PoamValidationConfig::default();
        Self::with_config(config)
    }

    /// Create a new POA&M validator with custom configuration
    pub fn with_config(config: PoamValidationConfig) -> Self {
        Self {
            severity_validator: SeverityValidator::new(&config.allowed_severities),
            status_validator: StatusValidator::new(&config.allowed_statuses),
            business_rule_validator: BusinessRuleValidator::new(&config.business_rules),
            cross_field_validator: CrossFieldValidator::new(),
            validation_config: config,
        }
    }

    /// Validate a POA&M item
    pub async fn validate_poam_item(&self, poam_data: &HashMap<String, serde_json::Value>) -> Result<PoamValidationResult> {
        let start_time = std::time::Instant::now();

        info!("Starting POA&M validation for item");

        let mut errors = Vec::new();
        let mut warnings = Vec::new();
        let mut suggestions = Vec::new();
        let mut field_results = HashMap::new();
        let mut business_rule_results = Vec::new();

        // Validate severity
        if let Some(severity_result) = self.validate_severity_field(poam_data).await? {
            field_results.insert("severity".to_string(), severity_result);
        }

        // Validate status
        if let Some(status_result) = self.validate_status_field(poam_data).await? {
            field_results.insert("status".to_string(), status_result);
        }

        // Validate required fields
        self.validate_required_fields(poam_data, &mut field_results, &mut errors).await?;

        // Validate business rules
        business_rule_results = self.validate_business_rules(poam_data).await?;

        // Validate cross-field relationships
        self.validate_cross_field_relationships(poam_data, &mut field_results, &mut errors).await?;

        // Generate suggestions
        suggestions = self.generate_suggestions(&field_results, &business_rule_results);

        // Collect errors and warnings from field results
        let field_results_len = field_results.len();
        for (_, field_result) in &field_results {
            if !field_result.passed {
                match field_result.status {
                    ValidationStatus::Invalid | ValidationStatus::Missing | ValidationStatus::EnumerationFailure => {
                        errors.push(ValidationError {
                            code: format!("FIELD_{}", field_result.field_name.to_uppercase()),
                            message: field_result.message.clone(),
                            field: Some(field_result.field_name.clone()),
                            severity: ValidationSeverity::Error,
                            suggested_fix: field_result.normalized_value.clone(),
                        });
                    }
                    ValidationStatus::TypeMismatch | ValidationStatus::ConditionallyMissing => {
                        warnings.push(ValidationWarning {
                            code: format!("FIELD_{}_WARNING", field_result.field_name.to_uppercase()),
                            message: field_result.message.clone(),
                            field: Some(field_result.field_name.clone()),
                            recommendation: field_result.normalized_value.clone(),
                        });
                    }
                    _ => {}
                }
            }
        }

        // Collect errors from business rules
        for rule_result in &business_rule_results {
            if !rule_result.passed {
                match rule_result.severity {
                    ValidationSeverity::Error | ValidationSeverity::Critical => {
                        errors.push(ValidationError {
                            code: format!("RULE_{}", rule_result.rule_name.to_uppercase()),
                            message: rule_result.message.clone(),
                            field: rule_result.affected_fields.first().cloned(),
                            severity: rule_result.severity.clone(),
                            suggested_fix: None,
                        });
                    }
                    ValidationSeverity::Warning => {
                        warnings.push(ValidationWarning {
                            code: format!("RULE_{}_WARNING", rule_result.rule_name.to_uppercase()),
                            message: rule_result.message.clone(),
                            field: rule_result.affected_fields.first().cloned(),
                            recommendation: None,
                        });
                    }
                    _ => {}
                }
            }
        }

        let elapsed_time = start_time.elapsed().as_millis() as u64;
        let is_valid = errors.is_empty() || self.validation_config.validation_mode == ValidationMode::Lenient;

        Ok(PoamValidationResult {
            is_valid,
            errors,
            warnings,
            suggestions,
            field_results,
            business_rule_results,
            performance_metrics: ValidationPerformanceMetrics {
                total_time_ms: elapsed_time,
                rules_evaluated: self.validation_config.business_rules.len(),
                fields_validated: field_results_len,
                cache_hit_rate: 0.0, // TODO: Implement cache hit tracking
            },
        })
    }

    /// Validate severity field
    pub async fn validate_severity_field(&self, poam_data: &HashMap<String, serde_json::Value>) -> Result<Option<FieldValidationResult>> {
        if let Some(severity_value) = poam_data.get("severity") {
            let severity_str = severity_value.as_str().unwrap_or("");
            let result = self.severity_validator.validate(severity_str);
            Ok(Some(result))
        } else {
            Ok(Some(FieldValidationResult {
                field_name: "severity".to_string(),
                passed: false,
                status: ValidationStatus::Missing,
                message: "Severity field is required".to_string(),
                original_value: None,
                normalized_value: None,
                confidence: 0.0,
            }))
        }
    }

    /// Validate status field
    pub async fn validate_status_field(&self, poam_data: &HashMap<String, serde_json::Value>) -> Result<Option<FieldValidationResult>> {
        if let Some(status_value) = poam_data.get("status") {
            let status_str = status_value.as_str().unwrap_or("");
            let result = self.status_validator.validate(status_str);
            Ok(Some(result))
        } else {
            Ok(Some(FieldValidationResult {
                field_name: "status".to_string(),
                passed: false,
                status: ValidationStatus::Missing,
                message: "Status field is required".to_string(),
                original_value: None,
                normalized_value: None,
                confidence: 0.0,
            }))
        }
    }

    /// Validate required fields
    async fn validate_required_fields(
        &self,
        poam_data: &HashMap<String, serde_json::Value>,
        field_results: &mut HashMap<String, FieldValidationResult>,
        errors: &mut Vec<ValidationError>,
    ) -> Result<()> {
        let required_fields = vec![
            "poam_id", "vulnerability_description", "severity", "status",
            "scheduled_completion_date", "point_of_contact"
        ];

        for field in required_fields {
            if !poam_data.contains_key(field) || poam_data.get(field).unwrap().is_null() {
                field_results.insert(field.to_string(), FieldValidationResult {
                    field_name: field.to_string(),
                    passed: false,
                    status: ValidationStatus::Missing,
                    message: format!("Required field '{}' is missing", field),
                    original_value: None,
                    normalized_value: None,
                    confidence: 0.0,
                });
            }
        }

        Ok(())
    }

    /// Validate business rules
    async fn validate_business_rules(&self, poam_data: &HashMap<String, serde_json::Value>) -> Result<Vec<BusinessRuleResult>> {
        self.business_rule_validator.validate_all(poam_data).await
    }

    /// Validate cross-field relationships
    async fn validate_cross_field_relationships(
        &self,
        poam_data: &HashMap<String, serde_json::Value>,
        field_results: &mut HashMap<String, FieldValidationResult>,
        _errors: &mut Vec<ValidationError>,
    ) -> Result<()> {
        self.cross_field_validator.validate(poam_data, field_results, _errors).await
    }

    /// Generate validation suggestions
    fn generate_suggestions(
        &self,
        field_results: &HashMap<String, FieldValidationResult>,
        business_rule_results: &Vec<BusinessRuleResult>,
    ) -> Vec<ValidationSuggestion> {
        let mut suggestions = Vec::new();

        // Generate suggestions based on field validation results
        for (field_name, result) in field_results {
            if !result.passed && result.normalized_value.is_some() {
                suggestions.push(ValidationSuggestion {
                    suggestion_type: "field_correction".to_string(),
                    message: format!("Consider using '{}' for field '{}'",
                        result.normalized_value.as_ref().unwrap(), field_name),
                    field: Some(field_name.clone()),
                    suggested_value: result.normalized_value.clone(),
                });
            }
        }

        // Generate suggestions based on business rule results
        for rule_result in business_rule_results {
            if !rule_result.passed && rule_result.severity == ValidationSeverity::Warning {
                suggestions.push(ValidationSuggestion {
                    suggestion_type: "business_rule_improvement".to_string(),
                    message: format!("Consider addressing: {}", rule_result.message),
                    field: rule_result.affected_fields.first().cloned(),
                    suggested_value: None,
                });
            }
        }

        suggestions
    }
}

impl SeverityValidator {
    /// Create a new severity validator
    pub fn new(allowed_severities: &[PoamSeverity]) -> Self {
        let mut severity_aliases = HashMap::new();

        // Add standard aliases
        severity_aliases.insert("critical".to_lowercase(), PoamSeverity::Critical);
        severity_aliases.insert("high".to_lowercase(), PoamSeverity::High);
        severity_aliases.insert("moderate".to_lowercase(), PoamSeverity::Moderate);
        severity_aliases.insert("medium".to_lowercase(), PoamSeverity::Moderate);
        severity_aliases.insert("low".to_lowercase(), PoamSeverity::Low);
        severity_aliases.insert("informational".to_lowercase(), PoamSeverity::Informational);
        severity_aliases.insert("info".to_lowercase(), PoamSeverity::Informational);

        // Add numeric aliases
        severity_aliases.insert("1".to_string(), PoamSeverity::Critical);
        severity_aliases.insert("2".to_string(), PoamSeverity::High);
        severity_aliases.insert("3".to_string(), PoamSeverity::Moderate);
        severity_aliases.insert("4".to_string(), PoamSeverity::Low);
        severity_aliases.insert("5".to_string(), PoamSeverity::Informational);

        Self {
            allowed_severities: allowed_severities.to_vec(),
            severity_aliases,
        }
    }

    /// Validate severity value
    pub fn validate(&self, severity: &str) -> FieldValidationResult {
        let normalized_severity = severity.trim().to_lowercase();

        if let Some(parsed_severity) = self.severity_aliases.get(&normalized_severity) {
            if self.allowed_severities.contains(parsed_severity) {
                FieldValidationResult {
                    field_name: "severity".to_string(),
                    passed: true,
                    status: ValidationStatus::Valid,
                    message: "Severity validation passed".to_string(),
                    original_value: Some(severity.to_string()),
                    normalized_value: Some(format!("{:?}", parsed_severity)),
                    confidence: 1.0,
                }
            } else {
                FieldValidationResult {
                    field_name: "severity".to_string(),
                    passed: false,
                    status: ValidationStatus::EnumerationFailure,
                    message: format!("Severity '{}' is not allowed", severity),
                    original_value: Some(severity.to_string()),
                    normalized_value: None,
                    confidence: 0.0,
                }
            }
        } else {
            // Try fuzzy matching
            let best_match = self.find_best_severity_match(&normalized_severity);
            FieldValidationResult {
                field_name: "severity".to_string(),
                passed: false,
                status: ValidationStatus::Invalid,
                message: format!("Invalid severity '{}'. Did you mean '{:?}'?", severity, best_match),
                original_value: Some(severity.to_string()),
                normalized_value: Some(format!("{:?}", best_match)),
                confidence: 0.5,
            }
        }
    }

    /// Find best severity match using fuzzy matching
    fn find_best_severity_match(&self, input: &str) -> PoamSeverity {
        let mut best_match = &PoamSeverity::Low;
        let mut best_score = 0.0;

        for (alias, severity) in &self.severity_aliases {
            let score = self.calculate_similarity(input, alias);
            if score > best_score {
                best_score = score;
                best_match = severity;
            }
        }

        best_match.clone()
    }

    /// Calculate string similarity (simple Levenshtein-based)
    fn calculate_similarity(&self, s1: &str, s2: &str) -> f64 {
        let len1 = s1.len();
        let len2 = s2.len();

        if len1 == 0 { return len2 as f64; }
        if len2 == 0 { return len1 as f64; }

        let max_len = len1.max(len2) as f64;
        let distance = self.levenshtein_distance(s1, s2) as f64;

        1.0 - (distance / max_len)
    }

    /// Calculate Levenshtein distance
    fn levenshtein_distance(&self, s1: &str, s2: &str) -> usize {
        let chars1: Vec<char> = s1.chars().collect();
        let chars2: Vec<char> = s2.chars().collect();
        let len1 = chars1.len();
        let len2 = chars2.len();

        let mut matrix = vec![vec![0; len2 + 1]; len1 + 1];

        for i in 0..=len1 {
            matrix[i][0] = i;
        }
        for j in 0..=len2 {
            matrix[0][j] = j;
        }

        for i in 1..=len1 {
            for j in 1..=len2 {
                let cost = if chars1[i - 1] == chars2[j - 1] { 0 } else { 1 };
                matrix[i][j] = (matrix[i - 1][j] + 1)
                    .min(matrix[i][j - 1] + 1)
                    .min(matrix[i - 1][j - 1] + cost);
            }
        }

        matrix[len1][len2]
    }
}

impl StatusValidator {
    /// Create a new status validator
    pub fn new(allowed_statuses: &[PoamStatus]) -> Self {
        let mut status_aliases = HashMap::new();

        // Add standard aliases
        status_aliases.insert("open".to_lowercase(), PoamStatus::Open);
        status_aliases.insert("in-progress".to_lowercase(), PoamStatus::InProgress);
        status_aliases.insert("in progress".to_lowercase(), PoamStatus::InProgress);
        status_aliases.insert("inprogress".to_lowercase(), PoamStatus::InProgress);
        status_aliases.insert("ongoing".to_lowercase(), PoamStatus::InProgress);
        status_aliases.insert("completed".to_lowercase(), PoamStatus::Completed);
        status_aliases.insert("complete".to_lowercase(), PoamStatus::Completed);
        status_aliases.insert("done".to_lowercase(), PoamStatus::Completed);
        status_aliases.insert("risk-accepted".to_lowercase(), PoamStatus::RiskAccepted);
        status_aliases.insert("risk accepted".to_lowercase(), PoamStatus::RiskAccepted);
        status_aliases.insert("accepted".to_lowercase(), PoamStatus::RiskAccepted);
        status_aliases.insert("deferred".to_lowercase(), PoamStatus::Deferred);
        status_aliases.insert("rejected".to_lowercase(), PoamStatus::Rejected);
        status_aliases.insert("closed".to_lowercase(), PoamStatus::Closed);

        // Define allowed status transitions
        let mut status_transitions = HashMap::new();
        status_transitions.insert(PoamStatus::Open, vec![PoamStatus::InProgress, PoamStatus::Deferred, PoamStatus::Rejected]);
        status_transitions.insert(PoamStatus::InProgress, vec![PoamStatus::Completed, PoamStatus::Deferred, PoamStatus::Open]);
        status_transitions.insert(PoamStatus::Completed, vec![PoamStatus::Closed, PoamStatus::RiskAccepted]);
        status_transitions.insert(PoamStatus::RiskAccepted, vec![PoamStatus::Closed]);
        status_transitions.insert(PoamStatus::Deferred, vec![PoamStatus::Open, PoamStatus::InProgress]);
        status_transitions.insert(PoamStatus::Rejected, vec![PoamStatus::Open]);
        status_transitions.insert(PoamStatus::Closed, vec![]); // Terminal state

        Self {
            allowed_statuses: allowed_statuses.to_vec(),
            status_aliases,
            status_transitions,
        }
    }

    /// Validate status value
    pub fn validate(&self, status: &str) -> FieldValidationResult {
        let normalized_status = status.trim().to_lowercase();

        if let Some(parsed_status) = self.status_aliases.get(&normalized_status) {
            if self.allowed_statuses.contains(parsed_status) {
                FieldValidationResult {
                    field_name: "status".to_string(),
                    passed: true,
                    status: ValidationStatus::Valid,
                    message: "Status validation passed".to_string(),
                    original_value: Some(status.to_string()),
                    normalized_value: Some(format!("{:?}", parsed_status)),
                    confidence: 1.0,
                }
            } else {
                FieldValidationResult {
                    field_name: "status".to_string(),
                    passed: false,
                    status: ValidationStatus::EnumerationFailure,
                    message: format!("Status '{}' is not allowed", status),
                    original_value: Some(status.to_string()),
                    normalized_value: None,
                    confidence: 0.0,
                }
            }
        } else {
            // Try fuzzy matching
            let best_match = self.find_best_status_match(&normalized_status);
            FieldValidationResult {
                field_name: "status".to_string(),
                passed: false,
                status: ValidationStatus::Invalid,
                message: format!("Invalid status '{}'. Did you mean '{:?}'?", status, best_match),
                original_value: Some(status.to_string()),
                normalized_value: Some(format!("{:?}", best_match)),
                confidence: 0.5,
            }
        }
    }

    /// Find best status match using fuzzy matching
    fn find_best_status_match(&self, input: &str) -> PoamStatus {
        let mut best_match = &PoamStatus::Open;
        let mut best_score = 0.0;

        for (alias, status) in &self.status_aliases {
            let score = self.calculate_similarity(input, alias);
            if score > best_score {
                best_score = score;
                best_match = status;
            }
        }

        best_match.clone()
    }

    /// Calculate string similarity (reuse from SeverityValidator)
    fn calculate_similarity(&self, s1: &str, s2: &str) -> f64 {
        let len1 = s1.len();
        let len2 = s2.len();

        if len1 == 0 { return len2 as f64; }
        if len2 == 0 { return len1 as f64; }

        let max_len = len1.max(len2) as f64;
        let distance = self.levenshtein_distance(s1, s2) as f64;

        1.0 - (distance / max_len)
    }

    /// Calculate Levenshtein distance (reuse from SeverityValidator)
    fn levenshtein_distance(&self, s1: &str, s2: &str) -> usize {
        let chars1: Vec<char> = s1.chars().collect();
        let chars2: Vec<char> = s2.chars().collect();
        let len1 = chars1.len();
        let len2 = chars2.len();

        let mut matrix = vec![vec![0; len2 + 1]; len1 + 1];

        for i in 0..=len1 {
            matrix[i][0] = i;
        }
        for j in 0..=len2 {
            matrix[0][j] = j;
        }

        for i in 1..=len1 {
            for j in 1..=len2 {
                let cost = if chars1[i - 1] == chars2[j - 1] { 0 } else { 1 };
                matrix[i][j] = (matrix[i - 1][j] + 1)
                    .min(matrix[i][j - 1] + 1)
                    .min(matrix[i - 1][j - 1] + cost);
            }
        }

        matrix[len1][len2]
    }

    /// Validate status transition
    pub fn validate_transition(&self, from_status: &PoamStatus, to_status: &PoamStatus) -> bool {
        if let Some(allowed_transitions) = self.status_transitions.get(from_status) {
            allowed_transitions.contains(to_status)
        } else {
            false
        }
    }
}

impl BusinessRuleValidator {
    /// Create a new business rule validator
    pub fn new(rules: &[BusinessRule]) -> Self {
        Self {
            rules: rules.to_vec(),
            rule_cache: HashMap::new(),
        }
    }

    /// Validate all business rules
    pub async fn validate_all(&self, poam_data: &HashMap<String, serde_json::Value>) -> Result<Vec<BusinessRuleResult>> {
        let mut results = Vec::new();

        for rule in &self.rules {
            if rule.enabled {
                let result = self.validate_rule(rule, poam_data).await?;
                results.push(result);
            }
        }

        Ok(results)
    }

    /// Validate a single business rule
    async fn validate_rule(&self, rule: &BusinessRule, poam_data: &HashMap<String, serde_json::Value>) -> Result<BusinessRuleResult> {
        let condition_met = self.evaluate_condition(&rule.condition, poam_data)?;

        if condition_met {
            // Execute rule action
            match &rule.action {
                RuleAction::RequireField { field } => {
                    let field_present = poam_data.contains_key(field) && !poam_data.get(field).unwrap().is_null();
                    Ok(BusinessRuleResult {
                        rule_name: rule.name.clone(),
                        passed: field_present,
                        message: if field_present {
                            format!("Required field '{}' is present", field)
                        } else {
                            format!("Required field '{}' is missing", field)
                        },
                        affected_fields: vec![field.clone()],
                        severity: rule.severity.clone(),
                    })
                }
                RuleAction::ValidateValue { field, validation } => {
                    let validation_passed = self.validate_field_value(field, validation, poam_data)?;
                    Ok(BusinessRuleResult {
                        rule_name: rule.name.clone(),
                        passed: validation_passed,
                        message: if validation_passed {
                            format!("Field '{}' validation passed", field)
                        } else {
                            format!("Field '{}' failed validation: {}", field, validation)
                        },
                        affected_fields: vec![field.clone()],
                        severity: rule.severity.clone(),
                    })
                }
                RuleAction::GenerateWarning { message } => {
                    Ok(BusinessRuleResult {
                        rule_name: rule.name.clone(),
                        passed: false,
                        message: message.clone(),
                        affected_fields: vec![],
                        severity: ValidationSeverity::Warning,
                    })
                }
                RuleAction::GenerateError { message } => {
                    Ok(BusinessRuleResult {
                        rule_name: rule.name.clone(),
                        passed: false,
                        message: message.clone(),
                        affected_fields: vec![],
                        severity: ValidationSeverity::Error,
                    })
                }
                _ => {
                    Ok(BusinessRuleResult {
                        rule_name: rule.name.clone(),
                        passed: true,
                        message: "Rule action not implemented".to_string(),
                        affected_fields: vec![],
                        severity: ValidationSeverity::Info,
                    })
                }
            }
        } else {
            Ok(BusinessRuleResult {
                rule_name: rule.name.clone(),
                passed: true,
                message: "Rule condition not met".to_string(),
                affected_fields: vec![],
                severity: ValidationSeverity::Info,
            })
        }
    }

    /// Evaluate rule condition
    fn evaluate_condition(&self, condition: &RuleCondition, poam_data: &HashMap<String, serde_json::Value>) -> Result<bool> {
        match condition {
            RuleCondition::FieldEquals { field, value } => {
                if let Some(field_value) = poam_data.get(field) {
                    Ok(field_value.as_str().unwrap_or("") == value)
                } else {
                    Ok(false)
                }
            }
            RuleCondition::FieldNotEquals { field, value } => {
                if let Some(field_value) = poam_data.get(field) {
                    Ok(field_value.as_str().unwrap_or("") != value)
                } else {
                    Ok(true)
                }
            }
            RuleCondition::FieldEmpty { field } => {
                if let Some(field_value) = poam_data.get(field) {
                    Ok(field_value.is_null() || field_value.as_str().unwrap_or("").is_empty())
                } else {
                    Ok(true)
                }
            }
            RuleCondition::FieldNotEmpty { field } => {
                if let Some(field_value) = poam_data.get(field) {
                    Ok(!field_value.is_null() && !field_value.as_str().unwrap_or("").is_empty())
                } else {
                    Ok(false)
                }
            }
            RuleCondition::DateComparison { field1, operator, field2 } => {
                self.evaluate_date_comparison(field1, operator, field2, poam_data)
            }
            RuleCondition::Complex { conditions, operator } => {
                self.evaluate_complex_condition(conditions, operator, poam_data)
            }
        }
    }

    /// Evaluate date comparison
    fn evaluate_date_comparison(&self, field1: &str, operator: &str, field2: &str, poam_data: &HashMap<String, serde_json::Value>) -> Result<bool> {
        let date1 = self.parse_date_field(field1, poam_data)?;
        let date2 = self.parse_date_field(field2, poam_data)?;

        if let (Some(d1), Some(d2)) = (date1, date2) {
            match operator {
                ">" => Ok(d1 > d2),
                ">=" => Ok(d1 >= d2),
                "<" => Ok(d1 < d2),
                "<=" => Ok(d1 <= d2),
                "==" => Ok(d1 == d2),
                "!=" => Ok(d1 != d2),
                _ => Err(Error::validation(format!("Unknown date comparison operator: {}", operator))),
            }
        } else {
            Ok(false)
        }
    }

    /// Parse date field
    fn parse_date_field(&self, field: &str, poam_data: &HashMap<String, serde_json::Value>) -> Result<Option<NaiveDate>> {
        if let Some(date_value) = poam_data.get(field) {
            if let Some(date_str) = date_value.as_str() {
                // Try multiple date formats
                let formats = vec!["%Y-%m-%d", "%m/%d/%Y", "%d/%m/%Y", "%Y-%m-%d %H:%M:%S"];

                for format in formats {
                    if let Ok(date) = NaiveDate::parse_from_str(date_str, format) {
                        return Ok(Some(date));
                    }
                }

                warn!("Failed to parse date: {}", date_str);
                Ok(None)
            } else {
                Ok(None)
            }
        } else {
            Ok(None)
        }
    }

    /// Evaluate complex condition
    fn evaluate_complex_condition(&self, conditions: &[RuleCondition], operator: &LogicalOperator, poam_data: &HashMap<String, serde_json::Value>) -> Result<bool> {
        match operator {
            LogicalOperator::And => {
                for condition in conditions {
                    if !self.evaluate_condition(condition, poam_data)? {
                        return Ok(false);
                    }
                }
                Ok(true)
            }
            LogicalOperator::Or => {
                for condition in conditions {
                    if self.evaluate_condition(condition, poam_data)? {
                        return Ok(true);
                    }
                }
                Ok(false)
            }
            LogicalOperator::Not => {
                if conditions.len() == 1 {
                    Ok(!self.evaluate_condition(&conditions[0], poam_data)?)
                } else {
                    Err(Error::validation("NOT operator requires exactly one condition".to_string()))
                }
            }
        }
    }

    /// Validate field value
    fn validate_field_value(&self, field: &str, validation: &str, poam_data: &HashMap<String, serde_json::Value>) -> Result<bool> {
        if let Some(field_value) = poam_data.get(field) {
            match validation {
                "not_empty" => Ok(!field_value.is_null() && !field_value.as_str().unwrap_or("").is_empty()),
                "is_date" => {
                    if let Some(date_str) = field_value.as_str() {
                        Ok(self.parse_date_field(field, poam_data)?.is_some())
                    } else {
                        Ok(false)
                    }
                }
                "is_email" => {
                    if let Some(email_str) = field_value.as_str() {
                        Ok(email_str.contains('@') && email_str.contains('.'))
                    } else {
                        Ok(false)
                    }
                }
                _ => {
                    // Try as regex pattern
                    if let Ok(regex) = Regex::new(validation) {
                        if let Some(value_str) = field_value.as_str() {
                            Ok(regex.is_match(value_str))
                        } else {
                            Ok(false)
                        }
                    } else {
                        warn!("Unknown validation type: {}", validation);
                        Ok(true)
                    }
                }
            }
        } else {
            Ok(false)
        }
    }
}

impl CrossFieldValidator {
    /// Create a new cross-field validator
    pub fn new() -> Self {
        let cross_field_rules = vec![
            CrossFieldRule {
                name: "completion_date_consistency".to_string(),
                primary_field: "status".to_string(),
                related_fields: vec!["actual_completion_date".to_string()],
                validation_logic: "if status == 'Completed' then actual_completion_date must not be empty".to_string(),
                error_message: "Completed items must have an actual completion date".to_string(),
                severity: ValidationSeverity::Error,
            },
            CrossFieldRule {
                name: "scheduled_date_logic".to_string(),
                primary_field: "scheduled_completion_date".to_string(),
                related_fields: vec!["actual_completion_date".to_string()],
                validation_logic: "actual_completion_date should be <= scheduled_completion_date + grace_period".to_string(),
                error_message: "Actual completion date should not be significantly later than scheduled date".to_string(),
                severity: ValidationSeverity::Warning,
            },
            CrossFieldRule {
                name: "critical_severity_timeline".to_string(),
                primary_field: "severity".to_string(),
                related_fields: vec!["scheduled_completion_date".to_string(), "created_date".to_string()],
                validation_logic: "if severity == 'Critical' then scheduled_completion_date - created_date <= 30 days".to_string(),
                error_message: "Critical items should be scheduled for completion within 30 days".to_string(),
                severity: ValidationSeverity::Warning,
            },
        ];

        Self {
            cross_field_rules,
        }
    }

    /// Validate cross-field relationships
    pub async fn validate(
        &self,
        poam_data: &HashMap<String, serde_json::Value>,
        field_results: &mut HashMap<String, FieldValidationResult>,
        errors: &mut Vec<ValidationError>,
    ) -> Result<()> {
        for rule in &self.cross_field_rules {
            let validation_result = self.validate_cross_field_rule(rule, poam_data).await?;

            if !validation_result.passed {
                match validation_result.severity {
                    ValidationSeverity::Error | ValidationSeverity::Critical => {
                        errors.push(ValidationError {
                            code: format!("CROSS_FIELD_{}", rule.name.to_uppercase()),
                            message: validation_result.message.clone(),
                            field: Some(rule.primary_field.clone()),
                            severity: validation_result.severity.clone(),
                            suggested_fix: None,
                        });
                    }
                    _ => {
                        // Add as field result for warnings
                        field_results.insert(
                            format!("cross_field_{}", rule.name),
                            FieldValidationResult {
                                field_name: rule.primary_field.clone(),
                                passed: false,
                                status: ValidationStatus::Invalid,
                                message: validation_result.message.clone(),
                                original_value: None,
                                normalized_value: None,
                                confidence: 0.8,
                            },
                        );
                    }
                }
            }
        }

        Ok(())
    }

    /// Validate a single cross-field rule
    async fn validate_cross_field_rule(&self, rule: &CrossFieldRule, poam_data: &HashMap<String, serde_json::Value>) -> Result<BusinessRuleResult> {
        match rule.name.as_str() {
            "completion_date_consistency" => {
                let status = poam_data.get("status").and_then(|v| v.as_str()).unwrap_or("");
                let completion_date = poam_data.get("actual_completion_date").and_then(|v| v.as_str()).unwrap_or("");

                let passed = if status.to_lowercase() == "completed" {
                    !completion_date.is_empty()
                } else {
                    true
                };

                Ok(BusinessRuleResult {
                    rule_name: rule.name.clone(),
                    passed,
                    message: if passed {
                        "Completion date consistency check passed".to_string()
                    } else {
                        rule.error_message.clone()
                    },
                    affected_fields: vec![rule.primary_field.clone()],
                    severity: rule.severity.clone(),
                })
            }
            "scheduled_date_logic" => {
                let scheduled_date = self.parse_date_from_data("scheduled_completion_date", poam_data);
                let actual_date = self.parse_date_from_data("actual_completion_date", poam_data);

                let passed = match (scheduled_date, actual_date) {
                    (Some(scheduled), Some(actual)) => {
                        // Allow 7 days grace period
                        let grace_period = chrono::Duration::days(7);
                        actual <= scheduled + grace_period
                    }
                    _ => true, // Skip validation if dates are missing
                };

                Ok(BusinessRuleResult {
                    rule_name: rule.name.clone(),
                    passed,
                    message: if passed {
                        "Scheduled date logic check passed".to_string()
                    } else {
                        rule.error_message.clone()
                    },
                    affected_fields: rule.related_fields.clone(),
                    severity: rule.severity.clone(),
                })
            }
            "critical_severity_timeline" => {
                let severity = poam_data.get("severity").and_then(|v| v.as_str()).unwrap_or("");
                let scheduled_date = self.parse_date_from_data("scheduled_completion_date", poam_data);
                let created_date = self.parse_date_from_data("created_date", poam_data);

                let passed = if severity.to_lowercase() == "critical" {
                    match (scheduled_date, created_date) {
                        (Some(scheduled), Some(created)) => {
                            let duration = scheduled - created;
                            duration.num_days() <= 30
                        }
                        _ => true, // Skip validation if dates are missing
                    }
                } else {
                    true
                };

                Ok(BusinessRuleResult {
                    rule_name: rule.name.clone(),
                    passed,
                    message: if passed {
                        "Critical severity timeline check passed".to_string()
                    } else {
                        rule.error_message.clone()
                    },
                    affected_fields: rule.related_fields.clone(),
                    severity: rule.severity.clone(),
                })
            }
            _ => {
                Ok(BusinessRuleResult {
                    rule_name: rule.name.clone(),
                    passed: true,
                    message: "Unknown cross-field rule".to_string(),
                    affected_fields: vec![],
                    severity: ValidationSeverity::Info,
                })
            }
        }
    }

    /// Parse date from POA&M data
    fn parse_date_from_data(&self, field: &str, poam_data: &HashMap<String, serde_json::Value>) -> Option<NaiveDate> {
        if let Some(date_value) = poam_data.get(field) {
            if let Some(date_str) = date_value.as_str() {
                let formats = vec!["%Y-%m-%d", "%m/%d/%Y", "%d/%m/%Y"];

                for format in formats {
                    if let Ok(date) = NaiveDate::parse_from_str(date_str, format) {
                        return Some(date);
                    }
                }
            }
        }
        None
    }
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
            business_rules: vec![
                BusinessRule {
                    name: "require_completion_date_for_completed".to_string(),
                    description: "Completed items must have an actual completion date".to_string(),
                    condition: RuleCondition::FieldEquals {
                        field: "status".to_string(),
                        value: "Completed".to_string(),
                    },
                    action: RuleAction::RequireField {
                        field: "actual_completion_date".to_string(),
                    },
                    severity: ValidationSeverity::Error,
                    enabled: true,
                },
                BusinessRule {
                    name: "require_scheduled_date_for_open".to_string(),
                    description: "Open items must have a scheduled completion date".to_string(),
                    condition: RuleCondition::FieldEquals {
                        field: "status".to_string(),
                        value: "Open".to_string(),
                    },
                    action: RuleAction::RequireField {
                        field: "scheduled_completion_date".to_string(),
                    },
                    severity: ValidationSeverity::Warning,
                    enabled: true,
                },
                BusinessRule {
                    name: "require_justification_for_deferred".to_string(),
                    description: "Deferred items should have justification".to_string(),
                    condition: RuleCondition::FieldEquals {
                        field: "status".to_string(),
                        value: "Deferred".to_string(),
                    },
                    action: RuleAction::GenerateWarning {
                        message: "Deferred items should include justification in comments".to_string(),
                    },
                    severity: ValidationSeverity::Warning,
                    enabled: true,
                },
            ],
            validation_mode: ValidationMode::Strict,
            custom_rules: vec![],
            performance_settings: PerformanceSettings {
                max_validation_time_ms: 100,
                enable_caching: true,
                cache_size_limit: 1000,
                parallel_threshold: 100,
            },
        }
    }
}

impl Default for PoamValidator {
    fn default() -> Self {
        Self::new()
    }
}
