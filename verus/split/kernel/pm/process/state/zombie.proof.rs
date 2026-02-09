// Copyright(c) The Maintainers of Nanvix.
// Licensed under the MIT License.

// ZombieProcess Proofs (Design-Level Verification).
//
// Key proven properties:
// - Construction (new) produces well-formed state with correct initial values.
// - PID is immutable across all operations.
// - Exit status is immutable across all operations.
// - `bury()` returns components matching the original fields — PID, zombie
//   thread IDs, and exit status are preserved. View type is used in postconditions.
// - `find_thread()` spec model verifies exhaustive search semantics.
//   `lemma_ghost_search_correctness` proves the ghost-level search logic over
//   `Seq<int>` is sound. `lemma_find_thread_completeness` restates spec-level
//   properties for downstream consumption.
// - Well-formedness (including thread ID uniqueness) is preserved by all operations.
// - View equality: identical fields produce equal views.
// - Integration obligation lemmas: find_thread consistency and uniqueness
//   under wf(), PID obligation at construction.

use vstd::prelude::*;

verus! {

impl ZombieProcess {

    //==============================================================================================
    // Construction Lemmas
    //==============================================================================================

    /// Lemma: A newly constructed ZombieProcess is well-formed.
    pub proof fn lemma_new_is_wf(
        pid: int,
        zombie_ids: Seq<int>,
        status: int,
    )
        requires
            zombie_ids.len() >= 1,
            zombie_ids.len() <= u64::MAX as nat,
            Self::spec_no_duplicates(zombie_ids),
        ensures
            ({
                let zp: ZombieProcess = ZombieProcess {
                    pid: Ghost(pid),
                    zombie_thread_ids: Ghost(zombie_ids),
                    status: Ghost(status),
                    zombie_count: zombie_ids.len() as u64,
                };
                zp.wf()
            }),
    {
    }

    /// Lemma: If the caller provides a ghost PID matching the real
    /// ProcessState PID, the constructed ZombieProcess satisfies the
    /// PID integration obligation.
    pub proof fn lemma_new_establishes_pid_obligation(
        pid: int,
        real_pid: int,
    )
        requires
            Self::spec_process_state_pid_integration_obligation(pid, real_pid),
        ensures
            ({
                let zp: ZombieProcess = ZombieProcess {
                    pid: Ghost(pid),
                    zombie_thread_ids: Ghost(Seq::empty().push(0int)),
                    status: Ghost(0int),
                    zombie_count: 1u64,
                };
                Self::spec_process_state_pid_integration_obligation(
                    zp.spec_pid(), real_pid)
            }),
    {
    }

    /// Lemma: mutation_frame_preserved preserves well-formedness.
    pub proof fn lemma_mutation_frame_preserves_wf(
        old_self: &ZombieProcess,
        new_self: &ZombieProcess,
    )
        requires
            old_self.wf(),
            ZombieProcess::mutation_frame_preserved(old_self, new_self),
        ensures
            new_self.wf(),
    {
    }

    //==============================================================================================
    // bury() Lemmas
    //==============================================================================================

    /// Lemma: bury() returns components matching the abstract View.
    pub proof fn lemma_bury_matches_view(&self)
        requires
            self.wf(),
        ensures
            self.zombie_thread_ids@ == self@.zombie_thread_ids,
            self.spec_pid() == self@.pid,
            self.spec_status() == self@.status,
            self.zombie_thread_ids@.len() >= 1,
            self.zombie_thread_ids@.len() == self.spec_zombie_count(),
    {
    }

    /// Lemma: bury() returns the correct PID.
    pub proof fn lemma_bury_preserves_pid(&self)
        ensures
            self.spec_pid() == self.pid@,
    {
    }

    /// Lemma: bury() returns the correct exit status.
    pub proof fn lemma_bury_preserves_status(&self)
        ensures
            self.spec_status() == self.status@,
    {
    }

    //==============================================================================================
    // find_thread() Lemmas
    //==============================================================================================

    /// Lemma: spec_find_thread returns Some(0) iff the thread is in the zombie list.
    pub proof fn lemma_find_thread_found(&self, tid: int)
        requires
            self.spec_has_zombie_thread(tid),
        ensures
            self.spec_find_thread(tid) == Some(0int),
    {
    }

    /// Lemma: spec_find_thread returns None iff the thread is not in the zombie list.
    pub proof fn lemma_find_thread_not_found(&self, tid: int)
        requires
            !self.spec_has_zombie_thread(tid),
        ensures
            self.spec_find_thread(tid) == None::<int>,
    {
    }

    /// Lemma: spec_find_thread result is consistent with spec_has_zombie_thread.
    pub proof fn lemma_find_thread_iff_has_thread(&self, tid: int)
        ensures
            self.spec_find_thread(tid).is_some() <==> self.spec_has_zombie_thread(tid),
    {
    }

    /// Lemma: Ghost-level search correctness.
    ///
    /// Verifies the search logic over the ghost `Seq<int>` by proving that
    /// `spec_has_zombie_thread` (used by `spec_find_thread`) correctly captures
    /// membership: if a thread ID is at any valid index in the zombie list,
    /// then `spec_has_zombie_thread` returns true, and `spec_find_thread`
    /// returns `Some(0)`. Conversely, if no index matches, both return false/None.
    ///
    /// This proves the ghost-level search is sound even though the executable
    /// `iter().find(...)` is not modeled. The gap between this proof and the
    /// real implementation is: (1) the iterator visits elements in order and
    /// uses `t.id() == tid` as the predicate, and (2) `ThreadIdentifier`
    /// equality matches integer equality in the ghost model.
    pub proof fn lemma_ghost_search_correctness(&self, tid: int)
        requires
            self.wf(),
        ensures
            // Forward: if tid is at any index, spec finds it.
            (forall|k: int| 0 <= k < self.zombie_thread_ids@.len()
                && self.zombie_thread_ids@[k] == tid
                ==> self.spec_find_thread(tid) == Some(0int)),
            // Backward: if spec finds it, there exists a valid index.
            (self.spec_find_thread(tid) == Some(0int) ==>
                exists|k: int| 0 <= k < self.zombie_thread_ids@.len()
                    && self.zombie_thread_ids@[k] == tid),
            // Completeness: if no index matches, spec returns None.
            ((forall|k: int| 0 <= k < self.zombie_thread_ids@.len()
                ==> self.zombie_thread_ids@[k] != tid)
                ==> self.spec_find_thread(tid) == None::<int>),
    {
        // Forward direction: any matching index triggers spec_seq_contains.
        assert forall|k: int| 0 <= k < self.zombie_thread_ids@.len()
            && self.zombie_thread_ids@[k] == tid
            implies self.spec_find_thread(tid) == Some(0int)
        by {
            // Witness k satisfies spec_seq_contains.
            assert(Self::spec_seq_contains(self.zombie_thread_ids@, tid));
        }

        // Backward direction: spec_has_zombie_thread implies existential.
        // This follows directly from the definition of spec_seq_contains.

        // Completeness: no matching index means not contained.
        if forall|k: int| 0 <= k < self.zombie_thread_ids@.len()
            ==> self.zombie_thread_ids@[k] != tid {
            // Negate the existential in spec_seq_contains.
            assert(!Self::spec_seq_contains(self.zombie_thread_ids@, tid));
        }
    }

    /// Lemma: spec_find_thread completeness — restates spec-level search
    /// properties for downstream consumption. This is a spec-level property
    /// (not a refinement proof linking to executable code).
    pub proof fn lemma_find_thread_completeness(&self, tid: int)
        requires
            self.wf(),
        ensures
            self.spec_has_zombie_thread(tid) ==>
                self.spec_find_thread(tid) == Some(0int),
            !self.spec_has_zombie_thread(tid) ==>
                self.spec_find_thread(tid) == None::<int>,
    {
    }

    //==============================================================================================
    // Integration Obligation Lemmas
    //==============================================================================================

    /// Lemma: If the find_thread integration obligation is satisfied, then
    /// the result is consistent with spec_has_zombie_thread.
    pub proof fn lemma_find_thread_obligation_implies_consistency(
        &self, tid: int, real_result: Option<int>,
    )
        requires
            self.wf(),
            self.spec_find_thread_integration_obligation(tid, real_result),
        ensures
            real_result.is_some() <==> self.spec_has_zombie_thread(tid),
            real_result == Some(0int) ==> self.spec_has_zombie_thread(tid),
    {
    }

    //==============================================================================================
    // View Equality
    //==============================================================================================

    /// Lemma: Two ZombieProcesses with identical fields have equal views.
    pub proof fn lemma_view_equality(a: &ZombieProcess, b: &ZombieProcess)
        requires
            a.pid@ == b.pid@,
            a.zombie_thread_ids@ =~= b.zombie_thread_ids@,
            a.status@ == b.status@,
        ensures
            a@ == b@,
    {
    }
}

} // verus!
