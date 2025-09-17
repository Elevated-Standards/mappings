import { describe, it } from 'node:test';
import assert from 'node:assert';
import { mappingSystem } from '../index.js';

describe('Security Framework Mappings', () => {
  it('should load all frameworks', () => {
    const frameworks = mappingSystem.getFrameworks();
    assert.ok(frameworks.length >= 3, 'Should have at least 3 frameworks');
    
    const frameworkIds = frameworks.map(f => f.id);
    assert.ok(frameworkIds.includes('soc2'), 'Should include SOC 2');
    assert.ok(frameworkIds.includes('iso27001'), 'Should include ISO 27001');
    assert.ok(frameworkIds.includes('nist-csf'), 'Should include NIST CSF');
  });

  it('should have controls in each framework', () => {
    const frameworks = mappingSystem.getFrameworks();
    
    frameworks.forEach(framework => {
      const controls = framework.getAllControls();
      assert.ok(controls.length > 0, `${framework.name} should have controls`);
      
      controls.forEach(control => {
        assert.ok(control.id, 'Control should have ID');
        assert.ok(control.title, 'Control should have title');
        assert.ok(control.description, 'Control should have description');
      });
    });
  });

  it('should generate gap analysis', () => {
    const analysis = mappingSystem.analyzeCompliance('soc2', 'iso27001');
    
    assert.ok(typeof analysis.coveragePercentage === 'number', 'Should return coverage percentage');
    assert.ok(analysis.coveragePercentage >= 0 && analysis.coveragePercentage <= 100, 'Coverage should be 0-100%');
    assert.ok(Array.isArray(analysis.gaps.source), 'Should return source gaps array');
    assert.ok(Array.isArray(analysis.gaps.target), 'Should return target gaps array');
  });

  it('should find existing mappings', () => {
    const mappings = mappingSystem.getEngine().findMappingsForControl('soc2', 'CC6.1');
    assert.ok(Array.isArray(mappings), 'Should return mappings array');
    assert.ok(mappings.length > 0, 'Should find at least one mapping for CC6.1');
  });

  it('should generate compliance matrix', () => {
    const matrix = mappingSystem.generateComplianceMatrix();
    assert.ok(typeof matrix === 'object', 'Should return matrix object');
    
    // Check that matrix has data for each framework
    const frameworks = mappingSystem.getFrameworks();
    frameworks.forEach(framework => {
      assert.ok(matrix[framework.id], `Matrix should have data for ${framework.id}`);
    });
  });

  it('should find similar controls', () => {
    const similar = mappingSystem.findPotentialMappings('soc2', 'CC6.1', 0.5);
    assert.ok(Array.isArray(similar), 'Should return similar controls array');
    
    similar.forEach(match => {
      assert.ok(match.frameworkId, 'Match should have framework ID');
      assert.ok(match.controlId, 'Match should have control ID');
      assert.ok(typeof match.similarity === 'number', 'Match should have similarity score');
      assert.ok(match.similarity >= 0.5, 'Similarity should be above threshold');
    });
  });

  it('should export and import data', () => {
    const exported = mappingSystem.export();
    assert.ok(exported.frameworks, 'Exported data should have frameworks');
    assert.ok(exported.mappings, 'Exported data should have mappings');
    assert.ok(Array.isArray(exported.frameworks), 'Frameworks should be array');
    assert.ok(Array.isArray(exported.mappings), 'Mappings should be array');
  });
});