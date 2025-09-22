// Modified: 2025-01-22

//! Caching and backup functionality for configuration loading
//!
//! This module provides configuration caching, backup creation,
//! and rollback capabilities for the mapping configuration loader.

use super::types::*;
use fedramp_core::{Result, Error};
use std::collections::HashMap;
use std::path::PathBuf;
use tracing::{debug, info, warn};
use crate::mapping::config::MappingConfiguration;

impl MappingConfigurationLoader {
    /// Create backup of current configuration for rollback
    pub async fn create_configuration_backup(&self) -> Result<()> {
        let current_config = {
            let cache = self.cached_config.read().unwrap();
            cache.clone()
        };

        if let Some(config) = current_config {
            let mut backup = self.config_backup.write().unwrap();
            *backup = Some(config);
            debug!("Created configuration backup");
        } else {
            debug!("No current configuration to backup");
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

    /// Update file modification times cache
    pub async fn update_file_mtimes(&self) -> Result<()> {
        let file_paths = vec![
            self.base_dir.join("mappings").join("inventory_mappings.json"),
            self.base_dir.join("mappings").join("poam_mappings.json"),
            self.base_dir.join("mappings").join("ssp_sections.json"),
            self.base_dir.join("schema").join("_controls.json"),
            self.base_dir.join("schema").join("_document.json"),
        ];

        let mut mtimes = self.file_mtimes.write().unwrap();

        for path in file_paths {
            if path.exists() {
                match tokio::fs::metadata(&path).await {
                    Ok(metadata) => {
                        match metadata.modified() {
                            Ok(mtime) => {
                                mtimes.insert(path.clone(), mtime);
                                debug!("Updated mtime for {}", path.display());
                            }
                            Err(e) => {
                                warn!("Failed to get modification time for {}: {}", path.display(), e);
                            }
                        }
                    }
                    Err(e) => {
                        warn!("Failed to read metadata for {}: {}", path.display(), e);
                    }
                }
            }
        }

        Ok(())
    }

    /// Get file modification times
    pub fn get_file_mtimes(&self) -> HashMap<PathBuf, std::time::SystemTime> {
        let mtimes = self.file_mtimes.read().unwrap();
        mtimes.clone()
    }

    /// Clear file modification times cache
    pub fn clear_file_mtimes(&self) {
        let mut mtimes = self.file_mtimes.write().unwrap();
        mtimes.clear();
        debug!("Cleared file modification times cache");
    }

    /// Check if backup configuration exists
    pub fn has_backup(&self) -> bool {
        let backup = self.config_backup.read().unwrap();
        backup.is_some()
    }

    /// Get backup configuration
    pub fn get_backup_configuration(&self) -> Option<MappingConfiguration> {
        let backup = self.config_backup.read().unwrap();
        backup.clone()
    }

    /// Clear backup configuration
    pub fn clear_backup(&self) {
        let mut backup = self.config_backup.write().unwrap();
        *backup = None;
        debug!("Cleared backup configuration");
    }

    /// Get cache statistics
    pub fn get_cache_stats(&self) -> CacheStats {
        let cache = self.cached_config.read().unwrap();
        let backup = self.config_backup.read().unwrap();
        let mtimes = self.file_mtimes.read().unwrap();
        let metrics = self.load_metrics.read().unwrap();

        CacheStats {
            has_cached_config: cache.is_some(),
            has_backup_config: backup.is_some(),
            tracked_files: mtimes.len(),
            total_loads: metrics.successful_loads + metrics.failed_loads,
            successful_loads: metrics.successful_loads,
            failed_loads: metrics.failed_loads,
            last_load_time: metrics.last_load_time,
            total_load_time_ms: metrics.total_load_time_ms,
        }
    }

    /// Invalidate cache for specific file
    pub async fn invalidate_file_cache(&self, file_path: &str) -> Result<()> {
        let mut mtimes = self.file_mtimes.write().unwrap();
        
        // Remove entries that match the file path
        let keys_to_remove: Vec<PathBuf> = mtimes
            .keys()
            .filter(|path| {
                path.file_name()
                    .and_then(|name| name.to_str())
                    .map_or(false, |name| name == file_path)
            })
            .cloned()
            .collect();

        for key in keys_to_remove {
            mtimes.remove(&key);
            debug!("Invalidated cache for {}", key.display());
        }

        Ok(())
    }

    /// Preload all configurations into cache
    pub async fn preload_cache(&mut self) -> Result<()> {
        info!("Preloading configuration cache");
        
        let start_time = std::time::Instant::now();
        let config = self.load_all_configurations_optimized().await?;
        let load_time = start_time.elapsed().as_millis() as u64;

        // Update file modification times
        self.update_file_mtimes().await?;

        info!("Successfully preloaded configuration cache in {}ms", load_time);
        Ok(())
    }

    /// Refresh cache if needed
    pub async fn refresh_cache_if_needed(&mut self) -> Result<bool> {
        if self.has_configuration_changed().await? {
            info!("Configuration files changed, refreshing cache");
            self.load_all_configurations_optimized().await?;
            self.update_file_mtimes().await?;
            Ok(true)
        } else {
            debug!("Cache is up to date");
            Ok(false)
        }
    }
}

/// Cache statistics
#[derive(Debug, Clone)]
pub struct CacheStats {
    /// Whether configuration is cached
    pub has_cached_config: bool,
    /// Whether backup configuration exists
    pub has_backup_config: bool,
    /// Number of tracked files
    pub tracked_files: usize,
    /// Total number of load attempts
    pub total_loads: u64,
    /// Number of successful loads
    pub successful_loads: u64,
    /// Number of failed loads
    pub failed_loads: u64,
    /// Last load time
    pub last_load_time: Option<std::time::SystemTime>,
    /// Total loading time in milliseconds
    pub total_load_time_ms: u64,
}

impl CacheStats {
    /// Calculate success rate
    pub fn success_rate(&self) -> f64 {
        if self.total_loads == 0 {
            0.0
        } else {
            self.successful_loads as f64 / self.total_loads as f64
        }
    }

    /// Calculate average load time
    pub fn average_load_time_ms(&self) -> f64 {
        if self.total_loads == 0 {
            0.0
        } else {
            self.total_load_time_ms as f64 / self.total_loads as f64
        }
    }

    /// Check if cache is healthy
    pub fn is_healthy(&self) -> bool {
        self.has_cached_config && self.success_rate() > 0.8
    }
}

impl std::fmt::Display for CacheStats {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "CacheStats {{ cached: {}, backup: {}, files: {}, loads: {}/{}, success_rate: {:.1}%, avg_time: {:.1}ms }}",
               self.has_cached_config,
               self.has_backup_config,
               self.tracked_files,
               self.successful_loads,
               self.total_loads,
               self.success_rate() * 100.0,
               self.average_load_time_ms())
    }
}
