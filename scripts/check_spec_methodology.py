#!/usr/bin/env python3
# Copyright(c) The Maintainers of Nanvix.
# Licensed under the MIT License.

"""
Spec methodology checker using tree-sitter.

Analyzes Verus split files against the guidelines in specifying-and-proving-types.md:
  Step 1: View types use abstract types; view() is pub closed spec fn.
  Step 2: inv() exists and is pub closed spec fn.
  Step 3: Public method specs only use self@/self.inv()/self.view(), not self.field.
  Step 5: No assume/admit/unjustified external_body.

Usage:
    python3 scripts/check_spec_methodology.py <verus_dir> [--module NAME]
    python3 scripts/check_spec_methodology.py verus/split/kernel/pm/process/state --module runnable

Output: JSON report to stdout, markdown report to --output if specified.
"""

import argparse
import json
import os
import re
import sys
from pathlib import Path
from typing import Any, Dict, List, Optional, Tuple

# Tree-sitter setup.
LANGUAGE_SO = os.environ.get("VERUS_LANGUAGE_SO", "/tmp/verus-tools-venv/verus.so")

try:
    from tree_sitter import Language, Parser

    _language = Language(LANGUAGE_SO, "rust")
    _parser = Parser()
    _parser.set_language(_language)
    HAS_TREE_SITTER = True
except Exception:
    HAS_TREE_SITTER = False


def node_text(node) -> str:
    """Extract text from a tree-sitter node."""
    return node.text.decode("utf-8")


def parse_file(path: str) -> Any:
    """Parse a file and return the tree-sitter tree."""
    with open(path, "r") as f:
        content = f.read()
    return _parser.parse(bytes(content, "utf-8")), content


def extract_structs(root) -> List[Tuple[str, Any]]:
    """Extract all struct definitions with their names."""
    query = _language.query("(struct_item) @struct")
    captures = query.captures(root)
    results = []
    for node, _ in captures:
        name_node = node.child_by_field_name("name")
        if name_node:
            results.append((node_text(name_node), node))
    return results


def extract_functions(root) -> List[Tuple[str, Any, List[str]]]:
    """Extract all functions with names and modifiers."""
    query = _language.query("(function_item) @fn")
    captures = query.captures(root)
    results = []
    for node, _ in captures:
        name_node = node.child_by_field_name("name")
        if not name_node:
            continue
        name = node_text(name_node)
        # Extract modifiers.
        mod_query = _language.query("(function_modifiers) @mods")
        mod_captures = mod_query.captures(node)
        modifiers = []
        for mod_node, _ in mod_captures:
            for child in mod_node.children:
                modifiers.append(node_text(child))
        results.append((name, node, modifiers))
    return results


def extract_impl_blocks(root) -> List[Tuple[str, Any]]:
    """Extract impl blocks with their type names."""
    query = _language.query("(impl_item) @impl")
    captures = query.captures(root)
    results = []
    for node, _ in captures:
        type_node = node.child_by_field_name("type")
        if type_node:
            results.append((node_text(type_node), node))
    return results


def check_view_type_abstraction(spec_structs: List[Tuple[str, Any]]) -> List[Dict]:
    """Step 1: Check View types use abstract types."""
    issues = []
    CONCRETE_TYPES = {
        "i8", "i16", "i32", "i64", "i128", "isize",
        "u8", "u16", "u32", "u64", "u128", "usize",
        "bool",  # bool is OK in View types.
        "f32", "f64",
    }
    # Concrete container types that should be abstract.
    CONCRETE_CONTAINERS = {"Vec", "HashMap", "HashSet", "BTreeMap", "BTreeSet", "String"}

    for name, node in spec_structs:
        if not name.endswith("View"):
            continue
        text = node_text(node)
        # Check for concrete integer types in field definitions.
        for concrete_t in ["i32", "i64", "u8", "u16", "u32", "u64", "usize", "isize"]:
            # Look for field type annotations like `: u32` or `: Vec<u32>`.
            pattern = rf":\s*{concrete_t}\b"
            if re.search(pattern, text):
                issues.append({
                    "step": 1,
                    "severity": "medium",
                    "type": name,
                    "issue": f"View type uses concrete type `{concrete_t}` instead of `int`",
                    "location": f"struct {name}",
                    "guideline": "View types should use int instead of i32/u64 etc.",
                })
        for container in CONCRETE_CONTAINERS:
            if container in text:
                issues.append({
                    "step": 1,
                    "severity": "high",
                    "type": name,
                    "issue": f"View type uses concrete container `{container}` instead of Seq/Set/Map",
                    "location": f"struct {name}",
                    "guideline": "View types should use Seq/Set/Map instead of Vec/HashMap etc.",
                })
    return issues


def check_view_fn(functions: List[Tuple[str, Any, List[str]]]) -> List[Dict]:
    """Step 1: Check view() is pub closed spec fn."""
    issues = []
    found_view = False
    for name, node, modifiers in functions:
        if name != "view":
            continue
        found_view = True
        text = node_text(node)
        is_pub = "pub" in modifiers
        is_spec = "spec" in modifiers
        # Check for 'closed' - it appears as a modifier or in function_modifiers.
        is_closed = "closed" in modifiers
        # view() via View trait impl is OK - those use `open spec fn`.
        # Only flag if it's a direct impl method that's open.
        if "open" in modifiers and "View for" not in text:
            issues.append({
                "step": 1,
                "severity": "high",
                "type": "view()",
                "issue": "view() should be `pub closed spec fn`, not `open`",
                "location": "fn view()",
                "guideline": "view() is public but closed so users can't see internals.",
            })
    return issues


def check_inv_fn(functions: List[Tuple[str, Any, List[str]]],
                 impl_type: str) -> List[Dict]:
    """Step 2: Check inv() exists and is pub closed spec fn."""
    issues = []
    found_inv = False
    for name, node, modifiers in functions:
        if name == "inv" or name == "wf":
            found_inv = True
            is_pub = "pub" in modifiers
            is_spec = "spec" in modifiers
            is_closed = "closed" in modifiers
            is_open = "open" in modifiers
            if is_open and not is_closed:
                issues.append({
                    "step": 2,
                    "severity": "medium",
                    "type": impl_type,
                    "issue": f"{name}() is `open spec fn` but guideline says `pub closed spec fn`",
                    "location": f"fn {name}()",
                    "guideline": "inv() should be pub closed so users can't see implementation internals.",
                })
    if not found_inv:
        issues.append({
            "step": 2,
            "severity": "high",
            "type": impl_type,
            "issue": "No inv()/wf() function found",
            "location": f"impl {impl_type}",
            "guideline": "Step 2: Write inv() as pub closed spec fn for implementation invariants.",
        })
    return issues


def check_public_method_specs(functions: List[Tuple[str, Any, List[str]]],
                               content: str, impl_type: str) -> List[Dict]:
    """Step 3: Public method specs should only use self@, not self.field."""
    issues = []
    for name, node, modifiers in functions:
        if "pub" not in modifiers:
            continue
        if "spec" in modifiers or "proof" in modifiers:
            continue
        # This is a public exec function. Check its specs.
        specs_node = node.child_by_field_name("specifications")
        if specs_node is None:
            continue
        spec_text = node_text(specs_node)

        # Check for self.field_name patterns (not self@ or self.inv() or self.view()).
        # Pattern: self.something where something is not inv()/view()/@.
        self_field_pattern = r"\bself\.(?!inv\b|view\b|@)(\w+)"
        matches = re.findall(self_field_pattern, spec_text)
        # Filter out method calls (followed by '(').
        field_refs = []
        for m in matches:
            # Check if it's a method call.
            pattern_with_paren = rf"\bself\.{m}\s*\("
            if not re.search(pattern_with_paren, spec_text):
                field_refs.append(m)
        if field_refs:
            issues.append({
                "step": 3,
                "severity": "high",
                "type": impl_type,
                "issue": f"Public method `{name}()` spec references self.{', self.'.join(set(field_refs))} directly",
                "location": f"fn {name}()",
                "guideline": "Public specs should use self@.field (view), not self.field (implementation).",
            })

        # Check for inv() in requires/ensures.
        has_requires_inv = "self.inv()" in spec_text or "old(self).inv()" in spec_text
        has_ensures_inv = "self.inv()" in spec_text
        if "&self" in node_text(node) or "&mut self" in node_text(node):
            if not has_requires_inv and "inv" not in spec_text and "wf" not in spec_text:
                issues.append({
                    "step": 3,
                    "severity": "medium",
                    "type": impl_type,
                    "issue": f"Public method `{name}()` may be missing inv()/wf() in requires",
                    "location": f"fn {name}()",
                    "guideline": "Public methods should require self.inv() and ensure self.inv().",
                })
    return issues


def check_no_cheating(root) -> List[Dict]:
    """Step 5: No assume/admit."""
    issues = []
    # Use tree-sitter queries.
    query = _language.query("(call_expression) @call")
    captures = query.captures(root)
    for node, _ in captures:
        fn_node = node.child_by_field_name("function")
        if fn_node:
            fn_name = node_text(fn_node).strip()
            if fn_name == "assume":
                issues.append({
                    "step": 5,
                    "severity": "critical",
                    "type": "cheating",
                    "issue": f"assume() found at line {node.start_point[0] + 1}",
                    "location": f"line {node.start_point[0] + 1}",
                    "guideline": "Remove all assume() before declaring success.",
                })
            elif fn_name == "admit":
                issues.append({
                    "step": 5,
                    "severity": "critical",
                    "type": "cheating",
                    "issue": f"admit() found at line {node.start_point[0] + 1}",
                    "location": f"line {node.start_point[0] + 1}",
                    "guideline": "Remove all admit() before declaring success.",
                })
    return issues


def analyze_module(verus_dir: str, file_stem: str) -> Dict:
    """Analyze a single module's spec methodology compliance."""
    spec_file = os.path.join(verus_dir, f"{file_stem}.spec.rs")
    exec_file = os.path.join(verus_dir, f"{file_stem}.rs")
    proof_file = os.path.join(verus_dir, f"{file_stem}.proof.rs")

    report = {
        "module": file_stem,
        "verus_dir": verus_dir,
        "files_found": {},
        "issues": [],
        "summary": {},
    }

    # Analyze spec file.
    if os.path.exists(spec_file):
        report["files_found"]["spec"] = spec_file
        tree, content = parse_file(spec_file)
        root = tree.root_node

        structs = extract_structs(root)
        report["issues"].extend(check_view_type_abstraction(structs))

        functions = extract_functions(root)
        report["issues"].extend(check_view_fn(functions))
        report["issues"].extend(check_no_cheating(root))

        # Check inv/wf in spec file impl blocks.
        impl_blocks = extract_impl_blocks(root)
        for impl_name, impl_node in impl_blocks:
            impl_fns = extract_functions(impl_node)
            # Only check inv for non-View types.
            if not impl_name.endswith("View"):
                report["issues"].extend(check_inv_fn(impl_fns, impl_name))
    else:
        report["issues"].append({
            "step": 0,
            "severity": "critical",
            "type": "missing_file",
            "issue": f"Spec file not found: {spec_file}",
            "location": spec_file,
            "guideline": "Three-file split requires .spec.rs file.",
        })

    # Analyze exec file.
    if os.path.exists(exec_file):
        report["files_found"]["exec"] = exec_file
        tree, content = parse_file(exec_file)
        root = tree.root_node

        # Check public method specs.
        impl_blocks = extract_impl_blocks(root)
        for impl_name, impl_node in impl_blocks:
            impl_fns = extract_functions(impl_node)
            report["issues"].extend(
                check_public_method_specs(impl_fns, content, impl_name)
            )

        report["issues"].extend(check_no_cheating(root))
    else:
        report["issues"].append({
            "step": 0,
            "severity": "critical",
            "type": "missing_file",
            "issue": f"Exec file not found: {exec_file}",
            "location": exec_file,
            "guideline": "Three-file split requires .rs file.",
        })

    # Analyze proof file.
    if os.path.exists(proof_file):
        report["files_found"]["proof"] = proof_file
        tree, _ = parse_file(proof_file)
        report["issues"].extend(check_no_cheating(tree.root_node))

    # Summary.
    by_severity = {}
    for issue in report["issues"]:
        sev = issue["severity"]
        by_severity[sev] = by_severity.get(sev, 0) + 1
    report["summary"] = {
        "total_issues": len(report["issues"]),
        "by_severity": by_severity,
        "by_step": {},
    }
    for issue in report["issues"]:
        step = issue["step"]
        report["summary"]["by_step"][step] = report["summary"]["by_step"].get(step, 0) + 1

    return report


def format_markdown(report: Dict) -> str:
    """Format a report as markdown."""
    lines = []
    lines.append(f"# Spec Methodology Report: {report['module']}")
    lines.append("")
    lines.append(f"**Directory:** {report['verus_dir']}")
    lines.append(f"**Total issues:** {report['summary']['total_issues']}")
    lines.append("")

    if not report["issues"]:
        lines.append("✅ No methodology violations found.")
        return "\n".join(lines)

    # Group by severity.
    for severity in ["critical", "high", "medium", "low"]:
        issues = [i for i in report["issues"] if i["severity"] == severity]
        if not issues:
            continue
        lines.append(f"## {severity.upper()} ({len(issues)})")
        lines.append("")
        for i, issue in enumerate(issues, 1):
            lines.append(f"{i}. **Step {issue['step']}** [{issue['type']}] "
                         f"at `{issue['location']}`")
            lines.append(f"   - {issue['issue']}")
            lines.append(f"   - Guideline: {issue['guideline']}")
            lines.append("")

    return "\n".join(lines)


def main() -> int:
    """Main entry point."""
    parser = argparse.ArgumentParser(description="Check Verus spec methodology compliance")
    parser.add_argument("verus_dir", help="Directory containing the verus split files")
    parser.add_argument("--module", "-m", help="Module file stem (e.g., 'runnable')")
    parser.add_argument("--output", "-o", help="Output markdown report file")
    parser.add_argument("--json", action="store_true", help="Output JSON instead of markdown")
    args = parser.parse_args()

    if not HAS_TREE_SITTER:
        print("ERROR: tree-sitter not available. Install with: pip install tree_sitter==0.21.3",
              file=sys.stderr)
        return 1

    verus_dir = args.verus_dir

    if args.module:
        modules = [args.module]
    else:
        # Auto-discover modules from .rs files (excluding spec/proof/mod/lib).
        modules = []
        for f in sorted(Path(verus_dir).glob("*.rs")):
            if f.name.endswith(".spec.rs") or f.name.endswith(".proof.rs"):
                continue
            if f.name in ("mod.rs", "lib.rs"):
                continue
            modules.append(f.stem)

    all_reports = []
    for module in modules:
        report = analyze_module(verus_dir, module)
        all_reports.append(report)

    if args.json:
        output = json.dumps(all_reports, indent=2)
    else:
        parts = [format_markdown(r) for r in all_reports]
        output = "\n\n---\n\n".join(parts)

    if args.output:
        Path(args.output).parent.mkdir(parents=True, exist_ok=True)
        with open(args.output, "w") as f:
            f.write(output)
        print(f"Report written to {args.output}")
    else:
        print(output)

    total = sum(r["summary"]["total_issues"] for r in all_reports)
    print(f"\nTotal: {total} issues across {len(all_reports)} modules", file=sys.stderr)
    return 0


if __name__ == "__main__":
    sys.exit(main())
