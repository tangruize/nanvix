#!/usr/bin/env python3
# Copyright(c) The Maintainers of Nanvix.
# Licensed under the MIT License.

"""
Detect potential alias spec functions in Verus code.

An alias is a spec function whose body is just:
1. A method call: self.foo() or self@.foo()
2. A field access: self.field or self@.field
3. A constant reference: SOME_CONSTANT
4. A simple delegation: other_fn(args)

Usage:
    ./check_spec_aliases.py [file.rs ...]
    If no files specified, checks all .rs files in current directory.
"""

import re
import sys
from pathlib import Path
from dataclasses import dataclass
from typing import Optional


@dataclass
class SpecFn:
    """Represents a spec function."""
    name: str
    line_num: int
    params: str
    body: str
    file: str


def extract_spec_functions(file_path: Path) -> list[SpecFn]:
    """Extract all spec functions from a file."""
    content = file_path.read_text()
    lines = content.split('\n')
    
    spec_fns = []
    i = 0
    
    while i < len(lines):
        line = lines[i]
        
        # Match spec fn declarations
        match = re.match(
            r'\s*(pub\s+)?(open|closed)?\s*spec\s+fn\s+(\w+)\s*(<[^>]*>)?\s*\(([^)]*)\)',
            line
        )
        
        if match:
            fn_name = match.group(3)
            params = match.group(5) or ""
            start_line = i + 1
            
            # Find the function body
            # Look for opening brace
            body_start = i
            brace_count = 0
            found_open = False
            
            while body_start < len(lines):
                for char in lines[body_start]:
                    if char == '{':
                        brace_count += 1
                        found_open = True
                    elif char == '}':
                        brace_count -= 1
                
                if found_open and brace_count == 0:
                    break
                body_start += 1
            
            # Extract body content (between first { and last })
            body_lines = lines[i:body_start + 1]
            body_text = '\n'.join(body_lines)
            
            # Find content between { and }
            brace_start = body_text.find('{')
            brace_end = body_text.rfind('}')
            
            if brace_start != -1 and brace_end != -1:
                body = body_text[brace_start + 1:brace_end].strip()
                
                spec_fns.append(SpecFn(
                    name=fn_name,
                    line_num=start_line,
                    params=params,
                    body=body,
                    file=str(file_path)
                ))
            
            i = body_start + 1
        else:
            i += 1
    
    return spec_fns


def classify_alias(fn: SpecFn) -> Optional[tuple[str, str]]:
    """
    Classify if a function is an alias.
    
    Returns:
        (alias_type, target) if it's an alias, None otherwise.
    """
    body = fn.body.strip()
    
    # Remove trailing semicolons if any
    body = body.rstrip(';').strip()
    
    # Skip multi-statement bodies (contains &&& or multiple lines with statements)
    if '&&&' in body or body.count('\n') > 2:
        # Check if it's just a simple expression with newlines
        clean_body = ' '.join(body.split())
        if '&&&' in clean_body:
            return None
    
    # Normalize whitespace
    clean_body = ' '.join(body.split())
    
    # Pattern 1: self.method() or self@.method()
    match = re.match(r'^self(@)?\.(\w+)\(\s*\)$', clean_body)
    if match:
        view = "@" if match.group(1) else ""
        method = match.group(2)
        return ("method_delegate", f"self{view}.{method}()")
    
    # Pattern 2: self.method(same_params) - delegate with same params
    match = re.match(r'^self(@)?\.(\w+)\(([^)]+)\)$', clean_body)
    if match:
        view = "@" if match.group(1) else ""
        method = match.group(2)
        args = match.group(3).strip()
        # Check if args are just the function params
        fn_params = [p.strip().split(':')[0].strip() for p in fn.params.split(',') if p.strip() and ':' in p]
        call_args = [a.strip() for a in args.split(',')]
        
        # Simple check: if single arg matches a param name
        if len(call_args) == 1 and len(fn_params) >= 1:
            if call_args[0] in fn_params or call_args[0].replace(' as int', '') in fn_params:
                return ("param_delegate", f"self{view}.{method}({args})")
    
    # Pattern 3: self.field or self@.field (field access)
    match = re.match(r'^self(@)?\.(\w+)$', clean_body)
    if match:
        view = "@" if match.group(1) else ""
        field = match.group(2)
        return ("field_access", f"self{view}.{field}")
    
    # Pattern 4: self.field.method() (chained access)
    match = re.match(r'^self(@)?\.(\w+)\.(\w+)\(\s*\)$', clean_body)
    if match:
        view = "@" if match.group(1) else ""
        field = match.group(2)
        method = match.group(3)
        return ("chained_delegate", f"self{view}.{field}.{method}()")
    
    # Pattern 5: CONSTANT (all caps identifier)
    match = re.match(r'^([A-Z][A-Z0-9_]+)$', clean_body)
    if match:
        return ("constant", match.group(1))
    
    # Pattern 6: expr / CONSTANT or expr % CONSTANT (simple arithmetic)
    match = re.match(r'^self(@)?\.(\w+)\s*(as\s+int)?\s*[/%]\s*([A-Z][A-Z0-9_]+)\s*(as\s+int)?$', clean_body)
    if match:
        return ("computed", clean_body)
    
    # Pattern 7: Simple comparison or boolean expression with one comparison
    if re.match(r'^self(@)?\.(\w+)\s*(==|!=|<|>|<=|>=)\s*\d+$', clean_body):
        return ("simple_predicate", clean_body)
    
    # Pattern 8: other.method() where other is a parameter
    fn_params = [p.strip().split(':')[0].strip() for p in fn.params.split(',') if p.strip() and ':' in p]
    for param in fn_params:
        if re.match(rf'^{re.escape(param)}\.(\w+)\(\s*\)$', clean_body):
            return ("param_method_delegate", clean_body)
    
    return None


def analyze_file(file_path: Path) -> list[tuple[SpecFn, str, str]]:
    """Analyze a file for alias spec functions."""
    spec_fns = extract_spec_functions(file_path)
    aliases = []
    
    for fn in spec_fns:
        result = classify_alias(fn)
        if result:
            alias_type, target = result
            aliases.append((fn, alias_type, target))
    
    return aliases


def main():
    """Main entry point."""
    if len(sys.argv) > 1:
        files = [Path(f) for f in sys.argv[1:]]
    else:
        files = list(Path('.').glob('*.rs'))
    
    all_aliases = []
    
    for file_path in sorted(files):
        if not file_path.exists():
            print(f"Warning: {file_path} not found", file=sys.stderr)
            continue
        
        aliases = analyze_file(file_path)
        all_aliases.extend(aliases)
    
    if not all_aliases:
        print("No obvious alias spec functions found.")
        return
    
    # Group by type
    by_type: dict[str, list] = {}
    for fn, alias_type, target in all_aliases:
        if alias_type not in by_type:
            by_type[alias_type] = []
        by_type[alias_type].append((fn, target))
    
    print("=" * 80)
    print("POTENTIAL ALIAS SPEC FUNCTIONS")
    print("=" * 80)
    print()
    
    type_descriptions = {
        "method_delegate": "Method Delegates (self.foo() or self@.foo())",
        "param_delegate": "Parameter Delegates (self.foo(param))",
        "field_access": "Field Accessors (self.field)",
        "chained_delegate": "Chained Delegates (self.field.method())",
        "constant": "Constant Wrappers",
        "computed": "Simple Computations",
        "simple_predicate": "Simple Predicates",
        "param_method_delegate": "Parameter Method Delegates",
    }
    
    for alias_type, items in sorted(by_type.items()):
        desc = type_descriptions.get(alias_type, alias_type)
        print(f"--- {desc} ({len(items)} found) ---")
        print()
        for fn, target in items:
            print(f"  {fn.file}:{fn.line_num} {fn.name}")
            print(f"    → {target}")
            print()
    
    print("=" * 80)
    print(f"Total: {len(all_aliases)} potential aliases found")
    print()
    print("NOTE: These are candidates for review. Some may be intentional")
    print("      (e.g., SMT term sharing optimization) or provide useful abstraction.")


if __name__ == "__main__":
    main()
