//! Data Consistency Checking for POA&M Items
//! 
//! Validates POA&M data for internal consistency, cross-reference validation, and logical coherence

use super::*;
use crate::poam::PoamItem;
use fedramp_core::{Result, Error};
use tracing::{debug, info};
use uuid::Uuid;
use chrono::{DateTime, Utc};
use std::collections::{HashMap, HashSet};

/// Result of consistency checking
#[derive(Debug, Clone)]
pub struct ConsistencyResult {
    /// Overall consistency score (0.0 to 1.0)
    pub score: f64,
    /// Consistency findings
    pub findings: Vec<QualityFinding>,
    /// Cross-reference validation results
    pub cross_reference_results: Vec<CrossReferenceResult>,
    /// Timeline consistency results
    pub timeline_results: Vec<TimelineConsistencyResult>,
}

/// Cross-reference validation result
#[derive(Debug, Clone)]
pub struct CrossReferenceResult {
    /// Reference type (e.g., "uuid", "status_workflow")
    pub reference_type: String,
    /// Items with valid references
    pub valid_references: usize,
    /// Items with invalid references
    pub invalid_references: usize,
    /// Items with missing references
    pub missing_references: usize,
    /// Success rate
    pub success_rate: f64,
    /// Details of invalid references
    pub invalid_details: Vec<String>,
}

/// Timeline consistency result
#[derive(Debug, Clone)]
pub struct TimelineConsistencyResult {
    /// Timeline check type
    pub check_type: String,
    /// Items that passed timeline checks
    pub passed_items: usize,
    /// Items that failed timeline checks
    pub failed_items: usize,
    /// Success rate
    pub success_rate: f64,
    /// Failed item details
    pub failed_details: Vec<TimelineIssue>,
}

/// Timeline consistency issue
#[derive(Debug, Clone)]
pub struct TimelineIssue {
    /// Item UUID with the issue
    pub item_uuid: String,
    /// Issue description
    pub issue_description: String,
    /// Suggested resolution
    pub suggested_resolution: String,
}

/// Consistency checker for POA&M data
#[derive(Debug, Clone)]
pub struct ConsistencyChecker {
    /// Valid status transitions
    valid_status_transitions: HashMap<String, Vec<String>>,
    /// Enable strict timeline validation
    strict_timeline_validation: bool,
    /// Maximum allowed date variance (in days)
    max_date_variance_days: i64,
}

impl ConsistencyChecker {
    /// Create a new consistency checker with default configuration
    pub fn new() -> Self {
        let mut valid_transitions = HashMap::new();
        
        // Define valid status transitions
        valid_transitions.insert("Open".to_string(), vec![
            "In Progress".to_string(),
            "On Hold".to_string(),
            "Cancelled".to_string(),
        ]);
        
        valid_transitions.insert("In Progress".to_string(), vec![
            "Completed".to_string(),
            "On Hold".to_string(),
            "Cancelled".to_string(),
        ]);
        
        valid_transitions.insert("On Hold".to_string(), vec![
            "In Progress".to_string(),
            "Cancelled".to_string(),
        ]);
        
        valid_transitions.insert("Completed".to_string(), vec![
            "Closed".to_string(),
        ]);

        Self {
            valid_status_transitions: valid_transitions,
            strict_timeline_validation: false,
            max_date_variance_days: 30,
        }
    }

    /// Create a consistency checker with custom configuration
    pub fn with_config(config: &QualityConfig) -> Self {
        let mut checker = Self::new();
        checker.strict_timeline_validation = config.strict_mode;
        checker
    }

    /// Check consistency of POA&M items
    pub fn check(&self, poam_items: &[PoamItem]) -> Result<ConsistencyResult> {
        info!("Checking consistency for {} POA&M items", poam_items.len());

        if poam_items.is_empty() {
            return Ok(ConsistencyResult {
                score: 1.0,
                findings: Vec::new(),
                cross_reference_results: Vec::new(),
                timeline_results: Vec::new(),
            });
        }

        // Perform cross-reference validation
        let cross_reference_results = self.validate_cross_references(poam_items)?;

        // Perform timeline consistency checks
        let timeline_results = self.validate_timeline_consistency(poam_items)?;

        // Generate consistency findings
        let findings = self.generate_consistency_findings(&cross_reference_results, &timeline_results)?;

        // Calculate overall consistency score
        let overall_score = self.calculate_consistency_score(&cross_reference_results, &timeline_results);

        debug!(
            "Consistency checking completed: Score: {:.2}, Findings: {}",
            overall_score,
            findings.len()
        );

        Ok(ConsistencyResult {
            score: overall_score,
            findings,
            cross_reference_results,
            timeline_results,
        })
    }

    /// Validate cross-references between POA&M items
    fn validate_cross_references(&self, poam_items: &[PoamItem]) -> Result<Vec<CrossReferenceResult>> {
        let mut results = Vec::new();

        // Check UUID uniqueness
        results.push(self.check_uuid_uniqueness(poam_items)?);

        // Check status workflow consistency
        results.push(self.check_status_workflow_consistency(poam_items)?);

        // Check milestone consistency
        results.push(self.check_milestone_consistency(poam_items)?);

        Ok(results)
    }

    /// Check UUID uniqueness across items
    fn check_uuid_uniqueness(&self, poam_items: &[PoamItem]) -> Result<CrossReferenceResult> {
        let mut uuid_counts = HashMap::new();
        let mut duplicate_uuids = Vec::new();

        for item in poam_items {
            let count = uuid_counts.entry(item.uuid.clone()).or_insert(0);
            *count += 1;
            if *count > 1 {
                duplicate_uuids.push(format!("UUID '{}' appears {} times", item.uuid, count));
            }
        }

        let unique_uuids = uuid_counts.len();
        let total_items = poam_items.len();
        let duplicate_count = total_items - unique_uuids;

        let success_rate = if total_items > 0 {
            unique_uuids as f64 / total_items as f64
        } else {
            1.0
        };

        Ok(CrossReferenceResult {
            reference_type: "uuid_uniqueness".to_string(),
            valid_references: unique_uuids,
            invalid_references: duplicate_count,
            missing_references: 0,
            success_rate,
            invalid_details: duplicate_uuids,
        })
    }

    /// Check status workflow consistency
    fn check_status_workflow_consistency(&self, poam_items: &[PoamItem]) -> Result<CrossReferenceResult> {
        let mut valid_count = 0;
        let mut invalid_details = Vec::new();

        for item in poam_items {
            let status_valid = self.valid_status_transitions.contains_key(&item.status);
            
            if status_valid {
                valid_count += 1;
            } else {
                invalid_details.push(format!("Item '{}' has invalid status '{}'", item.uuid, item.status));
            }
        }

        let invalid_count = poam_items.len() - valid_count;
        let success_rate = if !poam_items.is_empty() {
            valid_count as f64 / poam_items.len() as f64
        } else {
            1.0
        };

        Ok(CrossReferenceResult {
            reference_type: "status_workflow".to_string(),
            valid_references: valid_count,
            invalid_references: invalid_count,
            missing_references: 0,
            success_rate,
            invalid_details,
        })
    }

    /// Check milestone consistency
    fn check_milestone_consistency(&self, poam_items: &[PoamItem]) -> Result<CrossReferenceResult> {
        let mut valid_count = 0;
        let mut invalid_details = Vec::new();

        for item in poam_items {
            let mut item_valid = true;

            if let Some(milestones) = &item.milestones {
                // Check if milestones are in chronological order
                let mut milestone_dates = Vec::new();
                for milestone in milestones {
                    if let Some(date_str) = &milestone.scheduled_date {
                        if let Ok(date) = self.parse_date_string(date_str) {
                            milestone_dates.push(date);
                        }
                    }
                }

                // Verify chronological order
                for i in 1..milestone_dates.len() {
                    if milestone_dates[i] < milestone_dates[i-1] {
                        item_valid = false;
                        invalid_details.push(format!(
                            "Item '{}' has milestones not in chronological order",
                            item.uuid
                        ));
                        break;
                    }
                }
            }

            if item_valid {
                valid_count += 1;
            }
        }

        let invalid_count = poam_items.len() - valid_count;
        let success_rate = if !poam_items.is_empty() {
            valid_count as f64 / poam_items.len() as f64
        } else {
            1.0
        };

        Ok(CrossReferenceResult {
            reference_type: "milestone_consistency".to_string(),
            valid_references: valid_count,
            invalid_references: invalid_count,
            missing_references: 0,
            success_rate,
            invalid_details,
        })
    }

    /// Validate timeline consistency
    fn validate_timeline_consistency(&self, poam_items: &[PoamItem]) -> Result<Vec<TimelineConsistencyResult>> {
        let mut results = Vec::new();

        // Check date sequence consistency
        results.push(self.check_date_sequence_consistency(poam_items)?);

        // Check status-date alignment
        results.push(self.check_status_date_alignment(poam_items)?);

        Ok(results)
    }

    /// Check date sequence consistency
    fn check_date_sequence_consistency(&self, poam_items: &[PoamItem]) -> Result<TimelineConsistencyResult> {
        let mut passed_count = 0;
        let mut failed_details = Vec::new();

        for item in poam_items {
            let mut item_passed = true;

            // Check if actual completion date is after scheduled date (if both present)
            if let (Some(scheduled_str), Some(actual_str)) = (&item.scheduled_completion_date, &item.actual_completion_date) {
                if let (Ok(scheduled), Ok(actual)) = (
                    self.parse_date_string(scheduled_str),
                    self.parse_date_string(actual_str)
                ) {
                    // Allow some variance for early completion
                    let days_early = (scheduled - actual).num_days();
                    if days_early > self.max_date_variance_days {
                        item_passed = false;
                        failed_details.push(TimelineIssue {
                            item_uuid: item.uuid.clone(),
                            issue_description: format!(
                                "Actual completion date is {} days before scheduled date",
                                days_early
                            ),
                            suggested_resolution: "Verify dates are correct or update scheduled date".to_string(),
                        });
                    }
                }
            }

            if item_passed {
                passed_count += 1;
            }
        }

        let failed_count = poam_items.len() - passed_count;
        let success_rate = if !poam_items.is_empty() {
            passed_count as f64 / poam_items.len() as f64
        } else {
            1.0
        };

        Ok(TimelineConsistencyResult {
            check_type: "date_sequence".to_string(),
            passed_items: passed_count,
            failed_items: failed_count,
            success_rate,
            failed_details,
        })
    }

    /// Check status-date alignment
    fn check_status_date_alignment(&self, poam_items: &[PoamItem]) -> Result<TimelineConsistencyResult> {
        let mut passed_count = 0;
        let mut failed_details = Vec::new();

        for item in poam_items {
            let mut item_passed = true;

            // Completed items should have actual completion date
            if item.status == "Completed" && item.actual_completion_date.is_none() {
                item_passed = false;
                failed_details.push(TimelineIssue {
                    item_uuid: item.uuid.clone(),
                    issue_description: "Item marked as Completed but missing actual completion date".to_string(),
                    suggested_resolution: "Add actual completion date or update status".to_string(),
                });
            }

            // Open/In Progress items should not have actual completion date
            if matches!(item.status.as_str(), "Open" | "In Progress") && item.actual_completion_date.is_some() {
                item_passed = false;
                failed_details.push(TimelineIssue {
                    item_uuid: item.uuid.clone(),
                    issue_description: format!(
                        "Item has status '{}' but has actual completion date",
                        item.status
                    ),
                    suggested_resolution: "Update status to Completed or remove actual completion date".to_string(),
                });
            }

            if item_passed {
                passed_count += 1;
            }
        }

        let failed_count = poam_items.len() - passed_count;
        let success_rate = if !poam_items.is_empty() {
            passed_count as f64 / poam_items.len() as f64
        } else {
            1.0
        };

        Ok(TimelineConsistencyResult {
            check_type: "status_date_alignment".to_string(),
            passed_items: passed_count,
            failed_items: failed_count,
            success_rate,
            failed_details,
        })
    }

    /// Generate consistency findings
    fn generate_consistency_findings(
        &self,
        cross_reference_results: &[CrossReferenceResult],
        timeline_results: &[TimelineConsistencyResult],
    ) -> Result<Vec<QualityFinding>> {
        let mut findings = Vec::new();

        // Generate findings for cross-reference issues
        for result in cross_reference_results {
            if result.success_rate < 0.95 && result.invalid_references > 0 {
                let severity = if result.success_rate < 0.8 {
                    QualitySeverity::High
                } else {
                    QualitySeverity::Medium
                };

                findings.push(QualityFinding {
                    id: Uuid::new_v4().to_string(),
                    severity,
                    category: QualityCategory::Consistency,
                    description: format!(
                        "Cross-reference validation '{}' failed for {} items ({:.1}% success rate)",
                        result.reference_type,
                        result.invalid_references,
                        result.success_rate * 100.0
                    ),
                    affected_items: result.invalid_details.clone(),
                    impact_assessment: "Cross-reference inconsistencies can affect data integrity and processing reliability".to_string(),
                    recommendation: format!(
                        "Review and correct the {} items with '{}' cross-reference issues",
                        result.invalid_references,
                        result.reference_type
                    ),
                    location: Some(format!("consistency_check.{}", result.reference_type)),
                    metadata: HashMap::new(),
                });
            }
        }

        // Generate findings for timeline issues
        for result in timeline_results {
            if result.success_rate < 0.9 && result.failed_items > 0 {
                let severity = if result.success_rate < 0.7 {
                    QualitySeverity::High
                } else {
                    QualitySeverity::Medium
                };

                let affected_items: Vec<String> = result.failed_details.iter()
                    .map(|issue| issue.item_uuid.clone())
                    .collect();

                findings.push(QualityFinding {
                    id: Uuid::new_v4().to_string(),
                    severity,
                    category: QualityCategory::Consistency,
                    description: format!(
                        "Timeline consistency check '{}' failed for {} items ({:.1}% success rate)",
                        result.check_type,
                        result.failed_items,
                        result.success_rate * 100.0
                    ),
                    affected_items,
                    impact_assessment: "Timeline inconsistencies can affect project tracking and compliance reporting".to_string(),
                    recommendation: format!(
                        "Review and correct timeline issues for {} items with '{}' problems",
                        result.failed_items,
                        result.check_type
                    ),
                    location: Some(format!("consistency_check.{}", result.check_type)),
                    metadata: HashMap::new(),
                });
            }
        }

        Ok(findings)
    }

    /// Calculate overall consistency score
    fn calculate_consistency_score(
        &self,
        cross_reference_results: &[CrossReferenceResult],
        timeline_results: &[TimelineConsistencyResult],
    ) -> f64 {
        let mut total_score = 0.0;
        let mut count = 0;

        // Include cross-reference scores
        for result in cross_reference_results {
            total_score += result.success_rate;
            count += 1;
        }

        // Include timeline scores
        for result in timeline_results {
            total_score += result.success_rate;
            count += 1;
        }

        if count > 0 {
            total_score / count as f64
        } else {
            1.0
        }
    }

    /// Parse date string (simplified version)
    fn parse_date_string(&self, date_str: &str) -> Result<DateTime<Utc>> {
        // Try parsing as ISO 8601 first
        if let Ok(dt) = DateTime::parse_from_rfc3339(date_str) {
            return Ok(dt.with_timezone(&Utc));
        }

        Err(Error::document_parsing(format!("Invalid date format: {}", date_str)))
    }
}

impl Default for ConsistencyChecker {
    fn default() -> Self {
        Self::new()
    }
}
