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
}
