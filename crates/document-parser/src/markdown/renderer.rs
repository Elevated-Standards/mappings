// Modified: 2025-09-22

//! Markdown rendering functionality
//!
//! This module provides custom rendering capabilities for Markdown documents,
//! including HTML and plain text rendering with custom processing.

use fedramp_core::Result;
use pulldown_cmark::{Parser, Event};

use super::types::MarkdownParserConfig;

/// Custom Markdown renderer
#[derive(Debug, Clone)]
pub struct CustomRenderer {
    /// Configuration
    config: MarkdownParserConfig,
}

impl CustomRenderer {
    /// Create a new custom renderer
    pub fn new(config: MarkdownParserConfig) -> Self {
        Self { config }
    }

    /// Render Markdown to HTML with custom processing
    pub fn render_to_html(&self, markdown: &str) -> Result<String> {
        let parser = Parser::new(markdown);
        let mut html = String::new();
        pulldown_cmark::html::push_html(&mut html, parser);
        Ok(html)
    }

    /// Render Markdown to plain text
    pub fn render_to_text(&self, markdown: &str) -> Result<String> {
        let parser = Parser::new(markdown);
        let mut text = String::new();

        for event in parser {
            match event {
                Event::Text(t) | Event::Code(t) => text.push_str(&t),
                Event::SoftBreak | Event::HardBreak => text.push(' '),
                _ => {}
            }
        }

        Ok(text)
    }
}
