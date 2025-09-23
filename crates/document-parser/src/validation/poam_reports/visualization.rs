// Modified: 2025-09-23

//! POA&M validation report visualization engine
//!
//! This module provides data visualization capabilities for POA&M validation reports,
//! including charts, graphs, and interactive dashboard elements.

use super::types::*;
use fedramp_core::{Result, Error};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Visualization engine for POA&M reports
#[derive(Debug)]
pub struct PoamVisualizationEngine {
    /// Visualization configuration
    config: VisualizationConfig,
    /// Chart templates cache
    chart_templates: HashMap<ChartType, ChartTemplate>,
}

/// Visualization configuration
#[derive(Debug, Clone)]
pub struct VisualizationConfig {
    /// Enable interactive charts
    pub enable_interactive: bool,
    /// Chart color scheme
    pub color_scheme: ColorScheme,
    /// Chart dimensions
    pub default_width: u32,
    pub default_height: u32,
    /// Include data tables with charts
    pub include_data_tables: bool,
}

/// Color scheme for visualizations
#[derive(Debug, Clone)]
pub enum ColorScheme {
    Default,
    HighContrast,
    Colorblind,
    Monochrome,
}

/// Chart type enumeration
#[derive(Debug, Clone, Hash, PartialEq, Eq, Serialize, Deserialize)]
pub enum ChartType {
    BarChart,
    PieChart,
    LineChart,
    ScatterPlot,
    Histogram,
    Heatmap,
    Dashboard,
}

/// Chart template for rendering
#[derive(Debug, Clone)]
pub struct ChartTemplate {
    /// Template content (HTML/SVG/JSON)
    pub template: String,
    /// Chart type
    pub chart_type: ChartType,
    /// Required data fields
    pub required_fields: Vec<String>,
}

/// Visualization data structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VisualizationData {
    /// Chart type
    pub chart_type: ChartType,
    /// Chart title
    pub title: String,
    /// Chart data points
    pub data: Vec<DataPoint>,
    /// Chart configuration
    pub config: ChartConfig,
}

/// Individual data point for charts
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataPoint {
    /// X-axis value
    pub x: String,
    /// Y-axis value
    pub y: f64,
    /// Optional category/series
    pub category: Option<String>,
    /// Optional additional metadata
    pub metadata: HashMap<String, String>,
}

/// Chart-specific configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChartConfig {
    /// Chart width
    pub width: u32,
    /// Chart height
    pub height: u32,
    /// X-axis label
    pub x_label: String,
    /// Y-axis label
    pub y_label: String,
    /// Show legend
    pub show_legend: bool,
    /// Color palette
    pub colors: Vec<String>,
}

impl Default for VisualizationConfig {
    fn default() -> Self {
        Self {
            enable_interactive: true,
            color_scheme: ColorScheme::Default,
            default_width: 800,
            default_height: 600,
            include_data_tables: true,
        }
    }
}

impl PoamVisualizationEngine {
    /// Create a new visualization engine
    pub fn new(config: VisualizationConfig) -> Self {
        let mut engine = Self {
            config,
            chart_templates: HashMap::new(),
        };
        engine.initialize_templates();
        engine
    }

    /// Generate visualizations for a POA&M report
    pub fn generate_visualizations(&self, report: &PoamValidationReport) -> Result<Vec<VisualizationData>> {
        let mut visualizations = Vec::new();

        // Processing summary chart
        visualizations.push(self.create_processing_summary_chart(report)?);

        // Quality metrics chart
        visualizations.push(self.create_quality_metrics_chart(report)?);

        // Compliance status chart
        visualizations.push(self.create_compliance_chart(report)?);

        // Validation results breakdown
        visualizations.push(self.create_validation_breakdown_chart(report)?);

        // Recommendations priority chart
        if !report.recommendations.is_empty() {
            visualizations.push(self.create_recommendations_chart(report)?);
        }

        Ok(visualizations)
    }

    /// Create processing summary visualization
    fn create_processing_summary_chart(&self, report: &PoamValidationReport) -> Result<VisualizationData> {
        let summary = &report.processing_summary;
        
        let data = vec![
            DataPoint {
                x: "Successful".to_string(),
                y: summary.successful_items as f64,
                category: Some("success".to_string()),
                metadata: HashMap::new(),
            },
            DataPoint {
                x: "Errors".to_string(),
                y: summary.items_with_errors as f64,
                category: Some("error".to_string()),
                metadata: HashMap::new(),
            },
            DataPoint {
                x: "Warnings".to_string(),
                y: summary.items_with_warnings as f64,
                category: Some("warning".to_string()),
                metadata: HashMap::new(),
            },
        ];

        Ok(VisualizationData {
            chart_type: ChartType::BarChart,
            title: "Processing Summary".to_string(),
            data,
            config: ChartConfig {
                width: self.config.default_width,
                height: self.config.default_height,
                x_label: "Status".to_string(),
                y_label: "Number of Items".to_string(),
                show_legend: true,
                colors: self.get_color_palette(),
            },
        })
    }

    /// Create quality metrics visualization
    fn create_quality_metrics_chart(&self, report: &PoamValidationReport) -> Result<VisualizationData> {
        let data = vec![
            DataPoint {
                x: "Quality Score".to_string(),
                y: report.processing_summary.quality_score * 100.0,
                category: Some("quality".to_string()),
                metadata: HashMap::new(),
            },
            DataPoint {
                x: "Compliance Score".to_string(),
                y: report.processing_summary.compliance_score * 100.0,
                category: Some("compliance".to_string()),
                metadata: HashMap::new(),
            },
            DataPoint {
                x: "Completeness".to_string(),
                y: report.quality_assessment.completeness_score * 100.0,
                category: Some("completeness".to_string()),
                metadata: HashMap::new(),
            },
            DataPoint {
                x: "Accuracy".to_string(),
                y: report.quality_assessment.accuracy_score * 100.0,
                category: Some("accuracy".to_string()),
                metadata: HashMap::new(),
            },
        ];

        Ok(VisualizationData {
            chart_type: ChartType::BarChart,
            title: "Quality Metrics (%)".to_string(),
            data,
            config: ChartConfig {
                width: self.config.default_width,
                height: self.config.default_height,
                x_label: "Metric".to_string(),
                y_label: "Score (%)".to_string(),
                show_legend: false,
                colors: self.get_color_palette(),
            },
        })
    }

    /// Create compliance status visualization
    fn create_compliance_chart(&self, report: &PoamValidationReport) -> Result<VisualizationData> {
        let compliance = &report.compliance_status;
        
        let data = vec![
            DataPoint {
                x: "FedRAMP".to_string(),
                y: compliance.fedramp_compliance.score * 100.0,
                category: Some("fedramp".to_string()),
                metadata: HashMap::new(),
            },
            DataPoint {
                x: "OSCAL".to_string(),
                y: compliance.oscal_compliance.score * 100.0,
                category: Some("oscal".to_string()),
                metadata: HashMap::new(),
            },
            DataPoint {
                x: "Overall".to_string(),
                y: compliance.overall_score * 100.0,
                category: Some("overall".to_string()),
                metadata: HashMap::new(),
            },
        ];

        Ok(VisualizationData {
            chart_type: ChartType::BarChart,
            title: "Compliance Scores (%)".to_string(),
            data,
            config: ChartConfig {
                width: self.config.default_width,
                height: self.config.default_height,
                x_label: "Compliance Area".to_string(),
                y_label: "Score (%)".to_string(),
                show_legend: false,
                colors: self.get_color_palette(),
            },
        })
    }

    /// Create validation breakdown visualization
    fn create_validation_breakdown_chart(&self, report: &PoamValidationReport) -> Result<VisualizationData> {
        let validation = &report.validation_results;
        
        let data = vec![
            DataPoint {
                x: "Schema".to_string(),
                y: validation.schema_validation.compliance_score * 100.0,
                category: Some("schema".to_string()),
                metadata: HashMap::new(),
            },
            DataPoint {
                x: "Business Rules".to_string(),
                y: validation.business_rule_validation.compliance_score * 100.0,
                category: Some("business".to_string()),
                metadata: HashMap::new(),
            },
            DataPoint {
                x: "Data Quality".to_string(),
                y: validation.data_quality_validation.quality_score * 100.0,
                category: Some("quality".to_string()),
                metadata: HashMap::new(),
            },
            DataPoint {
                x: "Cross-Field".to_string(),
                y: validation.cross_field_validation.consistency_score * 100.0,
                category: Some("crossfield".to_string()),
                metadata: HashMap::new(),
            },
        ];

        Ok(VisualizationData {
            chart_type: ChartType::BarChart,
            title: "Validation Results Breakdown (%)".to_string(),
            data,
            config: ChartConfig {
                width: self.config.default_width,
                height: self.config.default_height,
                x_label: "Validation Type".to_string(),
                y_label: "Success Rate (%)".to_string(),
                show_legend: false,
                colors: self.get_color_palette(),
            },
        })
    }

    /// Create recommendations priority visualization
    fn create_recommendations_chart(&self, report: &PoamValidationReport) -> Result<VisualizationData> {
        let mut priority_counts = HashMap::new();
        
        for rec in &report.recommendations {
            let count = priority_counts.entry(rec.priority.clone()).or_insert(0);
            *count += 1;
        }

        let data = priority_counts.into_iter()
            .map(|(priority, count)| DataPoint {
                x: format!("{:?}", priority),
                y: count as f64,
                category: Some(format!("{:?}", priority).to_lowercase()),
                metadata: HashMap::new(),
            })
            .collect();

        Ok(VisualizationData {
            chart_type: ChartType::PieChart,
            title: "Recommendations by Priority".to_string(),
            data,
            config: ChartConfig {
                width: self.config.default_width,
                height: self.config.default_height,
                x_label: "Priority".to_string(),
                y_label: "Count".to_string(),
                show_legend: true,
                colors: self.get_color_palette(),
            },
        })
    }

    /// Get color palette based on configuration
    fn get_color_palette(&self) -> Vec<String> {
        match self.config.color_scheme {
            ColorScheme::Default => vec![
                "#3498db".to_string(), // Blue
                "#e74c3c".to_string(), // Red
                "#f39c12".to_string(), // Orange
                "#2ecc71".to_string(), // Green
                "#9b59b6".to_string(), // Purple
                "#1abc9c".to_string(), // Teal
            ],
            ColorScheme::HighContrast => vec![
                "#000000".to_string(), // Black
                "#ffffff".to_string(), // White
                "#ff0000".to_string(), // Red
                "#00ff00".to_string(), // Green
                "#0000ff".to_string(), // Blue
                "#ffff00".to_string(), // Yellow
            ],
            ColorScheme::Colorblind => vec![
                "#1f77b4".to_string(), // Blue
                "#ff7f0e".to_string(), // Orange
                "#2ca02c".to_string(), // Green
                "#d62728".to_string(), // Red
                "#9467bd".to_string(), // Purple
                "#8c564b".to_string(), // Brown
            ],
            ColorScheme::Monochrome => vec![
                "#2c3e50".to_string(), // Dark gray
                "#34495e".to_string(), // Gray
                "#7f8c8d".to_string(), // Light gray
                "#95a5a6".to_string(), // Lighter gray
                "#bdc3c7".to_string(), // Very light gray
                "#ecf0f1".to_string(), // Almost white
            ],
        }
    }

    /// Initialize chart templates
    fn initialize_templates(&mut self) {
        // Initialize basic chart templates
        // In a real implementation, these would be more sophisticated
        
        self.chart_templates.insert(
            ChartType::BarChart,
            ChartTemplate {
                template: "bar_chart_template".to_string(),
                chart_type: ChartType::BarChart,
                required_fields: vec!["x".to_string(), "y".to_string()],
            }
        );

        self.chart_templates.insert(
            ChartType::PieChart,
            ChartTemplate {
                template: "pie_chart_template".to_string(),
                chart_type: ChartType::PieChart,
                required_fields: vec!["x".to_string(), "y".to_string()],
            }
        );
    }

    /// Render visualization to HTML/SVG
    pub fn render_visualization(&self, viz: &VisualizationData) -> Result<String> {
        // This would integrate with a charting library like D3.js, Chart.js, or Plotly
        // For now, return a simple HTML representation
        
        let mut html = String::new();
        html.push_str(&format!("<div class='chart-container'>\n"));
        html.push_str(&format!("  <h3>{}</h3>\n", viz.title));
        html.push_str(&format!("  <div class='chart' data-type='{:?}'>\n", viz.chart_type));
        
        // Add data as JSON for JavaScript processing
        let data_json = serde_json::to_string(&viz.data)
            .map_err(|e| Error::validation(format!("Failed to serialize chart data: {}", e)))?;
        html.push_str(&format!("    <script type='application/json' class='chart-data'>{}</script>\n", data_json));
        
        html.push_str("  </div>\n");
        html.push_str("</div>\n");
        
        Ok(html)
    }
}
