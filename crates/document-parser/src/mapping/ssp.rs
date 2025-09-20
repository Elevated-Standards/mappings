//! SSP sections configuration for document parsing
//!
//! This module contains all structures and configurations specific to
//! System Security Plan (SSP) document parsing and section mapping.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// SSP sections configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SspSections {
    pub section_mappings: SectionMappings,
    pub control_extraction: ControlExtraction,
    pub table_mappings: TableMappings,
}

/// Section mappings configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SectionMappings {
    pub description: String,
    pub version: String,
    pub mappings: HashMap<String, SectionMapping>,
}

/// Individual section mapping
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SectionMapping {
    pub keywords: Vec<String>,
    pub target: String,
    pub required: bool,
    pub extract_patterns: Option<HashMap<String, String>>,
}

/// Control extraction patterns
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ControlExtraction {
    pub patterns: Vec<ExtractionPattern>,
}

/// Individual extraction pattern
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExtractionPattern {
    pub name: String,
    pub regex: String,
    pub description: String,
}

/// Table mappings for structured data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TableMappings {
    pub responsibility_matrix: ResponsibilityMatrix,
    pub inventory_summary: InventorySummary,
}

/// Responsibility matrix mapping
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResponsibilityMatrix {
    pub keywords: Vec<String>,
    pub columns: ResponsibilityColumns,
}

/// Responsibility matrix columns
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResponsibilityColumns {
    pub control_id: Vec<String>,
    pub customer_responsibility: Vec<String>,
    pub csp_responsibility: Vec<String>,
    pub shared_responsibility: Vec<String>,
}

/// Inventory summary mapping
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InventorySummary {
    pub keywords: Vec<String>,
    pub columns: InventorySummaryColumns,
}

/// Inventory summary columns
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InventorySummaryColumns {
    pub component_name: Vec<String>,
    pub component_type: Vec<String>,
    pub criticality: Vec<String>,
    pub environment: Vec<String>,
}
