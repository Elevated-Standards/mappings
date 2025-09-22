// Modified: 2025-01-22

//! Metadata processing functionality for Word documents
//!
//! This module provides metadata extraction and processing capabilities
//! for DOCX documents including document properties and custom metadata.

use super::types::*;
use fedramp_core::{Result, Error};
use docx_rs::*;
use std::collections::HashMap;
use tracing::{debug, warn};

/// Metadata processor for document properties
#[derive(Debug, Clone)]
pub struct MetadataProcessor {
    /// Configuration
    pub config: DocxParserConfig,
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

        // Extract core properties
        self.extract_core_properties(&mut metadata, docx)?;

        // Extract extended properties
        self.extract_extended_properties(&mut metadata, docx)?;

        // Extract custom properties
        self.extract_custom_properties(&mut metadata, docx)?;

        debug!("Extracted metadata with {} custom properties", metadata.custom_properties.len());
        Ok(metadata)
    }

    /// Extract core document properties
    pub fn extract_core_properties(&self, metadata: &mut DocumentMetadata, docx: &Docx) -> Result<()> {
        // TODO: Extract actual metadata from docx-rs once API is clarified
        // For now, provide placeholder metadata based on document analysis

        // Try to extract title from first heading or document content
        if let Some(title) = self.extract_title_from_content(docx)? {
            metadata.title = Some(title);
        } else {
            metadata.title = Some("Untitled Document".to_string());
        }

        // Set default author
        metadata.author = Some("Unknown Author".to_string());

        // Set default creation date
        metadata.created = Some(chrono::Utc::now().to_rfc3339());
        metadata.modified = Some(chrono::Utc::now().to_rfc3339());

        Ok(())
    }

    /// Extract extended document properties
    pub fn extract_extended_properties(&self, metadata: &mut DocumentMetadata, docx: &Docx) -> Result<()> {
        // TODO: Implement extended properties extraction
        // This would extract properties like word count, page count, etc.

        // For now, set some basic properties
        metadata.version = Some("1.0".to_string());
        metadata.language = Some("en-US".to_string());

        Ok(())
    }

    /// Extract custom document properties
    pub fn extract_custom_properties(&self, metadata: &mut DocumentMetadata, docx: &Docx) -> Result<()> {
        // TODO: Implement custom properties extraction
        // This would extract user-defined custom properties

        // For now, add some placeholder custom properties
        metadata.custom_properties.insert(
            "document_type".to_string(),
            "word_document".to_string(),
        );
        metadata.custom_properties.insert(
            "parser_version".to_string(),
            env!("CARGO_PKG_VERSION").to_string(),
        );

        Ok(())
    }

    /// Extract title from document content
    pub fn extract_title_from_content(&self, docx: &Docx) -> Result<Option<String>> {
        // Look for the first heading or paragraph that could be a title
        for child in &docx.document.children {
            if let DocumentChild::Paragraph(paragraph) = child {
                // Check if this paragraph has a title or heading style
                if let Some(title) = self.extract_title_from_paragraph(paragraph)? {
                    return Ok(Some(title));
                }
            }
        }

        Ok(None)
    }

    /// Extract title from a paragraph
    pub fn extract_title_from_paragraph(&self, paragraph: &Paragraph) -> Result<Option<String>> {
        // Check if paragraph has title or heading style
        let is_title_style = paragraph.property.style
            .as_ref()
            .map(|style| {
                matches!(style.val.as_str(), "Title" | "Heading1" | "Heading 1")
            })
            .unwrap_or(false);

        if !is_title_style {
            return Ok(None);
        }

        // Extract text from paragraph
        let mut text = String::new();
        for child in &paragraph.children {
            if let ParagraphChild::Run(run) = child {
                for run_child in &run.children {
                    if let RunChild::Text(text_element) = run_child {
                        text.push_str(&text_element.text);
                    }
                }
            }
        }

        let title = text.trim();
        if title.is_empty() {
            Ok(None)
        } else {
            Ok(Some(title.to_string()))
        }
    }

    /// Extract document statistics
    pub fn extract_document_statistics(&self, docx: &Docx) -> Result<DocumentStatistics> {
        let mut stats = DocumentStatistics {
            page_count: 0,
            word_count: 0,
            character_count: 0,
            paragraph_count: 0,
            table_count: 0,
            image_count: 0,
            footnote_count: 0,
            endnote_count: 0,
        };

        // Count elements in document
        for child in &docx.document.children {
            match child {
                DocumentChild::Paragraph(paragraph) => {
                    stats.paragraph_count += 1;
                    
                    // Count words and characters in paragraph
                    let paragraph_text = self.extract_paragraph_text(paragraph)?;
                    stats.word_count += paragraph_text.split_whitespace().count();
                    stats.character_count += paragraph_text.len();
                }
                DocumentChild::Table(_) => {
                    stats.table_count += 1;
                }
                _ => {}
            }
        }

        // TODO: Count images, footnotes, endnotes, and calculate page count
        // This would require more detailed analysis of the document structure

        debug!("Document statistics: {} paragraphs, {} words, {} characters, {} tables",
               stats.paragraph_count, stats.word_count, stats.character_count, stats.table_count);

        Ok(stats)
    }

    /// Extract paragraph text for statistics
    fn extract_paragraph_text(&self, paragraph: &Paragraph) -> Result<String> {
        let mut text = String::new();

        for child in &paragraph.children {
            if let ParagraphChild::Run(run) = child {
                for run_child in &run.children {
                    if let RunChild::Text(text_element) = run_child {
                        text.push_str(&text_element.text);
                    }
                }
            }
        }

        Ok(text)
    }

    /// Validate metadata completeness
    pub fn validate_metadata(&self, metadata: &DocumentMetadata) -> Vec<String> {
        let mut warnings = Vec::new();

        // Check for missing core metadata
        if metadata.title.is_none() {
            warnings.push("Document title is missing".to_string());
        }

        if metadata.author.is_none() {
            warnings.push("Document author is missing".to_string());
        }

        if metadata.created.is_none() {
            warnings.push("Document creation date is missing".to_string());
        }

        if metadata.modified.is_none() {
            warnings.push("Document modification date is missing".to_string());
        }

        // Check for empty or placeholder values
        if let Some(title) = &metadata.title {
            if title.trim().is_empty() || title == "Untitled Document" {
                warnings.push("Document title appears to be placeholder or empty".to_string());
            }
        }

        if let Some(author) = &metadata.author {
            if author.trim().is_empty() || author == "Unknown Author" {
                warnings.push("Document author appears to be placeholder or empty".to_string());
            }
        }

        // Check for very long titles (potential data quality issue)
        if let Some(title) = &metadata.title {
            if title.len() > 200 {
                warnings.push("Document title is unusually long (>200 characters)".to_string());
            }
        }

        warnings
    }

    /// Extract security and compliance metadata
    pub fn extract_security_metadata(&self, docx: &Docx) -> Result<SecurityMetadata> {
        let mut security_metadata = SecurityMetadata {
            has_password_protection: false,
            has_digital_signatures: false,
            has_macros: false,
            has_external_links: false,
            classification_level: None,
            sensitivity_label: None,
        };

        // TODO: Implement security metadata extraction
        // This would check for password protection, digital signatures, macros, etc.

        Ok(security_metadata)
    }
}

/// Document statistics
#[derive(Debug, Clone)]
pub struct DocumentStatistics {
    /// Number of pages
    pub page_count: usize,
    /// Number of words
    pub word_count: usize,
    /// Number of characters
    pub character_count: usize,
    /// Number of paragraphs
    pub paragraph_count: usize,
    /// Number of tables
    pub table_count: usize,
    /// Number of images
    pub image_count: usize,
    /// Number of footnotes
    pub footnote_count: usize,
    /// Number of endnotes
    pub endnote_count: usize,
}

/// Security and compliance metadata
#[derive(Debug, Clone)]
pub struct SecurityMetadata {
    /// Whether document has password protection
    pub has_password_protection: bool,
    /// Whether document has digital signatures
    pub has_digital_signatures: bool,
    /// Whether document contains macros
    pub has_macros: bool,
    /// Whether document has external links
    pub has_external_links: bool,
    /// Document classification level
    pub classification_level: Option<String>,
    /// Sensitivity label
    pub sensitivity_label: Option<String>,
}
