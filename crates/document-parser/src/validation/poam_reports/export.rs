// Modified: 2025-09-23

//! POA&M validation report export functionality
//!
//! This module provides export capabilities for POA&M validation reports
//! in various formats including HTML, PDF, JSON, CSV, Markdown, and Excel.

use super::types::*;
use fedramp_core::{Result, Error};
use serde_json;
use tracing::{debug, info, warn};
use std::collections::HashMap;

/// POA&M report exporter for multiple formats
#[derive(Debug)]
pub struct PoamReportExporter {
    /// Export configuration
    config: ExportConfig,
    /// Template cache for performance
    template_cache: HashMap<PoamReportFormat, String>,
}

/// Export configuration
#[derive(Debug, Clone)]
pub struct ExportConfig {
    /// Output directory for exported reports
    pub output_directory: String,
    /// Include styling and formatting
    pub include_styling: bool,
    /// Compress output files
    pub compress_output: bool,
    /// Maximum file size (bytes)
    pub max_file_size: u64,
    /// Custom templates directory
    pub templates_directory: Option<String>,
}

impl Default for ExportConfig {
    fn default() -> Self {
        Self {
            output_directory: "./reports".to_string(),
            include_styling: true,
            compress_output: false,
            max_file_size: 50 * 1024 * 1024, // 50MB
            templates_directory: None,
        }
    }
}

impl PoamReportExporter {
    /// Create a new POA&M report exporter
    pub fn new(config: ExportConfig) -> Self {
        Self {
            config,
            template_cache: HashMap::new(),
        }
    }

    /// Export report to specified format
    pub fn export_report(
        &mut self,
        report: &PoamValidationReport,
        format: PoamReportFormat,
    ) -> Result<String> {
        info!("Exporting POA&M report {} to {:?} format", report.report_id, format);

        match format {
            PoamReportFormat::Html => self.export_to_html(report),
            PoamReportFormat::Pdf => self.export_to_pdf(report),
            PoamReportFormat::Json => self.export_to_json(report),
            PoamReportFormat::Csv => self.export_to_csv(report),
            PoamReportFormat::Markdown => self.export_to_markdown(report),
            PoamReportFormat::Excel => self.export_to_excel(report),
        }
    }

    /// Export report to HTML format
    fn export_to_html(&mut self, report: &PoamValidationReport) -> Result<String> {
        let template = self.get_html_template()?;
        let content = self.render_html_content(report, &template)?;
        
        let filename = format!("poam_report_{}.html", report.report_id);
        let filepath = format!("{}/{}", self.config.output_directory, filename);
        
        std::fs::write(&filepath, content)
            .map_err(|e| Error::document_parsing(format!("Failed to write HTML report: {}", e)))?;
        
        Ok(filepath)
    }

    /// Export report to PDF format
    fn export_to_pdf(&mut self, report: &PoamValidationReport) -> Result<String> {
        // For now, generate HTML and indicate PDF conversion needed
        let template = self.get_html_template()?;
        let html_content = self.render_html_content(report, &template)?;
        
        let filename = format!("poam_report_{}.pdf", report.report_id);
        let filepath = format!("{}/{}", self.config.output_directory, filename);
        
        // In a real implementation, this would use a PDF generation library
        // For now, we'll save as HTML with PDF extension as placeholder
        std::fs::write(&filepath, html_content)
            .map_err(|e| Error::document_parsing(format!("Failed to write PDF report: {}", e)))?;
        
        warn!("PDF export not fully implemented - saved as HTML");
        Ok(filepath)
    }

    /// Export report to JSON format
    fn export_to_json(&mut self, report: &PoamValidationReport) -> Result<String> {
        let json_content = serde_json::to_string_pretty(report)
            .map_err(|e| Error::validation(format!("Failed to serialize report to JSON: {}", e)))?;
        
        let filename = format!("poam_report_{}.json", report.report_id);
        let filepath = format!("{}/{}", self.config.output_directory, filename);
        
        std::fs::write(&filepath, json_content)
            .map_err(|e| Error::document_parsing(format!("Failed to write JSON report: {}", e)))?;
        
        Ok(filepath)
    }

    /// Export report to CSV format
    fn export_to_csv(&mut self, report: &PoamValidationReport) -> Result<String> {
        let csv_content = self.generate_csv_content(report)?;
        
        let filename = format!("poam_report_{}.csv", report.report_id);
        let filepath = format!("{}/{}", self.config.output_directory, filename);
        
        std::fs::write(&filepath, csv_content)
            .map_err(|e| Error::document_parsing(format!("Failed to write CSV report: {}", e)))?;
        
        Ok(filepath)
    }

    /// Export report to Markdown format
    fn export_to_markdown(&mut self, report: &PoamValidationReport) -> Result<String> {
        let markdown_content = self.generate_markdown_content(report)?;
        
        let filename = format!("poam_report_{}.md", report.report_id);
        let filepath = format!("{}/{}", self.config.output_directory, filename);
        
        std::fs::write(&filepath, markdown_content)
            .map_err(|e| Error::document_parsing(format!("Failed to write Markdown report: {}", e)))?;
        
        Ok(filepath)
    }

    /// Export report to Excel format
    fn export_to_excel(&mut self, report: &PoamValidationReport) -> Result<String> {
        // For now, generate CSV content as Excel placeholder
        let csv_content = self.generate_csv_content(report)?;
        
        let filename = format!("poam_report_{}.xlsx", report.report_id);
        let filepath = format!("{}/{}", self.config.output_directory, filename);
        
        // In a real implementation, this would use an Excel generation library
        std::fs::write(&filepath, csv_content)
            .map_err(|e| Error::document_parsing(format!("Failed to write Excel report: {}", e)))?;
        
        warn!("Excel export not fully implemented - saved as CSV");
        Ok(filepath)
    }

    /// Get HTML template for report rendering
    fn get_html_template(&mut self) -> Result<String> {
        if let Some(template) = self.template_cache.get(&PoamReportFormat::Html) {
            return Ok(template.clone());
        }

        let template = self.load_or_create_html_template()?;
        self.template_cache.insert(PoamReportFormat::Html, template.clone());
        Ok(template)
    }

    /// Load or create HTML template
    fn load_or_create_html_template(&self) -> Result<String> {
        // Try to load custom template first
        if let Some(templates_dir) = &self.config.templates_directory {
            let template_path = format!("{}/poam_report.html", templates_dir);
            if std::path::Path::new(&template_path).exists() {
                return std::fs::read_to_string(&template_path)
                    .map_err(|e| Error::document_parsing(format!("Failed to read HTML template: {}", e)));
            }
        }

        // Use default template
        Ok(self.get_default_html_template())
    }

    /// Get default HTML template
    fn get_default_html_template(&self) -> String {
        r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>POA&M Validation Report</title>
    <style>
        body { font-family: Arial, sans-serif; margin: 20px; }
        .header { background-color: #f0f0f0; padding: 20px; border-radius: 5px; }
        .section { margin: 20px 0; }
        .metric { display: inline-block; margin: 10px; padding: 10px; background-color: #e9e9e9; border-radius: 3px; }
        .error { color: #d32f2f; }
        .warning { color: #f57c00; }
        .success { color: #388e3c; }
        table { border-collapse: collapse; width: 100%; }
        th, td { border: 1px solid #ddd; padding: 8px; text-align: left; }
        th { background-color: #f2f2f2; }
    </style>
</head>
<body>
    {{CONTENT}}
</body>
</html>"#.to_string()
    }

    /// Render HTML content using template
    fn render_html_content(&self, report: &PoamValidationReport, template: &str) -> Result<String> {
        let content = self.generate_html_body(report)?;
        Ok(template.replace("{{CONTENT}}", &content))
    }

    /// Generate HTML body content
    fn generate_html_body(&self, report: &PoamValidationReport) -> Result<String> {
        let mut html = String::new();

        // Header section
        html.push_str(&format!(
            r#"<div class="header">
                <h1>POA&M Validation Report</h1>
                <p><strong>Report ID:</strong> {}</p>
                <p><strong>Generated:</strong> {}</p>
                <p><strong>Document:</strong> {}</p>
            </div>"#,
            report.report_id,
            report.generated_at.format("%Y-%m-%d %H:%M:%S UTC"),
            report.document_info.document_name
        ));

        // Processing summary
        html.push_str(&format!(
            r#"<div class="section">
                <h2>Processing Summary</h2>
                <div class="metric">Total Items: {}</div>
                <div class="metric success">Successful: {}</div>
                <div class="metric error">Errors: {}</div>
                <div class="metric warning">Warnings: {}</div>
                <div class="metric">Quality Score: {:.1}%</div>
                <div class="metric">Compliance Score: {:.1}%</div>
            </div>"#,
            report.processing_summary.total_items_processed,
            report.processing_summary.successful_items,
            report.processing_summary.items_with_errors,
            report.processing_summary.items_with_warnings,
            report.processing_summary.quality_score * 100.0,
            report.processing_summary.compliance_score * 100.0
        ));

        // Recommendations
        if !report.recommendations.is_empty() {
            html.push_str(r#"<div class="section"><h2>Recommendations</h2><ul>"#);
            for rec in &report.recommendations {
                html.push_str(&format!(
                    r#"<li><strong>{}:</strong> {} (Priority: {:?})</li>"#,
                    rec.title, rec.description, rec.priority
                ));
            }
            html.push_str("</ul></div>");
        }

        Ok(html)
    }

    /// Generate CSV content
    fn generate_csv_content(&self, report: &PoamValidationReport) -> Result<String> {
        let mut csv = String::new();
        
        // Header
        csv.push_str("Metric,Value\n");
        
        // Basic metrics
        csv.push_str(&format!("Report ID,{}\n", report.report_id));
        csv.push_str(&format!("Generated At,{}\n", report.generated_at));
        csv.push_str(&format!("Total Items,{}\n", report.processing_summary.total_items_processed));
        csv.push_str(&format!("Successful Items,{}\n", report.processing_summary.successful_items));
        csv.push_str(&format!("Items with Errors,{}\n", report.processing_summary.items_with_errors));
        csv.push_str(&format!("Items with Warnings,{}\n", report.processing_summary.items_with_warnings));
        csv.push_str(&format!("Quality Score,{:.3}\n", report.processing_summary.quality_score));
        csv.push_str(&format!("Compliance Score,{:.3}\n", report.processing_summary.compliance_score));
        
        Ok(csv)
    }

    /// Generate Markdown content
    fn generate_markdown_content(&self, report: &PoamValidationReport) -> Result<String> {
        let mut md = String::new();
        
        // Title and metadata
        md.push_str(&format!("# POA&M Validation Report\n\n"));
        md.push_str(&format!("**Report ID:** {}\n", report.report_id));
        md.push_str(&format!("**Generated:** {}\n", report.generated_at));
        md.push_str(&format!("**Document:** {}\n\n", report.document_info.document_name));
        
        // Processing summary
        md.push_str("## Processing Summary\n\n");
        md.push_str(&format!("- **Total Items:** {}\n", report.processing_summary.total_items_processed));
        md.push_str(&format!("- **Successful Items:** {}\n", report.processing_summary.successful_items));
        md.push_str(&format!("- **Items with Errors:** {}\n", report.processing_summary.items_with_errors));
        md.push_str(&format!("- **Items with Warnings:** {}\n", report.processing_summary.items_with_warnings));
        md.push_str(&format!("- **Quality Score:** {:.1}%\n", report.processing_summary.quality_score * 100.0));
        md.push_str(&format!("- **Compliance Score:** {:.1}%\n\n", report.processing_summary.compliance_score * 100.0));
        
        // Recommendations
        if !report.recommendations.is_empty() {
            md.push_str("## Recommendations\n\n");
            for rec in &report.recommendations {
                md.push_str(&format!("### {} (Priority: {:?})\n\n", rec.title, rec.priority));
                md.push_str(&format!("{}\n\n", rec.description));
                if !rec.actions.is_empty() {
                    md.push_str("**Actions:**\n");
                    for action in &rec.actions {
                        md.push_str(&format!("- {}\n", action));
                    }
                    md.push_str("\n");
                }
            }
        }
        
        Ok(md)
    }

    /// Ensure output directory exists
    pub fn ensure_output_directory(&self) -> Result<()> {
        std::fs::create_dir_all(&self.config.output_directory)
            .map_err(|e| Error::document_parsing(format!("Failed to create output directory: {}", e)))?;
        Ok(())
    }
}
