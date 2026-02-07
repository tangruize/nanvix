#!/usr/bin/env python3
"""
Full conversion from verus! macro syntax to attribute-based syntax.

This script converts exec functions from inside verus! blocks to outside,
using #[verus_spec(...)] for specifications.

Key transformations:
1. Move exec functions outside verus! blocks
2. Add #[verus_spec(...)] with requires/ensures/decreases
3. Convert named returns (result: Type) to #[verus_spec(result => ...)]
4. Convert proof { } to proof! { }
5. Keep spec/proof functions, structs, enums, impls inside verus! blocks
"""

import re
import sys
import os
from typing import List, Tuple, Optional
from dataclasses import dataclass


def find_matching_brace(text: str, start: int) -> int:
    """Find the matching closing brace for an opening brace at start position."""
    depth = 0
    i = start
    in_string = False
    
    while i < len(text):
        c = text[i]
        
        # Handle string literals
        if c == '"' and (i == 0 or text[i-1] != '\\'):
            in_string = not in_string
            i += 1
            continue
            
        if in_string:
            if c == '\\' and i + 1 < len(text):
                i += 2  # Skip escaped character
                continue
            i += 1
            continue
        
        # Handle comments
        if c == '/' and i + 1 < len(text):
            if text[i+1] == '/':
                # Line comment
                while i < len(text) and text[i] != '\n':
                    i += 1
                continue
            elif text[i+1] == '*':
                # Block comment
                i += 2
                while i + 1 < len(text):
                    if text[i:i+2] == '*/':
                        i += 2
                        break
                    i += 1
                continue
        
        if c == '{':
            depth += 1
        elif c == '}':
            depth -= 1
            if depth == 0:
                return i
        i += 1
    return -1


def find_matching_paren(text: str, start: int) -> int:
    """Find the matching closing paren for an opening paren at start position."""
    depth = 0
    i = start
    in_string = False
    
    while i < len(text):
        c = text[i]
        
        if c == '"' and (i == 0 or text[i-1] != '\\'):
            in_string = not in_string
            i += 1
            continue
            
        if in_string:
            if c == '\\' and i + 1 < len(text):
                i += 2
                continue
            i += 1
            continue
        
        if c == '(':
            depth += 1
        elif c == ')':
            depth -= 1
            if depth == 0:
                return i
        i += 1
    return -1


@dataclass
class ParsedFunction:
    """A parsed function from verus! block."""
    full_text: str  # Complete function text
    attributes: str  # Attributes like #[verifier::...]
    visibility: str  # pub, pub(crate), etc.
    is_unsafe: bool
    name: str
    generics: str
    params: str
    return_clause: str  # Everything from -> to body start
    return_name: Optional[str]  # Named return e.g., result in (result: Type)
    return_type: str  # The actual type
    requires: Optional[str]
    ensures: Optional[str]
    decreases: Optional[str]
    where_clause: str
    body: str
    start_pos: int
    end_pos: int


def skip_whitespace_and_comments(text: str, pos: int) -> int:
    """Skip whitespace and comments starting at pos."""
    i = pos
    while i < len(text):
        if text[i] in ' \t\n\r':
            i += 1
            continue
        if i + 1 < len(text) and text[i:i+2] == '//':
            while i < len(text) and text[i] != '\n':
                i += 1
            continue
        if i + 1 < len(text) and text[i:i+2] == '/*':
            i += 2
            while i + 1 < len(text):
                if text[i:i+2] == '*/':
                    i += 2
                    break
                i += 1
            continue
        break
    return i


def is_exec_function(text: str) -> bool:
    """Check if the text starts with an exec function (not proof/spec)."""
    # Look for function pattern without proof/spec keyword
    pattern = r'^(\s*(#\[[^\]]+\]\s*)*)?(pub(\s*\([^)]*\))?\s+)?(unsafe\s+)?fn\s+'
    match = re.match(pattern, text)
    if not match:
        return False
    
    # Check it's not a proof or spec function
    if re.search(r'\b(proof|spec)\s+fn\b', text[:200]):
        return False
    
    return True


def parse_exec_function(text: str, start_offset: int) -> Optional[ParsedFunction]:
    """Parse an exec function from text.
    
    Returns ParsedFunction or None if not an exec function.
    """
    # Pattern to match function declaration
    # Attributes, visibility, unsafe, fn, name, generics, params
    pattern = r'''
        ^
        ((?:\s*\#\[[^\]]+\]\s*)*)     # Attributes (group 1)
        ((?:pub(?:\s*\([^)]*\))?\s+)?) # Visibility (group 2)
        (unsafe\s+)?                    # unsafe (group 3)
        fn\s+                           # fn keyword
        (\w+)                           # Function name (group 4)
        (<[^{]*>)?                      # Generics (group 5) - simplified
        \s*\(
    '''
    
    match = re.match(pattern, text, re.VERBOSE | re.DOTALL)
    if not match:
        return None
    
    attributes = match.group(1).strip() if match.group(1) else ""
    visibility = match.group(2).strip() if match.group(2) else ""
    is_unsafe = bool(match.group(3))
    name = match.group(4)
    generics = match.group(5) if match.group(5) else ""
    
    # Find the end of parameters
    paren_start = match.end() - 1
    paren_end = find_matching_paren(text, paren_start)
    if paren_end == -1:
        return None
    
    params = text[paren_start+1:paren_end]
    
    # Parse everything after params until body
    after_params = text[paren_end+1:]
    
    # Find the body (first { that's at depth 0)
    body_start = -1
    i = 0
    depth = 0
    in_string = False
    
    while i < len(after_params):
        c = after_params[i]
        
        if c == '"' and (i == 0 or after_params[i-1] != '\\'):
            in_string = not in_string
        
        if not in_string:
            if c == '<':
                depth += 1
            elif c == '>':
                depth -= 1
            elif c == '{' and depth == 0:
                body_start = i
                break
        i += 1
    
    if body_start == -1:
        return None
    
    return_clause = after_params[:body_start].strip()
    body_end_rel = find_matching_brace(after_params, body_start)
    if body_end_rel == -1:
        return None
    
    body = after_params[body_start+1:body_end_rel]
    
    # Parse return clause: -> (name: Type) requires ... ensures ... decreases ... where ...
    return_name = None
    return_type = ""
    requires = None
    ensures = None
    decreases = None
    where_clause = ""
    
    # Check for -> 
    arrow_match = re.match(r'\s*->\s*', return_clause)
    if arrow_match:
        rest = return_clause[arrow_match.end():]
        
        # Look for named return like (result: Type)
        named_return_match = re.match(r'\(\s*(\w+)\s*:\s*', rest)
        if named_return_match:
            return_name = named_return_match.group(1)
            # Find end of the type (matching paren)
            paren_start_idx = rest.index('(')
            paren_end_idx = find_matching_paren(rest, paren_start_idx)
            if paren_end_idx != -1:
                # Extract the type from inside
                inner = rest[paren_start_idx+1:paren_end_idx]
                # Type is after the colon
                colon_idx = inner.index(':')
                return_type = inner[colon_idx+1:].strip()
                rest = rest[paren_end_idx+1:]
            else:
                return_type = rest
                rest = ""
        else:
            # No named return - find where type ends (before requires/ensures/decreases/where)
            keywords = ['requires', 'ensures', 'decreases', 'where']
            end_pos = len(rest)
            for kw in keywords:
                kw_match = re.search(r'\b' + kw + r'\b', rest)
                if kw_match and kw_match.start() < end_pos:
                    end_pos = kw_match.start()
            return_type = rest[:end_pos].strip().rstrip(',')
            rest = rest[end_pos:]
        
        # Parse clauses from rest
        def extract_clause(text: str, keyword: str, keywords: List[str]) -> Tuple[Optional[str], str]:
            kw_match = re.search(r'\b' + keyword + r'\b', text)
            if not kw_match:
                return None, text
            
            start = kw_match.end()
            # Skip whitespace
            while start < len(text) and text[start] in ' \t\n':
                start += 1
            
            # Find end (before next keyword or end)
            end = len(text)
            for other_kw in keywords:
                if other_kw != keyword:
                    other_match = re.search(r'\b' + other_kw + r'\b', text[start:])
                    if other_match and start + other_match.start() < end:
                        end = start + other_match.start()
            
            content = text[start:end].strip().rstrip(',')
            remaining = text[:kw_match.start()] + text[end:]
            return content, remaining.strip()
        
        keywords = ['requires', 'ensures', 'decreases', 'where']
        requires, rest = extract_clause(rest, 'requires', keywords)
        ensures, rest = extract_clause(rest, 'ensures', keywords)
        decreases, rest = extract_clause(rest, 'decreases', keywords)
        
        # Check for where clause
        where_match = re.search(r'\bwhere\b', rest)
        if where_match:
            where_clause = rest[where_match.start():].strip()
    
    end_pos = paren_end + 1 + body_end_rel + 1
    
    return ParsedFunction(
        full_text=text[:end_pos],
        attributes=attributes,
        visibility=visibility,
        is_unsafe=is_unsafe,
        name=name,
        generics=generics,
        params=params,
        return_clause=return_clause,
        return_name=return_name,
        return_type=return_type,
        requires=requires,
        ensures=ensures,
        decreases=decreases,
        where_clause=where_clause,
        body=body,
        start_pos=start_offset,
        end_pos=start_offset + end_pos
    )


def format_verus_spec_attr(fn: ParsedFunction) -> str:
    """Format #[verus_spec(...)] attribute for a function."""
    parts = []
    
    if fn.return_name:
        parts.append(f"{fn.return_name} =>")
    
    if fn.requires:
        # Format requires clause
        parts.append(f"    requires\n        {fn.requires},")
    
    if fn.ensures:
        parts.append(f"    ensures\n        {fn.ensures},")
    
    if fn.decreases:
        parts.append(f"    decreases {fn.decreases},")
    
    if not parts:
        return "#[verus_spec]"
    
    inner = '\n'.join(parts)
    return f"#[verus_spec({inner}\n)]"


def convert_proof_blocks(body: str) -> str:
    """Convert proof { } to proof! { }."""
    result = []
    i = 0
    
    while i < len(body):
        if body[i:i+5] == 'proof':
            j = i + 5
            while j < len(body) and body[j] in ' \t\n':
                j += 1
            
            # Check it's not 'proof fn'
            if body[j:j+2] == 'fn':
                result.append(body[i])
                i += 1
                continue
            
            if j < len(body) and body[j] == '{':
                brace_end = find_matching_brace(body, j)
                if brace_end != -1:
                    whitespace = body[i+5:j]
                    content = body[j+1:brace_end]
                    content = convert_proof_blocks(content)  # Recursive
                    result.append('proof!')
                    result.append(whitespace)
                    result.append('{')
                    result.append(content)
                    result.append('}')
                    i = brace_end + 1
                    continue
        
        result.append(body[i])
        i += 1
    
    return ''.join(result)


def convert_function_to_attr_style(fn: ParsedFunction) -> str:
    """Convert a function to attribute style."""
    lines = []
    
    # Keep existing attributes
    if fn.attributes:
        lines.append(fn.attributes)
    
    # Add #[verus_spec] if we have specs
    if fn.requires or fn.ensures or fn.decreases or fn.return_name:
        lines.append(format_verus_spec_attr(fn))
    
    # Build function signature
    sig_parts = []
    if fn.visibility:
        sig_parts.append(fn.visibility)
    if fn.is_unsafe:
        sig_parts.append("unsafe")
    sig_parts.append("fn")
    sig_parts.append(fn.name + fn.generics)
    
    sig = ' '.join(sig_parts)
    sig += f"({fn.params})"
    
    # Add return type (without the named part)
    if fn.return_type:
        sig += f" -> {fn.return_type}"
    
    # Add where clause
    if fn.where_clause:
        sig += f"\n    {fn.where_clause}"
    
    lines.append(sig)
    
    # Convert proof blocks in body
    converted_body = convert_proof_blocks(fn.body)
    
    lines.append("{" + converted_body + "}")
    
    return '\n'.join(lines)


def extract_verus_blocks(content: str) -> List[Tuple[int, int, str]]:
    """Extract all verus! { ... } blocks."""
    blocks = []
    pattern = r'verus!\s*\{'
    
    for match in re.finditer(pattern, content):
        start = match.start()
        brace_start = match.end() - 1
        brace_end = find_matching_brace(content, brace_start)
        if brace_end != -1:
            block_content = content[brace_start+1:brace_end]
            blocks.append((start, brace_end + 1, block_content))
    
    return blocks


def process_verus_block(block_content: str) -> Tuple[str, str]:
    """Process verus! block content.
    
    Returns (remaining_in_verus, functions_to_move_out).
    """
    # For now, just convert proof blocks and keep everything in verus!
    converted = convert_proof_blocks(block_content)
    return converted, ""


def convert_file(filepath: str) -> str:
    """Convert a file to attribute-based syntax."""
    with open(filepath, 'r') as f:
        content = f.read()
    
    blocks = extract_verus_blocks(content)
    if not blocks:
        return content
    
    result = content
    for start, end, block_content in reversed(blocks):
        # Just convert proof blocks for now
        converted = convert_proof_blocks(block_content)
        new_block = f"verus! {{{converted}}}"
        result = result[:start] + new_block + result[end:]
    
    return result


def main():
    if len(sys.argv) < 2:
        print("Usage: convert_full_attr.py <file.rs> [--dry-run]")
        sys.exit(1)
    
    filepath = sys.argv[1]
    dry_run = '--dry-run' in sys.argv
    
    if not os.path.exists(filepath):
        print(f"Error: File not found: {filepath}")
        sys.exit(1)
    
    result = convert_file(filepath)
    
    if dry_run:
        print(result)
    else:
        with open(filepath, 'w') as f:
            f.write(result)
        print(f"Converted: {filepath}")


if __name__ == '__main__':
    main()
