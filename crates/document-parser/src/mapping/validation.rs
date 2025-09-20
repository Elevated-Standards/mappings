//! Configuration validation for mapping files
//!
//! This module provides validation functionality for mapping configurations
//! to ensure they are well-formed and consistent.

use fedramp_core::Result;
use regex::Regex;

use crate::mapping::config::MappingConfiguration;
use crate::mapping::inventory::InventoryMappings;
use crate::mapping::poam::PoamMappings;
use crate::mapping::ssp::SspSections;
use crate::mapping::control_document::{ControlMappings, DocumentStructures};

/// Validate inventory mappings
pub fn validate_inventory_mappings(inventory: &InventoryMappings) -> Result<Vec<String>> {
    let mut warnings = Vec::new();

    // Check required columns have valid field mappings
    for (key, mapping) in &inventory.fedramp_iiw_mappings.required_columns {
        if mapping.column_names.is_empty() {
            warnings.push(format!("Inventory mapping '{}' has no column names", key));
        }

        if mapping.field.is_empty() {
            warnings.push(format!("Inventory mapping '{}' has empty field name", key));
        }

        // Validate regex patterns if present
        if let Some(validation) = &mapping.validation {
            if validation == "ip_address" {
                if let Some(pattern) = &inventory.validation_rules.ip_address_pattern {
                    if let Err(e) = Regex::new(pattern) {
                        warnings.push(format!("Invalid IP address regex pattern: {}", e));
                    }
                }
            } else if validation == "mac_address" {
                if let Some(pattern) = &inventory.validation_rules.mac_address_pattern {
                    if let Err(e) = Regex::new(pattern) {
                        warnings.push(format!("Invalid MAC address regex pattern: {}", e));
                    }
                }
            }
        }
    }

    // Validate component type mappings
    for (key, mapping) in &inventory.component_type_mappings {
        if mapping.keywords.is_empty() {
            warnings.push(format!("Component type '{}' has no keywords", key));
        }
    }

    Ok(warnings)
}

/// Validate POA&M mappings
pub fn validate_poam_mappings(poam: &PoamMappings) -> Result<Vec<String>> {
    let mut warnings = Vec::new();

    // Check required columns
    for (key, mapping) in &poam.fedramp_v3_mappings.required_columns {
        if mapping.column_names.is_empty() {
            warnings.push(format!("POA&M mapping '{}' has no column names", key));
        }

        if mapping.oscal_field.is_empty() {
            warnings.push(format!("POA&M mapping '{}' has empty OSCAL field", key));
        }
    }

    // Validate control ID pattern
    if let Some(pattern) = &poam.fedramp_v3_mappings.validation_rules.control_id_pattern {
        if let Err(e) = Regex::new(pattern) {
            warnings.push(format!("Invalid control ID regex pattern: {}", e));
        }
    }

    // Validate milestone patterns
    for pattern in &poam.milestone_processing.patterns.milestone_format.patterns {
        if let Err(e) = Regex::new(pattern) {
            warnings.push(format!("Invalid milestone regex pattern: {}", e));
        }
    }

    Ok(warnings)
}

/// Validate SSP sections
pub fn validate_ssp_sections(ssp: &SspSections) -> Result<Vec<String>> {
    let mut warnings = Vec::new();

    // Validate section mappings
    for (key, mapping) in &ssp.section_mappings.mappings {
        if mapping.keywords.is_empty() {
            warnings.push(format!("SSP section '{}' has no keywords", key));
        }

        if mapping.target.is_empty() {
            warnings.push(format!("SSP section '{}' has empty target", key));
        }

        // Validate extract patterns
        if let Some(patterns) = &mapping.extract_patterns {
            for (pattern_key, pattern) in patterns {
                if let Err(e) = Regex::new(pattern) {
                    warnings.push(format!(
                        "Invalid extract pattern '{}' in section '{}': {}",
                        pattern_key, key, e
                    ));
                }
            }
        }
    }

    // Validate control extraction patterns
    for pattern in &ssp.control_extraction.patterns {
        if let Err(e) = Regex::new(&pattern.regex) {
            warnings.push(format!(
                "Invalid control extraction pattern '{}': {}",
                pattern.name, e
            ));
        }
    }

    Ok(warnings)
}

/// Validate control mappings
pub fn validate_control_mappings(controls: &ControlMappings) -> Result<Vec<String>> {
    let mut warnings = Vec::new();

    // Check metadata completeness
    if controls.metadata.source_file.is_empty() {
        warnings.push("Control mappings metadata missing source file".to_string());
    }

    if controls.metadata.framework.is_empty() {
        warnings.push("Control mappings metadata missing framework".to_string());
    }

    // Validate individual controls
    for control in &controls.controls {
        if control.control_id.is_empty() {
            warnings.push("Control mapping has empty control ID".to_string());
        }

        if control.implementation_status.is_empty() {
            warnings.push(format!(
                "Control '{}' has empty implementation status",
                control.control_id
            ));
        }

        // Validate control enhancements
        if let Some(enhancements) = &control.control_enhancements {
            for enhancement in enhancements {
                if enhancement.enhancement_id.is_empty() {
                    warnings.push(format!(
                        "Control '{}' has enhancement with empty ID",
                        control.control_id
                    ));
                }
            }
        }
    }

    Ok(warnings)
}

/// Validate document structures
pub fn validate_document_structures(documents: &DocumentStructures) -> Result<Vec<String>> {
    let mut warnings = Vec::new();

    // Check metadata
    if documents.metadata.source_file.is_empty() {
        warnings.push("Document structures metadata missing source file".to_string());
    }

    if documents.metadata.source_type.is_empty() {
        warnings.push("Document structures metadata missing source type".to_string());
    }

    // Validate sections
    for section in &documents.sections {
        if section.id.is_empty() {
            warnings.push("Document section has empty ID".to_string());
        }

        if section.title.is_empty() {
            warnings.push(format!("Document section '{}' has empty title", section.id));
        }

        // Validate tables
        if let Some(tables) = &section.tables {
            for table in tables {
                if table.id.is_empty() {
                    warnings.push(format!(
                        "Table in section '{}' has empty ID",
                        section.id
                    ));
                }

                if table.headers.is_empty() {
                    warnings.push(format!(
                        "Table '{}' in section '{}' has no headers",
                        table.id, section.id
                    ));
                }
            }
        }
    }

    Ok(warnings)
}

/// Detect configuration conflicts between different mapping files
pub fn detect_configuration_conflicts(config: &MappingConfiguration) -> Result<Vec<String>> {
    let mut conflicts = Vec::new();

    // Check for overlapping field mappings between inventory and POA&M
    if let (Some(inventory), Some(poam)) = (&config.inventory_mappings, &config.poam_mappings) {
        let inventory_fields: std::collections::HashSet<_> = inventory
            .fedramp_iiw_mappings
            .required_columns
            .values()
            .map(|m| &m.field)
            .collect();

        let poam_fields: std::collections::HashSet<_> = poam
            .fedramp_v3_mappings
            .required_columns
            .values()
            .map(|m| &m.oscal_field)
            .collect();

        for field in inventory_fields.intersection(&poam_fields) {
            conflicts.push(format!(
                "Field '{}' is mapped in both inventory and POA&M configurations",
                field
            ));
        }
    }

    // Check for duplicate control IDs in control mappings
    if let Some(controls) = &config.controls {
        let mut control_ids = std::collections::HashSet::new();
        for control in &controls.controls {
            if !control_ids.insert(&control.control_id) {
                conflicts.push(format!(
                    "Duplicate control ID '{}' found in control mappings",
                    control.control_id
                ));
            }
        }
    }

    Ok(conflicts)
}
