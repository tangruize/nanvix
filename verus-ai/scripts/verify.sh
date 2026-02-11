#!/bin/bash
# Copyright(c) The Maintainers of Nanvix.
# Licensed under the MIT License.

# Verus verification script with automatic git commit.
# Usage: ./verify.sh [module_name]
#
# This script:
# 1. Runs Verus verification from verus/split/ directory
# 2. Logs results with timestamp
# 3. Commits all changes in verus/ directory

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"
VERUS_DIR="$PROJECT_ROOT/verus/split"
HISTORY_DIR="$PROJECT_ROOT/verus-ai-history"
LOGS_DIR="$HISTORY_DIR/logs"

MODULE="${1:-}"
TIMESTAMP=$(date +"%Y%m%d_%H%M%S")

# If MODULE is a short name (no ::), try to find the full module path.
if [ -n "$MODULE" ] && [[ ! "$MODULE" == *"::"* ]]; then
    # Search for a matching module file in the split directory.
    FOUND_PATH=$(find "$VERUS_DIR" -name "${MODULE}.rs" ! -name "*.spec.rs" ! -name "*.proof.rs" -type f 2>/dev/null | head -1)
    if [ -z "$FOUND_PATH" ]; then
        # Fallback: replace underscores with / to form a path pattern.
        # e.g., sys_capability -> sys/capability, then search for that suffix.
        ALT_PATH=$(echo "$MODULE" | sed 's|_|/|g')
        FOUND_PATH=$(find "$VERUS_DIR" -path "*/${ALT_PATH}.rs" ! -name "*.spec.rs" ! -name "*.proof.rs" -type f 2>/dev/null | head -1)
    fi
    if [ -z "$FOUND_PATH" ]; then
        # Fallback: reverse underscore-separated segments and search.
        # e.g., running_thread -> thread/running, then search for that suffix.
        IFS='_' read -ra PARTS <<< "$MODULE"
        if [ "${#PARTS[@]}" -eq 2 ]; then
            REV_PATH="${PARTS[1]}/${PARTS[0]}"
            FOUND_PATH=$(find "$VERUS_DIR" -path "*/${REV_PATH}.rs" ! -name "*.spec.rs" ! -name "*.proof.rs" -type f 2>/dev/null | head -1)
        fi
    fi
    if [ -z "$FOUND_PATH" ]; then
        # Fallback: search for last segment as filename inside a path containing the first segment.
        # e.g., running_process -> find running.rs under a path containing /process/.
        IFS='_' read -ra PARTS <<< "$MODULE"
        if [ "${#PARTS[@]}" -eq 2 ]; then
            FOUND_PATH=$(find "$VERUS_DIR" -path "*/${PARTS[1]}/*/${PARTS[0]}.rs" ! -name "*.spec.rs" ! -name "*.proof.rs" -type f 2>/dev/null | head -1)
        fi
    fi
    if [ -z "$FOUND_PATH" ]; then
        # Fallback for 3+ segments: try first segment as directory, rest joined by underscore as filename.
        # e.g., kcall_lock_mutex -> kcall/lock_mutex.rs
        IFS='_' read -ra PARTS <<< "$MODULE"
        if [ "${#PARTS[@]}" -ge 3 ]; then
            DIR_PART="${PARTS[0]}"
            FILE_PART=$(IFS='_'; echo "${PARTS[*]:1}")
            FOUND_PATH=$(find "$VERUS_DIR" -path "*/${DIR_PART}/${FILE_PART}.rs" ! -name "*.spec.rs" ! -name "*.proof.rs" -type f 2>/dev/null | head -1)
        fi
    fi
    if [ -z "$FOUND_PATH" ]; then
        # Fallback: if second segment is "init" or "mod", search for mod.rs in a directory
        # matching the first segment. e.g., virt_init -> */virt/mod.rs -> kernel::mm::virt
        IFS='_' read -ra PARTS <<< "$MODULE"
        if [ "${#PARTS[@]}" -eq 2 ] && { [ "${PARTS[1]}" = "init" ] || [ "${PARTS[1]}" = "mod" ]; }; then
            FOUND_PATH=$(find "$VERUS_DIR" -path "*/${PARTS[0]}/mod.rs" -type f 2>/dev/null | head -1)
        fi
    fi
    if [ -z "$FOUND_PATH" ]; then
        # Fallback: search for mod.rs inside a directory matching the full module name.
        # e.g., virt -> */virt/mod.rs
        FOUND_PATH=$(find "$VERUS_DIR" -path "*/${MODULE}/mod.rs" -type f 2>/dev/null | head -1)
    fi
    if [ -n "$FOUND_PATH" ]; then
        # Convert file path to module path.
        # e.g., /path/verus/split/kernel/pm/sys/pid.rs -> kernel::pm::sys::pid
        # e.g., /path/verus/split/kernel/mm/virt/mod.rs -> kernel::mm::virt
        REL_PATH="${FOUND_PATH#$VERUS_DIR/}"
        REL_PATH="${REL_PATH%.rs}"
        FULL_MODULE=$(echo "$REL_PATH" | sed 's|/|::|g')
        # Strip trailing ::mod for mod.rs files.
        FULL_MODULE="${FULL_MODULE%::mod}"
        echo "Resolved module: $MODULE -> $FULL_MODULE"
        MODULE="$FULL_MODULE"
    fi
fi

# Create module-specific log directory if module is specified.
if [ -n "$MODULE" ]; then
    MODULE_LOG_DIR="$LOGS_DIR/$MODULE"
    mkdir -p "$MODULE_LOG_DIR"
    LOG_FILE="$MODULE_LOG_DIR/verify_${TIMESTAMP}.txt"
else
    mkdir -p "$LOGS_DIR"
    LOG_FILE="$LOGS_DIR/verify_all_${TIMESTAMP}.txt"
fi

cd "$VERUS_DIR"

# Build verification command.
VERUS_CMD="verus --crate-type lib lib.rs"
if [ -n "$MODULE" ]; then
    VERUS_CMD="$VERUS_CMD --verify-module $MODULE"
fi

echo "=== Verus Verification ===" | tee "$LOG_FILE"
echo "Timestamp: $(date)" | tee -a "$LOG_FILE"
echo "Module: ${MODULE:-all}" | tee -a "$LOG_FILE"
echo "Command: $VERUS_CMD" | tee -a "$LOG_FILE"
echo "Working dir: $VERUS_DIR" | tee -a "$LOG_FILE"
echo "" | tee -a "$LOG_FILE"

# Run verification and capture output.
set +e
START_TIME=$(date +%s)
$VERUS_CMD 2>&1 | tee -a "$LOG_FILE"
EXIT_CODE=${PIPESTATUS[0]}
END_TIME=$(date +%s)
set -e

DURATION=$((END_TIME - START_TIME))
echo "" | tee -a "$LOG_FILE"
echo "Duration: ${DURATION}s" | tee -a "$LOG_FILE"
echo "Exit code: $EXIT_CODE" | tee -a "$LOG_FILE"

# Extract verification results.
RESULTS=$(grep "verification results:" "$LOG_FILE" | tail -1 || echo "")
VERIFIED=$(echo "$RESULTS" | grep -oP '\d+ verified' | grep -oP '\d+' || echo "0")
ERRORS=$(echo "$RESULTS" | grep -oP '\d+ errors' | grep -oP '\d+' || echo "0")

echo "" | tee -a "$LOG_FILE"
echo "=== Summary ===" | tee -a "$LOG_FILE"
echo "Verified: $VERIFIED" | tee -a "$LOG_FILE"
echo "Errors: $ERRORS" | tee -a "$LOG_FILE"

# Determine result status.
if [ "$VERIFIED" -gt 0 ] 2>/dev/null && [ "$ERRORS" -eq 0 ] 2>/dev/null; then
    STATUS="PASSED"
    MSG="succeeded"
else
    STATUS="FAILED"
    MSG="failed"
fi
echo "Status: $STATUS" | tee -a "$LOG_FILE"

# Check for cheating patterns across all split files for the module.
CHEATING=""
cd "$VERUS_DIR"
if [ -n "$MODULE" ]; then
    # Search recursively in the split directory for all files related to this module.
    ALL_MODULE_FILES=$(find . -name "${MODULE}.rs" -o -name "${MODULE}.spec.rs" -o -name "${MODULE}.proof.rs" -o -name "lib.rs" -path "*/${MODULE}/*" -o -name "lib.spec.rs" -path "*/${MODULE}/*" -o -name "lib.proof.rs" -path "*/${MODULE}/*" 2>/dev/null || true)

    ASSUME_COUNT=0
    EXTERNAL_COUNT=0
    for f in $ALL_MODULE_FILES; do
        if [ -f "$f" ]; then
            AC=$(grep -c 'assume\s*(' "$f" 2>/dev/null || true)
            EC=$(grep -c 'external_body' "$f" 2>/dev/null || true)
            AC=$(echo "$AC" | head -1 | tr -d '[:space:]')
            EC=$(echo "$EC" | head -1 | tr -d '[:space:]')
            [ -z "$AC" ] && AC=0
            [ -z "$EC" ] && EC=0
            ASSUME_COUNT=$((ASSUME_COUNT + AC))
            EXTERNAL_COUNT=$((EXTERNAL_COUNT + EC))
        fi
    done

    if [ "$ASSUME_COUNT" -gt 0 ] 2>/dev/null; then
        CHEATING="assume:$ASSUME_COUNT"
    fi
    if [ "$EXTERNAL_COUNT" -gt 0 ] 2>/dev/null; then
        CHEATING="${CHEATING:+$CHEATING,}external_body:$EXTERNAL_COUNT"
    fi
fi

if [ -n "$CHEATING" ]; then
    echo "Cheating patterns: $CHEATING" | tee -a "$LOG_FILE"
fi

# Git commit all changes in verus directory.
cd "$PROJECT_ROOT"
if git diff --quiet verus/ 2>/dev/null && git diff --cached --quiet verus/ 2>/dev/null; then
    echo "No changes to commit in verus/" | tee -a "$LOG_FILE"
else
    git add verus/
    COMMIT_MSG="[verus] Verification $MSG for ${MODULE:-all} (verified:$VERIFIED, errors:$ERRORS)"
    if [ -n "$CHEATING" ]; then
        COMMIT_MSG="$COMMIT_MSG [$CHEATING]"
    fi
    git commit -m "$COMMIT_MSG" 2>/dev/null || true
    echo "Committed: $COMMIT_MSG" | tee -a "$LOG_FILE"
fi

# Also commit the log file.
git add "$LOG_FILE" 2>/dev/null || true
git commit -m "[verus-ai] Log: verify ${MODULE:-all} $STATUS" 2>/dev/null || true

exit $EXIT_CODE
