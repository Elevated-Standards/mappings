#!/usr/bin/env python3
"""
Simple example demonstrating the security frameworks mapping system.
"""

from mappings.system import mapping_system


def main():
    """Run the demo."""
    print("🔒 Security Frameworks Mapping System Demo\n")

    # 1. List all available frameworks
    print("📋 Available Frameworks:")
    frameworks = mapping_system.get_frameworks()
    for fw in frameworks:
        print(f"  • {fw.name} ({fw.id}) - {len(fw.get_all_controls())} controls")
    print()

    # 2. Analyze gaps between SOC 2 and ISO 27001
    print("🔍 Gap Analysis: SOC 2 → ISO 27001")
    soc2_to_iso = mapping_system.analyze_compliance("soc2", "iso27001")
    print(f"  Coverage: {soc2_to_iso.coverage_percentage:.1f}%")
    print(f"  Mapped Controls: {soc2_to_iso.mapped_controls}/{soc2_to_iso.total_source_controls}")
    print(f"  Unmapped: {len(soc2_to_iso.gaps['source'])} SOC 2 controls need additional work")
    print()

    # 3. Find mappings for a specific control
    print("🔗 Mappings for SOC 2 CC6.1 (Access Controls):")
    cc61_mappings = mapping_system.get_engine().find_mappings_for_control("soc2", "CC6.1")
    for mapping in cc61_mappings:
        target_fw = mapping.target_framework if mapping.source_framework == "soc2" else mapping.source_framework
        target_ctrl = mapping.target_control if mapping.source_framework == "soc2" else mapping.source_control
        print(f"  → {target_fw.upper()} {target_ctrl} [{mapping.mapping_type}, {mapping.confidence * 100:.0f}% confidence]")
    print()

    # 4. Generate compliance matrix
    print("📊 Compliance Coverage Matrix:")
    matrix = mapping_system.generate_compliance_matrix()
    framework_ids = matrix.frameworks

    # Print header
    header = "Source\\Target\t" + "".join(f"{fid:10}" for fid in framework_ids)
    print(header)

    # Print rows  
    for source in framework_ids:
        row = f"{source:15}"
        for target in framework_ids:
            if source == target:
                row += "    -     "
            else:
                coverage = matrix.matrix[source][target]["coverage"]
                row += f"{coverage:6.1f}%   "
        print(row)
    print()

    # 5. Find similar controls
    print("🔍 Similar Controls to SOC 2 CC6.1:")
    similar = mapping_system.find_potential_mappings("soc2", "CC6.1", 0.6)
    for match in similar[:3]:
        print(f"  • {match['framework_id'].upper()} {match['control_id']}: {match['similarity'] * 100:.0f}% similar")
        print(f"    Suggested mapping: {match['suggested_mapping_type']}")
    print()

    # 6. Identify critical gaps
    print("🚨 Critical Security Gaps to Address:")
    for source_id in ["soc2", "iso27001", "nist-csf"]:
        for target_id in ["soc2", "iso27001", "nist-csf"]:
            if source_id != target_id:
                analysis = mapping_system.analyze_compliance(source_id, target_id)
                critical_gaps = [gap for gap in analysis.gaps["target"] 
                               if gap["risk_level"] in ["critical", "high"]]
                
                if critical_gaps:
                    print(f"  {source_id.upper()} → {target_id.upper()}: {len(critical_gaps)} critical gaps")
                    for gap in critical_gaps[:2]:
                        print(f"    • {gap['id']}: {gap['title']}")
    print()

    # 7. Export data for further analysis
    print("💾 Exporting mapping data...")
    export_data = mapping_system.export()
    print(f"  Frameworks: {len(export_data['frameworks'])}")
    print(f"  Mappings: {len(export_data['mappings'])}")
    verified_mappings = len([m for m in export_data['mappings'] if m['verified']])
    print(f"  Verified mappings: {verified_mappings}")
    print()

    print("✅ Demo complete! Use the CLI tools for more detailed analysis:")
    print("  python -m mappings frameworks")
    print("  python -m mappings gaps soc2 iso27001")
    print("  python -m mappings report summary")


if __name__ == "__main__":
    main()