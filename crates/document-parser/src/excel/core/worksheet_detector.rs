//! Modified: 2025-09-23

//! Worksheet detection and enumeration
//!
//! This module contains functionality for detecting and analyzing worksheets
//! within Excel files, including metadata extraction and data presence detection.

use crate::excel::types::*;
use calamine::{Xlsx, Reader, Range, DataType};
use fedramp_core::{Result, Error};
use tracing::{debug, warn};

/// Worksheet detector for analyzing Excel file structure
#[derive(Debug, Clone)]
pub struct WorksheetDetector {
    /// Whether to analyze worksheet content for type detection
    analyze_content: bool,
    /// Maximum number of rows to analyze for type detection
    max_analysis_rows: usize,
}

impl WorksheetDetector {
    /// Create a new worksheet detector with default settings
    pub fn new() -> Self {
        Self {
            analyze_content: true,
            max_analysis_rows: 100,
        }
    }

    /// Create a worksheet detector with custom settings
    pub fn with_config(analyze_content: bool, max_analysis_rows: usize) -> Self {
        Self {
            analyze_content,
            max_analysis_rows,
        }
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

            let sheet_metadata = self.analyze_worksheet(workbook, sheet_name, index).await?;
            
            total_cells += sheet_metadata.cell_count;
            total_rows += sheet_metadata.row_count;
            
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

    /// Analyze a single worksheet to extract metadata
    async fn analyze_worksheet<R: std::io::Read + std::io::Seek>(
        &self,
        workbook: &mut Xlsx<R>,
        sheet_name: &str,
        index: usize,
    ) -> Result<WorksheetMetadata> {
        // Get worksheet range to determine size
        let range_result = workbook.worksheet_range(sheet_name);

        let (row_count, col_count, cell_count, has_data, worksheet_type, type_confidence) = match range_result {
            Some(Ok(range)) => {
                let dimensions = range.get_size();
                let rows = dimensions.0;
                let cols = dimensions.1;
                let cells = rows * cols;
                let has_data = !range.is_empty();
                
                // Analyze worksheet type if content analysis is enabled
                let (worksheet_type, type_confidence) = if self.analyze_content && has_data {
                    self.detect_worksheet_type(&range).await
                } else {
                    (WorksheetType::Worksheet, 1.0)
                };
                
                (rows, cols, cells, has_data, worksheet_type, type_confidence)
            }
            Some(Err(e)) => {
                warn!("Error reading worksheet '{}': {}", sheet_name, e);
                (0, 0, 0, false, WorksheetType::Worksheet, 0.0)
            }
            None => {
                warn!("Worksheet '{}' not found", sheet_name);
                (0, 0, 0, false, WorksheetType::Worksheet, 0.0)
            }
        };

        Ok(WorksheetMetadata {
            name: sheet_name.to_string(),
            index,
            row_count,
            column_count: col_count,
            cell_count,
            has_data,
            worksheet_type,
            type_confidence,
        })
    }

    /// Detect the type of worksheet based on content analysis
    async fn detect_worksheet_type(&self, range: &Range<DataType>) -> (WorksheetType, f64) {
        if range.is_empty() {
            return (WorksheetType::Worksheet, 1.0);
        }

        let dimensions = range.get_size();
        let rows_to_analyze = std::cmp::min(dimensions.0, self.max_analysis_rows);
        
        // Analyze patterns in the data
        let mut header_indicators = 0;
        let mut data_indicators = 0;
        let mut summary_indicators = 0;
        let mut chart_indicators = 0;

        // Check first few rows for patterns
        for row_idx in 0..std::cmp::min(5, rows_to_analyze) {
            for col_idx in 0..dimensions.1 {
                if let Some(cell) = range.get_value((row_idx as u32, col_idx as u32)) {
                    let cell_str = cell.to_string().to_lowercase();
                    
                    // Look for header-like content
                    if row_idx < 2 && self.is_header_like(&cell_str) {
                        header_indicators += 1;
                    }
                    
                    // Look for summary/calculation indicators
                    if self.is_summary_like(&cell_str) {
                        summary_indicators += 1;
                    }
                    
                    // Look for chart-related content
                    if self.is_chart_like(&cell_str) {
                        chart_indicators += 1;
                    }
                }
            }
        }

        // Count data rows (rows with mostly non-empty cells)
        for row_idx in 2..rows_to_analyze {
            let mut non_empty_cells = 0;
            for col_idx in 0..dimensions.1 {
                if let Some(cell) = range.get_value((row_idx as u32, col_idx as u32)) {
                    if !cell.to_string().trim().is_empty() {
                        non_empty_cells += 1;
                    }
                }
            }
            
            // If more than half the cells in a row are non-empty, consider it a data row
            if non_empty_cells > dimensions.1 / 2 {
                data_indicators += 1;
            }
        }

        // Determine worksheet type based on indicators
        let total_cells_analyzed = rows_to_analyze * dimensions.1;
        
        if chart_indicators > total_cells_analyzed / 10 {
            (WorksheetType::Chart, 0.8)
        } else if summary_indicators > total_cells_analyzed / 20 {
            (WorksheetType::Summary, 0.7)
        } else if header_indicators > 0 && data_indicators > 5 {
            (WorksheetType::Data, 0.9)
        } else if data_indicators > 0 {
            (WorksheetType::Worksheet, 0.8)
        } else {
            (WorksheetType::Worksheet, 0.5)
        }
    }

    /// Check if a cell value looks like a header
    fn is_header_like(&self, value: &str) -> bool {
        // Common header patterns
        let header_patterns = [
            "id", "name", "title", "description", "date", "time", "status",
            "type", "category", "value", "amount", "quantity", "price",
            "address", "email", "phone", "number", "code", "reference"
        ];
        
        header_patterns.iter().any(|pattern| value.contains(pattern))
    }

    /// Check if a cell value looks like summary/calculation content
    fn is_summary_like(&self, value: &str) -> bool {
        let summary_patterns = [
            "total", "sum", "average", "count", "max", "min", "subtotal",
            "grand total", "summary", "calculation", "formula"
        ];
        
        summary_patterns.iter().any(|pattern| value.contains(pattern))
    }

    /// Check if a cell value looks like chart-related content
    fn is_chart_like(&self, value: &str) -> bool {
        let chart_patterns = [
            "chart", "graph", "plot", "axis", "legend", "series",
            "x-axis", "y-axis", "data series"
        ];
        
        chart_patterns.iter().any(|pattern| value.contains(pattern))
    }

    /// Get statistics about the detected worksheets
    pub fn get_detection_statistics(&self, worksheet_info: &WorksheetInfo) -> DetectionStatistics {
        let mut type_counts = std::collections::HashMap::new();
        let mut total_confidence = 0.0;
        let mut sheets_with_data = 0;

        for sheet in &worksheet_info.sheets {
            *type_counts.entry(sheet.worksheet_type.clone()).or_insert(0) += 1;
            total_confidence += sheet.type_confidence;
            
            if sheet.has_data {
                sheets_with_data += 1;
            }
        }

        let average_confidence = if worksheet_info.sheets.is_empty() {
            0.0
        } else {
            total_confidence / worksheet_info.sheets.len() as f64
        };

        DetectionStatistics {
            total_worksheets: worksheet_info.total_count,
            sheets_with_data,
            type_distribution: type_counts,
            average_type_confidence: average_confidence,
            total_cells: worksheet_info.total_cells,
            total_rows: worksheet_info.total_rows,
        }
    }
}

impl Default for WorksheetDetector {
    fn default() -> Self {
        Self::new()
    }
}

/// Statistics about worksheet detection
#[derive(Debug, Clone)]
pub struct DetectionStatistics {
    /// Total number of worksheets
    pub total_worksheets: usize,
    /// Number of worksheets with data
    pub sheets_with_data: usize,
    /// Distribution of worksheet types
    pub type_distribution: std::collections::HashMap<WorksheetType, usize>,
    /// Average confidence in type detection
    pub average_type_confidence: f64,
    /// Total number of cells across all worksheets
    pub total_cells: usize,
    /// Total number of rows across all worksheets
    pub total_rows: usize,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_worksheet_detector_creation() {
        let detector = WorksheetDetector::new();
        assert!(detector.analyze_content);
        assert_eq!(detector.max_analysis_rows, 100);
    }

    #[test]
    fn test_worksheet_detector_with_config() {
        let detector = WorksheetDetector::with_config(false, 50);
        assert!(!detector.analyze_content);
        assert_eq!(detector.max_analysis_rows, 50);
    }

    #[test]
    fn test_header_detection() {
        let detector = WorksheetDetector::new();
        
        assert!(detector.is_header_like("user_id"));
        assert!(detector.is_header_like("Name"));
        assert!(detector.is_header_like("Email Address"));
        assert!(!detector.is_header_like("some random text"));
    }

    #[test]
    fn test_summary_detection() {
        let detector = WorksheetDetector::new();
        
        assert!(detector.is_summary_like("total amount"));
        assert!(detector.is_summary_like("Grand Total"));
        assert!(detector.is_summary_like("Average Score"));
        assert!(!detector.is_summary_like("regular data"));
    }

    #[test]
    fn test_chart_detection() {
        let detector = WorksheetDetector::new();
        
        assert!(detector.is_chart_like("chart title"));
        assert!(detector.is_chart_like("X-Axis"));
        assert!(detector.is_chart_like("Data Series"));
        assert!(!detector.is_chart_like("normal content"));
    }
}
