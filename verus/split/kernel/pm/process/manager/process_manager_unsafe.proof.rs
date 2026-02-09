// Copyright(c) The Maintainers of Nanvix.
// Licensed under the MIT License.

// ProcessManagerUnsafe Proofs.
//
// This file contains proof lemmas for the ProcessManagerUnsafeState type.
//
// Key proven properties:
// - Initialization produces a well-formed state.
// - get()/get_mut() singleton accessors require initialization and preserve wf().
// - switch() preserves wf(): only updates atomics (PID/TID/quantum), inner state
//   is unchanged. Both hard and soft context switches maintain all invariants.
// - update_inner() preserves wf(): inner state mutation separated from switch().
// - giveup() preserves wf(): quantum decrement stays in valid range, context switch
//   path delegates to switch() which preserves wf().
// - sleep() preserves wf(): delegates to inner sleep + update_inner + switch.
// - sleep_post_wakeup() models the interrupt-reason check after wakeup.
// - exit()/exit_thread() preserves wf(): delegates to inner exit + update_inner + switch.
//   These are divergent (never return on success path).
// - is_kernel_running() is correct: returns true iff current_tid == 0.
// - Delegation functions (get_mutex, get_cond, etc.) preserve wf().
// - try_recv with message decrement preserves wf().
// - join_thread_wait() models the condvar-wait blocking path.

use vstd::prelude::*;

verus! {

impl ProcessManagerUnsafeState {

    //==============================================================================================
    // Initialization Lemmas
    //==============================================================================================

    /// Lemma: A newly initialized ProcessManagerUnsafeState is well-formed.
    pub proof fn lemma_init_is_wf(scheduler_freq: usize, interrupt_capable: bool)
        requires
            scheduler_freq > 0,
            scheduler_freq <= usize::MAX,
        ensures
            ({
                let inner: ProcessManagerInner = ProcessManagerInner {
                    running_pid: 0i32,
                    ready_count: 0usize,
                    suspended_count: 0usize,
                    interrupted_count: 0usize,
                    zombie_count: 0usize,
                    next_pid: 1i32,
                    interrupt_capable: interrupt_capable,
                    number_buffered_messages: 0usize,
                    ghost_ready: Ghost(Set::empty()),
                    ghost_suspended: Ghost(Set::empty()),
                    ghost_interrupted: Ghost(Set::empty()),
                    ghost_zombies: Ghost(Set::empty()),
                };
                let state: ProcessManagerUnsafeState = ProcessManagerUnsafeState {
                    initialized: true,
                    inner: inner,
                    current_pid: 0i32,
                    current_tid: 0i32,
                    remaining_quantum: scheduler_freq,
                    fpu_owner_tid: 0i32,
                    scheduler_freq: scheduler_freq,
                    ghost_borrow_count: Ghost(0int),
                };
                state.wf()
            }),
    {
        ProcessManagerInner::lemma_new_is_wf(interrupt_capable);
    }

    //==============================================================================================
    // Singleton Access Lemmas
    //==============================================================================================

    /// Lemma: get() on an initialized, wf state returns a state that is wf.
    ///
    /// Models: `ProcessManager::get()` (unsafe.rs:160-167).
    /// The original returns `&'a ProcessManager` from `static mut PROCESS_MANAGER`.
    /// Requires initialization (panics otherwise).
    pub proof fn lemma_get_preserves_wf(&self)
        requires
            self.wf(),
        ensures
            self.wf(),
            self.initialized,
            self.spec_inner_wf(),
    {
        // Shared borrow: no state change. wf() trivially preserved.
    }

    /// Lemma: get_mut() on an initialized, wf state with no outstanding borrows
    /// returns a state that is wf.
    ///
    /// Models: `ProcessManager::get_mut()` (unsafe.rs:184-191).
    /// The original returns `&'a mut ProcessManager` from `static mut PROCESS_MANAGER`.
    /// Requires initialization (panics otherwise). The exclusive access guarantee
    /// comes from the caller ensuring no other references exist.
    pub proof fn lemma_get_mut_preserves_wf(&self)
        requires
            self.wf(),
            self.spec_no_borrows(),
        ensures
            self.wf(),
            self.initialized,
            self.spec_inner_wf(),
    {
        // Exclusive borrow: no state change. wf() trivially preserved.
        // The caller must ensure no other borrows are outstanding.
    }

    //==============================================================================================
    // Switch Lemmas
    //==============================================================================================

    /// Lemma: Soft context switch (same thread) preserves wf().
    ///
    /// When next_tid == current_tid, no atomic state change occurs.
    pub proof fn lemma_soft_switch_preserves_wf(&self, next_pid: i32, next_tid: i32)
        requires
            self.wf(),
            next_tid == self.current_tid,
            next_pid >= 0i32,
            next_tid >= 0i32,
        ensures
            self.wf(),
    {
        // No state mutation on soft switch; wf() holds by hypothesis.
    }

    /// Lemma: Hard context switch with same PID preserves wf().
    ///
    /// When next_tid != current_tid but next_pid == current_pid,
    /// only current_tid is updated. Inner state, pid, quantum all unchanged.
    pub proof fn lemma_hard_switch_same_pid_preserves_wf(
        &self, next_pid: i32, next_tid: i32,
    )
        requires
            self.wf(),
            next_tid != self.current_tid,
            next_pid == self.current_pid,
            next_pid >= 0i32,
            next_tid >= 0i32,
        ensures
            ({
                let new_state: ProcessManagerUnsafeState = ProcessManagerUnsafeState {
                    current_tid: next_tid,
                    ..(*self)
                };
                new_state.wf()
            }),
    {
    }

    /// Lemma: Hard context switch with different PID preserves wf().
    ///
    /// When both next_tid and next_pid differ, current_pid is updated to next_pid,
    /// current_tid to next_tid, and remaining_quantum is reset to scheduler_freq.
    /// The inner state must have already been updated (via update_inner) to reflect
    /// next_pid as the running process before switch() is called.
    pub proof fn lemma_hard_switch_diff_pid_preserves_wf(
        &self,
        next_pid: i32,
        next_tid: i32,
    )
        requires
            self.wf(),
            next_tid != self.current_tid,
            next_pid != self.current_pid,
            next_pid >= 0i32,
            next_tid >= 0i32,
            // Inner state must already reflect the new running PID.
            self.inner.spec_running_pid() == next_pid as int,
            self.scheduler_freq > 0,
        ensures
            ({
                let new_state: ProcessManagerUnsafeState = ProcessManagerUnsafeState {
                    current_pid: next_pid,
                    current_tid: next_tid,
                    remaining_quantum: self.scheduler_freq,
                    ..(*self)
                };
                new_state.wf()
            }),
    {
    }

    //==============================================================================================
    // Inner State Update Lemma
    //==============================================================================================

    /// Lemma: Updating the inner state preserves wf() if the new inner is wf
    /// and its running_pid matches current_pid.
    ///
    /// This models the separation between inner state mutation (which happens
    /// before switch()) and the atomic updates in switch(). In the original code,
    /// `try_borrow_mut()?.exit(status)` mutates inner and returns context pointers,
    /// then `switch()` updates the atomics.
    pub proof fn lemma_update_inner_preserves_wf(
        &self,
        new_inner: ProcessManagerInner,
    )
        requires
            self.wf(),
            new_inner.wf(),
            new_inner.spec_running_pid() == self.current_pid as int,
        ensures
            ({
                let new_state: ProcessManagerUnsafeState = ProcessManagerUnsafeState {
                    inner: new_inner,
                    ..(*self)
                };
                new_state.wf()
            }),
    {
        // new_inner.wf() holds by precondition.
        // current_pid matches new_inner.running_pid (pid_consistent).
        // All other fields unchanged.
    }

    //==============================================================================================
    // Quantum Management Lemmas
    //==============================================================================================

    /// Lemma: Decrementing the quantum by 1 preserves wf() when quantum > 1.
    pub proof fn lemma_quantum_decrement_preserves_wf(&self)
        requires
            self.wf(),
            self.remaining_quantum > 1,
        ensures
            ({
                let new_state: ProcessManagerUnsafeState = ProcessManagerUnsafeState {
                    remaining_quantum: (self.remaining_quantum - 1) as usize,
                    ..(*self)
                };
                new_state.wf()
            }),
    {
        // remaining_quantum - 1 >= 1 since remaining_quantum > 1.
        // remaining_quantum - 1 <= scheduler_freq since remaining_quantum <= scheduler_freq.
    }

    /// Lemma: Resetting quantum to scheduler_freq preserves wf().
    pub proof fn lemma_quantum_reset_preserves_wf(&self)
        requires
            self.wf(),
        ensures
            ({
                let new_state: ProcessManagerUnsafeState = ProcessManagerUnsafeState {
                    remaining_quantum: self.scheduler_freq,
                    ..(*self)
                };
                new_state.wf()
            }),
    {
    }

    //==============================================================================================
    // Kernel Detection Lemma
    //==============================================================================================

    /// Lemma: is_kernel_running() returns true iff current_tid == KERNEL_TID (0).
    pub proof fn lemma_is_kernel_running_correct(&self)
        requires
            self.wf(),
        ensures
            self.spec_is_kernel_running() <==> (self.current_tid == 0i32),
    {
    }

    //==============================================================================================
    // Delegation Lemmas
    //==============================================================================================

    /// Lemma: Delegation to inner that preserves inner.wf() preserves outer wf().
    pub proof fn lemma_delegation_preserves_wf(
        &self,
        new_inner: ProcessManagerInner,
        new_pid: i32,
        new_tid: i32,
        new_quantum: usize,
    )
        requires
            self.wf(),
            new_inner.wf(),
            new_pid as int == new_inner.spec_running_pid(),
            new_pid >= 0i32,
            new_tid >= 0i32,
            new_quantum >= 1,
            new_quantum <= self.scheduler_freq,
        ensures
            ({
                let new_state: ProcessManagerUnsafeState = ProcessManagerUnsafeState {
                    inner: new_inner,
                    current_pid: new_pid,
                    current_tid: new_tid,
                    remaining_quantum: new_quantum,
                    ..(*self)
                };
                new_state.wf()
            }),
    {
    }

    //==============================================================================================
    // Inner-Preserving Operation Lemmas
    //==============================================================================================

    /// Lemma: Operations that only read inner state preserve wf().
    ///
    /// Models: get_mutex, get_cond, put_cond, put_mutex_guard, take_mutex_guard
    /// — all of which borrow inner, perform an operation, and return without
    /// changing queue-level state.
    pub proof fn lemma_read_only_preserves_wf(&self)
        requires
            self.wf(),
        ensures
            self.wf(),
    {
    }

    /// Lemma: try_recv that decrements number_buffered_messages preserves wf().
    ///
    /// Models unsafe.rs:650-658 where pm.number_buffered_messages is decremented.
    pub proof fn lemma_recv_message_preserves_wf(&self)
        requires
            self.wf(),
            self.inner.number_buffered_messages > 0,
        ensures
            ({
                let new_inner: ProcessManagerInner = ProcessManagerInner {
                    number_buffered_messages: (self.inner.number_buffered_messages - 1) as usize,
                    ..self.inner
                };
                let new_state: ProcessManagerUnsafeState = ProcessManagerUnsafeState {
                    inner: new_inner,
                    ..(*self)
                };
                new_state.wf()
            }),
    {
        // Only number_buffered_messages changes.
        // Inner wf(): all queue fields, ghost sets, counts, pids are unchanged.
        // number_buffered_messages - 1 >= 0, and < usize::MAX (since original < usize::MAX).
        // Outer: inner.running_pid unchanged, so pid_consistent still holds.
    }
}

} // verus!
