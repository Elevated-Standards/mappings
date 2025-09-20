// Modified: 2025-09-20

//! Inventory models for FedRAMP compliance automation.
//!
//! This module defines data structures for system inventory management
//! including components, services, and infrastructure elements.

use crate::types::{EntityId, Result, RiskLevel, Timestamp};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;
use validator::Validate;

/// Component types in the system inventory
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum ComponentType {
    /// Software component
    Software,
    /// Hardware component
    Hardware,
    /// Service component
    Service,
    /// Network component
    Network,
    /// Database component
    Database,
    /// Operating system
    OperatingSystem,
    /// Virtual machine
    VirtualMachine,
    /// Container
    Container,
    /// Cloud service
    CloudService,
}

/// Component status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum ComponentStatus {
    /// Component is active and operational
    Active,
    /// Component is inactive
    Inactive,
    /// Component is under maintenance
    Maintenance,
    /// Component is deprecated
    Deprecated,
    /// Component is being decommissioned
    Decommissioning,
}

/// System component in the inventory
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct InventoryComponent {
    /// Unique component identifier
    pub id: EntityId,
    /// Component name
    #[validate(length(min = 1, max = 255))]
    pub name: String,
    /// Component type
    pub component_type: ComponentType,
    /// Component status
    pub status: ComponentStatus,
    /// Component description
    pub description: Option<String>,
    /// Component version
    pub version: Option<String>,
    /// Vendor/manufacturer
    pub vendor: Option<String>,
    /// Component location
    pub location: Option<String>,
    /// IP addresses associated with the component
    pub ip_addresses: Vec<String>,
    /// Hostnames associated with the component
    pub hostnames: Vec<String>,
    /// Ports used by the component
    pub ports: Vec<NetworkPort>,
    /// Security categorization
    pub security_categorization: Option<crate::types::SecurityCategorization>,
    /// Component properties
    pub properties: HashMap<String, String>,
    /// Component dependencies
    pub dependencies: Vec<ComponentDependency>,
    /// Responsible party
    pub responsible_party: Option<EntityId>,
    /// Creation timestamp
    pub created_at: Timestamp,
    /// Last update timestamp
    pub updated_at: Timestamp,
    /// Created by user ID
    pub created_by: EntityId,
    /// Last updated by user ID
    pub updated_by: EntityId,
}

/// Network port information
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct NetworkPort {
    /// Port number
    #[validate(range(min = 1, max = 65535))]
    pub port: u16,
    /// Protocol (TCP, UDP, etc.)
    #[validate(length(min = 1))]
    pub protocol: String,
    /// Port description/purpose
    pub description: Option<String>,
    /// Whether the port is publicly accessible
    pub public: bool,
}

/// Component dependency relationship
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComponentDependency {
    /// ID of the dependent component
    pub component_id: EntityId,
    /// Type of dependency
    pub dependency_type: DependencyType,
    /// Dependency description
    pub description: Option<String>,
    /// Whether this is a critical dependency
    pub critical: bool,
}

/// Types of component dependencies
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum DependencyType {
    /// Component requires another component to function
    Requires,
    /// Component provides services to another component
    Provides,
    /// Component communicates with another component
    Communicates,
    /// Component is hosted on another component
    HostedOn,
    /// Component hosts another component
    Hosts,
}

/// Software inventory item
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct SoftwareInventory {
    /// Unique software identifier
    pub id: EntityId,
    /// Software name
    #[validate(length(min = 1, max = 255))]
    pub name: String,
    /// Software version
    pub version: String,
    /// Software vendor
    pub vendor: Option<String>,
    /// Software license
    pub license: Option<String>,
    /// Installation date
    pub installed_date: Option<Timestamp>,
    /// Software category
    pub category: String,
    /// Software description
    pub description: Option<String>,
    /// Components where this software is installed
    pub installed_on: Vec<EntityId>,
    /// Software vulnerabilities
    pub vulnerabilities: Vec<SoftwareVulnerability>,
    /// Creation timestamp
    pub created_at: Timestamp,
    /// Last update timestamp
    pub updated_at: Timestamp,
}

/// Software vulnerability information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SoftwareVulnerability {
    /// CVE identifier
    pub cve_id: Option<String>,
    /// Vulnerability description
    pub description: String,
    /// Severity level
    pub severity: RiskLevel,
    /// CVSS score
    pub cvss_score: Option<f32>,
    /// Whether vulnerability is patched
    pub patched: bool,
    /// Patch date
    pub patch_date: Option<Timestamp>,
}

/// Network inventory item
#[derive(Debug, Clone, Serialize, Deserialize, Validate)]
pub struct NetworkInventory {
    /// Unique network identifier
    pub id: EntityId,
    /// Network name
    #[validate(length(min = 1, max = 255))]
    pub name: String,
    /// Network type (LAN, WAN, VPN, etc.)
    pub network_type: String,
    /// Network CIDR block
    pub cidr_block: String,
    /// VLAN ID
    pub vlan_id: Option<u16>,
    /// Network description
    pub description: Option<String>,
    /// Network security zone
    pub security_zone: String,
    /// Connected components
    pub connected_components: Vec<EntityId>,
    /// Network properties
    pub properties: HashMap<String, String>,
    /// Creation timestamp
    pub created_at: Timestamp,
    /// Last update timestamp
    pub updated_at: Timestamp,
}

impl InventoryComponent {
    /// Create a new inventory component
    pub fn new(
        name: String,
        component_type: ComponentType,
        created_by: EntityId,
    ) -> Self {
        let now = crate::utils::current_timestamp();
        let id = crate::utils::generate_uuid();

        Self {
            id,
            name,
            component_type,
            status: ComponentStatus::Active,
            description: None,
            version: None,
            vendor: None,
            location: None,
            ip_addresses: Vec::new(),
            hostnames: Vec::new(),
            ports: Vec::new(),
            security_categorization: None,
            properties: HashMap::new(),
            dependencies: Vec::new(),
            responsible_party: None,
            created_at: now,
            updated_at: now,
            created_by,
            updated_by: created_by,
        }
    }

    /// Add a network port to the component
    pub fn add_port(&mut self, port: NetworkPort) {
        self.ports.push(port);
        self.updated_at = crate::utils::current_timestamp();
    }

    /// Add a dependency to the component
    pub fn add_dependency(&mut self, dependency: ComponentDependency) {
        self.dependencies.push(dependency);
        self.updated_at = crate::utils::current_timestamp();
    }

    /// Check if component has public-facing ports
    pub fn has_public_ports(&self) -> bool {
        self.ports.iter().any(|port| port.public)
    }

    /// Get overall security impact level
    pub fn security_impact(&self) -> RiskLevel {
        self.security_categorization
            .as_ref()
            .map(|cat| cat.overall_impact())
            .unwrap_or(RiskLevel::Low)
    }
}

impl NetworkPort {
    /// Create a new network port
    pub fn new(port: u16, protocol: String, public: bool) -> Self {
        Self {
            port,
            protocol,
            description: None,
            public,
        }
    }

    /// Check if this is a well-known port
    pub fn is_well_known(&self) -> bool {
        self.port <= 1023
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_component_creation() {
        let user_id = Uuid::new_v4();
        let component = InventoryComponent::new(
            "Web Server".to_string(),
            ComponentType::Software,
            user_id,
        );

        assert_eq!(component.name, "Web Server");
        assert_eq!(component.component_type, ComponentType::Software);
        assert_eq!(component.status, ComponentStatus::Active);
        assert_eq!(component.created_by, user_id);
    }

    #[test]
    fn test_network_port() {
        let port = NetworkPort::new(80, "TCP".to_string(), true);
        assert_eq!(port.port, 80);
        assert_eq!(port.protocol, "TCP");
        assert!(port.public);
        assert!(port.is_well_known());

        let port = NetworkPort::new(8080, "TCP".to_string(), false);
        assert!(!port.is_well_known());
    }

    #[test]
    fn test_component_ports() {
        let user_id = Uuid::new_v4();
        let mut component = InventoryComponent::new(
            "Web Server".to_string(),
            ComponentType::Software,
            user_id,
        );

        let port = NetworkPort::new(443, "TCP".to_string(), true);
        component.add_port(port);

        assert!(component.has_public_ports());
        assert_eq!(component.ports.len(), 1);
    }
}
