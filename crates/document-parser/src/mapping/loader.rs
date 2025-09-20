//! Configuration loader for mapping files
//!
//! This module provides functionality to load mapping configurations from JSON files
//! with support for hot-reload, validation, and performance optimization.

use fedramp_core::{Result, Error};
use serde::Deserialize;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::{Arc, RwLock};
use std::time::Duration;
use tokio::fs;
use tokio::sync::mpsc;
use tracing::{debug, info, warn, error};
use regex::Regex;
use notify::{Watcher, RecursiveMode, Event, EventKind};
use crate::error::DocumentParserError;
use crate::mapping::config::{MappingConfiguration, LoadingMetrics};
use crate::mapping::inventory::{InventoryMappings, ValidationRules};
use crate::mapping::poam::{PoamMappings, PoamValidationRules};
use crate::mapping::ssp::SspSections;
use crate::mapping::control_document::{ControlMappings, DocumentStructures};
use crate::mapping::validation;

/// Configuration loader for mapping files
#[derive(Debug)]
pub struct MappingConfigurationLoader {
    /// Base directory for relative paths
    base_dir: PathBuf,
    /// Loaded configuration cache
    cached_config: Arc<RwLock<Option<MappingConfiguration>>>,
    /// Hot-reload watcher
    watcher: Option<notify::RecommendedWatcher>,
    /// Reload notification channel
    reload_tx: Option<mpsc::UnboundedSender<PathBuf>>,
    /// Configuration backup for rollback
    config_backup: Arc<RwLock<Option<MappingConfiguration>>>,
    /// File modification times for change detection
    file_mtimes: Arc<RwLock<HashMap<PathBuf, std::time::SystemTime>>>,
    /// Loading performance metrics
    load_metrics: Arc<RwLock<LoadingMetrics>>,
}

/// Hot-reload event handler
pub struct HotReloadHandler {
    /// Configuration loader reference
    loader: Arc<RwLock<MappingConfigurationLoader>>,
    /// Reload notification receiver
    reload_rx: mpsc::UnboundedReceiver<PathBuf>,
}

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

    /// Create a new configuration loader with hot-reload support
    pub fn with_hot_reload<P: AsRef<Path>>(base_dir: P) -> Result<(Self, HotReloadHandler)> {
        let (reload_tx, reload_rx) = mpsc::unbounded_channel();

        let mut loader = Self {
            base_dir: base_dir.as_ref().to_path_buf(),
            cached_config: Arc::new(RwLock::new(None)),
            watcher: None,
            reload_tx: Some(reload_tx),
            config_backup: Arc::new(RwLock::new(None)),
            file_mtimes: Arc::new(RwLock::new(HashMap::new())),
            load_metrics: Arc::new(RwLock::new(LoadingMetrics::default())),
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
            Err(validation_errors) => {
                warn!("Configuration validation failed, attempting rollback: {:?}", validation_errors);
                self.rollback_configuration().await?;
                // Update metrics with failure
                self.update_load_metrics(total_time, file_times, false).await;
                return Err(Error::document_parsing(format!(
                    "Configuration validation failed: {:?}",
                    validation_errors
                )));
            }
        }

        // Update metrics with success (critical configs loaded and validation passed)
        let success = has_inventory || has_poam || has_ssp; // At least one critical config loaded
        debug!("Updating metrics with success={}", success);
        self.update_load_metrics(total_time, file_times, success).await;

        // Atomically update cached configuration
        {
            let mut cache = self.cached_config.write().unwrap();
            *cache = Some(config.clone());
        }

        info!("Successfully loaded mapping configurations in {}ms", total_time);

        // Ensure we meet the sub-100ms requirement for performance
        if total_time > 100 {
            warn!("Configuration loading took {}ms, exceeding 100ms target", total_time);
        }

        Ok(config)
    }

    /// Load inventory mappings from JSON file
    pub async fn load_inventory_mappings(&self) -> Result<InventoryMappings> {
        let path = self.base_dir.join("mappings").join("inventory_mappings.json");
        let raw_json: serde_json::Value = self.load_json_file(&path).await?;

        // Check if it's wrapped in inventory_mappings or direct
        if let Some(inventory_mappings) = raw_json.get("inventory_mappings") {
            serde_json::from_value(inventory_mappings.clone())
                .map_err(|e| Error::document_parsing(format!("Failed to parse inventory mappings: {}", e)))
        } else {
            // Try to parse directly
            serde_json::from_value(raw_json)
                .map_err(|e| Error::document_parsing(format!("Failed to parse inventory mappings: {}", e)))
        }
    }

    /// Load POA&M mappings from JSON file
    pub async fn load_poam_mappings(&self) -> Result<PoamMappings> {
        let path = self.base_dir.join("mappings").join("poam_mappings.json");
        let raw_json: serde_json::Value = self.load_json_file(&path).await?;

        // Extract the poam_mappings section from the root object
        if let Some(poam_mappings) = raw_json.get("poam_mappings") {
            serde_json::from_value(poam_mappings.clone())
                .map_err(|e| Error::document_parsing(format!("Failed to parse POA&M mappings: {}", e)))
        } else {
            Err(Error::document_parsing("POA&M mappings section not found in JSON"))
        }
    }

    /// Load SSP sections from JSON file
    pub async fn load_ssp_sections(&self) -> Result<SspSections> {
        let path = self.base_dir.join("mappings").join("ssp_sections.json");
        self.load_json_file(&path).await
    }

    /// Load control mappings from schema file
    pub async fn load_control_mappings(&self) -> Result<ControlMappings> {
        let path = self.base_dir.join("schema").join("_controls.json");
        self.load_json_file(&path).await
    }

    /// Load document structures from schema file
    pub async fn load_document_structures(&self) -> Result<DocumentStructures> {
        let path = self.base_dir.join("schema").join("_document.json");
        self.load_json_file(&path).await
    }

    /// Generic JSON file loader with enhanced error handling and change detection
    async fn load_json_file<T>(&self, path: &Path) -> Result<T>
    where
        T: for<'de> Deserialize<'de>,
    {
        debug!("Loading JSON file: {}", path.display());

        // Check if file exists
        if !path.exists() {
            return Err(Error::document_parsing(format!(
                "Configuration file not found: {}. Please ensure the file exists and is accessible.",
                path.display()
            )));
        }

        // Get file metadata for change detection
        let metadata = fs::metadata(path).await.map_err(|e| {
            Error::document_parsing(format!(
                "Failed to read file metadata for {}: {}",
                path.display(),
                e
            ))
        })?;

        let modified_time = metadata.modified().map_err(|e| {
            Error::document_parsing(format!(
                "Failed to get modification time for {}: {}",
                path.display(),
                e
            ))
        })?;

        // Update file modification time tracking
        {
            let mut mtimes = self.file_mtimes.write().unwrap();
            mtimes.insert(path.to_path_buf(), modified_time);
        }

        // Check file size for reasonable limits (prevent loading extremely large files)
        let file_size = metadata.len();
        if file_size > 10 * 1024 * 1024 { // 10MB limit
            return Err(Error::document_parsing(format!(
                "Configuration file {} is too large ({} bytes). Maximum size is 10MB.",
                path.display(),
                file_size
            )));
        }

        // Read file contents with better error context
        let contents = fs::read_to_string(path).await.map_err(|e| {
            Error::document_parsing(format!(
                "Failed to read file {}: {}. Please check file permissions and encoding.",
                path.display(),
                e
            ))
        })?;

        // Validate JSON is not empty
        if contents.trim().is_empty() {
            return Err(Error::document_parsing(format!(
                "Configuration file {} is empty",
                path.display()
            )));
        }

        // Parse JSON with detailed error information
        serde_json::from_str(&contents).map_err(|e| {
            let line_col = format!(" at line {}, column {}", e.line(), e.column());

            Error::document_parsing(format!(
                "Failed to parse JSON from {}{}: {}. Please check the JSON syntax and structure.",
                path.display(),
                line_col,
                e
            ))
        })
    }

    /// Load configuration from custom file path
    pub async fn load_from_path<T, P>(&self, path: P) -> Result<T>
    where
        T: for<'de> Deserialize<'de>,
        P: AsRef<Path>,
    {
        let full_path = if path.as_ref().is_absolute() {
            path.as_ref().to_path_buf()
        } else {
            self.base_dir.join(path.as_ref())
        };

        self.load_json_file(&full_path).await
    }

    /// Get cached configuration if available
    pub fn get_cached_configuration(&self) -> Option<MappingConfiguration> {
        let cache = self.cached_config.read().unwrap();
        cache.clone()
    }

    /// Clear cached configuration
    pub fn clear_cache(&self) {
        let mut cache = self.cached_config.write().unwrap();
        *cache = None;
    }

    /// Create backup of current configuration for rollback capability
    async fn create_configuration_backup(&self) -> Result<()> {
        let current_config = {
            let cache = self.cached_config.read().unwrap();
            cache.clone()
        };

        if let Some(config) = current_config {
            let mut backup = self.config_backup.write().unwrap();
            *backup = Some(config);
            debug!("Created configuration backup");
        }

        Ok(())
    }

    /// Rollback to previous configuration
    pub async fn rollback_configuration(&self) -> Result<()> {
        let backup_config = {
            let backup = self.config_backup.read().unwrap();
            backup.clone()
        };

        if let Some(config) = backup_config {
            let mut cache = self.cached_config.write().unwrap();
            *cache = Some(config);
            info!("Successfully rolled back to previous configuration");
        } else {
            warn!("No backup configuration available for rollback");
        }

        Ok(())
    }

    /// Update loading performance metrics
    async fn update_load_metrics(&self, total_time: u64, file_times: HashMap<String, u64>, success: bool) {
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
    pub fn get_load_metrics(&self) -> LoadingMetrics {
        let metrics = self.load_metrics.read().unwrap();
        metrics.clone()
    }

    /// Load inventory mappings with timing
    async fn load_inventory_mappings_with_timing(&self) -> Result<(InventoryMappings, u64)> {
        let start = std::time::Instant::now();
        let result = self.load_inventory_mappings().await?;
        let elapsed = start.elapsed().as_millis() as u64;
        Ok((result, elapsed))
    }

    /// Load POA&M mappings with timing
    async fn load_poam_mappings_with_timing(&self) -> Result<(PoamMappings, u64)> {
        let start = std::time::Instant::now();
        let result = self.load_poam_mappings().await?;
        let elapsed = start.elapsed().as_millis() as u64;
        Ok((result, elapsed))
    }

    /// Load SSP sections with timing
    async fn load_ssp_sections_with_timing(&self) -> Result<(SspSections, u64)> {
        let start = std::time::Instant::now();
        let result = self.load_ssp_sections().await?;
        let elapsed = start.elapsed().as_millis() as u64;
        Ok((result, elapsed))
    }

    /// Load control mappings with timing
    async fn load_control_mappings_with_timing(&self) -> Result<(ControlMappings, u64)> {
        let start = std::time::Instant::now();
        let result = self.load_control_mappings().await?;
        let elapsed = start.elapsed().as_millis() as u64;
        Ok((result, elapsed))
    }

    /// Load document structures with timing
    async fn load_document_structures_with_timing(&self) -> Result<(DocumentStructures, u64)> {
        let start = std::time::Instant::now();
        let result = self.load_document_structures().await?;
        let elapsed = start.elapsed().as_millis() as u64;
        Ok((result, elapsed))
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

            let metadata = fs::metadata(&path).await.map_err(|e| {
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

    /// Validate loaded configuration
    pub fn validate_configuration(&self, config: &MappingConfiguration) -> Result<Vec<String>> {
        let mut warnings = Vec::new();

        // Validate inventory mappings
        if let Some(inventory) = &config.inventory_mappings {
            warnings.extend(validation::validate_inventory_mappings(inventory)?);
        }

        // Validate POA&M mappings
        if let Some(poam) = &config.poam_mappings {
            warnings.extend(validation::validate_poam_mappings(poam)?);
        }

        // Validate SSP sections
        if let Some(ssp) = &config.ssp_sections {
            warnings.extend(validation::validate_ssp_sections(ssp)?);
        }

        // Validate control mappings
        if let Some(controls) = &config.controls {
            warnings.extend(validation::validate_control_mappings(controls)?);
        }

        // Validate document structures
        if let Some(documents) = &config.documents {
            warnings.extend(validation::validate_document_structures(documents)?);
        }

        // Check for configuration conflicts
        warnings.extend(validation::detect_configuration_conflicts(config)?);

        Ok(warnings)
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
