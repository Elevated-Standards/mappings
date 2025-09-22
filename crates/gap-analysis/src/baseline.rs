//! Baseline Management
//!
//! Handles loading, caching, and managing framework baselines for gap analysis.

use fedramp_core::{Result, Error};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use chrono::{DateTime, Utc};
use crate::engine::{TargetBaseline, RequiredControl, ImplementationStatus, BaselineMetadata, ParameterRequirement};

/// Baseline manager for loading and caching framework baselines
#[derive(Debug, Clone)]
pub struct BaselineManager {
    /// Cached baselines
    baselines: HashMap<String, CachedBaseline>,
    /// JSON baseline loader
    json_loader: Option<JsonBaselineLoader>,
    /// Configuration
    config: BaselineConfig,
}

/// Cached baseline with metadata
#[derive(Debug, Clone)]
pub struct CachedBaseline {
    pub baseline: TargetBaseline,
    pub cached_at: DateTime<Utc>,
    pub cache_ttl: chrono::Duration,
    pub source_checksum: String,
}

/// Configuration for baseline management
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BaselineConfig {
    pub cache_ttl_hours: i64,
    pub auto_refresh: bool,
    pub parallel_loading: bool,
    pub validation_enabled: bool,
}

/// Trait for loading baselines from different sources
pub trait BaselineLoader: Send + Sync {
    fn load_baseline(&self, framework_id: &str, profile: &str) -> Result<TargetBaseline>;
    fn get_available_profiles(&self, framework_id: &str) -> Result<Vec<String>>;
    fn validate_baseline(&self, baseline: &TargetBaseline) -> Result<ValidationResult>;
}

/// JSON-based baseline loader for local mapping files
#[derive(Debug, Clone)]
pub struct JsonBaselineLoader {
    pub mappings_path: String,
    pub control_mappings: ControlMappings,
}

/// Control mappings structure from JSON files
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ControlMappings {
    pub control_mappings: ControlMappingsData,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ControlMappingsData {
    pub description: String,
    pub version: String,
    pub frameworks: HashMap<String, FrameworkData>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FrameworkData {
    #[serde(default)]
    pub baseline_profiles: HashMap<String, BaselineProfile>,
    #[serde(default)]
    pub control_families: HashMap<String, String>,
    #[serde(default)]
    pub domains: HashMap<String, String>,
    #[serde(default)]
    pub controls: HashMap<String, ControlDefinition>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BaselineProfile {
    pub profile_url: Option<String>,
    pub control_count: usize,
    #[serde(default)]
    pub controls: Vec<String>,
    #[serde(default)]
    pub enhancements: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ControlDefinition {
    pub title: String,
    pub description: String,
    pub family: String,
    #[serde(default)]
    pub enhancements: Vec<String>,
    #[serde(default)]
    pub parameters: HashMap<String, ParameterDefinition>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParameterDefinition {
    pub name: String,
    pub description: String,
    pub parameter_type: String,
    #[serde(default)]
    pub required: bool,
    #[serde(default)]
    pub default_value: Option<String>,
}

/// Validation result for baselines
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationResult {
    pub is_valid: bool,
    pub errors: Vec<ValidationError>,
    pub warnings: Vec<ValidationWarning>,
    pub statistics: ValidationStatistics,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationError {
    pub error_type: String,
    pub message: String,
    pub control_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationWarning {
    pub warning_type: String,
    pub message: String,
    pub control_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationStatistics {
    pub total_controls: usize,
    pub valid_controls: usize,
    pub invalid_controls: usize,
    pub missing_enhancements: usize,
    pub missing_parameters: usize,
}

impl BaselineManager {
    /// Create a new baseline manager
    pub fn new() -> Self {
        Self {
            baselines: HashMap::new(),
            json_loader: None,
            config: BaselineConfig::default(),
        }
    }

    /// Create baseline manager with JSON loader
    pub fn with_json_loader(mappings_path: String) -> Result<Self> {
        let mut manager = Self::new();
        
        // Load control mappings from JSON file
        let mappings_content = std::fs::read_to_string(&mappings_path)
            .map_err(|e| Error::document_parsing(format!("Failed to read mappings file: {}", e)))?;

        let control_mappings: ControlMappings = serde_json::from_str(&mappings_content)
            .map_err(|e| Error::document_parsing(format!("Failed to parse mappings JSON: {}", e)))?;

        let json_loader = JsonBaselineLoader {
            mappings_path,
            control_mappings,
        };

        manager.json_loader = Some(json_loader);
        Ok(manager)
    }

    /// Get baseline for framework and profile
    pub async fn get_baseline(&mut self, framework_id: &str, profile: &str) -> Result<TargetBaseline> {
        let cache_key = format!("{}:{}", framework_id, profile);
        
        // Check cache first
        if let Some(cached) = self.baselines.get(&cache_key) {
            if cached.cached_at + cached.cache_ttl > Utc::now() {
                return Ok(cached.baseline.clone());
            }
        }

        // Load from source
        let baseline = self.load_baseline_from_source(framework_id, profile).await?;
        
        // Cache the result
        self.cache_baseline(cache_key, baseline.clone()).await?;
        
        Ok(baseline)
    }

    /// Load baseline from configured source
    async fn load_baseline_from_source(&self, framework_id: &str, profile: &str) -> Result<TargetBaseline> {
        // Try JSON loader first
        if let Some(loader) = &self.json_loader {
            return loader.load_baseline(framework_id, profile);
        }

        Err(fedramp_core::Error::not_found(format!("No loader available for framework: {}", framework_id)))
    }

    /// Cache baseline with TTL
    async fn cache_baseline(&mut self, cache_key: String, baseline: TargetBaseline) -> Result<()> {
        let cached_baseline = CachedBaseline {
            baseline,
            cached_at: Utc::now(),
            cache_ttl: chrono::Duration::hours(self.config.cache_ttl_hours),
            source_checksum: "".to_string(), // TODO: implement checksum
        };

        self.baselines.insert(cache_key, cached_baseline);
        Ok(())
    }

    /// Get available frameworks
    pub fn get_available_frameworks(&self) -> Result<Vec<String>> {
        if let Some(json_loader) = &self.json_loader {
            return Ok(json_loader.control_mappings.control_mappings.frameworks.keys().cloned().collect());
        }
        Ok(Vec::new())
    }

    /// Get available profiles for a framework
    pub fn get_available_profiles(&self, framework_id: &str) -> Result<Vec<String>> {
        if let Some(loader) = &self.json_loader {
            return loader.get_available_profiles(framework_id);
        }
        Ok(Vec::new())
    }

    /// Validate all cached baselines
    pub async fn validate_baselines(&self) -> Result<HashMap<String, ValidationResult>> {
        let mut results = HashMap::new();

        for (cache_key, cached_baseline) in &self.baselines {
            if let Some(loader) = &self.json_loader {
                let validation_result = loader.validate_baseline(&cached_baseline.baseline)?;
                results.insert(cache_key.clone(), validation_result);
            }
        }

        Ok(results)
    }
}

impl BaselineLoader for JsonBaselineLoader {
    fn load_baseline(&self, framework_id: &str, profile: &str) -> Result<TargetBaseline> {
        let framework_data = self.control_mappings.control_mappings.frameworks
            .get(framework_id)
            .ok_or_else(|| fedramp_core::Error::not_found(format!("Framework not found: {}", framework_id)))?;

        let profile_data = framework_data.baseline_profiles
            .get(profile)
            .ok_or_else(|| fedramp_core::Error::not_found(format!("Profile not found: {} for framework {}", profile, framework_id)))?;

        let mut required_controls = HashMap::new();

        // Load controls from profile
        if !profile_data.controls.is_empty() {
            // Use explicit control list from profile
            for control_id in &profile_data.controls {
                let required_control = RequiredControl {
                    control_id: control_id.clone(),
                    required_status: ImplementationStatus::Implemented,
                    enhancements: Vec::new(),
                    parameters: HashMap::new(),
                };
                required_controls.insert(control_id.clone(), required_control);
            }
        } else {
            // Generate controls based on framework structure
            required_controls = self.generate_baseline_controls(framework_data, profile)?;
        }

        Ok(TargetBaseline {
            framework_id: framework_id.to_string(),
            profile_name: profile.to_string(),
            required_controls,
            baseline_metadata: BaselineMetadata {
                version: self.control_mappings.control_mappings.version.clone(),
                last_updated: Utc::now(),
            },
        })
    }

    fn get_available_profiles(&self, framework_id: &str) -> Result<Vec<String>> {
        let framework_data = self.control_mappings.control_mappings.frameworks
            .get(framework_id)
            .ok_or_else(|| fedramp_core::Error::not_found(format!("Framework not found: {}", framework_id)))?;

        Ok(framework_data.baseline_profiles.keys().cloned().collect())
    }

    fn validate_baseline(&self, baseline: &TargetBaseline) -> Result<ValidationResult> {
        let mut errors = Vec::new();
        let mut warnings = Vec::new();
        
        let total_controls = baseline.required_controls.len();
        let mut valid_controls = 0;
        let mut invalid_controls = 0;
        let mut missing_enhancements = 0;
        let mut missing_parameters = 0;

        for (control_id, required_control) in &baseline.required_controls {
            // Validate control ID format
            if !self.is_valid_control_id(control_id) {
                errors.push(ValidationError {
                    error_type: "invalid_control_id".to_string(),
                    message: format!("Invalid control ID format: {}", control_id),
                    control_id: Some(control_id.clone()),
                });
                invalid_controls += 1;
            } else {
                valid_controls += 1;
            }

            // Check for missing enhancements
            if required_control.enhancements.is_empty() {
                missing_enhancements += 1;
                warnings.push(ValidationWarning {
                    warning_type: "missing_enhancements".to_string(),
                    message: format!("Control {} has no enhancements defined", control_id),
                    control_id: Some(control_id.clone()),
                });
            }

            // Check for missing parameters
            if required_control.parameters.is_empty() {
                missing_parameters += 1;
            }
        }

        Ok(ValidationResult {
            is_valid: errors.is_empty(),
            errors,
            warnings,
            statistics: ValidationStatistics {
                total_controls,
                valid_controls,
                invalid_controls,
                missing_enhancements,
                missing_parameters,
            },
        })
    }
}

impl JsonBaselineLoader {
    /// Generate baseline controls based on framework structure
    fn generate_baseline_controls(&self, framework_data: &FrameworkData, profile: &str) -> Result<HashMap<String, RequiredControl>> {
        let mut required_controls = HashMap::new();

        // For NIST 800-53, generate controls based on families
        if !framework_data.control_families.is_empty() {
            for (family_id, _family_name) in &framework_data.control_families {
                // Generate basic controls for each family (simplified)
                for i in 1..=10 {
                    let control_id = format!("{}-{}", family_id, i);
                    let required_control = RequiredControl {
                        control_id: control_id.clone(),
                        required_status: ImplementationStatus::Implemented,
                        enhancements: Vec::new(),
                        parameters: HashMap::new(),
                    };
                    required_controls.insert(control_id, required_control);
                }
            }
        }

        // For NIST 800-171, generate controls based on domains
        if !framework_data.domains.is_empty() {
            for (domain_id, _domain_name) in &framework_data.domains {
                // Generate requirements for each domain (simplified)
                for i in 1..=20 {
                    let control_id = format!("{}.{}", domain_id, i);
                    let required_control = RequiredControl {
                        control_id: control_id.clone(),
                        required_status: ImplementationStatus::Implemented,
                        enhancements: Vec::new(),
                        parameters: HashMap::new(),
                    };
                    required_controls.insert(control_id, required_control);
                }
            }
        }

        Ok(required_controls)
    }

    /// Validate control ID format
    fn is_valid_control_id(&self, control_id: &str) -> bool {
        // Basic validation - could be more sophisticated
        !control_id.is_empty() && control_id.len() <= 20
    }
}

impl Default for BaselineConfig {
    fn default() -> Self {
        Self {
            cache_ttl_hours: 24,
            auto_refresh: true,
            parallel_loading: true,
            validation_enabled: true,
        }
    }
}


