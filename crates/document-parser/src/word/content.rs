// Modified: 2025-01-22

//! Content extraction functionality for Word documents
//!
//! This module provides content extraction capabilities including text extraction,
//! element parsing, and formatting preservation for DOCX documents.

use super::types::*;
use fedramp_core::{Result, Error};
use docx_rs::*;
use tracing::{debug, warn};

/// Content extractor for DOCX documents
#[derive(Debug, Clone)]
pub struct ContentExtractor {
    /// Configuration
    pub config: DocxParserConfig,
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

                        // Update counts
                        let words = paragraph_text.split_whitespace().count();
                        word_count += words;
                        character_count += paragraph_text.len() + 1; // +1 for newline
                    }
                }
                DocumentChild::Table(table) => {
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

                        // Update counts
                        let words = table_text.split_whitespace().count();
                        word_count += words;
                        character_count += table_text.len() + 1; // +1 for newline
                    }
                }
                _ => {
                    debug!("Skipping unsupported document child type");
                }
            }
        }

        let paragraph_count = elements.iter()
            .filter(|e| matches!(e.element_type, ElementType::Paragraph))
            .count();

        Ok(DocumentContent {
            text,
            elements,
            word_count,
            character_count,
            paragraph_count,
        })
    }

    /// Extract text from a paragraph
    pub fn extract_paragraph_text(&self, paragraph: &Paragraph) -> Result<String> {
        let mut text = String::new();

        for child in &paragraph.children {
            match child {
                ParagraphChild::Run(run) => {
                    let run_text = self.extract_run_text(run)?;
                    text.push_str(&run_text);
                }
                ParagraphChild::Insert(insert) => {
                    for insert_child in &insert.children {
                        if let InsertChild::Run(run) = insert_child {
                            let run_text = self.extract_run_text(run)?;
                            text.push_str(&run_text);
                        }
                    }
                }
                ParagraphChild::Delete(_) => {
                    // Skip deleted content
                }
                ParagraphChild::BookmarkStart(_) | ParagraphChild::BookmarkEnd(_) => {
                    // Skip bookmark markers
                }
                ParagraphChild::Hyperlink(hyperlink) => {
                    for hyperlink_child in &hyperlink.children {
                        // Note: HyperlinkChild enum structure may vary in docx-rs
                        // For now, skip hyperlink content extraction
                        debug!("Skipping hyperlink child extraction");
                    }
                }
                _ => {
                    debug!("Skipping unsupported paragraph child type");
                }
            }
        }

        Ok(text)
    }

    /// Extract text from a run
    pub fn extract_run_text(&self, run: &Run) -> Result<String> {
        let mut text = String::new();

        for child in &run.children {
            match child {
                RunChild::Text(text_element) => {
                    text.push_str(&text_element.text);
                }
                RunChild::Tab(_) => {
                    text.push('\t');
                }
                RunChild::Break(_br) => {
                    // Note: break_type field is private in docx-rs
                    text.push('\n');
                }
                RunChild::DeleteText(_) => {
                    // Skip deleted text
                }
                _ => {
                    debug!("Skipping unsupported run child type");
                }
            }
        }

        Ok(text)
    }

    /// Extract text from a table
    pub fn extract_table_text(&self, table: &Table) -> Result<String> {
        let mut text = String::new();

        for row_child in &table.rows {
            if let TableChild::TableRow(row) = row_child {
                let mut row_text = String::new();
                for cell_child in &row.cells {
                    if let TableRowChild::TableCell(cell) = cell_child {
                        let cell_text = self.extract_table_cell_text(cell)?;
                        if !row_text.is_empty() {
                            row_text.push('\t');
                        }
                        row_text.push_str(&cell_text);
                    }
                }
                if !row_text.is_empty() {
                    text.push_str(&row_text);
                    text.push('\n');
                }
            }
        }

        Ok(text)
    }

    /// Extract text from a table cell
    pub fn extract_table_cell_text(&self, cell: &TableCell) -> Result<String> {
        let mut text = String::new();

        for child in &cell.children {
            match child {
                TableCellContent::Paragraph(paragraph) => {
                    let paragraph_text = self.extract_paragraph_text(paragraph)?;
                    if !text.is_empty() && !paragraph_text.is_empty() {
                        text.push(' ');
                    }
                    text.push_str(&paragraph_text);
                }
                TableCellContent::Table(nested_table) => {
                    let table_text = self.extract_table_text(nested_table)?;
                    if !text.is_empty() && !table_text.is_empty() {
                        text.push(' ');
                    }
                    text.push_str(&table_text);
                }
                _ => {
                    debug!("Skipping unsupported table cell child type");
                }
            }
        }

        Ok(text)
    }

    /// Extract paragraph formatting
    pub fn extract_paragraph_formatting(&self, paragraph: &Paragraph) -> Option<ElementFormatting> {
        if !self.config.preserve_formatting {
            return None;
        }

        // TODO: Extract actual formatting from paragraph properties
        // For now, return basic formatting
        Some(ElementFormatting {
            bold: false,
            italic: false,
            underline: false,
            font_name: None,
            font_size: None,
            color: None,
            background_color: None,
        })
    }

    /// Extract run formatting
    pub fn extract_run_formatting(&self, run: &Run) -> Option<ElementFormatting> {
        if !self.config.preserve_formatting {
            return None;
        }

        let mut formatting = ElementFormatting {
            bold: false,
            italic: false,
            underline: false,
            font_name: None,
            font_size: None,
            color: None,
            background_color: None,
        };

        // Extract formatting from run properties
        let run_property = &run.run_property;
        if run_property.bold.is_some() {
            formatting.bold = true;
        }
        if run_property.italic.is_some() {
            formatting.italic = true;
        }
        if run_property.underline.is_some() {
            formatting.underline = true;
        }
        // Note: Font and color extraction may need adjustment based on docx-rs API
        // For now, use basic formatting detection

        Some(formatting)
    }

    /// Extract hyperlinks from content
    pub fn extract_hyperlinks(&self, docx: &Docx) -> Result<Vec<DocumentRelationship>> {
        let mut relationships = Vec::new();

        // TODO: Implement hyperlink extraction
        // This would traverse the document and extract hyperlink relationships

        Ok(relationships)
    }

    /// Extract footnotes and endnotes
    pub fn extract_notes(&self, docx: &Docx) -> Result<Vec<DocumentElement>> {
        let mut notes = Vec::new();

        // TODO: Implement footnote/endnote extraction
        // This would extract footnotes and endnotes from the document

        Ok(notes)
    }

    /// Calculate content statistics
    pub fn calculate_content_stats(&self, content: &DocumentContent) -> ContentStats {
        let sentences = content.text.split(&['.', '!', '?'][..]).count();
        let avg_words_per_sentence = if sentences > 0 {
            content.word_count as f64 / sentences as f64
        } else {
            0.0
        };

        let avg_chars_per_word = if content.word_count > 0 {
            content.character_count as f64 / content.word_count as f64
        } else {
            0.0
        };

        ContentStats {
            word_count: content.word_count,
            character_count: content.character_count,
            paragraph_count: content.paragraph_count,
            sentence_count: sentences,
            avg_words_per_sentence,
            avg_chars_per_word,
        }
    }
}

/// Content statistics
#[derive(Debug, Clone)]
pub struct ContentStats {
    /// Total word count
    pub word_count: usize,
    /// Total character count
    pub character_count: usize,
    /// Total paragraph count
    pub paragraph_count: usize,
    /// Total sentence count
    pub sentence_count: usize,
    /// Average words per sentence
    pub avg_words_per_sentence: f64,
    /// Average characters per word
    pub avg_chars_per_word: f64,
}
