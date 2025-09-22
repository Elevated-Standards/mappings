// Modified: 2025-01-22

//! Cross-field validation for POA&M documents
//!
//! This module provides validation that involves relationships between
//! multiple fields, ensuring data consistency across the entire document.

use super::types::{CrossFieldRule, FieldValidationResult, ValidationError, PoamStatus, PoamSeverity};
use crate::validation::types::{ValidationStatus, ValidationSeverity};
use fedramp_core::Result;
use serde_json::Value;
use std::collections::HashMap;
use chrono::NaiveDate;
use tracing::{info, warn};

/// Cross-field validator
#[derive(Debug, Clone)]
pub struct CrossFieldValidator {
    /// Cross-field rules
    pub cross_field_rules: Vec<CrossFieldRule>,
}

impl CrossFieldValidator {
    /// Create a new cross-field validator
    pub fn new() -> Self {
        let cross_field_rules = vec![
            CrossFieldRule {
                name: "completion_date_consistency".to_string(),
                primary_field: "completion_date".to_string(),
                related_fields: vec!["status".to_string(), "scheduled_completion_date".to_string()],
                validation_logic: "If status is 'Completed', completion_date must be set and not in the future".to_string(),
                error_message: "Completion date is required when status is 'Completed' and cannot be in the future".to_string(),
                severity: ValidationSeverity::Error,
            },
            CrossFieldRule {
                name: "scheduled_date_logic".to_string(),
                primary_field: "scheduled_completion_date".to_string(),
                related_fields: vec!["original_scheduled_completion_date".to_string(), "status".to_string()],
                validation_logic: "Scheduled completion date should not be before original scheduled date unless status is 'Deferred'".to_string(),
                error_message: "Scheduled completion date cannot be earlier than original date unless deferred".to_string(),
                severity: ValidationSeverity::Warning,
            },
            CrossFieldRule {
                name: "severity_status_consistency".to_string(),
                primary_field: "severity".to_string(),
                related_fields: vec!["status".to_string(), "completion_date".to_string()],
                validation_logic: "Critical severity items should not remain 'Open' for extended periods".to_string(),
                error_message: "Critical severity items should be addressed promptly".to_string(),
                severity: ValidationSeverity::Warning,
            },
            CrossFieldRule {
                name: "risk_acceptance_fields".to_string(),
                primary_field: "status".to_string(),
                related_fields: vec!["risk_acceptance_date".to_string(), "risk_acceptance_justification".to_string()],
                validation_logic: "If status is 'Risk Accepted', risk acceptance date and justification must be provided".to_string(),
                error_message: "Risk acceptance requires both date and justification".to_string(),
                severity: ValidationSeverity::Error,
            },
            CrossFieldRule {
                name: "milestone_consistency".to_string(),
                primary_field: "milestone_date".to_string(),
                related_fields: vec!["milestone_description".to_string(), "status".to_string()],
                validation_logic: "Milestone dates must have corresponding descriptions and be consistent with status".to_string(),
                error_message: "Milestone dates require descriptions and must align with current status".to_string(),
                severity: ValidationSeverity::Warning,
            },
        ];

        Self { cross_field_rules }
    }

    /// Validate cross-field relationships
    pub fn validate_cross_fields(
        &self,
        poam_data: &HashMap<String, Value>,
    ) -> Result<Vec<FieldValidationResult>> {
        let mut results = Vec::new();

        for rule in &self.cross_field_rules {
            let result = self.validate_cross_field_rule(rule, poam_data)?;
            results.push(result);
        }

        Ok(results)
    }

    /// Validate a single cross-field rule
    fn validate_cross_field_rule(
        &self,
        rule: &CrossFieldRule,
        poam_data: &HashMap<String, Value>,
    ) -> Result<FieldValidationResult> {
        match rule.name.as_str() {
            "completion_date_consistency" => self.validate_completion_date_consistency(poam_data),
            "scheduled_date_logic" => self.validate_scheduled_date_logic(poam_data),
            "severity_status_consistency" => self.validate_severity_status_consistency(poam_data),
            "risk_acceptance_fields" => self.validate_risk_acceptance_fields(poam_data),
            "milestone_consistency" => self.validate_milestone_consistency(poam_data),
            _ => {
                warn!("Unknown cross-field rule: {}", rule.name);
                Ok(FieldValidationResult {
                    field_name: rule.primary_field.clone(),
                    passed: true,
                    status: ValidationStatus::Valid,
                    error_message: None,
                    warning_message: None,
                    suggested_value: None,
                    confidence: 1.0,
                })
            }
        }
    }

    /// Validate completion date consistency
    fn validate_completion_date_consistency(
        &self,
        poam_data: &HashMap<String, Value>,
    ) -> Result<FieldValidationResult> {
        let status = poam_data.get("status").and_then(|v| v.as_str()).unwrap_or("");
        let completion_date = poam_data.get("completion_date").and_then(|v| v.as_str());

        if status.to_lowercase() == "completed" {
            if completion_date.is_none() || completion_date.unwrap().trim().is_empty() {
                return Ok(FieldValidationResult {
                    field_name: "completion_date".to_string(),
                    passed: false,
                    status: ValidationStatus::Invalid,
                    error_message: Some("Completion date is required when status is 'Completed'".to_string()),
                    warning_message: None,
                    suggested_value: Some(chrono::Utc::now().format("%Y-%m-%d").to_string()),
                    confidence: 0.0,
                });
            }

            // Check if completion date is in the future
            if let Some(date_str) = completion_date {
                if let Ok(completion_date) = NaiveDate::parse_from_str(date_str, "%Y-%m-%d") {
                    let today = chrono::Utc::now().date_naive();
                    if completion_date > today {
                        return Ok(FieldValidationResult {
                            field_name: "completion_date".to_string(),
                            passed: false,
                            status: ValidationStatus::Invalid,
                            error_message: Some("Completion date cannot be in the future".to_string()),
                            warning_message: None,
                            suggested_value: Some(today.format("%Y-%m-%d").to_string()),
                            confidence: 0.0,
                        });
                    }
                }
            }
        }

        Ok(FieldValidationResult {
            field_name: "completion_date".to_string(),
            passed: true,
            status: ValidationStatus::Valid,
            error_message: None,
            warning_message: None,
            suggested_value: None,
            confidence: 1.0,
        })
    }

    /// Validate scheduled date logic
    fn validate_scheduled_date_logic(
        &self,
        poam_data: &HashMap<String, Value>,
    ) -> Result<FieldValidationResult> {
        let scheduled_date = poam_data.get("scheduled_completion_date").and_then(|v| v.as_str());
        let original_date = poam_data.get("original_scheduled_completion_date").and_then(|v| v.as_str());
        let status = poam_data.get("status").and_then(|v| v.as_str()).unwrap_or("");

        if let (Some(scheduled_str), Some(original_str)) = (scheduled_date, original_date) {
            if let (Ok(scheduled), Ok(original)) = (
                NaiveDate::parse_from_str(scheduled_str, "%Y-%m-%d"),
                NaiveDate::parse_from_str(original_str, "%Y-%m-%d")
            ) {
                if scheduled < original && status.to_lowercase() != "deferred" {
                    return Ok(FieldValidationResult {
                        field_name: "scheduled_completion_date".to_string(),
                        passed: false,
                        status: ValidationStatus::Invalid,
                        error_message: None,
                        warning_message: Some("Scheduled completion date is earlier than original date but status is not 'Deferred'".to_string()),
                        suggested_value: Some(original_str.to_string()),
                        confidence: 0.7,
                    });
                }
            }
        }

        Ok(FieldValidationResult {
            field_name: "scheduled_completion_date".to_string(),
            passed: true,
            status: ValidationStatus::Valid,
            error_message: None,
            warning_message: None,
            suggested_value: None,
            confidence: 1.0,
        })
    }

    /// Validate severity and status consistency
    fn validate_severity_status_consistency(
        &self,
        poam_data: &HashMap<String, Value>,
    ) -> Result<FieldValidationResult> {
        let severity = poam_data.get("severity").and_then(|v| v.as_str()).unwrap_or("");
        let status = poam_data.get("status").and_then(|v| v.as_str()).unwrap_or("");
        let creation_date = poam_data.get("creation_date").and_then(|v| v.as_str());

        if severity.to_lowercase() == "critical" && status.to_lowercase() == "open" {
            if let Some(creation_str) = creation_date {
                if let Ok(creation_date) = NaiveDate::parse_from_str(creation_str, "%Y-%m-%d") {
                    let today = chrono::Utc::now().date_naive();
                    let days_open = (today - creation_date).num_days();
                    
                    if days_open > 30 {
                        return Ok(FieldValidationResult {
                            field_name: "severity".to_string(),
                            passed: false,
                            status: ValidationStatus::Invalid,
                            error_message: None,
                            warning_message: Some(format!("Critical severity item has been open for {} days", days_open)),
                            suggested_value: Some("Consider updating status or reviewing priority".to_string()),
                            confidence: 0.8,
                        });
                    }
                }
            }
        }

        Ok(FieldValidationResult {
            field_name: "severity".to_string(),
            passed: true,
            status: ValidationStatus::Valid,
            error_message: None,
            warning_message: None,
            suggested_value: None,
            confidence: 1.0,
        })
    }

    /// Validate risk acceptance fields
    fn validate_risk_acceptance_fields(
        &self,
        poam_data: &HashMap<String, Value>,
    ) -> Result<FieldValidationResult> {
        let status = poam_data.get("status").and_then(|v| v.as_str()).unwrap_or("");
        
        if status.to_lowercase() == "risk accepted" || status.to_lowercase() == "riskaccepted" {
            let risk_date = poam_data.get("risk_acceptance_date").and_then(|v| v.as_str());
            let risk_justification = poam_data.get("risk_acceptance_justification").and_then(|v| v.as_str());

            if risk_date.is_none() || risk_date.unwrap().trim().is_empty() {
                return Ok(FieldValidationResult {
                    field_name: "risk_acceptance_date".to_string(),
                    passed: false,
                    status: ValidationStatus::Invalid,
                    error_message: Some("Risk acceptance date is required when status is 'Risk Accepted'".to_string()),
                    warning_message: None,
                    suggested_value: Some(chrono::Utc::now().format("%Y-%m-%d").to_string()),
                    confidence: 0.0,
                });
            }

            if risk_justification.is_none() || risk_justification.unwrap().trim().is_empty() {
                return Ok(FieldValidationResult {
                    field_name: "risk_acceptance_justification".to_string(),
                    passed: false,
                    status: ValidationStatus::Invalid,
                    error_message: Some("Risk acceptance justification is required when status is 'Risk Accepted'".to_string()),
                    warning_message: None,
                    suggested_value: Some("Please provide justification for accepting this risk".to_string()),
                    confidence: 0.0,
                });
            }
        }

        Ok(FieldValidationResult {
            field_name: "status".to_string(),
            passed: true,
            status: ValidationStatus::Valid,
            error_message: None,
            warning_message: None,
            suggested_value: None,
            confidence: 1.0,
        })
    }

    /// Validate milestone consistency
    fn validate_milestone_consistency(
        &self,
        poam_data: &HashMap<String, Value>,
    ) -> Result<FieldValidationResult> {
        let milestone_date = poam_data.get("milestone_date").and_then(|v| v.as_str());
        let milestone_description = poam_data.get("milestone_description").and_then(|v| v.as_str());

        if let Some(date_str) = milestone_date {
            if !date_str.trim().is_empty() {
                if milestone_description.is_none() || milestone_description.unwrap().trim().is_empty() {
                    return Ok(FieldValidationResult {
                        field_name: "milestone_description".to_string(),
                        passed: false,
                        status: ValidationStatus::Invalid,
                        error_message: None,
                        warning_message: Some("Milestone date provided but description is missing".to_string()),
                        suggested_value: Some("Please provide a description for this milestone".to_string()),
                        confidence: 0.7,
                    });
                }
            }
        }

        Ok(FieldValidationResult {
            field_name: "milestone_date".to_string(),
            passed: true,
            status: ValidationStatus::Valid,
            error_message: None,
            warning_message: None,
            suggested_value: None,
            confidence: 1.0,
        })
    }

    /// Add a new cross-field rule
    pub fn add_rule(&mut self, rule: CrossFieldRule) {
        self.cross_field_rules.push(rule);
    }

    /// Remove a cross-field rule by name
    pub fn remove_rule(&mut self, rule_name: &str) -> bool {
        let initial_len = self.cross_field_rules.len();
        self.cross_field_rules.retain(|rule| rule.name != rule_name);
        self.cross_field_rules.len() < initial_len
    }

    /// Get all rule names
    pub fn get_rule_names(&self) -> Vec<String> {
        self.cross_field_rules.iter().map(|rule| rule.name.clone()).collect()
    }
}

impl Default for CrossFieldValidator {
    fn default() -> Self {
        Self::new()
    }
}
