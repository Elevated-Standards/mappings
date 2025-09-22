// Modified: 2025-09-22

//! Markdown utility functions
//!
//! This module provides utility functions for Markdown processing,
//! including frontmatter extraction, metadata processing, and text utilities.

use fedramp_core::{Result, Error};
use regex::Regex;
use std::collections::HashMap;
use pulldown_cmark::{Parser, Event, Tag};

use super::types::MarkdownMetadata;

/// Extract frontmatter from Markdown content
pub fn extract_frontmatter(content: &str) -> Result<(HashMap<String, serde_json::Value>, String)> {
    let frontmatter_regex = Regex::new(r"(?s)^---\s*\n(.*?)\n---\s*\n(.*)$").unwrap();

    if let Some(captures) = frontmatter_regex.captures(content) {
        let frontmatter_str = captures.get(1).unwrap().as_str();
        let markdown_content = captures.get(2).unwrap().as_str().to_string();

        // Parse YAML frontmatter
        let frontmatter: HashMap<String, serde_json::Value> = serde_yaml::from_str(frontmatter_str)
            .unwrap_or_else(|_| HashMap::new());

        Ok((frontmatter, markdown_content))
    } else {
        Ok((HashMap::new(), content.to_string()))
    }
}

/// Extract metadata from Markdown content and frontmatter
pub fn extract_metadata(content: &str, frontmatter: HashMap<String, serde_json::Value>) -> Result<MarkdownMetadata> {
    let mut metadata = MarkdownMetadata {
        title: None,
        author: None,
        description: None,
        tags: Vec::new(),
        created: None,
        modified: None,
        version: None,
        frontmatter: frontmatter.clone(),
    };

    // Extract from frontmatter
    if let Some(title) = frontmatter.get("title").and_then(|v| v.as_str()) {
        metadata.title = Some(title.to_string());
    }
    if let Some(author) = frontmatter.get("author").and_then(|v| v.as_str()) {
        metadata.author = Some(author.to_string());
    }
    if let Some(description) = frontmatter.get("description").and_then(|v| v.as_str()) {
        metadata.description = Some(description.to_string());
    }
    if let Some(tags) = frontmatter.get("tags").and_then(|v| v.as_array()) {
        metadata.tags = tags.iter()
            .filter_map(|v| v.as_str())
            .map(|s| s.to_string())
            .collect();
    }

    // Extract title from first heading if not in frontmatter
    if metadata.title.is_none() {
        if let Some(title) = extract_first_heading(content) {
            metadata.title = Some(title);
        }
    }

    Ok(metadata)
}

/// Extract first heading from content
pub fn extract_first_heading(content: &str) -> Option<String> {
    let parser = Parser::new(content);
    let mut in_heading = false;
    let mut heading_text = String::new();

    for event in parser {
        match event {
            Event::Start(Tag::Heading(_, _, _)) => {
                in_heading = true;
            }
            Event::End(Tag::Heading(_, _, _)) => {
                if in_heading && !heading_text.is_empty() {
                    return Some(heading_text.trim().to_string());
                }
                in_heading = false;
            }
            Event::Text(text) if in_heading => {
                heading_text.push_str(&text);
            }
            _ => {}
        }
    }

    None
}
