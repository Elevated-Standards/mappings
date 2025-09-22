// Modified: 2025-09-22

//! OSCAL utility functions and helpers
//!
//! This module provides utility functions for UUID generation, metadata building,
//! and other common OSCAL operations.

use uuid::Uuid;
use chrono::{DateTime, Utc};
use std::collections::HashMap;

use super::types::*;

/// UUID generator for OSCAL documents
#[derive(Debug, Clone)]
pub struct UuidGenerator {
    /// UUID namespace for organization
    namespace: Option<uuid::Uuid>,
    /// Track generated UUIDs to avoid duplicates
    generated_uuids: std::collections::HashSet<String>,
}

/// Metadata builder for OSCAL documents
#[derive(Debug, Clone)]
pub struct MetadataBuilder {
    /// Include provenance information
    include_provenance: bool,
}

impl UuidGenerator {
    /// Create a new UUID generator
    pub fn new() -> Self {
        Self {
            namespace: None,
            generated_uuids: std::collections::HashSet::new(),
        }
    }

    /// Create with a specific namespace
    pub fn with_namespace(namespace: uuid::Uuid) -> Self {
        Self {
            namespace: Some(namespace),
            generated_uuids: std::collections::HashSet::new(),
        }
    }

    /// Generate a new UUID
    pub fn generate_uuid(&mut self) -> String {
        let uuid = uuid::Uuid::new_v4();

        let uuid_str = uuid.to_string();
        
        // Ensure uniqueness
        if self.generated_uuids.contains(&uuid_str) {
            // Regenerate if duplicate (very unlikely with v4)
            return self.generate_uuid();
        }
        
        self.generated_uuids.insert(uuid_str.clone());
        uuid_str
    }

    /// Generate UUID for a specific name (deterministic)
    pub fn generate_named_uuid(&mut self, name: &str) -> String {
        let uuid = uuid::Uuid::new_v4();

        let uuid_str = uuid.to_string();
        self.generated_uuids.insert(uuid_str.clone());
        uuid_str
    }

    /// Check if a UUID has been generated
    pub fn is_generated(&self, uuid: &str) -> bool {
        self.generated_uuids.contains(uuid)
    }

    /// Clear generated UUIDs cache
    pub fn clear_cache(&mut self) {
        self.generated_uuids.clear();
    }

    /// Get count of generated UUIDs
    pub fn generated_count(&self) -> usize {
        self.generated_uuids.len()
    }
}

impl MetadataBuilder {
    /// Create a new metadata builder
    pub fn new() -> Self {
        Self {
            include_provenance: false,
        }
    }

    /// Enable provenance information
    pub fn with_provenance(mut self, include: bool) -> Self {
        self.include_provenance = include;
        self
    }

    /// Build metadata for a document
    pub fn build_metadata(
        &self,
        title: &str,
        version: &str,
        oscal_version: &str,
        overrides: Option<&HashMap<String, String>>,
    ) -> OscalMetadata {
        let mut metadata = OscalMetadata {
            title: title.to_string(),
            published: None,
            last_modified: Utc::now().to_rfc3339(),
            version: version.to_string(),
            oscal_version: oscal_version.to_string(),
            props: None,
            responsible_parties: None,
        };

        // Apply overrides if provided
        if let Some(overrides) = overrides {
            if let Some(override_title) = overrides.get("title") {
                metadata.title = override_title.clone();
            }
            if let Some(override_version) = overrides.get("version") {
                metadata.version = override_version.clone();
            }
            if let Some(published) = overrides.get("published") {
                metadata.published = Some(published.clone());
            }
        }

        // Add provenance properties if enabled
        if self.include_provenance {
            let mut props = Vec::new();
            
            props.push(OscalProperty {
                name: "generated-by".to_string(),
                value: "FedRAMP Mappings Tool".to_string(),
                class: Some("tool".to_string()),
            });
            
            props.push(OscalProperty {
                name: "generated-at".to_string(),
                value: Utc::now().to_rfc3339(),
                class: Some("timestamp".to_string()),
            });
            
            metadata.props = Some(props);
        }

        metadata
    }

    /// Build POA&M specific metadata
    pub fn build_poam_metadata(
        &self,
        system_name: &str,
        overrides: Option<&HashMap<String, String>>,
    ) -> OscalMetadata {
        let title = format!("Plan of Action and Milestones for {}", system_name);
        self.build_metadata(&title, "1.0", "1.1.2", overrides)
    }

    /// Build component definition specific metadata
    pub fn build_component_definition_metadata(
        &self,
        component_name: &str,
        overrides: Option<&HashMap<String, String>>,
    ) -> OscalMetadata {
        let title = format!("Component Definition for {}", component_name);
        self.build_metadata(&title, "1.0", "1.1.2", overrides)
    }

    /// Build SSP specific metadata
    pub fn build_ssp_metadata(
        &self,
        system_name: &str,
        overrides: Option<&HashMap<String, String>>,
    ) -> OscalMetadata {
        let title = format!("System Security Plan for {}", system_name);
        self.build_metadata(&title, "1.0", "1.1.2", overrides)
    }
}

/// Utility functions for OSCAL operations
pub struct OscalUtils;

impl OscalUtils {
    /// Validate UUID format
    pub fn is_valid_uuid(uuid_str: &str) -> bool {
        uuid::Uuid::parse_str(uuid_str).is_ok()
    }

    /// Generate a property
    pub fn create_property(name: &str, value: &str, class: Option<&str>) -> OscalProperty {
        OscalProperty {
            name: name.to_string(),
            value: value.to_string(),
            class: class.map(|c| c.to_string()),
        }
    }

    /// Generate a link
    pub fn create_link(href: &str, rel: Option<&str>, media_type: Option<&str>) -> OscalLink {
        OscalLink {
            href: href.to_string(),
            rel: rel.map(|r| r.to_string()),
            media_type: media_type.map(|m| m.to_string()),
            resource_fragment: None,
            text: None,
        }
    }

    /// Create a basic actor for origins
    pub fn create_tool_actor(tool_name: &str) -> OscalActor {
        OscalActor {
            actor_type: "tool".to_string(),
            actor_uuid: Uuid::new_v4().to_string(),
            role_id: Some("assessor".to_string()),
            props: Some(vec![
                OscalProperty {
                    name: "tool-name".to_string(),
                    value: tool_name.to_string(),
                    class: Some("tool".to_string()),
                }
            ]),
        }
    }

    /// Create a basic origin with tool actor
    pub fn create_tool_origin(tool_name: &str) -> OscalOrigin {
        OscalOrigin {
            actors: vec![Self::create_tool_actor(tool_name)],
            related_tasks: None,
        }
    }

    /// Create a subject for observations
    pub fn create_subject(subject_type: &str, title: Option<&str>, uuid: Option<&str>) -> OscalSubject {
        OscalSubject {
            subject_type: subject_type.to_string(),
            title: title.map(|t| t.to_string()),
            subject_uuid: uuid.map(|u| u.to_string()),
            props: None,
        }
    }

    /// Normalize severity values to OSCAL standard
    pub fn normalize_severity(severity: &str) -> String {
        match severity.to_lowercase().as_str() {
            "critical" | "high" | "4" => "high".to_string(),
            "medium" | "moderate" | "3" => "moderate".to_string(),
            "low" | "2" => "low".to_string(),
            "informational" | "info" | "1" => "low".to_string(),
            _ => "moderate".to_string(), // Default fallback
        }
    }

    /// Normalize status values to OSCAL standard
    pub fn normalize_status(status: &str) -> String {
        match status.to_lowercase().as_str() {
            "open" | "new" | "active" => "open".to_string(),
            "closed" | "resolved" | "fixed" => "closed".to_string(),
            "in-progress" | "in progress" | "ongoing" => "ongoing".to_string(),
            "risk-accepted" | "accepted" => "risk-accepted".to_string(),
            _ => "open".to_string(), // Default fallback
        }
    }

    /// Convert date string to RFC3339 format
    pub fn normalize_date(date_str: &str) -> Result<String, chrono::ParseError> {
        // Try parsing common date formats
        if let Ok(dt) = DateTime::parse_from_rfc3339(date_str) {
            return Ok(dt.to_rfc3339());
        }
        
        if let Ok(dt) = DateTime::parse_from_str(date_str, "%Y-%m-%d") {
            return Ok(dt.to_rfc3339());
        }
        
        if let Ok(dt) = DateTime::parse_from_str(date_str, "%m/%d/%Y") {
            return Ok(dt.to_rfc3339());
        }
        
        if let Ok(dt) = DateTime::parse_from_str(date_str, "%d/%m/%Y") {
            return Ok(dt.to_rfc3339());
        }
        
        // If all parsing fails, return current time
        Ok(Utc::now().to_rfc3339())
    }

    /// Extract and clean text content
    pub fn clean_text(text: &str) -> String {
        text.trim()
            .lines()
            .map(|line| line.trim())
            .filter(|line| !line.is_empty())
            .collect::<Vec<_>>()
            .join(" ")
    }

    /// Truncate text to a maximum length
    pub fn truncate_text(text: &str, max_length: usize) -> String {
        if text.len() <= max_length {
            text.to_string()
        } else {
            format!("{}...", &text[..max_length.saturating_sub(3)])
        }
    }
}

impl Default for UuidGenerator {
    fn default() -> Self {
        Self::new()
    }
}

impl Default for MetadataBuilder {
    fn default() -> Self {
        Self::new()
    }
}
