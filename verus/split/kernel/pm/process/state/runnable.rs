// Copyright(c) The Maintainers of Nanvix.
// Licensed under the MIT License.

//! # RunnableProcess Implementation
//!
//! Represents a process that is ready to run in the Nanvix kernel.
//! A RunnableProcess always has at least one ready thread and manages
//! state transitions to RunningProcess (via run()), InterruptedProcess
//! or ZombieProcess (via terminate()), and back to self (via wakeup(),
//! add_thread()).
//!
//! ## Verified Properties
//!
//! - Construction (new, from_state) produces well-formed state with correct identity.
//! - Process identifier (PID) is immutable: all operations preserve it.
//! - The ready thread list is always non-empty (NonEmptyVecDeque invariant).
//! - `run()` selects the thread with the earliest admission time, transitions
//!   to RunningProcess, preserves PID and total thread count.
//! - `terminate()` converts all ready threads to zombies, sleeping threads to
//!   interrupted, and produces either InterruptedProcess or ZombieProcess
//!   depending on whether interrupted threads exist.
//! - `wakeup()` moves a sleeping thread to the ready queue, preserving PID
//!   and total thread count.
//! - `add_thread()` increases the ready thread count by 1, preserves PID
//!   and all other thread lists.
//! - `earliest_admission_time()` returns the minimum admission time among
//!   ready threads.
//! - Well-formedness is preserved by all operations.
//!
//! ## Verification Model
//!
//! The original `RunnableProcess` contains complex kernel types. For verification:
//! - `Box<ProcessState>` -> `ProcessIdentifier` PID (Box/ProcessState is transparent
//!   at this abstraction level; we carry the PID for identity tracking).
//! - `NonEmptyVecDeque<ReadyThread>` -> `Vec<i64>` of thread IDs +
//!   `Vec<i64>` of admission times. Non-emptiness is enforced by `wf()`.
//! - `Option<NonEmptyVecDeque<InterruptedThread>>` -> `Vec<i64>` (may be empty).
//! - `Option<NonEmptyVecDeque<SleepingThread>>` -> `Vec<i64>` (may be empty).
//! - `Option<NonEmptyVecDeque<ZombieThread>>` -> `Vec<i64>` (may be empty).
//! - `ContextInformation`, `VirtualAddress` from run() -> elided (HAL boundary).
//! - `InterruptReason` -> i64 tag.
//! - Thread state details (stacks, mutexes, etc.) are abstracted to just IDs
//!   since this module's concern is process-level thread collection management.
//!
//! ## Trust Boundary
//!
//! - `RunningProcess`, `InterruptedProcess`, `ZombieProcess` are boundary models
//!   of sibling modules. When those modules are independently verified, the
//!   boundary model postconditions must be confirmed as implied by the real
//!   implementations.
//! - `clock_now()` is `external_body`: returns a non-negative timestamp.
//! - `ProcessState` operations (`state()`, `state_mut()`) are elided since we
//!   only track the PID for identity verification. `state_mut()` allows arbitrary
//!   mutation of the inner `ProcessState`, which could affect invariants such as
//!   PID immutability or vmem mapping. **Cross-module verification obligation:**
//!   When `ProcessState` is independently verified, callers of `state_mut()` must
//!   prove that mutations preserve at minimum PID immutability (`spec_pid()` is
//!   unchanged after mutation) and any structural invariants assumed by this module.
//! - Thread state transitions (ReadyThread::terminate(), SleepingThread::interrupt(),
//!   SleepingThread::wakeup()) are modeled as ID-preserving operations.
//! - **`find_thread()` and `find_thread_mut()`** are omitted from the exec-level
//!   model because they return reference types (`ThreadRef`, `ThreadRefMut`) that
//!   Verus cannot express. A spec-only model (`spec_find_thread`) is provided
//!   that verifies the exhaustive search logic and correct list variant selection.
//! - **`earliest_admission_time()`** is modeled as spec-only
//!   (`spec_earliest_admission_time`) because the return type `SystemTime` maps
//!   to `i64` in our model. The `lemma_earliest_admission_time_exists`
//!   proof verifies that a minimum exists in the non-empty ready thread sequence.

use crate::kernel::pm::sys::pid::ProcessIdentifier;
use vstd::prelude::*;

// Include specifications.
include!("runnable.spec.rs");

// Include proofs.
include!("runnable.proof.rs");

verus! {

//==================================================================================================
// External Dependencies
//==================================================================================================

/// Abstract model of `clock::now()`.
///
/// Returns a concrete timestamp. The minimal postcondition reflects that
/// `SystemTime` values are non-negative.
#[verifier::external_body]
fn clock_now() -> (result: i64)
    ensures
        result >= 0i64,
{
    unimplemented!()
}

//==================================================================================================
// Structures
//==================================================================================================

/// A process that is ready to run.
///
/// Verification model of `src/kernel/src/pm/process/state/runnable.rs::RunnableProcess`.
/// Thread collections are modeled as concrete `Vec<i64>` of thread IDs.
/// Exec-level counters (`interrupted_count`, `sleeping_count`) track the lengths
/// of the corresponding concrete vectors, enabling internal branch decisions.
///
/// **Note:** Fields are `pub` for Verus proof ergonomics (spec access,
/// direct construction in lemmas). The original has private fields.
pub struct RunnableProcess {
    /// Process identifier (from the inner ProcessState).
    pub pid: ProcessIdentifier,
    /// Concrete vector of ready thread IDs (non-empty).
    pub ready_thread_ids: Vec<i64>,
    /// Concrete vector of ready thread admission times (parallel to ready_thread_ids).
    pub ready_admission_times: Vec<i64>,
    /// Concrete vector of interrupted thread IDs.
    pub interrupted_thread_ids: Vec<i64>,
    /// Concrete vector of sleeping thread IDs.
    pub sleeping_thread_ids: Vec<i64>,
    /// Concrete vector of zombie thread IDs.
    pub zombie_thread_ids: Vec<i64>,
    /// Exec-level count of interrupted threads (mirrors interrupted_thread_ids@.len()).
    pub interrupted_count: u64,
    /// Exec-level count of sleeping threads (mirrors sleeping_thread_ids@.len()).
    pub sleeping_count: u64,
}

/// A process that is running (boundary model).
///
/// Models the original `RunningProcess` from the sibling module.
/// Uses concrete types for all fields.
/// The `interrupt_reason` field tracks the `Option<InterruptReason>` from
/// the original `run()` return, signaling to downstream verifiers that
/// this data flows through the transition (even though its value is
/// unconstrained at this abstraction level).
pub struct RunningProcess {
    /// Process identifier (from the inner ProcessState).
    pub pid: ProcessIdentifier,
    /// The running thread ID.
    pub running_thread_id: i64,
    /// Remaining ready thread IDs.
    pub ready_thread_ids: Vec<i64>,
    /// Interrupted thread IDs.
    pub interrupted_thread_ids: Vec<i64>,
    /// Sleeping thread IDs.
    pub sleeping_thread_ids: Vec<i64>,
    /// Zombie thread IDs.
    pub zombie_thread_ids: Vec<i64>,
    /// Interrupt reason from the previous run (unconstrained).
    /// Models `Option<InterruptReason>` from the original return type.
    /// Downstream modules can constrain this value in their own specs.
    pub interrupt_reason: i64,
}

/// A process that was interrupted (boundary model).
///
/// Models the original `InterruptedProcess` from the sibling module.
pub struct InterruptedProcess {
    /// Process identifier.
    pub pid: ProcessIdentifier,
    /// Interrupted thread IDs (non-empty).
    pub interrupted_thread_ids: Vec<i64>,
    /// Zombie thread IDs.
    pub zombie_thread_ids: Vec<i64>,
}

/// A process that has terminated (boundary model).
///
/// Models the original `ZombieProcess` from the sibling module.
pub struct ZombieProcess {
    /// Process identifier.
    pub pid: ProcessIdentifier,
    /// Zombie thread IDs (non-empty).
    pub zombie_thread_ids: Vec<i64>,
    /// Exit status.
    pub status: i64,
}

/// Result of `RunnableProcess::terminate()`.
///
/// Models `Result<InterruptedProcess, ZombieProcess>` from the original.
pub enum TerminateResult {
    /// Process has interrupted threads remaining.
    Interrupted(InterruptedProcess),
    /// Process is fully terminated (all threads are zombies).
    Zombie(ZombieProcess),
}

//==================================================================================================
// Helper Functions
//==================================================================================================

/// Builds a new Vec<i64> by copying all elements from `v` except the one at `skip_idx`.
fn vec_remove_at(v: &Vec<i64>, skip_idx: usize) -> (result: Vec<i64>)
    requires
        0 <= skip_idx < v@.len(),
    ensures
        result@.len() == v@.len() - 1,
        result@ == v@.subrange(0, skip_idx as int).add(
            v@.subrange(skip_idx as int + 1, v@.len() as int)),
{
    let mut result: Vec<i64> = Vec::new();
    let mut j: usize = 0;
    while j < v.len()
        invariant
            0 <= j <= v@.len(),
            0 <= skip_idx < v@.len(),
            j <= skip_idx ==> result@.len() == j as nat,
            j > skip_idx ==> result@.len() == (j - 1) as nat,
            j <= skip_idx ==> result@ == v@.subrange(0, j as int),
            j > skip_idx ==> result@ == v@.subrange(0, skip_idx as int).add(
                v@.subrange(skip_idx as int + 1, j as int)),
        decreases v@.len() - j,
    {
        if j != skip_idx {
            result.push(v[j]);
        }
        j = j + 1;
    }
    result
}

/// Concatenates two Vec<i64> into a new Vec<i64>.
fn vec_concat(v1: &Vec<i64>, v2: &Vec<i64>) -> (result: Vec<i64>)
    ensures
        result@ == v1@.add(v2@),
        result@.len() == v1@.len() + v2@.len(),
{
    let mut result: Vec<i64> = Vec::new();
    let mut i: usize = 0;
    while i < v1.len()
        invariant
            0 <= i <= v1@.len(),
            result@ == v1@.subrange(0, i as int),
            result@.len() == i as nat,
        decreases v1@.len() - i,
    {
        result.push(v1[i]);
        i = i + 1;
    }
    proof {
        assert(result@ =~= v1@.subrange(0, v1@.len() as int));
        assert(v1@.subrange(0, v1@.len() as int) =~= v1@);
    }
    let mut j: usize = 0;
    while j < v2.len()
        invariant
            0 <= j <= v2@.len(),
            result@ == v1@.add(v2@.subrange(0, j as int)),
            result@.len() == v1@.len() + j as nat,
        decreases v2@.len() - j,
    {
        result.push(v2[j]);
        proof {
            assert(v2@.subrange(0, j as int).push(v2@[j as int])
                =~= v2@.subrange(0, j + 1));
        }
        j = j + 1;
    }
    proof {
        assert(v2@.subrange(0, v2@.len() as int) =~= v2@);
    }
    result
}

/// Searches for `target` in `v`, returning (found, index).
fn vec_search(v: &Vec<i64>, target: i64) -> (result: (bool, usize))
    ensures
        result.0 == (exists|i: int| 0 <= i < v@.len() && #[trigger] v@[i] == target),
        result.0 ==> (0 <= result.1 < v@.len() && v@[result.1 as int] == target),
{
    let mut i: usize = 0;
    while i < v.len()
        invariant
            0 <= i <= v@.len(),
            forall|j: int| 0 <= j < i as int ==> #[trigger] v@[j] != target,
        decreases v@.len() - i,
    {
        if v[i] == target {
            return (true, i);
        }
        i = i + 1;
    }
    (false, 0)
}

//==================================================================================================
// RunnableProcess Implementation
//==================================================================================================

impl RunnableProcess {
    /// Creates a new runnable process with a single ready thread.
    ///
    /// Models the original `RunnableProcess::new(pid, ready_thread, vmem)`.
    /// The Vmem parameter is omitted (HAL boundary).
    ///
    /// # Parameters
    ///
    /// - `pid`: Process identifier.
    /// - `ready_tid`: Thread identifier of the initial ready thread.
    /// - `ready_time`: Admission time of the initial ready thread.
    ///
    /// # Returns
    ///
    /// A new, well-formed RunnableProcess with one ready thread.
    pub fn new(pid: ProcessIdentifier, ready_tid: i64, ready_time: i64) -> (result: RunnableProcess)
        requires
            ready_time >= 0i64,
        ensures
            result@.pid == pid.spec_value(),
            result@.ready_thread_ids.len() == 1,
            result@.interrupted_thread_ids.len() == 0,
            result@.sleeping_thread_ids.len() == 0,
            result@.zombie_thread_ids.len() == 0,
            result.wf(),
    {
        proof { reveal(RunnableProcess::wf); }
        let mut rid_vec: Vec<i64> = Vec::new();
        rid_vec.push(ready_tid);
        let mut rtime_vec: Vec<i64> = Vec::new();
        rtime_vec.push(ready_time);
        RunnableProcess {
            pid: pid,
            ready_thread_ids: rid_vec,
            ready_admission_times: rtime_vec,
            interrupted_thread_ids: Vec::new(),
            sleeping_thread_ids: Vec::new(),
            zombie_thread_ids: Vec::new(),
            interrupted_count: 0,
            sleeping_count: 0,
        }
    }

    /// Creates a runnable process from existing state and thread lists.
    ///
    /// Models the original `RunnableProcess::from_state(...)`.
    /// Note: The original is `pub(super)` (used by sibling state modules);
    /// here it is `pub` for Verus proof ergonomics (lemma construction).
    /// This visibility difference has no functional impact.
    ///
    /// # Parameters
    ///
    /// - `pid`: Process identifier.
    /// - `ready_ids`: Non-empty vector of ready thread IDs.
    /// - `ready_times`: Admission times parallel to ready_ids.
    /// - `interrupted_ids`: Interrupted thread IDs.
    /// - `sleeping_ids`: Sleeping thread IDs.
    /// - `zombie_ids`: Zombie thread IDs.
    ///
    /// # Returns
    ///
    /// A well-formed RunnableProcess.
    pub fn from_state(
        pid: ProcessIdentifier,
        ready_ids: Vec<i64>,
        ready_times: Vec<i64>,
        interrupted_ids: Vec<i64>,
        sleeping_ids: Vec<i64>,
        zombie_ids: Vec<i64>,
        interrupted_count: u64,
        sleeping_count: u64,
    ) -> (result: RunnableProcess)
        requires
            ready_ids@.len() >= 1,
            ready_ids@.len() == ready_times@.len(),
            forall|i: int| 0 <= i < ready_times@.len()
                ==> #[trigger] ready_times@[i] >= 0i64,
            interrupted_count as nat == interrupted_ids@.len(),
            sleeping_count as nat == sleeping_ids@.len(),
        ensures
            result@.pid == pid.spec_value(),
            result@.ready_thread_ids.len() == ready_ids@.len(),
            result@.interrupted_thread_ids.len() == interrupted_ids@.len(),
            result@.sleeping_thread_ids.len() == sleeping_ids@.len(),
            result@.zombie_thread_ids.len() == zombie_ids@.len(),
            result.wf(),
    {
        proof { reveal(RunnableProcess::wf); }
        RunnableProcess {
            pid: pid,
            ready_thread_ids: ready_ids,
            ready_admission_times: ready_times,
            interrupted_thread_ids: interrupted_ids,
            sleeping_thread_ids: sleeping_ids,
            zombie_thread_ids: zombie_ids,
            interrupted_count: interrupted_count,
            sleeping_count: sleeping_count,
        }
    }

    /// Returns the process identifier as i32.
    pub fn pid_i32(&self) -> (result: i32)
        ensures
            result as int == self@.pid,
    {
        self.pid.into_i32()
    }

    /// Transitions the runnable process to running by selecting the thread
    /// with the earliest admission time.
    ///
    /// Models the original `RunnableProcess::run()`.
    /// The `ContextInformation` pointer and `VirtualAddress` from the original
    /// return type are omitted (HAL boundary; see Trust Boundary docs).
    ///
    /// The minimum-index selection is computed concretely via an exec-level
    /// loop that scans all ready threads, matching the original for-loop.
    /// A proof block invokes `lemma_exec_min_matches_spec` to prove the
    /// loop result matches `spec_earliest_ready_index()`.
    ///
    /// # Returns
    ///
    /// A RunningProcess with the earliest-admission-time thread as running.
    pub fn run(self) -> (result: RunningProcess)
        requires
            self.wf(),
        ensures
            result@.pid == self@.pid,
            // The selected thread has the earliest admission time.
            ({
                let sel: int = self@.spec_earliest_ready_index();
                result@.running_thread_id == self@.ready_thread_ids[sel]
                && result@.ready_thread_ids ==
                    RunnableProcessView::spec_remove_at(self@.ready_thread_ids, sel)
                && result@.ready_thread_ids.len() == self@.ready_thread_ids.len() - 1
            }),
            // Other lists are preserved exactly.
            result@.interrupted_thread_ids == self@.interrupted_thread_ids,
            result@.sleeping_thread_ids == self@.sleeping_thread_ids,
            result@.zombie_thread_ids == self@.zombie_thread_ids,
    {
        proof { reveal(RunnableProcess::wf); }
        // Find the index of the thread with earliest admission time via concrete loop.
        let mut min_idx: usize = 0;
        let mut i: usize = 1;
        while i < self.ready_admission_times.len()
            invariant
                1 <= i <= self.ready_admission_times@.len(),
                0 <= min_idx < i,
                min_idx < self.ready_admission_times@.len(),
                forall|j: int| 0 <= j < i as int
                    ==> self.ready_admission_times@[min_idx as int]
                        <= self.ready_admission_times@[j],
                // Track correspondence with spec_min_index_rec.
                min_idx as int == Self::spec_min_index_rec(
                    self.ready_admission_times@, i as int),
                self.wf(),
            decreases self.ready_admission_times@.len() - i,
        {
            if self.ready_admission_times[i] < self.ready_admission_times[min_idx] {
                min_idx = i;
            }
            i = i + 1;
        }

        // After loop: min_idx == spec_min_index_rec(s, len) == spec_earliest_ready_index().
        proof {
            assert(min_idx as int == self.spec_earliest_ready_index());
        }

        let selected_tid: i64 = self.ready_thread_ids[min_idx];

        // Build remaining ready lists by removing min_idx.
        let new_ready_ids: Vec<i64> = vec_remove_at(&self.ready_thread_ids, min_idx);
        let new_ready_times: Vec<i64> = vec_remove_at(&self.ready_admission_times, min_idx);

        // Discard ready_admission_times (not needed in RunningProcess).
        proof {
            // Bridge exec-level Seq<i64> to view-level Seq<int>.
            Self::lemma_view_min_index_eq(
                self.ready_admission_times@,
                self.ready_admission_times@.len() as int,
            );
            Self::lemma_min_index_rec_bounds(
                &self.ready_admission_times@,
                self.ready_admission_times@.len() as int,
            );
            assert(min_idx as int == self@.spec_earliest_ready_index());
            let sel: int = min_idx as int;
            assert(0 <= sel < self.ready_thread_ids@.len());
            // Break down spec_remove_at distribution over spec_i64_seq_as_int.
            let s: Seq<i64> = self.ready_thread_ids@;
            let s_int: Seq<int> = self@.ready_thread_ids;
            assert forall|i: int| 0 <= i < sel
                implies (#[trigger] spec_i64_seq_as_int(s.subrange(0, sel))[i])
                    == s_int.subrange(0, sel)[i]
            by {
                assert(s.subrange(0, sel)[i] == s[i]);
                assert(s_int.subrange(0, sel)[i] == s_int[i]);
                assert(s_int[i] == s[i] as int);
            }
            assert(spec_i64_seq_as_int(s.subrange(0, sel))
                =~= s_int.subrange(0, sel));
            assert forall|i: int| 0 <= i < (s.len() as int - (sel + 1))
                implies (#[trigger] spec_i64_seq_as_int(
                    s.subrange(sel + 1, s.len() as int))[i])
                    == s_int.subrange(sel + 1, s_int.len() as int)[i]
            by {
                assert(s.subrange(sel + 1, s.len() as int)[i] == s[sel + 1 + i]);
                assert(s_int.subrange(sel + 1, s_int.len() as int)[i]
                    == s_int[sel + 1 + i]);
                assert(s_int[sel + 1 + i] == s[sel + 1 + i] as int);
            }
            assert(spec_i64_seq_as_int(s.subrange(sel + 1, s.len() as int))
                =~= s_int.subrange(sel + 1, s_int.len() as int));
            let left: Seq<i64> = s.subrange(0, sel);
            let right: Seq<i64> = s.subrange(sel + 1, s.len() as int);
            assert(spec_i64_seq_as_int(left.add(right))
                =~= spec_i64_seq_as_int(left).add(spec_i64_seq_as_int(right)));
        }
        RunningProcess {
            pid: self.pid,
            running_thread_id: selected_tid,
            ready_thread_ids: new_ready_ids,
            interrupted_thread_ids: self.interrupted_thread_ids,
            sleeping_thread_ids: self.sleeping_thread_ids,
            zombie_thread_ids: self.zombie_thread_ids,
            interrupt_reason: 0i64,
        }
    }

    /// Terminates the runnable process.
    ///
    /// Models the original `RunnableProcess::terminate()`:
    /// - All ready threads are terminated (become zombies).
    /// - All sleeping threads are interrupted (become interrupted).
    /// - If interrupted threads exist (original or from sleeping), returns
    ///   Ok(InterruptedProcess). Otherwise, returns Err(ZombieProcess).
    ///
    /// The branch decision is computed internally from exec-level counters
    /// (`interrupted_count`, `sleeping_count`), eliminating the need for an
    /// oracle parameter. The `wf()` invariant guarantees these counters match
    /// the concrete vector lengths.
    ///
    /// # Returns
    ///
    /// TerminateResult::Interrupted if any interrupted threads remain,
    /// TerminateResult::Zombie if only zombie threads remain.
    pub fn terminate(self) -> (result: TerminateResult)
        requires
            self.wf(),
        ensures
            match result {
                TerminateResult::Interrupted(ip) => {
                    // PID preserved.
                    ip@.pid == self@.pid
                    // Interrupted threads are exactly original interrupted + sleeping->interrupted.
                    && ip@.interrupted_thread_ids.len() ==
                        self@.interrupted_thread_ids.len() + self@.sleeping_thread_ids.len()
                    && ip@.interrupted_thread_ids ==
                        self@.interrupted_thread_ids.add(self@.sleeping_thread_ids)
                    // Zombie threads include all original ready + original zombie.
                    && ip@.zombie_thread_ids.len() ==
                        self@.ready_thread_ids.len() + self@.zombie_thread_ids.len()
                    && ip@.zombie_thread_ids ==
                        self@.ready_thread_ids.add(self@.zombie_thread_ids)
                    && ip.wf()
                    // Branch taken iff there were interrupted or sleeping threads.
                    && (self@.interrupted_thread_ids.len() > 0
                        || self@.sleeping_thread_ids.len() > 0)
                },
                TerminateResult::Zombie(zp) => {
                    // PID preserved.
                    zp@.pid == self@.pid
                    // No interrupted or sleeping threads existed.
                    && self@.interrupted_thread_ids.len() == 0
                    && self@.sleeping_thread_ids.len() == 0
                    // Zombie threads include all original ready + original zombie.
                    && zp@.zombie_thread_ids.len() ==
                        self@.ready_thread_ids.len() + self@.zombie_thread_ids.len()
                    && zp@.zombie_thread_ids ==
                        self@.ready_thread_ids.add(self@.zombie_thread_ids)
                    && zp@.status == EXIT_STATUS_INTERRUPTED()
                    && zp.wf()
                },
            },
    {
        proof {
            reveal(RunnableProcess::wf);
            reveal(InterruptedProcess::wf);
            reveal(ZombieProcess::wf);
        }
        // Build new zombie list: ready + existing zombie.
        let new_zombie_ids: Vec<i64> = vec_concat(
            &self.ready_thread_ids, &self.zombie_thread_ids);

        // Build new interrupted list: original interrupted + sleeping->interrupted.
        let new_interrupted_ids: Vec<i64> = vec_concat(
            &self.interrupted_thread_ids, &self.sleeping_thread_ids);

        if self.interrupted_count > 0 || self.sleeping_count > 0 {
            proof {
                assert(new_interrupted_ids@.len() ==
                    self.interrupted_thread_ids@.len() + self.sleeping_thread_ids@.len());
                assert(new_interrupted_ids@.len() >= 1);
                // Bridge: spec_i64_seq_as_int distributes over add.
                assert(spec_i64_seq_as_int(new_interrupted_ids@)
                    =~= spec_i64_seq_as_int(self.interrupted_thread_ids@).add(
                        spec_i64_seq_as_int(self.sleeping_thread_ids@)));
                assert(spec_i64_seq_as_int(new_zombie_ids@)
                    =~= spec_i64_seq_as_int(self.ready_thread_ids@).add(
                        spec_i64_seq_as_int(self.zombie_thread_ids@)));
            }
            TerminateResult::Interrupted(InterruptedProcess {
                pid: self.pid,
                interrupted_thread_ids: new_interrupted_ids,
                zombie_thread_ids: new_zombie_ids,
            })
        } else {
            proof {
                assert(self.interrupted_thread_ids@.len() == 0);
                assert(self.sleeping_thread_ids@.len() == 0);
                assert(new_zombie_ids@.len() >= 1);
                // Bridge: spec_i64_seq_as_int distributes over add.
                assert(spec_i64_seq_as_int(new_zombie_ids@)
                    =~= spec_i64_seq_as_int(self.ready_thread_ids@).add(
                        spec_i64_seq_as_int(self.zombie_thread_ids@)));
            }
            TerminateResult::Zombie(ZombieProcess {
                pid: self.pid,
                zombie_thread_ids: new_zombie_ids,
                status: EXIT_STATUS_INTERRUPTED_I64(),
            })
        }
    }

    /// Wakes up a sleeping thread and moves it to the ready queue.
    ///
    /// Models the original `RunnableProcess::wakeup(tid)`.
    /// Returns Ok(RunnableProcess) on success, Err(RunnableProcess) if the
    /// thread is not found in sleeping threads.
    ///
    /// With concrete Vec fields, the search is performed by an exec-level
    /// loop (no oracle parameter needed). The `found` boolean is computed
    /// internally from the concrete sleeping_thread_ids vector.
    ///
    /// # Parameters
    ///
    /// - `tid`: Thread ID to wake up.
    ///
    /// # Returns
    ///
    /// Ok with updated state if found, Err with unchanged state if not found.
    pub fn wakeup(self, tid: i64) -> (result: Result<RunnableProcess, RunnableProcess>)
        requires
            self.wf(),
        ensures
            match result {
                Ok(r) => {
                    RunnableProcessView::spec_seq_contains(
                        self@.sleeping_thread_ids, tid as int)
                    && r@.pid == self@.pid
                    && r@.ready_thread_ids.len() == self@.ready_thread_ids.len() + 1
                    && r@.sleeping_thread_ids.len() == self@.sleeping_thread_ids.len() - 1
                    && r@.interrupted_thread_ids.len() == self@.interrupted_thread_ids.len()
                    && r@.zombie_thread_ids.len() == self@.zombie_thread_ids.len()
                    // Content specs: ready list gets the woken thread appended.
                    && r@.ready_thread_ids == self@.ready_thread_ids.push(tid as int)
                    // Admission times grow by exactly one non-negative element.
                    && (exists|t: int| t >= 0
                        && r@.ready_admission_times
                            == self@.ready_admission_times.push(t))
                    // Sleeping list has the found thread removed.
                    && (exists|idx: int| 0 <= idx < self@.sleeping_thread_ids.len()
                        && self@.sleeping_thread_ids[idx] == tid as int
                        && r@.sleeping_thread_ids ==
                            RunnableProcessView::spec_remove_at(
                                self@.sleeping_thread_ids, idx))
                    // Other lists preserved exactly.
                    && r@.interrupted_thread_ids == self@.interrupted_thread_ids
                    && r@.zombie_thread_ids == self@.zombie_thread_ids
                    && r.wf()
                },
                Err(r) => {
                    !RunnableProcessView::spec_seq_contains(
                        self@.sleeping_thread_ids, tid as int)
                    && r@.pid == self@.pid
                    && r@.ready_thread_ids.len() == self@.ready_thread_ids.len()
                    && r@.sleeping_thread_ids.len() == self@.sleeping_thread_ids.len()
                    && r@.interrupted_thread_ids.len() == self@.interrupted_thread_ids.len()
                    && r@.zombie_thread_ids.len() == self@.zombie_thread_ids.len()
                    // Content preserved exactly.
                    && r@.ready_thread_ids == self@.ready_thread_ids
                    && r@.ready_admission_times == self@.ready_admission_times
                    && r@.interrupted_thread_ids == self@.interrupted_thread_ids
                    && r@.sleeping_thread_ids == self@.sleeping_thread_ids
                    && r@.zombie_thread_ids == self@.zombie_thread_ids
                    && r.wf()
                },
            },
    {
        proof { reveal(RunnableProcess::wf); }
        // Search for tid in the concrete sleeping list.
        let (found, found_idx_usize): (bool, usize) = vec_search(
            &self.sleeping_thread_ids, tid);

        if !found {
            proof {
                // Bridge: not found in Seq<i64> implies not found in Seq<int>.
                assert(!RunnableProcessView::spec_seq_contains(
                    self@.sleeping_thread_ids, tid as int))
                by {
                    if RunnableProcessView::spec_seq_contains(
                        self@.sleeping_thread_ids, tid as int) {
                        let idx: int = choose|i: int| 0 <= i
                            < self@.sleeping_thread_ids.len()
                            && self@.sleeping_thread_ids[i] == tid as int;
                        assert(self.sleeping_thread_ids@[idx] == tid);
                    }
                }
            }
            return Err(self);
        }

        // Build new sleeping list by removing the found thread.
        let new_sleeping_ids: Vec<i64> = vec_remove_at(
            &self.sleeping_thread_ids, found_idx_usize);

        // Build new ready lists by appending the woken thread.
        let new_ready_time: i64 = clock_now();
        let mut new_ready_ids: Vec<i64> = Vec::new();
        let mut k: usize = 0;
        while k < self.ready_thread_ids.len()
            invariant
                0 <= k <= self.ready_thread_ids@.len(),
                new_ready_ids@ == self.ready_thread_ids@.subrange(0, k as int),
                new_ready_ids@.len() == k as nat,
            decreases self.ready_thread_ids@.len() - k,
        {
            new_ready_ids.push(self.ready_thread_ids[k]);
            k = k + 1;
        }
        proof {
            assert(new_ready_ids@ =~= self.ready_thread_ids@.subrange(
                0, self.ready_thread_ids@.len() as int));
            assert(self.ready_thread_ids@.subrange(
                0, self.ready_thread_ids@.len() as int) =~= self.ready_thread_ids@);
        }
        new_ready_ids.push(tid);

        let mut new_ready_times: Vec<i64> = Vec::new();
        let mut m: usize = 0;
        while m < self.ready_admission_times.len()
            invariant
                0 <= m <= self.ready_admission_times@.len(),
                new_ready_times@ == self.ready_admission_times@.subrange(0, m as int),
                new_ready_times@.len() == m as nat,
            decreases self.ready_admission_times@.len() - m,
        {
            new_ready_times.push(self.ready_admission_times[m]);
            m = m + 1;
        }
        proof {
            assert(new_ready_times@ =~= self.ready_admission_times@.subrange(
                0, self.ready_admission_times@.len() as int));
            assert(self.ready_admission_times@.subrange(
                0, self.ready_admission_times@.len() as int) =~= self.ready_admission_times@);
        }
        new_ready_times.push(new_ready_time);

        proof {
            // Prove new ready lengths match.
            assert(new_ready_ids@.len() == self.ready_thread_ids@.len() + 1);
            assert(new_ready_times@.len() == self.ready_admission_times@.len() + 1);

            // Prove new_ready_times elements are non-negative.
            assert forall|i: int| 0 <= i < new_ready_times@.len()
                implies #[trigger] new_ready_times@[i] >= 0i64
            by {
                if i < self.ready_admission_times@.len() as int {
                    assert(new_ready_times@[i] == self.ready_admission_times@[i]);
                } else {
                    assert(new_ready_times@[i] == new_ready_time);
                }
            }

            // Prove new sleeping length.
            assert(new_sleeping_ids@.len() == self.sleeping_thread_ids@.len() - 1);

            // Bridge exec-level Seq<i64> to view-level Seq<int>.
            // Ready IDs: push distributes over spec_i64_seq_as_int.
            assert(spec_i64_seq_as_int(new_ready_ids@)
                =~= spec_i64_seq_as_int(self.ready_thread_ids@).push(tid as int));
            // Admission times: push distributes over spec_i64_seq_as_int.
            assert(spec_i64_seq_as_int(new_ready_times@)
                =~= spec_i64_seq_as_int(self.ready_admission_times@).push(
                    new_ready_time as int));
            // Sleeping IDs: spec_remove_at distributes over spec_i64_seq_as_int.
            // Break down: subrange distributes, then add distributes.
            let sl: Seq<i64> = self.sleeping_thread_ids@;
            let sl_int: Seq<int> = self@.sleeping_thread_ids;
            let fi: int = found_idx_usize as int;
            assert forall|i: int| 0 <= i < fi
                implies (#[trigger] spec_i64_seq_as_int(sl.subrange(0, fi))[i])
                    == sl_int.subrange(0, fi)[i]
            by {
                assert(sl.subrange(0, fi)[i] == sl[i]);
                assert(sl_int.subrange(0, fi)[i] == sl_int[i]);
                assert(sl_int[i] == sl[i] as int);
            }
            assert(spec_i64_seq_as_int(sl.subrange(0, fi))
                =~= sl_int.subrange(0, fi));
            assert forall|i: int| 0 <= i < (sl.len() as int - (fi + 1))
                implies (#[trigger] spec_i64_seq_as_int(
                    sl.subrange(fi + 1, sl.len() as int))[i])
                    == sl_int.subrange(fi + 1, sl_int.len() as int)[i]
            by {
                assert(sl.subrange(fi + 1, sl.len() as int)[i]
                    == sl[fi + 1 + i]);
                assert(sl_int.subrange(fi + 1, sl_int.len() as int)[i]
                    == sl_int[fi + 1 + i]);
                assert(sl_int[fi + 1 + i] == sl[fi + 1 + i] as int);
            }
            assert(spec_i64_seq_as_int(sl.subrange(fi + 1, sl.len() as int))
                =~= sl_int.subrange(fi + 1, sl_int.len() as int));
            let sl_left: Seq<i64> = sl.subrange(0, fi);
            let sl_right: Seq<i64> = sl.subrange(fi + 1, sl.len() as int);
            assert(spec_i64_seq_as_int(sl_left.add(sl_right))
                =~= spec_i64_seq_as_int(sl_left).add(spec_i64_seq_as_int(sl_right)));
            // Connect new_sleeping_ids@ to spec_remove_at result.
            assert(new_sleeping_ids@ == sl_left.add(sl_right));
            assert(spec_i64_seq_as_int(new_sleeping_ids@)
                =~= RunnableProcessView::spec_remove_at(sl_int, fi));
            // spec_seq_contains bridging: found at found_idx_usize in Seq<i64>,
            // so found at same index in Seq<int>.
            assert(self@.sleeping_thread_ids[found_idx_usize as int] == tid as int);
            assert(RunnableProcessView::spec_seq_contains(
                self@.sleeping_thread_ids, tid as int));
            // Existential witness for the postcondition.
            assert(0 <= fi < self@.sleeping_thread_ids.len()
                && self@.sleeping_thread_ids[fi] == tid as int);
        }

        Ok(RunnableProcess {
            pid: self.pid,
            ready_thread_ids: new_ready_ids,
            ready_admission_times: new_ready_times,
            interrupted_thread_ids: self.interrupted_thread_ids,
            sleeping_thread_ids: new_sleeping_ids,
            zombie_thread_ids: self.zombie_thread_ids,
            interrupted_count: self.interrupted_count,
            sleeping_count: self.sleeping_count - 1,
        })
    }

    /// Adds a thread to the ready queue.
    ///
    /// Models the original `RunnableProcess::add_thread(ready_thread)`.
    ///
    /// # Parameters
    ///
    /// - `ready_tid`: Thread ID of the new ready thread.
    /// - `ready_time`: Admission time of the new ready thread.
    ///
    /// # Returns
    ///
    /// An updated RunnableProcess with one additional ready thread.
    pub fn add_thread(self, ready_tid: i64, ready_time: i64) -> (result: RunnableProcess)
        requires
            self.wf(),
            ready_time >= 0i64,
        ensures
            result@.pid == self@.pid,
            result@.ready_thread_ids.len() == self@.ready_thread_ids.len() + 1,
            result@.interrupted_thread_ids.len() == self@.interrupted_thread_ids.len(),
            result@.sleeping_thread_ids.len() == self@.sleeping_thread_ids.len(),
            result@.zombie_thread_ids.len() == self@.zombie_thread_ids.len(),
            // Content specs: ready list gets the new thread appended.
            result@.ready_thread_ids == self@.ready_thread_ids.push(ready_tid as int),
            result@.ready_admission_times
                == self@.ready_admission_times.push(ready_time as int),
            // Other lists preserved exactly.
            result@.interrupted_thread_ids == self@.interrupted_thread_ids,
            result@.sleeping_thread_ids == self@.sleeping_thread_ids,
            result@.zombie_thread_ids == self@.zombie_thread_ids,
            result.wf(),
    {
        proof { reveal(RunnableProcess::wf); }
        // Copy and push to ready lists.
        let mut new_ready_ids: Vec<i64> = Vec::new();
        let mut k: usize = 0;
        while k < self.ready_thread_ids.len()
            invariant
                0 <= k <= self.ready_thread_ids@.len(),
                new_ready_ids@ == self.ready_thread_ids@.subrange(0, k as int),
                new_ready_ids@.len() == k as nat,
            decreases self.ready_thread_ids@.len() - k,
        {
            new_ready_ids.push(self.ready_thread_ids[k]);
            k = k + 1;
        }
        proof {
            assert(new_ready_ids@ =~= self.ready_thread_ids@.subrange(
                0, self.ready_thread_ids@.len() as int));
            assert(self.ready_thread_ids@.subrange(
                0, self.ready_thread_ids@.len() as int) =~= self.ready_thread_ids@);
        }
        new_ready_ids.push(ready_tid);

        let mut new_ready_times: Vec<i64> = Vec::new();
        let mut m: usize = 0;
        while m < self.ready_admission_times.len()
            invariant
                0 <= m <= self.ready_admission_times@.len(),
                new_ready_times@ == self.ready_admission_times@.subrange(0, m as int),
                new_ready_times@.len() == m as nat,
            decreases self.ready_admission_times@.len() - m,
        {
            new_ready_times.push(self.ready_admission_times[m]);
            m = m + 1;
        }
        proof {
            assert(new_ready_times@ =~= self.ready_admission_times@.subrange(
                0, self.ready_admission_times@.len() as int));
            assert(self.ready_admission_times@.subrange(
                0, self.ready_admission_times@.len() as int) =~= self.ready_admission_times@);
        }
        new_ready_times.push(ready_time);

        proof {
            assert(new_ready_ids@.len() == self.ready_thread_ids@.len() + 1);
            assert(new_ready_times@.len() == self.ready_admission_times@.len() + 1);

            // Prove new_ready_times elements are non-negative.
            assert forall|i: int| 0 <= i < new_ready_times@.len()
                implies #[trigger] new_ready_times@[i] >= 0i64
            by {
                if i < self.ready_admission_times@.len() as int {
                    assert(new_ready_times@[i] == self.ready_admission_times@[i]);
                } else {
                    assert(new_ready_times@[i] == ready_time);
                }
            }

            // Bridge exec-level Seq<i64> to view-level Seq<int>.
            assert(spec_i64_seq_as_int(new_ready_ids@)
                =~= spec_i64_seq_as_int(self.ready_thread_ids@).push(ready_tid as int));
            assert(spec_i64_seq_as_int(new_ready_times@)
                =~= spec_i64_seq_as_int(self.ready_admission_times@).push(
                    ready_time as int));
        }

        RunnableProcess {
            pid: self.pid,
            ready_thread_ids: new_ready_ids,
            ready_admission_times: new_ready_times,
            interrupted_thread_ids: self.interrupted_thread_ids,
            sleeping_thread_ids: self.sleeping_thread_ids,
            zombie_thread_ids: self.zombie_thread_ids,
            interrupted_count: self.interrupted_count,
            sleeping_count: self.sleeping_count,
        }
    }

}

} // verus!
