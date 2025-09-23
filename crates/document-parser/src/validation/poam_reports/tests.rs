// Modified: 2025-09-23

//! Tests for POA&M validation reports

#[cfg(test)]
mod tests {
    use super::super::*;
    use crate::poam::PoamItem;
    use crate::validation::poam_validator::types::{PoamValidationResult, ValidationError, ValidationWarning};
    use chrono::Utc;
    use std::time::Duration;

    fn create_sample_poam_items() -> Vec<PoamItem> {
        vec![
            PoamItem {
                uuid: "test-1".to_string(),
                title: "Sample Vulnerability 1".to_string(),
                description: "Test vulnerability description".to_string(),
                status: "Open".to_string(),
                severity: Some("High".to_string()),
                scheduled_completion_date: Some("2024-12-31T23:59:59Z".to_string()),
                actual_completion_date: None,
                responsible_entity: Some("Security Team".to_string()),
                resources_required: Some("2 FTE".to_string()),
                risk_assessment: Some("High risk".to_string()),
                milestones: None,
            },
            PoamItem {
                uuid: "test-2".to_string(),
                title: "Sample Vulnerability 2".to_string(),
                description: "Another test vulnerability".to_string(),
                status: "In Progress".to_string(),
                severity: Some("Medium".to_string()),
                scheduled_completion_date: Some("2024-11-30T23:59:59Z".to_string()),
                actual_completion_date: None,
                responsible_entity: Some("IT Team".to_string()),
                resources_required: Some("1 FTE".to_string()),
                risk_assessment: Some("Medium risk".to_string()),
                milestones: None,
            },
        ]
    }

    fn create_sample_validation_results() -> Vec<PoamValidationResult> {
        vec![
            PoamValidationResult {
                is_valid: true,
                errors: Vec::new(),
                warnings: vec![
                    ValidationWarning {
                        field: "severity".to_string(),
                        message: "Consider reviewing severity level".to_string(),
                        severity: "Low".to_string(),
                        suggestion: Some("Review against current threat landscape".to_string()),
                    }
                ],
                suggestions: Vec::new(),
                field_results: Vec::new(),
                business_rule_results: Vec::new(),
                performance_metrics: crate::validation::poam_validator::types::ValidationPerformanceMetrics {
                    validation_time_ms: 10,
                    memory_usage_bytes: 1024,
                    rules_evaluated: 5,
                    cache_hits: 2,
                    cache_misses: 3,
                },
            },
            PoamValidationResult {
                is_valid: false,
                errors: vec![
                    ValidationError {
                        field: "scheduled_completion_date".to_string(),
                        message: "Invalid date format".to_string(),
                        severity: "High".to_string(),
                        suggestion: Some("Use ISO 8601 format".to_string()),
                    }
                ],
                warnings: Vec::new(),
                suggestions: Vec::new(),
                field_results: Vec::new(),
                business_rule_results: Vec::new(),
                performance_metrics: crate::validation::poam_validator::types::ValidationPerformanceMetrics {
                    validation_time_ms: 15,
                    memory_usage_bytes: 1536,
                    rules_evaluated: 5,
                    cache_hits: 1,
                    cache_misses: 4,
                },
            },
        ]
    }

    #[test]
    fn test_poam_report_generator_creation() {
        let config = PoamReportConfig::default();
        let generator = PoamReportGenerator::new(config);
        
        // Verify generator was created successfully
        assert_eq!(generator.get_metrics().total_reports_generated, 0);
    }

    #[test]
    fn test_generate_processing_summary_report() {
        let config = PoamReportConfig::default();
        let mut generator = PoamReportGenerator::new(config);
        
        let poam_items = create_sample_poam_items();
        let validation_results = create_sample_validation_results();
        
        let result = generator.generate_report(
            PoamReportType::ProcessingSummary,
            &poam_items,
            &validation_results,
            "test_document.xlsx"
        );
        
        assert!(result.is_ok());
        let report = result.unwrap();
        
        // Verify report structure
        assert_eq!(report.report_type, PoamReportType::ProcessingSummary);
        assert_eq!(report.processing_summary.total_items_processed, 2);
        assert_eq!(report.processing_summary.successful_items, 1);
        assert_eq!(report.processing_summary.items_with_errors, 1);
        assert_eq!(report.processing_summary.items_with_warnings, 1);
        
        // Verify document info
        assert_eq!(report.document_info.document_name, "test_document.xlsx");
        assert_eq!(report.document_info.document_format, "POA&M");
    }

    #[test]
    fn test_generate_detailed_validation_report() {
        let config = PoamReportConfig::default();
        let mut generator = PoamReportGenerator::new(config);
        
        let poam_items = create_sample_poam_items();
        let validation_results = create_sample_validation_results();
        
        let result = generator.generate_report(
            PoamReportType::DetailedValidation,
            &poam_items,
            &validation_results,
            "test_document.xlsx"
        );
        
        assert!(result.is_ok());
        let report = result.unwrap();
        
        // Verify validation results
        assert!(!report.validation_results.schema_validation.is_valid);
        assert_eq!(report.validation_results.schema_validation.errors.len(), 1);
        assert_eq!(report.validation_results.schema_validation.warnings.len(), 1);
    }

    #[test]
    fn test_report_export_json() {
        let config = ExportConfig::default();
        let mut exporter = PoamReportExporter::new(config);
        
        // Create a sample report
        let poam_items = create_sample_poam_items();
        let validation_results = create_sample_validation_results();
        
        let mut generator = PoamReportGenerator::new(PoamReportConfig::default());
        let report = generator.generate_report(
            PoamReportType::ProcessingSummary,
            &poam_items,
            &validation_results,
            "test_document.xlsx"
        ).unwrap();
        
        // Ensure output directory exists
        std::fs::create_dir_all("./reports").ok();
        
        let result = exporter.export_report(&report, PoamReportFormat::Json);
        assert!(result.is_ok());
        
        let filepath = result.unwrap();
        assert!(filepath.ends_with(".json"));
        
        // Verify file was created
        assert!(std::path::Path::new(&filepath).exists());
        
        // Clean up
        std::fs::remove_file(&filepath).ok();
    }

    #[test]
    fn test_report_export_html() {
        let config = ExportConfig::default();
        let mut exporter = PoamReportExporter::new(config);
        
        // Create a sample report
        let poam_items = create_sample_poam_items();
        let validation_results = create_sample_validation_results();
        
        let mut generator = PoamReportGenerator::new(PoamReportConfig::default());
        let report = generator.generate_report(
            PoamReportType::ProcessingSummary,
            &poam_items,
            &validation_results,
            "test_document.xlsx"
        ).unwrap();
        
        // Ensure output directory exists
        std::fs::create_dir_all("./reports").ok();
        
        let result = exporter.export_report(&report, PoamReportFormat::Html);
        assert!(result.is_ok());
        
        let filepath = result.unwrap();
        assert!(filepath.ends_with(".html"));
        
        // Verify file was created and contains expected content
        assert!(std::path::Path::new(&filepath).exists());
        let content = std::fs::read_to_string(&filepath).unwrap();
        assert!(content.contains("POA&M Validation Report"));
        assert!(content.contains("Processing Summary"));
        
        // Clean up
        std::fs::remove_file(&filepath).ok();
    }

    #[test]
    fn test_visualization_generation() {
        let config = VisualizationConfig::default();
        let engine = PoamVisualizationEngine::new(config);
        
        // Create a sample report
        let poam_items = create_sample_poam_items();
        let validation_results = create_sample_validation_results();
        
        let mut generator = PoamReportGenerator::new(PoamReportConfig::default());
        let report = generator.generate_report(
            PoamReportType::ProcessingSummary,
            &poam_items,
            &validation_results,
            "test_document.xlsx"
        ).unwrap();
        
        let result = engine.generate_visualizations(&report);
        assert!(result.is_ok());
        
        let visualizations = result.unwrap();
        assert!(!visualizations.is_empty());
        
        // Verify we have expected visualizations
        let chart_types: Vec<_> = visualizations.iter().map(|v| &v.chart_type).collect();
        assert!(chart_types.contains(&&ChartType::BarChart));
    }

    #[test]
    fn test_report_caching() {
        let mut config = PoamReportConfig::default();
        config.enable_caching = true;
        let mut generator = PoamReportGenerator::new(config);
        
        let poam_items = create_sample_poam_items();
        let validation_results = create_sample_validation_results();
        
        // Generate first report
        let report1 = generator.generate_report(
            PoamReportType::ProcessingSummary,
            &poam_items,
            &validation_results,
            "test_document.xlsx"
        ).unwrap();
        
        // Generate second report with same data (should use cache)
        let report2 = generator.generate_report(
            PoamReportType::ProcessingSummary,
            &poam_items,
            &validation_results,
            "test_document.xlsx"
        ).unwrap();
        
        // Reports should have different IDs but same content structure
        assert_ne!(report1.report_id, report2.report_id);
        assert_eq!(report1.processing_summary.total_items_processed, 
                  report2.processing_summary.total_items_processed);
        
        // Verify cache metrics
        let metrics = generator.get_metrics();
        assert!(metrics.cache_hits > 0 || metrics.cache_misses > 0);
    }

    #[test]
    fn test_recommendations_generation() {
        let config = PoamReportConfig::default();
        let mut generator = PoamReportGenerator::new(config);
        
        // Create items with low quality to trigger recommendations
        let mut poam_items = create_sample_poam_items();
        poam_items.push(PoamItem {
            uuid: "incomplete-item".to_string(),
            title: "".to_string(), // Missing title
            description: "".to_string(), // Missing description
            status: "".to_string(), // Missing status
            severity: None,
            scheduled_completion_date: None,
            actual_completion_date: None,
            responsible_entity: None,
            resources_required: None,
            risk_assessment: None,
            milestones: None,
        });
        
        let validation_results = create_sample_validation_results();
        
        let report = generator.generate_report(
            PoamReportType::ProcessingSummary,
            &poam_items,
            &validation_results,
            "test_document.xlsx"
        ).unwrap();
        
        // Should have recommendations due to low quality
        assert!(!report.recommendations.is_empty());
        
        // Verify recommendation structure
        let rec = &report.recommendations[0];
        assert!(!rec.title.is_empty());
        assert!(!rec.description.is_empty());
        assert!(!rec.actions.is_empty());
    }
}
