# Create Column Validation Against Required Fields

**Task ID:** uKSY6YHYYcxHABer3984Vf
**Component:** 1.2: Column Mapping Engine
**Status:** Completed
**Priority:** High

## Overview

Validate detected columns against required field definitions in mapping files to ensure all mandatory fields are present and properly mapped for successful OSCAL conversion.

## Objectives

- Validate column mappings against required field definitions
- Identify missing mandatory columns
- Check data type compatibility
- Validate field constraints and enumeration values
- Generate detailed validation reports

## Technical Requirements

### Validation Types
1. **Required Field Validation**
   - Check presence of all mandatory columns
   - Validate against mapping file requirements
   - Support conditional requirements based on document type

2. **Data Type Validation**
   - Verify column data types match expected types
   - Handle type coercion where appropriate
   - Validate format constraints (dates, emails, URLs)

3. **Enumeration Validation**
   - Check values against allowed enumerations
   - Support case-insensitive matching
   - Handle common variations and aliases

4. **Cross-Field Validation**
   - Validate relationships between fields
   - Check referential integrity
   - Ensure logical consistency

### Core Functionality
1. **Validation Engine**
   - Rule-based validation system
   - Configurable validation severity levels
   - Support for custom validation rules

2. **Error Reporting**
   - Detailed error messages with suggestions
   - Severity classification (Error, Warning, Info)
   - Actionable recommendations for fixes

3. **Validation Context**
   - Document type-specific validation
   - Version-aware validation rules
   - Environment-specific requirements

## Implementation Details

### Data Structures
```rust
pub struct ColumnValidator {
    validation_rules: HashMap<String, Vec<ValidationRule>>,
    required_fields: HashMap<DocumentType, Vec<RequiredField>>,
    type_validators: HashMap<DataType, Box<dyn TypeValidator>>,
    enumeration_values: HashMap<String, Vec<String>>,
}

pub struct ValidationResult {
    pub is_valid: bool,
    pub errors: Vec<ValidationError>,
    pub warnings: Vec<ValidationWarning>,
    pub missing_required: Vec<RequiredField>,
    pub type_mismatches: Vec<TypeMismatch>,
    pub suggestions: Vec<ValidationSuggestion>,
}

pub struct RequiredField {
    pub name: String,
    pub data_type: DataType,
    pub description: String,
    pub alternatives: Vec<String>,
    pub conditional: Option<ConditionalRequirement>,
}
```

### Validation Rules
- **Mandatory Fields**: Asset Name, Asset Type, IP Address (for inventory)
- **POA&M Fields**: Weakness Name, Severity, Status, Scheduled Completion Date
- **SSP Fields**: System Name, Security Categorization, System Description
- **Data Formats**: ISO dates, valid IP addresses, email formats
- **Enumerations**: Asset types, severity levels, status values

### Key Features
- **Hierarchical Validation**: Field-level, record-level, document-level
- **Contextual Rules**: Different rules for different document types
- **Suggestion Engine**: Recommend fixes for validation failures
- **Batch Validation**: Validate multiple documents efficiently

## Dependencies

- `validator` crate for common validation patterns
- `regex` for format validation
- `chrono` for date validation
- `ipnet` for IP address validation

## Testing Requirements

- Unit tests for each validation rule type
- Integration tests with real document samples
- Edge case testing for boundary conditions
- Performance tests with large datasets
- Validation rule configuration tests

## Acceptance Criteria

- [x] Validate all required fields per document type
- [x] Check data type compatibility and constraints
- [x] Validate enumeration values and formats
- [x] Generate comprehensive validation reports
- [x] Provide actionable error messages and suggestions
- [x] Support configurable validation severity levels
- [x] Achieve <50ms validation time per document
- [x] Pass all validation test scenarios

## Related Tasks

- **Previous:** Implement fuzzy string matching for column detection
- **Next:** Build mapping confidence scoring
- **Depends on:** Load mapping configurations from JSON files
- **Enables:** Quality assurance and validation reporting

## Notes

- Leverage existing validation logic from mapping files
- Focus on FedRAMP-specific requirements and constraints
- Implement comprehensive error messaging for user guidance
- Consider validation rule versioning for future updates
- Support for custom validation rules per organization
