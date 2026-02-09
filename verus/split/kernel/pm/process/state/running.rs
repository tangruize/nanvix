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
//!   Sleeping threads are threaded through `InterruptedProcess` on the interrupted path.
//! - `exit()` moves running→zombie, terminates all ready threads, interrupts all sleeping
//!   threads; produces RunnableProcess (if interrupted threads exist) or ZombieProcess.
//!   Zombie thread content is threaded through `interrupted_resume()`.
//! - `exit_thread()` moves only the running thread→zombie; branches based on remaining
//!   threads: RunnableProcess, SleepingProcess, or ZombieProcess.
//! - `get_tid()` returns the running thread's ID.
//! - `wakeup()` moves a sleeping thread to ready, preserving PID and other lists.
//! - `try_join_thread()` is modeled spec-only via `spec_try_join_thread()`.
//! - `find_thread()` / `find_thread_mut()` are modeled spec-only via `spec_find_thread()`.
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
//! - `alarm: Option<SystemTime>` -> elided. Does not affect state machine logic.
//!
//! ## Trust Boundary
//!
//! - `RunnableProcess`, `SleepingProcess`, `InterruptedProcess`, `ZombieProcess`
//!   are boundary models of sibling modules.
//! - `InterruptedProcess::resume()` is modeled via `interrupted_resume()` (external_body).
//!   It preserves PID, produces a well-formed RunnableProcess, and threads through
//!   sleeping and zombie thread lists from the InterruptedProcess.
//! - Thread state transitions (schedule, sleep, exit) are modeled as ID-preserving.
//! - `find_thread()` / `find_thread_mut()` are modeled spec-only (return references
//!   that Verus cannot express). See `spec_find_thread()` in spec file.
//! - `try_join_thread()` is modeled spec-only via `spec_try_join_thread()`. The key
//!   property: joining a running thread errors, joining a zombie removes it, joining
//!   a live thread returns a condvar, joining a missing thread errors.
//! - `state()` / `state_mut()` are elided (ProcessState access modeled via PID).
//!   `state_mut()` permits arbitrary mutation; callers must preserve PID immutability.
//! - `running_mut()` returns `&mut RunningThread`, permitting arbitrary mutation.
//!   Callers must preserve the running thread's ID (`spec_running_thread_id()`)
//!   and any structural invariants. This is discharged when RunningThread is verified.
//! - `wakeup()` takes a `found: bool` oracle because `Seq::contains()` is spec-only.
//!   The precondition constrains it to match ghost state. See spec file for details.
//!
//! ## Bug Fix in Original Source
//!
//! Verification discovered a bug in `exit_thread()` (original line 286):
//! `self.zombie.take()` was passed to `InterruptedProcess::from_sleeping`, but
//! `self.zombie` was already consumed at line 261 into `zombie_threads`, so
//! `self.zombie.take()` was always `None`. This caused the just-exited running
//! thread's zombie state to be lost. The original source has been fixed to pass
//! `Some(zombie_threads)` instead, matching the verified model.
//!
//! ## Fields
//!
//! All struct fields are `pub` for Verus proof ergonomics (spec access, direct
//! construction in lemmas). The original has private fields with getter/setter
//! methods. This visibility difference has no functional impact on verification.

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
/// Models `InterruptedProcess` from the sibling module. Includes
/// `sleeping_thread_ids` to match the original struct which carries
/// sleeping threads through `from_sleeping()` and `resume()`.
pub struct InterruptedProcess {
    /// Process identifier.
    pub pid: Ghost<int>,
    /// Interrupted thread IDs (non-empty).
    pub interrupted_thread_ids: Ghost<Seq<int>>,
    /// Sleeping thread IDs (carried through resume).
    pub sleeping_thread_ids: Ghost<Seq<int>>,
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
/// In the original, `resume()` pops the front interrupted thread, makes it
/// ready, and passes through sleeping and zombie threads. Specifically:
///   result.ready = [popped_thread] (exactly one)
///   result.interrupted = ip.interrupted[1..] (remaining tail)
///   Thread IDs are fully preserved (permutation, not just count).
#[verifier::external_body]
fn interrupted_resume(ip: InterruptedProcess) -> (result: RunnableProcess)
    requires
        ip.wf(),
    ensures
        result.spec_pid() == ip.spec_pid(),
        result.wf(),
        // Sleeping threads are passed through resume() into the result.
        result.sleeping_thread_ids@ == ip.sleeping_thread_ids@,
        // Zombie threads are passed through resume() into the result.
        result.zombie_thread_ids@ == ip.zombie_thread_ids@,
        // Exactly one interrupted thread became ready: the front of the deque.
        result.ready_thread_ids@.len() == 1,
        result.ready_thread_ids@[0] == ip.interrupted_thread_ids@[0],
        // Remaining interrupted threads are the tail (IDs preserved exactly).
        result.interrupted_thread_ids@ ==
            ip.interrupted_thread_ids@.subrange(
                1, ip.interrupted_thread_ids@.len() as int),
        // Thread count conservation (follows from the above).
        result.ready_thread_ids@.len() + result.interrupted_thread_ids@.len()
            == ip.interrupted_thread_ids@.len(),
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
    /// - `ready_count`: Exec-level count of ready threads.
    /// - `interrupted_count`: Exec-level count of interrupted threads.
    /// - `sleeping_count`: Exec-level count of sleeping threads.
    /// - `zombie_count`: Exec-level count of zombie threads.
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
    ///
    /// # Returns
    ///
    /// The ghost thread identifier of the running thread.
    pub fn get_tid(&self) -> (result: Ghost<int>)
        ensures
            result@ == self.spec_running_thread_id(),
    {
        Ghost(self.running_thread_id@)
    }

    /// Returns the process state (modeled as PID).
    ///
    /// Models the original `RunningProcess::state()`.
    /// Since ProcessState is modeled as just a PID, this returns the PID value.
    /// The original returns `&ProcessState`, but references are elided in the
    /// verification model.
    ///
    /// # Returns
    ///
    /// The process identifier.
    #[verifier::external_body]
    pub fn state(&self) -> (result: Ghost<int>)
        ensures
            result@ == self.spec_pid(),
    {
        unimplemented!()
    }

    /// Returns a mutable reference to the process state.
    ///
    /// Models the original `RunningProcess::state_mut()`.
    /// The original returns `&mut ProcessState`, which permits mutation of
    /// ProcessState fields. Since we model ProcessState only as a PID, we
    /// return Ghost<int>. Callers must ensure `mutation_frame_preserved()`
    /// holds after any mutation (PID and all thread lists unchanged).
    ///
    /// # Returns
    ///
    /// The process identifier (as a ghost value).
    #[verifier::external_body]
    pub fn state_mut(&mut self) -> (result: Ghost<int>)
        ensures
            result@ == self.spec_pid(),
            // Frame: mutation through state_mut does not change modeled fields.
            self.spec_pid() == old(self).spec_pid(),
            self.spec_running_thread_id() == old(self).spec_running_thread_id(),
            self.ready_thread_ids@ == old(self).ready_thread_ids@,
            self.interrupted_thread_ids@ == old(self).interrupted_thread_ids@,
            self.sleeping_thread_ids@ == old(self).sleeping_thread_ids@,
            self.zombie_thread_ids@ == old(self).zombie_thread_ids@,
            self.wf() == old(self).wf(),
    {
        unimplemented!()
    }

    /// Returns a mutable reference to the running thread.
    ///
    /// Models the original `RunningProcess::running_mut()`.
    /// The original returns `&mut RunningThread`, which permits mutation of
    /// RunningThread fields (e.g., priority). Since we model RunningThread
    /// only as a thread ID, we return Ghost<int>. Callers must preserve the
    /// running thread's ID and all structural invariants.
    ///
    /// # Returns
    ///
    /// The running thread identifier (as a ghost value).
    #[verifier::external_body]
    pub fn running_mut(&mut self) -> (result: Ghost<int>)
        ensures
            result@ == self.spec_running_thread_id(),
            // Frame: mutation through running_mut does not change modeled fields.
            self.spec_pid() == old(self).spec_pid(),
            self.spec_running_thread_id() == old(self).spec_running_thread_id(),
            self.ready_thread_ids@ == old(self).ready_thread_ids@,
            self.interrupted_thread_ids@ == old(self).interrupted_thread_ids@,
            self.sleeping_thread_ids@ == old(self).sleeping_thread_ids@,
            self.zombie_thread_ids@ == old(self).zombie_thread_ids@,
            self.wf() == old(self).wf(),
    {
        unimplemented!()
    }

    /// Attempts to join (collect) a zombie thread by its identifier.
    ///
    /// Models the original `RunningProcess::try_join_thread(tid)`.
    /// Returns an abstract result tag matching `spec_try_join_thread()`:
    /// - `0`: Thread was zombie and has been removed from zombie list.
    /// - `1`: Thread is the running thread (OperationNotPermitted).
    /// - `2`: Thread is live (ready/sleeping/interrupted), returns condvar.
    /// - `3`: Thread not found (NoSuchProcess).
    ///
    /// On successful zombie join (tag=0), `self` is mutated: the zombie
    /// list shrinks by one (the joined thread is removed).
    ///
    /// # Parameters
    ///
    /// - `tid`: Ghost thread identifier to join.
    ///
    /// # Returns
    ///
    /// The result tag as a ghost int.
    #[verifier::external_body]
    pub fn try_join_thread(&mut self, tid: Ghost<int>) -> (result: Ghost<int>)
        requires
            old(self).wf(),
        ensures
            result@ == old(self).spec_try_join_thread(tid@),
            // PID and running thread unchanged.
            self.spec_pid() == old(self).spec_pid(),
            self.spec_running_thread_id() == old(self).spec_running_thread_id(),
            // Ready, interrupted, sleeping lists unchanged.
            self.ready_thread_ids@ == old(self).ready_thread_ids@,
            self.interrupted_thread_ids@ == old(self).interrupted_thread_ids@,
            self.sleeping_thread_ids@ == old(self).sleeping_thread_ids@,
            // Zombie list: removed on success, unchanged otherwise.
            (result@ == 0 ==> self.zombie_thread_ids@ ==
                old(self).spec_try_join_zombie_post(tid@)),
            (result@ != 0 ==> self.zombie_thread_ids@ == old(self).zombie_thread_ids@),
            self.wf(),
    {
        unimplemented!()
    }

    /// Finds a thread by its identifier and returns which list it belongs to.
    ///
    /// Models the original `RunningProcess::find_thread(tid)`.
    /// The original returns `Option<ThreadRef>` (a reference enum). Since
    /// Verus cannot express reference-returning functions, we return the
    /// abstract list variant from `spec_find_thread()`:
    /// - `Some(0)`: running thread.
    /// - `Some(1)`: ready thread.
    /// - `Some(2)`: interrupted thread.
    /// - `Some(3)`: sleeping thread.
    /// - `Some(4)`: zombie thread.
    /// - `None`: not found.
    ///
    /// # Parameters
    ///
    /// - `tid`: Ghost thread identifier to search for.
    ///
    /// # Returns
    ///
    /// The list variant as `Option<Ghost<int>>`.
    #[verifier::external_body]
    pub fn find_thread(&self, tid: Ghost<int>) -> (result: Option<Ghost<int>>)
        ensures
            match result {
                Some(v) => self.spec_find_thread(tid@) == Some(v@),
                None => self.spec_find_thread(tid@).is_none(),
            },
    {
        unimplemented!()
    }

    /// Finds a thread by its identifier (mutable variant).
    ///
    /// Models the original `RunningProcess::find_thread_mut(tid)`.
    /// Same semantics as `find_thread()` — returns the list variant.
    /// The mutable reference in the original allows in-place mutation of
    /// the found thread, but this does not change the thread's identity
    /// or list membership. Frame condition: self is unchanged.
    ///
    /// # Parameters
    ///
    /// - `tid`: Ghost thread identifier to search for.
    ///
    /// # Returns
    ///
    /// The list variant as `Option<Ghost<int>>`.
    #[verifier::external_body]
    pub fn find_thread_mut(&mut self, tid: Ghost<int>) -> (result: Option<Ghost<int>>)
        requires
            old(self).wf(),
        ensures
            match result {
                Some(v) => old(self).spec_find_thread(tid@) == Some(v@),
                None => old(self).spec_find_thread(tid@).is_none(),
            },
            // Frame: find_thread_mut does not change any modeled fields.
            self.spec_pid() == old(self).spec_pid(),
            self.spec_running_thread_id() == old(self).spec_running_thread_id(),
            self.ready_thread_ids@ == old(self).ready_thread_ids@,
            self.interrupted_thread_ids@ == old(self).interrupted_thread_ids@,
            self.sleeping_thread_ids@ == old(self).sleeping_thread_ids@,
            self.zombie_thread_ids@ == old(self).zombie_thread_ids@,
            self.wf() == old(self).wf(),
    {
        unimplemented!()
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
    /// Models the original `RunningProcess::sleep(alarm)`:
    /// - The running thread becomes a sleeping thread.
    /// - If ready threads exist, returns Runnable with sleeping threads updated.
    /// - If no ready but interrupted threads exist, constructs InterruptedProcess
    ///   with sleeping threads (matching `InterruptedProcess::from_sleeping`),
    ///   then calls `resume()` which passes sleeping threads through to the result.
    /// - Otherwise, returns Sleeping.
    ///
    /// The `alarm` parameter from the original is elided (does not affect
    /// process state machine logic; only affects SleepingThread wakeup timing).
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
                    // Sleeping threads in result include the running thread.
                    && rp.sleeping_thread_ids@.len() >= self.spec_sleeping_count() + 1
                    // Zombie threads preserved.
                    && rp.zombie_thread_ids@ == self.zombie_thread_ids@
                    // Ready branch: exact content specified.
                    && (self.spec_ready_count() > 0 ==> {
                        rp.ready_thread_ids@ == self.ready_thread_ids@
                        && rp.sleeping_thread_ids@ ==
                            self.sleeping_thread_ids@.push(self.running_thread_id@)
                        && rp.interrupted_thread_ids@ == self.interrupted_thread_ids@
                    })
                    // Interrupted branch: details from strengthened interrupted_resume().
                    && (self.spec_ready_count() == 0 && self.spec_interrupted_count() > 0 ==> {
                        rp.sleeping_thread_ids@ ==
                            self.sleeping_thread_ids@.push(self.running_thread_id@)
                        && rp.ready_thread_ids@.len() == 1
                        && rp.ready_thread_ids@[0] == self.interrupted_thread_ids@[0]
                        && rp.interrupted_thread_ids@ ==
                            self.interrupted_thread_ids@.subrange(
                                1, self.interrupted_thread_ids@.len() as int)
                    })
                },
                SleepResult::Sleeping(sp) => {
                    sp.spec_pid() == self.spec_pid()
                    && sp.wf()
                    // Branch: no ready and no interrupted threads.
                    && self.spec_ready_count() == 0
                    && self.spec_interrupted_count() == 0
                    // Sleeping list content: old sleeping + running thread.
                    && sp.sleeping_thread_ids@ ==
                        self.sleeping_thread_ids@.push(self.running_thread_id@)
                    && sp.sleeping_thread_ids@.len() ==
                        self.spec_sleeping_count() + 1
                    // Zombie threads preserved.
                    && sp.zombie_thread_ids@ == self.zombie_thread_ids@
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
                sleeping_thread_ids: Ghost(new_sleeping_ids),
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
    ///   (via InterruptedProcess.resume()). Note: in the original code at line 219,
    ///   `self.sleeping_threads.take()` is always `None` because sleeping threads were
    ///   already consumed at line 208. So InterruptedProcess has no sleeping threads here.
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
                    // Zombie threads in result contain running + ready + original zombie.
                    && rp.zombie_thread_ids@.len() ==
                        1 + self.spec_ready_count() + self.spec_zombie_count()
                    && rp.zombie_thread_ids@ ==
                        seq![self.running_thread_id@].add(
                            self.ready_thread_ids@).add(self.zombie_thread_ids@)
                    // No sleeping threads remain (all were converted to interrupted).
                    && rp.sleeping_thread_ids@.len() == 0
                    // Exactly one interrupted thread was resumed as ready.
                    && rp.ready_thread_ids@.len() == 1
                    // The ready thread is the first element of the combined interrupted list.
                    && rp.ready_thread_ids@[0] ==
                        self.interrupted_thread_ids@.add(self.sleeping_thread_ids@)[0]
                    // The remaining interrupted threads are the tail.
                    && rp.interrupted_thread_ids@ ==
                        self.interrupted_thread_ids@.add(self.sleeping_thread_ids@).subrange(
                            1, (self.spec_interrupted_count() + self.spec_sleeping_count()) as int)
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
                    && zp.zombie_thread_ids@ ==
                        seq![self.running_thread_id@].add(
                            self.ready_thread_ids@).add(self.zombie_thread_ids@)
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
            // In the original, self.sleeping_threads was already taken (line 208),
            // so InterruptedProcess::from_sleeping gets None for sleeping. We model
            // this faithfully with empty sleeping_thread_ids.
            let ip: InterruptedProcess = InterruptedProcess {
                pid: Ghost(self.pid@),
                interrupted_thread_ids: Ghost(new_interrupted_ids),
                sleeping_thread_ids: Ghost(Seq::empty()),
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
    /// ## Bug Fix
    ///
    /// Verification discovered that the original code's interrupted branch
    /// (line 286) passed `self.zombie.take()` to `InterruptedProcess::from_sleeping`,
    /// but `self.zombie` was already consumed at line 261, so this was always `None`.
    /// The original source has been patched to pass `Some(zombie_threads)` instead.
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
                    // Zombie list includes the exited running thread.
                    && rp.zombie_thread_ids@ ==
                        self.zombie_thread_ids@.push(self.running_thread_id@)
                    && rp.zombie_thread_ids@.len() == 1 + self.spec_zombie_count()
                    // Ready branch: content preserved.
                    && (self.spec_ready_count() > 0 ==> {
                        rp.ready_thread_ids@ == self.ready_thread_ids@
                        && rp.interrupted_thread_ids@ == self.interrupted_thread_ids@
                        && rp.sleeping_thread_ids@ == self.sleeping_thread_ids@
                    })
                    // Interrupted branch: details from strengthened interrupted_resume().
                    && (self.spec_ready_count() == 0 && self.spec_interrupted_count() > 0 ==> {
                        rp.sleeping_thread_ids@ == self.sleeping_thread_ids@
                        && rp.ready_thread_ids@.len() == 1
                        && rp.ready_thread_ids@[0] == self.interrupted_thread_ids@[0]
                        && rp.interrupted_thread_ids@ ==
                            self.interrupted_thread_ids@.subrange(
                                1, self.interrupted_thread_ids@.len() as int)
                    })
                },
                ExitThreadResult::Sleeping(sp) => {
                    sp.spec_pid() == self.spec_pid()
                    && sp.wf()
                    && self.spec_ready_count() == 0
                    && self.spec_interrupted_count() == 0
                    && self.spec_sleeping_count() > 0
                    // Sleeping threads preserved.
                    && sp.sleeping_thread_ids@ == self.sleeping_thread_ids@
                    // Zombie list includes the exited running thread.
                    && sp.zombie_thread_ids@ ==
                        self.zombie_thread_ids@.push(self.running_thread_id@)
                },
                ExitThreadResult::Zombie(zp) => {
                    zp.spec_pid() == self.spec_pid()
                    && zp.wf()
                    && zp.spec_status() == status@
                    && self.spec_ready_count() == 0
                    && self.spec_interrupted_count() == 0
                    && self.spec_sleeping_count() == 0
                    // Zombie list includes the running thread + original zombie.
                    && zp.zombie_thread_ids@ ==
                        self.zombie_thread_ids@.push(self.running_thread_id@)
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
            // Known divergence: original passes self.zombie.take() (=None) here.
            // We correctly pass new_zombie_ids (includes exited thread).
            let ip: InterruptedProcess = InterruptedProcess {
                pid: Ghost(self.pid@),
                interrupted_thread_ids: Ghost(self.interrupted_thread_ids@),
                sleeping_thread_ids: Ghost(self.sleeping_thread_ids@),
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
    ///   This is a trust assumption: callers must provide the correct value.
    ///   The precondition `found == spec_seq_contains(...)` ties it to ghost state.
    ///   In the original, the search is performed by `NonEmptyVecDeque::remove_if()`.
    ///
    /// # Returns
    ///
    /// Ok with updated state if found, Err with unchanged state if not found.
    pub fn wakeup(self, tid: Ghost<int>, found: bool) -> (result: Result<RunningProcess, RunningProcess>)
        requires
            self.wf(),
            found == Self::spec_seq_contains(self.sleeping_thread_ids@, tid@),
            self.ready_count < u64::MAX,
            self.sleeping_count > 0 || !found,
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
