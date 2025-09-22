//! Report generator implementation
//! Modified: 2025-01-22

use std::collections::HashMap;
use uuid::Uuid;
use lru::LruCache;
use std::num::NonZeroUsize;
use tracing::{info, warn};

use crate::{Error, Result};
use super::types::*;
use super::metrics::*;
use super::export::ReportExporter;
use crate::validation::types::{ValidationResult, QualityGrade, IssueSeverity, IssueCategory, RecommendationPriority, RecommendationCategory, EffortLevel, TrendDirection};
use crate::validation::overrides::{OverrideResolutionResult, OverrideMetrics};

// Define ValidationResults as a collection of ValidationResult for compatibility
pub struct ValidationResults {
    pub document_name: Option<String>,
    pub document_type: String,
    pub total_rows: usize,
    pub field_results: Vec<FieldValidationResult>,
    pub processing_time: std::time::Duration,
    pub overall_score: f64,
    pub warning_count: usize,
    pub error_count: usize,
}

// Define FieldValidationResult to match what the generator expects
#[derive(Debug, Clone)]
pub struct FieldValidationResult {
    pub field_id: String,
    pub target_field: String,
    pub source_column: Option<String>,
    pub confidence_score: f64,
    pub required: bool,
}

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
    /// Report exporter
    exporter: ReportExporter,
}

impl MappingReportGenerator {
    /// Create a new mapping report generator
    pub fn new() -> Self {
        let config = ReportConfig::default();
        let exporter = ReportExporter::new(config.clone());
        
        Self {
            config: config.clone(),
            template_engine: None,
            report_cache: LruCache::new(NonZeroUsize::new(100).unwrap()),
            historical_data: HistoricalReportData::default(),
            generation_metrics: ReportGenerationMetrics::default(),
            exporter,
        }
    }

    /// Create a new report generator with custom configuration
    pub fn with_config(config: ReportConfig) -> Self {
        let exporter = ReportExporter::new(config.clone());
        let mut generator = Self::new();
        generator.config = config;
        generator.exporter = exporter;

        // Initialize template engine if template directory is specified
        if let Some(ref template_dir) = generator.config.template_directory {
            generator.template_engine = Some(template_dir.clone());
        }

        generator
    }

    /// Get the current configuration
    pub fn config(&self) -> &ReportConfig {
        &self.config
    }

    /// Set the configuration
    pub fn set_config(&mut self, config: ReportConfig) {
        self.exporter = ReportExporter::new(config.clone());
        self.config = config;
    }

    /// Generate a comprehensive mapping report
    pub fn generate_report(
        &mut self,
        report_type: ReportType,
        validation_results: &ValidationResults,
        override_results: Option<&OverrideResolutionResult>,
    ) -> Result<MappingReport> {
        let start_time = std::time::Instant::now();
        
        info!("Generating {} report", format!("{:?}", report_type).to_lowercase());

        let report_id = Uuid::new_v4();

        // Check cache first if enabled
        if self.config.enable_caching {
            let cache_key = self.generate_cache_key(&report_type, validation_results);
            if let Some(cached) = self.get_cached_report(&cache_key) {
                info!("Returning cached report {}", report_id);
                return Ok(cached.report.clone());
            }
        }

        // Create document info
        let document_info = self.create_document_info(validation_results);

        // Create mapping summary
        let mapping_summary = self.create_mapping_summary(validation_results);

        // Create detailed results
        let detailed_results = self.create_detailed_results(validation_results);

        // Create quality metrics
        let quality_metrics = self.create_quality_metrics(validation_results);

        // Create validation summary
        let validation_summary = self.create_validation_summary(validation_results);

        // Create override summary
        let override_summary = self.create_override_summary(override_results);

        // Create processing metrics
        let processing_metrics = self.create_processing_metrics(validation_results);

        // Create trend analysis if historical data is available
        let trend_analysis = if self.config.include_detailed_analysis && !self.historical_data.quality_history.is_empty() {
            Some(self.create_trend_analysis())
        } else {
            None
        };

        // Create recommendations
        let recommendations = if self.config.include_recommendations {
            self.create_recommendations(validation_results, &quality_metrics)
        } else {
            Vec::new()
        };

        let report = MappingReport {
            report_id,
            report_type: report_type.clone(),
            format: self.config.default_format.clone(),
            generated_at: chrono::Utc::now(),
            document_info,
            mapping_summary,
            detailed_results,
            quality_metrics,
            validation_summary,
            override_summary,
            processing_metrics,
            trend_analysis,
            recommendations,
        };

        // Update generation metrics
        let generation_time = start_time.elapsed();
        self.generation_metrics.update_after_generation(
            report_type,
            generation_time,
            0, // Memory usage would be calculated in real implementation
            true, // Success
        );

        // Cache the report if caching is enabled
        if self.config.enable_caching {
            self.cache_report(report.clone());
        }

        info!("Generated report {} in {:?}", report_id, generation_time);
        Ok(report)
    }

    /// Export report to specified format
    pub fn export_report(&self, report: &MappingReport, format: ReportFormat) -> Result<String> {
        self.exporter.export_report(report, format)
    }

    /// Create document information from validation results
    fn create_document_info(&self, validation_results: &ValidationResults) -> DocumentInfo {
        DocumentInfo {
            file_name: validation_results.document_name.clone().unwrap_or_else(|| "unknown.xlsx".to_string()),
            document_type: validation_results.document_type.clone(),
            file_size: 0, // Would be calculated from actual file
            row_count: validation_results.total_rows,
            column_count: validation_results.field_results.len(),
            processed_at: chrono::Utc::now(),
            processing_duration: validation_results.processing_time,
            file_hash: "hash".to_string(), // Would be calculated from actual file
            encoding: "UTF-8".to_string(),
        }
    }

    /// Create mapping summary from validation results
    fn create_mapping_summary(&self, validation_results: &ValidationResults) -> MappingSummary {
        let total_fields = validation_results.field_results.len();
        let mapped_fields = validation_results.field_results.iter()
            .filter(|r| r.source_column.is_some())
            .count();

        let high_confidence_mappings = validation_results.field_results.iter()
            .filter(|r| r.confidence_score >= 0.8)
            .count();

        let review_required = validation_results.field_results.iter()
            .filter(|r| r.confidence_score < 0.7 && r.confidence_score >= 0.4)
            .count();

        let missing_required = validation_results.field_results.iter()
            .filter(|r| r.required && r.source_column.is_none())
            .count();

        let avg_confidence = if !validation_results.field_results.is_empty() {
            validation_results.field_results.iter()
                .map(|r| r.confidence_score)
                .sum::<f64>() / validation_results.field_results.len() as f64
        } else {
            0.0
        };

        MappingSummary {
            total_fields,
            mapped_fields,
            high_confidence_mappings,
            review_required,
            missing_required,
            success_rate: if total_fields > 0 { mapped_fields as f64 / total_fields as f64 } else { 0.0 },
            average_confidence: avg_confidence,
            overrides_applied: 0, // Would be calculated from override results
            conflicts_resolved: 0, // Would be calculated from conflict resolution
            completeness_percentage: validation_results.overall_score * 100.0,
            quality_score: validation_results.overall_score,
        }
    }

    /// Create detailed field mapping results
    fn create_detailed_results(&self, validation_results: &ValidationResults) -> Vec<FieldMappingResult> {
        validation_results.field_results.iter().map(|field_result| {
            FieldMappingResult {
                field_id: field_result.field_id.clone(),
                target_field: field_result.target_field.clone(),
                source_column: field_result.source_column.clone(),
                confidence_score: field_result.confidence_score,
                mapping_successful: field_result.source_column.is_some(),
                required: field_result.required,
                override_applied: false, // Would be determined from override results
                alternatives: Vec::new(), // Would be populated from alternative mappings
                issues: Vec::new(), // Would be populated from validation issues
                data_samples: Vec::new(), // Would be populated from actual data
            }
        }).collect()
    }

    /// Create quality metrics from validation results
    fn create_quality_metrics(&self, validation_results: &ValidationResults) -> QualityMetrics {
        let completeness = validation_results.overall_score;
        let consistency = 0.8; // Would be calculated from actual consistency checks
        let accuracy = 0.85; // Would be calculated from accuracy validation
        let validity = if validation_results.error_count == 0 { 1.0 } else { 0.7 };

        let overall_score = (completeness + consistency + accuracy + validity) / 4.0;

        let quality_grade = match overall_score {
            s if s >= 0.9 => QualityGrade::A,
            s if s >= 0.8 => QualityGrade::B,
            s if s >= 0.7 => QualityGrade::C,
            s if s >= 0.6 => QualityGrade::D,
            _ => QualityGrade::F,
        };

        QualityMetrics {
            completeness_score: completeness,
            mapping_accuracy: accuracy,
            format_compliance: validity,
            business_rule_compliance: 0.9,
            cross_field_consistency: consistency,
            data_freshness: 1.0,
            duplicate_score: 0.95,
            outlier_score: 0.9,
            reference_validation: 0.85,
            overall_quality_score: overall_score,
            quality_grade,
        }
    }

    /// Create validation summary
    fn create_validation_summary(&self, validation_results: &ValidationResults) -> ValidationSummary {
        ValidationSummary {
            total_validations: validation_results.field_results.len(),
            passed_validations: validation_results.field_results.iter()
                .filter(|r| r.source_column.is_some())
                .count(),
            failed_validations: validation_results.field_results.iter()
                .filter(|r| r.source_column.is_none())
                .count(),
            warning_count: validation_results.warning_count,
            error_count: validation_results.error_count,
            overall_score: validation_results.overall_score,
            common_failures: Vec::new(), // Would be populated from failure analysis
            performance_metrics: ValidationPerformanceMetrics {
                avg_validation_time_us: validation_results.processing_time.as_micros() as f64 / validation_results.field_results.len() as f64,
                total_validation_time: validation_results.processing_time,
                slow_validations: Vec::new(),
            },
        }
    }

    /// Create override summary
    fn create_override_summary(&self, override_results: Option<&OverrideResolutionResult>) -> OverrideSummary {
        if let Some(_results) = override_results {
            // For now, use placeholder values since OverrideResolutionResult structure is different
            OverrideSummary {
                total_overrides_evaluated: 1,
                overrides_applied: if _results.override_applied { 1 } else { 0 },
                conflicts_detected: 0,
                conflicts_resolved: 0,
                usage_details: Vec::new(),
                performance_metrics: OverridePerformanceMetrics {
                    avg_resolution_time_us: 100.0,
                    cache_hit_rate: 0.0,
                    total_processing_time: std::time::Duration::from_millis(1),
                },
            }
        } else {
            OverrideSummary {
                total_overrides_evaluated: 0,
                overrides_applied: 0,
                conflicts_detected: 0,
                conflicts_resolved: 0,
                usage_details: Vec::new(),
                performance_metrics: OverridePerformanceMetrics {
                    avg_resolution_time_us: 0.0,
                    cache_hit_rate: 0.0,
                    total_processing_time: std::time::Duration::from_millis(0),
                },
            }
        }
    }

    /// Create processing metrics
    fn create_processing_metrics(&self, validation_results: &ValidationResults) -> ProcessingMetrics {
        let mut metrics = ProcessingMetrics::new();
        metrics.validation_time = validation_results.processing_time;
        metrics.calculate_total_time();
        
        // Calculate throughput
        metrics.throughput_metrics.calculate_throughput(
            validation_results.total_rows,
            validation_results.field_results.len(),
            validation_results.field_results.len(),
            validation_results.processing_time,
        );

        metrics
    }

    /// Create trend analysis from historical data
    fn create_trend_analysis(&self) -> TrendAnalysis {
        let time_period = TimePeriod {
            start_date: self.historical_data.quality_history.first()
                .map(|q| q.timestamp)
                .unwrap_or_else(chrono::Utc::now),
            end_date: chrono::Utc::now(),
            data_points: self.historical_data.quality_history.len(),
        };

        TrendAnalysis {
            time_period,
            quality_trends: QualityTrends {
                overall_quality_trend: self.historical_data.get_recent_quality_trend(30),
                completeness_trend: TrendDirection::Stable,
                accuracy_trend: TrendDirection::Stable,
                consistency_trend: TrendDirection::Stable,
                validity_trend: TrendDirection::Stable,
            },
            performance_trends: PerformanceTrends {
                processing_time_trend: TrendDirection::Stable,
                throughput_trend: TrendDirection::Stable,
                memory_usage_trend: TrendDirection::Stable,
                error_rate_trend: TrendDirection::Stable,
            },
            issue_trends: IssueTrends {
                critical_issues_trend: TrendDirection::Stable,
                warning_trend: TrendDirection::Stable,
                error_trend: TrendDirection::Stable,
                overall_issue_trend: TrendDirection::Stable,
            },
            historical_quality_scores: self.historical_data.quality_history.clone(),
            historical_performance: self.historical_data.performance_history.clone(),
            common_issues: self.historical_data.issue_history.clone(),
            trend_recommendations: Vec::new(),
        }
    }

    /// Create recommendations based on validation results
    fn create_recommendations(&self, validation_results: &ValidationResults, quality_metrics: &QualityMetrics) -> Vec<Recommendation> {
        let mut recommendations = Vec::new();

        // Add recommendation for missing required fields
        let missing_required = validation_results.field_results.iter()
            .filter(|r| r.required && r.source_column.is_none())
            .count();

        if missing_required > 0 {
            recommendations.push(Recommendation {
                priority: RecommendationPriority::High,
                category: RecommendationCategory::DataQuality,
                title: "Address Missing Required Fields".to_string(),
                description: format!("{} required fields are missing and should be added", missing_required),
                suggested_action: "Review the source document and ensure all required fields are present".to_string(),
                effort_level: EffortLevel::Medium,
                expected_impact: "Improved compliance and data completeness".to_string(),
                implementation_steps: vec![
                    "Review missing field list".to_string(),
                    "Update data source to include required fields".to_string(),
                    "Re-validate document".to_string(),
                ],
                related_fields: validation_results.field_results.iter()
                    .filter(|r| r.required && r.source_column.is_none())
                    .map(|r| r.field_id.clone())
                    .collect(),
            });
        }

        // Add recommendation for low quality score
        if quality_metrics.completeness_score < 0.8 {
            recommendations.push(Recommendation {
                priority: RecommendationPriority::Medium,
                category: RecommendationCategory::DataQuality,
                title: "Improve Overall Data Quality".to_string(),
                description: format!("Current quality score is {:.1}% - consider data quality improvements", quality_metrics.completeness_score * 100.0),
                suggested_action: "Implement data quality checks and validation processes".to_string(),
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

    /// Generate cache key for report
    fn generate_cache_key(&self, report_type: &ReportType, validation_results: &ValidationResults) -> String {
        format!("{}_{}_{}_{}", 
            format!("{:?}", report_type).to_lowercase(),
            validation_results.document_type,
            validation_results.field_results.len(),
            validation_results.overall_score as u32
        )
    }

    /// Get cached report if available and not expired
    fn get_cached_report(&mut self, cache_key: &str) -> Option<&CachedReport> {
        if let Some(cached) = self.report_cache.get(cache_key) {
            if cached.expires_at > chrono::Utc::now() {
                return Some(cached);
            }
        }
        None
    }

    /// Cache a generated report
    fn cache_report(&mut self, report: MappingReport) {
        let report_id = report.report_id;
        let report_type_discriminant = std::mem::discriminant(&report.report_type);
        let cache_key = format!("{:?}_{}", report_type_discriminant, report_id);
        let cached_report = CachedReport {
            report,
            cached_at: chrono::Utc::now(),
            expires_at: chrono::Utc::now() + chrono::Duration::minutes(self.config.cache_expiration_minutes as i64),
            access_count: 0,
            size_bytes: 0, // Would be calculated
        };
        self.report_cache.put(cache_key, cached_report);
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

    /// Add historical quality score
    pub fn add_historical_quality_score(&mut self, score: HistoricalQualityScore) {
        self.historical_data.add_quality_score(score);
    }

    /// Add historical performance data
    pub fn add_historical_performance_data(&mut self, data: HistoricalPerformanceData) {
        self.historical_data.add_performance_data(data);
    }
}

impl Default for MappingReportGenerator {
    fn default() -> Self {
        Self::new()
    }
}
