// Modified: 2025-01-20

//! Audit models for FedRAMP compliance automation.
//!
//! This module defines data structures for audit trails,
//! change tracking, and compliance logging.

use crate::types::{EntityId, Result, Timestamp};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;
use validator::Validate;

/// Audit log entry
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct AuditLogEntry {
    /// Unique audit entry identifier
    pub id: EntityId,
    /// Event type
    pub event_type: AuditEventType,
    /// Event category
    pub category: AuditCategory,
    /// Event severity level
    pub severity: AuditSeverity,
    /// User who performed the action
    pub user_id: Option<EntityId>,
    /// Session ID (if applicable)
    pub session_id: Option<String>,
    /// Resource type affected
    pub resource_type: String,
    /// Resource ID affected
    pub resource_id: Option<EntityId>,
    /// Action performed
    #[validate(length(min = 1, max = 255))]
    pub action: String,
    /// Event description
    #[validate(length(min = 1))]
    pub description: String,
    /// Event details (structured data)
    pub details: AuditEventDetails,
    /// Event outcome
    pub outcome: AuditOutcome,
    /// Error message (if applicable)
    pub error_message: Option<String>,
    /// IP address of the client
    pub client_ip: Option<String>,
    /// User agent string
    pub user_agent: Option<String>,
    /// Request ID for correlation
    pub request_id: Option<String>,
    /// Event timestamp
    pub timestamp: Timestamp,
    /// Additional metadata
    pub metadata: HashMap<String, String>,
}

/// Types of audit events
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum AuditEventType {
    /// User authentication events
    Authentication,
    /// User authorization events
    Authorization,
    /// Data access events
    DataAccess,
    /// Data modification events
    DataModification,
    /// System configuration changes
    SystemConfiguration,
    /// Security events
    Security,
    /// Compliance events
    Compliance,
    /// Administrative actions
    Administrative,
    /// System events
    System,
    /// Error events
    Error,
}

/// Audit event categories
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum AuditCategory {
    /// User management
    UserManagement,
    /// Document management
    DocumentManagement,
    /// Finding management
    FindingManagement,
    /// System inventory
    SystemInventory,
    /// Control assessment
    ControlAssessment,
    /// Risk management
    RiskManagement,
    /// Compliance reporting
    ComplianceReporting,
    /// System administration
    SystemAdministration,
    /// Security monitoring
    SecurityMonitoring,
    /// Data export/import
    DataTransfer,
}

/// Audit event severity levels
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum AuditSeverity {
    /// Informational events
    Info,
    /// Warning events
    Warning,
    /// Error events
    Error,
    /// Critical events
    Critical,
}

/// Audit event outcome
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum AuditOutcome {
    /// Action succeeded
    Success,
    /// Action failed
    Failure,
    /// Action was denied
    Denied,
    /// Action is pending
    Pending,
}

/// Detailed audit event information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditEventDetails {
    /// Previous values (for modification events)
    pub previous_values: Option<serde_json::Value>,
    /// New values (for modification events)
    pub new_values: Option<serde_json::Value>,
    /// Fields that were changed
    pub changed_fields: Vec<String>,
    /// Additional context data
    pub context: HashMap<String, serde_json::Value>,
    /// Related entities
    pub related_entities: Vec<RelatedEntity>,
}

/// Related entity in an audit event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RelatedEntity {
    /// Entity type
    pub entity_type: String,
    /// Entity ID
    pub entity_id: EntityId,
    /// Relationship to the main event
    pub relationship: String,
}

/// Compliance audit trail
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComplianceAuditTrail {
    /// Trail ID
    pub id: EntityId,
    /// Compliance framework
    pub framework: String,
    /// Control identifier
    pub control_id: String,
    /// Audit events in this trail
    pub events: Vec<EntityId>,
    /// Trail start date
    pub start_date: Timestamp,
    /// Trail end date
    pub end_date: Option<Timestamp>,
    /// Trail status
    pub status: AuditTrailStatus,
    /// Auditor information
    pub auditor: Option<AuditorInfo>,
    /// Audit findings
    pub findings: Vec<AuditFinding>,
    /// Creation timestamp
    pub created_at: Timestamp,
    /// Last update timestamp
    pub updated_at: Timestamp,
}

/// Audit trail status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum AuditTrailStatus {
    /// Trail is active
    Active,
    /// Trail is completed
    Completed,
    /// Trail is suspended
    Suspended,
    /// Trail is archived
    Archived,
}

/// Auditor information
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct AuditorInfo {
    /// Auditor name
    #[validate(length(min = 1, max = 255))]
    pub name: String,
    /// Auditor organization
    pub organization: Option<String>,
    /// Auditor email
    #[validate(email)]
    pub email: String,
    /// Auditor certification
    pub certification: Option<String>,
    /// Audit start date
    pub audit_start_date: Timestamp,
    /// Expected completion date
    pub expected_completion_date: Option<Timestamp>,
}

/// Audit finding
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditFinding {
    /// Finding ID
    pub id: EntityId,
    /// Finding type
    pub finding_type: AuditFindingType,
    /// Finding severity
    pub severity: AuditSeverity,
    /// Finding description
    pub description: String,
    /// Affected controls
    pub affected_controls: Vec<String>,
    /// Evidence
    pub evidence: Vec<String>,
    /// Recommendations
    pub recommendations: Vec<String>,
    /// Finding status
    pub status: AuditFindingStatus,
    /// Response from organization
    pub response: Option<String>,
    /// Corrective action plan
    pub corrective_action_plan: Option<String>,
    /// Target completion date
    pub target_completion_date: Option<Timestamp>,
    /// Actual completion date
    pub actual_completion_date: Option<Timestamp>,
    /// Creation timestamp
    pub created_at: Timestamp,
    /// Last update timestamp
    pub updated_at: Timestamp,
}

/// Types of audit findings
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum AuditFindingType {
    /// Control deficiency
    ControlDeficiency,
    /// Documentation gap
    DocumentationGap,
    /// Implementation weakness
    ImplementationWeakness,
    /// Process improvement
    ProcessImprovement,
    /// Compliance violation
    ComplianceViolation,
    /// Best practice recommendation
    BestPractice,
}

/// Audit finding status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum AuditFindingStatus {
    /// Finding is open
    Open,
    /// Finding is being addressed
    InProgress,
    /// Finding has been resolved
    Resolved,
    /// Finding has been accepted as risk
    Accepted,
    /// Finding is disputed
    Disputed,
    /// Finding is closed
    Closed,
}

impl AuditLogEntry {
    /// Create a new audit log entry
    pub fn new(
        event_type: AuditEventType,
        category: AuditCategory,
        action: String,
        description: String,
        user_id: Option<EntityId>,
    ) -> Self {
        let id = crate::utils::generate_uuid();
        let timestamp = crate::utils::current_timestamp();

        Self {
            id,
            event_type,
            category,
            severity: AuditSeverity::Info,
            user_id,
            session_id: None,
            resource_type: "unknown".to_string(),
            resource_id: None,
            action,
            description,
            details: AuditEventDetails::default(),
            outcome: AuditOutcome::Success,
            error_message: None,
            client_ip: None,
            user_agent: None,
            request_id: None,
            timestamp,
            metadata: HashMap::new(),
        }
    }

    /// Set the outcome of the audit event
    pub fn with_outcome(mut self, outcome: AuditOutcome) -> Self {
        self.outcome = outcome;
        self
    }

    /// Set the severity of the audit event
    pub fn with_severity(mut self, severity: AuditSeverity) -> Self {
        self.severity = severity;
        self
    }

    /// Set the resource information
    pub fn with_resource(mut self, resource_type: String, resource_id: Option<EntityId>) -> Self {
        self.resource_type = resource_type;
        self.resource_id = resource_id;
        self
    }

    /// Add metadata to the audit event
    pub fn with_metadata(mut self, key: String, value: String) -> Self {
        self.metadata.insert(key, value);
        self
    }

    /// Set error message for failed events
    pub fn with_error(mut self, error_message: String) -> Self {
        self.error_message = Some(error_message);
        self.outcome = AuditOutcome::Failure;
        self.severity = AuditSeverity::Error;
        self
    }
}

impl Default for AuditEventDetails {
    fn default() -> Self {
        Self {
            previous_values: None,
            new_values: None,
            changed_fields: Vec::new(),
            context: HashMap::new(),
            related_entities: Vec::new(),
        }
    }
}

impl ComplianceAuditTrail {
    /// Create a new compliance audit trail
    pub fn new(framework: String, control_id: String) -> Self {
        let id = crate::utils::generate_uuid();
        let now = crate::utils::current_timestamp();

        Self {
            id,
            framework,
            control_id,
            events: Vec::new(),
            start_date: now,
            end_date: None,
            status: AuditTrailStatus::Active,
            auditor: None,
            findings: Vec::new(),
            created_at: now,
            updated_at: now,
        }
    }

    /// Add an audit event to the trail
    pub fn add_event(&mut self, event_id: EntityId) {
        self.events.push(event_id);
        self.updated_at = crate::utils::current_timestamp();
    }

    /// Complete the audit trail
    pub fn complete(&mut self) {
        self.status = AuditTrailStatus::Completed;
        self.end_date = Some(crate::utils::current_timestamp());
        self.updated_at = crate::utils::current_timestamp();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_audit_log_entry_creation() {
        let user_id = Uuid::new_v4();
        let entry = AuditLogEntry::new(
            AuditEventType::Authentication,
            AuditCategory::UserManagement,
            "login".to_string(),
            "User logged in successfully".to_string(),
            Some(user_id),
        );

        assert_eq!(entry.event_type, AuditEventType::Authentication);
        assert_eq!(entry.category, AuditCategory::UserManagement);
        assert_eq!(entry.action, "login");
        assert_eq!(entry.user_id, Some(user_id));
        assert_eq!(entry.outcome, AuditOutcome::Success);
        assert_eq!(entry.severity, AuditSeverity::Info);
    }

    #[test]
    fn test_audit_log_entry_builder() {
        let entry = AuditLogEntry::new(
            AuditEventType::DataModification,
            AuditCategory::DocumentManagement,
            "update".to_string(),
            "Document updated".to_string(),
            None,
        )
        .with_outcome(AuditOutcome::Success)
        .with_severity(AuditSeverity::Info)
        .with_resource("document".to_string(), Some(Uuid::new_v4()))
        .with_metadata("version".to_string(), "1.1".to_string());

        assert_eq!(entry.outcome, AuditOutcome::Success);
        assert_eq!(entry.severity, AuditSeverity::Info);
        assert_eq!(entry.resource_type, "document");
        assert!(entry.metadata.contains_key("version"));
    }

    #[test]
    fn test_compliance_audit_trail() {
        let mut trail = ComplianceAuditTrail::new(
            "NIST 800-53".to_string(),
            "AC-1".to_string(),
        );

        let event_id = Uuid::new_v4();
        trail.add_event(event_id);

        assert_eq!(trail.framework, "NIST 800-53");
        assert_eq!(trail.control_id, "AC-1");
        assert_eq!(trail.status, AuditTrailStatus::Active);
        assert!(trail.events.contains(&event_id));

        trail.complete();
        assert_eq!(trail.status, AuditTrailStatus::Completed);
        assert!(trail.end_date.is_some());
    }
}
