#!/usr/bin/env python3
# Copyright(c) The Maintainers of Nanvix.
# Licensed under the MIT License.

"""
Format checker for Verus-annotated Rust files.

Uses tree-sitter with the Verus parser to strip proof/spec/ghost annotations
from verus! blocks, then runs rustfmt --check on the resulting exec-only code.

Usage:
    python3 scripts/verus_fmt_check.py <file.rs> [--fix]
    python3 scripts/verus_fmt_check.py src/libs/bitmap/src/lib.rs
"""

import argparse
import os
import re
import subprocess
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
    import warnings
    warnings.filterwarnings("ignore", category=FutureWarning, module="tree_sitter")
    from tree_sitter import Language, Parser

    _language = Language(LANGUAGE_SO, "rust")
    _parser = Parser()
    _parser.set_language(_language)
    HAS_TREE_SITTER = True
except Exception as e:
    HAS_TREE_SITTER = False
    _import_error = str(e)

# Verus-specific AST node types to remove.
VERUS_NODES_TO_REMOVE = {
    "proof_block",
    "function_specifications",
    "loop_specifications",
}


def strip_verus_annotations(content_bytes: bytes) -> str:
    """Strip Verus-specific annotations using tree-sitter AST, preserving exec code."""
    tree = _parser.parse(content_bytes)
    remove_ranges: list[tuple[int, int]] = []

    # Replace ranges: (start_byte, end_byte, replacement_str).
    replace_ranges: list[tuple[int, int, str]] = []

    def collect_remove_ranges(node) -> None:
        # Remove proof blocks, function specs, loop specs.
        if node.type in VERUS_NODES_TO_REMOVE:
            remove_ranges.append((node.start_byte, node.end_byte))
            return
        # Remove spec/proof functions (they have spec/proof in function_modifiers).
        if node.type == "function_item":
            for child in node.children:
                if child.type == "function_modifiers":
                    mod_text: str = content_bytes[child.start_byte:child.end_byte].decode(errors="replace")
                    if "spec" in mod_text or "proof" in mod_text:
                        remove_ranges.append((node.start_byte, node.end_byte))
                        return
        # Remove ghost variable declarations.
        if node.type == "let_declaration":
            for child in node.children:
                if child.type == "ghost":
                    remove_ranges.append((node.start_byte, node.end_byte))
                    return
        # Handle verus! blocks.
        if node.type == "verus_block":
            # Check if preceded by #[cfg(verus_keep_ghost)] — skip entirely.
            parent = node.parent
            if parent and parent.parent:
                pp_children = list(parent.parent.children)
                idx: int = -1
                for i, sib in enumerate(pp_children):
                    if sib.start_byte == parent.start_byte and sib.end_byte == parent.end_byte:
                        idx = i
                        break
                if idx > 0:
                    prev = pp_children[idx - 1]
                    prev_text: str = content_bytes[prev.start_byte:prev.end_byte].decode(errors="replace")
                    if "verus_keep_ghost" in prev_text:
                        # Gated verus block: remove entirely (including the cfg attribute).
                        remove_ranges.append((prev.start_byte, parent.end_byte))
                        return
            # Non-gated verus block: strip wrapper, keep exec content.
            for child in node.children:
                if child.type in ("verus", "!"):
                    remove_ranges.append((child.start_byte, child.end_byte))
                if child.type == "block":
                    for bc in child.children:
                        if bc.type in ("{", "}"):
                            remove_ranges.append((bc.start_byte, bc.end_byte))
        # Fix Verus return type: -> (result: Type) => -> Type.
        if node.type == "function_item":
            children = list(node.children)
            for i, child in enumerate(children):
                if child.type == "->" and i + 1 < len(children) and children[i + 1].type == "(":
                    # Check if next-next is "result" identifier.
                    paren_open = children[i + 1]
                    if i + 2 < len(children) and children[i + 2].type == "identifier":
                        id_text: str = content_bytes[
                            children[i + 2].start_byte:children[i + 2].end_byte
                        ].decode()
                        if id_text == "result":
                            # Remove "(", "result", ":".
                            remove_ranges.append((paren_open.start_byte, paren_open.end_byte))
                            remove_ranges.append((children[i + 2].start_byte, children[i + 2].end_byte))
                            if i + 3 < len(children) and children[i + 3].type == ":":
                                remove_ranges.append((children[i + 3].start_byte, children[i + 3].end_byte))
                            # Find and remove closing ")".
                            for j in range(i + 4, len(children)):
                                if children[j].type == ")":
                                    remove_ranges.append((children[j].start_byte, children[j].end_byte))
                                    break
                            break
        for child in node.children:
            collect_remove_ranges(child)

    collect_remove_ranges(tree.root_node)
    remove_ranges.sort()

    # Extend each removal range to consume leading whitespace and preceding newline.
    extended_ranges: list[tuple[int, int]] = []
    for start, end in remove_ranges:
        # Walk back from start to consume leading whitespace on the same line.
        ext_start: int = start
        while ext_start > 0 and content_bytes[ext_start - 1:ext_start] in (b" ", b"\t"):
            ext_start -= 1
        # If we reached a newline, also consume it (removing the whole line).
        if ext_start > 0 and content_bytes[ext_start - 1:ext_start] == b"\n":
            ext_start -= 1
        extended_ranges.append((ext_start, end))

    # Build stripped content by removing extended ranges.
    result: list[bytes] = []
    pos: int = 0
    for start, end in extended_ranges:
        if start > pos:
            result.append(content_bytes[pos:start])
        if end > pos:
            pos = end
    result.append(content_bytes[pos:])
    stripped: str = b"".join(result).decode()

    # Clean up: collapse 3+ consecutive blank lines to 1.
    stripped = re.sub(r"\n[ \t]*\n[ \t]*\n", "\n\n", stripped)

    # Fix brace placement: when function_specifications/loop_specifications are removed,
    # the opening "{" ends up on a separate line. Move it to the previous line.
    stripped = re.sub(r"(\S)\s*\n\s*\{$", r"\1 {", stripped, flags=re.MULTILINE)

    # Remove "// verus!" comment left after stripping verus block wrapper.
    stripped = re.sub(r"^\s*// verus!\s*$", "", stripped, flags=re.MULTILINE)

    # Remove blank line immediately after opening brace (left by ghost variable removal).
    stripped = re.sub(r"\{\n\n", "{\n", stripped)

    # Collapse again after all post-processing.
    stripped = re.sub(r"\n{3,}", "\n\n", stripped)

    return stripped


def run_rustfmt_check(filepath: str, stripped: str, fix: bool = False) -> int:
    """Run rustfmt on stripped content, return exit code."""
    # Write temp file in same directory for mod resolution.
    src_dir: str = os.path.dirname(os.path.abspath(filepath))
    tmppath: str = os.path.join(src_dir, "_verus_fmt_check.rs")

    # Find rustfmt.toml.
    repo_root: str = os.path.dirname(os.path.dirname(os.path.dirname(src_dir)))
    config_path: str = os.path.join(repo_root, "rustfmt.toml")
    if not os.path.exists(config_path):
        # Walk up to find it.
        d: str = src_dir
        while d != "/":
            candidate: str = os.path.join(d, "rustfmt.toml")
            if os.path.exists(candidate):
                config_path = candidate
                break
            d = os.path.dirname(d)

    try:
        with open(tmppath, "w") as f:
            f.write(stripped)

        cmd: list[str] = ["rustfmt", "--edition", "2021"]
        if os.path.exists(config_path):
            cmd.extend(["--config-path", config_path])
        if not fix:
            cmd.append("--check")
        cmd.append(tmppath)

        result = subprocess.run(cmd, capture_output=True, text=True)

        if result.returncode != 0 and not fix:
            out: str = result.stdout or result.stderr
            lines: list[str] = out.strip().split("\n")
            if any("error:" in line for line in lines[:5]):
                print(f"❌ Parse error in stripped code for {filepath}:")
                for line in lines[:15]:
                    print(f"  {line}")
            else:
                diff_count: int = sum(1 for line in lines if line.startswith("Diff"))
                print(f"⚠️  {diff_count} format issue(s) in {filepath}:")
                for line in lines:
                    print(f"  {line}")

        return result.returncode
    finally:
        if os.path.exists(tmppath):
            os.unlink(tmppath)


def main() -> None:
    """Entry point."""
    ap = argparse.ArgumentParser(
        description="Check rustfmt compliance for Verus-annotated Rust files"
    )
    ap.add_argument("files", nargs="+", help="Rust source files to check")
    ap.add_argument(
        "--fix", action="store_true", help="Apply rustfmt fixes (not yet supported)"
    )
    args = ap.parse_args()

    if not HAS_TREE_SITTER:
        print(f"❌ tree-sitter not available: {_import_error}", file=sys.stderr)
        sys.exit(1)

    all_ok: bool = True
    for filepath in args.files:
        if not os.path.exists(filepath):
            print(f"❌ File not found: {filepath}", file=sys.stderr)
            all_ok = False
            continue

        with open(filepath, "rb") as f:
            content_bytes: bytes = f.read()

        stripped: str = strip_verus_annotations(content_bytes)
        rc: int = run_rustfmt_check(filepath, stripped, fix=args.fix)

        if rc == 0:
            print(f"✅ {filepath}")
        else:
            all_ok = False

    sys.exit(0 if all_ok else 1)


if __name__ == "__main__":
    main()
