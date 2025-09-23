// Modified: 2025-09-22

//! Override validation functionality
//!
//! This module provides comprehensive validation for override rules,
//! ensuring they are well-formed, safe, and follow best practices.

use super::types::*;
use crate::{Error, Result};
use regex::Regex;
use std::collections::HashMap;
use super::super::rules::ValidationRuleSet;
use tracing::{debug, warn};

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

impl OverrideValidator {
    /// Create a new override validator
    pub fn new() -> Self {
        Self {
            regex_cache: HashMap::new(),
            validation_rules: ValidationRuleSet::default(),
            max_cache_size: 1000,
        }
    }

    /// Create a validator with custom validation rules
    pub fn with_rules(validation_rules: ValidationRuleSet) -> Self {
        Self {
            regex_cache: HashMap::new(),
            validation_rules,
            max_cache_size: 1000,
        }
    }

    /// Set the maximum cache size
    pub fn set_max_cache_size(&mut self, max_size: usize) {
        self.max_cache_size = max_size;
        if self.regex_cache.len() > max_size {
            self.regex_cache.clear();
        }
    }

    /// Validate an override rule
    pub fn validate_override(&mut self, override_rule: &MappingOverride) -> Result<()> {
        // Basic field validation
        self.validate_basic_fields(override_rule)?;
        
        // Pattern validation
        self.validate_pattern(&override_rule.pattern, &override_rule.rule_type)?;
        
        // Condition validation
        self.validate_conditions(&override_rule.conditions)?;
        
        // Scope validation
        self.validate_scope(&override_rule.scope)?;
        
        // Business rule validation
        self.validate_business_rules(override_rule)?;

        debug!("Override '{}' passed validation", override_rule.name);
        Ok(())
    }

    /// Validate basic fields of an override rule
    fn validate_basic_fields(&self, override_rule: &MappingOverride) -> Result<()> {
        // Name validation
        if override_rule.name.is_empty() {
            return Err(Error::validation("Override name cannot be empty"));
        }
        if override_rule.name.len() > 255 {
            return Err(Error::validation("Override name too long (max 255 characters)"));
        }

        // Description validation
        if override_rule.description.is_empty() {
            return Err(Error::validation("Override description cannot be empty"));
        }
        if override_rule.description.len() > 1000 {
            return Err(Error::validation("Override description too long (max 1000 characters)"));
        }

        // Target field validation
        if override_rule.target_field.is_empty() {
            return Err(Error::validation("Target field cannot be empty"));
        }
        if override_rule.target_field.len() > 255 {
            return Err(Error::validation("Target field name too long (max 255 characters)"));
        }

        // Priority validation
        if override_rule.priority < -1000 || override_rule.priority > 1000 {
            return Err(Error::validation("Priority must be between -1000 and 1000"));
        }

        // Created by validation
        if override_rule.created_by.is_empty() {
            return Err(Error::validation("Created by field cannot be empty"));
        }

        // Version validation
        if override_rule.version == 0 {
            return Err(Error::validation("Version must be greater than 0"));
        }

        // Tags validation
        for tag in &override_rule.tags {
            if tag.is_empty() {
                return Err(Error::validation("Tags cannot be empty"));
            }
            if tag.len() > 50 {
                return Err(Error::validation("Tag too long (max 50 characters)"));
            }
        }

        Ok(())
    }

    /// Validate pattern configuration
    fn validate_pattern(&mut self, pattern: &OverridePattern, rule_type: &OverrideType) -> Result<()> {
        // Pattern string validation
        if pattern.pattern.is_empty() {
            return Err(Error::validation("Pattern cannot be empty"));
        }
        if pattern.pattern.len() > 1000 {
            return Err(Error::validation("Pattern too long (max 1000 characters)"));
        }

        // Rule type specific validation
        match rule_type {
            OverrideType::RegexPattern => {
                self.validate_regex_pattern(pattern)?;
            }
            OverrideType::FuzzyMatch => {
                self.validate_fuzzy_pattern(pattern)?;
            }
            OverrideType::PositionalMatch => {
                self.validate_positional_pattern(pattern)?;
            }
            OverrideType::ExactMatch | OverrideType::ContainsMatch | OverrideType::PrefixSuffixMatch => {
                // Basic string patterns - no special validation needed
            }
            OverrideType::ConditionalMatch => {
                // Conditional patterns may have special syntax
                self.validate_conditional_pattern(pattern)?;
            }
        }

        Ok(())
    }

    /// Validate regex pattern
    fn validate_regex_pattern(&mut self, pattern: &OverridePattern) -> Result<()> {
        // Try to compile the regex
        match self.get_or_compile_regex(&pattern.pattern) {
            Ok(_) => Ok(()),
            Err(e) => Err(Error::validation(format!("Invalid regex pattern: {}", e))),
        }
    }

    /// Validate fuzzy matching pattern
    fn validate_fuzzy_pattern(&self, pattern: &OverridePattern) -> Result<()> {
        if let Some(threshold) = pattern.fuzzy_threshold {
            if threshold < 0.0 || threshold > 1.0 {
                return Err(Error::validation("Fuzzy threshold must be between 0.0 and 1.0"));
            }
        } else {
            return Err(Error::validation("Fuzzy threshold is required for fuzzy matching"));
        }

        Ok(())
    }

    /// Validate positional pattern
    fn validate_positional_pattern(&self, pattern: &OverridePattern) -> Result<()> {
        if let Some(ref constraints) = pattern.position_constraints {
            // Validate position constraints
            if let (Some(min), Some(max)) = (constraints.min_index, constraints.max_index) {
                if min > max {
                    return Err(Error::validation("Minimum index cannot be greater than maximum index"));
                }
            }

            if let Some(exact) = constraints.exact_index {
                if exact > 10000 {
                    return Err(Error::validation("Exact index seems unreasonably large (>10000)"));
                }
            }

            if let Some(ref relative) = constraints.relative_position {
                let valid_positions = ["first", "last", "second", "third", "fourth", "fifth"];
                if !valid_positions.contains(&relative.as_str()) {
                    return Err(Error::validation(format!("Invalid relative position: {}", relative)));
                }
            }
        } else {
            return Err(Error::validation("Position constraints are required for positional matching"));
        }

        Ok(())
    }

    /// Validate conditional pattern
    fn validate_conditional_pattern(&self, _pattern: &OverridePattern) -> Result<()> {
        // TODO: Implement conditional pattern validation
        Ok(())
    }

    /// Validate conditions
    fn validate_conditions(&self, conditions: &[OverrideCondition]) -> Result<()> {
        for condition in conditions {
            self.validate_single_condition(condition)?;
        }

        // Check for conflicting conditions
        self.validate_condition_consistency(conditions)?;

        Ok(())
    }

    /// Validate a single condition
    fn validate_single_condition(&self, condition: &OverrideCondition) -> Result<()> {
        // Field validation
        if condition.field.is_empty() {
            return Err(Error::validation("Condition field cannot be empty"));
        }

        // Value validation based on condition type
        match condition.condition_type {
            ConditionType::DocumentType => {
                if !condition.value.is_string() {
                    return Err(Error::validation("Document type condition value must be a string"));
                }
            }
            ConditionType::FileName => {
                if !condition.value.is_string() {
                    return Err(Error::validation("File name condition value must be a string"));
                }
            }
            ConditionType::ColumnCount => {
                if !condition.value.is_number() {
                    return Err(Error::validation("Column count condition value must be a number"));
                }
                if let Some(count) = condition.value.as_u64() {
                    if count == 0 || count > 10000 {
                        return Err(Error::validation("Column count must be between 1 and 10000"));
                    }
                }
            }
            ConditionType::UserRole | ConditionType::Organization => {
                if !condition.value.is_string() {
                    return Err(Error::validation("User role/organization condition value must be a string"));
                }
            }
            ConditionType::DataSample => {
                // Data sample can be various types
            }
            ConditionType::CustomMetadata => {
                // Custom metadata can be various types
            }
        }

        // Operator validation
        self.validate_operator_compatibility(&condition.condition_type, &condition.operator)?;

        Ok(())
    }

    /// Validate operator compatibility with condition type
    fn validate_operator_compatibility(&self, condition_type: &ConditionType, operator: &ConditionOperator) -> Result<()> {
        match condition_type {
            ConditionType::ColumnCount => {
                match operator {
                    ConditionOperator::Equals | ConditionOperator::NotEquals |
                    ConditionOperator::GreaterThan | ConditionOperator::LessThan |
                    ConditionOperator::GreaterThanOrEqual | ConditionOperator::LessThanOrEqual => Ok(()),
                    _ => Err(Error::validation("Invalid operator for column count condition")),
                }
            }
            ConditionType::DocumentType | ConditionType::FileName |
            ConditionType::UserRole | ConditionType::Organization => {
                match operator {
                    ConditionOperator::Equals | ConditionOperator::NotEquals |
                    ConditionOperator::Contains | ConditionOperator::NotContains |
                    ConditionOperator::Matches | ConditionOperator::NotMatches |
                    ConditionOperator::In | ConditionOperator::NotIn => Ok(()),
                    _ => Err(Error::validation("Invalid operator for string condition")),
                }
            }
            _ => Ok(()), // Allow all operators for other condition types
        }
    }

    /// Validate condition consistency
    fn validate_condition_consistency(&self, conditions: &[OverrideCondition]) -> Result<()> {
        // Check for contradictory conditions
        for i in 0..conditions.len() {
            for j in (i + 1)..conditions.len() {
                let cond1 = &conditions[i];
                let cond2 = &conditions[j];

                if cond1.field == cond2.field && cond1.condition_type == cond2.condition_type {
                    // Check for contradictory operators
                    if self.are_operators_contradictory(&cond1.operator, &cond2.operator, &cond1.value, &cond2.value) {
                        warn!("Potentially contradictory conditions detected for field '{}'", cond1.field);
                    }
                }
            }
        }

        Ok(())
    }

    /// Check if two operators are contradictory
    fn are_operators_contradictory(
        &self,
        op1: &ConditionOperator,
        op2: &ConditionOperator,
        val1: &serde_json::Value,
        val2: &serde_json::Value,
    ) -> bool {
        match (op1, op2) {
            (ConditionOperator::Equals, ConditionOperator::NotEquals) |
            (ConditionOperator::NotEquals, ConditionOperator::Equals) => val1 == val2,
            (ConditionOperator::Contains, ConditionOperator::NotContains) |
            (ConditionOperator::NotContains, ConditionOperator::Contains) => val1 == val2,
            _ => false,
        }
    }

    /// Validate scope
    fn validate_scope(&self, scope: &OverrideScope) -> Result<()> {
        match scope {
            OverrideScope::Global => Ok(()),
            OverrideScope::DocumentType(doc_type) => {
                if doc_type.is_empty() {
                    Err(Error::validation("Document type scope cannot be empty"))
                } else {
                    Ok(())
                }
            }
            OverrideScope::Organization(org) => {
                if org.is_empty() {
                    Err(Error::validation("Organization scope cannot be empty"))
                } else {
                    Ok(())
                }
            }
            OverrideScope::User(user) => {
                if user.is_empty() {
                    Err(Error::validation("User scope cannot be empty"))
                } else {
                    Ok(())
                }
            }
            OverrideScope::Session(session) => {
                if session.is_empty() {
                    Err(Error::validation("Session scope cannot be empty"))
                } else {
                    Ok(())
                }
            }
            OverrideScope::Project(project) => {
                if project.is_empty() {
                    Err(Error::validation("Project scope cannot be empty"))
                } else {
                    Ok(())
                }
            }
        }
    }

    /// Validate business rules
    fn validate_business_rules(&self, override_rule: &MappingOverride) -> Result<()> {
        // Check if target field is a known valid field
        // This would typically check against a schema or known field list
        
        // Validate that active overrides have reasonable priorities
        if override_rule.active && override_rule.priority < -100 {
            warn!("Active override '{}' has very low priority ({})", override_rule.name, override_rule.priority);
        }

        // Validate creation and modification timestamps
        if override_rule.modified_at < override_rule.created_at {
            return Err(Error::validation("Modified date cannot be before created date"));
        }

        Ok(())
    }

    /// Get or compile a regex pattern
    fn get_or_compile_regex(&mut self, pattern: &str) -> Result<&Regex> {
        if !self.regex_cache.contains_key(pattern) {
            // Check cache size limit
            if self.regex_cache.len() >= self.max_cache_size {
                self.regex_cache.clear();
            }

            let regex = Regex::new(pattern)
                .map_err(|e| Error::validation(format!("Invalid regex: {}", e)))?;
            self.regex_cache.insert(pattern.to_string(), regex);
        }

        Ok(self.regex_cache.get(pattern).unwrap())
    }

    /// Clear the regex cache
    pub fn clear_cache(&mut self) {
        self.regex_cache.clear();
    }

    /// Get cache statistics
    pub fn get_cache_stats(&self) -> (usize, usize) {
        (self.regex_cache.len(), self.max_cache_size)
    }
}

impl Default for OverrideValidator {
    fn default() -> Self {
        Self::new()
    }
}
