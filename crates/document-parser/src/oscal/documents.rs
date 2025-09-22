// Modified: 2025-09-22

//! OSCAL document structures and containers
//!
//! This module contains the top-level OSCAL document structures that wrap
//! the core OSCAL content types.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use super::types::*;

/// OSCAL POA&M Document
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OscalPoamDocument {
    #[serde(rename = "plan-of-action-and-milestones")]
    pub plan_of_action_and_milestones: PlanOfActionAndMilestones,
}

/// Plan of Action and Milestones
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlanOfActionAndMilestones {
    pub uuid: String,
    pub metadata: OscalMetadata,
    #[serde(rename = "import-ssp", skip_serializing_if = "Option::is_none")]
    pub import_ssp: Option<ImportSsp>,
    #[serde(rename = "system-id", skip_serializing_if = "Option::is_none")]
    pub system_id: Option<String>,
    #[serde(rename = "local-definitions", skip_serializing_if = "Option::is_none")]
    pub local_definitions: Option<LocalDefinitions>,
    #[serde(rename = "observations", skip_serializing_if = "Option::is_none")]
    pub observations: Option<Vec<OscalObservation>>,
    #[serde(rename = "risks", skip_serializing_if = "Option::is_none")]
    pub risks: Option<Vec<OscalRisk>>,
    #[serde(rename = "findings", skip_serializing_if = "Option::is_none")]
    pub findings: Option<Vec<OscalFinding>>,
    #[serde(rename = "poam-items")]
    pub poam_items: Vec<OscalPoamItem>,
    #[serde(rename = "back-matter", skip_serializing_if = "Option::is_none")]
    pub back_matter: Option<serde_json::Value>,
}

/// Import SSP
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImportSsp {
    pub href: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub remarks: Option<String>,
}

/// Local Definitions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LocalDefinitions {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub components: Option<Vec<serde_json::Value>>,
    #[serde(rename = "inventory-items", skip_serializing_if = "Option::is_none")]
    pub inventory_items: Option<Vec<serde_json::Value>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub users: Option<Vec<serde_json::Value>>,
    #[serde(rename = "assessment-assets", skip_serializing_if = "Option::is_none")]
    pub assessment_assets: Option<Vec<serde_json::Value>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tasks: Option<Vec<OscalTask>>,
}

/// OSCAL Component Definition Document
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OscalComponentDefinitionDocument {
    #[serde(rename = "component-definition")]
    pub component_definition: ComponentDefinition,
}

/// Component Definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComponentDefinition {
    pub uuid: String,
    pub metadata: OscalMetadata,
    #[serde(rename = "import-component-definitions", skip_serializing_if = "Option::is_none")]
    pub import_component_definitions: Option<Vec<ImportComponentDefinition>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub components: Option<Vec<Component>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub capabilities: Option<Vec<Capability>>,
    #[serde(rename = "back-matter", skip_serializing_if = "Option::is_none")]
    pub back_matter: Option<serde_json::Value>,
}

/// Import Component Definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImportComponentDefinition {
    pub href: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub include_all: Option<IncludeAll>,
}

/// Include All
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IncludeAll {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub include_controls: Option<Vec<IncludeControl>>,
}

/// Include Control
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IncludeControl {
    #[serde(rename = "with-child-controls", skip_serializing_if = "Option::is_none")]
    pub with_child_controls: Option<String>,
    #[serde(rename = "with-ids", skip_serializing_if = "Option::is_none")]
    pub with_ids: Option<Vec<String>>,
}

/// Component
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Component {
    pub uuid: String,
    #[serde(rename = "type")]
    pub component_type: String,
    pub title: String,
    pub description: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub purpose: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub props: Option<Vec<OscalProperty>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub links: Option<Vec<OscalLink>>,
    #[serde(rename = "responsible-roles", skip_serializing_if = "Option::is_none")]
    pub responsible_roles: Option<Vec<ResponsibleRole>>,
    #[serde(rename = "control-implementations", skip_serializing_if = "Option::is_none")]
    pub control_implementations: Option<Vec<ControlImplementation>>,
}

/// Responsible Role
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResponsibleRole {
    #[serde(rename = "role-id")]
    pub role_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub props: Option<Vec<OscalProperty>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub links: Option<Vec<OscalLink>>,
    #[serde(rename = "party-uuids", skip_serializing_if = "Option::is_none")]
    pub party_uuids: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub remarks: Option<String>,
}

/// Control Implementation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ControlImplementation {
    pub uuid: String,
    pub source: String,
    pub description: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub props: Option<Vec<OscalProperty>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub links: Option<Vec<OscalLink>>,
    #[serde(rename = "set-parameters", skip_serializing_if = "Option::is_none")]
    pub set_parameters: Option<Vec<SetParameter>>,
    #[serde(rename = "implemented-requirements")]
    pub implemented_requirements: Vec<ImplementedRequirement>,
}

/// Set Parameter
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SetParameter {
    #[serde(rename = "param-id")]
    pub param_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub values: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub remarks: Option<String>,
}

/// Implemented Requirement
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImplementedRequirement {
    pub uuid: String,
    #[serde(rename = "control-id")]
    pub control_id: String,
    pub description: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub props: Option<Vec<OscalProperty>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub links: Option<Vec<OscalLink>>,
    #[serde(rename = "set-parameters", skip_serializing_if = "Option::is_none")]
    pub set_parameters: Option<Vec<SetParameter>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub statements: Option<Vec<Statement>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub remarks: Option<String>,
}

/// Statement
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Statement {
    #[serde(rename = "statement-id")]
    pub statement_id: String,
    pub uuid: String,
    pub description: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub props: Option<Vec<OscalProperty>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub links: Option<Vec<OscalLink>>,
    #[serde(rename = "responsible-roles", skip_serializing_if = "Option::is_none")]
    pub responsible_roles: Option<Vec<ResponsibleRole>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub remarks: Option<String>,
}

/// Capability
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Capability {
    pub uuid: String,
    pub name: String,
    pub description: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub props: Option<Vec<OscalProperty>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub links: Option<Vec<OscalLink>>,
    #[serde(rename = "incorporates-components", skip_serializing_if = "Option::is_none")]
    pub incorporates_components: Option<Vec<IncorporatesComponent>>,
    #[serde(rename = "control-implementations", skip_serializing_if = "Option::is_none")]
    pub control_implementations: Option<Vec<ControlImplementation>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub remarks: Option<String>,
}

/// Incorporates Component
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IncorporatesComponent {
    #[serde(rename = "component-uuid")]
    pub component_uuid: String,
    pub description: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub props: Option<Vec<OscalProperty>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub links: Option<Vec<OscalLink>>,
}

/// OSCAL System Security Plan Document
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OscalSspDocument {
    #[serde(rename = "system-security-plan")]
    pub system_security_plan: SystemSecurityPlan,
}

/// System Security Plan
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemSecurityPlan {
    pub uuid: String,
    pub metadata: OscalMetadata,
    #[serde(rename = "import-profile")]
    pub import_profile: ImportProfile,
    #[serde(rename = "system-characteristics")]
    pub system_characteristics: SystemCharacteristics,
    #[serde(rename = "system-implementation")]
    pub system_implementation: SystemImplementation,
    #[serde(rename = "control-implementation")]
    pub control_implementation: ControlImplementation,
    #[serde(rename = "back-matter", skip_serializing_if = "Option::is_none")]
    pub back_matter: Option<serde_json::Value>,
}

/// Import Profile
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImportProfile {
    pub href: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub remarks: Option<String>,
}

/// System Characteristics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemCharacteristics {
    #[serde(rename = "system-ids")]
    pub system_ids: Vec<SystemId>,
    #[serde(rename = "system-name")]
    pub system_name: String,
    #[serde(rename = "system-name-short", skip_serializing_if = "Option::is_none")]
    pub system_name_short: Option<String>,
    pub description: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub props: Option<Vec<OscalProperty>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub links: Option<Vec<OscalLink>>,
    #[serde(rename = "date-authorized", skip_serializing_if = "Option::is_none")]
    pub date_authorized: Option<String>,
    #[serde(rename = "security-sensitivity-level")]
    pub security_sensitivity_level: String,
    #[serde(rename = "system-information")]
    pub system_information: SystemInformation,
    #[serde(rename = "security-impact-level")]
    pub security_impact_level: SecurityImpactLevel,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<SystemStatus>,
    #[serde(rename = "authorization-boundary")]
    pub authorization_boundary: AuthorizationBoundary,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub remarks: Option<String>,
}

/// System ID
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemId {
    #[serde(rename = "identifier-type", skip_serializing_if = "Option::is_none")]
    pub identifier_type: Option<String>,
    pub id: String,
}

/// System Information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemInformation {
    #[serde(rename = "information-types")]
    pub information_types: Vec<InformationType>,
}

/// Information Type
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InformationType {
    pub uuid: String,
    pub title: String,
    pub description: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub categorizations: Option<Vec<Categorization>>,
    #[serde(rename = "confidentiality-impact")]
    pub confidentiality_impact: ImpactLevel,
    #[serde(rename = "integrity-impact")]
    pub integrity_impact: ImpactLevel,
    #[serde(rename = "availability-impact")]
    pub availability_impact: ImpactLevel,
}

/// Categorization
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Categorization {
    pub system: String,
    #[serde(rename = "information-type-ids", skip_serializing_if = "Option::is_none")]
    pub information_type_ids: Option<Vec<String>>,
}

/// Impact Level
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImpactLevel {
    pub base: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub selected: Option<String>,
    #[serde(rename = "adjustment-justification", skip_serializing_if = "Option::is_none")]
    pub adjustment_justification: Option<String>,
}

/// Security Impact Level
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityImpactLevel {
    #[serde(rename = "security-objective-confidentiality")]
    pub security_objective_confidentiality: String,
    #[serde(rename = "security-objective-integrity")]
    pub security_objective_integrity: String,
    #[serde(rename = "security-objective-availability")]
    pub security_objective_availability: String,
}

/// System Status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemStatus {
    pub state: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reason: Option<String>,
}

/// Authorization Boundary
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthorizationBoundary {
    pub description: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub props: Option<Vec<OscalProperty>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub links: Option<Vec<OscalLink>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub diagrams: Option<Vec<Diagram>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub remarks: Option<String>,
}

/// Diagram
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Diagram {
    pub uuid: String,
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub props: Option<Vec<OscalProperty>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub links: Option<Vec<OscalLink>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub caption: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub remarks: Option<String>,
}

/// System Implementation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemImplementation {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub props: Option<Vec<OscalProperty>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub links: Option<Vec<OscalLink>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub leveraged_authorizations: Option<Vec<LeveragedAuthorization>>,
    pub users: Vec<SystemUser>,
    pub components: Vec<SystemComponent>,
    #[serde(rename = "inventory-items", skip_serializing_if = "Option::is_none")]
    pub inventory_items: Option<Vec<InventoryItem>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub remarks: Option<String>,
}

/// Leveraged Authorization
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LeveragedAuthorization {
    pub uuid: String,
    pub title: String,
    #[serde(rename = "party-uuid")]
    pub party_uuid: String,
    #[serde(rename = "date-authorized")]
    pub date_authorized: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub props: Option<Vec<OscalProperty>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub links: Option<Vec<OscalLink>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub remarks: Option<String>,
}

/// System User
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemUser {
    pub uuid: String,
    pub title: Option<String>,
    #[serde(rename = "short-name", skip_serializing_if = "Option::is_none")]
    pub short_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub props: Option<Vec<OscalProperty>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub links: Option<Vec<OscalLink>>,
    #[serde(rename = "role-ids", skip_serializing_if = "Option::is_none")]
    pub role_ids: Option<Vec<String>>,
    #[serde(rename = "authorized-privileges", skip_serializing_if = "Option::is_none")]
    pub authorized_privileges: Option<Vec<AuthorizedPrivilege>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub remarks: Option<String>,
}

/// Authorized Privilege
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthorizedPrivilege {
    pub title: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(rename = "functions-performed")]
    pub functions_performed: Vec<String>,
}

/// System Component
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemComponent {
    pub uuid: String,
    #[serde(rename = "type")]
    pub component_type: String,
    pub title: String,
    pub description: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub purpose: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub props: Option<Vec<OscalProperty>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub links: Option<Vec<OscalLink>>,
    pub status: ComponentStatus,
    #[serde(rename = "responsible-roles", skip_serializing_if = "Option::is_none")]
    pub responsible_roles: Option<Vec<ResponsibleRole>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub protocols: Option<Vec<Protocol>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub remarks: Option<String>,
}

/// Component Status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComponentStatus {
    pub state: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reason: Option<String>,
}

/// Protocol
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Protocol {
    pub uuid: Option<String>,
    pub name: String,
    pub title: Option<String>,
    #[serde(rename = "port-ranges", skip_serializing_if = "Option::is_none")]
    pub port_ranges: Option<Vec<PortRange>>,
}

/// Port Range
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PortRange {
    pub start: u16,
    pub end: Option<u16>,
    pub transport: Option<String>,
}

/// Inventory Item
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InventoryItem {
    pub uuid: String,
    pub description: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub props: Option<Vec<OscalProperty>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub links: Option<Vec<OscalLink>>,
    #[serde(rename = "responsible-parties", skip_serializing_if = "Option::is_none")]
    pub responsible_parties: Option<HashMap<String, OscalResponsibleParty>>,
    #[serde(rename = "implemented-components", skip_serializing_if = "Option::is_none")]
    pub implemented_components: Option<Vec<ImplementedComponent>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub remarks: Option<String>,
}

/// Implemented Component
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImplementedComponent {
    #[serde(rename = "component-uuid")]
    pub component_uuid: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub props: Option<Vec<OscalProperty>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub links: Option<Vec<OscalLink>>,
    #[serde(rename = "responsible-parties", skip_serializing_if = "Option::is_none")]
    pub responsible_parties: Option<HashMap<String, OscalResponsibleParty>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub remarks: Option<String>,
}
