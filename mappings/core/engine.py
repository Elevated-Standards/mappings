"""
Mapping Engine - Core logic for framework mappings and analysis.
"""

from typing import Dict, List, Optional, Set
from datetime import datetime
import re

from .models import (
    SecurityFramework, SecurityControl, ControlMapping, GapAnalysis,
    ComplianceMatrix, MappingType
)


class MappingEngine:
    """Core engine for framework mappings and analysis."""

    def __init__(self):
        self.frameworks: Dict[str, SecurityFramework] = {}
        self.mappings: List[ControlMapping] = []
        self.mapping_rules: Dict = {}

    def register_framework(self, framework: SecurityFramework) -> None:
        """Register a security framework."""
        self.frameworks[framework.id] = framework

    def get_framework(self, framework_id: str) -> Optional[SecurityFramework]:
        """Get a framework by ID."""
        return self.frameworks.get(framework_id)

    def get_all_frameworks(self) -> List[SecurityFramework]:
        """Get all registered frameworks."""
        return list(self.frameworks.values())

    def add_mapping(self, source_framework: str, source_control: str,
                   target_framework: str, target_control: str,
                   mapping_type: MappingType = MappingType.EQUIVALENT,
                   confidence: float = 1.0) -> ControlMapping:
        """Add a control mapping between frameworks."""
        mapping = ControlMapping(
            source_framework=source_framework,
            source_control=source_control,
            target_framework=target_framework,
            target_control=target_control,
            mapping_type=mapping_type,
            confidence=confidence
        )
        
        self.mappings.append(mapping)

        # Update control mappings in both directions
        source_fw = self.get_framework(source_framework)
        target_fw = self.get_framework(target_framework)

        if source_fw:
            source_ctrl = source_fw.get_control(source_control)
            if source_ctrl:
                source_ctrl.add_mapping(target_framework, target_control, mapping_type, confidence)

        if target_fw:
            target_ctrl = target_fw.get_control(target_control)
            if target_ctrl:
                target_ctrl.add_mapping(source_framework, source_control, mapping_type, confidence)

        return mapping

    def find_mappings_for_control(self, framework_id: str, control_id: str) -> List[ControlMapping]:
        """Find mappings for a specific control."""
        return [
            mapping for mapping in self.mappings
            if (mapping.source_framework == framework_id and mapping.source_control == control_id) or
               (mapping.target_framework == framework_id and mapping.target_control == control_id)
        ]

    def find_mappings_between_frameworks(self, framework1_id: str, framework2_id: str) -> List[ControlMapping]:
        """Find mappings between two frameworks."""
        return [
            mapping for mapping in self.mappings
            if (mapping.source_framework == framework1_id and mapping.target_framework == framework2_id) or
               (mapping.source_framework == framework2_id and mapping.target_framework == framework1_id)
        ]

    def analyze_gaps(self, source_framework_id: str, target_framework_id: str) -> GapAnalysis:
        """Analyze gaps between two frameworks."""
        source_framework = self.get_framework(source_framework_id)
        target_framework = self.get_framework(target_framework_id)

        if not source_framework or not target_framework:
            raise ValueError("Framework not found")

        source_controls = source_framework.get_all_controls()
        target_controls = target_framework.get_all_controls()
        mappings = self.find_mappings_between_frameworks(source_framework_id, target_framework_id)

        mapped_source_controls: Set[str] = set()
        mapped_target_controls: Set[str] = set()

        for mapping in mappings:
            if mapping.source_framework == source_framework_id:
                mapped_source_controls.add(mapping.source_control)
                mapped_target_controls.add(mapping.target_control)
            else:
                mapped_source_controls.add(mapping.target_control)
                mapped_target_controls.add(mapping.source_control)

        unmapped_source_controls = [
            control for control in source_controls
            if control.id not in mapped_source_controls
        ]

        unmapped_target_controls = [
            control for control in target_controls
            if control.id not in mapped_target_controls
        ]

        coverage_percentage = (len(mappings) / len(source_controls)) * 100 if source_controls else 0

        return GapAnalysis(
            source_framework=source_framework_id,
            target_framework=target_framework_id,
            total_source_controls=len(source_controls),
            total_target_controls=len(target_controls),
            mapped_controls=len(mappings),
            coverage_percentage=coverage_percentage,
            unmapped_source_controls=[{
                "id": control.id,
                "title": control.title,
                "description": control.description,
                "risk_level": control.risk_level
            } for control in unmapped_source_controls],
            unmapped_target_controls=[{
                "id": control.id,
                "title": control.title,
                "description": control.description,
                "risk_level": control.risk_level
            } for control in unmapped_target_controls],
            gaps={
                "source": [{
                    "id": control.id,
                    "title": control.title,
                    "description": control.description,
                    "risk_level": control.risk_level
                } for control in unmapped_source_controls],
                "target": [{
                    "id": control.id,
                    "title": control.title,
                    "description": control.description,
                    "risk_level": control.risk_level
                } for control in unmapped_target_controls]
            }
        )

    def generate_compliance_matrix(self, framework_ids: Optional[List[str]] = None) -> ComplianceMatrix:
        """Generate compliance matrix between frameworks."""
        if framework_ids is None:
            framework_ids = list(self.frameworks.keys())

        matrix = {}

        for framework_id in framework_ids:
            matrix[framework_id] = {}
            for target_id in framework_ids:
                if framework_id != target_id:
                    analysis = self.analyze_gaps(framework_id, target_id)
                    matrix[framework_id][target_id] = {
                        "coverage": analysis.coverage_percentage,
                        "mappings": analysis.mapped_controls,
                        "gaps": len(analysis.gaps["source"])
                    }

        return ComplianceMatrix(
            frameworks=framework_ids,
            matrix=matrix
        )

    def find_similar_controls(self, control_id: str, framework_id: str, threshold: float = 0.7) -> List[Dict]:
        """Find similar controls across frameworks using text analysis."""
        source_framework = self.get_framework(framework_id)
        if not source_framework:
            return []
        
        source_control = source_framework.get_control(control_id)
        if not source_control:
            return []

        similarities = []

        for fw_id, framework in self.frameworks.items():
            if fw_id == framework_id:
                continue

            for control in framework.get_all_controls():
                similarity = self._calculate_text_similarity(
                    f"{source_control.title} {source_control.description}",
                    f"{control.title} {control.description}"
                )

                if similarity >= threshold:
                    similarities.append({
                        "framework_id": fw_id,
                        "control_id": control.id,
                        "title": control.title,
                        "similarity": similarity,
                        "suggested_mapping_type": MappingType.EQUIVALENT if similarity > 0.9 else MappingType.RELATED
                    })

        return sorted(similarities, key=lambda x: x["similarity"], reverse=True)

    def _calculate_text_similarity(self, text1: str, text2: str) -> float:
        """Calculate text similarity using Jaccard index."""
        # Simple word-based similarity calculation
        words1 = set(re.findall(r'\w+', text1.lower()))
        words2 = set(re.findall(r'\w+', text2.lower()))
        
        # Filter out very short words
        words1 = {w for w in words1 if len(w) > 2}
        words2 = {w for w in words2 if len(w) > 2}

        if not words1 or not words2:
            return 0.0

        intersection = words1.intersection(words2)
        union = words1.union(words2)

        return len(intersection) / len(union) if union else 0.0

    def export_mappings(self) -> Dict:
        """Export mappings to dictionary format."""
        return {
            "frameworks": [
                {
                    "id": fw.id,
                    "name": fw.name,
                    "version": fw.version,
                    "description": fw.description,
                    "domains": [domain.dict() for domain in fw.get_all_domains()],
                    "controls": [control.dict() for control in fw.get_all_controls()]
                }
                for fw in self.frameworks.values()
            ],
            "mappings": [mapping.dict() for mapping in self.mappings]
        }

    def import_mappings(self, data: Dict) -> None:
        """Import mappings from dictionary format."""
        # Import frameworks
        for fw_data in data.get("frameworks", []):
            framework = SecurityFramework(**fw_data)
            self.register_framework(framework)

        # Import mappings
        for mapping_data in data.get("mappings", []):
            mapping = ControlMapping(**mapping_data)
            self.mappings.append(mapping)