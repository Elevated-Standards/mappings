// Modified: 2025-09-20

//! # Control Mapping Engine
//!
//! Cross-reference controls between different frameworks (NIST 800-53 Rev 5, NIST 800-171 R3, CIS).

pub mod catalog;
pub mod mapping;
pub mod nist;
pub mod cis;
pub mod fedramp;
pub mod quality;

pub use catalog::*;
pub use mapping::*;
