# Getting Started with Security Frameworks Mappings

This guide will help you get started with using the security frameworks mapping system to analyze compliance gaps and relationships between different security standards.

## Installation

```bash
git clone https://github.com/Elevated-Standards/mappings.git
cd mappings
npm install
```

## Basic Usage

### 1. List Available Frameworks

```bash
npm run analyze frameworks
```

This will show all supported security frameworks including:
- SOC 2 (System and Organization Controls 2)
- ISO 27001 (Information Security Management System)
- NIST Cybersecurity Framework

### 2. Analyze Gaps Between Frameworks

```bash
npm run analyze gaps soc2 iso27001
npm run analyze gaps iso27001 nist-csf
```

Gap analysis shows:
- Coverage percentage between frameworks
- Number of mapped vs unmapped controls
- Critical gaps that need attention

### 3. Find Mappings for Specific Controls

```bash
npm run analyze mappings soc2 CC6.1
npm run analyze mappings iso27001 A.9.1.1
```

This shows how specific controls map to other frameworks and the confidence level of each mapping.

### 4. Generate Compliance Reports

```bash
# Generate comprehensive report
npm run report full --output compliance-report.json

# Generate summary to console
npm run report summary

# Generate gap analysis reports for all framework pairs
npm run report gaps --output ./gap-reports
```

## Programming Interface

### Basic Framework Analysis

```javascript
import { mappingSystem } from './src/index.js';

// Get all frameworks
const frameworks = mappingSystem.getFrameworks();
console.log('Available frameworks:', frameworks.map(f => f.name));

// Analyze compliance gaps
const analysis = mappingSystem.analyzeCompliance('soc2', 'iso27001');
console.log(`SOC 2 covers ${analysis.coveragePercentage.toFixed(1)}% of ISO 27001`);

// Find high-priority gaps
const criticalGaps = analysis.gaps.target.filter(gap => 
  gap.riskLevel === 'critical' || gap.riskLevel === 'high'
);
console.log('Critical gaps to address:', criticalGaps.length);
```

### Finding Control Mappings

```javascript
// Find mappings for a specific control
const mappings = mappingSystem.getEngine().findMappingsForControl('soc2', 'CC6.1');
mappings.forEach(mapping => {
  console.log(`${mapping.targetFramework} ${mapping.targetControl}: ${mapping.mappingType}`);
});

// Find similar controls using AI-powered analysis
const similar = mappingSystem.findPotentialMappings('soc2', 'CC6.1', 0.7);
similar.forEach(match => {
  console.log(`${match.frameworkId} ${match.controlId}: ${(match.similarity * 100).toFixed(0)}% similar`);
});
```

### Generate Compliance Matrix

```javascript
// Generate cross-framework compliance matrix
const matrix = mappingSystem.generateComplianceMatrix();

// Find best compliance path
let bestCoverage = 0;
let recommendedPath = null;
Object.entries(matrix).forEach(([source, targets]) => {
  Object.entries(targets).forEach(([target, data]) => {
    if (data.coverage > bestCoverage) {
      bestCoverage = data.coverage;
      recommendedPath = { source, target, coverage: data.coverage };
    }
  });
});

console.log(`Recommended: ${recommendedPath.source} → ${recommendedPath.target} (${recommendedPath.coverage.toFixed(1)}%)`);
```

## Common Use Cases

### 1. SOC 2 to ISO 27001 Migration

```bash
# Analyze what SOC 2 controls you have that map to ISO 27001
npm run analyze gaps soc2 iso27001 --output soc2-iso27001-analysis.json

# Find specific mappings for key SOC 2 controls
npm run analyze mappings soc2 CC6.1  # Access controls
npm run analyze mappings soc2 CC7.1  # Monitoring
```

### 2. Multi-Framework Compliance Strategy

```bash
# Generate comprehensive coverage matrix
npm run report full --format html --output compliance-dashboard.html

# Identify framework pairs with highest overlap
npm run report summary
```

### 3. Control Mapping Validation

```bash
# Find potential new mappings using similarity analysis
npm run analyze similar soc2 CC1.1 --threshold 0.6
npm run analyze similar iso27001 A.5.1.1 --threshold 0.7
```

## Framework Coverage

| Framework | Controls | Domains | Key Focus Areas |
|-----------|----------|---------|-----------------|
| SOC 2 | 12 | 5 | Trust services, operational controls |
| ISO 27001 | 14 | 14 | Information security management |
| NIST CSF | 15 | 5 | Cybersecurity framework functions |

## Mapping Confidence Levels

- **Equivalent (90%+)**: Controls address the same requirement
- **Related (70-89%)**: Controls are related but may have different scope
- **Partial (50-69%)**: Controls overlap but significant gaps remain
- **Informational (<50%)**: Controls are tangentially related

## Next Steps

1. **Explore Examples**: Check the `examples/` directory for detailed use cases
2. **Customize Mappings**: Add your organization-specific framework mappings
3. **Integrate with CI/CD**: Use the CLI tools in your compliance automation pipeline
4. **Contribute**: Help improve mapping accuracy by submitting feedback and corrections

## Support

For questions, issues, or contributions:
- GitHub Issues: Report bugs or request features
- Documentation: See `docs/` directory for detailed API reference
- Examples: Check `examples/` for real-world usage patterns