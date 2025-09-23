//! Modified: 2025-01-22

//! Compliance Assessor
//! 
//! Main compliance assessment coordinator that orchestrates FedRAMP, OSCAL, and regulatory compliance checks

use super::types::*;
use super::fedramp_compliance::FedRampComplianceChecker;
use super::oscal_compliance::OscalComplianceChecker;
use super::regulatory_compliance::RegulatoryComplianceChecker;
use super::super::*;
use crate::poam::PoamItem;
use fedramp_core::Result;
use tracing::{debug, info};
use uuid::Uuid;
use std::collections::HashMap;
use std::time::Instant;

/// Compliance assessor for POA&M data
#[derive(Debug, Clone)]
pub struct ComplianceAssessor {
    /// Compliance configuration
    config: ComplianceConfig,
    /// FedRAMP compliance checker
    fedramp_checker: FedRampComplianceChecker,
    /// OSCAL compliance checker
    oscal_checker: OscalComplianceChecker,
    /// Regulatory compliance checker
    regulatory_checker: RegulatoryComplianceChecker,
}

impl ComplianceAssessor {
    /// Create a new compliance assessor with default configuration
    pub fn new() -> Self {
        let config = ComplianceConfig::default();
        Self {
            fedramp_checker: FedRampComplianceChecker::with_config(&config),
            oscal_checker: OscalComplianceChecker::with_config(&config),
            regulatory_checker: RegulatoryComplianceChecker::with_config(&config),
            config,
        }
    }

    /// Create a compliance assessor with custom configuration
    pub fn with_config(config: ComplianceConfig) -> Self {
        Self {
            fedramp_checker: FedRampComplianceChecker::with_config(&config),
            oscal_checker: OscalComplianceChecker::with_config(&config),
            regulatory_checker: RegulatoryComplianceChecker::with_config(&config),
            config,
        }
    }

    /// Assess compliance of POA&M items
    pub fn assess(&self, poam_items: &[PoamItem]) -> Result<ComplianceResult> {
        let start_time = Instant::now();
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
        let fedramp_results = if self.config.enable_fedramp_checks {
            self.fedramp_checker.assess_compliance(poam_items)?
        } else {
            Vec::new()
        };

        // Perform OSCAL compliance checks
        let oscal_results = if self.config.enable_oscal_checks {
            self.oscal_checker.assess_compliance(poam_items)?
        } else {
            Vec::new()
        };

        // Perform regulatory compliance checks
        let regulatory_results = if self.config.enable_regulatory_checks {
            self.regulatory_checker.assess_compliance(poam_items)?
        } else {
            Vec::new()
        };

        // Generate compliance findings
        let findings = self.generate_compliance_findings(&fedramp_results, &oscal_results, &regulatory_results)?;

        // Calculate overall compliance score
        let overall_score = self.calculate_compliance_score(&fedramp_results, &oscal_results, &regulatory_results);

        let duration = start_time.elapsed();
        debug!(
            "Compliance assessment completed: Score: {:.2}, Findings: {}, Duration: {:?}",
            overall_score,
            findings.len(),
            duration
        );

        Ok(ComplianceResult {
            score: overall_score,
            findings,
            fedramp_results,
            oscal_results,
            regulatory_results,
        })
    }

    /// Assess compliance with detailed statistics
    pub fn assess_with_statistics(&self, poam_items: &[PoamItem]) -> Result<(ComplianceResult, ComplianceStatistics)> {
        let start_time = Instant::now();
        let result = self.assess(poam_items)?;
        let duration = start_time.elapsed();

        // Collect all results for statistics
        let all_results: Vec<&ComplianceCheckResult> = result.fedramp_results.iter()
            .chain(result.oscal_results.iter())
            .chain(result.regulatory_results.iter())
            .collect();

        let statistics = ComplianceStatistics::from_results(&all_results.into_iter().cloned().collect::<Vec<_>>(), duration.as_millis() as u64);

        Ok((result, statistics))
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
            if result.compliance_rate < self.config.min_compliance_rate && result.failed_items > 0 {
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

    /// Get compliance summary by standard
    pub fn get_compliance_summary(&self, result: &ComplianceResult) -> HashMap<String, f64> {
        let mut summary = HashMap::new();

        // FedRAMP compliance
        if !result.fedramp_results.is_empty() {
            let fedramp_score = result.fedramp_results.iter()
                .map(|r| r.compliance_rate)
                .sum::<f64>() / result.fedramp_results.len() as f64;
            summary.insert("FedRAMP".to_string(), fedramp_score);
        }

        // OSCAL compliance
        if !result.oscal_results.is_empty() {
            let oscal_score = result.oscal_results.iter()
                .map(|r| r.compliance_rate)
                .sum::<f64>() / result.oscal_results.len() as f64;
            summary.insert("OSCAL".to_string(), oscal_score);
        }

        // Regulatory compliance
        if !result.regulatory_results.is_empty() {
            let regulatory_score = result.regulatory_results.iter()
                .map(|r| r.compliance_rate)
                .sum::<f64>() / result.regulatory_results.len() as f64;
            summary.insert("Regulatory".to_string(), regulatory_score);
        }

        summary
    }

    /// Check if overall compliance meets threshold
    pub fn meets_compliance_threshold(&self, result: &ComplianceResult) -> bool {
        result.score >= self.config.min_compliance_rate
    }

    /// Get failed compliance checks
    pub fn get_failed_checks<'a>(&self, result: &'a ComplianceResult) -> Vec<&'a ComplianceCheckResult> {
        result.fedramp_results.iter()
            .chain(result.oscal_results.iter())
            .chain(result.regulatory_results.iter())
            .filter(|r| r.compliance_rate < self.config.min_compliance_rate)
            .collect()
    }

    /// Get compliance violations by severity
    pub fn get_violations_by_severity(&self, result: &ComplianceResult) -> HashMap<QualitySeverity, usize> {
        let mut violations = HashMap::new();

        let all_results = result.fedramp_results.iter()
            .chain(result.oscal_results.iter())
            .chain(result.regulatory_results.iter());

        for check_result in all_results {
            if !check_result.failed_details.is_empty() {
                *violations.entry(check_result.severity.clone()).or_insert(0) += check_result.failed_details.len();
            }
        }

        violations
    }

    /// Generate compliance report summary
    pub fn generate_report_summary(&self, result: &ComplianceResult) -> String {
        let total_checks = result.fedramp_results.len() + result.oscal_results.len() + result.regulatory_results.len();
        let failed_checks = self.get_failed_checks(result).len();
        let passed_checks = total_checks - failed_checks;

        format!(
            "Compliance Assessment Summary:\n\
            Overall Score: {:.1}%\n\
            Checks Passed: {}/{}\n\
            Total Findings: {}\n\
            Meets Threshold: {}",
            result.score * 100.0,
            passed_checks,
            total_checks,
            result.findings.len(),
            self.meets_compliance_threshold(result)
        )
    }

    /// Update configuration
    pub fn update_config(&mut self, config: ComplianceConfig) {
        self.config = config.clone();
        self.fedramp_checker = FedRampComplianceChecker::with_config(&config);
        self.oscal_checker = OscalComplianceChecker::with_config(&config);
        self.regulatory_checker = RegulatoryComplianceChecker::with_config(&config);
    }

    /// Get current configuration
    pub fn get_config(&self) -> &ComplianceConfig {
        &self.config
    }
}

impl Default for ComplianceAssessor {
    fn default() -> Self {
        Self::new()
    }
}
