// Modified: 2025-01-20

//! # FedRAMP Core
//!
//! Core data models, types, and utilities for FedRAMP compliance automation.
//! This crate provides the foundational components used across the entire platform.

pub mod models;
pub mod types;
pub mod error;
pub mod validation;
pub mod config;
pub mod utils;

// Re-export commonly used types
pub use error::{Error, Result};
pub use models::*;
pub use types::*;

/// Current version of the FedRAMP Core library
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Supported OSCAL version
pub const OSCAL_VERSION: &str = "1.1.2";

/// Supported FedRAMP template versions
pub mod fedramp_versions {
    pub const POAM_V3: &str = "3.0";
    pub const SSP_V4: &str = "4.0";
    pub const INVENTORY_V13: &str = "13.0";
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_version() {
        assert!(!VERSION.is_empty());
    }

    #[test]
    fn test_oscal_version() {
        assert_eq!(OSCAL_VERSION, "1.1.2");
    }
}
