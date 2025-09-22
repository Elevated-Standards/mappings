// Modified: 2025-01-22

//! Core Word document parsing functionality
//!
//! This module provides the main WordParser implementation with document parsing,
//! quality scoring, and table/image extraction capabilities.

use super::types::*;
use super::content::ContentExtractor;
use super::structure::StructureAnalyzer;
use super::metadata::MetadataProcessor;
use fedramp_core::{Result, Error};
use docx_rs::*;
use std::path::Path;
use tracing::{debug, info, warn};

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

impl WordParser {
    /// Create a new Word parser with default configuration
    pub fn new() -> Self {
        let config = DocxParserConfig::default();
        Self::with_config(config)
    }

    /// Create a new Word parser with custom configuration
    pub fn with_config(config: DocxParserConfig) -> Self {
        Self {
            max_file_size: 100 * 1024 * 1024, // 100MB default
            content_extractor: ContentExtractor::new(config.clone()),
            structure_analyzer: StructureAnalyzer::new(config.clone()),
            metadata_processor: MetadataProcessor::new(config.clone()),
            config,
        }
    }

    /// Set maximum file size
    pub fn with_max_file_size(mut self, max_size: usize) -> Self {
        self.max_file_size = max_size;
        self
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

        let mut table_counter = 0;

        for child in &docx.document.children {
            if let DocumentChild::Table(table) = child {
                table_counter += 1;
                
                let doc_table = self.extract_table_data(table, table_counter)?;
                tables.push(doc_table);
            }
        }

        debug!("Extracted {} tables from document", tables.len());
        Ok(tables)
    }

    /// Extract data from a single table
    fn extract_table_data(&self, table: &Table, table_id: usize) -> Result<DocumentTable> {
        let mut headers = Vec::new();
        let mut rows = Vec::new();

        // Extract table data
        for (row_index, row_child) in table.rows.iter().enumerate() {
            if let TableChild::TableRow(row) = row_child {
                let mut row_data = Vec::new();

                for cell_child in &row.cells {
                    if let TableRowChild::TableCell(cell) = cell_child {
                        let cell_text = self.extract_table_cell_text(cell)?;
                        row_data.push(cell_text);
                    }
                }

                // First row is typically headers
                if row_index == 0 && !row_data.is_empty() {
                    headers = row_data;
                } else if !row_data.is_empty() {
                    rows.push(row_data);
                }
            }
        }

        // If no headers were found, create generic ones
        if headers.is_empty() && !rows.is_empty() {
            let column_count = rows.first().map(|r| r.len()).unwrap_or(0);
            headers = (1..=column_count)
                .map(|i| format!("Column {}", i))
                .collect();
        }

        Ok(DocumentTable {
            id: format!("table_{}", table_id),
            title: None, // TODO: Extract table caption if available
            headers,
            rows,
            location: None,
            formatting: None, // TODO: Extract table formatting
        })
    }

    /// Extract text from table cell
    fn extract_table_cell_text(&self, cell: &TableCell) -> Result<String> {
        let mut text = String::new();

        for child in &cell.children {
            match child {
                TableCellContent::Paragraph(paragraph) => {
                    let paragraph_text = self.content_extractor.extract_paragraph_text(paragraph)?;
                    if !text.is_empty() && !paragraph_text.is_empty() {
                        text.push(' ');
                    }
                    text.push_str(&paragraph_text);
                }
                _ => {
                    debug!("Skipping unsupported table cell child type");
                }
            }
        }

        Ok(text.trim().to_string())
    }

    /// Extract images from DOCX document
    fn extract_images(&self, _docx: &Docx) -> Result<Vec<DocumentImage>> {
        let mut images = Vec::new();

        if !self.config.extract_images {
            return Ok(images);
        }

        // TODO: Implement image extraction from docx-rs
        // This would extract embedded images and their metadata

        Ok(images)
    }

    /// Extract relationships from DOCX document
    fn extract_relationships(&self, _docx: &Docx) -> Result<Vec<DocumentRelationship>> {
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

    /// Validate parsed document
    pub fn validate_document(&self, docx_document: &DocxDocument) -> Vec<String> {
        let mut validation_errors = Vec::new();

        // Validate content
        if docx_document.content.text.is_empty() {
            validation_errors.push("Document contains no text content".to_string());
        }

        if docx_document.content.word_count == 0 {
            validation_errors.push("Document has zero word count".to_string());
        }

        // Validate structure
        let structure_warnings = self.structure_analyzer.validate_structure(&docx_document.structure);
        validation_errors.extend(structure_warnings);

        // Validate metadata
        let metadata_warnings = self.metadata_processor.validate_metadata(&docx_document.metadata);
        validation_errors.extend(metadata_warnings);

        // Check for minimum content requirements
        if docx_document.content.word_count < 10 {
            validation_errors.push("Document content is too short (less than 10 words)".to_string());
        }

        // Check for table data quality
        for (i, table) in docx_document.tables.iter().enumerate() {
            if table.headers.is_empty() {
                validation_errors.push(format!("Table {} has no headers", i + 1));
            }
            if table.rows.is_empty() {
                validation_errors.push(format!("Table {} has no data rows", i + 1));
            }
        }

        validation_errors
    }

    /// Get parser configuration
    pub fn config(&self) -> &DocxParserConfig {
        &self.config
    }

    /// Get maximum file size
    pub fn max_file_size(&self) -> usize {
        self.max_file_size
    }
}

impl Default for WordParser {
    fn default() -> Self {
        Self::new()
    }
}
