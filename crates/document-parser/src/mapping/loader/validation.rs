// Modified: 2025-01-22

//! Configuration validation functionality
//!
//! This module provides validation logic for loaded mapping configurations
//! including structural validation, consistency checks, and warning detection.

use super::types::*;
use fedramp_core::{Result, Error};
use tracing::{debug, warn};
use crate::mapping::config::MappingConfiguration;
use crate::mapping::validation;

impl MappingConfigurationLoader {
    /// Validate loaded configuration
    pub fn validate_configuration(&self, config: &MappingConfiguration) -> Result<Vec<String>> {
        let start_time = std::time::Instant::now();
        debug!("Starting configuration validation");

        let mut warnings = Vec::new();

        // Validate inventory mappings
        if let Some(inventory) = &config.inventory_mappings {
            match validation::validate_inventory_mappings(inventory) {
                Ok(mut inv_warnings) => {
                    warnings.append(&mut inv_warnings);
                }
                Err(e) => {
                    warnings.push(format!("Inventory mappings validation error: {}", e));
                }
            }
        } else {
            warnings.push("No inventory mappings loaded".to_string());
        }

        // Validate POA&M mappings
        if let Some(poam) = &config.poam_mappings {
            match validation::validate_poam_mappings(poam) {
                Ok(mut poam_warnings) => {
                    warnings.append(&mut poam_warnings);
                }
                Err(e) => {
                    warnings.push(format!("POA&M mappings validation error: {}", e));
                }
            }
        } else {
            warnings.push("No POA&M mappings loaded".to_string());
        }

        // Validate SSP sections
        if let Some(ssp) = &config.ssp_sections {
            match validation::validate_ssp_sections(ssp) {
                Ok(mut ssp_warnings) => {
                    warnings.append(&mut ssp_warnings);
                }
                Err(e) => {
                    warnings.push(format!("SSP sections validation error: {}", e));
                }
            }
        } else {
            warnings.push("No SSP sections loaded".to_string());
        }

        // Validate control mappings
        if let Some(controls) = &config.controls {
            match validation::validate_control_mappings(controls) {
                Ok(mut control_warnings) => {
                    warnings.append(&mut control_warnings);
                }
                Err(e) => {
                    warnings.push(format!("Control mappings validation error: {}", e));
                }
            }
        } else {
            warnings.push("No control mappings loaded".to_string());
        }

        // Validate document structures
        if let Some(documents) = &config.documents {
            match validation::validate_document_structures(documents) {
                Ok(mut doc_warnings) => {
                    warnings.append(&mut doc_warnings);
                }
                Err(e) => {
                    warnings.push(format!("Document structures validation error: {}", e));
                }
            }
        } else {
            warnings.push("No document structures loaded".to_string());
        }

        // Check for configuration conflicts
        match validation::detect_configuration_conflicts(config) {
            Ok(mut conflict_warnings) => {
                warnings.append(&mut conflict_warnings);
            }
            Err(e) => {
                warnings.push(format!("Configuration conflict detection error: {}", e));
            }
        }

        // Perform additional validation checks
        warnings.extend(self.validate_configuration_completeness(config));
        warnings.extend(self.validate_configuration_consistency(config));

        let validation_time = start_time.elapsed().as_millis() as u64;
        debug!("Configuration validation completed in {}ms with {} warnings", 
               validation_time, warnings.len());

        Ok(warnings)
    }

    /// Validate configuration completeness
    fn validate_configuration_completeness(&self, config: &MappingConfiguration) -> Vec<String> {
        let mut warnings = Vec::new();

        // Check if we have at least one critical configuration
        let has_critical = config.inventory_mappings.is_some() 
            || config.poam_mappings.is_some() 
            || config.ssp_sections.is_some();

        if !has_critical {
            warnings.push("No critical configurations loaded (inventory, POA&M, or SSP)".to_string());
        }

        // Check for recommended configurations
        if config.controls.is_none() {
            warnings.push("Control mappings not loaded - some features may be limited".to_string());
        }

        if config.documents.is_none() {
            warnings.push("Document structures not loaded - document parsing may be limited".to_string());
        }

        // Check inventory mappings completeness
        if let Some(inventory) = &config.inventory_mappings {
            if inventory.fedramp_iiw_mappings.required_columns.is_empty() {
                warnings.push("Inventory mappings has no required columns defined".to_string());
            }

            if inventory.validation_rules.asset_types.as_ref().map_or(true, |types| types.is_empty()) {
                warnings.push("Inventory mappings has no asset types defined".to_string());
            }
        }

        // Check POA&M mappings completeness
        if let Some(poam) = &config.poam_mappings {
            if poam.fedramp_v3_mappings.required_columns.is_empty() {
                warnings.push("POA&M mappings has no required columns defined".to_string());
            }

            if poam.fedramp_v3_mappings.validation_rules.severity_levels.as_ref().map_or(true, |levels| levels.is_empty()) {
                warnings.push("POA&M mappings has no severity levels defined".to_string());
            }

            if poam.fedramp_v3_mappings.validation_rules.status_values.as_ref().map_or(true, |values| values.is_empty()) {
                warnings.push("POA&M mappings has no status values defined".to_string());
            }
        }

        // Check SSP sections completeness
        if let Some(ssp) = &config.ssp_sections {
            if ssp.section_mappings.mappings.is_empty() {
                warnings.push("SSP sections has no section mappings defined".to_string());
            }

            if ssp.control_extraction.patterns.is_empty() {
                warnings.push("SSP sections has no control extraction patterns defined".to_string());
            }
        }

        warnings
    }

    /// Validate configuration consistency
    fn validate_configuration_consistency(&self, config: &MappingConfiguration) -> Vec<String> {
        let mut warnings = Vec::new();

        // Check version consistency
        let mut versions = Vec::new();
        
        if let Some(inventory) = &config.inventory_mappings {
            versions.push(("inventory", &inventory.version));
        }
        
        if let Some(poam) = &config.poam_mappings {
            versions.push(("poam", &poam.version));
        }
        
        if let Some(ssp) = &config.ssp_sections {
            versions.push(("ssp", &ssp.section_mappings.version));
        }

        // Check if all versions are the same
        if versions.len() > 1 {
            let first_version = versions[0].1;
            for (name, version) in &versions[1..] {
                if version != &first_version {
                    warnings.push(format!(
                        "Version mismatch: {} has version '{}' but expected '{}'",
                        name, version, first_version
                    ));
                }
            }
        }

        // Check for overlapping column names between configurations
        if let (Some(inventory), Some(poam)) = (&config.inventory_mappings, &config.poam_mappings) {
            let inventory_columns: std::collections::HashSet<_> = inventory
                .fedramp_iiw_mappings
                .required_columns
                .keys()
                .collect();
            
            let poam_columns: std::collections::HashSet<_> = poam
                .fedramp_v3_mappings
                .required_columns
                .keys()
                .collect();

            let overlapping: Vec<_> = inventory_columns
                .intersection(&poam_columns)
                .collect();

            if !overlapping.is_empty() {
                warnings.push(format!(
                    "Overlapping column names between inventory and POA&M mappings: {:?}",
                    overlapping
                ));
            }
        }

        warnings
    }

    /// Validate configuration with detailed result
    pub fn validate_configuration_detailed(&self, config: &MappingConfiguration) -> ValidationResult {
        let start_time = std::time::Instant::now();
        
        let warnings = match self.validate_configuration(config) {
            Ok(warnings) => warnings,
            Err(e) => {
                return ValidationResult {
                    is_valid: false,
                    warnings: Vec::new(),
                    errors: vec![e.to_string()],
                    validation_time_ms: start_time.elapsed().as_millis() as u64,
                };
            }
        };

        let validation_time = start_time.elapsed().as_millis() as u64;

        // Determine if configuration is valid (no critical errors)
        let critical_warnings = warnings.iter()
            .filter(|w| w.contains("error") || w.contains("No critical configurations"))
            .count();

        ValidationResult {
            is_valid: critical_warnings == 0,
            warnings,
            errors: Vec::new(),
            validation_time_ms: validation_time,
        }
    }

    /// Quick validation check
    pub fn is_configuration_valid(&self, config: &MappingConfiguration) -> bool {
        // Quick check for critical configurations
        config.inventory_mappings.is_some() 
            || config.poam_mappings.is_some() 
            || config.ssp_sections.is_some()
    }
}
