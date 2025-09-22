//! POA&M template detection functionality
//! Modified: 2025-01-22

use crate::{Error, Result};
use super::types::*;

impl PoamTemplateDetector {
    /// Create a new template detector with default signatures
    pub fn new() -> Self {
        let mut detector = Self {
            template_signatures: Vec::new(),
            confidence_threshold: 0.7,
        };
        detector.initialize_default_signatures();
        detector
    }

    /// Initialize default POA&M template signatures
    fn initialize_default_signatures(&mut self) {
        // FedRAMP POA&M v3.0 template
        self.template_signatures.push(TemplateSignature {
            template_id: "fedramp_poam_v3".to_string(),
            name: "FedRAMP POA&M v3.0".to_string(),
            version: "3.0".to_string(),
            required_patterns: vec![
                "POA&M Item ID".to_string(),
                "Vulnerability Description".to_string(),
                "Security Control Number".to_string(),
                "Severity".to_string(),
                "POA&M Status".to_string(),
            ],
            optional_patterns: vec![
                "Office/Organization".to_string(),
                "Scheduled Completion Date".to_string(),
                "Actual Completion Date".to_string(),
                "Point of Contact".to_string(),
                "Resources Required".to_string(),
            ],
            worksheet_names: vec![
                "POA&M Items".to_string(),
                "Milestones".to_string(),
                "Resources".to_string(),
            ],
            weight: 1.0,
        });

        // FedRAMP POA&M v2.0 template
        self.template_signatures.push(TemplateSignature {
            template_id: "fedramp_poam_v2".to_string(),
            name: "FedRAMP POA&M v2.0".to_string(),
            version: "2.0".to_string(),
            required_patterns: vec![
                "POA&M ID".to_string(),
                "Weakness Description".to_string(),
                "Control Number".to_string(),
                "Risk Level".to_string(),
                "Status".to_string(),
            ],
            optional_patterns: vec![
                "Organization".to_string(),
                "Completion Date".to_string(),
                "Contact".to_string(),
                "Resources".to_string(),
            ],
            worksheet_names: vec![
                "POA&M".to_string(),
                "Milestones".to_string(),
            ],
            weight: 0.8,
        });

        // Generic POA&M template
        self.template_signatures.push(TemplateSignature {
            template_id: "generic_poam".to_string(),
            name: "Generic POA&M".to_string(),
            version: "1.0".to_string(),
            required_patterns: vec![
                "ID".to_string(),
                "Description".to_string(),
                "Control".to_string(),
                "Status".to_string(),
            ],
            optional_patterns: vec![
                "Severity".to_string(),
                "Date".to_string(),
                "Owner".to_string(),
            ],
            worksheet_names: vec![
                "POA&M".to_string(),
                "POAM".to_string(),
                "Items".to_string(),
            ],
            weight: 0.5,
        });
    }

    /// Detect POA&M template from column headers
    pub fn detect_template(&self, headers: &[String]) -> Result<TemplateInfo> {
        let mut best_match: Option<(TemplateInfo, f64)> = None;

        for signature in &self.template_signatures {
            let confidence = self.calculate_template_confidence(headers, signature);
            
            if confidence >= self.confidence_threshold {
                let matched_patterns = self.find_matched_patterns(headers, signature);
                let missing_patterns = self.find_missing_patterns(headers, signature);
                
                let template_info = TemplateInfo {
                    template_id: signature.template_id.clone(),
                    name: signature.name.clone(),
                    version: signature.version.clone(),
                    confidence,
                    matched_patterns,
                    missing_patterns,
                    metadata: std::collections::HashMap::new(),
                };

                match &best_match {
                    None => best_match = Some((template_info, confidence)),
                    Some((_, best_confidence)) => {
                        if confidence > *best_confidence {
                            best_match = Some((template_info, confidence));
                        }
                    }
                }
            }
        }

        match best_match {
            Some((template_info, _)) => Ok(template_info),
            None => {
                // Return default template info if no match found
                Ok(TemplateInfo {
                    template_id: "unknown".to_string(),
                    name: "Unknown Template".to_string(),
                    version: "0.0".to_string(),
                    confidence: 0.0,
                    matched_patterns: Vec::new(),
                    missing_patterns: Vec::new(),
                    metadata: std::collections::HashMap::new(),
                })
            }
        }
    }

    /// Calculate confidence score for template match
    fn calculate_template_confidence(&self, headers: &[String], signature: &TemplateSignature) -> f64 {
        let normalized_headers: Vec<String> = headers.iter()
            .map(|h| self.normalize_header(h))
            .collect();

        let mut required_matches = 0;
        let mut optional_matches = 0;

        // Check required patterns
        for pattern in &signature.required_patterns {
            let normalized_pattern = self.normalize_header(pattern);
            if normalized_headers.iter().any(|h| self.fuzzy_match(h, &normalized_pattern)) {
                required_matches += 1;
            }
        }

        // Check optional patterns
        for pattern in &signature.optional_patterns {
            let normalized_pattern = self.normalize_header(pattern);
            if normalized_headers.iter().any(|h| self.fuzzy_match(h, &normalized_pattern)) {
                optional_matches += 1;
            }
        }

        // Calculate confidence based on matches
        let required_score = if signature.required_patterns.is_empty() {
            1.0
        } else {
            required_matches as f64 / signature.required_patterns.len() as f64
        };

        let optional_score = if signature.optional_patterns.is_empty() {
            0.0
        } else {
            optional_matches as f64 / signature.optional_patterns.len() as f64
        };

        // Weight required patterns more heavily
        let confidence = (required_score * 0.8 + optional_score * 0.2) * signature.weight;
        confidence.min(1.0)
    }

    /// Find matched patterns
    fn find_matched_patterns(&self, headers: &[String], signature: &TemplateSignature) -> Vec<String> {
        let normalized_headers: Vec<String> = headers.iter()
            .map(|h| self.normalize_header(h))
            .collect();

        let mut matched = Vec::new();

        for pattern in &signature.required_patterns {
            let normalized_pattern = self.normalize_header(pattern);
            if normalized_headers.iter().any(|h| self.fuzzy_match(h, &normalized_pattern)) {
                matched.push(pattern.clone());
            }
        }

        for pattern in &signature.optional_patterns {
            let normalized_pattern = self.normalize_header(pattern);
            if normalized_headers.iter().any(|h| self.fuzzy_match(h, &normalized_pattern)) {
                matched.push(pattern.clone());
            }
        }

        matched
    }

    /// Find missing required patterns
    fn find_missing_patterns(&self, headers: &[String], signature: &TemplateSignature) -> Vec<String> {
        let normalized_headers: Vec<String> = headers.iter()
            .map(|h| self.normalize_header(h))
            .collect();

        signature.required_patterns.iter()
            .filter(|pattern| {
                let normalized_pattern = self.normalize_header(pattern);
                !normalized_headers.iter().any(|h| self.fuzzy_match(h, &normalized_pattern))
            })
            .cloned()
            .collect()
    }

    /// Normalize header for comparison
    pub fn normalize_header(&self, header: &str) -> String {
        header.to_lowercase()
            .replace("&", "and")
            .replace("-", " ")
            .replace("_", " ")
            .replace("  ", " ")
            .trim()
            .to_string()
    }

    /// Simple fuzzy matching for headers
    pub fn fuzzy_match(&self, header1: &str, header2: &str) -> bool {
        if header1 == header2 {
            return true;
        }

        // Check if one contains the other
        if header1.contains(header2) || header2.contains(header1) {
            return true;
        }

        // Simple word-based matching
        let words1: std::collections::HashSet<&str> = header1.split_whitespace().collect();
        let words2: std::collections::HashSet<&str> = header2.split_whitespace().collect();

        let intersection = words1.intersection(&words2).count();
        let union = words1.union(&words2).count();

        if union == 0 {
            return false;
        }

        let jaccard_similarity = intersection as f64 / union as f64;
        jaccard_similarity >= 0.5
    }

    /// Add custom template signature
    pub fn add_template_signature(&mut self, signature: TemplateSignature) {
        self.template_signatures.push(signature);
    }

    /// Set confidence threshold
    pub fn set_confidence_threshold(&mut self, threshold: f64) {
        self.confidence_threshold = threshold.clamp(0.0, 1.0);
    }

    /// Get confidence threshold
    pub fn confidence_threshold(&self) -> f64 {
        self.confidence_threshold
    }

    /// Get all template signatures
    pub fn template_signatures(&self) -> &[TemplateSignature] {
        &self.template_signatures
    }
}

impl Default for PoamTemplateDetector {
    fn default() -> Self {
        Self::new()
    }
}
