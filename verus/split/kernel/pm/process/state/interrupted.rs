// Copyright(c) The Maintainers of Nanvix.
// Licensed under the MIT License.

//! # InterruptedProcess Implementation
//!
//! Represents a process that was interrupted in the Nanvix kernel.
//! An InterruptedProcess has at least one interrupted thread, optional
//! sleeping threads, and optional zombie threads.
//!
//! ## Verified Properties
//!
//! - Construction (new, from_sleeping) produces well-formed state with correct identity.
//! - Process identifier (PID) is immutable: all operations preserve it.
//! - `resume()` pops the front interrupted thread, resumes it (ID-preserving),
//!   and produces a RunnableProcess with that thread as the only ready thread.
//!   Remaining interrupted, sleeping, and zombie threads are preserved.
//! - `find_thread()` / `find_thread_mut()` are modeled spec-only via
//!   `spec_find_thread()`.
//! - Well-formedness (including thread ID uniqueness and list disjointness)
//!   is preserved by all operations.
//! - Standalone `interrupt()` function is modeled as ID-preserving.
//!
//! ## Verification Model
//!
//! The original `InterruptedProcess` contains complex kernel types. For verification:
//! - `Box<ProcessState>` -> PID (`u64`, identity tracking only).
//! - `NonEmptyVecDeque<InterruptedThread>` -> `Vec<u64>` of thread IDs (concrete).
//! - `Option<NonEmptyVecDeque<SleepingThread>>` -> `Vec<u64>` (empty = None).
//! - `Option<NonEmptyVecDeque<ZombieThread>>` -> `Vec<u64>` (empty = None).
//! - `InterruptReason` -> modeled as spec constant `INTERRUPT_REASON_KILLED`.
//!   The standalone `interrupt()` function returns this tag explicitly.
//!
//! ## Trust Boundary
//!
//! - `RunnableProcess` is a boundary model of the sibling module. Its `wf()`
//!   includes no-duplicates and pairwise-disjoint conditions matching the
//!   structural integrity of the real type. `ready_admission_times` models
//!   the parallel admission time array.
//!   Note: The `InterruptedProcess` boundary model in `runnable.spec.rs`
//!   omits `sleeping_thread_ids`; a projection lemma
//!   (`lemma_project_to_runnable_boundary`) is provided in the proof file
//!   that extracts `(pid, interrupted_ids, zombie_ids)` matching the runnable
//!   module's boundary shape with all relevant invariants proven. Integration
//!   proofs must construct the runnable module's boundary type from this tuple.
//! - Thread state transitions (resume()) are ID-preserving.
//!   **Per-thread state mutation trust gap:** In the original
//!   `InterruptedThread::resume()`, `self.state.set_interrupt_reason(self.reason)`
//!   stores the interrupt reason into the thread's `ThreadState` before
//!   conversion to `ReadyThread`. This per-thread mutation is NOT modeled
//!   because threads are abstracted to integer IDs in this module.
//!   `spec_resume_reason_integration_obligation` defines the formal contract
//!   that the thread module's verification must establish. If downstream code
//!   relies on `interrupt_reason` being set, the thread module's proof must
//!   discharge this obligation (see `src/kernel/src/pm/thread/interrupted.rs`).
//! - `resume()` takes `admission_time: u64` as an oracle parameter.
//!   In the original code, admission time is set by `clock::now()` inside
//!   `ReadyThread::from_state()`. Since the clock is a HAL boundary outside
//!   this module's scope, the oracle pattern with caller-side obligations is
//!   used (consistent with `sleeping.rs`'s `wakeup_alarm` oracle approach).
//!   **Enforcement:** `resume_with_valid_clock()` is a verified wrapper that
//!   requires `spec_admission_time_valid()` in its precondition, enforcing the
//!   clock link at call sites that have access to the clock state. The base
//!   `resume()` accepts any `u64` value (non-negativity guaranteed by type).
//!   The postcondition of `resume_with_valid_clock`
//!   guarantees `spec_admission_time_valid(admission_times[0], clock_state)`.
//! - `find_thread()` / `find_thread_mut()` are **spec-level models** — they
//!   compute `spec_find_thread()` directly and do not model the executable
//!   search implementation. The original code performs linear searches through
//!   `iter().find(...)` across three collections with priority order
//!   (interrupted → sleeping → zombie). The spec captures this search order.
//!   **Trust gap:** The executable iterator-based search is NOT verified. Any
//!   bug in the real search (e.g., wrong predicate, wrong collection order)
//!   would not be caught. This is a fundamental Verus limitation: reference-typed
//!   return values (`Option<ThreadRef<'_>>`) cannot be expressed in Verus, and
//!   ghost sequences have no executable counterpart to iterate over. The
//!   `lemma_find_thread_refinement_assumption` documents the semantic equivalence
//!   assumption and its scope. `spec_find_thread_integration_obligation` defines
//!   the formal contract that integration proofs must discharge;
//!   `lemma_find_thread_obligation_implies_consistency` and
//!   `lemma_find_thread_result_unique` prove that once the obligation is met,
//!   the result is unambiguous under `wf()`. If Verus adds support for
//!   reference-typed returns or executable ghost iteration, this should be
//!   replaced with a verified implementation. Tagged for trust-boundary inventory.
//! - `state()` / `state_mut()` return the PID directly. The original returns
//!   `&ProcessState` / `&mut ProcessState`; since ProcessState is abstracted to
//!   PID, the frame condition holds trivially.
//!   **ProcessState abstraction gap:** There is no verified link between the
//!   concrete PID field and the real `ProcessState`'s internal PID. The model
//!   assumes the PID accurately reflects the real state.
//!   `spec_process_state_pid_integration_obligation` defines the formal
//!   contract: `pid == real_process_state.pid()`. This must be
//!   established at construction (`new()`/`from_sleeping()`) and is preserved
//!   by all operations (proven by PID-preservation postconditions).
//!   `lemma_pid_obligation_preserved_by_resume` formally proves preservation
//!   through `resume()`. If `ProcessState` becomes independently verifiable,
//!   the obligation should be discharged at construction sites.
//!
//! ## Fields
//!
//! All struct fields are `pub` for Verus proof ergonomics. The original has
//! private fields with getter/setter methods.

use vstd::prelude::*;

// Include specifications.
include!("interrupted.spec.rs");

// Include proofs.
include!("interrupted.proof.rs");

verus! {

//==================================================================================================
// Structures
//==================================================================================================

/// A process that was interrupted.
///
/// Verification model of `src/kernel/src/pm/process/state/interrupted.rs::InterruptedProcess`.
pub struct InterruptedProcess {
    /// Process identifier (from the inner ProcessState).
    pub pid: u64,
    /// Sleeping thread IDs (may be empty).
    pub sleeping_thread_ids: Vec<u64>,
    /// Interrupted thread IDs (non-empty).
    pub interrupted_thread_ids: Vec<u64>,
    /// Zombie thread IDs (may be empty).
    pub zombie_thread_ids: Vec<u64>,
}

/// A process that is ready to run (boundary model).
///
/// Models `RunnableProcess` from the sibling module.
pub struct RunnableProcess {
    /// Process identifier.
    pub pid: u64,
    /// Ready thread IDs (non-empty).
    pub ready_thread_ids: Vec<u64>,
    /// Ready thread admission times, parallel to ready_thread_ids.
    pub ready_admission_times: Vec<u64>,
    /// Interrupted thread IDs (may be empty).
    pub interrupted_thread_ids: Vec<u64>,
    /// Sleeping thread IDs (may be empty).
    pub sleeping_thread_ids: Vec<u64>,
    /// Zombie thread IDs (may be empty).
    pub zombie_thread_ids: Vec<u64>,
}

//==================================================================================================
// InterruptedProcess Implementation
//==================================================================================================

impl InterruptedProcess {
    /// Creates a new InterruptedProcess (without sleeping threads).
    ///
    /// Models the original `InterruptedProcess::new(state, interrupted_threads, zombie_threads)`.
    ///
    /// # Parameters
    ///
    /// - `pid`: Process identifier.
    /// - `interrupted_ids`: Interrupted thread IDs (must be non-empty).
    /// - `zombie_ids`: Zombie thread IDs (may be empty).
    ///
    /// # Returns
    ///
    /// A new, well-formed InterruptedProcess with no sleeping threads.
    pub fn new(
        pid: u64,
        interrupted_ids: Vec<u64>,
        zombie_ids: Vec<u64>,
    ) -> (result: InterruptedProcess)
        requires
            interrupted_ids@.len() >= 1,
            Self::spec_no_duplicates(interrupted_ids@),
            Self::spec_no_duplicates(zombie_ids@),
            Self::spec_seqs_disjoint(interrupted_ids@, zombie_ids@),
        ensures
            result.spec_pid() == pid,
            result.sleeping_thread_ids@.len() == 0,
            result.interrupted_thread_ids@ == interrupted_ids@,
            result.zombie_thread_ids@ == zombie_ids@,
            result.wf(),
    {
        InterruptedProcess {
            pid,
            sleeping_thread_ids: Vec::new(),
            interrupted_thread_ids: interrupted_ids,
            zombie_thread_ids: zombie_ids,
        }
    }

    /// Creates a new InterruptedProcess with sleeping threads.
    ///
    /// Models the original `InterruptedProcess::from_sleeping(state, sleeping_threads,
    /// interrupted_threads, zombie_threads)`.
    ///
    /// # Parameters
    ///
    /// - `pid`: Process identifier.
    /// - `sleeping_ids`: Sleeping thread IDs (may be empty).
    /// - `interrupted_ids`: Interrupted thread IDs (must be non-empty).
    /// - `zombie_ids`: Zombie thread IDs (may be empty).
    ///
    /// # Returns
    ///
    /// A new, well-formed InterruptedProcess.
    pub fn from_sleeping(
        pid: u64,
        sleeping_ids: Vec<u64>,
        interrupted_ids: Vec<u64>,
        zombie_ids: Vec<u64>,
    ) -> (result: InterruptedProcess)
        requires
            interrupted_ids@.len() >= 1,
            Self::spec_no_duplicates(interrupted_ids@),
            Self::spec_no_duplicates(sleeping_ids@),
            Self::spec_no_duplicates(zombie_ids@),
            Self::spec_seqs_disjoint(interrupted_ids@, sleeping_ids@),
            Self::spec_seqs_disjoint(interrupted_ids@, zombie_ids@),
            Self::spec_seqs_disjoint(sleeping_ids@, zombie_ids@),
        ensures
            result.spec_pid() == pid,
            result.sleeping_thread_ids@ == sleeping_ids@,
            result.interrupted_thread_ids@ == interrupted_ids@,
            result.zombie_thread_ids@ == zombie_ids@,
            result.wf(),
    {
        InterruptedProcess {
            pid,
            sleeping_thread_ids: sleeping_ids,
            interrupted_thread_ids: interrupted_ids,
            zombie_thread_ids: zombie_ids,
        }
    }

    /// Returns the process state (modeled as PID).
    ///
    /// Models the original `InterruptedProcess::state()`.
    ///
    /// # Returns
    ///
    /// The process identifier.
    pub fn state(&self) -> (result: u64)
        ensures
            result == self.spec_pid(),
    {
        self.pid
    }

    /// Returns a mutable reference to the process state.
    ///
    /// Models the original `InterruptedProcess::state_mut()`.
    /// In the real implementation this returns `&mut ProcessState`, allowing
    /// mutation of inner fields (e.g., capabilities). Since our model abstracts
    /// ProcessState to PID, the frame condition holds trivially.
    ///
    /// # Returns
    ///
    /// The process identifier.
    pub fn state_mut(&mut self) -> (result: u64)
        requires
            old(self).wf(),
        ensures
            result == self.spec_pid(),
            self.spec_pid() == old(self).spec_pid(),
            self.interrupted_thread_ids@ == old(self).interrupted_thread_ids@,
            self.sleeping_thread_ids@ == old(self).sleeping_thread_ids@,
            self.zombie_thread_ids@ == old(self).zombie_thread_ids@,
            self.wf(),
    {
        self.pid
    }

    /// Resumes the first interrupted thread and transitions to RunnableProcess.
    ///
    /// Models the original `InterruptedProcess::resume()`.
    /// Pops the front element from the interrupted thread list, converts it
    /// to a ready thread (ID-preserving), and creates a RunnableProcess with
    /// that thread as the only ready thread. Remaining interrupted threads
    /// become the interrupted list, and sleeping/zombie threads are preserved.
    ///
    /// ## Per-Thread State Mutation Trust Gap
    ///
    /// In the original `InterruptedThread::resume()`, `self.state.set_interrupt_reason(self.reason)`
    /// stores the interrupt reason into the thread's `ThreadState` before conversion
    /// to `ReadyThread`. This mutation is NOT modeled here because threads are
    /// abstracted to integer IDs. The interrupt_reason propagation property must
    /// be verified in the thread module's own Verus verification.
    ///
    /// ## Oracle Parameter
    ///
    /// `admission_time`: The admission time assigned to the resumed thread.
    /// In the original code, this is `clock::now()` inside `ReadyThread::from_state()`.
    /// Non-negativity is guaranteed by the `u64` type. Callers must establish
    /// equivalence with the real clock via `spec_admission_time_valid()` at the
    /// integration level. The postcondition records the oracle value in
    /// `ready_admission_times[0]` so downstream consumers can reason about it.
    ///
    /// # Parameters
    ///
    /// - `admission_time`: Admission time for the resumed thread.
    ///
    /// # Returns
    ///
    /// A RunnableProcess with the resumed thread as the only ready thread.
    pub fn resume(self, admission_time: u64) -> (result: RunnableProcess)
        requires
            self.wf(),
        ensures
            result.spec_pid() == self.spec_pid(),
            result.wf(),
            // Exactly one ready thread: the front interrupted thread.
            result.ready_thread_ids@.len() == 1,
            result.ready_thread_ids@[0] == self.interrupted_thread_ids@[0],
            // Admission time matches the oracle parameter.
            result.ready_admission_times@.len() == 1,
            result.ready_admission_times@[0] == admission_time,
            // Remaining interrupted threads (tail of original list).
            result.interrupted_thread_ids@ ==
                self.interrupted_thread_ids@.subrange(1, self.interrupted_thread_ids@.len() as int),
            result.interrupted_thread_ids@.len() == self.spec_interrupted_count() - 1,
            // Sleeping threads preserved.
            result.sleeping_thread_ids@ == self.sleeping_thread_ids@,
            // Zombie threads preserved.
            result.zombie_thread_ids@ == self.zombie_thread_ids@,
    {
        // Destructure self to work with individual fields.
        let pid: u64 = self.pid;
        let mut interrupted: Vec<u64> = self.interrupted_thread_ids;
        let sleeping: Vec<u64> = self.sleeping_thread_ids;
        let zombie: Vec<u64> = self.zombie_thread_ids;
        let ghost old_interrupted: Seq<u64> = interrupted@;

        // Pop front of interrupted list.
        let front_tid: u64 = interrupted.remove(0);

        // Build ready list (singleton).
        let mut ready: Vec<u64> = Vec::new();
        ready.push(front_tid);

        // Build admission times list (singleton).
        let mut admit_times: Vec<u64> = Vec::new();
        admit_times.push(admission_time);

        proof {
            // Connect Vec::remove(0) with subrange(1, len).
            assert(old_interrupted.remove(0int) =~=
                old_interrupted.subrange(1, old_interrupted.len() as int));

            let ready_spec: Seq<u64> = ready@;
            assert(ready_spec.len() == 1);
            assert(ready_spec[0] == front_tid);

            // Prove no-duplicates on the tail of the interrupted list.
            Self::lemma_subrange_preserves_no_duplicates(old_interrupted);

            // Prove the front element is not in the tail.
            Self::lemma_front_not_in_tail(old_interrupted);

            // Prove tail of interrupted is disjoint from sleeping and zombie.
            Self::lemma_tail_disjoint_sleeping(old_interrupted, sleeping@);
            Self::lemma_tail_disjoint_zombie(old_interrupted, zombie@);

            // Prove ready (singleton) is no-duplicates trivially.
            assert(InterruptedProcess::spec_no_duplicates(ready_spec)) by {
                assert forall|i: int, j: int| 0 <= i < j < ready_spec.len()
                    implies ready_spec[i] != ready_spec[j]
                by {
                    // ready_spec.len() == 1, so no i < j pair exists.
                }
            }

            // Prove ready is disjoint from remaining interrupted.
            assert(InterruptedProcess::spec_seqs_disjoint(ready_spec, interrupted@)) by {
                assert forall|i: int, j: int|
                    0 <= i < ready_spec.len() && 0 <= j < interrupted@.len()
                    implies ready_spec[i] != interrupted@[j]
                by {
                    assert(ready_spec[i] == front_tid);
                    assert(interrupted@[j] == old_interrupted[j + 1]);
                }
            }

            // Prove ready is disjoint from sleeping.
            assert(InterruptedProcess::spec_seqs_disjoint(ready_spec, sleeping@)) by {
                assert forall|i: int, j: int|
                    0 <= i < ready_spec.len() && 0 <= j < sleeping@.len()
                    implies ready_spec[i] != sleeping@[j]
                by {
                    assert(ready_spec[i] == front_tid);
                    assert(0 <= 0int < old_interrupted.len());
                    assert(old_interrupted[0] == front_tid);
                }
            }

            // Prove ready is disjoint from zombie.
            assert(InterruptedProcess::spec_seqs_disjoint(ready_spec, zombie@)) by {
                assert forall|i: int, j: int|
                    0 <= i < ready_spec.len() && 0 <= j < zombie@.len()
                    implies ready_spec[i] != zombie@[j]
                by {
                    assert(ready_spec[i] == front_tid);
                    assert(0 <= 0int < old_interrupted.len());
                    assert(old_interrupted[0] == front_tid);
                }
            }

            // Admission times: singleton matching oracle.
            let admit_spec: Seq<u64> = admit_times@;
            assert(admit_spec.len() == 1);
            assert(admit_spec[0] == admission_time);
        }

        RunnableProcess {
            pid,
            ready_thread_ids: ready,
            ready_admission_times: admit_times,
            interrupted_thread_ids: interrupted,
            sleeping_thread_ids: sleeping,
            zombie_thread_ids: zombie,
        }
    }

    /// Verified wrapper: resumes with a clock-validated admission time.
    ///
    /// **Verification-only helper** — this function does NOT exist in the
    /// original source (`src/kernel/src/pm/process/state/interrupted.rs`).
    /// It is provided as a stronger entry point for integration proofs that
    /// have access to the clock state, requiring `spec_admission_time_valid()`
    /// to enforce the link to `clock::now()`. Delegates to `resume()`.
    ///
    /// Use this instead of `resume()` when the caller can provide a
    /// verified clock state. The postconditions are identical to `resume()`
    /// plus the admission time validity guarantee.
    ///
    /// # Parameters
    ///
    /// - `admission_time`: Admission time (must be clock-validated).
    /// - `clock_state`: Ghost abstract clock state at the call site.
    ///
    /// # Returns
    ///
    /// A RunnableProcess with clock-validated admission time.
    pub fn resume_with_valid_clock(
        self, admission_time: u64, clock_state: Ghost<int>,
    ) -> (result: RunnableProcess)
        requires
            self.wf(),
            Self::spec_admission_time_valid(admission_time as int, clock_state@),
        ensures
            result.spec_pid() == self.spec_pid(),
            result.wf(),
            result.ready_thread_ids@.len() == 1,
            result.ready_thread_ids@[0] == self.interrupted_thread_ids@[0],
            result.ready_admission_times@.len() == 1,
            result.ready_admission_times@[0] == admission_time,
            Self::spec_admission_time_valid(
                result.ready_admission_times@[0] as int, clock_state@),
            result.interrupted_thread_ids@ ==
                self.interrupted_thread_ids@.subrange(
                    1, self.interrupted_thread_ids@.len() as int),
            result.interrupted_thread_ids@.len() == self.spec_interrupted_count() - 1,
            result.sleeping_thread_ids@ == self.sleeping_thread_ids@,
            result.zombie_thread_ids@ == self.zombie_thread_ids@,
    {
        self.resume(admission_time)
    }

    /// Finds a thread by its identifier and returns which list it belongs to.
    ///
    /// **Spec-level model** of the original `InterruptedProcess::find_thread(tid)`.
    /// This function computes `spec_find_thread()` directly in ghost mode.
    /// It does NOT model the executable `iter().find(...)` search — see
    /// `lemma_find_thread_refinement_assumption` for the trust assumption
    /// and `spec_find_thread_integration_obligation` for the formal contract
    /// that integration proofs must discharge.
    ///
    /// Under `wf()`, `lemma_find_thread_result_unique` proves that each
    /// thread ID appears in at most one list, so the spec result is
    /// deterministic. This is the strongest guarantee achievable without
    /// executable verification of the iterator logic (Verus limitation:
    /// cannot express reference-typed returns or iterate ghost sequences).
    ///
    /// Returns the abstract list variant:
    /// - `Some(0)`: interrupted thread.
    /// - `Some(1)`: sleeping thread.
    /// - `Some(2)`: zombie thread.
    /// - `None`: not found.
    ///
    /// # Parameters
    ///
    /// - `tid`: Thread identifier to search for.
    ///
    /// # Returns
    ///
    /// The ghost list variant.
    pub fn find_thread(&self, tid: u64) -> (result: Ghost<Option<int>>)
        ensures
            result@ == self.spec_find_thread(tid),
    {
        Ghost(self.spec_find_thread(tid))
    }

    /// Finds a thread by its identifier (mutable variant).
    ///
    /// **Spec-level model** of the original `InterruptedProcess::find_thread_mut(tid)`.
    /// Same trust scope as `find_thread()` — see its documentation.
    /// Frame condition: self is unchanged.
    ///
    /// # Parameters
    ///
    /// - `tid`: Thread identifier to search for.
    ///
    /// # Returns
    ///
    /// The ghost list variant.
    pub fn find_thread_mut(&mut self, tid: u64) -> (result: Ghost<Option<int>>)
        requires
            old(self).wf(),
        ensures
            result@ == old(self).spec_find_thread(tid),
            self.spec_pid() == old(self).spec_pid(),
            self.interrupted_thread_ids@ == old(self).interrupted_thread_ids@,
            self.sleeping_thread_ids@ == old(self).sleeping_thread_ids@,
            self.zombie_thread_ids@ == old(self).zombie_thread_ids@,
            self.wf(),
    {
        Ghost(old(self).spec_find_thread(tid))
    }
}

//==================================================================================================
// Standalone Function
//==================================================================================================

/// Converts a sleeping thread to an interrupted thread (ID-preserving).
///
/// Models the standalone `interrupt()` function from the original source.
/// The original always passes `InterruptReason::Killed`. The thread's
/// identity is preserved through the state transition.
///
/// # Parameters
///
/// - `sleeping_tid`: Thread identifier of the sleeping thread.
///
/// # Returns
///
/// A tuple of the same thread identifier and the interrupt reason
/// (`INTERRUPT_REASON_KILLED`), modeling the ID-preserving transition.
pub fn interrupt(sleeping_tid: u64) -> (result: (u64, u64))
    ensures
        result.0 == sleeping_tid,
        result.1 as int == InterruptedProcess::INTERRUPT_REASON_KILLED(),
{
    (sleeping_tid, 0u64)
}

} // verus!
