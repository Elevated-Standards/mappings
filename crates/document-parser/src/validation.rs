// Modified: 2025-09-20

//! Document validation and quality assurance
//!
//! This module provides comprehensive validation for parsed documents
//! including data completeness, consistency, and quality scoring.

use fedramp_core::{Result, Error};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::{Duration, Instant};
use tracing::{debug, info, warn};
use regex::Regex;
use chrono::{DateTime, NaiveDate};
use crate::mapping::{MappingConfiguration, InventoryMappings, PoamMappings};
use crate::fuzzy::FuzzyMatcher;

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

/// Comprehensive confidence scoring system for column mappings
pub struct ConfidenceScorer {
    /// Scoring configuration
    scoring_config: ScoringConfig,
    /// Factor weights for different confidence factors
    factor_weights: HashMap<ConfidenceFactor, f64>,
    /// Threshold configuration
    threshold_config: ThresholdConfig,
    /// Historical mapping data for learning
    historical_data: HistoricalMappings,
    /// Fuzzy matcher for string similarity
    fuzzy_matcher: FuzzyMatcher,
}

/// Configuration for confidence scoring
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScoringConfig {
    /// Minimum acceptable confidence score
    pub min_acceptable_score: f64,
    /// Threshold for requiring manual review
    pub review_threshold: f64,
    /// Threshold for automatic acceptance
    pub auto_accept_threshold: f64,
    /// Enable adaptive learning from user feedback
    pub adaptive_learning: bool,
    /// Performance target for batch scoring (milliseconds)
    pub performance_target_ms: u64,
}

/// Threshold configuration for confidence-based decisions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThresholdConfig {
    /// High confidence threshold (auto-accept)
    pub high_confidence: f64,
    /// Medium confidence threshold (review recommended)
    pub medium_confidence: f64,
    /// Low confidence threshold (manual verification required)
    pub low_confidence: f64,
    /// Very low confidence threshold (likely incorrect)
    pub very_low_confidence: f64,
}

/// Historical mapping data for adaptive learning
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistoricalMappings {
    /// Successful mappings with their confidence scores
    pub successful_mappings: HashMap<String, Vec<HistoricalMapping>>,
    /// Failed mappings with their confidence scores
    pub failed_mappings: HashMap<String, Vec<HistoricalMapping>>,
    /// User feedback on mapping quality
    pub user_feedback: HashMap<String, UserFeedback>,
    /// Accuracy statistics by confidence range
    pub accuracy_stats: HashMap<String, AccuracyStats>,
}

/// Historical mapping record
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistoricalMapping {
    /// Source column name
    pub source_column: String,
    /// Target field name
    pub target_field: String,
    /// Confidence score at time of mapping
    pub confidence_score: f64,
    /// Whether the mapping was successful
    pub was_successful: bool,
    /// Timestamp of the mapping
    pub timestamp: chrono::DateTime<chrono::Utc>,
    /// Document type context
    pub document_type: String,
}

/// User feedback on mapping quality
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserFeedback {
    /// User rating (1-5 scale)
    pub rating: u8,
    /// Whether the user accepted the mapping
    pub accepted: bool,
    /// User comments or corrections
    pub comments: Option<String>,
    /// Timestamp of feedback
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

/// Accuracy statistics for confidence ranges
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccuracyStats {
    /// Total number of mappings in this confidence range
    pub total_mappings: usize,
    /// Number of successful mappings
    pub successful_mappings: usize,
    /// Accuracy percentage
    pub accuracy_percentage: f64,
    /// Last updated timestamp
    pub last_updated: chrono::DateTime<chrono::Utc>,
}

/// Confidence factors used in scoring
#[derive(Debug, Clone, Hash, Eq, PartialEq, Serialize, Deserialize)]
pub enum ConfidenceFactor {
    /// String similarity between source and target
    StringSimilarity,
    /// Data type compatibility
    DataTypeMatch,
    /// Context relevance (document type, field position)
    ContextRelevance,
    /// Validation success rate
    ValidationSuccess,
    /// Historical accuracy for similar mappings
    HistoricalAccuracy,
    /// User feedback and corrections
    UserFeedback,
}

/// Comprehensive mapping confidence result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MappingConfidence {
    /// Overall confidence score (0.0 to 1.0)
    pub overall_score: f64,
    /// Individual factor scores
    pub factor_scores: HashMap<ConfidenceFactor, f64>,
    /// Threshold status for decision making
    pub threshold_status: ThresholdStatus,
    /// Confidence-based recommendations
    pub recommendations: Vec<ConfidenceRecommendation>,
    /// Risk factors that may affect confidence
    pub risk_factors: Vec<RiskFactor>,
    /// Detailed explanation of score calculation
    pub explanation: ConfidenceExplanation,
    /// Execution time for confidence calculation
    pub calculation_time: Duration,
}

/// Threshold status for confidence-based decisions
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ThresholdStatus {
    /// High confidence - auto-accept (>= 0.9)
    HighConfidence,
    /// Medium confidence - review recommended (0.7-0.9)
    MediumConfidence,
    /// Low confidence - manual verification required (0.5-0.7)
    LowConfidence,
    /// Very low confidence - likely incorrect (< 0.5)
    VeryLowConfidence,
}

/// Confidence-based recommendation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfidenceRecommendation {
    /// Type of recommendation
    pub recommendation_type: RecommendationType,
    /// Recommendation message
    pub message: String,
    /// Priority level
    pub priority: RecommendationPriority,
    /// Suggested actions
    pub suggested_actions: Vec<String>,
}

/// Type of confidence recommendation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RecommendationType {
    /// Accept the mapping automatically
    AutoAccept,
    /// Review the mapping manually
    ManualReview,
    /// Reject the mapping
    Reject,
    /// Suggest alternative mappings
    SuggestAlternatives,
    /// Request user feedback
    RequestFeedback,
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

/// Risk factor that may affect mapping confidence
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RiskFactor {
    /// Type of risk
    pub risk_type: RiskType,
    /// Risk severity
    pub severity: RiskSeverity,
    /// Description of the risk
    pub description: String,
    /// Impact on confidence score
    pub confidence_impact: f64,
}

/// Type of risk factor
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RiskType {
    /// Low string similarity
    LowStringSimilarity,
    /// Data type mismatch
    DataTypeMismatch,
    /// Missing validation data
    MissingValidationData,
    /// Historical failures
    HistoricalFailures,
    /// Ambiguous mapping
    AmbiguousMapping,
    /// Context mismatch
    ContextMismatch,
}

/// Risk severity levels
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, PartialOrd)]
pub enum RiskSeverity {
    /// Low risk
    Low,
    /// Medium risk
    Medium,
    /// High risk
    High,
    /// Critical risk
    Critical,
}

/// Detailed explanation of confidence score calculation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfidenceExplanation {
    /// Source column name
    pub source_column: String,
    /// Target field name
    pub target_field: String,
    /// Factor contributions to final score
    pub factor_contributions: HashMap<ConfidenceFactor, FactorContribution>,
    /// Weighted calculation details
    pub weighted_calculation: WeightedConfidenceCalculation,
    /// Applied adjustments
    pub adjustments: Vec<ConfidenceAdjustment>,
}

/// Contribution of a single factor to confidence score
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FactorContribution {
    /// Raw score from this factor
    pub raw_score: f64,
    /// Weight applied to this factor
    pub weight: f64,
    /// Weighted contribution
    pub weighted_contribution: f64,
    /// Explanation of how score was calculated
    pub explanation: String,
}

/// Weighted confidence calculation details
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WeightedConfidenceCalculation {
    /// Sum of all weighted contributions
    pub total_weighted_score: f64,
    /// Sum of all weights
    pub total_weight: f64,
    /// Base normalized score
    pub base_score: f64,
    /// Final score after adjustments
    pub final_score: f64,
}

/// Confidence score adjustment
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfidenceAdjustment {
    /// Type of adjustment
    pub adjustment_type: AdjustmentType,
    /// Adjustment value (positive or negative)
    pub adjustment_value: f64,
    /// Reason for adjustment
    pub reason: String,
}

/// Type of confidence adjustment
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AdjustmentType {
    /// Bonus for exact matches
    ExactMatchBonus,
    /// Penalty for ambiguous mappings
    AmbiguityPenalty,
    /// Historical accuracy adjustment
    HistoricalAdjustment,
    /// User feedback adjustment
    UserFeedbackAdjustment,
    /// Context relevance adjustment
    ContextAdjustment,
}

/// Quality metrics for a document
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QualityMetrics {
    /// Overall quality score (0.0 to 1.0)
    pub overall_score: f64,
    /// Completeness score (0.0 to 1.0)
    pub completeness_score: f64,
    /// Consistency score (0.0 to 1.0)
    pub consistency_score: f64,
    /// Accuracy score (0.0 to 1.0)
    pub accuracy_score: f64,
    /// Total number of fields
    pub total_fields: usize,
    /// Number of populated fields
    pub populated_fields: usize,
    /// Number of validation errors
    pub error_count: usize,
    /// Number of validation warnings
    pub warning_count: usize,
}

/// Comprehensive column validator for document validation
#[derive(Debug)]
pub struct ColumnValidator {
    /// Mapping configuration for validation rules
    mapping_config: MappingConfiguration,
    /// Compiled regex patterns for format validation
    regex_cache: HashMap<String, Regex>,
    /// Minimum quality threshold (0.0 to 1.0)
    min_quality_threshold: f64,
    /// Performance target for validation (milliseconds)
    performance_target_ms: u64,
}

/// Document validator for quality assurance
#[derive(Debug, Clone)]
pub struct DocumentValidator {
    /// Validation rules loaded from configuration
    rules: HashMap<String, Vec<ValidationRule>>,
    /// Minimum quality threshold
    min_quality_threshold: f64,
}

impl ColumnValidator {
    /// Create a new column validator with mapping configuration
    pub fn new(mapping_config: MappingConfiguration) -> Self {
        Self {
            mapping_config,
            regex_cache: HashMap::new(),
            min_quality_threshold: 0.8,
            performance_target_ms: 50, // Target <50ms per document
        }
    }

    /// Create a new column validator with custom thresholds
    pub fn with_thresholds(
        mapping_config: MappingConfiguration,
        quality_threshold: f64,
        performance_target_ms: u64,
    ) -> Self {
        Self {
            mapping_config,
            regex_cache: HashMap::new(),
            min_quality_threshold: quality_threshold,
            performance_target_ms,
        }
    }

    /// Validate columns against required fields for a specific document type
    pub fn validate_columns(
        &mut self,
        document_type: &str,
        detected_columns: &[String],
        sample_data: &HashMap<String, Vec<serde_json::Value>>,
    ) -> Result<ColumnValidationReport> {
        let start_time = Instant::now();
        info!("Starting column validation for document type: {}", document_type);

        let mut field_results = Vec::new();
        let mut missing_required = Vec::new();
        let mut type_mismatches = Vec::new();
        let mut enumeration_failures = Vec::new();
        let mut cross_field_results = Vec::new();

        // Get validation rules for the document type
        let validation_rules = self.get_validation_rules(document_type)?;

        // Validate each required field
        for rule in &validation_rules {
            let field_start = Instant::now();
            let result = self.validate_field(rule, detected_columns, sample_data)?;

            // Collect specific failure types
            match &result.status {
                ValidationStatus::MissingRequired => {
                    missing_required.push(RequiredFieldInfo {
                        field_id: rule.field_id.clone(),
                        expected_columns: rule.column_names.clone(),
                        oscal_field: rule.oscal_field.clone(),
                        description: format!("Required field for {}", document_type),
                        alternatives: self.suggest_alternatives(&rule.column_names, detected_columns),
                    });
                }
                ValidationStatus::Invalid => {
                    if let Some(ref validation_type) = rule.validation_type {
                        if validation_type.contains("enum") || validation_type.contains("values") {
                            if let Some(ref column_name) = result.source_column {
                                enumeration_failures.push(self.create_enumeration_failure(
                                    rule, column_name, sample_data
                                )?);
                            }
                        } else {
                            if let Some(ref column_name) = result.source_column {
                                type_mismatches.push(self.create_type_mismatch(
                                    rule, column_name, sample_data
                                )?);
                            }
                        }
                    }
                }
                _ => {}
            }

            field_results.push(result);
        }

        // Perform cross-field validation
        cross_field_results = self.validate_cross_field_rules(document_type, detected_columns, sample_data)?;

        // Calculate metrics
        let metrics = self.calculate_validation_metrics(&field_results, &cross_field_results);
        let total_execution_time = start_time.elapsed();

        // Check performance target
        if total_execution_time.as_millis() as u64 > self.performance_target_ms {
            warn!(
                "Column validation took {}ms, exceeding target of {}ms",
                total_execution_time.as_millis(),
                self.performance_target_ms
            );
        }

        let report = ColumnValidationReport {
            document_type: document_type.to_string(),
            is_valid: metrics.error_count == 0 && metrics.missing_required_count == 0,
            field_results,
            missing_required,
            type_mismatches,
            enumeration_failures,
            cross_field_results,
            metrics,
            total_execution_time,
            timestamp: chrono::Utc::now(),
        };

        info!(
            "Column validation completed in {}ms with {} errors, {} warnings",
            total_execution_time.as_millis(),
            report.metrics.error_count,
            report.metrics.warning_count
        );

        Ok(report)
    }

    /// Get validation rules for a specific document type
    fn get_validation_rules(&self, document_type: &str) -> Result<Vec<ColumnValidationRule>> {
        let mut rules = Vec::new();

        match document_type.to_lowercase().as_str() {
            "inventory" | "component-definition" => {
                if let Some(ref inventory_mappings) = self.mapping_config.inventory_mappings {
                    rules.extend(self.extract_inventory_rules(inventory_mappings)?);
                }
            }
            "poam" | "plan-of-action-and-milestones" => {
                if let Some(ref poam_mappings) = self.mapping_config.poam_mappings {
                    rules.extend(self.extract_poam_rules(poam_mappings)?);
                }
            }
            _ => {
                return Err(Error::document_parsing(format!(
                    "Unsupported document type for validation: {}",
                    document_type
                )));
            }
        }

        if rules.is_empty() {
            return Err(Error::document_parsing(format!(
                "No validation rules found for document type: {}",
                document_type
            )));
        }

        Ok(rules)
    }

    /// Extract validation rules from inventory mappings
    fn extract_inventory_rules(&self, mappings: &InventoryMappings) -> Result<Vec<ColumnValidationRule>> {
        let mut rules = Vec::new();

        // Extract from required_columns in fedramp_iiw_mappings
        for (field_id, column_mapping) in &mappings.fedramp_iiw_mappings.required_columns {
            let rule = ColumnValidationRule {
                field_id: field_id.clone(),
                column_names: column_mapping.column_names.clone(),
                oscal_field: column_mapping.field.clone(),
                required: column_mapping.required,
                validation_type: column_mapping.validation.clone(),
                allowed_values: self.extract_inventory_allowed_values(&mappings.validation_rules, &column_mapping.validation)?,
                pattern: self.extract_inventory_pattern(&mappings.validation_rules, &column_mapping.validation)?,
                data_type: None, // Not specified in current structure
                conditional: None, // TODO: Implement conditional requirements
            };
            rules.push(rule);
        }

        Ok(rules)
    }

    /// Extract validation rules from POA&M mappings
    fn extract_poam_rules(&self, mappings: &PoamMappings) -> Result<Vec<ColumnValidationRule>> {
        let mut rules = Vec::new();

        // Extract from required_columns in fedramp_v3_mappings
        for (field_id, column_mapping) in &mappings.fedramp_v3_mappings.required_columns {
            let rule = ColumnValidationRule {
                field_id: field_id.clone(),
                column_names: column_mapping.column_names.clone(),
                oscal_field: column_mapping.oscal_field.clone(),
                required: column_mapping.required,
                validation_type: column_mapping.validation.clone(),
                allowed_values: self.extract_poam_allowed_values(&mappings.fedramp_v3_mappings.validation_rules, &column_mapping.validation)?,
                pattern: self.extract_poam_pattern(&mappings.fedramp_v3_mappings.validation_rules, &column_mapping.validation)?,
                data_type: None, // Not specified in current structure
                conditional: None, // TODO: Implement conditional requirements
            };
            rules.push(rule);
        }

        Ok(rules)
    }

    /// Extract allowed values from inventory validation rules
    fn extract_inventory_allowed_values(&self, validation_rules: &crate::mapping::config::ValidationRules, validation_type: &Option<String>) -> Result<Option<Vec<String>>> {
        if let Some(ref validation_type) = validation_type {
            match validation_type.as_str() {
                "asset_types" => Ok(validation_rules.asset_types.clone()),
                "environments" => Ok(validation_rules.environments.clone()),
                "criticality_levels" => Ok(validation_rules.criticality_levels.clone()),
                "boolean_values" => Ok(validation_rules.boolean_values.clone()),
                _ => Ok(None),
            }
        } else {
            Ok(None)
        }
    }

    /// Extract pattern from inventory validation rules
    fn extract_inventory_pattern(&self, validation_rules: &crate::mapping::config::ValidationRules, validation_type: &Option<String>) -> Result<Option<String>> {
        if let Some(ref validation_type) = validation_type {
            match validation_type.as_str() {
                "ip_address" => Ok(validation_rules.ip_address_pattern.clone()),
                "mac_address" => Ok(validation_rules.mac_address_pattern.clone()),
                _ => Ok(None),
            }
        } else {
            Ok(None)
        }
    }

    /// Extract allowed values from POA&M validation rules
    fn extract_poam_allowed_values(&self, validation_rules: &crate::mapping::poam::PoamValidationRules, validation_type: &Option<String>) -> Result<Option<Vec<String>>> {
        if let Some(ref validation_type) = validation_type {
            match validation_type.as_str() {
                "severity_levels" => Ok(validation_rules.severity_levels.clone()),
                "status_values" => Ok(validation_rules.status_values.clone()),
                _ => Ok(None),
            }
        } else {
            Ok(None)
        }
    }

    /// Extract pattern from POA&M validation rules
    fn extract_poam_pattern(&self, validation_rules: &crate::mapping::poam::PoamValidationRules, validation_type: &Option<String>) -> Result<Option<String>> {
        if let Some(ref validation_type) = validation_type {
            match validation_type.as_str() {
                "control_id_list" | "control_id" => Ok(validation_rules.control_id_pattern.clone()),
                _ => Ok(None),
            }
        } else {
            Ok(None)
        }
    }

    /// Validate a single field against its rule
    fn validate_field(
        &mut self,
        rule: &ColumnValidationRule,
        detected_columns: &[String],
        sample_data: &HashMap<String, Vec<serde_json::Value>>,
    ) -> Result<ColumnValidationResult> {
        let start_time = Instant::now();

        // Find matching column
        let matched_column = self.find_matching_column(&rule.column_names, detected_columns);

        let (status, message, suggestions) = if let Some(ref column_name) = matched_column {
            // Column found, validate its data
            if let Some(column_data) = sample_data.get(column_name) {
                self.validate_column_data(rule, column_data)?
            } else {
                (
                    ValidationStatus::Warning,
                    "Column found but no sample data available for validation".to_string(),
                    vec!["Ensure the column contains data".to_string()],
                )
            }
        } else if rule.required {
            (
                ValidationStatus::MissingRequired,
                format!("Required field '{}' is missing", rule.field_id),
                self.suggest_alternatives(&rule.column_names, detected_columns),
            )
        } else {
            (
                ValidationStatus::MissingOptional,
                format!("Optional field '{}' is missing", rule.field_id),
                vec![],
            )
        };

        let severity = match status {
            ValidationStatus::Valid => ValidationSeverity::Info,
            ValidationStatus::Warning => ValidationSeverity::Warning,
            ValidationStatus::MissingRequired | ValidationStatus::Invalid => ValidationSeverity::Error,
            ValidationStatus::MissingOptional => ValidationSeverity::Info,
            ValidationStatus::Skipped => ValidationSeverity::Info,
        };

        Ok(ColumnValidationResult {
            field_id: rule.field_id.clone(),
            source_column: matched_column,
            oscal_field: rule.oscal_field.clone(),
            passed: matches!(status, ValidationStatus::Valid | ValidationStatus::Warning | ValidationStatus::MissingOptional),
            status,
            message,
            severity,
            suggestions,
            execution_time: start_time.elapsed(),
        })
    }

    /// Find matching column name using fuzzy matching
    fn find_matching_column(&self, expected_names: &[String], detected_columns: &[String]) -> Option<String> {
        // First try exact matches (case-insensitive)
        for expected in expected_names {
            for detected in detected_columns {
                if expected.to_lowercase() == detected.to_lowercase() {
                    return Some(detected.clone());
                }
            }
        }

        // Then try partial matches
        for expected in expected_names {
            for detected in detected_columns {
                if detected.to_lowercase().contains(&expected.to_lowercase()) ||
                   expected.to_lowercase().contains(&detected.to_lowercase()) {
                    return Some(detected.clone());
                }
            }
        }

        None
    }

    /// Validate column data against rule requirements
    fn validate_column_data(
        &mut self,
        rule: &ColumnValidationRule,
        column_data: &[serde_json::Value],
    ) -> Result<(ValidationStatus, String, Vec<String>)> {
        if column_data.is_empty() {
            return Ok((
                if rule.required { ValidationStatus::Invalid } else { ValidationStatus::Warning },
                "Column contains no data".to_string(),
                vec!["Ensure the column is populated with valid data".to_string()],
            ));
        }

        // Check for validation type
        if let Some(ref validation_type) = rule.validation_type {
            return self.validate_by_type(rule, column_data, validation_type);
        }

        // If no specific validation, just check for non-empty values
        let non_empty_count = column_data.iter()
            .filter(|v| !v.is_null() && !v.as_str().map_or(false, |s| s.trim().is_empty()))
            .count();

        if non_empty_count == 0 && rule.required {
            Ok((
                ValidationStatus::Invalid,
                "Required field contains only empty values".to_string(),
                vec!["Populate the field with valid data".to_string()],
            ))
        } else {
            Ok((
                ValidationStatus::Valid,
                format!("Field validation passed ({} non-empty values)", non_empty_count),
                vec![],
            ))
        }
    }

    /// Validate column data by specific validation type
    fn validate_by_type(
        &mut self,
        rule: &ColumnValidationRule,
        column_data: &[serde_json::Value],
        validation_type: &str,
    ) -> Result<(ValidationStatus, String, Vec<String>)> {
        match validation_type {
            "boolean" => self.validate_boolean_values(column_data),
            "date" => self.validate_date_values(column_data),
            "ip_address" => self.validate_ip_address_values(column_data),
            "email" => self.validate_email_values(column_data),
            "url" => self.validate_url_values(column_data),
            "unique_identifier" => self.validate_unique_identifier_values(column_data),
            "alphanumeric" => self.validate_alphanumeric_values(column_data),
            validation_type if validation_type.ends_with("_pattern") => {
                self.validate_pattern_values(rule, column_data, validation_type)
            }
            _ => {
                // Assume it's an enumeration validation
                self.validate_enumeration_values(rule, column_data, validation_type)
            }
        }
    }

    /// Validate boolean values
    fn validate_boolean_values(&self, column_data: &[serde_json::Value]) -> Result<(ValidationStatus, String, Vec<String>)> {
        let boolean_values = ["yes", "no", "y", "n", "true", "false", "1", "0", "on", "off"];
        let mut invalid_values = Vec::new();

        for value in column_data {
            if let Some(str_val) = value.as_str() {
                if !str_val.trim().is_empty() && !boolean_values.contains(&str_val.to_lowercase().as_str()) {
                    invalid_values.push(str_val.to_string());
                }
            } else if !value.is_null() && !value.is_boolean() {
                invalid_values.push(value.to_string());
            }
        }

        if invalid_values.is_empty() {
            Ok((
                ValidationStatus::Valid,
                "All boolean values are valid".to_string(),
                vec![],
            ))
        } else {
            Ok((
                ValidationStatus::Invalid,
                format!("Invalid boolean values found: {}", invalid_values.join(", ")),
                vec![
                    "Use standard boolean values: yes/no, true/false, 1/0".to_string(),
                    "Ensure consistent formatting across all rows".to_string(),
                ],
            ))
        }
    }

    /// Validate date values
    fn validate_date_values(&self, column_data: &[serde_json::Value]) -> Result<(ValidationStatus, String, Vec<String>)> {
        let mut invalid_values = Vec::new();

        for value in column_data {
            if let Some(str_val) = value.as_str() {
                if !str_val.trim().is_empty() && !self.is_valid_date(str_val) {
                    invalid_values.push(str_val.to_string());
                }
            } else if !value.is_null() {
                invalid_values.push(value.to_string());
            }
        }

        if invalid_values.is_empty() {
            Ok((
                ValidationStatus::Valid,
                "All date values are valid".to_string(),
                vec![],
            ))
        } else {
            Ok((
                ValidationStatus::Invalid,
                format!("Invalid date values found: {}", invalid_values.join(", ")),
                vec![
                    "Use ISO date format (YYYY-MM-DD) or common formats (MM/DD/YYYY, DD/MM/YYYY)".to_string(),
                    "Ensure dates are properly formatted and valid".to_string(),
                ],
            ))
        }
    }

    /// Validate IP address values
    fn validate_ip_address_values(&self, column_data: &[serde_json::Value]) -> Result<(ValidationStatus, String, Vec<String>)> {
        let mut invalid_values = Vec::new();

        for value in column_data {
            if let Some(str_val) = value.as_str() {
                if !str_val.trim().is_empty() && str_val.parse::<std::net::IpAddr>().is_err() {
                    invalid_values.push(str_val.to_string());
                }
            } else if !value.is_null() {
                invalid_values.push(value.to_string());
            }
        }

        if invalid_values.is_empty() {
            Ok((
                ValidationStatus::Valid,
                "All IP address values are valid".to_string(),
                vec![],
            ))
        } else {
            Ok((
                ValidationStatus::Invalid,
                format!("Invalid IP address values found: {}", invalid_values.join(", ")),
                vec![
                    "Use valid IPv4 (e.g., 192.168.1.1) or IPv6 format".to_string(),
                    "Remove any extra whitespace or invalid characters".to_string(),
                ],
            ))
        }
    }

    /// Validate email values
    fn validate_email_values(&self, column_data: &[serde_json::Value]) -> Result<(ValidationStatus, String, Vec<String>)> {
        let mut invalid_values = Vec::new();

        for value in column_data {
            if let Some(str_val) = value.as_str() {
                if !str_val.trim().is_empty() && !self.is_valid_email(str_val) {
                    invalid_values.push(str_val.to_string());
                }
            } else if !value.is_null() {
                invalid_values.push(value.to_string());
            }
        }

        if invalid_values.is_empty() {
            Ok((
                ValidationStatus::Valid,
                "All email values are valid".to_string(),
                vec![],
            ))
        } else {
            Ok((
                ValidationStatus::Invalid,
                format!("Invalid email values found: {}", invalid_values.join(", ")),
                vec![
                    "Use valid email format (user@domain.com)".to_string(),
                    "Ensure all email addresses contain @ and a valid domain".to_string(),
                ],
            ))
        }
    }

    /// Check if a string is a valid date
    fn is_valid_date(&self, date_str: &str) -> bool {
        // Try multiple date formats
        DateTime::parse_from_rfc3339(date_str).is_ok() ||
        NaiveDate::parse_from_str(date_str, "%Y-%m-%d").is_ok() ||
        NaiveDate::parse_from_str(date_str, "%m/%d/%Y").is_ok() ||
        NaiveDate::parse_from_str(date_str, "%d/%m/%Y").is_ok() ||
        NaiveDate::parse_from_str(date_str, "%Y/%m/%d").is_ok()
    }

    /// Check if a string is a valid email
    fn is_valid_email(&self, email: &str) -> bool {
        email.contains('@') && email.contains('.') && email.len() > 5
    }

    /// Validate URL values
    fn validate_url_values(&self, column_data: &[serde_json::Value]) -> Result<(ValidationStatus, String, Vec<String>)> {
        let mut invalid_values = Vec::new();

        for value in column_data {
            if let Some(str_val) = value.as_str() {
                if !str_val.trim().is_empty() && !self.is_valid_url(str_val) {
                    invalid_values.push(str_val.to_string());
                }
            } else if !value.is_null() {
                invalid_values.push(value.to_string());
            }
        }

        if invalid_values.is_empty() {
            Ok((ValidationStatus::Valid, "All URL values are valid".to_string(), vec![]))
        } else {
            Ok((
                ValidationStatus::Invalid,
                format!("Invalid URL values found: {}", invalid_values.join(", ")),
                vec!["Use valid URL format starting with http:// or https://".to_string()],
            ))
        }
    }

    /// Validate unique identifier values
    fn validate_unique_identifier_values(&self, column_data: &[serde_json::Value]) -> Result<(ValidationStatus, String, Vec<String>)> {
        let mut seen_values = std::collections::HashSet::new();
        let mut duplicates = Vec::new();
        let mut invalid_values = Vec::new();

        for value in column_data {
            if let Some(str_val) = value.as_str() {
                if !str_val.trim().is_empty() {
                    if !seen_values.insert(str_val.to_lowercase()) {
                        duplicates.push(str_val.to_string());
                    }
                    // Check if it's a valid identifier format
                    if !str_val.chars().all(|c| c.is_alphanumeric() || c == '-' || c == '_') {
                        invalid_values.push(str_val.to_string());
                    }
                }
            }
        }

        let mut messages = Vec::new();
        let mut suggestions = Vec::new();

        if !duplicates.is_empty() {
            messages.push(format!("Duplicate identifiers found: {}", duplicates.join(", ")));
            suggestions.push("Ensure all identifiers are unique".to_string());
        }

        if !invalid_values.is_empty() {
            messages.push(format!("Invalid identifier format: {}", invalid_values.join(", ")));
            suggestions.push("Use only alphanumeric characters, hyphens, and underscores".to_string());
        }

        if messages.is_empty() {
            Ok((ValidationStatus::Valid, "All unique identifiers are valid".to_string(), vec![]))
        } else {
            Ok((ValidationStatus::Invalid, messages.join("; "), suggestions))
        }
    }

    /// Validate alphanumeric values
    fn validate_alphanumeric_values(&self, column_data: &[serde_json::Value]) -> Result<(ValidationStatus, String, Vec<String>)> {
        let mut invalid_values = Vec::new();

        for value in column_data {
            if let Some(str_val) = value.as_str() {
                if !str_val.trim().is_empty() && !str_val.chars().all(|c| c.is_alphanumeric()) {
                    invalid_values.push(str_val.to_string());
                }
            } else if !value.is_null() {
                invalid_values.push(value.to_string());
            }
        }

        if invalid_values.is_empty() {
            Ok((ValidationStatus::Valid, "All alphanumeric values are valid".to_string(), vec![]))
        } else {
            Ok((
                ValidationStatus::Invalid,
                format!("Non-alphanumeric values found: {}", invalid_values.join(", ")),
                vec!["Use only letters and numbers".to_string()],
            ))
        }
    }

    /// Validate pattern values using regex
    fn validate_pattern_values(
        &mut self,
        rule: &ColumnValidationRule,
        column_data: &[serde_json::Value],
        validation_type: &str,
    ) -> Result<(ValidationStatus, String, Vec<String>)> {
        if let Some(ref pattern_str) = rule.pattern {
            let regex = self.get_or_compile_regex(validation_type, pattern_str)?;
            let mut invalid_values = Vec::new();

            for value in column_data {
                if let Some(str_val) = value.as_str() {
                    if !str_val.trim().is_empty() && !regex.is_match(str_val) {
                        invalid_values.push(str_val.to_string());
                    }
                }
            }

            if invalid_values.is_empty() {
                Ok((ValidationStatus::Valid, "All pattern values are valid".to_string(), vec![]))
            } else {
                Ok((
                    ValidationStatus::Invalid,
                    format!("Values not matching pattern: {}", invalid_values.join(", ")),
                    vec![format!("Ensure values match the required pattern: {}", pattern_str)],
                ))
            }
        } else {
            Ok((ValidationStatus::Skipped, "No pattern defined for validation".to_string(), vec![]))
        }
    }

    /// Validate enumeration values
    fn validate_enumeration_values(
        &self,
        rule: &ColumnValidationRule,
        column_data: &[serde_json::Value],
        _validation_type: &str,
    ) -> Result<(ValidationStatus, String, Vec<String>)> {
        if let Some(ref allowed_values) = rule.allowed_values {
            let allowed_lower: Vec<String> = allowed_values.iter().map(|v| v.to_lowercase()).collect();
            let mut invalid_values = Vec::new();

            for value in column_data {
                if let Some(str_val) = value.as_str() {
                    if !str_val.trim().is_empty() && !allowed_lower.contains(&str_val.to_lowercase()) {
                        invalid_values.push(str_val.to_string());
                    }
                }
            }

            if invalid_values.is_empty() {
                Ok((ValidationStatus::Valid, "All enumeration values are valid".to_string(), vec![]))
            } else {
                Ok((
                    ValidationStatus::Invalid,
                    format!("Invalid enumeration values: {}", invalid_values.join(", ")),
                    vec![format!("Use one of the allowed values: {}", allowed_values.join(", "))],
                ))
            }
        } else {
            Ok((ValidationStatus::Skipped, "No allowed values defined for enumeration".to_string(), vec![]))
        }
    }

    /// Get or compile regex pattern with caching
    fn get_or_compile_regex(&mut self, validation_type: &str, pattern: &str) -> Result<&Regex> {
        if !self.regex_cache.contains_key(validation_type) {
            let regex = Regex::new(pattern).map_err(|e| {
                Error::document_parsing(format!("Invalid regex pattern '{}': {}", pattern, e))
            })?;
            self.regex_cache.insert(validation_type.to_string(), regex);
        }
        Ok(self.regex_cache.get(validation_type).unwrap())
    }

    /// Check if a string is a valid URL
    fn is_valid_url(&self, url: &str) -> bool {
        url.starts_with("http://") || url.starts_with("https://")
    }

    /// Suggest alternative column names for missing fields
    fn suggest_alternatives(&self, expected_names: &[String], detected_columns: &[String]) -> Vec<String> {
        let mut suggestions = Vec::new();

        for expected in expected_names {
            for detected in detected_columns {
                // Calculate simple similarity
                let similarity = self.calculate_similarity(expected, detected);
                if similarity > 0.6 {
                    suggestions.push(format!("Consider '{}' (similarity: {:.1}%)", detected, similarity * 100.0));
                }
            }
        }

        // Limit suggestions to top 3
        suggestions.truncate(3);
        suggestions
    }

    /// Calculate simple string similarity
    fn calculate_similarity(&self, s1: &str, s2: &str) -> f64 {
        let s1_lower = s1.to_lowercase();
        let s2_lower = s2.to_lowercase();

        // Simple Jaccard similarity based on character n-grams
        let s1_chars: std::collections::HashSet<char> = s1_lower.chars().collect();
        let s2_chars: std::collections::HashSet<char> = s2_lower.chars().collect();

        let intersection = s1_chars.intersection(&s2_chars).count();
        let union = s1_chars.union(&s2_chars).count();

        if union == 0 {
            0.0
        } else {
            intersection as f64 / union as f64
        }
    }

    /// Create enumeration failure info
    fn create_enumeration_failure(
        &self,
        rule: &ColumnValidationRule,
        column_name: &str,
        sample_data: &HashMap<String, Vec<serde_json::Value>>,
    ) -> Result<EnumerationFailureInfo> {
        let mut invalid_values = Vec::new();
        let mut suggested_mappings = HashMap::new();

        if let Some(column_data) = sample_data.get(column_name) {
            if let Some(ref allowed_values) = rule.allowed_values {
                let allowed_lower: Vec<String> = allowed_values.iter().map(|v| v.to_lowercase()).collect();

                for value in column_data {
                    if let Some(str_val) = value.as_str() {
                        if !str_val.trim().is_empty() && !allowed_lower.contains(&str_val.to_lowercase()) {
                            invalid_values.push(str_val.to_string());

                            // Find best match for suggestion
                            let mut best_match = None;
                            let mut best_similarity = 0.0;

                            for allowed in allowed_values {
                                let similarity = self.calculate_similarity(str_val, allowed);
                                if similarity > best_similarity && similarity > 0.5 {
                                    best_similarity = similarity;
                                    best_match = Some(allowed.clone());
                                }
                            }

                            if let Some(suggestion) = best_match {
                                suggested_mappings.insert(str_val.to_string(), suggestion);
                            }
                        }
                    }
                }
            }
        }

        Ok(EnumerationFailureInfo {
            field_id: rule.field_id.clone(),
            column_name: column_name.to_string(),
            invalid_values,
            allowed_values: rule.allowed_values.clone().unwrap_or_default(),
            suggested_mappings,
        })
    }

    /// Create type mismatch info
    fn create_type_mismatch(
        &self,
        rule: &ColumnValidationRule,
        column_name: &str,
        sample_data: &HashMap<String, Vec<serde_json::Value>>,
    ) -> Result<TypeMismatchInfo> {
        let mut sample_values = Vec::new();
        let expected_type = rule.data_type.clone().unwrap_or_else(|| {
            rule.validation_type.clone().unwrap_or("string".to_string())
        });

        if let Some(column_data) = sample_data.get(column_name) {
            // Take up to 3 sample values that don't match the expected type
            for value in column_data.iter().take(10) {
                if let Some(str_val) = value.as_str() {
                    if !str_val.trim().is_empty() {
                        sample_values.push(str_val.to_string());
                        if sample_values.len() >= 3 {
                            break;
                        }
                    }
                }
            }
        }

        let actual_type = if sample_values.is_empty() {
            "empty".to_string()
        } else {
            "string".to_string() // Simplified type detection
        };

        let suggested_conversion = match expected_type.as_str() {
            "boolean" => Some("Convert to yes/no or true/false format".to_string()),
            "date" => Some("Use ISO date format (YYYY-MM-DD) or MM/DD/YYYY".to_string()),
            "number" => Some("Remove non-numeric characters and use decimal format".to_string()),
            _ => None,
        };

        Ok(TypeMismatchInfo {
            field_id: rule.field_id.clone(),
            column_name: column_name.to_string(),
            expected_type,
            actual_type,
            sample_values,
            suggested_conversion,
        })
    }

    /// Validate cross-field rules
    fn validate_cross_field_rules(
        &self,
        document_type: &str,
        _detected_columns: &[String],
        _sample_data: &HashMap<String, Vec<serde_json::Value>>,
    ) -> Result<Vec<CrossFieldValidationResult>> {
        let mut results = Vec::new();

        // Example cross-field validation rules
        match document_type.to_lowercase().as_str() {
            "inventory" => {
                // Add inventory-specific cross-field validations
                results.push(CrossFieldValidationResult {
                    rule_name: "asset_id_uniqueness".to_string(),
                    involved_fields: vec!["asset_id".to_string()],
                    passed: true, // Placeholder - implement actual logic
                    message: "Asset IDs should be unique across the inventory".to_string(),
                    severity: ValidationSeverity::Warning,
                });
            }
            "poam" => {
                // Add POA&M-specific cross-field validations
                results.push(CrossFieldValidationResult {
                    rule_name: "completion_date_consistency".to_string(),
                    involved_fields: vec!["scheduled_completion".to_string(), "actual_completion".to_string()],
                    passed: true, // Placeholder - implement actual logic
                    message: "Actual completion date should not be before scheduled date".to_string(),
                    severity: ValidationSeverity::Warning,
                });
            }
            _ => {}
        }

        Ok(results)
    }

    /// Calculate validation metrics
    fn calculate_validation_metrics(
        &self,
        field_results: &[ColumnValidationResult],
        cross_field_results: &[CrossFieldValidationResult],
    ) -> ValidationMetrics {
        let total_fields = field_results.len();
        let required_fields = field_results.iter().filter(|r| r.severity == ValidationSeverity::Error).count();
        let optional_fields = total_fields - required_fields;
        let valid_fields = field_results.iter().filter(|r| r.passed).count();
        let missing_required_count = field_results.iter()
            .filter(|r| r.status == ValidationStatus::MissingRequired)
            .count();
        let invalid_fields = field_results.iter()
            .filter(|r| r.status == ValidationStatus::Invalid)
            .count();
        let warning_count = field_results.iter()
            .filter(|r| r.severity == ValidationSeverity::Warning)
            .count() + cross_field_results.iter()
            .filter(|r| r.severity == ValidationSeverity::Warning)
            .count();
        let error_count = field_results.iter()
            .filter(|r| r.severity == ValidationSeverity::Error && !r.passed)
            .count() + cross_field_results.iter()
            .filter(|r| r.severity == ValidationSeverity::Error && !r.passed)
            .count();

        let validation_score = if total_fields == 0 {
            1.0
        } else {
            valid_fields as f64 / total_fields as f64
        };

        ValidationMetrics {
            total_fields,
            required_fields,
            optional_fields,
            valid_fields,
            missing_required_count,
            invalid_fields,
            warning_count,
            error_count,
            validation_score,
        }
    }
}

impl ConfidenceScorer {
    /// Create a new confidence scorer with default configuration
    pub fn new() -> Self {
        let mut factor_weights = HashMap::new();
        factor_weights.insert(ConfidenceFactor::StringSimilarity, 0.35);
        factor_weights.insert(ConfidenceFactor::DataTypeMatch, 0.25);
        factor_weights.insert(ConfidenceFactor::ContextRelevance, 0.20);
        factor_weights.insert(ConfidenceFactor::ValidationSuccess, 0.15);
        factor_weights.insert(ConfidenceFactor::HistoricalAccuracy, 0.03);
        factor_weights.insert(ConfidenceFactor::UserFeedback, 0.02);

        Self {
            scoring_config: ScoringConfig {
                min_acceptable_score: 0.5,
                review_threshold: 0.7,
                auto_accept_threshold: 0.9,
                adaptive_learning: true,
                performance_target_ms: 500,
            },
            factor_weights,
            threshold_config: ThresholdConfig {
                high_confidence: 0.9,
                medium_confidence: 0.7,
                low_confidence: 0.5,
                very_low_confidence: 0.3,
            },
            historical_data: HistoricalMappings {
                successful_mappings: HashMap::new(),
                failed_mappings: HashMap::new(),
                user_feedback: HashMap::new(),
                accuracy_stats: HashMap::new(),
            },
            fuzzy_matcher: FuzzyMatcher::for_fedramp_columns(),
        }
    }

    /// Create a confidence scorer with custom configuration
    pub fn with_config(
        scoring_config: ScoringConfig,
        factor_weights: HashMap<ConfidenceFactor, f64>,
        threshold_config: ThresholdConfig,
    ) -> Self {
        Self {
            scoring_config,
            factor_weights,
            threshold_config,
            historical_data: HistoricalMappings {
                successful_mappings: HashMap::new(),
                failed_mappings: HashMap::new(),
                user_feedback: HashMap::new(),
                accuracy_stats: HashMap::new(),
            },
            fuzzy_matcher: FuzzyMatcher::for_fedramp_columns(),
        }
    }

    /// Calculate comprehensive confidence score for a column mapping
    pub fn calculate_confidence(
        &mut self,
        source_column: &str,
        target_field: &str,
        expected_columns: &[String],
        sample_data: &[serde_json::Value],
        document_type: &str,
        validation_result: Option<&ColumnValidationResult>,
    ) -> Result<MappingConfidence> {
        let start_time = Instant::now();
        debug!("Calculating confidence for mapping '{}' -> '{}'", source_column, target_field);

        let mut factor_scores = HashMap::new();
        let mut factor_contributions = HashMap::new();
        let mut risk_factors = Vec::new();
        let mut adjustments = Vec::new();

        // Calculate individual factor scores
        let string_similarity = self.calculate_string_similarity_score(source_column, expected_columns)?;
        factor_scores.insert(ConfidenceFactor::StringSimilarity, string_similarity.raw_score);
        factor_contributions.insert(ConfidenceFactor::StringSimilarity, string_similarity);

        let data_type_score = self.calculate_data_type_score(sample_data, target_field)?;
        factor_scores.insert(ConfidenceFactor::DataTypeMatch, data_type_score.raw_score);
        factor_contributions.insert(ConfidenceFactor::DataTypeMatch, data_type_score);

        let context_score = self.calculate_context_relevance_score(source_column, target_field, document_type)?;
        factor_scores.insert(ConfidenceFactor::ContextRelevance, context_score.raw_score);
        factor_contributions.insert(ConfidenceFactor::ContextRelevance, context_score);

        let validation_score = self.calculate_validation_success_score(validation_result)?;
        factor_scores.insert(ConfidenceFactor::ValidationSuccess, validation_score.raw_score);
        factor_contributions.insert(ConfidenceFactor::ValidationSuccess, validation_score);

        let historical_score = self.calculate_historical_accuracy_score(source_column, target_field, document_type)?;
        factor_scores.insert(ConfidenceFactor::HistoricalAccuracy, historical_score.raw_score);
        factor_contributions.insert(ConfidenceFactor::HistoricalAccuracy, historical_score);

        let user_feedback_score = self.calculate_user_feedback_score(source_column, target_field)?;
        factor_scores.insert(ConfidenceFactor::UserFeedback, user_feedback_score.raw_score);
        factor_contributions.insert(ConfidenceFactor::UserFeedback, user_feedback_score);

        // Calculate weighted score
        let weighted_calculation = self.calculate_weighted_score(&factor_contributions)?;

        // Apply adjustments
        let (final_score, applied_adjustments) = self.apply_confidence_adjustments(
            weighted_calculation.base_score,
            source_column,
            expected_columns,
            &factor_scores,
        )?;

        adjustments.extend(applied_adjustments);

        // Determine threshold status
        let threshold_status = self.determine_threshold_status(final_score);

        // Generate recommendations
        let recommendations = self.generate_recommendations(
            final_score,
            &threshold_status,
            &factor_scores,
            &risk_factors,
        )?;

        // Identify risk factors
        risk_factors.extend(self.identify_risk_factors(&factor_scores, final_score)?);

        let calculation_time = start_time.elapsed();

        // Check performance target
        if calculation_time.as_millis() as u64 > self.scoring_config.performance_target_ms {
            warn!(
                "Confidence calculation took {}ms, exceeding target of {}ms",
                calculation_time.as_millis(),
                self.scoring_config.performance_target_ms
            );
        }

        let confidence = MappingConfidence {
            overall_score: final_score,
            factor_scores,
            threshold_status,
            recommendations,
            risk_factors,
            explanation: ConfidenceExplanation {
                source_column: source_column.to_string(),
                target_field: target_field.to_string(),
                factor_contributions,
                weighted_calculation: WeightedConfidenceCalculation {
                    total_weighted_score: weighted_calculation.total_weighted_score,
                    total_weight: weighted_calculation.total_weight,
                    base_score: weighted_calculation.base_score,
                    final_score,
                },
                adjustments,
            },
            calculation_time,
        };

        debug!(
            "Calculated confidence score {:.3} for '{}' -> '{}' in {}ms",
            final_score,
            source_column,
            target_field,
            calculation_time.as_millis()
        );

        Ok(confidence)
    }

    /// Calculate string similarity score between source column and expected columns
    fn calculate_string_similarity_score(&mut self, source_column: &str, expected_columns: &[String]) -> Result<FactorContribution> {
        let mut best_score = 0.0;
        let mut best_match = String::new();

        // Find the best fuzzy match
        for expected in expected_columns {
            let fuzzy_results = self.fuzzy_matcher.find_matches(source_column, &[expected.clone()]);
            if let Some(result) = fuzzy_results.first() {
                if result.confidence > best_score {
                    best_score = result.confidence;
                    best_match = expected.clone();
                }
            }
        }

        // If no fuzzy match found, calculate simple similarity
        if best_score == 0.0 {
            for expected in expected_columns {
                let similarity = self.calculate_simple_similarity(source_column, expected);
                if similarity > best_score {
                    best_score = similarity;
                    best_match = expected.clone();
                }
            }
        }

        let weight = self.factor_weights.get(&ConfidenceFactor::StringSimilarity).copied().unwrap_or(0.35);
        let weighted_contribution = best_score * weight;

        Ok(FactorContribution {
            raw_score: best_score,
            weight,
            weighted_contribution,
            explanation: if best_score > 0.8 {
                format!("High string similarity ({:.1}%) with expected column '{}'", best_score * 100.0, best_match)
            } else if best_score > 0.6 {
                format!("Moderate string similarity ({:.1}%) with expected column '{}'", best_score * 100.0, best_match)
            } else if best_score > 0.3 {
                format!("Low string similarity ({:.1}%) with expected column '{}'", best_score * 100.0, best_match)
            } else {
                "Very low string similarity with expected columns".to_string()
            },
        })
    }

    /// Calculate data type compatibility score
    fn calculate_data_type_score(&self, sample_data: &[serde_json::Value], target_field: &str) -> Result<FactorContribution> {
        if sample_data.is_empty() {
            let weight = self.factor_weights.get(&ConfidenceFactor::DataTypeMatch).copied().unwrap_or(0.25);
            return Ok(FactorContribution {
                raw_score: 0.5, // Neutral score for missing data
                weight,
                weighted_contribution: 0.5 * weight,
                explanation: "No sample data available for data type analysis".to_string(),
            });
        }

        // Analyze data types in sample
        let mut type_scores = HashMap::new();
        for value in sample_data.iter().take(10) { // Analyze up to 10 samples
            let detected_type = self.detect_data_type(value);
            *type_scores.entry(detected_type).or_insert(0) += 1;
        }

        // Determine expected type from target field
        let expected_type = self.infer_expected_type(target_field);

        // Calculate compatibility score
        let total_samples = type_scores.values().sum::<i32>() as f64;
        let compatible_samples = type_scores.get(&expected_type).copied().unwrap_or(0) as f64;
        let compatibility_score = if total_samples > 0.0 {
            compatible_samples / total_samples
        } else {
            0.5
        };

        let weight = self.factor_weights.get(&ConfidenceFactor::DataTypeMatch).copied().unwrap_or(0.25);
        let weighted_contribution = compatibility_score * weight;

        Ok(FactorContribution {
            raw_score: compatibility_score,
            weight,
            weighted_contribution,
            explanation: format!(
                "Data type compatibility: {:.1}% of samples match expected type '{}'",
                compatibility_score * 100.0,
                expected_type
            ),
        })
    }

    /// Calculate context relevance score
    fn calculate_context_relevance_score(&self, source_column: &str, target_field: &str, document_type: &str) -> Result<FactorContribution> {
        let mut relevance_score: f64 = 0.5; // Base score

        // Check document type context
        let document_bonus = match document_type.to_lowercase().as_str() {
            "inventory" | "component-definition" => {
                if target_field.contains("asset") || target_field.contains("component") || target_field.contains("inventory") {
                    0.2
                } else {
                    0.0
                }
            }
            "poam" | "plan-of-action-and-milestones" => {
                if target_field.contains("weakness") || target_field.contains("finding") || target_field.contains("remediation") {
                    0.2
                } else {
                    0.0
                }
            }
            _ => 0.0,
        };

        // Check field name context
        let field_bonus = if source_column.to_lowercase().contains(&target_field.to_lowercase()) ||
                            target_field.to_lowercase().contains(&source_column.to_lowercase()) {
            0.3
        } else {
            0.0
        };

        relevance_score += document_bonus + field_bonus;
        relevance_score = relevance_score.min(1.0); // Cap at 1.0

        let weight = self.factor_weights.get(&ConfidenceFactor::ContextRelevance).copied().unwrap_or(0.20);
        let weighted_contribution = relevance_score * weight;

        Ok(FactorContribution {
            raw_score: relevance_score,
            weight,
            weighted_contribution,
            explanation: format!(
                "Context relevance: document type '{}' and field relationship analysis",
                document_type
            ),
        })
    }

    /// Calculate validation success score
    fn calculate_validation_success_score(&self, validation_result: Option<&ColumnValidationResult>) -> Result<FactorContribution> {
        let (score, explanation) = match validation_result {
            Some(result) => {
                if result.passed {
                    match result.status {
                        ValidationStatus::Valid => (1.0, "Validation passed with valid status".to_string()),
                        ValidationStatus::Warning => (0.8, "Validation passed with warnings".to_string()),
                        ValidationStatus::MissingOptional => (0.7, "Optional field missing but acceptable".to_string()),
                        _ => (0.6, "Validation passed but with issues".to_string()),
                    }
                } else {
                    match result.status {
                        ValidationStatus::Invalid => (0.2, "Validation failed - invalid data".to_string()),
                        ValidationStatus::MissingRequired => (0.1, "Validation failed - required field missing".to_string()),
                        _ => (0.3, "Validation failed".to_string()),
                    }
                }
            }
            None => (0.5, "No validation result available".to_string()),
        };

        let weight = self.factor_weights.get(&ConfidenceFactor::ValidationSuccess).copied().unwrap_or(0.15);
        let weighted_contribution = score * weight;

        Ok(FactorContribution {
            raw_score: score,
            weight,
            weighted_contribution,
            explanation,
        })
    }

    /// Calculate historical accuracy score
    fn calculate_historical_accuracy_score(&self, source_column: &str, target_field: &str, document_type: &str) -> Result<FactorContribution> {
        let mapping_key = format!("{}->{}:{}", source_column, target_field, document_type);

        let score = if let Some(successful_mappings) = self.historical_data.successful_mappings.get(&mapping_key) {
            if let Some(failed_mappings) = self.historical_data.failed_mappings.get(&mapping_key) {
                let total = successful_mappings.len() + failed_mappings.len();
                if total > 0 {
                    successful_mappings.len() as f64 / total as f64
                } else {
                    0.5 // Neutral score
                }
            } else {
                0.8 // Only successful mappings found
            }
        } else if self.historical_data.failed_mappings.contains_key(&mapping_key) {
            0.2 // Only failed mappings found
        } else {
            0.5 // No historical data
        };

        let weight = self.factor_weights.get(&ConfidenceFactor::HistoricalAccuracy).copied().unwrap_or(0.03);
        let weighted_contribution = score * weight;

        Ok(FactorContribution {
            raw_score: score,
            weight,
            weighted_contribution,
            explanation: if score > 0.7 {
                "Strong historical accuracy for this mapping".to_string()
            } else if score > 0.5 {
                "Moderate historical accuracy for this mapping".to_string()
            } else if score > 0.3 {
                "Poor historical accuracy for this mapping".to_string()
            } else {
                "No historical data available for this mapping".to_string()
            },
        })
    }

    /// Calculate user feedback score
    fn calculate_user_feedback_score(&self, source_column: &str, target_field: &str) -> Result<FactorContribution> {
        let mapping_key = format!("{}:{}", source_column, target_field);

        let score = if let Some(feedback) = self.historical_data.user_feedback.get(&mapping_key) {
            if feedback.accepted {
                match feedback.rating {
                    5 => 1.0,
                    4 => 0.8,
                    3 => 0.6,
                    2 => 0.4,
                    1 => 0.2,
                    _ => 0.5,
                }
            } else {
                0.1 // User rejected the mapping
            }
        } else {
            0.5 // No user feedback available
        };

        let weight = self.factor_weights.get(&ConfidenceFactor::UserFeedback).copied().unwrap_or(0.02);
        let weighted_contribution = score * weight;

        Ok(FactorContribution {
            raw_score: score,
            weight,
            weighted_contribution,
            explanation: if score > 0.8 {
                "Positive user feedback for this mapping".to_string()
            } else if score > 0.5 {
                "Neutral or no user feedback available".to_string()
            } else {
                "Negative user feedback for this mapping".to_string()
            },
        })
    }

    /// Calculate weighted confidence score from factor contributions
    pub fn calculate_weighted_score(&self, factor_contributions: &HashMap<ConfidenceFactor, FactorContribution>) -> Result<WeightedConfidenceCalculation> {
        let mut total_weighted_score = 0.0;
        let mut total_weight = 0.0;

        for contribution in factor_contributions.values() {
            total_weighted_score += contribution.weighted_contribution;
            total_weight += contribution.weight;
        }

        let base_score = if total_weight > 0.0 {
            total_weighted_score / total_weight
        } else {
            0.5 // Default neutral score
        };

        Ok(WeightedConfidenceCalculation {
            total_weighted_score,
            total_weight,
            base_score,
            final_score: base_score, // Will be updated after adjustments
        })
    }

    /// Apply confidence adjustments based on various factors
    pub fn apply_confidence_adjustments(
        &self,
        base_score: f64,
        source_column: &str,
        expected_columns: &[String],
        factor_scores: &HashMap<ConfidenceFactor, f64>,
    ) -> Result<(f64, Vec<ConfidenceAdjustment>)> {
        let mut adjusted_score = base_score;
        let mut adjustments = Vec::new();

        // Exact match bonus
        if expected_columns.iter().any(|expected| expected.to_lowercase() == source_column.to_lowercase()) {
            let bonus = 0.1;
            adjusted_score += bonus;
            adjustments.push(ConfidenceAdjustment {
                adjustment_type: AdjustmentType::ExactMatchBonus,
                adjustment_value: bonus,
                reason: "Exact column name match found".to_string(),
            });
        }

        // Ambiguity penalty for very similar scores
        if let Some(&string_similarity) = factor_scores.get(&ConfidenceFactor::StringSimilarity) {
            if string_similarity > 0.4 && string_similarity < 0.7 {
                let penalty = -0.05;
                adjusted_score += penalty;
                adjustments.push(ConfidenceAdjustment {
                    adjustment_type: AdjustmentType::AmbiguityPenalty,
                    adjustment_value: penalty,
                    reason: "Moderate similarity may indicate ambiguous mapping".to_string(),
                });
            }
        }

        // Context relevance bonus
        if let Some(&context_score) = factor_scores.get(&ConfidenceFactor::ContextRelevance) {
            if context_score > 0.8 {
                let bonus = 0.05;
                adjusted_score += bonus;
                adjustments.push(ConfidenceAdjustment {
                    adjustment_type: AdjustmentType::ContextAdjustment,
                    adjustment_value: bonus,
                    reason: "High context relevance indicates strong mapping".to_string(),
                });
            }
        }

        // Ensure score stays within bounds
        adjusted_score = adjusted_score.max(0.0).min(1.0);

        Ok((adjusted_score, adjustments))
    }

    /// Determine threshold status based on confidence score
    fn determine_threshold_status(&self, confidence_score: f64) -> ThresholdStatus {
        if confidence_score >= self.threshold_config.high_confidence {
            ThresholdStatus::HighConfidence
        } else if confidence_score >= self.threshold_config.medium_confidence {
            ThresholdStatus::MediumConfidence
        } else if confidence_score >= self.threshold_config.low_confidence {
            ThresholdStatus::LowConfidence
        } else {
            ThresholdStatus::VeryLowConfidence
        }
    }

    /// Generate confidence-based recommendations
    fn generate_recommendations(
        &self,
        confidence_score: f64,
        threshold_status: &ThresholdStatus,
        factor_scores: &HashMap<ConfidenceFactor, f64>,
        risk_factors: &[RiskFactor],
    ) -> Result<Vec<ConfidenceRecommendation>> {
        let mut recommendations = Vec::new();

        match threshold_status {
            ThresholdStatus::HighConfidence => {
                recommendations.push(ConfidenceRecommendation {
                    recommendation_type: RecommendationType::AutoAccept,
                    message: format!("High confidence mapping ({:.1}%) - safe to auto-accept", confidence_score * 100.0),
                    priority: RecommendationPriority::Low,
                    suggested_actions: vec!["Accept mapping automatically".to_string()],
                });
            }
            ThresholdStatus::MediumConfidence => {
                recommendations.push(ConfidenceRecommendation {
                    recommendation_type: RecommendationType::ManualReview,
                    message: format!("Medium confidence mapping ({:.1}%) - review recommended", confidence_score * 100.0),
                    priority: RecommendationPriority::Medium,
                    suggested_actions: vec![
                        "Review mapping manually".to_string(),
                        "Verify sample data matches expected format".to_string(),
                    ],
                });
            }
            ThresholdStatus::LowConfidence => {
                recommendations.push(ConfidenceRecommendation {
                    recommendation_type: RecommendationType::ManualReview,
                    message: format!("Low confidence mapping ({:.1}%) - manual verification required", confidence_score * 100.0),
                    priority: RecommendationPriority::High,
                    suggested_actions: vec![
                        "Manually verify mapping correctness".to_string(),
                        "Consider alternative column mappings".to_string(),
                        "Check data quality and format".to_string(),
                    ],
                });
            }
            ThresholdStatus::VeryLowConfidence => {
                recommendations.push(ConfidenceRecommendation {
                    recommendation_type: RecommendationType::Reject,
                    message: format!("Very low confidence mapping ({:.1}%) - likely incorrect", confidence_score * 100.0),
                    priority: RecommendationPriority::Critical,
                    suggested_actions: vec![
                        "Reject this mapping".to_string(),
                        "Search for alternative column matches".to_string(),
                        "Verify column names and data format".to_string(),
                    ],
                });
            }
        }

        // Add specific recommendations based on factor scores
        if let Some(&string_similarity) = factor_scores.get(&ConfidenceFactor::StringSimilarity) {
            if string_similarity < 0.3 {
                recommendations.push(ConfidenceRecommendation {
                    recommendation_type: RecommendationType::SuggestAlternatives,
                    message: "Low string similarity - consider alternative column names".to_string(),
                    priority: RecommendationPriority::Medium,
                    suggested_actions: vec!["Review similar column names".to_string()],
                });
            }
        }

        // Add recommendations based on risk factors
        if risk_factors.iter().any(|r| r.severity == RiskSeverity::Critical) {
            recommendations.push(ConfidenceRecommendation {
                recommendation_type: RecommendationType::RequestFeedback,
                message: "Critical risk factors detected - user feedback recommended".to_string(),
                priority: RecommendationPriority::High,
                suggested_actions: vec!["Request user validation".to_string()],
            });
        }

        Ok(recommendations)
    }

    /// Identify risk factors that may affect mapping confidence
    pub fn identify_risk_factors(&self, factor_scores: &HashMap<ConfidenceFactor, f64>, overall_score: f64) -> Result<Vec<RiskFactor>> {
        let mut risk_factors = Vec::new();

        // Low string similarity risk
        if let Some(&string_similarity) = factor_scores.get(&ConfidenceFactor::StringSimilarity) {
            if string_similarity < 0.4 {
                risk_factors.push(RiskFactor {
                    risk_type: RiskType::LowStringSimilarity,
                    severity: if string_similarity < 0.2 { RiskSeverity::Critical } else { RiskSeverity::High },
                    description: format!("String similarity is low ({:.1}%)", string_similarity * 100.0),
                    confidence_impact: -0.2,
                });
            }
        }

        // Data type mismatch risk
        if let Some(&data_type_score) = factor_scores.get(&ConfidenceFactor::DataTypeMatch) {
            if data_type_score < 0.5 {
                risk_factors.push(RiskFactor {
                    risk_type: RiskType::DataTypeMismatch,
                    severity: if data_type_score < 0.3 { RiskSeverity::High } else { RiskSeverity::Medium },
                    description: format!("Data type compatibility is low ({:.1}%)", data_type_score * 100.0),
                    confidence_impact: -0.15,
                });
            }
        }

        // Overall low confidence risk
        if overall_score < 0.3 {
            risk_factors.push(RiskFactor {
                risk_type: RiskType::AmbiguousMapping,
                severity: RiskSeverity::Critical,
                description: "Overall confidence is very low, mapping may be incorrect".to_string(),
                confidence_impact: -0.3,
            });
        }

        Ok(risk_factors)
    }

    /// Calculate simple string similarity (fallback method)
    pub fn calculate_simple_similarity(&self, s1: &str, s2: &str) -> f64 {
        let s1_lower = s1.to_lowercase();
        let s2_lower = s2.to_lowercase();

        // Simple Jaccard similarity based on character n-grams
        let s1_chars: std::collections::HashSet<char> = s1_lower.chars().collect();
        let s2_chars: std::collections::HashSet<char> = s2_lower.chars().collect();

        let intersection = s1_chars.intersection(&s2_chars).count();
        let union = s1_chars.union(&s2_chars).count();

        if union == 0 {
            0.0
        } else {
            intersection as f64 / union as f64
        }
    }

    /// Detect data type from a JSON value
    pub fn detect_data_type(&self, value: &serde_json::Value) -> String {
        match value {
            serde_json::Value::Bool(_) => "boolean".to_string(),
            serde_json::Value::Number(_) => "number".to_string(),
            serde_json::Value::String(s) => {
                if s.trim().is_empty() {
                    "empty".to_string()
                } else if s.contains('@') && s.contains('.') {
                    "email".to_string()
                } else if s.starts_with("http://") || s.starts_with("https://") {
                    "url".to_string()
                } else if s.parse::<std::net::IpAddr>().is_ok() {
                    "ip_address".to_string()
                } else if DateTime::parse_from_rfc3339(s).is_ok() || NaiveDate::parse_from_str(s, "%Y-%m-%d").is_ok() {
                    "date".to_string()
                } else {
                    "string".to_string()
                }
            }
            serde_json::Value::Array(_) => "array".to_string(),
            serde_json::Value::Object(_) => "object".to_string(),
            serde_json::Value::Null => "null".to_string(),
        }
    }

    /// Infer expected data type from target field name
    pub fn infer_expected_type(&self, target_field: &str) -> String {
        let field_lower = target_field.to_lowercase();

        if field_lower.contains("date") || field_lower.contains("time") {
            "date".to_string()
        } else if field_lower.contains("email") {
            "email".to_string()
        } else if field_lower.contains("url") || field_lower.contains("link") {
            "url".to_string()
        } else if field_lower.contains("ip_address") || field_lower.contains("ip-address") ||
                  (field_lower.contains("ip") && field_lower.contains("address")) ||
                  field_lower == "ip" {
            "ip_address".to_string()
        } else if field_lower.contains("count") || field_lower.contains("number") || field_lower.contains("id") {
            "number".to_string()
        } else if field_lower.contains("enabled") || field_lower.contains("active") || field_lower.contains("flag") {
            "boolean".to_string()
        } else {
            "string".to_string()
        }
    }

    /// Process batch of mappings for confidence scoring (performance optimized)
    pub fn calculate_batch_confidence(
        &mut self,
        mappings: &[(String, String, Vec<String>, Vec<serde_json::Value>, String, Option<ColumnValidationResult>)],
    ) -> Result<Vec<MappingConfidence>> {
        let start_time = Instant::now();
        let mut results = Vec::new();

        info!("Processing batch of {} mappings for confidence scoring", mappings.len());

        for (source_column, target_field, expected_columns, sample_data, document_type, validation_result) in mappings {
            let confidence = self.calculate_confidence(
                source_column,
                target_field,
                expected_columns,
                sample_data,
                document_type,
                validation_result.as_ref(),
            )?;
            results.push(confidence);
        }

        let total_time = start_time.elapsed();
        let avg_time_per_mapping = total_time.as_millis() as f64 / mappings.len() as f64;

        info!(
            "Batch confidence scoring completed in {}ms (avg {:.1}ms per mapping)",
            total_time.as_millis(),
            avg_time_per_mapping
        );

        // Check performance target
        if total_time.as_millis() as u64 > self.scoring_config.performance_target_ms {
            warn!(
                "Batch confidence scoring took {}ms, exceeding target of {}ms",
                total_time.as_millis(),
                self.scoring_config.performance_target_ms
            );
        }

        Ok(results)
    }

    /// Update historical data with mapping results (for adaptive learning)
    pub fn update_historical_data(&mut self, mapping_key: String, was_successful: bool, confidence_score: f64, document_type: String) {
        let historical_mapping = HistoricalMapping {
            source_column: mapping_key.split("->").next().unwrap_or("").to_string(),
            target_field: mapping_key.split("->").nth(1).unwrap_or("").to_string(),
            confidence_score,
            was_successful,
            timestamp: chrono::Utc::now(),
            document_type: document_type.clone(),
        };

        if was_successful {
            self.historical_data.successful_mappings
                .entry(mapping_key.clone())
                .or_insert_with(Vec::new)
                .push(historical_mapping);
        } else {
            self.historical_data.failed_mappings
                .entry(mapping_key.clone())
                .or_insert_with(Vec::new)
                .push(historical_mapping);
        }

        // Update accuracy statistics
        self.update_accuracy_stats(confidence_score, was_successful);
    }

    /// Update accuracy statistics for confidence ranges
    fn update_accuracy_stats(&mut self, confidence_score: f64, was_successful: bool) {
        let range_key = if confidence_score >= 0.9 {
            "high".to_string()
        } else if confidence_score >= 0.7 {
            "medium".to_string()
        } else if confidence_score >= 0.5 {
            "low".to_string()
        } else {
            "very_low".to_string()
        };

        let stats = self.historical_data.accuracy_stats
            .entry(range_key)
            .or_insert_with(|| AccuracyStats {
                total_mappings: 0,
                successful_mappings: 0,
                accuracy_percentage: 0.0,
                last_updated: chrono::Utc::now(),
            });

        stats.total_mappings += 1;
        if was_successful {
            stats.successful_mappings += 1;
        }
        stats.accuracy_percentage = (stats.successful_mappings as f64 / stats.total_mappings as f64) * 100.0;
        stats.last_updated = chrono::Utc::now();
    }

    /// Add user feedback for adaptive learning
    pub fn add_user_feedback(&mut self, source_column: &str, target_field: &str, feedback: UserFeedback) {
        let mapping_key = format!("{}:{}", source_column, target_field);
        self.historical_data.user_feedback.insert(mapping_key, feedback);
    }

    /// Get confidence statistics for reporting
    pub fn get_confidence_statistics(&self) -> HashMap<String, f64> {
        let mut stats = HashMap::new();

        for (range, accuracy_stats) in &self.historical_data.accuracy_stats {
            stats.insert(format!("{}_accuracy", range), accuracy_stats.accuracy_percentage);
            stats.insert(format!("{}_total", range), accuracy_stats.total_mappings as f64);
        }

        stats
    }
}

impl DocumentValidator {
    /// Create a new document validator
    #[must_use]
    pub fn new() -> Self {
        Self {
            rules: HashMap::new(),
            min_quality_threshold: 0.8,
        }
    }

    /// Create a new document validator with custom quality threshold
    #[must_use]
    pub fn with_quality_threshold(min_quality_threshold: f64) -> Self {
        Self {
            rules: HashMap::new(),
            min_quality_threshold,
        }
    }

    /// Load validation rules from configuration
    pub fn load_rules(&mut self, rules: HashMap<String, Vec<ValidationRule>>) -> Result<()> {
        info!("Loading validation rules for {} document types", rules.len());
        self.rules = rules;
        Ok(())
    }

    /// Validate document content
    pub fn validate_document(
        &self,
        document_type: &str,
        content: &serde_json::Value,
    ) -> Result<Vec<ValidationResult>> {
        debug!("Validating document of type: {}", document_type);
        
        let mut results = Vec::new();
        
        if let Some(document_rules) = self.rules.get(document_type) {
            for rule in document_rules {
                let validation_result = self.apply_rule(rule, content)?;
                results.push(validation_result);
            }
        } else {
            warn!("No validation rules found for document type: {}", document_type);
        }
        
        info!("Completed validation with {} results", results.len());
        Ok(results)
    }

    /// Apply a single validation rule
    fn apply_rule(
        &self,
        rule: &ValidationRule,
        content: &serde_json::Value,
    ) -> Result<ValidationResult> {
        let field_value = self.extract_field_value(content, &rule.field_name);
        
        let passed = match rule.rule_type.as_str() {
            "required" => self.validate_required(&field_value),
            "format" => self.validate_format(&field_value, &rule.parameters)?,
            "range" => self.validate_range(&field_value, &rule.parameters)?,
            "length" => self.validate_length(&field_value, &rule.parameters)?,
            "pattern" => self.validate_pattern(&field_value, &rule.parameters)?,
            _ => {
                warn!("Unknown validation rule type: {}", rule.rule_type);
                true // Skip unknown rules
            }
        };

        Ok(ValidationResult {
            field_name: rule.field_name.clone(),
            rule_type: rule.rule_type.clone(),
            passed,
            message: if passed { None } else { Some(rule.error_message.clone()) },
            severity: rule.severity.clone(),
        })
    }

    /// Extract field value from content using dot notation
    fn extract_field_value<'a>(&self, content: &'a serde_json::Value, field_path: &str) -> Option<&'a serde_json::Value> {
        let parts: Vec<&str> = field_path.split('.').collect();
        let mut current = content;
        
        for part in parts {
            current = current.get(part)?;
        }
        
        Some(current)
    }

    /// Validate that a field is present and not null/empty
    fn validate_required(&self, value: &Option<&serde_json::Value>) -> bool {
        match value {
            Some(serde_json::Value::Null) => false,
            Some(serde_json::Value::String(s)) => !s.trim().is_empty(),
            Some(serde_json::Value::Array(arr)) => !arr.is_empty(),
            Some(serde_json::Value::Object(obj)) => !obj.is_empty(),
            Some(_) => true,
            None => false,
        }
    }

    /// Validate field format (email, date, etc.)
    fn validate_format(
        &self,
        value: &Option<&serde_json::Value>,
        parameters: &serde_json::Value,
    ) -> Result<bool> {
        let Some(value) = value else { return Ok(true) }; // Skip validation if field is missing
        let Some(format_type) = parameters.get("type").and_then(|t| t.as_str()) else {
            return Ok(true);
        };

        match format_type {
            "email" => Ok(self.validate_email_format(value)),
            "date" => Ok(self.validate_date_format(value)),
            "url" => Ok(self.validate_url_format(value)),
            "ip" => Ok(self.validate_ip_format(value)),
            _ => {
                warn!("Unknown format type: {}", format_type);
                Ok(true)
            }
        }
    }

    /// Validate numeric range
    fn validate_range(
        &self,
        value: &Option<&serde_json::Value>,
        parameters: &serde_json::Value,
    ) -> Result<bool> {
        let Some(value) = value else { return Ok(true) };
        let Some(num) = value.as_f64() else { return Ok(false) };

        let min = parameters.get("min").and_then(|v| v.as_f64());
        let max = parameters.get("max").and_then(|v| v.as_f64());

        let within_min = min.map_or(true, |m| num >= m);
        let within_max = max.map_or(true, |m| num <= m);

        Ok(within_min && within_max)
    }

    /// Validate string length
    fn validate_length(
        &self,
        value: &Option<&serde_json::Value>,
        parameters: &serde_json::Value,
    ) -> Result<bool> {
        let Some(value) = value else { return Ok(true) };
        let Some(text) = value.as_str() else { return Ok(false) };

        let min_len = parameters.get("min").and_then(|v| v.as_u64()).unwrap_or(0) as usize;
        let max_len = parameters.get("max").and_then(|v| v.as_u64()).map(|v| v as usize);

        let len = text.len();
        let within_min = len >= min_len;
        let within_max = max_len.map_or(true, |m| len <= m);

        Ok(within_min && within_max)
    }

    /// Validate against regex pattern
    fn validate_pattern(
        &self,
        value: &Option<&serde_json::Value>,
        parameters: &serde_json::Value,
    ) -> Result<bool> {
        let Some(value) = value else { return Ok(true) };
        let Some(text) = value.as_str() else { return Ok(false) };
        let Some(pattern) = parameters.get("pattern").and_then(|p| p.as_str()) else {
            return Ok(true);
        };

        // TODO: Implement regex validation
        // For now, just check if pattern is contained in text
        Ok(text.contains(pattern))
    }

    /// Validate email format
    fn validate_email_format(&self, value: &serde_json::Value) -> bool {
        if let Some(email) = value.as_str() {
            email.contains('@') && email.contains('.')
        } else {
            false
        }
    }

    /// Validate date format
    fn validate_date_format(&self, value: &serde_json::Value) -> bool {
        if let Some(date_str) = value.as_str() {
            // Basic date format validation
            chrono::DateTime::parse_from_rfc3339(date_str).is_ok()
                || chrono::NaiveDate::parse_from_str(date_str, "%Y-%m-%d").is_ok()
        } else {
            false
        }
    }

    /// Validate URL format
    fn validate_url_format(&self, value: &serde_json::Value) -> bool {
        if let Some(url) = value.as_str() {
            url.starts_with("http://") || url.starts_with("https://")
        } else {
            false
        }
    }

    /// Validate IP address format
    fn validate_ip_format(&self, value: &serde_json::Value) -> bool {
        if let Some(ip) = value.as_str() {
            ip.parse::<std::net::IpAddr>().is_ok()
        } else {
            false
        }
    }

    /// Calculate quality metrics for validation results
    pub fn calculate_quality_metrics(
        &self,
        validation_results: &[ValidationResult],
        total_fields: usize,
        populated_fields: usize,
    ) -> QualityMetrics {
        let error_count = validation_results
            .iter()
            .filter(|r| !r.passed && r.severity == "error")
            .count();
        
        let warning_count = validation_results
            .iter()
            .filter(|r| !r.passed && r.severity == "warning")
            .count();

        let completeness_score = if total_fields > 0 {
            populated_fields as f64 / total_fields as f64
        } else {
            1.0
        };

        let accuracy_score = if !validation_results.is_empty() {
            validation_results.iter().filter(|r| r.passed).count() as f64 / validation_results.len() as f64
        } else {
            1.0
        };

        // Simple consistency score (can be enhanced with cross-field validation)
        let consistency_score = if error_count == 0 { 1.0 } else { 0.8 };

        let overall_score = (completeness_score + accuracy_score + consistency_score) / 3.0;

        QualityMetrics {
            overall_score,
            completeness_score,
            consistency_score,
            accuracy_score,
            total_fields,
            populated_fields,
            error_count,
            warning_count,
        }
    }

    /// Check if document meets quality threshold
    pub fn meets_quality_threshold(&self, metrics: &QualityMetrics) -> bool {
        metrics.overall_score >= self.min_quality_threshold
    }
}

impl Default for DocumentValidator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_document_validator_creation() {
        let validator = DocumentValidator::new();
        assert_eq!(validator.min_quality_threshold, 0.8);
    }

    #[test]
    fn test_column_validator_creation() {
        let mapping_config = MappingConfiguration {
            inventory_mappings: None,
            poam_mappings: None,
            ssp_sections: None,
            controls: None,
            documents: None,
        };

        let validator = ColumnValidator::new(mapping_config);
        assert_eq!(validator.min_quality_threshold, 0.8);
        assert_eq!(validator.performance_target_ms, 50);
    }

    #[test]
    fn test_validation_status_ordering() {
        assert!(ValidationSeverity::Critical > ValidationSeverity::Error);
        assert!(ValidationSeverity::Error > ValidationSeverity::Warning);
        assert!(ValidationSeverity::Warning > ValidationSeverity::Info);
    }

    #[test]
    fn test_boolean_validation() {
        let validator = ColumnValidator::new(MappingConfiguration {
            inventory_mappings: None,
            poam_mappings: None,
            ssp_sections: None,
            controls: None,
            documents: None,
        });

        let valid_data = vec![
            json!("yes"),
            json!("no"),
            json!("true"),
            json!("false"),
            json!("1"),
            json!("0"),
        ];

        let result = validator.validate_boolean_values(&valid_data).unwrap();
        assert_eq!(result.0, ValidationStatus::Valid);

        let invalid_data = vec![
            json!("maybe"),
            json!("invalid"),
            json!("2"),
        ];

        let result = validator.validate_boolean_values(&invalid_data).unwrap();
        assert_eq!(result.0, ValidationStatus::Invalid);
    }

    #[test]
    fn test_date_validation() {
        let validator = ColumnValidator::new(MappingConfiguration {
            inventory_mappings: None,
            poam_mappings: None,
            ssp_sections: None,
            controls: None,
            documents: None,
        });

        let valid_data = vec![
            json!("2023-12-25"),
            json!("12/25/2023"),
            json!("25/12/2023"),
        ];

        let result = validator.validate_date_values(&valid_data).unwrap();
        assert_eq!(result.0, ValidationStatus::Valid);

        let invalid_data = vec![
            json!("invalid-date"),
            json!("2023-13-45"),
            json!("not a date"),
        ];

        let result = validator.validate_date_values(&invalid_data).unwrap();
        assert_eq!(result.0, ValidationStatus::Invalid);
    }

    #[test]
    fn test_ip_address_validation() {
        let validator = ColumnValidator::new(MappingConfiguration {
            inventory_mappings: None,
            poam_mappings: None,
            ssp_sections: None,
            controls: None,
            documents: None,
        });

        let valid_data = vec![
            json!("192.168.1.1"),
            json!("10.0.0.1"),
            json!("2001:db8::1"),
        ];

        let result = validator.validate_ip_address_values(&valid_data).unwrap();
        assert_eq!(result.0, ValidationStatus::Valid);

        let invalid_data = vec![
            json!("256.256.256.256"),
            json!("not.an.ip.address"),
            json!("192.168.1"),
        ];

        let result = validator.validate_ip_address_values(&invalid_data).unwrap();
        assert_eq!(result.0, ValidationStatus::Invalid);
    }

    #[test]
    fn test_email_validation() {
        let validator = ColumnValidator::new(MappingConfiguration {
            inventory_mappings: None,
            poam_mappings: None,
            ssp_sections: None,
            controls: None,
            documents: None,
        });

        let valid_data = vec![
            json!("user@example.com"),
            json!("test.email@domain.org"),
            json!("admin@company.gov"),
        ];

        let result = validator.validate_email_values(&valid_data).unwrap();
        assert_eq!(result.0, ValidationStatus::Valid);

        let invalid_data = vec![
            json!("invalid-email"),
            json!("@domain.com"),
            json!("user@"),
        ];

        let result = validator.validate_email_values(&invalid_data).unwrap();
        assert_eq!(result.0, ValidationStatus::Invalid);
    }

    #[test]
    fn test_unique_identifier_validation() {
        let validator = ColumnValidator::new(MappingConfiguration {
            inventory_mappings: None,
            poam_mappings: None,
            ssp_sections: None,
            controls: None,
            documents: None,
        });

        let valid_data = vec![
            json!("asset-001"),
            json!("component_123"),
            json!("system-abc"),
        ];

        let result = validator.validate_unique_identifier_values(&valid_data).unwrap();
        assert_eq!(result.0, ValidationStatus::Valid);

        let duplicate_data = vec![
            json!("asset-001"),
            json!("asset-002"),
            json!("asset-001"), // Duplicate
        ];

        let result = validator.validate_unique_identifier_values(&duplicate_data).unwrap();
        assert_eq!(result.0, ValidationStatus::Invalid);
        assert!(result.1.contains("Duplicate"));
    }

    #[test]
    fn test_similarity_calculation() {
        let validator = ColumnValidator::new(MappingConfiguration {
            inventory_mappings: None,
            poam_mappings: None,
            ssp_sections: None,
            controls: None,
            documents: None,
        });

        let similarity = validator.calculate_similarity("Asset Name", "Asset_Name");
        assert!(similarity > 0.6, "Expected similarity > 0.6, got {}", similarity);

        let similarity = validator.calculate_similarity("Asset Name", "Component Type");
        assert!(similarity < 0.5, "Expected similarity < 0.5, got {}", similarity);

        // Test exact match
        let similarity = validator.calculate_similarity("Asset Name", "Asset Name");
        assert_eq!(similarity, 1.0);

        // Test completely different strings
        let similarity = validator.calculate_similarity("Asset", "xyz123");
        assert!(similarity < 0.3);
    }

    #[test]
    fn test_confidence_scorer_creation() {
        let scorer = ConfidenceScorer::new();
        assert_eq!(scorer.scoring_config.min_acceptable_score, 0.5);
        assert_eq!(scorer.scoring_config.auto_accept_threshold, 0.9);
        assert_eq!(scorer.threshold_config.high_confidence, 0.9);
    }

    #[test]
    fn test_threshold_status_determination() {
        let scorer = ConfidenceScorer::new();

        assert_eq!(scorer.determine_threshold_status(0.95), ThresholdStatus::HighConfidence);
        assert_eq!(scorer.determine_threshold_status(0.8), ThresholdStatus::MediumConfidence);
        assert_eq!(scorer.determine_threshold_status(0.6), ThresholdStatus::LowConfidence);
        assert_eq!(scorer.determine_threshold_status(0.3), ThresholdStatus::VeryLowConfidence);
    }

    #[test]
    fn test_data_type_detection() {
        let scorer = ConfidenceScorer::new();

        assert_eq!(scorer.detect_data_type(&json!(true)), "boolean");
        assert_eq!(scorer.detect_data_type(&json!(42)), "number");
        assert_eq!(scorer.detect_data_type(&json!("test@example.com")), "email");
        assert_eq!(scorer.detect_data_type(&json!("https://example.com")), "url");
        assert_eq!(scorer.detect_data_type(&json!("192.168.1.1")), "ip_address");
        assert_eq!(scorer.detect_data_type(&json!("2023-12-25")), "date");
        assert_eq!(scorer.detect_data_type(&json!("regular text")), "string");
    }

    #[test]
    fn test_expected_type_inference() {
        let scorer = ConfidenceScorer::new();

        // Test each case individually
        let date_result = scorer.infer_expected_type("creation_date");
        assert_eq!(date_result, "date", "Expected 'date' for 'creation_date', got '{}'", date_result);

        let email_result = scorer.infer_expected_type("contact_email");
        assert_eq!(email_result, "email", "Expected 'email' for 'contact_email', got '{}'", email_result);

        let url_result = scorer.infer_expected_type("website_url");
        assert_eq!(url_result, "url", "Expected 'url' for 'website_url', got '{}'", url_result);

        let ip_result = scorer.infer_expected_type("ip_address");
        assert_eq!(ip_result, "ip_address", "Expected 'ip_address' for 'ip_address', got '{}'", ip_result);

        let number_result = scorer.infer_expected_type("asset_count");
        assert_eq!(number_result, "number", "Expected 'number' for 'asset_count', got '{}'", number_result);

        let boolean_result = scorer.infer_expected_type("is_enabled");
        assert_eq!(boolean_result, "boolean", "Expected 'boolean' for 'is_enabled', got '{}'", boolean_result);

        let string_result = scorer.infer_expected_type("description");
        assert_eq!(string_result, "string", "Expected 'string' for 'description', got '{}'", string_result);
    }

    #[test]
    fn test_simple_similarity_calculation() {
        let scorer = ConfidenceScorer::new();

        let similarity = scorer.calculate_simple_similarity("Asset Name", "Asset_Name");
        println!("Asset Name vs Asset_Name: {}", similarity);
        assert!(similarity > 0.6, "Expected similarity > 0.6, got {}", similarity);

        let similarity = scorer.calculate_simple_similarity("Asset Name", "Component Type");
        println!("Asset Name vs Component Type: {}", similarity);
        assert!(similarity < 0.5, "Expected similarity < 0.5, got {}", similarity);

        let similarity = scorer.calculate_simple_similarity("test", "test");
        println!("test vs test: {}", similarity);
        assert_eq!(similarity, 1.0);

        // Test completely different strings
        let similarity = scorer.calculate_simple_similarity("Asset", "xyz123");
        println!("Asset vs xyz123: {}", similarity);
        assert!(similarity < 0.3, "Expected similarity < 0.3, got {}", similarity);
    }

    #[test]
    fn test_risk_factor_identification() {
        let scorer = ConfidenceScorer::new();
        let mut factor_scores = HashMap::new();
        factor_scores.insert(ConfidenceFactor::StringSimilarity, 0.2); // Low similarity
        factor_scores.insert(ConfidenceFactor::DataTypeMatch, 0.3); // Low data type match

        let risk_factors = scorer.identify_risk_factors(&factor_scores, 0.25).unwrap();

        assert!(!risk_factors.is_empty());
        assert!(risk_factors.iter().any(|r| matches!(r.risk_type, RiskType::LowStringSimilarity)));
        assert!(risk_factors.iter().any(|r| matches!(r.risk_type, RiskType::DataTypeMismatch)));
        assert!(risk_factors.iter().any(|r| matches!(r.risk_type, RiskType::AmbiguousMapping)));
    }

    #[test]
    fn test_weighted_score_calculation() {
        let scorer = ConfidenceScorer::new();
        let mut factor_contributions = HashMap::new();

        factor_contributions.insert(ConfidenceFactor::StringSimilarity, FactorContribution {
            raw_score: 0.8,
            weight: 0.35,
            weighted_contribution: 0.28,
            explanation: "Test".to_string(),
        });

        factor_contributions.insert(ConfidenceFactor::DataTypeMatch, FactorContribution {
            raw_score: 0.9,
            weight: 0.25,
            weighted_contribution: 0.225,
            explanation: "Test".to_string(),
        });

        let result = scorer.calculate_weighted_score(&factor_contributions).unwrap();

        assert!((result.total_weighted_score - 0.505).abs() < 0.001);
        assert!((result.total_weight - 0.6).abs() < 0.001);
        assert!((result.base_score - 0.8416).abs() < 0.01);
    }

    #[test]
    fn test_confidence_adjustments() {
        let scorer = ConfidenceScorer::new();
        let expected_columns = vec!["Asset Name".to_string()];
        let mut factor_scores = HashMap::new();
        factor_scores.insert(ConfidenceFactor::StringSimilarity, 0.9);
        factor_scores.insert(ConfidenceFactor::ContextRelevance, 0.85);

        let (adjusted_score, adjustments) = scorer.apply_confidence_adjustments(
            0.8,
            "Asset Name", // Exact match
            &expected_columns,
            &factor_scores,
        ).unwrap();

        assert!(adjusted_score > 0.8); // Should be higher due to exact match bonus
        assert!(!adjustments.is_empty());
        assert!(adjustments.iter().any(|a| matches!(a.adjustment_type, AdjustmentType::ExactMatchBonus)));
    }
}
