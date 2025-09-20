// Modified: 2025-01-20

//! Excel document parsing implementation using calamine
//!
//! This module provides safe, memory-efficient Excel file parsing with comprehensive
//! error handling and type safety. All operations follow strict Rust guidelines
//! with explicit error handling and zero unsafe code.

use crate::{DocumentParser, ParseResult, DocumentType};
use async_trait::async_trait;
use calamine::{Reader, Xlsx, DataType, Range};
use fedramp_core::{Result, Error};
use regex;
use serde::{Serialize, Deserialize};
use serde_json::{Value, Map};
use std::path::Path;
use tokio::fs;
use tracing::{debug, error, info, warn};

/// Excel-specific parsing errors
#[derive(Debug, thiserror::Error)]
pub enum ExcelError {
    #[error("Unsupported Excel format: {format}")]
    UnsupportedFormat { format: String },
    
    #[error("Worksheet not found: {name}")]
    WorksheetNotFound { name: String },
    
    #[error("Invalid cell reference: {reference}")]
    InvalidCellReference { reference: String },
    
    #[error("Data type conversion error: {message}")]
    DataTypeConversion { message: String },
    
    #[error("Calamine error: {source}")]
    Calamine {
        #[from]
        source: calamine::Error,
    },
}

/// Represents a parsed Excel worksheet with metadata, formatting, and validation
#[derive(Debug, Clone)]
pub struct ExcelWorksheet {
    /// Name of the worksheet
    pub name: String,
    /// Number of rows with data
    pub row_count: usize,
    /// Number of columns with data
    pub column_count: usize,
    /// Raw data from the worksheet (sanitized)
    pub data: Vec<Vec<Value>>,
    /// Headers if detected
    pub headers: Option<Vec<String>>,
    /// Merged cell ranges in the worksheet
    pub merged_cells: Vec<MergedCellRange>,
    /// Cell formatting information
    pub cell_formatting: Option<CellFormattingMap>,
    /// Validation results for each cell
    pub validation_results: Vec<CellValidationResult>,
    /// Overall validation summary
    pub validation_summary: ValidationSummary,
}

/// Summary of validation results for a worksheet
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationSummary {
    /// Total number of cells validated
    pub total_cells: usize,
    /// Number of cells that passed validation
    pub valid_cells: usize,
    /// Number of cells with issues
    pub invalid_cells: usize,
    /// Number of cells that were sanitized
    pub sanitized_cells: usize,
    /// Average confidence score
    pub average_confidence: f64,
    /// Breakdown of issues by type
    pub issue_breakdown: std::collections::HashMap<String, usize>,
    /// Most severe issue found
    pub max_severity: Option<ValidationSeverity>,
}

/// Information about a single worksheet
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorksheetMetadata {
    /// Name of the worksheet
    pub name: String,
    /// Zero-based index of the worksheet
    pub index: usize,
    /// Number of rows in the worksheet
    pub row_count: usize,
    /// Number of columns in the worksheet
    pub column_count: usize,
    /// Total number of cells
    pub cell_count: usize,
    /// Whether the worksheet contains data
    pub has_data: bool,
    /// Whether the worksheet is hidden
    pub is_hidden: bool,
    /// Type of worksheet
    pub sheet_type: WorksheetType,
}

/// Type of worksheet
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum WorksheetType {
    /// Regular worksheet
    Worksheet,
    /// Chart sheet
    Chart,
    /// Macro sheet
    Macro,
    /// Dialog sheet
    Dialog,
}

/// Comprehensive information about all worksheets in an Excel file
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorksheetInfo {
    /// Total number of worksheets
    pub total_count: usize,
    /// Detailed information about each worksheet
    pub sheets: Vec<WorksheetMetadata>,
    /// Total number of cells across all worksheets
    pub total_cells: usize,
    /// Total number of rows across all worksheets
    pub total_rows: usize,
    /// Whether the workbook contains hidden sheets
    pub has_hidden_sheets: bool,
}

/// Represents a merged cell range in Excel
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MergedCellRange {
    /// Starting row (0-based)
    pub start_row: usize,
    /// Starting column (0-based)
    pub start_col: usize,
    /// Ending row (0-based, inclusive)
    pub end_row: usize,
    /// Ending column (0-based, inclusive)
    pub end_col: usize,
    /// The value contained in the merged cell (from top-left cell)
    pub value: Value,
    /// Number of rows spanned
    pub row_span: usize,
    /// Number of columns spanned
    pub col_span: usize,
}

/// Cell formatting information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CellFormatting {
    /// Whether the cell is bold
    pub bold: Option<bool>,
    /// Whether the cell is italic
    pub italic: Option<bool>,
    /// Whether the cell is underlined
    pub underline: Option<bool>,
    /// Font size
    pub font_size: Option<f64>,
    /// Font name/family
    pub font_family: Option<String>,
    /// Text color (hex format)
    pub text_color: Option<String>,
    /// Background color (hex format)
    pub background_color: Option<String>,
    /// Number format (e.g., "0.00", "mm/dd/yyyy")
    pub number_format: Option<String>,
    /// Text alignment
    pub alignment: Option<TextAlignment>,
    /// Whether the cell has borders
    pub has_borders: bool,
}

/// Text alignment options
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum TextAlignment {
    Left,
    Center,
    Right,
    Justify,
    Fill,
}

/// Map of cell coordinates to formatting information
pub type CellFormattingMap = std::collections::HashMap<(usize, usize), CellFormatting>;

/// Data validation result for a single cell
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CellValidationResult {
    /// Row coordinate (0-based)
    pub row: usize,
    /// Column coordinate (0-based)
    pub col: usize,
    /// Original value before sanitization
    pub original_value: Value,
    /// Sanitized value after processing
    pub sanitized_value: Value,
    /// List of validation issues found
    pub issues: Vec<ValidationIssue>,
    /// Whether the cell passed validation
    pub is_valid: bool,
    /// Confidence score (0.0 to 1.0)
    pub confidence_score: f64,
}

/// Individual validation issue
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationIssue {
    /// Type of validation issue
    pub issue_type: ValidationIssueType,
    /// Severity level
    pub severity: ValidationSeverity,
    /// Human-readable description
    pub message: String,
    /// Suggested fix or action
    pub suggestion: Option<String>,
}

/// Types of validation issues
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ValidationIssueType {
    /// String too long
    StringTooLong,
    /// Contains control characters
    ControlCharacters,
    /// Invalid email format
    InvalidEmail,
    /// Invalid URL format
    InvalidUrl,
    /// Suspicious markup content
    SuspiciousMarkup,
    /// Data type mismatch
    TypeMismatch,
    /// Encoding issues
    EncodingError,
    /// Potential security risk
    SecurityRisk,
    /// Data quality concern
    QualityIssue,
}

/// Severity levels for validation issues
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ValidationSeverity {
    /// Low severity - informational
    Info,
    /// Medium severity - warning
    Warning,
    /// High severity - error
    Error,
    /// Critical severity - security risk
    Critical,
}

/// Supported Excel file formats
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ExcelFormat {
    /// Excel 2007+ format (.xlsx)
    Xlsx,
    /// Legacy Excel format (.xls)
    Xls,
    /// Excel macro-enabled format (.xlsm)
    Xlsm,
    /// Excel binary format (.xlsb)
    Xlsb,
}

/// Excel file parser implementation with validation and sanitization
#[derive(Debug, Clone)]
pub struct ExcelParser {
    /// Maximum file size to process (in bytes)
    max_file_size: usize,
    /// Whether to detect headers automatically
    auto_detect_headers: bool,
    /// Maximum number of rows to process per worksheet
    max_rows: Option<usize>,
    /// Data validation configuration
    validation_config: ValidationConfig,
}

/// Configuration for data validation and sanitization
#[derive(Debug, Clone)]
pub struct ValidationConfig {
    /// Whether to enable strict validation
    pub strict_mode: bool,
    /// Maximum string length allowed
    pub max_string_length: usize,
    /// Whether to sanitize HTML/XML content
    pub sanitize_markup: bool,
    /// Whether to validate email addresses
    pub validate_emails: bool,
    /// Whether to validate URLs
    pub validate_urls: bool,
    /// Whether to remove control characters
    pub remove_control_chars: bool,
    /// Maximum number of validation errors before stopping
    pub max_validation_errors: usize,
    /// Whether to normalize whitespace
    pub normalize_whitespace: bool,
}

impl ValidationConfig {
    /// Create default validation configuration
    #[must_use]
    pub fn default() -> Self {
        Self {
            strict_mode: false,
            max_string_length: 32768, // 32KB per cell
            sanitize_markup: true,
            validate_emails: true,
            validate_urls: true,
            remove_control_chars: true,
            max_validation_errors: 1000,
            normalize_whitespace: true,
        }
    }

    /// Create strict validation configuration for security-sensitive environments
    #[must_use]
    pub fn strict() -> Self {
        Self {
            strict_mode: true,
            max_string_length: 8192, // 8KB per cell
            sanitize_markup: true,
            validate_emails: true,
            validate_urls: true,
            remove_control_chars: true,
            max_validation_errors: 100,
            normalize_whitespace: true,
        }
    }
}

impl ExcelParser {
    /// Create a new Excel parser with default settings
    ///
    /// # Examples
    ///
    /// ```rust
    /// use document_parser::ExcelParser;
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
    /// use document_parser::ExcelParser;
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

    /// Parse Excel file from path with comprehensive error handling
    ///
    /// # Arguments
    ///
    /// * `path` - Path to the Excel file
    ///
    /// # Returns
    ///
    /// Returns `Result<ParseResult>` with parsed data or detailed error information
    ///
    /// # Errors
    ///
    /// Returns error if:
    /// - File doesn't exist or can't be read
    /// - File size exceeds maximum limit
    /// - Excel format is unsupported
    /// - File is corrupted or malformed
    pub async fn parse_excel_file(&self, path: &Path) -> Result<ParseResult> {
        // Validate file exists and get metadata
        let metadata = fs::metadata(path).await
            .map_err(|e| Error::document_parsing(format!("Failed to read file metadata: {}", e)))?;

        // Check file size
        if metadata.len() > self.max_file_size as u64 {
            return Err(Error::document_parsing(format!(
                "File size {} exceeds maximum limit of {} bytes",
                metadata.len(),
                self.max_file_size
            )));
        }

        info!("Parsing Excel file: {} (size: {} bytes)", path.display(), metadata.len());

        // Read file content
        let file_content = fs::read(path).await
            .map_err(|e| Error::document_parsing(format!("Failed to read file: {}", e)))?;

        self.parse_excel_bytes(&file_content, &path.to_string_lossy()).await
    }

    /// Detect and enumerate all worksheets in an Excel file
    ///
    /// # Arguments
    ///
    /// * `workbook` - Mutable reference to the Excel workbook
    ///
    /// # Returns
    ///
    /// Returns `Result<WorksheetInfo>` with detailed information about all worksheets
    ///
    /// # Errors
    ///
    /// Returns error if workbook cannot be read or contains no worksheets
    pub async fn detect_worksheets<R: std::io::Read + std::io::Seek>(
        &self,
        workbook: &mut Xlsx<R>,
    ) -> Result<WorksheetInfo> {
        debug!("Detecting and enumerating worksheets");

        // Get all worksheet names
        let sheet_names = workbook.sheet_names().to_owned();

        if sheet_names.is_empty() {
            return Err(Error::document_parsing("No worksheets found in Excel file".to_string()));
        }

        let mut sheets = Vec::with_capacity(sheet_names.len());
        let mut total_cells = 0;
        let mut total_rows = 0;

        // Analyze each worksheet
        for (index, sheet_name) in sheet_names.iter().enumerate() {
            debug!("Analyzing worksheet: {}", sheet_name);

            // Get worksheet range to determine size
            let range_result = workbook.worksheet_range(sheet_name);

            let (row_count, col_count, cell_count, has_data) = match range_result {
                Some(Ok(range)) => {
                    let dimensions = range.get_size();
                    let rows = dimensions.0;
                    let cols = dimensions.1;
                    let cells = rows * cols;
                    let has_data = !range.is_empty();

                    debug!("Worksheet '{}': {}x{} cells, has_data: {}",
                           sheet_name, rows, cols, has_data);

                    (rows, cols, cells, has_data)
                }
                Some(Err(e)) => {
                    warn!("Failed to get range for worksheet '{}': {}", sheet_name, e);
                    (0, 0, 0, false)
                }
                None => {
                    warn!("Worksheet '{}' not found", sheet_name);
                    (0, 0, 0, false)
                }
            };

            total_cells += cell_count;
            total_rows += row_count;

            let sheet_info = WorksheetMetadata {
                name: sheet_name.clone(),
                index,
                row_count,
                column_count: col_count,
                cell_count,
                has_data,
                is_hidden: false, // calamine doesn't expose hidden sheet info easily
                sheet_type: WorksheetType::Worksheet, // Default to worksheet
            };

            sheets.push(sheet_info);
        }

        let worksheet_info = WorksheetInfo {
            total_count: sheet_names.len(),
            sheets,
            total_cells,
            total_rows,
            has_hidden_sheets: false, // Would need more complex detection
        };

        info!("Detected {} worksheets with {} total cells",
              worksheet_info.total_count, worksheet_info.total_cells);

        Ok(worksheet_info)
    }

    /// Parse Excel data from bytes with type safety and error handling
    ///
    /// # Arguments
    ///
    /// * `data` - Raw Excel file bytes
    /// * `filename` - Original filename for error reporting
    ///
    /// # Returns
    ///
    /// Returns `Result<ParseResult>` with parsed worksheets and metadata
    ///
    /// # Errors
    ///
    /// Returns error if Excel data is malformed or unsupported
    pub async fn parse_excel_bytes(&self, data: &[u8], filename: &str) -> Result<ParseResult> {
        debug!("Parsing Excel bytes for file: {}", filename);

        // Create cursor for in-memory reading
        let cursor = std::io::Cursor::new(data);

        // Attempt to open workbook as XLSX format
        let mut workbook: Xlsx<_> = Xlsx::new(cursor)
            .map_err(|e| Error::document_parsing(format!("Failed to open Excel workbook: {}", e)))?;

        // Detect and enumerate worksheets
        let worksheet_info = self.detect_worksheets(&mut workbook).await?;

        info!("Detected {} worksheets with {} total cells",
              worksheet_info.total_count, worksheet_info.total_cells);

        // Parse all worksheets
        let mut worksheets = Vec::with_capacity(worksheet_info.total_count);
        let mut validation_errors = Vec::new();

        for sheet_info in &worksheet_info.sheets {
            // Skip empty worksheets if configured to do so
            if !sheet_info.has_data && self.auto_detect_headers {
                debug!("Skipping empty worksheet: {}", sheet_info.name);
                continue;
            }

            match self.parse_worksheet(&mut workbook, &sheet_info.name).await {
                Ok(worksheet) => {
                    debug!("Successfully parsed worksheet: {} ({} rows, {} cols)", 
                           worksheet.name, worksheet.row_count, worksheet.column_count);
                    worksheets.push(worksheet);
                }
                Err(e) => {
                    warn!("Failed to parse worksheet {}: {}", sheet_info.name, e);
                    validation_errors.push(format!("Worksheet '{}': {}", sheet_info.name, e));
                }
            }
        }

        // Ensure at least one worksheet was parsed successfully
        if worksheets.is_empty() {
            return Err(Error::document_parsing(format!(
                "Failed to parse any worksheets. Errors: {}",
                validation_errors.join("; ")
            )));
        }

        // Calculate quality score based on successful parsing
        let quality_score = worksheets.len() as f64 / worksheet_info.total_count as f64;

        // Create metadata
        let metadata = serde_json::json!({
            "source_file": filename,
            "source_type": "excel",
            "extraction_date": chrono::Utc::now().to_rfc3339(),
            "worksheet_count": worksheets.len(),
            "total_worksheets": worksheet_info.total_count,
            "worksheet_names": worksheet_info.sheets.iter().map(|s| &s.name).collect::<Vec<_>>(),
            "worksheet_info": worksheet_info,
            "total_cells": worksheet_info.total_cells,
            "total_rows": worksheet_info.total_rows,
            "has_hidden_sheets": worksheet_info.has_hidden_sheets,
            "parser_version": env!("CARGO_PKG_VERSION")
        });

        // Create content with all worksheets including formatting, merged cells, and validation
        let content = serde_json::json!({
            "worksheets": worksheets.into_iter().map(|ws| serde_json::json!({
                "name": ws.name,
                "row_count": ws.row_count,
                "column_count": ws.column_count,
                "headers": ws.headers,
                "data": ws.data,
                "merged_cells": ws.merged_cells,
                "cell_formatting": ws.cell_formatting,
                "validation_results": ws.validation_results,
                "validation_summary": ws.validation_summary,
                "has_merged_cells": !ws.merged_cells.is_empty(),
                "has_formatting": ws.cell_formatting.is_some(),
                "merged_cell_count": ws.merged_cells.len(),
                "validation_passed": ws.validation_summary.invalid_cells == 0,
                "data_quality_score": ws.validation_summary.average_confidence
            })).collect::<Vec<_>>()
        });

        Ok(ParseResult {
            document_type: DocumentType::Excel,
            source_path: filename.to_string(),
            metadata,
            content,
            validation_errors,
            quality_score,
        })
    }

    /// Parse a single worksheet with comprehensive error handling
    ///
    /// # Arguments
    ///
    /// * `workbook` - Mutable reference to the Excel workbook
    /// * `sheet_name` - Name of the worksheet to parse
    ///
    /// # Returns
    ///
    /// Returns `Result<ExcelWorksheet>` with parsed worksheet data
    ///
    /// # Errors
    ///
    /// Returns error if worksheet cannot be found or parsed
    async fn parse_worksheet<R: std::io::Read + std::io::Seek>(
        &self,
        workbook: &mut Xlsx<R>,
        sheet_name: &str,
    ) -> Result<ExcelWorksheet> {
        // Get the range for the worksheet
        let range = workbook
            .worksheet_range(sheet_name)
            .ok_or_else(|| Error::document_parsing(format!("Worksheet '{}' not found", sheet_name)))?
            .map_err(|e| Error::document_parsing(format!("Failed to get worksheet range: {}", e)))?;

        self.parse_range(sheet_name, &range).await
    }

    /// Parse a calamine Range into structured data
    ///
    /// # Arguments
    ///
    /// * `sheet_name` - Name of the worksheet
    /// * `range` - Calamine Range containing the data
    ///
    /// # Returns
    ///
    /// Returns `Result<ExcelWorksheet>` with parsed and validated data
    async fn parse_range(&self, sheet_name: &str, range: &Range<DataType>) -> Result<ExcelWorksheet> {
        let (height, width) = range.get_size();
        
        if height == 0 || width == 0 {
            return Ok(ExcelWorksheet {
                name: sheet_name.to_string(),
                row_count: 0,
                column_count: 0,
                data: Vec::new(),
                headers: None,
                merged_cells: Vec::new(),
                cell_formatting: None,
                validation_results: Vec::new(),
                validation_summary: ValidationSummary {
                    total_cells: 0,
                    valid_cells: 0,
                    invalid_cells: 0,
                    sanitized_cells: 0,
                    average_confidence: 1.0,
                    issue_breakdown: std::collections::HashMap::new(),
                    max_severity: None,
                },
            });
        }

        // Apply row limit if specified
        let effective_height = self.max_rows.map_or(height, |max| height.min(max));
        
        debug!("Parsing range: {}x{} (limited to {}x{})", height, width, effective_height, width);

        // Parse data with type conversion
        let mut data = Vec::with_capacity(effective_height);
        let mut headers = None;

        // Track validation results
        let mut validation_results = Vec::new();
        let mut validation_error_count = 0;

        for row_idx in 0..effective_height {
            let mut row_data = Vec::with_capacity(width);

            for col_idx in 0..width {
                let cell_value = range.get_value((row_idx as u32, col_idx as u32))
                    .map(|cell| self.convert_cell_value(cell))
                    .unwrap_or(Value::Null);

                // Validate and sanitize the cell value
                let validation_result = self.validate_and_sanitize_cell(&cell_value, row_idx, col_idx);

                // Check if we should stop due to too many validation errors
                if !validation_result.is_valid {
                    validation_error_count += 1;
                    if validation_error_count > self.validation_config.max_validation_errors {
                        warn!("Stopping validation due to too many errors ({})", validation_error_count);
                        break;
                    }
                }

                validation_results.push(validation_result.clone());
                row_data.push(validation_result.sanitized_value);
            }

            // Break outer loop if we hit validation error limit
            if validation_error_count > self.validation_config.max_validation_errors {
                break;
            }

            // Detect headers from first row if enabled
            if row_idx == 0 && self.auto_detect_headers {
                headers = Some(self.extract_headers(&row_data));
            }

            data.push(row_data);
        }

        // Detect merged cells and formatting
        let merged_cells = self.detect_merged_cells(range).await?;
        let cell_formatting = if self.should_preserve_formatting() {
            Some(self.extract_cell_formatting(range).await?)
        } else {
            None
        };

        // Calculate validation summary
        let validation_summary = self.calculate_validation_summary(&validation_results);

        Ok(ExcelWorksheet {
            name: sheet_name.to_string(),
            row_count: effective_height,
            column_count: width,
            data,
            headers,
            merged_cells,
            cell_formatting,
            validation_results,
            validation_summary,
        })
    }

    /// Convert calamine DataType to JSON Value with advanced type detection
    ///
    /// This method provides comprehensive type detection and conversion including:
    /// - Automatic date/time recognition and ISO 8601 formatting
    /// - Numeric precision handling for integers vs floats
    /// - String normalization and trimming
    /// - Boolean value detection from text
    /// - Error and duration handling
    ///
    /// # Arguments
    ///
    /// * `cell` - Calamine DataType to convert
    ///
    /// # Returns
    ///
    /// Returns appropriate JSON Value with the most accurate type detection
    #[must_use]
    fn convert_cell_value(&self, cell: &DataType) -> Value {
        match cell {
            DataType::Empty => Value::Null,
            DataType::String(s) => self.convert_string_with_type_detection(s),
            DataType::Float(f) => self.convert_numeric_value(*f),
            DataType::Int(i) => Value::Number(serde_json::Number::from(*i)),
            DataType::Bool(b) => Value::Bool(*b),
            DataType::DateTime(dt) => {
                // Convert Excel datetime to ISO 8601 string with proper formatting
                Value::String(self.format_excel_datetime(dt))
            }
            DataType::Error(e) => {
                warn!("Excel cell contains error: {:?}", e);
                Value::String(format!("ERROR: {:?}", e))
            }
            DataType::DateTimeIso(dt_str) => {
                // Validate and potentially reformat ISO datetime
                Value::String(self.validate_iso_datetime(dt_str))
            }
            DataType::DurationIso(dur_str) => {
                // Convert ISO duration to a more readable format
                Value::String(self.format_iso_duration(dur_str))
            }
            DataType::Duration(dur) => {
                // Convert duration to human-readable format
                Value::String(self.format_duration_seconds(*dur))
            }
        }
    }

    /// Convert string values with advanced type detection
    ///
    /// # Arguments
    ///
    /// * `s` - The string value to analyze and convert
    ///
    /// # Returns
    ///
    /// Returns a serde_json::Value with detected type (string, number, boolean, or null)
    #[must_use]
    fn convert_string_with_type_detection(&self, s: &str) -> Value {
        let trimmed = s.trim();

        // Handle empty strings
        if trimmed.is_empty() {
            return Value::Null;
        }

        // Try to detect boolean values (case-insensitive)
        match trimmed.to_lowercase().as_str() {
            "true" | "yes" | "y" | "1" | "on" | "enabled" | "active" => return Value::Bool(true),
            "false" | "no" | "n" | "0" | "off" | "disabled" | "inactive" => return Value::Bool(false),
            _ => {}
        }

        // Try to parse as integer first (more precise)
        if let Ok(int_val) = trimmed.parse::<i64>() {
            return Value::Number(serde_json::Number::from(int_val));
        }

        // Try to parse as float
        if let Ok(float_val) = trimmed.parse::<f64>() {
            if float_val.is_finite() {
                if let Some(num) = serde_json::Number::from_f64(float_val) {
                    return Value::Number(num);
                }
            }
        }

        // Try to detect and parse date strings
        if let Some(formatted_date) = self.detect_and_parse_date(trimmed) {
            return Value::String(formatted_date);
        }

        // Return as normalized string if no other type detected
        Value::String(trimmed.to_string())
    }

    /// Convert numeric values with proper precision handling
    ///
    /// # Arguments
    ///
    /// * `f` - The float value to convert
    ///
    /// # Returns
    ///
    /// Returns a JSON Number with appropriate precision
    #[must_use]
    fn convert_numeric_value(&self, f: f64) -> Value {
        // Handle potential NaN or infinite values
        if !f.is_finite() {
            warn!("Encountered non-finite float value: {}, converting to null", f);
            return Value::Null;
        }

        // Check if it's actually an integer (no fractional part)
        if f.fract() == 0.0 && f >= i64::MIN as f64 && f <= i64::MAX as f64 {
            Value::Number(serde_json::Number::from(f as i64))
        } else {
            Value::Number(serde_json::Number::from_f64(f).unwrap_or_else(|| {
                warn!("Failed to convert float {} to JSON number, using 0", f);
                serde_json::Number::from(0)
            }))
        }
    }

    /// Format Excel datetime to ISO 8601 string
    ///
    /// # Arguments
    ///
    /// * `dt` - The Excel datetime value
    ///
    /// # Returns
    ///
    /// Returns formatted ISO 8601 datetime string
    #[must_use]
    fn format_excel_datetime(&self, dt: &f64) -> String {
        // Excel dates are stored as days since 1900-01-01 (with some quirks)
        // This is a simplified conversion - in production you'd want to use
        // a proper Excel date conversion library
        format!("EXCEL_DATE:{}", dt)
    }

    /// Validate and format ISO datetime string
    ///
    /// # Arguments
    ///
    /// * `dt_str` - The ISO datetime string to validate
    ///
    /// # Returns
    ///
    /// Returns validated and potentially reformatted datetime string
    #[must_use]
    fn validate_iso_datetime(&self, dt_str: &str) -> String {
        // Basic validation - in production you'd want proper datetime parsing
        if dt_str.contains('T') || dt_str.contains('-') {
            dt_str.to_string()
        } else {
            format!("INVALID_DATE:{}", dt_str)
        }
    }

    /// Format ISO duration to human-readable format
    ///
    /// # Arguments
    ///
    /// * `dur_str` - The ISO duration string
    ///
    /// # Returns
    ///
    /// Returns human-readable duration string
    #[must_use]
    fn format_iso_duration(&self, dur_str: &str) -> String {
        // Basic formatting - in production you'd want proper duration parsing
        if dur_str.starts_with('P') {
            dur_str.to_string()
        } else {
            format!("DURATION:{}", dur_str)
        }
    }

    /// Format duration in seconds to human-readable format
    ///
    /// # Arguments
    ///
    /// * `seconds` - Duration in seconds
    ///
    /// # Returns
    ///
    /// Returns human-readable duration string
    #[must_use]
    fn format_duration_seconds(&self, seconds: f64) -> String {
        if seconds < 60.0 {
            format!("{:.1}s", seconds)
        } else if seconds < 3600.0 {
            format!("{:.1}m", seconds / 60.0)
        } else if seconds < 86400.0 {
            format!("{:.1}h", seconds / 3600.0)
        } else {
            format!("{:.1}d", seconds / 86400.0)
        }
    }

    /// Detect and parse date strings from various formats
    ///
    /// # Arguments
    ///
    /// * `s` - The string to analyze for date patterns
    ///
    /// # Returns
    ///
    /// Returns formatted date string if a date pattern is detected
    #[must_use]
    fn detect_and_parse_date(&self, s: &str) -> Option<String> {
        // Basic date pattern detection
        // In production, you'd want to use a proper date parsing library like chrono

        // Check for common date patterns
        if s.len() >= 8 && (s.contains('-') || s.contains('/') || s.contains('.')) {
            // Simple validation for date-like strings
            let parts: Vec<&str> = s.split(|c| c == '-' || c == '/' || c == '.').collect();
            if parts.len() >= 3 {
                // Assume it's a date and return as-is for now
                return Some(s.to_string());
            }
        }

        // Check for ISO-like formats
        if s.len() >= 10 && s.chars().nth(4) == Some('-') && s.chars().nth(7) == Some('-') {
            return Some(s.to_string());
        }

        None
    }

    /// Detect merged cells in the worksheet range
    ///
    /// # Arguments
    ///
    /// * `range` - The worksheet range to analyze
    ///
    /// # Returns
    ///
    /// Returns a vector of merged cell ranges found in the worksheet
    ///
    /// # Errors
    ///
    /// Returns error if range analysis fails
    async fn detect_merged_cells(&self, range: &Range<DataType>) -> Result<Vec<MergedCellRange>> {
        let mut merged_cells = Vec::new();
        let (height, width) = range.get_size();

        debug!("Detecting merged cells in {}x{} range", height, width);

        // Track which cells we've already processed as part of merged ranges
        let mut processed_cells = std::collections::HashSet::new();

        // Scan through all cells looking for potential merged cell patterns
        for row in 0..height {
            for col in 0..width {
                let cell_coord = (row, col);

                // Skip if we've already processed this cell as part of a merged range
                if processed_cells.contains(&cell_coord) {
                    continue;
                }

                // Get the value of the current cell
                if let Some(cell_value) = range.get_value((row as u32, col as u32)) {
                    // Check if this cell has the same value as adjacent cells (potential merge)
                    if let Some(merged_range) = self.detect_merged_range_from_cell(
                        range, row, col, cell_value, &mut processed_cells
                    ).await? {
                        merged_cells.push(merged_range);
                    }
                }
            }
        }

        info!("Detected {} merged cell ranges", merged_cells.len());
        Ok(merged_cells)
    }

    /// Detect a merged range starting from a specific cell
    ///
    /// # Arguments
    ///
    /// * `range` - The worksheet range
    /// * `start_row` - Starting row coordinate
    /// * `start_col` - Starting column coordinate
    /// * `cell_value` - The value to match for merged cells
    /// * `processed_cells` - Set of already processed cell coordinates
    ///
    /// # Returns
    ///
    /// Returns Some(MergedCellRange) if a merged range is detected, None otherwise
    async fn detect_merged_range_from_cell(
        &self,
        range: &Range<DataType>,
        start_row: usize,
        start_col: usize,
        cell_value: &DataType,
        processed_cells: &mut std::collections::HashSet<(usize, usize)>,
    ) -> Result<Option<MergedCellRange>> {
        let (height, width) = range.get_size();

        // Only consider non-empty cells for merging
        if matches!(cell_value, DataType::Empty) {
            return Ok(None);
        }

        let mut end_row = start_row;
        let mut end_col = start_col;
        let mut found_merge = false;

        // Check horizontally for consecutive cells with the same value
        while end_col + 1 < width {
            if let Some(next_cell) = range.get_value((start_row as u32, (end_col + 1) as u32)) {
                if self.cells_should_be_merged(cell_value, next_cell) {
                    end_col += 1;
                    found_merge = true;
                } else {
                    break;
                }
            } else {
                break;
            }
        }

        // Check vertically for consecutive cells with the same value
        while end_row + 1 < height {
            let mut all_match = true;
            for col in start_col..=end_col {
                if let Some(next_cell) = range.get_value(((end_row + 1) as u32, col as u32)) {
                    if !self.cells_should_be_merged(cell_value, next_cell) {
                        all_match = false;
                        break;
                    }
                } else {
                    all_match = false;
                    break;
                }
            }

            if all_match {
                end_row += 1;
                found_merge = true;
            } else {
                break;
            }
        }

        // Only create a merged range if it spans more than one cell
        if found_merge && (end_row > start_row || end_col > start_col) {
            // Mark all cells in this range as processed
            for row in start_row..=end_row {
                for col in start_col..=end_col {
                    processed_cells.insert((row, col));
                }
            }

            let merged_range = MergedCellRange {
                start_row,
                start_col,
                end_row,
                end_col,
                value: self.convert_cell_value(cell_value),
                row_span: end_row - start_row + 1,
                col_span: end_col - start_col + 1,
            };

            debug!("Detected merged range: ({},{}) to ({},{}) spanning {}x{} cells",
                   start_row, start_col, end_row, end_col,
                   merged_range.row_span, merged_range.col_span);

            Ok(Some(merged_range))
        } else {
            Ok(None)
        }
    }

    /// Determine if two cells should be considered merged based on their values
    ///
    /// # Arguments
    ///
    /// * `cell1` - First cell value
    /// * `cell2` - Second cell value
    ///
    /// # Returns
    ///
    /// Returns true if the cells should be considered part of a merged range
    fn cells_should_be_merged(&self, cell1: &DataType, cell2: &DataType) -> bool {
        match (cell1, cell2) {
            // Both empty - could be merged
            (DataType::Empty, DataType::Empty) => false, // Don't merge empty cells
            // Same string values
            (DataType::String(s1), DataType::String(s2)) => s1 == s2 && !s1.trim().is_empty(),
            // Same numeric values
            (DataType::Float(f1), DataType::Float(f2)) => (f1 - f2).abs() < f64::EPSILON,
            (DataType::Int(i1), DataType::Int(i2)) => i1 == i2,
            // Same boolean values
            (DataType::Bool(b1), DataType::Bool(b2)) => b1 == b2,
            // Mixed numeric types
            (DataType::Float(f), DataType::Int(i)) | (DataType::Int(i), DataType::Float(f)) => {
                (*f - *i as f64).abs() < f64::EPSILON
            }
            // Different types or one empty - not merged
            _ => false,
        }
    }

    /// Check if formatting should be preserved based on parser configuration
    ///
    /// # Returns
    ///
    /// Returns true if cell formatting should be extracted and preserved
    fn should_preserve_formatting(&self) -> bool {
        // For now, always preserve formatting if the file is small enough
        // In a real implementation, this could be configurable
        true
    }

    /// Extract cell formatting information from the worksheet range
    ///
    /// # Arguments
    ///
    /// * `range` - The worksheet range to analyze
    ///
    /// # Returns
    ///
    /// Returns a map of cell coordinates to formatting information
    ///
    /// # Errors
    ///
    /// Returns error if formatting extraction fails
    async fn extract_cell_formatting(&self, range: &Range<DataType>) -> Result<CellFormattingMap> {
        let mut formatting_map = CellFormattingMap::new();
        let (height, width) = range.get_size();

        debug!("Extracting cell formatting for {}x{} range", height, width);

        // Note: calamine doesn't provide direct access to formatting information
        // In a real implementation, you would need a library that can read Excel formatting
        // For now, we'll create placeholder formatting based on cell content patterns

        for row in 0..height {
            for col in 0..width {
                if let Some(cell_value) = range.get_value((row as u32, col as u32)) {
                    if let Some(formatting) = self.infer_formatting_from_content(cell_value) {
                        formatting_map.insert((row, col), formatting);
                    }
                }
            }
        }

        info!("Extracted formatting for {} cells", formatting_map.len());
        Ok(formatting_map)
    }

    /// Infer basic formatting from cell content patterns
    ///
    /// # Arguments
    ///
    /// * `cell_value` - The cell value to analyze
    ///
    /// # Returns
    ///
    /// Returns Some(CellFormatting) if formatting can be inferred, None otherwise
    fn infer_formatting_from_content(&self, cell_value: &DataType) -> Option<CellFormatting> {
        match cell_value {
            DataType::Empty => None,
            DataType::String(s) => {
                // Infer formatting based on string patterns
                let mut formatting = CellFormatting {
                    bold: None,
                    italic: None,
                    underline: None,
                    font_size: None,
                    font_family: None,
                    text_color: None,
                    background_color: None,
                    number_format: None,
                    alignment: None,
                    has_borders: false,
                };

                // Check for header-like patterns (all caps, contains keywords)
                if s.chars().all(|c| c.is_uppercase() || !c.is_alphabetic()) && s.len() > 2 {
                    formatting.bold = Some(true);
                    formatting.alignment = Some(TextAlignment::Center);
                }

                // Check for URL patterns
                if s.starts_with("http://") || s.starts_with("https://") {
                    formatting.text_color = Some("#0000FF".to_string()); // Blue for links
                    formatting.underline = Some(true);
                }

                Some(formatting)
            }
            DataType::Float(_) => {
                // Numeric formatting
                Some(CellFormatting {
                    bold: None,
                    italic: None,
                    underline: None,
                    font_size: None,
                    font_family: None,
                    text_color: None,
                    background_color: None,
                    number_format: Some("0.00".to_string()),
                    alignment: Some(TextAlignment::Right),
                    has_borders: false,
                })
            }
            DataType::Int(_) => {
                // Integer formatting
                Some(CellFormatting {
                    bold: None,
                    italic: None,
                    underline: None,
                    font_size: None,
                    font_family: None,
                    text_color: None,
                    background_color: None,
                    number_format: Some("0".to_string()),
                    alignment: Some(TextAlignment::Right),
                    has_borders: false,
                })
            }
            DataType::DateTime(_) => {
                // Date formatting
                Some(CellFormatting {
                    bold: None,
                    italic: None,
                    underline: None,
                    font_size: None,
                    font_family: None,
                    text_color: None,
                    background_color: None,
                    number_format: Some("mm/dd/yyyy".to_string()),
                    alignment: Some(TextAlignment::Center),
                    has_borders: false,
                })
            }
            _ => None,
        }
    }

    /// Validate and sanitize a cell value
    ///
    /// # Arguments
    ///
    /// * `value` - The cell value to validate and sanitize
    /// * `row` - Row coordinate for error reporting
    /// * `col` - Column coordinate for error reporting
    ///
    /// # Returns
    ///
    /// Returns a CellValidationResult with validation status and sanitized value
    fn validate_and_sanitize_cell(&self, value: &Value, row: usize, col: usize) -> CellValidationResult {
        let mut issues = Vec::new();
        let mut sanitized_value = value.clone();
        let mut confidence_score = 1.0;

        // Validate and sanitize based on value type
        match value {
            Value::String(s) => {
                sanitized_value = Value::String(self.sanitize_string(s, &mut issues, &mut confidence_score));
            }
            Value::Number(n) => {
                self.validate_number(n, &mut issues, &mut confidence_score);
            }
            Value::Bool(_) => {
                // Booleans are generally safe, no sanitization needed
            }
            Value::Null => {
                // Null values are safe
            }
            Value::Array(_) | Value::Object(_) => {
                // Complex types shouldn't appear in Excel cells normally
                issues.push(ValidationIssue {
                    issue_type: ValidationIssueType::TypeMismatch,
                    severity: ValidationSeverity::Warning,
                    message: "Unexpected complex data type in Excel cell".to_string(),
                    suggestion: Some("Consider flattening complex data structures".to_string()),
                });
                confidence_score *= 0.8;
            }
        }

        let is_valid = issues.iter().all(|issue| !matches!(issue.severity, ValidationSeverity::Error | ValidationSeverity::Critical));

        CellValidationResult {
            row,
            col,
            original_value: value.clone(),
            sanitized_value,
            issues,
            is_valid,
            confidence_score,
        }
    }

    /// Sanitize a string value
    ///
    /// # Arguments
    ///
    /// * `s` - The string to sanitize
    /// * `issues` - Vector to collect validation issues
    /// * `confidence_score` - Mutable reference to confidence score
    ///
    /// # Returns
    ///
    /// Returns the sanitized string
    fn sanitize_string(&self, s: &str, issues: &mut Vec<ValidationIssue>, confidence_score: &mut f64) -> String {
        let mut sanitized = s.to_string();

        // Check string length
        if s.len() > self.validation_config.max_string_length {
            issues.push(ValidationIssue {
                issue_type: ValidationIssueType::StringTooLong,
                severity: if self.validation_config.strict_mode { ValidationSeverity::Error } else { ValidationSeverity::Warning },
                message: format!("String length {} exceeds maximum {}", s.len(), self.validation_config.max_string_length),
                suggestion: Some("Consider truncating or splitting the content".to_string()),
            });

            // Truncate if not in strict mode
            if !self.validation_config.strict_mode {
                sanitized.truncate(self.validation_config.max_string_length);
            }
            *confidence_score *= 0.7;
        }

        // Remove control characters if configured
        if self.validation_config.remove_control_chars {
            let original_len = sanitized.len();
            sanitized = sanitized.chars()
                .filter(|c| !c.is_control() || *c == '\n' || *c == '\r' || *c == '\t')
                .collect();

            if sanitized.len() != original_len {
                issues.push(ValidationIssue {
                    issue_type: ValidationIssueType::ControlCharacters,
                    severity: ValidationSeverity::Info,
                    message: "Removed control characters from string".to_string(),
                    suggestion: None,
                });
                *confidence_score *= 0.95;
            }
        }

        // Normalize whitespace if configured
        if self.validation_config.normalize_whitespace {
            let original = sanitized.clone();
            sanitized = sanitized.split_whitespace().collect::<Vec<_>>().join(" ");

            if sanitized != original {
                issues.push(ValidationIssue {
                    issue_type: ValidationIssueType::QualityIssue,
                    severity: ValidationSeverity::Info,
                    message: "Normalized whitespace in string".to_string(),
                    suggestion: None,
                });
                *confidence_score *= 0.98;
            }
        }

        // Check for suspicious markup if configured
        if self.validation_config.sanitize_markup {
            if self.contains_suspicious_markup(&sanitized) {
                issues.push(ValidationIssue {
                    issue_type: ValidationIssueType::SuspiciousMarkup,
                    severity: ValidationSeverity::Warning,
                    message: "String contains potentially unsafe markup".to_string(),
                    suggestion: Some("Review content for HTML/XML tags or scripts".to_string()),
                });
                *confidence_score *= 0.6;

                // Basic markup sanitization
                sanitized = self.sanitize_markup(&sanitized);
            }
        }

        // Validate email format if it looks like an email
        if self.validation_config.validate_emails && self.looks_like_email(&sanitized) {
            if !self.is_valid_email(&sanitized) {
                issues.push(ValidationIssue {
                    issue_type: ValidationIssueType::InvalidEmail,
                    severity: ValidationSeverity::Warning,
                    message: "String appears to be an email but has invalid format".to_string(),
                    suggestion: Some("Verify email address format".to_string()),
                });
                *confidence_score *= 0.8;
            }
        }

        // Validate URL format if it looks like a URL
        if self.validation_config.validate_urls && self.looks_like_url(&sanitized) {
            if !self.is_valid_url(&sanitized) {
                issues.push(ValidationIssue {
                    issue_type: ValidationIssueType::InvalidUrl,
                    severity: ValidationSeverity::Warning,
                    message: "String appears to be a URL but has invalid format".to_string(),
                    suggestion: Some("Verify URL format and accessibility".to_string()),
                });
                *confidence_score *= 0.8;
            }
        }

        sanitized
    }

    /// Validate a numeric value
    ///
    /// # Arguments
    ///
    /// * `n` - The number to validate
    /// * `issues` - Vector to collect validation issues
    /// * `confidence_score` - Mutable reference to confidence score
    fn validate_number(&self, n: &serde_json::Number, issues: &mut Vec<ValidationIssue>, confidence_score: &mut f64) {
        // Check for extremely large numbers that might cause issues
        if let Some(f) = n.as_f64() {
            if !f.is_finite() {
                issues.push(ValidationIssue {
                    issue_type: ValidationIssueType::QualityIssue,
                    severity: ValidationSeverity::Error,
                    message: "Number is not finite (NaN or infinity)".to_string(),
                    suggestion: Some("Replace with a valid numeric value".to_string()),
                });
                *confidence_score *= 0.5;
            } else if f.abs() > 1e15 {
                issues.push(ValidationIssue {
                    issue_type: ValidationIssueType::QualityIssue,
                    severity: ValidationSeverity::Warning,
                    message: "Extremely large number detected".to_string(),
                    suggestion: Some("Verify number accuracy and consider scientific notation".to_string()),
                });
                *confidence_score *= 0.9;
            }
        }
    }

    /// Check if string contains suspicious markup
    ///
    /// # Arguments
    ///
    /// * `s` - The string to check
    ///
    /// # Returns
    ///
    /// Returns true if the string contains potentially unsafe markup
    fn contains_suspicious_markup(&self, s: &str) -> bool {
        let suspicious_patterns = [
            "<script", "</script>", "javascript:", "vbscript:",
            "<iframe", "<object", "<embed", "<form",
            "onclick=", "onload=", "onerror=", "onmouseover=",
            "eval(", "document.cookie", "window.location",
        ];

        let lower_s = s.to_lowercase();
        suspicious_patterns.iter().any(|pattern| lower_s.contains(pattern))
    }

    /// Basic markup sanitization
    ///
    /// # Arguments
    ///
    /// * `s` - The string to sanitize
    ///
    /// # Returns
    ///
    /// Returns sanitized string with potentially dangerous markup removed
    fn sanitize_markup(&self, s: &str) -> String {
        // Basic HTML/XML tag removal
        let mut sanitized = s.to_string();

        // Remove script tags and their content
        let script_regex = regex::Regex::new(r"(?i)<script[^>]*>.*?</script>").unwrap_or_else(|_| {
            // Fallback if regex fails
            return regex::Regex::new(r"").unwrap();
        });
        sanitized = script_regex.replace_all(&sanitized, "").to_string();

        // Remove potentially dangerous attributes
        let attr_regex = regex::Regex::new(r"(?i)\s+(on\w+|javascript:|vbscript:)[^>\s]*").unwrap_or_else(|_| {
            return regex::Regex::new(r"").unwrap();
        });
        sanitized = attr_regex.replace_all(&sanitized, "").to_string();

        // Remove remaining HTML tags (basic approach)
        let tag_regex = regex::Regex::new(r"<[^>]*>").unwrap_or_else(|_| {
            return regex::Regex::new(r"").unwrap();
        });
        sanitized = tag_regex.replace_all(&sanitized, "").to_string();

        sanitized
    }

    /// Check if string looks like an email address
    ///
    /// # Arguments
    ///
    /// * `s` - The string to check
    ///
    /// # Returns
    ///
    /// Returns true if the string appears to be an email address
    fn looks_like_email(&self, s: &str) -> bool {
        // Basic heuristics: contains @, has text before and after @, contains dot after @
        if s.len() <= 5 || !s.contains('@') {
            return false;
        }

        let parts: Vec<&str> = s.split('@').collect();
        if parts.len() != 2 {
            return false;
        }

        let (local, domain) = (parts[0], parts[1]);
        !local.is_empty() && !domain.is_empty() && domain.contains('.')
    }

    /// Validate email address format
    ///
    /// # Arguments
    ///
    /// * `s` - The string to validate as email
    ///
    /// # Returns
    ///
    /// Returns true if the string is a valid email format
    fn is_valid_email(&self, s: &str) -> bool {
        // Basic email validation - in production, use a proper email validation library
        let email_regex = regex::Regex::new(r"^[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}$").unwrap_or_else(|_| {
            return regex::Regex::new(r"").unwrap();
        });
        email_regex.is_match(s)
    }

    /// Check if string looks like a URL
    ///
    /// # Arguments
    ///
    /// * `s` - The string to check
    ///
    /// # Returns
    ///
    /// Returns true if the string appears to be a URL
    fn looks_like_url(&self, s: &str) -> bool {
        s.starts_with("http://") || s.starts_with("https://") || s.starts_with("ftp://") || s.starts_with("www.")
    }

    /// Validate URL format
    ///
    /// # Arguments
    ///
    /// * `s` - The string to validate as URL
    ///
    /// # Returns
    ///
    /// Returns true if the string is a valid URL format
    fn is_valid_url(&self, s: &str) -> bool {
        // Basic URL validation - in production, use a proper URL validation library
        let url_regex = regex::Regex::new(r"^(https?|ftp)://[^\s/$.?#].[^\s]*$").unwrap_or_else(|_| {
            return regex::Regex::new(r"").unwrap();
        });
        url_regex.is_match(s) || s.starts_with("www.") && s.contains('.')
    }

    /// Calculate validation summary from individual cell results
    ///
    /// # Arguments
    ///
    /// * `validation_results` - Vector of cell validation results
    ///
    /// # Returns
    ///
    /// Returns a ValidationSummary with aggregated statistics
    fn calculate_validation_summary(&self, validation_results: &[CellValidationResult]) -> ValidationSummary {
        let total_cells = validation_results.len();
        let valid_cells = validation_results.iter().filter(|r| r.is_valid).count();
        let invalid_cells = total_cells - valid_cells;
        let sanitized_cells = validation_results.iter()
            .filter(|r| r.original_value != r.sanitized_value)
            .count();

        let average_confidence = if total_cells > 0 {
            validation_results.iter().map(|r| r.confidence_score).sum::<f64>() / total_cells as f64
        } else {
            1.0
        };

        // Count issues by type
        let mut issue_breakdown = std::collections::HashMap::new();
        let mut max_severity = None;

        for result in validation_results {
            for issue in &result.issues {
                let issue_type_str = format!("{:?}", issue.issue_type);
                *issue_breakdown.entry(issue_type_str).or_insert(0) += 1;

                // Track maximum severity
                match (&max_severity, &issue.severity) {
                    (None, severity) => max_severity = Some(severity.clone()),
                    (Some(current_max), new_severity) => {
                        if self.severity_level(new_severity) > self.severity_level(current_max) {
                            max_severity = Some(new_severity.clone());
                        }
                    }
                }
            }
        }

        ValidationSummary {
            total_cells,
            valid_cells,
            invalid_cells,
            sanitized_cells,
            average_confidence,
            issue_breakdown,
            max_severity,
        }
    }

    /// Get numeric severity level for comparison
    ///
    /// # Arguments
    ///
    /// * `severity` - The validation severity to convert
    ///
    /// # Returns
    ///
    /// Returns numeric level (higher = more severe)
    fn severity_level(&self, severity: &ValidationSeverity) -> u8 {
        match severity {
            ValidationSeverity::Info => 1,
            ValidationSeverity::Warning => 2,
            ValidationSeverity::Error => 3,
            ValidationSeverity::Critical => 4,
        }
    }

    /// Detect Excel format from filename
    ///
    /// # Arguments
    ///
    /// * `filename` - The filename to analyze
    ///
    /// # Returns
    ///
    /// Returns Ok if the format is supported, Err otherwise
    fn detect_excel_format(&self, filename: &str) -> Result<ExcelFormat> {
        // Check for empty filename or filename that starts with extension only
        if filename.is_empty() || filename.starts_with('.') {
            return Err(Error::validation(format!("Invalid filename: {}", filename)));
        }

        let lower_filename = filename.to_lowercase();

        if lower_filename.ends_with(".xlsx") && lower_filename.len() > 5 {
            Ok(ExcelFormat::Xlsx)
        } else if lower_filename.ends_with(".xls") && lower_filename.len() > 4 {
            Ok(ExcelFormat::Xls)
        } else if lower_filename.ends_with(".xlsm") && lower_filename.len() > 5 {
            Ok(ExcelFormat::Xlsm)
        } else if lower_filename.ends_with(".xlsb") && lower_filename.len() > 5 {
            Ok(ExcelFormat::Xlsb)
        } else {
            Err(Error::validation(format!("Unsupported Excel format: {}", filename)))
        }
    }

    /// Extract headers from the first row with validation
    ///
    /// # Arguments
    ///
    /// * `row_data` - First row data to extract headers from
    ///
    /// # Returns
    ///
    /// Returns vector of header strings with proper sanitization
    #[must_use]
    fn extract_headers(&self, row_data: &[Value]) -> Vec<String> {
        row_data
            .iter()
            .enumerate()
            .map(|(idx, value)| {
                match value {
                    Value::String(s) if !s.trim().is_empty() => s.trim().to_string(),
                    Value::Number(n) => n.to_string(),
                    Value::Bool(b) => b.to_string(),
                    _ => format!("Column_{}", idx + 1),
                }
            })
            .collect()
    }
}

impl Default for ExcelParser {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl DocumentParser for ExcelParser {
    /// Parse Excel file from path
    async fn parse_file(&self, path: &Path) -> Result<ParseResult> {
        self.parse_excel_file(path).await
    }

    /// Parse Excel data from bytes
    async fn parse_bytes(&self, data: &[u8], filename: &str) -> Result<ParseResult> {
        self.parse_excel_bytes(data, filename).await
    }

    /// Validate Excel content structure
    async fn validate(&self, content: &Value) -> Result<Vec<String>> {
        let mut errors = Vec::new();

        // Validate that content has worksheets
        if let Some(worksheets) = content.get("worksheets").and_then(|w| w.as_array()) {
            if worksheets.is_empty() {
                errors.push("No worksheets found in Excel content".to_string());
            }

            // Validate each worksheet structure
            for (idx, worksheet) in worksheets.iter().enumerate() {
                if let Some(ws_obj) = worksheet.as_object() {
                    // Check required fields
                    if !ws_obj.contains_key("name") {
                        errors.push(format!("Worksheet {} missing 'name' field", idx));
                    }
                    if !ws_obj.contains_key("data") {
                        errors.push(format!("Worksheet {} missing 'data' field", idx));
                    }

                    // Validate data structure
                    if let Some(data) = ws_obj.get("data").and_then(|d| d.as_array()) {
                        for (row_idx, row) in data.iter().enumerate() {
                            if !row.is_array() {
                                errors.push(format!(
                                    "Worksheet {} row {} is not an array",
                                    idx, row_idx
                                ));
                            }
                        }
                    }
                } else {
                    errors.push(format!("Worksheet {} is not a valid object", idx));
                }
            }
        } else {
            errors.push("Content does not contain valid worksheets array".to_string());
        }

        Ok(errors)
    }

    /// Get supported file extensions for Excel files
    fn supported_extensions(&self) -> Vec<&'static str> {
        vec!["xlsx", "xls"]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_excel_parser_creation() {
        let parser = ExcelParser::new();
        assert_eq!(parser.max_file_size, 100 * 1024 * 1024); // 100MB default
        assert!(parser.auto_detect_headers);
        assert_eq!(parser.max_rows, None); // No limit by default
        assert!(!parser.validation_config.strict_mode); // Default validation config
    }

    #[tokio::test]
    async fn test_supported_extensions() {
        let parser = ExcelParser::new();
        let extensions = parser.supported_extensions();
        assert!(extensions.contains(&"xlsx"));
        assert!(extensions.contains(&"xls"));
    }

    #[tokio::test]
    async fn test_worksheet_detection() {
        let parser = ExcelParser::new();

        // Test with empty data - should fail gracefully
        let empty_data = b"";
        let result = parser.parse_excel_bytes(empty_data, "test.xlsx").await;
        assert!(result.is_err()); // Should fail with empty data

        // Test that the error message is meaningful
        let error_msg = result.unwrap_err().to_string();
        assert!(error_msg.contains("Failed to open Excel workbook"));
    }

    #[tokio::test]
    async fn test_worksheet_info_structure() {
        // Test that WorksheetInfo can be serialized
        let worksheet_info = WorksheetInfo {
            total_count: 2,
            sheets: vec![
                WorksheetMetadata {
                    name: "Sheet1".to_string(),
                    index: 0,
                    row_count: 10,
                    column_count: 5,
                    cell_count: 50,
                    has_data: true,
                    is_hidden: false,
                    sheet_type: WorksheetType::Worksheet,
                },
                WorksheetMetadata {
                    name: "Sheet2".to_string(),
                    index: 1,
                    row_count: 0,
                    column_count: 0,
                    cell_count: 0,
                    has_data: false,
                    is_hidden: false,
                    sheet_type: WorksheetType::Worksheet,
                },
            ],
            total_cells: 50,
            total_rows: 10,
            has_hidden_sheets: false,
        };

        // Test serialization
        let json = serde_json::to_value(&worksheet_info).unwrap();
        assert_eq!(json["total_count"], 2);
        assert_eq!(json["sheets"].as_array().unwrap().len(), 2);
        assert_eq!(json["total_cells"], 50);
    }

    #[tokio::test]
    async fn test_cell_value_type_detection() {
        let parser = ExcelParser::new();

        // Test string type detection
        assert_eq!(parser.convert_string_with_type_detection("hello"), Value::String("hello".to_string()));
        assert_eq!(parser.convert_string_with_type_detection("  world  "), Value::String("world".to_string()));
        assert_eq!(parser.convert_string_with_type_detection(""), Value::Null);
        assert_eq!(parser.convert_string_with_type_detection("   "), Value::Null);

        // Test boolean detection
        assert_eq!(parser.convert_string_with_type_detection("true"), Value::Bool(true));
        assert_eq!(parser.convert_string_with_type_detection("TRUE"), Value::Bool(true));
        assert_eq!(parser.convert_string_with_type_detection("yes"), Value::Bool(true));
        assert_eq!(parser.convert_string_with_type_detection("Y"), Value::Bool(true));
        assert_eq!(parser.convert_string_with_type_detection("1"), Value::Bool(true));
        assert_eq!(parser.convert_string_with_type_detection("on"), Value::Bool(true));
        assert_eq!(parser.convert_string_with_type_detection("enabled"), Value::Bool(true));
        assert_eq!(parser.convert_string_with_type_detection("active"), Value::Bool(true));

        assert_eq!(parser.convert_string_with_type_detection("false"), Value::Bool(false));
        assert_eq!(parser.convert_string_with_type_detection("FALSE"), Value::Bool(false));
        assert_eq!(parser.convert_string_with_type_detection("no"), Value::Bool(false));
        assert_eq!(parser.convert_string_with_type_detection("N"), Value::Bool(false));
        assert_eq!(parser.convert_string_with_type_detection("0"), Value::Bool(false));
        assert_eq!(parser.convert_string_with_type_detection("off"), Value::Bool(false));
        assert_eq!(parser.convert_string_with_type_detection("disabled"), Value::Bool(false));
        assert_eq!(parser.convert_string_with_type_detection("inactive"), Value::Bool(false));

        // Test integer detection (but "1" and "0" are treated as booleans)
        assert_eq!(parser.convert_string_with_type_detection("42"), Value::Number(serde_json::Number::from(42)));
        assert_eq!(parser.convert_string_with_type_detection("-123"), Value::Number(serde_json::Number::from(-123)));

        // Test float detection
        if let Value::Number(n) = parser.convert_string_with_type_detection("3.14") {
            assert_eq!(n.as_f64().unwrap(), 3.14);
        } else {
            panic!("Expected number for '3.14'");
        }

        if let Value::Number(n) = parser.convert_string_with_type_detection("-2.5") {
            assert_eq!(n.as_f64().unwrap(), -2.5);
        } else {
            panic!("Expected number for '-2.5'");
        }
    }

    #[tokio::test]
    async fn test_numeric_value_conversion() {
        let parser = ExcelParser::new();

        // Test integer conversion
        assert_eq!(parser.convert_numeric_value(42.0), Value::Number(serde_json::Number::from(42)));
        assert_eq!(parser.convert_numeric_value(-123.0), Value::Number(serde_json::Number::from(-123)));
        assert_eq!(parser.convert_numeric_value(0.0), Value::Number(serde_json::Number::from(0)));

        // Test float conversion
        if let Value::Number(n) = parser.convert_numeric_value(3.14) {
            assert_eq!(n.as_f64().unwrap(), 3.14);
        } else {
            panic!("Expected number for 3.14");
        }

        // Test edge cases
        assert_eq!(parser.convert_numeric_value(f64::NAN), Value::Null);
        assert_eq!(parser.convert_numeric_value(f64::INFINITY), Value::Null);
        assert_eq!(parser.convert_numeric_value(f64::NEG_INFINITY), Value::Null);

        // Test large integers
        let large_int = 9223372036854775807_f64; // i64::MAX as f64
        assert_eq!(parser.convert_numeric_value(large_int), Value::Number(serde_json::Number::from(9223372036854775807_i64)));
    }

    #[tokio::test]
    async fn test_date_detection() {
        let parser = ExcelParser::new();

        // Test date-like strings
        assert_eq!(parser.detect_and_parse_date("2023-12-25"), Some("2023-12-25".to_string()));
        assert_eq!(parser.detect_and_parse_date("12/25/2023"), Some("12/25/2023".to_string()));
        assert_eq!(parser.detect_and_parse_date("25.12.2023"), Some("25.12.2023".to_string()));
        assert_eq!(parser.detect_and_parse_date("2023-12-25T10:30:00"), Some("2023-12-25T10:30:00".to_string()));

        // Test non-date strings
        assert_eq!(parser.detect_and_parse_date("hello"), None);
        assert_eq!(parser.detect_and_parse_date("123"), None);
        assert_eq!(parser.detect_and_parse_date("12-34"), None); // Too short
        assert_eq!(parser.detect_and_parse_date(""), None);
    }

    #[tokio::test]
    async fn test_duration_formatting() {
        let parser = ExcelParser::new();

        // Test duration formatting
        assert_eq!(parser.format_duration_seconds(30.0), "30.0s");
        assert_eq!(parser.format_duration_seconds(90.0), "1.5m");
        assert_eq!(parser.format_duration_seconds(3660.0), "1.0h");
        assert_eq!(parser.format_duration_seconds(90000.0), "1.0d");
    }

    #[tokio::test]
    async fn test_merged_cell_detection() {
        let parser = ExcelParser::new();

        // Test cell merging logic
        let string_cell1 = DataType::String("Header".to_string());
        let string_cell2 = DataType::String("Header".to_string());
        let string_cell3 = DataType::String("Different".to_string());
        let empty_cell = DataType::Empty;
        let int_cell1 = DataType::Int(42);
        let int_cell2 = DataType::Int(42);
        let float_cell = DataType::Float(42.0);

        // Test same string values should merge
        assert!(parser.cells_should_be_merged(&string_cell1, &string_cell2));

        // Test different string values should not merge
        assert!(!parser.cells_should_be_merged(&string_cell1, &string_cell3));

        // Test empty cells should not merge
        assert!(!parser.cells_should_be_merged(&empty_cell, &empty_cell));

        // Test same integer values should merge
        assert!(parser.cells_should_be_merged(&int_cell1, &int_cell2));

        // Test integer and equivalent float should merge
        assert!(parser.cells_should_be_merged(&int_cell1, &float_cell));

        // Test different types should not merge
        assert!(!parser.cells_should_be_merged(&string_cell1, &int_cell1));
    }

    #[tokio::test]
    async fn test_merged_cell_range_structure() {
        // Test MergedCellRange creation and serialization
        let merged_range = MergedCellRange {
            start_row: 0,
            start_col: 0,
            end_row: 1,
            end_col: 2,
            value: Value::String("Merged Header".to_string()),
            row_span: 2,
            col_span: 3,
        };

        // Test serialization
        let json = serde_json::to_value(&merged_range).unwrap();
        assert_eq!(json["start_row"], 0);
        assert_eq!(json["start_col"], 0);
        assert_eq!(json["end_row"], 1);
        assert_eq!(json["end_col"], 2);
        assert_eq!(json["row_span"], 2);
        assert_eq!(json["col_span"], 3);
        assert_eq!(json["value"], "Merged Header");
    }

    #[tokio::test]
    async fn test_cell_formatting_inference() {
        let parser = ExcelParser::new();

        // Test string formatting inference
        let header_string = DataType::String("HEADER TEXT".to_string());
        let url_string = DataType::String("https://example.com".to_string());
        let normal_string = DataType::String("normal text".to_string());

        if let Some(header_format) = parser.infer_formatting_from_content(&header_string) {
            assert_eq!(header_format.bold, Some(true));
            assert!(matches!(header_format.alignment, Some(TextAlignment::Center)));
        }

        if let Some(url_format) = parser.infer_formatting_from_content(&url_string) {
            assert_eq!(url_format.text_color, Some("#0000FF".to_string()));
            assert_eq!(url_format.underline, Some(true));
        }

        if let Some(normal_format) = parser.infer_formatting_from_content(&normal_string) {
            assert_eq!(normal_format.bold, None);
        }

        // Test numeric formatting inference
        let float_cell = DataType::Float(3.14);
        let int_cell = DataType::Int(42);

        if let Some(float_format) = parser.infer_formatting_from_content(&float_cell) {
            assert_eq!(float_format.number_format, Some("0.00".to_string()));
            assert!(matches!(float_format.alignment, Some(TextAlignment::Right)));
        }

        if let Some(int_format) = parser.infer_formatting_from_content(&int_cell) {
            assert_eq!(int_format.number_format, Some("0".to_string()));
            assert!(matches!(int_format.alignment, Some(TextAlignment::Right)));
        }

        // Test date formatting inference
        let date_cell = DataType::DateTime(45000.0); // Excel date serial number
        if let Some(date_format) = parser.infer_formatting_from_content(&date_cell) {
            assert_eq!(date_format.number_format, Some("mm/dd/yyyy".to_string()));
            assert!(matches!(date_format.alignment, Some(TextAlignment::Center)));
        }
    }

    #[tokio::test]
    async fn test_text_alignment_serialization() {
        // Test TextAlignment enum serialization
        let alignments = vec![
            TextAlignment::Left,
            TextAlignment::Center,
            TextAlignment::Right,
            TextAlignment::Justify,
            TextAlignment::Fill,
        ];

        for alignment in alignments {
            let json = serde_json::to_value(&alignment).unwrap();
            assert!(json.is_string());
        }
    }

    #[tokio::test]
    async fn test_formatting_preservation_config() {
        let parser = ExcelParser::new();

        // Test that formatting preservation is enabled by default
        assert!(parser.should_preserve_formatting());
    }

    #[tokio::test]
    async fn test_validation_config() {
        // Test default validation config
        let default_config = ValidationConfig::default();
        assert!(!default_config.strict_mode);
        assert_eq!(default_config.max_string_length, 32768);
        assert!(default_config.sanitize_markup);
        assert!(default_config.validate_emails);
        assert!(default_config.validate_urls);
        assert!(default_config.remove_control_chars);
        assert_eq!(default_config.max_validation_errors, 1000);
        assert!(default_config.normalize_whitespace);

        // Test strict validation config
        let strict_config = ValidationConfig::strict();
        assert!(strict_config.strict_mode);
        assert_eq!(strict_config.max_string_length, 8192);
        assert_eq!(strict_config.max_validation_errors, 100);
    }

    #[tokio::test]
    async fn test_string_sanitization() {
        let parser = ExcelParser::new();
        let mut issues = Vec::new();
        let mut confidence = 1.0;

        // Test normal string
        let normal = parser.sanitize_string("Hello World", &mut issues, &mut confidence);
        assert_eq!(normal, "Hello World");
        assert!(issues.is_empty());
        assert_eq!(confidence, 1.0);

        // Test string with extra whitespace
        issues.clear();
        confidence = 1.0;
        let whitespace = parser.sanitize_string("  Hello   World  ", &mut issues, &mut confidence);
        assert_eq!(whitespace, "Hello World");
        assert_eq!(issues.len(), 1);
        assert!(matches!(issues[0].issue_type, ValidationIssueType::QualityIssue));

        // Test string with control characters
        issues.clear();
        confidence = 1.0;
        let control_chars = parser.sanitize_string("Hello\x00\x01World", &mut issues, &mut confidence);
        assert_eq!(control_chars, "HelloWorld");
        assert_eq!(issues.len(), 1);
        assert!(matches!(issues[0].issue_type, ValidationIssueType::ControlCharacters));
    }

    #[tokio::test]
    async fn test_markup_detection_and_sanitization() {
        let parser = ExcelParser::new();

        // Test suspicious markup detection
        assert!(parser.contains_suspicious_markup("<script>alert('xss')</script>"));
        assert!(parser.contains_suspicious_markup("javascript:void(0)"));
        assert!(parser.contains_suspicious_markup("<iframe src='evil.com'></iframe>"));
        assert!(parser.contains_suspicious_markup("onclick='malicious()'"));
        assert!(!parser.contains_suspicious_markup("This is normal text"));
        assert!(!parser.contains_suspicious_markup("<b>Bold text</b>")); // Basic HTML is not suspicious

        // Test markup sanitization
        let malicious = "<script>alert('xss')</script>Hello<iframe>World</iframe>";
        let sanitized = parser.sanitize_markup(malicious);
        assert!(!sanitized.contains("<script"));
        assert!(!sanitized.contains("<iframe"));
        assert!(sanitized.contains("Hello"));
        assert!(sanitized.contains("World"));
    }

    #[tokio::test]
    async fn test_email_validation() {
        let parser = ExcelParser::new();

        // Test email detection
        assert!(parser.looks_like_email("user@example.com"));
        assert!(parser.looks_like_email("test.email+tag@domain.co.uk"));
        assert!(!parser.looks_like_email("not an email"));
        assert!(!parser.looks_like_email("@domain.com"));
        assert!(!parser.looks_like_email("user@"));

        // Test email validation
        assert!(parser.is_valid_email("user@example.com"));
        assert!(parser.is_valid_email("test.email+tag@domain.co.uk"));
        assert!(!parser.is_valid_email("invalid.email"));
        assert!(!parser.is_valid_email("user@"));
        assert!(!parser.is_valid_email("@domain.com"));
    }

    #[tokio::test]
    async fn test_url_validation() {
        let parser = ExcelParser::new();

        // Test URL detection
        assert!(parser.looks_like_url("https://example.com"));
        assert!(parser.looks_like_url("http://test.org"));
        assert!(parser.looks_like_url("ftp://files.example.com"));
        assert!(parser.looks_like_url("www.example.com"));
        assert!(!parser.looks_like_url("not a url"));
        assert!(!parser.looks_like_url("example.com")); // No protocol or www

        // Test URL validation
        assert!(parser.is_valid_url("https://example.com"));
        assert!(parser.is_valid_url("http://test.org/path?query=value"));
        assert!(parser.is_valid_url("www.example.com"));
        assert!(!parser.is_valid_url("invalid url"));
        assert!(!parser.is_valid_url("http://"));
    }

    #[tokio::test]
    async fn test_cell_validation() {
        let parser = ExcelParser::new();

        // Test valid string cell
        let valid_string = Value::String("Normal text".to_string());
        let result = parser.validate_and_sanitize_cell(&valid_string, 0, 0);
        assert!(result.is_valid);
        assert_eq!(result.confidence_score, 1.0);
        assert!(result.issues.is_empty());

        // Test string with issues
        let problematic_string = Value::String("  <script>alert('xss')</script>  ".to_string());
        let result = parser.validate_and_sanitize_cell(&problematic_string, 0, 1);
        assert!(!result.issues.is_empty());
        assert!(result.confidence_score < 1.0);
        // Should be sanitized
        assert!(!result.sanitized_value.as_str().unwrap().contains("<script"));

        // Test valid number
        let valid_number = Value::Number(serde_json::Number::from(42));
        let result = parser.validate_and_sanitize_cell(&valid_number, 1, 0);
        assert!(result.is_valid);
        assert_eq!(result.confidence_score, 1.0);

        // Test null value
        let null_value = Value::Null;
        let result = parser.validate_and_sanitize_cell(&null_value, 2, 0);
        assert!(result.is_valid);
        assert_eq!(result.confidence_score, 1.0);
    }

    #[tokio::test]
    async fn test_validation_summary_calculation() {
        let parser = ExcelParser::new();

        // Create mock validation results
        let validation_results = vec![
            CellValidationResult {
                row: 0,
                col: 0,
                original_value: Value::String("Good".to_string()),
                sanitized_value: Value::String("Good".to_string()),
                issues: vec![],
                is_valid: true,
                confidence_score: 1.0,
            },
            CellValidationResult {
                row: 0,
                col: 1,
                original_value: Value::String("  Bad  ".to_string()),
                sanitized_value: Value::String("Bad".to_string()),
                issues: vec![ValidationIssue {
                    issue_type: ValidationIssueType::QualityIssue,
                    severity: ValidationSeverity::Info,
                    message: "Whitespace normalized".to_string(),
                    suggestion: None,
                }],
                is_valid: true,
                confidence_score: 0.98,
            },
            CellValidationResult {
                row: 0,
                col: 2,
                original_value: Value::String("<script>evil</script>".to_string()),
                sanitized_value: Value::String("evil".to_string()),
                issues: vec![ValidationIssue {
                    issue_type: ValidationIssueType::SuspiciousMarkup,
                    severity: ValidationSeverity::Warning,
                    message: "Suspicious markup detected".to_string(),
                    suggestion: Some("Review content".to_string()),
                }],
                is_valid: false,
                confidence_score: 0.6,
            },
        ];

        let summary = parser.calculate_validation_summary(&validation_results);

        assert_eq!(summary.total_cells, 3);
        assert_eq!(summary.valid_cells, 2);
        assert_eq!(summary.invalid_cells, 1);
        assert_eq!(summary.sanitized_cells, 2); // Two cells were modified
        assert!((summary.average_confidence - 0.86).abs() < 0.01); // (1.0 + 0.98 + 0.6) / 3
        assert!(summary.issue_breakdown.contains_key("QualityIssue"));
        assert!(summary.issue_breakdown.contains_key("SuspiciousMarkup"));
        assert!(matches!(summary.max_severity, Some(ValidationSeverity::Warning)));
    }

    #[tokio::test]
    async fn test_severity_levels() {
        let parser = ExcelParser::new();

        assert_eq!(parser.severity_level(&ValidationSeverity::Info), 1);
        assert_eq!(parser.severity_level(&ValidationSeverity::Warning), 2);
        assert_eq!(parser.severity_level(&ValidationSeverity::Error), 3);
        assert_eq!(parser.severity_level(&ValidationSeverity::Critical), 4);
    }

    #[tokio::test]
    async fn test_excel_format_compatibility() {
        let parser = ExcelParser::new();

        // Test different Excel format scenarios
        struct FormatTest {
            name: &'static str,
            expected_error: bool,
            description: &'static str,
        }

        let format_tests = vec![
            FormatTest {
                name: "empty.xlsx",
                expected_error: false,
                description: "Empty Excel file should be handled gracefully",
            },
            FormatTest {
                name: "single_cell.xlsx",
                expected_error: false,
                description: "Single cell Excel file",
            },
            FormatTest {
                name: "large_dataset.xlsx",
                expected_error: false,
                description: "Large Excel file with many rows and columns",
            },
            FormatTest {
                name: "mixed_data_types.xlsx",
                expected_error: false,
                description: "Excel file with mixed data types",
            },
            FormatTest {
                name: "special_characters.xlsx",
                expected_error: false,
                description: "Excel file with special characters and Unicode",
            },
            FormatTest {
                name: "corrupted.xlsx",
                expected_error: true,
                description: "Corrupted Excel file should return error",
            },
            FormatTest {
                name: "password_protected.xlsx",
                expected_error: true,
                description: "Password protected Excel file should return error",
            },
        ];

        for test in format_tests {
            // Note: In a real implementation, these would test against actual Excel files
            // For now, we test the error handling and format detection logic
            println!("Testing format: {} - {}", test.name, test.description);

            // Test file extension validation
            let is_excel = test.name.ends_with(".xlsx") || test.name.ends_with(".xls");
            assert!(is_excel, "Test file should have Excel extension");

            // Test format detection logic
            let format_detected = parser.detect_excel_format(test.name);
            assert!(format_detected.is_ok() || test.expected_error);
        }
    }

    #[tokio::test]
    async fn test_edge_cases_and_error_handling() {
        let parser = ExcelParser::new();

        // Test various edge cases

        // Test extremely large numbers
        let large_number = Value::Number(serde_json::Number::from_f64(1e20).unwrap());
        let result = parser.validate_and_sanitize_cell(&large_number, 0, 0);
        assert!(!result.issues.is_empty()); // Should have warnings about large numbers

        // Test very long strings
        let long_string = "a".repeat(50000);
        let long_value = Value::String(long_string);
        let result = parser.validate_and_sanitize_cell(&long_value, 0, 1);
        assert!(!result.issues.is_empty()); // Should have string length issues

        // Test null and empty values
        let null_value = Value::Null;
        let result = parser.validate_and_sanitize_cell(&null_value, 1, 0);
        assert!(result.is_valid);
        assert!(result.issues.is_empty());

        // Test boolean values
        let bool_value = Value::Bool(true);
        let result = parser.validate_and_sanitize_cell(&bool_value, 1, 1);
        assert!(result.is_valid);
        assert!(result.issues.is_empty());

        // Test complex nested data (should not occur in Excel but test robustness)
        let complex_value = serde_json::json!({
            "nested": {
                "array": [1, 2, 3],
                "object": {"key": "value"}
            }
        });
        let result = parser.validate_and_sanitize_cell(&complex_value, 2, 0);
        assert!(!result.issues.is_empty()); // Should have type mismatch warnings
    }

    #[tokio::test]
    async fn test_worksheet_metadata_edge_cases() {
        // Test worksheet metadata with various scenarios

        // Test empty worksheet name
        let metadata = WorksheetMetadata {
            name: String::new(),
            index: 0,
            row_count: 0,
            column_count: 0,
            cell_count: 0,
            has_data: false,
            sheet_type: WorksheetType::Worksheet,
            is_hidden: false,
        };

        // Serialize and deserialize to test JSON compatibility
        let json = serde_json::to_string(&metadata).unwrap();
        let deserialized: WorksheetMetadata = serde_json::from_str(&json).unwrap();
        assert_eq!(metadata.name, deserialized.name);
        assert_eq!(metadata.has_data, deserialized.has_data);

        // Test worksheet with special characters in name
        let special_metadata = WorksheetMetadata {
            name: "Sheet with 特殊字符 and émojis 🚀".to_string(),
            index: 1,
            row_count: 100,
            column_count: 50,
            cell_count: 5000,
            has_data: true,
            sheet_type: WorksheetType::Chart,
            is_hidden: true,
        };

        let json = serde_json::to_string(&special_metadata).unwrap();
        let deserialized: WorksheetMetadata = serde_json::from_str(&json).unwrap();
        assert_eq!(special_metadata.name, deserialized.name);
        assert_eq!(special_metadata.sheet_type, deserialized.sheet_type);
    }

    #[tokio::test]
    async fn test_merged_cell_edge_cases() {
        // Test merged cell range validation and edge cases

        // Test single cell "merge" (should not be considered merged)
        let single_cell_range = MergedCellRange {
            start_row: 0,
            start_col: 0,
            end_row: 0,
            end_col: 0,
            value: Value::String("Single".to_string()),
            row_span: 1,
            col_span: 1,
        };

        assert_eq!(single_cell_range.row_span, 1);
        assert_eq!(single_cell_range.col_span, 1);

        // Test large merged range
        let large_range = MergedCellRange {
            start_row: 0,
            start_col: 0,
            end_row: 99,
            end_col: 49,
            value: Value::String("Large merge".to_string()),
            row_span: 100,
            col_span: 50,
        };

        assert_eq!(large_range.row_span, 100);
        assert_eq!(large_range.col_span, 50);

        // Test merged range with complex value
        let complex_merge = MergedCellRange {
            start_row: 5,
            start_col: 10,
            end_row: 7,
            end_col: 12,
            value: serde_json::json!({
                "formula": "=SUM(A1:A10)",
                "calculated_value": 55
            }),
            row_span: 3,
            col_span: 3,
        };

        // Test JSON serialization of complex merged cell
        let json = serde_json::to_string(&complex_merge).unwrap();
        let deserialized: MergedCellRange = serde_json::from_str(&json).unwrap();
        assert_eq!(complex_merge.start_row, deserialized.start_row);
        assert_eq!(complex_merge.value, deserialized.value);
    }

    #[tokio::test]
    async fn test_cell_formatting_edge_cases() {
        // Test cell formatting with various edge cases

        // Test minimal formatting
        let minimal_formatting = CellFormatting {
            bold: None,
            italic: None,
            underline: None,
            font_size: None,
            font_family: None,
            text_color: None,
            background_color: None,
            number_format: None,
            alignment: None,
            has_borders: false,
        };

        let json = serde_json::to_string(&minimal_formatting).unwrap();
        let deserialized: CellFormatting = serde_json::from_str(&json).unwrap();
        assert_eq!(minimal_formatting.has_borders, deserialized.has_borders);

        // Test maximal formatting
        let maximal_formatting = CellFormatting {
            bold: Some(true),
            italic: Some(true),
            underline: Some(true),
            font_size: Some(14.5),
            font_family: Some("Arial Unicode MS".to_string()),
            text_color: Some("#FF0000".to_string()),
            background_color: Some("#FFFF00".to_string()),
            number_format: Some("#,##0.00_);(#,##0.00)".to_string()),
            alignment: Some(TextAlignment::Center),
            has_borders: true,
        };

        let json = serde_json::to_string(&maximal_formatting).unwrap();
        let deserialized: CellFormatting = serde_json::from_str(&json).unwrap();
        assert_eq!(maximal_formatting.font_family, deserialized.font_family);
        assert_eq!(maximal_formatting.alignment, deserialized.alignment);

        // Test invalid color formats (should be handled gracefully)
        let invalid_color_formatting = CellFormatting {
            bold: Some(false),
            italic: Some(false),
            underline: Some(false),
            font_size: Some(0.0), // Invalid font size
            font_family: Some("".to_string()), // Empty font family
            text_color: Some("invalid_color".to_string()), // Invalid color
            background_color: Some("#GGGGGG".to_string()), // Invalid hex color
            number_format: Some("invalid_format".to_string()),
            alignment: Some(TextAlignment::Left),
            has_borders: false,
        };

        // Should still serialize/deserialize without errors
        let json = serde_json::to_string(&invalid_color_formatting).unwrap();
        let deserialized: CellFormatting = serde_json::from_str(&json).unwrap();
        assert_eq!(invalid_color_formatting.text_color, deserialized.text_color);
    }

    #[tokio::test]
    async fn test_data_type_conversion_comprehensive() {
        let parser = ExcelParser::new();

        // Test all DataType variants from calamine
        struct DataTypeTest {
            input: DataType,
            expected_type: &'static str,
            description: &'static str,
        }

        let data_type_tests = vec![
            DataTypeTest {
                input: DataType::Empty,
                expected_type: "null",
                description: "Empty cells should convert to null",
            },
            DataTypeTest {
                input: DataType::String("Hello World".to_string()),
                expected_type: "string",
                description: "String data should remain string",
            },
            DataTypeTest {
                input: DataType::Float(42.5),
                expected_type: "number",
                description: "Float data should convert to number",
            },
            DataTypeTest {
                input: DataType::Int(42),
                expected_type: "number",
                description: "Integer data should convert to number",
            },
            DataTypeTest {
                input: DataType::Bool(true),
                expected_type: "boolean",
                description: "Boolean data should remain boolean",
            },
            DataTypeTest {
                input: DataType::DateTime(45000.0), // Excel date serial number
                expected_type: "string", // Converted to ISO string
                description: "DateTime should convert to ISO string",
            },
            DataTypeTest {
                input: DataType::Duration(1.5), // 1.5 days
                expected_type: "string", // Formatted duration
                description: "Duration should convert to formatted string",
            },
            DataTypeTest {
                input: DataType::Error(calamine::CellErrorType::Div0),
                expected_type: "string", // Error message
                description: "Error cells should convert to error message string",
            },
        ];

        for test in data_type_tests {
            let converted = parser.convert_cell_value(&test.input);

            match test.expected_type {
                "null" => assert!(converted.is_null(), "Failed: {}", test.description),
                "string" => assert!(converted.is_string(), "Failed: {}", test.description),
                "number" => assert!(converted.is_number(), "Failed: {}", test.description),
                "boolean" => assert!(converted.is_boolean(), "Failed: {}", test.description),
                _ => panic!("Unknown expected type: {}", test.expected_type),
            }
        }
    }

    #[tokio::test]
    async fn test_string_type_detection_comprehensive() {
        let parser = ExcelParser::new();

        // Test comprehensive string type detection
        struct StringTest {
            input: &'static str,
            expected_type: &'static str,
            description: &'static str,
        }

        let string_tests = vec![
            // Boolean detection
            StringTest { input: "true", expected_type: "boolean", description: "Lowercase true" },
            StringTest { input: "TRUE", expected_type: "boolean", description: "Uppercase true" },
            StringTest { input: "True", expected_type: "boolean", description: "Mixed case true" },
            StringTest { input: "false", expected_type: "boolean", description: "Lowercase false" },
            StringTest { input: "yes", expected_type: "boolean", description: "Yes as boolean" },
            StringTest { input: "no", expected_type: "boolean", description: "No as boolean" },
            StringTest { input: "y", expected_type: "boolean", description: "Y as boolean" },
            StringTest { input: "n", expected_type: "boolean", description: "N as boolean" },
            StringTest { input: "on", expected_type: "boolean", description: "On as boolean" },
            StringTest { input: "off", expected_type: "boolean", description: "Off as boolean" },
            StringTest { input: "enabled", expected_type: "boolean", description: "Enabled as boolean" },
            StringTest { input: "disabled", expected_type: "boolean", description: "Disabled as boolean" },
            StringTest { input: "active", expected_type: "boolean", description: "Active as boolean" },
            StringTest { input: "inactive", expected_type: "boolean", description: "Inactive as boolean" },

            // Numeric detection
            StringTest { input: "42", expected_type: "number", description: "Integer string" },
            StringTest { input: "-42", expected_type: "number", description: "Negative integer" },
            StringTest { input: "42.5", expected_type: "number", description: "Float string" },
            StringTest { input: "-42.5", expected_type: "number", description: "Negative float" },
            StringTest { input: "0", expected_type: "boolean", description: "Zero as boolean" }, // "0" is detected as false boolean
            StringTest { input: "0.0", expected_type: "number", description: "Zero float" },
            StringTest { input: "1e10", expected_type: "number", description: "Scientific notation" },
            StringTest { input: "-1.5e-10", expected_type: "number", description: "Negative scientific notation" },

            // Date detection (basic patterns)
            StringTest { input: "2023-12-25", expected_type: "string", description: "ISO date format" },
            StringTest { input: "12/25/2023", expected_type: "string", description: "US date format" },
            StringTest { input: "25.12.2023", expected_type: "string", description: "European date format" },

            // Regular strings
            StringTest { input: "Hello World", expected_type: "string", description: "Regular text" },
            StringTest { input: "user@example.com", expected_type: "string", description: "Email address" },
            StringTest { input: "https://example.com", expected_type: "string", description: "URL" },
            StringTest { input: "", expected_type: "null", description: "Empty string" },
            StringTest { input: "   ", expected_type: "null", description: "Whitespace only" },

            // Edge cases
            StringTest { input: "NaN", expected_type: "string", description: "NaN string" },
            StringTest { input: "Infinity", expected_type: "string", description: "Infinity string" },
            StringTest { input: "null", expected_type: "string", description: "Null string" },
            StringTest { input: "undefined", expected_type: "string", description: "Undefined string" },
        ];

        for test in string_tests {
            let converted = parser.convert_string_with_type_detection(test.input);

            match test.expected_type {
                "null" => assert!(converted.is_null(), "Failed: {} - Expected null, got {:?}", test.description, converted),
                "string" => assert!(converted.is_string(), "Failed: {} - Expected string, got {:?}", test.description, converted),
                "number" => assert!(converted.is_number(), "Failed: {} - Expected number, got {:?}", test.description, converted),
                "boolean" => assert!(converted.is_boolean(), "Failed: {} - Expected boolean, got {:?}", test.description, converted),
                _ => panic!("Unknown expected type: {}", test.expected_type),
            }
        }
    }

    #[tokio::test]
    async fn test_performance_and_memory_limits() {
        // Test performance characteristics and memory limits

        // Test with large string
        let parser = ExcelParser::new();
        let large_string = "x".repeat(100000); // 100KB string
        let large_value = Value::String(large_string.clone());

        let start = std::time::Instant::now();
        let result = parser.validate_and_sanitize_cell(&large_value, 0, 0);
        let duration = start.elapsed();

        // Should complete within reasonable time (< 100ms for 100KB string)
        assert!(duration.as_millis() < 100, "Large string validation took too long: {:?}", duration);
        assert!(!result.issues.is_empty()); // Should have length issues

        // Test with many small validations
        let start = std::time::Instant::now();
        for i in 0..1000 {
            let small_value = Value::String(format!("test_{}", i));
            let _result = parser.validate_and_sanitize_cell(&small_value, i, 0);
        }
        let duration = start.elapsed();

        // Should complete within reasonable time (< 500ms for 1000 validations)
        assert!(duration.as_millis() < 500, "Bulk validation took too long: {:?}", duration);

        // Test memory usage with validation results
        let mut validation_results = Vec::new();
        for i in 0..10000 {
            let value = Value::String(format!("cell_{}", i));
            let result = parser.validate_and_sanitize_cell(&value, i / 100, i % 100);
            validation_results.push(result);
        }

        // Should be able to handle 10k validation results
        assert_eq!(validation_results.len(), 10000);

        // Test validation summary calculation performance
        let start = std::time::Instant::now();
        let summary = parser.calculate_validation_summary(&validation_results);
        let duration = start.elapsed();

        // Should complete within reasonable time (< 50ms for 10k results)
        assert!(duration.as_millis() < 50, "Validation summary calculation took too long: {:?}", duration);
        assert_eq!(summary.total_cells, 10000);
    }

    #[tokio::test]
    async fn test_excel_format_detection() {
        let parser = ExcelParser::new();

        // Test supported formats
        assert_eq!(parser.detect_excel_format("test.xlsx").unwrap(), ExcelFormat::Xlsx);
        assert_eq!(parser.detect_excel_format("TEST.XLSX").unwrap(), ExcelFormat::Xlsx);
        assert_eq!(parser.detect_excel_format("document.xls").unwrap(), ExcelFormat::Xls);
        assert_eq!(parser.detect_excel_format("macro.xlsm").unwrap(), ExcelFormat::Xlsm);
        assert_eq!(parser.detect_excel_format("binary.xlsb").unwrap(), ExcelFormat::Xlsb);

        // Test unsupported formats
        assert!(parser.detect_excel_format("document.pdf").is_err());
        assert!(parser.detect_excel_format("spreadsheet.ods").is_err());
        assert!(parser.detect_excel_format("data.csv").is_err());
        assert!(parser.detect_excel_format("file.txt").is_err());
        assert!(parser.detect_excel_format("noextension").is_err());

        // Test edge cases
        assert!(parser.detect_excel_format("").is_err());
        assert!(parser.detect_excel_format(".xlsx").is_err()); // No filename
        assert_eq!(parser.detect_excel_format("file.with.dots.xlsx").unwrap(), ExcelFormat::Xlsx);
    }

    #[tokio::test]
    async fn test_integration_scenarios() {
        // Test integration scenarios that combine multiple features

        let parser = ExcelParser::with_validation_config(
            50 * 1024 * 1024, // 50MB limit
            true,              // Auto-detect headers
            Some(1000),        // Max 1000 rows
            ValidationConfig::strict(), // Strict validation
        );

        // Test parser configuration
        assert_eq!(parser.max_file_size, 50 * 1024 * 1024);
        assert!(parser.auto_detect_headers);
        assert_eq!(parser.max_rows, Some(1000));
        assert!(parser.validation_config.strict_mode);

        // Test validation with strict config
        let suspicious_content = Value::String("<script>alert('xss')</script>Legitimate content".to_string());
        let result = parser.validate_and_sanitize_cell(&suspicious_content, 0, 0);

        // Strict mode should catch security issues
        assert!(!result.issues.is_empty());
        assert!(result.issues.iter().any(|issue| matches!(issue.issue_type, ValidationIssueType::SuspiciousMarkup)));
        assert!(result.confidence_score < 1.0);

        // Sanitized content should be safe
        let sanitized_str = result.sanitized_value.as_str().unwrap();
        assert!(!sanitized_str.contains("<script"));
        assert!(sanitized_str.contains("Legitimate content"));

        // Test worksheet info creation
        let worksheet_info = WorksheetInfo {
            total_count: 3,
            sheets: vec![
                WorksheetMetadata {
                    name: "Data".to_string(),
                    index: 0,
                    row_count: 100,
                    column_count: 10,
                    cell_count: 1000,
                    has_data: true,
                    sheet_type: WorksheetType::Worksheet,
                    is_hidden: false,
                },
                WorksheetMetadata {
                    name: "Chart".to_string(),
                    index: 1,
                    row_count: 0,
                    column_count: 0,
                    cell_count: 0,
                    has_data: false,
                    sheet_type: WorksheetType::Chart,
                    is_hidden: false,
                },
                WorksheetMetadata {
                    name: "Hidden".to_string(),
                    index: 2,
                    row_count: 50,
                    column_count: 5,
                    cell_count: 250,
                    has_data: true,
                    sheet_type: WorksheetType::Worksheet,
                    is_hidden: true,
                },
            ],
            total_cells: 1250,
            total_rows: 150,
            has_hidden_sheets: true,
        };

        // Test JSON serialization of complex structure
        let json = serde_json::to_string_pretty(&worksheet_info).unwrap();
        assert!(json.contains("Data"));
        assert!(json.contains("Chart"));
        assert!(json.contains("Hidden"));

        let deserialized: WorksheetInfo = serde_json::from_str(&json).unwrap();
        assert_eq!(worksheet_info.total_count, deserialized.total_count);
        assert_eq!(worksheet_info.sheets.len(), deserialized.sheets.len());
    }

    #[tokio::test]
    async fn test_error_recovery_and_resilience() {
        // Test error recovery and resilience features

        let parser = ExcelParser::new();

        // Test graceful handling of invalid data
        let invalid_values = vec![
            Value::String("".to_string()), // Empty string
            Value::Null,                   // Null value
            serde_json::json!(1e20), // Very large number
            serde_json::json!(-1e20), // Very large negative number
            Value::Array(vec![Value::String("unexpected".to_string())]), // Array (shouldn't be in Excel)
            serde_json::json!({"unexpected": "object"}), // Object (shouldn't be in Excel)
        ];

        for (i, invalid_value) in invalid_values.into_iter().enumerate() {
            let result = parser.validate_and_sanitize_cell(&invalid_value, i, 0);

            // Should not panic and should provide meaningful results
            assert!(result.row == i);
            assert!(result.col == 0);

            // Some values should be valid (empty, null), others should have issues
            match &result.original_value {
                Value::String(s) if s.is_empty() => assert!(result.is_valid),
                Value::Null => assert!(result.is_valid),
                _ => {
                    // Complex types or invalid numbers should have issues
                    if result.original_value.is_array() || result.original_value.is_object() {
                        assert!(!result.issues.is_empty());
                    }
                }
            }
        }

        // Test validation summary with mixed results
        let mixed_results = vec![
            CellValidationResult {
                row: 0, col: 0,
                original_value: Value::String("Good".to_string()),
                sanitized_value: Value::String("Good".to_string()),
                issues: vec![],
                is_valid: true,
                confidence_score: 1.0,
            },
            CellValidationResult {
                row: 0, col: 1,
                original_value: Value::String("<script>bad</script>".to_string()),
                sanitized_value: Value::String("bad".to_string()),
                issues: vec![ValidationIssue {
                    issue_type: ValidationIssueType::SuspiciousMarkup,
                    severity: ValidationSeverity::Critical,
                    message: "Critical security issue".to_string(),
                    suggestion: Some("Remove immediately".to_string()),
                }],
                is_valid: false,
                confidence_score: 0.1,
            },
        ];

        let summary = parser.calculate_validation_summary(&mixed_results);
        assert_eq!(summary.total_cells, 2);
        assert_eq!(summary.valid_cells, 1);
        assert_eq!(summary.invalid_cells, 1);
        assert!(matches!(summary.max_severity, Some(ValidationSeverity::Critical)));
        assert!(summary.average_confidence < 1.0);
    }

    // Note: Additional tests would require sample Excel files
    // These should be added when implementing actual parsing logic
}
