#!/usr/bin/env python3
# Copyright(c) The Maintainers of Nanvix.
# Licensed under the MIT License.

"""
Translate verus!{} macro-style Verus code to annotation-style
(#[verus_spec], proof!{}, proof_decl!{}, proof_with!{}).

Uses tree-sitter-verus for robust parsing.

Dependencies:
    pip install tree-sitter
    git clone https://github.com/q5438722/tree-sitter-verus.git
    cd tree-sitter-verus && git checkout dev && pip install -e .

Usage:
    python3 translate_verus_to_annotation.py input.rs [-o output.rs]
    python3 translate_verus_to_annotation.py input.rs --in-place

See TRANSLATOR.md for full documentation.
"""

import argparse
import re
import sys
from dataclasses import dataclass, field
from typing import Optional, List, Tuple

import tree_sitter_verus as tsv
from tree_sitter import Language, Parser, Node

# ==============================================================================
# Parser helpers
# ==============================================================================

def make_parser():
    lang = Language(tsv.language())
    return Parser(lang), lang

def nt(node: Node, src: bytes) -> str:
    """Get text of a tree-sitter node."""
    return src[node.start_byte:node.end_byte].decode("utf-8")

def fc(node: Node, typ: str) -> Optional[Node]:
    """Find first child of given type."""
    for c in node.children:
        if c.type == typ:
            return c
    return None

def fcs(node: Node, typ: str) -> List[Node]:
    """Find all children of given type."""
    return [c for c in node.children if c.type == typ]

def indent_at(src: bytes, pos: int) -> str:
    """Get whitespace indentation at a byte position."""
    ls = src.rfind(b"\n", 0, pos)
    ls = 0 if ls == -1 else ls + 1
    ind = []
    for i in range(ls, min(pos, len(src))):
        ch = src[i:i+1]
        if ch in (b" ", b"\t"):
            ind.append(ch.decode())
        else:
            break
    return "".join(ind)

def is_exec_fn(fn_node: Node) -> bool:
    """A function is exec if it has no mode keyword or mode == exec."""
    for c in fn_node.children:
        if c.type == "function_mode":
            return c.children[0].type == "exec" if c.children else False
    return True

def is_spec_or_proof_fn(fn_node: Node) -> bool:
    for c in fn_node.children:
        if c.type == "function_mode":
            if c.children:
                return c.children[0].type in ("spec", "proof")
    return False


# Clause types NOT supported in #[verus_spec] on loops (must stay in verus!{})
_UNSUPPORTED_LOOP_CLAUSES = {
    "invariant_except_break_clause",
    "invariant_ensures_clause",
    "ensures_clause",  # loop-level ensures not supported in verus_spec attr
}


def _has_unsupported_loop_clauses(node: Node) -> bool:
    """Recursively check if any loop in the subtree has unsupported clauses."""
    if node.type in ("for_expression", "while_expression", "loop_expression"):
        for c in node.children:
            if c.type in _UNSUPPORTED_LOOP_CLAUSES:
                return True
    for c in node.children:
        if c.type in ("function_item", "proof_block"):
            continue  # Don't cross function/proof boundaries
        if _has_unsupported_loop_clauses(c):
            return True
    return False


def _has_tracked_params_or_return(fn_node: Node, src: bytes) -> bool:
    """Check if a function has Tracked/Ghost params or return elements.

    Functions with Tracked params/returns must stay in verus!{} because:
    1. Tracked outputs: proof_with!(|= ...) doesn't handle early returns
    2. Tracked inputs: callers in verus!{} blocks still use old call convention
    """
    # Check parameters
    params = fc(fn_node, "parameters")
    if params:
        for p in params.children:
            if p.type == "parameter":
                pat = fc(p, "tuple_struct_pattern")
                if pat:
                    pat_text = nt(pat, src)
                    if pat_text.startswith("Tracked(") or pat_text.startswith("Ghost("):
                        return True
    # Check return type
    named_ret = fc(fn_node, "named_return_type")
    if named_ret:
        ret_text = nt(named_ret, src)
        if "Tracked<" in ret_text or "Ghost<" in ret_text:
            return True
    return False

# ==============================================================================
# Data structures
# ==============================================================================

@dataclass
class TrackedParam:
    pattern: str    # e.g. "Tracked(mem)"
    type_text: str  # e.g. "Tracked<PointsToRaw>"

@dataclass
class TrackedOutput:
    name: str       # e.g. "perms" (derived from ensures match pattern)
    type_text: str  # e.g. "Tracked<SlabPerms>"

# ==============================================================================
# Main translator
# ==============================================================================

class VerusTranslator:
    def __init__(self, source_text: str):
        self.source_text = source_text
        self.src = source_text.encode("utf-8")
        self.parser, self.language = make_parser()
        self.tree = self.parser.parse(self.src)
        self.root = self.tree.root_node

    # ------------------------------------------------------------------
    # Top level
    # ------------------------------------------------------------------
    def translate(self) -> str:
        vblocks = [c for c in self.root.children if c.type == "verus_block"]
        if not vblocks:
            return self.source_text

        replacements: List[Tuple[int, int, str]] = []
        for vb in vblocks:
            repl = self._process_verus_block(vb)
            replacements.append((vb.start_byte, vb.end_byte, repl))

        result = bytearray(self.src)
        for start, end, repl in reversed(replacements):
            result[start:end] = repl.encode("utf-8")

        output = result.decode("utf-8")

        # Add proc_macro_hygiene feature flag (gated on verus_keep_ghost) for
        # loop-level #[verus_spec] annotations, following SVSM pattern.
        feat = '#![cfg_attr(verus_keep_ghost, feature(proc_macro_hygiene))]'
        if "cfg_attr(verus_keep_ghost_body, verus_spec(" in output and feat not in output:
            lines = output.split("\n")
            # Find the last #![...] crate attribute at the TOP of the file
            # (stop at first non-comment, non-attribute, non-blank line)
            insert_idx = 0
            for i, line in enumerate(lines):
                stripped = line.strip()
                if stripped.startswith("#!["):
                    insert_idx = i + 1
                elif stripped == "" or stripped.startswith("//"):
                    continue
                elif insert_idx > 0:
                    break  # Past the header section
            if insert_idx == 0:
                insert_idx = 2  # Fallback: after copyright header
            lines.insert(insert_idx, feat)
            output = "\n".join(lines)

        # Clean excessive blank lines
        output = re.sub(r"\n{4,}", "\n\n\n", output)
        return output

    def _add_verus_stub_import(self, text: str) -> str:
        lines = text.split("\n")
        # Find the end of the import section: look for standalone use ... ; lines
        # that are at the top level (not indented)
        last_use_end = -1
        for i, line in enumerate(lines):
            stripped = line.strip()
            # Only consider top-level use statements (not indented)
            if line == stripped or line.startswith("use ") or line.startswith("pub use "):
                if stripped.startswith("use ") and stripped.endswith(";"):
                    last_use_end = i
                elif stripped.startswith("pub use ") and stripped.endswith(";"):
                    last_use_end = i
            # Multi-line use block ending at top level
            if stripped == "};" and not line.startswith("    ") and not line.startswith("\t"):
                last_use_end = i
        if last_use_end >= 0:
            lines.insert(last_use_end + 1, "use verus_stub::*;")
        return "\n".join(lines)

    # ------------------------------------------------------------------
    # Process one verus!{} block
    # ------------------------------------------------------------------
    def _process_verus_block(self, vblock: Node) -> str:
        src = self.src
        base_indent = indent_at(src, vblock.start_byte)

        # Collect items
        items = []
        pending_comments: List[Node] = []
        for child in vblock.children:
            if child.type in ("verus", "!", "{", "}"):
                continue
            if child.type in ("line_comment", "block_comment"):
                pending_comments.append(child)
                continue
            if child.type == "empty_statement":
                continue
            kind, attrs, inner = self._classify(child)
            items.append({
                "kind": kind,
                "node": child,
                "inner": inner,
                "attrs": attrs,
                "comments": pending_comments[:],
            })
            pending_comments = []

        # Separate exec and ghost items
        exec_parts = []
        ghost_parts = []

        for item in items:
            k = item["kind"]
            if k in ("struct", "enum"):
                exec_parts.append(item)
            elif k == "impl_with_exec":
                exec_parts.append(item)
            elif k == "impl_mixed":
                # Split: ghost methods stay, exec methods come out
                ghost_impl, exec_impl = self._split_mixed_impl(item)
                if ghost_impl:
                    ghost_parts.append(ghost_impl)
                if exec_impl:
                    exec_parts.append(exec_impl)
            elif k == "exec_fn":
                # Check if function has unsupported features for annotation style
                if item["inner"] and (
                    _has_unsupported_loop_clauses(item["inner"]) or
                    _has_tracked_params_or_return(item["inner"], self.src)
                ):
                    ghost_parts.append(item)
                else:
                    exec_parts.append(item)
            else:
                ghost_parts.append(item)

        # Build output sections
        sections = []

        for item in exec_parts:
            k = item["kind"]
            cmt = self._comments_text(item.get("comments", []))
            if k in ("struct", "enum"):
                sections.append(cmt + self._transform_struct(item))
            elif k in ("impl_with_exec", "exec_impl"):
                sections.append(cmt + self._transform_exec_impl(item))
            elif k == "exec_fn":
                sections.append(cmt + self._transform_exec_fn_item(item, base_indent))
            else:
                sections.append(cmt + nt(item["node"], src))

        if ghost_parts:
            body_parts = []
            for item in ghost_parts:
                cmt = self._comments_text(item.get("comments", []))
                body_parts.append(cmt + nt(item["node"], src))
            ghost_body = "\n\n".join(body_parts)
            sections.append(
                f"{base_indent}verus! {{\n\n{ghost_body}\n\n{base_indent}}}"
            )

        return "\n\n".join(sections)

    # ------------------------------------------------------------------
    # Classification
    # ------------------------------------------------------------------
    def _classify(self, node: Node) -> Tuple[str, List[Node], Optional[Node]]:
        """Returns (kind, attrs, inner_node)."""
        src = self.src
        if node.type != "declaration_with_attrs":
            return ("other", [], node)

        attrs = []
        inner = None
        for c in node.children:
            if c.type == "attribute_item":
                attrs.append(c)
            elif c.type not in ("line_comment", "block_comment"):
                if inner is None:
                    inner = c

        if inner is None:
            return ("other", attrs, None)

        if inner.type == "struct_item":
            return ("struct", attrs, inner)
        if inner.type == "enum_item":
            return ("enum", attrs, inner)
        if inner.type == "type_item":
            return ("type_alias", attrs, inner)

        if inner.type == "impl_item":
            # impl Trait for Type?
            for c in inner.children:
                if c.type == "for":
                    return ("impl_trait", attrs, inner)

            # Check methods
            dl = fc(inner, "declaration_list")
            has_exec = has_spec = False
            if dl:
                for sub in dl.children:
                    if sub.type == "declaration_with_attrs":
                        fn = fc(sub, "function_item")
                        if fn:
                            if is_exec_fn(fn):
                                has_exec = True
                            if is_spec_or_proof_fn(fn):
                                has_spec = True
            if has_exec and has_spec:
                return ("impl_mixed", attrs, inner)
            if has_exec:
                return ("impl_with_exec", attrs, inner)
            if has_spec:
                return ("impl_spec", attrs, inner)
            return ("impl_other", attrs, inner)

        if inner.type == "function_item":
            if is_exec_fn(inner):
                return ("exec_fn", attrs, inner)
            return ("spec_or_proof_fn", attrs, inner)

        return ("other", attrs, inner)

    # ------------------------------------------------------------------
    # Comment formatting
    # ------------------------------------------------------------------
    def _comments_text(self, nodes: List[Node]) -> str:
        if not nodes:
            return ""
        return "\n".join(nt(c, self.src) for c in nodes) + "\n"

    # ------------------------------------------------------------------
    # Struct transformation
    # ------------------------------------------------------------------
    def _transform_struct(self, item: dict) -> str:
        src = self.src
        inner = item["inner"]
        ind = indent_at(src, item["node"].start_byte)

        parts = []
        for attr in item["attrs"]:
            atxt = nt(attr, src)
            if "verifier::external_derive" in atxt:
                continue  # Replaced by #[verus_verify]
            if "verifier::ext_equal" in atxt:
                parts.append(f"{ind}#[cfg_attr(verus_keep_ghost, verifier::ext_equal)]")
                continue
            parts.append(f"{ind}{atxt}")

        parts.append(f"{ind}#[verus_verify]")
        parts.append(f"{ind}{nt(inner, src)}")
        return "\n".join(parts)

    # ------------------------------------------------------------------
    # Split mixed impl
    # ------------------------------------------------------------------
    def _split_mixed_impl(self, item: dict) -> Tuple[Optional[dict], Optional[dict]]:
        """Split impl with both exec and spec/proof methods."""
        # Return (ghost_item, exec_item) - each is a dict like item but with
        # 'kind' set to the appropriate value.
        # For now, just keep the whole thing as exec_impl and annotate methods
        # individually.
        ghost_item = {**item, "kind": "impl_mixed_ghost"}
        exec_item = {**item, "kind": "impl_with_exec"}
        return ghost_item, exec_item

    # ------------------------------------------------------------------
    # Impl block transformation
    # ------------------------------------------------------------------
    def _transform_exec_impl(self, item: dict) -> str:
        src = self.src
        inner = item["inner"]  # impl_item node
        ind = indent_at(src, item["node"].start_byte)

        # Get impl header: "impl Foo" or "impl<T> Foo<T>"
        dl = fc(inner, "declaration_list")
        if dl is None:
            return nt(item["node"], src)

        header_parts = []
        for c in inner.children:
            if c.type == "declaration_list":
                break
            header_parts.append(nt(c, src))
        header = " ".join(header_parts)

        # Process methods
        method_indent = ind + "    "
        methods = []
        ghost_methods = []
        for child in dl.children:
            if child.type != "declaration_with_attrs":
                continue
            fn_item = fc(child, "function_item")
            if fn_item is None:
                # type alias or other
                ghost_methods.append(nt(child, src))
                continue
            if is_exec_fn(fn_item):
                # Check if function has unsupported features for annotation style
                if _has_unsupported_loop_clauses(fn_item) or \
                   _has_tracked_params_or_return(fn_item, src):
                    ghost_methods.append(nt(child, src))
                else:
                    sub_attrs = [c for c in child.children if c.type == "attribute_item"]
                    methods.append(self._transform_exec_fn(fn_item, sub_attrs, method_indent))
            else:
                ghost_methods.append(nt(child, src))

        # Build output
        parts = []
        # Attributes on the impl
        for attr in item.get("attrs", []):
            atxt = nt(attr, src)
            if "cfg(verus_keep_ghost)" in atxt:
                continue  # Don't gate exec impl with verus_keep_ghost
            parts.append(f"{ind}{atxt}")

        # Ghost methods in residual verus!{} impl
        if ghost_methods:
            parts.append(f"{ind}verus! {{")
            parts.append(f"{ind}{header} {{")
            for gm in ghost_methods:
                parts.append(gm)
            parts.append(f"{ind}}}")
            parts.append(f"{ind}}} // verus!")
            parts.append("")

        # Exec methods — need #[verus_verify] on the impl block
        if methods:
            parts.append(f"{ind}#[verus_verify]")
            parts.append(f"{ind}{header} {{")
            for m in methods:
                parts.append(m)
            parts.append(f"{ind}}}")

        return "\n".join(parts)

    # ------------------------------------------------------------------
    # Single function transformation
    # ------------------------------------------------------------------
    def _transform_exec_fn_item(self, item: dict, base_indent: str) -> str:
        """Transform a top-level exec function (not inside impl)."""
        fn_node = item["inner"]
        attrs = item["attrs"]
        return self._transform_exec_fn(fn_node, attrs, base_indent)

    def _transform_exec_fn(self, fn_node: Node, attrs: List[Node], ind: str) -> str:
        """Transform one exec function to annotation style."""
        src = self.src
        fn_ind = indent_at(src, fn_node.start_byte)

        # 1. Extract function properties
        vis = ""
        v = fc(fn_node, "visibility_modifier")
        if v:
            vis = nt(v, src) + " "

        is_unsafe = any(c.type == "unsafe" or
                        (c.type == "function_modifiers" and "unsafe" in nt(c, src))
                        for c in fn_node.children)

        name_node = fc(fn_node, "identifier")
        name = nt(name_node, src) if name_node else ""

        generics_node = fc(fn_node, "type_parameters")
        generics = nt(generics_node, src) if generics_node else ""

        # 2. Extract parameters
        self_param, normal_params, tracked_params = self._extract_params(fn_node)

        # 3. Extract return type info — do NOT extract tracked outputs from return type.
        # The `with -> tracked_output` pattern is not well-supported in current verus
        # versions. Keep the original return type including Tracked elements.
        ret_name, exec_ret_type, _unused_outputs, _ = self._extract_return_info(fn_node)
        tracked_outputs = []  # Don't use tracked output extraction

        # If return type has tracked elements, use the ORIGINAL return type
        # (not the stripped one) to maintain API compatibility
        if _unused_outputs:
            # Keep full return type with Tracked elements
            named_ret = fc(fn_node, "named_return_type")
            if named_ret:
                past_colon = False
                type_parts = []
                for c in named_ret.children:
                    if c.type == ":":
                        past_colon = True
                        continue
                    if past_colon and c.type != ")":
                        type_parts.append(nt(c, src))
                exec_ret_type = " ".join(type_parts).strip() if type_parts else exec_ret_type

        # 4. Extract specifications
        requires, ensures, decreases = self._extract_specs(fn_node)

        # 5. No tracked output name resolution needed (outputs kept in return type)

        # 6. Build #[verus_spec(...)] attribute
        spec_attr = self._build_verus_spec_attr(
            ret_name, requires, ensures, decreases,
            tracked_params, tracked_outputs, fn_ind
        )

        # 7. Build function signature
        sig = self._build_fn_sig(vis, is_unsafe, name, generics,
                                  self_param, normal_params, exec_ret_type)

        # 8. Transform body
        body_node = fc(fn_node, "block")
        if body_node:
            body = self._transform_body(body_node, tracked_outputs)
        else:
            body = "{ }"

        # 9. Assemble output
        lines = []
        # Original attributes (except verifier:: ones already handled)
        for attr in attrs:
            atxt = nt(attr, src)
            if "verifier::external_body" in atxt:
                lines.append(f"{fn_ind}#[verus_verify(external_body)]")
            elif "verifier::external_derive" in atxt:
                continue
            elif "verifier::" in atxt:
                lines.append(f"{fn_ind}{atxt}")
            else:
                lines.append(f"{fn_ind}{atxt}")

        if spec_attr:
            lines.append(f"{fn_ind}{spec_attr}")

        lines.append(f"{fn_ind}{sig} {body}")
        return "\n".join(lines)

    # ------------------------------------------------------------------
    # Parameter extraction
    # ------------------------------------------------------------------
    def _extract_params(self, fn_node: Node):
        src = self.src
        params = fc(fn_node, "parameters")
        if params is None:
            return None, [], []

        self_param = None
        normal = []
        tracked = []

        for c in params.children:
            if c.type == "self_parameter":
                self_param = nt(c, src)
            elif c.type == "parameter":
                pat = fc(c, "tuple_struct_pattern")
                if pat:
                    pat_text = nt(pat, src)
                    if pat_text.startswith("Tracked(") or pat_text.startswith("Ghost("):
                        # Extract type text
                        type_text = self._param_type_text(c, pat)
                        tracked.append(TrackedParam(
                            pattern=pat_text,
                            type_text=type_text
                        ))
                        continue
                normal.append(nt(c, src))

        return self_param, normal, tracked

    def _param_type_text(self, param_node: Node, pat_node: Node) -> str:
        """Extract the type text from a parameter, skipping the pattern and colon."""
        src = self.src
        # Find everything after the ':' in the parameter
        past_colon = False
        parts = []
        for c in param_node.children:
            if c.type == ":":
                past_colon = True
                continue
            if past_colon:
                parts.append(nt(c, src))
        return " ".join(parts).strip()

    # ------------------------------------------------------------------
    # Return type extraction
    # ------------------------------------------------------------------
    def _extract_return_info(self, fn_node: Node):
        """Extract return type info.

        Returns: (ret_name, exec_ret_type, tracked_outputs, original_ensures)
        """
        src = self.src
        named_ret = fc(fn_node, "named_return_type")

        if named_ret is None:
            return None, None, [], None

        # Get named return variable
        ident = fc(named_ret, "identifier")
        ret_name = nt(ident, src) if ident else None

        # Get the type - look for direct tuple or generic type
        tuple_type = fc(named_ret, "tuple_type")
        if tuple_type:
            # Direct tuple return type: (T, Tracked<U>)
            return self._parse_tuple_return(tuple_type, ret_name)

        # Check for generic type like Result<(T, Tracked<U>), E>
        generic_type = fc(named_ret, "generic_type")
        if generic_type:
            return self._parse_generic_return(generic_type, ret_name, named_ret)

        # Simple type (not tuple, not generic with tracked)
        past_colon = False
        type_parts = []
        for c in named_ret.children:
            if c.type == ":":
                past_colon = True
                continue
            if past_colon and c.type != ")":
                type_parts.append(nt(c, src))

        exec_ret = " ".join(type_parts).strip() if type_parts else None
        return ret_name, exec_ret, [], None

    def _parse_tuple_return(self, tuple_type: Node, ret_name):
        """Parse a direct tuple return type for tracked elements."""
        src = self.src
        elements = []
        tracked_types = []
        for c in tuple_type.children:
            if c.type in ("(", ")", ","):
                continue
            elem_text = nt(c, src)
            if c.type == "generic_type":
                tid = fc(c, "type_identifier")
                if tid and nt(tid, src) in ("Tracked", "Ghost"):
                    tracked_types.append(TrackedOutput(name="", type_text=elem_text))
                    continue
            elements.append(elem_text)

        if tracked_types:
            if len(elements) == 1:
                exec_ret = elements[0]
            elif len(elements) == 0:
                exec_ret = "()"
            else:
                exec_ret = "(" + ", ".join(elements) + ")"
            return ret_name, exec_ret, tracked_types, None
        else:
            exec_ret = nt(tuple_type, src)
            return ret_name, exec_ret, [], None

    def _parse_generic_return(self, generic_type: Node, ret_name, named_ret: Node):
        """Parse generic return type like Result<(T, Tracked<U>), E>."""
        src = self.src
        tid = fc(generic_type, "type_identifier")
        if tid is None:
            exec_ret = nt(generic_type, src)
            return ret_name, exec_ret, [], None

        wrapper_name = nt(tid, src)  # "Result", "Option", etc.
        targs = fc(generic_type, "type_arguments")
        if targs is None:
            exec_ret = nt(generic_type, src)
            return ret_name, exec_ret, [], None

        # Look for a tuple type inside the type arguments that contains Tracked
        tuple_in_args = fc(targs, "tuple_type")
        if tuple_in_args is None:
            exec_ret = nt(generic_type, src)
            return ret_name, exec_ret, [], None

        # Parse the tuple for tracked elements
        elements = []
        tracked_types = []
        for c in tuple_in_args.children:
            if c.type in ("(", ")", ","):
                continue
            elem_text = nt(c, src)
            if c.type == "generic_type":
                inner_tid = fc(c, "type_identifier")
                if inner_tid and nt(inner_tid, src) in ("Tracked", "Ghost"):
                    tracked_types.append(TrackedOutput(name="", type_text=elem_text))
                    continue
            elements.append(elem_text)

        if not tracked_types:
            exec_ret = nt(generic_type, src)
            return ret_name, exec_ret, [], None

        # Build the new type without tracked elements
        if len(elements) == 1:
            ok_type = elements[0]
        elif len(elements) == 0:
            ok_type = "()"
        else:
            ok_type = "(" + ", ".join(elements) + ")"

        # Collect the other type arguments (e.g., Error type in Result<_, Error>)
        other_args = []
        past_tuple = False
        for c in targs.children:
            if c.type in ("<", ">"):
                continue
            if c is tuple_in_args:
                past_tuple = True
                continue
            if c.type == "," and not past_tuple:
                continue
            if c.type == ",":
                continue
            if past_tuple:
                other_args.append(nt(c, src))

        if other_args:
            exec_ret = f"{wrapper_name}<{ok_type}, {', '.join(other_args)}>"
        else:
            exec_ret = f"{wrapper_name}<{ok_type}>"

        return ret_name, exec_ret, tracked_types, None

    # ------------------------------------------------------------------
    # Handle Result<(T, Tracked<U>), E> return types
    # ------------------------------------------------------------------
    def _check_result_tracked_return(self, fn_node: Node):
        """Check if return type is Result<(T, Tracked<U>), E> and handle it."""
        src = self.src
        named_ret = fc(fn_node, "named_return_type")
        if named_ret is None:
            return None, None, []

        ret_text = nt(named_ret, src)
        ident = fc(named_ret, "identifier")
        ret_name = nt(ident, src) if ident else None

        # Look for Result<(T, Tracked<U>), E> pattern
        # This is a generic_type with type_identifier "Result"
        # and type_arguments containing a tuple_type
        for c in named_ret.children:
            if c.type == "generic_type":
                tid = fc(c, "type_identifier")
                if tid and nt(tid, src) == "Result":
                    targs = fc(c, "type_arguments")
                    if targs:
                        return self._parse_result_type(targs, ret_name)

        return ret_name, nt(named_ret, src), []

    def _parse_result_type(self, targs: Node, ret_name):
        """Parse Result type arguments for tracked elements."""
        src = self.src
        # Find tuple type inside type_arguments
        tuple_t = fc(targs, "tuple_type")
        if tuple_t is None:
            return ret_name, None, []

        elements = []
        tracked_outputs = []
        for c in tuple_t.children:
            if c.type in ("(", ")", ","):
                continue
            elem_text = nt(c, src)
            if c.type == "generic_type":
                tid = fc(c, "type_identifier")
                if tid and nt(tid, src) in ("Tracked", "Ghost"):
                    tracked_outputs.append(TrackedOutput(name="", type_text=elem_text))
                    continue
            elements.append(elem_text)

        if not tracked_outputs:
            return ret_name, None, []

        # Build the new Result type without tracked elements
        err_types = []
        past_comma = False
        depth = 0
        # We need the error type. Parse targs text.
        # Actually let's collect all top-level children of targs
        targs_children = [c for c in targs.children if c.type not in ("<", ">", ",")]
        # First is the ok type (tuple), rest are error types
        if len(targs_children) >= 2:
            err_type = nt(targs_children[1], src)
        else:
            err_type = "()"

        if len(elements) == 1:
            ok_type = elements[0]
        elif len(elements) == 0:
            ok_type = "()"
        else:
            ok_type = "(" + ", ".join(elements) + ")"

        exec_ret = f"Result<{ok_type}, {err_type}>"
        return ret_name, exec_ret, tracked_outputs

    # ------------------------------------------------------------------
    # Spec extraction
    # ------------------------------------------------------------------
    def _extract_specs(self, fn_node: Node):
        """Extract requires, ensures, decreases from fn_qualifier."""
        src = self.src
        qual = fc(fn_node, "fn_qualifier")
        if qual is None:
            return None, None, None

        requires = ensures = decreases = None
        for c in qual.children:
            if c.type == "requires_clause":
                txt = nt(c, src)
                requires = txt[len("requires"):].strip().rstrip(",").rstrip()
            elif c.type == "ensures_clause":
                txt = nt(c, src)
                ensures = txt[len("ensures"):].strip().rstrip(",").rstrip()
            elif c.type == "decreases_clause":
                txt = nt(c, src)
                decreases = txt[len("decreases"):].strip().rstrip(",").rstrip()

        return requires, ensures, decreases

    # ------------------------------------------------------------------
    # Resolve tracked output names from ensures patterns
    # ------------------------------------------------------------------
    def _resolve_tracked_output_names(self, tracked_outputs, ensures_text, ret_name):
        """Derive tracked output variable names from ensures match patterns.

        E.g., if ensures has Ok((slab, perms)) and the 2nd position is tracked,
        then the tracked output gets name 'perms' and the pattern becomes Ok(slab).
        """
        if not tracked_outputs:
            return tracked_outputs, ensures_text

        # Find Ok((...)) patterns in ensures
        # Pattern: Ok((var1, var2, ...))
        pattern = r'Ok\(\(([^)]*?(?:\([^)]*\))*[^)]*?)\)\)'

        def replace_ok_tuple(m):
            inner = m.group(1)
            # Split by commas at top level
            elems = self._split_top_level(inner)
            if len(elems) <= 1:
                return m.group(0)  # Don't modify single-element

            # Remove tracked elements (from the end, matching tracked_outputs count)
            n_tracked = len(tracked_outputs)
            if n_tracked >= len(elems):
                return m.group(0)  # Shouldn't happen

            # Assume tracked elements are at the end of the tuple
            exec_elems = elems[:len(elems) - n_tracked]
            tracked_elems = elems[len(elems) - n_tracked:]

            # Set tracked output names
            for i, te in enumerate(tracked_elems):
                name = te.strip()
                if i < len(tracked_outputs):
                    tracked_outputs[i].name = name

            if len(exec_elems) == 1:
                return f"Ok({exec_elems[0].strip()})"
            else:
                return f"Ok(({', '.join(e.strip() for e in exec_elems)}))"

        new_ensures = re.sub(pattern, replace_ok_tuple, ensures_text)

        # If tracked outputs still have no name, use a default
        for i, to in enumerate(tracked_outputs):
            if not to.name:
                to.name = f"tracked_out_{i}"

        return tracked_outputs, new_ensures

    def _split_top_level(self, text: str) -> List[str]:
        """Split text by commas at the top nesting level."""
        depth = 0
        parts = []
        current = []
        for ch in text:
            if ch in ("(", "<", "[", "{"):
                depth += 1
            elif ch in (")", ">", "]", "}"):
                depth -= 1
            if ch == "," and depth == 0:
                parts.append("".join(current))
                current = []
            else:
                current.append(ch)
        if current:
            parts.append("".join(current))
        return parts

    # ------------------------------------------------------------------
    # Build #[verus_spec(...)] attribute
    # ------------------------------------------------------------------
    def _build_verus_spec_attr(self, ret_name, requires, ensures, decreases,
                                tracked_params, tracked_outputs, ind):
        has_specs = requires or ensures or decreases
        has_with = tracked_params or tracked_outputs

        if not has_specs and not has_with:
            return None

        if not has_specs and not has_with:
            return "#[verus_spec]"

        ii = ind + "    "  # inner indent

        content_parts = []

        # With clause
        if has_with:
            with_lines = ["with"]
            for i_tp, tp in enumerate(tracked_params):
                is_last_input = (i_tp == len(tracked_params) - 1)
                trailing = "," if not is_last_input or tracked_outputs else ""
                with_lines.append(f"{ii}    {tp.pattern}: {tp.type_text}{trailing}")
            if tracked_outputs:
                out_parts = []
                for to in tracked_outputs:
                    out_parts.append(f"{to.name}: {to.type_text}")
                with_lines.append(f"{ii}        -> " + ", ".join(out_parts))
            content_parts.append("\n".join(with_lines))

        if requires:
            content_parts.append(f"requires\n{ii}    {requires}")
        if ensures:
            content_parts.append(f"ensures\n{ii}    {ensures}")
        if decreases:
            content_parts.append(f"decreases\n{ii}    {decreases}")

        if not content_parts:
            return "#[verus_spec]"

        # Build the attribute
        body = ("\n" + ii).join(content_parts)

        if ret_name:
            header = f"{ret_name} =>\n{ii}"
        else:
            header = ""

        return f"#[verus_spec({header}{body}\n{ind})]"

    # ------------------------------------------------------------------
    # Build clean function signature
    # ------------------------------------------------------------------
    def _build_fn_sig(self, vis, is_unsafe, name, generics,
                       self_param, normal_params, exec_ret_type):
        parts = []
        if vis:
            parts.append(vis.strip())
        if is_unsafe:
            parts.append("unsafe")
        parts.append("fn")
        parts.append(name + generics)

        all_params = []
        if self_param:
            all_params.append(self_param)
        all_params.extend(normal_params)
        parts[-1] += "(" + ", ".join(all_params) + ")"

        if exec_ret_type:
            parts.append(f"-> {exec_ret_type}")

        return " ".join(parts)

    # ------------------------------------------------------------------
    # Body transformation
    # ------------------------------------------------------------------
    def _transform_body(self, body_node: Node, tracked_outputs: List[TrackedOutput]) -> str:
        """Transform function body: proof→proof!, ghost/tracked let→proof_decl!, etc."""
        src = self.src
        body_bytes = src[body_node.start_byte:body_node.end_byte]
        offset = body_node.start_byte

        transforms: List[Tuple[int, int, str]] = []
        self._collect_body_transforms(body_node, transforms, offset)

        # Handle tracked return expressions if there are tracked outputs
        if tracked_outputs:
            self._transform_tracked_returns(body_node, transforms, offset, tracked_outputs)

        # Apply transforms in reverse order
        result = bytearray(body_bytes)
        transforms.sort(key=lambda t: t[0], reverse=True)
        for start, end, replacement in transforms:
            result[start:end] = replacement.encode("utf-8")

        return result.decode("utf-8")

    def _collect_body_transforms(self, node: Node, transforms: list, offset: int):
        """Recursively collect body transformations."""
        src = self.src

        if node.type == "proof_block":
            # proof { ... } → proof! { ... }
            proof_kw = fc(node, "proof")
            if proof_kw:
                transforms.append((
                    proof_kw.start_byte - offset,
                    proof_kw.end_byte - offset,
                    "proof!"
                ))
            return  # Don't recurse into proof blocks

        if node.type == "declaration_with_attrs":
            let_decl = fc(node, "let_declaration")
            if let_decl:
                has_ghost = any(c.type == "ghost" for c in let_decl.children)
                has_tracked = any(c.type == "tracked" for c in let_decl.children)
                if has_ghost or has_tracked:
                    let_text = nt(let_decl, src)
                    transforms.append((
                        node.start_byte - offset,
                        node.end_byte - offset,
                        f"proof_decl! {{ {let_text} }}"
                    ))
                    return

        # Handle loops with invariants
        if node.type in ("for_expression", "while_expression", "loop_expression"):
            clauses = self._extract_loop_clauses(node)
            if clauses:
                self._transform_loop(node, clauses, transforms, offset)
                return

        # Recurse
        for c in node.children:
            self._collect_body_transforms(c, transforms, offset)

    def _extract_loop_clauses(self, loop_node: Node) -> List[Tuple[str, str, Node]]:
        """Extract loop annotation clauses (invariant, decreases, etc.)."""
        src = self.src
        clause_types = {
            "invariant_clause": "invariant",
            "invariant_except_break_clause": "invariant_except_break",
            "invariant_ensures_clause": "invariant_ensures",
            "ensures_clause": "ensures",
            "decreases_clause": "decreases",
        }
        clauses = []
        for c in loop_node.children:
            if c.type in clause_types:
                kind = clause_types[c.type]
                full_text = nt(c, src)
                # Extract body (after keyword), strip trailing comma
                body = full_text[len(kind):].strip().rstrip(",").rstrip()
                clauses.append((kind, body, c))
        return clauses

    def _transform_loop(self, loop_node: Node, clauses, transforms, offset):
        """Add loop annotation before loop and remove clauses.

        Uses #[cfg_attr(verus_keep_ghost_body, verus_spec(...))] so the
        annotation is only active during verus verification builds.
        """
        src = self.src
        ind = indent_at(src, loop_node.start_byte)

        # Build annotation content — each clause on its own line(s)
        spec_lines = []
        for i_clause, (kind, body, _) in enumerate(clauses):
            # Indent the body relative to the clause keyword
            body_lines = body.split("\n")
            # First line stays with keyword
            first_line = f"{kind} {body_lines[0].strip()}" if body_lines else kind
            remaining = "\n".join(body_lines[1:]) if len(body_lines) > 1 else ""
            if remaining:
                # Add comma at end of clause (before next clause keyword)
                clause_text = first_line + "\n" + remaining
            else:
                clause_text = first_line
            # Add trailing comma between clauses
            if i_clause < len(clauses) - 1:
                clause_text = clause_text.rstrip(",") + ","
            spec_lines.append(clause_text)

        spec_body = f"\n{ind}    ".join(spec_lines)
        spec_attr = f"#[cfg_attr(verus_keep_ghost_body, verus_spec(\n{ind}    {spec_body}\n{ind}))]"

        # Insert annotation before the loop
        transforms.append((
            loop_node.start_byte - offset,
            loop_node.start_byte - offset,
            spec_attr + "\n" + ind
        ))

        # Remove all clauses as a contiguous range (including whitespace between them)
        # Find the byte range spanning all clauses
        first_start = min(c.start_byte for _, _, c in clauses)
        last_end = max(c.end_byte for _, _, c in clauses)
        # Also eat trailing comma after the last clause
        if last_end < len(src) and src[last_end:last_end+1] == b",":
            last_end += 1
        # Eat trailing whitespace up to the block opening brace
        while last_end < len(src) and src[last_end:last_end+1] in (b" ", b"\t", b"\n"):
            last_end += 1
        # Also remove leading whitespace (newlines between loop condition and clauses)
        while first_start > 0 and src[first_start-1:first_start] in (b" ", b"\t", b"\n"):
            first_start -= 1
        # Add back newline + indent so loop keyword connects with block brace
        transforms.append((first_start - offset, last_end - offset, "\n" + ind))

        # Recurse into the loop body
        block = fc(loop_node, "block")
        if block:
            self._collect_body_transforms(block, transforms, offset)

    def _transform_tracked_returns(self, body_node: Node, transforms: list,
                                    offset: int, tracked_outputs: List[TrackedOutput]):
        """Transform return expressions to use proof_with! for tracked outputs."""
        src = self.src

        # Find the last expression in the body (implicit return)
        # and any explicit return statements
        self._find_and_transform_returns(body_node, transforms, offset, tracked_outputs)

    def _find_and_transform_returns(self, node: Node, transforms: list,
                                     offset: int, tracked_outputs: List[TrackedOutput]):
        """Find return expressions with Tracked(...) and transform them."""
        src = self.src

        # Look for call expressions like Ok((x, Tracked(y))) at the end of blocks
        if node.type == "block":
            # The last non-} child might be the return expression
            children = [c for c in node.children if c.type not in ("{", "}")]
            if children:
                last = children[-1]
                self._try_transform_return_expr(last, transforms, offset, tracked_outputs)
            return

        # Look for explicit return statements
        if node.type == "expression_statement":
            for c in node.children:
                if c.type == "return_expression":
                    # Check if return value has Tracked
                    self._try_transform_return_expr(c, transforms, offset, tracked_outputs)
                    return

        # Recurse (but not into nested functions or proof blocks)
        if node.type in ("function_item", "proof_block", "closure_expression"):
            return
        for c in node.children:
            self._find_and_transform_returns(c, transforms, offset, tracked_outputs)

    def _try_transform_return_expr(self, expr_node: Node, transforms: list,
                                    offset: int, tracked_outputs: List[TrackedOutput]):
        """Try to transform a return expression that contains Tracked(...)."""
        src = self.src
        expr_text = nt(expr_node, src)

        if "Tracked(" not in expr_text:
            return

        ind = indent_at(src, expr_node.start_byte)

        # Pattern for Ok((items..., Tracked(var)))
        ok_pat = r'(Ok\()\((.+?),\s*Tracked\((\w+)\)\)(\))\s*$'
        m = re.search(ok_pat, expr_text, re.DOTALL)
        if m:
            inner = m.group(2).strip()
            tracked_var = m.group(3)
            new_expr = f"proof_with!(|= Tracked({tracked_var}));\n{ind}{m.group(1)}{inner}{m.group(4)}"
            if expr_node.type == "return_expression":
                ret_kw_end = expr_text.index("Ok")
                ret_prefix = expr_text[:ret_kw_end]
                new_expr = f"proof_with!(|= Tracked({tracked_var}));\n{ind}{ret_prefix}Ok({inner})"

            transforms.append((
                expr_node.start_byte - offset,
                expr_node.end_byte - offset,
                new_expr
            ))
            return

        # Pattern for bare tuple (x, Tracked(var))
        tuple_pat = r'\((.+?),\s*Tracked\((\w+)\)\)\s*$'
        m = re.search(tuple_pat, expr_text, re.DOTALL)
        if m:
            inner = m.group(1).strip()
            tracked_var = m.group(2)
            new_expr = f"proof_with!(|= Tracked({tracked_var}));\n{ind}{inner}"
            transforms.append((
                expr_node.start_byte - offset,
                expr_node.end_byte - offset,
                new_expr
            ))
            return


# ==============================================================================
# Main
# ==============================================================================

def main():
    ap = argparse.ArgumentParser(
        description="Translate verus!{} macro style to annotation style."
    )
    ap.add_argument("input", help="Input Rust file with verus!{} blocks")
    ap.add_argument("-o", "--output", help="Output file (default: stdout)")
    ap.add_argument("-i", "--in-place", action="store_true", help="Edit file in place")
    args = ap.parse_args()

    with open(args.input) as f:
        source = f.read()

    translator = VerusTranslator(source)
    result = translator.translate()

    if args.in_place:
        with open(args.input, "w") as f:
            f.write(result)
        print(f"Written in-place: {args.input}", file=sys.stderr)
    elif args.output:
        with open(args.output, "w") as f:
            f.write(result)
        print(f"Written to: {args.output}", file=sys.stderr)
    else:
        sys.stdout.write(result)


if __name__ == "__main__":
    main()
