//! Dashboard widgets and components
//!
//! This module provides widget definitions and data structures for
//! dashboard components like charts, status indicators, and metrics displays.

use fedramp_core::{Result, Error};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use chrono::{DateTime, Utc};
use crate::metrics::{ComplianceMetrics, KPI, TrendDirection};

/// Widget configuration and data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Widget {
    pub id: String,
    pub title: String,
    pub widget_type: WidgetType,
    pub size: WidgetSize,
    pub position: WidgetPosition,
    pub data: WidgetData,
    pub config: WidgetConfig,
    pub last_updated: DateTime<Utc>,
}

/// Widget type enumeration
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum WidgetType {
    StatusOverview,
    ProgressChart,
    MetricCard,
    TrendChart,
    ControlList,
    ComplianceGauge,
    AlertList,
    FrameworkComparison,
}

/// Widget size configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WidgetSize {
    pub width: u32,  // Grid units
    pub height: u32, // Grid units
    pub min_width: Option<u32>,
    pub min_height: Option<u32>,
}

/// Widget position on dashboard
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WidgetPosition {
    pub x: u32,
    pub y: u32,
    pub z_index: Option<u32>,
}

/// Widget data container
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum WidgetData {
    StatusOverview {
        total_controls: u32,
        implemented: u32,
        in_progress: u32,
        not_implemented: u32,
        percentage: f64,
    },
    ProgressChart {
        data_points: Vec<ChartDataPoint>,
        target_line: Option<f64>,
    },
    MetricCard {
        value: f64,
        unit: String,
        trend: TrendDirection,
        change_percentage: f64,
        target: Option<f64>,
    },
    TrendChart {
        series: Vec<ChartSeries>,
        time_range: TimeRange,
    },
    ControlList {
        controls: Vec<ControlSummary>,
        filter: ControlFilter,
    },
    ComplianceGauge {
        current_value: f64,
        target_value: f64,
        thresholds: Vec<GaugeThreshold>,
    },
    AlertList {
        alerts: Vec<Alert>,
        severity_filter: Option<AlertSeverity>,
    },
    FrameworkComparison {
        frameworks: Vec<FrameworkSummary>,
        comparison_metric: String,
    },
}

/// Chart data point
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChartDataPoint {
    pub timestamp: DateTime<Utc>,
    pub value: f64,
    pub label: Option<String>,
}

/// Chart series for multi-line charts
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChartSeries {
    pub name: String,
    pub data: Vec<ChartDataPoint>,
    pub color: Option<String>,
}

/// Time range for charts
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimeRange {
    pub start: DateTime<Utc>,
    pub end: DateTime<Utc>,
    pub granularity: TimeGranularity,
}

/// Time granularity for data aggregation
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum TimeGranularity {
    Hour,
    Day,
    Week,
    Month,
    Quarter,
    Year,
}

/// Control summary for list widgets
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ControlSummary {
    pub id: String,
    pub identifier: String,
    pub title: String,
    pub status: String,
    pub priority: String,
    pub due_date: Option<DateTime<Utc>>,
    pub assigned_to: Option<String>,
}

/// Control filter options
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ControlFilter {
    pub status: Option<String>,
    pub priority: Option<String>,
    pub framework: Option<String>,
    pub assigned_to: Option<String>,
    pub overdue_only: bool,
}

/// Gauge threshold configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GaugeThreshold {
    pub value: f64,
    pub color: String,
    pub label: String,
}

/// Alert information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Alert {
    pub id: String,
    pub title: String,
    pub message: String,
    pub severity: AlertSeverity,
    pub timestamp: DateTime<Utc>,
    pub acknowledged: bool,
    pub related_control: Option<String>,
}

/// Alert severity levels
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum AlertSeverity {
    Info,
    Warning,
    Error,
    Critical,
}

/// Framework summary for comparison
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FrameworkSummary {
    pub id: String,
    pub name: String,
    pub implementation_percentage: f64,
    pub control_count: u32,
    pub overdue_count: u32,
}

/// Widget configuration options
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WidgetConfig {
    pub refresh_interval: Option<u32>, // seconds
    pub auto_refresh: bool,
    pub show_legend: bool,
    pub show_grid: bool,
    pub color_scheme: Option<String>,
    pub custom_options: HashMap<String, serde_json::Value>,
}

/// Widget service for managing dashboard widgets
#[derive(Debug, Clone)]
pub struct WidgetService {
    widgets: HashMap<String, Widget>,
}

impl WidgetService {
    /// Create a new widget service
    pub fn new() -> Self {
        Self {
            widgets: HashMap::new(),
        }
    }

    /// Create default dashboard widgets
    pub fn create_default_widgets(&mut self) -> Result<()> {
        // Status Overview Widget
        let status_widget = Widget {
            id: "status-overview".to_string(),
            title: "Implementation Status".to_string(),
            widget_type: WidgetType::StatusOverview,
            size: WidgetSize {
                width: 4,
                height: 2,
                min_width: Some(3),
                min_height: Some(2),
            },
            position: WidgetPosition { x: 0, y: 0, z_index: None },
            data: WidgetData::StatusOverview {
                total_controls: 100,
                implemented: 65,
                in_progress: 20,
                not_implemented: 15,
                percentage: 65.0,
            },
            config: WidgetConfig {
                refresh_interval: Some(300),
                auto_refresh: true,
                show_legend: true,
                show_grid: false,
                color_scheme: Some("default".to_string()),
                custom_options: HashMap::new(),
            },
            last_updated: Utc::now(),
        };

        // Progress Chart Widget
        let progress_widget = Widget {
            id: "progress-chart".to_string(),
            title: "Implementation Progress".to_string(),
            widget_type: WidgetType::ProgressChart,
            size: WidgetSize {
                width: 6,
                height: 3,
                min_width: Some(4),
                min_height: Some(2),
            },
            position: WidgetPosition { x: 4, y: 0, z_index: None },
            data: WidgetData::ProgressChart {
                data_points: vec![
                    ChartDataPoint {
                        timestamp: Utc::now() - chrono::Duration::days(30),
                        value: 45.0,
                        label: Some("30 days ago".to_string()),
                    },
                    ChartDataPoint {
                        timestamp: Utc::now() - chrono::Duration::days(15),
                        value: 55.0,
                        label: Some("15 days ago".to_string()),
                    },
                    ChartDataPoint {
                        timestamp: Utc::now(),
                        value: 65.0,
                        label: Some("Today".to_string()),
                    },
                ],
                target_line: Some(80.0),
            },
            config: WidgetConfig {
                refresh_interval: Some(600),
                auto_refresh: true,
                show_legend: true,
                show_grid: true,
                color_scheme: Some("blue".to_string()),
                custom_options: HashMap::new(),
            },
            last_updated: Utc::now(),
        };

        // Compliance Gauge Widget
        let gauge_widget = Widget {
            id: "compliance-gauge".to_string(),
            title: "Compliance Score".to_string(),
            widget_type: WidgetType::ComplianceGauge,
            size: WidgetSize {
                width: 3,
                height: 3,
                min_width: Some(2),
                min_height: Some(2),
            },
            position: WidgetPosition { x: 0, y: 2, z_index: None },
            data: WidgetData::ComplianceGauge {
                current_value: 75.5,
                target_value: 85.0,
                thresholds: vec![
                    GaugeThreshold {
                        value: 50.0,
                        color: "#ef4444".to_string(),
                        label: "Poor".to_string(),
                    },
                    GaugeThreshold {
                        value: 70.0,
                        color: "#f59e0b".to_string(),
                        label: "Fair".to_string(),
                    },
                    GaugeThreshold {
                        value: 85.0,
                        color: "#10b981".to_string(),
                        label: "Good".to_string(),
                    },
                ],
            },
            config: WidgetConfig {
                refresh_interval: Some(300),
                auto_refresh: true,
                show_legend: false,
                show_grid: false,
                color_scheme: Some("gauge".to_string()),
                custom_options: HashMap::new(),
            },
            last_updated: Utc::now(),
        };

        self.widgets.insert(status_widget.id.clone(), status_widget);
        self.widgets.insert(progress_widget.id.clone(), progress_widget);
        self.widgets.insert(gauge_widget.id.clone(), gauge_widget);

        Ok(())
    }

    /// Get all widgets
    pub fn get_widgets(&self) -> Vec<Widget> {
        self.widgets.values().cloned().collect()
    }

    /// Get widget by ID
    pub fn get_widget(&self, widget_id: &str) -> Option<Widget> {
        self.widgets.get(widget_id).cloned()
    }

    /// Update widget data
    pub fn update_widget_data(&mut self, widget_id: &str, data: WidgetData) -> Result<()> {
        if let Some(widget) = self.widgets.get_mut(widget_id) {
            widget.data = data;
            widget.last_updated = Utc::now();
            Ok(())
        } else {
            Err(Error::not_found(format!("Widget not found: {}", widget_id)))
        }
    }

    /// Update widget from compliance metrics
    pub fn update_from_metrics(&mut self, metrics: &ComplianceMetrics) -> Result<()> {
        // Update status overview widget
        if let Some(widget) = self.widgets.get_mut("status-overview") {
            widget.data = WidgetData::StatusOverview {
                total_controls: metrics.status_breakdown.total as u32,
                implemented: metrics.status_breakdown.implemented as u32,
                in_progress: metrics.status_breakdown.in_progress as u32,
                not_implemented: metrics.status_breakdown.not_implemented as u32,
                percentage: metrics.implementation_percentage,
            };
            widget.last_updated = Utc::now();
        }

        // Update compliance gauge widget
        if let Some(widget) = self.widgets.get_mut("compliance-gauge") {
            widget.data = WidgetData::ComplianceGauge {
                current_value: metrics.implementation_percentage,
                target_value: 85.0,
                thresholds: vec![
                    GaugeThreshold {
                        value: 50.0,
                        color: "#ef4444".to_string(),
                        label: "Poor".to_string(),
                    },
                    GaugeThreshold {
                        value: 70.0,
                        color: "#f59e0b".to_string(),
                        label: "Fair".to_string(),
                    },
                    GaugeThreshold {
                        value: 85.0,
                        color: "#10b981".to_string(),
                        label: "Good".to_string(),
                    },
                ],
            };
            widget.last_updated = Utc::now();
        }

        Ok(())
    }
}

impl Default for WidgetService {
    fn default() -> Self {
        Self::new()
    }
}
