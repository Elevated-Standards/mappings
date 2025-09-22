// Modified: 2025-01-22

//! Word document parsing module
//!
//! This module provides comprehensive Word document parsing capabilities including:
//! - DOCX document parsing with docx-rs
//! - Content extraction and text analysis
//! - Document structure analysis and heading detection
//! - Metadata extraction and processing
//! - Table and image extraction
//! - Quality scoring and validation
//! - DocumentParser trait implementation for async parsing

pub mod types;
pub mod content;
pub mod structure;
pub mod metadata;
pub mod parser;
pub mod document_parser_impl;

// Re-export main types and structs for backward compatibility
pub use types::{
    DocxParserConfig,
    DocxDocument,
    DocumentMetadata,
    DocumentStructure,
    DocumentContent,
    DocumentElement,
    ElementType,
    ElementFormatting,
    DocumentLocation,
    DocumentTable,
    TableFormatting,
    DocumentImage,
    ImageDimensions,
    DocumentRelationship,
    RelationshipType,
    DocumentSection,
    DocumentHeading,
    TableOfContents,
    TocEntry,
    CrossReference,
    Bookmark,
    PageRange,
};

pub use content::{
    ContentExtractor,
    ContentStats,
};

pub use structure::{
    StructureAnalyzer,
    DocumentOutline,
};

pub use metadata::{
    MetadataProcessor,
    DocumentStatistics,
    SecurityMetadata,
};

pub use parser::WordParser;

// Tests module
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_word_parser_creation() {
        let parser = WordParser::new();
        assert_eq!(parser.max_file_size(), 100 * 1024 * 1024);
    }

    #[test]
    fn test_docx_parser_config_default() {
        let config = DocxParserConfig::default();
        assert!(config.extract_images);
        assert!(config.extract_tables);
        assert!(config.analyze_structure);
        assert!(config.extract_headers_footers);
        assert!(config.preserve_formatting);
        assert_eq!(config.max_nesting_depth, 10);
    }

    #[test]
    fn test_content_extractor_creation() {
        let config = DocxParserConfig::default();
        let extractor = ContentExtractor::new(config);
        assert!(extractor.config.extract_tables);
    }

    #[test]
    fn test_structure_analyzer_creation() {
        let config = DocxParserConfig::default();
        let analyzer = StructureAnalyzer::new(config);
        assert!(analyzer.config.analyze_structure);
    }

    #[test]
    fn test_metadata_processor_creation() {
        let config = DocxParserConfig::default();
        let processor = MetadataProcessor::new(config);
        assert!(processor.config.preserve_formatting);
    }

    #[test]
    fn test_document_element_types() {
        use ElementType::*;
        
        let paragraph = Paragraph;
        let heading = Heading;
        let list_item = ListItem;
        let table = Table;
        let image = Image;
        let hyperlink = Hyperlink;
        let footnote = Footnote;
        let endnote = Endnote;

        // Test that all element types can be created
        assert!(matches!(paragraph, Paragraph));
        assert!(matches!(heading, Heading));
        assert!(matches!(list_item, ListItem));
        assert!(matches!(table, Table));
        assert!(matches!(image, Image));
        assert!(matches!(hyperlink, Hyperlink));
        assert!(matches!(footnote, Footnote));
        assert!(matches!(endnote, Endnote));
    }

    #[test]
    fn test_relationship_types() {
        use RelationshipType::*;
        
        let hyperlink = Hyperlink;
        let cross_ref = CrossReference;
        let footnote_ref = FootnoteReference;
        let endnote_ref = EndnoteReference;
        let bookmark_ref = BookmarkReference;

        // Test that all relationship types can be created
        assert!(matches!(hyperlink, Hyperlink));
        assert!(matches!(cross_ref, CrossReference));
        assert!(matches!(footnote_ref, FootnoteReference));
        assert!(matches!(endnote_ref, EndnoteReference));
        assert!(matches!(bookmark_ref, BookmarkReference));
    }

    #[test]
    fn test_document_metadata_default() {
        use std::collections::HashMap;
        
        let metadata = DocumentMetadata {
            title: None,
            author: None,
            subject: None,
            description: None,
            created: None,
            modified: None,
            version: None,
            language: None,
            custom_properties: HashMap::new(),
        };

        assert!(metadata.title.is_none());
        assert!(metadata.author.is_none());
        assert!(metadata.custom_properties.is_empty());
    }

    #[test]
    fn test_document_location() {
        let location = DocumentLocation {
            page: Some(1),
            paragraph: Some(5),
            character_offset: Some(100),
        };

        assert_eq!(location.page, Some(1));
        assert_eq!(location.paragraph, Some(5));
        assert_eq!(location.character_offset, Some(100));
    }

    #[test]
    fn test_element_formatting() {
        let formatting = ElementFormatting {
            bold: true,
            italic: false,
            underline: true,
            font_name: Some("Arial".to_string()),
            font_size: Some(12.0),
            color: Some("#000000".to_string()),
            background_color: None,
        };

        assert!(formatting.bold);
        assert!(!formatting.italic);
        assert!(formatting.underline);
        assert_eq!(formatting.font_name, Some("Arial".to_string()));
        assert_eq!(formatting.font_size, Some(12.0));
    }

    #[test]
    fn test_image_dimensions() {
        let dimensions = ImageDimensions {
            width: 800,
            height: 600,
        };

        assert_eq!(dimensions.width, 800);
        assert_eq!(dimensions.height, 600);
    }

    #[test]
    fn test_page_range() {
        let page_range = PageRange {
            start: 1,
            end: 10,
        };

        assert_eq!(page_range.start, 1);
        assert_eq!(page_range.end, 10);
    }
}
