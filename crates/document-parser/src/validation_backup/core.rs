// Modified: 2025-09-22

//! Core validation implementation
//!
//! This module contains the main validation logic and document validator implementation.

use fedramp_core::{Result, Error};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::{Duration, Instant};
use tracing::{debug, info, warn};
use regex::Regex;
use chrono::{DateTime, NaiveDate};
use crate::mapping::{MappingConfiguration, InventoryMappings, PoamMappings};
use crate::fuzzy::FuzzyMatcher;
use uuid::Uuid;
use lru::LruCache;

use super::types::*;
use super::confidence::*;
use super::overrides::*;
use super::reports::*;

/// Column validator for field-level validation
#[derive(Debug)]
pub struct ColumnValidator {
    /// Mapping configuration for validation rules
    mapping_config: MappingConfiguration,
    /// Compiled regex patterns for format validation
    regex_cache: HashMap<String, Regex>,
    /// Minimum quality threshold for acceptance
    min_quality_threshold: f64,
    /// Performance target in milliseconds
    performance_target_ms: u64,
    /// Custom validation functions
    custom_validators: HashMap<String, fn(&[serde_json::Value]) -> Result<(ValidationResult, f64)>>,
}

/// Document validator for comprehensive document validation
#[derive(Debug, Clone)]
pub struct DocumentValidator {
    /// Validation rules loaded from configuration
    rules: HashMap<String, Vec<ValidationRule>>,
    /// Column validator for field-level validation
    column_validator: Option<ColumnValidator>,
}

impl ColumnValidator {
    /// Create a new column validator with mapping configuration
    pub fn new(mapping_config: MappingConfiguration) -> Self {
        Self {
            mapping_config,
            regex_cache: HashMap::new(),
            min_quality_threshold: 0.7,
            performance_target_ms: 100,
            custom_validators: HashMap::new(),
        }
    }

    /// Validate columns against mapping configuration
    pub fn validate_columns(
        &mut self,
        headers: &[String],
        data: &[Vec<serde_json::Value>],
        document_type: &str,
    ) -> Result<ColumnValidationReport> {
        let start_time = Instant::now();
        
        info!("Starting column validation for document type: {}", document_type);
        
        let validation_rules = self.get_validation_rules(document_type)?;
        let mut field_results = Vec::new();
        let mut missing_required = Vec::new();
        let mut type_mismatches = Vec::new();
        let mut enumeration_failures = Vec::new();
        let mut cross_field_results = Vec::new();

        // Validate each field according to the rules
        for rule in &validation_rules {
            let field_result = self.validate_field(rule, headers, data)?;
            
            match &field_result.status {
                ValidationStatus::MissingRequired => {
                    missing_required.push(RequiredFieldInfo {
                        field_id: rule.field_id.clone(),
                        expected_columns: rule.column_names.clone(),
                        oscal_field: rule.oscal_field.clone(),
                        description: format!("Required field {} is missing", rule.field_id),
                        alternatives: self.suggest_alternatives(&rule.field_id, headers),
                    });
                }
                ValidationStatus::Invalid => {
                    if let Some(data_type) = &rule.data_type {
                        type_mismatches.push(TypeMismatchInfo {
                            field_id: rule.field_id.clone(),
                            column_name: field_result.source_column.clone().unwrap_or_default(),
                            expected_type: data_type.clone(),
                            actual_type: "unknown".to_string(),
                            sample_values: vec!["sample".to_string()],
                            suggested_conversion: None,
                        });
                    }
                }
                _ => {}
            }

            field_results.push(field_result);
        }

        // Perform cross-field validation
        cross_field_results = self.validate_cross_fields(&field_results, data)?;

        // Calculate metrics
        let metrics = self.calculate_validation_metrics(&field_results, &missing_required);
        
        let total_execution_time = start_time.elapsed();
        
        Ok(ColumnValidationReport {
            document_type: document_type.to_string(),
            is_valid: metrics.error_count == 0 && missing_required.is_empty(),
            field_results,
            missing_required,
            type_mismatches,
            enumeration_failures,
            cross_field_results,
            metrics,
            total_execution_time,
            timestamp: chrono::Utc::now(),
        })
    }

    /// Get validation rules for a specific document type
    fn get_validation_rules(&self, document_type: &str) -> Result<Vec<ColumnValidationRule>> {
        match document_type.to_lowercase().as_str() {
            "inventory" => {
                if let Some(inventory_mappings) = &self.mapping_config.inventory {
                    Ok(self.convert_inventory_mappings_to_rules(inventory_mappings))
                } else {
                    Err(Error::ValidationError("No inventory mappings configured".to_string()))
                }
            }
            "poam" => {
                if let Some(poam_mappings) = &self.mapping_config.poam {
                    Ok(self.convert_poam_mappings_to_rules(poam_mappings))
                } else {
                    Err(Error::ValidationError("No POAM mappings configured".to_string()))
                }
            }
            _ => {
                warn!("Unknown document type: {}", document_type);
                Ok(Vec::new())
            }
        }
    }

    /// Convert inventory mappings to validation rules
    fn convert_inventory_mappings_to_rules(&self, mappings: &InventoryMappings) -> Vec<ColumnValidationRule> {
        let mut rules = Vec::new();
        
        // Add rules for each mapped field
        if let Some(asset_id) = &mappings.asset_id {
            rules.push(ColumnValidationRule {
                field_id: "asset_id".to_string(),
                column_names: vec![asset_id.clone()],
                oscal_field: "system-component.uuid".to_string(),
                required: true,
                validation_type: Some("format".to_string()),
                allowed_values: None,
                pattern: Some(r"^[A-Za-z0-9\-_]+$".to_string()),
                data_type: Some("string".to_string()),
                conditional: None,
            });
        }

        if let Some(asset_name) = &mappings.asset_name {
            rules.push(ColumnValidationRule {
                field_id: "asset_name".to_string(),
                column_names: vec![asset_name.clone()],
                oscal_field: "system-component.title".to_string(),
                required: true,
                validation_type: Some("format".to_string()),
                allowed_values: None,
                pattern: Some(r"^.{1,255}$".to_string()),
                data_type: Some("string".to_string()),
                conditional: None,
            });
        }

        if let Some(asset_type) = &mappings.asset_type {
            rules.push(ColumnValidationRule {
                field_id: "asset_type".to_string(),
                column_names: vec![asset_type.clone()],
                oscal_field: "system-component.type".to_string(),
                required: true,
                validation_type: Some("enumeration".to_string()),
                allowed_values: Some(vec![
                    "software".to_string(),
                    "hardware".to_string(),
                    "service".to_string(),
                    "policy".to_string(),
                    "process".to_string(),
                    "plan".to_string(),
                    "guidance".to_string(),
                    "standard".to_string(),
                    "validation".to_string(),
                ]),
                pattern: None,
                data_type: Some("string".to_string()),
                conditional: None,
            });
        }

        rules
    }

    /// Convert POAM mappings to validation rules
    fn convert_poam_mappings_to_rules(&self, mappings: &PoamMappings) -> Vec<ColumnValidationRule> {
        let mut rules = Vec::new();
        
        if let Some(poam_id) = &mappings.poam_id {
            rules.push(ColumnValidationRule {
                field_id: "poam_id".to_string(),
                column_names: vec![poam_id.clone()],
                oscal_field: "finding.uuid".to_string(),
                required: true,
                validation_type: Some("format".to_string()),
                allowed_values: None,
                pattern: Some(r"^[A-Za-z0-9\-_]+$".to_string()),
                data_type: Some("string".to_string()),
                conditional: None,
            });
        }

        if let Some(control_id) = &mappings.control_id {
            rules.push(ColumnValidationRule {
                field_id: "control_id".to_string(),
                column_names: vec![control_id.clone()],
                oscal_field: "finding.related-observations.observation.relevant-evidence.href".to_string(),
                required: true,
                validation_type: Some("format".to_string()),
                allowed_values: None,
                pattern: Some(r"^[A-Z]{2}-\d+(\.\d+)*$".to_string()),
                data_type: Some("string".to_string()),
                conditional: None,
            });
        }

        if let Some(severity) = &mappings.severity {
            rules.push(ColumnValidationRule {
                field_id: "severity".to_string(),
                column_names: vec![severity.clone()],
                oscal_field: "finding.target.status.state".to_string(),
                required: true,
                validation_type: Some("enumeration".to_string()),
                allowed_values: Some(vec![
                    "low".to_string(),
                    "moderate".to_string(),
                    "high".to_string(),
                    "critical".to_string(),
                ]),
                pattern: None,
                data_type: Some("string".to_string()),
                conditional: None,
            });
        }

        rules
    }

    /// Validate a single field according to its rule
    fn validate_field(
        &mut self,
        rule: &ColumnValidationRule,
        headers: &[String],
        data: &[Vec<serde_json::Value>],
    ) -> Result<ColumnValidationResult> {
        let start_time = Instant::now();
        
        // Find the column for this field
        let source_column = self.find_matching_column(rule, headers);
        
        if source_column.is_none() && rule.required {
            return Ok(ColumnValidationResult {
                field_id: rule.field_id.clone(),
                source_column: None,
                oscal_field: rule.oscal_field.clone(),
                passed: false,
                status: ValidationStatus::MissingRequired,
                message: format!("Required field '{}' not found in headers", rule.field_id),
                severity: ValidationSeverity::Error,
                suggestions: self.suggest_alternatives(&rule.field_id, headers),
                execution_time: start_time.elapsed(),
            });
        }

        if let Some(column_name) = source_column {
            // Find column index
            let column_index = headers.iter().position(|h| h == &column_name);
            
            if let Some(col_idx) = column_index {
                // Validate the data in this column
                let column_data: Vec<&serde_json::Value> = data.iter()
                    .filter_map(|row| row.get(col_idx))
                    .collect();
                
                let validation_result = self.validate_column_data(rule, &column_data)?;
                
                return Ok(ColumnValidationResult {
                    field_id: rule.field_id.clone(),
                    source_column: Some(column_name),
                    oscal_field: rule.oscal_field.clone(),
                    passed: validation_result.passed,
                    status: if validation_result.passed { 
                        ValidationStatus::Valid 
                    } else { 
                        ValidationStatus::Invalid 
                    },
                    message: validation_result.message.unwrap_or_else(|| "Validation completed".to_string()),
                    severity: if validation_result.passed {
                        ValidationSeverity::Info
                    } else {
                        ValidationSeverity::Error
                    },
                    suggestions: Vec::new(),
                    execution_time: start_time.elapsed(),
                });
            }
        }

        // Field not found but not required
        Ok(ColumnValidationResult {
            field_id: rule.field_id.clone(),
            source_column: None,
            oscal_field: rule.oscal_field.clone(),
            passed: true,
            status: ValidationStatus::MissingOptional,
            message: format!("Optional field '{}' not found", rule.field_id),
            severity: ValidationSeverity::Info,
            suggestions: Vec::new(),
            execution_time: start_time.elapsed(),
        })
    }

    /// Find a matching column for a validation rule
    fn find_matching_column(&self, rule: &ColumnValidationRule, headers: &[String]) -> Option<String> {
        // Try exact matches first
        for column_name in &rule.column_names {
            if headers.contains(column_name) {
                return Some(column_name.clone());
            }
        }
        
        // Try case-insensitive matches
        for column_name in &rule.column_names {
            for header in headers {
                if header.to_lowercase() == column_name.to_lowercase() {
                    return Some(header.clone());
                }
            }
        }
        
        // Try fuzzy matching as last resort
        let fuzzy_matcher = FuzzyMatcher::new();
        for column_name in &rule.column_names {
            for header in headers {
                if fuzzy_matcher.similarity(column_name, header) > 0.8 {
                    return Some(header.clone());
                }
            }
        }
        
        None
    }

    /// Validate data in a column according to the rule
    fn validate_column_data(
        &mut self,
        rule: &ColumnValidationRule,
        data: &[&serde_json::Value],
    ) -> Result<ValidationResult> {
        let mut passed = true;
        let mut messages = Vec::new();

        // Check for empty data
        if data.is_empty() {
            return Ok(ValidationResult {
                field_name: rule.field_id.clone(),
                rule_type: "presence".to_string(),
                passed: !rule.required,
                message: if rule.required {
                    Some("Required field has no data".to_string())
                } else {
                    None
                },
                severity: if rule.required { "error".to_string() } else { "info".to_string() },
            });
        }

        // Validate based on validation type
        if let Some(validation_type) = &rule.validation_type {
            match validation_type.as_str() {
                "enumeration" => {
                    if let Some(allowed_values) = &rule.allowed_values {
                        for value in data {
                            if let Some(str_value) = value.as_str() {
                                if !allowed_values.contains(&str_value.to_string()) {
                                    passed = false;
                                    messages.push(format!("Value '{}' not in allowed values", str_value));
                                }
                            }
                        }
                    }
                }
                "format" => {
                    if let Some(pattern) = &rule.pattern {
                        let regex = self.get_or_compile_regex(pattern)?;
                        for value in data {
                            if let Some(str_value) = value.as_str() {
                                if !regex.is_match(str_value) {
                                    passed = false;
                                    messages.push(format!("Value '{}' does not match required format", str_value));
                                }
                            }
                        }
                    }
                }
                _ => {
                    warn!("Unknown validation type: {}", validation_type);
                }
            }
        }

        Ok(ValidationResult {
            field_name: rule.field_id.clone(),
            rule_type: rule.validation_type.clone().unwrap_or_else(|| "general".to_string()),
            passed,
            message: if messages.is_empty() { None } else { Some(messages.join("; ")) },
            severity: if passed { "info".to_string() } else { "error".to_string() },
        })
    }

    /// Get or compile a regex pattern
    fn get_or_compile_regex(&mut self, pattern: &str) -> Result<&Regex> {
        if !self.regex_cache.contains_key(pattern) {
            let regex = Regex::new(pattern)
                .map_err(|e| Error::ValidationError(format!("Invalid regex pattern '{}': {}", pattern, e)))?;
            self.regex_cache.insert(pattern.to_string(), regex);
        }
        
        Ok(self.regex_cache.get(pattern).unwrap())
    }

    /// Suggest alternative column names for a missing field
    fn suggest_alternatives(&self, field_id: &str, headers: &[String]) -> Vec<String> {
        let fuzzy_matcher = FuzzyMatcher::new();
        let mut suggestions = Vec::new();
        
        for header in headers {
            let similarity = fuzzy_matcher.similarity(field_id, header);
            if similarity > 0.5 {
                suggestions.push(header.clone());
            }
        }
        
        suggestions.sort_by(|a, b| {
            let sim_a = fuzzy_matcher.similarity(field_id, a);
            let sim_b = fuzzy_matcher.similarity(field_id, b);
            sim_b.partial_cmp(&sim_a).unwrap_or(std::cmp::Ordering::Equal)
        });
        
        suggestions.truncate(3); // Limit to top 3 suggestions
        suggestions
    }

    /// Perform cross-field validation
    fn validate_cross_fields(
        &self,
        _field_results: &[ColumnValidationResult],
        _data: &[Vec<serde_json::Value>],
    ) -> Result<Vec<CrossFieldValidationResult>> {
        // Placeholder for cross-field validation logic
        // This would implement business rules that span multiple fields
        Ok(Vec::new())
    }

    /// Calculate validation metrics
    fn calculate_validation_metrics(
        &self,
        field_results: &[ColumnValidationResult],
        missing_required: &[RequiredFieldInfo],
    ) -> ValidationMetrics {
        let total_fields = field_results.len();
        let valid_fields = field_results.iter().filter(|r| r.passed).count();
        let invalid_fields = total_fields - valid_fields;
        let warning_count = field_results.iter()
            .filter(|r| r.severity == ValidationSeverity::Warning)
            .count();
        let error_count = field_results.iter()
            .filter(|r| r.severity == ValidationSeverity::Error)
            .count() + missing_required.len();
        
        let validation_score = if total_fields > 0 {
            valid_fields as f64 / total_fields as f64
        } else {
            1.0
        };

        ValidationMetrics {
            total_fields,
            required_fields: field_results.iter()
                .filter(|r| matches!(r.status, ValidationStatus::MissingRequired | ValidationStatus::Valid))
                .count(),
            optional_fields: field_results.iter()
                .filter(|r| matches!(r.status, ValidationStatus::MissingOptional))
                .count(),
            valid_fields,
            missing_required_count: missing_required.len(),
            invalid_fields,
            warning_count,
            error_count,
            validation_score,
        }
    }
}

impl DocumentValidator {
    /// Create a new document validator
    #[must_use]
    pub fn new() -> Self {
        Self {
            rules: HashMap::new(),
            column_validator: None,
        }
    }

    /// Add validation rules for a document type
    pub fn add_rules(&mut self, document_type: String, rules: Vec<ValidationRule>) {
        self.rules.insert(document_type, rules);
    }

    /// Set the column validator
    pub fn set_column_validator(&mut self, validator: ColumnValidator) {
        self.column_validator = Some(validator);
    }

    /// Validate a document with the given data
    pub fn validate_document(
        &mut self,
        document_type: &str,
        headers: &[String],
        data: &[Vec<serde_json::Value>],
    ) -> Result<ColumnValidationReport> {
        if let Some(ref mut column_validator) = self.column_validator {
            column_validator.validate_columns(headers, data, document_type)
        } else {
            Err(Error::ValidationError("No column validator configured".to_string()))
        }
    }

    /// Get validation rules for a document type
    pub fn get_rules(&self, document_type: &str) -> Option<&Vec<ValidationRule>> {
        self.rules.get(document_type)
    }

    /// Remove validation rules for a document type
    pub fn remove_rules(&mut self, document_type: &str) -> Option<Vec<ValidationRule>> {
        self.rules.remove(document_type)
    }

    /// Clear all validation rules
    pub fn clear_rules(&mut self) {
        self.rules.clear();
    }

    /// Get all document types with validation rules
    pub fn get_document_types(&self) -> Vec<&String> {
        self.rules.keys().collect()
    }
}

impl Default for DocumentValidator {
    fn default() -> Self {
        Self::new()
    }
}
