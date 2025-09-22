//! Core mapping engine and optimization structures
//! Modified: 2025-01-22
//!
//! This module contains the main mapping engine logic including the ColumnMapper,
//! OptimizedMappingLookup, and related functionality for efficient column mapping.
//!
//! The engine is split into several focused modules:
//! - `types`: Core type definitions and data structures
//! - `lookup`: Optimized lookup functionality for fast column mapping
//! - `mapper`: Main ColumnMapper implementation
//! - `tests`: Comprehensive test suite

// Module declarations
pub mod types;
pub mod lookup;
pub mod mapper;

#[cfg(test)]
pub mod tests;

// Re-export main types for backward compatibility
pub use types::{
    OptimizedMappingLookup,
    MappingEntry,
    MappingSourceType,
    FuzzyCandidate,
    ValidationRule,
    ValidationType,
    MappingResult,
    ColumnMapper,
    MappingStatistics,
    MappingEngineConfig,
    NormalizationRule,
};

// Re-export main functionality
pub use mapper::*;
pub use lookup::*;
