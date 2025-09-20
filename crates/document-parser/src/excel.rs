// Modified: 2025-01-20

//! Excel document parsing implementation using calamine
//!
//! This module provides safe, memory-efficient Excel file parsing with comprehensive
//! error handling and type safety. All operations follow strict Rust guidelines
//! with explicit error handling and zero unsafe code.

use crate::{DocumentParser, ParseResult, DocumentType};
use async_trait::async_trait;
use calamine::{Reader, Xlsx, Xls, open_workbook_auto, DataType, Range};
use fedramp_core::{Result, Error};
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
        
        // Attempt to open workbook with automatic format detection
        let mut workbook = open_workbook_auto(cursor)
            .map_err(|e| Error::document_parsing(format!("Failed to open Excel workbook: {}", e)))?;

        // Get worksheet names
        let sheet_names = workbook.sheet_names().to_owned();
        if sheet_names.is_empty() {
            return Err(Error::document_parsing("No worksheets found in Excel file".to_string()));
        }

        info!("Found {} worksheets: {:?}", sheet_names.len(), sheet_names);

        // Parse all worksheets
        let mut worksheets = Vec::with_capacity(sheet_names.len());
        let mut validation_errors = Vec::new();

        for sheet_name in &sheet_names {
            match self.parse_worksheet(&mut workbook, sheet_name).await {
                Ok(worksheet) => {
                    debug!("Successfully parsed worksheet: {} ({} rows, {} cols)", 
                           worksheet.name, worksheet.row_count, worksheet.column_count);
                    worksheets.push(worksheet);
                }
                Err(e) => {
                    warn!("Failed to parse worksheet {}: {}", sheet_name, e);
                    validation_errors.push(format!("Worksheet '{}': {}", sheet_name, e));
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
        let quality_score = worksheets.len() as f64 / sheet_names.len() as f64;

        // Create metadata
        let metadata = serde_json::json!({
            "source_file": filename,
            "source_type": "excel",
            "extraction_date": chrono::Utc::now().to_rfc3339(),
            "worksheet_count": worksheets.len(),
            "total_worksheets": sheet_names.len(),
            "worksheet_names": sheet_names,
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
        workbook: &mut calamine::Sheets<R>,
        sheet_name: &str,
    ) -> Result<ExcelWorksheet> {
        // Get the range for the worksheet
        let range = workbook
            .worksheet_range(sheet_name)
            .map_err(|e| Error::document_parsing(format!("Failed to get worksheet range: {}", e)))?
            .ok_or_else(|| Error::document_parsing(format!("Worksheet '{}' is empty", sheet_name)))?;

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
                let cell_value = range.get_value((row_idx, col_idx))
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
