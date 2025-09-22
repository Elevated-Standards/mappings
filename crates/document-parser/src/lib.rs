// Modified: 2025-09-20

//! # Document Parser
//!
//! Document parsing and OSCAL conversion for FedRAMP compliance documents.
//! Supports Excel, Word, and Markdown document formats.

pub mod excel;
pub mod word;
pub mod poam;

#[cfg(test)]
pub mod word_tests;
#[cfg(test)]
pub mod oscal_tests;
pub mod markdown;
pub mod mapping;
pub mod fuzzy;
pub mod validation;
pub mod oscal;
pub mod quality;
pub mod error;

use async_trait::async_trait;
use fedramp_core::{Result, Error};
use std::path::Path;
use tokio::fs;

pub use error::DocumentParserError;
pub use excel::ExcelParser;
pub use word::WordParser;
pub use markdown::MarkdownParser;
pub use mapping::ColumnMapper;
pub use validation::DocumentValidator;
pub use oscal::OscalGenerator;
pub use quality::PoamQualityChecker;

/// Supported document types
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DocumentType {
    Excel,
    Word,
    Markdown,
    Json,
}

impl DocumentType {
    /// Detect document type from file extension
    pub fn from_extension(path: &Path) -> Option<Self> {
        match path.extension()?.to_str()? {
            "xlsx" | "xls" => Some(Self::Excel),
            "docx" => Some(Self::Word),
            "md" | "markdown" => Some(Self::Markdown),
            "json" => Some(Self::Json),
            _ => None,
        }
    }
}

/// Document parsing result
#[derive(Debug, Clone)]
pub struct ParseResult {
    pub document_type: DocumentType,
    pub source_path: String,
    pub metadata: serde_json::Value,
    pub content: serde_json::Value,
    pub validation_errors: Vec<String>,
    pub quality_score: f64,
}

/// Trait for document parsers
#[async_trait]
pub trait DocumentParser {
    /// Parse a document from file path
    async fn parse_file(&self, path: &Path) -> Result<ParseResult>;
    
    /// Parse a document from bytes
    async fn parse_bytes(&self, data: &[u8], filename: &str) -> Result<ParseResult>;
    
    /// Validate parsed content
    async fn validate(&self, content: &serde_json::Value) -> Result<Vec<String>>;
    
    /// Get supported file extensions
    fn supported_extensions(&self) -> Vec<&'static str>;
}

/// Main document parser factory
pub struct DocumentParserFactory {
    excel_parser: ExcelParser,
    word_parser: WordParser,
    markdown_parser: MarkdownParser,
}

impl DocumentParserFactory {
    /// Create a new document parser factory
    pub fn new() -> Self {
        Self {
            excel_parser: ExcelParser::new(),
            word_parser: WordParser::new(),
            markdown_parser: MarkdownParser::new(),
        }
    }

    /// Parse a document based on its type
    pub async fn parse_document(&self, path: &Path) -> Result<ParseResult> {
        let document_type = DocumentType::from_extension(path)
            .ok_or_else(|| Error::document_parsing("Unsupported file type"))?;

        match document_type {
            DocumentType::Excel => self.excel_parser.parse_file(path).await,
            DocumentType::Word => self.word_parser.parse_file(path).await,
            DocumentType::Markdown => self.markdown_parser.parse_file(path).await,
            DocumentType::Json => {
                let content = fs::read_to_string(path).await?;
                let json: serde_json::Value = serde_json::from_str(&content)?;
                Ok(ParseResult {
                    document_type,
                    source_path: path.to_string_lossy().to_string(),
                    metadata: serde_json::json!({}),
                    content: json,
                    validation_errors: Vec::new(),
                    quality_score: 1.0,
                })
            }
        }
    }

    /// Parse document from bytes with filename hint
    pub async fn parse_bytes(&self, data: &[u8], filename: &str) -> Result<ParseResult> {
        let path = Path::new(filename);
        let document_type = DocumentType::from_extension(path)
            .ok_or_else(|| Error::document_parsing("Unsupported file type"))?;

        match document_type {
            DocumentType::Excel => self.excel_parser.parse_bytes(data, filename).await,
            DocumentType::Word => self.word_parser.parse_bytes(data, filename).await,
            DocumentType::Markdown => self.markdown_parser.parse_bytes(data, filename).await,
            DocumentType::Json => {
                let content = String::from_utf8(data.to_vec())
                    .map_err(|e| Error::document_parsing(format!("Invalid UTF-8: {}", e)))?;
                let json: serde_json::Value = serde_json::from_str(&content)?;
                Ok(ParseResult {
                    document_type,
                    source_path: filename.to_string(),
                    metadata: serde_json::json!({}),
                    content: json,
                    validation_errors: Vec::new(),
                    quality_score: 1.0,
                })
            }
        }
    }

    /// Get all supported file extensions
    pub fn supported_extensions(&self) -> Vec<&'static str> {
        let mut extensions = Vec::new();
        extensions.extend(self.excel_parser.supported_extensions());
        extensions.extend(self.word_parser.supported_extensions());
        extensions.extend(self.markdown_parser.supported_extensions());
        extensions.push("json");
        extensions
    }
}

impl Default for DocumentParserFactory {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_document_type_detection() {
        assert_eq!(
            DocumentType::from_extension(&PathBuf::from("test.xlsx")),
            Some(DocumentType::Excel)
        );
        assert_eq!(
            DocumentType::from_extension(&PathBuf::from("test.docx")),
            Some(DocumentType::Word)
        );
        assert_eq!(
            DocumentType::from_extension(&PathBuf::from("test.md")),
            Some(DocumentType::Markdown)
        );
        assert_eq!(
            DocumentType::from_extension(&PathBuf::from("test.json")),
            Some(DocumentType::Json)
        );
        assert_eq!(
            DocumentType::from_extension(&PathBuf::from("test.txt")),
            None
        );
    }

    #[test]
    fn test_factory_creation() {
        let factory = DocumentParserFactory::new();
        let extensions = factory.supported_extensions();
        assert!(extensions.contains(&"xlsx"));
        assert!(extensions.contains(&"docx"));
        assert!(extensions.contains(&"md"));
        assert!(extensions.contains(&"json"));
    }
}
