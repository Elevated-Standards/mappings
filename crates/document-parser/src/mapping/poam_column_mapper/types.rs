//! Type definitions and configuration structures for POA&M column mapping
//! Modified: 2025-01-22

use crate::{Error, Result};
use crate::mapping::{ColumnMapper, MappingResult};
use crate::mapping::poam::PoamValidationRules;
use crate::validation::{ColumnValidator, ValidationResult, DataType};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// POA&M-specific column mapper with template detection and field transformation
pub struct PoamColumnMapper {
    /// Base column mapper for core functionality
    pub base_mapper: ColumnMapper,
    /// POA&M mapping configuration
    pub mapping_config: PoamMappingConfig,
    /// Field transformers for data conversion
    pub field_transformers: HashMap<String, Box<dyn FieldTransformer>>,
    /// Validator for POA&M-specific validation
    pub validator: PoamMappingValidator,
    /// Template detector for identifying POA&M template versions
    pub template_detector: PoamTemplateDetector,
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
    /// Required fields for this template
    pub required_fields: Vec<String>,
    /// Optional fields for this template
    pub optional_fields: Vec<String>,
    /// Field mappings specific to this template
    pub field_mappings: HashMap<String, FieldMapping>,
    /// Template-specific validation rules
    pub validation_rules: Vec<ValidationRule>,
}

/// Field mapping configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FieldMapping {
    /// Source column patterns
    pub source_patterns: Vec<String>,
    /// Target OSCAL field
    pub target_field: String,
    /// Field data type
    pub data_type: DataType,
    /// Whether field is required
    pub required: bool,
    /// Default value if not found
    pub default_value: Option<serde_json::Value>,
    /// Transformation rules for this field
    pub transformations: Vec<String>,
    /// Validation rules for this field
    pub validations: Vec<String>,
}

/// Data transformation rule
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransformationRule {
    /// Rule name
    pub name: String,
    /// Source field pattern
    pub source_field: String,
    /// Target field
    pub target_field: String,
    /// Transformation type
    pub transformation_type: String,
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
    pub template_signatures: Vec<TemplateSignature>,
    /// Detection confidence threshold
    pub confidence_threshold: f64,
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
    pub column_validator: ColumnValidator,
    /// POA&M-specific validation rules
    pub poam_rules: PoamValidationRules,
    /// Validation cache for performance
    pub validation_cache: HashMap<String, ValidationResult>,
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
    /// Detection confidence (0.0 to 1.0)
    pub confidence: f64,
    /// Matched patterns
    pub matched_patterns: Vec<String>,
    /// Missing required patterns
    pub missing_patterns: Vec<String>,
    /// Additional metadata
    pub metadata: HashMap<String, serde_json::Value>,
}

/// POA&M field mapping result
#[derive(Debug, Clone)]
pub struct PoamFieldMapping {
    /// Source column name
    pub source_column: String,
    /// Target OSCAL field
    pub target_field: String,
    /// Mapping confidence
    pub confidence: f64,
    /// Field data type
    pub data_type: DataType,
    /// Whether field is required
    pub required: bool,
    /// Applied transformations
    pub transformations: Vec<String>,
    /// Validation status
    pub validation_status: bool,
}

/// Quality metrics for mapping assessment
#[derive(Debug, Clone)]
pub struct MappingQualityMetrics {
    /// Overall quality score (0.0 to 1.0)
    pub overall_quality: f64,
    /// Required field coverage (0.0 to 1.0)
    pub required_coverage: f64,
    /// Average confidence across all mappings
    pub average_confidence: f64,
    /// Number of validation errors
    pub validation_errors: usize,
    /// Number of warnings
    pub warnings: usize,
    /// Template detection confidence
    pub template_confidence: f64,
}

/// Transformation result
#[derive(Debug, Clone)]
pub struct TransformationResult {
    /// Field name
    pub field_name: String,
    /// Transformation name
    pub transformation_name: String,
    /// Success status
    pub success: bool,
    /// Original value
    pub original_value: serde_json::Value,
    /// Transformed value
    pub transformed_value: Option<serde_json::Value>,
    /// Error message if transformation failed
    pub error_message: Option<String>,
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
            required_coverage: 0.8,
            quality_threshold: 0.75,
            max_validation_errors: 5,
        }
    }
}

impl Default for TemplateInfo {
    fn default() -> Self {
        Self {
            template_id: "unknown".to_string(),
            name: "Unknown Template".to_string(),
            version: "0.0".to_string(),
            confidence: 0.0,
            matched_patterns: Vec::new(),
            missing_patterns: Vec::new(),
            metadata: HashMap::new(),
        }
    }
}
