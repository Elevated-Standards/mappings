//! Modified: 2025-01-22
//! 
//! Type definitions for accuracy validation functionality
//! 
//! This module contains all the data structures and types used for POA&M data accuracy validation,
//! including result types, statistics, and validation rule results.

use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::quality::{QualityFinding, QualitySeverity, QualityCategory};

/// Result of accuracy validation
/// 
/// Contains comprehensive accuracy assessment results including overall score,
/// detailed findings, field-level statistics, and validation rule results.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccuracyResult {
    /// Overall accuracy score (0.0 to 1.0)
    /// 
    /// Calculated as the weighted average of all validation rule success rates.
    /// A score of 1.0 indicates perfect accuracy, while 0.0 indicates complete failure.
    pub score: f64,
    
    /// Accuracy findings
    /// 
    /// List of quality findings that identify specific accuracy issues found during validation.
    /// Each finding includes severity, description, affected items, and recommendations.
    pub findings: Vec<QualityFinding>,
    
    /// Field-level accuracy statistics
    /// 
    /// Detailed statistics for each field analyzed, including population rates,
    /// validation success rates, and common error patterns.
    pub field_accuracy: HashMap<String, FieldAccuracyStats>,
    
    /// Validation rule results
    /// 
    /// Results from each individual validation rule execution, including
    /// pass/fail counts and detailed failure information.
    pub rule_results: Vec<ValidationRuleResult>,
}

/// Field-level accuracy statistics
/// 
/// Provides detailed analysis of accuracy for individual fields within POA&M items,
/// including population statistics, validation results, and error patterns.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FieldAccuracyStats {
    /// Field name
    /// 
    /// The name of the field being analyzed (e.g., "uuid", "status", "severity").
    pub field_name: String,
    
    /// Total items with this field populated
    /// 
    /// Count of POA&M items that have a non-null value for this field.
    pub populated_items: usize,
    
    /// Items with valid values
    /// 
    /// Count of populated items that pass validation rules for this field.
    pub valid_items: usize,
    
    /// Items with invalid values
    /// 
    /// Count of populated items that fail validation rules for this field.
    pub invalid_items: usize,
    
    /// Accuracy percentage
    /// 
    /// Percentage of populated items that have valid values (valid_items / populated_items * 100).
    pub accuracy_percentage: f64,
    
    /// Common validation errors
    /// 
    /// List of frequently occurring error patterns for this field,
    /// such as "Empty value" or "Leading/trailing whitespace".
    pub common_errors: Vec<String>,
}

/// Validation rule result
/// 
/// Contains the results of executing a specific validation rule against a set of POA&M items,
/// including success metrics and detailed failure information.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationRuleResult {
    /// Rule name
    /// 
    /// Unique identifier for the validation rule (e.g., "uuid_format", "date_format").
    pub rule_name: String,
    
    /// Rule description
    /// 
    /// Human-readable description of what the validation rule checks.
    pub description: String,
    
    /// Items that passed the rule
    /// 
    /// Count of POA&M items that successfully passed this validation rule.
    pub passed_items: usize,
    
    /// Items that failed the rule
    /// 
    /// Count of POA&M items that failed this validation rule.
    pub failed_items: usize,
    
    /// Success rate
    /// 
    /// Ratio of passed items to total items (passed_items / total_items).
    /// Range: 0.0 (all failed) to 1.0 (all passed).
    pub success_rate: f64,
    
    /// Failed item UUIDs
    /// 
    /// List of UUIDs for POA&M items that failed this validation rule.
    /// Used for detailed reporting and remediation tracking.
    pub failed_item_uuids: Vec<String>,
}

/// Accuracy validation configuration
/// 
/// Configuration parameters for customizing accuracy validation behavior,
/// including validation patterns, thresholds, and rule-specific settings.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccuracyConfig {
    /// Date format patterns for validation
    /// 
    /// List of date format patterns to accept during date validation.
    /// Uses chrono format strings (e.g., "%Y-%m-%d", "%m/%d/%Y").
    pub date_patterns: Vec<String>,
    
    /// Valid status values
    /// 
    /// List of acceptable status values for POA&M items.
    pub valid_statuses: Vec<String>,
    
    /// Valid severity levels
    /// 
    /// List of acceptable severity levels for POA&M items.
    pub valid_severities: Vec<String>,
    
    /// Minimum text field length thresholds
    /// 
    /// Minimum character counts for text fields to be considered valid.
    pub text_length_thresholds: HashMap<String, usize>,
    
    /// Enable strict UUID validation
    /// 
    /// Whether to enforce RFC 4122 UUID format validation.
    pub strict_uuid_validation: bool,
    
    /// Enable date logic validation
    /// 
    /// Whether to validate logical relationships between dates
    /// (e.g., actual completion should not be before scheduled completion).
    pub enable_date_logic_validation: bool,
    
    /// Accuracy score thresholds for findings generation
    /// 
    /// Thresholds for generating quality findings based on accuracy scores.
    pub accuracy_thresholds: AccuracyThresholds,
}

/// Accuracy score thresholds for quality assessment
/// 
/// Defines thresholds for categorizing accuracy issues by severity level.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccuracyThresholds {
    /// Critical threshold (below this is critical)
    /// 
    /// Accuracy scores below this threshold generate critical severity findings.
    pub critical_threshold: f64,
    
    /// High threshold (below this is high severity)
    /// 
    /// Accuracy scores below this threshold generate high severity findings.
    pub high_threshold: f64,
    
    /// Medium threshold (below this is medium severity)
    /// 
    /// Accuracy scores below this threshold generate medium severity findings.
    pub medium_threshold: f64,
    
    /// Minimum items threshold for rule evaluation
    /// 
    /// Minimum number of failed items required to generate a finding.
    pub min_failed_items: usize,
}

impl Default for AccuracyConfig {
    fn default() -> Self {
        let mut text_length_thresholds = HashMap::new();
        text_length_thresholds.insert("title".to_string(), 5);
        text_length_thresholds.insert("description".to_string(), 10);
        
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
            text_length_thresholds,
            strict_uuid_validation: true,
            enable_date_logic_validation: true,
            accuracy_thresholds: AccuracyThresholds::default(),
        }
    }
}

impl Default for AccuracyThresholds {
    fn default() -> Self {
        Self {
            critical_threshold: 0.5,
            high_threshold: 0.8,
            medium_threshold: 0.9,
            min_failed_items: 1,
        }
    }
}

impl AccuracyResult {
    /// Create a new empty accuracy result
    pub fn new() -> Self {
        Self {
            score: 1.0,
            findings: Vec::new(),
            field_accuracy: HashMap::new(),
            rule_results: Vec::new(),
        }
    }
    
    /// Check if the accuracy result indicates high quality data
    pub fn is_high_quality(&self) -> bool {
        self.score >= 0.9 && self.findings.iter().all(|f| f.severity != QualitySeverity::Critical)
    }
    
    /// Get the number of critical accuracy issues
    pub fn critical_issues_count(&self) -> usize {
        self.findings.iter()
            .filter(|f| f.severity == QualitySeverity::Critical)
            .count()
    }
    
    /// Get the total number of failed items across all rules
    pub fn total_failed_items(&self) -> usize {
        self.rule_results.iter()
            .map(|r| r.failed_items)
            .sum()
    }
}

impl FieldAccuracyStats {
    /// Create new field accuracy statistics
    pub fn new(field_name: String) -> Self {
        Self {
            field_name,
            populated_items: 0,
            valid_items: 0,
            invalid_items: 0,
            accuracy_percentage: 100.0,
            common_errors: Vec::new(),
        }
    }
    
    /// Check if the field has good accuracy (>= 90%)
    pub fn has_good_accuracy(&self) -> bool {
        self.accuracy_percentage >= 90.0
    }
    
    /// Get the population rate (populated_items / total_items)
    pub fn population_rate(&self, total_items: usize) -> f64 {
        if total_items == 0 {
            0.0
        } else {
            self.populated_items as f64 / total_items as f64
        }
    }
}

impl ValidationRuleResult {
    /// Create a new validation rule result
    pub fn new(rule_name: String, description: String) -> Self {
        Self {
            rule_name,
            description,
            passed_items: 0,
            failed_items: 0,
            success_rate: 1.0,
            failed_item_uuids: Vec::new(),
        }
    }
    
    /// Check if the rule has acceptable success rate (>= 90%)
    pub fn has_acceptable_success_rate(&self) -> bool {
        self.success_rate >= 0.9
    }
    
    /// Get the total number of items evaluated by this rule
    pub fn total_items(&self) -> usize {
        self.passed_items + self.failed_items
    }
}
