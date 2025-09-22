//! Enhanced date conversion module for POA&M documents
//!
//! This module provides comprehensive date parsing, conversion, and validation
//! capabilities to ensure OSCAL schema compliance and consistent date handling.

use crate::{Error, Result};
use chrono::{DateTime, NaiveDate, NaiveDateTime, Utc, TimeZone, Datelike, Timelike};
use chrono_tz::Tz;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::str::FromStr;
use thiserror::Error;

/// Date parsing errors
#[derive(Error, Debug)]
pub enum DateParseError {
    #[error("Invalid date format: {0}")]
    InvalidFormat(String),
    #[error("Ambiguous date: {0}")]
    AmbiguousDate(String),
    #[error("Date out of range: {0}")]
    OutOfRange(String),
    #[error("Timezone error: {0}")]
    TimezoneError(String),
}

/// Date parsing warnings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DateWarning {
    AmbiguousFormat { original: String, assumed_format: String },
    TimezoneAssumed { original: String, assumed_timezone: String },
    YearAssumed { original: String, assumed_year: i32 },
    LowConfidence { original: String, confidence: f64 },
}

/// Date parsing result with metadata
#[derive(Debug, Clone)]
pub struct DateParsingResult {
    pub parsed_date: Option<DateTime<Utc>>,
    pub original_format: String,
    pub confidence: f64,
    pub warnings: Vec<DateWarning>,
    pub iso_string: Option<String>,
}

/// Date parser trait for different format handlers
pub trait DateParser: Send + Sync {
    /// Check if this parser can handle the input
    fn can_parse(&self, input: &str) -> bool;
    
    /// Parse the input string to UTC datetime
    fn parse(&self, input: &str) -> std::result::Result<DateTime<Utc>, DateParseError>;
    
    /// Get confidence score for parsing this input (0.0 to 1.0)
    fn confidence(&self, input: &str) -> f64;
    
    /// Get parser name for debugging
    fn name(&self) -> &str;
}

/// Timezone configuration
#[derive(Debug, Clone)]
pub struct TimezoneConfig {
    pub default_timezone: Tz,
    pub assume_utc_for_dates: bool,
    pub business_timezone: Option<Tz>,
}

impl Default for TimezoneConfig {
    fn default() -> Self {
        Self {
            default_timezone: chrono_tz::UTC,
            assume_utc_for_dates: true,
            business_timezone: Some(chrono_tz::US::Eastern),
        }
    }
}

/// Date format preferences for ambiguous dates
#[derive(Debug, Clone)]
pub struct DateFormatPreferences {
    pub prefer_mdy: bool,  // MM/DD/YYYY vs DD/MM/YYYY
    pub prefer_4digit_year: bool,
    pub century_cutoff: i32,  // 2-digit years >= this are 19xx, < this are 20xx
}

impl Default for DateFormatPreferences {
    fn default() -> Self {
        Self {
            prefer_mdy: true,  // US format preference
            prefer_4digit_year: true,
            century_cutoff: 50,  // 50-99 = 1950-1999, 00-49 = 2000-2049
        }
    }
}

/// Date validation rule types
#[derive(Debug, Clone, PartialEq)]
pub enum DateRuleType {
    FutureDate,
    PastDate,
    BusinessDay,
    SequenceCheck,
    RangeValidation,
}

/// Date constraint for validation
pub enum DateConstraint {
    After(DateTime<Utc>),
    Before(DateTime<Utc>),
    Between(DateTime<Utc>, DateTime<Utc>),
    BusinessDaysOnly,
    WeekdaysOnly,
    Custom(Box<dyn Fn(&DateTime<Utc>) -> bool + Send + Sync>),
}

impl std::fmt::Debug for DateConstraint {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DateConstraint::After(dt) => write!(f, "After({})", dt),
            DateConstraint::Before(dt) => write!(f, "Before({})", dt),
            DateConstraint::Between(start, end) => write!(f, "Between({}, {})", start, end),
            DateConstraint::BusinessDaysOnly => write!(f, "BusinessDaysOnly"),
            DateConstraint::WeekdaysOnly => write!(f, "WeekdaysOnly"),
            DateConstraint::Custom(_) => write!(f, "Custom(<function>)"),
        }
    }
}

impl Clone for DateConstraint {
    fn clone(&self) -> Self {
        match self {
            DateConstraint::After(dt) => DateConstraint::After(*dt),
            DateConstraint::Before(dt) => DateConstraint::Before(*dt),
            DateConstraint::Between(start, end) => DateConstraint::Between(*start, *end),
            DateConstraint::BusinessDaysOnly => DateConstraint::BusinessDaysOnly,
            DateConstraint::WeekdaysOnly => DateConstraint::WeekdaysOnly,
            DateConstraint::Custom(_) => {
                // For custom functions, we can't clone them, so we'll create a no-op
                DateConstraint::Custom(Box::new(|_| true))
            }
        }
    }
}

/// Date validation rule
#[derive(Debug, Clone)]
pub struct DateValidationRule {
    pub name: String,
    pub field_name: String,
    pub rule_type: DateRuleType,
    pub constraint: DateConstraint,
    pub required: bool,
}

/// Main date converter with multiple parsers and validation
pub struct DateConverter {
    parsers: Vec<Box<dyn DateParser>>,
    timezone_config: TimezoneConfig,
    validation_rules: Vec<DateValidationRule>,
    format_preferences: DateFormatPreferences,
}

impl DateConverter {
    /// Create a new date converter with default configuration
    pub fn new() -> Self {
        let mut converter = Self {
            parsers: Vec::new(),
            timezone_config: TimezoneConfig::default(),
            validation_rules: Vec::new(),
            format_preferences: DateFormatPreferences::default(),
        };
        
        // Add default parsers in order of preference
        converter.add_default_parsers();
        converter
    }
    
    /// Create with custom configuration
    pub fn with_config(
        timezone_config: TimezoneConfig,
        format_preferences: DateFormatPreferences,
    ) -> Self {
        let mut converter = Self {
            parsers: Vec::new(),
            timezone_config,
            validation_rules: Vec::new(),
            format_preferences,
        };
        
        converter.add_default_parsers();
        converter
    }
    
    /// Add a custom date parser
    pub fn add_parser(&mut self, parser: Box<dyn DateParser>) {
        self.parsers.push(parser);
    }
    
    /// Add a validation rule
    pub fn add_validation_rule(&mut self, rule: DateValidationRule) {
        self.validation_rules.push(rule);
    }
    
    /// Parse a date string with full result metadata
    pub fn parse_date(&self, input: &str) -> DateParsingResult {
        let trimmed_input = input.trim();
        
        if trimmed_input.is_empty() {
            return DateParsingResult {
                parsed_date: None,
                original_format: "empty".to_string(),
                confidence: 0.0,
                warnings: Vec::new(),
                iso_string: None,
            };
        }
        
        // Try each parser in order
        for parser in &self.parsers {
            if parser.can_parse(trimmed_input) {
                match parser.parse(trimmed_input) {
                    Ok(datetime) => {
                        let confidence = parser.confidence(trimmed_input);
                        let iso_string = datetime.to_rfc3339();
                        
                        let mut warnings = Vec::new();
                        if confidence < 0.8 {
                            warnings.push(DateWarning::LowConfidence {
                                original: input.to_string(),
                                confidence,
                            });
                        }
                        
                        return DateParsingResult {
                            parsed_date: Some(datetime),
                            original_format: parser.name().to_string(),
                            confidence,
                            warnings,
                            iso_string: Some(iso_string),
                        };
                    }
                    Err(_) => continue,
                }
            }
        }
        
        // No parser could handle the input
        DateParsingResult {
            parsed_date: None,
            original_format: "unknown".to_string(),
            confidence: 0.0,
            warnings: vec![DateWarning::LowConfidence {
                original: input.to_string(),
                confidence: 0.0,
            }],
            iso_string: None,
        }
    }
    
    /// Convert date to ISO 8601 format
    pub fn to_iso8601(&self, input: &str) -> Result<String> {
        let result = self.parse_date(input);

        match result.parsed_date {
            Some(datetime) => Ok(datetime.to_rfc3339()),
            None => Err(Error::document_parsing(format!(
                "Unable to parse date: {}",
                input
            ))),
        }
    }

    /// Parse date with timezone context
    pub fn parse_date_with_timezone(&self, input: &str, source_timezone: Option<Tz>) -> DateParsingResult {
        let mut result = self.parse_date(input);

        if let Some(datetime) = result.parsed_date {
            // If no timezone was specified in the original date and we have a source timezone
            if let Some(tz) = source_timezone {
                // Check if the original parsing included timezone info
                if !input.contains('Z') && !input.contains('+') && !input.contains('-') {
                    // Assume the date is in the source timezone and convert to UTC
                    let local_dt = tz.from_utc_datetime(&datetime.naive_utc());
                    result.parsed_date = Some(local_dt.with_timezone(&Utc));
                    result.warnings.push(DateWarning::TimezoneAssumed {
                        original: input.to_string(),
                        assumed_timezone: tz.to_string(),
                    });
                }
            }
        }

        result
    }

    /// Convert datetime from one timezone to another
    pub fn convert_timezone(&self, datetime: &DateTime<Utc>, target_timezone: Tz) -> DateTime<Tz> {
        datetime.with_timezone(&target_timezone)
    }

    /// Get business hours adjusted date
    pub fn adjust_to_business_hours(&self, datetime: &DateTime<Utc>) -> DateTime<Utc> {
        let business_tz = self.timezone_config.business_timezone
            .unwrap_or(self.timezone_config.default_timezone);

        let local_dt = datetime.with_timezone(&business_tz);

        // If it's a weekend, move to next Monday
        let adjusted_date = match local_dt.weekday() {
            chrono::Weekday::Sat => local_dt + chrono::Duration::days(2),
            chrono::Weekday::Sun => local_dt + chrono::Duration::days(1),
            _ => local_dt,
        };

        // Ensure it's during business hours (9 AM - 5 PM)
        let hour = adjusted_date.hour();
        let final_dt = if hour < 9 {
            adjusted_date.with_hour(9).unwrap().with_minute(0).unwrap().with_second(0).unwrap()
        } else if hour >= 17 {
            (adjusted_date + chrono::Duration::days(1))
                .with_hour(9).unwrap().with_minute(0).unwrap().with_second(0).unwrap()
        } else {
            adjusted_date
        };

        final_dt.with_timezone(&Utc)
    }
    
    /// Validate a date against configured rules
    pub fn validate_date(&self, datetime: &DateTime<Utc>, field_name: &str) -> Vec<String> {
        let mut errors = Vec::new();

        for rule in &self.validation_rules {
            if rule.field_name == field_name || rule.field_name == "*" {
                if let Some(error) = self.validate_against_rule(datetime, rule) {
                    errors.push(error);
                }
            }
        }

        errors
    }

    /// Validate date sequence (e.g., start_date < end_date)
    pub fn validate_date_sequence(&self, dates: &[(&str, &DateTime<Utc>)]) -> Vec<String> {
        let mut errors = Vec::new();

        // Check for logical date sequences
        for i in 0..dates.len() {
            for j in (i + 1)..dates.len() {
                let (name1, date1) = dates[i];
                let (name2, date2) = dates[j];

                // Common sequence validations
                if (name1.contains("start") || name1.contains("open")) &&
                   (name2.contains("end") || name2.contains("close") || name2.contains("due")) {
                    if date1 > date2 {
                        errors.push(format!("{} ({}) must be before {} ({})",
                            name1, date1.format("%Y-%m-%d"),
                            name2, date2.format("%Y-%m-%d")
                        ));
                    }
                }

                if name1.contains("scheduled") && name2.contains("actual") {
                    // Actual dates can be after scheduled dates, but warn if significantly different
                    let diff = (*date2 - *date1).num_days();
                    if diff > 90 {
                        errors.push(format!("Warning: {} is {} days after {}",
                            name2, diff, name1
                        ));
                    }
                }
            }
        }

        errors
    }

    /// Add common POA&M validation rules
    pub fn add_poam_validation_rules(&mut self) {
        let now = Utc::now();

        // Scheduled completion dates should be in the future
        self.add_validation_rule(DateValidationRule {
            name: "Future Scheduled Date".to_string(),
            field_name: "scheduled_completion_date".to_string(),
            rule_type: DateRuleType::FutureDate,
            constraint: DateConstraint::After(now),
            required: false,
        });

        // Actual completion dates should not be in the future
        self.add_validation_rule(DateValidationRule {
            name: "Past Actual Date".to_string(),
            field_name: "actual_completion_date".to_string(),
            rule_type: DateRuleType::PastDate,
            constraint: DateConstraint::Before(now + chrono::Duration::days(1)),
            required: false,
        });

        // Milestone dates should be business days
        self.add_validation_rule(DateValidationRule {
            name: "Business Day Milestone".to_string(),
            field_name: "milestone_date".to_string(),
            rule_type: DateRuleType::BusinessDay,
            constraint: DateConstraint::BusinessDaysOnly,
            required: false,
        });

        // Dates should be within reasonable range (not too far in past/future)
        let min_date = now - chrono::Duration::days(365 * 10); // 10 years ago
        let max_date = now + chrono::Duration::days(365 * 5);  // 5 years from now

        self.add_validation_rule(DateValidationRule {
            name: "Reasonable Date Range".to_string(),
            field_name: "*".to_string(),
            rule_type: DateRuleType::RangeValidation,
            constraint: DateConstraint::Between(min_date, max_date),
            required: false,
        });
    }
    
    /// Add default parsers in order of preference
    fn add_default_parsers(&mut self) {
        use super::date_parsers::*;

        // Add optimized parser first for best performance
        self.parsers.push(Box::new(OptimizedDateParser::new(self.format_preferences.clone())));

        // Add parsers in order of preference (highest confidence first)
        self.parsers.push(Box::new(Iso8601Parser));
        self.parsers.push(Box::new(ExcelDateParser));

        if self.format_preferences.prefer_mdy {
            self.parsers.push(Box::new(UsDateParser::new(self.format_preferences.clone())));
            self.parsers.push(Box::new(EuropeanDateParser::new(self.format_preferences.clone())));
        } else {
            self.parsers.push(Box::new(EuropeanDateParser::new(self.format_preferences.clone())));
            self.parsers.push(Box::new(UsDateParser::new(self.format_preferences.clone())));
        }

        // Natural language parser has lowest priority
        self.parsers.push(Box::new(NaturalLanguageParser));
    }
    
    /// Validate a date against a specific rule
    fn validate_against_rule(&self, datetime: &DateTime<Utc>, rule: &DateValidationRule) -> Option<String> {
        match &rule.constraint {
            DateConstraint::After(after_date) => {
                if datetime <= after_date {
                    Some(format!("{}: Date must be after {}", rule.name, after_date.format("%Y-%m-%d")))
                } else {
                    None
                }
            }
            DateConstraint::Before(before_date) => {
                if datetime >= before_date {
                    Some(format!("{}: Date must be before {}", rule.name, before_date.format("%Y-%m-%d")))
                } else {
                    None
                }
            }
            DateConstraint::Between(start, end) => {
                if datetime < start || datetime > end {
                    Some(format!("{}: Date must be between {} and {}", 
                        rule.name, 
                        start.format("%Y-%m-%d"), 
                        end.format("%Y-%m-%d")
                    ))
                } else {
                    None
                }
            }
            DateConstraint::BusinessDaysOnly => {
                let weekday = datetime.weekday();
                if weekday == chrono::Weekday::Sat || weekday == chrono::Weekday::Sun {
                    Some(format!("{}: Date must be a business day", rule.name))
                } else {
                    None
                }
            }
            DateConstraint::WeekdaysOnly => {
                let weekday = datetime.weekday();
                if weekday == chrono::Weekday::Sat || weekday == chrono::Weekday::Sun {
                    Some(format!("{}: Date must be a weekday", rule.name))
                } else {
                    None
                }
            }
            DateConstraint::Custom(validator) => {
                if !validator(datetime) {
                    Some(format!("{}: Custom validation failed", rule.name))
                } else {
                    None
                }
            }
        }
    }
}

impl Default for DateConverter {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Debug for DateConverter {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("DateConverter")
            .field("parsers", &format!("{} parsers", self.parsers.len()))
            .field("timezone_config", &self.timezone_config)
            .field("validation_rules", &self.validation_rules)
            .field("format_preferences", &self.format_preferences)
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mapping::date_parsers::*;

    include!("date_converter_tests.rs");
}
