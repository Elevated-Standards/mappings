//! Type definitions and data structures for POA&M Excel parsing
//! Modified: 2025-01-22

use crate::excel::core::ExcelParser;
use crate::excel::types::*;
use serde::{Serialize, Deserialize};
use serde_json::Value;
use chrono::{DateTime, Utc};
use std::collections::HashMap;

/// POA&M-specific Excel parser for FedRAMP POA&M templates
#[derive(Debug, Clone)]
pub struct PoamParser {
    /// Base Excel parser for core functionality
    pub base_parser: ExcelParser,
    /// Template detector for identifying POA&M template versions
    pub template_detector: PoamTemplateDetector,
    /// Field mapper for POA&M-specific column mapping
    pub field_mapper: PoamFieldMapper,
    /// Validator for POA&M business rules
    pub validator: PoamValidator,
    /// Data enricher for calculated fields
    pub enricher: PoamDataEnricher,
}

/// POA&M template detector and version identifier
#[derive(Debug, Clone)]
pub struct PoamTemplateDetector {
    /// Known template signatures
    pub template_signatures: Vec<TemplateSignature>,
}

/// Template signature for identifying POA&M templates
#[derive(Debug, Clone)]
pub struct TemplateSignature {
    /// Template name and version
    pub name: String,
    /// Required column patterns
    pub required_columns: Vec<String>,
    /// Optional column patterns
    pub optional_columns: Vec<String>,
    /// Worksheet names that must be present
    pub required_worksheets: Vec<String>,
    /// Template version
    pub version: String,
}

/// POA&M field mapper for column-to-field mapping
#[derive(Debug, Clone)]
pub struct PoamFieldMapper {
    /// Column mapping configuration
    pub mapping_config: PoamMappingConfig,
}

/// POA&M mapping configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PoamMappingConfig {
    /// Required field mappings
    pub required_columns: HashMap<String, PoamFieldMapping>,
    /// Optional field mappings
    pub optional_columns: HashMap<String, PoamFieldMapping>,
    /// Validation rules
    pub validation_rules: HashMap<String, ValidationRule>,
}

/// Individual field mapping configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PoamFieldMapping {
    /// Possible column names for this field
    pub column_names: Vec<String>,
    /// Target OSCAL field path
    pub oscal_field: String,
    /// Whether this field is required
    pub required: bool,
    /// Validation rule name
    pub validation: Option<String>,
    /// Data transformation rule
    pub transformation: Option<String>,
}

/// Validation rule for POA&M fields
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationRule {
    /// Rule type
    pub rule_type: String,
    /// Allowed values (for enumeration rules)
    pub allowed_values: Option<Vec<String>>,
    /// Pattern for regex validation
    pub pattern: Option<String>,
    /// Custom validation logic identifier
    pub custom_validator: Option<String>,
}

/// POA&M validator for business rules
#[derive(Debug, Clone)]
pub struct PoamValidator {
    /// Validation rules
    pub validation_rules: HashMap<String, ValidationRule>,
}

/// POA&M data enricher for calculated fields
#[derive(Debug, Clone)]
pub struct PoamDataEnricher {
    /// Risk calculation rules
    pub risk_calculator: RiskCalculator,
}

/// Risk calculation engine
#[derive(Debug, Clone)]
pub struct RiskCalculator {
    /// Risk matrix configuration
    pub risk_matrix: RiskMatrix,
}

/// Risk assessment matrix
#[derive(Debug, Clone)]
pub struct RiskMatrix {
    /// Severity to impact mapping
    pub severity_impact_map: HashMap<PoamSeverity, u8>,
    /// Likelihood to probability mapping
    pub likelihood_probability_map: HashMap<PoamLikelihood, f64>,
    /// Risk calculation matrix
    pub risk_matrix: Vec<Vec<RiskRating>>,
}

/// Represents a single POA&M item with all required and optional fields
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PoamItem {
    /// Unique identifier for the POA&M item
    pub unique_id: String,
    /// Control identifier (e.g., AC-1, SC-7)
    pub control_id: Option<String>,
    /// CCI (Control Correlation Identifier)
    pub cci: Option<String>,
    /// System name or identifier
    pub system_name: Option<String>,
    /// Vulnerability ID from scanner or manual assessment
    pub vulnerability_id: Option<String>,
    /// Description of the weakness or vulnerability
    pub weakness_description: String,
    /// Source that identified the weakness
    pub source_identifier: Option<String>,
    /// Asset or component identifier
    pub asset_identifier: Option<String>,
    /// List of affected security controls
    pub security_controls: Vec<String>,
    /// Office or organization responsible
    pub office_organization: Option<String>,
    /// Human-readable names of security controls
    pub security_control_names: Vec<String>,
    /// Implementation guidance or recommendations
    pub implementation_guidance: Option<String>,
    /// Severity level of the finding
    pub severity: PoamSeverity,
    /// Likelihood of exploitation
    pub likelihood: Option<PoamLikelihood>,
    /// Impact if exploited
    pub impact: Option<PoamImpact>,
    /// Overall risk rating
    pub risk_rating: Option<RiskRating>,
    /// Current status of remediation
    pub status: PoamStatus,
    /// Scheduled completion date
    pub scheduled_completion_date: Option<DateTime<Utc>>,
    /// Actual completion date
    pub actual_completion_date: Option<DateTime<Utc>>,
    /// List of milestones for remediation
    pub milestones: Vec<PoamMilestone>,
    /// Required resources for remediation
    pub resources: Vec<PoamResource>,
    /// Point of contact for this item
    pub point_of_contact: Option<String>,
    /// Detailed remediation plan
    pub remediation_plan: Option<String>,
    /// List of affected assets
    pub affected_assets: Vec<String>,
    /// Additional comments or notes
    pub comments: Option<String>,
    /// Vendor information if applicable
    pub vendor_information: Option<String>,
    /// Estimated cost for remediation
    pub cost_estimate: Option<f64>,
    /// Date the weakness was first detected
    pub detection_date: Option<DateTime<Utc>>,
    /// Last updated timestamp
    pub last_updated: DateTime<Utc>,
}

/// Severity levels for POA&M items
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum PoamSeverity {
    /// Critical severity - immediate action required
    Critical,
    /// High severity - urgent action required
    High,
    /// Medium severity - timely action required
    Medium,
    /// Low severity - routine action acceptable
    Low,
    /// Informational - no action required
    Info,
}

impl PartialOrd for PoamSeverity {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for PoamSeverity {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        use std::cmp::Ordering;
        match (self, other) {
            (PoamSeverity::Critical, PoamSeverity::Critical) => Ordering::Equal,
            (PoamSeverity::Critical, _) => Ordering::Greater,
            (_, PoamSeverity::Critical) => Ordering::Less,

            (PoamSeverity::High, PoamSeverity::High) => Ordering::Equal,
            (PoamSeverity::High, _) => Ordering::Greater,
            (_, PoamSeverity::High) => Ordering::Less,

            (PoamSeverity::Medium, PoamSeverity::Medium) => Ordering::Equal,
            (PoamSeverity::Medium, PoamSeverity::Low | PoamSeverity::Info) => Ordering::Greater,
            (PoamSeverity::Low | PoamSeverity::Info, PoamSeverity::Medium) => Ordering::Less,

            (PoamSeverity::Low, PoamSeverity::Low) => Ordering::Equal,
            (PoamSeverity::Low, PoamSeverity::Info) => Ordering::Greater,
            (PoamSeverity::Info, PoamSeverity::Low) => Ordering::Less,

            (PoamSeverity::Info, PoamSeverity::Info) => Ordering::Equal,
        }
    }
}

/// Likelihood of exploitation for POA&M items
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum PoamLikelihood {
    /// Very high likelihood of exploitation
    VeryHigh,
    /// High likelihood of exploitation
    High,
    /// Medium likelihood of exploitation
    Medium,
    /// Low likelihood of exploitation
    Low,
    /// Very low likelihood of exploitation
    VeryLow,
}

impl PartialOrd for PoamLikelihood {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for PoamLikelihood {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        use std::cmp::Ordering;
        match (self, other) {
            (PoamLikelihood::VeryHigh, PoamLikelihood::VeryHigh) => Ordering::Equal,
            (PoamLikelihood::VeryHigh, _) => Ordering::Greater,
            (_, PoamLikelihood::VeryHigh) => Ordering::Less,

            (PoamLikelihood::High, PoamLikelihood::High) => Ordering::Equal,
            (PoamLikelihood::High, PoamLikelihood::Medium | PoamLikelihood::Low | PoamLikelihood::VeryLow) => Ordering::Greater,
            (PoamLikelihood::Medium | PoamLikelihood::Low | PoamLikelihood::VeryLow, PoamLikelihood::High) => Ordering::Less,

            (PoamLikelihood::Medium, PoamLikelihood::Medium) => Ordering::Equal,
            (PoamLikelihood::Medium, PoamLikelihood::Low | PoamLikelihood::VeryLow) => Ordering::Greater,
            (PoamLikelihood::Low | PoamLikelihood::VeryLow, PoamLikelihood::Medium) => Ordering::Less,

            (PoamLikelihood::Low, PoamLikelihood::Low) => Ordering::Equal,
            (PoamLikelihood::Low, PoamLikelihood::VeryLow) => Ordering::Greater,
            (PoamLikelihood::VeryLow, PoamLikelihood::Low) => Ordering::Less,

            (PoamLikelihood::VeryLow, PoamLikelihood::VeryLow) => Ordering::Equal,
        }
    }
}

/// Impact levels for POA&M items
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum PoamImpact {
    /// Very high impact if exploited
    VeryHigh,
    /// High impact if exploited
    High,
    /// Medium impact if exploited
    Medium,
    /// Low impact if exploited
    Low,
    /// Very low impact if exploited
    VeryLow,
}

/// Status of POA&M remediation
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum PoamStatus {
    /// Open - not yet started
    Open,
    /// In progress - actively being worked
    InProgress,
    /// Completed - remediation finished
    Completed,
    /// Risk accepted - no remediation planned
    RiskAccepted,
    /// False positive - not a real issue
    FalsePositive,
    /// Deferred - postponed to future
    Deferred,
}

/// Overall risk rating calculated from severity, likelihood, and impact
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum RiskRating {
    /// Very high risk
    VeryHigh,
    /// High risk
    High,
    /// Medium risk
    Medium,
    /// Low risk
    Low,
    /// Very low risk
    VeryLow,
}

impl PartialOrd for RiskRating {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for RiskRating {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        use std::cmp::Ordering;
        match (self, other) {
            (RiskRating::VeryHigh, RiskRating::VeryHigh) => Ordering::Equal,
            (RiskRating::VeryHigh, _) => Ordering::Greater,
            (_, RiskRating::VeryHigh) => Ordering::Less,

            (RiskRating::High, RiskRating::High) => Ordering::Equal,
            (RiskRating::High, _) => Ordering::Greater,
            (_, RiskRating::High) => Ordering::Less,

            (RiskRating::Medium, RiskRating::Medium) => Ordering::Equal,
            (RiskRating::Medium, RiskRating::Low | RiskRating::VeryLow) => Ordering::Greater,
            (RiskRating::Low | RiskRating::VeryLow, RiskRating::Medium) => Ordering::Less,

            (RiskRating::Low, RiskRating::Low) => Ordering::Equal,
            (RiskRating::Low, RiskRating::VeryLow) => Ordering::Greater,
            (RiskRating::VeryLow, RiskRating::Low) => Ordering::Less,

            (RiskRating::VeryLow, RiskRating::VeryLow) => Ordering::Equal,
        }
    }
}

/// Status of a POA&M milestone
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum MilestoneStatus {
    /// Not started
    NotStarted,
    /// In progress
    InProgress,
    /// Completed
    Completed,
    /// Delayed
    Delayed,
    /// Cancelled
    Cancelled,
}

/// Types of resources for POA&M remediation
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum ResourceType {
    /// Personnel resources
    Personnel,
    /// Hardware resources
    Hardware,
    /// Software resources
    Software,
    /// Training resources
    Training,
    /// Consulting services
    Consulting,
    /// Other resources
    Other,
}

/// Milestone for POA&M remediation tracking
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PoamMilestone {
    /// Milestone unique identifier
    pub id: String,
    /// Description of the milestone
    pub description: String,
    /// Scheduled completion date
    pub scheduled_date: Option<DateTime<Utc>>,
    /// Actual completion date
    pub actual_date: Option<DateTime<Utc>>,
    /// Current status of the milestone
    pub status: MilestoneStatus,
    /// Percentage complete (0-100)
    pub percent_complete: Option<u8>,
    /// Comments or notes about the milestone
    pub comments: Option<String>,
}

/// Resource required for POA&M remediation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PoamResource {
    /// Resource type
    pub resource_type: ResourceType,
    /// Description of the resource
    pub description: String,
    /// Estimated cost
    pub estimated_cost: Option<f64>,
    /// Actual cost
    pub actual_cost: Option<f64>,
    /// Required quantity
    pub quantity: Option<f64>,
    /// Unit of measurement
    pub unit: Option<String>,
}

/// Result of parsing a POA&M Excel file
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PoamParseResult {
    /// Parsed POA&M items
    pub items: Vec<PoamItem>,
    /// Template information detected
    pub template_info: Option<TemplateInfo>,
    /// Parsing statistics
    pub statistics: PoamParsingStatistics,
    /// Validation results
    pub validation_results: Vec<PoamValidationResult>,
    /// Overall quality score
    pub quality_score: f64,
}

/// Information about the detected POA&M template
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemplateInfo {
    /// Template name
    pub name: String,
    /// Template version
    pub version: String,
    /// Confidence score for template detection
    pub confidence: f64,
    /// Detected column mappings
    pub column_mappings: HashMap<String, String>,
}

/// Statistics about POA&M parsing
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PoamParsingStatistics {
    /// Total rows processed
    pub total_rows: usize,
    /// Successfully parsed items
    pub parsed_items: usize,
    /// Rows with errors
    pub error_rows: usize,
    /// Rows skipped (empty or header)
    pub skipped_rows: usize,
    /// Average confidence score
    pub average_confidence: f64,
}

/// Validation result for a POA&M item
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PoamValidationResult {
    /// Row number where validation failed
    pub row_number: usize,
    /// Field that failed validation
    pub field_name: String,
    /// Validation error message
    pub error_message: String,
    /// Severity of the validation error
    pub severity: ValidationSeverity,
    /// Suggested fix
    pub suggestion: Option<String>,
}

/// Internal result structure for worksheet parsing
pub(crate) struct PoamWorksheetParseResult {
    /// Parsed items
    pub items: Vec<PoamItem>,
    /// Validation results
    pub validation_results: Vec<PoamValidationResult>,
    /// Total rows processed
    pub total_rows: usize,
    /// Rows with errors
    pub error_rows: usize,
    /// Rows skipped
    pub skipped_rows: usize,
}

impl Default for PoamMappingConfig {
    fn default() -> Self {
        Self {
            required_columns: HashMap::new(),
            optional_columns: HashMap::new(),
            validation_rules: HashMap::new(),
        }
    }
}

impl Default for RiskMatrix {
    fn default() -> Self {
        let mut severity_impact_map = HashMap::new();
        severity_impact_map.insert(PoamSeverity::Critical, 5);
        severity_impact_map.insert(PoamSeverity::High, 4);
        severity_impact_map.insert(PoamSeverity::Medium, 3);
        severity_impact_map.insert(PoamSeverity::Low, 2);
        severity_impact_map.insert(PoamSeverity::Info, 1);

        let mut likelihood_probability_map = HashMap::new();
        likelihood_probability_map.insert(PoamLikelihood::VeryHigh, 0.9);
        likelihood_probability_map.insert(PoamLikelihood::High, 0.7);
        likelihood_probability_map.insert(PoamLikelihood::Medium, 0.5);
        likelihood_probability_map.insert(PoamLikelihood::Low, 0.3);
        likelihood_probability_map.insert(PoamLikelihood::VeryLow, 0.1);

        // Create a 5x5 risk matrix
        let risk_matrix = vec![
            vec![RiskRating::VeryLow, RiskRating::Low, RiskRating::Medium, RiskRating::High, RiskRating::VeryHigh],
            vec![RiskRating::Low, RiskRating::Low, RiskRating::Medium, RiskRating::High, RiskRating::VeryHigh],
            vec![RiskRating::Medium, RiskRating::Medium, RiskRating::Medium, RiskRating::High, RiskRating::VeryHigh],
            vec![RiskRating::High, RiskRating::High, RiskRating::High, RiskRating::High, RiskRating::VeryHigh],
            vec![RiskRating::VeryHigh, RiskRating::VeryHigh, RiskRating::VeryHigh, RiskRating::VeryHigh, RiskRating::VeryHigh],
        ];

        Self {
            severity_impact_map,
            likelihood_probability_map,
            risk_matrix,
        }
    }
}
