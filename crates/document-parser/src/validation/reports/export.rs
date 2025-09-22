//! Report export functionality
//! Modified: 2025-01-22

use crate::{Error, Result};
use super::types::*;

/// Report export functionality
#[derive(Debug)]
pub struct ReportExporter {
    /// Configuration for export operations
    config: ReportConfig,
}

impl ReportExporter {
    /// Create a new report exporter
    pub fn new(config: ReportConfig) -> Self {
        Self { config }
    }

    /// Export report to specified format
    pub fn export_report(&self, report: &MappingReport, format: ReportFormat) -> Result<String> {
        match format {
            ReportFormat::Json => self.export_to_json(report),
            ReportFormat::Html => self.export_to_html(report),
            ReportFormat::Csv => self.export_to_csv(report),
            ReportFormat::Markdown => self.export_to_markdown(report),
            ReportFormat::Pdf => self.export_to_pdf(report),
        }
    }

    /// Export report to JSON format
    fn export_to_json(&self, report: &MappingReport) -> Result<String> {
        serde_json::to_string_pretty(report)
            .map_err(|e| Error::document_parsing(format!("Failed to serialize report to JSON: {}", e)))
    }

    /// Export report to HTML format
    fn export_to_html(&self, report: &MappingReport) -> Result<String> {
        // For now, always use simple HTML generation
        // Template engine integration can be added later
        Ok(self.generate_simple_html(report))
    }

    /// Export report to CSV format
    fn export_to_csv(&self, report: &MappingReport) -> Result<String> {
        let mut csv_content = String::new();

        // Header
        csv_content.push_str("Field ID,Target Field,Source Column,Confidence Score,Mapping Successful,Required,Issues\n");

        // Data rows
        for result in &report.detailed_results {
            csv_content.push_str(&format!(
                "{},{},{},{:.3},{},{},{}\n",
                result.field_id,
                result.target_field,
                result.source_column.as_deref().unwrap_or(""),
                result.confidence_score,
                result.mapping_successful,
                result.required,
                result.issues.len()
            ));
        }

        Ok(csv_content)
    }

    /// Export report to Markdown format
    fn export_to_markdown(&self, report: &MappingReport) -> Result<String> {
        let mut md_content = String::new();

        md_content.push_str(&format!("# Mapping Validation Report\n\n"));
        md_content.push_str(&format!("**Report ID:** {}\n", report.report_id));
        md_content.push_str(&format!("**Generated:** {}\n", report.generated_at.format("%Y-%m-%d %H:%M:%S UTC")));
        md_content.push_str(&format!("**Document:** {}\n\n", report.document_info.file_name));

        md_content.push_str("## Summary\n\n");
        md_content.push_str(&format!("- **Total Fields:** {}\n", report.mapping_summary.total_fields));
        md_content.push_str(&format!("- **Mapped Fields:** {}\n", report.mapping_summary.mapped_fields));
        md_content.push_str(&format!("- **Success Rate:** {:.1}%\n", report.mapping_summary.success_rate * 100.0));
        md_content.push_str(&format!("- **Average Confidence:** {:.1}%\n", report.mapping_summary.average_confidence * 100.0));
        md_content.push_str(&format!("- **Quality Grade:** {:?}\n\n", report.quality_metrics.quality_grade));

        md_content.push_str("## Recommendations\n\n");
        for (i, rec) in report.recommendations.iter().enumerate() {
            md_content.push_str(&format!("{}. **{}** (Priority: {:?})\n", i + 1, rec.title, rec.priority));
            md_content.push_str(&format!("   - {}\n", rec.description));
            md_content.push_str(&format!("   - Action: {}\n\n", rec.suggested_action));
        }

        Ok(md_content)
    }

    /// Export report to PDF format (placeholder implementation)
    fn export_to_pdf(&self, _report: &MappingReport) -> Result<String> {
        // This would require a PDF generation library like `printpdf` or `wkhtmltopdf`
        // For now, return an error indicating PDF export is not implemented
        Err(Error::document_parsing("PDF export not yet implemented".to_string()))
    }

    /// Generate simple HTML report (fallback when no template engine)
    fn generate_simple_html(&self, report: &MappingReport) -> String {
        format!(
            r#"<!DOCTYPE html>
<html>
<head>
    <title>Mapping Validation Report</title>
    <style>
        body {{ font-family: Arial, sans-serif; margin: 20px; }}
        .header {{ background-color: #f0f0f0; padding: 10px; border-radius: 5px; }}
        .summary {{ margin: 20px 0; }}
        .quality-grade {{ font-size: 24px; font-weight: bold; }}
        .recommendations {{ margin: 20px 0; }}
        .recommendation {{ margin: 10px 0; padding: 10px; border-left: 4px solid #007acc; }}
        table {{ border-collapse: collapse; width: 100%; }}
        th, td {{ border: 1px solid #ddd; padding: 8px; text-align: left; }}
        th {{ background-color: #f2f2f2; }}
        .success {{ color: green; }}
        .warning {{ color: orange; }}
        .error {{ color: red; }}
        .metrics {{ display: grid; grid-template-columns: repeat(auto-fit, minmax(200px, 1fr)); gap: 20px; margin: 20px 0; }}
        .metric-card {{ background: #f9f9f9; padding: 15px; border-radius: 5px; }}
        .metric-value {{ font-size: 24px; font-weight: bold; color: #007acc; }}
    </style>
</head>
<body>
    <div class="header">
        <h1>Mapping Validation Report</h1>
        <p><strong>Report ID:</strong> {}</p>
        <p><strong>Generated:</strong> {}</p>
        <p><strong>Document:</strong> {}</p>
    </div>

    <div class="summary">
        <h2>Summary</h2>
        <div class="metrics">
            <div class="metric-card">
                <div class="metric-value">{}</div>
                <div>Total Fields</div>
            </div>
            <div class="metric-card">
                <div class="metric-value">{}</div>
                <div>Mapped Fields</div>
            </div>
            <div class="metric-card">
                <div class="metric-value">{:.1}%</div>
                <div>Success Rate</div>
            </div>
            <div class="metric-card">
                <div class="metric-value">{:.1}%</div>
                <div>Average Confidence</div>
            </div>
            <div class="metric-card">
                <div class="metric-value quality-grade">{:?}</div>
                <div>Quality Grade</div>
            </div>
        </div>
    </div>

    <div class="recommendations">
        <h2>Recommendations</h2>
        {}
    </div>

    <div class="detailed-results">
        <h2>Detailed Results</h2>
        <table>
            <thead>
                <tr>
                    <th>Field ID</th>
                    <th>Target Field</th>
                    <th>Source Column</th>
                    <th>Confidence</th>
                    <th>Status</th>
                    <th>Required</th>
                    <th>Issues</th>
                </tr>
            </thead>
            <tbody>
                {}
            </tbody>
        </table>
    </div>

    <div class="performance-metrics">
        <h2>Performance Metrics</h2>
        <div class="metrics">
            <div class="metric-card">
                <div class="metric-value">{:.2}s</div>
                <div>Total Processing Time</div>
            </div>
            <div class="metric-card">
                <div class="metric-value">{:.1} MB</div>
                <div>Peak Memory Usage</div>
            </div>
            <div class="metric-card">
                <div class="metric-value">{:.1}</div>
                <div>Rows/Second</div>
            </div>
        </div>
    </div>
</body>
</html>"#,
            report.report_id,
            report.generated_at.format("%Y-%m-%d %H:%M:%S UTC"),
            report.document_info.file_name,
            report.mapping_summary.total_fields,
            report.mapping_summary.mapped_fields,
            report.mapping_summary.success_rate * 100.0,
            report.mapping_summary.average_confidence * 100.0,
            report.quality_metrics.quality_grade,
            report.recommendations.iter()
                .map(|r| format!(r#"<div class="recommendation"><strong>{}</strong> (Priority: {:?})<br>{}<br><em>Action: {}</em></div>"#, 
                    r.title, r.priority, r.description, r.suggested_action))
                .collect::<Vec<_>>()
                .join(""),
            report.detailed_results.iter()
                .map(|r| format!(
                    r#"<tr>
                        <td>{}</td>
                        <td>{}</td>
                        <td>{}</td>
                        <td>{:.3}</td>
                        <td class="{}">{}</td>
                        <td>{}</td>
                        <td>{}</td>
                    </tr>"#,
                    r.field_id,
                    r.target_field,
                    r.source_column.as_deref().unwrap_or(""),
                    r.confidence_score,
                    if r.mapping_successful { "success" } else { "error" },
                    if r.mapping_successful { "✓" } else { "✗" },
                    if r.required { "Yes" } else { "No" },
                    r.issues.len()
                ))
                .collect::<Vec<_>>()
                .join(""),
            report.processing_metrics.total_processing_time.as_secs_f64(),
            report.processing_metrics.memory_metrics.peak_memory_mb(),
            report.processing_metrics.throughput_metrics.rows_per_second
        )
    }

    /// Export report with custom template
    pub fn export_with_template(&self, report: &MappingReport, template_path: &str) -> Result<String> {
        // This would integrate with a template engine like Handlebars or Tera
        // For now, fall back to simple HTML generation
        if template_path.ends_with(".html") {
            self.export_to_html(report)
        } else if template_path.ends_with(".md") {
            self.export_to_markdown(report)
        } else {
            Err(Error::document_parsing(format!("Unsupported template format: {}", template_path)))
        }
    }

    /// Get supported export formats
    pub fn supported_formats() -> Vec<ReportFormat> {
        vec![
            ReportFormat::Json,
            ReportFormat::Html,
            ReportFormat::Csv,
            ReportFormat::Markdown,
            // ReportFormat::Pdf, // Not yet implemented
        ]
    }

    /// Validate export format
    pub fn is_format_supported(format: &ReportFormat) -> bool {
        matches!(format, 
            ReportFormat::Json | 
            ReportFormat::Html | 
            ReportFormat::Csv | 
            ReportFormat::Markdown
        )
    }

    /// Get file extension for format
    pub fn get_file_extension(format: &ReportFormat) -> &'static str {
        match format {
            ReportFormat::Json => "json",
            ReportFormat::Html => "html",
            ReportFormat::Csv => "csv",
            ReportFormat::Markdown => "md",
            ReportFormat::Pdf => "pdf",
        }
    }

    /// Get MIME type for format
    pub fn get_mime_type(format: &ReportFormat) -> &'static str {
        match format {
            ReportFormat::Json => "application/json",
            ReportFormat::Html => "text/html",
            ReportFormat::Csv => "text/csv",
            ReportFormat::Markdown => "text/markdown",
            ReportFormat::Pdf => "application/pdf",
        }
    }
}

impl Default for ReportExporter {
    fn default() -> Self {
        Self::new(ReportConfig::default())
    }
}
