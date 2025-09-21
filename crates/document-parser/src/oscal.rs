// Modified: 2025-09-20

//! OSCAL output generation and validation
//!
//! This module provides comprehensive functionality to generate valid OSCAL JSON documents
//! from parsed document content with proper validation, metadata, and schema compliance.

use fedramp_core::{Result, Error};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tracing::{debug, info, warn};
use uuid::Uuid;
use chrono::{DateTime, Utc};

/// OSCAL document types
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
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
    pub origins: Option<Vec<OscalOrigin>>,
    pub subjects: Option<Vec<OscalSubject>>,
    pub relevant_evidence: Option<Vec<OscalRelevantEvidence>>,
    pub collected: String,
    pub expires: Option<String>,
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
}

/// OSCAL Required Asset
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OscalRequiredAsset {
    pub uuid: String,
    pub subjects: Vec<OscalSubject>,
    pub title: Option<String>,
    pub description: String,
    pub props: Option<Vec<OscalProperty>>,
}

/// OSCAL Task
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OscalTask {
    pub uuid: String,
    #[serde(rename = "type")]
    pub task_type: String,
    pub title: String,
    pub description: String,
    pub props: Option<Vec<OscalProperty>>,
    pub timing: Option<OscalTiming>,
    pub dependencies: Option<Vec<OscalDependency>>,
    pub subjects: Option<Vec<OscalSubject>>,
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
    pub start: String,
    pub end: Option<String>,
    pub props: Option<Vec<OscalProperty>>,
    pub logged_by: Option<Vec<OscalLoggedBy>>,
    pub status_change: Option<String>,
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
    pub origins: Option<Vec<OscalOrigin>>,
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
    pub status: Option<OscalImplementationStatus>,
}

/// OSCAL Implementation Status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OscalImplementationStatus {
    pub state: String,
    pub reason: Option<String>,
}

/// OSCAL generator for creating valid OSCAL documents
#[derive(Debug, Clone)]
pub struct OscalGenerator {
    /// OSCAL version to generate
    pub oscal_version: String,
    /// Whether to validate output against schemas
    pub validate_output: bool,
    /// POA&M item processor
    poam_processor: PoamItemProcessor,
    /// Risk processor
    risk_processor: RiskProcessor,
    /// Observation processor
    observation_processor: ObservationProcessor,
}

/// POA&M item processor for converting parsed data to OSCAL POA&M items
#[derive(Debug, Clone)]
pub struct PoamItemProcessor {
    /// Configuration for POA&M processing
    config: PoamProcessorConfig,
}

/// Risk processor for converting parsed data to OSCAL risks
#[derive(Debug, Clone)]
pub struct RiskProcessor {
    /// Configuration for risk processing
    config: RiskProcessorConfig,
}

/// Observation processor for converting parsed data to OSCAL observations
#[derive(Debug, Clone)]
pub struct ObservationProcessor {
    /// Configuration for observation processing
    config: ObservationProcessorConfig,
}

/// Configuration for POA&M processing
#[derive(Debug, Clone)]
pub struct PoamProcessorConfig {
    /// Default status for new POA&M items
    pub default_status: String,
    /// Default lifecycle for remediation
    pub default_lifecycle: String,
    /// Generate UUIDs for items without IDs
    pub generate_uuids: bool,
    /// Include tracking entries
    pub include_tracking: bool,
}

/// Configuration for risk processing
#[derive(Debug, Clone)]
pub struct RiskProcessorConfig {
    /// Default risk status
    pub default_status: String,
    /// Include characterizations
    pub include_characterizations: bool,
    /// Generate threat IDs
    pub generate_threat_ids: bool,
}

/// Configuration for observation processing
#[derive(Debug, Clone)]
pub struct ObservationProcessorConfig {
    /// Default observation methods
    pub default_methods: Vec<String>,
    /// Include evidence
    pub include_evidence: bool,
    /// Default expiration period in days
    pub default_expiration_days: Option<u32>,
}

impl OscalGenerator {
    /// Create a new OSCAL generator
    #[must_use]
    pub fn new() -> Self {
        Self {
            oscal_version: "1.1.2".to_string(),
            validate_output: true,
            poam_processor: PoamItemProcessor::new(),
            risk_processor: RiskProcessor::new(),
            observation_processor: ObservationProcessor::new(),
        }
    }

    /// Create a new OSCAL generator with custom configuration
    #[must_use]
    pub fn with_config(oscal_version: String, validate_output: bool) -> Self {
        Self {
            oscal_version,
            validate_output,
            poam_processor: PoamItemProcessor::new(),
            risk_processor: RiskProcessor::new(),
            observation_processor: ObservationProcessor::new(),
        }
    }

    /// Create a new OSCAL generator with custom processors
    #[must_use]
    pub fn with_processors(
        oscal_version: String,
        validate_output: bool,
        poam_processor: PoamItemProcessor,
        risk_processor: RiskProcessor,
        observation_processor: ObservationProcessor,
    ) -> Self {
        Self {
            oscal_version,
            validate_output,
            poam_processor,
            risk_processor,
            observation_processor,
        }
    }

    /// Generate OSCAL POA&M document from parsed content
    pub fn generate_poam(&self, content: &serde_json::Value, metadata: &serde_json::Value) -> Result<serde_json::Value> {
        info!("Generating OSCAL POA&M document");

        let document_uuid = Uuid::new_v4().to_string();
        let current_time = chrono::Utc::now().to_rfc3339();

        let oscal_metadata = self.create_metadata(
            "Plan of Action and Milestones",
            &current_time,
            metadata,
        )?;

        // Extract POA&M items from parsed content
        let poam_items = self.extract_poam_items(content)?;

        // Extract observations from parsed content
        let observations = self.extract_observations(content)?;

        // Extract risks from parsed content
        let risks = self.extract_risks(content)?;

        // Extract findings from parsed content
        let findings = self.extract_findings(content)?;

        // Create system ID from metadata or generate new one
        let system_id = self.extract_system_id(metadata)?;

        // Create import SSP reference
        let import_ssp = self.create_import_ssp_reference(metadata)?;

        // Create local definitions
        let local_definitions = self.create_local_definitions(content)?;

        let poam_document = serde_json::json!({
            "plan-of-action-and-milestones": {
                "uuid": document_uuid,
                "metadata": oscal_metadata,
                "import-ssp": import_ssp,
                "system-id": system_id,
                "local-definitions": local_definitions,
                "observations": observations,
                "risks": risks,
                "findings": findings,
                "poam-items": poam_items
            }
        });

        if self.validate_output {
            self.validate_oscal_document(&poam_document, &OscalDocumentType::PlanOfActionAndMilestones)?;
        }

        info!("Generated OSCAL POA&M with {} items, {} observations, {} risks, {} findings",
              poam_items.len(), observations.len(), risks.len(), findings.len());

        Ok(poam_document)
    }

    /// Extract POA&M items from parsed content
    fn extract_poam_items(&self, content: &serde_json::Value) -> Result<Vec<OscalPoamItem>> {
        let mut poam_items = Vec::new();

        // Handle different content structures
        if let Some(items_array) = content.get("poam_items").and_then(|v| v.as_array()) {
            // Direct POA&M items array
            for item in items_array {
                if let Ok(poam_item) = self.poam_processor.process_item(item) {
                    poam_items.push(poam_item);
                }
            }
        } else if let Some(rows) = content.get("rows").and_then(|v| v.as_array()) {
            // Spreadsheet-style data
            for (index, row) in rows.iter().enumerate() {
                if let Ok(poam_item) = self.poam_processor.process_row(row, index) {
                    poam_items.push(poam_item);
                }
            }
        } else if let Some(tables) = content.get("tables").and_then(|v| v.as_array()) {
            // Markdown tables or document tables
            for table in tables {
                if let Ok(items) = self.poam_processor.process_table(table) {
                    poam_items.extend(items);
                }
            }
        } else if let Some(sections) = content.get("sections").and_then(|v| v.as_array()) {
            // Document sections (Word/PDF)
            for section in sections {
                if let Ok(items) = self.poam_processor.process_section(section) {
                    poam_items.extend(items);
                }
            }
        }

        // If no items found, try to extract from general content
        if poam_items.is_empty() {
            if let Ok(items) = self.poam_processor.extract_from_general_content(content) {
                poam_items.extend(items);
            }
        }

        Ok(poam_items)
    }

    /// Extract observations from parsed content
    fn extract_observations(&self, content: &serde_json::Value) -> Result<Vec<OscalObservation>> {
        let mut observations = Vec::new();

        // Look for observations in various content structures
        if let Some(obs_array) = content.get("observations").and_then(|v| v.as_array()) {
            for obs in obs_array {
                if let Ok(observation) = self.observation_processor.process_observation(obs) {
                    observations.push(observation);
                }
            }
        }

        // Extract observations from POA&M items if they contain observation data
        if let Some(items) = content.get("poam_items").and_then(|v| v.as_array()) {
            for item in items {
                if let Ok(obs) = self.observation_processor.extract_from_poam_item(item) {
                    observations.extend(obs);
                }
            }
        }

        Ok(observations)
    }

    /// Extract risks from parsed content
    fn extract_risks(&self, content: &serde_json::Value) -> Result<Vec<OscalRisk>> {
        let mut risks = Vec::new();

        // Look for risks in various content structures
        if let Some(risks_array) = content.get("risks").and_then(|v| v.as_array()) {
            for risk in risks_array {
                if let Ok(oscal_risk) = self.risk_processor.process_risk(risk) {
                    risks.push(oscal_risk);
                }
            }
        }

        // Extract risks from POA&M items
        if let Some(items) = content.get("poam_items").and_then(|v| v.as_array()) {
            for item in items {
                if let Ok(risk) = self.risk_processor.extract_from_poam_item(item) {
                    if let Some(r) = risk {
                        risks.push(r);
                    }
                }
            }
        }

        Ok(risks)
    }

    /// Extract findings from parsed content
    fn extract_findings(&self, content: &serde_json::Value) -> Result<Vec<OscalFinding>> {
        let mut findings = Vec::new();

        // Look for findings in content
        if let Some(findings_array) = content.get("findings").and_then(|v| v.as_array()) {
            for finding in findings_array {
                if let Ok(oscal_finding) = self.process_finding(finding) {
                    findings.push(oscal_finding);
                }
            }
        }

        Ok(findings)
    }

    /// Extract system ID from metadata
    pub fn extract_system_id(&self, metadata: &serde_json::Value) -> Result<serde_json::Value> {
        let system_id = if let Some(id) = metadata.get("system_id").and_then(|v| v.as_str()) {
            id.to_string()
        } else if let Some(name) = metadata.get("system_name").and_then(|v| v.as_str()) {
            // Generate ID from system name
            name.to_lowercase().replace(' ', "-")
        } else {
            Uuid::new_v4().to_string()
        };

        Ok(serde_json::json!({
            "identifier-type": "https://ietf.org/rfc/rfc4122",
            "id": system_id
        }))
    }

    /// Create import SSP reference
    fn create_import_ssp_reference(&self, metadata: &serde_json::Value) -> Result<serde_json::Value> {
        let href = if let Some(ssp_href) = metadata.get("ssp_href").and_then(|v| v.as_str()) {
            ssp_href.to_string()
        } else {
            "#system-security-plan".to_string()
        };

        Ok(serde_json::json!({
            "href": href
        }))
    }

    /// Create local definitions
    fn create_local_definitions(&self, _content: &serde_json::Value) -> Result<serde_json::Value> {
        // TODO: Extract components, inventory items, and users from content
        Ok(serde_json::json!({
            "components": [],
            "inventory-items": [],
            "users": []
        }))
    }

    /// Process finding from parsed content
    fn process_finding(&self, finding: &serde_json::Value) -> Result<OscalFinding> {
        let uuid = finding.get("uuid")
            .and_then(|v| v.as_str())
            .unwrap_or(&Uuid::new_v4().to_string())
            .to_string();

        let title = finding.get("title")
            .and_then(|v| v.as_str())
            .unwrap_or("Finding")
            .to_string();

        let description = finding.get("description")
            .and_then(|v| v.as_str())
            .unwrap_or("No description provided")
            .to_string();

        let target = OscalTarget {
            target_type: finding.get("target_type")
                .and_then(|v| v.as_str())
                .unwrap_or("objective-id")
                .to_string(),
            target_id: finding.get("target_id")
                .and_then(|v| v.as_str())
                .unwrap_or("unknown")
                .to_string(),
            title: finding.get("target_title").and_then(|v| v.as_str()).map(|s| s.to_string()),
            description: finding.get("target_description").and_then(|v| v.as_str()).map(|s| s.to_string()),
            props: None,
            status: None,
        };

        Ok(OscalFinding {
            uuid,
            title,
            description,
            props: None,
            origins: None,
            target,
            implementation_statement_uuid: None,
            related_observations: None,
            related_risks: None,
        })
    }

    /// Generate OSCAL Component Definition from parsed content
    pub fn generate_component_definition(&self, content: &serde_json::Value, metadata: &serde_json::Value) -> Result<serde_json::Value> {
        info!("Generating OSCAL Component Definition document");
        
        let document_uuid = Uuid::new_v4().to_string();
        let current_time = chrono::Utc::now().to_rfc3339();
        
        let oscal_metadata = self.create_metadata(
            "Component Definition",
            &current_time,
            metadata,
        )?;

        // TODO: Implement actual Component Definition generation from parsed content
        let component_definition = serde_json::json!({
            "component-definition": {
                "uuid": document_uuid,
                "metadata": oscal_metadata,
                "components": [],
                "capabilities": []
            }
        });

        if self.validate_output {
            self.validate_oscal_document(&component_definition, &OscalDocumentType::ComponentDefinition)?;
        }

        Ok(component_definition)
    }

    /// Generate OSCAL System Security Plan from parsed content
    pub fn generate_ssp(&self, content: &serde_json::Value, metadata: &serde_json::Value) -> Result<serde_json::Value> {
        info!("Generating OSCAL System Security Plan document");
        
        let document_uuid = Uuid::new_v4().to_string();
        let current_time = chrono::Utc::now().to_rfc3339();
        
        let oscal_metadata = self.create_metadata(
            "System Security Plan",
            &current_time,
            metadata,
        )?;

        // TODO: Implement actual SSP generation from parsed content
        let ssp_document = serde_json::json!({
            "system-security-plan": {
                "uuid": document_uuid,
                "metadata": oscal_metadata,
                "import-profile": {
                    "href": "#profile"
                },
                "system-characteristics": {
                    "system-ids": [],
                    "system-name": "System Name",
                    "description": "System Description",
                    "security-sensitivity-level": "moderate",
                    "system-information": {
                        "information-types": []
                    },
                    "security-impact-level": {
                        "security-objective-confidentiality": "moderate",
                        "security-objective-integrity": "moderate",
                        "security-objective-availability": "moderate"
                    },
                    "status": {
                        "state": "operational"
                    },
                    "authorization-boundary": {
                        "description": "Authorization boundary description"
                    }
                },
                "system-implementation": {
                    "users": [],
                    "components": [],
                    "inventory-items": []
                },
                "control-implementation": {
                    "description": "Control implementation description",
                    "implemented-requirements": []
                }
            }
        });

        if self.validate_output {
            self.validate_oscal_document(&ssp_document, &OscalDocumentType::SystemSecurityPlan)?;
        }

        Ok(ssp_document)
    }

    /// Create OSCAL metadata structure
    pub fn create_metadata(&self, title: &str, timestamp: &str, source_metadata: &serde_json::Value) -> Result<OscalMetadata> {
        let mut props = Vec::new();
        
        // Add source information as properties
        if let Some(source_file) = source_metadata.get("source_file").and_then(|v| v.as_str()) {
            props.push(OscalProperty {
                name: "source-file".to_string(),
                value: source_file.to_string(),
                class: Some("source".to_string()),
            });
        }

        if let Some(parser_version) = source_metadata.get("parser_version").and_then(|v| v.as_str()) {
            props.push(OscalProperty {
                name: "parser-version".to_string(),
                value: parser_version.to_string(),
                class: Some("tool".to_string()),
            });
        }

        Ok(OscalMetadata {
            title: title.to_string(),
            published: Some(timestamp.to_string()),
            last_modified: timestamp.to_string(),
            version: "1.0.0".to_string(),
            oscal_version: self.oscal_version.clone(),
            props: if props.is_empty() { None } else { Some(props) },
            responsible_parties: None,
        })
    }

    /// Validate OSCAL document structure
    pub fn validate_oscal_document(&self, document: &serde_json::Value, doc_type: &OscalDocumentType) -> Result<()> {
        debug!("Validating OSCAL document of type: {:?}", doc_type);
        
        // Basic structural validation
        let root_key = match doc_type {
            OscalDocumentType::PlanOfActionAndMilestones => "plan-of-action-and-milestones",
            OscalDocumentType::ComponentDefinition => "component-definition",
            OscalDocumentType::SystemSecurityPlan => "system-security-plan",
            OscalDocumentType::AssessmentPlan => "assessment-plan",
            OscalDocumentType::AssessmentResults => "assessment-results",
        };

        let root_object = document.get(root_key)
            .ok_or_else(|| Error::document_parsing(format!("Missing root object: {}", root_key)))?;

        // Validate required fields
        if !root_object.get("uuid").is_some() {
            return Err(Error::document_parsing("Missing required field: uuid".to_string()));
        }

        if !root_object.get("metadata").is_some() {
            return Err(Error::document_parsing("Missing required field: metadata".to_string()));
        }

        // TODO: Implement comprehensive OSCAL schema validation
        info!("OSCAL document validation passed");
        Ok(())
    }

    /// Generate UUID for OSCAL objects
    #[must_use]
    pub fn generate_uuid() -> String {
        Uuid::new_v4().to_string()
    }

    /// Format OSCAL JSON with proper indentation
    pub fn format_json(&self, document: &serde_json::Value) -> Result<String> {
        serde_json::to_string_pretty(document)
            .map_err(|e| Error::document_parsing(format!("Failed to format JSON: {}", e)))
    }

    /// Validate OSCAL version compatibility
    pub fn validate_version_compatibility(&self, target_version: &str) -> Result<bool> {
        // Simple version comparison - can be enhanced with proper semver parsing
        Ok(self.oscal_version == target_version)
    }
}

impl Default for OscalGenerator {
    fn default() -> Self {
        Self::new()
    }
}

impl PoamItemProcessor {
    /// Create a new POA&M item processor
    pub fn new() -> Self {
        Self {
            config: PoamProcessorConfig::default(),
        }
    }

    /// Create a new POA&M item processor with custom configuration
    pub fn with_config(config: PoamProcessorConfig) -> Self {
        Self { config }
    }

    /// Process a single POA&M item from JSON
    pub fn process_item(&self, item: &serde_json::Value) -> Result<OscalPoamItem> {
        let uuid = if self.config.generate_uuids {
            item.get("uuid")
                .and_then(|v| v.as_str())
                .unwrap_or(&Uuid::new_v4().to_string())
                .to_string()
        } else {
            item.get("uuid")
                .and_then(|v| v.as_str())
                .ok_or_else(|| Error::document_parsing("Missing required field: uuid".to_string()))?
                .to_string()
        };

        let title = item.get("title")
            .or_else(|| item.get("vulnerability_description"))
            .or_else(|| item.get("description"))
            .and_then(|v| v.as_str())
            .unwrap_or("Untitled POA&M Item")
            .to_string();

        let description = item.get("description")
            .or_else(|| item.get("vulnerability_description"))
            .or_else(|| item.get("details"))
            .and_then(|v| v.as_str())
            .unwrap_or("No description provided")
            .to_string();

        // Create properties from various fields
        let mut props = Vec::new();

        if let Some(severity) = item.get("severity").and_then(|v| v.as_str()) {
            props.push(OscalProperty {
                name: "severity".to_string(),
                value: severity.to_string(),
                class: Some("impact".to_string()),
            });
        }

        if let Some(status) = item.get("status").and_then(|v| v.as_str()) {
            props.push(OscalProperty {
                name: "status".to_string(),
                value: status.to_string(),
                class: Some("state".to_string()),
            });
        }

        if let Some(control_id) = item.get("control_id").and_then(|v| v.as_str()) {
            props.push(OscalProperty {
                name: "control-id".to_string(),
                value: control_id.to_string(),
                class: Some("control".to_string()),
            });
        }

        if let Some(poc) = item.get("point_of_contact").and_then(|v| v.as_str()) {
            props.push(OscalProperty {
                name: "point-of-contact".to_string(),
                value: poc.to_string(),
                class: Some("contact".to_string()),
            });
        }

        // Create remediation tracking if enabled
        let remediation_tracking = if self.config.include_tracking {
            Some(self.create_remediation_tracking(item)?)
        } else {
            None
        };

        Ok(OscalPoamItem {
            uuid,
            title,
            description,
            props: if props.is_empty() { None } else { Some(props) },
            related_observations: None, // TODO: Link to observations
            related_risks: None, // TODO: Link to risks
            remediation_tracking,
        })
    }

    /// Process a row from spreadsheet data
    pub fn process_row(&self, row: &serde_json::Value, index: usize) -> Result<OscalPoamItem> {
        // Convert row data to item format
        let mut item_data = serde_json::Map::new();

        if let Some(row_obj) = row.as_object() {
            // Copy all row data
            for (key, value) in row_obj {
                item_data.insert(key.clone(), value.clone());
            }
        } else if let Some(row_array) = row.as_array() {
            // Handle array format (CSV-like)
            let headers = vec!["poam_id", "vulnerability_description", "severity", "status", "scheduled_completion_date", "point_of_contact"];
            for (i, value) in row_array.iter().enumerate() {
                if i < headers.len() {
                    item_data.insert(headers[i].to_string(), value.clone());
                }
            }
        }

        // Generate UUID if not present
        if !item_data.contains_key("uuid") {
            item_data.insert("uuid".to_string(), serde_json::Value::String(format!("poam-item-{}", index + 1)));
        }

        self.process_item(&serde_json::Value::Object(item_data))
    }

    /// Process a table to extract POA&M items
    pub fn process_table(&self, table: &serde_json::Value) -> Result<Vec<OscalPoamItem>> {
        let mut items = Vec::new();

        if let Some(rows) = table.get("rows").and_then(|v| v.as_array()) {
            for (index, row) in rows.iter().enumerate() {
                if let Ok(item) = self.process_table_row(table, row, index) {
                    items.push(item);
                }
            }
        }

        Ok(items)
    }

    /// Process a table row
    fn process_table_row(&self, table: &serde_json::Value, row: &serde_json::Value, index: usize) -> Result<OscalPoamItem> {
        let headers = table.get("headers")
            .and_then(|v| v.as_array())
            .map(|arr| arr.iter().filter_map(|v| v.as_str()).collect::<Vec<_>>())
            .unwrap_or_default();

        let empty_vec = vec![];
        let row_values = row.as_array().unwrap_or(&empty_vec);

        let mut item_data = serde_json::Map::new();

        for (i, header) in headers.iter().enumerate() {
            if i < row_values.len() {
                let normalized_header = header.to_lowercase().replace(' ', "_");
                item_data.insert(normalized_header, row_values[i].clone());
            }
        }

        // Generate UUID
        item_data.insert("uuid".to_string(), serde_json::Value::String(format!("table-item-{}", index + 1)));

        self.process_item(&serde_json::Value::Object(item_data))
    }

    /// Process a document section
    pub fn process_section(&self, section: &serde_json::Value) -> Result<Vec<OscalPoamItem>> {
        let mut items = Vec::new();

        // Look for POA&M-like content in section
        if let Some(content) = section.get("content").and_then(|v| v.as_str()) {
            // Try to extract structured data from text content
            if let Ok(extracted_items) = self.extract_from_text_content(content) {
                items.extend(extracted_items);
            }
        }

        Ok(items)
    }

    /// Extract POA&M items from general content
    pub fn extract_from_general_content(&self, content: &serde_json::Value) -> Result<Vec<OscalPoamItem>> {
        let mut items = Vec::new();

        // Try to find any structured data that could be POA&M items
        if let Some(content_obj) = content.as_object() {
            for (key, value) in content_obj {
                if key.contains("poam") || key.contains("item") || key.contains("vulnerability") {
                    if let Some(array) = value.as_array() {
                        for (index, item) in array.iter().enumerate() {
                            if let Ok(poam_item) = self.process_item(item) {
                                items.push(poam_item);
                            } else if let Ok(converted_item) = self.convert_to_poam_item(item, index) {
                                items.push(converted_item);
                            }
                        }
                    }
                }
            }
        }

        Ok(items)
    }

    /// Extract from text content using pattern matching
    fn extract_from_text_content(&self, _content: &str) -> Result<Vec<OscalPoamItem>> {
        // TODO: Implement text parsing for POA&M items
        Ok(Vec::new())
    }

    /// Convert generic item to POA&M item
    fn convert_to_poam_item(&self, item: &serde_json::Value, index: usize) -> Result<OscalPoamItem> {
        let mut converted = serde_json::Map::new();

        if let Some(obj) = item.as_object() {
            converted.extend(obj.clone());
        }

        // Ensure required fields
        if !converted.contains_key("uuid") {
            converted.insert("uuid".to_string(), serde_json::Value::String(format!("converted-item-{}", index + 1)));
        }
        if !converted.contains_key("title") {
            converted.insert("title".to_string(), serde_json::Value::String("Converted Item".to_string()));
        }
        if !converted.contains_key("description") {
            converted.insert("description".to_string(), serde_json::Value::String("Converted from source data".to_string()));
        }

        self.process_item(&serde_json::Value::Object(converted))
    }

    /// Create remediation tracking from item data
    fn create_remediation_tracking(&self, item: &serde_json::Value) -> Result<OscalRemediationTracking> {
        let mut tracking_entries = Vec::new();

        // Create initial tracking entry
        let current_time = Utc::now().to_rfc3339();
        let status = item.get("status").and_then(|v| v.as_str()).unwrap_or(&self.config.default_status);

        tracking_entries.push(OscalTrackingEntry {
            uuid: Uuid::new_v4().to_string(),
            title: Some("Initial Status".to_string()),
            description: Some(format!("POA&M item created with status: {}", status)),
            props: None,
            status_change: Some(status.to_string()),
            date_time_stamp: current_time,
        });

        // Add additional tracking entries if available
        if let Some(scheduled_date) = item.get("scheduled_completion_date").and_then(|v| v.as_str()) {
            tracking_entries.push(OscalTrackingEntry {
                uuid: Uuid::new_v4().to_string(),
                title: Some("Scheduled Completion".to_string()),
                description: Some(format!("Scheduled for completion on: {}", scheduled_date)),
                props: None,
                status_change: None,
                date_time_stamp: scheduled_date.to_string(),
            });
        }

        Ok(OscalRemediationTracking { tracking_entries })
    }
}

impl Default for PoamItemProcessor {
    fn default() -> Self {
        Self::new()
    }
}

impl Default for PoamProcessorConfig {
    fn default() -> Self {
        Self {
            default_status: "open".to_string(),
            default_lifecycle: "planning".to_string(),
            generate_uuids: true,
            include_tracking: true,
        }
    }
}

impl RiskProcessor {
    /// Create a new risk processor
    pub fn new() -> Self {
        Self {
            config: RiskProcessorConfig::default(),
        }
    }

    /// Create a new risk processor with custom configuration
    pub fn with_config(config: RiskProcessorConfig) -> Self {
        Self { config }
    }

    /// Process a risk from JSON data
    pub fn process_risk(&self, risk: &serde_json::Value) -> Result<OscalRisk> {
        let uuid = risk.get("uuid")
            .and_then(|v| v.as_str())
            .unwrap_or(&Uuid::new_v4().to_string())
            .to_string();

        let title = risk.get("title")
            .or_else(|| risk.get("risk_title"))
            .and_then(|v| v.as_str())
            .unwrap_or("Risk")
            .to_string();

        let description = risk.get("description")
            .or_else(|| risk.get("risk_description"))
            .and_then(|v| v.as_str())
            .unwrap_or("No description provided")
            .to_string();

        let statement = risk.get("statement")
            .or_else(|| risk.get("risk_statement"))
            .or_else(|| risk.get("description"))
            .and_then(|v| v.as_str())
            .unwrap_or(&description)
            .to_string();

        let status = risk.get("status")
            .and_then(|v| v.as_str())
            .unwrap_or(&self.config.default_status)
            .to_string();

        // Create properties
        let mut props = Vec::new();

        if let Some(severity) = risk.get("severity").and_then(|v| v.as_str()) {
            props.push(OscalProperty {
                name: "severity".to_string(),
                value: severity.to_string(),
                class: Some("impact".to_string()),
            });
        }

        if let Some(likelihood) = risk.get("likelihood").and_then(|v| v.as_str()) {
            props.push(OscalProperty {
                name: "likelihood".to_string(),
                value: likelihood.to_string(),
                class: Some("probability".to_string()),
            });
        }

        // Create threat IDs if enabled
        let threat_ids = if self.config.generate_threat_ids {
            Some(self.create_threat_ids(risk)?)
        } else {
            None
        };

        // Create characterizations if enabled
        let characterizations = if self.config.include_characterizations {
            Some(self.create_characterizations(risk)?)
        } else {
            None
        };

        Ok(OscalRisk {
            uuid,
            title,
            description,
            statement,
            props: if props.is_empty() { None } else { Some(props) },
            status,
            origins: None,
            threat_ids,
            characterizations,
            mitigating_factors: None,
            deadline: risk.get("deadline").and_then(|v| v.as_str()).map(|s| s.to_string()),
            remediations: None,
            risk_log: None,
        })
    }

    /// Extract risk from POA&M item
    pub fn extract_from_poam_item(&self, item: &serde_json::Value) -> Result<Option<OscalRisk>> {
        // Check if POA&M item contains risk information
        if let Some(severity) = item.get("severity").and_then(|v| v.as_str()) {
            if severity.to_lowercase() == "critical" || severity.to_lowercase() == "high" {
                let risk_data = serde_json::json!({
                    "uuid": format!("risk-{}", item.get("uuid").and_then(|v| v.as_str()).unwrap_or("unknown")),
                    "title": format!("Risk from {}", item.get("title").and_then(|v| v.as_str()).unwrap_or("POA&M Item")),
                    "description": item.get("description").and_then(|v| v.as_str()).unwrap_or("Risk derived from POA&M item"),
                    "severity": severity,
                    "status": "open"
                });

                return Ok(Some(self.process_risk(&risk_data)?));
            }
        }

        Ok(None)
    }

    /// Create threat IDs from risk data
    fn create_threat_ids(&self, risk: &serde_json::Value) -> Result<Vec<OscalThreatId>> {
        let mut threat_ids = Vec::new();

        if let Some(threat_id) = risk.get("threat_id").and_then(|v| v.as_str()) {
            threat_ids.push(OscalThreatId {
                system: "https://cve.mitre.org".to_string(),
                href: Some(format!("https://cve.mitre.org/cgi-bin/cvename.cgi?name={}", threat_id)),
                id: threat_id.to_string(),
            });
        }

        if let Some(cve_id) = risk.get("cve_id").and_then(|v| v.as_str()) {
            threat_ids.push(OscalThreatId {
                system: "https://cve.mitre.org".to_string(),
                href: Some(format!("https://cve.mitre.org/cgi-bin/cvename.cgi?name={}", cve_id)),
                id: cve_id.to_string(),
            });
        }

        Ok(threat_ids)
    }

    /// Create characterizations from risk data
    fn create_characterizations(&self, risk: &serde_json::Value) -> Result<Vec<OscalCharacterization>> {
        let mut characterizations = Vec::new();

        // Create basic characterization
        let mut facets = Vec::new();

        if let Some(severity) = risk.get("severity").and_then(|v| v.as_str()) {
            facets.push(OscalFacet {
                name: "severity".to_string(),
                system: "https://fedramp.gov".to_string(),
                value: severity.to_string(),
                props: None,
            });
        }

        if let Some(likelihood) = risk.get("likelihood").and_then(|v| v.as_str()) {
            facets.push(OscalFacet {
                name: "likelihood".to_string(),
                system: "https://fedramp.gov".to_string(),
                value: likelihood.to_string(),
                props: None,
            });
        }

        if !facets.is_empty() {
            characterizations.push(OscalCharacterization {
                props: Vec::new(),
                links: None,
                origin: OscalOrigin {
                    actors: vec![OscalActor {
                        actor_type: "tool".to_string(),
                        actor_uuid: Uuid::new_v4().to_string(),
                        role_id: Some("assessor".to_string()),
                        props: None,
                    }],
                    related_tasks: None,
                },
                facets,
            });
        }

        Ok(characterizations)
    }
}

impl Default for RiskProcessor {
    fn default() -> Self {
        Self::new()
    }
}

impl Default for RiskProcessorConfig {
    fn default() -> Self {
        Self {
            default_status: "open".to_string(),
            include_characterizations: true,
            generate_threat_ids: true,
        }
    }
}

impl ObservationProcessor {
    /// Create a new observation processor
    pub fn new() -> Self {
        Self {
            config: ObservationProcessorConfig::default(),
        }
    }

    /// Create a new observation processor with custom configuration
    pub fn with_config(config: ObservationProcessorConfig) -> Self {
        Self { config }
    }

    /// Process an observation from JSON data
    pub fn process_observation(&self, observation: &serde_json::Value) -> Result<OscalObservation> {
        let uuid = observation.get("uuid")
            .and_then(|v| v.as_str())
            .unwrap_or(&Uuid::new_v4().to_string())
            .to_string();

        let title = observation.get("title").and_then(|v| v.as_str()).map(|s| s.to_string());

        let description = observation.get("description")
            .and_then(|v| v.as_str())
            .unwrap_or("No description provided")
            .to_string();

        let methods = observation.get("methods")
            .and_then(|v| v.as_array())
            .map(|arr| arr.iter().filter_map(|v| v.as_str()).map(|s| s.to_string()).collect())
            .unwrap_or_else(|| self.config.default_methods.clone());

        let collected = observation.get("collected")
            .and_then(|v| v.as_str())
            .unwrap_or(&Utc::now().to_rfc3339())
            .to_string();

        let expires = if let Some(days) = self.config.default_expiration_days {
            let expiry_date = Utc::now() + chrono::Duration::days(days as i64);
            Some(expiry_date.to_rfc3339())
        } else {
            observation.get("expires").and_then(|v| v.as_str()).map(|s| s.to_string())
        };

        Ok(OscalObservation {
            uuid,
            title,
            description,
            props: None,
            methods,
            types: None,
            origins: None,
            subjects: None,
            relevant_evidence: None,
            collected,
            expires,
        })
    }

    /// Extract observations from POA&M item
    pub fn extract_from_poam_item(&self, item: &serde_json::Value) -> Result<Vec<OscalObservation>> {
        let mut observations = Vec::new();

        // Create observation from POA&M item if it contains observational data
        if let Some(description) = item.get("vulnerability_description").and_then(|v| v.as_str()) {
            let obs_data = serde_json::json!({
                "uuid": format!("obs-{}", item.get("uuid").and_then(|v| v.as_str()).unwrap_or("unknown")),
                "description": format!("Observation from POA&M: {}", description),
                "methods": ["examine"],
                "collected": Utc::now().to_rfc3339()
            });

            observations.push(self.process_observation(&obs_data)?);
        }

        Ok(observations)
    }
}

impl Default for ObservationProcessor {
    fn default() -> Self {
        Self::new()
    }
}

impl Default for ObservationProcessorConfig {
    fn default() -> Self {
        Self {
            default_methods: vec!["examine".to_string(), "interview".to_string(), "test".to_string()],
            include_evidence: true,
            default_expiration_days: Some(365), // 1 year
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[tokio::test]
    async fn test_oscal_generator_creation() {
        let generator = OscalGenerator::new();
        assert_eq!(generator.oscal_version, "1.1.2");
        assert!(generator.validate_output);
    }

    #[tokio::test]
    async fn test_poam_generation_basic() {
        let generator = OscalGenerator::new();

        let content = json!({
            "poam_items": [
                {
                    "uuid": "test-uuid-1",
                    "title": "Test Vulnerability",
                    "description": "A test vulnerability description",
                    "severity": "High",
                    "status": "Open"
                }
            ]
        });

        let metadata = json!({
            "source_file": "test.xlsx",
            "parser_version": "1.0.0"
        });

        let result = generator.generate_poam(&content, &metadata).unwrap();

        // Verify document structure
        assert!(result.get("plan-of-action-and-milestones").is_some());
        let poam_doc = result.get("plan-of-action-and-milestones").unwrap();

        assert!(poam_doc.get("uuid").is_some());
        assert!(poam_doc.get("metadata").is_some());
        assert!(poam_doc.get("system-id").is_some());
        assert!(poam_doc.get("poam-items").is_some());
    }

    #[tokio::test]
    async fn test_poam_item_processor() {
        let processor = PoamItemProcessor::new();

        let item_data = json!({
            "uuid": "test-item-1",
            "title": "Test Item",
            "description": "Test description",
            "severity": "Critical",
            "status": "Open"
        });

        let result = processor.process_item(&item_data).unwrap();

        assert_eq!(result.uuid, "test-item-1");
        assert_eq!(result.title, "Test Item");
        assert_eq!(result.description, "Test description");
        assert!(result.props.is_some());
    }

    #[tokio::test]
    async fn test_metadata_creation() {
        let generator = OscalGenerator::new();

        let source_metadata = json!({
            "source_file": "test.xlsx",
            "parser_version": "1.0.0"
        });

        let result = generator.create_metadata(
            "Test Document",
            "2024-01-15T10:00:00Z",
            &source_metadata
        ).unwrap();

        assert_eq!(result.title, "Test Document");
        assert_eq!(result.oscal_version, "1.1.2");
        assert!(result.props.is_some());
    }

    #[test]
    fn test_uuid_generation() {
        let uuid1 = OscalGenerator::generate_uuid();
        let uuid2 = OscalGenerator::generate_uuid();

        assert_ne!(uuid1, uuid2);
        assert_eq!(uuid1.len(), 36); // Standard UUID length
        assert!(uuid1.contains('-'));
    }
}
