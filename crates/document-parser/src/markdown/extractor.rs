// Modified: 2025-09-22

//! Markdown content extraction functionality
//!
//! This module provides content extraction capabilities for Markdown documents,
//! including text extraction, element parsing, and content analysis.

use fedramp_core::Result;
use pulldown_cmark::{Parser, Event, Tag};
use std::collections::HashMap;

use super::types::*;

/// Markdown content extractor
#[derive(Debug, Clone)]
pub struct MarkdownExtractor {
    /// Configuration
    config: MarkdownParserConfig,
}

impl MarkdownExtractor {
    /// Create a new content extractor
    pub fn new(config: MarkdownParserConfig) -> Self {
        Self { config }
    }

    /// Extract content from Markdown
    pub fn extract_content(&self, markdown: &str) -> Result<MarkdownContent> {
        let parser = Parser::new(markdown);
        let mut elements = Vec::new();
        let mut plain_text = String::new();
        let mut html = String::new();

        // Convert to HTML
        let parser_for_html = Parser::new(markdown);
        pulldown_cmark::html::push_html(&mut html, parser_for_html);

        // Extract elements and plain text
        let mut current_element_type = None;
        let mut current_content = String::new();

        for event in parser {
            match event {
                Event::Start(tag) => {
                    current_element_type = Some(self.tag_to_element_type(&tag));
                    current_content.clear();
                }
                Event::End(_) => {
                    if let Some(element_type) = current_element_type.take() {
                        if !current_content.trim().is_empty() {
                            elements.push(MarkdownElement {
                                element_type,
                                content: current_content.trim().to_string(),
                                attributes: HashMap::new(),
                                location: DocumentLocation {
                                    line: None,
                                    column: None,
                                    offset: None,
                                },
                            });
                        }
                    }
                }
                Event::Text(text) => {
                    current_content.push_str(&text);
                    plain_text.push_str(&text);
                }
                Event::Code(code) => {
                    current_content.push_str(&code);
                    plain_text.push_str(&code);
                }
                Event::SoftBreak | Event::HardBreak => {
                    current_content.push(' ');
                    plain_text.push(' ');
                }
                _ => {}
            }
        }

        let word_count = plain_text.split_whitespace().count();
        let character_count = plain_text.chars().count();

        Ok(MarkdownContent {
            raw_markdown: markdown.to_string(),
            plain_text,
            html,
            elements,
            word_count,
            character_count,
        })
    }

    /// Convert pulldown-cmark tag to element type
    fn tag_to_element_type(&self, tag: &Tag) -> MarkdownElementType {
        match tag {
            Tag::Paragraph => MarkdownElementType::Paragraph,
            Tag::Heading(_, _, _) => MarkdownElementType::Heading,
            Tag::List(_) => MarkdownElementType::List,
            Tag::Item => MarkdownElementType::ListItem,
            Tag::Table(_) => MarkdownElementType::Table,
            Tag::CodeBlock(_) => MarkdownElementType::CodeBlock,
            Tag::BlockQuote => MarkdownElementType::Blockquote,
            Tag::Link(_, _, _) => MarkdownElementType::Link,
            Tag::Image(_, _, _) => MarkdownElementType::Image,
            Tag::Emphasis => MarkdownElementType::Emphasis,
            Tag::Strong => MarkdownElementType::Strong,
            _ => MarkdownElementType::Paragraph,
        }
    }

    /// Extract tables from Markdown content
    pub fn extract_tables(&self, content: &str) -> Result<Vec<MarkdownTable>> {
        let mut tables = Vec::new();

        if !self.config.extract_tables {
            return Ok(tables);
        }

        let parser = Parser::new(content);
        let mut in_table = false;
        let mut in_table_head = false;
        let mut current_table_headers = Vec::new();
        let mut current_table_rows = Vec::new();
        let mut current_row = Vec::new();
        let mut current_cell = String::new();
        let mut table_id = 0;

        for event in parser {
            match event {
                Event::Start(Tag::Table(_)) => {
                    in_table = true;
                    in_table_head = false;
                    current_table_headers.clear();
                    current_table_rows.clear();
                    table_id += 1;
                }
                Event::End(Tag::Table(_)) => {
                    if in_table {
                        tables.push(MarkdownTable {
                            id: format!("table_{}", table_id),
                            caption: None,
                            headers: current_table_headers.clone(),
                            rows: current_table_rows.clone(),
                            alignment: vec![TableAlignment::None; current_table_headers.len()],
                            location: DocumentLocation {
                                line: None,
                                column: None,
                                offset: None,
                            },
                        });
                    }
                    in_table = false;
                    in_table_head = false;
                }
                Event::Start(Tag::TableHead) => {
                    in_table_head = true;
                }
                Event::End(Tag::TableHead) => {
                    in_table_head = false;
                }
                Event::Start(Tag::TableRow) => {
                    current_row.clear();
                }
                Event::End(Tag::TableRow) => {
                    if in_table {
                        if in_table_head || current_table_headers.is_empty() {
                            current_table_headers = current_row.clone();
                        } else {
                            current_table_rows.push(current_row.clone());
                        }
                    }
                }
                Event::Start(Tag::TableCell) => {
                    current_cell.clear();
                }
                Event::End(Tag::TableCell) => {
                    if in_table {
                        current_row.push(current_cell.trim().to_string());
                        current_cell.clear();
                    }
                }
                Event::Text(text) if in_table => {
                    current_cell.push_str(&text);
                }
                _ => {}
            }
        }

        Ok(tables)
    }

    /// Extract code blocks from Markdown content
    pub fn extract_code_blocks(&self, content: &str) -> Result<Vec<CodeBlock>> {
        let mut code_blocks = Vec::new();

        if !self.config.extract_code_blocks {
            return Ok(code_blocks);
        }

        let parser = Parser::new(content);
        let mut code_block_id = 0;

        for event in parser {
            if let Event::Start(Tag::CodeBlock(kind)) = event {
                code_block_id += 1;
                let language = match kind {
                    pulldown_cmark::CodeBlockKind::Fenced(lang) => {
                        if lang.is_empty() {
                            None
                        } else {
                            Some(lang.to_string())
                        }
                    }
                    pulldown_cmark::CodeBlockKind::Indented => None,
                };

                // Extract code content (this is simplified - in practice we'd need to track the next Text event)
                code_blocks.push(CodeBlock {
                    id: format!("code_{}", code_block_id),
                    language,
                    code: String::new(), // TODO: Extract actual code content
                    metadata: HashMap::new(),
                    location: DocumentLocation {
                        line: None,
                        column: None,
                        offset: None,
                    },
                });
            }
        }

        Ok(code_blocks)
    }

    /// Extract links from Markdown content
    pub fn extract_links(&self, content: &str) -> Result<Vec<MarkdownLink>> {
        let mut links = Vec::new();

        if !self.config.extract_links {
            return Ok(links);
        }

        let parser = Parser::new(content);
        let mut in_link = false;
        let mut link_text = String::new();
        let mut link_url = String::new();
        let mut link_title = None;

        for event in parser {
            match event {
                Event::Start(Tag::Link(_, dest_url, title)) => {
                    in_link = true;
                    link_text.clear();
                    link_url = dest_url.to_string();
                    link_title = if title.is_empty() { None } else { Some(title.to_string()) };
                }
                Event::End(Tag::Link(_, _, _)) => {
                    if in_link {
                        links.push(MarkdownLink {
                            text: link_text.clone(),
                            url: link_url.clone(),
                            title: link_title.clone(),
                            link_type: MarkdownLinkType::Inline, // Simplified
                            location: DocumentLocation {
                                line: None,
                                column: None,
                                offset: None,
                            },
                        });
                    }
                    in_link = false;
                }
                Event::Text(text) if in_link => {
                    link_text.push_str(&text);
                }
                _ => {}
            }
        }

        Ok(links)
    }

    /// Extract images from Markdown content
    pub fn extract_images(&self, content: &str) -> Result<Vec<MarkdownImage>> {
        let mut images = Vec::new();

        let parser = Parser::new(content);

        for event in parser {
            if let Event::Start(Tag::Image(_, dest_url, title)) = event {
                images.push(MarkdownImage {
                    alt_text: String::new(), // TODO: Extract alt text
                    url: dest_url.to_string(),
                    title: if title.is_empty() { None } else { Some(title.to_string()) },
                    dimensions: None,
                    location: DocumentLocation {
                        line: None,
                        column: None,
                        offset: None,
                    },
                });
            }
        }

        Ok(images)
    }
}
