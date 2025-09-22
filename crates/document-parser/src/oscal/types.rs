// Modified: 2025-09-22

//! OSCAL type definitions and data structures
//!
//! This module contains all the OSCAL data structures, enums, and type definitions
//! used for generating and validating OSCAL documents.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// OSCAL document types
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Hash)]
pub enum OscalDocumentType {
    #[serde(rename = "plan-of-action-and-milestones")]
    PlanOfActionAndMilestones,
    #[serde(rename = "component-definition")]
    ComponentDefinition,
    #[serde(rename = "system-security-plan")]
    SystemSecurityPlan,
    #[serde(rename = "assessment-plan")]
    AssessmentPlan,
    #[serde(rename = "assessment-results")]
    AssessmentResults,
}

/// OSCAL metadata structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OscalMetadata {
    pub title: String,
    pub published: Option<String>,
    pub last_modified: String,
    pub version: String,
    pub oscal_version: String,
    pub props: Option<Vec<OscalProperty>>,
    pub responsible_parties: Option<HashMap<String, OscalResponsibleParty>>,
}

/// OSCAL property
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OscalProperty {
    pub name: String,
    pub value: String,
    pub class: Option<String>,
}

/// OSCAL responsible party
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OscalResponsibleParty {
    pub party_uuids: Vec<String>,
}

/// OSCAL POA&M Item
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OscalPoamItem {
    pub uuid: String,
    pub title: String,
    pub description: String,
    pub props: Option<Vec<OscalProperty>>,
    pub related_observations: Option<Vec<OscalRelatedObservation>>,
    pub related_risks: Option<Vec<OscalRelatedRisk>>,
    pub remediation_tracking: Option<OscalRemediationTracking>,
}

/// OSCAL Related Observation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OscalRelatedObservation {
    pub observation_uuid: String,
}

/// OSCAL Related Risk
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OscalRelatedRisk {
    pub risk_uuid: String,
}

/// OSCAL Remediation Tracking
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OscalRemediationTracking {
    pub tracking_entries: Vec<OscalTrackingEntry>,
}

/// OSCAL Tracking Entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OscalTrackingEntry {
    pub uuid: String,
    pub title: Option<String>,
    pub description: Option<String>,
    pub props: Option<Vec<OscalProperty>>,
    pub status_change: Option<String>,
    pub date_time_stamp: String,
}

/// OSCAL Observation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OscalObservation {
    pub uuid: String,
    pub title: Option<String>,
    pub description: String,
    pub props: Option<Vec<OscalProperty>>,
    pub methods: Vec<String>,
    pub types: Option<Vec<String>>,
    pub origins: Vec<OscalOrigin>,
    pub subjects: Option<Vec<OscalSubject>>,
    pub relevant_evidence: Option<Vec<OscalRelevantEvidence>>,
    pub collected: String,
    pub expires: Option<String>,
    pub remarks: Option<String>,
}

/// OSCAL Origin
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OscalOrigin {
    pub actors: Vec<OscalActor>,
    pub related_tasks: Option<Vec<OscalRelatedTask>>,
}

/// OSCAL Actor
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OscalActor {
    #[serde(rename = "type")]
    pub actor_type: String,
    pub actor_uuid: String,
    pub role_id: Option<String>,
    pub props: Option<Vec<OscalProperty>>,
}

/// OSCAL Related Task
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OscalRelatedTask {
    pub task_uuid: String,
    pub props: Option<Vec<OscalProperty>>,
}

/// OSCAL Subject
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OscalSubject {
    #[serde(rename = "type")]
    pub subject_type: String,
    pub title: Option<String>,
    pub subject_uuid: Option<String>,
    pub props: Option<Vec<OscalProperty>>,
}

/// OSCAL Relevant Evidence
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OscalRelevantEvidence {
    pub href: Option<String>,
    pub description: String,
    pub props: Option<Vec<OscalProperty>>,
    pub remarks: Option<String>,
}

/// OSCAL Risk
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OscalRisk {
    pub uuid: String,
    pub title: String,
    pub description: String,
    pub statement: String,
    pub props: Option<Vec<OscalProperty>>,
    pub status: String,
    pub origins: Option<Vec<OscalOrigin>>,
    pub threat_ids: Option<Vec<OscalThreatId>>,
    pub characterizations: Option<Vec<OscalCharacterization>>,
    pub mitigating_factors: Option<Vec<OscalMitigatingFactor>>,
    pub deadline: Option<String>,
    pub remediations: Option<Vec<OscalRemediation>>,
    pub risk_log: Option<OscalRiskLog>,
    pub related_observations: Option<Vec<OscalRelatedObservation>>,
}

/// OSCAL Threat ID
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OscalThreatId {
    pub system: String,
    pub href: Option<String>,
    pub id: String,
}

/// OSCAL Characterization
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OscalCharacterization {
    pub props: Vec<OscalProperty>,
    pub links: Option<Vec<OscalLink>>,
    pub origin: OscalOrigin,
    pub facets: Vec<OscalFacet>,
}

/// OSCAL Facet
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OscalFacet {
    pub name: String,
    pub system: String,
    pub value: String,
    pub props: Option<Vec<OscalProperty>>,
}

/// OSCAL Mitigating Factor
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OscalMitigatingFactor {
    pub uuid: String,
    pub implementation_uuid: Option<String>,
    pub description: String,
    pub props: Option<Vec<OscalProperty>>,
    pub subjects: Option<Vec<OscalSubject>>,
}

/// OSCAL Remediation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OscalRemediation {
    pub uuid: String,
    pub lifecycle: String,
    pub title: String,
    pub description: String,
    pub props: Option<Vec<OscalProperty>>,
    pub origins: Option<Vec<OscalOrigin>>,
    pub required_assets: Option<Vec<OscalRequiredAsset>>,
    pub tasks: Option<Vec<OscalTask>>,
    pub remarks: Option<String>,
}

/// OSCAL Required Asset
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OscalRequiredAsset {
    pub uuid: String,
    pub subjects: Vec<OscalSubject>,
    pub title: Option<String>,
    pub description: Option<String>,
    pub props: Option<Vec<OscalProperty>>,
    pub remarks: Option<String>,
}

/// OSCAL Task
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OscalTask {
    pub uuid: String,
    #[serde(rename = "type")]
    pub task_type: String,
    pub title: String,
    pub description: Option<String>,
    pub props: Option<Vec<OscalProperty>>,
    pub timing: Option<OscalTiming>,
    pub dependencies: Option<Vec<OscalDependency>>,
    pub tasks: Option<Vec<OscalTask>>,
    pub associated_activities: Option<Vec<serde_json::Value>>,
    pub subjects: Option<Vec<OscalSubject>>,
    pub responsible_roles: Option<Vec<serde_json::Value>>,
    pub remarks: Option<String>,
}

/// OSCAL Timing
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OscalTiming {
    pub on_date: Option<OscalOnDate>,
    pub within_date_range: Option<OscalWithinDateRange>,
    pub at_frequency: Option<OscalAtFrequency>,
}

/// OSCAL On Date
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OscalOnDate {
    pub date: String,
}

/// OSCAL Within Date Range
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OscalWithinDateRange {
    pub start: String,
    pub end: String,
}

/// OSCAL At Frequency
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OscalAtFrequency {
    pub period: String,
    pub unit: String,
}

/// OSCAL Dependency
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OscalDependency {
    pub task_uuid: String,
}

/// OSCAL Risk Log
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OscalRiskLog {
    pub entries: Vec<OscalRiskLogEntry>,
}

/// OSCAL Risk Log Entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OscalRiskLogEntry {
    pub uuid: String,
    pub title: Option<String>,
    pub description: Option<String>,
    pub props: Option<Vec<OscalProperty>>,
    pub logged: String,
    pub logged_by: Option<OscalLoggedBy>,
    pub related_responses: Option<Vec<serde_json::Value>>,
    pub remarks: Option<String>,
}

/// OSCAL Logged By
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OscalLoggedBy {
    pub party_uuid: String,
    pub role_id: Option<String>,
}

/// OSCAL Link
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OscalLink {
    pub href: String,
    pub rel: Option<String>,
    pub media_type: Option<String>,
    pub resource_fragment: Option<String>,
    pub text: Option<String>,
}

/// OSCAL Finding
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OscalFinding {
    pub uuid: String,
    pub title: String,
    pub description: String,
    pub props: Option<Vec<OscalProperty>>,
    pub links: Option<Vec<OscalLink>>,
    pub origins: Vec<OscalOrigin>,
    pub target: OscalTarget,
    pub implementation_statement_uuid: Option<String>,
    pub related_observations: Option<Vec<OscalRelatedObservation>>,
    pub related_risks: Option<Vec<OscalRelatedRisk>>,
}

/// OSCAL Target
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OscalTarget {
    #[serde(rename = "type")]
    pub target_type: String,
    pub target_id: String,
    pub title: Option<String>,
    pub description: Option<String>,
    pub props: Option<Vec<OscalProperty>>,
    pub links: Option<Vec<OscalLink>>,
    pub status: Option<OscalImplementationStatus>,
    pub implementation_status: Option<OscalImplementationStatus>,
}

/// OSCAL Implementation Status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OscalImplementationStatus {
    pub state: String,
    pub reason: Option<String>,
}
