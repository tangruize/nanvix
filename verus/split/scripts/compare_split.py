#!/usr/bin/env python3
"""
Compare functions between original src files and split verus exec files.

This script extracts function names from both files and shows:
1. Functions in both (should match for exec functions)
2. Functions only in src (missing from exec - should be there)
3. Functions only in exec (extra - might be spec/proof leaked in)
"""

import re
import sys
import os

def extract_functions(filepath):
    """Extract function names from a Rust file."""
    if not os.path.exists(filepath):
        return set(), {}
    
    with open(filepath, 'r') as f:
        content = f.read()
    
    functions = {}
    
    # Match function definitions
    # Patterns: pub fn, fn, pub unsafe fn, unsafe fn, pub const fn, etc.
    # But NOT spec fn, proof fn, open spec fn, closed spec fn
    pattern = r'^\s*(pub\s+)?(unsafe\s+)?(const\s+)?fn\s+(\w+)'
    
    for i, line in enumerate(content.split('\n'), 1):
        # Skip spec and proof functions
        if 'spec fn' in line or 'proof fn' in line:
            continue
        
        match = re.match(pattern, line)
        if match:
            fn_name = match.group(4)
            functions[fn_name] = i
    
    return set(functions.keys()), functions

def extract_all_functions(filepath):
    """Extract ALL function names including spec and proof."""
    if not os.path.exists(filepath):
        return {}, {}, {}
    
    with open(filepath, 'r') as f:
        content = f.read()
    
    exec_fns = {}
    spec_fns = {}
    proof_fns = {}
    
    for i, line in enumerate(content.split('\n'), 1):
        # Spec functions
        match = re.match(r'^\s*(pub\s+)?(open\s+|closed\s+)?spec\s+fn\s+(\w+)', line)
        if match:
            spec_fns[match.group(3)] = i
            continue
        
        # Proof functions
        match = re.match(r'^\s*(pub\s+)?proof\s+fn\s+(\w+)', line)
        if match:
            proof_fns[match.group(2)] = i
            continue
        
        # Exec functions (regular fn)
        match = re.match(r'^\s*(pub\s+)?(unsafe\s+)?(const\s+)?fn\s+(\w+)', line)
        if match:
            exec_fns[match.group(4)] = i
    
    return exec_fns, spec_fns, proof_fns

def compare_files(src_path, exec_path, spec_path=None, proof_path=None):
    """Compare functions between src and split files."""
    print(f"=== Comparing ===")
    print(f"Source: {src_path}")
    print(f"Exec:   {exec_path}")
    if spec_path:
        print(f"Spec:   {spec_path}")
    if proof_path:
        print(f"Proof:  {proof_path}")
    print()
    
    # Extract from source (only exec functions)
    src_fns, src_details = extract_functions(src_path)
    
    # Extract from exec file
    exec_exec_fns, exec_spec_fns, exec_proof_fns = extract_all_functions(exec_path)
    
    # Extract from spec/proof files if provided
    spec_exec_fns, spec_spec_fns, spec_proof_fns = {}, {}, {}
    proof_exec_fns, proof_spec_fns, proof_proof_fns = {}, {}, {}
    
    if spec_path and os.path.exists(spec_path):
        spec_exec_fns, spec_spec_fns, spec_proof_fns = extract_all_functions(spec_path)
    
    if proof_path and os.path.exists(proof_path):
        proof_exec_fns, proof_spec_fns, proof_proof_fns = extract_all_functions(proof_path)
    
    # Analysis
    print("--- Source file (exec functions only) ---")
    print(f"Total exec functions: {len(src_fns)}")
    if src_fns:
        print(f"Functions: {sorted(src_fns)}")
    print()
    
    print("--- Split exec file ---")
    print(f"Exec functions: {len(exec_exec_fns)}")
    print(f"Spec functions (should be 0): {len(exec_spec_fns)}")
    print(f"Proof functions (should be 0): {len(exec_proof_fns)}")
    
    if exec_spec_fns:
        print(f"  WARNING: spec fns in exec: {list(exec_spec_fns.keys())}")
    if exec_proof_fns:
        print(f"  WARNING: proof fns in exec: {list(exec_proof_fns.keys())}")
    print()
    
    if spec_path:
        print("--- Split spec file ---")
        print(f"Spec functions: {len(spec_spec_fns)}")
        print(f"Exec functions (should be 0): {len(spec_exec_fns)}")
        print(f"Proof functions (should be 0): {len(spec_proof_fns)}")
        if spec_exec_fns:
            print(f"  WARNING: exec fns in spec: {list(spec_exec_fns.keys())}")
        if spec_proof_fns:
            print(f"  WARNING: proof fns in spec: {list(spec_proof_fns.keys())}")
        print()
    
    if proof_path:
        print("--- Split proof file ---")
        print(f"Proof functions: {len(proof_proof_fns)}")
        print(f"Exec functions (should be 0): {len(proof_exec_fns)}")
        print(f"Spec functions (should be 0): {len(proof_spec_fns)}")
        if proof_exec_fns:
            print(f"  WARNING: exec fns in proof: {list(proof_exec_fns.keys())}")
        if proof_spec_fns:
            print(f"  WARNING: spec fns in proof: {list(proof_spec_fns.keys())}")
        print()
    
    # Compare src exec functions with split exec functions
    exec_fn_set = set(exec_exec_fns.keys())
    
    in_both = src_fns & exec_fn_set
    only_in_src = src_fns - exec_fn_set
    only_in_exec = exec_fn_set - src_fns
    
    print("--- Comparison (src exec vs split exec) ---")
    print(f"Matching: {len(in_both)}")
    
    if only_in_src:
        print(f"\nMissing from exec (in src but not in split exec): {len(only_in_src)}")
        for fn in sorted(only_in_src):
            print(f"  - {fn} (src line {src_details.get(fn, '?')})")
    
    if only_in_exec:
        print(f"\nExtra in exec (in split exec but not in src): {len(only_in_exec)}")
        for fn in sorted(only_in_exec):
            print(f"  + {fn} (exec line {exec_exec_fns.get(fn, '?')})")
    
    # Summary
    print()
    print("--- Summary ---")
    issues = []
    if only_in_src:
        issues.append(f"{len(only_in_src)} missing from exec")
    if only_in_exec:
        issues.append(f"{len(only_in_exec)} extra in exec")
    if exec_spec_fns:
        issues.append(f"{len(exec_spec_fns)} spec fns leaked to exec")
    if exec_proof_fns:
        issues.append(f"{len(exec_proof_fns)} proof fns leaked to exec")
    if spec_exec_fns:
        issues.append(f"{len(spec_exec_fns)} exec fns in spec")
    if proof_exec_fns:
        issues.append(f"{len(proof_exec_fns)} exec fns in proof")
    
    if issues:
        print(f"ISSUES: {', '.join(issues)}")
        return False
    else:
        print("OK - Split looks correct!")
        return True

def main():
    if len(sys.argv) < 3:
        print("Usage: compare_split.py <src_file> <exec_file> [spec_file] [proof_file]")
        print()
        print("Example:")
        print("  compare_split.py src/kernel/src/mm/phys/frame.rs verus/split/kernel/mm/phys/frame.rs")
        sys.exit(1)
    
    src_path = sys.argv[1]
    exec_path = sys.argv[2]
    spec_path = sys.argv[3] if len(sys.argv) > 3 else None
    proof_path = sys.argv[4] if len(sys.argv) > 4 else None
    
    # Auto-detect spec/proof paths
    if spec_path is None:
        base = exec_path.rsplit('.', 1)[0]
        potential_spec = f"{base}.spec.rs"
        if os.path.exists(potential_spec):
            spec_path = potential_spec
    
    if proof_path is None:
        base = exec_path.rsplit('.', 1)[0]
        potential_proof = f"{base}.proof.rs"
        if os.path.exists(potential_proof):
            proof_path = potential_proof
    
    success = compare_files(src_path, exec_path, spec_path, proof_path)
    sys.exit(0 if success else 1)

if __name__ == '__main__':
    main()
