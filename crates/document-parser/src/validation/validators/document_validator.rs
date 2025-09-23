// Modified: 2025-09-22

//! Document-level validation implementation
//!
//! This module contains the DocumentValidator implementation for comprehensive
//! document validation including quality metrics and performance tracking.

use super::types::*;
use crate::{Result};
use super::super::types::*;
use super::super::rules::{ValidationRule, DataType, ValidationType};
use std::collections::HashMap;
use std::time::{Duration, Instant};
use tracing::{debug, info, warn};

impl DocumentValidator {
    /// Create a new document validator
    #[must_use]
    pub fn new() -> Self {
        Self {
            rules: HashMap::new(),
            column_validator: None,
            min_quality_threshold: 0.8,
            performance_metrics: HashMap::new(),
        }
    }

    /// Create a document validator with custom configuration
    pub fn with_config(config: DocumentValidationConfig) -> Self {
        Self {
            rules: HashMap::new(),
            column_validator: None,
            min_quality_threshold: config.min_quality_threshold,
            performance_metrics: HashMap::new(),
        }
    }

    /// Create a document validator with column validator
    pub fn with_column_validator(column_validator: ColumnValidator) -> Self {
        Self {
            rules: HashMap::new(),
            column_validator: Some(column_validator),
            min_quality_threshold: 0.8,
            performance_metrics: HashMap::new(),
        }
    }

    /// Create a document validator with both column validator and configuration
    pub fn with_column_validator_and_config(
        column_validator: ColumnValidator,
        config: DocumentValidationConfig,
    ) -> Self {
        Self {
            rules: HashMap::new(),
            column_validator: Some(column_validator),
            min_quality_threshold: config.min_quality_threshold,
            performance_metrics: HashMap::new(),
        }
    }

    /// Add validation rules for a specific field
    pub fn add_validation_rules(&mut self, field_id: String, rules: Vec<ValidationRule>) {
        self.rules.insert(field_id, rules);
    }

    /// Remove validation rules for a field
    pub fn remove_validation_rules(&mut self, field_id: &str) -> Option<Vec<ValidationRule>> {
        self.rules.remove(field_id)
    }

    /// Get validation rules for a field
    pub fn get_validation_rules(&self, field_id: &str) -> Option<&Vec<ValidationRule>> {
        self.rules.get(field_id)
    }

    /// Set the column validator
    pub fn set_column_validator(&mut self, column_validator: ColumnValidator) {
        self.column_validator = Some(column_validator);
    }

    /// Remove the column validator
    pub fn remove_column_validator(&mut self) -> Option<ColumnValidator> {
        self.column_validator.take()
    }

    /// Get a reference to the column validator
    pub fn get_column_validator(&self) -> Option<&ColumnValidator> {
        self.column_validator.as_ref()
    }

    /// Get the minimum quality threshold
    pub fn get_min_quality_threshold(&self) -> f64 {
        self.min_quality_threshold
    }

    /// Set the minimum quality threshold
    pub fn set_min_quality_threshold(&mut self, threshold: f64) {
        self.min_quality_threshold = threshold.clamp(0.0, 1.0);
    }

    /// Validate a complete document
    pub fn validate_document(
        &mut self,
        document_data: &HashMap<String, Vec<serde_json::Value>>,
        expected_schema: &HashMap<String, DataType>,
    ) -> Result<DocumentValidationResult> {
        let start_time = Instant::now();
        debug!("Starting document validation");

        let mut field_results = Vec::new();
        let mut overall_passed = true;
        let mut total_validation_time = Duration::new(0, 0);

        // Validate each field if column validator is available
        if let Some(ref column_validator) = self.column_validator {
            for (field_id, expected_type) in expected_schema {
                if let Some(column_data) = document_data.get(field_id.as_str()) {
                    let field_start = Instant::now();
                    
                    match column_validator.validate_column(
                        field_id,
                        field_id, // Using field_id as source_column for simplicity
                        column_data,
                        expected_type,
                    ) {
                        Ok(result) => {
                            if !result.passed {
                                overall_passed = false;
                            }
                            field_results.push(result);
                        }
                        Err(e) => {
                            warn!("Failed to validate field '{}': {}", field_id, e);
                            overall_passed = false;
                            
                            // Create a failed result
                            field_results.push(ColumnValidationResult {
                                field_id: field_id.clone(),
                                source_column: field_id.clone(),
                                passed: false,
                                status: ValidationStatus::Invalid,
                                severity: ValidationSeverity::Error,
                                message: format!("Validation failed: {}", e),
                                expected_type: Some(format!("{:?}", expected_type)),
                                actual_type: Some("Unknown".to_string()),
                                sample_invalid_values: Vec::new(),
                                validation_time_us: 0,
                            });
                        }
                    }
                    
                    total_validation_time += field_start.elapsed();
                } else {
                    warn!("Field '{}' not found in document data", field_id);
                    overall_passed = false;
                    
                    // Create a missing field result
                    field_results.push(ColumnValidationResult {
                        field_id: field_id.clone(),
                        source_column: field_id.clone(),
                        passed: false,
                        status: ValidationStatus::Invalid,
                        severity: ValidationSeverity::Error,
                        message: "Field not found in document".to_string(),
                        expected_type: Some(format!("{:?}", expected_type)),
                        actual_type: Some("Missing".to_string()),
                        sample_invalid_values: Vec::new(),
                        validation_time_us: 0,
                    });
                }
            }
        }

        // Apply custom validation rules
        for (field_id, rules) in &self.rules {
            if let Some(column_data) = document_data.get(field_id.as_str()) {
                for rule in rules {
                    let rule_start = Instant::now();
                    
                    match self.apply_validation_rule(field_id, column_data, rule) {
                        Ok(passed) => {
                            if !passed {
                                overall_passed = false;
                                info!("Validation rule '{}' failed for field '{}'", rule.field_name, field_id);
                            }
                        }
                        Err(e) => {
                            warn!("Failed to apply validation rule '{}' for field '{}': {}", rule.field_name, field_id, e);
                            overall_passed = false;
                        }
                    }
                    
                    total_validation_time += rule_start.elapsed();
                }
            }
        }

        let total_time = start_time.elapsed();
        
        // Update performance metrics
        self.performance_metrics.insert("last_validation".to_string(), total_time);
        self.performance_metrics.insert("field_validation".to_string(), total_validation_time);

        // Calculate quality metrics
        let quality_metrics = self.calculate_quality_metrics(&field_results);
        let meets_threshold = self.meets_quality_threshold(&quality_metrics);

        debug!("Document validation completed in {:?}", total_time);

        // Generate summary before moving field_results
        let summary = self.generate_validation_summary(&field_results, overall_passed, meets_threshold);

        Ok(DocumentValidationResult {
            passed: overall_passed && meets_threshold,
            field_results,
            quality_metrics,
            validation_time_ms: total_time.as_millis() as u64,
            meets_quality_threshold: meets_threshold,
            summary,
        })
    }

    /// Apply a single validation rule to field data
    fn apply_validation_rule(
        &self,
        field_id: &str,
        column_data: &[serde_json::Value],
        rule: &ValidationRule,
    ) -> Result<bool> {
        debug!("Applying validation rule '{}' to field '{}'", rule.field_name, field_id);

        // This is a simplified implementation
        // In a real implementation, you would have different rule types and logic
        match rule.validation_type {
            ValidationType::Presence => {
                // Check if field has non-empty values
                Ok(!column_data.is_empty() && column_data.iter().any(|v| !v.is_null()))
            }
            ValidationType::Pattern => {
                // Check pattern matching for string values
                if let Some(pattern) = rule.parameters.get("pattern").and_then(|v| v.as_str()) {
                    Ok(column_data.iter().all(|v| {
                        match v {
                            serde_json::Value::String(s) => {
                                regex::Regex::new(pattern)
                                    .map(|re| re.is_match(s))
                                    .unwrap_or(false)
                            }
                            serde_json::Value::Null => true, // Null values pass pattern checks
                            _ => false,
                        }
                    }))
                } else {
                    Err(crate::Error::validation("pattern rule missing pattern parameter".to_string()))
                }
            }
            ValidationType::Range => {
                // Check numeric range for values
                let min_value = rule.parameters.get("min_value").and_then(|v| v.as_f64());
                let max_value = rule.parameters.get("max_value").and_then(|v| v.as_f64());

                Ok(column_data.iter().all(|v| {
                    match v {
                        serde_json::Value::Number(n) => {
                            if let Some(num) = n.as_f64() {
                                let min_ok = min_value.map_or(true, |min| num >= min);
                                let max_ok = max_value.map_or(true, |max| num <= max);
                                min_ok && max_ok
                            } else {
                                false
                            }
                        }
                        serde_json::Value::Null => true, // Null values pass range checks
                        _ => false,
                    }
                }))
            }
            ValidationType::Email => {
                // Use email validation from helpers
                Ok(column_data.iter().all(|v| {
                    match v {
                        serde_json::Value::String(s) => super::validation_helpers::ValidationHelpers::is_valid_email(s),
                        serde_json::Value::Null => true,
                        _ => false,
                    }
                }))
            }
            ValidationType::Url => {
                // Use URL validation from helpers
                Ok(column_data.iter().all(|v| {
                    match v {
                        serde_json::Value::String(s) => super::validation_helpers::ValidationHelpers::is_valid_url(s),
                        serde_json::Value::Null => true,
                        _ => false,
                    }
                }))
            }
            ValidationType::IpAddress => {
                // Use IP address validation from helpers
                Ok(column_data.iter().all(|v| {
                    match v {
                        serde_json::Value::String(s) => super::validation_helpers::ValidationHelpers::is_valid_ip_address(s),
                        serde_json::Value::Null => true,
                        _ => false,
                    }
                }))
            }
            ValidationType::Uuid => {
                // Use UUID validation from helpers
                Ok(column_data.iter().all(|v| {
                    match v {
                        serde_json::Value::String(s) => super::validation_helpers::ValidationHelpers::is_valid_uuid(s),
                        serde_json::Value::Null => true,
                        _ => false,
                    }
                }))
            }
            ValidationType::DateFormat => {
                // Use date validation from helpers
                Ok(column_data.iter().all(|v| {
                    match v {
                        serde_json::Value::String(s) => super::validation_helpers::ValidationHelpers::is_valid_date(s),
                        serde_json::Value::Null => true,
                        _ => false,
                    }
                }))
            }
            _ => {
                warn!("Validation rule type not implemented: {:?}", rule.validation_type);
                Ok(true) // Unknown rules pass by default
            }
        }
    }

    /// Calculate quality metrics based on field validation results
    fn calculate_quality_metrics(&self, field_results: &[ColumnValidationResult]) -> QualityMetrics {
        if field_results.is_empty() {
            return QualityMetrics {
                completeness_score: 1.0,
                accuracy_score: 1.0,
                consistency_score: 1.0,
                overall_quality_score: 1.0,
                risk_level: super::super::types::RiskLevel::Low,
                quality_grade: super::super::types::QualityGrade::A,
                compliance_percentage: 100.0,
                critical_issues: 0,
                warnings: 0,
            };
        }

        let total_fields = field_results.len();
        let passed_fields = field_results.iter().filter(|r| r.passed).count();
        let failed_fields = total_fields - passed_fields;

        let overall_quality_score = passed_fields as f64 / total_fields as f64;

        // Count critical issues and warnings based on severity
        let critical_issues = field_results.iter()
            .filter(|r| !r.passed && r.severity == ValidationSeverity::Error)
            .count();
        let warnings = field_results.iter()
            .filter(|r| !r.passed && r.severity == ValidationSeverity::Warning)
            .count();

        // Determine risk level based on failure rate
        let risk_level = if overall_quality_score >= 0.9 {
            super::super::types::RiskLevel::Low
        } else if overall_quality_score >= 0.7 {
            super::super::types::RiskLevel::Medium
        } else {
            super::super::types::RiskLevel::High
        };

        // Determine quality grade
        let quality_grade = if overall_quality_score >= 0.95 {
            super::super::types::QualityGrade::A
        } else if overall_quality_score >= 0.85 {
            super::super::types::QualityGrade::B
        } else if overall_quality_score >= 0.75 {
            super::super::types::QualityGrade::C
        } else if overall_quality_score >= 0.65 {
            super::super::types::QualityGrade::D
        } else {
            super::super::types::QualityGrade::F
        };

        QualityMetrics {
            completeness_score: overall_quality_score,
            accuracy_score: overall_quality_score,
            consistency_score: overall_quality_score,
            overall_quality_score,
            risk_level,
            quality_grade,
            compliance_percentage: overall_quality_score * 100.0,
            critical_issues,
            warnings,
        }
    }

    /// Generate a validation summary
    fn generate_validation_summary(
        &self,
        field_results: &[ColumnValidationResult],
        overall_passed: bool,
        meets_threshold: bool,
    ) -> String {
        let total_fields = field_results.len();
        let passed_fields = field_results.iter().filter(|r| r.passed).count();
        let failed_fields = total_fields - passed_fields;

        format!(
            "Document validation: {} fields total, {} passed, {} failed. Overall: {}. Quality threshold: {}",
            total_fields,
            passed_fields,
            failed_fields,
            if overall_passed { "PASSED" } else { "FAILED" },
            if meets_threshold { "MET" } else { "NOT MET" }
        )
    }

    /// Check if document meets quality threshold
    pub fn meets_quality_threshold(&self, metrics: &QualityMetrics) -> bool {
        metrics.overall_quality_score >= self.min_quality_threshold
    }

    /// Get performance metrics
    pub fn get_performance_metrics(&self) -> &HashMap<String, Duration> {
        &self.performance_metrics
    }

    /// Clear performance metrics
    pub fn clear_performance_metrics(&mut self) {
        self.performance_metrics.clear();
    }

    /// Get the number of validation rules
    pub fn get_rule_count(&self) -> usize {
        self.rules.values().map(|rules| rules.len()).sum()
    }

    /// Check if the validator has any rules
    pub fn has_rules(&self) -> bool {
        !self.rules.is_empty()
    }

    /// List all field IDs with validation rules
    pub fn list_fields_with_rules(&self) -> Vec<&str> {
        self.rules.keys().map(|s| s.as_str()).collect()
    }
}

impl Default for DocumentValidator {
    fn default() -> Self {
        Self::new()
    }
}
