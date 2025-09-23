//! Modified: 2025-01-22
//! 
//! Main accuracy validator implementation
//! 
//! This module provides the primary AccuracyValidator implementation that coordinates
//! validation rules and field analysis to produce comprehensive accuracy assessments
//! for POA&M data.

use std::collections::HashMap;
use fedramp_core::Result;
use tracing::{debug, info};

use crate::poam::PoamItem;
use crate::quality::QualityConfig;
use super::types::{AccuracyResult, AccuracyConfig};
use super::rules::ValidationRuleExecutor;
use super::analyzers::FieldAccuracyAnalyzer;

/// Accuracy validator for POA&M data
/// 
/// Provides comprehensive accuracy validation by coordinating validation rules
/// and field analysis to assess data quality and generate detailed reports.
#[derive(Debug, Clone)]
pub struct AccuracyValidator {
    /// Validation rule executor
    rule_executor: ValidationRuleExecutor,
    
    /// Field accuracy analyzer
    field_analyzer: FieldAccuracyAnalyzer,
    
    /// Accuracy configuration
    config: AccuracyConfig,
}

impl AccuracyValidator {
    /// Create a new accuracy validator with default configuration
    pub fn new() -> Self {
        let config = AccuracyConfig::default();
        Self {
            rule_executor: ValidationRuleExecutor::new(config.clone()),
            field_analyzer: FieldAccuracyAnalyzer::new(config.clone()),
            config,
        }
    }
    
    /// Create an accuracy validator with custom configuration
    /// 
    /// Converts QualityConfig to AccuracyConfig and initializes all components
    /// with the provided configuration parameters.
    pub fn with_config(quality_config: &QualityConfig) -> Self {
        let config = Self::convert_quality_config(quality_config);
        Self {
            rule_executor: ValidationRuleExecutor::new(config.clone()),
            field_analyzer: FieldAccuracyAnalyzer::new(config.clone()),
            config,
        }
    }
    
    /// Create an accuracy validator with accuracy-specific configuration
    pub fn with_accuracy_config(config: AccuracyConfig) -> Self {
        Self {
            rule_executor: ValidationRuleExecutor::new(config.clone()),
            field_analyzer: FieldAccuracyAnalyzer::new(config.clone()),
            config,
        }
    }
    
    /// Validate accuracy of POA&M items
    /// 
    /// Performs comprehensive accuracy validation including format validation,
    /// value constraints, field analysis, and quality assessment.
    pub fn validate(&self, poam_items: &[PoamItem]) -> Result<AccuracyResult> {
        info!("Validating accuracy for {} POA&M items", poam_items.len());
        
        if poam_items.is_empty() {
            debug!("No POA&M items to validate, returning perfect accuracy score");
            return Ok(AccuracyResult {
                score: 1.0, // No items to validate means perfect accuracy
                findings: Vec::new(),
                field_accuracy: HashMap::new(),
                rule_results: Vec::new(),
            });
        }
        
        // Run validation rules
        let rule_results = self.run_validation_rules(poam_items)?;
        debug!("Completed {} validation rules", rule_results.len());
        
        // Calculate field-level accuracy
        let field_accuracy = self.field_analyzer.calculate_field_accuracy(poam_items)?;
        debug!("Analyzed accuracy for {} fields", field_accuracy.len());
        
        // Generate accuracy findings
        let findings = self.field_analyzer.generate_accuracy_findings(&rule_results, &field_accuracy)?;
        debug!("Generated {} accuracy findings", findings.len());
        
        // Calculate overall accuracy score
        let overall_score = self.field_analyzer.calculate_overall_accuracy_score(&rule_results, &field_accuracy);
        
        debug!(
            "Accuracy validation completed: Score: {:.2}, Findings: {}, Rules: {}, Fields: {}",
            overall_score,
            findings.len(),
            rule_results.len(),
            field_accuracy.len()
        );
        
        Ok(AccuracyResult {
            score: overall_score,
            findings,
            field_accuracy,
            rule_results,
        })
    }
    
    /// Run all validation rules on POA&M items
    /// 
    /// Executes all configured validation rules and returns detailed results
    /// for each rule including pass/fail counts and affected items.
    fn run_validation_rules(&self, poam_items: &[PoamItem]) -> Result<Vec<super::types::ValidationRuleResult>> {
        let mut rule_results = Vec::new();
        
        // UUID format validation
        if self.config.strict_uuid_validation {
            rule_results.push(self.rule_executor.validate_uuid_format(poam_items)?);
        }
        
        // Date format validation
        rule_results.push(self.rule_executor.validate_date_formats(poam_items)?);
        
        // Status value validation
        rule_results.push(self.rule_executor.validate_status_values(poam_items)?);
        
        // Severity value validation
        rule_results.push(self.rule_executor.validate_severity_values(poam_items)?);
        
        // Text field quality validation
        rule_results.push(self.rule_executor.validate_text_quality(poam_items)?);
        
        // Date logic validation
        if self.config.enable_date_logic_validation {
            rule_results.push(self.rule_executor.validate_date_logic(poam_items)?);
        }
        
        Ok(rule_results)
    }
    
    /// Convert QualityConfig to AccuracyConfig
    /// 
    /// Maps quality configuration parameters to accuracy-specific configuration,
    /// providing sensible defaults for accuracy-specific settings.
    fn convert_quality_config(quality_config: &QualityConfig) -> AccuracyConfig {
        let mut text_length_thresholds = HashMap::new();
        
        // Extract text length requirements from field rules
        for (field_name, field_rule) in &quality_config.field_rules {
            if let Some(min_length) = field_rule.min_length {
                text_length_thresholds.insert(field_name.clone(), min_length);
            }
        }
        
        // Set defaults for common fields if not specified
        text_length_thresholds.entry("title".to_string()).or_insert(5);
        text_length_thresholds.entry("description".to_string()).or_insert(10);
        
        // Extract valid values from field rules
        let valid_statuses = quality_config.field_rules.get("status")
            .and_then(|rule| rule.allowed_values.clone())
            .unwrap_or_else(|| vec![
                "Open".to_string(),
                "In Progress".to_string(),
                "Completed".to_string(),
                "Closed".to_string(),
                "Cancelled".to_string(),
                "On Hold".to_string(),
            ]);
        
        let valid_severities = quality_config.field_rules.get("severity")
            .and_then(|rule| rule.allowed_values.clone())
            .unwrap_or_else(|| vec![
                "Critical".to_string(),
                "High".to_string(),
                "Medium".to_string(),
                "Low".to_string(),
                "Informational".to_string(),
            ]);
        
        AccuracyConfig {
            date_patterns: vec![
                "%Y-%m-%d".to_string(),
                "%Y-%m-%dT%H:%M:%S%.fZ".to_string(),
                "%Y-%m-%dT%H:%M:%SZ".to_string(),
                "%m/%d/%Y".to_string(),
                "%d/%m/%Y".to_string(),
            ],
            valid_statuses,
            valid_severities,
            text_length_thresholds,
            strict_uuid_validation: true,
            enable_date_logic_validation: true,
            accuracy_thresholds: super::types::AccuracyThresholds {
                critical_threshold: quality_config.min_compliance_score * 0.5,
                high_threshold: quality_config.min_compliance_score * 0.8,
                medium_threshold: quality_config.min_compliance_score,
                min_failed_items: 1,
            },
        }
    }
    
    /// Get the current accuracy configuration
    pub fn get_config(&self) -> &AccuracyConfig {
        &self.config
    }
    
    /// Update the accuracy configuration
    /// 
    /// Updates the configuration and reinitializes all components with the new settings.
    pub fn update_config(&mut self, config: AccuracyConfig) {
        self.config = config.clone();
        self.rule_executor = ValidationRuleExecutor::new(config.clone());
        self.field_analyzer = FieldAccuracyAnalyzer::new(config);
    }
    
    /// Validate a single POA&M item for accuracy
    /// 
    /// Convenience method for validating individual items, useful for
    /// real-time validation during data entry or processing.
    pub fn validate_single_item(&self, poam_item: &PoamItem) -> Result<AccuracyResult> {
        self.validate(&[poam_item.clone()])
    }
    
    /// Get accuracy statistics summary
    /// 
    /// Returns a summary of accuracy validation capabilities and current configuration.
    pub fn get_accuracy_summary(&self) -> AccuracySummary {
        AccuracySummary {
            total_validation_rules: self.count_enabled_rules(),
            analyzed_fields: self.get_analyzed_fields(),
            strict_uuid_validation: self.config.strict_uuid_validation,
            date_logic_validation: self.config.enable_date_logic_validation,
            supported_date_patterns: self.config.date_patterns.len(),
            valid_status_count: self.config.valid_statuses.len(),
            valid_severity_count: self.config.valid_severities.len(),
        }
    }
    
    /// Count the number of enabled validation rules
    fn count_enabled_rules(&self) -> usize {
        let mut count = 4; // Always enabled: date_format, status_values, severity_values, text_quality
        
        if self.config.strict_uuid_validation {
            count += 1;
        }
        
        if self.config.enable_date_logic_validation {
            count += 1;
        }
        
        count
    }
    
    /// Get the list of fields that are analyzed for accuracy
    fn get_analyzed_fields(&self) -> Vec<String> {
        vec![
            "uuid".to_string(),
            "status".to_string(),
            "severity".to_string(),
            "title".to_string(),
            "description".to_string(),
            "scheduled_completion_date".to_string(),
            "actual_completion_date".to_string(),
        ]
    }
}

/// Summary of accuracy validation capabilities
#[derive(Debug, Clone)]
pub struct AccuracySummary {
    /// Total number of validation rules
    pub total_validation_rules: usize,
    
    /// List of fields analyzed for accuracy
    pub analyzed_fields: Vec<String>,
    
    /// Whether strict UUID validation is enabled
    pub strict_uuid_validation: bool,
    
    /// Whether date logic validation is enabled
    pub date_logic_validation: bool,
    
    /// Number of supported date patterns
    pub supported_date_patterns: usize,
    
    /// Number of valid status values
    pub valid_status_count: usize,
    
    /// Number of valid severity values
    pub valid_severity_count: usize,
}

impl Default for AccuracyValidator {
    fn default() -> Self {
        Self::new()
    }
}
