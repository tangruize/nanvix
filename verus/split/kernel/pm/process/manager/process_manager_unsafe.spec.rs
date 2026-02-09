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
// - `borrow_count: int` (ghost) — models RefCell borrow state for singleton access.
//
// The wf() predicate ties the global atomics to the inner state, ensuring:
// - current_pid matches inner.running_pid.
// - current_tid >= 0 (valid thread identifier).
// - remaining_quantum is in [1, scheduler_freq].
// - scheduler_freq > 0.
// - inner.wf() holds.
// - borrow_count >= 0 (no outstanding mutable borrows at rest).
//
// ## Context Switch Model (switch function)
//
// The original `switch()` takes raw pointers to ContextInformation and performs
// a hardware context switch. It does NOT modify ProcessManagerInner — the inner
// state mutation happens before switch() is called (in exit(), sleep(), schedule()).
// We model only the *atomic-level effects* of switch():
// - If next_tid != current_tid: hard switch. Update current_tid, and if
//   next_pid != current_pid, also update current_pid and reset quantum.
// - If next_tid == current_tid: soft switch (no-op or kernel idle).
//
// ## Singleton Access Model (get/get_mut)
//
// The original `get()` and `get_mut()` return references from `static mut
// PROCESS_MANAGER`. For verification, we model the singleton access pattern:
// - `get()` requires `initialized` and returns a shared reference (no mutation).
// - `get_mut()` requires `initialized` and exclusive access (borrow_count == 0).
// - `try_borrow_mut()` is the RefCell layer (T7 boundary), modeled by
//   incrementing/decrementing the ghost borrow_count.
//
// ## Trust Boundary: TID-to-PID Mapping (T3)
//
// The wf() predicate does not include a constraint tying current_tid to a thread
// within current_pid's process. The inner model uses Set<int> for PID-level queues
// and does not track per-process thread sets. The invariant that the current TID
// belongs to the current PID's thread set is maintained by the thread manager
// (T3 boundary). Fully modeling this would require extending ProcessManagerInner
// with a ghost map from PIDs to thread sets.

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
    /// Ghost borrow count for singleton access modeling.
    pub borrow_count: int,
}

//==================================================================================================
// Spec Functions
//==================================================================================================

impl ProcessManagerUnsafeState {
    /// Spec: the inner process manager is well-formed.
    pub open spec fn spec_inner_wf(&self) -> bool {
        self.inner.wf()
    }

    /// Spec: global current_pid matches inner running_pid.
    pub open spec fn spec_pid_consistent(&self) -> bool {
        self.current_pid as int == self.inner.spec_running_pid()
    }

    /// Spec: current_tid is a valid (non-negative) thread identifier.
    pub open spec fn spec_tid_valid(&self) -> bool {
        self.current_tid >= 0i32
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

    /// Spec: the ghost borrow count is non-negative (no outstanding mutable borrows at rest).
    pub open spec fn spec_borrow_valid(&self) -> bool {
        self.ghost_borrow_count@ >= 0int
    }

    /// Well-formedness invariant for the global unsafe state.
    ///
    /// Encodes all structural invariants that must hold at all times
    /// after initialization.
    pub open spec fn wf(&self) -> bool {
        self.initialized
        && self.spec_inner_wf()
        && self.spec_pid_consistent()
        && self.spec_tid_valid()
        && self.spec_quantum_valid()
        && self.spec_freq_positive()
        && self.spec_fpu_owner_valid()
        && self.spec_borrow_valid()
    }

    /// Spec: the system is not yet initialized.
    pub open spec fn spec_not_initialized(&self) -> bool {
        !self.initialized
    }

    /// Spec: the kernel thread is currently running.
    pub open spec fn spec_is_kernel_running(&self) -> bool {
        self.current_tid == 0i32
    }

    /// Spec: a hard context switch is needed (different thread).
    pub open spec fn spec_needs_hard_switch(&self, next_tid: int) -> bool {
        next_tid != self.current_tid as int
    }

    /// Spec: a PID change occurs during switch (different process).
    pub open spec fn spec_needs_pid_change(&self, next_pid: int) -> bool {
        next_pid != self.current_pid as int
    }

    /// Spec: the quantum has expired (remaining <= 1).
    pub open spec fn spec_quantum_expired(&self) -> bool {
        self.remaining_quantum <= 1
    }

    /// Spec: a process with the given PID exists in some queue.
    pub open spec fn spec_process_exists(&self, pid: int) -> bool {
        self.inner.spec_process_exists(pid)
    }

    /// Spec: no outstanding mutable borrows (safe to acquire exclusive access).
    pub open spec fn spec_no_borrows(&self) -> bool {
        self.ghost_borrow_count@ == 0int
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
            borrow_count: self.ghost_borrow_count@,
        }
    }
}

} // verus!
