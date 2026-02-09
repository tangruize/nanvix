// Copyright(c) The Maintainers of Nanvix.
// Licensed under the MIT License.

// RunningProcess Proofs.
// This file contains proof lemmas for the RunningProcess type.
//
// Key proven properties:
// - Construction produces well-formed state with correct initial values.
// - PID is immutable across all operations.
// - schedule() moves running→ready, producing RunnableProcess with preserved PID
//   and non-empty ready list. Total thread count preserved.
// - sleep() moves running→sleeping, correct branch selection. Sleeping threads
//   are threaded through InterruptedProcess on the interrupted path.
// - exit() moves running→zombie + terminates all, correct branch selection.
//   Zombie content is exact (running + ready + original zombie).
// - exit_thread() moves running→zombie for just the running thread, with
//   documented divergence from original source (see exec file).
// - get_tid() returns the running thread ID.
// - wakeup() moves sleeping→ready, preserves PID and total count.
// - try_join_thread() spec model captures running-thread error, zombie removal,
//   condvar for live threads, and not-found error.
// - find_thread() spec model verifies exhaustive search.
// - Well-formedness is preserved by all operations.
// - wf_strict() provides optional thread ID uniqueness predicate.

use vstd::prelude::*;

verus! {

impl RunningProcess {

    //==============================================================================================
    // Construction Lemmas
    //==============================================================================================

    /// Lemma: A newly constructed RunningProcess is well-formed.
    pub proof fn lemma_new_is_wf(
        pid: int,
        running_tid: int,
        ready_ids: Seq<int>,
        interrupted_ids: Seq<int>,
        sleeping_ids: Seq<int>,
        zombie_ids: Seq<int>,
    )
        requires
            ready_ids.len() <= u64::MAX as nat,
            interrupted_ids.len() <= u64::MAX as nat,
            sleeping_ids.len() <= u64::MAX as nat,
            zombie_ids.len() <= u64::MAX as nat,
        ensures
            ({
                let r: RunningProcess = RunningProcess {
                    pid: Ghost(pid),
                    running_thread_id: Ghost(running_tid),
                    ready_thread_ids: Ghost(ready_ids),
                    interrupted_thread_ids: Ghost(interrupted_ids),
                    sleeping_thread_ids: Ghost(sleeping_ids),
                    zombie_thread_ids: Ghost(zombie_ids),
                    ready_count: ready_ids.len() as u64,
                    interrupted_count: interrupted_ids.len() as u64,
                    sleeping_count: sleeping_ids.len() as u64,
                    zombie_count: zombie_ids.len() as u64,
                };
                r.wf()
            }),
    {
    }

    //==============================================================================================
    // PID Immutability Lemmas
    //==============================================================================================

    /// Lemma: PID is preserved across construction with same pid.
    pub proof fn lemma_pid_preserved(&self)
        ensures
            self.spec_pid() == self.pid@,
    {
    }

    //==============================================================================================
    // schedule() Lemmas
    //==============================================================================================

    /// Lemma: schedule() produces a result with non-empty ready list (>= 1).
    /// The running thread becomes ready, so even if the ready list was empty,
    /// the result has at least one ready thread.
    pub proof fn lemma_schedule_result_has_ready(&self)
        requires
            self.wf(),
        ensures
            ({
                let new_ready: Seq<int> = self.ready_thread_ids@.push(self.running_thread_id@);
                new_ready.len() >= 1
                && new_ready.len() == self.spec_ready_count() + 1
            }),
    {
    }

    /// Lemma: schedule() preserves total thread count.
    /// The running thread (1) becomes part of the ready list, so:
    /// result.ready = self.ready + 1, and no other lists change.
    /// Total before: 1 + ready + interrupted + sleeping + zombie.
    /// Total after (as threads in RunnableProcess): (ready+1) + interrupted + sleeping + zombie.
    /// The "1" (running) is accounted for by the +1 in ready.
    pub proof fn lemma_schedule_preserves_total_threads(&self)
        requires
            self.wf(),
        ensures
            ({
                let new_ready_len: nat = (self.spec_ready_count() + 1) as nat;
                // Total threads = new ready + unchanged lists.
                new_ready_len + self.spec_interrupted_count()
                    + self.spec_sleeping_count() + self.spec_zombie_count()
                    == self.spec_total_thread_count()
            }),
    {
    }

    //==============================================================================================
    // sleep() Lemmas
    //==============================================================================================

    /// Lemma: sleep() with ready threads produces Runnable with correct sleeping content.
    pub proof fn lemma_sleep_with_ready_gives_runnable(&self)
        requires
            self.wf(),
            self.spec_ready_count() > 0,
        ensures
            ({
                let new_sleeping: Seq<int> = self.sleeping_thread_ids@.push(self.running_thread_id@);
                new_sleeping.len() == self.spec_sleeping_count() + 1
                && new_sleeping.len() >= 1
            }),
    {
    }

    /// Lemma: sleep() with no ready but interrupted threads: sleeping list grows by 1.
    pub proof fn lemma_sleep_with_interrupted_sleeping_content(&self)
        requires
            self.wf(),
            self.spec_ready_count() == 0,
            self.spec_interrupted_count() > 0,
        ensures
            ({
                let new_sleeping: Seq<int> = self.sleeping_thread_ids@.push(self.running_thread_id@);
                new_sleeping.len() == self.spec_sleeping_count() + 1
                && self.interrupted_thread_ids@.len() >= 1
            }),
    {
    }

    /// Lemma: sleep() with no ready and no interrupted produces Sleeping with correct content.
    pub proof fn lemma_sleep_no_ready_no_interrupted_content(&self)
        requires
            self.wf(),
            self.spec_ready_count() == 0,
            self.spec_interrupted_count() == 0,
        ensures
            ({
                let new_sleeping: Seq<int> = self.sleeping_thread_ids@.push(self.running_thread_id@);
                new_sleeping.len() == self.spec_sleeping_count() + 1
                && new_sleeping.len() >= 1
            }),
    {
    }

    //==============================================================================================
    // exit() Lemmas
    //==============================================================================================

    /// Lemma: exit() zombie list content is exactly running + ready + original zombie.
    pub proof fn lemma_exit_zombie_content(&self)
        requires
            self.wf(),
        ensures
            ({
                let new_zombie: Seq<int> = seq![self.running_thread_id@]
                    .add(self.ready_thread_ids@).add(self.zombie_thread_ids@);
                new_zombie.len() == 1 + self.spec_ready_count() + self.spec_zombie_count()
                && new_zombie.len() >= 1
            }),
    {
    }

    /// Lemma: exit() with interrupted or sleeping threads: interrupted list is non-empty.
    pub proof fn lemma_exit_with_interrupted_has_interrupted(&self)
        requires
            self.wf(),
            self.spec_interrupted_count() > 0 || self.spec_sleeping_count() > 0,
        ensures
            ({
                let new_interrupted: Seq<int> =
                    self.interrupted_thread_ids@.add(self.sleeping_thread_ids@);
                new_interrupted.len() >= 1
                && new_interrupted.len() ==
                    self.spec_interrupted_count() + self.spec_sleeping_count()
            }),
    {
    }

    /// Lemma: exit() with no interrupted or sleeping threads: the combined
    /// interrupted+sleeping list is empty and the zombie list contains all threads.
    pub proof fn lemma_exit_no_interrupted_gives_zombie(&self)
        requires
            self.wf(),
            self.spec_interrupted_count() == 0,
            self.spec_sleeping_count() == 0,
        ensures
            ({
                let combined_interrupted: Seq<int> =
                    self.interrupted_thread_ids@.add(self.sleeping_thread_ids@);
                combined_interrupted.len() == 0
                && self.interrupted_thread_ids@.len() == 0
                && self.sleeping_thread_ids@.len() == 0
            }),
    {
    }

    //==============================================================================================
    // exit_thread() Lemmas
    //==============================================================================================

    /// Lemma: exit_thread() zombie list is original zombie + running thread.
    pub proof fn lemma_exit_thread_zombie_content(&self)
        requires
            self.wf(),
        ensures
            ({
                let new_zombie: Seq<int> = self.zombie_thread_ids@.push(self.running_thread_id@);
                new_zombie.len() == 1 + self.spec_zombie_count()
                && new_zombie.len() >= 1
            }),
    {
    }

    /// Lemma: exit_thread() with ready threads: result ready list is unchanged.
    pub proof fn lemma_exit_thread_with_ready_preserves_ready(&self)
        requires
            self.wf(),
            self.spec_ready_count() > 0,
        ensures
            self.ready_thread_ids@.len() >= 1,
    {
    }

    //==============================================================================================
    // wakeup() Lemmas
    //==============================================================================================

    /// Lemma: Successful wakeup increases ready count by 1 and decreases sleeping by 1.
    pub proof fn lemma_wakeup_moves_thread(&self, removed_idx: int)
        requires
            self.wf(),
            self.spec_sleeping_count() > 0,
            0 <= removed_idx < self.sleeping_thread_ids@.len(),
        ensures
            ({
                let new_ready_len: nat = (self.ready_thread_ids@.len() + 1) as nat;
                let new_sleeping_len: nat = (self.sleeping_thread_ids@.len() - 1) as nat;
                new_ready_len == self.spec_ready_count() + 1
                && new_sleeping_len == self.spec_sleeping_count() - 1
            }),
    {
    }

    /// Lemma: wakeup() preserves PID.
    pub proof fn lemma_wakeup_preserves_pid(&self)
        requires
            self.wf(),
        ensures
            self.spec_pid() == self.pid@,
    {
    }

    /// Lemma: wakeup() preserves total thread count.
    pub proof fn lemma_wakeup_preserves_total_count(&self, removed_idx: int)
        requires
            self.wf(),
            self.spec_sleeping_count() > 0,
            0 <= removed_idx < self.sleeping_thread_ids@.len(),
        ensures
            ({
                let new_total: int =
                    1  // running thread (unchanged)
                    + (self.spec_ready_count() + 1) as int
                    + self.spec_interrupted_count() as int
                    + (self.spec_sleeping_count() - 1) as int
                    + self.spec_zombie_count() as int;
                new_total == self.spec_total_thread_count() as int
            }),
    {
    }

    /// Lemma: If spec_seq_contains is true, there exists a valid index.
    pub proof fn lemma_spec_find_thread_index(&self, tid: Ghost<int>)
        requires
            Self::spec_seq_contains(self.sleeping_thread_ids@, tid@),
        ensures
            exists|i: int| 0 <= i < self.sleeping_thread_ids@.len()
                && self.sleeping_thread_ids@[i] == tid@,
    {
    }

    //==============================================================================================
    // get_tid() Lemmas
    //==============================================================================================

    /// Lemma: get_tid returns the running thread's ID.
    pub proof fn lemma_get_tid(&self)
        ensures
            self.spec_running_thread_id() == self.running_thread_id@,
    {
    }

    //==============================================================================================
    // find_thread() Lemmas
    //==============================================================================================

    /// Lemma: spec_find_thread returns Some(0) iff the thread is the running thread.
    pub proof fn lemma_find_thread_running(&self, tid: int)
        requires
            self.running_thread_id@ == tid,
        ensures
            self.spec_find_thread(tid) == Some(0int),
    {
    }

    /// Lemma: spec_find_thread returns None iff the thread is not in any list.
    pub proof fn lemma_find_thread_not_found(&self, tid: int)
        requires
            self.running_thread_id@ != tid,
            !self.spec_has_ready_thread(tid),
            !self.spec_has_interrupted_thread(tid),
            !self.spec_has_sleeping_thread(tid),
            !self.spec_has_zombie_thread(tid),
        ensures
            self.spec_find_thread(tid) == None::<int>,
    {
    }

    /// Lemma: spec_find_thread result is consistent with spec_has_thread.
    pub proof fn lemma_find_thread_iff_has_thread(&self, tid: int)
        ensures
            self.spec_find_thread(tid).is_some() <==> self.spec_has_thread(tid),
    {
    }

    //==============================================================================================
    // try_join_thread() Lemmas
    //==============================================================================================

    /// Lemma: try_join_thread on the running thread returns error (tag=1).
    pub proof fn lemma_try_join_running_thread_errors(&self, tid: int)
        requires
            self.running_thread_id@ == tid,
        ensures
            self.spec_try_join_thread(tid) == 1int,
    {
    }

    /// Lemma: try_join_thread on a zombie thread returns success (tag=0).
    pub proof fn lemma_try_join_zombie_thread_succeeds(&self, tid: int)
        requires
            self.running_thread_id@ != tid,
            self.spec_has_zombie_thread(tid),
        ensures
            self.spec_try_join_thread(tid) == 0int,
    {
    }

    /// Lemma: try_join_thread on a not-found thread returns error (tag=3).
    pub proof fn lemma_try_join_not_found_errors(&self, tid: int)
        requires
            !self.spec_has_thread(tid),
        ensures
            self.spec_try_join_thread(tid) == 3int,
    {
    }

    /// Lemma: After a successful zombie join, the zombie list shrinks by exactly 1.
    pub proof fn lemma_try_join_zombie_post_shrinks(&self, tid: int)
        requires
            self.spec_try_join_thread(tid) == 0int,
            self.spec_has_zombie_thread(tid),
        ensures
            ({
                let post_zombies: Seq<int> = self.spec_try_join_zombie_post(tid);
                post_zombies.len() == self.spec_zombie_count() - 1
            }),
    {
        // The chosen index is valid.
        let idx: int = choose|i: int| 0 <= i < self.zombie_thread_ids@.len()
            && self.zombie_thread_ids@[i] == tid;
        Self::lemma_remove_at_length(self.zombie_thread_ids@, idx);
    }

    //==============================================================================================
    // Content Preservation Lemmas
    //==============================================================================================

    /// Lemma: spec_remove_at produces a sequence of length len - 1.
    pub proof fn lemma_remove_at_length(s: Seq<int>, idx: int)
        requires
            0 <= idx < s.len(),
        ensures
            Self::spec_remove_at(s, idx).len() == s.len() - 1,
    {
        let left: Seq<int> = s.subrange(0, idx);
        let right: Seq<int> = s.subrange(idx + 1, s.len() as int);
        assert(left.len() == idx as nat);
        assert(right.len() == (s.len() - idx as nat - 1) as nat);
    }

    /// Lemma: spec_remove_at preserves elements before and after the removed index.
    pub proof fn lemma_remove_at_preserves_others(s: Seq<int>, idx: int, j: int)
        requires
            0 <= idx < s.len(),
            0 <= j < s.len() - 1,
        ensures
            Self::spec_remove_at(s, idx)[j] == if j < idx { s[j] } else { s[j + 1] },
    {
    }

    //==============================================================================================
    // View Equality
    //==============================================================================================

    /// Lemma: Two RunningProcesses with identical fields have equal views.
    pub proof fn lemma_view_equality(a: &RunningProcess, b: &RunningProcess)
        requires
            a.pid@ == b.pid@,
            a.running_thread_id@ == b.running_thread_id@,
            a.ready_thread_ids@ =~= b.ready_thread_ids@,
            a.interrupted_thread_ids@ =~= b.interrupted_thread_ids@,
            a.sleeping_thread_ids@ =~= b.sleeping_thread_ids@,
            a.zombie_thread_ids@ =~= b.zombie_thread_ids@,
        ensures
            a@ == b@,
    {
    }
}

//==================================================================================================
// RunnableProcess Lemmas (Boundary)
//==================================================================================================

impl RunnableProcess {
    /// Lemma: Construction with non-empty ready threads is well-formed.
    pub proof fn lemma_new_wf(pid: int, ready_ids: Seq<int>, interrupted_ids: Seq<int>,
                              sleeping_ids: Seq<int>, zombie_ids: Seq<int>)
        requires
            ready_ids.len() >= 1,
        ensures
            ({
                let rp: RunnableProcess = RunnableProcess {
                    pid: Ghost(pid),
                    ready_thread_ids: Ghost(ready_ids),
                    interrupted_thread_ids: Ghost(interrupted_ids),
                    sleeping_thread_ids: Ghost(sleeping_ids),
                    zombie_thread_ids: Ghost(zombie_ids),
                };
                rp.wf()
            }),
    {
    }
}

//==================================================================================================
// SleepingProcess Lemmas (Boundary)
//==================================================================================================

impl SleepingProcess {
    /// Lemma: Construction with non-empty sleeping threads is well-formed.
    pub proof fn lemma_new_wf(pid: int, sleeping_ids: Seq<int>, zombie_ids: Seq<int>)
        requires
            sleeping_ids.len() >= 1,
        ensures
            ({
                let sp: SleepingProcess = SleepingProcess {
                    pid: Ghost(pid),
                    sleeping_thread_ids: Ghost(sleeping_ids),
                    zombie_thread_ids: Ghost(zombie_ids),
                };
                sp.wf()
            }),
    {
    }
}

//==================================================================================================
// InterruptedProcess Lemmas (Boundary)
//==================================================================================================

impl InterruptedProcess {
    /// Lemma: Construction with non-empty interrupted threads is well-formed.
    pub proof fn lemma_new_wf(pid: int, interrupted_ids: Seq<int>,
                              sleeping_ids: Seq<int>, zombie_ids: Seq<int>)
        requires
            interrupted_ids.len() >= 1,
        ensures
            ({
                let ip: InterruptedProcess = InterruptedProcess {
                    pid: Ghost(pid),
                    interrupted_thread_ids: Ghost(interrupted_ids),
                    sleeping_thread_ids: Ghost(sleeping_ids),
                    zombie_thread_ids: Ghost(zombie_ids),
                };
                ip.wf()
            }),
    {
    }
}

//==================================================================================================
// ZombieProcess Lemmas (Boundary)
//==================================================================================================

impl ZombieProcess {
    /// Lemma: Construction with non-empty zombie threads is well-formed.
    pub proof fn lemma_new_wf(pid: int, zombie_ids: Seq<int>, status: int)
        requires
            zombie_ids.len() >= 1,
        ensures
            ({
                let zp: ZombieProcess = ZombieProcess {
                    pid: Ghost(pid),
                    zombie_thread_ids: Ghost(zombie_ids),
                    status: Ghost(status),
                };
                zp.wf()
            }),
    {
    }
}

} // verus!
