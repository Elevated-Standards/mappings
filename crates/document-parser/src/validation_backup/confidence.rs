// Modified: 2025-09-22

//! Confidence scoring system for column mappings
//!
//! This module provides comprehensive confidence scoring for mapping validation,
//! including adaptive learning from user feedback and historical data analysis.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::Duration;
use chrono::{DateTime, Utc};
use crate::fuzzy::FuzzyMatcher;

/// Comprehensive confidence scoring system for column mappings
pub struct ConfidenceScorer {
    /// Scoring configuration
    scoring_config: ScoringConfig,
    /// Factor weights for different confidence factors
    factor_weights: HashMap<ConfidenceFactor, f64>,
    /// Threshold configuration
    threshold_config: ThresholdConfig,
    /// Historical mapping data for learning
    historical_data: HistoricalMappings,
    /// Fuzzy matcher for string similarity
    fuzzy_matcher: FuzzyMatcher,
}

/// Configuration for confidence scoring
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScoringConfig {
    /// Minimum acceptable confidence score
    pub min_acceptable_score: f64,
    /// Threshold for requiring manual review
    pub review_threshold: f64,
    /// Threshold for automatic acceptance
    pub auto_accept_threshold: f64,
    /// Enable adaptive learning from user feedback
    pub adaptive_learning: bool,
    /// Performance target for batch scoring (milliseconds)
    pub performance_target_ms: u64,
}

/// Threshold configuration for confidence-based decisions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThresholdConfig {
    /// High confidence threshold (auto-accept)
    pub high_confidence: f64,
    /// Medium confidence threshold (review recommended)
    pub medium_confidence: f64,
    /// Low confidence threshold (manual verification required)
    pub low_confidence: f64,
    /// Very low confidence threshold (likely incorrect)
    pub very_low_confidence: f64,
}

/// Historical mapping data for adaptive learning
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistoricalMappings {
    /// Successful mappings with their confidence scores
    pub successful_mappings: HashMap<String, Vec<HistoricalMapping>>,
    /// Failed mappings with their confidence scores
    pub failed_mappings: HashMap<String, Vec<HistoricalMapping>>,
    /// User feedback on mapping quality
    pub user_feedback: HashMap<String, UserFeedback>,
    /// Accuracy statistics by confidence range
    pub accuracy_stats: HashMap<String, AccuracyStats>,
}

/// Historical mapping record
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistoricalMapping {
    /// Source column name
    pub source_column: String,
    /// Target field name
    pub target_field: String,
    /// Confidence score at time of mapping
    pub confidence_score: f64,
    /// Whether the mapping was successful
    pub was_successful: bool,
    /// Timestamp of the mapping
    pub timestamp: DateTime<Utc>,
    /// Document type context
    pub document_type: String,
    /// User who validated the mapping
    pub validated_by: Option<String>,
}

/// User feedback on mapping quality
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserFeedback {
    /// User rating (1-5 scale)
    pub rating: u8,
    /// Whether the user accepted the mapping
    pub accepted: bool,
    /// Free-text feedback
    pub comments: Option<String>,
    /// Timestamp of feedback
    pub timestamp: DateTime<Utc>,
    /// User identifier
    pub user_id: String,
}

/// Accuracy statistics for confidence ranges
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccuracyStats {
    /// Total number of mappings in this confidence range
    pub total_mappings: usize,
    /// Number of successful mappings
    pub successful_mappings: usize,
    /// Accuracy percentage
    pub accuracy_percentage: f64,
    /// Average user rating
    pub avg_user_rating: f64,
    /// Last updated timestamp
    pub last_updated: DateTime<Utc>,
}

/// Confidence factors used in scoring
#[derive(Debug, Clone, Hash, Eq, PartialEq, Serialize, Deserialize)]
pub enum ConfidenceFactor {
    /// String similarity between source and target
    StringSimilarity,
    /// Data type compatibility
    DataTypeCompatibility,
    /// Historical success rate
    HistoricalSuccess,
    /// User feedback patterns
    UserFeedbackPatterns,
    /// Context similarity (document type, etc.)
    ContextSimilarity,
    /// Semantic similarity
    SemanticSimilarity,
    /// Position-based hints
    PositionHints,
    /// Cross-validation with other fields
    CrossValidation,
}

/// Mapping confidence result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MappingConfidence {
    /// Overall confidence score (0.0 to 1.0)
    pub overall_score: f64,
    /// Individual factor scores
    pub factor_scores: HashMap<ConfidenceFactor, f64>,
    /// Weighted contributions
    pub weighted_contributions: HashMap<ConfidenceFactor, f64>,
    /// Threshold status
    pub threshold_status: ThresholdStatus,
    /// Recommendation
    pub recommendation: ConfidenceRecommendation,
    /// Risk factors
    pub risk_factors: Vec<RiskFactor>,
    /// Explanation of the confidence calculation
    pub explanation: ConfidenceExplanation,
}

/// Threshold status based on confidence score
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ThresholdStatus {
    /// High confidence - auto-accept (>= 0.9)
    HighConfidence,
    /// Medium confidence - review recommended (0.7-0.9)
    MediumConfidence,
    /// Low confidence - manual verification required (0.4-0.7)
    LowConfidence,
    /// Very low confidence - likely incorrect (< 0.4)
    VeryLowConfidence,
}

/// Confidence-based recommendation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfidenceRecommendation {
    /// Type of recommendation
    pub recommendation_type: RecommendationType,
    /// Recommendation message
    pub message: String,
    /// Priority level
    pub priority: RecommendationPriority,
    /// Estimated effort to resolve
    pub effort_level: EffortLevel,
    /// Suggested actions
    pub suggested_actions: Vec<String>,
}

/// Type of recommendation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RecommendationType {
    /// Accept the mapping automatically
    AutoAccept,
    /// Review the mapping manually
    ManualReview,
    /// Reject the mapping
    Reject,
    /// Request additional information
    RequestInfo,
    /// Use alternative mapping
    UseAlternative,
}

/// Recommendation priority levels
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, PartialOrd)]
pub enum RecommendationPriority {
    /// Low priority
    Low,
    /// Medium priority
    Medium,
    /// High priority
    High,
    /// Critical priority
    Critical,
}

/// Risk factor in mapping confidence
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RiskFactor {
    /// Type of risk
    pub risk_type: RiskType,
    /// Risk severity
    pub severity: RiskSeverity,
    /// Risk description
    pub description: String,
    /// Mitigation suggestions
    pub mitigation: Vec<String>,
}

/// Types of risks in mapping confidence
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RiskType {
    /// Low string similarity
    LowStringSimilarity,
    /// Data type mismatch
    DataTypeMismatch,
    /// Historical failure pattern
    HistoricalFailure,
    /// Ambiguous mapping
    AmbiguousMapping,
    /// Missing context
    MissingContext,
    /// Conflicting evidence
    ConflictingEvidence,
    /// Insufficient data
    InsufficientData,
}

/// Risk severity levels
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, PartialOrd)]
pub enum RiskSeverity {
    /// Low risk
    Low,
    /// Medium risk
    Medium,
    /// High risk
    High,
    /// Critical risk
    Critical,
}

/// Detailed explanation of confidence calculation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfidenceExplanation {
    /// Source column name
    pub source_column: String,
    /// Target field name
    pub target_field: String,
    /// Factor contributions
    pub factor_contributions: HashMap<ConfidenceFactor, FactorContribution>,
    /// Weighted calculation details
    pub weighted_calculation: WeightedConfidenceCalculation,
    /// Applied adjustments
    pub adjustments: Vec<ConfidenceAdjustment>,
    /// Final score calculation
    pub final_score_calculation: String,
}

/// Individual factor contribution to confidence
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FactorContribution {
    /// Raw score from this factor
    pub raw_score: f64,
    /// Weight applied to this factor
    pub weight: f64,
    /// Weighted contribution
    pub weighted_score: f64,
    /// Explanation of how this score was calculated
    pub explanation: String,
}

/// Weighted confidence calculation details
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WeightedConfidenceCalculation {
    /// Sum of all weighted contributions
    pub total_weighted_score: f64,
    /// Sum of all weights
    pub total_weights: f64,
    /// Base confidence score (before adjustments)
    pub base_confidence: f64,
    /// Number of factors considered
    pub factors_considered: usize,
}

/// Confidence score adjustment
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfidenceAdjustment {
    /// Type of adjustment
    pub adjustment_type: AdjustmentType,
    /// Adjustment value (positive or negative)
    pub adjustment_value: f64,
    /// Reason for adjustment
    pub reason: String,
}

/// Types of confidence adjustments
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AdjustmentType {
    /// Bonus for exact matches
    ExactMatchBonus,
    /// Penalty for ambiguous mappings
    AmbiguityPenalty,
    /// Historical success bonus
    HistoricalBonus,
    /// User feedback adjustment
    UserFeedbackAdjustment,
    /// Context-based adjustment
    ContextAdjustment,
    /// Data quality penalty
    DataQualityPenalty,
}

/// Effort level for recommendations
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, PartialOrd)]
pub enum EffortLevel {
    /// Minimal effort (< 1 hour)
    Minimal,
    /// Low effort (1-4 hours)
    Low,
    /// Medium effort (4-8 hours)
    Medium,
    /// High effort (1-2 days)
    High,
    /// Very high effort (> 2 days)
    VeryHigh,
}

impl ConfidenceScorer {
    /// Create a new confidence scorer with default configuration
    pub fn new() -> Self {
        let mut factor_weights = HashMap::new();
        factor_weights.insert(ConfidenceFactor::StringSimilarity, 0.3);
        factor_weights.insert(ConfidenceFactor::DataTypeCompatibility, 0.2);
        factor_weights.insert(ConfidenceFactor::HistoricalSuccess, 0.2);
        factor_weights.insert(ConfidenceFactor::UserFeedbackPatterns, 0.1);
        factor_weights.insert(ConfidenceFactor::ContextSimilarity, 0.1);
        factor_weights.insert(ConfidenceFactor::SemanticSimilarity, 0.05);
        factor_weights.insert(ConfidenceFactor::PositionHints, 0.03);
        factor_weights.insert(ConfidenceFactor::CrossValidation, 0.02);

        Self {
            scoring_config: ScoringConfig::default(),
            factor_weights,
            threshold_config: ThresholdConfig::default(),
            historical_data: HistoricalMappings::default(),
            fuzzy_matcher: FuzzyMatcher::new(),
        }
    }

    /// Calculate confidence score for a mapping
    pub fn calculate_confidence(
        &self,
        source_column: &str,
        target_field: &str,
        context: &str,
    ) -> MappingConfidence {
        let mut factor_scores = HashMap::new();
        let mut weighted_contributions = HashMap::new();

        // Calculate individual factor scores
        let string_similarity = self.fuzzy_matcher.similarity(source_column, target_field);
        factor_scores.insert(ConfidenceFactor::StringSimilarity, string_similarity);

        let data_type_score = self.calculate_data_type_compatibility(source_column, target_field);
        factor_scores.insert(ConfidenceFactor::DataTypeCompatibility, data_type_score);

        let historical_score = self.calculate_historical_success(source_column, target_field);
        factor_scores.insert(ConfidenceFactor::HistoricalSuccess, historical_score);

        // Calculate weighted contributions
        let mut total_weighted_score = 0.0;
        let mut total_weights = 0.0;

        for (factor, score) in &factor_scores {
            if let Some(weight) = self.factor_weights.get(factor) {
                let weighted_score = score * weight;
                weighted_contributions.insert(factor.clone(), weighted_score);
                total_weighted_score += weighted_score;
                total_weights += weight;
            }
        }

        let overall_score = if total_weights > 0.0 {
            total_weighted_score / total_weights
        } else {
            0.0
        };

        let threshold_status = self.determine_threshold_status(overall_score);
        let recommendation = self.generate_recommendation(&threshold_status, overall_score);
        let risk_factors = self.assess_risk_factors(&factor_scores);

        MappingConfidence {
            overall_score,
            factor_scores,
            weighted_contributions,
            threshold_status,
            recommendation,
            risk_factors,
            explanation: ConfidenceExplanation {
                source_column: source_column.to_string(),
                target_field: target_field.to_string(),
                factor_contributions: HashMap::new(),
                weighted_calculation: WeightedConfidenceCalculation {
                    total_weighted_score,
                    total_weights,
                    base_confidence: overall_score,
                    factors_considered: factor_scores.len(),
                },
                adjustments: Vec::new(),
                final_score_calculation: format!("Final score: {:.3}", overall_score),
            },
        }
    }

    /// Calculate data type compatibility score
    fn calculate_data_type_compatibility(&self, _source: &str, _target: &str) -> f64 {
        // Placeholder implementation
        0.8
    }

    /// Calculate historical success score
    fn calculate_historical_success(&self, source: &str, target: &str) -> f64 {
        let key = format!("{}:{}", source, target);
        if let Some(mappings) = self.historical_data.successful_mappings.get(&key) {
            if !mappings.is_empty() {
                let avg_confidence: f64 = mappings.iter()
                    .map(|m| m.confidence_score)
                    .sum::<f64>() / mappings.len() as f64;
                return avg_confidence;
            }
        }
        0.5 // Default neutral score
    }

    /// Determine threshold status based on score
    fn determine_threshold_status(&self, score: f64) -> ThresholdStatus {
        if score >= self.threshold_config.high_confidence {
            ThresholdStatus::HighConfidence
        } else if score >= self.threshold_config.medium_confidence {
            ThresholdStatus::MediumConfidence
        } else if score >= self.threshold_config.low_confidence {
            ThresholdStatus::LowConfidence
        } else {
            ThresholdStatus::VeryLowConfidence
        }
    }

    /// Generate recommendation based on threshold status
    fn generate_recommendation(&self, status: &ThresholdStatus, score: f64) -> ConfidenceRecommendation {
        match status {
            ThresholdStatus::HighConfidence => ConfidenceRecommendation {
                recommendation_type: RecommendationType::AutoAccept,
                message: "High confidence mapping - can be automatically accepted".to_string(),
                priority: RecommendationPriority::Low,
                effort_level: EffortLevel::Minimal,
                suggested_actions: vec!["Accept mapping".to_string()],
            },
            ThresholdStatus::MediumConfidence => ConfidenceRecommendation {
                recommendation_type: RecommendationType::ManualReview,
                message: "Medium confidence mapping - manual review recommended".to_string(),
                priority: RecommendationPriority::Medium,
                effort_level: EffortLevel::Low,
                suggested_actions: vec!["Review mapping manually".to_string(), "Verify data samples".to_string()],
            },
            ThresholdStatus::LowConfidence => ConfidenceRecommendation {
                recommendation_type: RecommendationType::RequestInfo,
                message: "Low confidence mapping - additional information needed".to_string(),
                priority: RecommendationPriority::High,
                effort_level: EffortLevel::Medium,
                suggested_actions: vec!["Gather more context".to_string(), "Check alternative mappings".to_string()],
            },
            ThresholdStatus::VeryLowConfidence => ConfidenceRecommendation {
                recommendation_type: RecommendationType::Reject,
                message: format!("Very low confidence mapping (score: {:.3}) - likely incorrect", score),
                priority: RecommendationPriority::Critical,
                effort_level: EffortLevel::High,
                suggested_actions: vec!["Find alternative mapping".to_string(), "Review field definitions".to_string()],
            },
        }
    }

    /// Assess risk factors for the mapping
    fn assess_risk_factors(&self, factor_scores: &HashMap<ConfidenceFactor, f64>) -> Vec<RiskFactor> {
        let mut risk_factors = Vec::new();

        if let Some(string_sim) = factor_scores.get(&ConfidenceFactor::StringSimilarity) {
            if *string_sim < 0.3 {
                risk_factors.push(RiskFactor {
                    risk_type: RiskType::LowStringSimilarity,
                    severity: RiskSeverity::High,
                    description: "Very low string similarity between source and target".to_string(),
                    mitigation: vec!["Check for abbreviations or synonyms".to_string()],
                });
            }
        }

        risk_factors
    }
}

impl Default for ScoringConfig {
    fn default() -> Self {
        Self {
            min_acceptable_score: 0.6,
            review_threshold: 0.7,
            auto_accept_threshold: 0.9,
            adaptive_learning: true,
            performance_target_ms: 100,
        }
    }
}

impl Default for ThresholdConfig {
    fn default() -> Self {
        Self {
            high_confidence: 0.9,
            medium_confidence: 0.7,
            low_confidence: 0.4,
            very_low_confidence: 0.2,
        }
    }
}

impl Default for HistoricalMappings {
    fn default() -> Self {
        Self {
            successful_mappings: HashMap::new(),
            failed_mappings: HashMap::new(),
            user_feedback: HashMap::new(),
            accuracy_stats: HashMap::new(),
        }
    }
}
