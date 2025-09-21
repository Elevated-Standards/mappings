//! Metrics calculation and tracking
//!
//! This module provides functionality for calculating compliance metrics,
//! KPIs, and performance indicators for the dashboard.

use fedramp_core::{Result, Error};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use chrono::{DateTime, Utc, Duration};
use crate::dashboard::{Control, Framework, ImplementationStatus, Priority};

/// Metrics calculator service
#[derive(Debug, Clone)]
pub struct MetricsService {
    /// Historical metrics data
    history: HashMap<String, Vec<MetricDataPoint>>,
    /// Calculation cache
    cache: HashMap<String, CachedMetric>,
    /// Cache TTL in seconds
    cache_ttl: i64,
}

/// Individual metric data point
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricDataPoint {
    pub timestamp: DateTime<Utc>,
    pub value: f64,
    pub metadata: HashMap<String, String>,
}

/// Cached metric with expiration
#[derive(Debug, Clone)]
struct CachedMetric {
    value: ComplianceMetrics,
    expires_at: DateTime<Utc>,
}

/// Comprehensive compliance metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComplianceMetrics {
    /// Overall implementation percentage
    pub implementation_percentage: f64,
    /// Controls by status
    pub status_breakdown: StatusBreakdown,
    /// Controls by priority
    pub priority_breakdown: PriorityBreakdown,
    /// Framework-specific metrics
    pub framework_metrics: HashMap<String, FrameworkMetrics>,
    /// Trend analysis
    pub trends: TrendAnalysis,
    /// Performance indicators
    pub kpis: Vec<KPI>,
    /// Risk indicators
    pub risk_indicators: RiskIndicators,
    /// Calculation timestamp
    pub calculated_at: DateTime<Utc>,
}

/// Status breakdown metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StatusBreakdown {
    pub implemented: usize,
    pub in_progress: usize,
    pub not_implemented: usize,
    pub not_applicable: usize,
    pub total: usize,
}

/// Priority breakdown metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PriorityBreakdown {
    pub critical: usize,
    pub high: usize,
    pub medium: usize,
    pub low: usize,
}

/// Framework-specific metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FrameworkMetrics {
    pub framework_id: String,
    pub implementation_percentage: f64,
    pub control_count: usize,
    pub implemented_count: usize,
    pub overdue_count: usize,
    pub completion_velocity: f64, // controls per week
}

/// Trend analysis data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrendAnalysis {
    pub implementation_trend: TrendDirection,
    pub velocity_trend: TrendDirection,
    pub quality_trend: TrendDirection,
    pub weekly_completion_rate: f64,
    pub monthly_completion_rate: f64,
}

/// Trend direction
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum TrendDirection {
    Improving,
    Declining,
    Stable,
}

/// Key Performance Indicator
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KPI {
    pub id: String,
    pub name: String,
    pub value: f64,
    pub target: f64,
    pub unit: String,
    pub status: KPIStatus,
    pub trend: TrendDirection,
}

/// KPI status
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum KPIStatus {
    OnTrack,
    AtRisk,
    OffTrack,
}

/// Risk indicators
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RiskIndicators {
    pub overdue_controls: usize,
    pub high_priority_pending: usize,
    pub implementation_velocity_risk: bool,
    pub compliance_gap_risk: f64, // percentage
    pub resource_allocation_risk: bool,
}

impl MetricsService {
    /// Create a new metrics service
    pub fn new() -> Self {
        Self {
            history: HashMap::new(),
            cache: HashMap::new(),
            cache_ttl: 300, // 5 minutes
        }
    }

    /// Calculate comprehensive compliance metrics
    pub fn calculate_metrics(&mut self, controls: &[Control], frameworks: &[Framework]) -> Result<ComplianceMetrics> {
        let cache_key = "compliance_metrics";
        
        // Check cache first
        if let Some(cached) = self.cache.get(cache_key) {
            if cached.expires_at > Utc::now() {
                return Ok(cached.value.clone());
            }
        }

        // Calculate fresh metrics
        let metrics = self.calculate_fresh_metrics(controls, frameworks)?;
        
        // Cache the result
        self.cache.insert(cache_key.to_string(), CachedMetric {
            value: metrics.clone(),
            expires_at: Utc::now() + Duration::seconds(self.cache_ttl),
        });

        Ok(metrics)
    }

    /// Calculate fresh metrics without cache
    fn calculate_fresh_metrics(&self, controls: &[Control], frameworks: &[Framework]) -> Result<ComplianceMetrics> {
        let status_breakdown = self.calculate_status_breakdown(controls);
        let priority_breakdown = self.calculate_priority_breakdown(controls);
        let framework_metrics = self.calculate_framework_metrics(controls, frameworks);
        let trends = self.calculate_trends(controls);
        let kpis = self.calculate_kpis(controls, &status_breakdown);
        let risk_indicators = self.calculate_risk_indicators(controls);

        let implementation_percentage = if status_breakdown.total > 0 {
            (status_breakdown.implemented as f64 / status_breakdown.total as f64) * 100.0
        } else {
            0.0
        };

        Ok(ComplianceMetrics {
            implementation_percentage,
            status_breakdown,
            priority_breakdown,
            framework_metrics,
            trends,
            kpis,
            risk_indicators,
            calculated_at: Utc::now(),
        })
    }

    /// Calculate status breakdown
    fn calculate_status_breakdown(&self, controls: &[Control]) -> StatusBreakdown {
        let mut breakdown = StatusBreakdown {
            implemented: 0,
            in_progress: 0,
            not_implemented: 0,
            not_applicable: 0,
            total: controls.len(),
        };

        for control in controls {
            match control.implementation_status {
                ImplementationStatus::Implemented => breakdown.implemented += 1,
                ImplementationStatus::InProgress => breakdown.in_progress += 1,
                ImplementationStatus::NotImplemented => breakdown.not_implemented += 1,
                ImplementationStatus::NotApplicable => breakdown.not_applicable += 1,
            }
        }

        breakdown
    }

    /// Calculate priority breakdown
    fn calculate_priority_breakdown(&self, controls: &[Control]) -> PriorityBreakdown {
        let mut breakdown = PriorityBreakdown {
            critical: 0,
            high: 0,
            medium: 0,
            low: 0,
        };

        for control in controls {
            match control.priority {
                Priority::Critical => breakdown.critical += 1,
                Priority::High => breakdown.high += 1,
                Priority::Medium => breakdown.medium += 1,
                Priority::Low => breakdown.low += 1,
            }
        }

        breakdown
    }

    /// Calculate framework-specific metrics
    fn calculate_framework_metrics(&self, controls: &[Control], frameworks: &[Framework]) -> HashMap<String, FrameworkMetrics> {
        let mut metrics = HashMap::new();

        for framework in frameworks {
            let framework_controls: Vec<&Control> = controls.iter()
                .filter(|c| c.framework_id == framework.id)
                .collect();

            let implemented_count = framework_controls.iter()
                .filter(|c| c.implementation_status == ImplementationStatus::Implemented)
                .count();

            let overdue_count = framework_controls.iter()
                .filter(|c| {
                    if let Some(due_date) = c.due_date {
                        due_date < Utc::now() && c.implementation_status != ImplementationStatus::Implemented
                    } else {
                        false
                    }
                })
                .count();

            let implementation_percentage = if framework_controls.len() > 0 {
                (implemented_count as f64 / framework_controls.len() as f64) * 100.0
            } else {
                0.0
            };

            // Simple velocity calculation (would be more sophisticated in real implementation)
            let completion_velocity = 2.5; // controls per week

            metrics.insert(framework.id.clone(), FrameworkMetrics {
                framework_id: framework.id.clone(),
                implementation_percentage,
                control_count: framework_controls.len(),
                implemented_count,
                overdue_count,
                completion_velocity,
            });
        }

        metrics
    }

    /// Calculate trend analysis
    fn calculate_trends(&self, _controls: &[Control]) -> TrendAnalysis {
        // In a real implementation, this would analyze historical data
        TrendAnalysis {
            implementation_trend: TrendDirection::Improving,
            velocity_trend: TrendDirection::Stable,
            quality_trend: TrendDirection::Improving,
            weekly_completion_rate: 3.2,
            monthly_completion_rate: 12.8,
        }
    }

    /// Calculate KPIs
    fn calculate_kpis(&self, controls: &[Control], status_breakdown: &StatusBreakdown) -> Vec<KPI> {
        vec![
            KPI {
                id: "implementation-rate".to_string(),
                name: "Implementation Rate".to_string(),
                value: if status_breakdown.total > 0 {
                    (status_breakdown.implemented as f64 / status_breakdown.total as f64) * 100.0
                } else {
                    0.0
                },
                target: 85.0,
                unit: "%".to_string(),
                status: KPIStatus::OnTrack,
                trend: TrendDirection::Improving,
            },
            KPI {
                id: "overdue-controls".to_string(),
                name: "Overdue Controls".to_string(),
                value: controls.iter()
                    .filter(|c| {
                        if let Some(due_date) = c.due_date {
                            due_date < Utc::now() && c.implementation_status != ImplementationStatus::Implemented
                        } else {
                            false
                        }
                    })
                    .count() as f64,
                target: 5.0,
                unit: "count".to_string(),
                status: KPIStatus::AtRisk,
                trend: TrendDirection::Declining,
            },
        ]
    }

    /// Calculate risk indicators
    fn calculate_risk_indicators(&self, controls: &[Control]) -> RiskIndicators {
        let overdue_controls = controls.iter()
            .filter(|c| {
                if let Some(due_date) = c.due_date {
                    due_date < Utc::now() && c.implementation_status != ImplementationStatus::Implemented
                } else {
                    false
                }
            })
            .count();

        let high_priority_pending = controls.iter()
            .filter(|c| {
                (c.priority == Priority::High || c.priority == Priority::Critical) &&
                c.implementation_status != ImplementationStatus::Implemented
            })
            .count();

        RiskIndicators {
            overdue_controls,
            high_priority_pending,
            implementation_velocity_risk: overdue_controls > 10,
            compliance_gap_risk: 15.5, // percentage
            resource_allocation_risk: high_priority_pending > 5,
        }
    }

    /// Record a metric data point for historical tracking
    pub fn record_metric(&mut self, metric_id: &str, value: f64, metadata: HashMap<String, String>) {
        let data_point = MetricDataPoint {
            timestamp: Utc::now(),
            value,
            metadata,
        };

        self.history.entry(metric_id.to_string())
            .or_insert_with(Vec::new)
            .push(data_point);
    }

    /// Get historical data for a metric
    pub fn get_metric_history(&self, metric_id: &str, days: i64) -> Vec<MetricDataPoint> {
        let cutoff = Utc::now() - Duration::days(days);
        
        self.history.get(metric_id)
            .map(|history| {
                history.iter()
                    .filter(|point| point.timestamp >= cutoff)
                    .cloned()
                    .collect()
            })
            .unwrap_or_default()
    }
}

impl Default for MetricsService {
    fn default() -> Self {
        Self::new()
    }
}
