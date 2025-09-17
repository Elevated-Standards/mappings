#!/usr/bin/env python3
"""
Command-line interface for security frameworks mapping analysis.
"""

import click
import json
from pathlib import Path
from typing import Optional

from ..system import mapping_system


@click.group()
def cli():
    """Security Frameworks Mapping System CLI"""
    pass


@cli.command()
def frameworks():
    """List all available frameworks."""
    frameworks = mapping_system.get_frameworks()
    
    click.echo("\n📋 Available Security Frameworks:\n")
    
    for framework in frameworks:
        click.echo(f"• {framework.name} ({framework.id})")
        click.echo(f"  Version: {framework.version}")
        click.echo(f"  Description: {framework.description}")
        click.echo(f"  Controls: {len(framework.get_all_controls())}")
        click.echo(f"  Domains: {len(framework.get_all_domains())}\n")


@cli.command()
@click.argument('source')
@click.argument('target')
@click.option('--output', '-o', help='Output file path')
def gaps(source: str, target: str, output: Optional[str]):
    """Analyze gaps between two frameworks."""
    try:
        analysis = mapping_system.analyze_compliance(source, target)
        
        click.echo(f"\n🔍 Gap Analysis: {source.upper()} → {target.upper()}\n")
        click.echo(f"Coverage: {analysis.coverage_percentage:.1f}%")
        click.echo(f"Mapped Controls: {analysis.mapped_controls}/{analysis.total_source_controls}")
        click.echo(f"Unmapped Source Controls: {len(analysis.gaps['source'])}")
        click.echo(f"Unmapped Target Controls: {len(analysis.gaps['target'])}\n")
        
        # Show critical gaps
        critical_gaps = [gap for gap in analysis.gaps["source"] 
                        if gap["risk_level"] in ["critical", "high"]]
        
        if critical_gaps:
            click.echo("🚨 Critical Gaps in Source Framework:")
            for gap in critical_gaps:
                click.echo(f"  • {gap['id']}: {gap['title']} [{gap['risk_level'].upper()}]")
            click.echo()

        if output:
            output_path = Path(output)
            with open(output_path, 'w') as f:
                json.dump(analysis.dict(), f, indent=2, default=str)
            click.echo(f"📄 Analysis saved to: {output_path}")

    except Exception as error:
        click.echo(f"❌ Error: {error}")
        raise click.Abort()


@cli.command()
@click.option('--output', '-o', help='Output file path')
def matrix(output: Optional[str]):
    """Generate compliance matrix for all frameworks."""
    matrix = mapping_system.generate_compliance_matrix()
    
    click.echo("\n📊 Compliance Coverage Matrix:\n")
    
    # Print matrix header
    frameworks = matrix.frameworks
    header = "Source\\Target\t" + "".join(f"{f:12}" for f in frameworks)
    click.echo(header)
    
    # Print matrix rows
    for source in frameworks:
        row = f"{source:15}"
        for target in frameworks:
            if source == target:
                row += "     -      "
            else:
                coverage = matrix.matrix[source][target]["coverage"]
                row += f"{coverage:6.1f}%    "
        click.echo(row)

    if output:
        output_path = Path(output)
        with open(output_path, 'w') as f:
            json.dump(matrix.dict(), f, indent=2, default=str)
        click.echo(f"\n📄 Matrix saved to: {output_path}")


@cli.command()
@click.argument('framework')
@click.argument('control')
def mappings(framework: str, control: str):
    """Find mappings for a specific control."""
    mappings = mapping_system.get_engine().find_mappings_for_control(framework, control)
    framework_obj = mapping_system.get_engine().get_framework(framework)
    control_obj = framework_obj.get_control(control) if framework_obj else None

    if not control_obj:
        click.echo(f"❌ Control {control} not found in framework {framework}")
        raise click.Abort()

    click.echo(f"\n🔗 Mappings for {framework.upper()} {control}:\n")
    click.echo(f"Control: {control_obj.title}")
    click.echo(f"Description: {control_obj.description}\n")

    if not mappings:
        click.echo("No mappings found for this control.")
        return

    for mapping in mappings:
        target_framework = (mapping.target_framework if mapping.source_framework == framework 
                          else mapping.source_framework)
        target_control = (mapping.target_control if mapping.source_framework == framework
                         else mapping.source_control)
        
        target_fw = mapping_system.get_engine().get_framework(target_framework)
        target_ctrl = target_fw.get_control(target_control) if target_fw else None

        click.echo(f"• {target_framework.upper()} {target_control} [{mapping.mapping_type}]")
        if target_ctrl:
            click.echo(f"  {target_ctrl.title}")
        click.echo(f"  Confidence: {mapping.confidence * 100:.0f}%")
        if mapping.notes:
            click.echo(f"  Notes: {mapping.notes}")
        click.echo()


@cli.command()
@click.argument('framework')
@click.argument('control')
@click.option('--threshold', default=0.7, help='Similarity threshold (0.0-1.0)')
def similar(framework: str, control: str, threshold: float):
    """Find similar controls across frameworks."""
    similar_controls = mapping_system.find_potential_mappings(framework, control, threshold)
    framework_obj = mapping_system.get_engine().get_framework(framework)
    control_obj = framework_obj.get_control(control) if framework_obj else None

    if not control_obj:
        click.echo(f"❌ Control {control} not found in framework {framework}")
        raise click.Abort()

    click.echo(f"\n🔍 Similar Controls to {framework.upper()} {control}:\n")
    click.echo(f"Source: {control_obj.title}\n")

    if not similar_controls:
        click.echo(f"No similar controls found above threshold {threshold}")
        return

    for match in similar_controls:
        click.echo(f"• {match['framework_id'].upper()} {match['control_id']} [{match['similarity'] * 100:.0f}% similarity]")
        click.echo(f"  {match['title']}")
        click.echo(f"  Suggested mapping: {match['suggested_mapping_type']}\n")


if __name__ == '__main__':
    cli()