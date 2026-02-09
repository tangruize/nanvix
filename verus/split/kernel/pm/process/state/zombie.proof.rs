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
// - Integration obligation lemmas:
//   - `lemma_find_thread_obligation_implies_consistency`: find_thread result
//     consistency under wf().
//   - `lemma_predicate_obligation_implies_search_equivalence`: given explicit
//     `real_ids` sequence matching ghost IDs element-wise (search predicate
//     obligation), ghost search correctness implies real search correctness.
//     Takes `real_ids: Seq<int>` parameter to avoid the tautology of comparing
//     ghost IDs against themselves.
//   - `lemma_find_thread_mut_obligation_preserves_wf`: caller obligation for
//     mutable thread access preserves identity.
//   - `lemma_pid_stability_obligation_well_formed`: PID stability obligation
//     is well-formed (reflexive). Substantive PID preservation is **verified**
//     in the `process_state` dependency module.
//   - `lemma_bury_satisfies_identity_obligation`: ghost model satisfies the
//     identity part of the ownership transfer obligation.
//   - PID obligation at construction.

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
    /// Covers PID, status, and zombie thread IDs — all match the View fields.
    pub proof fn lemma_bury_matches_view(&self)
        requires
            self.wf(),
        ensures
            self.zombie_thread_ids@ == self@.zombie_thread_ids,
            self.spec_pid() == self@.pid,
            self.spec_pid() == self.pid@,
            self.spec_status() == self@.status,
            self.spec_status() == self.status@,
            self.zombie_thread_ids@.len() >= 1,
            self.zombie_thread_ids@.len() == self.spec_zombie_count(),
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

    /// Lemma: If ghost IDs match real IDs at every index (search predicate
    /// obligation), then ghost search correctness implies real search
    /// correctness. Takes an explicit `real_ids` sequence representing the
    /// concrete `NonEmptyVecDeque<ZombieThread>` iteration order.
    ///
    /// Under wf() (no-duplicates), if `real_ids` has the same length and
    /// element-wise equality with the ghost sequence, then:
    /// - A tid appearing in the real list implies the spec finds it.
    /// - The spec finding a tid implies it appears in the real list.
    pub proof fn lemma_predicate_obligation_implies_search_equivalence(
        &self, tid: int, real_ids: Seq<int>,
    )
        requires
            self.wf(),
            // Real IDs have the same length as ghost IDs.
            real_ids.len() == self.zombie_thread_ids@.len(),
            // Per-element predicate obligation: every ghost ID matches its real ID.
            forall|k: int| 0 <= k < self.zombie_thread_ids@.len() ==>
                Self::spec_find_thread_search_predicate_obligation(
                    #[trigger] self.zombie_thread_ids@[k],
                    real_ids[k],
                ),
        ensures
            // Forward: if tid is at any index in real_ids, spec finds it.
            (forall|k: int| 0 <= k < real_ids.len()
                && real_ids[k] == tid
                ==> self.spec_find_thread(tid) == Some(0int)),
            // Backward: if spec finds it, tid exists at some index in real_ids.
            (self.spec_find_thread(tid) == Some(0int) ==>
                exists|k: int| 0 <= k < real_ids.len()
                    && real_ids[k] == tid),
    {
        self.lemma_ghost_search_correctness(tid);

        // Forward: real_ids[k] == tid implies ghost_ids[k] == tid (by predicate obligation).
        assert forall|k: int| 0 <= k < real_ids.len()
            && real_ids[k] == tid
            implies self.spec_find_thread(tid) == Some(0int)
        by {
            // predicate obligation: ghost_ids[k] == real_ids[k] == tid.
            assert(self.zombie_thread_ids@[k] == real_ids[k]);
            assert(Self::spec_seq_contains(self.zombie_thread_ids@, tid));
        }

        // Backward: spec finds tid means exists ghost index k with ghost_ids[k] == tid.
        // By predicate obligation, real_ids[k] == ghost_ids[k] == tid.
        if self.spec_find_thread(tid) == Some(0int) {
            let k: int = choose|k: int| 0 <= k < self.zombie_thread_ids@.len()
                && self.zombie_thread_ids@[k] == tid;
            assert(real_ids[k] == self.zombie_thread_ids@[k]);
            assert(real_ids[k] == tid);
        }
    }

    /// Lemma: `find_thread_mut()` caller obligation preservation.
    /// If the caller preserves thread identity (obligation satisfied),
    /// then the zombie list remains unchanged and wf() is preserved.
    pub proof fn lemma_find_thread_mut_obligation_preserves_wf(
        &self, idx: int, old_tid: int, new_tid: int,
    )
        requires
            self.wf(),
            0 <= idx < self.zombie_thread_ids@.len(),
            self.zombie_thread_ids@[idx] == old_tid,
            Self::spec_find_thread_mut_caller_obligation(old_tid, new_tid),
        ensures
            // Identity preserved means the list is unchanged.
            new_tid == old_tid,
    {
    }

    /// Lemma: PID stability obligation is well-formed.
    ///
    /// Confirms the obligation formulation is trivially satisfiable (reflexive).
    /// The substantive proof that mutation preserves PID is in the **verified**
    /// `process_state` dependency module, where every public mutator has a
    /// verified postcondition `self.spec_pid() == old(self).spec_pid()`.
    /// This lemma documents that the obligation spec is consistent, not that
    /// mutation preserves PID — the latter is proven cross-module.
    pub proof fn lemma_pid_stability_obligation_well_formed(&self)
        requires
            self.wf(),
        ensures
            Self::spec_state_mut_pid_stability_obligation(
                self.spec_pid(), self.spec_pid()),
    {
    }

    /// Lemma: `bury()` ownership obligation is satisfied by the ghost model.
    /// The ghost model's bury() postconditions establish the identity part
    /// of the ownership obligation.
    pub proof fn lemma_bury_satisfies_identity_obligation(&self)
        requires
            self.wf(),
        ensures
            Self::spec_bury_ownership_integration_obligation(
                self.zombie_thread_ids@, self.zombie_thread_ids@,
                self.spec_pid(), self.spec_pid(),
                self.spec_status(), self.spec_status(),
            ),
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
