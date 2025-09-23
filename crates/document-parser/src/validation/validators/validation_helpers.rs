// Modified: 2025-09-22

//! Validation helper functions and utilities
//!
//! This module contains helper functions for pattern matching, type detection,
//! and other validation utilities used by the validators.

use serde_json::Value;
use std::collections::HashMap;

/// Helper functions for validation operations
pub struct ValidationHelpers;

impl ValidationHelpers {
    /// Detect the actual data type of column values
    pub fn detect_data_type(values: &[Value]) -> String {
        let mut type_counts = HashMap::new();
        
        for value in values.iter().filter(|v| !v.is_null()) {
            let detected_type = match value {
                Value::String(s) => {
                    if Self::is_valid_date(s) {
                        "Date"
                    } else if Self::is_valid_email(s) {
                        "Email"
                    } else if Self::is_valid_url(s) {
                        "URL"
                    } else if Self::is_valid_ip_address(s) {
                        "IP Address"
                    } else if Self::is_valid_uuid(s) {
                        "UUID"
                    } else if s.parse::<i64>().is_ok() {
                        "Integer"
                    } else if s.parse::<f64>().is_ok() {
                        "Float"
                    } else {
                        "String"
                    }
                }
                Value::Number(n) => {
                    if n.is_i64() {
                        "Integer"
                    } else {
                        "Float"
                    }
                }
                Value::Bool(_) => "Boolean",
                Value::Array(_) => "Array",
                Value::Object(_) => "Object",
                Value::Null => "Null",
            };
            
            *type_counts.entry(detected_type).or_insert(0) += 1;
        }
        
        // Return the most common type
        type_counts.into_iter()
            .max_by_key(|(_, count)| *count)
            .map(|(type_name, _)| type_name.to_string())
            .unwrap_or_else(|| "Unknown".to_string())
    }

    /// Check if a string is a valid date
    pub fn is_valid_date(s: &str) -> bool {
        // Simple date validation - can be enhanced with proper date parsing
        let date_patterns = [
            r"^\d{4}-\d{2}-\d{2}$",           // YYYY-MM-DD
            r"^\d{2}/\d{2}/\d{4}$",           // MM/DD/YYYY
            r"^\d{2}-\d{2}-\d{4}$",           // MM-DD-YYYY
            r"^\d{1,2}/\d{1,2}/\d{4}$",       // M/D/YYYY
        ];
        
        date_patterns.iter().any(|pattern| {
            regex::Regex::new(pattern).map(|re| re.is_match(s)).unwrap_or(false)
        })
    }

    /// Check if a string is a valid email
    pub fn is_valid_email(s: &str) -> bool {
        // Simple email validation
        let email_pattern = r"^[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}$";
        regex::Regex::new(email_pattern).map(|re| re.is_match(s)).unwrap_or(false)
    }

    /// Check if a string is a valid URL
    pub fn is_valid_url(s: &str) -> bool {
        // Simple URL validation
        s.starts_with("http://") || s.starts_with("https://") || s.starts_with("ftp://")
    }

    /// Check if a string is a valid IP address
    pub fn is_valid_ip_address(s: &str) -> bool {
        // Simple IPv4 validation
        let ip_pattern = r"^(?:(?:25[0-5]|2[0-4][0-9]|[01]?[0-9][0-9]?)\.){3}(?:25[0-5]|2[0-4][0-9]|[01]?[0-9][0-9]?)$";
        regex::Regex::new(ip_pattern).map(|re| re.is_match(s)).unwrap_or(false)
    }

    /// Check if a string is a valid UUID
    pub fn is_valid_uuid(s: &str) -> bool {
        // Simple UUID validation
        let uuid_pattern = r"^[0-9a-fA-F]{8}-[0-9a-fA-F]{4}-[0-9a-fA-F]{4}-[0-9a-fA-F]{4}-[0-9a-fA-F]{12}$";
        regex::Regex::new(uuid_pattern).map(|re| re.is_match(s)).unwrap_or(false)
    }

    /// Check if a string matches a custom pattern
    pub fn matches_pattern(s: &str, pattern: &str) -> bool {
        regex::Regex::new(pattern).map(|re| re.is_match(s)).unwrap_or(false)
    }

    /// Calculate validity rate for a set of values
    pub fn calculate_validity_rate(valid_count: usize, total_count: usize) -> f64 {
        if total_count > 0 {
            valid_count as f64 / total_count as f64
        } else {
            1.0
        }
    }

    /// Determine validation status based on validity rate
    pub fn determine_status_from_rate(validity_rate: f64, valid_threshold: f64, invalid_threshold: f64) -> super::super::types::ValidationStatus {
        if validity_rate >= valid_threshold {
            super::super::types::ValidationStatus::Valid
        } else if validity_rate >= invalid_threshold {
            super::super::types::ValidationStatus::Invalid
        } else {
            super::super::types::ValidationStatus::TypeMismatch
        }
    }

    /// Check if a value is considered empty or null
    pub fn is_empty_value(value: &Value) -> bool {
        match value {
            Value::Null => true,
            Value::String(s) => s.trim().is_empty(),
            Value::Array(arr) => arr.is_empty(),
            Value::Object(obj) => obj.is_empty(),
            _ => false,
        }
    }

    /// Count non-empty values in a slice
    pub fn count_non_empty_values(values: &[Value]) -> usize {
        values.iter().filter(|v| !Self::is_empty_value(v)).count()
    }

    /// Extract sample invalid values with size limit
    pub fn extract_sample_invalid_values(
        values: &[Value],
        is_valid_fn: impl Fn(&Value) -> bool,
        max_samples: usize,
    ) -> Vec<String> {
        let mut invalid_values = Vec::new();
        
        for value in values {
            if !Self::is_empty_value(value) && !is_valid_fn(value) {
                invalid_values.push(value.to_string());
                if invalid_values.len() >= max_samples {
                    break;
                }
            }
        }
        
        invalid_values
    }

    /// Validate a value against multiple patterns
    pub fn validate_against_patterns(s: &str, patterns: &[String]) -> bool {
        patterns.iter().any(|pattern| Self::matches_pattern(s, pattern))
    }

    /// Check if a string represents a boolean value
    pub fn is_boolean_string(s: &str) -> bool {
        let s_lower = s.to_lowercase();
        matches!(s_lower.as_str(), "true" | "false" | "yes" | "no" | "1" | "0" | "on" | "off")
    }

    /// Check if a number represents a boolean value (0 or 1)
    pub fn is_boolean_number(value: &Value) -> bool {
        match value {
            Value::Number(n) => {
                n.is_i64() && (n.as_i64() == Some(0) || n.as_i64() == Some(1))
            }
            _ => false,
        }
    }

    /// Parse a string as an integer
    pub fn parse_integer(s: &str) -> Option<i64> {
        s.parse::<i64>().ok()
    }

    /// Parse a string as a float
    pub fn parse_float(s: &str) -> Option<f64> {
        s.parse::<f64>().ok()
    }

    /// Check if a string can be parsed as a number
    pub fn is_numeric_string(s: &str) -> bool {
        Self::parse_float(s).is_some()
    }

    /// Check if a string can be parsed as an integer
    pub fn is_integer_string(s: &str) -> bool {
        Self::parse_integer(s).is_some()
    }

    /// Normalize a string for comparison (trim, lowercase)
    pub fn normalize_string(s: &str) -> String {
        s.trim().to_lowercase()
    }

    /// Check if a string is within a length range
    pub fn is_length_valid(s: &str, min_length: Option<usize>, max_length: Option<usize>) -> bool {
        let len = s.len();
        
        if let Some(min) = min_length {
            if len < min {
                return false;
            }
        }
        
        if let Some(max) = max_length {
            if len > max {
                return false;
            }
        }
        
        true
    }

    /// Check if a number is within a range
    pub fn is_number_in_range(value: f64, min_value: Option<f64>, max_value: Option<f64>) -> bool {
        if let Some(min) = min_value {
            if value < min {
                return false;
            }
        }
        
        if let Some(max) = max_value {
            if value > max {
                return false;
            }
        }
        
        true
    }

    /// Generate a validation message with statistics
    pub fn generate_validation_message(
        validation_type: &str,
        valid_count: usize,
        total_count: usize,
        validity_rate: f64,
    ) -> String {
        format!(
            "{} validation: {}/{} values are valid ({:.1}%)",
            validation_type,
            valid_count,
            total_count,
            validity_rate * 100.0
        )
    }

    /// Check if a date string matches common formats
    pub fn is_common_date_format(s: &str) -> bool {
        let common_formats = [
            r"^\d{4}-\d{2}-\d{2}$",                    // ISO 8601: YYYY-MM-DD
            r"^\d{2}/\d{2}/\d{4}$",                    // US: MM/DD/YYYY
            r"^\d{2}-\d{2}-\d{4}$",                    // US: MM-DD-YYYY
            r"^\d{1,2}/\d{1,2}/\d{4}$",                // Flexible: M/D/YYYY
            r"^\d{4}/\d{2}/\d{2}$",                    // Alternative: YYYY/MM/DD
            r"^\d{2}\.\d{2}\.\d{4}$",                  // European: DD.MM.YYYY
            r"^\d{1,2}\.\d{1,2}\.\d{4}$",              // Flexible European: D.M.YYYY
        ];
        
        common_formats.iter().any(|pattern| Self::matches_pattern(s, pattern))
    }

    /// Check if a time string matches common formats
    pub fn is_common_time_format(s: &str) -> bool {
        let time_formats = [
            r"^\d{2}:\d{2}$",                          // HH:MM
            r"^\d{2}:\d{2}:\d{2}$",                    // HH:MM:SS
            r"^\d{1,2}:\d{2}$",                        // H:MM
            r"^\d{1,2}:\d{2}:\d{2}$",                  // H:MM:SS
            r"^\d{2}:\d{2}:\d{2}\.\d{3}$",             // HH:MM:SS.mmm
        ];
        
        time_formats.iter().any(|pattern| Self::matches_pattern(s, pattern))
    }

    /// Check if a datetime string matches common formats
    pub fn is_common_datetime_format(s: &str) -> bool {
        // Check for date + time combinations
        if let Some(space_pos) = s.find(' ') {
            let date_part = &s[..space_pos];
            let time_part = &s[space_pos + 1..];
            return Self::is_common_date_format(date_part) && Self::is_common_time_format(time_part);
        }
        
        // Check for ISO 8601 datetime formats
        let iso_datetime_formats = [
            r"^\d{4}-\d{2}-\d{2}T\d{2}:\d{2}:\d{2}$",           // YYYY-MM-DDTHH:MM:SS
            r"^\d{4}-\d{2}-\d{2}T\d{2}:\d{2}:\d{2}Z$",          // YYYY-MM-DDTHH:MM:SSZ
            r"^\d{4}-\d{2}-\d{2}T\d{2}:\d{2}:\d{2}\.\d{3}Z$",   // YYYY-MM-DDTHH:MM:SS.mmmZ
        ];
        
        iso_datetime_formats.iter().any(|pattern| Self::matches_pattern(s, pattern))
    }
}
