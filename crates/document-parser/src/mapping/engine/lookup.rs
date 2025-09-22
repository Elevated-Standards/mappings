//! Optimized lookup functionality for fast column mapping
//! Modified: 2025-01-22
//!
//! This module contains the implementation of OptimizedMappingLookup which provides
//! efficient exact and fuzzy matching for column names to target fields.

use fedramp_core::{Result, Error};
use std::collections::{HashMap, HashSet};
use tracing::{debug, info};
use serde_json::Value;

use crate::fuzzy::{FuzzyMatcher, FuzzyMatchResult};
use crate::mapping::config::MappingConfiguration;
use crate::mapping::inventory::InventoryMappings;
use crate::mapping::poam::{PoamMappings, PoamValidationRules};
use crate::mapping::ssp::SspSections;

use super::types::{
    OptimizedMappingLookup, MappingEntry, MappingSourceType, FuzzyCandidate,
    ValidationRule, ValidationType, MappingResult, MappingStatistics
};

impl OptimizedMappingLookup {
    /// Create optimized lookup structures from mapping configuration
    pub fn from_configuration(config: &MappingConfiguration) -> Result<Self> {
        let mut exact_matches = HashMap::new();
        let mut fuzzy_candidates = Vec::new();
        let mut validation_rules = HashMap::new();
        let mut required_fields = HashSet::new();

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
        required_fields: &mut HashSet<String>,
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

            // Add validation rule if present
            if let Some(validation) = &mapping.validation {
                let rule = ValidationRule {
                    rule_type: Self::parse_validation_type(validation),
                    pattern: None, // TODO: Parse regex patterns from validation string
                    allowed_values: None, // TODO: Parse allowed values from validation string
                    required: mapping.required,
                };
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
        required_fields: &mut HashSet<String>,
    ) -> Result<()> {
        for (_key, mapping) in &poam.fedramp_v3_mappings.required_columns {
            let entry = MappingEntry {
                target_field: mapping.oscal_field.clone(),
                source_type: MappingSourceType::Poam,
                required: mapping.required,
                validation: mapping.validation.clone(),
                data_type: mapping.data_type.clone(),
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

            // Add validation rule if present
            if let Some(validation) = &mapping.validation {
                let rule = ValidationRule {
                    rule_type: Self::parse_validation_type(validation),
                    pattern: None, // TODO: Parse regex patterns from validation string
                    allowed_values: None, // TODO: Parse allowed values from validation string
                    required: mapping.required,
                };
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
        validation_rules: &mut HashMap<String, ValidationRule>,
        required_fields: &mut HashSet<String>,
    ) -> Result<()> {
        // Process SSP sections - this is a simplified implementation
        // In a real implementation, you would process the actual SSP structure
        debug!("Processing SSP mappings: {:?}", ssp);
        
        // For now, we'll add some basic SSP-related mappings
        let ssp_fields = vec![
            ("system_name", "System Name", true),
            ("system_description", "System Description", false),
            ("authorization_boundary", "Authorization Boundary", true),
        ];

        for (field, column_name, required) in ssp_fields {
            let entry = MappingEntry {
                target_field: field.to_string(),
                source_type: MappingSourceType::SspSection,
                required,
                validation: None,
                data_type: Some("string".to_string()),
            };

            if required {
                required_fields.insert(field.to_string());
            }

            let normalized = Self::normalize_column_name(column_name);
            exact_matches.insert(normalized.clone(), entry.clone());

            fuzzy_candidates.push(FuzzyCandidate {
                original_name: column_name.to_string(),
                normalized_name: normalized,
                target_field: field.to_string(),
                source_type: MappingSourceType::SspSection,
                required,
            });
        }

        Ok(())
    }

    /// Normalize column name for consistent matching
    pub fn normalize_column_name(name: &str) -> String {
        name.to_lowercase()
            .replace([' ', '_', '-', '.'], "")
            .replace("&", "and")
            .chars()
            .filter(|c| c.is_alphanumeric())
            .collect()
    }

    /// Parse validation type from string
    fn parse_validation_type(validation: &str) -> ValidationType {
        match validation.to_lowercase().as_str() {
            "boolean" | "bool" => ValidationType::Boolean,
            "numeric" | "number" => ValidationType::Numeric,
            "date" => ValidationType::Date,
            "email" => ValidationType::Email,
            "url" => ValidationType::Url,
            s if s.starts_with("regex:") => ValidationType::Regex,
            s if s.starts_with("values:") => ValidationType::AllowedValues,
            s => ValidationType::Custom(s.to_string()),
        }
    }

    /// Find exact match for column name
    pub fn find_exact_match(&self, column_name: &str) -> Option<&MappingEntry> {
        let normalized = Self::normalize_column_name(column_name);
        self.exact_matches.get(&normalized)
    }

    /// Find fuzzy matches for column name
    pub fn find_fuzzy_matches(&mut self, column_name: &str, min_confidence: f64) -> Vec<MappingResult> {
        let fuzzy_results = self.fuzzy_matcher.find_matches(column_name, &self.fuzzy_targets);
        
        fuzzy_results
            .into_iter()
            .filter(|result| result.confidence >= min_confidence)
            .filter_map(|result| {
                // Find the corresponding fuzzy candidate
                self.fuzzy_candidates
                    .iter()
                    .find(|candidate| candidate.original_name == result.target)
                    .map(|candidate| MappingResult {
                        source_column: column_name.to_string(),
                        target_field: candidate.target_field.clone(),
                        confidence: result.confidence,
                        source_type: candidate.source_type.clone(),
                        required: candidate.required,
                        validation: None, // TODO: Get validation from rules
                        exact_match: false,
                    })
            })
            .collect()
    }

    /// Get validation rule for a field
    pub fn get_validation_rule(&self, field_name: &str) -> Option<&ValidationRule> {
        self.validation_rules.get(field_name)
    }

    /// Check if a field is required
    pub fn is_required_field(&self, field_name: &str) -> bool {
        self.required_fields.contains(field_name)
    }

    /// Get statistics about the lookup structures
    pub fn get_statistics(&self) -> Value {
        let mut source_type_counts = HashMap::new();
        for candidate in &self.fuzzy_candidates {
            *source_type_counts.entry(candidate.source_type.clone()).or_insert(0) += 1;
        }

        serde_json::json!({
            "exact_matches": self.exact_matches.len(),
            "fuzzy_candidates": self.fuzzy_candidates.len(),
            "validation_rules": self.validation_rules.len(),
            "required_fields": self.required_fields.len(),
            "source_type_breakdown": {
                "inventory": source_type_counts.get(&MappingSourceType::Inventory).unwrap_or(&0),
                "poam": source_type_counts.get(&MappingSourceType::Poam).unwrap_or(&0),
                "ssp_section": source_type_counts.get(&MappingSourceType::SspSection).unwrap_or(&0),
                "custom": source_type_counts.get(&MappingSourceType::Custom).unwrap_or(&0),
            }
        })
    }

    /// Clear fuzzy matcher cache for memory management
    pub fn clear_cache(&mut self) {
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
