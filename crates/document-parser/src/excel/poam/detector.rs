//! POA&M template detection functionality
//! Modified: 2025-01-22

use serde_json::Value;
use std::collections::HashMap;

use super::types::*;

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

    /// Add custom template signature
    pub fn add_template_signature(&mut self, signature: TemplateSignature) {
        self.template_signatures.push(signature);
    }

    /// Get all template signatures
    pub fn template_signatures(&self) -> &[TemplateSignature] {
        &self.template_signatures
    }

    /// Set template signatures
    pub fn set_template_signatures(&mut self, signatures: Vec<TemplateSignature>) {
        self.template_signatures = signatures;
    }

    /// Clear all template signatures
    pub fn clear_template_signatures(&mut self) {
        self.template_signatures.clear();
    }

    /// Get template signature by name
    pub fn get_template_signature(&self, name: &str) -> Option<&TemplateSignature> {
        self.template_signatures.iter().find(|sig| sig.name == name)
    }

    /// Remove template signature by name
    pub fn remove_template_signature(&mut self, name: &str) -> bool {
        if let Some(pos) = self.template_signatures.iter().position(|sig| sig.name == name) {
            self.template_signatures.remove(pos);
            true
        } else {
            false
        }
    }

    /// Update template signature
    pub fn update_template_signature(&mut self, signature: TemplateSignature) -> bool {
        if let Some(existing) = self.template_signatures.iter_mut().find(|sig| sig.name == signature.name) {
            *existing = signature;
            true
        } else {
            false
        }
    }

    /// Get template signature count
    pub fn template_signature_count(&self) -> usize {
        self.template_signatures.len()
    }

    /// Check if template signature exists
    pub fn has_template_signature(&self, name: &str) -> bool {
        self.template_signatures.iter().any(|sig| sig.name == name)
    }
}

impl Default for PoamTemplateDetector {
    fn default() -> Self {
        Self::new()
    }
}
