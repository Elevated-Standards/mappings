// Modified: 2025-09-22

//! Excel parsing module
//!
//! This module provides comprehensive Excel document parsing capabilities with
//! specialized support for FedRAMP POA&M templates and general Excel files.

pub mod core;
pub mod poam;
pub mod inventory;
pub mod types;
pub mod validation;

// Re-export main types for convenience
pub use core::ExcelParser;
pub use poam::{PoamParser, PoamItem, PoamParseResult};
pub use inventory::{InventoryParser, Asset, InventoryDocument, AssetType, AssetCategory};
pub use types::*;
pub use validation::ExcelValidator;
