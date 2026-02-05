#!/usr/bin/env python3
"""
Verus Code Splitter v2

Splits a Verus source file by function type:
- <name>.rs: exec code (structs, consts, exec fn, impl blocks with exec fn)
- <name>.spec.rs: spec functions only
- <name>.proof.rs: proof functions only

Key insight: Parse line-by-line to identify function types, even within impl blocks.
"""

import re
import sys
import os
from typing import List, Tuple, Optional

def find_matching_brace(lines: List[str], start_line: int, start_col: int) -> Tuple[int, int]:
    """Find matching closing brace. Returns (line_idx, col_idx)."""
    depth = 1
    i = start_line
    j = start_col + 1
    
    while i < len(lines):
        line = lines[i]
        while j < len(line):
            c = line[j]
            if c == '{':
                depth += 1
            elif c == '}':
                depth -= 1
                if depth == 0:
                    return (i, j)
            j += 1
        i += 1
        j = 0
    return (-1, -1)

def extract_function(lines: List[str], start_line: int) -> Tuple[List[str], int]:
    """Extract a complete function starting from start_line."""
    result = []
    i = start_line
    
    # Collect preceding doc comments and attributes
    while i > 0:
        prev = lines[i-1].strip()
        if prev.startswith('///') or prev.startswith('#['):
            i -= 1
        else:
            break
    
    # Find the opening brace
    brace_line = -1
    brace_col = -1
    for li in range(i, min(i + 20, len(lines))):  # Look ahead max 20 lines
        col = lines[li].find('{')
        if col != -1:
            brace_line = li
            brace_col = col
            break
    
    if brace_line == -1:
        # No brace found, might be a function declaration ending with ;
        for li in range(i, min(i + 10, len(lines))):
            if lines[li].rstrip().endswith(';'):
                return lines[i:li+1], li + 1
        return lines[i:i+1], i + 1
    
    # Find matching brace
    end_line, end_col = find_matching_brace(lines, brace_line, brace_col)
    if end_line == -1:
        return lines[i:brace_line+1], brace_line + 1
    
    return lines[i:end_line+1], end_line + 1

def classify_function(lines: List[str]) -> str:
    """Classify a function as 'spec', 'proof', or 'exec'."""
    text = '\n'.join(lines)
    
    if re.search(r'\bproof\s+fn\b', text):
        return 'proof'
    if re.search(r'\b(open\s+|closed\s+)?spec\s+fn\b', text):
        return 'spec'
    
    return 'exec'

def split_file(input_path: str, output_dir: Optional[str] = None) -> bool:
    """Split a Verus file into exec, spec, and proof files."""
    with open(input_path, 'r') as f:
        content = f.read()
    
    if output_dir is None:
        output_dir = os.path.dirname(input_path) or '.'
    os.makedirs(output_dir, exist_ok=True)
    
    lines = content.split('\n')
    
    # Find verus! block
    verus_start = -1
    verus_end = -1
    for i, line in enumerate(lines):
        if 'verus!' in line and '{' in line:
            verus_start = i
            break
    
    if verus_start == -1:
        print(f"No verus! block found in {input_path}")
        return False
    
    # Find end of verus! block
    for i in range(len(lines) - 1, verus_start, -1):
        if '} // verus!' in lines[i]:
            verus_end = i
            break
    
    if verus_end == -1:
        print(f"Could not find end of verus! block")
        return False
    
    # Pre-verus content
    pre_verus = '\n'.join(lines[:verus_start])
    post_verus = '\n'.join(lines[verus_end+1:])
    
    # Parse verus block content
    verus_lines = lines[verus_start+1:verus_end]
    
    exec_parts = []
    spec_fns = []
    proof_fns = []
    
    i = 0
    while i < len(verus_lines):
        line = verus_lines[i]
        stripped = line.strip()
        
        # Skip empty lines and comments
        if not stripped or stripped.startswith('//'):
            exec_parts.append(line)
            i += 1
            continue
        
        # Check if this is a function definition
        fn_match = re.search(r'\b(pub\s+)?(open\s+|closed\s+)?(spec|proof)?\s*fn\s+(\w+)', stripped)
        
        if fn_match:
            fn_type = fn_match.group(3)  # 'spec', 'proof', or None
            fn_lines, next_i = extract_function(verus_lines, i)
            
            if fn_type == 'spec':
                spec_fns.append(fn_lines)
            elif fn_type == 'proof':
                proof_fns.append(fn_lines)
            else:
                exec_parts.extend(fn_lines)
            
            i = next_i
            continue
        
        # Check for impl block
        impl_match = re.match(r'\s*impl\b', stripped)
        if impl_match:
            # Find the opening brace
            brace_col = line.find('{')
            if brace_col == -1:
                # Look on next lines
                impl_header = [line]
                j = i + 1
                while j < len(verus_lines):
                    impl_header.append(verus_lines[j])
                    if '{' in verus_lines[j]:
                        brace_col = verus_lines[j].find('{')
                        break
                    j += 1
                impl_start_line = j
            else:
                impl_header = [line]
                impl_start_line = i
            
            # Find end of impl
            end_line, _ = find_matching_brace(verus_lines, impl_start_line, 
                                               verus_lines[impl_start_line].find('{'))
            
            # Process impl block - separate spec/proof functions
            impl_exec_lines = impl_header.copy()
            impl_body_start = len(impl_header)
            
            j = impl_start_line + 1
            while j < end_line:
                inner_line = verus_lines[j]
                inner_stripped = inner_line.strip()
                
                inner_fn_match = re.search(r'\b(pub\s+)?(open\s+|closed\s+)?(spec|proof)?\s*fn\s+(\w+)', inner_stripped)
                if inner_fn_match:
                    fn_type = inner_fn_match.group(3)
                    fn_lines, next_j = extract_function(verus_lines, j)
                    
                    if fn_type == 'spec':
                        spec_fns.append(fn_lines)
                    elif fn_type == 'proof':
                        proof_fns.append(fn_lines)
                    else:
                        impl_exec_lines.extend(fn_lines)
                    j = next_j
                else:
                    impl_exec_lines.append(inner_line)
                    j += 1
            
            # Add closing brace
            impl_exec_lines.append(verus_lines[end_line])
            exec_parts.extend(impl_exec_lines)
            i = end_line + 1
            continue
        
        # Other content (structs, enums, consts, etc.)
        # Check if it's a multi-line item
        if '{' in stripped:
            brace_col = line.find('{')
            end_line, _ = find_matching_brace(verus_lines, i, brace_col)
            if end_line != -1:
                exec_parts.extend(verus_lines[i:end_line+1])
                i = end_line + 1
                continue
        
        exec_parts.append(line)
        i += 1
    
    # Generate output
    basename = os.path.basename(input_path).replace('.rs', '')
    
    # Main file
    main_lines = [pre_verus, '', 
                  '// Include specifications.', f'include!("{basename}.spec.rs");', '',
                  '// Include proofs.', f'include!("{basename}.proof.rs");', '',
                  'verus! {', '']
    main_lines.extend(exec_parts)
    main_lines.extend(['', '} // verus!'])
    if post_verus.strip():
        main_lines.extend(['', post_verus])
    
    # Spec file
    spec_lines = ['// Copyright(c) The Maintainers of Nanvix.',
                  '// Licensed under the MIT License.', '',
                  '// Specification functions.', '', 'verus! {', '']
    for fn in spec_fns:
        spec_lines.extend(fn)
        spec_lines.append('')
    spec_lines.append('} // verus!')
    
    # Proof file
    proof_lines = ['// Copyright(c) The Maintainers of Nanvix.',
                   '// Licensed under the MIT License.', '',
                   '// Proof functions and lemmas.', '', 'verus! {', '']
    for fn in proof_fns:
        proof_lines.extend(fn)
        proof_lines.append('')
    proof_lines.append('} // verus!')
    
    # Write files
    with open(os.path.join(output_dir, f'{basename}.rs'), 'w') as f:
        f.write('\n'.join(main_lines))
    with open(os.path.join(output_dir, f'{basename}.spec.rs'), 'w') as f:
        f.write('\n'.join(spec_lines))
    with open(os.path.join(output_dir, f'{basename}.proof.rs'), 'w') as f:
        f.write('\n'.join(proof_lines))
    
    print(f"  -> {basename}.rs (exec)")
    print(f"  -> {basename}.spec.rs ({len(spec_fns)} spec fns)")
    print(f"  -> {basename}.proof.rs ({len(proof_fns)} proof fns)")
    
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
