// Modified: 2025-09-22

//! Mapping override engine and conflict resolution
//!
//! This module provides functionality for managing mapping overrides,
//! resolving conflicts, and validating override rules.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;
use regex::Regex;
use lru::LruCache;

/// Mapping override engine for handling custom mapping rules
#[derive(Debug)]
pub struct MappingOverrideEngine {
    /// Active override rules
    overrides: Vec<MappingOverride>,
    /// LRU cache for override resolution results
    resolution_cache: LruCache<String, OverrideResolutionResult>,
    /// Conflict resolver
    conflict_resolver: ConflictResolver,
    /// Override validator
    validator: OverrideValidator,
    /// Performance metrics
    metrics: OverrideMetrics,
}

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

/// Conflict resolver for handling overlapping overrides
#[derive(Debug)]
pub struct ConflictResolver {
    /// Strategy for resolving conflicts
    resolution_strategy: ConflictResolutionStrategy,
    /// Maximum number of conflicts to report
    max_conflicts_reported: usize,
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

/// Override validator for validating override rules
#[derive(Debug)]
pub struct OverrideValidator {
    /// Compiled regex patterns cache
    regex_cache: HashMap<String, Regex>,
    /// Validation rule set
    validation_rules: ValidationRuleSet,
    /// Maximum cache size
    max_cache_size: usize,
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

impl MappingOverrideEngine {
    /// Create a new mapping override engine
    pub fn new() -> Self {
        Self {
            overrides: Vec::new(),
            resolution_cache: LruCache::new(std::num::NonZeroUsize::new(1000).unwrap()),
            conflict_resolver: ConflictResolver::new(),
            validator: OverrideValidator::new(),
            metrics: OverrideMetrics::default(),
        }
    }

    /// Add a new override rule
    pub fn add_override(&mut self, override_rule: MappingOverride) -> Result<(), String> {
        // Validate the override rule
        if let Err(e) = self.validator.validate_override(&override_rule) {
            return Err(format!("Invalid override rule: {}", e));
        }

        // Check for conflicts
        let conflicts = self.detect_conflicts(&override_rule);
        if !conflicts.is_empty() {
            self.metrics.conflicts_encountered += conflicts.len() as u64;
            // Handle conflicts based on resolution strategy
            match self.conflict_resolver.resolve_conflicts(&override_rule, &conflicts) {
                Ok(_) => {
                    self.overrides.push(override_rule);
                    Ok(())
                }
                Err(e) => Err(format!("Conflict resolution failed: {}", e)),
            }
        } else {
            self.overrides.push(override_rule);
            Ok(())
        }
    }

    /// Resolve override for a given column name and context
    pub fn resolve_override(
        &mut self,
        column_name: &str,
        context: &OverrideContext,
    ) -> OverrideResolutionResult {
        let start_time = std::time::Instant::now();

        // Check cache first
        let cache_key = format!("{}:{}", column_name, context.document_type);
        if let Some(cached_result) = self.resolution_cache.get(&cache_key) {
            self.metrics.total_applications += 1;
            return cached_result.clone();
        }

        let mut matching_overrides = Vec::new();

        // Find all matching overrides
        for override_rule in &self.overrides {
            if override_rule.active && self.matches_override(column_name, override_rule, context) {
                matching_overrides.push(override_rule.clone());
            }
        }

        let result = if matching_overrides.is_empty() {
            OverrideResolutionResult {
                override_applied: false,
                applied_override: None,
                target_field: None,
                confidence_score: 0.0,
                conflicts: Vec::new(),
                resolution_time_us: start_time.elapsed().as_micros() as u64,
            }
        } else if matching_overrides.len() == 1 {
            let override_rule = &matching_overrides[0];
            OverrideResolutionResult {
                override_applied: true,
                applied_override: Some(override_rule.clone()),
                target_field: Some(override_rule.target_field.clone()),
                confidence_score: 1.0,
                conflicts: Vec::new(),
                resolution_time_us: start_time.elapsed().as_micros() as u64,
            }
        } else {
            // Multiple matches - resolve conflicts
            let conflicts = self.create_conflicts_from_matches(&matching_overrides);
            match self.conflict_resolver.resolve_multiple_matches(&matching_overrides) {
                Ok(selected_override) => OverrideResolutionResult {
                    override_applied: true,
                    applied_override: Some(selected_override.clone()),
                    target_field: Some(selected_override.target_field.clone()),
                    confidence_score: 0.8, // Reduced confidence due to conflicts
                    conflicts,
                    resolution_time_us: start_time.elapsed().as_micros() as u64,
                },
                Err(_) => OverrideResolutionResult {
                    override_applied: false,
                    applied_override: None,
                    target_field: None,
                    confidence_score: 0.0,
                    conflicts,
                    resolution_time_us: start_time.elapsed().as_micros() as u64,
                },
            }
        };

        // Update metrics
        self.metrics.total_applications += 1;
        if result.override_applied {
            self.metrics.successful_matches += 1;
        }

        // Cache the result
        self.resolution_cache.put(cache_key, result.clone());

        result
    }

    /// Check if a column matches an override rule
    fn matches_override(
        &self,
        column_name: &str,
        override_rule: &MappingOverride,
        context: &OverrideContext,
    ) -> bool {
        // Check scope
        if !self.matches_scope(&override_rule.scope, context) {
            return false;
        }

        // Check conditions
        if !self.evaluate_conditions(&override_rule.conditions, context) {
            return false;
        }

        // Check pattern match
        self.matches_pattern(column_name, &override_rule.pattern, &override_rule.override_type)
    }

    /// Check if the scope matches the context
    fn matches_scope(&self, scope: &OverrideScope, context: &OverrideContext) -> bool {
        match scope {
            OverrideScope::Global => true,
            OverrideScope::DocumentType(doc_type) => doc_type == &context.document_type,
            OverrideScope::FilePattern(pattern) => {
                if let Some(file_name) = &context.file_name {
                    // Simple pattern matching - could be enhanced with regex
                    file_name.contains(pattern)
                } else {
                    false
                }
            }
            OverrideScope::User(user) => {
                if let Some(user_id) = &context.user_id {
                    user == user_id
                } else {
                    false
                }
            }
            _ => false, // Other scope types not implemented yet
        }
    }

    /// Evaluate override conditions
    fn evaluate_conditions(&self, conditions: &[OverrideCondition], context: &OverrideContext) -> bool {
        if conditions.is_empty() {
            return true;
        }

        for condition in conditions {
            let result = self.evaluate_single_condition(condition, context);
            if condition.required && !result {
                return false;
            }
        }

        true
    }

    /// Evaluate a single condition
    fn evaluate_single_condition(&self, condition: &OverrideCondition, context: &OverrideContext) -> bool {
        match condition.condition_type {
            ConditionType::DocumentType => {
                let expected = condition.value.as_str().unwrap_or("");
                match condition.operator {
                    ConditionOperator::Equals => context.document_type == expected,
                    ConditionOperator::NotEquals => context.document_type != expected,
                    _ => false,
                }
            }
            ConditionType::ColumnCount => {
                let expected = condition.value.as_u64().unwrap_or(0) as usize;
                match condition.operator {
                    ConditionOperator::Equals => context.headers.len() == expected,
                    ConditionOperator::GreaterThan => context.headers.len() > expected,
                    ConditionOperator::LessThan => context.headers.len() < expected,
                    _ => false,
                }
            }
            _ => false, // Other condition types not implemented yet
        }
    }

    /// Check if a column name matches a pattern
    fn matches_pattern(&self, column_name: &str, pattern: &OverridePattern, override_type: &OverrideType) -> bool {
        let target = if pattern.case_sensitive {
            column_name.to_string()
        } else {
            column_name.to_lowercase()
        };

        let pattern_str = if pattern.case_sensitive {
            pattern.pattern.clone()
        } else {
            pattern.pattern.to_lowercase()
        };

        match override_type {
            OverrideType::ExactMatch => target == pattern_str,
            OverrideType::Contains => target.contains(&pattern_str),
            OverrideType::StartsWith => target.starts_with(&pattern_str),
            OverrideType::EndsWith => target.ends_with(&pattern_str),
            OverrideType::RegexMatch => {
                if let Ok(regex) = Regex::new(&pattern.pattern) {
                    regex.is_match(&target)
                } else {
                    false
                }
            }
            _ => false, // Other types not implemented yet
        }
    }

    /// Detect conflicts with existing overrides
    fn detect_conflicts(&self, new_override: &MappingOverride) -> Vec<OverrideConflict> {
        let mut conflicts = Vec::new();

        for existing in &self.overrides {
            if self.has_conflict(new_override, existing) {
                conflicts.push(OverrideConflict {
                    conflicting_overrides: vec![existing.id, new_override.id],
                    conflict_type: ConflictType::PatternOverlap,
                    severity: ConflictSeverity::Medium,
                    description: "Override patterns overlap".to_string(),
                    suggested_resolution: "Adjust pattern specificity or priority".to_string(),
                });
            }
        }

        conflicts
    }

    /// Check if two overrides have a conflict
    fn has_conflict(&self, override1: &MappingOverride, override2: &MappingOverride) -> bool {
        // Simple conflict detection - could be enhanced
        override1.pattern.pattern == override2.pattern.pattern &&
        override1.scope == override2.scope &&
        override1.target_field != override2.target_field
    }

    /// Create conflicts from multiple matching overrides
    fn create_conflicts_from_matches(&self, matches: &[MappingOverride]) -> Vec<OverrideConflict> {
        if matches.len() <= 1 {
            return Vec::new();
        }

        vec![OverrideConflict {
            conflicting_overrides: matches.iter().map(|m| m.id).collect(),
            conflict_type: ConflictType::PatternOverlap,
            severity: ConflictSeverity::Medium,
            description: "Multiple overrides match the same pattern".to_string(),
            suggested_resolution: "Use priority or specificity to resolve".to_string(),
        }]
    }
}

impl OverrideContext {
    /// Create a new override context
    pub fn new(document_type: String) -> Self {
        Self {
            document_type,
            file_name: None,
            headers: Vec::new(),
            row_count: 0,
            file_size: 0,
            timestamp: chrono::Utc::now(),
            user_id: None,
            metadata: HashMap::new(),
        }
    }

    /// Set file name
    pub fn with_file_name(mut self, file_name: String) -> Self {
        self.file_name = Some(file_name);
        self
    }

    /// Set headers
    pub fn with_headers(mut self, headers: Vec<String>) -> Self {
        self.headers = headers;
        self
    }

    /// Set row count
    pub fn with_row_count(mut self, row_count: usize) -> Self {
        self.row_count = row_count;
        self
    }

    /// Set user ID
    pub fn with_user_id(mut self, user_id: String) -> Self {
        self.user_id = Some(user_id);
        self
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

    /// Resolve conflicts for a new override
    pub fn resolve_conflicts(
        &self,
        _new_override: &MappingOverride,
        conflicts: &[OverrideConflict],
    ) -> Result<(), String> {
        if conflicts.is_empty() {
            return Ok(());
        }

        match self.resolution_strategy {
            ConflictResolutionStrategy::Manual => {
                Err("Manual resolution required".to_string())
            }
            _ => Ok(()), // Auto-resolve for now
        }
    }

    /// Resolve multiple matching overrides
    pub fn resolve_multiple_matches(
        &self,
        matches: &[MappingOverride],
    ) -> Result<MappingOverride, String> {
        if matches.is_empty() {
            return Err("No matches to resolve".to_string());
        }

        if matches.len() == 1 {
            return Ok(matches[0].clone());
        }

        match self.resolution_strategy {
            ConflictResolutionStrategy::HighestPriority => {
                let highest_priority = matches.iter()
                    .max_by_key(|m| m.priority)
                    .unwrap();
                Ok(highest_priority.clone())
            }
            ConflictResolutionStrategy::MostRecent => {
                let most_recent = matches.iter()
                    .max_by_key(|m| m.created_at)
                    .unwrap();
                Ok(most_recent.clone())
            }
            _ => Err("Resolution strategy not implemented".to_string()),
        }
    }
}

impl OverrideValidator {
    /// Create a new override validator
    pub fn new() -> Self {
        Self {
            regex_cache: HashMap::new(),
            validation_rules: ValidationRuleSet::default(),
            max_cache_size: 100,
        }
    }

    /// Validate an override rule
    pub fn validate_override(&mut self, override_rule: &MappingOverride) -> Result<(), String> {
        // Validate pattern length
        if override_rule.pattern.pattern.len() > self.validation_rules.max_pattern_length {
            return Err("Pattern too long".to_string());
        }

        // Validate conditions count
        if override_rule.conditions.len() > self.validation_rules.max_conditions {
            return Err("Too many conditions".to_string());
        }

        // Validate priority range
        if override_rule.priority < self.validation_rules.min_priority ||
           override_rule.priority > self.validation_rules.max_priority {
            return Err("Priority out of range".to_string());
        }

        // Validate regex pattern if applicable
        if override_rule.override_type == OverrideType::RegexMatch {
            if let Err(e) = Regex::new(&override_rule.pattern.pattern) {
                return Err(format!("Invalid regex pattern: {}", e));
            }
        }

        Ok(())
    }
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
