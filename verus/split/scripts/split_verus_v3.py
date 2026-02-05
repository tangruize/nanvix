#!/usr/bin/env python3
"""
Verus code splitter - splits a verus file into exec, spec, and proof files.

Rules:
- spec fn, open spec fn, closed spec fn -> spec file
- proof fn -> proof file  
- pub fn, fn (without spec/proof) -> exec file (stays in main)
- Structs, enums, constants, use statements -> stay in exec (main file)
- impl blocks are split: spec fn methods go to spec, proof fn methods go to proof
"""

import re
import sys
from dataclasses import dataclass
from typing import List, Tuple, Optional

@dataclass
class CodeBlock:
    """Represents a block of code with its type and content."""
    block_type: str  # 'spec', 'proof', 'exec', 'struct', 'const', 'use', 'impl_header', 'other'
    content: str
    start_line: int
    end_line: int
    impl_context: Optional[str] = None  # For methods, which impl block they belong to

def find_matching_brace(lines: List[str], start_idx: int, start_col: int = 0) -> int:
    """Find the line index of the matching closing brace."""
    depth = 0
    found_first = False
    
    for i in range(start_idx, len(lines)):
        line = lines[i] if i > start_idx else lines[i][start_col:]
        for char in line:
            if char == '{':
                depth += 1
                found_first = True
            elif char == '}':
                depth -= 1
                if found_first and depth == 0:
                    return i
    return len(lines) - 1

def is_spec_fn(line: str) -> bool:
    """Check if line starts a spec function."""
    patterns = [
        r'^\s*(pub\s+)?(open\s+|closed\s+)?spec\s+fn\s+',
    ]
    return any(re.match(p, line) for p in patterns)

def is_proof_fn(line: str) -> bool:
    """Check if line starts a proof function."""
    return bool(re.match(r'^\s*(pub\s+)?proof\s+fn\s+', line))

def is_exec_fn(line: str) -> bool:
    """Check if line starts an exec function (regular fn without spec/proof)."""
    # Match fn but not spec fn or proof fn
    if re.match(r'^\s*(pub\s+)?fn\s+', line):
        if not is_spec_fn(line) and not is_proof_fn(line):
            return True
    return False

def is_impl_start(line: str) -> bool:
    """Check if line starts an impl block."""
    return bool(re.match(r'^\s*impl\s+', line))

def is_struct_start(line: str) -> bool:
    """Check if line starts a struct definition."""
    return bool(re.match(r'^\s*(pub\s+)?struct\s+', line))

def is_enum_start(line: str) -> bool:
    """Check if line starts an enum definition."""
    return bool(re.match(r'^\s*(pub\s+)?enum\s+', line))

def is_const_start(line: str) -> bool:
    """Check if line starts a const definition."""
    return bool(re.match(r'^\s*(pub\s+)?const\s+', line))

def is_type_alias(line: str) -> bool:
    """Check if line is a type alias."""
    return bool(re.match(r'^\s*(pub\s+)?type\s+', line))

def get_impl_header(lines: List[str], start_idx: int) -> str:
    """Extract the impl header (e.g., 'impl View for Foo' or 'impl Foo')."""
    line = lines[start_idx].strip()
    # Extract until we hit {
    match = re.match(r'(impl\s+.*?)\s*\{', line)
    if match:
        return match.group(1).strip()
    # Multi-line impl header
    header = line
    for i in range(start_idx + 1, len(lines)):
        header += ' ' + lines[i].strip()
        if '{' in lines[i]:
            match = re.match(r'(impl\s+.*?)\s*\{', header)
            if match:
                return match.group(1).strip()
            break
    return header.split('{')[0].strip()

def extract_function(lines: List[str], start_idx: int) -> Tuple[str, int]:
    """Extract a complete function including doc comments and attributes."""
    # Look back for doc comments and attributes
    actual_start = start_idx
    while actual_start > 0:
        prev_line = lines[actual_start - 1].strip()
        if prev_line.startswith('///') or prev_line.startswith('#[') or prev_line.startswith('//='):
            actual_start -= 1
        elif prev_line == '':
            actual_start -= 1
        else:
            break
    
    # Find the end of the function
    end_idx = find_matching_brace(lines, start_idx)
    
    # Extract the content
    content = '\n'.join(lines[actual_start:end_idx + 1])
    return content, end_idx

def split_impl_block(lines: List[str], start_idx: int) -> Tuple[List[CodeBlock], int]:
    """Split an impl block into spec, proof, and exec parts."""
    blocks = []
    impl_header = get_impl_header(lines, start_idx)
    end_idx = find_matching_brace(lines, start_idx)
    
    # Find where the impl body starts
    body_start = start_idx
    for i in range(start_idx, end_idx + 1):
        if '{' in lines[i]:
            body_start = i + 1
            break
    
    spec_methods = []
    proof_methods = []
    exec_methods = []
    
    i = body_start
    while i < end_idx:
        line = lines[i]
        stripped = line.strip()
        
        if stripped == '' or stripped.startswith('//'):
            i += 1
            continue
        
        # Check for section comments (keep with next method)
        if stripped.startswith('//=='):
            # Look ahead for the next function
            section_start = i
            i += 1
            while i < end_idx and (lines[i].strip() == '' or lines[i].strip().startswith('//')):
                i += 1
            if i < end_idx:
                # Include section comment with the function
                continue
            else:
                i = section_start + 1
                continue
        
        if is_spec_fn(stripped):
            content, fn_end = extract_function(lines, i)
            spec_methods.append(content)
            i = fn_end + 1
        elif is_proof_fn(stripped):
            content, fn_end = extract_function(lines, i)
            proof_methods.append(content)
            i = fn_end + 1
        elif is_exec_fn(stripped):
            content, fn_end = extract_function(lines, i)
            exec_methods.append(content)
            i = fn_end + 1
        else:
            i += 1
    
    if spec_methods:
        spec_content = f"{impl_header} {{\n" + '\n\n'.join(spec_methods) + "\n}"
        blocks.append(CodeBlock('spec', spec_content, start_idx, end_idx, impl_header))
    
    if proof_methods:
        proof_content = f"{impl_header} {{\n" + '\n\n'.join(proof_methods) + "\n}"
        blocks.append(CodeBlock('proof', proof_content, start_idx, end_idx, impl_header))
    
    if exec_methods:
        exec_content = f"{impl_header} {{\n" + '\n\n'.join(exec_methods) + "\n}"
        blocks.append(CodeBlock('exec', exec_content, start_idx, end_idx, impl_header))
    
    return blocks, end_idx

def parse_verus_file(content: str) -> Tuple[str, List[CodeBlock], str]:
    """Parse a verus file and extract code blocks."""
    lines = content.split('\n')
    
    # Find verus! block
    verus_start = -1
    verus_end = -1
    for i, line in enumerate(lines):
        if 'verus!' in line and '{' in line:
            verus_start = i
            verus_end = find_matching_brace(lines, i)
            break
    
    if verus_start < 0:
        return content, [], ""
    
    # Extract header (before verus!)
    header = '\n'.join(lines[:verus_start])
    
    # Extract footer (after verus!)
    footer = '\n'.join(lines[verus_end + 1:]) if verus_end + 1 < len(lines) else ""
    
    blocks = []
    i = verus_start + 1  # Skip the verus! { line
    
    while i < verus_end:
        line = lines[i]
        stripped = line.strip()
        
        if stripped == '' or stripped.startswith('//'):
            i += 1
            continue
        
        # Handle standalone spec/proof functions
        if is_spec_fn(stripped):
            content, fn_end = extract_function(lines, i)
            blocks.append(CodeBlock('spec', content, i, fn_end))
            i = fn_end + 1
        elif is_proof_fn(stripped):
            content, fn_end = extract_function(lines, i)
            blocks.append(CodeBlock('proof', content, i, fn_end))
            i = fn_end + 1
        elif is_impl_start(stripped):
            impl_blocks, impl_end = split_impl_block(lines, i)
            blocks.extend(impl_blocks)
            i = impl_end + 1
        elif is_struct_start(stripped) or is_enum_start(stripped):
            # Find end of struct/enum
            end = find_matching_brace(lines, i)
            # Look back for attributes and doc comments
            actual_start = i
            while actual_start > verus_start and (
                lines[actual_start - 1].strip().startswith('///') or
                lines[actual_start - 1].strip().startswith('#[') or
                lines[actual_start - 1].strip().startswith('//==') or
                lines[actual_start - 1].strip() == ''
            ):
                actual_start -= 1
            content = '\n'.join(lines[actual_start:end + 1])
            blocks.append(CodeBlock('struct', content, actual_start, end))
            i = end + 1
        elif is_const_start(stripped):
            # Single line const
            end = i
            while end < verus_end and ';' not in lines[end]:
                end += 1
            # Look back for doc comments
            actual_start = i
            while actual_start > verus_start and (
                lines[actual_start - 1].strip().startswith('///') or
                lines[actual_start - 1].strip().startswith('//==') or
                lines[actual_start - 1].strip() == ''
            ):
                actual_start -= 1
            content = '\n'.join(lines[actual_start:end + 1])
            blocks.append(CodeBlock('const', content, actual_start, end))
            i = end + 1
        elif is_type_alias(stripped):
            end = i
            while end < verus_end and ';' not in lines[end]:
                end += 1
            content = '\n'.join(lines[i:end + 1])
            blocks.append(CodeBlock('type', content, i, end))
            i = end + 1
        elif is_exec_fn(stripped):
            content, fn_end = extract_function(lines, i)
            blocks.append(CodeBlock('exec', content, i, fn_end))
            i = fn_end + 1
        else:
            i += 1
    
    return header, blocks, footer

def generate_spec_file(blocks: List[CodeBlock]) -> str:
    """Generate the spec file content."""
    spec_blocks = [b for b in blocks if b.block_type == 'spec']
    if not spec_blocks:
        return ""
    
    content = """// Copyright(c) The Maintainers of Nanvix.
// Licensed under the MIT License.

// Specifications.

verus! {

"""
    content += '\n\n'.join(b.content for b in spec_blocks)
    content += "\n\n} // verus!\n"
    return content

def generate_proof_file(blocks: List[CodeBlock], footer: str) -> str:
    """Generate the proof file content."""
    proof_blocks = [b for b in blocks if b.block_type == 'proof']
    if not proof_blocks and '#[cfg(verus_keep_ghost)]' not in footer:
        return ""
    
    content = """// Copyright(c) The Maintainers of Nanvix.
// Licensed under the MIT License.

// Proofs and lemmas.

verus! {

"""
    if proof_blocks:
        content += '\n\n'.join(b.content for b in proof_blocks)
    content += "\n\n} // verus!\n"
    
    # Add test module if present in footer
    if '#[cfg(verus_keep_ghost)]' in footer:
        content += '\n' + footer
    
    return content

def generate_exec_file(header: str, blocks: List[CodeBlock], footer: str) -> str:
    """Generate the exec file content (main file with includes)."""
    exec_blocks = [b for b in blocks if b.block_type in ('exec', 'struct', 'const', 'type')]
    
    # Check if we need spec and proof files
    has_spec = any(b.block_type == 'spec' for b in blocks)
    has_proof = any(b.block_type == 'proof' for b in blocks) or '#[cfg(verus_keep_ghost)]' in footer
    
    # Build the new header with includes (outside verus! block)
    new_header = header.rstrip()
    
    if has_spec or has_proof:
        new_header += "\n"
        if has_spec:
            new_header += "\n// Include specifications.\n"
            new_header += 'include!("{filename}.spec.rs");\n'
        if has_proof:
            new_header += "\n// Include proofs.\n"  
            new_header += 'include!("{filename}.proof.rs");\n'
    
    content = new_header + "\n\nverus! {\n\n"
    content += '\n\n'.join(b.content for b in exec_blocks)
    content += "\n\n} // verus!\n"
    
    # Add footer if it doesn't have test module (that goes to proof file)
    if footer.strip() and '#[cfg(verus_keep_ghost)]' not in footer:
        content += '\n' + footer
    
    return content

def split_file(input_path: str, output_prefix: str = None):
    """Split a verus file into exec, spec, and proof files."""
    with open(input_path, 'r') as f:
        content = f.read()
    
    if output_prefix is None:
        output_prefix = input_path.rsplit('.', 1)[0]
    
    # Get just the filename for include statements
    import os
    filename = os.path.basename(output_prefix)
    
    header, blocks, footer = parse_verus_file(content)
    
    # Generate files
    spec_content = generate_spec_file(blocks)
    proof_content = generate_proof_file(blocks, footer)
    exec_content = generate_exec_file(header, blocks, footer)
    
    # Replace placeholder with actual filename
    exec_content = exec_content.replace('{filename}', filename)
    
    # Write files
    if spec_content:
        with open(f"{output_prefix}.spec.rs", 'w') as f:
            f.write(spec_content)
        print(f"Created: {output_prefix}.spec.rs")
    
    if proof_content:
        with open(f"{output_prefix}.proof.rs", 'w') as f:
            f.write(proof_content)
        print(f"Created: {output_prefix}.proof.rs")
    
    with open(f"{output_prefix}.rs", 'w') as f:
        f.write(exec_content)
    print(f"Updated: {output_prefix}.rs")
    
    # Summary
    spec_count = len([b for b in blocks if b.block_type == 'spec'])
    proof_count = len([b for b in blocks if b.block_type == 'proof'])
    exec_count = len([b for b in blocks if b.block_type == 'exec'])
    struct_count = len([b for b in blocks if b.block_type == 'struct'])
    
    print(f"\nSummary:")
    print(f"  Spec blocks: {spec_count}")
    print(f"  Proof blocks: {proof_count}")
    print(f"  Exec blocks: {exec_count}")
    print(f"  Struct/Enum blocks: {struct_count}")

if __name__ == '__main__':
    if len(sys.argv) < 2:
        print("Usage: split_verus_v3.py <input_file> [output_prefix]")
        sys.exit(1)
    
    input_file = sys.argv[1]
    output_prefix = sys.argv[2] if len(sys.argv) > 2 else None
    split_file(input_file, output_prefix)
