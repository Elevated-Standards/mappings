//! Report generation type definitions
//! Modified: 2025-01-22

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::Duration;
use uuid::Uuid;
use chrono::{DateTime, Utc};

use crate::validation_backup::types::*;
use crate::validation_backup::confidence::*;
use crate::validation_backup::overrides::*;

/// Configuration for report generation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReportConfig {
    /// Default report format
    pub default_format: ReportFormat,
    /// Include data visualizations
    pub include_visualizations: bool,
    /// Include detailed explanations
    pub include_explanations: bool,
    /// Include performance metrics
    pub include_performance_metrics: bool,
    /// Include historical trends
    pub include_trends: bool,
    /// Maximum report size in MB
    pub max_report_size_mb: usize,
    /// Report retention period in days
    pub retention_days: u32,
    /// Enable report caching
    pub enable_caching: bool,
}

/// Comprehensive mapping analysis report
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
    pub field_mappings: Vec<FieldMappingResult>,
    /// Data quality assessment
    pub quality_assessment: DataQualityAssessment,
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
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
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
    /// Technical deep-dive
    Technical,
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
    /// Excel format for spreadsheet analysis
    Excel,
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
    pub avg_confidence_score: f64,
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
    pub oscal_field: String,
    /// Source column that was mapped (if any)
    pub source_column: Option<String>,
    /// Confidence score for this mapping
    pub confidence_score: f64,
    /// Mapping status
    pub status: FieldValidationStatus,
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

/// Severity level for mapping issues
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, PartialOrd)]
pub enum IssueSeverity {
    /// Informational message
    Info,
    /// Warning that should be reviewed
    Warning,
    /// Error that must be addressed
    Error,
    /// Critical issue requiring immediate attention
    Critical,
}

/// Category of mapping issue
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum IssueCategory {
    /// Missing required field
    MissingRequired,
    /// Low confidence mapping
    LowConfidence,
    /// Data format mismatch
    FormatMismatch,
    /// Business rule violation
    BusinessRuleViolation,
    /// Duplicate mapping detected
    DuplicateMapping,
    /// Inconsistent data values
    InconsistentData,
    /// Performance concern
    Performance,
    /// Configuration issue
    Configuration,
    /// Data quality concern
    DataQuality,
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
    /// Detailed quality metrics
    pub detailed_metrics: QualityMetrics,
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
}

/// Risk level assessment
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, PartialOrd)]
pub enum RiskLevel {
    /// Very low risk
    VeryLow,
    /// Low risk
    Low,
    /// Medium risk
    Medium,
    /// High risk
    High,
    /// Very high risk
    VeryHigh,
}

/// Quality grade assessment
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum QualityGrade {
    /// Excellent quality (90-100%)
    A,
    /// Good quality (80-89%)
    B,
    /// Acceptable quality (70-79%)
    C,
    /// Poor quality (60-69%)
    D,
    /// Failing quality (<60%)
    F,
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
    /// Estimated effort level
    pub effort_level: EffortLevel,
    /// Expected impact of implementing
    pub expected_impact: String,
    /// Step-by-step implementation guide
    pub implementation_steps: Vec<String>,
    /// Fields related to this recommendation
    pub related_fields: Vec<String>,
}

/// Priority level for recommendations
pub use crate::validation_backup::types::RecommendationPriority;

/// Category of recommendation
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum RecommendationCategory {
    /// Data quality improvement
    DataQuality,
    /// Mapping accuracy improvement
    MappingAccuracy,
    /// Performance optimization
    Performance,
    /// Configuration adjustment
    Configuration,
    /// Process improvement
    Process,
    /// Training or documentation
    Training,
}

/// Effort level required for implementation
pub use crate::validation_backup::types::EffortLevel;

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
            include_explanations: true,
            include_performance_metrics: true,
            include_trends: false,
            max_report_size_mb: 50,
            retention_days: 30,
            enable_caching: true,
        }
    }
}
