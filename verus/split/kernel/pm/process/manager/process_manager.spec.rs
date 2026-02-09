// Copyright(c) The Maintainers of Nanvix.
// Licensed under the MIT License.

// ProcessManagerInner Specification.
// This file contains spec functions and View types for the ProcessManagerInner type.
//
// ## Verification Model
//
// The process manager tracks processes across five queues: running (single PID),
// ready, suspended, interrupted, and zombie. For verification, we model each queue
// as a Ghost<Set<int>> paired with a runtime count. The wf() predicate ties the
// ghost sets to the runtime counts and encodes the key invariants:
// - Queues are pairwise disjoint (a process is in exactly one state).
// - The kernel process (PID 0) is always alive (running or ready).
// - All PIDs are bounded by next_pid (monotonic allocation).
// - Counts are bounded to prevent arithmetic overflow.

use vstd::prelude::*;

verus! {

//==================================================================================================
// View Type
//==================================================================================================

/// Abstract view of a ProcessManagerInner.
///
/// Models the logical state of the process manager: which PIDs are in which queue,
/// the next PID to allocate, and configuration/message state.
#[verifier::ext_equal]
pub struct ProcessManagerInnerView {
    /// PID of the currently running process.
    pub running_pid: int,
    /// Set of PIDs in the ready queue.
    pub ready_pids: Set<int>,
    /// Set of PIDs in the suspended queue.
    pub suspended_pids: Set<int>,
    /// Set of PIDs in the interrupted queue.
    pub interrupted_pids: Set<int>,
    /// Set of PIDs in the zombie queue.
    pub zombie_pids: Set<int>,
    /// Next PID to allocate.
    pub next_pid: int,
    /// Whether the platform supports interrupts.
    pub interrupt_capable: bool,
    /// Number of buffered (unconsumed) messages.
    pub number_buffered_messages: nat,
}

//==================================================================================================
// Spec Functions
//==================================================================================================

impl ProcessManagerInner {
    /// Spec function: returns the running process PID.
    pub open spec fn spec_running_pid(&self) -> int {
        self.running_pid as int
    }

    /// Spec function: ghost sets are finite.
    pub open spec fn spec_sets_finite(&self) -> bool {
        self.ghost_ready@.finite()
        && self.ghost_suspended@.finite()
        && self.ghost_interrupted@.finite()
        && self.ghost_zombies@.finite()
    }

    /// Spec function: ghost set sizes match runtime counts.
    pub open spec fn spec_counts_match(&self) -> bool {
        self.ghost_ready@.len() == self.ready_count as nat
        && self.ghost_suspended@.len() == self.suspended_count as nat
        && self.ghost_interrupted@.len() == self.interrupted_count as nat
        && self.ghost_zombies@.len() == self.zombie_count as nat
    }

    /// Spec function: all queues are pairwise disjoint.
    pub open spec fn spec_queues_disjoint(&self) -> bool {
        self.ghost_ready@.disjoint(self.ghost_suspended@)
        && self.ghost_ready@.disjoint(self.ghost_interrupted@)
        && self.ghost_ready@.disjoint(self.ghost_zombies@)
        && self.ghost_suspended@.disjoint(self.ghost_interrupted@)
        && self.ghost_suspended@.disjoint(self.ghost_zombies@)
        && self.ghost_interrupted@.disjoint(self.ghost_zombies@)
    }

    /// Spec function: running PID is not in any queue.
    pub open spec fn spec_running_exclusive(&self) -> bool {
        !self.ghost_ready@.contains(self.running_pid as int)
        && !self.ghost_suspended@.contains(self.running_pid as int)
        && !self.ghost_interrupted@.contains(self.running_pid as int)
        && !self.ghost_zombies@.contains(self.running_pid as int)
    }

    /// Spec function: kernel process (PID 0) safety invariant.
    ///
    /// The kernel is always either running or in the ready queue, and never
    /// in suspended, interrupted, or zombie queues.
    pub open spec fn spec_kernel_safe(&self) -> bool {
        (self.running_pid as int == 0int || self.ghost_ready@.contains(0int))
        && !self.ghost_suspended@.contains(0int)
        && !self.ghost_interrupted@.contains(0int)
        && !self.ghost_zombies@.contains(0int)
    }

    /// Spec function: PID bounds invariant.
    ///
    /// All PIDs are non-negative and strictly less than next_pid.
    /// next_pid >= 1 (since kernel PID 0 is always allocated).
    pub open spec fn spec_pid_bounds(&self) -> bool {
        self.next_pid >= 1i32
        && self.running_pid >= 0i32
        && (self.running_pid as int) < (self.next_pid as int)
        && (forall |pid: int| self.ghost_ready@.contains(pid)
            ==> 0 <= pid && pid < self.next_pid as int)
        && (forall |pid: int| self.ghost_suspended@.contains(pid)
            ==> 0 <= pid && pid < self.next_pid as int)
        && (forall |pid: int| self.ghost_interrupted@.contains(pid)
            ==> 0 <= pid && pid < self.next_pid as int)
        && (forall |pid: int| self.ghost_zombies@.contains(pid)
            ==> 0 <= pid && pid < self.next_pid as int)
    }

    /// Spec function: counts are bounded to prevent arithmetic overflow.
    ///
    /// The total number of processes (running + all queues) is at most i32::MAX,
    /// ensuring any pairwise sum of counts fits in usize.
    pub open spec fn spec_counts_bounded(&self) -> bool {
        (self.ready_count as int) + (self.suspended_count as int)
            + (self.interrupted_count as int) + (self.zombie_count as int) + 1
            <= i32::MAX as int
        && (self.number_buffered_messages as int) < usize::MAX as int
    }

    /// Well-formedness invariant for the process manager.
    ///
    /// Encodes all structural invariants that must hold at all times.
    pub open spec fn wf(&self) -> bool {
        self.spec_sets_finite()
        && self.spec_counts_match()
        && self.spec_queues_disjoint()
        && self.spec_running_exclusive()
        && self.spec_kernel_safe()
        && self.spec_pid_bounds()
        && self.spec_counts_bounded()
    }

    /// Spec function: a process exists in some queue.
    pub open spec fn spec_process_exists(&self, pid: int) -> bool {
        self.running_pid as int == pid
        || self.ghost_ready@.contains(pid)
        || self.ghost_suspended@.contains(pid)
        || self.ghost_interrupted@.contains(pid)
        || self.ghost_zombies@.contains(pid)
    }

    /// Spec function: ready queue is non-empty.
    pub open spec fn spec_has_ready(&self) -> bool {
        self.ready_count > 0
    }

    /// Spec function: zombie queue is non-empty.
    pub open spec fn spec_has_zombies(&self) -> bool {
        self.zombie_count > 0
    }

    /// Spec function: checks if a PID is fresh (not used anywhere).
    pub open spec fn spec_pid_is_fresh(&self, pid: int) -> bool {
        self.running_pid as int != pid
        && !self.ghost_ready@.contains(pid)
        && !self.ghost_suspended@.contains(pid)
        && !self.ghost_interrupted@.contains(pid)
        && !self.ghost_zombies@.contains(pid)
    }

    /// Spec function: the set of all PIDs in the ready queue after inserting
    /// the running PID (models the state after the running process yields).
    pub open spec fn spec_ready_with_running(&self) -> Set<int> {
        self.ghost_ready@.insert(self.running_pid as int)
    }
}

//==================================================================================================
// View Implementation
//==================================================================================================

impl View for ProcessManagerInner {
    type V = ProcessManagerInnerView;

    open spec fn view(&self) -> ProcessManagerInnerView {
        ProcessManagerInnerView {
            running_pid: self.running_pid as int,
            ready_pids: self.ghost_ready@,
            suspended_pids: self.ghost_suspended@,
            interrupted_pids: self.ghost_interrupted@,
            zombie_pids: self.ghost_zombies@,
            next_pid: self.next_pid as int,
            interrupt_capable: self.interrupt_capable,
            number_buffered_messages: self.number_buffered_messages as nat,
        }
    }
}

} // verus!
