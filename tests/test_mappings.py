"""
Tests for the security frameworks mapping system.
"""

import pytest
from mappings.system import mapping_system
from mappings.core.models import SecurityFramework, SecurityControl, RiskLevel


def test_load_frameworks():
    """Test that all frameworks are loaded."""
    frameworks = mapping_system.get_frameworks()
    assert len(frameworks) >= 3
    
    framework_ids = [f.id for f in frameworks]
    assert "soc2" in framework_ids
    assert "iso27001" in framework_ids
    assert "nist-csf" in framework_ids


def test_frameworks_have_controls():
    """Test that each framework has controls."""
    frameworks = mapping_system.get_frameworks()
    
    for framework in frameworks:
        controls = framework.get_all_controls()
        assert len(controls) > 0
        
        for control in controls:
            assert control.id
            assert control.title
            assert control.description


def test_gap_analysis():
    """Test gap analysis functionality."""
    analysis = mapping_system.analyze_compliance("soc2", "iso27001")
    
    assert isinstance(analysis.coverage_percentage, float)
    assert 0 <= analysis.coverage_percentage <= 100
    assert isinstance(analysis.gaps, dict)
    assert "source" in analysis.gaps
    assert "target" in analysis.gaps


def test_find_mappings():
    """Test finding mappings for controls."""
    mappings = mapping_system.get_engine().find_mappings_for_control("soc2", "CC6.1")
    assert isinstance(mappings, list)
    assert len(mappings) > 0


def test_compliance_matrix():
    """Test compliance matrix generation."""
    matrix = mapping_system.generate_compliance_matrix()
    assert hasattr(matrix, 'frameworks')
    assert hasattr(matrix, 'matrix')
    
    # Check that matrix has data for each framework
    frameworks = mapping_system.get_frameworks()
    for framework in frameworks:
        assert framework.id in matrix.matrix


def test_similar_controls():
    """Test finding similar controls."""
    similar = mapping_system.find_potential_mappings("soc2", "CC6.1", 0.5)
    assert isinstance(similar, list)
    
    for match in similar:
        assert "framework_id" in match
        assert "control_id" in match
        assert "similarity" in match
        assert match["similarity"] >= 0.5


def test_export_import():
    """Test export and import functionality."""
    exported = mapping_system.export()
    assert "frameworks" in exported
    assert "mappings" in exported
    assert isinstance(exported["frameworks"], list)
    assert isinstance(exported["mappings"], list)


def test_security_control_model():
    """Test SecurityControl model."""
    control = SecurityControl(
        id="TEST-1",
        title="Test Control",
        description="Test description",
        framework_id="test",
        risk_level=RiskLevel.HIGH
    )
    
    assert control.id == "TEST-1"
    assert control.risk_level == RiskLevel.HIGH
    
    control.add_tag("test")
    assert "test" in control.tags


def test_security_framework_model():
    """Test SecurityFramework model."""
    framework = SecurityFramework(
        id="test-fw",
        name="Test Framework",
        version="1.0",
        description="Test description"
    )
    
    control = SecurityControl(
        id="T1",
        title="Control 1",
        description="Test control",
        framework_id="test-fw"
    )
    
    framework.add_control(control)
    assert framework.get_control("T1") == control
    assert len(framework.get_all_controls()) == 1