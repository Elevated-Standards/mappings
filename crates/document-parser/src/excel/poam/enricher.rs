//! POA&M data enrichment and risk calculation functionality
//! Modified: 2025-01-22

use super::types::*;

impl PoamDataEnricher {
    /// Create a new data enricher
    pub fn new() -> Self {
        Self {
            risk_calculator: RiskCalculator::new(),
        }
    }

    /// Create a new data enricher with custom risk calculator
    pub fn with_risk_calculator(risk_calculator: RiskCalculator) -> Self {
        Self {
            risk_calculator,
        }
    }

    /// Get the risk calculator
    pub fn risk_calculator(&self) -> &RiskCalculator {
        &self.risk_calculator
    }

    /// Set the risk calculator
    pub fn set_risk_calculator(&mut self, risk_calculator: RiskCalculator) {
        self.risk_calculator = risk_calculator;
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

        // Enrich control information
        item = self.enrich_control_information(item);

        // Enrich security control names
        item = self.enrich_security_control_names(item);

        // Calculate milestone progress
        item = self.calculate_milestone_progress(item);

        // Calculate resource totals
        item = self.calculate_resource_totals(item);

        item
    }

    /// Enrich multiple POA&M items
    pub fn enrich_poam_items(&self, items: Vec<PoamItem>) -> Vec<PoamItem> {
        items.into_iter().map(|item| self.enrich_poam_item(item)).collect()
    }

    /// Enrich control information based on control ID
    fn enrich_control_information(&self, mut item: PoamItem) -> PoamItem {
        if let Some(control_id) = &item.control_id {
            // Extract control family from control ID
            if let Some(family) = self.extract_control_family(control_id) {
                // Add control family to security controls if not already present
                if !item.security_controls.contains(&family) {
                    item.security_controls.push(family);
                }
            }

            // TODO: Add other control enrichment logic
            // - Validate control IDs against NIST catalog
            // - Enrich with control family information
            // - Add related controls
            // - Add control implementation guidance
        }

        item
    }

    /// Enrich security control names based on control IDs
    fn enrich_security_control_names(&self, mut item: PoamItem) -> PoamItem {
        // Clear existing names to rebuild from current control IDs
        item.security_control_names.clear();

        // Add names for each security control
        for control in &item.security_controls {
            if let Some(name) = self.get_control_name(control) {
                item.security_control_names.push(name);
            }
        }

        item
    }

    /// Calculate milestone progress and update item status
    fn calculate_milestone_progress(&self, mut item: PoamItem) -> PoamItem {
        if !item.milestones.is_empty() {
            let completed_milestones = item.milestones.iter()
                .filter(|m| m.status == MilestoneStatus::Completed)
                .count();

            let total_milestones = item.milestones.len();
            let progress_percentage = (completed_milestones as f64 / total_milestones as f64) * 100.0;

            // Update item status based on milestone progress
            if progress_percentage == 100.0 && item.status != PoamStatus::Completed {
                // All milestones completed, but item not marked as completed
                // This might indicate the item should be reviewed for completion
            } else if progress_percentage > 0.0 && item.status == PoamStatus::Open {
                // Some progress made, but item still marked as open
                // This might indicate the item should be marked as in progress
            }
        }

        item
    }

    /// Calculate resource totals and update cost estimates
    fn calculate_resource_totals(&self, mut item: PoamItem) -> PoamItem {
        if !item.resources.is_empty() {
            let total_estimated_cost: f64 = item.resources.iter()
                .filter_map(|r| r.estimated_cost)
                .sum();

            let total_actual_cost: f64 = item.resources.iter()
                .filter_map(|r| r.actual_cost)
                .sum();

            // Update item cost estimate if not already set
            if item.cost_estimate.is_none() && total_estimated_cost > 0.0 {
                item.cost_estimate = Some(total_estimated_cost);
            }

            // TODO: Add logic to track actual vs estimated costs
            // TODO: Add resource utilization metrics
        }

        item
    }

    /// Extract control family from control ID
    fn extract_control_family(&self, control_id: &str) -> Option<String> {
        // Extract family prefix from control ID (e.g., "AC" from "AC-1")
        let parts: Vec<&str> = control_id.split('-').collect();
        if parts.len() >= 2 {
            Some(parts[0].to_uppercase())
        } else {
            None
        }
    }

    /// Get human-readable control name from control ID
    fn get_control_name(&self, control_id: &str) -> Option<String> {
        // This is a simplified mapping - in practice, this would use
        // a comprehensive NIST control catalog
        match control_id.to_uppercase().as_str() {
            "AC" => Some("Access Control".to_string()),
            "AU" => Some("Audit and Accountability".to_string()),
            "AT" => Some("Awareness and Training".to_string()),
            "CM" => Some("Configuration Management".to_string()),
            "CP" => Some("Contingency Planning".to_string()),
            "IA" => Some("Identification and Authentication".to_string()),
            "IR" => Some("Incident Response".to_string()),
            "MA" => Some("Maintenance".to_string()),
            "MP" => Some("Media Protection".to_string()),
            "PS" => Some("Personnel Security".to_string()),
            "PE" => Some("Physical and Environmental Protection".to_string()),
            "PL" => Some("Planning".to_string()),
            "PM" => Some("Program Management".to_string()),
            "RA" => Some("Risk Assessment".to_string()),
            "CA" => Some("Security Assessment and Authorization".to_string()),
            "SC" => Some("System and Communications Protection".to_string()),
            "SI" => Some("System and Information Integrity".to_string()),
            "SA" => Some("System and Services Acquisition".to_string()),
            _ => None,
        }
    }

    /// Calculate enrichment statistics
    pub fn calculate_enrichment_statistics(&self, original_items: &[PoamItem], enriched_items: &[PoamItem]) -> EnrichmentStatistics {
        let total_items = original_items.len();
        let mut risk_ratings_added = 0;
        let mut control_names_added = 0;
        let mut cost_estimates_added = 0;

        for (original, enriched) in original_items.iter().zip(enriched_items.iter()) {
            if original.risk_rating.is_none() && enriched.risk_rating.is_some() {
                risk_ratings_added += 1;
            }

            if original.security_control_names.len() < enriched.security_control_names.len() {
                control_names_added += 1;
            }

            if original.cost_estimate.is_none() && enriched.cost_estimate.is_some() {
                cost_estimates_added += 1;
            }
        }

        EnrichmentStatistics {
            total_items,
            risk_ratings_added,
            control_names_added,
            cost_estimates_added,
        }
    }
}

impl RiskCalculator {
    /// Create a new risk calculator
    pub fn new() -> Self {
        Self {
            risk_matrix: RiskMatrix::default(),
        }
    }

    /// Create a new risk calculator with custom risk matrix
    pub fn with_risk_matrix(risk_matrix: RiskMatrix) -> Self {
        Self {
            risk_matrix,
        }
    }

    /// Get the risk matrix
    pub fn risk_matrix(&self) -> &RiskMatrix {
        &self.risk_matrix
    }

    /// Set the risk matrix
    pub fn set_risk_matrix(&mut self, risk_matrix: RiskMatrix) {
        self.risk_matrix = risk_matrix;
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

    /// Calculate risk score as numeric value
    pub fn calculate_risk_score(
        &self,
        severity: &PoamSeverity,
        likelihood: &PoamLikelihood,
        impact: &PoamImpact,
    ) -> f64 {
        let severity_score = self.risk_matrix.severity_impact_map.get(severity).copied().unwrap_or(3) as f64;
        let likelihood_probability = self.risk_matrix.likelihood_probability_map.get(likelihood).copied().unwrap_or(0.5);
        let impact_score = match impact {
            PoamImpact::VeryHigh => 5.0,
            PoamImpact::High => 4.0,
            PoamImpact::Medium => 3.0,
            PoamImpact::Low => 2.0,
            PoamImpact::VeryLow => 1.0,
        };

        // Risk = Severity * Likelihood * Impact
        severity_score * likelihood_probability * impact_score
    }
}

/// Enrichment statistics
#[derive(Debug, Clone)]
pub struct EnrichmentStatistics {
    /// Total number of items processed
    pub total_items: usize,
    /// Number of risk ratings added
    pub risk_ratings_added: usize,
    /// Number of items with control names added
    pub control_names_added: usize,
    /// Number of cost estimates added
    pub cost_estimates_added: usize,
}

impl Default for PoamDataEnricher {
    fn default() -> Self {
        Self::new()
    }
}

impl Default for RiskCalculator {
    fn default() -> Self {
        Self::new()
    }
}
