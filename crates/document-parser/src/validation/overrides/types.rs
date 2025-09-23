// Modified: 2025-09-22

//! Override type definitions and data structures
//!
//! This module contains all the type definitions, enums, and data structures
//! used for mapping override functionality in the validation system.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::Duration;
use uuid::Uuid;
use chrono::{DateTime, Utc};

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
    /// Pattern to match against
    pub pattern: OverridePattern,
    /// Target field to map to
    pub target_field: String,
    /// Priority for conflict resolution (higher = more important)
    pub priority: i32,
    /// Conditions that must be met for this override to apply
    pub conditions: Vec<OverrideCondition>,
    /// Scope where this override applies
    pub scope: OverrideScope,
    /// User who created this override
    pub created_by: String,
    /// When this override was created
    pub created_at: DateTime<Utc>,
    /// When this override was last modified
    pub modified_at: DateTime<Utc>,
    /// Whether this override is currently active
    pub active: bool,
    /// Version number for tracking changes
    pub version: u32,
    /// Tags for categorization and filtering
    pub tags: Vec<String>,
}

/// Types of override rules
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum OverrideType {
    /// Exact string match (case-sensitive or insensitive)
    ExactMatch,
    /// Regular expression pattern matching
    RegexPattern,
    /// Fuzzy string matching with similarity threshold
    FuzzyMatch,
    /// Contains substring matching
    ContainsMatch,
    /// Prefix/suffix matching
    PrefixSuffixMatch,
    /// Position-based matching (column index)
    PositionalMatch,
    /// Conditional matching based on other fields
    ConditionalMatch,
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
    /// Additional regex flags for regex patterns
    pub regex_flags: Option<String>,
    /// Similarity threshold for fuzzy matching (0.0-1.0)
    pub fuzzy_threshold: Option<f64>,
    /// Position constraints for positional matching
    pub position_constraints: Option<PositionConstraints>,
}

/// Position constraints for column matching
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PositionConstraints {
    /// Minimum column index (0-based)
    pub min_index: Option<usize>,
    /// Maximum column index (0-based)
    pub max_index: Option<usize>,
    /// Exact column index
    pub exact_index: Option<usize>,
    /// Relative position (e.g., "first", "last", "second")
    pub relative_position: Option<String>,
}

/// Condition that must be met for override to apply
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OverrideCondition {
    /// Type of condition to check
    pub condition_type: ConditionType,
    /// Field or property to check
    pub field: String,
    /// Comparison operator
    pub operator: ConditionOperator,
    /// Value to compare against
    pub value: serde_json::Value,
    /// Whether this condition is required (AND) or optional (OR)
    pub required: bool,
}

/// Types of conditions for override rules
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ConditionType {
    /// Document type condition
    DocumentType,
    /// File name pattern condition
    FileName,
    /// Column count condition
    ColumnCount,
    /// Data sample condition (check actual data)
    DataSample,
    /// User role condition
    UserRole,
    /// Organization condition
    Organization,
    /// Custom metadata condition
    CustomMetadata,
}

/// Operators for condition evaluation
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
    /// Document type specific (e.g., only for inventory documents)
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

/// Strategies for resolving override conflicts
#[derive(Debug, Clone, PartialEq)]
pub enum ConflictResolutionStrategy {
    /// Use highest priority rule
    HighestPriority,
    /// Use most recently created rule
    MostRecent,
    /// Use most specific rule (narrowest scope)
    MostSpecific,
    /// Combine rules if possible
    Combine,
    /// Report conflict and use fallback
    ReportAndFallback,
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
    pub last_updated: DateTime<Utc>,
}

/// Result of override resolution
#[derive(Debug, Clone)]
pub struct OverrideResolutionResult {
    /// Whether an override was applied
    pub override_applied: bool,
    /// Target field if override was applied
    pub target_field: Option<String>,
    /// Confidence score for the override match
    pub confidence: f64,
    /// Applied override rule
    pub applied_override: Option<MappingOverride>,
    /// Alternative overrides that also matched
    pub alternatives: Vec<MappingOverride>,
    /// Conflicts detected during resolution
    pub conflicts: Vec<OverrideConflict>,
    /// Time taken to resolve
    pub resolution_time: Duration,
    /// Whether result came from cache
    pub from_cache: bool,
}

/// Information about override conflicts
#[derive(Debug, Clone)]
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
    pub suggested_resolution: Option<String>,
    /// How the conflict was resolved
    pub resolution_applied: Option<ConflictResolutionStrategy>,
}

/// Types of conflicts between override rules
#[derive(Debug, Clone, PartialEq)]
pub enum ConflictType {
    /// Multiple rules match the same pattern
    PatternOverlap,
    /// Rules have the same priority
    PriorityTie,
    /// Rules have circular dependencies
    CircularDependency,
    /// Rules have contradictory conditions
    ContradictoryConditions,
    /// Rules have overlapping scopes
    ScopeOverlap,
}

/// Severity levels for conflicts
#[derive(Debug, Clone, PartialEq, PartialOrd)]
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
    /// File name being processed
    pub file_name: Option<String>,
    /// User performing the operation
    pub user_id: Option<String>,
    /// Organization context
    pub organization: Option<String>,
    /// Session identifier
    pub session_id: Option<String>,
    /// Project identifier
    pub project_id: Option<String>,
    /// Custom metadata
    pub metadata: HashMap<String, serde_json::Value>,
    /// Column count in the document
    pub column_count: Option<usize>,
    /// Sample data for analysis
    pub sample_data: Option<Vec<serde_json::Value>>,
}

/// Result of conflict resolution
#[derive(Debug, Clone)]
pub struct ConflictResolution {
    /// Selected override after conflict resolution
    pub selected_override: Option<MappingOverride>,
    /// Alternative overrides that were not selected
    pub alternatives: Vec<MappingOverride>,
    /// Conflicts that were detected
    pub conflicts: Vec<OverrideConflict>,
}
