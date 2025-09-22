//! Report generation system for mapping validation
//! Modified: 2025-01-22

pub mod types;
pub mod metrics;
pub mod export;
pub mod generator;

// Re-export main types for backward compatibility
pub use types::*;
pub use metrics::{
    ProcessingMetrics, MemoryUsageMetrics, ThroughputMetrics, CachedReport,
    HistoricalReportData, ReportGenerationMetrics, TrendAnalysis, TimePeriod,
    QualityTrends, PerformanceTrends, IssueTrends,
    HistoricalQualityScore, HistoricalPerformanceData, CommonIssueInfo,
    IssueRateDataPoint, TrendRecommendation,
};
pub use crate::validation::types::TrendDirection;
pub use export::ReportExporter;
pub use generator::MappingReportGenerator;
