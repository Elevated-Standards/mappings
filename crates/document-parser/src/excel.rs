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

/// Represents a parsed Excel worksheet with metadata
#[derive(Debug, Clone)]
pub struct ExcelWorksheet {
    /// Name of the worksheet
    pub name: String,
    /// Number of rows with data
    pub row_count: usize,
    /// Number of columns with data
    pub column_count: usize,
    /// Raw data from the worksheet
    pub data: Vec<Vec<Value>>,
    /// Headers if detected
    pub headers: Option<Vec<String>>,
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
#[derive(Debug, Clone, Serialize, Deserialize)]
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

/// Excel file parser implementation
#[derive(Debug, Clone)]
pub struct ExcelParser {
    /// Maximum file size to process (in bytes)
    max_file_size: usize,
    /// Whether to detect headers automatically
    auto_detect_headers: bool,
    /// Maximum number of rows to process per worksheet
    max_rows: Option<usize>,
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

        // Create content with all worksheets
        let content = serde_json::json!({
            "worksheets": worksheets.into_iter().map(|ws| serde_json::json!({
                "name": ws.name,
                "row_count": ws.row_count,
                "column_count": ws.column_count,
                "headers": ws.headers,
                "data": ws.data
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
            });
        }

        // Apply row limit if specified
        let effective_height = self.max_rows.map_or(height, |max| height.min(max));
        
        debug!("Parsing range: {}x{} (limited to {}x{})", height, width, effective_height, width);

        // Parse data with type conversion
        let mut data = Vec::with_capacity(effective_height);
        let mut headers = None;

        for row_idx in 0..effective_height {
            let mut row_data = Vec::with_capacity(width);
            
            for col_idx in 0..width {
                let cell_value = range.get_value((row_idx as u32, col_idx as u32))
                    .map(|cell| self.convert_cell_value(cell))
                    .unwrap_or(Value::Null);
                
                row_data.push(cell_value);
            }
            
            // Detect headers from first row if enabled
            if row_idx == 0 && self.auto_detect_headers {
                headers = Some(self.extract_headers(&row_data));
            }
            
            data.push(row_data);
        }

        Ok(ExcelWorksheet {
            name: sheet_name.to_string(),
            row_count: effective_height,
            column_count: width,
            data,
            headers,
        })
    }

    /// Convert calamine DataType to serde_json Value with type safety
    ///
    /// # Arguments
    ///
    /// * `cell` - Calamine DataType to convert
    ///
    /// # Returns
    ///
    /// Returns appropriate JSON Value with proper type conversion
    #[must_use]
    fn convert_cell_value(&self, cell: &DataType) -> Value {
        match cell {
            DataType::Empty => Value::Null,
            DataType::String(s) => Value::String(s.trim().to_string()),
            DataType::Float(f) => {
                // Handle potential NaN or infinite values
                if f.is_finite() {
                    Value::Number(serde_json::Number::from_f64(*f).unwrap_or_else(|| {
                        warn!("Failed to convert float {} to JSON number, using 0", f);
                        serde_json::Number::from(0)
                    }))
                } else {
                    warn!("Encountered non-finite float value: {}, converting to null", f);
                    Value::Null
                }
            }
            DataType::Int(i) => Value::Number(serde_json::Number::from(*i)),
            DataType::Bool(b) => Value::Bool(*b),
            DataType::DateTime(dt) => {
                // Convert Excel datetime to ISO 8601 string
                Value::String(dt.to_string())
            }
            DataType::Error(e) => {
                warn!("Excel cell contains error: {:?}", e);
                Value::Null
            }
            DataType::DateTimeIso(dt_str) => Value::String(dt_str.clone()),
            DataType::DurationIso(dur_str) => Value::String(dur_str.clone()),
            DataType::Duration(dur) => Value::Number(serde_json::Number::from_f64(*dur).unwrap_or_else(|| {
                warn!("Failed to convert duration {} to JSON number, using 0", dur);
                serde_json::Number::from(0)
            })),
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
        assert_eq!(parser.max_rows, Some(1_000_000));
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

    // Note: Additional tests would require sample Excel files
    // These should be added when implementing actual parsing logic
}
