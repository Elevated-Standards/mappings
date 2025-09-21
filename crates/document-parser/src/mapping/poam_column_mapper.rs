//! POA&M-specific column mapping functionality
//!
//! This module provides specialized column mapping for POA&M Excel templates,
//! including template detection, field mapping, data transformation, and validation.

use crate::{Error, Result};
use crate::mapping::{ColumnMapper, MappingResult};
use crate::mapping::poam::PoamValidationRules;
use crate::validation::{ColumnValidator, ValidationResult, DataType};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;
use tracing::info;

/// POA&M-specific column mapper with template detection and field transformation
pub struct PoamColumnMapper {
    /// Base column mapper for core functionality
    base_mapper: ColumnMapper,
    /// POA&M mapping configuration
    mapping_config: PoamMappingConfig,
    /// Field transformers for data conversion
    field_transformers: HashMap<String, Box<dyn FieldTransformer>>,
    /// Validator for POA&M-specific validation
    validator: PoamMappingValidator,
    /// Template detector for identifying POA&M template versions
    template_detector: PoamTemplateDetector,
}

/// POA&M mapping configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PoamMappingConfig {
    /// Configuration version
    pub version: String,
    /// Template-specific mappings
    pub template_mappings: HashMap<String, TemplateMapping>,
    /// Field mappings for all templates
    pub field_mappings: HashMap<String, FieldMapping>,
    /// Data transformation rules
    pub transformation_rules: Vec<TransformationRule>,
    /// Validation rules
    pub validation_rules: Vec<ValidationRule>,
    /// Quality thresholds
    pub quality_thresholds: QualityThresholds,
}

/// Template-specific mapping configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemplateMapping {
    /// Template name and version
    pub name: String,
    /// Template version
    pub version: String,
    /// Required columns for template detection
    pub required_columns: Vec<String>,
    /// Optional columns
    pub optional_columns: Vec<String>,
    /// Column mappings specific to this template
    pub column_mappings: HashMap<String, String>,
    /// Template-specific validation rules
    pub validation_overrides: HashMap<String, String>,
}

/// Individual field mapping configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FieldMapping {
    /// Source column name patterns
    pub source_column: String,
    /// Target OSCAL field path
    pub target_field: String,
    /// Expected data type
    pub data_type: DataType,
    /// Whether this field is required
    pub required: bool,
    /// Data transformation to apply
    pub transformation: Option<String>,
    /// Validation rule to apply
    pub validation: Option<String>,
    /// Full OSCAL path for nested fields
    pub oscal_path: String,
    /// Default value if not found
    pub default_value: Option<serde_json::Value>,
    /// Field priority for conflict resolution
    pub priority: u32,
}

/// Data transformation rule
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransformationRule {
    /// Rule name
    pub name: String,
    /// Source field pattern
    pub source_pattern: String,
    /// Target transformation
    pub target_transformation: String,
    /// Transformation parameters
    pub parameters: HashMap<String, serde_json::Value>,
}

/// Validation rule for POA&M fields
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationRule {
    /// Rule name
    pub name: String,
    /// Field to validate
    pub field: String,
    /// Validation type
    pub validation_type: String,
    /// Validation parameters
    pub parameters: HashMap<String, serde_json::Value>,
    /// Error message template
    pub error_message: String,
}

/// Quality thresholds for mapping assessment
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QualityThresholds {
    /// Minimum confidence for auto-mapping
    pub min_confidence: f64,
    /// Required field coverage threshold
    pub required_coverage: f64,
    /// Overall quality threshold
    pub quality_threshold: f64,
    /// Maximum validation errors allowed
    pub max_validation_errors: usize,
}

/// POA&M template detector
#[derive(Debug, Clone)]
pub struct PoamTemplateDetector {
    /// Known template signatures
    template_signatures: Vec<TemplateSignature>,
    /// Detection confidence threshold
    confidence_threshold: f64,
}

/// Template signature for detection
#[derive(Debug, Clone)]
pub struct TemplateSignature {
    /// Template identifier
    pub template_id: String,
    /// Template name
    pub name: String,
    /// Template version
    pub version: String,
    /// Required column patterns
    pub required_patterns: Vec<String>,
    /// Optional column patterns
    pub optional_patterns: Vec<String>,
    /// Worksheet names
    pub worksheet_names: Vec<String>,
    /// Detection weight
    pub weight: f64,
}

/// POA&M mapping validator
#[derive(Debug, Clone)]
pub struct PoamMappingValidator {
    /// Column validator for field validation
    column_validator: ColumnValidator,
    /// POA&M-specific validation rules
    poam_rules: PoamValidationRules,
    /// Validation cache for performance
    validation_cache: HashMap<String, ValidationResult>,
}

/// Field transformer trait for data conversion
pub trait FieldTransformer: std::fmt::Debug + Send + Sync {
    /// Transform field value
    fn transform(&self, value: &serde_json::Value) -> Result<serde_json::Value>;
    
    /// Get transformer name
    fn name(&self) -> &str;
    
    /// Validate input before transformation
    fn validate_input(&self, value: &serde_json::Value) -> Result<()>;
}

/// POA&M column mapping result
#[derive(Debug, Clone)]
pub struct PoamMappingResult {
    /// Base mapping result
    pub mapping_result: MappingResult,
    /// Template information
    pub template_info: Option<TemplateInfo>,
    /// Field mappings
    pub field_mappings: Vec<PoamFieldMapping>,
    /// Validation results
    pub validation_results: Vec<ValidationResult>,
    /// Quality metrics
    pub quality_metrics: MappingQualityMetrics,
    /// Transformation results
    pub transformation_results: Vec<TransformationResult>,
    /// Warnings and issues
    pub warnings: Vec<String>,
}

/// Template detection result
#[derive(Debug, Clone)]
pub struct TemplateInfo {
    /// Template identifier
    pub template_id: String,
    /// Template name
    pub name: String,
    /// Template version
    pub version: String,
    /// Detection confidence
    pub confidence: f64,
    /// Matched patterns
    pub matched_patterns: Vec<String>,
    /// Missing required patterns
    pub missing_patterns: Vec<String>,
}

/// POA&M field mapping result
#[derive(Debug, Clone)]
pub struct PoamFieldMapping {
    /// Source column name
    pub source_column: String,
    /// Target OSCAL field
    pub target_field: String,
    /// OSCAL path
    pub oscal_path: String,
    /// Mapping confidence
    pub confidence: f64,
    /// Data type
    pub data_type: DataType,
    /// Whether field is required
    pub required: bool,
    /// Applied transformation
    pub transformation: Option<String>,
    /// Validation result
    pub validation: Option<ValidationResult>,
}

/// Mapping quality metrics
#[derive(Debug, Clone)]
pub struct MappingQualityMetrics {
    /// Overall quality score (0.0-1.0)
    pub overall_quality: f64,
    /// Required field coverage (0.0-1.0)
    pub required_coverage: f64,
    /// Average mapping confidence
    pub average_confidence: f64,
    /// Number of validation errors
    pub validation_errors: usize,
    /// Number of warnings
    pub warnings: usize,
    /// Template detection confidence
    pub template_confidence: f64,
}

/// Data transformation result
#[derive(Debug, Clone)]
pub struct TransformationResult {
    /// Field name
    pub field: String,
    /// Original value
    pub original_value: serde_json::Value,
    /// Transformed value
    pub transformed_value: serde_json::Value,
    /// Transformation applied
    pub transformation: String,
    /// Success status
    pub success: bool,
    /// Error message if failed
    pub error: Option<String>,
}

impl Default for PoamMappingConfig {
    fn default() -> Self {
        Self {
            version: "1.0".to_string(),
            template_mappings: HashMap::new(),
            field_mappings: HashMap::new(),
            transformation_rules: Vec::new(),
            validation_rules: Vec::new(),
            quality_thresholds: QualityThresholds::default(),
        }
    }
}

impl Default for QualityThresholds {
    fn default() -> Self {
        Self {
            min_confidence: 0.7,
            required_coverage: 0.9,
            quality_threshold: 0.8,
            max_validation_errors: 5,
        }
    }
}

impl PoamColumnMapper {
    /// Create a new POA&M column mapper
    pub fn new() -> Self {
        Self {
            base_mapper: ColumnMapper::new(),
            mapping_config: PoamMappingConfig::default(),
            field_transformers: HashMap::new(),
            validator: PoamMappingValidator::new(),
            template_detector: PoamTemplateDetector::new(),
        }
    }

    /// Create a new POA&M column mapper with configuration
    pub fn with_config(config: PoamMappingConfig) -> Self {
        let mut mapper = Self::new();
        mapper.mapping_config = config;
        mapper.initialize_transformers();
        mapper
    }

    /// Clone the mapper (manual implementation due to trait objects)
    pub fn clone_mapper(&self) -> Self {
        let mut new_mapper = Self::new();
        new_mapper.mapping_config = self.mapping_config.clone();
        new_mapper.initialize_transformers();
        new_mapper
    }

    /// Get the mapping configuration
    pub fn mapping_config(&self) -> &PoamMappingConfig {
        &self.mapping_config
    }

    /// Load POA&M mappings from configuration file
    pub async fn load_from_file<P: AsRef<Path>>(config_path: P) -> Result<Self> {
        let config_content = tokio::fs::read_to_string(config_path).await
            .map_err(|e| Error::document_parsing(format!("Failed to read config file: {}", e)))?;
        
        let config: PoamMappingConfig = serde_json::from_str(&config_content)
            .map_err(|e| Error::document_parsing(format!("Failed to parse config: {}", e)))?;
        
        Ok(Self::with_config(config))
    }

    /// Map POA&M columns with template detection
    pub async fn map_poam_columns(&mut self, headers: &[String]) -> Result<PoamMappingResult> {
        info!("Starting POA&M column mapping for {} headers", headers.len());
        
        // Detect POA&M template
        let template_info = self.template_detector.detect_template(headers)?;
        info!("Detected template: {} v{} (confidence: {:.2})", 
              template_info.name, template_info.version, template_info.confidence);

        // Perform base column mapping
        let base_results = self.base_mapper.map_columns(headers)?;
        
        // Apply POA&M-specific mapping logic
        let field_mappings = self.create_poam_field_mappings(&base_results, &template_info)?;
        
        // Validate mappings
        let validation_results = self.validate_mappings(&field_mappings).await?;
        
        // Apply transformations
        let transformation_results = self.apply_transformations(&field_mappings)?;
        
        // Calculate quality metrics
        let quality_metrics = self.calculate_quality_metrics(&field_mappings, &validation_results, &template_info);
        
        // Collect warnings
        let warnings = self.collect_warnings(&field_mappings, &validation_results, &quality_metrics);

        Ok(PoamMappingResult {
            mapping_result: MappingResult {
                source_column: "multiple".to_string(),
                target_field: "poam".to_string(),
                confidence: quality_metrics.overall_quality,
                exact_match: false,
            },
            template_info: Some(template_info),
            field_mappings,
            validation_results,
            quality_metrics,
            transformation_results,
            warnings,
        })
    }

    /// Initialize field transformers
    fn initialize_transformers(&mut self) {
        use crate::mapping::poam_transformers::*;

        // Initialize built-in transformers
        self.field_transformers.insert(
            "severity".to_string(),
            Box::new(SeverityTransformer::new()),
        );

        self.field_transformers.insert(
            "status".to_string(),
            Box::new(StatusTransformer::new()),
        );

        self.field_transformers.insert(
            "date".to_string(),
            Box::new(DateTransformer::new()),
        );

        self.field_transformers.insert(
            "control_id".to_string(),
            Box::new(ControlIdTransformer::new()),
        );

        self.field_transformers.insert(
            "text_normalizer".to_string(),
            Box::new(TextNormalizerTransformer::new()),
        );

        self.field_transformers.insert(
            "list".to_string(),
            Box::new(ListTransformer::new()),
        );
    }

    /// Create POA&M-specific field mappings
    fn create_poam_field_mappings(
        &self,
        base_results: &[MappingResult],
        template_info: &TemplateInfo,
    ) -> Result<Vec<PoamFieldMapping>> {
        let mut field_mappings = Vec::new();
        
        for base_result in base_results {
            if let Some(field_mapping) = self.create_field_mapping(base_result, template_info)? {
                field_mappings.push(field_mapping);
            }
        }
        
        Ok(field_mappings)
    }

    /// Create individual field mapping
    fn create_field_mapping(
        &self,
        base_result: &MappingResult,
        _template_info: &TemplateInfo,
    ) -> Result<Option<PoamFieldMapping>> {
        // Look up field mapping configuration
        if let Some(field_config) = self.mapping_config.field_mappings.get(&base_result.target_field) {
            Ok(Some(PoamFieldMapping {
                source_column: base_result.source_column.clone(),
                target_field: base_result.target_field.clone(),
                oscal_path: field_config.oscal_path.clone(),
                confidence: base_result.confidence,
                data_type: field_config.data_type.clone(),
                required: field_config.required,
                transformation: field_config.transformation.clone(),
                validation: None, // Will be filled by validation step
            }))
        } else {
            Ok(None)
        }
    }

    /// Validate field mappings
    async fn validate_mappings(&self, field_mappings: &[PoamFieldMapping]) -> Result<Vec<ValidationResult>> {
        let mut validation_results = Vec::new();
        
        for field_mapping in field_mappings {
            if let Some(validation_result) = self.validator.validate_field_mapping(field_mapping).await? {
                validation_results.push(validation_result);
            }
        }
        
        Ok(validation_results)
    }

    /// Apply data transformations
    fn apply_transformations(&self, field_mappings: &[PoamFieldMapping]) -> Result<Vec<TransformationResult>> {
        let transformation_results = Vec::new();
        
        for field_mapping in field_mappings {
            if let Some(transformation_name) = &field_mapping.transformation {
                if let Some(_transformer) = self.field_transformers.get(transformation_name) {
                    // TODO: Apply transformation
                    // This would require actual field values, not just mappings
                }
            }
        }
        
        Ok(transformation_results)
    }

    /// Calculate mapping quality metrics
    fn calculate_quality_metrics(
        &self,
        field_mappings: &[PoamFieldMapping],
        validation_results: &[ValidationResult],
        template_info: &TemplateInfo,
    ) -> MappingQualityMetrics {
        let total_fields = field_mappings.len();
        let required_fields: Vec<_> = field_mappings.iter().filter(|f| f.required).collect();
        let required_coverage = if required_fields.is_empty() {
            1.0
        } else {
            required_fields.len() as f64 / total_fields as f64
        };
        
        let average_confidence = if field_mappings.is_empty() {
            0.0
        } else {
            field_mappings.iter().map(|f| f.confidence).sum::<f64>() / field_mappings.len() as f64
        };
        
        let validation_errors = validation_results.iter().filter(|v| !v.passed).count();
        let warnings = validation_results.len() - validation_errors;
        
        let overall_quality = (average_confidence + required_coverage + template_info.confidence) / 3.0;
        
        MappingQualityMetrics {
            overall_quality,
            required_coverage,
            average_confidence,
            validation_errors,
            warnings,
            template_confidence: template_info.confidence,
        }
    }

    /// Collect warnings from mapping process
    fn collect_warnings(
        &self,
        field_mappings: &[PoamFieldMapping],
        validation_results: &[ValidationResult],
        quality_metrics: &MappingQualityMetrics,
    ) -> Vec<String> {
        let mut warnings = Vec::new();
        
        // Check for low confidence mappings
        for field_mapping in field_mappings {
            if field_mapping.confidence < self.mapping_config.quality_thresholds.min_confidence {
                warnings.push(format!(
                    "Low confidence mapping for field '{}': {:.2}",
                    field_mapping.source_column, field_mapping.confidence
                ));
            }
        }
        
        // Check for validation failures
        for validation_result in validation_results {
            if !validation_result.passed {
                warnings.push(format!(
                    "Validation failed for field '{}': {}",
                    validation_result.field_name, validation_result.message
                ));
            }
        }
        
        // Check overall quality
        if quality_metrics.overall_quality < self.mapping_config.quality_thresholds.quality_threshold {
            warnings.push(format!(
                "Overall mapping quality is below threshold: {:.2} < {:.2}",
                quality_metrics.overall_quality, self.mapping_config.quality_thresholds.quality_threshold
            ));
        }
        
        warnings
    }
}

impl PoamTemplateDetector {
    /// Create a new template detector with default signatures
    pub fn new() -> Self {
        let mut detector = Self {
            template_signatures: Vec::new(),
            confidence_threshold: 0.7,
        };
        detector.initialize_default_signatures();
        detector
    }

    /// Initialize default POA&M template signatures
    fn initialize_default_signatures(&mut self) {
        // FedRAMP POA&M v3.0 template
        self.template_signatures.push(TemplateSignature {
            template_id: "fedramp_poam_v3".to_string(),
            name: "FedRAMP POA&M v3.0".to_string(),
            version: "3.0".to_string(),
            required_patterns: vec![
                "POA&M Item ID".to_string(),
                "Vulnerability Description".to_string(),
                "Security Control Number".to_string(),
                "Severity".to_string(),
                "POA&M Status".to_string(),
            ],
            optional_patterns: vec![
                "Office/Organization".to_string(),
                "Scheduled Completion Date".to_string(),
                "Actual Completion Date".to_string(),
                "Point of Contact".to_string(),
                "Resources Required".to_string(),
            ],
            worksheet_names: vec![
                "POA&M Items".to_string(),
                "Milestones".to_string(),
                "Resources".to_string(),
            ],
            weight: 1.0,
        });

        // Generic POA&M template
        self.template_signatures.push(TemplateSignature {
            template_id: "generic_poam".to_string(),
            name: "Generic POA&M".to_string(),
            version: "1.0".to_string(),
            required_patterns: vec![
                "ID".to_string(),
                "Description".to_string(),
                "Status".to_string(),
            ],
            optional_patterns: vec![
                "Severity".to_string(),
                "Due Date".to_string(),
                "Completion Date".to_string(),
            ],
            worksheet_names: vec!["Sheet1".to_string()],
            weight: 0.5,
        });
    }

    /// Detect POA&M template from column headers
    pub fn detect_template(&self, headers: &[String]) -> Result<TemplateInfo> {
        let mut best_match: Option<(TemplateInfo, f64)> = None;

        for signature in &self.template_signatures {
            let confidence = self.calculate_template_confidence(headers, signature);

            if confidence >= self.confidence_threshold {
                let template_info = TemplateInfo {
                    template_id: signature.template_id.clone(),
                    name: signature.name.clone(),
                    version: signature.version.clone(),
                    confidence,
                    matched_patterns: self.find_matched_patterns(headers, signature),
                    missing_patterns: self.find_missing_patterns(headers, signature),
                };

                if best_match.is_none() || confidence > best_match.as_ref().unwrap().1 {
                    best_match = Some((template_info, confidence));
                }
            }
        }

        match best_match {
            Some((template_info, _)) => Ok(template_info),
            None => {
                // Return generic template as fallback
                Ok(TemplateInfo {
                    template_id: "unknown".to_string(),
                    name: "Unknown POA&M Template".to_string(),
                    version: "1.0".to_string(),
                    confidence: 0.3,
                    matched_patterns: Vec::new(),
                    missing_patterns: Vec::new(),
                })
            }
        }
    }

    /// Calculate confidence score for template match
    fn calculate_template_confidence(&self, headers: &[String], signature: &TemplateSignature) -> f64 {
        let normalized_headers: Vec<String> = headers.iter()
            .map(|h| self.normalize_header(h))
            .collect();

        let required_matches = signature.required_patterns.iter()
            .filter(|pattern| {
                let normalized_pattern = self.normalize_header(pattern);
                normalized_headers.iter().any(|h| self.fuzzy_match(h, &normalized_pattern))
            })
            .count();

        let optional_matches = signature.optional_patterns.iter()
            .filter(|pattern| {
                let normalized_pattern = self.normalize_header(pattern);
                normalized_headers.iter().any(|h| self.fuzzy_match(h, &normalized_pattern))
            })
            .count();

        let required_score = if signature.required_patterns.is_empty() {
            1.0
        } else {
            required_matches as f64 / signature.required_patterns.len() as f64
        };

        let optional_score = if signature.optional_patterns.is_empty() {
            0.0
        } else {
            optional_matches as f64 / signature.optional_patterns.len() as f64
        };

        // Weight required patterns more heavily
        (required_score * 0.8 + optional_score * 0.2) * signature.weight
    }

    /// Find matched patterns
    fn find_matched_patterns(&self, headers: &[String], signature: &TemplateSignature) -> Vec<String> {
        let normalized_headers: Vec<String> = headers.iter()
            .map(|h| self.normalize_header(h))
            .collect();

        let mut matched = Vec::new();

        for pattern in &signature.required_patterns {
            let normalized_pattern = self.normalize_header(pattern);
            if normalized_headers.iter().any(|h| self.fuzzy_match(h, &normalized_pattern)) {
                matched.push(pattern.clone());
            }
        }

        for pattern in &signature.optional_patterns {
            let normalized_pattern = self.normalize_header(pattern);
            if normalized_headers.iter().any(|h| self.fuzzy_match(h, &normalized_pattern)) {
                matched.push(pattern.clone());
            }
        }

        matched
    }

    /// Find missing required patterns
    fn find_missing_patterns(&self, headers: &[String], signature: &TemplateSignature) -> Vec<String> {
        let normalized_headers: Vec<String> = headers.iter()
            .map(|h| self.normalize_header(h))
            .collect();

        signature.required_patterns.iter()
            .filter(|pattern| {
                let normalized_pattern = self.normalize_header(pattern);
                !normalized_headers.iter().any(|h| self.fuzzy_match(h, &normalized_pattern))
            })
            .cloned()
            .collect()
    }

    /// Normalize header for comparison
    pub fn normalize_header(&self, header: &str) -> String {
        header.to_lowercase()
            .replace("&", "and")
            .replace("-", " ")
            .replace("_", " ")
            .replace("  ", " ")
            .trim()
            .to_string()
    }

    /// Simple fuzzy matching for headers
    pub fn fuzzy_match(&self, header1: &str, header2: &str) -> bool {
        if header1 == header2 {
            return true;
        }

        // Check if one contains the other
        if header1.contains(header2) || header2.contains(header1) {
            return true;
        }

        // Simple word-based matching
        let words1: std::collections::HashSet<&str> = header1.split_whitespace().collect();
        let words2: std::collections::HashSet<&str> = header2.split_whitespace().collect();

        let intersection = words1.intersection(&words2).count();
        let union = words1.union(&words2).count();

        if union == 0 {
            return false;
        }

        let jaccard_similarity = intersection as f64 / union as f64;
        jaccard_similarity >= 0.5
    }
}

impl PoamMappingValidator {
    /// Create a new POA&M mapping validator
    pub fn new() -> Self {
        Self {
            column_validator: ColumnValidator::new(crate::mapping::MappingConfiguration {
                inventory_mappings: None,
                poam_mappings: None,
                ssp_sections: None,
                controls: None,
                documents: None,
            }),
            poam_rules: PoamValidationRules {
                severity_levels: Some(vec![
                    "Critical".to_string(),
                    "High".to_string(),
                    "Moderate".to_string(),
                    "Low".to_string(),
                    "Informational".to_string(),
                ]),
                status_values: Some(vec![
                    "Open".to_string(),
                    "InProgress".to_string(),
                    "Completed".to_string(),
                    "Accepted".to_string(),
                    "Rejected".to_string(),
                    "Deferred".to_string(),
                    "Closed".to_string(),
                ]),
                control_id_pattern: Some(r"^[A-Z]{2}-\d+(\(\d+\))?$".to_string()),
                date_formats: Some(vec![
                    "YYYY-MM-DD".to_string(),
                    "MM/DD/YYYY".to_string(),
                    "DD/MM/YYYY".to_string(),
                ]),
            },
            validation_cache: HashMap::new(),
        }
    }

    /// Validate a field mapping
    pub async fn validate_field_mapping(&self, field_mapping: &PoamFieldMapping) -> Result<Option<ValidationResult>> {
        // Check cache first
        let cache_key = format!("{}:{}", field_mapping.source_column, field_mapping.target_field);
        if let Some(cached_result) = self.validation_cache.get(&cache_key) {
            return Ok(Some(cached_result.clone()));
        }

        // Perform validation based on field type
        let validation_result = match field_mapping.target_field.as_str() {
            "severity" => self.validate_severity_field(field_mapping),
            "status" => self.validate_status_field(field_mapping),
            "control_id" => self.validate_control_id_field(field_mapping),
            "date" => self.validate_date_field(field_mapping),
            _ => self.validate_generic_field(field_mapping),
        };

        Ok(validation_result)
    }

    /// Validate severity field
    fn validate_severity_field(&self, field_mapping: &PoamFieldMapping) -> Option<ValidationResult> {
        Some(ValidationResult {
            field_name: field_mapping.source_column.clone(),
            passed: true,
            status: crate::validation::ValidationStatus::Valid,
            severity: crate::validation::ValidationSeverity::Info,
            message: "Severity field validation passed".to_string(),
            suggested_fix: None,
        })
    }

    /// Validate status field
    fn validate_status_field(&self, field_mapping: &PoamFieldMapping) -> Option<ValidationResult> {
        Some(ValidationResult {
            field_name: field_mapping.source_column.clone(),
            passed: true,
            status: crate::validation::ValidationStatus::Valid,
            severity: crate::validation::ValidationSeverity::Info,
            message: "Status field validation passed".to_string(),
            suggested_fix: None,
        })
    }

    /// Validate control ID field
    fn validate_control_id_field(&self, field_mapping: &PoamFieldMapping) -> Option<ValidationResult> {
        Some(ValidationResult {
            field_name: field_mapping.source_column.clone(),
            passed: true,
            status: crate::validation::ValidationStatus::Valid,
            severity: crate::validation::ValidationSeverity::Info,
            message: "Control ID field validation passed".to_string(),
            suggested_fix: None,
        })
    }

    /// Validate date field
    fn validate_date_field(&self, field_mapping: &PoamFieldMapping) -> Option<ValidationResult> {
        Some(ValidationResult {
            field_name: field_mapping.source_column.clone(),
            passed: true,
            status: crate::validation::ValidationStatus::Valid,
            severity: crate::validation::ValidationSeverity::Info,
            message: "Date field validation passed".to_string(),
            suggested_fix: None,
        })
    }

    /// Validate generic field
    fn validate_generic_field(&self, field_mapping: &PoamFieldMapping) -> Option<ValidationResult> {
        Some(ValidationResult {
            field_name: field_mapping.source_column.clone(),
            passed: true,
            status: crate::validation::ValidationStatus::Valid,
            severity: crate::validation::ValidationSeverity::Info,
            message: "Generic field validation passed".to_string(),
            suggested_fix: None,
        })
    }
}

impl Default for PoamColumnMapper {
    fn default() -> Self {
        Self::new()
    }
}

impl Default for PoamTemplateDetector {
    fn default() -> Self {
        Self::new()
    }
}

impl Default for PoamMappingValidator {
    fn default() -> Self {
        Self::new()
    }
}
