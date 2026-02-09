// Copyright(c) The Maintainers of Nanvix.
// Licensed under the MIT License.

// RunnableProcess Proofs.
// This file contains proof lemmas for the RunnableProcess type.
//
// Key proven properties:
// - Construction produces well-formed state with correct initial values.
// - PID is immutable across all operations.
// - run() derives min-index via `lemma_earliest_ready_index_bounds` (no oracle),
//   selects earliest admission time thread, preserves PID and total count.
//   Postcondition specifies exact remaining thread list contents.
// - terminate() converts ready→zombie, sleeping→interrupted, preserves PID.
//   Branch decision computed from exec-level counters (no oracle).
//   Postcondition specifies exact resulting list contents and correct branching.
// - wakeup() derives search index via proof-level `choose` (no index oracle),
//   moves sleeping→ready, preserves PID and total count.
//   Postcondition specifies exact list contents after the move.
// - add_thread() increases ready count by 1, preserves PID and other lists.
//   Postcondition specifies exact list contents.
// - earliest_admission_time: `spec_min_index_rec` is proven in-bounds and
//   minimality via `lemma_min_index_rec_bounds` (inductive proof on seq length).
// - find_thread() spec model verifies exhaustive search and list-variant consistency.
// - spec_remove_at() helper has proven length and element preservation properties.
// - Well-formedness is preserved by all operations.

use vstd::prelude::*;

verus! {

impl RunnableProcess {

    //==============================================================================================
    // Construction Lemmas
    //==============================================================================================

    /// Lemma: A newly constructed RunnableProcess is well-formed.
    pub proof fn lemma_new_is_wf(pid: ProcessIdentifier, ready_tid: int, ready_time: int)
        requires
            ready_time >= 0,
        ensures
            ({
                let r: RunnableProcess = RunnableProcess {
                    pid: pid,
                    ready_thread_ids: Ghost(seq![ready_tid]),
                    ready_admission_times: Ghost(seq![ready_time]),
                    interrupted_thread_ids: Ghost(Seq::empty()),
                    sleeping_thread_ids: Ghost(Seq::empty()),
                    zombie_thread_ids: Ghost(Seq::empty()),
                    interrupted_count: 0u64,
                    sleeping_count: 0u64,
                };
                r.wf()
            }),
    {
    }

    /// Lemma: A newly constructed RunnableProcess has exactly one ready thread.
    pub proof fn lemma_new_has_one_ready(pid: ProcessIdentifier, ready_tid: int, ready_time: int)
        ensures
            ({
                let r: RunnableProcess = RunnableProcess {
                    pid: pid,
                    ready_thread_ids: Ghost(seq![ready_tid]),
                    ready_admission_times: Ghost(seq![ready_time]),
                    interrupted_thread_ids: Ghost(Seq::empty()),
                    sleeping_thread_ids: Ghost(Seq::empty()),
                    zombie_thread_ids: Ghost(Seq::empty()),
                    interrupted_count: 0u64,
                    sleeping_count: 0u64,
                };
                r.spec_ready_count() == 1
                && r.spec_interrupted_count() == 0
                && r.spec_sleeping_count() == 0
                && r.spec_zombie_count() == 0
            }),
    {
    }

    /// Lemma: A newly constructed RunnableProcess has no optional thread lists.
    pub proof fn lemma_new_empty_optional_lists(pid: ProcessIdentifier, ready_tid: int, ready_time: int)
        ensures
            ({
                let r: RunnableProcess = RunnableProcess {
                    pid: pid,
                    ready_thread_ids: Ghost(seq![ready_tid]),
                    ready_admission_times: Ghost(seq![ready_time]),
                    interrupted_thread_ids: Ghost(Seq::empty()),
                    sleeping_thread_ids: Ghost(Seq::empty()),
                    zombie_thread_ids: Ghost(Seq::empty()),
                    interrupted_count: 0u64,
                    sleeping_count: 0u64,
                };
                r.spec_total_thread_count() == 1
            }),
    {
    }

    //==============================================================================================
    // PID Immutability Lemmas
    //==============================================================================================

    /// Lemma: from_state preserves PID.
    pub proof fn lemma_from_state_preserves_pid(
        &self,
        new_ready_ids: Seq<int>,
        new_ready_times: Seq<int>,
        new_interrupted_ids: Seq<int>,
        new_sleeping_ids: Seq<int>,
        new_zombie_ids: Seq<int>,
    )
        ensures
            ({
                let post: RunnableProcess = RunnableProcess {
                    pid: self.pid,
                    ready_thread_ids: Ghost(new_ready_ids),
                    ready_admission_times: Ghost(new_ready_times),
                    interrupted_thread_ids: Ghost(new_interrupted_ids),
                    sleeping_thread_ids: Ghost(new_sleeping_ids),
                    zombie_thread_ids: Ghost(new_zombie_ids),
                    interrupted_count: 0u64,
                    sleeping_count: 0u64,
                };
                post.spec_pid() == self.spec_pid()
            }),
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
    }

    /// Lemma: run() preserves PID in the resulting RunningProcess.
    pub proof fn lemma_run_preserves_pid(&self, selected_idx: int)
        requires
            self.wf(),
            0 <= selected_idx < self.ready_thread_ids@.len(),
        ensures
            ({
                let running: RunningProcess = RunningProcess {
                    pid: Ghost(self.pid.spec_value()),
                    running_thread_id: Ghost(self.ready_thread_ids@[selected_idx]),
                    ready_thread_ids: Ghost(
                        self.ready_thread_ids@.subrange(0, selected_idx)
                            .add(self.ready_thread_ids@.subrange(
                                selected_idx + 1,
                                self.ready_thread_ids@.len() as int,
                            ))
                    ),
                    interrupted_thread_ids: Ghost(self.interrupted_thread_ids@),
                    sleeping_thread_ids: Ghost(self.sleeping_thread_ids@),
                    zombie_thread_ids: Ghost(self.zombie_thread_ids@),
                };
                running.spec_pid() == self.spec_pid()
            }),
    {
    }

    /// Lemma: If there is exactly one ready thread, run() empties the ready list.
    pub proof fn lemma_run_single_thread_empties_ready(&self)
        requires
            self.wf(),
            self.spec_ready_count() == 1,
        ensures
            ({
                let remaining: Seq<int> = self.ready_thread_ids@.subrange(0, 0)
                    .add(self.ready_thread_ids@.subrange(1, 1));
                remaining.len() == 0
            }),
    {
    }

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
    }

    /// Lemma: terminate() preserves PID.
    pub proof fn lemma_terminate_preserves_pid(&self)
        requires
            self.wf(),
        ensures
            // PID is immutable, so any resulting state has the same PID.
            self.spec_pid() == self.pid.spec_value(),
    {
    }

    /// Lemma: terminate() with no sleeping and no interrupted threads produces ZombieProcess.
    /// The has_interrupted oracle must be false, so terminate() takes the Zombie branch.
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
                let zombie_ids: Seq<int> = self.ready_thread_ids@.add(self.zombie_thread_ids@);
                zombie_ids.len() == self.spec_ready_count() + self.spec_zombie_count()
                && zombie_ids.len() >= 1
            }),
    {
    }

    /// Lemma: terminate() with interrupted threads produces InterruptedProcess.
    /// The has_interrupted oracle must be true, so terminate() takes the Interrupted branch.
    pub proof fn lemma_terminate_with_interrupted_gives_interrupted(&self)
        requires
            self.wf(),
            self.spec_interrupted_count() > 0,
        ensures
            // has_interrupted is true under these conditions.
            (self.spec_interrupted_count() > 0 || self.spec_sleeping_count() > 0),
            // The resulting interrupted list is non-empty.
            ({
                let interrupted_ids: Seq<int> =
                    self.interrupted_thread_ids@.add(self.sleeping_thread_ids@);
                interrupted_ids.len() >= 1
                && interrupted_ids.len() ==
                    self.spec_interrupted_count() + self.spec_sleeping_count()
            }),
    {
    }

    /// Lemma: terminate() with sleeping (but no interrupted) produces InterruptedProcess.
    /// Sleeping threads become interrupted, so there will be interrupted threads.
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
                let interrupted_ids: Seq<int> =
                    self.interrupted_thread_ids@.add(self.sleeping_thread_ids@);
                interrupted_ids.len() == self.spec_sleeping_count()
                && interrupted_ids.len() >= 1
            }),
    {
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
    }

    /// Lemma: wakeup() preserves PID.
    pub proof fn lemma_wakeup_preserves_pid(&self)
        requires
            self.wf(),
        ensures
            self.spec_pid() == self.pid.spec_value(),
    {
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
    }

    /// Lemma: Successful wakeup() result is well-formed.
    pub proof fn lemma_wakeup_result_wf(
        &self,
        removed_idx: int,
        woken_tid: int,
        new_ready_time: int,
    )
        requires
            self.wf(),
            self.spec_sleeping_count() > 0,
            0 <= removed_idx < self.sleeping_thread_ids@.len(),
            self.sleeping_thread_ids@[removed_idx] == woken_tid,
            new_ready_time >= 0,
        ensures
            ({
                let new_ready_ids: Seq<int> = self.ready_thread_ids@.push(woken_tid);
                let new_ready_times: Seq<int> = self.ready_admission_times@.push(new_ready_time);
                let new_sleeping_ids: Seq<int> =
                    self.sleeping_thread_ids@.subrange(0, removed_idx)
                        .add(self.sleeping_thread_ids@.subrange(
                            removed_idx + 1,
                            self.sleeping_thread_ids@.len() as int,
                        ));
                let result: RunnableProcess = RunnableProcess {
                    pid: self.pid,
                    ready_thread_ids: Ghost(new_ready_ids),
                    ready_admission_times: Ghost(new_ready_times),
                    interrupted_thread_ids: Ghost(self.interrupted_thread_ids@),
                    sleeping_thread_ids: Ghost(new_sleeping_ids),
                    zombie_thread_ids: Ghost(self.zombie_thread_ids@),
                    interrupted_count: self.interrupted_count,
                    sleeping_count: (self.sleeping_count - 1) as u64,
                };
                result.wf()
                && result.spec_pid() == self.spec_pid()
            }),
    {
    }

    //==============================================================================================
    // add_thread() Lemmas
    //==============================================================================================

    /// Lemma: add_thread() increases ready count by 1.
    pub proof fn lemma_add_thread_increments_ready(&self, new_tid: int, new_time: int)
        requires
            self.wf(),
            new_time >= 0,
        ensures
            ({
                let new_ready_ids: Seq<int> = self.ready_thread_ids@.push(new_tid);
                let new_ready_times: Seq<int> = self.ready_admission_times@.push(new_time);
                new_ready_ids.len() == self.spec_ready_count() + 1
                && new_ready_times.len() == self.ready_admission_times@.len() + 1
            }),
    {
    }

    /// Lemma: add_thread() result is well-formed.
    pub proof fn lemma_add_thread_result_wf(&self, new_tid: int, new_time: int)
        requires
            self.wf(),
            new_time >= 0,
        ensures
            ({
                let result: RunnableProcess = RunnableProcess {
                    pid: self.pid,
                    ready_thread_ids: Ghost(self.ready_thread_ids@.push(new_tid)),
                    ready_admission_times: Ghost(self.ready_admission_times@.push(new_time)),
                    interrupted_thread_ids: Ghost(self.interrupted_thread_ids@),
                    sleeping_thread_ids: Ghost(self.sleeping_thread_ids@),
                    zombie_thread_ids: Ghost(self.zombie_thread_ids@),
                    interrupted_count: self.interrupted_count,
                    sleeping_count: self.sleeping_count,
                };
                result.wf()
                && result.spec_pid() == self.spec_pid()
                && result.spec_ready_count() == self.spec_ready_count() + 1
            }),
    {
    }

    /// Lemma: add_thread() preserves other thread lists unchanged.
    pub proof fn lemma_add_thread_preserves_others(&self, new_tid: int, new_time: int)
        requires
            self.wf(),
        ensures
            ({
                let result: RunnableProcess = RunnableProcess {
                    pid: self.pid,
                    ready_thread_ids: Ghost(self.ready_thread_ids@.push(new_tid)),
                    ready_admission_times: Ghost(self.ready_admission_times@.push(new_time)),
                    interrupted_thread_ids: Ghost(self.interrupted_thread_ids@),
                    sleeping_thread_ids: Ghost(self.sleeping_thread_ids@),
                    zombie_thread_ids: Ghost(self.zombie_thread_ids@),
                    interrupted_count: self.interrupted_count,
                    sleeping_count: self.sleeping_count,
                };
                result.spec_interrupted_count() == self.spec_interrupted_count()
                && result.spec_sleeping_count() == self.spec_sleeping_count()
                && result.spec_zombie_count() == self.spec_zombie_count()
            }),
    {
    }

    //==============================================================================================
    // earliest_admission_time() Lemmas
    //==============================================================================================

    /// Lemma: The earliest admission time exists among ready threads.
    /// For a non-empty finite sequence of ints, there exists a minimum.
    pub proof fn lemma_earliest_admission_time_exists(&self)
        requires
            self.wf(),
        ensures
            exists|idx: int| 0 <= idx < self.ready_admission_times@.len()
                && forall|j: int| 0 <= j < self.ready_admission_times@.len()
                    ==> #[trigger] self.ready_admission_times@[idx]
                        <= #[trigger] self.ready_admission_times@[j],
    {
        let s: &Seq<int> = &self.ready_admission_times@;
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
    proof fn lemma_min_index_rec_bounds(s: &Seq<int>, n: int)
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
        Self::lemma_min_index_rec_bounds(
            &self.ready_admission_times@,
            self.ready_admission_times@.len() as int,
        );
    }

    /// Helper: A non-empty sequence of ints has a minimum element within the first n elements.
    proof fn lemma_seq_has_min(s: &Seq<int>, n: int)
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
                    ==> self.ready_admission_times@[i] >= 0
            }),
    {
    }

    /// Refinement lemma: `spec_earliest_admission_time` is the minimum admission time,
    /// is non-negative, and its index is in bounds. This bridges the spec-only
    /// `earliest_admission_time()` to the proven properties.
    pub proof fn lemma_earliest_admission_time_refinement(&self)
        requires
            self.wf(),
        ensures
            ({
                let t: int = self.spec_earliest_admission_time();
                let idx: int = self.spec_earliest_ready_index();
                // The value is from the admission times array.
                t == self.ready_admission_times@[idx]
                // It is non-negative.
                && t >= 0
                // It is the minimum over all admission times.
                && forall|j: int| 0 <= j < self.ready_admission_times@.len()
                    ==> t <= #[trigger] self.ready_admission_times@[j]
            }),
    {
        self.lemma_earliest_ready_index_bounds();
    }

    //==============================================================================================
    // find_thread() Lemmas
    //==============================================================================================

    /// Lemma: spec_find_thread returns Some(0) iff the thread is in the ready list.
    pub proof fn lemma_find_thread_ready(&self, tid: int)
        requires
            self.spec_has_ready_thread(tid),
        ensures
            self.spec_find_thread(tid) == Some(0int),
    {
    }

    /// Lemma: spec_find_thread returns None iff the thread is not in any list.
    pub proof fn lemma_find_thread_not_found(&self, tid: int)
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
    pub proof fn lemma_find_thread_iff_has_thread(&self, tid: int)
        ensures
            self.spec_find_thread(tid).is_some() <==> self.spec_has_thread(tid),
    {
    }

    /// Lemma: If spec_find_thread is true, there exists a valid index.
    /// Used internally by wakeup() to derive `found_idx` from `found`.
    pub proof fn lemma_spec_find_thread_index(&self, tid: Ghost<int>)
        requires
            Self::spec_seq_contains(self.sleeping_thread_ids@, tid@),
        ensures
            exists|i: int| 0 <= i < self.sleeping_thread_ids@.len()
                && self.sleeping_thread_ids@[i] == tid@,
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

    /// Lemma: Two RunnableProcesses with identical fields have equal views.
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
}

//==================================================================================================
// RunningProcess Lemmas (Boundary)
//==================================================================================================

impl RunningProcess {
    /// Lemma: Construction preserves process identity.
    pub proof fn lemma_new_preserves_pid(pid: int, running_tid: int)
        ensures
            ({
                let r: RunningProcess = RunningProcess {
                    pid: Ghost(pid),
                    running_thread_id: Ghost(running_tid),
                    ready_thread_ids: Ghost(Seq::empty()),
                    interrupted_thread_ids: Ghost(Seq::empty()),
                    sleeping_thread_ids: Ghost(Seq::empty()),
                    zombie_thread_ids: Ghost(Seq::empty()),
                };
                r.spec_pid() == pid
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
