// Modified: 2025-01-20

//! # Gap Analysis Tool
//!
//! Automatically identify missing controls and implementation gaps across security frameworks.

pub mod engine;
pub mod baseline;
pub mod prioritization;
pub mod remediation;

pub use engine::*;
pub use baseline::*;
