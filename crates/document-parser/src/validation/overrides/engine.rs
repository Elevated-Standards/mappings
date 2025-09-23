// Modified: 2025-09-22

//! Main mapping override engine implementation
//!
//! This module contains the core MappingOverrideEngine that handles
//! override rule management, resolution, and pattern matching.

use super::types::*;
use super::resolver::ConflictResolver;
use super::validator::OverrideValidator;
use crate::{Error, Result};
use lru::LruCache;
use std::num::NonZeroUsize;
use chrono::Utc;
use tracing::{info, warn, debug};
use uuid::Uuid;

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

    /// Clear the override cache
    pub fn clear_cache(&mut self) {
        self.override_cache.clear();
    }

    /// Get all active overrides
    pub fn get_active_overrides(&self) -> Vec<&MappingOverride> {
        self.overrides.iter().filter(|o| o.active).collect()
    }

    /// Get override by ID
    pub fn get_override(&self, id: &Uuid) -> Option<&MappingOverride> {
        self.overrides.iter().find(|o| o.id == *id)
    }

    /// Update an existing override
    pub fn update_override(&mut self, updated_override: MappingOverride) -> Result<bool> {
        if let Some(existing) = self.overrides.iter_mut().find(|o| o.id == updated_override.id) {
            // Validate the updated override
            self.validator.validate_override(&updated_override)?;
            
            *existing = updated_override;
            
            // Re-sort by priority
            self.overrides.sort_by(|a, b| b.priority.cmp(&a.priority));
            
            // Clear cache since rules have changed
            self.override_cache.clear();
            
            Ok(true)
        } else {
            Ok(false)
        }
    }
}

impl Default for MappingOverrideEngine {
    fn default() -> Self {
        Self::new()
    }
}
