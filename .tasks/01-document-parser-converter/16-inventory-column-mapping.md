# Map Columns Using inventory_mappings.json

**Task ID:** jQe7zTNnmSLHpx2yhFQ5yn  
**Component:** 1.4: Inventory Document Processor  
**Status:** Not Started  
**Priority:** High  

## Overview

Use existing inventory mapping configuration to map Excel columns to OSCAL component fields, enabling accurate transformation of inventory data to OSCAL component definitions.

## Objectives

- Load and apply inventory_mappings.json configuration
- Map inventory Excel columns to OSCAL component fields
- Handle asset type-specific mapping variations
- Support multi-worksheet inventory processing
- Validate mapping completeness and accuracy

## Technical Requirements

### Mapping Configuration
1. **Inventory Mapping Structure**
   - Load existing inventory_mappings.json
   - Parse asset type-specific mappings
   - Support nested component structures
   - Handle conditional mapping rules

2. **OSCAL Component Mapping**
   - Map to OSCAL component definition schema
   - Handle component hierarchies and relationships
   - Support control implementation mappings
   - Maintain OSCAL compliance requirements

3. **Asset Type Variations**
   - Hardware-specific field mappings
   - Software component mappings
   - Network device configurations
   - Virtual asset representations

### Core Functionality
1. **Multi-Worksheet Mapping**
   - Process multiple inventory worksheets
   - Correlate data across worksheets
   - Handle cross-references and relationships
   - Maintain data consistency

2. **Asset-Specific Transformations**
   - Apply asset type-specific rules
   - Transform technical specifications
   - Handle vendor-specific data formats
   - Normalize asset attributes

3. **Component Relationship Mapping**
   - Map asset dependencies
   - Process component hierarchies
   - Handle service relationships
   - Support network topology

## Implementation Details

### Data Structures
```rust
pub struct InventoryColumnMapper {
    mapping_config: InventoryMappingConfig,
    base_mapper: ColumnMapper,
    asset_transformers: HashMap<AssetType, Box<dyn AssetTransformer>>,
    relationship_mapper: RelationshipMapper,
}

pub struct InventoryMappingConfig {
    pub version: String,
    pub asset_type_mappings: HashMap<AssetType, AssetMapping>,
    pub component_mappings: HashMap<String, ComponentMapping>,
    pub relationship_mappings: Vec<RelationshipMapping>,
    pub validation_rules: Vec<ValidationRule>,
}

pub struct ComponentMapping {
    pub source_fields: Vec<String>,
    pub target_component: String,
    pub component_type: ComponentType,
    pub properties: Vec<PropertyMapping>,
    pub control_implementations: Vec<ControlImplementationMapping>,
}

pub struct InventoryMappingResult {
    pub components: Vec<Component>,
    pub relationships: Vec<ComponentRelationship>,
    pub unmapped_assets: Vec<String>,
    pub mapping_confidence: f64,
    pub validation_results: Vec<ValidationResult>,
}
```

### Asset-Specific Mappings
1. **Hardware Components**
   - Asset ID → component.uuid
   - Asset Name → component.title
   - Hardware Type → component.type
   - Specifications → component.props
   - Location → component.props.location

2. **Software Components**
   - Software Name → component.title
   - Version → component.props.version
   - Vendor → component.responsible-parties
   - License → component.props.license
   - Installation Path → component.props.installation-path

3. **Network Components**
   - Device Name → component.title
   - IP Address → component.props.ip-address
   - MAC Address → component.props.mac-address
   - Network Segment → component.props.network-segment
   - Ports/Protocols → component.protocols

### Key Features
- **Multi-Asset Support**: Handle diverse asset types and mappings
- **Relationship Preservation**: Maintain asset relationships in OSCAL
- **Validation Integration**: Comprehensive mapping validation
- **Template Flexibility**: Support various inventory formats

## Dependencies

- Column Mapping Engine
- Inventory-specific Excel Parser
- OSCAL component schema definitions
- Existing inventory_mappings.json file

## Testing Requirements

- Unit tests for inventory mapping logic
- Integration tests with real inventory templates
- Asset type-specific mapping validation
- OSCAL component schema compliance tests
- Performance tests with large inventory files

## Acceptance Criteria

- [ ] Load and parse inventory_mappings.json successfully
- [ ] Map all standard inventory fields to OSCAL components
- [ ] Handle asset type-specific mapping variations
- [ ] Process multi-worksheet inventory structures
- [ ] Generate OSCAL-compliant component mappings
- [ ] Support asset relationship preservation
- [ ] Achieve >95% mapping accuracy for standard templates
- [ ] Pass comprehensive inventory mapping tests

## Related Tasks

- **Previous:** Implement inventory-specific Excel parser
- **Next:** Validate asset types and environments
- **Depends on:** Column Mapping Engine and inventory parser
- **Enables:** OSCAL component JSON generation

## Notes

- Leverage existing mapping configurations and logic
- Focus on OSCAL component definition compliance
- Support for complex asset relationships and hierarchies
- Consider integration with asset management systems
- Plan for mapping rule evolution and versioning
