// Copyright(c) The Maintainers of Nanvix.
// Licensed under the MIT License.

//! # RunningProcess Implementation
//!
//! Represents a process that is currently running in the Nanvix kernel.
//! A RunningProcess has exactly one running thread, plus optional queues of
//! ready, interrupted, sleeping, and zombie threads.
//!
//! ## Verified Properties
//!
//! - Construction (new) produces well-formed state with correct identity.
//! - Process identifier (PID) is immutable: all operations preserve it.
//! - `schedule()` moves running→ready, producing a RunnableProcess with PID preserved
//!   and total thread count maintained (running thread becomes ready).
//! - `sleep()` moves running→sleeping; branches correctly based on available threads:
//!   produces RunnableProcess (if ready or interrupted threads exist) or SleepingProcess.
//! - `exit()` moves running→zombie, terminates all ready threads, interrupts all sleeping
//!   threads; produces RunnableProcess (if interrupted threads exist) or ZombieProcess.
//! - `exit_thread()` moves only the running thread→zombie; branches based on remaining
//!   threads: RunnableProcess, SleepingProcess, or ZombieProcess.
//! - `get_tid()` returns the running thread's ID.
//! - `wakeup()` moves a sleeping thread to ready, preserving PID and other lists.
//! - Well-formedness is preserved by all operations.
//!
//! ## Verification Model
//!
//! The original `RunningProcess` contains complex kernel types. For verification:
//! - `Box<ProcessState>` -> PID (int, identity tracking only).
//! - `RunningThread` -> ghost int (thread ID).
//! - `Option<NonEmptyVecDeque<T>>` -> `Seq<int>` (empty = None, non-empty = Some).
//! - `ContextInformation*` -> elided (HAL boundary).
//! - `Condvar` -> elided (sync primitive boundary).
//! - `ExitStatus` -> int.
//!
//! ## Trust Boundary
//!
//! - `RunnableProcess`, `SleepingProcess`, `InterruptedProcess`, `ZombieProcess`
//!   are boundary models of sibling modules.
//! - `InterruptedProcess::resume()` is modeled as producing a RunnableProcess with
//!   preserved PID (external_body).
//! - Thread state transitions (schedule, sleep, exit) are modeled as ID-preserving.
//! - `find_thread()` / `find_thread_mut()` are modeled spec-only (return references).
//! - `try_join_thread()` is modeled spec-only (complex return type with references).
//! - `state()` / `state_mut()` are elided (ProcessState access modeled via PID).

use vstd::prelude::*;

// Include specifications.
include!("running.spec.rs");

// Include proofs.
include!("running.proof.rs");

verus! {

//==================================================================================================
// Structures
//==================================================================================================

/// A process that is currently running.
///
/// Verification model of `src/kernel/src/pm/process/state/running.rs::RunningProcess`.
/// Thread collections are modeled as ghost sequences of thread IDs.
/// Exec-level counters track ghost sequence lengths for branch decisions.
///
/// **Note:** Fields are `pub` for Verus proof ergonomics.
pub struct RunningProcess {
    /// Process identifier (from the inner ProcessState).
    pub pid: Ghost<int>,
    /// Running thread ID.
    pub running_thread_id: Ghost<int>,
    /// Ghost sequence of ready thread IDs.
    pub ready_thread_ids: Ghost<Seq<int>>,
    /// Ghost sequence of interrupted thread IDs.
    pub interrupted_thread_ids: Ghost<Seq<int>>,
    /// Ghost sequence of sleeping thread IDs.
    pub sleeping_thread_ids: Ghost<Seq<int>>,
    /// Ghost sequence of zombie thread IDs.
    pub zombie_thread_ids: Ghost<Seq<int>>,
    /// Exec-level count of ready threads.
    pub ready_count: u64,
    /// Exec-level count of interrupted threads.
    pub interrupted_count: u64,
    /// Exec-level count of sleeping threads.
    pub sleeping_count: u64,
    /// Exec-level count of zombie threads.
    pub zombie_count: u64,
}

/// A process that is ready to run (boundary model).
///
/// Models `RunnableProcess` from the sibling module.
pub struct RunnableProcess {
    /// Process identifier.
    pub pid: Ghost<int>,
    /// Ready thread IDs (non-empty).
    pub ready_thread_ids: Ghost<Seq<int>>,
    /// Interrupted thread IDs.
    pub interrupted_thread_ids: Ghost<Seq<int>>,
    /// Sleeping thread IDs.
    pub sleeping_thread_ids: Ghost<Seq<int>>,
    /// Zombie thread IDs.
    pub zombie_thread_ids: Ghost<Seq<int>>,
}

/// A process that is sleeping (boundary model).
///
/// Models `SleepingProcess` from the sibling module.
pub struct SleepingProcess {
    /// Process identifier.
    pub pid: Ghost<int>,
    /// Sleeping thread IDs (non-empty).
    pub sleeping_thread_ids: Ghost<Seq<int>>,
    /// Zombie thread IDs.
    pub zombie_thread_ids: Ghost<Seq<int>>,
}

/// A process that was interrupted (boundary model).
///
/// Models `InterruptedProcess` from the sibling module.
pub struct InterruptedProcess {
    /// Process identifier.
    pub pid: Ghost<int>,
    /// Interrupted thread IDs (non-empty).
    pub interrupted_thread_ids: Ghost<Seq<int>>,
    /// Zombie thread IDs.
    pub zombie_thread_ids: Ghost<Seq<int>>,
}

/// A process that has terminated (boundary model).
///
/// Models `ZombieProcess` from the sibling module.
pub struct ZombieProcess {
    /// Process identifier.
    pub pid: Ghost<int>,
    /// Zombie thread IDs (non-empty).
    pub zombie_thread_ids: Ghost<Seq<int>>,
    /// Exit status.
    pub status: Ghost<int>,
}

/// Result of `RunningProcess::schedule()`.
///
/// Models `(RunnableProcess, *mut ContextInformation)` — the context pointer is elided.
pub struct ScheduleResult {
    /// The resulting runnable process.
    pub process: RunnableProcess,
}

/// Result of `RunningProcess::sleep()`.
///
/// Models `Result<(RunnableProcess, ctx), (SleepingProcess, ctx)>`.
pub enum SleepResult {
    /// Process has ready (or interrupted→resumed) threads and becomes runnable.
    Runnable(RunnableProcess),
    /// Process has no ready or interrupted threads and becomes sleeping.
    Sleeping(SleepingProcess),
}

/// Result of `RunningProcess::exit()`.
///
/// Models `Result<(RunnableProcess, ctx), (ZombieProcess, ctx)>`.
pub enum ExitResult {
    /// Process has interrupted threads remaining; resumes as runnable.
    Runnable(RunnableProcess),
    /// Process is fully terminated (all threads zombie).
    Zombie(ZombieProcess),
}

/// Result of `RunningProcess::exit_thread()`.
///
/// Models the three-way Result from the original.
pub enum ExitThreadResult {
    /// Process has ready threads; becomes runnable.
    Runnable(RunnableProcess),
    /// Process has sleeping threads; becomes sleeping.
    Sleeping(SleepingProcess),
    /// Process has no ready, interrupted, or sleeping threads; becomes zombie.
    Zombie(ZombieProcess),
}

//==================================================================================================
// External Dependencies
//==================================================================================================

/// Models `InterruptedProcess::resume()` — transitions to RunnableProcess.
///
/// In the original, resume picks an interrupted thread and makes it ready.
/// We model that the PID is preserved and the result has a non-empty ready list.
#[verifier::external_body]
fn interrupted_resume(ip: InterruptedProcess) -> (result: RunnableProcess)
    requires
        ip.wf(),
    ensures
        result.spec_pid() == ip.spec_pid(),
        result.wf(),
{
    unimplemented!()
}

//==================================================================================================
// RunningProcess Implementation
//==================================================================================================

impl RunningProcess {
    /// Creates a new RunningProcess.
    ///
    /// Models the original `RunningProcess::new(state, running, ready, interrupted, sleeping, zombie)`.
    ///
    /// # Parameters
    ///
    /// - `pid`: Process identifier.
    /// - `running_tid`: Thread identifier of the running thread.
    /// - `ready_ids`: Ready thread IDs.
    /// - `interrupted_ids`: Interrupted thread IDs.
    /// - `sleeping_ids`: Sleeping thread IDs.
    /// - `zombie_ids`: Zombie thread IDs.
    ///
    /// # Returns
    ///
    /// A new, well-formed RunningProcess.
    pub fn new(
        pid: Ghost<int>,
        running_tid: Ghost<int>,
        ready_ids: Ghost<Seq<int>>,
        interrupted_ids: Ghost<Seq<int>>,
        sleeping_ids: Ghost<Seq<int>>,
        zombie_ids: Ghost<Seq<int>>,
        ready_count: u64,
        interrupted_count: u64,
        sleeping_count: u64,
        zombie_count: u64,
    ) -> (result: RunningProcess)
        requires
            ready_count as nat == ready_ids@.len(),
            interrupted_count as nat == interrupted_ids@.len(),
            sleeping_count as nat == sleeping_ids@.len(),
            zombie_count as nat == zombie_ids@.len(),
        ensures
            result.spec_pid() == pid@,
            result.spec_running_thread_id() == running_tid@,
            result.spec_ready_count() == ready_ids@.len(),
            result.spec_interrupted_count() == interrupted_ids@.len(),
            result.spec_sleeping_count() == sleeping_ids@.len(),
            result.spec_zombie_count() == zombie_ids@.len(),
            result.wf(),
    {
        RunningProcess {
            pid,
            running_thread_id: running_tid,
            ready_thread_ids: ready_ids,
            interrupted_thread_ids: interrupted_ids,
            sleeping_thread_ids: sleeping_ids,
            zombie_thread_ids: zombie_ids,
            ready_count,
            interrupted_count,
            sleeping_count,
            zombie_count,
        }
    }

    /// Returns the running thread's identifier.
    ///
    /// Models the original `RunningProcess::get_tid()`.
    pub fn get_tid(&self) -> (result: Ghost<int>)
        ensures
            result@ == self.spec_running_thread_id(),
    {
        Ghost(self.running_thread_id@)
    }

    /// Transitions to a RunnableProcess by scheduling the running thread.
    ///
    /// Models the original `RunningProcess::schedule()`:
    /// - The running thread becomes a ready thread (pushed onto the ready queue).
    /// - Returns a RunnableProcess with all threads plus the newly ready one.
    ///
    /// # Returns
    ///
    /// A ScheduleResult containing the resulting RunnableProcess.
    pub fn schedule(self) -> (result: ScheduleResult)
        requires
            self.wf(),
        ensures
            // PID preserved.
            result.process.spec_pid() == self.spec_pid(),
            // The formerly running thread is now in the ready list.
            result.process.ready_thread_ids@.len() == self.spec_ready_count() + 1,
            // Content: ready list is old ready + running thread appended.
            result.process.ready_thread_ids@ ==
                self.ready_thread_ids@.push(self.running_thread_id@),
            // Other lists preserved.
            result.process.interrupted_thread_ids@ == self.interrupted_thread_ids@,
            result.process.sleeping_thread_ids@ == self.sleeping_thread_ids@,
            result.process.zombie_thread_ids@ == self.zombie_thread_ids@,
            // Result is well-formed (non-empty ready list).
            result.process.wf(),
    {
        let ghost new_ready_ids: Seq<int> = self.ready_thread_ids@.push(self.running_thread_id@);

        proof {
            assert(new_ready_ids.len() == self.ready_thread_ids@.len() + 1);
            assert(new_ready_ids.len() >= 1);
        }

        ScheduleResult {
            process: RunnableProcess {
                pid: Ghost(self.pid@),
                ready_thread_ids: Ghost(new_ready_ids),
                interrupted_thread_ids: Ghost(self.interrupted_thread_ids@),
                sleeping_thread_ids: Ghost(self.sleeping_thread_ids@),
                zombie_thread_ids: Ghost(self.zombie_thread_ids@),
            },
        }
    }

    /// Puts the running thread to sleep.
    ///
    /// Models the original `RunningProcess::sleep()`:
    /// - The running thread becomes a sleeping thread.
    /// - If ready threads exist, returns Runnable.
    /// - If no ready but interrupted threads exist, interrupted.resume() → Runnable.
    /// - Otherwise, returns Sleeping.
    ///
    /// # Returns
    ///
    /// SleepResult indicating the resulting process state.
    pub fn sleep(self) -> (result: SleepResult)
        requires
            self.wf(),
        ensures
            match result {
                SleepResult::Runnable(rp) => {
                    rp.spec_pid() == self.spec_pid()
                    && rp.wf()
                    // Branch: there were ready or interrupted threads.
                    && (self.spec_ready_count() > 0 || self.spec_interrupted_count() > 0)
                },
                SleepResult::Sleeping(sp) => {
                    sp.spec_pid() == self.spec_pid()
                    && sp.wf()
                    // Branch: no ready and no interrupted threads.
                    && self.spec_ready_count() == 0
                    && self.spec_interrupted_count() == 0
                    // Sleeping list includes the running thread.
                    && sp.sleeping_thread_ids@.len() ==
                        self.spec_sleeping_count() + 1
                },
            },
    {
        let ghost new_sleeping_ids: Seq<int> =
            self.sleeping_thread_ids@.push(self.running_thread_id@);

        proof {
            assert(new_sleeping_ids.len() == self.sleeping_thread_ids@.len() + 1);
            assert(new_sleeping_ids.len() >= 1);
        }

        // Check if there are ready threads.
        if self.ready_count > 0 {
            return SleepResult::Runnable(RunnableProcess {
                pid: Ghost(self.pid@),
                ready_thread_ids: Ghost(self.ready_thread_ids@),
                interrupted_thread_ids: Ghost(self.interrupted_thread_ids@),
                sleeping_thread_ids: Ghost(new_sleeping_ids),
                zombie_thread_ids: Ghost(self.zombie_thread_ids@),
            });
        }

        // Check if there are interrupted threads.
        if self.interrupted_count > 0 {
            proof {
                assert(self.interrupted_thread_ids@.len() >= 1);
            }
            let ip: InterruptedProcess = InterruptedProcess {
                pid: Ghost(self.pid@),
                interrupted_thread_ids: Ghost(self.interrupted_thread_ids@),
                zombie_thread_ids: Ghost(self.zombie_thread_ids@),
            };
            let rp: RunnableProcess = interrupted_resume(ip);
            return SleepResult::Runnable(rp);
        }

        // No ready or interrupted threads — become sleeping.
        SleepResult::Sleeping(SleepingProcess {
            pid: Ghost(self.pid@),
            sleeping_thread_ids: Ghost(new_sleeping_ids),
            zombie_thread_ids: Ghost(self.zombie_thread_ids@),
        })
    }

    /// Exits the process (terminates all threads).
    ///
    /// Models the original `RunningProcess::exit(status)`:
    /// - The running thread becomes a zombie.
    /// - All ready threads are terminated (become zombies).
    /// - All sleeping threads are interrupted.
    /// - If interrupted threads exist (original + from sleeping), returns Runnable
    ///   (via InterruptedProcess.resume()).
    /// - Otherwise, returns Zombie.
    ///
    /// # Parameters
    ///
    /// - `status`: Exit status for the process.
    ///
    /// # Returns
    ///
    /// ExitResult indicating the resulting process state.
    pub fn exit(self, status: Ghost<int>) -> (result: ExitResult)
        requires
            self.wf(),
        ensures
            match result {
                ExitResult::Runnable(rp) => {
                    rp.spec_pid() == self.spec_pid()
                    && rp.wf()
                    // Branch: there were interrupted or sleeping threads.
                    && (self.spec_interrupted_count() > 0
                        || self.spec_sleeping_count() > 0)
                },
                ExitResult::Zombie(zp) => {
                    zp.spec_pid() == self.spec_pid()
                    && zp.wf()
                    && zp.spec_status() == status@
                    // Branch: no interrupted or sleeping threads.
                    && self.spec_interrupted_count() == 0
                    && self.spec_sleeping_count() == 0
                    // Zombie list includes running + all ready + original zombie.
                    && zp.zombie_thread_ids@.len() ==
                        1 + self.spec_ready_count() + self.spec_zombie_count()
                },
            },
    {
        // Running thread becomes zombie.
        let ghost running_zombie: Seq<int> = seq![self.running_thread_id@];

        // All ready threads become zombies.
        let ghost ready_zombies: Seq<int> = self.ready_thread_ids@;

        // Combine: running zombie + ready zombies + existing zombies.
        let ghost new_zombie_ids: Seq<int> =
            running_zombie.add(ready_zombies).add(self.zombie_thread_ids@);

        proof {
            assert(new_zombie_ids.len() ==
                1 + self.ready_thread_ids@.len() + self.zombie_thread_ids@.len());
            assert(new_zombie_ids.len() >= 1);
        }

        // Sleeping threads become interrupted.
        let ghost new_interrupted_ids: Seq<int> =
            self.interrupted_thread_ids@.add(self.sleeping_thread_ids@);

        if self.interrupted_count > 0 || self.sleeping_count > 0 {
            proof {
                assert(new_interrupted_ids.len() ==
                    self.interrupted_thread_ids@.len() + self.sleeping_thread_ids@.len());
                assert(new_interrupted_ids.len() >= 1);
            }
            let ip: InterruptedProcess = InterruptedProcess {
                pid: Ghost(self.pid@),
                interrupted_thread_ids: Ghost(new_interrupted_ids),
                zombie_thread_ids: Ghost(new_zombie_ids),
            };
            let rp: RunnableProcess = interrupted_resume(ip);
            ExitResult::Runnable(rp)
        } else {
            proof {
                assert(self.interrupted_thread_ids@.len() == 0);
                assert(self.sleeping_thread_ids@.len() == 0);
            }
            ExitResult::Zombie(ZombieProcess {
                pid: Ghost(self.pid@),
                zombie_thread_ids: Ghost(new_zombie_ids),
                status,
            })
        }
    }

    /// Exits the running thread only.
    ///
    /// Models the original `RunningProcess::exit_thread(status)`:
    /// - The running thread becomes a zombie.
    /// - If ready threads exist, returns Runnable.
    /// - Else if interrupted threads exist, InterruptedProcess.resume() → Runnable.
    /// - Else if sleeping threads exist, returns Sleeping.
    /// - Otherwise, returns Zombie.
    ///
    /// # Parameters
    ///
    /// - `status`: Exit status for the thread.
    ///
    /// # Returns
    ///
    /// ExitThreadResult indicating the resulting process state.
    pub fn exit_thread(self, status: Ghost<int>) -> (result: ExitThreadResult)
        requires
            self.wf(),
        ensures
            match result {
                ExitThreadResult::Runnable(rp) => {
                    rp.spec_pid() == self.spec_pid()
                    && rp.wf()
                    && (self.spec_ready_count() > 0 || self.spec_interrupted_count() > 0)
                },
                ExitThreadResult::Sleeping(sp) => {
                    sp.spec_pid() == self.spec_pid()
                    && sp.wf()
                    && self.spec_ready_count() == 0
                    && self.spec_interrupted_count() == 0
                    && self.spec_sleeping_count() > 0
                },
                ExitThreadResult::Zombie(zp) => {
                    zp.spec_pid() == self.spec_pid()
                    && zp.wf()
                    && zp.spec_status() == status@
                    && self.spec_ready_count() == 0
                    && self.spec_interrupted_count() == 0
                    && self.spec_sleeping_count() == 0
                    // Zombie list includes the running thread + original zombie.
                    && zp.zombie_thread_ids@.len() == 1 + self.spec_zombie_count()
                },
            },
    {
        // Running thread becomes zombie.
        let ghost new_zombie_ids: Seq<int> =
            self.zombie_thread_ids@.push(self.running_thread_id@);

        proof {
            assert(new_zombie_ids.len() == self.zombie_thread_ids@.len() + 1);
            assert(new_zombie_ids.len() >= 1);
        }

        if self.ready_count > 0 {
            return ExitThreadResult::Runnable(RunnableProcess {
                pid: Ghost(self.pid@),
                ready_thread_ids: Ghost(self.ready_thread_ids@),
                interrupted_thread_ids: Ghost(self.interrupted_thread_ids@),
                sleeping_thread_ids: Ghost(self.sleeping_thread_ids@),
                zombie_thread_ids: Ghost(new_zombie_ids),
            });
        }

        if self.interrupted_count > 0 {
            proof {
                assert(self.interrupted_thread_ids@.len() >= 1);
            }
            let ip: InterruptedProcess = InterruptedProcess {
                pid: Ghost(self.pid@),
                interrupted_thread_ids: Ghost(self.interrupted_thread_ids@),
                zombie_thread_ids: Ghost(new_zombie_ids),
            };
            let rp: RunnableProcess = interrupted_resume(ip);
            return ExitThreadResult::Runnable(rp);
        }

        if self.sleeping_count > 0 {
            return ExitThreadResult::Sleeping(SleepingProcess {
                pid: Ghost(self.pid@),
                sleeping_thread_ids: Ghost(self.sleeping_thread_ids@),
                zombie_thread_ids: Ghost(new_zombie_ids),
            });
        }

        ExitThreadResult::Zombie(ZombieProcess {
            pid: Ghost(self.pid@),
            zombie_thread_ids: Ghost(new_zombie_ids),
            status,
        })
    }

    /// Wakes up a sleeping thread and moves it to the ready queue.
    ///
    /// Models the original `RunningProcess::wakeup(tid)`.
    /// Returns Ok(RunningProcess) on success, Err(RunningProcess) if not found.
    ///
    /// # Parameters
    ///
    /// - `tid`: Ghost thread ID to wake up.
    /// - `found`: Oracle parameter — whether the thread was found in sleeping list.
    ///
    /// # Returns
    ///
    /// Ok with updated state if found, Err with unchanged state if not found.
    pub fn wakeup(self, tid: Ghost<int>, found: bool) -> (result: Result<RunningProcess, RunningProcess>)
        requires
            self.wf(),
            found == Self::spec_seq_contains(self.sleeping_thread_ids@, tid@),
        ensures
            match result {
                Ok(r) => {
                    found
                    && r.spec_pid() == self.spec_pid()
                    && r.spec_running_thread_id() == self.spec_running_thread_id()
                    && r.spec_ready_count() == self.spec_ready_count() + 1
                    && r.spec_sleeping_count() == self.spec_sleeping_count() - 1
                    && r.spec_interrupted_count() == self.spec_interrupted_count()
                    && r.spec_zombie_count() == self.spec_zombie_count()
                    // Content: ready list gets the woken thread appended.
                    && r.ready_thread_ids@ == self.ready_thread_ids@.push(tid@)
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
                    && r.spec_running_thread_id() == self.spec_running_thread_id()
                    && r.spec_ready_count() == self.spec_ready_count()
                    && r.spec_sleeping_count() == self.spec_sleeping_count()
                    && r.spec_interrupted_count() == self.spec_interrupted_count()
                    && r.spec_zombie_count() == self.spec_zombie_count()
                    // Content preserved exactly.
                    && r.ready_thread_ids@ == self.ready_thread_ids@
                    && r.interrupted_thread_ids@ == self.interrupted_thread_ids@
                    && r.sleeping_thread_ids@ == self.sleeping_thread_ids@
                    && r.zombie_thread_ids@ == self.zombie_thread_ids@
                    && r.wf()
                },
            },
    {
        if !found {
            return Err(RunningProcess {
                pid: Ghost(self.pid@),
                running_thread_id: Ghost(self.running_thread_id@),
                ready_thread_ids: Ghost(self.ready_thread_ids@),
                interrupted_thread_ids: Ghost(self.interrupted_thread_ids@),
                sleeping_thread_ids: Ghost(self.sleeping_thread_ids@),
                zombie_thread_ids: Ghost(self.zombie_thread_ids@),
                ready_count: self.ready_count,
                interrupted_count: self.interrupted_count,
                sleeping_count: self.sleeping_count,
                zombie_count: self.zombie_count,
            });
        }

        // Derive the index via proof using `choose`.
        let ghost found_idx: int = choose|i: int|
            0 <= i < self.sleeping_thread_ids@.len()
            && self.sleeping_thread_ids@[i] == tid@;

        proof {
            self.lemma_spec_find_thread_index(tid);
        }

        let ghost new_ready_ids: Seq<int> = self.ready_thread_ids@.push(tid@);
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

            // Prove new ready length.
            assert(new_ready_ids.len() == self.ready_thread_ids@.len() + 1);
        }

        Ok(RunningProcess {
            pid: Ghost(self.pid@),
            running_thread_id: Ghost(self.running_thread_id@),
            ready_thread_ids: Ghost(new_ready_ids),
            interrupted_thread_ids: Ghost(self.interrupted_thread_ids@),
            sleeping_thread_ids: Ghost(new_sleeping_ids),
            zombie_thread_ids: Ghost(self.zombie_thread_ids@),
            ready_count: self.ready_count + 1,
            interrupted_count: self.interrupted_count,
            sleeping_count: self.sleeping_count - 1,
            zombie_count: self.zombie_count,
        })
    }
}

} // verus!
