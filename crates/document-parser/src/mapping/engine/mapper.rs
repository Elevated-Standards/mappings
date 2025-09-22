//! Column mapper implementation for detecting and mapping document columns
//! Modified: 2025-01-22
//!
//! This module contains the main ColumnMapper implementation that provides
//! the primary interface for column mapping operations.

use fedramp_core::{Result, Error};
use std::collections::HashMap;
use std::path::Path;
use tracing::{debug, info, warn};

use crate::mapping::config::ColumnMapping;
use crate::mapping::loader::MappingConfigurationLoader;

use super::types::{ColumnMapper, OptimizedMappingLookup, MappingResult, MappingEngineConfig};
use super::lookup::*;

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

    /// Create a new column mapper with custom configuration
    pub fn with_config(config: MappingEngineConfig) -> Self {
        Self {
            mappings: HashMap::new(),
            optimized_lookup: None,
            min_confidence: config.min_confidence,
            config_loader: None,
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
    pub async fn load_configuration_from_file<P: AsRef<Path>>(&mut self, _path: P) -> Result<()> {
        if self.config_loader.is_none() {
            self.config_loader = Some(MappingConfigurationLoader::new("."));
        }

        // Load the specific configuration file
        // This is a simplified implementation - in practice you'd load the specific file
        self.load_configurations().await
    }

    /// Add a custom column mapping
    pub fn add_mapping(&mut self, source_column: String, mapping: ColumnMapping) {
        self.mappings.insert(source_column, mapping);
    }

    /// Remove a column mapping
    pub fn remove_mapping(&mut self, source_column: &str) -> Option<ColumnMapping> {
        self.mappings.remove(source_column)
    }

    /// Map column headers to target fields
    pub fn map_columns(&mut self, headers: &[String]) -> Result<Vec<MappingResult>> {
        let mut results = Vec::new();

        for header in headers {
            debug!("Mapping column: {}", header);

            // Try optimized lookup first
            if let Some(lookup) = &mut self.optimized_lookup {
                // Try exact match first
                if let Some(entry) = lookup.find_exact_match(header) {
                    results.push(MappingResult {
                        source_column: header.clone(),
                        target_field: entry.target_field.clone(),
                        confidence: 1.0, // Exact match
                        source_type: entry.source_type.clone(),
                        required: entry.required,
                        validation: entry.validation.clone(),
                        exact_match: true,
                    });
                    continue;
                }

                // Try fuzzy matching
                let fuzzy_matches = lookup.find_fuzzy_matches(header, self.min_confidence);
                if !fuzzy_matches.is_empty() {
                    // Take the best match
                    if let Some(best_match) = fuzzy_matches.into_iter().max_by(|a, b| {
                        a.confidence.partial_cmp(&b.confidence).unwrap_or(std::cmp::Ordering::Equal)
                    }) {
                        results.push(best_match);
                        continue;
                    }
                }
            }

            // Fall back to legacy mapping if no optimized lookup or no match found
            if let Some(mapping) = self.mappings.get(header) {
                results.push(MappingResult {
                    source_column: header.clone(),
                    target_field: mapping.target_field.clone(),
                    confidence: 1.0, // Exact match from legacy mapping
                    source_type: super::types::MappingSourceType::Custom,
                    required: mapping.required,
                    validation: mapping.data_type.clone(),
                    exact_match: true,
                });
            } else {
                debug!("No mapping found for column: {}", header);
            }
        }

        Ok(results)
    }

    /// Get mapping suggestions for a column name
    pub fn get_mapping_suggestions(&mut self, column_name: &str, max_suggestions: usize) -> Result<Vec<MappingResult>> {
        if let Some(lookup) = &mut self.optimized_lookup {
            let mut suggestions = lookup.find_fuzzy_matches(column_name, 0.3); // Lower threshold for suggestions
            
            // Sort by confidence (highest first)
            suggestions.sort_by(|a, b| b.confidence.partial_cmp(&a.confidence).unwrap_or(std::cmp::Ordering::Equal));
            
            // Limit to max_suggestions
            suggestions.truncate(max_suggestions);
            
            Ok(suggestions)
        } else {
            Ok(Vec::new())
        }
    }

    /// Validate mapped data against validation rules
    pub fn validate_mapped_data(&self, field_name: &str, value: &str) -> Result<bool> {
        if let Some(lookup) = &self.optimized_lookup {
            if let Some(rule) = lookup.get_validation_rule(field_name) {
                return self.apply_validation_rule(rule, value);
            }
        }

        // No validation rule found, consider valid
        Ok(true)
    }

    /// Apply a validation rule to a value
    fn apply_validation_rule(&self, rule: &super::types::ValidationRule, value: &str) -> Result<bool> {
        use super::types::ValidationType;

        match &rule.rule_type {
            ValidationType::Boolean => {
                let normalized = value.to_lowercase();
                Ok(matches!(normalized.as_str(), "true" | "false" | "yes" | "no" | "1" | "0"))
            }
            ValidationType::Numeric => {
                Ok(value.parse::<f64>().is_ok())
            }
            ValidationType::Date => {
                // Simple date validation - in practice you'd use a proper date parser
                Ok(value.contains('-') || value.contains('/'))
            }
            ValidationType::Email => {
                Ok(value.contains('@') && value.contains('.'))
            }
            ValidationType::Url => {
                Ok(value.starts_with("http://") || value.starts_with("https://"))
            }
            ValidationType::Regex => {
                if let Some(pattern) = &rule.pattern {
                    Ok(pattern.is_match(value))
                } else {
                    Ok(true) // No pattern to validate against
                }
            }
            ValidationType::AllowedValues => {
                if let Some(allowed) = &rule.allowed_values {
                    Ok(allowed.contains(&value.to_string()))
                } else {
                    Ok(true) // No allowed values to validate against
                }
            }
            ValidationType::Custom(_) => {
                // Custom validation would be implemented based on the specific rule
                Ok(true)
            }
        }
    }

    /// Get statistics about the current mapping configuration
    pub fn get_statistics(&self) -> serde_json::Value {
        if let Some(lookup) = &self.optimized_lookup {
            lookup.get_statistics()
        } else {
            serde_json::json!({
                "legacy_mappings": self.mappings.len(),
                "optimized_lookup": false
            })
        }
    }

    /// Clear caches to free memory
    pub fn clear_caches(&mut self) {
        if let Some(lookup) = &mut self.optimized_lookup {
            lookup.clear_cache();
        }
    }

    /// Check if a field is required based on current configuration
    pub fn is_required_field(&self, field_name: &str) -> bool {
        if let Some(lookup) = &self.optimized_lookup {
            lookup.is_required_field(field_name)
        } else {
            // Check legacy mappings
            self.mappings.values().any(|mapping| mapping.target_field == field_name && mapping.required)
        }
    }

    /// Get all required fields from current configuration
    pub fn get_required_fields(&self) -> Vec<String> {
        if let Some(lookup) = &self.optimized_lookup {
            lookup.required_fields.iter().cloned().collect()
        } else {
            // Get from legacy mappings
            self.mappings
                .values()
                .filter(|mapping| mapping.required)
                .map(|mapping| mapping.target_field.clone())
                .collect()
        }
    }

    /// Get the current minimum confidence threshold
    pub fn get_min_confidence(&self) -> f64 {
        self.min_confidence
    }

    /// Set the minimum confidence threshold
    pub fn set_min_confidence(&mut self, min_confidence: f64) {
        self.min_confidence = min_confidence.clamp(0.0, 1.0);
    }

    /// Check if optimized lookup is available
    pub fn has_optimized_lookup(&self) -> bool {
        self.optimized_lookup.is_some()
    }

    /// Get the number of configured mappings
    pub fn mapping_count(&self) -> usize {
        if let Some(lookup) = &self.optimized_lookup {
            lookup.exact_matches.len()
        } else {
            self.mappings.len()
        }
    }
}
