//! POA&M mappings configuration for POA&M Excel templates
//!
//! This module contains all structures and configurations specific to
//! POA&M document parsing and mapping to OSCAL assessment results.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// POA&M mappings configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PoamMappings {
    pub description: String,
    pub version: String,
    pub fedramp_v3_mappings: PoamColumnMappings,
    pub risk_mappings: RiskMappings,
    pub finding_mappings: FindingMappings,
    pub milestone_processing: MilestoneProcessing,
    pub quality_checks: QualityChecks,
}

/// POA&M column mappings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PoamColumnMappings {
    pub required_columns: HashMap<String, PoamColumnMapping>,
    pub validation_rules: PoamValidationRules,
}

/// Individual POA&M column mapping
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PoamColumnMapping {
    pub column_names: Vec<String>,
    pub oscal_field: String,
    pub required: bool,
    pub validation: Option<String>,
    pub data_type: Option<String>,
}

/// POA&M validation rules
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PoamValidationRules {
    pub severity_levels: Option<Vec<String>>,
    pub status_values: Option<Vec<String>>,
    pub control_id_pattern: Option<String>,
    pub date_formats: Option<Vec<String>>,
}

/// Risk mappings for POA&M
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RiskMappings {
    pub severity_to_risk_level: HashMap<String, RiskLevel>,
    pub status_to_implementation: HashMap<String, String>,
}

/// Risk level mapping
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RiskLevel {
    pub risk_impact: String,
    pub risk_likelihood: String,
}

/// Finding mappings for origin detection
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FindingMappings {
    pub origin_types: HashMap<String, OriginType>,
}

/// Origin type mapping
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OriginType {
    pub keywords: Vec<String>,
    pub oscal_origin_type: String,
}

/// Milestone processing configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MilestoneProcessing {
    pub patterns: MilestonePatterns,
}

/// Milestone parsing patterns
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MilestonePatterns {
    pub multiple_milestones: MultipleMilestones,
    pub milestone_format: MilestoneFormat,
}

/// Multiple milestone configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MultipleMilestones {
    pub separator_patterns: Vec<String>,
    pub description: String,
}

/// Milestone format configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MilestoneFormat {
    pub patterns: Vec<String>,
    pub groups: Vec<String>,
}

/// Quality checks configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QualityChecks {
    pub required_field_completeness: RequiredFieldCompleteness,
    pub data_consistency: DataConsistency,
    pub control_validation: ControlValidation,
}

/// Required field completeness checks
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RequiredFieldCompleteness {
    pub critical_fields: Vec<String>,
    pub minimum_completion_rate: f64,
}

/// Data consistency checks
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataConsistency {
    pub date_logic: String,
    pub status_logic: String,
}

/// Control validation configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ControlValidation {
    pub verify_control_ids: bool,
    pub validate_against_catalog: String,
}
