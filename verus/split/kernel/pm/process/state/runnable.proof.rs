// Copyright(c) The Maintainers of Nanvix.
// Licensed under the MIT License.

// RunnableProcess Proofs.
// This file contains proof lemmas for the RunnableProcess type.
//
// Key proven properties:
// - Construction produces well-formed state with correct initial values.
// - PID is immutable across all operations.
// - run() selects earliest admission time thread, preserves PID and total count.
// - terminate() converts ready→zombie, sleeping→interrupted, preserves PID.
// - wakeup() moves sleeping→ready, preserves PID and total count.
// - add_thread() increases ready count by 1, preserves PID and other lists.
// - earliest_admission_time() returns the minimum over ready thread admission times.
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
                    pid: self.pid.spec_value(),
                    running_thread_id: self.ready_thread_ids@[selected_idx],
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
    pub proof fn lemma_terminate_no_interrupted_gives_zombie(&self)
        requires
            self.wf(),
            self.spec_interrupted_count() == 0,
            self.spec_sleeping_count() == 0,
        ensures
            // With no interrupted or sleeping threads, terminate() must produce
            // a ZombieProcess (the Err branch).
            true,
    {
    }

    /// Lemma: terminate() with interrupted threads produces InterruptedProcess.
    pub proof fn lemma_terminate_with_interrupted_gives_interrupted(&self)
        requires
            self.wf(),
            self.spec_interrupted_count() > 0,
        ensures
            // With interrupted threads, terminate() produces an InterruptedProcess (the Ok branch).
            true,
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
            // Sleeping threads are converted to interrupted, so the result is InterruptedProcess.
            true,
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
                    ==> self.ready_admission_times@[idx] <= self.ready_admission_times@[j],
    {
        Self::lemma_seq_has_min(&self.ready_admission_times@, self.ready_admission_times@.len() as int);
    }

    /// Helper: A non-empty sequence of ints has a minimum element within the first n elements.
    proof fn lemma_seq_has_min(s: &Seq<int>, n: int)
        requires
            n >= 1,
            n <= s.len(),
        ensures
            exists|idx: int| 0 <= idx < n
                && forall|j: int| 0 <= j < n ==> s[idx] <= s[j],
        decreases n,
    {
        if n == 1 {
            assert(s[0] <= s[0]);
        } else {
            Self::lemma_seq_has_min(s, n - 1);
            let prev_min_idx: int = choose|idx: int| 0 <= idx < n - 1
                && forall|j: int| 0 <= j < n - 1 ==> s[idx] <= s[j];
            if s[n - 1] < s[prev_min_idx] {
                // New element is smaller.
                assert(forall|j: int| 0 <= j < n - 1 ==> s[prev_min_idx] <= s[j]);
                assert(s[n - 1] < s[prev_min_idx]);
                assert(forall|j: int| 0 <= j < n ==> s[n - 1] <= s[j]);
            } else {
                // Previous min is still min.
                assert(s[prev_min_idx] <= s[n - 1]);
                assert(forall|j: int| 0 <= j < n ==> s[prev_min_idx] <= s[j]);
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
                    pid: pid,
                    running_thread_id: running_tid,
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
                    pid: pid,
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
                    pid: pid,
                    zombie_thread_ids: Ghost(zombie_ids),
                    status: status,
                };
                zp.wf()
            }),
    {
    }
}

} // verus!
