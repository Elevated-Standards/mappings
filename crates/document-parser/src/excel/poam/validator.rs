//! POA&M business rule validation functionality
//! Modified: 2025-01-22

use crate::excel::types::ValidationSeverity;
use std::collections::HashMap;

use super::types::*;

impl PoamValidator {
    /// Create a new POA&M validator
    pub fn new() -> Self {
        Self {
            validation_rules: HashMap::new(),
        }
    }

    /// Create a new POA&M validator with custom rules
    pub fn with_rules(rules: HashMap<String, ValidationRule>) -> Self {
        Self {
            validation_rules: rules,
        }
    }

    /// Get validation rules
    pub fn validation_rules(&self) -> &HashMap<String, ValidationRule> {
        &self.validation_rules
    }

    /// Set validation rules
    pub fn set_validation_rules(&mut self, rules: HashMap<String, ValidationRule>) {
        self.validation_rules = rules;
    }

    /// Add validation rule
    pub fn add_validation_rule(&mut self, name: String, rule: ValidationRule) {
        self.validation_rules.insert(name, rule);
    }

    /// Remove validation rule
    pub fn remove_validation_rule(&mut self, name: &str) -> Option<ValidationRule> {
        self.validation_rules.remove(name)
    }

    /// Validate a POA&M item against business rules
    pub fn validate_poam_item(&self, item: &PoamItem) -> Vec<PoamValidationResult> {
        let mut results = Vec::new();

        // Validate required fields
        if item.unique_id.trim().is_empty() {
            results.push(PoamValidationResult {
                row_number: 0, // TODO: Pass actual row number
                field_name: "unique_id".to_string(),
                error_message: "Unique ID is required".to_string(),
                severity: ValidationSeverity::Error,
                suggestion: Some("Provide a unique identifier for this POA&M item".to_string()),
            });
        }

        if item.weakness_description.trim().is_empty() {
            results.push(PoamValidationResult {
                row_number: 0,
                field_name: "weakness_description".to_string(),
                error_message: "Weakness description is required".to_string(),
                severity: ValidationSeverity::Error,
                suggestion: Some("Provide a detailed description of the weakness".to_string()),
            });
        }

        // Validate severity
        // (Severity is an enum, so it's always valid)

        // Validate dates
        if let (Some(scheduled), Some(actual)) = (&item.scheduled_completion_date, &item.actual_completion_date) {
            if actual < scheduled {
                results.push(PoamValidationResult {
                    row_number: 0,
                    field_name: "actual_completion_date".to_string(),
                    error_message: "Actual completion date is before scheduled date".to_string(),
                    severity: ValidationSeverity::Warning,
                    suggestion: Some("Verify the completion dates are correct".to_string()),
                });
            }
        }

        // Validate control ID format
        if let Some(control_id) = &item.control_id {
            if !self.is_valid_control_id(control_id) {
                results.push(PoamValidationResult {
                    row_number: 0,
                    field_name: "control_id".to_string(),
                    error_message: format!("Invalid control ID format: {}", control_id),
                    severity: ValidationSeverity::Warning,
                    suggestion: Some("Control ID should follow NIST format (e.g., AC-1, SC-7)".to_string()),
                });
            }
        }

        // Validate CCI format
        if let Some(cci) = &item.cci {
            if !self.is_valid_cci(cci) {
                results.push(PoamValidationResult {
                    row_number: 0,
                    field_name: "cci".to_string(),
                    error_message: format!("Invalid CCI format: {}", cci),
                    severity: ValidationSeverity::Warning,
                    suggestion: Some("CCI should be in format CCI-XXXXXX".to_string()),
                });
            }
        }

        // Validate cost estimate
        if let Some(cost) = item.cost_estimate {
            if cost < 0.0 {
                results.push(PoamValidationResult {
                    row_number: 0,
                    field_name: "cost_estimate".to_string(),
                    error_message: "Cost estimate cannot be negative".to_string(),
                    severity: ValidationSeverity::Error,
                    suggestion: Some("Provide a valid positive cost estimate".to_string()),
                });
            }
        }

        // Validate milestones
        for (index, milestone) in item.milestones.iter().enumerate() {
            let milestone_results = self.validate_milestone(milestone, index);
            results.extend(milestone_results);
        }

        // Validate resources
        for (index, resource) in item.resources.iter().enumerate() {
            let resource_results = self.validate_resource(resource, index);
            results.extend(resource_results);
        }

        results
    }

    /// Validate multiple POA&M items
    pub fn validate_poam_items(&self, items: &[PoamItem]) -> Vec<PoamValidationResult> {
        let mut all_results = Vec::new();

        for item in items {
            let item_results = self.validate_poam_item(item);
            all_results.extend(item_results);
        }

        // Cross-item validation
        let cross_validation_results = self.validate_cross_item_rules(items);
        all_results.extend(cross_validation_results);

        all_results
    }

    /// Validate a milestone
    fn validate_milestone(&self, milestone: &PoamMilestone, index: usize) -> Vec<PoamValidationResult> {
        let mut results = Vec::new();

        if milestone.description.trim().is_empty() {
            results.push(PoamValidationResult {
                row_number: 0,
                field_name: format!("milestones[{}].description", index),
                error_message: "Milestone description is required".to_string(),
                severity: ValidationSeverity::Error,
                suggestion: Some("Provide a description for the milestone".to_string()),
            });
        }

        if let Some(percent) = milestone.percent_complete {
            if percent > 100 {
                results.push(PoamValidationResult {
                    row_number: 0,
                    field_name: format!("milestones[{}].percent_complete", index),
                    error_message: "Percent complete cannot exceed 100%".to_string(),
                    severity: ValidationSeverity::Error,
                    suggestion: Some("Provide a valid percentage between 0 and 100".to_string()),
                });
            }
        }

        if let (Some(scheduled), Some(actual)) = (&milestone.scheduled_date, &milestone.actual_date) {
            if actual < scheduled {
                results.push(PoamValidationResult {
                    row_number: 0,
                    field_name: format!("milestones[{}].actual_date", index),
                    error_message: "Milestone actual date is before scheduled date".to_string(),
                    severity: ValidationSeverity::Warning,
                    suggestion: Some("Verify the milestone dates are correct".to_string()),
                });
            }
        }

        results
    }

    /// Validate a resource
    fn validate_resource(&self, resource: &PoamResource, index: usize) -> Vec<PoamValidationResult> {
        let mut results = Vec::new();

        if resource.description.trim().is_empty() {
            results.push(PoamValidationResult {
                row_number: 0,
                field_name: format!("resources[{}].description", index),
                error_message: "Resource description is required".to_string(),
                severity: ValidationSeverity::Error,
                suggestion: Some("Provide a description for the resource".to_string()),
            });
        }

        if let Some(cost) = resource.estimated_cost {
            if cost < 0.0 {
                results.push(PoamValidationResult {
                    row_number: 0,
                    field_name: format!("resources[{}].estimated_cost", index),
                    error_message: "Resource estimated cost cannot be negative".to_string(),
                    severity: ValidationSeverity::Error,
                    suggestion: Some("Provide a valid positive cost estimate".to_string()),
                });
            }
        }

        if let Some(cost) = resource.actual_cost {
            if cost < 0.0 {
                results.push(PoamValidationResult {
                    row_number: 0,
                    field_name: format!("resources[{}].actual_cost", index),
                    error_message: "Resource actual cost cannot be negative".to_string(),
                    severity: ValidationSeverity::Error,
                    suggestion: Some("Provide a valid positive actual cost".to_string()),
                });
            }
        }

        if let Some(quantity) = resource.quantity {
            if quantity < 0.0 {
                results.push(PoamValidationResult {
                    row_number: 0,
                    field_name: format!("resources[{}].quantity", index),
                    error_message: "Resource quantity cannot be negative".to_string(),
                    severity: ValidationSeverity::Error,
                    suggestion: Some("Provide a valid positive quantity".to_string()),
                });
            }
        }

        results
    }

    /// Validate cross-item business rules
    fn validate_cross_item_rules(&self, items: &[PoamItem]) -> Vec<PoamValidationResult> {
        let mut results = Vec::new();

        // Check for duplicate unique IDs
        let mut unique_ids = HashMap::new();
        for (index, item) in items.iter().enumerate() {
            if let Some(existing_index) = unique_ids.insert(&item.unique_id, index) {
                results.push(PoamValidationResult {
                    row_number: index + 1,
                    field_name: "unique_id".to_string(),
                    error_message: format!("Duplicate unique ID '{}' found at rows {} and {}", 
                                         item.unique_id, existing_index + 1, index + 1),
                    severity: ValidationSeverity::Error,
                    suggestion: Some("Ensure all POA&M items have unique identifiers".to_string()),
                });
            }
        }

        results
    }

    /// Validate control ID format
    fn is_valid_control_id(&self, control_id: &str) -> bool {
        // Basic NIST control ID format validation
        let control_id = control_id.trim().to_uppercase();
        
        // Should match pattern like AC-1, SC-7, AC-2(1), etc.
        if control_id.len() < 4 {
            return false;
        }

        let parts: Vec<&str> = control_id.split('-').collect();
        if parts.len() != 2 {
            return false;
        }

        // First part should be 2 letters
        if parts[0].len() != 2 || !parts[0].chars().all(|c| c.is_ascii_alphabetic()) {
            return false;
        }

        // Second part should start with a number
        if !parts[1].chars().next().unwrap_or('a').is_ascii_digit() {
            return false;
        }

        true
    }

    /// Validate CCI format
    fn is_valid_cci(&self, cci: &str) -> bool {
        // CCI format: CCI-XXXXXX where X is a digit
        let cci = cci.trim().to_uppercase();
        
        if !cci.starts_with("CCI-") {
            return false;
        }

        let number_part = &cci[4..];
        if number_part.len() != 6 {
            return false;
        }

        number_part.chars().all(|c| c.is_ascii_digit())
    }

    /// Get validation statistics
    pub fn get_validation_statistics(&self, results: &[PoamValidationResult]) -> ValidationStatistics {
        let total_validations = results.len();
        let errors = results.iter().filter(|r| r.severity == ValidationSeverity::Error).count();
        let warnings = results.iter().filter(|r| r.severity == ValidationSeverity::Warning).count();
        let info = results.iter().filter(|r| r.severity == ValidationSeverity::Info).count();

        ValidationStatistics {
            total_validations,
            errors,
            warnings,
            info,
        }
    }
}

/// Validation statistics
#[derive(Debug, Clone)]
pub struct ValidationStatistics {
    /// Total number of validation results
    pub total_validations: usize,
    /// Number of errors
    pub errors: usize,
    /// Number of warnings
    pub warnings: usize,
    /// Number of info messages
    pub info: usize,
}

impl Default for PoamValidator {
    fn default() -> Self {
        Self::new()
    }
}
