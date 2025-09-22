// Modified: 2025-01-22

//! Core configuration loading functionality
//!
//! This module contains the main implementation of the MappingConfigurationLoader
//! including basic loading methods and configuration management.

use super::types::*;
use fedramp_core::{Result, Error};
use std::path::Path;
use std::sync::{Arc, RwLock};
use std::collections::HashMap;
use tokio::fs;
use tracing::{debug, info, warn, error};
use crate::mapping::config::{MappingConfiguration, LoadingMetrics};
use crate::mapping::inventory::InventoryMappings;
use crate::mapping::poam::PoamMappings;
use crate::mapping::ssp::SspSections;
use crate::mapping::control_document::{ControlMappings, DocumentStructures};

impl MappingConfigurationLoader {
    /// Create a new configuration loader
    pub fn new<P: AsRef<Path>>(base_dir: P) -> Self {
        Self {
            base_dir: base_dir.as_ref().to_path_buf(),
            cached_config: Arc::new(RwLock::new(None)),
            watcher: None,
            reload_tx: None,
            config_backup: Arc::new(RwLock::new(None)),
            file_mtimes: Arc::new(RwLock::new(HashMap::new())),
            load_metrics: Arc::new(RwLock::new(LoadingMetrics::default())),
        }
    }

    /// Load all mapping configurations from default file locations
    pub async fn load_all_configurations(&mut self) -> Result<MappingConfiguration> {
        info!("Loading all mapping configurations from {}", self.base_dir.display());

        let mut config = MappingConfiguration {
            inventory_mappings: None,
            poam_mappings: None,
            ssp_sections: None,
            controls: None,
            documents: None,
        };

        // Load inventory mappings
        if let Ok(inventory) = self.load_inventory_mappings().await {
            config.inventory_mappings = Some(inventory);
            info!("Successfully loaded inventory mappings");
        } else {
            warn!("Failed to load inventory mappings");
        }

        // Load POA&M mappings
        if let Ok(poam) = self.load_poam_mappings().await {
            config.poam_mappings = Some(poam);
            info!("Successfully loaded POA&M mappings");
        } else {
            warn!("Failed to load POA&M mappings");
        }

        // Load SSP sections
        if let Ok(ssp) = self.load_ssp_sections().await {
            config.ssp_sections = Some(ssp);
            info!("Successfully loaded SSP sections");
        } else {
            warn!("Failed to load SSP sections");
        }

        // Load control mappings
        if let Ok(controls) = self.load_control_mappings().await {
            config.controls = Some(controls);
            info!("Successfully loaded control mappings");
        } else {
            warn!("Failed to load control mappings");
        }

        // Load document structures
        if let Ok(documents) = self.load_document_structures().await {
            config.documents = Some(documents);
            info!("Successfully loaded document structures");
        } else {
            warn!("Failed to load document structures");
        }

        // Update cached configuration atomically
        {
            let mut cache = self.cached_config.write().unwrap();
            *cache = Some(config.clone());
        }

        Ok(config)
    }

    /// Load inventory mappings from file
    pub async fn load_inventory_mappings(&self) -> Result<InventoryMappings> {
        let file_path = self.base_dir.join("mappings").join("inventory_mappings.json");
        debug!("Loading inventory mappings from {}", file_path.display());

        let content = fs::read_to_string(&file_path).await.map_err(|e| {
            Error::document_parsing(format!(
                "Failed to read inventory mappings file {}: {}",
                file_path.display(),
                e
            ))
        })?;

        let inventory: InventoryMappings = serde_json::from_str(&content).map_err(|e| {
            Error::document_parsing(format!(
                "Failed to parse inventory mappings JSON from {}: {} at line {}",
                file_path.display(),
                e,
                e.line()
            ))
        })?;

        debug!("Successfully loaded inventory mappings with {} required columns",
               inventory.fedramp_iiw_mappings.required_columns.len());

        Ok(inventory)
    }

    /// Load POA&M mappings from file
    pub async fn load_poam_mappings(&self) -> Result<PoamMappings> {
        let file_path = self.base_dir.join("mappings").join("poam_mappings.json");
        debug!("Loading POA&M mappings from {}", file_path.display());

        let content = fs::read_to_string(&file_path).await.map_err(|e| {
            Error::document_parsing(format!(
                "Failed to read POA&M mappings file {}: {}",
                file_path.display(),
                e
            ))
        })?;

        let poam: PoamMappings = serde_json::from_str(&content).map_err(|e| {
            Error::document_parsing(format!(
                "Failed to parse POA&M mappings JSON from {}: {} at line {}",
                file_path.display(),
                e,
                e.line()
            ))
        })?;

        debug!("Successfully loaded POA&M mappings with {} required columns",
               poam.fedramp_v3_mappings.required_columns.len());

        Ok(poam)
    }

    /// Load SSP sections from file
    pub async fn load_ssp_sections(&self) -> Result<SspSections> {
        let file_path = self.base_dir.join("mappings").join("ssp_sections.json");
        debug!("Loading SSP sections from {}", file_path.display());

        let content = fs::read_to_string(&file_path).await.map_err(|e| {
            Error::document_parsing(format!(
                "Failed to read SSP sections file {}: {}",
                file_path.display(),
                e
            ))
        })?;

        let ssp: SspSections = serde_json::from_str(&content).map_err(|e| {
            Error::document_parsing(format!(
                "Failed to parse SSP sections JSON from {}: {} at line {}",
                file_path.display(),
                e,
                e.line()
            ))
        })?;

        debug!("Successfully loaded SSP sections with {} mappings",
               ssp.section_mappings.mappings.len());

        Ok(ssp)
    }

    /// Load control mappings from file
    pub async fn load_control_mappings(&self) -> Result<ControlMappings> {
        let file_path = self.base_dir.join("schema").join("_controls.json");
        debug!("Loading control mappings from {}", file_path.display());

        let content = fs::read_to_string(&file_path).await.map_err(|e| {
            Error::document_parsing(format!(
                "Failed to read control mappings file {}: {}",
                file_path.display(),
                e
            ))
        })?;

        let controls: ControlMappings = serde_json::from_str(&content).map_err(|e| {
            Error::document_parsing(format!(
                "Failed to parse control mappings JSON from {}: {} at line {}",
                file_path.display(),
                e,
                e.line()
            ))
        })?;

        debug!("Successfully loaded control mappings");

        Ok(controls)
    }

    /// Load document structures from file
    pub async fn load_document_structures(&self) -> Result<DocumentStructures> {
        let file_path = self.base_dir.join("schema").join("_document.json");
        debug!("Loading document structures from {}", file_path.display());

        let content = fs::read_to_string(&file_path).await.map_err(|e| {
            Error::document_parsing(format!(
                "Failed to read document structures file {}: {}",
                file_path.display(),
                e
            ))
        })?;

        let documents: DocumentStructures = serde_json::from_str(&content).map_err(|e| {
            Error::document_parsing(format!(
                "Failed to parse document structures JSON from {}: {} at line {}",
                file_path.display(),
                e,
                e.line()
            ))
        })?;

        debug!("Successfully loaded document structures");

        Ok(documents)
    }

    /// Get cached configuration
    pub fn get_cached_configuration(&self) -> Option<MappingConfiguration> {
        let cache = self.cached_config.read().unwrap();
        cache.clone()
    }

    /// Clear cached configuration
    pub fn clear_cache(&self) {
        let mut cache = self.cached_config.write().unwrap();
        *cache = None;
        info!("Configuration cache cleared");
    }

    /// Get base directory
    pub fn base_dir(&self) -> &Path {
        &self.base_dir
    }

    /// Check if configuration is cached
    pub fn is_cached(&self) -> bool {
        let cache = self.cached_config.read().unwrap();
        cache.is_some()
    }
}
