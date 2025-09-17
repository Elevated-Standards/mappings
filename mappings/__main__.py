"""
Main entry point for the mappings package.
"""

import sys
from .cli.main import cli as analyze_cli
from .cli.report import report as report_cli


def main():
    """Main entry point."""
    if len(sys.argv) > 1 and sys.argv[1] == 'report':
        # Remove 'report' from argv and call report CLI
        sys.argv = [sys.argv[0]] + sys.argv[2:]
        report_cli()
    else:
        # Default to analyze CLI
        if len(sys.argv) > 1 and sys.argv[1] in ['frameworks', 'gaps', 'mappings', 'similar', 'matrix']:
            analyze_cli()
        else:
            # Show help for both CLIs
            print("Security Frameworks Mapping System")
            print("\nAvailable commands:")
            print("  python -m mappings frameworks              - List all frameworks")
            print("  python -m mappings gaps <source> <target>  - Analyze gaps")
            print("  python -m mappings mappings <fw> <ctrl>    - Find mappings")
            print("  python -m mappings similar <fw> <ctrl>     - Find similar controls")
            print("  python -m mappings matrix                  - Generate matrix")
            print("  python -m mappings report summary          - Generate summary")
            print("  python -m mappings report full             - Generate full report")


if __name__ == '__main__':
    main()