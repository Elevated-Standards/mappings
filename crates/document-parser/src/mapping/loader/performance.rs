// Modified: 2025-01-22

//! Performance optimization for configuration loading
//!
//! This module provides parallel loading, timing, metrics collection,
//! and performance optimization features for configuration loading.

use super::types::*;
use fedramp_core::{Result, Error};
use std::collections::HashMap;
use tracing::{debug, info, warn};
use crate::mapping::config::MappingConfiguration;
use crate::mapping::inventory::InventoryMappings;
use crate::mapping::poam::PoamMappings;
use crate::mapping::ssp::SspSections;
use crate::mapping::control_document::{ControlMappings, DocumentStructures};

impl MappingConfigurationLoader {
    /// Load all configurations with enhanced performance and error handling
    pub async fn load_all_configurations_optimized(&mut self) -> Result<MappingConfiguration> {
        let start_time = std::time::Instant::now();
        info!("Loading all mapping configurations (optimized) from {}", self.base_dir.display());

        // Create backup of current configuration for rollback
        self.create_configuration_backup().await?;

        let mut config = MappingConfiguration {
            inventory_mappings: None,
            poam_mappings: None,
            ssp_sections: None,
            controls: None,
            documents: None,
        };

        let mut load_errors = Vec::new();
        let mut file_times = HashMap::new();

        // Load configurations in parallel for better performance
        let (inventory_result, poam_result, ssp_result, controls_result, documents_result) = tokio::join!(
            self.load_inventory_mappings_with_timing(),
            self.load_poam_mappings_with_timing(),
            self.load_ssp_sections_with_timing(),
            self.load_control_mappings_with_timing(),
            self.load_document_structures_with_timing()
        );

        // Process results and collect errors
        if let Ok((inventory, load_time)) = inventory_result {
            config.inventory_mappings = Some(inventory);
            file_times.insert("inventory_mappings.json".to_string(), load_time);
            info!("Successfully loaded inventory mappings in {}ms", load_time);
        } else if let Err(e) = inventory_result {
            load_errors.push(format!("Failed to load inventory mappings: {}", e));
            warn!("Failed to load inventory mappings: {}", e);
        }

        if let Ok((poam, load_time)) = poam_result {
            config.poam_mappings = Some(poam);
            file_times.insert("poam_mappings.json".to_string(), load_time);
            info!("Successfully loaded POA&M mappings in {}ms", load_time);
        } else if let Err(e) = poam_result {
            load_errors.push(format!("Failed to load POA&M mappings: {}", e));
            warn!("Failed to load POA&M mappings: {}", e);
        }

        if let Ok((ssp, load_time)) = ssp_result {
            config.ssp_sections = Some(ssp);
            file_times.insert("ssp_sections.json".to_string(), load_time);
            info!("Successfully loaded SSP sections in {}ms", load_time);
        } else if let Err(e) = ssp_result {
            load_errors.push(format!("Failed to load SSP sections: {}", e));
            warn!("Failed to load SSP sections: {}", e);
        }

        if let Ok((controls, load_time)) = controls_result {
            config.controls = Some(controls);
            file_times.insert("_controls.json".to_string(), load_time);
            info!("Successfully loaded control mappings in {}ms", load_time);
        } else if let Err(e) = controls_result {
            load_errors.push(format!("Failed to load control mappings: {}", e));
            warn!("Failed to load control mappings: {}", e);
        }

        if let Ok((documents, load_time)) = documents_result {
            config.documents = Some(documents);
            file_times.insert("_document.json".to_string(), load_time);
            info!("Successfully loaded document structures in {}ms", load_time);
        } else if let Err(e) = documents_result {
            load_errors.push(format!("Failed to load document structures: {}", e));
            warn!("Failed to load document structures: {}", e);
        }

        let total_time = start_time.elapsed().as_millis() as u64;

        // Check if we have at least some critical configurations loaded
        let has_inventory = config.inventory_mappings.is_some();
        let has_poam = config.poam_mappings.is_some();
        let has_ssp = config.ssp_sections.is_some();

        debug!("Configuration loading status: inventory={}, poam={}, ssp={}, errors={}",
               has_inventory, has_poam, has_ssp, load_errors.len());

        if !has_inventory && !has_poam && !has_ssp {
            // Update metrics with failure
            self.update_load_metrics(total_time, file_times, false).await;
            return Err(Error::document_parsing(format!(
                "Failed to load any critical mapping configurations. Errors: {}",
                load_errors.join("; ")
            )));
        }

        // Validate configuration before caching
        match self.validate_configuration(&config) {
            Ok(warnings) => {
                if !warnings.is_empty() {
                    debug!("Configuration validation warnings: {:?}", warnings);
                }
                debug!("Configuration validation passed");
            }
            Err(e) => {
                warn!("Configuration validation failed: {}", e);
                // Don't fail completely on validation errors, just warn
            }
        }

        // Update cached configuration atomically
        {
            let mut cache = self.cached_config.write().unwrap();
            *cache = Some(config.clone());
        }

        // Update metrics with success
        self.update_load_metrics(total_time, file_times, true).await;

        info!("Successfully loaded configuration in {}ms with {} warnings and {} errors",
              total_time, 0, load_errors.len());

        Ok(config)
    }

    /// Load inventory mappings with timing
    pub async fn load_inventory_mappings_with_timing(&self) -> Result<(InventoryMappings, u64)> {
        let start = std::time::Instant::now();
        let result = self.load_inventory_mappings().await?;
        let elapsed = start.elapsed().as_millis() as u64;
        Ok((result, elapsed))
    }

    /// Load POA&M mappings with timing
    pub async fn load_poam_mappings_with_timing(&self) -> Result<(PoamMappings, u64)> {
        let start = std::time::Instant::now();
        let result = self.load_poam_mappings().await?;
        let elapsed = start.elapsed().as_millis() as u64;
        Ok((result, elapsed))
    }

    /// Load SSP sections with timing
    pub async fn load_ssp_sections_with_timing(&self) -> Result<(SspSections, u64)> {
        let start = std::time::Instant::now();
        let result = self.load_ssp_sections().await?;
        let elapsed = start.elapsed().as_millis() as u64;
        Ok((result, elapsed))
    }

    /// Load control mappings with timing
    pub async fn load_control_mappings_with_timing(&self) -> Result<(ControlMappings, u64)> {
        let start = std::time::Instant::now();
        let result = self.load_control_mappings().await?;
        let elapsed = start.elapsed().as_millis() as u64;
        Ok((result, elapsed))
    }

    /// Load document structures with timing
    pub async fn load_document_structures_with_timing(&self) -> Result<(DocumentStructures, u64)> {
        let start = std::time::Instant::now();
        let result = self.load_document_structures().await?;
        let elapsed = start.elapsed().as_millis() as u64;
        Ok((result, elapsed))
    }

    /// Update loading performance metrics
    pub async fn update_load_metrics(&self, total_time: u64, file_times: HashMap<String, u64>, success: bool) {
        let mut metrics = self.load_metrics.write().unwrap();
        metrics.total_load_time_ms = total_time;
        metrics.file_load_times = file_times;
        metrics.last_load_time = Some(std::time::SystemTime::now());

        if success {
            metrics.successful_loads += 1;
        } else {
            metrics.failed_loads += 1;
        }
    }

    /// Get loading performance metrics
    pub fn get_load_metrics(&self) -> crate::mapping::config::LoadingMetrics {
        let metrics = self.load_metrics.read().unwrap();
        metrics.clone()
    }

    /// Check if configuration files have changed since last load
    pub async fn has_configuration_changed(&self) -> Result<bool> {
        let file_paths = vec![
            self.base_dir.join("mappings").join("inventory_mappings.json"),
            self.base_dir.join("mappings").join("poam_mappings.json"),
            self.base_dir.join("mappings").join("ssp_sections.json"),
            self.base_dir.join("schema").join("_controls.json"),
            self.base_dir.join("schema").join("_document.json"),
        ];

        let mtimes = self.file_mtimes.read().unwrap();

        for path in file_paths {
            if !path.exists() {
                continue; // Skip non-existent files
            }

            let metadata = tokio::fs::metadata(&path).await.map_err(|e| {
                Error::document_parsing(format!(
                    "Failed to read metadata for {}: {}",
                    path.display(),
                    e
                ))
            })?;

            let current_mtime = metadata.modified().map_err(|e| {
                Error::document_parsing(format!(
                    "Failed to get modification time for {}: {}",
                    path.display(),
                    e
                ))
            })?;

            if let Some(cached_mtime) = mtimes.get(&path) {
                if current_mtime > *cached_mtime {
                    debug!("Configuration file {} has changed", path.display());
                    return Ok(true);
                }
            } else {
                // File not in cache, consider it changed
                debug!("Configuration file {} not in cache, considering changed", path.display());
                return Ok(true);
            }
        }

        Ok(false)
    }

    /// Force reload configuration if files have changed
    pub async fn reload_if_changed(&mut self) -> Result<Option<MappingConfiguration>> {
        if self.has_configuration_changed().await? {
            info!("Configuration files have changed, reloading...");
            let config = self.load_all_configurations_optimized().await?;
            Ok(Some(config))
        } else {
            debug!("No configuration changes detected");
            Ok(None)
        }
    }
}
