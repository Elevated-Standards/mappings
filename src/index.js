import { MappingEngine } from './core/mapping-engine.js';
import { createSOC2Framework } from '../frameworks/soc2.js';
import { createISO27001Framework } from '../frameworks/iso27001.js';
import { createNISTFramework } from '../frameworks/nist.js';

/**
 * Main entry point for the security frameworks mapping system
 */
export class SecurityFrameworkMappings {
  constructor() {
    this.engine = new MappingEngine();
    this.initializeFrameworks();
    this.initializeBasicMappings();
  }

  /**
   * Initialize all supported frameworks
   */
  initializeFrameworks() {
    // Register core frameworks
    this.engine.registerFramework(createSOC2Framework());
    this.engine.registerFramework(createISO27001Framework());
    this.engine.registerFramework(createNISTFramework());
  }

  /**
   * Initialize basic cross-framework mappings
   */
  initializeBasicMappings() {
    // SOC 2 to ISO 27001 mappings
    this.engine.addMapping('soc2', 'CC1.1', 'iso27001', 'A.5.1.1', 'equivalent', 0.9);
    this.engine.addMapping('soc2', 'CC6.1', 'iso27001', 'A.9.1.1', 'related', 0.8);
    this.engine.addMapping('soc2', 'CC6.2', 'iso27001', 'A.9.2.1', 'equivalent', 0.9);
    this.engine.addMapping('soc2', 'CC7.1', 'iso27001', 'A.12.6.1', 'related', 0.7);

    // SOC 2 to NIST CSF mappings
    this.engine.addMapping('soc2', 'CC6.1', 'nist-csf', 'PR.AC-1', 'equivalent', 0.9);
    this.engine.addMapping('soc2', 'CC7.1', 'nist-csf', 'DE.CM-1', 'equivalent', 0.8);
    this.engine.addMapping('soc2', 'CC1.1', 'nist-csf', 'ID.GV-1', 'related', 0.7);

    // ISO 27001 to NIST CSF mappings
    this.engine.addMapping('iso27001', 'A.8.1.1', 'nist-csf', 'ID.AM-1', 'equivalent', 0.9);
    this.engine.addMapping('iso27001', 'A.9.1.1', 'nist-csf', 'PR.AC-1', 'related', 0.8);
    this.engine.addMapping('iso27001', 'A.10.1.1', 'nist-csf', 'PR.DS-1', 'related', 0.7);
    this.engine.addMapping('iso27001', 'A.12.6.1', 'nist-csf', 'ID.RA-1', 'equivalent', 0.8);

    // Mark some mappings as verified
    this.engine.mappings.forEach(mapping => {
      if (mapping.confidence >= 0.8) {
        mapping.verify();
      }
    });
  }

  /**
   * Get the mapping engine instance
   */
  getEngine() {
    return this.engine;
  }

  /**
   * Get all available frameworks
   */
  getFrameworks() {
    return this.engine.getAllFrameworks();
  }

  /**
   * Analyze compliance gaps between frameworks
   */
  analyzeCompliance(sourceFramework, targetFramework) {
    return this.engine.analyzeGaps(sourceFramework, targetFramework);
  }

  /**
   * Generate comprehensive compliance matrix
   */
  generateComplianceMatrix() {
    const frameworkIds = Array.from(this.engine.frameworks.keys());
    return this.engine.generateComplianceMatrix(frameworkIds);
  }

  /**
   * Find potential mappings using similarity analysis
   */
  findPotentialMappings(frameworkId, controlId, threshold = 0.7) {
    return this.engine.findSimilarControls(controlId, frameworkId, threshold);
  }

  /**
   * Export all data to JSON
   */
  export() {
    return this.engine.exportMappings();
  }

  /**
   * Generate compliance report for specific frameworks
   */
  generateReport(frameworks = null) {
    const targetFrameworks = frameworks || Array.from(this.engine.frameworks.keys());
    const report = {
      generatedAt: new Date().toISOString(),
      frameworks: targetFrameworks.map(fwId => {
        const framework = this.engine.getFramework(fwId);
        return {
          id: framework.id,
          name: framework.name,
          version: framework.version,
          totalControls: framework.getAllControls().length,
          domains: framework.getAllDomains().length
        };
      }),
      mappings: {
        total: this.engine.mappings.length,
        verified: this.engine.mappings.filter(m => m.verified).length,
        byType: this.groupMappingsByType()
      },
      coverage: this.calculateFrameworkCoverage(targetFrameworks),
      gaps: this.identifyGaps(targetFrameworks)
    };

    return report;
  }

  /**
   * Group mappings by type for reporting
   */
  groupMappingsByType() {
    const grouped = {};
    this.engine.mappings.forEach(mapping => {
      if (!grouped[mapping.mappingType]) {
        grouped[mapping.mappingType] = 0;
      }
      grouped[mapping.mappingType]++;
    });
    return grouped;
  }

  /**
   * Calculate coverage percentages between frameworks
   */
  calculateFrameworkCoverage(frameworkIds) {
    const coverage = {};
    
    frameworkIds.forEach(sourceId => {
      coverage[sourceId] = {};
      frameworkIds.forEach(targetId => {
        if (sourceId !== targetId) {
          const analysis = this.engine.analyzeGaps(sourceId, targetId);
          coverage[sourceId][targetId] = {
            percentage: Math.round(analysis.coveragePercentage * 100) / 100,
            mappedControls: analysis.mappedControls,
            totalControls: analysis.totalSourceControls
          };
        }
      });
    });

    return coverage;
  }

  /**
   * Identify significant gaps across frameworks
   */
  identifyGaps(frameworkIds) {
    const gaps = {};
    
    frameworkIds.forEach(sourceId => {
      gaps[sourceId] = {};
      frameworkIds.forEach(targetId => {
        if (sourceId !== targetId) {
          const analysis = this.engine.analyzeGaps(sourceId, targetId);
          gaps[sourceId][targetId] = analysis.gaps.source.filter(gap => 
            gap.riskLevel === 'high' || gap.riskLevel === 'critical'
          );
        }
      });
    });

    return gaps;
  }
}

// Create and export default instance
export const mappingSystem = new SecurityFrameworkMappings();