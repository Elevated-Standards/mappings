// Modified: 2025-09-22

//! Validation and mapping report generation
//!
//! This module provides comprehensive reporting functionality for validation results,
//! mapping analysis, and performance metrics with support for multiple output formats.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::Duration;
use uuid::Uuid;
use chrono::{DateTime, Utc};
use super::types::*;
use super::confidence::*;
use super::overrides::*;

/// Mapping report generator for creating comprehensive analysis reports
#[derive(Debug)]
pub struct MappingReportGenerator {
    /// Report configuration
    config: ReportConfig,
    /// Template engine for HTML reports (placeholder for future implementation)
    template_engine: Option<String>,
    /// Cache for generated reports
    report_cache: HashMap<String, CachedReport>,
    /// Historical report data
    historical_data: HistoricalReportData,
    /// Report generation metrics
    metrics: ReportGenerationMetrics,
}

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
    /// Report format
    pub format: ReportFormat,
    /// Generation timestamp
    pub generated_at: DateTime<Utc>,
    /// Document information
    pub document_info: DocumentInfo,
    /// Mapping summary
    pub mapping_summary: MappingSummary,
    /// Field mapping results
    pub field_mappings: Vec<FieldMappingResult>,
    /// Data quality assessment
    pub quality_assessment: DataQualityAssessment,
    /// Validation summary
    pub validation_summary: ValidationSummary,
    /// Override summary
    pub override_summary: OverrideSummary,
    /// Processing metrics
    pub processing_metrics: ProcessingMetrics,
    /// Trend analysis (if enabled)
    pub trend_analysis: Option<TrendAnalysis>,
    /// Recommendations
    pub recommendations: Vec<Recommendation>,
}

/// Types of reports that can be generated
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ReportType {
    /// Summary report with key metrics
    Summary,
    /// Detailed analysis report
    Detailed,
    /// Performance analysis report
    Performance,
    /// Quality assessment report
    Quality,
    /// Trend analysis report
    Trends,
    /// Compliance report
    Compliance,
    /// Custom report with specific focus
    Custom(String),
}

/// Supported report formats
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
    /// Excel format with multiple sheets
    Excel,
    /// Markdown format for documentation
    Markdown,
}

/// Document information for reports
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentInfo {
    /// Document file name
    pub file_name: String,
    /// Document type (inventory, poam, ssp)
    pub document_type: String,
    /// File size in bytes
    pub file_size: u64,
    /// Number of rows
    pub row_count: usize,
    /// Number of columns
    pub column_count: usize,
    /// Processing timestamp
    pub processed_at: DateTime<Utc>,
    /// Processing duration
    pub processing_duration: Duration,
    /// File hash for integrity checking
    pub file_hash: String,
    /// Encoding detected
    pub encoding: String,
}

/// Summary of mapping results
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MappingSummary {
    /// Total number of fields expected
    pub total_fields: usize,
    /// Number of fields successfully mapped
    pub mapped_fields: usize,
    /// Number of fields with high confidence
    pub high_confidence_mappings: usize,
    /// Number of fields requiring review
    pub review_required: usize,
    /// Number of missing required fields
    pub missing_required: usize,
    /// Overall mapping success rate
    pub success_rate: f64,
    /// Average confidence score
    pub avg_confidence_score: f64,
    /// Number of overrides applied
    pub overrides_applied: usize,
    /// Number of conflicts resolved
    pub conflicts_resolved: usize,
    /// Data completeness percentage
    pub completeness_percentage: f64,
    /// Data quality score
    pub quality_score: f64,
}

/// Individual field mapping result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FieldMappingResult {
    /// Expected field identifier
    pub field_id: String,
    /// Target OSCAL field path
    pub oscal_field: String,
    /// Source column name (if mapped)
    pub source_column: Option<String>,
    /// Mapping confidence score
    pub confidence_score: f64,
    /// Mapping status
    pub status: ValidationStatus,
    /// Whether an override was applied
    pub override_applied: bool,
    /// Alternative mappings considered
    pub alternatives: Vec<MappingAlternative>,
    /// Issues encountered
    pub issues: Vec<MappingIssue>,
    /// Data samples from the field
    pub data_samples: Vec<String>,
}

/// Alternative mapping option
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MappingAlternative {
    /// Alternative source column
    pub source_column: String,
    /// Confidence score for alternative
    pub confidence_score: f64,
    /// Reason why this wasn't selected
    pub rejection_reason: String,
}

/// Issue encountered during mapping
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MappingIssue {
    /// Issue severity level
    pub severity: IssueSeverity,
    /// Issue category
    pub category: IssueCategory,
    /// Issue description
    pub description: String,
    /// Suggested resolution
    pub suggested_resolution: String,
    /// Whether this issue blocks processing
    pub blocking: bool,
}

/// Severity levels for mapping issues
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, PartialOrd)]
pub enum IssueSeverity {
    /// Informational message
    Info,
    /// Warning that should be reviewed
    Warning,
    /// Error that should be fixed
    Error,
    /// Critical error that blocks processing
    Critical,
}

/// Categories of mapping issues
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum IssueCategory {
    /// Missing required field
    MissingRequired,
    /// Low confidence mapping
    LowConfidence,
    /// Data type mismatch
    DataTypeMismatch,
    /// Format validation failure
    FormatValidation,
    /// Enumeration validation failure
    EnumerationValidation,
    /// Cross-field validation failure
    CrossFieldValidation,
    /// Override conflict
    OverrideConflict,
    /// Performance issue
    Performance,
    /// Data quality issue
    DataQuality,
}

/// Data quality assessment
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

/// Detailed quality metrics
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

/// Risk levels for data quality
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

/// Quality grades
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum QualityGrade {
    /// Excellent quality (90-100%)
    A,
    /// Good quality (80-89%)
    B,
    /// Fair quality (70-79%)
    C,
    /// Poor quality (60-69%)
    D,
    /// Very poor quality (< 60%)
    F,
}

/// Recommendation for improvement
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
    /// Estimated effort to implement
    pub effort_level: EffortLevel,
    /// Expected impact
    pub expected_impact: String,
    /// Implementation steps
    pub implementation_steps: Vec<String>,
    /// Related fields or areas
    pub related_fields: Vec<String>,
}

/// Categories of recommendations
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum RecommendationCategory {
    /// Data quality improvement
    DataQuality,
    /// Mapping accuracy improvement
    MappingAccuracy,
    /// Performance optimization
    Performance,
    /// Process improvement
    Process,
    /// Training and documentation
    Training,
    /// Tool configuration
    Configuration,
    /// Data governance
    Governance,
}

/// Override summary for reports
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OverrideSummary {
    /// Total number of override rules evaluated
    pub total_overrides_evaluated: usize,
    /// Number of overrides applied
    pub overrides_applied: usize,
    /// Number of conflicts encountered
    pub conflicts_encountered: usize,
    /// Most frequently used overrides
    pub most_used_overrides: Vec<OverrideUsageInfo>,
    /// Override performance metrics
    pub performance_metrics: OverridePerformanceMetrics,
}

/// Override usage information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OverrideUsageInfo {
    /// Override rule name
    pub override_name: String,
    /// Number of times applied
    pub usage_count: usize,
    /// Success rate
    pub success_rate: f64,
    /// Average confidence score
    pub avg_confidence: f64,
}

/// Override performance metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OverridePerformanceMetrics {
    /// Average override resolution time (microseconds)
    pub avg_resolution_time_us: f64,
    /// Cache hit rate for override lookups
    pub cache_hit_rate: f64,
    /// Number of validation failures
    pub validation_failures: usize,
}

/// Processing metrics for reports
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessingMetrics {
    /// Total processing time
    pub total_processing_time: Duration,
    /// Time spent on column detection
    pub column_detection_time: Duration,
    /// Time spent on mapping resolution
    pub mapping_resolution_time: Duration,
    /// Time spent on validation
    pub validation_time: Duration,
    /// Time spent on override resolution
    pub override_resolution_time: Duration,
    /// Memory usage metrics
    pub memory_metrics: MemoryUsageMetrics,
    /// Throughput metrics
    pub throughput_metrics: ThroughputMetrics,
}

/// Memory usage metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryUsageMetrics {
    /// Peak memory usage in bytes
    pub peak_memory_bytes: u64,
    /// Average memory usage in bytes
    pub avg_memory_bytes: u64,
    /// Memory usage at end of processing
    pub final_memory_bytes: u64,
}

/// Throughput metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThroughputMetrics {
    /// Rows processed per second
    pub rows_per_second: f64,
    /// Fields processed per second
    pub fields_per_second: f64,
    /// Validations per second
    pub validations_per_second: f64,
}

/// Trend analysis data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrendAnalysis {
    /// Time period covered by the analysis
    pub time_period: TimePeriod,
    /// Quality score trends
    pub quality_trends: QualityTrends,
    /// Performance trends
    pub performance_trends: PerformanceTrends,
    /// Issue trends
    pub issue_trends: IssueTrends,
    /// Historical data points
    pub historical_quality: Vec<HistoricalQualityScore>,
    /// Historical performance data
    pub historical_performance: Vec<HistoricalPerformanceData>,
    /// Common issues over time
    pub common_issues: Vec<CommonIssueInfo>,
    /// Trend-based recommendations
    pub trend_recommendations: Vec<TrendRecommendation>,
}

/// Time period for analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimePeriod {
    /// Start of the analysis period
    pub start_date: DateTime<Utc>,
    /// End of the analysis period
    pub end_date: DateTime<Utc>,
    /// Description of the period
    pub description: String,
}

/// Quality trends over time
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QualityTrends {
    /// Overall quality score trend
    pub overall_quality_trend: TrendDirection,
    /// Completeness score trend
    pub completeness_trend: TrendDirection,
    /// Accuracy trend
    pub accuracy_trend: TrendDirection,
    /// Consistency trend
    pub consistency_trend: TrendDirection,
    /// Validity trend
    pub validity_trend: TrendDirection,
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
    /// Error rate trend
    pub error_rate_trend: TrendDirection,
}

/// Issue trends over time
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IssueTrends {
    /// Critical issues trend
    pub critical_issues_trend: TrendDirection,
    /// Warning trend
    pub warning_trend: TrendDirection,
    /// Missing required fields trend
    pub missing_required_trend: TrendDirection,
}

/// Direction of a trend
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum TrendDirection {
    /// Improving trend
    Improving,
    /// Stable trend
    Stable,
    /// Declining trend
    Declining,
    /// Insufficient data
    InsufficientData,
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
    /// Document type
    pub document_type: String,
}

/// Historical performance data point
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistoricalPerformanceData {
    /// Timestamp of the measurement
    pub timestamp: DateTime<Utc>,
    /// Processing time in milliseconds
    pub processing_time_ms: f64,
    /// Memory usage in bytes
    pub memory_usage_bytes: u64,
    /// Throughput (rows per second)
    pub throughput_rps: f64,
    /// Error count
    pub error_count: usize,
}

/// Common issue information over time
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommonIssueInfo {
    /// Issue category
    pub issue_category: IssueCategory,
    /// Trend direction for this issue
    pub trend_direction: TrendDirection,
    /// Current occurrence rate
    pub current_rate: f64,
    /// Historical rates
    pub historical_rates: Vec<IssueRateDataPoint>,
    /// Impact description
    pub impact_description: String,
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
    pub supporting_trends: Vec<String>,
    /// Priority level
    pub priority: RecommendationPriority,
    /// Expected impact
    pub expected_impact: String,
    /// Implementation timeline
    pub timeline: String,
}

/// Cached report for performance
#[derive(Debug, Clone)]
pub struct CachedReport {
    /// The cached report
    pub report: MappingReport,
    /// When the report was cached
    pub cached_at: DateTime<Utc>,
    /// Cache expiry time
    pub expires_at: DateTime<Utc>,
    /// Cache hit count
    pub hit_count: u64,
}

/// Historical report data for trend analysis
#[derive(Debug, Clone, Default)]
pub struct HistoricalReportData {
    /// Historical quality scores
    pub quality_history: Vec<HistoricalQualityScore>,
    /// Historical performance data
    pub performance_history: Vec<HistoricalPerformanceData>,
    /// Historical issue data
    pub issue_history: Vec<CommonIssueInfo>,
    /// Report generation history
    pub generation_history: Vec<DateTime<Utc>>,
}

/// Report generation metrics
#[derive(Debug, Clone, Default)]
pub struct ReportGenerationMetrics {
    /// Total reports generated
    pub total_reports_generated: u64,
    /// Average generation time in milliseconds
    pub avg_generation_time_ms: f64,
    /// Cache hit rate
    pub cache_hit_rate: f64,
    /// Most requested report types
    pub popular_report_types: HashMap<ReportType, u64>,
    /// Error count during generation
    pub generation_errors: u64,
    /// Peak memory usage during generation
    pub peak_memory_usage_bytes: u64,
}

impl MappingReportGenerator {
    /// Create a new mapping report generator
    pub fn new() -> Self {
        Self {
            config: ReportConfig::default(),
            template_engine: None,
            report_cache: HashMap::new(),
            historical_data: HistoricalReportData::default(),
            metrics: ReportGenerationMetrics::default(),
        }
    }

    /// Generate a comprehensive mapping report
    pub fn generate_report(
        &mut self,
        report_type: ReportType,
        validation_report: &ColumnValidationReport,
        confidence_results: &[MappingConfidence],
        override_summary: &OverrideSummary,
    ) -> Result<MappingReport, String> {
        let start_time = std::time::Instant::now();

        let report_id = Uuid::new_v4();

        // Create document info
        let document_info = DocumentInfo {
            file_name: "document.xlsx".to_string(), // Would be passed in real implementation
            document_type: validation_report.document_type.clone(),
            file_size: 0, // Would be calculated
            row_count: 0, // Would be passed in
            column_count: validation_report.field_results.len(),
            processed_at: chrono::Utc::now(),
            processing_duration: validation_report.total_execution_time,
            file_hash: "hash".to_string(), // Would be calculated
            encoding: "UTF-8".to_string(),
        };

        // Create mapping summary
        let mapping_summary = self.create_mapping_summary(validation_report, confidence_results);

        // Create field mappings
        let field_mappings = self.create_field_mappings(validation_report, confidence_results);

        // Create quality assessment
        let quality_assessment = self.create_quality_assessment(validation_report);

        // Create validation summary
        let validation_summary = self.create_validation_summary(validation_report);

        // Create processing metrics
        let processing_metrics = self.create_processing_metrics(validation_report);

        // Create recommendations
        let recommendations = self.create_recommendations(validation_report, confidence_results);

        let report = MappingReport {
            report_id,
            report_type: report_type.clone(),
            format: self.config.default_format.clone(),
            generated_at: chrono::Utc::now(),
            document_info,
            mapping_summary,
            field_mappings,
            quality_assessment,
            validation_summary,
            override_summary: override_summary.clone(),
            processing_metrics,
            trend_analysis: None, // Would be populated if historical data available
            recommendations,
        };

        // Update metrics
        self.metrics.total_reports_generated += 1;
        self.metrics.avg_generation_time_ms =
            (self.metrics.avg_generation_time_ms + start_time.elapsed().as_millis() as f64) / 2.0;

        *self.metrics.popular_report_types.entry(report_type).or_insert(0) += 1;

        Ok(report)
    }

    /// Create mapping summary from validation results
    fn create_mapping_summary(
        &self,
        validation_report: &ColumnValidationReport,
        confidence_results: &[MappingConfidence],
    ) -> MappingSummary {
        let total_fields = validation_report.field_results.len();
        let mapped_fields = validation_report.field_results.iter()
            .filter(|r| r.source_column.is_some())
            .count();

        let high_confidence_mappings = confidence_results.iter()
            .filter(|c| c.overall_score >= 0.8)
            .count();

        let review_required = confidence_results.iter()
            .filter(|c| c.overall_score < 0.7 && c.overall_score >= 0.4)
            .count();

        let avg_confidence_score = if !confidence_results.is_empty() {
            confidence_results.iter().map(|c| c.overall_score).sum::<f64>() / confidence_results.len() as f64
        } else {
            0.0
        };

        MappingSummary {
            total_fields,
            mapped_fields,
            high_confidence_mappings,
            review_required,
            missing_required: validation_report.missing_required.len(),
            success_rate: if total_fields > 0 { mapped_fields as f64 / total_fields as f64 } else { 0.0 },
            avg_confidence_score,
            overrides_applied: 0, // Would be calculated from override results
            conflicts_resolved: 0, // Would be calculated from conflict resolution
            completeness_percentage: validation_report.metrics.validation_score * 100.0,
            quality_score: validation_report.metrics.validation_score,
        }
    }

    /// Create field mappings from validation results
    fn create_field_mappings(
        &self,
        validation_report: &ColumnValidationReport,
        confidence_results: &[MappingConfidence],
    ) -> Vec<FieldMappingResult> {
        validation_report.field_results.iter().enumerate().map(|(i, field_result)| {
            let confidence_score = confidence_results.get(i)
                .map(|c| c.overall_score)
                .unwrap_or(0.0);

            FieldMappingResult {
                field_id: field_result.field_id.clone(),
                oscal_field: field_result.oscal_field.clone(),
                source_column: field_result.source_column.clone(),
                confidence_score,
                status: field_result.status.clone(),
                override_applied: false, // Would be determined from override results
                alternatives: Vec::new(), // Would be populated from alternative mappings
                issues: Vec::new(), // Would be populated from validation issues
                data_samples: Vec::new(), // Would be populated from actual data
            }
        }).collect()
    }

    /// Create quality assessment from validation results
    fn create_quality_assessment(&self, validation_report: &ColumnValidationReport) -> DataQualityAssessment {
        let completeness = validation_report.metrics.validation_score;
        let consistency = 0.8; // Would be calculated from actual consistency checks
        let accuracy = 0.85; // Would be calculated from accuracy validation
        let validity = if validation_report.metrics.error_count == 0 { 1.0 } else { 0.7 };

        let overall_score = (completeness + consistency + accuracy + validity) / 4.0;

        let quality_grade = match overall_score {
            s if s >= 0.9 => QualityGrade::A,
            s if s >= 0.8 => QualityGrade::B,
            s if s >= 0.7 => QualityGrade::C,
            s if s >= 0.6 => QualityGrade::D,
            _ => QualityGrade::F,
        };

        let risk_level = match overall_score {
            s if s >= 0.8 => RiskLevel::Low,
            s if s >= 0.6 => RiskLevel::Medium,
            s if s >= 0.4 => RiskLevel::High,
            _ => RiskLevel::VeryHigh,
        };

        DataQualityAssessment {
            completeness,
            consistency,
            accuracy,
            validity,
            overall_score,
            quality_grade,
            risk_level,
            detailed_metrics: QualityMetrics {
                completeness_score: completeness,
                mapping_accuracy: accuracy,
                format_compliance: validity,
                business_rule_compliance: 0.9,
                cross_field_consistency: consistency,
                data_freshness: 1.0,
                duplicate_score: 0.95,
                outlier_score: 0.9,
                reference_validation: 0.85,
            },
        }
    }

    /// Create validation summary
    fn create_validation_summary(&self, validation_report: &ColumnValidationReport) -> ValidationSummary {
        ValidationSummary {
            total_validations: validation_report.metrics.total_fields,
            passed_validations: validation_report.metrics.valid_fields,
            failed_validations: validation_report.metrics.invalid_fields,
            warning_count: validation_report.metrics.warning_count,
            error_count: validation_report.metrics.error_count,
            overall_score: validation_report.metrics.validation_score,
            common_failures: Vec::new(), // Would be populated from failure analysis
            performance_metrics: ValidationPerformanceMetrics {
                avg_validation_time_us: validation_report.total_execution_time.as_micros() as f64 / validation_report.metrics.total_fields as f64,
                total_validation_time: validation_report.total_execution_time,
                slow_validations: Vec::new(),
            },
        }
    }

    /// Create processing metrics
    fn create_processing_metrics(&self, validation_report: &ColumnValidationReport) -> ProcessingMetrics {
        ProcessingMetrics {
            total_processing_time: validation_report.total_execution_time,
            column_detection_time: Duration::from_millis(10), // Placeholder
            mapping_resolution_time: Duration::from_millis(50), // Placeholder
            validation_time: validation_report.total_execution_time,
            override_resolution_time: Duration::from_millis(5), // Placeholder
            memory_metrics: MemoryUsageMetrics {
                peak_memory_bytes: 1024 * 1024, // Placeholder
                avg_memory_bytes: 512 * 1024, // Placeholder
                final_memory_bytes: 256 * 1024, // Placeholder
            },
            throughput_metrics: ThroughputMetrics {
                rows_per_second: 1000.0, // Placeholder
                fields_per_second: 100.0, // Placeholder
                validations_per_second: 50.0, // Placeholder
            },
        }
    }

    /// Create recommendations based on validation results
    fn create_recommendations(
        &self,
        validation_report: &ColumnValidationReport,
        _confidence_results: &[MappingConfidence],
    ) -> Vec<Recommendation> {
        let mut recommendations = Vec::new();

        // Add recommendation for missing required fields
        if !validation_report.missing_required.is_empty() {
            recommendations.push(Recommendation {
                priority: RecommendationPriority::High,
                category: RecommendationCategory::DataQuality,
                title: "Address Missing Required Fields".to_string(),
                description: format!("{} required fields are missing and should be added", validation_report.missing_required.len()),
                effort_level: EffortLevel::Medium,
                expected_impact: "Improved compliance and data completeness".to_string(),
                implementation_steps: vec![
                    "Review missing field list".to_string(),
                    "Update data source to include required fields".to_string(),
                    "Re-validate document".to_string(),
                ],
                related_fields: validation_report.missing_required.iter().map(|f| f.field_id.clone()).collect(),
            });
        }

        // Add recommendation for low validation score
        if validation_report.metrics.validation_score < 0.8 {
            recommendations.push(Recommendation {
                priority: RecommendationPriority::Medium,
                category: RecommendationCategory::DataQuality,
                title: "Improve Overall Data Quality".to_string(),
                description: format!("Current validation score is {:.1}% - consider data quality improvements", validation_report.metrics.validation_score * 100.0),
                effort_level: EffortLevel::High,
                expected_impact: "Better compliance and reduced manual review effort".to_string(),
                implementation_steps: vec![
                    "Analyze validation failures".to_string(),
                    "Implement data quality checks at source".to_string(),
                    "Establish data governance processes".to_string(),
                ],
                related_fields: Vec::new(),
            });
        }

        recommendations
    }
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
