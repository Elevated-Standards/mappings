// Modified: 2025-01-20

//! Common type definitions for FedRAMP compliance automation.
//!
//! This module provides shared type definitions, enums, and constants
//! used throughout the FedRAMP compliance automation platform.

use serde::{Deserialize, Serialize};
use std::fmt;

/// OSCAL version compatibility
pub const OSCAL_VERSION: &str = "1.1.2";

/// FedRAMP template versions
pub const FEDRAMP_TEMPLATE_VERSION: &str = "2023-06-30";

/// Risk levels as defined by FIPS 199
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum RiskLevel {
    /// Low impact
    Low,
    /// Moderate impact
    Moderate,
    /// High impact
    High,
}

impl fmt::Display for RiskLevel {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            RiskLevel::Low => write!(f, "low"),
            RiskLevel::Moderate => write!(f, "moderate"),
            RiskLevel::High => write!(f, "high"),
        }
    }
}

/// Security categorization for information systems
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SecurityCategorization {
    /// Confidentiality impact level
    pub confidentiality: RiskLevel,
    /// Integrity impact level
    pub integrity: RiskLevel,
    /// Availability impact level
    pub availability: RiskLevel,
}

impl SecurityCategorization {
    /// Get the overall system impact level (highest of the three categories)
    pub fn overall_impact(&self) -> RiskLevel {
        use RiskLevel::*;
        match (self.confidentiality, self.integrity, self.availability) {
            (High, _, _) | (_, High, _) | (_, _, High) => High,
            (Moderate, _, _) | (_, Moderate, _) | (_, _, Moderate) => Moderate,
            (Low, Low, Low) => Low,
        }
    }
}

/// FedRAMP authorization boundary types
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum AuthorizationBoundary {
    /// Cloud Service Provider (CSP) boundary
    CloudServiceProvider,
    /// Agency boundary
    Agency,
    /// Hybrid boundary
    Hybrid,
}

/// FedRAMP service models
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum ServiceModel {
    /// Software as a Service
    SaaS,
    /// Platform as a Service
    PaaS,
    /// Infrastructure as a Service
    IaaS,
}

/// FedRAMP deployment models
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum DeploymentModel {
    /// Public cloud
    Public,
    /// Private cloud
    Private,
    /// Community cloud
    Community,
    /// Hybrid cloud
    Hybrid,
}

/// Common identifier type for entities
pub type EntityId = uuid::Uuid;

/// Common timestamp type
pub type Timestamp = chrono::DateTime<chrono::Utc>;

/// Common result type for the platform
pub type Result<T> = std::result::Result<T, crate::error::Error>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_security_categorization_overall_impact() {
        let cat = SecurityCategorization {
            confidentiality: RiskLevel::High,
            integrity: RiskLevel::Moderate,
            availability: RiskLevel::Low,
        };
        assert_eq!(cat.overall_impact(), RiskLevel::High);

        let cat = SecurityCategorization {
            confidentiality: RiskLevel::Moderate,
            integrity: RiskLevel::Moderate,
            availability: RiskLevel::Low,
        };
        assert_eq!(cat.overall_impact(), RiskLevel::Moderate);

        let cat = SecurityCategorization {
            confidentiality: RiskLevel::Low,
            integrity: RiskLevel::Low,
            availability: RiskLevel::Low,
        };
        assert_eq!(cat.overall_impact(), RiskLevel::Low);
    }

    #[test]
    fn test_risk_level_display() {
        assert_eq!(RiskLevel::Low.to_string(), "low");
        assert_eq!(RiskLevel::Moderate.to_string(), "moderate");
        assert_eq!(RiskLevel::High.to_string(), "high");
    }
}
