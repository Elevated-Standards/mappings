// Modified: 2025-01-20

//! Column mapping and field detection for document parsing
//!
//! This module provides functionality to map document columns to OSCAL fields
//! using fuzzy matching and configuration-based mapping rules.

use fedramp_core::{Result, Error};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tracing::{debug, info, warn};

/// Column mapping configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ColumnMapping {
    /// Target OSCAL field name
    pub target_field: String,
    /// Possible column names to match
    pub source_columns: Vec<String>,
    /// Whether this field is required
    pub required: bool,
    /// Data type validation
    pub data_type: Option<String>,
    /// Default value if not found
    pub default_value: Option<serde_json::Value>,
}

/// Column mapping result with confidence score
#[derive(Debug, Clone)]
pub struct MappingResult {
    /// Source column name that was matched
    pub source_column: String,
    /// Target OSCAL field name
    pub target_field: String,
    /// Confidence score (0.0 to 1.0)
    pub confidence: f64,
    /// Whether this was an exact match
    pub exact_match: bool,
}

/// Column mapper for detecting and mapping document columns
#[derive(Debug, Clone)]
pub struct ColumnMapper {
    /// Mapping configurations loaded from JSON files
    mappings: HashMap<String, ColumnMapping>,
    /// Minimum confidence threshold for fuzzy matching
    min_confidence: f64,
}

impl ColumnMapper {
    /// Create a new column mapper
    #[must_use]
    pub fn new() -> Self {
        Self {
            mappings: HashMap::new(),
            min_confidence: 0.7,
        }
    }

    /// Create a new column mapper with custom confidence threshold
    #[must_use]
    pub fn with_confidence_threshold(min_confidence: f64) -> Self {
        Self {
            mappings: HashMap::new(),
            min_confidence,
        }
    }

    /// Load mapping configuration from JSON
    pub fn load_mappings(&mut self, mappings: HashMap<String, ColumnMapping>) -> Result<()> {
        info!("Loading {} column mappings", mappings.len());
        self.mappings = mappings;
        Ok(())
    }

    /// Map column headers to OSCAL fields
    pub fn map_columns(&self, headers: &[String]) -> Result<Vec<MappingResult>> {
        debug!("Mapping {} column headers", headers.len());
        
        let mut results = Vec::new();
        
        for header in headers {
            if let Some(mapping_result) = self.find_best_match(header)? {
                results.push(mapping_result);
            } else {
                warn!("No mapping found for column: {}", header);
            }
        }
        
        info!("Successfully mapped {}/{} columns", results.len(), headers.len());
        Ok(results)
    }

    /// Find the best mapping match for a column header
    fn find_best_match(&self, header: &str) -> Result<Option<MappingResult>> {
        let header_normalized = self.normalize_string(header);
        let mut best_match: Option<MappingResult> = None;
        let mut best_confidence = 0.0;

        for (target_field, mapping) in &self.mappings {
            for source_column in &mapping.source_columns {
                let source_normalized = self.normalize_string(source_column);
                
                // Check for exact match first
                if header_normalized == source_normalized {
                    return Ok(Some(MappingResult {
                        source_column: header.to_string(),
                        target_field: target_field.clone(),
                        confidence: 1.0,
                        exact_match: true,
                    }));
                }

                // Calculate fuzzy match confidence
                let confidence = self.calculate_similarity(&header_normalized, &source_normalized);
                
                if confidence >= self.min_confidence && confidence > best_confidence {
                    best_confidence = confidence;
                    best_match = Some(MappingResult {
                        source_column: header.to_string(),
                        target_field: target_field.clone(),
                        confidence,
                        exact_match: false,
                    });
                }
            }
        }

        Ok(best_match)
    }

    /// Normalize string for comparison
    fn normalize_string(&self, s: &str) -> String {
        s.to_lowercase()
            .chars()
            .filter(|c| c.is_alphanumeric())
            .collect()
    }

    /// Calculate similarity between two strings using Levenshtein distance
    fn calculate_similarity(&self, s1: &str, s2: &str) -> f64 {
        if s1 == s2 {
            return 1.0;
        }

        let len1 = s1.len();
        let len2 = s2.len();

        if len1 == 0 || len2 == 0 {
            return 0.0;
        }

        let distance = self.levenshtein_distance(s1, s2);
        let max_len = len1.max(len2);
        
        1.0 - (distance as f64 / max_len as f64)
    }

    /// Calculate Levenshtein distance between two strings
    fn levenshtein_distance(&self, s1: &str, s2: &str) -> usize {
        let chars1: Vec<char> = s1.chars().collect();
        let chars2: Vec<char> = s2.chars().collect();
        let len1 = chars1.len();
        let len2 = chars2.len();

        let mut matrix = vec![vec![0; len2 + 1]; len1 + 1];

        // Initialize first row and column
        for i in 0..=len1 {
            matrix[i][0] = i;
        }
        for j in 0..=len2 {
            matrix[0][j] = j;
        }

        // Fill the matrix
        for i in 1..=len1 {
            for j in 1..=len2 {
                let cost = if chars1[i - 1] == chars2[j - 1] { 0 } else { 1 };
                matrix[i][j] = (matrix[i - 1][j] + 1)
                    .min(matrix[i][j - 1] + 1)
                    .min(matrix[i - 1][j - 1] + cost);
            }
        }

        matrix[len1][len2]
    }

    /// Validate that all required fields have mappings
    pub fn validate_required_mappings(&self, mapping_results: &[MappingResult]) -> Result<Vec<String>> {
        let mut errors = Vec::new();
        let mapped_fields: std::collections::HashSet<_> = 
            mapping_results.iter().map(|r| &r.target_field).collect();

        for (target_field, mapping) in &self.mappings {
            if mapping.required && !mapped_fields.contains(target_field) {
                errors.push(format!("Required field '{}' not found in document", target_field));
            }
        }

        Ok(errors)
    }

    /// Generate mapping confidence report
    pub fn generate_mapping_report(&self, mapping_results: &[MappingResult]) -> serde_json::Value {
        let total_mappings = mapping_results.len();
        let exact_matches = mapping_results.iter().filter(|r| r.exact_match).count();
        let avg_confidence = if total_mappings > 0 {
            mapping_results.iter().map(|r| r.confidence).sum::<f64>() / total_mappings as f64
        } else {
            0.0
        };

        serde_json::json!({
            "total_mappings": total_mappings,
            "exact_matches": exact_matches,
            "fuzzy_matches": total_mappings - exact_matches,
            "average_confidence": avg_confidence,
            "mappings": mapping_results.iter().map(|r| serde_json::json!({
                "source_column": r.source_column,
                "target_field": r.target_field,
                "confidence": r.confidence,
                "exact_match": r.exact_match
            })).collect::<Vec<_>>()
        })
    }
}

impl Default for ColumnMapper {
    fn default() -> Self {
        Self::new()
    }
}
