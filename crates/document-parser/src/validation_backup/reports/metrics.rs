//! Processing and performance metrics for reports
//! Modified: 2025-01-22

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::Duration;

use crate::validation_backup::types::*;

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

/// Validation summary statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationSummary {
    /// Total number of validations performed
    pub total_validations: usize,
    /// Number of validations that passed
    pub passed_validations: usize,
    /// Number of validations that failed
    pub failed_validations: usize,
    /// Number of warnings generated
    pub warning_count: usize,
    /// Number of errors encountered
    pub error_count: usize,
    /// Overall validation score (0.0-1.0)
    pub overall_score: f64,
    /// Most common validation failures
    pub common_failures: Vec<String>,
    /// Performance metrics for validation
    pub performance_metrics: ValidationPerformanceMetrics,
}

/// Performance metrics specific to validation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationPerformanceMetrics {
    /// Average validation time per field (microseconds)
    pub avg_validation_time_us: f64,
    /// Total time spent on validation
    pub total_validation_time: Duration,
    /// Slowest validation operations
    pub slow_validations: Vec<String>,
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
    pub popular_report_types: HashMap<super::types::ReportType, u64>,
    /// Error rate during generation
    pub error_rate: f64,
}

/// Cached report information
#[derive(Debug, Clone)]
pub struct CachedReport {
    /// The cached report
    pub report: super::types::MappingReport,
    /// When the report was cached
    pub cached_at: chrono::DateTime<chrono::Utc>,
    /// Cache expiration time
    pub expires_at: chrono::DateTime<chrono::Utc>,
    /// Number of times this report was accessed
    pub access_count: u64,
    /// Size of the cached report in bytes
    pub size_bytes: u64,
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

impl ValidationSummary {
    /// Create new validation summary
    pub fn new() -> Self {
        Self {
            total_validations: 0,
            passed_validations: 0,
            failed_validations: 0,
            warning_count: 0,
            error_count: 0,
            overall_score: 0.0,
            common_failures: Vec::new(),
            performance_metrics: ValidationPerformanceMetrics::new(),
        }
    }

    /// Calculate overall validation score
    pub fn calculate_score(&mut self) {
        if self.total_validations > 0 {
            self.overall_score = self.passed_validations as f64 / self.total_validations as f64;
        }
    }

    /// Get validation success rate
    pub fn success_rate(&self) -> f64 {
        if self.total_validations > 0 {
            self.passed_validations as f64 / self.total_validations as f64
        } else {
            0.0
        }
    }

    /// Get validation failure rate
    pub fn failure_rate(&self) -> f64 {
        if self.total_validations > 0 {
            self.failed_validations as f64 / self.total_validations as f64
        } else {
            0.0
        }
    }
}

impl ValidationPerformanceMetrics {
    /// Create new validation performance metrics
    pub fn new() -> Self {
        Self {
            avg_validation_time_us: 0.0,
            total_validation_time: Duration::from_millis(0),
            slow_validations: Vec::new(),
        }
    }

    /// Calculate average validation time
    pub fn calculate_avg_time(&mut self, total_validations: usize) {
        if total_validations > 0 {
            self.avg_validation_time_us = 
                self.total_validation_time.as_micros() as f64 / total_validations as f64;
        }
    }

    /// Check if validation performance is acceptable
    pub fn is_performance_acceptable(&self) -> bool {
        // Consider performance acceptable if average validation time is under 1ms
        self.avg_validation_time_us < 1000.0
    }
}

impl ReportGenerationMetrics {
    /// Update metrics after generating a report
    pub fn update_after_generation(
        &mut self,
        report_type: super::types::ReportType,
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

impl Default for ValidationSummary {
    fn default() -> Self {
        Self::new()
    }
}

impl Default for ValidationPerformanceMetrics {
    fn default() -> Self {
        Self::new()
    }
}
