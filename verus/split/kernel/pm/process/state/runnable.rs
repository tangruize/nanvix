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
//! - `ProcessState` operations (state(), state_mut()) are elided since we
//!   only track the PID for identity verification.
//! - Thread state transitions (ReadyThread::terminate(), SleepingThread::interrupt(),
//!   SleepingThread::wakeup()) are modeled as ID-preserving operations.
//! - `find_thread()` and `find_thread_mut()` are omitted from the verified model
//!   because they return reference types (`ThreadRef`, `ThreadRefMut`) that Verus
//!   cannot express. Their correctness is implied by the thread list membership
//!   invariants.

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
pub struct RunningProcess {
    /// Process identifier value.
    pub pid: int,
    /// The running thread ID.
    pub running_thread_id: int,
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
    pub pid: int,
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
    pub pid: int,
    /// Zombie thread IDs (non-empty).
    pub zombie_thread_ids: Ghost<Seq<int>>,
    /// Exit status.
    pub status: int,
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
        RunnableProcess {
            pid: pid,
            ready_thread_ids: Ghost(seq![ready_tid@]),
            ready_admission_times: Ghost(seq![ready_time@]),
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

    /// Returns the process identifier value.
    pub fn pid(&self) -> (result: int)
        ensures
            result == self.spec_pid(),
    {
        self.pid.value()
    }

    /// Transitions the runnable process to running by selecting the thread
    /// with the earliest admission time.
    ///
    /// Models the original `RunnableProcess::run()`.
    /// The `ContextInformation` pointer and `VirtualAddress` from the original
    /// return type are omitted (HAL boundary).
    ///
    /// # Parameters
    ///
    /// - `selected_idx`: Oracle parameter — the index of the ready thread with
    ///   the earliest admission time. Precondition ties this to the ghost state.
    ///
    /// # Returns
    ///
    /// A RunningProcess with the selected thread as running.
    pub fn run(self, selected_idx: Ghost<int>) -> (result: RunningProcess)
        requires
            self.wf(),
            0 <= selected_idx@ < self.ready_thread_ids@.len(),
            // The selected index has the earliest admission time.
            forall|j: int| 0 <= j < self.ready_admission_times@.len()
                ==> self.ready_admission_times@[selected_idx@]
                    <= self.ready_admission_times@[j],
        ensures
            result.spec_pid() == self.spec_pid(),
            result.spec_running_thread_id() == self.ready_thread_ids@[selected_idx@],
            // Remaining ready threads are the original minus the selected one.
            result.ready_thread_ids@.len() == self.spec_ready_count() - 1,
            // Other lists are preserved.
            result.interrupted_thread_ids@ == self.interrupted_thread_ids@,
            result.sleeping_thread_ids@ == self.sleeping_thread_ids@,
            result.zombie_thread_ids@ == self.zombie_thread_ids@,
    {
        let ghost remaining_ready: Seq<int> =
            self.ready_thread_ids@.subrange(0, selected_idx@)
                .add(self.ready_thread_ids@.subrange(
                    selected_idx@ + 1,
                    self.ready_thread_ids@.len() as int,
                ));

        proof {
            // Prove the remaining sequence has the expected length.
            let s: Seq<int> = self.ready_thread_ids@;
            let idx: int = selected_idx@;
            let left: Seq<int> = s.subrange(0, idx);
            let right: Seq<int> = s.subrange(idx + 1, s.len() as int);
            assert(left.len() == idx as nat);
            assert(right.len() == (s.len() - idx as nat - 1) as nat);
            assert(left.add(right).len() == (s.len() - 1) as nat);
        }

        RunningProcess {
            pid: self.pid.spec_value(),
            running_thread_id: self.ready_thread_ids@[selected_idx@],
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
                    ip.spec_pid() == self.spec_pid()
                    // Has interrupted threads.
                    && ip.interrupted_thread_ids@.len() >= 1
                    // Zombie threads include all original ready + original zombie.
                    && ip.zombie_thread_ids@.len() ==
                        self.spec_ready_count() + self.spec_zombie_count()
                    && ip.wf()
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
                    && zp.spec_status() == EXIT_STATUS_INTERRUPTED()
                    && zp.wf()
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

        if self.interrupted_thread_ids@.len() > 0 || self.sleeping_thread_ids@.len() > 0 {
            proof {
                assert(new_interrupted_ids.len() ==
                    self.interrupted_thread_ids@.len() + self.sleeping_thread_ids@.len());
                assert(new_interrupted_ids.len() >= 1);
            }
            TerminateResult::Interrupted(InterruptedProcess {
                pid: self.pid.spec_value(),
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
                pid: self.pid.spec_value(),
                zombie_thread_ids: Ghost(new_zombie_ids),
                status: status,
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
    pub fn wakeup(self, tid: Ghost<int>, found: bool, found_idx: Ghost<int>) -> (result: Result<RunnableProcess, RunnableProcess>)
        requires
            self.wf(),
            found ==> (
                0 <= found_idx@ < self.sleeping_thread_ids@.len()
                && self.sleeping_thread_ids@[found_idx@] == tid@
            ),
            !found ==> (
                forall|i: int| 0 <= i < self.sleeping_thread_ids@.len()
                    ==> self.sleeping_thread_ids@[i] != tid@
            ),
        ensures
            match result {
                Ok(r) => {
                    found
                    && r.spec_pid() == self.spec_pid()
                    && r.spec_ready_count() == self.spec_ready_count() + 1
                    && r.spec_sleeping_count() == self.spec_sleeping_count() - 1
                    && r.spec_interrupted_count() == self.spec_interrupted_count()
                    && r.spec_zombie_count() == self.spec_zombie_count()
                    && r.wf()
                },
                Err(r) => {
                    !found
                    && r.spec_pid() == self.spec_pid()
                    && r.spec_ready_count() == self.spec_ready_count()
                    && r.spec_sleeping_count() == self.spec_sleeping_count()
                    && r.spec_interrupted_count() == self.spec_interrupted_count()
                    && r.spec_zombie_count() == self.spec_zombie_count()
                    && r.wf()
                },
            },
    {
        if found {
            let new_ready_time: int = clock_now();
            let ghost new_ready_ids: Seq<int> = self.ready_thread_ids@.push(tid@);
            let ghost new_ready_times: Seq<int> = self.ready_admission_times@.push(new_ready_time);
            let ghost new_sleeping_ids: Seq<int> =
                self.sleeping_thread_ids@.subrange(0, found_idx@)
                    .add(self.sleeping_thread_ids@.subrange(
                        found_idx@ + 1,
                        self.sleeping_thread_ids@.len() as int,
                    ));

            proof {
                // Prove new sleeping length.
                let s: Seq<int> = self.sleeping_thread_ids@;
                let idx: int = found_idx@;
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

    /// Returns the earliest admission time among ready threads.
    ///
    /// Models the original `RunnableProcess::earliest_admission_time()`.
    ///
    /// # Parameters
    ///
    /// - `min_idx`: Oracle parameter — the index of the minimum admission time.
    ///
    /// # Returns
    ///
    /// The earliest admission time.
    pub fn earliest_admission_time(&self, min_idx: Ghost<int>) -> (result: int)
        requires
            self.wf(),
            0 <= min_idx@ < self.ready_admission_times@.len(),
            forall|j: int| 0 <= j < self.ready_admission_times@.len()
                ==> self.ready_admission_times@[min_idx@]
                    <= self.ready_admission_times@[j],
        ensures
            result == self.ready_admission_times@[min_idx@],
            result >= 0,
            // The result is the minimum: no admission time is smaller.
            forall|j: int| 0 <= j < self.ready_admission_times@.len()
                ==> result <= self.ready_admission_times@[j],
    {
        self.ready_admission_times@[min_idx@]
    }
}

} // verus!
