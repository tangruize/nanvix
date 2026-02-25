#!/usr/bin/env python3
# Copyright(c) The Maintainers of Nanvix.
# Licensed under the MIT License.

"""
Detect dead (uncalled) exec functions in Verus verified code.

Compares functions in the Verus exec file against the original source file.
Functions that exist only in Verus and have zero callers are flagged as dead code.

Usage:
    python3 scripts/check_dead_code.py <source> <verus_exec> [--verus-dir DIR]

Examples:
    python3 scripts/check_dead_code.py src/libs/bitmap/src/lib.rs verus/split/libs/bitmap/lib.rs
    python3 scripts/check_dead_code.py src/libs/bitmap/src/lib.rs verus/split/libs/bitmap/lib.rs --verus-dir verus/split
"""

import argparse
import os
import re
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


def extract_fn_names(filepath: str, exec_only: bool = True) -> set:
    """Extract function names from a Rust file using tree-sitter."""
    with open(filepath, "r") as f:
        content = f.read()

    tree = _parser.parse(bytes(content, "utf-8"))
    query = _language.query("(function_item) @fn")
    captures = query.captures(tree.root_node)

    names = set()
    for node, _ in captures:
        if exec_only:
            # Skip spec/proof functions.
            mods_query = _language.query("(function_modifiers) @mods")
            mod_captures = mods_query.captures(node)
            if mod_captures:
                mod_text = mod_captures[0][0].text.decode("utf-8")
                if "spec" in mod_text or "proof" in mod_text:
                    continue
        name_node = node.child_by_field_name("name")
        if name_node:
            names.add(name_node.text.decode("utf-8"))
    return names


def count_references(name: str, search_dir: str) -> int:
    """Count references to a function name across all .rs files in a directory.

    Excludes the function definition itself (fn <name>).
    """
    count = 0
    for root, dirs, files in os.walk(search_dir):
        # Skip backup directories.
        dirs[:] = [d for d in dirs if d not in ("backup", "target")]
        for fname in files:
            if not fname.endswith(".rs"):
                continue
            fpath = os.path.join(root, fname)
            with open(fpath, "r") as f:
                content = f.read()
            # Count call-site references: name( or .name( or ::name(
            # Exclude "fn name" definitions.
            calls = len(re.findall(rf"(?<!fn\s)\b{re.escape(name)}\s*[\(]", content))
            # Also count method-style: .name(
            method_calls = len(re.findall(rf"\.\s*{re.escape(name)}\s*[\(]", content))
            count += calls + method_calls
    return count


def main() -> int:
    """Main entry point."""
    parser = argparse.ArgumentParser(
        description="Detect dead (uncalled) exec functions in Verus code"
    )
    parser.add_argument("source", help="Original source file")
    parser.add_argument("verus", help="Verus exec file")
    parser.add_argument(
        "--verus-dir",
        default=None,
        help="Directory to search for callers (default: parent of verus file)",
    )
    args = parser.parse_args()

    if not HAS_TREE_SITTER:
        print("ERROR: tree-sitter not available.", file=sys.stderr)
        return 1

    source_fns = extract_fn_names(args.source, exec_only=False)
    verus_fns = extract_fn_names(args.verus, exec_only=True)

    # Functions in Verus but not in source.
    extra_fns = verus_fns - source_fns

    if not extra_fns:
        print("✅ No extra functions in Verus code.")
        return 0

    search_dir = args.verus_dir or str(Path(args.verus).parent.parent.parent)

    print(f"Checking {len(extra_fns)} extra function(s) for callers in {search_dir}/\n")
    print(f"{'Function':<40} {'Callers':>7}  Status")
    print("-" * 60)

    dead_count = 0
    for name in sorted(extra_fns):
        refs = count_references(name, search_dir)
        status = "⚠️  DEAD CODE" if refs == 0 else f"✅ ({refs} refs)"
        if refs == 0:
            dead_count += 1
        print(f"{name:<40} {refs:>7}  {status}")

    print(f"\nSummary: {dead_count} dead / {len(extra_fns)} extra functions")
    return 1 if dead_count > 0 else 0


if __name__ == "__main__":
    sys.exit(main())
