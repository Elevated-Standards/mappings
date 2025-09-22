// Modified: 2025-01-22

//! Type definitions for mapping configuration loader
//!
//! This module contains all the core types, structs, and data structures
//! used throughout the mapping configuration loading system.

use fedramp_core::Result;
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::{Arc, RwLock};
use tokio::sync::mpsc;
use crate::mapping::config::{MappingConfiguration, LoadingMetrics};

/// Configuration loader for mapping files
#[derive(Debug)]
pub struct MappingConfigurationLoader {
    /// Base directory for relative paths
    pub base_dir: PathBuf,
    /// Loaded configuration cache
    pub cached_config: Arc<RwLock<Option<MappingConfiguration>>>,
    /// Hot-reload watcher
    pub watcher: Option<notify::RecommendedWatcher>,
    /// Reload notification channel
    pub reload_tx: Option<mpsc::UnboundedSender<PathBuf>>,
    /// Configuration backup for rollback
    pub config_backup: Arc<RwLock<Option<MappingConfiguration>>>,
    /// File modification times for change detection
    pub file_mtimes: Arc<RwLock<HashMap<PathBuf, std::time::SystemTime>>>,
    /// Loading performance metrics
    pub load_metrics: Arc<RwLock<LoadingMetrics>>,
}

/// Hot-reload event handler
#[derive(Debug)]
pub struct HotReloadHandler {
    /// Configuration loader reference
    pub loader: Arc<RwLock<MappingConfigurationLoader>>,
    /// Reload notification receiver
    pub reload_rx: mpsc::UnboundedReceiver<PathBuf>,
}

/// Configuration loading result with timing information
#[derive(Debug, Clone)]
pub struct LoadResult<T> {
    /// The loaded configuration data
    pub data: T,
    /// Time taken to load in milliseconds
    pub load_time_ms: u64,
    /// File path that was loaded
    pub file_path: PathBuf,
    /// File modification time
    pub modified_time: std::time::SystemTime,
}

/// Configuration file change event
#[derive(Debug, Clone)]
pub struct ConfigChangeEvent {
    /// Path of the changed file
    pub file_path: PathBuf,
    /// Type of change that occurred
    pub change_type: ChangeType,
    /// Timestamp of the change
    pub timestamp: std::time::SystemTime,
}

/// Type of configuration file change
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ChangeType {
    /// File was created
    Created,
    /// File was modified
    Modified,
    /// File was deleted
    Deleted,
    /// File was renamed
    Renamed,
}

/// Configuration validation result
#[derive(Debug, Clone)]
pub struct ValidationResult {
    /// Whether validation passed
    pub is_valid: bool,
    /// Validation warnings (non-fatal issues)
    pub warnings: Vec<String>,
    /// Validation errors (fatal issues)
    pub errors: Vec<String>,
    /// Validation performance metrics
    pub validation_time_ms: u64,
}

/// Configuration loading options
#[derive(Debug, Clone)]
pub struct LoadingOptions {
    /// Whether to enable parallel loading
    pub parallel_loading: bool,
    /// Whether to validate configuration after loading
    pub validate_after_load: bool,
    /// Whether to create backup before loading
    pub create_backup: bool,
    /// Maximum time to wait for file operations (ms)
    pub timeout_ms: u64,
    /// Whether to enable detailed performance metrics
    pub enable_metrics: bool,
}

impl Default for LoadingOptions {
    fn default() -> Self {
        Self {
            parallel_loading: true,
            validate_after_load: true,
            create_backup: true,
            timeout_ms: 5000,
            enable_metrics: true,
        }
    }
}

/// Configuration file metadata
#[derive(Debug, Clone)]
pub struct FileMetadata {
    /// File path
    pub path: PathBuf,
    /// File size in bytes
    pub size: u64,
    /// Last modification time
    pub modified: std::time::SystemTime,
    /// Whether file exists
    pub exists: bool,
    /// File checksum for integrity verification
    pub checksum: Option<String>,
}

/// Configuration loading statistics
#[derive(Debug, Clone)]
pub struct LoadingStats {
    /// Total number of files loaded
    pub files_loaded: usize,
    /// Total loading time in milliseconds
    pub total_time_ms: u64,
    /// Average loading time per file
    pub avg_time_per_file_ms: f64,
    /// Number of successful loads
    pub successful_loads: usize,
    /// Number of failed loads
    pub failed_loads: usize,
    /// Cache hit rate
    pub cache_hit_rate: f64,
}

impl Default for LoadingStats {
    fn default() -> Self {
        Self {
            files_loaded: 0,
            total_time_ms: 0,
            avg_time_per_file_ms: 0.0,
            successful_loads: 0,
            failed_loads: 0,
            cache_hit_rate: 0.0,
        }
    }
}

/// Configuration cache entry
#[derive(Debug, Clone)]
pub struct CacheEntry<T> {
    /// Cached data
    pub data: T,
    /// When the data was cached
    pub cached_at: std::time::SystemTime,
    /// File modification time when cached
    pub file_mtime: std::time::SystemTime,
    /// Cache hit count
    pub hit_count: u64,
}

/// Configuration loading error context
#[derive(Debug, Clone)]
pub struct LoadingError {
    /// Error message
    pub message: String,
    /// File path that caused the error
    pub file_path: Option<PathBuf>,
    /// Error category
    pub category: ErrorCategory,
    /// Underlying error cause
    pub cause: Option<String>,
}

/// Category of loading error
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ErrorCategory {
    /// File system error (file not found, permission denied, etc.)
    FileSystem,
    /// JSON parsing error
    JsonParsing,
    /// Configuration validation error
    Validation,
    /// Network error (for remote configurations)
    Network,
    /// Timeout error
    Timeout,
    /// Unknown error
    Unknown,
}

impl std::fmt::Display for LoadingError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}: {}", self.category, self.message)?;
        if let Some(path) = &self.file_path {
            write!(f, " (file: {})", path.display())?;
        }
        if let Some(cause) = &self.cause {
            write!(f, " - caused by: {}", cause)?;
        }
        Ok(())
    }
}

impl std::fmt::Display for ErrorCategory {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ErrorCategory::FileSystem => write!(f, "File System Error"),
            ErrorCategory::JsonParsing => write!(f, "JSON Parsing Error"),
            ErrorCategory::Validation => write!(f, "Validation Error"),
            ErrorCategory::Network => write!(f, "Network Error"),
            ErrorCategory::Timeout => write!(f, "Timeout Error"),
            ErrorCategory::Unknown => write!(f, "Unknown Error"),
        }
    }
}

impl std::error::Error for LoadingError {}
