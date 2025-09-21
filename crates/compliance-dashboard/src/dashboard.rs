//! Dashboard core functionality
//!
//! This module provides the main dashboard service for managing compliance data,
//! control status tracking, and real-time updates.

use fedramp_core::{Result, Error};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;
use chrono::{DateTime, Utc};

/// Dashboard service for managing compliance data
#[derive(Debug, Clone)]
pub struct DashboardService {
    /// Framework data cache
    frameworks: HashMap<String, Framework>,
    /// Control data cache
    controls: HashMap<String, Control>,
    /// Metrics cache
    metrics: HashMap<String, Metric>,
    /// Last update timestamp
    last_updated: DateTime<Utc>,
}

/// Framework definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Framework {
    pub id: String,
    pub name: String,
    pub version: String,
    pub description: String,
    pub control_count: usize,
    pub implemented_count: usize,
    pub in_progress_count: usize,
    pub not_implemented_count: usize,
    pub last_updated: DateTime<Utc>,
}

/// Control implementation details
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Control {
    pub id: String,
    pub framework_id: String,
    pub identifier: String,
    pub title: String,
    pub description: String,
    pub implementation_status: ImplementationStatus,
    pub priority: Priority,
    pub category: String,
    pub assigned_to: Option<String>,
    pub due_date: Option<DateTime<Utc>>,
    pub completion_date: Option<DateTime<Utc>>,
    pub evidence_count: usize,
    pub last_updated: DateTime<Utc>,
}

/// Implementation status enumeration
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum ImplementationStatus {
    NotImplemented,
    InProgress,
    Implemented,
    NotApplicable,
}

/// Priority levels
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum Priority {
    Low,
    Medium,
    High,
    Critical,
}

/// Dashboard metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Metric {
    pub id: String,
    pub name: String,
    pub value: f64,
    pub unit: String,
    pub trend: Trend,
    pub last_updated: DateTime<Utc>,
}

/// Trend direction
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Trend {
    Up,
    Down,
    Stable,
}

/// Dashboard overview data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DashboardOverview {
    pub total_controls: usize,
    pub implemented_controls: usize,
    pub in_progress_controls: usize,
    pub not_implemented_controls: usize,
    pub implementation_percentage: f64,
    pub frameworks: Vec<Framework>,
    pub recent_updates: Vec<Control>,
    pub key_metrics: Vec<Metric>,
    pub last_updated: DateTime<Utc>,
}

impl DashboardService {
    /// Create a new dashboard service
    pub fn new() -> Self {
        Self {
            frameworks: HashMap::new(),
            controls: HashMap::new(),
            metrics: HashMap::new(),
            last_updated: Utc::now(),
        }
    }

    /// Initialize with sample data for demonstration
    pub fn with_sample_data() -> Self {
        let mut service = Self::new();
        service.load_sample_data();
        service
    }

    /// Get dashboard overview
    pub fn get_overview(&self) -> Result<DashboardOverview> {
        let total_controls = self.controls.len();
        let implemented_controls = self.controls.values()
            .filter(|c| c.implementation_status == ImplementationStatus::Implemented)
            .count();
        let in_progress_controls = self.controls.values()
            .filter(|c| c.implementation_status == ImplementationStatus::InProgress)
            .count();
        let not_implemented_controls = self.controls.values()
            .filter(|c| c.implementation_status == ImplementationStatus::NotImplemented)
            .count();

        let implementation_percentage = if total_controls > 0 {
            (implemented_controls as f64 / total_controls as f64) * 100.0
        } else {
            0.0
        };

        let frameworks: Vec<Framework> = self.frameworks.values().cloned().collect();
        
        let mut recent_updates: Vec<Control> = self.controls.values().cloned().collect();
        recent_updates.sort_by(|a, b| b.last_updated.cmp(&a.last_updated));
        recent_updates.truncate(10);

        let key_metrics: Vec<Metric> = self.metrics.values().cloned().collect();

        Ok(DashboardOverview {
            total_controls,
            implemented_controls,
            in_progress_controls,
            not_implemented_controls,
            implementation_percentage,
            frameworks,
            recent_updates,
            key_metrics,
            last_updated: self.last_updated,
        })
    }

    /// Get controls by framework
    pub fn get_controls_by_framework(&self, framework_id: &str) -> Result<Vec<Control>> {
        let controls: Vec<Control> = self.controls.values()
            .filter(|c| c.framework_id == framework_id)
            .cloned()
            .collect();
        Ok(controls)
    }

    /// Update control status
    pub fn update_control_status(&mut self, control_id: &str, status: ImplementationStatus) -> Result<()> {
        if let Some(control) = self.controls.get_mut(control_id) {
            control.implementation_status = status;
            control.last_updated = Utc::now();
            self.last_updated = Utc::now();
            Ok(())
        } else {
            Err(Error::not_found(format!("Control not found: {}", control_id)))
        }
    }

    /// Load sample data for demonstration
    fn load_sample_data(&mut self) {
        // Sample frameworks
        let nist_800_53 = Framework {
            id: "nist-800-53".to_string(),
            name: "NIST 800-53".to_string(),
            version: "Rev 5".to_string(),
            description: "Security and Privacy Controls for Federal Information Systems".to_string(),
            control_count: 1000,
            implemented_count: 650,
            in_progress_count: 200,
            not_implemented_count: 150,
            last_updated: Utc::now(),
        };

        let nist_800_171 = Framework {
            id: "nist-800-171".to_string(),
            name: "NIST 800-171".to_string(),
            version: "Rev 2".to_string(),
            description: "Protecting Controlled Unclassified Information".to_string(),
            control_count: 110,
            implemented_count: 85,
            in_progress_count: 15,
            not_implemented_count: 10,
            last_updated: Utc::now(),
        };

        self.frameworks.insert(nist_800_53.id.clone(), nist_800_53);
        self.frameworks.insert(nist_800_171.id.clone(), nist_800_171);

        // Sample controls
        let controls = vec![
            Control {
                id: "ac-1".to_string(),
                framework_id: "nist-800-53".to_string(),
                identifier: "AC-1".to_string(),
                title: "Access Control Policy and Procedures".to_string(),
                description: "Develop, document, and disseminate access control policy and procedures.".to_string(),
                implementation_status: ImplementationStatus::Implemented,
                priority: Priority::High,
                category: "Access Control".to_string(),
                assigned_to: Some("security-team".to_string()),
                due_date: None,
                completion_date: Some(Utc::now()),
                evidence_count: 3,
                last_updated: Utc::now(),
            },
            Control {
                id: "ac-2".to_string(),
                framework_id: "nist-800-53".to_string(),
                identifier: "AC-2".to_string(),
                title: "Account Management".to_string(),
                description: "Manage information system accounts.".to_string(),
                implementation_status: ImplementationStatus::InProgress,
                priority: Priority::High,
                category: "Access Control".to_string(),
                assigned_to: Some("it-team".to_string()),
                due_date: Some(Utc::now() + chrono::Duration::days(30)),
                completion_date: None,
                evidence_count: 1,
                last_updated: Utc::now(),
            },
            Control {
                id: "ac-3".to_string(),
                framework_id: "nist-800-53".to_string(),
                identifier: "AC-3".to_string(),
                title: "Access Enforcement".to_string(),
                description: "Enforce approved authorizations for logical access.".to_string(),
                implementation_status: ImplementationStatus::NotImplemented,
                priority: Priority::Medium,
                category: "Access Control".to_string(),
                assigned_to: None,
                due_date: Some(Utc::now() + chrono::Duration::days(60)),
                completion_date: None,
                evidence_count: 0,
                last_updated: Utc::now(),
            },
        ];

        for control in controls {
            self.controls.insert(control.id.clone(), control);
        }

        // Sample metrics
        let metrics = vec![
            Metric {
                id: "implementation-rate".to_string(),
                name: "Implementation Rate".to_string(),
                value: 75.5,
                unit: "%".to_string(),
                trend: Trend::Up,
                last_updated: Utc::now(),
            },
            Metric {
                id: "overdue-controls".to_string(),
                name: "Overdue Controls".to_string(),
                value: 12.0,
                unit: "count".to_string(),
                trend: Trend::Down,
                last_updated: Utc::now(),
            },
        ];

        for metric in metrics {
            self.metrics.insert(metric.id.clone(), metric);
        }

        self.last_updated = Utc::now();
    }
}

impl Default for DashboardService {
    fn default() -> Self {
        Self::new()
    }
}
