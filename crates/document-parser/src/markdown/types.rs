// Modified: 2025-09-22

//! Markdown document types and data structures
//!
//! This module contains all the data structures, enums, and type definitions
//! used throughout the Markdown parsing system.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Configuration for Markdown parser
#[derive(Debug, Clone)]
pub struct MarkdownParserConfig {
    /// Enable GitHub Flavored Markdown
    pub enable_gfm: bool,
    /// Extract tables
    pub extract_tables: bool,
    /// Extract code blocks
    pub extract_code_blocks: bool,
    /// Extract links and references
    pub extract_links: bool,
    /// Process SSP-specific content
    pub process_ssp_content: bool,
    /// Enable frontmatter parsing
    pub parse_frontmatter: bool,
    /// Maximum heading depth
    pub max_heading_depth: usize,
    /// Enable math support
    pub enable_math: bool,
}

impl Default for MarkdownParserConfig {
    fn default() -> Self {
        Self {
            enable_gfm: true,
            extract_tables: true,
            extract_code_blocks: true,
            extract_links: true,
            process_ssp_content: true,
            parse_frontmatter: true,
            max_heading_depth: 6,
            enable_math: false,
        }
    }
}

/// Parsed Markdown document structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarkdownDocument {
    /// Document metadata
    pub metadata: MarkdownMetadata,
    /// Document structure
    pub structure: MarkdownStructure,
    /// Document content
    pub content: MarkdownContent,
    /// Tables in the document
    pub tables: Vec<MarkdownTable>,
    /// Code blocks in the document
    pub code_blocks: Vec<CodeBlock>,
    /// Links in the document
    pub links: Vec<MarkdownLink>,
    /// Images in the document
    pub images: Vec<MarkdownImage>,
}

/// Markdown document metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarkdownMetadata {
    /// Document title
    pub title: Option<String>,
    /// Document author
    pub author: Option<String>,
    /// Document description
    pub description: Option<String>,
    /// Document tags
    pub tags: Vec<String>,
    /// Creation date
    pub created: Option<String>,
    /// Last modified date
    pub modified: Option<String>,
    /// Document version
    pub version: Option<String>,
    /// Custom frontmatter properties
    pub frontmatter: HashMap<String, serde_json::Value>,
}

/// Markdown document structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarkdownStructure {
    /// Document headings
    pub headings: Vec<MarkdownHeading>,
    /// Document sections
    pub sections: Vec<MarkdownSection>,
    /// Table of contents
    pub table_of_contents: Option<TableOfContents>,
    /// Cross references
    pub cross_references: Vec<CrossReference>,
    /// Document outline
    pub outline: DocumentOutline,
}

/// Markdown document content
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarkdownContent {
    /// Raw markdown text
    pub raw_markdown: String,
    /// Plain text content
    pub plain_text: String,
    /// HTML content
    pub html: String,
    /// Content elements
    pub elements: Vec<MarkdownElement>,
    /// Word count
    pub word_count: usize,
    /// Character count
    pub character_count: usize,
}

/// Markdown heading
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarkdownHeading {
    /// Heading text
    pub text: String,
    /// Heading level (1-6)
    pub level: usize,
    /// Heading ID/anchor
    pub id: Option<String>,
    /// Location in document
    pub location: DocumentLocation,
}

/// Markdown section
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarkdownSection {
    /// Section identifier
    pub id: String,
    /// Section title
    pub title: String,
    /// Section level
    pub level: usize,
    /// Section content
    pub content: Vec<MarkdownElement>,
    /// Subsections
    pub subsections: Vec<MarkdownSection>,
    /// Section metadata
    pub metadata: SectionMetadata,
}

/// Markdown element
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarkdownElement {
    /// Element type
    pub element_type: MarkdownElementType,
    /// Element content
    pub content: String,
    /// Element attributes
    pub attributes: HashMap<String, String>,
    /// Element location
    pub location: DocumentLocation,
}

/// Type of Markdown element
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum MarkdownElementType {
    /// Paragraph
    Paragraph,
    /// Heading
    Heading,
    /// List (ordered or unordered)
    List,
    /// List item
    ListItem,
    /// Table
    Table,
    /// Code block
    CodeBlock,
    /// Blockquote
    Blockquote,
    /// Horizontal rule
    HorizontalRule,
    /// Link
    Link,
    /// Image
    Image,
    /// Emphasis (italic)
    Emphasis,
    /// Strong (bold)
    Strong,
    /// Inline code
    InlineCode,
    /// Line break
    LineBreak,
}

/// Markdown table
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarkdownTable {
    /// Table identifier
    pub id: String,
    /// Table caption
    pub caption: Option<String>,
    /// Table headers
    pub headers: Vec<String>,
    /// Table rows
    pub rows: Vec<Vec<String>>,
    /// Column alignment
    pub alignment: Vec<TableAlignment>,
    /// Table location
    pub location: DocumentLocation,
}

/// Table column alignment
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum TableAlignment {
    /// Left aligned
    Left,
    /// Center aligned
    Center,
    /// Right aligned
    Right,
    /// No specific alignment
    None,
}

/// Code block
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodeBlock {
    /// Code block identifier
    pub id: String,
    /// Programming language
    pub language: Option<String>,
    /// Code content
    pub code: String,
    /// Code block metadata
    pub metadata: HashMap<String, String>,
    /// Code block location
    pub location: DocumentLocation,
}

/// Markdown link
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarkdownLink {
    /// Link text
    pub text: String,
    /// Link URL
    pub url: String,
    /// Link title
    pub title: Option<String>,
    /// Link type
    pub link_type: MarkdownLinkType,
    /// Link location
    pub location: DocumentLocation,
}

/// Type of Markdown link
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum MarkdownLinkType {
    /// Inline link
    Inline,
    /// Reference link
    Reference,
    /// Autolink
    Autolink,
    /// Email link
    Email,
}

/// Markdown image
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarkdownImage {
    /// Image alt text
    pub alt_text: String,
    /// Image URL
    pub url: String,
    /// Image title
    pub title: Option<String>,
    /// Image dimensions (if available)
    pub dimensions: Option<ImageDimensions>,
    /// Image location
    pub location: DocumentLocation,
}

/// Image dimensions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImageDimensions {
    /// Width in pixels
    pub width: Option<u32>,
    /// Height in pixels
    pub height: Option<u32>,
}

/// Table of contents
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TableOfContents {
    /// TOC entries
    pub entries: Vec<TocEntry>,
    /// TOC title
    pub title: Option<String>,
    /// Maximum depth
    pub max_depth: usize,
}

/// Table of contents entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TocEntry {
    /// Entry text
    pub text: String,
    /// Entry level
    pub level: usize,
    /// Entry anchor/ID
    pub anchor: Option<String>,
    /// Entry location
    pub location: DocumentLocation,
}

/// Cross reference
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CrossReference {
    /// Reference text
    pub text: String,
    /// Reference target
    pub target: String,
    /// Reference type
    pub reference_type: String,
    /// Reference location
    pub location: DocumentLocation,
}

/// Document outline
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentOutline {
    /// Outline nodes
    pub nodes: Vec<OutlineNode>,
    /// Total depth
    pub depth: usize,
}

/// Outline node
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OutlineNode {
    /// Node title
    pub title: String,
    /// Node level
    pub level: usize,
    /// Node children
    pub children: Vec<OutlineNode>,
    /// Node anchor
    pub anchor: Option<String>,
}

/// Section metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SectionMetadata {
    /// Section type (e.g., "control", "narrative", "table")
    pub section_type: Option<String>,
    /// Control ID (for SSP sections)
    pub control_id: Option<String>,
    /// Implementation status
    pub implementation_status: Option<String>,
    /// Responsibility
    pub responsibility: Option<String>,
    /// Custom metadata
    pub custom: HashMap<String, serde_json::Value>,
}

/// Document location information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentLocation {
    /// Line number
    pub line: Option<usize>,
    /// Column number
    pub column: Option<usize>,
    /// Character offset
    pub offset: Option<usize>,
}
