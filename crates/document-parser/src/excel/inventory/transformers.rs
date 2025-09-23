// Modified: 2025-09-23

//! Asset transformers for converting inventory assets to OSCAL components
//! 
//! This module provides specialized transformers for different asset types,
//! handling the conversion from inventory data to OSCAL component format
//! with type-specific logic and validation.

use super::column_mapper::*;
use super::types::*;
use crate::mapping::inventory::InventoryColumnMappings;
use crate::Result;
use fedramp_core::Error;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tracing::{debug, warn};
use uuid::Uuid;

/// Asset transformer trait for type-specific processing
pub trait AssetTransformer: Send + Sync {
    /// Transform asset data to OSCAL component
    fn transform_asset(&self, asset: &Asset, mapping: &InventoryColumnMappings) -> Result<OscalComponent>;

    /// Validate asset data
    fn validate_asset(&self, asset: &Asset, mapping: &InventoryColumnMappings) -> Result<Vec<MappingValidationResult>>;

    /// Get asset type this transformer handles
    fn asset_type(&self) -> AssetType;
}

/// Hardware asset transformer
#[derive(Debug, Clone)]
pub struct HardwareTransformer {
    /// Hardware-specific configuration
    config: HardwareTransformerConfig,
}

/// Software asset transformer
#[derive(Debug, Clone)]
pub struct SoftwareTransformer {
    /// Software-specific configuration
    config: SoftwareTransformerConfig,
}

/// Network asset transformer
#[derive(Debug, Clone)]
pub struct NetworkTransformer {
    /// Network-specific configuration
    config: NetworkTransformerConfig,
}

/// Virtual asset transformer
#[derive(Debug, Clone)]
pub struct VirtualTransformer {
    /// Virtual-specific configuration
    config: VirtualTransformerConfig,
}

/// Data asset transformer
#[derive(Debug, Clone)]
pub struct DataTransformer {
    /// Data-specific configuration
    config: DataTransformerConfig,
}

/// Cloud asset transformer
#[derive(Debug, Clone)]
pub struct CloudTransformer {
    /// Cloud-specific configuration
    config: CloudTransformerConfig,
}

/// Configuration for hardware transformer
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HardwareTransformerConfig {
    /// Default component type for hardware
    pub default_component_type: String,
    /// Hardware-specific property mappings
    pub property_mappings: HashMap<String, String>,
    /// Required fields for hardware assets
    pub required_fields: Vec<String>,
}

/// Configuration for software transformer
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SoftwareTransformerConfig {
    /// Default component type for software
    pub default_component_type: String,
    /// Software-specific property mappings
    pub property_mappings: HashMap<String, String>,
    /// Required fields for software assets
    pub required_fields: Vec<String>,
}

/// Configuration for network transformer
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkTransformerConfig {
    /// Default component type for network
    pub default_component_type: String,
    /// Network-specific property mappings
    pub property_mappings: HashMap<String, String>,
    /// Required fields for network assets
    pub required_fields: Vec<String>,
}

/// Configuration for virtual transformer
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VirtualTransformerConfig {
    /// Default component type for virtual
    pub default_component_type: String,
    /// Virtual-specific property mappings
    pub property_mappings: HashMap<String, String>,
    /// Required fields for virtual assets
    pub required_fields: Vec<String>,
}

/// Configuration for data transformer
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataTransformerConfig {
    /// Default component type for data
    pub default_component_type: String,
    /// Data-specific property mappings
    pub property_mappings: HashMap<String, String>,
    /// Required fields for data assets
    pub required_fields: Vec<String>,
}

/// Configuration for cloud transformer
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CloudTransformerConfig {
    /// Default component type for cloud
    pub default_component_type: String,
    /// Cloud-specific property mappings
    pub property_mappings: HashMap<String, String>,
    /// Required fields for cloud assets
    pub required_fields: Vec<String>,
}

impl HardwareTransformer {
    /// Create a new hardware transformer
    pub fn new() -> Self {
        let config = HardwareTransformerConfig {
            default_component_type: "hardware".to_string(),
            property_mappings: Self::create_hardware_property_mappings(),
            required_fields: vec![
                "id".to_string(),
                "name".to_string(),
                "asset_type".to_string(),
                "environment".to_string(),
            ],
        };

        Self { config }
    }

    /// Create hardware-specific property mappings
    fn create_hardware_property_mappings() -> HashMap<String, String> {
        let mut mappings = HashMap::new();
        mappings.insert("manufacturer".to_string(), "hardware-manufacturer".to_string());
        mappings.insert("model".to_string(), "hardware-model".to_string());
        mappings.insert("serial_number".to_string(), "hardware-serial-number".to_string());
        mappings.insert("location".to_string(), "physical-location".to_string());
        mappings.insert("rack_position".to_string(), "rack-position".to_string());
        mappings.insert("power_consumption".to_string(), "power-consumption".to_string());
        mappings
    }
}

impl AssetTransformer for HardwareTransformer {
    fn transform_asset(&self, asset: &Asset, _mapping: &InventoryColumnMappings) -> Result<OscalComponent> {
        debug!("Transforming hardware asset: {}", asset.asset_id);

        let mut props = HashMap::new();
        
        // Map basic properties
        if let Some(ref hardware_info) = asset.hardware_info {
            if !hardware_info.manufacturer.is_empty() {
                props.insert("hardware-manufacturer".to_string(), hardware_info.manufacturer.clone());
            }
            if !hardware_info.model.is_empty() {
                props.insert("hardware-model".to_string(), hardware_info.model.clone());
            }
            if let Some(ref serial_number) = hardware_info.serial_number {
                props.insert("hardware-serial-number".to_string(), serial_number.clone());
            }
        }

        // Map network information
        if let Some(ref network_info) = asset.network_info {
            if !network_info.ip_addresses.is_empty() {
                let ip_address = network_info.ip_addresses[0].to_string();
                props.insert("ip-address".to_string(), ip_address);
            }
            if !network_info.mac_addresses.is_empty() {
                let mac_address = &network_info.mac_addresses[0];
                props.insert("mac-address".to_string(), mac_address.clone());
            }
        }

        // Map environment and criticality
        props.insert("environment".to_string(), asset.environment.to_string());
        props.insert("criticality".to_string(), asset.criticality.to_string());

        // Create responsible roles
        let mut responsible_roles = HashMap::new();
        if !asset.owner.is_empty() {
            responsible_roles.insert("asset-owner".to_string(), asset.owner.clone());
        }

        let component = OscalComponent {
            uuid: asset.asset_id.clone(),
            title: asset.asset_name.clone(),
            description: Some(asset.description.clone()),
            component_type: self.config.default_component_type.clone(),
            props,
            responsible_roles,
            control_implementations: Vec::new(), // Will be populated by control mapping
        };

        Ok(component)
    }

    fn validate_asset(&self, asset: &Asset, _mapping: &InventoryColumnMappings) -> Result<Vec<MappingValidationResult>> {
        let mut results = Vec::new();

        // Validate required fields
        for field in &self.config.required_fields {
            let validation_result = match field.as_str() {
                "id" => {
                    if asset.asset_id.is_empty() {
                        MappingValidationResult {
                            asset_id: asset.asset_id.clone(),
                            status: ValidationStatus::Failed,
                            message: "Asset ID is required".to_string(),
                            field: Some("id".to_string()),
                        }
                    } else {
                        MappingValidationResult {
                            asset_id: asset.asset_id.clone(),
                            status: ValidationStatus::Passed,
                            message: "Asset ID is valid".to_string(),
                            field: Some("id".to_string()),
                        }
                    }
                }
                "name" => {
                    if asset.asset_name.is_empty() {
                        MappingValidationResult {
                            asset_id: asset.asset_id.clone(),
                            status: ValidationStatus::Failed,
                            message: "Asset name is required".to_string(),
                            field: Some("name".to_string()),
                        }
                    } else {
                        MappingValidationResult {
                            asset_id: asset.asset_id.clone(),
                            status: ValidationStatus::Passed,
                            message: "Asset name is valid".to_string(),
                            field: Some("name".to_string()),
                        }
                    }
                }
                _ => MappingValidationResult {
                    asset_id: asset.asset_id.clone(),
                    status: ValidationStatus::Passed,
                    message: format!("Field {} validated", field),
                    field: Some(field.clone()),
                },
            };
            results.push(validation_result);
        }

        // Validate hardware-specific fields
        if asset.hardware_info.is_none() {
            results.push(MappingValidationResult {
                asset_id: asset.asset_id.clone(),
                status: ValidationStatus::Warning,
                message: "Hardware information is missing".to_string(),
                field: Some("hardware_info".to_string()),
            });
        }

        Ok(results)
    }

    fn asset_type(&self) -> AssetType {
        AssetType::Hardware
    }
}

impl SoftwareTransformer {
    /// Create a new software transformer
    pub fn new() -> Self {
        let config = SoftwareTransformerConfig {
            default_component_type: "software".to_string(),
            property_mappings: Self::create_software_property_mappings(),
            required_fields: vec![
                "id".to_string(),
                "name".to_string(),
                "asset_type".to_string(),
                "environment".to_string(),
            ],
        };

        Self { config }
    }

    /// Create software-specific property mappings
    fn create_software_property_mappings() -> HashMap<String, String> {
        let mut mappings = HashMap::new();
        mappings.insert("vendor".to_string(), "software-vendor".to_string());
        mappings.insert("version".to_string(), "software-version".to_string());
        mappings.insert("license".to_string(), "software-license".to_string());
        mappings.insert("installation_path".to_string(), "installation-path".to_string());
        mappings.insert("configuration_file".to_string(), "configuration-file".to_string());
        mappings
    }
}

impl AssetTransformer for SoftwareTransformer {
    fn transform_asset(&self, asset: &Asset, _mapping: &InventoryColumnMappings) -> Result<OscalComponent> {
        debug!("Transforming software asset: {}", asset.asset_id);

        let mut props = HashMap::new();
        
        // Map software-specific properties
        if let Some(ref software_info) = asset.software_info {
            if !software_info.vendor.is_empty() {
                props.insert("software-vendor".to_string(), software_info.vendor.clone());
            }
            if !software_info.version.is_empty() {
                props.insert("software-version".to_string(), software_info.version.clone());
            }
            if let Some(ref license) = software_info.license {
                props.insert("software-license".to_string(), license.license_type.clone());
            }
        }

        // Map environment and criticality
        props.insert("environment".to_string(), asset.environment.to_string());
        props.insert("criticality".to_string(), asset.criticality.to_string());

        // Create responsible roles
        let mut responsible_roles = HashMap::new();
        if !asset.owner.is_empty() {
            responsible_roles.insert("asset-owner".to_string(), asset.owner.clone());
        }

        let component = OscalComponent {
            uuid: asset.asset_id.clone(),
            title: asset.asset_name.clone(),
            description: Some(asset.description.clone()),
            component_type: self.config.default_component_type.clone(),
            props,
            responsible_roles,
            control_implementations: Vec::new(),
        };

        Ok(component)
    }

    fn validate_asset(&self, asset: &Asset, _mapping: &InventoryColumnMappings) -> Result<Vec<MappingValidationResult>> {
        let mut results = Vec::new();

        // Basic validation similar to hardware transformer
        if asset.asset_id.is_empty() {
            results.push(MappingValidationResult {
                asset_id: asset.asset_id.clone(),
                status: ValidationStatus::Failed,
                message: "Asset ID is required".to_string(),
                field: Some("id".to_string()),
            });
        }

        if asset.asset_name.is_empty() {
            results.push(MappingValidationResult {
                asset_id: asset.asset_id.clone(),
                status: ValidationStatus::Failed,
                message: "Asset name is required".to_string(),
                field: Some("name".to_string()),
            });
        }

        // Validate software-specific fields
        if asset.software_info.is_none() {
            results.push(MappingValidationResult {
                asset_id: asset.asset_id.clone(),
                status: ValidationStatus::Warning,
                message: "Software information is missing".to_string(),
                field: Some("software_info".to_string()),
            });
        }

        Ok(results)
    }

    fn asset_type(&self) -> AssetType {
        AssetType::Software
    }
}

impl NetworkTransformer {
    /// Create a new network transformer
    pub fn new() -> Self {
        let config = NetworkTransformerConfig {
            default_component_type: "hardware".to_string(), // Network devices are typically hardware
            property_mappings: Self::create_network_property_mappings(),
            required_fields: vec![
                "id".to_string(),
                "name".to_string(),
                "asset_type".to_string(),
                "environment".to_string(),
            ],
        };

        Self { config }
    }

    /// Create network-specific property mappings
    fn create_network_property_mappings() -> HashMap<String, String> {
        let mut mappings = HashMap::new();
        mappings.insert("ip_address".to_string(), "ip-address".to_string());
        mappings.insert("mac_address".to_string(), "mac-address".to_string());
        mappings.insert("subnet".to_string(), "network-subnet".to_string());
        mappings.insert("vlan".to_string(), "vlan-id".to_string());
        mappings.insert("ports".to_string(), "network-ports".to_string());
        mappings.insert("protocols".to_string(), "supported-protocols".to_string());
        mappings
    }
}

impl AssetTransformer for NetworkTransformer {
    fn transform_asset(&self, asset: &Asset, _mapping: &InventoryColumnMappings) -> Result<OscalComponent> {
        debug!("Transforming network asset: {}", asset.asset_id);

        let mut props = HashMap::new();

        // Map network-specific properties
        if let Some(ref network_info) = asset.network_info {
            if !network_info.ip_addresses.is_empty() {
                let ip_address = network_info.ip_addresses[0].to_string();
                props.insert("ip-address".to_string(), ip_address);
            }
            if !network_info.mac_addresses.is_empty() {
                let mac_address = &network_info.mac_addresses[0];
                props.insert("mac-address".to_string(), mac_address.clone());
            }
            if !network_info.network_segments.is_empty() {
                let segment = &network_info.network_segments[0];
                props.insert("network-segment".to_string(), segment.clone());
            }
            if !network_info.protocols.is_empty() {
                let protocol = &network_info.protocols[0];
                props.insert("protocol".to_string(), protocol.clone());
            }
        }

        // Map environment and criticality
        props.insert("environment".to_string(), asset.environment.to_string());
        props.insert("criticality".to_string(), asset.criticality.to_string());

        // Create responsible roles
        let mut responsible_roles = HashMap::new();
        if !asset.owner.is_empty() {
            responsible_roles.insert("asset-owner".to_string(), asset.owner.clone());
        }

        let component = OscalComponent {
            uuid: asset.asset_id.clone(),
            title: asset.asset_name.clone(),
            description: Some(asset.description.clone()),
            component_type: self.config.default_component_type.clone(),
            props,
            responsible_roles,
            control_implementations: Vec::new(),
        };

        Ok(component)
    }

    fn validate_asset(&self, asset: &Asset, _mapping: &InventoryColumnMappings) -> Result<Vec<MappingValidationResult>> {
        let mut results = Vec::new();

        // Basic validation
        if asset.asset_id.is_empty() {
            results.push(MappingValidationResult {
                asset_id: asset.asset_id.clone(),
                status: ValidationStatus::Failed,
                message: "Asset ID is required".to_string(),
                field: Some("id".to_string()),
            });
        }

        if asset.asset_name.is_empty() {
            results.push(MappingValidationResult {
                asset_id: asset.asset_id.clone(),
                status: ValidationStatus::Failed,
                message: "Asset name is required".to_string(),
                field: Some("name".to_string()),
            });
        }

        // Validate network-specific fields
        if let Some(ref network_info) = asset.network_info {
            for ip_address in &network_info.ip_addresses {
                if !Self::is_valid_ip_address(&ip_address.to_string()) {
                    results.push(MappingValidationResult {
                        asset_id: asset.asset_id.clone(),
                        status: ValidationStatus::Failed,
                        message: format!("Invalid IP address: {}", ip_address),
                        field: Some("ip_address".to_string()),
                    });
                }
            }

            for mac_address in &network_info.mac_addresses {
                if !Self::is_valid_mac_address(mac_address) {
                    results.push(MappingValidationResult {
                        asset_id: asset.asset_id.clone(),
                        status: ValidationStatus::Failed,
                        message: format!("Invalid MAC address: {}", mac_address),
                        field: Some("mac_address".to_string()),
                    });
                }
            }
        } else {
            results.push(MappingValidationResult {
                asset_id: asset.asset_id.clone(),
                status: ValidationStatus::Warning,
                message: "Network information is missing".to_string(),
                field: Some("network_info".to_string()),
            });
        }

        Ok(results)
    }

    fn asset_type(&self) -> AssetType {
        AssetType::Network
    }
}

impl NetworkTransformer {
    /// Validate IP address format
    fn is_valid_ip_address(ip: &str) -> bool {
        // Simple IPv4 validation
        let parts: Vec<&str> = ip.split('.').collect();
        if parts.len() != 4 {
            return false;
        }

        for part in parts {
            if let Ok(num) = part.parse::<u8>() {
                if num > 255 {
                    return false;
                }
            } else {
                return false;
            }
        }

        true
    }

    /// Validate MAC address format
    fn is_valid_mac_address(mac: &str) -> bool {
        // Simple MAC address validation (XX:XX:XX:XX:XX:XX or XX-XX-XX-XX-XX-XX)
        let cleaned = mac.replace([':', '-'], "");
        cleaned.len() == 12 && cleaned.chars().all(|c| c.is_ascii_hexdigit())
    }
}

// Implement remaining transformers with similar patterns
impl VirtualTransformer {
    pub fn new() -> Self {
        let config = VirtualTransformerConfig {
            default_component_type: "software".to_string(), // Virtual assets are typically software
            property_mappings: HashMap::new(),
            required_fields: vec!["id".to_string(), "name".to_string()],
        };
        Self { config }
    }
}

impl AssetTransformer for VirtualTransformer {
    fn transform_asset(&self, asset: &Asset, _mapping: &InventoryColumnMappings) -> Result<OscalComponent> {
        let mut props = HashMap::new();
        props.insert("environment".to_string(), asset.environment.to_string());
        props.insert("criticality".to_string(), asset.criticality.to_string());
        props.insert("virtual".to_string(), "true".to_string());

        let mut responsible_roles = HashMap::new();
        if !asset.owner.is_empty() {
            responsible_roles.insert("asset-owner".to_string(), asset.owner.clone());
        }

        Ok(OscalComponent {
            uuid: asset.asset_id.clone(),
            title: asset.asset_name.clone(),
            description: Some(asset.description.clone()),
            component_type: self.config.default_component_type.clone(),
            props,
            responsible_roles,
            control_implementations: Vec::new(),
        })
    }

    fn validate_asset(&self, asset: &Asset, _mapping: &InventoryColumnMappings) -> Result<Vec<MappingValidationResult>> {
        let mut results = Vec::new();
        if asset.asset_id.is_empty() {
            results.push(MappingValidationResult {
                asset_id: asset.asset_id.clone(),
                status: ValidationStatus::Failed,
                message: "Asset ID is required".to_string(),
                field: Some("id".to_string()),
            });
        }
        Ok(results)
    }

    fn asset_type(&self) -> AssetType {
        AssetType::Virtual
    }
}

impl DataTransformer {
    pub fn new() -> Self {
        let config = DataTransformerConfig {
            default_component_type: "software".to_string(),
            property_mappings: HashMap::new(),
            required_fields: vec!["id".to_string(), "name".to_string()],
        };
        Self { config }
    }
}

impl AssetTransformer for DataTransformer {
    fn transform_asset(&self, asset: &Asset, _mapping: &InventoryColumnMappings) -> Result<OscalComponent> {
        let mut props = HashMap::new();
        props.insert("environment".to_string(), asset.environment.to_string());
        props.insert("criticality".to_string(), asset.criticality.to_string());
        props.insert("asset-type".to_string(), "data".to_string());

        let mut responsible_roles = HashMap::new();
        if !asset.owner.is_empty() {
            responsible_roles.insert("asset-owner".to_string(), asset.owner.clone());
        }

        Ok(OscalComponent {
            uuid: asset.asset_id.clone(),
            title: asset.asset_name.clone(),
            description: Some(asset.description.clone()),
            component_type: self.config.default_component_type.clone(),
            props,
            responsible_roles,
            control_implementations: Vec::new(),
        })
    }

    fn validate_asset(&self, asset: &Asset, _mapping: &InventoryColumnMappings) -> Result<Vec<MappingValidationResult>> {
        let mut results = Vec::new();
        if asset.asset_id.is_empty() {
            results.push(MappingValidationResult {
                asset_id: asset.asset_id.clone(),
                status: ValidationStatus::Failed,
                message: "Asset ID is required".to_string(),
                field: Some("id".to_string()),
            });
        }
        Ok(results)
    }

    fn asset_type(&self) -> AssetType {
        AssetType::Data
    }
}

impl CloudTransformer {
    pub fn new() -> Self {
        let config = CloudTransformerConfig {
            default_component_type: "service".to_string(),
            property_mappings: HashMap::new(),
            required_fields: vec!["id".to_string(), "name".to_string()],
        };
        Self { config }
    }
}

impl AssetTransformer for CloudTransformer {
    fn transform_asset(&self, asset: &Asset, _mapping: &InventoryColumnMappings) -> Result<OscalComponent> {
        let mut props = HashMap::new();
        props.insert("environment".to_string(), asset.environment.to_string());
        props.insert("criticality".to_string(), asset.criticality.to_string());
        props.insert("deployment-model".to_string(), "cloud".to_string());

        if let Some(ref cloud_info) = asset.cloud_info {
            if !cloud_info.provider.is_empty() {
                props.insert("cloud-provider".to_string(), cloud_info.provider.clone());
            }
            if !cloud_info.region.is_empty() {
                props.insert("cloud-region".to_string(), cloud_info.region.clone());
            }
        }

        let mut responsible_roles = HashMap::new();
        if !asset.owner.is_empty() {
            responsible_roles.insert("asset-owner".to_string(), asset.owner.clone());
        }

        Ok(OscalComponent {
            uuid: asset.asset_id.clone(),
            title: asset.asset_name.clone(),
            description: Some(asset.description.clone()),
            component_type: self.config.default_component_type.clone(),
            props,
            responsible_roles,
            control_implementations: Vec::new(),
        })
    }

    fn validate_asset(&self, asset: &Asset, _mapping: &InventoryColumnMappings) -> Result<Vec<MappingValidationResult>> {
        let mut results = Vec::new();
        if asset.asset_id.is_empty() {
            results.push(MappingValidationResult {
                asset_id: asset.asset_id.clone(),
                status: ValidationStatus::Failed,
                message: "Asset ID is required".to_string(),
                field: Some("id".to_string()),
            });
        }
        Ok(results)
    }

    fn asset_type(&self) -> AssetType {
        AssetType::Cloud
    }
}
