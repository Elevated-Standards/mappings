// Modified: 2025-01-22

//! Business rule validation for POA&M documents
//!
//! This module provides validation for complex business rules that involve
//! multiple fields, conditional logic, and custom validation functions.

use super::types::{
    BusinessRule, BusinessRuleResult, RuleCondition, RuleAction, LogicalOperator,
    CustomValidationRule, ValidationError, ValidationWarning
};
use crate::validation::types::ValidationSeverity;
use fedramp_core::Result;
use serde_json::Value;
use std::collections::HashMap;
use regex::Regex;
use chrono::NaiveDate;
use tracing::{info, warn};

/// Business rule validator
#[derive(Debug, Clone)]
pub struct BusinessRuleValidator {
    /// Business rules
    pub rules: Vec<BusinessRule>,
    /// Rule cache
    pub rule_cache: HashMap<String, BusinessRuleResult>,
}

impl BusinessRuleValidator {
    /// Create a new business rule validator
    pub fn new(rules: &[BusinessRule]) -> Self {
        Self {
            rules: rules.to_vec(),
            rule_cache: HashMap::new(),
        }
    }

    /// Validate all business rules against POA&M data
    pub fn validate_business_rules(
        &mut self,
        poam_data: &HashMap<String, Value>,
    ) -> Result<Vec<BusinessRuleResult>> {
        let mut results = Vec::new();

        for rule in &self.rules.clone() {
            if !rule.enabled {
                continue;
            }

            // Check cache first
            let cache_key = format!("{}_{}", rule.name, self.hash_data(poam_data));
            if let Some(cached_result) = self.rule_cache.get(&cache_key) {
                results.push(cached_result.clone());
                continue;
            }

            let result = self.evaluate_rule(rule, poam_data)?;
            
            // Cache the result
            self.rule_cache.insert(cache_key, result.clone());
            results.push(result);
        }

        Ok(results)
    }

    /// Evaluate a single business rule
    fn evaluate_rule(
        &self,
        rule: &BusinessRule,
        poam_data: &HashMap<String, Value>,
    ) -> Result<BusinessRuleResult> {
        let condition_met = self.evaluate_condition(&rule.condition, poam_data)?;
        
        let mut message = None;
        let mut action_taken = None;

        if condition_met {
            // Execute the rule action
            match &rule.action {
                RuleAction::RequireField { field } => {
                    if !poam_data.contains_key(field) || poam_data[field].is_null() {
                        message = Some(format!("Required field '{}' is missing", field));
                        action_taken = Some("require_field".to_string());
                    }
                }
                RuleAction::ValidateValue { field, validation } => {
                    if let Some(value) = poam_data.get(field) {
                        let validation_result = self.validate_field_value(value, validation)?;
                        if !validation_result {
                            message = Some(format!("Field '{}' failed validation: {}", field, validation));
                            action_taken = Some("validate_value".to_string());
                        }
                    }
                }
                RuleAction::SetDefault { field, value } => {
                    if !poam_data.contains_key(field) || poam_data[field].is_null() {
                        message = Some(format!("Set default value '{}' for field '{}'", value, field));
                        action_taken = Some("set_default".to_string());
                    }
                }
                RuleAction::GenerateWarning { message: msg } => {
                    message = Some(msg.clone());
                    action_taken = Some("generate_warning".to_string());
                }
                RuleAction::GenerateError { message: msg } => {
                    message = Some(msg.clone());
                    action_taken = Some("generate_error".to_string());
                }
            }
        }

        Ok(BusinessRuleResult {
            rule_name: rule.name.clone(),
            passed: condition_met && message.is_none(),
            message,
            action_taken,
            severity: rule.severity.clone(),
        })
    }

    /// Evaluate a rule condition
    fn evaluate_condition(
        &self,
        condition: &RuleCondition,
        poam_data: &HashMap<String, Value>,
    ) -> Result<bool> {
        match condition {
            RuleCondition::FieldEquals { field, value } => {
                if let Some(field_value) = poam_data.get(field) {
                    Ok(field_value.as_str().unwrap_or("") == value)
                } else {
                    Ok(false)
                }
            }
            RuleCondition::FieldNotEquals { field, value } => {
                if let Some(field_value) = poam_data.get(field) {
                    Ok(field_value.as_str().unwrap_or("") != value)
                } else {
                    Ok(true) // Missing field is not equal to any value
                }
            }
            RuleCondition::FieldEmpty { field } => {
                if let Some(field_value) = poam_data.get(field) {
                    Ok(field_value.is_null() || field_value.as_str().unwrap_or("").trim().is_empty())
                } else {
                    Ok(true) // Missing field is considered empty
                }
            }
            RuleCondition::FieldNotEmpty { field } => {
                if let Some(field_value) = poam_data.get(field) {
                    Ok(!field_value.is_null() && !field_value.as_str().unwrap_or("").trim().is_empty())
                } else {
                    Ok(false) // Missing field is considered empty
                }
            }
            RuleCondition::FieldMatches { field, pattern } => {
                if let Some(field_value) = poam_data.get(field) {
                    if let Some(value_str) = field_value.as_str() {
                        let regex = Regex::new(pattern).map_err(|e| {
                            fedramp_core::Error::validation(format!("Invalid regex pattern '{}': {}", pattern, e))
                        })?;
                        Ok(regex.is_match(value_str))
                    } else {
                        Ok(false)
                    }
                } else {
                    Ok(false)
                }
            }
            RuleCondition::Complex { conditions, operator } => {
                self.evaluate_complex_condition(conditions, operator, poam_data)
            }
        }
    }

    /// Evaluate a complex condition with multiple sub-conditions
    fn evaluate_complex_condition(
        &self,
        conditions: &[RuleCondition],
        operator: &LogicalOperator,
        poam_data: &HashMap<String, Value>,
    ) -> Result<bool> {
        match operator {
            LogicalOperator::And => {
                for condition in conditions {
                    if !self.evaluate_condition(condition, poam_data)? {
                        return Ok(false);
                    }
                }
                Ok(true)
            }
            LogicalOperator::Or => {
                for condition in conditions {
                    if self.evaluate_condition(condition, poam_data)? {
                        return Ok(true);
                    }
                }
                Ok(false)
            }
            LogicalOperator::Not => {
                if conditions.len() != 1 {
                    return Err(fedramp_core::Error::validation(
                        "NOT operator requires exactly one condition".to_string()
                    ));
                }
                Ok(!self.evaluate_condition(&conditions[0], poam_data)?)
            }
        }
    }

    /// Validate a field value against a validation rule
    fn validate_field_value(&self, value: &Value, validation: &str) -> Result<bool> {
        match validation {
            "required" => Ok(!value.is_null() && !value.as_str().unwrap_or("").trim().is_empty()),
            "email" => {
                if let Some(email_str) = value.as_str() {
                    let email_regex = Regex::new(r"^[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}$")
                        .map_err(|e| fedramp_core::Error::validation(format!("Email regex error: {}", e)))?;
                    Ok(email_regex.is_match(email_str))
                } else {
                    Ok(false)
                }
            }
            "date" => {
                if let Some(date_str) = value.as_str() {
                    Ok(NaiveDate::parse_from_str(date_str, "%Y-%m-%d").is_ok() ||
                       NaiveDate::parse_from_str(date_str, "%m/%d/%Y").is_ok() ||
                       NaiveDate::parse_from_str(date_str, "%d/%m/%Y").is_ok())
                } else {
                    Ok(false)
                }
            }
            "numeric" => Ok(value.is_number()),
            "positive_number" => {
                if let Some(num) = value.as_f64() {
                    Ok(num > 0.0)
                } else {
                    Ok(false)
                }
            }
            "url" => {
                if let Some(url_str) = value.as_str() {
                    let url_regex = Regex::new(r"^https?://[^\s/$.?#].[^\s]*$")
                        .map_err(|e| fedramp_core::Error::validation(format!("URL regex error: {}", e)))?;
                    Ok(url_regex.is_match(url_str))
                } else {
                    Ok(false)
                }
            }
            _ => {
                warn!("Unknown validation rule: {}", validation);
                Ok(true) // Unknown validation rules pass by default
            }
        }
    }

    /// Generate a hash for POA&M data for caching
    fn hash_data(&self, data: &HashMap<String, Value>) -> u64 {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let mut hasher = DefaultHasher::new();
        
        // Sort keys for consistent hashing
        let mut sorted_keys: Vec<_> = data.keys().collect();
        sorted_keys.sort();
        
        for key in sorted_keys {
            key.hash(&mut hasher);
            if let Some(value) = data.get(key) {
                value.to_string().hash(&mut hasher);
            }
        }
        
        hasher.finish()
    }

    /// Add a new business rule
    pub fn add_rule(&mut self, rule: BusinessRule) {
        self.rules.push(rule);
        self.rule_cache.clear(); // Clear cache when rules change
    }

    /// Remove a business rule by name
    pub fn remove_rule(&mut self, rule_name: &str) -> bool {
        let initial_len = self.rules.len();
        self.rules.retain(|rule| rule.name != rule_name);
        let removed = self.rules.len() < initial_len;
        
        if removed {
            self.rule_cache.clear(); // Clear cache when rules change
        }
        
        removed
    }

    /// Enable or disable a rule
    pub fn set_rule_enabled(&mut self, rule_name: &str, enabled: bool) -> bool {
        for rule in &mut self.rules {
            if rule.name == rule_name {
                rule.enabled = enabled;
                self.rule_cache.clear(); // Clear cache when rules change
                return true;
            }
        }
        false
    }

    /// Get all rule names
    pub fn get_rule_names(&self) -> Vec<String> {
        self.rules.iter().map(|rule| rule.name.clone()).collect()
    }

    /// Clear the rule cache
    pub fn clear_cache(&mut self) {
        self.rule_cache.clear();
    }

    /// Get cache statistics
    pub fn get_cache_stats(&self) -> (usize, usize) {
        (self.rule_cache.len(), self.rules.len())
    }
}
