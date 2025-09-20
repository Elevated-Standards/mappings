# Map Columns Using poam_mappings.json Configuration

**Task ID:** bFDm3vJ24BMfayUm56jBpy  
**Component:** 1.3: POA&M Document Processor  
**Status:** Not Started  
**Priority:** High  

## Overview

Use existing POA&M mapping configuration to map Excel columns to OSCAL fields, leveraging the established mapping rules while supporting template variations and customizations.

## Objectives

- Load and apply poam_mappings.json configuration
- Map POA&M Excel columns to OSCAL POA&M fields
- Handle mapping variations and template differences
- Validate mapping completeness and accuracy
- Support custom mapping overrides for POA&M

## Technical Requirements

### Mapping Configuration
1. **POA&M Mapping File Structure**
   - Load existing poam_mappings.json
   - Parse mapping rules and field definitions
   - Support nested field mappings
   - Handle conditional mapping rules

2. **OSCAL POA&M Field Mapping**
   - Map to OSCAL POA&M schema fields
   - Handle OSCAL-specific data structures
   - Support UUID generation and referencing
   - Maintain OSCAL compliance requirements

3. **Template Variation Handling**
   - Support multiple POA&M template formats
   - Handle column name variations
   - Apply fuzzy matching for similar fields
   - Manage template-specific mapping rules

### Core Functionality
1. **Mapping Engine Integration**
   - Integrate with Column Mapping Engine
   - Apply POA&M-specific mapping rules
   - Handle mapping conflicts and ambiguities
   - Support mapping confidence scoring

2. **Field Transformation**
   - Transform Excel data to OSCAL format
   - Handle data type conversions
   - Apply field-specific validation rules
   - Normalize data formats and values

3. **Mapping Validation**
   - Validate mapping completeness
   - Check required field coverage
   - Verify OSCAL schema compliance
   - Generate mapping quality reports

## Implementation Details

### Data Structures
```rust
pub struct PoamColumnMapper {
    mapping_config: PoamMappingConfig,
    base_mapper: ColumnMapper,
    field_transformers: HashMap<String, Box<dyn FieldTransformer>>,
    validator: PoamMappingValidator,
}

pub struct PoamMappingConfig {
    pub version: String,
    pub template_mappings: HashMap<String, TemplateMapping>,
    pub field_mappings: HashMap<String, FieldMapping>,
    pub transformation_rules: Vec<TransformationRule>,
    pub validation_rules: Vec<ValidationRule>,
}

pub struct FieldMapping {
    pub source_column: String,
    pub target_field: String,
    pub data_type: DataType,
    pub required: bool,
    pub transformation: Option<String>,
    pub validation: Option<String>,
    pub oscal_path: String,
}

pub struct PoamMappingResult {
    pub mapped_fields: HashMap<String, Value>,
    pub unmapped_columns: Vec<String>,
    pub missing_required: Vec<String>,
    pub mapping_confidence: f64,
    pub validation_results: Vec<ValidationResult>,
}
```

### POA&M-Specific Mappings
1. **Core POA&M Fields**
   - Unique ID → poam-item.uuid
   - Weakness Name → poam-item.title
   - Description → poam-item.description
   - Severity → poam-item.risk.risk-level
   - Status → poam-item.status

2. **Risk Assessment Fields**
   - Likelihood → poam-item.risk.likelihood
   - Impact → poam-item.risk.impact
   - Risk Rating → poam-item.risk.risk-rating
   - CVSS Score → poam-item.risk.cvss-score

3. **Remediation Fields**
   - Scheduled Completion → poam-item.scheduled-completion-date
   - Actual Completion → poam-item.actual-completion-date
   - Remediation Plan → poam-item.remediation-plan
   - Point of Contact → poam-item.responsible-parties

### Key Features
- **OSCAL Compliance**: Ensure all mappings produce valid OSCAL POA&M
- **Template Flexibility**: Support various POA&M template formats
- **Data Validation**: Comprehensive validation of mapped data
- **Confidence Scoring**: Provide mapping confidence metrics

## Dependencies

- Column Mapping Engine
- POA&M-specific Excel Parser
- OSCAL schema validation
- Existing poam_mappings.json file

## Testing Requirements

- Unit tests for POA&M mapping logic
- Integration tests with real POA&M templates
- OSCAL schema validation tests
- Mapping accuracy and completeness tests
- Performance tests with large POA&M files

## Acceptance Criteria

- [ ] Load and parse poam_mappings.json successfully
- [ ] Map all standard POA&M fields to OSCAL format
- [ ] Handle template variations and column differences
- [ ] Validate mapping completeness and accuracy
- [ ] Generate OSCAL-compliant field mappings
- [ ] Support custom mapping overrides
- [ ] Achieve >95% mapping accuracy for standard templates
- [ ] Pass comprehensive POA&M mapping tests

## Related Tasks

- **Previous:** Implement POA&M-specific Excel parser
- **Next:** Validate severity levels and status values
- **Depends on:** Column Mapping Engine and POA&M parser
- **Enables:** OSCAL POA&M JSON generation

## Notes

- Leverage existing mapping logic and configurations
- Focus on OSCAL POA&M schema compliance
- Implement comprehensive error handling and reporting
- Support for mapping rule versioning and updates
- Consider integration with vulnerability databases for enrichment
