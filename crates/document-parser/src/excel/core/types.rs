//! Modified: 2025-09-23

//! Core Excel parser types and configuration
//!
//! This module contains the type definitions for the Excel parser including
//! the main parser struct, configuration options, and related types.

use crate::excel::types::*;
use crate::excel::validation::ExcelValidator;

/// Main Excel parser implementation
#[derive(Debug, Clone)]
pub struct ExcelParser {
    /// Maximum file size to process (in bytes)
    pub(crate) max_file_size: usize,
    /// Whether to automatically detect headers
    pub(crate) auto_detect_headers: bool,
    /// Maximum number of rows to process per worksheet
    pub(crate) max_rows: Option<usize>,
    /// Validation configuration
    pub(crate) validation_config: ValidationConfig,
}

impl ExcelParser {
    /// Create a new Excel parser with default settings
    ///
    /// # Examples
    ///
    /// ```rust
    /// use document_parser::excel::ExcelParser;
    ///
    /// let parser = ExcelParser::new();
    /// ```
    #[must_use]
    pub fn new() -> Self {
        Self {
            max_file_size: 100 * 1024 * 1024, // 100MB
            auto_detect_headers: true,
            max_rows: None,
            validation_config: ValidationConfig::default(),
        }
    }

    /// Create a new Excel parser with custom configuration
    ///
    /// # Arguments
    ///
    /// * `max_file_size` - Maximum file size in bytes
    /// * `auto_detect_headers` - Whether to automatically detect headers
    /// * `max_rows` - Maximum number of rows to process per worksheet
    ///
    /// # Examples
    ///
    /// ```rust
    /// use document_parser::excel::ExcelParser;
    ///
    /// let parser = ExcelParser::with_config(50 * 1024 * 1024, true, Some(10000));
    /// ```
    #[must_use]
    pub fn with_config(
        max_file_size: usize,
        auto_detect_headers: bool,
        max_rows: Option<usize>,
    ) -> Self {
        Self {
            max_file_size,
            auto_detect_headers,
            max_rows,
            validation_config: ValidationConfig::default(),
        }
    }

    /// Create a new Excel parser with custom validation configuration
    ///
    /// # Arguments
    ///
    /// * `max_file_size` - Maximum file size in bytes
    /// * `auto_detect_headers` - Whether to automatically detect headers
    /// * `max_rows` - Maximum number of rows to process per worksheet
    /// * `validation_config` - Validation and sanitization configuration
    ///
    /// # Examples
    ///
    /// ```rust
    /// use document_parser::excel::{ExcelParser, ValidationConfig};
    ///
    /// let config = ValidationConfig::strict();
    /// let parser = ExcelParser::with_validation_config(50 * 1024 * 1024, true, Some(10000), config);
    /// ```
    #[must_use]
    pub fn with_validation_config(
        max_file_size: usize,
        auto_detect_headers: bool,
        max_rows: Option<usize>,
        validation_config: ValidationConfig,
    ) -> Self {
        Self {
            max_file_size,
            auto_detect_headers,
            max_rows,
            validation_config,
        }
    }

    /// Get the maximum file size setting
    pub fn max_file_size(&self) -> usize {
        self.max_file_size
    }

    /// Set the maximum file size
    pub fn set_max_file_size(&mut self, max_file_size: usize) {
        self.max_file_size = max_file_size;
    }

    /// Get the auto-detect headers setting
    pub fn auto_detect_headers(&self) -> bool {
        self.auto_detect_headers
    }

    /// Set the auto-detect headers setting
    pub fn set_auto_detect_headers(&mut self, auto_detect_headers: bool) {
        self.auto_detect_headers = auto_detect_headers;
    }

    /// Get the maximum rows setting
    pub fn max_rows(&self) -> Option<usize> {
        self.max_rows
    }

    /// Set the maximum rows setting
    pub fn set_max_rows(&mut self, max_rows: Option<usize>) {
        self.max_rows = max_rows;
    }

    /// Get the validation configuration
    pub fn validation_config(&self) -> &ValidationConfig {
        &self.validation_config
    }

    /// Set the validation configuration
    pub fn set_validation_config(&mut self, validation_config: ValidationConfig) {
        self.validation_config = validation_config;
    }

    /// Update validation configuration with a closure
    pub fn update_validation_config<F>(&mut self, f: F)
    where
        F: FnOnce(&mut ValidationConfig),
    {
        f(&mut self.validation_config);
    }

    /// Create a builder for configuring the parser
    pub fn builder() -> ExcelParserBuilder {
        ExcelParserBuilder::new()
    }
}

impl Default for ExcelParser {
    fn default() -> Self {
        Self::new()
    }
}

/// Builder for configuring ExcelParser instances
#[derive(Debug, Clone)]
pub struct ExcelParserBuilder {
    max_file_size: usize,
    auto_detect_headers: bool,
    max_rows: Option<usize>,
    validation_config: ValidationConfig,
}

impl ExcelParserBuilder {
    /// Create a new builder with default settings
    pub fn new() -> Self {
        Self {
            max_file_size: 100 * 1024 * 1024, // 100MB
            auto_detect_headers: true,
            max_rows: None,
            validation_config: ValidationConfig::default(),
        }
    }

    /// Set the maximum file size
    pub fn max_file_size(mut self, max_file_size: usize) -> Self {
        self.max_file_size = max_file_size;
        self
    }

    /// Set the auto-detect headers setting
    pub fn auto_detect_headers(mut self, auto_detect_headers: bool) -> Self {
        self.auto_detect_headers = auto_detect_headers;
        self
    }

    /// Set the maximum rows setting
    pub fn max_rows(mut self, max_rows: Option<usize>) -> Self {
        self.max_rows = max_rows;
        self
    }

    /// Set the validation configuration
    pub fn validation_config(mut self, validation_config: ValidationConfig) -> Self {
        self.validation_config = validation_config;
        self
    }

    /// Build the ExcelParser instance
    pub fn build(self) -> ExcelParser {
        ExcelParser {
            max_file_size: self.max_file_size,
            auto_detect_headers: self.auto_detect_headers,
            max_rows: self.max_rows,
            validation_config: self.validation_config,
        }
    }
}

impl Default for ExcelParserBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// Configuration presets for common use cases
impl ExcelParser {
    /// Create a parser optimized for large files
    pub fn for_large_files() -> Self {
        Self::builder()
            .max_file_size(500 * 1024 * 1024) // 500MB
            .max_rows(Some(100_000)) // Limit to 100k rows
            .validation_config(ValidationConfig::lenient())
            .build()
    }

    /// Create a parser optimized for strict validation
    pub fn for_strict_validation() -> Self {
        Self::builder()
            .max_file_size(50 * 1024 * 1024) // 50MB
            .validation_config(ValidationConfig::strict())
            .build()
    }

    /// Create a parser optimized for performance
    pub fn for_performance() -> Self {
        Self::builder()
            .max_file_size(200 * 1024 * 1024) // 200MB
            .max_rows(Some(50_000)) // Limit to 50k rows
            .auto_detect_headers(false) // Skip header detection
            .validation_config(ValidationConfig::minimal())
            .build()
    }

    /// Create a parser for development/testing
    pub fn for_development() -> Self {
        Self::builder()
            .max_file_size(10 * 1024 * 1024) // 10MB
            .max_rows(Some(1_000)) // Limit to 1k rows
            .validation_config(ValidationConfig::debug())
            .build()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_excel_parser_creation() {
        let parser = ExcelParser::new();
        assert_eq!(parser.max_file_size(), 100 * 1024 * 1024);
        assert!(parser.auto_detect_headers());
        assert_eq!(parser.max_rows(), None);
    }

    #[test]
    fn test_excel_parser_with_config() {
        let parser = ExcelParser::with_config(50 * 1024 * 1024, false, Some(1000));
        assert_eq!(parser.max_file_size(), 50 * 1024 * 1024);
        assert!(!parser.auto_detect_headers());
        assert_eq!(parser.max_rows(), Some(1000));
    }

    #[test]
    fn test_excel_parser_builder() {
        let parser = ExcelParser::builder()
            .max_file_size(25 * 1024 * 1024)
            .auto_detect_headers(false)
            .max_rows(Some(500))
            .build();
        
        assert_eq!(parser.max_file_size(), 25 * 1024 * 1024);
        assert!(!parser.auto_detect_headers());
        assert_eq!(parser.max_rows(), Some(500));
    }

    #[test]
    fn test_excel_parser_presets() {
        let large_files_parser = ExcelParser::for_large_files();
        assert_eq!(large_files_parser.max_file_size(), 500 * 1024 * 1024);
        assert_eq!(large_files_parser.max_rows(), Some(100_000));

        let strict_parser = ExcelParser::for_strict_validation();
        assert_eq!(strict_parser.max_file_size(), 50 * 1024 * 1024);

        let performance_parser = ExcelParser::for_performance();
        assert_eq!(performance_parser.max_file_size(), 200 * 1024 * 1024);
        assert!(!performance_parser.auto_detect_headers());

        let dev_parser = ExcelParser::for_development();
        assert_eq!(dev_parser.max_file_size(), 10 * 1024 * 1024);
        assert_eq!(dev_parser.max_rows(), Some(1_000));
    }

    #[test]
    fn test_excel_parser_setters() {
        let mut parser = ExcelParser::new();
        
        parser.set_max_file_size(75 * 1024 * 1024);
        assert_eq!(parser.max_file_size(), 75 * 1024 * 1024);
        
        parser.set_auto_detect_headers(false);
        assert!(!parser.auto_detect_headers());
        
        parser.set_max_rows(Some(2000));
        assert_eq!(parser.max_rows(), Some(2000));
    }
}
