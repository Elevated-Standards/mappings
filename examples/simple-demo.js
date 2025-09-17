import { mappingSystem } from '../src/index.js';

/**
 * Simple example demonstrating the security frameworks mapping system
 */

console.log('🔒 Security Frameworks Mapping System Demo\n');

// 1. List all available frameworks
console.log('📋 Available Frameworks:');
const frameworks = mappingSystem.getFrameworks();
frameworks.forEach(fw => {
  console.log(`  • ${fw.name} (${fw.id}) - ${fw.getAllControls().length} controls`);
});
console.log();

// 2. Analyze gaps between SOC 2 and ISO 27001
console.log('🔍 Gap Analysis: SOC 2 → ISO 27001');
const soc2ToIso = mappingSystem.analyzeCompliance('soc2', 'iso27001');
console.log(`  Coverage: ${soc2ToIso.coveragePercentage.toFixed(1)}%`);
console.log(`  Mapped Controls: ${soc2ToIso.mappedControls}/${soc2ToIso.totalSourceControls}`);
console.log(`  Unmapped: ${soc2ToIso.gaps.source.length} SOC 2 controls need additional work`);
console.log();

// 3. Find mappings for a specific control
console.log('🔗 Mappings for SOC 2 CC6.1 (Access Controls):');
const cc61Mappings = mappingSystem.getEngine().findMappingsForControl('soc2', 'CC6.1');
cc61Mappings.forEach(mapping => {
  const targetFw = mapping.targetFramework === 'soc2' ? mapping.sourceFramework : mapping.targetFramework;
  const targetCtrl = mapping.targetFramework === 'soc2' ? mapping.sourceControl : mapping.targetControl;
  console.log(`  → ${targetFw.toUpperCase()} ${targetCtrl} [${mapping.mappingType}, ${(mapping.confidence * 100).toFixed(0)}% confidence]`);
});
console.log();

// 4. Generate compliance matrix
console.log('📊 Compliance Coverage Matrix:');
const matrix = mappingSystem.generateComplianceMatrix();
const frameworkIds = Object.keys(matrix);

// Print header
console.log('Source\\Target\t', frameworkIds.map(id => id.padEnd(10)).join(' '));

// Print rows  
frameworkIds.forEach(source => {
  let row = source.padEnd(15);
  frameworkIds.forEach(target => {
    if (source === target) {
      row += '    -     ';
    } else {
      const coverage = matrix[source][target]?.coverage || 0;
      row += `${coverage.toFixed(1)}%`.padEnd(10);
    }
  });
  console.log(row);
});
console.log();

// 5. Find similar controls
console.log('🔍 Similar Controls to SOC 2 CC6.1:');
const similar = mappingSystem.findPotentialMappings('soc2', 'CC6.1', 0.6);
similar.slice(0, 3).forEach(match => {
  console.log(`  • ${match.frameworkId.toUpperCase()} ${match.controlId}: ${(match.similarity * 100).toFixed(0)}% similar`);
  console.log(`    Suggested mapping: ${match.suggestedMappingType}`);
});
console.log();

// 6. Identify critical gaps
console.log('🚨 Critical Security Gaps to Address:');
['soc2', 'iso27001', 'nist-csf'].forEach(sourceId => {
  ['soc2', 'iso27001', 'nist-csf'].forEach(targetId => {
    if (sourceId !== targetId) {
      const analysis = mappingSystem.analyzeCompliance(sourceId, targetId);
      const criticalGaps = analysis.gaps.target.filter(gap => 
        gap.riskLevel === 'critical' || gap.riskLevel === 'high'
      );
      
      if (criticalGaps.length > 0) {
        console.log(`  ${sourceId.toUpperCase()} → ${targetId.toUpperCase()}: ${criticalGaps.length} critical gaps`);
        criticalGaps.slice(0, 2).forEach(gap => {
          console.log(`    • ${gap.id}: ${gap.title}`);
        });
      }
    }
  });
});
console.log();

// 7. Export data for further analysis
console.log('💾 Exporting mapping data...');
const exportData = mappingSystem.export();
console.log(`  Frameworks: ${exportData.frameworks.length}`);
console.log(`  Mappings: ${exportData.mappings.length}`);
console.log(`  Verified mappings: ${exportData.mappings.filter(m => m.verified).length}`);
console.log();

console.log('✅ Demo complete! Use the CLI tools for more detailed analysis:');
console.log('  npm run analyze frameworks');
console.log('  npm run analyze gaps soc2 iso27001');
console.log('  npm run report summary');