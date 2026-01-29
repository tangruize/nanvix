# Copyright(c) The Maintainers of Nanvix.
# Licensed under the MIT License.

"""
Report generation for the Verus AI verification workflow.
"""

import json
from dataclasses import dataclass
from datetime import datetime
from pathlib import Path
from typing import Dict, List, Optional

from config import LOGS_DIR, PROJECT_ROOT, REVIEWS_DIR, VERUS_DIR, get_existing_modules
from guardrails import CheatingReport, detect_cheating_in_module, run_verus


@dataclass
class ModuleStats:
    """Statistics for a verified module."""

    name: str
    lines_of_code: int
    spec_fn_count: int
    proof_fn_count: int
    exec_fn_count: int
    ensures_count: int
    requires_count: int
    external_body_count: int
    assume_count: int
    verus_passes: bool


def count_patterns(content: str) -> Dict[str, int]:
    """Count various Verus patterns in code."""
    import re

    patterns = {
        "spec_fn": r"\bspec\s+fn\b",
        "proof_fn": r"\bproof\s+fn\b",
        "exec_fn": r"\bpub\s+fn\b|\bfn\s+\w+\s*\(",
        "ensures": r"\bensures\b",
        "requires": r"\brequires\b",
        "invariant": r"\binvariant\b",
        "external_body": r"external_body",
        "assume": r"\bassume\s*\(",
    }

    counts = {}
    for name, pattern in patterns.items():
        counts[name] = len(re.findall(pattern, content, re.IGNORECASE))

    return counts


def analyze_module(module_name: str) -> Optional[ModuleStats]:
    """Analyze a verified module."""
    file_path = VERUS_DIR / f"{module_name}.rs"

    if not file_path.exists():
        return None

    content = file_path.read_text()
    lines = len(content.split("\n"))
    counts = count_patterns(content)

    verus_passes, _ = run_verus(module_name)

    return ModuleStats(
        name=module_name,
        lines_of_code=lines,
        spec_fn_count=counts.get("spec_fn", 0),
        proof_fn_count=counts.get("proof_fn", 0),
        exec_fn_count=counts.get("exec_fn", 0),
        ensures_count=counts.get("ensures", 0),
        requires_count=counts.get("requires", 0),
        external_body_count=counts.get("external_body", 0),
        assume_count=counts.get("assume", 0),
        verus_passes=verus_passes,
    )


def generate_summary_report() -> str:
    """Generate a summary report of all verified modules."""
    lines = [
        "# Verus AI Verification Summary Report",
        f"Generated: {datetime.now().strftime('%Y-%m-%d %H:%M:%S')}",
        "",
        "## Module Statistics",
        "",
        "| Module | LoC | spec fn | proof fn | ensures | requires | external_body | assume | Passes |",
        "|--------|-----|---------|----------|---------|----------|---------------|--------|--------|",
    ]

    total_loc = 0
    total_spec = 0
    total_proof = 0
    total_ensures = 0
    total_requires = 0
    total_external = 0
    total_assume = 0

    for module_name in get_existing_modules():
        stats = analyze_module(module_name)
        if stats:
            status = "✓" if stats.verus_passes else "✗"
            lines.append(
                f"| {stats.name} | {stats.lines_of_code} | {stats.spec_fn_count} | "
                f"{stats.proof_fn_count} | {stats.ensures_count} | {stats.requires_count} | "
                f"{stats.external_body_count} | {stats.assume_count} | {status} |"
            )
            total_loc += stats.lines_of_code
            total_spec += stats.spec_fn_count
            total_proof += stats.proof_fn_count
            total_ensures += stats.ensures_count
            total_requires += stats.requires_count
            total_external += stats.external_body_count
            total_assume += stats.assume_count

    lines.append(
        f"| **Total** | **{total_loc}** | **{total_spec}** | **{total_proof}** | "
        f"**{total_ensures}** | **{total_requires}** | **{total_external}** | **{total_assume}** | |"
    )

    lines.extend(
        [
            "",
            "## Cheating Detection",
            "",
        ]
    )

    any_cheating = False
    for module_name in get_existing_modules():
        report = detect_cheating_in_module(module_name)
        if report.has_cheating():
            any_cheating = True
            lines.append(f"- **{module_name}**: {report.summary()}")

    if not any_cheating:
        lines.append("No cheating patterns detected in core modules.")

    lines.extend(
        [
            "",
            "## Review History",
            "",
        ]
    )

    review_dir = REVIEWS_DIR
    if review_dir.exists():
        review_files = sorted(review_dir.glob("*.md"))
        lines.append(f"Total review sessions: {len(review_files)}")
        lines.append("")
        lines.append("| File | Module | Model |")
        lines.append("|------|--------|-------|")
        for rf in review_files[-10:]:  # Last 10 reviews.
            parts = rf.stem.split("_")
            if len(parts) >= 2:
                module = parts[0]
                model = parts[1] if len(parts) > 1 else "?"
                lines.append(f"| {rf.name} | {module} | {model} |")

    return "\n".join(lines)


def save_report(content: str, filename: str = "VERIFICATION_REPORT.md") -> Path:
    """Save report to file."""
    report_path = PROJECT_ROOT / filename
    report_path.write_text(content)
    return report_path


if __name__ == "__main__":
    report = generate_summary_report()
    path = save_report(report)
    print(f"Report saved to {path}")
    print()
    print(report)
