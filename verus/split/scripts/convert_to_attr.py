#!/usr/bin/env python3
"""
Convert split exec files from verus! macro syntax to attribute-based syntax.

Strategy:
- Types (struct, enum) stay in verus! block
- Functions move outside verus! block with #[verus_spec] attributes
- spec fn, proof fn, impl View stay in verus! block
- include! statements stay in verus! block

Usage:
    python3 convert_to_attr.py <input_file.rs> [--dry-run]
"""

import re
import sys
import argparse
from typing import List, Optional, Tuple, Set
from dataclasses import dataclass, field
from enum import Enum, auto

class ItemType(Enum):
    INCLUDE = auto()
    SPEC_FN = auto()
    PROOF_FN = auto()
    EXEC_FN = auto()
    IMPL_VIEW = auto()
    TYPE_DEF = auto()  # struct or enum
    IMPL_BLOCK = auto()
    COMMENT = auto()
    OTHER = auto()

@dataclass
class FunctionSpec:
    requires: List[str] = field(default_factory=list)
    ensures: List[str] = field(default_factory=list)
    decreases: List[str] = field(default_factory=list)
    recommends: List[str] = field(default_factory=list)
    result_name: Optional[str] = None

@dataclass
class ParsedItem:
    item_type: ItemType
    start_line: int
    end_line: int
    doc_start: int  # Where doc comments start
    content: str
    specs: Optional[FunctionSpec] = None
    attrs: List[str] = field(default_factory=list)

def find_matching_brace_lines(lines: List[str], start_line: int) -> int:
    """Find line containing matching closing brace."""
    depth = 0
    first_found = False
    for i in range(start_line, len(lines)):
        for ch in lines[i]:
            if ch == '{':
                depth += 1
                first_found = True
            elif ch == '}':
                depth -= 1
                if first_found and depth == 0:
                    return i
    return len(lines) - 1

def find_matching_brace(text: str, start: int) -> int:
    depth = 0
    i = start
    while i < len(text):
        if text[i] == '{':
            depth += 1
        elif text[i] == '}':
            depth -= 1
            if depth == 0:
                return i
        i += 1
    return len(text) - 1

def identify_item_type(line: str) -> ItemType:
    """Identify what type of item starts at this line."""
    stripped = line.strip()
    
    if stripped.startswith('include!'):
        return ItemType.INCLUDE
    if re.match(r'(pub\s+)?(open\s+|closed\s+)?spec\s+fn\s+', stripped):
        return ItemType.SPEC_FN
    if re.match(r'(pub\s+)?proof\s+fn\s+', stripped):
        return ItemType.PROOF_FN
    if re.match(r'impl\s+View\s+for\s+', stripped):
        return ItemType.IMPL_VIEW
    if re.match(r'(pub\s+)?(struct|enum)\s+\w+', stripped) and not stripped.startswith('//'):
        return ItemType.TYPE_DEF
    if re.match(r'(pub\s+)?(unsafe\s+)?(const\s+)?fn\s+', stripped):
        return ItemType.EXEC_FN
    if re.match(r'impl\s+', stripped) and not re.match(r'impl\s+View\s+', stripped):
        return ItemType.IMPL_BLOCK
    if stripped.startswith('//') or stripped == '':
        return ItemType.COMMENT
    return ItemType.OTHER

def parse_spec_clause(lines: List[str], start_idx: int, keyword: str) -> Tuple[List[str], int]:
    """Parse a spec clause."""
    result = []
    i = start_idx
    
    line = lines[i].strip()
    if not line.startswith(keyword):
        return [], i
    
    rest = line[len(keyword):].strip()
    if rest:
        result.append(rest)
    i += 1
    
    keywords = ('requires', 'ensures', 'decreases', 'recommends', 'opens_invariants')
    while i < len(lines):
        stripped = lines[i].strip()
        if any(stripped.startswith(kw) for kw in keywords):
            break
        if stripped == '{' or (stripped.endswith('{') and '=>' not in stripped):
            break
        if stripped and not stripped.startswith('//'):
            result.append(stripped)
        i += 1
    
    return result, i

def parse_function_specs(lines: List[str], fn_start: int) -> Tuple[FunctionSpec, int]:
    """Parse function specs starting after signature."""
    specs = FunctionSpec()
    i = fn_start
    
    # Skip to end of signature
    while i < len(lines):
        stripped = lines[i].strip()
        if any(stripped.startswith(kw) for kw in ('requires', 'ensures', 'decreases', 'recommends', 'opens_invariants')):
            break
        if stripped == '{' or stripped.endswith('{'):
            return specs, i
        i += 1
    
    # Parse spec clauses
    while i < len(lines):
        stripped = lines[i].strip()
        
        if stripped.startswith('requires'):
            specs.requires, i = parse_spec_clause(lines, i, 'requires')
        elif stripped.startswith('ensures'):
            specs.ensures, i = parse_spec_clause(lines, i, 'ensures')
        elif stripped.startswith('decreases'):
            specs.decreases, i = parse_spec_clause(lines, i, 'decreases')
        elif stripped.startswith('recommends'):
            specs.recommends, i = parse_spec_clause(lines, i, 'recommends')
        elif stripped.startswith('opens_invariants'):
            _, i = parse_spec_clause(lines, i, 'opens_invariants')
        elif stripped == '{' or stripped.endswith('{'):
            break
        else:
            i += 1
    
    return specs, i

def convert_return_type(sig: str) -> Tuple[str, Optional[str]]:
    """Convert -> (result: Type) to -> Type, return (new_sig, result_name)."""
    pattern = r'->\s*\((\w+)\s*:\s*([^)]+)\)\s*$'
    match = re.search(pattern, sig.rstrip())
    if match:
        result_name = match.group(1)
        return_type = match.group(2).strip()
        new_sig = re.sub(pattern, f'-> {return_type}', sig.rstrip())
        return new_sig, result_name
    return sig, None

def format_verus_spec(specs: FunctionSpec, result_name: Optional[str], indent: str) -> str:
    """Format specs as #[verus_spec(...)]."""
    name = result_name or specs.result_name or 'result'
    parts = []
    
    if specs.requires:
        req_lines = '\n'.join(f'{indent}        {l}' for l in specs.requires)
        parts.append(f'{indent}    requires\n{req_lines}')
    
    if specs.ensures:
        ens_lines = '\n'.join(f'{indent}        {l}' for l in specs.ensures)
        parts.append(f'{indent}    ensures\n{ens_lines}')
    
    if specs.decreases:
        dec_lines = '\n'.join(f'{indent}        {l}' for l in specs.decreases)
        parts.append(f'{indent}    decreases\n{dec_lines}')
    
    if specs.recommends:
        rec_lines = '\n'.join(f'{indent}        {l}' for l in specs.recommends)
        parts.append(f'{indent}    recommends\n{rec_lines}')
    
    if not parts:
        return f'{indent}#[verus_spec]\n'
    
    spec_content = ',\n'.join(parts)
    spec_content = re.sub(r',\s*$', '', spec_content, flags=re.MULTILINE)
    
    return f'{indent}#[verus_spec({name} =>\n{spec_content},\n{indent})]\n'

def convert_proof_blocks(body: str) -> str:
    """Convert proof { ... } to proof! { ... }."""
    return re.sub(r'(\s*)proof\s*\{', r'\1proof! {', body)

def convert_function_to_attr(lines: List[str], fn_start: int, fn_end: int, doc_start: int) -> str:
    """Convert a function to attribute-based syntax."""
    result = []
    
    # Collect signature lines (before specs)
    sig_lines = []
    i = fn_start
    keywords = ('requires', 'ensures', 'decreases', 'recommends', 'opens_invariants')
    
    while i <= fn_end:
        stripped = lines[i].strip()
        if any(stripped.startswith(kw) for kw in keywords):
            break
        if stripped == '{':
            break
        if stripped.endswith('{') and '=>' not in stripped:
            sig_lines.append(lines[i].rstrip()[:-1].rstrip())
            break
        sig_lines.append(lines[i].rstrip())
        i += 1
    
    signature = '\n'.join(sig_lines)
    
    # Parse specs
    specs, body_start = parse_function_specs(lines, fn_start)
    
    # Find body start
    while body_start <= fn_end and '{' not in lines[body_start]:
        body_start += 1
    
    # Convert signature
    converted_sig, result_name = convert_return_type(signature)
    if result_name:
        specs.result_name = result_name
    
    # Get indent
    indent_match = re.match(r'^(\s*)', lines[fn_start])
    indent = indent_match.group(1) if indent_match else ''
    
    # Collect doc comments
    for j in range(doc_start, fn_start):
        line = lines[j]
        stripped = line.strip()
        if stripped.startswith('///') or stripped.startswith('//='):
            result.append(line)
    
    # Collect and convert attributes
    for j in range(doc_start, fn_start):
        line = lines[j]
        stripped = line.strip()
        if stripped.startswith('#['):
            if '#[verifier::external_body]' in line:
                result.append(line.replace('#[verifier::external_body]', '#[verus_verify(external_body)]'))
            elif '#[verifier::external]' in line:
                result.append(line.replace('#[verifier::external]', '#[verus_verify(external)]'))
            else:
                result.append(line)
    
    # Add verus_spec
    has_specs = specs.requires or specs.ensures or specs.decreases or specs.recommends
    if has_specs:
        result.append(format_verus_spec(specs, result_name, indent).rstrip())
    else:
        result.append(f'{indent}#[verus_spec]')
    
    # Add converted signature
    result.append(converted_sig)
    
    # Add body
    sig_line = lines[body_start] if body_start <= fn_end else ''
    brace_pos = sig_line.find('{')
    
    if brace_pos >= 0:
        if sig_line[:brace_pos].strip() and not converted_sig.rstrip().endswith('{'):
            result.append(indent + '{')
        after_brace = sig_line[brace_pos + 1:].rstrip()
        if after_brace:
            result.append(after_brace)
        body_lines = lines[body_start + 1:fn_end + 1]
    else:
        body_lines = lines[body_start:fn_end + 1]
    
    body_text = '\n'.join(body_lines)
    body_text = convert_proof_blocks(body_text)
    result.append(body_text)
    
    return '\n'.join(result)

def process_verus_block(lines: List[str]) -> Tuple[str, str]:
    """
    Process content inside a verus! block.
    Returns: (code_for_outside, code_to_keep_inside)
    """
    # First pass: identify all items and their boundaries
    items = []
    processed = set()
    i = 0
    
    while i < len(lines):
        if i in processed:
            i += 1
            continue
        
        line = lines[i]
        item_type = identify_item_type(line)
        
        if item_type in (ItemType.COMMENT, ItemType.OTHER):
            i += 1
            continue
        
        # Find doc comments and attributes that belong to this item
        doc_start = i
        while doc_start > 0:
            prev = lines[doc_start - 1].strip()
            if prev.startswith('#[') or prev.startswith('///') or prev.startswith('//=') or prev == '':
                doc_start -= 1
            else:
                break
        
        # Find end of item
        if item_type in (ItemType.SPEC_FN, ItemType.PROOF_FN, ItemType.EXEC_FN):
            end = find_matching_brace_lines(lines, i)
        elif item_type == ItemType.IMPL_VIEW:
            end = find_matching_brace_lines(lines, i)
        elif item_type == ItemType.TYPE_DEF:
            end = i
            if '{' in line:
                end = find_matching_brace_lines(lines, i)
            else:
                while end < len(lines) and '{' not in lines[end]:
                    end += 1
                if end < len(lines):
                    end = find_matching_brace_lines(lines, end)
        elif item_type == ItemType.IMPL_BLOCK:
            end = find_matching_brace_lines(lines, i)
        elif item_type == ItemType.INCLUDE:
            end = i
        else:
            end = i
        
        items.append(ParsedItem(
            item_type=item_type,
            start_line=i,
            end_line=end,
            doc_start=doc_start,
            content='\n'.join(lines[doc_start:end+1])
        ))
        
        # Mark lines as processed
        for j in range(doc_start, end + 1):
            processed.add(j)
        
        i = end + 1
    
    # Second pass: distribute items
    inside_items = []
    outside_items = []
    
    for item in items:
        if item.item_type in (ItemType.INCLUDE, ItemType.SPEC_FN, ItemType.PROOF_FN, ItemType.IMPL_VIEW, ItemType.TYPE_DEF):
            inside_items.append(item.content)
        elif item.item_type == ItemType.EXEC_FN:
            converted = convert_function_to_attr(lines, item.start_line, item.end_line, item.doc_start)
            outside_items.append(converted)
        elif item.item_type == ItemType.IMPL_BLOCK:
            # Check if impl has only spec methods
            impl_content = item.content
            has_exec = re.search(r'(?<!spec\s)(?<!proof\s)pub\s+fn\s+', impl_content) or \
                       re.search(r'(?<!spec\s)(?<!proof\s)^\s+fn\s+', impl_content, re.MULTILINE)
            
            if not has_exec:
                inside_items.append(item.content)
            else:
                # Process impl block - convert exec functions
                impl_lines = lines[item.doc_start:item.end_line + 1]
                converted = convert_impl_block(impl_lines)
                outside_items.append(converted)
    
    # Collect remaining comments/separators
    remaining = []
    for i, line in enumerate(lines):
        if i not in processed:
            stripped = line.strip()
            if stripped.startswith('//=') or stripped == '':
                if stripped.startswith('//='):
                    remaining.append(line)
    
    inside_code = '\n'.join(inside_items)
    outside_code = '\n'.join(outside_items)
    
    return outside_code, inside_code

def convert_impl_block(impl_lines: List[str]) -> str:
    """Convert an impl block, processing its functions."""
    result = []
    i = 0
    
    while i < len(impl_lines):
        line = impl_lines[i]
        stripped = line.strip()
        
        # Check for exec function
        if re.match(r'\s*(pub\s+)?(unsafe\s+)?(const\s+)?fn\s+', stripped) and \
           not re.match(r'\s*(pub\s+)?(open\s+|closed\s+)?spec\s+fn\s+', stripped) and \
           not re.match(r'\s*(pub\s+)?proof\s+fn\s+', stripped):
            
            fn_end = find_matching_brace_lines(impl_lines, i)
            
            # Find doc start
            doc_start = i
            while doc_start > 0:
                prev = impl_lines[doc_start - 1].strip()
                if prev.startswith('#[') or prev.startswith('///') or prev.startswith('//=') or prev == '':
                    doc_start -= 1
                else:
                    break
            
            converted = convert_function_to_attr(impl_lines, i, fn_end, doc_start)
            result.append(converted)
            
            i = fn_end + 1
            continue
        
        result.append(line)
        i += 1
    
    return '\n'.join(result)

def process_file(content: str) -> str:
    """Process file and convert to attribute-based syntax."""
    lines = content.split('\n')
    result_lines = []
    
    feature_added = False
    i = 0
    
    while i < len(lines):
        line = lines[i]
        stripped = line.strip()
        
        # Add feature after module docs
        if not feature_added:
            if stripped.startswith('//!') or stripped.startswith('// Copyright') or stripped.startswith('// Licensed') or stripped == '':
                result_lines.append(line)
                i += 1
                continue
            else:
                result_lines.append('#![feature(proc_macro_hygiene)]')
                result_lines.append('')
                feature_added = True
        
        # Handle verus! block
        if stripped.startswith('verus!') and '{' in line:
            verus_end = find_matching_brace_lines(lines, i)
            verus_lines = lines[i+1:verus_end]
            
            outside_code, inside_code = process_verus_block(verus_lines)
            
            # Keep verus! block for types, specs, etc.
            if inside_code.strip():
                result_lines.append('verus! {')
                result_lines.append(inside_code)
                result_lines.append('} // verus!')
                result_lines.append('')
            
            # Add converted functions outside
            if outside_code.strip():
                result_lines.append(outside_code)
            
            i = verus_end + 1
            continue
        
        result_lines.append(line)
        i += 1
    
    # Clean up multiple empty lines
    cleaned = []
    prev_empty = False
    for line in result_lines:
        if line.strip() == '':
            if not prev_empty:
                cleaned.append(line)
            prev_empty = True
        else:
            cleaned.append(line)
            prev_empty = False
    
    return '\n'.join(cleaned)

def main():
    parser = argparse.ArgumentParser(description='Convert verus! macro to attribute-based syntax')
    parser.add_argument('input', help='Input file')
    parser.add_argument('--dry-run', '-n', action='store_true', help='Print without writing')
    parser.add_argument('--output', '-o', help='Output file (default: overwrite input)')
    args = parser.parse_args()
    
    with open(args.input, 'r') as f:
        content = f.read()
    
    result = process_file(content)
    
    if args.dry_run:
        print(result)
    else:
        output = args.output or args.input
        with open(output, 'w') as f:
            f.write(result)
        print(f"Converted: {args.input}")

if __name__ == '__main__':
    main()
