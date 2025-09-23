// Modified: 2025-09-22

//! Mapping override system for custom column mappings
//!
//! This module provides a comprehensive override system for customizing
//! column mappings in document parsing. It includes:
//!
//! - **Types**: Core data structures and enums for override rules
//! - **Engine**: Main override engine for rule management and resolution
//! - **Context**: Context management for override resolution
//! - **Resolver**: Conflict resolution between multiple matching rules
//! - **Validator**: Validation of override rules for safety and correctness
//!
//! # Example Usage
//!
//! ```rust
//! use crate::validation::overrides::{
//!     MappingOverrideEngine, MappingOverride, OverrideType, OverridePattern,
//!     OverrideScope, OverrideContext
//! };
//! use uuid::Uuid;
//! use chrono::Utc;
//!
//! // Create an override engine
//! let mut engine = MappingOverrideEngine::new();
//!
//! // Create an override rule
//! let override_rule = MappingOverride {
//!     id: Uuid::new_v4(),
//!     name: "Custom Status Mapping".to_string(),
//!     description: "Map 'Status' column to 'current_status' field".to_string(),
//!     rule_type: OverrideType::ExactMatch,
//!     pattern: OverridePattern {
//!         pattern: "Status".to_string(),
//!         case_sensitive: false,
//!         whole_word: true,
//!         regex_flags: None,
//!         fuzzy_threshold: None,
//!         position_constraints: None,
//!     },
//!     target_field: "current_status".to_string(),
//!     priority: 100,
//!     conditions: Vec::new(),
//!     scope: OverrideScope::Global,
//!     created_by: "admin".to_string(),
//!     created_at: Utc::now(),
//!     modified_at: Utc::now(),
//!     active: true,
//!     version: 1,
//!     tags: vec!["status".to_string(), "mapping".to_string()],
//! };
//!
//! // Add the override rule
//! engine.add_override(override_rule).unwrap();
//!
//! // Create a context for resolution
//! let context = OverrideContext::new("inventory".to_string())
//!     .with_file_name("inventory.xlsx".to_string());
//!
//! // Resolve mapping for a column
//! let result = engine.resolve_mapping("Status", "inventory", &context).unwrap();
//! if result.override_applied {
//!     println!("Mapped to: {}", result.target_field.unwrap());
//! }
//! ```

pub mod types;
pub mod engine;
pub mod context;
pub mod resolver;
pub mod validator;

// Re-export all public types for backward compatibility
pub use types::*;
pub use engine::MappingOverrideEngine;
pub use context::OverrideContextBuilder;
pub use resolver::ConflictResolver;
pub use validator::OverrideValidator;

// Re-export commonly used types at the module level
pub use types::{
    MappingOverride,
    OverrideType,
    OverridePattern,
    OverrideCondition,
    OverrideScope,
    OverrideContext,
    OverrideResolutionResult,
    OverrideConflict,
    ConflictType,
    ConflictSeverity,
    ConflictResolutionStrategy,
    OverrideMetrics,
};

#[cfg(test)]
mod tests {
    use super::*;
    use uuid::Uuid;
    use chrono::Utc;

    fn create_test_override() -> MappingOverride {
        MappingOverride {
            id: Uuid::new_v4(),
            name: "Test Override".to_string(),
            description: "Test override for unit tests".to_string(),
            rule_type: OverrideType::ExactMatch,
            pattern: OverridePattern {
                pattern: "test_column".to_string(),
                case_sensitive: false,
                whole_word: true,
                regex_flags: None,
                fuzzy_threshold: None,
                position_constraints: None,
            },
            target_field: "test_field".to_string(),
            priority: 100,
            conditions: Vec::new(),
            scope: OverrideScope::Global,
            created_by: "test_user".to_string(),
            created_at: Utc::now(),
            modified_at: Utc::now(),
            active: true,
            version: 1,
            tags: vec!["test".to_string()],
        }
    }

    #[test]
    fn test_override_engine_creation() {
        let engine = MappingOverrideEngine::new();
        assert_eq!(engine.get_active_overrides().len(), 0);
    }

    #[test]
    fn test_add_override() {
        let mut engine = MappingOverrideEngine::new();
        let override_rule = create_test_override();
        
        let result = engine.add_override(override_rule);
        assert!(result.is_ok());
        assert_eq!(engine.get_active_overrides().len(), 1);
    }

    #[test]
    fn test_context_creation() {
        let context = OverrideContext::new("test_doc".to_string());
        assert_eq!(context.document_type, "test_doc");
        assert!(context.file_name.is_none());
    }

    #[test]
    fn test_context_builder() {
        let context = OverrideContext::builder("test_doc".to_string())
            .file_name("test.xlsx".to_string())
            .user_id("user123".to_string())
            .build();

        assert_eq!(context.document_type, "test_doc");
        assert_eq!(context.file_name, Some("test.xlsx".to_string()));
        assert_eq!(context.user_id, Some("user123".to_string()));
    }

    #[test]
    fn test_conflict_resolver_creation() {
        let resolver = ConflictResolver::new();
        assert_eq!(*resolver.get_strategy(), ConflictResolutionStrategy::HighestPriority);
    }

    #[test]
    fn test_override_validator_creation() {
        let validator = OverrideValidator::new();
        let (cache_size, max_size) = validator.get_cache_stats();
        assert_eq!(cache_size, 0);
        assert_eq!(max_size, 1000);
    }

    #[test]
    fn test_override_validation() {
        let mut validator = OverrideValidator::new();
        let override_rule = create_test_override();
        
        let result = validator.validate_override(&override_rule);
        assert!(result.is_ok());
    }

    #[test]
    fn test_resolve_mapping_no_overrides() {
        let mut engine = MappingOverrideEngine::new();
        let context = OverrideContext::new("test_doc".to_string());
        
        let result = engine.resolve_mapping("test_column", "test_doc", &context).unwrap();
        assert!(!result.override_applied);
        assert!(result.target_field.is_none());
    }

    #[test]
    fn test_resolve_mapping_with_override() {
        let mut engine = MappingOverrideEngine::new();
        let override_rule = create_test_override();
        engine.add_override(override_rule).unwrap();
        
        let context = OverrideContext::new("test_doc".to_string());
        let result = engine.resolve_mapping("test_column", "test_doc", &context).unwrap();
        
        assert!(result.override_applied);
        assert_eq!(result.target_field, Some("test_field".to_string()));
    }

    #[test]
    fn test_cache_key_generation() {
        let context = OverrideContext::new("test_doc".to_string())
            .with_file_name("test.xlsx".to_string())
            .with_user_id("user123".to_string());

        let cache_key = context.cache_key();
        assert!(cache_key.contains("doc:test_doc"));
        assert!(cache_key.contains("file:test.xlsx"));
        assert!(cache_key.contains("user:user123"));
    }

    #[test]
    fn test_override_metrics() {
        let engine = MappingOverrideEngine::new();
        let metrics = engine.get_metrics();
        
        assert_eq!(metrics.total_applications, 0);
        assert_eq!(metrics.successful_matches, 0);
        assert_eq!(metrics.conflicts_detected, 0);
    }
}
