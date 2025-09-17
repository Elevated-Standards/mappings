import { ControlMapping } from './models.js';

/**
 * Mapping Engine - Core logic for framework mappings and analysis
 */
export class MappingEngine {
  constructor() {
    this.frameworks = new Map();
    this.mappings = [];
    this.mappingRules = new Map();
  }

  /**
   * Register a security framework
   */
  registerFramework(framework) {
    this.frameworks.set(framework.id, framework);
  }

  /**
   * Get a framework by ID
   */
  getFramework(frameworkId) {
    return this.frameworks.get(frameworkId);
  }

  /**
   * Get all registered frameworks
   */
  getAllFrameworks() {
    return Array.from(this.frameworks.values());
  }

  /**
   * Add a control mapping between frameworks
   */
  addMapping(sourceFramework, sourceControl, targetFramework, targetControl, mappingType = 'equivalent', confidence = 1.0) {
    const mapping = new ControlMapping(sourceFramework, sourceControl, targetFramework, targetControl, mappingType);
    mapping.setConfidence(confidence);
    this.mappings.push(mapping);

    // Update control mappings in both directions
    const sourceControlObj = this.getFramework(sourceFramework)?.getControl(sourceControl);
    const targetControlObj = this.getFramework(targetFramework)?.getControl(targetControl);

    if (sourceControlObj) {
      sourceControlObj.addMapping(targetFramework, targetControl, mappingType);
    }
    if (targetControlObj) {
      targetControlObj.addMapping(sourceFramework, sourceControl, mappingType);
    }

    return mapping;
  }

  /**
   * Find mappings for a specific control
   */
  findMappingsForControl(frameworkId, controlId) {
    return this.mappings.filter(mapping => 
      (mapping.sourceFramework === frameworkId && mapping.sourceControl === controlId) ||
      (mapping.targetFramework === frameworkId && mapping.targetControl === controlId)
    );
  }

  /**
   * Find mappings between two frameworks
   */
  findMappingsBetweenFrameworks(framework1Id, framework2Id) {
    return this.mappings.filter(mapping =>
      (mapping.sourceFramework === framework1Id && mapping.targetFramework === framework2Id) ||
      (mapping.sourceFramework === framework2Id && mapping.targetFramework === framework1Id)
    );
  }

  /**
   * Analyze gaps between two frameworks
   */
  analyzeGaps(sourceFrameworkId, targetFrameworkId) {
    const sourceFramework = this.getFramework(sourceFrameworkId);
    const targetFramework = this.getFramework(targetFrameworkId);

    if (!sourceFramework || !targetFramework) {
      throw new Error('Framework not found');
    }

    const sourceControls = sourceFramework.getAllControls();
    const mappings = this.findMappingsBetweenFrameworks(sourceFrameworkId, targetFrameworkId);
    
    const mappedSourceControls = new Set();
    const mappedTargetControls = new Set();

    mappings.forEach(mapping => {
      if (mapping.sourceFramework === sourceFrameworkId) {
        mappedSourceControls.add(mapping.sourceControl);
        mappedTargetControls.add(mapping.targetControl);
      } else {
        mappedSourceControls.add(mapping.targetControl);
        mappedTargetControls.add(mapping.sourceControl);
      }
    });

    const unmappedSourceControls = sourceControls.filter(control => 
      !mappedSourceControls.has(control.id)
    );

    const targetControls = targetFramework.getAllControls();
    const unmappedTargetControls = targetControls.filter(control =>
      !mappedTargetControls.has(control.id)
    );

    return {
      sourceFramework: sourceFrameworkId,
      targetFramework: targetFrameworkId,
      totalSourceControls: sourceControls.length,
      totalTargetControls: targetControls.length,
      mappedControls: mappings.length,
      coveragePercentage: (mappings.length / sourceControls.length) * 100,
      unmappedSourceControls,
      unmappedTargetControls,
      gaps: {
        source: unmappedSourceControls.map(control => ({
          id: control.id,
          title: control.title,
          description: control.description,
          riskLevel: control.riskLevel
        })),
        target: unmappedTargetControls.map(control => ({
          id: control.id,
          title: control.title,
          description: control.description,
          riskLevel: control.riskLevel
        }))
      }
    };
  }

  /**
   * Generate compliance matrix between frameworks
   */
  generateComplianceMatrix(frameworkIds) {
    const matrix = {};
    
    frameworkIds.forEach(frameworkId => {
      matrix[frameworkId] = {};
      frameworkIds.forEach(targetId => {
        if (frameworkId !== targetId) {
          const analysis = this.analyzeGaps(frameworkId, targetId);
          matrix[frameworkId][targetId] = {
            coverage: analysis.coveragePercentage,
            mappings: analysis.mappedControls,
            gaps: analysis.gaps.source.length
          };
        }
      });
    });

    return matrix;
  }

  /**
   * Find similar controls across frameworks using text analysis
   */
  findSimilarControls(controlId, frameworkId, threshold = 0.7) {
    const sourceControl = this.getFramework(frameworkId)?.getControl(controlId);
    if (!sourceControl) return [];

    const similarities = [];
    
    this.frameworks.forEach((framework, fwId) => {
      if (fwId === frameworkId) return;
      
      framework.getAllControls().forEach(control => {
        const similarity = this.calculateTextSimilarity(
          sourceControl.title + ' ' + sourceControl.description,
          control.title + ' ' + control.description
        );
        
        if (similarity >= threshold) {
          similarities.push({
            frameworkId: fwId,
            controlId: control.id,
            title: control.title,
            similarity: similarity,
            suggestedMappingType: similarity > 0.9 ? 'equivalent' : 'related'
          });
        }
      });
    });

    return similarities.sort((a, b) => b.similarity - a.similarity);
  }

  /**
   * Simple text similarity calculation using Jaccard index
   */
  calculateTextSimilarity(text1, text2) {
    const words1 = new Set(text1.toLowerCase().split(/\W+/).filter(w => w.length > 2));
    const words2 = new Set(text2.toLowerCase().split(/\W+/).filter(w => w.length > 2));
    
    const intersection = new Set([...words1].filter(x => words2.has(x)));
    const union = new Set([...words1, ...words2]);
    
    return intersection.size / union.size;
  }

  /**
   * Export mappings to JSON
   */
  exportMappings() {
    return {
      frameworks: Array.from(this.frameworks.values()).map(fw => ({
        id: fw.id,
        name: fw.name,
        version: fw.version,
        description: fw.description,
        domains: Array.from(fw.domains.values()),
        controls: Array.from(fw.controls.values())
      })),
      mappings: this.mappings.map(mapping => ({
        sourceFramework: mapping.sourceFramework,
        sourceControl: mapping.sourceControl,
        targetFramework: mapping.targetFramework,
        targetControl: mapping.targetControl,
        mappingType: mapping.mappingType,
        confidence: mapping.confidence,
        notes: mapping.notes,
        verified: mapping.verified,
        lastUpdated: mapping.lastUpdated
      }))
    };
  }

  /**
   * Import mappings from JSON
   */
  importMappings(data) {
    // Import frameworks
    data.frameworks?.forEach(fwData => {
      const framework = new SecurityFramework(fwData.id, fwData.name, fwData.version, fwData.description);
      
      fwData.domains?.forEach(domainData => {
        const domain = new SecurityDomain(domainData.id, domainData.name, domainData.description, domainData.frameworkId);
        framework.addDomain(domain);
      });
      
      fwData.controls?.forEach(controlData => {
        const control = new SecurityControl(
          controlData.id,
          controlData.title,
          controlData.description,
          controlData.frameworkId,
          controlData.domainId
        );
        
        // Restore other properties
        control.requirements = controlData.requirements || [];
        control.implementationGuidance = controlData.implementationGuidance || '';
        control.testingProcedures = controlData.testingProcedures || '';
        control.riskLevel = controlData.riskLevel || 'medium';
        control.controlType = controlData.controlType || 'procedural';
        control.tags = controlData.tags || [];
        
        framework.addControl(control);
      });
      
      this.registerFramework(framework);
    });

    // Import mappings
    data.mappings?.forEach(mappingData => {
      const mapping = new ControlMapping(
        mappingData.sourceFramework,
        mappingData.sourceControl,
        mappingData.targetFramework,
        mappingData.targetControl,
        mappingData.mappingType
      );
      
      mapping.setConfidence(mappingData.confidence);
      mapping.notes = mappingData.notes || '';
      mapping.verified = mappingData.verified || false;
      mapping.lastUpdated = new Date(mappingData.lastUpdated);
      
      this.mappings.push(mapping);
    });
  }
}