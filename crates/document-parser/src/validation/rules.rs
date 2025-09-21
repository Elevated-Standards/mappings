//! Validation rules and configurations

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use super::types::ValidationSeverity;

/// A validation rule for a specific field
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationRule {
    /// Field name to validate
    pub field_name: String,
    /// Type of validation to perform
    pub validation_type: ValidationType,
    /// Whether this field is required
    pub required: bool,
    /// Severity level if validation fails
    pub severity: ValidationSeverity,
    /// Custom error message
    pub error_message: Option<String>,
    /// Additional validation parameters
    pub parameters: HashMap<String, serde_json::Value>,
}

/// Column-specific validation rule
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ColumnValidationRule {
    /// Field identifier
    pub field_id: String,
    /// Human-readable field name
    pub field_name: String,
    /// Expected data type
    pub expected_type: DataType,
    /// Whether this field is required
    pub required: bool,
    /// Allowed values for enumeration fields
    pub allowed_values: Option<Vec<String>>,
    /// Regular expression pattern for string validation
    pub pattern: Option<String>,
    /// Minimum value for numeric fields
    pub min_value: Option<f64>,
    /// Maximum value for numeric fields
    pub max_value: Option<f64>,
    /// Conditional requirements
    pub conditional_requirements: Vec<ConditionalRequirement>,
    /// Custom validation function name
    pub custom_validator: Option<String>,
}

/// Conditional requirement for field validation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConditionalRequirement {
    /// Field that determines the condition
    pub depends_on: String,
    /// Value that triggers the requirement
    pub trigger_value: serde_json::Value,
    /// Whether the field becomes required when condition is met
    pub makes_required: bool,
}

/// Types of validation that can be performed
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ValidationType {
    /// Check if field is present
    Presence,
    /// Validate data type
    DataType,
    /// Validate against enumeration values
    Enumeration,
    /// Validate string pattern (regex)
    Pattern,
    /// Validate numeric range
    Range,
    /// Validate date format
    DateFormat,
    /// Validate email format
    Email,
    /// Validate URL format
    Url,
    /// Validate IP address format
    IpAddress,
    /// Validate UUID format
    Uuid,
    /// Custom validation function
    Custom(String),
}

/// Expected data types for validation
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum DataType {
    /// String data type
    String,
    /// Integer number
    Integer,
    /// Floating point number
    Float,
    /// Boolean value
    Boolean,
    /// Date value
    Date,
    /// DateTime value
    DateTime,
    /// UUID value
    Uuid,
    /// Email address
    Email,
    /// URL
    Url,
    /// IP address
    IpAddress,
    /// JSON object
    Object,
    /// JSON array
    Array,
    /// Any type (no validation)
    Any,
}

/// Configuration for validation thresholds
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThresholdConfig {
    /// High confidence threshold (auto-accept)
    pub high_confidence: f64,
    /// Medium confidence threshold (review recommended)
    pub medium_confidence: f64,
    /// Low confidence threshold (manual review required)
    pub low_confidence: f64,
    /// Minimum acceptable confidence for auto-processing
    pub min_acceptable: f64,
    /// Performance threshold in milliseconds
    pub performance_threshold_ms: u64,
}

impl Default for ThresholdConfig {
    fn default() -> Self {
        Self {
            high_confidence: 0.9,
            medium_confidence: 0.7,
            low_confidence: 0.5,
            min_acceptable: 0.6,
            performance_threshold_ms: 100,
        }
    }
}

/// Validation rule set for a document type
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationRuleSet {
    /// Maximum pattern length
    pub max_pattern_length: usize,
    /// Maximum number of conditions per rule
    pub max_conditions: usize,
    /// Allowed validation types
    pub allowed_types: Vec<ValidationType>,
    /// Required tags for certain scopes
    pub required_tags: HashMap<String, Vec<String>>,
    /// Forbidden patterns
    pub forbidden_patterns: Vec<String>,
    /// Target field allowlist
    pub target_field_allowlist: Option<Vec<String>>,
}

impl Default for ValidationRuleSet {
    fn default() -> Self {
        Self {
            max_pattern_length: 1000,
            max_conditions: 10,
            allowed_types: vec![
                ValidationType::Presence,
                ValidationType::DataType,
                ValidationType::Enumeration,
                ValidationType::Pattern,
                ValidationType::Range,
                ValidationType::DateFormat,
                ValidationType::Email,
                ValidationType::Url,
                ValidationType::IpAddress,
                ValidationType::Uuid,
            ],
            required_tags: HashMap::new(),
            forbidden_patterns: vec![
                r".*\.\*.*".to_string(), // Overly broad regex
                r".*\+.*".to_string(),   // Potentially dangerous regex
            ],
            target_field_allowlist: None,
        }
    }
}

/// Configuration for scoring system
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScoringConfig {
    /// Minimum acceptable confidence score
    pub min_acceptable_score: f64,
    /// Weight for string similarity factor
    pub string_similarity_weight: f64,
    /// Weight for semantic similarity factor
    pub semantic_similarity_weight: f64,
    /// Weight for historical success factor
    pub historical_success_weight: f64,
    /// Weight for user feedback factor
    pub user_feedback_weight: f64,
    /// Weight for data type compatibility factor
    pub data_type_weight: f64,
    /// Threshold configuration
    pub thresholds: ThresholdConfig,
}

impl Default for ScoringConfig {
    fn default() -> Self {
        Self {
            min_acceptable_score: 0.6,
            string_similarity_weight: 0.3,
            semantic_similarity_weight: 0.25,
            historical_success_weight: 0.2,
            user_feedback_weight: 0.15,
            data_type_weight: 0.1,
            thresholds: ThresholdConfig::default(),
        }
    }
}

/// Historical mapping data for confidence scoring
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistoricalMappings {
    /// Successful mappings with their confidence scores
    pub successful_mappings: HashMap<String, Vec<HistoricalMapping>>,
    /// Failed mappings to avoid
    pub failed_mappings: HashMap<String, Vec<HistoricalMapping>>,
    /// User feedback on mappings
    pub user_feedback: HashMap<String, Vec<UserFeedback>>,
    /// Accuracy statistics by confidence range
    pub accuracy_stats: HashMap<String, AccuracyStats>,
}

/// Individual historical mapping record
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistoricalMapping {
    /// Source column name
    pub source_column: String,
    /// Target field name
    pub target_field: String,
    /// Confidence score when mapped
    pub confidence_score: f64,
    /// Whether the mapping was successful
    pub was_successful: bool,
    /// Timestamp of the mapping
    pub timestamp: chrono::DateTime<chrono::Utc>,
    /// Document type context
    pub document_type: String,
    /// User who performed the mapping
    pub user_id: Option<String>,
}

/// User feedback on mapping quality
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserFeedback {
    /// User rating (1-5 scale)
    pub rating: u8,
    /// Optional comment from user
    pub comment: Option<String>,
    /// Timestamp of feedback
    pub timestamp: chrono::DateTime<chrono::Utc>,
    /// User identifier
    pub user_id: String,
    /// Whether user confirmed the mapping
    pub confirmed: bool,
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
    /// Average user rating
    pub avg_user_rating: Option<f64>,
    /// Last updated timestamp
    pub last_updated: chrono::DateTime<chrono::Utc>,
}

impl Default for HistoricalMappings {
    fn default() -> Self {
        Self {
            successful_mappings: HashMap::new(),
            failed_mappings: HashMap::new(),
            user_feedback: HashMap::new(),
            accuracy_stats: HashMap::new(),
        }
    }
}
