// Copyright(c) The Maintainers of Nanvix.
// Licensed under the MIT License.

// SleepingProcess Proofs.
//
// Key proven properties:
// - Construction produces well-formed state with correct initial values.
// - terminate() converts all sleeping→interrupted, produces InterruptedProcess
//   with PID preserved and interrupted list = original sleeping list.
// - wakeup() moves sleeping→ready, preserves PID and thread conservation.
//   Under wf() uniqueness, the existential in the postcondition is unique.
// - wakeup_alarm() partitions sleeping threads, preserves PID and conservation.
// - add_thread() transitions to RunnableProcess with correct thread lists.
// - find_thread() spec model verifies exhaustive search.
// - Well-formedness (including thread ID uniqueness) is preserved by all operations.

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
            Self::spec_no_duplicates(sleeping_ids),
            Self::spec_no_duplicates(zombie_ids),
            Self::spec_seqs_disjoint(sleeping_ids, zombie_ids),
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
