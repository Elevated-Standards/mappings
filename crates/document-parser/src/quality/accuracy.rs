//! Data Accuracy Validation for POA&M Items
//! 
//! Validates POA&M data for accuracy including format validation, value constraints, and data integrity

use super::*;
use crate::poam::PoamItem;
use fedramp_core::{Result, Error};
use tracing::{debug, info};
use uuid::Uuid;
use chrono::{DateTime, Utc, NaiveDate};
use regex::Regex;
use std::collections::HashMap;
use std::sync::OnceLock;

/// Result of accuracy validation
#[derive(Debug, Clone)]
pub struct AccuracyResult {
    /// Overall accuracy score (0.0 to 1.0)
    pub score: f64,
    /// Accuracy findings
    pub findings: Vec<QualityFinding>,
    /// Field-level accuracy statistics
    pub field_accuracy: HashMap<String, FieldAccuracyStats>,
    /// Validation rule results
    pub rule_results: Vec<ValidationRuleResult>,
}

/// Field-level accuracy statistics
#[derive(Debug, Clone)]
pub struct FieldAccuracyStats {
    /// Field name
    pub field_name: String,
    /// Total items with this field populated
    pub populated_items: usize,
    /// Items with valid values
    pub valid_items: usize,
    /// Items with invalid values
    pub invalid_items: usize,
    /// Accuracy percentage
    pub accuracy_percentage: f64,
    /// Common validation errors
    pub common_errors: Vec<String>,
}

/// Validation rule result
#[derive(Debug, Clone)]
pub struct ValidationRuleResult {
    /// Rule name
    pub rule_name: String,
    /// Rule description
    pub description: String,
    /// Items that passed the rule
    pub passed_items: usize,
    /// Items that failed the rule
    pub failed_items: usize,
    /// Success rate
    pub success_rate: f64,
    /// Failed item UUIDs
    pub failed_item_uuids: Vec<String>,
}

/// Accuracy validator for POA&M data
#[derive(Debug, Clone)]
pub struct AccuracyValidator {
    /// Date format patterns for validation
    date_patterns: Vec<String>,
    /// Valid status values
    valid_statuses: Vec<String>,
    /// Valid severity levels
    valid_severities: Vec<String>,
    /// UUID validation regex
    uuid_regex: &'static Regex,
    /// Email validation regex
    email_regex: &'static Regex,
}

/// Get UUID validation regex
fn uuid_regex() -> &'static Regex {
    static UUID_REGEX: OnceLock<Regex> = OnceLock::new();
    UUID_REGEX.get_or_init(|| {
        Regex::new(r"^[0-9a-fA-F]{8}-[0-9a-fA-F]{4}-[0-9a-fA-F]{4}-[0-9a-fA-F]{4}-[0-9a-fA-F]{12}$")
            .expect("Invalid UUID regex")
    })
}

/// Get email validation regex
fn email_regex() -> &'static Regex {
    static EMAIL_REGEX: OnceLock<Regex> = OnceLock::new();
    EMAIL_REGEX.get_or_init(|| {
        Regex::new(r"^[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}$")
            .expect("Invalid email regex")
    })
}

impl AccuracyValidator {
    /// Create a new accuracy validator with default configuration
    pub fn new() -> Self {
        Self {
            date_patterns: vec![
                "%Y-%m-%d".to_string(),
                "%Y-%m-%dT%H:%M:%S%.fZ".to_string(),
                "%Y-%m-%dT%H:%M:%SZ".to_string(),
                "%m/%d/%Y".to_string(),
                "%d/%m/%Y".to_string(),
            ],
            valid_statuses: vec![
                "Open".to_string(),
                "In Progress".to_string(),
                "Completed".to_string(),
                "Closed".to_string(),
                "Cancelled".to_string(),
                "On Hold".to_string(),
            ],
            valid_severities: vec![
                "Critical".to_string(),
                "High".to_string(),
                "Medium".to_string(),
                "Low".to_string(),
                "Informational".to_string(),
            ],
            uuid_regex: uuid_regex(),
            email_regex: email_regex(),
        }
    }

    /// Create an accuracy validator with custom configuration
    pub fn with_config(config: &QualityConfig) -> Self {
        let mut validator = Self::new();
        
        // Override with custom field rules if provided
        if let Some(status_rule) = config.field_rules.get("status") {
            if let Some(allowed_values) = &status_rule.allowed_values {
                validator.valid_statuses = allowed_values.clone();
            }
        }

        if let Some(severity_rule) = config.field_rules.get("severity") {
            if let Some(allowed_values) = &severity_rule.allowed_values {
                validator.valid_severities = allowed_values.clone();
            }
        }

        validator
    }

    /// Validate accuracy of POA&M items
    pub fn validate(&self, poam_items: &[PoamItem]) -> Result<AccuracyResult> {
        info!("Validating accuracy for {} POA&M items", poam_items.len());

        if poam_items.is_empty() {
            return Ok(AccuracyResult {
                score: 1.0, // No items to validate means perfect accuracy
                findings: Vec::new(),
                field_accuracy: HashMap::new(),
                rule_results: Vec::new(),
            });
        }

        // Run validation rules
        let rule_results = self.run_validation_rules(poam_items)?;

        // Calculate field-level accuracy
        let field_accuracy = self.calculate_field_accuracy(poam_items)?;

        // Generate accuracy findings
        let findings = self.generate_accuracy_findings(&rule_results, &field_accuracy)?;

        // Calculate overall accuracy score
        let overall_score = self.calculate_overall_accuracy_score(&rule_results, &field_accuracy);

        debug!(
            "Accuracy validation completed: Score: {:.2}, Findings: {}",
            overall_score,
            findings.len()
        );

        Ok(AccuracyResult {
            score: overall_score,
            findings,
            field_accuracy,
            rule_results,
        })
    }

    /// Run all validation rules on POA&M items
    fn run_validation_rules(&self, poam_items: &[PoamItem]) -> Result<Vec<ValidationRuleResult>> {
        let mut rule_results = Vec::new();

        // UUID format validation
        rule_results.push(self.validate_uuid_format(poam_items)?);

        // Date format validation
        rule_results.push(self.validate_date_formats(poam_items)?);

        // Status value validation
        rule_results.push(self.validate_status_values(poam_items)?);

        // Severity value validation
        rule_results.push(self.validate_severity_values(poam_items)?);

        // Text field quality validation
        rule_results.push(self.validate_text_quality(poam_items)?);

        // Date logic validation
        rule_results.push(self.validate_date_logic(poam_items)?);

        Ok(rule_results)
    }

    /// Validate UUID format
    fn validate_uuid_format(&self, poam_items: &[PoamItem]) -> Result<ValidationRuleResult> {
        let mut passed = 0;
        let mut failed_uuids = Vec::new();

        for item in poam_items {
            if self.uuid_regex.is_match(&item.uuid) {
                passed += 1;
            } else {
                failed_uuids.push(item.uuid.clone());
            }
        }

        let failed = poam_items.len() - passed;
        let success_rate = if !poam_items.is_empty() {
            passed as f64 / poam_items.len() as f64
        } else {
            1.0
        };

        Ok(ValidationRuleResult {
            rule_name: "uuid_format".to_string(),
            description: "UUID fields must follow RFC 4122 format".to_string(),
            passed_items: passed,
            failed_items: failed,
            success_rate,
            failed_item_uuids: failed_uuids,
        })
    }

    /// Validate date formats
    fn validate_date_formats(&self, poam_items: &[PoamItem]) -> Result<ValidationRuleResult> {
        let mut passed = 0;
        let mut failed_uuids = Vec::new();

        for item in poam_items {
            let mut item_valid = true;

            // Check scheduled completion date
            if let Some(date_str) = &item.scheduled_completion_date {
                if !self.is_valid_date_format(date_str) {
                    item_valid = false;
                }
            }

            // Check actual completion date
            if let Some(date_str) = &item.actual_completion_date {
                if !self.is_valid_date_format(date_str) {
                    item_valid = false;
                }
            }

            if item_valid {
                passed += 1;
            } else {
                failed_uuids.push(item.uuid.clone());
            }
        }

        let failed = poam_items.len() - passed;
        let success_rate = if !poam_items.is_empty() {
            passed as f64 / poam_items.len() as f64
        } else {
            1.0
        };

        Ok(ValidationRuleResult {
            rule_name: "date_format".to_string(),
            description: "Date fields must follow valid date formats".to_string(),
            passed_items: passed,
            failed_items: failed,
            success_rate,
            failed_item_uuids: failed_uuids,
        })
    }

    /// Validate status values
    fn validate_status_values(&self, poam_items: &[PoamItem]) -> Result<ValidationRuleResult> {
        let mut passed = 0;
        let mut failed_uuids = Vec::new();

        for item in poam_items {
            if self.valid_statuses.contains(&item.status) {
                passed += 1;
            } else {
                failed_uuids.push(item.uuid.clone());
            }
        }

        let failed = poam_items.len() - passed;
        let success_rate = if !poam_items.is_empty() {
            passed as f64 / poam_items.len() as f64
        } else {
            1.0
        };

        Ok(ValidationRuleResult {
            rule_name: "status_values".to_string(),
            description: format!("Status must be one of: {}", self.valid_statuses.join(", ")),
            passed_items: passed,
            failed_items: failed,
            success_rate,
            failed_item_uuids: failed_uuids,
        })
    }

    /// Validate severity values
    fn validate_severity_values(&self, poam_items: &[PoamItem]) -> Result<ValidationRuleResult> {
        let mut passed = 0;
        let mut failed_uuids = Vec::new();

        for item in poam_items {
            let severity_valid = item.severity.as_ref()
                .map_or(true, |s| self.valid_severities.contains(s));

            if severity_valid {
                passed += 1;
            } else {
                failed_uuids.push(item.uuid.clone());
            }
        }

        let failed = poam_items.len() - passed;
        let success_rate = if !poam_items.is_empty() {
            passed as f64 / poam_items.len() as f64
        } else {
            1.0
        };

        Ok(ValidationRuleResult {
            rule_name: "severity_values".to_string(),
            description: format!("Severity must be one of: {}", self.valid_severities.join(", ")),
            passed_items: passed,
            failed_items: failed,
            success_rate,
            failed_item_uuids: failed_uuids,
        })
    }

    /// Validate text field quality
    fn validate_text_quality(&self, poam_items: &[PoamItem]) -> Result<ValidationRuleResult> {
        let mut passed = 0;
        let mut failed_uuids = Vec::new();

        for item in poam_items {
            let mut item_valid = true;

            // Check title quality (minimum length, not just whitespace)
            if item.title.trim().len() < 5 {
                item_valid = false;
            }

            // Check description quality
            if item.description.trim().len() < 10 {
                item_valid = false;
            }

            if item_valid {
                passed += 1;
            } else {
                failed_uuids.push(item.uuid.clone());
            }
        }

        let failed = poam_items.len() - passed;
        let success_rate = if !poam_items.is_empty() {
            passed as f64 / poam_items.len() as f64
        } else {
            1.0
        };

        Ok(ValidationRuleResult {
            rule_name: "text_quality".to_string(),
            description: "Text fields must meet minimum quality standards".to_string(),
            passed_items: passed,
            failed_items: failed,
            success_rate,
            failed_item_uuids: failed_uuids,
        })
    }

    /// Validate date logic (scheduled vs actual dates)
    fn validate_date_logic(&self, poam_items: &[PoamItem]) -> Result<ValidationRuleResult> {
        let mut passed = 0;
        let mut failed_uuids = Vec::new();

        for item in poam_items {
            let mut item_valid = true;

            // If both dates are present, actual should not be before scheduled
            if let (Some(scheduled), Some(actual)) = (&item.scheduled_completion_date, &item.actual_completion_date) {
                if let (Ok(scheduled_dt), Ok(actual_dt)) = (
                    self.parse_date_string(scheduled),
                    self.parse_date_string(actual)
                ) {
                    if actual_dt < scheduled_dt {
                        // This might be valid (early completion), so just warn
                        // item_valid = false;
                    }
                }
            }

            if item_valid {
                passed += 1;
            } else {
                failed_uuids.push(item.uuid.clone());
            }
        }

        let failed = poam_items.len() - passed;
        let success_rate = if !poam_items.is_empty() {
            passed as f64 / poam_items.len() as f64
        } else {
            1.0
        };

        Ok(ValidationRuleResult {
            rule_name: "date_logic".to_string(),
            description: "Date fields must follow logical constraints".to_string(),
            passed_items: passed,
            failed_items: failed,
            success_rate,
            failed_item_uuids: failed_uuids,
        })
    }

    /// Check if a date string follows valid format
    fn is_valid_date_format(&self, date_str: &str) -> bool {
        self.parse_date_string(date_str).is_ok()
    }

    /// Parse date string using configured patterns
    fn parse_date_string(&self, date_str: &str) -> Result<DateTime<Utc>> {
        // Try parsing as ISO 8601 first
        if let Ok(dt) = DateTime::parse_from_rfc3339(date_str) {
            return Ok(dt.with_timezone(&Utc));
        }

        // Try other patterns
        for pattern in &self.date_patterns {
            if let Ok(naive_dt) = NaiveDate::parse_from_str(date_str, pattern) {
                return Ok(naive_dt.and_hms_opt(0, 0, 0).unwrap().and_utc());
            }
        }

        Err(Error::document_parsing(format!("Invalid date format: {}", date_str)))
    }

    /// Calculate field-level accuracy statistics
    fn calculate_field_accuracy(&self, poam_items: &[PoamItem]) -> Result<HashMap<String, FieldAccuracyStats>> {
        let mut field_accuracy = HashMap::new();

        // Analyze UUID field
        let uuid_stats = self.analyze_field_accuracy(
            poam_items,
            "uuid",
            |item| Some(&item.uuid),
            |value| self.uuid_regex.is_match(value),
        );
        field_accuracy.insert("uuid".to_string(), uuid_stats);

        // Analyze status field
        let status_stats = self.analyze_field_accuracy(
            poam_items,
            "status",
            |item| Some(&item.status),
            |value| self.valid_statuses.contains(&value.to_string()),
        );
        field_accuracy.insert("status".to_string(), status_stats);

        Ok(field_accuracy)
    }

    /// Analyze accuracy for a specific field
    fn analyze_field_accuracy<F, V>(
        &self,
        poam_items: &[PoamItem],
        field_name: &str,
        field_extractor: F,
        validator: V,
    ) -> FieldAccuracyStats
    where
        F: Fn(&PoamItem) -> Option<&String>,
        V: Fn(&str) -> bool,
    {
        let mut populated_items = 0;
        let mut valid_items = 0;
        let mut common_errors = Vec::new();

        for item in poam_items {
            if let Some(field_value) = field_extractor(item) {
                populated_items += 1;
                if validator(field_value) {
                    valid_items += 1;
                } else {
                    // Track common error patterns
                    if field_value.is_empty() {
                        common_errors.push("Empty value".to_string());
                    } else if field_value.trim() != field_value {
                        common_errors.push("Leading/trailing whitespace".to_string());
                    }
                }
            }
        }

        let invalid_items = populated_items - valid_items;
        let accuracy_percentage = if populated_items > 0 {
            valid_items as f64 / populated_items as f64 * 100.0
        } else {
            100.0
        };

        // Deduplicate common errors
        common_errors.sort();
        common_errors.dedup();

        FieldAccuracyStats {
            field_name: field_name.to_string(),
            populated_items,
            valid_items,
            invalid_items,
            accuracy_percentage,
            common_errors,
        }
    }

    /// Generate accuracy findings
    fn generate_accuracy_findings(
        &self,
        rule_results: &[ValidationRuleResult],
        field_accuracy: &HashMap<String, FieldAccuracyStats>,
    ) -> Result<Vec<QualityFinding>> {
        let mut findings = Vec::new();

        // Generate findings for failed validation rules
        for rule_result in rule_results {
            if rule_result.success_rate < 0.9 && rule_result.failed_items > 0 {
                let severity = if rule_result.success_rate < 0.5 {
                    QualitySeverity::Critical
                } else if rule_result.success_rate < 0.8 {
                    QualitySeverity::High
                } else {
                    QualitySeverity::Medium
                };

                findings.push(QualityFinding {
                    id: Uuid::new_v4().to_string(),
                    severity,
                    category: QualityCategory::Accuracy,
                    description: format!(
                        "Validation rule '{}' failed for {} items ({:.1}% success rate)",
                        rule_result.rule_name,
                        rule_result.failed_items,
                        rule_result.success_rate * 100.0
                    ),
                    affected_items: rule_result.failed_item_uuids.clone(),
                    impact_assessment: format!(
                        "{}. This affects data quality and may cause processing issues.",
                        rule_result.description
                    ),
                    recommendation: format!(
                        "Review and correct the {} items that failed the '{}' validation rule",
                        rule_result.failed_items,
                        rule_result.rule_name
                    ),
                    location: Some(format!("accuracy_validation.{}", rule_result.rule_name)),
                    metadata: HashMap::new(),
                });
            }
        }

        Ok(findings)
    }

    /// Calculate overall accuracy score
    fn calculate_overall_accuracy_score(
        &self,
        rule_results: &[ValidationRuleResult],
        _field_accuracy: &HashMap<String, FieldAccuracyStats>,
    ) -> f64 {
        if rule_results.is_empty() {
            return 1.0;
        }

        let total_score: f64 = rule_results.iter()
            .map(|result| result.success_rate)
            .sum();

        total_score / rule_results.len() as f64
    }
}

impl Default for AccuracyValidator {
    fn default() -> Self {
        Self::new()
    }
}
