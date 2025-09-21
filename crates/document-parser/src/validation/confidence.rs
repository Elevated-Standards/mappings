//! Confidence scoring system for mapping validation

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::Duration;
use super::rules::{ScoringConfig, ThresholdConfig, HistoricalMappings};
use super::types::RecommendationPriority;

/// Confidence scorer for mapping validation
#[derive(Debug, Clone)]
pub struct ConfidenceScorer {
    /// Scoring configuration
    scoring_config: ScoringConfig,
    /// Historical mapping data
    historical_data: HistoricalMappings,
    /// Performance metrics
    performance_metrics: HashMap<String, Duration>,
}

/// Factors that contribute to confidence scoring
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum ConfidenceFactor {
    /// String similarity between source and target
    StringSimilarity,
    /// Semantic similarity (NLP-based)
    SemanticSimilarity,
    /// Historical success rate for this mapping
    HistoricalSuccess,
    /// User feedback and ratings
    UserFeedback,
    /// Data type compatibility
    DataTypeCompatibility,
    /// Column position hints
    ColumnPosition,
    /// Exact match bonus
    ExactMatch,
    /// Pattern matching score
    PatternMatch,
}

/// Comprehensive mapping confidence result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MappingConfidence {
    /// Overall confidence score (0.0 to 1.0)
    pub overall_score: f64,
    /// Individual factor scores
    pub factor_scores: HashMap<ConfidenceFactor, f64>,
    /// Threshold status for decision making
    pub threshold_status: ThresholdStatus,
    /// Recommendations based on confidence
    pub recommendations: Vec<ConfidenceRecommendation>,
    /// Risk factors that lower confidence
    pub risk_factors: Vec<RiskFactor>,
    /// Detailed explanation of the score
    pub explanation: ConfidenceExplanation,
    /// Time taken to calculate confidence
    pub calculation_time: Duration,
}

/// Threshold status for confidence-based decisions
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ThresholdStatus {
    /// High confidence - auto-accept (>= 0.9)
    HighConfidence,
    /// Medium confidence - review recommended (0.7-0.89)
    MediumConfidence,
    /// Low confidence - manual review required (0.5-0.69)
    LowConfidence,
    /// Very low confidence - likely incorrect (< 0.5)
    VeryLowConfidence,
}

/// Recommendation based on confidence analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfidenceRecommendation {
    /// Type of recommendation
    pub recommendation_type: RecommendationType,
    /// Priority level
    pub priority: RecommendationPriority,
    /// Detailed message
    pub message: String,
    /// Confidence threshold that triggered this recommendation
    pub threshold: f64,
}

/// Types of recommendations for confidence handling
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum RecommendationType {
    /// Accept the mapping automatically
    AutoAccept,
    /// Accept with user confirmation
    AcceptWithConfirmation,
    /// Review manually before accepting
    ManualReview,
    /// Reject the mapping
    Reject,
    /// Seek additional validation
    SeekValidation,
    /// Consider alternative mappings
    ConsiderAlternatives,
}

/// Risk factor that affects confidence
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RiskFactor {
    /// Type of risk
    pub risk_type: RiskType,
    /// Severity of the risk
    pub severity: RiskSeverity,
    /// Description of the risk
    pub description: String,
    /// Impact on confidence score
    pub confidence_impact: f64,
    /// Suggested mitigation
    pub mitigation: Option<String>,
}

/// Types of risks that can affect mapping confidence
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum RiskType {
    /// Low string similarity
    LowStringSimilarity,
    /// No historical data
    NoHistoricalData,
    /// Conflicting user feedback
    ConflictingFeedback,
    /// Data type mismatch
    DataTypeMismatch,
    /// Ambiguous column name
    AmbiguousColumnName,
    /// Multiple possible targets
    MultiplePossibleTargets,
    /// Unusual column position
    UnusualColumnPosition,
}

/// Severity levels for risk factors
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
    /// Individual factor contributions
    pub factor_contributions: HashMap<ConfidenceFactor, FactorContribution>,
    /// Weighted calculation details
    pub weighted_calculation: WeightedConfidenceCalculation,
    /// Applied adjustments
    pub adjustments: Vec<ConfidenceAdjustment>,
}

/// Contribution of a single factor to confidence
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FactorContribution {
    /// Raw score from this factor
    pub raw_score: f64,
    /// Weight applied to this factor
    pub weight: f64,
    /// Weighted contribution to final score
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
    pub total_weight: f64,
    /// Base confidence score before adjustments
    pub base_score: f64,
    /// Final score after all adjustments
    pub final_score: f64,
}

/// Adjustment applied to confidence score
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfidenceAdjustment {
    /// Type of adjustment
    pub adjustment_type: AdjustmentType,
    /// Amount of adjustment (positive or negative)
    pub adjustment_amount: f64,
    /// Reason for the adjustment
    pub reason: String,
    /// Whether this adjustment was applied
    pub applied: bool,
}

/// Types of adjustments that can be applied to confidence scores
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum AdjustmentType {
    /// Bonus for exact matches
    ExactMatchBonus,
    /// Penalty for ambiguous matches
    AmbiguityPenalty,
    /// Bonus for historical success
    HistoricalSuccessBonus,
    /// Penalty for historical failures
    HistoricalFailurePenalty,
    /// Bonus for positive user feedback
    UserFeedbackBonus,
    /// Penalty for negative user feedback
    UserFeedbackPenalty,
    /// Penalty for data type mismatches
    DataTypePenalty,
    /// Bonus for semantic similarity
    SemanticSimilarityBonus,
}

impl ConfidenceScorer {
    /// Create a new confidence scorer with default configuration
    pub fn new() -> Self {
        Self {
            scoring_config: ScoringConfig::default(),
            historical_data: HistoricalMappings::default(),
            performance_metrics: HashMap::new(),
        }
    }

    /// Create a new confidence scorer with custom configuration
    pub fn with_config(config: ScoringConfig) -> Self {
        Self {
            scoring_config: config,
            historical_data: HistoricalMappings::default(),
            performance_metrics: HashMap::new(),
        }
    }

    /// Calculate confidence for a mapping
    pub fn calculate_confidence(
        &self,
        source_column: &str,
        target_field: &str,
        context: &MappingContext,
    ) -> Result<MappingConfidence, Box<dyn std::error::Error>> {
        let start_time = std::time::Instant::now();
        
        let mut factor_scores = HashMap::new();
        let mut risk_factors = Vec::new();
        
        // Calculate individual factor scores
        factor_scores.insert(
            ConfidenceFactor::StringSimilarity,
            self.calculate_string_similarity(source_column, target_field)?,
        );
        
        factor_scores.insert(
            ConfidenceFactor::SemanticSimilarity,
            self.calculate_semantic_similarity(source_column, target_field)?,
        );
        
        factor_scores.insert(
            ConfidenceFactor::HistoricalSuccess,
            self.calculate_historical_success(source_column, target_field, &context.document_type)?,
        );
        
        factor_scores.insert(
            ConfidenceFactor::DataTypeCompatibility,
            self.calculate_data_type_compatibility(context)?,
        );
        
        // Calculate weighted overall score
        let overall_score = self.calculate_weighted_score(&factor_scores)?;
        
        // Determine threshold status
        let threshold_status = self.determine_threshold_status(overall_score);
        
        // Generate recommendations
        let recommendations = self.generate_recommendations(overall_score, &factor_scores)?;
        
        // Create explanation
        let explanation = self.create_explanation(source_column, target_field, &factor_scores)?;
        
        let calculation_time = start_time.elapsed();
        
        Ok(MappingConfidence {
            overall_score,
            factor_scores,
            threshold_status,
            recommendations,
            risk_factors,
            explanation,
            calculation_time,
        })
    }

    /// Calculate string similarity between source and target
    fn calculate_string_similarity(&self, source: &str, target: &str) -> Result<f64, Box<dyn std::error::Error>> {
        // Simple Levenshtein distance-based similarity
        let distance = levenshtein_distance(source, target);
        let max_len = source.len().max(target.len()) as f64;
        
        if max_len == 0.0 {
            return Ok(1.0);
        }
        
        Ok(1.0 - (distance as f64 / max_len))
    }

    /// Calculate semantic similarity (placeholder implementation)
    fn calculate_semantic_similarity(&self, _source: &str, _target: &str) -> Result<f64, Box<dyn std::error::Error>> {
        // TODO: Implement actual semantic similarity using NLP models
        Ok(0.5)
    }

    /// Calculate historical success rate
    fn calculate_historical_success(
        &self,
        source: &str,
        target: &str,
        document_type: &str,
    ) -> Result<f64, Box<dyn std::error::Error>> {
        let key = format!("{}:{}", source, target);
        
        if let Some(mappings) = self.historical_data.successful_mappings.get(&key) {
            let relevant_mappings: Vec<_> = mappings
                .iter()
                .filter(|m| m.document_type == document_type)
                .collect();
            
            if !relevant_mappings.is_empty() {
                let success_rate = relevant_mappings.iter()
                    .map(|m| if m.was_successful { 1.0 } else { 0.0 })
                    .sum::<f64>() / relevant_mappings.len() as f64;
                
                return Ok(success_rate);
            }
        }
        
        Ok(0.5) // Default neutral score when no historical data
    }

    /// Calculate data type compatibility
    fn calculate_data_type_compatibility(&self, _context: &MappingContext) -> Result<f64, Box<dyn std::error::Error>> {
        // TODO: Implement data type compatibility checking
        Ok(0.8)
    }

    /// Calculate weighted overall score
    fn calculate_weighted_score(&self, factor_scores: &HashMap<ConfidenceFactor, f64>) -> Result<f64, Box<dyn std::error::Error>> {
        let mut weighted_sum = 0.0;
        let mut total_weight = 0.0;
        
        for (factor, score) in factor_scores {
            let weight = match factor {
                ConfidenceFactor::StringSimilarity => self.scoring_config.string_similarity_weight,
                ConfidenceFactor::SemanticSimilarity => self.scoring_config.semantic_similarity_weight,
                ConfidenceFactor::HistoricalSuccess => self.scoring_config.historical_success_weight,
                ConfidenceFactor::UserFeedback => self.scoring_config.user_feedback_weight,
                ConfidenceFactor::DataTypeCompatibility => self.scoring_config.data_type_weight,
                _ => 0.1, // Default weight for other factors
            };
            
            weighted_sum += score * weight;
            total_weight += weight;
        }
        
        if total_weight > 0.0 {
            Ok(weighted_sum / total_weight)
        } else {
            Ok(0.0)
        }
    }

    /// Determine threshold status based on score
    fn determine_threshold_status(&self, score: f64) -> ThresholdStatus {
        if score >= self.scoring_config.thresholds.high_confidence {
            ThresholdStatus::HighConfidence
        } else if score >= self.scoring_config.thresholds.medium_confidence {
            ThresholdStatus::MediumConfidence
        } else if score >= self.scoring_config.thresholds.low_confidence {
            ThresholdStatus::LowConfidence
        } else {
            ThresholdStatus::VeryLowConfidence
        }
    }

    /// Generate recommendations based on confidence analysis
    fn generate_recommendations(
        &self,
        score: f64,
        _factor_scores: &HashMap<ConfidenceFactor, f64>,
    ) -> Result<Vec<ConfidenceRecommendation>, Box<dyn std::error::Error>> {
        let mut recommendations = Vec::new();
        
        if score >= self.scoring_config.thresholds.high_confidence {
            recommendations.push(ConfidenceRecommendation {
                recommendation_type: RecommendationType::AutoAccept,
                priority: RecommendationPriority::Low,
                message: "High confidence mapping - safe to auto-accept".to_string(),
                threshold: self.scoring_config.thresholds.high_confidence,
            });
        } else if score >= self.scoring_config.thresholds.medium_confidence {
            recommendations.push(ConfidenceRecommendation {
                recommendation_type: RecommendationType::AcceptWithConfirmation,
                priority: RecommendationPriority::Medium,
                message: "Medium confidence mapping - recommend user confirmation".to_string(),
                threshold: self.scoring_config.thresholds.medium_confidence,
            });
        } else {
            recommendations.push(ConfidenceRecommendation {
                recommendation_type: RecommendationType::ManualReview,
                priority: RecommendationPriority::High,
                message: "Low confidence mapping - requires manual review".to_string(),
                threshold: self.scoring_config.thresholds.low_confidence,
            });
        }
        
        Ok(recommendations)
    }

    /// Create detailed explanation of confidence calculation
    fn create_explanation(
        &self,
        source_column: &str,
        target_field: &str,
        factor_scores: &HashMap<ConfidenceFactor, f64>,
    ) -> Result<ConfidenceExplanation, Box<dyn std::error::Error>> {
        let mut factor_contributions = HashMap::new();
        
        for (factor, score) in factor_scores {
            let weight = match factor {
                ConfidenceFactor::StringSimilarity => self.scoring_config.string_similarity_weight,
                ConfidenceFactor::SemanticSimilarity => self.scoring_config.semantic_similarity_weight,
                ConfidenceFactor::HistoricalSuccess => self.scoring_config.historical_success_weight,
                ConfidenceFactor::UserFeedback => self.scoring_config.user_feedback_weight,
                ConfidenceFactor::DataTypeCompatibility => self.scoring_config.data_type_weight,
                _ => 0.1,
            };
            
            factor_contributions.insert(factor.clone(), FactorContribution {
                raw_score: *score,
                weight,
                weighted_score: score * weight,
                explanation: format!("Factor {:?} contributed {:.3} with weight {:.3}", factor, score, weight),
            });
        }
        
        let total_weighted_score: f64 = factor_contributions.values().map(|c| c.weighted_score).sum();
        let total_weight: f64 = factor_contributions.values().map(|c| c.weight).sum();
        let base_score = if total_weight > 0.0 { total_weighted_score / total_weight } else { 0.0 };
        
        Ok(ConfidenceExplanation {
            source_column: source_column.to_string(),
            target_field: target_field.to_string(),
            factor_contributions,
            weighted_calculation: WeightedConfidenceCalculation {
                total_weighted_score,
                total_weight,
                base_score,
                final_score: base_score, // No adjustments in this simple implementation
            },
            adjustments: Vec::new(),
        })
    }
}

/// Context information for mapping confidence calculation
#[derive(Debug, Clone)]
pub struct MappingContext {
    /// Document type being processed
    pub document_type: String,
    /// Available column data for analysis
    pub column_data: Option<Vec<serde_json::Value>>,
    /// Expected data type for target field
    pub expected_data_type: Option<String>,
    /// Column position in the document
    pub column_position: Option<usize>,
}

/// Simple Levenshtein distance calculation
fn levenshtein_distance(s1: &str, s2: &str) -> usize {
    let len1 = s1.len();
    let len2 = s2.len();
    let mut matrix = vec![vec![0; len2 + 1]; len1 + 1];

    for i in 0..=len1 {
        matrix[i][0] = i;
    }
    for j in 0..=len2 {
        matrix[0][j] = j;
    }

    for (i, c1) in s1.chars().enumerate() {
        for (j, c2) in s2.chars().enumerate() {
            let cost = if c1 == c2 { 0 } else { 1 };
            matrix[i + 1][j + 1] = (matrix[i][j + 1] + 1)
                .min(matrix[i + 1][j] + 1)
                .min(matrix[i][j] + cost);
        }
    }

    matrix[len1][len2]
}

impl Default for ConfidenceScorer {
    fn default() -> Self {
        Self::new()
    }
}
