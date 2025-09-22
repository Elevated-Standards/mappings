// Modified: 2025-09-22

//! Markdown validation functionality
//!
//! This module provides validation capabilities for Markdown documents,
//! including structure validation, content quality checks, and link validation.

use fedramp_core::Result;
use url::Url;

use super::types::MarkdownDocument;

/// Validate Markdown document content
pub async fn validate_markdown_content(content: &serde_json::Value) -> Result<Vec<String>> {
    let mut errors = Vec::new();

    // Validate document structure
    if let Some(markdown_doc) = content.as_object() {
        // Check for required content
        if let Some(content_obj) = markdown_doc.get("content").and_then(|v| v.as_object()) {
            if let Some(plain_text) = content_obj.get("plain_text").and_then(|v| v.as_str()) {
                if plain_text.trim().is_empty() {
                    errors.push("Document contains no readable text content".to_string());
                }
            } else {
                errors.push("Document missing plain text content".to_string());
            }

            if let Some(word_count) = content_obj.get("word_count").and_then(|v| v.as_u64()) {
                if word_count < 10 {
                    errors.push("Document has very little content (less than 10 words)".to_string());
                }
            }
        }

        // Check structure quality
        if let Some(structure) = markdown_doc.get("structure").and_then(|v| v.as_object()) {
            if let Some(headings) = structure.get("headings").and_then(|v| v.as_array()) {
                if headings.is_empty() {
                    errors.push("Document has no headings - consider adding structure".to_string());
                }

                // Check heading hierarchy
                let mut prev_level = 0;
                for heading in headings {
                    if let Some(level) = heading.get("level").and_then(|v| v.as_u64()) {
                        if level as usize > prev_level + 1 && prev_level > 0 {
                            errors.push("Document has inconsistent heading hierarchy".to_string());
                            break;
                        }
                        prev_level = level as usize;
                    }
                }
            }
        }

        // Check metadata quality
        if let Some(metadata) = markdown_doc.get("metadata").and_then(|v| v.as_object()) {
            if metadata.get("title").is_none() || metadata.get("title").and_then(|v| v.as_str()).map_or(true, |s| s.is_empty()) {
                errors.push("Document missing title - consider adding a title".to_string());
            }
        }

        // Validate links
        if let Some(links) = markdown_doc.get("links").and_then(|v| v.as_array()) {
            for link in links {
                if let Some(url) = link.get("url").and_then(|v| v.as_str()) {
                    if !url.starts_with("http://") && !url.starts_with("https://") && !url.starts_with("#") && !url.starts_with("/") {
                        if Url::parse(url).is_err() {
                            errors.push(format!("Invalid URL format: {}", url));
                        }
                    }
                }
            }
        }
    } else {
        errors.push("Invalid Markdown document structure".to_string());
    }

    Ok(errors)
}

/// Calculate quality score for parsed document
pub fn calculate_quality_score(markdown_document: &MarkdownDocument) -> f64 {
    let mut score = 0.0;
    let mut max_score = 0.0;

    // Content completeness (40% of score)
    max_score += 40.0;
    if !markdown_document.content.plain_text.is_empty() {
        score += 20.0;

        // Bonus for substantial content
        if markdown_document.content.word_count > 100 {
            score += 10.0;
        }
        if markdown_document.content.word_count > 500 {
            score += 10.0;
        }
    }

    // Structure quality (30% of score)
    max_score += 30.0;
    if !markdown_document.structure.headings.is_empty() {
        score += 15.0;

        // Bonus for well-structured documents
        if markdown_document.structure.headings.len() > 3 {
            score += 10.0;
        }
        if !markdown_document.structure.sections.is_empty() {
            score += 5.0;
        }
    }

    // Metadata completeness (20% of score)
    max_score += 20.0;
    let mut metadata_fields = 0;
    if markdown_document.metadata.title.is_some() { metadata_fields += 1; }
    if markdown_document.metadata.author.is_some() { metadata_fields += 1; }
    if markdown_document.metadata.description.is_some() { metadata_fields += 1; }
    if !markdown_document.metadata.tags.is_empty() { metadata_fields += 1; }

    score += (metadata_fields as f64 / 4.0) * 20.0;

    // Rich content (10% of score)
    max_score += 10.0;
    if !markdown_document.tables.is_empty() {
        score += 3.0;
    }
    if !markdown_document.code_blocks.is_empty() {
        score += 3.0;
    }
    if !markdown_document.links.is_empty() {
        score += 2.0;
    }
    if !markdown_document.images.is_empty() {
        score += 2.0;
    }

    // Normalize score to 0.0-1.0 range
    if max_score > 0.0 {
        score / max_score
    } else {
        0.0
    }
}
