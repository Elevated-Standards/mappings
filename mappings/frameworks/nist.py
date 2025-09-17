"""
NIST Cybersecurity Framework Definition
"""

from ..core.models import SecurityFramework, SecurityDomain, SecurityControl, RiskLevel, ControlType


def create_nist_framework() -> SecurityFramework:
    """Create NIST Cybersecurity Framework definition."""
    framework = SecurityFramework(
        id="nist-csf",
        name="NIST Cybersecurity Framework",
        version="1.1",
        description="Framework for Improving Critical Infrastructure Cybersecurity"
    )

    # NIST CSF Core Functions
    domains = [
        SecurityDomain(
            id="ID",
            name="Identify",
            description="Develop an organizational understanding to manage cybersecurity risk",
            framework_id="nist-csf"
        ),
        SecurityDomain(
            id="PR",
            name="Protect",
            description="Develop and implement appropriate safeguards to ensure delivery of critical services",
            framework_id="nist-csf"
        ),
        SecurityDomain(
            id="DE",
            name="Detect",
            description="Develop and implement appropriate activities to identify the occurrence of a cybersecurity event",
            framework_id="nist-csf"
        ),
        SecurityDomain(
            id="RS",
            name="Respond",
            description="Develop and implement appropriate activities to take action regarding a detected cybersecurity incident",
            framework_id="nist-csf"
        ),
        SecurityDomain(
            id="RC",
            name="Recover",
            description="Develop and implement appropriate activities to maintain plans for resilience and to restore any capabilities or services",
            framework_id="nist-csf"
        )
    ]

    for domain in domains:
        framework.add_domain(domain)

    # NIST CSF Core Controls
    controls = [
        # Identify
        SecurityControl(
            id="ID.AM-1",
            title="Asset Management - Physical Devices and Systems",
            description="Physical devices and systems within the organization are inventoried",
            framework_id="nist-csf",
            domain_id="ID",
            risk_level=RiskLevel.HIGH,
            control_type=ControlType.PROCEDURAL,
            tags={"nist", "cybersecurity", "framework", "assets"}
        ),
        SecurityControl(
            id="ID.AM-2",
            title="Asset Management - Software Platforms and Applications",
            description="Software platforms and applications within the organization are inventoried",
            framework_id="nist-csf",
            domain_id="ID",
            risk_level=RiskLevel.HIGH,
            control_type=ControlType.PROCEDURAL,
            tags={"nist", "cybersecurity", "framework", "assets"}
        ),
        SecurityControl(
            id="ID.GV-1",
            title="Governance - Information Security Policy",
            description="Organizational cybersecurity policy is established and communicated",
            framework_id="nist-csf",
            domain_id="ID",
            risk_level=RiskLevel.CRITICAL,
            control_type=ControlType.PROCEDURAL,
            tags={"nist", "cybersecurity", "framework", "governance"}
        ),
        SecurityControl(
            id="ID.RA-1",
            title="Risk Assessment - Risk Management Process",
            description="Asset vulnerabilities are identified and documented",
            framework_id="nist-csf",
            domain_id="ID",
            risk_level=RiskLevel.HIGH,
            control_type=ControlType.PROCEDURAL,
            tags={"nist", "cybersecurity", "framework", "risk-assessment"}
        ),
        # Protect
        SecurityControl(
            id="PR.AC-1",
            title="Access Control - Identity Management",
            description="Identities and credentials are issued, managed, verified, revoked, and audited for authorized devices, users and processes",
            framework_id="nist-csf",
            domain_id="PR",
            risk_level=RiskLevel.CRITICAL,
            control_type=ControlType.TECHNICAL,
            tags={"nist", "cybersecurity", "framework", "access-control"}
        ),
        SecurityControl(
            id="PR.AC-3",
            title="Access Control - Remote Access",
            description="Remote access is managed",
            framework_id="nist-csf",
            domain_id="PR",
            risk_level=RiskLevel.HIGH,
            control_type=ControlType.TECHNICAL,
            tags={"nist", "cybersecurity", "framework", "access-control"}
        ),
        SecurityControl(
            id="PR.DS-1",
            title="Data Security - Data-at-rest Protection",
            description="Data-at-rest is protected",
            framework_id="nist-csf",
            domain_id="PR",
            risk_level=RiskLevel.HIGH,
            control_type=ControlType.TECHNICAL,
            tags={"nist", "cybersecurity", "framework", "data-security"}
        ),
        SecurityControl(
            id="PR.DS-2",
            title="Data Security - Data-in-transit Protection",
            description="Data-in-transit is protected",
            framework_id="nist-csf",
            domain_id="PR",
            risk_level=RiskLevel.HIGH,
            control_type=ControlType.TECHNICAL,
            tags={"nist", "cybersecurity", "framework", "data-security"}
        ),
        SecurityControl(
            id="PR.PT-1",
            title="Protective Technology - Audit Logs",
            description="Audit/log records are determined, documented, implemented, and reviewed",
            framework_id="nist-csf",
            domain_id="PR",
            risk_level=RiskLevel.MEDIUM,
            control_type=ControlType.TECHNICAL,
            tags={"nist", "cybersecurity", "framework", "logging"}
        ),
        # Detect
        SecurityControl(
            id="DE.AE-1",
            title="Anomalies and Events - Baseline Establishment",
            description="A baseline of network operations and expected data flows for users and systems is established and managed",
            framework_id="nist-csf",
            domain_id="DE",
            risk_level=RiskLevel.MEDIUM,
            control_type=ControlType.TECHNICAL,
            tags={"nist", "cybersecurity", "framework", "detection"}
        ),
        SecurityControl(
            id="DE.CM-1",
            title="Security Continuous Monitoring - System Monitoring",
            description="The network and physical environment is monitored to detect potential cybersecurity events",
            framework_id="nist-csf",
            domain_id="DE",
            risk_level=RiskLevel.HIGH,
            control_type=ControlType.TECHNICAL,
            tags={"nist", "cybersecurity", "framework", "monitoring"}
        ),
        # Respond
        SecurityControl(
            id="RS.RP-1",
            title="Response Planning - Response Plan",
            description="Response plan is executed during or after an incident",
            framework_id="nist-csf",
            domain_id="RS",
            risk_level=RiskLevel.HIGH,
            control_type=ControlType.PROCEDURAL,
            tags={"nist", "cybersecurity", "framework", "incident-response"}
        ),
        SecurityControl(
            id="RS.CO-2",
            title="Communications - Incident Reporting",
            description="Incidents are reported consistent with established criteria",
            framework_id="nist-csf",
            domain_id="RS",
            risk_level=RiskLevel.MEDIUM,
            control_type=ControlType.PROCEDURAL,
            tags={"nist", "cybersecurity", "framework", "incident-response"}
        ),
        # Recover
        SecurityControl(
            id="RC.RP-1",
            title="Recovery Planning - Recovery Plan",
            description="Recovery plan is executed during or after a cybersecurity incident",
            framework_id="nist-csf",
            domain_id="RC",
            risk_level=RiskLevel.HIGH,
            control_type=ControlType.PROCEDURAL,
            tags={"nist", "cybersecurity", "framework", "recovery"}
        ),
        SecurityControl(
            id="RC.IM-1",
            title="Improvements - Lessons Learned",
            description="Recovery plans incorporate lessons learned",
            framework_id="nist-csf",
            domain_id="RC",
            risk_level=RiskLevel.MEDIUM,
            control_type=ControlType.PROCEDURAL,
            tags={"nist", "cybersecurity", "framework", "recovery"}
        )
    ]

    for control in controls:
        framework.add_control(control)

    return framework