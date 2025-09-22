// Modified: 2025-01-22

//! DocumentParser trait implementation for Word documents
//!
//! This module provides the async DocumentParser trait implementation
//! for WordParser with comprehensive parsing and validation capabilities.

use super::parser::WordParser;
use crate::{DocumentParser, ParseResult, DocumentType};
use async_trait::async_trait;
use fedramp_core::Result;
use std::path::Path;
use tracing::{info, warn};

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

        // Validate document and get validation errors (before moving data)
        let validation_errors = self.validate_document(&docx_document);

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
            validation_errors,
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

        // Validate document and get validation errors (before moving data)
        let validation_errors = self.validate_document(&docx_document);

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
            validation_errors,
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
