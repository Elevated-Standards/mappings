//! Gap Prioritization
//!
//! Risk-based prioritization system for gap remediation using multi-criteria
//! decision analysis combining risk, impact, effort, and ROI factors.

use fedramp_core::{Result, Error};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use crate::engine::{Gap, GapSeverity, BusinessImpact, ImpactAssessment};

/// Prioritization engine for gap remediation
#[derive(Debug, Clone)]
pub struct PrioritizationEngine {
    /// Prioritization criteria and weights
    pub criteria: PrioritizationCriteria,
    /// Prioritization algorithms
    pub algorithms: Vec<PrioritizationAlgorithm>,
    /// Configuration settings
    pub config: PrioritizationConfig,
}

/// Prioritization criteria with configurable weights
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrioritizationCriteria {
    /// Risk impact weight (0.0 - 1.0)
    pub risk_weight: f64,
    /// Business impact weight (0.0 - 1.0)
    pub business_impact_weight: f64,
    /// Implementation effort weight (0.0 - 1.0) - inverse relationship
    pub effort_weight: f64,
    /// Return on investment weight (0.0 - 1.0)
    pub roi_weight: f64,
    /// Compliance urgency weight (0.0 - 1.0)
    pub compliance_urgency_weight: f64,
    /// Stakeholder priority weight (0.0 - 1.0)
    pub stakeholder_priority_weight: f64,
}

/// Prioritization algorithm types
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum PrioritizationAlgorithm {
    /// Weighted sum of all criteria
    WeightedSum,
    /// Analytic Hierarchy Process
    AnalyticHierarchyProcess,
    /// TOPSIS (Technique for Order Preference by Similarity to Ideal Solution)
    Topsis,
    /// Simple ranking based on severity
    SeverityBased,
    /// Custom algorithm with user-defined logic
    Custom,
}

/// Configuration for prioritization engine
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrioritizationConfig {
    /// Default algorithm to use
    pub default_algorithm: PrioritizationAlgorithm,
    /// Enable multi-algorithm consensus
    pub use_consensus: bool,
    /// Minimum consensus threshold (0.0 - 1.0)
    pub consensus_threshold: f64,
    /// Priority categories
    pub priority_categories: PriorityCategories,
}

/// Priority categories with thresholds
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PriorityCategories {
    /// Critical priority threshold (0.8 - 1.0)
    pub critical_threshold: f64,
    /// High priority threshold (0.6 - 0.8)
    pub high_threshold: f64,
    /// Medium priority threshold (0.4 - 0.6)
    pub medium_threshold: f64,
    /// Low priority threshold (0.0 - 0.4)
    pub low_threshold: f64,
}

/// Prioritized gap with calculated priority score
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrioritizedGap {
    /// Original gap information
    pub gap: Gap,
    /// Calculated priority score (0.0 - 1.0)
    pub priority_score: f64,
    /// Priority category
    pub priority_category: PriorityCategory,
    /// Priority rank within the analysis
    pub priority_rank: usize,
    /// Detailed scoring breakdown
    pub scoring_breakdown: ScoringBreakdown,
    /// Prioritization metadata
    pub metadata: PrioritizationMetadata,
}

/// Priority categories
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum PriorityCategory {
    Critical,
    High,
    Medium,
    Low,
}

/// Detailed scoring breakdown for transparency
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScoringBreakdown {
    /// Risk score component (0.0 - 1.0)
    pub risk_score: f64,
    /// Business impact score component (0.0 - 1.0)
    pub business_impact_score: f64,
    /// Implementation effort score component (0.0 - 1.0, inverted)
    pub effort_score: f64,
    /// ROI score component (0.0 - 1.0)
    pub roi_score: f64,
    /// Compliance urgency score component (0.0 - 1.0)
    pub compliance_urgency_score: f64,
    /// Stakeholder priority score component (0.0 - 1.0)
    pub stakeholder_priority_score: f64,
    /// Weighted contributions
    pub weighted_contributions: HashMap<String, f64>,
}

/// Prioritization metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrioritizationMetadata {
    /// Algorithm used for prioritization
    pub algorithm_used: PrioritizationAlgorithm,
    /// Criteria weights used
    pub criteria_weights: PrioritizationCriteria,
    /// Confidence in prioritization (0.0 - 1.0)
    pub confidence: f64,
    /// Alternative rankings from other algorithms
    pub alternative_rankings: HashMap<String, f64>,
}

/// Prioritization matrix for visualization and analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrioritizationMatrix {
    /// Matrix dimensions
    pub dimensions: MatrixDimensions,
    /// Gap positions in the matrix
    pub gap_positions: HashMap<String, MatrixPosition>,
    /// Priority quadrants
    pub quadrants: PriorityQuadrants,
}

/// Matrix dimensions for 2D prioritization visualization
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MatrixDimensions {
    /// X-axis criterion (e.g., "impact")
    pub x_axis: String,
    /// Y-axis criterion (e.g., "effort")
    pub y_axis: String,
    /// Matrix size (e.g., 10x10)
    pub size: (usize, usize),
}

/// Position in prioritization matrix
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MatrixPosition {
    pub x: f64,
    pub y: f64,
    pub quadrant: String,
}

/// Priority quadrants for matrix visualization
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PriorityQuadrants {
    /// High impact, low effort (quick wins)
    pub quick_wins: Vec<String>,
    /// High impact, high effort (major projects)
    pub major_projects: Vec<String>,
    /// Low impact, low effort (fill-ins)
    pub fill_ins: Vec<String>,
    /// Low impact, high effort (questionable)
    pub questionable: Vec<String>,
}

impl PrioritizationEngine {
    /// Create a new prioritization engine
    pub fn new() -> Self {
        Self {
            criteria: PrioritizationCriteria::default(),
            algorithms: vec![PrioritizationAlgorithm::WeightedSum],
            config: PrioritizationConfig::default(),
        }
    }

    /// Prioritize gaps using configured algorithm
    pub async fn prioritize_gaps(&self, gaps: &[Gap]) -> Result<Vec<PrioritizedGap>> {
        let mut prioritized_gaps = Vec::new();

        // Calculate priority scores for each gap
        for gap in gaps {
            let priority_score = self.calculate_priority_score(gap).await?;
            let priority_category = self.determine_priority_category(priority_score);
            let scoring_breakdown = self.calculate_scoring_breakdown(gap).await?;
            
            let prioritized_gap = PrioritizedGap {
                gap: gap.clone(),
                priority_score,
                priority_category,
                priority_rank: 0, // Will be set after sorting
                scoring_breakdown,
                metadata: PrioritizationMetadata {
                    algorithm_used: self.config.default_algorithm.clone(),
                    criteria_weights: self.criteria.clone(),
                    confidence: 0.85, // TODO: Calculate actual confidence
                    alternative_rankings: HashMap::new(),
                },
            };
            
            prioritized_gaps.push(prioritized_gap);
        }

        // Sort by priority score (descending)
        prioritized_gaps.sort_by(|a, b| {
            b.priority_score.partial_cmp(&a.priority_score)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        // Assign ranks
        for (index, prioritized_gap) in prioritized_gaps.iter_mut().enumerate() {
            prioritized_gap.priority_rank = index + 1;
        }

        Ok(prioritized_gaps)
    }

    /// Calculate priority score for a single gap
    async fn calculate_priority_score(&self, gap: &Gap) -> Result<f64> {
        match self.config.default_algorithm {
            PrioritizationAlgorithm::WeightedSum => {
                self.calculate_weighted_sum_score(gap).await
            }
            PrioritizationAlgorithm::SeverityBased => {
                self.calculate_severity_based_score(gap).await
            }
            _ => {
                // Default to weighted sum for other algorithms
                self.calculate_weighted_sum_score(gap).await
            }
        }
    }

    /// Calculate weighted sum priority score
    async fn calculate_weighted_sum_score(&self, gap: &Gap) -> Result<f64> {
        let risk_score = self.calculate_risk_score(gap);
        let business_impact_score = self.calculate_business_impact_score(gap);
        let effort_score = self.calculate_effort_score(gap);
        let roi_score = self.calculate_roi_score(gap);
        let compliance_urgency_score = self.calculate_compliance_urgency_score(gap);
        let stakeholder_priority_score = self.calculate_stakeholder_priority_score(gap);

        let weighted_score = 
            (risk_score * self.criteria.risk_weight) +
            (business_impact_score * self.criteria.business_impact_weight) +
            (effort_score * self.criteria.effort_weight) +
            (roi_score * self.criteria.roi_weight) +
            (compliance_urgency_score * self.criteria.compliance_urgency_weight) +
            (stakeholder_priority_score * self.criteria.stakeholder_priority_weight);

        // Normalize by total weights
        let total_weight = self.criteria.risk_weight +
            self.criteria.business_impact_weight +
            self.criteria.effort_weight +
            self.criteria.roi_weight +
            self.criteria.compliance_urgency_weight +
            self.criteria.stakeholder_priority_weight;

        Ok(if total_weight > 0.0 {
            weighted_score / total_weight
        } else {
            0.0
        })
    }

    /// Calculate severity-based priority score
    async fn calculate_severity_based_score(&self, gap: &Gap) -> Result<f64> {
        match gap.severity {
            GapSeverity::Critical => Ok(1.0),
            GapSeverity::High => Ok(0.8),
            GapSeverity::Medium => Ok(0.6),
            GapSeverity::Low => Ok(0.4),
            GapSeverity::Informational => Ok(0.2),
        }
    }

    /// Calculate detailed scoring breakdown
    async fn calculate_scoring_breakdown(&self, gap: &Gap) -> Result<ScoringBreakdown> {
        let risk_score = self.calculate_risk_score(gap);
        let business_impact_score = self.calculate_business_impact_score(gap);
        let effort_score = self.calculate_effort_score(gap);
        let roi_score = self.calculate_roi_score(gap);
        let compliance_urgency_score = self.calculate_compliance_urgency_score(gap);
        let stakeholder_priority_score = self.calculate_stakeholder_priority_score(gap);

        let mut weighted_contributions = HashMap::new();
        weighted_contributions.insert("risk".to_string(), risk_score * self.criteria.risk_weight);
        weighted_contributions.insert("business_impact".to_string(), business_impact_score * self.criteria.business_impact_weight);
        weighted_contributions.insert("effort".to_string(), effort_score * self.criteria.effort_weight);
        weighted_contributions.insert("roi".to_string(), roi_score * self.criteria.roi_weight);
        weighted_contributions.insert("compliance_urgency".to_string(), compliance_urgency_score * self.criteria.compliance_urgency_weight);
        weighted_contributions.insert("stakeholder_priority".to_string(), stakeholder_priority_score * self.criteria.stakeholder_priority_weight);

        Ok(ScoringBreakdown {
            risk_score,
            business_impact_score,
            effort_score,
            roi_score,
            compliance_urgency_score,
            stakeholder_priority_score,
            weighted_contributions,
        })
    }

    /// Calculate risk score component
    fn calculate_risk_score(&self, gap: &Gap) -> f64 {
        match gap.severity {
            GapSeverity::Critical => 1.0,
            GapSeverity::High => 0.8,
            GapSeverity::Medium => 0.6,
            GapSeverity::Low => 0.4,
            GapSeverity::Informational => 0.2,
        }
    }

    /// Calculate business impact score component
    fn calculate_business_impact_score(&self, gap: &Gap) -> f64 {
        match gap.impact_assessment.business_impact {
            BusinessImpact::Critical => 1.0,
            BusinessImpact::High => 0.8,
            BusinessImpact::Medium => 0.6,
            BusinessImpact::Low => 0.4,
            BusinessImpact::Minimal => 0.2,
        }
    }

    /// Calculate effort score component (inverted - lower effort = higher score)
    fn calculate_effort_score(&self, gap: &Gap) -> f64 {
        let effort_hours = gap.remediation_guidance.estimated_effort.hours;
        
        // Normalize effort to 0-1 scale (inverted)
        if effort_hours <= 8 {
            1.0 // Very low effort
        } else if effort_hours <= 40 {
            0.8 // Low effort
        } else if effort_hours <= 160 {
            0.6 // Medium effort
        } else if effort_hours <= 400 {
            0.4 // High effort
        } else {
            0.2 // Very high effort
        }
    }

    /// Calculate ROI score component
    fn calculate_roi_score(&self, gap: &Gap) -> f64 {
        // Simplified ROI calculation based on impact vs effort
        let impact_score = self.calculate_business_impact_score(gap);
        let effort_score = self.calculate_effort_score(gap);
        
        // ROI = (Impact - Effort) normalized to 0-1
        ((impact_score + effort_score) / 2.0).min(1.0).max(0.0)
    }

    /// Calculate compliance urgency score component
    fn calculate_compliance_urgency_score(&self, gap: &Gap) -> f64 {
        // Based on gap type and severity
        let base_score = match gap.gap_type {
            crate::engine::GapType::Missing => 1.0,
            crate::engine::GapType::Partial => 0.7,
            crate::engine::GapType::Outdated => 0.8,
            crate::engine::GapType::EnhancementMissing => 0.6,
            crate::engine::GapType::ParameterMissing => 0.5,
            crate::engine::GapType::Insufficient => 0.7,
        };

        // Adjust by severity
        let severity_multiplier = match gap.severity {
            GapSeverity::Critical => 1.0,
            GapSeverity::High => 0.9,
            GapSeverity::Medium => 0.7,
            GapSeverity::Low => 0.5,
            GapSeverity::Informational => 0.3,
        };

        base_score * severity_multiplier
    }

    /// Calculate stakeholder priority score component
    fn calculate_stakeholder_priority_score(&self, _gap: &Gap) -> f64 {
        // Placeholder - would integrate with stakeholder input system
        0.7
    }

    /// Determine priority category based on score
    fn determine_priority_category(&self, score: f64) -> PriorityCategory {
        if score >= self.config.priority_categories.critical_threshold {
            PriorityCategory::Critical
        } else if score >= self.config.priority_categories.high_threshold {
            PriorityCategory::High
        } else if score >= self.config.priority_categories.medium_threshold {
            PriorityCategory::Medium
        } else {
            PriorityCategory::Low
        }
    }

    /// Generate prioritization matrix for visualization
    pub fn generate_prioritization_matrix(&self, prioritized_gaps: &[PrioritizedGap]) -> Result<PrioritizationMatrix> {
        let mut gap_positions = HashMap::new();
        let mut quadrants = PriorityQuadrants {
            quick_wins: Vec::new(),
            major_projects: Vec::new(),
            fill_ins: Vec::new(),
            questionable: Vec::new(),
        };

        for prioritized_gap in prioritized_gaps {
            let impact_score = prioritized_gap.scoring_breakdown.business_impact_score;
            let effort_score = prioritized_gap.scoring_breakdown.effort_score;
            
            let position = MatrixPosition {
                x: impact_score,
                y: effort_score,
                quadrant: self.determine_quadrant(impact_score, effort_score),
            };

            // Categorize into quadrants
            if impact_score >= 0.7 && effort_score >= 0.7 {
                quadrants.quick_wins.push(prioritized_gap.gap.gap_id.clone());
            } else if impact_score >= 0.7 && effort_score < 0.7 {
                quadrants.major_projects.push(prioritized_gap.gap.gap_id.clone());
            } else if impact_score < 0.7 && effort_score >= 0.7 {
                quadrants.fill_ins.push(prioritized_gap.gap.gap_id.clone());
            } else {
                quadrants.questionable.push(prioritized_gap.gap.gap_id.clone());
            }

            gap_positions.insert(prioritized_gap.gap.gap_id.clone(), position);
        }

        Ok(PrioritizationMatrix {
            dimensions: MatrixDimensions {
                x_axis: "Business Impact".to_string(),
                y_axis: "Implementation Effort (Inverted)".to_string(),
                size: (10, 10),
            },
            gap_positions,
            quadrants,
        })
    }

    /// Determine matrix quadrant for a gap
    fn determine_quadrant(&self, impact: f64, effort: f64) -> String {
        if impact >= 0.7 && effort >= 0.7 {
            "Quick Wins".to_string()
        } else if impact >= 0.7 && effort < 0.7 {
            "Major Projects".to_string()
        } else if impact < 0.7 && effort >= 0.7 {
            "Fill-ins".to_string()
        } else {
            "Questionable".to_string()
        }
    }
}

impl Default for PrioritizationCriteria {
    fn default() -> Self {
        Self {
            risk_weight: 0.25,
            business_impact_weight: 0.25,
            effort_weight: 0.20,
            roi_weight: 0.15,
            compliance_urgency_weight: 0.10,
            stakeholder_priority_weight: 0.05,
        }
    }
}

impl Default for PrioritizationConfig {
    fn default() -> Self {
        Self {
            default_algorithm: PrioritizationAlgorithm::WeightedSum,
            use_consensus: false,
            consensus_threshold: 0.8,
            priority_categories: PriorityCategories::default(),
        }
    }
}

impl Default for PriorityCategories {
    fn default() -> Self {
        Self {
            critical_threshold: 0.8,
            high_threshold: 0.6,
            medium_threshold: 0.4,
            low_threshold: 0.0,
        }
    }
}
