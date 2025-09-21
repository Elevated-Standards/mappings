//! Mapping override system for custom column mappings

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::Duration;
use uuid::Uuid;
use chrono::{DateTime, Utc};
use regex::Regex;
use lru::LruCache;
use std::num::NonZeroUsize;
use tracing::{info, warn, debug};
use crate::{Error, Result};
use super::rules::ValidationRuleSet;

/// Mapping override engine for managing custom column mappings
#[derive(Debug)]
pub struct MappingOverrideEngine {
    /// Active override rules
    overrides: Vec<MappingOverride>,
    /// Conflict resolver for handling rule conflicts
    conflict_resolver: ConflictResolver,
    /// Validator for override rules
    validator: OverrideValidator,
    /// LRU cache for override resolution results
    override_cache: LruCache<String, OverrideResolutionResult>,
    /// Performance metrics
    metrics: OverrideMetrics,
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

/// Conflict resolver for handling overlapping override rules
#[derive(Debug, Clone)]
pub struct ConflictResolver {
    /// Strategy for resolving conflicts
    resolution_strategy: ConflictResolutionStrategy,
    /// Maximum number of conflicts to report
    max_conflicts_reported: usize,
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

/// Validator for override rules
#[derive(Debug, Clone)]
pub struct OverrideValidator {
    /// Compiled regex patterns cache
    regex_cache: HashMap<String, Regex>,
    /// Validation rule set
    validation_rules: ValidationRuleSet,
    /// Maximum cache size
    max_cache_size: usize,
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

impl MappingOverrideEngine {
    /// Create a new mapping override engine
    pub fn new() -> Self {
        Self {
            overrides: Vec::new(),
            conflict_resolver: ConflictResolver::new(),
            validator: OverrideValidator::new(),
            override_cache: LruCache::new(NonZeroUsize::new(1000).unwrap()),
            metrics: OverrideMetrics::default(),
        }
    }

    /// Add a new override rule
    pub fn add_override(&mut self, override_rule: MappingOverride) -> Result<()> {
        // Validate the override rule
        self.validator.validate_override(&override_rule)?;

        // Check for conflicts
        let conflicts = self.detect_conflicts(&override_rule)?;
        if !conflicts.is_empty() {
            warn!("Adding override '{}' with {} conflicts", override_rule.name, conflicts.len());
        }

        // Add the override
        self.overrides.push(override_rule.clone());
        
        // Sort by priority (highest first)
        self.overrides.sort_by(|a, b| b.priority.cmp(&a.priority));
        
        // Clear cache since rules have changed
        self.override_cache.clear();
        
        info!("Added override rule: {}", override_rule.name);
        Ok(())
    }

    /// Remove an override rule by ID
    pub fn remove_override(&mut self, override_id: &Uuid) -> Result<bool> {
        let initial_len = self.overrides.len();
        self.overrides.retain(|o| o.id != *override_id);
        
        let removed = self.overrides.len() < initial_len;
        if removed {
            self.override_cache.clear();
            info!("Removed override rule: {}", override_id);
        }
        
        Ok(removed)
    }

    /// Resolve mapping using override rules
    pub fn resolve_mapping(
        &mut self,
        source_column: &str,
        document_type: &str,
        context: &OverrideContext,
    ) -> Result<OverrideResolutionResult> {
        let start_time = std::time::Instant::now();
        
        // Check cache first
        let cache_key = format!("{}:{}:{}", source_column, document_type, context.document_type);
        if let Some(cached_result) = self.override_cache.get(&cache_key) {
            let mut result = cached_result.clone();
            result.from_cache = true;
            return Ok(result);
        }

        let mut matching_overrides = Vec::new();
        let mut conflicts = Vec::new();

        // Find all matching overrides
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
        let (applied_override, alternatives) = if matching_overrides.is_empty() {
            (None, Vec::new())
        } else if matching_overrides.len() == 1 {
            let (override_rule, _confidence) = matching_overrides.into_iter().next().unwrap();
            (Some(override_rule), Vec::new())
        } else {
            // Multiple matches - resolve conflicts
            let resolved = self.conflict_resolver.resolve_conflicts(matching_overrides)?;
            conflicts.extend(resolved.conflicts);
            (resolved.selected_override, resolved.alternatives)
        };

        let resolution_time = start_time.elapsed();
        
        let result = OverrideResolutionResult {
            override_applied: applied_override.is_some(),
            target_field: applied_override.as_ref().map(|o| o.target_field.clone()),
            confidence: if applied_override.is_some() { 1.0 } else { 0.0 },
            applied_override,
            alternatives,
            conflicts,
            resolution_time,
            from_cache: false,
        };

        // Cache the result
        self.override_cache.put(cache_key, result.clone());

        // Update metrics
        self.metrics.total_applications += 1;
        if result.override_applied {
            self.metrics.successful_matches += 1;
        }
        if !result.conflicts.is_empty() {
            self.metrics.conflicts_detected += 1;
        }
        
        // Update average resolution time
        let resolution_time_us = resolution_time.as_micros() as f64;
        self.metrics.avg_resolution_time_us = 
            (self.metrics.avg_resolution_time_us * 0.9) + (resolution_time_us * 0.1);
        
        self.metrics.last_updated = Utc::now();

        Ok(result)
    }

    /// Check if pattern matches source column
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
                
                if matches {
                    Ok(Some(1.0))
                } else {
                    Ok(None)
                }
            }
            OverrideType::ContainsMatch => {
                let matches = if pattern.case_sensitive {
                    source_column.contains(&pattern.pattern)
                } else {
                    source_column.to_lowercase().contains(&pattern.pattern.to_lowercase())
                };
                
                if matches {
                    Ok(Some(0.8))
                } else {
                    Ok(None)
                }
            }
            OverrideType::RegexPattern => {
                // TODO: Implement regex matching with caching
                Ok(Some(0.9))
            }
            OverrideType::FuzzyMatch => {
                // TODO: Implement fuzzy matching
                Ok(Some(0.7))
            }
            _ => {
                debug!("Pattern matching not implemented for rule type: {:?}", rule_type);
                Ok(None)
            }
        }
    }

    /// Check if scope matches context
    fn check_scope_match(&self, scope: &OverrideScope, context: &OverrideContext) -> Result<bool> {
        match scope {
            OverrideScope::Global => Ok(true),
            OverrideScope::DocumentType(doc_type) => Ok(context.document_type == *doc_type),
            OverrideScope::Organization(org) => Ok(context.organization.as_ref() == Some(org)),
            OverrideScope::User(user) => Ok(context.user_id.as_ref() == Some(user)),
            OverrideScope::Session(session) => Ok(context.session_id.as_ref() == Some(session)),
            OverrideScope::Project(project) => Ok(context.project_id.as_ref() == Some(project)),
        }
    }

    /// Evaluate conditions for override rule
    fn evaluate_conditions(
        &self,
        conditions: &[OverrideCondition],
        _source_column: &str,
        document_type: &str,
        context: &OverrideContext,
    ) -> Result<bool> {
        if conditions.is_empty() {
            return Ok(true);
        }

        let mut required_conditions_met = true;
        let mut optional_conditions_met = false;

        for condition in conditions {
            let condition_result = self.evaluate_single_condition(condition, _source_column, document_type, context)?;
            
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
        match condition.condition_type {
            ConditionType::DocumentType => {
                let expected = condition.value.as_str().unwrap_or("");
                match condition.operator {
                    ConditionOperator::Equals => Ok(document_type == expected),
                    ConditionOperator::NotEquals => Ok(document_type != expected),
                    _ => Ok(false),
                }
            }
            ConditionType::FileName => {
                if let Some(file_name) = &context.file_name {
                    let expected = condition.value.as_str().unwrap_or("");
                    match condition.operator {
                        ConditionOperator::Contains => Ok(file_name.contains(expected)),
                        ConditionOperator::NotContains => Ok(!file_name.contains(expected)),
                        _ => Ok(false),
                    }
                } else {
                    Ok(false)
                }
            }
            _ => {
                debug!("Condition evaluation not implemented for type: {:?}", condition.condition_type);
                Ok(true) // Default to true for unimplemented conditions
            }
        }
    }

    /// Detect conflicts with existing override rules
    fn detect_conflicts(&self, new_override: &MappingOverride) -> Result<Vec<OverrideConflict>> {
        let mut conflicts = Vec::new();

        for existing in &self.overrides {
            if existing.id == new_override.id {
                continue;
            }

            // Check for priority ties with overlapping patterns
            if existing.priority == new_override.priority 
                && existing.scope == new_override.scope 
                && self.patterns_overlap(&existing.pattern, &new_override.pattern)? {
                conflicts.push(OverrideConflict {
                    conflicting_overrides: vec![existing.id, new_override.id],
                    conflict_type: ConflictType::PriorityTie,
                    severity: ConflictSeverity::Medium,
                    description: format!(
                        "Override '{}' has the same priority ({}) as existing override '{}'",
                        new_override.name, new_override.priority, existing.name
                    ),
                    suggested_resolution: Some("Adjust priority levels to resolve conflict".to_string()),
                    resolution_applied: None,
                });
            }
        }

        Ok(conflicts)
    }

    /// Check if two patterns overlap
    fn patterns_overlap(&self, _pattern1: &OverridePattern, _pattern2: &OverridePattern) -> Result<bool> {
        // TODO: Implement pattern overlap detection
        Ok(false)
    }

    /// Get current metrics
    pub fn get_metrics(&self) -> &OverrideMetrics {
        &self.metrics
    }
}

impl OverrideContext {
    /// Create a new override context
    pub fn new(document_type: String) -> Self {
        Self {
            document_type,
            file_name: None,
            user_id: None,
            organization: None,
            session_id: None,
            project_id: None,
            metadata: HashMap::new(),
            column_count: None,
            sample_data: None,
        }
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
    ) -> Result<ConflictResolution> {
        match self.resolution_strategy {
            ConflictResolutionStrategy::HighestPriority => {
                let mut sorted = matching_overrides;
                sorted.sort_by(|a, b| b.0.priority.cmp(&a.0.priority));
                
                let selected = sorted.first().map(|(o, _)| o.clone());
                let alternatives = sorted.into_iter().skip(1).map(|(o, _)| o).collect();
                
                Ok(ConflictResolution {
                    selected_override: selected,
                    alternatives,
                    conflicts: Vec::new(),
                })
            }
            _ => {
                // TODO: Implement other resolution strategies
                Ok(ConflictResolution {
                    selected_override: None,
                    alternatives: matching_overrides.into_iter().map(|(o, _)| o).collect(),
                    conflicts: Vec::new(),
                })
            }
        }
    }
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

impl OverrideValidator {
    /// Create a new override validator
    pub fn new() -> Self {
        Self {
            regex_cache: HashMap::new(),
            validation_rules: ValidationRuleSet::default(),
            max_cache_size: 1000,
        }
    }

    /// Validate an override rule
    pub fn validate_override(&self, _override_rule: &MappingOverride) -> Result<()> {
        // TODO: Implement comprehensive override validation
        Ok(())
    }
}

impl Default for MappingOverrideEngine {
    fn default() -> Self {
        Self::new()
    }
}

impl Default for ConflictResolver {
    fn default() -> Self {
        Self::new()
    }
}

impl Default for OverrideValidator {
    fn default() -> Self {
        Self::new()
    }
}
