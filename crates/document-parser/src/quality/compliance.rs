//! Compliance Assessment for POA&M Items
//! 
//! Validates POA&M data against FedRAMP requirements, OSCAL schema compliance, and regulatory standards

use super::*;
use crate::poam::PoamItem;
use fedramp_core::{Result, Error};
use tracing::{debug, info};
use uuid::Uuid;
use std::collections::HashMap;

/// Result of compliance assessment
#[derive(Debug, Clone)]
pub struct ComplianceResult {
    /// Overall compliance score (0.0 to 1.0)
    pub score: f64,
    /// Compliance findings
    pub findings: Vec<QualityFinding>,
    /// FedRAMP compliance results
    pub fedramp_results: Vec<ComplianceCheckResult>,
    /// OSCAL compliance results
    pub oscal_results: Vec<ComplianceCheckResult>,
    /// Regulatory compliance results
    pub regulatory_results: Vec<ComplianceCheckResult>,
}

/// Individual compliance check result
#[derive(Debug, Clone)]
pub struct ComplianceCheckResult {
    /// Check name/identifier
    pub check_name: String,
    /// Check description
    pub description: String,
    /// Compliance standard (e.g., "FedRAMP", "OSCAL", "NIST")
    pub standard: String,
    /// Items that passed the check
    pub passed_items: usize,
    /// Items that failed the check
    pub failed_items: usize,
    /// Items not applicable to this check
    pub not_applicable_items: usize,
    /// Compliance rate (passed / (passed + failed))
    pub compliance_rate: f64,
    /// Severity of non-compliance
    pub severity: QualitySeverity,
    /// Failed item details
    pub failed_details: Vec<ComplianceViolation>,
}

/// Compliance violation details
#[derive(Debug, Clone)]
pub struct ComplianceViolation {
    /// Item UUID with the violation
    pub item_uuid: String,
    /// Violation description
    pub violation_description: String,
    /// Required action to achieve compliance
    pub required_action: String,
    /// Compliance requirement reference
    pub requirement_reference: Option<String>,
}

/// Compliance assessor for POA&M data
#[derive(Debug, Clone)]
pub struct ComplianceAssessor {
    /// Enable FedRAMP compliance checks
    enable_fedramp_checks: bool,
    /// Enable OSCAL compliance checks
    enable_oscal_checks: bool,
    /// Enable regulatory compliance checks
    enable_regulatory_checks: bool,
    /// Required FedRAMP fields
    fedramp_required_fields: Vec<String>,
    /// OSCAL schema requirements
    oscal_requirements: Vec<String>,
}

impl ComplianceAssessor {
    /// Create a new compliance assessor with default configuration
    pub fn new() -> Self {
        Self {
            enable_fedramp_checks: true,
            enable_oscal_checks: true,
            enable_regulatory_checks: true,
            fedramp_required_fields: vec![
                "uuid".to_string(),
                "title".to_string(),
                "description".to_string(),
                "status".to_string(),
                "scheduled_completion_date".to_string(),
                "responsible_entity".to_string(),
                "resources_required".to_string(),
            ],
            oscal_requirements: vec![
                "valid_uuid_format".to_string(),
                "iso8601_dates".to_string(),
                "required_metadata".to_string(),
                "valid_status_values".to_string(),
            ],
        }
    }

    /// Create a compliance assessor with custom configuration
    pub fn with_config(config: &QualityConfig) -> Self {
        let mut assessor = Self::new();
        assessor.fedramp_required_fields = config.required_fields.clone();
        assessor
    }

    /// Assess compliance of POA&M items
    pub fn assess(&self, poam_items: &[PoamItem]) -> Result<ComplianceResult> {
        info!("Assessing compliance for {} POA&M items", poam_items.len());

        if poam_items.is_empty() {
            return Ok(ComplianceResult {
                score: 1.0,
                findings: Vec::new(),
                fedramp_results: Vec::new(),
                oscal_results: Vec::new(),
                regulatory_results: Vec::new(),
            });
        }

        // Perform FedRAMP compliance checks
        let fedramp_results = if self.enable_fedramp_checks {
            self.assess_fedramp_compliance(poam_items)?
        } else {
            Vec::new()
        };

        // Perform OSCAL compliance checks
        let oscal_results = if self.enable_oscal_checks {
            self.assess_oscal_compliance(poam_items)?
        } else {
            Vec::new()
        };

        // Perform regulatory compliance checks
        let regulatory_results = if self.enable_regulatory_checks {
            self.assess_regulatory_compliance(poam_items)?
        } else {
            Vec::new()
        };

        // Generate compliance findings
        let findings = self.generate_compliance_findings(&fedramp_results, &oscal_results, &regulatory_results)?;

        // Calculate overall compliance score
        let overall_score = self.calculate_compliance_score(&fedramp_results, &oscal_results, &regulatory_results);

        debug!(
            "Compliance assessment completed: Score: {:.2}, Findings: {}",
            overall_score,
            findings.len()
        );

        Ok(ComplianceResult {
            score: overall_score,
            findings,
            fedramp_results,
            oscal_results,
            regulatory_results,
        })
    }

    /// Assess FedRAMP compliance requirements
    fn assess_fedramp_compliance(&self, poam_items: &[PoamItem]) -> Result<Vec<ComplianceCheckResult>> {
        let mut results = Vec::new();

        // Check required field completeness
        results.push(self.check_fedramp_required_fields(poam_items)?);

        // Check POA&M item quality standards
        results.push(self.check_fedramp_quality_standards(poam_items)?);

        // Check remediation timeline requirements
        results.push(self.check_fedramp_timeline_requirements(poam_items)?);

        // Check risk assessment requirements
        results.push(self.check_fedramp_risk_requirements(poam_items)?);

        Ok(results)
    }

    /// Check FedRAMP required fields
    fn check_fedramp_required_fields(&self, poam_items: &[PoamItem]) -> Result<ComplianceCheckResult> {
        let mut passed = 0;
        let mut failed_details = Vec::new();

        for item in poam_items {
            let mut item_compliant = true;
            let mut missing_fields = Vec::new();

            for field in &self.fedramp_required_fields {
                if !self.is_field_populated(item, field) {
                    item_compliant = false;
                    missing_fields.push(field.clone());
                }
            }

            if item_compliant {
                passed += 1;
            } else {
                failed_details.push(ComplianceViolation {
                    item_uuid: item.uuid.clone(),
                    violation_description: format!("Missing required FedRAMP fields: {}", missing_fields.join(", ")),
                    required_action: "Populate all required FedRAMP fields".to_string(),
                    requirement_reference: Some("FedRAMP POA&M Template Requirements".to_string()),
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
            check_name: "fedramp_required_fields".to_string(),
            description: "All FedRAMP required fields must be populated".to_string(),
            standard: "FedRAMP".to_string(),
            passed_items: passed,
            failed_items: failed,
            not_applicable_items: 0,
            compliance_rate,
            severity: QualitySeverity::High,
            failed_details,
        })
    }

    /// Check FedRAMP quality standards
    fn check_fedramp_quality_standards(&self, poam_items: &[PoamItem]) -> Result<ComplianceCheckResult> {
        let mut passed = 0;
        let mut failed_details = Vec::new();

        for item in poam_items {
            let mut item_compliant = true;
            let mut quality_issues = Vec::new();

            // Check title quality (minimum 10 characters, descriptive)
            if item.title.trim().len() < 10 {
                item_compliant = false;
                quality_issues.push("Title too short (minimum 10 characters)".to_string());
            }

            // Check description quality (minimum 50 characters, detailed)
            if item.description.trim().len() < 50 {
                item_compliant = false;
                quality_issues.push("Description too short (minimum 50 characters)".to_string());
            }

            // Check responsible entity is specified
            if item.responsible_entity.as_ref().map_or(true, |s| s.trim().is_empty()) {
                item_compliant = false;
                quality_issues.push("Responsible entity not specified".to_string());
            }

            if item_compliant {
                passed += 1;
            } else {
                failed_details.push(ComplianceViolation {
                    item_uuid: item.uuid.clone(),
                    violation_description: format!("Quality standard violations: {}", quality_issues.join("; ")),
                    required_action: "Improve data quality to meet FedRAMP standards".to_string(),
                    requirement_reference: Some("FedRAMP POA&M Quality Requirements".to_string()),
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
            check_name: "fedramp_quality_standards".to_string(),
            description: "POA&M items must meet FedRAMP quality standards".to_string(),
            standard: "FedRAMP".to_string(),
            passed_items: passed,
            failed_items: failed,
            not_applicable_items: 0,
            compliance_rate,
            severity: QualitySeverity::Medium,
            failed_details,
        })
    }

    /// Check FedRAMP timeline requirements
    fn check_fedramp_timeline_requirements(&self, poam_items: &[PoamItem]) -> Result<ComplianceCheckResult> {
        let mut passed = 0;
        let mut failed_details = Vec::new();

        for item in poam_items {
            let mut item_compliant = true;
            let mut timeline_issues = Vec::new();

            // Check that scheduled completion date is present
            if item.scheduled_completion_date.is_none() {
                item_compliant = false;
                timeline_issues.push("Missing scheduled completion date".to_string());
            }

            // Check that completed items have actual completion date
            if item.status == "Completed" && item.actual_completion_date.is_none() {
                item_compliant = false;
                timeline_issues.push("Completed item missing actual completion date".to_string());
            }

            if item_compliant {
                passed += 1;
            } else {
                failed_details.push(ComplianceViolation {
                    item_uuid: item.uuid.clone(),
                    violation_description: format!("Timeline requirement violations: {}", timeline_issues.join("; ")),
                    required_action: "Ensure all timeline requirements are met".to_string(),
                    requirement_reference: Some("FedRAMP POA&M Timeline Requirements".to_string()),
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
            check_name: "fedramp_timeline_requirements".to_string(),
            description: "POA&M items must meet FedRAMP timeline requirements".to_string(),
            standard: "FedRAMP".to_string(),
            passed_items: passed,
            failed_items: failed,
            not_applicable_items: 0,
            compliance_rate,
            severity: QualitySeverity::High,
            failed_details,
        })
    }

    /// Check FedRAMP risk assessment requirements
    fn check_fedramp_risk_requirements(&self, poam_items: &[PoamItem]) -> Result<ComplianceCheckResult> {
        let mut passed = 0;
        let mut failed_details = Vec::new();

        for item in poam_items {
            let mut item_compliant = true;
            let mut risk_issues = Vec::new();

            // Check that severity is specified for open items
            if matches!(item.status.as_str(), "Open" | "In Progress") && item.severity.is_none() {
                item_compliant = false;
                risk_issues.push("Open/In Progress item missing severity assessment".to_string());
            }

            // Check that risk assessment is provided
            if item.risk_assessment.as_ref().map_or(true, |s| s.trim().is_empty()) {
                item_compliant = false;
                risk_issues.push("Missing risk assessment".to_string());
            }

            if item_compliant {
                passed += 1;
            } else {
                failed_details.push(ComplianceViolation {
                    item_uuid: item.uuid.clone(),
                    violation_description: format!("Risk requirement violations: {}", risk_issues.join("; ")),
                    required_action: "Complete risk assessment and severity classification".to_string(),
                    requirement_reference: Some("FedRAMP Risk Assessment Requirements".to_string()),
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
            check_name: "fedramp_risk_requirements".to_string(),
            description: "POA&M items must include proper risk assessment".to_string(),
            standard: "FedRAMP".to_string(),
            passed_items: passed,
            failed_items: failed,
            not_applicable_items: 0,
            compliance_rate,
            severity: QualitySeverity::High,
            failed_details,
        })
    }

    /// Assess OSCAL compliance requirements
    fn assess_oscal_compliance(&self, poam_items: &[PoamItem]) -> Result<Vec<ComplianceCheckResult>> {
        let mut results = Vec::new();

        // Check UUID format compliance
        results.push(self.check_oscal_uuid_format(poam_items)?);

        // Check date format compliance
        results.push(self.check_oscal_date_format(poam_items)?);

        Ok(results)
    }

    /// Check OSCAL UUID format compliance
    fn check_oscal_uuid_format(&self, poam_items: &[PoamItem]) -> Result<ComplianceCheckResult> {
        let uuid_regex = regex::Regex::new(r"^[0-9a-fA-F]{8}-[0-9a-fA-F]{4}-[0-9a-fA-F]{4}-[0-9a-fA-F]{4}-[0-9a-fA-F]{12}$")
            .map_err(|e| Error::document_parsing(format!("Invalid UUID regex: {}", e)))?;

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
            check_name: "oscal_uuid_format".to_string(),
            description: "UUIDs must conform to OSCAL/RFC 4122 format".to_string(),
            standard: "OSCAL".to_string(),
            passed_items: passed,
            failed_items: failed,
            not_applicable_items: 0,
            compliance_rate,
            severity: QualitySeverity::High,
            failed_details,
        })
    }

    /// Check OSCAL date format compliance
    fn check_oscal_date_format(&self, poam_items: &[PoamItem]) -> Result<ComplianceCheckResult> {
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
            check_name: "oscal_date_format".to_string(),
            description: "Dates must be in ISO 8601 format".to_string(),
            standard: "OSCAL".to_string(),
            passed_items: passed,
            failed_items: failed,
            not_applicable_items: 0,
            compliance_rate,
            severity: QualitySeverity::Medium,
            failed_details,
        })
    }

    /// Assess regulatory compliance requirements
    fn assess_regulatory_compliance(&self, _poam_items: &[PoamItem]) -> Result<Vec<ComplianceCheckResult>> {
        // Placeholder for regulatory compliance checks
        // This could include FISMA, NIST, or other regulatory requirements
        Ok(Vec::new())
    }

    /// Generate compliance findings
    fn generate_compliance_findings(
        &self,
        fedramp_results: &[ComplianceCheckResult],
        oscal_results: &[ComplianceCheckResult],
        regulatory_results: &[ComplianceCheckResult],
    ) -> Result<Vec<QualityFinding>> {
        let mut findings = Vec::new();

        // Process all compliance results
        let all_results = fedramp_results.iter()
            .chain(oscal_results.iter())
            .chain(regulatory_results.iter());

        for result in all_results {
            if result.compliance_rate < 0.9 && result.failed_items > 0 {
                let affected_items: Vec<String> = result.failed_details.iter()
                    .map(|v| v.item_uuid.clone())
                    .collect();

                findings.push(QualityFinding {
                    id: Uuid::new_v4().to_string(),
                    severity: result.severity.clone(),
                    category: QualityCategory::Compliance,
                    description: format!(
                        "{} compliance check '{}' failed for {} items ({:.1}% compliance rate)",
                        result.standard,
                        result.check_name,
                        result.failed_items,
                        result.compliance_rate * 100.0
                    ),
                    affected_items,
                    impact_assessment: format!(
                        "{}. Non-compliance may affect certification and regulatory approval.",
                        result.description
                    ),
                    recommendation: format!(
                        "Address {} compliance violations for {} standard",
                        result.failed_items,
                        result.standard
                    ),
                    location: Some(format!("compliance_check.{}.{}", result.standard.to_lowercase(), result.check_name)),
                    metadata: HashMap::new(),
                });
            }
        }

        Ok(findings)
    }

    /// Calculate overall compliance score
    fn calculate_compliance_score(
        &self,
        fedramp_results: &[ComplianceCheckResult],
        oscal_results: &[ComplianceCheckResult],
        regulatory_results: &[ComplianceCheckResult],
    ) -> f64 {
        let all_results = fedramp_results.iter()
            .chain(oscal_results.iter())
            .chain(regulatory_results.iter());

        let mut total_score = 0.0;
        let mut count = 0;

        for result in all_results {
            total_score += result.compliance_rate;
            count += 1;
        }

        if count > 0 {
            total_score / count as f64
        } else {
            1.0
        }
    }

    /// Check if a field is populated for a POA&M item
    fn is_field_populated(&self, item: &PoamItem, field: &str) -> bool {
        match field {
            "uuid" => !item.uuid.is_empty(),
            "title" => !item.title.is_empty(),
            "description" => !item.description.is_empty(),
            "status" => !item.status.is_empty(),
            "scheduled_completion_date" => item.scheduled_completion_date.is_some(),
            "actual_completion_date" => item.actual_completion_date.is_some(),
            "responsible_entity" => item.responsible_entity.as_ref().map_or(false, |s| !s.is_empty()),
            "resources_required" => item.resources_required.as_ref().map_or(false, |s| !s.is_empty()),
            "severity" => item.severity.as_ref().map_or(false, |s| !s.is_empty()),
            "risk_assessment" => item.risk_assessment.as_ref().map_or(false, |s| !s.is_empty()),
            _ => false,
        }
    }

    /// Check if a date string is in ISO 8601 format
    fn is_iso8601_date(&self, date_str: &str) -> bool {
        chrono::DateTime::parse_from_rfc3339(date_str).is_ok()
    }
}

impl Default for ComplianceAssessor {
    fn default() -> Self {
        Self::new()
    }
}
