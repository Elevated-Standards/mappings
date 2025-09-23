//! Quality assessment framework for POA&M documents
//! 
//! This module provides comprehensive quality checks for POA&M data including:
//! - Data completeness analysis
//! - Accuracy validation
//! - Consistency checking
//! - Compliance assessment

pub mod poam_quality;
pub mod completeness;
pub mod accuracy;
pub mod consistency;
pub mod compliance;

pub use poam_quality::*;
pub use completeness::*;
pub use accuracy::*;
pub use consistency::*;
pub use compliance::{
    ComplianceAssessor, ComplianceResult, ComplianceCheckResult, ComplianceViolation,
    ComplianceConfig, ComplianceCheckType, ComplianceStatistics,
    FedRampComplianceChecker, OscalComplianceChecker, RegulatoryComplianceChecker, RegulatoryRule
};

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use chrono::{DateTime, Utc};

/// Quality severity levels for findings
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum QualitySeverity {
    /// Critical issues that block processing or compliance
    Critical,
    /// High impact quality issues
    High,
    /// Moderate quality concerns
    Medium,
    /// Minor quality issues
    Low,
    /// Informational findings
    Info,
}

/// Quality assessment categories
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum QualityCategory {
    /// Data completeness issues
    Completeness,
    /// Data accuracy problems
    Accuracy,
    /// Data consistency violations
    Consistency,
    /// Compliance requirement failures
    Compliance,
    /// Format and structure issues
    Format,
    /// Business logic violations
    BusinessLogic,
    /// Reference integrity problems
    ReferenceIntegrity,
}

/// Quality finding with detailed information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QualityFinding {
    /// Unique identifier for the finding
    pub id: String,
    /// Severity level of the finding
    pub severity: QualitySeverity,
    /// Category of the quality issue
    pub category: QualityCategory,
    /// Human-readable description
    pub description: String,
    /// Items affected by this finding
    pub affected_items: Vec<String>,
    /// Assessment of the impact
    pub impact_assessment: String,
    /// Recommended action to resolve
    pub recommendation: String,
    /// Location or context of the finding
    pub location: Option<String>,
    /// Additional metadata
    pub metadata: HashMap<String, serde_json::Value>,
}

/// Quality improvement recommendation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QualityRecommendation {
    /// Unique identifier for the recommendation
    pub id: String,
    /// Priority level for implementation
    pub priority: QualitySeverity,
    /// Title of the recommendation
    pub title: String,
    /// Detailed description
    pub description: String,
    /// Expected impact of implementing
    pub expected_impact: String,
    /// Implementation effort estimate
    pub effort_estimate: String,
    /// Related quality findings
    pub related_findings: Vec<String>,
}

/// Comprehensive quality metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QualityMetrics {
    /// Total number of POA&M items assessed
    pub total_items: usize,
    /// Number of complete items
    pub complete_items: usize,
    /// Number of incomplete items
    pub incomplete_items: usize,
    /// Total error count
    pub error_count: usize,
    /// Total warning count
    pub warning_count: usize,
    /// Number of missing required fields
    pub missing_required_fields: usize,
    /// Number of data quality issues
    pub data_quality_issues: usize,
    /// Field-level completeness rates
    pub field_completeness: HashMap<String, f64>,
    /// Category-specific metrics
    pub category_metrics: HashMap<QualityCategory, CategoryMetrics>,
}

/// Category-specific quality metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CategoryMetrics {
    /// Score for this category (0.0 to 1.0)
    pub score: f64,
    /// Number of findings in this category
    pub finding_count: usize,
    /// Number of items affected
    pub affected_items: usize,
    /// Category-specific details
    pub details: HashMap<String, serde_json::Value>,
}

/// Overall quality assessment result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QualityAssessment {
    /// Unique identifier for this assessment
    pub assessment_id: String,
    /// Timestamp of the assessment
    pub timestamp: DateTime<Utc>,
    /// Overall quality score (0.0 to 1.0)
    pub overall_score: f64,
    /// Completeness score (0.0 to 1.0)
    pub completeness_score: f64,
    /// Accuracy score (0.0 to 1.0)
    pub accuracy_score: f64,
    /// Consistency score (0.0 to 1.0)
    pub consistency_score: f64,
    /// Compliance score (0.0 to 1.0)
    pub compliance_score: f64,
    /// Detailed quality metrics
    pub quality_metrics: QualityMetrics,
    /// All quality findings
    pub findings: Vec<QualityFinding>,
    /// Quality improvement recommendations
    pub recommendations: Vec<QualityRecommendation>,
    /// Assessment configuration used
    pub config_summary: HashMap<String, serde_json::Value>,
}

/// Configuration for quality assessment
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QualityConfig {
    /// Minimum acceptable overall score
    pub min_overall_score: f64,
    /// Minimum acceptable completeness score
    pub min_completeness_score: f64,
    /// Minimum acceptable accuracy score
    pub min_accuracy_score: f64,
    /// Minimum acceptable consistency score
    pub min_consistency_score: f64,
    /// Minimum acceptable compliance score
    pub min_compliance_score: f64,
    /// Weights for different quality dimensions
    pub dimension_weights: DimensionWeights,
    /// Required fields configuration
    pub required_fields: Vec<String>,
    /// Optional but recommended fields
    pub recommended_fields: Vec<String>,
    /// Field-specific validation rules
    pub field_rules: HashMap<String, FieldValidationRule>,
    /// Enable strict validation mode
    pub strict_mode: bool,
    /// Custom quality thresholds
    pub custom_thresholds: HashMap<String, f64>,
}

/// Weights for different quality dimensions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DimensionWeights {
    /// Weight for completeness (default: 0.3)
    pub completeness: f64,
    /// Weight for accuracy (default: 0.3)
    pub accuracy: f64,
    /// Weight for consistency (default: 0.2)
    pub consistency: f64,
    /// Weight for compliance (default: 0.2)
    pub compliance: f64,
}

/// Field-specific validation rule
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FieldValidationRule {
    /// Field is required
    pub required: bool,
    /// Minimum length for text fields
    pub min_length: Option<usize>,
    /// Maximum length for text fields
    pub max_length: Option<usize>,
    /// Pattern validation (regex)
    pub pattern: Option<String>,
    /// Allowed values (enumeration)
    pub allowed_values: Option<Vec<String>>,
    /// Custom validation function name
    pub custom_validator: Option<String>,
}

impl Default for QualityConfig {
    fn default() -> Self {
        Self {
            min_overall_score: 0.7,
            min_completeness_score: 0.8,
            min_accuracy_score: 0.9,
            min_consistency_score: 0.8,
            min_compliance_score: 0.9,
            dimension_weights: DimensionWeights::default(),
            required_fields: vec![
                "uuid".to_string(),
                "title".to_string(),
                "description".to_string(),
                "status".to_string(),
                "scheduled_completion_date".to_string(),
            ],
            recommended_fields: vec![
                "responsible_entity".to_string(),
                "resources_required".to_string(),
                "milestones".to_string(),
                "risk_assessment".to_string(),
            ],
            field_rules: HashMap::new(),
            strict_mode: false,
            custom_thresholds: HashMap::new(),
        }
    }
}

impl Default for DimensionWeights {
    fn default() -> Self {
        Self {
            completeness: 0.3,
            accuracy: 0.3,
            consistency: 0.2,
            compliance: 0.2,
        }
    }
}

impl QualitySeverity {
    /// Get numeric weight for severity (higher = more severe)
    pub fn weight(&self) -> u8 {
        match self {
            QualitySeverity::Critical => 5,
            QualitySeverity::High => 4,
            QualitySeverity::Medium => 3,
            QualitySeverity::Low => 2,
            QualitySeverity::Info => 1,
        }
    }
}

impl QualityAssessment {
    /// Check if the assessment passes minimum quality thresholds
    pub fn passes_quality_gates(&self, config: &QualityConfig) -> bool {
        self.overall_score >= config.min_overall_score
            && self.completeness_score >= config.min_completeness_score
            && self.accuracy_score >= config.min_accuracy_score
            && self.consistency_score >= config.min_consistency_score
            && self.compliance_score >= config.min_compliance_score
    }

    /// Get critical findings that must be addressed
    pub fn critical_findings(&self) -> Vec<&QualityFinding> {
        self.findings
            .iter()
            .filter(|f| f.severity == QualitySeverity::Critical)
            .collect()
    }

    /// Get high priority recommendations
    pub fn high_priority_recommendations(&self) -> Vec<&QualityRecommendation> {
        self.recommendations
            .iter()
            .filter(|r| matches!(r.priority, QualitySeverity::Critical | QualitySeverity::High))
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_quality_severity_weight() {
        assert_eq!(QualitySeverity::Critical.weight(), 5);
        assert_eq!(QualitySeverity::High.weight(), 4);
        assert_eq!(QualitySeverity::Medium.weight(), 3);
        assert_eq!(QualitySeverity::Low.weight(), 2);
        assert_eq!(QualitySeverity::Info.weight(), 1);
    }

    #[test]
    fn test_quality_config_default() {
        let config = QualityConfig::default();

        assert_eq!(config.min_overall_score, 0.7);
        assert_eq!(config.min_completeness_score, 0.8);
        assert_eq!(config.min_accuracy_score, 0.9);
        assert_eq!(config.min_consistency_score, 0.8);
        assert_eq!(config.min_compliance_score, 0.9);

        assert!(config.required_fields.contains(&"uuid".to_string()));
        assert!(config.required_fields.contains(&"title".to_string()));
        assert!(config.required_fields.contains(&"description".to_string()));

        assert!(config.recommended_fields.contains(&"responsible_entity".to_string()));
        assert!(config.recommended_fields.contains(&"resources_required".to_string()));
    }

    #[test]
    fn test_dimension_weights_default() {
        let weights = DimensionWeights::default();

        assert_eq!(weights.completeness, 0.3);
        assert_eq!(weights.accuracy, 0.3);
        assert_eq!(weights.consistency, 0.2);
        assert_eq!(weights.compliance, 0.2);

        // Weights should sum to 1.0
        let total = weights.completeness + weights.accuracy + weights.consistency + weights.compliance;
        assert!((total - 1.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_quality_assessment_gates() {
        let config = QualityConfig::default();

        // Assessment that passes all gates
        let passing_assessment = QualityAssessment {
            assessment_id: "test".to_string(),
            timestamp: chrono::Utc::now(),
            overall_score: 0.8,
            completeness_score: 0.9,
            accuracy_score: 0.95,
            consistency_score: 0.85,
            compliance_score: 0.92,
            quality_metrics: QualityMetrics {
                total_items: 10,
                complete_items: 8,
                incomplete_items: 2,
                error_count: 1,
                warning_count: 3,
                missing_required_fields: 2,
                data_quality_issues: 4,
                field_completeness: HashMap::new(),
                category_metrics: HashMap::new(),
            },
            findings: Vec::new(),
            recommendations: Vec::new(),
            config_summary: HashMap::new(),
        };

        assert!(passing_assessment.passes_quality_gates(&config));

        // Assessment that fails overall score
        let failing_assessment = QualityAssessment {
            overall_score: 0.6, // Below 0.7 threshold
            ..passing_assessment.clone()
        };

        assert!(!failing_assessment.passes_quality_gates(&config));
    }

    #[test]
    fn test_quality_finding_filtering() {
        let findings = vec![
            QualityFinding {
                id: "1".to_string(),
                severity: QualitySeverity::Critical,
                category: QualityCategory::Completeness,
                description: "Critical issue".to_string(),
                affected_items: vec!["item1".to_string()],
                impact_assessment: "High impact".to_string(),
                recommendation: "Fix immediately".to_string(),
                location: None,
                metadata: HashMap::new(),
            },
            QualityFinding {
                id: "2".to_string(),
                severity: QualitySeverity::Medium,
                category: QualityCategory::Accuracy,
                description: "Medium issue".to_string(),
                affected_items: vec!["item2".to_string()],
                impact_assessment: "Medium impact".to_string(),
                recommendation: "Fix when possible".to_string(),
                location: None,
                metadata: HashMap::new(),
            },
        ];

        let assessment = QualityAssessment {
            assessment_id: "test".to_string(),
            timestamp: chrono::Utc::now(),
            overall_score: 0.8,
            completeness_score: 0.8,
            accuracy_score: 0.8,
            consistency_score: 0.8,
            compliance_score: 0.8,
            quality_metrics: QualityMetrics {
                total_items: 2,
                complete_items: 1,
                incomplete_items: 1,
                error_count: 1,
                warning_count: 1,
                missing_required_fields: 1,
                data_quality_issues: 2,
                field_completeness: HashMap::new(),
                category_metrics: HashMap::new(),
            },
            findings,
            recommendations: Vec::new(),
            config_summary: HashMap::new(),
        };

        let critical_findings = assessment.critical_findings();
        assert_eq!(critical_findings.len(), 1);
        assert_eq!(critical_findings[0].severity, QualitySeverity::Critical);
    }
}
