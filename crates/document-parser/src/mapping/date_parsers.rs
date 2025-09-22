//! Date parser implementations for various formats
//!
//! This module contains specific date parser implementations for different
//! date formats commonly found in POA&M documents.

use super::date_converter::{DateParser, DateParseError, DateFormatPreferences};
use chrono::{DateTime, NaiveDate, NaiveDateTime, Utc, TimeZone, Datelike};
use regex::Regex;
use std::sync::OnceLock;
use std::collections::HashMap;

/// ISO 8601 date parser (highest priority)
pub struct Iso8601Parser;

impl DateParser for Iso8601Parser {
    fn can_parse(&self, input: &str) -> bool {
        // Check for ISO 8601 patterns
        static ISO_REGEX: OnceLock<Regex> = OnceLock::new();
        let regex = ISO_REGEX.get_or_init(|| {
            Regex::new(r"^\d{4}-\d{2}-\d{2}(T\d{2}:\d{2}:\d{2}(\.\d+)?(Z|[+-]\d{2}:\d{2})?)?$").unwrap()
        });
        
        regex.is_match(input.trim())
    }
    
    fn parse(&self, input: &str) -> std::result::Result<DateTime<Utc>, DateParseError> {
        let trimmed = input.trim();
        
        // Try full ISO 8601 with timezone
        if let Ok(dt) = DateTime::parse_from_rfc3339(trimmed) {
            return Ok(dt.with_timezone(&Utc));
        }
        
        // Try ISO date only (YYYY-MM-DD)
        if let Ok(date) = NaiveDate::parse_from_str(trimmed, "%Y-%m-%d") {
            return Ok(date.and_hms_opt(0, 0, 0).unwrap().and_utc());
        }
        
        // Try ISO datetime without timezone
        if let Ok(dt) = NaiveDateTime::parse_from_str(trimmed, "%Y-%m-%dT%H:%M:%S") {
            return Ok(dt.and_utc());
        }
        
        Err(DateParseError::InvalidFormat(format!("Invalid ISO 8601 format: {}", input)))
    }
    
    fn confidence(&self, input: &str) -> f64 {
        if self.can_parse(input) {
            if input.contains('T') {
                // Check for timezone indicators after the time part
                let time_part = input.split('T').nth(1).unwrap_or("");
                if time_part.contains('Z') || time_part.contains('+') || time_part.contains('-') {
                    0.95 // Full ISO 8601 with timezone
                } else {
                    0.9 // ISO datetime without timezone
                }
            } else {
                0.85 // ISO date only
            }
        } else {
            0.0
        }
    }
    
    fn name(&self) -> &str {
        "ISO8601"
    }
}

/// US date format parser (MM/DD/YYYY, MM-DD-YYYY)
pub struct UsDateParser {
    preferences: DateFormatPreferences,
}

impl UsDateParser {
    pub fn new(preferences: DateFormatPreferences) -> Self {
        Self { preferences }
    }
}

impl DateParser for UsDateParser {
    fn can_parse(&self, input: &str) -> bool {
        static US_REGEX: OnceLock<Regex> = OnceLock::new();
        let regex = US_REGEX.get_or_init(|| {
            Regex::new(r"^\d{1,2}[/-]\d{1,2}[/-]\d{2,4}(\s+\d{1,2}:\d{2}(:\d{2})?(\s*(AM|PM))?)?$").unwrap()
        });
        
        regex.is_match(input.trim())
    }
    
    fn parse(&self, input: &str) -> std::result::Result<DateTime<Utc>, DateParseError> {
        let trimmed = input.trim();
        
        // Try various US formats - order matters! Try more specific formats first
        let formats = if trimmed.matches('/').count() == 2 || trimmed.matches('-').count() == 2 {
            let parts: Vec<&str> = trimmed.split(|c| c == '/' || c == '-').collect();
            if parts.len() >= 3 {
                let year_part = parts[2].split_whitespace().next().unwrap_or("");
                if year_part.len() == 2 {
                    // 2-digit year formats
                    vec![
                        "%m/%d/%y",
                        "%m-%d-%y",
                        "%m/%d/%y %H:%M:%S",
                        "%m-%d-%y %H:%M:%S",
                        "%m/%d/%y %I:%M:%S %p",
                        "%m-%d-%y %I:%M:%S %p",
                        "%m/%d/%y %H:%M",
                        "%m-%d-%y %H:%M",
                    ]
                } else {
                    // 4-digit year formats
                    vec![
                        "%m/%d/%Y",
                        "%m-%d-%Y",
                        "%m/%d/%Y %H:%M:%S",
                        "%m-%d-%Y %H:%M:%S",
                        "%m/%d/%Y %I:%M:%S %p",
                        "%m-%d-%Y %I:%M:%S %p",
                        "%m/%d/%Y %H:%M",
                        "%m-%d-%Y %H:%M",
                    ]
                }
            } else {
                vec![
                    "%m/%d/%Y",
                    "%m-%d-%Y",
                    "%m/%d/%y",
                    "%m-%d-%y",
                ]
            }
        } else {
            vec![
                "%m/%d/%Y",
                "%m-%d-%Y",
                "%m/%d/%y",
                "%m-%d-%y",
                "%m/%d/%Y %H:%M:%S",
                "%m-%d-%Y %H:%M:%S",
                "%m/%d/%Y %I:%M:%S %p",
                "%m-%d-%Y %I:%M:%S %p",
                "%m/%d/%Y %H:%M",
                "%m-%d-%Y %H:%M",
            ]
        };
        
        for format in formats {
            if let Ok(dt) = NaiveDateTime::parse_from_str(trimmed, format) {
                return Ok(dt.and_utc());
            }

            if let Ok(date) = NaiveDate::parse_from_str(trimmed, format) {
                let year = if format.contains("%y") {
                    // For 2-digit years, chrono returns the actual 2-digit value
                    let two_digit_year = date.year();
                    if two_digit_year >= self.preferences.century_cutoff {
                        1900 + two_digit_year
                    } else {
                        2000 + two_digit_year
                    }
                } else {
                    date.year()
                };

                let adjusted_date = NaiveDate::from_ymd_opt(year, date.month(), date.day())
                    .ok_or_else(|| DateParseError::OutOfRange(format!("Invalid date: {}", input)))?;

                return Ok(adjusted_date.and_hms_opt(0, 0, 0).unwrap().and_utc());
            }
        }
        
        Err(DateParseError::InvalidFormat(format!("Invalid US date format: {}", input)))
    }
    
    fn confidence(&self, input: &str) -> f64 {
        if !self.can_parse(input) {
            return 0.0;
        }
        
        let trimmed = input.trim();
        
        // Higher confidence for 4-digit years
        if trimmed.matches('/').count() == 2 || trimmed.matches('-').count() == 2 {
            let parts: Vec<&str> = trimmed.split(|c| c == '/' || c == '-').collect();
            if parts.len() >= 3 {
                if let Ok(year) = parts[2].parse::<i32>() {
                    if year > 1900 && year < 2100 {
                        return if self.preferences.prefer_mdy { 0.85 } else { 0.7 };
                    }
                }
                // 2-digit year
                return if self.preferences.prefer_mdy { 0.75 } else { 0.6 };
            }
        }
        
        0.5
    }
    
    fn name(&self) -> &str {
        "US_Date"
    }
}

/// European date format parser (DD/MM/YYYY, DD-MM-YYYY)
pub struct EuropeanDateParser {
    preferences: DateFormatPreferences,
}

impl EuropeanDateParser {
    pub fn new(preferences: DateFormatPreferences) -> Self {
        Self { preferences }
    }
}

impl DateParser for EuropeanDateParser {
    fn can_parse(&self, input: &str) -> bool {
        static EU_REGEX: OnceLock<Regex> = OnceLock::new();
        let regex = EU_REGEX.get_or_init(|| {
            Regex::new(r"^\d{1,2}[/-]\d{1,2}[/-]\d{2,4}(\s+\d{1,2}:\d{2}(:\d{2})?)?$").unwrap()
        });
        
        regex.is_match(input.trim())
    }
    
    fn parse(&self, input: &str) -> std::result::Result<DateTime<Utc>, DateParseError> {
        let trimmed = input.trim();
        
        // Try various European formats
        let formats = vec![
            "%d/%m/%Y",
            "%d-%m-%Y",
            "%d/%m/%y",
            "%d-%m-%y",
            "%d/%m/%Y %H:%M:%S",
            "%d-%m-%Y %H:%M:%S",
            "%d/%m/%Y %H:%M",
            "%d-%m-%Y %H:%M",
        ];
        
        for format in formats {
            if let Ok(dt) = NaiveDateTime::parse_from_str(trimmed, format) {
                return Ok(dt.and_utc());
            }
            
            if let Ok(date) = NaiveDate::parse_from_str(trimmed, format) {
                let year = if format.contains("%y") {
                    // For 2-digit years, chrono returns the actual 2-digit value
                    let two_digit_year = date.year();
                    if two_digit_year >= self.preferences.century_cutoff {
                        1900 + two_digit_year
                    } else {
                        2000 + two_digit_year
                    }
                } else {
                    date.year()
                };

                let adjusted_date = NaiveDate::from_ymd_opt(year, date.month(), date.day())
                    .ok_or_else(|| DateParseError::OutOfRange(format!("Invalid date: {}", input)))?;

                return Ok(adjusted_date.and_hms_opt(0, 0, 0).unwrap().and_utc());
            }
        }
        
        Err(DateParseError::InvalidFormat(format!("Invalid European date format: {}", input)))
    }
    
    fn confidence(&self, input: &str) -> f64 {
        if !self.can_parse(input) {
            return 0.0;
        }
        
        let trimmed = input.trim();
        
        // Higher confidence for 4-digit years
        if trimmed.matches('/').count() == 2 || trimmed.matches('-').count() == 2 {
            let parts: Vec<&str> = trimmed.split(|c| c == '/' || c == '-').collect();
            if parts.len() >= 3 {
                if let Ok(year) = parts[2].parse::<i32>() {
                    if year > 1900 && year < 2100 {
                        return if !self.preferences.prefer_mdy { 0.85 } else { 0.7 };
                    }
                }
                // 2-digit year
                return if !self.preferences.prefer_mdy { 0.75 } else { 0.6 };
            }
        }
        
        0.5
    }
    
    fn name(&self) -> &str {
        "European_Date"
    }
}

/// Excel serial date parser
pub struct ExcelDateParser;

impl DateParser for ExcelDateParser {
    fn can_parse(&self, input: &str) -> bool {
        // Check if input is a number that could be an Excel serial date
        if let Ok(num) = input.trim().parse::<f64>() {
            // Excel dates are typically between 1 (1900-01-01) and ~50000 (2037)
            num >= 1.0 && num <= 100000.0 && num.fract() == 0.0
        } else {
            false
        }
    }
    
    fn parse(&self, input: &str) -> std::result::Result<DateTime<Utc>, DateParseError> {
        let serial = input.trim().parse::<f64>()
            .map_err(|_| DateParseError::InvalidFormat(format!("Invalid Excel serial number: {}", input)))?;
        
        // Excel epoch is 1900-01-01, but Excel incorrectly treats 1900 as a leap year
        // So we use 1899-12-30 as the base date
        let excel_epoch = NaiveDate::from_ymd_opt(1899, 12, 30)
            .ok_or_else(|| DateParseError::OutOfRange("Invalid Excel epoch".to_string()))?;
        
        let date = excel_epoch.checked_add_days(chrono::Days::new(serial as u64))
            .ok_or_else(|| DateParseError::OutOfRange(format!("Excel serial date out of range: {}", serial)))?;
        
        Ok(date.and_hms_opt(0, 0, 0).unwrap().and_utc())
    }
    
    fn confidence(&self, input: &str) -> f64 {
        if self.can_parse(input) {
            if let Ok(num) = input.trim().parse::<f64>() {
                // Higher confidence for typical Excel date range
                if num >= 36526.0 && num <= 47482.0 { // 2000-2030 range
                    0.8
                } else if num >= 1.0 && num <= 50000.0 {
                    0.6
                } else {
                    0.3 // Lower confidence for very large numbers
                }
            } else {
                0.0
            }
        } else {
            0.0
        }
    }
    
    fn name(&self) -> &str {
        "Excel_Serial"
    }
}

/// Performance-optimized date parser with format caching
pub struct OptimizedDateParser {
    preferences: DateFormatPreferences,
    format_cache: std::sync::Mutex<HashMap<String, String>>,
}

impl OptimizedDateParser {
    pub fn new(preferences: DateFormatPreferences) -> Self {
        Self {
            preferences,
            format_cache: std::sync::Mutex::new(HashMap::new()),
        }
    }

    /// Quick format detection based on pattern analysis
    fn detect_likely_format(&self, input: &str) -> Option<&'static str> {
        let trimmed = input.trim();

        // Quick character-based detection
        if trimmed.len() == 10 && trimmed.chars().nth(4) == Some('-') && trimmed.chars().nth(7) == Some('-') {
            return Some("%Y-%m-%d"); // ISO date
        }

        if trimmed.contains('T') && (trimmed.contains('Z') || trimmed.contains('+')) {
            return Some("%Y-%m-%dT%H:%M:%S%z"); // ISO datetime with timezone
        }

        if trimmed.contains('T') {
            return Some("%Y-%m-%dT%H:%M:%S"); // ISO datetime
        }

        // Check for US/European format patterns
        if trimmed.matches('/').count() == 2 {
            let parts: Vec<&str> = trimmed.split('/').collect();
            if parts.len() == 3 {
                if parts[2].len() == 4 {
                    return if self.preferences.prefer_mdy { Some("%m/%d/%Y") } else { Some("%d/%m/%Y") };
                } else if parts[2].len() == 2 {
                    return if self.preferences.prefer_mdy { Some("%m/%d/%y") } else { Some("%d/%m/%y") };
                }
            }
        }

        if trimmed.matches('-').count() == 2 && !trimmed.starts_with(|c: char| c.is_ascii_digit() && c as u8 > b'3') {
            let parts: Vec<&str> = trimmed.split('-').collect();
            if parts.len() == 3 && parts[0].len() <= 2 {
                if parts[2].len() == 4 {
                    return if self.preferences.prefer_mdy { Some("%m-%d-%Y") } else { Some("%d-%m-%Y") };
                } else if parts[2].len() == 2 {
                    return if self.preferences.prefer_mdy { Some("%m-%d-%y") } else { Some("%d-%m-%y") };
                }
            }
        }

        None
    }
}

impl DateParser for OptimizedDateParser {
    fn can_parse(&self, input: &str) -> bool {
        let trimmed = input.trim();

        // Quick rejection for obviously invalid inputs
        if trimmed.is_empty() || trimmed.len() < 6 || trimmed.len() > 30 {
            return false;
        }

        // Must contain digits
        if !trimmed.chars().any(|c| c.is_ascii_digit()) {
            return false;
        }

        // Check for common date separators or ISO format
        trimmed.contains('/') || trimmed.contains('-') || trimmed.contains('T')
    }

    fn parse(&self, input: &str) -> std::result::Result<DateTime<Utc>, DateParseError> {
        let trimmed = input.trim();

        // Check cache first
        if let Ok(cache) = self.format_cache.lock() {
            if let Some(cached_format) = cache.get(trimmed) {
                if let Ok(dt) = NaiveDateTime::parse_from_str(trimmed, cached_format) {
                    return Ok(dt.and_utc());
                }
                if let Ok(date) = NaiveDate::parse_from_str(trimmed, cached_format) {
                    return Ok(date.and_hms_opt(0, 0, 0).unwrap().and_utc());
                }
            }
        }

        // Try likely format first
        if let Some(likely_format) = self.detect_likely_format(trimmed) {
            if let Ok(dt) = NaiveDateTime::parse_from_str(trimmed, likely_format) {
                // Cache successful format
                if let Ok(mut cache) = self.format_cache.lock() {
                    cache.insert(trimmed.to_string(), likely_format.to_string());
                }
                return Ok(dt.and_utc());
            }

            if let Ok(date) = NaiveDate::parse_from_str(trimmed, likely_format) {
                let year = if likely_format.contains("%y") {
                    let two_digit_year = date.year();
                    if two_digit_year >= self.preferences.century_cutoff {
                        1900 + two_digit_year
                    } else {
                        2000 + two_digit_year
                    }
                } else {
                    date.year()
                };

                let adjusted_date = NaiveDate::from_ymd_opt(year, date.month(), date.day())
                    .ok_or_else(|| DateParseError::OutOfRange(format!("Invalid date: {}", input)))?;

                // Cache successful format
                if let Ok(mut cache) = self.format_cache.lock() {
                    cache.insert(trimmed.to_string(), likely_format.to_string());
                }

                return Ok(adjusted_date.and_hms_opt(0, 0, 0).unwrap().and_utc());
            }
        }

        // Fallback to comprehensive parsing (similar to existing parsers)
        Err(DateParseError::InvalidFormat(format!("Could not parse date: {}", input)))
    }

    fn confidence(&self, input: &str) -> f64 {
        if !self.can_parse(input) {
            return 0.0;
        }

        let trimmed = input.trim();

        // Higher confidence for formats we can detect quickly
        if let Some(_) = self.detect_likely_format(trimmed) {
            0.9
        } else {
            0.5
        }
    }

    fn name(&self) -> &str {
        "Optimized"
    }
}

/// Natural language date parser using dateparser crate
pub struct NaturalLanguageParser;

impl DateParser for NaturalLanguageParser {
    fn can_parse(&self, input: &str) -> bool {
        // Check for common natural language patterns
        let input_lower = input.to_lowercase();
        input_lower.contains("today") ||
        input_lower.contains("tomorrow") ||
        input_lower.contains("yesterday") ||
        input_lower.contains("next") ||
        input_lower.contains("last") ||
        input_lower.contains("end of") ||
        input_lower.contains("beginning of") ||
        input_lower.contains("monday") ||
        input_lower.contains("tuesday") ||
        input_lower.contains("wednesday") ||
        input_lower.contains("thursday") ||
        input_lower.contains("friday") ||
        input_lower.contains("saturday") ||
        input_lower.contains("sunday") ||
        input_lower.contains("january") || input_lower.contains("jan") ||
        input_lower.contains("february") || input_lower.contains("feb") ||
        input_lower.contains("march") || input_lower.contains("mar") ||
        input_lower.contains("april") || input_lower.contains("apr") ||
        input_lower.contains("may") ||
        input_lower.contains("june") || input_lower.contains("jun") ||
        input_lower.contains("july") || input_lower.contains("jul") ||
        input_lower.contains("august") || input_lower.contains("aug") ||
        input_lower.contains("september") || input_lower.contains("sep") ||
        input_lower.contains("october") || input_lower.contains("oct") ||
        input_lower.contains("november") || input_lower.contains("nov") ||
        input_lower.contains("december") || input_lower.contains("dec")
    }
    
    fn parse(&self, input: &str) -> std::result::Result<DateTime<Utc>, DateParseError> {
        match dateparser::parse(input) {
            Ok(dt) => Ok(dt.with_timezone(&Utc)),
            Err(e) => Err(DateParseError::InvalidFormat(format!("Natural language parsing failed: {}", e))),
        }
    }
    
    fn confidence(&self, input: &str) -> f64 {
        if self.can_parse(input) {
            // Lower confidence for natural language parsing
            0.4
        } else {
            0.0
        }
    }
    
    fn name(&self) -> &str {
        "Natural_Language"
    }
}
