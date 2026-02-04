#!/bin/bash
# Copyright(c) The Maintainers of Nanvix.
# Licensed under the MIT License.

# Description: Check for unused semantic properties in Verus specifications.
# Usage: ./check_unused_semantic_props.sh [file.rs]
#   If no file is specified, checks all .rs files in the current directory.

set -e

cd "$(dirname "$0")"

check_prop() {
    local file=$1
    local name=$2
    local line=$(grep -n "pub open spec fn $name\|pub closed spec fn $name" "$file" 2>/dev/null | head -1 | cut -d: -f1)
    if [ -z "$line" ]; then
        return
    fi

    # Count usage in same file (excluding definition line)
    local same_file_usage=$(grep -n "$name" "$file" | grep -v "^$line:" | grep -v "^\s*//" | wc -l)

    # Check cross-file usage (excluding same-named functions in other files)
    local other_usage=0
    for other_file in *.rs; do
        if [ "$other_file" != "$file" ]; then
            # Count references that are not function definitions
            local refs
            refs=$(grep -c "$name" "$other_file" 2>/dev/null) || refs=0
            local defs
            defs=$(grep -c "spec fn $name" "$other_file" 2>/dev/null) || defs=0
            other_usage=$((other_usage + refs - defs))
        fi
    done

    if [ "$same_file_usage" -eq 0 ] && [ "$other_usage" -le 0 ]; then
        echo "❌ $file:$line $name - UNUSED"
    elif [ "$same_file_usage" -eq 0 ]; then
        echo "⚠️  $file:$line $name - only in other files ($other_usage refs)"
    else
        echo "✅ $file:$line $name - used ($same_file_usage local, $other_usage external)"
    fi
}

echo "=== Unused Semantic Properties Analysis ==="
echo ""

if [ -n "$1" ]; then
    # Check specific file
    files="$1"
else
    # Check all .rs files
    files="*.rs"
fi

for file in $files; do
    if [ ! -f "$file" ]; then
        continue
    fi
    echo "--- $file ---"
    # Extract all spec function names
    grep -n "pub open spec fn\|pub closed spec fn" "$file" 2>/dev/null | while read line; do
        fn_name=$(echo "$line" | sed 's/.*spec fn \([a-z_]*\).*/\1/')
        # Skip view functions and simple accessors
        if [[ "$fn_name" != "view" && "$fn_name" != "@" ]]; then
            check_prop "$file" "$fn_name"
        fi
    done
    echo ""
done
