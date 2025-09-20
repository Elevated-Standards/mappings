// Modified: 2025-01-20

//! Document validation and quality assurance
//!
//! This module provides comprehensive validation for parsed documents
//! including data completeness, consistency, and quality scoring.

use fedramp_core::{Result, Error};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tracing::{debug, info, warn};

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

/// Document validator for quality assurance
#[derive(Debug, Clone)]
pub struct DocumentValidator {
    /// Validation rules loaded from configuration
    rules: HashMap<String, Vec<ValidationRule>>,
    /// Minimum quality threshold
    min_quality_threshold: f64,
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
