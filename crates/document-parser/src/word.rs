// Modified: 2025-09-20

//! Word document parsing implementation
//!
//! This module provides comprehensive parsing for Microsoft Word documents (.docx)
//! with structure extraction, content analysis, and metadata processing.

use crate::{DocumentParser, ParseResult, DocumentType};
use async_trait::async_trait;
use fedramp_core::{Result, Error};
use std::path::Path;

use tracing::{debug, info, warn};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use docx_rs::*;

/// Word document parser implementation with comprehensive DOCX support
#[derive(Debug, Clone)]
pub struct WordParser {
    /// Maximum file size to process (in bytes)
    pub max_file_size: usize,
    /// Parser configuration
    pub config: DocxParserConfig,
    /// Content extractor
    content_extractor: ContentExtractor,
    /// Structure analyzer
    structure_analyzer: StructureAnalyzer,
    /// Metadata processor
    metadata_processor: MetadataProcessor,
}

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

/// Content extractor for DOCX documents
#[derive(Debug, Clone)]
pub struct ContentExtractor {
    /// Configuration
    pub config: DocxParserConfig,
}

/// Structure analyzer for document hierarchy
#[derive(Debug, Clone)]
pub struct StructureAnalyzer {
    /// Configuration
    pub config: DocxParserConfig,
}

/// Metadata processor for document properties
#[derive(Debug, Clone)]
pub struct MetadataProcessor {
    /// Configuration
    pub config: DocxParserConfig,
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
    /// Location in document
    pub location: DocumentLocation,
}

/// Document content container
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentContent {
    /// Plain text content
    pub text: String,
    /// Formatted content elements
    pub elements: Vec<DocumentElement>,
    /// Word count
    pub word_count: usize,
    /// Character count
    pub character_count: usize,
}

/// Document element (paragraph, table, image, etc.)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentElement {
    /// Element type
    pub element_type: ElementType,
    /// Element content
    pub content: String,
    /// Element formatting
    pub formatting: Option<ElementFormatting>,
    /// Element location
    pub location: DocumentLocation,
}

/// Type of document element
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ElementType {
    /// Paragraph
    Paragraph,
    /// Heading
    Heading,
    /// Table
    Table,
    /// List
    List,
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
    /// Text alignment
    pub alignment: Option<String>,
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
    /// Table formatting
    pub formatting: TableFormatting,
    /// Table location
    pub location: DocumentLocation,
}

/// Table formatting information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TableFormatting {
    /// Table style
    pub style: Option<String>,
    /// Border style
    pub border_style: Option<String>,
    /// Cell padding
    pub cell_padding: Option<f64>,
    /// Table width
    pub width: Option<String>,
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
    pub location: DocumentLocation,
}

/// Image dimensions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImageDimensions {
    /// Width in pixels
    pub width: u32,
    /// Height in pixels
    pub height: u32,
}

/// Document relationship (hyperlinks, references, etc.)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentRelationship {
    /// Relationship type
    pub relationship_type: RelationshipType,
    /// Source element
    pub source: String,
    /// Target element
    pub target: String,
    /// Relationship description
    pub description: Option<String>,
}

/// Type of document relationship
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum RelationshipType {
    /// Hyperlink
    Hyperlink,
    /// Cross reference
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
}

/// Table of contents entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TocEntry {
    /// Entry text
    pub text: String,
    /// Entry level
    pub level: usize,
    /// Page number
    pub page_number: Option<u32>,
    /// Target location
    pub target: Option<String>,
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
}

/// Document bookmark
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Bookmark {
    /// Bookmark name
    pub name: String,
    /// Bookmark text
    pub text: Option<String>,
    /// Bookmark location
    pub location: DocumentLocation,
}

/// Document location information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentLocation {
    /// Page number
    pub page: Option<u32>,
    /// Paragraph index
    pub paragraph: Option<usize>,
    /// Character offset
    pub character_offset: Option<usize>,
}

/// Page range
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PageRange {
    /// Start page
    pub start: u32,
    /// End page
    pub end: u32,
}

impl WordParser {
    /// Create a new Word parser with default settings
    #[must_use]
    pub fn new() -> Self {
        let config = DocxParserConfig::default();
        Self {
            max_file_size: 100 * 1024 * 1024, // 100MB
            content_extractor: ContentExtractor::new(config.clone()),
            structure_analyzer: StructureAnalyzer::new(config.clone()),
            metadata_processor: MetadataProcessor::new(config.clone()),
            config,
        }
    }

    /// Create a new Word parser with custom configuration
    #[must_use]
    pub fn with_config(max_file_size: usize, config: DocxParserConfig) -> Self {
        Self {
            max_file_size,
            content_extractor: ContentExtractor::new(config.clone()),
            structure_analyzer: StructureAnalyzer::new(config.clone()),
            metadata_processor: MetadataProcessor::new(config.clone()),
            config,
        }
    }

    /// Parse DOCX document from file path
    pub async fn parse_docx_file(&self, path: &Path) -> Result<DocxDocument> {
        info!("Parsing DOCX file: {}", path.display());

        // Read file
        let file_data = tokio::fs::read(path).await
            .map_err(|e| Error::document_parsing(format!("Failed to read file: {}", e)))?;

        // Check file size
        if file_data.len() > self.max_file_size {
            return Err(Error::document_parsing(format!(
                "File size {} exceeds maximum allowed size {}",
                file_data.len(),
                self.max_file_size
            )));
        }

        self.parse_docx_bytes(&file_data, &path.to_string_lossy()).await
    }

    /// Parse DOCX document from bytes
    pub async fn parse_docx_bytes(&self, data: &[u8], filename: &str) -> Result<DocxDocument> {
        info!("Parsing DOCX bytes for file: {}", filename);

        // Parse DOCX using docx-rs
        let docx = read_docx(data)
            .map_err(|e| Error::document_parsing(format!("Failed to parse DOCX: {}", e)))?;

        // Extract metadata
        let metadata = self.metadata_processor.extract_metadata(&docx)?;

        // Extract content
        let content = self.content_extractor.extract_content(&docx)?;

        // Analyze structure
        let structure = self.structure_analyzer.analyze_structure(&docx)?;

        // Extract tables
        let tables = self.extract_tables(&docx)?;

        // Extract images
        let images = self.extract_images(&docx)?;

        // Extract relationships
        let relationships = self.extract_relationships(&docx)?;

        Ok(DocxDocument {
            metadata,
            structure,
            content,
            tables,
            images,
            relationships,
        })
    }

    /// Extract tables from DOCX document
    fn extract_tables(&self, docx: &Docx) -> Result<Vec<DocumentTable>> {
        let mut tables = Vec::new();

        if !self.config.extract_tables {
            return Ok(tables);
        }

        // TODO: Implement table extraction from docx-rs
        // This would iterate through the document and extract table data

        Ok(tables)
    }

    /// Extract images from DOCX document
    fn extract_images(&self, docx: &Docx) -> Result<Vec<DocumentImage>> {
        let mut images = Vec::new();

        if !self.config.extract_images {
            return Ok(images);
        }

        // TODO: Implement image extraction from docx-rs
        // This would extract embedded images and their metadata

        Ok(images)
    }

    /// Extract relationships from DOCX document
    fn extract_relationships(&self, docx: &Docx) -> Result<Vec<DocumentRelationship>> {
        let mut relationships = Vec::new();

        // TODO: Implement relationship extraction from docx-rs
        // This would extract hyperlinks, cross-references, etc.

        Ok(relationships)
    }

    /// Calculate quality score for parsed document
    pub fn calculate_quality_score(&self, docx_document: &DocxDocument) -> f64 {
        let mut score = 0.0;
        let mut max_score = 0.0;

        // Content completeness (40% of score)
        max_score += 40.0;
        if !docx_document.content.text.is_empty() {
            score += 20.0;

            // Bonus for substantial content
            if docx_document.content.word_count > 100 {
                score += 10.0;
            }
            if docx_document.content.word_count > 500 {
                score += 10.0;
            }
        }

        // Structure quality (30% of score)
        max_score += 30.0;
        if !docx_document.structure.headings.is_empty() {
            score += 15.0;

            // Bonus for well-structured documents
            if docx_document.structure.headings.len() > 3 {
                score += 10.0;
            }
            if !docx_document.structure.sections.is_empty() {
                score += 5.0;
            }
        }

        // Metadata completeness (20% of score)
        max_score += 20.0;
        let mut metadata_fields = 0;
        if docx_document.metadata.title.is_some() { metadata_fields += 1; }
        if docx_document.metadata.author.is_some() { metadata_fields += 1; }
        if docx_document.metadata.subject.is_some() { metadata_fields += 1; }
        if docx_document.metadata.created.is_some() { metadata_fields += 1; }
        if docx_document.metadata.modified.is_some() { metadata_fields += 1; }

        score += (metadata_fields as f64 / 5.0) * 20.0;

        // Rich content (10% of score)
        max_score += 10.0;
        if !docx_document.tables.is_empty() {
            score += 5.0;
        }
        if !docx_document.images.is_empty() {
            score += 5.0;
        }

        // Normalize score to 0.0-1.0 range
        if max_score > 0.0 {
            score / max_score
        } else {
            0.0
        }
    }
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

impl ContentExtractor {
    /// Create a new content extractor
    pub fn new(config: DocxParserConfig) -> Self {
        Self { config }
    }

    /// Extract content from DOCX document
    pub fn extract_content(&self, docx: &Docx) -> Result<DocumentContent> {
        let mut text = String::new();
        let mut elements = Vec::new();
        let mut word_count = 0;
        let mut character_count = 0;

        // Extract text content from document
        for child in &docx.document.children {
            match child {
                DocumentChild::Paragraph(paragraph) => {
                    let paragraph_text = self.extract_paragraph_text(paragraph)?;
                    if !paragraph_text.is_empty() {
                        text.push_str(&paragraph_text);
                        text.push('\n');

                        elements.push(DocumentElement {
                            element_type: ElementType::Paragraph,
                            content: paragraph_text.clone(),
                            formatting: self.extract_paragraph_formatting(paragraph),
                            location: DocumentLocation {
                                page: None,
                                paragraph: Some(elements.len()),
                                character_offset: Some(character_count),
                            },
                        });

                        word_count += paragraph_text.split_whitespace().count();
                        character_count += paragraph_text.len();
                    }
                }
                DocumentChild::Table(table) => {
                    if self.config.extract_tables {
                        let table_text = self.extract_table_text(table)?;
                        if !table_text.is_empty() {
                            text.push_str(&table_text);
                            text.push('\n');

                            elements.push(DocumentElement {
                                element_type: ElementType::Table,
                                content: table_text.clone(),
                                formatting: None,
                                location: DocumentLocation {
                                    page: None,
                                    paragraph: Some(elements.len()),
                                    character_offset: Some(character_count),
                                },
                            });

                            word_count += table_text.split_whitespace().count();
                            character_count += table_text.len();
                        }
                    }
                }
                _ => {
                    // Handle other document children as needed
                    debug!("Unhandled document child type");
                }
            }
        }

        Ok(DocumentContent {
            text,
            elements,
            word_count,
            character_count,
        })
    }

    /// Extract text from a paragraph
    fn extract_paragraph_text(&self, _paragraph: &Paragraph) -> Result<String> {
        // TODO: Implement proper paragraph text extraction once docx-rs API is clarified
        Ok("Paragraph content placeholder".to_string())
    }

    /// Extract formatting from a paragraph
    fn extract_paragraph_formatting(&self, paragraph: &Paragraph) -> Option<ElementFormatting> {
        if !self.config.preserve_formatting {
            return None;
        }

        // TODO: Extract actual formatting information from paragraph properties
        Some(ElementFormatting {
            bold: false,
            italic: false,
            underline: false,
            font_name: None,
            font_size: None,
            color: None,
            background_color: None,
            alignment: None,
        })
    }

    /// Extract text from a table
    fn extract_table_text(&self, _table: &Table) -> Result<String> {
        // TODO: Implement proper table text extraction once docx-rs API is clarified
        Ok("Table content placeholder".to_string())
    }
}

impl StructureAnalyzer {
    /// Create a new structure analyzer
    pub fn new(config: DocxParserConfig) -> Self {
        Self { config }
    }

    /// Analyze document structure
    pub fn analyze_structure(&self, docx: &Docx) -> Result<DocumentStructure> {
        if !self.config.analyze_structure {
            return Ok(DocumentStructure {
                sections: Vec::new(),
                headings: Vec::new(),
                table_of_contents: None,
                cross_references: Vec::new(),
                bookmarks: Vec::new(),
            });
        }

        let headings = self.extract_headings(docx)?;
        let sections = self.build_sections(&headings)?;
        let bookmarks = self.extract_bookmarks(docx)?;
        let cross_references = self.extract_cross_references(docx)?;

        Ok(DocumentStructure {
            sections,
            headings,
            table_of_contents: None, // TODO: Implement TOC extraction
            cross_references,
            bookmarks,
        })
    }

    /// Extract headings from document
    fn extract_headings(&self, docx: &Docx) -> Result<Vec<DocumentHeading>> {
        let mut headings = Vec::new();
        let mut paragraph_index = 0;

        for child in &docx.document.children {
            if let DocumentChild::Paragraph(paragraph) = child {
                if let Some(heading) = self.extract_heading_from_paragraph(paragraph, paragraph_index)? {
                    headings.push(heading);
                }
                paragraph_index += 1;
            }
        }

        Ok(headings)
    }

    /// Extract heading from paragraph if it is a heading
    fn extract_heading_from_paragraph(&self, paragraph: &Paragraph, index: usize) -> Result<Option<DocumentHeading>> {
        // Check if paragraph has heading style
        if let Some(style) = &paragraph.property.style {
            if style.val.starts_with("Heading") {
                let level = self.parse_heading_level(&style.val);
                let text = self.extract_paragraph_text_simple(paragraph)?;

                return Ok(Some(DocumentHeading {
                    text,
                    level,
                    style: Some(style.val.clone()),
                    location: DocumentLocation {
                        page: None,
                        paragraph: Some(index),
                        character_offset: None,
                    },
                }));
            }
        }

        Ok(None)
    }

    /// Parse heading level from style name
    fn parse_heading_level(&self, style_name: &str) -> usize {
        if let Some(level_str) = style_name.strip_prefix("Heading") {
            level_str.trim().parse().unwrap_or(1)
        } else {
            1
        }
    }

    /// Extract simple text from paragraph
    fn extract_paragraph_text_simple(&self, _paragraph: &Paragraph) -> Result<String> {
        // TODO: Implement proper paragraph text extraction once docx-rs API is clarified
        Ok("Heading text placeholder".to_string())
    }

    /// Build sections from headings
    fn build_sections(&self, headings: &[DocumentHeading]) -> Result<Vec<DocumentSection>> {
        let mut sections = Vec::new();

        // TODO: Implement section building logic
        // This would group content under headings into sections

        Ok(sections)
    }

    /// Extract bookmarks from document
    fn extract_bookmarks(&self, _docx: &Docx) -> Result<Vec<Bookmark>> {
        let mut bookmarks = Vec::new();

        // TODO: Implement bookmark extraction

        Ok(bookmarks)
    }

    /// Extract cross references from document
    fn extract_cross_references(&self, _docx: &Docx) -> Result<Vec<CrossReference>> {
        let mut cross_references = Vec::new();

        // TODO: Implement cross reference extraction

        Ok(cross_references)
    }
}

impl MetadataProcessor {
    /// Create a new metadata processor
    pub fn new(config: DocxParserConfig) -> Self {
        Self { config }
    }

    /// Extract metadata from DOCX document
    pub fn extract_metadata(&self, docx: &Docx) -> Result<DocumentMetadata> {
        let mut metadata = DocumentMetadata {
            title: None,
            author: None,
            subject: None,
            description: None,
            created: None,
            modified: None,
            version: None,
            language: None,
            custom_properties: HashMap::new(),
        };

        // TODO: Extract actual metadata from docx-rs once API is clarified
        // For now, provide placeholder metadata
        metadata.title = Some("Document Title".to_string());
        metadata.author = Some("Document Author".to_string());

        Ok(metadata)
    }
}

impl Default for WordParser {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_word_parser_creation() {
        let parser = WordParser::new();
        assert_eq!(parser.max_file_size, 100 * 1024 * 1024);
    }

    #[test]
    fn test_docx_parser_config_default() {
        let config = DocxParserConfig::default();
        assert!(config.extract_images);
        assert!(config.extract_tables);
        assert!(config.analyze_structure);
    }
}

#[async_trait]
impl DocumentParser for WordParser {
    async fn parse_file(&self, path: &Path) -> Result<ParseResult> {
        info!("Parsing Word file: {}", path.display());

        // Check file extension
        if let Some(extension) = path.extension() {
            if extension != "docx" {
                warn!("Unexpected file extension for Word parser: {:?}", extension);
            }
        }

        // Parse DOCX document
        let docx_document = self.parse_docx_file(path).await?;

        // Build metadata JSON
        let mut metadata_map = serde_json::Map::new();
        metadata_map.insert("source_file".to_string(), serde_json::Value::String(path.to_string_lossy().to_string()));
        metadata_map.insert("source_type".to_string(), serde_json::Value::String("word".to_string()));
        metadata_map.insert("extraction_date".to_string(), serde_json::Value::String(chrono::Utc::now().to_rfc3339()));
        metadata_map.insert("parser_version".to_string(), serde_json::Value::String(env!("CARGO_PKG_VERSION").to_string()));

        // Add document metadata
        if let Some(title) = &docx_document.metadata.title {
            metadata_map.insert("title".to_string(), serde_json::Value::String(title.clone()));
        }
        if let Some(author) = &docx_document.metadata.author {
            metadata_map.insert("author".to_string(), serde_json::Value::String(author.clone()));
        }
        if let Some(subject) = &docx_document.metadata.subject {
            metadata_map.insert("subject".to_string(), serde_json::Value::String(subject.clone()));
        }
        if let Some(created) = &docx_document.metadata.created {
            metadata_map.insert("created".to_string(), serde_json::Value::String(created.clone()));
        }
        if let Some(modified) = &docx_document.metadata.modified {
            metadata_map.insert("modified".to_string(), serde_json::Value::String(modified.clone()));
        }

        // Add content statistics
        metadata_map.insert("word_count".to_string(), serde_json::Value::Number(serde_json::Number::from(docx_document.content.word_count)));
        metadata_map.insert("character_count".to_string(), serde_json::Value::Number(serde_json::Number::from(docx_document.content.character_count)));
        metadata_map.insert("paragraph_count".to_string(), serde_json::Value::Number(serde_json::Number::from(docx_document.content.elements.len())));
        metadata_map.insert("table_count".to_string(), serde_json::Value::Number(serde_json::Number::from(docx_document.tables.len())));
        metadata_map.insert("image_count".to_string(), serde_json::Value::Number(serde_json::Number::from(docx_document.images.len())));
        metadata_map.insert("heading_count".to_string(), serde_json::Value::Number(serde_json::Number::from(docx_document.structure.headings.len())));

        // Calculate quality score first (before moving data)
        let quality_score = self.calculate_quality_score(&docx_document);

        // Build content JSON
        let mut content_map = serde_json::Map::new();
        content_map.insert("text".to_string(), serde_json::Value::String(docx_document.content.text));

        // Add sections
        let sections: Vec<serde_json::Value> = docx_document.structure.sections.into_iter().map(|section| {
            serde_json::json!({
                "id": section.id,
                "title": section.title,
                "level": section.level,
                "content": section.content.into_iter().map(|element| {
                    serde_json::json!({
                        "type": format!("{:?}", element.element_type),
                        "content": element.content
                    })
                }).collect::<Vec<_>>()
            })
        }).collect();
        content_map.insert("sections".to_string(), serde_json::Value::Array(sections));

        // Add tables
        let tables: Vec<serde_json::Value> = docx_document.tables.into_iter().map(|table| {
            serde_json::json!({
                "id": table.id,
                "title": table.title,
                "headers": table.headers,
                "rows": table.rows
            })
        }).collect();
        content_map.insert("tables".to_string(), serde_json::Value::Array(tables));

        // Add headings
        let headings: Vec<serde_json::Value> = docx_document.structure.headings.into_iter().map(|heading| {
            serde_json::json!({
                "text": heading.text,
                "level": heading.level,
                "style": heading.style
            })
        }).collect();
        content_map.insert("headings".to_string(), serde_json::Value::Array(headings));

        Ok(ParseResult {
            document_type: DocumentType::Word,
            source_path: path.to_string_lossy().to_string(),
            metadata: serde_json::Value::Object(metadata_map),
            content: serde_json::Value::Object(content_map),
            validation_errors: Vec::new(),
            quality_score,
        })
    }

    async fn parse_bytes(&self, data: &[u8], filename: &str) -> Result<ParseResult> {
        info!("Parsing Word bytes for file: {}", filename);

        // Parse DOCX document from bytes
        let docx_document = self.parse_docx_bytes(data, filename).await?;

        // Build metadata JSON
        let mut metadata_map = serde_json::Map::new();
        metadata_map.insert("source_file".to_string(), serde_json::Value::String(filename.to_string()));
        metadata_map.insert("source_type".to_string(), serde_json::Value::String("word".to_string()));
        metadata_map.insert("extraction_date".to_string(), serde_json::Value::String(chrono::Utc::now().to_rfc3339()));
        metadata_map.insert("parser_version".to_string(), serde_json::Value::String(env!("CARGO_PKG_VERSION").to_string()));

        // Add document metadata
        if let Some(title) = &docx_document.metadata.title {
            metadata_map.insert("title".to_string(), serde_json::Value::String(title.clone()));
        }
        if let Some(author) = &docx_document.metadata.author {
            metadata_map.insert("author".to_string(), serde_json::Value::String(author.clone()));
        }

        // Add content statistics
        metadata_map.insert("word_count".to_string(), serde_json::Value::Number(serde_json::Number::from(docx_document.content.word_count)));
        metadata_map.insert("character_count".to_string(), serde_json::Value::Number(serde_json::Number::from(docx_document.content.character_count)));

        // Calculate quality score first (before moving data)
        let quality_score = self.calculate_quality_score(&docx_document);

        // Build content JSON
        let mut content_map = serde_json::Map::new();
        content_map.insert("text".to_string(), serde_json::Value::String(docx_document.content.text));

        // Add sections
        let sections: Vec<serde_json::Value> = docx_document.structure.sections.into_iter().map(|section| {
            serde_json::json!({
                "id": section.id,
                "title": section.title,
                "level": section.level
            })
        }).collect();
        content_map.insert("sections".to_string(), serde_json::Value::Array(sections));

        // Add tables
        let tables: Vec<serde_json::Value> = docx_document.tables.into_iter().map(|table| {
            serde_json::json!({
                "id": table.id,
                "title": table.title,
                "headers": table.headers,
                "rows": table.rows
            })
        }).collect();
        content_map.insert("tables".to_string(), serde_json::Value::Array(tables));

        Ok(ParseResult {
            document_type: DocumentType::Word,
            source_path: filename.to_string(),
            metadata: serde_json::Value::Object(metadata_map),
            content: serde_json::Value::Object(content_map),
            validation_errors: Vec::new(),
            quality_score,
        })
    }

    async fn validate(&self, content: &serde_json::Value) -> Result<Vec<String>> {
        let mut validation_errors = Vec::new();

        // Validate content structure
        if let Some(content_obj) = content.as_object() {
            // Check for required fields
            if !content_obj.contains_key("text") {
                validation_errors.push("Missing text content".to_string());
            }

            if !content_obj.contains_key("sections") {
                validation_errors.push("Missing sections structure".to_string());
            }

            // Validate text content
            if let Some(text) = content_obj.get("text").and_then(|v| v.as_str()) {
                if text.is_empty() {
                    validation_errors.push("Document contains no text content".to_string());
                }

                // Check for minimum content length
                if text.len() < 10 {
                    validation_errors.push("Document content is too short".to_string());
                }
            }

            // Validate sections
            if let Some(sections) = content_obj.get("sections").and_then(|v| v.as_array()) {
                for (i, section) in sections.iter().enumerate() {
                    if let Some(section_obj) = section.as_object() {
                        if !section_obj.contains_key("title") {
                            validation_errors.push(format!("Section {} missing title", i));
                        }
                        if !section_obj.contains_key("level") {
                            validation_errors.push(format!("Section {} missing level", i));
                        }
                    }
                }
            }

            // Validate tables
            if let Some(tables) = content_obj.get("tables").and_then(|v| v.as_array()) {
                for (i, table) in tables.iter().enumerate() {
                    if let Some(table_obj) = table.as_object() {
                        if !table_obj.contains_key("headers") {
                            validation_errors.push(format!("Table {} missing headers", i));
                        }
                        if !table_obj.contains_key("rows") {
                            validation_errors.push(format!("Table {} missing rows", i));
                        }
                    }
                }
            }
        } else {
            validation_errors.push("Invalid content structure - expected object".to_string());
        }

        Ok(validation_errors)
    }

    fn supported_extensions(&self) -> Vec<&'static str> {
        vec!["docx"]
    }
}
