// Modified: 2025-09-23

//! Validation for inventory data and assets
//! 
//! This module provides comprehensive validation capabilities for inventory assets,
//! relationships, and data integrity checks.

use super::types::*;
use crate::Result;
use fedramp_core::Error;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::net::IpAddr;
use std::str::FromStr;
use tracing::{debug, info, warn};
use regex::Regex;

/// Inventory validator for comprehensive data validation
#[derive(Debug, Clone)]
pub struct InventoryValidator {
    /// Configuration for validation behavior
    config: ValidatorConfig,
    /// Asset validation rules
    asset_rules: Vec<AssetValidationRule>,
    /// Relationship validation rules
    relationship_rules: Vec<RelationshipValidationRule>,
    /// Field validators
    field_validators: HashMap<String, FieldValidator>,
}

/// Configuration for inventory validation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidatorConfig {
    /// Enable strict validation mode
    pub strict_mode: bool,
    /// Enable asset ID uniqueness validation
    pub validate_asset_id_uniqueness: bool,
    /// Enable network data validation
    pub validate_network_data: bool,
    /// Enable relationship integrity validation
    pub validate_relationship_integrity: bool,
    /// Enable business rule validation
    pub validate_business_rules: bool,
    /// Maximum validation errors before stopping
    pub max_errors: Option<usize>,
    /// Enable warning collection
    pub collect_warnings: bool,
}

impl Default for ValidatorConfig {
    fn default() -> Self {
        Self {
            strict_mode: false,
            validate_asset_id_uniqueness: true,
            validate_network_data: true,
            validate_relationship_integrity: true,
            validate_business_rules: true,
            max_errors: Some(100),
            collect_warnings: true,
        }
    }
}

/// Asset validation rule
#[derive(Debug, Clone)]
pub struct AssetValidationRule {
    /// Rule name
    pub name: String,
    /// Rule description
    pub description: String,
    /// Asset types this rule applies to
    pub applicable_types: Vec<AssetType>,
    /// Validation function
    pub validator: AssetRuleValidator,
    /// Rule severity
    pub severity: ValidationSeverity,
    /// Rule priority
    pub priority: u32,
}

/// Asset rule validator types
#[derive(Debug, Clone)]
pub enum AssetRuleValidator {
    /// Required field validation
    RequiredField(String),
    /// Field format validation
    FieldFormat { field: String, pattern: String },
    /// Field value validation
    FieldValue { field: String, allowed_values: Vec<String> },
    /// Cross-field validation
    CrossField { fields: Vec<String>, rule: String },
    /// Custom validation logic
    Custom(String),
}

/// Relationship validation rule
#[derive(Debug, Clone)]
pub struct RelationshipValidationRule {
    /// Rule name
    pub name: String,
    /// Rule description
    pub description: String,
    /// Relationship types this rule applies to
    pub applicable_types: Vec<RelationshipType>,
    /// Validation function
    pub validator: RelationshipRuleValidator,
    /// Rule severity
    pub severity: ValidationSeverity,
}

/// Relationship rule validator types
#[derive(Debug, Clone)]
pub enum RelationshipRuleValidator {
    /// Asset existence validation
    AssetExistence,
    /// Circular dependency detection
    CircularDependency,
    /// Relationship type compatibility
    TypeCompatibility,
    /// Custom validation logic
    Custom(String),
}

/// Validation severity levels
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum ValidationSeverity {
    /// Information level
    Info,
    /// Warning level
    Warning,
    /// Error level
    Error,
    /// Critical error level
    Critical,
}

/// Field validator for specific data types
#[derive(Debug, Clone)]
pub struct FieldValidator {
    /// Validator name
    pub name: String,
    /// Validation pattern (regex)
    pub pattern: Option<Regex>,
    /// Allowed values
    pub allowed_values: Option<Vec<String>>,
    /// Custom validation function
    pub custom_validator: Option<String>,
}

impl InventoryValidator {
    /// Create a new inventory validator
    pub fn new() -> Self {
        Self::with_config(ValidatorConfig::default())
    }

    /// Create validator with custom configuration
    pub fn with_config(config: ValidatorConfig) -> Self {
        let asset_rules = Self::create_default_asset_rules();
        let relationship_rules = Self::create_default_relationship_rules();
        let field_validators = Self::create_default_field_validators();

        Self {
            config,
            asset_rules,
            relationship_rules,
            field_validators,
        }
    }

    /// Validate complete inventory
    pub async fn validate_inventory(
        &self,
        assets: &[Asset],
        relationships: &[AssetRelationship],
    ) -> Result<InventoryValidationResults> {
        info!("Starting inventory validation for {} assets and {} relationships", 
              assets.len(), relationships.len());

        let mut results = InventoryValidationResults::default();
        let mut error_count = 0;

        // Validate asset uniqueness
        if self.config.validate_asset_id_uniqueness {
            self.validate_asset_id_uniqueness(assets, &mut results)?;
        }

        // Validate individual assets
        for asset in assets {
            let asset_result = self.validate_asset(asset).await?;
            
            error_count += asset_result.errors.len();
            if let Some(max_errors) = self.config.max_errors {
                if error_count >= max_errors {
                    warn!("Maximum error count reached: {}", max_errors);
                    break;
                }
            }

            results.asset_results.insert(asset.asset_id.clone(), asset_result);
        }

        // Validate relationships
        if self.config.validate_relationship_integrity {
            for relationship in relationships {
                let rel_result = self.validate_relationship(relationship, assets).await?;
                results.relationship_results.push(rel_result);
            }
        }

        // Validate business rules
        if self.config.validate_business_rules {
            self.validate_business_rules(assets, relationships, &mut results).await?;
        }

        // Calculate summary
        results.summary = self.calculate_validation_summary(&results);
        results.is_valid = results.summary.total_errors == 0;

        info!("Inventory validation completed: {} errors, {} warnings", 
              results.summary.total_errors, results.summary.total_warnings);

        Ok(results)
    }

    /// Validate asset ID uniqueness
    fn validate_asset_id_uniqueness(
        &self,
        assets: &[Asset],
        results: &mut InventoryValidationResults,
    ) -> Result<()> {
        let mut seen_ids = HashSet::new();
        let mut duplicate_ids = Vec::new();

        for asset in assets {
            if !seen_ids.insert(asset.asset_id.clone()) {
                duplicate_ids.push(asset.asset_id.clone());
            }
        }

        for duplicate_id in duplicate_ids {
            results.errors.push(ValidationError {
                code: "DUPLICATE_ASSET_ID".to_string(),
                message: format!("Duplicate asset ID found: {}", duplicate_id),
                asset_id: Some(duplicate_id),
                field: Some("asset_id".to_string()),
                row: None,
            });
        }

        Ok(())
    }

    /// Validate a single asset
    pub async fn validate_asset(&self, asset: &Asset) -> Result<AssetValidationResult> {
        let mut result = AssetValidationResult {
            asset_id: asset.asset_id.clone(),
            is_valid: true,
            field_results: HashMap::new(),
            errors: Vec::new(),
            warnings: Vec::new(),
            suggestions: Vec::new(),
        };

        // Validate basic fields
        self.validate_basic_asset_fields(asset, &mut result)?;

        // Validate network data if present
        if self.config.validate_network_data {
            if let Some(network_info) = &asset.network_info {
                self.validate_network_info(network_info, &mut result)?;
            }
        }

        // Apply asset validation rules
        for rule in &self.asset_rules {
            if rule.applicable_types.is_empty() || rule.applicable_types.contains(&asset.asset_type) {
                self.apply_asset_rule(rule, asset, &mut result)?;
            }
        }

        // Update overall validation status
        result.is_valid = result.errors.is_empty();

        Ok(result)
    }

    /// Validate basic asset fields
    fn validate_basic_asset_fields(
        &self,
        asset: &Asset,
        result: &mut AssetValidationResult,
    ) -> Result<()> {
        // Validate asset ID
        let asset_id_result = self.validate_field("asset_id", &asset.asset_id)?;
        result.field_results.insert("asset_id".to_string(), asset_id_result);

        // Validate asset name
        let asset_name_result = self.validate_field("asset_name", &asset.asset_name)?;
        result.field_results.insert("asset_name".to_string(), asset_name_result);

        // Validate owner
        if !asset.owner.is_empty() {
            let owner_result = self.validate_field("owner", &asset.owner)?;
            result.field_results.insert("owner".to_string(), owner_result);
        }

        // Validate description
        if asset.description.len() > 1000 {
            result.warnings.push(ValidationWarning {
                code: "LONG_DESCRIPTION".to_string(),
                message: "Description is very long (>1000 characters)".to_string(),
                asset_id: Some(asset.asset_id.clone()),
                field: Some("description".to_string()),
                row: None,
            });
        }

        Ok(())
    }

    /// Validate network information
    fn validate_network_info(
        &self,
        network_info: &NetworkInfo,
        result: &mut AssetValidationResult,
    ) -> Result<()> {
        // Validate IP addresses
        for (i, ip_addr) in network_info.ip_addresses.iter().enumerate() {
            let field_name = format!("ip_address_{}", i);
            let ip_result = self.validate_field("ip_address", &ip_addr.to_string())?;
            result.field_results.insert(field_name, ip_result);
        }

        // Validate MAC addresses
        for (i, mac_addr) in network_info.mac_addresses.iter().enumerate() {
            let field_name = format!("mac_address_{}", i);
            let mac_result = self.validate_field("mac_address", mac_addr)?;
            result.field_results.insert(field_name, mac_result);
        }

        // Validate DNS names
        for (i, dns_name) in network_info.dns_names.iter().enumerate() {
            let field_name = format!("dns_name_{}", i);
            let dns_result = self.validate_field("dns_name", dns_name)?;
            result.field_results.insert(field_name, dns_result);
        }

        Ok(())
    }

    /// Validate a single field
    fn validate_field(&self, field_type: &str, value: &str) -> Result<FieldValidationResult> {
        let mut field_result = FieldValidationResult {
            field: field_type.to_string(),
            is_valid: true,
            error: None,
            warning: None,
        };

        // Check if field is empty for required fields
        if value.trim().is_empty() {
            match field_type {
                "asset_id" | "asset_name" => {
                    field_result.is_valid = false;
                    field_result.error = Some(format!("{} cannot be empty", field_type));
                    return Ok(field_result);
                }
                _ => {}
            }
        }

        // Apply field-specific validation
        if let Some(validator) = self.field_validators.get(field_type) {
            self.apply_field_validator(validator, value, &mut field_result)?;
        }

        Ok(field_result)
    }

    /// Apply field validator
    fn apply_field_validator(
        &self,
        validator: &FieldValidator,
        value: &str,
        result: &mut FieldValidationResult,
    ) -> Result<()> {
        // Pattern validation
        if let Some(pattern) = &validator.pattern {
            if !pattern.is_match(value) {
                result.is_valid = false;
                result.error = Some(format!("Value '{}' does not match required pattern", value));
                return Ok(());
            }
        }

        // Allowed values validation
        if let Some(allowed_values) = &validator.allowed_values {
            if !allowed_values.contains(&value.to_string()) {
                result.is_valid = false;
                result.error = Some(format!("Value '{}' is not in allowed values", value));
                return Ok(());
            }
        }

        // Custom validation would go here
        if let Some(_custom_validator) = &validator.custom_validator {
            // Placeholder for custom validation logic
        }

        Ok(())
    }

    /// Apply asset validation rule
    fn apply_asset_rule(
        &self,
        rule: &AssetValidationRule,
        asset: &Asset,
        result: &mut AssetValidationResult,
    ) -> Result<()> {
        match &rule.validator {
            AssetRuleValidator::RequiredField(field) => {
                let field_value = self.get_asset_field_value(asset, field);
                if field_value.trim().is_empty() {
                    let message = format!("Required field '{}' is missing", field);
                    match rule.severity {
                        ValidationSeverity::Error | ValidationSeverity::Critical => {
                            result.errors.push(ValidationError {
                                code: "MISSING_REQUIRED_FIELD".to_string(),
                                message,
                                asset_id: Some(asset.asset_id.clone()),
                                field: Some(field.clone()),
                                row: None,
                            });
                        }
                        ValidationSeverity::Warning => {
                            result.warnings.push(ValidationWarning {
                                code: "MISSING_REQUIRED_FIELD".to_string(),
                                message,
                                asset_id: Some(asset.asset_id.clone()),
                                field: Some(field.clone()),
                                row: None,
                            });
                        }
                        ValidationSeverity::Info => {
                            // Info level - could be logged but not added to results
                        }
                    }
                }
            }
            AssetRuleValidator::FieldFormat { field, pattern } => {
                let field_value = self.get_asset_field_value(asset, field);
                if !field_value.is_empty() {
                    if let Ok(regex) = Regex::new(pattern) {
                        if !regex.is_match(&field_value) {
                            let message = format!("Field '{}' format is invalid", field);
                            match rule.severity {
                                ValidationSeverity::Error | ValidationSeverity::Critical => {
                                    result.errors.push(ValidationError {
                                        code: "INVALID_FIELD_FORMAT".to_string(),
                                        message,
                                        asset_id: Some(asset.asset_id.clone()),
                                        field: Some(field.clone()),
                                        row: None,
                                    });
                                }
                                ValidationSeverity::Warning => {
                                    result.warnings.push(ValidationWarning {
                                        code: "INVALID_FIELD_FORMAT".to_string(),
                                        message,
                                        asset_id: Some(asset.asset_id.clone()),
                                        field: Some(field.clone()),
                                        row: None,
                                    });
                                }
                                ValidationSeverity::Info => {}
                            }
                        }
                    }
                }
            }
            AssetRuleValidator::FieldValue { field, allowed_values } => {
                let field_value = self.get_asset_field_value(asset, field);
                if !field_value.is_empty() && !allowed_values.contains(&field_value) {
                    let message = format!("Field '{}' has invalid value: {}", field, field_value);
                    match rule.severity {
                        ValidationSeverity::Error | ValidationSeverity::Critical => {
                            result.errors.push(ValidationError {
                                code: "INVALID_FIELD_VALUE".to_string(),
                                message,
                                asset_id: Some(asset.asset_id.clone()),
                                field: Some(field.clone()),
                                row: None,
                            });
                        }
                        ValidationSeverity::Warning => {
                            result.warnings.push(ValidationWarning {
                                code: "INVALID_FIELD_VALUE".to_string(),
                                message,
                                asset_id: Some(asset.asset_id.clone()),
                                field: Some(field.clone()),
                                row: None,
                            });
                        }
                        ValidationSeverity::Info => {}
                    }
                }
            }
            AssetRuleValidator::CrossField { fields: _, rule: _ } => {
                // Placeholder for cross-field validation
            }
            AssetRuleValidator::Custom(_) => {
                // Placeholder for custom validation
            }
        }

        Ok(())
    }

    /// Get asset field value by name
    fn get_asset_field_value(&self, asset: &Asset, field: &str) -> String {
        match field {
            "asset_id" => asset.asset_id.clone(),
            "asset_name" => asset.asset_name.clone(),
            "description" => asset.description.clone(),
            "owner" => asset.owner.clone(),
            "location" => asset.location.clone().unwrap_or_default(),
            "environment" => format!("{:?}", asset.environment),
            "criticality" => format!("{:?}", asset.criticality),
            _ => asset.custom_attributes.get(field).cloned().unwrap_or_default(),
        }
    }

    /// Validate a single relationship
    pub async fn validate_relationship(
        &self,
        relationship: &AssetRelationship,
        assets: &[Asset],
    ) -> Result<RelationshipValidationResult> {
        let mut result = RelationshipValidationResult {
            relationship_id: relationship.id.clone(),
            is_valid: true,
            errors: Vec::new(),
            warnings: Vec::new(),
        };

        // Check if source and target assets exist
        let source_exists = assets.iter().any(|a| a.asset_id == relationship.source_asset_id);
        let target_exists = assets.iter().any(|a| a.asset_id == relationship.target_asset_id);

        if !source_exists {
            result.errors.push(format!("Source asset '{}' does not exist", relationship.source_asset_id));
        }

        if !target_exists {
            result.errors.push(format!("Target asset '{}' does not exist", relationship.target_asset_id));
        }

        // Check for self-relationships
        if relationship.source_asset_id == relationship.target_asset_id {
            result.warnings.push("Self-relationship detected".to_string());
        }

        // Apply relationship validation rules
        for rule in &self.relationship_rules {
            if rule.applicable_types.is_empty() || rule.applicable_types.contains(&relationship.relationship_type) {
                self.apply_relationship_rule(rule, relationship, assets, &mut result)?;
            }
        }

        result.is_valid = result.errors.is_empty();
        Ok(result)
    }

    /// Apply relationship validation rule
    fn apply_relationship_rule(
        &self,
        rule: &RelationshipValidationRule,
        relationship: &AssetRelationship,
        assets: &[Asset],
        result: &mut RelationshipValidationResult,
    ) -> Result<()> {
        match &rule.validator {
            RelationshipRuleValidator::AssetExistence => {
                // Already checked in validate_relationship
            }
            RelationshipRuleValidator::CircularDependency => {
                // Placeholder for circular dependency detection
            }
            RelationshipRuleValidator::TypeCompatibility => {
                // Check if relationship type is compatible with asset types
                if let (Some(source_asset), Some(target_asset)) = (
                    assets.iter().find(|a| a.asset_id == relationship.source_asset_id),
                    assets.iter().find(|a| a.asset_id == relationship.target_asset_id),
                ) {
                    if !self.are_relationship_types_compatible(
                        &relationship.relationship_type,
                        &source_asset.asset_type,
                        &target_asset.asset_type,
                    ) {
                        result.warnings.push(format!(
                            "Relationship type '{:?}' may not be compatible with asset types '{:?}' and '{:?}'",
                            relationship.relationship_type,
                            source_asset.asset_type,
                            target_asset.asset_type
                        ));
                    }
                }
            }
            RelationshipRuleValidator::Custom(_) => {
                // Placeholder for custom validation
            }
        }

        Ok(())
    }

    /// Check if relationship type is compatible with asset types
    fn are_relationship_types_compatible(
        &self,
        rel_type: &RelationshipType,
        source_type: &AssetType,
        target_type: &AssetType,
    ) -> bool {
        match rel_type {
            RelationshipType::DependsOn => {
                // Software can depend on hardware, virtual on hardware, etc.
                matches!(
                    (source_type, target_type),
                    (AssetType::Software, AssetType::Hardware) |
                    (AssetType::Virtual, AssetType::Hardware) |
                    (AssetType::Service, AssetType::Software) |
                    (AssetType::Service, AssetType::Hardware)
                )
            }
            RelationshipType::Hosts => {
                // Hardware can host software/virtual, cloud can host services
                matches!(
                    (source_type, target_type),
                    (AssetType::Hardware, AssetType::Software) |
                    (AssetType::Hardware, AssetType::Virtual) |
                    (AssetType::Cloud, AssetType::Service) |
                    (AssetType::Cloud, AssetType::Software)
                )
            }
            RelationshipType::ConnectedTo => {
                // Network assets can connect to anything
                source_type == &AssetType::Network || target_type == &AssetType::Network
            }
            _ => true, // Other relationship types are generally compatible
        }
    }

    /// Validate business rules
    async fn validate_business_rules(
        &self,
        assets: &[Asset],
        relationships: &[AssetRelationship],
        results: &mut InventoryValidationResults,
    ) -> Result<()> {
        // Business rule: Critical assets should have owners
        for asset in assets {
            if asset.criticality == Criticality::Critical && asset.owner.trim().is_empty() {
                results.warnings.push(ValidationWarning {
                    code: "CRITICAL_ASSET_NO_OWNER".to_string(),
                    message: "Critical asset should have an owner".to_string(),
                    asset_id: Some(asset.asset_id.clone()),
                    field: Some("owner".to_string()),
                    row: None,
                });
            }
        }

        // Business rule: Production assets should have location
        for asset in assets {
            if asset.environment == Environment::Production && asset.location.is_none() {
                results.warnings.push(ValidationWarning {
                    code: "PRODUCTION_ASSET_NO_LOCATION".to_string(),
                    message: "Production asset should have a location".to_string(),
                    asset_id: Some(asset.asset_id.clone()),
                    field: Some("location".to_string()),
                    row: None,
                });
            }
        }

        Ok(())
    }

    /// Calculate validation summary
    fn calculate_validation_summary(&self, results: &InventoryValidationResults) -> ValidationSummary {
        let total_assets = results.asset_results.len();
        let valid_assets = results.asset_results.values().filter(|r| r.is_valid).count();
        let invalid_assets = total_assets - valid_assets;

        let total_relationships = results.relationship_results.len();
        let valid_relationships = results.relationship_results.iter().filter(|r| r.is_valid).count();
        let invalid_relationships = total_relationships - valid_relationships;

        let total_errors = results.errors.len() + 
                          results.asset_results.values().map(|r| r.errors.len()).sum::<usize>() +
                          results.relationship_results.iter().map(|r| r.errors.len()).sum::<usize>();

        let total_warnings = results.warnings.len() +
                            results.asset_results.values().map(|r| r.warnings.len()).sum::<usize>() +
                            results.relationship_results.iter().map(|r| r.warnings.len()).sum::<usize>();

        ValidationSummary {
            total_assets,
            valid_assets,
            invalid_assets,
            total_relationships,
            valid_relationships,
            invalid_relationships,
            total_errors,
            total_warnings,
        }
    }

    /// Create default asset validation rules
    fn create_default_asset_rules() -> Vec<AssetValidationRule> {
        vec![
            AssetValidationRule {
                name: "Asset ID Required".to_string(),
                description: "Asset ID is required for all assets".to_string(),
                applicable_types: vec![], // Applies to all types
                validator: AssetRuleValidator::RequiredField("asset_id".to_string()),
                severity: ValidationSeverity::Error,
                priority: 100,
            },
            AssetValidationRule {
                name: "Asset Name Required".to_string(),
                description: "Asset name is required for all assets".to_string(),
                applicable_types: vec![], // Applies to all types
                validator: AssetRuleValidator::RequiredField("asset_name".to_string()),
                severity: ValidationSeverity::Error,
                priority: 90,
            },
            AssetValidationRule {
                name: "Critical Assets Need Owner".to_string(),
                description: "Critical assets should have an assigned owner".to_string(),
                applicable_types: vec![], // Applies to all types
                validator: AssetRuleValidator::RequiredField("owner".to_string()),
                severity: ValidationSeverity::Warning,
                priority: 50,
            },
        ]
    }

    /// Create default relationship validation rules
    fn create_default_relationship_rules() -> Vec<RelationshipValidationRule> {
        vec![
            RelationshipValidationRule {
                name: "Asset Existence".to_string(),
                description: "Both source and target assets must exist".to_string(),
                applicable_types: vec![], // Applies to all types
                validator: RelationshipRuleValidator::AssetExistence,
                severity: ValidationSeverity::Error,
            },
            RelationshipValidationRule {
                name: "Type Compatibility".to_string(),
                description: "Relationship type should be compatible with asset types".to_string(),
                applicable_types: vec![], // Applies to all types
                validator: RelationshipRuleValidator::TypeCompatibility,
                severity: ValidationSeverity::Warning,
            },
        ]
    }

    /// Create default field validators
    fn create_default_field_validators() -> HashMap<String, FieldValidator> {
        let mut validators = HashMap::new();

        // IP address validator
        validators.insert("ip_address".to_string(), FieldValidator {
            name: "IP Address".to_string(),
            pattern: Some(Regex::new(r"^(?:[0-9]{1,3}\.){3}[0-9]{1,3}$").unwrap()),
            allowed_values: None,
            custom_validator: None,
        });

        // MAC address validator
        validators.insert("mac_address".to_string(), FieldValidator {
            name: "MAC Address".to_string(),
            pattern: Some(Regex::new(r"^([0-9A-Fa-f]{2}[:-]){5}([0-9A-Fa-f]{2})$").unwrap()),
            allowed_values: None,
            custom_validator: None,
        });

        // DNS name validator
        validators.insert("dns_name".to_string(), FieldValidator {
            name: "DNS Name".to_string(),
            pattern: Some(Regex::new(r"^[a-zA-Z0-9]([a-zA-Z0-9\-]{0,61}[a-zA-Z0-9])?(\.[a-zA-Z0-9]([a-zA-Z0-9\-]{0,61}[a-zA-Z0-9])?)*$").unwrap()),
            allowed_values: None,
            custom_validator: None,
        });

        validators
    }

    /// Get validator configuration
    pub fn get_config(&self) -> &ValidatorConfig {
        &self.config
    }

    /// Update validator configuration
    pub fn update_config(&mut self, config: ValidatorConfig) {
        self.config = config;
    }
}

impl Default for InventoryValidator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validator_creation() {
        let validator = InventoryValidator::new();
        assert!(validator.config.validate_asset_id_uniqueness);
    }

    #[test]
    fn test_field_validation() {
        let validator = InventoryValidator::new();
        
        // Test IP address validation
        let ip_result = validator.validate_field("ip_address", "192.168.1.1").unwrap();
        assert!(ip_result.is_valid);
        
        let invalid_ip_result = validator.validate_field("ip_address", "invalid_ip").unwrap();
        assert!(!invalid_ip_result.is_valid);
    }

    #[test]
    fn test_asset_field_value_extraction() {
        let validator = InventoryValidator::new();
        let asset = Asset::new("test_id".to_string(), "test_name".to_string(), AssetType::Hardware);
        
        assert_eq!(validator.get_asset_field_value(&asset, "asset_id"), "test_id");
        assert_eq!(validator.get_asset_field_value(&asset, "asset_name"), "test_name");
    }
}
