//! Main POA&M parser implementation
//! Modified: 2025-01-22

use crate::excel::core::ExcelParser;
use crate::excel::types::ValidationSeverity;
use serde_json::Value;
use chrono::Utc;
use tracing::{debug, info, warn};
use fedramp_core::{Result, Error};

use super::types::*;

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
        _row_data: &[Value],
        row_index: usize,
        _headers: &Option<Vec<String>>,
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

impl Default for PoamParser {
    fn default() -> Self {
        Self::new()
    }
}
