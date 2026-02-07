#!/usr/bin/env python3
# Copyright(c) The Maintainers of Nanvix.
# Licensed under the MIT License.

"""
Find repeated expression patterns that could benefit from being extracted
into delegate/alias spec functions.

Looks for patterns like:
1. self.field.method() - repeated chained calls
2. self@.field.method() - repeated view access chains
3. expr / CONSTANT or expr % CONSTANT - repeated computations
4. Complex boolean expressions - repeated predicates

Usage:
    ./find_repeated_patterns.py [directory]
    Default: current directory
"""

import re
import sys
from pathlib import Path
from collections import defaultdict
from dataclasses import dataclass
from typing import Optional


@dataclass
class PatternMatch:
    """Represents a found pattern."""
    pattern: str
    file: str
    line_num: int
    context: str  # surrounding code


def normalize_pattern(pattern: str) -> str:
    """Normalize a pattern for comparison."""
    # Remove whitespace
    pattern = ' '.join(pattern.split())
    # Normalize variable names to placeholders for some patterns
    return pattern


def extract_patterns(file_path: Path) -> list[PatternMatch]:
    """Extract potentially repeated patterns from a file."""
    content = file_path.read_text()
    lines = content.split('\n')
    patterns = []
    
    for i, line in enumerate(lines):
        line_num = i + 1
        
        # Skip comments and string literals (simple heuristic)
        stripped = line.strip()
        if stripped.startswith('//') or stripped.startswith('///'):
            continue
        
        # Pattern 1: self.field.method() or self@.field.method()
        for match in re.finditer(r'self(@)?\.(\w+)\.(\w+)\(\s*\)', line):
            view = "@" if match.group(1) else ""
            field = match.group(2)
            method = match.group(3)
            pattern = f"self{view}.{field}.{method}()"
            patterns.append(PatternMatch(pattern, str(file_path), line_num, stripped))
        
        # Pattern 2: self.field.method(args) with simple args
        for match in re.finditer(r'self(@)?\.(\w+)\.(\w+)\(([^)]+)\)', line):
            view = "@" if match.group(1) else ""
            field = match.group(2)
            method = match.group(3)
            args = match.group(4).strip()
            # Only simple args (single variable or literal)
            if re.match(r'^[\w]+$', args) or re.match(r'^\d+$', args):
                pattern = f"self{view}.{field}.{method}(<arg>)"
                patterns.append(PatternMatch(pattern, str(file_path), line_num, stripped))
        
        # Pattern 3: self@.field (view field access)
        for match in re.finditer(r'self@\.(\w+)(?!\s*\()', line):
            field = match.group(1)
            # Skip if it's part of a longer chain
            full_match = match.group(0)
            after_pos = match.end()
            if after_pos < len(line) and line[after_pos] == '.':
                continue
            pattern = f"self@.{field}"
            patterns.append(PatternMatch(pattern, str(file_path), line_num, stripped))
        
        # Pattern 4: old(self)@.method() or old(self)@.field
        for match in re.finditer(r'old\(self\)@\.(\w+)(?:\(\))?', line):
            access = match.group(1)
            has_parens = "()" in match.group(0)
            pattern = f"old(self)@.{access}" + ("()" if has_parens else "")
            patterns.append(PatternMatch(pattern, str(file_path), line_num, stripped))
        
        # Pattern 5: expr as int / CONSTANT or % CONSTANT
        for match in re.finditer(r'(\w+(?:\.\w+)*)\s+as\s+int\s*[/%]\s*([A-Z_]+)', line):
            expr = match.group(1)
            const = match.group(2)
            pattern = f"<expr> as int / {const}"
            patterns.append(PatternMatch(pattern, str(file_path), line_num, stripped))
        
        # Pattern 6: Repeated comparison patterns
        for match in re.finditer(r'(\w+(?:\.\w+)*)\s*(==|!=|<=|>=|<|>)\s*(\d+|[A-Z_]+)', line):
            expr = match.group(1)
            op = match.group(2)
            val = match.group(3)
            # Only if expr is a chain
            if '.' in expr:
                pattern = f"{expr} {op} <val>"
                patterns.append(PatternMatch(pattern, str(file_path), line_num, stripped))
        
        # Pattern 7: forall|i: int| 0 <= i < self.something
        for match in re.finditer(r'forall\|(\w+):\s*int\|\s*0\s*<=\s*\1\s*<\s*(self[^=]+)', line):
            bound = match.group(2).strip()
            # Normalize the bound expression
            bound = re.sub(r'\s+', ' ', bound)
            if bound.endswith('==>'):
                bound = bound[:-3].strip()
            pattern = f"forall|i| 0 <= i < {bound}"
            patterns.append(PatternMatch(pattern, str(file_path), line_num, stripped))
        
        # Pattern 8: self.field.contains(x) or self@.field.contains(x)
        for match in re.finditer(r'self(@)?\.(\w+)\.contains\((\w+)\)', line):
            view = "@" if match.group(1) else ""
            field = match.group(2)
            pattern = f"self{view}.{field}.contains(<arg>)"
            patterns.append(PatternMatch(pattern, str(file_path), line_num, stripped))
    
    return patterns


def find_existing_aliases(directory: Path) -> set[str]:
    """Find existing spec function names."""
    aliases = set()
    
    for file_path in directory.rglob('*.rs'):
        content = file_path.read_text()
        # Find spec fn declarations
        for match in re.finditer(r'spec\s+fn\s+(\w+)', content):
            aliases.add(match.group(1))
    
    return aliases


def analyze_patterns(patterns: list[PatternMatch], min_count: int = 3) -> dict:
    """Analyze patterns and group by frequency."""
    # Group by normalized pattern
    by_pattern = defaultdict(list)
    for p in patterns:
        by_pattern[p.pattern].append(p)
    
    # Filter by minimum count
    frequent = {k: v for k, v in by_pattern.items() if len(v) >= min_count}
    
    return frequent


def main():
    """Main entry point."""
    if len(sys.argv) > 1:
        directory = Path(sys.argv[1])
    else:
        directory = Path('.')
    
    if not directory.exists():
        print(f"Error: {directory} not found", file=sys.stderr)
        sys.exit(1)
    
    # Find all Rust files
    rs_files = list(directory.rglob('*.rs'))
    
    # Skip backup directories
    rs_files = [f for f in rs_files if 'backup' not in str(f)]
    
    print(f"Analyzing {len(rs_files)} Rust files in {directory}...")
    print()
    
    # Extract all patterns
    all_patterns = []
    for file_path in rs_files:
        try:
            patterns = extract_patterns(file_path)
            all_patterns.extend(patterns)
        except Exception as e:
            print(f"Warning: Error processing {file_path}: {e}", file=sys.stderr)
    
    # Analyze frequency
    frequent = analyze_patterns(all_patterns, min_count=3)
    
    if not frequent:
        print("No frequently repeated patterns found (threshold: 3 occurrences).")
        return
    
    # Sort by frequency
    sorted_patterns = sorted(frequent.items(), key=lambda x: -len(x[1]))
    
    print("=" * 80)
    print("FREQUENTLY REPEATED PATTERNS (potential extraction candidates)")
    print("=" * 80)
    print()
    
    for pattern, matches in sorted_patterns[:30]:  # Top 30
        files = set(m.file for m in matches)
        print(f"Pattern: {pattern}")
        print(f"  Count: {len(matches)} occurrences in {len(files)} file(s)")
        
        # Show file distribution
        file_counts = defaultdict(int)
        for m in matches:
            # Shorten path for display
            short_path = str(Path(m.file).relative_to(directory) if directory != Path('.') else m.file)
            file_counts[short_path] += 1
        
        for f, c in sorted(file_counts.items(), key=lambda x: -x[1])[:3]:
            print(f"    {f}: {c}")
        if len(file_counts) > 3:
            print(f"    ... and {len(file_counts) - 3} more files")
        
        # Show a few example lines
        print("  Examples:")
        seen_contexts = set()
        for m in matches[:3]:
            ctx = m.context[:80] + "..." if len(m.context) > 80 else m.context
            if ctx not in seen_contexts:
                seen_contexts.add(ctx)
                short_file = Path(m.file).name
                print(f"    {short_file}:{m.line_num}: {ctx}")
        print()
    
    print("=" * 80)
    print(f"Total: {len(sorted_patterns)} distinct patterns with 3+ occurrences")
    print(f"Total occurrences: {sum(len(v) for v in frequent.values())}")
    print()
    print("Consider extracting high-frequency patterns into spec functions for:")
    print("  - Better readability")
    print("  - SMT term sharing (may improve verification performance)")
    print("  - Easier maintenance")


if __name__ == "__main__":
    main()
