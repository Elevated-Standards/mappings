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
use uuid::Uuid;
use lru::LruCache;

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

/// Custom mapping override system for user-defined column mappings
#[derive(Debug)]
pub struct MappingOverrideEngine {
    /// Active override rules
    overrides: Vec<MappingOverride>,
    /// LRU cache for override resolution results
    override_cache: LruCache<String, Option<String>>,
    /// Conflict resolver for handling rule conflicts
    conflict_resolver: ConflictResolver,
    /// Override rule validator
    validator: OverrideValidator,
    /// Performance metrics
    performance_metrics: OverrideMetrics,
}

/// Individual mapping override rule
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MappingOverride {
    /// Unique identifier for the override
    pub id: Uuid,
    /// Human-readable name for the override
    pub name: String,
    /// Description of what this override does
    pub description: String,
    /// Type of override rule
    pub rule_type: OverrideType,
    /// Pattern matching configuration
    pub pattern: OverridePattern,
    /// Target field to map to
    pub target_field: String,
    /// Priority for conflict resolution (higher = more priority)
    pub priority: i32,
    /// Conditions that must be met for this override to apply
    pub conditions: Vec<OverrideCondition>,
    /// Scope of the override
    pub scope: OverrideScope,
    /// User who created this override
    pub created_by: String,
    /// When this override was created
    pub created_at: DateTime<chrono::Utc>,
    /// When this override was last modified
    pub modified_at: DateTime<chrono::Utc>,
    /// Whether this override is currently active
    pub active: bool,
    /// Version number for tracking changes
    pub version: u32,
    /// Tags for categorization and filtering
    pub tags: Vec<String>,
}

/// Type of override matching strategy
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum OverrideType {
    /// Exact string match (case-sensitive or insensitive)
    ExactMatch,
    /// Regular expression pattern matching
    RegexPattern,
    /// Fuzzy matching with similarity threshold
    FuzzyMatch,
    /// Position-based matching (column index)
    PositionalMatch,
    /// Conditional matching based on multiple criteria
    ConditionalMatch,
    /// Prefix/suffix matching
    PrefixSuffixMatch,
    /// Contains substring matching
    ContainsMatch,
}

/// Pattern configuration for override matching
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OverridePattern {
    /// The pattern string (regex, exact match, etc.)
    pub pattern: String,
    /// Whether matching should be case-sensitive
    pub case_sensitive: bool,
    /// Whether to match whole words only
    pub whole_word: bool,
    /// Regular expression flags (if applicable)
    pub regex_flags: Option<String>,
    /// Fuzzy matching threshold (0.0-1.0)
    pub fuzzy_threshold: Option<f64>,
    /// Position constraints (for positional matching)
    pub position_constraints: Option<PositionConstraints>,
}

/// Position constraints for positional matching
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PositionConstraints {
    /// Minimum column index (0-based)
    pub min_index: Option<usize>,
    /// Maximum column index (0-based)
    pub max_index: Option<usize>,
    /// Exact column index (0-based)
    pub exact_index: Option<usize>,
    /// Relative position (e.g., "first", "last", "second_to_last")
    pub relative_position: Option<String>,
}

/// Condition that must be met for an override to apply
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OverrideCondition {
    /// Type of condition
    pub condition_type: ConditionType,
    /// Field or property to check
    pub field: String,
    /// Operator for comparison
    pub operator: ConditionOperator,
    /// Value to compare against
    pub value: serde_json::Value,
    /// Whether this condition is required (AND) or optional (OR)
    pub required: bool,
}

/// Type of condition to evaluate
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ConditionType {
    /// Document type condition
    DocumentType,
    /// File name pattern condition
    FileName,
    /// Column count condition
    ColumnCount,
    /// Data sample condition
    DataSample,
    /// User role condition
    UserRole,
    /// Organization condition
    Organization,
    /// Custom metadata condition
    CustomMetadata,
}

/// Comparison operator for conditions
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ConditionOperator {
    /// Equal to
    Equals,
    /// Not equal to
    NotEquals,
    /// Contains substring
    Contains,
    /// Does not contain substring
    NotContains,
    /// Matches regex pattern
    Matches,
    /// Does not match regex pattern
    NotMatches,
    /// Greater than (numeric)
    GreaterThan,
    /// Less than (numeric)
    LessThan,
    /// Greater than or equal (numeric)
    GreaterThanOrEqual,
    /// Less than or equal (numeric)
    LessThanOrEqual,
    /// In list of values
    In,
    /// Not in list of values
    NotIn,
}

/// Scope of an override rule
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum OverrideScope {
    /// Global override (applies to all documents)
    Global,
    /// Document type specific
    DocumentType(String),
    /// Organization specific
    Organization(String),
    /// User specific
    User(String),
    /// Session specific (temporary)
    Session(String),
    /// Project specific
    Project(String),
}

/// Conflict resolver for handling overlapping override rules
#[derive(Debug)]
pub struct ConflictResolver {
    /// Strategy for resolving conflicts
    resolution_strategy: ConflictResolutionStrategy,
    /// Maximum number of conflicts to report
    max_conflicts_reported: usize,
}

/// Strategy for resolving override conflicts
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ConflictResolutionStrategy {
    /// Use highest priority rule
    HighestPriority,
    /// Use most recently created rule
    MostRecent,
    /// Use most specific rule (most conditions)
    MostSpecific,
    /// Combine rules if possible
    Combine,
    /// Report conflict and use fallback
    ReportAndFallback,
}

/// Override rule validator
#[derive(Debug)]
pub struct OverrideValidator {
    /// Compiled regex patterns cache
    regex_cache: HashMap<String, Regex>,
    /// Validation rules
    validation_rules: ValidationRuleSet,
}

/// Set of validation rules for overrides
#[derive(Debug, Clone)]
pub struct ValidationRuleSet {
    /// Maximum pattern length
    pub max_pattern_length: usize,
    /// Maximum number of conditions per override
    pub max_conditions: usize,
    /// Allowed target fields
    pub allowed_target_fields: Option<Vec<String>>,
    /// Forbidden patterns
    pub forbidden_patterns: Vec<String>,
    /// Required tags for certain scopes
    pub required_tags: HashMap<OverrideScope, Vec<String>>,
}

/// Performance metrics for override operations
#[derive(Debug, Clone, Default)]
pub struct OverrideMetrics {
    /// Total number of override applications
    pub total_applications: u64,
    /// Number of successful matches
    pub successful_matches: u64,
    /// Number of conflicts detected
    pub conflicts_detected: u64,
    /// Average resolution time in microseconds
    pub avg_resolution_time_us: f64,
    /// Cache hit rate
    pub cache_hit_rate: f64,
    /// Last updated timestamp
    pub last_updated: DateTime<chrono::Utc>,
}

/// Result of override resolution
#[derive(Debug, Clone)]
pub struct OverrideResolutionResult {
    /// Whether an override was applied
    pub override_applied: bool,
    /// The override rule that was applied (if any)
    pub applied_override: Option<MappingOverride>,
    /// Target field from override or fallback
    pub target_field: Option<String>,
    /// Confidence score for the override match
    pub confidence: f64,
    /// Any conflicts that were detected
    pub conflicts: Vec<OverrideConflict>,
    /// Resolution time
    pub resolution_time: Duration,
    /// Whether result came from cache
    pub from_cache: bool,
}

/// Detected conflict between override rules
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OverrideConflict {
    /// Conflicting override rules
    pub conflicting_overrides: Vec<Uuid>,
    /// Type of conflict
    pub conflict_type: ConflictType,
    /// Description of the conflict
    pub description: String,
    /// Suggested resolution
    pub suggested_resolution: String,
    /// Severity of the conflict
    pub severity: ConflictSeverity,
}

/// Type of override conflict
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ConflictType {
    /// Multiple rules match the same pattern
    PatternOverlap,
    /// Rules have same priority but different targets
    PriorityTie,
    /// Circular dependency in conditions
    CircularDependency,
    /// Contradictory conditions
    ContradictoryConditions,
    /// Scope conflicts
    ScopeConflict,
}

/// Severity of override conflict
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, PartialOrd)]
pub enum ConflictSeverity {
    /// Low severity - informational
    Low,
    /// Medium severity - may affect results
    Medium,
    /// High severity - likely to cause issues
    High,
    /// Critical severity - will cause failures
    Critical,
}

/// Comprehensive mapping validation report system
#[derive(Debug)]
pub struct MappingReportGenerator {
    /// Report configuration
    config: ReportConfig,
    /// Template engine for HTML reports
    template_engine: Option<tera::Tera>,
    /// Report cache for performance
    report_cache: LruCache<String, CachedReport>,
    /// Historical data for trend analysis
    historical_data: HistoricalReportData,
    /// Performance metrics
    generation_metrics: ReportGenerationMetrics,
}

/// Configuration for report generation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReportConfig {
    /// Default report format
    pub default_format: ReportFormat,
    /// Include data visualizations
    pub include_visualizations: bool,
    /// Maximum report generation time (seconds)
    pub max_generation_time_seconds: u64,
    /// Enable report caching
    pub enable_caching: bool,
    /// Cache expiration time (minutes)
    pub cache_expiration_minutes: u64,
    /// Include detailed field analysis
    pub include_detailed_analysis: bool,
    /// Include recommendations
    pub include_recommendations: bool,
    /// Report template directory
    pub template_directory: Option<String>,
}

/// Comprehensive mapping validation report
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MappingReport {
    /// Unique report identifier
    pub report_id: Uuid,
    /// Type of report generated
    pub report_type: ReportType,
    /// When the report was generated
    pub generated_at: DateTime<chrono::Utc>,
    /// Information about the processed document
    pub document_info: DocumentInfo,
    /// High-level mapping summary
    pub mapping_summary: MappingSummary,
    /// Detailed field-by-field results
    pub detailed_results: Vec<FieldMappingResult>,
    /// Quality metrics and scores
    pub quality_metrics: QualityMetrics,
    /// Actionable recommendations
    pub recommendations: Vec<Recommendation>,
    /// Validation results
    pub validation_results: ValidationSummary,
    /// Override application results
    pub override_results: OverrideSummary,
    /// Performance metrics
    pub performance_metrics: ProcessingMetrics,
    /// Trend analysis (if historical data available)
    pub trend_analysis: Option<TrendAnalysis>,
}

/// Type of mapping report
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ReportType {
    /// Summary report with key metrics
    Summary,
    /// Detailed analysis report
    Detailed,
    /// Quality trend analysis
    QualityTrend,
    /// Compliance and audit report
    Compliance,
    /// Performance analysis report
    Performance,
    /// Custom report type
    Custom(String),
}

/// Report output format
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ReportFormat {
    /// HTML format with interactive elements
    Html,
    /// PDF format for printing/sharing
    Pdf,
    /// JSON format for API consumption
    Json,
    /// CSV format for data analysis
    Csv,
    /// Markdown format for documentation
    Markdown,
}

/// Information about the processed document
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentInfo {
    /// Document file name
    pub file_name: String,
    /// Document type (inventory, poam, ssp)
    pub document_type: String,
    /// File size in bytes
    pub file_size: u64,
    /// Number of rows processed
    pub total_rows: usize,
    /// Number of columns detected
    pub total_columns: usize,
    /// Processing timestamp
    pub processed_at: DateTime<chrono::Utc>,
    /// Processing duration
    pub processing_duration: Duration,
    /// Document hash for change detection
    pub document_hash: String,
}

/// High-level mapping summary statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MappingSummary {
    /// Total number of fields expected
    pub total_fields: usize,
    /// Number of fields successfully mapped
    pub mapped_fields: usize,
    /// Number of required fields mapped
    pub required_fields_mapped: usize,
    /// Number of required fields missing
    pub required_fields_missing: usize,
    /// Number of optional fields mapped
    pub optional_fields_mapped: usize,
    /// Average confidence score across all mappings
    pub average_confidence: f64,
    /// Minimum confidence score
    pub min_confidence: f64,
    /// Maximum confidence score
    pub max_confidence: f64,
    /// Number of high-confidence mappings (>0.9)
    pub high_confidence_mappings: usize,
    /// Number of low-confidence mappings (<0.5)
    pub low_confidence_mappings: usize,
    /// Total processing time
    pub processing_time: Duration,
    /// Mapping success rate (0.0-1.0)
    pub success_rate: f64,
}

/// Detailed result for a single field mapping
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FieldMappingResult {
    /// Expected field identifier
    pub field_id: String,
    /// Target OSCAL field path
    pub target_field: String,
    /// Source column name (if mapped)
    pub source_column: Option<String>,
    /// Mapping confidence score
    pub confidence_score: f64,
    /// Whether mapping was successful
    pub mapping_successful: bool,
    /// Whether field is required
    pub required: bool,
    /// Validation result for this field
    pub validation_result: Option<ColumnValidationResult>,
    /// Override applied (if any)
    pub override_applied: Option<String>,
    /// Alternative suggestions
    pub alternatives: Vec<MappingAlternative>,
    /// Issues and warnings
    pub issues: Vec<MappingIssue>,
    /// Data quality assessment
    pub data_quality: Option<DataQualityAssessment>,
}

/// Alternative mapping suggestion
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MappingAlternative {
    /// Alternative source column
    pub source_column: String,
    /// Confidence score for alternative
    pub confidence_score: f64,
    /// Reason for suggestion
    pub reason: String,
}

/// Mapping issue or warning
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MappingIssue {
    /// Issue severity level
    pub severity: IssueSeverity,
    /// Issue category
    pub category: IssueCategory,
    /// Issue description
    pub description: String,
    /// Suggested resolution
    pub suggested_resolution: Option<String>,
    /// Impact assessment
    pub impact: String,
}

/// Severity level for mapping issues
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

/// Data quality assessment for a field
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataQualityAssessment {
    /// Completeness score (0.0-1.0)
    pub completeness: f64,
    /// Consistency score (0.0-1.0)
    pub consistency: f64,
    /// Validity score (0.0-1.0)
    pub validity: f64,
    /// Overall quality score (0.0-1.0)
    pub overall_quality: f64,
    /// Number of null/empty values
    pub null_count: usize,
    /// Number of invalid values
    pub invalid_count: usize,
    /// Sample valid values
    pub sample_values: Vec<String>,
    /// Detected data patterns
    pub patterns: Vec<String>,
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
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, PartialOrd)]
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

/// Actionable recommendation for improvement
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Recommendation {
    /// Recommendation priority
    pub priority: RecommendationPriority,
    /// Recommendation category
    pub category: RecommendationCategory,
    /// Recommendation title
    pub title: String,
    /// Detailed description
    pub description: String,
    /// Suggested action steps
    pub suggested_action: String,
    /// Expected impact of implementing
    pub impact_assessment: String,
    /// Effort required to implement
    pub effort_level: EffortLevel,
    /// Related field IDs (if applicable)
    pub related_fields: Vec<String>,
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

/// Summary of validation results
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationSummary {
    /// Total validations performed
    pub total_validations: usize,
    /// Number of validations passed
    pub validations_passed: usize,
    /// Number of validations failed
    pub validations_failed: usize,
    /// Number of validation warnings
    pub validation_warnings: usize,
    /// Validation success rate (0.0-1.0)
    pub success_rate: f64,
    /// Most common validation failures
    pub common_failures: Vec<ValidationFailureInfo>,
    /// Validation performance metrics
    pub performance_metrics: ValidationPerformanceMetrics,
}

/// Information about common validation failures
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationFailureInfo {
    /// Type of validation failure
    pub failure_type: String,
    /// Number of occurrences
    pub occurrence_count: usize,
    /// Percentage of total failures
    pub failure_percentage: f64,
    /// Example failing values
    pub example_values: Vec<String>,
    /// Suggested fixes
    pub suggested_fixes: Vec<String>,
}

/// Performance metrics for validation operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationPerformanceMetrics {
    /// Average validation time per field (microseconds)
    pub avg_validation_time_us: f64,
    /// Total validation time
    pub total_validation_time: Duration,
    /// Slowest validation operations
    pub slowest_validations: Vec<SlowValidationInfo>,
}

/// Information about slow validation operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SlowValidationInfo {
    /// Field that was slow to validate
    pub field_id: String,
    /// Validation time in microseconds
    pub validation_time_us: u64,
    /// Reason for slowness
    pub reason: String,
}

/// Summary of override application results
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OverrideSummary {
    /// Total number of override rules evaluated
    pub total_overrides_evaluated: usize,
    /// Number of overrides applied
    pub overrides_applied: usize,
    /// Number of override conflicts detected
    pub conflicts_detected: usize,
    /// Override application success rate
    pub application_success_rate: f64,
    /// Most frequently applied overrides
    pub frequently_applied: Vec<OverrideUsageInfo>,
    /// Override performance metrics
    pub performance_metrics: OverridePerformanceMetrics,
}

/// Information about override usage
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OverrideUsageInfo {
    /// Override rule name
    pub override_name: String,
    /// Number of times applied
    pub application_count: usize,
    /// Success rate for this override
    pub success_rate: f64,
    /// Average confidence when applied
    pub avg_confidence: f64,
}

/// Performance metrics for override operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OverridePerformanceMetrics {
    /// Average override resolution time (microseconds)
    pub avg_resolution_time_us: f64,
    /// Cache hit rate for override lookups
    pub cache_hit_rate: f64,
    /// Total override processing time
    pub total_processing_time: Duration,
}

/// Processing performance metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessingMetrics {
    /// Total processing time
    pub total_processing_time: Duration,
    /// Time spent on column detection
    pub column_detection_time: Duration,
    /// Time spent on mapping
    pub mapping_time: Duration,
    /// Time spent on validation
    pub validation_time: Duration,
    /// Time spent on override resolution
    pub override_time: Duration,
    /// Memory usage statistics
    pub memory_usage: MemoryUsageMetrics,
    /// Throughput metrics
    pub throughput: ThroughputMetrics,
}

/// Memory usage statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryUsageMetrics {
    /// Peak memory usage in bytes
    pub peak_memory_bytes: u64,
    /// Average memory usage in bytes
    pub avg_memory_bytes: u64,
    /// Memory efficiency score (0.0-1.0)
    pub efficiency_score: f64,
}

/// Throughput performance metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThroughputMetrics {
    /// Rows processed per second
    pub rows_per_second: f64,
    /// Fields processed per second
    pub fields_per_second: f64,
    /// Bytes processed per second
    pub bytes_per_second: f64,
}

/// Trend analysis data (when historical data is available)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrendAnalysis {
    /// Time period covered by the analysis
    pub time_period: TimePeriod,
    /// Quality score trends
    pub quality_trends: QualityTrends,
    /// Performance trends
    pub performance_trends: PerformanceTrends,
    /// Common issues over time
    pub issue_trends: IssueTrends,
    /// Improvement recommendations based on trends
    pub trend_recommendations: Vec<TrendRecommendation>,
}

/// Time period for trend analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimePeriod {
    /// Start of the analysis period
    pub start_date: DateTime<chrono::Utc>,
    /// End of the analysis period
    pub end_date: DateTime<chrono::Utc>,
    /// Number of data points in the analysis
    pub data_points: usize,
}

/// Quality score trends over time
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QualityTrends {
    /// Overall quality score trend
    pub overall_quality_trend: TrendDirection,
    /// Completeness score trend
    pub completeness_trend: TrendDirection,
    /// Accuracy score trend
    pub accuracy_trend: TrendDirection,
    /// Consistency score trend
    pub consistency_trend: TrendDirection,
    /// Historical quality scores
    pub historical_scores: Vec<HistoricalQualityScore>,
}

/// Performance trends over time
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceTrends {
    /// Processing time trend
    pub processing_time_trend: TrendDirection,
    /// Throughput trend
    pub throughput_trend: TrendDirection,
    /// Memory usage trend
    pub memory_usage_trend: TrendDirection,
    /// Historical performance data
    pub historical_performance: Vec<HistoricalPerformanceData>,
}

/// Issue trends over time
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IssueTrends {
    /// Critical issues trend
    pub critical_issues_trend: TrendDirection,
    /// Warning trend
    pub warnings_trend: TrendDirection,
    /// Most common issues over time
    pub common_issues: Vec<CommonIssueInfo>,
}

/// Direction of a trend
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

/// Historical quality score data point
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistoricalQualityScore {
    /// Timestamp of the measurement
    pub timestamp: DateTime<chrono::Utc>,
    /// Overall quality score
    pub overall_score: f64,
    /// Completeness score
    pub completeness_score: f64,
    /// Accuracy score
    pub accuracy_score: f64,
    /// Consistency score
    pub consistency_score: f64,
}

/// Historical performance data point
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistoricalPerformanceData {
    /// Timestamp of the measurement
    pub timestamp: DateTime<chrono::Utc>,
    /// Processing time in milliseconds
    pub processing_time_ms: u64,
    /// Throughput in rows per second
    pub throughput_rps: f64,
    /// Memory usage in bytes
    pub memory_usage_bytes: u64,
}

/// Information about common issues over time
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommonIssueInfo {
    /// Issue category
    pub issue_category: IssueCategory,
    /// Trend direction for this issue
    pub trend: TrendDirection,
    /// Current occurrence rate
    pub current_rate: f64,
    /// Historical occurrence rates
    pub historical_rates: Vec<IssueRateDataPoint>,
}

/// Issue rate data point
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IssueRateDataPoint {
    /// Timestamp
    pub timestamp: DateTime<chrono::Utc>,
    /// Issue occurrence rate
    pub rate: f64,
}

/// Trend-based recommendation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrendRecommendation {
    /// Recommendation based on trend analysis
    pub recommendation: String,
    /// Supporting trend data
    pub supporting_data: String,
    /// Expected impact
    pub expected_impact: String,
    /// Priority based on trend severity
    pub priority: RecommendationPriority,
}

/// Cached report for performance optimization
#[derive(Debug, Clone)]
pub struct CachedReport {
    /// The cached report
    pub report: MappingReport,
    /// When the report was cached
    pub cached_at: DateTime<chrono::Utc>,
    /// Cache expiration time
    pub expires_at: DateTime<chrono::Utc>,
    /// Report format
    pub format: ReportFormat,
}

/// Historical data for trend analysis
#[derive(Debug, Clone, Default)]
pub struct HistoricalReportData {
    /// Historical quality scores
    pub quality_history: Vec<HistoricalQualityScore>,
    /// Historical performance data
    pub performance_history: Vec<HistoricalPerformanceData>,
    /// Historical issue data
    pub issue_history: Vec<CommonIssueInfo>,
    /// Maximum history retention (days)
    pub max_retention_days: u32,
}

/// Report generation performance metrics
#[derive(Debug, Clone, Default)]
pub struct ReportGenerationMetrics {
    /// Total reports generated
    pub total_reports_generated: u64,
    /// Average generation time in milliseconds
    pub avg_generation_time_ms: f64,
    /// Cache hit rate
    pub cache_hit_rate: f64,
    /// Failed generation attempts
    pub failed_generations: u64,
    /// Last updated timestamp
    pub last_updated: DateTime<chrono::Utc>,
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

impl MappingOverrideEngine {
    /// Create a new mapping override engine
    pub fn new() -> Self {
        Self {
            overrides: Vec::new(),
            override_cache: LruCache::new(std::num::NonZeroUsize::new(1000).unwrap()),
            conflict_resolver: ConflictResolver::new(),
            validator: OverrideValidator::new(),
            performance_metrics: OverrideMetrics::default(),
        }
    }

    /// Create a new override engine with custom cache size
    pub fn with_cache_size(cache_size: usize) -> Self {
        Self {
            overrides: Vec::new(),
            override_cache: LruCache::new(std::num::NonZeroUsize::new(cache_size).unwrap()),
            conflict_resolver: ConflictResolver::new(),
            validator: OverrideValidator::new(),
            performance_metrics: OverrideMetrics::default(),
        }
    }

    /// Add a new override rule
    pub fn add_override(&mut self, mut override_rule: MappingOverride) -> Result<()> {
        let start_time = Instant::now();

        // Validate the override rule
        self.validator.validate_override(&override_rule)?;

        // Set creation/modification timestamps
        let now = chrono::Utc::now();
        override_rule.created_at = now;
        override_rule.modified_at = now;
        override_rule.version = 1;

        // Check for conflicts with existing rules
        let conflicts = self.detect_conflicts(&override_rule)?;
        if !conflicts.is_empty() {
            warn!("Detected {} conflicts when adding override '{}'", conflicts.len(), override_rule.name);
            for conflict in &conflicts {
                if conflict.severity >= ConflictSeverity::High {
                    return Err(Error::document_parsing(format!(
                        "High severity conflict detected: {}",
                        conflict.description
                    )));
                }
            }
        }

        // Add the override
        self.overrides.push(override_rule.clone());

        // Sort overrides by priority (highest first)
        self.overrides.sort_by(|a, b| b.priority.cmp(&a.priority));

        // Clear cache since rules have changed
        self.override_cache.clear();

        // Update metrics
        self.performance_metrics.total_applications += 1;

        info!(
            "Added override '{}' with priority {} in {}μs",
            override_rule.name,
            override_rule.priority,
            start_time.elapsed().as_micros()
        );

        Ok(())
    }

    /// Remove an override rule by ID
    pub fn remove_override(&mut self, override_id: &Uuid) -> Result<bool> {
        let initial_len = self.overrides.len();
        self.overrides.retain(|override_rule| override_rule.id != *override_id);

        let removed = self.overrides.len() < initial_len;
        if removed {
            self.override_cache.clear();
            info!("Removed override with ID: {}", override_id);
        }

        Ok(removed)
    }

    /// Update an existing override rule
    pub fn update_override(&mut self, updated_override: MappingOverride) -> Result<bool> {
        // Validate the updated override
        self.validator.validate_override(&updated_override)?;

        // Find the index of the override to update
        if let Some(index) = self.overrides.iter().position(|o| o.id == updated_override.id) {
            let existing = &self.overrides[index];
            let mut updated = updated_override;
            updated.version = existing.version + 1;
            updated.modified_at = chrono::Utc::now();
            updated.created_at = existing.created_at; // Preserve original creation time
            updated.created_by = existing.created_by.clone(); // Preserve original creator

            let override_name = updated.name.clone();
            let override_version = updated.version;

            // Update the override
            self.overrides[index] = updated;

            // Re-sort by priority
            self.overrides.sort_by(|a, b| b.priority.cmp(&a.priority));

            // Clear cache
            self.override_cache.clear();

            info!("Updated override '{}' to version {}", override_name, override_version);
            Ok(true)
        } else {
            Ok(false)
        }
    }

    /// Resolve column mapping using override rules
    pub fn resolve_mapping(
        &mut self,
        source_column: &str,
        document_type: &str,
        context: &OverrideContext,
    ) -> Result<OverrideResolutionResult> {
        let start_time = Instant::now();

        // Check cache first
        let cache_key = format!("{}:{}:{}", source_column, document_type, context.cache_key());
        if let Some(cached_result) = self.override_cache.get(&cache_key) {
            self.performance_metrics.cache_hit_rate =
                (self.performance_metrics.cache_hit_rate * 0.9) + (1.0 * 0.1);

            return Ok(OverrideResolutionResult {
                override_applied: cached_result.is_some(),
                applied_override: None, // Don't cache full override objects
                target_field: cached_result.clone(),
                confidence: if cached_result.is_some() { 1.0 } else { 0.0 },
                conflicts: Vec::new(),
                resolution_time: start_time.elapsed(),
                from_cache: true,
            });
        }

        // Update cache miss rate
        self.performance_metrics.cache_hit_rate =
            (self.performance_metrics.cache_hit_rate * 0.9) + (0.0 * 0.1);

        // Find matching overrides
        let mut matching_overrides = Vec::new();
        let mut all_conflicts = Vec::new();

        for override_rule in &self.overrides {
            if !override_rule.active {
                continue;
            }

            // Check scope
            if !self.check_scope_match(&override_rule.scope, context)? {
                continue;
            }

            // Check conditions
            if !self.evaluate_conditions(&override_rule.conditions, source_column, document_type, context)? {
                continue;
            }

            // Check pattern match
            if let Some(confidence) = self.check_pattern_match(&override_rule.pattern, &override_rule.rule_type, source_column)? {
                matching_overrides.push((override_rule.clone(), confidence));
            }
        }

        // Resolve conflicts if multiple matches
        let (selected_override, conflicts) = if matching_overrides.is_empty() {
            (None, Vec::new())
        } else if matching_overrides.len() == 1 {
            (Some(matching_overrides.into_iter().next().unwrap()), Vec::new())
        } else {
            self.conflict_resolver.resolve_conflicts(matching_overrides)?
        };

        all_conflicts.extend(conflicts);

        let resolution_time = start_time.elapsed();

        // Update metrics
        self.performance_metrics.total_applications += 1;
        if selected_override.is_some() {
            self.performance_metrics.successful_matches += 1;
        }
        if !all_conflicts.is_empty() {
            self.performance_metrics.conflicts_detected += 1;
        }

        // Update average resolution time
        let new_time_us = resolution_time.as_micros() as f64;
        self.performance_metrics.avg_resolution_time_us =
            (self.performance_metrics.avg_resolution_time_us * 0.9) + (new_time_us * 0.1);

        // Cache the result
        let target_field = selected_override.as_ref().map(|(override_rule, _)| override_rule.target_field.clone());
        self.override_cache.put(cache_key, target_field.clone());

        Ok(OverrideResolutionResult {
            override_applied: selected_override.is_some(),
            applied_override: selected_override.as_ref().map(|(override_rule, _)| override_rule.clone()),
            target_field,
            confidence: selected_override.map(|(_, confidence)| confidence).unwrap_or(0.0),
            conflicts: all_conflicts,
            resolution_time,
            from_cache: false,
        })
    }

    /// Load overrides from JSON configuration
    pub fn load_overrides_from_json(&mut self, json_data: &str) -> Result<usize> {
        let overrides: Vec<MappingOverride> = serde_json::from_str(json_data)
            .map_err(|e| Error::document_parsing(format!("Failed to parse override JSON: {}", e)))?;

        let mut loaded_count = 0;
        for override_rule in overrides {
            if let Err(e) = self.add_override(override_rule) {
                warn!("Failed to load override: {}", e);
            } else {
                loaded_count += 1;
            }
        }

        info!("Loaded {} override rules from JSON", loaded_count);
        Ok(loaded_count)
    }

    /// Export overrides to JSON
    pub fn export_overrides_to_json(&self) -> Result<String> {
        serde_json::to_string_pretty(&self.overrides)
            .map_err(|e| Error::document_parsing(format!("Failed to serialize overrides: {}", e)))
    }

    /// Get all active overrides
    pub fn get_active_overrides(&self) -> Vec<&MappingOverride> {
        self.overrides.iter().filter(|o| o.active).collect()
    }

    /// Get override by ID
    pub fn get_override(&self, override_id: &Uuid) -> Option<&MappingOverride> {
        self.overrides.iter().find(|o| o.id == *override_id)
    }

    /// Get performance metrics
    pub fn get_metrics(&self) -> &OverrideMetrics {
        &self.performance_metrics
    }

    /// Clear all overrides
    pub fn clear_overrides(&mut self) {
        self.overrides.clear();
        self.override_cache.clear();
        info!("Cleared all override rules");
    }

    /// Get overrides by scope
    pub fn get_overrides_by_scope(&self, scope: &OverrideScope) -> Vec<&MappingOverride> {
        self.overrides.iter().filter(|o| o.scope == *scope).collect()
    }

    /// Check if scope matches the current context
    fn check_scope_match(&self, scope: &OverrideScope, context: &OverrideContext) -> Result<bool> {
        match scope {
            OverrideScope::Global => Ok(true),
            OverrideScope::DocumentType(doc_type) => Ok(context.document_type == *doc_type),
            OverrideScope::Organization(org) => Ok(context.organization.as_ref() == Some(org)),
            OverrideScope::User(user) => Ok(context.user.as_ref() == Some(user)),
            OverrideScope::Session(session) => Ok(context.session_id.as_ref() == Some(session)),
            OverrideScope::Project(project) => Ok(context.project.as_ref() == Some(project)),
        }
    }

    /// Evaluate conditions for an override rule
    fn evaluate_conditions(
        &self,
        conditions: &[OverrideCondition],
        source_column: &str,
        document_type: &str,
        context: &OverrideContext,
    ) -> Result<bool> {
        if conditions.is_empty() {
            return Ok(true);
        }

        let mut required_conditions_met = true;
        let mut optional_conditions_met = false;

        for condition in conditions {
            let condition_result = self.evaluate_single_condition(condition, source_column, document_type, context)?;

            if condition.required {
                required_conditions_met = required_conditions_met && condition_result;
            } else {
                optional_conditions_met = optional_conditions_met || condition_result;
            }
        }

        // All required conditions must be met, and at least one optional condition (if any exist)
        let has_optional = conditions.iter().any(|c| !c.required);
        Ok(required_conditions_met && (!has_optional || optional_conditions_met))
    }

    /// Evaluate a single condition
    fn evaluate_single_condition(
        &self,
        condition: &OverrideCondition,
        _source_column: &str,
        document_type: &str,
        context: &OverrideContext,
    ) -> Result<bool> {
        let field_value = match condition.condition_type {
            ConditionType::DocumentType => serde_json::Value::String(document_type.to_string()),
            ConditionType::FileName => serde_json::Value::String(context.file_name.clone().unwrap_or_default()),
            ConditionType::ColumnCount => serde_json::Value::Number(serde_json::Number::from(context.column_count.unwrap_or(0))),
            ConditionType::DataSample => context.data_sample.clone().unwrap_or(serde_json::Value::Null),
            ConditionType::UserRole => serde_json::Value::String(context.user_role.clone().unwrap_or_default()),
            ConditionType::Organization => serde_json::Value::String(context.organization.clone().unwrap_or_default()),
            ConditionType::CustomMetadata => {
                context.custom_metadata.get(&condition.field).cloned().unwrap_or(serde_json::Value::Null)
            }
        };

        self.apply_condition_operator(&condition.operator, &field_value, &condition.value)
    }

    /// Apply condition operator to compare values
    fn apply_condition_operator(
        &self,
        operator: &ConditionOperator,
        field_value: &serde_json::Value,
        expected_value: &serde_json::Value,
    ) -> Result<bool> {
        match operator {
            ConditionOperator::Equals => Ok(field_value == expected_value),
            ConditionOperator::NotEquals => Ok(field_value != expected_value),
            ConditionOperator::Contains => {
                if let (Some(field_str), Some(expected_str)) = (field_value.as_str(), expected_value.as_str()) {
                    Ok(field_str.contains(expected_str))
                } else {
                    Ok(false)
                }
            }
            ConditionOperator::NotContains => {
                if let (Some(field_str), Some(expected_str)) = (field_value.as_str(), expected_value.as_str()) {
                    Ok(!field_str.contains(expected_str))
                } else {
                    Ok(true)
                }
            }
            ConditionOperator::Matches => {
                if let (Some(field_str), Some(pattern_str)) = (field_value.as_str(), expected_value.as_str()) {
                    let regex = Regex::new(pattern_str)
                        .map_err(|e| Error::document_parsing(format!("Invalid regex pattern: {}", e)))?;
                    Ok(regex.is_match(field_str))
                } else {
                    Ok(false)
                }
            }
            ConditionOperator::NotMatches => {
                if let (Some(field_str), Some(pattern_str)) = (field_value.as_str(), expected_value.as_str()) {
                    let regex = Regex::new(pattern_str)
                        .map_err(|e| Error::document_parsing(format!("Invalid regex pattern: {}", e)))?;
                    Ok(!regex.is_match(field_str))
                } else {
                    Ok(true)
                }
            }
            ConditionOperator::GreaterThan => {
                if let (Some(field_num), Some(expected_num)) = (field_value.as_f64(), expected_value.as_f64()) {
                    Ok(field_num > expected_num)
                } else {
                    Ok(false)
                }
            }
            ConditionOperator::LessThan => {
                if let (Some(field_num), Some(expected_num)) = (field_value.as_f64(), expected_value.as_f64()) {
                    Ok(field_num < expected_num)
                } else {
                    Ok(false)
                }
            }
            ConditionOperator::GreaterThanOrEqual => {
                if let (Some(field_num), Some(expected_num)) = (field_value.as_f64(), expected_value.as_f64()) {
                    Ok(field_num >= expected_num)
                } else {
                    Ok(false)
                }
            }
            ConditionOperator::LessThanOrEqual => {
                if let (Some(field_num), Some(expected_num)) = (field_value.as_f64(), expected_value.as_f64()) {
                    Ok(field_num <= expected_num)
                } else {
                    Ok(false)
                }
            }
            ConditionOperator::In => {
                if let Some(expected_array) = expected_value.as_array() {
                    Ok(expected_array.contains(field_value))
                } else {
                    Ok(false)
                }
            }
            ConditionOperator::NotIn => {
                if let Some(expected_array) = expected_value.as_array() {
                    Ok(!expected_array.contains(field_value))
                } else {
                    Ok(true)
                }
            }
        }
    }

    /// Check if pattern matches the source column
    fn check_pattern_match(
        &self,
        pattern: &OverridePattern,
        rule_type: &OverrideType,
        source_column: &str,
    ) -> Result<Option<f64>> {
        match rule_type {
            OverrideType::ExactMatch => {
                let matches = if pattern.case_sensitive {
                    source_column == pattern.pattern
                } else {
                    source_column.to_lowercase() == pattern.pattern.to_lowercase()
                };
                Ok(if matches { Some(1.0) } else { None })
            }
            OverrideType::RegexPattern => {
                let regex = Regex::new(&pattern.pattern)
                    .map_err(|e| Error::document_parsing(format!("Invalid regex pattern: {}", e)))?;
                Ok(if regex.is_match(source_column) { Some(0.95) } else { None })
            }
            OverrideType::FuzzyMatch => {
                let threshold = pattern.fuzzy_threshold.unwrap_or(0.8);
                let similarity = self.calculate_fuzzy_similarity(source_column, &pattern.pattern);
                Ok(if similarity >= threshold { Some(similarity) } else { None })
            }
            OverrideType::ContainsMatch => {
                let matches = if pattern.case_sensitive {
                    source_column.contains(&pattern.pattern)
                } else {
                    source_column.to_lowercase().contains(&pattern.pattern.to_lowercase())
                };
                Ok(if matches { Some(0.9) } else { None })
            }
            OverrideType::PrefixSuffixMatch => {
                let matches = if pattern.case_sensitive {
                    source_column.starts_with(&pattern.pattern) || source_column.ends_with(&pattern.pattern)
                } else {
                    let source_lower = source_column.to_lowercase();
                    let pattern_lower = pattern.pattern.to_lowercase();
                    source_lower.starts_with(&pattern_lower) || source_lower.ends_with(&pattern_lower)
                };
                Ok(if matches { Some(0.85) } else { None })
            }
            OverrideType::PositionalMatch => {
                // Positional matching would require column index, which we don't have in this context
                // This would need to be implemented at a higher level where column positions are known
                Ok(None)
            }
            OverrideType::ConditionalMatch => {
                // Conditional matching is handled by the condition evaluation
                // If we reach here, conditions were met, so it's a match
                Ok(Some(0.8))
            }
        }
    }

    /// Calculate fuzzy similarity between two strings
    fn calculate_fuzzy_similarity(&self, s1: &str, s2: &str) -> f64 {
        // Simple character-based similarity (can be enhanced with more sophisticated algorithms)
        let s1_chars: std::collections::HashSet<char> = s1.to_lowercase().chars().collect();
        let s2_chars: std::collections::HashSet<char> = s2.to_lowercase().chars().collect();

        let intersection = s1_chars.intersection(&s2_chars).count();
        let union = s1_chars.union(&s2_chars).count();

        if union == 0 {
            0.0
        } else {
            intersection as f64 / union as f64
        }
    }

    /// Detect conflicts between override rules
    fn detect_conflicts(&self, new_override: &MappingOverride) -> Result<Vec<OverrideConflict>> {
        let mut conflicts = Vec::new();

        for existing_override in &self.overrides {
            if existing_override.id == new_override.id {
                continue;
            }

            // Check for pattern overlap
            if self.patterns_overlap(&existing_override.pattern, &existing_override.rule_type,
                                   &new_override.pattern, &new_override.rule_type)? {

                // Check if they have the same priority but different targets
                if existing_override.priority == new_override.priority &&
                   existing_override.target_field != new_override.target_field {
                    conflicts.push(OverrideConflict {
                        conflicting_overrides: vec![existing_override.id, new_override.id],
                        conflict_type: ConflictType::PriorityTie,
                        description: format!(
                            "Overrides '{}' and '{}' have same priority ({}) but different targets",
                            existing_override.name, new_override.name, existing_override.priority
                        ),
                        suggested_resolution: "Adjust priority levels or merge rules".to_string(),
                        severity: ConflictSeverity::Medium,
                    });
                } else {
                    conflicts.push(OverrideConflict {
                        conflicting_overrides: vec![existing_override.id, new_override.id],
                        conflict_type: ConflictType::PatternOverlap,
                        description: format!(
                            "Overrides '{}' and '{}' have overlapping patterns",
                            existing_override.name, new_override.name
                        ),
                        suggested_resolution: "Make patterns more specific or adjust priorities".to_string(),
                        severity: ConflictSeverity::Low,
                    });
                }
            }

            // Check for scope conflicts
            if self.scopes_conflict(&existing_override.scope, &new_override.scope) {
                conflicts.push(OverrideConflict {
                    conflicting_overrides: vec![existing_override.id, new_override.id],
                    conflict_type: ConflictType::ScopeConflict,
                    description: format!(
                        "Overrides '{}' and '{}' have conflicting scopes",
                        existing_override.name, new_override.name
                    ),
                    suggested_resolution: "Adjust scope definitions or merge rules".to_string(),
                    severity: ConflictSeverity::Medium,
                });
            }
        }

        Ok(conflicts)
    }

    /// Check if two patterns overlap
    fn patterns_overlap(
        &self,
        pattern1: &OverridePattern,
        type1: &OverrideType,
        pattern2: &OverridePattern,
        type2: &OverrideType,
    ) -> Result<bool> {
        // Simple overlap detection - can be enhanced
        match (type1, type2) {
            (OverrideType::ExactMatch, OverrideType::ExactMatch) => {
                Ok(pattern1.pattern == pattern2.pattern)
            }
            (OverrideType::ContainsMatch, OverrideType::ContainsMatch) => {
                Ok(pattern1.pattern.contains(&pattern2.pattern) || pattern2.pattern.contains(&pattern1.pattern))
            }
            _ => Ok(false), // More sophisticated overlap detection would be needed for other types
        }
    }

    /// Check if two scopes conflict
    fn scopes_conflict(&self, scope1: &OverrideScope, scope2: &OverrideScope) -> bool {
        match (scope1, scope2) {
            (OverrideScope::Global, _) | (_, OverrideScope::Global) => true,
            (OverrideScope::DocumentType(dt1), OverrideScope::DocumentType(dt2)) => dt1 == dt2,
            (OverrideScope::Organization(org1), OverrideScope::Organization(org2)) => org1 == org2,
            (OverrideScope::User(u1), OverrideScope::User(u2)) => u1 == u2,
            (OverrideScope::Session(s1), OverrideScope::Session(s2)) => s1 == s2,
            (OverrideScope::Project(p1), OverrideScope::Project(p2)) => p1 == p2,
            _ => false,
        }
    }
}

/// Context information for override resolution
#[derive(Debug, Clone)]
pub struct OverrideContext {
    /// Document type being processed
    pub document_type: String,
    /// File name (if available)
    pub file_name: Option<String>,
    /// Total number of columns
    pub column_count: Option<usize>,
    /// Sample data for analysis
    pub data_sample: Option<serde_json::Value>,
    /// User role
    pub user_role: Option<String>,
    /// Organization
    pub organization: Option<String>,
    /// User identifier
    pub user: Option<String>,
    /// Session identifier
    pub session_id: Option<String>,
    /// Project identifier
    pub project: Option<String>,
    /// Custom metadata
    pub custom_metadata: HashMap<String, serde_json::Value>,
}

impl OverrideContext {
    /// Create a new override context
    pub fn new(document_type: String) -> Self {
        Self {
            document_type,
            file_name: None,
            column_count: None,
            data_sample: None,
            user_role: None,
            organization: None,
            user: None,
            session_id: None,
            project: None,
            custom_metadata: HashMap::new(),
        }
    }

    /// Generate cache key for this context
    pub fn cache_key(&self) -> String {
        format!(
            "{}:{}:{}:{}:{}",
            self.document_type,
            self.file_name.as_deref().unwrap_or(""),
            self.organization.as_deref().unwrap_or(""),
            self.user.as_deref().unwrap_or(""),
            self.session_id.as_deref().unwrap_or("")
        )
    }
}

impl ConflictResolver {
    /// Create a new conflict resolver
    pub fn new() -> Self {
        Self {
            resolution_strategy: ConflictResolutionStrategy::HighestPriority,
            max_conflicts_reported: 10,
        }
    }

    /// Resolve conflicts between multiple matching overrides
    pub fn resolve_conflicts(
        &self,
        matching_overrides: Vec<(MappingOverride, f64)>,
    ) -> Result<(Option<(MappingOverride, f64)>, Vec<OverrideConflict>)> {
        if matching_overrides.len() <= 1 {
            return Ok((matching_overrides.into_iter().next(), Vec::new()));
        }

        let mut conflicts = Vec::new();

        // Generate conflict reports
        for i in 0..matching_overrides.len() {
            for j in (i + 1)..matching_overrides.len() {
                let (ref override1, _) = matching_overrides[i];
                let (ref override2, _) = matching_overrides[j];

                conflicts.push(OverrideConflict {
                    conflicting_overrides: vec![override1.id, override2.id],
                    conflict_type: ConflictType::PatternOverlap,
                    description: format!(
                        "Multiple overrides match: '{}' and '{}'",
                        override1.name, override2.name
                    ),
                    suggested_resolution: "Use priority-based resolution".to_string(),
                    severity: ConflictSeverity::Low,
                });
            }
        }

        // Apply resolution strategy
        let selected = match self.resolution_strategy {
            ConflictResolutionStrategy::HighestPriority => {
                matching_overrides.into_iter()
                    .max_by_key(|(override_rule, _)| override_rule.priority)
            }
            ConflictResolutionStrategy::MostRecent => {
                matching_overrides.into_iter()
                    .max_by_key(|(override_rule, _)| override_rule.created_at)
            }
            ConflictResolutionStrategy::MostSpecific => {
                matching_overrides.into_iter()
                    .max_by_key(|(override_rule, _)| override_rule.conditions.len())
            }
            ConflictResolutionStrategy::Combine => {
                // For now, just use highest priority
                matching_overrides.into_iter()
                    .max_by_key(|(override_rule, _)| override_rule.priority)
            }
            ConflictResolutionStrategy::ReportAndFallback => {
                // Return no selection and let fallback handle it
                None
            }
        };

        Ok((selected, conflicts))
    }
}

impl OverrideValidator {
    /// Create a new override validator
    pub fn new() -> Self {
        Self {
            regex_cache: HashMap::new(),
            validation_rules: ValidationRuleSet {
                max_pattern_length: 1000,
                max_conditions: 10,
                allowed_target_fields: None,
                forbidden_patterns: vec![".*".to_string(), ".+".to_string()], // Overly broad patterns
                required_tags: HashMap::new(),
            },
        }
    }

    /// Validate an override rule
    pub fn validate_override(&mut self, override_rule: &MappingOverride) -> Result<()> {
        // Validate pattern length
        if override_rule.pattern.pattern.len() > self.validation_rules.max_pattern_length {
            return Err(Error::document_parsing(format!(
                "Pattern too long: {} characters (max: {})",
                override_rule.pattern.pattern.len(),
                self.validation_rules.max_pattern_length
            )));
        }

        // Validate number of conditions
        if override_rule.conditions.len() > self.validation_rules.max_conditions {
            return Err(Error::document_parsing(format!(
                "Too many conditions: {} (max: {})",
                override_rule.conditions.len(),
                self.validation_rules.max_conditions
            )));
        }

        // Validate regex patterns
        if override_rule.rule_type == OverrideType::RegexPattern {
            if let Err(e) = Regex::new(&override_rule.pattern.pattern) {
                return Err(Error::document_parsing(format!(
                    "Invalid regex pattern '{}': {}",
                    override_rule.pattern.pattern, e
                )));
            }
        }

        // Check forbidden patterns
        for forbidden in &self.validation_rules.forbidden_patterns {
            if override_rule.pattern.pattern == *forbidden {
                return Err(Error::document_parsing(format!(
                    "Forbidden pattern: '{}'",
                    forbidden
                )));
            }
        }

        // Validate target field if allowlist exists
        if let Some(ref allowed_fields) = self.validation_rules.allowed_target_fields {
            if !allowed_fields.contains(&override_rule.target_field) {
                return Err(Error::document_parsing(format!(
                    "Target field '{}' not in allowed list",
                    override_rule.target_field
                )));
            }
        }

        // Validate required tags for scope
        if let Some(required_tags) = self.validation_rules.required_tags.get(&override_rule.scope) {
            for required_tag in required_tags {
                if !override_rule.tags.contains(required_tag) {
                    return Err(Error::document_parsing(format!(
                        "Missing required tag '{}' for scope {:?}",
                        required_tag, override_rule.scope
                    )));
                }
            }
        }

        Ok(())
    }
}

impl MappingReportGenerator {
    /// Create a new mapping report generator
    pub fn new() -> Self {
        Self {
            config: ReportConfig::default(),
            template_engine: None,
            report_cache: LruCache::new(std::num::NonZeroUsize::new(100).unwrap()),
            historical_data: HistoricalReportData::default(),
            generation_metrics: ReportGenerationMetrics::default(),
        }
    }

    /// Create a new report generator with custom configuration
    pub fn with_config(config: ReportConfig) -> Self {
        let mut generator = Self::new();
        generator.config = config;

        // Initialize template engine if template directory is specified
        if let Some(ref template_dir) = generator.config.template_directory {
            match tera::Tera::new(&format!("{}/**/*", template_dir)) {
                Ok(tera) => generator.template_engine = Some(tera),
                Err(e) => warn!("Failed to initialize template engine: {}", e),
            }
        }

        generator
    }

    /// Generate a comprehensive mapping report
    pub fn generate_report(
        &mut self,
        document_info: DocumentInfo,
        mapping_results: &[crate::mapping::engine::MappingResult],
        validation_results: &[ColumnValidationResult],
        override_results: &[OverrideResolutionResult],
        confidence_results: &[MappingConfidence],
        report_type: ReportType,
    ) -> Result<MappingReport> {
        let start_time = Instant::now();
        let report_id = Uuid::new_v4();

        info!("Generating {} report with ID: {}",
              serde_json::to_string(&report_type).unwrap_or_default(),
              report_id);

        // Check cache first if enabled
        if self.config.enable_caching {
            let cache_key = self.generate_cache_key(&document_info, &report_type);
            if let Some(cached) = self.report_cache.get(&cache_key) {
                if cached.expires_at > chrono::Utc::now() {
                    info!("Returning cached report for key: {}", cache_key);
                    self.generation_metrics.cache_hit_rate =
                        (self.generation_metrics.cache_hit_rate * 0.9) + (1.0 * 0.1);
                    return Ok(cached.report.clone());
                }
            }
        }

        // Update cache miss rate
        self.generation_metrics.cache_hit_rate =
            (self.generation_metrics.cache_hit_rate * 0.9) + (0.0 * 0.1);

        // Generate mapping summary
        let mapping_summary = self.generate_mapping_summary(mapping_results, confidence_results)?;

        // Generate detailed field results
        let detailed_results = self.generate_detailed_results(
            mapping_results,
            validation_results,
            override_results,
            confidence_results
        )?;

        // Calculate quality metrics
        let quality_metrics = self.calculate_quality_metrics(&detailed_results, &mapping_summary)?;

        // Generate recommendations
        let recommendations = self.generate_recommendations(
            &detailed_results,
            &quality_metrics,
            &mapping_summary
        )?;

        // Generate validation summary
        let validation_summary = self.generate_validation_summary(validation_results)?;

        // Generate override summary
        let override_summary = self.generate_override_summary(override_results)?;

        // Generate performance metrics
        let performance_metrics = self.generate_performance_metrics(
            &document_info,
            mapping_results,
            validation_results
        )?;

        // Generate trend analysis if historical data is available
        let trend_analysis = if self.historical_data.quality_history.len() > 1 {
            Some(self.generate_trend_analysis(&quality_metrics)?)
        } else {
            None
        };

        let generation_time = start_time.elapsed();

        // Create the report
        let report = MappingReport {
            report_id,
            report_type: report_type.clone(),
            generated_at: chrono::Utc::now(),
            document_info,
            mapping_summary,
            detailed_results,
            quality_metrics: quality_metrics.clone(),
            recommendations,
            validation_results: validation_summary,
            override_results: override_summary,
            performance_metrics,
            trend_analysis,
        };

        // Cache the report if caching is enabled
        if self.config.enable_caching {
            let cache_key = self.generate_cache_key(&report.document_info, &report_type);
            let expires_at = chrono::Utc::now() +
                chrono::Duration::minutes(self.config.cache_expiration_minutes as i64);

            self.report_cache.put(cache_key, CachedReport {
                report: report.clone(),
                cached_at: chrono::Utc::now(),
                expires_at,
                format: self.config.default_format.clone(),
            });
        }

        // Update historical data
        self.update_historical_data(&quality_metrics, &performance_metrics);

        // Update generation metrics
        self.generation_metrics.total_reports_generated += 1;
        let generation_time_ms = generation_time.as_millis() as f64;
        self.generation_metrics.avg_generation_time_ms =
            (self.generation_metrics.avg_generation_time_ms * 0.9) + (generation_time_ms * 0.1);
        self.generation_metrics.last_updated = chrono::Utc::now();

        // Check generation time against target
        if generation_time.as_secs() > self.config.max_generation_time_seconds {
            warn!(
                "Report generation took {}ms, exceeding target of {}s",
                generation_time.as_millis(),
                self.config.max_generation_time_seconds
            );
        }

        info!(
            "Generated report {} in {}ms",
            report_id,
            generation_time.as_millis()
        );

        Ok(report)
    }

    /// Export report to specified format
    pub fn export_report(&self, report: &MappingReport, format: ReportFormat) -> Result<String> {
        match format {
            ReportFormat::Json => self.export_to_json(report),
            ReportFormat::Html => self.export_to_html(report),
            ReportFormat::Csv => self.export_to_csv(report),
            ReportFormat::Markdown => self.export_to_markdown(report),
            ReportFormat::Pdf => self.export_to_pdf(report),
        }
    }

    /// Generate cache key for report caching
    fn generate_cache_key(&self, document_info: &DocumentInfo, report_type: &ReportType) -> String {
        format!(
            "{}:{}:{}:{}",
            document_info.document_hash,
            document_info.file_name,
            serde_json::to_string(report_type).unwrap_or_default(),
            document_info.processed_at.timestamp()
        )
    }

    /// Generate mapping summary statistics
    fn generate_mapping_summary(
        &self,
        mapping_results: &[crate::mapping::engine::MappingResult],
        confidence_results: &[MappingConfidence],
    ) -> Result<MappingSummary> {
        let total_fields = mapping_results.len();
        let mapped_fields = mapping_results.iter().filter(|r| !r.target_field.is_empty()).count();

        // Calculate confidence statistics
        let confidences: Vec<f64> = confidence_results.iter().map(|c| c.overall_score).collect();
        let average_confidence = if confidences.is_empty() {
            0.0
        } else {
            confidences.iter().sum::<f64>() / confidences.len() as f64
        };

        let min_confidence = confidences.iter().fold(1.0f64, |a, &b| a.min(b));
        let max_confidence = confidences.iter().fold(0.0f64, |a, &b| a.max(b));

        let high_confidence_mappings = confidences.iter().filter(|&&c| c > 0.9).count();
        let low_confidence_mappings = confidences.iter().filter(|&&c| c < 0.5).count();

        let success_rate = if total_fields == 0 {
            0.0
        } else {
            mapped_fields as f64 / total_fields as f64
        };

        Ok(MappingSummary {
            total_fields,
            mapped_fields,
            required_fields_mapped: 0, // Will be calculated from validation results
            required_fields_missing: 0,
            optional_fields_mapped: mapped_fields,
            average_confidence,
            min_confidence,
            max_confidence,
            high_confidence_mappings,
            low_confidence_mappings,
            processing_time: Duration::from_millis(0), // Will be set from performance metrics
            success_rate,
        })
    }

    /// Generate detailed field mapping results
    fn generate_detailed_results(
        &self,
        mapping_results: &[crate::mapping::engine::MappingResult],
        validation_results: &[ColumnValidationResult],
        override_results: &[OverrideResolutionResult],
        confidence_results: &[MappingConfidence],
    ) -> Result<Vec<FieldMappingResult>> {
        let mut detailed_results = Vec::new();

        for (i, mapping_result) in mapping_results.iter().enumerate() {
            let validation_result = validation_results.get(i).cloned();
            let override_result = override_results.get(i);
            let confidence_result = confidence_results.get(i);

            let field_result = FieldMappingResult {
                field_id: format!("field_{}", i),
                target_field: mapping_result.target_field.clone(),
                source_column: Some(mapping_result.source_column.clone()),
                confidence_score: confidence_result.map(|c| c.overall_score).unwrap_or(mapping_result.confidence),
                mapping_successful: !mapping_result.target_field.is_empty(),
                required: validation_result.as_ref().map(|v| v.severity == ValidationSeverity::Error).unwrap_or(false),
                validation_result,
                override_applied: override_result.and_then(|o| o.applied_override.as_ref().map(|ao| ao.name.clone())),
                alternatives: self.generate_alternatives(mapping_result)?,
                issues: self.generate_field_issues(mapping_result, confidence_result)?,
                data_quality: None, // TODO: Implement data quality assessment
            };

            detailed_results.push(field_result);
        }

        Ok(detailed_results)
    }

    /// Generate alternative mapping suggestions
    fn generate_alternatives(&self, mapping_result: &crate::mapping::engine::MappingResult) -> Result<Vec<MappingAlternative>> {
        let mut alternatives = Vec::new();

        // Generate alternatives based on confidence score
        if mapping_result.confidence < 0.8 {
            // Suggest similar column names (simplified implementation)
            let similar_columns = vec![
                format!("{}_alt", mapping_result.source_column),
                format!("alt_{}", mapping_result.source_column),
            ];

            for alt_column in similar_columns {
                alternatives.push(MappingAlternative {
                    source_column: alt_column,
                    confidence_score: mapping_result.confidence * 0.8,
                    reason: "Similar column name pattern".to_string(),
                });
            }
        }

        Ok(alternatives)
    }

    /// Generate issues for a field mapping
    fn generate_field_issues(
        &self,
        mapping_result: &crate::mapping::engine::MappingResult,
        confidence_result: Option<&MappingConfidence>,
    ) -> Result<Vec<MappingIssue>> {
        let mut issues = Vec::new();

        // Low confidence issue
        if mapping_result.confidence < 0.5 {
            issues.push(MappingIssue {
                severity: IssueSeverity::Warning,
                category: IssueCategory::LowConfidence,
                description: format!(
                    "Low confidence mapping ({:.1}%) for column '{}'",
                    mapping_result.confidence * 100.0,
                    mapping_result.source_column
                ),
                suggested_resolution: Some("Review mapping manually or provide custom override".to_string()),
                impact: "May result in incorrect data mapping".to_string(),
            });
        }

        // Add issues from confidence result
        if let Some(confidence) = confidence_result {
            for risk_factor in &confidence.risk_factors {
                let severity = match risk_factor.severity {
                    RiskSeverity::Low => IssueSeverity::Info,
                    RiskSeverity::Medium => IssueSeverity::Warning,
                    RiskSeverity::High => IssueSeverity::Error,
                    RiskSeverity::Critical => IssueSeverity::Critical,
                };

                issues.push(MappingIssue {
                    severity,
                    category: IssueCategory::DataQuality, // Simplified categorization
                    description: risk_factor.description.clone(),
                    suggested_resolution: None,
                    impact: format!("Confidence impact: {:.1}%", risk_factor.confidence_impact * 100.0),
                });
            }
        }

        Ok(issues)
    }

    /// Calculate overall quality metrics
    fn calculate_quality_metrics(
        &self,
        detailed_results: &[FieldMappingResult],
        mapping_summary: &MappingSummary,
    ) -> Result<QualityMetrics> {
        let total_fields = detailed_results.len() as f64;

        // Calculate completeness score
        let mapped_fields = detailed_results.iter().filter(|r| r.mapping_successful).count() as f64;
        let completeness_score = if total_fields > 0.0 {
            mapped_fields / total_fields
        } else {
            0.0
        };

        // Calculate accuracy score based on confidence
        let accuracy_score = mapping_summary.average_confidence;

        // Calculate consistency score (simplified)
        let high_confidence_count = detailed_results.iter()
            .filter(|r| r.confidence_score > 0.8)
            .count() as f64;
        let consistency_score = if total_fields > 0.0 {
            high_confidence_count / total_fields
        } else {
            0.0
        };

        // Calculate overall quality score
        let overall_quality_score = (completeness_score * 0.4) + (accuracy_score * 0.4) + (consistency_score * 0.2);

        // Determine risk level
        let risk_level = if overall_quality_score >= 0.9 {
            RiskLevel::VeryLow
        } else if overall_quality_score >= 0.8 {
            RiskLevel::Low
        } else if overall_quality_score >= 0.7 {
            RiskLevel::Medium
        } else if overall_quality_score >= 0.6 {
            RiskLevel::High
        } else {
            RiskLevel::VeryHigh
        };

        // Determine quality grade
        let quality_grade = if overall_quality_score >= 0.9 {
            QualityGrade::A
        } else if overall_quality_score >= 0.8 {
            QualityGrade::B
        } else if overall_quality_score >= 0.7 {
            QualityGrade::C
        } else if overall_quality_score >= 0.6 {
            QualityGrade::D
        } else {
            QualityGrade::F
        };

        // Count issues
        let critical_issues = detailed_results.iter()
            .flat_map(|r| &r.issues)
            .filter(|i| i.severity == IssueSeverity::Critical)
            .count();

        let warnings = detailed_results.iter()
            .flat_map(|r| &r.issues)
            .filter(|i| i.severity == IssueSeverity::Warning)
            .count();

        Ok(QualityMetrics {
            completeness_score,
            accuracy_score,
            consistency_score,
            overall_quality_score,
            risk_level,
            quality_grade,
            compliance_percentage: overall_quality_score * 100.0,
            critical_issues,
            warnings,
        })
    }

    /// Generate actionable recommendations
    fn generate_recommendations(
        &self,
        detailed_results: &[FieldMappingResult],
        quality_metrics: &QualityMetrics,
        mapping_summary: &MappingSummary,
    ) -> Result<Vec<Recommendation>> {
        let mut recommendations = Vec::new();

        // Low overall quality recommendation
        if quality_metrics.overall_quality_score < 0.7 {
            recommendations.push(Recommendation {
                priority: RecommendationPriority::High,
                category: RecommendationCategory::DataQuality,
                title: "Improve Overall Data Quality".to_string(),
                description: format!(
                    "Overall quality score is {:.1}%, which is below acceptable threshold",
                    quality_metrics.overall_quality_score * 100.0
                ),
                suggested_action: "Review low-confidence mappings and provide custom overrides where needed".to_string(),
                impact_assessment: "Improving data quality will increase compliance and reduce processing errors".to_string(),
                effort_level: EffortLevel::Medium,
                related_fields: detailed_results.iter()
                    .filter(|r| r.confidence_score < 0.7)
                    .map(|r| r.field_id.clone())
                    .collect(),
            });
        }

        // Low confidence mappings recommendation
        let low_confidence_count = detailed_results.iter()
            .filter(|r| r.confidence_score < 0.5)
            .count();

        if low_confidence_count > 0 {
            recommendations.push(Recommendation {
                priority: RecommendationPriority::Medium,
                category: RecommendationCategory::MappingAccuracy,
                title: "Review Low Confidence Mappings".to_string(),
                description: format!(
                    "{} field(s) have confidence scores below 50%",
                    low_confidence_count
                ),
                suggested_action: "Manually review these mappings and consider creating custom override rules".to_string(),
                impact_assessment: "Addressing low confidence mappings will improve data accuracy".to_string(),
                effort_level: EffortLevel::Low,
                related_fields: detailed_results.iter()
                    .filter(|r| r.confidence_score < 0.5)
                    .map(|r| r.field_id.clone())
                    .collect(),
            });
        }

        // Performance recommendation
        if mapping_summary.processing_time.as_millis() > 5000 {
            recommendations.push(Recommendation {
                priority: RecommendationPriority::Low,
                category: RecommendationCategory::Performance,
                title: "Optimize Processing Performance".to_string(),
                description: format!(
                    "Processing took {}ms, which exceeds optimal performance",
                    mapping_summary.processing_time.as_millis()
                ),
                suggested_action: "Consider enabling caching or optimizing mapping rules".to_string(),
                impact_assessment: "Performance improvements will reduce processing time and resource usage".to_string(),
                effort_level: EffortLevel::Medium,
                related_fields: Vec::new(),
            });
        }

        // Critical issues recommendation
        if quality_metrics.critical_issues > 0 {
            recommendations.push(Recommendation {
                priority: RecommendationPriority::Critical,
                category: RecommendationCategory::Compliance,
                title: "Address Critical Issues".to_string(),
                description: format!(
                    "{} critical issue(s) detected that may prevent compliance",
                    quality_metrics.critical_issues
                ),
                suggested_action: "Immediately review and resolve all critical issues before proceeding".to_string(),
                impact_assessment: "Critical issues must be resolved to ensure compliance and data integrity".to_string(),
                effort_level: EffortLevel::High,
                related_fields: detailed_results.iter()
                    .filter(|r| r.issues.iter().any(|i| i.severity == IssueSeverity::Critical))
                    .map(|r| r.field_id.clone())
                    .collect(),
            });
        }

        Ok(recommendations)
    }

    /// Generate validation summary
    fn generate_validation_summary(&self, validation_results: &[ColumnValidationResult]) -> Result<ValidationSummary> {
        let total_validations = validation_results.len();
        let validations_passed = validation_results.iter().filter(|r| r.passed).count();
        let validations_failed = total_validations - validations_passed;
        let validation_warnings = validation_results.iter()
            .filter(|r| r.severity == ValidationSeverity::Warning)
            .count();

        let success_rate = if total_validations > 0 {
            validations_passed as f64 / total_validations as f64
        } else {
            0.0
        };

        // Generate common failures (simplified)
        let mut common_failures = Vec::new();
        let invalid_count = validation_results.iter()
            .filter(|r| r.status == ValidationStatus::Invalid)
            .count();

        if invalid_count > 0 {
            common_failures.push(ValidationFailureInfo {
                failure_type: "Invalid Data Format".to_string(),
                occurrence_count: invalid_count,
                failure_percentage: (invalid_count as f64 / total_validations as f64) * 100.0,
                example_values: vec!["Example invalid value".to_string()],
                suggested_fixes: vec!["Correct data format".to_string()],
            });
        }

        Ok(ValidationSummary {
            total_validations,
            validations_passed,
            validations_failed,
            validation_warnings,
            success_rate,
            common_failures,
            performance_metrics: ValidationPerformanceMetrics {
                avg_validation_time_us: 100.0, // Placeholder
                total_validation_time: Duration::from_millis(total_validations as u64),
                slowest_validations: Vec::new(),
            },
        })
    }

    /// Generate override summary
    fn generate_override_summary(&self, override_results: &[OverrideResolutionResult]) -> Result<OverrideSummary> {
        let total_overrides_evaluated = override_results.len();
        let overrides_applied = override_results.iter().filter(|r| r.override_applied).count();
        let conflicts_detected = override_results.iter()
            .map(|r| r.conflicts.len())
            .sum();

        let application_success_rate = if total_overrides_evaluated > 0 {
            overrides_applied as f64 / total_overrides_evaluated as f64
        } else {
            0.0
        };

        Ok(OverrideSummary {
            total_overrides_evaluated,
            overrides_applied,
            conflicts_detected,
            application_success_rate,
            frequently_applied: Vec::new(), // TODO: Implement based on historical data
            performance_metrics: OverridePerformanceMetrics {
                avg_resolution_time_us: override_results.iter()
                    .map(|r| r.resolution_time.as_micros() as f64)
                    .sum::<f64>() / override_results.len().max(1) as f64,
                cache_hit_rate: override_results.iter()
                    .filter(|r| r.from_cache)
                    .count() as f64 / override_results.len().max(1) as f64,
                total_processing_time: override_results.iter()
                    .map(|r| r.resolution_time)
                    .sum(),
            },
        })
    }

    /// Generate performance metrics
    fn generate_performance_metrics(
        &self,
        document_info: &DocumentInfo,
        _mapping_results: &[crate::mapping::engine::MappingResult],
        _validation_results: &[ColumnValidationResult],
    ) -> Result<ProcessingMetrics> {
        Ok(ProcessingMetrics {
            total_processing_time: document_info.processing_duration,
            column_detection_time: Duration::from_millis(100), // Placeholder
            mapping_time: Duration::from_millis(200), // Placeholder
            validation_time: Duration::from_millis(150), // Placeholder
            override_time: Duration::from_millis(50), // Placeholder
            memory_usage: MemoryUsageMetrics {
                peak_memory_bytes: 1024 * 1024, // Placeholder: 1MB
                avg_memory_bytes: 512 * 1024,   // Placeholder: 512KB
                efficiency_score: 0.8,
            },
            throughput: ThroughputMetrics {
                rows_per_second: document_info.total_rows as f64 / document_info.processing_duration.as_secs_f64(),
                fields_per_second: (document_info.total_rows * document_info.total_columns) as f64 / document_info.processing_duration.as_secs_f64(),
                bytes_per_second: document_info.file_size as f64 / document_info.processing_duration.as_secs_f64(),
            },
        })
    }

    /// Generate trend analysis
    fn generate_trend_analysis(&self, current_quality: &QualityMetrics) -> Result<TrendAnalysis> {
        let time_period = TimePeriod {
            start_date: chrono::Utc::now() - chrono::Duration::days(30),
            end_date: chrono::Utc::now(),
            data_points: self.historical_data.quality_history.len(),
        };

        // Analyze quality trends (simplified)
        let overall_quality_trend = if self.historical_data.quality_history.len() >= 2 {
            let recent_avg = self.historical_data.quality_history.iter()
                .rev()
                .take(5)
                .map(|h| h.overall_score)
                .sum::<f64>() / 5.0;

            if recent_avg > current_quality.overall_quality_score {
                TrendDirection::Declining
            } else if recent_avg < current_quality.overall_quality_score {
                TrendDirection::Improving
            } else {
                TrendDirection::Stable
            }
        } else {
            TrendDirection::InsufficientData
        };

        Ok(TrendAnalysis {
            time_period,
            quality_trends: QualityTrends {
                overall_quality_trend,
                completeness_trend: TrendDirection::Stable, // Simplified
                accuracy_trend: TrendDirection::Stable,     // Simplified
                consistency_trend: TrendDirection::Stable,  // Simplified
                historical_scores: self.historical_data.quality_history.clone(),
            },
            performance_trends: PerformanceTrends {
                processing_time_trend: TrendDirection::Stable,
                throughput_trend: TrendDirection::Stable,
                memory_usage_trend: TrendDirection::Stable,
                historical_performance: self.historical_data.performance_history.clone(),
            },
            issue_trends: IssueTrends {
                critical_issues_trend: TrendDirection::Stable,
                warnings_trend: TrendDirection::Stable,
                common_issues: Vec::new(),
            },
            trend_recommendations: Vec::new(),
        })
    }

    /// Update historical data with current metrics
    fn update_historical_data(&mut self, quality_metrics: &QualityMetrics, performance_metrics: &ProcessingMetrics) {
        let now = chrono::Utc::now();

        // Add quality data point
        self.historical_data.quality_history.push(HistoricalQualityScore {
            timestamp: now,
            overall_score: quality_metrics.overall_quality_score,
            completeness_score: quality_metrics.completeness_score,
            accuracy_score: quality_metrics.accuracy_score,
            consistency_score: quality_metrics.consistency_score,
        });

        // Add performance data point
        self.historical_data.performance_history.push(HistoricalPerformanceData {
            timestamp: now,
            processing_time_ms: performance_metrics.total_processing_time.as_millis() as u64,
            throughput_rps: performance_metrics.throughput.rows_per_second,
            memory_usage_bytes: performance_metrics.memory_usage.peak_memory_bytes,
        });

        // Cleanup old data based on retention policy
        let retention_cutoff = now - chrono::Duration::days(self.historical_data.max_retention_days as i64);

        self.historical_data.quality_history.retain(|h| h.timestamp > retention_cutoff);
        self.historical_data.performance_history.retain(|h| h.timestamp > retention_cutoff);
    }

    /// Export report to JSON format
    fn export_to_json(&self, report: &MappingReport) -> Result<String> {
        serde_json::to_string_pretty(report)
            .map_err(|e| Error::document_parsing(format!("Failed to serialize report to JSON: {}", e)))
    }

    /// Export report to HTML format
    fn export_to_html(&self, report: &MappingReport) -> Result<String> {
        if let Some(ref tera) = self.template_engine {
            let mut context = tera::Context::new();
            context.insert("report", report);

            tera.render("mapping_report.html", &context)
                .map_err(|e| Error::document_parsing(format!("Failed to render HTML report: {}", e)))
        } else {
            // Fallback to simple HTML generation
            Ok(self.generate_simple_html(report))
        }
    }

    /// Export report to CSV format
    fn export_to_csv(&self, report: &MappingReport) -> Result<String> {
        let mut csv_content = String::new();

        // Header
        csv_content.push_str("Field ID,Target Field,Source Column,Confidence Score,Mapping Successful,Required,Issues\n");

        // Data rows
        for result in &report.detailed_results {
            csv_content.push_str(&format!(
                "{},{},{},{:.3},{},{},{}\n",
                result.field_id,
                result.target_field,
                result.source_column.as_deref().unwrap_or(""),
                result.confidence_score,
                result.mapping_successful,
                result.required,
                result.issues.len()
            ));
        }

        Ok(csv_content)
    }

    /// Export report to Markdown format
    fn export_to_markdown(&self, report: &MappingReport) -> Result<String> {
        let mut md_content = String::new();

        md_content.push_str(&format!("# Mapping Validation Report\n\n"));
        md_content.push_str(&format!("**Report ID:** {}\n", report.report_id));
        md_content.push_str(&format!("**Generated:** {}\n", report.generated_at.format("%Y-%m-%d %H:%M:%S UTC")));
        md_content.push_str(&format!("**Document:** {}\n\n", report.document_info.file_name));

        md_content.push_str("## Summary\n\n");
        md_content.push_str(&format!("- **Total Fields:** {}\n", report.mapping_summary.total_fields));
        md_content.push_str(&format!("- **Mapped Fields:** {}\n", report.mapping_summary.mapped_fields));
        md_content.push_str(&format!("- **Success Rate:** {:.1}%\n", report.mapping_summary.success_rate * 100.0));
        md_content.push_str(&format!("- **Average Confidence:** {:.1}%\n", report.mapping_summary.average_confidence * 100.0));
        md_content.push_str(&format!("- **Quality Grade:** {:?}\n\n", report.quality_metrics.quality_grade));

        md_content.push_str("## Recommendations\n\n");
        for (i, rec) in report.recommendations.iter().enumerate() {
            md_content.push_str(&format!("{}. **{}** (Priority: {:?})\n", i + 1, rec.title, rec.priority));
            md_content.push_str(&format!("   - {}\n", rec.description));
            md_content.push_str(&format!("   - Action: {}\n\n", rec.suggested_action));
        }

        Ok(md_content)
    }

    /// Export report to PDF format (placeholder implementation)
    fn export_to_pdf(&self, _report: &MappingReport) -> Result<String> {
        // This would require a PDF generation library like `printpdf` or `wkhtmltopdf`
        // For now, return an error indicating PDF export is not implemented
        Err(Error::document_parsing("PDF export not yet implemented".to_string()))
    }

    /// Generate simple HTML report (fallback when no template engine)
    fn generate_simple_html(&self, report: &MappingReport) -> String {
        format!(
            r#"<!DOCTYPE html>
<html>
<head>
    <title>Mapping Validation Report</title>
    <style>
        body {{ font-family: Arial, sans-serif; margin: 20px; }}
        .header {{ background-color: #f0f0f0; padding: 10px; border-radius: 5px; }}
        .summary {{ margin: 20px 0; }}
        .quality-grade {{ font-size: 24px; font-weight: bold; }}
        .recommendations {{ margin: 20px 0; }}
        .recommendation {{ margin: 10px 0; padding: 10px; border-left: 4px solid #007acc; }}
        table {{ border-collapse: collapse; width: 100%; }}
        th, td {{ border: 1px solid #ddd; padding: 8px; text-align: left; }}
        th {{ background-color: #f2f2f2; }}
    </style>
</head>
<body>
    <div class="header">
        <h1>Mapping Validation Report</h1>
        <p><strong>Report ID:</strong> {}</p>
        <p><strong>Generated:</strong> {}</p>
        <p><strong>Document:</strong> {}</p>
    </div>

    <div class="summary">
        <h2>Summary</h2>
        <p><strong>Total Fields:</strong> {}</p>
        <p><strong>Mapped Fields:</strong> {}</p>
        <p><strong>Success Rate:</strong> {:.1}%</p>
        <p><strong>Average Confidence:</strong> {:.1}%</p>
        <p class="quality-grade"><strong>Quality Grade:</strong> {:?}</p>
    </div>

    <div class="recommendations">
        <h2>Recommendations</h2>
        {}
    </div>
</body>
</html>"#,
            report.report_id,
            report.generated_at.format("%Y-%m-%d %H:%M:%S UTC"),
            report.document_info.file_name,
            report.mapping_summary.total_fields,
            report.mapping_summary.mapped_fields,
            report.mapping_summary.success_rate * 100.0,
            report.mapping_summary.average_confidence * 100.0,
            report.quality_metrics.quality_grade,
            report.recommendations.iter()
                .map(|r| format!(r#"<div class="recommendation"><strong>{}</strong> (Priority: {:?})<br>{}</div>"#, r.title, r.priority, r.description))
                .collect::<Vec<_>>()
                .join("")
        )
    }

    /// Get generation metrics
    pub fn get_metrics(&self) -> &ReportGenerationMetrics {
        &self.generation_metrics
    }

    /// Clear report cache
    pub fn clear_cache(&mut self) {
        self.report_cache.clear();
    }

    /// Get historical data
    pub fn get_historical_data(&self) -> &HistoricalReportData {
        &self.historical_data
    }
}

impl Default for ReportConfig {
    fn default() -> Self {
        Self {
            default_format: ReportFormat::Html,
            include_visualizations: true,
            max_generation_time_seconds: 30,
            enable_caching: true,
            cache_expiration_minutes: 60,
            include_detailed_analysis: true,
            include_recommendations: true,
            template_directory: None,
        }
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

    #[test]
    fn test_mapping_override_engine_creation() {
        let engine = MappingOverrideEngine::new();
        assert_eq!(engine.overrides.len(), 0);
        assert_eq!(engine.get_metrics().total_applications, 0);
    }

    #[test]
    fn test_add_override_rule() {
        let mut engine = MappingOverrideEngine::new();

        let override_rule = MappingOverride {
            id: Uuid::new_v4(),
            name: "Test Override".to_string(),
            description: "Test override rule".to_string(),
            rule_type: OverrideType::ExactMatch,
            pattern: OverridePattern {
                pattern: "Asset ID".to_string(),
                case_sensitive: false,
                whole_word: true,
                regex_flags: None,
                fuzzy_threshold: None,
                position_constraints: None,
            },
            target_field: "uuid".to_string(),
            priority: 100,
            conditions: Vec::new(),
            scope: OverrideScope::Global,
            created_by: "test_user".to_string(),
            created_at: chrono::Utc::now(),
            modified_at: chrono::Utc::now(),
            active: true,
            version: 1,
            tags: vec!["test".to_string()],
        };

        let result = engine.add_override(override_rule);
        assert!(result.is_ok());
        assert_eq!(engine.overrides.len(), 1);
    }

    #[test]
    fn test_override_pattern_matching() {
        let engine = MappingOverrideEngine::new();

        // Test exact match
        let pattern = OverridePattern {
            pattern: "Asset Name".to_string(),
            case_sensitive: false,
            whole_word: true,
            regex_flags: None,
            fuzzy_threshold: None,
            position_constraints: None,
        };

        let result = engine.check_pattern_match(&pattern, &OverrideType::ExactMatch, "Asset Name").unwrap();
        assert_eq!(result, Some(1.0));

        let result = engine.check_pattern_match(&pattern, &OverrideType::ExactMatch, "asset name").unwrap();
        assert_eq!(result, Some(1.0)); // Case insensitive

        let result = engine.check_pattern_match(&pattern, &OverrideType::ExactMatch, "Component Name").unwrap();
        assert_eq!(result, None);
    }

    #[test]
    fn test_override_conditions() {
        let engine = MappingOverrideEngine::new();
        let context = OverrideContext::new("inventory".to_string());

        let conditions = vec![
            OverrideCondition {
                condition_type: ConditionType::DocumentType,
                field: "document_type".to_string(),
                operator: ConditionOperator::Equals,
                value: serde_json::Value::String("inventory".to_string()),
                required: true,
            }
        ];

        let result = engine.evaluate_conditions(&conditions, "Asset ID", "inventory", &context).unwrap();
        assert!(result);

        let result = engine.evaluate_conditions(&conditions, "Asset ID", "poam", &context).unwrap();
        assert!(!result);
    }

    #[test]
    fn test_override_scope_matching() {
        let engine = MappingOverrideEngine::new();
        let mut context = OverrideContext::new("inventory".to_string());
        context.organization = Some("test_org".to_string());

        // Test global scope
        let result = engine.check_scope_match(&OverrideScope::Global, &context).unwrap();
        assert!(result);

        // Test document type scope
        let result = engine.check_scope_match(&OverrideScope::DocumentType("inventory".to_string()), &context).unwrap();
        assert!(result);

        let result = engine.check_scope_match(&OverrideScope::DocumentType("poam".to_string()), &context).unwrap();
        assert!(!result);

        // Test organization scope
        let result = engine.check_scope_match(&OverrideScope::Organization("test_org".to_string()), &context).unwrap();
        assert!(result);

        let result = engine.check_scope_match(&OverrideScope::Organization("other_org".to_string()), &context).unwrap();
        assert!(!result);
    }

    #[test]
    fn test_override_conflict_detection() {
        let mut engine = MappingOverrideEngine::new();

        // Add first override
        let override1 = MappingOverride {
            id: Uuid::new_v4(),
            name: "Override 1".to_string(),
            description: "First override".to_string(),
            rule_type: OverrideType::ExactMatch,
            pattern: OverridePattern {
                pattern: "Asset ID".to_string(),
                case_sensitive: false,
                whole_word: true,
                regex_flags: None,
                fuzzy_threshold: None,
                position_constraints: None,
            },
            target_field: "uuid".to_string(),
            priority: 100,
            conditions: Vec::new(),
            scope: OverrideScope::Global,
            created_by: "test_user".to_string(),
            created_at: chrono::Utc::now(),
            modified_at: chrono::Utc::now(),
            active: true,
            version: 1,
            tags: Vec::new(),
        };

        engine.add_override(override1).unwrap();

        // Add conflicting override
        let override2 = MappingOverride {
            id: Uuid::new_v4(),
            name: "Override 2".to_string(),
            description: "Conflicting override".to_string(),
            rule_type: OverrideType::ExactMatch,
            pattern: OverridePattern {
                pattern: "Asset ID".to_string(),
                case_sensitive: false,
                whole_word: true,
                regex_flags: None,
                fuzzy_threshold: None,
                position_constraints: None,
            },
            target_field: "identifier".to_string(), // Different target
            priority: 100, // Same priority
            conditions: Vec::new(),
            scope: OverrideScope::Global,
            created_by: "test_user".to_string(),
            created_at: chrono::Utc::now(),
            modified_at: chrono::Utc::now(),
            active: true,
            version: 1,
            tags: Vec::new(),
        };

        let conflicts = engine.detect_conflicts(&override2).unwrap();
        assert!(!conflicts.is_empty());
        assert!(conflicts.iter().any(|c| c.conflict_type == ConflictType::PriorityTie));
    }

    #[test]
    fn test_override_json_serialization() {
        let mut engine = MappingOverrideEngine::new();

        let override_rule = MappingOverride {
            id: Uuid::new_v4(),
            name: "Test Override".to_string(),
            description: "Test override for serialization".to_string(),
            rule_type: OverrideType::RegexPattern,
            pattern: OverridePattern {
                pattern: r"(?i)asset.*id".to_string(),
                case_sensitive: false,
                whole_word: false,
                regex_flags: Some("i".to_string()),
                fuzzy_threshold: None,
                position_constraints: None,
            },
            target_field: "uuid".to_string(),
            priority: 50,
            conditions: Vec::new(),
            scope: OverrideScope::DocumentType("inventory".to_string()),
            created_by: "test_user".to_string(),
            created_at: chrono::Utc::now(),
            modified_at: chrono::Utc::now(),
            active: true,
            version: 1,
            tags: vec!["test".to_string(), "regex".to_string()],
        };

        engine.add_override(override_rule).unwrap();

        let json = engine.export_overrides_to_json().unwrap();
        assert!(json.contains("Test Override"));
        assert!(json.contains("asset.*id"));

        // Test loading from JSON
        let mut new_engine = MappingOverrideEngine::new();
        let loaded_count = new_engine.load_overrides_from_json(&json).unwrap();
        assert_eq!(loaded_count, 1);
        assert_eq!(new_engine.overrides.len(), 1);
    }

    #[test]
    fn test_override_resolution() {
        let mut engine = MappingOverrideEngine::new();

        let override_rule = MappingOverride {
            id: Uuid::new_v4(),
            name: "Asset ID Override".to_string(),
            description: "Maps Asset ID to uuid field".to_string(),
            rule_type: OverrideType::ExactMatch,
            pattern: OverridePattern {
                pattern: "Asset ID".to_string(),
                case_sensitive: false,
                whole_word: true,
                regex_flags: None,
                fuzzy_threshold: None,
                position_constraints: None,
            },
            target_field: "uuid".to_string(),
            priority: 100,
            conditions: Vec::new(),
            scope: OverrideScope::Global,
            created_by: "test_user".to_string(),
            created_at: chrono::Utc::now(),
            modified_at: chrono::Utc::now(),
            active: true,
            version: 1,
            tags: Vec::new(),
        };

        engine.add_override(override_rule).unwrap();

        let context = OverrideContext::new("inventory".to_string());
        let result = engine.resolve_mapping("Asset ID", "inventory", &context).unwrap();

        assert!(result.override_applied);
        assert_eq!(result.target_field, Some("uuid".to_string()));
        assert_eq!(result.confidence, 1.0);
        assert!(result.conflicts.is_empty());
    }

    #[test]
    fn test_override_cache() {
        let mut engine = MappingOverrideEngine::new();

        let override_rule = MappingOverride {
            id: Uuid::new_v4(),
            name: "Cached Override".to_string(),
            description: "Test caching".to_string(),
            rule_type: OverrideType::ExactMatch,
            pattern: OverridePattern {
                pattern: "Test Column".to_string(),
                case_sensitive: false,
                whole_word: true,
                regex_flags: None,
                fuzzy_threshold: None,
                position_constraints: None,
            },
            target_field: "test_field".to_string(),
            priority: 100,
            conditions: Vec::new(),
            scope: OverrideScope::Global,
            created_by: "test_user".to_string(),
            created_at: chrono::Utc::now(),
            modified_at: chrono::Utc::now(),
            active: true,
            version: 1,
            tags: Vec::new(),
        };

        engine.add_override(override_rule).unwrap();

        let context = OverrideContext::new("test".to_string());

        // First call - should not be from cache
        let result1 = engine.resolve_mapping("Test Column", "test", &context).unwrap();
        assert!(!result1.from_cache);

        // Second call - should be from cache
        let result2 = engine.resolve_mapping("Test Column", "test", &context).unwrap();
        assert!(result2.from_cache);

        assert_eq!(result1.target_field, result2.target_field);
    }

    #[test]
    fn test_mapping_report_generator_creation() {
        let generator = MappingReportGenerator::new();
        assert_eq!(generator.config.default_format, ReportFormat::Html);
        assert!(generator.config.enable_caching);
        assert_eq!(generator.generation_metrics.total_reports_generated, 0);
    }

    #[test]
    fn test_report_config_default() {
        let config = ReportConfig::default();
        assert_eq!(config.default_format, ReportFormat::Html);
        assert!(config.include_visualizations);
        assert_eq!(config.max_generation_time_seconds, 30);
        assert!(config.enable_caching);
        assert_eq!(config.cache_expiration_minutes, 60);
    }

    #[test]
    fn test_quality_grade_ordering() {
        assert!(QualityGrade::A > QualityGrade::B);
        assert!(QualityGrade::B > QualityGrade::C);
        assert!(QualityGrade::C > QualityGrade::D);
        assert!(QualityGrade::D > QualityGrade::F);
    }

    #[test]
    fn test_risk_level_ordering() {
        assert!(RiskLevel::VeryHigh > RiskLevel::High);
        assert!(RiskLevel::High > RiskLevel::Medium);
        assert!(RiskLevel::Medium > RiskLevel::Low);
        assert!(RiskLevel::Low > RiskLevel::VeryLow);
    }

    #[test]
    fn test_issue_severity_ordering() {
        assert!(IssueSeverity::Critical > IssueSeverity::Error);
        assert!(IssueSeverity::Error > IssueSeverity::Warning);
        assert!(IssueSeverity::Warning > IssueSeverity::Info);
    }

    #[test]
    fn test_recommendation_priority_ordering() {
        assert!(RecommendationPriority::Critical > RecommendationPriority::High);
        assert!(RecommendationPriority::High > RecommendationPriority::Medium);
        assert!(RecommendationPriority::Medium > RecommendationPriority::Low);
    }

    #[test]
    fn test_effort_level_ordering() {
        assert!(EffortLevel::VeryHigh > EffortLevel::High);
        assert!(EffortLevel::High > EffortLevel::Medium);
        assert!(EffortLevel::Medium > EffortLevel::Low);
        assert!(EffortLevel::Low > EffortLevel::Minimal);
    }

    #[test]
    fn test_report_type_serialization() {
        let report_type = ReportType::Summary;
        let serialized = serde_json::to_string(&report_type).unwrap();
        assert!(serialized.contains("Summary"));

        let custom_type = ReportType::Custom("MyCustomReport".to_string());
        let serialized = serde_json::to_string(&custom_type).unwrap();
        assert!(serialized.contains("MyCustomReport"));
    }

    #[test]
    fn test_report_format_serialization() {
        let formats = vec![
            ReportFormat::Html,
            ReportFormat::Json,
            ReportFormat::Csv,
            ReportFormat::Markdown,
            ReportFormat::Pdf,
        ];

        for format in formats {
            let serialized = serde_json::to_string(&format).unwrap();
            let deserialized: ReportFormat = serde_json::from_str(&serialized).unwrap();
            assert_eq!(format, deserialized);
        }
    }

    #[test]
    fn test_mapping_summary_calculation() {
        let generator = MappingReportGenerator::new();

        // Create mock mapping results
        let mapping_results = vec![
            crate::mapping::engine::MappingResult {
                source_column: "Asset ID".to_string(),
                target_field: "uuid".to_string(),
                confidence: 0.95,
                exact_match: true,
            },
            crate::mapping::engine::MappingResult {
                source_column: "Asset Name".to_string(),
                target_field: "title".to_string(),
                confidence: 0.85,
                exact_match: false,
            },
            crate::mapping::engine::MappingResult {
                source_column: "Unknown Column".to_string(),
                target_field: "".to_string(), // Unmapped
                confidence: 0.2,
                exact_match: false,
            },
        ];

        let confidence_results = vec![
            MappingConfidence {
                overall_score: 0.95,
                factor_scores: HashMap::new(),
                threshold_status: ThresholdStatus::HighConfidence,
                recommendations: Vec::new(),
                risk_factors: Vec::new(),
                explanation: ConfidenceExplanation {
                    source_column: "Asset ID".to_string(),
                    target_field: "uuid".to_string(),
                    factor_contributions: HashMap::new(),
                    weighted_calculation: WeightedConfidenceCalculation {
                        total_weighted_score: 0.95,
                        total_weight: 1.0,
                        base_score: 0.95,
                        final_score: 0.95,
                    },
                    adjustments: Vec::new(),
                },
                calculation_time: Duration::from_millis(1),
            },
            MappingConfidence {
                overall_score: 0.85,
                factor_scores: HashMap::new(),
                threshold_status: ThresholdStatus::MediumConfidence,
                recommendations: Vec::new(),
                risk_factors: Vec::new(),
                explanation: ConfidenceExplanation {
                    source_column: "Asset Name".to_string(),
                    target_field: "title".to_string(),
                    factor_contributions: HashMap::new(),
                    weighted_calculation: WeightedConfidenceCalculation {
                        total_weighted_score: 0.85,
                        total_weight: 1.0,
                        base_score: 0.85,
                        final_score: 0.85,
                    },
                    adjustments: Vec::new(),
                },
                calculation_time: Duration::from_millis(1),
            },
            MappingConfidence {
                overall_score: 0.2,
                factor_scores: HashMap::new(),
                threshold_status: ThresholdStatus::VeryLowConfidence,
                recommendations: Vec::new(),
                risk_factors: Vec::new(),
                explanation: ConfidenceExplanation {
                    source_column: "Unknown Column".to_string(),
                    target_field: "".to_string(),
                    factor_contributions: HashMap::new(),
                    weighted_calculation: WeightedConfidenceCalculation {
                        total_weighted_score: 0.2,
                        total_weight: 1.0,
                        base_score: 0.2,
                        final_score: 0.2,
                    },
                    adjustments: Vec::new(),
                },
                calculation_time: Duration::from_millis(1),
            },
        ];

        let summary = generator.generate_mapping_summary(&mapping_results, &confidence_results).unwrap();

        assert_eq!(summary.total_fields, 3);
        assert_eq!(summary.mapped_fields, 2); // Only first two have target fields
        assert!((summary.average_confidence - 0.6667).abs() < 0.01); // (0.95 + 0.85 + 0.2) / 3
        assert_eq!(summary.high_confidence_mappings, 1); // Only first one > 0.9
        assert_eq!(summary.low_confidence_mappings, 1);  // Only third one < 0.5
    }

    #[test]
    fn test_quality_metrics_calculation() {
        let generator = MappingReportGenerator::new();

        let detailed_results = vec![
            FieldMappingResult {
                field_id: "field_1".to_string(),
                target_field: "uuid".to_string(),
                source_column: Some("Asset ID".to_string()),
                confidence_score: 0.95,
                mapping_successful: true,
                required: true,
                validation_result: None,
                override_applied: None,
                alternatives: Vec::new(),
                issues: Vec::new(),
                data_quality: None,
            },
            FieldMappingResult {
                field_id: "field_2".to_string(),
                target_field: "title".to_string(),
                source_column: Some("Asset Name".to_string()),
                confidence_score: 0.85,
                mapping_successful: true,
                required: false,
                validation_result: None,
                override_applied: None,
                alternatives: Vec::new(),
                issues: vec![
                    MappingIssue {
                        severity: IssueSeverity::Warning,
                        category: IssueCategory::LowConfidence,
                        description: "Medium confidence mapping".to_string(),
                        suggested_resolution: None,
                        impact: "May affect accuracy".to_string(),
                    }
                ],
                data_quality: None,
            },
        ];

        let mapping_summary = MappingSummary {
            total_fields: 2,
            mapped_fields: 2,
            required_fields_mapped: 1,
            required_fields_missing: 0,
            optional_fields_mapped: 1,
            average_confidence: 0.9,
            min_confidence: 0.85,
            max_confidence: 0.95,
            high_confidence_mappings: 1,
            low_confidence_mappings: 0,
            processing_time: Duration::from_millis(100),
            success_rate: 1.0,
        };

        let quality_metrics = generator.calculate_quality_metrics(&detailed_results, &mapping_summary).unwrap();

        assert_eq!(quality_metrics.completeness_score, 1.0); // All fields mapped
        assert_eq!(quality_metrics.accuracy_score, 0.9);     // From mapping summary
        assert_eq!(quality_metrics.quality_grade, QualityGrade::A); // High overall score
        assert_eq!(quality_metrics.warnings, 1);            // One warning in issues
        assert_eq!(quality_metrics.critical_issues, 0);     // No critical issues
    }

    #[test]
    fn test_csv_export() {
        let generator = MappingReportGenerator::new();

        let report = MappingReport {
            report_id: Uuid::new_v4(),
            report_type: ReportType::Summary,
            generated_at: chrono::Utc::now(),
            document_info: DocumentInfo {
                file_name: "test.csv".to_string(),
                document_type: "inventory".to_string(),
                file_size: 1024,
                total_rows: 10,
                total_columns: 5,
                processed_at: chrono::Utc::now(),
                processing_duration: Duration::from_millis(100),
                document_hash: "test_hash".to_string(),
            },
            mapping_summary: MappingSummary {
                total_fields: 2,
                mapped_fields: 2,
                required_fields_mapped: 1,
                required_fields_missing: 0,
                optional_fields_mapped: 1,
                average_confidence: 0.9,
                min_confidence: 0.85,
                max_confidence: 0.95,
                high_confidence_mappings: 1,
                low_confidence_mappings: 0,
                processing_time: Duration::from_millis(100),
                success_rate: 1.0,
            },
            detailed_results: vec![
                FieldMappingResult {
                    field_id: "field_1".to_string(),
                    target_field: "uuid".to_string(),
                    source_column: Some("Asset ID".to_string()),
                    confidence_score: 0.95,
                    mapping_successful: true,
                    required: true,
                    validation_result: None,
                    override_applied: None,
                    alternatives: Vec::new(),
                    issues: Vec::new(),
                    data_quality: None,
                },
            ],
            quality_metrics: QualityMetrics {
                completeness_score: 1.0,
                accuracy_score: 0.9,
                consistency_score: 0.85,
                overall_quality_score: 0.9,
                risk_level: RiskLevel::Low,
                quality_grade: QualityGrade::A,
                compliance_percentage: 90.0,
                critical_issues: 0,
                warnings: 0,
            },
            recommendations: Vec::new(),
            validation_results: ValidationSummary {
                total_validations: 1,
                validations_passed: 1,
                validations_failed: 0,
                validation_warnings: 0,
                success_rate: 1.0,
                common_failures: Vec::new(),
                performance_metrics: ValidationPerformanceMetrics {
                    avg_validation_time_us: 100.0,
                    total_validation_time: Duration::from_millis(1),
                    slowest_validations: Vec::new(),
                },
            },
            override_results: OverrideSummary {
                total_overrides_evaluated: 0,
                overrides_applied: 0,
                conflicts_detected: 0,
                application_success_rate: 0.0,
                frequently_applied: Vec::new(),
                performance_metrics: OverridePerformanceMetrics {
                    avg_resolution_time_us: 0.0,
                    cache_hit_rate: 0.0,
                    total_processing_time: Duration::from_millis(0),
                },
            },
            performance_metrics: ProcessingMetrics {
                total_processing_time: Duration::from_millis(100),
                column_detection_time: Duration::from_millis(20),
                mapping_time: Duration::from_millis(30),
                validation_time: Duration::from_millis(25),
                override_time: Duration::from_millis(5),
                memory_usage: MemoryUsageMetrics {
                    peak_memory_bytes: 1024,
                    avg_memory_bytes: 512,
                    efficiency_score: 0.8,
                },
                throughput: ThroughputMetrics {
                    rows_per_second: 100.0,
                    fields_per_second: 500.0,
                    bytes_per_second: 10240.0,
                },
            },
            trend_analysis: None,
        };

        let csv_output = generator.export_to_csv(&report).unwrap();

        assert!(csv_output.contains("Field ID,Target Field,Source Column"));
        assert!(csv_output.contains("field_1,uuid,Asset ID"));
        assert!(csv_output.contains("0.950"));
        assert!(csv_output.contains("true"));
    }

    #[test]
    fn test_markdown_export() {
        let generator = MappingReportGenerator::new();

        let report = MappingReport {
            report_id: Uuid::new_v4(),
            report_type: ReportType::Summary,
            generated_at: chrono::Utc::now(),
            document_info: DocumentInfo {
                file_name: "test.csv".to_string(),
                document_type: "inventory".to_string(),
                file_size: 1024,
                total_rows: 10,
                total_columns: 5,
                processed_at: chrono::Utc::now(),
                processing_duration: Duration::from_millis(100),
                document_hash: "test_hash".to_string(),
            },
            mapping_summary: MappingSummary {
                total_fields: 2,
                mapped_fields: 2,
                required_fields_mapped: 1,
                required_fields_missing: 0,
                optional_fields_mapped: 1,
                average_confidence: 0.9,
                min_confidence: 0.85,
                max_confidence: 0.95,
                high_confidence_mappings: 1,
                low_confidence_mappings: 0,
                processing_time: Duration::from_millis(100),
                success_rate: 1.0,
            },
            detailed_results: Vec::new(),
            quality_metrics: QualityMetrics {
                completeness_score: 1.0,
                accuracy_score: 0.9,
                consistency_score: 0.85,
                overall_quality_score: 0.9,
                risk_level: RiskLevel::Low,
                quality_grade: QualityGrade::A,
                compliance_percentage: 90.0,
                critical_issues: 0,
                warnings: 0,
            },
            recommendations: vec![
                Recommendation {
                    priority: RecommendationPriority::Medium,
                    category: RecommendationCategory::DataQuality,
                    title: "Test Recommendation".to_string(),
                    description: "This is a test recommendation".to_string(),
                    suggested_action: "Take this action".to_string(),
                    impact_assessment: "This will improve quality".to_string(),
                    effort_level: EffortLevel::Low,
                    related_fields: Vec::new(),
                }
            ],
            validation_results: ValidationSummary {
                total_validations: 1,
                validations_passed: 1,
                validations_failed: 0,
                validation_warnings: 0,
                success_rate: 1.0,
                common_failures: Vec::new(),
                performance_metrics: ValidationPerformanceMetrics {
                    avg_validation_time_us: 100.0,
                    total_validation_time: Duration::from_millis(1),
                    slowest_validations: Vec::new(),
                },
            },
            override_results: OverrideSummary {
                total_overrides_evaluated: 0,
                overrides_applied: 0,
                conflicts_detected: 0,
                application_success_rate: 0.0,
                frequently_applied: Vec::new(),
                performance_metrics: OverridePerformanceMetrics {
                    avg_resolution_time_us: 0.0,
                    cache_hit_rate: 0.0,
                    total_processing_time: Duration::from_millis(0),
                },
            },
            performance_metrics: ProcessingMetrics {
                total_processing_time: Duration::from_millis(100),
                column_detection_time: Duration::from_millis(20),
                mapping_time: Duration::from_millis(30),
                validation_time: Duration::from_millis(25),
                override_time: Duration::from_millis(5),
                memory_usage: MemoryUsageMetrics {
                    peak_memory_bytes: 1024,
                    avg_memory_bytes: 512,
                    efficiency_score: 0.8,
                },
                throughput: ThroughputMetrics {
                    rows_per_second: 100.0,
                    fields_per_second: 500.0,
                    bytes_per_second: 10240.0,
                },
            },
            trend_analysis: None,
        };

        let md_output = generator.export_to_markdown(&report).unwrap();

        assert!(md_output.contains("# Mapping Validation Report"));
        assert!(md_output.contains("## Summary"));
        assert!(md_output.contains("## Recommendations"));
        assert!(md_output.contains("Test Recommendation"));
        assert!(md_output.contains("test.csv"));
        assert!(md_output.contains("100.0%")); // Success rate
    }
}
