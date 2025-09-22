// Modified: 2025-09-22

//! Excel-specific validation logic
//!
//! This module provides validation and sanitization functionality for Excel data,
//! including security checks, data type validation, and content sanitization.

use crate::excel::types::*;
use serde_json::Value;
use regex::Regex;
use tracing::{debug, warn};
use std::collections::HashMap;

/// Excel data validator with security and quality checks
#[derive(Debug, Clone)]
pub struct ExcelValidator {
    /// Validation configuration
    config: ValidationConfig,
    /// Compiled regex patterns for injection detection
    injection_patterns: Vec<Regex>,
    /// Compiled regex patterns for suspicious content
    suspicious_patterns: Vec<Regex>,
}

impl ExcelValidator {
    /// Create a new Excel validator with the given configuration
    ///
    /// # Arguments
    ///
    /// * `config` - Validation configuration
    ///
    /// # Examples
    ///
    /// ```rust
    /// use document_parser::excel::{ExcelValidator, ValidationConfig};
    ///
    /// let config = ValidationConfig::strict();
    /// let validator = ExcelValidator::new(config);
    /// ```
    #[must_use]
    pub fn new(config: ValidationConfig) -> Self {
        let injection_patterns = Self::compile_injection_patterns();
        let suspicious_patterns = Self::compile_suspicious_patterns();

        Self {
            config,
            injection_patterns,
            suspicious_patterns,
        }
    }

    /// Create a new validator with default configuration
    #[must_use]
    pub fn default() -> Self {
        Self::new(ValidationConfig::default())
    }

    /// Validate and sanitize a single cell value
    ///
    /// # Arguments
    ///
    /// * `value` - The cell value to validate
    /// * `row` - Row coordinate (0-based)
    /// * `column` - Column coordinate (0-based)
    ///
    /// # Returns
    ///
    /// Returns a `CellValidationResult` with validation status and any issues found
    pub fn validate_cell(&self, value: &Value, row: usize, column: usize) -> CellValidationResult {
        let original_value = Some(value.clone());
        let mut issues = Vec::new();
        let mut sanitized_value = value.clone();
        let mut is_valid = true;
        let mut confidence = 1.0;

        // Validate based on value type
        match value {
            Value::String(s) => {
                let (string_issues, sanitized_string, string_confidence) = self.validate_string(s);
                issues.extend(string_issues);
                sanitized_value = Value::String(sanitized_string);
                confidence *= string_confidence;
                if !issues.is_empty() {
                    is_valid = false;
                }
            }
            Value::Number(n) => {
                let (number_issues, number_confidence) = self.validate_number(n.as_f64().unwrap_or(0.0));
                issues.extend(number_issues);
                confidence *= number_confidence;
                if !issues.is_empty() {
                    is_valid = false;
                }
            }
            Value::Bool(_) => {
                // Boolean values are generally safe
                confidence = 1.0;
            }
            Value::Null => {
                // Check if null is allowed in this context
                if self.config.strict_mode {
                    issues.push(ValidationIssue {
                        issue_type: ValidationIssueType::MissingValue,
                        severity: ValidationSeverity::Warning,
                        message: "Empty cell in strict mode".to_string(),
                        suggestion: Some("Consider providing a value".to_string()),
                        auto_fixed: false,
                    });
                    confidence = 0.8;
                }
            }
            Value::Array(_) | Value::Object(_) => {
                // Complex types are unusual in Excel cells
                issues.push(ValidationIssue {
                    issue_type: ValidationIssueType::InconsistentType,
                    severity: ValidationSeverity::Warning,
                    message: "Complex data type in Excel cell".to_string(),
                    suggestion: Some("Consider flattening the data structure".to_string()),
                    auto_fixed: false,
                });
                confidence = 0.6;
            }
        }

        CellValidationResult {
            row,
            column,
            original_value,
            sanitized_value: Some(sanitized_value),
            is_valid,
            issues,
            confidence,
        }
    }

    /// Validate a string value for security and quality issues
    fn validate_string(&self, s: &str) -> (Vec<ValidationIssue>, String, f64) {
        let mut issues = Vec::new();
        let mut sanitized = s.to_string();
        let mut confidence = 1.0;

        // Check string length
        if s.len() > self.config.max_string_length {
            issues.push(ValidationIssue {
                issue_type: ValidationIssueType::StringTooLong,
                severity: ValidationSeverity::Warning,
                message: format!("String length {} exceeds maximum {}", s.len(), self.config.max_string_length),
                suggestion: Some("Consider truncating the string".to_string()),
                auto_fixed: false,
            });
            
            if self.config.auto_fix {
                sanitized.truncate(self.config.max_string_length);
                issues.last_mut().unwrap().auto_fixed = true;
            }
            confidence *= 0.8;
        }

        // Check for injection patterns
        if self.config.check_injection {
            for pattern in &self.injection_patterns {
                if pattern.is_match(s) {
                    issues.push(ValidationIssue {
                        issue_type: ValidationIssueType::PotentialInjection,
                        severity: ValidationSeverity::Critical,
                        message: "Potential injection pattern detected".to_string(),
                        suggestion: Some("Remove or escape special characters".to_string()),
                        auto_fixed: false,
                    });
                    
                    if self.config.sanitize_content {
                        sanitized = self.sanitize_injection_patterns(&sanitized);
                        issues.last_mut().unwrap().auto_fixed = true;
                    }
                    confidence *= 0.3;
                    break;
                }
            }
        }

        // Check for suspicious patterns
        for pattern in &self.suspicious_patterns {
            if pattern.is_match(s) {
                issues.push(ValidationIssue {
                    issue_type: ValidationIssueType::SuspiciousPattern,
                    severity: ValidationSeverity::Warning,
                    message: "Suspicious content pattern detected".to_string(),
                    suggestion: Some("Review content for potential issues".to_string()),
                    auto_fixed: false,
                });
                confidence *= 0.7;
                break;
            }
        }

        // Check for invalid characters
        if s.chars().any(|c| c.is_control() && c != '\n' && c != '\r' && c != '\t') {
            issues.push(ValidationIssue {
                issue_type: ValidationIssueType::InvalidCharacters,
                severity: ValidationSeverity::Warning,
                message: "Invalid control characters detected".to_string(),
                suggestion: Some("Remove control characters".to_string()),
                auto_fixed: false,
            });
            
            if self.config.sanitize_content {
                sanitized = sanitized.chars()
                    .filter(|&c| !c.is_control() || c == '\n' || c == '\r' || c == '\t')
                    .collect();
                issues.last_mut().unwrap().auto_fixed = true;
            }
            confidence *= 0.9;
        }

        (issues, sanitized, confidence)
    }

    /// Validate a numeric value
    fn validate_number(&self, n: f64) -> (Vec<ValidationIssue>, f64) {
        let mut issues = Vec::new();
        let mut confidence = 1.0;

        // Check for special float values
        if n.is_infinite() {
            issues.push(ValidationIssue {
                issue_type: ValidationIssueType::OutOfRange,
                severity: ValidationSeverity::Error,
                message: "Infinite number value".to_string(),
                suggestion: Some("Replace with a finite number".to_string()),
                auto_fixed: false,
            });
            confidence = 0.0;
        } else if n.is_nan() {
            issues.push(ValidationIssue {
                issue_type: ValidationIssueType::InvalidNumber,
                severity: ValidationSeverity::Error,
                message: "Not-a-Number (NaN) value".to_string(),
                suggestion: Some("Replace with a valid number or null".to_string()),
                auto_fixed: false,
            });
            confidence = 0.0;
        }

        // Check for extremely large numbers that might cause issues
        if n.abs() > 1e15 {
            issues.push(ValidationIssue {
                issue_type: ValidationIssueType::OutOfRange,
                severity: ValidationSeverity::Warning,
                message: "Extremely large number value".to_string(),
                suggestion: Some("Consider using scientific notation or smaller values".to_string()),
                auto_fixed: false,
            });
            confidence *= 0.8;
        }

        (issues, confidence)
    }

    /// Compile regex patterns for injection detection
    fn compile_injection_patterns() -> Vec<Regex> {
        let patterns = vec![
            r"(?i)=\s*cmd\s*\|",           // Command injection
            r"(?i)=\s*system\s*\(",       // System calls
            r"(?i)=\s*exec\s*\(",         // Exec calls
            r"(?i)=\s*eval\s*\(",         // Eval calls
            r"(?i)javascript:",           // JavaScript URLs
            r"(?i)vbscript:",             // VBScript URLs
            r"(?i)data:.*base64",         // Base64 data URLs
            r"(?i)<script[^>]*>",         // Script tags
            r"(?i)on\w+\s*=",             // Event handlers
            r"(?i)=\s*[\x22\x27].*[\x22\x27].*\+",  // String concatenation
        ];

        patterns
            .into_iter()
            .filter_map(|pattern| {
                Regex::new(pattern).map_err(|e| {
                    warn!("Failed to compile injection pattern '{}': {}", pattern, e);
                    e
                }).ok()
            })
            .collect()
    }

    /// Compile regex patterns for suspicious content detection
    fn compile_suspicious_patterns() -> Vec<Regex> {
        let patterns = vec![
            r"(?i)password\s*[:=]\s*\S+",     // Password patterns
            r"(?i)api[_-]?key\s*[:=]\s*\S+",  // API key patterns
            r"(?i)secret\s*[:=]\s*\S+",       // Secret patterns
            r"(?i)token\s*[:=]\s*\S+",        // Token patterns
            r"\b\d{4}[-\s]?\d{4}[-\s]?\d{4}[-\s]?\d{4}\b", // Credit card patterns
            r"\b\d{3}-\d{2}-\d{4}\b",         // SSN patterns
        ];

        patterns
            .into_iter()
            .filter_map(|pattern| {
                Regex::new(pattern).map_err(|e| {
                    warn!("Failed to compile suspicious pattern '{}': {}", pattern, e);
                    e
                }).ok()
            })
            .collect()
    }

    /// Sanitize injection patterns from a string
    fn sanitize_injection_patterns(&self, s: &str) -> String {
        let mut sanitized = s.to_string();
        
        // Remove or escape dangerous patterns
        sanitized = sanitized.replace("=cmd", "cmd");
        sanitized = sanitized.replace("=system", "system");
        sanitized = sanitized.replace("=exec", "exec");
        sanitized = sanitized.replace("=eval", "eval");
        sanitized = sanitized.replace("javascript:", "");
        sanitized = sanitized.replace("vbscript:", "");
        
        // Remove script tags
        let script_regex = Regex::new(r"(?i)<script[^>]*>.*?</script>").unwrap();
        sanitized = script_regex.replace_all(&sanitized, "").to_string();
        
        sanitized
    }

    /// Generate a validation summary for a collection of cell results
    pub fn generate_summary(&self, results: &[CellValidationResult]) -> ValidationSummary {
        let total_cells = results.len();
        let valid_cells = results.iter().filter(|r| r.is_valid).count();
        let invalid_cells = total_cells - valid_cells;
        let sanitized_cells = results.iter()
            .filter(|r| r.original_value != r.sanitized_value)
            .count();
        
        let average_confidence = if total_cells > 0 {
            results.iter().map(|r| r.confidence).sum::<f64>() / total_cells as f64
        } else {
            1.0
        };

        let mut issue_breakdown = HashMap::new();
        let mut max_severity = None;

        for result in results {
            for issue in &result.issues {
                let count = issue_breakdown.entry(format!("{:?}", issue.issue_type)).or_insert(0);
                *count += 1;

                if max_severity.is_none() || issue.severity > *max_severity.as_ref().unwrap() {
                    max_severity = Some(issue.severity.clone());
                }
            }
        }

        ValidationSummary {
            total_cells,
            valid_cells,
            invalid_cells,
            sanitized_cells,
            average_confidence,
            issue_breakdown,
            max_severity,
        }
    }
}

impl Default for ExcelValidator {
    fn default() -> Self {
        Self::new(ValidationConfig::default())
    }
}
