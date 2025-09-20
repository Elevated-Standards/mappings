// Modified: 2025-01-20

//! Column mapping and field detection for document parsing
//!
//! This module provides functionality to map document columns to OSCAL fields
//! using fuzzy matching and configuration-based mapping rules.

use fedramp_core::{Result, Error};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::{Arc, RwLock};
use std::time::Duration;
use tokio::fs;
use tokio::sync::mpsc;
use tracing::{debug, info, warn, error};
use regex::Regex;
use notify::{Watcher, RecursiveMode, Event, EventKind};
use crate::error::DocumentParserError;
use crate::fuzzy::{FuzzyMatcher, FuzzyMatchConfig, FuzzyMatchResult};

/// Column mapping configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ColumnMapping {
    /// Target OSCAL field name
    pub target_field: String,
    /// Possible column names to match
    pub source_columns: Vec<String>,
    /// Whether this field is required
    pub required: bool,
    /// Data type validation
    pub data_type: Option<String>,
    /// Default value if not found
    pub default_value: Option<serde_json::Value>,
}

/// Comprehensive mapping configuration loaded from JSON files
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MappingConfiguration {
    /// Inventory mappings for asset inventory workbooks
    pub inventory_mappings: Option<InventoryMappings>,
    /// POA&M mappings for POA&M Excel templates
    pub poam_mappings: Option<PoamMappings>,
    /// SSP section mappings for document parsing
    pub ssp_sections: Option<SspSections>,
    /// Control framework mappings
    pub controls: Option<ControlMappings>,
    /// Document structure definitions
    pub documents: Option<DocumentStructures>,
}

/// Inventory mappings configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InventoryMappings {
    pub description: String,
    pub version: String,
    pub fedramp_iiw_mappings: InventoryColumnMappings,
    pub validation_rules: ValidationRules,
    pub component_grouping: ComponentGrouping,
    pub component_type_mappings: HashMap<String, ComponentTypeMapping>,
    pub security_mappings: SecurityMappings,
    pub control_inheritance: ControlInheritance,
}

/// Inventory column mappings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InventoryColumnMappings {
    pub required_columns: HashMap<String, InventoryColumnMapping>,
}

/// Individual inventory column mapping
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InventoryColumnMapping {
    pub column_names: Vec<String>,
    pub field: String,
    pub required: bool,
    pub validation: Option<String>,
}

/// Validation rules for different data types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationRules {
    pub asset_types: Option<Vec<String>>,
    pub environments: Option<Vec<String>>,
    pub criticality_levels: Option<Vec<String>>,
    pub boolean_values: Option<Vec<String>>,
    pub ip_address_pattern: Option<String>,
    pub mac_address_pattern: Option<String>,
}

/// Component grouping strategies
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComponentGrouping {
    pub strategies: HashMap<String, GroupingStrategy>,
}

/// Individual grouping strategy
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GroupingStrategy {
    pub description: String,
    pub priority: u32,
}

/// Component type mapping
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComponentTypeMapping {
    #[serde(rename = "type")]
    pub component_type: String,
    pub keywords: Vec<String>,
}

/// Security mappings for risk assessment
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityMappings {
    pub criticality_to_impact: HashMap<String, ImpactMapping>,
    pub risk_factors: HashMap<String, RiskFactor>,
}

/// Impact level mapping
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImpactMapping {
    pub confidentiality_impact: String,
    pub integrity_impact: String,
    pub availability_impact: String,
}

/// Risk factor configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RiskFactor {
    pub multiplier: f64,
    pub description: String,
}

/// Control inheritance configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ControlInheritance {
    pub infrastructure_controls: Vec<String>,
    pub platform_controls: Vec<String>,
    pub inheritance_mappings: HashMap<String, InheritanceMapping>,
}

/// Individual inheritance mapping
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InheritanceMapping {
    pub inherited_controls: String,
    pub provider_responsibility: String,
}

/// POA&M mappings configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PoamMappings {
    pub description: String,
    pub version: String,
    pub fedramp_v3_mappings: PoamColumnMappings,
    pub risk_mappings: RiskMappings,
    pub finding_mappings: FindingMappings,
    pub milestone_processing: MilestoneProcessing,
    pub quality_checks: QualityChecks,
}

/// POA&M column mappings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PoamColumnMappings {
    pub required_columns: HashMap<String, PoamColumnMapping>,
    pub validation_rules: PoamValidationRules,
}

/// Individual POA&M column mapping
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PoamColumnMapping {
    pub column_names: Vec<String>,
    pub oscal_field: String,
    pub required: bool,
    pub validation: Option<String>,
}

/// POA&M validation rules
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PoamValidationRules {
    pub severity_levels: Option<Vec<String>>,
    pub status_values: Option<Vec<String>>,
    pub control_id_pattern: Option<String>,
    pub date_formats: Option<Vec<String>>,
}

/// Risk mappings for POA&M
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RiskMappings {
    pub severity_to_risk_level: HashMap<String, RiskLevel>,
    pub status_to_implementation: HashMap<String, String>,
}

/// Risk level mapping
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RiskLevel {
    pub risk_impact: String,
    pub risk_likelihood: String,
}

/// Finding mappings for origin detection
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FindingMappings {
    pub origin_types: HashMap<String, OriginType>,
}

/// Origin type mapping
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OriginType {
    pub keywords: Vec<String>,
    pub oscal_origin_type: String,
}

/// Milestone processing configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MilestoneProcessing {
    pub patterns: MilestonePatterns,
}

/// Milestone parsing patterns
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MilestonePatterns {
    pub multiple_milestones: MultipleMilestones,
    pub milestone_format: MilestoneFormat,
}

/// Multiple milestone configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MultipleMilestones {
    pub separator_patterns: Vec<String>,
    pub description: String,
}

/// Milestone format configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MilestoneFormat {
    pub patterns: Vec<String>,
    pub groups: Vec<String>,
}

/// Quality checks configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QualityChecks {
    pub required_field_completeness: RequiredFieldCompleteness,
    pub data_consistency: DataConsistency,
    pub control_validation: ControlValidation,
}

/// Required field completeness checks
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RequiredFieldCompleteness {
    pub critical_fields: Vec<String>,
    pub minimum_completion_rate: f64,
}

/// Data consistency checks
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataConsistency {
    pub date_logic: String,
    pub status_logic: String,
}

/// Control validation configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ControlValidation {
    pub verify_control_ids: bool,
    pub validate_against_catalog: String,
}

/// SSP sections configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SspSections {
    pub section_mappings: SectionMappings,
    pub control_extraction: ControlExtraction,
    pub table_mappings: TableMappings,
}

/// Section mappings configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SectionMappings {
    pub description: String,
    pub version: String,
    pub mappings: HashMap<String, SectionMapping>,
}

/// Individual section mapping
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SectionMapping {
    pub keywords: Vec<String>,
    pub target: String,
    pub required: bool,
    pub extract_patterns: Option<HashMap<String, String>>,
}

/// Control extraction patterns
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ControlExtraction {
    pub patterns: Vec<ExtractionPattern>,
}

/// Individual extraction pattern
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExtractionPattern {
    pub name: String,
    pub regex: String,
    pub description: String,
}

/// Table mappings for structured data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TableMappings {
    pub responsibility_matrix: ResponsibilityMatrix,
    pub inventory_summary: InventorySummary,
}

/// Responsibility matrix mapping
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResponsibilityMatrix {
    pub keywords: Vec<String>,
    pub columns: ResponsibilityColumns,
}

/// Responsibility matrix columns
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResponsibilityColumns {
    pub control_id: Vec<String>,
    pub customer_responsibility: Vec<String>,
    pub csp_responsibility: Vec<String>,
    pub shared_responsibility: Vec<String>,
}

/// Inventory summary mapping
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InventorySummary {
    pub keywords: Vec<String>,
    pub columns: InventorySummaryColumns,
}

/// Inventory summary columns
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InventorySummaryColumns {
    pub component_name: Vec<String>,
    pub component_type: Vec<String>,
    pub criticality: Vec<String>,
    pub environment: Vec<String>,
}

/// Control mappings configuration (from schema files)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ControlMappings {
    pub metadata: ControlMetadata,
    pub controls: Vec<ControlMapping>,
}

/// Control metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ControlMetadata {
    pub source_file: String,
    pub sheet_name: String,
    pub extraction_date: String,
    pub framework: String,
    pub version: Option<String>,
    pub hash: Option<String>,
}

/// Individual control mapping
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ControlMapping {
    pub control_id: String,
    pub control_title: Option<String>,
    pub control_description: Option<String>,
    pub implementation_status: String,
    pub customer_responsibility: Option<String>,
    pub csp_responsibility: Option<String>,
    pub shared_responsibility: Option<String>,
    pub implementation_guidance: Option<String>,
    pub assessment_procedures: Option<String>,
    pub notes: Option<String>,
    pub control_enhancements: Option<Vec<ControlEnhancement>>,
    pub source: ControlSource,
}

/// Control enhancement
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ControlEnhancement {
    pub enhancement_id: String,
    pub enhancement_title: Option<String>,
    pub implementation_status: String,
    pub notes: Option<String>,
}

/// Control source information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ControlSource {
    pub file: String,
    pub sheet: String,
    pub row: u32,
    pub col_range: Option<String>,
}

/// Document structures configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentStructures {
    pub metadata: DocumentMetadata,
    pub sections: Vec<DocumentSection>,
}

/// Document metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentMetadata {
    pub source_file: String,
    pub source_type: String,
    pub extraction_date: String,
    pub pandoc_version: Option<String>,
    pub hash: Option<String>,
}

/// Document section
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentSection {
    pub id: String,
    pub title: String,
    pub level: u32,
    pub text: String,
    pub tables: Option<Vec<DocumentTable>>,
    pub source: DocumentSectionSource,
}

/// Document table
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentTable {
    pub id: String,
    pub caption: Option<String>,
    pub headers: Vec<String>,
    pub rows: Vec<Vec<String>>,
    pub source: DocumentTableSource,
}

/// Document table source
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentTableSource {
    pub section_id: String,
    pub paragraph: Option<u32>,
    pub table_index: Option<u32>,
}

/// Document section source
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentSectionSource {
    pub file: String,
    pub heading_path: Vec<String>,
    pub paragraph_start: Option<u32>,
    pub paragraph_end: Option<u32>,
}

/// Configuration loader for mapping files
#[derive(Debug)]
pub struct MappingConfigurationLoader {
    /// Base directory for relative paths
    base_dir: PathBuf,
    /// Loaded configuration cache
    cached_config: Arc<RwLock<Option<MappingConfiguration>>>,
    /// Hot-reload watcher
    watcher: Option<notify::RecommendedWatcher>,
    /// Reload notification channel
    reload_tx: Option<mpsc::UnboundedSender<PathBuf>>,
}

/// Hot-reload event handler
pub struct HotReloadHandler {
    /// Configuration loader reference
    loader: Arc<RwLock<MappingConfigurationLoader>>,
    /// Reload notification receiver
    reload_rx: mpsc::UnboundedReceiver<PathBuf>,
}

impl MappingConfigurationLoader {
    /// Create a new configuration loader
    pub fn new<P: AsRef<Path>>(base_dir: P) -> Self {
        Self {
            base_dir: base_dir.as_ref().to_path_buf(),
            cached_config: Arc::new(RwLock::new(None)),
            watcher: None,
            reload_tx: None,
        }
    }

    /// Create a new configuration loader with hot-reload support
    pub fn with_hot_reload<P: AsRef<Path>>(base_dir: P) -> Result<(Self, HotReloadHandler)> {
        let (reload_tx, reload_rx) = mpsc::unbounded_channel();

        let mut loader = Self {
            base_dir: base_dir.as_ref().to_path_buf(),
            cached_config: Arc::new(RwLock::new(None)),
            watcher: None,
            reload_tx: Some(reload_tx),
        };

        // Set up file system watcher
        let tx = loader.reload_tx.as_ref().unwrap().clone();
        let base_dir_clone = loader.base_dir.clone();

        let mut watcher = notify::recommended_watcher(move |res: notify::Result<Event>| {
            match res {
                Ok(event) => {
                    if matches!(event.kind, EventKind::Modify(_) | EventKind::Create(_)) {
                        for path in event.paths {
                            if path.extension().map_or(false, |ext| ext == "json") {
                                if let Some(relative_path) = path.strip_prefix(&base_dir_clone).ok() {
                                    if let Err(e) = tx.send(relative_path.to_path_buf()) {
                                        error!("Failed to send reload notification: {}", e);
                                    }
                                }
                            }
                        }
                    }
                }
                Err(e) => error!("File watcher error: {}", e),
            }
        }).map_err(|e| Error::document_parsing(format!("Failed to create file watcher: {}", e)))?;

        // Watch the mappings and schema directories
        let mappings_dir = loader.base_dir.join("mappings");
        let schema_dir = loader.base_dir.join("schema");

        if mappings_dir.exists() {
            watcher.watch(&mappings_dir, RecursiveMode::NonRecursive)
                .map_err(|e| Error::document_parsing(format!("Failed to watch mappings directory: {}", e)))?;
        }

        if schema_dir.exists() {
            watcher.watch(&schema_dir, RecursiveMode::NonRecursive)
                .map_err(|e| Error::document_parsing(format!("Failed to watch schema directory: {}", e)))?;
        }

        loader.watcher = Some(watcher);

        let handler = HotReloadHandler {
            loader: Arc::new(RwLock::new(loader)),
            reload_rx,
        };

        // Return a clone of the loader for external use
        let loader_clone = {
            let loader_ref = handler.loader.read().unwrap();
            Self {
                base_dir: loader_ref.base_dir.clone(),
                cached_config: Arc::clone(&loader_ref.cached_config),
                watcher: None, // Don't clone the watcher
                reload_tx: None, // Don't clone the sender
            }
        };

        Ok((loader_clone, handler))
    }

    /// Load all mapping configurations from default file locations
    pub async fn load_all_configurations(&mut self) -> Result<MappingConfiguration> {
        info!("Loading all mapping configurations from {}", self.base_dir.display());

        let mut config = MappingConfiguration {
            inventory_mappings: None,
            poam_mappings: None,
            ssp_sections: None,
            controls: None,
            documents: None,
        };

        // Load inventory mappings
        if let Ok(inventory) = self.load_inventory_mappings().await {
            config.inventory_mappings = Some(inventory);
            info!("Successfully loaded inventory mappings");
        } else {
            warn!("Failed to load inventory mappings");
        }

        // Load POA&M mappings
        if let Ok(poam) = self.load_poam_mappings().await {
            config.poam_mappings = Some(poam);
            info!("Successfully loaded POA&M mappings");
        } else {
            warn!("Failed to load POA&M mappings");
        }

        // Load SSP sections
        if let Ok(ssp) = self.load_ssp_sections().await {
            config.ssp_sections = Some(ssp);
            info!("Successfully loaded SSP sections");
        } else {
            warn!("Failed to load SSP sections");
        }

        // Load control mappings
        if let Ok(controls) = self.load_control_mappings().await {
            config.controls = Some(controls);
            info!("Successfully loaded control mappings");
        } else {
            warn!("Failed to load control mappings");
        }

        // Load document structures
        if let Ok(documents) = self.load_document_structures().await {
            config.documents = Some(documents);
            info!("Successfully loaded document structures");
        } else {
            warn!("Failed to load document structures");
        }

        // Update cached configuration atomically
        {
            let mut cache = self.cached_config.write().unwrap();
            *cache = Some(config.clone());
        }

        Ok(config)
    }

    /// Load inventory mappings from JSON file
    pub async fn load_inventory_mappings(&self) -> Result<InventoryMappings> {
        let path = self.base_dir.join("mappings").join("inventory_mappings.json");
        let raw_json: serde_json::Value = self.load_json_file(&path).await?;

        // Check if it's wrapped in inventory_mappings or direct
        if let Some(inventory_mappings) = raw_json.get("inventory_mappings") {
            serde_json::from_value(inventory_mappings.clone())
                .map_err(|e| Error::document_parsing(format!("Failed to parse inventory mappings: {}", e)))
        } else {
            // Try to parse directly
            serde_json::from_value(raw_json)
                .map_err(|e| Error::document_parsing(format!("Failed to parse inventory mappings: {}", e)))
        }
    }

    /// Load POA&M mappings from JSON file
    pub async fn load_poam_mappings(&self) -> Result<PoamMappings> {
        let path = self.base_dir.join("mappings").join("poam_mappings.json");
        let raw_json: serde_json::Value = self.load_json_file(&path).await?;

        // Extract the poam_mappings section from the root object
        if let Some(poam_mappings) = raw_json.get("poam_mappings") {
            serde_json::from_value(poam_mappings.clone())
                .map_err(|e| Error::document_parsing(format!("Failed to parse POA&M mappings: {}", e)))
        } else {
            Err(Error::document_parsing("POA&M mappings section not found in JSON"))
        }
    }

    /// Load SSP sections from JSON file
    pub async fn load_ssp_sections(&self) -> Result<SspSections> {
        let path = self.base_dir.join("mappings").join("ssp_sections.json");
        self.load_json_file(&path).await
    }

    /// Load control mappings from schema file
    pub async fn load_control_mappings(&self) -> Result<ControlMappings> {
        let path = self.base_dir.join("schema").join("_controls.json");
        self.load_json_file(&path).await
    }

    /// Load document structures from schema file
    pub async fn load_document_structures(&self) -> Result<DocumentStructures> {
        let path = self.base_dir.join("schema").join("_document.json");
        self.load_json_file(&path).await
    }

    /// Generic JSON file loader with error handling
    async fn load_json_file<T>(&self, path: &Path) -> Result<T>
    where
        T: for<'de> Deserialize<'de>,
    {
        debug!("Loading JSON file: {}", path.display());

        // Check if file exists
        if !path.exists() {
            return Err(Error::document_parsing(format!(
                "Configuration file not found: {}",
                path.display()
            )));
        }

        // Read file contents
        let contents = fs::read_to_string(path).await.map_err(|e| {
            Error::document_parsing(format!(
                "Failed to read file {}: {}",
                path.display(),
                e
            ))
        })?;

        // Parse JSON
        serde_json::from_str(&contents).map_err(|e| {
            Error::document_parsing(format!(
                "Failed to parse JSON from {}: {}",
                path.display(),
                e
            ))
        })
    }

    /// Load configuration from custom file path
    pub async fn load_from_path<T, P>(&self, path: P) -> Result<T>
    where
        T: for<'de> Deserialize<'de>,
        P: AsRef<Path>,
    {
        let full_path = if path.as_ref().is_absolute() {
            path.as_ref().to_path_buf()
        } else {
            self.base_dir.join(path.as_ref())
        };

        self.load_json_file(&full_path).await
    }

    /// Get cached configuration if available
    pub fn get_cached_configuration(&self) -> Option<MappingConfiguration> {
        let cache = self.cached_config.read().unwrap();
        cache.clone()
    }

    /// Clear cached configuration
    pub fn clear_cache(&self) {
        let mut cache = self.cached_config.write().unwrap();
        *cache = None;
    }

    /// Validate loaded configuration
    pub fn validate_configuration(&self, config: &MappingConfiguration) -> Result<Vec<String>> {
        let mut warnings = Vec::new();

        // Validate inventory mappings
        if let Some(inventory) = &config.inventory_mappings {
            warnings.extend(self.validate_inventory_mappings(inventory)?);
        }

        // Validate POA&M mappings
        if let Some(poam) = &config.poam_mappings {
            warnings.extend(self.validate_poam_mappings(poam)?);
        }

        // Validate SSP sections
        if let Some(ssp) = &config.ssp_sections {
            warnings.extend(self.validate_ssp_sections(ssp)?);
        }

        // Validate control mappings
        if let Some(controls) = &config.controls {
            warnings.extend(self.validate_control_mappings(controls)?);
        }

        // Validate document structures
        if let Some(documents) = &config.documents {
            warnings.extend(self.validate_document_structures(documents)?);
        }

        // Check for configuration conflicts
        warnings.extend(self.detect_configuration_conflicts(config)?);

        Ok(warnings)
    }

    /// Validate inventory mappings
    fn validate_inventory_mappings(&self, inventory: &InventoryMappings) -> Result<Vec<String>> {
        let mut warnings = Vec::new();

        // Check required columns have valid field mappings
        for (key, mapping) in &inventory.fedramp_iiw_mappings.required_columns {
            if mapping.column_names.is_empty() {
                warnings.push(format!("Inventory mapping '{}' has no column names", key));
            }

            if mapping.field.is_empty() {
                warnings.push(format!("Inventory mapping '{}' has empty field name", key));
            }

            // Validate regex patterns if present
            if let Some(validation) = &mapping.validation {
                if validation == "ip_address" {
                    if let Some(pattern) = &inventory.validation_rules.ip_address_pattern {
                        if let Err(e) = Regex::new(pattern) {
                            warnings.push(format!("Invalid IP address regex pattern: {}", e));
                        }
                    }
                } else if validation == "mac_address" {
                    if let Some(pattern) = &inventory.validation_rules.mac_address_pattern {
                        if let Err(e) = Regex::new(pattern) {
                            warnings.push(format!("Invalid MAC address regex pattern: {}", e));
                        }
                    }
                }
            }
        }

        // Validate component type mappings
        for (key, mapping) in &inventory.component_type_mappings {
            if mapping.keywords.is_empty() {
                warnings.push(format!("Component type '{}' has no keywords", key));
            }
        }

        Ok(warnings)
    }

    /// Validate POA&M mappings
    fn validate_poam_mappings(&self, poam: &PoamMappings) -> Result<Vec<String>> {
        let mut warnings = Vec::new();

        // Check required columns
        for (key, mapping) in &poam.fedramp_v3_mappings.required_columns {
            if mapping.column_names.is_empty() {
                warnings.push(format!("POA&M mapping '{}' has no column names", key));
            }

            if mapping.oscal_field.is_empty() {
                warnings.push(format!("POA&M mapping '{}' has empty OSCAL field", key));
            }
        }

        // Validate control ID pattern
        if let Some(pattern) = &poam.fedramp_v3_mappings.validation_rules.control_id_pattern {
            if let Err(e) = Regex::new(pattern) {
                warnings.push(format!("Invalid control ID regex pattern: {}", e));
            }
        }

        // Validate milestone patterns
        for pattern in &poam.milestone_processing.patterns.milestone_format.patterns {
            if let Err(e) = Regex::new(pattern) {
                warnings.push(format!("Invalid milestone regex pattern: {}", e));
            }
        }

        Ok(warnings)
    }

    /// Validate SSP sections
    fn validate_ssp_sections(&self, ssp: &SspSections) -> Result<Vec<String>> {
        let mut warnings = Vec::new();

        // Validate section mappings
        for (key, mapping) in &ssp.section_mappings.mappings {
            if mapping.keywords.is_empty() {
                warnings.push(format!("SSP section '{}' has no keywords", key));
            }

            if mapping.target.is_empty() {
                warnings.push(format!("SSP section '{}' has empty target", key));
            }

            // Validate extract patterns
            if let Some(patterns) = &mapping.extract_patterns {
                for (pattern_key, pattern) in patterns {
                    if let Err(e) = Regex::new(pattern) {
                        warnings.push(format!(
                            "Invalid extract pattern '{}' in section '{}': {}",
                            pattern_key, key, e
                        ));
                    }
                }
            }
        }

        // Validate control extraction patterns
        for pattern in &ssp.control_extraction.patterns {
            if let Err(e) = Regex::new(&pattern.regex) {
                warnings.push(format!(
                    "Invalid control extraction pattern '{}': {}",
                    pattern.name, e
                ));
            }
        }

        Ok(warnings)
    }

    /// Validate control mappings
    fn validate_control_mappings(&self, controls: &ControlMappings) -> Result<Vec<String>> {
        let mut warnings = Vec::new();

        // Check metadata completeness
        if controls.metadata.source_file.is_empty() {
            warnings.push("Control mappings metadata missing source file".to_string());
        }

        if controls.metadata.framework.is_empty() {
            warnings.push("Control mappings metadata missing framework".to_string());
        }

        // Validate individual controls
        for control in &controls.controls {
            if control.control_id.is_empty() {
                warnings.push("Control mapping has empty control ID".to_string());
            }

            if control.implementation_status.is_empty() {
                warnings.push(format!(
                    "Control '{}' has empty implementation status",
                    control.control_id
                ));
            }

            // Validate control enhancements
            if let Some(enhancements) = &control.control_enhancements {
                for enhancement in enhancements {
                    if enhancement.enhancement_id.is_empty() {
                        warnings.push(format!(
                            "Control '{}' has enhancement with empty ID",
                            control.control_id
                        ));
                    }
                }
            }
        }

        Ok(warnings)
    }

    /// Validate document structures
    fn validate_document_structures(&self, documents: &DocumentStructures) -> Result<Vec<String>> {
        let mut warnings = Vec::new();

        // Check metadata
        if documents.metadata.source_file.is_empty() {
            warnings.push("Document structures metadata missing source file".to_string());
        }

        if documents.metadata.source_type.is_empty() {
            warnings.push("Document structures metadata missing source type".to_string());
        }

        // Validate sections
        for section in &documents.sections {
            if section.id.is_empty() {
                warnings.push("Document section has empty ID".to_string());
            }

            if section.title.is_empty() {
                warnings.push(format!("Document section '{}' has empty title", section.id));
            }

            // Validate tables
            if let Some(tables) = &section.tables {
                for table in tables {
                    if table.id.is_empty() {
                        warnings.push(format!(
                            "Table in section '{}' has empty ID",
                            section.id
                        ));
                    }

                    if table.headers.is_empty() {
                        warnings.push(format!(
                            "Table '{}' in section '{}' has no headers",
                            table.id, section.id
                        ));
                    }
                }
            }
        }

        Ok(warnings)
    }

    /// Detect configuration conflicts between different mapping files
    fn detect_configuration_conflicts(&self, config: &MappingConfiguration) -> Result<Vec<String>> {
        let mut conflicts = Vec::new();

        // Check for overlapping field mappings between inventory and POA&M
        if let (Some(inventory), Some(poam)) = (&config.inventory_mappings, &config.poam_mappings) {
            let inventory_fields: std::collections::HashSet<_> = inventory
                .fedramp_iiw_mappings
                .required_columns
                .values()
                .map(|m| &m.field)
                .collect();

            let poam_fields: std::collections::HashSet<_> = poam
                .fedramp_v3_mappings
                .required_columns
                .values()
                .map(|m| &m.oscal_field)
                .collect();

            for field in inventory_fields.intersection(&poam_fields) {
                conflicts.push(format!(
                    "Field '{}' is mapped in both inventory and POA&M configurations",
                    field
                ));
            }
        }

        // Check for duplicate control IDs in control mappings
        if let Some(controls) = &config.controls {
            let mut control_ids = std::collections::HashSet::new();
            for control in &controls.controls {
                if !control_ids.insert(&control.control_id) {
                    conflicts.push(format!(
                        "Duplicate control ID '{}' found in control mappings",
                        control.control_id
                    ));
                }
            }
        }

        Ok(conflicts)
    }
}

impl HotReloadHandler {
    /// Start the hot-reload handler
    pub async fn start(mut self) -> Result<()> {
        info!("Starting hot-reload handler for mapping configurations");

        while let Some(changed_path) = self.reload_rx.recv().await {
            info!("Configuration file changed: {}", changed_path.display());

            // Debounce rapid file changes
            tokio::time::sleep(Duration::from_millis(100)).await;

            // Reload configuration
            if let Err(e) = self.reload_configuration(&changed_path).await {
                error!("Failed to reload configuration after file change: {}", e);
            }
        }

        Ok(())
    }

    /// Reload configuration after file change
    async fn reload_configuration(&self, changed_path: &Path) -> Result<()> {
        let loader = {
            let loader_ref = self.loader.read().unwrap();
            MappingConfigurationLoader {
                base_dir: loader_ref.base_dir.clone(),
                cached_config: Arc::clone(&loader_ref.cached_config),
                watcher: None,
                reload_tx: None,
            }
        };

        // Determine which configuration to reload based on the changed file
        match changed_path.file_name().and_then(|n| n.to_str()) {
            Some("inventory_mappings.json") => {
                info!("Reloading inventory mappings");
                if let Ok(inventory) = loader.load_inventory_mappings().await {
                    // Update only the inventory part of the cached configuration
                    self.update_cached_inventory(inventory).await?;
                }
            }
            Some("poam_mappings.json") => {
                info!("Reloading POA&M mappings");
                if let Ok(poam) = loader.load_poam_mappings().await {
                    self.update_cached_poam(poam).await?;
                }
            }
            Some("ssp_sections.json") => {
                info!("Reloading SSP sections");
                if let Ok(ssp) = loader.load_ssp_sections().await {
                    self.update_cached_ssp(ssp).await?;
                }
            }
            Some("_controls.json") => {
                info!("Reloading control mappings");
                if let Ok(controls) = loader.load_control_mappings().await {
                    self.update_cached_controls(controls).await?;
                }
            }
            Some("_document.json") => {
                info!("Reloading document structures");
                if let Ok(documents) = loader.load_document_structures().await {
                    self.update_cached_documents(documents).await?;
                }
            }
            _ => {
                warn!("Unknown configuration file changed: {}", changed_path.display());
            }
        }

        Ok(())
    }

    /// Update cached inventory mappings
    async fn update_cached_inventory(&self, inventory: InventoryMappings) -> Result<()> {
        let loader = self.loader.read().unwrap();
        let mut cache = loader.cached_config.write().unwrap();
        if let Some(config) = cache.as_mut() {
            config.inventory_mappings = Some(inventory);
        }
        Ok(())
    }

    /// Update cached POA&M mappings
    async fn update_cached_poam(&self, poam: PoamMappings) -> Result<()> {
        let loader = self.loader.read().unwrap();
        let mut cache = loader.cached_config.write().unwrap();
        if let Some(config) = cache.as_mut() {
            config.poam_mappings = Some(poam);
        }
        Ok(())
    }

    /// Update cached SSP sections
    async fn update_cached_ssp(&self, ssp: SspSections) -> Result<()> {
        let loader = self.loader.read().unwrap();
        let mut cache = loader.cached_config.write().unwrap();
        if let Some(config) = cache.as_mut() {
            config.ssp_sections = Some(ssp);
        }
        Ok(())
    }

    /// Update cached control mappings
    async fn update_cached_controls(&self, controls: ControlMappings) -> Result<()> {
        let loader = self.loader.read().unwrap();
        let mut cache = loader.cached_config.write().unwrap();
        if let Some(config) = cache.as_mut() {
            config.controls = Some(controls);
        }
        Ok(())
    }

    /// Update cached document structures
    async fn update_cached_documents(&self, documents: DocumentStructures) -> Result<()> {
        let loader = self.loader.read().unwrap();
        let mut cache = loader.cached_config.write().unwrap();
        if let Some(config) = cache.as_mut() {
            config.documents = Some(documents);
        }
        Ok(())
    }
}

/// Optimized lookup structures for fast column mapping
pub struct OptimizedMappingLookup {
    /// Exact match lookup for column names to target fields
    exact_matches: HashMap<String, MappingEntry>,
    /// Normalized column names for fuzzy matching
    fuzzy_candidates: Vec<FuzzyCandidate>,
    /// Validation rules lookup
    validation_rules: HashMap<String, ValidationRule>,
    /// Required fields tracking
    required_fields: std::collections::HashSet<String>,
    /// Advanced fuzzy matcher
    fuzzy_matcher: FuzzyMatcher,
    /// Target strings for fuzzy matching
    fuzzy_targets: Vec<String>,
}

/// Mapping entry for lookup results
#[derive(Debug, Clone)]
pub struct MappingEntry {
    pub target_field: String,
    pub source_type: MappingSourceType,
    pub required: bool,
    pub validation: Option<String>,
    pub data_type: Option<String>,
}

/// Source type for mapping entries
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MappingSourceType {
    Inventory,
    Poam,
    SspSection,
    Control,
    Document,
}

/// Fuzzy matching candidate
#[derive(Debug, Clone)]
pub struct FuzzyCandidate {
    pub original_name: String,
    pub normalized_name: String,
    pub target_field: String,
    pub source_type: MappingSourceType,
    pub required: bool,
}

/// Validation rule for field validation
#[derive(Debug, Clone)]
pub struct ValidationRule {
    pub rule_type: ValidationType,
    pub pattern: Option<Regex>,
    pub allowed_values: Option<Vec<String>>,
}

/// Validation type enumeration
#[derive(Debug, Clone)]
pub enum ValidationType {
    Regex,
    AllowedValues,
    Boolean,
    IpAddress,
    MacAddress,
    Date,
    ControlId,
    UniqueIdentifier,
}

impl OptimizedMappingLookup {
    /// Create optimized lookup structures from mapping configuration
    pub fn from_configuration(config: &MappingConfiguration) -> Result<Self> {
        let mut exact_matches = HashMap::new();
        let mut fuzzy_candidates = Vec::new();
        let mut validation_rules = HashMap::new();
        let mut required_fields = std::collections::HashSet::new();

        // Process inventory mappings
        if let Some(inventory) = &config.inventory_mappings {
            Self::process_inventory_mappings(
                inventory,
                &mut exact_matches,
                &mut fuzzy_candidates,
                &mut validation_rules,
                &mut required_fields,
            )?;
        }

        // Process POA&M mappings
        if let Some(poam) = &config.poam_mappings {
            Self::process_poam_mappings(
                poam,
                &mut exact_matches,
                &mut fuzzy_candidates,
                &mut validation_rules,
                &mut required_fields,
            )?;
        }

        // Process SSP sections
        if let Some(ssp) = &config.ssp_sections {
            Self::process_ssp_mappings(
                ssp,
                &mut exact_matches,
                &mut fuzzy_candidates,
                &mut validation_rules,
                &mut required_fields,
            )?;
        }

        // Create fuzzy targets list for the advanced fuzzy matcher
        let fuzzy_targets: Vec<String> = fuzzy_candidates
            .iter()
            .map(|candidate| candidate.original_name.clone())
            .collect();

        Ok(Self {
            exact_matches,
            fuzzy_candidates,
            validation_rules,
            required_fields,
            fuzzy_matcher: FuzzyMatcher::for_fedramp_columns(),
            fuzzy_targets,
        })
    }

    /// Process inventory mappings into lookup structures
    fn process_inventory_mappings(
        inventory: &InventoryMappings,
        exact_matches: &mut HashMap<String, MappingEntry>,
        fuzzy_candidates: &mut Vec<FuzzyCandidate>,
        validation_rules: &mut HashMap<String, ValidationRule>,
        required_fields: &mut std::collections::HashSet<String>,
    ) -> Result<()> {
        for (_key, mapping) in &inventory.fedramp_iiw_mappings.required_columns {
            let entry = MappingEntry {
                target_field: mapping.field.clone(),
                source_type: MappingSourceType::Inventory,
                required: mapping.required,
                validation: mapping.validation.clone(),
                data_type: None,
            };

            if mapping.required {
                required_fields.insert(mapping.field.clone());
            }

            // Add exact matches
            for column_name in &mapping.column_names {
                let normalized = Self::normalize_column_name(column_name);
                exact_matches.insert(normalized.clone(), entry.clone());

                // Add fuzzy candidate
                fuzzy_candidates.push(FuzzyCandidate {
                    original_name: column_name.clone(),
                    normalized_name: normalized,
                    target_field: mapping.field.clone(),
                    source_type: MappingSourceType::Inventory,
                    required: mapping.required,
                });
            }

            // Add validation rules
            if let Some(validation) = &mapping.validation {
                let rule = Self::create_validation_rule(validation, &inventory.validation_rules)?;
                validation_rules.insert(mapping.field.clone(), rule);
            }
        }

        Ok(())
    }

    /// Process POA&M mappings into lookup structures
    fn process_poam_mappings(
        poam: &PoamMappings,
        exact_matches: &mut HashMap<String, MappingEntry>,
        fuzzy_candidates: &mut Vec<FuzzyCandidate>,
        validation_rules: &mut HashMap<String, ValidationRule>,
        required_fields: &mut std::collections::HashSet<String>,
    ) -> Result<()> {
        for (_key, mapping) in &poam.fedramp_v3_mappings.required_columns {
            let entry = MappingEntry {
                target_field: mapping.oscal_field.clone(),
                source_type: MappingSourceType::Poam,
                required: mapping.required,
                validation: mapping.validation.clone(),
                data_type: None,
            };

            if mapping.required {
                required_fields.insert(mapping.oscal_field.clone());
            }

            // Add exact matches
            for column_name in &mapping.column_names {
                let normalized = Self::normalize_column_name(column_name);
                exact_matches.insert(normalized.clone(), entry.clone());

                // Add fuzzy candidate
                fuzzy_candidates.push(FuzzyCandidate {
                    original_name: column_name.clone(),
                    normalized_name: normalized,
                    target_field: mapping.oscal_field.clone(),
                    source_type: MappingSourceType::Poam,
                    required: mapping.required,
                });
            }

            // Add validation rules
            if let Some(validation) = &mapping.validation {
                let rule = Self::create_poam_validation_rule(validation, &poam.fedramp_v3_mappings.validation_rules)?;
                validation_rules.insert(mapping.oscal_field.clone(), rule);
            }
        }

        Ok(())
    }

    /// Process SSP mappings into lookup structures
    fn process_ssp_mappings(
        ssp: &SspSections,
        exact_matches: &mut HashMap<String, MappingEntry>,
        fuzzy_candidates: &mut Vec<FuzzyCandidate>,
        _validation_rules: &mut HashMap<String, ValidationRule>,
        required_fields: &mut std::collections::HashSet<String>,
    ) -> Result<()> {
        for (_key, mapping) in &ssp.section_mappings.mappings {
            let entry = MappingEntry {
                target_field: mapping.target.clone(),
                source_type: MappingSourceType::SspSection,
                required: mapping.required,
                validation: None,
                data_type: None,
            };

            if mapping.required {
                required_fields.insert(mapping.target.clone());
            }

            // Add keywords as fuzzy candidates
            for keyword in &mapping.keywords {
                let normalized = Self::normalize_column_name(keyword);

                fuzzy_candidates.push(FuzzyCandidate {
                    original_name: keyword.clone(),
                    normalized_name: normalized.clone(),
                    target_field: mapping.target.clone(),
                    source_type: MappingSourceType::SspSection,
                    required: mapping.required,
                });

                // Also add as exact match for efficiency
                exact_matches.insert(normalized, entry.clone());
            }
        }

        Ok(())
    }

    /// Normalize column name for consistent matching
    fn normalize_column_name(name: &str) -> String {
        name.to_lowercase()
            .chars()
            .filter(|c| c.is_alphanumeric())
            .collect()
    }

    /// Create validation rule from inventory validation type
    fn create_validation_rule(
        validation: &str,
        rules: &ValidationRules,
    ) -> Result<ValidationRule> {
        match validation {
            "ip_address" => {
                if let Some(pattern) = &rules.ip_address_pattern {
                    Ok(ValidationRule {
                        rule_type: ValidationType::IpAddress,
                        pattern: Some(Regex::new(pattern).map_err(|e| {
                            Error::document_parsing(format!("Invalid IP address pattern: {}", e))
                        })?),
                        allowed_values: None,
                    })
                } else {
                    Ok(ValidationRule {
                        rule_type: ValidationType::IpAddress,
                        pattern: None,
                        allowed_values: None,
                    })
                }
            }
            "mac_address" => {
                if let Some(pattern) = &rules.mac_address_pattern {
                    Ok(ValidationRule {
                        rule_type: ValidationType::MacAddress,
                        pattern: Some(Regex::new(pattern).map_err(|e| {
                            Error::document_parsing(format!("Invalid MAC address pattern: {}", e))
                        })?),
                        allowed_values: None,
                    })
                } else {
                    Ok(ValidationRule {
                        rule_type: ValidationType::MacAddress,
                        pattern: None,
                        allowed_values: None,
                    })
                }
            }
            "boolean" => Ok(ValidationRule {
                rule_type: ValidationType::Boolean,
                pattern: None,
                allowed_values: rules.boolean_values.clone(),
            }),
            "asset_types" => Ok(ValidationRule {
                rule_type: ValidationType::AllowedValues,
                pattern: None,
                allowed_values: rules.asset_types.clone(),
            }),
            "environments" => Ok(ValidationRule {
                rule_type: ValidationType::AllowedValues,
                pattern: None,
                allowed_values: rules.environments.clone(),
            }),
            "criticality_levels" => Ok(ValidationRule {
                rule_type: ValidationType::AllowedValues,
                pattern: None,
                allowed_values: rules.criticality_levels.clone(),
            }),
            "unique_identifier" => Ok(ValidationRule {
                rule_type: ValidationType::UniqueIdentifier,
                pattern: None,
                allowed_values: None,
            }),
            _ => Ok(ValidationRule {
                rule_type: ValidationType::AllowedValues,
                pattern: None,
                allowed_values: None,
            }),
        }
    }

    /// Create validation rule from POA&M validation type
    fn create_poam_validation_rule(
        validation: &str,
        rules: &PoamValidationRules,
    ) -> Result<ValidationRule> {
        match validation {
            "severity_levels" => Ok(ValidationRule {
                rule_type: ValidationType::AllowedValues,
                pattern: None,
                allowed_values: rules.severity_levels.clone(),
            }),
            "status_values" => Ok(ValidationRule {
                rule_type: ValidationType::AllowedValues,
                pattern: None,
                allowed_values: rules.status_values.clone(),
            }),
            "control_id_list" => {
                if let Some(pattern) = &rules.control_id_pattern {
                    Ok(ValidationRule {
                        rule_type: ValidationType::ControlId,
                        pattern: Some(Regex::new(pattern).map_err(|e| {
                            Error::document_parsing(format!("Invalid control ID pattern: {}", e))
                        })?),
                        allowed_values: None,
                    })
                } else {
                    Ok(ValidationRule {
                        rule_type: ValidationType::ControlId,
                        pattern: None,
                        allowed_values: None,
                    })
                }
            }
            "date" => Ok(ValidationRule {
                rule_type: ValidationType::Date,
                pattern: None,
                allowed_values: rules.date_formats.clone(),
            }),
            "alphanumeric" => Ok(ValidationRule {
                rule_type: ValidationType::Regex,
                pattern: Some(Regex::new(r"^[a-zA-Z0-9]+$").unwrap()),
                allowed_values: None,
            }),
            _ => Ok(ValidationRule {
                rule_type: ValidationType::AllowedValues,
                pattern: None,
                allowed_values: None,
            }),
        }
    }

    /// Find exact match for column name
    pub fn find_exact_match(&self, column_name: &str) -> Option<&MappingEntry> {
        let normalized = Self::normalize_column_name(column_name);
        self.exact_matches.get(&normalized)
    }

    /// Find best fuzzy match for column name using advanced fuzzy matching
    pub fn find_fuzzy_match(&mut self, column_name: &str, min_confidence: f64) -> Option<MappingResult> {
        // Use the advanced fuzzy matcher
        let matches = self.fuzzy_matcher.find_matches(column_name, &self.fuzzy_targets);

        if let Some(best_match) = matches.first() {
            if best_match.confidence >= min_confidence {
                // Find the corresponding candidate to get the target field
                if let Some(candidate) = self.fuzzy_candidates
                    .iter()
                    .find(|c| c.original_name == best_match.target) {

                    return Some(MappingResult {
                        source_column: column_name.to_string(),
                        target_field: candidate.target_field.clone(),
                        confidence: best_match.confidence,
                        exact_match: best_match.exact_match,
                    });
                }
            }
        }

        // Fallback to legacy fuzzy matching for backward compatibility
        self.find_fuzzy_match_legacy(column_name, min_confidence)
    }

    /// Legacy fuzzy matching implementation for backward compatibility
    fn find_fuzzy_match_legacy(&self, column_name: &str, min_confidence: f64) -> Option<MappingResult> {
        let normalized = Self::normalize_column_name(column_name);
        let mut best_match: Option<&FuzzyCandidate> = None;
        let mut best_confidence = 0.0;

        for candidate in &self.fuzzy_candidates {
            let confidence = self.calculate_similarity(&normalized, &candidate.normalized_name);
            if confidence >= min_confidence && confidence > best_confidence {
                best_confidence = confidence;
                best_match = Some(candidate);
            }
        }

        best_match.map(|candidate| MappingResult {
            source_column: column_name.to_string(),
            target_field: candidate.target_field.clone(),
            confidence: best_confidence,
            exact_match: false,
        })
    }

    /// Calculate similarity between two normalized strings
    fn calculate_similarity(&self, s1: &str, s2: &str) -> f64 {
        if s1 == s2 {
            return 1.0;
        }

        let len1 = s1.len();
        let len2 = s2.len();

        if len1 == 0 || len2 == 0 {
            return 0.0;
        }

        let distance = self.levenshtein_distance(s1, s2);
        let max_len = len1.max(len2);

        1.0 - (distance as f64 / max_len as f64)
    }

    /// Calculate Levenshtein distance between two strings
    fn levenshtein_distance(&self, s1: &str, s2: &str) -> usize {
        let chars1: Vec<char> = s1.chars().collect();
        let chars2: Vec<char> = s2.chars().collect();
        let len1 = chars1.len();
        let len2 = chars2.len();

        let mut matrix = vec![vec![0; len2 + 1]; len1 + 1];

        // Initialize first row and column
        for i in 0..=len1 {
            matrix[i][0] = i;
        }
        for j in 0..=len2 {
            matrix[0][j] = j;
        }

        // Fill the matrix
        for i in 1..=len1 {
            for j in 1..=len2 {
                let cost = if chars1[i - 1] == chars2[j - 1] { 0 } else { 1 };
                matrix[i][j] = (matrix[i - 1][j] + 1)
                    .min(matrix[i][j - 1] + 1)
                    .min(matrix[i - 1][j - 1] + cost);
            }
        }

        matrix[len1][len2]
    }

    /// Get validation rule for a field
    pub fn get_validation_rule(&self, field_name: &str) -> Option<&ValidationRule> {
        self.validation_rules.get(field_name)
    }

    /// Check if field is required
    pub fn is_required_field(&self, field_name: &str) -> bool {
        self.required_fields.contains(field_name)
    }

    /// Get all required fields
    pub fn get_required_fields(&self) -> &std::collections::HashSet<String> {
        &self.required_fields
    }

    /// Get statistics about the lookup structures
    pub fn get_statistics(&self) -> serde_json::Value {
        let source_type_counts = self.fuzzy_candidates
            .iter()
            .fold(HashMap::new(), |mut acc, candidate| {
                *acc.entry(format!("{:?}", candidate.source_type)).or_insert(0) += 1;
                acc
            });

        let (cache_size, cache_capacity) = self.fuzzy_matcher.cache_stats();

        serde_json::json!({
            "exact_matches": self.exact_matches.len(),
            "fuzzy_candidates": self.fuzzy_candidates.len(),
            "validation_rules": self.validation_rules.len(),
            "required_fields": self.required_fields.len(),
            "source_type_distribution": source_type_counts,
            "fuzzy_cache_size": cache_size,
            "fuzzy_cache_capacity": cache_capacity
        })
    }

    /// Update fuzzy matching configuration
    pub fn update_fuzzy_config(&mut self, config: FuzzyMatchConfig) {
        self.fuzzy_matcher.update_config(config);
    }

    /// Clear fuzzy matching cache
    pub fn clear_fuzzy_cache(&mut self) {
        self.fuzzy_matcher.clear_cache();
    }

    /// Get detailed fuzzy match results with algorithm breakdown
    pub fn get_detailed_fuzzy_matches(&mut self, column_name: &str, min_confidence: f64) -> Vec<FuzzyMatchResult> {
        self.fuzzy_matcher.find_matches(column_name, &self.fuzzy_targets)
            .into_iter()
            .filter(|result| result.confidence >= min_confidence)
            .collect()
    }
}

/// Column mapping result with confidence score
#[derive(Debug, Clone)]
pub struct MappingResult {
    /// Source column name that was matched
    pub source_column: String,
    /// Target OSCAL field name
    pub target_field: String,
    /// Confidence score (0.0 to 1.0)
    pub confidence: f64,
    /// Whether this was an exact match
    pub exact_match: bool,
}

/// Column mapper for detecting and mapping document columns
pub struct ColumnMapper {
    /// Legacy mapping configurations (for backward compatibility)
    mappings: HashMap<String, ColumnMapping>,
    /// Optimized lookup structures
    optimized_lookup: Option<OptimizedMappingLookup>,
    /// Minimum confidence threshold for fuzzy matching
    min_confidence: f64,
    /// Configuration loader
    config_loader: Option<MappingConfigurationLoader>,
}

impl ColumnMapper {
    /// Create a new column mapper
    #[must_use]
    pub fn new() -> Self {
        Self {
            mappings: HashMap::new(),
            optimized_lookup: None,
            min_confidence: 0.7,
            config_loader: None,
        }
    }

    /// Create a new column mapper with custom confidence threshold
    #[must_use]
    pub fn with_confidence_threshold(min_confidence: f64) -> Self {
        Self {
            mappings: HashMap::new(),
            optimized_lookup: None,
            min_confidence,
            config_loader: None,
        }
    }

    /// Create a new column mapper with configuration loader
    pub fn with_config_loader<P: AsRef<Path>>(base_dir: P) -> Self {
        Self {
            mappings: HashMap::new(),
            optimized_lookup: None,
            min_confidence: 0.7,
            config_loader: Some(MappingConfigurationLoader::new(base_dir)),
        }
    }

    /// Load mapping configurations from JSON files
    pub async fn load_configurations(&mut self) -> Result<()> {
        if let Some(loader) = &mut self.config_loader {
            let config = loader.load_all_configurations().await?;

            // Validate configuration
            let warnings = loader.validate_configuration(&config)?;
            if !warnings.is_empty() {
                warn!("Configuration validation warnings: {:?}", warnings);
            }

            // Create optimized lookup structures
            self.optimized_lookup = Some(OptimizedMappingLookup::from_configuration(&config)?);

            info!("Successfully loaded and optimized mapping configurations");
            Ok(())
        } else {
            Err(Error::document_parsing("No configuration loader available"))
        }
    }

    /// Load specific mapping configuration from file
    pub async fn load_configuration_from_file<P: AsRef<Path>>(&mut self, path: P) -> Result<()> {
        if self.config_loader.is_none() {
            self.config_loader = Some(MappingConfigurationLoader::new("."));
        }

        if let Some(loader) = &mut self.config_loader {
            let config: MappingConfiguration = loader.load_from_path(path).await?;
            self.optimized_lookup = Some(OptimizedMappingLookup::from_configuration(&config)?);
            info!("Successfully loaded configuration from file");
            Ok(())
        } else {
            Err(Error::document_parsing("Failed to initialize configuration loader"))
        }
    }

    /// Load mapping configuration from JSON
    pub fn load_mappings(&mut self, mappings: HashMap<String, ColumnMapping>) -> Result<()> {
        info!("Loading {} column mappings", mappings.len());
        self.mappings = mappings;
        Ok(())
    }

    /// Map column headers to OSCAL fields
    pub fn map_columns(&mut self, headers: &[String]) -> Result<Vec<MappingResult>> {
        debug!("Mapping {} column headers", headers.len());

        let mut results = Vec::new();

        for header in headers {
            if let Some(mapping_result) = self.find_best_match(header)? {
                results.push(mapping_result);
            } else {
                warn!("No mapping found for column: {}", header);
            }
        }

        info!("Successfully mapped {}/{} columns", results.len(), headers.len());
        Ok(results)
    }

    /// Find the best mapping match for a column header
    fn find_best_match(&mut self, header: &str) -> Result<Option<MappingResult>> {
        // Try optimized lookup first if available
        if let Some(lookup) = &mut self.optimized_lookup {
            // Check for exact match
            if let Some(entry) = lookup.find_exact_match(header) {
                return Ok(Some(MappingResult {
                    source_column: header.to_string(),
                    target_field: entry.target_field.clone(),
                    confidence: 1.0,
                    exact_match: true,
                }));
            }

            // Try fuzzy match
            if let Some(fuzzy_result) = lookup.find_fuzzy_match(header, self.min_confidence) {
                return Ok(Some(fuzzy_result));
            }

            return Ok(None);
        }

        // Fall back to legacy mapping logic for backward compatibility
        self.find_best_match_legacy(header)
    }

    /// Legacy mapping logic for backward compatibility
    fn find_best_match_legacy(&self, header: &str) -> Result<Option<MappingResult>> {
        let header_normalized = self.normalize_string(header);
        let mut best_match: Option<MappingResult> = None;
        let mut best_confidence = 0.0;

        for (target_field, mapping) in &self.mappings {
            for source_column in &mapping.source_columns {
                let source_normalized = self.normalize_string(source_column);

                // Check for exact match first
                if header_normalized == source_normalized {
                    return Ok(Some(MappingResult {
                        source_column: header.to_string(),
                        target_field: target_field.clone(),
                        confidence: 1.0,
                        exact_match: true,
                    }));
                }

                // Calculate fuzzy match confidence
                let confidence = self.calculate_similarity(&header_normalized, &source_normalized);

                if confidence >= self.min_confidence && confidence > best_confidence {
                    best_confidence = confidence;
                    best_match = Some(MappingResult {
                        source_column: header.to_string(),
                        target_field: target_field.clone(),
                        confidence,
                        exact_match: false,
                    });
                }
            }
        }

        Ok(best_match)
    }

    /// Normalize string for comparison
    fn normalize_string(&self, s: &str) -> String {
        s.to_lowercase()
            .chars()
            .filter(|c| c.is_alphanumeric())
            .collect()
    }

    /// Calculate similarity between two strings using Levenshtein distance
    fn calculate_similarity(&self, s1: &str, s2: &str) -> f64 {
        if s1 == s2 {
            return 1.0;
        }

        let len1 = s1.len();
        let len2 = s2.len();

        if len1 == 0 || len2 == 0 {
            return 0.0;
        }

        let distance = self.levenshtein_distance(s1, s2);
        let max_len = len1.max(len2);
        
        1.0 - (distance as f64 / max_len as f64)
    }

    /// Calculate Levenshtein distance between two strings
    fn levenshtein_distance(&self, s1: &str, s2: &str) -> usize {
        let chars1: Vec<char> = s1.chars().collect();
        let chars2: Vec<char> = s2.chars().collect();
        let len1 = chars1.len();
        let len2 = chars2.len();

        let mut matrix = vec![vec![0; len2 + 1]; len1 + 1];

        // Initialize first row and column
        for i in 0..=len1 {
            matrix[i][0] = i;
        }
        for j in 0..=len2 {
            matrix[0][j] = j;
        }

        // Fill the matrix
        for i in 1..=len1 {
            for j in 1..=len2 {
                let cost = if chars1[i - 1] == chars2[j - 1] { 0 } else { 1 };
                matrix[i][j] = (matrix[i - 1][j] + 1)
                    .min(matrix[i][j - 1] + 1)
                    .min(matrix[i - 1][j - 1] + cost);
            }
        }

        matrix[len1][len2]
    }

    /// Validate that all required fields have mappings
    pub fn validate_required_mappings(&self, mapping_results: &[MappingResult]) -> Result<Vec<String>> {
        let mut errors = Vec::new();
        let mapped_fields: std::collections::HashSet<_> =
            mapping_results.iter().map(|r| &r.target_field).collect();

        // Use optimized lookup if available
        if let Some(lookup) = &self.optimized_lookup {
            for required_field in lookup.get_required_fields() {
                if !mapped_fields.contains(required_field) {
                    errors.push(format!("Required field '{}' not found in document", required_field));
                }
            }
        } else {
            // Fall back to legacy validation
            for (target_field, mapping) in &self.mappings {
                if mapping.required && !mapped_fields.contains(target_field) {
                    errors.push(format!("Required field '{}' not found in document", target_field));
                }
            }
        }

        Ok(errors)
    }

    /// Validate field value using configured validation rules
    pub fn validate_field_value(&self, field_name: &str, value: &str) -> Result<Vec<String>> {
        let mut errors = Vec::new();

        if let Some(lookup) = &self.optimized_lookup {
            if let Some(rule) = lookup.get_validation_rule(field_name) {
                match &rule.rule_type {
                    ValidationType::Regex => {
                        if let Some(pattern) = &rule.pattern {
                            if !pattern.is_match(value) {
                                errors.push(format!("Field '{}' value '{}' does not match required pattern", field_name, value));
                            }
                        }
                    }
                    ValidationType::AllowedValues => {
                        if let Some(allowed) = &rule.allowed_values {
                            if !allowed.contains(&value.to_string()) {
                                errors.push(format!("Field '{}' value '{}' is not in allowed values: {:?}", field_name, value, allowed));
                            }
                        }
                    }
                    ValidationType::Boolean => {
                        let normalized = value.to_lowercase();
                        if !["true", "false", "yes", "no", "y", "n", "1", "0"].contains(&normalized.as_str()) {
                            errors.push(format!("Field '{}' value '{}' is not a valid boolean", field_name, value));
                        }
                    }
                    ValidationType::IpAddress => {
                        if let Some(pattern) = &rule.pattern {
                            if !pattern.is_match(value) {
                                errors.push(format!("Field '{}' value '{}' is not a valid IP address", field_name, value));
                            }
                        }
                    }
                    ValidationType::MacAddress => {
                        if let Some(pattern) = &rule.pattern {
                            if !pattern.is_match(value) {
                                errors.push(format!("Field '{}' value '{}' is not a valid MAC address", field_name, value));
                            }
                        }
                    }
                    ValidationType::Date => {
                        // Basic date validation - could be enhanced with chrono parsing
                        if value.is_empty() {
                            errors.push(format!("Field '{}' date value cannot be empty", field_name));
                        }
                    }
                    ValidationType::ControlId => {
                        if let Some(pattern) = &rule.pattern {
                            if !pattern.is_match(value) {
                                errors.push(format!("Field '{}' value '{}' is not a valid control ID", field_name, value));
                            }
                        }
                    }
                    ValidationType::UniqueIdentifier => {
                        if value.is_empty() {
                            errors.push(format!("Field '{}' unique identifier cannot be empty", field_name));
                        }
                    }
                }
            }
        }

        Ok(errors)
    }

    /// Get mapping statistics
    pub fn get_mapping_statistics(&self) -> serde_json::Value {
        if let Some(lookup) = &self.optimized_lookup {
            lookup.get_statistics()
        } else {
            serde_json::json!({
                "legacy_mappings": self.mappings.len(),
                "optimized_lookup": false
            })
        }
    }

    /// Generate mapping confidence report
    pub fn generate_mapping_report(&self, mapping_results: &[MappingResult]) -> serde_json::Value {
        let total_mappings = mapping_results.len();
        let exact_matches = mapping_results.iter().filter(|r| r.exact_match).count();
        let avg_confidence = if total_mappings > 0 {
            mapping_results.iter().map(|r| r.confidence).sum::<f64>() / total_mappings as f64
        } else {
            0.0
        };

        serde_json::json!({
            "total_mappings": total_mappings,
            "exact_matches": exact_matches,
            "fuzzy_matches": total_mappings - exact_matches,
            "average_confidence": avg_confidence,
            "mappings": mapping_results.iter().map(|r| serde_json::json!({
                "source_column": r.source_column,
                "target_field": r.target_field,
                "confidence": r.confidence,
                "exact_match": r.exact_match
            })).collect::<Vec<_>>()
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;
    use tokio::time::Instant;

    /// Create a temporary directory with test mapping files
    async fn create_test_mappings_dir() -> Result<TempDir> {
        let temp_dir = TempDir::new().unwrap();
        let mappings_dir = temp_dir.path().join("mappings");
        let schema_dir = temp_dir.path().join("schema");

        fs::create_dir_all(&mappings_dir).unwrap();
        fs::create_dir_all(&schema_dir).unwrap();

        // Create test inventory mappings (matching the actual structure)
        let inventory_json = serde_json::json!({
            "description": "Test inventory mappings",
            "version": "1.0",
            "fedramp_iiw_mappings": {
                "required_columns": {
                    "asset_id": {
                        "column_names": ["Asset ID", "Component ID"],
                        "field": "uuid",
                        "required": true,
                        "validation": "unique_identifier"
                    }
                }
            },
            "validation_rules": {
                "asset_types": ["hardware", "software"],
                "boolean_values": ["yes", "no"]
            },
            "component_grouping": {
                "strategies": {}
            },
            "component_type_mappings": {},
            "security_mappings": {
                "criticality_to_impact": {},
                "risk_factors": {}
            },
            "control_inheritance": {
                "infrastructure_controls": [],
                "platform_controls": [],
                "inheritance_mappings": {}
            }
        });

        fs::write(
            mappings_dir.join("inventory_mappings.json"),
            serde_json::to_string_pretty(&inventory_json).unwrap(),
        ).unwrap();

        // Create test POA&M mappings
        let poam_json = serde_json::json!({
            "poam_mappings": {
                "description": "Test POA&M mappings",
                "version": "1.0",
                "fedramp_v3_mappings": {
                    "required_columns": {
                        "poam_id": {
                            "column_names": ["POA&M Item ID"],
                            "oscal_field": "uuid",
                            "required": true,
                            "validation": "alphanumeric"
                        }
                    },
                    "validation_rules": {
                        "severity_levels": ["Low", "High"],
                        "status_values": ["Open", "Closed"]
                    }
                },
                "risk_mappings": {
                    "severity_to_risk_level": {},
                    "status_to_implementation": {}
                },
                "finding_mappings": {
                    "origin_types": {}
                },
                "milestone_processing": {
                    "patterns": {
                        "multiple_milestones": {
                            "separator_patterns": [";"],
                            "description": "Test patterns"
                        },
                        "milestone_format": {
                            "patterns": ["test"],
                            "groups": ["description"]
                        }
                    }
                },
                "quality_checks": {
                    "required_field_completeness": {
                        "critical_fields": ["poam_id"],
                        "minimum_completion_rate": 0.95
                    },
                    "data_consistency": {
                        "date_logic": "test",
                        "status_logic": "test"
                    },
                    "control_validation": {
                        "verify_control_ids": true,
                        "validate_against_catalog": "test"
                    }
                }
            }
        });

        fs::write(
            mappings_dir.join("poam_mappings.json"),
            serde_json::to_string_pretty(&poam_json).unwrap(),
        ).unwrap();

        // Create test SSP sections
        let ssp_json = serde_json::json!({
            "section_mappings": {
                "description": "Test SSP sections",
                "version": "1.0",
                "mappings": {
                    "system_identification": {
                        "keywords": ["system name"],
                        "target": "system-characteristics.system-name",
                        "required": true
                    }
                }
            },
            "control_extraction": {
                "patterns": [
                    {
                        "name": "nist_800_53",
                        "regex": "\\b[A-Z]{2}-\\d+\\b",
                        "description": "NIST controls"
                    }
                ]
            },
            "table_mappings": {
                "responsibility_matrix": {
                    "keywords": ["responsibility"],
                    "columns": {
                        "control_id": ["control"],
                        "customer_responsibility": ["customer"],
                        "csp_responsibility": ["csp"],
                        "shared_responsibility": ["shared"]
                    }
                },
                "inventory_summary": {
                    "keywords": ["inventory"],
                    "columns": {
                        "component_name": ["name"],
                        "component_type": ["type"],
                        "criticality": ["criticality"],
                        "environment": ["environment"]
                    }
                }
            }
        });

        fs::write(
            mappings_dir.join("ssp_sections.json"),
            serde_json::to_string_pretty(&ssp_json).unwrap(),
        ).unwrap();

        Ok(temp_dir)
    }

    #[tokio::test]
    async fn test_load_inventory_mappings() {
        let temp_dir = create_test_mappings_dir().await.unwrap();
        let loader = MappingConfigurationLoader::new(temp_dir.path());

        let result = loader.load_inventory_mappings().await;
        assert!(result.is_ok());

        let inventory = result.unwrap();
        assert_eq!(inventory.description, "Test inventory mappings");
        assert_eq!(inventory.version, "1.0");
        assert!(inventory.fedramp_iiw_mappings.required_columns.contains_key("asset_id"));
    }

    #[tokio::test]
    async fn test_load_poam_mappings() {
        let temp_dir = create_test_mappings_dir().await.unwrap();
        let loader = MappingConfigurationLoader::new(temp_dir.path());

        let result = loader.load_poam_mappings().await;
        assert!(result.is_ok());

        let poam = result.unwrap();
        assert_eq!(poam.description, "Test POA&M mappings");
        assert!(poam.fedramp_v3_mappings.required_columns.contains_key("poam_id"));
    }

    #[tokio::test]
    async fn test_load_ssp_sections() {
        let temp_dir = create_test_mappings_dir().await.unwrap();
        let loader = MappingConfigurationLoader::new(temp_dir.path());

        let result = loader.load_ssp_sections().await;
        assert!(result.is_ok());

        let ssp = result.unwrap();
        assert_eq!(ssp.section_mappings.description, "Test SSP sections");
        assert!(ssp.section_mappings.mappings.contains_key("system_identification"));
    }

    #[tokio::test]
    async fn test_load_all_configurations() {
        let temp_dir = create_test_mappings_dir().await.unwrap();
        let mut loader = MappingConfigurationLoader::new(temp_dir.path());

        let result = loader.load_all_configurations().await;
        assert!(result.is_ok());

        let config = result.unwrap();
        assert!(config.inventory_mappings.is_some());
        assert!(config.poam_mappings.is_some());
        assert!(config.ssp_sections.is_some());
    }

    #[tokio::test]
    async fn test_configuration_validation() {
        let temp_dir = create_test_mappings_dir().await.unwrap();
        let mut loader = MappingConfigurationLoader::new(temp_dir.path());
        let config = loader.load_all_configurations().await.unwrap();

        let warnings = loader.validate_configuration(&config).unwrap();
        // Should have some warnings due to incomplete test data
        assert!(!warnings.is_empty());
    }

    #[tokio::test]
    async fn test_optimized_lookup_creation() {
        let temp_dir = create_test_mappings_dir().await.unwrap();
        let mut loader = MappingConfigurationLoader::new(temp_dir.path());
        let config = loader.load_all_configurations().await.unwrap();

        let lookup = OptimizedMappingLookup::from_configuration(&config);
        assert!(lookup.is_ok());

        let lookup = lookup.unwrap();
        let stats = lookup.get_statistics();
        assert!(stats["exact_matches"].as_u64().unwrap() > 0);
    }

    #[tokio::test]
    async fn test_column_mapping_with_optimized_lookup() {
        let temp_dir = create_test_mappings_dir().await.unwrap();
        let mut mapper = ColumnMapper::with_config_loader(temp_dir.path());

        let result = mapper.load_configurations().await;
        assert!(result.is_ok());

        let headers = vec!["Asset ID".to_string(), "POA&M Item ID".to_string()];
        let mapping_results = mapper.map_columns(&headers).unwrap();

        assert_eq!(mapping_results.len(), 2);
        assert!(mapping_results.iter().any(|r| r.target_field == "uuid"));
    }

    #[tokio::test]
    async fn test_fuzzy_matching() {
        let temp_dir = create_test_mappings_dir().await.unwrap();
        let mut mapper = ColumnMapper::with_config_loader(temp_dir.path());
        mapper.load_configurations().await.unwrap();

        // Test fuzzy matching with slight variations
        let headers = vec!["Asset_ID".to_string(), "Component_ID".to_string()];
        let mapping_results = mapper.map_columns(&headers).unwrap();

        assert!(!mapping_results.is_empty());
        assert!(mapping_results.iter().any(|r| r.confidence > 0.7));
    }

    #[tokio::test]
    async fn test_performance_sub_100ms() {
        let temp_dir = create_test_mappings_dir().await.unwrap();
        let mut loader = MappingConfigurationLoader::new(temp_dir.path());

        let start = Instant::now();
        let _config = loader.load_all_configurations().await.unwrap();
        let duration = start.elapsed();

        assert!(duration.as_millis() < 100, "Loading took {}ms, should be < 100ms", duration.as_millis());
    }

    #[tokio::test]
    async fn test_error_handling_missing_file() {
        let temp_dir = TempDir::new().unwrap();
        let loader = MappingConfigurationLoader::new(temp_dir.path());

        let result = loader.load_inventory_mappings().await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_error_handling_malformed_json() {
        let temp_dir = TempDir::new().unwrap();
        let mappings_dir = temp_dir.path().join("mappings");
        fs::create_dir_all(&mappings_dir).unwrap();

        // Write malformed JSON
        fs::write(
            mappings_dir.join("inventory_mappings.json"),
            "{ invalid json",
        ).unwrap();

        let loader = MappingConfigurationLoader::new(temp_dir.path());
        let result = loader.load_inventory_mappings().await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_validation_rules() {
        let temp_dir = create_test_mappings_dir().await.unwrap();
        let mut mapper = ColumnMapper::with_config_loader(temp_dir.path());
        mapper.load_configurations().await.unwrap();

        // Test field validation
        let errors = mapper.validate_field_value("uuid", "").unwrap();
        assert!(!errors.is_empty()); // Should fail for empty unique identifier
    }

    #[tokio::test]
    async fn test_required_field_validation() {
        let temp_dir = create_test_mappings_dir().await.unwrap();
        let mut mapper = ColumnMapper::with_config_loader(temp_dir.path());
        mapper.load_configurations().await.unwrap();

        // Test with missing required fields
        let mapping_results = vec![]; // Empty results
        let errors = mapper.validate_required_mappings(&mapping_results).unwrap();
        assert!(!errors.is_empty()); // Should have errors for missing required fields
    }

    #[tokio::test]
    async fn test_advanced_fuzzy_matching() {
        let temp_dir = create_test_mappings_dir().await.unwrap();
        let mut mapper = ColumnMapper::with_config_loader(temp_dir.path());
        mapper.load_configurations().await.unwrap();

        // Test advanced fuzzy matching with various column name variations
        let test_cases = vec![
            ("asset_id", "uuid"),
            ("Asset_ID", "uuid"),
            ("ASSET-ID", "uuid"),
            ("AssetIdentifier", "uuid"),
            ("poam_item_id", "uuid"),
            ("POA&M-Item-ID", "uuid"),
        ];

        for (input, expected_field) in test_cases {
            let headers = vec![input.to_string()];
            let results = mapper.map_columns(&headers).unwrap();

            if !results.is_empty() {
                assert_eq!(results[0].target_field, expected_field,
                    "Failed to map '{}' to '{}'", input, expected_field);
                assert!(results[0].confidence > 0.6,
                    "Low confidence for '{}': {}", input, results[0].confidence);
            }
        }
    }

    #[tokio::test]
    async fn test_fuzzy_matching_performance() {
        let temp_dir = create_test_mappings_dir().await.unwrap();
        let mut mapper = ColumnMapper::with_config_loader(temp_dir.path());
        mapper.load_configurations().await.unwrap();

        // Create a large list of column headers to test performance
        let mut headers = Vec::new();
        for i in 0..100 {
            headers.push(format!("Column_{}", i));
            headers.push(format!("Asset_ID_{}", i));
            headers.push(format!("Component_Name_{}", i));
        }

        let start = std::time::Instant::now();
        let _results = mapper.map_columns(&headers).unwrap();
        let duration = start.elapsed();

        // Should complete within reasonable time
        assert!(duration.as_millis() < 1000,
            "Fuzzy matching took {}ms, should be < 1000ms", duration.as_millis());
    }

    #[tokio::test]
    async fn test_detailed_fuzzy_match_results() {
        let temp_dir = create_test_mappings_dir().await.unwrap();
        let mut mapper = ColumnMapper::with_config_loader(temp_dir.path());
        mapper.load_configurations().await.unwrap();

        // Get detailed fuzzy match results
        let detailed_results = if let Some(lookup) = &mut mapper.optimized_lookup {
            lookup.get_detailed_fuzzy_matches("Asset_ID", 0.5)
        } else {
            Vec::new()
        };

        // Should have detailed algorithm scores for non-exact matches
        for result in &detailed_results {
            if !result.exact_match {
                assert!(!result.algorithm_scores.is_empty(), "Non-exact matches should have algorithm scores");
                // Check that at least one expected algorithm is present
                let has_expected_algo = result.algorithm_scores.contains_key("levenshtein") ||
                                       result.algorithm_scores.contains_key("jaro_winkler") ||
                                       result.algorithm_scores.contains_key("ngram");
                assert!(has_expected_algo, "Should have at least one expected algorithm score");
            }
            assert!(!result.preprocessing_applied.is_empty(), "Should have preprocessing information");
        }
    }

    #[tokio::test]
    async fn test_fuzzy_cache_functionality() {
        let temp_dir = create_test_mappings_dir().await.unwrap();
        let mut mapper = ColumnMapper::with_config_loader(temp_dir.path());
        mapper.load_configurations().await.unwrap();

        // Clear cache and check stats
        if let Some(lookup) = &mut mapper.optimized_lookup {
            lookup.clear_fuzzy_cache();
            let stats = lookup.get_statistics();
            assert_eq!(stats["fuzzy_cache_size"].as_u64().unwrap(), 0);
        }

        // Perform some matches to populate cache
        let headers = vec!["Asset_ID".to_string(), "Component_Name".to_string()];
        let _results = mapper.map_columns(&headers).unwrap();

        // Check that cache has entries
        if let Some(lookup) = &mapper.optimized_lookup {
            let stats = lookup.get_statistics();
            assert!(stats["fuzzy_cache_size"].as_u64().unwrap() > 0);
        }
    }

    #[tokio::test]
    async fn test_fuzzy_config_update() {
        let temp_dir = create_test_mappings_dir().await.unwrap();
        let mut mapper = ColumnMapper::with_config_loader(temp_dir.path());
        mapper.load_configurations().await.unwrap();

        if let Some(lookup) = &mut mapper.optimized_lookup {
            // Update fuzzy matching configuration
            let mut new_config = crate::fuzzy::FuzzyMatchConfig::default();
            new_config.min_confidence = 0.9;
            new_config.max_results = 3;

            lookup.update_fuzzy_config(new_config);

            // Test that new configuration is applied
            let results = lookup.get_detailed_fuzzy_matches("Asset", 0.9);
            assert!(results.len() <= 3);
            for result in &results {
                assert!(result.confidence >= 0.9);
            }
        }
    }
}

impl Default for ColumnMapper {
    fn default() -> Self {
        Self::new()
    }
}
