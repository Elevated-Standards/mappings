//! Validation and mapping report generation module
//! Modified: 2025-01-22
//!
//! This module provides comprehensive reporting functionality for validation results,
//! mapping analysis, and performance metrics with support for multiple output formats.

pub mod types;
pub mod metrics;
pub mod trends;
pub mod generator;

// Re-export all public types and traits for backward compatibility
pub use types::*;
pub use metrics::*;
pub use trends::*;
pub use generator::*;
