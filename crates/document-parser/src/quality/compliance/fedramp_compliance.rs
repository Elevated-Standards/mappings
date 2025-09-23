//! Modified: 2025-01-22

//! FedRAMP Compliance Checks
//! 
//! Implementation of FedRAMP-specific compliance validation for POA&M items

use super::types::*;
use super::super::*;
use crate::poam::PoamItem;
use fedramp_core::Result;

/// FedRAMP compliance checker
#[derive(Debug, Clone)]
pub struct FedRampComplianceChecker {
    /// Required FedRAMP fields
    required_fields: Vec<String>,
    /// Enable quality standards checking
    enable_quality_checks: bool,
    /// Enable timeline requirements checking
    enable_timeline_checks: bool,
    /// Enable risk assessment checking
    enable_risk_checks: bool,
}

impl FedRampComplianceChecker {
    /// Create a new FedRAMP compliance checker
    pub fn new() -> Self {
        Self {
            required_fields: vec![
                "uuid".to_string(),
                "title".to_string(),
                "description".to_string(),
                "status".to_string(),
                "scheduled_completion_date".to_string(),
                "responsible_entity".to_string(),
                "resources_required".to_string(),
            ],
            enable_quality_checks: true,
            enable_timeline_checks: true,
            enable_risk_checks: true,
        }
    }

    /// Create a FedRAMP compliance checker with custom configuration
    pub fn with_config(config: &ComplianceConfig) -> Self {
        Self {
            required_fields: config.fedramp_required_fields.clone(),
            enable_quality_checks: true,
            enable_timeline_checks: true,
            enable_risk_checks: true,
        }
    }

    /// Assess FedRAMP compliance requirements
    pub fn assess_compliance(&self, poam_items: &[PoamItem]) -> Result<Vec<ComplianceCheckResult>> {
        let mut results = Vec::new();

        // Check required field completeness
        results.push(self.check_required_fields(poam_items)?);

        // Check POA&M item quality standards
        if self.enable_quality_checks {
            results.push(self.check_quality_standards(poam_items)?);
        }

        // Check remediation timeline requirements
        if self.enable_timeline_checks {
            results.push(self.check_timeline_requirements(poam_items)?);
        }

        // Check risk assessment requirements
        if self.enable_risk_checks {
            results.push(self.check_risk_requirements(poam_items)?);
        }

        Ok(results)
    }

    /// Check FedRAMP required fields
    pub fn check_required_fields(&self, poam_items: &[PoamItem]) -> Result<ComplianceCheckResult> {
        let mut passed = 0;
        let mut failed_details = Vec::new();

        for item in poam_items {
            let mut item_compliant = true;
            let mut missing_fields = Vec::new();

            for field in &self.required_fields {
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
            check_name: ComplianceCheckType::FedRampRequiredFields.check_name(),
            description: "All FedRAMP required fields must be populated".to_string(),
            standard: ComplianceCheckType::FedRampRequiredFields.standard().to_string(),
            passed_items: passed,
            failed_items: failed,
            not_applicable_items: 0,
            compliance_rate,
            severity: ComplianceCheckType::FedRampRequiredFields.default_severity(),
            failed_details,
        })
    }

    /// Check FedRAMP quality standards
    pub fn check_quality_standards(&self, poam_items: &[PoamItem]) -> Result<ComplianceCheckResult> {
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
            check_name: ComplianceCheckType::FedRampQualityStandards.check_name(),
            description: "POA&M items must meet FedRAMP quality standards".to_string(),
            standard: ComplianceCheckType::FedRampQualityStandards.standard().to_string(),
            passed_items: passed,
            failed_items: failed,
            not_applicable_items: 0,
            compliance_rate,
            severity: ComplianceCheckType::FedRampQualityStandards.default_severity(),
            failed_details,
        })
    }

    /// Check FedRAMP timeline requirements
    pub fn check_timeline_requirements(&self, poam_items: &[PoamItem]) -> Result<ComplianceCheckResult> {
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
            check_name: ComplianceCheckType::FedRampTimelineRequirements.check_name(),
            description: "POA&M items must meet FedRAMP timeline requirements".to_string(),
            standard: ComplianceCheckType::FedRampTimelineRequirements.standard().to_string(),
            passed_items: passed,
            failed_items: failed,
            not_applicable_items: 0,
            compliance_rate,
            severity: ComplianceCheckType::FedRampTimelineRequirements.default_severity(),
            failed_details,
        })
    }

    /// Check FedRAMP risk assessment requirements
    pub fn check_risk_requirements(&self, poam_items: &[PoamItem]) -> Result<ComplianceCheckResult> {
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
            check_name: ComplianceCheckType::FedRampRiskRequirements.check_name(),
            description: "POA&M items must include proper risk assessment".to_string(),
            standard: ComplianceCheckType::FedRampRiskRequirements.standard().to_string(),
            passed_items: passed,
            failed_items: failed,
            not_applicable_items: 0,
            compliance_rate,
            severity: ComplianceCheckType::FedRampRiskRequirements.default_severity(),
            failed_details,
        })
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
}

impl Default for FedRampComplianceChecker {
    fn default() -> Self {
        Self::new()
    }
}
