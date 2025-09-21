//! Test cases for validation modules

#[cfg(test)]
mod tests {
    use uuid::Uuid;
    use chrono::Utc;

    // Import all the modules we need to test
    use crate::validation::types::*;
    use crate::validation::rules::*;
    use crate::validation::confidence::*;
    use crate::validation::overrides::*;
    use crate::validation::reports::*;
    use crate::validation::validators::*;

    #[test]
    fn test_validation_status_ordering() {
        assert!(ValidationSeverity::Critical > ValidationSeverity::Error);
        assert!(ValidationSeverity::Error > ValidationSeverity::Warning);
        assert!(ValidationSeverity::Warning > ValidationSeverity::Info);
    }

    #[test]
    fn test_quality_grade_ordering() {
        assert!(QualityGrade::A > QualityGrade::B);
        assert!(QualityGrade::B > QualityGrade::C);
        assert!(QualityGrade::C > QualityGrade::D);
        assert!(QualityGrade::D > QualityGrade::F);
    }

    #[test]
    fn test_risk_level_ordering() {
        assert!(RiskLevel::VeryHigh > RiskLevel::High);
        assert!(RiskLevel::High > RiskLevel::Medium);
        assert!(RiskLevel::Medium > RiskLevel::Low);
        assert!(RiskLevel::Low > RiskLevel::VeryLow);
    }

    #[test]
    fn test_issue_severity_ordering() {
        assert!(IssueSeverity::Critical > IssueSeverity::Error);
        assert!(IssueSeverity::Error > IssueSeverity::Warning);
        assert!(IssueSeverity::Warning > IssueSeverity::Info);
    }

    #[test]
    fn test_recommendation_priority_ordering() {
        assert!(RecommendationPriority::Critical > RecommendationPriority::High);
        assert!(RecommendationPriority::High > RecommendationPriority::Medium);
        assert!(RecommendationPriority::Medium > RecommendationPriority::Low);
    }

    #[test]
    fn test_effort_level_ordering() {
        assert!(EffortLevel::VeryHigh > EffortLevel::High);
        assert!(EffortLevel::High > EffortLevel::Medium);
        assert!(EffortLevel::Medium > EffortLevel::Low);
        assert!(EffortLevel::Low > EffortLevel::Minimal);
    }

    #[test]
    fn test_threshold_config_default() {
        let config = ThresholdConfig::default();
        assert_eq!(config.high_confidence, 0.9);
        assert_eq!(config.medium_confidence, 0.7);
        assert_eq!(config.low_confidence, 0.5);
        assert_eq!(config.min_acceptable, 0.6);
        assert_eq!(config.performance_threshold_ms, 100);
    }

    #[test]
    fn test_scoring_config_default() {
        let config = ScoringConfig::default();
        assert_eq!(config.min_acceptable_score, 0.6);
        assert_eq!(config.string_similarity_weight, 0.3);
        assert_eq!(config.semantic_similarity_weight, 0.25);
        assert_eq!(config.historical_success_weight, 0.2);
        assert_eq!(config.user_feedback_weight, 0.15);
        assert_eq!(config.data_type_weight, 0.1);
    }

    #[test]
    fn test_confidence_scorer_creation() {
        let _scorer = ConfidenceScorer::new();
        // Test that scorer can be created successfully
    }

    #[test]
    fn test_confidence_scorer_with_config() {
        let mut config = ScoringConfig::default();
        config.min_acceptable_score = 0.8;

        let _scorer = ConfidenceScorer::with_config(config);
        // Test that scorer can be created with custom config
    }

    #[test]
    fn test_confidence_calculation() {
        let scorer = ConfidenceScorer::new();
        let context = MappingContext {
            document_type: "inventory".to_string(),
            column_data: None,
            expected_data_type: None,
            column_position: None,
        };

        let result = scorer.calculate_confidence("Asset ID", "uuid", &context);
        assert!(result.is_ok());
        
        let confidence = result.unwrap();
        assert!(confidence.overall_score >= 0.0 && confidence.overall_score <= 1.0);
        assert!(!confidence.factor_scores.is_empty());
    }

    #[test]
    fn test_threshold_status_values() {
        // Test that threshold status values can be created
        let _high = ThresholdStatus::HighConfidence;
        let _medium = ThresholdStatus::MediumConfidence;
        let _low = ThresholdStatus::LowConfidence;
        let _very_low = ThresholdStatus::VeryLowConfidence;
    }

    #[test]
    fn test_mapping_override_engine_creation() {
        let engine = MappingOverrideEngine::new();
        assert_eq!(engine.get_metrics().total_applications, 0);
    }

    #[test]
    fn test_add_override_rule() {
        let mut engine = MappingOverrideEngine::new();
        
        let override_rule = MappingOverride {
            id: Uuid::new_v4(),
            name: "Test Override".to_string(),
            description: "Test override rule".to_string(),
            rule_type: OverrideType::ExactMatch,
            pattern: OverridePattern {
                pattern: "Asset ID".to_string(),
                case_sensitive: false,
                whole_word: true,
                regex_flags: None,
                fuzzy_threshold: None,
                position_constraints: None,
            },
            target_field: "uuid".to_string(),
            priority: 100,
            conditions: Vec::new(),
            scope: OverrideScope::Global,
            created_by: "test_user".to_string(),
            created_at: Utc::now(),
            modified_at: Utc::now(),
            active: true,
            version: 1,
            tags: vec!["test".to_string()],
        };

        let result = engine.add_override(override_rule);
        assert!(result.is_ok());
    }

    #[test]
    fn test_override_pattern_creation() {
        // Test that override patterns can be created
        let _pattern = OverridePattern {
            pattern: "Asset Name".to_string(),
            case_sensitive: false,
            whole_word: true,
            regex_flags: None,
            fuzzy_threshold: None,
            position_constraints: None,
        };
    }

    #[test]
    fn test_override_conditions_creation() {
        let _context = OverrideContext::new("inventory".to_string());

        let _conditions = vec![
            OverrideCondition {
                condition_type: ConditionType::DocumentType,
                field: "document_type".to_string(),
                operator: ConditionOperator::Equals,
                value: serde_json::Value::String("inventory".to_string()),
                required: true,
            }
        ];
    }

    #[test]
    fn test_override_scope_creation() {
        let mut _context = OverrideContext::new("inventory".to_string());
        _context.organization = Some("test_org".to_string());

        // Test scope types can be created
        let _global = OverrideScope::Global;
        let _doc_type = OverrideScope::DocumentType("inventory".to_string());
        let _org = OverrideScope::Organization("test_org".to_string());
    }

    #[test]
    fn test_override_conflict_types() {
        // Test that conflict types can be created
        let _priority_tie = ConflictType::PriorityTie;
        let _pattern_overlap = ConflictType::PatternOverlap;
        let _circular_dependency = ConflictType::CircularDependency;
        let _contradictory_conditions = ConflictType::ContradictoryConditions;
        let _scope_overlap = ConflictType::ScopeOverlap;
    }

    #[test]
    fn test_override_resolution() {
        let mut engine = MappingOverrideEngine::new();
        
        let override_rule = MappingOverride {
            id: Uuid::new_v4(),
            name: "Asset ID Override".to_string(),
            description: "Maps Asset ID to uuid field".to_string(),
            rule_type: OverrideType::ExactMatch,
            pattern: OverridePattern {
                pattern: "Asset ID".to_string(),
                case_sensitive: false,
                whole_word: true,
                regex_flags: None,
                fuzzy_threshold: None,
                position_constraints: None,
            },
            target_field: "uuid".to_string(),
            priority: 100,
            conditions: Vec::new(),
            scope: OverrideScope::Global,
            created_by: "test_user".to_string(),
            created_at: Utc::now(),
            modified_at: Utc::now(),
            active: true,
            version: 1,
            tags: Vec::new(),
        };
        
        engine.add_override(override_rule).unwrap();
        
        let context = OverrideContext::new("inventory".to_string());
        let result = engine.resolve_mapping("Asset ID", "inventory", &context).unwrap();
        
        assert!(result.override_applied);
        assert_eq!(result.target_field, Some("uuid".to_string()));
        assert_eq!(result.confidence, 1.0);
        assert!(result.conflicts.is_empty());
    }

    #[test]
    fn test_override_cache() {
        let mut engine = MappingOverrideEngine::new();
        
        let override_rule = MappingOverride {
            id: Uuid::new_v4(),
            name: "Cached Override".to_string(),
            description: "Test caching".to_string(),
            rule_type: OverrideType::ExactMatch,
            pattern: OverridePattern {
                pattern: "Test Column".to_string(),
                case_sensitive: false,
                whole_word: true,
                regex_flags: None,
                fuzzy_threshold: None,
                position_constraints: None,
            },
            target_field: "test_field".to_string(),
            priority: 100,
            conditions: Vec::new(),
            scope: OverrideScope::Global,
            created_by: "test_user".to_string(),
            created_at: Utc::now(),
            modified_at: Utc::now(),
            active: true,
            version: 1,
            tags: Vec::new(),
        };
        
        engine.add_override(override_rule).unwrap();
        
        let context = OverrideContext::new("test".to_string());
        
        // First call - should not be from cache
        let result1 = engine.resolve_mapping("Test Column", "test", &context).unwrap();
        assert!(!result1.from_cache);
        
        // Second call - should be from cache
        let result2 = engine.resolve_mapping("Test Column", "test", &context).unwrap();
        assert!(result2.from_cache);
        
        assert_eq!(result1.target_field, result2.target_field);
    }

    #[test]
    fn test_report_config_default() {
        let config = ReportConfig::default();
        assert_eq!(config.default_format, ReportFormat::Html);
        assert!(config.include_visualizations);
        assert_eq!(config.max_generation_time_seconds, 30);
        assert!(config.enable_caching);
        assert_eq!(config.cache_expiration_minutes, 60);
    }

    #[test]
    fn test_report_type_serialization() {
        let report_type = ReportType::Summary;
        let serialized = serde_json::to_string(&report_type).unwrap();
        assert!(serialized.contains("Summary"));
        
        let custom_type = ReportType::Custom("MyCustomReport".to_string());
        let serialized = serde_json::to_string(&custom_type).unwrap();
        assert!(serialized.contains("MyCustomReport"));
    }

    #[test]
    fn test_report_format_serialization() {
        let formats = vec![
            ReportFormat::Html,
            ReportFormat::Json,
            ReportFormat::Csv,
            ReportFormat::Markdown,
            ReportFormat::Pdf,
        ];
        
        for format in formats {
            let serialized = serde_json::to_string(&format).unwrap();
            let deserialized: ReportFormat = serde_json::from_str(&serialized).unwrap();
            assert_eq!(format, deserialized);
        }
    }

    #[test]
    fn test_document_validator_creation() {
        let _validator = DocumentValidator::new();
        // Test that validator can be created successfully
    }

    #[test]
    fn test_quality_metrics_creation() {
        let _high_quality = crate::validation::reports::QualityMetrics {
            completeness_score: 0.9,
            accuracy_score: 0.9,
            consistency_score: 0.9,
            overall_quality_score: 0.9,
            risk_level: RiskLevel::Low,
            quality_grade: QualityGrade::A,
            compliance_percentage: 90.0,
            critical_issues: 0,
            warnings: 0,
        };

        let _low_quality = crate::validation::reports::QualityMetrics {
            completeness_score: 0.5,
            accuracy_score: 0.5,
            consistency_score: 0.5,
            overall_quality_score: 0.5,
            risk_level: RiskLevel::High,
            quality_grade: QualityGrade::F,
            compliance_percentage: 50.0,
            critical_issues: 5,
            warnings: 10,
        };
    }
}
