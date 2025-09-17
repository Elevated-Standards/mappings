#!/usr/bin/env node

import yargs from 'yargs';
import { hideBin } from 'yargs/helpers';
import { mappingSystem } from '../index.js';
import fs from 'fs';
import path from 'path';

/**
 * CLI for analyzing security framework mappings and gaps
 */
const argv = yargs(hideBin(process.argv))
  .command('frameworks', 'List all available frameworks', {}, listFrameworks)
  .command('gaps <source> <target>', 'Analyze gaps between two frameworks', {
    source: {
      describe: 'Source framework ID',
      type: 'string',
      demandOption: true
    },
    target: {
      describe: 'Target framework ID', 
      type: 'string',
      demandOption: true
    },
    output: {
      describe: 'Output file path',
      type: 'string',
      alias: 'o'
    }
  }, analyzeGaps)
  .command('matrix', 'Generate compliance matrix for all frameworks', {
    output: {
      describe: 'Output file path',
      type: 'string',
      alias: 'o'
    }
  }, generateMatrix)
  .command('mappings <framework> <control>', 'Find mappings for a specific control', {
    framework: {
      describe: 'Framework ID',
      type: 'string',
      demandOption: true
    },
    control: {
      describe: 'Control ID',
      type: 'string', 
      demandOption: true
    }
  }, findMappings)
  .command('similar <framework> <control>', 'Find similar controls across frameworks', {
    framework: {
      describe: 'Framework ID',
      type: 'string',
      demandOption: true
    },
    control: {
      describe: 'Control ID',
      type: 'string',
      demandOption: true
    },
    threshold: {
      describe: 'Similarity threshold (0.0-1.0)',
      type: 'number',
      default: 0.7
    }
  }, findSimilar)
  .demandCommand(1, 'You need to specify a command')
  .help()
  .argv;

/**
 * List all available frameworks
 */
function listFrameworks() {
  const frameworks = mappingSystem.getFrameworks();
  
  console.log('\n📋 Available Security Frameworks:\n');
  
  frameworks.forEach(framework => {
    console.log(`• ${framework.name} (${framework.id})`);
    console.log(`  Version: ${framework.version}`);
    console.log(`  Description: ${framework.description}`);
    console.log(`  Controls: ${framework.getAllControls().length}`);
    console.log(`  Domains: ${framework.getAllDomains().length}\n`);
  });
}

/**
 * Analyze gaps between two frameworks
 */
function analyzeGaps(argv) {
  try {
    const analysis = mappingSystem.analyzeCompliance(argv.source, argv.target);
    
    console.log(`\n🔍 Gap Analysis: ${argv.source.toUpperCase()} → ${argv.target.toUpperCase()}\n`);
    console.log(`Coverage: ${analysis.coveragePercentage.toFixed(1)}%`);
    console.log(`Mapped Controls: ${analysis.mappedControls}/${analysis.totalSourceControls}`);
    console.log(`Unmapped Source Controls: ${analysis.gaps.source.length}`);
    console.log(`Unmapped Target Controls: ${analysis.gaps.target.length}\n`);
    
    if (analysis.gaps.source.length > 0) {
      console.log('🚨 Critical Gaps in Source Framework:');
      analysis.gaps.source
        .filter(gap => gap.riskLevel === 'critical' || gap.riskLevel === 'high')
        .forEach(gap => {
          console.log(`  • ${gap.id}: ${gap.title} [${gap.riskLevel.toUpperCase()}]`);
        });
      console.log();
    }

    if (argv.output) {
      const outputPath = path.resolve(argv.output);
      fs.writeFileSync(outputPath, JSON.stringify(analysis, null, 2));
      console.log(`📄 Analysis saved to: ${outputPath}`);
    }

  } catch (error) {
    console.error(`❌ Error: ${error.message}`);
    process.exit(1);
  }
}

/**
 * Generate compliance matrix
 */
function generateMatrix(argv) {
  const matrix = mappingSystem.generateComplianceMatrix();
  
  console.log('\n📊 Compliance Coverage Matrix:\n');
  
  // Print matrix header
  const frameworks = Object.keys(matrix);
  console.log('Source\\Target\t', frameworks.map(f => f.padEnd(12)).join(''));
  
  // Print matrix rows
  frameworks.forEach(source => {
    let row = source.padEnd(15);
    frameworks.forEach(target => {
      if (source === target) {
        row += '     -      ';
      } else {
        const coverage = matrix[source][target]?.coverage || 0;
        row += `${coverage.toFixed(1)}%`.padEnd(11);
      }
    });
    console.log(row);
  });

  if (argv.output) {
    const outputPath = path.resolve(argv.output);
    fs.writeFileSync(outputPath, JSON.stringify(matrix, null, 2));
    console.log(`\n📄 Matrix saved to: ${outputPath}`);
  }
}

/**
 * Find mappings for a specific control
 */
function findMappings(argv) {
  const mappings = mappingSystem.getEngine().findMappingsForControl(argv.framework, argv.control);
  const framework = mappingSystem.getEngine().getFramework(argv.framework);
  const control = framework?.getControl(argv.control);

  if (!control) {
    console.error(`❌ Control ${argv.control} not found in framework ${argv.framework}`);
    process.exit(1);
  }

  console.log(`\n🔗 Mappings for ${argv.framework.toUpperCase()} ${argv.control}:\n`);
  console.log(`Control: ${control.title}`);
  console.log(`Description: ${control.description}\n`);

  if (mappings.length === 0) {
    console.log('No mappings found for this control.');
    return;
  }

  mappings.forEach(mapping => {
    const targetFramework = mapping.sourceFramework === argv.framework ? 
      mapping.targetFramework : mapping.sourceFramework;
    const targetControl = mapping.sourceFramework === argv.framework ?
      mapping.targetControl : mapping.sourceControl;
    
    const targetFw = mappingSystem.getEngine().getFramework(targetFramework);
    const targetCtrl = targetFw?.getControl(targetControl);

    console.log(`• ${targetFramework.toUpperCase()} ${targetControl} [${mapping.mappingType}]`);
    if (targetCtrl) {
      console.log(`  ${targetCtrl.title}`);
    }
    console.log(`  Confidence: ${(mapping.confidence * 100).toFixed(0)}%`);
    if (mapping.notes) {
      console.log(`  Notes: ${mapping.notes}`);
    }
    console.log();
  });
}

/**
 * Find similar controls across frameworks
 */
function findSimilar(argv) {
  const similar = mappingSystem.findPotentialMappings(argv.framework, argv.control, argv.threshold);
  const framework = mappingSystem.getEngine().getFramework(argv.framework);
  const control = framework?.getControl(argv.control);

  if (!control) {
    console.error(`❌ Control ${argv.control} not found in framework ${argv.framework}`);
    process.exit(1);
  }

  console.log(`\n🔍 Similar Controls to ${argv.framework.toUpperCase()} ${argv.control}:\n`);
  console.log(`Source: ${control.title}\n`);

  if (similar.length === 0) {
    console.log(`No similar controls found above threshold ${argv.threshold}`);
    return;
  }

  similar.forEach(match => {
    console.log(`• ${match.frameworkId.toUpperCase()} ${match.controlId} [${(match.similarity * 100).toFixed(0)}% similarity]`);
    console.log(`  ${match.title}`);
    console.log(`  Suggested mapping: ${match.suggestedMappingType}\n`);
  });
}