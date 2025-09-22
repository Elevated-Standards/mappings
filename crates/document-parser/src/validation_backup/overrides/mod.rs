// Modified: 2025-09-22

//! Mapping override engine and conflict resolution
//!
//! This module provides functionality for managing mapping overrides,
//! resolving conflicts, and validating override rules.

pub mod types;
pub mod engine;
pub mod context;
pub mod resolver;
pub mod validator;

// Re-export all public types for backward compatibility
pub use types::*;
pub use engine::MappingOverrideEngine;
pub use context::*;
pub use resolver::{ConflictResolver, ConflictAnalysis};
pub use validator::{OverrideValidator, CacheStats};
