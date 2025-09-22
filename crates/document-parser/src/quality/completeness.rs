//! Data Completeness Analysis for POA&M Items
//! 
//! Analyzes POA&M data for completeness across required and recommended fields

use super::*;
use crate::poam::PoamItem;
use fedramp_core::{Result, Error};
use tracing::{debug, info};
use uuid::Uuid;
use std::collections::HashMap;

/// Result of completeness analysis
#[derive(Debug, Clone)]
pub struct CompletenessResult {
    /// Overall completeness score (0.0 to 1.0)
    pub score: f64,
    /// Completeness findings
    pub findings: Vec<QualityFinding>,
    /// Field-level completeness statistics
    pub field_stats: HashMap<String, FieldCompletenessStats>,
    /// Item-level completeness scores
    pub item_scores: Vec<ItemCompletenessScore>,
}

/// Field-level completeness statistics
#[derive(Debug, Clone)]
pub struct FieldCompletenessStats {
    /// Field name
    pub field_name: String,
    /// Total items assessed
    pub total_items: usize,
    /// Items with this field populated
    pub populated_items: usize,
    /// Completeness percentage
    pub completeness_percentage: f64,
    /// Field is required
    pub is_required: bool,
    /// Field is recommended
    pub is_recommended: bool,
}

/// Item-level completeness score
#[derive(Debug, Clone)]
pub struct ItemCompletenessScore {
    /// Item UUID
    pub item_uuid: String,
    /// Overall completeness score for this item
    pub score: f64,
    /// Required fields completeness
    pub required_completeness: f64,
    /// Recommended fields completeness
    pub recommended_completeness: f64,
    /// Missing required fields
    pub missing_required: Vec<String>,
    /// Missing recommended fields
    pub missing_recommended: Vec<String>,
}

/// Completeness analyzer for POA&M data
#[derive(Debug, Clone)]
pub struct CompletenessAnalyzer {
    /// Required fields for completeness assessment
    required_fields: Vec<String>,
    /// Recommended fields for quality assessment
    recommended_fields: Vec<String>,
    /// Minimum required field completeness threshold
    min_required_threshold: f64,
    /// Minimum recommended field completeness threshold
    min_recommended_threshold: f64,
}

impl CompletenessAnalyzer {
    /// Create a new completeness analyzer with default configuration
    pub fn new() -> Self {
        Self {
            required_fields: vec![
                "uuid".to_string(),
                "title".to_string(),
                "description".to_string(),
                "status".to_string(),
                "scheduled_completion_date".to_string(),
            ],
            recommended_fields: vec![
                "responsible_entity".to_string(),
                "resources_required".to_string(),
                "milestones".to_string(),
                "risk_assessment".to_string(),
                "actual_completion_date".to_string(),
                "severity".to_string(),
            ],
            min_required_threshold: 1.0, // 100% for required fields
            min_recommended_threshold: 0.7, // 70% for recommended fields
        }
    }

    /// Create a completeness analyzer with custom configuration
    pub fn with_config(config: &QualityConfig) -> Self {
        Self {
            required_fields: config.required_fields.clone(),
            recommended_fields: config.recommended_fields.clone(),
            min_required_threshold: config.min_completeness_score,
            min_recommended_threshold: 0.7,
        }
    }

    /// Analyze completeness of POA&M items
    pub fn analyze(&self, poam_items: &[PoamItem]) -> Result<CompletenessResult> {
        info!("Analyzing completeness for {} POA&M items", poam_items.len());

        if poam_items.is_empty() {
            return Ok(CompletenessResult {
                score: 0.0,
                findings: vec![self.create_finding(
                    QualitySeverity::Critical,
                    "No POA&M items found for completeness analysis".to_string(),
                    vec![],
                    "Cannot assess completeness without data".to_string(),
                    "Ensure POA&M data is properly loaded and parsed".to_string(),
                )?],
                field_stats: HashMap::new(),
                item_scores: Vec::new(),
            });
        }

        // Calculate field-level statistics
        let field_stats = self.calculate_field_statistics(poam_items)?;

        // Calculate item-level scores
        let item_scores = self.calculate_item_scores(poam_items)?;

        // Generate findings based on completeness analysis
        let findings = self.generate_completeness_findings(&field_stats, &item_scores)?;

        // Calculate overall completeness score
        let overall_score = self.calculate_overall_score(&field_stats, &item_scores);

        debug!(
            "Completeness analysis completed: Score: {:.2}, Findings: {}",
            overall_score,
            findings.len()
        );

        Ok(CompletenessResult {
            score: overall_score,
            findings,
            field_stats,
            item_scores,
        })
    }

    /// Calculate field-level completeness statistics
    fn calculate_field_statistics(&self, poam_items: &[PoamItem]) -> Result<HashMap<String, FieldCompletenessStats>> {
        let mut field_stats = HashMap::new();
        let total_items = poam_items.len();

        // Analyze required fields
        for field in &self.required_fields {
            let populated_count = poam_items.iter()
                .filter(|item| self.is_field_populated(item, field))
                .count();

            let completeness_percentage = if total_items > 0 {
                populated_count as f64 / total_items as f64 * 100.0
            } else {
                0.0
            };

            field_stats.insert(field.clone(), FieldCompletenessStats {
                field_name: field.clone(),
                total_items,
                populated_items: populated_count,
                completeness_percentage,
                is_required: true,
                is_recommended: false,
            });
        }

        // Analyze recommended fields
        for field in &self.recommended_fields {
            let populated_count = poam_items.iter()
                .filter(|item| self.is_field_populated(item, field))
                .count();

            let completeness_percentage = if total_items > 0 {
                populated_count as f64 / total_items as f64 * 100.0
            } else {
                0.0
            };

            field_stats.insert(field.clone(), FieldCompletenessStats {
                field_name: field.clone(),
                total_items,
                populated_items: populated_count,
                completeness_percentage,
                is_required: false,
                is_recommended: true,
            });
        }

        Ok(field_stats)
    }

    /// Calculate item-level completeness scores
    fn calculate_item_scores(&self, poam_items: &[PoamItem]) -> Result<Vec<ItemCompletenessScore>> {
        let mut item_scores = Vec::new();

        for item in poam_items {
            let mut missing_required = Vec::new();
            let mut missing_recommended = Vec::new();

            // Check required fields
            let required_populated = self.required_fields.iter()
                .filter(|field| {
                    let populated = self.is_field_populated(item, field);
                    if !populated {
                        missing_required.push((*field).clone());
                    }
                    populated
                })
                .count();

            let required_completeness = if !self.required_fields.is_empty() {
                required_populated as f64 / self.required_fields.len() as f64
            } else {
                1.0
            };

            // Check recommended fields
            let recommended_populated = self.recommended_fields.iter()
                .filter(|field| {
                    let populated = self.is_field_populated(item, field);
                    if !populated {
                        missing_recommended.push((*field).clone());
                    }
                    populated
                })
                .count();

            let recommended_completeness = if !self.recommended_fields.is_empty() {
                recommended_populated as f64 / self.recommended_fields.len() as f64
            } else {
                1.0
            };

            // Calculate overall item score (weighted towards required fields)
            let score = (required_completeness * 0.8) + (recommended_completeness * 0.2);

            item_scores.push(ItemCompletenessScore {
                item_uuid: item.uuid.clone(),
                score,
                required_completeness,
                recommended_completeness,
                missing_required,
                missing_recommended,
            });
        }

        Ok(item_scores)
    }

    /// Generate completeness findings
    fn generate_completeness_findings(
        &self,
        field_stats: &HashMap<String, FieldCompletenessStats>,
        item_scores: &[ItemCompletenessScore],
    ) -> Result<Vec<QualityFinding>> {
        let mut findings = Vec::new();

        // Check for required fields with low completeness
        for (field_name, stats) in field_stats {
            if stats.is_required && stats.completeness_percentage < self.min_required_threshold * 100.0 {
                let severity = if stats.completeness_percentage < 50.0 {
                    QualitySeverity::Critical
                } else if stats.completeness_percentage < 80.0 {
                    QualitySeverity::High
                } else {
                    QualitySeverity::Medium
                };

                findings.push(self.create_finding(
                    severity,
                    format!(
                        "Required field '{}' is only {:.1}% complete ({}/{} items)",
                        field_name,
                        stats.completeness_percentage,
                        stats.populated_items,
                        stats.total_items
                    ),
                    vec![field_name.clone()],
                    format!(
                        "Missing required field data affects compliance and processing quality. {} items are missing this critical information.",
                        stats.total_items - stats.populated_items
                    ),
                    format!(
                        "Review data collection processes for '{}' field and ensure all items have this required information populated.",
                        field_name
                    ),
                )?);
            }
        }

        // Check for items with very low completeness
        let low_completeness_items: Vec<_> = item_scores.iter()
            .filter(|score| score.score < 0.5)
            .collect();

        if !low_completeness_items.is_empty() {
            findings.push(self.create_finding(
                QualitySeverity::High,
                format!(
                    "{} POA&M items have very low completeness (< 50%)",
                    low_completeness_items.len()
                ),
                low_completeness_items.iter().map(|s| s.item_uuid.clone()).collect(),
                "Items with low completeness may not meet compliance requirements and could affect processing quality".to_string(),
                "Review and complete missing information for items with low completeness scores".to_string(),
            )?);
        }

        // Check overall completeness trends
        let avg_completeness = item_scores.iter()
            .map(|s| s.score)
            .sum::<f64>() / item_scores.len() as f64;

        if avg_completeness < 0.8 {
            findings.push(self.create_finding(
                QualitySeverity::Medium,
                format!(
                    "Overall data completeness is {:.1}%, below recommended threshold of 80%",
                    avg_completeness * 100.0
                ),
                vec!["overall_completeness".to_string()],
                "Low overall completeness affects data quality and may impact compliance assessments".to_string(),
                "Implement systematic data collection improvements to increase overall completeness".to_string(),
            )?);
        }

        Ok(findings)
    }

    /// Calculate overall completeness score
    fn calculate_overall_score(
        &self,
        field_stats: &HashMap<String, FieldCompletenessStats>,
        item_scores: &[ItemCompletenessScore],
    ) -> f64 {
        if item_scores.is_empty() {
            return 0.0;
        }

        // Weight required fields more heavily
        let required_score = self.required_fields.iter()
            .filter_map(|field| field_stats.get(field))
            .map(|stats| stats.completeness_percentage / 100.0)
            .sum::<f64>() / self.required_fields.len().max(1) as f64;

        let recommended_score = self.recommended_fields.iter()
            .filter_map(|field| field_stats.get(field))
            .map(|stats| stats.completeness_percentage / 100.0)
            .sum::<f64>() / self.recommended_fields.len().max(1) as f64;

        // Weighted combination: 80% required, 20% recommended
        (required_score * 0.8) + (recommended_score * 0.2)
    }

    /// Check if a field is populated for a POA&M item
    fn is_field_populated(&self, item: &PoamItem, field: &str) -> bool {
        match field {
            "uuid" => !item.uuid.is_empty(),
            "title" => !item.title.is_empty(),
            "description" => !item.description.is_empty(),
            "status" => !item.status.is_empty(),
            "scheduled_completion_date" => item.scheduled_completion_date.is_some(),
            "actual_completion_date" => item.actual_completion_date.is_some(),
            "responsible_entity" => item.responsible_entity.as_ref().map_or(false, |s| !s.is_empty()),
            "resources_required" => item.resources_required.as_ref().map_or(false, |s| !s.is_empty()),
            "severity" => item.severity.as_ref().map_or(false, |s| !s.is_empty()),
            "milestones" => item.milestones.as_ref().map_or(false, |m| !m.is_empty()),
            "risk_assessment" => item.risk_assessment.as_ref().map_or(false, |r| !r.is_empty()),
            _ => false,
        }
    }

    /// Create a quality finding
    fn create_finding(
        &self,
        severity: QualitySeverity,
        description: String,
        affected_items: Vec<String>,
        impact_assessment: String,
        recommendation: String,
    ) -> Result<QualityFinding> {
        Ok(QualityFinding {
            id: Uuid::new_v4().to_string(),
            severity,
            category: QualityCategory::Completeness,
            description,
            affected_items,
            impact_assessment,
            recommendation,
            location: Some("completeness_analysis".to_string()),
            metadata: HashMap::new(),
        })
    }
}

impl Default for CompletenessAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}
