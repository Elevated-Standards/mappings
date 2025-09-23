//! Modified: 2025-09-23

//! Document validator implementation
//!
//! This module contains the implementation of document-level validation functionality
//! including comprehensive document validation and coordination of column validation.

use fedramp_core::{Result, Error};
use tracing::{debug, info, warn};

use super::types::*;
use super::super::types::*;

impl DocumentValidator {
    /// Validate a document with the given data
    pub fn validate_document(
        &mut self,
        document_type: &str,
        headers: &[String],
        data: &[Vec<serde_json::Value>],
    ) -> Result<ColumnValidationReport> {
        if let Some(ref mut column_validator) = self.column_validator {
            column_validator.validate_columns(headers, data, document_type)
        } else {
            Err(Error::ValidationError("No column validator configured".to_string()))
        }
    }

    /// Validate multiple documents of the same type
    pub fn validate_documents(
        &mut self,
        document_type: &str,
        documents: &[(Vec<String>, Vec<Vec<serde_json::Value>>)],
    ) -> Result<Vec<ColumnValidationReport>> {
        let mut reports = Vec::new();
        
        for (headers, data) in documents {
            let report = self.validate_document(document_type, headers, data)?;
            reports.push(report);
        }
        
        Ok(reports)
    }

    /// Validate a document with custom validation rules
    pub fn validate_document_with_rules(
        &mut self,
        document_type: &str,
        headers: &[String],
        data: &[Vec<serde_json::Value>],
        custom_rules: Vec<ValidationRule>,
    ) -> Result<ColumnValidationReport> {
        // Temporarily store existing rules
        let existing_rules = self.rules.get(document_type).cloned();
        
        // Set custom rules
        self.add_rules(document_type.to_string(), custom_rules);
        
        // Perform validation
        let result = self.validate_document(document_type, headers, data);
        
        // Restore existing rules
        if let Some(rules) = existing_rules {
            self.add_rules(document_type.to_string(), rules);
        } else {
            self.remove_rules(document_type);
        }
        
        result
    }

    /// Get validation summary for a document type
    pub fn get_validation_summary(&self, document_type: &str) -> Option<ValidationSummary> {
        self.rules.get(document_type).map(|rules| {
            ValidationSummary {
                document_type: document_type.to_string(),
                total_rules: rules.len(),
                required_rules: rules.iter().filter(|r| r.required.unwrap_or(false)).count(),
                optional_rules: rules.iter().filter(|r| !r.required.unwrap_or(false)).count(),
                rule_types: rules.iter()
                    .filter_map(|r| r.rule_type.clone())
                    .collect::<std::collections::HashSet<_>>()
                    .into_iter()
                    .collect(),
            }
        })
    }

    /// Check if a document type is supported
    pub fn is_document_type_supported(&self, document_type: &str) -> bool {
        self.rules.contains_key(document_type)
    }

    /// Get all supported document types
    pub fn get_supported_document_types(&self) -> Vec<String> {
        self.rules.keys().cloned().collect()
    }

    /// Add multiple rule sets at once
    pub fn add_multiple_rule_sets(&mut self, rule_sets: Vec<(String, Vec<ValidationRule>)>) {
        for (document_type, rules) in rule_sets {
            self.add_rules(document_type, rules);
        }
    }

    /// Remove multiple rule sets at once
    pub fn remove_multiple_rule_sets(&mut self, document_types: &[String]) -> Vec<(String, Vec<ValidationRule>)> {
        let mut removed = Vec::new();
        
        for document_type in document_types {
            if let Some(rules) = self.remove_rules(document_type) {
                removed.push((document_type.clone(), rules));
            }
        }
        
        removed
    }

    /// Clone validation rules for a document type
    pub fn clone_rules(&self, source_type: &str, target_type: &str) -> Result<()> {
        if let Some(rules) = self.rules.get(source_type) {
            let cloned_rules = rules.clone();
            self.rules.insert(target_type.to_string(), cloned_rules);
            Ok(())
        } else {
            Err(Error::ValidationError(format!("No rules found for document type: {}", source_type)))
        }
    }

    /// Merge validation rules from another validator
    pub fn merge_rules_from(&mut self, other: &DocumentValidator) {
        for (document_type, rules) in &other.rules {
            if let Some(existing_rules) = self.rules.get_mut(document_type) {
                // Merge rules, avoiding duplicates based on field_id
                let existing_field_ids: std::collections::HashSet<_> = existing_rules.iter()
                    .filter_map(|r| r.field_id.as_ref())
                    .collect();
                
                for rule in rules {
                    if let Some(field_id) = &rule.field_id {
                        if !existing_field_ids.contains(field_id) {
                            existing_rules.push(rule.clone());
                        }
                    } else {
                        existing_rules.push(rule.clone());
                    }
                }
            } else {
                self.rules.insert(document_type.clone(), rules.clone());
            }
        }
    }

    /// Export validation rules to JSON
    pub fn export_rules_to_json(&self) -> Result<String> {
        serde_json::to_string_pretty(&self.rules)
            .map_err(|e| Error::ValidationError(format!("Failed to serialize rules: {}", e)))
    }

    /// Import validation rules from JSON
    pub fn import_rules_from_json(&mut self, json: &str) -> Result<()> {
        let rules: std::collections::HashMap<String, Vec<ValidationRule>> = serde_json::from_str(json)
            .map_err(|e| Error::ValidationError(format!("Failed to deserialize rules: {}", e)))?;
        
        self.rules = rules;
        Ok(())
    }

    /// Validate rules configuration
    pub fn validate_rules_configuration(&self) -> Result<Vec<String>> {
        let mut issues = Vec::new();
        
        for (document_type, rules) in &self.rules {
            // Check for duplicate field IDs
            let mut field_ids = std::collections::HashSet::new();
            for rule in rules {
                if let Some(field_id) = &rule.field_id {
                    if !field_ids.insert(field_id) {
                        issues.push(format!("Duplicate field ID '{}' in document type '{}'", field_id, document_type));
                    }
                }
            }
            
            // Check for missing required fields
            if rules.is_empty() {
                issues.push(format!("No validation rules defined for document type '{}'", document_type));
            }
            
            // Check for invalid rule configurations
            for rule in rules {
                if rule.field_id.is_none() {
                    issues.push(format!("Rule missing field_id in document type '{}'", document_type));
                }
                
                if rule.rule_type.is_none() {
                    issues.push(format!("Rule missing rule_type for field '{}' in document type '{}'", 
                        rule.field_id.as_deref().unwrap_or("unknown"), document_type));
                }
            }
        }
        
        Ok(issues)
    }

    /// Get statistics about validation rules
    pub fn get_rules_statistics(&self) -> ValidationRulesStatistics {
        let total_document_types = self.rules.len();
        let total_rules = self.rules.values().map(|rules| rules.len()).sum();
        let total_required_rules = self.rules.values()
            .flat_map(|rules| rules.iter())
            .filter(|rule| rule.required.unwrap_or(false))
            .count();
        let total_optional_rules = total_rules - total_required_rules;
        
        let rule_types: std::collections::HashSet<_> = self.rules.values()
            .flat_map(|rules| rules.iter())
            .filter_map(|rule| rule.rule_type.as_ref())
            .collect();
        
        ValidationRulesStatistics {
            total_document_types,
            total_rules,
            total_required_rules,
            total_optional_rules,
            unique_rule_types: rule_types.len(),
            rule_types: rule_types.into_iter().cloned().collect(),
        }
    }
}

/// Validation summary for a document type
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ValidationSummary {
    /// Document type
    pub document_type: String,
    /// Total number of validation rules
    pub total_rules: usize,
    /// Number of required rules
    pub required_rules: usize,
    /// Number of optional rules
    pub optional_rules: usize,
    /// Types of validation rules
    pub rule_types: Vec<String>,
}

/// Statistics about validation rules
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ValidationRulesStatistics {
    /// Total number of document types
    pub total_document_types: usize,
    /// Total number of validation rules
    pub total_rules: usize,
    /// Total number of required rules
    pub total_required_rules: usize,
    /// Total number of optional rules
    pub total_optional_rules: usize,
    /// Number of unique rule types
    pub unique_rule_types: usize,
    /// List of rule types
    pub rule_types: Vec<String>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mapping::MappingConfiguration;

    #[test]
    fn test_document_validator_creation() {
        let validator = DocumentValidator::new();
        assert!(validator.rules.is_empty());
        assert!(validator.column_validator.is_none());
    }

    #[test]
    fn test_document_validator_add_rules() {
        let mut validator = DocumentValidator::new();
        let rules = vec![ValidationRule {
            field_id: Some("test_field".to_string()),
            rule_type: Some("required".to_string()),
            required: Some(true),
            ..Default::default()
        }];
        
        validator.add_rules("test_doc".to_string(), rules);
        assert_eq!(validator.rules.len(), 1);
        assert!(validator.is_document_type_supported("test_doc"));
    }

    #[test]
    fn test_document_validator_validation_summary() {
        let mut validator = DocumentValidator::new();
        let rules = vec![
            ValidationRule {
                field_id: Some("required_field".to_string()),
                rule_type: Some("required".to_string()),
                required: Some(true),
                ..Default::default()
            },
            ValidationRule {
                field_id: Some("optional_field".to_string()),
                rule_type: Some("optional".to_string()),
                required: Some(false),
                ..Default::default()
            },
        ];
        
        validator.add_rules("test_doc".to_string(), rules);
        let summary = validator.get_validation_summary("test_doc").unwrap();
        
        assert_eq!(summary.total_rules, 2);
        assert_eq!(summary.required_rules, 1);
        assert_eq!(summary.optional_rules, 1);
    }

    #[test]
    fn test_document_validator_rules_statistics() {
        let mut validator = DocumentValidator::new();
        let rules = vec![
            ValidationRule {
                field_id: Some("field1".to_string()),
                rule_type: Some("required".to_string()),
                required: Some(true),
                ..Default::default()
            },
            ValidationRule {
                field_id: Some("field2".to_string()),
                rule_type: Some("format".to_string()),
                required: Some(false),
                ..Default::default()
            },
        ];
        
        validator.add_rules("doc1".to_string(), rules.clone());
        validator.add_rules("doc2".to_string(), rules);
        
        let stats = validator.get_rules_statistics();
        assert_eq!(stats.total_document_types, 2);
        assert_eq!(stats.total_rules, 4);
        assert_eq!(stats.total_required_rules, 2);
        assert_eq!(stats.total_optional_rules, 2);
    }
}
