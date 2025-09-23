// Modified: 2025-09-23

//! Type definitions for inventory parsing and asset management

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::net::IpAddr;
use chrono::{DateTime, Utc};
use uuid::Uuid;

/// Complete inventory document structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InventoryDocument {
    /// Document metadata
    pub metadata: InventoryMetadata,
    /// List of all assets in the inventory
    pub assets: Vec<Asset>,
    /// Asset relationships and dependencies
    pub relationships: Vec<AssetRelationship>,
    /// Validation results for the inventory
    pub validation_results: InventoryValidationResults,
    /// Template information used for parsing
    pub template_info: InventoryTemplateInfo,
}

/// Metadata for inventory document
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InventoryMetadata {
    /// Type of inventory template used
    pub template_type: InventoryTemplateType,
    /// Version of the template
    pub template_version: String,
    /// Source file path
    pub source_file: String,
    /// Timestamp when parsed
    pub parsed_at: DateTime<Utc>,
    /// Total number of assets
    pub asset_count: usize,
    /// Total number of relationships
    pub relationship_count: usize,
}

/// Asset representation with comprehensive information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Asset {
    /// Unique asset identifier
    pub asset_id: String,
    /// Human-readable asset name
    pub asset_name: String,
    /// Type of asset (hardware, software, etc.)
    pub asset_type: AssetType,
    /// Category within the asset type
    pub asset_category: AssetCategory,
    /// Detailed description of the asset
    pub description: String,
    /// Asset owner or responsible party
    pub owner: String,
    /// Physical or logical location
    pub location: Option<String>,
    /// Environment where asset is deployed
    pub environment: Environment,
    /// Business criticality level
    pub criticality: Criticality,
    /// Network-specific information
    pub network_info: Option<NetworkInfo>,
    /// Software-specific information
    pub software_info: Option<SoftwareInfo>,
    /// Hardware-specific information
    pub hardware_info: Option<HardwareInfo>,
    /// Cloud-specific information
    pub cloud_info: Option<CloudInfo>,
    /// Asset relationships
    pub relationships: Vec<String>, // Asset IDs
    /// Compliance status
    pub compliance_status: ComplianceStatus,
    /// Custom attributes
    pub custom_attributes: HashMap<String, String>,
    /// Asset tags for categorization
    pub tags: Vec<String>,
    /// Creation timestamp
    pub created_at: Option<DateTime<Utc>>,
    /// Last update timestamp
    pub updated_at: Option<DateTime<Utc>>,
}

/// Asset type enumeration
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum AssetType {
    /// Physical hardware assets
    Hardware,
    /// Software applications and systems
    Software,
    /// Network infrastructure components
    Network,
    /// Virtual machines and containers
    Virtual,
    /// Data stores and repositories
    Data,
    /// Cloud services and resources
    Cloud,
    /// Business services
    Service,
}

/// Asset category for more specific classification
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum AssetCategory {
    // Hardware categories
    Server,
    Workstation,
    Laptop,
    MobileDevice,
    NetworkDevice,
    StorageDevice,
    SecurityDevice,
    
    // Software categories
    OperatingSystem,
    Application,
    Database,
    Middleware,
    SecuritySoftware,
    DevelopmentTool,
    
    // Network categories
    Router,
    Switch,
    Firewall,
    LoadBalancer,
    WirelessAccessPoint,
    
    // Virtual categories
    VirtualMachine,
    Container,
    VirtualNetwork,
    
    // Data categories
    FileSystem,
    DatabaseInstance,
    DataWarehouse,
    BackupStorage,
    
    // Cloud categories
    ComputeInstance,
    StorageService,
    DatabaseService,
    NetworkService,
    SecurityService,
    
    // Service categories
    WebService,
    APIService,
    BusinessService,
    
    // Generic
    Other,
}

/// Environment classification
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Environment {
    /// Production environment
    Production,
    /// Development environment
    Development,
    /// Testing environment
    Testing,
    /// Staging environment
    Staging,
    /// Training environment
    Training,
    /// Disaster recovery environment
    DisasterRecovery,
    /// Sandbox environment
    Sandbox,
}

/// Criticality levels for business impact
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum Criticality {
    /// Low business impact
    Low,
    /// Medium business impact
    Medium,
    /// High business impact
    High,
    /// Critical business impact
    Critical,
}

/// Network-specific asset information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkInfo {
    /// IP addresses assigned to the asset
    pub ip_addresses: Vec<IpAddr>,
    /// MAC addresses for network interfaces
    pub mac_addresses: Vec<String>,
    /// Network segments or VLANs
    pub network_segments: Vec<String>,
    /// Network ports and services
    pub ports: Vec<NetworkPort>,
    /// Supported protocols
    pub protocols: Vec<String>,
    /// DNS names
    pub dns_names: Vec<String>,
    /// Network interfaces
    pub interfaces: Vec<NetworkInterface>,
}

/// Network port information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkPort {
    /// Port number
    pub port: u16,
    /// Protocol (TCP, UDP, etc.)
    pub protocol: String,
    /// Service running on port
    pub service: Option<String>,
    /// Port state (open, closed, filtered)
    pub state: String,
}

/// Network interface information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkInterface {
    /// Interface name
    pub name: String,
    /// Interface type (ethernet, wireless, etc.)
    pub interface_type: String,
    /// MAC address
    pub mac_address: Option<String>,
    /// IP addresses
    pub ip_addresses: Vec<IpAddr>,
    /// Interface status
    pub status: String,
}

/// Software-specific asset information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SoftwareInfo {
    /// Software vendor
    pub vendor: String,
    /// Software version
    pub version: String,
    /// License information
    pub license: Option<LicenseInfo>,
    /// Installation path
    pub installation_path: Option<String>,
    /// Configuration details
    pub configuration: HashMap<String, String>,
    /// Dependencies
    pub dependencies: Vec<String>,
    /// Patch level
    pub patch_level: Option<String>,
    /// Support status
    pub support_status: SupportStatus,
}

/// License information for software
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LicenseInfo {
    /// License type
    pub license_type: String,
    /// License key or identifier
    pub license_key: Option<String>,
    /// Number of licenses
    pub license_count: Option<u32>,
    /// License expiration date
    pub expiration_date: Option<DateTime<Utc>>,
    /// License compliance status
    pub compliance_status: String,
}

/// Support status for software
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum SupportStatus {
    /// Actively supported
    Supported,
    /// Extended support
    ExtendedSupport,
    /// End of life
    EndOfLife,
    /// Unknown status
    Unknown,
}

/// Hardware-specific asset information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HardwareInfo {
    /// Hardware manufacturer
    pub manufacturer: String,
    /// Hardware model
    pub model: String,
    /// Serial number
    pub serial_number: Option<String>,
    /// Asset tag
    pub asset_tag: Option<String>,
    /// Hardware specifications
    pub specifications: HashMap<String, String>,
    /// Warranty information
    pub warranty: Option<WarrantyInfo>,
    /// Physical location details
    pub physical_location: Option<PhysicalLocation>,
}

/// Warranty information for hardware
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WarrantyInfo {
    /// Warranty provider
    pub provider: String,
    /// Warranty type
    pub warranty_type: String,
    /// Warranty start date
    pub start_date: DateTime<Utc>,
    /// Warranty end date
    pub end_date: DateTime<Utc>,
    /// Warranty status
    pub status: String,
}

/// Physical location information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PhysicalLocation {
    /// Building or facility
    pub building: String,
    /// Floor number
    pub floor: Option<String>,
    /// Room number
    pub room: Option<String>,
    /// Rack location
    pub rack: Option<String>,
    /// Rack unit position
    pub rack_unit: Option<String>,
}

/// Cloud-specific asset information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CloudInfo {
    /// Cloud provider
    pub provider: String,
    /// Cloud region
    pub region: String,
    /// Availability zone
    pub availability_zone: Option<String>,
    /// Instance type or size
    pub instance_type: Option<String>,
    /// Cloud resource ID
    pub resource_id: String,
    /// Resource tags
    pub resource_tags: HashMap<String, String>,
    /// Billing information
    pub billing_info: Option<BillingInfo>,
}

/// Billing information for cloud resources
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BillingInfo {
    /// Cost center
    pub cost_center: String,
    /// Monthly cost estimate
    pub monthly_cost: Option<f64>,
    /// Billing account
    pub billing_account: String,
}

/// Asset relationship information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssetRelationship {
    /// Unique relationship identifier
    pub id: String,
    /// Source asset ID
    pub source_asset_id: String,
    /// Target asset ID
    pub target_asset_id: String,
    /// Type of relationship
    pub relationship_type: RelationshipType,
    /// Relationship description
    pub description: Option<String>,
    /// Relationship strength or importance
    pub strength: RelationshipStrength,
    /// Custom attributes
    pub attributes: HashMap<String, String>,
}

/// Types of asset relationships
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum RelationshipType {
    /// Asset depends on another asset
    DependsOn,
    /// Asset hosts another asset
    Hosts,
    /// Asset is connected to another asset
    ConnectedTo,
    /// Asset manages another asset
    Manages,
    /// Asset monitors another asset
    Monitors,
    /// Asset backs up another asset
    BacksUp,
    /// Asset replicates another asset
    Replicates,
    /// Generic relationship
    Related,
}

/// Relationship strength levels
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum RelationshipStrength {
    /// Weak relationship
    Weak,
    /// Medium relationship
    Medium,
    /// Strong relationship
    Strong,
    /// Critical relationship
    Critical,
}

/// Compliance status for assets
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComplianceStatus {
    /// Overall compliance score (0.0 to 1.0)
    pub overall_score: f64,
    /// Compliance framework results
    pub framework_results: HashMap<String, ComplianceResult>,
    /// Last assessment date
    pub last_assessed: Option<DateTime<Utc>>,
    /// Next assessment due date
    pub next_assessment: Option<DateTime<Utc>>,
    /// Compliance notes
    pub notes: Vec<String>,
}

/// Compliance result for a specific framework
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComplianceResult {
    /// Framework name
    pub framework: String,
    /// Compliance status
    pub status: ComplianceStatusType,
    /// Score (0.0 to 1.0)
    pub score: f64,
    /// Findings or issues
    pub findings: Vec<String>,
    /// Remediation actions
    pub remediation_actions: Vec<String>,
}

/// Compliance status types
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ComplianceStatusType {
    /// Fully compliant
    Compliant,
    /// Partially compliant
    PartiallyCompliant,
    /// Non-compliant
    NonCompliant,
    /// Not assessed
    NotAssessed,
    /// Assessment in progress
    InProgress,
}

/// Inventory template information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InventoryTemplateInfo {
    /// Type of inventory template
    pub template_type: InventoryTemplateType,
    /// Template version
    pub version: String,
    /// Worksheets containing asset data
    pub asset_worksheets: Vec<String>,
    /// Worksheets containing relationship data
    pub relationship_worksheets: Vec<String>,
    /// Column mappings for each worksheet
    pub column_mappings: HashMap<String, HashMap<String, String>>,
}

/// Types of inventory templates
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum InventoryTemplateType {
    /// FedRAMP Integrated Inventory Workbook
    FedRampIntegrated,
    /// Network-focused inventory
    NetworkInventory,
    /// Software-focused inventory
    SoftwareInventory,
    /// Custom inventory template
    Custom,
}

/// Validation results for inventory processing
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InventoryValidationResults {
    /// Overall validation status
    pub is_valid: bool,
    /// Validation errors
    pub errors: Vec<ValidationError>,
    /// Validation warnings
    pub warnings: Vec<ValidationWarning>,
    /// Asset-specific validation results
    pub asset_results: HashMap<String, AssetValidationResult>,
    /// Relationship validation results
    pub relationship_results: Vec<RelationshipValidationResult>,
    /// Summary statistics
    pub summary: ValidationSummary,
}

/// Validation error information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationError {
    /// Error code
    pub code: String,
    /// Error message
    pub message: String,
    /// Asset ID if applicable
    pub asset_id: Option<String>,
    /// Field name if applicable
    pub field: Option<String>,
    /// Row number if applicable
    pub row: Option<usize>,
}

/// Validation warning information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationWarning {
    /// Warning code
    pub code: String,
    /// Warning message
    pub message: String,
    /// Asset ID if applicable
    pub asset_id: Option<String>,
    /// Field name if applicable
    pub field: Option<String>,
    /// Row number if applicable
    pub row: Option<usize>,
}

/// Asset-specific validation result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssetValidationResult {
    /// Asset ID
    pub asset_id: String,
    /// Validation status
    pub is_valid: bool,
    /// Field validation results
    pub field_results: HashMap<String, FieldValidationResult>,
    /// Asset-level errors
    pub errors: Vec<String>,
    /// Asset-level warnings
    pub warnings: Vec<String>,
}

/// Field validation result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FieldValidationResult {
    /// Field name
    pub field: String,
    /// Validation status
    pub is_valid: bool,
    /// Error message if invalid
    pub error: Option<String>,
    /// Warning message if applicable
    pub warning: Option<String>,
}

/// Relationship validation result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RelationshipValidationResult {
    /// Relationship ID
    pub relationship_id: String,
    /// Validation status
    pub is_valid: bool,
    /// Validation errors
    pub errors: Vec<String>,
    /// Validation warnings
    pub warnings: Vec<String>,
}

/// Validation summary statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationSummary {
    /// Total assets processed
    pub total_assets: usize,
    /// Valid assets
    pub valid_assets: usize,
    /// Invalid assets
    pub invalid_assets: usize,
    /// Total relationships processed
    pub total_relationships: usize,
    /// Valid relationships
    pub valid_relationships: usize,
    /// Invalid relationships
    pub invalid_relationships: usize,
    /// Total errors
    pub total_errors: usize,
    /// Total warnings
    pub total_warnings: usize,
}

impl Default for InventoryValidationResults {
    fn default() -> Self {
        Self {
            is_valid: true,
            errors: Vec::new(),
            warnings: Vec::new(),
            asset_results: HashMap::new(),
            relationship_results: Vec::new(),
            summary: ValidationSummary {
                total_assets: 0,
                valid_assets: 0,
                invalid_assets: 0,
                total_relationships: 0,
                valid_relationships: 0,
                invalid_relationships: 0,
                total_errors: 0,
                total_warnings: 0,
            },
        }
    }
}

impl Asset {
    /// Create a new asset with minimal required information
    pub fn new(asset_id: String, asset_name: String, asset_type: AssetType) -> Self {
        Self {
            asset_id,
            asset_name,
            asset_type,
            asset_category: AssetCategory::Other,
            description: String::new(),
            owner: String::new(),
            location: None,
            environment: Environment::Production,
            criticality: Criticality::Medium,
            network_info: None,
            software_info: None,
            hardware_info: None,
            cloud_info: None,
            relationships: Vec::new(),
            compliance_status: ComplianceStatus {
                overall_score: 0.0,
                framework_results: HashMap::new(),
                last_assessed: None,
                next_assessment: None,
                notes: Vec::new(),
            },
            custom_attributes: HashMap::new(),
            tags: Vec::new(),
            created_at: Some(Utc::now()),
            updated_at: Some(Utc::now()),
        }
    }

    /// Check if asset is critical
    pub fn is_critical(&self) -> bool {
        self.criticality == Criticality::Critical
    }

    /// Check if asset is in production environment
    pub fn is_production(&self) -> bool {
        self.environment == Environment::Production
    }

    /// Get asset type as string
    pub fn asset_type_string(&self) -> &'static str {
        match self.asset_type {
            AssetType::Hardware => "Hardware",
            AssetType::Software => "Software",
            AssetType::Network => "Network",
            AssetType::Virtual => "Virtual",
            AssetType::Data => "Data",
            AssetType::Cloud => "Cloud",
            AssetType::Service => "Service",
        }
    }
}
