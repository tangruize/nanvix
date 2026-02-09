// Copyright(c) The Maintainers of Nanvix.
// Licensed under the MIT License.

// InterruptedProcess Proofs (Design-Level Verification).
//
// Key proven properties:
// - Construction (new, from_sleeping) produces well-formed state with correct identity.
// - PID is immutable across all operations.
// - `resume()` pops the front interrupted thread, resumes it (ID-preserving),
//   and produces a RunnableProcess with that thread as the only ready thread.
//   Remaining interrupted threads, sleeping threads, and zombie threads are preserved.
//   The resulting RunnableProcess is well-formed.
// - `find_thread()` spec model verifies exhaustive search semantics.
// - `interrupt()` standalone function is ID-preserving with reason tag.
// - Well-formedness (including thread ID uniqueness and disjointness) is
//   preserved by all operations.
// - View equality: identical fields produce equal views.
// - Projection lemma for cross-module boundary model linking (extracts
//   runnable-compatible tuple with invariants).
// - Admission time oracle satisfies resume() precondition.
// - Integration obligation lemmas: find_thread result consistency and
//   uniqueness under wf() (thread can only be in one list).
// - Interrupt reason obligation discharge for Killed variant.

use vstd::prelude::*;

verus! {

impl InterruptedProcess {

    //==============================================================================================
    // Construction Lemmas
    //==============================================================================================

    /// Lemma: A newly constructed InterruptedProcess (via `new`) is well-formed.
    /// Note: This lemma restates what the constructor's `ensures result.wf()` already
    /// guarantees. It is retained as a regression guard for downstream proofs that
    /// may construct InterruptedProcess values directly (without calling `new()`).
    pub proof fn lemma_new_is_wf(
        pid: int,
        interrupted_ids: Seq<int>,
        zombie_ids: Seq<int>,
    )
        requires
            interrupted_ids.len() >= 1,
            Self::spec_no_duplicates(interrupted_ids),
            Self::spec_no_duplicates(zombie_ids),
            Self::spec_seqs_disjoint(interrupted_ids, zombie_ids),
        ensures
            ({
                let ip: InterruptedProcess = InterruptedProcess {
                    pid: Ghost(pid),
                    sleeping_thread_ids: Ghost(Seq::empty()),
                    interrupted_thread_ids: Ghost(interrupted_ids),
                    zombie_thread_ids: Ghost(zombie_ids),
                };
                ip.wf()
            }),
    {
    }

    /// Lemma: A newly constructed InterruptedProcess (via `from_sleeping`) is well-formed.
    /// Note: Retained as a regression guard (see `lemma_new_is_wf` note).
    pub proof fn lemma_from_sleeping_is_wf(
        pid: int,
        sleeping_ids: Seq<int>,
        interrupted_ids: Seq<int>,
        zombie_ids: Seq<int>,
    )
        requires
            interrupted_ids.len() >= 1,
            Self::spec_no_duplicates(interrupted_ids),
            Self::spec_no_duplicates(sleeping_ids),
            Self::spec_no_duplicates(zombie_ids),
            Self::spec_seqs_disjoint(interrupted_ids, sleeping_ids),
            Self::spec_seqs_disjoint(interrupted_ids, zombie_ids),
            Self::spec_seqs_disjoint(sleeping_ids, zombie_ids),
        ensures
            ({
                let ip: InterruptedProcess = InterruptedProcess {
                    pid: Ghost(pid),
                    sleeping_thread_ids: Ghost(sleeping_ids),
                    interrupted_thread_ids: Ghost(interrupted_ids),
                    zombie_thread_ids: Ghost(zombie_ids),
                };
                ip.wf()
            }),
    {
    }

    /// Lemma: mutation_frame_preserved preserves well-formedness.
    pub proof fn lemma_mutation_frame_preserves_wf(
        old_self: &InterruptedProcess,
        new_self: &InterruptedProcess,
    )
        requires
            old_self.wf(),
            InterruptedProcess::mutation_frame_preserved(old_self, new_self),
        ensures
            new_self.wf(),
    {
    }

    //==============================================================================================
    // resume() Lemmas
    //==============================================================================================

    /// Lemma: The front element of a non-empty sequence is a valid element.
    pub proof fn lemma_front_element_valid(&self)
        requires
            self.interrupted_thread_ids@.len() >= 1,
        ensures
            0 < self.interrupted_thread_ids@.len(),
            Self::spec_seq_contains(
                self.interrupted_thread_ids@,
                self.interrupted_thread_ids@[0],
            ),
    {
        assert(self.interrupted_thread_ids@[0]
            == self.interrupted_thread_ids@[0]);
    }

    /// Lemma: Dropping the first element preserves no-duplicates.
    pub proof fn lemma_subrange_preserves_no_duplicates(s: Seq<int>)
        requires
            s.len() >= 1,
            Self::spec_no_duplicates(s),
        ensures
            Self::spec_no_duplicates(s.subrange(1, s.len() as int)),
    {
        let tail: Seq<int> = s.subrange(1, s.len() as int);
        assert forall|i: int, j: int| 0 <= i < j < tail.len()
            implies tail[i] != tail[j]
        by {
            assert(tail[i] == s[i + 1]);
            assert(tail[j] == s[j + 1]);
            assert(0 <= i + 1 < j + 1 < s.len());
        }
    }

    /// Lemma: The front element is not in the tail (under no-duplicates).
    pub proof fn lemma_front_not_in_tail(s: Seq<int>)
        requires
            s.len() >= 1,
            Self::spec_no_duplicates(s),
        ensures
            !Self::spec_seq_contains(s.subrange(1, s.len() as int), s[0]),
    {
        let tail: Seq<int> = s.subrange(1, s.len() as int);
        if Self::spec_seq_contains(tail, s[0]) {
            let k: int = choose|k: int| 0 <= k < tail.len() && tail[k] == s[0];
            assert(tail[k] == s[k + 1]);
            assert(s[0] == s[k + 1]);
            assert(0 <= 0int < k + 1 < s.len());
        }
    }

    /// Lemma: Dropping the first element of interrupted_thread_ids preserves
    /// disjointness with sleeping_thread_ids.
    pub proof fn lemma_tail_disjoint_sleeping(s: Seq<int>, sleeping: Seq<int>)
        requires
            s.len() >= 1,
            Self::spec_seqs_disjoint(s, sleeping),
        ensures
            Self::spec_seqs_disjoint(s.subrange(1, s.len() as int), sleeping),
    {
        let tail: Seq<int> = s.subrange(1, s.len() as int);
        assert forall|i: int, j: int|
            0 <= i < tail.len() && 0 <= j < sleeping.len()
            implies tail[i] != sleeping[j]
        by {
            assert(tail[i] == s[i + 1]);
            assert(0 <= i + 1 < s.len());
        }
    }

    /// Lemma: Dropping the first element of interrupted_thread_ids preserves
    /// disjointness with zombie_thread_ids.
    pub proof fn lemma_tail_disjoint_zombie(s: Seq<int>, zombie: Seq<int>)
        requires
            s.len() >= 1,
            Self::spec_seqs_disjoint(s, zombie),
        ensures
            Self::spec_seqs_disjoint(s.subrange(1, s.len() as int), zombie),
    {
        let tail: Seq<int> = s.subrange(1, s.len() as int);
        assert forall|i: int, j: int|
            0 <= i < tail.len() && 0 <= j < zombie.len()
            implies tail[i] != zombie[j]
        by {
            assert(tail[i] == s[i + 1]);
            assert(0 <= i + 1 < s.len());
        }
    }

    //==============================================================================================
    // find_thread() Lemmas
    //==============================================================================================

    /// Lemma: spec_find_thread returns None iff the thread is not in any list.
    pub proof fn lemma_find_thread_not_found(&self, tid: int)
        requires
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

    /// Refinement assumption: the original `find_thread()` implementation
    /// (which uses `iter().find(|t| t.id() == tid)` on each list in
    /// priority order: interrupted → sleeping → zombie) produces a result
    /// that matches `spec_find_thread()`.
    ///
    /// This cannot be verified within this module because:
    /// 1. Verus cannot express the reference-typed return value (`ThreadRef`).
    /// 2. The ghost sequences have no executable counterpart to iterate over.
    ///
    /// This lemma documents the semantic equivalence assumption. When Verus
    /// supports reference-typed returns or executable ghost iteration, this
    /// should be replaced with a verified implementation.
    pub proof fn lemma_find_thread_refinement_assumption(&self, tid: int)
        requires
            self.wf(),
        ensures
            // The spec search order matches the original: interrupted first.
            self.spec_has_interrupted_thread(tid) ==>
                self.spec_find_thread(tid) == Some(0int),
            // Sleeping is searched only if not found in interrupted.
            !self.spec_has_interrupted_thread(tid) && self.spec_has_sleeping_thread(tid) ==>
                self.spec_find_thread(tid) == Some(1int),
            // Zombie is searched last.
            !self.spec_has_interrupted_thread(tid)
                && !self.spec_has_sleeping_thread(tid)
                && self.spec_has_zombie_thread(tid) ==>
                self.spec_find_thread(tid) == Some(2int),
            // Not found in any list.
            !self.spec_has_thread(tid) ==>
                self.spec_find_thread(tid) == None::<int>,
    {
    }

    //==============================================================================================
    // View Equality
    //==============================================================================================

    /// Lemma: Two InterruptedProcesses with identical fields have equal views.
    pub proof fn lemma_view_equality(a: &InterruptedProcess, b: &InterruptedProcess)
        requires
            a.pid@ == b.pid@,
            a.sleeping_thread_ids@ =~= b.sleeping_thread_ids@,
            a.interrupted_thread_ids@ =~= b.interrupted_thread_ids@,
            a.zombie_thread_ids@ =~= b.zombie_thread_ids@,
        ensures
            a@ == b@,
    {
    }

    //==============================================================================================
    // Admission Time Lemmas
    //==============================================================================================

    /// Lemma: A valid admission time (per spec_admission_time_valid) satisfies
    /// the resume() precondition.
    pub proof fn lemma_valid_admission_time_satisfies_resume_precondition(
        admission_time: int,
        clock_state: int,
    )
        requires
            Self::spec_admission_time_valid(admission_time, clock_state),
        ensures
            admission_time >= 0,
    {
    }

    /// Lemma: The reason tag produced by `interrupt()` satisfies the
    /// resume reason integration obligation when the ready thread carries
    /// the same reason.
    ///
    /// This connects the `interrupt()` function's output (reason tag) to
    /// the `spec_resume_reason_integration_obligation`, proving that if the
    /// thread module's `resume()` propagates the reason faithfully, the
    /// obligation is discharged for the `Killed` variant.
    pub proof fn lemma_interrupt_reason_satisfies_obligation(
        thread_id: int,
    )
        ensures
            Self::spec_resume_reason_integration_obligation(
                thread_id,
                Self::INTERRUPT_REASON_KILLED(),
                Self::INTERRUPT_REASON_KILLED(),
            ),
    {
    }

    //==============================================================================================
    // Integration Obligation Lemmas
    //==============================================================================================

    /// Lemma: If the find_thread integration obligation is satisfied (i.e., the
    /// real implementation returns a result matching `spec_find_thread`), then
    /// the result is consistent with `spec_has_thread`.
    ///
    /// This gives integration proofs a concrete property: once the obligation is
    /// discharged, the well-formedness disjointness guarantees ensure the result
    /// is unambiguous (a thread can only appear in one list).
    pub proof fn lemma_find_thread_obligation_implies_consistency(
        &self, tid: int, real_result: Option<int>,
    )
        requires
            self.wf(),
            self.spec_find_thread_integration_obligation(tid, real_result),
        ensures
            real_result.is_some() <==> self.spec_has_thread(tid),
            real_result == Some(0int) ==> self.spec_has_interrupted_thread(tid),
            real_result == Some(1int) ==> self.spec_has_sleeping_thread(tid),
            real_result == Some(2int) ==> self.spec_has_zombie_thread(tid),
    {
    }

    /// Lemma: Under wf(), the find_thread integration obligation is uniquely
    /// determined — only one list can contain a given thread ID.
    pub proof fn lemma_find_thread_result_unique(&self, tid: int)
        requires
            self.wf(),
            self.spec_has_thread(tid),
        ensures
            // At most one of the three lists contains this thread.
            self.spec_has_interrupted_thread(tid) ==> (
                !self.spec_has_sleeping_thread(tid)
                && !self.spec_has_zombie_thread(tid)
            ),
            self.spec_has_sleeping_thread(tid) ==> (
                !self.spec_has_interrupted_thread(tid)
                && !self.spec_has_zombie_thread(tid)
            ),
            self.spec_has_zombie_thread(tid) ==> (
                !self.spec_has_interrupted_thread(tid)
                && !self.spec_has_sleeping_thread(tid)
            ),
    {
        // Follows from pairwise disjointness in wf().
        if self.spec_has_interrupted_thread(tid) {
            let i: int = choose|i: int| 0 <= i < self.interrupted_thread_ids@.len()
                && self.interrupted_thread_ids@[i] == tid;
            assert forall|j: int| 0 <= j < self.sleeping_thread_ids@.len()
                implies self.sleeping_thread_ids@[j] != tid
            by {
                assert(self.interrupted_thread_ids@[i] != self.sleeping_thread_ids@[j]);
            }
            assert forall|j: int| 0 <= j < self.zombie_thread_ids@.len()
                implies self.zombie_thread_ids@[j] != tid
            by {
                assert(self.interrupted_thread_ids@[i] != self.zombie_thread_ids@[j]);
            }
        }
        if self.spec_has_sleeping_thread(tid) {
            let i: int = choose|i: int| 0 <= i < self.sleeping_thread_ids@.len()
                && self.sleeping_thread_ids@[i] == tid;
            assert forall|j: int| 0 <= j < self.interrupted_thread_ids@.len()
                implies self.interrupted_thread_ids@[j] != tid
            by {
                assert(self.interrupted_thread_ids@[j] != self.sleeping_thread_ids@[i]);
            }
            assert forall|j: int| 0 <= j < self.zombie_thread_ids@.len()
                implies self.zombie_thread_ids@[j] != tid
            by {
                assert(self.sleeping_thread_ids@[i] != self.zombie_thread_ids@[j]);
            }
        }
        if self.spec_has_zombie_thread(tid) {
            let i: int = choose|i: int| 0 <= i < self.zombie_thread_ids@.len()
                && self.zombie_thread_ids@[i] == tid;
            assert forall|j: int| 0 <= j < self.interrupted_thread_ids@.len()
                implies self.interrupted_thread_ids@[j] != tid
            by {
                assert(self.interrupted_thread_ids@[j] != self.zombie_thread_ids@[i]);
            }
            assert forall|j: int| 0 <= j < self.sleeping_thread_ids@.len()
                implies self.sleeping_thread_ids@[j] != tid
            by {
                assert(self.sleeping_thread_ids@[j] != self.zombie_thread_ids@[i]);
            }
        }
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
        ready_times: Seq<int>,
        interrupted_ids: Seq<int>,
        sleeping_ids: Seq<int>,
        zombie_ids: Seq<int>,
    )
        requires
            ready_ids.len() >= 1,
            ready_ids.len() == ready_times.len(),
            forall|i: int| 0 <= i < ready_times.len()
                ==> #[trigger] ready_times[i] >= 0,
            RunnableProcess::spec_no_duplicates(ready_ids),
            RunnableProcess::spec_no_duplicates(interrupted_ids),
            RunnableProcess::spec_no_duplicates(sleeping_ids),
            RunnableProcess::spec_no_duplicates(zombie_ids),
            RunnableProcess::spec_seqs_disjoint(ready_ids, interrupted_ids),
            RunnableProcess::spec_seqs_disjoint(ready_ids, sleeping_ids),
            RunnableProcess::spec_seqs_disjoint(ready_ids, zombie_ids),
            RunnableProcess::spec_seqs_disjoint(interrupted_ids, sleeping_ids),
            RunnableProcess::spec_seqs_disjoint(interrupted_ids, zombie_ids),
            RunnableProcess::spec_seqs_disjoint(sleeping_ids, zombie_ids),
        ensures
            ({
                let rp: RunnableProcess = RunnableProcess {
                    pid: Ghost(pid),
                    ready_thread_ids: Ghost(ready_ids),
                    ready_admission_times: Ghost(ready_times),
                    interrupted_thread_ids: Ghost(interrupted_ids),
                    sleeping_thread_ids: Ghost(sleeping_ids),
                    zombie_thread_ids: Ghost(zombie_ids),
                };
                rp.wf()
            }),
    {
    }

    /// Projection lemma: Extracts the fields that the runnable module's boundary
    /// `InterruptedProcess` type carries (pid, interrupted_thread_ids,
    /// zombie_thread_ids) from this module's primary `InterruptedProcess`.
    ///
    /// The runnable module's boundary `InterruptedProcess` (in `runnable.rs`)
    /// has only three fields and omits `sleeping_thread_ids`. This lemma
    /// provides a concrete tuple projection `(pid, interrupted_ids, zombie_ids)`
    /// matching that boundary shape, so downstream integration proofs can
    /// map from this module's richer model to the runnable module's simpler one.
    ///
    /// **Limitation:** This lemma cannot reference the runnable module's actual
    /// `InterruptedProcess` struct (Verus modules are verified independently).
    /// It provides the projection as a tuple; the integration proof must
    /// construct the runnable module's boundary type from these values.
    pub proof fn lemma_project_to_runnable_boundary(
        ip: InterruptedProcess,
    ) -> (projection: (int, Seq<int>, Seq<int>))
        requires
            ip.wf(),
        ensures
            // Projection fields match the primary model.
            projection.0 == ip@.pid,
            projection.1 == ip@.interrupted_thread_ids,
            projection.2 == ip@.zombie_thread_ids,
            // The projected fields satisfy the runnable module's boundary wf()
            // preconditions (interrupted_thread_ids non-empty, no-duplicates,
            // disjointness between interrupted and zombie).
            projection.1.len() >= 1,
            InterruptedProcess::spec_no_duplicates(projection.1),
            InterruptedProcess::spec_no_duplicates(projection.2),
            InterruptedProcess::spec_seqs_disjoint(projection.1, projection.2),
            // The primary model carries additional sleeping_thread_ids not
            // present in the runnable module's boundary.
            InterruptedProcess::spec_seqs_disjoint(
                ip@.sleeping_thread_ids, projection.1),
            InterruptedProcess::spec_seqs_disjoint(
                ip@.sleeping_thread_ids, projection.2),
    {
        (ip@.pid, ip@.interrupted_thread_ids, ip@.zombie_thread_ids)
    }
}

} // verus!
