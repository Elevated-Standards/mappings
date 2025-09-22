// Modified: 2025-09-22

//! Override type definitions and data structures
//!
//! This module contains all the type definitions, enums, and data structures
//! used for mapping override functionality.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

/// Mapping override rule
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MappingOverride {
    /// Unique identifier for the override
    pub id: Uuid,
    /// Human-readable name for the override
    pub name: String,
    /// Description of what this override does
    pub description: String,
    /// Override type (exact match, regex, etc.)
    pub override_type: OverrideType,
    /// Pattern to match against
    pub pattern: OverridePattern,
    /// Target field to map to
    pub target_field: String,
    /// Priority (higher numbers take precedence)
    pub priority: i32,
    /// Whether this override is active
    pub active: bool,
    /// Scope of the override (global, document-specific, etc.)
    pub scope: OverrideScope,
    /// Conditions that must be met for this override to apply
    pub conditions: Vec<OverrideCondition>,
    /// Position constraints (column index, etc.)
    pub position_constraints: Option<PositionConstraints>,
    /// Created timestamp
    pub created_at: chrono::DateTime<chrono::Utc>,
    /// Last modified timestamp
    pub modified_at: chrono::DateTime<chrono::Utc>,
    /// Created by user
    pub created_by: String,
}

/// Types of override patterns
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum OverrideType {
    /// Exact string match (case-sensitive or insensitive)
    ExactMatch,
    /// Regular expression pattern matching
    RegexMatch,
    /// Fuzzy string matching with threshold
    FuzzyMatch,
    /// Starts with pattern
    StartsWith,
    /// Ends with pattern
    EndsWith,
    /// Contains pattern
    Contains,
    /// Word boundary match
    WordBoundary,
}

/// Override pattern configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OverridePattern {
    /// The pattern string (regex, exact match, etc.)
    pub pattern: String,
    /// Whether matching should be case-sensitive
    pub case_sensitive: bool,
    /// For fuzzy matching, the minimum similarity threshold
    pub similarity_threshold: Option<f64>,
    /// Whether to match whole words only
    pub whole_words_only: bool,
    /// Whether to normalize whitespace before matching
    pub normalize_whitespace: bool,
    /// Whether to strip special characters before matching
    pub strip_special_chars: bool,
}

/// Position constraints for override matching
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PositionConstraints {
    /// Minimum column index (0-based)
    pub min_index: Option<usize>,
    /// Maximum column index (0-based)
    pub max_index: Option<usize>,
    /// Specific column indices where this override applies
    pub specific_indices: Option<Vec<usize>>,
    /// Whether to apply to header row only
    pub header_only: bool,
}

/// Condition that must be met for override to apply
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

/// Types of conditions for override application
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ConditionType {
    /// Document type condition
    DocumentType,
    /// File name pattern condition
    FileName,
    /// Column count condition
    ColumnCount,
    /// Row count condition
    RowCount,
    /// Header content condition
    HeaderContent,
    /// Cell content condition
    CellContent,
    /// File size condition
    FileSize,
    /// Date condition
    Date,
    /// User condition
    User,
}

/// Operators for condition evaluation
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ConditionOperator {
    /// Equal to
    Equals,
    /// Not equal to
    NotEquals,
    /// Greater than
    GreaterThan,
    /// Greater than or equal to
    GreaterThanOrEqual,
    /// Less than
    LessThan,
    /// Less than or equal to
    LessThanOrEqual,
    /// Contains
    Contains,
    /// Does not contain
    NotContains,
    /// Starts with
    StartsWith,
    /// Ends with
    EndsWith,
    /// Matches regex
    MatchesRegex,
    /// In list
    In,
    /// Not in list
    NotIn,
}

/// Scope of override application
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum OverrideScope {
    /// Global override (applies to all documents)
    Global,
    /// Document type specific
    DocumentType(String),
    /// File name pattern specific
    FilePattern(String),
    /// User specific
    User(String),
    /// Organization specific
    Organization(String),
    /// Project specific
    Project(String),
}

/// Strategies for resolving override conflicts
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ConflictResolutionStrategy {
    /// Use highest priority rule
    HighestPriority,
    /// Use most recently created rule
    MostRecent,
    /// Use most specific rule (narrowest scope)
    MostSpecific,
    /// Combine rules if possible
    Combine,
    /// Report conflict and require manual resolution
    Manual,
}

/// Validation rules for override patterns
#[derive(Debug, Clone)]
pub struct ValidationRuleSet {
    /// Maximum pattern length
    pub max_pattern_length: usize,
    /// Maximum number of conditions per override
    pub max_conditions: usize,
    /// Allowed regex features
    pub allowed_regex_features: Vec<String>,
    /// Forbidden patterns
    pub forbidden_patterns: Vec<String>,
    /// Maximum priority value
    pub max_priority: i32,
    /// Minimum priority value
    pub min_priority: i32,
}

/// Metrics for override engine performance
#[derive(Debug, Clone, Default)]
pub struct OverrideMetrics {
    /// Total number of override applications
    pub total_applications: u64,
    /// Number of successful matches
    pub successful_matches: u64,
    /// Number of conflicts encountered
    pub conflicts_encountered: u64,
    /// Number of validation failures
    pub validation_failures: u64,
    /// Average resolution time in microseconds
    pub avg_resolution_time_us: f64,
    /// Cache hit rate
    pub cache_hit_rate: f64,
}

/// Result of override resolution
#[derive(Debug, Clone)]
pub struct OverrideResolutionResult {
    /// Whether an override was applied
    pub override_applied: bool,
    /// The override rule that was applied (if any)
    pub applied_override: Option<MappingOverride>,
    /// Target field from the override
    pub target_field: Option<String>,
    /// Confidence score for the override match
    pub confidence_score: f64,
    /// Any conflicts that were encountered
    pub conflicts: Vec<OverrideConflict>,
    /// Resolution time in microseconds
    pub resolution_time_us: u64,
}

/// Override conflict information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OverrideConflict {
    /// Conflicting override rules
    pub conflicting_overrides: Vec<Uuid>,
    /// Type of conflict
    pub conflict_type: ConflictType,
    /// Severity of the conflict
    pub severity: ConflictSeverity,
    /// Description of the conflict
    pub description: String,
    /// Suggested resolution
    pub suggested_resolution: String,
}

/// Types of override conflicts
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ConflictType {
    /// Multiple rules match the same pattern
    PatternOverlap,
    /// Rules have same priority but different targets
    PriorityTie,
    /// Rules have conflicting conditions
    ConflictingConditions,
    /// Circular dependency between rules
    CircularDependency,
    /// Rule scope conflicts
    ScopeConflict,
}

/// Severity of override conflicts
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

/// Context for override resolution
#[derive(Debug, Clone)]
pub struct OverrideContext {
    /// Document type being processed
    pub document_type: String,
    /// File name (if available)
    pub file_name: Option<String>,
    /// Column headers
    pub headers: Vec<String>,
    /// Number of rows in the document
    pub row_count: usize,
    /// File size in bytes
    pub file_size: u64,
    /// Processing timestamp
    pub timestamp: chrono::DateTime<chrono::Utc>,
    /// User identifier
    pub user_id: Option<String>,
    /// Additional metadata
    pub metadata: HashMap<String, serde_json::Value>,
}

impl Default for ValidationRuleSet {
    fn default() -> Self {
        Self {
            max_pattern_length: 1000,
            max_conditions: 10,
            allowed_regex_features: vec!["basic".to_string()],
            forbidden_patterns: vec![".*".to_string()], // Overly broad patterns
            max_priority: 1000,
            min_priority: -1000,
        }
    }
}
