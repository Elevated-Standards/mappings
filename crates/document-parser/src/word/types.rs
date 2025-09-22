// Modified: 2025-01-22

//! Type definitions for Word document parsing
//!
//! This module contains all the core types, structs, and data structures
//! used throughout the Word document parsing system.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Configuration for DOCX parser
#[derive(Debug, Clone)]
pub struct DocxParserConfig {
    /// Extract images and embedded objects
    pub extract_images: bool,
    /// Extract table data
    pub extract_tables: bool,
    /// Extract headers and footers
    pub extract_headers_footers: bool,
    /// Preserve formatting information
    pub preserve_formatting: bool,
    /// Maximum depth for nested structures
    pub max_nesting_depth: usize,
    /// Enable structure analysis
    pub analyze_structure: bool,
}

impl Default for DocxParserConfig {
    fn default() -> Self {
        Self {
            extract_images: true,
            extract_tables: true,
            extract_headers_footers: true,
            preserve_formatting: true,
            max_nesting_depth: 10,
            analyze_structure: true,
        }
    }
}

/// Parsed DOCX document structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocxDocument {
    /// Document metadata
    pub metadata: DocumentMetadata,
    /// Document structure
    pub structure: DocumentStructure,
    /// Document content
    pub content: DocumentContent,
    /// Tables in the document
    pub tables: Vec<DocumentTable>,
    /// Images in the document
    pub images: Vec<DocumentImage>,
    /// Document relationships
    pub relationships: Vec<DocumentRelationship>,
}

/// Document metadata information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentMetadata {
    /// Document title
    pub title: Option<String>,
    /// Document author
    pub author: Option<String>,
    /// Document subject
    pub subject: Option<String>,
    /// Document description
    pub description: Option<String>,
    /// Creation date
    pub created: Option<String>,
    /// Last modified date
    pub modified: Option<String>,
    /// Document version
    pub version: Option<String>,
    /// Document language
    pub language: Option<String>,
    /// Custom properties
    pub custom_properties: HashMap<String, String>,
}

/// Document structure hierarchy
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentStructure {
    /// Document sections
    pub sections: Vec<DocumentSection>,
    /// Document headings
    pub headings: Vec<DocumentHeading>,
    /// Table of contents
    pub table_of_contents: Option<TableOfContents>,
    /// Cross references
    pub cross_references: Vec<CrossReference>,
    /// Bookmarks
    pub bookmarks: Vec<Bookmark>,
}

/// Document section
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentSection {
    /// Section identifier
    pub id: String,
    /// Section title
    pub title: String,
    /// Section level (1-6)
    pub level: usize,
    /// Section content
    pub content: Vec<DocumentElement>,
    /// Subsections
    pub subsections: Vec<DocumentSection>,
    /// Page range
    pub page_range: Option<PageRange>,
}

/// Document heading
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentHeading {
    /// Heading text
    pub text: String,
    /// Heading level (1-6)
    pub level: usize,
    /// Heading style
    pub style: Option<String>,
    /// Heading location
    pub location: Option<DocumentLocation>,
}

/// Document content container
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentContent {
    /// Full text content
    pub text: String,
    /// Document elements
    pub elements: Vec<DocumentElement>,
    /// Word count
    pub word_count: usize,
    /// Character count
    pub character_count: usize,
    /// Paragraph count
    pub paragraph_count: usize,
}

/// Document element (paragraph, heading, etc.)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentElement {
    /// Element type
    pub element_type: ElementType,
    /// Element content
    pub content: String,
    /// Formatting information
    pub formatting: Option<ElementFormatting>,
    /// Element location
    pub location: DocumentLocation,
}

/// Type of document element
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ElementType {
    /// Paragraph
    Paragraph,
    /// Heading
    Heading,
    /// List item
    ListItem,
    /// Table
    Table,
    /// Image
    Image,
    /// Hyperlink
    Hyperlink,
    /// Footnote
    Footnote,
    /// Endnote
    Endnote,
}

/// Element formatting information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ElementFormatting {
    /// Bold text
    pub bold: bool,
    /// Italic text
    pub italic: bool,
    /// Underlined text
    pub underline: bool,
    /// Font name
    pub font_name: Option<String>,
    /// Font size
    pub font_size: Option<f64>,
    /// Text color
    pub color: Option<String>,
    /// Background color
    pub background_color: Option<String>,
}

/// Document location information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentLocation {
    /// Page number
    pub page: Option<usize>,
    /// Paragraph number
    pub paragraph: Option<usize>,
    /// Character offset
    pub character_offset: Option<usize>,
}

/// Document table
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentTable {
    /// Table identifier
    pub id: String,
    /// Table title/caption
    pub title: Option<String>,
    /// Table headers
    pub headers: Vec<String>,
    /// Table rows
    pub rows: Vec<Vec<String>>,
    /// Table location
    pub location: Option<DocumentLocation>,
    /// Table formatting
    pub formatting: Option<TableFormatting>,
}

/// Table formatting information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TableFormatting {
    /// Border style
    pub border_style: Option<String>,
    /// Cell padding
    pub cell_padding: Option<f64>,
    /// Table width
    pub width: Option<f64>,
    /// Column widths
    pub column_widths: Vec<f64>,
}

/// Document image
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentImage {
    /// Image identifier
    pub id: String,
    /// Image title/alt text
    pub title: Option<String>,
    /// Image description
    pub description: Option<String>,
    /// Image format (png, jpg, etc.)
    pub format: Option<String>,
    /// Image size in bytes
    pub size: Option<usize>,
    /// Image dimensions
    pub dimensions: Option<ImageDimensions>,
    /// Image location
    pub location: Option<DocumentLocation>,
}

/// Image dimensions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImageDimensions {
    /// Width in pixels
    pub width: u32,
    /// Height in pixels
    pub height: u32,
}

/// Document relationship (hyperlinks, cross-references, etc.)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentRelationship {
    /// Relationship identifier
    pub id: String,
    /// Relationship type
    pub relationship_type: RelationshipType,
    /// Source location
    pub source: DocumentLocation,
    /// Target location or URL
    pub target: String,
    /// Relationship description
    pub description: Option<String>,
}

/// Type of document relationship
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RelationshipType {
    /// Hyperlink to external URL
    Hyperlink,
    /// Cross-reference to another part of document
    CrossReference,
    /// Footnote reference
    FootnoteReference,
    /// Endnote reference
    EndnoteReference,
    /// Bookmark reference
    BookmarkReference,
}

/// Table of contents
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TableOfContents {
    /// TOC entries
    pub entries: Vec<TocEntry>,
    /// TOC title
    pub title: Option<String>,
    /// TOC location
    pub location: Option<DocumentLocation>,
}

/// Table of contents entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TocEntry {
    /// Entry text
    pub text: String,
    /// Entry level
    pub level: usize,
    /// Page number
    pub page: Option<usize>,
    /// Target location
    pub target: Option<DocumentLocation>,
}

/// Cross reference
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CrossReference {
    /// Reference identifier
    pub id: String,
    /// Reference text
    pub text: String,
    /// Reference type
    pub reference_type: String,
    /// Target identifier
    pub target_id: String,
    /// Reference location
    pub location: DocumentLocation,
}

/// Document bookmark
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Bookmark {
    /// Bookmark identifier
    pub id: String,
    /// Bookmark name
    pub name: String,
    /// Bookmark location
    pub location: DocumentLocation,
}

/// Page range
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PageRange {
    /// Start page
    pub start: usize,
    /// End page
    pub end: usize,
}
