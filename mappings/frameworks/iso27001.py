"""
ISO 27001 Framework Definition
Information Security Management System
"""

from ..core.models import SecurityFramework, SecurityDomain, SecurityControl, RiskLevel, ControlType


def create_iso27001_framework() -> SecurityFramework:
    """Create ISO 27001 framework definition."""
    framework = SecurityFramework(
        id="iso27001",
        name="ISO 27001",
        version="2022",
        description="Information Security Management System - Requirements for establishing, implementing, maintaining and continually improving an information security management system"
    )

    # ISO 27001 Annex A Control Domains
    domains = [
        SecurityDomain(
            id="A.5",
            name="Information Security Policies",
            description="Organizational information security",
            framework_id="iso27001"
        ),
        SecurityDomain(
            id="A.6",
            name="Organization of Information Security",
            description="Internal organization and mobile devices",
            framework_id="iso27001"
        ),
        SecurityDomain(
            id="A.7",
            name="Human Resource Security",
            description="Personnel security controls",
            framework_id="iso27001"
        ),
        SecurityDomain(
            id="A.8",
            name="Asset Management",
            description="Asset responsibility and information classification",
            framework_id="iso27001"
        ),
        SecurityDomain(
            id="A.9",
            name="Access Control",
            description="Business requirements for access control",
            framework_id="iso27001"
        ),
        SecurityDomain(
            id="A.10",
            name="Cryptography",
            description="Cryptographic controls",
            framework_id="iso27001"
        ),
        SecurityDomain(
            id="A.11",
            name="Physical and Environmental Security",
            description="Secure areas and equipment protection",
            framework_id="iso27001"
        ),
        SecurityDomain(
            id="A.12",
            name="Operations Security",
            description="Operational procedures and responsibilities",
            framework_id="iso27001"
        )
    ]

    for domain in domains:
        framework.add_domain(domain)

    # Key ISO 27001 Controls
    controls = [
        # Information Security Policies
        SecurityControl(
            id="A.5.1.1",
            title="Information Security Policy",
            description="An information security policy shall be defined, approved by management, published and communicated to employees and relevant external parties",
            framework_id="iso27001",
            domain_id="A.5",
            risk_level=RiskLevel.CRITICAL,
            control_type=ControlType.PROCEDURAL,
            tags={"iso27001", "isms", "information-security", "policy"}
        ),
        SecurityControl(
            id="A.5.1.2",
            title="Review of Information Security Policy",
            description="The information security policy shall be reviewed at planned intervals or if significant changes occur",
            framework_id="iso27001",
            domain_id="A.5",
            risk_level=RiskLevel.HIGH,
            control_type=ControlType.PROCEDURAL,
            tags={"iso27001", "isms", "information-security", "policy"}
        ),
        # Organization of Information Security
        SecurityControl(
            id="A.6.1.1",
            title="Information Security Roles and Responsibilities",
            description="All information security responsibilities shall be defined and allocated",
            framework_id="iso27001",
            domain_id="A.6",
            risk_level=RiskLevel.HIGH,
            control_type=ControlType.PROCEDURAL,
            tags={"iso27001", "isms", "information-security", "roles"}
        ),
        SecurityControl(
            id="A.6.2.1",
            title="Mobile Device Policy",
            description="A policy and supporting security measures shall be adopted to manage the risks introduced by using mobile devices",
            framework_id="iso27001",
            domain_id="A.6",
            risk_level=RiskLevel.MEDIUM,
            control_type=ControlType.PROCEDURAL,
            tags={"iso27001", "isms", "information-security", "mobile"}
        ),
        # Human Resource Security
        SecurityControl(
            id="A.7.1.1",
            title="Screening",
            description="Background verification checks on all candidates for employment shall be carried out in accordance with relevant laws, regulations and ethics",
            framework_id="iso27001",
            domain_id="A.7",
            risk_level=RiskLevel.MEDIUM,
            control_type=ControlType.PROCEDURAL,
            tags={"iso27001", "isms", "information-security", "hr"}
        ),
        SecurityControl(
            id="A.7.2.2",
            title="Information Security Awareness, Education and Training",
            description="All employees of the organization and, where relevant, contractors shall receive appropriate awareness education and training",
            framework_id="iso27001",
            domain_id="A.7",
            risk_level=RiskLevel.MEDIUM,
            control_type=ControlType.PROCEDURAL,
            tags={"iso27001", "isms", "information-security", "training"}
        ),
        # Asset Management
        SecurityControl(
            id="A.8.1.1",
            title="Inventory of Assets",
            description="Assets associated with information and information processing facilities shall be identified",
            framework_id="iso27001",
            domain_id="A.8",
            risk_level=RiskLevel.HIGH,
            control_type=ControlType.PROCEDURAL,
            tags={"iso27001", "isms", "information-security", "assets"}
        ),
        SecurityControl(
            id="A.8.2.1",
            title="Classification of Information",
            description="Information shall be classified in terms of legal requirements, value, criticality and sensitivity",
            framework_id="iso27001",
            domain_id="A.8",
            risk_level=RiskLevel.HIGH,
            control_type=ControlType.PROCEDURAL,
            tags={"iso27001", "isms", "information-security", "classification"}
        ),
        # Access Control
        SecurityControl(
            id="A.9.1.1",
            title="Access Control Policy",
            description="An access control policy shall be established, documented and reviewed based on business and information security requirements",
            framework_id="iso27001",
            domain_id="A.9",
            risk_level=RiskLevel.CRITICAL,
            control_type=ControlType.PROCEDURAL,
            tags={"iso27001", "isms", "information-security", "access-control"}
        ),
        SecurityControl(
            id="A.9.2.1",
            title="User Registration and De-registration",
            description="A formal user registration and de-registration process shall be implemented to enable assignment of access rights",
            framework_id="iso27001",
            domain_id="A.9",
            risk_level=RiskLevel.CRITICAL,
            control_type=ControlType.TECHNICAL,
            tags={"iso27001", "isms", "information-security", "access-control"}
        ),
        # Cryptography
        SecurityControl(
            id="A.10.1.1",
            title="Policy on the Use of Cryptographic Controls",
            description="A policy on the use of cryptographic controls for protection of information shall be developed and implemented",
            framework_id="iso27001",
            domain_id="A.10",
            risk_level=RiskLevel.HIGH,
            control_type=ControlType.TECHNICAL,
            tags={"iso27001", "isms", "information-security", "cryptography"}
        ),
        # Physical and Environmental Security
        SecurityControl(
            id="A.11.1.1",
            title="Physical Security Perimeter",
            description="Security perimeters shall be defined and used to protect areas that contain either sensitive or critical information",
            framework_id="iso27001",
            domain_id="A.11",
            risk_level=RiskLevel.MEDIUM,
            control_type=ControlType.PHYSICAL,
            tags={"iso27001", "isms", "information-security", "physical"}
        ),
        # Operations Security
        SecurityControl(
            id="A.12.1.2",
            title="Change Management",
            description="Changes to the organization, business processes, information processing facilities and systems shall be controlled",
            framework_id="iso27001",
            domain_id="A.12",
            risk_level=RiskLevel.HIGH,
            control_type=ControlType.PROCEDURAL,
            tags={"iso27001", "isms", "information-security", "change-management"}
        ),
        SecurityControl(
            id="A.12.6.1",
            title="Management of Technical Vulnerabilities",
            description="Information about technical vulnerabilities of information systems being used shall be obtained in a timely fashion",
            framework_id="iso27001",
            domain_id="A.12",
            risk_level=RiskLevel.CRITICAL,
            control_type=ControlType.TECHNICAL,
            tags={"iso27001", "isms", "information-security", "vulnerability-management"}
        )
    ]

    for control in controls:
        framework.add_control(control)

    return framework