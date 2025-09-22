// Modified: 2025-01-22

//! Structure analysis functionality for Word documents
//!
//! This module provides document structure analysis including heading detection,
//! section organization, and document hierarchy extraction.

use super::types::{self, *};
use fedramp_core::{Result, Error};
use docx_rs::*;
use tracing::{debug, warn};

/// Structure analyzer for document hierarchy
#[derive(Debug, Clone)]
pub struct StructureAnalyzer {
    /// Configuration
    pub config: DocxParserConfig,
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

        // Extract headings first
        let headings = self.extract_headings(docx)?;

        // Build sections from headings
        let sections = self.build_sections(&headings)?;

        // Extract table of contents
        let table_of_contents = self.extract_table_of_contents(docx)?;

        // Extract cross references
        let cross_references = self.extract_cross_references(docx)?;

        // Extract bookmarks
        let bookmarks = self.extract_bookmarks(docx)?;

        Ok(DocumentStructure {
            sections,
            headings,
            table_of_contents,
            cross_references,
            bookmarks,
        })
    }

    /// Extract headings from document
    pub fn extract_headings(&self, docx: &Docx) -> Result<Vec<DocumentHeading>> {
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

        debug!("Extracted {} headings from document", headings.len());
        Ok(headings)
    }

    /// Extract heading from paragraph if it is a heading
    pub fn extract_heading_from_paragraph(&self, paragraph: &Paragraph, paragraph_index: usize) -> Result<Option<DocumentHeading>> {
        // Check if paragraph has heading style
        let heading_level = self.get_heading_level(paragraph)?;
        
        if heading_level.is_none() {
            return Ok(None);
        }

        let level = heading_level.unwrap();

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

        if text.trim().is_empty() {
            return Ok(None);
        }

        Ok(Some(DocumentHeading {
            text: text.trim().to_string(),
            level,
            style: self.get_paragraph_style(paragraph),
            location: Some(DocumentLocation {
                page: None,
                paragraph: Some(paragraph_index),
                character_offset: None,
            }),
        }))
    }

    /// Get heading level from paragraph style
    pub fn get_heading_level(&self, paragraph: &Paragraph) -> Result<Option<usize>> {
        // Check paragraph properties for heading style
        if let Some(style) = &paragraph.property.style {
            // Parse heading level from style name
            if style.val.starts_with("Heading") {
                if let Some(level_str) = style.val.chars().last() {
                    if let Some(level) = level_str.to_digit(10) {
                        return Ok(Some(level as usize));
                    }
                }
            }

            // Check for other heading style patterns
            match style.val.as_str() {
                "Title" => return Ok(Some(1)),
                "Subtitle" => return Ok(Some(2)),
                _ => {}
            }
        }

        // TODO: Add more sophisticated heading detection based on formatting
        // For now, return None if no explicit heading style is found
        Ok(None)
    }

    /// Get paragraph style name
    pub fn get_paragraph_style(&self, paragraph: &Paragraph) -> Option<String> {
        paragraph.property.style
            .as_ref()
            .map(|style| style.val.clone())
    }

    /// Build sections from headings
    pub fn build_sections(&self, headings: &[DocumentHeading]) -> Result<Vec<DocumentSection>> {
        let mut sections = Vec::new();

        if headings.is_empty() {
            return Ok(sections);
        }

        let mut current_section: Option<DocumentSection> = None;
        let mut section_counter = 0;

        for heading in headings {
            // Create new section for level 1 headings or if no current section
            if heading.level == 1 || current_section.is_none() {
                // Save previous section if exists
                if let Some(section) = current_section.take() {
                    sections.push(section);
                }

                // Create new section
                section_counter += 1;
                current_section = Some(DocumentSection {
                    id: format!("section_{}", section_counter),
                    title: heading.text.clone(),
                    level: heading.level,
                    content: Vec::new(),
                    subsections: Vec::new(),
                    page_range: None,
                });
            } else if let Some(ref mut section) = current_section {
                // Add as subsection if level > 1
                if heading.level > section.level {
                    let subsection = DocumentSection {
                        id: format!("section_{}_{}", section_counter, section.subsections.len() + 1),
                        title: heading.text.clone(),
                        level: heading.level,
                        content: Vec::new(),
                        subsections: Vec::new(),
                        page_range: None,
                    };
                    section.subsections.push(subsection);
                }
            }
        }

        // Add final section
        if let Some(section) = current_section {
            sections.push(section);
        }

        debug!("Built {} sections from headings", sections.len());
        Ok(sections)
    }

    /// Extract table of contents
    pub fn extract_table_of_contents(&self, docx: &Docx) -> Result<Option<types::TableOfContents>> {
        // TODO: Implement TOC extraction
        // This would look for TOC fields and extract the table of contents structure
        Ok(None)
    }

    /// Extract cross references
    pub fn extract_cross_references(&self, docx: &Docx) -> Result<Vec<CrossReference>> {
        let mut cross_references = Vec::new();

        // TODO: Implement cross-reference extraction
        // This would look for cross-reference fields and extract their targets

        Ok(cross_references)
    }

    /// Extract bookmarks
    pub fn extract_bookmarks(&self, docx: &Docx) -> Result<Vec<Bookmark>> {
        let mut bookmarks = Vec::new();

        // TODO: Implement bookmark extraction
        // This would traverse the document and extract bookmark markers

        Ok(bookmarks)
    }

    /// Analyze document outline
    pub fn analyze_outline(&self, headings: &[DocumentHeading]) -> DocumentOutline {
        let mut outline_levels = vec![0; 7]; // Support up to 6 heading levels

        for heading in headings {
            if heading.level <= 6 {
                outline_levels[heading.level] += 1;
            }
        }

        let max_depth = outline_levels.iter()
            .rposition(|&count| count > 0)
            .unwrap_or(0);

        let total_headings = headings.len();
        let avg_headings_per_level = if max_depth > 0 {
            total_headings as f64 / max_depth as f64
        } else {
            0.0
        };

        DocumentOutline {
            total_headings,
            max_depth,
            level_counts: outline_levels[1..=6].to_vec(), // Skip level 0
            avg_headings_per_level,
            is_well_structured: max_depth > 0 && total_headings >= max_depth,
        }
    }

    /// Validate document structure
    pub fn validate_structure(&self, structure: &DocumentStructure) -> Vec<String> {
        let mut warnings = Vec::new();

        // Check for missing headings
        if structure.headings.is_empty() {
            warnings.push("Document has no headings - structure may be unclear".to_string());
        }

        // Check for heading level gaps
        let mut levels: Vec<usize> = structure.headings.iter().map(|h| h.level).collect();
        levels.sort_unstable();
        levels.dedup();

        for i in 1..levels.len() {
            if levels[i] - levels[i-1] > 1 {
                warnings.push(format!(
                    "Heading level gap detected: level {} followed by level {}",
                    levels[i-1], levels[i]
                ));
            }
        }

        // Check for very deep nesting
        if let Some(&max_level) = levels.iter().max() {
            if max_level > 6 {
                warnings.push(format!(
                    "Very deep heading nesting detected (level {}). Consider restructuring.",
                    max_level
                ));
            }
        }

        // Check for sections without content
        for (i, section) in structure.sections.iter().enumerate() {
            if section.content.is_empty() && section.subsections.is_empty() {
                warnings.push(format!("Section {} '{}' appears to be empty", i + 1, section.title));
            }
        }

        warnings
    }
}

/// Document outline analysis
#[derive(Debug, Clone)]
pub struct DocumentOutline {
    /// Total number of headings
    pub total_headings: usize,
    /// Maximum heading depth
    pub max_depth: usize,
    /// Count of headings at each level (1-6)
    pub level_counts: Vec<usize>,
    /// Average headings per level
    pub avg_headings_per_level: f64,
    /// Whether the document is well-structured
    pub is_well_structured: bool,
}
