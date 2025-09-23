// Modified: 2025-09-23

//! Asset processing for inventory data
//! 
//! This module handles the conversion of raw Excel data into structured Asset objects,
//! including data enrichment, validation, and categorization.

use super::types::*;
use crate::Result;
use fedramp_core::Error;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::net::IpAddr;
use std::str::FromStr;
use chrono::{DateTime, Utc};
use tracing::{debug, info, warn};
use uuid::Uuid;

/// Asset processor for converting raw data to structured assets
#[derive(Debug, Clone)]
pub struct AssetProcessor {
    /// Configuration for processing behavior
    config: ProcessorConfig,
    /// Asset categorization rules
    categorization_rules: CategorizationRules,
    /// Data enrichment engine
    enrichment_engine: EnrichmentEngine,
    /// Field validators
    field_validators: HashMap<String, FieldValidatorType>,
}

/// Configuration for asset processing
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessorConfig {
    /// Enable automatic asset categorization
    pub enable_categorization: bool,
    /// Enable data enrichment
    pub enable_enrichment: bool,
    /// Enable field validation
    pub enable_validation: bool,
    /// Generate asset IDs if missing
    pub generate_asset_ids: bool,
    /// Default environment for assets
    pub default_environment: Environment,
    /// Default criticality for assets
    pub default_criticality: Criticality,
    /// Custom field mappings
    pub custom_field_mappings: HashMap<String, String>,
}

impl Default for ProcessorConfig {
    fn default() -> Self {
        Self {
            enable_categorization: true,
            enable_enrichment: true,
            enable_validation: true,
            generate_asset_ids: true,
            default_environment: Environment::Production,
            default_criticality: Criticality::Medium,
            custom_field_mappings: HashMap::new(),
        }
    }
}

/// Asset categorization rules
#[derive(Debug, Clone)]
pub struct CategorizationRules {
    /// Type-based categorization rules
    type_rules: HashMap<AssetType, Vec<CategoryRule>>,
    /// Keyword-based categorization
    keyword_rules: Vec<KeywordRule>,
}

/// Rule for categorizing assets based on characteristics
#[derive(Debug, Clone)]
pub struct CategoryRule {
    /// Asset category to assign
    pub category: AssetCategory,
    /// Conditions that must be met
    pub conditions: Vec<Condition>,
    /// Rule priority (higher = more important)
    pub priority: u32,
}

/// Keyword-based categorization rule
#[derive(Debug, Clone)]
pub struct KeywordRule {
    /// Keywords to match against
    pub keywords: Vec<String>,
    /// Asset category to assign
    pub category: AssetCategory,
    /// Fields to search in
    pub search_fields: Vec<String>,
    /// Case sensitive matching
    pub case_sensitive: bool,
}

/// Condition for categorization rules
#[derive(Debug, Clone)]
pub enum Condition {
    /// Field contains specific value
    FieldContains { field: String, value: String },
    /// Field equals specific value
    FieldEquals { field: String, value: String },
    /// Field matches regex pattern
    FieldMatches { field: String, pattern: String },
    /// Field is not empty
    FieldNotEmpty { field: String },
}

/// Data enrichment engine
#[derive(Debug, Clone)]
pub struct EnrichmentEngine {
    /// Network data enrichment
    network_enricher: NetworkEnricher,
    /// Software data enrichment
    software_enricher: SoftwareEnricher,
    /// Hardware data enrichment
    hardware_enricher: HardwareEnricher,
    /// Cloud data enrichment
    cloud_enricher: CloudEnricher,
}

/// Network data enrichment
#[derive(Debug, Clone)]
pub struct NetworkEnricher {
    /// IP address validation and normalization
    pub validate_ip_addresses: bool,
    /// MAC address validation and normalization
    pub validate_mac_addresses: bool,
    /// DNS resolution
    pub resolve_dns_names: bool,
}

/// Software data enrichment
#[derive(Debug, Clone)]
pub struct SoftwareEnricher {
    /// Version normalization
    pub normalize_versions: bool,
    /// Vendor name standardization
    pub standardize_vendors: bool,
    /// License detection
    pub detect_licenses: bool,
}

/// Hardware data enrichment
#[derive(Debug, Clone)]
pub struct HardwareEnricher {
    /// Manufacturer name standardization
    pub standardize_manufacturers: bool,
    /// Model name normalization
    pub normalize_models: bool,
    /// Specification parsing
    pub parse_specifications: bool,
}

/// Cloud data enrichment
#[derive(Debug, Clone)]
pub struct CloudEnricher {
    /// Provider name standardization
    pub standardize_providers: bool,
    /// Region validation
    pub validate_regions: bool,
    /// Resource tag parsing
    pub parse_resource_tags: bool,
}

/// Field validator types
#[derive(Debug, Clone)]
pub enum FieldValidatorType {
    /// IP address validator
    IpAddress,
    /// MAC address validator
    MacAddress,
    /// Email validator
    Email,
    /// URL validator
    Url,
    /// Required field validator
    Required,
}

/// Validation result for a field
#[derive(Debug, Clone)]
pub struct ValidationResult {
    /// Validation passed
    pub is_valid: bool,
    /// Normalized value
    pub normalized_value: Option<String>,
    /// Validation error message
    pub error: Option<String>,
    /// Validation warnings
    pub warnings: Vec<String>,
}

impl AssetProcessor {
    /// Create a new asset processor with default configuration
    pub fn new() -> Self {
        Self::with_config(ProcessorConfig::default())
    }

    /// Create asset processor with custom configuration
    pub fn with_config(config: ProcessorConfig) -> Self {
        let categorization_rules = Self::create_default_categorization_rules();
        let enrichment_engine = EnrichmentEngine::new();
        let field_validators = Self::create_default_validators();

        Self {
            config,
            categorization_rules,
            enrichment_engine,
            field_validators,
        }
    }

    /// Process a single asset row from Excel data
    pub async fn process_asset_row(
        &self,
        row_data: &HashMap<String, String>,
        headers: &[String],
        asset_type: AssetType,
        template_info: &InventoryTemplateInfo,
        row_number: usize,
    ) -> Result<Asset> {
        debug!("Processing asset row {}", row_number);

        // Extract basic asset information
        let mut asset = self.extract_basic_asset_info(row_data, asset_type, row_number)?;

        // Apply field mappings
        self.apply_field_mappings(&mut asset, row_data, template_info)?;

        // Categorize asset if enabled
        if self.config.enable_categorization {
            asset.asset_category = self.categorize_asset(&asset, row_data)?;
        }

        // Enrich asset data if enabled
        if self.config.enable_enrichment {
            self.enrich_asset_data(&mut asset, row_data).await?;
        }

        // Validate asset data if enabled
        if self.config.enable_validation {
            self.validate_asset_data(&asset, row_data)?;
        }

        debug!("Successfully processed asset: {}", asset.asset_id);
        Ok(asset)
    }

    /// Extract basic asset information from row data
    fn extract_basic_asset_info(
        &self,
        row_data: &HashMap<String, String>,
        asset_type: AssetType,
        row_number: usize,
    ) -> Result<Asset> {
        // Extract or generate asset ID
        let asset_id = if let Some(id) = row_data.get("asset_id").or_else(|| row_data.get("Asset ID")) {
            if id.trim().is_empty() && self.config.generate_asset_ids {
                format!("asset_{}", Uuid::new_v4().to_string().replace('-', "")[..8].to_uppercase())
            } else {
                id.trim().to_string()
            }
        } else if self.config.generate_asset_ids {
            format!("asset_{}", Uuid::new_v4().to_string().replace('-', "")[..8].to_uppercase())
        } else {
            return Err(Error::validation(format!("Asset ID missing in row {}", row_number)));
        };

        // Extract asset name
        let asset_name = row_data
            .get("asset_name")
            .or_else(|| row_data.get("Asset Name"))
            .or_else(|| row_data.get("Name"))
            .map(|s| s.trim().to_string())
            .unwrap_or_else(|| format!("Asset {}", asset_id));

        // Extract description
        let description = row_data
            .get("description")
            .or_else(|| row_data.get("Description"))
            .map(|s| s.trim().to_string())
            .unwrap_or_default();

        // Extract owner
        let owner = row_data
            .get("owner")
            .or_else(|| row_data.get("Owner"))
            .or_else(|| row_data.get("Responsible Party"))
            .map(|s| s.trim().to_string())
            .unwrap_or_default();

        // Extract location
        let location = row_data
            .get("location")
            .or_else(|| row_data.get("Location"))
            .or_else(|| row_data.get("Physical Location"))
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty());

        // Extract environment
        let environment = self.parse_environment(
            row_data
                .get("environment")
                .or_else(|| row_data.get("Environment"))
                .map(|s| s.trim())
                .unwrap_or("")
        );

        // Extract criticality
        let criticality = self.parse_criticality(
            row_data
                .get("criticality")
                .or_else(|| row_data.get("Criticality"))
                .or_else(|| row_data.get("Business Impact"))
                .map(|s| s.trim())
                .unwrap_or("")
        );

        // Create basic asset
        let mut asset = Asset::new(asset_id, asset_name, asset_type);
        asset.description = description;
        asset.owner = owner;
        asset.location = location;
        asset.environment = environment;
        asset.criticality = criticality;

        Ok(asset)
    }

    /// Parse environment from string
    fn parse_environment(&self, env_str: &str) -> Environment {
        match env_str.to_lowercase().as_str() {
            "prod" | "production" => Environment::Production,
            "dev" | "development" => Environment::Development,
            "test" | "testing" => Environment::Testing,
            "stage" | "staging" => Environment::Staging,
            "train" | "training" => Environment::Training,
            "dr" | "disaster_recovery" | "disaster recovery" => Environment::DisasterRecovery,
            "sandbox" | "sand" => Environment::Sandbox,
            _ => self.config.default_environment.clone(),
        }
    }

    /// Parse criticality from string
    fn parse_criticality(&self, crit_str: &str) -> Criticality {
        match crit_str.to_lowercase().as_str() {
            "low" | "l" | "1" => Criticality::Low,
            "medium" | "med" | "m" | "2" => Criticality::Medium,
            "high" | "h" | "3" => Criticality::High,
            "critical" | "crit" | "c" | "4" => Criticality::Critical,
            _ => self.config.default_criticality.clone(),
        }
    }

    /// Apply field mappings from template
    fn apply_field_mappings(
        &self,
        asset: &mut Asset,
        row_data: &HashMap<String, String>,
        template_info: &InventoryTemplateInfo,
    ) -> Result<()> {
        // Apply custom field mappings
        for (source_field, target_field) in &self.config.custom_field_mappings {
            if let Some(value) = row_data.get(source_field) {
                asset.custom_attributes.insert(target_field.clone(), value.clone());
            }
        }

        // Extract tags if present
        if let Some(tags_str) = row_data.get("tags").or_else(|| row_data.get("Tags")) {
            asset.tags = tags_str
                .split(',')
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty())
                .collect();
        }

        Ok(())
    }

    /// Categorize asset based on rules
    fn categorize_asset(
        &self,
        asset: &Asset,
        row_data: &HashMap<String, String>,
    ) -> Result<AssetCategory> {
        // Check type-specific rules first
        if let Some(rules) = self.categorization_rules.type_rules.get(&asset.asset_type) {
            for rule in rules {
                if self.evaluate_category_rule(rule, asset, row_data)? {
                    return Ok(rule.category.clone());
                }
            }
        }

        // Check keyword rules
        for rule in &self.categorization_rules.keyword_rules {
            if self.evaluate_keyword_rule(rule, asset, row_data)? {
                return Ok(rule.category.clone());
            }
        }

        // Default categorization based on asset type
        Ok(match asset.asset_type {
            AssetType::Hardware => AssetCategory::Server,
            AssetType::Software => AssetCategory::Application,
            AssetType::Network => AssetCategory::Router,
            AssetType::Virtual => AssetCategory::VirtualMachine,
            AssetType::Data => AssetCategory::DatabaseInstance,
            AssetType::Cloud => AssetCategory::ComputeInstance,
            AssetType::Service => AssetCategory::WebService,
        })
    }

    /// Evaluate category rule conditions
    fn evaluate_category_rule(
        &self,
        rule: &CategoryRule,
        asset: &Asset,
        row_data: &HashMap<String, String>,
    ) -> Result<bool> {
        for condition in &rule.conditions {
            if !self.evaluate_condition(condition, asset, row_data)? {
                return Ok(false);
            }
        }
        Ok(true)
    }

    /// Evaluate keyword rule
    fn evaluate_keyword_rule(
        &self,
        rule: &KeywordRule,
        asset: &Asset,
        row_data: &HashMap<String, String>,
    ) -> Result<bool> {
        for field in &rule.search_fields {
            let field_value = match field.as_str() {
                "asset_name" => &asset.asset_name,
                "description" => &asset.description,
                _ => row_data.get(field).map(|s| s.as_str()).unwrap_or(""),
            };

            let search_value = if rule.case_sensitive {
                field_value.to_string()
            } else {
                field_value.to_lowercase()
            };

            for keyword in &rule.keywords {
                let search_keyword = if rule.case_sensitive {
                    keyword.clone()
                } else {
                    keyword.to_lowercase()
                };

                if search_value.contains(&search_keyword) {
                    return Ok(true);
                }
            }
        }
        Ok(false)
    }

    /// Evaluate a single condition
    fn evaluate_condition(
        &self,
        condition: &Condition,
        asset: &Asset,
        row_data: &HashMap<String, String>,
    ) -> Result<bool> {
        match condition {
            Condition::FieldContains { field, value } => {
                let field_value = self.get_field_value(field, asset, row_data);
                Ok(field_value.to_lowercase().contains(&value.to_lowercase()))
            }
            Condition::FieldEquals { field, value } => {
                let field_value = self.get_field_value(field, asset, row_data);
                Ok(field_value.to_lowercase() == value.to_lowercase())
            }
            Condition::FieldMatches { field, pattern } => {
                let field_value = self.get_field_value(field, asset, row_data);
                // Simple pattern matching (could be enhanced with regex)
                Ok(field_value.to_lowercase().contains(&pattern.to_lowercase()))
            }
            Condition::FieldNotEmpty { field } => {
                let field_value = self.get_field_value(field, asset, row_data);
                Ok(!field_value.trim().is_empty())
            }
        }
    }

    /// Get field value from asset or row data
    fn get_field_value(&self, field: &str, asset: &Asset, row_data: &HashMap<String, String>) -> String {
        match field {
            "asset_name" => asset.asset_name.clone(),
            "description" => asset.description.clone(),
            "owner" => asset.owner.clone(),
            "asset_type" => asset.asset_type_string().to_string(),
            _ => row_data.get(field).cloned().unwrap_or_default(),
        }
    }

    /// Enrich asset data with additional information
    async fn enrich_asset_data(
        &self,
        asset: &mut Asset,
        row_data: &HashMap<String, String>,
    ) -> Result<()> {
        // Enrich network information
        if let Some(network_info) = self.enrich_network_data(row_data).await? {
            asset.network_info = Some(network_info);
        }

        // Enrich software information
        if asset.asset_type == AssetType::Software {
            if let Some(software_info) = self.enrich_software_data(row_data).await? {
                asset.software_info = Some(software_info);
            }
        }

        // Enrich hardware information
        if asset.asset_type == AssetType::Hardware {
            if let Some(hardware_info) = self.enrich_hardware_data(row_data).await? {
                asset.hardware_info = Some(hardware_info);
            }
        }

        // Enrich cloud information
        if asset.asset_type == AssetType::Cloud {
            if let Some(cloud_info) = self.enrich_cloud_data(row_data).await? {
                asset.cloud_info = Some(cloud_info);
            }
        }

        Ok(())
    }

    /// Enrich network data
    async fn enrich_network_data(&self, row_data: &HashMap<String, String>) -> Result<Option<NetworkInfo>> {
        let mut network_info = NetworkInfo {
            ip_addresses: Vec::new(),
            mac_addresses: Vec::new(),
            network_segments: Vec::new(),
            ports: Vec::new(),
            protocols: Vec::new(),
            dns_names: Vec::new(),
            interfaces: Vec::new(),
        };

        let mut has_network_data = false;

        // Parse IP addresses
        if let Some(ip_str) = row_data.get("ip_address").or_else(|| row_data.get("IP Address")) {
            for ip_part in ip_str.split(',') {
                if let Ok(ip) = IpAddr::from_str(ip_part.trim()) {
                    network_info.ip_addresses.push(ip);
                    has_network_data = true;
                }
            }
        }

        // Parse MAC addresses
        if let Some(mac_str) = row_data.get("mac_address").or_else(|| row_data.get("MAC Address")) {
            for mac_part in mac_str.split(',') {
                let mac = mac_part.trim().to_string();
                if !mac.is_empty() {
                    network_info.mac_addresses.push(mac);
                    has_network_data = true;
                }
            }
        }

        // Parse network segments
        if let Some(segment_str) = row_data.get("network_segment").or_else(|| row_data.get("VLAN")) {
            for segment_part in segment_str.split(',') {
                let segment = segment_part.trim().to_string();
                if !segment.is_empty() {
                    network_info.network_segments.push(segment);
                    has_network_data = true;
                }
            }
        }

        // Parse DNS names
        if let Some(dns_str) = row_data.get("dns_name").or_else(|| row_data.get("Hostname")) {
            for dns_part in dns_str.split(',') {
                let dns = dns_part.trim().to_string();
                if !dns.is_empty() {
                    network_info.dns_names.push(dns);
                    has_network_data = true;
                }
            }
        }

        Ok(if has_network_data {
            Some(network_info)
        } else {
            None
        })
    }

    /// Enrich software data
    async fn enrich_software_data(&self, row_data: &HashMap<String, String>) -> Result<Option<SoftwareInfo>> {
        let vendor = row_data
            .get("vendor")
            .or_else(|| row_data.get("Vendor"))
            .or_else(|| row_data.get("Manufacturer"))
            .map(|s| s.trim().to_string())
            .unwrap_or_default();

        let version = row_data
            .get("version")
            .or_else(|| row_data.get("Version"))
            .map(|s| s.trim().to_string())
            .unwrap_or_default();

        if vendor.is_empty() && version.is_empty() {
            return Ok(None);
        }

        let software_info = SoftwareInfo {
            vendor,
            version,
            license: None, // Could be enriched from license data
            installation_path: row_data.get("installation_path").cloned(),
            configuration: HashMap::new(),
            dependencies: Vec::new(),
            patch_level: row_data.get("patch_level").cloned(),
            support_status: SupportStatus::Unknown,
        };

        Ok(Some(software_info))
    }

    /// Enrich hardware data
    async fn enrich_hardware_data(&self, row_data: &HashMap<String, String>) -> Result<Option<HardwareInfo>> {
        let manufacturer = row_data
            .get("manufacturer")
            .or_else(|| row_data.get("Manufacturer"))
            .or_else(|| row_data.get("Vendor"))
            .map(|s| s.trim().to_string())
            .unwrap_or_default();

        let model = row_data
            .get("model")
            .or_else(|| row_data.get("Model"))
            .map(|s| s.trim().to_string())
            .unwrap_or_default();

        if manufacturer.is_empty() && model.is_empty() {
            return Ok(None);
        }

        let hardware_info = HardwareInfo {
            manufacturer,
            model,
            serial_number: row_data.get("serial_number").or_else(|| row_data.get("Serial Number")).cloned(),
            asset_tag: row_data.get("asset_tag").or_else(|| row_data.get("Asset Tag")).cloned(),
            specifications: HashMap::new(),
            warranty: None,
            physical_location: None,
        };

        Ok(Some(hardware_info))
    }

    /// Enrich cloud data
    async fn enrich_cloud_data(&self, row_data: &HashMap<String, String>) -> Result<Option<CloudInfo>> {
        let provider = row_data
            .get("cloud_provider")
            .or_else(|| row_data.get("Provider"))
            .map(|s| s.trim().to_string())
            .unwrap_or_default();

        let region = row_data
            .get("region")
            .or_else(|| row_data.get("Region"))
            .map(|s| s.trim().to_string())
            .unwrap_or_default();

        let resource_id = row_data
            .get("resource_id")
            .or_else(|| row_data.get("Instance ID"))
            .map(|s| s.trim().to_string())
            .unwrap_or_default();

        if provider.is_empty() && region.is_empty() && resource_id.is_empty() {
            return Ok(None);
        }

        let cloud_info = CloudInfo {
            provider,
            region,
            availability_zone: row_data.get("availability_zone").cloned(),
            instance_type: row_data.get("instance_type").cloned(),
            resource_id,
            resource_tags: HashMap::new(),
            billing_info: None,
        };

        Ok(Some(cloud_info))
    }

    /// Validate asset data
    fn validate_asset_data(&self, asset: &Asset, row_data: &HashMap<String, String>) -> Result<()> {
        // Basic validation
        if asset.asset_id.trim().is_empty() {
            return Err(Error::validation("Asset ID cannot be empty"));
        }

        if asset.asset_name.trim().is_empty() {
            return Err(Error::validation("Asset name cannot be empty"));
        }

        // Validate network information if present
        if let Some(network_info) = &asset.network_info {
            for ip in &network_info.ip_addresses {
                // IP validation is already done during parsing
                debug!("Validated IP address: {}", ip);
            }
        }

        Ok(())
    }

    /// Create default categorization rules
    fn create_default_categorization_rules() -> CategorizationRules {
        let mut type_rules = HashMap::new();
        
        // Hardware categorization rules
        type_rules.insert(AssetType::Hardware, vec![
            CategoryRule {
                category: AssetCategory::Server,
                conditions: vec![
                    Condition::FieldContains { field: "asset_name".to_string(), value: "server".to_string() },
                ],
                priority: 10,
            },
            CategoryRule {
                category: AssetCategory::Workstation,
                conditions: vec![
                    Condition::FieldContains { field: "asset_name".to_string(), value: "workstation".to_string() },
                ],
                priority: 10,
            },
        ]);

        // Software categorization rules
        type_rules.insert(AssetType::Software, vec![
            CategoryRule {
                category: AssetCategory::Database,
                conditions: vec![
                    Condition::FieldContains { field: "asset_name".to_string(), value: "database".to_string() },
                ],
                priority: 10,
            },
            CategoryRule {
                category: AssetCategory::OperatingSystem,
                conditions: vec![
                    Condition::FieldContains { field: "asset_name".to_string(), value: "windows".to_string() },
                ],
                priority: 10,
            },
        ]);

        let keyword_rules = vec![
            KeywordRule {
                keywords: vec!["firewall".to_string(), "security".to_string()],
                category: AssetCategory::SecurityDevice,
                search_fields: vec!["asset_name".to_string(), "description".to_string()],
                case_sensitive: false,
            },
            KeywordRule {
                keywords: vec!["router".to_string(), "switch".to_string()],
                category: AssetCategory::NetworkDevice,
                search_fields: vec!["asset_name".to_string(), "description".to_string()],
                case_sensitive: false,
            },
        ];

        CategorizationRules {
            type_rules,
            keyword_rules,
        }
    }

    /// Create default field validators
    fn create_default_validators() -> HashMap<String, FieldValidatorType> {
        let mut validators = HashMap::new();
        validators.insert("ip_address".to_string(), FieldValidatorType::IpAddress);
        validators.insert("mac_address".to_string(), FieldValidatorType::MacAddress);
        validators.insert("email".to_string(), FieldValidatorType::Email);
        validators.insert("asset_id".to_string(), FieldValidatorType::Required);
        validators.insert("asset_name".to_string(), FieldValidatorType::Required);
        validators
    }

    /// Get processor configuration
    pub fn get_config(&self) -> &ProcessorConfig {
        &self.config
    }

    /// Update processor configuration
    pub fn update_config(&mut self, config: ProcessorConfig) {
        self.config = config;
    }
}

impl EnrichmentEngine {
    /// Create new enrichment engine
    pub fn new() -> Self {
        Self {
            network_enricher: NetworkEnricher {
                validate_ip_addresses: true,
                validate_mac_addresses: true,
                resolve_dns_names: false,
            },
            software_enricher: SoftwareEnricher {
                normalize_versions: true,
                standardize_vendors: true,
                detect_licenses: true,
            },
            hardware_enricher: HardwareEnricher {
                standardize_manufacturers: true,
                normalize_models: true,
                parse_specifications: true,
            },
            cloud_enricher: CloudEnricher {
                standardize_providers: true,
                validate_regions: true,
                parse_resource_tags: true,
            },
        }
    }
}

impl Default for AssetProcessor {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_processor_creation() {
        let processor = AssetProcessor::new();
        assert!(processor.config.enable_categorization);
    }

    #[test]
    fn test_environment_parsing() {
        let processor = AssetProcessor::new();
        
        assert_eq!(processor.parse_environment("production"), Environment::Production);
        assert_eq!(processor.parse_environment("dev"), Environment::Development);
        assert_eq!(processor.parse_environment("test"), Environment::Testing);
        assert_eq!(processor.parse_environment("unknown"), Environment::Production); // default
    }

    #[test]
    fn test_criticality_parsing() {
        let processor = AssetProcessor::new();
        
        assert_eq!(processor.parse_criticality("low"), Criticality::Low);
        assert_eq!(processor.parse_criticality("high"), Criticality::High);
        assert_eq!(processor.parse_criticality("critical"), Criticality::Critical);
        assert_eq!(processor.parse_criticality("unknown"), Criticality::Medium); // default
    }
}
