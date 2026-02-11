// Copyright(c) The Maintainers of Nanvix.
// Licensed under the MIT License.

// Proof lemmas for RunningProcess and boundary process types.

use vstd::prelude::*;

verus! {

//==================================================================================================
// RunningProcess Lemmas
//==================================================================================================

impl RunningProcess {
    /// A well-formed RunningProcess has matching counts and lengths.
    ///
    /// This is essentially the definition of wf(), but stated as a lemma for callers
    /// that hold a reference to a well-formed process.
    pub proof fn lemma_new_is_wf(&self)
        requires
            self.ready_count as nat == self.ready_thread_ids@.len(),
            self.interrupted_count as nat == self.interrupted_thread_ids@.len(),
            self.sleeping_count as nat == self.sleeping_thread_ids@.len(),
            self.zombie_count as nat == self.zombie_thread_ids@.len(),
        ensures
            self.wf(),
    {
    }

    /// wf_strict implies wf.
    pub proof fn lemma_wf_strict_implies_wf(&self)
        requires
            self.wf_strict(),
        ensures
            self.wf(),
    {
    }

    /// Two RunningProcesses with identical views are observationally equal.
    pub proof fn lemma_view_equality(a: &RunningProcess, b: &RunningProcess)
        requires
            a@ == b@,
        ensures
            a.spec_pid() == b.spec_pid(),
            a.spec_running_thread_id() == b.spec_running_thread_id(),
            a.ready_thread_ids@ == b.ready_thread_ids@,
            a.interrupted_thread_ids@ == b.interrupted_thread_ids@,
            a.sleeping_thread_ids@ == b.sleeping_thread_ids@,
            a.zombie_thread_ids@ == b.zombie_thread_ids@,
    {
    }

    /// Removing an element from a sequence decreases its length by one.
    pub proof fn lemma_remove_at_length(s: Seq<u64>, idx: int)
        requires
            0 <= idx < s.len(),
        ensures
            Self::spec_remove_at(s, idx).len() == s.len() - 1,
    {
        let left: Seq<u64> = s.subrange(0, idx);
        let right: Seq<u64> = s.subrange(idx + 1, s.len() as int);
        assert(left.len() == idx as nat);
        assert(right.len() == (s.len() - idx as nat - 1) as nat);
        assert(left.add(right).len() == (s.len() - 1) as nat);
    }

    /// After removing a zombie thread via try_join_thread, the zombie list shrinks by one.
    pub proof fn lemma_try_join_zombie_post_shrinks(&self, tid: u64)
        requires
            self.spec_try_join_thread(tid) == JOIN_TAG_ZOMBIE as int,
            self.spec_has_zombie_thread(tid),
        ensures
            ({
                let post_zombies: Seq<u64> = self.spec_try_join_zombie_post(tid);
                post_zombies.len() == self.spec_zombie_count() - 1
            }),
    {
        let s: Seq<u64> = self.zombie_thread_ids@;
        let idx: int = choose|i: int|
            0 <= i < s.len()
            && #[trigger] s[i] == tid;
        Self::lemma_remove_at_length(s, idx);
    }
}

//==================================================================================================
// RunnableProcess Lemmas
//==================================================================================================

impl RunnableProcess {
    /// A RunnableProcess with non-empty ready list is well-formed.
    pub proof fn lemma_new_wf(&self)
        requires
            self.ready_thread_ids@.len() >= 1,
        ensures
            self.wf(),
    {
    }
}

//==================================================================================================
// SleepingProcess Lemmas
//==================================================================================================

impl SleepingProcess {
    /// A SleepingProcess with non-empty sleeping list is well-formed.
    pub proof fn lemma_new_wf(&self)
        requires
            self.sleeping_thread_ids@.len() >= 1,
        ensures
            self.wf(),
    {
    }
}

//==================================================================================================
// InterruptedProcess Lemmas
//==================================================================================================

impl InterruptedProcess {
    /// An InterruptedProcess with non-empty interrupted list is well-formed.
    pub proof fn lemma_new_wf(&self)
        requires
            self.interrupted_thread_ids@.len() >= 1,
        ensures
            self.wf(),
    {
    }
}

//==================================================================================================
// ZombieProcess Lemmas
//==================================================================================================

impl ZombieProcess {
    /// A ZombieProcess with non-empty zombie list is well-formed.
    pub proof fn lemma_new_wf(&self)
        requires
            self.zombie_thread_ids@.len() >= 1,
        ensures
            self.wf(),
    {
    }
}

} // verus!
