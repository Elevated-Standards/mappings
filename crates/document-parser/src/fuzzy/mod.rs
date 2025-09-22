// Modified: 2025-09-22

//! Advanced fuzzy string matching for column detection (modular implementation)
//!
//! This module provides multiple fuzzy matching algorithms to handle column name
//! variations, typos, and different formatting styles commonly found in FedRAMP documents.
//!
//! The module is organized into several sub-modules:
//! - `types`: Core fuzzy matching type definitions and data structures
//! - `algorithms`: Implementation of various fuzzy matching algorithms
//! - `preprocessing`: Text preprocessing utilities for normalization
//! - `matcher`: Main fuzzy matching implementation that combines algorithms

pub mod types;
pub mod algorithms;
pub mod preprocessing;
pub mod matcher;

// Re-export commonly used types for convenience
pub use types::*;
pub use algorithms::*;
pub use preprocessing::TextPreprocessor;
pub use matcher::FuzzyMatcher;
