//! Modified: 2025-01-22

//! Compliance Assessment Types
//! 
//! Type definitions for compliance assessment results, violations, and configuration

use super::super::*;
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

/// Configuration for compliance assessment
#[derive(Debug, Clone)]
pub struct ComplianceConfig {
    /// Enable FedRAMP compliance checks
    pub enable_fedramp_checks: bool,
    /// Enable OSCAL compliance checks
    pub enable_oscal_checks: bool,
    /// Enable regulatory compliance checks
    pub enable_regulatory_checks: bool,
    /// Required FedRAMP fields
    pub fedramp_required_fields: Vec<String>,
    /// OSCAL schema requirements
    pub oscal_requirements: Vec<String>,
    /// Regulatory requirements
    pub regulatory_requirements: Vec<String>,
    /// Minimum compliance rate threshold
    pub min_compliance_rate: f64,
    /// Severity threshold for findings generation
    pub severity_threshold: QualitySeverity,
}

impl Default for ComplianceConfig {
    fn default() -> Self {
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
            regulatory_requirements: vec![
                "fisma_compliance".to_string(),
                "nist_compliance".to_string(),
                "fedramp_baseline".to_string(),
            ],
            min_compliance_rate: 0.9,
            severity_threshold: QualitySeverity::Medium,
        }
    }
}

impl From<&super::super::QualityConfig> for ComplianceConfig {
    fn from(quality_config: &super::super::QualityConfig) -> Self {
        Self {
            enable_fedramp_checks: true,
            enable_oscal_checks: true,
            enable_regulatory_checks: true,
            fedramp_required_fields: quality_config.required_fields.clone(),
            oscal_requirements: vec![
                "valid_uuid_format".to_string(),
                "iso8601_dates".to_string(),
                "required_metadata".to_string(),
                "valid_status_values".to_string(),
            ],
            regulatory_requirements: vec![
                "fisma_compliance".to_string(),
                "nist_compliance".to_string(),
                "fedramp_baseline".to_string(),
            ],
            min_compliance_rate: quality_config.min_compliance_score,
            severity_threshold: QualitySeverity::Medium,
        }
    }
}

/// Compliance check type enumeration
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum ComplianceCheckType {
    /// FedRAMP required fields check
    FedRampRequiredFields,
    /// FedRAMP quality standards check
    FedRampQualityStandards,
    /// FedRAMP timeline requirements check
    FedRampTimelineRequirements,
    /// FedRAMP risk assessment requirements check
    FedRampRiskRequirements,
    /// OSCAL UUID format check
    OscalUuidFormat,
    /// OSCAL date format check
    OscalDateFormat,
    /// OSCAL metadata requirements check
    OscalMetadataRequirements,
    /// Regulatory FISMA compliance check
    RegulatoryFismaCompliance,
    /// Regulatory NIST compliance check
    RegulatoryNistCompliance,
    /// Custom compliance check
    Custom(String),
}

impl ComplianceCheckType {
    /// Get the standard name for this check type
    pub fn standard(&self) -> &str {
        match self {
            Self::FedRampRequiredFields
            | Self::FedRampQualityStandards
            | Self::FedRampTimelineRequirements
            | Self::FedRampRiskRequirements => "FedRAMP",
            Self::OscalUuidFormat
            | Self::OscalDateFormat
            | Self::OscalMetadataRequirements => "OSCAL",
            Self::RegulatoryFismaCompliance
            | Self::RegulatoryNistCompliance => "Regulatory",
            Self::Custom(_) => "Custom",
        }
    }

    /// Get the check name for this check type
    pub fn check_name(&self) -> String {
        match self {
            Self::FedRampRequiredFields => "fedramp_required_fields".to_string(),
            Self::FedRampQualityStandards => "fedramp_quality_standards".to_string(),
            Self::FedRampTimelineRequirements => "fedramp_timeline_requirements".to_string(),
            Self::FedRampRiskRequirements => "fedramp_risk_requirements".to_string(),
            Self::OscalUuidFormat => "oscal_uuid_format".to_string(),
            Self::OscalDateFormat => "oscal_date_format".to_string(),
            Self::OscalMetadataRequirements => "oscal_metadata_requirements".to_string(),
            Self::RegulatoryFismaCompliance => "regulatory_fisma_compliance".to_string(),
            Self::RegulatoryNistCompliance => "regulatory_nist_compliance".to_string(),
            Self::Custom(name) => name.clone(),
        }
    }

    /// Get the default severity for this check type
    pub fn default_severity(&self) -> QualitySeverity {
        match self {
            Self::FedRampRequiredFields
            | Self::FedRampTimelineRequirements
            | Self::FedRampRiskRequirements
            | Self::OscalUuidFormat => QualitySeverity::High,
            Self::FedRampQualityStandards
            | Self::OscalDateFormat
            | Self::OscalMetadataRequirements => QualitySeverity::Medium,
            Self::RegulatoryFismaCompliance
            | Self::RegulatoryNistCompliance => QualitySeverity::High,
            Self::Custom(_) => QualitySeverity::Medium,
        }
    }
}

/// Compliance assessment statistics
#[derive(Debug, Clone)]
pub struct ComplianceStatistics {
    /// Total number of items assessed
    pub total_items: usize,
    /// Total number of checks performed
    pub total_checks: usize,
    /// Number of checks that passed
    pub passed_checks: usize,
    /// Number of checks that failed
    pub failed_checks: usize,
    /// Overall compliance rate across all checks
    pub overall_compliance_rate: f64,
    /// Compliance rates by standard
    pub compliance_by_standard: HashMap<String, f64>,
    /// Number of violations by severity
    pub violations_by_severity: HashMap<QualitySeverity, usize>,
    /// Assessment duration in milliseconds
    pub assessment_duration_ms: u64,
}

impl ComplianceStatistics {
    /// Create new compliance statistics
    pub fn new() -> Self {
        Self {
            total_items: 0,
            total_checks: 0,
            passed_checks: 0,
            failed_checks: 0,
            overall_compliance_rate: 0.0,
            compliance_by_standard: HashMap::new(),
            violations_by_severity: HashMap::new(),
            assessment_duration_ms: 0,
        }
    }

    /// Calculate statistics from compliance results
    pub fn from_results(results: &[ComplianceCheckResult], duration_ms: u64) -> Self {
        let mut stats = Self::new();
        stats.assessment_duration_ms = duration_ms;
        stats.total_checks = results.len();

        let mut total_compliance = 0.0;
        let mut standard_totals: HashMap<String, (f64, usize)> = HashMap::new();

        for result in results {
            if result.compliance_rate >= 0.9 {
                stats.passed_checks += 1;
            } else {
                stats.failed_checks += 1;
            }

            total_compliance += result.compliance_rate;

            // Track by standard
            let entry = standard_totals.entry(result.standard.clone()).or_insert((0.0, 0));
            entry.0 += result.compliance_rate;
            entry.1 += 1;

            // Track violations by severity
            if !result.failed_details.is_empty() {
                *stats.violations_by_severity.entry(result.severity.clone()).or_insert(0) += result.failed_details.len();
            }
        }

        // Calculate overall compliance rate
        if !results.is_empty() {
            stats.overall_compliance_rate = total_compliance / results.len() as f64;
        }

        // Calculate compliance by standard
        for (standard, (total, count)) in standard_totals {
            stats.compliance_by_standard.insert(standard, total / count as f64);
        }

        stats
    }

    /// Check if compliance meets minimum threshold
    pub fn meets_threshold(&self, threshold: f64) -> bool {
        self.overall_compliance_rate >= threshold
    }

    /// Get the most common violation severity
    pub fn most_common_violation_severity(&self) -> Option<QualitySeverity> {
        self.violations_by_severity
            .iter()
            .max_by_key(|(_, count)| *count)
            .map(|(severity, _)| severity.clone())
    }
}

impl Default for ComplianceStatistics {
    fn default() -> Self {
        Self::new()
    }
}
