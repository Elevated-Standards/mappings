"""
SOC 2 Framework Definition
System and Organization Controls 2
"""

from ..core.models import SecurityFramework, SecurityDomain, SecurityControl, RiskLevel, ControlType


def create_soc2_framework() -> SecurityFramework:
    """Create SOC 2 framework definition."""
    framework = SecurityFramework(
        id="soc2",
        name="SOC 2",
        version="2017",
        description="System and Organization Controls 2 - Trust Services Criteria for Security, Availability, Processing Integrity, Confidentiality, and Privacy"
    )

    # SOC 2 Trust Services Categories (Domains)
    domains = [
        SecurityDomain(
            id="security",
            name="Security",
            description="The system is protected against unauthorized access",
            framework_id="soc2"
        ),
        SecurityDomain(
            id="availability",
            name="Availability",
            description="The system is available for operation and use",
            framework_id="soc2"
        ),
        SecurityDomain(
            id="processing_integrity",
            name="Processing Integrity",
            description="System processing is complete, valid, accurate, timely, and authorized",
            framework_id="soc2"
        ),
        SecurityDomain(
            id="confidentiality",
            name="Confidentiality",
            description="Information designated as confidential is protected",
            framework_id="soc2"
        ),
        SecurityDomain(
            id="privacy",
            name="Privacy",
            description="Personal information is collected, used, retained, disclosed, and disposed of in conformity with commitments",
            framework_id="soc2"
        )
    ]

    for domain in domains:
        framework.add_domain(domain)

    # SOC 2 Security Controls (Common Criteria)
    controls = [
        SecurityControl(
            id="CC1.1",
            title="Control Environment - Integrity and Ethical Values",
            description="The entity demonstrates a commitment to integrity and ethical values",
            framework_id="soc2",
            domain_id="security",
            risk_level=RiskLevel.HIGH,
            control_type=ControlType.PROCEDURAL,
            tags={"soc2", "audit", "compliance", "governance"}
        ),
        SecurityControl(
            id="CC1.2",
            title="Control Environment - Board Independence",
            description="The board of directors demonstrates independence from management and exercises oversight",
            framework_id="soc2",
            domain_id="security",
            risk_level=RiskLevel.MEDIUM,
            control_type=ControlType.PROCEDURAL,
            tags={"soc2", "audit", "compliance", "governance"}
        ),
        SecurityControl(
            id="CC2.1",
            title="Communication and Information - Internal Communication",
            description="The entity obtains or generates and uses relevant, quality information to support the functioning of internal control",
            framework_id="soc2",
            domain_id="security",
            risk_level=RiskLevel.MEDIUM,
            control_type=ControlType.PROCEDURAL,
            tags={"soc2", "audit", "compliance", "communication"}
        ),
        SecurityControl(
            id="CC3.1",
            title="Risk Assessment - Objectives",
            description="The entity specifies objectives with sufficient clarity to enable the identification and assessment of risks",
            framework_id="soc2",
            domain_id="security",
            risk_level=RiskLevel.HIGH,
            control_type=ControlType.PROCEDURAL,
            tags={"soc2", "audit", "compliance", "risk-management"}
        ),
        SecurityControl(
            id="CC4.1",
            title="Monitoring Activities - Ongoing Monitoring",
            description="The entity selects, develops, and performs ongoing and/or separate evaluations",
            framework_id="soc2",
            domain_id="security",
            risk_level=RiskLevel.HIGH,
            control_type=ControlType.PROCEDURAL,
            tags={"soc2", "audit", "compliance", "monitoring"}
        ),
        SecurityControl(
            id="CC5.1",
            title="Control Activities - Selection and Development",
            description="The entity selects and develops control activities that contribute to the mitigation of risks",
            framework_id="soc2",
            domain_id="security",
            risk_level=RiskLevel.HIGH,
            control_type=ControlType.PROCEDURAL,
            tags={"soc2", "audit", "compliance", "controls"}
        ),
        SecurityControl(
            id="CC6.1",
            title="Logical and Physical Access Controls - Access Control",
            description="The entity implements logical access security software, infrastructure, and architectures over protected information assets",
            framework_id="soc2",
            domain_id="security",
            risk_level=RiskLevel.CRITICAL,
            control_type=ControlType.TECHNICAL,
            tags={"soc2", "audit", "compliance", "access-control"}
        ),
        SecurityControl(
            id="CC6.2",
            title="Logical and Physical Access Controls - Authentication",
            description="Prior to issuing system credentials and granting system access, the entity registers and authorizes new internal and external users",
            framework_id="soc2",
            domain_id="security",
            risk_level=RiskLevel.CRITICAL,
            control_type=ControlType.TECHNICAL,
            tags={"soc2", "audit", "compliance", "authentication"}
        ),
        SecurityControl(
            id="CC6.7",
            title="System Operations - Data Transmission",
            description="The entity restricts the transmission of data and software to defined system users",
            framework_id="soc2",
            domain_id="security",
            risk_level=RiskLevel.HIGH,
            control_type=ControlType.TECHNICAL,
            tags={"soc2", "audit", "compliance", "data-transmission"}
        ),
        SecurityControl(
            id="CC7.1",
            title="System Operations - System Monitoring",
            description="The entity monitors the system and various communications channels for security events",
            framework_id="soc2",
            domain_id="security",
            risk_level=RiskLevel.HIGH,
            control_type=ControlType.TECHNICAL,
            tags={"soc2", "audit", "compliance", "monitoring"}
        ),
        # Availability Controls
        SecurityControl(
            id="A1.1",
            title="Availability - System Capacity",
            description="The entity maintains system capacity consistent with system processing requirements",
            framework_id="soc2",
            domain_id="availability",
            risk_level=RiskLevel.HIGH,
            control_type=ControlType.TECHNICAL,
            tags={"soc2", "audit", "compliance", "availability"}
        ),
        SecurityControl(
            id="A1.2",
            title="Availability - Environmental Protection",
            description="The entity authorizes, designs, develops or acquires, implements, operates, approves, maintains, and monitors environmental protections",
            framework_id="soc2",
            domain_id="availability",
            risk_level=RiskLevel.MEDIUM,
            control_type=ControlType.PHYSICAL,
            tags={"soc2", "audit", "compliance", "availability"}
        )
    ]

    for control in controls:
        framework.add_control(control)

    return framework