//! Modified: 2025-01-22

//! OSCAL Compliance Checks
//! 
//! Implementation of OSCAL-specific compliance validation for POA&M items

use super::types::*;
use super::super::*;
use crate::poam::PoamItem;
use fedramp_core::{Result, Error};
use regex::Regex;
use std::sync::OnceLock;

/// OSCAL compliance checker
#[derive(Debug, Clone)]
pub struct OscalComplianceChecker {
    /// OSCAL requirements to check
    requirements: Vec<String>,
    /// Enable UUID format checking
    enable_uuid_checks: bool,
    /// Enable date format checking
    enable_date_checks: bool,
    /// Enable metadata requirements checking
    enable_metadata_checks: bool,
}

impl OscalComplianceChecker {
    /// Create a new OSCAL compliance checker
    pub fn new() -> Self {
        Self {
            requirements: vec![
                "valid_uuid_format".to_string(),
                "iso8601_dates".to_string(),
                "required_metadata".to_string(),
                "valid_status_values".to_string(),
            ],
            enable_uuid_checks: true,
            enable_date_checks: true,
            enable_metadata_checks: true,
        }
    }

    /// Create an OSCAL compliance checker with custom configuration
    pub fn with_config(config: &ComplianceConfig) -> Self {
        Self {
            requirements: config.oscal_requirements.clone(),
            enable_uuid_checks: true,
            enable_date_checks: true,
            enable_metadata_checks: true,
        }
    }

    /// Assess OSCAL compliance requirements
    pub fn assess_compliance(&self, poam_items: &[PoamItem]) -> Result<Vec<ComplianceCheckResult>> {
        let mut results = Vec::new();

        // Check UUID format compliance
        if self.enable_uuid_checks {
            results.push(self.check_uuid_format(poam_items)?);
        }

        // Check date format compliance
        if self.enable_date_checks {
            results.push(self.check_date_format(poam_items)?);
        }

        // Check metadata requirements
        if self.enable_metadata_checks {
            results.push(self.check_metadata_requirements(poam_items)?);
        }

        Ok(results)
    }

    /// Check OSCAL UUID format compliance
    pub fn check_uuid_format(&self, poam_items: &[PoamItem]) -> Result<ComplianceCheckResult> {
        static UUID_REGEX: OnceLock<Regex> = OnceLock::new();
        let uuid_regex = UUID_REGEX.get_or_init(|| {
            Regex::new(r"^[0-9a-fA-F]{8}-[0-9a-fA-F]{4}-[0-9a-fA-F]{4}-[0-9a-fA-F]{4}-[0-9a-fA-F]{12}$")
                .expect("Valid UUID regex")
        });

        let mut passed = 0;
        let mut failed_details = Vec::new();

        for item in poam_items {
            if uuid_regex.is_match(&item.uuid) {
                passed += 1;
            } else {
                failed_details.push(ComplianceViolation {
                    item_uuid: item.uuid.clone(),
                    violation_description: "UUID does not conform to RFC 4122 format".to_string(),
                    required_action: "Use properly formatted UUID (RFC 4122)".to_string(),
                    requirement_reference: Some("OSCAL UUID Requirements".to_string()),
                });
            }
        }

        let failed = poam_items.len() - passed;
        let compliance_rate = if !poam_items.is_empty() {
            passed as f64 / poam_items.len() as f64
        } else {
            1.0
        };

        Ok(ComplianceCheckResult {
            check_name: ComplianceCheckType::OscalUuidFormat.check_name(),
            description: "UUIDs must conform to OSCAL/RFC 4122 format".to_string(),
            standard: ComplianceCheckType::OscalUuidFormat.standard().to_string(),
            passed_items: passed,
            failed_items: failed,
            not_applicable_items: 0,
            compliance_rate,
            severity: ComplianceCheckType::OscalUuidFormat.default_severity(),
            failed_details,
        })
    }

    /// Check OSCAL date format compliance
    pub fn check_date_format(&self, poam_items: &[PoamItem]) -> Result<ComplianceCheckResult> {
        let mut passed = 0;
        let mut failed_details = Vec::new();

        for item in poam_items {
            let mut item_compliant = true;
            let mut date_issues = Vec::new();

            // Check scheduled completion date format
            if let Some(date_str) = &item.scheduled_completion_date {
                if !self.is_iso8601_date(date_str) {
                    item_compliant = false;
                    date_issues.push("Scheduled completion date not in ISO 8601 format".to_string());
                }
            }

            // Check actual completion date format
            if let Some(date_str) = &item.actual_completion_date {
                if !self.is_iso8601_date(date_str) {
                    item_compliant = false;
                    date_issues.push("Actual completion date not in ISO 8601 format".to_string());
                }
            }

            if item_compliant {
                passed += 1;
            } else {
                failed_details.push(ComplianceViolation {
                    item_uuid: item.uuid.clone(),
                    violation_description: format!("Date format violations: {}", date_issues.join("; ")),
                    required_action: "Use ISO 8601 date format (YYYY-MM-DDTHH:MM:SSZ)".to_string(),
                    requirement_reference: Some("OSCAL Date Format Requirements".to_string()),
                });
            }
        }

        let failed = poam_items.len() - passed;
        let compliance_rate = if !poam_items.is_empty() {
            passed as f64 / poam_items.len() as f64
        } else {
            1.0
        };

        Ok(ComplianceCheckResult {
            check_name: ComplianceCheckType::OscalDateFormat.check_name(),
            description: "Dates must be in ISO 8601 format".to_string(),
            standard: ComplianceCheckType::OscalDateFormat.standard().to_string(),
            passed_items: passed,
            failed_items: failed,
            not_applicable_items: 0,
            compliance_rate,
            severity: ComplianceCheckType::OscalDateFormat.default_severity(),
            failed_details,
        })
    }

    /// Check OSCAL metadata requirements
    pub fn check_metadata_requirements(&self, poam_items: &[PoamItem]) -> Result<ComplianceCheckResult> {
        let mut passed = 0;
        let mut failed_details = Vec::new();

        for item in poam_items {
            let mut item_compliant = true;
            let mut metadata_issues = Vec::new();

            // Check that UUID is present and valid
            if item.uuid.is_empty() {
                item_compliant = false;
                metadata_issues.push("Missing UUID".to_string());
            }

            // Check that title is present and meaningful
            if item.title.trim().is_empty() {
                item_compliant = false;
                metadata_issues.push("Missing title".to_string());
            }

            // Check that description is present
            if item.description.trim().is_empty() {
                item_compliant = false;
                metadata_issues.push("Missing description".to_string());
            }

            // Check that status is valid OSCAL status
            if !self.is_valid_oscal_status(&item.status) {
                item_compliant = false;
                metadata_issues.push(format!("Invalid OSCAL status: '{}'", item.status));
            }

            if item_compliant {
                passed += 1;
            } else {
                failed_details.push(ComplianceViolation {
                    item_uuid: item.uuid.clone(),
                    violation_description: format!("Metadata requirement violations: {}", metadata_issues.join("; ")),
                    required_action: "Ensure all required OSCAL metadata is present and valid".to_string(),
                    requirement_reference: Some("OSCAL Metadata Requirements".to_string()),
                });
            }
        }

        let failed = poam_items.len() - passed;
        let compliance_rate = if !poam_items.is_empty() {
            passed as f64 / poam_items.len() as f64
        } else {
            1.0
        };

        Ok(ComplianceCheckResult {
            check_name: ComplianceCheckType::OscalMetadataRequirements.check_name(),
            description: "POA&M items must include required OSCAL metadata".to_string(),
            standard: ComplianceCheckType::OscalMetadataRequirements.standard().to_string(),
            passed_items: passed,
            failed_items: failed,
            not_applicable_items: 0,
            compliance_rate,
            severity: ComplianceCheckType::OscalMetadataRequirements.default_severity(),
            failed_details,
        })
    }

    /// Check if a date string is in ISO 8601 format
    fn is_iso8601_date(&self, date_str: &str) -> bool {
        chrono::DateTime::parse_from_rfc3339(date_str).is_ok()
    }

    /// Check if a status value is valid according to OSCAL standards
    fn is_valid_oscal_status(&self, status: &str) -> bool {
        matches!(
            status.to_lowercase().as_str(),
            "open" | "in-progress" | "completed" | "closed" | "cancelled" | "deferred"
        )
    }

    /// Validate UUID format using regex
    pub fn validate_uuid_format(uuid: &str) -> bool {
        static UUID_REGEX: OnceLock<Regex> = OnceLock::new();
        let uuid_regex = UUID_REGEX.get_or_init(|| {
            Regex::new(r"^[0-9a-fA-F]{8}-[0-9a-fA-F]{4}-[0-9a-fA-F]{4}-[0-9a-fA-F]{4}-[0-9a-fA-F]{12}$")
                .expect("Valid UUID regex")
        });
        uuid_regex.is_match(uuid)
    }

    /// Validate date format using chrono
    pub fn validate_iso8601_date(date_str: &str) -> bool {
        chrono::DateTime::parse_from_rfc3339(date_str).is_ok()
    }

    /// Get list of valid OSCAL status values
    pub fn valid_oscal_statuses() -> Vec<&'static str> {
        vec!["open", "in-progress", "completed", "closed", "cancelled", "deferred"]
    }

    /// Normalize status value to OSCAL standard
    pub fn normalize_status(status: &str) -> String {
        match status.to_lowercase().trim() {
            "open" => "open".to_string(),
            "in progress" | "in-progress" | "inprogress" => "in-progress".to_string(),
            "complete" | "completed" | "done" => "completed".to_string(),
            "close" | "closed" => "closed".to_string(),
            "cancel" | "cancelled" | "canceled" => "cancelled".to_string(),
            "defer" | "deferred" | "postponed" => "deferred".to_string(),
            _ => status.to_string(), // Return original if no match
        }
    }

    /// Check if POA&M item has all required OSCAL fields
    pub fn has_required_oscal_fields(item: &PoamItem) -> bool {
        !item.uuid.is_empty()
            && !item.title.trim().is_empty()
            && !item.description.trim().is_empty()
            && !item.status.trim().is_empty()
    }

    /// Generate OSCAL compliance summary
    pub fn generate_compliance_summary(results: &[ComplianceCheckResult]) -> String {
        let total_checks = results.len();
        let passed_checks = results.iter().filter(|r| r.compliance_rate >= 0.9).count();
        let overall_rate = if total_checks > 0 {
            results.iter().map(|r| r.compliance_rate).sum::<f64>() / total_checks as f64
        } else {
            1.0
        };

        format!(
            "OSCAL Compliance Summary: {}/{} checks passed ({:.1}% overall compliance)",
            passed_checks,
            total_checks,
            overall_rate * 100.0
        )
    }
}

impl Default for OscalComplianceChecker {
    fn default() -> Self {
        Self::new()
    }
}
