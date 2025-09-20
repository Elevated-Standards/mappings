//! Basic configuration structures for document mapping
//!
//! This module contains the core configuration structures used across
//! different mapping types (inventory, POA&M, SSP, etc.).

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Column mapping configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ColumnMapping {
    /// Target OSCAL field name
    pub target_field: String,
    /// Possible column names to match
    pub source_columns: Vec<String>,
    /// Whether this field is required
    pub required: bool,
    /// Data type validation
    pub data_type: Option<String>,
    /// Default value if not found
    pub default_value: Option<serde_json::Value>,
}

/// Comprehensive mapping configuration loaded from JSON files
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MappingConfiguration {
    /// Inventory mappings for asset inventory workbooks
    pub inventory_mappings: Option<crate::mapping::inventory::InventoryMappings>,
    /// POA&M mappings for POA&M Excel templates
    pub poam_mappings: Option<crate::mapping::poam::PoamMappings>,
    /// SSP section mappings for document parsing
    pub ssp_sections: Option<crate::mapping::ssp::SspSections>,
    /// Control framework mappings
    pub controls: Option<crate::mapping::control_document::ControlMappings>,
    /// Document structure definitions
    pub documents: Option<crate::mapping::control_document::DocumentStructures>,
}

/// Validation rules for different data types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationRules {
    pub asset_types: Option<Vec<String>>,
    pub environments: Option<Vec<String>>,
    pub criticality_levels: Option<Vec<String>>,
    pub boolean_values: Option<Vec<String>>,
    pub ip_address_pattern: Option<String>,
    pub mac_address_pattern: Option<String>,
}

/// Component grouping strategies
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComponentGrouping {
    pub strategies: HashMap<String, GroupingStrategy>,
}

/// Individual grouping strategy
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GroupingStrategy {
    pub description: String,
    pub priority: u32,
}

/// Component type mapping
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComponentTypeMapping {
    #[serde(rename = "type")]
    pub component_type: String,
    pub keywords: Vec<String>,
}

/// Security mappings for risk assessment
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityMappings {
    pub criticality_to_impact: HashMap<String, ImpactMapping>,
    pub risk_factors: HashMap<String, RiskFactor>,
}

/// Impact level mapping
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImpactMapping {
    pub confidentiality_impact: String,
    pub integrity_impact: String,
    pub availability_impact: String,
}

/// Risk factor configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RiskFactor {
    pub multiplier: f64,
    pub description: String,
}

/// Control inheritance configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ControlInheritance {
    pub infrastructure_controls: Vec<String>,
    pub platform_controls: Vec<String>,
    pub inheritance_mappings: HashMap<String, InheritanceMapping>,
}

/// Individual inheritance mapping
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InheritanceMapping {
    pub inherited_controls: String,
    pub provider_responsibility: String,
}

/// Loading performance metrics
#[derive(Debug, Clone, Default)]
pub struct LoadingMetrics {
    /// Total loading time in milliseconds
    pub total_load_time_ms: u64,
    /// Individual file loading times
    pub file_load_times: HashMap<String, u64>,
    /// Number of successful loads
    pub successful_loads: u64,
    /// Number of failed loads
    pub failed_loads: u64,
    /// Last load timestamp
    pub last_load_time: Option<std::time::SystemTime>,
}
