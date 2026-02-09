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
// - `create_thread` / `try_add_thread`: Adds a thread to a process. If the process
//    is ready, no queue change (modeled by `create_thread_in_ready`). If sleeping,
//    the process wakes to ready (modeled by `create_thread_from_suspended` which
//    delegates to `wakeup_to_ready`).
// - `wakeup` / `try_wakeup`: Searches all queues for a sleeping thread and wakes
//    it. The queue transition (suspended→ready) is modeled by `wakeup_to_ready`.
//    The running-process case is modeled by `wakeup_running_noop`, the ready-process
//    case by `wakeup_ready_noop`, the not-found error path by `wakeup_not_found`,
//    and the found-but-failed path by `wakeup_suspended_failed_noop`.
// - `exit_thread` (internal branching): Depending on remaining threads, the process
//    goes to ready, suspended, or zombie. Modeled by `exit_thread_running`,
//    `exit_thread_to_suspended`, and `exit_thread_to_zombie` respectively.
//    These branches are mutually exclusive: the process has either (a) remaining
//    runnable threads (→ready), (b) only sleeping threads (→suspended), or
//    (c) all threads are zombie (→zombie). The branch choice is a parameter
//    in the verified model, trusting the thread-level logic to select correctly.
//    `exit_thread_running` also models the Ok path of `exit()` (mod.rs:918-922)
//    where the process-level exit finds remaining runnable threads.
// - `sleep` (internal branching): If the running thread sleeps and other threads
//    are runnable, the process stays ready (modeled by `sleep_thread_running`).
//    If all threads are sleeping, the process moves to suspended (modeled by
//    `sleep_running`). These branches are mutually exclusive.
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
// `Rc<RefCell<_>>`. Every outer method has a verified stub (prefixed `outer_`)
// that models the pass-through delegation and proves wf() preservation.
// The mapping from outer public API to verified inner/outer functions:
//
// - `ProcessManager::get_pid` → verified: `outer_get_pid`
// - `ProcessManager::get_tid` → verified: `outer_get_tid`
// - `ProcessManager::create_process` → verified: `create_process`
// - `ProcessManager::create_thread` → verified: `create_thread_in_ready`,
//    `create_thread_from_suspended`
// - `ProcessManager::set_thread_data_area` → verified: `set_thread_data_area`
// - `ProcessManager::get_thread_data_area` → verified: `get_thread_data_area`
// - `ProcessManager::has_capability` → verified: `outer_has_capability`
// - `ProcessManager::capctl` → verified: `capctl`, `capctl_error_noop`
// - `ProcessManager::terminate` → verified: `outer_terminate_ready` (unified
//    ready-path with to_zombie branch), `outer_terminate_error` (error paths),
//    `terminate_ready`, `terminate_ready_stays_ready`, `terminate_suspended`
// - `ProcessManager::harvest_zombies` → verified: `harvest_zombie`,
//    `harvest_zombies_wrapper`
// - `ProcessManager::vmcopy_from_user` → verified: `outer_vmcopy_from_user`
// - `ProcessManager::vmcopy_to_user` → verified: `outer_vmcopy_to_user`
// - `ProcessManager::mmap` → verified: `outer_mmap`
// - `ProcessManager::munmap` → verified: `outer_munmap`
// - `ProcessManager::mctrl` → verified: `outer_mctrl`
// - `ProcessManager::mmio_alloc` → verified: `outer_mmio_alloc`
// - `ProcessManager::mmio_free` → verified: `outer_mmio_free`
// - `ProcessManager::attach_pmio` → verified: `outer_attach_pmio`
// - `ProcessManager::detach_pmio` → verified: `outer_detach_pmio`
// - `ProcessManager::read_pmio` → verified: `outer_read_pmio`
// - `ProcessManager::write_pmio` → verified: `outer_write_pmio`
// - `ProcessManager::post_message` → verified: `outer_post_message`,
//    `outer_post_message_not_found`, `post_message`, `post_message_not_found`
// - `ProcessManager::add_event` → verified: `outer_add_event`
// - `ProcessManager::remove_event` → verified: `outer_remove_event`
// - `ProcessManager::number_buffered_messages` → verified: `outer_number_buffered_messages`,
//    `get_buffered_message_count`
// - `ProcessManager::handle_fpu_exception` → verified: `handle_fpu_exception`
// - `ProcessManager::try_borrow` / `try_borrow_mut` → RefCell borrow (T2 runtime)
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
// Notable error paths explicitly documented:
// - `terminate` for the running process (mod.rs:1044-1049): returns
//    `Err(InvalidArgument)`. The running process cannot be terminated; this is
//    a precondition violation (no queue change).
// - `wakeup`/`try_wakeup` not-found (mod.rs:802-811): returns `Err(NoSuchEntry)`.
//    Modeled by `wakeup_not_found` (no queue change).
// - `try_wakeup` found-but-failed (mod.rs:832): the thread is found in a
//    suspended process but the wakeup fails (e.g., thread is not sleeping).
//    Process stays suspended. Modeled by `wakeup_suspended_failed_noop`.
//
// ## Queue Ordering and LinkedList→Set Abstraction
//
// The original uses `LinkedList` with FIFO ordering and `take_earliest_ready`
// selects by earliest admission time. The verified model uses `Set<int>` which
// abstracts away ordering. This is acceptable for the current verification
// goals (process partitioning, kernel liveness, PID uniqueness). If scheduling
// fairness properties are needed in the future, consider using `Seq<int>` for
// the ready queue. The `chosen_next` parameter in `schedule` and `full_schedule`
// corresponds to the result of `take_earliest_ready`; the choice is abstracted
// as a precondition (trust boundary T1).
//
// The `LinkedList<Process>` → `Set<int>` abstraction is justified by Rust's
// affine type system: process handles (RunningProcess, RunnableProcess, etc.)
// are move-only types that cannot be duplicated. This means each process can
// appear in exactly one queue at a time, matching the `Set<int>` disjointness
// model. The formal link between the concrete `LinkedList` and the ghost `Set`
// relies on this language-level uniqueness guarantee.
//
// ## Scheduler Composition
//
// The original `schedule()` (mod.rs:640-678) performs: push running→ready,
// check_alarm (suspended→interrupted for expired alarms), resume all
// interrupted→ready, take_earliest_ready→running. The verified model provides:
// - `alarm_interrupt`: individual suspended→interrupted transition.
// - `resume_all_interrupted`: batch interrupted→ready transition.
// - `schedule`: running↔ready swap.
// - `full_schedule`: composes resume_all_interrupted + schedule in one step.
// Callers model the full original schedule as: zero or more `alarm_interrupt`
// calls (for each expired alarm), then one `full_schedule` call.

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

    /// Spec function: whether a new process can be created.
    ///
    /// Returns true iff PID space is not exhausted. Nanvix uses monotonic PID
    /// allocation (PIDs are never recycled), so the system has a hard upper
    /// bound of `i32::MAX` (~2 billion) processes over its lifetime. This is
    /// a known design limitation: once `next_pid == i32::MAX`, no new processes
    /// can be created. To support longer-running systems, PID recycling from
    /// harvested zombies would be needed (future work).
    pub open spec fn spec_can_create_process(&self) -> bool {
        (self.next_pid as int) < i32::MAX as int
    }

    /// Spec function: the set of all PIDs in the ready queue after inserting
    /// the running PID (models the state after the running process yields).
    pub open spec fn spec_ready_with_running(&self) -> Set<int> {
        self.ghost_ready@.insert(self.running_pid as int)
    }

    /// Spec function: the full pool of schedulable PIDs after merging interrupted
    /// into ready and adding the running PID.
    ///
    /// Models the set of candidates for `take_earliest_ready` in the original
    /// `schedule()`, which is called after `resume_all_interrupted()`.
    pub open spec fn spec_full_schedule_pool(&self) -> Set<int> {
        self.ghost_ready@.union(self.ghost_interrupted@).insert(self.running_pid as int)
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
