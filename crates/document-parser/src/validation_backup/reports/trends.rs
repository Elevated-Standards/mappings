//! Historical trend analysis for reports
//! Modified: 2025-01-22

use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};

use super::types::*;

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

/// Direction of a trend
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum TrendDirection {
    /// Improving trend
    Improving,
    /// Stable trend
    Stable,
    /// Declining trend
    Declining,
    /// Volatile trend
    Volatile,
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

impl TrendAnalysis {
    /// Create a new trend analysis
    pub fn new(time_period: TimePeriod) -> Self {
        Self {
            time_period,
            quality_trends: QualityTrends::default(),
            performance_trends: PerformanceTrends::default(),
            issue_trends: IssueTrends::default(),
            historical_quality_scores: Vec::new(),
            historical_performance: Vec::new(),
            common_issues: Vec::new(),
            trend_recommendations: Vec::new(),
        }
    }

    /// Analyze trends from historical data
    pub fn analyze_trends(&mut self, historical_data: &HistoricalReportData) {
        self.analyze_quality_trends(&historical_data.quality_history);
        self.analyze_performance_trends(&historical_data.performance_history);
        self.analyze_issue_trends(&historical_data.issue_history);
        self.generate_trend_recommendations();
    }

    /// Analyze quality score trends
    fn analyze_quality_trends(&mut self, quality_history: &[HistoricalQualityScore]) {
        if quality_history.len() < 2 {
            self.quality_trends = QualityTrends::insufficient_data();
            return;
        }

        self.quality_trends.overall_quality_trend = 
            self.calculate_trend_direction(quality_history.iter().map(|q| q.overall_score).collect());
        self.quality_trends.completeness_trend = 
            self.calculate_trend_direction(quality_history.iter().map(|q| q.completeness).collect());
        self.quality_trends.accuracy_trend = 
            self.calculate_trend_direction(quality_history.iter().map(|q| q.accuracy).collect());
        self.quality_trends.consistency_trend = 
            self.calculate_trend_direction(quality_history.iter().map(|q| q.consistency).collect());
        self.quality_trends.validity_trend = 
            self.calculate_trend_direction(quality_history.iter().map(|q| q.validity).collect());
    }

    /// Analyze performance trends
    fn analyze_performance_trends(&mut self, performance_history: &[HistoricalPerformanceData]) {
        if performance_history.len() < 2 {
            self.performance_trends = PerformanceTrends::insufficient_data();
            return;
        }

        // For processing time, declining values are improving
        let processing_times: Vec<f64> = performance_history.iter().map(|p| p.processing_time_ms).collect();
        self.performance_trends.processing_time_trend = self.calculate_inverse_trend_direction(processing_times);

        self.performance_trends.throughput_trend = 
            self.calculate_trend_direction(performance_history.iter().map(|p| p.throughput_rps).collect());

        // For memory usage, declining values are improving
        let memory_usage: Vec<f64> = performance_history.iter().map(|p| p.memory_usage_mb).collect();
        self.performance_trends.memory_usage_trend = self.calculate_inverse_trend_direction(memory_usage);

        // For error rate, declining values are improving
        let error_rates: Vec<f64> = performance_history.iter().map(|p| p.error_rate).collect();
        self.performance_trends.error_rate_trend = self.calculate_inverse_trend_direction(error_rates);
    }

    /// Analyze issue occurrence trends
    fn analyze_issue_trends(&mut self, issue_history: &[CommonIssueInfo]) {
        if issue_history.is_empty() {
            self.issue_trends = IssueTrends::insufficient_data();
            return;
        }

        // Analyze trends for different issue severities
        let critical_issues: Vec<f64> = issue_history.iter()
            .filter(|i| matches!(i.issue_category, IssueCategory::MissingRequired | IssueCategory::BusinessRuleViolation))
            .map(|i| i.current_rate)
            .collect();

        self.issue_trends.critical_issues_trend = self.calculate_inverse_trend_direction(critical_issues);

        // Overall issue trend
        let overall_rates: Vec<f64> = issue_history.iter().map(|i| i.current_rate).collect();
        self.issue_trends.overall_issue_trend = self.calculate_inverse_trend_direction(overall_rates);
    }

    /// Calculate trend direction from a series of values
    fn calculate_trend_direction(&self, values: Vec<f64>) -> TrendDirection {
        if values.len() < 2 {
            return TrendDirection::InsufficientData;
        }

        let first_half = &values[0..values.len()/2];
        let second_half = &values[values.len()/2..];

        let first_avg = first_half.iter().sum::<f64>() / first_half.len() as f64;
        let second_avg = second_half.iter().sum::<f64>() / second_half.len() as f64;

        let change_ratio = (second_avg - first_avg) / first_avg.max(0.001);

        match change_ratio {
            r if r > 0.05 => TrendDirection::Improving,
            r if r < -0.05 => TrendDirection::Declining,
            _ => TrendDirection::Stable,
        }
    }

    /// Calculate inverse trend direction (where lower values are better)
    fn calculate_inverse_trend_direction(&self, values: Vec<f64>) -> TrendDirection {
        match self.calculate_trend_direction(values) {
            TrendDirection::Improving => TrendDirection::Declining,
            TrendDirection::Declining => TrendDirection::Improving,
            other => other,
        }
    }

    /// Generate recommendations based on trend analysis
    fn generate_trend_recommendations(&mut self) {
        // Add recommendations based on declining quality trends
        if self.quality_trends.overall_quality_trend == TrendDirection::Declining {
            self.trend_recommendations.push(TrendRecommendation {
                recommendation: "Overall data quality is declining. Implement quality monitoring and improvement processes.".to_string(),
                supporting_trends: vec!["Overall quality trend: Declining".to_string()],
                confidence: 0.8,
                expected_impact: "Improved data quality and compliance".to_string(),
                time_frame: "2-4 weeks".to_string(),
                priority: RecommendationPriority::High,
            });
        }

        // Add recommendations based on performance trends
        if self.performance_trends.processing_time_trend == TrendDirection::Declining {
            self.trend_recommendations.push(TrendRecommendation {
                recommendation: "Processing performance is declining. Consider optimization or resource scaling.".to_string(),
                supporting_trends: vec!["Processing time trend: Declining".to_string()],
                confidence: 0.7,
                expected_impact: "Improved processing speed and user experience".to_string(),
                time_frame: "1-2 weeks".to_string(),
                priority: RecommendationPriority::Medium,
            });
        }
    }
}

impl QualityTrends {
    /// Create quality trends with insufficient data
    fn insufficient_data() -> Self {
        Self {
            overall_quality_trend: TrendDirection::InsufficientData,
            completeness_trend: TrendDirection::InsufficientData,
            accuracy_trend: TrendDirection::InsufficientData,
            consistency_trend: TrendDirection::InsufficientData,
            validity_trend: TrendDirection::InsufficientData,
        }
    }
}

impl PerformanceTrends {
    /// Create performance trends with insufficient data
    fn insufficient_data() -> Self {
        Self {
            processing_time_trend: TrendDirection::InsufficientData,
            throughput_trend: TrendDirection::InsufficientData,
            memory_usage_trend: TrendDirection::InsufficientData,
            error_rate_trend: TrendDirection::InsufficientData,
        }
    }
}

impl IssueTrends {
    /// Create issue trends with insufficient data
    fn insufficient_data() -> Self {
        Self {
            critical_issues_trend: TrendDirection::InsufficientData,
            warning_trend: TrendDirection::InsufficientData,
            error_trend: TrendDirection::InsufficientData,
            overall_issue_trend: TrendDirection::InsufficientData,
        }
    }
}

impl Default for QualityTrends {
    fn default() -> Self {
        Self::insufficient_data()
    }
}

impl Default for PerformanceTrends {
    fn default() -> Self {
        Self::insufficient_data()
    }
}

impl Default for IssueTrends {
    fn default() -> Self {
        Self::insufficient_data()
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
