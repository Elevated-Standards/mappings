// Modified: 2025-01-20

//! System models for FedRAMP compliance automation.
//!
//! This module defines data structures for information systems
//! and their security boundaries.

use crate::types::{
    AuthorizationBoundary, DeploymentModel, EntityId, Result, SecurityCategorization,
    ServiceModel, Timestamp,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;
use validator::Validate;

/// Information system definition
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct InformationSystem {
    /// Unique system identifier
    pub id: EntityId,
    /// System name
    #[validate(length(min = 1, max = 255))]
    pub name: String,
    /// System abbreviation/acronym
    pub abbreviation: Option<String>,
    /// System description
    #[validate(length(min = 1))]
    pub description: String,
    /// System version
    pub version: Option<String>,
    /// System status
    pub status: SystemStatus,
    /// System type
    pub system_type: SystemType,
    /// Service model (SaaS, PaaS, IaaS)
    pub service_model: ServiceModel,
    /// Deployment model (Public, Private, etc.)
    pub deployment_model: DeploymentModel,
    /// Authorization boundary
    pub authorization_boundary: AuthorizationBoundary,
    /// Security categorization
    pub security_categorization: SecurityCategorization,
    /// System owner
    pub system_owner: SystemContact,
    /// Authorizing official
    pub authorizing_official: SystemContact,
    /// System security officer
    pub system_security_officer: Option<SystemContact>,
    /// System components
    pub components: Vec<EntityId>,
    /// System boundaries
    pub boundaries: SystemBoundaries,
    /// System properties
    pub properties: HashMap<String, String>,
    /// Creation timestamp
    pub created_at: Timestamp,
    /// Last update timestamp
    pub updated_at: Timestamp,
    /// Created by user ID
    pub created_by: EntityId,
    /// Last updated by user ID
    pub updated_by: EntityId,
}

/// System status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum SystemStatus {
    /// System is in development
    Development,
    /// System is in testing
    Testing,
    /// System is operational
    Operational,
    /// System is under maintenance
    Maintenance,
    /// System is being decommissioned
    Decommissioning,
    /// System is retired
    Retired,
}

/// System type classification
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum SystemType {
    /// General support system
    GeneralSupport,
    /// Major application
    MajorApplication,
    /// Minor application
    MinorApplication,
    /// Cloud service
    CloudService,
    /// Hybrid system
    Hybrid,
}

/// System contact information
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct SystemContact {
    /// Contact name
    #[validate(length(min = 1, max = 255))]
    pub name: String,
    /// Contact title/role
    pub title: Option<String>,
    /// Organization
    pub organization: Option<String>,
    /// Email address
    #[validate(email)]
    pub email: String,
    /// Phone number
    pub phone: Option<String>,
    /// Address
    pub address: Option<ContactAddress>,
}

/// Contact address
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContactAddress {
    /// Street address
    pub street: String,
    /// City
    pub city: String,
    /// State/province
    pub state: String,
    /// Postal code
    pub postal_code: String,
    /// Country
    pub country: String,
}

/// System boundaries definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemBoundaries {
    /// Network boundaries
    pub network: Vec<NetworkBoundary>,
    /// Physical boundaries
    pub physical: Vec<PhysicalBoundary>,
    /// Logical boundaries
    pub logical: Vec<LogicalBoundary>,
    /// Data flow boundaries
    pub data_flow: Vec<DataFlowBoundary>,
}

/// Network boundary definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkBoundary {
    /// Boundary name
    pub name: String,
    /// Network segments included
    pub network_segments: Vec<String>,
    /// Security controls at this boundary
    pub security_controls: Vec<String>,
    /// Boundary description
    pub description: Option<String>,
}

/// Physical boundary definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PhysicalBoundary {
    /// Boundary name
    pub name: String,
    /// Physical locations included
    pub locations: Vec<String>,
    /// Physical security controls
    pub security_controls: Vec<String>,
    /// Boundary description
    pub description: Option<String>,
}

/// Logical boundary definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogicalBoundary {
    /// Boundary name
    pub name: String,
    /// Logical components included
    pub components: Vec<EntityId>,
    /// Access controls
    pub access_controls: Vec<String>,
    /// Boundary description
    pub description: Option<String>,
}

/// Data flow boundary definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataFlowBoundary {
    /// Boundary name
    pub name: String,
    /// Data types crossing this boundary
    pub data_types: Vec<String>,
    /// Data protection controls
    pub protection_controls: Vec<String>,
    /// Boundary description
    pub description: Option<String>,
}

/// System interconnection
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct SystemInterconnection {
    /// Interconnection ID
    pub id: EntityId,
    /// Source system ID
    pub source_system: EntityId,
    /// Target system ID
    pub target_system: EntityId,
    /// Interconnection type
    pub interconnection_type: InterconnectionType,
    /// Connection description
    #[validate(length(min = 1))]
    pub description: String,
    /// Data exchanged
    pub data_exchanged: Vec<String>,
    /// Security controls
    pub security_controls: Vec<String>,
    /// Connection status
    pub status: ConnectionStatus,
    /// Authorization date
    pub authorization_date: Option<Timestamp>,
    /// Expiration date
    pub expiration_date: Option<Timestamp>,
    /// Creation timestamp
    pub created_at: Timestamp,
    /// Last update timestamp
    pub updated_at: Timestamp,
}

/// Types of system interconnections
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum InterconnectionType {
    /// Direct network connection
    DirectConnection,
    /// VPN connection
    VpnConnection,
    /// API integration
    ApiIntegration,
    /// File transfer
    FileTransfer,
    /// Database connection
    DatabaseConnection,
    /// Web service
    WebService,
}

/// Connection status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum ConnectionStatus {
    /// Connection is active
    Active,
    /// Connection is inactive
    Inactive,
    /// Connection is pending approval
    PendingApproval,
    /// Connection is suspended
    Suspended,
    /// Connection is terminated
    Terminated,
}

impl InformationSystem {
    /// Create a new information system
    pub fn new(
        name: String,
        description: String,
        service_model: ServiceModel,
        deployment_model: DeploymentModel,
        security_categorization: SecurityCategorization,
        system_owner: SystemContact,
        authorizing_official: SystemContact,
        created_by: EntityId,
    ) -> Self {
        let now = crate::utils::current_timestamp();
        let id = crate::utils::generate_uuid();

        Self {
            id,
            name,
            abbreviation: None,
            description,
            version: None,
            status: SystemStatus::Development,
            system_type: SystemType::CloudService,
            service_model,
            deployment_model,
            authorization_boundary: AuthorizationBoundary::CloudServiceProvider,
            security_categorization,
            system_owner,
            authorizing_official,
            system_security_officer: None,
            components: Vec::new(),
            boundaries: SystemBoundaries::default(),
            properties: HashMap::new(),
            created_at: now,
            updated_at: now,
            created_by,
            updated_by: created_by,
        }
    }

    /// Add a component to the system
    pub fn add_component(&mut self, component_id: EntityId) {
        if !self.components.contains(&component_id) {
            self.components.push(component_id);
            self.updated_at = crate::utils::current_timestamp();
        }
    }

    /// Remove a component from the system
    pub fn remove_component(&mut self, component_id: &EntityId) {
        if let Some(pos) = self.components.iter().position(|id| id == component_id) {
            self.components.remove(pos);
            self.updated_at = crate::utils::current_timestamp();
        }
    }

    /// Update system status
    pub fn update_status(&mut self, status: SystemStatus, updated_by: EntityId) {
        self.status = status;
        self.updated_at = crate::utils::current_timestamp();
        self.updated_by = updated_by;
    }

    /// Check if system is operational
    pub fn is_operational(&self) -> bool {
        self.status == SystemStatus::Operational
    }

    /// Get overall security impact level
    pub fn security_impact(&self) -> crate::types::RiskLevel {
        self.security_categorization.overall_impact()
    }
}

impl Default for SystemBoundaries {
    fn default() -> Self {
        Self {
            network: Vec::new(),
            physical: Vec::new(),
            logical: Vec::new(),
            data_flow: Vec::new(),
        }
    }
}

impl SystemContact {
    /// Create a new system contact
    pub fn new(name: String, email: String) -> Self {
        Self {
            name,
            title: None,
            organization: None,
            email,
            phone: None,
            address: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::RiskLevel;

    #[test]
    fn test_system_creation() {
        let user_id = Uuid::new_v4();
        let owner = SystemContact::new("John Doe".to_string(), "john@example.com".to_string());
        let ao = SystemContact::new("Jane Smith".to_string(), "jane@example.com".to_string());
        let categorization = SecurityCategorization {
            confidentiality: RiskLevel::Moderate,
            integrity: RiskLevel::Moderate,
            availability: RiskLevel::Moderate,
        };

        let system = InformationSystem::new(
            "Test System".to_string(),
            "A test system for validation".to_string(),
            ServiceModel::SaaS,
            DeploymentModel::Public,
            categorization,
            owner,
            ao,
            user_id,
        );

        assert_eq!(system.name, "Test System");
        assert_eq!(system.service_model, ServiceModel::SaaS);
        assert_eq!(system.deployment_model, DeploymentModel::Public);
        assert_eq!(system.status, SystemStatus::Development);
        assert!(!system.is_operational());
        assert_eq!(system.security_impact(), RiskLevel::Moderate);
    }

    #[test]
    fn test_component_management() {
        let user_id = Uuid::new_v4();
        let owner = SystemContact::new("John Doe".to_string(), "john@example.com".to_string());
        let ao = SystemContact::new("Jane Smith".to_string(), "jane@example.com".to_string());
        let categorization = SecurityCategorization {
            confidentiality: RiskLevel::Low,
            integrity: RiskLevel::Low,
            availability: RiskLevel::Low,
        };

        let mut system = InformationSystem::new(
            "Test System".to_string(),
            "A test system".to_string(),
            ServiceModel::PaaS,
            DeploymentModel::Private,
            categorization,
            owner,
            ao,
            user_id,
        );

        let component_id = Uuid::new_v4();
        system.add_component(component_id);
        assert_eq!(system.components.len(), 1);
        assert!(system.components.contains(&component_id));

        system.remove_component(&component_id);
        assert_eq!(system.components.len(), 0);
    }
}
