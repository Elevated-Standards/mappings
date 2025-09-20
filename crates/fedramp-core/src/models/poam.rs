// Modified: 2025-09-20

//! POA&M (Plan of Action and Milestones) models for FedRAMP compliance automation.
//!
//! This module defines data structures for managing security findings,
//! vulnerabilities, and remediation plans.

use crate::types::{EntityId, Result, RiskLevel, Timestamp};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;
use validator::Validate;

/// POA&M finding status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum FindingStatus {
    /// Finding is open and needs attention
    Open,
    /// Finding is being investigated
    InProgress,
    /// Finding has been remediated
    Remediated,
    /// Finding has been accepted as risk
    RiskAccepted,
    /// Finding is a false positive
    FalsePositive,
    /// Finding has been closed
    Closed,
}

/// Source of the finding
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum FindingSource {
    /// Security assessment
    SecurityAssessment,
    /// Vulnerability scan
    VulnerabilityScanning,
    /// Penetration testing
    PenetrationTesting,
    /// Code review
    CodeReview,
    /// Continuous monitoring
    ContinuousMonitoring,
    /// Incident response
    IncidentResponse,
    /// Self-assessment
    SelfAssessment,
    /// Third-party assessment
    ThirdPartyAssessment,
}

/// POA&M finding/weakness
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct PoamFinding {
    /// Unique finding identifier
    pub id: EntityId,
    /// Finding title
    #[validate(length(min = 1, max = 255))]
    pub title: String,
    /// Finding description
    #[validate(length(min = 1))]
    pub description: String,
    /// Finding status
    pub status: FindingStatus,
    /// Risk level/severity
    pub risk_level: RiskLevel,
    /// Source of the finding
    pub source: FindingSource,
    /// Control identifier(s) affected
    pub affected_controls: Vec<String>,
    /// System components affected
    pub affected_components: Vec<EntityId>,
    /// CVE identifiers (if applicable)
    pub cve_ids: Vec<String>,
    /// CWE identifiers (if applicable)
    pub cwe_ids: Vec<String>,
    /// CVSS score
    pub cvss_score: Option<f32>,
    /// Finding details
    pub details: FindingDetails,
    /// Remediation plan
    pub remediation: RemediationPlan,
    /// Finding metadata
    pub metadata: HashMap<String, String>,
    /// Creation timestamp
    pub created_at: Timestamp,
    /// Last update timestamp
    pub updated_at: Timestamp,
    /// Created by user ID
    pub created_by: EntityId,
    /// Last updated by user ID
    pub updated_by: EntityId,
    /// Assigned to user ID
    pub assigned_to: Option<EntityId>,
}

/// Detailed finding information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FindingDetails {
    /// Technical details of the finding
    pub technical_details: Option<String>,
    /// Steps to reproduce (if applicable)
    pub reproduction_steps: Option<String>,
    /// Evidence/proof of the finding
    pub evidence: Vec<FindingEvidence>,
    /// Impact assessment
    pub impact_assessment: String,
    /// Likelihood assessment
    pub likelihood: Option<String>,
    /// Business impact
    pub business_impact: Option<String>,
}

/// Evidence supporting a finding
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FindingEvidence {
    /// Evidence type
    pub evidence_type: EvidenceType,
    /// Evidence description
    pub description: String,
    /// File path or URL to evidence
    pub location: Option<String>,
    /// Evidence timestamp
    pub timestamp: Timestamp,
    /// Evidence metadata
    pub metadata: HashMap<String, String>,
}

/// Types of evidence
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum EvidenceType {
    /// Screenshot
    Screenshot,
    /// Log file
    LogFile,
    /// Scan report
    ScanReport,
    /// Configuration file
    ConfigFile,
    /// Network capture
    NetworkCapture,
    /// Documentation
    Documentation,
    /// Other evidence
    Other,
}

/// Remediation plan for a finding
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct RemediationPlan {
    /// Planned remediation actions
    pub planned_actions: String,
    /// Remediation timeline
    pub timeline: RemediationTimeline,
    /// Resources required
    pub resources_required: Option<String>,
    /// Cost estimate
    pub cost_estimate: Option<f64>,
    /// Remediation priority
    pub priority: RemediationPriority,
    /// Milestones
    pub milestones: Vec<RemediationMilestone>,
    /// Completion percentage
    #[validate(range(min = 0, max = 100))]
    pub completion_percentage: u8,
    /// Remediation notes
    pub notes: Option<String>,
}

/// Remediation timeline
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RemediationTimeline {
    /// Planned start date
    pub planned_start: Option<Timestamp>,
    /// Planned completion date
    pub planned_completion: Option<Timestamp>,
    /// Actual start date
    pub actual_start: Option<Timestamp>,
    /// Actual completion date
    pub actual_completion: Option<Timestamp>,
    /// Extended deadline (if applicable)
    pub extended_deadline: Option<Timestamp>,
    /// Reason for extension
    pub extension_reason: Option<String>,
}

/// Remediation priority levels
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum RemediationPriority {
    /// Critical priority
    Critical,
    /// High priority
    High,
    /// Medium priority
    Medium,
    /// Low priority
    Low,
}

/// Remediation milestone
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RemediationMilestone {
    /// Milestone ID
    pub id: EntityId,
    /// Milestone description
    pub description: String,
    /// Target date
    pub target_date: Timestamp,
    /// Completion date
    pub completion_date: Option<Timestamp>,
    /// Milestone status
    pub status: MilestoneStatus,
    /// Milestone notes
    pub notes: Option<String>,
}

/// Milestone status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum MilestoneStatus {
    /// Not started
    NotStarted,
    /// In progress
    InProgress,
    /// Completed
    Completed,
    /// Delayed
    Delayed,
    /// Cancelled
    Cancelled,
}

impl PoamFinding {
    /// Create a new POA&M finding
    pub fn new(
        title: String,
        description: String,
        risk_level: RiskLevel,
        source: FindingSource,
        created_by: EntityId,
    ) -> Self {
        let now = crate::utils::current_timestamp();
        let id = crate::utils::generate_uuid();

        Self {
            id,
            title,
            description,
            status: FindingStatus::Open,
            risk_level,
            source,
            affected_controls: Vec::new(),
            affected_components: Vec::new(),
            cve_ids: Vec::new(),
            cwe_ids: Vec::new(),
            cvss_score: None,
            details: FindingDetails::default(),
            remediation: RemediationPlan::default(),
            metadata: HashMap::new(),
            created_at: now,
            updated_at: now,
            created_by,
            updated_by: created_by,
            assigned_to: None,
        }
    }

    /// Update finding status
    pub fn update_status(&mut self, status: FindingStatus, updated_by: EntityId) {
        self.status = status;
        self.updated_at = crate::utils::current_timestamp();
        self.updated_by = updated_by;
    }

    /// Assign finding to a user
    pub fn assign_to(&mut self, user_id: EntityId, updated_by: EntityId) {
        self.assigned_to = Some(user_id);
        self.updated_at = crate::utils::current_timestamp();
        self.updated_by = updated_by;
    }

    /// Check if finding is overdue
    pub fn is_overdue(&self) -> bool {
        if let Some(planned_completion) = self.remediation.timeline.planned_completion {
            let now = crate::utils::current_timestamp();
            now > planned_completion && !self.is_closed()
        } else {
            false
        }
    }

    /// Check if finding is closed
    pub fn is_closed(&self) -> bool {
        matches!(
            self.status,
            FindingStatus::Remediated
                | FindingStatus::RiskAccepted
                | FindingStatus::FalsePositive
                | FindingStatus::Closed
        )
    }

    /// Get finding age in days
    pub fn age_days(&self) -> i64 {
        let now = crate::utils::current_timestamp();
        (now - self.created_at).num_days()
    }
}

impl Default for FindingDetails {
    fn default() -> Self {
        Self {
            technical_details: None,
            reproduction_steps: None,
            evidence: Vec::new(),
            impact_assessment: "To be determined".to_string(),
            likelihood: None,
            business_impact: None,
        }
    }
}

impl Default for RemediationPlan {
    fn default() -> Self {
        Self {
            planned_actions: "To be determined".to_string(),
            timeline: RemediationTimeline::default(),
            resources_required: None,
            cost_estimate: None,
            priority: RemediationPriority::Medium,
            milestones: Vec::new(),
            completion_percentage: 0,
            notes: None,
        }
    }
}

impl Default for RemediationTimeline {
    fn default() -> Self {
        Self {
            planned_start: None,
            planned_completion: None,
            actual_start: None,
            actual_completion: None,
            extended_deadline: None,
            extension_reason: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_finding_creation() {
        let user_id = Uuid::new_v4();
        let finding = PoamFinding::new(
            "SQL Injection Vulnerability".to_string(),
            "Application is vulnerable to SQL injection attacks".to_string(),
            RiskLevel::High,
            FindingSource::VulnerabilityScanning,
            user_id,
        );

        assert_eq!(finding.title, "SQL Injection Vulnerability");
        assert_eq!(finding.risk_level, RiskLevel::High);
        assert_eq!(finding.status, FindingStatus::Open);
        assert_eq!(finding.created_by, user_id);
        assert!(!finding.is_closed());
    }

    #[test]
    fn test_finding_status_update() {
        let user_id = Uuid::new_v4();
        let mut finding = PoamFinding::new(
            "Test Finding".to_string(),
            "Test description".to_string(),
            RiskLevel::Medium,
            FindingSource::SecurityAssessment,
            user_id,
        );

        finding.update_status(FindingStatus::Remediated, user_id);
        assert_eq!(finding.status, FindingStatus::Remediated);
        assert!(finding.is_closed());
    }

    #[test]
    fn test_finding_assignment() {
        let user_id = Uuid::new_v4();
        let assignee_id = Uuid::new_v4();
        let mut finding = PoamFinding::new(
            "Test Finding".to_string(),
            "Test description".to_string(),
            RiskLevel::Low,
            FindingSource::SelfAssessment,
            user_id,
        );

        finding.assign_to(assignee_id, user_id);
        assert_eq!(finding.assigned_to, Some(assignee_id));
    }
}
