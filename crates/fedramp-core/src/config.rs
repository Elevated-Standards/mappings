// Modified: 2025-01-20

//! Configuration management for FedRAMP compliance automation.
//!
//! This module provides configuration structures and utilities for
//! managing platform settings and environment-specific configurations.

use crate::error::Error;
use crate::types::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::env;
use std::path::Path;

/// Main configuration structure for the FedRAMP platform
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FedRampConfig {
    /// Database configuration
    pub database: DatabaseConfig,
    /// API server configuration
    pub server: ServerConfig,
    /// Security configuration
    pub security: SecurityConfig,
    /// Logging configuration
    pub logging: LoggingConfig,
    /// FedRAMP-specific settings
    pub fedramp: FedRampSettings,
}

/// Database configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseConfig {
    /// Database URL
    pub url: String,
    /// Maximum number of connections in the pool
    pub max_connections: u32,
    /// Connection timeout in seconds
    pub timeout_seconds: u64,
    /// Enable SSL/TLS
    pub ssl: bool,
}

/// API server configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerConfig {
    /// Server host address
    pub host: String,
    /// Server port
    pub port: u16,
    /// Enable CORS
    pub cors: bool,
    /// Request timeout in seconds
    pub timeout_seconds: u64,
    /// Maximum request body size in bytes
    pub max_body_size: usize,
}

/// Security configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityConfig {
    /// JWT secret key
    pub jwt_secret: String,
    /// JWT expiration time in hours
    pub jwt_expiration_hours: u64,
    /// Enable rate limiting
    pub rate_limiting: bool,
    /// Rate limit requests per minute
    pub rate_limit_rpm: u32,
    /// Enable audit logging
    pub audit_logging: bool,
}

/// Logging configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoggingConfig {
    /// Log level (trace, debug, info, warn, error)
    pub level: String,
    /// Log format (json, pretty)
    pub format: String,
    /// Enable file logging
    pub file_logging: bool,
    /// Log file path
    pub file_path: Option<String>,
}

/// FedRAMP-specific settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FedRampSettings {
    /// OSCAL version to use
    pub oscal_version: String,
    /// FedRAMP template version
    pub template_version: String,
    /// Default security categorization
    pub default_categorization: String,
    /// Validation strictness level
    pub validation_strict: bool,
    /// Document templates directory
    pub templates_dir: String,
    /// Output directory for generated documents
    pub output_dir: String,
}

impl Default for FedRampConfig {
    fn default() -> Self {
        Self {
            database: DatabaseConfig::default(),
            server: ServerConfig::default(),
            security: SecurityConfig::default(),
            logging: LoggingConfig::default(),
            fedramp: FedRampSettings::default(),
        }
    }
}

impl Default for DatabaseConfig {
    fn default() -> Self {
        Self {
            url: env::var("DATABASE_URL")
                .unwrap_or_else(|_| "postgresql://localhost/fedramp".to_string()),
            max_connections: 10,
            timeout_seconds: 30,
            ssl: true,
        }
    }
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            host: env::var("HOST").unwrap_or_else(|_| "0.0.0.0".to_string()),
            port: env::var("PORT")
                .unwrap_or_else(|_| "8080".to_string())
                .parse()
                .unwrap_or(8080),
            cors: true,
            timeout_seconds: 60,
            max_body_size: 16 * 1024 * 1024, // 16MB
        }
    }
}

impl Default for SecurityConfig {
    fn default() -> Self {
        Self {
            jwt_secret: env::var("JWT_SECRET")
                .unwrap_or_else(|_| "your-secret-key".to_string()),
            jwt_expiration_hours: 24,
            rate_limiting: true,
            rate_limit_rpm: 100,
            audit_logging: true,
        }
    }
}

impl Default for LoggingConfig {
    fn default() -> Self {
        Self {
            level: env::var("LOG_LEVEL").unwrap_or_else(|_| "info".to_string()),
            format: env::var("LOG_FORMAT").unwrap_or_else(|_| "json".to_string()),
            file_logging: false,
            file_path: None,
        }
    }
}

impl Default for FedRampSettings {
    fn default() -> Self {
        Self {
            oscal_version: crate::types::OSCAL_VERSION.to_string(),
            template_version: crate::types::FEDRAMP_TEMPLATE_VERSION.to_string(),
            default_categorization: "moderate".to_string(),
            validation_strict: false,
            templates_dir: "./templates".to_string(),
            output_dir: "./output".to_string(),
        }
    }
}

impl FedRampConfig {
    /// Load configuration from environment variables
    pub fn from_env() -> Result<Self> {
        Ok(Self::default())
    }

    /// Load configuration from a TOML file
    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Self> {
        let content = std::fs::read_to_string(path).map_err(|e| Error::configuration(format!("Failed to read config file: {}", e)))?;

        serde_json::from_str(&content).map_err(|e| Error::configuration(format!("Failed to parse config file: {}", e)))
    }

    /// Validate the configuration
    pub fn validate(&self) -> Result<()> {
        // Validate database URL
        if self.database.url.is_empty() {
            return Err(Error::configuration("Database URL cannot be empty"));
        }

        // Validate server port
        if self.server.port == 0 {
            return Err(Error::configuration("Server port must be greater than 0"));
        }

        // Validate JWT secret
        if self.security.jwt_secret.is_empty() {
            return Err(Error::configuration("JWT secret cannot be empty"));
        }

        // Validate log level
        let valid_levels = ["trace", "debug", "info", "warn", "error"];
        if !valid_levels.contains(&self.logging.level.as_str()) {
            return Err(Error::configuration(format!("Invalid log level: {}", self.logging.level)));
        }

        Ok(())
    }

    /// Get environment-specific configuration
    pub fn for_environment(env: &str) -> Result<Self> {
        let mut config = Self::from_env()?;

        match env {
            "development" => {
                config.logging.level = "debug".to_string();
                config.security.rate_limiting = false;
                config.fedramp.validation_strict = false;
            }
            "production" => {
                config.logging.level = "info".to_string();
                config.security.rate_limiting = true;
                config.fedramp.validation_strict = true;
            }
            "test" => {
                config.logging.level = "warn".to_string();
                config.database.url = "postgresql://localhost/fedramp_test".to_string();
                config.security.rate_limiting = false;
            }
            _ => {
                return Err(Error::configuration(format!("Unknown environment: {}", env)));
            }
        }

        config.validate()?;
        Ok(config)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = FedRampConfig::default();
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_environment_configs() {
        assert!(FedRampConfig::for_environment("development").is_ok());
        assert!(FedRampConfig::for_environment("production").is_ok());
        assert!(FedRampConfig::for_environment("test").is_ok());
        assert!(FedRampConfig::for_environment("invalid").is_err());
    }
}
