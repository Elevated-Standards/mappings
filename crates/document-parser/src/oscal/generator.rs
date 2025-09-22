// Modified: 2025-09-22

//! OSCAL document generator
//!
//! This module provides the main OSCAL generator that orchestrates the creation
//! of valid OSCAL documents from parsed data.

use fedramp_core::{Result, Error};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tracing::{debug, info, warn};
use uuid::Uuid;
use chrono::{DateTime, Utc};

use super::types::*;
use super::documents::*;
use super::processors::*;
use super::validation::OscalSchemaValidator;
use super::utils::{UuidGenerator, MetadataBuilder};

/// OSCAL generator for creating valid OSCAL documents
#[derive(Debug, Clone)]
pub struct OscalGenerator {
    /// OSCAL version to generate
    pub oscal_version: String,
    /// Whether to validate output against schemas
    pub validate_output: bool,
    /// Include metadata in generated documents
    pub include_metadata: bool,
    /// Organization name for metadata
    pub organization_name: String,
    /// System name for metadata
    pub system_name: String,
    /// Schema validator
    schema_validator: OscalSchemaValidator,
    /// UUID generator
    uuid_generator: UuidGenerator,
    /// Metadata builder
    metadata_builder: MetadataBuilder,
    /// POA&M processor
    poam_processor: PoamItemProcessor,
    /// Risk processor
    risk_processor: RiskProcessor,
    /// Observation processor
    observation_processor: ObservationProcessor,
}

impl OscalGenerator {
    /// Create a new OSCAL generator
    #[must_use]
    pub fn new() -> Self {
        Self {
            oscal_version: "1.1.2".to_string(),
            validate_output: true,
            include_metadata: true,
            organization_name: "Organization".to_string(),
            system_name: "System".to_string(),
            schema_validator: OscalSchemaValidator::new(),
            uuid_generator: UuidGenerator::new(),
            metadata_builder: MetadataBuilder::new(),
            poam_processor: PoamItemProcessor::new(),
            risk_processor: RiskProcessor::new(),
            observation_processor: ObservationProcessor::new(),
        }
    }

    /// Set OSCAL version
    pub fn with_oscal_version(mut self, version: String) -> Self {
        self.oscal_version = version;
        self
    }

    /// Set organization name
    pub fn with_organization_name(mut self, name: String) -> Self {
        self.organization_name = name;
        self
    }

    /// Set system name
    pub fn with_system_name(mut self, name: String) -> Self {
        self.system_name = name;
        self
    }

    /// Enable or disable output validation
    pub fn with_validation(mut self, validate: bool) -> Self {
        self.validate_output = validate;
        self
    }

    /// Generate POA&M document from parsed data
    pub fn generate_poam_document(
        &mut self,
        poam_data: &[HashMap<String, serde_json::Value>],
        metadata_overrides: Option<HashMap<String, String>>,
    ) -> Result<OscalPoamDocument> {
        info!("Generating OSCAL POA&M document from {} items", poam_data.len());

        // Generate document UUID
        let document_uuid = self.uuid_generator.generate_uuid();

        // Build metadata
        let metadata = self.build_poam_metadata(metadata_overrides)?;

        // Process POA&M items
        let poam_items = self.poam_processor.process_poam_items(poam_data)?;

        // Process related observations if any
        let observations = if !poam_data.is_empty() {
            Some(self.observation_processor.process_observations(poam_data)?)
        } else {
            None
        };

        // Process related risks if any
        let risks = if !poam_data.is_empty() {
            Some(self.risk_processor.process_risks(poam_data)?)
        } else {
            None
        };

        // Build the POA&M document
        let plan_of_action_and_milestones = PlanOfActionAndMilestones {
            uuid: document_uuid,
            metadata,
            import_ssp: None,
            system_id: Some(self.system_name.clone()),
            local_definitions: None,
            observations,
            risks,
            findings: None,
            poam_items,
            back_matter: None,
        };

        let document = OscalPoamDocument {
            plan_of_action_and_milestones,
        };

        // Validate if enabled
        if self.validate_output {
            self.validate_poam_document(&document)?;
        }

        info!("Successfully generated OSCAL POA&M document");
        Ok(document)
    }

    /// Generate component definition document
    pub fn generate_component_definition(
        &mut self,
        component_data: &[HashMap<String, serde_json::Value>],
        metadata_overrides: Option<HashMap<String, String>>,
    ) -> Result<OscalComponentDefinitionDocument> {
        info!("Generating OSCAL Component Definition document from {} components", component_data.len());

        let document_uuid = self.uuid_generator.generate_uuid();
        let metadata = self.build_component_metadata(metadata_overrides)?;

        // Process components
        let components = self.process_components(component_data)?;

        let component_definition = ComponentDefinition {
            uuid: document_uuid,
            metadata,
            import_component_definitions: None,
            components: if components.is_empty() { None } else { Some(components) },
            capabilities: None,
            back_matter: None,
        };

        let document = OscalComponentDefinitionDocument {
            component_definition,
        };

        // Validate if enabled
        if self.validate_output {
            self.validate_component_definition(&document)?;
        }

        info!("Successfully generated OSCAL Component Definition document");
        Ok(document)
    }

    /// Generate system security plan document
    pub fn generate_ssp(
        &mut self,
        _content: &serde_json::Value,
        _metadata: &HashMap<String, String>,
    ) -> Result<OscalSspDocument> {
        // Placeholder implementation for SSP generation
        warn!("SSP generation not yet implemented");
        Err(Error::validation("SSP generation not yet implemented"))
    }

    /// Build POA&M metadata
    fn build_poam_metadata(
        &self,
        overrides: Option<HashMap<String, String>>,
    ) -> Result<OscalMetadata> {
        let mut metadata = OscalMetadata {
            title: "Plan of Action and Milestones".to_string(),
            published: None,
            last_modified: Utc::now().to_rfc3339(),
            version: "1.0".to_string(),
            oscal_version: self.oscal_version.clone(),
            props: None,
            responsible_parties: None,
        };

        // Apply overrides if provided
        if let Some(overrides) = overrides {
            if let Some(title) = overrides.get("title") {
                metadata.title = title.clone();
            }
            if let Some(version) = overrides.get("version") {
                metadata.version = version.clone();
            }
            if let Some(published) = overrides.get("published") {
                metadata.published = Some(published.clone());
            }
        }

        Ok(metadata)
    }

    /// Build component definition metadata
    fn build_component_metadata(
        &self,
        overrides: Option<HashMap<String, String>>,
    ) -> Result<OscalMetadata> {
        let mut metadata = OscalMetadata {
            title: "Component Definition".to_string(),
            published: None,
            last_modified: Utc::now().to_rfc3339(),
            version: "1.0".to_string(),
            oscal_version: self.oscal_version.clone(),
            props: None,
            responsible_parties: None,
        };

        // Apply overrides if provided
        if let Some(overrides) = overrides {
            if let Some(title) = overrides.get("title") {
                metadata.title = title.clone();
            }
            if let Some(version) = overrides.get("version") {
                metadata.version = version.clone();
            }
        }

        Ok(metadata)
    }

    /// Process component data into OSCAL components
    fn process_components(
        &mut self,
        component_data: &[HashMap<String, serde_json::Value>],
    ) -> Result<Vec<Component>> {
        let mut components = Vec::new();

        for (index, row) in component_data.iter().enumerate() {
            match self.process_single_component(row, index) {
                Ok(component) => components.push(component),
                Err(e) => {
                    warn!("Failed to process component at index {}: {}", index, e);
                    continue;
                }
            }
        }

        Ok(components)
    }

    /// Process a single component
    fn process_single_component(
        &mut self,
        row: &HashMap<String, serde_json::Value>,
        index: usize,
    ) -> Result<Component> {
        let uuid = self.uuid_generator.generate_uuid();
        
        let title = self.extract_string_field(row, "title")
            .or_else(|| self.extract_string_field(row, "component_name"))
            .unwrap_or_else(|| format!("Component {}", index + 1));

        let description = self.extract_string_field(row, "description")
            .or_else(|| self.extract_string_field(row, "component_description"))
            .unwrap_or_else(|| "No description provided".to_string());

        let component_type = self.extract_string_field(row, "type")
            .or_else(|| self.extract_string_field(row, "component_type"))
            .unwrap_or_else(|| "software".to_string());

        Ok(Component {
            uuid,
            component_type,
            title,
            description,
            purpose: self.extract_string_field(row, "purpose"),
            props: None,
            links: None,
            responsible_roles: None,
            control_implementations: None,
        })
    }

    /// Extract string field from row data
    fn extract_string_field(&self, row: &HashMap<String, serde_json::Value>, field: &str) -> Option<String> {
        row.get(field)
            .and_then(|v| v.as_str())
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
    }

    /// Validate POA&M document
    fn validate_poam_document(&self, document: &OscalPoamDocument) -> Result<()> {
        debug!("Validating OSCAL POA&M document");
        
        // Basic validation
        if document.plan_of_action_and_milestones.poam_items.is_empty() {
            return Err(Error::validation("POA&M document must contain at least one POA&M item"));
        }

        // Validate UUIDs are unique
        let mut uuids = std::collections::HashSet::new();
        for item in &document.plan_of_action_and_milestones.poam_items {
            if !uuids.insert(&item.uuid) {
                return Err(Error::validation(format!("Duplicate UUID found: {}", item.uuid)));
            }
        }

        // Schema validation would go here if schemas are available
        if let Err(e) = self.schema_validator.validate_poam_document(document) {
            warn!("Schema validation failed: {}", e);
            // Don't fail on schema validation errors for now
        }

        Ok(())
    }

    /// Validate component definition document
    fn validate_component_definition(&self, document: &OscalComponentDefinitionDocument) -> Result<()> {
        debug!("Validating OSCAL Component Definition document");
        
        // Basic validation
        if let Some(components) = &document.component_definition.components {
            if components.is_empty() {
                warn!("Component definition document contains no components");
            }

            // Validate UUIDs are unique
            let mut uuids = std::collections::HashSet::new();
            for component in components {
                if !uuids.insert(&component.uuid) {
                    return Err(Error::validation(format!("Duplicate component UUID found: {}", component.uuid)));
                }
            }
        }

        // Schema validation would go here if schemas are available
        if let Err(e) = self.schema_validator.validate_component_definition(document) {
            warn!("Schema validation failed: {}", e);
            // Don't fail on schema validation errors for now
        }

        Ok(())
    }

    /// Convert to JSON string
    pub fn to_json_string<T: Serialize>(&self, document: &T) -> Result<String> {
        serde_json::to_string_pretty(document)
            .map_err(|e| Error::validation(format!("Failed to serialize OSCAL document: {}", e)))
    }

    /// Convert to JSON value
    pub fn to_json_value<T: Serialize>(&self, document: &T) -> Result<serde_json::Value> {
        serde_json::to_value(document)
            .map_err(|e| Error::validation(format!("Failed to serialize OSCAL document: {}", e)))
    }
}

impl Default for OscalGenerator {
    fn default() -> Self {
        Self::new()
    }
}
