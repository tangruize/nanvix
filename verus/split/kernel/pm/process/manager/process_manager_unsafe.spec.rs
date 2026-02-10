// Copyright(c) The Maintainers of Nanvix.
// Licensed under the MIT License.

// ProcessManagerUnsafe Specification.
//
// This file contains spec functions and View types for the ProcessManagerUnsafe model.
//
// ## Verification Model
//
// The unsafe.rs module provides global static access to the ProcessManager singleton.
// It manages context switches, quantum tracking, and delegates to ProcessManagerInner
// for state transitions. For verification, we model the global state as a single struct
// `ProcessManagerUnsafeState` containing:
// - `initialized: bool` — whether PROCESS_MANAGER has been set.
// - `inner: ProcessManagerInner` — the verified inner process manager state.
// - `current_pid: i32` — models the CURRENT_PID atomic.
// - `current_tid: i32` — models the CURRENT_TID atomic.
// - `remaining_quantum: usize` — models the REMAINING_QUANTUM atomic.
// - `fpu_owner_tid: i32` — models the FPU_OWNER_TID atomic.
// - `scheduler_freq: usize` — models the SCHEDULER_FREQ build constant.
// - `ghost_diverged: Ghost<bool>` — ghost flag for machine-checked divergence (T10).
//
// The wf() predicate ties the global atomics to the inner state, ensuring:
// - ghost_diverged == false (state has not diverged via exit/exit_thread).
// - current_pid matches inner.running_pid.
// - current_tid >= 0 (valid thread identifier).
// - remaining_quantum is in [1, scheduler_freq].
// - scheduler_freq > 0.
// - inner.wf() holds.
//
// ## Context Switch Model (switch function)
//
// The original `switch()` (unsafe.rs:758-803) performs:
// 1. `let previous_pid = CURRENT_PID.load()` — reads the OLD PID from the atomic.
// 2. `let previous_tid = CURRENT_TID.load()` — reads the OLD TID from the atomic.
// 3. If next_tid != previous_tid (hard switch):
//    a. If next_pid != previous_pid: reset REMAINING_QUANTUM, store CURRENT_PID.
//    b. Store CURRENT_TID.
//    c. ContextInformation::switch(from, to) — hardware context switch.
// 4. If next_tid == previous_tid (soft switch): no-op or kernel idle.
//
// Crucially, the inner ProcessManagerInner mutation (e.g., exit(), sleep(), schedule())
// happens BEFORE switch() is called, but switch() reads the OLD PID/TID from the
// atomics. This means after inner mutation, inner.running_pid may differ from
// CURRENT_PID — the atomics are stale. switch() detects this staleness via the
// `next_pid != previous_pid` comparison and updates the atomics.
//
// Our model captures this by having switch() accept `new_inner` (the post-mutation
// inner state) and compare `next_pid` against `old(self).current_pid` (the stale
// atomic). The function updates inner, current_pid, current_tid, and quantum
// atomically, restoring wf() consistency.
//
// ## Singleton Access Model (get/get_mut)
//
// The original `get()` and `get_mut()` return references from `static mut
// PROCESS_MANAGER`. For verification, we model the singleton access pattern:
// - `get()` requires `initialized` and ensures wf() (shared reference, no mutation).
// - `get_mut()` requires `initialized` and exclusive access. The exclusive access
//   guarantee comes from the `unsafe` contract: callers must ensure no other
//   references exist. This is a T7 trust boundary — Nanvix is single-core with
//   cooperative scheduling, so re-entrant calls (e.g., from interrupt handlers)
//   are the only source of aliasing, which is prevented by disabling interrupts.
//
// ## Trust Boundary: TID-to-PID Mapping (T9)
//
// The wf() predicate does not include a constraint tying current_tid to a thread
// within current_pid's process. The inner model uses Set<int> for PID-level queues
// and does not track per-process thread sets. The invariant that the current TID
// belongs to the current PID's thread set is maintained by the thread manager
// (T3 boundary). Fully modeling this would require extending ProcessManagerInner
// with a ghost map from PIDs to thread sets — a cross-module change affecting all
// inner operations (create_thread, exit_thread, schedule, etc.). The current model
// verifies all state transitions THIS module performs; TID↔PID membership is an
// orthogonal invariant maintained by the thread manager subsystem.

use vstd::prelude::*;

verus! {

//==================================================================================================
// View Type
//==================================================================================================

/// Abstract view of the global process manager unsafe state.
#[verifier::ext_equal]
pub struct ProcessManagerUnsafeStateView {
    /// Whether the process manager has been initialized.
    pub initialized: bool,
    /// Abstract view of the inner process manager.
    pub inner: ProcessManagerInnerView,
    /// PID of the currently running process (atomic global).
    pub current_pid: int,
    /// TID of the currently running thread (atomic global).
    pub current_tid: int,
    /// Remaining quantum ticks for the current thread.
    pub remaining_quantum: nat,
    /// TID of the thread that owns the FPU.
    pub fpu_owner_tid: int,
    /// Scheduler frequency (quantum size).
    pub scheduler_freq: nat,
    /// Whether an exit/exit_thread has completed (diverged state).
    pub diverged: bool,
}

//==================================================================================================
// Spec Functions
//==================================================================================================

impl ProcessManagerUnsafeState {
    /// Spec: the state has not diverged (no exit/exit_thread completed).
    ///
    /// After a successful exit() or exit_thread(), this becomes false.
    /// Since wf() requires spec_not_diverged(), no further operations
    /// can be called on a diverged state — machine-checked divergence (T10).
    pub open spec fn spec_not_diverged(&self) -> bool {
        self.ghost_diverged@ == false
    }

    /// Spec: the inner process manager is well-formed.
    pub open spec fn spec_inner_wf(&self) -> bool {
        self.inner.wf()
    }

    /// Spec: global current_pid matches inner running_pid.
    pub open spec fn spec_pid_consistent(&self) -> bool {
        self.current_pid as int == self.inner.spec_running_pid()
    }

    /// Spec: current_tid is a valid thread identifier: non-negative and bounded.
    ///
    /// The upper bound `< i32::MAX` provides defense-in-depth: while TIDs are
    /// only compared (not used arithmetically) in this module, bounding them
    /// ensures no overflow if future code adds arithmetic on TIDs.
    pub open spec fn spec_tid_valid(&self) -> bool {
        self.current_tid >= 0i32
        && self.current_tid < i32::MAX
    }

    /// Spec: remaining quantum is in valid range [1, scheduler_freq].
    ///
    /// The lower bound of 1 is correct because:
    /// - The quantum is only decremented when `remaining_quantum > 1` (giveup_no_switch).
    /// - When `remaining_quantum <= 1`, a context switch occurs and quantum is reset
    ///   to `scheduler_freq` (which is >= 1).
    /// - Thus the quantum can never reach 0 during normal operation.
    pub open spec fn spec_quantum_valid(&self) -> bool {
        self.remaining_quantum >= 1
        && self.remaining_quantum <= self.scheduler_freq
    }

    /// Spec: scheduler frequency is positive.
    pub open spec fn spec_freq_positive(&self) -> bool {
        self.scheduler_freq > 0
        && self.scheduler_freq <= usize::MAX
    }

    /// Spec: the FPU owner TID is a valid (non-negative) thread identifier.
    pub open spec fn spec_fpu_owner_valid(&self) -> bool {
        self.fpu_owner_tid >= 0i32
    }

    /// Well-formedness invariant for the global unsafe state.
    ///
    /// Encodes all structural invariants that must hold at all times
    /// after initialization.
    pub open spec fn wf(&self) -> bool {
        self.initialized
        && self.spec_not_diverged()
        && self.spec_inner_wf()
        && self.spec_pid_consistent()
        && self.spec_tid_valid()
        && self.spec_quantum_valid()
        && self.spec_freq_positive()
        && self.spec_fpu_owner_valid()
    }

    /// Spec: the system is not yet initialized.
    pub open spec fn spec_not_initialized(&self) -> bool {
        !self.initialized
    }

    /// Spec: the kernel thread is currently running.
    pub open spec fn spec_is_kernel_running(&self) -> bool {
        self.current_tid == 0i32
    }

    /// Spec: the quantum has expired (remaining <= 1).
    pub open spec fn spec_quantum_expired(&self) -> bool {
        self.remaining_quantum <= 1
    }

    /// Spec: a process with the given PID exists in some queue.
    pub open spec fn spec_process_exists(&self, pid: int) -> bool {
        self.inner.spec_process_exists(pid)
    }
}

//==================================================================================================
// View Implementation
//==================================================================================================

impl View for ProcessManagerUnsafeState {
    type V = ProcessManagerUnsafeStateView;

    open spec fn view(&self) -> ProcessManagerUnsafeStateView {
        ProcessManagerUnsafeStateView {
            initialized: self.initialized,
            inner: self.inner@,
            current_pid: self.current_pid as int,
            current_tid: self.current_tid as int,
            remaining_quantum: self.remaining_quantum as nat,
            fpu_owner_tid: self.fpu_owner_tid as int,
            scheduler_freq: self.scheduler_freq as nat,
            diverged: self.ghost_diverged@,
        }
    }
}

} // verus!
