// Modified: 2025-09-23

//! Core parsing functionality for inventory Excel files
//! 
//! This module provides the main parsing interface and orchestrates
//! the various components of inventory processing.

use super::types::*;
use super::{InventoryTemplateDetector, AssetProcessor, RelationshipMapper, InventoryValidator};
use crate::Result;
use fedramp_core::Error;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tracing::{debug, info, warn};

/// Core inventory parser implementation
#[derive(Debug, Clone)]
pub struct InventoryParserCore {
    /// Configuration for parsing operations
    config: InventoryParserConfig,
    /// Template detector
    template_detector: InventoryTemplateDetector,
    /// Asset processor
    asset_processor: AssetProcessor,
    /// Relationship mapper
    relationship_mapper: RelationshipMapper,
    /// Validator
    validator: InventoryValidator,
}

impl InventoryParserCore {
    /// Create a new inventory parser core
    pub fn new() -> Self {
        Self::with_config(InventoryParserConfig::default())
    }

    /// Create parser core with custom configuration
    pub fn with_config(config: InventoryParserConfig) -> Self {
        let template_detector = InventoryTemplateDetector::new();
        let asset_processor = AssetProcessor::new();
        let relationship_mapper = RelationshipMapper::new();
        let validator = InventoryValidator::new();

        Self {
            config,
            template_detector,
            asset_processor,
            relationship_mapper,
            validator,
        }
    }

    /// Parse inventory from file path
    pub async fn parse_from_file(&mut self, file_path: &str) -> Result<InventoryDocument> {
        info!("Starting inventory parsing from file: {}", file_path);

        // For now, create a mock workbook structure
        let mock_workbook = self.create_mock_workbook(file_path)?;
        
        // Detect template
        let template_info = self.template_detector.detect_template(&mock_workbook)?;
        
        debug!("Detected template: {:?}", template_info);

        // Parse assets
        let assets = self.parse_assets_from_workbook(&mock_workbook, &template_info).await?;
        
        info!("Parsed {} assets from inventory", assets.len());

        // Process relationships if enabled
        let relationships = if self.config.enable_relationships {
            self.relationship_mapper.map_relationships(&assets, &mock_workbook, &template_info).await?
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

    /// Parse assets from workbook
    async fn parse_assets_from_workbook(
        &mut self,
        workbook: &MockWorkbook,
        template_info: &InventoryTemplateInfo,
    ) -> Result<Vec<Asset>> {
        let mut all_assets = Vec::new();

        for worksheet_name in &template_info.asset_worksheets {
            debug!("Parsing assets from worksheet: {}", worksheet_name);

            let worksheet_assets = self.parse_worksheet_assets(workbook, worksheet_name, template_info).await?;
            all_assets.extend(worksheet_assets);

            // Check max assets limit
            if let Some(max_assets) = self.config.max_assets {
                if all_assets.len() >= max_assets {
                    warn!("Reached maximum asset limit: {}", max_assets);
                    break;
                }
            }
        }

        Ok(all_assets)
    }

    /// Parse assets from a specific worksheet
    async fn parse_worksheet_assets(
        &mut self,
        workbook: &MockWorkbook,
        worksheet_name: &str,
        template_info: &InventoryTemplateInfo,
    ) -> Result<Vec<Asset>> {
        debug!("Processing worksheet: {}", worksheet_name);

        // Get worksheet data
        let worksheet_data = workbook.get_worksheet_data(worksheet_name)?;
        let asset_type = self.determine_asset_type(worksheet_name, template_info);
        
        let mut assets = Vec::new();

        for (row_index, row_data) in worksheet_data.iter().enumerate() {
            match self.asset_processor.process_asset_row(
                row_data,
                &[], // Headers would be extracted in real implementation
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

    /// Create a mock workbook for testing/placeholder purposes
    fn create_mock_workbook(&self, file_path: &str) -> Result<MockWorkbook> {
        debug!("Creating mock workbook for: {}", file_path);

        let mut worksheets = HashMap::new();
        
        // Create sample hardware inventory data
        let hardware_data = vec![
            {
                let mut row = HashMap::new();
                row.insert("Asset ID".to_string(), "HW001".to_string());
                row.insert("Asset Name".to_string(), "Web Server 01".to_string());
                row.insert("Asset Type".to_string(), "Server".to_string());
                row.insert("Owner".to_string(), "IT Team".to_string());
                row.insert("Environment".to_string(), "Production".to_string());
                row.insert("Criticality".to_string(), "High".to_string());
                row.insert("IP Address".to_string(), "192.168.1.10".to_string());
                row
            },
            {
                let mut row = HashMap::new();
                row.insert("Asset ID".to_string(), "HW002".to_string());
                row.insert("Asset Name".to_string(), "Database Server".to_string());
                row.insert("Asset Type".to_string(), "Server".to_string());
                row.insert("Owner".to_string(), "Database Team".to_string());
                row.insert("Environment".to_string(), "Production".to_string());
                row.insert("Criticality".to_string(), "Critical".to_string());
                row.insert("IP Address".to_string(), "192.168.1.20".to_string());
                row
            },
        ];
        worksheets.insert("Hardware Inventory".to_string(), hardware_data);

        // Create sample software inventory data
        let software_data = vec![
            {
                let mut row = HashMap::new();
                row.insert("Asset ID".to_string(), "SW001".to_string());
                row.insert("Asset Name".to_string(), "Apache HTTP Server".to_string());
                row.insert("Asset Type".to_string(), "Web Server".to_string());
                row.insert("Vendor".to_string(), "Apache Software Foundation".to_string());
                row.insert("Version".to_string(), "2.4.41".to_string());
                row.insert("Owner".to_string(), "IT Team".to_string());
                row
            },
            {
                let mut row = HashMap::new();
                row.insert("Asset ID".to_string(), "SW002".to_string());
                row.insert("Asset Name".to_string(), "PostgreSQL".to_string());
                row.insert("Asset Type".to_string(), "Database".to_string());
                row.insert("Vendor".to_string(), "PostgreSQL Global Development Group".to_string());
                row.insert("Version".to_string(), "13.4".to_string());
                row.insert("Owner".to_string(), "Database Team".to_string());
                row
            },
        ];
        worksheets.insert("Software Inventory".to_string(), software_data);

        Ok(MockWorkbook {
            file_path: file_path.to_string(),
            worksheets,
        })
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

/// Mock workbook structure for testing
#[derive(Debug, Clone)]
pub struct MockWorkbook {
    /// Source file path
    pub file_path: String,
    /// Worksheet data
    pub worksheets: HashMap<String, Vec<HashMap<String, String>>>,
}

impl MockWorkbook {
    /// Get worksheet data by name
    pub fn get_worksheet_data(&self, worksheet_name: &str) -> Result<&Vec<HashMap<String, String>>> {
        self.worksheets.get(worksheet_name)
            .ok_or_else(|| Error::validation(format!("Worksheet '{}' not found", worksheet_name)))
    }

    /// Get worksheet names
    pub fn get_worksheet_names(&self) -> Vec<String> {
        self.worksheets.keys().cloned().collect()
    }

    /// Check if worksheet exists
    pub fn has_worksheet(&self, worksheet_name: &str) -> bool {
        self.worksheets.contains_key(worksheet_name)
    }
}

impl Default for InventoryParserCore {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parser_core_creation() {
        let parser = InventoryParserCore::new();
        assert!(parser.config.enable_categorization);
    }

    #[test]
    fn test_mock_workbook_creation() {
        let parser = InventoryParserCore::new();
        let workbook = parser.create_mock_workbook("test.xlsx").unwrap();
        
        assert_eq!(workbook.file_path, "test.xlsx");
        assert!(workbook.has_worksheet("Hardware Inventory"));
        assert!(workbook.has_worksheet("Software Inventory"));
    }

    #[test]
    fn test_asset_type_determination() {
        let parser = InventoryParserCore::new();
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

    #[tokio::test]
    async fn test_inventory_parsing() {
        let mut parser = InventoryParserCore::new();
        let result = parser.parse_from_file("test_inventory.xlsx").await;
        
        assert!(result.is_ok());
        let inventory = result.unwrap();
        assert!(!inventory.assets.is_empty());
        assert_eq!(inventory.metadata.source_file, "test_inventory.xlsx");
    }
}
