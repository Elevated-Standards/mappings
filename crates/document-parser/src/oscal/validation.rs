// Modified: 2025-09-22

//! OSCAL schema validation
//!
//! This module provides validation functionality for OSCAL documents
//! against official OSCAL schemas.

use fedramp_core::{Result, Error};
use std::collections::HashMap;
use tracing::{debug, info, warn};

use super::types::*;
use super::documents::*;

/// OSCAL schema validator
#[derive(Debug, Clone)]
pub struct OscalSchemaValidator {
    /// Cached compiled schemas
    schemas: HashMap<OscalDocumentType, String>,
    /// Enable strict validation
    strict_validation: bool,
}

impl OscalSchemaValidator {
    /// Create a new OSCAL schema validator
    pub fn new() -> Self {
        Self {
            schemas: HashMap::new(),
            strict_validation: false,
        }
    }

    /// Enable or disable strict validation
    pub fn with_strict_validation(mut self, strict: bool) -> Self {
        self.strict_validation = strict;
        self
    }

    /// Validate a POA&M document
    pub fn validate_poam_document(&self, document: &OscalPoamDocument) -> Result<()> {
        debug!("Validating POA&M document");
        
        // Basic structural validation
        self.validate_poam_structure(document)?;
        
        // Schema validation (placeholder - would use actual JSON schema validation)
        if self.strict_validation {
            self.validate_against_schema(document, &OscalDocumentType::PlanOfActionAndMilestones)?;
        }
        
        info!("POA&M document validation completed successfully");
        Ok(())
    }

    /// Validate a component definition document
    pub fn validate_component_definition(&self, document: &OscalComponentDefinitionDocument) -> Result<()> {
        debug!("Validating Component Definition document");
        
        // Basic structural validation
        self.validate_component_definition_structure(document)?;
        
        // Schema validation (placeholder)
        if self.strict_validation {
            self.validate_against_schema(document, &OscalDocumentType::ComponentDefinition)?;
        }
        
        info!("Component Definition document validation completed successfully");
        Ok(())
    }

    /// Validate a system security plan document
    pub fn validate_ssp_document(&self, document: &OscalSspDocument) -> Result<()> {
        debug!("Validating SSP document");
        
        // Basic structural validation
        self.validate_ssp_structure(document)?;
        
        // Schema validation (placeholder)
        if self.strict_validation {
            self.validate_against_schema(document, &OscalDocumentType::SystemSecurityPlan)?;
        }
        
        info!("SSP document validation completed successfully");
        Ok(())
    }

    /// Validate POA&M document structure
    fn validate_poam_structure(&self, document: &OscalPoamDocument) -> Result<()> {
        let poam = &document.plan_of_action_and_milestones;
        
        // Validate required fields
        if poam.uuid.is_empty() {
            return Err(Error::validation("POA&M document UUID is required"));
        }

        if poam.metadata.title.is_empty() {
            return Err(Error::validation("POA&M document title is required"));
        }

        if poam.metadata.oscal_version.is_empty() {
            return Err(Error::validation("OSCAL version is required"));
        }
        
        // Validate POA&M items
        if poam.poam_items.is_empty() {
            warn!("POA&M document contains no POA&M items");
        }
        
        for (index, item) in poam.poam_items.iter().enumerate() {
            self.validate_poam_item(item, index)?;
        }
        
        // Validate observations if present
        if let Some(observations) = &poam.observations {
            for (index, observation) in observations.iter().enumerate() {
                self.validate_observation(observation, index)?;
            }
        }
        
        // Validate risks if present
        if let Some(risks) = &poam.risks {
            for (index, risk) in risks.iter().enumerate() {
                self.validate_risk(risk, index)?;
            }
        }
        
        Ok(())
    }

    /// Validate component definition structure
    fn validate_component_definition_structure(&self, document: &OscalComponentDefinitionDocument) -> Result<()> {
        let comp_def = &document.component_definition;
        
        // Validate required fields
        if comp_def.uuid.is_empty() {
            return Err(Error::validation("Component Definition UUID is required"));
        }

        if comp_def.metadata.title.is_empty() {
            return Err(Error::validation("Component Definition title is required"));
        }
        
        // Validate components if present
        if let Some(components) = &comp_def.components {
            for (index, component) in components.iter().enumerate() {
                self.validate_component(component, index)?;
            }
        }
        
        Ok(())
    }

    /// Validate SSP structure
    fn validate_ssp_structure(&self, document: &OscalSspDocument) -> Result<()> {
        let ssp = &document.system_security_plan;
        
        // Validate required fields
        if ssp.uuid.is_empty() {
            return Err(Error::validation("SSP UUID is required"));
        }

        if ssp.metadata.title.is_empty() {
            return Err(Error::validation("SSP title is required"));
        }
        
        // Validate system characteristics
        self.validate_system_characteristics(&ssp.system_characteristics)?;
        
        Ok(())
    }

    /// Validate a POA&M item
    fn validate_poam_item(&self, item: &OscalPoamItem, index: usize) -> Result<()> {
        if item.uuid.is_empty() {
            return Err(Error::validation(format!("POA&M item {} UUID is required", index)));
        }

        if item.title.is_empty() {
            return Err(Error::validation(format!("POA&M item {} title is required", index)));
        }

        if item.description.is_empty() {
            return Err(Error::validation(format!("POA&M item {} description is required", index)));
        }

        // Validate UUID format (basic check)
        if !self.is_valid_uuid_format(&item.uuid) {
            return Err(Error::validation(format!("POA&M item {} has invalid UUID format", index)));
        }
        
        Ok(())
    }

    /// Validate an observation
    fn validate_observation(&self, observation: &OscalObservation, index: usize) -> Result<()> {
        if observation.uuid.is_empty() {
            return Err(Error::validation(format!("Observation {} UUID is required", index)));
        }

        if observation.description.is_empty() {
            return Err(Error::validation(format!("Observation {} description is required", index)));
        }

        if observation.methods.is_empty() {
            return Err(Error::validation(format!("Observation {} must have at least one method", index)));
        }

        if observation.origins.is_empty() {
            return Err(Error::validation(format!("Observation {} must have at least one origin", index)));
        }
        
        Ok(())
    }

    /// Validate a risk
    fn validate_risk(&self, risk: &OscalRisk, index: usize) -> Result<()> {
        if risk.uuid.is_empty() {
            return Err(Error::validation(format!("Risk {} UUID is required", index)));
        }

        if risk.title.is_empty() {
            return Err(Error::validation(format!("Risk {} title is required", index)));
        }

        if risk.description.is_empty() {
            return Err(Error::validation(format!("Risk {} description is required", index)));
        }

        if risk.statement.is_empty() {
            return Err(Error::validation(format!("Risk {} statement is required", index)));
        }

        if risk.status.is_empty() {
            return Err(Error::validation(format!("Risk {} status is required", index)));
        }
        
        Ok(())
    }

    /// Validate a component
    fn validate_component(&self, component: &Component, index: usize) -> Result<()> {
        if component.uuid.is_empty() {
            return Err(Error::validation(format!("Component {} UUID is required", index)));
        }

        if component.title.is_empty() {
            return Err(Error::validation(format!("Component {} title is required", index)));
        }

        if component.description.is_empty() {
            return Err(Error::validation(format!("Component {} description is required", index)));
        }

        if component.component_type.is_empty() {
            return Err(Error::validation(format!("Component {} type is required", index)));
        }
        
        Ok(())
    }

    /// Validate system characteristics
    fn validate_system_characteristics(&self, characteristics: &SystemCharacteristics) -> Result<()> {
        if characteristics.system_ids.is_empty() {
            return Err(Error::validation("System must have at least one system ID"));
        }

        if characteristics.system_name.is_empty() {
            return Err(Error::validation("System name is required"));
        }

        if characteristics.description.is_empty() {
            return Err(Error::validation("System description is required"));
        }

        if characteristics.security_sensitivity_level.is_empty() {
            return Err(Error::validation("Security sensitivity level is required"));
        }
        
        Ok(())
    }

    /// Validate against OSCAL schema (placeholder implementation)
    fn validate_against_schema<T>(&self, _document: &T, document_type: &OscalDocumentType) -> Result<()> {
        // This would implement actual JSON schema validation
        // For now, just log that schema validation would occur
        debug!("Schema validation for {:?} (placeholder)", document_type);
        
        // In a real implementation, this would:
        // 1. Load the appropriate OSCAL schema for the document type
        // 2. Serialize the document to JSON
        // 3. Validate the JSON against the schema
        // 4. Return validation errors if any
        
        Ok(())
    }

    /// Check if a string is a valid UUID format
    fn is_valid_uuid_format(&self, uuid_str: &str) -> bool {
        // Basic UUID format validation
        uuid_str.len() == 36 && 
        uuid_str.chars().enumerate().all(|(i, c)| {
            match i {
                8 | 13 | 18 | 23 => c == '-',
                _ => c.is_ascii_hexdigit(),
            }
        })
    }

    /// Load schema for document type (placeholder)
    fn _load_schema(&mut self, document_type: &OscalDocumentType) -> Result<()> {
        if self.schemas.contains_key(document_type) {
            return Ok(());
        }
        
        // In a real implementation, this would load the actual OSCAL schema
        let schema_content = match document_type {
            OscalDocumentType::PlanOfActionAndMilestones => "poam-schema-placeholder",
            OscalDocumentType::ComponentDefinition => "component-definition-schema-placeholder",
            OscalDocumentType::SystemSecurityPlan => "ssp-schema-placeholder",
            OscalDocumentType::AssessmentPlan => "assessment-plan-schema-placeholder",
            OscalDocumentType::AssessmentResults => "assessment-results-schema-placeholder",
        };
        
        self.schemas.insert(document_type.clone(), schema_content.to_string());
        Ok(())
    }
}

impl Default for OscalSchemaValidator {
    fn default() -> Self {
        Self::new()
    }
}
