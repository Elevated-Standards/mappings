// Modified: 2025-01-22

//! Field-level validation for POA&M documents
//!
//! This module provides validation for individual fields including
//! severity levels, status values, and field-specific business rules.

use super::types::{PoamSeverity, PoamStatus, FieldValidationResult, ValidationError, ValidationWarning};
use crate::validation::types::{ValidationStatus, ValidationSeverity};
use fedramp_core::Result;
use std::collections::HashMap;
use tracing::warn;

/// Severity validator
#[derive(Debug, Clone)]
pub struct SeverityValidator {
    /// Allowed severities
    pub allowed_severities: Vec<PoamSeverity>,
    /// Severity aliases
    pub severity_aliases: HashMap<String, PoamSeverity>,
}

/// Status validator
#[derive(Debug, Clone)]
pub struct StatusValidator {
    /// Allowed statuses
    pub allowed_statuses: Vec<PoamStatus>,
    /// Status aliases
    pub status_aliases: HashMap<String, PoamStatus>,
    /// Status transitions
    pub status_transitions: HashMap<PoamStatus, Vec<PoamStatus>>,
}

impl SeverityValidator {
    /// Create a new severity validator
    pub fn new(allowed_severities: &[PoamSeverity]) -> Self {
        let mut severity_aliases = HashMap::new();

        // Add standard aliases
        severity_aliases.insert("critical".to_string(), PoamSeverity::Critical);
        severity_aliases.insert("crit".to_string(), PoamSeverity::Critical);
        severity_aliases.insert("high".to_string(), PoamSeverity::High);
        severity_aliases.insert("h".to_string(), PoamSeverity::High);
        severity_aliases.insert("moderate".to_string(), PoamSeverity::Moderate);
        severity_aliases.insert("med".to_string(), PoamSeverity::Moderate);
        severity_aliases.insert("medium".to_string(), PoamSeverity::Moderate);
        severity_aliases.insert("mod".to_string(), PoamSeverity::Moderate);
        severity_aliases.insert("low".to_string(), PoamSeverity::Low);
        severity_aliases.insert("l".to_string(), PoamSeverity::Low);
        severity_aliases.insert("informational".to_string(), PoamSeverity::Informational);
        severity_aliases.insert("info".to_string(), PoamSeverity::Informational);
        severity_aliases.insert("i".to_string(), PoamSeverity::Informational);

        Self {
            allowed_severities: allowed_severities.to_vec(),
            severity_aliases,
        }
    }

    /// Validate a severity value
    pub fn validate_severity(&self, severity_str: &str) -> Result<FieldValidationResult> {
        let normalized = severity_str.trim().to_lowercase();
        
        // Check direct match first
        if let Some(severity) = self.severity_aliases.get(&normalized) {
            if self.allowed_severities.contains(severity) {
                return Ok(FieldValidationResult {
                    field_name: "severity".to_string(),
                    passed: true,
                    status: ValidationStatus::Valid,
                    error_message: None,
                    warning_message: None,
                    suggested_value: None,
                    confidence: 1.0,
                });
            } else {
                return Ok(FieldValidationResult {
                    field_name: "severity".to_string(),
                    passed: false,
                    status: ValidationStatus::Invalid,
                    error_message: Some(format!("Severity '{}' is not allowed", severity_str)),
                    warning_message: None,
                    suggested_value: self.suggest_severity(&normalized),
                    confidence: 0.0,
                });
            }
        }

        // Try fuzzy matching for suggestions
        let suggestion = self.suggest_severity(&normalized);
        
        Ok(FieldValidationResult {
            field_name: "severity".to_string(),
            passed: false,
            status: ValidationStatus::Invalid,
            error_message: Some(format!("Invalid severity value: '{}'", severity_str)),
            warning_message: None,
            suggested_value: suggestion,
            confidence: 0.0,
        })
    }

    /// Suggest a severity value based on fuzzy matching
    fn suggest_severity(&self, input: &str) -> Option<String> {
        let mut best_match = None;
        let mut best_distance = usize::MAX;

        for (alias, severity) in &self.severity_aliases {
            if self.allowed_severities.contains(severity) {
                let distance = self.levenshtein_distance(input, alias);
                if distance < best_distance && distance <= 2 {
                    best_distance = distance;
                    best_match = Some(alias.clone());
                }
            }
        }

        best_match
    }

    /// Calculate Levenshtein distance between two strings
    fn levenshtein_distance(&self, s1: &str, s2: &str) -> usize {
        let len1 = s1.len();
        let len2 = s2.len();
        let mut matrix = vec![vec![0; len2 + 1]; len1 + 1];

        for i in 0..=len1 {
            matrix[i][0] = i;
        }
        for j in 0..=len2 {
            matrix[0][j] = j;
        }

        for (i, c1) in s1.chars().enumerate() {
            for (j, c2) in s2.chars().enumerate() {
                let cost = if c1 == c2 { 0 } else { 1 };
                matrix[i + 1][j + 1] = std::cmp::min(
                    std::cmp::min(
                        matrix[i][j + 1] + 1,     // deletion
                        matrix[i + 1][j] + 1,     // insertion
                    ),
                    matrix[i][j] + cost,          // substitution
                );
            }
        }

        matrix[len1][len2]
    }

    /// Get all allowed severities
    pub fn get_allowed_severities(&self) -> &[PoamSeverity] {
        &self.allowed_severities
    }

    /// Check if a severity is allowed
    pub fn is_severity_allowed(&self, severity: &PoamSeverity) -> bool {
        self.allowed_severities.contains(severity)
    }
}

impl StatusValidator {
    /// Create a new status validator
    pub fn new(allowed_statuses: &[PoamStatus]) -> Self {
        let mut status_aliases = HashMap::new();

        // Add standard aliases
        status_aliases.insert("open".to_string(), PoamStatus::Open);
        status_aliases.insert("new".to_string(), PoamStatus::Open);
        status_aliases.insert("in_progress".to_string(), PoamStatus::InProgress);
        status_aliases.insert("inprogress".to_string(), PoamStatus::InProgress);
        status_aliases.insert("in progress".to_string(), PoamStatus::InProgress);
        status_aliases.insert("progress".to_string(), PoamStatus::InProgress);
        status_aliases.insert("working".to_string(), PoamStatus::InProgress);
        status_aliases.insert("completed".to_string(), PoamStatus::Completed);
        status_aliases.insert("complete".to_string(), PoamStatus::Completed);
        status_aliases.insert("done".to_string(), PoamStatus::Completed);
        status_aliases.insert("finished".to_string(), PoamStatus::Completed);
        status_aliases.insert("risk_accepted".to_string(), PoamStatus::RiskAccepted);
        status_aliases.insert("riskaccepted".to_string(), PoamStatus::RiskAccepted);
        status_aliases.insert("risk accepted".to_string(), PoamStatus::RiskAccepted);
        status_aliases.insert("accepted".to_string(), PoamStatus::RiskAccepted);
        status_aliases.insert("deferred".to_string(), PoamStatus::Deferred);
        status_aliases.insert("defer".to_string(), PoamStatus::Deferred);
        status_aliases.insert("postponed".to_string(), PoamStatus::Deferred);
        status_aliases.insert("rejected".to_string(), PoamStatus::Rejected);
        status_aliases.insert("reject".to_string(), PoamStatus::Rejected);
        status_aliases.insert("denied".to_string(), PoamStatus::Rejected);
        status_aliases.insert("closed".to_string(), PoamStatus::Closed);
        status_aliases.insert("close".to_string(), PoamStatus::Closed);

        // Define valid status transitions
        let mut status_transitions = HashMap::new();
        status_transitions.insert(PoamStatus::Open, vec![
            PoamStatus::InProgress,
            PoamStatus::RiskAccepted,
            PoamStatus::Deferred,
            PoamStatus::Rejected,
            PoamStatus::Closed,
        ]);
        status_transitions.insert(PoamStatus::InProgress, vec![
            PoamStatus::Completed,
            PoamStatus::RiskAccepted,
            PoamStatus::Deferred,
            PoamStatus::Rejected,
            PoamStatus::Open,
        ]);
        status_transitions.insert(PoamStatus::Completed, vec![
            PoamStatus::Closed,
            PoamStatus::InProgress, // Reopening
        ]);
        status_transitions.insert(PoamStatus::RiskAccepted, vec![
            PoamStatus::Closed,
            PoamStatus::Open, // Reopening
        ]);
        status_transitions.insert(PoamStatus::Deferred, vec![
            PoamStatus::Open,
            PoamStatus::InProgress,
            PoamStatus::Closed,
        ]);
        status_transitions.insert(PoamStatus::Rejected, vec![
            PoamStatus::Open, // Reopening
            PoamStatus::Closed,
        ]);
        status_transitions.insert(PoamStatus::Closed, vec![
            PoamStatus::Open, // Reopening
        ]);

        Self {
            allowed_statuses: allowed_statuses.to_vec(),
            status_aliases,
            status_transitions,
        }
    }

    /// Validate a status value
    pub fn validate_status(&self, status_str: &str) -> Result<FieldValidationResult> {
        let normalized = status_str.trim().to_lowercase();
        
        // Check direct match first
        if let Some(status) = self.status_aliases.get(&normalized) {
            if self.allowed_statuses.contains(status) {
                return Ok(FieldValidationResult {
                    field_name: "status".to_string(),
                    passed: true,
                    status: ValidationStatus::Valid,
                    error_message: None,
                    warning_message: None,
                    suggested_value: None,
                    confidence: 1.0,
                });
            } else {
                return Ok(FieldValidationResult {
                    field_name: "status".to_string(),
                    passed: false,
                    status: ValidationStatus::Invalid,
                    error_message: Some(format!("Status '{}' is not allowed", status_str)),
                    warning_message: None,
                    suggested_value: self.suggest_status(&normalized),
                    confidence: 0.0,
                });
            }
        }

        // Try fuzzy matching for suggestions
        let suggestion = self.suggest_status(&normalized);
        
        Ok(FieldValidationResult {
            field_name: "status".to_string(),
            passed: false,
            status: ValidationStatus::Invalid,
            error_message: Some(format!("Invalid status value: '{}'", status_str)),
            warning_message: None,
            suggested_value: suggestion,
            confidence: 0.0,
        })
    }

    /// Validate status transition
    pub fn validate_status_transition(&self, from_status: &PoamStatus, to_status: &PoamStatus) -> Result<bool> {
        if let Some(allowed_transitions) = self.status_transitions.get(from_status) {
            Ok(allowed_transitions.contains(to_status))
        } else {
            warn!("Unknown status transition from {:?}", from_status);
            Ok(false)
        }
    }

    /// Suggest a status value based on fuzzy matching
    fn suggest_status(&self, input: &str) -> Option<String> {
        let mut best_match = None;
        let mut best_distance = usize::MAX;

        for (alias, status) in &self.status_aliases {
            if self.allowed_statuses.contains(status) {
                let distance = self.levenshtein_distance(input, alias);
                if distance < best_distance && distance <= 2 {
                    best_distance = distance;
                    best_match = Some(alias.clone());
                }
            }
        }

        best_match
    }

    /// Calculate Levenshtein distance between two strings
    fn levenshtein_distance(&self, s1: &str, s2: &str) -> usize {
        let len1 = s1.len();
        let len2 = s2.len();
        let mut matrix = vec![vec![0; len2 + 1]; len1 + 1];

        for i in 0..=len1 {
            matrix[i][0] = i;
        }
        for j in 0..=len2 {
            matrix[0][j] = j;
        }

        for (i, c1) in s1.chars().enumerate() {
            for (j, c2) in s2.chars().enumerate() {
                let cost = if c1 == c2 { 0 } else { 1 };
                matrix[i + 1][j + 1] = std::cmp::min(
                    std::cmp::min(
                        matrix[i][j + 1] + 1,     // deletion
                        matrix[i + 1][j] + 1,     // insertion
                    ),
                    matrix[i][j] + cost,          // substitution
                );
            }
        }

        matrix[len1][len2]
    }

    /// Get all allowed statuses
    pub fn get_allowed_statuses(&self) -> &[PoamStatus] {
        &self.allowed_statuses
    }

    /// Check if a status is allowed
    pub fn is_status_allowed(&self, status: &PoamStatus) -> bool {
        self.allowed_statuses.contains(status)
    }
}
