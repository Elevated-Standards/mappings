//! POA&M Quality Assessment Implementation
//! 
//! Main quality checker that orchestrates all quality assessment components

use super::*;
use crate::poam::PoamItem;
use fedramp_core::{Result, Error};
use tracing::{debug, info, warn};
use uuid::Uuid;
use chrono::Utc;
use std::collections::HashMap;

/// Main POA&M quality assessment engine
#[derive(Debug, Clone)]
pub struct PoamQualityChecker {
    /// Completeness analyzer
    completeness_analyzer: CompletenessAnalyzer,
    /// Accuracy validator
    accuracy_validator: AccuracyValidator,
    /// Consistency checker
    consistency_checker: ConsistencyChecker,
    /// Compliance assessor
    compliance_assessor: ComplianceAssessor,
    /// Quality configuration
    quality_config: QualityConfig,
}

impl PoamQualityChecker {
    /// Create a new quality checker with default configuration
    pub fn new() -> Self {
        Self {
            completeness_analyzer: CompletenessAnalyzer::new(),
            accuracy_validator: AccuracyValidator::new(),
            consistency_checker: ConsistencyChecker::new(),
            compliance_assessor: ComplianceAssessor::new(),
            quality_config: QualityConfig::default(),
        }
    }

    /// Create a new quality checker with custom configuration
    pub fn with_config(config: QualityConfig) -> Self {
        Self {
            completeness_analyzer: CompletenessAnalyzer::with_config(&config),
            accuracy_validator: AccuracyValidator::with_config(&config),
            consistency_checker: ConsistencyChecker::with_config(&config),
            compliance_assessor: ComplianceAssessor::with_config(&config),
            quality_config: config,
        }
    }

    /// Perform comprehensive quality assessment on POA&M items
    pub fn assess_quality(&self, poam_items: &[PoamItem]) -> Result<QualityAssessment> {
        info!("Starting comprehensive quality assessment for {} POA&M items", poam_items.len());

        let assessment_id = Uuid::new_v4().to_string();
        let timestamp = Utc::now();

        // Perform individual quality assessments
        let completeness_result = self.completeness_analyzer.analyze(poam_items)?;
        let accuracy_result = self.accuracy_validator.validate(poam_items)?;
        let consistency_result = self.consistency_checker.check(poam_items)?;
        let compliance_result = self.compliance_assessor.assess(poam_items)?;

        // Combine findings from all assessments
        let mut all_findings = Vec::new();
        all_findings.extend(completeness_result.findings);
        all_findings.extend(accuracy_result.findings);
        all_findings.extend(consistency_result.findings);
        all_findings.extend(compliance_result.findings);

        // Calculate overall scores using configured weights
        let weights = &self.quality_config.dimension_weights;
        let overall_score = (completeness_result.score * weights.completeness)
            + (accuracy_result.score * weights.accuracy)
            + (consistency_result.score * weights.consistency)
            + (compliance_result.score * weights.compliance);

        // Generate quality metrics
        let quality_metrics = self.calculate_quality_metrics(poam_items, &all_findings)?;

        // Generate recommendations based on findings
        let recommendations = self.generate_recommendations(&all_findings, &quality_metrics)?;

        // Create configuration summary
        let config_summary = self.create_config_summary();

        let assessment = QualityAssessment {
            assessment_id,
            timestamp,
            overall_score,
            completeness_score: completeness_result.score,
            accuracy_score: accuracy_result.score,
            consistency_score: consistency_result.score,
            compliance_score: compliance_result.score,
            quality_metrics,
            findings: all_findings,
            recommendations,
            config_summary,
        };

        info!(
            "Quality assessment completed: Overall score: {:.2}, Findings: {}, Recommendations: {}",
            assessment.overall_score,
            assessment.findings.len(),
            assessment.recommendations.len()
        );

        Ok(assessment)
    }

    /// Calculate comprehensive quality metrics
    fn calculate_quality_metrics(
        &self,
        poam_items: &[PoamItem],
        findings: &[QualityFinding],
    ) -> Result<QualityMetrics> {
        let total_items = poam_items.len();
        let error_count = findings.iter()
            .filter(|f| matches!(f.severity, QualitySeverity::Critical | QualitySeverity::High))
            .count();
        let warning_count = findings.iter()
            .filter(|f| matches!(f.severity, QualitySeverity::Medium | QualitySeverity::Low))
            .count();

        // Calculate field completeness rates
        let mut field_completeness = HashMap::new();
        for field in &self.quality_config.required_fields {
            let complete_count = poam_items.iter()
                .filter(|item| self.is_field_complete(item, field))
                .count();
            let completeness_rate = if total_items > 0 {
                complete_count as f64 / total_items as f64
            } else {
                0.0
            };
            field_completeness.insert(field.clone(), completeness_rate);
        }

        // Calculate category-specific metrics
        let mut category_metrics = HashMap::new();
        for category in [
            QualityCategory::Completeness,
            QualityCategory::Accuracy,
            QualityCategory::Consistency,
            QualityCategory::Compliance,
        ] {
            let category_findings: Vec<_> = findings.iter()
                .filter(|f| f.category == category)
                .collect();
            
            let affected_items = category_findings.iter()
                .flat_map(|f| &f.affected_items)
                .collect::<std::collections::HashSet<_>>()
                .len();

            let score = self.calculate_category_score(&category, &category_findings, total_items);

            category_metrics.insert(category, CategoryMetrics {
                score,
                finding_count: category_findings.len(),
                affected_items,
                details: HashMap::new(),
            });
        }

        let complete_items = poam_items.iter()
            .filter(|item| self.is_item_complete(item))
            .count();

        Ok(QualityMetrics {
            total_items,
            complete_items,
            incomplete_items: total_items - complete_items,
            error_count,
            warning_count,
            missing_required_fields: self.count_missing_required_fields(poam_items),
            data_quality_issues: findings.len(),
            field_completeness,
            category_metrics,
        })
    }

    /// Generate quality improvement recommendations
    fn generate_recommendations(
        &self,
        findings: &[QualityFinding],
        metrics: &QualityMetrics,
    ) -> Result<Vec<QualityRecommendation>> {
        let mut recommendations = Vec::new();

        // Generate recommendations based on critical findings
        let critical_findings: Vec<_> = findings.iter()
            .filter(|f| f.severity == QualitySeverity::Critical)
            .collect();

        if !critical_findings.is_empty() {
            recommendations.push(QualityRecommendation {
                id: Uuid::new_v4().to_string(),
                priority: QualitySeverity::Critical,
                title: "Address Critical Quality Issues".to_string(),
                description: format!(
                    "There are {} critical quality issues that must be resolved immediately. These issues prevent proper processing and compliance.",
                    critical_findings.len()
                ),
                expected_impact: "Enables proper document processing and compliance validation".to_string(),
                effort_estimate: "High - requires immediate attention".to_string(),
                related_findings: critical_findings.iter().map(|f| f.id.clone()).collect(),
            });
        }

        // Generate completeness recommendations
        if (metrics.complete_items as f64) / (metrics.total_items as f64) < self.quality_config.min_completeness_score {
            recommendations.push(QualityRecommendation {
                id: Uuid::new_v4().to_string(),
                priority: QualitySeverity::High,
                title: "Improve Data Completeness".to_string(),
                description: format!(
                    "Only {}/{} items ({:.1}%) are complete. Focus on populating required fields and improving data collection processes.",
                    metrics.complete_items,
                    metrics.total_items,
                    metrics.complete_items as f64 / metrics.total_items as f64 * 100.0
                ),
                expected_impact: "Improved data quality and compliance scores".to_string(),
                effort_estimate: "Medium - systematic data collection improvement".to_string(),
                related_findings: findings.iter()
                    .filter(|f| f.category == QualityCategory::Completeness)
                    .map(|f| f.id.clone())
                    .collect(),
            });
        }

        // Generate field-specific recommendations
        for (field, completeness) in &metrics.field_completeness {
            if *completeness < 0.8 {
                recommendations.push(QualityRecommendation {
                    id: Uuid::new_v4().to_string(),
                    priority: if *completeness < 0.5 { QualitySeverity::High } else { QualitySeverity::Medium },
                    title: format!("Improve {} Field Completeness", field),
                    description: format!(
                        "The '{}' field is only {:.1}% complete. This field is important for quality and compliance.",
                        field, completeness * 100.0
                    ),
                    expected_impact: "Better data completeness and quality scores".to_string(),
                    effort_estimate: "Low - focused data collection effort".to_string(),
                    related_findings: Vec::new(),
                });
            }
        }

        Ok(recommendations)
    }

    /// Create configuration summary for the assessment
    fn create_config_summary(&self) -> HashMap<String, serde_json::Value> {
        let mut summary = HashMap::new();
        summary.insert("min_overall_score".to_string(), 
                      serde_json::Value::Number(serde_json::Number::from_f64(self.quality_config.min_overall_score).unwrap()));
        summary.insert("strict_mode".to_string(), 
                      serde_json::Value::Bool(self.quality_config.strict_mode));
        summary.insert("required_fields_count".to_string(), 
                      serde_json::Value::Number(serde_json::Number::from(self.quality_config.required_fields.len())));
        summary
    }

    /// Check if a specific field is complete for an item
    fn is_field_complete(&self, item: &PoamItem, field: &str) -> bool {
        match field {
            "uuid" => !item.uuid.is_empty(),
            "title" => !item.title.is_empty(),
            "description" => !item.description.is_empty(),
            "status" => !item.status.is_empty(),
            "scheduled_completion_date" => item.scheduled_completion_date.is_some(),
            "responsible_entity" => item.responsible_entity.as_ref().map_or(false, |s| !s.is_empty()),
            "resources_required" => item.resources_required.as_ref().map_or(false, |s| !s.is_empty()),
            _ => false,
        }
    }

    /// Check if an item is considered complete
    fn is_item_complete(&self, item: &PoamItem) -> bool {
        self.quality_config.required_fields.iter()
            .all(|field| self.is_field_complete(item, field))
    }

    /// Count missing required fields across all items
    fn count_missing_required_fields(&self, poam_items: &[PoamItem]) -> usize {
        poam_items.iter()
            .map(|item| {
                self.quality_config.required_fields.iter()
                    .filter(|field| !self.is_field_complete(item, field))
                    .count()
            })
            .sum()
    }

    /// Calculate score for a specific quality category
    fn calculate_category_score(
        &self,
        category: &QualityCategory,
        findings: &[&QualityFinding],
        total_items: usize,
    ) -> f64 {
        if total_items == 0 {
            return 1.0;
        }

        let penalty = findings.iter()
            .map(|f| f.severity.weight() as f64 * 0.1)
            .sum::<f64>();

        let max_penalty = total_items as f64 * 0.5; // Maximum 50% penalty
        let normalized_penalty = (penalty / max_penalty).min(1.0);

        (1.0 - normalized_penalty).max(0.0)
    }
}

impl Default for PoamQualityChecker {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::poam::PoamItem;

    #[test]
    fn test_quality_checker_creation() {
        let checker = PoamQualityChecker::new();
        assert_eq!(checker.quality_config.min_overall_score, 0.7);
        assert_eq!(checker.quality_config.min_completeness_score, 0.8);
    }

    #[test]
    fn test_quality_checker_with_config() {
        let mut config = QualityConfig::default();
        config.min_overall_score = 0.9;
        config.strict_mode = true;

        let checker = PoamQualityChecker::with_config(config.clone());
        assert_eq!(checker.quality_config.min_overall_score, 0.9);
        assert!(checker.quality_config.strict_mode);
    }

    #[tokio::test]
    async fn test_assess_quality_empty_items() {
        let checker = PoamQualityChecker::new();
        let empty_items: Vec<PoamItem> = Vec::new();

        let result = checker.assess_quality(&empty_items);
        assert!(result.is_ok());

        let assessment = result.unwrap();
        assert_eq!(assessment.overall_score, 0.0);
        assert!(!assessment.findings.is_empty()); // Should have findings about no data
    }

    #[tokio::test]
    async fn test_assess_quality_sample_items() {
        let checker = PoamQualityChecker::new();
        let items = vec![
            PoamItem::sample(),
            PoamItem::new(
                "incomplete-item".to_string(),
                "Incomplete Item".to_string(),
                "This item is missing required fields".to_string(),
                "Open".to_string(),
            ),
        ];

        let result = checker.assess_quality(&items);
        assert!(result.is_ok());

        let assessment = result.unwrap();
        assert!(assessment.overall_score > 0.0);
        assert!(assessment.overall_score < 1.0); // Should not be perfect due to incomplete item
        assert_eq!(assessment.quality_metrics.total_items, 2);
        assert_eq!(assessment.quality_metrics.complete_items, 1); // Only sample item is complete
        assert_eq!(assessment.quality_metrics.incomplete_items, 1);
    }

    #[test]
    fn test_is_field_complete() {
        let checker = PoamQualityChecker::new();
        let complete_item = PoamItem::sample();
        let incomplete_item = PoamItem::new(
            "test".to_string(),
            "Test".to_string(),
            "Test description".to_string(),
            "Open".to_string(),
        );

        // Test complete item
        assert!(checker.is_field_complete(&complete_item, "uuid"));
        assert!(checker.is_field_complete(&complete_item, "title"));
        assert!(checker.is_field_complete(&complete_item, "description"));
        assert!(checker.is_field_complete(&complete_item, "status"));
        assert!(checker.is_field_complete(&complete_item, "scheduled_completion_date"));
        assert!(checker.is_field_complete(&complete_item, "responsible_entity"));

        // Test incomplete item
        assert!(checker.is_field_complete(&incomplete_item, "uuid"));
        assert!(checker.is_field_complete(&incomplete_item, "title"));
        assert!(checker.is_field_complete(&incomplete_item, "description"));
        assert!(checker.is_field_complete(&incomplete_item, "status"));
        assert!(!checker.is_field_complete(&incomplete_item, "scheduled_completion_date"));
        assert!(!checker.is_field_complete(&incomplete_item, "responsible_entity"));
    }

    #[test]
    fn test_is_item_complete() {
        let checker = PoamQualityChecker::new();
        let complete_item = PoamItem::sample();
        let incomplete_item = PoamItem::new(
            "test".to_string(),
            "Test".to_string(),
            "Test description".to_string(),
            "Open".to_string(),
        );

        assert!(checker.is_item_complete(&complete_item));
        assert!(!checker.is_item_complete(&incomplete_item));
    }

    #[test]
    fn test_count_missing_required_fields() {
        let checker = PoamQualityChecker::new();
        let items = vec![
            PoamItem::sample(), // Complete item - 0 missing fields
            PoamItem::new(      // Incomplete item - missing scheduled_completion_date
                "test".to_string(),
                "Test".to_string(),
                "Test description".to_string(),
                "Open".to_string(),
            ),
        ];

        let missing_count = checker.count_missing_required_fields(&items);
        assert_eq!(missing_count, 1); // One missing field from the incomplete item
    }

    #[tokio::test]
    async fn test_quality_assessment_performance() {
        let checker = PoamQualityChecker::new();

        // Create a larger dataset for performance testing
        let mut items = Vec::new();
        for i in 0..100 {
            let mut item = PoamItem::sample();
            item.uuid = format!("item-{}", i);
            item.title = format!("Test Item {}", i);
            items.push(item);
        }

        let start = std::time::Instant::now();
        let result = checker.assess_quality(&items);
        let duration = start.elapsed();

        assert!(result.is_ok());
        assert!(duration.as_millis() < 1000); // Should complete in less than 1 second

        let assessment = result.unwrap();
        assert_eq!(assessment.quality_metrics.total_items, 100);
        assert!(assessment.overall_score > 0.8); // Should have high quality for complete items
    }
}
