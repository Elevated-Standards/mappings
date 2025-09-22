// Modified: 2025-09-22

//! Markdown document parsing module (modular implementation)
//!
//! This module provides comprehensive parsing for Markdown documents (.md)
//! with structure extraction, content analysis, and SSP-specific processing.
//!
//! The module is organized into several sub-modules:
//! - `types`: Core Markdown type definitions and data structures
//! - `parser`: Main Markdown parser implementation with DocumentParser trait
//! - `extractor`: Content extraction functionality for text, tables, links, etc.
//! - `analyzer`: Structure analysis functionality for headings, sections, and outlines
//! - `renderer`: Custom rendering functionality for HTML and plain text
//! - `validation`: Validation logic for document structure and content quality
//! - `utils`: Utility functions for frontmatter extraction and metadata processing

pub mod types;
pub mod parser;
pub mod extractor;
pub mod analyzer;
pub mod renderer;
pub mod validation;
pub mod utils;

// Re-export commonly used types for convenience
pub use types::*;
pub use parser::MarkdownParser;
pub use extractor::MarkdownExtractor;
pub use analyzer::MarkdownStructureAnalyzer;
pub use renderer::CustomRenderer;
pub use validation::{validate_markdown_content, calculate_quality_score};
pub use utils::{extract_frontmatter, extract_metadata, extract_first_heading};
