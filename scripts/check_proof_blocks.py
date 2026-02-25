#!/usr/bin/env python3
# Copyright(c) The Maintainers of Nanvix.
# Licensed under the MIT License.

"""
Audit proof block sizes in Verus source files using tree-sitter.

Finds all `proof { ... }` blocks and reports those exceeding a threshold.
Proof blocks over the threshold should be extracted into lemmas in .proof.rs
for readability.

Usage:
    python3 scripts/check_proof_blocks.py <file_or_dir> [--threshold N] [--all]

Examples:
    python3 scripts/check_proof_blocks.py verus/split/libs/bitmap/lib.rs
    python3 scripts/check_proof_blocks.py verus/split/ --threshold 10
    python3 scripts/check_proof_blocks.py verus/split/libs/bitmap/lib.rs --all
"""

import argparse
import os
import sys
from pathlib import Path

# Tree-sitter setup.
LANGUAGE_SO = os.environ.get(
    "VERUS_LANGUAGE_SO",
    os.path.join(
        os.path.dirname(os.path.dirname(os.path.abspath(__file__))),
        "verus-tools-venv",
        "verus.so",
    ),
)

try:
    from tree_sitter import Language, Parser

    _language = Language(LANGUAGE_SO, "rust")
    _parser = Parser()
    _parser.set_language(_language)
    HAS_TREE_SITTER = True
except Exception:
    HAS_TREE_SITTER = False


def find_parent_fn(node) -> str:
    """Walk up the tree to find the enclosing function name."""
    p = node.parent
    while p:
        if p.type == "function_item":
            name_node = p.child_by_field_name("name")
            if name_node:
                return name_node.text.decode("utf-8")
        p = p.parent
    return "?"


def audit_file(filepath: str, threshold: int, show_all: bool) -> list:
    """Audit proof blocks in a single file.

    Returns a list of (file, fn_name, start, end, lines) for blocks over threshold.
    """
    with open(filepath, "r") as f:
        content = f.read()

    tree = _parser.parse(bytes(content, "utf-8"))
    query = _language.query("(proof_block) @pb")
    captures = query.captures(tree.root_node)

    results = []
    for node, _ in captures:
        lines = node.end_point[0] - node.start_point[0] + 1
        fn_name = find_parent_fn(node)
        start = node.start_point[0] + 1
        end = node.end_point[0] + 1

        if show_all or lines > threshold:
            results.append((filepath, fn_name, start, end, lines))

    return results


def main() -> int:
    """Main entry point."""
    parser = argparse.ArgumentParser(
        description="Audit proof block sizes in Verus source files"
    )
    parser.add_argument(
        "path",
        help="Verus source file or directory to audit",
    )
    parser.add_argument(
        "--threshold", "-t",
        type=int,
        default=5,
        help="Line threshold for extraction (default: 5)",
    )
    parser.add_argument(
        "--all", "-a",
        action="store_true",
        help="Show all proof blocks, not just those over threshold",
    )
    args = parser.parse_args()

    if not HAS_TREE_SITTER:
        print(
            "ERROR: tree-sitter not available. Install: pip install tree_sitter==0.21.3",
            file=sys.stderr,
        )
        return 1

    path = Path(args.path)
    if not path.exists():
        print(f"ERROR: Path not found: {path}", file=sys.stderr)
        return 1

    # Collect files to audit.
    if path.is_file():
        files = [str(path)]
    else:
        files = sorted(
            str(f) for f in path.rglob("*.rs")
            if not f.name.endswith(".spec.rs")
            and not f.name.endswith(".proof.rs")
            and not f.name.endswith(".test.rs")
            and "backup" not in str(f)
            and "target" not in str(f)
        )

    all_results = []
    for filepath in files:
        results = audit_file(filepath, args.threshold, args.all)
        all_results.extend(results)

    if not all_results:
        if args.all:
            print("No proof blocks found.")
        else:
            print(f"✅ All proof blocks are ≤{args.threshold} lines.")
        return 0

    # Print results.
    over_threshold = [r for r in all_results if r[4] > args.threshold]

    if args.all:
        print(f"{'File':<45} {'Function':<25} {'Start':>5} {'End':>5} {'Lines':>5}  Status")
        print("-" * 100)
        for filepath, fn_name, start, end, lines in all_results:
            short_path = str(Path(filepath).relative_to(Path.cwd())) if Path(filepath).is_relative_to(Path.cwd()) else filepath
            status = f"⚠️  EXTRACT (>{args.threshold})" if lines > args.threshold else "✅"
            print(f"{short_path:<45} {fn_name:<25} {start:>5} {end:>5} {lines:>5}  {status}")
    else:
        print(f"⚠️  Found {len(over_threshold)} proof block(s) over {args.threshold} lines:\n")
        print(f"{'File':<45} {'Function':<25} {'Start':>5} {'End':>5} {'Lines':>5}")
        print("-" * 90)
        for filepath, fn_name, start, end, lines in over_threshold:
            short_path = str(Path(filepath).relative_to(Path.cwd())) if Path(filepath).is_relative_to(Path.cwd()) else filepath
            print(f"{short_path:<45} {fn_name:<25} {start:>5} {end:>5} {lines:>5}")

    # Summary.
    total = len(all_results)
    over = len(over_threshold)
    print(f"\nSummary: {total} proof blocks total, {over} over {args.threshold} lines"
          f"{', ' + str(total - over) + ' OK' if over > 0 else ''}")

    return 1 if over > 0 else 0


if __name__ == "__main__":
    sys.exit(main())
