// Modified: 2025-09-20

//! # Compliance Dashboard
//!
//! Real-time tracking of control implementation status across NIST 800-53, NIST 800-171, and other frameworks.

pub mod dashboard;
pub mod metrics;
pub mod widgets;
pub mod realtime;
pub mod api;

pub use dashboard::{DashboardService, DashboardOverview, Control, Framework, ImplementationStatus, Priority};
pub use metrics::{MetricsService, ComplianceMetrics, StatusBreakdown, KPI, TrendDirection};
pub use widgets::{WidgetService, Widget, WidgetType, WidgetData};
pub use realtime::{RealtimeService, RealtimeEvent, WebSocketMessage, SystemStatus};
pub use api::{create_router, start_server};

use fedramp_core::Result;

/// Main dashboard application
#[derive(Debug, Clone)]
pub struct ComplianceDashboard {
    /// Dashboard service
    pub dashboard: DashboardService,
    /// Metrics service
    pub metrics: MetricsService,
    /// Widget service
    pub widgets: WidgetService,
    /// Real-time service
    pub realtime: RealtimeService,
}

impl ComplianceDashboard {
    /// Create a new compliance dashboard
    pub fn new() -> Self {
        Self {
            dashboard: DashboardService::new(),
            metrics: MetricsService::new(),
            widgets: WidgetService::new(),
            realtime: RealtimeService::new(),
        }
    }

    /// Create dashboard with sample data
    pub fn with_sample_data() -> Self {
        let mut dashboard = Self {
            dashboard: DashboardService::with_sample_data(),
            metrics: MetricsService::new(),
            widgets: WidgetService::new(),
            realtime: RealtimeService::new(),
        };

        // Initialize widgets with default configuration
        if let Err(e) = dashboard.widgets.create_default_widgets() {
            eprintln!("Failed to create default widgets: {}", e);
        }

        dashboard
    }

    /// Initialize the dashboard
    pub async fn initialize(&mut self) -> Result<()> {
        // Create default widgets
        self.widgets.create_default_widgets()?;

        // Generate sample real-time events for demonstration
        self.realtime.generate_sample_events().await?;

        Ok(())
    }

    /// Get comprehensive dashboard data
    pub async fn get_dashboard_data(&mut self) -> Result<DashboardData> {
        let overview = self.dashboard.get_overview()?;
        let widgets = self.widgets.get_widgets();
        let connection_stats = self.realtime.get_connection_stats().await;

        Ok(DashboardData {
            overview,
            widgets,
            connection_stats,
        })
    }

    /// Update dashboard with new control data
    pub async fn update_control_status(&mut self, control_id: &str, status: ImplementationStatus) -> Result<()> {
        // Update dashboard service
        self.dashboard.update_control_status(control_id, status.clone())?;

        // Broadcast real-time event
        self.realtime.broadcast_event(RealtimeEvent::ControlStatusUpdated {
            control_id: control_id.to_string(),
            old_status: "unknown".to_string(), // In real implementation, we'd track the old status
            new_status: format!("{:?}", status).to_lowercase(),
            updated_by: Some("system".to_string()),
            timestamp: chrono::Utc::now(),
        }).await?;

        Ok(())
    }
}

/// Complete dashboard data structure
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct DashboardData {
    pub overview: DashboardOverview,
    pub widgets: Vec<Widget>,
    pub connection_stats: realtime::ConnectionStats,
}

impl Default for ComplianceDashboard {
    fn default() -> Self {
        Self::new()
    }
}
