//! Main validator implementations

use std::collections::HashMap;
use std::time::{Duration, Instant};
use serde_json::Value;
use tracing::{debug, info, warn};
use crate::{Error, Result};
use crate::mapping::MappingConfiguration;
use super::types::*;
use super::rules::*;

/// Column validator for validating individual columns
#[derive(Debug, Clone)]
pub struct ColumnValidator {
    /// Mapping configuration for validation rules
    mapping_config: MappingConfiguration,
    /// Minimum quality threshold for acceptance
    min_quality_threshold: f64,
    /// Performance target in milliseconds
    performance_target_ms: u64,
    /// Custom validation functions
    custom_validators: HashMap<String, fn(&[Value]) -> Result<(ValidationStatus, String)>>,
}

/// Document validator for comprehensive document validation
#[derive(Debug, Clone)]
pub struct DocumentValidator {
    /// Validation rules loaded from configuration
    rules: HashMap<String, Vec<ValidationRule>>,
    /// Column validator for field-level validation
    column_validator: Option<ColumnValidator>,
    /// Minimum quality threshold for document acceptance
    min_quality_threshold: f64,
    /// Performance metrics
    performance_metrics: HashMap<String, Duration>,
}

impl ColumnValidator {
    /// Create a new column validator with mapping configuration
    pub fn new(mapping_config: MappingConfiguration) -> Self {
        Self {
            mapping_config,
            min_quality_threshold: 0.8,
            performance_target_ms: 50,
            custom_validators: HashMap::new(),
        }
    }

    /// Validate a column against expected data type and constraints
    pub fn validate_column(
        &self,
        field_id: &str,
        source_column: &str,
        column_data: &[Value],
        expected_type: &DataType,
    ) -> Result<ColumnValidationResult> {
        let start_time = Instant::now();
        
        debug!("Validating column '{}' for field '{}'", source_column, field_id);

        let (status, message, sample_invalid_values) = match expected_type {
            DataType::String => self.validate_string_values(column_data)?,
            DataType::Integer => self.validate_integer_values(column_data)?,
            DataType::Float => self.validate_float_values(column_data)?,
            DataType::Boolean => self.validate_boolean_values(column_data)?,
            DataType::Date => self.validate_date_values(column_data)?,
            DataType::DateTime => self.validate_datetime_values(column_data)?,
            DataType::Email => self.validate_email_values(column_data)?,
            DataType::Url => self.validate_url_values(column_data)?,
            DataType::IpAddress => self.validate_ip_address_values(column_data)?,
            DataType::Uuid => self.validate_uuid_values(column_data)?,
            DataType::Any => (ValidationStatus::Valid, "No validation required".to_string(), Vec::new()),
            _ => (ValidationStatus::Valid, "Validation not implemented for this type".to_string(), Vec::new()),
        };

        let validation_time = start_time.elapsed();
        let severity = if status == ValidationStatus::Valid {
            ValidationSeverity::Info
        } else {
            ValidationSeverity::Warning
        };

        Ok(ColumnValidationResult {
            field_id: field_id.to_string(),
            source_column: source_column.to_string(),
            passed: status == ValidationStatus::Valid,
            status,
            severity,
            message,
            expected_type: Some(format!("{:?}", expected_type)),
            actual_type: Some(self.detect_data_type(column_data)),
            sample_invalid_values,
            validation_time_us: validation_time.as_micros() as u64,
        })
    }

    /// Validate string values
    pub fn validate_string_values(&self, values: &[Value]) -> Result<(ValidationStatus, String, Vec<String>)> {
        let mut invalid_values = Vec::new();
        let mut valid_count = 0;

        for value in values {
            match value {
                Value::String(_) => valid_count += 1,
                Value::Null => {}, // Null values are acceptable for optional fields
                _ => {
                    invalid_values.push(value.to_string());
                    if invalid_values.len() >= 5 { // Limit sample size
                        break;
                    }
                }
            }
        }

        let total_non_null = values.iter().filter(|v| !v.is_null()).count();
        let validity_rate = if total_non_null > 0 {
            valid_count as f64 / total_non_null as f64
        } else {
            1.0
        };

        let status = if validity_rate >= 0.9 {
            ValidationStatus::Valid
        } else if validity_rate >= 0.7 {
            ValidationStatus::Invalid
        } else {
            ValidationStatus::TypeMismatch
        };

        let message = format!(
            "String validation: {}/{} values are valid strings ({:.1}%)",
            valid_count, total_non_null, validity_rate * 100.0
        );

        Ok((status, message, invalid_values))
    }

    /// Validate integer values
    pub fn validate_integer_values(&self, values: &[Value]) -> Result<(ValidationStatus, String, Vec<String>)> {
        let mut invalid_values = Vec::new();
        let mut valid_count = 0;

        for value in values {
            match value {
                Value::Number(n) if n.is_i64() => valid_count += 1,
                Value::String(s) => {
                    if s.parse::<i64>().is_ok() {
                        valid_count += 1;
                    } else {
                        invalid_values.push(s.clone());
                        if invalid_values.len() >= 5 {
                            break;
                        }
                    }
                }
                Value::Null => {}, // Null values are acceptable
                _ => {
                    invalid_values.push(value.to_string());
                    if invalid_values.len() >= 5 {
                        break;
                    }
                }
            }
        }

        let total_non_null = values.iter().filter(|v| !v.is_null()).count();
        let validity_rate = if total_non_null > 0 {
            valid_count as f64 / total_non_null as f64
        } else {
            1.0
        };

        let status = if validity_rate >= 0.9 {
            ValidationStatus::Valid
        } else {
            ValidationStatus::TypeMismatch
        };

        let message = format!(
            "Integer validation: {}/{} values are valid integers ({:.1}%)",
            valid_count, total_non_null, validity_rate * 100.0
        );

        Ok((status, message, invalid_values))
    }

    /// Validate float values
    pub fn validate_float_values(&self, values: &[Value]) -> Result<(ValidationStatus, String, Vec<String>)> {
        let mut invalid_values = Vec::new();
        let mut valid_count = 0;

        for value in values {
            match value {
                Value::Number(_) => valid_count += 1,
                Value::String(s) => {
                    if s.parse::<f64>().is_ok() {
                        valid_count += 1;
                    } else {
                        invalid_values.push(s.clone());
                        if invalid_values.len() >= 5 {
                            break;
                        }
                    }
                }
                Value::Null => {},
                _ => {
                    invalid_values.push(value.to_string());
                    if invalid_values.len() >= 5 {
                        break;
                    }
                }
            }
        }

        let total_non_null = values.iter().filter(|v| !v.is_null()).count();
        let validity_rate = if total_non_null > 0 {
            valid_count as f64 / total_non_null as f64
        } else {
            1.0
        };

        let status = if validity_rate >= 0.9 {
            ValidationStatus::Valid
        } else {
            ValidationStatus::TypeMismatch
        };

        let message = format!(
            "Float validation: {}/{} values are valid numbers ({:.1}%)",
            valid_count, total_non_null, validity_rate * 100.0
        );

        Ok((status, message, invalid_values))
    }

    /// Validate boolean values
    pub fn validate_boolean_values(&self, values: &[Value]) -> Result<(ValidationStatus, String, Vec<String>)> {
        let mut invalid_values = Vec::new();
        let mut valid_count = 0;

        for value in values {
            match value {
                Value::Bool(_) => valid_count += 1,
                Value::String(s) => {
                    let s_lower = s.to_lowercase();
                    if matches!(s_lower.as_str(), "true" | "false" | "yes" | "no" | "1" | "0" | "on" | "off") {
                        valid_count += 1;
                    } else {
                        invalid_values.push(s.clone());
                        if invalid_values.len() >= 5 {
                            break;
                        }
                    }
                }
                Value::Number(n) => {
                    if n.is_i64() && (n.as_i64() == Some(0) || n.as_i64() == Some(1)) {
                        valid_count += 1;
                    } else {
                        invalid_values.push(n.to_string());
                        if invalid_values.len() >= 5 {
                            break;
                        }
                    }
                }
                Value::Null => {},
                _ => {
                    invalid_values.push(value.to_string());
                    if invalid_values.len() >= 5 {
                        break;
                    }
                }
            }
        }

        let total_non_null = values.iter().filter(|v| !v.is_null()).count();
        let validity_rate = if total_non_null > 0 {
            valid_count as f64 / total_non_null as f64
        } else {
            1.0
        };

        let status = if validity_rate >= 0.9 {
            ValidationStatus::Valid
        } else {
            ValidationStatus::Invalid
        };

        let message = format!(
            "Boolean validation: {}/{} values are valid booleans ({:.1}%)",
            valid_count, total_non_null, validity_rate * 100.0
        );

        Ok((status, message, invalid_values))
    }

    /// Validate date values
    pub fn validate_date_values(&self, values: &[Value]) -> Result<(ValidationStatus, String, Vec<String>)> {
        let mut invalid_values = Vec::new();
        let mut valid_count = 0;

        for value in values {
            match value {
                Value::String(s) => {
                    if self.is_valid_date(s) {
                        valid_count += 1;
                    } else {
                        invalid_values.push(s.clone());
                        if invalid_values.len() >= 5 {
                            break;
                        }
                    }
                }
                Value::Null => {},
                _ => {
                    invalid_values.push(value.to_string());
                    if invalid_values.len() >= 5 {
                        break;
                    }
                }
            }
        }

        let total_non_null = values.iter().filter(|v| !v.is_null()).count();
        let validity_rate = if total_non_null > 0 {
            valid_count as f64 / total_non_null as f64
        } else {
            1.0
        };

        let status = if validity_rate >= 0.9 {
            ValidationStatus::Valid
        } else {
            ValidationStatus::Invalid
        };

        let message = format!(
            "Date validation: {}/{} values are valid dates ({:.1}%)",
            valid_count, total_non_null, validity_rate * 100.0
        );

        Ok((status, message, invalid_values))
    }

    /// Validate datetime values
    pub fn validate_datetime_values(&self, values: &[Value]) -> Result<(ValidationStatus, String, Vec<String>)> {
        // For now, use the same logic as date validation
        // TODO: Implement proper datetime validation with time components
        self.validate_date_values(values)
    }

    /// Validate email values
    pub fn validate_email_values(&self, values: &[Value]) -> Result<(ValidationStatus, String, Vec<String>)> {
        let mut invalid_values = Vec::new();
        let mut valid_count = 0;

        for value in values {
            match value {
                Value::String(s) => {
                    if self.is_valid_email(s) {
                        valid_count += 1;
                    } else {
                        invalid_values.push(s.clone());
                        if invalid_values.len() >= 5 {
                            break;
                        }
                    }
                }
                Value::Null => {},
                _ => {
                    invalid_values.push(value.to_string());
                    if invalid_values.len() >= 5 {
                        break;
                    }
                }
            }
        }

        let total_non_null = values.iter().filter(|v| !v.is_null()).count();
        let validity_rate = if total_non_null > 0 {
            valid_count as f64 / total_non_null as f64
        } else {
            1.0
        };

        let status = if validity_rate >= 0.9 {
            ValidationStatus::Valid
        } else {
            ValidationStatus::Invalid
        };

        let message = format!(
            "Email validation: {}/{} values are valid emails ({:.1}%)",
            valid_count, total_non_null, validity_rate * 100.0
        );

        Ok((status, message, invalid_values))
    }

    /// Validate URL values
    pub fn validate_url_values(&self, values: &[Value]) -> Result<(ValidationStatus, String, Vec<String>)> {
        let mut invalid_values = Vec::new();
        let mut valid_count = 0;

        for value in values {
            match value {
                Value::String(s) => {
                    if self.is_valid_url(s) {
                        valid_count += 1;
                    } else {
                        invalid_values.push(s.clone());
                        if invalid_values.len() >= 5 {
                            break;
                        }
                    }
                }
                Value::Null => {},
                _ => {
                    invalid_values.push(value.to_string());
                    if invalid_values.len() >= 5 {
                        break;
                    }
                }
            }
        }

        let total_non_null = values.iter().filter(|v| !v.is_null()).count();
        let validity_rate = if total_non_null > 0 {
            valid_count as f64 / total_non_null as f64
        } else {
            1.0
        };

        let status = if validity_rate >= 0.9 {
            ValidationStatus::Valid
        } else {
            ValidationStatus::Invalid
        };

        let message = format!(
            "URL validation: {}/{} values are valid URLs ({:.1}%)",
            valid_count, total_non_null, validity_rate * 100.0
        );

        Ok((status, message, invalid_values))
    }

    /// Validate IP address values
    pub fn validate_ip_address_values(&self, values: &[Value]) -> Result<(ValidationStatus, String, Vec<String>)> {
        let mut invalid_values = Vec::new();
        let mut valid_count = 0;

        for value in values {
            match value {
                Value::String(s) => {
                    if self.is_valid_ip_address(s) {
                        valid_count += 1;
                    } else {
                        invalid_values.push(s.clone());
                        if invalid_values.len() >= 5 {
                            break;
                        }
                    }
                }
                Value::Null => {},
                _ => {
                    invalid_values.push(value.to_string());
                    if invalid_values.len() >= 5 {
                        break;
                    }
                }
            }
        }

        let total_non_null = values.iter().filter(|v| !v.is_null()).count();
        let validity_rate = if total_non_null > 0 {
            valid_count as f64 / total_non_null as f64
        } else {
            1.0
        };

        let status = if validity_rate >= 0.9 {
            ValidationStatus::Valid
        } else {
            ValidationStatus::Invalid
        };

        let message = format!(
            "IP address validation: {}/{} values are valid IP addresses ({:.1}%)",
            valid_count, total_non_null, validity_rate * 100.0
        );

        Ok((status, message, invalid_values))
    }

    /// Validate UUID values
    pub fn validate_uuid_values(&self, values: &[Value]) -> Result<(ValidationStatus, String, Vec<String>)> {
        let mut invalid_values = Vec::new();
        let mut valid_count = 0;

        for value in values {
            match value {
                Value::String(s) => {
                    if self.is_valid_uuid(s) {
                        valid_count += 1;
                    } else {
                        invalid_values.push(s.clone());
                        if invalid_values.len() >= 5 {
                            break;
                        }
                    }
                }
                Value::Null => {},
                _ => {
                    invalid_values.push(value.to_string());
                    if invalid_values.len() >= 5 {
                        break;
                    }
                }
            }
        }

        let total_non_null = values.iter().filter(|v| !v.is_null()).count();
        let validity_rate = if total_non_null > 0 {
            valid_count as f64 / total_non_null as f64
        } else {
            1.0
        };

        let status = if validity_rate >= 0.9 {
            ValidationStatus::Valid
        } else {
            ValidationStatus::Invalid
        };

        let message = format!(
            "UUID validation: {}/{} values are valid UUIDs ({:.1}%)",
            valid_count, total_non_null, validity_rate * 100.0
        );

        Ok((status, message, invalid_values))
    }

    /// Detect the actual data type of column values
    fn detect_data_type(&self, values: &[Value]) -> String {
        let mut type_counts = HashMap::new();
        
        for value in values.iter().filter(|v| !v.is_null()) {
            let detected_type = match value {
                Value::String(s) => {
                    if self.is_valid_date(s) {
                        "Date"
                    } else if self.is_valid_email(s) {
                        "Email"
                    } else if self.is_valid_url(s) {
                        "URL"
                    } else if self.is_valid_ip_address(s) {
                        "IP Address"
                    } else if self.is_valid_uuid(s) {
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
    fn is_valid_date(&self, s: &str) -> bool {
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
    fn is_valid_email(&self, s: &str) -> bool {
        // Simple email validation
        let email_pattern = r"^[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}$";
        regex::Regex::new(email_pattern).map(|re| re.is_match(s)).unwrap_or(false)
    }

    /// Check if a string is a valid URL
    fn is_valid_url(&self, s: &str) -> bool {
        // Simple URL validation
        s.starts_with("http://") || s.starts_with("https://") || s.starts_with("ftp://")
    }

    /// Check if a string is a valid IP address
    fn is_valid_ip_address(&self, s: &str) -> bool {
        // Simple IPv4 validation
        let ip_pattern = r"^(?:(?:25[0-5]|2[0-4][0-9]|[01]?[0-9][0-9]?)\.){3}(?:25[0-5]|2[0-4][0-9]|[01]?[0-9][0-9]?)$";
        regex::Regex::new(ip_pattern).map(|re| re.is_match(s)).unwrap_or(false)
    }

    /// Check if a string is a valid UUID
    fn is_valid_uuid(&self, s: &str) -> bool {
        // Simple UUID validation
        let uuid_pattern = r"^[0-9a-fA-F]{8}-[0-9a-fA-F]{4}-[0-9a-fA-F]{4}-[0-9a-fA-F]{4}-[0-9a-fA-F]{12}$";
        regex::Regex::new(uuid_pattern).map(|re| re.is_match(s)).unwrap_or(false)
    }
}

impl DocumentValidator {
    /// Create a new document validator
    #[must_use]
    pub fn new() -> Self {
        Self {
            rules: HashMap::new(),
            column_validator: None,
            min_quality_threshold: 0.8,
            performance_metrics: HashMap::new(),
        }
    }

    /// Create a document validator with column validator
    pub fn with_column_validator(column_validator: ColumnValidator) -> Self {
        Self {
            rules: HashMap::new(),
            column_validator: Some(column_validator),
            min_quality_threshold: 0.8,
            performance_metrics: HashMap::new(),
        }
    }

    /// Check if document meets quality threshold
    pub fn meets_quality_threshold(&self, metrics: &super::QualityMetrics) -> bool {
        metrics.overall_quality_score >= self.min_quality_threshold
    }

    /// Get performance metrics
    pub fn get_performance_metrics(&self) -> &HashMap<String, Duration> {
        &self.performance_metrics
    }
}

impl Default for DocumentValidator {
    fn default() -> Self {
        Self::new()
    }
}
