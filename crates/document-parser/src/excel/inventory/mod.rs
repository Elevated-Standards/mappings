// Modified: 2025-09-23

//! Inventory-specific Excel parser for FedRAMP Integrated Inventory Workbooks
//! 
//! This module provides specialized parsing capabilities for inventory documents,
//! supporting complex asset structures, relationships, and categorization.

pub mod types;
pub mod parser;
pub mod detector;
pub mod processor;
pub mod validator;
pub mod mapper;
pub mod column_mapper;
pub mod transformers;
pub mod asset_validator;
pub mod network_processor;

pub use types::*;
pub use parser::*;
pub use detector::*;
pub use processor::*;
pub use validator::*;
pub use mapper::*;
pub use column_mapper::*;
pub use transformers::*;
pub use asset_validator::*;
pub use network_processor::*;

use parser::{InventoryParserCore, MockWorkbook};
use crate::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tracing::{debug, info, warn};

/// Main inventory parser for FedRAMP Integrated Inventory Workbooks
#[derive(Debug, Clone)]
pub struct InventoryParser {
    /// Core parser implementation
    core: InventoryParserCore,
}

/// Configuration for inventory parsing operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InventoryParserConfig {
    /// Enable strict validation mode
    pub strict_validation: bool,
    /// Maximum number of assets to process
    pub max_assets: Option<usize>,
    /// Enable relationship mapping
    pub enable_relationships: bool,
    /// Enable asset categorization
    pub enable_categorization: bool,
    /// Timeout for processing operations (seconds)
    pub processing_timeout: u64,
    /// Enable data enrichment
    pub enable_enrichment: bool,
    /// Custom field mappings
    pub custom_mappings: HashMap<String, String>,
}

impl Default for InventoryParserConfig {
    fn default() -> Self {
        Self {
            strict_validation: false,
            max_assets: Some(10000),
            enable_relationships: true,
            enable_categorization: true,
            processing_timeout: 300,
            enable_enrichment: true,
            custom_mappings: HashMap::new(),
        }
    }
}

impl InventoryParser {
    /// Create a new inventory parser with default configuration
    pub fn new() -> Self {
        Self::with_config(InventoryParserConfig::default())
    }

    /// Create a new inventory parser with custom configuration
    pub fn with_config(config: InventoryParserConfig) -> Self {
        let core = InventoryParserCore::with_config(config);
        Self { core }
    }

    /// Parse inventory from Excel file
    pub async fn parse_inventory(&mut self, file_path: &str) -> Result<InventoryDocument> {
        self.core.parse_from_file(file_path).await
    }



    /// Get parser configuration
    pub fn get_config(&self) -> &InventoryParserConfig {
        self.core.get_config()
    }

    /// Update parser configuration
    pub fn update_config(&mut self, config: InventoryParserConfig) {
        self.core.update_config(config);
    }
}

impl Default for InventoryParser {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_inventory_parser_creation() {
        let parser = InventoryParser::new();
        assert!(parser.is_ok());
    }

    #[test]
    fn test_inventory_parser_config() {
        let config = InventoryParserConfig {
            strict_validation: true,
            max_assets: Some(5000),
            enable_relationships: false,
            ..Default::default()
        };

        let parser = InventoryParser::with_config(config.clone());
        assert!(parser.is_ok());

        let parser = parser.unwrap();
        assert_eq!(parser.get_config().strict_validation, true);
        assert_eq!(parser.get_config().max_assets, Some(5000));
        assert_eq!(parser.get_config().enable_relationships, false);
    }

    #[test]
    fn test_asset_type_determination() {
        let parser = InventoryParser::new().unwrap();
        let template_info = InventoryTemplateInfo {
            template_type: InventoryTemplateType::FedRampIntegrated,
            version: "1.0".to_string(),
            asset_worksheets: vec!["Hardware".to_string()],
            relationship_worksheets: vec![],
            column_mappings: HashMap::new(),
        };

        assert_eq!(
            parser.determine_asset_type("Hardware Inventory", &template_info),
            AssetType::Hardware
        );
        assert_eq!(
            parser.determine_asset_type("Software Applications", &template_info),
            AssetType::Software
        );
        assert_eq!(
            parser.determine_asset_type("Network Devices", &template_info),
            AssetType::Network
        );
    }
}
