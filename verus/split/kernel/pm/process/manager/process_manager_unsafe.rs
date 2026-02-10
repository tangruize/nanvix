// Copyright(c) The Maintainers of Nanvix.
// Licensed under the MIT License.

//! # ProcessManagerUnsafe Verification Model
//!
//! Verified model of the global unsafe ProcessManager interface (unsafe.rs).
//!
//! ## Verified Properties
//!
//! - Initialization produces a well-formed global state (init).
//! - Singleton accessors (get/get_mut) require initialization and preserve wf().
//!   get_mut() exclusive access is enforced via the `unsafe` contract (T7).
//! - Context switch (switch) correctly models the original code flow:
//!   inner mutation + atomic updates in one step, comparing next_pid against
//!   the OLD current_pid (pre-mutation) to detect PID changes and reset quantum.
//! - Quantum management (giveup) preserves wf(): unified entry point covering
//!   both decrement and context-switch paths with complete postconditions.
//! - Sleep preserves wf(): delegates to switch for inner+atomic updates.
//!   Post-wakeup interrupt-reason check modeled by sleep_post_wakeup().
//! - Exit/exit_thread preserve wf() through switch, then set ghost_diverged flag.
//!   Machine-checked divergence: wf() requires !diverged, so no operations
//!   can be called after exit. Hard-switch precondition made explicit.
//! - is_kernel_running() correctly reflects current_tid == 0.
//! - Delegation functions (get_mutex, get_cond, etc.) preserve wf() and model
//!   Result-like success/failure outcomes.
//! - try_recv with thread-specific message decrement preserves wf().
//! - Wakeup preserves wf() by delegating to inner wakeup.
//! - join_thread models all three paths: harvest, condvar-wait, and error.
//!
//! ## Verification Model
//!
//! The original module uses `static mut PROCESS_MANAGER: Option<ProcessManager>`,
//! plus atomic globals (CURRENT_PID, CURRENT_TID, REMAINING_QUANTUM, FPU_OWNER_TID).
//! We model this as a single `ProcessManagerUnsafeState` struct that contains
//! a `ProcessManagerInner` (from the verified process_manager module) plus the
//! global atomic state.
//!
//! ## Trust Boundaries
//!
//! - **T5: Raw pointer context switch.** The actual `ContextInformation::switch(from, to)`
//!   is a hardware-level operation. We verify the state-level effects (inner mutation,
//!   PID/TID/quantum updates) but not the raw pointer manipulation. The `user_tda`
//!   parameter (user-space thread data area virtual address) is also abstracted away;
//!   it affects address space setup for the next thread but not queue-level state.
//! - **T6: Atomic ordering.** Atomic loads/stores use `ORDER` (Relaxed/SeqCst).
//!   Memory ordering correctness is not modeled; Nanvix is single-core cooperative.
//! - **T7: Singleton exclusive access.** `get_mut()` returns `&mut ProcessManager` from
//!   `static mut`. Exclusive access is enforced by the `unsafe` contract: callers must
//!   ensure no other references exist. In Nanvix, this is guaranteed by disabling
//!   interrupts (single-core, cooperative scheduling). The RefCell `try_borrow_mut()`
//!   provides a runtime check; re-entrant calls return `Err(ResourceBusy)`.
//! - **T8: Interrupt enable/disable.** `Interrupts::enable()` and `interrupts.wait()`
//!   in the kernel-idle path are HAL operations, modeled as external.
//! - **T9: TID-to-PID mapping (PID↔TID membership).** The invariant that current_tid
//!   belongs to current_pid's thread set is maintained by the thread manager (T3
//!   boundary from inner module). The inner ProcessManagerInner model uses `Set<int>`
//!   for PID-level queue tracking and does NOT store per-process thread sets. Extending
//!   wf() with a `current_tid ∈ threads(current_pid)` constraint would require adding
//!   a ghost `Map<int, Set<int>>` (PID→thread set) to ProcessManagerInner and propagating
//!   it through all inner operations (create_thread, exit_thread, schedule, etc.) — a
//!   cross-module change that is outside the scope of this module's verification. The
//!   current model verifies all state transitions that THIS module performs (inner
//!   mutation + atomic updates); the TID↔PID membership is an orthogonal invariant
//!   maintained by a different subsystem.
//! - **T10: Divergence (exit/exit_thread non-returning) — Machine-Checked.**
//!   exit() and exit_thread() return `Result<!, Error>` in the original code: on the
//!   success path, `Self::switch()` performs a hardware context switch that swaps the
//!   stack pointer and never returns to the caller. The verified model captures
//!   divergence via the `ghost_diverged` flag: after exit/exit_thread, the flag is
//!   set to true, which invalidates wf() (wf requires spec_not_diverged()). Since all
//!   other operations require wf(), no further operations can be called on a diverged
//!   state — providing machine-checked divergence prevention. The postconditions of
//!   exit/exit_thread describe the system state as seen by the NEXT scheduled process
//!   (for global invariant reasoning), not the exiting process's continuation.
//! - **T11: Per-thread message delivery.** try_recv_some/try_recv_none model message
//!   reception as a count decrement. The inner model tracks only
//!   `number_buffered_messages: usize` (a per-process aggregate count), not per-thread
//!   message queues. Verifying that a specific TID receives the correct message would
//!   require extending ProcessManagerInner with ghost per-thread message queues — a
//!   cross-module concern outside this module's scope.
//! - **T12: Synchronization object state.** Delegation functions (get_mutex, get_cond,
//!   put_mutex_guard, put_cond, take_mutex_guard) are thin wrappers that delegate to
//!   ProcessManagerInner methods. The synchronization object tables and guard ownership
//!   state live inside the inner module and are not duplicated here. This module verifies
//!   that delegation preserves wf() (no queue-level side effects); the actual mutex/condvar
//!   correctness is verified in the inner module.
//! - **T13: Uninitialized panic.** get() and get_mut() panic if called before init().
//!   The model requires `wf()` (which includes `initialized == true`) as a precondition,
//!   so the uninitialized path is excluded by construction. This is standard Verus practice:
//!   precondition violations (programming errors) are not modeled as execution paths.
//!
//! ## Scope and Limitations
//!
//! This module verifies the **unsafe wrapper layer**: the global singleton lifecycle,
//! atomic PID/TID/quantum management, and context-switch control flow. The verification
//! boundary is deliberately scoped to state that this module owns or mutates directly.
//!
//! **What is verified:**
//! - All state transitions preserve wf() (32 verified functions, 0 errors).
//! - switch() correctly models stale-atomic PID comparison and quantum reset.
//! - giveup() correctly branches on quantum and preserves/updates state.
//! - No `assume` or `external_body` in this module.
//!
//! **What is deferred to dependency modules (cross-module concerns):**
//! - PID↔TID thread membership (T9): ProcessManagerInner tracks PIDs in `Set<int>`
//!   queues but has no per-process thread sets. Adding `current_tid ∈ threads(current_pid)`
//!   to wf() requires a `Ghost<Map<int, Set<int>>>` in ProcessManagerInner plus updates
//!   to all 96 verified inner functions — a separate verification task.
//! - Per-thread message queues (T11): Inner model uses aggregate `number_buffered_messages`.
//! - Synchronization object tables (T12): Mutex/condvar state lives inside inner module.
//!
//! **What is beyond Verus expressiveness:**
//! - Temporal/liveness properties: join_thread loop termination depends on eventual
//!   notify_all, which requires fair scheduling assumptions outside first-order logic.

use vstd::prelude::*;

// Include the inner ProcessManagerInner from the verified module.
use crate::kernel::pm::process::manager::process_manager::ProcessManagerInner;
use crate::kernel::pm::process::manager::process_manager::ProcessManagerInnerView;

// Include specifications.
include!("process_manager_unsafe.spec.rs");

// Include proofs.
include!("process_manager_unsafe.proof.rs");

verus! {

//==================================================================================================
// Constants
//==================================================================================================

/// Kernel process identifier (PID 0).
pub const KERNEL_PID_RAW: i32 = 0;

/// Kernel thread identifier (TID 0).
pub const KERNEL_TID_RAW: i32 = 0;

//==================================================================================================
// Structures
//==================================================================================================

/// Verification model of the global unsafe process manager state.
///
/// Models the `static mut PROCESS_MANAGER`, `CURRENT_PID`, `CURRENT_TID`,
/// `REMAINING_QUANTUM`, and `FPU_OWNER_TID` globals from unsafe.rs.
pub struct ProcessManagerUnsafeState {
    /// Whether the process manager has been initialized.
    pub initialized: bool,
    /// Inner process manager state (verified model).
    pub inner: ProcessManagerInner,
    /// Current process identifier (models CURRENT_PID atomic).
    pub current_pid: i32,
    /// Current thread identifier (models CURRENT_TID atomic).
    pub current_tid: i32,
    /// Remaining quantum for the current thread (models REMAINING_QUANTUM atomic).
    pub remaining_quantum: usize,
    /// FPU owner thread identifier (models FPU_OWNER_TID atomic).
    pub fpu_owner_tid: i32,
    /// Scheduler frequency / quantum size (models SCHEDULER_FREQ constant).
    pub scheduler_freq: usize,
    /// Ghost flag for machine-checked divergence (T10).
    /// Set to true by exit()/exit_thread() to prevent post-exit reasoning.
    /// wf() requires ghost_diverged@ == false, so after exit, no further
    /// operations can be called on this state.
    pub ghost_diverged: Ghost<bool>,
}

//==================================================================================================
// Implementations
//==================================================================================================

impl ProcessManagerUnsafeState {

    //==============================================================================================
    // Initialization
    //==============================================================================================

    /// Models `ProcessManager::init()` (unsafe.rs:125-143).
    ///
    /// Initializes the global process manager. The kernel process (PID 0) starts
    /// as the running process with TID 0. REMAINING_QUANTUM is set to SCHEDULER_FREQ.
    ///
    /// The original code:
    /// 1. Checks PROCESS_MANAGER is None (panics if Some).
    /// 2. Creates ProcessManagerInner via Rc<RefCell<_>>.
    /// 3. Sets PROCESS_MANAGER = Some(ProcessManager(pm.clone())).
    /// 4. Returns ProcessManager(pm).
    pub fn init(scheduler_freq: usize, interrupt_capable: bool) -> (result: Self)
        requires
            scheduler_freq > 0,
            scheduler_freq <= usize::MAX,
        ensures
            result.wf(),
            result.initialized,
            result.current_pid == KERNEL_PID_RAW,
            result.current_tid == KERNEL_TID_RAW,
            result.remaining_quantum == scheduler_freq,
            result.inner.spec_running_pid() == KERNEL_PID_RAW as int,
    {
        let inner: ProcessManagerInner = ProcessManagerInner::new(interrupt_capable);

        proof {
            ProcessManagerUnsafeState::lemma_init_is_wf(scheduler_freq, interrupt_capable);
        }

        ProcessManagerUnsafeState {
            initialized: true,
            inner: inner,
            current_pid: KERNEL_PID_RAW,
            current_tid: KERNEL_TID_RAW,
            remaining_quantum: scheduler_freq,
            fpu_owner_tid: KERNEL_TID_RAW,
            scheduler_freq: scheduler_freq,
            ghost_diverged: Ghost(false),
        }
    }

    //==============================================================================================
    // Singleton Accessors
    //==============================================================================================

    /// Models `ProcessManager::get()` (unsafe.rs:160-167).
    ///
    /// Returns a shared reference to the process manager.
    /// The original panics if PROCESS_MANAGER is None.
    ///
    /// Shared (immutable) borrow: multiple get() calls can coexist.
    pub fn get(&self)
        requires
            self.wf(),
        ensures
            self.wf(),
            self.initialized,
            self.spec_inner_wf(),
    {
    }

    /// Models `ProcessManager::get_mut()` (unsafe.rs:184-191).
    ///
    /// Returns an exclusive mutable reference to the process manager.
    /// The original panics if PROCESS_MANAGER is None.
    ///
    /// Exclusive access is enforced by the `unsafe` contract (T7): callers must
    /// ensure no other references exist. In Nanvix, this is guaranteed by disabling
    /// interrupts before calling get_mut() (single-core, cooperative scheduling).
    /// The RefCell `try_borrow_mut()` provides an additional runtime check.
    pub fn get_mut(&self)
        requires
            self.wf(),
        ensures
            self.wf(),
            self.initialized,
            self.spec_inner_wf(),
    {
    }

    //==============================================================================================
    // Global Accessors
    //==============================================================================================

    /// Models `ProcessManager::is_kernel_running()` (unsafe.rs:726-728).
    ///
    /// Returns true if the kernel thread (TID 0) is currently running.
    pub fn is_kernel_running(&self) -> (result: bool)
        requires
            self.wf(),
        ensures
            result == self.spec_is_kernel_running(),
            result == (self.current_tid == KERNEL_TID_RAW),
    {
        self.current_tid == KERNEL_TID_RAW
    }

    //==============================================================================================
    // Context Switch (Inner Mutation + Atomic Updates)
    //==============================================================================================

    /// Models the combined inner mutation + `ProcessManager::switch()` (unsafe.rs:758-803).
    ///
    /// In the original code:
    /// 1. Inner mutation (e.g., `try_borrow_mut()?.exit(status)`) changes
    ///    ProcessManagerInner and returns (next_pid, next_tid, from, to, user_tda).
    /// 2. `switch()` reads `previous_pid = CURRENT_PID.load()` (the OLD PID).
    /// 3. `switch()` reads `previous_tid = CURRENT_TID.load()` (the OLD TID).
    /// 4. If next_tid != previous_tid (hard switch):
    ///    a. If next_pid != previous_pid: reset REMAINING_QUANTUM, store CURRENT_PID.
    ///    b. Store CURRENT_TID.
    ///    c. ContextInformation::switch(from, to) — hardware context switch (T5).
    /// 5. If next_tid == previous_tid (soft switch): no-op or kernel idle (T8).
    ///
    /// Crucially, the PID comparison in step 4a is between next_pid and previous_pid
    /// (the OLD atomic value, NOT the new inner.running_pid). The inner state has
    /// already been mutated at this point but the atomics still hold the old values.
    ///
    /// We model this by accepting `new_inner` (post-mutation inner state) and
    /// performing all updates atomically. The PID change detection compares
    /// `next_pid` against `old(self).current_pid` (the stale atomic), matching
    /// the original `CURRENT_PID.load()` behavior.
    pub fn switch(
        &mut self,
        new_inner: ProcessManagerInner,
        next_pid: i32,
        next_tid: i32,
    )
        requires
            old(self).wf(),
            new_inner.wf(),
            new_inner.spec_running_pid() == next_pid as int,
            next_pid >= 0i32,
            next_tid >= 0i32,
            next_tid < i32::MAX,            // Same thread implies same process (scheduler invariant).
            next_tid == old(self).current_tid ==> next_pid == old(self).current_pid,
        ensures
            self.wf(),
            self.initialized == old(self).initialized,
            self.inner == new_inner,
            self.scheduler_freq == old(self).scheduler_freq,
            self.fpu_owner_tid == old(self).fpu_owner_tid,
            // Hard switch: TID updated.
            next_tid != old(self).current_tid ==> self.current_tid == next_tid,
            // Soft switch: TID unchanged.
            next_tid == old(self).current_tid ==> self.current_tid == old(self).current_tid,
            // Hard switch with PID change: PID updated, quantum reset.
            (next_tid != old(self).current_tid && next_pid != old(self).current_pid)
                ==> (self.current_pid == next_pid && self.remaining_quantum == self.scheduler_freq),
            // Hard switch, same PID: PID unchanged, quantum unchanged.
            // Design note: same-PID hard switches (thread switch within same process)
            // intentionally inherit the previous thread's remaining quantum. This matches
            // the original code where REMAINING_QUANTUM is only reset on PID changes
            // (unsafe.rs:775-777). The scheduler treats quantum as per-process, not
            // per-thread: threads within the same process share the process's time slice.
            (next_tid != old(self).current_tid && next_pid == old(self).current_pid)
                ==> (self.current_pid == old(self).current_pid
                     && self.remaining_quantum == old(self).remaining_quantum),
            // Soft switch: PID unchanged, quantum unchanged.
            next_tid == old(self).current_tid
                ==> (self.current_pid == old(self).current_pid
                     && self.remaining_quantum == old(self).remaining_quantum),
    {
        // Update inner state (models the pre-switch inner mutation).
        self.inner = new_inner;

        // Model switch() atomics: compare against the OLD current_pid/current_tid.
        // At this point, self.current_pid and self.current_tid still hold old values
        // (inner update does not touch atomics), matching the original code's
        // `CURRENT_PID.load()` / `CURRENT_TID.load()`.
        if next_tid != self.current_tid {
            // Hard context switch.
            if next_pid != self.current_pid {
                // PID change detected (next_pid != previous_pid): reset quantum.
                self.remaining_quantum = self.scheduler_freq;
                self.current_pid = next_pid;
            }
            self.current_tid = next_tid;
        }
        // Soft switch: no atomic changes; kernel idle path is T8.
    }

    //==============================================================================================
    // Giveup (Voluntary Yield) — Unified Entry Point
    //==============================================================================================

    /// Models `ProcessManager::giveup()` (unsafe.rs:494-522) — no-switch path.
    ///
    /// When remaining quantum > 1, decrement it. No context switch.
    fn giveup_no_switch(&mut self)
        requires
            old(self).wf(),
            old(self).remaining_quantum > 1,
        ensures
            self.wf(),
            self.remaining_quantum == old(self).remaining_quantum - 1,
            self.inner == old(self).inner,
            self.current_pid == old(self).current_pid,
            self.current_tid == old(self).current_tid,
            self.scheduler_freq == old(self).scheduler_freq,
    {
        proof {
            self.lemma_quantum_decrement_preserves_wf();
        }
        self.remaining_quantum = self.remaining_quantum - 1;
    }

    /// Models `ProcessManager::giveup()` (unsafe.rs:494-522) — context-switch path.
    ///
    /// When quantum expired, perform inner.schedule() + switch().
    fn giveup_with_switch(
        &mut self,
        new_inner: ProcessManagerInner,
        chosen_next_pid: i32,
        chosen_next_tid: i32,
    )
        requires
            old(self).wf(),
            old(self).remaining_quantum <= 1,
            new_inner.wf(),
            new_inner.spec_running_pid() == chosen_next_pid as int,
            chosen_next_pid >= 0i32,
            chosen_next_tid >= 0i32,
            chosen_next_tid < i32::MAX,
            chosen_next_tid == old(self).current_tid ==> chosen_next_pid == old(self).current_pid,
        ensures
            self.wf(),
            self.inner == new_inner,
            self.scheduler_freq == old(self).scheduler_freq,
    {
        self.switch(new_inner, chosen_next_pid, chosen_next_tid);
    }

    /// Models `ProcessManager::giveup()` (unsafe.rs:494-522) — unified entry point.
    ///
    /// The original is a single function with an if/else branch on remaining_quantum.
    /// This composite function dispatches to the appropriate path.
    ///
    /// Parameters for the context-switch path are provided but only used when
    /// quantum has expired.
    pub fn giveup(
        &mut self,
        new_inner: ProcessManagerInner,
        chosen_next_pid: i32,
        chosen_next_tid: i32,
    )
        requires
            old(self).wf(),
            // Context-switch path parameters (used only when quantum expired).
            new_inner.wf(),
            new_inner.spec_running_pid() == chosen_next_pid as int,
            chosen_next_pid >= 0i32,
            chosen_next_tid >= 0i32,
            chosen_next_tid < i32::MAX,
            chosen_next_tid == old(self).current_tid ==> chosen_next_pid == old(self).current_pid,
            // If quantum not expired, inner state is unchanged (no mutation on no-switch path).
            old(self).remaining_quantum > 1 ==> new_inner == old(self).inner,
        ensures
            self.wf(),
            self.scheduler_freq == old(self).scheduler_freq,
            // No-switch path: only quantum decremented, all else unchanged.
            old(self).remaining_quantum > 1 ==> (
                self.remaining_quantum == old(self).remaining_quantum - 1
                && self.current_pid == old(self).current_pid
                && self.current_tid == old(self).current_tid
                && self.inner == old(self).inner
            ),
            // Switch path: inner updated, context switch performed.
            old(self).remaining_quantum <= 1 ==> self.inner == new_inner,
    {
        if self.remaining_quantum > 1 {
            self.giveup_no_switch();
        } else {
            self.giveup_with_switch(new_inner, chosen_next_pid, chosen_next_tid);
        }
    }

    /// Models `ProcessManager::giveup()` error path.
    ///
    /// The original giveup() calls `Self::get_mut().try_borrow_mut()?.schedule()?`
    /// on the context-switch path. Both `try_borrow_mut()` and `schedule()` can
    /// fail (returning Err), in which case giveup returns early with no state change.
    /// This function models those error paths.
    pub fn giveup_error(&self)
        requires
            self.wf(),
        ensures
            self.wf(),
    {
    }

    //==============================================================================================
    // Sleep
    //==============================================================================================

    /// Models `ProcessManager::sleep()` pre-switch path (unsafe.rs:438-454).
    ///
    /// Suspends the calling thread. The inner state transitions
    /// (running→suspended or running stays ready if other threads exist)
    /// are modeled by inner.sleep_running() or inner.sleep_thread_running(),
    /// which are already verified. Then switch() is called.
    ///
    /// The `alarm` ghost parameter models the original `alarm: Option<SystemTime>`:
    /// - `alarm@ < 0`: None (indefinite sleep, wakeup only via explicit signal).
    /// - `alarm@ >= 0`: Some(time) (timed sleep, wakeup on timeout or signal).
    /// The alarm affects whether the inner module places the thread on a timed-sleep
    /// queue vs. indefinite-sleep queue. This is captured in `new_inner` (the post-
    /// mutation inner state); the ghost parameter enables callers to reason about
    /// which sleep variant was used.
    pub fn sleep(
        &mut self,
        Ghost(alarm): Ghost<int>,
        new_inner: ProcessManagerInner,
        chosen_next_pid: i32,
        chosen_next_tid: i32,
    )
        requires
            old(self).wf(),
            new_inner.wf(),
            new_inner.spec_running_pid() == chosen_next_pid as int,
            chosen_next_pid >= 0i32,
            chosen_next_tid >= 0i32,
            chosen_next_tid < i32::MAX,
            // Cannot sleep the kernel.
            old(self).current_pid != KERNEL_PID_RAW,
            // Same thread implies same process.
            chosen_next_tid == old(self).current_tid ==> chosen_next_pid == old(self).current_pid,
        ensures
            self.wf(),
            self.inner == new_inner,
            self.scheduler_freq == old(self).scheduler_freq,
    {
        self.switch(new_inner, chosen_next_pid, chosen_next_tid);
    }

    /// Models `ProcessManager::sleep()` post-wakeup path (unsafe.rs:457-468).
    ///
    /// After the thread is woken up (resumed from the context switch), it checks
    /// `interrupt_reason()`. If the thread was interrupted (e.g., by a signal),
    /// sleep returns `Err(SleepError::Interrupted(reason))`. Otherwise, returns Ok(()).
    ///
    /// The interrupt_reason check is a read-only query on the inner state that
    /// clears the interrupt reason. No queue-level state change occurs.
    ///
    /// Parameters:
    /// - `was_interrupted`: trust-boundary input (T3) modeling the result of
    ///   `interrupt_reason()`, which reads from the thread's internal state.
    ///   The correctness of this value depends on the thread manager correctly
    ///   setting/clearing the interrupt reason flag.
    ///
    /// Returns: `was_interrupted` — true if the sleep was interrupted, false if normal wakeup.
    pub fn sleep_post_wakeup(&self, was_interrupted: bool) -> (result: bool)
        requires
            self.wf(),
        ensures
            self.wf(),
            result == was_interrupted,
    {
        was_interrupted
    }

    /// Models `ProcessManager::sleep()` error path.
    ///
    /// The original sleep() calls `Self::get_mut().try_borrow_mut()?.sleep_running()`
    /// which can fail with `try_borrow_mut()` returning Err(ResourceBusy) or
    /// inner sleep operations returning an error. In those cases, no state change
    /// occurs and the error is propagated to the caller.
    pub fn sleep_error(&self)
        requires
            self.wf(),
        ensures
            self.wf(),
    {
    }

    //==============================================================================================
    // Exit Process
    //==============================================================================================

    /// Models `ProcessManager::exit()` (unsafe.rs:221-242).
    ///
    /// Terminates the calling process. The inner state transitions
    /// (running→zombie, ready→running) are modeled by inner.exit_running()
    /// which is already verified. Then switch() is called.
    ///
    /// ## Divergence (T10) — Machine-Checked
    ///
    /// The original function returns `Result<!, Error>`: on the success path,
    /// `Self::switch()` performs a context switch and never returns. This model
    /// captures divergence via the `ghost_diverged` flag: after exit(), the flag
    /// is set to true, which invalidates wf() (wf requires spec_not_diverged()).
    /// Since all other operations require wf(), no further operations can be
    /// called on a diverged state — providing machine-checked divergence prevention.
    ///
    /// The postconditions describe the system state as seen by the NEXT scheduled
    /// process (for global invariant reasoning), not the exiting process.
    pub fn exit(
        &mut self,
        new_inner: ProcessManagerInner,
        chosen_next_pid: i32,
        chosen_next_tid: i32,
    )
        requires
            old(self).wf(),
            new_inner.wf(),
            new_inner.spec_running_pid() == chosen_next_pid as int,
            chosen_next_pid >= 0i32,
            chosen_next_tid >= 0i32,
            chosen_next_tid < i32::MAX,
            // Cannot exit the kernel process or from the kernel thread.
            // The original doc says "calling thread is not a kernel thread" (TID check).
            // We also check PID because exiting a process exits all its threads, and the
            // kernel process (PID 0) must never be exited. Both checks are needed:
            // - PID check: prevents exiting the kernel process (structural invariant).
            // - TID check: matches the original safety contract (no kernel thread exit).
            old(self).current_pid != KERNEL_PID_RAW,
            old(self).current_tid != KERNEL_TID_RAW,
            // Exit always switches to a different thread (hard switch).
            chosen_next_tid != old(self).current_tid,
            // Same thread implies same process (vacuously true given above).
            chosen_next_tid == old(self).current_tid ==> chosen_next_pid == old(self).current_pid,
        ensures
            // Diverged: wf() no longer holds — no further operations possible.
            self.ghost_diverged@ == true,
            // System state for global invariant reasoning:
            self.inner == new_inner,
            self.scheduler_freq == old(self).scheduler_freq,
            self.current_pid == chosen_next_pid,
            self.current_tid == chosen_next_tid,
            (chosen_next_pid != old(self).current_pid)
                ==> self.remaining_quantum == self.scheduler_freq,
    {
        self.switch(new_inner, chosen_next_pid, chosen_next_tid);
        self.ghost_diverged = Ghost(true);
    }

    /// Models `ProcessManager::exit()` error path.
    ///
    /// The original exit() calls `Self::get_mut().try_borrow_mut()?.exit(status)`
    /// which can fail with `try_borrow_mut()` returning Err(ResourceBusy). In that
    /// case, no state change occurs and the error is returned to the caller.
    /// The `status: i32` exit code parameter is consumed by the inner exit() call
    /// and captured in the `new_inner` on the success path; on the error path it
    /// is irrelevant.
    pub fn exit_error(&self)
        requires
            self.wf(),
        ensures
            self.wf(),
    {
    }

    //==============================================================================================
    // Exit Thread
    //==============================================================================================

    /// Models `ProcessManager::exit_thread()` (unsafe.rs:272-308).
    ///
    /// Terminates the calling thread. The inner state transitions depend on
    /// remaining threads (modeled by exit_thread_running, exit_thread_to_suspended,
    /// or exit_thread_to_zombie in the inner module). Then switch() is called.
    ///
    /// ## Divergence (T10) — Machine-Checked
    ///
    /// Same as exit(): sets ghost_diverged to true, invalidating wf().
    /// See exit() documentation for full explanation.
    pub fn exit_thread(
        &mut self,
        new_inner: ProcessManagerInner,
        chosen_next_pid: i32,
        chosen_next_tid: i32,
    )
        requires
            old(self).wf(),
            new_inner.wf(),
            new_inner.spec_running_pid() == chosen_next_pid as int,
            chosen_next_pid >= 0i32,
            chosen_next_tid >= 0i32,
            chosen_next_tid < i32::MAX,
            // Cannot exit the kernel thread.
            old(self).current_tid != KERNEL_TID_RAW,
            // Exit thread always switches to a different thread (hard switch).
            chosen_next_tid != old(self).current_tid,
            // Same thread implies same process (vacuously true given above).
            chosen_next_tid == old(self).current_tid ==> chosen_next_pid == old(self).current_pid,
        ensures
            // Diverged: wf() no longer holds — no further operations possible.
            self.ghost_diverged@ == true,
            // System state for global invariant reasoning:
            self.inner == new_inner,
            self.scheduler_freq == old(self).scheduler_freq,
            self.current_pid == chosen_next_pid,
            self.current_tid == chosen_next_tid,
            (chosen_next_pid != old(self).current_pid)
                ==> self.remaining_quantum == self.scheduler_freq,
    {
        self.switch(new_inner, chosen_next_pid, chosen_next_tid);
        self.ghost_diverged = Ghost(true);
    }

    /// Models `ProcessManager::exit_thread()` error path.
    ///
    /// Same as exit_error(): `try_borrow_mut()` failure returns Err with no state change.
    pub fn exit_thread_error(&self)
        requires
            self.wf(),
        ensures
            self.wf(),
    {
    }

    //==============================================================================================
    // Delegation Functions (No Queue-Level State Change)
    //==============================================================================================

    /// Models `ProcessManager::get_mutex()` (unsafe.rs:547-549).
    ///
    /// Delegates to inner.get_mutex(). Returns Ok(Mutex) or Err(Error).
    /// No queue-level state change regardless of success or failure.
    ///
    /// ## Trust Boundary (T12)
    ///
    /// Mutex/condvar table state and guard ownership live in ProcessManagerInner.
    /// This module verifies delegation preserves wf(); actual synchronization
    /// correctness is verified in the inner module.
    ///
    /// Parameter `succeeds` models whether the operation succeeds (true) or
    /// fails with an error (false). The queue-level state is unchanged in both cases.
    /// The `addr` ghost parameter identifies which mutex is being accessed,
    /// enabling callers to distinguish between different get_mutex calls.
    pub fn get_mutex(&self, Ghost(addr): Ghost<int>, succeeds: bool) -> (result: bool)
        requires
            self.wf(),
            addr >= 0,
        ensures
            self.wf(),
            result == succeeds,
    {
        succeeds
    }

    /// Models `ProcessManager::put_mutex_guard()` (unsafe.rs:569-577).
    ///
    /// Delegates to inner.put_mutex_guard(). Returns Ok(()) or Err(Error).
    /// No queue-level state change.
    pub fn put_mutex_guard(&self, Ghost(addr): Ghost<int>, succeeds: bool) -> (result: bool)
        requires
            self.wf(),
            addr >= 0,
        ensures
            self.wf(),
            result == succeeds,
    {
        succeeds
    }

    /// Models `ProcessManager::get_cond()` (unsafe.rs:602-604).
    ///
    /// Delegates to inner.get_cond(). Returns Ok(Condvar) or Err(Error).
    /// No queue-level state change.
    pub fn get_cond(&self, Ghost(addr): Ghost<int>, succeeds: bool) -> (result: bool)
        requires
            self.wf(),
            addr >= 0,
        ensures
            self.wf(),
            result == succeeds,
    {
        succeeds
    }

    /// Models `ProcessManager::put_cond()` (unsafe.rs:627-629).
    ///
    /// Delegates to inner.put_cond(). Returns Ok(()) or Err(Error).
    /// No queue-level state change.
    pub fn put_cond(&self, Ghost(addr): Ghost<int>, succeeds: bool) -> (result: bool)
        requires
            self.wf(),
            addr >= 0,
        ensures
            self.wf(),
            result == succeeds,
    {
        succeeds
    }

    /// Models `ProcessManager::wakeup()` (unsafe.rs:678-681).
    ///
    /// Delegates to inner.wakeup(tid). Queue-level effects (suspended→ready)
    /// are verified in the inner module. The running PID does not change
    /// (wakeup does not perform a context switch).
    ///
    /// The `woken_tid` ghost parameter models the original `tid: ThreadIdentifier`:
    /// the specific thread being woken. The inner wakeup moves this thread from
    /// the suspended queue to the ready queue. The ghost parameter enables callers
    /// to reason about which thread was woken (e.g., in join_thread condvar signaling).
    pub fn wakeup(&mut self, Ghost(woken_tid): Ghost<int>, new_inner: ProcessManagerInner)
        requires
            old(self).wf(),
            new_inner.wf(),
            new_inner.spec_running_pid() == old(self).inner.spec_running_pid(),
            woken_tid >= 0,
        ensures
            self.wf(),
            self.inner == new_inner,
            self.current_pid == old(self).current_pid,
            self.current_tid == old(self).current_tid,
            self.remaining_quantum == old(self).remaining_quantum,
            self.scheduler_freq == old(self).scheduler_freq,
    {
        self.inner = new_inner;
    }

    /// Models `ProcessManager::take_mutex_guard()` (unsafe.rs:707-715).
    ///
    /// Delegates to inner.take_mutex_guard(). Returns Ok(MutexGuard) or Err(Error).
    /// No queue-level state change.
    pub fn take_mutex_guard(&self, Ghost(addr): Ghost<int>, succeeds: bool) -> (result: bool)
        requires
            self.wf(),
            addr >= 0,
        ensures
            self.wf(),
            result == succeeds,
    {
        succeeds
    }

    //==============================================================================================
    // Message Reception
    //==============================================================================================

    /// Models `ProcessManager::try_recv()` (unsafe.rs:649-659).
    ///
    /// Attempts to receive a message for the specified thread. If a message is
    /// found, decrements number_buffered_messages. Queue-level state is unchanged.
    ///
    /// The `tid` parameter models the thread-specific message reception:
    /// `running.state_mut().receive_message(tid)`. The message is dequeued from
    /// the running process's message buffer for the given TID. The ghost `tid`
    /// parameter ensures callers reason about which thread receives the message.
    ///
    /// ## Trust Boundary (T11)
    ///
    /// The inner model only tracks `number_buffered_messages` as an aggregate count.
    /// Per-thread message queues are not modeled; verifying that a specific TID
    /// receives the correct message requires extending the inner model with ghost
    /// per-thread queues.
    pub fn try_recv_some(&mut self, Ghost(tid): Ghost<int>)
        requires
            old(self).wf(),
            old(self).inner.number_buffered_messages > 0,
            tid >= 0,
        ensures
            self.wf(),
            self.inner.number_buffered_messages
                == old(self).inner.number_buffered_messages - 1,
            self.current_pid == old(self).current_pid,
            self.current_tid == old(self).current_tid,
            self.remaining_quantum == old(self).remaining_quantum,
            self.scheduler_freq == old(self).scheduler_freq,
    {
        proof {
            self.lemma_recv_message_preserves_wf();
        }
        self.inner = ProcessManagerInner {
            number_buffered_messages: (self.inner.number_buffered_messages - 1) as usize,
            ..self.inner
        };
    }

    /// Models `ProcessManager::try_recv()` when no message is available.
    ///
    /// Returns None. No state change.
    pub fn try_recv_none(&self, Ghost(tid): Ghost<int>)
        requires
            self.wf(),
            tid >= 0,
        ensures
            self.wf(),
    {
    }

    //==============================================================================================
    // Join Thread
    //==============================================================================================

    /// Models `ProcessManager::join_thread()` harvest path (unsafe.rs:354-396).
    ///
    /// When the target thread is already a zombie, harvest it. The harvesting
    /// involves unmapping user stack pages (memory management, external to the
    /// queue model). No queue-level state change occurs.
    ///
    /// ## Trust Boundary (T14: Harvest Re-borrow)
    ///
    /// The original harvest loop re-borrows the process manager
    /// (`Self::get_mut().try_borrow_mut()?`) within the page-unmapping loop to
    /// access the process's vmem. This re-borrow can theoretically fail with
    /// Err(ResourceBusy). The error is logged with `warn!` but the harvest
    /// continues (best-effort page unmapping). Since this failure only affects
    /// memory cleanup (not queue-level state) and the queue model does not track
    /// page mappings, we model the harvest as a no-op.
    pub fn join_thread_harvest(&self)
        requires
            self.wf(),
        ensures
            self.wf(),
    {
    }

    /// Models `ProcessManager::join_thread()` condvar-wait path (unsafe.rs:401-402).
    ///
    /// When the target thread is not yet a zombie, the calling thread blocks on
    /// the join condition variable via `join_cond.wait(None)?`. This delegates to
    /// sleep(), which suspends the calling thread until the condvar is signaled.
    ///
    /// The blocking path eventually terminates when the target thread exits and
    /// signals the join condvar. Liveness depends on:
    /// 1. The target thread eventually calls exit_thread(), which transitions it
    ///    to zombie state and calls join_cond.notify_all().
    /// 2. notify_all() wakes the waiting thread, which re-enters the loop and
    ///    finds the zombie thread on the next iteration.
    ///
    /// This liveness argument is a trust boundary: we verify that each iteration
    /// preserves wf(), but do not formally prove termination.
    pub fn join_thread_wait(
        &mut self,
        new_inner: ProcessManagerInner,
        chosen_next_pid: i32,
        chosen_next_tid: i32,
    )
        requires
            old(self).wf(),
            new_inner.wf(),
            new_inner.spec_running_pid() == chosen_next_pid as int,
            chosen_next_pid >= 0i32,
            chosen_next_tid >= 0i32,
            chosen_next_tid < i32::MAX,
            // Cannot sleep the kernel.
            old(self).current_pid != KERNEL_PID_RAW,
            // Same thread implies same process.
            chosen_next_tid == old(self).current_tid ==> chosen_next_pid == old(self).current_pid,
        ensures
            self.wf(),
            self.inner == new_inner,
            self.scheduler_freq == old(self).scheduler_freq,
    {
        // join_cond.wait(None) passes alarm=None (indefinite sleep, modeled as -1).
        self.sleep(Ghost(-1int), new_inner, chosen_next_pid, chosen_next_tid);
    }

    /// Models `ProcessManager::join_thread()` error path.
    ///
    /// Returns Err(SleepError::Generic(error)). No state change.
    pub fn join_thread_error(&self)
        requires
            self.wf(),
        ensures
            self.wf(),
    {
    }

    /// Models `ProcessManager::join_thread()` (unsafe.rs:341-408) — unified entry point.
    ///
    /// The original function is a loop:
    /// ```text
    /// loop {
    ///     match try_join_thread(tid) {
    ///         Ok(Some(status)) => return Ok(status),   // harvest path
    ///         Ok(None) => join_cond.wait(None)?,        // wait path (sleep)
    ///         Err(e) => return Err(SleepError::Generic(e)), // error path
    ///     }
    /// }
    /// ```
    ///
    /// This unified function models one iteration of the loop:
    /// - `outcome == 0`: harvest path — returns Ok(exit_status). Target thread is zombie,
    ///   harvested (pages unmapped — external to queue model). No queue-level state change.
    /// - `outcome == 1`: wait path — returns Ok(0) (placeholder; actual return happens after
    ///   wakeup on a future iteration). Target thread not zombie, block on condvar via sleep().
    /// - `outcome == 2`: error path — returns Err. try_join_thread or condvar-wait failed.
    ///   No queue-level state change.
    ///
    /// Each iteration preserves wf(). The loop terminates when outcome == 0 or 2.
    /// Liveness (eventual outcome == 0) depends on the target thread eventually
    /// calling exit_thread() and signaling the join condvar — a trust boundary.
    ///
    /// ## Return Value Modeling
    ///
    /// The original returns `Result<ExitStatus, SleepError>`. We model this as:
    /// - `result.0`: true = Ok, false = Err
    /// - `result.1`: on Ok (outcome 0), the harvested thread's exit status (ghost input
    ///   `exit_status`); on Ok (outcome 1), 0 (loop continues); on Err, 0 (unused).
    ///
    /// The exit_status value is a T3 trust boundary input: its correctness depends on
    /// the inner module correctly storing the status when the target thread called exit().
    ///
    /// ## Loop Model
    ///
    /// Verus does not natively support loop invariant reasoning. This single-iteration
    /// model is the standard Verus approach: callers prove that wf() is maintained across
    /// each iteration and that terminal outcomes (0 or 2) produce the correct return.
    /// A caller-level proof can compose iterations to show full loop correctness.
    pub fn join_thread(
        &mut self,
        outcome: u8,
        exit_status: i32,
        new_inner: ProcessManagerInner,
        chosen_next_pid: i32,
        chosen_next_tid: i32,
    ) -> (result: (bool, i32))
        requires
            old(self).wf(),
            outcome <= 2,
            // Wait path parameters (used only when outcome == 1).
            new_inner.wf(),
            new_inner.spec_running_pid() == chosen_next_pid as int,
            chosen_next_pid >= 0i32,
            chosen_next_tid >= 0i32,
            chosen_next_tid < i32::MAX,
            // Wait path: cannot sleep the kernel.
            outcome == 1 ==> old(self).current_pid != KERNEL_PID_RAW,
            // Same thread implies same process.
            chosen_next_tid == old(self).current_tid ==> chosen_next_pid == old(self).current_pid,
            // If not wait path, new_inner must match current inner (no mutation).
            outcome != 1 ==> new_inner == old(self).inner,
        ensures
            self.wf(),
            self.scheduler_freq == old(self).scheduler_freq,
            // Return value modeling.
            // Harvest: Ok(exit_status).
            outcome == 0 ==> (result.0 == true && result.1 == exit_status),
            // Wait: Ok(0) — loop continues, actual return on future iteration.
            outcome == 1 ==> (result.0 == true && result.1 == 0),
            // Error: Err.
            outcome == 2 ==> result.0 == false,
            // Harvest path: no queue-level state change.
            outcome == 0 ==> (
                self.inner == old(self).inner
                && self.current_pid == old(self).current_pid
                && self.current_tid == old(self).current_tid
                && self.remaining_quantum == old(self).remaining_quantum
            ),
            // Wait path: inner updated via sleep/switch.
            outcome == 1 ==> self.inner == new_inner,
            // Error path: no queue-level state change.
            outcome == 2 ==> (
                self.inner == old(self).inner
                && self.current_pid == old(self).current_pid
                && self.current_tid == old(self).current_tid
                && self.remaining_quantum == old(self).remaining_quantum
            ),
    {
        if outcome == 1 {
            self.join_thread_wait(new_inner, chosen_next_pid, chosen_next_tid);
            (true, 0i32)
        } else if outcome == 0 {
            (true, exit_status)
        } else {
            (false, 0i32)
        }
    }
}

} // verus!
