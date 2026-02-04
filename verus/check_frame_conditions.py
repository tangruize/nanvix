#!/usr/bin/env python3
"""
Detect verbose frame conditions in Verus postconditions.

This script finds places where multiple field equality conditions are used
in postconditions, which could be simplified using struct update syntax:

Before:
    &&& self@.field_a == old(self)@.field_a
    &&& self@.field_b == old(self)@.field_b
    &&& self@.field_c == old(self)@.field_c

After:
    self@ == (ViewType { field_that_changes: self@.field_that_changes, ..old(self)@ })

Usage:
    python3 check_frame_conditions.py [file.rs ...]
    
If no files specified, checks all .rs files in current directory.
"""

import re
import sys
from pathlib import Path
from collections import defaultdict
from dataclasses import dataclass
from typing import List, Dict, Tuple, Optional


@dataclass
class FrameCondition:
    """Represents a group of field equality conditions."""
    file: str
    start_line: int
    end_line: int
    function_name: str
    fields: List[str]
    lines: List[str]


def find_function_context(lines: List[str], line_idx: int) -> str:
    """Find the function name containing this line."""
    # Look backwards for fn definition
    for i in range(line_idx, -1, -1):
        line = lines[i]
        # Match: pub fn name, fn name, pub unsafe fn name, etc.
        match = re.search(r'\b(?:pub\s+)?(?:unsafe\s+)?fn\s+(\w+)', line)
        if match:
            return match.group(1)
    return "<unknown>"


def find_ensures_block(lines: List[str], line_idx: int) -> Tuple[int, int]:
    """Find the start and end of the ensures block containing this line."""
    start = line_idx
    end = line_idx
    
    # Look backwards for 'ensures'
    for i in range(line_idx, -1, -1):
        if 'ensures' in lines[i]:
            start = i
            break
        # Stop if we hit requires or fn
        if 'requires' in lines[i] or re.match(r'\s*(?:pub\s+)?(?:unsafe\s+)?fn\s+', lines[i]):
            start = line_idx
            break
    
    # Look forwards for end of ensures (next block or opening brace)
    brace_depth = 0
    for i in range(line_idx, len(lines)):
        line = lines[i]
        brace_depth += line.count('{') - line.count('}')
        # End at function body start or next section
        if (brace_depth > 0 and '{' in line) or 'requires' in line:
            end = i - 1
            break
        end = i
    
    return start, end


def detect_frame_conditions(file_path: str) -> List[FrameCondition]:
    """Detect verbose frame conditions in a file."""
    results = []
    
    try:
        with open(file_path, 'r') as f:
            content = f.read()
            lines = content.split('\n')
    except Exception as e:
        print(f"Error reading {file_path}: {e}", file=sys.stderr)
        return []
    
    # Pattern for field equality: self@.field == old(self)@.field
    # Also matches: self.field@ == old(self).field@, etc.
    field_eq_pattern = re.compile(
        r'self@?\.(\w+)@?\s*==\s*old\(self\)@?\.(\w+)@?'
    )
    
    # Alternative pattern: old(self)@.field == self@.field
    field_eq_pattern_rev = re.compile(
        r'old\(self\)@?\.(\w+)@?\s*==\s*self@?\.(\w+)@?'
    )
    
    # Track consecutive field equalities
    current_group: Dict[str, any] = {
        'fields': [],
        'start_line': -1,
        'end_line': -1,
        'lines': [],
    }
    
    in_ensures = False
    
    for i, line in enumerate(lines):
        # Track if we're in an ensures block
        if 'ensures' in line:
            in_ensures = True
        elif re.match(r'\s*\{', line.strip()) and in_ensures:
            # End of ensures block (function body starts)
            in_ensures = False
            # Flush current group
            if len(current_group['fields']) >= 2:
                fn_name = find_function_context(lines, current_group['start_line'])
                results.append(FrameCondition(
                    file=file_path,
                    start_line=current_group['start_line'] + 1,
                    end_line=current_group['end_line'] + 1,
                    function_name=fn_name,
                    fields=current_group['fields'],
                    lines=current_group['lines'],
                ))
            current_group = {'fields': [], 'start_line': -1, 'end_line': -1, 'lines': []}
        
        if not in_ensures:
            continue
        
        # Check for field equality patterns
        match = field_eq_pattern.search(line)
        if not match:
            match = field_eq_pattern_rev.search(line)
        
        if match:
            field1, field2 = match.group(1), match.group(2)
            # Only count if same field on both sides
            if field1 == field2:
                if current_group['start_line'] == -1:
                    current_group['start_line'] = i
                current_group['end_line'] = i
                current_group['fields'].append(field1)
                current_group['lines'].append(line.strip())
        else:
            # Non-matching line in ensures - check if we should flush
            # Only flush if line has actual condition (not just whitespace or &&)
            stripped = line.strip()
            if stripped and stripped not in ['&&', '&&&', '']:
                if len(current_group['fields']) >= 2:
                    fn_name = find_function_context(lines, current_group['start_line'])
                    results.append(FrameCondition(
                        file=file_path,
                        start_line=current_group['start_line'] + 1,
                        end_line=current_group['end_line'] + 1,
                        function_name=fn_name,
                        fields=current_group['fields'],
                        lines=current_group['lines'],
                    ))
                current_group = {'fields': [], 'start_line': -1, 'end_line': -1, 'lines': []}
    
    # Flush any remaining group
    if len(current_group['fields']) >= 2:
        fn_name = find_function_context(lines, current_group['start_line'])
        results.append(FrameCondition(
            file=file_path,
            start_line=current_group['start_line'] + 1,
            end_line=current_group['end_line'] + 1,
            function_name=fn_name,
            fields=current_group['fields'],
            lines=current_group['lines'],
        ))
    
    return results


def detect_static_fields_unchanged(file_path: str) -> List[Tuple[int, str, str]]:
    """Detect uses of static_fields_unchanged helper."""
    results = []
    
    try:
        with open(file_path, 'r') as f:
            lines = f.readlines()
    except Exception as e:
        print(f"Error reading {file_path}: {e}", file=sys.stderr)
        return []
    
    for i, line in enumerate(lines):
        if 'static_fields_unchanged' in line:
            fn_name = find_function_context([l.rstrip() for l in lines], i)
            results.append((i + 1, fn_name, line.strip()))
    
    return results


def main():
    if len(sys.argv) > 1:
        files = [Path(f) for f in sys.argv[1:]]
    else:
        files = list(Path('.').glob('*.rs'))
    
    all_conditions = []
    all_helper_uses = []
    
    for file_path in files:
        if not file_path.exists():
            print(f"File not found: {file_path}", file=sys.stderr)
            continue
        
        conditions = detect_frame_conditions(str(file_path))
        all_conditions.extend(conditions)
        
        helper_uses = detect_static_fields_unchanged(str(file_path))
        all_helper_uses.extend([(str(file_path), *u) for u in helper_uses])
    
    # Report verbose frame conditions
    print("=" * 80)
    print("VERBOSE FRAME CONDITIONS (can be simplified with struct update syntax)")
    print("=" * 80)
    print()
    
    if not all_conditions:
        print("  No verbose frame conditions found.")
    else:
        # Group by file
        by_file = defaultdict(list)
        for cond in all_conditions:
            by_file[cond.file].append(cond)
        
        for file_path, conditions in sorted(by_file.items()):
            print(f"📁 {file_path}")
            print()
            for cond in conditions:
                print(f"  Function: {cond.function_name}()")
                print(f"  Lines: {cond.start_line}-{cond.end_line}")
                print(f"  Fields unchanged: {', '.join(cond.fields)} ({len(cond.fields)} fields)")
                print()
                print("  Current code:")
                for line in cond.lines[:5]:  # Show max 5 lines
                    print(f"    {line}")
                if len(cond.lines) > 5:
                    print(f"    ... and {len(cond.lines) - 5} more lines")
                print()
                print("  Suggested replacement:")
                print(f"    self@ == (ViewType {{ changed_field: self@.changed_field, ..old(self)@ }})")
                print()
                print("-" * 60)
                print()
    
    # Report uses of static_fields_unchanged helper
    print()
    print("=" * 80)
    print("USES OF static_fields_unchanged HELPER (can also be simplified)")
    print("=" * 80)
    print()
    
    if not all_helper_uses:
        print("  No uses of static_fields_unchanged found.")
    else:
        for file_path, line_no, fn_name, line_content in all_helper_uses:
            print(f"  {file_path}:{line_no} in {fn_name}()")
            print(f"    {line_content}")
            print()
    
    # Summary
    print()
    print("=" * 80)
    print("SUMMARY")
    print("=" * 80)
    total_fields = sum(len(c.fields) for c in all_conditions)
    print(f"  Verbose frame conditions: {len(all_conditions)} locations, {total_fields} total field comparisons")
    print(f"  Helper function uses: {len(all_helper_uses)} locations")
    print()
    print("To simplify, replace patterns like:")
    print("    &&& self@.field_a == old(self)@.field_a")
    print("    &&& self@.field_b == old(self)@.field_b")
    print()
    print("With struct update syntax:")
    print("    self@ == (ViewType { changed_field: self@.changed_field, ..old(self)@ })")
    print()


if __name__ == '__main__':
    main()
