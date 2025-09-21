//! Tests for Word document parsing

#[cfg(test)]
mod tests {
    use super::super::word::*;
    use crate::DocumentParser;

    #[tokio::test]
    async fn test_word_parser_creation() {
        let parser = WordParser::new();
        assert_eq!(parser.max_file_size, 100 * 1024 * 1024);
        assert!(parser.config.extract_images);
        assert!(parser.config.extract_tables);
        assert!(parser.config.analyze_structure);
    }

    #[tokio::test]
    async fn test_word_parser_with_config() {
        let config = DocxParserConfig {
            extract_images: false,
            extract_tables: true,
            extract_headers_footers: false,
            preserve_formatting: true,
            max_nesting_depth: 5,
            analyze_structure: true,
        };
        
        let parser = WordParser::with_config(50 * 1024 * 1024, config.clone());
        assert_eq!(parser.max_file_size, 50 * 1024 * 1024);
        assert!(!parser.config.extract_images);
        assert!(parser.config.extract_tables);
        assert!(!parser.config.extract_headers_footers);
    }

    #[test]
    fn test_docx_parser_config_default() {
        let config = DocxParserConfig::default();
        assert!(config.extract_images);
        assert!(config.extract_tables);
        assert!(config.extract_headers_footers);
        assert!(config.preserve_formatting);
        assert_eq!(config.max_nesting_depth, 10);
        assert!(config.analyze_structure);
    }

    #[test]
    fn test_content_extractor_creation() {
        let config = DocxParserConfig::default();
        let extractor = ContentExtractor::new(config);
        assert!(extractor.config.extract_images);
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
    fn test_document_metadata_creation() {
        let metadata = DocumentMetadata {
            title: Some("Test Document".to_string()),
            author: Some("Test Author".to_string()),
            subject: Some("Test Subject".to_string()),
            description: Some("Test Description".to_string()),
            created: Some("2024-01-01T00:00:00Z".to_string()),
            modified: Some("2024-01-02T00:00:00Z".to_string()),
            version: Some("1.0".to_string()),
            language: Some("en-US".to_string()),
            custom_properties: std::collections::HashMap::new(),
        };
        
        assert_eq!(metadata.title, Some("Test Document".to_string()));
        assert_eq!(metadata.author, Some("Test Author".to_string()));
        assert!(metadata.custom_properties.is_empty());
    }

    #[test]
    fn test_document_structure_creation() {
        let structure = DocumentStructure {
            sections: Vec::new(),
            headings: Vec::new(),
            table_of_contents: None,
            cross_references: Vec::new(),
            bookmarks: Vec::new(),
        };
        
        assert!(structure.sections.is_empty());
        assert!(structure.headings.is_empty());
        assert!(structure.table_of_contents.is_none());
    }

    #[test]
    fn test_document_content_creation() {
        let content = DocumentContent {
            text: "This is test content".to_string(),
            elements: Vec::new(),
            word_count: 4,
            character_count: 20,
        };
        
        assert_eq!(content.text, "This is test content");
        assert_eq!(content.word_count, 4);
        assert_eq!(content.character_count, 20);
    }

    #[test]
    fn test_document_element_creation() {
        let element = DocumentElement {
            element_type: ElementType::Paragraph,
            content: "Test paragraph content".to_string(),
            formatting: None,
            location: DocumentLocation {
                page: Some(1),
                paragraph: Some(0),
                character_offset: Some(0),
            },
        };
        
        assert_eq!(element.element_type, ElementType::Paragraph);
        assert_eq!(element.content, "Test paragraph content");
        assert!(element.formatting.is_none());
    }

    #[test]
    fn test_element_formatting_creation() {
        let formatting = ElementFormatting {
            bold: true,
            italic: false,
            underline: true,
            font_name: Some("Arial".to_string()),
            font_size: Some(12.0),
            color: Some("#000000".to_string()),
            background_color: None,
            alignment: Some("left".to_string()),
        };
        
        assert!(formatting.bold);
        assert!(!formatting.italic);
        assert!(formatting.underline);
        assert_eq!(formatting.font_name, Some("Arial".to_string()));
        assert_eq!(formatting.font_size, Some(12.0));
    }

    #[test]
    fn test_document_table_creation() {
        let table = DocumentTable {
            id: "table1".to_string(),
            title: Some("Test Table".to_string()),
            headers: vec!["Column 1".to_string(), "Column 2".to_string()],
            rows: vec![
                vec!["Row 1 Col 1".to_string(), "Row 1 Col 2".to_string()],
                vec!["Row 2 Col 1".to_string(), "Row 2 Col 2".to_string()],
            ],
            formatting: TableFormatting {
                style: Some("TableGrid".to_string()),
                border_style: Some("single".to_string()),
                cell_padding: Some(5.0),
                width: Some("100%".to_string()),
            },
            location: DocumentLocation {
                page: Some(1),
                paragraph: Some(5),
                character_offset: Some(100),
            },
        };
        
        assert_eq!(table.id, "table1");
        assert_eq!(table.headers.len(), 2);
        assert_eq!(table.rows.len(), 2);
        assert_eq!(table.rows[0].len(), 2);
    }

    #[test]
    fn test_document_image_creation() {
        let image = DocumentImage {
            id: "image1".to_string(),
            title: Some("Test Image".to_string()),
            description: Some("A test image".to_string()),
            format: Some("png".to_string()),
            size: Some(1024),
            dimensions: Some(ImageDimensions {
                width: 800,
                height: 600,
            }),
            location: DocumentLocation {
                page: Some(2),
                paragraph: Some(10),
                character_offset: Some(500),
            },
        };
        
        assert_eq!(image.id, "image1");
        assert_eq!(image.format, Some("png".to_string()));
        assert_eq!(image.size, Some(1024));
        assert!(image.dimensions.is_some());
        
        let dims = image.dimensions.unwrap();
        assert_eq!(dims.width, 800);
        assert_eq!(dims.height, 600);
    }

    #[test]
    fn test_document_relationship_creation() {
        let relationship = DocumentRelationship {
            relationship_type: RelationshipType::Hyperlink,
            source: "paragraph1".to_string(),
            target: "https://example.com".to_string(),
            description: Some("External link".to_string()),
        };
        
        assert_eq!(relationship.relationship_type, RelationshipType::Hyperlink);
        assert_eq!(relationship.source, "paragraph1");
        assert_eq!(relationship.target, "https://example.com");
    }

    #[test]
    fn test_element_type_variants() {
        let types = vec![
            ElementType::Paragraph,
            ElementType::Heading,
            ElementType::Table,
            ElementType::List,
            ElementType::Image,
            ElementType::Hyperlink,
            ElementType::Footnote,
            ElementType::Endnote,
        ];
        
        assert_eq!(types.len(), 8);
        assert_eq!(types[0], ElementType::Paragraph);
        assert_eq!(types[1], ElementType::Heading);
    }

    #[test]
    fn test_relationship_type_variants() {
        let types = vec![
            RelationshipType::Hyperlink,
            RelationshipType::CrossReference,
            RelationshipType::FootnoteReference,
            RelationshipType::EndnoteReference,
            RelationshipType::BookmarkReference,
        ];
        
        assert_eq!(types.len(), 5);
        assert_eq!(types[0], RelationshipType::Hyperlink);
        assert_eq!(types[1], RelationshipType::CrossReference);
    }

    #[test]
    fn test_document_heading_creation() {
        let heading = DocumentHeading {
            text: "Chapter 1: Introduction".to_string(),
            level: 1,
            style: Some("Heading1".to_string()),
            location: DocumentLocation {
                page: Some(1),
                paragraph: Some(0),
                character_offset: Some(0),
            },
        };
        
        assert_eq!(heading.text, "Chapter 1: Introduction");
        assert_eq!(heading.level, 1);
        assert_eq!(heading.style, Some("Heading1".to_string()));
    }

    #[test]
    fn test_table_of_contents_creation() {
        let toc = TableOfContents {
            entries: vec![
                TocEntry {
                    text: "Introduction".to_string(),
                    level: 1,
                    page_number: Some(1),
                    target: Some("heading1".to_string()),
                },
                TocEntry {
                    text: "Background".to_string(),
                    level: 2,
                    page_number: Some(2),
                    target: Some("heading2".to_string()),
                },
            ],
            title: Some("Table of Contents".to_string()),
        };
        
        assert_eq!(toc.entries.len(), 2);
        assert_eq!(toc.entries[0].text, "Introduction");
        assert_eq!(toc.entries[0].level, 1);
        assert_eq!(toc.title, Some("Table of Contents".to_string()));
    }

    #[test]
    fn test_bookmark_creation() {
        let bookmark = Bookmark {
            name: "bookmark1".to_string(),
            text: Some("Important Section".to_string()),
            location: DocumentLocation {
                page: Some(3),
                paragraph: Some(15),
                character_offset: Some(750),
            },
        };
        
        assert_eq!(bookmark.name, "bookmark1");
        assert_eq!(bookmark.text, Some("Important Section".to_string()));
        assert_eq!(bookmark.location.page, Some(3));
    }

    #[tokio::test]
    async fn test_parser_supported_extensions() {
        let parser = WordParser::new();
        let extensions = parser.supported_extensions();
        assert_eq!(extensions, vec!["docx"]);
    }



    #[tokio::test]
    async fn test_quality_score_calculation() {
        let parser = WordParser::new();
        
        // Create a test document with good content
        let docx_document = DocxDocument {
            metadata: DocumentMetadata {
                title: Some("Test Document".to_string()),
                author: Some("Test Author".to_string()),
                subject: Some("Test Subject".to_string()),
                description: None,
                created: Some("2024-01-01T00:00:00Z".to_string()),
                modified: Some("2024-01-02T00:00:00Z".to_string()),
                version: None,
                language: None,
                custom_properties: std::collections::HashMap::new(),
            },
            structure: DocumentStructure {
                sections: vec![],
                headings: vec![
                    DocumentHeading {
                        text: "Introduction".to_string(),
                        level: 1,
                        style: Some("Heading1".to_string()),
                        location: DocumentLocation {
                            page: Some(1),
                            paragraph: Some(0),
                            character_offset: Some(0),
                        },
                    },
                ],
                table_of_contents: None,
                cross_references: vec![],
                bookmarks: vec![],
            },
            content: DocumentContent {
                text: "This is a test document with substantial content that should score well in quality assessment.".to_string(),
                elements: vec![],
                word_count: 150,
                character_count: 500,
            },
            tables: vec![],
            images: vec![],
            relationships: vec![],
        };
        
        let quality_score = parser.calculate_quality_score(&docx_document);
        assert!(quality_score > 0.5);
        assert!(quality_score <= 1.0);
    }
}
