// Modified: 2025-09-22

//! Main mapping override engine implementation
//!
//! This module contains the core MappingOverrideEngine that handles
//! override rule management, resolution, and conflict detection.

use super::types::*;
use super::resolver::ConflictResolver;
use super::validator::OverrideValidator;
use lru::LruCache;
use regex::Regex;
use std::collections::HashMap;

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

    /// Get current metrics
    pub fn get_metrics(&self) -> &OverrideMetrics {
        &self.metrics
    }

    /// Clear the resolution cache
    pub fn clear_cache(&mut self) {
        self.resolution_cache.clear();
    }

    /// Get all active overrides
    pub fn get_active_overrides(&self) -> Vec<&MappingOverride> {
        self.overrides.iter().filter(|o| o.active).collect()
    }

    /// Remove an override by ID
    pub fn remove_override(&mut self, id: &uuid::Uuid) -> bool {
        if let Some(pos) = self.overrides.iter().position(|o| &o.id == id) {
            self.overrides.remove(pos);
            self.resolution_cache.clear(); // Clear cache after modification
            true
        } else {
            false
        }
    }
}
