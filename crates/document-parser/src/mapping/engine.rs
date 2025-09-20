//! Core mapping engine and optimization structures
//!
//! This module contains the main mapping engine logic including the ColumnMapper,
//! OptimizedMappingLookup, and related functionality for efficient column mapping.

use fedramp_core::{Result, Error};

use std::collections::HashMap;
use std::path::Path;
use tracing::{debug, info, warn};
use regex::Regex;
use crate::fuzzy::{FuzzyMatcher, FuzzyMatchConfig, FuzzyMatchResult};
use crate::mapping::config::{ColumnMapping, MappingConfiguration, ValidationRules};
use crate::mapping::inventory::InventoryMappings;
use crate::mapping::poam::{PoamMappings, PoamValidationRules};
use crate::mapping::ssp::SspSections;
use crate::mapping::loader::MappingConfigurationLoader;

/// Optimized lookup structures for fast column mapping
pub struct OptimizedMappingLookup {
    /// Exact match lookup for column names to target fields
    exact_matches: HashMap<String, MappingEntry>,
    /// Normalized column names for fuzzy matching
    fuzzy_candidates: Vec<FuzzyCandidate>,
    /// Validation rules lookup
    validation_rules: HashMap<String, ValidationRule>,
    /// Required fields tracking
    required_fields: std::collections::HashSet<String>,
    /// Advanced fuzzy matcher
    fuzzy_matcher: FuzzyMatcher,
    /// Target strings for fuzzy matching
    fuzzy_targets: Vec<String>,
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
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MappingSourceType {
    Inventory,
    Poam,
    SspSection,
    Control,
    Document,
}

/// Fuzzy matching candidate
#[derive(Debug, Clone)]
pub struct FuzzyCandidate {
    pub original_name: String,
    pub normalized_name: String,
    pub target_field: String,
    pub source_type: MappingSourceType,
    pub required: bool,
}

/// Validation rule for field validation
#[derive(Debug, Clone)]
pub struct ValidationRule {
    pub rule_type: ValidationType,
    pub pattern: Option<Regex>,
    pub allowed_values: Option<Vec<String>>,
}

/// Validation type enumeration
#[derive(Debug, Clone)]
pub enum ValidationType {
    Regex,
    AllowedValues,
    Boolean,
    IpAddress,
    MacAddress,
    Date,
    ControlId,
    UniqueIdentifier,
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
pub struct ColumnMapper {
    /// Legacy mapping configurations (for backward compatibility)
    mappings: HashMap<String, ColumnMapping>,
    /// Optimized lookup structures
    optimized_lookup: Option<OptimizedMappingLookup>,
    /// Minimum confidence threshold for fuzzy matching
    min_confidence: f64,
    /// Configuration loader
    config_loader: Option<MappingConfigurationLoader>,
}

impl OptimizedMappingLookup {
    /// Create optimized lookup structures from mapping configuration
    pub fn from_configuration(config: &MappingConfiguration) -> Result<Self> {
        let mut exact_matches = HashMap::new();
        let mut fuzzy_candidates = Vec::new();
        let mut validation_rules = HashMap::new();
        let mut required_fields = std::collections::HashSet::new();

        // Process inventory mappings
        if let Some(inventory) = &config.inventory_mappings {
            Self::process_inventory_mappings(
                inventory,
                &mut exact_matches,
                &mut fuzzy_candidates,
                &mut validation_rules,
                &mut required_fields,
            )?;
        }

        // Process POA&M mappings
        if let Some(poam) = &config.poam_mappings {
            Self::process_poam_mappings(
                poam,
                &mut exact_matches,
                &mut fuzzy_candidates,
                &mut validation_rules,
                &mut required_fields,
            )?;
        }

        // Process SSP sections
        if let Some(ssp) = &config.ssp_sections {
            Self::process_ssp_mappings(
                ssp,
                &mut exact_matches,
                &mut fuzzy_candidates,
                &mut validation_rules,
                &mut required_fields,
            )?;
        }

        // Create fuzzy targets list for the advanced fuzzy matcher
        let fuzzy_targets: Vec<String> = fuzzy_candidates
            .iter()
            .map(|candidate| candidate.original_name.clone())
            .collect();

        Ok(Self {
            exact_matches,
            fuzzy_candidates,
            validation_rules,
            required_fields,
            fuzzy_matcher: FuzzyMatcher::for_fedramp_columns(),
            fuzzy_targets,
        })
    }

    /// Process inventory mappings into lookup structures
    fn process_inventory_mappings(
        inventory: &InventoryMappings,
        exact_matches: &mut HashMap<String, MappingEntry>,
        fuzzy_candidates: &mut Vec<FuzzyCandidate>,
        validation_rules: &mut HashMap<String, ValidationRule>,
        required_fields: &mut std::collections::HashSet<String>,
    ) -> Result<()> {
        for (_key, mapping) in &inventory.fedramp_iiw_mappings.required_columns {
            let entry = MappingEntry {
                target_field: mapping.field.clone(),
                source_type: MappingSourceType::Inventory,
                required: mapping.required,
                validation: mapping.validation.clone(),
                data_type: None,
            };

            if mapping.required {
                required_fields.insert(mapping.field.clone());
            }

            // Add exact matches
            for column_name in &mapping.column_names {
                let normalized = Self::normalize_column_name(column_name);
                exact_matches.insert(normalized.clone(), entry.clone());

                // Add fuzzy candidate
                fuzzy_candidates.push(FuzzyCandidate {
                    original_name: column_name.clone(),
                    normalized_name: normalized,
                    target_field: mapping.field.clone(),
                    source_type: MappingSourceType::Inventory,
                    required: mapping.required,
                });
            }

            // Add validation rules
            if let Some(validation) = &mapping.validation {
                let rule = Self::create_validation_rule(validation, &inventory.validation_rules)?;
                validation_rules.insert(mapping.field.clone(), rule);
            }
        }

        Ok(())
    }

    /// Process POA&M mappings into lookup structures
    fn process_poam_mappings(
        poam: &PoamMappings,
        exact_matches: &mut HashMap<String, MappingEntry>,
        fuzzy_candidates: &mut Vec<FuzzyCandidate>,
        validation_rules: &mut HashMap<String, ValidationRule>,
        required_fields: &mut std::collections::HashSet<String>,
    ) -> Result<()> {
        for (_key, mapping) in &poam.fedramp_v3_mappings.required_columns {
            let entry = MappingEntry {
                target_field: mapping.oscal_field.clone(),
                source_type: MappingSourceType::Poam,
                required: mapping.required,
                validation: mapping.validation.clone(),
                data_type: None,
            };

            if mapping.required {
                required_fields.insert(mapping.oscal_field.clone());
            }

            // Add exact matches
            for column_name in &mapping.column_names {
                let normalized = Self::normalize_column_name(column_name);
                exact_matches.insert(normalized.clone(), entry.clone());

                // Add fuzzy candidate
                fuzzy_candidates.push(FuzzyCandidate {
                    original_name: column_name.clone(),
                    normalized_name: normalized,
                    target_field: mapping.oscal_field.clone(),
                    source_type: MappingSourceType::Poam,
                    required: mapping.required,
                });
            }

            // Add validation rules
            if let Some(validation) = &mapping.validation {
                let rule = Self::create_poam_validation_rule(validation, &poam.fedramp_v3_mappings.validation_rules)?;
                validation_rules.insert(mapping.oscal_field.clone(), rule);
            }
        }

        Ok(())
    }

    /// Process SSP mappings into lookup structures
    fn process_ssp_mappings(
        ssp: &SspSections,
        exact_matches: &mut HashMap<String, MappingEntry>,
        fuzzy_candidates: &mut Vec<FuzzyCandidate>,
        _validation_rules: &mut HashMap<String, ValidationRule>,
        required_fields: &mut std::collections::HashSet<String>,
    ) -> Result<()> {
        for (_key, mapping) in &ssp.section_mappings.mappings {
            let entry = MappingEntry {
                target_field: mapping.target.clone(),
                source_type: MappingSourceType::SspSection,
                required: mapping.required,
                validation: None,
                data_type: None,
            };

            if mapping.required {
                required_fields.insert(mapping.target.clone());
            }

            // Add keywords as fuzzy candidates
            for keyword in &mapping.keywords {
                let normalized = Self::normalize_column_name(keyword);

                fuzzy_candidates.push(FuzzyCandidate {
                    original_name: keyword.clone(),
                    normalized_name: normalized.clone(),
                    target_field: mapping.target.clone(),
                    source_type: MappingSourceType::SspSection,
                    required: mapping.required,
                });

                // Also add as exact match for efficiency
                exact_matches.insert(normalized, entry.clone());
            }
        }

        Ok(())
    }

    /// Normalize column name for consistent matching
    fn normalize_column_name(name: &str) -> String {
        name.to_lowercase()
            .chars()
            .filter(|c| c.is_alphanumeric())
            .collect()
    }

    /// Create validation rule from inventory validation type
    fn create_validation_rule(
        validation: &str,
        rules: &ValidationRules,
    ) -> Result<ValidationRule> {
        match validation {
            "ip_address" => {
                if let Some(pattern) = &rules.ip_address_pattern {
                    Ok(ValidationRule {
                        rule_type: ValidationType::IpAddress,
                        pattern: Some(Regex::new(pattern).map_err(|e| {
                            Error::document_parsing(format!("Invalid IP address pattern: {}", e))
                        })?),
                        allowed_values: None,
                    })
                } else {
                    Ok(ValidationRule {
                        rule_type: ValidationType::IpAddress,
                        pattern: None,
                        allowed_values: None,
                    })
                }
            }
            "mac_address" => {
                if let Some(pattern) = &rules.mac_address_pattern {
                    Ok(ValidationRule {
                        rule_type: ValidationType::MacAddress,
                        pattern: Some(Regex::new(pattern).map_err(|e| {
                            Error::document_parsing(format!("Invalid MAC address pattern: {}", e))
                        })?),
                        allowed_values: None,
                    })
                } else {
                    Ok(ValidationRule {
                        rule_type: ValidationType::MacAddress,
                        pattern: None,
                        allowed_values: None,
                    })
                }
            }
            "boolean" => Ok(ValidationRule {
                rule_type: ValidationType::Boolean,
                pattern: None,
                allowed_values: rules.boolean_values.clone(),
            }),
            "asset_types" => Ok(ValidationRule {
                rule_type: ValidationType::AllowedValues,
                pattern: None,
                allowed_values: rules.asset_types.clone(),
            }),
            "environments" => Ok(ValidationRule {
                rule_type: ValidationType::AllowedValues,
                pattern: None,
                allowed_values: rules.environments.clone(),
            }),
            "criticality_levels" => Ok(ValidationRule {
                rule_type: ValidationType::AllowedValues,
                pattern: None,
                allowed_values: rules.criticality_levels.clone(),
            }),
            "unique_identifier" => Ok(ValidationRule {
                rule_type: ValidationType::UniqueIdentifier,
                pattern: None,
                allowed_values: None,
            }),
            _ => Ok(ValidationRule {
                rule_type: ValidationType::AllowedValues,
                pattern: None,
                allowed_values: None,
            }),
        }
    }

    /// Create validation rule from POA&M validation type
    fn create_poam_validation_rule(
        validation: &str,
        rules: &PoamValidationRules,
    ) -> Result<ValidationRule> {
        match validation {
            "severity_levels" => Ok(ValidationRule {
                rule_type: ValidationType::AllowedValues,
                pattern: None,
                allowed_values: rules.severity_levels.clone(),
            }),
            "status_values" => Ok(ValidationRule {
                rule_type: ValidationType::AllowedValues,
                pattern: None,
                allowed_values: rules.status_values.clone(),
            }),
            "control_id_list" => {
                if let Some(pattern) = &rules.control_id_pattern {
                    Ok(ValidationRule {
                        rule_type: ValidationType::ControlId,
                        pattern: Some(Regex::new(pattern).map_err(|e| {
                            Error::document_parsing(format!("Invalid control ID pattern: {}", e))
                        })?),
                        allowed_values: None,
                    })
                } else {
                    Ok(ValidationRule {
                        rule_type: ValidationType::ControlId,
                        pattern: None,
                        allowed_values: None,
                    })
                }
            }
            "date" => Ok(ValidationRule {
                rule_type: ValidationType::Date,
                pattern: None,
                allowed_values: rules.date_formats.clone(),
            }),
            "alphanumeric" => Ok(ValidationRule {
                rule_type: ValidationType::Regex,
                pattern: Some(Regex::new(r"^[a-zA-Z0-9]+$").unwrap()),
                allowed_values: None,
            }),
            _ => Ok(ValidationRule {
                rule_type: ValidationType::AllowedValues,
                pattern: None,
                allowed_values: None,
            }),
        }
    }

    /// Find exact match for column name
    pub fn find_exact_match(&self, column_name: &str) -> Option<&MappingEntry> {
        let normalized = Self::normalize_column_name(column_name);
        self.exact_matches.get(&normalized)
    }

    /// Find best fuzzy match for column name using advanced fuzzy matching
    pub fn find_fuzzy_match(&mut self, column_name: &str, min_confidence: f64) -> Option<MappingResult> {
        // Use the advanced fuzzy matcher
        let matches = self.fuzzy_matcher.find_matches(column_name, &self.fuzzy_targets);

        if let Some(best_match) = matches.first() {
            if best_match.confidence >= min_confidence {
                // Find the corresponding candidate to get the target field
                if let Some(candidate) = self.fuzzy_candidates
                    .iter()
                    .find(|c| c.original_name == best_match.target) {

                    return Some(MappingResult {
                        source_column: column_name.to_string(),
                        target_field: candidate.target_field.clone(),
                        confidence: best_match.confidence,
                        exact_match: best_match.exact_match,
                    });
                }
            }
        }

        // Fallback to legacy fuzzy matching for backward compatibility
        self.find_fuzzy_match_legacy(column_name, min_confidence)
    }

    /// Legacy fuzzy matching implementation for backward compatibility
    fn find_fuzzy_match_legacy(&self, column_name: &str, min_confidence: f64) -> Option<MappingResult> {
        let normalized = Self::normalize_column_name(column_name);
        let mut best_match: Option<&FuzzyCandidate> = None;
        let mut best_confidence = 0.0;

        for candidate in &self.fuzzy_candidates {
            let confidence = self.calculate_similarity(&normalized, &candidate.normalized_name);
            if confidence >= min_confidence && confidence > best_confidence {
                best_confidence = confidence;
                best_match = Some(candidate);
            }
        }

        best_match.map(|candidate| MappingResult {
            source_column: column_name.to_string(),
            target_field: candidate.target_field.clone(),
            confidence: best_confidence,
            exact_match: false,
        })
    }

    /// Calculate similarity between two normalized strings
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

    /// Get validation rule for a field
    pub fn get_validation_rule(&self, field_name: &str) -> Option<&ValidationRule> {
        self.validation_rules.get(field_name)
    }

    /// Check if field is required
    pub fn is_required_field(&self, field_name: &str) -> bool {
        self.required_fields.contains(field_name)
    }

    /// Get all required fields
    pub fn get_required_fields(&self) -> &std::collections::HashSet<String> {
        &self.required_fields
    }

    /// Get statistics about the lookup structures
    pub fn get_statistics(&self) -> serde_json::Value {
        let source_type_counts = self.fuzzy_candidates
            .iter()
            .fold(HashMap::new(), |mut acc, candidate| {
                *acc.entry(format!("{:?}", candidate.source_type)).or_insert(0) += 1;
                acc
            });

        let (cache_size, cache_capacity) = self.fuzzy_matcher.cache_stats();

        serde_json::json!({
            "exact_matches": self.exact_matches.len(),
            "fuzzy_candidates": self.fuzzy_candidates.len(),
            "validation_rules": self.validation_rules.len(),
            "required_fields": self.required_fields.len(),
            "source_type_distribution": source_type_counts,
            "fuzzy_cache_size": cache_size,
            "fuzzy_cache_capacity": cache_capacity
        })
    }

    /// Update fuzzy matching configuration
    pub fn update_fuzzy_config(&mut self, config: FuzzyMatchConfig) {
        self.fuzzy_matcher.update_config(config);
    }

    /// Clear fuzzy matching cache
    pub fn clear_fuzzy_cache(&mut self) {
        self.fuzzy_matcher.clear_cache();
    }

    /// Get detailed fuzzy match results with algorithm breakdown
    pub fn get_detailed_fuzzy_matches(&mut self, column_name: &str, min_confidence: f64) -> Vec<FuzzyMatchResult> {
        self.fuzzy_matcher.find_matches(column_name, &self.fuzzy_targets)
            .into_iter()
            .filter(|result| result.confidence >= min_confidence)
            .collect()
    }
}

impl ColumnMapper {
    /// Create a new column mapper
    #[must_use]
    pub fn new() -> Self {
        Self {
            mappings: HashMap::new(),
            optimized_lookup: None,
            min_confidence: 0.7,
            config_loader: None,
        }
    }

    /// Create a new column mapper with custom confidence threshold
    #[must_use]
    pub fn with_confidence_threshold(min_confidence: f64) -> Self {
        Self {
            mappings: HashMap::new(),
            optimized_lookup: None,
            min_confidence,
            config_loader: None,
        }
    }

    /// Create a new column mapper with configuration loader
    pub fn with_config_loader<P: AsRef<Path>>(base_dir: P) -> Self {
        Self {
            mappings: HashMap::new(),
            optimized_lookup: None,
            min_confidence: 0.7,
            config_loader: Some(MappingConfigurationLoader::new(base_dir)),
        }
    }

    /// Load mapping configurations from JSON files
    pub async fn load_configurations(&mut self) -> Result<()> {
        if let Some(loader) = &mut self.config_loader {
            let config = loader.load_all_configurations().await?;

            // Validate configuration
            let warnings = loader.validate_configuration(&config)?;
            if !warnings.is_empty() {
                warn!("Configuration validation warnings: {:?}", warnings);
            }

            // Create optimized lookup structures
            self.optimized_lookup = Some(OptimizedMappingLookup::from_configuration(&config)?);

            info!("Successfully loaded and optimized mapping configurations");
            Ok(())
        } else {
            Err(Error::document_parsing("No configuration loader available"))
        }
    }

    /// Load specific mapping configuration from file
    pub async fn load_configuration_from_file<P: AsRef<Path>>(&mut self, path: P) -> Result<()> {
        if self.config_loader.is_none() {
            self.config_loader = Some(MappingConfigurationLoader::new("."));
        }

        if let Some(loader) = &mut self.config_loader {
            let config: MappingConfiguration = loader.load_from_path(path).await?;
            self.optimized_lookup = Some(OptimizedMappingLookup::from_configuration(&config)?);
            info!("Successfully loaded configuration from file");
            Ok(())
        } else {
            Err(Error::document_parsing("Failed to initialize configuration loader"))
        }
    }

    /// Load mapping configuration from JSON
    pub fn load_mappings(&mut self, mappings: HashMap<String, ColumnMapping>) -> Result<()> {
        info!("Loading {} column mappings", mappings.len());
        self.mappings = mappings;
        Ok(())
    }

    /// Map column headers to OSCAL fields
    pub fn map_columns(&mut self, headers: &[String]) -> Result<Vec<MappingResult>> {
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
    fn find_best_match(&mut self, header: &str) -> Result<Option<MappingResult>> {
        // Try optimized lookup first if available
        if let Some(lookup) = &mut self.optimized_lookup {
            // Check for exact match
            if let Some(entry) = lookup.find_exact_match(header) {
                return Ok(Some(MappingResult {
                    source_column: header.to_string(),
                    target_field: entry.target_field.clone(),
                    confidence: 1.0,
                    exact_match: true,
                }));
            }

            // Try fuzzy match
            if let Some(fuzzy_result) = lookup.find_fuzzy_match(header, self.min_confidence) {
                return Ok(Some(fuzzy_result));
            }

            return Ok(None);
        }

        // Fall back to legacy mapping logic for backward compatibility
        self.find_best_match_legacy(header)
    }

    /// Legacy mapping logic for backward compatibility
    fn find_best_match_legacy(&self, header: &str) -> Result<Option<MappingResult>> {
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

        // Use optimized lookup if available
        if let Some(lookup) = &self.optimized_lookup {
            for required_field in lookup.get_required_fields() {
                if !mapped_fields.contains(required_field) {
                    errors.push(format!("Required field '{}' not found in document", required_field));
                }
            }
        } else {
            // Fall back to legacy validation
            for (target_field, mapping) in &self.mappings {
                if mapping.required && !mapped_fields.contains(target_field) {
                    errors.push(format!("Required field '{}' not found in document", target_field));
                }
            }
        }

        Ok(errors)
    }

    /// Validate field value using configured validation rules
    pub fn validate_field_value(&self, field_name: &str, value: &str) -> Result<Vec<String>> {
        let mut errors = Vec::new();

        if let Some(lookup) = &self.optimized_lookup {
            if let Some(rule) = lookup.get_validation_rule(field_name) {
                match &rule.rule_type {
                    ValidationType::Regex => {
                        if let Some(pattern) = &rule.pattern {
                            if !pattern.is_match(value) {
                                errors.push(format!("Field '{}' value '{}' does not match required pattern", field_name, value));
                            }
                        }
                    }
                    ValidationType::AllowedValues => {
                        if let Some(allowed) = &rule.allowed_values {
                            if !allowed.contains(&value.to_string()) {
                                errors.push(format!("Field '{}' value '{}' is not in allowed values: {:?}", field_name, value, allowed));
                            }
                        }
                    }
                    ValidationType::Boolean => {
                        let normalized = value.to_lowercase();
                        if !["true", "false", "yes", "no", "y", "n", "1", "0"].contains(&normalized.as_str()) {
                            errors.push(format!("Field '{}' value '{}' is not a valid boolean", field_name, value));
                        }
                    }
                    ValidationType::IpAddress => {
                        if let Some(pattern) = &rule.pattern {
                            if !pattern.is_match(value) {
                                errors.push(format!("Field '{}' value '{}' is not a valid IP address", field_name, value));
                            }
                        }
                    }
                    ValidationType::MacAddress => {
                        if let Some(pattern) = &rule.pattern {
                            if !pattern.is_match(value) {
                                errors.push(format!("Field '{}' value '{}' is not a valid MAC address", field_name, value));
                            }
                        }
                    }
                    ValidationType::Date => {
                        // Basic date validation - could be enhanced with chrono parsing
                        if value.is_empty() {
                            errors.push(format!("Field '{}' date value cannot be empty", field_name));
                        }
                    }
                    ValidationType::ControlId => {
                        if let Some(pattern) = &rule.pattern {
                            if !pattern.is_match(value) {
                                errors.push(format!("Field '{}' value '{}' is not a valid control ID", field_name, value));
                            }
                        }
                    }
                    ValidationType::UniqueIdentifier => {
                        if value.is_empty() {
                            errors.push(format!("Field '{}' unique identifier cannot be empty", field_name));
                        }
                    }
                }
            }
        }

        Ok(errors)
    }

    /// Get mapping statistics
    pub fn get_mapping_statistics(&self) -> serde_json::Value {
        if let Some(lookup) = &self.optimized_lookup {
            lookup.get_statistics()
        } else {
            serde_json::json!({
                "legacy_mappings": self.mappings.len(),
                "optimized_lookup": false
            })
        }
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;
    use crate::mapping::loader::MappingConfigurationLoader;

    /// Create a temporary directory with test mapping files
    async fn create_test_mappings_dir() -> Result<TempDir> {
        let temp_dir = TempDir::new().unwrap();
        let mappings_dir = temp_dir.path().join("mappings");

        fs::create_dir_all(&mappings_dir).unwrap();

        // Create test inventory mappings
        let inventory_json = serde_json::json!({
            "description": "Test inventory mappings",
            "version": "1.0",
            "fedramp_iiw_mappings": {
                "required_columns": {
                    "asset_id": {
                        "column_names": ["Asset ID", "Component ID"],
                        "field": "uuid",
                        "required": true,
                        "validation": "unique_identifier"
                    }
                }
            },
            "validation_rules": {
                "asset_types": ["hardware", "software"],
                "boolean_values": ["yes", "no"]
            },
            "component_grouping": {
                "strategies": {}
            },
            "component_type_mappings": {},
            "security_mappings": {
                "criticality_to_impact": {},
                "risk_factors": {}
            },
            "control_inheritance": {
                "infrastructure_controls": [],
                "platform_controls": [],
                "inheritance_mappings": {}
            }
        });

        fs::write(
            mappings_dir.join("inventory_mappings.json"),
            serde_json::to_string_pretty(&inventory_json).unwrap(),
        ).unwrap();

        Ok(temp_dir)
    }

    #[tokio::test]
    async fn test_optimized_lookup_creation() {
        let temp_dir = create_test_mappings_dir().await.unwrap();
        let mut loader = MappingConfigurationLoader::new(temp_dir.path());
        let config = loader.load_all_configurations().await.unwrap();

        let lookup = OptimizedMappingLookup::from_configuration(&config);
        assert!(lookup.is_ok());

        let lookup = lookup.unwrap();
        let stats = lookup.get_statistics();
        assert!(stats["exact_matches"].as_u64().unwrap() > 0);
    }

    #[tokio::test]
    async fn test_column_mapping_with_optimized_lookup() {
        let temp_dir = create_test_mappings_dir().await.unwrap();
        let mut mapper = ColumnMapper::with_config_loader(temp_dir.path());

        let result = mapper.load_configurations().await;
        assert!(result.is_ok());

        let headers = vec!["Asset ID".to_string()];
        let mapping_results = mapper.map_columns(&headers).unwrap();

        assert_eq!(mapping_results.len(), 1);
        assert!(mapping_results.iter().any(|r| r.target_field == "uuid"));
    }

    #[tokio::test]
    async fn test_fuzzy_matching() {
        let temp_dir = create_test_mappings_dir().await.unwrap();
        let mut mapper = ColumnMapper::with_config_loader(temp_dir.path());
        mapper.load_configurations().await.unwrap();

        // Test fuzzy matching with slight variations
        let headers = vec!["Asset_ID".to_string(), "Component_ID".to_string()];
        let mapping_results = mapper.map_columns(&headers).unwrap();

        assert!(!mapping_results.is_empty());
        assert!(mapping_results.iter().any(|r| r.confidence > 0.7));
    }
}
