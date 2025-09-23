//! Modified: 2025-01-22

//! Regulatory Compliance Checks
//! 
//! Implementation of regulatory compliance validation for POA&M items including FISMA, NIST, and other standards

use super::types::*;
use super::super::*;
use crate::poam::PoamItem;
use fedramp_core::Result;
use std::collections::HashMap;

/// Regulatory compliance checker
#[derive(Debug, Clone)]
pub struct RegulatoryComplianceChecker {
    /// Regulatory requirements to check
    requirements: Vec<String>,
    /// Enable FISMA compliance checking
    enable_fisma_checks: bool,
    /// Enable NIST compliance checking
    enable_nist_checks: bool,
    /// Enable custom regulatory checks
    enable_custom_checks: bool,
    /// Custom regulatory rules
    custom_rules: HashMap<String, RegulatoryRule>,
}

/// Regulatory compliance rule
#[derive(Debug, Clone)]
pub struct RegulatoryRule {
    /// Rule name
    pub name: String,
    /// Rule description
    pub description: String,
    /// Regulatory standard (e.g., "FISMA", "NIST", "SOX")
    pub standard: String,
    /// Rule severity
    pub severity: QualitySeverity,
    /// Required fields for this rule
    pub required_fields: Vec<String>,
    /// Validation function name
    pub validation_function: String,
}

impl RegulatoryComplianceChecker {
    /// Create a new regulatory compliance checker
    pub fn new() -> Self {
        Self {
            requirements: vec![
                "fisma_compliance".to_string(),
                "nist_compliance".to_string(),
                "fedramp_baseline".to_string(),
            ],
            enable_fisma_checks: true,
            enable_nist_checks: true,
            enable_custom_checks: false,
            custom_rules: HashMap::new(),
        }
    }

    /// Create a regulatory compliance checker with custom configuration
    pub fn with_config(config: &ComplianceConfig) -> Self {
        Self {
            requirements: config.regulatory_requirements.clone(),
            enable_fisma_checks: true,
            enable_nist_checks: true,
            enable_custom_checks: false,
            custom_rules: HashMap::new(),
        }
    }

    /// Add a custom regulatory rule
    pub fn add_custom_rule(&mut self, rule: RegulatoryRule) {
        self.custom_rules.insert(rule.name.clone(), rule);
        self.enable_custom_checks = true;
    }

    /// Assess regulatory compliance requirements
    pub fn assess_compliance(&self, poam_items: &[PoamItem]) -> Result<Vec<ComplianceCheckResult>> {
        let mut results = Vec::new();

        // Check FISMA compliance
        if self.enable_fisma_checks {
            results.push(self.check_fisma_compliance(poam_items)?);
        }

        // Check NIST compliance
        if self.enable_nist_checks {
            results.push(self.check_nist_compliance(poam_items)?);
        }

        // Check custom regulatory rules
        if self.enable_custom_checks {
            for rule in self.custom_rules.values() {
                results.push(self.check_custom_rule(poam_items, rule)?);
            }
        }

        Ok(results)
    }

    /// Check FISMA compliance requirements
    pub fn check_fisma_compliance(&self, poam_items: &[PoamItem]) -> Result<ComplianceCheckResult> {
        let mut passed = 0;
        let mut failed_details = Vec::new();

        for item in poam_items {
            let mut item_compliant = true;
            let mut fisma_issues = Vec::new();

            // FISMA requires proper categorization of security controls
            if item.severity.is_none() {
                item_compliant = false;
                fisma_issues.push("Missing security control categorization (severity)".to_string());
            }

            // FISMA requires risk assessment
            if item.risk_assessment.as_ref().map_or(true, |s| s.trim().is_empty()) {
                item_compliant = false;
                fisma_issues.push("Missing FISMA-required risk assessment".to_string());
            }

            // FISMA requires responsible entity identification
            if item.responsible_entity.as_ref().map_or(true, |s| s.trim().is_empty()) {
                item_compliant = false;
                fisma_issues.push("Missing responsible entity identification".to_string());
            }

            // FISMA requires remediation timeline
            if item.scheduled_completion_date.is_none() {
                item_compliant = false;
                fisma_issues.push("Missing FISMA-required remediation timeline".to_string());
            }

            if item_compliant {
                passed += 1;
            } else {
                failed_details.push(ComplianceViolation {
                    item_uuid: item.uuid.clone(),
                    violation_description: format!("FISMA compliance violations: {}", fisma_issues.join("; ")),
                    required_action: "Address FISMA compliance requirements".to_string(),
                    requirement_reference: Some("FISMA Security Control Requirements".to_string()),
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
            check_name: ComplianceCheckType::RegulatoryFismaCompliance.check_name(),
            description: "POA&M items must meet FISMA compliance requirements".to_string(),
            standard: ComplianceCheckType::RegulatoryFismaCompliance.standard().to_string(),
            passed_items: passed,
            failed_items: failed,
            not_applicable_items: 0,
            compliance_rate,
            severity: ComplianceCheckType::RegulatoryFismaCompliance.default_severity(),
            failed_details,
        })
    }

    /// Check NIST compliance requirements
    pub fn check_nist_compliance(&self, poam_items: &[PoamItem]) -> Result<ComplianceCheckResult> {
        let mut passed = 0;
        let mut failed_details = Vec::new();

        for item in poam_items {
            let mut item_compliant = true;
            let mut nist_issues = Vec::new();

            // NIST requires detailed vulnerability description
            if item.description.trim().len() < 100 {
                item_compliant = false;
                nist_issues.push("Insufficient vulnerability description for NIST requirements (minimum 100 characters)".to_string());
            }

            // NIST requires impact assessment
            if item.risk_assessment.as_ref().map_or(true, |s| s.trim().len() < 50) {
                item_compliant = false;
                nist_issues.push("Insufficient impact assessment for NIST requirements".to_string());
            }

            // NIST requires proper status tracking
            if !matches!(item.status.to_lowercase().as_str(), "open" | "in progress" | "completed" | "closed") {
                item_compliant = false;
                nist_issues.push("Invalid status for NIST compliance".to_string());
            }

            if item_compliant {
                passed += 1;
            } else {
                failed_details.push(ComplianceViolation {
                    item_uuid: item.uuid.clone(),
                    violation_description: format!("NIST compliance violations: {}", nist_issues.join("; ")),
                    required_action: "Address NIST compliance requirements".to_string(),
                    requirement_reference: Some("NIST SP 800-53 Requirements".to_string()),
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
            check_name: ComplianceCheckType::RegulatoryNistCompliance.check_name(),
            description: "POA&M items must meet NIST compliance requirements".to_string(),
            standard: ComplianceCheckType::RegulatoryNistCompliance.standard().to_string(),
            passed_items: passed,
            failed_items: failed,
            not_applicable_items: 0,
            compliance_rate,
            severity: ComplianceCheckType::RegulatoryNistCompliance.default_severity(),
            failed_details,
        })
    }

    /// Check custom regulatory rule
    pub fn check_custom_rule(&self, poam_items: &[PoamItem], rule: &RegulatoryRule) -> Result<ComplianceCheckResult> {
        let mut passed = 0;
        let mut failed_details = Vec::new();

        for item in poam_items {
            let mut item_compliant = true;
            let mut rule_issues = Vec::new();

            // Check required fields for this rule
            for field in &rule.required_fields {
                if !self.is_field_populated(item, field) {
                    item_compliant = false;
                    rule_issues.push(format!("Missing required field: {}", field));
                }
            }

            // Apply custom validation based on rule type
            match rule.validation_function.as_str() {
                "validate_security_controls" => {
                    if !self.validate_security_controls(item) {
                        item_compliant = false;
                        rule_issues.push("Security controls validation failed".to_string());
                    }
                }
                "validate_documentation" => {
                    if !self.validate_documentation(item) {
                        item_compliant = false;
                        rule_issues.push("Documentation validation failed".to_string());
                    }
                }
                _ => {
                    // Default validation - just check required fields
                }
            }

            if item_compliant {
                passed += 1;
            } else {
                failed_details.push(ComplianceViolation {
                    item_uuid: item.uuid.clone(),
                    violation_description: format!("{} violations: {}", rule.name, rule_issues.join("; ")),
                    required_action: format!("Address {} compliance requirements", rule.standard),
                    requirement_reference: Some(format!("{} - {}", rule.standard, rule.name)),
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
            check_name: rule.name.clone(),
            description: rule.description.clone(),
            standard: rule.standard.clone(),
            passed_items: passed,
            failed_items: failed,
            not_applicable_items: 0,
            compliance_rate,
            severity: rule.severity.clone(),
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

    /// Validate security controls for regulatory compliance
    fn validate_security_controls(&self, item: &PoamItem) -> bool {
        // Check that severity is properly categorized
        if let Some(severity) = &item.severity {
            matches!(severity.to_lowercase().as_str(), "low" | "medium" | "high" | "critical")
        } else {
            false
        }
    }

    /// Validate documentation requirements for regulatory compliance
    fn validate_documentation(&self, item: &PoamItem) -> bool {
        // Check that description is sufficiently detailed
        item.description.trim().len() >= 50
            && item.risk_assessment.as_ref().map_or(false, |s| s.trim().len() >= 30)
    }

    /// Get available regulatory standards
    pub fn available_standards() -> Vec<&'static str> {
        vec!["FISMA", "NIST", "SOX", "HIPAA", "PCI-DSS", "ISO-27001"]
    }

    /// Create a standard FISMA rule
    pub fn create_fisma_rule() -> RegulatoryRule {
        RegulatoryRule {
            name: "fisma_baseline".to_string(),
            description: "FISMA baseline security requirements".to_string(),
            standard: "FISMA".to_string(),
            severity: QualitySeverity::High,
            required_fields: vec![
                "severity".to_string(),
                "risk_assessment".to_string(),
                "responsible_entity".to_string(),
                "scheduled_completion_date".to_string(),
            ],
            validation_function: "validate_security_controls".to_string(),
        }
    }

    /// Create a standard NIST rule
    pub fn create_nist_rule() -> RegulatoryRule {
        RegulatoryRule {
            name: "nist_sp800_53".to_string(),
            description: "NIST SP 800-53 security control requirements".to_string(),
            standard: "NIST".to_string(),
            severity: QualitySeverity::High,
            required_fields: vec![
                "description".to_string(),
                "risk_assessment".to_string(),
                "status".to_string(),
            ],
            validation_function: "validate_documentation".to_string(),
        }
    }
}

impl Default for RegulatoryComplianceChecker {
    fn default() -> Self {
        Self::new()
    }
}
