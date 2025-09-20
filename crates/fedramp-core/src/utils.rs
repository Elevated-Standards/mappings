// Modified: 2025-09-20

//! Utility functions for FedRAMP compliance automation.
//!
//! This module provides common utility functions used throughout
//! the FedRAMP compliance automation platform.

use crate::error::Error;
use crate::types::{Result, Timestamp};
use serde_json::Value;
use std::collections::HashMap;
use std::path::Path;

/// Generate a new UUID v4
pub fn generate_uuid() -> uuid::Uuid {
    uuid::Uuid::new_v4()
}

/// Get current UTC timestamp
pub fn current_timestamp() -> Timestamp {
    chrono::Utc::now()
}

/// Format timestamp as ISO 8601 string
pub fn format_timestamp(timestamp: &Timestamp) -> String {
    timestamp.to_rfc3339()
}

/// Parse ISO 8601 timestamp string
pub fn parse_timestamp(timestamp_str: &str) -> Result<Timestamp> {
    chrono::DateTime::parse_from_rfc3339(timestamp_str)
        .map(|dt| dt.with_timezone(&chrono::Utc))
        .map_err(|e| Error::validation(format!("Invalid timestamp format '{}': {}", timestamp_str, e)))
}

/// Sanitize string for use in file names
pub fn sanitize_filename(input: &str) -> String {
    input
        .chars()
        .map(|c| match c {
            'a'..='z' | 'A'..='Z' | '0'..='9' | '-' | '_' | '.' => c,
            ' ' => '_',
            _ => '-',
        })
        .collect()
}

/// Convert string to kebab-case
pub fn to_kebab_case(input: &str) -> String {
    input
        .chars()
        .enumerate()
        .map(|(i, c)| {
            if c.is_uppercase() && i > 0 {
                format!("-{}", c.to_lowercase())
            } else {
                c.to_lowercase().to_string()
            }
        })
        .collect()
}

/// Convert string to snake_case
pub fn to_snake_case(input: &str) -> String {
    input
        .chars()
        .enumerate()
        .map(|(i, c)| {
            if c.is_uppercase() && i > 0 {
                format!("_{}", c.to_lowercase())
            } else {
                c.to_lowercase().to_string()
            }
        })
        .collect()
}

/// Merge two JSON values recursively
pub fn merge_json(base: &mut Value, overlay: &Value) {
    match (base.as_object_mut(), overlay.as_object()) {
        (Some(base_map), Some(overlay_map)) => {
            for (key, value) in overlay_map {
                match base_map.get_mut(key) {
                    Some(base_value) => merge_json(base_value, value),
                    None => {
                        base_map.insert(key.clone(), value.clone());
                    }
                }
            }
        }
        _ => *base = overlay.clone(),
    }
}

/// Deep clone a JSON value
pub fn deep_clone_json(value: &Value) -> Value {
    value.clone()
}

/// Extract nested value from JSON using dot notation
pub fn get_nested_value<'a>(value: &'a Value, path: &str) -> Option<&'a Value> {
    let parts: Vec<&str> = path.split('.').collect();
    let mut current = value;

    for part in parts {
        match current {
            Value::Object(map) => {
                current = map.get(part)?;
            }
            Value::Array(arr) => {
                let index: usize = part.parse().ok()?;
                current = arr.get(index)?;
            }
            _ => return None,
        }
    }

    Some(current)
}

/// Set nested value in JSON using dot notation
pub fn set_nested_value(value: &mut Value, path: &str, new_value: Value) -> Result<()> {
    let parts: Vec<&str> = path.split('.').collect();
    if parts.is_empty() {
        return Err(Error::validation("Empty path provided"));
    }

    let mut current = value;
    for (i, part) in parts.iter().enumerate() {
        if i == parts.len() - 1 {
            // Last part - set the value
            match current {
                Value::Object(map) => {
                    map.insert(part.to_string(), new_value);
                    return Ok(());
                }
                _ => {
                    return Err(Error::validation(format!("Cannot set property '{}' on non-object", part)));
                }
            }
        } else {
            // Navigate deeper
            match current {
                Value::Object(map) => {
                    current = map
                        .entry(part.to_string())
                        .or_insert_with(|| Value::Object(serde_json::Map::new()));
                }
                _ => {
                    return Err(Error::validation(format!("Cannot navigate through non-object at '{}'", part)));
                }
            }
        }
    }

    Ok(())
}

/// Calculate file hash (simple hash for now)
pub fn calculate_file_hash<P: AsRef<Path>>(path: P) -> Result<String> {
    use std::io::Read;
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};

    let mut file = std::fs::File::open(path).map_err(|e| Error::internal(format!("Failed to open file for hashing: {}", e)))?;

    let mut hasher = DefaultHasher::new();
    let mut buffer = [0; 8192];

    loop {
        let bytes_read = file.read(&mut buffer).map_err(|e| Error::internal(format!("Failed to read file for hashing: {}", e)))?;

        if bytes_read == 0 {
            break;
        }

        buffer[..bytes_read].hash(&mut hasher);
    }

    Ok(format!("{:x}", hasher.finish()))
}

/// Validate and normalize email address
pub fn normalize_email(email: &str) -> Result<String> {
    let trimmed = email.trim().to_lowercase();
    
    if !trimmed.contains('@') {
        return Err(Error::validation("Invalid email format: missing @ symbol"));
    }

    let parts: Vec<&str> = trimmed.split('@').collect();
    if parts.len() != 2 || parts[0].is_empty() || parts[1].is_empty() {
        return Err(Error::validation("Invalid email format"));
    }

    Ok(trimmed)
}

/// Create a HashMap from key-value pairs
pub fn hashmap_from_pairs<K, V>(pairs: Vec<(K, V)>) -> HashMap<K, V>
where
    K: std::hash::Hash + Eq,
{
    pairs.into_iter().collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_sanitize_filename() {
        assert_eq!(sanitize_filename("Hello World!"), "Hello_World-");
        assert_eq!(sanitize_filename("test-file_123.txt"), "test-file_123.txt");
    }

    #[test]
    fn test_case_conversion() {
        assert_eq!(to_kebab_case("HelloWorld"), "hello-world");
        assert_eq!(to_snake_case("HelloWorld"), "hello_world");
    }

    #[test]
    fn test_json_operations() {
        let mut base = json!({"a": 1, "b": {"c": 2}});
        let overlay = json!({"b": {"d": 3}, "e": 4});
        
        merge_json(&mut base, &overlay);
        
        assert_eq!(base["a"], 1);
        assert_eq!(base["b"]["c"], 2);
        assert_eq!(base["b"]["d"], 3);
        assert_eq!(base["e"], 4);
    }

    #[test]
    fn test_nested_value_access() {
        let value = json!({"a": {"b": {"c": 42}}});
        
        assert_eq!(get_nested_value(&value, "a.b.c"), Some(&json!(42)));
        assert_eq!(get_nested_value(&value, "a.b.d"), None);
    }

    #[test]
    fn test_normalize_email() {
        assert_eq!(normalize_email("  Test@Example.COM  ").unwrap(), "test@example.com");
        assert!(normalize_email("invalid-email").is_err());
    }

    #[test]
    fn test_timestamp_operations() {
        let now = current_timestamp();
        let formatted = format_timestamp(&now);
        let parsed = parse_timestamp(&formatted).unwrap();
        
        // Allow for small differences due to precision
        assert!((now.timestamp() - parsed.timestamp()).abs() < 1);
    }
}
