// Modified: 2025-09-22

//! POAM-specific Excel parsing and validation
//!
//! This module provides specialized parsing for FedRAMP POA&M (Plan of Action and Milestones)
//! Excel templates, including template detection, field mapping, and business rule validation.

use crate::excel::core::ExcelParser;
use crate::excel::types::*;
use serde::{Serialize, Deserialize};
use serde_json::Value;
use chrono::{DateTime, Utc};
use uuid::Uuid;
use std::collections::HashMap;
use tracing::{debug, info, warn};
use fedramp_core::{Result, Error};

/// POA&M-specific Excel parser for FedRAMP POA&M templates
#[derive(Debug, Clone)]
pub struct PoamParser {
    /// Base Excel parser for core functionality
    base_parser: ExcelParser,
    /// Template detector for identifying POA&M template versions
    template_detector: PoamTemplateDetector,
    /// Field mapper for POA&M-specific column mapping
    field_mapper: PoamFieldMapper,
    /// Validator for POA&M business rules
    validator: PoamValidator,
    /// Data enricher for calculated fields
    enricher: PoamDataEnricher,
}

/// POA&M template detector and version identifier
#[derive(Debug, Clone)]
pub struct PoamTemplateDetector {
    /// Known template signatures
    template_signatures: Vec<TemplateSignature>,
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
    mapping_config: PoamMappingConfig,
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
    validation_rules: HashMap<String, ValidationRule>,
}

/// POA&M data enricher for calculated fields
#[derive(Debug, Clone)]
pub struct PoamDataEnricher {
    /// Risk calculation rules
    risk_calculator: RiskCalculator,
}

/// Risk calculation engine
#[derive(Debug, Clone)]
pub struct RiskCalculator {
    /// Risk matrix configuration
    risk_matrix: RiskMatrix,
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

impl PoamParser {
    /// Create a new POA&M parser with default configuration
    pub fn new() -> Self {
        Self {
            base_parser: ExcelParser::new(),
            template_detector: PoamTemplateDetector::new(),
            field_mapper: PoamFieldMapper::new(),
            validator: PoamValidator::new(),
            enricher: PoamDataEnricher::new(),
        }
    }

    /// Create a new POA&M parser with custom Excel parser configuration
    pub fn with_excel_config(excel_parser: ExcelParser) -> Self {
        Self {
            base_parser: excel_parser,
            template_detector: PoamTemplateDetector::new(),
            field_mapper: PoamFieldMapper::new(),
            validator: PoamValidator::new(),
            enricher: PoamDataEnricher::new(),
        }
    }

    /// Parse a POA&M Excel file from path
    ///
    /// # Arguments
    ///
    /// * `path` - Path to the POA&M Excel file
    ///
    /// # Returns
    ///
    /// Returns `Result<PoamParseResult>` with parsed POA&M data
    ///
    /// # Errors
    ///
    /// Returns error if file cannot be read, parsed, or doesn't contain valid POA&M data
    pub async fn parse_poam_file(&self, path: &std::path::Path) -> Result<PoamParseResult> {
        info!("Parsing POA&M file: {}", path.display());

        // First parse as regular Excel file
        let excel_result = self.base_parser.parse_excel_file(path).await?;

        // Extract worksheets from the result
        let worksheets = excel_result.content
            .get("worksheets")
            .and_then(|v| v.as_array())
            .ok_or_else(|| Error::document_parsing("No worksheets found in Excel file".to_string()))?;

        self.parse_poam_worksheets(worksheets).await
    }

    /// Parse POA&M data from Excel worksheets
    async fn parse_poam_worksheets(&self, worksheets: &[Value]) -> Result<PoamParseResult> {
        let mut all_items = Vec::new();
        let mut all_validation_results = Vec::new();
        let mut template_info = None;
        let mut total_rows = 0;
        let mut error_rows = 0;
        let mut skipped_rows = 0;

        for worksheet in worksheets {
            let worksheet_name = worksheet
                .get("name")
                .and_then(|v| v.as_str())
                .unwrap_or("Unknown");

            debug!("Processing worksheet: {}", worksheet_name);

            // Detect if this worksheet contains POA&M data
            if let Some(detected_template) = self.template_detector.detect_template(worksheet) {
                info!("Detected POA&M template: {} v{}", detected_template.name, detected_template.version);
                template_info = Some(detected_template);

                // Parse the worksheet as POA&M data
                match self.parse_poam_worksheet(worksheet).await {
                    Ok(result) => {
                        total_rows += result.total_rows;
                        error_rows += result.error_rows;
                        skipped_rows += result.skipped_rows;
                        all_items.extend(result.items);
                        all_validation_results.extend(result.validation_results);
                    }
                    Err(e) => {
                        warn!("Failed to parse POA&M worksheet '{}': {}", worksheet_name, e);
                        error_rows += 1;
                    }
                }
            } else {
                debug!("Worksheet '{}' does not appear to contain POA&M data", worksheet_name);
                skipped_rows += 1;
            }
        }

        let parsed_items = all_items.len();
        let average_confidence = if parsed_items > 0 {
            // Calculate average confidence from validation results
            let total_confidence: f64 = all_validation_results
                .iter()
                .map(|_| 0.8) // Placeholder confidence calculation
                .sum();
            total_confidence / parsed_items as f64
        } else {
            0.0
        };

        let quality_score = if total_rows > 0 {
            (parsed_items as f64 / total_rows as f64) * average_confidence
        } else {
            0.0
        };

        Ok(PoamParseResult {
            items: all_items,
            template_info,
            statistics: PoamParsingStatistics {
                total_rows,
                parsed_items,
                error_rows,
                skipped_rows,
                average_confidence,
            },
            validation_results: all_validation_results,
            quality_score,
        })
    }

    /// Parse a single worksheet as POA&M data
    async fn parse_poam_worksheet(&self, worksheet: &Value) -> Result<PoamWorksheetParseResult> {
        let data = worksheet
            .get("data")
            .and_then(|v| v.as_array())
            .ok_or_else(|| Error::document_parsing("Worksheet data is not an array".to_string()))?;

        let headers = worksheet
            .get("headers")
            .and_then(|v| v.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|v| v.as_str())
                    .map(|s| s.to_string())
                    .collect::<Vec<_>>()
            });

        let mut items = Vec::new();
        let mut validation_results = Vec::new();
        let mut error_rows = 0;
        let mut skipped_rows = 0;

        // Skip header row if headers were detected
        let start_row = if headers.is_some() { 1 } else { 0 };

        for (row_index, row_data) in data.iter().enumerate().skip(start_row) {
            let row_array = match row_data.as_array() {
                Some(arr) => arr,
                None => {
                    skipped_rows += 1;
                    continue;
                }
            };

            // Check if row is empty
            if row_array.iter().all(|v| v.is_null() || (v.is_string() && v.as_str().unwrap_or("").trim().is_empty())) {
                skipped_rows += 1;
                continue;
            }

            match self.parse_poam_row(row_array, row_index, &headers).await {
                Ok(item) => {
                    items.push(item);
                }
                Err(e) => {
                    error_rows += 1;
                    validation_results.push(PoamValidationResult {
                        row_number: row_index + 1,
                        field_name: "row".to_string(),
                        error_message: e.to_string(),
                        severity: ValidationSeverity::Error,
                        suggestion: Some("Check row data format and required fields".to_string()),
                    });
                }
            }
        }

        Ok(PoamWorksheetParseResult {
            items,
            validation_results,
            total_rows: data.len(),
            error_rows,
            skipped_rows,
        })
    }

    /// Parse a single row of POA&M data
    async fn parse_poam_row(
        &self,
        row_data: &[Value],
        row_index: usize,
        headers: &Option<Vec<String>>,
    ) -> Result<PoamItem> {
        // This is a simplified implementation - in practice, this would use
        // the field mapper to map columns to POA&M fields based on the detected template

        // For now, create a basic POA&M item with placeholder data
        let unique_id = format!("POAM-{:06}", row_index + 1);

        Ok(PoamItem {
            unique_id,
            control_id: None,
            cci: None,
            system_name: None,
            vulnerability_id: None,
            weakness_description: "Placeholder description".to_string(),
            source_identifier: None,
            asset_identifier: None,
            security_controls: Vec::new(),
            office_organization: None,
            security_control_names: Vec::new(),
            implementation_guidance: None,
            severity: PoamSeverity::Medium,
            likelihood: None,
            impact: None,
            risk_rating: None,
            status: PoamStatus::Open,
            scheduled_completion_date: None,
            actual_completion_date: None,
            milestones: Vec::new(),
            resources: Vec::new(),
            point_of_contact: None,
            remediation_plan: None,
            affected_assets: Vec::new(),
            comments: None,
            vendor_information: None,
            cost_estimate: None,
            detection_date: None,
            last_updated: Utc::now(),
        })
    }
}

/// Internal result structure for worksheet parsing
struct PoamWorksheetParseResult {
    /// Parsed items
    items: Vec<PoamItem>,
    /// Validation results
    validation_results: Vec<PoamValidationResult>,
    /// Total rows processed
    total_rows: usize,
    /// Rows with errors
    error_rows: usize,
    /// Rows skipped
    skipped_rows: usize,
}

impl PoamTemplateDetector {
    /// Create a new template detector with default signatures
    pub fn new() -> Self {
        let template_signatures = vec![
            TemplateSignature {
                name: "FedRAMP POA&M Template".to_string(),
                version: "3.0".to_string(),
                required_columns: vec![
                    "Unique ID".to_string(),
                    "Control ID".to_string(),
                    "Weakness Description".to_string(),
                    "Severity".to_string(),
                    "Status".to_string(),
                ],
                optional_columns: vec![
                    "CCI".to_string(),
                    "System Name".to_string(),
                    "Vulnerability ID".to_string(),
                    "Source".to_string(),
                    "Asset ID".to_string(),
                ],
                required_worksheets: vec!["POA&M".to_string()],
            },
            TemplateSignature {
                name: "Legacy FedRAMP POA&M".to_string(),
                version: "2.0".to_string(),
                required_columns: vec![
                    "POA&M ID".to_string(),
                    "Control".to_string(),
                    "Description".to_string(),
                    "Risk Level".to_string(),
                ],
                optional_columns: vec![
                    "Remediation Plan".to_string(),
                    "Completion Date".to_string(),
                ],
                required_worksheets: vec!["POAM".to_string(), "POA&M Items".to_string()],
            },
        ];

        Self { template_signatures }
    }

    /// Detect POA&M template from worksheet data
    pub fn detect_template(&self, worksheet: &Value) -> Option<TemplateInfo> {
        let headers = worksheet
            .get("headers")
            .and_then(|v| v.as_array())?
            .iter()
            .filter_map(|v| v.as_str())
            .map(|s| s.to_string())
            .collect::<Vec<_>>();

        let worksheet_name = worksheet
            .get("name")
            .and_then(|v| v.as_str())
            .unwrap_or("");

        let mut best_match: Option<(f64, &TemplateSignature)> = None;

        for signature in &self.template_signatures {
            let confidence = self.calculate_template_confidence(signature, &headers, worksheet_name);

            if confidence > 0.5 {
                if let Some((best_confidence, _)) = best_match {
                    if confidence > best_confidence {
                        best_match = Some((confidence, signature));
                    }
                } else {
                    best_match = Some((confidence, signature));
                }
            }
        }

        if let Some((confidence, signature)) = best_match {
            let column_mappings = self.create_column_mappings(signature, &headers);

            Some(TemplateInfo {
                name: signature.name.clone(),
                version: signature.version.clone(),
                confidence,
                column_mappings,
            })
        } else {
            None
        }
    }

    /// Calculate confidence score for template match
    fn calculate_template_confidence(
        &self,
        signature: &TemplateSignature,
        headers: &[String],
        worksheet_name: &str,
    ) -> f64 {
        let mut score = 0.0;
        let mut total_weight = 0.0;

        // Check worksheet name match
        let worksheet_weight = 0.3;
        total_weight += worksheet_weight;
        if signature.required_worksheets.iter().any(|name| {
            worksheet_name.to_lowercase().contains(&name.to_lowercase())
        }) {
            score += worksheet_weight;
        }

        // Check required columns
        let required_weight = 0.5;
        total_weight += required_weight;
        let required_matches = signature.required_columns.iter()
            .filter(|req_col| {
                headers.iter().any(|header| {
                    self.fuzzy_match(header, req_col)
                })
            })
            .count();

        if !signature.required_columns.is_empty() {
            score += required_weight * (required_matches as f64 / signature.required_columns.len() as f64);
        }

        // Check optional columns (bonus points)
        let optional_weight = 0.2;
        total_weight += optional_weight;
        let optional_matches = signature.optional_columns.iter()
            .filter(|opt_col| {
                headers.iter().any(|header| {
                    self.fuzzy_match(header, opt_col)
                })
            })
            .count();

        if !signature.optional_columns.is_empty() {
            score += optional_weight * (optional_matches as f64 / signature.optional_columns.len() as f64);
        }

        if total_weight > 0.0 {
            score / total_weight
        } else {
            0.0
        }
    }

    /// Perform fuzzy matching between header and expected column name
    fn fuzzy_match(&self, header: &str, expected: &str) -> bool {
        let header_clean = header.to_lowercase().replace(&[' ', '_', '-', '.'], "");
        let expected_clean = expected.to_lowercase().replace(&[' ', '_', '-', '.'], "");

        // Exact match
        if header_clean == expected_clean {
            return true;
        }

        // Contains match
        if header_clean.contains(&expected_clean) || expected_clean.contains(&header_clean) {
            return true;
        }

        // TODO: Implement more sophisticated fuzzy matching (Levenshtein distance, etc.)
        false
    }

    /// Create column mappings for detected template
    fn create_column_mappings(&self, signature: &TemplateSignature, headers: &[String]) -> HashMap<String, String> {
        let mut mappings = HashMap::new();

        // Map required columns
        for req_col in &signature.required_columns {
            if let Some(header) = headers.iter().find(|h| self.fuzzy_match(h, req_col)) {
                mappings.insert(req_col.clone(), header.clone());
            }
        }

        // Map optional columns
        for opt_col in &signature.optional_columns {
            if let Some(header) = headers.iter().find(|h| self.fuzzy_match(h, opt_col)) {
                mappings.insert(opt_col.clone(), header.clone());
            }
        }

        mappings
    }
}

impl PoamFieldMapper {
    /// Create a new field mapper with default configuration
    pub fn new() -> Self {
        Self {
            mapping_config: PoamMappingConfig::default(),
        }
    }

    /// Map row data to POA&M fields based on column mappings
    pub fn map_row_to_poam(&self, row_data: &[Value], column_mappings: &HashMap<String, String>) -> Result<PoamItem> {
        // This is a simplified implementation
        // In practice, this would use the mapping configuration to transform data

        let unique_id = self.extract_field(row_data, column_mappings, "Unique ID")
            .unwrap_or_else(|| format!("POAM-{}", uuid::Uuid::new_v4()));

        let weakness_description = self.extract_field(row_data, column_mappings, "Weakness Description")
            .unwrap_or_else(|| "No description provided".to_string());

        Ok(PoamItem {
            unique_id,
            control_id: self.extract_field(row_data, column_mappings, "Control ID"),
            cci: self.extract_field(row_data, column_mappings, "CCI"),
            system_name: self.extract_field(row_data, column_mappings, "System Name"),
            vulnerability_id: self.extract_field(row_data, column_mappings, "Vulnerability ID"),
            weakness_description,
            source_identifier: self.extract_field(row_data, column_mappings, "Source"),
            asset_identifier: self.extract_field(row_data, column_mappings, "Asset ID"),
            security_controls: Vec::new(), // TODO: Parse from control_id
            office_organization: self.extract_field(row_data, column_mappings, "Office/Organization"),
            security_control_names: Vec::new(),
            implementation_guidance: self.extract_field(row_data, column_mappings, "Implementation Guidance"),
            severity: self.parse_severity(&self.extract_field(row_data, column_mappings, "Severity")),
            likelihood: None, // TODO: Parse likelihood
            impact: None, // TODO: Parse impact
            risk_rating: None, // TODO: Calculate risk rating
            status: self.parse_status(&self.extract_field(row_data, column_mappings, "Status")),
            scheduled_completion_date: None, // TODO: Parse date
            actual_completion_date: None, // TODO: Parse date
            milestones: Vec::new(),
            resources: Vec::new(),
            point_of_contact: self.extract_field(row_data, column_mappings, "Point of Contact"),
            remediation_plan: self.extract_field(row_data, column_mappings, "Remediation Plan"),
            affected_assets: Vec::new(),
            comments: self.extract_field(row_data, column_mappings, "Comments"),
            vendor_information: self.extract_field(row_data, column_mappings, "Vendor Information"),
            cost_estimate: None, // TODO: Parse cost
            detection_date: None, // TODO: Parse date
            last_updated: Utc::now(),
        })
    }

    /// Extract field value from row data using column mappings
    fn extract_field(&self, row_data: &[Value], column_mappings: &HashMap<String, String>, field_name: &str) -> Option<String> {
        // Find the column index for this field
        if let Some(column_name) = column_mappings.get(field_name) {
            // TODO: Find column index by name and extract value
            // For now, return None as this requires header-to-index mapping
        }
        None
    }

    /// Parse severity from string value
    fn parse_severity(&self, value: &Option<String>) -> PoamSeverity {
        let severity_str = value.as_ref().map(|s| s.to_lowercase()).unwrap_or_default();
        match severity_str.as_str() {
            "critical" => PoamSeverity::Critical,
            "high" => PoamSeverity::High,
            "medium" | "moderate" => PoamSeverity::Medium,
            "low" => PoamSeverity::Low,
            "info" | "informational" => PoamSeverity::Info,
            _ => PoamSeverity::Medium, // Default
        }
    }

    /// Parse status from string value
    fn parse_status(&self, value: &Option<String>) -> PoamStatus {
        let status_str = value.as_ref().map(|s| s.to_lowercase()).unwrap_or_default();
        match status_str.as_str() {
            "open" => PoamStatus::Open,
            "in progress" | "inprogress" | "in-progress" => PoamStatus::InProgress,
            "completed" | "complete" => PoamStatus::Completed,
            "risk accepted" | "riskaccepted" | "accepted" => PoamStatus::RiskAccepted,
            "false positive" | "falsepositive" => PoamStatus::FalsePositive,
            "deferred" => PoamStatus::Deferred,
            _ => PoamStatus::Open, // Default
        }
    }
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

impl PoamValidator {
    /// Create a new POA&M validator
    pub fn new() -> Self {
        Self {
            validation_rules: HashMap::new(),
        }
    }

    /// Validate a POA&M item against business rules
    pub fn validate_poam_item(&self, item: &PoamItem) -> Vec<PoamValidationResult> {
        let mut results = Vec::new();

        // Validate required fields
        if item.unique_id.trim().is_empty() {
            results.push(PoamValidationResult {
                row_number: 0, // TODO: Pass actual row number
                field_name: "unique_id".to_string(),
                error_message: "Unique ID is required".to_string(),
                severity: ValidationSeverity::Error,
                suggestion: Some("Provide a unique identifier for this POA&M item".to_string()),
            });
        }

        if item.weakness_description.trim().is_empty() {
            results.push(PoamValidationResult {
                row_number: 0,
                field_name: "weakness_description".to_string(),
                error_message: "Weakness description is required".to_string(),
                severity: ValidationSeverity::Error,
                suggestion: Some("Provide a detailed description of the weakness".to_string()),
            });
        }

        // Validate severity
        // (Severity is an enum, so it's always valid)

        // Validate dates
        if let (Some(scheduled), Some(actual)) = (&item.scheduled_completion_date, &item.actual_completion_date) {
            if actual < scheduled {
                results.push(PoamValidationResult {
                    row_number: 0,
                    field_name: "actual_completion_date".to_string(),
                    error_message: "Actual completion date is before scheduled date".to_string(),
                    severity: ValidationSeverity::Warning,
                    suggestion: Some("Verify the completion dates are correct".to_string()),
                });
            }
        }

        results
    }
}

impl PoamDataEnricher {
    /// Create a new data enricher
    pub fn new() -> Self {
        Self {
            risk_calculator: RiskCalculator::new(),
        }
    }

    /// Enrich POA&M item with calculated fields
    pub fn enrich_poam_item(&self, mut item: PoamItem) -> PoamItem {
        // Calculate risk rating if not already set
        if item.risk_rating.is_none() {
            if let (Some(likelihood), Some(impact)) = (&item.likelihood, &item.impact) {
                item.risk_rating = Some(self.risk_calculator.calculate_risk_rating(
                    &item.severity,
                    likelihood,
                    impact,
                ));
            }
        }

        // TODO: Add other enrichment logic
        // - Validate control IDs against NIST catalog
        // - Enrich with control family information
        // - Calculate compliance scores
        // - Add related controls

        item
    }
}

impl RiskCalculator {
    /// Create a new risk calculator
    pub fn new() -> Self {
        Self {
            risk_matrix: RiskMatrix::default(),
        }
    }

    /// Calculate risk rating based on severity, likelihood, and impact
    pub fn calculate_risk_rating(
        &self,
        severity: &PoamSeverity,
        likelihood: &PoamLikelihood,
        impact: &PoamImpact,
    ) -> RiskRating {
        // Simple risk calculation based on severity and likelihood
        // In practice, this would use a more sophisticated risk matrix

        let severity_score = match severity {
            PoamSeverity::Critical => 5,
            PoamSeverity::High => 4,
            PoamSeverity::Medium => 3,
            PoamSeverity::Low => 2,
            PoamSeverity::Info => 1,
        };

        let likelihood_score = match likelihood {
            PoamLikelihood::VeryHigh => 5,
            PoamLikelihood::High => 4,
            PoamLikelihood::Medium => 3,
            PoamLikelihood::Low => 2,
            PoamLikelihood::VeryLow => 1,
        };

        let impact_score = match impact {
            PoamImpact::VeryHigh => 5,
            PoamImpact::High => 4,
            PoamImpact::Medium => 3,
            PoamImpact::Low => 2,
            PoamImpact::VeryLow => 1,
        };

        // Calculate composite risk score
        let risk_score = (severity_score + likelihood_score + impact_score) as f64 / 3.0;

        match risk_score {
            score if score >= 4.5 => RiskRating::VeryHigh,
            score if score >= 3.5 => RiskRating::High,
            score if score >= 2.5 => RiskRating::Medium,
            score if score >= 1.5 => RiskRating::Low,
            _ => RiskRating::VeryLow,
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
