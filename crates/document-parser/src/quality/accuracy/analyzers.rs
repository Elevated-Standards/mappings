//! Modified: 2025-01-22
//! 
//! Field analysis and statistics calculation for accuracy validation
//! 
//! This module provides functionality for analyzing individual fields within POA&M items,
//! calculating accuracy statistics, and generating detailed field-level reports.

use std::collections::HashMap;
use fedramp_core::Result;
use serde_json::Value;

use crate::poam::PoamItem;
use crate::quality::{QualityFinding, QualitySeverity, QualityCategory};
use super::types::{FieldAccuracyStats, ValidationRuleResult, AccuracyConfig};
use super::rules::{uuid_regex, email_regex};

/// Field accuracy analyzer for POA&M data
///
/// Provides comprehensive analysis of field-level accuracy including population rates,
/// validation success rates, error pattern detection, and statistical reporting.
#[derive(Debug, Clone)]
pub struct FieldAccuracyAnalyzer {
    /// Configuration for accuracy analysis
    config: AccuracyConfig,
}

impl FieldAccuracyAnalyzer {
    /// Create a new field accuracy analyzer with configuration
    pub fn new(config: AccuracyConfig) -> Self {
        Self { config }
    }
    
    /// Create a new field accuracy analyzer with default configuration
    pub fn with_defaults() -> Self {
        Self::new(AccuracyConfig::default())
    }
    
    /// Calculate field-level accuracy statistics for all relevant fields
    /// 
    /// Analyzes multiple fields within POA&M items and returns comprehensive
    /// accuracy statistics for each field, including population rates and validation results.
    pub fn calculate_field_accuracy(&self, poam_items: &[PoamItem]) -> Result<HashMap<String, FieldAccuracyStats>> {
        let mut field_accuracy = HashMap::new();
        
        // Analyze UUID field
        let uuid_stats = self.analyze_field_accuracy(
            poam_items,
            "uuid",
            |item| Some(&item.uuid),
            |value| uuid_regex().is_match(value),
        );
        field_accuracy.insert("uuid".to_string(), uuid_stats);
        
        // Analyze status field
        let status_stats = self.analyze_field_accuracy(
            poam_items,
            "status",
            |item| Some(&item.status),
            |value| self.config.valid_statuses.contains(&value.to_string()),
        );
        field_accuracy.insert("status".to_string(), status_stats);
        
        // Analyze severity field (optional)
        let severity_stats = self.analyze_optional_field_accuracy(
            poam_items,
            "severity",
            |item| item.severity.as_ref(),
            |value| self.config.valid_severities.contains(&value.to_string()),
        );
        field_accuracy.insert("severity".to_string(), severity_stats);
        
        // Analyze title field
        let title_min_length = self.config.text_length_thresholds.get("title").copied().unwrap_or(5);
        let title_stats = self.analyze_field_accuracy(
            poam_items,
            "title",
            |item| Some(&item.title),
            |value| value.trim().len() >= title_min_length,
        );
        field_accuracy.insert("title".to_string(), title_stats);
        
        // Analyze description field
        let description_min_length = self.config.text_length_thresholds.get("description").copied().unwrap_or(10);
        let description_stats = self.analyze_field_accuracy(
            poam_items,
            "description",
            |item| Some(&item.description),
            |value| value.trim().len() >= description_min_length,
        );
        field_accuracy.insert("description".to_string(), description_stats);
        
        // Analyze scheduled completion date field (optional)
        let scheduled_date_stats = self.analyze_optional_field_accuracy(
            poam_items,
            "scheduled_completion_date",
            |item| item.scheduled_completion_date.as_ref(),
            |value| self.is_valid_date_format(value),
        );
        field_accuracy.insert("scheduled_completion_date".to_string(), scheduled_date_stats);
        
        // Analyze actual completion date field (optional)
        let actual_date_stats = self.analyze_optional_field_accuracy(
            poam_items,
            "actual_completion_date",
            |item| item.actual_completion_date.as_ref(),
            |value| self.is_valid_date_format(value),
        );
        field_accuracy.insert("actual_completion_date".to_string(), actual_date_stats);
        
        Ok(field_accuracy)
    }
    
    /// Analyze accuracy for a specific required field
    /// 
    /// Performs detailed analysis of a field that should be present in all POA&M items,
    /// calculating validation success rates and identifying common error patterns.
    pub fn analyze_field_accuracy<F, V>(
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
                    self.identify_error_patterns(field_value, &mut common_errors);
                }
            } else {
                // Field is missing
                common_errors.push("Missing value".to_string());
            }
        }
        
        let invalid_items = populated_items - valid_items;
        let accuracy_percentage = if populated_items > 0 {
            valid_items as f64 / populated_items as f64 * 100.0
        } else {
            0.0  // No populated items means 0% accuracy for required fields
        };
        
        // Deduplicate and sort common errors
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
    
    /// Analyze accuracy for a specific optional field
    /// 
    /// Performs detailed analysis of a field that may be absent in POA&M items,
    /// treating missing values as acceptable rather than errors.
    pub fn analyze_optional_field_accuracy<F, V>(
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
                    self.identify_error_patterns(field_value, &mut common_errors);
                }
            }
            // Missing values are acceptable for optional fields
        }
        
        let invalid_items = populated_items - valid_items;
        let accuracy_percentage = if populated_items > 0 {
            valid_items as f64 / populated_items as f64 * 100.0
        } else {
            100.0  // No populated items means 100% accuracy for optional fields
        };
        
        // Deduplicate and sort common errors
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
    
    /// Generate accuracy findings based on field statistics and rule results
    /// 
    /// Creates quality findings for accuracy issues, categorizing them by severity
    /// based on the impact and frequency of the problems identified.
    pub fn generate_accuracy_findings(
        &self,
        rule_results: &[ValidationRuleResult],
        field_accuracy: &HashMap<String, FieldAccuracyStats>,
    ) -> Result<Vec<QualityFinding>> {
        let mut findings = Vec::new();
        
        // Generate findings for failed validation rules
        for rule_result in rule_results {
            if rule_result.success_rate < self.config.accuracy_thresholds.medium_threshold 
                && rule_result.failed_items >= self.config.accuracy_thresholds.min_failed_items {
                
                let severity = self.determine_finding_severity(rule_result.success_rate);
                
                findings.push(QualityFinding {
                    id: uuid::Uuid::new_v4().to_string(),
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
        
        // Generate findings for poor field accuracy
        for (field_name, stats) in field_accuracy {
            if stats.accuracy_percentage < self.config.accuracy_thresholds.medium_threshold * 100.0
                && stats.invalid_items >= self.config.accuracy_thresholds.min_failed_items {
                
                let severity = self.determine_finding_severity(stats.accuracy_percentage / 100.0);
                
                findings.push(QualityFinding {
                    id: uuid::Uuid::new_v4().to_string(),
                    severity,
                    category: QualityCategory::Accuracy,
                    description: format!(
                        "Field '{}' has poor accuracy: {:.1}% ({} invalid out of {} populated)",
                        field_name,
                        stats.accuracy_percentage,
                        stats.invalid_items,
                        stats.populated_items
                    ),
                    affected_items: Vec::new(), // Field-level finding doesn't have specific item UUIDs
                    impact_assessment: format!(
                        "Poor data quality in '{}' field affects data reliability and processing accuracy. Common errors: {}",
                        field_name,
                        stats.common_errors.join(", ")
                    ),
                    recommendation: format!(
                        "Review and improve data entry processes for '{}' field. Address common errors: {}",
                        field_name,
                        stats.common_errors.join(", ")
                    ),
                    location: Some(format!("field_accuracy.{}", field_name)),
                    metadata: {
                        let mut metadata = HashMap::new();
                        metadata.insert("field_name".to_string(), Value::String(field_name.clone()));
                        metadata.insert("accuracy_percentage".to_string(), Value::Number(serde_json::Number::from_f64(stats.accuracy_percentage).unwrap_or_else(|| serde_json::Number::from(0))));
                        metadata.insert("invalid_items".to_string(), Value::Number(serde_json::Number::from(stats.invalid_items)));
                        metadata.insert("populated_items".to_string(), Value::Number(serde_json::Number::from(stats.populated_items)));
                        metadata
                    },
                });
            }
        }
        
        Ok(findings)
    }
    
    /// Calculate overall accuracy score from rule results and field statistics
    /// 
    /// Computes a weighted accuracy score considering both validation rule success rates
    /// and field-level accuracy statistics.
    pub fn calculate_overall_accuracy_score(
        &self,
        rule_results: &[ValidationRuleResult],
        field_accuracy: &HashMap<String, FieldAccuracyStats>,
    ) -> f64 {
        if rule_results.is_empty() && field_accuracy.is_empty() {
            return 1.0;
        }
        
        let mut total_score = 0.0;
        let mut weight_sum = 0.0;
        
        // Weight validation rule results (70% of total score)
        if !rule_results.is_empty() {
            let rule_score: f64 = rule_results.iter()
                .map(|result| result.success_rate)
                .sum::<f64>() / rule_results.len() as f64;
            total_score += rule_score * 0.7;
            weight_sum += 0.7;
        }
        
        // Weight field accuracy results (30% of total score)
        if !field_accuracy.is_empty() {
            let field_score: f64 = field_accuracy.values()
                .map(|stats| stats.accuracy_percentage / 100.0)
                .sum::<f64>() / field_accuracy.len() as f64;
            total_score += field_score * 0.3;
            weight_sum += 0.3;
        }
        
        if weight_sum > 0.0 {
            total_score / weight_sum
        } else {
            1.0
        }
    }
    
    /// Identify common error patterns in field values
    fn identify_error_patterns(&self, field_value: &str, common_errors: &mut Vec<String>) {
        if field_value.is_empty() {
            common_errors.push("Empty value".to_string());
        } else if field_value.trim() != field_value {
            common_errors.push("Leading/trailing whitespace".to_string());
        } else if field_value.chars().all(|c| c.is_whitespace()) {
            common_errors.push("Only whitespace".to_string());
        } else if field_value.len() < 3 {
            common_errors.push("Too short".to_string());
        }
    }
    
    /// Determine finding severity based on accuracy score
    fn determine_finding_severity(&self, accuracy_score: f64) -> QualitySeverity {
        if accuracy_score < self.config.accuracy_thresholds.critical_threshold {
            QualitySeverity::Critical
        } else if accuracy_score < self.config.accuracy_thresholds.high_threshold {
            QualitySeverity::High
        } else {
            QualitySeverity::Medium
        }
    }
    
    /// Check if a date string follows valid format using configured patterns
    fn is_valid_date_format(&self, date_str: &str) -> bool {
        // Try parsing as ISO 8601 first
        if chrono::DateTime::parse_from_rfc3339(date_str).is_ok() {
            return true;
        }
        
        // Try other patterns
        for pattern in &self.config.date_patterns {
            if chrono::NaiveDate::parse_from_str(date_str, pattern).is_ok() {
                return true;
            }
        }
        
        false
    }
}

impl Default for FieldAccuracyAnalyzer {
    fn default() -> Self {
        Self::with_defaults()
    }
}
