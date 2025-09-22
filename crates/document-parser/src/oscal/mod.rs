// Modified: 2025-09-22

//! OSCAL output generation and validation (modular implementation)
//!
//! This module provides comprehensive functionality to generate valid OSCAL JSON documents
//! from parsed document content with proper validation, metadata, and schema compliance.
//!
//! The module is organized into several sub-modules:
//! - `types`: Core OSCAL type definitions and data structures
//! - `documents`: Top-level OSCAL document containers
//! - `processors`: Business logic for transforming data into OSCAL structures
//! - `generator`: Main OSCAL generator orchestrating document creation
//! - `validation`: Schema validation and structural validation
//! - `utils`: Utility functions for UUID generation, metadata building, etc.

pub mod types;
pub mod documents;
pub mod processors;
pub mod generator;
pub mod validation;
pub mod utils;

// Re-export commonly used types for convenience
pub use types::*;
pub use documents::*;
pub use processors::{PoamItemProcessor, RiskProcessor, ObservationProcessor};
pub use generator::OscalGenerator;
pub use validation::OscalSchemaValidator;
pub use utils::{UuidGenerator, MetadataBuilder, OscalUtils};
