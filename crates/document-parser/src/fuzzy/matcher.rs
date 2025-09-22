// Modified: 2025-09-22

//! Main fuzzy matching implementation
//!
//! This module contains the core FuzzyMatcher struct that combines multiple
//! algorithms and provides the main fuzzy matching functionality.

use std::collections::HashMap;
use std::hash::{Hash, Hasher, DefaultHasher};
use std::time::Instant;
use lru::LruCache;
use tracing::{debug, trace};

use super::types::*;
use super::algorithms::*;
use super::preprocessing::TextPreprocessor;

/// Main fuzzy matcher that combines multiple algorithms
pub struct FuzzyMatcher {
    /// Available fuzzy matching algorithms
    algorithms: Vec<Box<dyn FuzzyAlgorithm>>,
    /// Text preprocessor
    preprocessor: TextPreprocessor,
    /// Configuration
    config: FuzzyMatchConfig,
    /// LRU cache for results
    cache: Option<LruCache<CacheKey, FuzzyMatchResult>>,
    /// Target index for efficient matching
    target_index: Option<TargetIndex>,
    /// Configuration hash for cache invalidation
    config_hash: u64,
}

impl FuzzyMatcher {
    /// Create a new fuzzy matcher with default algorithms and configuration
    pub fn new() -> Self {
        Self::with_config(FuzzyMatchConfig::default())
    }

    /// Create a new fuzzy matcher with custom configuration
    pub fn with_config(config: FuzzyMatchConfig) -> Self {
        let mut algorithms: Vec<Box<dyn FuzzyAlgorithm>> = Vec::new();

        // Add algorithms based on weights in config
        if config.algorithm_weights.contains_key("levenshtein") {
            algorithms.push(Box::new(LevenshteinAlgorithm));
        }
        if config.algorithm_weights.contains_key("jaro_winkler") {
            algorithms.push(Box::new(JaroWinklerAlgorithm));
        }
        if config.algorithm_weights.contains_key("ngram") {
            algorithms.push(Box::new(NgramAlgorithm::default()));
        }
        if config.algorithm_weights.contains_key("soundex") {
            algorithms.push(Box::new(SoundexAlgorithm));
        }

        let cache = if config.enable_caching {
            Some(LruCache::new(std::num::NonZeroUsize::new(config.cache_size).unwrap()))
        } else {
            None
        };

        let config_hash = Self::calculate_config_hash(&config);

        Self {
            algorithms,
            preprocessor: TextPreprocessor::new(),
            config,
            cache,
            target_index: None,
            config_hash,
        }
    }

    /// Create a fuzzy matcher optimized for FedRAMP column matching
    pub fn for_fedramp_columns() -> Self {
        let mut config = FuzzyMatchConfig::default();
        config.min_confidence = 0.6; // Lower threshold for more matches
        config.max_results = 5;

        // Adjust weights for column name matching
        config.algorithm_weights.insert("levenshtein".to_string(), 0.25);
        config.algorithm_weights.insert("jaro_winkler".to_string(), 0.45);
        config.algorithm_weights.insert("ngram".to_string(), 0.25);
        config.algorithm_weights.insert("soundex".to_string(), 0.05);

        Self::with_config(config)
    }
    
    /// Calculate hash of configuration for cache invalidation
    fn calculate_config_hash(config: &FuzzyMatchConfig) -> u64 {
        let mut hasher = DefaultHasher::new();
        config.min_confidence.to_bits().hash(&mut hasher);
        config.max_results.hash(&mut hasher);
        config.enable_caching.hash(&mut hasher);
        config.cache_size.hash(&mut hasher);
        config.include_explanations.hash(&mut hasher);
        
        // Hash algorithm weights
        let mut weights: Vec<_> = config.algorithm_weights.iter().collect();
        weights.sort_by_key(|(k, _)| *k);
        for (name, weight) in weights {
            name.hash(&mut hasher);
            weight.to_bits().hash(&mut hasher);
        }
        
        hasher.finish()
    }
    
    /// Build an index of targets for efficient matching
    pub fn build_index(&mut self, targets: Vec<String>) {
        let mut length_buckets: HashMap<usize, Vec<String>> = HashMap::new();
        let mut first_char_index: HashMap<char, Vec<String>> = HashMap::new();
        
        for target in &targets {
            // Length-based indexing
            let len = target.chars().count();
            length_buckets.entry(len).or_insert_with(Vec::new).push(target.clone());
            
            // First character indexing
            if let Some(first_char) = target.chars().next() {
                let normalized_char = first_char.to_lowercase().next().unwrap_or(first_char);
                first_char_index.entry(normalized_char).or_insert_with(Vec::new).push(target.clone());
            }
        }
        
        self.target_index = Some(TargetIndex {
            length_buckets,
            first_char_index,
            all_targets: targets,
        });
        
        debug!("Built fuzzy match index with {} targets", self.target_index.as_ref().unwrap().all_targets.len());
    }
    
    /// Find the best fuzzy matches for a source string against a list of targets
    pub fn find_matches(&mut self, source: &str, targets: &[String]) -> Vec<FuzzyMatchResult> {
        let start_time = Instant::now();

        debug!("Finding fuzzy matches for '{}' against {} targets", source, targets.len());

        // Early termination for exact matches
        for target in targets {
            if source == target {
                return vec![FuzzyMatchResult {
                    target: target.clone(),
                    confidence: 1.0,
                    algorithm_scores: HashMap::new(),
                    preprocessing_applied: vec!["exact_match".to_string()],
                    exact_match: true,
                    explanation: None,
                }];
            }
        }

        // Use optimized batch processing for large target lists
        if targets.len() > 100 {
            self.find_matches_batch_optimized(source, targets)
        } else {
            self.find_matches_standard(source, targets)
        }
    }

    /// Standard matching for smaller target lists
    fn find_matches_standard(&mut self, source: &str, targets: &[String]) -> Vec<FuzzyMatchResult> {
        let mut results = Vec::new();

        for target in targets {
            if let Some(result) = self.match_single(source, target) {
                if result.confidence >= self.config.min_confidence {
                    results.push(result);
                }
            }
        }

        // Sort by confidence (descending)
        results.sort_by(|a, b| b.confidence.partial_cmp(&a.confidence).unwrap_or(std::cmp::Ordering::Equal));

        // Limit results
        results.truncate(self.config.max_results);

        debug!("Found {} matches above threshold {}", results.len(), self.config.min_confidence);
        results
    }

    /// Optimized batch processing for large target lists
    fn find_matches_batch_optimized(&mut self, source: &str, targets: &[String]) -> Vec<FuzzyMatchResult> {
        let mut results = Vec::new();

        // Preprocess source once
        let (processed_source, source_steps) = self.preprocessor.preprocess(source);

        for target in targets {
            if let Some(result) = self.calculate_match_optimized(source, target, &processed_source, &source_steps) {
                if result.confidence >= self.config.min_confidence {
                    results.push(result);
                }
            }

            // Limit results during processing for efficiency
            if results.len() >= self.config.max_results * 2 {
                break;
            }
        }

        // Sort by confidence (descending)
        results.sort_by(|a, b| b.confidence.partial_cmp(&a.confidence).unwrap_or(std::cmp::Ordering::Equal));

        // Limit to max_results
        results.truncate(self.config.max_results);

        debug!("Found {} fuzzy matches for '{}' in batch mode", results.len(), source);
        results
    }
    
    /// Get candidate targets from index for efficient matching
    fn get_candidate_targets(&self, source: &str, index: &TargetIndex) -> Vec<String> {
        let mut candidates = std::collections::HashSet::new();
        let source_len = source.chars().count();
        
        // Add targets with similar lengths (±2 characters)
        for len in (source_len.saturating_sub(2))..=(source_len + 2) {
            if let Some(targets) = index.length_buckets.get(&len) {
                for target in targets {
                    candidates.insert(target.clone());
                }
            }
        }
        
        // Add targets with same first character
        if let Some(first_char) = source.chars().next() {
            let normalized_char = first_char.to_lowercase().next().unwrap_or(first_char);
            if let Some(targets) = index.first_char_index.get(&normalized_char) {
                for target in targets {
                    candidates.insert(target.clone());
                }
            }
        }
        
        // If we don't have enough candidates, add some random ones
        if candidates.len() < 50 {
            for target in index.all_targets.iter().take(100) {
                candidates.insert(target.clone());
            }
        }
        
        candidates.into_iter().collect()
    }
    
    /// Find the best single match for a source string against a target
    pub fn match_single(&mut self, source: &str, target: &str) -> Option<FuzzyMatchResult> {
        let (processed_source, source_steps) = self.preprocessor.preprocess(source);
        self.calculate_match_optimized(source, target, &processed_source, &source_steps)
    }

    /// Calculate match score between source and target (optimized version)
    fn calculate_match_optimized(&mut self, source: &str, target: &str, processed_source: &str, source_steps: &[String]) -> Option<FuzzyMatchResult> {
        // Check cache
        if let Some(ref mut cache) = self.cache {
            let cache_key = CacheKey {
                source: source.to_string(),
                target: target.to_string(),
                config_hash: self.config_hash,
            };
            
            if let Some(cached_result) = cache.get(&cache_key) {
                trace!("Cache hit for '{}' -> '{}'", source, target);
                return Some(cached_result.clone());
            }
        }
        
        // Preprocess strings
        let (processed_source, source_steps) = self.preprocessor.preprocess(source);
        let (processed_target, target_steps) = self.preprocessor.preprocess(target);
        
        // Check for exact match after preprocessing
        let exact_match = processed_source == processed_target;
        if exact_match {
            let result = FuzzyMatchResult {
                target: target.to_string(),
                confidence: 1.0,
                algorithm_scores: HashMap::new(),
                preprocessing_applied: source_steps,
                exact_match: true,
                explanation: None,
            };
            
            // Cache the result
            if let Some(ref mut cache) = self.cache {
                let cache_key = CacheKey {
                    source: source.to_string(),
                    target: target.to_string(),
                    config_hash: self.config_hash,
                };
                cache.put(cache_key, result.clone());
            }
            
            return Some(result);
        }
        
        // Calculate algorithm scores
        let mut algorithm_scores = HashMap::new();
        let mut algorithm_contributions = HashMap::new();
        let mut total_weighted_score = 0.0;
        let mut total_weight = 0.0;
        let mut algorithms_executed = 0;
        
        for algorithm in &self.algorithms {
            let algo_start = Instant::now();
            
            let (s1, s2) = if algorithm.needs_preprocessing() {
                (processed_source.as_str(), processed_target.as_str())
            } else {
                (source, target)
            };
            
            let raw_score = algorithm.similarity(s1, s2);
            let weight = self.config.algorithm_weights.get(algorithm.name()).copied().unwrap_or(0.0);
            let weighted_score = raw_score * weight;
            
            algorithm_scores.insert(algorithm.name().to_string(), raw_score);
            
            if self.config.include_explanations {
                algorithm_contributions.insert(algorithm.name().to_string(), AlgorithmContribution {
                    raw_score,
                    weight,
                    weighted_score,
                    processing_time_ms: algo_start.elapsed().as_secs_f64() * 1000.0,
                });
            }
            
            total_weighted_score += weighted_score;
            total_weight += weight;
            algorithms_executed += 1;
        }
        
        // Calculate final confidence score
        let confidence = if total_weight > 0.0 {
            total_weighted_score / total_weight
        } else {
            0.0
        };
        
        // Create explanation if requested
        let explanation = if self.config.include_explanations {
            Some(MatchExplanation {
                original_source: source.to_string(),
                original_target: target.to_string(),
                processed_source,
                processed_target,
                algorithm_contributions,
                weighted_calculation: WeightedCalculation {
                    total_weighted_score,
                    total_weight,
                    final_score: confidence,
                },
                performance_metrics: PerformanceMetrics {
                    cache_hit: false,
                    algorithms_executed,
                    early_termination: false,
                },
            })
        } else {
            None
        };
        
        let result = FuzzyMatchResult {
            target: target.to_string(),
            confidence,
            algorithm_scores,
            preprocessing_applied: source_steps,
            exact_match: false,
            explanation,
        };
        
        // Cache the result
        if let Some(ref mut cache) = self.cache {
            let cache_key = CacheKey {
                source: source.to_string(),
                target: target.to_string(),
                config_hash: self.config_hash,
            };
            cache.put(cache_key, result.clone());
        }
        
        Some(result)
    }
    
    /// Clear the cache
    pub fn clear_cache(&mut self) {
        if let Some(ref mut cache) = self.cache {
            cache.clear();
        }
    }

    /// Get cache statistics
    pub fn cache_stats(&self) -> (usize, usize) {
        if let Some(ref cache) = self.cache {
            (cache.len(), cache.cap().get())
        } else {
            (0, 0)
        }
    }

    /// Update the configuration and clear cache if needed
    pub fn update_config(&mut self, new_config: FuzzyMatchConfig) {
        let new_hash = Self::calculate_config_hash(&new_config);
        if new_hash != self.config_hash {
            self.clear_cache();
            self.config_hash = new_hash;
        }

        // Resize cache if needed
        if new_config.enable_caching != self.config.enable_caching ||
           new_config.cache_size != self.config.cache_size {
            if new_config.enable_caching {
                self.cache = Some(LruCache::new(
                    std::num::NonZeroUsize::new(new_config.cache_size).unwrap_or(
                        std::num::NonZeroUsize::new(1000).unwrap()
                    )
                ));
            } else {
                self.cache = None;
            }
        }

        self.config = new_config;
    }
}

impl Default for FuzzyMatcher {
    fn default() -> Self {
        Self::new()
    }
}
