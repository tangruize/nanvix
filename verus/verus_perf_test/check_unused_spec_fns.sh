#!/bin/bash
# Check for unused spec functions in Verus code
# Usage: ./check_unused_spec_fns.sh [directory]

DIR="${1:-.}"
cd "$DIR" || exit 1

echo "=== Checking for unused spec functions in $DIR ==="
echo ""

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

unused_count=0
used_count=0
total_count=0

# Find all spec fn definitions
# Patterns: "spec fn name", "open spec fn name", "closed spec fn name", 
#           "pub spec fn name", "pub open spec fn name", "pub closed spec fn name"
grep -rn "spec fn [a-z_][a-z0-9_]*" *.rs 2>/dev/null | while read -r line; do
    # Extract file, line number, and function name
    file=$(echo "$line" | cut -d: -f1)
    lineno=$(echo "$line" | cut -d: -f2)
    
    # Extract function name (handles various patterns)
    func_name=$(echo "$line" | grep -oP 'spec fn \K[a-z_][a-z0-9_]*' | head -1)
    
    if [ -z "$func_name" ]; then
        continue
    fi
    
    total_count=$((total_count + 1))
    
    # Count occurrences of this function name (excluding the definition line itself)
    # We look for: func_name( or func_name::<  or .func_name( or ::func_name(
    usage_count=$(grep -rn "\b${func_name}\b" *.rs 2>/dev/null | grep -v "spec fn ${func_name}" | grep -v "^${file}:${lineno}:" | wc -l)
    
    # Also check if it's used in comments (documentation)
    doc_usage=$(grep -rn "/// .*${func_name}" *.rs 2>/dev/null | wc -l)
    
    if [ "$usage_count" -eq 0 ]; then
        echo -e "${RED}[UNUSED]${NC} ${file}:${lineno} - ${func_name}()"
        unused_count=$((unused_count + 1))
    else
        # Check if only used in its own file (might still be dead code)
        other_file_usage=$(grep -rn "\b${func_name}\b" *.rs 2>/dev/null | grep -v "spec fn ${func_name}" | grep -v "^${file}:" | wc -l)
        if [ "$other_file_usage" -eq 0 ]; then
            echo -e "${YELLOW}[INTERNAL]${NC} ${file}:${lineno} - ${func_name}() (only used in same file: ${usage_count} times)"
        else
            echo -e "${GREEN}[USED]${NC} ${file}:${lineno} - ${func_name}() (${usage_count} usages, ${other_file_usage} in other files)"
        fi
        used_count=$((used_count + 1))
    fi
done

echo ""
echo "=== Summary ==="
echo "Scanned directory: $DIR"
echo "To see only unused: $0 | grep UNUSED"
