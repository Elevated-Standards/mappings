#!/usr/bin/env node

import yargs from 'yargs';
import { hideBin } from 'yargs/helpers';
import { mappingSystem } from '../index.js';
import fs from 'fs';
import path from 'path';

/**
 * CLI for generating compliance reports
 */
const argv = yargs(hideBin(process.argv))
  .command('full', 'Generate comprehensive compliance report', {
    output: {
      describe: 'Output file path',
      type: 'string',
      alias: 'o',
      default: 'compliance-report.json'
    },
    format: {
      describe: 'Output format',
      type: 'string',
      choices: ['json', 'csv', 'html'],
      default: 'json'
    }
  }, generateFullReport)
  .command('summary', 'Generate summary report', {}, generateSummary)
  .command('gaps', 'Generate gap analysis report for all framework pairs', {
    output: {
      describe: 'Output directory',
      type: 'string',
      alias: 'o',
      default: './reports'
    }
  }, generateGapReports)
  .demandCommand(1, 'You need to specify a report type')
  .help()
  .argv;

/**
 * Generate comprehensive compliance report
 */
function generateFullReport(argv) {
  console.log('📊 Generating comprehensive compliance report...\n');
  
  const report = mappingSystem.generateReport();
  
  // Ensure output directory exists
  const outputPath = path.resolve(argv.output);
  const outputDir = path.dirname(outputPath);
  if (!fs.existsSync(outputDir)) {
    fs.mkdirSync(outputDir, { recursive: true });
  }

  switch (argv.format) {
    case 'json':
      fs.writeFileSync(outputPath, JSON.stringify(report, null, 2));
      break;
    case 'csv':
      generateCSVReport(report, outputPath);
      break;
    case 'html':
      generateHTMLReport(report, outputPath);
      break;
  }

  console.log(`✅ Report generated: ${outputPath}`);
  
  // Print summary to console
  console.log('\n📋 Report Summary:');
  console.log(`• Frameworks analyzed: ${report.frameworks.length}`);
  console.log(`• Total mappings: ${report.mappings.total}`);
  console.log(`• Verified mappings: ${report.mappings.verified}`);
  console.log(`• Generated at: ${report.generatedAt}\n`);
}

/**
 * Generate summary report to console
 */
function generateSummary() {
  const report = mappingSystem.generateReport();
  
  console.log('\n📊 Security Frameworks Mapping Summary\n');
  console.log('='.repeat(50));
  
  console.log('\n🏗️  Frameworks:');
  report.frameworks.forEach(fw => {
    console.log(`• ${fw.name} (${fw.id}) - ${fw.totalControls} controls, ${fw.domains} domains`);
  });
  
  console.log('\n🔗 Mappings:');
  console.log(`• Total: ${report.mappings.total}`);
  console.log(`• Verified: ${report.mappings.verified} (${((report.mappings.verified / report.mappings.total) * 100).toFixed(1)}%)`);
  
  console.log('\n📈 Coverage by Framework:');
  Object.entries(report.coverage).forEach(([source, targets]) => {
    console.log(`\n${source.toUpperCase()}:`);
    Object.entries(targets).forEach(([target, data]) => {
      console.log(`  → ${target}: ${data.percentage}% (${data.mappedControls}/${data.totalControls})`);
    });
  });
  
  console.log('\n🚨 High-Priority Gaps:');
  Object.entries(report.gaps).forEach(([source, targets]) => {
    Object.entries(targets).forEach(([target, gaps]) => {
      if (gaps.length > 0) {
        console.log(`\n${source.toUpperCase()} → ${target.toUpperCase()}: ${gaps.length} critical gaps`);
        gaps.slice(0, 3).forEach(gap => {
          console.log(`  • ${gap.id}: ${gap.title}`);
        });
        if (gaps.length > 3) {
          console.log(`  ... and ${gaps.length - 3} more`);
        }
      }
    });
  });
}

/**
 * Generate gap analysis reports for all framework pairs
 */
function generateGapReports(argv) {
  console.log('📋 Generating gap analysis reports...\n');
  
  const outputDir = path.resolve(argv.output);
  if (!fs.existsSync(outputDir)) {
    fs.mkdirSync(outputDir, { recursive: true });
  }

  const frameworks = mappingSystem.getFrameworks();
  let reportCount = 0;

  frameworks.forEach(source => {
    frameworks.forEach(target => {
      if (source.id !== target.id) {
        const analysis = mappingSystem.analyzeCompliance(source.id, target.id);
        const filename = `gap-analysis-${source.id}-to-${target.id}.json`;
        const filepath = path.join(outputDir, filename);
        
        fs.writeFileSync(filepath, JSON.stringify(analysis, null, 2));
        reportCount++;
        
        console.log(`✅ Generated: ${filename}`);
      }
    });
  });

  console.log(`\n📊 Generated ${reportCount} gap analysis reports in ${outputDir}`);
}

/**
 * Generate CSV report
 */
function generateCSVReport(report, outputPath) {
  const csvPath = outputPath.replace(/\.[^/.]+$/, '.csv');
  
  // Generate mappings CSV
  const csvLines = ['Source Framework,Source Control,Target Framework,Target Control,Mapping Type,Confidence,Verified'];
  
  // This would need the actual mappings data from the engine
  const mappings = mappingSystem.getEngine().mappings;
  mappings.forEach(mapping => {
    csvLines.push([
      mapping.sourceFramework,
      mapping.sourceControl,
      mapping.targetFramework,
      mapping.targetControl,
      mapping.mappingType,
      mapping.confidence,
      mapping.verified
    ].join(','));
  });
  
  fs.writeFileSync(csvPath, csvLines.join('\n'));
}

/**
 * Generate HTML report
 */
function generateHTMLReport(report, outputPath) {
  const htmlPath = outputPath.replace(/\.[^/.]+$/, '.html');
  
  const html = `
<!DOCTYPE html>
<html>
<head>
    <title>Security Frameworks Compliance Report</title>
    <style>
        body { font-family: Arial, sans-serif; margin: 40px; }
        .header { background: #f5f5f5; padding: 20px; border-radius: 8px; }
        .framework { margin: 20px 0; padding: 15px; border: 1px solid #ddd; border-radius: 5px; }
        .coverage-matrix { border-collapse: collapse; width: 100%; margin: 20px 0; }
        .coverage-matrix th, .coverage-matrix td { border: 1px solid #ddd; padding: 8px; text-align: center; }
        .coverage-matrix th { background: #f2f2f2; }
        .high-coverage { background: #d4edda; }
        .medium-coverage { background: #fff3cd; }
        .low-coverage { background: #f8d7da; }
    </style>
</head>
<body>
    <div class="header">
        <h1>Security Frameworks Compliance Report</h1>
        <p>Generated: ${report.generatedAt}</p>
        <p>Total Mappings: ${report.mappings.total} (${report.mappings.verified} verified)</p>
    </div>
    
    <h2>Frameworks</h2>
    ${report.frameworks.map(fw => `
        <div class="framework">
            <h3>${fw.name} (${fw.id})</h3>
            <p>Version: ${fw.version}</p>
            <p>Controls: ${fw.totalControls} | Domains: ${fw.domains}</p>
        </div>
    `).join('')}
    
    <h2>Coverage Matrix</h2>
    <table class="coverage-matrix">
        <tr>
            <th>Source / Target</th>
            ${report.frameworks.map(fw => `<th>${fw.id}</th>`).join('')}
        </tr>
        ${report.frameworks.map(source => `
            <tr>
                <th>${source.id}</th>
                ${report.frameworks.map(target => {
                  if (source.id === target.id) return '<td>-</td>';
                  const coverage = report.coverage[source.id]?.[target.id]?.percentage || 0;
                  const cssClass = coverage > 70 ? 'high-coverage' : coverage > 40 ? 'medium-coverage' : 'low-coverage';
                  return `<td class="${cssClass}">${coverage.toFixed(1)}%</td>`;
                }).join('')}
            </tr>
        `).join('')}
    </table>
</body>
</html>`;
  
  fs.writeFileSync(htmlPath, html);
}