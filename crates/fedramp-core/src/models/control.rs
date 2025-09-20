// Modified: 2025-01-20

//! Security control models

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use schemars::JsonSchema;
use uuid::Uuid;
use validator::Validate;

/// Security control implementation status
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum ImplementationStatus {
    Implemented,
    PartiallyImplemented,
    Planned,
    AlternativeImplementation,
    NotApplicable,
}

/// Control responsibility type
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum ResponsibilityType {
    Customer,
    Csp,
    Shared,
    Inherited,
}

/// Security framework type
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum Framework {
    #[serde(rename = "NIST-800-53")]
    Nist80053,
    #[serde(rename = "NIST-800-171")]
    Nist800171,
    #[serde(rename = "CIS")]
    Cis,
    #[serde(rename = "Custom")]
    Custom,
}

/// Control enhancement
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, Validate)]
pub struct ControlEnhancement {
    pub enhancement_id: String,
    pub enhancement_title: Option<String>,
    pub implementation_status: ImplementationStatus,
    pub notes: Option<String>,
}

/// Security control
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, Validate)]
pub struct Control {
    pub id: Uuid,
    #[validate(length(min = 1))]
    pub control_id: String,
    #[validate(length(min = 1))]
    pub control_title: String,
    pub control_description: Option<String>,
    pub implementation_status: ImplementationStatus,
    pub customer_responsibility: Option<String>,
    pub csp_responsibility: Option<String>,
    pub shared_responsibility: Option<String>,
    pub implementation_guidance: Option<String>,
    pub assessment_procedures: Option<String>,
    pub notes: Option<String>,
    pub control_enhancements: Vec<ControlEnhancement>,
    pub framework: Framework,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Control mapping between frameworks
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, Validate)]
pub struct ControlMapping {
    pub id: Uuid,
    #[validate(length(min = 1))]
    pub source_control_id: String,
    pub source_framework: Framework,
    #[validate(length(min = 1))]
    pub target_control_id: String,
    pub target_framework: Framework,
    #[validate(range(min = 0.0, max = 1.0))]
    pub confidence_score: f64,
    pub mapping_type: MappingType,
    pub notes: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Type of control mapping relationship
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum MappingType {
    Exact,
    Partial,
    Related,
    Derived,
}

/// Control family
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, Validate)]
pub struct ControlFamily {
    pub id: Uuid,
    #[validate(length(min = 1))]
    pub family_id: String,
    #[validate(length(min = 1))]
    pub family_name: String,
    pub description: Option<String>,
    pub framework: Framework,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Control baseline (Low, Moderate, High)
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum BaselineLevel {
    Low,
    Moderate,
    High,
}

/// Control baseline definition
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, Validate)]
pub struct ControlBaseline {
    pub id: Uuid,
    #[validate(length(min = 1))]
    pub baseline_name: String,
    pub baseline_level: BaselineLevel,
    pub framework: Framework,
    pub control_ids: Vec<String>,
    pub description: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl Control {
    /// Create a new control
    pub fn new(
        control_id: String,
        control_title: String,
        framework: Framework,
        implementation_status: ImplementationStatus,
    ) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4(),
            control_id,
            control_title,
            control_description: None,
            implementation_status,
            customer_responsibility: None,
            csp_responsibility: None,
            shared_responsibility: None,
            implementation_guidance: None,
            assessment_procedures: None,
            notes: None,
            control_enhancements: Vec::new(),
            framework,
            created_at: now,
            updated_at: now,
        }
    }

    /// Check if control is fully implemented
    pub fn is_implemented(&self) -> bool {
        self.implementation_status == ImplementationStatus::Implemented
    }

    /// Get primary responsibility type
    pub fn primary_responsibility(&self) -> ResponsibilityType {
        if self.customer_responsibility.is_some() && self.csp_responsibility.is_some() {
            ResponsibilityType::Shared
        } else if self.customer_responsibility.is_some() {
            ResponsibilityType::Customer
        } else if self.csp_responsibility.is_some() {
            ResponsibilityType::Csp
        } else {
            ResponsibilityType::Inherited
        }
    }
}

impl ControlMapping {
    /// Create a new control mapping
    pub fn new(
        source_control_id: String,
        source_framework: Framework,
        target_control_id: String,
        target_framework: Framework,
        confidence_score: f64,
        mapping_type: MappingType,
    ) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4(),
            source_control_id,
            source_framework,
            target_control_id,
            target_framework,
            confidence_score,
            mapping_type,
            notes: None,
            created_at: now,
            updated_at: now,
        }
    }

    /// Check if mapping has high confidence
    pub fn is_high_confidence(&self) -> bool {
        self.confidence_score >= 0.8
    }
}
