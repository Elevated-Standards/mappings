//! Modified: 2025-09-23

//! Excel file parsing implementation
//!
//! This module contains the implementation for parsing Excel files from paths
//! and byte arrays with comprehensive error handling and validation.

use crate::{DocumentParser, ParseResult, DocumentType};
use crate::excel::types::*;
use async_trait::async_trait;
use calamine::{Reader, Xlsx};
use fedramp_core::{Result, Error};
use std::path::Path;
use tokio::fs;
use tracing::{debug, info, warn};
use chrono::Utc;

use super::types::ExcelParser;
use super::worksheet_detector::WorksheetDetector;
use super::worksheet_parser::WorksheetParser;

impl ExcelParser {
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
        let detector = WorksheetDetector::new();
        let worksheet_info = detector.detect_worksheets(&mut workbook).await?;
        
        // Parse all worksheets
        let parser = WorksheetParser::new(self);
        let mut worksheets = Vec::new();
        let mut all_validation_errors = Vec::new();
        let mut total_quality_score = 0.0;

        for sheet_metadata in &worksheet_info.sheets {
            if !sheet_metadata.has_data {
                debug!("Skipping empty worksheet: {}", sheet_metadata.name);
                continue;
            }

            match parser.parse_worksheet(&mut workbook, &sheet_metadata.name).await {
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

    /// Validate file size before processing
    pub fn validate_file_size(&self, file_size: u64) -> Result<()> {
        if file_size > self.max_file_size as u64 {
            return Err(Error::document_parsing(format!(
                "File size {} exceeds maximum limit of {} bytes",
                file_size,
                self.max_file_size
            )));
        }
        Ok(())
    }

    /// Check if a file extension is supported
    pub fn is_supported_extension(&self, extension: &str) -> bool {
        let supported = ["xlsx", "xls", "xlsm", "xltx", "xltm"];
        supported.contains(&extension.to_lowercase().as_str())
    }

    /// Get file format from extension
    pub fn get_format_from_extension(&self, extension: &str) -> Option<ExcelFormat> {
        match extension.to_lowercase().as_str() {
            "xlsx" | "xlsm" | "xltx" | "xltm" => Some(ExcelFormat::Xlsx),
            "xls" => Some(ExcelFormat::Xls),
            _ => None,
        }
    }

    /// Estimate memory usage for a file
    pub fn estimate_memory_usage(&self, file_size: u64) -> usize {
        // Rough estimate: Excel files expand 2-3x in memory
        // Add overhead for validation and processing
        (file_size as usize * 3) + (10 * 1024 * 1024) // 10MB overhead
    }

    /// Check if file can be processed with current settings
    pub fn can_process_file(&self, file_size: u64) -> Result<ProcessingCapability> {
        if file_size > self.max_file_size as u64 {
            return Ok(ProcessingCapability::TooLarge {
                file_size,
                max_size: self.max_file_size as u64,
            });
        }

        let estimated_memory = self.estimate_memory_usage(file_size);
        let available_memory = get_available_memory();

        if estimated_memory > available_memory {
            return Ok(ProcessingCapability::InsufficientMemory {
                required: estimated_memory,
                available: available_memory,
            });
        }

        Ok(ProcessingCapability::CanProcess {
            estimated_memory,
            processing_time_estimate: estimate_processing_time(file_size),
        })
    }
}

/// Processing capability assessment
#[derive(Debug, Clone)]
pub enum ProcessingCapability {
    /// File can be processed
    CanProcess {
        estimated_memory: usize,
        processing_time_estimate: std::time::Duration,
    },
    /// File is too large
    TooLarge {
        file_size: u64,
        max_size: u64,
    },
    /// Insufficient memory
    InsufficientMemory {
        required: usize,
        available: usize,
    },
}

/// Get available system memory (rough estimate)
fn get_available_memory() -> usize {
    // This is a simplified implementation
    // In a real system, you'd use system APIs to get actual available memory
    1024 * 1024 * 1024 // 1GB default
}

/// Estimate processing time based on file size
fn estimate_processing_time(file_size: u64) -> std::time::Duration {
    // Rough estimate: 1MB per second
    let seconds = (file_size / (1024 * 1024)).max(1);
    std::time::Duration::from_secs(seconds)
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
    async fn validate(&self, content: &serde_json::Value) -> Result<Vec<String>> {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_file_size_validation() {
        let parser = ExcelParser::new();
        
        // Test valid file size
        assert!(parser.validate_file_size(50 * 1024 * 1024).is_ok());
        
        // Test file size too large
        assert!(parser.validate_file_size(200 * 1024 * 1024).is_err());
    }

    #[test]
    fn test_supported_extensions() {
        let parser = ExcelParser::new();
        
        assert!(parser.is_supported_extension("xlsx"));
        assert!(parser.is_supported_extension("XLSX"));
        assert!(parser.is_supported_extension("xls"));
        assert!(parser.is_supported_extension("xlsm"));
        assert!(!parser.is_supported_extension("pdf"));
        assert!(!parser.is_supported_extension("txt"));
    }

    #[test]
    fn test_format_detection() {
        let parser = ExcelParser::new();
        
        assert_eq!(parser.get_format_from_extension("xlsx"), Some(ExcelFormat::Xlsx));
        assert_eq!(parser.get_format_from_extension("xls"), Some(ExcelFormat::Xls));
        assert_eq!(parser.get_format_from_extension("pdf"), None);
    }

    #[test]
    fn test_memory_estimation() {
        let parser = ExcelParser::new();
        
        let file_size = 10 * 1024 * 1024; // 10MB
        let estimated = parser.estimate_memory_usage(file_size);
        
        // Should be roughly 3x file size plus overhead
        assert!(estimated > file_size as usize * 2);
        assert!(estimated < file_size as usize * 5);
    }

    #[test]
    fn test_processing_capability() {
        let parser = ExcelParser::new();
        
        // Test small file
        let capability = parser.can_process_file(1024 * 1024).unwrap(); // 1MB
        assert!(matches!(capability, ProcessingCapability::CanProcess { .. }));
        
        // Test large file
        let capability = parser.can_process_file(200 * 1024 * 1024).unwrap(); // 200MB
        assert!(matches!(capability, ProcessingCapability::TooLarge { .. }));
    }
}
