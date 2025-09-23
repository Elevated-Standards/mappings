// Modified: 2025-09-23

//! Relationship mapping for inventory assets
//! 
//! This module handles the discovery and mapping of relationships between assets,
//! including dependencies, connections, and hierarchical relationships.

use super::types::*;
use super::parser::MockWorkbook;
use crate::Result;
use fedramp_core::Error;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use tracing::{debug, info, warn};
use uuid::Uuid;

/// Relationship mapper for discovering asset relationships
#[derive(Debug, Clone)]
pub struct RelationshipMapper {
    /// Configuration for relationship mapping
    config: MapperConfig,
    /// Relationship detection rules
    detection_rules: Vec<RelationshipRule>,
    /// Network topology analyzer
    network_analyzer: NetworkTopologyAnalyzer,
    /// Dependency analyzer
    dependency_analyzer: DependencyAnalyzer,
}

/// Configuration for relationship mapping
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MapperConfig {
    /// Enable automatic relationship detection
    pub enable_auto_detection: bool,
    /// Enable network topology analysis
    pub enable_network_analysis: bool,
    /// Enable dependency analysis
    pub enable_dependency_analysis: bool,
    /// Maximum relationship distance to consider
    pub max_relationship_distance: u32,
    /// Minimum confidence threshold for relationships
    pub confidence_threshold: f64,
    /// Enable bidirectional relationships
    pub enable_bidirectional: bool,
}

impl Default for MapperConfig {
    fn default() -> Self {
        Self {
            enable_auto_detection: true,
            enable_network_analysis: true,
            enable_dependency_analysis: true,
            max_relationship_distance: 3,
            confidence_threshold: 0.6,
            enable_bidirectional: true,
        }
    }
}

/// Rule for detecting relationships between assets
#[derive(Debug, Clone)]
pub struct RelationshipRule {
    /// Rule name
    pub name: String,
    /// Relationship type to create
    pub relationship_type: RelationshipType,
    /// Source asset conditions
    pub source_conditions: Vec<AssetCondition>,
    /// Target asset conditions
    pub target_conditions: Vec<AssetCondition>,
    /// Relationship conditions
    pub relationship_conditions: Vec<RelationshipCondition>,
    /// Rule confidence weight
    pub confidence: f64,
    /// Rule priority
    pub priority: u32,
}

/// Condition for asset matching in relationships
#[derive(Debug, Clone)]
pub enum AssetCondition {
    /// Asset type matches
    AssetTypeEquals(AssetType),
    /// Asset category matches
    CategoryEquals(AssetCategory),
    /// Asset name contains text
    NameContains(String),
    /// Asset has specific attribute
    HasAttribute { key: String, value: Option<String> },
    /// Asset is in specific environment
    EnvironmentEquals(Environment),
    /// Asset has network information
    HasNetworkInfo,
    /// Asset has specific IP address
    HasIpAddress(String),
}

/// Condition for relationship validation
#[derive(Debug, Clone)]
pub enum RelationshipCondition {
    /// Assets are in same network segment
    SameNetworkSegment,
    /// Assets have network connectivity
    NetworkConnectivity,
    /// Assets share common attributes
    SharedAttributes(Vec<String>),
    /// Assets are in same location
    SameLocation,
    /// Assets have compatible types
    CompatibleTypes,
}

/// Network topology analyzer
#[derive(Debug, Clone)]
pub struct NetworkTopologyAnalyzer {
    /// Enable subnet analysis
    pub analyze_subnets: bool,
    /// Enable VLAN analysis
    pub analyze_vlans: bool,
    /// Enable port connectivity analysis
    pub analyze_ports: bool,
}

/// Dependency analyzer
#[derive(Debug, Clone)]
pub struct DependencyAnalyzer {
    /// Enable software dependency analysis
    pub analyze_software_deps: bool,
    /// Enable hardware dependency analysis
    pub analyze_hardware_deps: bool,
    /// Enable service dependency analysis
    pub analyze_service_deps: bool,
}

/// Relationship detection result
#[derive(Debug, Clone)]
pub struct RelationshipDetectionResult {
    /// Detected relationships
    pub relationships: Vec<AssetRelationship>,
    /// Detection statistics
    pub statistics: DetectionStatistics,
    /// Detection warnings
    pub warnings: Vec<String>,
}

/// Statistics for relationship detection
#[derive(Debug, Clone)]
pub struct DetectionStatistics {
    /// Total assets analyzed
    pub total_assets: usize,
    /// Total relationships detected
    pub total_relationships: usize,
    /// Relationships by type
    pub relationships_by_type: HashMap<RelationshipType, usize>,
    /// Average confidence score
    pub average_confidence: f64,
    /// Detection time in milliseconds
    pub detection_time_ms: u64,
}

impl RelationshipMapper {
    /// Create a new relationship mapper
    pub fn new() -> Self {
        Self::with_config(MapperConfig::default())
    }

    /// Create relationship mapper with custom configuration
    pub fn with_config(config: MapperConfig) -> Self {
        let detection_rules = Self::create_default_rules();
        let network_analyzer = NetworkTopologyAnalyzer {
            analyze_subnets: true,
            analyze_vlans: true,
            analyze_ports: true,
        };
        let dependency_analyzer = DependencyAnalyzer {
            analyze_software_deps: true,
            analyze_hardware_deps: true,
            analyze_service_deps: true,
        };

        Self {
            config,
            detection_rules,
            network_analyzer,
            dependency_analyzer,
        }
    }

    /// Map relationships between assets
    pub async fn map_relationships(
        &self,
        assets: &[Asset],
        workbook: &MockWorkbook,
        template_info: &InventoryTemplateInfo,
    ) -> Result<Vec<AssetRelationship>> {
        info!("Starting relationship mapping for {} assets", assets.len());
        let start_time = std::time::Instant::now();

        let mut all_relationships = Vec::new();
        let mut warnings = Vec::new();

        // Parse explicit relationships from worksheets
        if !template_info.relationship_worksheets.is_empty() {
            let explicit_relationships = self.parse_explicit_relationships(workbook, template_info).await?;
            all_relationships.extend(explicit_relationships);
        }

        // Detect automatic relationships if enabled
        if self.config.enable_auto_detection {
            let detected_relationships = self.detect_automatic_relationships(assets).await?;
            all_relationships.extend(detected_relationships.relationships);
            warnings.extend(detected_relationships.warnings);
        }

        // Analyze network topology if enabled
        if self.config.enable_network_analysis {
            let network_relationships = self.analyze_network_topology(assets).await?;
            all_relationships.extend(network_relationships);
        }

        // Analyze dependencies if enabled
        if self.config.enable_dependency_analysis {
            let dependency_relationships = self.analyze_dependencies(assets).await?;
            all_relationships.extend(dependency_relationships);
        }

        // Remove duplicates and validate relationships
        let final_relationships = self.deduplicate_and_validate_relationships(all_relationships, assets)?;

        let detection_time = start_time.elapsed().as_millis() as u64;
        info!(
            "Relationship mapping completed: {} relationships detected in {}ms",
            final_relationships.len(),
            detection_time
        );

        if !warnings.is_empty() {
            warn!("Relationship mapping warnings: {:?}", warnings);
        }

        Ok(final_relationships)
    }

    /// Parse explicit relationships from relationship worksheets
    async fn parse_explicit_relationships(
        &self,
        workbook: &MockWorkbook,
        template_info: &InventoryTemplateInfo,
    ) -> Result<Vec<AssetRelationship>> {
        let mut relationships = Vec::new();

        for worksheet_name in &template_info.relationship_worksheets {
            debug!("Parsing relationships from worksheet: {}", worksheet_name);

            let worksheet_data = workbook.get_worksheet_data(worksheet_name)?;
            let headers = self.extract_relationship_headers(worksheet_name)?;
            let data_rows = worksheet_data;

            for (row_index, row_data) in data_rows.iter().enumerate() {
                match self.parse_relationship_row(row_data, row_index + 2).await {
                    Ok(relationship) => relationships.push(relationship),
                    Err(e) => {
                        warn!("Failed to parse relationship at row {}: {}", row_index + 2, e);
                    }
                }
            }
        }

        debug!("Parsed {} explicit relationships", relationships.len());
        Ok(relationships)
    }

    /// Parse a single relationship row
    async fn parse_relationship_row(
        &self,
        row_data: &HashMap<String, String>,
        row_number: usize,
    ) -> Result<AssetRelationship> {
        let source_asset_id = row_data
            .get("source_asset_id")
            .or_else(|| row_data.get("Source Asset ID"))
            .or_else(|| row_data.get("From Asset"))
            .ok_or_else(|| Error::validation(format!("Source asset ID missing in row {}", row_number)))?
            .trim()
            .to_string();

        let target_asset_id = row_data
            .get("target_asset_id")
            .or_else(|| row_data.get("Target Asset ID"))
            .or_else(|| row_data.get("To Asset"))
            .ok_or_else(|| Error::validation(format!("Target asset ID missing in row {}", row_number)))?
            .trim()
            .to_string();

        let relationship_type = self.parse_relationship_type(
            row_data
                .get("relationship_type")
                .or_else(|| row_data.get("Relationship Type"))
                .or_else(|| row_data.get("Type"))
                .map(|s| s.trim())
                .unwrap_or("Related")
        );

        let description = row_data
            .get("description")
            .or_else(|| row_data.get("Description"))
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty());

        let strength = self.parse_relationship_strength(
            row_data
                .get("strength")
                .or_else(|| row_data.get("Strength"))
                .or_else(|| row_data.get("Priority"))
                .map(|s| s.trim())
                .unwrap_or("Medium")
        );

        let relationship = AssetRelationship {
            id: Uuid::new_v4().to_string(),
            source_asset_id,
            target_asset_id,
            relationship_type,
            description,
            strength,
            attributes: HashMap::new(),
        };

        Ok(relationship)
    }

    /// Parse relationship type from string
    fn parse_relationship_type(&self, type_str: &str) -> RelationshipType {
        match type_str.to_lowercase().as_str() {
            "depends_on" | "depends on" | "dependency" => RelationshipType::DependsOn,
            "hosts" | "hosting" => RelationshipType::Hosts,
            "connected_to" | "connected to" | "connection" => RelationshipType::ConnectedTo,
            "manages" | "management" => RelationshipType::Manages,
            "monitors" | "monitoring" => RelationshipType::Monitors,
            "backs_up" | "backs up" | "backup" => RelationshipType::BacksUp,
            "replicates" | "replication" => RelationshipType::Replicates,
            _ => RelationshipType::Related,
        }
    }

    /// Parse relationship strength from string
    fn parse_relationship_strength(&self, strength_str: &str) -> RelationshipStrength {
        match strength_str.to_lowercase().as_str() {
            "weak" | "low" | "1" => RelationshipStrength::Weak,
            "medium" | "med" | "2" => RelationshipStrength::Medium,
            "strong" | "high" | "3" => RelationshipStrength::Strong,
            "critical" | "crit" | "4" => RelationshipStrength::Critical,
            _ => RelationshipStrength::Medium,
        }
    }

    /// Detect automatic relationships between assets
    async fn detect_automatic_relationships(&self, assets: &[Asset]) -> Result<RelationshipDetectionResult> {
        let mut relationships = Vec::new();
        let mut warnings = Vec::new();
        let mut relationships_by_type = HashMap::new();

        for rule in &self.detection_rules {
            let rule_relationships = self.apply_relationship_rule(rule, assets).await?;
            
            for relationship in rule_relationships {
                let rel_type = relationship.relationship_type.clone();
                *relationships_by_type.entry(rel_type).or_insert(0) += 1;
                relationships.push(relationship);
            }
        }

        let average_confidence = if relationships.is_empty() {
            0.0
        } else {
            relationships.iter().map(|_| 0.8).sum::<f64>() / relationships.len() as f64
        };

        let total_relationships = relationships.len();

        Ok(RelationshipDetectionResult {
            relationships,
            statistics: DetectionStatistics {
                total_assets: assets.len(),
                total_relationships,
                relationships_by_type,
                average_confidence,
                detection_time_ms: 0, // Would be calculated in real implementation
            },
            warnings,
        })
    }

    /// Apply a relationship rule to find matching assets
    async fn apply_relationship_rule(
        &self,
        rule: &RelationshipRule,
        assets: &[Asset],
    ) -> Result<Vec<AssetRelationship>> {
        let mut relationships = Vec::new();

        // Find source assets matching source conditions
        let source_assets: Vec<&Asset> = assets
            .iter()
            .filter(|asset| self.asset_matches_conditions(asset, &rule.source_conditions))
            .collect();

        // Find target assets matching target conditions
        let target_assets: Vec<&Asset> = assets
            .iter()
            .filter(|asset| self.asset_matches_conditions(asset, &rule.target_conditions))
            .collect();

        // Create relationships between matching source and target assets
        for source_asset in &source_assets {
            for target_asset in &target_assets {
                // Skip self-relationships
                if source_asset.asset_id == target_asset.asset_id {
                    continue;
                }

                // Check relationship conditions
                if self.relationship_conditions_met(&rule.relationship_conditions, source_asset, target_asset) {
                    let relationship = AssetRelationship {
                        id: Uuid::new_v4().to_string(),
                        source_asset_id: source_asset.asset_id.clone(),
                        target_asset_id: target_asset.asset_id.clone(),
                        relationship_type: rule.relationship_type.clone(),
                        description: Some(format!("Auto-detected by rule: {}", rule.name)),
                        strength: RelationshipStrength::Medium,
                        attributes: HashMap::new(),
                    };

                    relationships.push(relationship);

                    // Create bidirectional relationship if enabled
                    if self.config.enable_bidirectional && self.is_bidirectional_type(&rule.relationship_type) {
                        let reverse_relationship = AssetRelationship {
                            id: Uuid::new_v4().to_string(),
                            source_asset_id: target_asset.asset_id.clone(),
                            target_asset_id: source_asset.asset_id.clone(),
                            relationship_type: self.get_reverse_relationship_type(&rule.relationship_type),
                            description: Some(format!("Auto-detected reverse by rule: {}", rule.name)),
                            strength: RelationshipStrength::Medium,
                            attributes: HashMap::new(),
                        };

                        relationships.push(reverse_relationship);
                    }
                }
            }
        }

        Ok(relationships)
    }

    /// Check if asset matches conditions
    fn asset_matches_conditions(&self, asset: &Asset, conditions: &[AssetCondition]) -> bool {
        for condition in conditions {
            if !self.asset_matches_condition(asset, condition) {
                return false;
            }
        }
        true
    }

    /// Check if asset matches a single condition
    fn asset_matches_condition(&self, asset: &Asset, condition: &AssetCondition) -> bool {
        match condition {
            AssetCondition::AssetTypeEquals(asset_type) => asset.asset_type == *asset_type,
            AssetCondition::CategoryEquals(category) => asset.asset_category == *category,
            AssetCondition::NameContains(text) => asset.asset_name.to_lowercase().contains(&text.to_lowercase()),
            AssetCondition::HasAttribute { key, value } => {
                if let Some(attr_value) = asset.custom_attributes.get(key) {
                    match value {
                        Some(expected) => attr_value == expected,
                        None => true, // Just check if attribute exists
                    }
                } else {
                    false
                }
            }
            AssetCondition::EnvironmentEquals(env) => asset.environment == *env,
            AssetCondition::HasNetworkInfo => asset.network_info.is_some(),
            AssetCondition::HasIpAddress(ip) => {
                if let Some(network_info) = &asset.network_info {
                    network_info.ip_addresses.iter().any(|addr| addr.to_string() == *ip)
                } else {
                    false
                }
            }
        }
    }

    /// Check if relationship conditions are met
    fn relationship_conditions_met(
        &self,
        conditions: &[RelationshipCondition],
        source_asset: &Asset,
        target_asset: &Asset,
    ) -> bool {
        for condition in conditions {
            if !self.relationship_condition_met(condition, source_asset, target_asset) {
                return false;
            }
        }
        true
    }

    /// Check if a single relationship condition is met
    fn relationship_condition_met(
        &self,
        condition: &RelationshipCondition,
        source_asset: &Asset,
        target_asset: &Asset,
    ) -> bool {
        match condition {
            RelationshipCondition::SameNetworkSegment => {
                if let (Some(source_net), Some(target_net)) = (&source_asset.network_info, &target_asset.network_info) {
                    !source_net.network_segments.is_empty() && 
                    !target_net.network_segments.is_empty() &&
                    source_net.network_segments.iter().any(|seg| target_net.network_segments.contains(seg))
                } else {
                    false
                }
            }
            RelationshipCondition::NetworkConnectivity => {
                // Simplified connectivity check based on IP address ranges
                if let (Some(source_net), Some(target_net)) = (&source_asset.network_info, &target_asset.network_info) {
                    !source_net.ip_addresses.is_empty() && !target_net.ip_addresses.is_empty()
                } else {
                    false
                }
            }
            RelationshipCondition::SharedAttributes(attrs) => {
                attrs.iter().any(|attr| {
                    source_asset.custom_attributes.get(attr) == target_asset.custom_attributes.get(attr) &&
                    source_asset.custom_attributes.contains_key(attr)
                })
            }
            RelationshipCondition::SameLocation => {
                source_asset.location.is_some() && 
                source_asset.location == target_asset.location
            }
            RelationshipCondition::CompatibleTypes => {
                self.are_compatible_types(&source_asset.asset_type, &target_asset.asset_type)
            }
        }
    }

    /// Check if asset types are compatible for relationships
    fn are_compatible_types(&self, type1: &AssetType, type2: &AssetType) -> bool {
        match (type1, type2) {
            (AssetType::Software, AssetType::Hardware) => true,
            (AssetType::Hardware, AssetType::Software) => true,
            (AssetType::Virtual, AssetType::Hardware) => true,
            (AssetType::Hardware, AssetType::Virtual) => true,
            (AssetType::Network, AssetType::Hardware) => true,
            (AssetType::Hardware, AssetType::Network) => true,
            (AssetType::Service, AssetType::Software) => true,
            (AssetType::Software, AssetType::Service) => true,
            _ => false,
        }
    }

    /// Check if relationship type is bidirectional
    fn is_bidirectional_type(&self, rel_type: &RelationshipType) -> bool {
        matches!(rel_type, RelationshipType::ConnectedTo | RelationshipType::Related)
    }

    /// Get reverse relationship type
    fn get_reverse_relationship_type(&self, rel_type: &RelationshipType) -> RelationshipType {
        match rel_type {
            RelationshipType::DependsOn => RelationshipType::Related, // Reverse is not direct
            RelationshipType::Hosts => RelationshipType::Related,
            RelationshipType::Manages => RelationshipType::Related,
            RelationshipType::Monitors => RelationshipType::Related,
            RelationshipType::BacksUp => RelationshipType::Related,
            RelationshipType::Replicates => RelationshipType::Related,
            RelationshipType::ConnectedTo => RelationshipType::ConnectedTo,
            RelationshipType::Related => RelationshipType::Related,
        }
    }

    /// Analyze network topology for relationships
    async fn analyze_network_topology(&self, assets: &[Asset]) -> Result<Vec<AssetRelationship>> {
        let mut relationships = Vec::new();

        // Group assets by network segments
        let mut segment_groups: HashMap<String, Vec<&Asset>> = HashMap::new();
        
        for asset in assets {
            if let Some(network_info) = &asset.network_info {
                for segment in &network_info.network_segments {
                    segment_groups.entry(segment.clone()).or_default().push(asset);
                }
            }
        }

        // Create relationships within network segments
        for (segment, segment_assets) in segment_groups {
            if segment_assets.len() > 1 {
                for i in 0..segment_assets.len() {
                    for j in (i + 1)..segment_assets.len() {
                        let relationship = AssetRelationship {
                            id: Uuid::new_v4().to_string(),
                            source_asset_id: segment_assets[i].asset_id.clone(),
                            target_asset_id: segment_assets[j].asset_id.clone(),
                            relationship_type: RelationshipType::ConnectedTo,
                            description: Some(format!("Connected via network segment: {}", segment)),
                            strength: RelationshipStrength::Medium,
                            attributes: {
                                let mut attrs = HashMap::new();
                                attrs.insert("network_segment".to_string(), segment.clone());
                                attrs
                            },
                        };
                        relationships.push(relationship);
                    }
                }
            }
        }

        debug!("Network topology analysis found {} relationships", relationships.len());
        Ok(relationships)
    }

    /// Analyze dependencies between assets
    async fn analyze_dependencies(&self, assets: &[Asset]) -> Result<Vec<AssetRelationship>> {
        let mut relationships = Vec::new();

        // Simple dependency analysis: software depends on hardware
        for software_asset in assets.iter().filter(|a| a.asset_type == AssetType::Software) {
            for hardware_asset in assets.iter().filter(|a| a.asset_type == AssetType::Hardware) {
                // Check if they share network connectivity or location
                let has_connection = self.assets_have_connection(software_asset, hardware_asset);
                
                if has_connection {
                    let relationship = AssetRelationship {
                        id: Uuid::new_v4().to_string(),
                        source_asset_id: software_asset.asset_id.clone(),
                        target_asset_id: hardware_asset.asset_id.clone(),
                        relationship_type: RelationshipType::DependsOn,
                        description: Some("Software dependency on hardware".to_string()),
                        strength: RelationshipStrength::Strong,
                        attributes: HashMap::new(),
                    };
                    relationships.push(relationship);
                }
            }
        }

        debug!("Dependency analysis found {} relationships", relationships.len());
        Ok(relationships)
    }

    /// Check if two assets have a connection
    fn assets_have_connection(&self, asset1: &Asset, asset2: &Asset) -> bool {
        // Check same location
        if asset1.location.is_some() && asset1.location == asset2.location {
            return true;
        }

        // Check network connectivity
        if let (Some(net1), Some(net2)) = (&asset1.network_info, &asset2.network_info) {
            // Check shared network segments
            if net1.network_segments.iter().any(|seg| net2.network_segments.contains(seg)) {
                return true;
            }

            // Check IP address ranges (simplified)
            if !net1.ip_addresses.is_empty() && !net2.ip_addresses.is_empty() {
                return true; // Simplified - would need proper subnet analysis
            }
        }

        false
    }

    /// Remove duplicate relationships and validate
    fn deduplicate_and_validate_relationships(
        &self,
        relationships: Vec<AssetRelationship>,
        assets: &[Asset],
    ) -> Result<Vec<AssetRelationship>> {
        let mut unique_relationships = Vec::new();
        let mut seen_pairs = HashSet::new();
        let asset_ids: HashSet<String> = assets.iter().map(|a| a.asset_id.clone()).collect();

        for relationship in &relationships {
            // Validate that both assets exist
            if !asset_ids.contains(&relationship.source_asset_id) || 
               !asset_ids.contains(&relationship.target_asset_id) {
                continue;
            }

            // Create a unique key for the relationship pair
            let pair_key = if relationship.source_asset_id < relationship.target_asset_id {
                format!("{}:{}", relationship.source_asset_id, relationship.target_asset_id)
            } else {
                format!("{}:{}", relationship.target_asset_id, relationship.source_asset_id)
            };

            if !seen_pairs.contains(&pair_key) {
                seen_pairs.insert(pair_key);
                unique_relationships.push(relationship.clone());
            }
        }

        debug!("Deduplicated {} relationships to {}", relationships.len(), unique_relationships.len());
        Ok(unique_relationships)
    }

    /// Extract relationship headers from worksheet
    fn extract_relationship_headers(&self, worksheet_name: &str) -> Result<Vec<String>> {
        // Placeholder implementation
        Ok(vec![
            "Source Asset ID".to_string(),
            "Target Asset ID".to_string(),
            "Relationship Type".to_string(),
            "Description".to_string(),
            "Strength".to_string(),
        ])
    }

    /// Extract relationship data from worksheet
    fn extract_relationship_data(
        &self,
        worksheet_name: &str,
        headers: &[String],
    ) -> Result<Vec<HashMap<String, String>>> {
        // Placeholder implementation
        Ok(Vec::new())
    }

    /// Create default relationship detection rules
    fn create_default_rules() -> Vec<RelationshipRule> {
        vec![
            // Software depends on hardware
            RelationshipRule {
                name: "Software-Hardware Dependency".to_string(),
                relationship_type: RelationshipType::DependsOn,
                source_conditions: vec![AssetCondition::AssetTypeEquals(AssetType::Software)],
                target_conditions: vec![AssetCondition::AssetTypeEquals(AssetType::Hardware)],
                relationship_conditions: vec![RelationshipCondition::SameLocation],
                confidence: 0.8,
                priority: 10,
            },
            // Network connectivity
            RelationshipRule {
                name: "Network Connectivity".to_string(),
                relationship_type: RelationshipType::ConnectedTo,
                source_conditions: vec![AssetCondition::HasNetworkInfo],
                target_conditions: vec![AssetCondition::HasNetworkInfo],
                relationship_conditions: vec![RelationshipCondition::SameNetworkSegment],
                confidence: 0.9,
                priority: 20,
            },
        ]
    }

    /// Get mapper configuration
    pub fn get_config(&self) -> &MapperConfig {
        &self.config
    }

    /// Update mapper configuration
    pub fn update_config(&mut self, config: MapperConfig) {
        self.config = config;
    }
}

impl Default for RelationshipMapper {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mapper_creation() {
        let mapper = RelationshipMapper::new();
        assert!(mapper.config.enable_auto_detection);
    }

    #[test]
    fn test_relationship_type_parsing() {
        let mapper = RelationshipMapper::new();
        
        assert_eq!(mapper.parse_relationship_type("depends on"), RelationshipType::DependsOn);
        assert_eq!(mapper.parse_relationship_type("hosts"), RelationshipType::Hosts);
        assert_eq!(mapper.parse_relationship_type("connected to"), RelationshipType::ConnectedTo);
        assert_eq!(mapper.parse_relationship_type("unknown"), RelationshipType::Related);
    }

    #[test]
    fn test_relationship_strength_parsing() {
        let mapper = RelationshipMapper::new();
        
        assert_eq!(mapper.parse_relationship_strength("weak"), RelationshipStrength::Weak);
        assert_eq!(mapper.parse_relationship_strength("strong"), RelationshipStrength::Strong);
        assert_eq!(mapper.parse_relationship_strength("critical"), RelationshipStrength::Critical);
        assert_eq!(mapper.parse_relationship_strength("unknown"), RelationshipStrength::Medium);
    }
}
