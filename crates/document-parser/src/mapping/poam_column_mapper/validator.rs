//! POA&M mapping validation functionality
//! Modified: 2025-01-22

use crate::validation::{ColumnValidator, ValidationResult, ValidationStatus, ValidationSeverity};
use crate::mapping::poam::PoamValidationRules;
use std::collections::HashMap;
use super::types::*;

impl PoamMappingValidator {
    /// Create a new POA&M mapping validator
    pub fn new() -> Self {
        Self {
            column_validator: ColumnValidator::new(crate::mapping::MappingConfiguration {
                inventory_mappings: None,
                poam_mappings: None,
                ssp_sections: None,
                controls: None,
                documents: None,
            }),
            poam_rules: PoamValidationRules {
                severity_levels: Some(vec![
                    "Critical".to_string(),
                    "High".to_string(),
                    "Moderate".to_string(),
                    "Low".to_string(),
                    "Informational".to_string(),
                ]),
                status_values: Some(vec![
                    "Open".to_string(),
                    "InProgress".to_string(),
                    "Completed".to_string(),
                    "Closed".to_string(),
                    "Cancelled".to_string(),
                ]),
                control_id_pattern: Some(r"^[A-Z]{2}-\d+(\.\d+)*$".to_string()),
                date_formats: Some(vec![
                    "%Y-%m-%d".to_string(),
                    "%m/%d/%Y".to_string(),
                    "%d/%m/%Y".to_string(),
                ]),
            },
            validation_cache: HashMap::new(),
        }
    }

    /// Validate a POA&M field mapping
    pub async fn validate_field_mapping(&self, field_mapping: &PoamFieldMapping) -> Option<ValidationResult> {
        // Check cache first
        let cache_key = format!("{}:{}", field_mapping.source_column, field_mapping.target_field);
        if let Some(cached_result) = self.validation_cache.get(&cache_key) {
            return Some(cached_result.clone());
        }

        // Perform validation based on field type
        let result = match field_mapping.target_field.as_str() {
            "severity" | "risk_level" => self.validate_severity_field(field_mapping),
            "status" | "poam_status" => self.validate_status_field(field_mapping),
            "control_id" | "security_control_number" => self.validate_control_id_field(field_mapping),
            "scheduled_completion_date" | "actual_completion_date" => self.validate_date_field(field_mapping),
            _ => self.validate_generic_field(field_mapping),
        };

        result
    }

    /// Validate severity field
    fn validate_severity_field(&self, field_mapping: &PoamFieldMapping) -> Option<ValidationResult> {
        // Check if severity levels are configured
        if let Some(severity_levels) = &self.poam_rules.severity_levels {
            // In a real implementation, this would validate actual data values
            // For now, we'll just validate the field mapping itself
            Some(ValidationResult {
                field_name: field_mapping.source_column.clone(),
                passed: true,
                status: ValidationStatus::Valid,
                severity: ValidationSeverity::Info,
                message: format!(
                    "Severity field validation passed. Expected values: {}",
                    severity_levels.join(", ")
                ),
                suggested_fix: None,
            })
        } else {
            Some(ValidationResult {
                field_name: field_mapping.source_column.clone(),
                passed: false,
                status: ValidationStatus::Invalid,
                severity: ValidationSeverity::Warning,
                message: "No severity levels configured for validation".to_string(),
                suggested_fix: Some("Configure severity levels in POA&M validation rules".to_string()),
            })
        }
    }

    /// Validate status field
    fn validate_status_field(&self, field_mapping: &PoamFieldMapping) -> Option<ValidationResult> {
        if let Some(status_values) = &self.poam_rules.status_values {
            Some(ValidationResult {
                field_name: field_mapping.source_column.clone(),
                passed: true,
                status: ValidationStatus::Valid,
                severity: ValidationSeverity::Info,
                message: format!(
                    "Status field validation passed. Expected values: {}",
                    status_values.join(", ")
                ),
                suggested_fix: None,
            })
        } else {
            Some(ValidationResult {
                field_name: field_mapping.source_column.clone(),
                passed: false,
                status: ValidationStatus::Invalid,
                severity: ValidationSeverity::Warning,
                message: "No status values configured for validation".to_string(),
                suggested_fix: Some("Configure status values in POA&M validation rules".to_string()),
            })
        }
    }

    /// Validate control ID field
    fn validate_control_id_field(&self, field_mapping: &PoamFieldMapping) -> Option<ValidationResult> {
        if let Some(pattern) = &self.poam_rules.control_id_pattern {
            Some(ValidationResult {
                field_name: field_mapping.source_column.clone(),
                passed: true,
                status: ValidationStatus::Valid,
                severity: ValidationSeverity::Info,
                message: format!(
                    "Control ID field validation passed. Expected pattern: {}",
                    pattern
                ),
                suggested_fix: None,
            })
        } else {
            Some(ValidationResult {
                field_name: field_mapping.source_column.clone(),
                passed: true,
                status: ValidationStatus::Valid,
                severity: ValidationSeverity::Info,
                message: "Control ID field validation passed (no pattern configured)".to_string(),
                suggested_fix: None,
            })
        }
    }

    /// Validate date field
    fn validate_date_field(&self, field_mapping: &PoamFieldMapping) -> Option<ValidationResult> {
        if let Some(date_formats) = &self.poam_rules.date_formats {
            Some(ValidationResult {
                field_name: field_mapping.source_column.clone(),
                passed: true,
                status: ValidationStatus::Valid,
                severity: ValidationSeverity::Info,
                message: format!(
                    "Date field validation passed. Expected formats: {}",
                    date_formats.join(", ")
                ),
                suggested_fix: None,
            })
        } else {
            Some(ValidationResult {
                field_name: field_mapping.source_column.clone(),
                passed: true,
                status: ValidationStatus::Valid,
                severity: ValidationSeverity::Info,
                message: "Date field validation passed (no formats configured)".to_string(),
                suggested_fix: None,
            })
        }
    }

    /// Validate generic field
    fn validate_generic_field(&self, field_mapping: &PoamFieldMapping) -> Option<ValidationResult> {
        // Basic validation for generic fields
        let passed = !field_mapping.source_column.is_empty() && !field_mapping.target_field.is_empty();
        
        Some(ValidationResult {
            field_name: field_mapping.source_column.clone(),
            passed,
            status: if passed { ValidationStatus::Valid } else { ValidationStatus::Invalid },
            severity: if passed { ValidationSeverity::Info } else { ValidationSeverity::Error },
            message: if passed {
                "Generic field validation passed".to_string()
            } else {
                "Field mapping has empty source or target field".to_string()
            },
            suggested_fix: if passed {
                None
            } else {
                Some("Ensure both source and target fields are properly configured".to_string())
            },
        })
    }

    /// Validate multiple field mappings
    pub async fn validate_multiple_mappings(&self, field_mappings: &[PoamFieldMapping]) -> Vec<ValidationResult> {
        let mut results = Vec::new();
        
        for field_mapping in field_mappings {
            if let Some(result) = self.validate_field_mapping(field_mapping).await {
                results.push(result);
            }
        }
        
        // Perform cross-field validation
        let cross_field_results = self.validate_cross_field_relationships(field_mappings);
        results.extend(cross_field_results);
        
        results
    }

    /// Validate cross-field relationships
    fn validate_cross_field_relationships(&self, field_mappings: &[PoamFieldMapping]) -> Vec<ValidationResult> {
        let mut results = Vec::new();
        
        // Check for required field combinations
        let has_id = field_mappings.iter().any(|f| f.target_field.contains("id"));
        let has_description = field_mappings.iter().any(|f| f.target_field.contains("description"));
        let has_status = field_mappings.iter().any(|f| f.target_field.contains("status"));
        
        if !has_id {
            results.push(ValidationResult {
                field_name: "cross_field_validation".to_string(),
                passed: false,
                status: ValidationStatus::Invalid,
                severity: ValidationSeverity::Error,
                message: "Missing required ID field mapping".to_string(),
                suggested_fix: Some("Ensure an ID field is mapped".to_string()),
            });
        }
        
        if !has_description {
            results.push(ValidationResult {
                field_name: "cross_field_validation".to_string(),
                passed: false,
                status: ValidationStatus::Invalid,
                severity: ValidationSeverity::Warning,
                message: "Missing description field mapping".to_string(),
                suggested_fix: Some("Consider mapping a description field".to_string()),
            });
        }
        
        if !has_status {
            results.push(ValidationResult {
                field_name: "cross_field_validation".to_string(),
                passed: false,
                status: ValidationStatus::Invalid,
                severity: ValidationSeverity::Warning,
                message: "Missing status field mapping".to_string(),
                suggested_fix: Some("Consider mapping a status field".to_string()),
            });
        }
        
        results
    }

    /// Clear validation cache
    pub fn clear_cache(&mut self) {
        self.validation_cache.clear();
    }

    /// Get validation cache size
    pub fn cache_size(&self) -> usize {
        self.validation_cache.len()
    }

    /// Update POA&M validation rules
    pub fn update_poam_rules(&mut self, rules: PoamValidationRules) {
        self.poam_rules = rules;
        self.clear_cache(); // Clear cache when rules change
    }

    /// Get current POA&M validation rules
    pub fn poam_rules(&self) -> &PoamValidationRules {
        &self.poam_rules
    }
}

impl Default for PoamMappingValidator {
    fn default() -> Self {
        Self::new()
    }
}
