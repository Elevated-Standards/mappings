// Modified: 2025-09-20

//! Error types for the FedRAMP compliance platform

use thiserror::Error;

/// Main error type for the FedRAMP platform
#[derive(Error, Debug)]
pub enum Error {
    #[error("Validation error: {message}")]
    Validation { message: String },

    #[error("Configuration error: {message}")]
    Configuration { message: String },

    #[error("Database error: {source}")]
    Database {
        #[from]
        source: sqlx::Error,
    },

    #[error("Serialization error: {source}")]
    Serialization {
        #[from]
        source: serde_json::Error,
    },

    #[error("IO error: {source}")]
    Io {
        #[from]
        source: std::io::Error,
    },

    #[error("Document parsing error: {message}")]
    DocumentParsing { message: String },

    #[error("OSCAL validation error: {message}")]
    OscalValidation { message: String },

    #[error("Control mapping error: {message}")]
    ControlMapping { message: String },

    #[error("Framework conversion error: {message}")]
    FrameworkConversion { message: String },

    #[error("Authentication error: {message}")]
    Authentication { message: String },

    #[error("Authorization error: {message}")]
    Authorization { message: String },

    #[error("Not found: {resource}")]
    NotFound { resource: String },

    #[error("Conflict: {message}")]
    Conflict { message: String },

    #[error("Internal server error: {message}")]
    Internal { message: String },

    #[error("External service error: {service}: {message}")]
    ExternalService { service: String, message: String },

    #[error("Rate limit exceeded")]
    RateLimit,

    #[error("Timeout error: {operation}")]
    Timeout { operation: String },

    #[error("Unknown error: {message}")]
    Unknown { message: String },
}

impl Error {
    /// Create a validation error
    pub fn validation(message: impl Into<String>) -> Self {
        Self::Validation {
            message: message.into(),
        }
    }

    /// Create a configuration error
    pub fn configuration(message: impl Into<String>) -> Self {
        Self::Configuration {
            message: message.into(),
        }
    }

    /// Create a document parsing error
    pub fn document_parsing(message: impl Into<String>) -> Self {
        Self::DocumentParsing {
            message: message.into(),
        }
    }

    /// Create an OSCAL validation error
    pub fn oscal_validation(message: impl Into<String>) -> Self {
        Self::OscalValidation {
            message: message.into(),
        }
    }

    /// Create a control mapping error
    pub fn control_mapping(message: impl Into<String>) -> Self {
        Self::ControlMapping {
            message: message.into(),
        }
    }

    /// Create a framework conversion error
    pub fn framework_conversion(message: impl Into<String>) -> Self {
        Self::FrameworkConversion {
            message: message.into(),
        }
    }

    /// Create an authentication error
    pub fn authentication(message: impl Into<String>) -> Self {
        Self::Authentication {
            message: message.into(),
        }
    }

    /// Create an authorization error
    pub fn authorization(message: impl Into<String>) -> Self {
        Self::Authorization {
            message: message.into(),
        }
    }

    /// Create a not found error
    pub fn not_found(resource: impl Into<String>) -> Self {
        Self::NotFound {
            resource: resource.into(),
        }
    }

    /// Create a conflict error
    pub fn conflict(message: impl Into<String>) -> Self {
        Self::Conflict {
            message: message.into(),
        }
    }

    /// Create an internal error
    pub fn internal(message: impl Into<String>) -> Self {
        Self::Internal {
            message: message.into(),
        }
    }

    /// Create an external service error
    pub fn external_service(service: impl Into<String>, message: impl Into<String>) -> Self {
        Self::ExternalService {
            service: service.into(),
            message: message.into(),
        }
    }

    /// Create a timeout error
    pub fn timeout(operation: impl Into<String>) -> Self {
        Self::Timeout {
            operation: operation.into(),
        }
    }

    /// Create an unknown error
    pub fn unknown(message: impl Into<String>) -> Self {
        Self::Unknown {
            message: message.into(),
        }
    }

    /// Check if this is a client error (4xx)
    pub fn is_client_error(&self) -> bool {
        matches!(
            self,
            Self::Validation { .. }
                | Self::Authentication { .. }
                | Self::Authorization { .. }
                | Self::NotFound { .. }
                | Self::Conflict { .. }
                | Self::RateLimit
        )
    }

    /// Check if this is a server error (5xx)
    pub fn is_server_error(&self) -> bool {
        matches!(
            self,
            Self::Database { .. }
                | Self::Internal { .. }
                | Self::ExternalService { .. }
                | Self::Timeout { .. }
                | Self::Unknown { .. }
        )
    }

    /// Get HTTP status code for this error
    pub fn status_code(&self) -> u16 {
        match self {
            Self::Validation { .. } => 400,
            Self::Authentication { .. } => 401,
            Self::Authorization { .. } => 403,
            Self::NotFound { .. } => 404,
            Self::Conflict { .. } => 409,
            Self::RateLimit => 429,
            Self::Database { .. }
            | Self::Internal { .. }
            | Self::ExternalService { .. }
            | Self::Timeout { .. }
            | Self::Unknown { .. } => 500,
            _ => 500,
        }
    }
}

/// Result type alias for the FedRAMP platform
pub type Result<T> = std::result::Result<T, Error>;
