#!/usr/bin/env python3
"""
CLI for generating compliance reports.
"""

import click
import json
from pathlib import Path
from typing import Optional
from datetime import datetime

from ..system import mapping_system


@click.group()
def report():
    """Generate compliance reports."""
    pass


@report.command()
@click.option('--output', '-o', default='compliance-report.json', help='Output file path')
@click.option('--format', 'output_format', type=click.Choice(['json', 'html']), default='json', help='Output format')
def full(output: str, output_format: str):
    """Generate comprehensive compliance report."""
    click.echo("📊 Generating comprehensive compliance report...\n")
    
    report_data = mapping_system.generate_report()
    
    output_path = Path(output)
    output_path.parent.mkdir(parents=True, exist_ok=True)

    if output_format == 'json':
        with open(output_path, 'w') as f:
            json.dump(report_data, f, indent=2, default=str)
    elif output_format == 'html':
        html_path = output_path.with_suffix('.html')
        _generate_html_report(report_data, html_path)
        output_path = html_path

    click.echo(f"✅ Report generated: {output_path}")
    
    # Print summary to console
    click.echo("\n📋 Report Summary:")
    click.echo(f"• Frameworks analyzed: {len(report_data['frameworks'])}")
    click.echo(f"• Total mappings: {report_data['mappings']['total']}")
    click.echo(f"• Verified mappings: {report_data['mappings']['verified']}")
    click.echo(f"• Generated at: {report_data['generated_at']}\n")


@report.command()
def summary():
    """Generate summary report to console."""
    report_data = mapping_system.generate_report()
    
    click.echo("\n📊 Security Frameworks Mapping Summary\n")
    click.echo("=" * 50)
    
    click.echo("\n🏗️  Frameworks:")
    for fw in report_data["frameworks"]:
        click.echo(f"• {fw['name']} ({fw['id']}) - {fw['total_controls']} controls, {fw['domains']} domains")
    
    click.echo("\n🔗 Mappings:")
    click.echo(f"• Total: {report_data['mappings']['total']}")
    click.echo(f"• Verified: {report_data['mappings']['verified']} ({(report_data['mappings']['verified'] / report_data['mappings']['total'] * 100):.1f}%)")
    
    click.echo("\n📈 Coverage by Framework:")
    for source, targets in report_data["coverage"].items():
        click.echo(f"\n{source.upper()}:")
        for target, data in targets.items():
            click.echo(f"  → {target}: {data['percentage']}% ({data['mapped_controls']}/{data['total_controls']})")
    
    click.echo("\n🚨 High-Priority Gaps:")
    for source, targets in report_data["gaps"].items():
        for target, gaps in targets.items():
            if gaps:
                click.echo(f"\n{source.upper()} → {target.upper()}: {len(gaps)} critical gaps")
                for gap in gaps[:3]:  # Show first 3
                    click.echo(f"  • {gap['id']}: {gap['title']}")
                if len(gaps) > 3:
                    click.echo(f"  ... and {len(gaps) - 3} more")


@report.command()
@click.option('--output', '-o', default='./gap-reports', help='Output directory')
def gaps(output: str):
    """Generate gap analysis reports for all framework pairs."""
    click.echo("📋 Generating gap analysis reports...\n")
    
    output_dir = Path(output)
    output_dir.mkdir(parents=True, exist_ok=True)

    frameworks = mapping_system.get_frameworks()
    report_count = 0

    for source in frameworks:
        for target in frameworks:
            if source.id != target.id:
                analysis = mapping_system.analyze_compliance(source.id, target.id)
                filename = f"gap-analysis-{source.id}-to-{target.id}.json"
                filepath = output_dir / filename
                
                with open(filepath, 'w') as f:
                    json.dump(analysis.dict(), f, indent=2, default=str)
                
                report_count += 1
                click.echo(f"✅ Generated: {filename}")

    click.echo(f"\n📊 Generated {report_count} gap analysis reports in {output_dir}")


def _generate_html_report(report_data: dict, output_path: Path):
    """Generate HTML report."""
    html_content = f"""
<!DOCTYPE html>
<html>
<head>
    <title>Security Frameworks Compliance Report</title>
    <style>
        body {{ font-family: Arial, sans-serif; margin: 40px; }}
        .header {{ background: #f5f5f5; padding: 20px; border-radius: 8px; }}
        .framework {{ margin: 20px 0; padding: 15px; border: 1px solid #ddd; border-radius: 5px; }}
        .coverage-matrix {{ border-collapse: collapse; width: 100%; margin: 20px 0; }}
        .coverage-matrix th, .coverage-matrix td {{ border: 1px solid #ddd; padding: 8px; text-align: center; }}
        .coverage-matrix th {{ background: #f2f2f2; }}
        .high-coverage {{ background: #d4edda; }}
        .medium-coverage {{ background: #fff3cd; }}
        .low-coverage {{ background: #f8d7da; }}
    </style>
</head>
<body>
    <div class="header">
        <h1>Security Frameworks Compliance Report</h1>
        <p>Generated: {report_data['generated_at']}</p>
        <p>Total Mappings: {report_data['mappings']['total']} ({report_data['mappings']['verified']} verified)</p>
    </div>
    
    <h2>Frameworks</h2>
    {''.join([f'''
        <div class="framework">
            <h3>{fw['name']} ({fw['id']})</h3>
            <p>Controls: {fw['total_controls']} | Domains: {fw['domains']}</p>
        </div>
    ''' for fw in report_data['frameworks']])}
    
    <h2>Coverage Matrix</h2>
    <table class="coverage-matrix">
        <tr>
            <th>Source / Target</th>
            {''.join([f'<th>{fw["id"]}</th>' for fw in report_data["frameworks"]])}
        </tr>
        {''.join([f'''
            <tr>
                <th>{source["id"]}</th>
                {''.join([
                    '<td>-</td>' if source["id"] == target["id"] else
                    f'<td class="{_get_coverage_class(report_data["coverage"][source["id"]][target["id"]]["percentage"])}">{report_data["coverage"][source["id"]][target["id"]]["percentage"]:.1f}%</td>'
                    for target in report_data["frameworks"]
                ])}
            </tr>
        ''' for source in report_data["frameworks"]])}
    </table>
</body>
</html>"""
    
    with open(output_path, 'w') as f:
        f.write(html_content)


def _get_coverage_class(coverage: float) -> str:
    """Get CSS class for coverage percentage."""
    if coverage > 70:
        return "high-coverage"
    elif coverage > 40:
        return "medium-coverage"
    else:
        return "low-coverage"


if __name__ == '__main__':
    report()