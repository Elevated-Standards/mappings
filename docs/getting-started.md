# Getting Started with Security Frameworks Mappings

This guide will help you get started with using the security frameworks mapping system to analyze compliance gaps and relationships between different security standards.

## Installation

```bash
git clone https://github.com/Elevated-Standards/mappings.git
cd mappings
pip install -r requirements.txt
```

## Basic Usage

### 1. List Available Frameworks

```bash
python -m mappings frameworks
```

This will show all supported security frameworks including:
- SOC 2 (System and Organization Controls 2)
- ISO 27001 (Information Security Management System)
- NIST Cybersecurity Framework

### 2. Analyze Gaps Between Frameworks

```bash
python -m mappings gaps soc2 iso27001
python -m mappings gaps iso27001 nist-csf
```

Gap analysis shows:
- Coverage percentage between frameworks
- Number of mapped vs unmapped controls
- Critical gaps that need attention

### 3. Find Mappings for Specific Controls

```bash
python -m mappings mappings soc2 CC6.1
python -m mappings mappings iso27001 A.9.1.1
```

This shows how specific controls map to other frameworks and the confidence level of each mapping.

### 4. Generate Compliance Reports

```bash
# Generate comprehensive report
python -m mappings report full --output compliance-report.json

# Generate summary to console
python -m mappings report summary

# Generate gap analysis reports for all framework pairs
python -m mappings report gaps --output ./gap-reports
```

## Programming Interface

### Basic Framework Analysis

```python
from mappings.system import mapping_system

# Get all frameworks
frameworks = mapping_system.get_frameworks()
print('Available frameworks:', [f.name for f in frameworks])

# Analyze compliance gaps
analysis = mapping_system.analyze_compliance('soc2', 'iso27001')
print(f'SOC 2 covers {analysis.coverage_percentage:.1f}% of ISO 27001')

# Find high-priority gaps
critical_gaps = [gap for gap in analysis.gaps["target"] 
                 if gap["risk_level"] in ["critical", "high"]]
print('Critical gaps to address:', len(critical_gaps))
```

### Finding Control Mappings

```python
# Find mappings for a specific control
mappings = mapping_system.get_engine().find_mappings_for_control('soc2', 'CC6.1')
for mapping in mappings:
    print(f'{mapping.target_framework} {mapping.target_control}: {mapping.mapping_type}')

# Find similar controls using AI-powered analysis
similar = mapping_system.find_potential_mappings('soc2', 'CC6.1', 0.7)
for match in similar:
    print(f'{match["framework_id"]} {match["control_id"]}: {match["similarity"] * 100:.0f}% similar')
```

### Generate Compliance Matrix

```python
# Generate cross-framework compliance matrix
matrix = mapping_system.generate_compliance_matrix()

# Find best compliance path
best_coverage = 0
recommended_path = None
for source in matrix.frameworks:
    for target in matrix.frameworks:
        if source != target:
            coverage = matrix.matrix[source][target]["coverage"]
            if coverage > best_coverage:
                best_coverage = coverage
                recommended_path = {"source": source, "target": target, "coverage": coverage}

print(f'Recommended: {recommended_path["source"]} → {recommended_path["target"]} ({recommended_path["coverage"]:.1f}%)')
```

## Common Use Cases

### 1. SOC 2 to ISO 27001 Migration

```bash
# Analyze what SOC 2 controls you have that map to ISO 27001
python -m mappings gaps soc2 iso27001 --output soc2-iso27001-analysis.json

# Find specific mappings for key SOC 2 controls
python -m mappings mappings soc2 CC6.1  # Access controls
python -m mappings mappings soc2 CC7.1  # Monitoring
```

### 2. Multi-Framework Compliance Strategy

```bash
# Generate comprehensive coverage matrix
python -m mappings report full --format html --output compliance-dashboard.html

# Identify framework pairs with highest overlap
python -m mappings report summary
```

### 3. Control Mapping Validation

```bash
# Find potential new mappings using similarity analysis
python -m mappings similar soc2 CC1.1 --threshold 0.6
python -m mappings similar iso27001 A.5.1.1 --threshold 0.7
```

## Framework Coverage

| Framework | Controls | Domains | Key Focus Areas |
|-----------|----------|---------|-----------------|
| SOC 2 | 12 | 5 | Trust services, operational controls |
| ISO 27001 | 14 | 8 | Information security management |
| NIST CSF | 15 | 5 | Cybersecurity framework functions |

## Mapping Confidence Levels

- **Equivalent (90%+)**: Controls address the same requirement
- **Related (70-89%)**: Controls are related but may have different scope
- **Partial (50-69%)**: Controls overlap but significant gaps remain
- **Informational (<50%)**: Controls are tangentially related

## Requirements

- Python 3.8+
- Dependencies listed in requirements.txt

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