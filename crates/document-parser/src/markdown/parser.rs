// Modified: 2025-09-22

//! Markdown document parser implementation
//!
//! This module provides the main Markdown parser implementation with comprehensive
//! SSP support and DocumentParser trait implementation.

use crate::{DocumentParser, ParseResult, DocumentType};
use async_trait::async_trait;
use fedramp_core::{Result, Error};
use std::path::Path;
use tracing::{info};

use super::types::*;
use super::extractor::MarkdownExtractor;
use super::analyzer::MarkdownStructureAnalyzer;
use super::renderer::CustomRenderer;
use super::validation::{validate_markdown_content, calculate_quality_score};
use super::utils::{extract_frontmatter, extract_metadata};

/// Markdown document parser implementation with comprehensive SSP support
#[derive(Debug, Clone)]
pub struct MarkdownParser {
    /// Maximum file size to process (in bytes)
    pub max_file_size: usize,
    /// Parser configuration
    pub config: MarkdownParserConfig,
    /// Content extractor
    content_extractor: MarkdownExtractor,
    /// Structure analyzer
    structure_analyzer: MarkdownStructureAnalyzer,
    /// Custom renderer
    renderer: CustomRenderer,
}

impl MarkdownParser {
    /// Create a new Markdown parser with default settings
    #[must_use]
    pub fn new() -> Self {
        let config = MarkdownParserConfig::default();
        Self {
            max_file_size: 10 * 1024 * 1024, // 10MB
            content_extractor: MarkdownExtractor::new(config.clone()),
            structure_analyzer: MarkdownStructureAnalyzer::new(config.clone()),
            renderer: CustomRenderer::new(config.clone()),
            config,
        }
    }

    /// Create a new Markdown parser with custom configuration
    #[must_use]
    pub fn with_config(max_file_size: usize, config: MarkdownParserConfig) -> Self {
        Self {
            max_file_size,
            content_extractor: MarkdownExtractor::new(config.clone()),
            structure_analyzer: MarkdownStructureAnalyzer::new(config.clone()),
            renderer: CustomRenderer::new(config.clone()),
            config,
        }
    }

    /// Parse Markdown document from file path
    pub async fn parse_markdown_file(&self, path: &Path) -> Result<MarkdownDocument> {
        info!("Parsing Markdown file: {}", path.display());

        // Read file
        let file_data = tokio::fs::read_to_string(path).await
            .map_err(|e| Error::document_parsing(format!("Failed to read file: {}", e)))?;

        // Check file size
        if file_data.len() > self.max_file_size {
            return Err(Error::document_parsing(format!(
                "File size {} exceeds maximum allowed size {}",
                file_data.len(),
                self.max_file_size
            )));
        }

        self.parse_markdown_content(&file_data, &path.to_string_lossy()).await
    }

    /// Parse Markdown document from string content
    pub async fn parse_markdown_content(&self, content: &str, filename: &str) -> Result<MarkdownDocument> {
        info!("Parsing Markdown content for file: {}", filename);

        // Extract frontmatter if enabled
        let (frontmatter, markdown_content) = if self.config.parse_frontmatter {
            extract_frontmatter(content)?
        } else {
            (std::collections::HashMap::new(), content.to_string())
        };

        // Extract metadata
        let metadata = extract_metadata(&markdown_content, frontmatter)?;

        // Extract content
        let content_result = self.content_extractor.extract_content(&markdown_content)?;

        // Analyze structure
        let structure = self.structure_analyzer.analyze_structure(&markdown_content)?;

        // Extract tables
        let tables = self.content_extractor.extract_tables(&markdown_content)?;

        // Extract code blocks
        let code_blocks = self.content_extractor.extract_code_blocks(&markdown_content)?;

        // Extract links
        let links = self.content_extractor.extract_links(&markdown_content)?;

        // Extract images
        let images = self.content_extractor.extract_images(&markdown_content)?;

        Ok(MarkdownDocument {
            metadata,
            structure,
            content: content_result,
            tables,
            code_blocks,
            links,
            images,
        })
    }

    /// Calculate quality score for parsed document
    pub fn calculate_quality_score(&self, markdown_document: &MarkdownDocument) -> f64 {
        calculate_quality_score(markdown_document)
    }
}

impl Default for MarkdownParser {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl DocumentParser for MarkdownParser {
    async fn parse_file(&self, path: &Path) -> Result<ParseResult> {
        info!("Parsing Markdown file: {}", path.display());

        // Parse the Markdown document
        let markdown_doc = self.parse_markdown_file(path).await?;

        // Calculate quality score
        let quality_score = self.calculate_quality_score(&markdown_doc);

        // Convert to ParseResult format
        let content = serde_json::to_value(&markdown_doc)
            .map_err(|e| Error::document_parsing(format!("Failed to serialize Markdown document: {}", e)))?;

        // Validate content
        let validation_errors = self.validate(&content).await?;

        Ok(ParseResult {
            document_type: DocumentType::Markdown,
            source_path: path.to_string_lossy().to_string(),
            metadata: serde_json::json!({
                "source_file": path.to_string_lossy(),
                "source_type": "markdown",
                "extraction_date": chrono::Utc::now().to_rfc3339(),
                "parser_version": env!("CARGO_PKG_VERSION"),
                "title": markdown_doc.metadata.title,
                "author": markdown_doc.metadata.author,
                "description": markdown_doc.metadata.description,
                "tags": markdown_doc.metadata.tags,
                "word_count": markdown_doc.content.word_count,
                "character_count": markdown_doc.content.character_count,
                "heading_count": markdown_doc.structure.headings.len(),
                "section_count": markdown_doc.structure.sections.len(),
                "table_count": markdown_doc.tables.len(),
                "code_block_count": markdown_doc.code_blocks.len(),
                "link_count": markdown_doc.links.len(),
                "image_count": markdown_doc.images.len()
            }),
            content,
            validation_errors,
            quality_score,
        })
    }

    async fn parse_bytes(&self, data: &[u8], filename: &str) -> Result<ParseResult> {
        info!("Parsing Markdown bytes for file: {}", filename);

        // Convert bytes to string
        let content = String::from_utf8(data.to_vec())
            .map_err(|e| Error::document_parsing(format!("Invalid UTF-8 in Markdown file: {}", e)))?;

        // Check file size
        if content.len() > self.max_file_size {
            return Err(Error::document_parsing(format!(
                "File size {} exceeds maximum allowed size {}",
                content.len(),
                self.max_file_size
            )));
        }

        // Parse the Markdown document
        let markdown_doc = self.parse_markdown_content(&content, filename).await?;

        // Calculate quality score
        let quality_score = self.calculate_quality_score(&markdown_doc);

        // Convert to ParseResult format
        let content_value = serde_json::to_value(&markdown_doc)
            .map_err(|e| Error::document_parsing(format!("Failed to serialize Markdown document: {}", e)))?;

        // Validate content
        let validation_errors = self.validate(&content_value).await?;

        Ok(ParseResult {
            document_type: DocumentType::Markdown,
            source_path: filename.to_string(),
            metadata: serde_json::json!({
                "source_file": filename,
                "source_type": "markdown",
                "extraction_date": chrono::Utc::now().to_rfc3339(),
                "parser_version": env!("CARGO_PKG_VERSION"),
                "title": markdown_doc.metadata.title,
                "author": markdown_doc.metadata.author,
                "description": markdown_doc.metadata.description,
                "tags": markdown_doc.metadata.tags,
                "word_count": markdown_doc.content.word_count,
                "character_count": markdown_doc.content.character_count,
                "heading_count": markdown_doc.structure.headings.len(),
                "section_count": markdown_doc.structure.sections.len(),
                "table_count": markdown_doc.tables.len(),
                "code_block_count": markdown_doc.code_blocks.len(),
                "link_count": markdown_doc.links.len(),
                "image_count": markdown_doc.images.len()
            }),
            content: content_value,
            validation_errors,
            quality_score,
        })
    }

    async fn validate(&self, content: &serde_json::Value) -> Result<Vec<String>> {
        validate_markdown_content(content).await
    }

    fn supported_extensions(&self) -> Vec<&'static str> {
        vec!["md", "markdown", "mdown", "mkd", "mkdn"]
    }
}
