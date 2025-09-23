//! Modified: 2025-09-23

//! Core Excel parsing module
//!
//! This module provides the core Excel parsing functionality including file parsing,
//! worksheet detection, and data conversion. It contains the main Excel parser
//! implementation with comprehensive error handling, validation, and type safety.

// Module declarations
pub mod types;
pub mod file_parser;
pub mod worksheet_detector;
pub mod worksheet_parser;

// Re-export all public types and functions for backward compatibility
pub use types::*;
pub use file_parser::*;
pub use worksheet_detector::*;
pub use worksheet_parser::*;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::excel::types::ValidationConfig;

    #[test]
    fn test_excel_parser_integration() {
        let parser = ExcelParser::new();
        
        // Test basic configuration
        assert_eq!(parser.max_file_size(), 100 * 1024 * 1024);
        assert!(parser.auto_detect_headers());
        assert_eq!(parser.max_rows(), None);
        
        // Test supported extensions
        let extensions = parser.supported_extensions();
        assert!(extensions.contains(&"xlsx"));
        assert!(extensions.contains(&"xls"));
        assert!(extensions.contains(&"xlsm"));
        
        // Test file size validation
        assert!(parser.validate_file_size(50 * 1024 * 1024).is_ok());
        assert!(parser.validate_file_size(200 * 1024 * 1024).is_err());
    }

    #[test]
    fn test_excel_parser_builder() {
        let parser = ExcelParser::builder()
            .max_file_size(25 * 1024 * 1024)
            .auto_detect_headers(false)
            .max_rows(Some(1000))
            .validation_config(ValidationConfig::strict())
            .build();
        
        assert_eq!(parser.max_file_size(), 25 * 1024 * 1024);
        assert!(!parser.auto_detect_headers());
        assert_eq!(parser.max_rows(), Some(1000));
    }

    #[test]
    fn test_excel_parser_presets() {
        // Test large files preset
        let large_parser = ExcelParser::for_large_files();
        assert_eq!(large_parser.max_file_size(), 500 * 1024 * 1024);
        assert_eq!(large_parser.max_rows(), Some(100_000));
        
        // Test strict validation preset
        let strict_parser = ExcelParser::for_strict_validation();
        assert_eq!(strict_parser.max_file_size(), 50 * 1024 * 1024);
        
        // Test performance preset
        let perf_parser = ExcelParser::for_performance();
        assert_eq!(perf_parser.max_file_size(), 200 * 1024 * 1024);
        assert!(!perf_parser.auto_detect_headers());
        assert_eq!(perf_parser.max_rows(), Some(50_000));
        
        // Test development preset
        let dev_parser = ExcelParser::for_development();
        assert_eq!(dev_parser.max_file_size(), 10 * 1024 * 1024);
        assert_eq!(dev_parser.max_rows(), Some(1_000));
    }

    #[test]
    fn test_worksheet_detector_integration() {
        let detector = WorksheetDetector::new();
        
        // Test configuration
        assert!(detector.analyze_content);
        assert_eq!(detector.max_analysis_rows, 100);
        
        // Test pattern detection
        assert!(detector.is_header_like("user_id"));
        assert!(detector.is_summary_like("total amount"));
        assert!(detector.is_chart_like("chart title"));
    }

    #[test]
    fn test_worksheet_detector_with_config() {
        let detector = WorksheetDetector::with_config(false, 50);
        
        assert!(!detector.analyze_content);
        assert_eq!(detector.max_analysis_rows, 50);
    }

    #[test]
    fn test_processing_capability() {
        let parser = ExcelParser::new();
        
        // Test small file capability
        let capability = parser.can_process_file(1024 * 1024).unwrap(); // 1MB
        assert!(matches!(capability, ProcessingCapability::CanProcess { .. }));
        
        // Test large file capability
        let capability = parser.can_process_file(200 * 1024 * 1024).unwrap(); // 200MB
        assert!(matches!(capability, ProcessingCapability::TooLarge { .. }));
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
    fn test_format_detection() {
        let parser = ExcelParser::new();
        
        assert_eq!(parser.get_format_from_extension("xlsx"), Some(crate::excel::types::ExcelFormat::Xlsx));
        assert_eq!(parser.get_format_from_extension("xls"), Some(crate::excel::types::ExcelFormat::Xls));
        assert_eq!(parser.get_format_from_extension("pdf"), None);
        assert_eq!(parser.get_format_from_extension("txt"), None);
    }

    #[test]
    fn test_extension_support() {
        let parser = ExcelParser::new();
        
        assert!(parser.is_supported_extension("xlsx"));
        assert!(parser.is_supported_extension("XLSX"));
        assert!(parser.is_supported_extension("xls"));
        assert!(parser.is_supported_extension("xlsm"));
        assert!(parser.is_supported_extension("xltx"));
        assert!(parser.is_supported_extension("xltm"));
        assert!(!parser.is_supported_extension("pdf"));
        assert!(!parser.is_supported_extension("txt"));
        assert!(!parser.is_supported_extension("doc"));
    }

    #[test]
    fn test_parser_configuration_updates() {
        let mut parser = ExcelParser::new();
        
        // Test individual setters
        parser.set_max_file_size(75 * 1024 * 1024);
        assert_eq!(parser.max_file_size(), 75 * 1024 * 1024);
        
        parser.set_auto_detect_headers(false);
        assert!(!parser.auto_detect_headers());
        
        parser.set_max_rows(Some(2000));
        assert_eq!(parser.max_rows(), Some(2000));
        
        // Test validation config update
        let original_config = parser.validation_config().clone();
        parser.update_validation_config(|config| {
            *config = ValidationConfig::strict();
        });
        assert_ne!(parser.validation_config(), &original_config);
    }

    #[test]
    fn test_worksheet_statistics() {
        use serde_json::Value;
        
        let parser = ExcelParser::new();
        let worksheet_parser = WorksheetParser::new(&parser);
        
        // Create a mock worksheet with test data
        let data = vec![
            vec![
                Value::String("Name".to_string()),
                Value::String("Age".to_string()),
                Value::String("Date".to_string()),
            ],
            vec![
                Value::String("John".to_string()),
                Value::Number(serde_json::Number::from(25)),
                Value::String("2023-01-01".to_string()),
            ],
            vec![
                Value::String("Jane".to_string()),
                Value::Number(serde_json::Number::from(30)),
                Value::Null,
            ],
        ];
        
        let worksheet = crate::excel::types::ExcelWorksheet {
            name: "TestSheet".to_string(),
            row_count: 3,
            column_count: 3,
            data,
            headers: Some(vec!["Name".to_string(), "Age".to_string(), "Date".to_string()]),
            merged_cells: Vec::new(),
            cell_formatting: None,
            validation_results: Vec::new(),
            validation_summary: crate::excel::types::ValidationSummary {
                total_cells: 9,
                valid_cells: 8,
                invalid_cells: 0,
                sanitized_cells: 0,
                average_confidence: 0.9,
                issue_breakdown: std::collections::HashMap::new(),
                max_severity: None,
            },
        };
        
        let stats = worksheet_parser.get_worksheet_statistics(&worksheet);
        
        assert_eq!(stats.total_cells, 9);
        assert_eq!(stats.non_empty_cells, 8); // All except one null
        assert_eq!(stats.empty_cells, 1);
        assert_eq!(stats.numeric_cells, 2);
        assert_eq!(stats.text_cells, 4); // "Name", "John", "Jane", "Age" (header)
        assert_eq!(stats.date_cells, 1); // "2023-01-01"
        assert_eq!(stats.boolean_cells, 0);
        assert!((stats.data_density - (8.0 / 9.0)).abs() < 0.001);
    }

    #[test]
    fn test_column_extraction() {
        use serde_json::Value;
        
        let parser = ExcelParser::new();
        let worksheet_parser = WorksheetParser::new(&parser);
        
        let data = vec![
            vec![
                Value::String("A1".to_string()),
                Value::String("B1".to_string()),
                Value::String("C1".to_string()),
            ],
            vec![
                Value::String("A2".to_string()),
                Value::String("B2".to_string()),
                Value::String("C2".to_string()),
            ],
        ];
        
        let worksheet = crate::excel::types::ExcelWorksheet {
            name: "TestSheet".to_string(),
            row_count: 2,
            column_count: 3,
            data,
            headers: None,
            merged_cells: Vec::new(),
            cell_formatting: None,
            validation_results: Vec::new(),
            validation_summary: crate::excel::types::ValidationSummary::default(),
        };
        
        // Extract columns 0 and 2
        let extracted = worksheet_parser.extract_columns(&worksheet, &[0, 2]).unwrap();
        
        assert_eq!(extracted.len(), 2); // 2 rows
        assert_eq!(extracted[0].len(), 2); // 2 columns
        assert_eq!(extracted[0][0], Value::String("A1".to_string()));
        assert_eq!(extracted[0][1], Value::String("C1".to_string()));
        assert_eq!(extracted[1][0], Value::String("A2".to_string()));
        assert_eq!(extracted[1][1], Value::String("C2".to_string()));
    }

    #[test]
    fn test_date_detection_patterns() {
        let parser = ExcelParser::new();
        let worksheet_parser = WorksheetParser::new(&parser);
        
        // Test various date formats
        assert!(worksheet_parser.looks_like_date("2023-12-25"));
        assert!(worksheet_parser.looks_like_date("12/25/2023"));
        assert!(worksheet_parser.looks_like_date("12-25-2023"));
        assert!(worksheet_parser.looks_like_date("1/1/2023"));
        assert!(worksheet_parser.looks_like_date("01/01/2023"));
        
        // Test non-date strings
        assert!(!worksheet_parser.looks_like_date("not a date"));
        assert!(!worksheet_parser.looks_like_date("12345"));
        assert!(!worksheet_parser.looks_like_date("abc/def/ghi"));
        assert!(!worksheet_parser.looks_like_date(""));
    }
}
