# Build Mapping Confidence Scoring

**Task ID:** gsWvRKGyhKSfbUPD3gnjrC
**Component:** 1.2: Column Mapping Engine
**Status:** Completed
**Priority:** Medium

## Overview

Implement confidence scoring system for column mapping accuracy to help users understand the reliability of automatic mappings and identify areas requiring manual review.

## Objectives

- Develop confidence scoring algorithms for column mappings
- Provide transparency in mapping decisions
- Enable quality-based filtering and prioritization
- Support manual review workflows
- Generate confidence-based recommendations

## Technical Requirements

### Scoring Factors
1. **String Similarity Score**
   - Fuzzy matching confidence from multiple algorithms
   - Weighted combination of different similarity metrics
   - Bonus for exact matches and common patterns

2. **Data Type Compatibility**
   - Type inference confidence from sample data
   - Format validation success rate
   - Consistency across data samples

3. **Context Relevance**
   - Document type appropriateness
   - Field position and proximity to related fields
   - Historical mapping success rates

4. **Validation Success**
   - Required field coverage
   - Constraint satisfaction
   - Cross-field consistency

### Core Functionality
1. **Multi-Factor Scoring**
   - Combine multiple confidence factors
   - Configurable factor weights
   - Non-linear scoring functions

2. **Threshold Management**
   - Configurable confidence thresholds
   - Quality-based filtering
   - Escalation rules for low confidence

3. **Confidence Tracking**
   - Per-field confidence scores
   - Document-level confidence aggregation
   - Historical confidence trends

## Implementation Details

### Data Structures
```rust
pub struct ConfidenceScorer {
    scoring_config: ScoringConfig,
    factor_weights: HashMap<ConfidenceFactor, f64>,
    threshold_config: ThresholdConfig,
    historical_data: HistoricalMappings,
}

pub struct MappingConfidence {
    pub overall_score: f64,
    pub factor_scores: HashMap<ConfidenceFactor, f64>,
    pub threshold_status: ThresholdStatus,
    pub recommendations: Vec<ConfidenceRecommendation>,
    pub risk_factors: Vec<RiskFactor>,
}

pub enum ConfidenceFactor {
    StringSimilarity,
    DataTypeMatch,
    ContextRelevance,
    ValidationSuccess,
    HistoricalAccuracy,
    UserFeedback,
}

pub struct ScoringConfig {
    pub min_acceptable_score: f64,
    pub review_threshold: f64,
    pub auto_accept_threshold: f64,
    pub factor_weights: HashMap<ConfidenceFactor, f64>,
}
```

### Scoring Algorithm
1. **Base Score Calculation**
   - String similarity: 0.0 - 1.0
   - Data type match: 0.0 - 1.0
   - Context relevance: 0.0 - 1.0

2. **Weighted Combination**
   - Apply configurable weights to each factor
   - Normalize to 0.0 - 1.0 range
   - Apply non-linear scaling for edge cases

3. **Confidence Categories**
   - High (0.9+): Auto-accept
   - Medium (0.7-0.9): Review recommended
   - Low (0.5-0.7): Manual verification required
   - Very Low (<0.5): Likely incorrect

### Key Features
- **Adaptive Scoring**: Learn from user feedback and corrections
- **Explainable AI**: Detailed breakdown of confidence factors
- **Risk Assessment**: Identify high-risk mappings for review
- **Batch Scoring**: Efficient scoring of multiple mappings

## Dependencies

- Statistical libraries for scoring algorithms
- Machine learning libraries for adaptive scoring
- Metrics collection for historical analysis

## Testing Requirements

- Unit tests for scoring algorithms
- Integration tests with real mapping scenarios
- Accuracy validation against known good/bad mappings
- Performance tests for large-scale scoring
- User acceptance testing for score interpretability

## Acceptance Criteria

- [x] Implement multi-factor confidence scoring
- [x] Achieve >90% accuracy in identifying good/bad mappings
- [x] Provide explainable confidence breakdowns
- [x] Support configurable scoring thresholds
- [x] Enable adaptive learning from user feedback
- [x] Generate actionable confidence-based recommendations
- [x] Process 1000+ mappings in <500ms
- [x] Pass comprehensive scoring accuracy tests

## Related Tasks

- **Previous:** Create column validation against required fields
- **Next:** Add support for custom mapping overrides
- **Depends on:** Fuzzy string matching implementation
- **Enables:** Quality-based workflow automation

## Notes

- Focus on interpretability and actionability of scores
- Consider domain expertise in scoring algorithm design
- Implement comprehensive logging for score debugging
- Support for A/B testing of scoring algorithms
- Plan for continuous improvement based on user feedback
