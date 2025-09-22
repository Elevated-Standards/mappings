// Modified: 2025-09-22

//! Override validation functionality
//!
//! This module provides functionality for validating override rules
//! to ensure they are well-formed and safe to use.

use super::types::*;
use regex::Regex;
use std::collections::HashMap;

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

impl OverrideValidator {
    /// Create a new override validator
    pub fn new() -> Self {
        Self {
            regex_cache: HashMap::new(),
            validation_rules: ValidationRuleSet::default(),
            max_cache_size: 100,
        }
    }

    /// Create a validator with custom rules
    pub fn with_rules(validation_rules: ValidationRuleSet) -> Self {
        Self {
            regex_cache: HashMap::new(),
            validation_rules,
            max_cache_size: 100,
        }
    }

    /// Set validation rules
    pub fn set_validation_rules(&mut self, rules: ValidationRuleSet) {
        self.validation_rules = rules;
    }

    /// Set maximum cache size
    pub fn set_max_cache_size(&mut self, size: usize) {
        self.max_cache_size = size;
        // Trim cache if it's too large
        if self.regex_cache.len() > size {
            self.regex_cache.clear();
        }
    }

    /// Validate an override rule
    pub fn validate_override(&mut self, override_rule: &MappingOverride) -> Result<(), String> {
        // Validate basic fields
        self.validate_basic_fields(override_rule)?;
        
        // Validate pattern
        self.validate_pattern(&override_rule.pattern, &override_rule.override_type)?;
        
        // Validate conditions
        self.validate_conditions(&override_rule.conditions)?;
        
        // Validate scope
        self.validate_scope(&override_rule.scope)?;
        
        // Validate position constraints
        if let Some(constraints) = &override_rule.position_constraints {
            self.validate_position_constraints(constraints)?;
        }
        
        // Validate priority range
        self.validate_priority(override_rule.priority)?;

        Ok(())
    }

    /// Validate basic fields
    fn validate_basic_fields(&self, override_rule: &MappingOverride) -> Result<(), String> {
        if override_rule.name.is_empty() {
            return Err("Override name cannot be empty".to_string());
        }

        if override_rule.name.len() > 100 {
            return Err("Override name too long (max 100 characters)".to_string());
        }

        if override_rule.target_field.is_empty() {
            return Err("Target field cannot be empty".to_string());
        }

        if override_rule.target_field.len() > 200 {
            return Err("Target field name too long (max 200 characters)".to_string());
        }

        if override_rule.created_by.is_empty() {
            return Err("Created by field cannot be empty".to_string());
        }

        Ok(())
    }

    /// Validate pattern
    fn validate_pattern(&mut self, pattern: &OverridePattern, override_type: &OverrideType) -> Result<(), String> {
        // Validate pattern length
        if pattern.pattern.len() > self.validation_rules.max_pattern_length {
            return Err(format!("Pattern too long (max {} characters)", 
                self.validation_rules.max_pattern_length));
        }

        if pattern.pattern.is_empty() {
            return Err("Pattern cannot be empty".to_string());
        }

        // Check forbidden patterns
        for forbidden in &self.validation_rules.forbidden_patterns {
            if pattern.pattern == *forbidden {
                return Err(format!("Pattern '{}' is forbidden", forbidden));
            }
        }

        // Validate regex pattern if applicable
        if *override_type == OverrideType::RegexMatch {
            self.validate_regex_pattern(&pattern.pattern)?;
        }

        // Validate fuzzy matching threshold
        if *override_type == OverrideType::FuzzyMatch {
            if let Some(threshold) = pattern.similarity_threshold {
                if threshold < 0.0 || threshold > 1.0 {
                    return Err("Similarity threshold must be between 0.0 and 1.0".to_string());
                }
            } else {
                return Err("Fuzzy match requires similarity threshold".to_string());
            }
        }

        Ok(())
    }

    /// Validate regex pattern
    fn validate_regex_pattern(&mut self, pattern: &str) -> Result<(), String> {
        // Check cache first
        if self.regex_cache.contains_key(pattern) {
            return Ok(());
        }

        // Try to compile the regex
        match Regex::new(pattern) {
            Ok(regex) => {
                // Cache the compiled regex if cache isn't full
                if self.regex_cache.len() < self.max_cache_size {
                    self.regex_cache.insert(pattern.to_string(), regex);
                }
                Ok(())
            }
            Err(e) => Err(format!("Invalid regex pattern: {}", e)),
        }
    }

    /// Validate conditions
    fn validate_conditions(&self, conditions: &[OverrideCondition]) -> Result<(), String> {
        if conditions.len() > self.validation_rules.max_conditions {
            return Err(format!("Too many conditions (max {})", 
                self.validation_rules.max_conditions));
        }

        for (i, condition) in conditions.iter().enumerate() {
            self.validate_single_condition(condition)
                .map_err(|e| format!("Condition {}: {}", i + 1, e))?;
        }

        Ok(())
    }

    /// Validate a single condition
    fn validate_single_condition(&self, condition: &OverrideCondition) -> Result<(), String> {
        if condition.field.is_empty() {
            return Err("Condition field cannot be empty".to_string());
        }

        // Validate condition type and operator compatibility
        match (&condition.condition_type, &condition.operator) {
            (ConditionType::ColumnCount | ConditionType::RowCount | ConditionType::FileSize, 
             ConditionOperator::Contains | ConditionOperator::NotContains | 
             ConditionOperator::StartsWith | ConditionOperator::EndsWith) => {
                return Err("Numeric conditions cannot use string operators".to_string());
            }
            (ConditionType::DocumentType | ConditionType::FileName | 
             ConditionType::HeaderContent | ConditionType::CellContent,
             ConditionOperator::GreaterThan | ConditionOperator::LessThan |
             ConditionOperator::GreaterThanOrEqual | ConditionOperator::LessThanOrEqual) => {
                return Err("String conditions cannot use numeric operators".to_string());
            }
            _ => {} // Valid combination
        }

        // Validate value type based on condition type
        match condition.condition_type {
            ConditionType::ColumnCount | ConditionType::RowCount | ConditionType::FileSize => {
                if !condition.value.is_number() {
                    return Err("Numeric condition requires numeric value".to_string());
                }
            }
            ConditionType::DocumentType | ConditionType::FileName | 
            ConditionType::HeaderContent | ConditionType::CellContent => {
                if !condition.value.is_string() {
                    return Err("String condition requires string value".to_string());
                }
            }
            _ => {} // Other types not strictly validated yet
        }

        Ok(())
    }

    /// Validate scope
    fn validate_scope(&self, scope: &OverrideScope) -> Result<(), String> {
        match scope {
            OverrideScope::Global => Ok(()),
            OverrideScope::DocumentType(doc_type) => {
                if doc_type.is_empty() {
                    Err("Document type cannot be empty".to_string())
                } else {
                    Ok(())
                }
            }
            OverrideScope::FilePattern(pattern) => {
                if pattern.is_empty() {
                    Err("File pattern cannot be empty".to_string())
                } else {
                    Ok(())
                }
            }
            OverrideScope::User(user) => {
                if user.is_empty() {
                    Err("User cannot be empty".to_string())
                } else {
                    Ok(())
                }
            }
            OverrideScope::Organization(org) => {
                if org.is_empty() {
                    Err("Organization cannot be empty".to_string())
                } else {
                    Ok(())
                }
            }
            OverrideScope::Project(project) => {
                if project.is_empty() {
                    Err("Project cannot be empty".to_string())
                } else {
                    Ok(())
                }
            }
        }
    }

    /// Validate position constraints
    fn validate_position_constraints(&self, constraints: &PositionConstraints) -> Result<(), String> {
        if let (Some(min), Some(max)) = (constraints.min_index, constraints.max_index) {
            if min > max {
                return Err("Minimum index cannot be greater than maximum index".to_string());
            }
        }

        if let Some(indices) = &constraints.specific_indices {
            if indices.is_empty() {
                return Err("Specific indices cannot be empty if provided".to_string());
            }
            
            // Check for duplicates
            let mut sorted_indices = indices.clone();
            sorted_indices.sort_unstable();
            sorted_indices.dedup();
            if sorted_indices.len() != indices.len() {
                return Err("Specific indices cannot contain duplicates".to_string());
            }
        }

        Ok(())
    }

    /// Validate priority
    fn validate_priority(&self, priority: i32) -> Result<(), String> {
        if priority < self.validation_rules.min_priority ||
           priority > self.validation_rules.max_priority {
            return Err(format!("Priority {} out of range ({} to {})", 
                priority, 
                self.validation_rules.min_priority, 
                self.validation_rules.max_priority));
        }
        Ok(())
    }

    /// Clear the regex cache
    pub fn clear_cache(&mut self) {
        self.regex_cache.clear();
    }

    /// Get cache statistics
    pub fn get_cache_stats(&self) -> CacheStats {
        CacheStats {
            size: self.regex_cache.len(),
            max_size: self.max_cache_size,
            hit_rate: 0.0, // Would need to track hits/misses to calculate
        }
    }

    /// Get validation rules
    pub fn get_validation_rules(&self) -> &ValidationRuleSet {
        &self.validation_rules
    }

    /// Validate multiple overrides for conflicts
    pub fn validate_override_set(&mut self, overrides: &[MappingOverride]) -> Result<(), Vec<String>> {
        let mut errors = Vec::new();

        // Validate each override individually
        for (i, override_rule) in overrides.iter().enumerate() {
            if let Err(e) = self.validate_override(override_rule) {
                errors.push(format!("Override {}: {}", i + 1, e));
            }
        }

        // Check for duplicate IDs
        let mut ids = std::collections::HashSet::new();
        for (i, override_rule) in overrides.iter().enumerate() {
            if !ids.insert(override_rule.id) {
                errors.push(format!("Override {}: Duplicate ID {}", i + 1, override_rule.id));
            }
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }
}

/// Cache statistics
#[derive(Debug, Clone)]
pub struct CacheStats {
    /// Current cache size
    pub size: usize,
    /// Maximum cache size
    pub max_size: usize,
    /// Cache hit rate (0.0 to 1.0)
    pub hit_rate: f64,
}

impl Default for OverrideValidator {
    fn default() -> Self {
        Self::new()
    }
}
