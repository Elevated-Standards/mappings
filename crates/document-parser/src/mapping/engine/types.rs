//! Core types and data structures for the mapping engine
//! Modified: 2025-01-22
//!
//! This module contains all the type definitions, enums, and data structures
//! used throughout the mapping engine for column mapping and validation.

use std::collections::{HashMap, HashSet};
use regex::Regex;
use crate::fuzzy::FuzzyMatcher;

/// Optimized lookup structures for fast column mapping
pub struct OptimizedMappingLookup {
    /// Exact match lookup for column names to target fields
    pub exact_matches: HashMap<String, MappingEntry>,
    /// Normalized column names for fuzzy matching
    pub fuzzy_candidates: Vec<FuzzyCandidate>,
    /// Validation rules lookup
    pub validation_rules: HashMap<String, ValidationRule>,
    /// Required fields tracking
    pub required_fields: HashSet<String>,
    /// Advanced fuzzy matcher
    pub fuzzy_matcher: FuzzyMatcher,
    /// Target strings for fuzzy matching
    pub fuzzy_targets: Vec<String>,
}

/// Mapping entry for lookup results
#[derive(Debug, Clone)]
pub struct MappingEntry {
    pub target_field: String,
    pub source_type: MappingSourceType,
    pub required: bool,
    pub validation: Option<String>,
    pub data_type: Option<String>,
}

/// Source type for mapping entries
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum MappingSourceType {
    Inventory,
    Poam,
    SspSection,
    Custom,
}

/// Fuzzy candidate for column matching
#[derive(Debug, Clone)]
pub struct FuzzyCandidate {
    pub original_name: String,
    pub normalized_name: String,
    pub target_field: String,
    pub source_type: MappingSourceType,
    pub required: bool,
}

/// Validation rule for column data
#[derive(Debug, Clone)]
pub struct ValidationRule {
    pub rule_type: ValidationType,
    pub pattern: Option<Regex>,
    pub allowed_values: Option<Vec<String>>,
    pub required: bool,
}

/// Type of validation to perform
#[derive(Debug, Clone)]
pub enum ValidationType {
    Regex,
    AllowedValues,
    Boolean,
    Numeric,
    Date,
    Email,
    Url,
    Custom(String),
}

/// Result of column mapping operation
#[derive(Debug, Clone)]
pub struct MappingResult {
    /// Source column name that was matched
    pub source_column: String,
    /// Target OSCAL field name
    pub target_field: String,
    /// Confidence score (0.0 to 1.0)
    pub confidence: f64,
    /// Source type of the mapping
    pub source_type: MappingSourceType,
    /// Whether this field is required
    pub required: bool,
    /// Validation rule if applicable
    pub validation: Option<String>,
    /// Whether this was an exact match
    pub exact_match: bool,
}

/// Column mapper for detecting and mapping document columns
pub struct ColumnMapper {
    /// Legacy mapping configurations (for backward compatibility)
    pub mappings: HashMap<String, crate::mapping::config::ColumnMapping>,
    /// Optimized lookup structures
    pub optimized_lookup: Option<OptimizedMappingLookup>,
    /// Minimum confidence threshold for fuzzy matching
    pub min_confidence: f64,
    /// Configuration loader
    pub config_loader: Option<crate::mapping::loader::MappingConfigurationLoader>,
}

/// Statistics about the mapping lookup structures
#[derive(Debug, Clone)]
pub struct MappingStatistics {
    pub exact_matches_count: usize,
    pub fuzzy_candidates_count: usize,
    pub validation_rules_count: usize,
    pub required_fields_count: usize,
    pub source_type_breakdown: HashMap<MappingSourceType, usize>,
}

/// Configuration for mapping engine behavior
#[derive(Debug, Clone)]
pub struct MappingEngineConfig {
    /// Minimum confidence threshold for fuzzy matching
    pub min_confidence: f64,
    /// Maximum number of fuzzy match results to return
    pub max_fuzzy_results: usize,
    /// Whether to enable caching for performance
    pub enable_caching: bool,
    /// Whether to normalize column names for matching
    pub normalize_column_names: bool,
    /// Custom normalization rules
    pub normalization_rules: Vec<NormalizationRule>,
}

/// Rule for normalizing column names
#[derive(Debug, Clone)]
pub struct NormalizationRule {
    pub pattern: Regex,
    pub replacement: String,
    pub description: String,
}

impl Default for MappingEngineConfig {
    fn default() -> Self {
        Self {
            min_confidence: 0.7,
            max_fuzzy_results: 10,
            enable_caching: true,
            normalize_column_names: true,
            normalization_rules: Vec::new(),
        }
    }
}

impl MappingSourceType {
    /// Get a human-readable description of the source type
    pub fn description(&self) -> &'static str {
        match self {
            MappingSourceType::Inventory => "FedRAMP Inventory and Implementation Worksheet",
            MappingSourceType::Poam => "Plan of Action and Milestones",
            MappingSourceType::SspSection => "System Security Plan Section",
            MappingSourceType::Custom => "Custom Mapping Configuration",
        }
    }

    /// Get the priority order for this source type (lower = higher priority)
    pub fn priority(&self) -> u8 {
        match self {
            MappingSourceType::Poam => 1,
            MappingSourceType::Inventory => 2,
            MappingSourceType::SspSection => 3,
            MappingSourceType::Custom => 4,
        }
    }
}

impl ValidationType {
    /// Check if this validation type requires a pattern
    pub fn requires_pattern(&self) -> bool {
        matches!(self, ValidationType::Regex | ValidationType::Custom(_))
    }

    /// Check if this validation type requires allowed values
    pub fn requires_allowed_values(&self) -> bool {
        matches!(self, ValidationType::AllowedValues)
    }

    /// Get a description of this validation type
    pub fn description(&self) -> String {
        match self {
            ValidationType::Regex => "Regular expression pattern validation".to_string(),
            ValidationType::AllowedValues => "Must match one of the allowed values".to_string(),
            ValidationType::Boolean => "Must be a boolean value (true/false, yes/no, 1/0)".to_string(),
            ValidationType::Numeric => "Must be a numeric value".to_string(),
            ValidationType::Date => "Must be a valid date format".to_string(),
            ValidationType::Email => "Must be a valid email address".to_string(),
            ValidationType::Url => "Must be a valid URL".to_string(),
            ValidationType::Custom(name) => format!("Custom validation: {}", name),
        }
    }
}

impl MappingResult {
    /// Create a new mapping result
    pub fn new(
        source_column: String,
        target_field: String,
        confidence: f64,
        source_type: MappingSourceType,
        required: bool,
        validation: Option<String>,
        exact_match: bool,
    ) -> Self {
        Self {
            source_column,
            target_field,
            confidence,
            source_type,
            required,
            validation,
            exact_match,
        }
    }

    /// Check if this mapping result meets the minimum confidence threshold
    pub fn meets_threshold(&self, min_confidence: f64) -> bool {
        self.confidence >= min_confidence
    }

    /// Get a quality score based on confidence and other factors
    pub fn quality_score(&self) -> f64 {
        let mut score = self.confidence;
        
        // Boost score for exact matches
        if self.confidence >= 1.0 {
            score += 0.1;
        }
        
        // Boost score for required fields
        if self.required {
            score += 0.05;
        }
        
        // Boost score based on source type priority
        let priority_boost = match self.source_type.priority() {
            1 => 0.05,
            2 => 0.03,
            3 => 0.01,
            _ => 0.0,
        };
        score += priority_boost;
        
        score.min(1.0)
    }
}

impl Default for ColumnMapper {
    fn default() -> Self {
        Self::new()
    }
}
