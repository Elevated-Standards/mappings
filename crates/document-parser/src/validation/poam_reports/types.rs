// Modified: 2025-09-23

//! POA&M validation report types and data structures
//!
//! This module defines the core data structures for POA&M validation reports,
//! including report types, processing summaries, validation results, and metrics.

use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use uuid::Uuid;
use std::collections::HashMap;
use std::time::Duration;

use crate::validation::poam_validator::types::{
    PoamValidationResult, ValidationError, ValidationWarning, ValidationSuggestion
};
use crate::quality::{QualityAssessment, QualityMetrics};

/// Type of POA&M validation report
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum PoamReportType {
    /// Processing summary report with key metrics
    ProcessingSummary,
    /// Detailed validation report with item-by-item results
    DetailedValidation,
    /// Compliance assessment report
    ComplianceAssessment,
    /// Quality trend report with historical analysis
    QualityTrend,
    /// Executive summary for stakeholders
    ExecutiveSummary,
    /// Performance analysis report
    Performance,
}

/// Output format for POA&M reports
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum PoamReportFormat {
    /// HTML format with interactive elements
    Html,
    /// PDF format for printing/sharing
    Pdf,
    /// JSON format for API consumption
    Json,
    /// CSV format for data analysis
    Csv,
    /// Markdown format for documentation
    Markdown,
    /// Excel format for detailed analysis
    Excel,
}

/// Complete POA&M validation report
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PoamValidationReport {
    /// Unique report identifier
    pub report_id: Uuid,
    /// Type of report
    pub report_type: PoamReportType,
    /// Output format
    pub format: PoamReportFormat,
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
    pub recommendations: Vec<Recommendation>,
    /// Report metadata and generation info
    pub metadata: ReportMetadata,
}

/// Information about the processed document
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentInfo {
    /// Source document path or identifier
    pub source_path: String,
    /// Document name or title
    pub document_name: String,
    /// Document size in bytes
    pub document_size: u64,
    /// Document format (Excel, CSV, etc.)
    pub document_format: String,
    /// Processing timestamp
    pub processed_at: DateTime<Utc>,
    /// Document checksum for integrity
    pub checksum: Option<String>,
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
    /// Overall quality score (0.0-1.0)
    pub quality_score: f64,
    /// Overall compliance score (0.0-1.0)
    pub compliance_score: f64,
    /// Processing success rate
    pub success_rate: f64,
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
    /// Cross-field validation results
    pub cross_field_validation: CrossFieldValidationResult,
}

/// Schema validation results
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SchemaValidationResult {
    /// Whether schema validation passed
    pub is_valid: bool,
    /// Schema validation errors
    pub errors: Vec<ValidationError>,
    /// Schema validation warnings
    pub warnings: Vec<ValidationWarning>,
    /// Schema compliance score
    pub compliance_score: f64,
}

/// Business rule validation results
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BusinessRuleValidationResult {
    /// Number of business rules evaluated
    pub rules_evaluated: usize,
    /// Number of rules that passed
    pub rules_passed: usize,
    /// Number of rules that failed
    pub rules_failed: usize,
    /// Business rule compliance score
    pub compliance_score: f64,
    /// Rule-specific results
    pub rule_results: Vec<RuleResult>,
}

/// Individual business rule result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuleResult {
    /// Rule identifier
    pub rule_id: String,
    /// Rule name or description
    pub rule_name: String,
    /// Whether the rule passed
    pub passed: bool,
    /// Items that failed this rule
    pub failed_items: Vec<String>,
    /// Rule severity level
    pub severity: String,
}

/// Data quality validation results
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataQualityValidationResult {
    /// Overall data quality score
    pub quality_score: f64,
    /// Accuracy assessment results
    pub accuracy_results: AccuracyResults,
    /// Consistency assessment results
    pub consistency_results: ConsistencyResults,
    /// Completeness assessment results
    pub completeness_results: CompletenessResults,
}

/// Accuracy assessment results
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccuracyResults {
    /// Accuracy score (0.0-1.0)
    pub score: f64,
    /// Number of accuracy issues found
    pub issues_found: usize,
    /// Field-specific accuracy scores
    pub field_scores: HashMap<String, f64>,
}

/// Consistency assessment results
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConsistencyResults {
    /// Consistency score (0.0-1.0)
    pub score: f64,
    /// Number of consistency issues found
    pub issues_found: usize,
    /// Cross-field consistency results
    pub cross_field_results: HashMap<String, f64>,
}

/// Completeness assessment results
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompletenessResults {
    /// Completeness score (0.0-1.0)
    pub score: f64,
    /// Number of incomplete items
    pub incomplete_items: usize,
    /// Field-specific completeness scores
    pub field_completeness: HashMap<String, f64>,
}

/// Completeness validation results
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompletenessValidationResult {
    /// Completeness score (0.0-1.0)
    pub score: f64,
    /// Number of incomplete items
    pub incomplete_items: usize,
    /// Field-specific completeness scores
    pub field_completeness: HashMap<String, f64>,
}

/// Cross-field validation results
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CrossFieldValidationResult {
    /// Number of cross-field rules evaluated
    pub rules_evaluated: usize,
    /// Number of cross-field validation errors
    pub errors_found: usize,
    /// Cross-field consistency score
    pub consistency_score: f64,
    /// Specific cross-field issues
    pub issues: Vec<CrossFieldIssue>,
}

/// Cross-field validation issue
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CrossFieldIssue {
    /// Issue identifier
    pub issue_id: String,
    /// Fields involved in the issue
    pub fields: Vec<String>,
    /// Issue description
    pub description: String,
    /// Affected POA&M items
    pub affected_items: Vec<String>,
    /// Issue severity
    pub severity: String,
}

/// Compliance status information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComplianceStatus {
    /// Overall compliance score (0.0-1.0)
    pub overall_score: f64,
    /// FedRAMP requirement compliance
    pub fedramp_compliance: ComplianceCategory,
    /// OSCAL schema compliance
    pub oscal_compliance: ComplianceCategory,
    /// Regulatory compliance indicators
    pub regulatory_compliance: Vec<RegulatoryCompliance>,
    /// Risk assessment results
    pub risk_assessment: RiskAssessment,
}

/// Compliance category results
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComplianceCategory {
    /// Compliance score for this category
    pub score: f64,
    /// Number of requirements met
    pub requirements_met: usize,
    /// Total number of requirements
    pub total_requirements: usize,
    /// Specific compliance issues
    pub issues: Vec<ComplianceIssue>,
}

/// Individual compliance issue
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComplianceIssue {
    /// Issue identifier
    pub issue_id: String,
    /// Requirement that failed
    pub requirement: String,
    /// Issue description
    pub description: String,
    /// Severity level
    pub severity: String,
    /// Recommended remediation
    pub remediation: Option<String>,
}

/// Regulatory compliance information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegulatoryCompliance {
    /// Regulation name (e.g., "FISMA", "NIST SP 800-53")
    pub regulation: String,
    /// Compliance score for this regulation
    pub score: f64,
    /// Compliance status
    pub status: String,
    /// Specific requirements
    pub requirements: Vec<String>,
}

/// Risk assessment results
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RiskAssessment {
    /// Overall risk score
    pub overall_risk_score: f64,
    /// Risk level (Low, Moderate, High)
    pub risk_level: String,
    /// Identified risks
    pub identified_risks: Vec<IdentifiedRisk>,
    /// Risk mitigation recommendations
    pub mitigation_recommendations: Vec<String>,
}

/// Individual identified risk
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IdentifiedRisk {
    /// Risk identifier
    pub risk_id: String,
    /// Risk description
    pub description: String,
    /// Risk severity
    pub severity: String,
    /// Likelihood of occurrence
    pub likelihood: String,
    /// Potential impact
    pub impact: String,
    /// Affected POA&M items
    pub affected_items: Vec<String>,
}

/// Actionable recommendation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Recommendation {
    /// Recommendation identifier
    pub id: String,
    /// Recommendation category
    pub category: RecommendationCategory,
    /// Priority level
    pub priority: RecommendationPriority,
    /// Recommendation title
    pub title: String,
    /// Detailed description
    pub description: String,
    /// Specific actions to take
    pub actions: Vec<String>,
    /// Expected impact of implementing
    pub expected_impact: String,
    /// Estimated effort required
    pub effort_estimate: Option<String>,
}

/// Recommendation category
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum RecommendationCategory {
    DataQuality,
    Compliance,
    Performance,
    Security,
    Process,
    Documentation,
}

/// Recommendation priority
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum RecommendationPriority {
    Critical,
    High,
    Medium,
    Low,
}

/// Report metadata and generation information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReportMetadata {
    /// Tool version that generated the report
    pub tool_version: String,
    /// Report generation time
    pub generation_time: Duration,
    /// Configuration used for processing
    pub processing_config: ProcessingConfig,
    /// Report template version
    pub template_version: String,
    /// Additional metadata
    pub additional_metadata: HashMap<String, String>,
}

/// Processing configuration summary
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessingConfig {
    /// Validation mode used
    pub validation_mode: String,
    /// Quality thresholds applied
    pub quality_thresholds: HashMap<String, f64>,
    /// Business rules enabled
    pub business_rules_enabled: Vec<String>,
    /// Custom configurations
    pub custom_config: HashMap<String, String>,
}
