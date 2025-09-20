//! Control and document mappings configuration
//!
//! This module contains structures for control framework mappings
//! and document structure definitions from schema files.

use serde::{Deserialize, Serialize};

/// Control mappings configuration (from schema files)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ControlMappings {
    pub metadata: ControlMetadata,
    pub controls: Vec<ControlMapping>,
}

/// Control metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ControlMetadata {
    pub source_file: String,
    pub sheet_name: String,
    pub extraction_date: String,
    pub framework: String,
    pub version: Option<String>,
    pub hash: Option<String>,
}

/// Individual control mapping
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ControlMapping {
    pub control_id: String,
    pub control_title: Option<String>,
    pub control_description: Option<String>,
    pub implementation_status: String,
    pub customer_responsibility: Option<String>,
    pub csp_responsibility: Option<String>,
    pub shared_responsibility: Option<String>,
    pub implementation_guidance: Option<String>,
    pub assessment_procedures: Option<String>,
    pub notes: Option<String>,
    pub control_enhancements: Option<Vec<ControlEnhancement>>,
    pub source: ControlSource,
}

/// Control enhancement
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ControlEnhancement {
    pub enhancement_id: String,
    pub enhancement_title: Option<String>,
    pub implementation_status: String,
    pub notes: Option<String>,
}

/// Control source information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ControlSource {
    pub file: String,
    pub sheet: String,
    pub row: u32,
    pub col_range: Option<String>,
}

/// Document structures configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentStructures {
    pub metadata: DocumentMetadata,
    pub sections: Vec<DocumentSection>,
}

/// Document metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentMetadata {
    pub source_file: String,
    pub source_type: String,
    pub extraction_date: String,
    pub pandoc_version: Option<String>,
    pub hash: Option<String>,
}

/// Document section
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentSection {
    pub id: String,
    pub title: String,
    pub level: u32,
    pub text: String,
    pub tables: Option<Vec<DocumentTable>>,
    pub source: DocumentSectionSource,
}

/// Document table
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentTable {
    pub id: String,
    pub caption: Option<String>,
    pub headers: Vec<String>,
    pub rows: Vec<Vec<String>>,
    pub source: DocumentTableSource,
}

/// Document table source
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentTableSource {
    pub section_id: String,
    pub paragraph: Option<u32>,
    pub table_index: Option<u32>,
}

/// Document section source
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentSectionSource {
    pub file: String,
    pub heading_path: Vec<String>,
    pub paragraph_start: Option<u32>,
    pub paragraph_end: Option<u32>,
}
