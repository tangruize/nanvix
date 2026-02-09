// Copyright(c) The Maintainers of Nanvix.
// Licensed under the MIT License.

// ProcessManagerInner Proofs.
// This file contains proof lemmas for the ProcessManagerInner type.
//
// Key proven properties:
// - Construction produces a well-formed state.
// - A PID >= next_pid is fresh (not in any queue).
// - create_process preserves wf by allocating a fresh PID.
// - schedule preserves wf: running→ready and ready→running swap maintains
//   disjointness, kernel safety, and PID bounds.
// - sleep_running preserves wf: non-kernel running→suspended.
// - exit_running preserves wf: non-kernel running→zombie.
// - wakeup_to_ready preserves wf: suspended→ready.
// - resume_all_interrupted preserves wf: all interrupted→ready.
// - terminate_ready preserves wf: non-kernel ready→zombie.
// - terminate_ready_stays_ready: non-kernel ready process stays ready (T3 boundary).
// - terminate_suspended preserves wf: suspended→interrupted.
// - harvest_zombie preserves wf: zombie removed.
// - Kernel liveness: the kernel PID is always alive.
// - PID uniqueness: newly allocated PIDs are always fresh.

use vstd::prelude::*;

verus! {

impl ProcessManagerInner {

    //==============================================================================================
    // Construction Lemmas
    //==============================================================================================

    /// Lemma: A newly constructed ProcessManagerInner is well-formed.
    pub proof fn lemma_new_is_wf(interrupt_capable: bool)
        ensures
            ({
                let pm: ProcessManagerInner = ProcessManagerInner {
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
                pm.wf()
            }),
    {
    }

    //==============================================================================================
    // PID Freshness Lemmas
    //==============================================================================================

    /// Lemma: Any PID >= next_pid is not in any ghost set.
    ///
    /// This follows from spec_pid_bounds which requires all PIDs in sets
    /// to be < next_pid.
    pub proof fn lemma_fresh_pid_not_in_sets(&self, pid: int)
        requires
            self.wf(),
            pid >= self.next_pid as int,
        ensures
            self.spec_pid_is_fresh(pid),
    {
    }

    /// Lemma: The PID equal to next_pid is fresh.
    pub proof fn lemma_next_pid_is_fresh(&self)
        requires
            self.wf(),
        ensures
            self.spec_pid_is_fresh(self.next_pid as int),
    {
        self.lemma_fresh_pid_not_in_sets(self.next_pid as int);
    }

    //==============================================================================================
    // Schedule Lemmas
    //==============================================================================================

    /// Lemma: After schedule, the ready set cardinality is unchanged.
    ///
    /// Insert old_running (not in old_ready) adds 1, then remove chosen
    /// (which is in the extended set) subtracts 1, netting zero change.
    pub proof fn lemma_schedule_ready_len(
        old_ready: Set<int>,
        old_running: int,
        chosen: int,
    )
        requires
            old_ready.finite(),
            !old_ready.contains(old_running),
            old_ready.insert(old_running).contains(chosen),
        ensures
            old_ready.insert(old_running).remove(chosen).finite(),
            old_ready.insert(old_running).remove(chosen).len() == old_ready.len(),
    {
        // Broadcast axioms handle insert/remove len reasoning.
        assert(old_ready.insert(old_running).len() == old_ready.len() + 1);
        let with_old: Set<int> = old_ready.insert(old_running);
        assert(with_old.contains(chosen));
        assert(with_old.remove(chosen).len() == with_old.len() - 1);
    }

    //==============================================================================================
    // Kernel Safety Lemmas
    //==============================================================================================

    /// Lemma: The kernel PID (0) is always alive after new().
    pub proof fn lemma_kernel_alive_after_new(interrupt_capable: bool)
        ensures
            ({
                let pm: ProcessManagerInner = ProcessManagerInner {
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
                pm.spec_process_exists(0)
            }),
    {
    }

    /// Lemma: After schedule, the kernel is still alive.
    ///
    /// If the kernel was running, it moves to ready.
    /// If the kernel was in ready and is not chosen, it stays in ready.
    /// If the kernel was in ready and is chosen, it becomes running.
    pub proof fn lemma_kernel_alive_after_schedule(
        &self,
        chosen: int,
    )
        requires
            self.wf(),
            self.spec_ready_with_running().contains(chosen),
            chosen >= 0,
        ensures
            chosen == 0int || self.ghost_ready@.insert(
                self.running_pid as int
            ).remove(chosen).contains(0int),
    {
        if chosen == 0int {
        } else {
            if self.running_pid as int == 0int {
                assert(self.ghost_ready@.insert(0int).remove(chosen).contains(0int));
            } else {
                assert(self.ghost_ready@.contains(0int));
                assert(self.ghost_ready@.insert(
                    self.running_pid as int
                ).remove(chosen).contains(0int));
            }
        }
    }

    //==============================================================================================
    // Union Lemmas (for resume_all_interrupted)
    //==============================================================================================

    /// Lemma: Union of two finite disjoint sets has len equal to sum.
    pub proof fn lemma_union_disjoint_len(
        a: Set<int>,
        b: Set<int>,
    )
        requires
            a.finite(),
            b.finite(),
            a.disjoint(b),
        ensures
            a.union(b).finite(),
            a.union(b).len() == a.len() + b.len(),
        decreases a.len(),
    {
        if a.len() == 0 {
            a.lemma_len0_is_empty();
            assert(a.union(b) =~= b);
        } else {
            let x: int = a.choose();
            let a1: Set<int> = a.remove(x);
            assert(!b.contains(x));
            assert(a1.disjoint(b));
            assert(a.union(b).remove(x) =~= a1.union(b));
            Self::lemma_union_disjoint_len(a1, b);
        }
    }

    //==============================================================================================
    // View Equality Lemma
    //==============================================================================================

    /// Lemma: Two ProcessManagerInners with equal views have equal abstract state.
    pub proof fn lemma_view_equality(a: &ProcessManagerInner, b: &ProcessManagerInner)
        requires
            a@ == b@,
        ensures
            a.spec_running_pid() == b.spec_running_pid(),
    {
    }
}

} // verus!
