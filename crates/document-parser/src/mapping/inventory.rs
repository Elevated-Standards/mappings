//! Inventory mappings configuration for asset inventory workbooks
//!
//! This module contains all structures and configurations specific to
//! inventory document parsing and mapping to OSCAL components.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use crate::mapping::config::{
    ValidationRules, ComponentGrouping, ComponentTypeMapping, 
    SecurityMappings, ControlInheritance
};

/// Inventory mappings configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InventoryMappings {
    pub description: String,
    pub version: String,
    pub fedramp_iiw_mappings: InventoryColumnMappings,
    pub validation_rules: ValidationRules,
    pub component_grouping: ComponentGrouping,
    pub component_type_mappings: HashMap<String, ComponentTypeMapping>,
    pub security_mappings: SecurityMappings,
    pub control_inheritance: ControlInheritance,
}

/// Inventory column mappings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InventoryColumnMappings {
    pub required_columns: HashMap<String, InventoryColumnMapping>,
}

/// Individual inventory column mapping
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InventoryColumnMapping {
    pub column_names: Vec<String>,
    pub field: String,
    pub required: bool,
    pub validation: Option<String>,
}
