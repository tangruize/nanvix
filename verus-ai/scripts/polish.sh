#!/bin/bash
# Copyright(c) The Maintainers of Nanvix.
# Licensed under the MIT License.

# Polish script for verified modules.
# Runs three phases: consistency, simplify, strengthen.
# Comment out phases you've already completed.

set -e

# Check arguments.
if [ $# -lt 1 ]; then
    echo "Usage: $0 <module_name> [source_path]"
    echo "Example: $0 slab src/libs/slab/src/lib.rs"
    exit 1
fi

MODULE="$1"
SOURCE="${2:-}"

cd "$(dirname "$0")/../.."

echo "=============================================="
echo "POLISH: $MODULE"
echo "=============================================="

# Build source argument if provided.
SOURCE_ARG=""
if [ -n "$SOURCE" ]; then
    SOURCE_ARG="--source $SOURCE"
fi

# ==================================================
# Phase 1: Consistency Check
# ==================================================
echo ""
echo "[Phase 1/3] Consistency Check..."
python3 verus-ai/workflow.py consistency "$MODULE" $SOURCE_ARG

# ==================================================
# Phase 2: Simplify
# ==================================================
echo ""
echo "[Phase 2/3] Simplify..."
python3 verus-ai/workflow.py simplify "$MODULE" $SOURCE_ARG

# ==================================================
# Phase 3: Strengthen Specs
# ==================================================
echo ""
echo "[Phase 3/3] Strengthen Specs..."
python3 verus-ai/workflow.py strengthen "$MODULE" $SOURCE_ARG

echo ""
echo "=============================================="
echo "POLISH COMPLETE: $MODULE"
echo "=============================================="
echo ""
echo "Reports generated:"
echo "  - verus-ai-history/consistency/${MODULE}.md"
echo "  - verus-ai-history/simplify/${MODULE}.md"
echo "  - verus-ai-history/strengthen/${MODULE}.md"
