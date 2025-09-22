// Modified: 2025-09-22

//! Override context management
//!
//! This module provides functionality for managing override contexts,
//! which contain the information needed to evaluate override conditions.

use super::types::OverrideContext;
use std::collections::HashMap;

impl OverrideContext {
    /// Create a new override context
    pub fn new(document_type: String) -> Self {
        Self {
            document_type,
            file_name: None,
            headers: Vec::new(),
            row_count: 0,
            file_size: 0,
            timestamp: chrono::Utc::now(),
            user_id: None,
            metadata: HashMap::new(),
        }
    }

    /// Set file name
    pub fn with_file_name(mut self, file_name: String) -> Self {
        self.file_name = Some(file_name);
        self
    }

    /// Set headers
    pub fn with_headers(mut self, headers: Vec<String>) -> Self {
        self.headers = headers;
        self
    }

    /// Set row count
    pub fn with_row_count(mut self, row_count: usize) -> Self {
        self.row_count = row_count;
        self
    }

    /// Set file size
    pub fn with_file_size(mut self, file_size: u64) -> Self {
        self.file_size = file_size;
        self
    }

    /// Set user ID
    pub fn with_user_id(mut self, user_id: String) -> Self {
        self.user_id = Some(user_id);
        self
    }

    /// Set timestamp
    pub fn with_timestamp(mut self, timestamp: chrono::DateTime<chrono::Utc>) -> Self {
        self.timestamp = timestamp;
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

    /// Check if metadata contains key
    pub fn has_metadata(&self, key: &str) -> bool {
        self.metadata.contains_key(key)
    }

    /// Get all metadata keys
    pub fn metadata_keys(&self) -> Vec<&String> {
        self.metadata.keys().collect()
    }

    /// Get the number of headers
    pub fn header_count(&self) -> usize {
        self.headers.len()
    }

    /// Check if a header exists
    pub fn has_header(&self, header: &str) -> bool {
        self.headers.iter().any(|h| h == header)
    }

    /// Get header at index
    pub fn get_header(&self, index: usize) -> Option<&String> {
        self.headers.get(index)
    }

    /// Find header index by name (case-insensitive)
    pub fn find_header_index(&self, header: &str) -> Option<usize> {
        let header_lower = header.to_lowercase();
        self.headers.iter().position(|h| h.to_lowercase() == header_lower)
    }

    /// Get file extension from file name
    pub fn get_file_extension(&self) -> Option<String> {
        self.file_name.as_ref().and_then(|name| {
            name.rfind('.').map(|pos| name[pos + 1..].to_lowercase())
        })
    }

    /// Check if file name matches pattern
    pub fn file_name_matches(&self, pattern: &str) -> bool {
        if let Some(file_name) = &self.file_name {
            file_name.contains(pattern)
        } else {
            false
        }
    }

    /// Get document age in days
    pub fn get_document_age_days(&self) -> i64 {
        let now = chrono::Utc::now();
        (now - self.timestamp).num_days()
    }

    /// Check if document is recent (within specified days)
    pub fn is_recent(&self, days: i64) -> bool {
        self.get_document_age_days() <= days
    }

    /// Get file size in MB
    pub fn get_file_size_mb(&self) -> f64 {
        self.file_size as f64 / (1024.0 * 1024.0)
    }

    /// Check if file is large (above specified MB threshold)
    pub fn is_large_file(&self, mb_threshold: f64) -> bool {
        self.get_file_size_mb() > mb_threshold
    }

    /// Create a summary string for logging/debugging
    pub fn summary(&self) -> String {
        format!(
            "OverrideContext {{ doc_type: {}, file: {:?}, headers: {}, rows: {}, size: {} bytes, user: {:?} }}",
            self.document_type,
            self.file_name,
            self.headers.len(),
            self.row_count,
            self.file_size,
            self.user_id
        )
    }

    /// Validate the context for completeness
    pub fn validate(&self) -> Result<(), String> {
        if self.document_type.is_empty() {
            return Err("Document type cannot be empty".to_string());
        }

        if self.headers.is_empty() {
            return Err("Headers cannot be empty".to_string());
        }

        if self.row_count == 0 {
            return Err("Row count must be greater than 0".to_string());
        }

        Ok(())
    }

    /// Create a minimal context for testing
    pub fn minimal(document_type: &str) -> Self {
        Self::new(document_type.to_string())
            .with_headers(vec!["Column1".to_string(), "Column2".to_string()])
            .with_row_count(1)
    }

    /// Create a context from document metadata
    pub fn from_document_metadata(
        document_type: String,
        file_name: Option<String>,
        headers: Vec<String>,
        row_count: usize,
        file_size: u64,
    ) -> Self {
        Self::new(document_type)
            .with_file_name(file_name.unwrap_or_default())
            .with_headers(headers)
            .with_row_count(row_count)
            .with_file_size(file_size)
    }

    /// Clone with updated document type
    pub fn with_document_type(&self, document_type: String) -> Self {
        let mut context = self.clone();
        context.document_type = document_type;
        context
    }

    /// Clone with updated headers
    pub fn with_updated_headers(&self, headers: Vec<String>) -> Self {
        let mut context = self.clone();
        context.headers = headers;
        context
    }

    /// Clone with additional header
    pub fn with_additional_header(&self, header: String) -> Self {
        let mut context = self.clone();
        context.headers.push(header);
        context
    }

    /// Check if context matches another context for caching purposes
    pub fn cache_key_matches(&self, other: &Self) -> bool {
        self.document_type == other.document_type &&
        self.file_name == other.file_name &&
        self.headers == other.headers &&
        self.row_count == other.row_count
    }

    /// Generate a cache key for this context
    pub fn cache_key(&self) -> String {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let mut hasher = DefaultHasher::new();
        self.document_type.hash(&mut hasher);
        self.file_name.hash(&mut hasher);
        self.headers.hash(&mut hasher);
        self.row_count.hash(&mut hasher);
        
        format!("ctx_{:x}", hasher.finish())
    }
}
