#!/bin/bash
# Copyright(c) The Maintainers of Nanvix.
# Licensed under the MIT License.

# Run the PM spec improvement pipeline on all scheduler modules.
#
# This script runs four improvement steps on each module in dependency order:
#   1. improve-abstraction  (add View-level abstract transition specs, with review)
#   2. exec-integrity       (check exec code integrity vs original, with review)
#   3. consistency          (check semantic consistency, already in workflow)
#   4. strengthen           (strengthen weak postconditions, already in workflow)
#
# Each step calls a Copilot agent (prover), then a reviewer (claude-opus-4.6),
# then the prover fixes review issues. One round only per step.
#
# Usage:
#   ./run_improve_pm.sh                  # Run all modules, all steps
#   ./run_improve_pm.sh --from N         # Start from module N (1-based)
#   ./run_improve_pm.sh --only N         # Run only module N
#   ./run_improve_pm.sh --step STEP      # Run only a specific step
#   ./run_improve_pm.sh --list           # List all modules
#   ./run_improve_pm.sh --dry-run        # Show what would be done
#
# Steps: improve-abstraction, exec-integrity, consistency, strengthen
# Use --step multiple times: --step improve-abstraction --step exec-integrity

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"
WORKFLOW="$PROJECT_ROOT/verus-ai/workflow.py"
TIMESTAMP=$(date +"%Y%m%d_%H%M%S")
LOG_DIR="$PROJECT_ROOT/verus-ai-history/logs"
PIPELINE_LOG="$LOG_DIR/improve_pm_${TIMESTAMP}.log"

# Module definitions: "MODULE_NAME|SOURCE_PATH|NEEDS_ABSTRACTION"
# NEEDS_ABSTRACTION: yes = run improve-abstraction, no = skip that step.
MODULES=(
    # Phase 1: System Types (simple wrappers, no abstraction needed).
    "pid|src/libs/sys/src/sys/pm/pid.rs|no"
    "tid|src/libs/sys/src/sys/pm/tid.rs|no"
    "sys_capability|src/libs/sys/src/sys/pm/capability.rs|no"

    # Phase 2: Synchronization Primitives (already abstract).
    "spinlock|src/kernel/src/pm/sync/spinlock.rs|no"
    "fence|src/kernel/src/pm/sync/fence.rs|no"
    "condvar|src/kernel/src/pm/sync/condvar.rs|no"
    "mutex|src/kernel/src/pm/sync/mutex.rs|no"
    "semaphore|src/kernel/src/pm/sync/semaphore.rs|no"

    # Phase 3: Thread State Machine (already abstract).
    "thread_state|src/kernel/src/pm/thread/state.rs|no"
    "interrupted|src/kernel/src/pm/thread/interrupted.rs|no"
    "ready|src/kernel/src/pm/thread/ready.rs|no"
    "running_thread|src/kernel/src/pm/thread/running.rs|no"
    "sleeping_thread|src/kernel/src/pm/thread/sleeping.rs|no"
    "zombie_thread|src/kernel/src/pm/thread/zombie.rs|no"
    "thread_manager|src/kernel/src/pm/thread/mod.rs|no"

    # Phase 4: Clock (already abstract).
    "clock|src/kernel/src/pm/clock.rs|no"

    # Phase 5: Process State Machine (NEEDS abstraction improvement).
    "process_capability|src/kernel/src/pm/process/capability.rs|no"
    "process_state|src/kernel/src/pm/process/state/mod.rs|no"
    "runnable|src/kernel/src/pm/process/state/runnable.rs|yes"
    "running_process|src/kernel/src/pm/process/state/running.rs|yes"
    "sleeping_process|src/kernel/src/pm/process/state/sleeping.rs|yes"
    "interrupted_process|src/kernel/src/pm/process/state/interrupted.rs|yes"
    "zombie_process|src/kernel/src/pm/process/state/zombie.rs|yes"

    # Phase 6: Core Scheduler.
    "process_manager|src/kernel/src/pm/process/manager/mod.rs|no"
)

TOTAL=${#MODULES[@]}

# All available steps.
ALL_STEPS=("improve-abstraction" "exec-integrity" "consistency" "strengthen")

#==================================================================================================
# Argument parsing.
#==================================================================================================

START_FROM=1
ONLY=""
LIST_ONLY=false
DRY_RUN=false
SELECTED_STEPS=()

while [[ $# -gt 0 ]]; do
    case "$1" in
        --from)
            START_FROM="$2"
            shift 2
            ;;
        --only)
            ONLY="$2"
            shift 2
            ;;
        --step)
            SELECTED_STEPS+=("$2")
            shift 2
            ;;
        --list)
            LIST_ONLY=true
            shift
            ;;
        --dry-run)
            DRY_RUN=true
            shift
            ;;
        -h|--help)
            echo "Usage: $0 [--from N] [--only N] [--step STEP] [--list] [--dry-run]"
            echo ""
            echo "Options:"
            echo "  --from N     Start from module N (1-based)"
            echo "  --only N     Run only module N (1-based)"
            echo "  --step STEP  Run only specific step(s). Repeatable."
            echo "               Steps: improve-abstraction, exec-integrity, consistency, strengthen"
            echo "  --list       List all modules and their steps"
            echo "  --dry-run    Show what would be done without executing"
            echo ""
            exit 0
            ;;
        *)
            echo "Unknown option: $1"
            exit 1
            ;;
    esac
done

# Default to all steps if none specified.
if [ ${#SELECTED_STEPS[@]} -eq 0 ]; then
    SELECTED_STEPS=("${ALL_STEPS[@]}")
fi

#==================================================================================================
# List modules.
#==================================================================================================

if [ "$LIST_ONLY" = true ]; then
    echo "PM Improvement Pipeline Modules:"
    echo "================================="
    for i in "${!MODULES[@]}"; do
        IFS='|' read -r name source needs_abstraction <<< "${MODULES[$i]}"
        idx=$((i + 1))
        steps=""
        if [ "$needs_abstraction" = "yes" ]; then
            steps="[abstraction, integrity, consistency, strengthen]"
        else
            steps="[integrity, consistency, strengthen]"
        fi
        printf "  %2d. %-25s %s\n" "$idx" "$name" "$steps"
    done
    echo ""
    echo "Total: $TOTAL modules"
    exit 0
fi

#==================================================================================================
# Helper: check if a step should run for a module.
#==================================================================================================

should_run_step() {
    local step="$1"
    local needs_abstraction="$2"

    # Check if step is in selected steps.
    local found=false
    for s in "${SELECTED_STEPS[@]}"; do
        if [ "$s" = "$step" ]; then
            found=true
            break
        fi
    done
    if [ "$found" = false ]; then
        return 1
    fi

    # improve-abstraction only runs for modules that need it.
    if [ "$step" = "improve-abstraction" ] && [ "$needs_abstraction" != "yes" ]; then
        return 1
    fi

    return 0
}

#==================================================================================================
# Main execution.
#==================================================================================================

mkdir -p "$LOG_DIR"

echo "============================================================" | tee "$PIPELINE_LOG"
echo "  NANVIX PM IMPROVEMENT PIPELINE" | tee -a "$PIPELINE_LOG"
echo "============================================================" | tee -a "$PIPELINE_LOG"
echo "" | tee -a "$PIPELINE_LOG"
echo "Total modules: $TOTAL" | tee -a "$PIPELINE_LOG"
echo "Starting from: $START_FROM" | tee -a "$PIPELINE_LOG"
echo "Steps: ${SELECTED_STEPS[*]}" | tee -a "$PIPELINE_LOG"
if [ -n "$ONLY" ]; then
    echo "Running only:  $ONLY" | tee -a "$PIPELINE_LOG"
fi
if [ "$DRY_RUN" = true ]; then
    echo "Mode: DRY RUN" | tee -a "$PIPELINE_LOG"
fi
echo "Timestamp:     $(date)" | tee -a "$PIPELINE_LOG"
echo "" | tee -a "$PIPELINE_LOG"

PASSED=0
FAILED=0
SKIPPED=0

for i in "${!MODULES[@]}"; do
    idx=$((i + 1))

    # Skip modules before start point.
    if [ "$idx" -lt "$START_FROM" ]; then
        continue
    fi

    # If --only is set, skip all but the specified module.
    if [ -n "$ONLY" ] && [ "$idx" -ne "$ONLY" ]; then
        continue
    fi

    IFS='|' read -r name source needs_abstraction <<< "${MODULES[$i]}"

    echo "" | tee -a "$PIPELINE_LOG"
    echo "============================================================" | tee -a "$PIPELINE_LOG"
    echo "  [$idx/$TOTAL] $name" | tee -a "$PIPELINE_LOG"
    echo "  Source: $source" | tee -a "$PIPELINE_LOG"
    echo "============================================================" | tee -a "$PIPELINE_LOG"

    # Check if source file exists.
    if [ ! -f "$PROJECT_ROOT/$source" ]; then
        echo "[SKIP] Source file not found: $source" | tee -a "$PIPELINE_LOG"
        SKIPPED=$((SKIPPED + 1))
        continue
    fi

    MODULE_PASSED=true

    for step in "${ALL_STEPS[@]}"; do
        if ! should_run_step "$step" "$needs_abstraction"; then
            continue
        fi

        echo "" | tee -a "$PIPELINE_LOG"
        echo "  --- Step: $step ---" | tee -a "$PIPELINE_LOG"

        if [ "$DRY_RUN" = true ]; then
            echo "  [DRY] python3 $WORKFLOW $step $name --source $source" | tee -a "$PIPELINE_LOG"
            continue
        fi

        cd "$PROJECT_ROOT"
        if python3 "$WORKFLOW" "$step" "$name" --source "$source" 2>&1 | tee -a "$PIPELINE_LOG"; then
            echo "  [$step] PASSED" | tee -a "$PIPELINE_LOG"
        else
            echo "  [$step] FAILED (continuing)" | tee -a "$PIPELINE_LOG"
            MODULE_PASSED=false
        fi
    done

    if [ "$DRY_RUN" = true ]; then
        continue
    fi

    if [ "$MODULE_PASSED" = true ]; then
        echo "" | tee -a "$PIPELINE_LOG"
        echo "[PASS] $name completed" | tee -a "$PIPELINE_LOG"
        PASSED=$((PASSED + 1))
    else
        echo "" | tee -a "$PIPELINE_LOG"
        echo "[FAIL] $name had failures" | tee -a "$PIPELINE_LOG"
        FAILED=$((FAILED + 1))
    fi
done

#==================================================================================================
# Summary.
#==================================================================================================

echo "" | tee -a "$PIPELINE_LOG"
echo "============================================================" | tee -a "$PIPELINE_LOG"
echo "  PIPELINE SUMMARY" | tee -a "$PIPELINE_LOG"
echo "============================================================" | tee -a "$PIPELINE_LOG"
echo "  Passed:  $PASSED" | tee -a "$PIPELINE_LOG"
echo "  Failed:  $FAILED" | tee -a "$PIPELINE_LOG"
echo "  Skipped: $SKIPPED" | tee -a "$PIPELINE_LOG"
echo "  Total:   $TOTAL" | tee -a "$PIPELINE_LOG"
echo "  Log:     $PIPELINE_LOG" | tee -a "$PIPELINE_LOG"
echo "============================================================" | tee -a "$PIPELINE_LOG"

# Git commit the log.
cd "$PROJECT_ROOT"
git add "$PIPELINE_LOG" 2>/dev/null || true
git commit -m "[verus-ai] Improvement pipeline log: ${TIMESTAMP}" 2>/dev/null || true

if [ "$FAILED" -gt 0 ]; then
    echo ""
    echo "Some modules failed. Use --from N to resume."
    exit 1
fi

exit 0
