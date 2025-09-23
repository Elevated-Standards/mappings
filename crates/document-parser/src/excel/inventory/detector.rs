// Modified: 2025-09-23

//! Template detection for inventory Excel files
//! 
//! This module provides functionality to automatically detect the type and structure
//! of inventory templates, enabling appropriate parsing strategies.

use super::types::*;
use super::parser::MockWorkbook;
use crate::Result;
use fedramp_core::Error;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tracing::{debug, info, warn};

/// Template detector for identifying inventory formats
#[derive(Debug, Clone)]
pub struct InventoryTemplateDetector {
    /// Known template patterns
    template_patterns: Vec<TemplatePattern>,
    /// Configuration for detection behavior
    config: DetectorConfig,
}

/// Configuration for template detection
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DetectorConfig {
    /// Enable fuzzy matching for worksheet names
    pub enable_fuzzy_matching: bool,
    /// Minimum confidence threshold for detection
    pub confidence_threshold: f64,
    /// Maximum worksheets to analyze
    pub max_worksheets: usize,
    /// Enable header analysis
    pub analyze_headers: bool,
}

impl Default for DetectorConfig {
    fn default() -> Self {
        Self {
            enable_fuzzy_matching: true,
            confidence_threshold: 0.7,
            max_worksheets: 20,
            analyze_headers: true,
        }
    }
}

/// Template pattern for matching
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemplatePattern {
    /// Template type this pattern matches
    pub template_type: InventoryTemplateType,
    /// Template version
    pub version: String,
    /// Required worksheet names (case-insensitive)
    pub required_worksheets: Vec<String>,
    /// Optional worksheet names
    pub optional_worksheets: Vec<String>,
    /// Required header patterns
    pub required_headers: HashMap<String, Vec<String>>,
    /// Template confidence weight
    pub confidence_weight: f64,
}

/// Detection result with confidence scoring
#[derive(Debug, Clone)]
pub struct DetectionResult {
    /// Detected template type
    pub template_type: InventoryTemplateType,
    /// Detection confidence (0.0 to 1.0)
    pub confidence: f64,
    /// Template information
    pub template_info: InventoryTemplateInfo,
    /// Detection details
    pub details: DetectionDetails,
}

/// Detailed detection information
#[derive(Debug, Clone)]
pub struct DetectionDetails {
    /// Matched worksheet patterns
    pub matched_worksheets: Vec<String>,
    /// Matched header patterns
    pub matched_headers: HashMap<String, Vec<String>>,
    /// Confidence breakdown by category
    pub confidence_breakdown: HashMap<String, f64>,
    /// Detection warnings
    pub warnings: Vec<String>,
}

impl InventoryTemplateDetector {
    /// Create a new template detector with default patterns
    pub fn new() -> Self {
        let template_patterns = Self::create_default_patterns();
        let config = DetectorConfig::default();

        Self {
            template_patterns,
            config,
        }
    }

    /// Create detector with custom configuration
    pub fn with_config(config: DetectorConfig) -> Self {
        let template_patterns = Self::create_default_patterns();

        Self {
            template_patterns,
            config,
        }
    }

    /// Detect template type from workbook
    pub fn detect_template(&self, workbook: &MockWorkbook) -> Result<InventoryTemplateInfo> {
        info!("Starting template detection");

        let worksheet_names = workbook.get_worksheet_names();
        debug!("Found worksheets: {:?}", worksheet_names);

        let mut best_match: Option<DetectionResult> = None;
        let mut best_confidence = 0.0;

        // Test each template pattern
        for pattern in &self.template_patterns {
            let result = self.test_pattern(pattern, workbook, &worksheet_names)?;
            
            debug!(
                "Pattern {:?} confidence: {:.2}",
                pattern.template_type, result.confidence
            );

            if result.confidence > best_confidence && result.confidence >= self.config.confidence_threshold {
                best_confidence = result.confidence;
                best_match = Some(result);
            }
        }

        match best_match {
            Some(result) => {
                info!(
                    "Detected template: {:?} with confidence {:.2}",
                    result.template_type, result.confidence
                );
                Ok(result.template_info)
            }
            None => {
                warn!("No template pattern matched with sufficient confidence");
                // Return a custom template as fallback
                Ok(self.create_custom_template(workbook, &worksheet_names)?)
            }
        }
    }

    /// Test a specific pattern against the workbook
    fn test_pattern(
        &self,
        pattern: &TemplatePattern,
        workbook: &MockWorkbook,
        worksheet_names: &[String],
    ) -> Result<DetectionResult> {
        let mut confidence_scores = HashMap::new();
        let mut matched_worksheets = Vec::new();
        let mut matched_headers = HashMap::new();
        let mut warnings = Vec::new();

        // Test worksheet name matching
        let worksheet_confidence = self.test_worksheet_names(
            pattern,
            worksheet_names,
            &mut matched_worksheets,
            &mut warnings,
        );
        confidence_scores.insert("worksheets".to_string(), worksheet_confidence);

        // Test header patterns if enabled
        let header_confidence = if self.config.analyze_headers {
            self.test_header_patterns(
                pattern,
                workbook,
                &matched_worksheets,
                &mut matched_headers,
                &mut warnings,
            )?
        } else {
            1.0 // Skip header analysis
        };
        confidence_scores.insert("headers".to_string(), header_confidence);

        // Calculate overall confidence
        let overall_confidence = self.calculate_overall_confidence(&confidence_scores, pattern);

        // Create template info if confidence is sufficient
        let template_info = if overall_confidence >= self.config.confidence_threshold {
            self.create_template_info(pattern, &matched_worksheets, &matched_headers)?
        } else {
            // Create minimal template info for low confidence matches
            InventoryTemplateInfo {
                template_type: pattern.template_type.clone(),
                version: pattern.version.clone(),
                asset_worksheets: matched_worksheets.clone(),
                relationship_worksheets: Vec::new(),
                column_mappings: HashMap::new(),
            }
        };

        Ok(DetectionResult {
            template_type: pattern.template_type.clone(),
            confidence: overall_confidence,
            template_info,
            details: DetectionDetails {
                matched_worksheets,
                matched_headers,
                confidence_breakdown: confidence_scores,
                warnings,
            },
        })
    }

    /// Test worksheet name matching
    fn test_worksheet_names(
        &self,
        pattern: &TemplatePattern,
        worksheet_names: &[String],
        matched_worksheets: &mut Vec<String>,
        warnings: &mut Vec<String>,
    ) -> f64 {
        let mut required_matches = 0;
        let mut optional_matches = 0;

        // Check required worksheets
        for required in &pattern.required_worksheets {
            if let Some(matched) = self.find_matching_worksheet(required, worksheet_names) {
                required_matches += 1;
                matched_worksheets.push(matched);
            } else {
                warnings.push(format!("Required worksheet '{}' not found", required));
            }
        }

        // Check optional worksheets
        for optional in &pattern.optional_worksheets {
            if let Some(matched) = self.find_matching_worksheet(optional, worksheet_names) {
                optional_matches += 1;
                matched_worksheets.push(matched);
            }
        }

        // Calculate confidence based on matches
        let required_ratio = if pattern.required_worksheets.is_empty() {
            1.0
        } else {
            required_matches as f64 / pattern.required_worksheets.len() as f64
        };

        let optional_ratio = if pattern.optional_worksheets.is_empty() {
            0.0
        } else {
            optional_matches as f64 / pattern.optional_worksheets.len() as f64
        };

        // Weight required matches more heavily
        (required_ratio * 0.8) + (optional_ratio * 0.2)
    }

    /// Find matching worksheet name (with fuzzy matching if enabled)
    fn find_matching_worksheet(&self, target: &str, worksheet_names: &[String]) -> Option<String> {
        let target_lower = target.to_lowercase();

        // Exact match first
        for name in worksheet_names {
            if name.to_lowercase() == target_lower {
                return Some(name.clone());
            }
        }

        // Fuzzy matching if enabled
        if self.config.enable_fuzzy_matching {
            for name in worksheet_names {
                let name_lower = name.to_lowercase();
                
                // Contains match
                if name_lower.contains(&target_lower) || target_lower.contains(&name_lower) {
                    return Some(name.clone());
                }

                // Word-based matching
                if self.fuzzy_match_words(&target_lower, &name_lower) {
                    return Some(name.clone());
                }
            }
        }

        None
    }

    /// Simple fuzzy matching based on word overlap
    fn fuzzy_match_words(&self, target: &str, candidate: &str) -> bool {
        let target_words: Vec<&str> = target.split_whitespace().collect();
        let candidate_words: Vec<&str> = candidate.split_whitespace().collect();

        if target_words.is_empty() || candidate_words.is_empty() {
            return false;
        }

        let mut matches = 0;
        for target_word in &target_words {
            for candidate_word in &candidate_words {
                if target_word == candidate_word || 
                   candidate_word.contains(target_word) || 
                   target_word.contains(candidate_word) {
                    matches += 1;
                    break;
                }
            }
        }

        // Require at least 50% word overlap
        (matches as f64 / target_words.len() as f64) >= 0.5
    }

    /// Test header patterns in worksheets
    fn test_header_patterns(
        &self,
        pattern: &TemplatePattern,
        workbook: &MockWorkbook,
        matched_worksheets: &[String],
        matched_headers: &mut HashMap<String, Vec<String>>,
        warnings: &mut Vec<String>,
    ) -> Result<f64> {
        if pattern.required_headers.is_empty() {
            return Ok(1.0);
        }

        let mut total_confidence = 0.0;
        let mut worksheet_count = 0;

        for worksheet_name in matched_worksheets {
            if let Some(expected_headers) = pattern.required_headers.get(worksheet_name) {
                match self.extract_worksheet_headers(workbook, worksheet_name) {
                    Ok(actual_headers) => {
                        let confidence = self.calculate_header_confidence(expected_headers, &actual_headers);
                        total_confidence += confidence;
                        worksheet_count += 1;

                        let mut matched_for_worksheet = Vec::new();
                        for expected in expected_headers {
                            if self.find_matching_header(expected, &actual_headers).is_some() {
                                matched_for_worksheet.push(expected.clone());
                            }
                        }
                        matched_headers.insert(worksheet_name.clone(), matched_for_worksheet);
                    }
                    Err(e) => {
                        warnings.push(format!("Failed to extract headers from '{}': {}", worksheet_name, e));
                    }
                }
            }
        }

        Ok(if worksheet_count > 0 {
            total_confidence / worksheet_count as f64
        } else {
            0.0
        })
    }

    /// Extract headers from a worksheet
    fn extract_worksheet_headers(&self, workbook: &MockWorkbook, worksheet_name: &str) -> Result<Vec<String>> {
        // This is a placeholder implementation
        // In a real implementation, this would extract the first row as headers
        let _worksheet_data = workbook.get_worksheet_data(worksheet_name)?;
        
        // For now, return some common inventory headers
        Ok(vec![
            "Asset ID".to_string(),
            "Asset Name".to_string(),
            "Asset Type".to_string(),
            "Description".to_string(),
            "Owner".to_string(),
            "Environment".to_string(),
            "Criticality".to_string(),
            "IP Address".to_string(),
            "MAC Address".to_string(),
            "Operating System".to_string(),
        ])
    }

    /// Find matching header with fuzzy matching
    fn find_matching_header(&self, target: &str, headers: &[String]) -> Option<String> {
        let target_lower = target.to_lowercase();

        // Exact match
        for header in headers {
            if header.to_lowercase() == target_lower {
                return Some(header.clone());
            }
        }

        // Fuzzy match
        if self.config.enable_fuzzy_matching {
            for header in headers {
                let header_lower = header.to_lowercase();
                if header_lower.contains(&target_lower) || target_lower.contains(&header_lower) {
                    return Some(header.clone());
                }
            }
        }

        None
    }

    /// Calculate header matching confidence
    fn calculate_header_confidence(&self, expected: &[String], actual: &[String]) -> f64 {
        if expected.is_empty() {
            return 1.0;
        }

        let mut matches = 0;
        for expected_header in expected {
            if self.find_matching_header(expected_header, actual).is_some() {
                matches += 1;
            }
        }

        matches as f64 / expected.len() as f64
    }

    /// Calculate overall confidence from component scores
    fn calculate_overall_confidence(
        &self,
        confidence_scores: &HashMap<String, f64>,
        pattern: &TemplatePattern,
    ) -> f64 {
        let worksheet_weight = 0.7;
        let header_weight = 0.3;

        let worksheet_confidence = confidence_scores.get("worksheets").unwrap_or(&0.0);
        let header_confidence = confidence_scores.get("headers").unwrap_or(&1.0);

        let base_confidence = (worksheet_confidence * worksheet_weight) + (header_confidence * header_weight);
        
        // Apply pattern-specific weight
        base_confidence * pattern.confidence_weight
    }

    /// Create template info from detection results
    fn create_template_info(
        &self,
        pattern: &TemplatePattern,
        matched_worksheets: &[String],
        matched_headers: &HashMap<String, Vec<String>>,
    ) -> Result<InventoryTemplateInfo> {
        // Determine asset and relationship worksheets
        let mut asset_worksheets = Vec::new();
        let mut relationship_worksheets = Vec::new();

        for worksheet in matched_worksheets {
            let worksheet_lower = worksheet.to_lowercase();
            if worksheet_lower.contains("relationship") || 
               worksheet_lower.contains("dependency") ||
               worksheet_lower.contains("connection") {
                relationship_worksheets.push(worksheet.clone());
            } else {
                asset_worksheets.push(worksheet.clone());
            }
        }

        // Create column mappings from matched headers
        let mut column_mappings = HashMap::new();
        for (worksheet, headers) in matched_headers {
            let mut mapping = HashMap::new();
            for header in headers {
                // Create standard field mappings
                let standard_field = self.map_header_to_standard_field(header);
                mapping.insert(header.clone(), standard_field);
            }
            column_mappings.insert(worksheet.clone(), mapping);
        }

        Ok(InventoryTemplateInfo {
            template_type: pattern.template_type.clone(),
            version: pattern.version.clone(),
            asset_worksheets,
            relationship_worksheets,
            column_mappings,
        })
    }

    /// Map header to standard field name
    fn map_header_to_standard_field(&self, header: &str) -> String {
        let header_lower = header.to_lowercase();
        
        if header_lower.contains("asset") && header_lower.contains("id") {
            "asset_id".to_string()
        } else if header_lower.contains("asset") && header_lower.contains("name") {
            "asset_name".to_string()
        } else if header_lower.contains("type") {
            "asset_type".to_string()
        } else if header_lower.contains("description") {
            "description".to_string()
        } else if header_lower.contains("owner") {
            "owner".to_string()
        } else if header_lower.contains("environment") {
            "environment".to_string()
        } else if header_lower.contains("criticality") || header_lower.contains("critical") {
            "criticality".to_string()
        } else if header_lower.contains("ip") {
            "ip_address".to_string()
        } else if header_lower.contains("mac") {
            "mac_address".to_string()
        } else if header_lower.contains("os") || header_lower.contains("operating") {
            "operating_system".to_string()
        } else {
            // Use header as-is for unknown fields
            header.to_lowercase().replace(' ', "_")
        }
    }

    /// Create custom template for unrecognized formats
    fn create_custom_template(
        &self,
        workbook: &MockWorkbook,
        worksheet_names: &[String],
    ) -> Result<InventoryTemplateInfo> {
        info!("Creating custom template for unrecognized format");

        Ok(InventoryTemplateInfo {
            template_type: InventoryTemplateType::Custom,
            version: "1.0".to_string(),
            asset_worksheets: worksheet_names.to_vec(),
            relationship_worksheets: Vec::new(),
            column_mappings: HashMap::new(),
        })
    }



    /// Create default template patterns
    fn create_default_patterns() -> Vec<TemplatePattern> {
        vec![
            // FedRAMP Integrated Inventory Workbook
            TemplatePattern {
                template_type: InventoryTemplateType::FedRampIntegrated,
                version: "1.0".to_string(),
                required_worksheets: vec![
                    "Hardware Inventory".to_string(),
                    "Software Inventory".to_string(),
                ],
                optional_worksheets: vec![
                    "Network Devices".to_string(),
                    "Asset Relationships".to_string(),
                    "Vulnerability Assessment".to_string(),
                ],
                required_headers: {
                    let mut headers = HashMap::new();
                    headers.insert("Hardware Inventory".to_string(), vec![
                        "Asset ID".to_string(),
                        "Asset Name".to_string(),
                        "Asset Type".to_string(),
                        "Owner".to_string(),
                        "Environment".to_string(),
                    ]);
                    headers.insert("Software Inventory".to_string(), vec![
                        "Software Name".to_string(),
                        "Version".to_string(),
                        "Vendor".to_string(),
                        "License".to_string(),
                    ]);
                    headers
                },
                confidence_weight: 1.0,
            },
            // Network-focused inventory
            TemplatePattern {
                template_type: InventoryTemplateType::NetworkInventory,
                version: "1.0".to_string(),
                required_worksheets: vec![
                    "Network Devices".to_string(),
                ],
                optional_worksheets: vec![
                    "Network Topology".to_string(),
                    "IP Addresses".to_string(),
                ],
                required_headers: {
                    let mut headers = HashMap::new();
                    headers.insert("Network Devices".to_string(), vec![
                        "Device Name".to_string(),
                        "IP Address".to_string(),
                        "MAC Address".to_string(),
                        "Device Type".to_string(),
                    ]);
                    headers
                },
                confidence_weight: 0.9,
            },
            // Software-focused inventory
            TemplatePattern {
                template_type: InventoryTemplateType::SoftwareInventory,
                version: "1.0".to_string(),
                required_worksheets: vec![
                    "Software".to_string(),
                ],
                optional_worksheets: vec![
                    "Licenses".to_string(),
                    "Dependencies".to_string(),
                ],
                required_headers: {
                    let mut headers = HashMap::new();
                    headers.insert("Software".to_string(), vec![
                        "Software Name".to_string(),
                        "Version".to_string(),
                        "Vendor".to_string(),
                    ]);
                    headers
                },
                confidence_weight: 0.8,
            },
        ]
    }

    /// Get detector configuration
    pub fn get_config(&self) -> &DetectorConfig {
        &self.config
    }

    /// Update detector configuration
    pub fn update_config(&mut self, config: DetectorConfig) {
        self.config = config;
    }

    /// Add custom template pattern
    pub fn add_template_pattern(&mut self, pattern: TemplatePattern) {
        self.template_patterns.push(pattern);
    }
}

impl Default for InventoryTemplateDetector {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detector_creation() {
        let detector = InventoryTemplateDetector::new();
        assert!(!detector.template_patterns.is_empty());
    }

    #[test]
    fn test_fuzzy_matching() {
        let detector = InventoryTemplateDetector::new();
        
        assert!(detector.fuzzy_match_words("hardware inventory", "hardware_inventory_sheet"));
        assert!(detector.fuzzy_match_words("software", "software applications"));
        assert!(!detector.fuzzy_match_words("hardware", "network devices"));
    }

    #[test]
    fn test_header_mapping() {
        let detector = InventoryTemplateDetector::new();
        
        assert_eq!(detector.map_header_to_standard_field("Asset ID"), "asset_id");
        assert_eq!(detector.map_header_to_standard_field("Asset Name"), "asset_name");
        assert_eq!(detector.map_header_to_standard_field("IP Address"), "ip_address");
        assert_eq!(detector.map_header_to_standard_field("Custom Field"), "custom_field");
    }
}
