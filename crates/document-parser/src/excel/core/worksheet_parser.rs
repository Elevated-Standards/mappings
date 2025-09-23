//! Modified: 2025-09-23

//! Worksheet parsing implementation
//!
//! This module contains functionality for parsing individual worksheets,
//! including data conversion, validation, and header detection.

use crate::excel::types::*;
use crate::excel::validation::ExcelValidator;
use calamine::{Xlsx, DataType, Reader};
use fedramp_core::{Result, Error};
use serde_json::Value;
use tracing::{debug, warn};
use regex::Regex;

use super::types::ExcelParser;

/// Worksheet parser for processing individual Excel worksheets
#[derive(Debug)]
pub struct WorksheetParser<'a> {
    /// Reference to the parent Excel parser
    parser: &'a ExcelParser,
}

impl<'a> WorksheetParser<'a> {
    /// Create a new worksheet parser
    pub fn new(parser: &'a ExcelParser) -> Self {
        Self { parser }
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
            return Ok(self.create_empty_worksheet(sheet_name));
        }

        let dimensions = range.get_size();
        let row_count = dimensions.0;
        let column_count = dimensions.1;

        // Limit rows if configured
        let effective_row_count = if let Some(max_rows) = self.parser.max_rows {
            std::cmp::min(row_count, max_rows)
        } else {
            row_count
        };

        debug!("Worksheet dimensions: {}x{} (processing {}x{})",
               row_count, column_count, effective_row_count, column_count);

        // Convert range data to JSON values with validation
        let validator = ExcelValidator::new(self.parser.validation_config.clone());
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
        let headers = if self.parser.auto_detect_headers && !data.is_empty() {
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

    /// Create an empty worksheet structure
    fn create_empty_worksheet(&self, sheet_name: &str) -> ExcelWorksheet {
        ExcelWorksheet {
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
        }
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

    /// Parse worksheet with custom validation configuration
    pub async fn parse_worksheet_with_validation<R: std::io::Read + std::io::Seek>(
        &self,
        workbook: &mut Xlsx<R>,
        sheet_name: &str,
        validation_config: ValidationConfig,
    ) -> Result<ExcelWorksheet> {
        debug!("Parsing worksheet '{}' with custom validation", sheet_name);

        // Get worksheet range
        let range = workbook
            .worksheet_range(sheet_name)
            .ok_or_else(|| Error::document_parsing(format!("Worksheet '{}' not found", sheet_name)))?
            .map_err(|e| Error::document_parsing(format!("Failed to read worksheet '{}': {}", sheet_name, e)))?;

        if range.is_empty() {
            return Ok(self.create_empty_worksheet(sheet_name));
        }

        let dimensions = range.get_size();
        let effective_row_count = if let Some(max_rows) = self.parser.max_rows {
            std::cmp::min(dimensions.0, max_rows)
        } else {
            dimensions.0
        };

        // Use custom validator
        let validator = ExcelValidator::new(validation_config);
        let mut data = Vec::with_capacity(effective_row_count);
        let mut validation_results = Vec::new();

        for row_idx in 0..effective_row_count {
            let mut row_data = Vec::with_capacity(dimensions.1);

            for col_idx in 0..dimensions.1 {
                let cell_value = range.get_value((row_idx as u32, col_idx as u32));
                let json_value = self.convert_cell_to_json(cell_value);

                let validation_result = validator.validate_cell(&json_value, row_idx, col_idx);
                let sanitized_value = validation_result.sanitized_value.clone().unwrap_or(json_value);

                validation_results.push(validation_result);
                row_data.push(sanitized_value);
            }

            data.push(row_data);
        }

        let validation_summary = validator.generate_summary(&validation_results);
        let headers = if self.parser.auto_detect_headers && !data.is_empty() {
            self.detect_headers(&data[0])
        } else {
            None
        };

        Ok(ExcelWorksheet {
            name: sheet_name.to_string(),
            row_count: effective_row_count,
            column_count: dimensions.1,
            data,
            headers,
            merged_cells: Vec::new(),
            cell_formatting: None,
            validation_results,
            validation_summary,
        })
    }

    /// Extract specific columns from a worksheet
    pub fn extract_columns(&self, worksheet: &ExcelWorksheet, column_indices: &[usize]) -> Result<Vec<Vec<Value>>> {
        let mut extracted_data = Vec::new();

        for row in &worksheet.data {
            let mut extracted_row = Vec::new();
            
            for &col_idx in column_indices {
                if col_idx < row.len() {
                    extracted_row.push(row[col_idx].clone());
                } else {
                    extracted_row.push(Value::Null);
                }
            }
            
            extracted_data.push(extracted_row);
        }

        Ok(extracted_data)
    }

    /// Get worksheet statistics
    pub fn get_worksheet_statistics(&self, worksheet: &ExcelWorksheet) -> WorksheetStatistics {
        let mut non_empty_cells = 0;
        let mut numeric_cells = 0;
        let mut text_cells = 0;
        let mut date_cells = 0;
        let mut boolean_cells = 0;

        for row in &worksheet.data {
            for cell in row {
                match cell {
                    Value::Null => {}
                    Value::Number(_) => {
                        non_empty_cells += 1;
                        numeric_cells += 1;
                    }
                    Value::String(s) => {
                        if !s.trim().is_empty() {
                            non_empty_cells += 1;
                            // Simple date detection
                            if self.looks_like_date(s) {
                                date_cells += 1;
                            } else {
                                text_cells += 1;
                            }
                        }
                    }
                    Value::Bool(_) => {
                        non_empty_cells += 1;
                        boolean_cells += 1;
                    }
                    _ => {
                        non_empty_cells += 1;
                    }
                }
            }
        }

        let total_cells = worksheet.row_count * worksheet.column_count;
        let empty_cells = total_cells - non_empty_cells;

        WorksheetStatistics {
            total_cells,
            non_empty_cells,
            empty_cells,
            numeric_cells,
            text_cells,
            date_cells,
            boolean_cells,
            data_density: if total_cells > 0 {
                non_empty_cells as f64 / total_cells as f64
            } else {
                0.0
            },
        }
    }

    /// Simple date detection heuristic
    fn looks_like_date(&self, value: &str) -> bool {
        // Simple patterns for date detection
        let date_patterns = [
            r"\d{4}-\d{2}-\d{2}",     // YYYY-MM-DD
            r"\d{2}/\d{2}/\d{4}",     // MM/DD/YYYY
            r"\d{2}-\d{2}-\d{4}",     // MM-DD-YYYY
            r"\d{1,2}/\d{1,2}/\d{4}", // M/D/YYYY
        ];

        date_patterns.iter().any(|pattern| {
            Regex::new(pattern)
                .map(|re| re.is_match(value))
                .unwrap_or(false)
        })
    }
}

/// Statistics about a worksheet
#[derive(Debug, Clone)]
pub struct WorksheetStatistics {
    /// Total number of cells
    pub total_cells: usize,
    /// Number of non-empty cells
    pub non_empty_cells: usize,
    /// Number of empty cells
    pub empty_cells: usize,
    /// Number of numeric cells
    pub numeric_cells: usize,
    /// Number of text cells
    pub text_cells: usize,
    /// Number of date cells
    pub date_cells: usize,
    /// Number of boolean cells
    pub boolean_cells: usize,
    /// Data density (non-empty cells / total cells)
    pub data_density: f64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_header_detection() {
        let parser = ExcelParser::new();
        let worksheet_parser = WorksheetParser::new(&parser);
        
        let first_row = vec![
            Value::String("Name".to_string()),
            Value::String("Age".to_string()),
            Value::String("Email".to_string()),
        ];
        
        let headers = worksheet_parser.detect_headers(&first_row);
        assert!(headers.is_some());
        
        let headers = headers.unwrap();
        assert_eq!(headers.len(), 3);
        assert_eq!(headers[0], "Name");
        assert_eq!(headers[1], "Age");
        assert_eq!(headers[2], "Email");
    }

    #[test]
    fn test_date_detection() {
        let parser = ExcelParser::new();
        let worksheet_parser = WorksheetParser::new(&parser);
        
        assert!(worksheet_parser.looks_like_date("2023-12-25"));
        assert!(worksheet_parser.looks_like_date("12/25/2023"));
        assert!(worksheet_parser.looks_like_date("12-25-2023"));
        assert!(worksheet_parser.looks_like_date("1/1/2023"));
        assert!(!worksheet_parser.looks_like_date("not a date"));
        assert!(!worksheet_parser.looks_like_date("12345"));
    }

    #[test]
    fn test_empty_worksheet_creation() {
        let parser = ExcelParser::new();
        let worksheet_parser = WorksheetParser::new(&parser);
        
        let empty_worksheet = worksheet_parser.create_empty_worksheet("TestSheet");
        
        assert_eq!(empty_worksheet.name, "TestSheet");
        assert_eq!(empty_worksheet.row_count, 0);
        assert_eq!(empty_worksheet.column_count, 0);
        assert!(empty_worksheet.data.is_empty());
        assert!(empty_worksheet.headers.is_none());
    }
}
