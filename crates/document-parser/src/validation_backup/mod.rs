// Modified: 2025-09-22

//! Validation backup module organization
//!
//! This module organizes the validation backup functionality into logical sub-modules:
//! - `types`: Core validation types and data structures
//! - `confidence`: Confidence scoring system for mappings
//! - `overrides`: Override engine and conflict resolution
//! - `reports`: Report generation and analysis
//! - `core`: Main validation implementation

pub mod types;
pub mod confidence;
pub mod overrides;
pub mod reports;
pub mod core;

// Re-export commonly used types for convenience
pub use types::*;
pub use confidence::{ConfidenceScorer, MappingConfidence, ThresholdStatus};
pub use overrides::{MappingOverrideEngine, OverrideResolutionResult, ConflictResolver};
pub use reports::{MappingReportGenerator, MappingReport, ReportType, ReportFormat};
pub use core::{ColumnValidator, DocumentValidator};
