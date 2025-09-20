// Modified: 2025-09-20

//! Document validation and quality assurance
//!
//! This module provides comprehensive validation for parsed documents
//! including data completeness, consistency, and quality scoring.

use fedramp_core::{Result, Error};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::{Duration, Instant};
use tracing::{debug, info, warn};
use regex::Regex;
use chrono::{DateTime, NaiveDate};
use crate::mapping::{MappingConfiguration, InventoryMappings, PoamMappings};

/// Validation rule configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationRule {
    /// Field name to validate
    pub field_name: String,
    /// Rule type (required, format, range, etc.)
    pub rule_type: String,
    /// Rule parameters
    pub parameters: serde_json::Value,
    /// Error message template
    pub error_message: String,
    /// Severity level (error, warning, info)
    pub severity: String,
}

/// Enhanced validation rule for column validation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ColumnValidationRule {
    /// Field identifier
    pub field_id: String,
    /// Column names that map to this field
    pub column_names: Vec<String>,
    /// Target OSCAL field path
    pub oscal_field: String,
    /// Whether this field is required
    pub required: bool,
    /// Validation type (enumeration, format, etc.)
    pub validation_type: Option<String>,
    /// Allowed values for enumeration validation
    pub allowed_values: Option<Vec<String>>,
    /// Regex pattern for format validation
    pub pattern: Option<String>,
    /// Data type expected
    pub data_type: Option<String>,
    /// Conditional requirements
    pub conditional: Option<ConditionalRequirement>,
}

/// Conditional requirement for field validation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConditionalRequirement {
    /// Field that determines the condition
    pub depends_on: String,
    /// Values that trigger the requirement
    pub trigger_values: Vec<String>,
    /// Whether the condition is inverted (NOT logic)
    pub inverted: bool,
}

/// Validation result for a single field
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationResult {
    /// Field name that was validated
    pub field_name: String,
    /// Rule that was applied
    pub rule_type: String,
    /// Whether validation passed
    pub passed: bool,
    /// Error message if validation failed
    pub message: Option<String>,
    /// Severity level
    pub severity: String,
}

/// Enhanced validation result for column validation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ColumnValidationResult {
    /// Field identifier
    pub field_id: String,
    /// Source column name (if found)
    pub source_column: Option<String>,
    /// Target OSCAL field path
    pub oscal_field: String,
    /// Whether validation passed
    pub passed: bool,
    /// Validation status
    pub status: ValidationStatus,
    /// Error or warning message
    pub message: String,
    /// Severity level
    pub severity: ValidationSeverity,
    /// Suggested fixes
    pub suggestions: Vec<String>,
    /// Execution time for this validation
    pub execution_time: Duration,
}

/// Validation status enumeration
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ValidationStatus {
    /// Field is present and valid
    Valid,
    /// Field is missing but required
    MissingRequired,
    /// Field is present but invalid
    Invalid,
    /// Field has warnings but is acceptable
    Warning,
    /// Field is missing but optional
    MissingOptional,
    /// Field validation was skipped
    Skipped,
}

/// Validation severity levels
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, PartialOrd)]
pub enum ValidationSeverity {
    /// Informational message
    Info,
    /// Warning that should be addressed
    Warning,
    /// Error that must be fixed
    Error,
    /// Critical error that prevents processing
    Critical,
}

/// Comprehensive column validation report
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ColumnValidationReport {
    /// Document type being validated
    pub document_type: String,
    /// Overall validation status
    pub is_valid: bool,
    /// Individual field validation results
    pub field_results: Vec<ColumnValidationResult>,
    /// Missing required fields
    pub missing_required: Vec<RequiredFieldInfo>,
    /// Data type mismatches
    pub type_mismatches: Vec<TypeMismatchInfo>,
    /// Enumeration validation failures
    pub enumeration_failures: Vec<EnumerationFailureInfo>,
    /// Cross-field validation results
    pub cross_field_results: Vec<CrossFieldValidationResult>,
    /// Overall validation metrics
    pub metrics: ValidationMetrics,
    /// Total validation time
    pub total_execution_time: Duration,
    /// Validation timestamp
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

/// Information about missing required fields
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RequiredFieldInfo {
    /// Field identifier
    pub field_id: String,
    /// Expected column names
    pub expected_columns: Vec<String>,
    /// Target OSCAL field
    pub oscal_field: String,
    /// Description of the field
    pub description: String,
    /// Suggested alternatives
    pub alternatives: Vec<String>,
}

/// Information about data type mismatches
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TypeMismatchInfo {
    /// Field identifier
    pub field_id: String,
    /// Source column name
    pub column_name: String,
    /// Expected data type
    pub expected_type: String,
    /// Actual data type detected
    pub actual_type: String,
    /// Sample values that caused the mismatch
    pub sample_values: Vec<String>,
    /// Suggested conversion
    pub suggested_conversion: Option<String>,
}

/// Information about enumeration validation failures
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnumerationFailureInfo {
    /// Field identifier
    pub field_id: String,
    /// Source column name
    pub column_name: String,
    /// Invalid values found
    pub invalid_values: Vec<String>,
    /// Allowed values
    pub allowed_values: Vec<String>,
    /// Suggested mappings for invalid values
    pub suggested_mappings: HashMap<String, String>,
}

/// Cross-field validation result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CrossFieldValidationResult {
    /// Validation rule name
    pub rule_name: String,
    /// Fields involved in the validation
    pub involved_fields: Vec<String>,
    /// Whether validation passed
    pub passed: bool,
    /// Validation message
    pub message: String,
    /// Severity level
    pub severity: ValidationSeverity,
}

/// Validation metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationMetrics {
    /// Total number of fields validated
    pub total_fields: usize,
    /// Number of required fields
    pub required_fields: usize,
    /// Number of optional fields
    pub optional_fields: usize,
    /// Number of fields that passed validation
    pub valid_fields: usize,
    /// Number of missing required fields
    pub missing_required_count: usize,
    /// Number of invalid fields
    pub invalid_fields: usize,
    /// Number of warnings
    pub warning_count: usize,
    /// Number of errors
    pub error_count: usize,
    /// Overall validation score (0.0 to 1.0)
    pub validation_score: f64,
}

/// Quality metrics for a document
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QualityMetrics {
    /// Overall quality score (0.0 to 1.0)
    pub overall_score: f64,
    /// Completeness score (0.0 to 1.0)
    pub completeness_score: f64,
    /// Consistency score (0.0 to 1.0)
    pub consistency_score: f64,
    /// Accuracy score (0.0 to 1.0)
    pub accuracy_score: f64,
    /// Total number of fields
    pub total_fields: usize,
    /// Number of populated fields
    pub populated_fields: usize,
    /// Number of validation errors
    pub error_count: usize,
    /// Number of validation warnings
    pub warning_count: usize,
}

/// Comprehensive column validator for document validation
#[derive(Debug)]
pub struct ColumnValidator {
    /// Mapping configuration for validation rules
    mapping_config: MappingConfiguration,
    /// Compiled regex patterns for format validation
    regex_cache: HashMap<String, Regex>,
    /// Minimum quality threshold (0.0 to 1.0)
    min_quality_threshold: f64,
    /// Performance target for validation (milliseconds)
    performance_target_ms: u64,
}

/// Document validator for quality assurance
#[derive(Debug, Clone)]
pub struct DocumentValidator {
    /// Validation rules loaded from configuration
    rules: HashMap<String, Vec<ValidationRule>>,
    /// Minimum quality threshold
    min_quality_threshold: f64,
}

impl ColumnValidator {
    /// Create a new column validator with mapping configuration
    pub fn new(mapping_config: MappingConfiguration) -> Self {
        Self {
            mapping_config,
            regex_cache: HashMap::new(),
            min_quality_threshold: 0.8,
            performance_target_ms: 50, // Target <50ms per document
        }
    }

    /// Create a new column validator with custom thresholds
    pub fn with_thresholds(
        mapping_config: MappingConfiguration,
        quality_threshold: f64,
        performance_target_ms: u64,
    ) -> Self {
        Self {
            mapping_config,
            regex_cache: HashMap::new(),
            min_quality_threshold: quality_threshold,
            performance_target_ms,
        }
    }

    /// Validate columns against required fields for a specific document type
    pub fn validate_columns(
        &mut self,
        document_type: &str,
        detected_columns: &[String],
        sample_data: &HashMap<String, Vec<serde_json::Value>>,
    ) -> Result<ColumnValidationReport> {
        let start_time = Instant::now();
        info!("Starting column validation for document type: {}", document_type);

        let mut field_results = Vec::new();
        let mut missing_required = Vec::new();
        let mut type_mismatches = Vec::new();
        let mut enumeration_failures = Vec::new();
        let mut cross_field_results = Vec::new();

        // Get validation rules for the document type
        let validation_rules = self.get_validation_rules(document_type)?;

        // Validate each required field
        for rule in &validation_rules {
            let field_start = Instant::now();
            let result = self.validate_field(rule, detected_columns, sample_data)?;

            // Collect specific failure types
            match &result.status {
                ValidationStatus::MissingRequired => {
                    missing_required.push(RequiredFieldInfo {
                        field_id: rule.field_id.clone(),
                        expected_columns: rule.column_names.clone(),
                        oscal_field: rule.oscal_field.clone(),
                        description: format!("Required field for {}", document_type),
                        alternatives: self.suggest_alternatives(&rule.column_names, detected_columns),
                    });
                }
                ValidationStatus::Invalid => {
                    if let Some(ref validation_type) = rule.validation_type {
                        if validation_type.contains("enum") || validation_type.contains("values") {
                            if let Some(ref column_name) = result.source_column {
                                enumeration_failures.push(self.create_enumeration_failure(
                                    rule, column_name, sample_data
                                )?);
                            }
                        } else {
                            if let Some(ref column_name) = result.source_column {
                                type_mismatches.push(self.create_type_mismatch(
                                    rule, column_name, sample_data
                                )?);
                            }
                        }
                    }
                }
                _ => {}
            }

            field_results.push(result);
        }

        // Perform cross-field validation
        cross_field_results = self.validate_cross_field_rules(document_type, detected_columns, sample_data)?;

        // Calculate metrics
        let metrics = self.calculate_validation_metrics(&field_results, &cross_field_results);
        let total_execution_time = start_time.elapsed();

        // Check performance target
        if total_execution_time.as_millis() as u64 > self.performance_target_ms {
            warn!(
                "Column validation took {}ms, exceeding target of {}ms",
                total_execution_time.as_millis(),
                self.performance_target_ms
            );
        }

        let report = ColumnValidationReport {
            document_type: document_type.to_string(),
            is_valid: metrics.error_count == 0 && metrics.missing_required_count == 0,
            field_results,
            missing_required,
            type_mismatches,
            enumeration_failures,
            cross_field_results,
            metrics,
            total_execution_time,
            timestamp: chrono::Utc::now(),
        };

        info!(
            "Column validation completed in {}ms with {} errors, {} warnings",
            total_execution_time.as_millis(),
            report.metrics.error_count,
            report.metrics.warning_count
        );

        Ok(report)
    }

    /// Get validation rules for a specific document type
    fn get_validation_rules(&self, document_type: &str) -> Result<Vec<ColumnValidationRule>> {
        let mut rules = Vec::new();

        match document_type.to_lowercase().as_str() {
            "inventory" | "component-definition" => {
                if let Some(ref inventory_mappings) = self.mapping_config.inventory_mappings {
                    rules.extend(self.extract_inventory_rules(inventory_mappings)?);
                }
            }
            "poam" | "plan-of-action-and-milestones" => {
                if let Some(ref poam_mappings) = self.mapping_config.poam_mappings {
                    rules.extend(self.extract_poam_rules(poam_mappings)?);
                }
            }
            _ => {
                return Err(Error::document_parsing(format!(
                    "Unsupported document type for validation: {}",
                    document_type
                )));
            }
        }

        if rules.is_empty() {
            return Err(Error::document_parsing(format!(
                "No validation rules found for document type: {}",
                document_type
            )));
        }

        Ok(rules)
    }

    /// Extract validation rules from inventory mappings
    fn extract_inventory_rules(&self, mappings: &InventoryMappings) -> Result<Vec<ColumnValidationRule>> {
        let mut rules = Vec::new();

        // Extract from required_columns in fedramp_iiw_mappings
        for (field_id, column_mapping) in &mappings.fedramp_iiw_mappings.required_columns {
            let rule = ColumnValidationRule {
                field_id: field_id.clone(),
                column_names: column_mapping.column_names.clone(),
                oscal_field: column_mapping.field.clone(),
                required: column_mapping.required,
                validation_type: column_mapping.validation.clone(),
                allowed_values: self.extract_inventory_allowed_values(&mappings.validation_rules, &column_mapping.validation)?,
                pattern: self.extract_inventory_pattern(&mappings.validation_rules, &column_mapping.validation)?,
                data_type: None, // Not specified in current structure
                conditional: None, // TODO: Implement conditional requirements
            };
            rules.push(rule);
        }

        Ok(rules)
    }

    /// Extract validation rules from POA&M mappings
    fn extract_poam_rules(&self, mappings: &PoamMappings) -> Result<Vec<ColumnValidationRule>> {
        let mut rules = Vec::new();

        // Extract from required_columns in fedramp_v3_mappings
        for (field_id, column_mapping) in &mappings.fedramp_v3_mappings.required_columns {
            let rule = ColumnValidationRule {
                field_id: field_id.clone(),
                column_names: column_mapping.column_names.clone(),
                oscal_field: column_mapping.oscal_field.clone(),
                required: column_mapping.required,
                validation_type: column_mapping.validation.clone(),
                allowed_values: self.extract_poam_allowed_values(&mappings.fedramp_v3_mappings.validation_rules, &column_mapping.validation)?,
                pattern: self.extract_poam_pattern(&mappings.fedramp_v3_mappings.validation_rules, &column_mapping.validation)?,
                data_type: None, // Not specified in current structure
                conditional: None, // TODO: Implement conditional requirements
            };
            rules.push(rule);
        }

        Ok(rules)
    }

    /// Extract allowed values from inventory validation rules
    fn extract_inventory_allowed_values(&self, validation_rules: &crate::mapping::config::ValidationRules, validation_type: &Option<String>) -> Result<Option<Vec<String>>> {
        if let Some(ref validation_type) = validation_type {
            match validation_type.as_str() {
                "asset_types" => Ok(validation_rules.asset_types.clone()),
                "environments" => Ok(validation_rules.environments.clone()),
                "criticality_levels" => Ok(validation_rules.criticality_levels.clone()),
                "boolean_values" => Ok(validation_rules.boolean_values.clone()),
                _ => Ok(None),
            }
        } else {
            Ok(None)
        }
    }

    /// Extract pattern from inventory validation rules
    fn extract_inventory_pattern(&self, validation_rules: &crate::mapping::config::ValidationRules, validation_type: &Option<String>) -> Result<Option<String>> {
        if let Some(ref validation_type) = validation_type {
            match validation_type.as_str() {
                "ip_address" => Ok(validation_rules.ip_address_pattern.clone()),
                "mac_address" => Ok(validation_rules.mac_address_pattern.clone()),
                _ => Ok(None),
            }
        } else {
            Ok(None)
        }
    }

    /// Extract allowed values from POA&M validation rules
    fn extract_poam_allowed_values(&self, validation_rules: &crate::mapping::poam::PoamValidationRules, validation_type: &Option<String>) -> Result<Option<Vec<String>>> {
        if let Some(ref validation_type) = validation_type {
            match validation_type.as_str() {
                "severity_levels" => Ok(validation_rules.severity_levels.clone()),
                "status_values" => Ok(validation_rules.status_values.clone()),
                _ => Ok(None),
            }
        } else {
            Ok(None)
        }
    }

    /// Extract pattern from POA&M validation rules
    fn extract_poam_pattern(&self, validation_rules: &crate::mapping::poam::PoamValidationRules, validation_type: &Option<String>) -> Result<Option<String>> {
        if let Some(ref validation_type) = validation_type {
            match validation_type.as_str() {
                "control_id_list" | "control_id" => Ok(validation_rules.control_id_pattern.clone()),
                _ => Ok(None),
            }
        } else {
            Ok(None)
        }
    }

    /// Validate a single field against its rule
    fn validate_field(
        &mut self,
        rule: &ColumnValidationRule,
        detected_columns: &[String],
        sample_data: &HashMap<String, Vec<serde_json::Value>>,
    ) -> Result<ColumnValidationResult> {
        let start_time = Instant::now();

        // Find matching column
        let matched_column = self.find_matching_column(&rule.column_names, detected_columns);

        let (status, message, suggestions) = if let Some(ref column_name) = matched_column {
            // Column found, validate its data
            if let Some(column_data) = sample_data.get(column_name) {
                self.validate_column_data(rule, column_data)?
            } else {
                (
                    ValidationStatus::Warning,
                    "Column found but no sample data available for validation".to_string(),
                    vec!["Ensure the column contains data".to_string()],
                )
            }
        } else if rule.required {
            (
                ValidationStatus::MissingRequired,
                format!("Required field '{}' is missing", rule.field_id),
                self.suggest_alternatives(&rule.column_names, detected_columns),
            )
        } else {
            (
                ValidationStatus::MissingOptional,
                format!("Optional field '{}' is missing", rule.field_id),
                vec![],
            )
        };

        let severity = match status {
            ValidationStatus::Valid => ValidationSeverity::Info,
            ValidationStatus::Warning => ValidationSeverity::Warning,
            ValidationStatus::MissingRequired | ValidationStatus::Invalid => ValidationSeverity::Error,
            ValidationStatus::MissingOptional => ValidationSeverity::Info,
            ValidationStatus::Skipped => ValidationSeverity::Info,
        };

        Ok(ColumnValidationResult {
            field_id: rule.field_id.clone(),
            source_column: matched_column,
            oscal_field: rule.oscal_field.clone(),
            passed: matches!(status, ValidationStatus::Valid | ValidationStatus::Warning | ValidationStatus::MissingOptional),
            status,
            message,
            severity,
            suggestions,
            execution_time: start_time.elapsed(),
        })
    }

    /// Find matching column name using fuzzy matching
    fn find_matching_column(&self, expected_names: &[String], detected_columns: &[String]) -> Option<String> {
        // First try exact matches (case-insensitive)
        for expected in expected_names {
            for detected in detected_columns {
                if expected.to_lowercase() == detected.to_lowercase() {
                    return Some(detected.clone());
                }
            }
        }

        // Then try partial matches
        for expected in expected_names {
            for detected in detected_columns {
                if detected.to_lowercase().contains(&expected.to_lowercase()) ||
                   expected.to_lowercase().contains(&detected.to_lowercase()) {
                    return Some(detected.clone());
                }
            }
        }

        None
    }

    /// Validate column data against rule requirements
    fn validate_column_data(
        &mut self,
        rule: &ColumnValidationRule,
        column_data: &[serde_json::Value],
    ) -> Result<(ValidationStatus, String, Vec<String>)> {
        if column_data.is_empty() {
            return Ok((
                if rule.required { ValidationStatus::Invalid } else { ValidationStatus::Warning },
                "Column contains no data".to_string(),
                vec!["Ensure the column is populated with valid data".to_string()],
            ));
        }

        // Check for validation type
        if let Some(ref validation_type) = rule.validation_type {
            return self.validate_by_type(rule, column_data, validation_type);
        }

        // If no specific validation, just check for non-empty values
        let non_empty_count = column_data.iter()
            .filter(|v| !v.is_null() && !v.as_str().map_or(false, |s| s.trim().is_empty()))
            .count();

        if non_empty_count == 0 && rule.required {
            Ok((
                ValidationStatus::Invalid,
                "Required field contains only empty values".to_string(),
                vec!["Populate the field with valid data".to_string()],
            ))
        } else {
            Ok((
                ValidationStatus::Valid,
                format!("Field validation passed ({} non-empty values)", non_empty_count),
                vec![],
            ))
        }
    }

    /// Validate column data by specific validation type
    fn validate_by_type(
        &mut self,
        rule: &ColumnValidationRule,
        column_data: &[serde_json::Value],
        validation_type: &str,
    ) -> Result<(ValidationStatus, String, Vec<String>)> {
        match validation_type {
            "boolean" => self.validate_boolean_values(column_data),
            "date" => self.validate_date_values(column_data),
            "ip_address" => self.validate_ip_address_values(column_data),
            "email" => self.validate_email_values(column_data),
            "url" => self.validate_url_values(column_data),
            "unique_identifier" => self.validate_unique_identifier_values(column_data),
            "alphanumeric" => self.validate_alphanumeric_values(column_data),
            validation_type if validation_type.ends_with("_pattern") => {
                self.validate_pattern_values(rule, column_data, validation_type)
            }
            _ => {
                // Assume it's an enumeration validation
                self.validate_enumeration_values(rule, column_data, validation_type)
            }
        }
    }

    /// Validate boolean values
    fn validate_boolean_values(&self, column_data: &[serde_json::Value]) -> Result<(ValidationStatus, String, Vec<String>)> {
        let boolean_values = ["yes", "no", "y", "n", "true", "false", "1", "0", "on", "off"];
        let mut invalid_values = Vec::new();

        for value in column_data {
            if let Some(str_val) = value.as_str() {
                if !str_val.trim().is_empty() && !boolean_values.contains(&str_val.to_lowercase().as_str()) {
                    invalid_values.push(str_val.to_string());
                }
            } else if !value.is_null() && !value.is_boolean() {
                invalid_values.push(value.to_string());
            }
        }

        if invalid_values.is_empty() {
            Ok((
                ValidationStatus::Valid,
                "All boolean values are valid".to_string(),
                vec![],
            ))
        } else {
            Ok((
                ValidationStatus::Invalid,
                format!("Invalid boolean values found: {}", invalid_values.join(", ")),
                vec![
                    "Use standard boolean values: yes/no, true/false, 1/0".to_string(),
                    "Ensure consistent formatting across all rows".to_string(),
                ],
            ))
        }
    }

    /// Validate date values
    fn validate_date_values(&self, column_data: &[serde_json::Value]) -> Result<(ValidationStatus, String, Vec<String>)> {
        let mut invalid_values = Vec::new();

        for value in column_data {
            if let Some(str_val) = value.as_str() {
                if !str_val.trim().is_empty() && !self.is_valid_date(str_val) {
                    invalid_values.push(str_val.to_string());
                }
            } else if !value.is_null() {
                invalid_values.push(value.to_string());
            }
        }

        if invalid_values.is_empty() {
            Ok((
                ValidationStatus::Valid,
                "All date values are valid".to_string(),
                vec![],
            ))
        } else {
            Ok((
                ValidationStatus::Invalid,
                format!("Invalid date values found: {}", invalid_values.join(", ")),
                vec![
                    "Use ISO date format (YYYY-MM-DD) or common formats (MM/DD/YYYY, DD/MM/YYYY)".to_string(),
                    "Ensure dates are properly formatted and valid".to_string(),
                ],
            ))
        }
    }

    /// Validate IP address values
    fn validate_ip_address_values(&self, column_data: &[serde_json::Value]) -> Result<(ValidationStatus, String, Vec<String>)> {
        let mut invalid_values = Vec::new();

        for value in column_data {
            if let Some(str_val) = value.as_str() {
                if !str_val.trim().is_empty() && str_val.parse::<std::net::IpAddr>().is_err() {
                    invalid_values.push(str_val.to_string());
                }
            } else if !value.is_null() {
                invalid_values.push(value.to_string());
            }
        }

        if invalid_values.is_empty() {
            Ok((
                ValidationStatus::Valid,
                "All IP address values are valid".to_string(),
                vec![],
            ))
        } else {
            Ok((
                ValidationStatus::Invalid,
                format!("Invalid IP address values found: {}", invalid_values.join(", ")),
                vec![
                    "Use valid IPv4 (e.g., 192.168.1.1) or IPv6 format".to_string(),
                    "Remove any extra whitespace or invalid characters".to_string(),
                ],
            ))
        }
    }

    /// Validate email values
    fn validate_email_values(&self, column_data: &[serde_json::Value]) -> Result<(ValidationStatus, String, Vec<String>)> {
        let mut invalid_values = Vec::new();

        for value in column_data {
            if let Some(str_val) = value.as_str() {
                if !str_val.trim().is_empty() && !self.is_valid_email(str_val) {
                    invalid_values.push(str_val.to_string());
                }
            } else if !value.is_null() {
                invalid_values.push(value.to_string());
            }
        }

        if invalid_values.is_empty() {
            Ok((
                ValidationStatus::Valid,
                "All email values are valid".to_string(),
                vec![],
            ))
        } else {
            Ok((
                ValidationStatus::Invalid,
                format!("Invalid email values found: {}", invalid_values.join(", ")),
                vec![
                    "Use valid email format (user@domain.com)".to_string(),
                    "Ensure all email addresses contain @ and a valid domain".to_string(),
                ],
            ))
        }
    }

    /// Check if a string is a valid date
    fn is_valid_date(&self, date_str: &str) -> bool {
        // Try multiple date formats
        DateTime::parse_from_rfc3339(date_str).is_ok() ||
        NaiveDate::parse_from_str(date_str, "%Y-%m-%d").is_ok() ||
        NaiveDate::parse_from_str(date_str, "%m/%d/%Y").is_ok() ||
        NaiveDate::parse_from_str(date_str, "%d/%m/%Y").is_ok() ||
        NaiveDate::parse_from_str(date_str, "%Y/%m/%d").is_ok()
    }

    /// Check if a string is a valid email
    fn is_valid_email(&self, email: &str) -> bool {
        email.contains('@') && email.contains('.') && email.len() > 5
    }

    /// Validate URL values
    fn validate_url_values(&self, column_data: &[serde_json::Value]) -> Result<(ValidationStatus, String, Vec<String>)> {
        let mut invalid_values = Vec::new();

        for value in column_data {
            if let Some(str_val) = value.as_str() {
                if !str_val.trim().is_empty() && !self.is_valid_url(str_val) {
                    invalid_values.push(str_val.to_string());
                }
            } else if !value.is_null() {
                invalid_values.push(value.to_string());
            }
        }

        if invalid_values.is_empty() {
            Ok((ValidationStatus::Valid, "All URL values are valid".to_string(), vec![]))
        } else {
            Ok((
                ValidationStatus::Invalid,
                format!("Invalid URL values found: {}", invalid_values.join(", ")),
                vec!["Use valid URL format starting with http:// or https://".to_string()],
            ))
        }
    }

    /// Validate unique identifier values
    fn validate_unique_identifier_values(&self, column_data: &[serde_json::Value]) -> Result<(ValidationStatus, String, Vec<String>)> {
        let mut seen_values = std::collections::HashSet::new();
        let mut duplicates = Vec::new();
        let mut invalid_values = Vec::new();

        for value in column_data {
            if let Some(str_val) = value.as_str() {
                if !str_val.trim().is_empty() {
                    if !seen_values.insert(str_val.to_lowercase()) {
                        duplicates.push(str_val.to_string());
                    }
                    // Check if it's a valid identifier format
                    if !str_val.chars().all(|c| c.is_alphanumeric() || c == '-' || c == '_') {
                        invalid_values.push(str_val.to_string());
                    }
                }
            }
        }

        let mut messages = Vec::new();
        let mut suggestions = Vec::new();

        if !duplicates.is_empty() {
            messages.push(format!("Duplicate identifiers found: {}", duplicates.join(", ")));
            suggestions.push("Ensure all identifiers are unique".to_string());
        }

        if !invalid_values.is_empty() {
            messages.push(format!("Invalid identifier format: {}", invalid_values.join(", ")));
            suggestions.push("Use only alphanumeric characters, hyphens, and underscores".to_string());
        }

        if messages.is_empty() {
            Ok((ValidationStatus::Valid, "All unique identifiers are valid".to_string(), vec![]))
        } else {
            Ok((ValidationStatus::Invalid, messages.join("; "), suggestions))
        }
    }

    /// Validate alphanumeric values
    fn validate_alphanumeric_values(&self, column_data: &[serde_json::Value]) -> Result<(ValidationStatus, String, Vec<String>)> {
        let mut invalid_values = Vec::new();

        for value in column_data {
            if let Some(str_val) = value.as_str() {
                if !str_val.trim().is_empty() && !str_val.chars().all(|c| c.is_alphanumeric()) {
                    invalid_values.push(str_val.to_string());
                }
            } else if !value.is_null() {
                invalid_values.push(value.to_string());
            }
        }

        if invalid_values.is_empty() {
            Ok((ValidationStatus::Valid, "All alphanumeric values are valid".to_string(), vec![]))
        } else {
            Ok((
                ValidationStatus::Invalid,
                format!("Non-alphanumeric values found: {}", invalid_values.join(", ")),
                vec!["Use only letters and numbers".to_string()],
            ))
        }
    }

    /// Validate pattern values using regex
    fn validate_pattern_values(
        &mut self,
        rule: &ColumnValidationRule,
        column_data: &[serde_json::Value],
        validation_type: &str,
    ) -> Result<(ValidationStatus, String, Vec<String>)> {
        if let Some(ref pattern_str) = rule.pattern {
            let regex = self.get_or_compile_regex(validation_type, pattern_str)?;
            let mut invalid_values = Vec::new();

            for value in column_data {
                if let Some(str_val) = value.as_str() {
                    if !str_val.trim().is_empty() && !regex.is_match(str_val) {
                        invalid_values.push(str_val.to_string());
                    }
                }
            }

            if invalid_values.is_empty() {
                Ok((ValidationStatus::Valid, "All pattern values are valid".to_string(), vec![]))
            } else {
                Ok((
                    ValidationStatus::Invalid,
                    format!("Values not matching pattern: {}", invalid_values.join(", ")),
                    vec![format!("Ensure values match the required pattern: {}", pattern_str)],
                ))
            }
        } else {
            Ok((ValidationStatus::Skipped, "No pattern defined for validation".to_string(), vec![]))
        }
    }

    /// Validate enumeration values
    fn validate_enumeration_values(
        &self,
        rule: &ColumnValidationRule,
        column_data: &[serde_json::Value],
        _validation_type: &str,
    ) -> Result<(ValidationStatus, String, Vec<String>)> {
        if let Some(ref allowed_values) = rule.allowed_values {
            let allowed_lower: Vec<String> = allowed_values.iter().map(|v| v.to_lowercase()).collect();
            let mut invalid_values = Vec::new();

            for value in column_data {
                if let Some(str_val) = value.as_str() {
                    if !str_val.trim().is_empty() && !allowed_lower.contains(&str_val.to_lowercase()) {
                        invalid_values.push(str_val.to_string());
                    }
                }
            }

            if invalid_values.is_empty() {
                Ok((ValidationStatus::Valid, "All enumeration values are valid".to_string(), vec![]))
            } else {
                Ok((
                    ValidationStatus::Invalid,
                    format!("Invalid enumeration values: {}", invalid_values.join(", ")),
                    vec![format!("Use one of the allowed values: {}", allowed_values.join(", "))],
                ))
            }
        } else {
            Ok((ValidationStatus::Skipped, "No allowed values defined for enumeration".to_string(), vec![]))
        }
    }

    /// Get or compile regex pattern with caching
    fn get_or_compile_regex(&mut self, validation_type: &str, pattern: &str) -> Result<&Regex> {
        if !self.regex_cache.contains_key(validation_type) {
            let regex = Regex::new(pattern).map_err(|e| {
                Error::document_parsing(format!("Invalid regex pattern '{}': {}", pattern, e))
            })?;
            self.regex_cache.insert(validation_type.to_string(), regex);
        }
        Ok(self.regex_cache.get(validation_type).unwrap())
    }

    /// Check if a string is a valid URL
    fn is_valid_url(&self, url: &str) -> bool {
        url.starts_with("http://") || url.starts_with("https://")
    }

    /// Suggest alternative column names for missing fields
    fn suggest_alternatives(&self, expected_names: &[String], detected_columns: &[String]) -> Vec<String> {
        let mut suggestions = Vec::new();

        for expected in expected_names {
            for detected in detected_columns {
                // Calculate simple similarity
                let similarity = self.calculate_similarity(expected, detected);
                if similarity > 0.6 {
                    suggestions.push(format!("Consider '{}' (similarity: {:.1}%)", detected, similarity * 100.0));
                }
            }
        }

        // Limit suggestions to top 3
        suggestions.truncate(3);
        suggestions
    }

    /// Calculate simple string similarity
    fn calculate_similarity(&self, s1: &str, s2: &str) -> f64 {
        let s1_lower = s1.to_lowercase();
        let s2_lower = s2.to_lowercase();

        // Simple Jaccard similarity based on character n-grams
        let s1_chars: std::collections::HashSet<char> = s1_lower.chars().collect();
        let s2_chars: std::collections::HashSet<char> = s2_lower.chars().collect();

        let intersection = s1_chars.intersection(&s2_chars).count();
        let union = s1_chars.union(&s2_chars).count();

        if union == 0 {
            0.0
        } else {
            intersection as f64 / union as f64
        }
    }

    /// Create enumeration failure info
    fn create_enumeration_failure(
        &self,
        rule: &ColumnValidationRule,
        column_name: &str,
        sample_data: &HashMap<String, Vec<serde_json::Value>>,
    ) -> Result<EnumerationFailureInfo> {
        let mut invalid_values = Vec::new();
        let mut suggested_mappings = HashMap::new();

        if let Some(column_data) = sample_data.get(column_name) {
            if let Some(ref allowed_values) = rule.allowed_values {
                let allowed_lower: Vec<String> = allowed_values.iter().map(|v| v.to_lowercase()).collect();

                for value in column_data {
                    if let Some(str_val) = value.as_str() {
                        if !str_val.trim().is_empty() && !allowed_lower.contains(&str_val.to_lowercase()) {
                            invalid_values.push(str_val.to_string());

                            // Find best match for suggestion
                            let mut best_match = None;
                            let mut best_similarity = 0.0;

                            for allowed in allowed_values {
                                let similarity = self.calculate_similarity(str_val, allowed);
                                if similarity > best_similarity && similarity > 0.5 {
                                    best_similarity = similarity;
                                    best_match = Some(allowed.clone());
                                }
                            }

                            if let Some(suggestion) = best_match {
                                suggested_mappings.insert(str_val.to_string(), suggestion);
                            }
                        }
                    }
                }
            }
        }

        Ok(EnumerationFailureInfo {
            field_id: rule.field_id.clone(),
            column_name: column_name.to_string(),
            invalid_values,
            allowed_values: rule.allowed_values.clone().unwrap_or_default(),
            suggested_mappings,
        })
    }

    /// Create type mismatch info
    fn create_type_mismatch(
        &self,
        rule: &ColumnValidationRule,
        column_name: &str,
        sample_data: &HashMap<String, Vec<serde_json::Value>>,
    ) -> Result<TypeMismatchInfo> {
        let mut sample_values = Vec::new();
        let expected_type = rule.data_type.clone().unwrap_or_else(|| {
            rule.validation_type.clone().unwrap_or("string".to_string())
        });

        if let Some(column_data) = sample_data.get(column_name) {
            // Take up to 3 sample values that don't match the expected type
            for value in column_data.iter().take(10) {
                if let Some(str_val) = value.as_str() {
                    if !str_val.trim().is_empty() {
                        sample_values.push(str_val.to_string());
                        if sample_values.len() >= 3 {
                            break;
                        }
                    }
                }
            }
        }

        let actual_type = if sample_values.is_empty() {
            "empty".to_string()
        } else {
            "string".to_string() // Simplified type detection
        };

        let suggested_conversion = match expected_type.as_str() {
            "boolean" => Some("Convert to yes/no or true/false format".to_string()),
            "date" => Some("Use ISO date format (YYYY-MM-DD) or MM/DD/YYYY".to_string()),
            "number" => Some("Remove non-numeric characters and use decimal format".to_string()),
            _ => None,
        };

        Ok(TypeMismatchInfo {
            field_id: rule.field_id.clone(),
            column_name: column_name.to_string(),
            expected_type,
            actual_type,
            sample_values,
            suggested_conversion,
        })
    }

    /// Validate cross-field rules
    fn validate_cross_field_rules(
        &self,
        document_type: &str,
        _detected_columns: &[String],
        _sample_data: &HashMap<String, Vec<serde_json::Value>>,
    ) -> Result<Vec<CrossFieldValidationResult>> {
        let mut results = Vec::new();

        // Example cross-field validation rules
        match document_type.to_lowercase().as_str() {
            "inventory" => {
                // Add inventory-specific cross-field validations
                results.push(CrossFieldValidationResult {
                    rule_name: "asset_id_uniqueness".to_string(),
                    involved_fields: vec!["asset_id".to_string()],
                    passed: true, // Placeholder - implement actual logic
                    message: "Asset IDs should be unique across the inventory".to_string(),
                    severity: ValidationSeverity::Warning,
                });
            }
            "poam" => {
                // Add POA&M-specific cross-field validations
                results.push(CrossFieldValidationResult {
                    rule_name: "completion_date_consistency".to_string(),
                    involved_fields: vec!["scheduled_completion".to_string(), "actual_completion".to_string()],
                    passed: true, // Placeholder - implement actual logic
                    message: "Actual completion date should not be before scheduled date".to_string(),
                    severity: ValidationSeverity::Warning,
                });
            }
            _ => {}
        }

        Ok(results)
    }

    /// Calculate validation metrics
    fn calculate_validation_metrics(
        &self,
        field_results: &[ColumnValidationResult],
        cross_field_results: &[CrossFieldValidationResult],
    ) -> ValidationMetrics {
        let total_fields = field_results.len();
        let required_fields = field_results.iter().filter(|r| r.severity == ValidationSeverity::Error).count();
        let optional_fields = total_fields - required_fields;
        let valid_fields = field_results.iter().filter(|r| r.passed).count();
        let missing_required_count = field_results.iter()
            .filter(|r| r.status == ValidationStatus::MissingRequired)
            .count();
        let invalid_fields = field_results.iter()
            .filter(|r| r.status == ValidationStatus::Invalid)
            .count();
        let warning_count = field_results.iter()
            .filter(|r| r.severity == ValidationSeverity::Warning)
            .count() + cross_field_results.iter()
            .filter(|r| r.severity == ValidationSeverity::Warning)
            .count();
        let error_count = field_results.iter()
            .filter(|r| r.severity == ValidationSeverity::Error && !r.passed)
            .count() + cross_field_results.iter()
            .filter(|r| r.severity == ValidationSeverity::Error && !r.passed)
            .count();

        let validation_score = if total_fields == 0 {
            1.0
        } else {
            valid_fields as f64 / total_fields as f64
        };

        ValidationMetrics {
            total_fields,
            required_fields,
            optional_fields,
            valid_fields,
            missing_required_count,
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
            min_quality_threshold: 0.8,
        }
    }

    /// Create a new document validator with custom quality threshold
    #[must_use]
    pub fn with_quality_threshold(min_quality_threshold: f64) -> Self {
        Self {
            rules: HashMap::new(),
            min_quality_threshold,
        }
    }

    /// Load validation rules from configuration
    pub fn load_rules(&mut self, rules: HashMap<String, Vec<ValidationRule>>) -> Result<()> {
        info!("Loading validation rules for {} document types", rules.len());
        self.rules = rules;
        Ok(())
    }

    /// Validate document content
    pub fn validate_document(
        &self,
        document_type: &str,
        content: &serde_json::Value,
    ) -> Result<Vec<ValidationResult>> {
        debug!("Validating document of type: {}", document_type);
        
        let mut results = Vec::new();
        
        if let Some(document_rules) = self.rules.get(document_type) {
            for rule in document_rules {
                let validation_result = self.apply_rule(rule, content)?;
                results.push(validation_result);
            }
        } else {
            warn!("No validation rules found for document type: {}", document_type);
        }
        
        info!("Completed validation with {} results", results.len());
        Ok(results)
    }

    /// Apply a single validation rule
    fn apply_rule(
        &self,
        rule: &ValidationRule,
        content: &serde_json::Value,
    ) -> Result<ValidationResult> {
        let field_value = self.extract_field_value(content, &rule.field_name);
        
        let passed = match rule.rule_type.as_str() {
            "required" => self.validate_required(&field_value),
            "format" => self.validate_format(&field_value, &rule.parameters)?,
            "range" => self.validate_range(&field_value, &rule.parameters)?,
            "length" => self.validate_length(&field_value, &rule.parameters)?,
            "pattern" => self.validate_pattern(&field_value, &rule.parameters)?,
            _ => {
                warn!("Unknown validation rule type: {}", rule.rule_type);
                true // Skip unknown rules
            }
        };

        Ok(ValidationResult {
            field_name: rule.field_name.clone(),
            rule_type: rule.rule_type.clone(),
            passed,
            message: if passed { None } else { Some(rule.error_message.clone()) },
            severity: rule.severity.clone(),
        })
    }

    /// Extract field value from content using dot notation
    fn extract_field_value<'a>(&self, content: &'a serde_json::Value, field_path: &str) -> Option<&'a serde_json::Value> {
        let parts: Vec<&str> = field_path.split('.').collect();
        let mut current = content;
        
        for part in parts {
            current = current.get(part)?;
        }
        
        Some(current)
    }

    /// Validate that a field is present and not null/empty
    fn validate_required(&self, value: &Option<&serde_json::Value>) -> bool {
        match value {
            Some(serde_json::Value::Null) => false,
            Some(serde_json::Value::String(s)) => !s.trim().is_empty(),
            Some(serde_json::Value::Array(arr)) => !arr.is_empty(),
            Some(serde_json::Value::Object(obj)) => !obj.is_empty(),
            Some(_) => true,
            None => false,
        }
    }

    /// Validate field format (email, date, etc.)
    fn validate_format(
        &self,
        value: &Option<&serde_json::Value>,
        parameters: &serde_json::Value,
    ) -> Result<bool> {
        let Some(value) = value else { return Ok(true) }; // Skip validation if field is missing
        let Some(format_type) = parameters.get("type").and_then(|t| t.as_str()) else {
            return Ok(true);
        };

        match format_type {
            "email" => Ok(self.validate_email_format(value)),
            "date" => Ok(self.validate_date_format(value)),
            "url" => Ok(self.validate_url_format(value)),
            "ip" => Ok(self.validate_ip_format(value)),
            _ => {
                warn!("Unknown format type: {}", format_type);
                Ok(true)
            }
        }
    }

    /// Validate numeric range
    fn validate_range(
        &self,
        value: &Option<&serde_json::Value>,
        parameters: &serde_json::Value,
    ) -> Result<bool> {
        let Some(value) = value else { return Ok(true) };
        let Some(num) = value.as_f64() else { return Ok(false) };

        let min = parameters.get("min").and_then(|v| v.as_f64());
        let max = parameters.get("max").and_then(|v| v.as_f64());

        let within_min = min.map_or(true, |m| num >= m);
        let within_max = max.map_or(true, |m| num <= m);

        Ok(within_min && within_max)
    }

    /// Validate string length
    fn validate_length(
        &self,
        value: &Option<&serde_json::Value>,
        parameters: &serde_json::Value,
    ) -> Result<bool> {
        let Some(value) = value else { return Ok(true) };
        let Some(text) = value.as_str() else { return Ok(false) };

        let min_len = parameters.get("min").and_then(|v| v.as_u64()).unwrap_or(0) as usize;
        let max_len = parameters.get("max").and_then(|v| v.as_u64()).map(|v| v as usize);

        let len = text.len();
        let within_min = len >= min_len;
        let within_max = max_len.map_or(true, |m| len <= m);

        Ok(within_min && within_max)
    }

    /// Validate against regex pattern
    fn validate_pattern(
        &self,
        value: &Option<&serde_json::Value>,
        parameters: &serde_json::Value,
    ) -> Result<bool> {
        let Some(value) = value else { return Ok(true) };
        let Some(text) = value.as_str() else { return Ok(false) };
        let Some(pattern) = parameters.get("pattern").and_then(|p| p.as_str()) else {
            return Ok(true);
        };

        // TODO: Implement regex validation
        // For now, just check if pattern is contained in text
        Ok(text.contains(pattern))
    }

    /// Validate email format
    fn validate_email_format(&self, value: &serde_json::Value) -> bool {
        if let Some(email) = value.as_str() {
            email.contains('@') && email.contains('.')
        } else {
            false
        }
    }

    /// Validate date format
    fn validate_date_format(&self, value: &serde_json::Value) -> bool {
        if let Some(date_str) = value.as_str() {
            // Basic date format validation
            chrono::DateTime::parse_from_rfc3339(date_str).is_ok()
                || chrono::NaiveDate::parse_from_str(date_str, "%Y-%m-%d").is_ok()
        } else {
            false
        }
    }

    /// Validate URL format
    fn validate_url_format(&self, value: &serde_json::Value) -> bool {
        if let Some(url) = value.as_str() {
            url.starts_with("http://") || url.starts_with("https://")
        } else {
            false
        }
    }

    /// Validate IP address format
    fn validate_ip_format(&self, value: &serde_json::Value) -> bool {
        if let Some(ip) = value.as_str() {
            ip.parse::<std::net::IpAddr>().is_ok()
        } else {
            false
        }
    }

    /// Calculate quality metrics for validation results
    pub fn calculate_quality_metrics(
        &self,
        validation_results: &[ValidationResult],
        total_fields: usize,
        populated_fields: usize,
    ) -> QualityMetrics {
        let error_count = validation_results
            .iter()
            .filter(|r| !r.passed && r.severity == "error")
            .count();
        
        let warning_count = validation_results
            .iter()
            .filter(|r| !r.passed && r.severity == "warning")
            .count();

        let completeness_score = if total_fields > 0 {
            populated_fields as f64 / total_fields as f64
        } else {
            1.0
        };

        let accuracy_score = if !validation_results.is_empty() {
            validation_results.iter().filter(|r| r.passed).count() as f64 / validation_results.len() as f64
        } else {
            1.0
        };

        // Simple consistency score (can be enhanced with cross-field validation)
        let consistency_score = if error_count == 0 { 1.0 } else { 0.8 };

        let overall_score = (completeness_score + accuracy_score + consistency_score) / 3.0;

        QualityMetrics {
            overall_score,
            completeness_score,
            consistency_score,
            accuracy_score,
            total_fields,
            populated_fields,
            error_count,
            warning_count,
        }
    }

    /// Check if document meets quality threshold
    pub fn meets_quality_threshold(&self, metrics: &QualityMetrics) -> bool {
        metrics.overall_score >= self.min_quality_threshold
    }
}

impl Default for DocumentValidator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_document_validator_creation() {
        let validator = DocumentValidator::new();
        assert_eq!(validator.min_quality_threshold, 0.8);
    }

    #[test]
    fn test_column_validator_creation() {
        let mapping_config = MappingConfiguration {
            inventory_mappings: None,
            poam_mappings: None,
            ssp_sections: None,
            controls: None,
            documents: None,
        };

        let validator = ColumnValidator::new(mapping_config);
        assert_eq!(validator.min_quality_threshold, 0.8);
        assert_eq!(validator.performance_target_ms, 50);
    }

    #[test]
    fn test_validation_status_ordering() {
        assert!(ValidationSeverity::Critical > ValidationSeverity::Error);
        assert!(ValidationSeverity::Error > ValidationSeverity::Warning);
        assert!(ValidationSeverity::Warning > ValidationSeverity::Info);
    }

    #[test]
    fn test_boolean_validation() {
        let validator = ColumnValidator::new(MappingConfiguration {
            inventory_mappings: None,
            poam_mappings: None,
            ssp_sections: None,
            controls: None,
            documents: None,
        });

        let valid_data = vec![
            json!("yes"),
            json!("no"),
            json!("true"),
            json!("false"),
            json!("1"),
            json!("0"),
        ];

        let result = validator.validate_boolean_values(&valid_data).unwrap();
        assert_eq!(result.0, ValidationStatus::Valid);

        let invalid_data = vec![
            json!("maybe"),
            json!("invalid"),
            json!("2"),
        ];

        let result = validator.validate_boolean_values(&invalid_data).unwrap();
        assert_eq!(result.0, ValidationStatus::Invalid);
    }

    #[test]
    fn test_date_validation() {
        let validator = ColumnValidator::new(MappingConfiguration {
            inventory_mappings: None,
            poam_mappings: None,
            ssp_sections: None,
            controls: None,
            documents: None,
        });

        let valid_data = vec![
            json!("2023-12-25"),
            json!("12/25/2023"),
            json!("25/12/2023"),
        ];

        let result = validator.validate_date_values(&valid_data).unwrap();
        assert_eq!(result.0, ValidationStatus::Valid);

        let invalid_data = vec![
            json!("invalid-date"),
            json!("2023-13-45"),
            json!("not a date"),
        ];

        let result = validator.validate_date_values(&invalid_data).unwrap();
        assert_eq!(result.0, ValidationStatus::Invalid);
    }

    #[test]
    fn test_ip_address_validation() {
        let validator = ColumnValidator::new(MappingConfiguration {
            inventory_mappings: None,
            poam_mappings: None,
            ssp_sections: None,
            controls: None,
            documents: None,
        });

        let valid_data = vec![
            json!("192.168.1.1"),
            json!("10.0.0.1"),
            json!("2001:db8::1"),
        ];

        let result = validator.validate_ip_address_values(&valid_data).unwrap();
        assert_eq!(result.0, ValidationStatus::Valid);

        let invalid_data = vec![
            json!("256.256.256.256"),
            json!("not.an.ip.address"),
            json!("192.168.1"),
        ];

        let result = validator.validate_ip_address_values(&invalid_data).unwrap();
        assert_eq!(result.0, ValidationStatus::Invalid);
    }

    #[test]
    fn test_email_validation() {
        let validator = ColumnValidator::new(MappingConfiguration {
            inventory_mappings: None,
            poam_mappings: None,
            ssp_sections: None,
            controls: None,
            documents: None,
        });

        let valid_data = vec![
            json!("user@example.com"),
            json!("test.email@domain.org"),
            json!("admin@company.gov"),
        ];

        let result = validator.validate_email_values(&valid_data).unwrap();
        assert_eq!(result.0, ValidationStatus::Valid);

        let invalid_data = vec![
            json!("invalid-email"),
            json!("@domain.com"),
            json!("user@"),
        ];

        let result = validator.validate_email_values(&invalid_data).unwrap();
        assert_eq!(result.0, ValidationStatus::Invalid);
    }

    #[test]
    fn test_unique_identifier_validation() {
        let validator = ColumnValidator::new(MappingConfiguration {
            inventory_mappings: None,
            poam_mappings: None,
            ssp_sections: None,
            controls: None,
            documents: None,
        });

        let valid_data = vec![
            json!("asset-001"),
            json!("component_123"),
            json!("system-abc"),
        ];

        let result = validator.validate_unique_identifier_values(&valid_data).unwrap();
        assert_eq!(result.0, ValidationStatus::Valid);

        let duplicate_data = vec![
            json!("asset-001"),
            json!("asset-002"),
            json!("asset-001"), // Duplicate
        ];

        let result = validator.validate_unique_identifier_values(&duplicate_data).unwrap();
        assert_eq!(result.0, ValidationStatus::Invalid);
        assert!(result.1.contains("Duplicate"));
    }

    #[test]
    fn test_similarity_calculation() {
        let validator = ColumnValidator::new(MappingConfiguration {
            inventory_mappings: None,
            poam_mappings: None,
            ssp_sections: None,
            controls: None,
            documents: None,
        });

        let similarity = validator.calculate_similarity("Asset Name", "Asset_Name");
        assert!(similarity > 0.6, "Expected similarity > 0.6, got {}", similarity);

        let similarity = validator.calculate_similarity("Asset Name", "Component Type");
        assert!(similarity < 0.5, "Expected similarity < 0.5, got {}", similarity);

        // Test exact match
        let similarity = validator.calculate_similarity("Asset Name", "Asset Name");
        assert_eq!(similarity, 1.0);

        // Test completely different strings
        let similarity = validator.calculate_similarity("Asset", "xyz123");
        assert!(similarity < 0.3);
    }
}
