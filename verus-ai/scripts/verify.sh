#!/bin/bash
# Copyright(c) The Maintainers of Nanvix.
# Licensed under the MIT License.

# Verus verification script with automatic git commit.
# Usage: ./verify.sh [module_name]
#
# This script:
# 1. Runs Verus verification
# 2. Logs results with timestamp
# 3. Commits all changes in verus/ directory

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"
VERUS_DIR="$PROJECT_ROOT/verus"
HISTORY_DIR="$PROJECT_ROOT/verus-ai-history"
LOGS_DIR="$HISTORY_DIR/logs"

MODULE="${1:-}"
TIMESTAMP=$(date +"%Y%m%d_%H%M%S")

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

# Check for cheating patterns.
CHEATING=""
cd "$VERUS_DIR"
if [ -n "$MODULE" ] && [ -f "${MODULE}.rs" ]; then
    ASSUME_COUNT=$(grep -c 'assume\s*(' "${MODULE}.rs" 2>/dev/null || true)
    EXTERNAL_COUNT=$(grep -c 'external_body' "${MODULE}.rs" 2>/dev/null || true)
    # Handle empty or multi-line output.
    ASSUME_COUNT=$(echo "$ASSUME_COUNT" | head -1 | tr -d '[:space:]')
    EXTERNAL_COUNT=$(echo "$EXTERNAL_COUNT" | head -1 | tr -d '[:space:]')
    [ -z "$ASSUME_COUNT" ] && ASSUME_COUNT=0
    [ -z "$EXTERNAL_COUNT" ] && EXTERNAL_COUNT=0
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
