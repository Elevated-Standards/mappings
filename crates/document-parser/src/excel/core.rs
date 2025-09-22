// Modified: 2025-09-22

//! Core Excel parsing functionality
//!
//! This module provides the main Excel parsing implementation using calamine,
//! with comprehensive error handling, validation, and type safety.

use crate::{DocumentParser, ParseResult, DocumentType};
use crate::excel::types::*;
use crate::excel::validation::ExcelValidator;
use async_trait::async_trait;
use calamine::{Reader, Xlsx, DataType, Range};
use fedramp_core::{Result, Error};
use serde_json::{Value, Map};
use std::path::Path;
use tokio::fs;
use tracing::{debug, error, info, warn};
use chrono::{DateTime, Utc, NaiveDate};
use uuid::Uuid;

/// Main Excel parser implementation
#[derive(Debug, Clone)]
pub struct ExcelParser {
    /// Maximum file size to process (in bytes)
    max_file_size: usize,
    /// Whether to automatically detect headers
    auto_detect_headers: bool,
    /// Maximum number of rows to process per worksheet
    max_rows: Option<usize>,
    /// Validation configuration
    validation_config: ValidationConfig,
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

    /// Parse Excel data from byte array
    ///
    /// # Arguments
    ///
    /// * `data` - Excel file data as bytes
    /// * `filename` - Original filename for context
    ///
    /// # Returns
    ///
    /// Returns `Result<ParseResult>` with parsed data or detailed error information
    ///
    /// # Errors
    ///
    /// Returns error if Excel format is unsupported or file is corrupted
    pub async fn parse_excel_bytes(&self, data: &[u8], filename: &str) -> Result<ParseResult> {
        debug!("Parsing Excel data from bytes (size: {} bytes)", data.len());

        // Create cursor for reading
        let cursor = std::io::Cursor::new(data);
        
        // Try to open as Excel file
        let mut workbook = Xlsx::new(cursor)
            .map_err(|e| Error::document_parsing(format!("Failed to open Excel file: {}", e)))?;

        // Detect worksheets
        let worksheet_info = self.detect_worksheets(&mut workbook).await?;
        
        // Parse all worksheets
        let mut worksheets = Vec::new();
        let mut all_validation_errors = Vec::new();
        let mut total_quality_score = 0.0;

        for sheet_metadata in &worksheet_info.sheets {
            if !sheet_metadata.has_data {
                debug!("Skipping empty worksheet: {}", sheet_metadata.name);
                continue;
            }

            match self.parse_worksheet(&mut workbook, &sheet_metadata.name).await {
                Ok(worksheet) => {
                    total_quality_score += worksheet.validation_summary.average_confidence;
                    
                    // Collect validation errors
                    for result in &worksheet.validation_results {
                        for issue in &result.issues {
                            if issue.severity >= ValidationSeverity::Error {
                                all_validation_errors.push(format!(
                                    "Sheet '{}' Row {} Col {}: {}",
                                    worksheet.name,
                                    result.row + 1,
                                    result.column + 1,
                                    issue.message
                                ));
                            }
                        }
                    }
                    
                    worksheets.push(worksheet);
                }
                Err(e) => {
                    warn!("Failed to parse worksheet '{}': {}", sheet_metadata.name, e);
                    all_validation_errors.push(format!(
                        "Failed to parse worksheet '{}': {}",
                        sheet_metadata.name,
                        e
                    ));
                }
            }
        }

        // Calculate overall quality score
        let quality_score = if worksheets.is_empty() {
            0.0
        } else {
            total_quality_score / worksheets.len() as f64
        };

        // Create metadata
        let metadata = serde_json::json!({
            "file_info": {
                "filename": filename,
                "format": worksheet_info.format,
                "total_worksheets": worksheet_info.total_count,
                "parsed_worksheets": worksheets.len(),
                "total_cells": worksheet_info.total_cells,
                "total_rows": worksheet_info.total_rows
            },
            "parsing_info": {
                "parser_version": "1.0.0",
                "parsed_at": Utc::now().to_rfc3339(),
                "validation_config": self.validation_config
            }
        });

        // Create content with all worksheets
        let content = serde_json::json!({
            "worksheets": worksheets.iter().map(|ws| {
                serde_json::json!({
                    "name": ws.name,
                    "row_count": ws.row_count,
                    "column_count": ws.column_count,
                    "headers": ws.headers,
                    "data": ws.data,
                    "merged_cells": ws.merged_cells,
                    "validation_summary": ws.validation_summary
                })
            }).collect::<Vec<_>>()
        });

        Ok(ParseResult {
            document_type: DocumentType::Excel,
            source_path: filename.to_string(),
            metadata,
            content,
            validation_errors: all_validation_errors,
            quality_score,
        })
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
                    
                    total_cells += cells;
                    total_rows += rows;
                    
                    (rows, cols, cells, has_data)
                }
                Some(Err(e)) => {
                    warn!("Error reading worksheet '{}': {}", sheet_name, e);
                    (0, 0, 0, false)
                }
                None => {
                    warn!("Worksheet '{}' not found", sheet_name);
                    (0, 0, 0, false)
                }
            };

            let sheet_metadata = WorksheetMetadata {
                name: sheet_name.clone(),
                index,
                row_count,
                column_count: col_count,
                cell_count,
                has_data,
                worksheet_type: WorksheetType::Worksheet, // Default to worksheet
                type_confidence: 1.0,
            };

            sheets.push(sheet_metadata);
        }

        let has_data = sheets.iter().any(|s| s.has_data);

        Ok(WorksheetInfo {
            total_count: sheet_names.len(),
            sheets,
            total_cells,
            total_rows,
            has_data,
            format: ExcelFormat::Xlsx, // Assume XLSX for now
        })
    }

    /// Parse a single worksheet from the Excel file
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
    /// Returns error if worksheet cannot be read or parsed
    pub async fn parse_worksheet<R: std::io::Read + std::io::Seek>(
        &self,
        workbook: &mut Xlsx<R>,
        sheet_name: &str,
    ) -> Result<ExcelWorksheet> {
        debug!("Parsing worksheet: {}", sheet_name);

        // Get worksheet range
        let range = workbook
            .worksheet_range(sheet_name)
            .ok_or_else(|| Error::document_parsing(format!("Worksheet '{}' not found", sheet_name)))?
            .map_err(|e| Error::document_parsing(format!("Failed to read worksheet '{}': {}", sheet_name, e)))?;

        if range.is_empty() {
            debug!("Worksheet '{}' is empty", sheet_name);
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

        let dimensions = range.get_size();
        let row_count = dimensions.0;
        let column_count = dimensions.1;

        // Limit rows if configured
        let effective_row_count = if let Some(max_rows) = self.max_rows {
            std::cmp::min(row_count, max_rows)
        } else {
            row_count
        };

        debug!("Worksheet dimensions: {}x{} (processing {}x{})",
               row_count, column_count, effective_row_count, column_count);

        // Convert range data to JSON values with validation
        let validator = ExcelValidator::new(self.validation_config.clone());
        let mut data = Vec::with_capacity(effective_row_count);
        let mut validation_results = Vec::new();

        for row_idx in 0..effective_row_count {
            let mut row_data = Vec::with_capacity(column_count);

            for col_idx in 0..column_count {
                let cell_value = range.get_value((row_idx as u32, col_idx as u32));
                let json_value = self.convert_cell_to_json(cell_value);

                // Validate the cell
                let validation_result = validator.validate_cell(&json_value, row_idx, col_idx);
                let sanitized_value = validation_result.sanitized_value.clone().unwrap_or(json_value);

                validation_results.push(validation_result);
                row_data.push(sanitized_value);
            }

            data.push(row_data);
        }

        // Generate validation summary
        let validation_summary = validator.generate_summary(&validation_results);

        // Detect headers if enabled
        let headers = if self.auto_detect_headers && !data.is_empty() {
            self.detect_headers(&data[0])
        } else {
            None
        };

        // TODO: Detect merged cells (requires additional calamine features)
        let merged_cells = Vec::new();

        // TODO: Extract cell formatting (requires additional calamine features)
        let cell_formatting = None;

        Ok(ExcelWorksheet {
            name: sheet_name.to_string(),
            row_count: effective_row_count,
            column_count,
            data,
            headers,
            merged_cells,
            cell_formatting,
            validation_results,
            validation_summary,
        })
    }

    /// Convert a calamine DataType to a JSON Value
    fn convert_cell_to_json(&self, cell: Option<&DataType>) -> Value {
        match cell {
            Some(DataType::Int(i)) => Value::Number(serde_json::Number::from(*i)),
            Some(DataType::Float(f)) => {
                if let Some(num) = serde_json::Number::from_f64(*f) {
                    Value::Number(num)
                } else {
                    Value::Null
                }
            }
            Some(DataType::String(s)) => Value::String(s.clone()),
            Some(DataType::Bool(b)) => Value::Bool(*b),
            Some(DataType::DateTime(dt)) => {
                // Convert Excel datetime to ISO string
                Value::String(dt.to_string())
            }
            Some(DataType::Error(e)) => {
                warn!("Excel cell error: {:?}", e);
                Value::Null
            }
            Some(DataType::Duration(dur)) => {
                Value::String(format!("{}s", dur))
            }
            Some(DataType::DateTimeIso(dt_str)) => {
                Value::String(dt_str.clone())
            }
            Some(DataType::DurationIso(dur_str)) => {
                Value::String(dur_str.clone())
            }
            Some(DataType::Empty) | None => Value::Null,
        }
    }

    /// Attempt to detect headers from the first row
    fn detect_headers(&self, first_row: &[Value]) -> Option<Vec<String>> {
        // Simple heuristic: if all values in first row are strings, treat as headers
        let all_strings = first_row.iter().all(|v| matches!(v, Value::String(_)));

        if all_strings && !first_row.is_empty() {
            let headers: Vec<String> = first_row
                .iter()
                .enumerate()
                .map(|(i, v)| {
                    match v {
                        Value::String(s) if !s.trim().is_empty() => s.trim().to_string(),
                        _ => format!("Column_{}", i + 1),
                    }
                })
                .collect();
            Some(headers)
        } else {
            None
        }
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

    /// Validate parsed Excel content
    async fn validate(&self, content: &Value) -> Result<Vec<String>> {
        let mut errors = Vec::new();

        // Basic validation of the content structure
        if let Some(worksheets) = content.get("worksheets").and_then(|v| v.as_array()) {
            for (i, worksheet) in worksheets.iter().enumerate() {
                if !worksheet.is_object() {
                    errors.push(format!("Worksheet {} is not a valid object", i));
                    continue;
                }

                // Validate required fields
                if worksheet.get("name").is_none() {
                    errors.push(format!("Worksheet {} missing required 'name' field", i));
                }

                if worksheet.get("data").is_none() {
                    errors.push(format!("Worksheet {} missing required 'data' field", i));
                }
            }
        } else {
            errors.push("Content missing 'worksheets' array".to_string());
        }

        Ok(errors)
    }

    /// Get supported file extensions for Excel files
    fn supported_extensions(&self) -> Vec<&'static str> {
        vec!["xlsx", "xls", "xlsm", "xltx", "xltm"]
    }
}
