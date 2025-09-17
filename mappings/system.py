"""
Main entry point for the security frameworks mapping system.
"""

from typing import Dict, List, Optional
from datetime import datetime
from .core.engine import MappingEngine
from .core.models import MappingType
from .frameworks.soc2 import create_soc2_framework
from .frameworks.iso27001 import create_iso27001_framework
from .frameworks.nist import create_nist_framework


class SecurityFrameworkMappings:
    """Main entry point for the security frameworks mapping system."""

    def __init__(self):
        self.engine = MappingEngine()
        self._initialize_frameworks()
        self._initialize_basic_mappings()

    def _initialize_frameworks(self) -> None:
        """Initialize all supported frameworks."""
        self.engine.register_framework(create_soc2_framework())
        self.engine.register_framework(create_iso27001_framework())
        self.engine.register_framework(create_nist_framework())

    def _initialize_basic_mappings(self) -> None:
        """Initialize basic cross-framework mappings."""
        # SOC 2 to ISO 27001 mappings
        self.engine.add_mapping("soc2", "CC1.1", "iso27001", "A.5.1.1", MappingType.EQUIVALENT, 0.9)
        self.engine.add_mapping("soc2", "CC6.1", "iso27001", "A.9.1.1", MappingType.RELATED, 0.8)
        self.engine.add_mapping("soc2", "CC6.2", "iso27001", "A.9.2.1", MappingType.EQUIVALENT, 0.9)
        self.engine.add_mapping("soc2", "CC7.1", "iso27001", "A.12.6.1", MappingType.RELATED, 0.7)

        # SOC 2 to NIST CSF mappings
        self.engine.add_mapping("soc2", "CC6.1", "nist-csf", "PR.AC-1", MappingType.EQUIVALENT, 0.9)
        self.engine.add_mapping("soc2", "CC7.1", "nist-csf", "DE.CM-1", MappingType.EQUIVALENT, 0.8)
        self.engine.add_mapping("soc2", "CC1.1", "nist-csf", "ID.GV-1", MappingType.RELATED, 0.7)

        # ISO 27001 to NIST CSF mappings
        self.engine.add_mapping("iso27001", "A.8.1.1", "nist-csf", "ID.AM-1", MappingType.EQUIVALENT, 0.9)
        self.engine.add_mapping("iso27001", "A.9.1.1", "nist-csf", "PR.AC-1", MappingType.RELATED, 0.8)
        self.engine.add_mapping("iso27001", "A.10.1.1", "nist-csf", "PR.DS-1", MappingType.RELATED, 0.7)
        self.engine.add_mapping("iso27001", "A.12.6.1", "nist-csf", "ID.RA-1", MappingType.EQUIVALENT, 0.8)

        # Mark some mappings as verified
        for mapping in self.engine.mappings:
            if mapping.confidence >= 0.8:
                mapping.verify()

    def get_engine(self) -> MappingEngine:
        """Get the mapping engine instance."""
        return self.engine

    def get_frameworks(self) -> List:
        """Get all available frameworks."""
        return self.engine.get_all_frameworks()

    def analyze_compliance(self, source_framework: str, target_framework: str):
        """Analyze compliance gaps between frameworks."""
        return self.engine.analyze_gaps(source_framework, target_framework)

    def generate_compliance_matrix(self):
        """Generate comprehensive compliance matrix."""
        framework_ids = list(self.engine.frameworks.keys())
        return self.engine.generate_compliance_matrix(framework_ids)

    def find_potential_mappings(self, framework_id: str, control_id: str, threshold: float = 0.7):
        """Find potential mappings using similarity analysis."""
        return self.engine.find_similar_controls(control_id, framework_id, threshold)

    def export(self) -> Dict:
        """Export all data to dictionary format."""
        return self.engine.export_mappings()

    def generate_report(self, frameworks: Optional[List[str]] = None) -> Dict:
        """Generate compliance report for specific frameworks."""
        target_frameworks = frameworks or list(self.engine.frameworks.keys())
        
        report = {
            "generated_at": str(datetime.now()),
            "frameworks": [
                {
                    "id": framework.id,
                    "name": framework.name,
                    "version": framework.version,
                    "total_controls": len(framework.get_all_controls()),
                    "domains": len(framework.get_all_domains())
                }
                for framework in [self.engine.get_framework(fwid) for fwid in target_frameworks]
                if framework
            ],
            "mappings": {
                "total": len(self.engine.mappings),
                "verified": len([m for m in self.engine.mappings if m.verified]),
                "by_type": self._group_mappings_by_type()
            },
            "coverage": self._calculate_framework_coverage(target_frameworks),
            "gaps": self._identify_gaps(target_frameworks)
        }

        return report

    def _group_mappings_by_type(self) -> Dict:
        """Group mappings by type for reporting."""
        grouped = {}
        for mapping in self.engine.mappings:
            mapping_type = mapping.mapping_type
            if mapping_type not in grouped:
                grouped[mapping_type] = 0
            grouped[mapping_type] += 1
        return grouped

    def _calculate_framework_coverage(self, framework_ids: List[str]) -> Dict:
        """Calculate coverage percentages between frameworks."""
        coverage = {}
        
        for source_id in framework_ids:
            coverage[source_id] = {}
            for target_id in framework_ids:
                if source_id != target_id:
                    analysis = self.engine.analyze_gaps(source_id, target_id)
                    coverage[source_id][target_id] = {
                        "percentage": round(analysis.coverage_percentage, 2),
                        "mapped_controls": analysis.mapped_controls,
                        "total_controls": analysis.total_source_controls
                    }

        return coverage

    def _identify_gaps(self, framework_ids: List[str]) -> Dict:
        """Identify significant gaps across frameworks."""
        gaps = {}
        
        for source_id in framework_ids:
            gaps[source_id] = {}
            for target_id in framework_ids:
                if source_id != target_id:
                    analysis = self.engine.analyze_gaps(source_id, target_id)
                    gaps[source_id][target_id] = [
                        gap for gap in analysis.gaps["source"]
                        if gap["risk_level"] in ["high", "critical"]
                    ]

        return gaps


# Create and export default instance
mapping_system = SecurityFrameworkMappings()