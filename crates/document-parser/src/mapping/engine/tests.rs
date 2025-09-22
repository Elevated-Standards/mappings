//! Tests for the mapping engine functionality
//! Modified: 2025-01-22
//!
//! This module contains comprehensive tests for the mapping engine including
//! optimized lookup creation, column mapping, and fuzzy matching.

#[cfg(test)]
mod tests {
    use super::super::*;
    use crate::mapping::loader::MappingConfigurationLoader;
    use std::fs;
    use tempfile::TempDir;
    use tokio;

    /// Create a temporary directory with test mapping configurations
    async fn create_test_mappings_dir() -> Result<TempDir, Box<dyn std::error::Error>> {
        let temp_dir = TempDir::new()?;
        let mappings_dir = temp_dir.path().join("mappings");
        fs::create_dir_all(&mappings_dir)?;

        // Create test POA&M mappings
        let poam_json = serde_json::json!({
            "description": "Test POA&M mappings",
            "version": "1.0",
            "required_columns": {
                "weakness_id": {
                    "column_names": ["Weakness ID", "Vuln ID", "Finding ID"],
                    "field": "weakness_id",
                    "required": true,
                    "data_type": "string",
                    "validation": "unique_identifier"
                },
                "weakness_name": {
                    "column_names": ["Weakness Name", "Vulnerability Title", "Finding Title"],
                    "field": "weakness_name",
                    "required": true,
                    "data_type": "string"
                }
            },
            "validation_rules": {
                "severity_levels": ["Low", "Moderate", "High", "Critical"],
                "status_values": ["Open", "Ongoing", "Completed"]
            }
        });

        fs::write(
            mappings_dir.join("poam_mappings.json"),
            serde_json::to_string_pretty(&poam_json).unwrap(),
        )?;

        // Create test inventory mappings
        let inventory_json = serde_json::json!({
            "description": "Test inventory mappings",
            "version": "1.0",
            "fedramp_iiw_mappings": {
                "required_columns": {
                    "asset_id": {
                        "column_names": ["Asset ID", "Component ID"],
                        "field": "uuid",
                        "required": true,
                        "validation": "unique_identifier"
                    }
                }
            },
            "validation_rules": {
                "asset_types": ["hardware", "software"],
                "boolean_values": ["yes", "no"]
            },
            "component_grouping": {
                "strategies": {}
            },
            "component_type_mappings": {},
            "security_mappings": {
                "criticality_to_impact": {},
                "risk_factors": {}
            },
            "control_inheritance": {
                "infrastructure_controls": [],
                "platform_controls": [],
                "inheritance_mappings": {}
            }
        });

        fs::write(
            mappings_dir.join("inventory_mappings.json"),
            serde_json::to_string_pretty(&inventory_json).unwrap(),
        )?;

        Ok(temp_dir)
    }

    #[tokio::test]
    async fn test_optimized_lookup_creation() {
        let temp_dir = create_test_mappings_dir().await.unwrap();
        let mut loader = MappingConfigurationLoader::new(temp_dir.path());
        let config = loader.load_all_configurations().await.unwrap();

        let lookup = OptimizedMappingLookup::from_configuration(&config);
        assert!(lookup.is_ok());

        let lookup = lookup.unwrap();
        let stats = lookup.get_statistics();
        assert!(stats["exact_matches"].as_u64().unwrap() > 0);
    }

    #[tokio::test]
    async fn test_column_mapping_with_optimized_lookup() {
        let temp_dir = create_test_mappings_dir().await.unwrap();
        let mut mapper = ColumnMapper::with_config_loader(temp_dir.path());

        let result = mapper.load_configurations().await;
        assert!(result.is_ok());

        let headers = vec!["Asset ID".to_string()];
        let mapping_results = mapper.map_columns(&headers).unwrap();

        assert_eq!(mapping_results.len(), 1);
        assert!(mapping_results.iter().any(|r| r.target_field == "uuid"));
    }

    #[tokio::test]
    async fn test_fuzzy_matching() {
        let temp_dir = create_test_mappings_dir().await.unwrap();
        let mut mapper = ColumnMapper::with_config_loader(temp_dir.path());
        mapper.load_configurations().await.unwrap();

        // Test fuzzy matching with slight variations
        let headers = vec!["Asset_ID".to_string(), "Component_ID".to_string()];
        let mapping_results = mapper.map_columns(&headers).unwrap();

        assert!(!mapping_results.is_empty());
        assert!(mapping_results.iter().any(|r| r.confidence > 0.7));
    }

    #[tokio::test]
    async fn test_exact_match_priority() {
        let temp_dir = create_test_mappings_dir().await.unwrap();
        let mut mapper = ColumnMapper::with_config_loader(temp_dir.path());
        mapper.load_configurations().await.unwrap();

        // Test that exact matches get priority over fuzzy matches
        let headers = vec!["Asset ID".to_string()];
        let mapping_results = mapper.map_columns(&headers).unwrap();

        assert_eq!(mapping_results.len(), 1);
        assert_eq!(mapping_results[0].confidence, 1.0); // Exact match
        assert_eq!(mapping_results[0].target_field, "uuid");
    }

    #[tokio::test]
    async fn test_mapping_suggestions() {
        let temp_dir = create_test_mappings_dir().await.unwrap();
        let mut mapper = ColumnMapper::with_config_loader(temp_dir.path());
        mapper.load_configurations().await.unwrap();

        // Test getting mapping suggestions
        let suggestions = mapper.get_mapping_suggestions("Asset", 5).unwrap();
        assert!(!suggestions.is_empty());
        
        // Suggestions should be sorted by confidence
        for i in 1..suggestions.len() {
            assert!(suggestions[i-1].confidence >= suggestions[i].confidence);
        }
    }

    #[tokio::test]
    async fn test_validation_rules() {
        let temp_dir = create_test_mappings_dir().await.unwrap();
        let mut mapper = ColumnMapper::with_config_loader(temp_dir.path());
        mapper.load_configurations().await.unwrap();

        // Test boolean validation
        assert!(mapper.validate_mapped_data("test_boolean", "true").unwrap());
        assert!(mapper.validate_mapped_data("test_boolean", "false").unwrap());
        assert!(mapper.validate_mapped_data("test_boolean", "yes").unwrap());
        assert!(mapper.validate_mapped_data("test_boolean", "no").unwrap());

        // Test numeric validation
        assert!(mapper.validate_mapped_data("test_numeric", "123.45").unwrap());
        assert!(mapper.validate_mapped_data("test_numeric", "0").unwrap());

        // Test email validation (basic)
        assert!(mapper.validate_mapped_data("test_email", "test@example.com").unwrap());

        // Test URL validation (basic)
        assert!(mapper.validate_mapped_data("test_url", "https://example.com").unwrap());
    }

    #[tokio::test]
    async fn test_required_fields() {
        let temp_dir = create_test_mappings_dir().await.unwrap();
        let mut mapper = ColumnMapper::with_config_loader(temp_dir.path());
        mapper.load_configurations().await.unwrap();

        let required_fields = mapper.get_required_fields();
        assert!(!required_fields.is_empty());
        assert!(required_fields.contains(&"uuid".to_string()));
    }

    #[tokio::test]
    async fn test_confidence_threshold() {
        let mut mapper = ColumnMapper::new();
        
        // Test default confidence threshold
        assert_eq!(mapper.get_min_confidence(), 0.7);
        
        // Test setting confidence threshold
        mapper.set_min_confidence(0.8);
        assert_eq!(mapper.get_min_confidence(), 0.8);
        
        // Test clamping
        mapper.set_min_confidence(1.5);
        assert_eq!(mapper.get_min_confidence(), 1.0);
        
        mapper.set_min_confidence(-0.5);
        assert_eq!(mapper.get_min_confidence(), 0.0);
    }

    #[test]
    fn test_column_name_normalization() {
        assert_eq!(
            OptimizedMappingLookup::normalize_column_name("Asset ID"),
            "assetid"
        );
        assert_eq!(
            OptimizedMappingLookup::normalize_column_name("Component_Name"),
            "componentname"
        );
        assert_eq!(
            OptimizedMappingLookup::normalize_column_name("POA&M Status"),
            "poamstatus"
        );
        assert_eq!(
            OptimizedMappingLookup::normalize_column_name("System-Name"),
            "systemname"
        );
    }

    #[test]
    fn test_mapping_result_quality_score() {
        use super::types::{MappingResult, MappingSourceType};

        let result = MappingResult::new(
            "test_column".to_string(),
            "test_field".to_string(),
            0.8,
            MappingSourceType::Poam,
            true,
            None,
            false,
        );

        let quality = result.quality_score();
        assert!(quality > 0.8); // Should be boosted due to required field and high-priority source
        assert!(quality <= 1.0);
    }

    #[test]
    fn test_mapping_source_type_priority() {
        use super::types::MappingSourceType;

        assert_eq!(MappingSourceType::Poam.priority(), 1);
        assert_eq!(MappingSourceType::Inventory.priority(), 2);
        assert_eq!(MappingSourceType::SspSection.priority(), 3);
        assert_eq!(MappingSourceType::Custom.priority(), 4);
    }
}
