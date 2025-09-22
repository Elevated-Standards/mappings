//! Main POA&M column mapper implementation
//! Modified: 2025-01-22

use crate::{Error, Result};
use crate::mapping::ColumnMapper;
use std::path::Path;
use std::collections::HashMap;
use tracing::info;

use super::types::*;

impl PoamColumnMapper {
    /// Create a new POA&M column mapper with default configuration
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
            mapping_result: crate::mapping::MappingResult {
                source_column: "multiple".to_string(),
                target_field: "poam".to_string(),
                confidence: quality_metrics.overall_quality,
                source_type: crate::mapping::engine::types::MappingSourceType::Poam,
                required: true,
                validation: None,
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

    /// Create POA&M-specific field mappings from base results
    fn create_poam_field_mappings(
        &self,
        base_results: &[crate::mapping::MappingResult],
        template_info: &TemplateInfo,
    ) -> Result<Vec<PoamFieldMapping>> {
        let mut field_mappings = Vec::new();

        for base_result in base_results {
            // Look up field mapping configuration
            if let Some(field_config) = self.mapping_config.field_mappings.get(&base_result.target_field) {
                let field_mapping = PoamFieldMapping {
                    source_column: base_result.source_column.clone(),
                    target_field: base_result.target_field.clone(),
                    confidence: base_result.confidence,
                    data_type: field_config.data_type.clone(),
                    required: field_config.required,
                    transformations: field_config.transformations.clone(),
                    validation_status: true, // Will be updated during validation
                };
                field_mappings.push(field_mapping);
            } else {
                // Create default field mapping
                let field_mapping = PoamFieldMapping {
                    source_column: base_result.source_column.clone(),
                    target_field: base_result.target_field.clone(),
                    confidence: base_result.confidence,
                    data_type: crate::validation::DataType::String,
                    required: base_result.required,
                    transformations: Vec::new(),
                    validation_status: true,
                };
                field_mappings.push(field_mapping);
            }
        }

        Ok(field_mappings)
    }

    /// Validate field mappings
    async fn validate_mappings(&self, field_mappings: &[PoamFieldMapping]) -> Result<Vec<crate::validation::ValidationResult>> {
        let mut validation_results = Vec::new();

        for field_mapping in field_mappings {
            if let Some(result) = self.validator.validate_field_mapping(field_mapping).await {
                validation_results.push(result);
            }
        }

        Ok(validation_results)
    }

    /// Apply transformations to field mappings
    fn apply_transformations(&self, field_mappings: &[PoamFieldMapping]) -> Result<Vec<TransformationResult>> {
        let mut transformation_results = Vec::new();

        for field_mapping in field_mappings {
            for transformation_name in &field_mapping.transformations {
                if let Some(transformer) = self.field_transformers.get(transformation_name) {
                    // For now, create a dummy transformation result
                    // In a real implementation, this would apply the actual transformation
                    let result = TransformationResult {
                        field_name: field_mapping.source_column.clone(),
                        transformation_name: transformation_name.clone(),
                        success: true,
                        original_value: serde_json::Value::String("example".to_string()),
                        transformed_value: Some(serde_json::Value::String("transformed_example".to_string())),
                        error_message: None,
                    };
                    transformation_results.push(result);
                }
            }
        }

        Ok(transformation_results)
    }

    /// Calculate quality metrics for the mapping
    fn calculate_quality_metrics(
        &self,
        field_mappings: &[PoamFieldMapping],
        validation_results: &[crate::validation::ValidationResult],
        template_info: &TemplateInfo,
    ) -> MappingQualityMetrics {
        let total_fields = field_mappings.len();
        let required_fields = field_mappings.iter().filter(|f| f.required).count();
        
        let required_coverage = if total_fields == 0 {
            0.0
        } else {
            required_fields as f64 / total_fields as f64
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
        validation_results: &[crate::validation::ValidationResult],
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

impl Default for PoamColumnMapper {
    fn default() -> Self {
        Self::new()
    }
}
