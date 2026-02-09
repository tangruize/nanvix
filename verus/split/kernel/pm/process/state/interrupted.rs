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
//! - `Box<ProcessState>` -> PID (int, identity tracking only).
//! - `NonEmptyVecDeque<InterruptedThread>` -> `Seq<int>` of thread IDs (ghost).
//! - `Option<NonEmptyVecDeque<SleepingThread>>` -> `Seq<int>` (empty = None).
//! - `Option<NonEmptyVecDeque<ZombieThread>>` -> `Seq<int>` (empty = None).
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
//!   because threads are abstracted to integer IDs in this module. If
//!   downstream code relies on `interrupt_reason` being set in the ready
//!   thread's state, that property must be verified in the thread module's
//!   own verification (see `src/kernel/src/pm/thread/interrupted.rs`).
//! - `resume()` takes `admission_time: Ghost<int>` as an oracle parameter.
//!   In the original code, admission time is set by `clock::now()` inside
//!   `ReadyThread::from_state()`. Since the clock is a HAL boundary outside
//!   this module's scope, the oracle pattern with caller-side obligations is
//!   used (consistent with `sleeping.rs`'s `wakeup_alarm` oracle approach).
//!   The `spec_admission_time_valid()` predicate defines the caller contract;
//!   `lemma_valid_admission_time_satisfies_resume_precondition` proves that
//!   a valid oracle satisfies the `resume()` precondition. The link to real
//!   `clock::now()` must be established at the integration proof level.
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
//!   assumption and its scope. If Verus adds support for reference-typed returns
//!   or executable ghost iteration, this should be replaced with a verified
//!   implementation. Tagged for trust-boundary inventory.
//! - `state()` / `state_mut()` are implemented as pure ghost returns (no
//!   external_body). The original returns `&ProcessState` / `&mut ProcessState`;
//!   since ProcessState is abstracted to PID and all fields are ghost, the
//!   frame condition holds trivially without trusted assumptions.
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
    pub pid: Ghost<int>,
    /// Ghost sequence of sleeping thread IDs (may be empty).
    pub sleeping_thread_ids: Ghost<Seq<int>>,
    /// Ghost sequence of interrupted thread IDs (non-empty).
    pub interrupted_thread_ids: Ghost<Seq<int>>,
    /// Ghost sequence of zombie thread IDs (may be empty).
    pub zombie_thread_ids: Ghost<Seq<int>>,
}

/// A process that is ready to run (boundary model).
///
/// Models `RunnableProcess` from the sibling module.
pub struct RunnableProcess {
    /// Process identifier.
    pub pid: Ghost<int>,
    /// Ready thread IDs (non-empty).
    pub ready_thread_ids: Ghost<Seq<int>>,
    /// Ready thread admission times, parallel to ready_thread_ids.
    pub ready_admission_times: Ghost<Seq<int>>,
    /// Interrupted thread IDs (may be empty).
    pub interrupted_thread_ids: Ghost<Seq<int>>,
    /// Sleeping thread IDs (may be empty).
    pub sleeping_thread_ids: Ghost<Seq<int>>,
    /// Zombie thread IDs (may be empty).
    pub zombie_thread_ids: Ghost<Seq<int>>,
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
    /// - `interrupted_ids`: Ghost interrupted thread IDs (must be non-empty).
    /// - `zombie_ids`: Ghost zombie thread IDs (may be empty).
    ///
    /// # Returns
    ///
    /// A new, well-formed InterruptedProcess with no sleeping threads.
    pub fn new(
        pid: Ghost<int>,
        interrupted_ids: Ghost<Seq<int>>,
        zombie_ids: Ghost<Seq<int>>,
    ) -> (result: InterruptedProcess)
        requires
            interrupted_ids@.len() >= 1,
            Self::spec_no_duplicates(interrupted_ids@),
            Self::spec_no_duplicates(zombie_ids@),
            Self::spec_seqs_disjoint(interrupted_ids@, zombie_ids@),
        ensures
            result.spec_pid() == pid@,
            result.sleeping_thread_ids@.len() == 0,
            result.interrupted_thread_ids@ == interrupted_ids@,
            result.zombie_thread_ids@ == zombie_ids@,
            result.wf(),
    {
        InterruptedProcess {
            pid,
            sleeping_thread_ids: Ghost(Seq::empty()),
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
    /// - `sleeping_ids`: Ghost sleeping thread IDs (may be empty).
    /// - `interrupted_ids`: Ghost interrupted thread IDs (must be non-empty).
    /// - `zombie_ids`: Ghost zombie thread IDs (may be empty).
    ///
    /// # Returns
    ///
    /// A new, well-formed InterruptedProcess.
    pub fn from_sleeping(
        pid: Ghost<int>,
        sleeping_ids: Ghost<Seq<int>>,
        interrupted_ids: Ghost<Seq<int>>,
        zombie_ids: Ghost<Seq<int>>,
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
            result.spec_pid() == pid@,
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
    pub fn state(&self) -> (result: Ghost<int>)
        ensures
            result@ == self.spec_pid(),
    {
        Ghost(self.pid@)
    }

    /// Returns a mutable reference to the process state.
    ///
    /// Models the original `InterruptedProcess::state_mut()`.
    /// In the real implementation this returns `&mut ProcessState`, allowing
    /// mutation of inner fields (e.g., capabilities). Since our model abstracts
    /// ProcessState to PID and all fields are ghost, the frame condition holds
    /// trivially — nothing is mutated.
    ///
    /// # Returns
    ///
    /// The process identifier (as a ghost value).
    pub fn state_mut(&mut self) -> (result: Ghost<int>)
        requires
            old(self).wf(),
        ensures
            result@ == self.spec_pid(),
            self.spec_pid() == old(self).spec_pid(),
            self.interrupted_thread_ids@ == old(self).interrupted_thread_ids@,
            self.sleeping_thread_ids@ == old(self).sleeping_thread_ids@,
            self.zombie_thread_ids@ == old(self).zombie_thread_ids@,
            self.wf(),
    {
        Ghost(self.pid@)
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
    /// The oracle must be non-negative. Callers must establish equivalence with
    /// the real clock via `spec_admission_time_valid()` at the integration level.
    /// The postcondition records the oracle value in `ready_admission_times[0]`
    /// so downstream consumers can reason about the timestamp.
    ///
    /// # Parameters
    ///
    /// - `admission_time`: Ghost admission time for the resumed thread.
    ///
    /// # Returns
    ///
    /// A RunnableProcess with the resumed thread as the only ready thread.
    pub fn resume(self, admission_time: Ghost<int>) -> (result: RunnableProcess)
        requires
            self.wf(),
            admission_time@ >= 0,
        ensures
            result.spec_pid() == self.spec_pid(),
            result.wf(),
            // Exactly one ready thread: the front interrupted thread.
            result.ready_thread_ids@.len() == 1,
            result.ready_thread_ids@[0] == self.interrupted_thread_ids@[0],
            // Admission time matches the oracle parameter.
            result.ready_admission_times@.len() == 1,
            result.ready_admission_times@[0] == admission_time@,
            // Remaining interrupted threads (tail of original list).
            result.interrupted_thread_ids@ ==
                self.interrupted_thread_ids@.subrange(1, self.interrupted_thread_ids@.len() as int),
            result.interrupted_thread_ids@.len() == self.spec_interrupted_count() - 1,
            // Sleeping threads preserved.
            result.sleeping_thread_ids@ == self.sleeping_thread_ids@,
            // Zombie threads preserved.
            result.zombie_thread_ids@ == self.zombie_thread_ids@,
    {
        let ghost front_tid: int = self.interrupted_thread_ids@[0];
        let ghost remaining: Seq<int> =
            self.interrupted_thread_ids@.subrange(1, self.interrupted_thread_ids@.len() as int);

        proof {
            // The ready list has exactly one element.
            let ready: Seq<int> = Seq::<int>::empty().push(front_tid);
            assert(ready.len() == 1);
            assert(ready[0] == front_tid);

            // The remaining interrupted list has length - 1.
            assert(remaining.len() == (self.interrupted_thread_ids@.len() - 1) as nat);

            // Prove no-duplicates on the tail of the interrupted list.
            Self::lemma_subrange_preserves_no_duplicates(self.interrupted_thread_ids@);

            // Prove the front element is not in the tail.
            Self::lemma_front_not_in_tail(self.interrupted_thread_ids@);

            // Prove tail of interrupted is disjoint from sleeping and zombie.
            Self::lemma_tail_disjoint_sleeping(
                self.interrupted_thread_ids@, self.sleeping_thread_ids@);
            Self::lemma_tail_disjoint_zombie(
                self.interrupted_thread_ids@, self.zombie_thread_ids@);

            // Prove ready (singleton) is no-duplicates trivially.
            assert(InterruptedProcess::spec_no_duplicates(ready)) by {
                assert forall|i: int, j: int| 0 <= i < j < ready.len()
                    implies ready[i] != ready[j]
                by {
                    // ready.len() == 1, so no i < j pair exists.
                }
            }

            // Prove ready is disjoint from remaining interrupted.
            assert(InterruptedProcess::spec_seqs_disjoint(ready, remaining)) by {
                assert forall|i: int, j: int|
                    0 <= i < ready.len() && 0 <= j < remaining.len()
                    implies ready[i] != remaining[j]
                by {
                    assert(ready[i] == front_tid);
                    assert(remaining[j] == self.interrupted_thread_ids@[j + 1]);
                }
            }

            // Prove ready is disjoint from sleeping.
            assert(InterruptedProcess::spec_seqs_disjoint(ready, self.sleeping_thread_ids@)) by {
                assert forall|i: int, j: int|
                    0 <= i < ready.len() && 0 <= j < self.sleeping_thread_ids@.len()
                    implies ready[i] != self.sleeping_thread_ids@[j]
                by {
                    assert(ready[i] == front_tid);
                    assert(0 <= 0int < self.interrupted_thread_ids@.len());
                    assert(self.interrupted_thread_ids@[0] == front_tid);
                }
            }

            // Prove ready is disjoint from zombie.
            assert(InterruptedProcess::spec_seqs_disjoint(ready, self.zombie_thread_ids@)) by {
                assert forall|i: int, j: int|
                    0 <= i < ready.len() && 0 <= j < self.zombie_thread_ids@.len()
                    implies ready[i] != self.zombie_thread_ids@[j]
                by {
                    assert(ready[i] == front_tid);
                    assert(0 <= 0int < self.interrupted_thread_ids@.len());
                    assert(self.interrupted_thread_ids@[0] == front_tid);
                }
            }

            // Admission times: singleton matching oracle.
            let admit: Seq<int> = Seq::<int>::empty().push(admission_time@);
            assert(admit.len() == 1);
            assert(admit[0] == admission_time@);
            assert(admit[0] >= 0);
        }

        RunnableProcess {
            pid: Ghost(self.pid@),
            ready_thread_ids: Ghost(Seq::<int>::empty().push(front_tid)),
            ready_admission_times: Ghost(Seq::<int>::empty().push(admission_time@)),
            interrupted_thread_ids: Ghost(remaining),
            sleeping_thread_ids: Ghost(self.sleeping_thread_ids@),
            zombie_thread_ids: Ghost(self.zombie_thread_ids@),
        }
    }

    /// Finds a thread by its identifier and returns which list it belongs to.
    ///
    /// **Spec-level model** of the original `InterruptedProcess::find_thread(tid)`.
    /// This function computes `spec_find_thread()` directly in ghost mode.
    /// It does NOT model the executable `iter().find(...)` search — see
    /// `lemma_find_thread_refinement_assumption` for the trust assumption
    /// connecting this spec to the real implementation.
    ///
    /// Returns the abstract list variant:
    /// - `Some(0)`: interrupted thread.
    /// - `Some(1)`: sleeping thread.
    /// - `Some(2)`: zombie thread.
    /// - `None`: not found.
    ///
    /// # Parameters
    ///
    /// - `tid`: Ghost thread identifier to search for.
    ///
    /// # Returns
    ///
    /// The ghost list variant.
    pub fn find_thread(&self, tid: Ghost<int>) -> (result: Ghost<Option<int>>)
        ensures
            result@ == self.spec_find_thread(tid@),
    {
        Ghost(self.spec_find_thread(tid@))
    }

    /// Finds a thread by its identifier (mutable variant).
    ///
    /// **Spec-level model** of the original `InterruptedProcess::find_thread_mut(tid)`.
    /// Same trust scope as `find_thread()` — see its documentation.
    /// Frame condition: self is unchanged.
    ///
    /// # Parameters
    ///
    /// - `tid`: Ghost thread identifier to search for.
    ///
    /// # Returns
    ///
    /// The ghost list variant.
    pub fn find_thread_mut(&mut self, tid: Ghost<int>) -> (result: Ghost<Option<int>>)
        requires
            old(self).wf(),
        ensures
            result@ == old(self).spec_find_thread(tid@),
            self.spec_pid() == old(self).spec_pid(),
            self.interrupted_thread_ids@ == old(self).interrupted_thread_ids@,
            self.sleeping_thread_ids@ == old(self).sleeping_thread_ids@,
            self.zombie_thread_ids@ == old(self).zombie_thread_ids@,
            self.wf(),
    {
        Ghost(old(self).spec_find_thread(tid@))
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
/// - `sleeping_tid`: Ghost thread identifier of the sleeping thread.
///
/// # Returns
///
/// A tuple of the same thread identifier and the interrupt reason
/// (`INTERRUPT_REASON_KILLED`), modeling the ID-preserving transition.
pub fn interrupt(sleeping_tid: Ghost<int>) -> (result: (Ghost<int>, Ghost<int>))
    ensures
        result.0@ == sleeping_tid@,
        result.1@ == InterruptedProcess::INTERRUPT_REASON_KILLED(),
{
    (sleeping_tid, Ghost(InterruptedProcess::INTERRUPT_REASON_KILLED()))
}

} // verus!
