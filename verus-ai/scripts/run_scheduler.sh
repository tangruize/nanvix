#!/bin/bash
# Copyright(c) The Maintainers of Nanvix.
# Licensed under the MIT License.

# Run the full Verus verification pipeline for the Nanvix scheduler (PM subsystem).
#
# This script verifies scheduler source files in bottom-up dependency order,
# producing three-file split output (exec, spec, proof) under verus/split/kernel/pm/.
#
# Usage:
#   ./run_scheduler.sh              # Run all modules from scratch
#   ./run_scheduler.sh --from N     # Start from module N (1-based, for resuming)
#   ./run_scheduler.sh --only N     # Run only module N
#   ./run_scheduler.sh --list       # List all modules in order
#
# The verification order follows the bottom-up dependency graph:
#
#   PHASE 1: System Types (sys library)
#     1. pid          - ProcessIdentifier type
#     2. tid          - ThreadIdentifier type
#     3. capability   - Capability enum (sys)
#
#   PHASE 2: Synchronization Primitives
#     4. spinlock     - Atomic spinlock
#     5. fence        - Synchronization barrier
#     6. condvar      - Condition variable
#     7. mutex        - Mutex (depends on condvar)
#     8. semaphore    - Semaphore (depends on condvar)
#
#   PHASE 3: Thread State Machine
#     9.  thread_state      - ThreadState core data
#     10. interrupted       - InterruptedThread + InterruptReason
#     11. ready             - ReadyThread state
#     12. running           - RunningThread state
#     13. sleeping          - SleepingThread state
#     14. zombie            - ZombieThread (thread)
#     15. thread_manager    - ThreadRef/ThreadRefMut, ThreadManager
#
#   PHASE 4: Clock
#     16. clock        - Timer ticks, system time
#
#   PHASE 5: Process State Machine
#     17. process_capability - Capabilities bitfield
#     18. process_state      - ProcessState core data
#     19. runnable           - RunnableProcess
#     20. running_process    - RunningProcess
#     21. sleeping_process   - SleepingProcess
#     22. interrupted_process - InterruptedProcess
#     23. zombie_process     - ZombieProcess
#
#   PHASE 6: Core Scheduler
#     24. process_manager    - ProcessManager (central orchestrator)

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"
WORKFLOW="$PROJECT_ROOT/verus-ai/workflow.py"

# Define all modules in verification order.
# Format: "MODULE_NAME|SOURCE_PATH|OUTPUT_SUBDIR|FILE_STEM"
MODULES=(
    # Phase 1: System Types
    "pid|src/libs/sys/src/sys/pm/pid.rs|kernel/pm/sys|pid"
    "tid|src/libs/sys/src/sys/pm/tid.rs|kernel/pm/sys|tid"
    "sys_capability|src/libs/sys/src/sys/pm/capability.rs|kernel/pm/sys|capability"

    # Phase 2: Synchronization Primitives
    "spinlock|src/kernel/src/pm/sync/spinlock.rs|kernel/pm/sync|spinlock"
    "fence|src/kernel/src/pm/sync/fence.rs|kernel/pm/sync|fence"
    "condvar|src/kernel/src/pm/sync/condvar.rs|kernel/pm/sync|condvar"
    "mutex|src/kernel/src/pm/sync/mutex.rs|kernel/pm/sync|mutex"
    "semaphore|src/kernel/src/pm/sync/semaphore.rs|kernel/pm/sync|semaphore"

    # Phase 3: Thread State Machine
    "thread_state|src/kernel/src/pm/thread/state.rs|kernel/pm/thread|state"
    "interrupted|src/kernel/src/pm/thread/interrupted.rs|kernel/pm/thread|interrupted"
    "ready|src/kernel/src/pm/thread/ready.rs|kernel/pm/thread|ready"
    "running_thread|src/kernel/src/pm/thread/running.rs|kernel/pm/thread|running"
    "sleeping_thread|src/kernel/src/pm/thread/sleeping.rs|kernel/pm/thread|sleeping"
    "zombie_thread|src/kernel/src/pm/thread/zombie.rs|kernel/pm/thread|zombie"
    "thread_manager|src/kernel/src/pm/thread/mod.rs|kernel/pm/thread|thread_manager"

    # Phase 4: Clock
    "clock|src/kernel/src/pm/clock.rs|kernel/pm|clock"

    # Phase 5: Process State Machine
    "process_capability|src/kernel/src/pm/process/capability.rs|kernel/pm/process|capability"
    "process_state|src/kernel/src/pm/process/state/mod.rs|kernel/pm/process/state|process_state"
    "runnable|src/kernel/src/pm/process/state/runnable.rs|kernel/pm/process/state|runnable"
    "running_process|src/kernel/src/pm/process/state/running.rs|kernel/pm/process/state|running"
    "sleeping_process|src/kernel/src/pm/process/state/sleeping.rs|kernel/pm/process/state|sleeping"
    "interrupted_process|src/kernel/src/pm/process/state/interrupted.rs|kernel/pm/process/state|interrupted"
    "zombie_process|src/kernel/src/pm/process/state/zombie.rs|kernel/pm/process/state|zombie"

    # Phase 6: Core Scheduler
    "process_manager|src/kernel/src/pm/process/manager/mod.rs|kernel/pm/process/manager|process_manager"
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
    echo "Scheduler Verification Modules (in dependency order):"
    echo "======================================================"
    for i in "${!MODULES[@]}"; do
        IFS='|' read -r name source output_subdir file_stem <<< "${MODULES[$i]}"
        idx=$((i + 1))
        printf "  %2d. %-25s %s\n" "$idx" "$name" "$source"
    done
    echo ""
    echo "Total: $TOTAL modules"
    exit 0
fi

#==================================================================================================
# Main execution.
#==================================================================================================

echo "============================================================"
echo "  NANVIX SCHEDULER VERIFICATION PIPELINE"
echo "============================================================"
echo ""
echo "Total modules: $TOTAL"
echo "Starting from: $START_FROM"
if [ -n "$ONLY" ]; then
    echo "Running only:  $ONLY"
fi
echo "Output dir:    verus/split/kernel/pm/"
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
echo "  PIPELINE SUMMARY"
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
