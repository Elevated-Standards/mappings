//! POA&M field transformers for data conversion
//!
//! This module provides field transformers for converting POA&M data
//! from Excel format to OSCAL-compatible format.

use crate::{Error, Result};
use crate::mapping::poam_column_mapper::FieldTransformer;
use serde_json::Value;
use chrono::{DateTime, NaiveDate, Utc};
use std::collections::HashMap;

/// Severity level transformer
#[derive(Debug, Clone)]
pub struct SeverityTransformer {
    /// Severity mapping table
    severity_map: HashMap<String, String>,
}

/// Status transformer
#[derive(Debug, Clone)]
pub struct StatusTransformer {
    /// Status mapping table
    status_map: HashMap<String, String>,
}

/// Date transformer
#[derive(Debug, Clone)]
pub struct DateTransformer {
    /// Supported date formats
    date_formats: Vec<String>,
}

/// Control ID transformer
#[derive(Debug, Clone)]
pub struct ControlIdTransformer {
    /// Control ID normalization rules
    normalization_rules: HashMap<String, String>,
}

/// Text normalizer transformer
#[derive(Debug, Clone)]
pub struct TextNormalizerTransformer {
    /// Whether to trim whitespace
    trim_whitespace: bool,
    /// Whether to normalize case
    normalize_case: bool,
    /// Maximum length
    max_length: Option<usize>,
}

/// List transformer for comma-separated values
#[derive(Debug, Clone)]
pub struct ListTransformer {
    /// Separator patterns
    separators: Vec<String>,
    /// Whether to trim individual items
    trim_items: bool,
    /// Whether to remove empty items
    remove_empty: bool,
}

impl SeverityTransformer {
    /// Create a new severity transformer
    pub fn new() -> Self {
        let mut severity_map = HashMap::new();
        
        // Standard severity mappings
        severity_map.insert("critical".to_string(), "critical".to_string());
        severity_map.insert("high".to_string(), "high".to_string());
        severity_map.insert("moderate".to_string(), "moderate".to_string());
        severity_map.insert("medium".to_string(), "moderate".to_string());
        severity_map.insert("low".to_string(), "low".to_string());
        severity_map.insert("informational".to_string(), "informational".to_string());
        severity_map.insert("info".to_string(), "informational".to_string());
        
        // Numeric mappings
        severity_map.insert("1".to_string(), "critical".to_string());
        severity_map.insert("2".to_string(), "high".to_string());
        severity_map.insert("3".to_string(), "moderate".to_string());
        severity_map.insert("4".to_string(), "low".to_string());
        severity_map.insert("5".to_string(), "informational".to_string());
        
        Self { severity_map }
    }
}

impl FieldTransformer for SeverityTransformer {
    fn transform(&self, value: &Value) -> Result<Value> {
        let input_str = match value {
            Value::String(s) => s.to_lowercase().trim().to_string(),
            Value::Number(n) => n.to_string(),
            _ => return Err(Error::document_parsing("Invalid severity value type".to_string())),
        };

        if let Some(mapped_severity) = self.severity_map.get(&input_str) {
            Ok(Value::String(mapped_severity.clone()))
        } else {
            // Default to low if unknown
            Ok(Value::String("low".to_string()))
        }
    }

    fn name(&self) -> &str {
        "severity"
    }

    fn validate_input(&self, value: &Value) -> Result<()> {
        match value {
            Value::String(_) | Value::Number(_) => Ok(()),
            _ => Err(Error::document_parsing("Severity must be string or number".to_string())),
        }
    }
}

impl StatusTransformer {
    /// Create a new status transformer
    pub fn new() -> Self {
        let mut status_map = HashMap::new();
        
        // Standard status mappings
        status_map.insert("open".to_string(), "open".to_string());
        status_map.insert("in progress".to_string(), "in-progress".to_string());
        status_map.insert("in-progress".to_string(), "in-progress".to_string());
        status_map.insert("ongoing".to_string(), "in-progress".to_string());
        status_map.insert("completed".to_string(), "completed".to_string());
        status_map.insert("complete".to_string(), "completed".to_string());
        status_map.insert("closed".to_string(), "completed".to_string());
        status_map.insert("accepted".to_string(), "risk-accepted".to_string());
        status_map.insert("risk accepted".to_string(), "risk-accepted".to_string());
        status_map.insert("deferred".to_string(), "deferred".to_string());
        status_map.insert("rejected".to_string(), "rejected".to_string());
        
        Self { status_map }
    }
}

impl FieldTransformer for StatusTransformer {
    fn transform(&self, value: &Value) -> Result<Value> {
        let input_str = match value {
            Value::String(s) => s.to_lowercase().trim().to_string(),
            _ => return Err(Error::document_parsing("Invalid status value type".to_string())),
        };

        if let Some(mapped_status) = self.status_map.get(&input_str) {
            Ok(Value::String(mapped_status.clone()))
        } else {
            // Default to open if unknown
            Ok(Value::String("open".to_string()))
        }
    }

    fn name(&self) -> &str {
        "status"
    }

    fn validate_input(&self, value: &Value) -> Result<()> {
        match value {
            Value::String(_) => Ok(()),
            _ => Err(Error::document_parsing("Status must be string".to_string())),
        }
    }
}

impl DateTransformer {
    /// Create a new date transformer
    pub fn new() -> Self {
        Self {
            date_formats: vec![
                "%Y-%m-%d".to_string(),
                "%m/%d/%Y".to_string(),
                "%d/%m/%Y".to_string(),
                "%Y-%m-%d %H:%M:%S".to_string(),
                "%m/%d/%Y %H:%M:%S".to_string(),
            ],
        }
    }

    /// Parse date from various formats
    fn parse_date(&self, date_str: &str) -> Result<DateTime<Utc>> {
        let trimmed = date_str.trim();
        
        // Try each format
        for format in &self.date_formats {
            if let Ok(naive_date) = NaiveDate::parse_from_str(trimmed, format) {
                return Ok(naive_date.and_hms_opt(0, 0, 0).unwrap().and_utc());
            }
            
            if let Ok(naive_datetime) = chrono::NaiveDateTime::parse_from_str(trimmed, format) {
                return Ok(naive_datetime.and_utc());
            }
        }
        
        Err(Error::document_parsing(format!("Unable to parse date: {}", date_str)))
    }
}

impl FieldTransformer for DateTransformer {
    fn transform(&self, value: &Value) -> Result<Value> {
        let date_str = match value {
            Value::String(s) => s,
            Value::Number(n) => {
                // Handle Excel date serial numbers
                if let Some(days) = n.as_f64() {
                    // Excel epoch is 1900-01-01, but Excel incorrectly treats 1900 as a leap year
                    let excel_epoch = NaiveDate::from_ymd_opt(1899, 12, 30).unwrap();
                    if let Some(date) = excel_epoch.checked_add_days(chrono::Days::new(days as u64)) {
                        let datetime = date.and_hms_opt(0, 0, 0).unwrap().and_utc();
                        return Ok(Value::String(datetime.format("%Y-%m-%d").to_string()));
                    }
                }
                return Err(Error::document_parsing("Invalid numeric date value".to_string()));
            }
            _ => return Err(Error::document_parsing("Invalid date value type".to_string())),
        };

        let parsed_date = self.parse_date(date_str)?;
        Ok(Value::String(parsed_date.format("%Y-%m-%d").to_string()))
    }

    fn name(&self) -> &str {
        "date"
    }

    fn validate_input(&self, value: &Value) -> Result<()> {
        match value {
            Value::String(_) | Value::Number(_) => Ok(()),
            _ => Err(Error::document_parsing("Date must be string or number".to_string())),
        }
    }
}

impl ControlIdTransformer {
    /// Create a new control ID transformer
    pub fn new() -> Self {
        let mut normalization_rules = HashMap::new();
        
        // Common control ID variations
        normalization_rules.insert("ac-1".to_string(), "AC-1".to_string());
        normalization_rules.insert("ac1".to_string(), "AC-1".to_string());
        normalization_rules.insert("access control 1".to_string(), "AC-1".to_string());
        
        Self { normalization_rules }
    }

    /// Normalize control ID format
    fn normalize_control_id(&self, control_id: &str) -> String {
        let trimmed = control_id.trim().to_uppercase();
        
        // Check for exact match in normalization rules
        if let Some(normalized) = self.normalization_rules.get(&trimmed.to_lowercase()) {
            return normalized.clone();
        }
        
        // Apply standard normalization
        if let Some(captures) = regex::Regex::new(r"^([A-Z]{2})[-\s]*(\d+)(?:\((\d+)\))?$")
            .unwrap()
            .captures(&trimmed)
        {
            let family = captures.get(1).unwrap().as_str();
            let number = captures.get(2).unwrap().as_str();

            if let Some(enhancement) = captures.get(3) {
                format!("{}-{}({})", family, number, enhancement.as_str())
            } else {
                format!("{}-{}", family, number)
            }
        } else {
            trimmed
        }
    }
}

impl FieldTransformer for ControlIdTransformer {
    fn transform(&self, value: &Value) -> Result<Value> {
        let control_str = match value {
            Value::String(s) => s,
            _ => return Err(Error::document_parsing("Invalid control ID value type".to_string())),
        };

        let normalized = self.normalize_control_id(control_str);
        Ok(Value::String(normalized))
    }

    fn name(&self) -> &str {
        "control_id"
    }

    fn validate_input(&self, value: &Value) -> Result<()> {
        match value {
            Value::String(_) => Ok(()),
            _ => Err(Error::document_parsing("Control ID must be string".to_string())),
        }
    }
}

impl TextNormalizerTransformer {
    /// Create a new text normalizer transformer
    pub fn new() -> Self {
        Self {
            trim_whitespace: true,
            normalize_case: false,
            max_length: None,
        }
    }

    /// Create with custom options
    pub fn with_options(trim_whitespace: bool, normalize_case: bool, max_length: Option<usize>) -> Self {
        Self {
            trim_whitespace,
            normalize_case,
            max_length,
        }
    }
}

impl FieldTransformer for TextNormalizerTransformer {
    fn transform(&self, value: &Value) -> Result<Value> {
        let mut text = match value {
            Value::String(s) => s.clone(),
            Value::Number(n) => n.to_string(),
            Value::Bool(b) => b.to_string(),
            _ => return Err(Error::document_parsing("Cannot normalize non-text value".to_string())),
        };

        if self.trim_whitespace {
            text = text.trim().to_string();
        }

        if self.normalize_case {
            text = text.to_lowercase();
        }

        if let Some(max_len) = self.max_length {
            if text.len() > max_len {
                text.truncate(max_len);
            }
        }

        Ok(Value::String(text))
    }

    fn name(&self) -> &str {
        "text_normalizer"
    }

    fn validate_input(&self, _value: &Value) -> Result<()> {
        Ok(()) // Accept any value type
    }
}

impl ListTransformer {
    /// Create a new list transformer
    pub fn new() -> Self {
        Self {
            separators: vec![",".to_string(), ";".to_string(), "|".to_string()],
            trim_items: true,
            remove_empty: true,
        }
    }

    /// Create with custom separators
    pub fn with_separators(separators: Vec<String>) -> Self {
        Self {
            separators,
            trim_items: true,
            remove_empty: true,
        }
    }
}

impl FieldTransformer for ListTransformer {
    fn transform(&self, value: &Value) -> Result<Value> {
        let text = match value {
            Value::String(s) => s,
            _ => return Err(Error::document_parsing("List transformer requires string input".to_string())),
        };

        // Try each separator
        for separator in &self.separators {
            if text.contains(separator) {
                let mut items: Vec<String> = text.split(separator)
                    .map(|item| {
                        if self.trim_items {
                            item.trim().to_string()
                        } else {
                            item.to_string()
                        }
                    })
                    .collect();

                if self.remove_empty {
                    items.retain(|item| !item.is_empty());
                }

                let json_items: Vec<Value> = items.into_iter()
                    .map(Value::String)
                    .collect();

                return Ok(Value::Array(json_items));
            }
        }

        // No separator found, return single item array
        let item = if self.trim_items {
            text.trim().to_string()
        } else {
            text.to_string()
        };

        if self.remove_empty && item.is_empty() {
            Ok(Value::Array(vec![]))
        } else {
            Ok(Value::Array(vec![Value::String(item)]))
        }
    }

    fn name(&self) -> &str {
        "list"
    }

    fn validate_input(&self, value: &Value) -> Result<()> {
        match value {
            Value::String(_) => Ok(()),
            _ => Err(Error::document_parsing("List transformer requires string input".to_string())),
        }
    }
}

// Default implementations
impl Default for SeverityTransformer {
    fn default() -> Self {
        Self::new()
    }
}

impl Default for StatusTransformer {
    fn default() -> Self {
        Self::new()
    }
}

impl Default for DateTransformer {
    fn default() -> Self {
        Self::new()
    }
}

impl Default for ControlIdTransformer {
    fn default() -> Self {
        Self::new()
    }
}

impl Default for TextNormalizerTransformer {
    fn default() -> Self {
        Self::new()
    }
}

impl Default for ListTransformer {
    fn default() -> Self {
        Self::new()
    }
}
