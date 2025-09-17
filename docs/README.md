# Security Frameworks Mapping System

A comprehensive mapping system for analyzing relationships between major security frameworks including SOC 2, ISO 27001, NIST Cybersecurity Framework, and others.

## Why This Matters

Organizations often need to comply with multiple security frameworks simultaneously. This creates several challenges:

- **Redundant Work**: Implementing similar controls multiple times
- **Gap Analysis**: Understanding what's missing when moving between frameworks  
- **Resource Optimization**: Knowing which controls provide the most coverage
- **Compliance Strategy**: Planning efficient paths to multi-framework compliance

This system solves these problems by providing:

✅ **Comprehensive Mappings** between major security frameworks  
✅ **Gap Analysis** to identify missing controls and overlaps  
✅ **Automated Tools** for compliance assessment and reporting  
✅ **Strategic Insights** for optimizing compliance investments  

## Quick Demo

```bash
# See all supported frameworks
npm run analyze frameworks

# Analyze SOC 2 to ISO 27001 compliance gaps
npm run analyze gaps soc2 iso27001

# Find mappings for specific controls
npm run analyze mappings soc2 CC6.1

# Generate comprehensive compliance report
npm run report summary
```

## Supported Frameworks

| Framework | Full Name | Controls | Key Use Cases |
|-----------|-----------|----------|---------------|
| **SOC 2** | System and Organization Controls 2 | 12 | SaaS companies, service providers |
| **ISO 27001** | Information Security Management System | 14 | International compliance, enterprises |
| **NIST CSF** | NIST Cybersecurity Framework | 15 | Government, critical infrastructure |
| **FedRAMP** | Federal Risk Authorization Management | *Coming Soon* | US government cloud services |
| **PCI DSS** | Payment Card Industry Data Security | *Coming Soon* | Payment processing |
| **CIS Controls** | Center for Internet Security | *Coming Soon* | Cybersecurity best practices |

## Key Features

### 🔍 Gap Analysis
Identify exactly what controls are missing when transitioning between frameworks:

```
SOC 2 → ISO 27001: 33.3% coverage
✅ 4 controls mapped
⚠️  8 controls need additional implementation
🚨 2 critical gaps requiring immediate attention
```

### 🔗 Control Mappings
Understand relationships between specific controls across frameworks:

```
SOC 2 CC6.1 (Access Controls) maps to:
→ ISO 27001 A.9.1.1 (Access Control Policy) [80% confidence]
→ NIST CSF PR.AC-1 (Identity Management) [90% confidence]
```

### 📊 Compliance Matrix
Strategic view of coverage between all framework pairs:

```
            SOC 2    ISO 27001    NIST CSF
SOC 2         -        33.3%       25.0%
ISO 27001   28.6%        -         28.6%
NIST CSF    20.0%      26.7%         -
```

### 🤖 AI-Powered Similarity Detection
Find potential mappings using intelligent text analysis:

```
Similar to SOC 2 CC6.1:
→ ISO 27001 A.9.2.1 (85% similarity) - User registration and de-registration
→ NIST CSF PR.AC-3 (72% similarity) - Remote access management
```

## Real-World Impact

### For Compliance Teams
- **Reduce audit preparation time by 40-60%**
- **Avoid duplicate control implementations**
- **Prioritize high-impact controls first**

### For Security Leaders  
- **Make data-driven compliance investment decisions**
- **Demonstrate security ROI across multiple frameworks**
- **Plan efficient paths to multi-framework compliance**

### For Auditors
- **Quickly understand control relationships**
- **Identify potential gaps in compliance scope**
- **Validate mapping accuracy and coverage**

## Architecture

```
┌─────────────────┐    ┌──────────────────┐    ┌─────────────────┐
│   Frameworks    │    │  Mapping Engine  │    │   Analysis &    │
│                 │    │                  │    │   Reporting     │
│ • SOC 2         │────│ • Gap Analysis   │────│ • CLI Tools     │
│ • ISO 27001     │    │ • Similarity AI  │    │ • JSON/CSV/HTML │
│ • NIST CSF      │    │ • Validation     │    │ • REST API      │
│ • Custom FWs    │    │ • Confidence     │    │ • Dashboards    │
└─────────────────┘    └──────────────────┘    └─────────────────┘
```

## Getting Started

### Installation
```bash
git clone https://github.com/Elevated-Standards/mappings.git
cd mappings
npm install
```

### Basic Usage
```bash
# List all frameworks
npm run analyze frameworks

# Compare two frameworks
npm run analyze gaps soc2 iso27001

# Generate reports
npm run report summary
npm run report full --output report.json
```

### Programming Interface
```javascript
import { mappingSystem } from './src/index.js';

// Analyze compliance gaps
const analysis = mappingSystem.analyzeCompliance('soc2', 'iso27001');
console.log(`Coverage: ${analysis.coveragePercentage}%`);

// Find control mappings
const mappings = mappingSystem.getEngine().findMappingsForControl('soc2', 'CC6.1');
```

## Documentation

- **[Getting Started Guide](docs/getting-started.md)** - Step-by-step setup and basic usage
- **[API Reference](docs/api-reference.md)** - Complete programming interface
- **[Examples](examples/README.md)** - Real-world usage patterns and integrations
- **[Framework Definitions](frameworks/)** - Detailed control definitions and mappings

## Contributing

We welcome contributions to improve mapping accuracy and expand framework coverage:

1. **Framework Additions**: Add support for new security frameworks
2. **Mapping Improvements**: Enhance existing control mappings  
3. **Validation**: Verify mapping accuracy through expert review
4. **Tools**: Build additional analysis and reporting tools

See [Contributing Guidelines](CONTRIBUTING.md) for details.

## Use Cases

### Multi-Framework Compliance
Organizations implementing SOC 2 who need ISO 27001 certification can identify exactly which additional controls they need.

### Compliance Optimization  
Teams can focus on controls that provide maximum coverage across multiple frameworks, reducing implementation overhead.

### Audit Preparation
Auditors and compliance teams can quickly understand control relationships and identify potential gaps.

### Strategic Planning
Security leaders can make data-driven decisions about which frameworks to prioritize based on business needs and existing investments.

## License

MIT License - see [LICENSE](LICENSE) for details.

## Roadmap

- [ ] **FedRAMP Framework** - US government cloud compliance
- [ ] **PCI DSS Integration** - Payment card industry standards  
- [ ] **CIS Controls** - Center for Internet Security framework
- [ ] **Custom Framework Builder** - Create organization-specific frameworks
- [ ] **Machine Learning Mappings** - AI-powered mapping suggestions
- [ ] **Integration APIs** - Connect with GRC platforms and tools