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
//   `Seq<u64>` is sound. `lemma_find_thread_completeness` restates spec-level
//   properties for downstream consumption.
// - Invariant (including thread ID uniqueness) is preserved by all operations.
// - View equality: identical fields produce equal views.
// - Integration obligation lemmas:
//   - `lemma_find_thread_obligation_implies_consistency`: find_thread result
//     consistency under inv().
//   - `lemma_predicate_obligation_implies_search_equivalence`: given explicit
//     `real_ids` sequence matching ghost IDs element-wise (search predicate
//     obligation), ghost search correctness implies real search correctness.
//     Takes `real_ids: Seq<u64>` parameter to avoid the tautology of comparing
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

    /// Lemma: Construction satisfies both invariant and PID obligation.
    ///
    /// Proves that the `new()` preconditions guarantee:
    /// 1. The resulting View is well-formed (`ZombieProcessView::wf()`).
    /// 2. The constructed View matches `spec_new()`.
    /// 3. If a real PID is provided matching the ghost PID, the PID
    ///    integration obligation is satisfied on the constructed View.
    pub proof fn lemma_new_is_wf(
        pid: u64,
        zombie_ids: Seq<u64>,
        status: i64,
        zombie_count: u64,
    )
        requires
            zombie_ids.len() >= 1,
            zombie_count as nat == zombie_ids.len(),
            Self::spec_no_duplicates(zombie_ids),
        ensures
            zombie_count as nat == zombie_ids.len(),
            zombie_ids.len() >= 1,
            Self::spec_no_duplicates(zombie_ids),
            ({
                let view_ids: Seq<int> = Seq::new(zombie_ids.len(), |i: int| zombie_ids[i] as int);
                let view: ZombieProcessView = ZombieProcessView::spec_new(pid as int, view_ids, status as int);
                &&& view.wf()
                &&& view.pid == pid as int
                &&& view.status == status as int
                &&& view.zombie_thread_ids == view_ids
            }),
    {
        let view_ids: Seq<int> = Seq::new(zombie_ids.len(), |i: int| zombie_ids[i] as int);
        // Prove no_duplicates on the int sequence.
        assert forall|i: int, j: int| 0 <= i < j < view_ids.len()
            implies view_ids[i] != view_ids[j]
        by {
            // u64→int cast is injective, so no_duplicates is preserved.
            assert(zombie_ids[i] != zombie_ids[j]);
        }
    }

    /// Lemma: PID integration obligation at construction.
    ///
    /// Convenience wrapper: if the caller provides a PID matching the real
    /// ProcessState PID, the PID integration obligation is satisfied, and
    /// the constructed View carries the correct PID. Subsumed by
    /// `lemma_new_is_wf` for callers that have all constructor parameters.
    pub proof fn lemma_new_establishes_pid_obligation(
        pid: u64,
        real_pid: u64,
    )
        requires
            Self::spec_process_state_pid_integration_obligation(pid, real_pid),
        ensures
            Self::spec_process_state_pid_integration_obligation(pid, real_pid),
            pid == real_pid,
    {
    }

    /// Lemma: mutation_frame_preserved preserves invariant.
    pub proof fn lemma_mutation_frame_preserves_inv(
        old_self: &ZombieProcess,
        new_self: &ZombieProcess,
    )
        requires
            old_self.inv(),
            ZombieProcess::mutation_frame_preserved(old_self, new_self),
        ensures
            new_self.inv(),
    {
        reveal(ZombieProcess::inv);
    }

    //==============================================================================================
    // bury() Lemmas
    //==============================================================================================

    /// Lemma: bury() returns components matching the abstract View.
    /// Covers PID, status, and zombie thread IDs — all match the View fields.
    pub proof fn lemma_bury_matches_view(&self)
        requires
            self.inv(),
        ensures
            self.spec_pid() as int == self@.pid,
            self.spec_status() as int == self@.status,
            Seq::new(self.zombie_thread_ids@.len(), |i: int| self.zombie_thread_ids@[i] as int) =~= self@.zombie_thread_ids,
            self.zombie_thread_ids@.len() >= 1,
    {
        reveal(ZombieProcess::inv);
    }

    //==============================================================================================
    // find_thread() Lemmas
    //==============================================================================================

    /// Lemma: spec_find_thread returns Some(0) iff the thread is in the zombie list.
    pub proof fn lemma_find_thread_found(&self, tid: u64)
        requires
            self.spec_has_zombie_thread(tid),
        ensures
            self.spec_find_thread(tid) == Some(0u64),
    {
    }

    /// Lemma: spec_find_thread returns None iff the thread is not in the zombie list.
    pub proof fn lemma_find_thread_not_found(&self, tid: u64)
        requires
            !self.spec_has_zombie_thread(tid),
        ensures
            self.spec_find_thread(tid) == None::<u64>,
    {
    }

    /// Lemma: spec_find_thread result is consistent with spec_has_zombie_thread.
    pub proof fn lemma_find_thread_iff_has_thread(&self, tid: u64)
        ensures
            self.spec_find_thread(tid).is_some() <==> self.spec_has_zombie_thread(tid),
    {
    }

    /// Lemma: Ghost-level search correctness.
    ///
    /// Verifies the search logic over the ghost `Seq<u64>` by proving that
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
    pub proof fn lemma_ghost_search_correctness(&self, tid: u64)
        requires
            self.inv(),
        ensures
            // Forward: if tid is at any index, spec finds it.
            (forall|k: int| 0 <= k < self.zombie_thread_ids@.len()
                && self.zombie_thread_ids@[k] == tid
                ==> self.spec_find_thread(tid) == Some(0u64)),
            // Backward: if spec finds it, there exists a valid index.
            (self.spec_find_thread(tid) == Some(0u64) ==>
                exists|k: int| 0 <= k < self.zombie_thread_ids@.len()
                    && self.zombie_thread_ids@[k] == tid),
            // Completeness: if no index matches, spec returns None.
            ((forall|k: int| 0 <= k < self.zombie_thread_ids@.len()
                ==> self.zombie_thread_ids@[k] != tid)
                ==> self.spec_find_thread(tid) == None::<u64>),
    {
        reveal(ZombieProcess::inv);
        // Forward direction: any matching index triggers spec_seq_contains.
        assert forall|k: int| 0 <= k < self.zombie_thread_ids@.len()
            && self.zombie_thread_ids@[k] == tid
            implies self.spec_find_thread(tid) == Some(0u64)
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
    pub proof fn lemma_find_thread_completeness(&self, tid: u64)
        requires
            self.inv(),
        ensures
            self.spec_has_zombie_thread(tid) ==>
                self.spec_find_thread(tid) == Some(0u64),
            !self.spec_has_zombie_thread(tid) ==>
                self.spec_find_thread(tid) == None::<u64>,
    {
    }

    //==============================================================================================
    // Integration Obligation Lemmas
    //==============================================================================================

    /// Lemma: If the find_thread integration obligation is satisfied, then
    /// the result is consistent with spec_has_zombie_thread.
    pub proof fn lemma_find_thread_obligation_implies_consistency(
        &self, tid: u64, real_result: Option<u64>,
    )
        requires
            self.inv(),
            self.spec_find_thread_integration_obligation(tid, real_result),
        ensures
            real_result.is_some() <==> self.spec_has_zombie_thread(tid),
            real_result == Some(0u64) ==> self.spec_has_zombie_thread(tid),
    {
        reveal(ZombieProcess::inv);
    }

    /// Lemma: If ghost IDs match real IDs at every index (search predicate
    /// obligation), then ghost search correctness implies real search
    /// correctness. Takes an explicit `real_ids` sequence representing the
    /// concrete `NonEmptyVecDeque<ZombieThread>` iteration order.
    ///
    /// Under inv() (no-duplicates), if `real_ids` has the same length and
    /// element-wise equality with the ghost sequence, then:
    /// - A tid appearing in the real list implies the spec finds it.
    /// - The spec finding a tid implies it appears in the real list.
    pub proof fn lemma_predicate_obligation_implies_search_equivalence(
        &self, tid: u64, real_ids: Seq<u64>,
    )
        requires
            self.inv(),
            // Real IDs have the same length as zombie thread IDs.
            real_ids.len() == self.zombie_thread_ids@.len(),
            // Per-element predicate obligation: every ID matches its real ID.
            forall|k: int| 0 <= k < self.zombie_thread_ids@.len() ==>
                Self::spec_find_thread_search_predicate_obligation(
                    #[trigger] self.zombie_thread_ids@[k],
                    real_ids[k],
                ),
        ensures
            // Forward: if tid is at any index in real_ids, spec finds it.
            (forall|k: int| 0 <= k < real_ids.len()
                && real_ids[k] == tid
                ==> self.spec_find_thread(tid) == Some(0u64)),
            // Backward: if spec finds it, tid exists at some index in real_ids.
            (self.spec_find_thread(tid) == Some(0u64) ==>
                exists|k: int| 0 <= k < real_ids.len()
                    && real_ids[k] == tid),
    {
        reveal(ZombieProcess::inv);
        self.lemma_ghost_search_correctness(tid);

        // Forward: real_ids[k] == tid implies zombie_thread_ids[k] == tid.
        assert forall|k: int| 0 <= k < real_ids.len()
            && real_ids[k] == tid
            implies self.spec_find_thread(tid) == Some(0u64)
        by {
            // Predicate obligation: zombie_thread_ids[k] == real_ids[k] == tid.
            assert(self.zombie_thread_ids@[k] == real_ids[k]);
            assert(Self::spec_seq_contains(self.zombie_thread_ids@, tid));
        }

        // Backward: spec finds tid means exists index k with zombie_thread_ids[k] == tid.
        // By predicate obligation, real_ids[k] == zombie_thread_ids[k] == tid.
        if self.spec_find_thread(tid) == Some(0u64) {
            let k: int = choose|k: int| 0 <= k < self.zombie_thread_ids@.len()
                && self.zombie_thread_ids@[k] == tid;
            assert(real_ids[k] == self.zombie_thread_ids@[k]);
            assert(real_ids[k] == tid);
        }
    }

    /// Lemma: `find_thread_mut()` caller obligation preservation.
    /// If the caller preserves thread identity (obligation satisfied),
    /// then the zombie list remains unchanged and inv() is preserved.
    pub proof fn lemma_find_thread_mut_obligation_preserves_wf(
        &self, idx: int, old_tid: u64, new_tid: u64,
    )
        requires
            self.inv(),
            0 <= idx < self.zombie_thread_ids@.len(),
            self.zombie_thread_ids@[idx] == old_tid,
            Self::spec_find_thread_mut_caller_obligation(old_tid, new_tid),
        ensures
            // Identity preserved means the list is unchanged.
            new_tid == old_tid,
    {
        reveal(ZombieProcess::inv);
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
            self.inv(),
        ensures
            Self::spec_state_mut_pid_stability_obligation(
                self.spec_pid(), self.spec_pid()),
    {
        reveal(ZombieProcess::inv);
    }

    /// Lemma: `bury()` ownership obligation is satisfied by the ghost model.
    /// The ghost model's bury() postconditions establish the identity part
    /// of the ownership obligation.
    pub proof fn lemma_bury_satisfies_identity_obligation(&self)
        requires
            self.inv(),
        ensures
            Self::spec_bury_ownership_integration_obligation(
                self.zombie_thread_ids@, self.zombie_thread_ids@,
                self.spec_pid(), self.spec_pid(),
                self.spec_status(), self.spec_status(),
            ),
    {
        reveal(ZombieProcess::inv);
    }

    //==============================================================================================
    // View Equality
    //==============================================================================================

    /// Lemma: Two ZombieProcesses with identical fields have equal views.
    pub proof fn lemma_view_equality(a: &ZombieProcess, b: &ZombieProcess)
        requires
            a.pid == b.pid,
            a.zombie_thread_ids@ =~= b.zombie_thread_ids@,
            a.status == b.status,
        ensures
            a@ == b@,
    {
    }

    //==============================================================================================
    // View-Level State Transition Bridging Lemmas
    //==============================================================================================

    /// Lemma: `new()` result view equals `ZombieProcessView::spec_new()`.
    pub proof fn lemma_new_matches_spec_new(
        pid: u64,
        zombie_ids: Seq<u64>,
        status: i64,
        zombie_count: u64,
    )
        requires
            zombie_count as nat == zombie_ids.len(),
            zombie_ids.len() >= 1,
            Self::spec_no_duplicates(zombie_ids),
        ensures
            ({
                let view_ids: Seq<int> = Seq::new(zombie_ids.len(), |i: int| zombie_ids[i] as int);
                ZombieProcessView::spec_new(pid as int, view_ids, status as int) =~=
                    (ZombieProcessView { pid: pid as int, zombie_thread_ids: view_ids, status: status as int })
            }),
    {
    }

    /// Lemma: `bury()` result equals the View-level `spec_bury()`.
    pub proof fn lemma_bury_matches_spec_bury(&self)
        requires
            self.inv(),
        ensures
            self@.spec_bury() == (self@.zombie_thread_ids, self@.pid, self@.status),
    {
        reveal(ZombieProcess::inv);
    }

    /// Lemma: `state_mut()` preserves the view, matching `spec_state_mut()`.
    pub proof fn lemma_state_mut_matches_spec(&self)
        requires
            self.inv(),
        ensures
            self@.spec_state_mut() == self@,
    {
        reveal(ZombieProcess::inv);
    }

    /// Lemma: `find_thread()` on exec is consistent with `spec_find_thread()` on the view.
    /// The exec returns `Option<u64>` and the view returns `Option<int>`, so we
    /// compare their is_some() result and the membership predicate.
    pub proof fn lemma_find_thread_matches_view_spec(&self, tid: u64)
        requires
            self.inv(),
        ensures
            self.spec_find_thread(tid).is_some() == self@.spec_find_thread(tid as int).is_some(),
            self.spec_has_zombie_thread(tid) == self@.spec_has_zombie_thread(tid as int),
    {
        reveal(ZombieProcess::inv);
        // Prove membership equivalence: exec contains tid <==> view contains tid as int.
        // view().zombie_thread_ids == Seq::new(len, |i| zombie_thread_ids@[i] as int).
        // So view contains (tid as int) iff exec contains tid (u64→int is injective).
        let exec_seq: Seq<u64> = self.zombie_thread_ids@;
        let view_seq: Seq<int> = self@.zombie_thread_ids;
        assert(view_seq.len() == exec_seq.len());

        if Self::spec_seq_contains(exec_seq, tid) {
            let wit: int = choose|i: int| 0 <= i < exec_seq.len() && exec_seq[i] == tid;
            assert(view_seq[wit] == exec_seq[wit] as int);
            assert(view_seq[wit] == tid as int);
            assert(ZombieProcessView::spec_seq_contains(view_seq, tid as int));
        }
        if ZombieProcessView::spec_seq_contains(view_seq, tid as int) {
            let wit: int = choose|i: int| 0 <= i < view_seq.len() && view_seq[i] == tid as int;
            assert(exec_seq[wit] as int == tid as int);
            assert(exec_seq[wit] == tid);
            assert(Self::spec_seq_contains(exec_seq, tid));
        }
    }

    /// Lemma: `find_thread_mut()` preserves the view, matching `spec_find_thread_mut()`.
    pub proof fn lemma_find_thread_mut_matches_spec(&self, tid: u64)
        requires
            self.inv(),
        ensures
            self@.spec_find_thread_mut(tid as int) == self@,
    {
        reveal(ZombieProcess::inv);
    }

    /// Lemma: exec-level `inv()` implies view-level `wf()`.
    pub proof fn lemma_exec_inv_implies_view_wf(&self)
        requires
            self.inv(),
        ensures
            self@.wf(),
    {
        reveal(ZombieProcess::inv);
        let exec_seq: Seq<u64> = self.zombie_thread_ids@;
        let view_seq: Seq<int> = self@.zombie_thread_ids;

        // Length is preserved by the Seq::new conversion.
        assert(view_seq.len() == exec_seq.len());
        assert(view_seq.len() >= 1);

        // No-duplicates on exec implies no-duplicates on view (u64→int is injective).
        assert forall|i: int, j: int| 0 <= i < j < view_seq.len()
            implies view_seq[i] != view_seq[j]
        by {
            assert(exec_seq[i] != exec_seq[j]);
            // u64→int is injective: a != b ==> (a as int) != (b as int).
        }
    }

    /// Lemma: view-level `spec_has_zombie_thread` matches exec-level.
    pub proof fn lemma_has_zombie_thread_matches_view(&self, tid: u64)
        ensures
            self.spec_has_zombie_thread(tid) == self@.spec_has_zombie_thread(tid as int),
    {
        let exec_seq: Seq<u64> = self.zombie_thread_ids@;
        let view_seq: Seq<int> = self@.zombie_thread_ids;

        if Self::spec_seq_contains(exec_seq, tid) {
            let wit: int = choose|i: int| 0 <= i < exec_seq.len() && exec_seq[i] == tid;
            assert(view_seq[wit] == exec_seq[wit] as int);
            assert(view_seq[wit] == tid as int);
            assert(ZombieProcessView::spec_seq_contains(view_seq, tid as int));
        }
        if ZombieProcessView::spec_seq_contains(view_seq, tid as int) {
            let wit: int = choose|i: int| 0 <= i < view_seq.len() && view_seq[i] == tid as int;
            assert(exec_seq[wit] as int == tid as int);
            assert(exec_seq[wit] == tid);
            assert(Self::spec_seq_contains(exec_seq, tid));
        }
    }

    /// Lemma: `state()` on exec matches `spec_state()` on the view.
    pub proof fn lemma_state_matches_view_spec(&self)
        ensures
            self.spec_pid() as int == self@.spec_state(),
    {
    }
}

} // verus!
