#!/usr/bin/env python3
"""
Check for missing error! logging in Verus ports compared to original Nanvix code.

This script:
1. Maps Verus modules to their corresponding Nanvix source files
2. Finds all error! macro calls in the Nanvix source
3. Checks if corresponding error logging exists in the Verus version
4. Reports any missing error logging
"""

import re
import os
from pathlib import Path
from dataclasses import dataclass
from typing import Optional


@dataclass
class ErrorCall:
    """Represents an error! macro call in source code."""
    file: str
    line_num: int
    function: str
    code: str
    context: str  # surrounding lines for context


@dataclass
class VerusErrorCheck:
    """Result of checking if error logging exists in Verus."""
    has_logging: bool
    verus_file: str
    verus_line: Optional[int]
    note: str


# Mapping from Verus module to Nanvix source file(s)
MODULE_MAPPING = {
    "frame.rs": [
        "src/kernel/src/mm/phys/frame.rs",
    ],
    "bitmap.rs": [
        "src/libs/bitmap/src/lib.rs",
    ],
    # Add more mappings as needed
}

# Function name mapping (Nanvix -> Verus alternatives)
FUNCTION_MAPPING = {
    "alloc": ["alloc", "alloc_index"],
    "free": ["free"],
    "book": ["book"],
    "alloc_range": ["alloc_range", "alloc_range_unchecked", "alloc_range_from_region", "alloc_range_inner"],
}


def find_error_calls(file_path: str) -> list[ErrorCall]:
    """Find all error! macro calls in a file."""
    if not os.path.exists(file_path):
        print(f"  Warning: File not found: {file_path}")
        return []
    
    with open(file_path, 'r') as f:
        lines = f.readlines()
    
    error_calls = []
    current_function = "<unknown>"
    
    for i, line in enumerate(lines):
        # Track current function (simple heuristic)
        fn_match = re.search(r'(?:pub\s+)?fn\s+(\w+)', line)
        if fn_match:
            current_function = fn_match.group(1)
        
        # Find error! calls
        if 'error!' in line:
            # Get context (3 lines before and after)
            start = max(0, i - 3)
            end = min(len(lines), i + 4)
            context = ''.join(lines[start:end])
            
            error_calls.append(ErrorCall(
                file=file_path,
                line_num=i + 1,
                function=current_function,
                code=line.strip(),
                context=context
            ))
    
    return error_calls


def check_verus_has_error_logging(verus_file: str, function_name: str) -> VerusErrorCheck:
    """Check if a Verus function has error logging."""
    if not os.path.exists(verus_file):
        return VerusErrorCheck(
            has_logging=False,
            verus_file=verus_file,
            verus_line=None,
            note="Verus file not found"
        )
    
    with open(verus_file, 'r') as f:
        content = f.read()
    
    # Find where this function starts
    fn_pattern = rf'pub\s+fn\s+{re.escape(function_name)}\s*[\(<&]'
    fn_match = re.search(fn_pattern, content)
    
    if not fn_match:
        return VerusErrorCheck(
            has_logging=False,
            verus_file=verus_file,
            verus_line=None,
            note=f"Function '{function_name}' not found"
        )
    
    fn_start_pos = fn_match.start()
    fn_line_num = content[:fn_start_pos].count('\n') + 1
    
    # Get the function body (everything until next pub fn)
    remaining = content[fn_start_pos:]
    next_fn_match = re.search(r'\n\s*pub\s+fn\s+\w+', remaining[100:])  # skip current fn header
    if next_fn_match:
        fn_body = remaining[:next_fn_match.start() + 100]
    else:
        fn_body = remaining[:3000]  # just take next 3000 chars
    
    # Check for error logging patterns
    if '.log()' in fn_body:
        log_pos = fn_body.find('.log()')
        log_line = content[:fn_start_pos + log_pos].count('\n') + 1
        return VerusErrorCheck(
            has_logging=True,
            verus_file=verus_file,
            verus_line=log_line,
            note="Found .log() call"
        )
    
    if 'NOTE: Original Nanvix code has: error!' in fn_body:
        return VerusErrorCheck(
            has_logging=True,
            verus_file=verus_file,
            verus_line=fn_line_num,
            note="Found NOTE comment about error logging"
        )
    
    return VerusErrorCheck(
        has_logging=False,
        verus_file=verus_file,
        verus_line=None,
        note=f"No error logging found (fn at line {fn_line_num})"
    )


def analyze_module(verus_module: str, nanvix_sources: list[str], verus_dir: str, nanvix_root: str):
    """Analyze a module for missing error logging."""
    print(f"\n{'='*80}")
    print(f"Analyzing: {verus_module}")
    print(f"{'='*80}")
    
    verus_path = os.path.join(verus_dir, verus_module)
    
    all_error_calls = []
    for source in nanvix_sources:
        source_path = os.path.join(nanvix_root, source)
        error_calls = find_error_calls(source_path)
        all_error_calls.extend(error_calls)
    
    if not all_error_calls:
        print(f"  No error! calls found in Nanvix source(s)")
        return [], []
    
    print(f"\n  Found {len(all_error_calls)} error! call(s) in Nanvix source:")
    
    missing = []
    present = []
    
    for ec in all_error_calls:
        print(f"\n  [{ec.line_num}] In function '{ec.function}':")
        print(f"      {ec.code}")
        
        # Get alternative function names to check
        functions_to_check = FUNCTION_MAPPING.get(ec.function, [ec.function])
        
        # Check Verus version for each possible function name
        found = False
        found_check = None
        for fn_name in functions_to_check:
            check = check_verus_has_error_logging(verus_path, fn_name)
            if check.has_logging:
                found = True
                found_check = check
                found_check.note = f"{found_check.note} (in {fn_name})"
                break
        
        if found and found_check:
            print(f"      ✓ Verus has logging at line {found_check.verus_line}: {found_check.note}")
            present.append((ec, found_check))
        else:
            print(f"      ✗ MISSING in Verus: checked functions {functions_to_check}")
            missing.append((ec, check))
    
    return missing, present


def main():
    # Determine paths
    script_dir = Path(__file__).parent
    verus_dir = str(script_dir)
    nanvix_root = str(script_dir.parent)
    
    print("=" * 80)
    print("Error Logging Check: Nanvix vs Verus")
    print("=" * 80)
    print(f"Verus dir: {verus_dir}")
    print(f"Nanvix root: {nanvix_root}")
    
    total_missing = []
    total_present = []
    
    for verus_module, nanvix_sources in MODULE_MAPPING.items():
        missing, present = analyze_module(verus_module, nanvix_sources, verus_dir, nanvix_root)
        total_missing.extend(missing)
        total_present.extend(present)
    
    # Summary
    print(f"\n{'='*80}")
    print("SUMMARY")
    print(f"{'='*80}")
    print(f"Total error! calls in Nanvix: {len(total_missing) + len(total_present)}")
    print(f"  ✓ Present in Verus: {len(total_present)}")
    print(f"  ✗ Missing in Verus: {len(total_missing)}")
    
    if total_missing:
        print(f"\n{'='*80}")
        print("MISSING ERROR LOGGING (needs to be added):")
        print(f"{'='*80}")
        for ec, check in total_missing:
            print(f"\n  File: {ec.file}")
            print(f"  Line: {ec.line_num}")
            print(f"  Function: {ec.function}")
            print(f"  Code: {ec.code}")
            print(f"  Verus file: {check.verus_file}")
    
    return len(total_missing)


if __name__ == "__main__":
    exit(main())
