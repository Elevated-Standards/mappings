//! POA&M field mapping functionality
//! Modified: 2025-01-22

use serde_json::Value;
use chrono::Utc;
use std::collections::HashMap;
use fedramp_core::Result;

use super::types::*;

impl PoamFieldMapper {
    /// Create a new field mapper with default configuration
    pub fn new() -> Self {
        Self {
            mapping_config: PoamMappingConfig::default(),
        }
    }

    /// Create a new field mapper with custom configuration
    pub fn with_config(config: PoamMappingConfig) -> Self {
        Self {
            mapping_config: config,
        }
    }

    /// Get the mapping configuration
    pub fn mapping_config(&self) -> &PoamMappingConfig {
        &self.mapping_config
    }

    /// Set the mapping configuration
    pub fn set_mapping_config(&mut self, config: PoamMappingConfig) {
        self.mapping_config = config;
    }

    /// Map row data to POA&M fields based on column mappings
    pub fn map_row_to_poam(&self, row_data: &[Value], column_mappings: &HashMap<String, String>) -> Result<PoamItem> {
        // This is a simplified implementation
        // In practice, this would use the mapping configuration to transform data

        let unique_id = self.extract_field(row_data, column_mappings, "Unique ID")
            .unwrap_or_else(|| format!("POAM-{}", uuid::Uuid::new_v4()));

        let weakness_description = self.extract_field(row_data, column_mappings, "Weakness Description")
            .unwrap_or_else(|| "No description provided".to_string());

        Ok(PoamItem {
            unique_id,
            control_id: self.extract_field(row_data, column_mappings, "Control ID"),
            cci: self.extract_field(row_data, column_mappings, "CCI"),
            system_name: self.extract_field(row_data, column_mappings, "System Name"),
            vulnerability_id: self.extract_field(row_data, column_mappings, "Vulnerability ID"),
            weakness_description,
            source_identifier: self.extract_field(row_data, column_mappings, "Source"),
            asset_identifier: self.extract_field(row_data, column_mappings, "Asset ID"),
            security_controls: Vec::new(), // TODO: Parse from control_id
            office_organization: self.extract_field(row_data, column_mappings, "Office/Organization"),
            security_control_names: Vec::new(),
            implementation_guidance: self.extract_field(row_data, column_mappings, "Implementation Guidance"),
            severity: self.parse_severity(&self.extract_field(row_data, column_mappings, "Severity")),
            likelihood: None, // TODO: Parse likelihood
            impact: None, // TODO: Parse impact
            risk_rating: None, // TODO: Calculate risk rating
            status: self.parse_status(&self.extract_field(row_data, column_mappings, "Status")),
            scheduled_completion_date: None, // TODO: Parse date
            actual_completion_date: None, // TODO: Parse date
            milestones: Vec::new(),
            resources: Vec::new(),
            point_of_contact: self.extract_field(row_data, column_mappings, "Point of Contact"),
            remediation_plan: self.extract_field(row_data, column_mappings, "Remediation Plan"),
            affected_assets: Vec::new(),
            comments: self.extract_field(row_data, column_mappings, "Comments"),
            vendor_information: self.extract_field(row_data, column_mappings, "Vendor Information"),
            cost_estimate: None, // TODO: Parse cost
            detection_date: None, // TODO: Parse date
            last_updated: Utc::now(),
        })
    }

    /// Extract field value from row data using column mappings
    fn extract_field(&self, _row_data: &[Value], _column_mappings: &HashMap<String, String>, _field_name: &str) -> Option<String> {
        // Find the column index for this field
        // if let Some(_column_name) = column_mappings.get(field_name) {
        //     // TODO: Find column index by name and extract value
        //     // For now, return None as this requires header-to-index mapping
        // }
        None
    }

    /// Parse severity from string value
    fn parse_severity(&self, value: &Option<String>) -> PoamSeverity {
        let severity_str = value.as_ref().map(|s| s.to_lowercase()).unwrap_or_default();
        match severity_str.as_str() {
            "critical" => PoamSeverity::Critical,
            "high" => PoamSeverity::High,
            "medium" | "moderate" => PoamSeverity::Medium,
            "low" => PoamSeverity::Low,
            "info" | "informational" => PoamSeverity::Info,
            _ => PoamSeverity::Medium, // Default
        }
    }

    /// Parse status from string value
    fn parse_status(&self, value: &Option<String>) -> PoamStatus {
        let status_str = value.as_ref().map(|s| s.to_lowercase()).unwrap_or_default();
        match status_str.as_str() {
            "open" => PoamStatus::Open,
            "in progress" | "inprogress" | "in-progress" => PoamStatus::InProgress,
            "completed" | "complete" => PoamStatus::Completed,
            "risk accepted" | "riskaccepted" | "accepted" => PoamStatus::RiskAccepted,
            "false positive" | "falsepositive" => PoamStatus::FalsePositive,
            "deferred" => PoamStatus::Deferred,
            _ => PoamStatus::Open, // Default
        }
    }

    /// Parse likelihood from string value
    pub fn parse_likelihood(&self, value: &Option<String>) -> Option<PoamLikelihood> {
        let likelihood_str = value.as_ref().map(|s| s.to_lowercase()).unwrap_or_default();
        match likelihood_str.as_str() {
            "very high" | "veryhigh" => Some(PoamLikelihood::VeryHigh),
            "high" => Some(PoamLikelihood::High),
            "medium" | "moderate" => Some(PoamLikelihood::Medium),
            "low" => Some(PoamLikelihood::Low),
            "very low" | "verylow" => Some(PoamLikelihood::VeryLow),
            _ => None,
        }
    }

    /// Parse impact from string value
    pub fn parse_impact(&self, value: &Option<String>) -> Option<PoamImpact> {
        let impact_str = value.as_ref().map(|s| s.to_lowercase()).unwrap_or_default();
        match impact_str.as_str() {
            "very high" | "veryhigh" => Some(PoamImpact::VeryHigh),
            "high" => Some(PoamImpact::High),
            "medium" | "moderate" => Some(PoamImpact::Medium),
            "low" => Some(PoamImpact::Low),
            "very low" | "verylow" => Some(PoamImpact::VeryLow),
            _ => None,
        }
    }

    /// Parse milestone status from string value
    pub fn parse_milestone_status(&self, value: &Option<String>) -> MilestoneStatus {
        let status_str = value.as_ref().map(|s| s.to_lowercase()).unwrap_or_default();
        match status_str.as_str() {
            "not started" | "notstarted" => MilestoneStatus::NotStarted,
            "in progress" | "inprogress" | "in-progress" => MilestoneStatus::InProgress,
            "completed" | "complete" => MilestoneStatus::Completed,
            "delayed" => MilestoneStatus::Delayed,
            "cancelled" | "canceled" => MilestoneStatus::Cancelled,
            _ => MilestoneStatus::NotStarted, // Default
        }
    }

    /// Parse resource type from string value
    pub fn parse_resource_type(&self, value: &Option<String>) -> ResourceType {
        let resource_str = value.as_ref().map(|s| s.to_lowercase()).unwrap_or_default();
        match resource_str.as_str() {
            "personnel" | "people" | "staff" => ResourceType::Personnel,
            "hardware" | "hw" => ResourceType::Hardware,
            "software" | "sw" => ResourceType::Software,
            "training" | "education" => ResourceType::Training,
            "consulting" | "consultant" => ResourceType::Consulting,
            _ => ResourceType::Other, // Default
        }
    }

    /// Extract string value from JSON Value
    pub fn extract_string_value(&self, value: &Value) -> Option<String> {
        match value {
            Value::String(s) => Some(s.clone()),
            Value::Number(n) => Some(n.to_string()),
            Value::Bool(b) => Some(b.to_string()),
            _ => None,
        }
    }

    /// Extract numeric value from JSON Value
    pub fn extract_numeric_value(&self, value: &Value) -> Option<f64> {
        match value {
            Value::Number(n) => n.as_f64(),
            Value::String(s) => s.parse().ok(),
            _ => None,
        }
    }

    /// Extract boolean value from JSON Value
    pub fn extract_boolean_value(&self, value: &Value) -> Option<bool> {
        match value {
            Value::Bool(b) => Some(*b),
            Value::String(s) => match s.to_lowercase().as_str() {
                "true" | "yes" | "1" | "on" => Some(true),
                "false" | "no" | "0" | "off" => Some(false),
                _ => None,
            },
            Value::Number(n) => {
                if let Some(i) = n.as_i64() {
                    Some(i != 0)
                } else {
                    None
                }
            },
            _ => None,
        }
    }

    /// Validate field mapping configuration
    pub fn validate_mapping_config(&self) -> Vec<String> {
        let mut errors = Vec::new();

        // Check for required field mappings
        let required_fields = ["unique_id", "weakness_description", "severity", "status"];
        for field in &required_fields {
            if !self.mapping_config.required_columns.contains_key(*field) {
                errors.push(format!("Missing required field mapping: {}", field));
            }
        }

        // Check for circular references in validation rules
        for (rule_name, rule) in &self.mapping_config.validation_rules {
            if let Some(custom_validator) = &rule.custom_validator {
                if custom_validator == rule_name {
                    errors.push(format!("Circular reference in validation rule: {}", rule_name));
                }
            }
        }

        errors
    }

    /// Get field mapping by name
    pub fn get_field_mapping(&self, field_name: &str) -> Option<&PoamFieldMapping> {
        self.mapping_config.required_columns.get(field_name)
            .or_else(|| self.mapping_config.optional_columns.get(field_name))
    }

    /// Get validation rule by name
    pub fn get_validation_rule(&self, rule_name: &str) -> Option<&ValidationRule> {
        self.mapping_config.validation_rules.get(rule_name)
    }
}

impl Default for PoamFieldMapper {
    fn default() -> Self {
        Self::new()
    }
}
