//! Processing metrics and performance tracking for reports
//! Modified: 2025-01-22

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::Duration;
use chrono::{DateTime, Utc};

use super::types::*;
use crate::validation::types::{TrendDirection, IssueCategory, RecommendationPriority};

/// Processing performance metrics
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

/// Memory usage tracking
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryUsageMetrics {
    /// Peak memory usage in bytes
    pub peak_memory_bytes: u64,
    /// Average memory usage in bytes
    pub avg_memory_bytes: u64,
    /// Final memory usage in bytes
    pub final_memory_bytes: u64,
}

/// Throughput performance metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThroughputMetrics {
    /// Rows processed per second
    pub rows_per_second: f64,
    /// Fields processed per second
    pub fields_per_second: f64,
    /// Validations performed per second
    pub validations_per_second: f64,
}

/// Cached report information
#[derive(Debug, Clone)]
pub struct CachedReport {
    /// The cached report
    pub report: MappingReport,
    /// When the report was cached
    pub cached_at: DateTime<Utc>,
    /// Cache expiration time
    pub expires_at: DateTime<Utc>,
    /// Number of times this report was accessed
    pub access_count: u64,
    /// Size of the cached report in bytes
    pub size_bytes: u64,
}

/// Historical report data storage
#[derive(Debug, Clone, Default)]
pub struct HistoricalReportData {
    /// Historical quality scores
    pub quality_history: Vec<HistoricalQualityScore>,
    /// Historical performance data
    pub performance_history: Vec<HistoricalPerformanceData>,
    /// Historical issue data
    pub issue_history: Vec<CommonIssueInfo>,
    /// Maximum number of historical data points to retain
    pub max_history_points: usize,
}

/// Report generation metrics tracking
#[derive(Debug, Clone, Default)]
pub struct ReportGenerationMetrics {
    /// Total reports generated
    pub total_reports_generated: u64,
    /// Average generation time in milliseconds
    pub avg_generation_time_ms: f64,
    /// Peak memory usage during generation
    pub peak_memory_usage_bytes: u64,
    /// Cache hit rate for report generation
    pub cache_hit_rate: f64,
    /// Most popular report types
    pub popular_report_types: HashMap<ReportType, u64>,
    /// Error rate during generation
    pub error_rate: f64,
}

/// Historical trend analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrendAnalysis {
    /// Time period covered by the analysis
    pub time_period: TimePeriod,
    /// Quality score trends
    pub quality_trends: QualityTrends,
    /// Performance trends
    pub performance_trends: PerformanceTrends,
    /// Issue occurrence trends
    pub issue_trends: IssueTrends,
    /// Historical data points
    pub historical_quality_scores: Vec<HistoricalQualityScore>,
    /// Historical performance data
    pub historical_performance: Vec<HistoricalPerformanceData>,
    /// Common issues over time
    pub common_issues: Vec<CommonIssueInfo>,
    /// Trend-based recommendations
    pub trend_recommendations: Vec<TrendRecommendation>,
}

/// Time period for trend analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimePeriod {
    /// Start of the analysis period
    pub start_date: DateTime<Utc>,
    /// End of the analysis period
    pub end_date: DateTime<Utc>,
    /// Number of data points in the period
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
    /// Validity score trend
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

/// Issue occurrence trends
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IssueTrends {
    /// Critical issues trend
    pub critical_issues_trend: TrendDirection,
    /// Warning trend
    pub warning_trend: TrendDirection,
    /// Error trend
    pub error_trend: TrendDirection,
    /// Overall issue rate trend
    pub overall_issue_trend: TrendDirection,
}



/// Historical quality score data point
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistoricalQualityScore {
    /// Timestamp of the measurement
    pub timestamp: DateTime<Utc>,
    /// Overall quality score
    pub overall_score: f64,
    /// Completeness score
    pub completeness: f64,
    /// Accuracy score
    pub accuracy: f64,
    /// Consistency score
    pub consistency: f64,
    /// Validity score
    pub validity: f64,
    /// Number of documents processed
    pub document_count: usize,
}

/// Historical performance data point
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistoricalPerformanceData {
    /// Timestamp of the measurement
    pub timestamp: DateTime<Utc>,
    /// Processing time in milliseconds
    pub processing_time_ms: f64,
    /// Throughput (rows per second)
    pub throughput_rps: f64,
    /// Memory usage in MB
    pub memory_usage_mb: f64,
    /// Error rate (0.0-1.0)
    pub error_rate: f64,
    /// Number of documents processed
    pub document_count: usize,
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
    /// Impact on overall quality
    pub quality_impact: f64,
}

/// Data point for issue occurrence rate
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IssueRateDataPoint {
    /// Timestamp
    pub timestamp: DateTime<Utc>,
    /// Issue occurrence rate
    pub rate: f64,
}

/// Recommendation based on trend analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrendRecommendation {
    /// Recommendation based on trend analysis
    pub recommendation: String,
    /// Supporting trend data
    pub supporting_trends: Vec<String>,
    /// Confidence in the recommendation (0.0-1.0)
    pub confidence: f64,
    /// Expected impact if implemented
    pub expected_impact: String,
    /// Time frame for implementation
    pub time_frame: String,
    /// Priority level
    pub priority: RecommendationPriority,
}

impl ProcessingMetrics {
    /// Create new processing metrics with default values
    pub fn new() -> Self {
        Self {
            total_processing_time: Duration::from_millis(0),
            column_detection_time: Duration::from_millis(0),
            mapping_resolution_time: Duration::from_millis(0),
            validation_time: Duration::from_millis(0),
            override_resolution_time: Duration::from_millis(0),
            memory_metrics: MemoryUsageMetrics::new(),
            throughput_metrics: ThroughputMetrics::new(),
        }
    }

    /// Calculate total processing time from components
    pub fn calculate_total_time(&mut self) {
        self.total_processing_time = self.column_detection_time
            + self.mapping_resolution_time
            + self.validation_time
            + self.override_resolution_time;
    }

    /// Get processing efficiency score (0.0-1.0)
    pub fn efficiency_score(&self) -> f64 {
        let total_ms = self.total_processing_time.as_millis() as f64;
        if total_ms == 0.0 {
            return 1.0;
        }

        // Efficiency based on throughput and memory usage
        let throughput_score = (self.throughput_metrics.rows_per_second / 1000.0).min(1.0);
        let memory_efficiency = if self.memory_metrics.peak_memory_bytes > 0 {
            (self.memory_metrics.avg_memory_bytes as f64 / self.memory_metrics.peak_memory_bytes as f64)
        } else {
            1.0
        };

        (throughput_score + memory_efficiency) / 2.0
    }
}

impl MemoryUsageMetrics {
    /// Create new memory metrics with default values
    pub fn new() -> Self {
        Self {
            peak_memory_bytes: 0,
            avg_memory_bytes: 0,
            final_memory_bytes: 0,
        }
    }

    /// Calculate memory efficiency (0.0-1.0)
    pub fn memory_efficiency(&self) -> f64 {
        if self.peak_memory_bytes == 0 {
            return 1.0;
        }

        self.avg_memory_bytes as f64 / self.peak_memory_bytes as f64
    }

    /// Get memory usage in MB
    pub fn peak_memory_mb(&self) -> f64 {
        self.peak_memory_bytes as f64 / (1024.0 * 1024.0)
    }

    /// Get average memory usage in MB
    pub fn avg_memory_mb(&self) -> f64 {
        self.avg_memory_bytes as f64 / (1024.0 * 1024.0)
    }
}

impl ThroughputMetrics {
    /// Create new throughput metrics with default values
    pub fn new() -> Self {
        Self {
            rows_per_second: 0.0,
            fields_per_second: 0.0,
            validations_per_second: 0.0,
        }
    }

    /// Calculate throughput from processing time and counts
    pub fn calculate_throughput(
        &mut self,
        row_count: usize,
        field_count: usize,
        validation_count: usize,
        processing_time: Duration,
    ) {
        let seconds = processing_time.as_secs_f64();
        if seconds > 0.0 {
            self.rows_per_second = row_count as f64 / seconds;
            self.fields_per_second = field_count as f64 / seconds;
            self.validations_per_second = validation_count as f64 / seconds;
        }
    }

    /// Get overall throughput score (0.0-1.0)
    pub fn throughput_score(&self) -> f64 {
        // Normalize based on expected performance thresholds
        let row_score = (self.rows_per_second / 1000.0).min(1.0);
        let field_score = (self.fields_per_second / 100.0).min(1.0);
        let validation_score = (self.validations_per_second / 50.0).min(1.0);

        (row_score + field_score + validation_score) / 3.0
    }
}

impl ReportGenerationMetrics {
    /// Update metrics after generating a report
    pub fn update_after_generation(
        &mut self,
        report_type: ReportType,
        generation_time: Duration,
        memory_used: u64,
        success: bool,
    ) {
        self.total_reports_generated += 1;
        
        // Update average generation time
        let new_time_ms = generation_time.as_millis() as f64;
        self.avg_generation_time_ms = 
            (self.avg_generation_time_ms + new_time_ms) / 2.0;

        // Update peak memory usage
        if memory_used > self.peak_memory_usage_bytes {
            self.peak_memory_usage_bytes = memory_used;
        }

        // Update popular report types
        *self.popular_report_types.entry(report_type).or_insert(0) += 1;

        // Update error rate
        if !success {
            let total_errors = (self.error_rate * (self.total_reports_generated - 1) as f64) + 1.0;
            self.error_rate = total_errors / self.total_reports_generated as f64;
        } else {
            let total_errors = self.error_rate * (self.total_reports_generated - 1) as f64;
            self.error_rate = total_errors / self.total_reports_generated as f64;
        }
    }

    /// Get generation performance score (0.0-1.0)
    pub fn performance_score(&self) -> f64 {
        let time_score = if self.avg_generation_time_ms > 0.0 {
            (1000.0 / self.avg_generation_time_ms).min(1.0)
        } else {
            1.0
        };

        let error_score = 1.0 - self.error_rate;
        let cache_score = self.cache_hit_rate;

        (time_score + error_score + cache_score) / 3.0
    }
}

impl HistoricalReportData {
    /// Create new historical data with default retention
    pub fn new() -> Self {
        Self {
            quality_history: Vec::new(),
            performance_history: Vec::new(),
            issue_history: Vec::new(),
            max_history_points: 100, // Default to 100 data points
        }
    }

    /// Add quality score to history
    pub fn add_quality_score(&mut self, score: HistoricalQualityScore) {
        self.quality_history.push(score);
        if self.quality_history.len() > self.max_history_points {
            self.quality_history.remove(0);
        }
    }

    /// Add performance data to history
    pub fn add_performance_data(&mut self, data: HistoricalPerformanceData) {
        self.performance_history.push(data);
        if self.performance_history.len() > self.max_history_points {
            self.performance_history.remove(0);
        }
    }

    /// Get recent quality trend
    pub fn get_recent_quality_trend(&self, days: u32) -> TrendDirection {
        let cutoff = Utc::now() - chrono::Duration::days(days as i64);
        let recent_scores: Vec<f64> = self.quality_history.iter()
            .filter(|q| q.timestamp > cutoff)
            .map(|q| q.overall_score)
            .collect();

        if recent_scores.len() < 2 {
            return TrendDirection::InsufficientData;
        }

        // Simple trend calculation
        let first = recent_scores[0];
        let last = recent_scores[recent_scores.len() - 1];
        let change = (last - first) / first.max(0.001);

        match change {
            c if c > 0.05 => TrendDirection::Improving,
            c if c < -0.05 => TrendDirection::Declining,
            _ => TrendDirection::Stable,
        }
    }
}

impl Default for ProcessingMetrics {
    fn default() -> Self {
        Self::new()
    }
}

impl Default for MemoryUsageMetrics {
    fn default() -> Self {
        Self::new()
    }
}

impl Default for ThroughputMetrics {
    fn default() -> Self {
        Self::new()
    }
}
