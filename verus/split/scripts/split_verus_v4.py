#!/usr/bin/env python3
"""
Verus code splitter v4 - splits a verus file into exec, spec, and proof files.

This version uses a more robust parsing approach that handles Verus-specific
syntax like requires/ensures clauses.

Rules:
- spec fn, open spec fn, closed spec fn -> spec file
- proof fn -> proof file  
- pub fn, fn (without spec/proof) -> exec file (stays in main)
- Structs, enums, constants -> stay in exec (main file)
"""

import re
import sys
import os
from typing import List, Tuple, Optional

def find_function_end(lines: List[str], start_idx: int) -> int:
    """Find the end of a function, handling Verus requires/ensures syntax.
    
    Verus functions can have ensures clauses with blocks like:
        ensures result is Some ==> { ... },
    
    The actual function body { is distinguished by being on its own line
    (with only whitespace before it), not embedded in ensures clauses.
    """
    depth = 0
    in_body = False
    
    for i in range(start_idx, len(lines)):
        line = lines[i]
        stripped = line.strip()
        
        for j, char in enumerate(line):
            if char == '{':
                depth += 1
                if not in_body:
                    # Function body { is typically:
                    # 1. On its own line: "    {"
                    # 2. Or at the end of fn signature: "fn foo() {"
                    before_brace = line[:j].strip()
                    after_brace = line[j+1:].strip()
                    
                    # If { is at the start of line (only whitespace before)
                    # and depth becomes 1, this is the function body
                    if before_brace == '':
                        in_body = True
                    # Or if this is the first { and it's after a ) on fn signature line
                    elif depth == 1 and ')' in before_brace and not '==>' in before_brace:
                        in_body = True
                        
            elif char == '}':
                depth -= 1
                if in_body and depth == 0:
                    return i
    
    return len(lines) - 1

def find_block_end(lines: List[str], start_idx: int) -> int:
    """Find the end of a block (struct, impl, etc.)."""
    depth = 0
    found_first = False
    
    for i in range(start_idx, len(lines)):
        line = lines[i]
        for char in line:
            if char == '{':
                depth += 1
                found_first = True
            elif char == '}':
                depth -= 1
                if found_first and depth == 0:
                    return i
    
    return len(lines) - 1

def get_doc_comments_start(lines: List[str], idx: int, min_idx: int = 0) -> int:
    """Look back to find where doc comments/attributes start."""
    actual_start = idx
    while actual_start > min_idx:
        prev_line = lines[actual_start - 1].strip()
        if (prev_line.startswith('///') or 
            prev_line.startswith('#[') or 
            prev_line.startswith('//==') or
            prev_line == ''):
            actual_start -= 1
        else:
            break
    return actual_start

def is_spec_fn_line(line: str) -> bool:
    """Check if line declares a spec function."""
    stripped = line.strip()
    return bool(re.match(r'(pub\s+)?(open\s+|closed\s+)?spec\s+fn\s+', stripped))

def is_proof_fn_line(line: str) -> bool:
    """Check if line declares a proof function."""
    stripped = line.strip()
    return bool(re.match(r'(pub\s+)?proof\s+fn\s+', stripped))

def is_exec_fn_line(line: str) -> bool:
    """Check if line declares an exec function (regular fn)."""
    stripped = line.strip()
    # Match fn, pub fn, unsafe fn, pub unsafe fn, but not spec fn or proof fn
    if re.match(r'(pub\s+)?(unsafe\s+)?fn\s+', stripped):
        if not is_spec_fn_line(line) and not is_proof_fn_line(line):
            return True
    return False

def is_impl_line(line: str) -> bool:
    """Check if line starts an impl block."""
    return bool(re.match(r'\s*impl\s+', line))

def is_struct_line(line: str) -> bool:
    """Check if line starts a struct."""
    return bool(re.match(r'\s*(#\[|pub\s+)?(ghost\s+|tracked\s+)?struct\s+', line.strip()))

def is_ghost_struct_line(line: str) -> bool:
    """Check if line starts a ghost or tracked struct (goes to spec file)."""
    stripped = line.strip()
    return bool(re.match(r'\s*(pub\s+)?(ghost|tracked)\s+struct\s+', stripped))

def is_enum_line(line: str) -> bool:
    """Check if line starts an enum."""
    return bool(re.match(r'\s*(pub\s+)?enum\s+', line.strip()))

def is_trait_line(line: str) -> bool:
    """Check if line starts a trait definition."""
    return bool(re.match(r'\s*(pub\s+)?trait\s+', line.strip()))

def is_const_line(line: str) -> bool:
    """Check if line starts a const."""
    return bool(re.match(r'\s*(pub\s+)?(spec\s+)?const\s+', line.strip()))

def is_spec_const_line(line: str) -> bool:
    """Check if line starts a spec const (goes to spec file)."""
    return bool(re.match(r'\s*(pub\s+)?spec\s+const\s+', line.strip()))

def extract_impl_header(lines: List[str], start_idx: int) -> str:
    """Extract impl header like 'impl Foo' or 'impl View for Foo'."""
    result = ""
    for i in range(start_idx, len(lines)):
        result += lines[i]
        if '{' in lines[i]:
            break
        result += " "
    # Extract just the impl ... part
    match = re.search(r'(impl\s+[^{]+)', result)
    if match:
        return match.group(1).strip()
    return result.split('{')[0].strip()

def split_verus_file(filepath: str):
    """Split a Verus file into exec, spec, and proof parts."""
    with open(filepath, 'r') as f:
        content = f.read()
    
    lines = content.split('\n')
    
    # Find verus! block boundaries
    verus_start = -1
    verus_end = -1
    for i, line in enumerate(lines):
        if 'verus!' in line and '{' in line:
            verus_start = i
            verus_end = find_block_end(lines, i)
            break
    
    if verus_start < 0:
        print(f"No verus! block found in {filepath}")
        return
    
    # Extract header (before verus!)
    header_lines = lines[:verus_start]
    
    # Extract footer (after verus!)
    footer_lines = lines[verus_end + 1:] if verus_end + 1 < len(lines) else []
    footer = '\n'.join(footer_lines)
    
    # Process content inside verus! block
    spec_parts = []
    proof_parts = []
    exec_parts = []
    
    i = verus_start + 1
    while i < verus_end:
        line = lines[i]
        stripped = line.strip()
        
        # Skip empty lines and comments at top level
        if stripped == '' or (stripped.startswith('//') and not stripped.startswith('//=')):
            i += 1
            continue
        
        # Handle impl blocks - split their contents
        if is_impl_line(line):
            impl_start = get_doc_comments_start(lines, i, verus_start + 1)
            impl_end = find_block_end(lines, i)
            impl_header = extract_impl_header(lines, i)
            
            # Find where impl body starts
            body_start = i
            for j in range(i, impl_end + 1):
                if '{' in lines[j]:
                    body_start = j + 1
                    break
            
            # Collect methods by type
            spec_methods = []
            proof_methods = []
            exec_methods = []
            
            j = body_start
            while j < impl_end:
                method_line = lines[j]
                method_stripped = method_line.strip()
                
                if method_stripped == '' or method_stripped.startswith('//'):
                    j += 1
                    continue
                
                if is_spec_fn_line(method_stripped):
                    fn_start = get_doc_comments_start(lines, j, body_start)
                    fn_end = find_function_end(lines, j)
                    spec_methods.append('\n'.join(lines[fn_start:fn_end + 1]))
                    j = fn_end + 1
                elif is_proof_fn_line(method_stripped):
                    fn_start = get_doc_comments_start(lines, j, body_start)
                    fn_end = find_function_end(lines, j)
                    proof_methods.append('\n'.join(lines[fn_start:fn_end + 1]))
                    j = fn_end + 1
                elif is_exec_fn_line(method_stripped):
                    fn_start = get_doc_comments_start(lines, j, body_start)
                    fn_end = find_function_end(lines, j)
                    exec_methods.append('\n'.join(lines[fn_start:fn_end + 1]))
                    j = fn_end + 1
                else:
                    j += 1
            
            # Add impl blocks to appropriate lists
            # Special case: if impl only has spec methods (like impl View), 
            # keep the entire impl block intact in spec file
            if spec_methods and not proof_methods and not exec_methods:
                # Keep entire impl block for spec (includes type aliases, etc.)
                spec_parts.append('\n'.join(lines[impl_start:impl_end + 1]))
            else:
                if spec_methods:
                    spec_parts.append(f"{impl_header} {{\n" + '\n\n'.join(spec_methods) + "\n}")
                if proof_methods:
                    proof_parts.append(f"{impl_header} {{\n" + '\n\n'.join(proof_methods) + "\n}")
                if exec_methods:
                    exec_parts.append(f"{impl_header} {{\n" + '\n\n'.join(exec_methods) + "\n}")
            
            i = impl_end + 1
        
        # Handle standalone spec functions
        elif is_spec_fn_line(stripped):
            fn_start = get_doc_comments_start(lines, i, verus_start + 1)
            fn_end = find_function_end(lines, i)
            spec_parts.append('\n'.join(lines[fn_start:fn_end + 1]))
            i = fn_end + 1
        
        # Handle standalone proof functions
        elif is_proof_fn_line(stripped):
            fn_start = get_doc_comments_start(lines, i, verus_start + 1)
            fn_end = find_function_end(lines, i)
            proof_parts.append('\n'.join(lines[fn_start:fn_end + 1]))
            i = fn_end + 1
        
        # Handle standalone exec functions
        elif is_exec_fn_line(stripped):
            fn_start = get_doc_comments_start(lines, i, verus_start + 1)
            fn_end = find_function_end(lines, i)
            exec_parts.append('\n'.join(lines[fn_start:fn_end + 1]))
            i = fn_end + 1
        
        # Handle structs, enums, consts - keep in exec
        # Exception: ghost struct goes to spec file
        elif is_struct_line(stripped) or stripped.startswith('#['):
            block_start = get_doc_comments_start(lines, i, verus_start + 1)
            # Check if next non-empty line is struct
            check_idx = i
            while check_idx < verus_end and lines[check_idx].strip().startswith('#['):
                check_idx += 1
            if check_idx < verus_end and is_struct_line(lines[check_idx]):
                block_end = find_block_end(lines, check_idx)
                struct_content = '\n'.join(lines[block_start:block_end + 1])
                # ghost struct goes to spec file
                if is_ghost_struct_line(lines[check_idx]):
                    spec_parts.append(struct_content)
                else:
                    exec_parts.append(struct_content)
                i = block_end + 1
            else:
                i += 1
        
        elif is_enum_line(stripped):
            block_start = get_doc_comments_start(lines, i, verus_start + 1)
            block_end = find_block_end(lines, i)
            exec_parts.append('\n'.join(lines[block_start:block_end + 1]))
            i = block_end + 1
        
        elif is_trait_line(stripped):
            # Trait definitions stay in exec file
            block_start = get_doc_comments_start(lines, i, verus_start + 1)
            block_end = find_block_end(lines, i)
            exec_parts.append('\n'.join(lines[block_start:block_end + 1]))
            i = block_end + 1
        
        elif is_const_line(stripped):
            const_start = get_doc_comments_start(lines, i, verus_start + 1)
            # Find end of const (semicolon)
            const_end = i
            while const_end < verus_end and ';' not in lines[const_end]:
                const_end += 1
            const_content = '\n'.join(lines[const_start:const_end + 1])
            # spec const goes to spec file
            if is_spec_const_line(lines[i]):
                spec_parts.append(const_content)
            else:
                exec_parts.append(const_content)
            i = const_end + 1
        
        else:
            i += 1
    
    # Generate output files
    base_path = filepath.rsplit('.', 1)[0]
    filename = os.path.basename(base_path)
    
    # Write spec file
    if spec_parts:
        spec_content = """// Copyright(c) The Maintainers of Nanvix.
// Licensed under the MIT License.

// Specifications.

verus! {

""" + '\n\n'.join(spec_parts) + "\n\n} // verus!\n"
        
        with open(f"{base_path}.spec.rs", 'w') as f:
            f.write(spec_content)
        print(f"Created: {base_path}.spec.rs")
    
    # Write proof file
    has_test_mod = '#[cfg(verus_keep_ghost)]' in footer
    if proof_parts or has_test_mod:
        proof_content = """// Copyright(c) The Maintainers of Nanvix.
// Licensed under the MIT License.

// Proofs and lemmas.

verus! {

"""
        if proof_parts:
            proof_content += '\n\n'.join(proof_parts)
        proof_content += "\n\n} // verus!\n"
        
        if has_test_mod:
            proof_content += '\n' + footer
        
        with open(f"{base_path}.proof.rs", 'w') as f:
            f.write(proof_content)
        print(f"Created: {base_path}.proof.rs")
    
    # Write main (exec) file
    header = '\n'.join(header_lines).rstrip()
    
    # Add includes
    if spec_parts:
        header += f"\n\n// Include specifications.\ninclude!(\"{filename}.spec.rs\");\n"
    if proof_parts or has_test_mod:
        header += f"\n// Include proofs.\ninclude!(\"{filename}.proof.rs\");\n"
    
    exec_content = header + "\n\nverus! {\n\n"
    exec_content += '\n\n'.join(exec_parts)
    exec_content += "\n\n} // verus!\n"
    
    # Add non-test footer
    if footer.strip() and not has_test_mod:
        exec_content += '\n' + footer
    
    with open(filepath, 'w') as f:
        f.write(exec_content)
    print(f"Updated: {filepath}")
    
    # Summary
    print(f"\nSummary:")
    print(f"  Spec parts: {len(spec_parts)}")
    print(f"  Proof parts: {len(proof_parts)}")
    print(f"  Exec parts: {len(exec_parts)}")

if __name__ == '__main__':
    if len(sys.argv) < 2:
        print("Usage: split_verus_v4.py <input_file>")
        sys.exit(1)
    
    split_verus_file(sys.argv[1])
