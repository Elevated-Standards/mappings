// Modified: 2025-09-20

//! OSCAL output generation and validation
//!
//! This module provides functionality to generate valid OSCAL JSON documents
//! from parsed document content with proper validation and metadata.

use fedramp_core::{Result, Error};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tracing::{debug, info, warn};
use uuid::Uuid;

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

/// OSCAL generator for creating valid OSCAL documents
#[derive(Debug, Clone)]
pub struct OscalGenerator {
    /// OSCAL version to generate
    oscal_version: String,
    /// Whether to validate output against schemas
    validate_output: bool,
}

impl OscalGenerator {
    /// Create a new OSCAL generator
    #[must_use]
    pub fn new() -> Self {
        Self {
            oscal_version: "1.1.2".to_string(),
            validate_output: true,
        }
    }

    /// Create a new OSCAL generator with custom configuration
    #[must_use]
    pub fn with_config(oscal_version: String, validate_output: bool) -> Self {
        Self {
            oscal_version,
            validate_output,
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

        // TODO: Implement actual POA&M generation from parsed content
        let poam_document = serde_json::json!({
            "plan-of-action-and-milestones": {
                "uuid": document_uuid,
                "metadata": oscal_metadata,
                "import-ssp": {
                    "href": "#system-security-plan"
                },
                "system-id": {
                    "identifier-type": "https://ietf.org/rfc/rfc4122",
                    "id": Uuid::new_v4().to_string()
                },
                "local-definitions": {
                    "components": [],
                    "inventory-items": [],
                    "users": []
                },
                "observations": [],
                "risks": [],
                "findings": [],
                "poam-items": []
            }
        });

        if self.validate_output {
            self.validate_oscal_document(&poam_document, &OscalDocumentType::PlanOfActionAndMilestones)?;
        }

        Ok(poam_document)
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
    fn create_metadata(&self, title: &str, timestamp: &str, source_metadata: &serde_json::Value) -> Result<OscalMetadata> {
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
    fn validate_oscal_document(&self, document: &serde_json::Value, doc_type: &OscalDocumentType) -> Result<()> {
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
