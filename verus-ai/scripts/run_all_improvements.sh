#!/bin/bash
# Copyright(c) The Maintainers of Nanvix.
# Licensed under the MIT License.

# Run spec methodology check and exec consistency check on all verified modules.
#
# Two flows per module:
#   1. spec-methodology  — tree-sitter detects guideline violations, AI fixes
#   2. exec-consistency  — tree-sitter AST hash diff, AI fixes or documents
#
# Each flow: prover → reviewer (claude-opus-4.6, 1 round) → prover fix.
#
# Usage:
#   ./run_all_improvements.sh                  # Run all modules, all steps
#   ./run_all_improvements.sh --from N         # Start from module N (1-based)
#   ./run_all_improvements.sh --only N         # Run only module N
#   ./run_all_improvements.sh --step STEP      # Run only a specific step
#   ./run_all_improvements.sh --list           # List all modules
#   ./run_all_improvements.sh --dry-run        # Show what would be done
#
# Steps: spec-methodology, exec-consistency

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"
WORKFLOW="$PROJECT_ROOT/verus-ai/workflow.py"
TIMESTAMP=$(date +"%Y%m%d_%H%M%S")
LOG_DIR="$PROJECT_ROOT/verus-ai-history/logs"
PIPELINE_LOG="$LOG_DIR/all_improvements_${TIMESTAMP}.log"

# Module definitions: "MODULE_NAME|SOURCE_PATH"
MODULES=(
    # === PM: System Types ===
    "pid|src/libs/sys/src/sys/pm/pid.rs"
    "tid|src/libs/sys/src/sys/pm/tid.rs"
    "sys_capability|src/libs/sys/src/sys/pm/capability.rs"

    # === PM: Synchronization Primitives ===
    "spinlock|src/kernel/src/pm/sync/spinlock.rs"
    "fence|src/kernel/src/pm/sync/fence.rs"
    "condvar|src/kernel/src/pm/sync/condvar.rs"
    "mutex|src/kernel/src/pm/sync/mutex.rs"
    "semaphore|src/kernel/src/pm/sync/semaphore.rs"

    # === PM: Thread State Machine ===
    "thread_state|src/kernel/src/pm/thread/state.rs"
    "interrupted|src/kernel/src/pm/thread/interrupted.rs"
    "ready|src/kernel/src/pm/thread/ready.rs"
    "running_thread|src/kernel/src/pm/thread/running.rs"
    "sleeping_thread|src/kernel/src/pm/thread/sleeping.rs"
    "zombie_thread|src/kernel/src/pm/thread/zombie.rs"
    "thread_manager|src/kernel/src/pm/thread/mod.rs"

    # === PM: Clock ===
    "clock|src/kernel/src/pm/clock.rs"

    # === PM: Process State Machine ===
    "process_capability|src/kernel/src/pm/process/capability.rs"
    "process_state|src/kernel/src/pm/process/state/mod.rs"
    "runnable|src/kernel/src/pm/process/state/runnable.rs"
    "running_process|src/kernel/src/pm/process/state/running.rs"
    "sleeping_process|src/kernel/src/pm/process/state/sleeping.rs"
    "interrupted_process|src/kernel/src/pm/process/state/interrupted.rs"
    "zombie_process|src/kernel/src/pm/process/state/zombie.rs"

    # === PM: Core Scheduler ===
    "process_manager|src/kernel/src/pm/process/manager/mod.rs"

    # === MM: Physical Memory ===
    "frame|src/kernel/src/mm/phys/frame.rs"
    "kpool|src/kernel/src/mm/phys/kpool.rs"
    "upool|src/kernel/src/mm/phys/upool.rs"
    "manager|src/kernel/src/mm/phys/manager.rs"

    # === MM: Virtual Memory ===
    "kpage|src/kernel/src/mm/virt/kpage.rs"
    "vmem|src/kernel/src/mm/virt/vmem.rs"

    # === MM: Stacks and Heaps ===
    "kheap|src/kernel/src/mm/kheap.rs"
    "kredzone|src/kernel/src/mm/kredzone.rs"
    "kstack|src/kernel/src/mm/kstack.rs"
    "ustack|src/kernel/src/mm/ustack.rs"

    # === HAL ===
    "frame_hal|src/kernel/src/hal/mem/types/address/frame.rs"

    # === KCall Infrastructure ===
    "scoreboard|src/kernel/src/kcall/mod.rs"
    "dispatcher|src/kernel/src/kcall/dispatcher.rs"
    "handler|src/kernel/src/kcall/handler.rs"

    # === PM KCall Handlers ===
    "kcall_sleep|src/kernel/src/pm/kcall/sleep.rs"
    "kcall_lock_mutex|src/kernel/src/pm/kcall/lock_mutex.rs"
    "kcall_unlock_mutex|src/kernel/src/pm/kcall/unlock_mutex.rs"
    "kcall_wait_cond|src/kernel/src/pm/kcall/wait_cond.rs"
    "kcall_signal_cond|src/kernel/src/pm/kcall/signal_cond.rs"
    "kcall_terminate|src/kernel/src/pm/kcall/terminate.rs"
    "kcall_create_thread|src/kernel/src/pm/kcall/create_thread.rs"
    "kcall_join_thread|src/kernel/src/pm/kcall/join_thread.rs"
)

TOTAL=${#MODULES[@]}
ALL_STEPS=("spec-methodology" "exec-consistency")

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
            echo "               Steps: spec-methodology, exec-consistency"
            echo "  --list       List all modules"
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
    echo "All Verified Modules (spec-methodology + exec-consistency):"
    echo "============================================================"
    for i in "${!MODULES[@]}"; do
        IFS='|' read -r name source <<< "${MODULES[$i]}"
        idx=$((i + 1))
        printf "  %2d. %-28s %s\n" "$idx" "$name" "$source"
    done
    echo ""
    echo "Total: $TOTAL modules"
    echo "Steps: ${ALL_STEPS[*]}"
    exit 0
fi

#==================================================================================================
# Main execution.
#==================================================================================================

mkdir -p "$LOG_DIR"

echo "============================================================" | tee "$PIPELINE_LOG"
echo "  NANVIX SPEC METHODOLOGY + EXEC CONSISTENCY PIPELINE" | tee -a "$PIPELINE_LOG"
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

    if [ "$idx" -lt "$START_FROM" ]; then
        continue
    fi

    if [ -n "$ONLY" ] && [ "$idx" -ne "$ONLY" ]; then
        continue
    fi

    IFS='|' read -r name source <<< "${MODULES[$i]}"

    echo "" | tee -a "$PIPELINE_LOG"
    echo "============================================================" | tee -a "$PIPELINE_LOG"
    echo "  [$idx/$TOTAL] $name" | tee -a "$PIPELINE_LOG"
    echo "  Source: $source" | tee -a "$PIPELINE_LOG"
    echo "============================================================" | tee -a "$PIPELINE_LOG"

    if [ ! -f "$PROJECT_ROOT/$source" ]; then
        echo "[SKIP] Source file not found: $source" | tee -a "$PIPELINE_LOG"
        SKIPPED=$((SKIPPED + 1))
        continue
    fi

    MODULE_PASSED=true

    for step in "${SELECTED_STEPS[@]}"; do
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

cd "$PROJECT_ROOT"
git add "$PIPELINE_LOG" 2>/dev/null || true
git commit -m "[verus-ai] Improvement pipeline log: ${TIMESTAMP}" 2>/dev/null || true

if [ "$FAILED" -gt 0 ]; then
    echo ""
    echo "Some modules failed. Use --from N to resume."
    exit 1
fi

exit 0
