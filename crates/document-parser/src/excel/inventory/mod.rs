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

pub use types::*;
pub use parser::*;
pub use detector::*;
pub use processor::*;
pub use validator::*;
pub use mapper::*;

use crate::excel::core::ExcelParser;
use crate::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tracing::{debug, info, warn};

/// Main inventory parser for FedRAMP Integrated Inventory Workbooks
#[derive(Debug, Clone)]
pub struct InventoryParser {
    /// Base Excel parser for file operations
    base_parser: ExcelParser,
    /// Template detector for identifying inventory formats
    template_detector: InventoryTemplateDetector,
    /// Asset processor for handling different asset types
    asset_processor: AssetProcessor,
    /// Relationship mapper for asset dependencies
    relationship_mapper: RelationshipMapper,
    /// Validator for inventory data integrity
    validator: InventoryValidator,
    /// Configuration for parsing behavior
    config: InventoryParserConfig,
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
    pub fn new() -> Result<Self> {
        Self::with_config(InventoryParserConfig::default())
    }

    /// Create a new inventory parser with custom configuration
    pub fn with_config(config: InventoryParserConfig) -> Result<Self> {
        let base_parser = ExcelParser::new()?;
        let template_detector = InventoryTemplateDetector::new();
        let asset_processor = AssetProcessor::new();
        let relationship_mapper = RelationshipMapper::new();
        let validator = InventoryValidator::new();

        Ok(Self {
            base_parser,
            template_detector,
            asset_processor,
            relationship_mapper,
            validator,
            config,
        })
    }

    /// Parse inventory from Excel file
    pub async fn parse_inventory(&mut self, file_path: &str) -> Result<InventoryDocument> {
        info!("Starting inventory parsing for file: {}", file_path);

        // Load and detect template
        let workbook = self.base_parser.load_workbook(file_path).await?;
        let template_info = self.template_detector.detect_template(&workbook)?;
        
        debug!("Detected template: {:?}", template_info);

        // Parse assets from worksheets
        let mut assets = Vec::new();
        for worksheet_name in &template_info.asset_worksheets {
            let worksheet_assets = self.parse_worksheet_assets(&workbook, worksheet_name, &template_info).await?;
            assets.extend(worksheet_assets);
        }

        info!("Parsed {} assets from inventory", assets.len());

        // Process relationships if enabled
        let relationships = if self.config.enable_relationships {
            self.relationship_mapper.map_relationships(&assets, &workbook, &template_info).await?
        } else {
            Vec::new()
        };

        // Validate inventory data
        let validation_results = if self.config.strict_validation {
            self.validator.validate_inventory(&assets, &relationships).await?
        } else {
            InventoryValidationResults::default()
        };

        // Create inventory document
        let inventory = InventoryDocument {
            metadata: InventoryMetadata {
                template_type: template_info.template_type.clone(),
                template_version: template_info.version.clone(),
                source_file: file_path.to_string(),
                parsed_at: chrono::Utc::now(),
                asset_count: assets.len(),
                relationship_count: relationships.len(),
            },
            assets,
            relationships,
            validation_results,
            template_info,
        };

        info!("Inventory parsing completed successfully");
        Ok(inventory)
    }

    /// Parse assets from a specific worksheet
    async fn parse_worksheet_assets(
        &mut self,
        workbook: &crate::excel::core::Workbook,
        worksheet_name: &str,
        template_info: &InventoryTemplateInfo,
    ) -> Result<Vec<Asset>> {
        debug!("Parsing assets from worksheet: {}", worksheet_name);

        let worksheet = workbook.get_worksheet(worksheet_name)?;
        let asset_type = self.determine_asset_type(worksheet_name, template_info);
        
        let mut assets = Vec::new();
        let headers = self.extract_headers(&worksheet)?;
        let data_rows = self.extract_data_rows(&worksheet, &headers)?;

        for (row_index, row_data) in data_rows.iter().enumerate() {
            match self.asset_processor.process_asset_row(
                row_data,
                &headers,
                asset_type.clone(),
                template_info,
                row_index + 2, // Account for header row
            ).await {
                Ok(asset) => assets.push(asset),
                Err(e) => {
                    warn!("Failed to process asset at row {}: {}", row_index + 2, e);
                    if self.config.strict_validation {
                        return Err(e);
                    }
                }
            }

            // Check max assets limit
            if let Some(max_assets) = self.config.max_assets {
                if assets.len() >= max_assets {
                    warn!("Reached maximum asset limit: {}", max_assets);
                    break;
                }
            }
        }

        debug!("Parsed {} assets from worksheet {}", assets.len(), worksheet_name);
        Ok(assets)
    }

    /// Determine asset type from worksheet name
    fn determine_asset_type(&self, worksheet_name: &str, template_info: &InventoryTemplateInfo) -> AssetType {
        let name_lower = worksheet_name.to_lowercase();
        
        if name_lower.contains("hardware") || name_lower.contains("server") || name_lower.contains("device") {
            AssetType::Hardware
        } else if name_lower.contains("software") || name_lower.contains("application") || name_lower.contains("app") {
            AssetType::Software
        } else if name_lower.contains("network") || name_lower.contains("router") || name_lower.contains("switch") {
            AssetType::Network
        } else if name_lower.contains("virtual") || name_lower.contains("vm") || name_lower.contains("container") {
            AssetType::Virtual
        } else if name_lower.contains("data") || name_lower.contains("database") || name_lower.contains("storage") {
            AssetType::Data
        } else if name_lower.contains("cloud") || name_lower.contains("service") {
            AssetType::Cloud
        } else {
            // Default based on template type
            match template_info.template_type {
                InventoryTemplateType::FedRampIntegrated => AssetType::Hardware,
                InventoryTemplateType::NetworkInventory => AssetType::Network,
                InventoryTemplateType::SoftwareInventory => AssetType::Software,
                InventoryTemplateType::Custom => AssetType::Hardware,
            }
        }
    }

    /// Extract headers from worksheet
    fn extract_headers(&self, worksheet: &crate::excel::core::Worksheet) -> Result<Vec<String>> {
        // Implementation would extract headers from first row
        // This is a placeholder for the actual implementation
        Ok(vec![
            "Asset ID".to_string(),
            "Asset Name".to_string(),
            "Asset Type".to_string(),
            "Description".to_string(),
            "Owner".to_string(),
            "Environment".to_string(),
            "Criticality".to_string(),
        ])
    }

    /// Extract data rows from worksheet
    fn extract_data_rows(
        &self,
        worksheet: &crate::excel::core::Worksheet,
        headers: &[String],
    ) -> Result<Vec<HashMap<String, String>>> {
        // Implementation would extract data rows
        // This is a placeholder for the actual implementation
        Ok(Vec::new())
    }

    /// Get parser configuration
    pub fn get_config(&self) -> &InventoryParserConfig {
        &self.config
    }

    /// Update parser configuration
    pub fn update_config(&mut self, config: InventoryParserConfig) {
        self.config = config;
    }
}

impl Default for InventoryParser {
    fn default() -> Self {
        Self::new().expect("Failed to create default inventory parser")
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
