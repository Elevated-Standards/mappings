// Modified: 2025-09-23

//! Inventory column mapping functionality for OSCAL component generation
//! 
//! This module provides specialized column mapping for inventory Excel files,
//! using the inventory_mappings.json configuration to map columns to OSCAL
//! component fields with asset type-specific handling.

use super::types::*;
use super::transformers::*;
use crate::mapping::loader::MappingConfigurationLoader;
use crate::mapping::inventory::{InventoryMappings, InventoryColumnMapping};
use crate::Result;
use fedramp_core::Error;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tracing::{debug, info, warn};

/// Configuration for inventory column mapping (using existing InventoryMappings)
pub type InventoryMappingConfig = InventoryMappings;

/// Asset-specific mapping configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssetMapping {
    /// Required columns for this asset type
    pub required_columns: HashMap<String, ColumnMapping>,
    /// Optional columns
    pub optional_columns: HashMap<String, ColumnMapping>,
    /// Asset-specific transformations
    pub transformations: Vec<FieldTransformation>,
    /// Validation rules specific to this asset type
    pub validation_rules: Vec<String>,
}

/// Column mapping definition (using existing InventoryColumnMapping)
pub type ColumnMapping = InventoryColumnMapping;

/// Component mapping for OSCAL generation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComponentMapping {
    /// Source fields from inventory
    pub source_fields: Vec<String>,
    /// Target component type in OSCAL
    pub target_component: String,
    /// Component type classification
    pub component_type: ComponentType,
    /// Property mappings
    pub properties: Vec<PropertyMapping>,
    /// Control implementation mappings
    pub control_implementations: Vec<ControlImplementationMapping>,
}

/// Property mapping for component properties
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PropertyMapping {
    /// Source field name
    pub source_field: String,
    /// Target property name
    pub target_property: String,
    /// Property namespace
    pub namespace: Option<String>,
    /// Property class
    pub class: Option<String>,
}

/// Control implementation mapping
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ControlImplementationMapping {
    /// Control ID
    pub control_id: String,
    /// Implementation description template
    pub description_template: String,
    /// Source fields for implementation details
    pub source_fields: Vec<String>,
}

/// Relationship mapping between components
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RelationshipMapping {
    /// Relationship type
    pub relationship_type: String,
    /// Source component field
    pub source_field: String,
    /// Target component field
    pub target_field: String,
    /// Relationship direction
    pub direction: RelationshipDirection,
}

/// Field transformation rule
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FieldTransformation {
    /// Source field name
    pub source_field: String,
    /// Transformation type
    pub transformation_type: TransformationType,
    /// Transformation parameters
    pub parameters: HashMap<String, String>,
}

/// Validation rule definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationRule {
    /// Rule type
    pub rule_type: String,
    /// Rule parameters
    pub parameters: HashMap<String, String>,
    /// Error message template
    pub error_message: String,
}

/// Component type for OSCAL
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum ComponentType {
    /// Hardware component
    Hardware,
    /// Software component
    Software,
    /// Service component
    Service,
    /// Network component
    Network,
    /// Data component
    Data,
    /// Other component type
    Other(String),
}

/// Relationship direction
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum RelationshipDirection {
    /// Bidirectional relationship
    Bidirectional,
    /// Source to target
    SourceToTarget,
    /// Target to source
    TargetToSource,
}

/// Transformation type
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum TransformationType {
    /// Normalize text (lowercase, trim, etc.)
    Normalize,
    /// Format as UUID
    FormatUuid,
    /// Parse boolean value
    ParseBoolean,
    /// Validate IP address
    ValidateIpAddress,
    /// Validate MAC address
    ValidateMacAddress,
    /// Custom transformation
    Custom(String),
}

/// Result of inventory mapping operation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InventoryMappingResult {
    /// Generated OSCAL components
    pub components: Vec<OscalComponent>,
    /// Component relationships
    pub relationships: Vec<ComponentRelationship>,
    /// Assets that couldn't be mapped
    pub unmapped_assets: Vec<String>,
    /// Overall mapping confidence score
    pub mapping_confidence: f64,
    /// Validation results
    pub validation_results: Vec<MappingValidationResult>,
    /// Mapping statistics
    pub statistics: MappingStatistics,
}

/// OSCAL component representation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OscalComponent {
    /// Component UUID
    pub uuid: String,
    /// Component title
    pub title: String,
    /// Component description
    pub description: Option<String>,
    /// Component type
    pub component_type: String,
    /// Component properties
    pub props: HashMap<String, String>,
    /// Responsible roles
    pub responsible_roles: HashMap<String, String>,
    /// Control implementations
    pub control_implementations: Vec<ControlImplementation>,
}

/// Component relationship
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComponentRelationship {
    /// Source component UUID
    pub source_uuid: String,
    /// Target component UUID
    pub target_uuid: String,
    /// Relationship type
    pub relationship_type: String,
    /// Relationship description
    pub description: Option<String>,
}

/// Control implementation details
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ControlImplementation {
    /// Control ID
    pub control_id: String,
    /// Implementation description
    pub description: String,
    /// Implementation status
    pub implementation_status: String,
    /// Source asset information
    pub source_asset: Option<String>,
}

/// Mapping validation result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MappingValidationResult {
    /// Asset ID that was validated
    pub asset_id: String,
    /// Validation status
    pub status: ValidationStatus,
    /// Validation message
    pub message: String,
    /// Field that failed validation (if applicable)
    pub field: Option<String>,
}

/// Mapping statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MappingStatistics {
    /// Total assets processed
    pub total_assets: usize,
    /// Successfully mapped assets
    pub mapped_assets: usize,
    /// Failed mappings
    pub failed_mappings: usize,
    /// Components generated
    pub components_generated: usize,
    /// Relationships discovered
    pub relationships_discovered: usize,
    /// Processing time in milliseconds
    pub processing_time_ms: u64,
}

/// Validation status
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum ValidationStatus {
    /// Validation passed
    Passed,
    /// Validation failed
    Failed,
    /// Validation warning
    Warning,
    /// Validation skipped
    Skipped,
}

/// Main inventory column mapper
pub struct InventoryColumnMapper {
    /// Mapping configuration
    mapping_config: InventoryMappings,
    /// Mapping loader for configuration files
    mapping_loader: MappingConfigurationLoader,
    /// Asset transformers by type
    asset_transformers: HashMap<AssetType, Box<dyn AssetTransformer>>,
    /// Relationship mappings
    relationship_mappings: Vec<RelationshipMapping>,
}



impl InventoryColumnMapper {
    /// Create a new inventory column mapper
    pub async fn new() -> Result<Self> {
        let mut mapping_loader = MappingConfigurationLoader::new("mappings");
        let mapping_config = mapping_loader.load_inventory_mappings().await?;

        let asset_transformers = Self::create_asset_transformers()?;
        let relationship_mappings = Vec::new();

        Ok(Self {
            mapping_config,
            mapping_loader,
            asset_transformers,
            relationship_mappings,
        })
    }

    /// Create asset transformers for different asset types
    fn create_asset_transformers() -> Result<HashMap<AssetType, Box<dyn AssetTransformer>>> {
        let mut transformers: HashMap<AssetType, Box<dyn AssetTransformer>> = HashMap::new();
        
        // Add transformers for each asset type
        transformers.insert(AssetType::Hardware, Box::new(HardwareTransformer::new()));
        transformers.insert(AssetType::Software, Box::new(SoftwareTransformer::new()));
        transformers.insert(AssetType::Network, Box::new(NetworkTransformer::new()));
        transformers.insert(AssetType::Virtual, Box::new(VirtualTransformer::new()));
        transformers.insert(AssetType::Data, Box::new(DataTransformer::new()));
        transformers.insert(AssetType::Cloud, Box::new(CloudTransformer::new()));
        
        Ok(transformers)
    }

    /// Map inventory assets to OSCAL components
    pub async fn map_inventory_to_components(
        &self,
        inventory: &InventoryDocument,
    ) -> Result<InventoryMappingResult> {
        info!("Starting inventory to OSCAL component mapping");
        let start_time = std::time::Instant::now();

        let mut components = Vec::new();
        let mut relationships = Vec::new();
        let mut unmapped_assets = Vec::new();
        let mut validation_results = Vec::new();
        let mut total_confidence = 0.0;

        // Process each asset
        for asset in &inventory.assets {
            match self.map_asset_to_component(asset).await {
                Ok(component_result) => {
                    components.push(component_result.component);
                    validation_results.extend(component_result.validation_results);
                    total_confidence += component_result.confidence;
                }
                Err(e) => {
                    warn!("Failed to map asset {}: {}", asset.asset_id, e);
                    unmapped_assets.push(asset.asset_id.clone());
                    validation_results.push(MappingValidationResult {
                        asset_id: asset.asset_id.clone(),
                        status: ValidationStatus::Failed,
                        message: format!("Mapping failed: {}", e),
                        field: None,
                    });
                }
            }
        }

        // Process relationships
        if !inventory.relationships.is_empty() {
            relationships = self.map_asset_relationships(&inventory.relationships, &components).await?;
        }

        let processing_time = start_time.elapsed().as_millis() as u64;
        let mapping_confidence = if !components.is_empty() {
            total_confidence / components.len() as f64
        } else {
            0.0
        };

        let statistics = MappingStatistics {
            total_assets: inventory.assets.len(),
            mapped_assets: components.len(),
            failed_mappings: unmapped_assets.len(),
            components_generated: components.len(),
            relationships_discovered: relationships.len(),
            processing_time_ms: processing_time,
        };

        info!(
            "Inventory mapping completed: {}/{} assets mapped, {} components generated",
            components.len(),
            inventory.assets.len(),
            components.len()
        );

        Ok(InventoryMappingResult {
            components,
            relationships,
            unmapped_assets,
            mapping_confidence,
            validation_results,
            statistics,
        })
    }

    /// Map a single asset to an OSCAL component
    async fn map_asset_to_component(&self, asset: &Asset) -> Result<ComponentMappingResult> {
        debug!("Mapping asset {} to OSCAL component", asset.asset_id);

        // Get the appropriate transformer for this asset type
        let transformer = self.asset_transformers.get(&asset.asset_type)
            .ok_or_else(|| Error::validation(format!("No transformer found for asset type: {:?}", asset.asset_type)))?;

        // Use the FedRAMP IIW mappings for all asset types for now
        let asset_mapping = &self.mapping_config.fedramp_iiw_mappings;

        // Transform asset to component
        let component = transformer.transform_asset(asset, asset_mapping)?;

        // Validate the mapping
        let validation_results = transformer.validate_asset(asset, asset_mapping)?;

        // Calculate confidence based on validation results
        let confidence = self.calculate_mapping_confidence(&validation_results);

        Ok(ComponentMappingResult {
            component,
            validation_results,
            confidence,
        })
    }

    /// Map asset relationships to component relationships
    async fn map_asset_relationships(
        &self,
        asset_relationships: &[AssetRelationship],
        components: &[OscalComponent],
    ) -> Result<Vec<ComponentRelationship>> {
        debug!("Mapping {} asset relationships to component relationships", asset_relationships.len());

        let mut component_relationships = Vec::new();
        let component_map: HashMap<String, &OscalComponent> = components
            .iter()
            .map(|c| (c.uuid.clone(), c))
            .collect();

        for asset_rel in asset_relationships {
            // Find corresponding components
            if let (Some(source_comp), Some(target_comp)) = (
                component_map.get(&asset_rel.source_asset_id),
                component_map.get(&asset_rel.target_asset_id),
            ) {
                let component_rel = ComponentRelationship {
                    source_uuid: source_comp.uuid.clone(),
                    target_uuid: target_comp.uuid.clone(),
                    relationship_type: asset_rel.relationship_type.to_string(),
                    description: Some(format!(
                        "Relationship between {} and {}",
                        source_comp.title,
                        target_comp.title
                    )),
                };
                component_relationships.push(component_rel);
            }
        }

        debug!("Mapped {} component relationships", component_relationships.len());
        Ok(component_relationships)
    }

    /// Calculate mapping confidence based on validation results
    fn calculate_mapping_confidence(&self, validation_results: &[MappingValidationResult]) -> f64 {
        if validation_results.is_empty() {
            return 1.0;
        }

        let passed_count = validation_results
            .iter()
            .filter(|r| r.status == ValidationStatus::Passed)
            .count();

        passed_count as f64 / validation_results.len() as f64
    }

    /// Get mapping configuration
    pub fn get_mapping_config(&self) -> &InventoryMappingConfig {
        &self.mapping_config
    }

    /// Reload mapping configuration
    pub async fn reload_configuration(&mut self) -> Result<()> {
        info!("Reloading inventory mapping configuration");
        self.mapping_config = self.mapping_loader.load_inventory_mappings().await?;
        Ok(())
    }
}

/// Result of mapping a single asset to a component
#[derive(Debug, Clone)]
struct ComponentMappingResult {
    /// Generated component
    component: OscalComponent,
    /// Validation results
    validation_results: Vec<MappingValidationResult>,
    /// Mapping confidence score
    confidence: f64,
}

// Note: Default implementation removed since new() is now async
