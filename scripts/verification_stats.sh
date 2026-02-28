#!/usr/bin/env bash
# Copyright(c) The Maintainers of Nanvix.
# Licensed under the MIT License.
#
# verification_stats.sh — Collect quantifiable verification statistics.
#
# Usage:
#   ./scripts/verification_stats.sh [NANVIX_DEV_DIR] [VERUS_SPLIT_DIR] [VERUS_BIN]
#
# Defaults:
#   NANVIX_DEV_DIR = ~/nanvix-dev
#   VERUS_SPLIT_DIR = verus/split  (relative to repo root)
#   VERUS_BIN       = ~/verus-x86-linux/verus

set -euo pipefail

REPO_ROOT="$(cd "$(dirname "$0")/.." && pwd)"
NANVIX_DEV="${1:-$HOME/nanvix-dev}"
VERUS_SPLIT="${2:-$REPO_ROOT/verus/split}"
VERUS_BIN="${3:-$HOME/verus-x86-linux/verus}"

# ─── helpers ────────────────────────────────────────────────────────────────────

# SLOC: non-blank, non-comment lines (// and /* */ block-comment continuation).
sloc() { cat "$@" 2>/dev/null | grep -v '^\s*$' | grep -v '^\s*//' | grep -c -v '^\s*\*' || echo 0; }

# Non-blank lines.
nblank() { cat "$@" 2>/dev/null | grep -c -v '^\s*$' || echo 0; }

# ─── source code (from clean nanvix-dev) ────────────────────────────────────────

K="$NANVIX_DEV/src/kernel/src"
L="$NANVIX_DEV/src/libs"

# MM subsystem — files with Verus counterparts (elf.rs excluded).
MM_KERNEL_FILES=(
    "$K"/mm/kheap.rs "$K"/mm/kredzone.rs "$K"/mm/kstack.rs "$K"/mm/ustack.rs "$K"/mm/mod.rs
    "$K"/mm/phys/frame.rs "$K"/mm/phys/kpool.rs "$K"/mm/phys/upool.rs
    "$K"/mm/phys/manager.rs "$K"/mm/phys/mod.rs
    "$K"/mm/virt/kpage.rs "$K"/mm/virt/vmem.rs "$K"/mm/virt/manager.rs "$K"/mm/virt/mod.rs
)
MM_HAL_FILES=("$K"/hal/mem/types/address/frame.rs)
MM_LIB_FILES=("$L"/bitmap/src/lib.rs "$L"/error/src/lib.rs "$L"/raw-array/src/lib.rs "$L"/slab/src/lib.rs)

mm_kernel_sloc=$(sloc "${MM_KERNEL_FILES[@]}")
mm_hal_sloc=$(sloc "${MM_HAL_FILES[@]}")
mm_lib_sloc=$(sloc "${MM_LIB_FILES[@]}")
mm_total_sloc=$((mm_kernel_sloc + mm_hal_sloc + mm_lib_sloc))

# Scheduler subsystem — files with Verus counterparts.
SCHED_PM_KCALL_FILES=(
    "$K"/pm/kcall/create_thread.rs "$K"/pm/kcall/join_thread.rs
    "$K"/pm/kcall/lock_mutex.rs "$K"/pm/kcall/signal_cond.rs "$K"/pm/kcall/sleep.rs
    "$K"/pm/kcall/terminate.rs "$K"/pm/kcall/unlock_mutex.rs "$K"/pm/kcall/wait_cond.rs
    "$K"/pm/kcall/mod.rs
)
SCHED_KCALL_FILES=("$K"/kcall/dispatcher.rs "$K"/kcall/handler.rs "$K"/kcall/mod.rs)
SCHED_SYS_FILES=(
    "$L"/sys/src/sys/pm/capability.rs "$L"/sys/src/sys/pm/pid.rs "$L"/sys/src/sys/pm/tid.rs
)

sched_clock_sloc=$(sloc "$K"/pm/clock.rs)
sched_mod_sloc=$(sloc "$K"/pm/mod.rs)
sched_thread_sloc=$(sloc "$K"/pm/thread/*.rs)
sched_process_sloc=$(sloc "$K"/pm/process/capability.rs "$K"/pm/process/mod.rs \
    "$K"/pm/process/manager/mod.rs "$K"/pm/process/manager/unsafe.rs \
    "$K"/pm/process/state/*.rs)
sched_sync_sloc=$(sloc "$K"/pm/sync/*.rs)
sched_pm_kcall_sloc=$(sloc "${SCHED_PM_KCALL_FILES[@]}")
sched_kcall_sloc=$(sloc "${SCHED_KCALL_FILES[@]}")
sched_sys_sloc=$(sloc "${SCHED_SYS_FILES[@]}")
sched_total_sloc=$((sched_clock_sloc + sched_mod_sloc + sched_thread_sloc + sched_process_sloc \
    + sched_sync_sloc + sched_pm_kcall_sloc + sched_kcall_sloc + sched_sys_sloc))

src_total_sloc=$((mm_total_sloc + sched_total_sloc))

# ─── Verus proof code ──────────────────────────────────────────────────────────

V="$VERUS_SPLIT"

verus_spec_sloc=$(find "$V" -name '*.spec.rs' -not -path '*/backup/*' -not -path '*/scripts/*' \
    -exec cat {} + | grep -v '^\s*$' | grep -v '^\s*//' | grep -c -v '^\s*\*' || echo 0)
verus_proof_sloc=$(find "$V" -name '*.proof.rs' -not -path '*/backup/*' -not -path '*/scripts/*' \
    -exec cat {} + | grep -v '^\s*$' | grep -v '^\s*//' | grep -c -v '^\s*\*' || echo 0)
verus_impl_sloc=$(find "$V" -name '*.rs' -not -name '*.spec.rs' -not -name '*.proof.rs' \
    -not -name '*.test.rs' -not -path '*/backup/*' -not -path '*/scripts/*' \
    -not -name 'test_extern.rs' -exec cat {} + | grep -v '^\s*$' | grep -v '^\s*//' | grep -c -v '^\s*\*' || echo 0)
verus_test_sloc=$(find "$V" -name '*.test.rs' -not -path '*/backup/*' -not -path '*/scripts/*' \
    -exec cat {} + 2>/dev/null | grep -v '^\s*$' | grep -v '^\s*//' | grep -c -v '^\s*\*' || echo 0)
verus_total_sloc=$((verus_spec_sloc + verus_proof_sloc + verus_impl_sloc + verus_test_sloc))

# ─── proof modules count ───────────────────────────────────────────────────────

proof_modules=$(find "$V" -name '*.proof.rs' -not -path '*/backup/*' -not -path '*/scripts/*' \
    | while read -r f; do
    dir=$(dirname "$f"); base=$(basename "$f" .proof.rs)
    echo "$dir/$base"
done | sort -u | wc -l)

mm_modules=$(find "$V" -name '*.proof.rs' -not -path '*/backup/*' -not -path '*/scripts/*' \
    | while read -r f; do
    dir=$(dirname "$f"); base=$(basename "$f" .proof.rs)
    reldir=$(echo "$dir" | sed "s|.*verus/split/||")
    echo "$reldir/$base"
done | sort -u | grep -cE 'kernel/mm|kernel/hal|libs/bitmap|libs/error|libs/raw_array|libs/slab' || echo 0)

sched_modules=$(find "$V" -name '*.proof.rs' -not -path '*/backup/*' -not -path '*/scripts/*' \
    | while read -r f; do
    dir=$(dirname "$f"); base=$(basename "$f" .proof.rs)
    reldir=$(echo "$dir" | sed "s|.*verus/split/||")
    echo "$reldir/$base"
done | sort -u | grep -cE 'kernel/pm|kernel/kcall|libs/sys' || echo 0)

# ─── correctness metrics ───────────────────────────────────────────────────────

assume_count=$(grep -r 'assume(' "$V" --include='*.rs' --exclude-dir=backup --exclude-dir=scripts \
    | grep -v test_extern | grep -v '^\s*//' | grep -v '//!' | wc -l)

external_body_count=$(grep -rP '^\s*#\[verifier::external_body\]' "$V" --include='*.rs' \
    --exclude-dir=backup --exclude-dir=scripts | grep -v test_extern | wc -l)

# ─── run verus (optional) ──────────────────────────────────────────────────────

if [ -x "$VERUS_BIN" ]; then
    verus_output=$(cd "$V" && "$VERUS_BIN" --crate-type lib lib.rs 2>&1 || true)
    verified=$(echo "$verus_output" | grep -oP '\d+ verified' | grep -oP '\d+' || echo "?")
    errors=$(echo "$verus_output" | grep -oP '\d+ errors' | grep -oP '\d+' || echo "?")
    time_output=$( { time (cd "$V" && "$VERUS_BIN" --crate-type lib lib.rs >/dev/null 2>&1); } 2>&1 )
    wall_time=$(echo "$time_output" | grep real | awk '{print $2}')
else
    verified="?"
    errors="?"
    wall_time="(verus not found)"
fi

# ─── proof-to-source ratio ─────────────────────────────────────────────────────

if [ "$src_total_sloc" -gt 0 ]; then
    ratio=$(python3 -c "print(f'{$verus_total_sloc / $src_total_sloc:.1f}')")
else
    ratio="?"
fi

# ─── output ─────────────────────────────────────────────────────────────────────

echo ""
echo "╔══════════════════════════════════════════════════════════════════════════════╗"
echo "║                  Nanvix Formal Verification Statistics                      ║"
echo "╠══════════════════════════════════════════════════════════════════════════════╣"
echo "║  All line counts are SLOC (non-blank, non-comment).                        ║"
echo "╚══════════════════════════════════════════════════════════════════════════════╝"
echo ""

printf "%-30s %s\n" "── Source Code (SLOC) ──" ""
printf "  %-28s %6d   (kernel: %d, hal: %d, libs: %d)\n" \
    "MM subsystem" "$mm_total_sloc" "$mm_kernel_sloc" "$mm_hal_sloc" "$mm_lib_sloc"
printf "  %-28s %6d   (pm: %d, kcall: %d, libs: %d)\n" \
    "Scheduler subsystem" "$sched_total_sloc" \
    "$((sched_clock_sloc+sched_mod_sloc+sched_thread_sloc+sched_process_sloc+sched_sync_sloc+sched_pm_kcall_sloc))" \
    "$sched_kcall_sloc" "$sched_sys_sloc"
printf "  %-28s %6d\n" "TOTAL" "$src_total_sloc"
echo ""

printf "%-30s %s\n" "── Verus Proof Code (SLOC) ──" ""
printf "  %-28s %6d\n" "spec" "$verus_spec_sloc"
printf "  %-28s %6d\n" "proof" "$verus_proof_sloc"
printf "  %-28s %6d\n" "impl" "$verus_impl_sloc"
printf "  %-28s %6d\n" "test" "$verus_test_sloc"
printf "  %-28s %6d\n" "TOTAL" "$verus_total_sloc"
echo ""

printf "%-30s %s\n" "── Modules ──" ""
printf "  %-28s %6d\n" "MM modules" "$mm_modules"
printf "  %-28s %6d\n" "Scheduler modules" "$sched_modules"
printf "  %-28s %6d\n" "TOTAL" "$proof_modules"
echo ""

printf "%-30s %s\n" "── Correctness ──" ""
printf "  %-28s %6s   (%s errors)\n" "Properties proven" "$verified" "$errors"
printf "  %-28s %6d\n" "assume statements" "$assume_count"
printf "  %-28s %6d\n" "external_body" "$external_body_count"
echo ""

printf "%-30s %s\n" "── Efficiency ──" ""
printf "  %-28s %6s\n" "Verification wall time" "$wall_time"
printf "  %-28s %5s:1\n" "Proof-to-source ratio" "$ratio"
echo ""

# ─── CSV output ─────────────────────────────────────────────────────────────────

CSV="$REPO_ROOT/verification_stats.csv"
cat > "$CSV" << CSVEOF
Category,Metric,Value,Notes
Scope,Subsystems Verified,2,Memory management + Scheduler
Scope,Modules with Proofs,$proof_modules,MM: $mm_modules; Scheduler: $sched_modules
Scope,Rust Code Verified (SLOC),$src_total_sloc,MM: $mm_total_sloc; Scheduler: $sched_total_sloc
Scope,Verus Proof Code (SLOC),$verus_total_sloc,spec: $verus_spec_sloc; proof: $verus_proof_sloc; impl: $verus_impl_sloc
Correctness,Properties Proven,$verified,${errors} errors; wall time $wall_time
Correctness,assume Statements,$assume_count,All in raw_array
Correctness,external_body,$external_body_count,See verification_analysis.md for breakdown
Correctness Classes,Verified Classes,6 classes (16 properties),Memory Safety; Concurrency; State Machine; Liveness & Conservation; Kernel Integrity; Access Control
Efficiency,Proof-to-Source Ratio,${ratio}:1,$verus_total_sloc proof / $src_total_sloc source (SLOC)
CSVEOF

echo "CSV written to: $CSV"
