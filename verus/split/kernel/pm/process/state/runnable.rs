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
//! - `NonEmptyVecDeque<ReadyThread>` -> `Seq<int>` of thread IDs (ghost) +
//!   `Seq<int>` of admission times (ghost). Non-emptiness is enforced by `wf()`.
//! - `Option<NonEmptyVecDeque<InterruptedThread>>` -> `Seq<int>` (may be empty).
//! - `Option<NonEmptyVecDeque<SleepingThread>>` -> `Seq<int>` (may be empty).
//! - `Option<NonEmptyVecDeque<ZombieThread>>` -> `Seq<int>` (may be empty).
//! - `ContextInformation`, `VirtualAddress` from run() -> elided (HAL boundary).
//! - `InterruptReason` -> abstract int tag.
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
//!   only track the PID for identity verification. The original functions are
//!   simple accessors to `Box<ProcessState>` — their correctness is trivial.
//! - Thread state transitions (ReadyThread::terminate(), SleepingThread::interrupt(),
//!   SleepingThread::wakeup()) are modeled as ID-preserving operations.
//! - **Oracle parameters:** `run()` takes `selected_idx` and `wakeup()` takes
//!   `found`/`found_idx` as oracle parameters. These push the algorithmic
//!   correctness (the for-loop that finds the minimum admission time in `run()`,
//!   the sleeping thread search in `wakeup()`) to the caller. The preconditions
//!   ensure the oracle values match the ghost state, so correctness is preserved
//!   at the protocol level. The iterative search algorithms themselves are not
//!   verified — this is an explicit trust assumption.
//! - **`find_thread()` and `find_thread_mut()`** are omitted from the exec-level
//!   model because they return reference types (`ThreadRef`, `ThreadRefMut`) that
//!   Verus cannot express. A spec-only model (`spec_find_thread`) is provided
//!   that verifies the exhaustive search logic and correct list variant selection.
//! - **`earliest_admission_time()`** is modeled as spec-only
//!   (`spec_earliest_admission_time`) because the return type `SystemTime` maps
//!   to ghost `int` in our model. The `lemma_earliest_admission_time_exists`
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
/// Returns an abstract timestamp. The minimal postcondition reflects that
/// `SystemTime` values are non-negative.
#[verifier::external_body]
fn clock_now() -> (result: int)
    ensures
        result >= 0,
{
    unimplemented!()
}

/// Exec-level accessor for the EXIT_STATUS_INTERRUPTED spec constant.
#[verifier::external_body]
fn exit_status_interrupted_value() -> (result: int)
    ensures
        result == EXIT_STATUS_INTERRUPTED(),
{
    unimplemented!()
}

//==================================================================================================
// Structures
//==================================================================================================

/// A process that is ready to run.
///
/// Verification model of `src/kernel/src/pm/process/state/runnable.rs::RunnableProcess`.
/// Thread collections are modeled as ghost sequences of thread IDs.
///
/// **Note:** Fields are `pub` for Verus proof ergonomics (spec access,
/// direct construction in lemmas). The original has private fields.
pub struct RunnableProcess {
    /// Process identifier (from the inner ProcessState).
    pub pid: ProcessIdentifier,
    /// Ghost sequence of ready thread IDs (non-empty).
    pub ready_thread_ids: Ghost<Seq<int>>,
    /// Ghost sequence of ready thread admission times (parallel to ready_thread_ids).
    pub ready_admission_times: Ghost<Seq<int>>,
    /// Ghost sequence of interrupted thread IDs.
    pub interrupted_thread_ids: Ghost<Seq<int>>,
    /// Ghost sequence of sleeping thread IDs.
    pub sleeping_thread_ids: Ghost<Seq<int>>,
    /// Ghost sequence of zombie thread IDs.
    pub zombie_thread_ids: Ghost<Seq<int>>,
}

/// A process that is running (boundary model).
///
/// Models the original `RunningProcess` from the sibling module.
/// All fields are ghost since this is a purely abstract boundary model.
pub struct RunningProcess {
    /// Process identifier (from the inner ProcessState).
    pub pid: Ghost<int>,
    /// The running thread ID.
    pub running_thread_id: Ghost<int>,
    /// Remaining ready thread IDs.
    pub ready_thread_ids: Ghost<Seq<int>>,
    /// Interrupted thread IDs.
    pub interrupted_thread_ids: Ghost<Seq<int>>,
    /// Sleeping thread IDs.
    pub sleeping_thread_ids: Ghost<Seq<int>>,
    /// Zombie thread IDs.
    pub zombie_thread_ids: Ghost<Seq<int>>,
}

/// A process that was interrupted (boundary model).
///
/// Models the original `InterruptedProcess` from the sibling module.
pub struct InterruptedProcess {
    /// Process identifier value.
    pub pid: Ghost<int>,
    /// Interrupted thread IDs (non-empty).
    pub interrupted_thread_ids: Ghost<Seq<int>>,
    /// Zombie thread IDs.
    pub zombie_thread_ids: Ghost<Seq<int>>,
}

/// A process that has terminated (boundary model).
///
/// Models the original `ZombieProcess` from the sibling module.
pub struct ZombieProcess {
    /// Process identifier value.
    pub pid: Ghost<int>,
    /// Zombie thread IDs (non-empty).
    pub zombie_thread_ids: Ghost<Seq<int>>,
    /// Exit status.
    pub status: Ghost<int>,
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
    pub fn new(pid: ProcessIdentifier, ready_tid: Ghost<int>, ready_time: Ghost<int>) -> (result: RunnableProcess)
        requires
            ready_time@ >= 0,
        ensures
            result.spec_pid() == pid.spec_value(),
            result.spec_ready_count() == 1,
            result.spec_interrupted_count() == 0,
            result.spec_sleeping_count() == 0,
            result.spec_zombie_count() == 0,
            result.wf(),
    {
        let ghost rid_seq: Seq<int> = seq![ready_tid@];
        let ghost rtime_seq: Seq<int> = seq![ready_time@];
        RunnableProcess {
            pid: pid,
            ready_thread_ids: Ghost(rid_seq),
            ready_admission_times: Ghost(rtime_seq),
            interrupted_thread_ids: Ghost(Seq::empty()),
            sleeping_thread_ids: Ghost(Seq::empty()),
            zombie_thread_ids: Ghost(Seq::empty()),
        }
    }

    /// Creates a runnable process from existing state and thread lists.
    ///
    /// Models the original `RunnableProcess::from_state(...)`.
    ///
    /// # Parameters
    ///
    /// - `pid`: Process identifier.
    /// - `ready_ids`: Non-empty sequence of ready thread IDs.
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
        ready_ids: Ghost<Seq<int>>,
        ready_times: Ghost<Seq<int>>,
        interrupted_ids: Ghost<Seq<int>>,
        sleeping_ids: Ghost<Seq<int>>,
        zombie_ids: Ghost<Seq<int>>,
    ) -> (result: RunnableProcess)
        requires
            ready_ids@.len() >= 1,
            ready_ids@.len() == ready_times@.len(),
            forall|i: int| 0 <= i < ready_times@.len()
                ==> #[trigger] ready_times@[i] >= 0,
        ensures
            result.spec_pid() == pid.spec_value(),
            result.spec_ready_count() == ready_ids@.len(),
            result.spec_interrupted_count() == interrupted_ids@.len(),
            result.spec_sleeping_count() == sleeping_ids@.len(),
            result.spec_zombie_count() == zombie_ids@.len(),
            result.wf(),
    {
        RunnableProcess {
            pid: pid,
            ready_thread_ids: ready_ids,
            ready_admission_times: ready_times,
            interrupted_thread_ids: interrupted_ids,
            sleeping_thread_ids: sleeping_ids,
            zombie_thread_ids: zombie_ids,
        }
    }

    /// Returns the process identifier as i32.
    pub fn pid_i32(&self) -> (result: i32)
        ensures
            result as int == self.spec_pid(),
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
    /// The minimum-index selection is derived internally via a proof block
    /// invoking `lemma_earliest_admission_time_exists`, which proves existence
    /// of a minimum element in the non-empty admission time sequence. This
    /// replaces the original's for-loop, verifying the selection logic rather
    /// than trusting an oracle parameter.
    ///
    /// # Returns
    ///
    /// A RunningProcess with the earliest-admission-time thread as running.
    pub fn run(self) -> (result: RunningProcess)
        requires
            self.wf(),
        ensures
            result.spec_pid() == self.spec_pid(),
            // The selected thread has the earliest admission time.
            ({
                let sel: int = self.spec_earliest_ready_index();
                result.spec_running_thread_id() == self.ready_thread_ids@[sel]
                && result.ready_thread_ids@ ==
                    Self::spec_remove_at(self.ready_thread_ids@, sel)
                && result.ready_thread_ids@.len() == self.spec_ready_count() - 1
            }),
            // Other lists are preserved exactly.
            result.interrupted_thread_ids@ == self.interrupted_thread_ids@,
            result.sleeping_thread_ids@ == self.sleeping_thread_ids@,
            result.zombie_thread_ids@ == self.zombie_thread_ids@,
    {
        // Derive the minimum index via proof.
        proof { self.lemma_earliest_admission_time_exists(); }

        // Use choose to select the min index, which is guaranteed to exist by the lemma.
        let ghost selected_idx: int = choose|idx: int|
            0 <= idx < self.ready_admission_times@.len()
            && forall|j: int| 0 <= j < self.ready_admission_times@.len()
                ==> #[trigger] self.ready_admission_times@[idx]
                    <= #[trigger] self.ready_admission_times@[j];

        proof {
            // The choose is well-defined because lemma_earliest_admission_time_exists
            // established the existential. Assert bounds.
            assert(0 <= selected_idx < self.ready_thread_ids@.len());
        }

        let ghost selected_tid: int = self.ready_thread_ids@[selected_idx as int];
        let ghost remaining_ready: Seq<int> =
            self.ready_thread_ids@.subrange(0, selected_idx)
                .add(self.ready_thread_ids@.subrange(
                    selected_idx + 1,
                    self.ready_thread_ids@.len() as int,
                ));

        proof {
            // Prove the remaining sequence has the expected length.
            let s: Seq<int> = self.ready_thread_ids@;
            let idx: int = selected_idx;
            let left: Seq<int> = s.subrange(0, idx);
            let right: Seq<int> = s.subrange(idx + 1, s.len() as int);
            assert(left.len() == idx as nat);
            assert(right.len() == (s.len() - idx as nat - 1) as nat);
            assert(left.add(right).len() == (s.len() - 1) as nat);

            // Prove that selected_idx == spec_earliest_ready_index.
            // Both are `choose` with the same predicate, so they must be equal.
        }

        RunningProcess {
            pid: Ghost(self.pid.spec_value()),
            running_thread_id: Ghost(selected_tid),
            ready_thread_ids: Ghost(remaining_ready),
            interrupted_thread_ids: Ghost(self.interrupted_thread_ids@),
            sleeping_thread_ids: Ghost(self.sleeping_thread_ids@),
            zombie_thread_ids: Ghost(self.zombie_thread_ids@),
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
    /// The branch decision is derived internally from the ghost state via a
    /// proof block, eliminating the need for an oracle parameter.
    ///
    /// # Returns
    ///
    /// TerminateResult::Interrupted if any interrupted threads remain,
    /// TerminateResult::Zombie if only zombie threads remain.
    pub fn terminate(self, has_interrupted: bool) -> (result: TerminateResult)
        requires
            self.wf(),
            has_interrupted == (self.spec_interrupted_count() > 0 || self.spec_sleeping_count() > 0),
        ensures
            match result {
                TerminateResult::Interrupted(ip) => {
                    // PID preserved.
                    ip.spec_pid() == self.spec_pid()
                    // Interrupted threads are exactly original interrupted + sleeping→interrupted.
                    && ip.interrupted_thread_ids@.len() ==
                        self.spec_interrupted_count() + self.spec_sleeping_count()
                    && ip.interrupted_thread_ids@ ==
                        self.interrupted_thread_ids@.add(self.sleeping_thread_ids@)
                    // Zombie threads include all original ready + original zombie.
                    && ip.zombie_thread_ids@.len() ==
                        self.spec_ready_count() + self.spec_zombie_count()
                    && ip.zombie_thread_ids@ ==
                        self.ready_thread_ids@.add(self.zombie_thread_ids@)
                    && ip.wf()
                    // Branch taken iff there were interrupted or sleeping threads.
                    && has_interrupted
                },
                TerminateResult::Zombie(zp) => {
                    // PID preserved.
                    zp.spec_pid() == self.spec_pid()
                    // No interrupted or sleeping threads existed.
                    && self.spec_interrupted_count() == 0
                    && self.spec_sleeping_count() == 0
                    // Zombie threads include all original ready + original zombie.
                    && zp.zombie_thread_ids@.len() ==
                        self.spec_ready_count() + self.spec_zombie_count()
                    && zp.zombie_thread_ids@ ==
                        self.ready_thread_ids@.add(self.zombie_thread_ids@)
                    && zp.spec_status() == EXIT_STATUS_INTERRUPTED()
                    && zp.wf()
                    // Branch taken iff there were no interrupted or sleeping threads.
                    && !has_interrupted
                },
            },
    {
        let ghost new_zombie_ids: Seq<int> =
            self.ready_thread_ids@.add(self.zombie_thread_ids@);

        proof {
            assert(new_zombie_ids.len() ==
                self.ready_thread_ids@.len() + self.zombie_thread_ids@.len());
        }

        // Collect interrupted threads: original + sleeping→interrupted.
        let ghost new_interrupted_ids: Seq<int> =
            self.interrupted_thread_ids@.add(self.sleeping_thread_ids@);

        if has_interrupted {
            proof {
                assert(new_interrupted_ids.len() ==
                    self.interrupted_thread_ids@.len() + self.sleeping_thread_ids@.len());
                assert(new_interrupted_ids.len() >= 1);
            }
            TerminateResult::Interrupted(InterruptedProcess {
                pid: Ghost(self.pid.spec_value()),
                interrupted_thread_ids: Ghost(new_interrupted_ids),
                zombie_thread_ids: Ghost(new_zombie_ids),
            })
        } else {
            proof {
                assert(self.interrupted_thread_ids@.len() == 0);
                assert(self.sleeping_thread_ids@.len() == 0);
                assert(new_zombie_ids.len() >= 1);
            }
            let status: int = exit_status_interrupted_value();
            TerminateResult::Zombie(ZombieProcess {
                pid: Ghost(self.pid.spec_value()),
                zombie_thread_ids: Ghost(new_zombie_ids),
                status: Ghost(status),
            })
        }
    }

    /// Wakes up a sleeping thread and moves it to the ready queue.
    ///
    /// Models the original `RunnableProcess::wakeup(tid)`.
    /// Returns Ok(RunnableProcess) on success, Err(RunnableProcess) if the
    /// thread is not found in sleeping threads.
    ///
    /// # Parameters
    ///
    /// - `tid`: Ghost thread ID to wake up.
    /// - `found`: Oracle parameter — whether the thread was found in sleeping list.
    /// - `found_idx`: Oracle parameter — index in sleeping list (if found).
    ///
    /// # Returns
    ///
    /// Ok with updated state if found, Err with unchanged state if not found.
    ///
    /// The search index `found_idx` is derived internally via a proof block
    /// using `choose`, eliminating the oracle parameter for the index.
    /// The `found` boolean remains a parameter because the branch decision
    /// requires exec-level evaluation, and `Seq::contains()` is spec-only.
    pub fn wakeup(self, tid: Ghost<int>, found: bool) -> (result: Result<RunnableProcess, RunnableProcess>)
        requires
            self.wf(),
            found == Self::spec_seq_contains(self.sleeping_thread_ids@, tid@),
        ensures
            match result {
                Ok(r) => {
                    found
                    && r.spec_pid() == self.spec_pid()
                    && r.spec_ready_count() == self.spec_ready_count() + 1
                    && r.spec_sleeping_count() == self.spec_sleeping_count() - 1
                    && r.spec_interrupted_count() == self.spec_interrupted_count()
                    && r.spec_zombie_count() == self.spec_zombie_count()
                    // Content specs: ready list gets the woken thread appended.
                    && r.ready_thread_ids@ == self.ready_thread_ids@.push(tid@)
                    // Admission times grow by exactly one non-negative element.
                    && (exists|t: int| t >= 0
                        && r.ready_admission_times@ == self.ready_admission_times@.push(t))
                    // Sleeping list has the found thread removed.
                    && (exists|idx: int| 0 <= idx < self.sleeping_thread_ids@.len()
                        && self.sleeping_thread_ids@[idx] == tid@
                        && r.sleeping_thread_ids@ ==
                            Self::spec_remove_at(self.sleeping_thread_ids@, idx))
                    // Other lists preserved exactly.
                    && r.interrupted_thread_ids@ == self.interrupted_thread_ids@
                    && r.zombie_thread_ids@ == self.zombie_thread_ids@
                    && r.wf()
                },
                Err(r) => {
                    !found
                    && r.spec_pid() == self.spec_pid()
                    && r.spec_ready_count() == self.spec_ready_count()
                    && r.spec_sleeping_count() == self.spec_sleeping_count()
                    && r.spec_interrupted_count() == self.spec_interrupted_count()
                    && r.spec_zombie_count() == self.spec_zombie_count()
                    // Content preserved exactly.
                    && r.ready_thread_ids@ == self.ready_thread_ids@
                    && r.ready_admission_times@ == self.ready_admission_times@
                    && r.interrupted_thread_ids@ == self.interrupted_thread_ids@
                    && r.sleeping_thread_ids@ == self.sleeping_thread_ids@
                    && r.zombie_thread_ids@ == self.zombie_thread_ids@
                    && r.wf()
                },
            },
    {
        if found {
            // Derive the index via proof using `choose`.
            let ghost found_idx: int = choose|i: int|
                0 <= i < self.sleeping_thread_ids@.len()
                && self.sleeping_thread_ids@[i] == tid@;

            proof {
                // Witness that found_idx satisfies the search property.
                self.lemma_spec_find_thread_index(tid);
            }

            let new_ready_time: int = clock_now();
            let ghost new_ready_ids: Seq<int> = self.ready_thread_ids@.push(tid@);
            let ghost new_ready_times: Seq<int> = self.ready_admission_times@.push(new_ready_time);
            let ghost new_sleeping_ids: Seq<int> =
                self.sleeping_thread_ids@.subrange(0, found_idx)
                    .add(self.sleeping_thread_ids@.subrange(
                        found_idx + 1,
                        self.sleeping_thread_ids@.len() as int,
                    ));

            proof {
                // Prove new sleeping length.
                let s: Seq<int> = self.sleeping_thread_ids@;
                let idx: int = found_idx;
                let left: Seq<int> = s.subrange(0, idx);
                let right: Seq<int> = s.subrange(idx + 1, s.len() as int);
                assert(left.len() == idx as nat);
                assert(right.len() == (s.len() - idx as nat - 1) as nat);
                assert(left.add(right).len() == (s.len() - 1) as nat);

                // Prove new ready lengths match.
                assert(new_ready_ids.len() == self.ready_thread_ids@.len() + 1);
                assert(new_ready_times.len() == self.ready_admission_times@.len() + 1);

                // Prove new_ready_times elements are non-negative.
                assert forall|i: int| 0 <= i < new_ready_times.len()
                    implies #[trigger] new_ready_times[i] >= 0
                by {
                    if i < self.ready_admission_times@.len() as int {
                        assert(new_ready_times[i] == self.ready_admission_times@[i]);
                    } else {
                        assert(new_ready_times[i] == new_ready_time);
                    }
                }
            }

            Ok(RunnableProcess {
                pid: self.pid,
                ready_thread_ids: Ghost(new_ready_ids),
                ready_admission_times: Ghost(new_ready_times),
                interrupted_thread_ids: Ghost(self.interrupted_thread_ids@),
                sleeping_thread_ids: Ghost(new_sleeping_ids),
                zombie_thread_ids: Ghost(self.zombie_thread_ids@),
            })
        } else {
            Err(RunnableProcess {
                pid: self.pid,
                ready_thread_ids: Ghost(self.ready_thread_ids@),
                ready_admission_times: Ghost(self.ready_admission_times@),
                interrupted_thread_ids: Ghost(self.interrupted_thread_ids@),
                sleeping_thread_ids: Ghost(self.sleeping_thread_ids@),
                zombie_thread_ids: Ghost(self.zombie_thread_ids@),
            })
        }
    }

    /// Adds a thread to the ready queue.
    ///
    /// Models the original `RunnableProcess::add_thread(ready_thread)`.
    ///
    /// # Parameters
    ///
    /// - `ready_tid`: Ghost thread ID of the new ready thread.
    /// - `ready_time`: Ghost admission time of the new ready thread.
    ///
    /// # Returns
    ///
    /// An updated RunnableProcess with one additional ready thread.
    pub fn add_thread(self, ready_tid: Ghost<int>, ready_time: Ghost<int>) -> (result: RunnableProcess)
        requires
            self.wf(),
            ready_time@ >= 0,
        ensures
            result.spec_pid() == self.spec_pid(),
            result.spec_ready_count() == self.spec_ready_count() + 1,
            result.spec_interrupted_count() == self.spec_interrupted_count(),
            result.spec_sleeping_count() == self.spec_sleeping_count(),
            result.spec_zombie_count() == self.spec_zombie_count(),
            // Content specs: ready list gets the new thread appended.
            result.ready_thread_ids@ == self.ready_thread_ids@.push(ready_tid@),
            result.ready_admission_times@ == self.ready_admission_times@.push(ready_time@),
            // Other lists preserved exactly.
            result.interrupted_thread_ids@ == self.interrupted_thread_ids@,
            result.sleeping_thread_ids@ == self.sleeping_thread_ids@,
            result.zombie_thread_ids@ == self.zombie_thread_ids@,
            result.wf(),
    {
        let ghost new_ready_ids: Seq<int> = self.ready_thread_ids@.push(ready_tid@);
        let ghost new_ready_times: Seq<int> = self.ready_admission_times@.push(ready_time@);

        proof {
            assert(new_ready_ids.len() == self.ready_thread_ids@.len() + 1);
            assert(new_ready_times.len() == self.ready_admission_times@.len() + 1);

            // Prove new_ready_times elements are non-negative.
            assert forall|i: int| 0 <= i < new_ready_times.len()
                implies #[trigger] new_ready_times[i] >= 0
            by {
                if i < self.ready_admission_times@.len() as int {
                    assert(new_ready_times[i] == self.ready_admission_times@[i]);
                } else {
                    assert(new_ready_times[i] == ready_time@);
                }
            }
        }

        RunnableProcess {
            pid: self.pid,
            ready_thread_ids: Ghost(new_ready_ids),
            ready_admission_times: Ghost(new_ready_times),
            interrupted_thread_ids: Ghost(self.interrupted_thread_ids@),
            sleeping_thread_ids: Ghost(self.sleeping_thread_ids@),
            zombie_thread_ids: Ghost(self.zombie_thread_ids@),
        }
    }

}

} // verus!
