//! Modified: 2025-01-22
//! 
//! Validation rules for accuracy checking
//! 
//! This module contains individual validation rule implementations for checking
//! various aspects of POA&M data accuracy, including format validation, value constraints,
//! and logical consistency checks.

use std::sync::OnceLock;
use regex::Regex;
use chrono::{DateTime, Utc, NaiveDate};
use fedramp_core::{Result, Error};

use crate::poam::PoamItem;
use super::types::{ValidationRuleResult, AccuracyConfig};

/// Get UUID validation regex (RFC 4122 format)
pub fn uuid_regex() -> &'static Regex {
    static UUID_REGEX: OnceLock<Regex> = OnceLock::new();
    UUID_REGEX.get_or_init(|| {
        Regex::new(r"^[0-9a-fA-F]{8}-[0-9a-fA-F]{4}-[0-9a-fA-F]{4}-[0-9a-fA-F]{4}-[0-9a-fA-F]{12}$")
            .expect("Invalid UUID regex")
    })
}

/// Get email validation regex
pub fn email_regex() -> &'static Regex {
    static EMAIL_REGEX: OnceLock<Regex> = OnceLock::new();
    EMAIL_REGEX.get_or_init(|| {
        Regex::new(r"^[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}$")
            .expect("Invalid email regex")
    })
}

/// Validation rule executor for POA&M accuracy checks
///
/// Provides methods for executing individual validation rules against POA&M items,
/// with configurable validation parameters and detailed result reporting.
#[derive(Debug, Clone)]
pub struct ValidationRuleExecutor {
    /// Configuration for validation rules
    config: AccuracyConfig,
}

impl ValidationRuleExecutor {
    /// Create a new validation rule executor with configuration
    pub fn new(config: AccuracyConfig) -> Self {
        Self { config }
    }
    
    /// Create a new validation rule executor with default configuration
    pub fn with_defaults() -> Self {
        Self::new(AccuracyConfig::default())
    }
    
    /// Validate UUID format for all POA&M items
    /// 
    /// Checks that UUID fields follow RFC 4122 format specification.
    /// Returns detailed results including failed item UUIDs for remediation.
    pub fn validate_uuid_format(&self, poam_items: &[PoamItem]) -> Result<ValidationRuleResult> {
        let mut passed = 0;
        let mut failed_uuids = Vec::new();
        
        if !self.config.strict_uuid_validation {
            // If strict validation is disabled, consider all UUIDs valid
            return Ok(ValidationRuleResult {
                rule_name: "uuid_format".to_string(),
                description: "UUID validation disabled".to_string(),
                passed_items: poam_items.len(),
                failed_items: 0,
                success_rate: 1.0,
                failed_item_uuids: Vec::new(),
            });
        }
        
        let uuid_pattern = uuid_regex();
        
        for item in poam_items {
            if uuid_pattern.is_match(&item.uuid) {
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
    
    /// Validate date formats for all POA&M items
    /// 
    /// Checks that date fields follow configured date format patterns.
    /// Validates both scheduled and actual completion dates.
    pub fn validate_date_formats(&self, poam_items: &[PoamItem]) -> Result<ValidationRuleResult> {
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
    
    /// Validate status values for all POA&M items
    /// 
    /// Checks that status fields contain only allowed values from configuration.
    pub fn validate_status_values(&self, poam_items: &[PoamItem]) -> Result<ValidationRuleResult> {
        let mut passed = 0;
        let mut failed_uuids = Vec::new();
        
        for item in poam_items {
            if self.config.valid_statuses.contains(&item.status) {
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
            description: format!("Status must be one of: {}", self.config.valid_statuses.join(", ")),
            passed_items: passed,
            failed_items: failed,
            success_rate,
            failed_item_uuids: failed_uuids,
        })
    }
    
    /// Validate severity values for all POA&M items
    /// 
    /// Checks that severity fields contain only allowed values from configuration.
    /// Treats missing severity as valid (optional field).
    pub fn validate_severity_values(&self, poam_items: &[PoamItem]) -> Result<ValidationRuleResult> {
        let mut passed = 0;
        let mut failed_uuids = Vec::new();
        
        for item in poam_items {
            let severity_valid = item.severity.as_ref()
                .map_or(true, |s| self.config.valid_severities.contains(s));
            
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
            description: format!("Severity must be one of: {}", self.config.valid_severities.join(", ")),
            passed_items: passed,
            failed_items: failed,
            success_rate,
            failed_item_uuids: failed_uuids,
        })
    }
    
    /// Validate text field quality for all POA&M items
    /// 
    /// Checks that text fields meet minimum length requirements and quality standards.
    /// Uses configured thresholds for different text fields.
    pub fn validate_text_quality(&self, poam_items: &[PoamItem]) -> Result<ValidationRuleResult> {
        let mut passed = 0;
        let mut failed_uuids = Vec::new();
        
        let title_min_length = self.config.text_length_thresholds.get("title").copied().unwrap_or(5);
        let description_min_length = self.config.text_length_thresholds.get("description").copied().unwrap_or(10);
        
        for item in poam_items {
            let mut item_valid = true;
            
            // Check title quality (minimum length, not just whitespace)
            if item.title.trim().len() < title_min_length {
                item_valid = false;
            }
            
            // Check description quality
            if item.description.trim().len() < description_min_length {
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
    
    /// Validate date logic for all POA&M items
    /// 
    /// Checks logical relationships between dates, such as ensuring actual completion
    /// dates are not unreasonably before scheduled completion dates.
    pub fn validate_date_logic(&self, poam_items: &[PoamItem]) -> Result<ValidationRuleResult> {
        let mut passed = 0;
        let mut failed_uuids = Vec::new();
        
        if !self.config.enable_date_logic_validation {
            // If date logic validation is disabled, consider all items valid
            return Ok(ValidationRuleResult {
                rule_name: "date_logic".to_string(),
                description: "Date logic validation disabled".to_string(),
                passed_items: poam_items.len(),
                failed_items: 0,
                success_rate: 1.0,
                failed_item_uuids: Vec::new(),
            });
        }
        
        for item in poam_items {
            let mut item_valid = true;
            
            // If both dates are present, check logical consistency
            if let (Some(scheduled), Some(actual)) = (&item.scheduled_completion_date, &item.actual_completion_date) {
                if let (Ok(scheduled_dt), Ok(actual_dt)) = (
                    self.parse_date_string(scheduled),
                    self.parse_date_string(actual)
                ) {
                    // Allow early completion, but flag if actual is significantly before scheduled
                    let days_early = (scheduled_dt - actual_dt).num_days();
                    if days_early > 365 {  // More than a year early might indicate data error
                        item_valid = false;
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
        for pattern in &self.config.date_patterns {
            if let Ok(naive_dt) = NaiveDate::parse_from_str(date_str, pattern) {
                return Ok(naive_dt.and_hms_opt(0, 0, 0).unwrap().and_utc());
            }
        }
        
        Err(Error::document_parsing(format!("Invalid date format: {}", date_str)))
    }
}

impl Default for ValidationRuleExecutor {
    fn default() -> Self {
        Self::with_defaults()
    }
}
