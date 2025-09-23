//! Modified: 2025-09-23

//! Column validator implementation
//!
//! This module contains the implementation of column-level validation functionality
//! including field validation, data type checking, and format validation.

use fedramp_core::{Result, Error};
use std::time::Instant;
use tracing::{debug, info, warn};
use regex::Regex;
use crate::mapping::{InventoryMappings, PoamMappings};
use crate::fuzzy::FuzzyMatcher;

use super::types::*;
use super::super::types::*;

impl ColumnValidator {
    /// Validate columns against mapping configuration
    pub fn validate_columns(
        &mut self,
        headers: &[String],
        data: &[Vec<serde_json::Value>],
        document_type: &str,
    ) -> Result<ColumnValidationReport> {
        let start_time = Instant::now();
        
        info!("Starting column validation for document type: {}", document_type);
        
        let validation_rules = self.get_validation_rules(document_type)?;
        let mut field_results = Vec::new();
        let mut missing_required = Vec::new();
        let mut type_mismatches = Vec::new();
        let mut enumeration_failures = Vec::new();
        let mut cross_field_results = Vec::new();

        // Validate each field according to the rules
        for rule in &validation_rules {
            let field_result = self.validate_field(rule, headers, data)?;
            
            match &field_result.status {
                ValidationStatus::MissingRequired => {
                    missing_required.push(RequiredFieldInfo {
                        field_id: rule.field_id.clone(),
                        expected_columns: rule.column_names.clone(),
                        oscal_field: rule.oscal_field.clone(),
                        description: format!("Required field {} is missing", rule.field_id),
                        alternatives: self.suggest_alternatives(&rule.field_id, headers),
                    });
                }
                ValidationStatus::Invalid => {
                    if let Some(data_type) = &rule.data_type {
                        type_mismatches.push(TypeMismatchInfo {
                            field_id: rule.field_id.clone(),
                            column_name: field_result.source_column.clone().unwrap_or_default(),
                            expected_type: data_type.clone(),
                            actual_type: "unknown".to_string(),
                            sample_values: vec!["sample".to_string()],
                            suggested_conversion: None,
                        });
                    }
                }
                _ => {}
            }

            field_results.push(field_result);
        }

        // Perform cross-field validation
        cross_field_results = self.validate_cross_fields(&field_results, data)?;

        // Calculate metrics
        let metrics = self.calculate_validation_metrics(&field_results, &missing_required);
        
        let total_execution_time = start_time.elapsed();
        
        Ok(ColumnValidationReport {
            document_type: document_type.to_string(),
            is_valid: metrics.error_count == 0 && missing_required.is_empty(),
            field_results,
            missing_required,
            type_mismatches,
            enumeration_failures,
            cross_field_results,
            metrics,
            total_execution_time,
            timestamp: chrono::Utc::now(),
        })
    }

    /// Get validation rules for a specific document type
    fn get_validation_rules(&self, document_type: &str) -> Result<Vec<ColumnValidationRule>> {
        match document_type.to_lowercase().as_str() {
            "inventory" => {
                if let Some(inventory_mappings) = &self.mapping_config.inventory {
                    Ok(self.convert_inventory_mappings_to_rules(inventory_mappings))
                } else {
                    Err(Error::ValidationError("No inventory mappings configured".to_string()))
                }
            }
            "poam" => {
                if let Some(poam_mappings) = &self.mapping_config.poam {
                    Ok(self.convert_poam_mappings_to_rules(poam_mappings))
                } else {
                    Err(Error::ValidationError("No POAM mappings configured".to_string()))
                }
            }
            _ => {
                warn!("Unknown document type: {}", document_type);
                Ok(Vec::new())
            }
        }
    }

    /// Convert inventory mappings to validation rules
    fn convert_inventory_mappings_to_rules(&self, mappings: &InventoryMappings) -> Vec<ColumnValidationRule> {
        let mut rules = Vec::new();
        
        // Add rules for each mapped field
        if let Some(asset_id) = &mappings.asset_id {
            rules.push(ColumnValidationRule {
                field_id: "asset_id".to_string(),
                column_names: vec![asset_id.clone()],
                oscal_field: "system-component.uuid".to_string(),
                required: true,
                validation_type: Some("format".to_string()),
                allowed_values: None,
                pattern: Some(r"^[A-Za-z0-9\-_]+$".to_string()),
                data_type: Some("string".to_string()),
                conditional: None,
            });
        }

        if let Some(asset_name) = &mappings.asset_name {
            rules.push(ColumnValidationRule {
                field_id: "asset_name".to_string(),
                column_names: vec![asset_name.clone()],
                oscal_field: "system-component.title".to_string(),
                required: true,
                validation_type: Some("format".to_string()),
                allowed_values: None,
                pattern: Some(r"^.{1,255}$".to_string()),
                data_type: Some("string".to_string()),
                conditional: None,
            });
        }

        if let Some(asset_type) = &mappings.asset_type {
            rules.push(ColumnValidationRule {
                field_id: "asset_type".to_string(),
                column_names: vec![asset_type.clone()],
                oscal_field: "system-component.type".to_string(),
                required: true,
                validation_type: Some("enumeration".to_string()),
                allowed_values: Some(vec![
                    "software".to_string(),
                    "hardware".to_string(),
                    "service".to_string(),
                    "policy".to_string(),
                    "process".to_string(),
                    "plan".to_string(),
                    "guidance".to_string(),
                    "standard".to_string(),
                    "validation".to_string(),
                ]),
                pattern: None,
                data_type: Some("string".to_string()),
                conditional: None,
            });
        }

        rules
    }

    /// Convert POAM mappings to validation rules
    fn convert_poam_mappings_to_rules(&self, mappings: &PoamMappings) -> Vec<ColumnValidationRule> {
        let mut rules = Vec::new();
        
        if let Some(poam_id) = &mappings.poam_id {
            rules.push(ColumnValidationRule {
                field_id: "poam_id".to_string(),
                column_names: vec![poam_id.clone()],
                oscal_field: "finding.uuid".to_string(),
                required: true,
                validation_type: Some("format".to_string()),
                allowed_values: None,
                pattern: Some(r"^[A-Za-z0-9\-_]+$".to_string()),
                data_type: Some("string".to_string()),
                conditional: None,
            });
        }

        if let Some(control_id) = &mappings.control_id {
            rules.push(ColumnValidationRule {
                field_id: "control_id".to_string(),
                column_names: vec![control_id.clone()],
                oscal_field: "finding.related-observations.observation.relevant-evidence.href".to_string(),
                required: true,
                validation_type: Some("format".to_string()),
                allowed_values: None,
                pattern: Some(r"^[A-Z]{2}-\d+(\.\d+)*$".to_string()),
                data_type: Some("string".to_string()),
                conditional: None,
            });
        }

        if let Some(severity) = &mappings.severity {
            rules.push(ColumnValidationRule {
                field_id: "severity".to_string(),
                column_names: vec![severity.clone()],
                oscal_field: "finding.target.status.state".to_string(),
                required: true,
                validation_type: Some("enumeration".to_string()),
                allowed_values: Some(vec![
                    "low".to_string(),
                    "moderate".to_string(),
                    "high".to_string(),
                    "critical".to_string(),
                ]),
                pattern: None,
                data_type: Some("string".to_string()),
                conditional: None,
            });
        }

        rules
    }

    /// Validate a single field according to its rule
    fn validate_field(
        &mut self,
        rule: &ColumnValidationRule,
        headers: &[String],
        data: &[Vec<serde_json::Value>],
    ) -> Result<ColumnValidationResult> {
        let start_time = Instant::now();
        
        // Find the column for this field
        let source_column = self.find_matching_column(rule, headers);
        
        if source_column.is_none() && rule.required {
            return Ok(ColumnValidationResult {
                field_id: rule.field_id.clone(),
                source_column: None,
                oscal_field: rule.oscal_field.clone(),
                passed: false,
                status: ValidationStatus::MissingRequired,
                message: format!("Required field '{}' not found in headers", rule.field_id),
                severity: ValidationSeverity::Error,
                suggestions: self.suggest_alternatives(&rule.field_id, headers),
                execution_time: start_time.elapsed(),
            });
        }

        if let Some(column_name) = source_column {
            // Find column index
            let column_index = headers.iter().position(|h| h == &column_name);
            
            if let Some(col_idx) = column_index {
                // Validate the data in this column
                let column_data: Vec<&serde_json::Value> = data.iter()
                    .filter_map(|row| row.get(col_idx))
                    .collect();
                
                let validation_result = self.validate_column_data(rule, &column_data)?;
                
                return Ok(ColumnValidationResult {
                    field_id: rule.field_id.clone(),
                    source_column: Some(column_name),
                    oscal_field: rule.oscal_field.clone(),
                    passed: validation_result.passed,
                    status: if validation_result.passed { 
                        ValidationStatus::Valid 
                    } else { 
                        ValidationStatus::Invalid 
                    },
                    message: validation_result.message.unwrap_or_else(|| "Validation completed".to_string()),
                    severity: if validation_result.passed {
                        ValidationSeverity::Info
                    } else {
                        ValidationSeverity::Error
                    },
                    suggestions: Vec::new(),
                    execution_time: start_time.elapsed(),
                });
            }
        }

        // Field not found but not required
        Ok(ColumnValidationResult {
            field_id: rule.field_id.clone(),
            source_column: None,
            oscal_field: rule.oscal_field.clone(),
            passed: true,
            status: ValidationStatus::MissingOptional,
            message: format!("Optional field '{}' not found", rule.field_id),
            severity: ValidationSeverity::Info,
            suggestions: Vec::new(),
            execution_time: start_time.elapsed(),
        })
    }
}
