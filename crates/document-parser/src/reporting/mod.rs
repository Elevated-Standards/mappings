//! POA&M Validation Reporting Framework
//! 
//! This module provides comprehensive reporting capabilities for POA&M validation results,
//! including multiple report formats, data visualization, and automated distribution.

pub mod generator;
pub mod templates;
pub mod formats;
pub mod visualization;
pub mod distribution;

pub use generator::*;
pub use templates::*;
pub use formats::*;
pub use visualization::*;
pub use distribution::*;

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use chrono::{DateTime, Utc, Duration};
use uuid::Uuid;
use crate::quality::{QualityAssessment, QualityFinding, QualityRecommendation};

/// Types of reports that can be generated
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ReportType {
    /// High-level processing summary
    ProcessingSummary,
    /// Detailed validation results
    DetailedValidation,
    /// Compliance assessment report
    ComplianceAssessment,
    /// Quality trend analysis
    QualityTrend,
    /// Executive summary
    ExecutiveSummary,
    /// Custom report type
    Custom(String),
}

/// Report output formats
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ReportFormat {
    /// HTML format with interactive elements
    Html,
    /// PDF format for printing and archival
    Pdf,
    /// JSON format for API consumption
    Json,
    /// CSV format for data analysis
    Csv,
    /// Markdown format for documentation
    Markdown,
}

/// Report delivery methods
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum DeliveryMethod {
    /// Save to file system
    File(String),
    /// Send via email
    Email(Vec<String>),
    /// Upload to web service
    WebService(String),
    /// Store in database
    Database,
    /// Return in memory
    InMemory,
}

/// Complete POA&M validation report
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PoamValidationReport {
    /// Unique report identifier
    pub report_id: String,
    /// Type of report
    pub report_type: ReportType,
    /// When the report was generated
    pub generated_at: DateTime<Utc>,
    /// Information about the processed document
    pub document_info: DocumentInfo,
    /// High-level processing summary
    pub processing_summary: ProcessingSummary,
    /// Detailed validation results
    pub validation_results: ValidationResults,
    /// Quality assessment results
    pub quality_assessment: QualityAssessment,
    /// Compliance status information
    pub compliance_status: ComplianceStatus,
    /// Actionable recommendations
    pub recommendations: Vec<ReportRecommendation>,
    /// Report metadata
    pub metadata: ReportMetadata,
}

/// Information about the processed document
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentInfo {
    /// Original document filename
    pub filename: String,
    /// Document size in bytes
    pub file_size: u64,
    /// Document type (Excel, Word, etc.)
    pub document_type: String,
    /// Processing timestamp
    pub processed_at: DateTime<Utc>,
    /// Document version or revision
    pub version: Option<String>,
    /// Source system or organization
    pub source: Option<String>,
}

/// High-level processing summary
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessingSummary {
    /// Total number of POA&M items processed
    pub total_items_processed: usize,
    /// Number of successfully processed items
    pub successful_items: usize,
    /// Number of items with errors
    pub items_with_errors: usize,
    /// Number of items with warnings
    pub items_with_warnings: usize,
    /// Total processing time
    pub processing_time: Duration,
    /// Overall quality score (0.0 to 1.0)
    pub quality_score: f64,
    /// Overall compliance score (0.0 to 1.0)
    pub compliance_score: f64,
    /// Processing success rate
    pub success_rate: f64,
    /// Data completeness rate
    pub completeness_rate: f64,
}

/// Detailed validation results
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationResults {
    /// Schema validation results
    pub schema_validation: SchemaValidationResult,
    /// Business rule validation results
    pub business_rule_validation: BusinessRuleValidationResult,
    /// Data quality validation results
    pub data_quality_validation: DataQualityValidationResult,
    /// Completeness validation results
    pub completeness_validation: CompletenessValidationResult,
}

/// Schema validation results
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SchemaValidationResult {
    /// Whether schema validation passed
    pub passed: bool,
    /// Number of schema violations
    pub violations: usize,
    /// Detailed violation descriptions
    pub violation_details: Vec<ValidationViolation>,
    /// Schema version used for validation
    pub schema_version: String,
}

/// Business rule validation results
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BusinessRuleValidationResult {
    /// Number of rules evaluated
    pub rules_evaluated: usize,
    /// Number of rules that passed
    pub rules_passed: usize,
    /// Number of rules that failed
    pub rules_failed: usize,
    /// Detailed rule results
    pub rule_results: Vec<RuleResult>,
}

/// Data quality validation results
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataQualityValidationResult {
    /// Overall data quality score
    pub quality_score: f64,
    /// Number of quality issues found
    pub issues_found: usize,
    /// Quality issues by severity
    pub issues_by_severity: HashMap<String, usize>,
    /// Detailed quality findings
    pub quality_findings: Vec<QualityFinding>,
}

/// Completeness validation results
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompletenessValidationResult {
    /// Overall completeness percentage
    pub completeness_percentage: f64,
    /// Number of required fields missing
    pub missing_required_fields: usize,
    /// Number of recommended fields missing
    pub missing_recommended_fields: usize,
    /// Field-level completeness statistics
    pub field_completeness: HashMap<String, f64>,
}

/// Compliance status information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComplianceStatus {
    /// Overall compliance score
    pub overall_score: f64,
    /// FedRAMP compliance status
    pub fedramp_compliance: ComplianceResult,
    /// OSCAL compliance status
    pub oscal_compliance: ComplianceResult,
    /// Regulatory compliance status
    pub regulatory_compliance: Vec<ComplianceResult>,
    /// Compliance gaps and issues
    pub compliance_gaps: Vec<ComplianceGap>,
}

/// Individual compliance result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComplianceResult {
    /// Compliance standard name
    pub standard: String,
    /// Compliance score (0.0 to 1.0)
    pub score: f64,
    /// Whether compliance requirements are met
    pub compliant: bool,
    /// Number of requirements checked
    pub requirements_checked: usize,
    /// Number of requirements passed
    pub requirements_passed: usize,
    /// Detailed compliance findings
    pub findings: Vec<String>,
}

/// Compliance gap information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComplianceGap {
    /// Gap identifier
    pub gap_id: String,
    /// Gap description
    pub description: String,
    /// Affected standard
    pub standard: String,
    /// Severity of the gap
    pub severity: String,
    /// Recommended remediation
    pub remediation: String,
    /// Estimated effort to resolve
    pub effort_estimate: Option<String>,
}

/// Report-specific recommendation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReportRecommendation {
    /// Recommendation identifier
    pub id: String,
    /// Priority level
    pub priority: String,
    /// Recommendation title
    pub title: String,
    /// Detailed description
    pub description: String,
    /// Expected impact
    pub expected_impact: String,
    /// Implementation steps
    pub implementation_steps: Vec<String>,
    /// Related quality findings
    pub related_findings: Vec<String>,
}

/// Report metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReportMetadata {
    /// Report generator version
    pub generator_version: String,
    /// Report template used
    pub template_name: String,
    /// Report configuration
    pub configuration: HashMap<String, serde_json::Value>,
    /// Generation statistics
    pub generation_stats: GenerationStats,
}

/// Report generation statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenerationStats {
    /// Time taken to generate report
    pub generation_time: Duration,
    /// Report size in bytes
    pub report_size: u64,
    /// Number of sections generated
    pub sections_generated: usize,
    /// Number of visualizations created
    pub visualizations_created: usize,
}

/// Validation violation details
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationViolation {
    /// Violation type
    pub violation_type: String,
    /// Field or element that violated the rule
    pub field: String,
    /// Violation description
    pub description: String,
    /// Severity level
    pub severity: String,
    /// Suggested fix
    pub suggested_fix: Option<String>,
}

/// Business rule result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuleResult {
    /// Rule identifier
    pub rule_id: String,
    /// Rule name
    pub rule_name: String,
    /// Whether the rule passed
    pub passed: bool,
    /// Rule description
    pub description: String,
    /// Items that failed the rule
    pub failed_items: Vec<String>,
    /// Failure reason
    pub failure_reason: Option<String>,
}

impl ReportType {
    /// Get the display name for the report type
    pub fn display_name(&self) -> &str {
        match self {
            ReportType::ProcessingSummary => "Processing Summary",
            ReportType::DetailedValidation => "Detailed Validation",
            ReportType::ComplianceAssessment => "Compliance Assessment",
            ReportType::QualityTrend => "Quality Trend Analysis",
            ReportType::ExecutiveSummary => "Executive Summary",
            ReportType::Custom(name) => name,
        }
    }

    /// Get the default filename for the report type
    pub fn default_filename(&self) -> String {
        let base = match self {
            ReportType::ProcessingSummary => "processing_summary",
            ReportType::DetailedValidation => "detailed_validation",
            ReportType::ComplianceAssessment => "compliance_assessment",
            ReportType::QualityTrend => "quality_trend",
            ReportType::ExecutiveSummary => "executive_summary",
            ReportType::Custom(name) => &name.to_lowercase().replace(' ', "_"),
        };
        format!("{}_{}", base, chrono::Utc::now().format("%Y%m%d_%H%M%S"))
    }
}

impl ReportFormat {
    /// Get the file extension for the format
    pub fn file_extension(&self) -> &str {
        match self {
            ReportFormat::Html => "html",
            ReportFormat::Pdf => "pdf",
            ReportFormat::Json => "json",
            ReportFormat::Csv => "csv",
            ReportFormat::Markdown => "md",
        }
    }

    /// Get the MIME type for the format
    pub fn mime_type(&self) -> &str {
        match self {
            ReportFormat::Html => "text/html",
            ReportFormat::Pdf => "application/pdf",
            ReportFormat::Json => "application/json",
            ReportFormat::Csv => "text/csv",
            ReportFormat::Markdown => "text/markdown",
        }
    }
}

impl ProcessingSummary {
    /// Calculate the success rate
    pub fn calculate_success_rate(&mut self) {
        if self.total_items_processed > 0 {
            self.success_rate = self.successful_items as f64 / self.total_items_processed as f64;
        } else {
            self.success_rate = 0.0;
        }
    }

    /// Calculate the completeness rate
    pub fn calculate_completeness_rate(&mut self, complete_items: usize) {
        if self.total_items_processed > 0 {
            self.completeness_rate = complete_items as f64 / self.total_items_processed as f64;
        } else {
            self.completeness_rate = 0.0;
        }
    }
}

impl ComplianceResult {
    /// Check if the compliance result meets the threshold
    pub fn meets_threshold(&self, threshold: f64) -> bool {
        self.score >= threshold
    }

    /// Get the compliance percentage
    pub fn compliance_percentage(&self) -> f64 {
        self.score * 100.0
    }
}
