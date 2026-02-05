#!/usr/bin/env python3
"""
Verus Code Splitter

Splits a Verus source file into three parts:
- <name>.rs (impl): exec functions, structs, consts, external code
- <name>.spec.rs (spec): spec functions, View traits, invariants  
- <name>.proof.rs (proof): proof functions, lemmas, axioms, tests

Usage:
    python3 split_verus.py <input.rs> [output_dir]

Example:
    python3 split_verus.py raw_array.rs ./libs/raw_array/
"""

import re
import sys
import os
from typing import List, Tuple, Optional


def find_matching_brace(text: str, start: int) -> int:
    """Find the matching closing brace for an opening brace at position start."""
    depth = 0
    i = start
    in_string = False
    in_line_comment = False
    in_block_comment = False
    
    while i < len(text):
        c = text[i]
        
        if not in_string:
            if in_line_comment:
                if c == '\n':
                    in_line_comment = False
                i += 1
                continue
            if in_block_comment:
                if c == '*' and i + 1 < len(text) and text[i + 1] == '/':
                    in_block_comment = False
                    i += 2
                    continue
                i += 1
                continue
            if c == '/' and i + 1 < len(text):
                if text[i + 1] == '/':
                    in_line_comment = True
                    i += 2
                    continue
                if text[i + 1] == '*':
                    in_block_comment = True
                    i += 2
                    continue
        
        if c == '"' and (i == 0 or text[i-1] != '\\'):
            in_string = not in_string
        
        if not in_string:
            if c == '{':
                depth += 1
            elif c == '}':
                depth -= 1
                if depth == 0:
                    return i
        i += 1
    return -1


def extract_item(lines: List[str], start_idx: int) -> Tuple[List[str], int]:
    """Extract a complete item (fn, struct, impl, etc.) starting from start_idx."""
    result = []
    i = start_idx
    
    # Collect leading doc comments and attributes
    while i < len(lines):
        line = lines[i].strip()
        if line.startswith('///') or line.startswith('#[') or line.startswith('//='):
            result.append(lines[i])
            i += 1
        elif line == '':
            if result:  # Only add empty lines if we've started collecting
                result.append(lines[i])
            i += 1
        else:
            break
    
    if i >= len(lines):
        return result, i
    
    # Detect if this is a function (need special handling for requires/ensures)
    first_line = lines[i].strip() if i < len(lines) else ''
    is_function = bool(re.search(r'\b(pub\s+)?(open\s+|closed\s+)?(spec\s+|proof\s+|exec\s+)?fn\s+', first_line))
    
    # Find the item body
    brace_depth = 0
    found_body_brace = False
    in_signature = is_function  # For functions, we're initially in signature
    
    while i < len(lines):
        line = lines[i]
        stripped = line.strip()
        result.append(line)
        
        # For functions, detect when we've left the signature (requires/ensures/decreases)
        # The body starts with a line that is just '{' or ends with '{'  after signature
        if is_function and in_signature:
            # Check if this line starts the body (line is just '{' or a '{' at proper position)
            if stripped == '{':
                in_signature = False
                found_body_brace = True
                brace_depth = 1
                i += 1
                continue
            # Still in signature, don't count braces in expressions like (Foo { field })
            i += 1
            continue
        
        # Count braces for body
        for c in line:
            if c == '{':
                brace_depth += 1
                found_body_brace = True
            elif c == '}':
                brace_depth -= 1
        
        i += 1
        
        # Check for semicolon-terminated items (const, axiom without body)
        if not found_body_brace and stripped.endswith(';'):
            break
        
        # Check if we've closed all braces
        if found_body_brace and brace_depth == 0:
            break
    
    return result, i


def classify_item(lines: List[str]) -> str:
    """Classify an item as 'spec', 'proof', or 'exec'."""
    text = '\n'.join(lines)
    
    # Test modules
    if '#[cfg(verus_keep_ghost)]' in text:
        return 'proof'
    
    # Proof/lemma functions
    if re.search(r'\bproof\s+fn\b', text):
        return 'proof'
    if re.search(r'\baxiom\s+fn\b', text):
        return 'proof'
    
    # Spec functions
    if re.search(r'\b(open\s+|closed\s+)?spec\s+fn\b', text):
        return 'spec'
    if re.search(r'\buninterp\s+spec\s+fn\b', text):
        return 'spec'
    
    # View trait impl
    if re.search(r'impl\s*(<[^>]*>)?\s*View\s+for\b', text):
        return 'spec'
    
    # Spec structs
    if re.search(r'(pub\s+)?struct\s+\w+View\s*[<{]', text):
        return 'spec'
    
    # impl blocks with only spec functions
    if re.match(r'\s*impl', text):
        has_spec = bool(re.search(r'\b(open\s+|closed\s+)?spec\s+fn\b', text))
        has_exec = bool(re.search(r'\bpub\s+fn\b|\bfn\s+\w+\s*\([^)]*\)\s*(->\s*[^{]+)?\s*\{', text))
        has_exec = has_exec and not re.search(r'\b(spec|proof)\s+fn\b', text)
        
        if has_spec and not has_exec:
            return 'spec'
    
    return 'exec'


def parse_verus_block(content: str) -> Tuple[List[Tuple[str, List[str]]], str, str]:
    """
    Parse a verus! block and classify its contents.
    Returns: (items, pre_content, post_content)
    where items is a list of (kind, lines) tuples
    """
    # Find verus! block
    match = re.search(r'verus!\s*\{', content)
    if not match:
        return [], content, ''
    
    pre_content = content[:match.start()]
    brace_start = match.end() - 1
    brace_end = find_matching_brace(content, brace_start)
    
    if brace_end == -1:
        return [], content, ''
    
    verus_inner = content[match.end():brace_end]
    post_content = content[brace_end + 1:]
    
    # Parse items inside verus block
    lines = verus_inner.split('\n')
    items = []
    i = 0
    
    while i < len(lines):
        line = lines[i].strip()
        
        # Skip empty lines and simple comments at top level
        if line == '' or (line.startswith('//') and not line.startswith('//=')):
            i += 1
            continue
        
        # Found an item start
        item_lines, next_i = extract_item(lines, i)
        if item_lines:
            # Remove leading/trailing empty lines
            while item_lines and item_lines[0].strip() == '':
                item_lines.pop(0)
            while item_lines and item_lines[-1].strip() == '':
                item_lines.pop()
            
            if item_lines:
                kind = classify_item(item_lines)
                items.append((kind, item_lines))
        
        i = next_i if next_i > i else i + 1
    
    return items, pre_content, post_content


def split_file(input_path: str, output_dir: Optional[str] = None) -> bool:
    """Split a Verus file into impl, spec, and proof files."""
    with open(input_path, 'r') as f:
        content = f.read()
    
    if output_dir is None:
        output_dir = os.path.dirname(input_path) or '.'
    os.makedirs(output_dir, exist_ok=True)
    
    # Handle multiple verus! blocks
    all_items = []
    remaining = content
    pre_content = ''
    post_content = ''
    
    first_block = True
    while 'verus!' in remaining:
        items, pre, post = parse_verus_block(remaining)
        if first_block:
            pre_content = pre
            first_block = False
        all_items.extend(items)
        
        # Check if there's another verus! block in post
        if 'verus!' in post:
            remaining = post
        else:
            post_content = post
            break
    
    if not all_items:
        print(f"No items found in verus! blocks in {input_path}")
        return False
    
    # Separate by kind
    exec_items = [lines for kind, lines in all_items if kind == 'exec']
    spec_items = [lines for kind, lines in all_items if kind == 'spec']
    proof_items = [lines for kind, lines in all_items if kind == 'proof']
    
    # Further separate exec items: structs/enums/consts first, then functions
    def is_type_def(lines):
        text = '\n'.join(lines)
        return bool(re.search(r'^\s*(pub\s+)?(struct|enum|const|tracked struct)\s+', text, re.MULTILINE))
    
    type_items = [item for item in exec_items if is_type_def(item)]
    fn_items = [item for item in exec_items if not is_type_def(item)]
    
    basename = os.path.basename(input_path).replace('.rs', '')
    
    # Generate impl file:
    # 1. First verus! block with type definitions (struct, enum, const)
    # 2. Include spec and proof (they have their own verus! blocks, can reference types)
    # 3. Second verus! block with exec functions
    impl_lines = [pre_content.rstrip(), '']
    
    # First verus! block: type definitions
    if type_items:
        impl_lines.append('verus! {')
        impl_lines.append('')
        for item in type_items:
            impl_lines.extend(item)
            impl_lines.append('')
        impl_lines.append('} // verus!')
        impl_lines.append('')
    
    # Include spec and proof
    impl_lines.append('// Include specifications.')
    impl_lines.append(f'include!("{basename}.spec.rs");')
    impl_lines.append('')
    impl_lines.append('// Include proofs.')
    impl_lines.append(f'include!("{basename}.proof.rs");')
    impl_lines.append('')
    
    # Second verus! block: exec functions
    if fn_items:
        impl_lines.append('verus! {')
        impl_lines.append('')
        for item in fn_items:
            impl_lines.extend(item)
            impl_lines.append('')
        impl_lines.append('} // verus!')
    
    if post_content.strip():
        impl_lines.append('')
        impl_lines.append(post_content.strip())
    
    # Generate spec file (with verus! wrapper)
    spec_lines = ['// Copyright(c) The Maintainers of Nanvix.', '// Licensed under the MIT License.', '',
                  '// Specification functions, View traits, and invariants.', '', 'verus! {', '']
    for item in spec_items:
        spec_lines.extend(item)
        spec_lines.append('')
    spec_lines.append('} // verus!')
    
    # Generate proof file (with verus! wrapper)
    proof_lines = ['// Copyright(c) The Maintainers of Nanvix.', '// Licensed under the MIT License.', '',
                   '// Proof functions, lemmas, axioms, and tests.', '', 'verus! {', '']
    for item in proof_items:
        proof_lines.extend(item)
        proof_lines.append('')
    proof_lines.append('} // verus!')
    
    # Write files
    impl_path = os.path.join(output_dir, f'{basename}.rs')
    spec_path = os.path.join(output_dir, f'{basename}.spec.rs')
    proof_path = os.path.join(output_dir, f'{basename}.proof.rs')
    
    with open(impl_path, 'w') as f:
        f.write('\n'.join(impl_lines) + '\n')
    print(f"  -> {impl_path}")
    
    with open(spec_path, 'w') as f:
        f.write('\n'.join(spec_lines) + '\n')
    print(f"  -> {spec_path}")
    
    with open(proof_path, 'w') as f:
        f.write('\n'.join(proof_lines) + '\n')
    print(f"  -> {proof_path}")
    
    print(f"  Summary: {len(exec_items)} exec, {len(spec_items)} spec, {len(proof_items)} proof")
    return True


def main():
    if len(sys.argv) < 2:
        print(__doc__)
        sys.exit(1)
    
    input_path = sys.argv[1]
    output_dir = sys.argv[2] if len(sys.argv) > 2 else None
    
    print(f"Splitting: {input_path}")
    if split_file(input_path, output_dir):
        print("Done!")
    else:
        sys.exit(1)


if __name__ == '__main__':
    main()
