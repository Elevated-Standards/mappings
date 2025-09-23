// Modified: 2025-09-22

//! Field-level validation implementation
//!
//! This module contains the ColumnValidator implementation for validating
//! individual columns against expected data types and constraints.

use super::types::*;
use super::validation_helpers::ValidationHelpers;
use crate::{Result};
use crate::mapping::MappingConfiguration;
use super::super::types::*;
use super::super::rules::DataType;
use serde_json::Value;
use std::collections::HashMap;
use std::time::Instant;
use tracing::debug;

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

    /// Create a column validator with custom configuration
    pub fn with_config(mapping_config: MappingConfiguration, config: ColumnValidationConfig) -> Self {
        Self {
            mapping_config,
            min_quality_threshold: config.min_quality_threshold,
            performance_target_ms: config.performance_target_ms,
            custom_validators: HashMap::new(),
        }
    }

    /// Add a custom validator function
    pub fn add_custom_validator(
        &mut self,
        name: String,
        validator: fn(&[Value]) -> Result<(ValidationStatus, String)>,
    ) {
        self.custom_validators.insert(name, validator);
    }

    /// Remove a custom validator
    pub fn remove_custom_validator(&mut self, name: &str) -> bool {
        self.custom_validators.remove(name).is_some()
    }

    /// Get the minimum quality threshold
    pub fn get_min_quality_threshold(&self) -> f64 {
        self.min_quality_threshold
    }

    /// Set the minimum quality threshold
    pub fn set_min_quality_threshold(&mut self, threshold: f64) {
        self.min_quality_threshold = threshold.clamp(0.0, 1.0);
    }

    /// Get the performance target in milliseconds
    pub fn get_performance_target_ms(&self) -> u64 {
        self.performance_target_ms
    }

    /// Set the performance target in milliseconds
    pub fn set_performance_target_ms(&mut self, target_ms: u64) {
        self.performance_target_ms = target_ms;
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
            actual_type: Some(ValidationHelpers::detect_data_type(column_data)),
            sample_invalid_values,
            validation_time_us: validation_time.as_micros() as u64,
        })
    }

    /// Validate a column using a custom validator
    pub fn validate_column_custom(
        &self,
        field_id: &str,
        source_column: &str,
        column_data: &[Value],
        validator_name: &str,
    ) -> Result<ColumnValidationResult> {
        let start_time = Instant::now();

        let (status, message) = if let Some(validator) = self.custom_validators.get(validator_name) {
            validator(column_data)?
        } else {
            return Err(crate::Error::validation(format!("Custom validator '{}' not found", validator_name)));
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
            expected_type: Some("Custom".to_string()),
            actual_type: Some(ValidationHelpers::detect_data_type(column_data)),
            sample_invalid_values: Vec::new(),
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
                    if ValidationHelpers::is_valid_date(s) {
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
                    if ValidationHelpers::is_valid_email(s) {
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
                    if ValidationHelpers::is_valid_url(s) {
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
                    if ValidationHelpers::is_valid_ip_address(s) {
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
                    if ValidationHelpers::is_valid_uuid(s) {
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
}
