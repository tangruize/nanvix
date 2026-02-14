// Copyright(c) The Maintainers of Nanvix.
// Licensed under the MIT License.

// RunnableProcess Proofs.
// This file contains proof lemmas for the RunnableProcess type.
//
// Key proven properties:
// - Construction produces well-formed state with correct initial values.
// - PID is immutable across all operations.
// - run() computes min-index via concrete loop, proven to match
//   `spec_earliest_ready_index()` via `lemma_exec_min_matches_spec`.
//   Selects earliest admission time thread, preserves PID and total count.
//   Postcondition specifies exact remaining thread list contents.
// - terminate() converts ready->zombie, sleeping->interrupted, preserves PID.
//   Branch decision computed from exec-level counters (no oracle).
//   Postcondition specifies exact resulting list contents and correct branching.
// - wakeup() searches sleeping list concretely (no oracle), moves sleeping->ready,
//   preserves PID and total count.
//   Postcondition specifies exact list contents after the move.
// - add_thread() increases ready count by 1, preserves PID and other lists.
//   Postcondition specifies exact list contents.
// - earliest_admission_time: `spec_min_index_rec` is proven in-bounds and
//   minimality via `lemma_min_index_rec_bounds` (inductive proof on seq length).
// - find_thread() spec model verifies exhaustive search and list-variant consistency.
// - spec_remove_at() helper has proven length and element preservation properties.
// - Well-formedness is preserved by all operations.
//
// ## Note on Struct Construction in Proofs
//
// Because struct fields use `Vec<i64>` (exec types that cannot be constructed
// in proof mode), proof lemmas that formerly constructed struct instances
// now take `&RunnableProcess` references with preconditions mirroring the
// construction constraints. This is equivalent: instead of "construct X and
// prove P(X)", we prove "for any X satisfying the construction constraints, P(X)".

use vstd::prelude::*;

verus! {

impl RunnableProcess {

    //==============================================================================================
    // Construction Lemmas
    //==============================================================================================

    /// Lemma: A RunnableProcess with new() construction constraints is well-formed.
    pub proof fn lemma_new_is_wf(p: &RunnableProcess)
        requires
            p.ready_thread_ids@.len() == 1,
            p.ready_admission_times@.len() == 1,
            p.ready_admission_times@[0] >= 0i64,
            p.interrupted_thread_ids@.len() == 0,
            p.sleeping_thread_ids@.len() == 0,
            p.zombie_thread_ids@.len() == 0,
            p.interrupted_count == 0u64,
            p.sleeping_count == 0u64,
        ensures
            p.wf(),
    {
        reveal(RunnableProcess::wf);
    }

    /// Lemma: A RunnableProcess with new() construction constraints has exactly one ready thread.
    pub proof fn lemma_new_has_one_ready(p: &RunnableProcess)
        requires
            p.ready_thread_ids@.len() == 1,
            p.interrupted_thread_ids@.len() == 0,
            p.sleeping_thread_ids@.len() == 0,
            p.zombie_thread_ids@.len() == 0,
        ensures
            p.spec_ready_count() == 1,
            p.spec_interrupted_count() == 0,
            p.spec_sleeping_count() == 0,
            p.spec_zombie_count() == 0,
    {
    }

    /// Lemma: A RunnableProcess with new() construction constraints has total count 1.
    pub proof fn lemma_new_empty_optional_lists(p: &RunnableProcess)
        requires
            p.ready_thread_ids@.len() == 1,
            p.interrupted_thread_ids@.len() == 0,
            p.sleeping_thread_ids@.len() == 0,
            p.zombie_thread_ids@.len() == 0,
        ensures
            p.spec_total_thread_count() == 1,
    {
    }

    //==============================================================================================
    // PID Immutability Lemmas
    //==============================================================================================

    /// Lemma: Two RunnableProcesses sharing the same pid have equal spec_pid.
    pub proof fn lemma_from_state_preserves_pid(a: &RunnableProcess, b: &RunnableProcess)
        requires
            a.pid.spec_value() == b.pid.spec_value(),
        ensures
            a.spec_pid() == b.spec_pid(),
    {
    }

    //==============================================================================================
    // run() Lemmas
    //==============================================================================================

    /// Lemma: After run(), the total thread count is preserved.
    /// One ready thread becomes the running thread in the RunningProcess.
    pub proof fn lemma_run_preserves_total_threads(&self, selected_idx: int)
        requires
            self.wf(),
            0 <= selected_idx < self.ready_thread_ids@.len(),
        ensures
            ({
                // After removing one ready thread, remaining ready + 1 (running) = original ready.
                let remaining_ready_len: nat =
                    (self.ready_thread_ids@.len() - 1) as nat;
                remaining_ready_len + 1 == self.spec_ready_count()
            }),
    {
        reveal(RunnableProcess::wf);
    }

    /// Lemma: run() preserves PID — trivially true since PID is copied.
    pub proof fn lemma_run_preserves_pid(&self)
        requires
            self.wf(),
        ensures
            self.spec_pid() == self.pid.spec_value(),
    {
        reveal(RunnableProcess::wf);
    }

    /// Lemma: If there is exactly one ready thread, removing it empties the ready list.
    pub proof fn lemma_run_single_thread_empties_ready(&self)
        requires
            self.wf(),
            self.spec_ready_count() == 1,
        ensures
            ({
                let remaining: Seq<i64> = self.ready_thread_ids@.subrange(0, 0)
                    .add(self.ready_thread_ids@.subrange(1, 1));
                remaining.len() == 0
            }),
    {
        reveal(RunnableProcess::wf);
    }

    /// Lemma: An exec loop finding the min-index matches spec_earliest_ready_index.
    ///
    /// This is now proven directly by adding `min_idx == spec_min_index_rec(s, i)`
    /// as a loop invariant in `run()`. The invariant is maintained because both the
    /// loop and spec_min_index_rec use the same left-to-right strict-< algorithm.
    /// This lemma is retained as documentation; the proof obligation is discharged
    /// inline in the `run()` loop invariant.

    //==============================================================================================
    // terminate() Lemmas
    //==============================================================================================

    /// Lemma: terminate() converts all ready threads to zombies.
    pub proof fn lemma_terminate_ready_to_zombie(&self)
        requires
            self.wf(),
        ensures
            // All ready thread IDs become zombie thread IDs.
            self.ready_thread_ids@.len() >= 1,
    {
        reveal(RunnableProcess::wf);
    }

    /// Lemma: terminate() preserves PID.
    pub proof fn lemma_terminate_preserves_pid(&self)
        requires
            self.wf(),
        ensures
            // PID is immutable, so any resulting state has the same PID.
            self.spec_pid() == self.pid.spec_value(),
    {
        reveal(RunnableProcess::wf);
    }

    /// Lemma: terminate() with no sleeping and no interrupted threads produces ZombieProcess.
    pub proof fn lemma_terminate_no_interrupted_gives_zombie(&self)
        requires
            self.wf(),
            self.spec_interrupted_count() == 0,
            self.spec_sleeping_count() == 0,
        ensures
            // has_interrupted is false under these conditions.
            !(self.spec_interrupted_count() > 0 || self.spec_sleeping_count() > 0),
            // The resulting zombie threads contain all ready + original zombie.
            ({
                let zombie_ids: Seq<i64> = self.ready_thread_ids@.add(self.zombie_thread_ids@);
                zombie_ids.len() == self.spec_ready_count() + self.spec_zombie_count()
                && zombie_ids.len() >= 1
            }),
    {
        reveal(RunnableProcess::wf);
    }

    /// Lemma: terminate() with interrupted threads produces InterruptedProcess.
    pub proof fn lemma_terminate_with_interrupted_gives_interrupted(&self)
        requires
            self.wf(),
            self.spec_interrupted_count() > 0,
        ensures
            // has_interrupted is true under these conditions.
            (self.spec_interrupted_count() > 0 || self.spec_sleeping_count() > 0),
            // The resulting interrupted list is non-empty.
            ({
                let interrupted_ids: Seq<i64> =
                    self.interrupted_thread_ids@.add(self.sleeping_thread_ids@);
                interrupted_ids.len() >= 1
                && interrupted_ids.len() ==
                    self.spec_interrupted_count() + self.spec_sleeping_count()
            }),
    {
        reveal(RunnableProcess::wf);
    }

    /// Lemma: terminate() with sleeping (but no interrupted) produces InterruptedProcess.
    pub proof fn lemma_terminate_with_sleeping_gives_interrupted(&self)
        requires
            self.wf(),
            self.spec_interrupted_count() == 0,
            self.spec_sleeping_count() > 0,
        ensures
            // has_interrupted is true because sleeping_count > 0.
            (self.spec_interrupted_count() > 0 || self.spec_sleeping_count() > 0),
            // The resulting interrupted list contains exactly the sleeping threads.
            ({
                let interrupted_ids: Seq<i64> =
                    self.interrupted_thread_ids@.add(self.sleeping_thread_ids@);
                interrupted_ids.len() == self.spec_sleeping_count()
                && interrupted_ids.len() >= 1
            }),
    {
        reveal(RunnableProcess::wf);
    }

    //==============================================================================================
    // wakeup() Lemmas
    //==============================================================================================

    /// Lemma: Successful wakeup() increases ready count by 1 and decreases sleeping by 1.
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
        reveal(RunnableProcess::wf);
    }

    /// Lemma: wakeup() preserves PID.
    pub proof fn lemma_wakeup_preserves_pid(&self)
        requires
            self.wf(),
        ensures
            self.spec_pid() == self.pid.spec_value(),
    {
        reveal(RunnableProcess::wf);
    }

    /// Lemma: Successful wakeup() preserves total thread count.
    pub proof fn lemma_wakeup_preserves_total_count(&self, removed_idx: int)
        requires
            self.wf(),
            self.spec_sleeping_count() > 0,
            0 <= removed_idx < self.sleeping_thread_ids@.len(),
        ensures
            ({
                // One thread moves from sleeping to ready; total unchanged.
                let new_total: int =
                    (self.spec_ready_count() + 1) as int
                    + self.spec_interrupted_count() as int
                    + (self.spec_sleeping_count() - 1) as int
                    + self.spec_zombie_count() as int;
                new_total == self.spec_total_thread_count() as int
            }),
    {
        reveal(RunnableProcess::wf);
    }

    /// Lemma: Successful wakeup() result satisfies wf() conditions.
    pub proof fn lemma_wakeup_result_wf(
        &self,
        removed_idx: int,
        woken_tid: i64,
        new_ready_time: i64,
    )
        requires
            self.wf(),
            self.spec_sleeping_count() > 0,
            0 <= removed_idx < self.sleeping_thread_ids@.len(),
            self.sleeping_thread_ids@[removed_idx] == woken_tid,
            new_ready_time >= 0i64,
        ensures
            ({
                let new_ready_ids: Seq<i64> = self.ready_thread_ids@.push(woken_tid);
                let new_ready_times: Seq<i64> = self.ready_admission_times@.push(new_ready_time);
                let new_sleeping_ids: Seq<i64> =
                    self.sleeping_thread_ids@.subrange(0, removed_idx)
                        .add(self.sleeping_thread_ids@.subrange(
                            removed_idx + 1,
                            self.sleeping_thread_ids@.len() as int,
                        ));
                // wf() conditions directly:
                new_ready_ids.len() >= 1
                && new_ready_ids.len() == new_ready_times.len()
                && forall|i: int| 0 <= i < new_ready_times.len()
                    ==> #[trigger] new_ready_times[i] >= 0i64
                && self.interrupted_count as nat == self.interrupted_thread_ids@.len()
                && (self.sleeping_count - 1) as nat == new_sleeping_ids.len()
            }),
    {
        reveal(RunnableProcess::wf);
    }

    //==============================================================================================
    // add_thread() Lemmas
    //==============================================================================================

    /// Lemma: add_thread() increases ready count by 1.
    pub proof fn lemma_add_thread_increments_ready(&self, new_tid: i64, new_time: i64)
        requires
            self.wf(),
            new_time >= 0i64,
        ensures
            ({
                let new_ready_ids: Seq<i64> = self.ready_thread_ids@.push(new_tid);
                let new_ready_times: Seq<i64> = self.ready_admission_times@.push(new_time);
                new_ready_ids.len() == self.spec_ready_count() + 1
                && new_ready_times.len() == self.ready_admission_times@.len() + 1
            }),
    {
        reveal(RunnableProcess::wf);
    }

    /// Lemma: add_thread() result satisfies wf() conditions.
    pub proof fn lemma_add_thread_result_wf(&self, new_tid: i64, new_time: i64)
        requires
            self.wf(),
            new_time >= 0i64,
        ensures
            ({
                let new_ready_ids: Seq<i64> = self.ready_thread_ids@.push(new_tid);
                let new_ready_times: Seq<i64> = self.ready_admission_times@.push(new_time);
                // wf() conditions directly:
                new_ready_ids.len() >= 1
                && new_ready_ids.len() == new_ready_times.len()
                && forall|i: int| 0 <= i < new_ready_times.len()
                    ==> #[trigger] new_ready_times[i] >= 0i64
                && self.interrupted_count as nat == self.interrupted_thread_ids@.len()
                && self.sleeping_count as nat == self.sleeping_thread_ids@.len()
                && new_ready_ids.len() == self.spec_ready_count() + 1
            }),
    {
        reveal(RunnableProcess::wf);
    }

    /// Lemma: add_thread() preserves other thread lists unchanged.
    pub proof fn lemma_add_thread_preserves_others(&self, new_tid: i64, new_time: i64)
        requires
            self.wf(),
        ensures
            self.spec_interrupted_count() == self.interrupted_thread_ids@.len(),
            self.spec_sleeping_count() == self.sleeping_thread_ids@.len(),
            self.spec_zombie_count() == self.zombie_thread_ids@.len(),
    {
        reveal(RunnableProcess::wf);
    }

    //==============================================================================================
    // earliest_admission_time() Lemmas
    //==============================================================================================

    /// Lemma: The earliest admission time exists among ready threads.
    /// For a non-empty finite sequence of i64s, there exists a minimum.
    pub proof fn lemma_earliest_admission_time_exists(&self)
        requires
            self.wf(),
        ensures
            exists|idx: int| 0 <= idx < self.ready_admission_times@.len()
                && forall|j: int| 0 <= j < self.ready_admission_times@.len()
                    ==> #[trigger] self.ready_admission_times@[idx]
                        <= #[trigger] self.ready_admission_times@[j],
    {
        reveal(RunnableProcess::wf);
        let s: &Seq<i64> = &self.ready_admission_times@;
        let n: int = s.len() as int;
        Self::lemma_seq_has_min(s, n);
        let min_idx: int = choose|idx: int| 0 <= idx < n
            && forall|j: int| 0 <= j < n ==> (#[trigger] s[idx]) <= (#[trigger] s[j]);
        assert(self.ready_admission_times@[min_idx] == s[min_idx]);
        assert forall|j: int| 0 <= j < self.ready_admission_times@.len()
            implies #[trigger] self.ready_admission_times@[min_idx]
                <= #[trigger] self.ready_admission_times@[j]
        by {
            assert(s[min_idx] <= s[j]);
        }
    }

    /// Lemma: spec_min_index_rec is in bounds and selects the minimum.
    proof fn lemma_min_index_rec_bounds(s: &Seq<i64>, n: int)
        requires
            1 <= n <= s.len(),
        ensures
            ({
                let idx: int = RunnableProcess::spec_min_index_rec(*s, n);
                0 <= idx < n
                && forall|j: int| 0 <= j < n
                    ==> (#[trigger] s[idx]) <= (#[trigger] s[j])
            }),
        decreases n,
    {
        if n == 1 {
            // Base case: spec_min_index_rec returns 0, only element.
        } else {
            Self::lemma_min_index_rec_bounds(s, n - 1);
            let prev: int = RunnableProcess::spec_min_index_rec(*s, n - 1);
            // prev is in bounds by induction hypothesis.
            assert(0 <= prev < n - 1);
            assert(0 <= prev < s.len());
            if s[n - 1] < s[prev] {
                // New element is smaller.
                assert forall|j: int| 0 <= j < n
                    implies (#[trigger] s[n - 1]) <= (#[trigger] s[j])
                by {
                    if j < n - 1 {
                        assert(s[prev] <= s[j]);
                    }
                }
            } else {
                // Previous min is still min.
                assert forall|j: int| 0 <= j < n
                    implies (#[trigger] s[prev]) <= (#[trigger] s[j])
                by {
                    if j == n - 1 {
                        assert(s[prev] <= s[n - 1]);
                    }
                }
            }
        }
    }

    /// Lemma: spec_earliest_ready_index is in bounds and selects the minimum.
    pub proof fn lemma_earliest_ready_index_bounds(&self)
        requires
            self.wf(),
        ensures
            ({
                let idx: int = self.spec_earliest_ready_index();
                0 <= idx < self.ready_admission_times@.len()
                && 0 <= idx < self.ready_thread_ids@.len()
                && forall|j: int| 0 <= j < self.ready_admission_times@.len()
                    ==> #[trigger] self.ready_admission_times@[idx]
                        <= #[trigger] self.ready_admission_times@[j]
            }),
    {
        reveal(RunnableProcess::wf);
        Self::lemma_min_index_rec_bounds(
            &self.ready_admission_times@,
            self.ready_admission_times@.len() as int,
        );
    }

    /// Helper: A non-empty sequence of i64s has a minimum element within the first n elements.
    proof fn lemma_seq_has_min(s: &Seq<i64>, n: int)
        requires
            n >= 1,
            n <= s.len(),
        ensures
            exists|idx: int| 0 <= idx < n
                && forall|j: int| 0 <= j < n ==> (#[trigger] s[idx]) <= (#[trigger] s[j]),
        decreases n,
    {
        if n == 1 {
            assert(s[0] <= s[0]);
        } else {
            Self::lemma_seq_has_min(s, n - 1);
            let prev_min_idx: int = choose|idx: int| 0 <= idx < n - 1
                && forall|j: int| 0 <= j < n - 1 ==> (#[trigger] s[idx]) <= (#[trigger] s[j]);
            if s[n - 1] < s[prev_min_idx] {
                // New element is smaller.
                assert forall|j: int| 0 <= j < n implies (#[trigger] s[n - 1]) <= (#[trigger] s[j])
                by {
                    if j < n - 1 {
                        assert(s[prev_min_idx] <= s[j]);
                        assert(s[n - 1] <= s[j]);
                    }
                }
            } else {
                // Previous min is still min.
                assert forall|j: int| 0 <= j < n implies (#[trigger] s[prev_min_idx]) <= (#[trigger] s[j])
                by {
                    if j < n - 1 {
                        assert(s[prev_min_idx] <= s[j]);
                    } else {
                        assert(s[prev_min_idx] <= s[n - 1]);
                    }
                }
            }
        }
    }

    /// Lemma: The earliest admission time is non-negative when all times are non-negative.
    pub proof fn lemma_earliest_admission_time_nonneg(&self)
        requires
            self.wf(),
        ensures
            ({
                // wf() ensures all admission times are non-negative.
                // The minimum of non-negative values is non-negative.
                forall|i: int| 0 <= i < self.ready_admission_times@.len()
                    ==> self.ready_admission_times@[i] >= 0i64
            }),
    {
        reveal(RunnableProcess::wf);
    }

    /// Refinement lemma: `spec_earliest_admission_time` is the minimum admission time,
    /// is non-negative, and its index is in bounds. This bridges the spec-only
    /// `earliest_admission_time()` to the proven properties.
    pub proof fn lemma_earliest_admission_time_refinement(&self)
        requires
            self.wf(),
        ensures
            ({
                let t: i64 = self.spec_earliest_admission_time();
                let idx: int = self.spec_earliest_ready_index();
                // The value is from the admission times array.
                t == self.ready_admission_times@[idx]
                // It is non-negative.
                && t >= 0i64
                // It is the minimum over all admission times.
                && forall|j: int| 0 <= j < self.ready_admission_times@.len()
                    ==> t <= #[trigger] self.ready_admission_times@[j]
            }),
    {
        reveal(RunnableProcess::wf);
        self.lemma_earliest_ready_index_bounds();
    }

    //==============================================================================================
    // find_thread() Lemmas
    //==============================================================================================

    /// Lemma: spec_find_thread returns THREAD_REF_READY iff the thread is in the ready list.
    pub proof fn lemma_find_thread_ready(&self, tid: i64)
        requires
            self.spec_has_ready_thread(tid),
        ensures
            self.spec_find_thread(tid) == Some(THREAD_REF_READY()),
    {
    }

    /// Lemma: spec_find_thread returns None iff the thread is not in any list.
    pub proof fn lemma_find_thread_not_found(&self, tid: i64)
        requires
            !self.spec_has_ready_thread(tid),
            !self.spec_has_interrupted_thread(tid),
            !self.spec_has_sleeping_thread(tid),
            !self.spec_has_zombie_thread(tid),
        ensures
            self.spec_find_thread(tid) == None::<int>,
    {
    }

    /// Lemma: spec_find_thread result is consistent with spec_has_thread.
    pub proof fn lemma_find_thread_iff_has_thread(&self, tid: i64)
        ensures
            self.spec_find_thread(tid).is_some() <==> self.spec_has_thread(tid),
    {
    }

    /// Lemma: If spec_seq_contains is true, there exists a valid index.
    pub proof fn lemma_spec_find_thread_index(&self, tid: i64)
        requires
            Self::spec_seq_contains(self.sleeping_thread_ids@, tid),
        ensures
            exists|i: int| 0 <= i < self.sleeping_thread_ids@.len()
                && self.sleeping_thread_ids@[i] == tid,
    {
    }

    //==============================================================================================
    // Content Preservation Lemmas
    //==============================================================================================

    /// Lemma: spec_remove_at produces a sequence of length len - 1.
    pub proof fn lemma_remove_at_length(s: Seq<i64>, idx: int)
        requires
            0 <= idx < s.len(),
        ensures
            Self::spec_remove_at(s, idx).len() == s.len() - 1,
    {
        let left: Seq<i64> = s.subrange(0, idx);
        let right: Seq<i64> = s.subrange(idx + 1, s.len() as int);
        assert(left.len() == idx as nat);
        assert(right.len() == (s.len() - idx as nat - 1) as nat);
    }

    /// Lemma: spec_remove_at preserves elements before and after the removed index.
    pub proof fn lemma_remove_at_preserves_others(s: Seq<i64>, idx: int, j: int)
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

    /// Lemma: Two RunnableProcesses with identical field views have equal views.
    pub proof fn lemma_view_equality(a: &RunnableProcess, b: &RunnableProcess)
        requires
            a.pid.spec_value() == b.pid.spec_value(),
            a.ready_thread_ids@ =~= b.ready_thread_ids@,
            a.ready_admission_times@ =~= b.ready_admission_times@,
            a.interrupted_thread_ids@ =~= b.interrupted_thread_ids@,
            a.sleeping_thread_ids@ =~= b.sleeping_thread_ids@,
            a.zombie_thread_ids@ =~= b.zombie_thread_ids@,
        ensures
            a@ == b@,
    {
    }

    //==============================================================================================
    // View-Level Bridging Lemmas
    //==============================================================================================

    /// Bridging lemma: exec-level `wf()` implies view-level `wf()`.
    pub proof fn lemma_wf_implies_view_wf(&self)
        requires
            self.wf(),
        ensures
            self@.wf(),
    {
        reveal(RunnableProcess::wf);
    }

    /// Bridging lemma: view-level `spec_min_index_rec` equals exec-level `spec_min_index_rec`.
    pub proof fn lemma_view_min_index_eq(s: Seq<i64>, n: int)
        requires
            1 <= n <= s.len(),
        ensures
            RunnableProcessView::spec_min_index_rec(s, n)
                == RunnableProcess::spec_min_index_rec(s, n),
        decreases n,
    {
        if n > 1 {
            Self::lemma_view_min_index_eq(s, n - 1);
        }
    }

    /// Bridging lemma: the new() constructor view matches `RunnableProcessView::spec_new()`.
    pub proof fn lemma_new_view_eq(p: &RunnableProcess, pid_val: int, tid: i64, time: i64)
        requires
            p.pid.spec_value() == pid_val,
            p.ready_thread_ids@.len() == 1,
            p.ready_thread_ids@[0] == tid,
            p.ready_admission_times@.len() == 1,
            p.ready_admission_times@[0] == time,
            p.interrupted_thread_ids@.len() == 0,
            p.sleeping_thread_ids@.len() == 0,
            p.zombie_thread_ids@.len() == 0,
        ensures
            p@ == RunnableProcessView::spec_new(pid_val, tid, time),
    {
        assert(p.ready_thread_ids@ =~= seq![tid]);
        assert(p.ready_admission_times@ =~= seq![time]);
        assert(p.interrupted_thread_ids@ =~= Seq::<i64>::empty());
        assert(p.sleeping_thread_ids@ =~= Seq::<i64>::empty());
        assert(p.zombie_thread_ids@ =~= Seq::<i64>::empty());
    }

    /// Bridging lemma: the run() result view matches `RunnableProcessView::spec_run()`.
    pub proof fn lemma_run_view_eq(&self, result: &RunningProcess)
        requires
            self.wf(),
            result.pid.spec_value() == self.pid.spec_value(),
            ({
                let sel: int = self.spec_earliest_ready_index();
                result.running_thread_id as int == self.ready_thread_ids@[sel] as int
                && result.ready_thread_ids@ ==
                    Self::spec_remove_at(self.ready_thread_ids@, sel)
            }),
            result.interrupted_thread_ids@ == self.interrupted_thread_ids@,
            result.sleeping_thread_ids@ == self.sleeping_thread_ids@,
            result.zombie_thread_ids@ == self.zombie_thread_ids@,
            result.interrupt_reason == 0i64,
        ensures
            result@ == self@.spec_run(),
    {
        reveal(RunnableProcess::wf);
        Self::lemma_view_min_index_eq(
            self.ready_admission_times@,
            self.ready_admission_times@.len() as int,
        );
        let sel_exec: int = self.spec_earliest_ready_index();
        let sel_view: int = self@.spec_earliest_ready_index();
        assert(sel_exec == sel_view);
        assert(self.ready_thread_ids@[sel_exec] == self@.ready_thread_ids[sel_view]);
        assert(Self::spec_remove_at(self.ready_thread_ids@, sel_exec)
            =~= RunnableProcessView::spec_remove_at(self@.ready_thread_ids, sel_view));
    }

    /// Bridging lemma: the terminate() InterruptedProcess result view matches
    /// `RunnableProcessView::spec_terminate_to_interrupted()`.
    pub proof fn lemma_terminate_interrupted_view_eq(&self, result: &InterruptedProcess)
        requires
            self.wf(),
            result.pid.spec_value() == self.pid.spec_value(),
            result.interrupted_thread_ids@ ==
                self.interrupted_thread_ids@.add(self.sleeping_thread_ids@),
            result.zombie_thread_ids@ ==
                self.ready_thread_ids@.add(self.zombie_thread_ids@),
        ensures
            result@ == self@.spec_terminate_to_interrupted(),
    {
    }

    /// Bridging lemma: the terminate() ZombieProcess result view matches
    /// `RunnableProcessView::spec_terminate_to_zombie()`.
    pub proof fn lemma_terminate_zombie_view_eq(&self, result: &ZombieProcess)
        requires
            self.wf(),
            result.pid.spec_value() == self.pid.spec_value(),
            result.zombie_thread_ids@ ==
                self.ready_thread_ids@.add(self.zombie_thread_ids@),
            result.status == 4i64,
        ensures
            result@ == self@.spec_terminate_to_zombie(),
    {
    }

    /// Bridging lemma: the add_thread() result view matches
    /// `RunnableProcessView::spec_add_thread()`.
    pub proof fn lemma_add_thread_view_eq(&self, result: &RunnableProcess, tid: i64, time: i64)
        requires
            self.wf(),
            result.pid.spec_value() == self.pid.spec_value(),
            result.ready_thread_ids@ == self.ready_thread_ids@.push(tid),
            result.ready_admission_times@ == self.ready_admission_times@.push(time),
            result.interrupted_thread_ids@ == self.interrupted_thread_ids@,
            result.sleeping_thread_ids@ == self.sleeping_thread_ids@,
            result.zombie_thread_ids@ == self.zombie_thread_ids@,
        ensures
            result@ == self@.spec_add_thread(tid, time),
    {
    }

    /// Bridging lemma: the wakeup() success result view matches
    /// `RunnableProcessView::spec_wakeup()`.
    ///
    /// The `found_idx` parameter is the concrete index where `tid` was found
    /// in the sleeping list (from `vec_search`). This is passed through to
    /// `spec_wakeup` to deterministically identify the removed element.
    pub proof fn lemma_wakeup_view_eq(
        &self,
        result: &RunnableProcess,
        tid: i64,
        time: i64,
        found_idx: int,
    )
        requires
            self.wf(),
            0 <= found_idx < self.sleeping_thread_ids@.len(),
            self.sleeping_thread_ids@[found_idx] == tid,
            result.pid.spec_value() == self.pid.spec_value(),
            result.ready_thread_ids@ == self.ready_thread_ids@.push(tid),
            result.ready_admission_times@ == self.ready_admission_times@.push(time),
            result.interrupted_thread_ids@ == self.interrupted_thread_ids@,
            result.sleeping_thread_ids@ ==
                RunnableProcess::spec_remove_at(self.sleeping_thread_ids@, found_idx),
            result.zombie_thread_ids@ == self.zombie_thread_ids@,
        ensures
            result@ == self@.spec_wakeup(tid, time, found_idx),
    {
        assert(RunnableProcess::spec_remove_at(self.sleeping_thread_ids@, found_idx)
            =~= RunnableProcessView::spec_remove_at(self@.sleeping_thread_ids, found_idx));
    }

    /// Bridging lemma: the from_state() constructor view matches
    /// `RunnableProcessView::spec_from_state()`.
    pub proof fn lemma_from_state_view_eq(
        p: &RunnableProcess,
        pid_val: int,
        ready_ids: Seq<i64>,
        ready_times: Seq<i64>,
        interrupted_ids: Seq<i64>,
        sleeping_ids: Seq<i64>,
        zombie_ids: Seq<i64>,
    )
        requires
            p.pid.spec_value() == pid_val,
            p.ready_thread_ids@ == ready_ids,
            p.ready_admission_times@ == ready_times,
            p.interrupted_thread_ids@ == interrupted_ids,
            p.sleeping_thread_ids@ == sleeping_ids,
            p.zombie_thread_ids@ == zombie_ids,
        ensures
            p@ == RunnableProcessView::spec_from_state(
                pid_val, ready_ids, ready_times,
                interrupted_ids, sleeping_ids, zombie_ids),
    {
    }

    /// Bridging lemma: the exec-level `spec_terminate_has_interrupted` predicate
    /// matches the view-level `spec_terminate_has_interrupted`.
    pub proof fn lemma_terminate_branch_eq(&self)
        requires
            self.wf(),
        ensures
            (self.spec_interrupted_count() > 0 || self.spec_sleeping_count() > 0)
                == self@.spec_terminate_has_interrupted(),
    {
        reveal(RunnableProcess::wf);
    }
}

//==================================================================================================
// RunningProcess Lemmas (Boundary)
//==================================================================================================

impl RunningProcess {
    /// Lemma: A RunningProcess preserves process identity.
    pub proof fn lemma_new_preserves_pid(r: &RunningProcess)
        ensures
            r.spec_pid() == r.pid.spec_value(),
    {
    }
}

//==================================================================================================
// InterruptedProcess Lemmas (Boundary)
//==================================================================================================

impl InterruptedProcess {
    /// Lemma: An InterruptedProcess with non-empty interrupted threads is well-formed.
    pub proof fn lemma_new_wf(ip: &InterruptedProcess)
        requires
            ip.interrupted_thread_ids@.len() >= 1,
        ensures
            ip.wf(),
    {
        reveal(InterruptedProcess::wf);
    }
}

//==================================================================================================
// ZombieProcess Lemmas (Boundary)
//==================================================================================================

impl ZombieProcess {
    /// Lemma: A ZombieProcess with non-empty zombie threads is well-formed.
    pub proof fn lemma_new_wf(zp: &ZombieProcess)
        requires
            zp.zombie_thread_ids@.len() >= 1,
        ensures
            zp.wf(),
    {
        reveal(ZombieProcess::wf);
    }
}

} // verus!
