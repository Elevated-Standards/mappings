// Modified: 2025-01-22

//! Mapping configuration loader module
//!
//! This module provides functionality to load mapping configurations from JSON files
//! with support for hot-reload, validation, performance optimization, and caching.
//!
//! ## Module Structure
//!
//! - `types` - Core type definitions and data structures
//! - `core` - Main configuration loading functionality
//! - `hot_reload` - File watching and automatic reloading
//! - `performance` - Parallel loading and performance optimization
//! - `cache` - Configuration caching and backup management
//! - `validation` - Configuration validation and consistency checks
//!
//! ## Usage
//!
//! ```rust
//! use crate::mapping::loader::{MappingConfigurationLoader, HotReloadHandler};
//!
//! // Basic usage
//! let mut loader = MappingConfigurationLoader::new("/path/to/mappings");
//! let config = loader.load_all_configurations().await?;
//!
//! // With hot-reload support
//! let (loader, handler) = MappingConfigurationLoader::with_hot_reload("/path/to/mappings")?;
//! tokio::spawn(async move { handler.start().await });
//!
//! // Optimized loading with parallel processing
//! let config = loader.load_all_configurations_optimized().await?;
//! ```

pub mod types;
pub mod core;
pub mod hot_reload;
pub mod performance;
pub mod cache;
pub mod validation;

// Re-export main types for backward compatibility
pub use types::{
    MappingConfigurationLoader,
    HotReloadHandler,
    LoadResult,
    ConfigChangeEvent,
    ChangeType,
    ValidationResult,
    LoadingOptions,
    FileMetadata,
    LoadingStats,
    CacheEntry,
    LoadingError,
    ErrorCategory,
};

pub use cache::{CacheStats};

// Re-export all functionality from core module
pub use core::*;

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;
    use tokio::time::Instant;

    /// Create a temporary directory with test mapping files
    async fn create_test_mappings_dir() -> fedramp_core::Result<TempDir> {
        let temp_dir = TempDir::new().unwrap();
        let mappings_dir = temp_dir.path().join("mappings");
        let schema_dir = temp_dir.path().join("schema");

        fs::create_dir_all(&mappings_dir).unwrap();
        fs::create_dir_all(&schema_dir).unwrap();

        // Create test inventory mappings (matching the actual structure)
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
        ).unwrap();

        // Create test POA&M mappings
        let poam_json = serde_json::json!({
            "poam_mappings": {
                "description": "Test POA&M mappings",
                "version": "1.0",
                "fedramp_v3_mappings": {
                    "required_columns": {
                        "poam_id": {
                            "column_names": ["POA&M Item ID"],
                            "oscal_field": "uuid",
                            "required": true,
                            "validation": "alphanumeric"
                        }
                    },
                    "validation_rules": {
                        "severity_levels": ["Low", "High"],
                        "status_values": ["Open", "Closed"]
                    }
                },
                "risk_mappings": {
                    "severity_to_risk_level": {},
                    "status_to_implementation": {}
                },
                "finding_mappings": {
                    "origin_types": {}
                },
                "milestone_processing": {
                    "patterns": {
                        "multiple_milestones": {
                            "separator_patterns": [";"],
                            "description": "Test patterns"
                        },
                        "milestone_format": {
                            "patterns": ["test"],
                            "groups": ["description"]
                        }
                    }
                },
                "quality_checks": {
                    "required_field_completeness": {
                        "critical_fields": ["poam_id"],
                        "minimum_completion_rate": 0.95
                    },
                    "data_consistency": {
                        "date_logic": "test",
                        "status_logic": "test"
                    },
                    "control_validation": {
                        "verify_control_ids": true,
                        "validate_against_catalog": "test"
                    }
                }
            }
        });

        fs::write(
            mappings_dir.join("poam_mappings.json"),
            serde_json::to_string_pretty(&poam_json).unwrap(),
        ).unwrap();

        // Create test SSP sections
        let ssp_json = serde_json::json!({
            "section_mappings": {
                "description": "Test SSP sections",
                "version": "1.0",
                "mappings": {
                    "system_identification": {
                        "keywords": ["system name"],
                        "target": "system-characteristics.system-name",
                        "required": true
                    }
                }
            },
            "control_extraction": {
                "patterns": [
                    {
                        "name": "nist_800_53",
                        "regex": "\\b[A-Z]{2}-\\d+\\b",
                        "description": "NIST controls"
                    }
                ]
            },
            "table_mappings": {
                "responsibility_matrix": {
                    "keywords": ["responsibility"],
                    "columns": {
                        "control_id": ["control"],
                        "customer_responsibility": ["customer"],
                        "csp_responsibility": ["csp"],
                        "shared_responsibility": ["shared"]
                    }
                },
                "inventory_summary": {
                    "keywords": ["inventory"],
                    "columns": {
                        "component_name": ["name"],
                        "component_type": ["type"],
                        "criticality": ["criticality"],
                        "environment": ["environment"]
                    }
                }
            }
        });

        fs::write(
            mappings_dir.join("ssp_sections.json"),
            serde_json::to_string_pretty(&ssp_json).unwrap(),
        ).unwrap();

        Ok(temp_dir)
    }

    #[tokio::test]
    async fn test_load_inventory_mappings() {
        let temp_dir = create_test_mappings_dir().await.unwrap();
        let loader = MappingConfigurationLoader::new(temp_dir.path());

        let result = loader.load_inventory_mappings().await;
        assert!(result.is_ok());

        let inventory = result.unwrap();
        assert_eq!(inventory.description, "Test inventory mappings");
        assert_eq!(inventory.version, "1.0");
        assert!(inventory.fedramp_iiw_mappings.required_columns.contains_key("asset_id"));
    }

    #[tokio::test]
    async fn test_load_poam_mappings() {
        let temp_dir = create_test_mappings_dir().await.unwrap();
        let loader = MappingConfigurationLoader::new(temp_dir.path());

        let result = loader.load_poam_mappings().await;
        assert!(result.is_ok());

        let poam = result.unwrap();
        assert_eq!(poam.description, "Test POA&M mappings");
        assert!(poam.fedramp_v3_mappings.required_columns.contains_key("poam_id"));
    }

    #[tokio::test]
    async fn test_load_ssp_sections() {
        let temp_dir = create_test_mappings_dir().await.unwrap();
        let loader = MappingConfigurationLoader::new(temp_dir.path());

        let result = loader.load_ssp_sections().await;
        assert!(result.is_ok());

        let ssp = result.unwrap();
        assert_eq!(ssp.section_mappings.description, "Test SSP sections");
        assert!(ssp.section_mappings.mappings.contains_key("system_identification"));
    }

    #[tokio::test]
    async fn test_load_all_configurations() {
        let temp_dir = create_test_mappings_dir().await.unwrap();
        let mut loader = MappingConfigurationLoader::new(temp_dir.path());

        let result = loader.load_all_configurations().await;
        assert!(result.is_ok());

        let config = result.unwrap();
        assert!(config.inventory_mappings.is_some());
        assert!(config.poam_mappings.is_some());
        assert!(config.ssp_sections.is_some());
    }

    #[tokio::test]
    async fn test_configuration_validation() {
        let temp_dir = create_test_mappings_dir().await.unwrap();
        let mut loader = MappingConfigurationLoader::new(temp_dir.path());
        let config = loader.load_all_configurations().await.unwrap();

        let warnings = loader.validate_configuration(&config).unwrap();
        // Should have some warnings due to incomplete test data
        assert!(!warnings.is_empty());
    }

    #[tokio::test]
    async fn test_performance_sub_100ms() {
        let temp_dir = create_test_mappings_dir().await.unwrap();
        let mut loader = MappingConfigurationLoader::new(temp_dir.path());

        let start = Instant::now();
        let _config = loader.load_all_configurations().await.unwrap();
        let duration = start.elapsed();

        assert!(duration.as_millis() < 100, "Loading took {}ms, should be < 100ms", duration.as_millis());
    }

    #[tokio::test]
    async fn test_error_handling_missing_file() {
        let temp_dir = TempDir::new().unwrap();
        let loader = MappingConfigurationLoader::new(temp_dir.path());

        let result = loader.load_inventory_mappings().await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_error_handling_malformed_json() {
        let temp_dir = TempDir::new().unwrap();
        let mappings_dir = temp_dir.path().join("mappings");
        fs::create_dir_all(&mappings_dir).unwrap();

        // Write malformed JSON
        fs::write(
            mappings_dir.join("inventory_mappings.json"),
            "{ invalid json",
        ).unwrap();

        let loader = MappingConfigurationLoader::new(temp_dir.path());
        let result = loader.load_inventory_mappings().await;
        assert!(result.is_err());

        // Check that error message is descriptive
        let error_msg = result.unwrap_err().to_string();
        assert!(error_msg.contains("Failed to parse JSON"));
        assert!(error_msg.contains("line"));
    }
}
