# Security Frameworks Mapping Examples

This directory contains practical examples of how to use the security frameworks mapping system.

## Quick Start Example

```javascript
import { mappingSystem } from '../src/index.js';

// Get all available frameworks
const frameworks = mappingSystem.getFrameworks();
console.log('Available frameworks:', frameworks.map(f => f.name));

// Analyze compliance gaps between SOC 2 and ISO 27001
const gapAnalysis = mappingSystem.analyzeCompliance('soc2', 'iso27001');
console.log(`Coverage: ${gapAnalysis.coveragePercentage.toFixed(1)}%`);

// Find mappings for a specific control
const mappings = mappingSystem.getEngine().findMappingsForControl('soc2', 'CC6.1');
console.log('Mappings found:', mappings.length);

// Generate comprehensive compliance matrix
const matrix = mappingSystem.generateComplianceMatrix();
console.log('Compliance matrix generated');
```

## Command Line Examples

### Analyze gaps between frameworks
```bash
npm run analyze gaps soc2 iso27001
npm run analyze gaps iso27001 nist-csf
```

### Find mappings for specific controls
```bash
npm run analyze mappings soc2 CC6.1
npm run analyze mappings iso27001 A.9.1.1
```

### Generate compliance reports
```bash
npm run report full --output compliance-report.json
npm run report summary
npm run report gaps --output ./gap-reports
```

### Find similar controls
```bash
npm run analyze similar soc2 CC6.1 --threshold 0.7
```

## Use Case Examples

### 1. SOC 2 to ISO 27001 Compliance Assessment

```javascript
// Assess how well SOC 2 controls cover ISO 27001 requirements
const analysis = mappingSystem.analyzeCompliance('soc2', 'iso27001');

console.log(`SOC 2 covers ${analysis.coveragePercentage.toFixed(1)}% of ISO 27001 requirements`);
console.log(`Gaps to address: ${analysis.gaps.target.length} ISO 27001 controls`);

// Focus on high-risk gaps
const criticalGaps = analysis.gaps.target.filter(gap => 
  gap.riskLevel === 'critical' || gap.riskLevel === 'high'
);

console.log('Critical gaps to prioritize:');
criticalGaps.forEach(gap => {
  console.log(`- ${gap.id}: ${gap.title}`);
});
```

### 2. Multi-Framework Compliance Strategy

```javascript
// Generate compliance matrix for strategic planning
const matrix = mappingSystem.generateComplianceMatrix();

// Identify frameworks with highest coverage overlap
const frameworks = ['soc2', 'iso27001', 'nist-csf'];
let bestCoverage = 0;
let recommendedPair = null;

frameworks.forEach(source => {
  frameworks.forEach(target => {
    if (source !== target) {
      const coverage = matrix[source][target]?.coverage || 0;
      if (coverage > bestCoverage) {
        bestCoverage = coverage;
        recommendedPair = { source, target, coverage };
      }
    }
  });
});

console.log(`Recommended compliance path: ${recommendedPair.source} → ${recommendedPair.target}`);
console.log(`Coverage: ${recommendedPair.coverage.toFixed(1)}%`);
```

### 3. Automated Compliance Mapping

```javascript
// Find potential new mappings using similarity analysis
const potentialMappings = mappingSystem.findPotentialMappings('soc2', 'CC1.1', 0.6);

console.log('Suggested new mappings:');
potentialMappings.forEach(match => {
  console.log(`${match.frameworkId} ${match.controlId}: ${(match.similarity * 100).toFixed(0)}% similar`);
  console.log(`Suggested type: ${match.suggestedMappingType}`);
});
```

### 4. Custom Framework Integration

```javascript
import { SecurityFramework, SecurityControl } from '../src/core/models.js';

// Create a custom framework
const customFramework = new SecurityFramework(
  'custom-fw',
  'Custom Security Framework',
  '1.0',
  'Organization-specific security requirements'
);

// Add custom controls
const customControl = new SecurityControl(
  'C-1.1',
  'Data Classification Policy',
  'All data must be classified according to sensitivity levels',
  'custom-fw'
);

customFramework.addControl(customControl);
mappingSystem.getEngine().registerFramework(customFramework);

// Map to existing frameworks
mappingSystem.getEngine().addMapping('custom-fw', 'C-1.1', 'iso27001', 'A.8.2.1', 'equivalent', 0.9);
```

## Integration Examples

### CI/CD Pipeline Integration

```yaml
# .github/workflows/compliance-check.yml
name: Compliance Assessment
on: [push, pull_request]

jobs:
  compliance:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v2
      - uses: actions/setup-node@v2
        with:
          node-version: '18'
      
      - name: Install dependencies
        run: npm install
      
      - name: Run compliance analysis
        run: |
          npm run analyze gaps soc2 iso27001 --output soc2-iso27001-gaps.json
          npm run report summary
      
      - name: Upload compliance report
        uses: actions/upload-artifact@v2
        with:
          name: compliance-reports
          path: "*.json"
```

### API Integration Example

```javascript
import express from 'express';
import { mappingSystem } from '../src/index.js';

const app = express();

// REST API endpoints
app.get('/api/frameworks', (req, res) => {
  const frameworks = mappingSystem.getFrameworks();
  res.json(frameworks.map(f => ({
    id: f.id,
    name: f.name,
    version: f.version,
    controlCount: f.getAllControls().length
  })));
});

app.get('/api/gaps/:source/:target', (req, res) => {
  try {
    const analysis = mappingSystem.analyzeCompliance(req.params.source, req.params.target);
    res.json(analysis);
  } catch (error) {
    res.status(400).json({ error: error.message });
  }
});

app.listen(3000, () => {
  console.log('Compliance API server running on port 3000');
});
```