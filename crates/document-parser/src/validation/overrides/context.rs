// Modified: 2025-09-22

//! Override context management and utilities
//!
//! This module provides functionality for managing override contexts,
//! including builder patterns, validation, and utility methods.

use super::types::OverrideContext;
use std::collections::HashMap;

impl OverrideContext {
    /// Create a new override context
    pub fn new(document_type: String) -> Self {
        Self {
            document_type,
            file_name: None,
            user_id: None,
            organization: None,
            session_id: None,
            project_id: None,
            metadata: HashMap::new(),
            column_count: None,
            sample_data: None,
        }
    }

    /// Create a builder for constructing override contexts
    pub fn builder(document_type: String) -> OverrideContextBuilder {
        OverrideContextBuilder::new(document_type)
    }

    /// Set the file name
    pub fn with_file_name(mut self, file_name: String) -> Self {
        self.file_name = Some(file_name);
        self
    }

    /// Set the user ID
    pub fn with_user_id(mut self, user_id: String) -> Self {
        self.user_id = Some(user_id);
        self
    }

    /// Set the organization
    pub fn with_organization(mut self, organization: String) -> Self {
        self.organization = Some(organization);
        self
    }

    /// Set the session ID
    pub fn with_session_id(mut self, session_id: String) -> Self {
        self.session_id = Some(session_id);
        self
    }

    /// Set the project ID
    pub fn with_project_id(mut self, project_id: String) -> Self {
        self.project_id = Some(project_id);
        self
    }

    /// Set the column count
    pub fn with_column_count(mut self, column_count: usize) -> Self {
        self.column_count = Some(column_count);
        self
    }

    /// Set sample data
    pub fn with_sample_data(mut self, sample_data: Vec<serde_json::Value>) -> Self {
        self.sample_data = Some(sample_data);
        self
    }

    /// Add metadata entry
    pub fn with_metadata(mut self, key: String, value: serde_json::Value) -> Self {
        self.metadata.insert(key, value);
        self
    }

    /// Add multiple metadata entries
    pub fn with_metadata_map(mut self, metadata: HashMap<String, serde_json::Value>) -> Self {
        self.metadata.extend(metadata);
        self
    }

    /// Get metadata value by key
    pub fn get_metadata(&self, key: &str) -> Option<&serde_json::Value> {
        self.metadata.get(key)
    }

    /// Check if context has metadata key
    pub fn has_metadata(&self, key: &str) -> bool {
        self.metadata.contains_key(key)
    }

    /// Get metadata as string
    pub fn get_metadata_string(&self, key: &str) -> Option<String> {
        self.metadata.get(key)?.as_str().map(|s| s.to_string())
    }

    /// Get metadata as number
    pub fn get_metadata_number(&self, key: &str) -> Option<f64> {
        self.metadata.get(key)?.as_f64()
    }

    /// Get metadata as boolean
    pub fn get_metadata_bool(&self, key: &str) -> Option<bool> {
        self.metadata.get(key)?.as_bool()
    }

    /// Validate the context for completeness
    pub fn validate(&self) -> Result<(), String> {
        if self.document_type.is_empty() {
            return Err("Document type cannot be empty".to_string());
        }

        // Validate metadata values
        for (key, value) in &self.metadata {
            if key.is_empty() {
                return Err("Metadata keys cannot be empty".to_string());
            }
            
            // Check for reasonable value types
            match value {
                serde_json::Value::Null => {
                    return Err(format!("Metadata value for key '{}' cannot be null", key));
                }
                serde_json::Value::Object(obj) if obj.is_empty() => {
                    return Err(format!("Metadata value for key '{}' cannot be empty object", key));
                }
                serde_json::Value::Array(arr) if arr.is_empty() => {
                    return Err(format!("Metadata value for key '{}' cannot be empty array", key));
                }
                _ => {} // Valid value types
            }
        }

        // Validate column count if present
        if let Some(count) = self.column_count {
            if count == 0 {
                return Err("Column count must be greater than 0".to_string());
            }
            if count > 10000 {
                return Err("Column count seems unreasonably large (>10000)".to_string());
            }
        }

        // Validate sample data if present
        if let Some(ref sample_data) = self.sample_data {
            if sample_data.is_empty() {
                return Err("Sample data cannot be empty if provided".to_string());
            }
            if sample_data.len() > 1000 {
                return Err("Sample data is too large (>1000 entries)".to_string());
            }
        }

        Ok(())
    }

    /// Generate a cache key for this context
    pub fn cache_key(&self) -> String {
        let mut key_parts = vec![
            format!("doc:{}", self.document_type),
        ];

        if let Some(ref file_name) = self.file_name {
            key_parts.push(format!("file:{}", file_name));
        }

        if let Some(ref user_id) = self.user_id {
            key_parts.push(format!("user:{}", user_id));
        }

        if let Some(ref organization) = self.organization {
            key_parts.push(format!("org:{}", organization));
        }

        if let Some(ref session_id) = self.session_id {
            key_parts.push(format!("session:{}", session_id));
        }

        if let Some(ref project_id) = self.project_id {
            key_parts.push(format!("project:{}", project_id));
        }

        if let Some(column_count) = self.column_count {
            key_parts.push(format!("cols:{}", column_count));
        }

        // Add sorted metadata keys for consistent cache keys
        let mut metadata_keys: Vec<_> = self.metadata.keys().collect();
        metadata_keys.sort();
        for key in metadata_keys {
            if let Some(value) = self.metadata.get(key) {
                key_parts.push(format!("meta:{}:{}", key, value));
            }
        }

        key_parts.join("|")
    }

    /// Create a minimal context for testing
    #[cfg(test)]
    pub fn minimal(document_type: &str) -> Self {
        Self::new(document_type.to_string())
    }

    /// Create a context with common test data
    #[cfg(test)]
    pub fn test_context() -> Self {
        Self::new("test_document".to_string())
            .with_file_name("test.xlsx".to_string())
            .with_user_id("test_user".to_string())
            .with_organization("test_org".to_string())
            .with_column_count(10)
    }
}

/// Builder for constructing override contexts
#[derive(Debug)]
pub struct OverrideContextBuilder {
    context: OverrideContext,
}

impl OverrideContextBuilder {
    /// Create a new builder
    pub fn new(document_type: String) -> Self {
        Self {
            context: OverrideContext::new(document_type),
        }
    }

    /// Set the file name
    pub fn file_name(mut self, file_name: String) -> Self {
        self.context.file_name = Some(file_name);
        self
    }

    /// Set the user ID
    pub fn user_id(mut self, user_id: String) -> Self {
        self.context.user_id = Some(user_id);
        self
    }

    /// Set the organization
    pub fn organization(mut self, organization: String) -> Self {
        self.context.organization = Some(organization);
        self
    }

    /// Set the session ID
    pub fn session_id(mut self, session_id: String) -> Self {
        self.context.session_id = Some(session_id);
        self
    }

    /// Set the project ID
    pub fn project_id(mut self, project_id: String) -> Self {
        self.context.project_id = Some(project_id);
        self
    }

    /// Set the column count
    pub fn column_count(mut self, column_count: usize) -> Self {
        self.context.column_count = Some(column_count);
        self
    }

    /// Set sample data
    pub fn sample_data(mut self, sample_data: Vec<serde_json::Value>) -> Self {
        self.context.sample_data = Some(sample_data);
        self
    }

    /// Add metadata entry
    pub fn metadata(mut self, key: String, value: serde_json::Value) -> Self {
        self.context.metadata.insert(key, value);
        self
    }

    /// Add multiple metadata entries
    pub fn metadata_map(mut self, metadata: HashMap<String, serde_json::Value>) -> Self {
        self.context.metadata.extend(metadata);
        self
    }

    /// Build the context
    pub fn build(self) -> OverrideContext {
        self.context
    }

    /// Build and validate the context
    pub fn build_validated(self) -> Result<OverrideContext, String> {
        let context = self.context;
        context.validate()?;
        Ok(context)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_context_creation() {
        let context = OverrideContext::new("test_doc".to_string());
        assert_eq!(context.document_type, "test_doc");
        assert!(context.file_name.is_none());
        assert!(context.metadata.is_empty());
    }

    #[test]
    fn test_context_builder() {
        let context = OverrideContext::builder("test_doc".to_string())
            .file_name("test.xlsx".to_string())
            .user_id("user123".to_string())
            .organization("org456".to_string())
            .column_count(5)
            .metadata("key1".to_string(), serde_json::Value::String("value1".to_string()))
            .build();

        assert_eq!(context.document_type, "test_doc");
        assert_eq!(context.file_name, Some("test.xlsx".to_string()));
        assert_eq!(context.user_id, Some("user123".to_string()));
        assert_eq!(context.organization, Some("org456".to_string()));
        assert_eq!(context.column_count, Some(5));
        assert_eq!(context.metadata.len(), 1);
    }

    #[test]
    fn test_context_validation() {
        let valid_context = OverrideContext::new("test_doc".to_string());
        assert!(valid_context.validate().is_ok());

        let invalid_context = OverrideContext::new("".to_string());
        assert!(invalid_context.validate().is_err());
    }

    #[test]
    fn test_cache_key_generation() {
        let context = OverrideContext::new("test_doc".to_string())
            .with_file_name("test.xlsx".to_string())
            .with_user_id("user123".to_string());

        let cache_key = context.cache_key();
        assert!(cache_key.contains("doc:test_doc"));
        assert!(cache_key.contains("file:test.xlsx"));
        assert!(cache_key.contains("user:user123"));
    }
}
