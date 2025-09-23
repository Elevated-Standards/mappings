// Modified: 2025-09-23

//! POA&M validation report generation module
//!
//! This module provides comprehensive validation reporting for POA&M processing results,
//! including processing summaries, detailed validation results, compliance assessments,
//! and quality trend analysis.

pub mod types;
pub mod generator;
pub mod export;
pub mod visualization;

#[cfg(test)]
mod tests;

// Re-export main types for convenience
pub use types::*;
pub use generator::PoamReportGenerator;
pub use export::PoamReportExporter;
pub use visualization::PoamVisualizationEngine;
