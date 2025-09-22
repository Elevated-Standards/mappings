//! POA&M (Plan of Action and Milestones) data structures
//! 
//! Simple POA&M item structure for quality assessment and processing

use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};

/// Simple POA&M item structure for quality assessment
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PoamItem {
    /// Unique identifier for the POA&M item
    pub uuid: String,
    /// Title/name of the weakness or vulnerability
    pub title: String,
    /// Detailed description of the issue
    pub description: String,
    /// Current status of the POA&M item
    pub status: String,
    /// Severity level (Critical, High, Medium, Low, etc.)
    pub severity: Option<String>,
    /// Scheduled completion date
    pub scheduled_completion_date: Option<String>,
    /// Actual completion date
    pub actual_completion_date: Option<String>,
    /// Responsible entity or person
    pub responsible_entity: Option<String>,
    /// Resources required for remediation
    pub resources_required: Option<String>,
    /// Risk assessment information
    pub risk_assessment: Option<String>,
    /// List of milestones
    pub milestones: Option<Vec<PoamMilestone>>,
}

/// POA&M milestone structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PoamMilestone {
    /// Milestone description
    pub description: String,
    /// Scheduled date for this milestone
    pub scheduled_date: Option<String>,
    /// Actual completion date for this milestone
    pub actual_date: Option<String>,
    /// Status of this milestone
    pub status: Option<String>,
}

impl PoamItem {
    /// Create a new POA&M item with minimal required fields
    pub fn new(uuid: String, title: String, description: String, status: String) -> Self {
        Self {
            uuid,
            title,
            description,
            status,
            severity: None,
            scheduled_completion_date: None,
            actual_completion_date: None,
            responsible_entity: None,
            resources_required: None,
            risk_assessment: None,
            milestones: None,
        }
    }

    /// Create a sample POA&M item for testing
    pub fn sample() -> Self {
        Self {
            uuid: "550e8400-e29b-41d4-a716-446655440000".to_string(),
            title: "Sample Vulnerability".to_string(),
            description: "This is a sample vulnerability for testing purposes".to_string(),
            status: "Open".to_string(),
            severity: Some("High".to_string()),
            scheduled_completion_date: Some("2024-12-31T23:59:59Z".to_string()),
            actual_completion_date: None,
            responsible_entity: Some("Security Team".to_string()),
            resources_required: Some("2 FTE, Security tools".to_string()),
            risk_assessment: Some("High risk due to potential data exposure".to_string()),
            milestones: Some(vec![
                PoamMilestone {
                    description: "Initial assessment".to_string(),
                    scheduled_date: Some("2024-06-30T23:59:59Z".to_string()),
                    actual_date: None,
                    status: Some("In Progress".to_string()),
                },
                PoamMilestone {
                    description: "Remediation implementation".to_string(),
                    scheduled_date: Some("2024-11-30T23:59:59Z".to_string()),
                    actual_date: None,
                    status: Some("Not Started".to_string()),
                },
            ]),
        }
    }

    /// Check if the item is complete (has all required fields)
    pub fn is_complete(&self) -> bool {
        !self.uuid.is_empty()
            && !self.title.is_empty()
            && !self.description.is_empty()
            && !self.status.is_empty()
            && self.scheduled_completion_date.is_some()
    }

    /// Check if the item is closed/completed
    pub fn is_closed(&self) -> bool {
        matches!(self.status.as_str(), "Completed" | "Closed" | "Cancelled")
    }

    /// Get the completion percentage based on milestones
    pub fn completion_percentage(&self) -> f64 {
        if let Some(milestones) = &self.milestones {
            if milestones.is_empty() {
                return if self.is_closed() { 100.0 } else { 0.0 };
            }

            let completed_milestones = milestones.iter()
                .filter(|m| m.status.as_ref().map_or(false, |s| s == "Completed"))
                .count();

            (completed_milestones as f64 / milestones.len() as f64) * 100.0
        } else {
            if self.is_closed() { 100.0 } else { 0.0 }
        }
    }
}

impl PoamMilestone {
    /// Create a new milestone
    pub fn new(description: String, scheduled_date: Option<String>) -> Self {
        Self {
            description,
            scheduled_date,
            actual_date: None,
            status: Some("Not Started".to_string()),
        }
    }

    /// Check if the milestone is completed
    pub fn is_completed(&self) -> bool {
        self.status.as_ref().map_or(false, |s| s == "Completed")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_poam_item_creation() {
        let item = PoamItem::new(
            "test-uuid".to_string(),
            "Test Item".to_string(),
            "Test description".to_string(),
            "Open".to_string(),
        );

        assert_eq!(item.uuid, "test-uuid");
        assert_eq!(item.title, "Test Item");
        assert_eq!(item.description, "Test description");
        assert_eq!(item.status, "Open");
        assert!(!item.is_complete()); // Missing scheduled_completion_date
        assert!(!item.is_closed());
    }

    #[test]
    fn test_sample_poam_item() {
        let item = PoamItem::sample();

        assert!(!item.uuid.is_empty());
        assert!(!item.title.is_empty());
        assert!(!item.description.is_empty());
        assert_eq!(item.status, "Open");
        assert_eq!(item.severity, Some("High".to_string()));
        assert!(item.scheduled_completion_date.is_some());
        assert!(item.responsible_entity.is_some());
        assert!(item.milestones.is_some());
        assert!(item.is_complete());
        assert!(!item.is_closed());
    }

    #[test]
    fn test_completion_percentage() {
        let mut item = PoamItem::sample();
        
        // Initially no milestones completed
        assert_eq!(item.completion_percentage(), 0.0);

        // Complete first milestone
        if let Some(ref mut milestones) = item.milestones {
            milestones[0].status = Some("Completed".to_string());
        }
        assert_eq!(item.completion_percentage(), 50.0);

        // Complete all milestones
        if let Some(ref mut milestones) = item.milestones {
            milestones[1].status = Some("Completed".to_string());
        }
        assert_eq!(item.completion_percentage(), 100.0);
    }

    #[test]
    fn test_milestone_creation() {
        let milestone = PoamMilestone::new(
            "Test milestone".to_string(),
            Some("2024-12-31T23:59:59Z".to_string()),
        );

        assert_eq!(milestone.description, "Test milestone");
        assert_eq!(milestone.scheduled_date, Some("2024-12-31T23:59:59Z".to_string()));
        assert_eq!(milestone.status, Some("Not Started".to_string()));
        assert!(!milestone.is_completed());
    }

    #[test]
    fn test_item_status_checks() {
        let mut item = PoamItem::sample();

        // Initially open
        assert!(!item.is_closed());

        // Mark as completed
        item.status = "Completed".to_string();
        assert!(item.is_closed());

        // Mark as closed
        item.status = "Closed".to_string();
        assert!(item.is_closed());

        // Mark as cancelled
        item.status = "Cancelled".to_string();
        assert!(item.is_closed());
    }
}
