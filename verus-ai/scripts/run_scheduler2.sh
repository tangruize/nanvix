#!/bin/bash
# Copyright(c) The Maintainers of Nanvix.
# Licensed under the MIT License.

# Run the supplementary Verus verification pipeline for scheduler-related code
# NOT covered by run_scheduler.sh.
#
# This script covers three categories of missing files:
#
#   PHASE 7: Scheduler Engine (unsafe.rs)
#     The core scheduling entry points: giveup(), switch(), exit(), exit_thread(),
#     sleep(), and the global state (REMAINING_QUANTUM, CURRENT_PID, CURRENT_TID).
#     This is the most critical missing piece from the original pipeline.
#
#   PHASE 8: Kernel Call Infrastructure
#     The kcall dispatch layer that connects user-space system calls to the
#     scheduler. Includes the ScoreBoard (serialized kcall dispatch via
#     Mutex + Semaphore) and the do_kcall() dispatcher.
#
#   PHASE 9: PM Kernel Call Handlers
#     Individual kcall handlers that invoke scheduler operations: sleep, mutex
#     lock/unlock, condvar wait/signal, thread create/join, and process terminate.
#
# Prerequisites:
#   Run run_scheduler.sh first. This script depends on verified output from
#   Phases 1-6 (sys types, sync primitives, thread/process state machines,
#   clock, and process_manager).
#
# Usage:
#   ./run_scheduler2.sh              # Run all modules from scratch
#   ./run_scheduler2.sh --from N     # Start from module N (1-based, for resuming)
#   ./run_scheduler2.sh --only N     # Run only module N
#   ./run_scheduler2.sh --list       # List all modules in order
#
# The verification order follows the bottom-up dependency graph:
#
#   PHASE 7: Scheduler Engine
#     1. process_manager_unsafe - giveup/switch/exit/sleep (depends on Phase 6)
#
#   PHASE 8: Kernel Call Infrastructure
#     2. kcall_scoreboard  - ScoreBoard: Mutex + Semaphore kcall serialization
#     3. kcall_dispatcher  - do_kcall(): user→kernel entry, fast/slow path routing
#     4. kcall_handler     - kcall_handler(): kernel thread main loop
#
#   PHASE 9: PM Kernel Call Handlers (scheduling-related only)
#     5.  kcall_sleep          - sleep() kcall handler
#     6.  kcall_lock_mutex     - mutex lock handler (may sleep)
#     7.  kcall_unlock_mutex   - mutex unlock handler (may wakeup)
#     8.  kcall_wait_cond      - condvar wait handler (sleep + reacquire mutex)
#     9.  kcall_signal_cond    - condvar signal/broadcast handler
#     10. kcall_terminate      - process termination handler
#     11. kcall_create_thread  - thread creation handler
#     12. kcall_join_thread    - thread join handler (may sleep)

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"
WORKFLOW="$PROJECT_ROOT/verus-ai/workflow.py"

# Define all modules in verification order.
# Format: "MODULE_NAME|SOURCE_PATH|OUTPUT_SUBDIR|FILE_STEM"
MODULES=(
    # Phase 7: Scheduler Engine
    "process_manager_unsafe|src/kernel/src/pm/process/manager/unsafe.rs|kernel/pm/process/manager|process_manager_unsafe"

    # Phase 8: Kernel Call Infrastructure
    "kcall_scoreboard|src/kernel/src/kcall/mod.rs|kernel/kcall|scoreboard"
    "kcall_dispatcher|src/kernel/src/kcall/dispatcher.rs|kernel/kcall|dispatcher"
    "kcall_handler|src/kernel/src/kcall/handler.rs|kernel/kcall|handler"

    # Phase 9: PM Kernel Call Handlers (scheduling-related)
    "kcall_sleep|src/kernel/src/pm/kcall/sleep.rs|kernel/pm/kcall|sleep"
    "kcall_lock_mutex|src/kernel/src/pm/kcall/lock_mutex.rs|kernel/pm/kcall|lock_mutex"
    "kcall_unlock_mutex|src/kernel/src/pm/kcall/unlock_mutex.rs|kernel/pm/kcall|unlock_mutex"
    "kcall_wait_cond|src/kernel/src/pm/kcall/wait_cond.rs|kernel/pm/kcall|wait_cond"
    "kcall_signal_cond|src/kernel/src/pm/kcall/signal_cond.rs|kernel/pm/kcall|signal_cond"
    "kcall_terminate|src/kernel/src/pm/kcall/terminate.rs|kernel/pm/kcall|terminate"
    "kcall_create_thread|src/kernel/src/pm/kcall/create_thread.rs|kernel/pm/kcall|create_thread"
    "kcall_join_thread|src/kernel/src/pm/kcall/join_thread.rs|kernel/pm/kcall|join_thread"
)

TOTAL=${#MODULES[@]}

#==================================================================================================
# Argument parsing.
#==================================================================================================

START_FROM=1
ONLY=""
LIST_ONLY=false

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
        --list)
            LIST_ONLY=true
            shift
            ;;
        -h|--help)
            echo "Usage: $0 [--from N] [--only N] [--list]"
            echo ""
            echo "Options:"
            echo "  --from N   Start from module N (1-based)"
            echo "  --only N   Run only module N (1-based)"
            echo "  --list     List all modules in order"
            echo ""
            echo "Note: Run run_scheduler.sh first (Phases 1-6 are prerequisites)."
            exit 0
            ;;
        *)
            echo "Unknown option: $1"
            exit 1
            ;;
    esac
done

#==================================================================================================
# List modules.
#==================================================================================================

if [ "$LIST_ONLY" = true ]; then
    echo "Scheduler Supplementary Verification Modules (Phases 7-9):"
    echo "==========================================================="
    for i in "${!MODULES[@]}"; do
        IFS='|' read -r name source output_subdir file_stem <<< "${MODULES[$i]}"
        idx=$((i + 1))
        printf "  %2d. %-28s %s\n" "$idx" "$name" "$source"
    done
    echo ""
    echo "Total: $TOTAL modules"
    echo ""
    echo "Prerequisite: run_scheduler.sh (Phases 1-6, 24 modules)"
    exit 0
fi

#==================================================================================================
# Main execution.
#==================================================================================================

echo "============================================================"
echo "  NANVIX SCHEDULER SUPPLEMENTARY VERIFICATION PIPELINE"
echo "  (Phases 7-9: Engine, Kcall Infrastructure, PM Handlers)"
echo "============================================================"
echo ""
echo "Total modules: $TOTAL"
echo "Starting from: $START_FROM"
if [ -n "$ONLY" ]; then
    echo "Running only:  $ONLY"
fi
echo "Output dir:    verus/split/kernel/{pm,kcall}/"
echo "Timestamp:     $(date)"
echo ""

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

    IFS='|' read -r name source output_subdir file_stem <<< "${MODULES[$i]}"

    echo ""
    echo "============================================================"
    echo "  [$idx/$TOTAL] $name"
    echo "  Source: $source"
    echo "  Output: verus/split/$output_subdir/"
    echo "  Files:  $file_stem.rs, $file_stem.spec.rs, $file_stem.proof.rs"
    echo "============================================================"

    # Check if source file exists.
    if [ ! -f "$PROJECT_ROOT/$source" ]; then
        echo "[SKIP] Source file not found: $source"
        SKIPPED=$((SKIPPED + 1))
        continue
    fi

    # Run the verification workflow.
    cd "$PROJECT_ROOT"
    if python3 "$WORKFLOW" verify "$source" \
        --name "$name" \
        --output-subdir "$output_subdir" \
        --file-stem "$file_stem"; then
        echo ""
        echo "[PASS] $name verified successfully"
        PASSED=$((PASSED + 1))
    else
        echo ""
        echo "[FAIL] $name verification did not pass all reviewers"
        FAILED=$((FAILED + 1))
        # Continue to next module even if this one failed.
    fi
done

#==================================================================================================
# Summary.
#==================================================================================================

echo ""
echo "============================================================"
echo "  PIPELINE SUMMARY (Supplementary)"
echo "============================================================"
echo "  Passed:  $PASSED"
echo "  Failed:  $FAILED"
echo "  Skipped: $SKIPPED"
echo "  Total:   $TOTAL"
echo "============================================================"

if [ "$FAILED" -gt 0 ]; then
    echo ""
    echo "Some modules failed. Use --from N to resume from a specific module."
    exit 1
fi

exit 0
