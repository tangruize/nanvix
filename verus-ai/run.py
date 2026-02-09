#!/usr/bin/env python3
# Copyright(c) The Maintainers of Nanvix.
# Licensed under the MIT License.

"""
Quick runner for the Verus AI workflow.

Usage:
    ./run.py verify <module>     # Verify a module
    ./run.py status              # Show status
    ./run.py check [module]      # Check for cheating
    ./run.py report              # Generate report
    ./run.py demo                # Run demo (dry run)
"""

import sys
from pathlib import Path

# Add parent directory to path for imports.
sys.path.insert(0, str(Path(__file__).parent))

from workflow import main as workflow_main
from report import generate_summary_report, save_report


def demo_mode() -> None:
    """Run a demonstration of the workflow (dry run)."""
    print("=" * 60)
    print("VERUS AI VERIFICATION WORKFLOW - DEMO MODE")
    print("=" * 60)
    print()
    print("This workflow automates formal verification using AI agents:")
    print()
    print("1. PROVER (claude-opus-4.6)")
    print("   - Generates Verus specifications and proofs")
    print("   - Iterates until verification passes")
    print()
    print("2. REVIEWERS (3 models)")
    print("   - claude-opus-4.6: Primary review")
    print("   - gpt-5.2-codex: Secondary review")
    print("   - gemini-3-pro-preview: Tertiary review")
    print()
    print("3. ITERATION")
    print("   - Prover addresses reviewer issues")
    print("   - Continues until all reviewers assign A+")
    print()
    print("4. GUARDRAILS")
    print("   - No assume statements allowed")
    print("   - external_body must be justified")
    print("   - Git commits track all changes")
    print()
    print("Usage:")
    print("  ./run.py verify <source_path>              # Verify a new module")
    print("  ./run.py verify <source_path> --name foo   # Specify module name")
    print("  ./run.py verify <module> --resume          # Resume verification")
    print("  ./run.py status                            # Show verified modules")
    print("  ./run.py check [module]                    # Check for cheating")
    print("  ./run.py report                            # Generate report")
    print()
    print("Already verified modules (in verus/):")
    from config import get_existing_modules
    for name in get_existing_modules():
        print(f"  - {name}")


def main() -> int:
    """Main entry point."""
    if len(sys.argv) < 2:
        print(__doc__)
        return 1

    cmd = sys.argv[1]

    if cmd == "demo":
        demo_mode()
        return 0
    elif cmd == "report":
        report = generate_summary_report()
        path = save_report(report)
        print(f"Report saved to {path}")
        return 0
    else:
        # Delegate to workflow.
        return workflow_main()


if __name__ == "__main__":
    sys.exit(main())
