//! POA&M column mapping module
//! Modified: 2025-01-22
//!
//! This module provides specialized column mapping functionality for POA&M Excel templates,
//! including template detection, field mapping, data transformation, and validation.

pub mod types;
pub mod mapper;
pub mod detector;
pub mod validator;

// Re-export all public types and traits for backward compatibility
pub use types::*;
pub use mapper::*;
pub use detector::*;
pub use validator::*;
