// Modified: 2025-01-20

//! Document models for FedRAMP compliance automation.
//!
//! This module defines data structures for various FedRAMP documents
//! including SSPs, POA&Ms, and other compliance artifacts.

use crate::types::{EntityId, Result, Timestamp};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;
use validator::Validate;

/// Document types supported by the platform
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum DocumentType {
    /// System Security Plan
    SystemSecurityPlan,
    /// Plan of Action and Milestones
    PlanOfActionMilestones,
    /// Security Assessment Plan
    SecurityAssessmentPlan,
    /// Security Assessment Report
    SecurityAssessmentReport,
    /// Continuous Monitoring Plan
    ContinuousMonitoringPlan,
    /// Incident Response Plan
    IncidentResponsePlan,
    /// Configuration Management Plan
    ConfigurationManagementPlan,
    /// Contingency Plan
    ContingencyPlan,
}

/// Document status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum DocumentStatus {
    /// Draft document
    Draft,
    /// Under review
    UnderReview,
    /// Approved
    Approved,
    /// Published
    Published,
    /// Archived
    Archived,
}

/// Base document structure
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct Document {
    /// Unique document identifier
    pub id: EntityId,
    /// Document type
    pub document_type: DocumentType,
    /// Document title
    #[validate(length(min = 1, max = 255))]
    pub title: String,
    /// Document version
    #[validate(length(min = 1, max = 50))]
    pub version: String,
    /// Document status
    pub status: DocumentStatus,
    /// Document description
    pub description: Option<String>,
    /// Document metadata
    pub metadata: DocumentMetadata,
    /// Document content (JSON structure)
    pub content: serde_json::Value,
    /// Creation timestamp
    pub created_at: Timestamp,
    /// Last update timestamp
    pub updated_at: Timestamp,
    /// Created by user ID
    pub created_by: EntityId,
    /// Last updated by user ID
    pub updated_by: EntityId,
}

/// Document metadata
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct DocumentMetadata {
    /// OSCAL version
    #[validate(length(min = 1))]
    pub oscal_version: String,
    /// FedRAMP template version
    #[validate(length(min = 1))]
    pub template_version: String,
    /// Document classification
    pub classification: String,
    /// Document tags
    pub tags: Vec<String>,
    /// Custom properties
    pub properties: HashMap<String, String>,
    /// Document checksum
    pub checksum: Option<String>,
    /// File size in bytes
    pub file_size: Option<u64>,
}

/// Document revision history
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentRevision {
    /// Revision ID
    pub id: EntityId,
    /// Document ID
    pub document_id: EntityId,
    /// Revision number
    pub revision: u32,
    /// Revision description
    pub description: String,
    /// Changes made in this revision
    pub changes: Vec<DocumentChange>,
    /// Revision timestamp
    pub created_at: Timestamp,
    /// Created by user ID
    pub created_by: EntityId,
}

/// Document change record
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentChange {
    /// Change type
    pub change_type: ChangeType,
    /// Field or section that was changed
    pub field: String,
    /// Previous value (if applicable)
    pub old_value: Option<serde_json::Value>,
    /// New value
    pub new_value: serde_json::Value,
    /// Change description
    pub description: Option<String>,
}

/// Types of changes that can be made to a document
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum ChangeType {
    /// Field was added
    Added,
    /// Field was modified
    Modified,
    /// Field was removed
    Removed,
    /// Section was restructured
    Restructured,
}

/// Document template definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentTemplate {
    /// Template ID
    pub id: EntityId,
    /// Template name
    pub name: String,
    /// Document type this template applies to
    pub document_type: DocumentType,
    /// Template version
    pub version: String,
    /// Template schema (JSON Schema)
    pub schema: serde_json::Value,
    /// Default values
    pub defaults: serde_json::Value,
    /// Required fields
    pub required_fields: Vec<String>,
    /// Template description
    pub description: Option<String>,
    /// Creation timestamp
    pub created_at: Timestamp,
    /// Last update timestamp
    pub updated_at: Timestamp,
}

impl Document {
    /// Create a new document
    pub fn new(
        document_type: DocumentType,
        title: String,
        version: String,
        created_by: EntityId,
    ) -> Self {
        let now = crate::utils::current_timestamp();
        let id = crate::utils::generate_uuid();

        Self {
            id,
            document_type,
            title,
            version,
            status: DocumentStatus::Draft,
            description: None,
            metadata: DocumentMetadata::default(),
            content: serde_json::Value::Object(serde_json::Map::new()),
            created_at: now,
            updated_at: now,
            created_by,
            updated_by: created_by,
        }
    }

    /// Update document content
    pub fn update_content(&mut self, content: serde_json::Value, updated_by: EntityId) {
        self.content = content;
        self.updated_at = crate::utils::current_timestamp();
        self.updated_by = updated_by;
    }

    /// Change document status
    pub fn change_status(&mut self, status: DocumentStatus, updated_by: EntityId) {
        self.status = status;
        self.updated_at = crate::utils::current_timestamp();
        self.updated_by = updated_by;
    }

    /// Get document age in days
    pub fn age_days(&self) -> i64 {
        let now = crate::utils::current_timestamp();
        (now - self.created_at).num_days()
    }

    /// Check if document is editable
    pub fn is_editable(&self) -> bool {
        matches!(self.status, DocumentStatus::Draft | DocumentStatus::UnderReview)
    }
}

impl Default for DocumentMetadata {
    fn default() -> Self {
        Self {
            oscal_version: crate::types::OSCAL_VERSION.to_string(),
            template_version: crate::types::FEDRAMP_TEMPLATE_VERSION.to_string(),
            classification: "unclassified".to_string(),
            tags: Vec::new(),
            properties: HashMap::new(),
            checksum: None,
            file_size: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_document_creation() {
        let user_id = Uuid::new_v4();
        let doc = Document::new(
            DocumentType::SystemSecurityPlan,
            "Test SSP".to_string(),
            "1.0".to_string(),
            user_id,
        );

        assert_eq!(doc.document_type, DocumentType::SystemSecurityPlan);
        assert_eq!(doc.title, "Test SSP");
        assert_eq!(doc.version, "1.0");
        assert_eq!(doc.status, DocumentStatus::Draft);
        assert_eq!(doc.created_by, user_id);
        assert!(doc.is_editable());
    }

    #[test]
    fn test_document_status_change() {
        let user_id = Uuid::new_v4();
        let mut doc = Document::new(
            DocumentType::SystemSecurityPlan,
            "Test SSP".to_string(),
            "1.0".to_string(),
            user_id,
        );

        doc.change_status(DocumentStatus::Published, user_id);
        assert_eq!(doc.status, DocumentStatus::Published);
        assert!(!doc.is_editable());
    }
}
