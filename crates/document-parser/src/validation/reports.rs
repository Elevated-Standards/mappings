//! Report generation system for mapping validation

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::{Duration, Instant};
use uuid::Uuid;
use chrono::{DateTime, Utc};
use lru::LruCache;
use std::num::NonZeroUsize;
use tracing::{info, warn};
use crate::{Error, Result};
use super::types::*;
use super::overrides::{OverrideResolutionResult, OverrideMetrics};

/// Comprehensive mapping validation report generator
#[derive(Debug)]
pub struct MappingReportGenerator {
    /// Report configuration
    config: ReportConfig,
    /// Template engine for HTML reports (placeholder for future implementation)
    template_engine: Option<String>,
    /// Report cache for performance
    report_cache: LruCache<String, CachedReport>,
    /// Historical data for trend analysis
    historical_data: HistoricalReportData,
    /// Performance metrics
    generation_metrics: ReportGenerationMetrics,
}

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
    /// When the report was generated
    pub generated_at: DateTime<Utc>,
    /// Information about the processed document
    pub document_info: DocumentInfo,
    /// High-level mapping summary
    pub mapping_summary: MappingSummary,
    /// Detailed field-by-field results
    pub detailed_results: Vec<FieldMappingResult>,
    /// Quality metrics and scores
    pub quality_metrics: QualityMetrics,
    /// Actionable recommendations
    pub recommendations: Vec<Recommendation>,
    /// Validation results
    pub validation_results: ValidationSummary,
    /// Override application results
    pub override_results: OverrideSummary,
    /// Performance metrics
    pub performance_metrics: ProcessingMetrics,
    /// Trend analysis (if historical data available)
    pub trend_analysis: Option<TrendAnalysis>,
}

/// Type of mapping report
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ReportType {
    /// Summary report with key metrics
    Summary,
    /// Detailed analysis report
    Detailed,
    /// Quality trend analysis
    QualityTrend,
    /// Compliance and audit report
    Compliance,
    /// Performance analysis report
    Performance,
    /// Custom report type
    Custom(String),
}

/// Report output format
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
    /// Number of rows processed
    pub total_rows: usize,
    /// Number of columns detected
    pub total_columns: usize,
    /// Processing timestamp
    pub processed_at: DateTime<Utc>,
    /// Processing duration
    pub processing_duration: Duration,
    /// Document hash for change detection
    pub document_hash: String,
}

/// High-level mapping summary statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MappingSummary {
    /// Total number of fields expected
    pub total_fields: usize,
    /// Number of fields successfully mapped
    pub mapped_fields: usize,
    /// Number of required fields mapped
    pub required_fields_mapped: usize,
    /// Number of required fields missing
    pub required_fields_missing: usize,
    /// Number of optional fields mapped
    pub optional_fields_mapped: usize,
    /// Average confidence score across all mappings
    pub average_confidence: f64,
    /// Minimum confidence score
    pub min_confidence: f64,
    /// Maximum confidence score
    pub max_confidence: f64,
    /// Number of high-confidence mappings (>0.9)
    pub high_confidence_mappings: usize,
    /// Number of low-confidence mappings (<0.5)
    pub low_confidence_mappings: usize,
    /// Total processing time
    pub processing_time: Duration,
    /// Mapping success rate (0.0-1.0)
    pub success_rate: f64,
}

/// Detailed result for a single field mapping
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FieldMappingResult {
    /// Expected field identifier
    pub field_id: String,
    /// Target OSCAL field path
    pub target_field: String,
    /// Source column name (if mapped)
    pub source_column: Option<String>,
    /// Mapping confidence score
    pub confidence_score: f64,
    /// Whether mapping was successful
    pub mapping_successful: bool,
    /// Whether field is required
    pub required: bool,
    /// Validation result for this field
    pub validation_result: Option<ColumnValidationResult>,
    /// Override applied (if any)
    pub override_applied: Option<String>,
    /// Alternative suggestions
    pub alternatives: Vec<MappingAlternative>,
    /// Issues and warnings
    pub issues: Vec<MappingIssue>,
    /// Data quality assessment
    pub data_quality: Option<DataQualityAssessment>,
}

/// Alternative mapping suggestion
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MappingAlternative {
    /// Alternative source column
    pub source_column: String,
    /// Confidence score for alternative
    pub confidence_score: f64,
    /// Reason for suggestion
    pub reason: String,
}

/// Mapping issue or warning
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MappingIssue {
    /// Issue severity level
    pub severity: IssueSeverity,
    /// Issue category
    pub category: IssueCategory,
    /// Issue description
    pub description: String,
    /// Suggested resolution
    pub suggested_resolution: Option<String>,
    /// Impact assessment
    pub impact: String,
}

/// Data quality assessment for a field
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataQualityAssessment {
    /// Completeness score (0.0-1.0)
    pub completeness: f64,
    /// Consistency score (0.0-1.0)
    pub consistency: f64,
    /// Validity score (0.0-1.0)
    pub validity: f64,
    /// Overall quality score (0.0-1.0)
    pub overall_quality: f64,
    /// Number of null/empty values
    pub null_count: usize,
    /// Number of invalid values
    pub invalid_count: usize,
    /// Sample valid values
    pub sample_values: Vec<String>,
    /// Detected data patterns
    pub patterns: Vec<String>,
}

/// Overall quality metrics for the mapping
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QualityMetrics {
    /// Data completeness score (0.0-1.0)
    pub completeness_score: f64,
    /// Mapping accuracy score (0.0-1.0)
    pub accuracy_score: f64,
    /// Data consistency score (0.0-1.0)
    pub consistency_score: f64,
    /// Overall quality score (0.0-1.0)
    pub overall_quality_score: f64,
    /// Risk level assessment
    pub risk_level: RiskLevel,
    /// Quality grade (A-F)
    pub quality_grade: QualityGrade,
    /// Compliance percentage
    pub compliance_percentage: f64,
    /// Number of critical issues
    pub critical_issues: usize,
    /// Number of warnings
    pub warnings: usize,
}

/// Actionable recommendation for improvement
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Recommendation {
    /// Recommendation priority
    pub priority: RecommendationPriority,
    /// Recommendation category
    pub category: RecommendationCategory,
    /// Recommendation title
    pub title: String,
    /// Detailed description
    pub description: String,
    /// Suggested action steps
    pub suggested_action: String,
    /// Expected impact of implementing
    pub impact_assessment: String,
    /// Effort required to implement
    pub effort_level: EffortLevel,
    /// Related field IDs (if applicable)
    pub related_fields: Vec<String>,
}

/// Summary of validation results
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationSummary {
    /// Total validations performed
    pub total_validations: usize,
    /// Number of validations passed
    pub validations_passed: usize,
    /// Number of validations failed
    pub validations_failed: usize,
    /// Number of validation warnings
    pub validation_warnings: usize,
    /// Validation success rate (0.0-1.0)
    pub success_rate: f64,
    /// Most common validation failures
    pub common_failures: Vec<ValidationFailureInfo>,
    /// Validation performance metrics
    pub performance_metrics: ValidationPerformanceMetrics,
}

/// Information about common validation failures
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationFailureInfo {
    /// Type of validation failure
    pub failure_type: String,
    /// Number of occurrences
    pub occurrence_count: usize,
    /// Percentage of total failures
    pub failure_percentage: f64,
    /// Example failing values
    pub example_values: Vec<String>,
    /// Suggested fixes
    pub suggested_fixes: Vec<String>,
}

/// Performance metrics for validation operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationPerformanceMetrics {
    /// Average validation time per field (microseconds)
    pub avg_validation_time_us: f64,
    /// Total validation time
    pub total_validation_time: Duration,
    /// Slowest validation operations
    pub slowest_validations: Vec<SlowValidationInfo>,
}

/// Information about slow validation operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SlowValidationInfo {
    /// Field that was slow to validate
    pub field_id: String,
    /// Validation time in microseconds
    pub validation_time_us: u64,
    /// Reason for slowness
    pub reason: String,
}

/// Summary of override application results
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OverrideSummary {
    /// Total number of override rules evaluated
    pub total_overrides_evaluated: usize,
    /// Number of overrides applied
    pub overrides_applied: usize,
    /// Number of override conflicts detected
    pub conflicts_detected: usize,
    /// Override application success rate
    pub application_success_rate: f64,
    /// Most frequently applied overrides
    pub frequently_applied: Vec<OverrideUsageInfo>,
    /// Override performance metrics
    pub performance_metrics: OverridePerformanceMetrics,
}

/// Information about override usage
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OverrideUsageInfo {
    /// Override rule name
    pub override_name: String,
    /// Number of times applied
    pub application_count: usize,
    /// Success rate for this override
    pub success_rate: f64,
    /// Average confidence when applied
    pub avg_confidence: f64,
}

/// Performance metrics for override operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OverridePerformanceMetrics {
    /// Average override resolution time (microseconds)
    pub avg_resolution_time_us: f64,
    /// Cache hit rate for override lookups
    pub cache_hit_rate: f64,
    /// Total override processing time
    pub total_processing_time: Duration,
}

/// Processing performance metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessingMetrics {
    /// Total processing time
    pub total_processing_time: Duration,
    /// Time spent on column detection
    pub column_detection_time: Duration,
    /// Time spent on mapping
    pub mapping_time: Duration,
    /// Time spent on validation
    pub validation_time: Duration,
    /// Time spent on override resolution
    pub override_time: Duration,
    /// Memory usage statistics
    pub memory_usage: MemoryUsageMetrics,
    /// Throughput metrics
    pub throughput: ThroughputMetrics,
}

/// Memory usage statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryUsageMetrics {
    /// Peak memory usage in bytes
    pub peak_memory_bytes: u64,
    /// Average memory usage in bytes
    pub avg_memory_bytes: u64,
    /// Memory efficiency score (0.0-1.0)
    pub efficiency_score: f64,
}

/// Throughput performance metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThroughputMetrics {
    /// Rows processed per second
    pub rows_per_second: f64,
    /// Fields processed per second
    pub fields_per_second: f64,
    /// Bytes processed per second
    pub bytes_per_second: f64,
}

/// Cached report for performance optimization
#[derive(Debug, Clone)]
pub struct CachedReport {
    /// The cached report
    pub report: MappingReport,
    /// When the report was cached
    pub cached_at: DateTime<Utc>,
    /// Cache expiration time
    pub expires_at: DateTime<Utc>,
    /// Report format
    pub format: ReportFormat,
}

/// Historical data for trend analysis
#[derive(Debug, Clone, Default)]
pub struct HistoricalReportData {
    /// Historical quality scores
    pub quality_history: Vec<HistoricalQualityScore>,
    /// Historical performance data
    pub performance_history: Vec<HistoricalPerformanceData>,
    /// Historical issue data
    pub issue_history: Vec<CommonIssueInfo>,
    /// Maximum history retention (days)
    pub max_retention_days: u32,
}

/// Report generation performance metrics
#[derive(Debug, Clone, Default)]
pub struct ReportGenerationMetrics {
    /// Total reports generated
    pub total_reports_generated: u64,
    /// Average generation time in milliseconds
    pub avg_generation_time_ms: f64,
    /// Cache hit rate
    pub cache_hit_rate: f64,
    /// Failed generation attempts
    pub failed_generations: u64,
    /// Last updated timestamp
    pub last_updated: DateTime<Utc>,
}

/// Trend analysis data (when historical data is available)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrendAnalysis {
    /// Time period covered by the analysis
    pub time_period: TimePeriod,
    /// Quality score trends
    pub quality_trends: QualityTrends,
    /// Performance trends
    pub performance_trends: PerformanceTrends,
    /// Common issues over time
    pub issue_trends: IssueTrends,
    /// Improvement recommendations based on trends
    pub trend_recommendations: Vec<TrendRecommendation>,
}

/// Time period for trend analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimePeriod {
    /// Start of the analysis period
    pub start_date: DateTime<Utc>,
    /// End of the analysis period
    pub end_date: DateTime<Utc>,
    /// Number of data points in the analysis
    pub data_points: usize,
}

/// Quality score trends over time
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QualityTrends {
    /// Overall quality score trend
    pub overall_quality_trend: TrendDirection,
    /// Completeness score trend
    pub completeness_trend: TrendDirection,
    /// Accuracy score trend
    pub accuracy_trend: TrendDirection,
    /// Consistency score trend
    pub consistency_trend: TrendDirection,
    /// Historical quality scores
    pub historical_scores: Vec<HistoricalQualityScore>,
}

/// Performance trends over time
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceTrends {
    /// Processing time trend
    pub processing_time_trend: TrendDirection,
    /// Throughput trend
    pub throughput_trend: TrendDirection,
    /// Memory usage trend
    pub memory_usage_trend: TrendDirection,
    /// Historical performance data
    pub historical_performance: Vec<HistoricalPerformanceData>,
}

/// Issue trends over time
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IssueTrends {
    /// Critical issues trend
    pub critical_issues_trend: TrendDirection,
    /// Warning trend
    pub warnings_trend: TrendDirection,
    /// Most common issues over time
    pub common_issues: Vec<CommonIssueInfo>,
}

/// Historical quality score data point
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistoricalQualityScore {
    /// Timestamp of the measurement
    pub timestamp: DateTime<Utc>,
    /// Overall quality score
    pub overall_score: f64,
    /// Completeness score
    pub completeness_score: f64,
    /// Accuracy score
    pub accuracy_score: f64,
    /// Consistency score
    pub consistency_score: f64,
}

/// Historical performance data point
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistoricalPerformanceData {
    /// Timestamp of the measurement
    pub timestamp: DateTime<Utc>,
    /// Processing time in milliseconds
    pub processing_time_ms: u64,
    /// Throughput in rows per second
    pub throughput_rps: f64,
    /// Memory usage in bytes
    pub memory_usage_bytes: u64,
}

/// Information about common issues over time
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommonIssueInfo {
    /// Issue category
    pub issue_category: IssueCategory,
    /// Trend direction for this issue
    pub trend: TrendDirection,
    /// Current occurrence rate
    pub current_rate: f64,
    /// Historical occurrence rates
    pub historical_rates: Vec<IssueRateDataPoint>,
}

/// Issue rate data point
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IssueRateDataPoint {
    /// Timestamp
    pub timestamp: DateTime<Utc>,
    /// Issue occurrence rate
    pub rate: f64,
}

/// Trend-based recommendation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrendRecommendation {
    /// Recommendation based on trend analysis
    pub recommendation: String,
    /// Supporting trend data
    pub supporting_data: String,
    /// Expected impact
    pub expected_impact: String,
    /// Priority based on trend severity
    pub priority: RecommendationPriority,
}

impl MappingReportGenerator {
    /// Create a new mapping report generator
    pub fn new() -> Self {
        Self {
            config: ReportConfig::default(),
            template_engine: None,
            report_cache: LruCache::new(std::num::NonZeroUsize::new(100).unwrap()),
            historical_data: HistoricalReportData::default(),
            generation_metrics: ReportGenerationMetrics::default(),
        }
    }

    /// Create a new report generator with custom configuration
    pub fn with_config(config: ReportConfig) -> Self {
        let mut generator = Self::new();
        generator.config = config;

        // Initialize template engine if template directory is specified
        if let Some(ref template_dir) = generator.config.template_directory {
            generator.template_engine = Some(template_dir.clone());
        }

        generator
    }

    /// Export report to specified format
    pub fn export_report(&self, report: &MappingReport, format: ReportFormat) -> Result<String> {
        match format {
            ReportFormat::Json => self.export_to_json(report),
            ReportFormat::Html => self.export_to_html(report),
            ReportFormat::Csv => self.export_to_csv(report),
            ReportFormat::Markdown => self.export_to_markdown(report),
            ReportFormat::Pdf => self.export_to_pdf(report),
        }
    }

    /// Export report to JSON format
    fn export_to_json(&self, report: &MappingReport) -> Result<String> {
        serde_json::to_string_pretty(report)
            .map_err(|e| Error::document_parsing(format!("Failed to serialize report to JSON: {}", e)))
    }

    /// Export report to HTML format
    fn export_to_html(&self, report: &MappingReport) -> Result<String> {
        // For now, always use simple HTML generation
        // Template engine integration can be added later
        Ok(self.generate_simple_html(report))
    }

    /// Export report to CSV format
    fn export_to_csv(&self, report: &MappingReport) -> Result<String> {
        let mut csv_content = String::new();

        // Header
        csv_content.push_str("Field ID,Target Field,Source Column,Confidence Score,Mapping Successful,Required,Issues\n");

        // Data rows
        for result in &report.detailed_results {
            csv_content.push_str(&format!(
                "{},{},{},{:.3},{},{},{}\n",
                result.field_id,
                result.target_field,
                result.source_column.as_deref().unwrap_or(""),
                result.confidence_score,
                result.mapping_successful,
                result.required,
                result.issues.len()
            ));
        }

        Ok(csv_content)
    }

    /// Export report to Markdown format
    fn export_to_markdown(&self, report: &MappingReport) -> Result<String> {
        let mut md_content = String::new();

        md_content.push_str(&format!("# Mapping Validation Report\n\n"));
        md_content.push_str(&format!("**Report ID:** {}\n", report.report_id));
        md_content.push_str(&format!("**Generated:** {}\n", report.generated_at.format("%Y-%m-%d %H:%M:%S UTC")));
        md_content.push_str(&format!("**Document:** {}\n\n", report.document_info.file_name));

        md_content.push_str("## Summary\n\n");
        md_content.push_str(&format!("- **Total Fields:** {}\n", report.mapping_summary.total_fields));
        md_content.push_str(&format!("- **Mapped Fields:** {}\n", report.mapping_summary.mapped_fields));
        md_content.push_str(&format!("- **Success Rate:** {:.1}%\n", report.mapping_summary.success_rate * 100.0));
        md_content.push_str(&format!("- **Average Confidence:** {:.1}%\n", report.mapping_summary.average_confidence * 100.0));
        md_content.push_str(&format!("- **Quality Grade:** {:?}\n\n", report.quality_metrics.quality_grade));

        md_content.push_str("## Recommendations\n\n");
        for (i, rec) in report.recommendations.iter().enumerate() {
            md_content.push_str(&format!("{}. **{}** (Priority: {:?})\n", i + 1, rec.title, rec.priority));
            md_content.push_str(&format!("   - {}\n", rec.description));
            md_content.push_str(&format!("   - Action: {}\n\n", rec.suggested_action));
        }

        Ok(md_content)
    }

    /// Export report to PDF format (placeholder implementation)
    fn export_to_pdf(&self, _report: &MappingReport) -> Result<String> {
        // This would require a PDF generation library like `printpdf` or `wkhtmltopdf`
        // For now, return an error indicating PDF export is not implemented
        Err(Error::document_parsing("PDF export not yet implemented".to_string()))
    }

    /// Generate simple HTML report (fallback when no template engine)
    fn generate_simple_html(&self, report: &MappingReport) -> String {
        format!(
            r#"<!DOCTYPE html>
<html>
<head>
    <title>Mapping Validation Report</title>
    <style>
        body {{ font-family: Arial, sans-serif; margin: 20px; }}
        .header {{ background-color: #f0f0f0; padding: 10px; border-radius: 5px; }}
        .summary {{ margin: 20px 0; }}
        .quality-grade {{ font-size: 24px; font-weight: bold; }}
        .recommendations {{ margin: 20px 0; }}
        .recommendation {{ margin: 10px 0; padding: 10px; border-left: 4px solid #007acc; }}
        table {{ border-collapse: collapse; width: 100%; }}
        th, td {{ border: 1px solid #ddd; padding: 8px; text-align: left; }}
        th {{ background-color: #f2f2f2; }}
    </style>
</head>
<body>
    <div class="header">
        <h1>Mapping Validation Report</h1>
        <p><strong>Report ID:</strong> {}</p>
        <p><strong>Generated:</strong> {}</p>
        <p><strong>Document:</strong> {}</p>
    </div>

    <div class="summary">
        <h2>Summary</h2>
        <p><strong>Total Fields:</strong> {}</p>
        <p><strong>Mapped Fields:</strong> {}</p>
        <p><strong>Success Rate:</strong> {:.1}%</p>
        <p><strong>Average Confidence:</strong> {:.1}%</p>
        <p class="quality-grade"><strong>Quality Grade:</strong> {:?}</p>
    </div>

    <div class="recommendations">
        <h2>Recommendations</h2>
        {}
    </div>
</body>
</html>"#,
            report.report_id,
            report.generated_at.format("%Y-%m-%d %H:%M:%S UTC"),
            report.document_info.file_name,
            report.mapping_summary.total_fields,
            report.mapping_summary.mapped_fields,
            report.mapping_summary.success_rate * 100.0,
            report.mapping_summary.average_confidence * 100.0,
            report.quality_metrics.quality_grade,
            report.recommendations.iter()
                .map(|r| format!(r#"<div class="recommendation"><strong>{}</strong> (Priority: {:?})<br>{}</div>"#, r.title, r.priority, r.description))
                .collect::<Vec<_>>()
                .join("")
        )
    }

    /// Get generation metrics
    pub fn get_metrics(&self) -> &ReportGenerationMetrics {
        &self.generation_metrics
    }

    /// Clear report cache
    pub fn clear_cache(&mut self) {
        self.report_cache.clear();
    }

    /// Get historical data
    pub fn get_historical_data(&self) -> &HistoricalReportData {
        &self.historical_data
    }
}

impl Default for MappingReportGenerator {
    fn default() -> Self {
        Self::new()
    }
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
