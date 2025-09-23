// Modified: 2025-09-22

//! Validator type definitions and configuration structures
//!
//! This module contains the core validator structs and their configuration
//! for field-level and document-level validation.

use std::collections::HashMap;
use std::time::Duration;
use serde_json::Value;
use crate::{Result};
use crate::mapping::MappingConfiguration;
use super::super::types::ValidationStatus;

/// Column validator for validating individual columns
#[derive(Debug, Clone)]
pub struct ColumnValidator {
    /// Mapping configuration for validation rules
    pub(crate) mapping_config: MappingConfiguration,
    /// Minimum quality threshold for acceptance
    pub(crate) min_quality_threshold: f64,
    /// Performance target in milliseconds
    pub(crate) performance_target_ms: u64,
    /// Custom validation functions
    pub(crate) custom_validators: HashMap<String, fn(&[Value]) -> Result<(ValidationStatus, String)>>,
}

/// Document validator for comprehensive document validation
#[derive(Debug, Clone)]
pub struct DocumentValidator {
    /// Validation rules loaded from configuration
    pub(crate) rules: HashMap<String, Vec<super::super::rules::ValidationRule>>,
    /// Column validator for field-level validation
    pub(crate) column_validator: Option<ColumnValidator>,
    /// Minimum quality threshold for document acceptance
    pub(crate) min_quality_threshold: f64,
    /// Performance metrics
    pub(crate) performance_metrics: HashMap<String, Duration>,
}

/// Configuration for column validation
#[derive(Debug, Clone)]
pub struct ColumnValidationConfig {
    /// Minimum quality threshold for acceptance (0.0 - 1.0)
    pub min_quality_threshold: f64,
    /// Performance target in milliseconds
    pub performance_target_ms: u64,
    /// Whether to collect sample invalid values
    pub collect_samples: bool,
    /// Maximum number of sample invalid values to collect
    pub max_sample_size: usize,
    /// Minimum validity rate for different validation statuses
    pub validity_thresholds: ValidityThresholds,
}

/// Thresholds for determining validation status based on validity rates
#[derive(Debug, Clone)]
pub struct ValidityThresholds {
    /// Minimum rate for Valid status (default: 0.9)
    pub valid_threshold: f64,
    /// Minimum rate for Invalid status (default: 0.7)
    pub invalid_threshold: f64,
    /// Below this rate results in TypeMismatch status
    pub type_mismatch_threshold: f64,
}

/// Configuration for document validation
#[derive(Debug, Clone)]
pub struct DocumentValidationConfig {
    /// Minimum quality threshold for document acceptance (0.0 - 1.0)
    pub min_quality_threshold: f64,
    /// Whether to enable performance tracking
    pub track_performance: bool,
    /// Maximum validation time before timeout (in milliseconds)
    pub max_validation_time_ms: u64,
    /// Whether to validate all fields or stop at first failure
    pub fail_fast: bool,
}

/// Validation pattern configuration for different data types
#[derive(Debug, Clone)]
pub struct ValidationPatterns {
    /// Date format patterns
    pub date_patterns: Vec<String>,
    /// Email validation pattern
    pub email_pattern: String,
    /// URL validation patterns
    pub url_patterns: Vec<String>,
    /// IP address validation pattern
    pub ip_pattern: String,
    /// UUID validation pattern
    pub uuid_pattern: String,
}

/// Custom validator function type
pub type CustomValidatorFn = fn(&[Value]) -> Result<(ValidationStatus, String)>;

/// Document validation result containing all field validation results and quality metrics
#[derive(Debug, Clone)]
pub struct DocumentValidationResult {
    /// Whether the overall document validation passed
    pub passed: bool,
    /// Individual field validation results
    pub field_results: Vec<super::super::types::ColumnValidationResult>,
    /// Quality metrics for the document
    pub quality_metrics: super::super::types::QualityMetrics,
    /// Total validation time in milliseconds
    pub validation_time_ms: u64,
    /// Whether the document meets the quality threshold
    pub meets_quality_threshold: bool,
    /// Summary of the validation results
    pub summary: String,
}

/// Registry for custom validation functions
#[derive(Debug, Clone)]
pub struct ValidatorRegistry {
    /// Registered custom validators by name
    validators: HashMap<String, CustomValidatorFn>,
    /// Metadata about validators
    metadata: HashMap<String, ValidatorMetadata>,
}

/// Metadata about a custom validator
#[derive(Debug, Clone)]
pub struct ValidatorMetadata {
    /// Human-readable name
    pub name: String,
    /// Description of what the validator does
    pub description: String,
    /// Data types this validator can handle
    pub supported_types: Vec<String>,
    /// Performance characteristics
    pub performance_info: ValidatorPerformanceInfo,
}

/// Performance information for a validator
#[derive(Debug, Clone)]
pub struct ValidatorPerformanceInfo {
    /// Average execution time in microseconds
    pub avg_execution_time_us: u64,
    /// Memory usage characteristics
    pub memory_usage: MemoryUsage,
    /// Whether the validator is thread-safe
    pub thread_safe: bool,
}

/// Memory usage characteristics
#[derive(Debug, Clone)]
pub enum MemoryUsage {
    /// Constant memory usage
    Constant,
    /// Linear with input size
    Linear,
    /// Logarithmic with input size
    Logarithmic,
    /// Quadratic with input size (should be avoided)
    Quadratic,
}

impl Default for ColumnValidationConfig {
    fn default() -> Self {
        Self {
            min_quality_threshold: 0.8,
            performance_target_ms: 50,
            collect_samples: true,
            max_sample_size: 5,
            validity_thresholds: ValidityThresholds::default(),
        }
    }
}

impl Default for ValidityThresholds {
    fn default() -> Self {
        Self {
            valid_threshold: 0.9,
            invalid_threshold: 0.7,
            type_mismatch_threshold: 0.0,
        }
    }
}

impl Default for DocumentValidationConfig {
    fn default() -> Self {
        Self {
            min_quality_threshold: 0.8,
            track_performance: true,
            max_validation_time_ms: 5000,
            fail_fast: false,
        }
    }
}

impl Default for ValidationPatterns {
    fn default() -> Self {
        Self {
            date_patterns: vec![
                r"^\d{4}-\d{2}-\d{2}$".to_string(),           // YYYY-MM-DD
                r"^\d{2}/\d{2}/\d{4}$".to_string(),           // MM/DD/YYYY
                r"^\d{2}-\d{2}-\d{4}$".to_string(),           // MM-DD-YYYY
                r"^\d{1,2}/\d{1,2}/\d{4}$".to_string(),       // M/D/YYYY
            ],
            email_pattern: r"^[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}$".to_string(),
            url_patterns: vec![
                "http://".to_string(),
                "https://".to_string(),
                "ftp://".to_string(),
            ],
            ip_pattern: r"^(?:(?:25[0-5]|2[0-4][0-9]|[01]?[0-9][0-9]?)\.){3}(?:25[0-5]|2[0-4][0-9]|[01]?[0-9][0-9]?)$".to_string(),
            uuid_pattern: r"^[0-9a-fA-F]{8}-[0-9a-fA-F]{4}-[0-9a-fA-F]{4}-[0-9a-fA-F]{4}-[0-9a-fA-F]{12}$".to_string(),
        }
    }
}

impl ValidatorRegistry {
    /// Create a new validator registry
    pub fn new() -> Self {
        Self {
            validators: HashMap::new(),
            metadata: HashMap::new(),
        }
    }

    /// Register a custom validator
    pub fn register_validator(
        &mut self,
        name: String,
        validator: CustomValidatorFn,
        metadata: ValidatorMetadata,
    ) {
        self.validators.insert(name.clone(), validator);
        self.metadata.insert(name, metadata);
    }

    /// Get a validator by name
    pub fn get_validator(&self, name: &str) -> Option<&CustomValidatorFn> {
        self.validators.get(name)
    }

    /// Get validator metadata
    pub fn get_metadata(&self, name: &str) -> Option<&ValidatorMetadata> {
        self.metadata.get(name)
    }

    /// List all registered validators
    pub fn list_validators(&self) -> Vec<&str> {
        self.validators.keys().map(|s| s.as_str()).collect()
    }

    /// Remove a validator
    pub fn remove_validator(&mut self, name: &str) -> bool {
        let removed_validator = self.validators.remove(name).is_some();
        let removed_metadata = self.metadata.remove(name).is_some();
        removed_validator && removed_metadata
    }

    /// Check if a validator is registered
    pub fn has_validator(&self, name: &str) -> bool {
        self.validators.contains_key(name)
    }

    /// Get the number of registered validators
    pub fn count(&self) -> usize {
        self.validators.len()
    }

    /// Clear all validators
    pub fn clear(&mut self) {
        self.validators.clear();
        self.metadata.clear();
    }
}

impl Default for ValidatorRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl ValidatorMetadata {
    /// Create new validator metadata
    pub fn new(
        name: String,
        description: String,
        supported_types: Vec<String>,
    ) -> Self {
        Self {
            name,
            description,
            supported_types,
            performance_info: ValidatorPerformanceInfo::default(),
        }
    }

    /// Set performance information
    pub fn with_performance_info(mut self, performance_info: ValidatorPerformanceInfo) -> Self {
        self.performance_info = performance_info;
        self
    }
}

impl Default for ValidatorPerformanceInfo {
    fn default() -> Self {
        Self {
            avg_execution_time_us: 100,
            memory_usage: MemoryUsage::Linear,
            thread_safe: true,
        }
    }
}

impl ValidatorPerformanceInfo {
    /// Create new performance info
    pub fn new(
        avg_execution_time_us: u64,
        memory_usage: MemoryUsage,
        thread_safe: bool,
    ) -> Self {
        Self {
            avg_execution_time_us,
            memory_usage,
            thread_safe,
        }
    }
}
