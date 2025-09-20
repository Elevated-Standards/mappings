// Modified: 2025-01-20

//! Validation utilities for FedRAMP compliance data.
//!
//! This module provides validation functions and utilities for ensuring
//! data integrity and compliance with FedRAMP requirements.

use crate::error::Error;
use crate::types::Result;
use serde_json::Value;
use std::collections::HashMap;
use validator::{Validate, ValidationError, ValidationErrors};

/// Trait for validating FedRAMP compliance data
pub trait FedRampValidate {
    /// Validate the data for FedRAMP compliance
    fn validate_fedramp(&self) -> Result<()>;
}

/// Validation context for FedRAMP data
#[derive(Debug, Clone)]
pub struct ValidationContext {
    /// Additional validation rules
    pub rules: HashMap<String, Value>,
    /// Strict validation mode
    pub strict: bool,
}

impl Default for ValidationContext {
    fn default() -> Self {
        Self {
            rules: HashMap::new(),
            strict: false,
        }
    }
}

impl ValidationContext {
    /// Create a new validation context
    pub fn new() -> Self {
        Self::default()
    }

    /// Enable strict validation mode
    pub fn strict(mut self) -> Self {
        self.strict = true;
        self
    }

    /// Add a validation rule
    pub fn with_rule(mut self, key: String, value: Value) -> Self {
        self.rules.insert(key, value);
        self
    }
}

/// Validate a struct using the validator crate
pub fn validate_struct<T: Validate>(data: &T) -> Result<()> {
    data.validate()
        .map_err(|e| Error::validation(format_validation_errors(&e)))
}

/// Format validation errors into a human-readable string
pub fn format_validation_errors(errors: &ValidationErrors) -> String {
    let mut messages = Vec::new();
    
    for (field, field_errors) in errors.field_errors() {
        for error in field_errors {
            let message = match &error.message {
                Some(msg) => msg.to_string(),
                None => format!("Invalid value for field '{}'", field),
            };
            messages.push(format!("{}: {}", field, message));
        }
    }
    
    messages.join("; ")
}

/// Validate OSCAL version compatibility
pub fn validate_oscal_version(version: &str) -> Result<()> {
    if version != crate::types::OSCAL_VERSION {
        return Err(Error::validation(format!(
            "Unsupported OSCAL version '{}'. Expected '{}'",
            version,
            crate::types::OSCAL_VERSION
        )));
    }
    Ok(())
}

/// Validate FedRAMP template version
pub fn validate_fedramp_template_version(version: &str) -> Result<()> {
    if version != crate::types::FEDRAMP_TEMPLATE_VERSION {
        return Err(Error::validation(format!(
            "Unsupported FedRAMP template version '{}'. Expected '{}'",
            version,
            crate::types::FEDRAMP_TEMPLATE_VERSION
        )));
    }
    Ok(())
}

/// Validate UUID format
pub fn validate_uuid(uuid_str: &str) -> Result<uuid::Uuid> {
    uuid::Uuid::parse_str(uuid_str).map_err(|e| Error::validation(format!("Invalid UUID format '{}': {}", uuid_str, e)))
}

/// Validate email format
pub fn validate_email(email: &str) -> Result<()> {
    if !email.contains('@') || !email.contains('.') {
        return Err(Error::validation(format!("Invalid email format: {}", email)));
    }
    Ok(())
}

/// Validate URL format
pub fn validate_url(url: &str) -> Result<()> {
    if !url.starts_with("http://") && !url.starts_with("https://") {
        return Err(Error::validation(format!("Invalid URL format: {}", url)));
    }
    Ok(())
}

/// Custom validation error for FedRAMP-specific rules
pub fn fedramp_validation_error(message: &str) -> ValidationError {
    ValidationError::new("fedramp_compliance")
}

#[cfg(test)]
mod tests {
    use super::*;
    use validator::Validate;

    #[derive(Validate)]
    struct TestStruct {
        #[validate(length(min = 1))]
        name: String,
        #[validate(email)]
        email: String,
    }

    #[test]
    fn test_validate_struct_success() {
        let data = TestStruct {
            name: "Test".to_string(),
            email: "test@example.com".to_string(),
        };
        assert!(validate_struct(&data).is_ok());
    }

    #[test]
    fn test_validate_struct_failure() {
        let data = TestStruct {
            name: "".to_string(),
            email: "invalid-email".to_string(),
        };
        assert!(validate_struct(&data).is_err());
    }

    #[test]
    fn test_validate_oscal_version() {
        assert!(validate_oscal_version(crate::types::OSCAL_VERSION).is_ok());
        assert!(validate_oscal_version("1.0.0").is_err());
    }

    #[test]
    fn test_validate_uuid() {
        let valid_uuid = "550e8400-e29b-41d4-a716-446655440000";
        assert!(validate_uuid(valid_uuid).is_ok());
        assert!(validate_uuid("invalid-uuid").is_err());
    }

    #[test]
    fn test_validate_email() {
        assert!(validate_email("test@example.com").is_ok());
        assert!(validate_email("invalid-email").is_err());
    }

    #[test]
    fn test_validate_url() {
        assert!(validate_url("https://example.com").is_ok());
        assert!(validate_url("http://example.com").is_ok());
        assert!(validate_url("invalid-url").is_err());
    }
}
