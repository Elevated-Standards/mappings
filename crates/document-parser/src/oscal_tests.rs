//! Tests for OSCAL generation functionality

#[cfg(test)]
mod tests {
    use super::super::oscal::*;
    use serde_json::json;
    use std::collections::HashMap;

    #[tokio::test]
    async fn test_oscal_generator_creation() {
        let generator = OscalGenerator::new();
        assert_eq!(generator.oscal_version, "1.1.2");
        assert!(generator.validate_output);
    }

    #[tokio::test]
    async fn test_oscal_generator_with_config() {
        let generator = OscalGenerator::with_config("1.0.0".to_string(), false);
        assert_eq!(generator.oscal_version, "1.0.0");
        assert!(!generator.validate_output);
    }

    #[tokio::test]
    async fn test_poam_generation_basic() {
        let generator = OscalGenerator::new();
        
        let content = json!({
            "poam_items": [
                {
                    "uuid": "test-uuid-1",
                    "title": "Test Vulnerability",
                    "description": "A test vulnerability description",
                    "severity": "High",
                    "status": "Open",
                    "control_id": "AC-1",
                    "point_of_contact": "John Doe"
                }
            ]
        });

        let metadata = json!({
            "source_file": "test.xlsx",
            "parser_version": "1.0.0",
            "system_name": "Test System"
        });

        let result = generator.generate_poam(&content, &metadata).unwrap();
        
        // Verify document structure
        assert!(result.get("plan-of-action-and-milestones").is_some());
        let poam_doc = result.get("plan-of-action-and-milestones").unwrap();
        
        assert!(poam_doc.get("uuid").is_some());
        assert!(poam_doc.get("metadata").is_some());
        assert!(poam_doc.get("system-id").is_some());
        assert!(poam_doc.get("poam-items").is_some());
        
        // Verify POA&M items
        let poam_items = poam_doc.get("poam-items").unwrap().as_array().unwrap();
        assert_eq!(poam_items.len(), 1);
        
        let item = &poam_items[0];
        assert_eq!(item.get("uuid").unwrap().as_str().unwrap(), "test-uuid-1");
        assert_eq!(item.get("title").unwrap().as_str().unwrap(), "Test Vulnerability");
        assert!(item.get("props").is_some());
    }

    #[tokio::test]
    async fn test_poam_generation_from_rows() {
        let generator = OscalGenerator::new();
        
        let content = json!({
            "rows": [
                ["POA-001", "SQL Injection Vulnerability", "Critical", "Open", "2024-12-31", "Jane Smith"],
                ["POA-002", "Cross-Site Scripting", "High", "In Progress", "2024-11-30", "Bob Johnson"]
            ]
        });

        let metadata = json!({
            "source_file": "poam.csv",
            "parser_version": "1.0.0"
        });

        let result = generator.generate_poam(&content, &metadata).unwrap();
        let poam_doc = result.get("plan-of-action-and-milestones").unwrap();
        let poam_items = poam_doc.get("poam-items").unwrap().as_array().unwrap();
        
        assert_eq!(poam_items.len(), 2);
        assert_eq!(poam_items[0].get("title").unwrap().as_str().unwrap(), "SQL Injection Vulnerability");
        assert_eq!(poam_items[1].get("title").unwrap().as_str().unwrap(), "Cross-Site Scripting");
    }

    #[tokio::test]
    async fn test_poam_generation_from_tables() {
        let generator = OscalGenerator::new();
        
        let content = json!({
            "tables": [
                {
                    "headers": ["ID", "Description", "Severity", "Status"],
                    "rows": [
                        ["POA-001", "Buffer Overflow", "High", "Open"],
                        ["POA-002", "Weak Authentication", "Medium", "Closed"]
                    ]
                }
            ]
        });

        let metadata = json!({
            "source_file": "document.md",
            "parser_version": "1.0.0"
        });

        let result = generator.generate_poam(&content, &metadata).unwrap();
        let poam_doc = result.get("plan-of-action-and-milestones").unwrap();
        let poam_items = poam_doc.get("poam-items").unwrap().as_array().unwrap();
        
        assert_eq!(poam_items.len(), 2);
        assert!(poam_items[0].get("uuid").is_some());
        assert!(poam_items[0].get("title").is_some());
    }

    #[tokio::test]
    async fn test_poam_item_processor() {
        let processor = PoamItemProcessor::new();
        
        let item_data = json!({
            "uuid": "test-item-1",
            "title": "Test Item",
            "description": "Test description",
            "severity": "Critical",
            "status": "Open",
            "control_id": "SC-7",
            "point_of_contact": "Security Team"
        });

        let result = processor.process_item(&item_data).unwrap();
        
        assert_eq!(result.uuid, "test-item-1");
        assert_eq!(result.title, "Test Item");
        assert_eq!(result.description, "Test description");
        assert!(result.props.is_some());
        
        let props = result.props.unwrap();
        assert!(props.iter().any(|p| p.name == "severity" && p.value == "Critical"));
        assert!(props.iter().any(|p| p.name == "status" && p.value == "Open"));
        assert!(props.iter().any(|p| p.name == "control-id" && p.value == "SC-7"));
    }

    #[tokio::test]
    async fn test_risk_processor() {
        let processor = RiskProcessor::new();
        
        let risk_data = json!({
            "uuid": "risk-1",
            "title": "High Risk Item",
            "description": "A high-risk security issue",
            "severity": "High",
            "likelihood": "Likely",
            "status": "open",
            "cve_id": "CVE-2023-1234"
        });

        let result = processor.process_risk(&risk_data).unwrap();
        
        assert_eq!(result.uuid, "risk-1");
        assert_eq!(result.title, "High Risk Item");
        assert_eq!(result.status, "open");
        assert!(result.threat_ids.is_some());
        assert!(result.characterizations.is_some());
        
        let threat_ids = result.threat_ids.unwrap();
        assert_eq!(threat_ids.len(), 1);
        assert_eq!(threat_ids[0].id, "CVE-2023-1234");
    }

    #[tokio::test]
    async fn test_observation_processor() {
        let processor = ObservationProcessor::new();
        
        let obs_data = json!({
            "uuid": "obs-1",
            "title": "Security Observation",
            "description": "Observed security issue",
            "methods": ["examine", "test"],
            "collected": "2024-01-15T10:00:00Z"
        });

        let result = processor.process_observation(&obs_data).unwrap();
        
        assert_eq!(result.uuid, "obs-1");
        assert_eq!(result.title, Some("Security Observation".to_string()));
        assert_eq!(result.description, "Observed security issue");
        assert_eq!(result.methods, vec!["examine", "test"]);
        assert!(result.expires.is_some());
    }

    #[tokio::test]
    async fn test_metadata_creation() {
        let generator = OscalGenerator::new();
        
        let source_metadata = json!({
            "source_file": "test.xlsx",
            "parser_version": "1.0.0",
            "extraction_date": "2024-01-15T10:00:00Z"
        });

        let result = generator.create_metadata(
            "Test Document",
            "2024-01-15T10:00:00Z",
            &source_metadata
        ).unwrap();

        assert_eq!(result.title, "Test Document");
        assert_eq!(result.oscal_version, "1.1.2");
        assert!(result.props.is_some());
        
        let props = result.props.unwrap();
        assert!(props.iter().any(|p| p.name == "source-file" && p.value == "test.xlsx"));
        assert!(props.iter().any(|p| p.name == "parser-version" && p.value == "1.0.0"));
    }

    #[tokio::test]
    async fn test_system_id_extraction() {
        let generator = OscalGenerator::new();
        
        // Test with explicit system ID
        let metadata1 = json!({
            "system_id": "test-system-123"
        });
        let result1 = generator.extract_system_id(&metadata1).unwrap();
        assert_eq!(result1.get("id").unwrap().as_str().unwrap(), "test-system-123");
        
        // Test with system name
        let metadata2 = json!({
            "system_name": "Test System Name"
        });
        let result2 = generator.extract_system_id(&metadata2).unwrap();
        assert_eq!(result2.get("id").unwrap().as_str().unwrap(), "test-system-name");
        
        // Test with no system info (should generate UUID)
        let metadata3 = json!({});
        let result3 = generator.extract_system_id(&metadata3).unwrap();
        assert!(result3.get("id").is_some());
    }

    #[tokio::test]
    async fn test_validation() {
        let generator = OscalGenerator::new();
        
        let valid_doc = json!({
            "plan-of-action-and-milestones": {
                "uuid": "test-uuid",
                "metadata": {
                    "title": "Test POA&M",
                    "last_modified": "2024-01-15T10:00:00Z",
                    "version": "1.0.0",
                    "oscal_version": "1.1.2"
                }
            }
        });

        let result = generator.validate_oscal_document(&valid_doc, &OscalDocumentType::PlanOfActionAndMilestones);
        assert!(result.is_ok());
        
        let invalid_doc = json!({
            "plan-of-action-and-milestones": {
                "metadata": {
                    "title": "Test POA&M"
                }
            }
        });

        let result2 = generator.validate_oscal_document(&invalid_doc, &OscalDocumentType::PlanOfActionAndMilestones);
        assert!(result2.is_err());
    }

    #[tokio::test]
    async fn test_json_formatting() {
        let generator = OscalGenerator::new();
        
        let doc = json!({
            "test": "value",
            "nested": {
                "key": "value"
            }
        });

        let formatted = generator.format_json(&doc).unwrap();
        assert!(formatted.contains("{\n"));
        assert!(formatted.contains("  \"test\": \"value\""));
    }

    #[tokio::test]
    async fn test_uuid_generation() {
        let uuid1 = OscalGenerator::generate_uuid();
        let uuid2 = OscalGenerator::generate_uuid();
        
        assert_ne!(uuid1, uuid2);
        assert_eq!(uuid1.len(), 36); // Standard UUID length
        assert!(uuid1.contains('-'));
    }

    #[test]
    fn test_processor_configs() {
        let poam_config = PoamProcessorConfig::default();
        assert_eq!(poam_config.default_status, "open");
        assert!(poam_config.generate_uuids);
        assert!(poam_config.include_tracking);

        let risk_config = RiskProcessorConfig::default();
        assert_eq!(risk_config.default_status, "open");
        assert!(risk_config.include_characterizations);
        assert!(risk_config.generate_threat_ids);

        let obs_config = ObservationProcessorConfig::default();
        assert!(obs_config.default_methods.contains(&"examine".to_string()));
        assert!(obs_config.include_evidence);
        assert_eq!(obs_config.default_expiration_days, Some(365));
    }

    #[tokio::test]
    async fn test_empty_content_handling() {
        let generator = OscalGenerator::new();
        
        let empty_content = json!({});
        let metadata = json!({
            "source_file": "empty.xlsx",
            "parser_version": "1.0.0"
        });

        let result = generator.generate_poam(&empty_content, &metadata).unwrap();
        let poam_doc = result.get("plan-of-action-and-milestones").unwrap();
        let poam_items = poam_doc.get("poam-items").unwrap().as_array().unwrap();
        
        assert_eq!(poam_items.len(), 0);
        assert!(poam_doc.get("uuid").is_some());
        assert!(poam_doc.get("metadata").is_some());
    }
}
