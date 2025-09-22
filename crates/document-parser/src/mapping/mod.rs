//! Column mapping and field detection for document parsing
//!
//! This module provides functionality to map document columns to OSCAL fields
//! using fuzzy matching and configuration-based mapping rules.
//!
//! The module is organized into several sub-modules:
//! - `config`: Basic configuration structures
//! - `inventory`: Inventory-specific mappings
//! - `poam`: POA&M-specific mappings  
//! - `ssp`: SSP-specific mappings
//! - `control_document`: Control and document mappings
//! - `loader`: Configuration loading and hot-reload
//! - `engine`: Core mapping engine and optimization
//! - `validation`: Configuration validation

pub mod config;
pub mod inventory;
pub mod poam;
pub mod poam_column_mapper;
pub mod poam_transformers;
pub mod date_converter;
pub mod date_parsers;
pub mod ssp;
pub mod control_document;
pub mod loader;
pub mod engine;
pub mod validation;

#[cfg(test)]
pub mod poam_column_mapper_tests;

// Re-export main public APIs for backward compatibility
pub use config::{
    ColumnMapping, 
    MappingConfiguration, 
    ValidationRules, 
    ComponentGrouping, 
    GroupingStrategy,
    ComponentTypeMapping,
    SecurityMappings,
    ImpactMapping,
    RiskFactor,
    ControlInheritance,
    InheritanceMapping,
    LoadingMetrics,
};

pub use inventory::{
    InventoryMappings,
    InventoryColumnMappings,
    InventoryColumnMapping,
};

pub use poam::{
    PoamMappings,
    PoamColumnMappings,
    PoamColumnMapping,
    PoamValidationRules,
    RiskMappings,
    RiskLevel,
    FindingMappings,
    OriginType,
    MilestoneProcessing,
    MilestonePatterns,
    MultipleMilestones,
    MilestoneFormat,
    QualityChecks,
    RequiredFieldCompleteness,
    DataConsistency,
    ControlValidation,
};

pub use poam_column_mapper::{
    PoamColumnMapper,
    PoamMappingConfig,
    PoamMappingResult,
    TemplateInfo,
    PoamFieldMapping,
    MappingQualityMetrics,
    TransformationResult,
    PoamTemplateDetector,
    PoamMappingValidator,
};

pub use poam_transformers::{
    SeverityTransformer,
    StatusTransformer,
    DateTransformer,
    ControlIdTransformer,
    TextNormalizerTransformer,
    ListTransformer,
};

pub use date_converter::{
    DateConverter,
    DateParsingResult,
    DateParser,
    TimezoneConfig,
    DateFormatPreferences,
    DateValidationRule,
    DateRuleType,
    DateConstraint,
    DateParseError,
    DateWarning,
};

pub use ssp::{
    SspSections,
    SectionMappings,
    SectionMapping,
    ControlExtraction,
    ExtractionPattern,
    TableMappings,
    ResponsibilityMatrix,
    ResponsibilityColumns,
    InventorySummary,
    InventorySummaryColumns,
};

pub use control_document::{
    ControlMappings,
    ControlMetadata,
    ControlMapping,
    ControlEnhancement,
    ControlSource,
    DocumentStructures,
    DocumentMetadata,
    DocumentSection,
    DocumentTable,
    DocumentTableSource,
    DocumentSectionSource,
};

pub use loader::{
    MappingConfigurationLoader,
    HotReloadHandler,
};

pub use engine::{
    OptimizedMappingLookup,
    MappingEntry,
    MappingSourceType,
    FuzzyCandidate,
    ValidationRule,
    ValidationType,
    MappingResult,
    ColumnMapper,
};
