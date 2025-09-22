//! POA&M Excel parsing module
//! Modified: 2025-01-22
//!
//! This module provides specialized parsing functionality for FedRAMP POA&M Excel templates,
//! including template detection, field mapping, business rule validation, and data enrichment.

pub mod types;
pub mod parser;
pub mod detector;
pub mod mapper;
pub mod validator;
pub mod enricher;

// Re-export all public types and traits for backward compatibility
pub use types::*;
pub use parser::*;
pub use detector::*;
pub use mapper::*;
pub use validator::*;
pub use enricher::*;
