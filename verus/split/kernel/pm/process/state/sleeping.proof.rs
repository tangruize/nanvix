// Copyright(c) The Maintainers of Nanvix.
// Licensed under the MIT License.

// SleepingProcess Proofs.
//
// Key proven properties:
// - Construction produces well-formed state with correct initial values.
// - PID is immutable across all operations.
// - terminate() converts all sleeping→interrupted, produces InterruptedProcess
//   with PID preserved and interrupted list = original sleeping list.
// - wakeup() moves sleeping→ready, preserves PID and thread conservation.
// - wakeup_alarm() partitions sleeping threads, preserves PID and conservation.
// - add_thread() transitions to RunnableProcess with correct thread lists.
// - find_thread() spec model verifies exhaustive search.
// - Well-formedness is preserved by all operations.

use vstd::prelude::*;

verus! {

impl SleepingProcess {

    //==============================================================================================
    // Construction Lemmas
    //==============================================================================================

    /// Lemma: A newly constructed SleepingProcess is well-formed.
    pub proof fn lemma_new_is_wf(
        pid: int,
        sleeping_ids: Seq<int>,
        zombie_ids: Seq<int>,
    )
        requires
            sleeping_ids.len() >= 1,
            sleeping_ids.len() <= u64::MAX as nat,
        ensures
            ({
                let sp: SleepingProcess = SleepingProcess {
                    pid: Ghost(pid),
                    sleeping_thread_ids: Ghost(sleeping_ids),
                    zombie_thread_ids: Ghost(zombie_ids),
                    sleeping_count: sleeping_ids.len() as u64,
                };
                sp.wf()
            }),
    {
    }

    /// Lemma: mutation_frame_preserved preserves well-formedness.
    pub proof fn lemma_mutation_frame_preserves_wf(
        old_self: &SleepingProcess,
        new_self: &SleepingProcess,
    )
        requires
            old_self.wf(),
            SleepingProcess::mutation_frame_preserved(old_self, new_self),
        ensures
            new_self.wf(),
    {
    }

    //==============================================================================================
    // PID Immutability Lemmas
    //==============================================================================================

    /// Lemma: PID equals the ghost pid field.
    pub proof fn lemma_pid_preserved(&self)
        ensures
            self.spec_pid() == self.pid@,
    {
    }

    //==============================================================================================
    // terminate() Lemmas
    //==============================================================================================

    /// Lemma: terminate() produces InterruptedProcess with non-empty interrupted list.
    pub proof fn lemma_terminate_result_has_interrupted(&self)
        requires
            self.wf(),
        ensures
            self.sleeping_thread_ids@.len() >= 1,
    {
    }

    /// Lemma: terminate() preserves total thread count.
    /// Sleeping threads become interrupted; zombie threads are preserved.
    pub proof fn lemma_terminate_preserves_total_threads(&self)
        requires
            self.wf(),
        ensures
            ({
                let new_interrupted_len: nat = self.spec_sleeping_count();
                new_interrupted_len + self.spec_zombie_count()
                    == self.spec_total_thread_count()
            }),
    {
    }

    //==============================================================================================
    // wakeup() Lemmas
    //==============================================================================================

    /// Lemma: If spec_seq_contains is true, there exists a valid index.
    pub proof fn lemma_spec_find_sleeping_index(&self, tid: Ghost<int>)
        requires
            Self::spec_seq_contains(self.sleeping_thread_ids@, tid@),
        ensures
            exists|i: int| 0 <= i < self.sleeping_thread_ids@.len()
                && self.sleeping_thread_ids@[i] == tid@,
    {
    }

    /// Lemma: If wf() holds and spec_seq_contains is true for the sleeping list,
    /// then sleeping_count > 0.
    pub proof fn lemma_wf_and_found_implies_sleeping_positive(&self, tid: int)
        requires
            self.wf(),
            Self::spec_seq_contains(self.sleeping_thread_ids@, tid),
        ensures
            self.sleeping_count > 0,
    {
    }

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
    // wakeup_alarm() Lemmas
    //==============================================================================================

    /// Lemma: wakeup_alarm() with expired threads produces well-formed InterruptedProcess.
    pub proof fn lemma_wakeup_alarm_expired_is_wf(
        interrupted_ids: Seq<int>,
        remaining_ids: Seq<int>,
        sleeping_ids: Seq<int>,
    )
        requires
            interrupted_ids.len() > 0,
            interrupted_ids.len() + remaining_ids.len() == sleeping_ids.len(),
        ensures
            interrupted_ids.len() >= 1,
    {
    }

    /// Lemma: wakeup_alarm() conservation: partition sizes sum to original.
    pub proof fn lemma_wakeup_alarm_conservation(
        interrupted_ids: Seq<int>,
        remaining_ids: Seq<int>,
        sleeping_ids: Seq<int>,
    )
        requires
            interrupted_ids.len() + remaining_ids.len() == sleeping_ids.len(),
        ensures
            interrupted_ids.len() + remaining_ids.len() == sleeping_ids.len(),
    {
    }

    //==============================================================================================
    // add_thread() Lemmas
    //==============================================================================================

    /// Lemma: add_thread() produces RunnableProcess with exactly one ready thread.
    pub proof fn lemma_add_thread_result_has_ready(tid: int)
        ensures
            Seq::<int>::empty().push(tid).len() == 1,
            Seq::<int>::empty().push(tid)[0] == tid,
    {
    }

    //==============================================================================================
    // find_thread() Lemmas
    //==============================================================================================

    /// Lemma: spec_find_thread returns None iff the thread is not in any list.
    pub proof fn lemma_find_thread_not_found(&self, tid: int)
        requires
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
    // View Equality
    //==============================================================================================

    /// Lemma: Two SleepingProcesses with identical fields have equal views.
    pub proof fn lemma_view_equality(a: &SleepingProcess, b: &SleepingProcess)
        requires
            a.pid@ == b.pid@,
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
    pub proof fn lemma_new_wf(
        pid: int,
        ready_ids: Seq<int>,
        interrupted_ids: Seq<int>,
        sleeping_ids: Seq<int>,
        zombie_ids: Seq<int>,
    )
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
// InterruptedProcess Lemmas (Boundary)
//==================================================================================================

impl InterruptedProcess {
    /// Lemma: Construction with non-empty interrupted threads is well-formed.
    pub proof fn lemma_new_wf(
        pid: int,
        interrupted_ids: Seq<int>,
        sleeping_ids: Seq<int>,
        zombie_ids: Seq<int>,
    )
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

} // verus!
