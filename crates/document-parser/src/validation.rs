//! Document validation and quality assessment module
//!
//! This module provides comprehensive validation capabilities for document parsing,
//! including field validation, data type checking, quality assessment, confidence scoring,
//! mapping overrides, and detailed reporting.
//!
//! The module is organized into several submodules:
//! - `types`: Basic validation types and enums
//! - `rules`: Validation rules and configurations
//! - `confidence`: Confidence scoring system for mapping validation
//! - `overrides`: Mapping override system for custom column mappings
//! - `reports`: Report generation system for mapping validation
//! - `validators`: Main validator implementations
//! - `tests`: Comprehensive test suite

// Submodule declarations
pub mod types;
pub mod rules;
pub mod confidence;
pub mod overrides;
pub mod reports;
pub mod validators;
pub mod poam_validator;

#[cfg(test)]
mod tests;

#[cfg(test)]
pub mod poam_validator_tests;

// Re-export public API from submodules
pub use types::*;
pub use rules::{
    ValidationRule, ColumnValidationRule, ConditionalRequirement, ValidationType, DataType,
    ThresholdConfig, ValidationRuleSet, ScoringConfig, HistoricalMappings, HistoricalMapping,
    UserFeedback, AccuracyStats,
};
pub use confidence::{
    ConfidenceScorer, ConfidenceFactor, MappingConfidence, ThresholdStatus,
    ConfidenceRecommendation, RecommendationType, RiskFactor, RiskType, RiskSeverity,
    ConfidenceExplanation, FactorContribution, WeightedConfidenceCalculation,
    ConfidenceAdjustment, AdjustmentType, MappingContext,
};
pub use overrides::{
    MappingOverrideEngine, MappingOverride, OverrideType, OverridePattern, PositionConstraints,
    OverrideCondition, ConditionType, ConditionOperator, OverrideScope, ConflictResolver,
    ConflictResolutionStrategy, OverrideValidator, OverrideMetrics, OverrideResolutionResult,
    OverrideConflict, ConflictType, ConflictSeverity, OverrideContext, ConflictResolution,
};
pub use reports::{
    MappingReportGenerator, ReportConfig, MappingReport, ReportType, ReportFormat,
    DocumentInfo, MappingSummary, FieldMappingResult, MappingAlternative, MappingIssue,
    DataQualityAssessment, QualityMetrics, Recommendation, ValidationSummary,
    ValidationFailureInfo, ValidationPerformanceMetrics, SlowValidationInfo,
    OverrideSummary, OverrideUsageInfo, OverridePerformanceMetrics, ProcessingMetrics,
    MemoryUsageMetrics, ThroughputMetrics, TrendAnalysis, TimePeriod, QualityTrends,
    PerformanceTrends, IssueTrends, HistoricalQualityScore, HistoricalPerformanceData,
    CommonIssueInfo, IssueRateDataPoint, TrendRecommendation, CachedReport,
    HistoricalReportData, ReportGenerationMetrics,
};
pub use validators::{ColumnValidator, DocumentValidator};
pub use poam_validator::{
    PoamValidator, PoamValidationConfig, PoamSeverity, PoamStatus, ValidationMode,
    BusinessRule, RuleCondition, RuleAction, LogicalOperator, CustomValidationRule,
    PerformanceSettings, PoamValidationResult, ValidationError, ValidationWarning,
    ValidationSuggestion, FieldValidationResult, BusinessRuleResult,
    SeverityValidator, StatusValidator, BusinessRuleValidator, CrossFieldValidator, CrossFieldRule,
};

// Convenience functions for common validation operations

/// Create a new document validator with default settings
pub fn create_document_validator() -> DocumentValidator {
    DocumentValidator::new()
}

/// Create a new column validator with mapping configuration
pub fn create_column_validator(mapping_config: crate::mapping::MappingConfiguration) -> ColumnValidator {
    ColumnValidator::new(mapping_config)
}

/// Create a new confidence scorer with default configuration
pub fn create_confidence_scorer() -> ConfidenceScorer {
    ConfidenceScorer::new()
}

/// Create a new mapping override engine
pub fn create_override_engine() -> MappingOverrideEngine {
    MappingOverrideEngine::new()
}

/// Create a new mapping report generator with default configuration
pub fn create_report_generator() -> MappingReportGenerator {
    MappingReportGenerator::new()
}

// Module-level constants for common validation thresholds
pub const DEFAULT_HIGH_CONFIDENCE_THRESHOLD: f64 = 0.9;
pub const DEFAULT_MEDIUM_CONFIDENCE_THRESHOLD: f64 = 0.7;
pub const DEFAULT_LOW_CONFIDENCE_THRESHOLD: f64 = 0.5;
pub const DEFAULT_MIN_ACCEPTABLE_THRESHOLD: f64 = 0.6;
pub const DEFAULT_QUALITY_THRESHOLD: f64 = 0.8;
pub const DEFAULT_PERFORMANCE_TARGET_MS: u64 = 100;

/// Validation system information and utilities
pub struct ValidationSystemInfo;

impl ValidationSystemInfo {
    /// Get version information for the validation system
    pub fn version() -> &'static str {
        env!("CARGO_PKG_VERSION")
    }
    
    /// Get supported validation types
    pub fn supported_validation_types() -> Vec<ValidationType> {
        vec![
            ValidationType::Presence,
            ValidationType::DataType,
            ValidationType::Enumeration,
            ValidationType::Pattern,
            ValidationType::Range,
            ValidationType::DateFormat,
            ValidationType::Email,
            ValidationType::Url,
            ValidationType::IpAddress,
            ValidationType::Uuid,
        ]
    }
    
    /// Get supported data types
    pub fn supported_data_types() -> Vec<DataType> {
        vec![
            DataType::String,
            DataType::Integer,
            DataType::Float,
            DataType::Boolean,
            DataType::Date,
            DataType::DateTime,
            DataType::Uuid,
            DataType::Email,
            DataType::Url,
            DataType::IpAddress,
            DataType::Object,
            DataType::Array,
            DataType::Any,
        ]
    }
    
    /// Get supported report formats
    pub fn supported_report_formats() -> Vec<ReportFormat> {
        vec![
            ReportFormat::Html,
            ReportFormat::Json,
            ReportFormat::Csv,
            ReportFormat::Markdown,
            ReportFormat::Pdf,
        ]
    }
}
