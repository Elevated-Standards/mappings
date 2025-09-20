# Load Mapping Configurations from JSON Files

**Task ID:** 43zPydvyCC5r6hsyAaNype  
**Component:** 1.2: Column Mapping Engine  
**Status:** Not Started  
**Priority:** High  

## Overview

Load and parse existing mapping files (inventory_mappings.json, poam_mappings.json, ssp_sections.json) to enable dynamic column detection and mapping functionality.

## Objectives

- Load mapping configurations from existing JSON files
- Parse and validate mapping file structure
- Create in-memory mapping structures for efficient lookup
- Support hot-reloading of mapping configurations
- Implement error handling for malformed mapping files

## Technical Requirements

### Input Files
- `inventory_mappings.json` - Asset inventory column mappings
- `poam_mappings.json` - POA&M document column mappings  
- `ssp_sections.json` - SSP section identification keywords
- `_controls.json` - Control framework mappings
- `_document.json` - Document structure definitions

### Core Functionality
1. **JSON File Loading**
   - Async file reading with proper error handling
   - Support for relative and absolute file paths
   - Validation of JSON structure and required fields

2. **Mapping Structure Creation**
   - Parse column mapping definitions
   - Create efficient lookup tables and indices
   - Support for nested mapping hierarchies

3. **Configuration Validation**
   - Validate mapping file schema compliance
   - Check for required fields and data types
   - Detect and report configuration conflicts

4. **Hot-Reload Support**
   - File system watching for configuration changes
   - Atomic configuration updates without service interruption
   - Rollback capability for invalid configurations

## Implementation Details

### Data Structures
```rust
pub struct MappingConfiguration {
    pub inventory_mappings: InventoryMappings,
    pub poam_mappings: PoamMappings,
    pub ssp_sections: SspSections,
    pub controls: ControlMappings,
    pub documents: DocumentStructures,
}

pub struct ColumnMapping {
    pub source_column: String,
    pub target_field: String,
    pub data_type: DataType,
    pub required: bool,
    pub validation_rules: Vec<ValidationRule>,
    pub aliases: Vec<String>,
}
```

### Key Features
- **Efficient Lookup**: Hash-based column name resolution
- **Fuzzy Matching Support**: Prepare data for fuzzy string matching
- **Validation Rules**: Load field validation requirements
- **Metadata Preservation**: Maintain source file information

## Dependencies

- `serde_json` for JSON parsing
- `tokio::fs` for async file operations
- `notify` for file system watching
- `thiserror` for error handling

## Testing Requirements

- Unit tests for each mapping file type
- Integration tests with actual mapping files
- Error handling tests for malformed JSON
- Performance tests for large mapping files
- Hot-reload functionality tests

## Acceptance Criteria

- [ ] Successfully load all existing mapping files
- [ ] Parse and validate JSON structure
- [ ] Create efficient in-memory mapping structures
- [ ] Implement comprehensive error handling
- [ ] Support configuration hot-reloading
- [ ] Achieve sub-100ms loading time for mapping files
- [ ] Pass all unit and integration tests

## Related Tasks

- **Next:** Implement fuzzy string matching for column detection
- **Depends on:** Excel Parser Engine completion
- **Enables:** Column validation and mapping confidence scoring

## Notes

- Existing mapping files are already well-structured
- Focus on preserving existing mapping logic
- Consider backward compatibility for future mapping file updates
- Implement comprehensive logging for debugging mapping issues
