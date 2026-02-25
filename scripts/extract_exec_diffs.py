#!/usr/bin/env python3
# Copyright(c) The Maintainers of Nanvix.
# Licensed under the MIT License.

"""
Extract per-function diffs from an exec consistency check.

Given a source file and a Verus file, runs check_exec_consistency.py (or takes
its JSON output), then extracts the source and Verus code for each mismatched
or missing function into individual diff files for easy review.

Generates two diff views:
  - full/    : source vs full Verus code (includes spec/proof annotations)
  - exec-only/ : source vs Verus code stripped of all ghost/proof/spec annotations

Usage:
    python3 scripts/extract_exec_diffs.py <source> <verus> -o <output_dir>

Output structure:
    <output_dir>/
        summary.md                - Overview table with links
        consistency_report.md     - AST consistency report with links
        full/
            <func>.diff           - Source vs full Verus (with spec/proof)
            <func>_source.rs      - Source version
            <func>_verus.rs       - Full Verus version
        exec-only/
            <func>.diff           - Source vs Verus stripped of ghost/proof
            <func>_verus_stripped.rs - Verus version with annotations stripped
"""

import argparse
import difflib
import json
import os
import subprocess
import sys
from pathlib import Path


def extract_lines(filepath: str, line_range: str) -> str:
    """Extract lines from a file given a range like '72-93'."""
    if not line_range:
        return ""
    parts = line_range.split("-")
    start = int(parts[0])
    end = int(parts[1])
    with open(filepath, "r") as f:
        lines = f.readlines()
    return "".join(lines[start - 1:end])


def _get_strip_fn():
    """Import strip function from check_exec_consistency."""
    script_dir = Path(__file__).resolve().parent
    sys.path.insert(0, str(script_dir))
    from check_exec_consistency import strip_ghost_from_function
    return strip_ghost_from_function


def _generate_diffs_for_dir(
    out: Path, functions: list, source_path: str, verus_path: str,
    strip_fn=None,
) -> list:
    """Generate per-function diff files in the given directory.

    If strip_fn is provided, it is applied to the Verus code before diffing.
    Returns summary lines for the table.
    """
    out.mkdir(parents=True, exist_ok=True)
    summary_lines = []

    for fn in functions:
        name = fn["name"]
        fname = name.replace("::", "__")  # Safe filename.
        status = fn["status"]

        if status == "MATCH":
            continue

        files_generated = []

        if status == "MISMATCH":
            src_code = extract_lines(source_path, fn.get("src_lines", ""))
            verus_code = extract_lines(verus_path, fn.get("verus_lines", ""))

            # Optionally strip ghost/proof annotations.
            verus_display = strip_fn(verus_code) if strip_fn else verus_code
            verus_suffix = "_verus_stripped.rs" if strip_fn else "_verus.rs"

            # Write source version.
            src_file = out / f"{fname}_source.rs"
            src_file.write_text(src_code)
            files_generated.append(src_file.name)

            # Write verus version.
            verus_file = out / f"{fname}{verus_suffix}"
            verus_file.write_text(verus_display)
            files_generated.append(verus_file.name)

            # Generate unified diff.
            diff = list(difflib.unified_diff(
                src_code.splitlines(keepends=False),
                verus_display.splitlines(keepends=False),
                fromfile=f"source:{name}",
                tofile=f"verus:{name}",
                lineterm="",
            ))
            diff_text = "\n".join(diff)
            if diff_text:
                diff_file = out / f"{fname}.diff"
                diff_file.write_text(diff_text + "\n")
                files_generated.append(diff_file.name)

        elif status == "MISSING_IN_VERUS":
            src_code = extract_lines(source_path, fn.get("src_lines", ""))
            if src_code:
                src_file = out / f"{fname}_source.rs"
                src_file.write_text(src_code)
                files_generated.append(f"{src_file.name} (MISSING in verus)")

        elif status == "EXTRA_IN_VERUS":
            verus_code = extract_lines(verus_path, fn.get("verus_lines", ""))
            if verus_code:
                verus_file = out / f"{fname}_verus.rs"
                verus_file.write_text(verus_code)
                files_generated.append(f"{verus_file.name} (EXTRA)")

        summary_lines.append(
            f"| `{name}` | {status} | {', '.join(files_generated)} |"
        )

    return summary_lines


def main() -> int:
    """Main entry point."""
    parser = argparse.ArgumentParser(
        description="Extract per-function diffs from exec consistency check"
    )
    parser.add_argument("source", help="Original source file path")
    parser.add_argument("verus", help="Verus verified file path")
    parser.add_argument(
        "-o", "--output", required=True, help="Output directory for diff files"
    )
    parser.add_argument(
        "--json-input",
        help="Use pre-computed JSON from check_exec_consistency.py instead of running it",
    )
    args = parser.parse_args()

    # Get consistency report.
    if args.json_input:
        with open(args.json_input) as f:
            report = json.load(f)
    else:
        # Find the checker script relative to this script.
        script_dir = Path(__file__).resolve().parent
        checker = script_dir / "check_exec_consistency.py"
        # Use the same python that's running this script.
        venv_python = script_dir.parent / "verus-tools-venv" / "bin" / "python3"
        python = str(venv_python) if venv_python.exists() else sys.executable

        result = subprocess.run(
            [python, str(checker), args.source, args.verus, "--json"],
            capture_output=True,
            text=True,
        )
        if result.returncode not in (0, 1):
            print(f"ERROR: checker failed: {result.stderr}", file=sys.stderr)
            return 2
        report = json.loads(result.stdout)

    out = Path(args.output)
    out.mkdir(parents=True, exist_ok=True)

    source_path = args.source
    verus_path = args.verus
    functions = report.get("functions", [])

    # Generate full diffs (source vs full Verus with spec/proof).
    full_dir = out / "full"
    full_lines = _generate_diffs_for_dir(
        full_dir, functions, source_path, verus_path, strip_fn=None,
    )

    # Generate exec-only diffs (source vs Verus stripped of ghost/proof).
    try:
        strip_fn = _get_strip_fn()
    except Exception as e:
        print(f"WARNING: Could not load strip function: {e}", file=sys.stderr)
        strip_fn = None

    exec_only_lines = []
    if strip_fn:
        exec_only_dir = out / "exec-only"
        exec_only_lines = _generate_diffs_for_dir(
            exec_only_dir, functions, source_path, verus_path, strip_fn=strip_fn,
        )

    # Write summary.
    summary_lines = [
        f"# Exec Diff: {Path(source_path).stem}",
        "",
        f"**Source:** `{source_path}`",
        f"**Verus:** `{verus_path}`",
        "",
        "## Full Diffs (source vs Verus with spec/proof)",
        "",
        "Directory: `full/`",
        "",
        "| Function | Status | Files |",
        "|----------|--------|-------|",
    ]
    summary_lines.extend(full_lines)

    if exec_only_lines:
        summary_lines.extend([
            "",
            "## Exec-Only Diffs (source vs Verus stripped of ghost/proof)",
            "",
            "Directory: `exec-only/`",
            "",
            "These diffs show only the executable code differences, with all Verus",
            "annotations (requires/ensures, proof blocks, ghost variables, invariants)",
            "removed. This makes it easier to spot real exec logic changes.",
            "",
            "| Function | Status | Files |",
            "|----------|--------|-------|",
        ])
        summary_lines.extend(exec_only_lines)

    # Write struct diffs.
    structs = report.get("structs", [])
    struct_issues = [s for s in structs if s["status"] not in ("MATCH", "EXPECTED_EXTRA")]
    if struct_issues:
        summary_lines.append("")
        summary_lines.append("## Struct Issues")
        summary_lines.append("")
        summary_lines.append("| Struct | Status | Files |")
        summary_lines.append("|--------|--------|-------|")

        for s in struct_issues:
            sname = s["name"]
            sstatus = s["status"]
            sfiles = []

            if sstatus == "MISMATCH":
                src_code = extract_lines(source_path, s.get("src_lines", ""))
                verus_code = extract_lines(verus_path, s.get("verus_lines", ""))

                if src_code:
                    sf = full_dir / f"struct_{sname}_source.rs"
                    sf.write_text(src_code)
                    sfiles.append(sf.name)
                if verus_code:
                    vf = full_dir / f"struct_{sname}_verus.rs"
                    vf.write_text(verus_code)
                    sfiles.append(vf.name)
                if src_code and verus_code:
                    diff = list(difflib.unified_diff(
                        src_code.splitlines(keepends=False),
                        verus_code.splitlines(keepends=False),
                        fromfile=f"source:struct_{sname}",
                        tofile=f"verus:struct_{sname}",
                        lineterm="",
                    ))
                    diff_text = "\n".join(diff)
                    if diff_text:
                        df = full_dir / f"struct_{sname}.diff"
                        df.write_text(diff_text + "\n")
                        sfiles.append(df.name)

            elif sstatus == "MISSING_IN_VERUS":
                src_code = extract_lines(source_path, s.get("src_lines", ""))
                if src_code:
                    sf = full_dir / f"struct_{sname}_source.rs"
                    sf.write_text(src_code)
                    sfiles.append(f"{sf.name} (MISSING in verus)")

            elif sstatus == "EXTRA_IN_VERUS":
                verus_code = extract_lines(verus_path, s.get("verus_lines", ""))
                if verus_code:
                    vf = full_dir / f"struct_{sname}_verus.rs"
                    vf.write_text(verus_code)
                    sfiles.append(f"{vf.name} (EXTRA)")

            summary_lines.append(
                f"| `{sname}` | {sstatus} | {', '.join(sfiles)} |"
            )

    # Write summary.
    summary_file = out / "summary.md"
    summary_file.write_text("\n".join(summary_lines) + "\n")

    s = report["summary"]["functions"]
    print(f"Output: {out}/")
    print(f"  full/       : {s['mismatched']} diffs (source vs full Verus)")
    if strip_fn:
        print(f"  exec-only/  : {s['mismatched']} diffs (source vs stripped Verus)")
    print(f"  {s['missing_in_verus']} missing → source files")
    print(f"  {s['extra_in_verus']} extra → verus files")
    print(f"  summary.md written")

    return 0 if s["mismatched"] == 0 and s["missing_in_verus"] == 0 else 1


if __name__ == "__main__":
    sys.exit(main())
