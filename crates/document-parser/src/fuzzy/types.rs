// Modified: 2025-09-22

//! Fuzzy matching type definitions and data structures
//!
//! This module contains all the core types used in fuzzy string matching,
//! including match results, explanations, configurations, and performance metrics.

use std::collections::HashMap;
use std::hash::{Hash, Hasher};

/// Result of a fuzzy matching operation
#[derive(Debug, Clone, PartialEq)]
pub struct FuzzyMatchResult {
    /// The target string that was matched
    pub target: String,
    /// Overall confidence score (0.0 to 1.0)
    pub confidence: f64,
    /// Scores from individual algorithms
    pub algorithm_scores: HashMap<String, f64>,
    /// Preprocessing steps that were applied
    pub preprocessing_applied: Vec<String>,
    /// Whether this was an exact match after preprocessing
    pub exact_match: bool,
    /// Detailed explanation of the matching process
    pub explanation: Option<MatchExplanation>,
}

/// Detailed explanation of how a match was computed
#[derive(Debug, Clone, PartialEq)]
pub struct MatchExplanation {
    /// Original source and target strings
    pub original_source: String,
    pub original_target: String,
    /// Preprocessed source and target strings
    pub processed_source: String,
    pub processed_target: String,
    /// Algorithm contributions to final score
    pub algorithm_contributions: HashMap<String, AlgorithmContribution>,
    /// Final weighted calculation
    pub weighted_calculation: WeightedCalculation,
    /// Performance metrics
    pub performance_metrics: PerformanceMetrics,
}

/// Contribution of a single algorithm to the final score
#[derive(Debug, Clone, PartialEq)]
pub struct AlgorithmContribution {
    /// Raw similarity score from the algorithm
    pub raw_score: f64,
    /// Weight applied to this algorithm
    pub weight: f64,
    /// Weighted contribution to final score
    pub weighted_score: f64,
    /// Processing time for this algorithm
    pub processing_time_ms: f64,
}

/// Final weighted calculation details
#[derive(Debug, Clone, PartialEq)]
pub struct WeightedCalculation {
    /// Sum of all weighted contributions
    pub total_weighted_score: f64,
    /// Sum of all weights
    pub total_weight: f64,
    /// Final normalized score
    pub final_score: f64,
}

/// Performance metrics for a fuzzy match operation
#[derive(Debug, Clone, PartialEq)]
pub struct PerformanceMetrics {
    /// Whether the result was retrieved from cache
    pub cache_hit: bool,
    /// Number of algorithms that were executed
    pub algorithms_executed: usize,
    /// Whether early termination was used
    pub early_termination: bool,
}

/// Configuration for fuzzy matching behavior
#[derive(Debug, Clone)]
pub struct FuzzyMatchConfig {
    /// Minimum confidence threshold for matches
    pub min_confidence: f64,
    /// Maximum number of results to return
    pub max_results: usize,
    /// Weights for different algorithms
    pub algorithm_weights: HashMap<String, f64>,
    /// Whether to enable caching
    pub enable_caching: bool,
    /// Maximum cache size
    pub cache_size: usize,
    /// Whether to include detailed explanations
    pub include_explanations: bool,
    /// Enable/disable specific preprocessing steps
    pub preprocessing_enabled: bool,
}

impl Default for FuzzyMatchConfig {
    fn default() -> Self {
        let mut algorithm_weights = HashMap::new();
        algorithm_weights.insert("levenshtein".to_string(), 0.3);
        algorithm_weights.insert("jaro_winkler".to_string(), 0.3);
        algorithm_weights.insert("ngram".to_string(), 0.25);
        algorithm_weights.insert("soundex".to_string(), 0.15);

        Self {
            min_confidence: 0.6,
            max_results: 10,
            algorithm_weights,
            enable_caching: true,
            cache_size: 1000,
            include_explanations: false,
            preprocessing_enabled: true,
        }
    }
}

/// Trait for fuzzy matching algorithms
pub trait FuzzyAlgorithm: Send + Sync {
    /// Name of the algorithm
    fn name(&self) -> &'static str;
    
    /// Calculate similarity between two strings (0.0 to 1.0)
    fn similarity(&self, s1: &str, s2: &str) -> f64;
    
    /// Whether this algorithm benefits from preprocessing
    fn needs_preprocessing(&self) -> bool {
        true
    }
}

/// Cache key for fuzzy matching results
#[derive(Debug, Clone, Eq)]
pub(crate) struct CacheKey {
    pub source: String,
    pub target: String,
    pub config_hash: u64,
}

impl PartialEq for CacheKey {
    fn eq(&self, other: &Self) -> bool {
        self.source == other.source && self.target == other.target && self.config_hash == other.config_hash
    }
}

impl Hash for CacheKey {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.source.hash(state);
        self.target.hash(state);
        self.config_hash.hash(state);
    }
}

/// Target index for efficient fuzzy matching
#[derive(Debug, Clone)]
pub(crate) struct TargetIndex {
    /// Length-based index for quick filtering
    pub length_buckets: HashMap<usize, Vec<String>>,
    /// First character index
    pub first_char_index: HashMap<char, Vec<String>>,
    /// All targets for fallback
    pub all_targets: Vec<String>,
}
