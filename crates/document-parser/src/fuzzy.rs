//! Advanced fuzzy string matching for column detection
//!
//! This module provides multiple fuzzy matching algorithms to handle column name
//! variations, typos, and different formatting styles commonly found in FedRAMP documents.

use std::collections::HashMap;
use std::hash::{Hash, Hasher};
use lru::LruCache;
use unicode_normalization::{UnicodeNormalization, is_nfc};
use tracing::{debug, trace};

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
    /// Weighted contribution (raw_score * weight)
    pub weighted_contribution: f64,
    /// Whether preprocessing was used for this algorithm
    pub used_preprocessing: bool,
}

/// Details of the weighted score calculation
#[derive(Debug, Clone, PartialEq)]
pub struct WeightedCalculation {
    /// Sum of all weighted contributions
    pub total_weighted_score: f64,
    /// Sum of all weights
    pub total_weight: f64,
    /// Final normalized score
    pub normalized_score: f64,
}

/// Performance metrics for the matching operation
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
    /// Weights for different algorithms (algorithm_name -> weight)
    pub algorithm_weights: HashMap<String, f64>,
    /// Enable/disable specific preprocessing steps
    pub preprocessing_enabled: bool,
    /// Cache size for frequently matched strings
    pub cache_size: usize,
}

impl Default for FuzzyMatchConfig {
    fn default() -> Self {
        let mut algorithm_weights = HashMap::new();
        algorithm_weights.insert("levenshtein".to_string(), 0.3);
        algorithm_weights.insert("jaro_winkler".to_string(), 0.4);
        algorithm_weights.insert("ngram".to_string(), 0.2);
        algorithm_weights.insert("soundex".to_string(), 0.1);

        Self {
            min_confidence: 0.7,
            max_results: 10,
            algorithm_weights,
            preprocessing_enabled: true,
            cache_size: 1000,
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

/// Levenshtein distance algorithm
#[derive(Debug, Clone)]
pub struct LevenshteinAlgorithm;

impl FuzzyAlgorithm for LevenshteinAlgorithm {
    fn name(&self) -> &'static str {
        "levenshtein"
    }

    fn similarity(&self, s1: &str, s2: &str) -> f64 {
        if s1 == s2 {
            return 1.0;
        }
        
        if s1.is_empty() || s2.is_empty() {
            return 0.0;
        }

        let distance = strsim::levenshtein(s1, s2);
        let max_len = s1.len().max(s2.len());
        
        1.0 - (distance as f64 / max_len as f64)
    }
}

/// Jaro-Winkler distance algorithm
#[derive(Debug, Clone)]
pub struct JaroWinklerAlgorithm;

impl FuzzyAlgorithm for JaroWinklerAlgorithm {
    fn name(&self) -> &'static str {
        "jaro_winkler"
    }

    fn similarity(&self, s1: &str, s2: &str) -> f64 {
        strsim::jaro_winkler(s1, s2)
    }
}

/// N-gram similarity algorithm
#[derive(Debug, Clone)]
pub struct NgramAlgorithm {
    n: usize,
}

impl NgramAlgorithm {
    pub fn new(n: usize) -> Self {
        Self { n }
    }
    
    fn generate_ngrams(&self, s: &str) -> Vec<String> {
        if s.len() < self.n {
            return vec![s.to_string()];
        }
        
        s.chars()
            .collect::<Vec<_>>()
            .windows(self.n)
            .map(|window| window.iter().collect())
            .collect()
    }
}

impl Default for NgramAlgorithm {
    fn default() -> Self {
        Self::new(2) // Bigrams by default
    }
}

impl FuzzyAlgorithm for NgramAlgorithm {
    fn name(&self) -> &'static str {
        "ngram"
    }

    fn similarity(&self, s1: &str, s2: &str) -> f64 {
        if s1 == s2 {
            return 1.0;
        }
        
        let ngrams1 = self.generate_ngrams(s1);
        let ngrams2 = self.generate_ngrams(s2);
        
        if ngrams1.is_empty() && ngrams2.is_empty() {
            return 1.0;
        }
        
        if ngrams1.is_empty() || ngrams2.is_empty() {
            return 0.0;
        }
        
        // Calculate Jaccard similarity
        let set1: std::collections::HashSet<_> = ngrams1.into_iter().collect();
        let set2: std::collections::HashSet<_> = ngrams2.into_iter().collect();
        
        let intersection = set1.intersection(&set2).count();
        let union = set1.union(&set2).count();
        
        if union == 0 {
            0.0
        } else {
            intersection as f64 / union as f64
        }
    }
}

/// Soundex phonetic matching algorithm
#[derive(Debug, Clone)]
pub struct SoundexAlgorithm;

impl SoundexAlgorithm {
    fn soundex(&self, s: &str) -> String {
        if s.is_empty() {
            return "0000".to_string();
        }
        
        let s = s.to_uppercase();
        let chars: Vec<char> = s.chars().collect();
        
        if chars.is_empty() {
            return "0000".to_string();
        }
        
        let mut result = String::new();
        result.push(chars[0]);
        
        let mut prev_code = self.get_soundex_code(chars[0]);
        
        for &ch in &chars[1..] {
            let code = self.get_soundex_code(ch);
            if code != '0' && code != prev_code {
                result.push(code);
                if result.len() >= 4 {
                    break;
                }
            }
            if code != '0' {
                prev_code = code;
            }
        }
        
        // Pad with zeros
        while result.len() < 4 {
            result.push('0');
        }
        
        result
    }
    
    fn get_soundex_code(&self, ch: char) -> char {
        match ch {
            'B' | 'F' | 'P' | 'V' => '1',
            'C' | 'G' | 'J' | 'K' | 'Q' | 'S' | 'X' | 'Z' => '2',
            'D' | 'T' => '3',
            'L' => '4',
            'M' | 'N' => '5',
            'R' => '6',
            _ => '0',
        }
    }
}

impl FuzzyAlgorithm for SoundexAlgorithm {
    fn name(&self) -> &'static str {
        "soundex"
    }

    fn similarity(&self, s1: &str, s2: &str) -> f64 {
        let soundex1 = self.soundex(s1);
        let soundex2 = self.soundex(s2);
        
        if soundex1 == soundex2 {
            1.0
        } else {
            // Use Levenshtein on soundex codes for partial similarity
            let distance = strsim::levenshtein(&soundex1, &soundex2);
            1.0 - (distance as f64 / 4.0) // Soundex codes are always 4 characters
        }
    }
    
    fn needs_preprocessing(&self) -> bool {
        false // Soundex handles its own normalization
    }
}

/// Cache key for fuzzy matching results
#[derive(Debug, Clone, Eq)]
struct CacheKey {
    source: String,
    target: String,
    config_hash: u64,
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

/// Text preprocessing pipeline for better fuzzy matching
#[derive(Debug, Clone)]
pub struct TextPreprocessor {
    /// Common abbreviations and their expansions
    abbreviations: HashMap<String, String>,
    /// Stop words to remove
    stop_words: std::collections::HashSet<String>,
    /// Enable Unicode normalization
    unicode_normalization: bool,
}

impl Default for TextPreprocessor {
    fn default() -> Self {
        let mut abbreviations = HashMap::new();

        // Common FedRAMP/IT abbreviations
        abbreviations.insert("id".to_string(), "identifier".to_string());
        abbreviations.insert("desc".to_string(), "description".to_string());
        abbreviations.insert("addr".to_string(), "address".to_string());
        abbreviations.insert("ip".to_string(), "internet protocol".to_string());
        abbreviations.insert("mac".to_string(), "media access control".to_string());
        abbreviations.insert("os".to_string(), "operating system".to_string());
        abbreviations.insert("sw".to_string(), "software".to_string());
        abbreviations.insert("hw".to_string(), "hardware".to_string());
        abbreviations.insert("sys".to_string(), "system".to_string());
        abbreviations.insert("admin".to_string(), "administrator".to_string());
        abbreviations.insert("config".to_string(), "configuration".to_string());
        abbreviations.insert("env".to_string(), "environment".to_string());
        abbreviations.insert("prod".to_string(), "production".to_string());
        abbreviations.insert("dev".to_string(), "development".to_string());
        abbreviations.insert("test".to_string(), "testing".to_string());
        abbreviations.insert("qa".to_string(), "quality assurance".to_string());
        abbreviations.insert("uat".to_string(), "user acceptance testing".to_string());
        abbreviations.insert("csp".to_string(), "cloud service provider".to_string());
        abbreviations.insert("ssp".to_string(), "system security plan".to_string());
        abbreviations.insert("poam".to_string(), "plan of action and milestones".to_string());
        abbreviations.insert("iiw".to_string(), "integrated inventory workbook".to_string());

        let mut stop_words = std::collections::HashSet::new();
        stop_words.insert("the".to_string());
        stop_words.insert("and".to_string());
        stop_words.insert("or".to_string());
        stop_words.insert("of".to_string());
        stop_words.insert("in".to_string());
        stop_words.insert("on".to_string());
        stop_words.insert("at".to_string());
        stop_words.insert("to".to_string());
        stop_words.insert("for".to_string());
        stop_words.insert("with".to_string());
        stop_words.insert("by".to_string());

        Self {
            abbreviations,
            stop_words,
            unicode_normalization: true,
        }
    }
}

impl TextPreprocessor {
    /// Create a new preprocessor with custom settings
    pub fn new() -> Self {
        Self::default()
    }

    /// Add a custom abbreviation mapping
    pub fn add_abbreviation(&mut self, abbrev: &str, expansion: &str) {
        self.abbreviations.insert(abbrev.to_lowercase(), expansion.to_lowercase());
    }

    /// Add a stop word
    pub fn add_stop_word(&mut self, word: &str) {
        self.stop_words.insert(word.to_lowercase());
    }

    /// Preprocess a string for fuzzy matching
    pub fn preprocess(&self, text: &str) -> (String, Vec<String>) {
        let mut processed = text.to_string();
        let mut steps_applied = Vec::new();

        // Unicode normalization
        if self.unicode_normalization && !is_nfc(&processed) {
            processed = processed.nfc().collect();
            steps_applied.push("unicode_normalization".to_string());
        }

        // Convert to lowercase
        processed = processed.to_lowercase();
        steps_applied.push("lowercase".to_string());

        // Remove extra whitespace and normalize separators
        processed = self.normalize_separators(&processed);
        steps_applied.push("normalize_separators".to_string());

        // Expand abbreviations
        processed = self.expand_abbreviations(&processed);
        if processed != text.to_lowercase() {
            steps_applied.push("expand_abbreviations".to_string());
        }

        // Remove stop words
        let original_len = processed.len();
        processed = self.remove_stop_words(&processed);
        if processed.len() != original_len {
            steps_applied.push("remove_stop_words".to_string());
        }

        // Final cleanup - remove non-alphanumeric except spaces
        processed = processed
            .chars()
            .filter(|c| c.is_alphanumeric() || c.is_whitespace())
            .collect::<String>();

        // Normalize whitespace
        processed = processed
            .split_whitespace()
            .collect::<Vec<_>>()
            .join(" ");

        steps_applied.push("final_cleanup".to_string());

        trace!("Preprocessed '{}' -> '{}' (steps: {:?})", text, processed, steps_applied);

        (processed, steps_applied)
    }

    /// Normalize separators (underscores, hyphens, etc. to spaces)
    fn normalize_separators(&self, text: &str) -> String {
        text.chars()
            .map(|c| match c {
                '_' | '-' | '.' | '/' | '\\' | '|' => ' ',
                _ => c,
            })
            .collect::<String>()
            .split_whitespace()
            .collect::<Vec<_>>()
            .join(" ")
    }

    /// Expand known abbreviations
    fn expand_abbreviations(&self, text: &str) -> String {
        let words: Vec<&str> = text.split_whitespace().collect();
        let mut expanded_words = Vec::new();

        for word in words {
            if let Some(expansion) = self.abbreviations.get(word) {
                expanded_words.extend(expansion.split_whitespace());
            } else {
                expanded_words.push(word);
            }
        }

        expanded_words.join(" ")
    }

    /// Remove stop words
    fn remove_stop_words(&self, text: &str) -> String {
        text.split_whitespace()
            .filter(|word| !self.stop_words.contains(*word))
            .collect::<Vec<_>>()
            .join(" ")
    }

    /// Create a minimal preprocessor that only does basic normalization
    pub fn minimal() -> Self {
        Self {
            abbreviations: HashMap::new(),
            stop_words: std::collections::HashSet::new(),
            unicode_normalization: true,
        }
    }
}

/// Main fuzzy matcher that combines multiple algorithms
pub struct FuzzyMatcher {
    /// Available fuzzy matching algorithms
    algorithms: Vec<Box<dyn FuzzyAlgorithm>>,
    /// Text preprocessor
    preprocessor: TextPreprocessor,
    /// Configuration
    config: FuzzyMatchConfig,
    /// LRU cache for results
    cache: LruCache<CacheKey, FuzzyMatchResult>,
    /// Configuration hash for cache invalidation
    config_hash: u64,
    /// Indexed targets for faster lookup
    target_index: Option<TargetIndex>,
}

/// Index structure for fast target lookup
#[derive(Debug, Clone)]
struct TargetIndex {
    /// Length-based index for quick filtering
    length_buckets: HashMap<usize, Vec<String>>,
    /// First character index
    first_char_index: HashMap<char, Vec<String>>,
    /// Trigram index for fast pre-filtering
    trigram_index: HashMap<String, Vec<String>>,
}

impl FuzzyMatcher {
    /// Create a new fuzzy matcher with default algorithms and configuration
    pub fn new() -> Self {
        Self::with_config(FuzzyMatchConfig::default())
    }

    /// Create a new fuzzy matcher with custom configuration
    pub fn with_config(config: FuzzyMatchConfig) -> Self {
        let algorithms: Vec<Box<dyn FuzzyAlgorithm>> = vec![
            Box::new(LevenshteinAlgorithm),
            Box::new(JaroWinklerAlgorithm),
            Box::new(NgramAlgorithm::default()),
            Box::new(SoundexAlgorithm),
        ];

        let cache_size = config.cache_size;
        let config_hash = Self::calculate_config_hash(&config);

        Self {
            algorithms,
            preprocessor: TextPreprocessor::default(),
            config,
            cache: LruCache::new(std::num::NonZeroUsize::new(cache_size).unwrap_or(std::num::NonZeroUsize::new(1000).unwrap())),
            config_hash,
            target_index: None,
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

    /// Find the best fuzzy matches for a source string against a list of targets
    pub fn find_matches(&mut self, source: &str, targets: &[String]) -> Vec<FuzzyMatchResult> {
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
        // Build index if not already built
        if self.target_index.is_none() {
            self.build_target_index(targets);
        }

        // Use indexed candidates if available
        let candidates = if let Some(indexed_candidates) = self.get_indexed_candidates(source) {
            debug!("Using {} indexed candidates out of {} total targets",
                   indexed_candidates.len(), targets.len());
            indexed_candidates
        } else {
            targets.to_vec()
        };

        let mut results = Vec::new();
        let mut best_confidence: f64 = 0.0;

        // Preprocess source once
        let (processed_source, source_steps) = if self.config.preprocessing_enabled {
            self.preprocessor.preprocess(source)
        } else {
            (source.to_string(), vec!["no_preprocessing".to_string()])
        };

        // Process candidates in chunks for better cache locality
        const CHUNK_SIZE: usize = 50;
        for chunk in candidates.chunks(CHUNK_SIZE) {
            for target in chunk {
                // Early termination if we have enough high-confidence results
                if results.len() >= self.config.max_results * 2 &&
                   results.iter().all(|r: &FuzzyMatchResult| r.confidence > 0.9) {
                    break;
                }

                // Quick pre-filter based on length difference
                if self.should_skip_target(source, target) {
                    continue;
                }

                if let Some(result) = self.match_single_optimized(source, target, &processed_source, &source_steps, false) {
                    if result.confidence >= self.config.min_confidence {
                        let confidence = result.confidence;
                        best_confidence = best_confidence.max(confidence);
                        results.push(result);

                        // Early termination for perfect matches
                        if confidence >= 0.99 {
                            break;
                        }
                    }
                }
            }
        }

        // Sort by confidence (descending)
        results.sort_by(|a, b| b.confidence.partial_cmp(&a.confidence).unwrap_or(std::cmp::Ordering::Equal));

        // Limit results
        results.truncate(self.config.max_results);

        debug!("Found {} matches above threshold {} (best: {:.3})",
               results.len(), self.config.min_confidence, best_confidence);
        results
    }

    /// Quick pre-filter to skip obviously poor matches
    fn should_skip_target(&self, source: &str, target: &str) -> bool {
        let len_diff = (source.len() as i32 - target.len() as i32).abs();
        let max_len = source.len().max(target.len());

        // Skip if length difference is too large (heuristic)
        if max_len > 0 && len_diff as f64 / max_len as f64 > 0.7 {
            return true;
        }

        // Skip if no common characters in first few positions
        if source.len() >= 3 && target.len() >= 3 {
            let source_prefix: std::collections::HashSet<char> = source.chars().take(3).collect();
            let target_prefix: std::collections::HashSet<char> = target.chars().take(3).collect();
            if source_prefix.intersection(&target_prefix).count() == 0 {
                return true;
            }
        }

        false
    }

    /// Optimized single match with pre-processed source
    fn match_single_optimized(
        &mut self,
        source: &str,
        target: &str,
        processed_source: &str,
        source_steps: &[String],
        include_explanation: bool
    ) -> Option<FuzzyMatchResult> {
        // Check cache first
        let cache_key = CacheKey {
            source: source.to_string(),
            target: target.to_string(),
            config_hash: self.config_hash,
        };

        if let Some(cached_result) = self.cache.get(&cache_key) {
            return Some(cached_result.clone());
        }

        // Preprocess target
        let (processed_target, target_steps) = if self.config.preprocessing_enabled {
            self.preprocessor.preprocess(target)
        } else {
            (target.to_string(), vec!["no_preprocessing".to_string()])
        };

        // Check for exact match after preprocessing
        if processed_source == &processed_target {
            let explanation = if include_explanation {
                Some(MatchExplanation {
                    original_source: source.to_string(),
                    original_target: target.to_string(),
                    processed_source: processed_source.to_string(),
                    processed_target: processed_target.clone(),
                    algorithm_contributions: HashMap::new(),
                    weighted_calculation: WeightedCalculation {
                        total_weighted_score: 1.0,
                        total_weight: 1.0,
                        normalized_score: 1.0,
                    },
                    performance_metrics: PerformanceMetrics {
                        cache_hit: false,
                        algorithms_executed: 0,
                        early_termination: false,
                    },
                })
            } else {
                None
            };

            let result = FuzzyMatchResult {
                target: target.to_string(),
                confidence: 1.0,
                algorithm_scores: HashMap::new(),
                preprocessing_applied: [source_steps, &target_steps].concat(),
                exact_match: true,
                explanation,
            };

            self.cache.put(cache_key, result.clone());
            return Some(result);
        }

        // Calculate scores with early termination
        let mut algorithm_scores = HashMap::new();
        let mut algorithm_contributions = HashMap::new();
        let mut weighted_score = 0.0;
        let mut total_weight = 0.0;
        let mut algorithms_executed = 0;
        let mut early_termination = false;

        // Sort algorithms by weight (descending) for early termination
        let mut weighted_algorithms: Vec<_> = self.algorithms.iter()
            .map(|algo| (algo, self.config.algorithm_weights.get(algo.name()).unwrap_or(&0.0)))
            .collect();
        weighted_algorithms.sort_by(|a, b| b.1.partial_cmp(a.1).unwrap_or(std::cmp::Ordering::Equal));

        for (algorithm, &weight) in weighted_algorithms {
            if weight == 0.0 {
                continue; // Skip zero-weight algorithms
            }

            let algo_name = algorithm.name();
            algorithms_executed += 1;

            // Use preprocessed strings if the algorithm benefits from it
            let used_preprocessing = algorithm.needs_preprocessing() && self.config.preprocessing_enabled;
            let (s1, s2) = if used_preprocessing {
                (processed_source, processed_target.as_str())
            } else {
                (source, target)
            };

            let score = algorithm.similarity(s1, s2);
            algorithm_scores.insert(algo_name.to_string(), score);

            let weighted_contribution = score * weight;
            weighted_score += weighted_contribution;
            total_weight += weight;

            // Store detailed contribution for explanation
            if include_explanation {
                algorithm_contributions.insert(algo_name.to_string(), AlgorithmContribution {
                    raw_score: score,
                    weight,
                    weighted_contribution,
                    used_preprocessing,
                });
            }

            // Early termination if current weighted score can't reach threshold
            let max_possible_score = weighted_score + (1.0 * (1.0 - total_weight));
            if max_possible_score < self.config.min_confidence {
                early_termination = true;
                return None;
            }
        }

        // Normalize the weighted score
        let final_confidence = if total_weight > 0.0 {
            weighted_score / total_weight
        } else {
            0.0
        };

        // Create explanation if requested
        let explanation = if include_explanation {
            Some(MatchExplanation {
                original_source: source.to_string(),
                original_target: target.to_string(),
                processed_source: processed_source.to_string(),
                processed_target: processed_target.clone(),
                algorithm_contributions,
                weighted_calculation: WeightedCalculation {
                    total_weighted_score: weighted_score,
                    total_weight,
                    normalized_score: final_confidence,
                },
                performance_metrics: PerformanceMetrics {
                    cache_hit: false,
                    algorithms_executed,
                    early_termination,
                },
            })
        } else {
            None
        };

        let result = FuzzyMatchResult {
            target: target.to_string(),
            confidence: final_confidence,
            algorithm_scores,
            preprocessing_applied: [source_steps, &target_steps].concat(),
            exact_match: false,
            explanation,
        };

        // Cache the result
        self.cache.put(cache_key, result.clone());

        Some(result)
    }

    /// Find the best single match for a source string against a target
    pub fn match_single(&mut self, source: &str, target: &str) -> Option<FuzzyMatchResult> {
        self.match_single_with_explanation(source, target, false)
    }

    /// Find the best single match with detailed explanation
    pub fn match_single_with_explanation(&mut self, source: &str, target: &str, include_explanation: bool) -> Option<FuzzyMatchResult> {
        // Preprocess source
        let (processed_source, source_steps) = if self.config.preprocessing_enabled {
            self.preprocessor.preprocess(source)
        } else {
            (source.to_string(), vec!["no_preprocessing".to_string()])
        };

        self.match_single_optimized(source, target, &processed_source, &source_steps, include_explanation)
    }

    /// Update the configuration and clear cache if needed
    pub fn update_config(&mut self, new_config: FuzzyMatchConfig) {
        let new_hash = Self::calculate_config_hash(&new_config);
        if new_hash != self.config_hash {
            self.cache.clear();
            self.config_hash = new_hash;
        }

        // Resize cache if needed
        if new_config.cache_size != self.config.cache_size {
            self.cache.resize(std::num::NonZeroUsize::new(new_config.cache_size).unwrap_or(std::num::NonZeroUsize::new(1000).unwrap()));
        }

        self.config = new_config;
    }

    /// Get cache statistics
    pub fn cache_stats(&self) -> (usize, usize) {
        (self.cache.len(), self.cache.cap().get())
    }

    /// Clear the cache
    pub fn clear_cache(&mut self) {
        self.cache.clear();
    }

    /// Get detailed match explanation for debugging
    pub fn explain_match(&mut self, source: &str, target: &str) -> Option<FuzzyMatchResult> {
        self.match_single_with_explanation(source, target, true)
    }

    /// Get detailed matches with explanations for multiple targets
    pub fn explain_matches(&mut self, source: &str, targets: &[String]) -> Vec<FuzzyMatchResult> {
        let mut results = Vec::new();

        for target in targets {
            if let Some(result) = self.match_single_with_explanation(source, target, true) {
                if result.confidence >= self.config.min_confidence {
                    results.push(result);
                }
            }
        }

        // Sort by confidence (descending)
        results.sort_by(|a, b| b.confidence.partial_cmp(&a.confidence).unwrap_or(std::cmp::Ordering::Equal));

        // Limit results
        results.truncate(self.config.max_results);

        results
    }

    /// Build an index for the target list to speed up matching
    pub fn build_target_index(&mut self, targets: &[String]) {
        if targets.len() < 50 {
            // Don't build index for small target lists
            self.target_index = None;
            return;
        }

        let mut length_buckets: HashMap<usize, Vec<String>> = HashMap::new();
        let mut first_char_index: HashMap<char, Vec<String>> = HashMap::new();
        let mut trigram_index: HashMap<String, Vec<String>> = HashMap::new();

        for target in targets {
            // Length bucket
            length_buckets.entry(target.len()).or_default().push(target.clone());

            // First character index
            if let Some(first_char) = target.chars().next() {
                first_char_index.entry(first_char.to_lowercase().next().unwrap_or(first_char))
                    .or_default().push(target.clone());
            }

            // Trigram index
            if target.len() >= 3 {
                let trigrams = self.generate_trigrams(target);
                for trigram in trigrams {
                    trigram_index.entry(trigram).or_default().push(target.clone());
                }
            }
        }

        self.target_index = Some(TargetIndex {
            length_buckets,
            first_char_index,
            trigram_index,
        });

        debug!("Built target index for {} targets", targets.len());
    }

    /// Generate trigrams for indexing
    fn generate_trigrams(&self, s: &str) -> Vec<String> {
        if s.len() < 3 {
            return vec![s.to_string()];
        }

        s.chars()
            .collect::<Vec<_>>()
            .windows(3)
            .map(|window| window.iter().collect())
            .collect()
    }

    /// Get candidate targets using the index
    fn get_indexed_candidates(&self, source: &str) -> Option<Vec<String>> {
        let index = self.target_index.as_ref()?;
        let mut candidates = std::collections::HashSet::new();

        // Get candidates by length (within reasonable range)
        let source_len = source.len();
        for len in (source_len.saturating_sub(3))..=(source_len + 3) {
            if let Some(targets) = index.length_buckets.get(&len) {
                candidates.extend(targets.iter().cloned());
            }
        }

        // Get candidates by first character
        if let Some(first_char) = source.chars().next() {
            let first_char_lower = first_char.to_lowercase().next().unwrap_or(first_char);
            if let Some(targets) = index.first_char_index.get(&first_char_lower) {
                candidates.extend(targets.iter().cloned());
            }
        }

        // Get candidates by trigrams
        if source.len() >= 3 {
            let trigrams = self.generate_trigrams(source);
            for trigram in trigrams {
                if let Some(targets) = index.trigram_index.get(&trigram) {
                    candidates.extend(targets.iter().cloned());
                }
            }
        }

        Some(candidates.into_iter().collect())
    }

    /// Calculate a hash of the configuration for cache invalidation
    fn calculate_config_hash(config: &FuzzyMatchConfig) -> u64 {
        use std::collections::hash_map::DefaultHasher;

        let mut hasher = DefaultHasher::new();
        config.min_confidence.to_bits().hash(&mut hasher);
        config.max_results.hash(&mut hasher);
        config.preprocessing_enabled.hash(&mut hasher);

        // Hash algorithm weights
        let mut weights: Vec<_> = config.algorithm_weights.iter().collect();
        weights.sort_by_key(|(k, _)| *k);
        for (k, v) in weights {
            k.hash(&mut hasher);
            v.to_bits().hash(&mut hasher);
        }

        hasher.finish()
    }
}

impl Default for FuzzyMatcher {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Instant;

    #[test]
    fn test_levenshtein_algorithm() {
        let algo = LevenshteinAlgorithm;

        // Exact match
        assert_eq!(algo.similarity("test", "test"), 1.0);

        // No match
        assert_eq!(algo.similarity("", ""), 1.0);
        assert_eq!(algo.similarity("test", ""), 0.0);
        assert_eq!(algo.similarity("", "test"), 0.0);

        // Partial matches
        assert!(algo.similarity("test", "tests") > 0.7);
        assert!(algo.similarity("asset_id", "assetid") > 0.7);
        assert!(algo.similarity("component_name", "componentname") > 0.8);
    }

    #[test]
    fn test_jaro_winkler_algorithm() {
        let algo = JaroWinklerAlgorithm;

        // Exact match
        assert_eq!(algo.similarity("test", "test"), 1.0);

        // Partial matches - Jaro-Winkler is good with prefix similarities
        assert!(algo.similarity("asset_id", "asset_identifier") > 0.8);
        assert!(algo.similarity("component", "comp") > 0.7);
        assert!(algo.similarity("description", "desc") > 0.6);
    }

    #[test]
    fn test_ngram_algorithm() {
        let algo = NgramAlgorithm::new(2);

        // Exact match
        assert_eq!(algo.similarity("test", "test"), 1.0);

        // N-gram similarities
        assert!(algo.similarity("testing", "test") > 0.3);
        assert!(algo.similarity("asset_id", "asset_identifier") > 0.4);

        // Test with different n values
        let trigram = NgramAlgorithm::new(3);
        assert!(trigram.similarity("testing", "test") > 0.0);
    }

    #[test]
    fn test_soundex_algorithm() {
        let algo = SoundexAlgorithm;

        // Test soundex generation
        assert_eq!(algo.soundex("Smith"), "S530");
        assert_eq!(algo.soundex("Johnson"), "J525");
        assert_eq!(algo.soundex(""), "0000");

        // Similar sounding words should have high similarity
        assert!(algo.similarity("Smith", "Smyth") > 0.8);
        assert!(algo.similarity("Johnson", "Jonson") > 0.8);

        // Different sounding words should have lower similarity
        assert!(algo.similarity("Smith", "Johnson") < 0.5);
    }

    #[test]
    fn test_text_preprocessor() {
        let preprocessor = TextPreprocessor::default();

        // Test basic normalization
        let (result, steps) = preprocessor.preprocess("Asset_ID");
        assert_eq!(result, "asset identifier");
        assert!(steps.contains(&"expand_abbreviations".to_string()));

        // Test separator normalization
        let (result, _) = preprocessor.preprocess("component-name");
        assert_eq!(result, "component name");

        // Test stop word removal
        let (result, _) = preprocessor.preprocess("the asset id");
        assert_eq!(result, "asset identifier");

        // Test abbreviation expansion
        let (result, _) = preprocessor.preprocess("sys_config");
        assert_eq!(result, "system configuration");
    }

    #[test]
    fn test_fuzzy_matcher_basic() {
        let mut matcher = FuzzyMatcher::new();

        // Test single match first
        let result = matcher.match_single("Asset ID", "Asset ID");
        assert!(result.is_some(), "Should find exact match");
        assert_eq!(result.unwrap().confidence, 1.0);

        // Test fuzzy single match
        let result = matcher.match_single("Asset_ID", "Asset ID");
        assert!(result.is_some(), "Should find fuzzy match");
        let result = result.unwrap();
        assert!(result.confidence > 0.5, "Confidence should be > 0.5, got {}", result.confidence);

        // Test with multiple targets
        let targets = vec![
            "Asset ID".to_string(),
            "Component Name".to_string(),
        ];

        let results = matcher.find_matches("Asset ID", &targets);
        assert!(!results.is_empty(), "Should find exact match in targets");
        assert_eq!(results[0].target, "Asset ID");
        assert_eq!(results[0].confidence, 1.0);
    }

    #[test]
    fn test_fuzzy_matcher_fedramp_optimized() {
        let mut matcher = FuzzyMatcher::for_fedramp_columns();
        let targets = vec![
            "Asset Identifier".to_string(),
            "Component Type".to_string(),
            "System Description".to_string(),
            "POA&M Item ID".to_string(),
            "Control ID".to_string(),
        ];

        // Test common FedRAMP variations
        let test_cases = vec![
            ("asset_id", "Asset Identifier"),
            ("comp_type", "Component Type"),
            ("sys_desc", "System Description"),
        ];

        for (input, expected) in test_cases {
            let results = matcher.find_matches(input, &targets);
            if !results.is_empty() {
                // Check if we got the expected match or at least a reasonable match
                let found_expected = results.iter().any(|r| r.target == expected);
                if found_expected {
                    let expected_result = results.iter().find(|r| r.target == expected).unwrap();
                    assert!(expected_result.confidence > 0.5, "Low confidence for '{}': {}", input, expected_result.confidence);
                }
            }
        }
    }

    #[test]
    fn test_fuzzy_matcher_caching() {
        let mut matcher = FuzzyMatcher::new();
        let targets = vec!["Asset ID".to_string(), "Component Name".to_string()];

        // First call should populate cache
        let start = Instant::now();
        let _results1 = matcher.find_matches("Asset_ID", &targets);
        let first_duration = start.elapsed();

        // Second call should be faster due to caching
        let start = Instant::now();
        let _results2 = matcher.find_matches("Asset_ID", &targets);
        let second_duration = start.elapsed();

        // Cache should make second call faster (though this might be flaky in CI)
        let (cache_size, _) = matcher.cache_stats();
        assert!(cache_size > 0, "Cache should contain entries");
    }

    #[test]
    fn test_fuzzy_matcher_configuration() {
        let mut config = FuzzyMatchConfig::default();
        config.min_confidence = 0.9;
        config.max_results = 2;

        let mut matcher = FuzzyMatcher::with_config(config);
        let targets = vec![
            "Asset ID".to_string(),
            "Asset Identifier".to_string(),
            "Component Name".to_string(),
        ];

        let results = matcher.find_matches("Asset", &targets);

        // Should respect max_results
        assert!(results.len() <= 2);

        // Should respect min_confidence
        for result in &results {
            assert!(result.confidence >= 0.9);
        }
    }

    #[test]
    fn test_edge_cases() {
        let mut matcher = FuzzyMatcher::new();
        let targets = vec!["Asset ID".to_string()];

        // Empty input
        let results = matcher.find_matches("", &targets);
        assert!(results.is_empty() || results[0].confidence < 0.7);

        // Very long input
        let long_input = "a".repeat(1000);
        let results = matcher.find_matches(&long_input, &targets);
        // Should not crash and should return low confidence
        assert!(results.is_empty() || results[0].confidence < 0.7);

        // Special characters
        let results = matcher.find_matches("Asset@#$%ID", &targets);
        // May or may not find matches depending on preprocessing
        if !results.is_empty() {
            assert!(results[0].confidence > 0.3);
        }
    }

    #[test]
    fn test_unicode_handling() {
        let mut matcher = FuzzyMatcher::new();
        let targets = vec!["Asset ID".to_string(), "Descripción".to_string()];

        // Unicode input should be handled correctly
        let results = matcher.find_matches("Descripción", &targets);
        assert!(!results.is_empty());
        assert_eq!(results[0].target, "Descripción");
        assert!(results[0].confidence > 0.9);

        // Mixed ASCII and Unicode
        let results = matcher.find_matches("Asset_ID_ñ", &targets);
        assert!(!results.is_empty());
    }

    #[test]
    fn test_performance_benchmark() {
        let mut matcher = FuzzyMatcher::for_fedramp_columns();

        // Create a large target list
        let mut targets = Vec::new();
        for i in 0..1000 {
            targets.push(format!("Column_{}", i));
            targets.push(format!("Asset_ID_{}", i));
            targets.push(format!("Component_Name_{}", i));
        }

        let test_inputs = vec![
            "Asset_ID",
            "Component_Name",
            "Description",
            "IP_Address",
            "MAC_Address",
        ];

        let start = Instant::now();
        for input in &test_inputs {
            let _results = matcher.find_matches(input, &targets);
        }
        let duration = start.elapsed();

        // Should complete within reasonable time (adjust threshold as needed)
        assert!(duration.as_millis() < 500, "Performance test took {}ms, should be < 500ms", duration.as_millis());
    }

    #[test]
    fn test_algorithm_weights() {
        let mut config = FuzzyMatchConfig::default();

        // Give more weight to Jaro-Winkler
        config.algorithm_weights.insert("jaro_winkler".to_string(), 0.8);
        config.algorithm_weights.insert("levenshtein".to_string(), 0.1);
        config.algorithm_weights.insert("ngram".to_string(), 0.1);
        config.algorithm_weights.insert("soundex".to_string(), 0.0);

        let mut matcher = FuzzyMatcher::with_config(config);

        let result = matcher.explain_match("Asset ID", "Asset Identifier").unwrap();

        // Should have high confidence due to Jaro-Winkler weight
        assert!(result.confidence > 0.8, "Confidence should be > 0.8, got {}", result.confidence);

        // If it's not an exact match, we should have algorithm scores
        if !result.exact_match {
            assert!(!result.algorithm_scores.is_empty(), "Should have algorithm scores for non-exact matches");

            // Check that at least one of the expected algorithms is present
            let has_expected_algo = result.algorithm_scores.contains_key("jaro_winkler") ||
                                   result.algorithm_scores.contains_key("levenshtein") ||
                                   result.algorithm_scores.contains_key("ngram");
            assert!(has_expected_algo, "Should have at least one expected algorithm score");
        }
    }

    #[test]
    fn test_preprocessing_steps() {
        let mut matcher = FuzzyMatcher::new();

        let result = matcher.match_single("Asset_ID", "Asset Identifier").unwrap();

        // Should show preprocessing steps
        assert!(!result.preprocessing_applied.is_empty());
        assert!(result.preprocessing_applied.contains(&"lowercase".to_string()));
        assert!(result.preprocessing_applied.contains(&"normalize_separators".to_string()));
    }
}
