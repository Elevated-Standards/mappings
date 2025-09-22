// Modified: 2025-09-22

//! Common types and data structures for Excel parsing
//!
//! This module contains all the shared data structures, enums, and type definitions
//! used across the Excel parsing functionality.

use serde::{Serialize, Deserialize};
use serde_json::Value;
use chrono::{DateTime, Utc};
use uuid::Uuid;

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
    /// Detected worksheet type
    pub worksheet_type: WorksheetType,
    /// Confidence score for type detection
    pub type_confidence: f64,
}

/// Type of worksheet content
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum WorksheetType {
    /// Regular worksheet
    Worksheet,
    /// Chart sheet
    Chart,
    /// Macro sheet
    Macro,
    /// Dialog sheet
    Dialog,
    /// Unknown type
    Unknown,
}

/// Information about all worksheets in an Excel file
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorksheetInfo {
    /// Total number of worksheets
    pub total_count: usize,
    /// List of worksheet metadata
    pub sheets: Vec<WorksheetMetadata>,
    /// Total number of cells across all worksheets
    pub total_cells: usize,
    /// Total number of rows across all worksheets
    pub total_rows: usize,
    /// Whether any worksheets contain data
    pub has_data: bool,
    /// Detected file format
    pub format: ExcelFormat,
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
    /// Value in the merged range
    pub value: Option<Value>,
    /// Number of cells in the range
    pub cell_count: usize,
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
    /// Font name
    pub font_name: Option<String>,
    /// Text color (hex format)
    pub text_color: Option<String>,
    /// Background color (hex format)
    pub background_color: Option<String>,
    /// Number format
    pub number_format: Option<String>,
    /// Text alignment
    pub alignment: Option<TextAlignment>,
    /// Whether the cell is locked
    pub locked: Option<bool>,
    /// Whether the cell is hidden
    pub hidden: Option<bool>,
}

/// Text alignment options
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum TextAlignment {
    Left,
    Center,
    Right,
    Justify,
    Fill,
    CenterContinuous,
    Distributed,
}

/// Type alias for cell formatting map
pub type CellFormattingMap = std::collections::HashMap<(usize, usize), CellFormatting>;

/// Result of validating a single cell
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CellValidationResult {
    /// Row coordinate (0-based)
    pub row: usize,
    /// Column coordinate (0-based)
    pub column: usize,
    /// Original value before sanitization
    pub original_value: Option<Value>,
    /// Sanitized value
    pub sanitized_value: Option<Value>,
    /// Whether the cell passed validation
    pub is_valid: bool,
    /// List of validation issues found
    pub issues: Vec<ValidationIssue>,
    /// Confidence score for the validation
    pub confidence: f64,
}

/// Represents a validation issue found in a cell
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationIssue {
    /// Type of validation issue
    pub issue_type: ValidationIssueType,
    /// Severity of the issue
    pub severity: ValidationSeverity,
    /// Human-readable description
    pub message: String,
    /// Suggested fix or action
    pub suggestion: Option<String>,
    /// Whether the issue was automatically fixed
    pub auto_fixed: bool,
}

/// Types of validation issues that can be found
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum ValidationIssueType {
    /// String too long
    StringTooLong,
    /// Invalid characters detected
    InvalidCharacters,
    /// Potential injection attack
    PotentialInjection,
    /// Invalid date format
    InvalidDate,
    /// Invalid number format
    InvalidNumber,
    /// Missing required value
    MissingValue,
    /// Value outside expected range
    OutOfRange,
    /// Inconsistent data type
    InconsistentType,
    /// Suspicious pattern detected
    SuspiciousPattern,
    /// Encoding issue
    EncodingIssue,
}

/// Severity levels for validation issues
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
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
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum ExcelFormat {
    /// Excel 2007+ format (.xlsx)
    Xlsx,
    /// Excel 97-2003 format (.xls)
    Xls,
    /// Excel macro-enabled format (.xlsm)
    Xlsm,
    /// Excel template format (.xltx)
    Xltx,
    /// Excel macro-enabled template (.xltm)
    Xltm,
}

/// Configuration for Excel parsing validation and sanitization
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationConfig {
    /// Whether to enable strict validation
    pub strict_mode: bool,
    /// Maximum string length allowed
    pub max_string_length: usize,
    /// Whether to sanitize potentially dangerous content
    pub sanitize_content: bool,
    /// Whether to validate data types
    pub validate_types: bool,
    /// Whether to check for injection patterns
    pub check_injection: bool,
    /// Custom validation rules
    pub custom_rules: Vec<String>,
    /// Whether to auto-fix issues when possible
    pub auto_fix: bool,
}

impl ValidationConfig {
    /// Create default validation configuration
    #[must_use]
    pub fn default() -> Self {
        Self {
            strict_mode: false,
            max_string_length: 32_768, // 32KB max string length
            sanitize_content: true,
            validate_types: true,
            check_injection: true,
            custom_rules: Vec::new(),
            auto_fix: true,
        }
    }

    /// Create strict validation configuration
    #[must_use]
    pub fn strict() -> Self {
        Self {
            strict_mode: true,
            max_string_length: 8_192, // 8KB max in strict mode
            sanitize_content: true,
            validate_types: true,
            check_injection: true,
            custom_rules: Vec::new(),
            auto_fix: false, // Don't auto-fix in strict mode
        }
    }

    /// Create permissive validation configuration
    #[must_use]
    pub fn permissive() -> Self {
        Self {
            strict_mode: false,
            max_string_length: 65_536, // 64KB max in permissive mode
            sanitize_content: false,
            validate_types: false,
            check_injection: false,
            custom_rules: Vec::new(),
            auto_fix: true,
        }
    }
}

impl Default for ValidationConfig {
    fn default() -> Self {
        Self::default()
    }
}
