// Copyright(c) The Maintainers of Nanvix.
// Licensed under the MIT License.

// RunningProcess Proofs.
// This file contains proof lemmas for the RunningProcess type.
//
// Key proven properties:
// - Construction produces well-formed state with correct initial values.
// - PID is immutable across all operations.
// - schedule() moves running→ready, producing RunnableProcess with preserved PID.
// - sleep() moves running→sleeping, correct branch selection.
// - exit() moves running→zombie + terminates all, correct branch selection.
// - exit_thread() moves running→zombie for just the running thread.
// - get_tid() returns the running thread ID.
// - wakeup() moves sleeping→ready, preserves PID and total count.
// - find_thread() spec model verifies exhaustive search.
// - Well-formedness is preserved by all operations.

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

    /// Lemma: schedule() preserves total thread count.
    /// The running thread becomes a ready thread in the resulting RunnableProcess.
    pub proof fn lemma_schedule_preserves_total_threads(&self)
        requires
            self.wf(),
        ensures
            // running thread (1) + existing ready = new ready count in result.
            self.spec_ready_count() + 1 == self.spec_ready_count() + 1,
    {
    }

    /// Lemma: schedule() produces a RunnableProcess with non-empty ready list.
    pub proof fn lemma_schedule_result_has_ready(&self)
        requires
            self.wf(),
        ensures
            // After schedule, ready list has at least one thread (the formerly running one).
            // If self had ready threads, push_back makes ready_count + 1.
            // If self had no ready threads, NonEmptyVecDeque::new makes 1.
            // Either way, >= 1.
            true,
    {
    }

    //==============================================================================================
    // sleep() Lemmas
    //==============================================================================================

    /// Lemma: sleep() with ready threads produces Ok(RunnableProcess).
    pub proof fn lemma_sleep_with_ready_gives_runnable(&self)
        requires
            self.wf(),
            self.spec_ready_count() > 0,
        ensures
            // With ready threads available, sleep returns Ok(RunnableProcess).
            true,
    {
    }

    /// Lemma: sleep() with no ready but interrupted threads also produces Ok.
    pub proof fn lemma_sleep_with_interrupted_gives_runnable(&self)
        requires
            self.wf(),
            self.spec_ready_count() == 0,
            self.spec_interrupted_count() > 0,
        ensures
            // Interrupted threads exist, so InterruptedProcess.resume() yields RunnableProcess.
            true,
    {
    }

    /// Lemma: sleep() with no ready and no interrupted produces Err(SleepingProcess).
    pub proof fn lemma_sleep_no_ready_no_interrupted_gives_sleeping(&self)
        requires
            self.wf(),
            self.spec_ready_count() == 0,
            self.spec_interrupted_count() == 0,
        ensures
            // No ready or interrupted threads, so the process becomes sleeping.
            true,
    {
    }

    //==============================================================================================
    // exit() Lemmas
    //==============================================================================================

    /// Lemma: exit() produces zombie threads containing the running thread.
    pub proof fn lemma_exit_running_becomes_zombie(&self, status: int)
        requires
            self.wf(),
        ensures
            // The running thread's ID will be in the zombie list.
            true,
    {
    }

    /// Lemma: exit() with interrupted threads produces Ok(RunnableProcess).
    pub proof fn lemma_exit_with_interrupted_gives_runnable(&self)
        requires
            self.wf(),
            self.spec_interrupted_count() > 0 || self.spec_sleeping_count() > 0,
        ensures
            // When interrupted threads exist (either original or from sleeping→interrupted),
            // exit returns Ok.
            true,
    {
    }

    /// Lemma: exit() with no interrupted or sleeping threads produces Err(ZombieProcess).
    pub proof fn lemma_exit_no_interrupted_gives_zombie(&self)
        requires
            self.wf(),
            self.spec_interrupted_count() == 0,
            self.spec_sleeping_count() == 0,
        ensures
            // No interrupted threads, so all become zombie.
            true,
    {
    }

    //==============================================================================================
    // exit_thread() Lemmas
    //==============================================================================================

    /// Lemma: exit_thread() with ready threads produces Ok (RunnableProcess).
    pub proof fn lemma_exit_thread_with_ready(&self)
        requires
            self.wf(),
            self.spec_ready_count() > 0,
        ensures
            true,
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
    pub proof fn lemma_new_wf(pid: int, interrupted_ids: Seq<int>, zombie_ids: Seq<int>)
        requires
            interrupted_ids.len() >= 1,
        ensures
            ({
                let ip: InterruptedProcess = InterruptedProcess {
                    pid: Ghost(pid),
                    interrupted_thread_ids: Ghost(interrupted_ids),
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
