# Implement Fuzzy String Matching for Column Detection

**Task ID:** 9cDthaDSZsdeZiEpwDMkFF  
**Component:** 1.2: Column Mapping Engine  
**Status:** Not Started  
**Priority:** High  

## Overview

Build fuzzy matching algorithm to detect column names with variations and typos, enabling robust column identification even when Excel headers don't exactly match mapping configurations.

## Objectives

- Implement fuzzy string matching algorithms
- Handle common column name variations and typos
- Support multiple matching strategies with configurable thresholds
- Optimize for performance with large column sets
- Provide confidence scoring for matches

## Technical Requirements

### Fuzzy Matching Algorithms
1. **Levenshtein Distance**
   - Edit distance calculation for character-level differences
   - Configurable maximum distance threshold
   - Normalized scoring (0.0 to 1.0)

2. **Jaro-Winkler Distance**
   - Better handling of transpositions
   - Prefix bonus for common prefixes
   - Optimized for name-like strings

3. **N-gram Similarity**
   - Character and word-level n-grams
   - Jaccard similarity coefficient
   - Robust against word order changes

4. **Phonetic Matching**
   - Soundex algorithm for phonetically similar names
   - Handle pronunciation-based variations
   - Useful for transcription errors

### Core Functionality
1. **Multi-Algorithm Matching**
   - Combine multiple algorithms for best results
   - Weighted scoring based on algorithm strengths
   - Configurable algorithm selection per mapping type

2. **Preprocessing Pipeline**
   - Text normalization (case, whitespace, punctuation)
   - Common abbreviation expansion
   - Stop word removal for better matching

3. **Performance Optimization**
   - Efficient string indexing and caching
   - Early termination for low-confidence matches
   - Batch processing for multiple columns

## Implementation Details

### Data Structures
```rust
pub struct FuzzyMatcher {
    algorithms: Vec<Box<dyn MatchingAlgorithm>>,
    preprocessor: TextPreprocessor,
    cache: LruCache<String, Vec<MatchResult>>,
    config: MatchingConfig,
}

pub struct MatchResult {
    pub target: String,
    pub confidence: f64,
    pub algorithm_scores: HashMap<String, f64>,
    pub preprocessing_applied: Vec<String>,
}

pub struct MatchingConfig {
    pub min_confidence: f64,
    pub max_results: usize,
    pub algorithm_weights: HashMap<String, f64>,
    pub preprocessing_options: PreprocessingOptions,
}
```

### Key Features
- **Configurable Thresholds**: Adjustable confidence levels per document type
- **Caching**: LRU cache for frequently matched strings
- **Batch Processing**: Efficient matching of multiple columns
- **Detailed Scoring**: Per-algorithm confidence breakdown

### Common Column Variations
- Case variations: "Asset Name" vs "asset name" vs "ASSET_NAME"
- Punctuation differences: "IP Address" vs "IP-Address" vs "IP_Address"
- Abbreviations: "Description" vs "Desc" vs "Description/Notes"
- Typos: "Assest Name" vs "Asset Name"
- Word order: "Name Asset" vs "Asset Name"

## Dependencies

- `strsim` for string similarity algorithms
- `unicode-normalization` for text preprocessing
- `lru` for caching frequently matched strings
- `regex` for text preprocessing patterns

## Testing Requirements

- Unit tests for each matching algorithm
- Integration tests with real column name variations
- Performance benchmarks with large column sets
- Accuracy tests against known good/bad matches
- Configuration validation tests

## Acceptance Criteria

- [ ] Implement multiple fuzzy matching algorithms
- [ ] Achieve >95% accuracy on known column variations
- [ ] Process 1000+ column comparisons in <100ms
- [ ] Support configurable confidence thresholds
- [ ] Provide detailed match scoring and explanations
- [ ] Handle Unicode and international characters
- [ ] Pass comprehensive test suite

## Related Tasks

- **Previous:** Load mapping configurations from JSON files
- **Next:** Create column validation against required fields
- **Enables:** Mapping confidence scoring and validation reports

## Notes

- Focus on common variations found in FedRAMP documents
- Consider domain-specific abbreviations and terminology
- Implement comprehensive logging for match debugging
- Support for user feedback to improve matching accuracy
- Consider machine learning approaches for future enhancement
