#!/usr/bin/env python3
# Copyright(c) The Maintainers of Nanvix.
# Licensed under the MIT License.

"""
Verus Exec Code Diff Tool

Compares executable code between original Nanvix source files and their
Verus-verified counterparts using tree-sitter-verus for AST-level analysis.

Uses Tianyu's tree-sitter-verus parser to:
1. Extract exec functions (non-spec, non-proof) from both source and verus files.
2. Compare struct definitions, function signatures, and function bodies.
3. Categorize changes: ghost insertion, logic change, refactor, type change.
4. Generate a diff report for AI review.

Usage:
    python3 scripts/verus_exec_diff.py [--source-dir SRC] [--verus-dir VERUS] [--output REPORT]
    python3 scripts/verus_exec_diff.py --ai-review  # Also generate AI review prompt
"""

import os
import sys
import json
import argparse
import hashlib
import difflib
import re
from dataclasses import dataclass, field
from typing import Optional

# Add tree-sitter-verus to path.
SCRIPT_DIR: str = os.path.dirname(os.path.abspath(__file__))
REPO_ROOT: str = os.path.dirname(SCRIPT_DIR)
sys.path.insert(0, os.path.join(REPO_ROOT, "tree-sitter-verus"))

import tree_sitter
from tree_sitter import Language, Parser

# ──────────────────────────────────────────────────────────────────────────────
# Configuration: source ↔ verus file mappings for PM modules.
# ──────────────────────────────────────────────────────────────────────────────

PM_MAPPINGS: list[dict[str, str]] = [
    {
        "name": "capability",
        "source": "src/kernel/src/pm/process/capability.rs",
        "verus": "verus/split/kernel/pm/process/capability.rs",
    },
    {
        "name": "clock",
        "source": "src/kernel/src/pm/clock.rs",
        "verus": "verus/split/kernel/pm/clock.rs",
    },
    {
        "name": "process_manager",
        "source": "src/kernel/src/pm/process/manager/mod.rs",
        "verus": "verus/split/kernel/pm/process/manager/process_manager.rs",
    },
    {
        "name": "thread_manager",
        "source": "src/kernel/src/pm/thread/mod.rs",
        "verus": "verus/split/kernel/pm/thread/thread_manager.rs",
    },
    {
        "name": "mutex",
        "source": "src/kernel/src/pm/sync/mutex.rs"
        if os.path.exists(os.path.join(REPO_ROOT, "src/kernel/src/pm/sync/mutex.rs"))
        else "src/libs/sys/src/pm/sync/mutex.rs",
        "verus": "verus/split/kernel/pm/sync/mutex.rs",
    },
    {
        "name": "semaphore",
        "source": "src/kernel/src/pm/sync/semaphore.rs"
        if os.path.exists(os.path.join(REPO_ROOT, "src/kernel/src/pm/sync/semaphore.rs"))
        else "src/libs/sys/src/pm/sync/semaphore.rs",
        "verus": "verus/split/kernel/pm/sync/semaphore.rs",
    },
    {
        "name": "spinlock",
        "source": "src/kernel/src/pm/sync/spinlock.rs"
        if os.path.exists(os.path.join(REPO_ROOT, "src/kernel/src/pm/sync/spinlock.rs"))
        else "src/libs/sys/src/pm/sync/spinlock.rs",
        "verus": "verus/split/kernel/pm/sync/spinlock.rs",
    },
    {
        "name": "condvar",
        "source": "src/kernel/src/pm/sync/condvar.rs"
        if os.path.exists(os.path.join(REPO_ROOT, "src/kernel/src/pm/sync/condvar.rs"))
        else "src/libs/sys/src/pm/sync/condvar.rs",
        "verus": "verus/split/kernel/pm/sync/condvar.rs",
    },
    {
        "name": "fence",
        "source": "src/kernel/src/pm/sync/fence.rs"
        if os.path.exists(os.path.join(REPO_ROOT, "src/kernel/src/pm/sync/fence.rs"))
        else "src/libs/sys/src/pm/sync/fence.rs",
        "verus": "verus/split/kernel/pm/sync/fence.rs",
    },
]


# ──────────────────────────────────────────────────────────────────────────────
# Tree-sitter Verus Parser (adapted from Tianyu's verus_parser_example.py).
# ──────────────────────────────────────────────────────────────────────────────


def node_to_text(node: tree_sitter.Node) -> str:
    """Extract text from a tree-sitter node."""
    return node.text.decode()


class VerusExecParser:
    """Parser for extracting exec-level code from Verus and plain Rust files."""

    def __init__(self, language_path: str) -> None:
        self.language: Language = Language(language_path, "rust")
        self.parser: Parser = Parser()
        self.parser.set_language(self.language)

    def parse(self, source: str) -> tree_sitter.Tree:
        """Parse source code into a tree-sitter tree."""
        return self.parser.parse(bytes(source, "utf8"))

    def extract_function_modifiers(self, function: tree_sitter.Node) -> list[str]:
        """Extract function modifiers (pub, spec, proof, etc.)."""
        query = self.language.query("(function_modifiers) @function_modifiers")
        modifiers = [node for node, name in query.captures(function)]
        if len(modifiers) == 0:
            return []
        return [node_to_text(m) for m in modifiers[0].children]

    def extract_functions(self, root: tree_sitter.Node) -> list[tree_sitter.Node]:
        """Extract all function_item nodes."""
        query = self.language.query("(function_item) @fn")
        return [node for node, _ in query.captures(root)]

    def extract_exec_functions(self, root: tree_sitter.Node) -> list[tree_sitter.Node]:
        """Extract only exec functions (not spec, not proof)."""
        fns: list[tree_sitter.Node] = self.extract_functions(root)
        result: list[tree_sitter.Node] = []
        for fn in fns:
            mods: list[str] = self.extract_function_modifiers(fn)
            if "spec" not in mods and "proof" not in mods:
                result.append(fn)
        return result

    def extract_struct_items(self, root: tree_sitter.Node) -> list[tree_sitter.Node]:
        """Extract all struct definitions."""
        query = self.language.query("(struct_item) @s")
        return [node for node, _ in query.captures(root)]

    def extract_enum_items(self, root: tree_sitter.Node) -> list[tree_sitter.Node]:
        """Extract all enum definitions."""
        query = self.language.query("(enum_item) @e")
        return [node for node, _ in query.captures(root)]

    def extract_impl_items(self, root: tree_sitter.Node) -> list[tree_sitter.Node]:
        """Extract all impl blocks."""
        query = self.language.query("(impl_item) @i")
        return [node for node, _ in query.captures(root)]

    def get_fn_name(self, fn_node: tree_sitter.Node) -> str:
        """Get function name from a function_item node."""
        name_node = fn_node.child_by_field_name("name")
        if name_node is not None:
            return node_to_text(name_node)
        return "<unknown>"

    def get_fn_params(self, fn_node: tree_sitter.Node) -> str:
        """Get function parameters text."""
        params_node = fn_node.child_by_field_name("parameters")
        if params_node is not None:
            return node_to_text(params_node)
        return "()"

    def get_fn_return_type(self, fn_node: tree_sitter.Node) -> Optional[str]:
        """Get function return type text."""
        ret_node = fn_node.child_by_field_name("return_type")
        if ret_node is not None:
            return node_to_text(ret_node)
        return None

    def get_fn_body(self, fn_node: tree_sitter.Node) -> Optional[str]:
        """Get function body text."""
        body_node = fn_node.child_by_field_name("body")
        if body_node is not None:
            return node_to_text(body_node)
        return None

    def get_struct_name(self, struct_node: tree_sitter.Node) -> str:
        """Get struct name."""
        name_node = struct_node.child_by_field_name("name")
        if name_node is not None:
            return node_to_text(name_node)
        return "<unknown>"

    def get_struct_fields(self, struct_node: tree_sitter.Node) -> list[str]:
        """Get struct field declarations as text lines."""
        body = struct_node.child_by_field_name("body")
        if body is None:
            return []
        fields: list[str] = []
        for child in body.children:
            if child.type == "field_declaration":
                fields.append(node_to_text(child).strip())
        return fields

    def strip_verus_annotations(self, body_text: str) -> str:
        """Strip Verus-specific annotations from a function body for comparison.

        Removes: proof blocks, ghost variables, assert/assume, requires/ensures,
        invariant/decreases, reveal calls.
        """
        # Remove proof { ... } blocks (greedy but approximate).
        result: str = body_text
        # Remove `let ghost ...` lines.
        result = re.sub(r"let\s+ghost\s+[^;]+;", "", result)
        # Remove `proof { ... }` blocks (simple nesting).
        result = re.sub(r"proof\s*\{[^}]*\}", "", result, flags=re.DOTALL)
        # Remove assert/assume statements.
        result = re.sub(r"assert\s*\([^)]*\)\s*;", "", result)
        result = re.sub(r"assert\s*\([^)]*\)\s*by\s*\([^)]*\)\s*;", "", result)
        result = re.sub(r"assume\s*\([^)]*\)\s*;", "", result)
        # Remove reveal calls.
        result = re.sub(r"reveal\s*\([^)]*\)\s*;", "", result)
        result = re.sub(r"reveal_with_fuel\s*\([^)]*\)\s*;", "", result)
        # Collapse whitespace.
        result = re.sub(r"\n\s*\n", "\n", result)
        return result.strip()

    def get_tree_hash(self, node: tree_sitter.Node) -> str:
        """Compute semantic hash of a node, ignoring verification artifacts.

        Adapted from Tianyu's verus_parser_example.py.
        """
        try:
            children_hash_list: list[str] = [
                self.get_tree_hash(child)
                for child in filter(
                    lambda x: len(node_to_text(x)) > 1, node.children
                )
            ]
            children_hash_list = [h for h in children_hash_list if len(h) > 0]
            children_hash: str = " ".join(children_hash_list).strip()

            leaf_node: bool = len(node.children) == 0
            if leaf_node:
                node_representation: str = node_to_text(node)
            else:
                node_representation = node.type + children_hash
            node_hash: str = hashlib.md5(
                node_representation.encode()
            ).hexdigest()

            # Skip verification-only constructs (from Tianyu's implementation).
            if node.type == "use_declaration":
                node_hash = ""
            if node.type in ("attribute_item", "inner_attribute_item"):
                node_hash = ""
            if node.type == "let_declaration":
                if sum(
                    node_to_text(child) == "ghost"
                    for child in node.children
                ) > 0:
                    node_hash = ""
            if node.type == "call_expression":
                if node_to_text(node).strip().startswith("reveal"):
                    node_hash = ""
            if not leaf_node and children_hash == "":
                node_hash = ""
        except RecursionError:
            node_hash = ""
        return node_hash


# ──────────────────────────────────────────────────────────────────────────────
# Diff Analysis Data Structures.
# ──────────────────────────────────────────────────────────────────────────────


@dataclass
class StructDiff:
    """Difference found in a struct definition."""

    name: str
    change_type: str  # "added_field", "removed_field", "modified_field",
                      # "added_struct", "removed_struct", "ghost_field_added"
    detail: str
    severity: str  # "critical", "high", "medium", "low"


@dataclass
class FunctionDiff:
    """Difference found in a function."""

    name: str
    change_type: str  # "missing_in_verus", "added_in_verus", "signature_changed",
                      # "body_changed", "semantics_preserved", "type_changed"
    detail: str
    severity: str
    source_text: str = ""
    verus_text: str = ""


@dataclass
class ModuleDiffReport:
    """Complete diff report for one module."""

    module_name: str
    source_path: str
    verus_path: str
    struct_diffs: list[StructDiff] = field(default_factory=list)
    function_diffs: list[FunctionDiff] = field(default_factory=list)
    tree_hash_match: bool = False
    summary: str = ""


# ──────────────────────────────────────────────────────────────────────────────
# Diff Engine.
# ──────────────────────────────────────────────────────────────────────────────


class VerusExecDiffEngine:
    """Compare source and verus exec code at AST level."""

    def __init__(self, language_path: str) -> None:
        self.parser: VerusExecParser = VerusExecParser(language_path)

    def compare_module(
        self, source_path: str, verus_path: str, module_name: str
    ) -> ModuleDiffReport:
        """Compare a source module with its verus-verified counterpart."""
        report: ModuleDiffReport = ModuleDiffReport(
            module_name=module_name,
            source_path=source_path,
            verus_path=verus_path,
        )

        # Check file existence.
        abs_source: str = os.path.join(REPO_ROOT, source_path)
        abs_verus: str = os.path.join(REPO_ROOT, verus_path)

        if not os.path.exists(abs_source):
            report.summary = f"Source file not found: {source_path}"
            return report
        if not os.path.exists(abs_verus):
            report.summary = f"Verus file not found: {verus_path}"
            return report

        with open(abs_source, "r") as f:
            source_code: str = f.read()
        with open(abs_verus, "r") as f:
            verus_code: str = f.read()

        source_tree: tree_sitter.Tree = self.parser.parse(source_code)
        verus_tree: tree_sitter.Tree = self.parser.parse(verus_code)

        # Tree hash comparison (Tianyu's method).
        source_hash: str = self.parser.get_tree_hash(source_tree.root_node)
        verus_hash: str = self.parser.get_tree_hash(verus_tree.root_node)
        report.tree_hash_match = source_hash == verus_hash

        # Struct comparison.
        self._compare_structs(
            source_tree.root_node, verus_tree.root_node, report
        )

        # Function comparison.
        self._compare_functions(
            source_tree.root_node, verus_tree.root_node, report, source_code, verus_code
        )

        # Generate summary.
        critical: int = sum(
            1
            for d in report.struct_diffs + report.function_diffs
            if d.severity == "critical"
        )
        high: int = sum(
            1
            for d in report.struct_diffs + report.function_diffs
            if d.severity == "high"
        )
        medium: int = sum(
            1
            for d in report.struct_diffs + report.function_diffs
            if d.severity == "medium"
        )
        report.summary = (
            f"Tree hash {'MATCH' if report.tree_hash_match else 'MISMATCH'}. "
            f"Issues: {critical} critical, {high} high, {medium} medium. "
            f"Structs: {len(report.struct_diffs)} diffs. "
            f"Functions: {len(report.function_diffs)} diffs."
        )

        return report

    def _compare_structs(
        self,
        source_root: tree_sitter.Node,
        verus_root: tree_sitter.Node,
        report: ModuleDiffReport,
    ) -> None:
        """Compare struct definitions between source and verus."""
        source_structs: dict[str, tree_sitter.Node] = {
            self.parser.get_struct_name(s): s
            for s in self.parser.extract_struct_items(source_root)
        }
        verus_structs: dict[str, tree_sitter.Node] = {
            self.parser.get_struct_name(s): s
            for s in self.parser.extract_struct_items(verus_root)
        }

        # Check for added structs in verus (could be View types — OK).
        for name in verus_structs:
            if name not in source_structs:
                # View types are expected additions.
                if name.endswith("View") or name.endswith("Model"):
                    report.struct_diffs.append(
                        StructDiff(
                            name=name,
                            change_type="added_struct_view",
                            detail=f"View/Model type added for verification (expected).",
                            severity="low",
                        )
                    )
                else:
                    report.struct_diffs.append(
                        StructDiff(
                            name=name,
                            change_type="added_struct",
                            detail=f"New struct added in verus that doesn't exist in source.",
                            severity="high",
                        )
                    )

        # Check for missing structs.
        for name in source_structs:
            if name not in verus_structs:
                report.struct_diffs.append(
                    StructDiff(
                        name=name,
                        change_type="removed_struct",
                        detail=f"Struct exists in source but missing in verus.",
                        severity="critical",
                    )
                )

        # Compare fields of matching structs.
        for name in source_structs:
            if name not in verus_structs:
                continue
            source_fields: list[str] = self.parser.get_struct_fields(
                source_structs[name]
            )
            verus_fields: list[str] = self.parser.get_struct_fields(
                verus_structs[name]
            )

            # Detect ghost fields added in verus.
            for vf in verus_fields:
                is_ghost: bool = "ghost" in vf.lower() or "Ghost<" in vf
                # Check if this field exists in source.
                field_name_match: Optional[str] = None
                for sf in source_fields:
                    # Extract field name (before the colon).
                    sf_name: str = sf.split(":")[0].strip().split()[-1] if ":" in sf else sf
                    vf_name: str = vf.split(":")[0].strip().split()[-1] if ":" in vf else vf
                    if sf_name == vf_name:
                        field_name_match = sf
                        break

                if field_name_match is None and is_ghost:
                    report.struct_diffs.append(
                        StructDiff(
                            name=name,
                            change_type="ghost_field_added",
                            detail=f"Ghost field added to exec struct: {vf}",
                            severity="high",
                        )
                    )
                elif field_name_match is None and not is_ghost:
                    report.struct_diffs.append(
                        StructDiff(
                            name=name,
                            change_type="added_field",
                            detail=f"New exec field added: {vf}",
                            severity="critical",
                        )
                    )

            # Check for visibility changes (private → pub).
            for sf in source_fields:
                sf_name: str = sf.split(":")[0].strip().split()[-1] if ":" in sf else sf
                for vf in verus_fields:
                    vf_name = vf.split(":")[0].strip().split()[-1] if ":" in vf else vf
                    if sf_name == vf_name:
                        sf_is_pub: bool = sf.strip().startswith("pub")
                        vf_is_pub: bool = vf.strip().startswith("pub")
                        if not sf_is_pub and vf_is_pub:
                            report.struct_diffs.append(
                                StructDiff(
                                    name=name,
                                    change_type="visibility_changed",
                                    detail=f"Field '{sf_name}' changed from private to pub (Verus constraint).",
                                    severity="medium",
                                )
                            )
                        # Check type changes.
                        sf_type: str = sf.split(":", 1)[1].strip() if ":" in sf else ""
                        vf_type: str = vf.split(":", 1)[1].strip() if ":" in vf else ""
                        if sf_type != vf_type and sf_type and vf_type:
                            report.struct_diffs.append(
                                StructDiff(
                                    name=name,
                                    change_type="type_changed",
                                    detail=f"Field '{sf_name}' type changed: '{sf_type}' → '{vf_type}'.",
                                    severity="high",
                                )
                            )

    def _compare_functions(
        self,
        source_root: tree_sitter.Node,
        verus_root: tree_sitter.Node,
        report: ModuleDiffReport,
        source_code: str,
        verus_code: str,
    ) -> None:
        """Compare exec functions between source and verus."""
        source_fns: list[tree_sitter.Node] = self.parser.extract_exec_functions(
            source_root
        )
        verus_fns: list[tree_sitter.Node] = self.parser.extract_exec_functions(
            verus_root
        )

        source_fn_map: dict[str, tree_sitter.Node] = {
            self.parser.get_fn_name(fn): fn for fn in source_fns
        }
        verus_fn_map: dict[str, tree_sitter.Node] = {
            self.parser.get_fn_name(fn): fn for fn in verus_fns
        }

        # Missing functions.
        for name in source_fn_map:
            if name not in verus_fn_map:
                report.function_diffs.append(
                    FunctionDiff(
                        name=name,
                        change_type="missing_in_verus",
                        detail=f"Function exists in source but not in verus exec code.",
                        severity="high",
                        source_text=node_to_text(source_fn_map[name]),
                    )
                )

        # Added functions (non-spec, non-proof).
        for name in verus_fn_map:
            if name not in source_fn_map:
                # Helper functions might be OK.
                mods: list[str] = self.parser.extract_function_modifiers(verus_fn_map[name])
                fn_text: str = node_to_text(verus_fn_map[name])
                if "to_mask" in name or "spec_" in name:
                    sev: str = "low"
                else:
                    sev = "medium"
                report.function_diffs.append(
                    FunctionDiff(
                        name=name,
                        change_type="added_in_verus",
                        detail=f"New exec function in verus (not in source). Modifiers: {mods}",
                        severity=sev,
                        verus_text=fn_text,
                    )
                )

        # Compare matching functions.
        for name in source_fn_map:
            if name not in verus_fn_map:
                continue

            src_fn: tree_sitter.Node = source_fn_map[name]
            vrs_fn: tree_sitter.Node = verus_fn_map[name]

            # Compare signatures.
            src_params: str = self.parser.get_fn_params(src_fn)
            vrs_params: str = self.parser.get_fn_params(vrs_fn)
            src_ret: Optional[str] = self.parser.get_fn_return_type(src_fn)
            vrs_ret: Optional[str] = self.parser.get_fn_return_type(vrs_fn)

            if src_params != vrs_params:
                report.function_diffs.append(
                    FunctionDiff(
                        name=name,
                        change_type="signature_changed",
                        detail=f"Parameters differ: source='{src_params}' vs verus='{vrs_params}'",
                        severity="high",
                        source_text=node_to_text(src_fn),
                        verus_text=node_to_text(vrs_fn),
                    )
                )

            if src_ret != vrs_ret:
                report.function_diffs.append(
                    FunctionDiff(
                        name=name,
                        change_type="return_type_changed",
                        detail=f"Return type differs: source='{src_ret}' vs verus='{vrs_ret}'",
                        severity="high",
                        source_text=node_to_text(src_fn),
                        verus_text=node_to_text(vrs_fn),
                    )
                )

            # Compare bodies (stripping verus annotations).
            src_body: Optional[str] = self.parser.get_fn_body(src_fn)
            vrs_body: Optional[str] = self.parser.get_fn_body(vrs_fn)

            if src_body is not None and vrs_body is not None:
                stripped_vrs: str = self.parser.strip_verus_annotations(vrs_body)
                # Normalize whitespace for comparison.
                src_norm: str = " ".join(src_body.split())
                vrs_norm: str = " ".join(stripped_vrs.split())

                if src_norm != vrs_norm:
                    # Check tree hashes for semantic equivalence.
                    src_hash: str = self.parser.get_tree_hash(src_fn)
                    vrs_hash: str = self.parser.get_tree_hash(vrs_fn)

                    if src_hash == vrs_hash:
                        report.function_diffs.append(
                            FunctionDiff(
                                name=name,
                                change_type="body_changed_hash_match",
                                detail="Body text differs but tree hash matches (verification artifacts only).",
                                severity="low",
                                source_text=src_body,
                                verus_text=vrs_body,
                            )
                        )
                    else:
                        # Generate unified diff.
                        diff_lines: list[str] = list(
                            difflib.unified_diff(
                                src_body.splitlines(),
                                stripped_vrs.splitlines(),
                                fromfile="source",
                                tofile="verus",
                                lineterm="",
                            )
                        )
                        diff_text: str = "\n".join(diff_lines[:50])
                        report.function_diffs.append(
                            FunctionDiff(
                                name=name,
                                change_type="body_changed_semantic",
                                detail=f"Body differs semantically. Diff:\n{diff_text}",
                                severity="critical",
                                source_text=src_body,
                                verus_text=vrs_body,
                            )
                        )


# ──────────────────────────────────────────────────────────────────────────────
# Report Generation.
# ──────────────────────────────────────────────────────────────────────────────


def generate_markdown_report(reports: list[ModuleDiffReport]) -> str:
    """Generate a Markdown diff report."""
    lines: list[str] = [
        "# Verus Exec Code Diff Report",
        "",
        "This report compares executable code between original Nanvix source",
        "files and their Verus-verified counterparts.",
        "",
        "## Summary",
        "",
    ]

    total_critical: int = 0
    total_high: int = 0
    total_medium: int = 0
    total_low: int = 0

    for r in reports:
        for d in r.struct_diffs + r.function_diffs:
            if d.severity == "critical":
                total_critical += 1
            elif d.severity == "high":
                total_high += 1
            elif d.severity == "medium":
                total_medium += 1
            else:
                total_low += 1

    lines.append(f"| Severity | Count |")
    lines.append(f"|----------|-------|")
    lines.append(f"| Critical | {total_critical} |")
    lines.append(f"| High     | {total_high} |")
    lines.append(f"| Medium   | {total_medium} |")
    lines.append(f"| Low      | {total_low} |")
    lines.append("")

    for r in reports:
        lines.append(f"## Module: {r.module_name}")
        lines.append("")
        lines.append(f"- **Source:** `{r.source_path}`")
        lines.append(f"- **Verus:** `{r.verus_path}`")
        lines.append(f"- **Tree Hash Match:** {'✅ Yes' if r.tree_hash_match else '❌ No'}")
        lines.append(f"- **Summary:** {r.summary}")
        lines.append("")

        if r.struct_diffs:
            lines.append("### Struct Differences")
            lines.append("")
            lines.append("| Struct | Type | Severity | Detail |")
            lines.append("|--------|------|----------|--------|")
            for d in r.struct_diffs:
                lines.append(
                    f"| `{d.name}` | {d.change_type} | **{d.severity}** | {d.detail} |"
                )
            lines.append("")

        if r.function_diffs:
            lines.append("### Function Differences")
            lines.append("")
            for d in r.function_diffs:
                icon: str = {"critical": "🔴", "high": "🟠", "medium": "🟡", "low": "🟢"}.get(
                    d.severity, "⚪"
                )
                lines.append(f"#### {icon} `{d.name}` — {d.change_type} ({d.severity})")
                lines.append("")
                lines.append(d.detail)
                lines.append("")
                if d.source_text and d.verus_text and d.severity in ("critical", "high"):
                    lines.append("<details>")
                    lines.append("<summary>Source vs Verus code</summary>")
                    lines.append("")
                    lines.append("**Source:**")
                    lines.append("```rust")
                    lines.append(d.source_text[:2000])
                    lines.append("```")
                    lines.append("")
                    lines.append("**Verus:**")
                    lines.append("```rust")
                    lines.append(d.verus_text[:2000])
                    lines.append("```")
                    lines.append("</details>")
                    lines.append("")

        if not r.struct_diffs and not r.function_diffs:
            lines.append("*No differences found.*")
            lines.append("")

    lines.append("---")
    lines.append("*Generated by `scripts/verus_exec_diff.py` using tree-sitter-verus.*")

    return "\n".join(lines)


def generate_ai_review_prompt(reports: list[ModuleDiffReport]) -> str:
    """Generate an AI review prompt for semantic change detection."""
    lines: list[str] = [
        "# AI Review Request: Verus Exec Code Semantic Equivalence",
        "",
        "Please review the following differences between original Nanvix source code",
        "and the Verus-verified versions. For each difference:",
        "",
        "1. **Classify** whether the change is:",
        "   - **Ghost insertion** (verification artifact, no semantic change) → OK",
        "   - **Verus compatibility** (e.g., pub field for spec access) → Document",
        "   - **Semantic change** (logic, control flow, or type change) → REQUIRES FIX",
        "",
        "2. **Assess** whether the verified code preserves the original semantics.",
        "",
        "3. **Recommend** fixes for any semantic changes that break equivalence.",
        "",
    ]

    for r in reports:
        critical_or_high: list = [
            d for d in r.struct_diffs + r.function_diffs
            if d.severity in ("critical", "high")
        ]
        if not critical_or_high:
            continue

        lines.append(f"## Module: {r.module_name}")
        lines.append("")
        for d in critical_or_high:
            lines.append(f"### {d.name} — {d.change_type}")
            lines.append(f"Severity: {d.severity}")
            lines.append(f"Detail: {d.detail}")
            if hasattr(d, "source_text") and d.source_text:
                lines.append("")
                lines.append("Source:")
                lines.append("```rust")
                lines.append(d.source_text[:3000])
                lines.append("```")
            if hasattr(d, "verus_text") and d.verus_text:
                lines.append("")
                lines.append("Verus:")
                lines.append("```rust")
                lines.append(d.verus_text[:3000])
                lines.append("```")
            lines.append("")

    return "\n".join(lines)


# ──────────────────────────────────────────────────────────────────────────────
# Main.
# ──────────────────────────────────────────────────────────────────────────────


def main() -> None:
    """Main entry point."""
    arg_parser: argparse.ArgumentParser = argparse.ArgumentParser(
        description="Compare Nanvix source code with Verus-verified exec code."
    )
    arg_parser.add_argument(
        "--language-path",
        default="/tmp/verus-tools-venv/verus.so",
        help="Path to tree-sitter Verus language .so file.",
    )
    arg_parser.add_argument(
        "--output",
        default=None,
        help="Output report path (default: verus-ai-history/exec_diff_report.md).",
    )
    arg_parser.add_argument(
        "--ai-review",
        action="store_true",
        help="Also generate an AI review prompt file.",
    )
    arg_parser.add_argument(
        "--json",
        action="store_true",
        help="Also output a JSON report.",
    )
    args: argparse.Namespace = arg_parser.parse_args()

    # Build language if needed.
    if not os.path.exists(args.language_path):
        print(f"Building tree-sitter language at {args.language_path}...")
        Language.build_library(args.language_path, [os.path.join(REPO_ROOT, "tree-sitter-verus")])

    engine: VerusExecDiffEngine = VerusExecDiffEngine(args.language_path)

    # Filter mappings to only existing files.
    valid_mappings: list[dict[str, str]] = []
    for m in PM_MAPPINGS:
        src: str = os.path.join(REPO_ROOT, m["source"])
        vrs: str = os.path.join(REPO_ROOT, m["verus"])
        if os.path.exists(src) and os.path.exists(vrs):
            valid_mappings.append(m)
        else:
            if not os.path.exists(src):
                print(f"⚠️  Skipping {m['name']}: source not found ({m['source']})")
            if not os.path.exists(vrs):
                print(f"⚠️  Skipping {m['name']}: verus not found ({m['verus']})")

    # Run comparisons.
    reports: list[ModuleDiffReport] = []
    for m in valid_mappings:
        print(f"Comparing {m['name']}...")
        r: ModuleDiffReport = engine.compare_module(
            m["source"], m["verus"], m["name"]
        )
        reports.append(r)
        print(f"  → {r.summary}")

    # Generate reports.
    output_dir: str = os.path.join(REPO_ROOT, "verus-ai-history")
    os.makedirs(output_dir, exist_ok=True)

    output_path: str = args.output or os.path.join(
        output_dir, "exec_diff_report.md"
    )
    md_report: str = generate_markdown_report(reports)
    with open(output_path, "w") as f:
        f.write(md_report)
    print(f"\n📄 Report written to: {output_path}")

    if args.ai_review:
        ai_path: str = os.path.join(output_dir, "exec_diff_ai_review_prompt.md")
        ai_prompt: str = generate_ai_review_prompt(reports)
        with open(ai_path, "w") as f:
            f.write(ai_prompt)
        print(f"🤖 AI review prompt written to: {ai_path}")

    if args.json:
        json_path: str = os.path.join(output_dir, "exec_diff_report.json")
        json_data: list[dict] = []
        for r in reports:
            json_data.append({
                "module": r.module_name,
                "source": r.source_path,
                "verus": r.verus_path,
                "tree_hash_match": r.tree_hash_match,
                "summary": r.summary,
                "struct_diffs": [
                    {"name": d.name, "type": d.change_type, "severity": d.severity, "detail": d.detail}
                    for d in r.struct_diffs
                ],
                "function_diffs": [
                    {"name": d.name, "type": d.change_type, "severity": d.severity, "detail": d.detail}
                    for d in r.function_diffs
                ],
            })
        with open(json_path, "w") as f:
            json.dump(json_data, f, indent=2)
        print(f"📊 JSON report written to: {json_path}")


if __name__ == "__main__":
    main()
