"""
Core data models for security frameworks and controls.
"""

from datetime import datetime
from typing import Dict, List, Optional, Set
from enum import Enum
from pydantic import BaseModel, Field


class RiskLevel(str, Enum):
    """Risk level enumeration for security controls."""
    LOW = "low"
    MEDIUM = "medium"
    HIGH = "high"
    CRITICAL = "critical"


class ControlType(str, Enum):
    """Control type enumeration."""
    TECHNICAL = "technical"
    PROCEDURAL = "procedural"
    PHYSICAL = "physical"


class MappingType(str, Enum):
    """Mapping relationship type enumeration."""
    EQUIVALENT = "equivalent"
    PARTIAL = "partial"
    RELATED = "related"
    PARENT = "parent"
    CHILD = "child"


class ControlMapping(BaseModel):
    """Represents a mapping relationship between controls across frameworks."""
    
    source_framework: str
    source_control: str
    target_framework: str
    target_control: str
    mapping_type: MappingType = MappingType.EQUIVALENT
    confidence: float = Field(ge=0.0, le=1.0, default=1.0)
    notes: str = ""
    verified: bool = False
    last_updated: datetime = Field(default_factory=datetime.now)

    def verify(self) -> None:
        """Mark this mapping as verified."""
        self.verified = True
        self.last_updated = datetime.now()


class SecurityControl(BaseModel):
    """Represents an individual security control or requirement."""
    
    id: str
    title: str
    description: str
    framework_id: str
    domain_id: Optional[str] = None
    requirements: List[str] = Field(default_factory=list)
    implementation_guidance: str = ""
    testing_procedures: str = ""
    risk_level: RiskLevel = RiskLevel.MEDIUM
    control_type: ControlType = ControlType.PROCEDURAL
    tags: Set[str] = Field(default_factory=set)
    mappings: Dict[str, List[Dict]] = Field(default_factory=dict)

    def add_requirement(self, requirement: str) -> None:
        """Add a requirement to this control."""
        self.requirements.append(requirement)

    def add_mapping(self, framework_id: str, control_id: str, 
                   mapping_type: MappingType = MappingType.EQUIVALENT, 
                   confidence: float = 1.0) -> None:
        """Add a mapping to another framework control."""
        if framework_id not in self.mappings:
            self.mappings[framework_id] = []
        
        self.mappings[framework_id].append({
            "control_id": control_id,
            "mapping_type": mapping_type,
            "confidence": confidence
        })

    def get_mappings(self, framework_id: Optional[str] = None) -> Dict:
        """Get mappings for this control."""
        if framework_id:
            return self.mappings.get(framework_id, [])
        return self.mappings

    def add_tag(self, tag: str) -> None:
        """Add a tag to this control."""
        self.tags.add(tag)


class SecurityDomain(BaseModel):
    """Represents a domain or category within a security framework."""
    
    id: str
    name: str
    description: str
    framework_id: str
    controls: Dict[str, SecurityControl] = Field(default_factory=dict)

    def add_control(self, control: SecurityControl) -> None:
        """Add a control to this domain."""
        self.controls[control.id] = control

    def get_controls(self) -> List[SecurityControl]:
        """Get all controls in this domain."""
        return list(self.controls.values())


class SecurityFramework(BaseModel):
    """Represents a security framework with its controls and requirements."""
    
    id: str
    name: str
    version: str
    description: str
    domains: Dict[str, SecurityDomain] = Field(default_factory=dict)
    controls: Dict[str, SecurityControl] = Field(default_factory=dict)

    def add_domain(self, domain: SecurityDomain) -> None:
        """Add a domain to this framework."""
        self.domains[domain.id] = domain

    def add_control(self, control: SecurityControl) -> None:
        """Add a control to this framework."""
        self.controls[control.id] = control
        
        # Associate control with domain if specified
        if control.domain_id and control.domain_id in self.domains:
            self.domains[control.domain_id].add_control(control)

    def get_control(self, control_id: str) -> Optional[SecurityControl]:
        """Get a specific control by ID."""
        return self.controls.get(control_id)

    def get_all_controls(self) -> List[SecurityControl]:
        """Get all controls in this framework."""
        return list(self.controls.values())

    def get_domain(self, domain_id: str) -> Optional[SecurityDomain]:
        """Get a specific domain by ID."""
        return self.domains.get(domain_id)

    def get_all_domains(self) -> List[SecurityDomain]:
        """Get all domains in this framework."""
        return list(self.domains.values())


class GapAnalysis(BaseModel):
    """Results of a gap analysis between two frameworks."""
    
    source_framework: str
    target_framework: str
    total_source_controls: int
    total_target_controls: int
    mapped_controls: int
    coverage_percentage: float
    unmapped_source_controls: List[Dict]
    unmapped_target_controls: List[Dict]
    gaps: Dict[str, List[Dict]]


class ComplianceMatrix(BaseModel):
    """Compliance coverage matrix between frameworks."""
    
    frameworks: List[str]
    matrix: Dict[str, Dict[str, Dict]]
    generated_at: datetime = Field(default_factory=datetime.now)