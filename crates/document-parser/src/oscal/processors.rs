// Modified: 2025-09-22

//! OSCAL processors for transforming data into OSCAL structures
//!
//! This module contains processors that transform parsed document data
//! into valid OSCAL structures with proper validation and enrichment.

use fedramp_core::{Result, Error};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tracing::{debug, info, warn};
use uuid::Uuid;
use chrono::{DateTime, Utc};

use super::types::*;

/// POA&M item processor configuration
#[derive(Debug, Clone)]
pub struct PoamProcessorConfig {
    /// Default status for new POA&M items
    pub default_status: String,
    /// Default lifecycle for remediation
    pub default_lifecycle: String,
    /// Include tracking entries
    pub include_tracking: bool,
    /// Include related observations
    pub include_observations: bool,
    /// Include related risks
    pub include_risks: bool,
}

/// Risk processor configuration
#[derive(Debug, Clone)]
pub struct RiskProcessorConfig {
    /// Default risk status
    pub default_status: String,
    /// Include characterizations
    pub include_characterizations: bool,
    /// Include mitigating factors
    pub include_mitigating_factors: bool,
    /// Include remediations
    pub include_remediations: bool,
}

/// Observation processor configuration
#[derive(Debug, Clone)]
pub struct ObservationProcessorConfig {
    /// Default observation methods
    pub default_methods: Vec<String>,
    /// Include evidence
    pub include_evidence: bool,
    /// Include subjects
    pub include_subjects: bool,
}

/// POA&M item processor
#[derive(Debug, Clone)]
pub struct PoamItemProcessor {
    /// Configuration for POA&M processing
    config: PoamProcessorConfig,
}

/// Risk processor
#[derive(Debug, Clone)]
pub struct RiskProcessor {
    /// Configuration for risk processing
    config: RiskProcessorConfig,
}

/// Observation processor
#[derive(Debug, Clone)]
pub struct ObservationProcessor {
    /// Configuration for observation processing
    config: ObservationProcessorConfig,
}

impl PoamItemProcessor {
    /// Create a new POA&M item processor
    pub fn new() -> Self {
        Self {
            config: PoamProcessorConfig::default(),
        }
    }

    /// Create with custom configuration
    pub fn with_config(config: PoamProcessorConfig) -> Self {
        Self { config }
    }

    /// Process POA&M data into OSCAL POA&M items
    pub fn process_poam_items(
        &self,
        poam_data: &[HashMap<String, serde_json::Value>],
    ) -> Result<Vec<OscalPoamItem>> {
        let mut poam_items = Vec::new();

        for (index, row) in poam_data.iter().enumerate() {
            match self.process_single_poam_item(row, index) {
                Ok(item) => poam_items.push(item),
                Err(e) => {
                    warn!("Failed to process POA&M item at index {}: {}", index, e);
                    continue;
                }
            }
        }

        info!("Processed {} POA&M items", poam_items.len());
        Ok(poam_items)
    }

    /// Process a single POA&M item
    fn process_single_poam_item(
        &self,
        row: &HashMap<String, serde_json::Value>,
        index: usize,
    ) -> Result<OscalPoamItem> {
        // Extract required fields
        let title = self.extract_string_field(row, "title")
            .or_else(|| self.extract_string_field(row, "weakness_name"))
            .or_else(|| self.extract_string_field(row, "finding_title"))
            .unwrap_or_else(|| format!("POA&M Item {}", index + 1));

        let description = self.extract_string_field(row, "description")
            .or_else(|| self.extract_string_field(row, "weakness_description"))
            .or_else(|| self.extract_string_field(row, "finding_description"))
            .unwrap_or_else(|| "No description provided".to_string());

        // Generate UUID
        let uuid = Uuid::new_v4().to_string();

        // Extract properties
        let mut props = Vec::new();
        
        // Add control ID if available
        if let Some(control_id) = self.extract_string_field(row, "control_id") {
            props.push(OscalProperty {
                name: "control-id".to_string(),
                value: control_id,
                class: Some("control".to_string()),
            });
        }

        // Add severity if available
        if let Some(severity) = self.extract_string_field(row, "severity") {
            props.push(OscalProperty {
                name: "severity".to_string(),
                value: severity,
                class: Some("impact".to_string()),
            });
        }

        // Add status if available
        if let Some(status) = self.extract_string_field(row, "status") {
            props.push(OscalProperty {
                name: "status".to_string(),
                value: status,
                class: Some("state".to_string()),
            });
        }

        // Process related observations if configured
        let related_observations = if self.config.include_observations {
            self.process_related_observations(row)?
        } else {
            None
        };

        // Process related risks if configured
        let related_risks = if self.config.include_risks {
            self.process_related_risks(row)?
        } else {
            None
        };

        // Process remediation tracking if configured
        let remediation_tracking = if self.config.include_tracking {
            self.process_remediation_tracking(row)?
        } else {
            None
        };

        Ok(OscalPoamItem {
            uuid,
            title,
            description,
            props: if props.is_empty() { None } else { Some(props) },
            related_observations,
            related_risks,
            remediation_tracking,
        })
    }

    /// Extract string field from row data
    fn extract_string_field(&self, row: &HashMap<String, serde_json::Value>, field: &str) -> Option<String> {
        row.get(field)
            .and_then(|v| v.as_str())
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
    }

    /// Process related observations
    fn process_related_observations(
        &self,
        row: &HashMap<String, serde_json::Value>,
    ) -> Result<Option<Vec<OscalRelatedObservation>>> {
        // Look for observation references
        let mut observations = Vec::new();

        if let Some(observation_id) = self.extract_string_field(row, "observation_id") {
            observations.push(OscalRelatedObservation {
                observation_uuid: observation_id,
            });
        }

        if observations.is_empty() {
            Ok(None)
        } else {
            Ok(Some(observations))
        }
    }

    /// Process related risks
    fn process_related_risks(
        &self,
        row: &HashMap<String, serde_json::Value>,
    ) -> Result<Option<Vec<OscalRelatedRisk>>> {
        let mut risks = Vec::new();

        if let Some(risk_id) = self.extract_string_field(row, "risk_id") {
            risks.push(OscalRelatedRisk {
                risk_uuid: risk_id,
            });
        }

        if risks.is_empty() {
            Ok(None)
        } else {
            Ok(Some(risks))
        }
    }

    /// Process remediation tracking
    fn process_remediation_tracking(
        &self,
        row: &HashMap<String, serde_json::Value>,
    ) -> Result<Option<OscalRemediationTracking>> {
        let mut tracking_entries = Vec::new();

        // Create a tracking entry from available data
        let entry_uuid = Uuid::new_v4().to_string();
        let mut props = Vec::new();

        // Add milestone date if available
        if let Some(milestone_date) = self.extract_string_field(row, "milestone_date") {
            props.push(OscalProperty {
                name: "milestone-date".to_string(),
                value: milestone_date,
                class: Some("date".to_string()),
            });
        }

        // Add scheduled completion date if available
        if let Some(scheduled_completion) = self.extract_string_field(row, "scheduled_completion_date") {
            props.push(OscalProperty {
                name: "scheduled-completion".to_string(),
                value: scheduled_completion,
                class: Some("date".to_string()),
            });
        }

        let tracking_entry = OscalTrackingEntry {
            uuid: entry_uuid,
            title: Some("Initial Entry".to_string()),
            description: self.extract_string_field(row, "remediation_plan"),
            props: if props.is_empty() { None } else { Some(props) },
            status_change: self.extract_string_field(row, "status"),
            date_time_stamp: Utc::now().to_rfc3339(),
        };

        tracking_entries.push(tracking_entry);

        if tracking_entries.is_empty() {
            Ok(None)
        } else {
            Ok(Some(OscalRemediationTracking { tracking_entries }))
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

    /// Create with custom configuration
    pub fn with_config(config: RiskProcessorConfig) -> Self {
        Self { config }
    }

    /// Process risk data into OSCAL risks
    pub fn process_risks(
        &self,
        risk_data: &[HashMap<String, serde_json::Value>],
    ) -> Result<Vec<OscalRisk>> {
        let mut risks = Vec::new();

        for (index, row) in risk_data.iter().enumerate() {
            match self.process_single_risk(row, index) {
                Ok(risk) => risks.push(risk),
                Err(e) => {
                    warn!("Failed to process risk at index {}: {}", index, e);
                    continue;
                }
            }
        }

        info!("Processed {} risks", risks.len());
        Ok(risks)
    }

    /// Process a single risk
    fn process_single_risk(
        &self,
        row: &HashMap<String, serde_json::Value>,
        index: usize,
    ) -> Result<OscalRisk> {
        let uuid = Uuid::new_v4().to_string();
        
        let title = self.extract_string_field(row, "title")
            .or_else(|| self.extract_string_field(row, "risk_title"))
            .unwrap_or_else(|| format!("Risk {}", index + 1));

        let description = self.extract_string_field(row, "description")
            .or_else(|| self.extract_string_field(row, "risk_description"))
            .unwrap_or_else(|| "No description provided".to_string());

        let statement = self.extract_string_field(row, "statement")
            .or_else(|| self.extract_string_field(row, "risk_statement"))
            .unwrap_or_else(|| description.clone());

        let status = self.extract_string_field(row, "status")
            .unwrap_or_else(|| self.config.default_status.clone());

        // Build properties
        let mut props = Vec::new();
        
        if let Some(likelihood) = self.extract_string_field(row, "likelihood") {
            props.push(OscalProperty {
                name: "likelihood".to_string(),
                value: likelihood,
                class: Some("assessment".to_string()),
            });
        }

        if let Some(impact) = self.extract_string_field(row, "impact") {
            props.push(OscalProperty {
                name: "impact".to_string(),
                value: impact,
                class: Some("assessment".to_string()),
            });
        }

        Ok(OscalRisk {
            uuid,
            title,
            description,
            statement,
            props: if props.is_empty() { None } else { Some(props) },
            status,
            origins: None,
            threat_ids: None,
            characterizations: None,
            mitigating_factors: None,
            deadline: self.extract_string_field(row, "deadline"),
            remediations: None,
            risk_log: None,
            related_observations: None,
        })
    }

    /// Extract string field from row data
    fn extract_string_field(&self, row: &HashMap<String, serde_json::Value>, field: &str) -> Option<String> {
        row.get(field)
            .and_then(|v| v.as_str())
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
    }
}

impl ObservationProcessor {
    /// Create a new observation processor
    pub fn new() -> Self {
        Self {
            config: ObservationProcessorConfig::default(),
        }
    }

    /// Create with custom configuration
    pub fn with_config(config: ObservationProcessorConfig) -> Self {
        Self { config }
    }

    /// Process observation data into OSCAL observations
    pub fn process_observations(
        &self,
        observation_data: &[HashMap<String, serde_json::Value>],
    ) -> Result<Vec<OscalObservation>> {
        let mut observations = Vec::new();

        for (index, row) in observation_data.iter().enumerate() {
            match self.process_single_observation(row, index) {
                Ok(observation) => observations.push(observation),
                Err(e) => {
                    warn!("Failed to process observation at index {}: {}", index, e);
                    continue;
                }
            }
        }

        info!("Processed {} observations", observations.len());
        Ok(observations)
    }

    /// Process a single observation
    fn process_single_observation(
        &self,
        row: &HashMap<String, serde_json::Value>,
        index: usize,
    ) -> Result<OscalObservation> {
        let uuid = Uuid::new_v4().to_string();
        
        let title = self.extract_string_field(row, "title")
            .or_else(|| self.extract_string_field(row, "observation_title"));

        let description = self.extract_string_field(row, "description")
            .or_else(|| self.extract_string_field(row, "observation_description"))
            .unwrap_or_else(|| format!("Observation {}", index + 1));

        let methods = self.extract_string_field(row, "methods")
            .map(|m| m.split(',').map(|s| s.trim().to_string()).collect())
            .unwrap_or_else(|| self.config.default_methods.clone());

        let collected = self.extract_string_field(row, "collected")
            .unwrap_or_else(|| Utc::now().to_rfc3339());

        // Create a basic origin
        let actor = OscalActor {
            actor_type: "tool".to_string(),
            actor_uuid: Uuid::new_v4().to_string(),
            role_id: Some("assessor".to_string()),
            props: None,
        };

        let origin = OscalOrigin {
            actors: vec![actor],
            related_tasks: None,
        };

        Ok(OscalObservation {
            uuid,
            title,
            description,
            props: None,
            methods,
            types: None,
            origins: vec![origin],
            subjects: None,
            relevant_evidence: None,
            collected,
            expires: None,
            remarks: None,
        })
    }

    /// Extract string field from row data
    fn extract_string_field(&self, row: &HashMap<String, serde_json::Value>, field: &str) -> Option<String> {
        row.get(field)
            .and_then(|v| v.as_str())
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
    }
}

impl Default for PoamProcessorConfig {
    fn default() -> Self {
        Self {
            default_status: "open".to_string(),
            default_lifecycle: "planning".to_string(),
            include_tracking: true,
            include_observations: true,
            include_risks: true,
        }
    }
}

impl Default for RiskProcessorConfig {
    fn default() -> Self {
        Self {
            default_status: "open".to_string(),
            include_characterizations: true,
            include_mitigating_factors: true,
            include_remediations: true,
        }
    }
}

impl Default for ObservationProcessorConfig {
    fn default() -> Self {
        Self {
            default_methods: vec!["examine".to_string(), "interview".to_string(), "test".to_string()],
            include_evidence: true,
            include_subjects: true,
        }
    }
}

impl Default for PoamItemProcessor {
    fn default() -> Self {
        Self::new()
    }
}

impl Default for RiskProcessor {
    fn default() -> Self {
        Self::new()
    }
}

impl Default for ObservationProcessor {
    fn default() -> Self {
        Self::new()
    }
}
