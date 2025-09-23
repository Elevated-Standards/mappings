// Modified: 2025-09-23

//! POA&M validation report generator implementation
//!
//! This module provides the main report generation engine for POA&M validation reports,
//! orchestrating data aggregation, analysis, and report creation.

use super::types::*;
use crate::validation::poam_validator::types::{PoamValidationResult, PoamValidationConfig};
use crate::quality::{QualityAssessment, PoamQualityChecker};
use crate::poam::PoamItem;
use fedramp_core::{Result, Error};
use tracing::{debug, info, warn};
use uuid::Uuid;
use chrono::Utc;
use std::collections::HashMap;
use std::time::{Duration, Instant};

/// Configuration for POA&M report generation
#[derive(Debug, Clone)]
pub struct PoamReportConfig {
    /// Default report format
    pub default_format: PoamReportFormat,
    /// Include data visualizations
    pub include_visualizations: bool,
    /// Maximum report generation time (seconds)
    pub max_generation_time_seconds: u64,
    /// Enable report caching
    pub enable_caching: bool,
    /// Cache expiration time (minutes)
    pub cache_expiration_minutes: u64,
    /// Include detailed analysis
    pub include_detailed_analysis: bool,
    /// Include recommendations
    pub include_recommendations: bool,
    /// Organization name for reports
    pub organization_name: String,
    /// System name for reports
    pub system_name: String,
}

impl Default for PoamReportConfig {
    fn default() -> Self {
        Self {
            default_format: PoamReportFormat::Html,
            include_visualizations: true,
            max_generation_time_seconds: 30,
            enable_caching: true,
            cache_expiration_minutes: 60,
            include_detailed_analysis: true,
            include_recommendations: true,
            organization_name: "Organization".to_string(),
            system_name: "System".to_string(),
        }
    }
}

/// Main POA&M validation report generator
#[derive(Debug)]
pub struct PoamReportGenerator {
    /// Report generation configuration
    config: PoamReportConfig,
    /// Quality assessment engine
    quality_checker: PoamQualityChecker,
    /// Report cache for performance
    report_cache: HashMap<String, CachedReport>,
    /// Generation metrics
    generation_metrics: GenerationMetrics,
}

/// Cached report entry
#[derive(Debug, Clone)]
struct CachedReport {
    report: PoamValidationReport,
    cached_at: chrono::DateTime<Utc>,
    cache_key: String,
}

/// Report generation metrics
#[derive(Debug, Default)]
struct GenerationMetrics {
    total_reports_generated: u64,
    total_generation_time: Duration,
    cache_hits: u64,
    cache_misses: u64,
}

impl PoamReportGenerator {
    /// Create a new POA&M report generator
    pub fn new(config: PoamReportConfig) -> Self {
        Self {
            config,
            quality_checker: PoamQualityChecker::new(),
            report_cache: HashMap::new(),
            generation_metrics: GenerationMetrics::default(),
        }
    }

    /// Generate a comprehensive POA&M validation report
    pub fn generate_report(
        &mut self,
        report_type: PoamReportType,
        poam_items: &[PoamItem],
        validation_results: &[PoamValidationResult],
        document_path: &str,
    ) -> Result<PoamValidationReport> {
        let start_time = Instant::now();
        info!("Generating {} POA&M report for {} items", 
              format!("{:?}", report_type), poam_items.len());

        // Check cache first if enabled
        if self.config.enable_caching {
            let cache_key = self.generate_cache_key(&report_type, poam_items, validation_results);
            if let Some(cached) = self.get_cached_report(&cache_key) {
                let report = cached.report.clone();
                self.generation_metrics.cache_hits += 1;
                info!("Returning cached POA&M report");
                return Ok(report);
            }
            self.generation_metrics.cache_misses += 1;
        }

        let report_id = Uuid::new_v4();
        let generated_at = Utc::now();

        // Create document info
        let document_info = self.create_document_info(document_path, poam_items)?;

        // Create processing summary
        let processing_summary = self.create_processing_summary(
            poam_items, validation_results, start_time
        )?;

        // Create detailed validation results
        let validation_results_summary = self.create_validation_results(validation_results)?;

        // Perform quality assessment
        let quality_assessment = if self.config.include_detailed_analysis {
            self.quality_checker.assess_quality(poam_items)?
        } else {
            self.create_basic_quality_assessment(poam_items)?
        };

        // Create compliance status
        let compliance_status = self.create_compliance_status(
            poam_items, validation_results, &quality_assessment
        )?;

        // Generate recommendations
        let recommendations = if self.config.include_recommendations {
            self.generate_recommendations(
                poam_items, validation_results, &quality_assessment, &compliance_status
            )?
        } else {
            Vec::new()
        };

        // Create report metadata
        let metadata = self.create_report_metadata(start_time)?;

        let report = PoamValidationReport {
            report_id,
            report_type: report_type.clone(),
            format: self.config.default_format.clone(),
            generated_at,
            document_info,
            processing_summary,
            validation_results: validation_results_summary,
            quality_assessment,
            compliance_status,
            recommendations,
            metadata,
        };

        // Update metrics
        let generation_time = start_time.elapsed();
        self.generation_metrics.total_reports_generated += 1;
        self.generation_metrics.total_generation_time += generation_time;

        // Cache the report if caching is enabled
        if self.config.enable_caching {
            self.cache_report(report.clone());
        }

        info!("Generated POA&M report {} in {:?}", report_id, generation_time);
        Ok(report)
    }

    /// Create document information
    fn create_document_info(&self, document_path: &str, poam_items: &[PoamItem]) -> Result<DocumentInfo> {
        let document_name = std::path::Path::new(document_path)
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("Unknown")
            .to_string();

        // Estimate document size based on items
        let estimated_size = poam_items.len() * 1024; // Rough estimate

        Ok(DocumentInfo {
            source_path: document_path.to_string(),
            document_name,
            document_size: estimated_size as u64,
            document_format: "POA&M".to_string(),
            processed_at: Utc::now(),
            checksum: None, // Could be calculated if needed
        })
    }

    /// Create processing summary
    fn create_processing_summary(
        &self,
        poam_items: &[PoamItem],
        validation_results: &[PoamValidationResult],
        start_time: Instant,
    ) -> Result<ProcessingSummary> {
        let total_items = poam_items.len();
        let processing_time = start_time.elapsed();

        // Count items with errors and warnings
        let mut items_with_errors = 0;
        let mut items_with_warnings = 0;

        for result in validation_results {
            if !result.errors.is_empty() {
                items_with_errors += 1;
            }
            if !result.warnings.is_empty() {
                items_with_warnings += 1;
            }
        }

        let successful_items = total_items - items_with_errors;
        let success_rate = if total_items > 0 {
            successful_items as f64 / total_items as f64
        } else {
            1.0
        };

        // Calculate quality and compliance scores
        let quality_score = self.calculate_overall_quality_score(poam_items, validation_results)?;
        let compliance_score = self.calculate_compliance_score(validation_results)?;

        Ok(ProcessingSummary {
            total_items_processed: total_items,
            successful_items,
            items_with_errors,
            items_with_warnings,
            processing_time,
            quality_score,
            compliance_score,
            success_rate,
        })
    }

    /// Calculate overall quality score
    fn calculate_overall_quality_score(
        &self,
        poam_items: &[PoamItem],
        validation_results: &[PoamValidationResult],
    ) -> Result<f64> {
        if poam_items.is_empty() {
            return Ok(1.0);
        }

        // Calculate based on completeness and validation success
        let complete_items = poam_items.iter().filter(|item| item.is_complete()).count();
        let completeness_score = complete_items as f64 / poam_items.len() as f64;

        let validation_success = validation_results.iter()
            .filter(|result| result.errors.is_empty())
            .count();
        let validation_score = if !validation_results.is_empty() {
            validation_success as f64 / validation_results.len() as f64
        } else {
            1.0
        };

        // Weighted average
        Ok((completeness_score * 0.6) + (validation_score * 0.4))
    }

    /// Calculate compliance score
    fn calculate_compliance_score(&self, validation_results: &[PoamValidationResult]) -> Result<f64> {
        if validation_results.is_empty() {
            return Ok(1.0);
        }

        let total_validations = validation_results.len();
        let successful_validations = validation_results.iter()
            .filter(|result| result.errors.is_empty())
            .count();

        Ok(successful_validations as f64 / total_validations as f64)
    }

    /// Generate cache key for report caching
    fn generate_cache_key(
        &self,
        report_type: &PoamReportType,
        poam_items: &[PoamItem],
        validation_results: &[PoamValidationResult],
    ) -> String {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let mut hasher = DefaultHasher::new();
        report_type.hash(&mut hasher);
        poam_items.len().hash(&mut hasher);
        validation_results.len().hash(&mut hasher);
        
        // Hash first few items for uniqueness
        for (i, item) in poam_items.iter().take(5).enumerate() {
            i.hash(&mut hasher);
            item.uuid.hash(&mut hasher);
            item.title.hash(&mut hasher);
        }

        format!("poam_report_{:x}", hasher.finish())
    }

    /// Get cached report if available and not expired
    fn get_cached_report(&self, cache_key: &str) -> Option<&CachedReport> {
        if let Some(cached) = self.report_cache.get(cache_key) {
            let now = Utc::now();
            let expiration = cached.cached_at + 
                chrono::Duration::minutes(self.config.cache_expiration_minutes as i64);
            
            if now < expiration {
                return Some(cached);
            }
        }
        None
    }

    /// Cache a generated report
    fn cache_report(&mut self, report: PoamValidationReport) {
        let cache_key = format!("report_{}", report.report_id);
        let cached = CachedReport {
            report,
            cached_at: Utc::now(),
            cache_key: cache_key.clone(),
        };
        self.report_cache.insert(cache_key, cached);
    }

    /// Create validation results summary
    fn create_validation_results(&self, validation_results: &[PoamValidationResult]) -> Result<ValidationResults> {
        let schema_validation = self.create_schema_validation_result(validation_results)?;
        let business_rule_validation = self.create_business_rule_validation_result(validation_results)?;
        let data_quality_validation = self.create_data_quality_validation_result(validation_results)?;
        let completeness_validation = self.create_completeness_validation_result(validation_results)?;
        let cross_field_validation = self.create_cross_field_validation_result(validation_results)?;

        Ok(ValidationResults {
            schema_validation,
            business_rule_validation,
            data_quality_validation,
            completeness_validation,
            cross_field_validation,
        })
    }

    /// Create schema validation result summary
    fn create_schema_validation_result(&self, validation_results: &[PoamValidationResult]) -> Result<SchemaValidationResult> {
        let mut total_errors = Vec::new();
        let mut total_warnings = Vec::new();

        for result in validation_results {
            total_errors.extend(result.errors.clone());
            total_warnings.extend(result.warnings.clone());
        }

        let is_valid = total_errors.is_empty();
        let compliance_score = if validation_results.is_empty() {
            1.0
        } else {
            let valid_count = validation_results.iter()
                .filter(|r| r.errors.is_empty())
                .count();
            valid_count as f64 / validation_results.len() as f64
        };

        Ok(SchemaValidationResult {
            is_valid,
            errors: total_errors,
            warnings: total_warnings,
            compliance_score,
        })
    }

    /// Create business rule validation result summary
    fn create_business_rule_validation_result(&self, validation_results: &[PoamValidationResult]) -> Result<BusinessRuleValidationResult> {
        let mut rule_results = Vec::new();
        let mut total_rules = 0;
        let mut passed_rules = 0;

        // Aggregate business rule results from validation results
        for result in validation_results {
            for rule_result in &result.business_rule_results {
                total_rules += 1;
                if rule_result.passed {
                    passed_rules += 1;
                }

                // Convert to our report format
                let report_rule = RuleResult {
                    rule_id: rule_result.rule_name.clone(),
                    rule_name: rule_result.rule_name.clone(),
                    passed: rule_result.passed,
                    failed_items: Vec::new(), // Business rules don't track individual failed items
                    severity: "Medium".to_string(), // Default severity
                };
                rule_results.push(report_rule);
            }
        }

        let compliance_score = if total_rules > 0 {
            passed_rules as f64 / total_rules as f64
        } else {
            1.0
        };

        Ok(BusinessRuleValidationResult {
            rules_evaluated: total_rules,
            rules_passed: passed_rules,
            rules_failed: total_rules - passed_rules,
            compliance_score,
            rule_results,
        })
    }

    /// Create data quality validation result summary
    fn create_data_quality_validation_result(&self, validation_results: &[PoamValidationResult]) -> Result<DataQualityValidationResult> {
        // Calculate aggregate quality scores
        let accuracy_score = self.calculate_accuracy_score(validation_results)?;
        let consistency_score = self.calculate_consistency_score(validation_results)?;
        let completeness_score = self.calculate_completeness_score(validation_results)?;

        let overall_quality_score = (accuracy_score + consistency_score + completeness_score) / 3.0;

        Ok(DataQualityValidationResult {
            quality_score: overall_quality_score,
            accuracy_results: AccuracyResults {
                score: accuracy_score,
                issues_found: 0, // Would be calculated from detailed analysis
                field_scores: HashMap::new(),
            },
            consistency_results: ConsistencyResults {
                score: consistency_score,
                issues_found: 0,
                cross_field_results: HashMap::new(),
            },
            completeness_results: CompletenessResults {
                score: completeness_score,
                incomplete_items: 0,
                field_completeness: HashMap::new(),
            },
        })
    }

    /// Calculate accuracy score from validation results
    fn calculate_accuracy_score(&self, validation_results: &[PoamValidationResult]) -> Result<f64> {
        if validation_results.is_empty() {
            return Ok(1.0);
        }

        let total_items = validation_results.len();
        let accurate_items = validation_results.iter()
            .filter(|result| result.errors.is_empty() && result.warnings.len() <= 2)
            .count();

        Ok(accurate_items as f64 / total_items as f64)
    }

    /// Calculate consistency score from validation results
    fn calculate_consistency_score(&self, validation_results: &[PoamValidationResult]) -> Result<f64> {
        // For now, use a simple metric based on validation success
        // In a real implementation, this would analyze cross-field consistency
        if validation_results.is_empty() {
            return Ok(1.0);
        }

        let consistent_items = validation_results.iter()
            .filter(|result| result.errors.is_empty())
            .count();

        Ok(consistent_items as f64 / validation_results.len() as f64)
    }

    /// Calculate completeness score from validation results
    fn calculate_completeness_score(&self, validation_results: &[PoamValidationResult]) -> Result<f64> {
        // Simple completeness metric based on validation success
        if validation_results.is_empty() {
            return Ok(1.0);
        }

        let complete_items = validation_results.iter()
            .filter(|result| {
                // Consider complete if no critical errors
                !result.errors.iter().any(|e| e.severity == crate::validation::types::ValidationSeverity::Critical)
            })
            .count();

        Ok(complete_items as f64 / validation_results.len() as f64)
    }

    /// Create completeness validation result summary
    fn create_completeness_validation_result(&self, validation_results: &[PoamValidationResult]) -> Result<CompletenessValidationResult> {
        let completeness_score = self.calculate_completeness_score(validation_results)?;

        Ok(CompletenessValidationResult {
            score: completeness_score,
            incomplete_items: 0, // Would be calculated from detailed analysis
            field_completeness: HashMap::new(),
        })
    }

    /// Create cross-field validation result summary
    fn create_cross_field_validation_result(&self, validation_results: &[PoamValidationResult]) -> Result<CrossFieldValidationResult> {
        let mut total_rules = 0;
        let mut total_errors = 0;

        // Count cross-field validation issues
        for result in validation_results {
            // In a real implementation, we'd have specific cross-field validation results
            total_rules += 1;
            if !result.errors.is_empty() {
                total_errors += 1;
            }
        }

        let consistency_score = if total_rules > 0 {
            (total_rules - total_errors) as f64 / total_rules as f64
        } else {
            1.0
        };

        Ok(CrossFieldValidationResult {
            rules_evaluated: total_rules,
            errors_found: total_errors,
            consistency_score,
            issues: Vec::new(), // Would be populated with actual cross-field issues
        })
    }

    /// Create basic quality assessment for lightweight reports
    fn create_basic_quality_assessment(&self, poam_items: &[PoamItem]) -> Result<QualityAssessment> {
        use crate::quality::{QualityAssessment, QualityFinding, QualityRecommendation, QualityMetrics, QualitySeverity};

        let assessment_id = Uuid::new_v4().to_string();
        let timestamp = Utc::now();

        // Calculate basic scores
        let complete_items = poam_items.iter().filter(|item| item.is_complete()).count();
        let completeness_score = if !poam_items.is_empty() {
            complete_items as f64 / poam_items.len() as f64
        } else {
            1.0
        };

        let overall_score = completeness_score;

        Ok(QualityAssessment {
            assessment_id,
            timestamp,
            overall_score,
            completeness_score,
            accuracy_score: 0.8, // Default values for basic assessment
            consistency_score: 0.8,
            compliance_score: 0.8,
            quality_metrics: QualityMetrics {
                total_items: poam_items.len(),
                complete_items,
                incomplete_items: poam_items.len() - complete_items,
                error_count: 0,
                warning_count: 0,
                missing_required_fields: 0,
                data_quality_issues: 0,
                field_completeness: HashMap::new(),
                category_metrics: HashMap::new(),
            },
            findings: Vec::new(),
            recommendations: Vec::new(),
            config_summary: HashMap::new(),
        })
    }

    /// Create compliance status assessment
    fn create_compliance_status(
        &self,
        poam_items: &[PoamItem],
        validation_results: &[PoamValidationResult],
        quality_assessment: &QualityAssessment,
    ) -> Result<ComplianceStatus> {
        let overall_score = quality_assessment.compliance_score;

        // FedRAMP compliance assessment
        let fedramp_compliance = ComplianceCategory {
            score: overall_score,
            requirements_met: (overall_score * 10.0) as usize,
            total_requirements: 10,
            issues: Vec::new(),
        };

        // OSCAL compliance assessment
        let oscal_compliance = ComplianceCategory {
            score: self.calculate_compliance_score(validation_results)?,
            requirements_met: validation_results.iter().filter(|r| r.errors.is_empty()).count(),
            total_requirements: validation_results.len(),
            issues: Vec::new(),
        };

        // Risk assessment
        let risk_assessment = RiskAssessment {
            overall_risk_score: 1.0 - overall_score,
            risk_level: if overall_score > 0.8 {
                "Low".to_string()
            } else if overall_score > 0.6 {
                "Moderate".to_string()
            } else {
                "High".to_string()
            },
            identified_risks: Vec::new(),
            mitigation_recommendations: vec![
                "Improve data completeness".to_string(),
                "Enhance validation processes".to_string(),
            ],
        };

        Ok(ComplianceStatus {
            overall_score,
            fedramp_compliance,
            oscal_compliance,
            regulatory_compliance: vec![
                RegulatoryCompliance {
                    regulation: "FISMA".to_string(),
                    score: overall_score,
                    status: "Compliant".to_string(),
                    requirements: vec!["Data integrity".to_string(), "Access controls".to_string()],
                }
            ],
            risk_assessment,
        })
    }

    /// Generate actionable recommendations
    fn generate_recommendations(
        &self,
        poam_items: &[PoamItem],
        validation_results: &[PoamValidationResult],
        quality_assessment: &QualityAssessment,
        compliance_status: &ComplianceStatus,
    ) -> Result<Vec<Recommendation>> {
        let mut recommendations = Vec::new();

        // Data quality recommendations
        if quality_assessment.completeness_score < 0.8 {
            recommendations.push(Recommendation {
                id: "DQ001".to_string(),
                category: RecommendationCategory::DataQuality,
                priority: RecommendationPriority::High,
                title: "Improve Data Completeness".to_string(),
                description: "Several POA&M items are missing required fields".to_string(),
                actions: vec![
                    "Review incomplete items".to_string(),
                    "Implement data validation at entry".to_string(),
                    "Provide training on required fields".to_string(),
                ],
                expected_impact: "Improved compliance and reporting accuracy".to_string(),
                effort_estimate: Some("2-3 weeks".to_string()),
            });
        }

        // Compliance recommendations
        if compliance_status.overall_score < 0.9 {
            recommendations.push(Recommendation {
                id: "COMP001".to_string(),
                category: RecommendationCategory::Compliance,
                priority: RecommendationPriority::Medium,
                title: "Enhance Compliance Monitoring".to_string(),
                description: "Implement automated compliance checking".to_string(),
                actions: vec![
                    "Set up automated validation rules".to_string(),
                    "Create compliance dashboards".to_string(),
                    "Schedule regular compliance reviews".to_string(),
                ],
                expected_impact: "Improved regulatory compliance".to_string(),
                effort_estimate: Some("1-2 weeks".to_string()),
            });
        }

        // Performance recommendations
        if validation_results.len() > 100 {
            recommendations.push(Recommendation {
                id: "PERF001".to_string(),
                category: RecommendationCategory::Performance,
                priority: RecommendationPriority::Low,
                title: "Optimize Processing Performance".to_string(),
                description: "Large datasets may benefit from performance optimization".to_string(),
                actions: vec![
                    "Implement batch processing".to_string(),
                    "Add progress tracking".to_string(),
                    "Consider parallel processing".to_string(),
                ],
                expected_impact: "Faster processing times".to_string(),
                effort_estimate: Some("1 week".to_string()),
            });
        }

        Ok(recommendations)
    }

    /// Create report metadata
    fn create_report_metadata(&self, start_time: Instant) -> Result<ReportMetadata> {
        let generation_time = start_time.elapsed();

        Ok(ReportMetadata {
            tool_version: env!("CARGO_PKG_VERSION").to_string(),
            generation_time,
            processing_config: ProcessingConfig {
                validation_mode: "Standard".to_string(),
                quality_thresholds: [
                    ("completeness".to_string(), 0.8),
                    ("accuracy".to_string(), 0.8),
                    ("consistency".to_string(), 0.8),
                ].iter().cloned().collect(),
                business_rules_enabled: vec!["required_fields".to_string(), "date_validation".to_string()],
                custom_config: HashMap::new(),
            },
            template_version: "1.0.0".to_string(),
            additional_metadata: [
                ("organization".to_string(), self.config.organization_name.clone()),
                ("system".to_string(), self.config.system_name.clone()),
            ].iter().cloned().collect(),
        })
    }

    /// Get generation metrics
    pub fn get_metrics(&self) -> &GenerationMetrics {
        &self.generation_metrics
    }

    /// Clear report cache
    pub fn clear_cache(&mut self) {
        self.report_cache.clear();
    }
}
