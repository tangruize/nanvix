#!/usr/bin/env python3
# Copyright(c) The Maintainers of Nanvix.
# Licensed under the MIT License.

"""
Utility functions for the Verus AI verification workflow.
"""

import re
import subprocess
from pathlib import Path
from typing import List, Optional, Tuple

from config import PROJECT_ROOT, REVIEWS_DIR, VERUS_DIR


def get_review_history(module_name: str) -> List[Tuple[str, str, str]]:
    """
    Get review history for a module.

    Returns:
        List of (filename, model, grade) tuples.
    """
    review_dir = REVIEWS_DIR
    if not review_dir.exists():
        return []

    results = []
    for review_file in sorted(review_dir.glob(f"{module_name}_*.md")):
        content = review_file.read_text()

        # Extract model from filename.
        parts = review_file.stem.split("_")
        model = parts[1] if len(parts) > 1 else "unknown"

        # Extract grade.
        grade = "?"
        grade_match = re.search(r"##?\s*Grade\s*:\s*([A-F][+-]?)", content, re.IGNORECASE)
        if grade_match:
            grade = grade_match.group(1)

        results.append((review_file.name, model, grade))

    return results


def compare_with_original(module_name: str, source_path: Optional[str] = None) -> Optional[str]:
    """
    Compare verified module with original source.

    Parameters:
        module_name: Name of the verified module.
        source_path: Optional path to original source file.

    Returns:
        Diff output or None if comparison not possible.
    """
    verified_path = VERUS_DIR / f"{module_name}.rs"
    if not verified_path.exists():
        return None

    # If no source path provided, we can only analyze the verified version.
    if source_path is None:
        verified_content = verified_path.read_text()
        verified_fns = set(re.findall(r"\bfn\s+(\w+)\s*\(", verified_content))

        lines = [
            f"# Verified Module: {module_name}",
            "",
            f"Verified functions: {len(verified_fns)}",
            "",
            "Functions:",
        ]
        for fn in sorted(verified_fns):
            lines.append(f"  - {fn}")
        return "\n".join(lines)

    original_path = PROJECT_ROOT / Path(source_path)
    verified_path = VERUS_DIR / f"{module_name}.rs"

    if not original_path.exists() or not verified_path.exists():
        return None

    # Count functions in each.
    original_content = original_path.read_text()
    verified_content = verified_path.read_text()

    original_fns = set(re.findall(r"\bfn\s+(\w+)\s*\(", original_content))
    verified_fns = set(re.findall(r"\bfn\s+(\w+)\s*\(", verified_content))

    missing = original_fns - verified_fns
    extra = verified_fns - original_fns

    lines = [
        f"# Coverage Comparison: {module_name}",
        "",
        f"Original functions: {len(original_fns)}",
        f"Verified functions: {len(verified_fns)}",
        "",
    ]

    if missing:
        lines.append("## Missing in verified version:")
        for fn in sorted(missing):
            lines.append(f"  - {fn}")
        lines.append("")

    if extra:
        lines.append("## Added in verified version (specs, proofs, helpers):")
        for fn in sorted(extra):
            lines.append(f"  - {fn}")
        lines.append("")

    return "\n".join(lines)


def extract_properties(module_name: str) -> List[str]:
    """
    Extract key properties from a verified module.

    Returns:
        List of property descriptions.
    """
    file_path = VERUS_DIR / f"{module_name}.rs"
    if not file_path.exists():
        return []

    content = file_path.read_text()
    properties = []

    # Extract ensures clauses with meaningful names.
    ensures_pattern = r"ensures\s*\{([^}]+)\}"
    for match in re.finditer(ensures_pattern, content, re.DOTALL):
        clause = match.group(1).strip()
        # Simplify multi-line clauses.
        clause = " ".join(clause.split())
        if len(clause) < 200:  # Skip overly complex clauses.
            properties.append(clause)

    # Extract invariant definitions.
    inv_pattern = r"spec\s+fn\s+(inv\w*|invariant\w*)\s*\("
    for match in re.finditer(inv_pattern, content):
        properties.append(f"Invariant: {match.group(1)}")

    return properties[:20]  # Limit to top 20.


def validate_module(module_name: str) -> dict:
    """
    Validate a verified module comprehensively.

    Returns:
        Dictionary with validation results.
    """
    from guardrails import detect_cheating_in_module, run_verus

    results = {
        "module": module_name,
        "exists": False,
        "verus_passes": False,
        "has_cheating": False,
        "cheating_summary": "",
        "loc": 0,
        "properties": 0,
    }

    file_path = VERUS_DIR / f"{module_name}.rs"
    if not file_path.exists():
        return results

    results["exists"] = True
    results["loc"] = len(file_path.read_text().split("\n"))

    # Run Verus.
    success, output = run_verus(module_name)
    results["verus_passes"] = success

    # Extract property count from output.
    prop_match = re.search(r"(\d+)\s+verified", output)
    if prop_match:
        results["properties"] = int(prop_match.group(1))

    # Check cheating.
    cheating = detect_cheating_in_module(module_name)
    results["has_cheating"] = cheating.has_cheating()
    results["cheating_summary"] = cheating.summary()

    return results


if __name__ == "__main__":
    import sys

    if len(sys.argv) < 2:
        print("Usage: python utils.py <command> [args]")
        print("Commands: history <module>, compare <module>, props <module>, validate <module>")
        sys.exit(1)

    cmd = sys.argv[1]

    if cmd == "history" and len(sys.argv) > 2:
        history = get_review_history(sys.argv[2])
        print(f"Review history for {sys.argv[2]}:")
        for filename, model, grade in history:
            print(f"  {filename}: {model} -> {grade}")

    elif cmd == "compare" and len(sys.argv) > 2:
        result = compare_with_original(sys.argv[2])
        print(result or "Comparison not available")

    elif cmd == "props" and len(sys.argv) > 2:
        props = extract_properties(sys.argv[2])
        print(f"Properties in {sys.argv[2]}:")
        for prop in props:
            print(f"  - {prop}")

    elif cmd == "validate" and len(sys.argv) > 2:
        result = validate_module(sys.argv[2])
        for k, v in result.items():
            print(f"  {k}: {v}")

    else:
        print("Unknown command or missing arguments")
        sys.exit(1)
