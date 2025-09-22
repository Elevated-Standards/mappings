// Modified: 2025-01-22

//! POA&M validation module
//!
//! This module provides comprehensive validation for POA&M (Plan of Action and Milestones)
//! documents including field validation, business rules, and cross-field validation.

pub mod types;
pub mod field_validation;
pub mod business_rules;
pub mod cross_field;
pub mod core;

// Re-export main types and structs for backward compatibility
pub use types::{
    PoamValidationConfig, PoamValidationResult, PoamSeverity, PoamStatus, ValidationMode,
    BusinessRule, RuleCondition, RuleAction, LogicalOperator, CustomValidationRule,
    PerformanceSettings, ValidationError, ValidationWarning, ValidationSuggestion,
    FieldValidationResult, BusinessRuleResult, ValidationPerformanceMetrics, CrossFieldRule
};

pub use field_validation::{SeverityValidator, StatusValidator};
pub use business_rules::BusinessRuleValidator;
pub use cross_field::CrossFieldValidator;
pub use core::PoamValidator;
