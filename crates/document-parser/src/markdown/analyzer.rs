// Modified: 2025-09-22

//! Markdown structure analysis functionality
//!
//! This module provides structure analysis capabilities for Markdown documents,
//! including heading extraction, section building, and outline generation.

use fedramp_core::Result;
use pulldown_cmark::{Parser, Event, Tag};
use std::collections::HashMap;

use super::types::*;

/// Markdown structure analyzer
#[derive(Debug, Clone)]
pub struct MarkdownStructureAnalyzer {
    /// Configuration
    config: MarkdownParserConfig,
}

impl MarkdownStructureAnalyzer {
    /// Create a new structure analyzer
    pub fn new(config: MarkdownParserConfig) -> Self {
        Self { config }
    }

    /// Analyze document structure
    pub fn analyze_structure(&self, markdown: &str) -> Result<MarkdownStructure> {
        let headings = self.extract_headings(markdown)?;
        let sections = self.build_sections(&headings, markdown)?;
        let table_of_contents = self.generate_table_of_contents(&headings)?;
        let cross_references = self.extract_cross_references(markdown)?;
        let outline = self.build_outline(&headings)?;

        Ok(MarkdownStructure {
            headings,
            sections,
            table_of_contents: Some(table_of_contents),
            cross_references,
            outline,
        })
    }

    /// Extract headings from Markdown
    fn extract_headings(&self, markdown: &str) -> Result<Vec<MarkdownHeading>> {
        let mut headings = Vec::new();
        let parser = Parser::new(markdown);
        let mut in_heading = false;
        let mut current_level = 1;
        let mut heading_text = String::new();

        for event in parser {
            match event {
                Event::Start(Tag::Heading(level, _, _)) => {
                    in_heading = true;
                    current_level = level as usize;
                    heading_text.clear();
                }
                Event::End(Tag::Heading(_, _, _)) => {
                    if in_heading && !heading_text.is_empty() {
                        let id = self.generate_heading_id(&heading_text);
                        headings.push(MarkdownHeading {
                            text: heading_text.trim().to_string(),
                            level: current_level,
                            id: Some(id),
                            location: DocumentLocation {
                                line: None,
                                column: None,
                                offset: None,
                            },
                        });
                    }
                    in_heading = false;
                }
                Event::Text(text) if in_heading => {
                    heading_text.push_str(&text);
                }
                _ => {}
            }
        }

        Ok(headings)
    }

    /// Generate heading ID from text
    pub fn generate_heading_id(&self, text: &str) -> String {
        text.to_lowercase()
            .chars()
            .map(|c| if c.is_alphanumeric() { c } else { '-' })
            .collect::<String>()
            .split('-')
            .filter(|s| !s.is_empty())
            .collect::<Vec<_>>()
            .join("-")
    }

    /// Build sections from headings
    fn build_sections(&self, headings: &[MarkdownHeading], _markdown: &str) -> Result<Vec<MarkdownSection>> {
        let mut sections = Vec::new();
        let mut section_id = 0;

        for heading in headings {
            section_id += 1;
            sections.push(MarkdownSection {
                id: format!("section_{}", section_id),
                title: heading.text.clone(),
                level: heading.level,
                content: Vec::new(), // TODO: Extract section content
                subsections: Vec::new(), // TODO: Build subsection hierarchy
                metadata: SectionMetadata {
                    section_type: None,
                    control_id: None,
                    implementation_status: None,
                    responsibility: None,
                    custom: HashMap::new(),
                },
            });
        }

        Ok(sections)
    }

    /// Generate table of contents
    fn generate_table_of_contents(&self, headings: &[MarkdownHeading]) -> Result<TableOfContents> {
        let entries = headings.iter()
            .filter(|h| h.level <= self.config.max_heading_depth)
            .map(|h| TocEntry {
                text: h.text.clone(),
                level: h.level,
                anchor: h.id.clone(),
                location: h.location.clone(),
            })
            .collect();

        let max_depth = headings.iter()
            .map(|h| h.level)
            .max()
            .unwrap_or(0);

        Ok(TableOfContents {
            entries,
            title: Some("Table of Contents".to_string()),
            max_depth,
        })
    }

    /// Extract cross references
    fn extract_cross_references(&self, _markdown: &str) -> Result<Vec<CrossReference>> {
        // TODO: Implement cross-reference extraction
        Ok(Vec::new())
    }

    /// Build document outline
    fn build_outline(&self, headings: &[MarkdownHeading]) -> Result<DocumentOutline> {
        let mut nodes = Vec::new();
        let mut stack: Vec<OutlineNode> = Vec::new();
        let max_depth = headings.iter().map(|h| h.level).max().unwrap_or(0);

        for heading in headings {
            let node = OutlineNode {
                title: heading.text.clone(),
                level: heading.level,
                children: Vec::new(),
                anchor: heading.id.clone(),
            };

            // Build hierarchy
            while let Some(last) = stack.last() {
                if last.level < heading.level {
                    break;
                }
                let completed_node = stack.pop().unwrap();
                if let Some(parent) = stack.last_mut() {
                    parent.children.push(completed_node);
                } else {
                    nodes.push(completed_node);
                }
            }

            stack.push(node);
        }

        // Add remaining nodes
        while let Some(node) = stack.pop() {
            if let Some(parent) = stack.last_mut() {
                parent.children.push(node);
            } else {
                nodes.push(node);
            }
        }

        Ok(DocumentOutline {
            nodes,
            depth: max_depth,
        })
    }
}
