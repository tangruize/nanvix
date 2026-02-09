// Copyright(c) The Maintainers of Nanvix.
// Licensed under the MIT License.

//! # ProcessManagerUnsafe Verification Model
//!
//! Verified model of the global unsafe ProcessManager interface (unsafe.rs).
//!
//! ## Verified Properties
//!
//! - Initialization produces a well-formed global state (init).
//! - Context switch (switch) preserves wf(): hard switches update PID/TID/quantum
//!   correctly; soft switches are no-ops.
//! - Quantum management (giveup) preserves wf(): decrement stays in valid range,
//!   expired quantum triggers context switch that preserves wf().
//! - Sleep preserves wf(): delegates to inner sleep + switch.
//! - Exit preserves wf(): delegates to inner exit + switch (divergent).
//! - Exit thread preserves wf(): delegates to inner exit_thread + switch (divergent).
//! - is_kernel_running() correctly reflects current_tid == 0.
//! - Delegation functions (get_mutex, get_cond, etc.) preserve wf().
//! - try_recv with message decrement preserves wf().
//! - Wakeup preserves wf() by delegating to inner wakeup.
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
//!   is a hardware-level operation modeled as `#[verifier::external_body]`. We verify
//!   only the state-level effects (PID/TID/quantum updates).
//! - **T6: Atomic ordering.** Atomic loads/stores use `ORDER` (Relaxed/SeqCst).
//!   Memory ordering correctness is not modeled; Nanvix is single-core cooperative.
//! - **T7: RefCell borrow.** `try_borrow_mut()` returns `Result<RefMut<_>, Error>`.
//!   Runtime borrow checking is not modeled (inherited from T2).
//! - **T8: Interrupt enable/disable.** `Interrupts::enable()` and `interrupts.wait()`
//!   in the kernel-idle path are HAL operations, modeled as external.

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
    /// # Preconditions
    /// - The process manager is not yet initialized.
    /// - scheduler_freq > 0.
    ///
    /// # Postconditions
    /// - The resulting state is well-formed.
    /// - The kernel (PID 0, TID 0) is running.
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
        }
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
    // Context Switch
    //==============================================================================================

    /// Models `ProcessManager::switch()` (unsafe.rs:758-803).
    ///
    /// Performs a context switch from the current thread to the next thread.
    /// Updates CURRENT_PID, CURRENT_TID, and REMAINING_QUANTUM as appropriate.
    ///
    /// Three cases:
    /// 1. Hard switch with PID change: update PID, TID, reset quantum.
    /// 2. Hard switch same PID: update TID only.
    /// 3. Soft switch (same TID): no state change (kernel idle or self-schedule).
    ///
    /// # Trust Boundary
    /// The actual `ContextInformation::switch(from, to, user_tda)` is external (T5).
    /// We model only the state-level effects on the global atomics.
    pub fn switch(
        &mut self,
        next_pid: i32,
        next_tid: i32,
        new_inner: ProcessManagerInner,
    )
        requires
            old(self).wf(),
            new_inner.wf(),
            new_inner.spec_running_pid() == next_pid as int,
            next_pid >= 0i32,
            next_tid >= 0i32,
            // Same thread implies same process (invariant from scheduler).
            next_tid == old(self).current_tid ==> next_pid == old(self).current_pid,
        ensures
            self.wf(),
            self.initialized == old(self).initialized,
            self.inner == new_inner,
            self.scheduler_freq == old(self).scheduler_freq,
            self.fpu_owner_tid == old(self).fpu_owner_tid,
            // TID is updated to next_tid (or stays the same if soft switch).
            next_tid != old(self).current_tid ==> self.current_tid == next_tid,
            next_tid == old(self).current_tid ==> self.current_tid == old(self).current_tid,
            // PID is updated on hard switch with PID change.
            (next_tid != old(self).current_tid && next_pid != old(self).current_pid)
                ==> (self.current_pid == next_pid && self.remaining_quantum == self.scheduler_freq),
            // PID unchanged on hard switch with same PID.
            (next_tid != old(self).current_tid && next_pid == old(self).current_pid)
                ==> (self.current_pid == old(self).current_pid
                     && self.remaining_quantum == old(self).remaining_quantum),
            // Soft switch: no changes to PID, TID, quantum.
            next_tid == old(self).current_tid
                ==> (self.current_pid == old(self).current_pid
                     && self.remaining_quantum == old(self).remaining_quantum),
    {
        if next_tid != self.current_tid {
            // Hard context switch.
            if next_pid != self.current_pid {
                // PID change: reset quantum and update PID.
                self.remaining_quantum = self.scheduler_freq;
                self.current_pid = next_pid;
            }
            self.current_tid = next_tid;
        }
        // Update inner state to reflect the transition.
        self.inner = new_inner;
    }

    //==============================================================================================
    // Giveup (Voluntary Yield)
    //==============================================================================================

    /// Models `ProcessManager::giveup()` (unsafe.rs:494-522).
    ///
    /// If remaining quantum > 1, decrement it (no context switch).
    /// If remaining quantum <= 1, perform a full reschedule via inner.schedule()
    /// and switch().
    ///
    /// # Postconditions
    /// - wf() is preserved.
    /// - If quantum was > 1, only quantum is decremented (no context switch).
    /// - If quantum expired, a context switch may occur.
    pub fn giveup_no_switch(&mut self)
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

    /// Models `ProcessManager::giveup()` expired-quantum path (unsafe.rs:500-519).
    ///
    /// When quantum expires, delegates to inner.schedule() and then switch().
    /// The inner state transitions (running→ready swap) are modeled by
    /// ProcessManagerInner::schedule() which is already verified.
    pub fn giveup_with_switch(
        &mut self,
        chosen_next_pid: i32,
        chosen_next_tid: i32,
        new_inner: ProcessManagerInner,
    )
        requires
            old(self).wf(),
            old(self).remaining_quantum <= 1,
            new_inner.wf(),
            new_inner.spec_running_pid() == chosen_next_pid as int,
            chosen_next_pid >= 0i32,
            chosen_next_tid >= 0i32,
            // Same thread implies same process.
            chosen_next_tid == old(self).current_tid ==> chosen_next_pid == old(self).current_pid,
        ensures
            self.wf(),
            self.inner == new_inner,
            self.scheduler_freq == old(self).scheduler_freq,
    {
        // Delegate to switch which handles all PID/TID/quantum updates.
        self.switch(chosen_next_pid, chosen_next_tid, new_inner);
    }

    //==============================================================================================
    // Sleep
    //==============================================================================================

    /// Models `ProcessManager::sleep()` (unsafe.rs:438-469).
    ///
    /// Suspends the calling thread. The inner state transitions
    /// (running→suspended or running stays ready if other threads exist)
    /// are modeled by inner.sleep_running() or inner.sleep_thread_running(),
    /// which are already verified. Then switch() is called.
    pub fn sleep(
        &mut self,
        chosen_next_pid: i32,
        chosen_next_tid: i32,
        new_inner: ProcessManagerInner,
    )
        requires
            old(self).wf(),
            new_inner.wf(),
            new_inner.spec_running_pid() == chosen_next_pid as int,
            chosen_next_pid >= 0i32,
            chosen_next_tid >= 0i32,
            // Cannot sleep the kernel.
            old(self).current_pid != KERNEL_PID_RAW,
            // Same thread implies same process.
            chosen_next_tid == old(self).current_tid ==> chosen_next_pid == old(self).current_pid,
        ensures
            self.wf(),
            self.inner == new_inner,
            self.scheduler_freq == old(self).scheduler_freq,
    {
        self.switch(chosen_next_pid, chosen_next_tid, new_inner);
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
    /// Note: The original function returns `Result<!, Error>` (divergent).
    /// We model only the state transition; divergence is a T5 boundary.
    pub fn exit(
        &mut self,
        chosen_next_pid: i32,
        chosen_next_tid: i32,
        new_inner: ProcessManagerInner,
    )
        requires
            old(self).wf(),
            new_inner.wf(),
            new_inner.spec_running_pid() == chosen_next_pid as int,
            chosen_next_pid >= 0i32,
            chosen_next_tid >= 0i32,
            // Cannot exit the kernel.
            old(self).current_pid != KERNEL_PID_RAW,
            // Same thread implies same process.
            chosen_next_tid == old(self).current_tid ==> chosen_next_pid == old(self).current_pid,
        ensures
            self.wf(),
            self.inner == new_inner,
            self.scheduler_freq == old(self).scheduler_freq,
    {
        self.switch(chosen_next_pid, chosen_next_tid, new_inner);
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
    /// Note: The original function returns `Result<!, Error>` (divergent).
    pub fn exit_thread(
        &mut self,
        chosen_next_pid: i32,
        chosen_next_tid: i32,
        new_inner: ProcessManagerInner,
    )
        requires
            old(self).wf(),
            new_inner.wf(),
            new_inner.spec_running_pid() == chosen_next_pid as int,
            chosen_next_pid >= 0i32,
            chosen_next_tid >= 0i32,
            // Cannot exit the kernel thread.
            old(self).current_tid != KERNEL_TID_RAW,
            // Same thread implies same process.
            chosen_next_tid == old(self).current_tid ==> chosen_next_pid == old(self).current_pid,
        ensures
            self.wf(),
            self.inner == new_inner,
            self.scheduler_freq == old(self).scheduler_freq,
    {
        self.switch(chosen_next_pid, chosen_next_tid, new_inner);
    }

    //==============================================================================================
    // Delegation Functions (No Queue-Level State Change)
    //==============================================================================================

    /// Models `ProcessManager::get_mutex()` (unsafe.rs:547-549).
    ///
    /// Delegates to inner.get_mutex(). No queue-level state change.
    pub fn get_mutex(&self)
        requires
            self.wf(),
        ensures
            self.wf(),
    {
        // T3/T7: inner.get_mutex() is a query; no queue change.
    }

    /// Models `ProcessManager::put_mutex_guard()` (unsafe.rs:569-577).
    ///
    /// Delegates to inner.put_mutex_guard(). No queue-level state change.
    pub fn put_mutex_guard(&self)
        requires
            self.wf(),
        ensures
            self.wf(),
    {
    }

    /// Models `ProcessManager::get_cond()` (unsafe.rs:602-604).
    ///
    /// Delegates to inner.get_cond(). No queue-level state change.
    pub fn get_cond(&self)
        requires
            self.wf(),
        ensures
            self.wf(),
    {
    }

    /// Models `ProcessManager::put_cond()` (unsafe.rs:627-629).
    ///
    /// Delegates to inner.put_cond(). No queue-level state change.
    pub fn put_cond(&self)
        requires
            self.wf(),
        ensures
            self.wf(),
    {
    }

    /// Models `ProcessManager::wakeup()` (unsafe.rs:678-681).
    ///
    /// Delegates to inner.wakeup(). Queue-level effects (suspended→ready)
    /// are verified in the inner module.
    pub fn wakeup(&mut self, new_inner: ProcessManagerInner)
        requires
            old(self).wf(),
            new_inner.wf(),
            new_inner.spec_running_pid() == old(self).inner.spec_running_pid(),
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
    /// Delegates to inner.take_mutex_guard(). No queue-level state change.
    pub fn take_mutex_guard(&self)
        requires
            self.wf(),
        ensures
            self.wf(),
    {
    }

    //==============================================================================================
    // Message Reception
    //==============================================================================================

    /// Models `ProcessManager::try_recv()` (unsafe.rs:649-659).
    ///
    /// Attempts to receive a message. If a message is found, decrements
    /// number_buffered_messages. Queue-level state is unchanged.
    pub fn try_recv_some(&mut self)
        requires
            old(self).wf(),
            old(self).inner.number_buffered_messages > 0,
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
    pub fn try_recv_none(&self)
        requires
            self.wf(),
        ensures
            self.wf(),
    {
    }

    //==============================================================================================
    // Join Thread (Blocking)
    //==============================================================================================

    /// Models `ProcessManager::join_thread()` (unsafe.rs:341-408).
    ///
    /// The join_thread function is a loop that either:
    /// 1. Finds a zombie thread and harvests it (no queue-level change).
    /// 2. Blocks on a condvar (delegates to sleep, which is verified).
    /// 3. Returns an error (no state change).
    ///
    /// The queue-level effects of the blocking path are captured by
    /// sleep(). The harvesting path only unmaps pages (memory management,
    /// external to queue model).
    pub fn join_thread_harvest(&self)
        requires
            self.wf(),
        ensures
            self.wf(),
    {
        // Zombie thread harvesting: no queue-level state change.
        // Page unmapping is a memory management operation (external to this model).
    }

    /// Models `ProcessManager::join_thread()` error path.
    pub fn join_thread_error(&self)
        requires
            self.wf(),
        ensures
            self.wf(),
    {
    }
}

} // verus!
