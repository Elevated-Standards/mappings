// Modified: 2025-01-20

//! Error types for document parsing operations
//!
//! Provides comprehensive error handling for all document parsing scenarios
//! with detailed error messages and proper error chaining.

use thiserror::Error;

/// Document parser specific errors
#[derive(Debug, Error)]
pub enum DocumentParserError {
    #[error("Unsupported document format: {format}")]
    UnsupportedFormat { format: String },

    #[error("File not found: {path}")]
    FileNotFound { path: String },

    #[error("File too large: {size} bytes (max: {max_size} bytes)")]
    FileTooLarge { size: u64, max_size: u64 },

    #[error("Invalid file content: {message}")]
    InvalidContent { message: String },

    #[error("Parsing failed: {message}")]
    ParsingFailed { message: String },

    #[error("Validation failed: {errors:?}")]
    ValidationFailed { errors: Vec<String> },

    #[error("IO error: {source}")]
    Io {
        #[from]
        source: std::io::Error,
    },

    #[error("JSON error: {source}")]
    Json {
        #[from]
        source: serde_json::Error,
    },

    #[error("UTF-8 conversion error: {source}")]
    Utf8 {
        #[from]
        source: std::string::FromUtf8Error,
    },

    #[error("Configuration loading failed: {message}")]
    ConfigurationLoadingFailed { message: String },

    #[error("Configuration validation failed: {errors:?}")]
    ConfigurationValidationFailed { errors: Vec<String> },

    #[error("Mapping configuration not found: {config_type}")]
    MappingConfigurationNotFound { config_type: String },

    #[error("Invalid mapping configuration: {message}")]
    InvalidMappingConfiguration { message: String },

    #[error("Configuration conflict detected: {conflicts:?}")]
    ConfigurationConflict { conflicts: Vec<String> },

    #[error("Schema validation failed: {field} - {message}")]
    SchemaValidationFailed { field: String, message: String },

    #[error("Required field missing: {field}")]
    RequiredFieldMissing { field: String },

    #[error("Invalid field value: {field} = '{value}' - {reason}")]
    InvalidFieldValue {
        field: String,
        value: String,
        reason: String,
    },

    #[error("Regex compilation failed: {pattern} - {source}")]
    RegexCompilationFailed {
        pattern: String,
        #[source]
        source: regex::Error,
    },

    #[error("Hot-reload failed: {message}")]
    HotReloadFailed { message: String },

    #[error("Cache operation failed: {operation} - {message}")]
    CacheOperationFailed {
        operation: String,
        message: String,
    },
}

impl DocumentParserError {
    /// Create a new unsupported format error
    #[must_use]
    pub fn unsupported_format(format: impl Into<String>) -> Self {
        Self::UnsupportedFormat {
            format: format.into(),
        }
    }

    /// Create a new file not found error
    #[must_use]
    pub fn file_not_found(path: impl Into<String>) -> Self {
        Self::FileNotFound { path: path.into() }
    }

    /// Create a new file too large error
    #[must_use]
    pub fn file_too_large(size: u64, max_size: u64) -> Self {
        Self::FileTooLarge { size, max_size }
    }

    /// Create a new invalid content error
    #[must_use]
    pub fn invalid_content(message: impl Into<String>) -> Self {
        Self::InvalidContent {
            message: message.into(),
        }
    }

    /// Create a new parsing failed error
    #[must_use]
    pub fn parsing_failed(message: impl Into<String>) -> Self {
        Self::ParsingFailed {
            message: message.into(),
        }
    }

    /// Create a new validation failed error
    #[must_use]
    pub fn validation_failed(errors: Vec<String>) -> Self {
        Self::ValidationFailed { errors }
    }

    /// Create a new configuration loading failed error
    #[must_use]
    pub fn configuration_loading_failed(message: impl Into<String>) -> Self {
        Self::ConfigurationLoadingFailed {
            message: message.into(),
        }
    }

    /// Create a new configuration validation failed error
    #[must_use]
    pub fn configuration_validation_failed(errors: Vec<String>) -> Self {
        Self::ConfigurationValidationFailed { errors }
    }

    /// Create a new mapping configuration not found error
    #[must_use]
    pub fn mapping_configuration_not_found(config_type: impl Into<String>) -> Self {
        Self::MappingConfigurationNotFound {
            config_type: config_type.into(),
        }
    }

    /// Create a new invalid mapping configuration error
    #[must_use]
    pub fn invalid_mapping_configuration(message: impl Into<String>) -> Self {
        Self::InvalidMappingConfiguration {
            message: message.into(),
        }
    }

    /// Create a new configuration conflict error
    #[must_use]
    pub fn configuration_conflict(conflicts: Vec<String>) -> Self {
        Self::ConfigurationConflict { conflicts }
    }

    /// Create a new schema validation failed error
    #[must_use]
    pub fn schema_validation_failed(field: impl Into<String>, message: impl Into<String>) -> Self {
        Self::SchemaValidationFailed {
            field: field.into(),
            message: message.into(),
        }
    }

    /// Create a new required field missing error
    #[must_use]
    pub fn required_field_missing(field: impl Into<String>) -> Self {
        Self::RequiredFieldMissing {
            field: field.into(),
        }
    }

    /// Create a new invalid field value error
    #[must_use]
    pub fn invalid_field_value(
        field: impl Into<String>,
        value: impl Into<String>,
        reason: impl Into<String>,
    ) -> Self {
        Self::InvalidFieldValue {
            field: field.into(),
            value: value.into(),
            reason: reason.into(),
        }
    }

    /// Create a new regex compilation failed error
    #[must_use]
    pub fn regex_compilation_failed(pattern: impl Into<String>, source: regex::Error) -> Self {
        Self::RegexCompilationFailed {
            pattern: pattern.into(),
            source,
        }
    }

    /// Create a new hot-reload failed error
    #[must_use]
    pub fn hot_reload_failed(message: impl Into<String>) -> Self {
        Self::HotReloadFailed {
            message: message.into(),
        }
    }

    /// Create a new cache operation failed error
    #[must_use]
    pub fn cache_operation_failed(
        operation: impl Into<String>,
        message: impl Into<String>,
    ) -> Self {
        Self::CacheOperationFailed {
            operation: operation.into(),
            message: message.into(),
        }
    }
}
