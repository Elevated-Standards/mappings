// Modified: 2025-01-22

//! Asset validation module for comprehensive asset validation
//!
//! This module provides validation for asset types, environments, attributes,
//! and relationships to ensure inventory data consistency and compliance.

use std::collections::{HashMap, HashSet};
use std::net::IpAddr;
use std::str::FromStr;

use fedramp_core::{Result, Error};
use serde::{Deserialize, Serialize};
use tracing::{debug, info, warn};

use super::types::{
    Asset, AssetType, Environment, Criticality, AssetRelationship, RelationshipType,
    NetworkInfo, HardwareInfo, SoftwareInfo, CloudInfo, AssetValidationResult,
    ComplianceStatusType, ValidationError, ValidationWarning, ValidationSuggestion
};

use crate::validation::types::{ValidationSeverity, ValidationStatus};

/// Comprehensive asset validator
#[derive(Debug, Clone)]
pub struct AssetValidator {
    /// Asset type validator
    type_validator: AssetTypeValidator,
    /// Environment validator
    environment_validator: EnvironmentValidator,
    /// Attribute validator
    attribute_validator: AttributeValidator,
    /// Relationship validator
    relationship_validator: RelationshipValidator,
    /// Validation configuration
    validation_config: AssetValidationConfig,
}

/// Asset validation configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssetValidationConfig {
    /// Allowed asset types
    pub allowed_asset_types: Vec<AssetType>,
    /// Allowed environments
    pub allowed_environments: Vec<Environment>,
    /// Allowed criticality levels
    pub criticality_levels: Vec<Criticality>,
    /// Asset validation rules
    pub validation_rules: Vec<AssetValidationRule>,
    /// Custom enumerations for validation
    pub custom_enumerations: HashMap<String, Vec<String>>,
    /// Strict validation mode
    pub strict_mode: bool,
    /// Maximum validation errors before stopping
    pub max_errors: Option<usize>,
}



/// Asset validation rule
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssetValidationRule {
    /// Rule name
    pub name: String,
    /// Rule description
    pub description: String,
    /// Asset types this rule applies to
    pub asset_types: Vec<AssetType>,
    /// Environments this rule applies to
    pub environments: Vec<Environment>,
    /// Rule condition
    pub condition: RuleCondition,
    /// Rule validation logic
    pub validation: RuleValidation,
    /// Rule severity
    pub severity: ValidationSeverity,
    /// Rule priority (higher = more important)
    pub priority: u32,
}

/// Rule condition for when to apply validation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RuleCondition {
    /// Always apply the rule
    Always,
    /// Apply if asset type matches
    AssetTypeEquals(AssetType),
    /// Apply if environment matches
    EnvironmentEquals(Environment),
    /// Apply if criticality matches
    CriticalityEquals(Criticality),
    /// Apply if field has specific value
    FieldEquals { field: String, value: String },
    /// Apply if field is not empty
    FieldNotEmpty(String),
    /// Apply if multiple conditions are met (AND)
    And(Vec<RuleCondition>),
    /// Apply if any condition is met (OR)
    Or(Vec<RuleCondition>),
    /// Apply if condition is not met (NOT)
    Not(Box<RuleCondition>),
}

/// Rule validation logic
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RuleValidation {
    /// Field must be present and not empty
    RequiredField(String),
    /// Field must match enumeration values
    FieldEnumeration { field: String, allowed_values: Vec<String> },
    /// Field must match pattern
    FieldPattern { field: String, pattern: String },
    /// Field must be within range
    FieldRange { field: String, min: Option<f64>, max: Option<f64> },
    /// Cross-field validation
    CrossField { fields: Vec<String>, rule: String },
    /// Custom validation function
    Custom(String),
}







/// Asset validation metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssetValidationMetrics {
    /// Total validation time in milliseconds
    pub validation_time_ms: u64,
    /// Number of rules evaluated
    pub rules_evaluated: usize,
    /// Number of errors found
    pub error_count: usize,
    /// Number of warnings found
    pub warning_count: usize,
    /// Number of suggestions generated
    pub suggestion_count: usize,
}

/// Asset type validator
#[derive(Debug, Clone)]
pub struct AssetTypeValidator {
    /// Allowed asset types
    allowed_types: HashSet<AssetType>,
    /// Asset type aliases for flexible matching
    type_aliases: HashMap<String, AssetType>,
}

/// Environment validator
#[derive(Debug, Clone)]
pub struct EnvironmentValidator {
    /// Allowed environments
    allowed_environments: HashSet<Environment>,
    /// Environment aliases for flexible matching
    environment_aliases: HashMap<String, Environment>,
}

/// Attribute validator
#[derive(Debug, Clone)]
pub struct AttributeValidator {
    /// Allowed criticality levels
    allowed_criticality: HashSet<Criticality>,
    /// Custom attribute validators
    custom_validators: HashMap<String, AttributeValidatorFn>,
}

/// Relationship validator
#[derive(Debug, Clone)]
pub struct RelationshipValidator {
    /// Allowed relationship types
    allowed_relationship_types: HashSet<RelationshipType>,
    /// Relationship validation rules
    relationship_rules: Vec<RelationshipValidationRule>,
}

/// Attribute validator function type
pub type AttributeValidatorFn = fn(&str) -> Result<bool>;

/// Relationship validation rule
#[derive(Debug, Clone)]
pub struct RelationshipValidationRule {
    /// Rule name
    pub name: String,
    /// Source asset types
    pub source_types: Vec<AssetType>,
    /// Target asset types
    pub target_types: Vec<AssetType>,
    /// Allowed relationship types
    pub allowed_relationships: Vec<RelationshipType>,
    /// Rule description
    pub description: String,
}

impl Default for AssetValidationConfig {
    fn default() -> Self {
        Self {
            allowed_asset_types: vec![
                AssetType::Hardware,
                AssetType::Software,
                AssetType::Network,
                AssetType::Virtual,
                AssetType::Data,
                AssetType::Service,
                AssetType::Cloud,
            ],
            allowed_environments: vec![
                Environment::Production,
                Environment::Development,
                Environment::Testing,
                Environment::Staging,
                Environment::Training,
                Environment::DisasterRecovery,
                Environment::Sandbox,
            ],
            criticality_levels: vec![
                Criticality::Low,
                Criticality::Medium,
                Criticality::High,
                Criticality::Critical,
            ],
            validation_rules: Vec::new(),
            custom_enumerations: HashMap::new(),
            strict_mode: false,
            max_errors: Some(100),
        }
    }
}

impl AssetValidator {
    /// Create a new asset validator with default configuration
    pub fn new() -> Self {
        Self::with_config(AssetValidationConfig::default())
    }

    /// Create a new asset validator with custom configuration
    pub fn with_config(config: AssetValidationConfig) -> Self {
        let type_validator = AssetTypeValidator::new(&config.allowed_asset_types);
        let environment_validator = EnvironmentValidator::new(&config.allowed_environments);
        let attribute_validator = AttributeValidator::new(&config.criticality_levels);
        let relationship_validator = RelationshipValidator::new();

        Self {
            type_validator,
            environment_validator,
            attribute_validator,
            relationship_validator,
            validation_config: config,
        }
    }

    /// Validate a single asset
    pub async fn validate_asset(&self, asset: &Asset) -> Result<AssetValidationResult> {
        let start_time = std::time::Instant::now();

        debug!("Validating asset: {}", asset.asset_id);

        let mut result = AssetValidationResult {
            is_valid: true,
            asset_id: asset.asset_id.clone(),
            field_results: HashMap::new(),
            errors: Vec::new(),
            warnings: Vec::new(),
            suggestions: Vec::new(),
        };

        // Validate asset type
        self.validate_asset_type(asset, &mut result)?;

        // Validate environment
        self.validate_environment(asset, &mut result)?;

        // Validate attributes
        self.validate_attributes(asset, &mut result)?;

        // Apply custom validation rules
        self.apply_validation_rules(asset, &mut result)?;

        // Update validation status
        result.is_valid = result.errors.is_empty();

        let validation_time = start_time.elapsed();
        debug!("Asset validation completed for {} in {}ms",
               asset.asset_id, validation_time.as_millis());

        Ok(result)
    }

    /// Validate multiple assets
    pub async fn validate_assets(&self, assets: &[Asset]) -> Result<Vec<AssetValidationResult>> {
        let mut results = Vec::new();
        let mut error_count = 0;

        info!("Validating {} assets", assets.len());

        for asset in assets {
            let result = self.validate_asset(asset).await?;

            error_count += result.errors.len();
            if let Some(max_errors) = self.validation_config.max_errors {
                if error_count >= max_errors {
                    warn!("Maximum error count reached: {}", max_errors);
                    break;
                }
            }

            results.push(result);
        }

        info!("Asset validation completed. {} assets processed, {} total errors",
              results.len(), error_count);

        Ok(results)
    }

    /// Validate asset relationships
    pub async fn validate_relationships(
        &self,
        relationships: &[AssetRelationship],
        assets: &[Asset],
    ) -> Result<Vec<AssetValidationResult>> {
        let mut results = Vec::new();

        debug!("Validating {} asset relationships", relationships.len());

        for relationship in relationships {
            let result = self.relationship_validator
                .validate_relationship(relationship, assets)
                .await?;
            results.push(result);
        }

        Ok(results)
    }

    /// Validate asset type
    fn validate_asset_type(&self, asset: &Asset, result: &mut AssetValidationResult) -> Result<()> {
        if !self.type_validator.is_valid_type(&asset.asset_type) {
            result.errors.push(ValidationError {
                code: "INVALID_ASSET_TYPE".to_string(),
                message: format!("Invalid asset type: {:?}", asset.asset_type),
                field: Some("asset_type".to_string()),
                asset_id: Some(asset.asset_id.clone()),
                row: None,
            });

            // Suggest valid asset types
            if let Some(suggestion) = self.type_validator.suggest_asset_type(&asset.asset_type) {
                result.suggestions.push(ValidationSuggestion {
                    suggestion_type: "asset_type_correction".to_string(),
                    message: format!("Did you mean '{:?}'?", suggestion),
                    field: Some("asset_type".to_string()),
                    suggested_value: Some(format!("{:?}", suggestion)),
                    asset_id: Some(asset.asset_id.clone()),
                    row: None,
                });
            }
        }

        Ok(())
    }

    /// Validate environment
    fn validate_environment(&self, asset: &Asset, result: &mut AssetValidationResult) -> Result<()> {
        if !self.environment_validator.is_valid_environment(&asset.environment) {
            result.errors.push(ValidationError {
                code: "INVALID_ENVIRONMENT".to_string(),
                message: format!("Invalid environment: {:?}", asset.environment),
                field: Some("environment".to_string()),
                asset_id: Some(asset.asset_id.clone()),
                row: None,
            });

            // Suggest valid environments
            if let Some(suggestion) = self.environment_validator.suggest_environment(&asset.environment) {
                result.suggestions.push(ValidationSuggestion {
                    suggestion_type: "environment_correction".to_string(),
                    message: format!("Did you mean '{:?}'?", suggestion),
                    field: Some("environment".to_string()),
                    suggested_value: Some(format!("{:?}", suggestion)),
                    asset_id: Some(asset.asset_id.clone()),
                    row: None,
                });
            }
        }

        Ok(())
    }

    /// Validate asset attributes
    fn validate_attributes(&self, asset: &Asset, result: &mut AssetValidationResult) -> Result<()> {
        // Validate criticality
        if !self.attribute_validator.is_valid_criticality(&asset.criticality) {
            result.errors.push(ValidationError {
                code: "INVALID_CRITICALITY".to_string(),
                message: format!("Invalid criticality level: {:?}", asset.criticality),
                field: Some("criticality".to_string()),
                asset_id: Some(asset.asset_id.clone()),
                row: None,
            });
        }

        // Validate required fields based on asset type
        self.validate_required_fields(asset, result)?;

        // Validate type-specific attributes
        self.validate_type_specific_attributes(asset, result)?;

        // Validate network information if present
        if let Some(network_info) = &asset.network_info {
            self.validate_network_info(network_info, asset, result)?;
        }

        Ok(())
    }

    /// Validate required fields based on asset type
    fn validate_required_fields(&self, asset: &Asset, result: &mut AssetValidationResult) -> Result<()> {
        // Basic required fields for all assets
        if asset.asset_name.trim().is_empty() {
            result.errors.push(ValidationError {
                code: "MISSING_ASSET_NAME".to_string(),
                message: "Asset name is required".to_string(),
                field: Some("asset_name".to_string()),
                asset_id: Some(asset.asset_id.clone()),
                row: None,
            });
        }

        if asset.owner.trim().is_empty() && asset.criticality == Criticality::Critical {
            result.warnings.push(ValidationWarning {
                code: "CRITICAL_ASSET_NO_OWNER".to_string(),
                message: "Critical assets should have an owner assigned".to_string(),
                field: Some("owner".to_string()),
                asset_id: Some(asset.asset_id.clone()),
                row: None,
            });
        }

        // Asset type-specific required fields
        match asset.asset_type {
            AssetType::Hardware => {
                if asset.hardware_info.is_none() {
                    result.warnings.push(ValidationWarning {
                        code: "MISSING_HARDWARE_INFO".to_string(),
                        message: "Hardware assets should have hardware information".to_string(),
                        field: Some("hardware_info".to_string()),
                        asset_id: Some(asset.asset_id.clone()),
                        row: None,
                    });
                }
            }
            AssetType::Software => {
                if asset.software_info.is_none() {
                    result.warnings.push(ValidationWarning {
                        code: "MISSING_SOFTWARE_INFO".to_string(),
                        message: "Software assets should have software information".to_string(),
                        field: Some("software_info".to_string()),
                        asset_id: Some(asset.asset_id.clone()),
                        row: None,
                    });
                }
            }
            AssetType::Network => {
                if asset.network_info.is_none() {
                    result.errors.push(ValidationError {
                        code: "MISSING_NETWORK_INFO".to_string(),
                        message: "Network assets must have network information".to_string(),
                        field: Some("network_info".to_string()),
                        asset_id: Some(asset.asset_id.clone()),
                        row: None,
                    });
                }
            }
            AssetType::Cloud => {
                if asset.cloud_info.is_none() {
                    result.warnings.push(ValidationWarning {
                        code: "MISSING_CLOUD_INFO".to_string(),
                        message: "Cloud assets should have cloud information".to_string(),
                        field: Some("cloud_info".to_string()),
                        asset_id: Some(asset.asset_id.clone()),
                        row: None,
                    });
                }
            }
            _ => {} // Other asset types don't have specific requirements
        }

        Ok(())
    }

    /// Validate type-specific attributes
    fn validate_type_specific_attributes(&self, asset: &Asset, result: &mut AssetValidationResult) -> Result<()> {
        match asset.asset_type {
            AssetType::Hardware => {
                if let Some(hardware_info) = &asset.hardware_info {
                    self.validate_hardware_info(hardware_info, asset, result)?;
                }
            }
            AssetType::Software => {
                if let Some(software_info) = &asset.software_info {
                    self.validate_software_info(software_info, asset, result)?;
                }
            }
            AssetType::Virtual => {
                // Virtual assets can use hardware_info for VM specifications
                if let Some(hardware_info) = &asset.hardware_info {
                    self.validate_hardware_info(hardware_info, asset, result)?;
                }
            }
            AssetType::Data => {
                // Data assets can use software_info for database/storage specifications
                if let Some(software_info) = &asset.software_info {
                    self.validate_software_info(software_info, asset, result)?;
                }
            }
            AssetType::Cloud => {
                if let Some(cloud_info) = &asset.cloud_info {
                    self.validate_cloud_info(cloud_info, asset, result)?;
                }
            }
            _ => {} // Other types don't have specific validation
        }

        Ok(())
    }

    /// Validate network information
    fn validate_network_info(&self, network_info: &NetworkInfo, asset: &Asset, result: &mut AssetValidationResult) -> Result<()> {
        // IP addresses are already validated as IpAddr types
        // Validate that we have at least one IP address for network assets
        if network_info.ip_addresses.is_empty() {
            result.warnings.push(ValidationWarning {
                code: "NO_IP_ADDRESSES".to_string(),
                message: "Network asset has no IP addresses assigned".to_string(),
                asset_id: Some(asset.asset_id.clone()),
                field: Some("ip_addresses".to_string()),
                row: None,
            });
        }

        // Validate MAC addresses (basic format check)
        for mac_str in &network_info.mac_addresses {
            if !self.is_valid_mac_address(mac_str) {
                result.errors.push(ValidationError {
                    code: "INVALID_MAC_ADDRESS".to_string(),
                    message: format!("Invalid MAC address: {}", mac_str),
                    asset_id: Some(asset.asset_id.clone()),
                    field: Some("mac_addresses".to_string()),
                    row: None,
                });
            }
        }

        Ok(())
    }

    /// Validate hardware information
    fn validate_hardware_info(&self, hardware_info: &HardwareInfo, asset: &Asset, result: &mut AssetValidationResult) -> Result<()> {
        if hardware_info.manufacturer.trim().is_empty() {
            result.warnings.push(ValidationWarning {
                code: "MISSING_MANUFACTURER".to_string(),
                message: "Hardware manufacturer should be specified".to_string(),
                field: Some("manufacturer".to_string()),
                asset_id: Some(asset.asset_id.clone()),
                row: None,
            });
        }

        if hardware_info.model.trim().is_empty() {
            result.warnings.push(ValidationWarning {
                code: "MISSING_MODEL".to_string(),
                message: "Hardware model should be specified".to_string(),
                field: Some("model".to_string()),
                asset_id: Some(asset.asset_id.clone()),
                row: None,
            });
        }

        Ok(())
    }

    /// Validate software information
    fn validate_software_info(&self, software_info: &SoftwareInfo, asset: &Asset, result: &mut AssetValidationResult) -> Result<()> {
        if software_info.vendor.trim().is_empty() {
            result.warnings.push(ValidationWarning {
                code: "MISSING_VENDOR".to_string(),
                message: "Software vendor should be specified".to_string(),
                field: Some("vendor".to_string()),
                asset_id: Some(asset.asset_id.clone()),
                row: None,
            });
        }

        if software_info.version.trim().is_empty() {
            result.warnings.push(ValidationWarning {
                code: "MISSING_VERSION".to_string(),
                message: "Software version should be specified".to_string(),
                field: Some("version".to_string()),
                asset_id: Some(asset.asset_id.clone()),
                row: None,
            });
        }

        Ok(())
    }



    /// Validate cloud information
    fn validate_cloud_info(&self, cloud_info: &CloudInfo, asset: &Asset, result: &mut AssetValidationResult) -> Result<()> {
        if cloud_info.provider.trim().is_empty() {
            result.warnings.push(ValidationWarning {
                code: "MISSING_CLOUD_PROVIDER".to_string(),
                message: "Cloud provider should be specified".to_string(),
                field: Some("provider".to_string()),
                asset_id: Some(asset.asset_id.clone()),
                row: None,
            });
        }

        Ok(())
    }

    /// Apply custom validation rules
    fn apply_validation_rules(&self, asset: &Asset, result: &mut AssetValidationResult) -> Result<()> {
        for rule in &self.validation_config.validation_rules {
            if self.should_apply_rule(rule, asset)? {
                self.apply_single_rule(rule, asset, result)?;
                // Rule applied successfully
            }
        }

        Ok(())
    }

    /// Check if a rule should be applied to an asset
    fn should_apply_rule(&self, rule: &AssetValidationRule, asset: &Asset) -> Result<bool> {
        self.evaluate_rule_condition(&rule.condition, asset)
    }

    /// Evaluate a rule condition
    fn evaluate_rule_condition(&self, condition: &RuleCondition, asset: &Asset) -> Result<bool> {
        match condition {
            RuleCondition::Always => Ok(true),
            RuleCondition::AssetTypeEquals(asset_type) => Ok(asset.asset_type == *asset_type),
            RuleCondition::EnvironmentEquals(environment) => Ok(asset.environment == *environment),
            RuleCondition::CriticalityEquals(criticality) => Ok(asset.criticality == *criticality),
            RuleCondition::FieldEquals { field, value } => {
                let field_value = self.get_asset_field_value(asset, field);
                Ok(field_value == *value)
            }
            RuleCondition::FieldNotEmpty(field) => {
                let field_value = self.get_asset_field_value(asset, field);
                Ok(!field_value.trim().is_empty())
            }
            RuleCondition::And(conditions) => {
                for condition in conditions {
                    if !self.evaluate_rule_condition(condition, asset)? {
                        return Ok(false);
                    }
                }
                Ok(true)
            }
            RuleCondition::Or(conditions) => {
                for condition in conditions {
                    if self.evaluate_rule_condition(condition, asset)? {
                        return Ok(true);
                    }
                }
                Ok(false)
            }
            RuleCondition::Not(condition) => {
                Ok(!self.evaluate_rule_condition(condition, asset)?)
            }
        }
    }

    /// Get asset field value as string
    fn get_asset_field_value(&self, asset: &Asset, field: &str) -> String {
        match field {
            "asset_id" => asset.asset_id.clone(),
            "asset_name" => asset.asset_name.clone(),
            "description" => asset.description.clone(),
            "owner" => asset.owner.clone(),
            "location" => asset.location.clone().unwrap_or_default(),
            "asset_type" => format!("{:?}", asset.asset_type),
            "environment" => format!("{:?}", asset.environment),
            "criticality" => format!("{:?}", asset.criticality),
            _ => String::new(),
        }
    }

    /// Apply a single validation rule
    fn apply_single_rule(&self, rule: &AssetValidationRule, asset: &Asset, result: &mut AssetValidationResult) -> Result<()> {
        match &rule.validation {
            RuleValidation::RequiredField(field) => {
                let field_value = self.get_asset_field_value(asset, field);
                if field_value.trim().is_empty() {
                    let error = ValidationError {
                        code: format!("RULE_{}", rule.name.to_uppercase()),
                        message: format!("Required field '{}' is missing", field),
                        field: Some(field.clone()),
                        asset_id: Some(asset.asset_id.clone()),
                        row: None,
                    };

                    match rule.severity {
                        ValidationSeverity::Error | ValidationSeverity::Critical => {
                            result.errors.push(error);
                        }
                        ValidationSeverity::Warning => {
                            result.warnings.push(ValidationWarning {
                                code: error.code,
                                message: error.message,
                                field: error.field,
                                asset_id: error.asset_id,
                                row: None,
                            });
                        }
                        ValidationSeverity::Info => {
                            // Info level - could be logged but not added to results
                        }
                    }
                }
            }
            RuleValidation::FieldEnumeration { field, allowed_values } => {
                let field_value = self.get_asset_field_value(asset, field);
                if !field_value.trim().is_empty() && !allowed_values.contains(&field_value) {
                    result.errors.push(ValidationError {
                        code: format!("RULE_{}", rule.name.to_uppercase()),
                        message: format!("Field '{}' has invalid value: {}", field, field_value),
                        field: Some(field.clone()),
                        asset_id: Some(asset.asset_id.clone()),
                        row: None,
                    });
                }
            }
            _ => {
                // Other validation types would be implemented here
            }
        }

        Ok(())
    }

    /// Determine compliance status based on validation results
    fn determine_compliance_status(&self, result: &AssetValidationResult) -> ComplianceStatusType {
        if !result.errors.is_empty() {
            ComplianceStatusType::NonCompliant
        } else if !result.warnings.is_empty() {
            ComplianceStatusType::PartiallyCompliant
        } else {
            ComplianceStatusType::Compliant
        }
    }

    /// Check if MAC address format is valid
    fn is_valid_mac_address(&self, mac: &str) -> bool {
        // Basic MAC address validation (supports common formats)
        let mac_patterns = [
            r"^([0-9A-Fa-f]{2}[:-]){5}([0-9A-Fa-f]{2})$", // XX:XX:XX:XX:XX:XX or XX-XX-XX-XX-XX-XX
            r"^([0-9A-Fa-f]{4}\.){2}([0-9A-Fa-f]{4})$",    // XXXX.XXXX.XXXX
        ];

        for pattern in &mac_patterns {
            if regex::Regex::new(pattern).unwrap().is_match(mac) {
                return true;
            }
        }

        false
    }
}

impl AssetTypeValidator {
    /// Create a new asset type validator
    pub fn new(allowed_types: &[AssetType]) -> Self {
        let allowed_types = allowed_types.iter().cloned().collect();
        let mut type_aliases = HashMap::new();

        // Add common aliases for asset types
        type_aliases.insert("hw".to_string(), AssetType::Hardware);
        type_aliases.insert("hardware".to_string(), AssetType::Hardware);
        type_aliases.insert("sw".to_string(), AssetType::Software);
        type_aliases.insert("software".to_string(), AssetType::Software);
        type_aliases.insert("net".to_string(), AssetType::Network);
        type_aliases.insert("network".to_string(), AssetType::Network);
        type_aliases.insert("vm".to_string(), AssetType::Virtual);
        type_aliases.insert("virtual".to_string(), AssetType::Virtual);
        type_aliases.insert("data".to_string(), AssetType::Data);
        type_aliases.insert("service".to_string(), AssetType::Service);
        type_aliases.insert("cloud".to_string(), AssetType::Cloud);

        Self {
            allowed_types,
            type_aliases,
        }
    }

    /// Check if asset type is valid
    pub fn is_valid_type(&self, asset_type: &AssetType) -> bool {
        self.allowed_types.contains(asset_type)
    }

    /// Suggest a valid asset type based on input
    pub fn suggest_asset_type(&self, _asset_type: &AssetType) -> Option<AssetType> {
        // Simple suggestion logic - could be enhanced with fuzzy matching
        // Return the first allowed type as a suggestion
        self.allowed_types.iter().next().cloned()
    }
}

impl EnvironmentValidator {
    /// Create a new environment validator
    pub fn new(allowed_environments: &[Environment]) -> Self {
        let allowed_environments = allowed_environments.iter().cloned().collect();
        let mut environment_aliases = HashMap::new();

        // Add common aliases for environments
        environment_aliases.insert("prod".to_string(), Environment::Production);
        environment_aliases.insert("production".to_string(), Environment::Production);
        environment_aliases.insert("dev".to_string(), Environment::Development);
        environment_aliases.insert("development".to_string(), Environment::Development);
        environment_aliases.insert("test".to_string(), Environment::Testing);
        environment_aliases.insert("testing".to_string(), Environment::Testing);
        environment_aliases.insert("stage".to_string(), Environment::Staging);
        environment_aliases.insert("staging".to_string(), Environment::Staging);
        environment_aliases.insert("train".to_string(), Environment::Training);
        environment_aliases.insert("training".to_string(), Environment::Training);
        environment_aliases.insert("dr".to_string(), Environment::DisasterRecovery);
        environment_aliases.insert("disaster_recovery".to_string(), Environment::DisasterRecovery);

        Self {
            allowed_environments,
            environment_aliases,
        }
    }

    /// Check if environment is valid
    pub fn is_valid_environment(&self, environment: &Environment) -> bool {
        self.allowed_environments.contains(environment)
    }

    /// Suggest a valid environment based on input
    pub fn suggest_environment(&self, _environment: &Environment) -> Option<Environment> {
        // Simple suggestion logic - could be enhanced with fuzzy matching
        // Return the first allowed environment as a suggestion
        self.allowed_environments.iter().next().cloned()
    }
}

impl AttributeValidator {
    /// Create a new attribute validator
    pub fn new(allowed_criticality: &[Criticality]) -> Self {
        let allowed_criticality = allowed_criticality.iter().cloned().collect();
        let custom_validators = HashMap::new();

        Self {
            allowed_criticality,
            custom_validators,
        }
    }

    /// Check if criticality level is valid
    pub fn is_valid_criticality(&self, criticality: &Criticality) -> bool {
        self.allowed_criticality.contains(criticality)
    }

    /// Add a custom attribute validator
    pub fn add_custom_validator(&mut self, name: String, validator: AttributeValidatorFn) {
        self.custom_validators.insert(name, validator);
    }
}

impl RelationshipValidator {
    /// Create a new relationship validator
    pub fn new() -> Self {
        let allowed_relationship_types = vec![
            RelationshipType::DependsOn,
            RelationshipType::Hosts,
            RelationshipType::ConnectedTo,
            RelationshipType::Manages,
            RelationshipType::Monitors,
            RelationshipType::BacksUp,
            RelationshipType::Replicates,
            RelationshipType::Related,
        ].into_iter().collect();

        let relationship_rules = vec![
            RelationshipValidationRule {
                name: "software_hardware_dependency".to_string(),
                source_types: vec![AssetType::Software],
                target_types: vec![AssetType::Hardware],
                allowed_relationships: vec![RelationshipType::DependsOn, RelationshipType::Hosts],
                description: "Software can depend on or be hosted by hardware".to_string(),
            },
            RelationshipValidationRule {
                name: "network_connectivity".to_string(),
                source_types: vec![AssetType::Network],
                target_types: vec![AssetType::Hardware, AssetType::Virtual],
                allowed_relationships: vec![RelationshipType::ConnectedTo],
                description: "Network assets can be connected to hardware or virtual assets".to_string(),
            },
        ];

        Self {
            allowed_relationship_types,
            relationship_rules,
        }
    }

    /// Validate a relationship between assets
    pub async fn validate_relationship(
        &self,
        relationship: &AssetRelationship,
        assets: &[Asset],
    ) -> Result<AssetValidationResult> {
        let mut result = AssetValidationResult {
            is_valid: true,
            asset_id: relationship.source_asset_id.clone(),
            field_results: HashMap::new(),
            errors: Vec::new(),
            warnings: Vec::new(),
            suggestions: Vec::new(),
        };

        // Find source and target assets
        let source_asset = assets.iter().find(|a| a.asset_id == relationship.source_asset_id);
        let target_asset = assets.iter().find(|a| a.asset_id == relationship.target_asset_id);

        if source_asset.is_none() {
            result.errors.push(ValidationError {
                code: "INVALID_SOURCE_ASSET".to_string(),
                message: format!("Source asset not found: {}", relationship.source_asset_id),
                field: Some("source_asset_id".to_string()),
                asset_id: Some(relationship.source_asset_id.clone()),
                row: None,
            });
        }

        if target_asset.is_none() {
            result.errors.push(ValidationError {
                code: "INVALID_TARGET_ASSET".to_string(),
                message: format!("Target asset not found: {}", relationship.target_asset_id),
                field: Some("target_asset_id".to_string()),
                asset_id: Some(relationship.source_asset_id.clone()),
                row: None,
            });
        }

        // Validate relationship type
        if !self.allowed_relationship_types.contains(&relationship.relationship_type) {
            result.errors.push(ValidationError {
                code: "INVALID_RELATIONSHIP_TYPE".to_string(),
                message: format!("Invalid relationship type: {:?}", relationship.relationship_type),
                field: Some("relationship_type".to_string()),
                asset_id: Some(relationship.source_asset_id.clone()),
                row: None,
            });
        }

        result.is_valid = result.errors.is_empty();

        Ok(result)
    }
}
