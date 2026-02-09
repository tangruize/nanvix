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
//
// ## Trust Boundary T3: Thread-Level Operations
//
// The following original `ProcessManagerInner` functions are not modeled because
// they operate at the thread level within a process. Their queue-level effects
// are captured by the verified transition functions:
//
// - `create_thread` / `try_add_thread`: Adds a thread to a process. May move a
//    sleeping process to ready (modeled by `wakeup_to_ready`).
// - `wakeup` / `try_wakeup`: Searches all queues for a sleeping thread and wakes
//    it. The queue transition (suspended→ready) is modeled by `wakeup_to_ready`.
// - `exit_thread` (internal branching): Depending on remaining threads, the process
//    goes to ready, suspended, or zombie. Modeled by `exit_thread_running`,
//    `exit_thread_to_suspended`, and `exit_thread_to_zombie` respectively.
// - `set_thread_data_area` / `get_thread_data_area`: Thread metadata; no queue change.
// - `try_join_thread`: Thread join; no queue-level state change.
// - `get_mutex` / `get_cond` / `put_cond` / `put_mutex_guard` / `take_mutex_guard`:
//    Synchronization primitives; no queue-level state change.
// - `find_process` / `find_process_mut` / `find_process_by_tid` / `find_thread_mut`:
//    Query operations; no state change.
// - `handle_fpu_exception`: FPU state management; no queue-level state change.
// - `interrupt_reason`: Returns and clears the interrupt reason; no queue change.
//
// ## Trust Boundary T2: Outer ProcessManager Wrapper
//
// The outer `ProcessManager` (mod.rs:1529-1982) wraps `ProcessManagerInner` in
// `Rc<RefCell<_>>`. The mapping from outer public API to verified inner functions:
//
// - `ProcessManager::get_pid` → reads `running.state().pid()` (query, no mutation)
// - `ProcessManager::get_tid` → reads `running.get_tid()` (query, no mutation)
// - `ProcessManager::create_process` → `inner.create_process` (verified: `create_process`)
// - `ProcessManager::create_thread` → `inner.create_thread` (trust boundary T3)
// - `ProcessManager::set_thread_data_area` → `inner.set_thread_data_area` (T3, no queue change)
// - `ProcessManager::get_thread_data_area` → `inner.get_thread_data_area` (T3, no queue change)
// - `ProcessManager::has_capability` → reads process capability (query, no mutation)
// - `ProcessManager::capctl` → `inner.capctl` (verified: `capctl`, no queue change)
// - `ProcessManager::terminate` → `inner.terminate` (verified: `terminate_ready`,
//    `terminate_ready_stays_ready`, `terminate_suspended`)
// - `ProcessManager::harvest_zombies` → `inner.harvest_zombies` + memory cleanup
//    (verified: `harvest_zombie` for queue transition; memory cleanup is out of scope)
// - `ProcessManager::vmcopy_from_user` / `vmcopy_to_user` → memory ops (no queue change)
// - `ProcessManager::mmap` / `munmap` / `mctrl` → memory management (no queue change)
// - `ProcessManager::mmio_alloc` / `mmio_free` → MMIO management (no queue change)
// - `ProcessManager::attach_pmio` / `detach_pmio` / `read_pmio` / `write_pmio` → PMIO (no queue change)
// - `ProcessManager::post_message` → `inner.post_message` (verified: `post_message`)
// - `ProcessManager::add_event` / `remove_event` → event management (no queue change)
// - `ProcessManager::number_buffered_messages` → reads counter (query, no mutation)
// - `ProcessManager::handle_fpu_exception` → `inner.handle_fpu_exception` (T3, no queue change)
// - `ProcessManager::try_borrow` / `try_borrow_mut` → RefCell borrow (T2)
//
// The `try_borrow`/`try_borrow_mut` pattern returns `Err(ResourceBusy)` on
// contention. Since Nanvix is single-threaded with cooperative scheduling,
// borrow failures can only occur during re-entrant calls (e.g., interrupt
// handlers). This is a runtime safety mechanism, not a formal invariant.
//
// ## Error Path Verification Model
//
// Original functions return `Result<T, Error>` with failure modes including
// process-not-found, kernel-process-rejection, and running-process-rejection.
// The verified model uses preconditions to eliminate error cases (e.g.,
// `terminate_ready` requires `ghost_ready@.contains(pid)`). This is standard
// for verification: preconditions model the conditions under which the operation
// succeeds. Error-handling code paths (which return early with Error objects
// without mutating state) are trivially state-preserving and do not require
// formal verification. Callers must ensure preconditions hold at call sites.
//
// ## Queue Ordering
//
// The original uses `LinkedList` with FIFO ordering and `take_earliest_ready`
// selects by earliest admission time. The verified model uses `Set<int>` which
// abstracts away ordering. This is acceptable for the current verification
// goals (process partitioning, kernel liveness, PID uniqueness). If scheduling
// fairness properties are needed in the future, consider using `Seq<int>` for
// the ready queue.

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
    /// The total number of processes (running + all queues) is at most next_pid,
    /// since each process has a unique PID in [0, next_pid). This ensures any
    /// pairwise sum of counts fits in usize (since next_pid is i32).
    pub open spec fn spec_counts_bounded(&self) -> bool {
        (self.ready_count as int) + (self.suspended_count as int)
            + (self.interrupted_count as int) + (self.zombie_count as int) + 1
            <= self.next_pid as int
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
