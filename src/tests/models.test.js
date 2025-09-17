import { describe, it } from 'node:test';
import assert from 'node:assert';
import { SecurityFramework, SecurityDomain, SecurityControl, ControlMapping } from '../core/models.js';

describe('Security Framework Models', () => {
  it('should create a security framework', () => {
    const framework = new SecurityFramework('test-fw', 'Test Framework', '1.0', 'Test description');
    
    assert.strictEqual(framework.id, 'test-fw');
    assert.strictEqual(framework.name, 'Test Framework');
    assert.strictEqual(framework.version, '1.0');
    assert.strictEqual(framework.description, 'Test description');
    assert.ok(framework.domains instanceof Map);
    assert.ok(framework.controls instanceof Map);
  });

  it('should add domains to framework', () => {
    const framework = new SecurityFramework('test-fw', 'Test Framework', '1.0', 'Test description');
    const domain = new SecurityDomain('D1', 'Domain 1', 'Test domain', 'test-fw');
    
    framework.addDomain(domain);
    
    assert.strictEqual(framework.domains.size, 1);
    assert.strictEqual(framework.getDomain('D1'), domain);
  });

  it('should add controls to framework', () => {
    const framework = new SecurityFramework('test-fw', 'Test Framework', '1.0', 'Test description');
    const control = new SecurityControl('C1', 'Control 1', 'Test control', 'test-fw');
    
    framework.addControl(control);
    
    assert.strictEqual(framework.controls.size, 1);
    assert.strictEqual(framework.getControl('C1'), control);
  });

  it('should create security control with mappings', () => {
    const control = new SecurityControl('C1', 'Control 1', 'Test control', 'test-fw');
    
    control.addMapping('other-fw', 'OC1', 'equivalent');
    control.addTag('test');
    
    const mappings = control.getMappings('other-fw');
    assert.strictEqual(mappings.length, 1);
    assert.strictEqual(mappings[0].controlId, 'OC1');
    assert.strictEqual(mappings[0].mappingType, 'equivalent');
    assert.ok(control.tags.includes('test'));
  });

  it('should create control mapping', () => {
    const mapping = new ControlMapping('fw1', 'C1', 'fw2', 'C2', 'equivalent');
    
    assert.strictEqual(mapping.sourceFramework, 'fw1');
    assert.strictEqual(mapping.sourceControl, 'C1');
    assert.strictEqual(mapping.targetFramework, 'fw2');
    assert.strictEqual(mapping.targetControl, 'C2');
    assert.strictEqual(mapping.mappingType, 'equivalent');
    assert.strictEqual(mapping.confidence, 1.0);
    
    mapping.setConfidence(0.8);
    assert.strictEqual(mapping.confidence, 0.8);
  });

  it('should enforce confidence bounds', () => {
    const mapping = new ControlMapping('fw1', 'C1', 'fw2', 'C2');
    
    mapping.setConfidence(1.5); // Above max
    assert.strictEqual(mapping.confidence, 1.0);
    
    mapping.setConfidence(-0.5); // Below min
    assert.strictEqual(mapping.confidence, 0.0);
  });
});