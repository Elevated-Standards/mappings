//! Report generation type definitions
//! Modified: 2025-01-22

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::Duration;
use uuid::Uuid;
use chrono::{DateTime, Utc};

use crate::validation::types::*;
use super::metrics::{ProcessingMetrics, TrendAnalysis};

/// Configuration for report generation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReportConfig {
    /// Default report format
    pub default_format: ReportFormat,
    /// Include data visualizations
    pub include_visualizations: bool,
    /// Maximum report generation time (seconds)
    pub max_generation_time_seconds: u64,
    /// Enable report caching
    pub enable_caching: bool,
    /// Cache expiration time (minutes)
    pub cache_expiration_minutes: u64,
    /// Include detailed field analysis
    pub include_detailed_analysis: bool,
    /// Include recommendations
    pub include_recommendations: bool,
    /// Report template directory
    pub template_directory: Option<String>,
}

/// Comprehensive mapping validation report
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MappingReport {
    /// Unique report identifier
    pub report_id: Uuid,
    /// Type of report generated
    pub report_type: ReportType,
    /// Report output format
    pub format: ReportFormat,
    /// When the report was generated
    pub generated_at: DateTime<Utc>,
    /// Information about the processed document
    pub document_info: DocumentInfo,
    /// High-level mapping summary
    pub mapping_summary: MappingSummary,
    /// Detailed field mapping results
    pub detailed_results: Vec<FieldMappingResult>,
    /// Data quality assessment
    pub quality_metrics: QualityMetrics,
    /// Validation summary
    pub validation_summary: ValidationSummary,
    /// Override usage summary
    pub override_summary: OverrideSummary,
    /// Processing performance metrics
    pub processing_metrics: ProcessingMetrics,
    /// Historical trend analysis (if available)
    pub trend_analysis: Option<TrendAnalysis>,
    /// Actionable recommendations
    pub recommendations: Vec<Recommendation>,
}

/// Type of report to generate
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum ReportType {
    /// Summary report with key metrics
    Summary,
    /// Detailed analysis report
    Detailed,
    /// Performance-focused report
    Performance,
    /// Quality assessment report
    Quality,
    /// Trend analysis report
    Trends,
    /// Executive summary
    Executive,
}

/// Output format for reports
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ReportFormat {
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
}

/// Information about the processed document
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentInfo {
    /// Document file name
    pub file_name: String,
    /// Document type (inventory, poam, ssp)
    pub document_type: String,
    /// File size in bytes
    pub file_size: u64,
    /// Number of data rows processed
    pub row_count: usize,
    /// Number of columns detected
    pub column_count: usize,
    /// When the document was processed
    pub processed_at: DateTime<Utc>,
    /// Total processing duration
    pub processing_duration: Duration,
    /// File content hash for integrity
    pub file_hash: String,
    /// File encoding detected
    pub encoding: String,
}

/// High-level mapping summary statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MappingSummary {
    /// Total number of fields expected
    pub total_fields: usize,
    /// Number of fields successfully mapped
    pub mapped_fields: usize,
    /// Number of high-confidence mappings
    pub high_confidence_mappings: usize,
    /// Number of mappings requiring review
    pub review_required: usize,
    /// Number of missing required fields
    pub missing_required: usize,
    /// Overall mapping success rate (0.0-1.0)
    pub success_rate: f64,
    /// Average confidence score across all mappings
    pub average_confidence: f64,
    /// Number of override rules applied
    pub overrides_applied: usize,
    /// Number of conflicts resolved
    pub conflicts_resolved: usize,
    /// Data completeness percentage
    pub completeness_percentage: f64,
    /// Overall quality score
    pub quality_score: f64,
}

/// Detailed result for a single field mapping
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FieldMappingResult {
    /// Expected field identifier
    pub field_id: String,
    /// Target OSCAL field path
    pub target_field: String,
    /// Source column that was mapped (if any)
    pub source_column: Option<String>,
    /// Confidence score for this mapping
    pub confidence_score: f64,
    /// Whether mapping was successful
    pub mapping_successful: bool,
    /// Whether this field is required
    pub required: bool,
    /// Whether an override rule was applied
    pub override_applied: bool,
    /// Alternative mapping suggestions
    pub alternatives: Vec<MappingAlternative>,
    /// Issues identified with this mapping
    pub issues: Vec<MappingIssue>,
    /// Sample data values for verification
    pub data_samples: Vec<String>,
}

/// Alternative mapping suggestion
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MappingAlternative {
    /// Alternative source column
    pub source_column: String,
    /// Confidence score for alternative
    pub confidence_score: f64,
    /// Reason for suggesting this alternative
    pub reason: String,
}

/// Issue identified during mapping
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MappingIssue {
    /// Issue severity level
    pub severity: IssueSeverity,
    /// Issue category
    pub category: IssueCategory,
    /// Human-readable issue description
    pub description: String,
    /// Suggested resolution
    pub suggested_resolution: Option<String>,
    /// Related field or column
    pub related_field: Option<String>,
}

/// Comprehensive data quality assessment
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataQualityAssessment {
    /// Completeness score (0.0-1.0)
    pub completeness: f64,
    /// Consistency score (0.0-1.0)
    pub consistency: f64,
    /// Accuracy score (0.0-1.0)
    pub accuracy: f64,
    /// Validity score (0.0-1.0)
    pub validity: f64,
    /// Overall quality score (0.0-1.0)
    pub overall_score: f64,
    /// Quality grade (A-F)
    pub quality_grade: QualityGrade,
    /// Risk level assessment
    pub risk_level: RiskLevel,
}

/// Detailed quality metrics breakdown
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QualityMetrics {
    /// Data completeness score (0.0-1.0)
    pub completeness_score: f64,
    /// Mapping accuracy score (0.0-1.0)
    pub mapping_accuracy: f64,
    /// Format compliance score (0.0-1.0)
    pub format_compliance: f64,
    /// Business rule compliance score (0.0-1.0)
    pub business_rule_compliance: f64,
    /// Cross-field consistency score (0.0-1.0)
    pub cross_field_consistency: f64,
    /// Data freshness score (0.0-1.0)
    pub data_freshness: f64,
    /// Duplicate detection score (0.0-1.0)
    pub duplicate_score: f64,
    /// Outlier detection score (0.0-1.0)
    pub outlier_score: f64,
    /// Reference data validation score (0.0-1.0)
    pub reference_validation: f64,
    /// Overall quality score (0.0-1.0)
    pub overall_quality_score: f64,
    /// Overall quality grade
    pub quality_grade: QualityGrade,
}

/// Actionable recommendation for improvement
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Recommendation {
    /// Recommendation priority
    pub priority: RecommendationPriority,
    /// Recommendation category
    pub category: RecommendationCategory,
    /// Short recommendation title
    pub title: String,
    /// Detailed description
    pub description: String,
    /// Suggested action to take
    pub suggested_action: String,
    /// Estimated effort level
    pub effort_level: EffortLevel,
    /// Expected impact of implementing
    pub expected_impact: String,
    /// Step-by-step implementation guide
    pub implementation_steps: Vec<String>,
    /// Fields related to this recommendation
    pub related_fields: Vec<String>,
}

/// Validation summary statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationSummary {
    /// Total validations performed
    pub total_validations: usize,
    /// Number of validations passed
    pub passed_validations: usize,
    /// Number of validations failed
    pub failed_validations: usize,
    /// Number of warnings generated
    pub warning_count: usize,
    /// Number of errors encountered
    pub error_count: usize,
    /// Overall validation score (0.0-1.0)
    pub overall_score: f64,
    /// Most common validation failures
    pub common_failures: Vec<ValidationFailureInfo>,
    /// Performance metrics for validation
    pub performance_metrics: ValidationPerformanceMetrics,
}

/// Information about validation failures
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationFailureInfo {
    /// Type of validation failure
    pub failure_type: String,
    /// Number of occurrences
    pub occurrence_count: usize,
    /// Percentage of total failures
    pub percentage: f64,
    /// Example fields that failed
    pub example_fields: Vec<String>,
    /// Suggested remediation
    pub suggested_remediation: String,
}

/// Performance metrics specific to validation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationPerformanceMetrics {
    /// Average validation time per field (microseconds)
    pub avg_validation_time_us: f64,
    /// Total validation time
    pub total_validation_time: Duration,
    /// Slowest validation operations
    pub slow_validations: Vec<SlowValidationInfo>,
}

/// Information about slow validation operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SlowValidationInfo {
    /// Field that was slow to validate
    pub field_id: String,
    /// Validation time in microseconds
    pub validation_time_us: f64,
    /// Reason for slow validation
    pub reason: String,
}

/// Summary of override rule usage
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OverrideSummary {
    /// Total number of override rules evaluated
    pub total_overrides_evaluated: usize,
    /// Number of overrides applied
    pub overrides_applied: usize,
    /// Number of conflicts detected
    pub conflicts_detected: usize,
    /// Number of conflicts resolved
    pub conflicts_resolved: usize,
    /// Override usage details
    pub usage_details: Vec<OverrideUsageInfo>,
    /// Performance metrics for override processing
    pub performance_metrics: OverridePerformanceMetrics,
}

/// Information about specific override usage
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OverrideUsageInfo {
    /// Override rule name
    pub override_name: String,
    /// Number of times applied
    pub application_count: usize,
    /// Success rate for this override
    pub success_rate: f64,
    /// Average confidence improvement
    pub avg_confidence_improvement: f64,
}

/// Performance metrics for override processing
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OverridePerformanceMetrics {
    /// Average override resolution time (microseconds)
    pub avg_resolution_time_us: f64,
    /// Cache hit rate for override lookups
    pub cache_hit_rate: f64,
    /// Total override processing time
    pub total_processing_time: Duration,
}

impl Default for ReportConfig {
    fn default() -> Self {
        Self {
            default_format: ReportFormat::Html,
            include_visualizations: true,
            max_generation_time_seconds: 30,
            enable_caching: true,
            cache_expiration_minutes: 60,
            include_detailed_analysis: true,
            include_recommendations: true,
            template_directory: None,
        }
    }
}
