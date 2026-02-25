#!/usr/bin/env python3
# Copyright(c) The Maintainers of Nanvix.
# Licensed under the MIT License.

"""
Exec consistency checker using tree-sitter AST hashing.

Uses Tianyu's tree-sitter-verus parser to compare exec functions between
the original source and the Verus verified version at the AST level.
Ghost/proof annotations are stripped before comparison so only executable
logic is compared.

Usage:
    python3 scripts/check_exec_consistency.py <source_file> <verus_exec_file>
    python3 scripts/check_exec_consistency.py src/kernel/src/pm/process/state/runnable.rs \
        verus/split/kernel/pm/process/state/runnable.rs --output report.md
"""

import argparse
import hashlib
import json
import os
import re
import sys
from pathlib import Path
from typing import Any, Dict, List, Optional, Tuple

# Tree-sitter setup.
LANGUAGE_SO = os.environ.get("VERUS_LANGUAGE_SO", os.path.join(os.path.dirname(os.path.dirname(os.path.abspath(__file__))), "verus-tools-venv", "verus.so"))

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
    """Parse a file and return the tree-sitter tree and content."""
    with open(path, "r") as f:
        content = f.read()
    return _parser.parse(bytes(content, "utf-8")), content


def _is_return_name_binding(node, parent) -> bool:
    """Check if a node is part of a Verus named return binding (e.g., `(result: T)`).

    Verus uses `-> (name: Type)` for named returns; standard Rust uses `-> Type`.
    This is always different but semantically equivalent, so we skip it in hashing.
    """
    if parent is None or parent.type != "function_item":
        return False
    # Find the '->' token in siblings; named return parts come after it.
    children = parent.children
    after_arrow = False
    block_reached = False
    for child in children:
        if child.type == "block":
            block_reached = True
        if after_arrow and not block_reached:
            # These are return-type related tokens.
            # In named return: ( identifier : Type )
            # The identifier and surrounding parens/colon are extra.
            if child.id == node.id:
                text = node_text(node)
                # Skip the parentheses and the name:colon pair.
                if node.type == "identifier" and text != parent.child_by_field_name("name").text.decode("utf-8"):
                    return True
                if text == "(" or text == ")" or text == ":":
                    return True
        if node_text(child) == "->":
            after_arrow = True
    return False


def get_tree_hash(node, parent=None) -> str:
    """
    Compute a structural hash of an AST node, ignoring ghost/proof annotations.

    Based on Tianyu's get_tree_hash from verus_parser_example.py.
    """
    try:
        # Skip Verus named return binding tokens (e.g., `(result:` and `)` in `-> (result: T)`).
        if _is_return_name_binding(node, parent):
            return ""

        children_hash_list = [
            get_tree_hash(child, parent=node)
            for child in node.children
            if len(node_text(child)) > 1
        ]
        children_hash_list = [h for h in children_hash_list if len(h) > 0]
        children_hash = " ".join(children_hash_list).strip()

        leaf_node = len(node.children) == 0
        if leaf_node:
            node_representation = node_text(node)
        else:
            node_representation = node.type + children_hash
        node_hash = hashlib.md5(node_representation.encode()).hexdigest()

        # Ignore ghost/proof/verification constructs.
        if node.type == "use_declaration":
            node_hash = ""
        if node.type in ("attribute_item", "inner_attribute_item"):
            node_hash = ""
        # Ignore Verus requires/ensures/invariant specifications.
        if node.type == "function_specifications":
            node_hash = ""
        # Ignore proof blocks (proof { ... }).
        if node.type == "proof_block":
            node_hash = ""
        if node.type == "let_declaration":
            if any(node_text(child) == "ghost" for child in node.children):
                node_hash = ""
        if node.type == "call_expression":
            if node_text(node).strip().startswith("reveal"):
                node_hash = ""
        if not leaf_node and children_hash == "":
            node_hash = ""
    except RecursionError:
        node_hash = ""
    return node_hash


def extract_function_modifiers(node) -> List[str]:
    """Extract modifiers from a function node."""
    query = _language.query("(function_modifiers) @mods")
    captures = query.captures(node)
    if not captures:
        return []
    mod_node = captures[0][0]
    return [node_text(child) for child in mod_node.children]


def _find_verus_ranges(content: str) -> List[Tuple[int, int]]:
    """Find line ranges (0-indexed) of verus! { ... } blocks by text search."""
    lines = content.split("\n")
    ranges = []
    for i, line in enumerate(lines):
        if re.match(r"^verus!\s*\{", line.strip()):
            for j in range(i + 1, len(lines)):
                if re.match(r"^}\s*//\s*verus!", lines[j].strip()):
                    ranges.append((i, j))
                    break
    return ranges


def _classify_verus_function(node, verus_ranges: List[Tuple[int, int]]) -> str:
    """Classify a function as VERIFIED, EXTERNAL_BODY, or UNVERIFIED."""
    line = node.start_point[0]
    in_verus = any(s <= line <= e for s, e in verus_ranges)
    if not in_verus:
        return "UNVERIFIED"
    text = node_text(node)[:500]
    if "external_body" in text:
        return "EXTERNAL_BODY"
    return "VERIFIED"


def extract_exec_functions(root, content: str = "") -> List[Tuple[str, Any, str, str]]:
    """Extract exec (non-spec, non-proof) functions with names, hashes, and verification status.

    Returns list of (name, node, hash, verification_status).
    verification_status is one of: VERIFIED, EXTERNAL_BODY, UNVERIFIED.
    """
    verus_ranges = _find_verus_ranges(content) if content else []

    query = _language.query("(function_item) @fn")
    captures = query.captures(root)
    results = []
    for node, _ in captures:
        modifiers = extract_function_modifiers(node)
        if "spec" in modifiers or "proof" in modifiers:
            continue
        name_node = node.child_by_field_name("name")
        if not name_node:
            continue
        name = node_text(name_node)
        # Hash the function body, ignoring ghost constructs.
        fn_hash = get_tree_hash(node)
        vstatus = _classify_verus_function(node, verus_ranges) if verus_ranges else ""
        results.append((name, node, fn_hash, vstatus))
    return results


def extract_struct_defs(root) -> List[Tuple[str, Any, str]]:
    """Extract struct definitions with names and hashes."""
    query = _language.query("(struct_item) @s")
    captures = query.captures(root)
    results = []
    for node, _ in captures:
        name_node = node.child_by_field_name("name")
        if not name_node:
            continue
        name = node_text(name_node)
        s_hash = get_tree_hash(node)
        results.append((name, node, s_hash))
    return results


def strip_verus_annotations(content: str) -> str:
    """
    Strip Verus-specific annotations from exec code for text-level diff.

    Removes requires/ensures/invariant blocks, proof blocks, ghost vars, etc.
    """
    # Remove proof blocks: proof { ... } (handles nested braces).
    content = _strip_balanced_block(content, r"\bproof\s*")
    # Remove ghost let bindings.
    content = re.sub(r"\blet\s+ghost\b[^;]*;", "", content)
    # Remove assert/assume/admit statements with by-blocks.
    content = _strip_balanced_block(content, r"\bassert\s+")
    content = re.sub(r"\b(assert|assume|admit)\s*\([^)]*\)\s*;", "", content)
    # Remove requires/ensures/invariant spec blocks (line-based).
    content = _strip_spec_lines(content)
    # Remove Verus named return: -> (name: Type) => -> Type.
    content = re.sub(r"->\s*\(\s*\w+\s*:\s*", "-> ", content)
    content = re.sub(r"(->.*\S)\s*\)\s*$", r"\1", content, flags=re.MULTILINE)
    # Remove #[trigger], #![trigger ...], #![auto], #[verifier::...].
    content = re.sub(r"#\[trigger\]", "", content)
    content = re.sub(r"#!\[trigger[^\]]*\]", "", content)
    content = re.sub(r"#!\[auto\]", "", content)
    content = re.sub(r"#\[verifier::[^\]]*\]\s*", "", content)
    content = re.sub(r"#\[cfg\(not\(verus_keep_ghost\)\)\]\s*", "", content)
    # Collapse multiple blank lines.
    content = re.sub(r"\n{3,}", "\n\n", content)
    return content


def _strip_spec_lines(content: str) -> str:
    """Remove requires/ensures/recommends/invariant/decreases blocks line-by-line.

    Spec blocks start with a keyword and end at a line containing only '{' or '}'
    (the function/loop body delimiter), or at the next spec keyword.
    """
    lines = content.split("\n")
    result = []
    in_spec = False

    for line in lines:
        stripped = line.strip()

        # Detect start of spec block.
        if not in_spec and re.match(
            r"^\s*(requires|ensures|recommends|invariant_except_break|invariant|decreases)\b",
            stripped,
        ):
            in_spec = True
            continue

        if in_spec:
            # Spec block ends at a line that is just '{' (body start).
            if stripped == "{":
                in_spec = False
                result.append(line)
                continue
            # Another spec keyword continues stripping.
            if re.match(
                r"^\s*(requires|ensures|recommends|invariant_except_break|invariant|decreases)\b",
                stripped,
            ):
                continue
            # Skip spec content lines.
            continue

        result.append(line)

    return "\n".join(result)


def _strip_balanced_block(content: str, prefix_pattern: str) -> str:
    """Strip blocks matching prefix_pattern { ... } with balanced brace matching."""
    result = []
    i = 0
    pat = re.compile(prefix_pattern + r"\{")
    while i < len(content):
        m = pat.search(content, i)
        if not m:
            result.append(content[i:])
            break
        result.append(content[i:m.start()])
        # Find matching closing brace.
        depth = 1
        j = m.end()
        while j < len(content) and depth > 0:
            if content[j] == '{':
                depth += 1
            elif content[j] == '}':
                depth -= 1
            j += 1
        # Skip trailing semicolons/newlines.
        while j < len(content) and content[j] in ' \t\n;':
            j += 1
        i = j
    return "".join(result)


def strip_ghost_from_function(fn_text: str) -> str:
    """
    Strip all Verus ghost/proof/spec annotations from a function body.

    Returns the exec-only version of the function for comparison with source.
    """
    return strip_verus_annotations(fn_text)


def compare_modules(
    source_path: str, verus_path: str
) -> Dict:
    """Compare exec code between source and Verus version."""
    report: Dict[str, Any] = {
        "source": source_path,
        "verus": verus_path,
        "functions": [],
        "structs": [],
        "summary": {},
    }

    # Parse both files.
    src_tree, src_content = parse_file(source_path)
    verus_tree, verus_content = parse_file(verus_path)

    # Compare exec functions.
    src_fns = {name: (node, h, vs) for name, node, h, vs in extract_exec_functions(src_tree.root_node)}
    verus_fns = {name: (node, h, vs) for name, node, h, vs in extract_exec_functions(verus_tree.root_node, verus_content)}

    matched = 0
    mismatched = 0
    missing_in_verus = 0
    extra_in_verus = 0

    # Check source functions.
    for name in sorted(src_fns.keys()):
        src_node, src_hash, _ = src_fns[name]
        if name in verus_fns:
            verus_node, verus_hash, vstatus = verus_fns[name]
            if src_hash == verus_hash or src_hash == "" or verus_hash == "":
                status = "MATCH"
                matched += 1
            else:
                status = "MISMATCH"
                mismatched += 1
            report["functions"].append({
                "name": name,
                "status": status,
                "src_hash": src_hash,
                "verus_hash": verus_hash,
                "src_lines": f"{src_node.start_point[0]+1}-{src_node.end_point[0]+1}",
                "verus_lines": f"{verus_node.start_point[0]+1}-{verus_node.end_point[0]+1}",
                "verification": vstatus,
            })
        else:
            missing_in_verus += 1
            report["functions"].append({
                "name": name,
                "status": "MISSING_IN_VERUS",
                "src_hash": src_hash,
                "verus_hash": "",
                "src_lines": f"{src_node.start_point[0]+1}-{src_node.end_point[0]+1}",
                "verus_lines": "",
                "verification": "",
            })

    # Check for extra functions in Verus.
    for name in sorted(verus_fns.keys()):
        if name not in src_fns:
            verus_node, verus_hash, vstatus = verus_fns[name]
            extra_in_verus += 1
            report["functions"].append({
                "name": name,
                "status": "EXTRA_IN_VERUS",
                "src_hash": "",
                "verus_hash": verus_hash,
                "src_lines": "",
                "verus_lines": f"{verus_node.start_point[0]+1}-{verus_node.end_point[0]+1}",
                "verification": vstatus,
            })

    # Compare struct definitions.
    src_structs = {name: (node, h) for name, node, h in extract_struct_defs(src_tree.root_node)}
    verus_structs = {name: (node, h) for name, node, h in extract_struct_defs(verus_tree.root_node)}

    struct_match = 0
    struct_mismatch = 0
    for name in sorted(set(src_structs.keys()) | set(verus_structs.keys())):
        entry: Dict[str, Any] = {"name": name, "src_lines": "", "verus_lines": ""}
        if name in src_structs and name in verus_structs:
            src_node, src_h = src_structs[name]
            verus_node, verus_h = verus_structs[name]
            entry["src_lines"] = f"{src_node.start_point[0]+1}-{src_node.end_point[0]+1}"
            entry["verus_lines"] = f"{verus_node.start_point[0]+1}-{verus_node.end_point[0]+1}"
            if src_h == verus_h or src_h == "" or verus_h == "":
                status = "MATCH"
                struct_match += 1
            else:
                status = "MISMATCH"
                struct_mismatch += 1
        elif name in src_structs:
            src_node, _ = src_structs[name]
            entry["src_lines"] = f"{src_node.start_point[0]+1}-{src_node.end_point[0]+1}"
            status = "MISSING_IN_VERUS"
            struct_mismatch += 1
        else:
            verus_node, _ = verus_structs[name]
            entry["verus_lines"] = f"{verus_node.start_point[0]+1}-{verus_node.end_point[0]+1}"
            # View types and ghost structs are expected extra.
            if name.endswith("View") or name.endswith("Ghost"):
                status = "EXPECTED_EXTRA"
                struct_match += 1
            else:
                status = "EXTRA_IN_VERUS"
                struct_mismatch += 1
        entry["status"] = status
        report["structs"].append(entry)

    report["summary"] = {
        "functions": {
            "matched": matched,
            "mismatched": mismatched,
            "missing_in_verus": missing_in_verus,
            "extra_in_verus": extra_in_verus,
            "total_source": len(src_fns),
            "total_verus": len(verus_fns),
        },
        "structs": {
            "matched": struct_match,
            "mismatched": struct_mismatch,
        },
        "consistent": mismatched == 0 and missing_in_verus == 0,
    }

    return report


def format_markdown(report: Dict) -> str:
    """Format report as markdown."""
    lines = []
    lines.append(f"# Exec Consistency Report")
    lines.append("")
    lines.append(f"**Source:** `{report['source']}`")
    lines.append(f"**Verus:** `{report['verus']}`")
    lines.append("")

    s = report["summary"]
    fn_s = s["functions"]
    lines.append("## Summary")
    lines.append("")
    lines.append(f"- Functions matched: {fn_s['matched']}/{fn_s['total_source']}")
    lines.append(f"- Functions mismatched: {fn_s['mismatched']}")
    lines.append(f"- Missing in Verus: {fn_s['missing_in_verus']}")
    lines.append(f"- Extra in Verus: {fn_s['extra_in_verus']}")
    lines.append(f"- **Consistent: {'YES' if s['consistent'] else 'NO'}**")
    lines.append("")

    # Functions table.
    problem_fns = [f for f in report["functions"] if f["status"] != "MATCH"]
    if problem_fns:
        lines.append("## Inconsistent Functions")
        lines.append("")
        lines.append("| Function | Status | Source Lines | Verus Lines |")
        lines.append("|----------|--------|-------------|-------------|")
        for f in problem_fns:
            name = f['name']
            status = f['status']
            # Add links to diff/source/verus files.
            links = []
            if status == "MISMATCH":
                links.append(f"[{name}.diff]({name}.diff)")
                links.append(f"[{name}_source.rs]({name}_source.rs)")
                links.append(f"[{name}_verus.rs]({name}_verus.rs)")
            elif status == "EXTRA_IN_VERUS":
                links.append(f"[{name}_verus.rs]({name}_verus.rs)")
            elif status == "MISSING_IN_VERUS":
                links.append(f"[{name}_source.rs]({name}_source.rs)")
            link_str = " ".join(links)
            name_cell = f"`{name}` {link_str}" if links else f"`{name}`"
            lines.append(f"| {name_cell} | {status} | {f['src_lines']} | {f['verus_lines']} |")
        lines.append("")

    # All functions.
    lines.append("## All Functions")
    lines.append("")
    lines.append("| Function | Status | Hash Match | Verification |")
    lines.append("|----------|--------|------------|--------------|")
    for f in report["functions"]:
        name = f['name']
        status = f['status']
        vstatus = f.get('verification', '')
        match_str = "✅" if status == "MATCH" else "❌"
        if vstatus == "UNVERIFIED":
            verify_str = "⚠️ UNVERIFIED"
        elif vstatus == "EXTERNAL_BODY":
            verify_str = "🔒 external_body"
        elif vstatus == "VERIFIED":
            verify_str = "✅ verified"
        else:
            verify_str = ""
        if status == "EXTRA_IN_VERUS":
            name_cell = f"`{name}` [{name}_verus.rs]({name}_verus.rs)"
        else:
            name_cell = f"`{name}`"
        lines.append(f"| {name_cell} | {status} | {match_str} | {verify_str} |")
    lines.append("")

    # Unverified functions warning.
    unverified = [f for f in report["functions"] if f.get("verification") == "UNVERIFIED"]
    external_body = [f for f in report["functions"] if f.get("verification") == "EXTERNAL_BODY"]
    if unverified or external_body:
        lines.append("## Verification Coverage")
        lines.append("")
        if unverified:
            lines.append(f"**⚠️ {len(unverified)} function(s) are UNVERIFIED** (outside `verus!` block):")
            lines.append("")
            for f in unverified:
                lines.append(f"- `{f['name']}` (lines {f.get('verus_lines', f.get('src_lines', '?'))})")
            lines.append("")
            lines.append("These functions are not checked by Verus at all. Justify why each")
            lines.append("cannot be verified, or move them inside `verus!` with proper contracts.")
            lines.append("")
        if external_body:
            lines.append(f"**🔒 {len(external_body)} function(s) use `external_body`** (body not verified):")
            lines.append("")
            for f in external_body:
                lines.append(f"- `{f['name']}` (lines {f.get('verus_lines', '?')})")
            lines.append("")
            lines.append("These functions have requires/ensures contracts but the body is trusted.")
            lines.append("Justify why `external_body` is necessary for each.")
            lines.append("")

    # Struct comparison.
    problem_structs = [s for s in report["structs"] if s["status"] not in ("MATCH", "EXPECTED_EXTRA")]
    if problem_structs:
        lines.append("## Inconsistent Structs")
        lines.append("")
        lines.append("| Struct | Status | Source Lines | Verus Lines |")
        lines.append("|--------|--------|-------------|-------------|")
        for s in problem_structs:
            lines.append(f"| `{s['name']}` | {s['status']} | {s.get('src_lines', '')} | {s.get('verus_lines', '')} |")
        lines.append("")

    return "\n".join(lines)


def main() -> int:
    """Main entry point."""
    parser = argparse.ArgumentParser(
        description="Check exec code consistency using tree-sitter AST hashing"
    )
    parser.add_argument("source", help="Original source file path")
    parser.add_argument("verus", help="Verus verified exec file path")
    parser.add_argument("--output", "-o", help="Output markdown report file")
    parser.add_argument("--json", action="store_true", help="Output JSON instead of markdown")
    args = parser.parse_args()

    if not HAS_TREE_SITTER:
        print(
            "ERROR: tree-sitter not available. Install: pip install tree_sitter==0.21.3",
            file=sys.stderr,
        )
        return 1

    if not os.path.exists(args.source):
        print(f"ERROR: Source file not found: {args.source}", file=sys.stderr)
        return 1
    if not os.path.exists(args.verus):
        print(f"ERROR: Verus file not found: {args.verus}", file=sys.stderr)
        return 1

    report = compare_modules(args.source, args.verus)

    if args.json:
        output = json.dumps(report, indent=2)
    else:
        output = format_markdown(report)

    if args.output:
        Path(args.output).parent.mkdir(parents=True, exist_ok=True)
        with open(args.output, "w") as f:
            f.write(output)
        print(f"Report written to {args.output}", file=sys.stderr)
    else:
        print(output)

    if not report["summary"]["consistent"]:
        print(
            f"\n⚠️  INCONSISTENCIES FOUND: {report['summary']['functions']['mismatched']} "
            f"mismatched, {report['summary']['functions']['missing_in_verus']} missing",
            file=sys.stderr,
        )
        return 1

    print("\n✅ All exec functions consistent.", file=sys.stderr)
    return 0


if __name__ == "__main__":
    sys.exit(main())
