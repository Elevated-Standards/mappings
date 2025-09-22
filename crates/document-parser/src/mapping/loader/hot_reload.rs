// Modified: 2025-01-22

//! Hot-reload functionality for configuration files
//!
//! This module provides file watching and automatic reloading capabilities
//! for mapping configuration files with debouncing and error handling.

use super::types::*;
use fedramp_core::{Result, Error};
use std::path::Path;
use std::sync::{Arc, RwLock};
use std::time::Duration;
use tokio::sync::mpsc;
use tracing::{debug, info, warn, error};
use notify::{Watcher, RecursiveMode, Event, EventKind};
use crate::mapping::inventory::InventoryMappings;
use crate::mapping::poam::PoamMappings;
use crate::mapping::ssp::SspSections;
use crate::mapping::control_document::{ControlMappings, DocumentStructures};

impl MappingConfigurationLoader {
    /// Create a new configuration loader with hot-reload support
    pub fn with_hot_reload<P: AsRef<Path>>(base_dir: P) -> Result<(Self, HotReloadHandler)> {
        let (reload_tx, reload_rx) = mpsc::unbounded_channel();

        let mut loader = Self {
            base_dir: base_dir.as_ref().to_path_buf(),
            cached_config: Arc::new(RwLock::new(None)),
            watcher: None,
            reload_tx: Some(reload_tx),
            config_backup: Arc::new(RwLock::new(None)),
            file_mtimes: Arc::new(RwLock::new(std::collections::HashMap::new())),
            load_metrics: Arc::new(RwLock::new(crate::mapping::config::LoadingMetrics::default())),
        };

        // Set up file system watcher
        let tx = loader.reload_tx.as_ref().unwrap().clone();
        let base_dir_clone = loader.base_dir.clone();

        let mut watcher = notify::recommended_watcher(move |res: notify::Result<Event>| {
            match res {
                Ok(event) => {
                    if matches!(event.kind, EventKind::Modify(_) | EventKind::Create(_)) {
                        for path in event.paths {
                            if path.extension().map_or(false, |ext| ext == "json") {
                                if let Some(relative_path) = path.strip_prefix(&base_dir_clone).ok() {
                                    if let Err(e) = tx.send(relative_path.to_path_buf()) {
                                        error!("Failed to send reload notification: {}", e);
                                    }
                                }
                            }
                        }
                    }
                }
                Err(e) => error!("File watcher error: {}", e),
            }
        }).map_err(|e| Error::document_parsing(format!("Failed to create file watcher: {}", e)))?;

        // Watch the mappings and schema directories
        let mappings_dir = loader.base_dir.join("mappings");
        let schema_dir = loader.base_dir.join("schema");

        if mappings_dir.exists() {
            watcher.watch(&mappings_dir, RecursiveMode::NonRecursive)
                .map_err(|e| Error::document_parsing(format!("Failed to watch mappings directory: {}", e)))?;
        }

        if schema_dir.exists() {
            watcher.watch(&schema_dir, RecursiveMode::NonRecursive)
                .map_err(|e| Error::document_parsing(format!("Failed to watch schema directory: {}", e)))?;
        }

        loader.watcher = Some(watcher);

        let handler = HotReloadHandler {
            loader: Arc::new(RwLock::new(loader)),
            reload_rx,
        };

        // Return a clone of the loader for external use
        let loader_clone = {
            let loader_ref = handler.loader.read().unwrap();
            Self {
                base_dir: loader_ref.base_dir.clone(),
                cached_config: Arc::clone(&loader_ref.cached_config),
                watcher: None, // Don't clone the watcher
                reload_tx: None, // Don't clone the sender
                config_backup: Arc::clone(&loader_ref.config_backup),
                file_mtimes: Arc::clone(&loader_ref.file_mtimes),
                load_metrics: Arc::clone(&loader_ref.load_metrics),
            }
        };

        Ok((loader_clone, handler))
    }
}

impl HotReloadHandler {
    /// Start the hot-reload handler
    pub async fn start(mut self) -> Result<()> {
        info!("Starting hot-reload handler for mapping configurations");

        while let Some(changed_path) = self.reload_rx.recv().await {
            info!("Configuration file changed: {}", changed_path.display());

            // Debounce rapid file changes
            tokio::time::sleep(Duration::from_millis(100)).await;

            // Reload configuration
            if let Err(e) = self.reload_configuration(&changed_path).await {
                error!("Failed to reload configuration after file change: {}", e);
            }
        }

        Ok(())
    }

    /// Reload configuration after file change
    async fn reload_configuration(&self, changed_path: &Path) -> Result<()> {
        let loader = {
            let loader_ref = self.loader.read().unwrap();
            MappingConfigurationLoader {
                base_dir: loader_ref.base_dir.clone(),
                cached_config: Arc::clone(&loader_ref.cached_config),
                watcher: None,
                reload_tx: None,
                config_backup: Arc::clone(&loader_ref.config_backup),
                file_mtimes: Arc::clone(&loader_ref.file_mtimes),
                load_metrics: Arc::clone(&loader_ref.load_metrics),
            }
        };

        // Determine which configuration to reload based on the changed file
        match changed_path.file_name().and_then(|n| n.to_str()) {
            Some("inventory_mappings.json") => {
                info!("Reloading inventory mappings");
                if let Ok(inventory) = loader.load_inventory_mappings().await {
                    self.update_cached_inventory(inventory).await?;
                }
            }
            Some("poam_mappings.json") => {
                info!("Reloading POA&M mappings");
                if let Ok(poam) = loader.load_poam_mappings().await {
                    self.update_cached_poam(poam).await?;
                }
            }
            Some("ssp_sections.json") => {
                info!("Reloading SSP sections");
                if let Ok(ssp) = loader.load_ssp_sections().await {
                    self.update_cached_ssp(ssp).await?;
                }
            }
            Some("_controls.json") => {
                info!("Reloading control mappings");
                if let Ok(controls) = loader.load_control_mappings().await {
                    self.update_cached_controls(controls).await?;
                }
            }
            Some("_document.json") => {
                info!("Reloading document structures");
                if let Ok(documents) = loader.load_document_structures().await {
                    self.update_cached_documents(documents).await?;
                }
            }
            _ => {
                warn!("Unknown configuration file changed: {}", changed_path.display());
            }
        }

        Ok(())
    }

    /// Update cached inventory mappings
    async fn update_cached_inventory(&self, inventory: InventoryMappings) -> Result<()> {
        let loader = self.loader.read().unwrap();
        let mut cache = loader.cached_config.write().unwrap();
        if let Some(config) = cache.as_mut() {
            config.inventory_mappings = Some(inventory);
        }
        Ok(())
    }

    /// Update cached POA&M mappings
    async fn update_cached_poam(&self, poam: PoamMappings) -> Result<()> {
        let loader = self.loader.read().unwrap();
        let mut cache = loader.cached_config.write().unwrap();
        if let Some(config) = cache.as_mut() {
            config.poam_mappings = Some(poam);
        }
        Ok(())
    }

    /// Update cached SSP sections
    async fn update_cached_ssp(&self, ssp: SspSections) -> Result<()> {
        let loader = self.loader.read().unwrap();
        let mut cache = loader.cached_config.write().unwrap();
        if let Some(config) = cache.as_mut() {
            config.ssp_sections = Some(ssp);
        }
        Ok(())
    }

    /// Update cached control mappings
    async fn update_cached_controls(&self, controls: ControlMappings) -> Result<()> {
        let loader = self.loader.read().unwrap();
        let mut cache = loader.cached_config.write().unwrap();
        if let Some(config) = cache.as_mut() {
            config.controls = Some(controls);
        }
        Ok(())
    }

    /// Update cached document structures
    async fn update_cached_documents(&self, documents: DocumentStructures) -> Result<()> {
        let loader = self.loader.read().unwrap();
        let mut cache = loader.cached_config.write().unwrap();
        if let Some(config) = cache.as_mut() {
            config.documents = Some(documents);
        }
        Ok(())
    }
}
