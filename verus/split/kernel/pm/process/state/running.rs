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
//! - `Box<ProcessState>` -> PID (u64, identity tracking only).
//! - `RunningThread` -> u64 (thread ID).
//! - `Option<NonEmptyVecDeque<T>>` -> `Vec<u64>` (empty = None, non-empty = Some).
//! - `ContextInformation*` -> elided (HAL boundary).
//! - `Condvar` -> elided (sync primitive boundary).
//! - `ExitStatus` -> u64.
//! - `alarm: Option<SystemTime>` -> elided. Does not affect state machine logic.
//!
//! ## Trust Boundary
//!
//! - `RunnableProcess`, `SleepingProcess`, `InterruptedProcess`, `ZombieProcess`
//!   are boundary models of sibling modules.
//! - `InterruptedProcess::resume()` is modeled via `interrupted_resume()` (external_body).
//!   It preserves PID, produces a well-formed RunnableProcess, and threads through
//!   sleeping and zombie thread lists from the InterruptedProcess. This assumption
//!   is discharged when `src/kernel/src/pm/process/state/interrupted.rs::resume()`
//!   is independently verified.
//! - Thread state transitions (schedule, sleep, exit) are modeled as ID-preserving.
//! - `find_thread()` / `find_thread_mut()` compute results from `spec_find_thread()`.
//!   The exec-level search correctness (iterator-based linear scan) is a trust assumption
//!   until Verus supports reference-returning functions. `find_thread_mut()` returns
//!   `&mut ThreadRefMut` in the original, permitting mutation of the found thread;
//!   callers must preserve thread identity and list membership after such mutation.
//! - `try_join_thread()` is modeled spec-only via `spec_try_join_thread()`. The key
//!   property: joining a running thread errors, joining a zombie removes it, joining
//!   a live thread returns a condvar, joining a missing thread errors.
//! - `state()` / `state_mut()` are elided (ProcessState access modeled via PID).
//!   `state_mut()` permits arbitrary mutation; callers must preserve PID immutability.
//!   Discharged when `ProcessState` is independently verified.
//! - `running_mut()` returns `&mut RunningThread`, permitting arbitrary mutation.
//!   Callers must preserve the running thread's ID (`spec_running_thread_id()`)
//!   and any structural invariants. Discharged when `RunningThread` is independently verified.
//!
//! ## Oracle Parameters
//!
//! `wakeup()` and `try_join_thread()` take oracle parameters (`found: bool`, `tag: u8`)
//! because the exec-level search through concrete `Vec<u64>` would require loop
//! invariants at every call site. The preconditions
//! (`found == spec_seq_contains(...)`, `tag == spec_try_join_thread(tid)`) are verified
//! by Verus at every call site. **All callers of these functions must be verified
//! (not `external_body` or `assume`) for the oracle contracts to hold.** If a caller
//! is itself `external_body`, the oracle constraint becomes a trust assumption.
//!
//! ## Modeling Assumptions
//!
//! - `ready_count < u64::MAX` in `wakeup()`: prevents arithmetic overflow on
//!   `ready_count + 1`. Real systems never approach 2^64 threads per process.
//! - `ContextInformation`, `Condvar`, `SystemTime` (alarm) are elided. These affect
//!   HAL context switching, synchronization, and timing but not process state machine
//!   logic. If HAL or sync correctness is ever verified, these elisions must be revisited.
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
//!
//! ## Concrete Types
//!
//! All struct fields use concrete types matching simplified representations of
//! the original kernel types:
//! - `Box<ProcessState>` → `u64` (PID).
//! - `RunningThread` → `u64` (thread ID).
//! - `Option<NonEmptyVecDeque<T>>` → `Vec<u64>` (thread ID list).
//! - `ExitStatus` → `u64`.

use vstd::prelude::*;

// Include specifications.
include!("running.spec.rs");

// Include proofs.
include!("running.proof.rs");

verus! {

//==================================================================================================
// Helper Functions
//==================================================================================================

/// Appends all elements from `src` to `dst`.
fn vec_push_all(dst: &mut Vec<u64>, src: &Vec<u64>)
    ensures
        dst@ =~= old(dst)@.add(src@),
{
    let src_len: usize = src.len();
    let mut i: usize = 0;
    while i < src_len
        invariant
            0 <= i <= src_len,
            src_len == src@.len(),
            dst@ =~= old(dst)@.add(src@.subrange(0, i as int)),
        decreases src_len - i,
    {
        dst.push(src[i]);
        proof {
            assert(src@.subrange(0, i as int).push(src@[i as int])
                =~= src@.subrange(0, (i + 1) as int));
        }
        i = i + 1;
    }
    proof {
        assert(src@.subrange(0, src_len as int) =~= src@);
    }
}

/// Creates a new Vec with the element at `skip` removed.
fn vec_remove_at(v: &Vec<u64>, skip: usize) -> (result: Vec<u64>)
    requires
        skip < v@.len(),
    ensures
        result@ =~= v@.subrange(0, skip as int).add(
            v@.subrange(skip as int + 1, v@.len() as int)),
{
    let mut result: Vec<u64> = Vec::new();
    let len: usize = v.len();
    let mut i: usize = 0;
    while i < skip
        invariant
            0 <= i <= skip,
            skip < len,
            len == v@.len(),
            result@ =~= v@.subrange(0, i as int),
        decreases skip - i,
    {
        result.push(v[i]);
        proof {
            assert(v@.subrange(0, i as int).push(v@[i as int])
                =~= v@.subrange(0, (i + 1) as int));
        }
        i = i + 1;
    }
    proof {
        assert(v@.subrange(skip as int + 1, (skip + 1) as int).len() == 0);
        assert(v@.subrange(0, skip as int).add(
            v@.subrange(skip as int + 1, (skip + 1) as int))
            =~= v@.subrange(0, skip as int));
    }
    i = skip + 1;
    while i < len
        invariant
            skip < len,
            len == v@.len(),
            skip as int + 1 <= i as int,
            i <= len,
            result@ =~= v@.subrange(0, skip as int).add(
                v@.subrange(skip as int + 1, i as int)),
        decreases len - i,
    {
        result.push(v[i]);
        proof {
            assert(v@.subrange(skip as int + 1, i as int).push(v@[i as int])
                =~= v@.subrange(skip as int + 1, (i + 1) as int));
            assert(v@.subrange(0, skip as int).add(
                v@.subrange(skip as int + 1, i as int)).push(v@[i as int])
                =~= v@.subrange(0, skip as int).add(
                    v@.subrange(skip as int + 1, (i + 1) as int)));
        }
        i = i + 1;
    }
    result
}

//==================================================================================================
// Structures
//==================================================================================================

/// A process that is currently running.
///
/// Verification model of `src/kernel/src/pm/process/state/running.rs::RunningProcess`.
/// Thread collections are modeled as concrete Vec of thread IDs.
/// Exec-level counters track Vec lengths for efficient branch decisions.
pub struct RunningProcess {
    /// Process identifier (from the inner ProcessState).
    pub pid: u64,
    /// Running thread ID.
    pub running_thread_id: u64,
    /// Concrete vector of ready thread IDs.
    pub ready_thread_ids: Vec<u64>,
    /// Concrete vector of interrupted thread IDs.
    pub interrupted_thread_ids: Vec<u64>,
    /// Concrete vector of sleeping thread IDs.
    pub sleeping_thread_ids: Vec<u64>,
    /// Concrete vector of zombie thread IDs.
    pub zombie_thread_ids: Vec<u64>,
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
    pub pid: u64,
    /// Ready thread IDs (non-empty).
    pub ready_thread_ids: Vec<u64>,
    /// Interrupted thread IDs.
    pub interrupted_thread_ids: Vec<u64>,
    /// Sleeping thread IDs.
    pub sleeping_thread_ids: Vec<u64>,
    /// Zombie thread IDs.
    pub zombie_thread_ids: Vec<u64>,
}

/// A process that is sleeping (boundary model).
///
/// Models `SleepingProcess` from the sibling module.
pub struct SleepingProcess {
    /// Process identifier.
    pub pid: u64,
    /// Sleeping thread IDs (non-empty).
    pub sleeping_thread_ids: Vec<u64>,
    /// Zombie thread IDs.
    pub zombie_thread_ids: Vec<u64>,
}

/// A process that was interrupted (boundary model).
///
/// Models `InterruptedProcess` from the sibling module. Includes
/// `sleeping_thread_ids` to match the original struct which carries
/// sleeping threads through `from_sleeping()` and `resume()`.
pub struct InterruptedProcess {
    /// Process identifier.
    pub pid: u64,
    /// Interrupted thread IDs (non-empty).
    pub interrupted_thread_ids: Vec<u64>,
    /// Sleeping thread IDs (carried through resume).
    pub sleeping_thread_ids: Vec<u64>,
    /// Zombie thread IDs.
    pub zombie_thread_ids: Vec<u64>,
}

/// A process that has terminated (boundary model).
///
/// Models `ZombieProcess` from the sibling module.
pub struct ZombieProcess {
    /// Process identifier.
    pub pid: u64,
    /// Zombie thread IDs (non-empty).
    pub zombie_thread_ids: Vec<u64>,
    /// Exit status.
    pub status: u64,
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
/// Original source: `src/kernel/src/pm/process/state/interrupted.rs::resume()`.
/// This external_body is discharged when that function is independently verified.
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
        pid: u64,
        running_tid: u64,
        ready_ids: Vec<u64>,
        interrupted_ids: Vec<u64>,
        sleeping_ids: Vec<u64>,
        zombie_ids: Vec<u64>,
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
            result.spec_pid() == pid,
            result.spec_running_thread_id() == running_tid,
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
    /// The thread identifier of the running thread.
    pub fn get_tid(&self) -> (result: u64)
        ensures
            result == self.spec_running_thread_id(),
    {
        self.running_thread_id
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
    pub fn state(&self) -> (result: u64)
        ensures
            result == self.spec_pid(),
    {
        unimplemented!()
    }

    /// Returns a mutable reference to the process state.
    ///
    /// Models the original `RunningProcess::state_mut()`.
    /// The original returns `&mut ProcessState`, which permits mutation of
    /// ProcessState fields. Since we model ProcessState only as a PID, we
    /// return u64. Callers must ensure `mutation_frame_preserved()`
    /// holds after any mutation (PID and all thread lists unchanged).
    ///
    /// # Returns
    ///
    /// The process identifier.
    #[verifier::external_body]
    pub fn state_mut(&mut self) -> (result: u64)
        ensures
            result == self.spec_pid(),
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
    /// only as a thread ID, we return u64. Callers must preserve the
    /// running thread's ID and all structural invariants.
    ///
    /// # Returns
    ///
    /// The running thread identifier.
    #[verifier::external_body]
    pub fn running_mut(&mut self) -> (result: u64)
        ensures
            result == self.spec_running_thread_id(),
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
    /// Returns the result tag matching `spec_try_join_thread()`:
    /// - `0`: Thread was zombie and has been removed from zombie list.
    /// - `1`: Thread is the running thread (OperationNotPermitted).
    /// - `2`: Thread is live (ready/sleeping/interrupted), returns condvar.
    /// - `3`: Thread not found (NoSuchProcess).
    ///
    /// On successful zombie join (tag=0), `self` is mutated: the zombie
    /// list shrinks by one (the joined thread is removed).
    ///
    /// ## Oracle Parameter
    ///
    /// The `tag` parameter is required because this function performs exec-level
    /// mutation (zombie_count decrement, zombie_thread_ids update) that requires
    /// an exec-level branch decision. The precondition `tag == spec_try_join_thread(tid)`
    /// is verified by Verus at every call site, ensuring callers cannot pass
    /// inconsistent values. In the original code, the search is performed by
    /// iterating over `NonEmptyVecDeque` collections.
    ///
    /// # Parameters
    ///
    /// - `tid`: Thread identifier to join.
    /// - `tag`: Oracle — the join result. Must equal `spec_try_join_thread(tid)`.
    ///
    /// # Returns
    ///
    /// The result tag.
    pub fn try_join_thread(&mut self, tid: u64, tag: u8) -> (result: u8)
        requires
            old(self).wf(),
            tag as int == old(self).spec_try_join_thread(tid),
        ensures
            result == tag,
            result as int == old(self).spec_try_join_thread(tid),
            // PID and running thread unchanged.
            self.spec_pid() == old(self).spec_pid(),
            self.spec_running_thread_id() == old(self).spec_running_thread_id(),
            // Ready, interrupted, sleeping lists unchanged.
            self.ready_thread_ids@ == old(self).ready_thread_ids@,
            self.interrupted_thread_ids@ == old(self).interrupted_thread_ids@,
            self.sleeping_thread_ids@ == old(self).sleeping_thread_ids@,
            self.ready_count == old(self).ready_count,
            self.interrupted_count == old(self).interrupted_count,
            self.sleeping_count == old(self).sleeping_count,
            // Zombie list: removed on success, unchanged otherwise.
            (tag == JOIN_TAG_ZOMBIE) ==> (
                (exists|idx: int|
                    0 <= idx < old(self).zombie_thread_ids@.len()
                    && old(self).zombie_thread_ids@[idx] == tid
                    && self.zombie_thread_ids@ ==
                        Self::spec_remove_at(old(self).zombie_thread_ids@, idx))
                && self.zombie_count as nat == old(self).spec_zombie_count() - 1
            ),
            (tag != JOIN_TAG_ZOMBIE) ==> (
                self.zombie_thread_ids@ == old(self).zombie_thread_ids@
                && self.zombie_count == old(self).zombie_count
            ),
            self.wf(),
    {
        if tag == JOIN_TAG_ZOMBIE {
            // Zombie found — remove it from the zombie list.
            proof {
                assert(old(self).spec_has_zombie_thread(tid));
                assert(old(self).zombie_thread_ids@.len() > 0);
                assert(old(self).zombie_count > 0);
            }

            // Find the first occurrence of tid using a flag (no break).
            let zlen: usize = self.zombie_thread_ids.len();
            let mut idx: usize = 0;
            let mut found_it: bool = false;
            while idx < zlen && !found_it
                invariant
                    0 <= idx <= zlen,
                    zlen == old(self).zombie_thread_ids@.len(),
                    self.zombie_thread_ids@ =~= old(self).zombie_thread_ids@,
                    !found_it ==> forall|j: int| 0 <= j < idx as int
                        ==> self.zombie_thread_ids@[j] != tid,
                    found_it ==> (
                        idx < zlen
                        && self.zombie_thread_ids@[idx as int] == tid
                        && forall|j: int| 0 <= j < idx as int
                            ==> self.zombie_thread_ids@[j] != tid
                    ),
                    Self::spec_seq_contains(old(self).zombie_thread_ids@, tid),
                    self.pid == old(self).pid,
                    self.running_thread_id == old(self).running_thread_id,
                    self.ready_thread_ids@ =~= old(self).ready_thread_ids@,
                    self.interrupted_thread_ids@ =~= old(self).interrupted_thread_ids@,
                    self.sleeping_thread_ids@ =~= old(self).sleeping_thread_ids@,
                    self.ready_count == old(self).ready_count,
                    self.interrupted_count == old(self).interrupted_count,
                    self.sleeping_count == old(self).sleeping_count,
                    self.zombie_count == old(self).zombie_count,
                decreases zlen - idx, if found_it { 0int } else { 1int },
            {
                if self.zombie_thread_ids[idx] == tid {
                    found_it = true;
                } else {
                    idx = idx + 1;
                }
            }

            proof {
                // After loop: found_it must be true (otherwise all elements != tid,
                // contradicting spec_seq_contains).
                if !found_it {
                    assert(idx >= zlen);
                    assert(forall|j: int| 0 <= j < zlen as int
                        ==> self.zombie_thread_ids@[j] != tid);
                    // This contradicts spec_seq_contains.
                    assert(false);
                }
                assert(found_it);
                assert(idx < zlen);
                assert(self.zombie_thread_ids@[idx as int] == tid);
            }

            let new_zombies: Vec<u64> = vec_remove_at(&self.zombie_thread_ids, idx);

            proof {
                // Prove the new zombie list length.
                let s: Seq<u64> = old(self).zombie_thread_ids@;
                Self::lemma_remove_at_length(s, idx as int);
            }

            self.zombie_thread_ids = new_zombies;
            self.zombie_count = self.zombie_count - 1;

            proof {
                // Witness for the existential in ensures.
                assert(0 <= idx as int && (idx as int) < old(self).zombie_thread_ids@.len());
                assert(old(self).zombie_thread_ids@[idx as int] == tid);
                assert(self.zombie_thread_ids@ =~=
                    Self::spec_remove_at(old(self).zombie_thread_ids@, idx as int));
            }

            JOIN_TAG_ZOMBIE
        } else {
            tag
        }
    }

    /// Finds a thread by its identifier and returns which list it belongs to.
    ///
    /// Models the original `RunningProcess::find_thread(tid)`.
    /// The original returns `Option<ThreadRef>` (a reference enum). Since
    /// Verus cannot express reference-returning functions, we return the
    /// abstract list variant from `spec_find_thread()` as a ghost value:
    /// - `Some(0)`: running thread.
    /// - `Some(1)`: ready thread.
    /// - `Some(2)`: interrupted thread.
    /// - `Some(3)`: sleeping thread.
    /// - `Some(4)`: zombie thread.
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
    /// Models the original `RunningProcess::find_thread_mut(tid)`.
    /// Same semantics as `find_thread()` — returns the ghost list variant.
    /// The mutable reference in the original allows in-place mutation of
    /// the found thread, but this does not change the thread's identity
    /// or list membership. Frame condition: self is unchanged.
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
            // Frame: find_thread_mut does not change any modeled fields.
            self.spec_pid() == old(self).spec_pid(),
            self.spec_running_thread_id() == old(self).spec_running_thread_id(),
            self.ready_thread_ids@ == old(self).ready_thread_ids@,
            self.interrupted_thread_ids@ == old(self).interrupted_thread_ids@,
            self.sleeping_thread_ids@ == old(self).sleeping_thread_ids@,
            self.zombie_thread_ids@ == old(self).zombie_thread_ids@,
            self.wf() == old(self).wf(),
    {
        Ghost(old(self).spec_find_thread(tid))
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
                self.ready_thread_ids@.push(self.running_thread_id),
            // Other lists preserved.
            result.process.interrupted_thread_ids@ == self.interrupted_thread_ids@,
            result.process.sleeping_thread_ids@ == self.sleeping_thread_ids@,
            result.process.zombie_thread_ids@ == self.zombie_thread_ids@,
            // Result is well-formed (non-empty ready list).
            result.process.wf(),
    {
        let RunningProcess {
            pid, running_thread_id, mut ready_thread_ids, interrupted_thread_ids,
            sleeping_thread_ids, zombie_thread_ids, ready_count, interrupted_count,
            sleeping_count, zombie_count,
        } = self;

        ready_thread_ids.push(running_thread_id);

        proof {
            assert(ready_thread_ids@.len() >= 1);
        }

        ScheduleResult {
            process: RunnableProcess {
                pid,
                ready_thread_ids,
                interrupted_thread_ids,
                sleeping_thread_ids,
                zombie_thread_ids,
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
                    && rp.sleeping_thread_ids@.len() == self.spec_sleeping_count() + 1
                    // Zombie threads preserved.
                    && rp.zombie_thread_ids@ == self.zombie_thread_ids@
                    // Ready branch: exact content specified.
                    && (self.spec_ready_count() > 0 ==> {
                        rp.ready_thread_ids@ == self.ready_thread_ids@
                        && rp.sleeping_thread_ids@ ==
                            self.sleeping_thread_ids@.push(self.running_thread_id)
                        && rp.interrupted_thread_ids@ == self.interrupted_thread_ids@
                    })
                    // Interrupted branch: details from strengthened interrupted_resume().
                    && (self.spec_ready_count() == 0 && self.spec_interrupted_count() > 0 ==> {
                        rp.sleeping_thread_ids@ ==
                            self.sleeping_thread_ids@.push(self.running_thread_id)
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
                        self.sleeping_thread_ids@.push(self.running_thread_id)
                    && sp.sleeping_thread_ids@.len() ==
                        self.spec_sleeping_count() + 1
                    // Zombie threads preserved.
                    && sp.zombie_thread_ids@ == self.zombie_thread_ids@
                },
            },
    {
        let RunningProcess {
            pid, running_thread_id, ready_thread_ids, interrupted_thread_ids,
            mut sleeping_thread_ids, zombie_thread_ids, ready_count, interrupted_count,
            sleeping_count, zombie_count,
        } = self;

        sleeping_thread_ids.push(running_thread_id);

        proof {
            assert(sleeping_thread_ids@.len() >= 1);
        }

        // Check if there are ready threads.
        if ready_count > 0 {
            return SleepResult::Runnable(RunnableProcess {
                pid,
                ready_thread_ids,
                interrupted_thread_ids,
                sleeping_thread_ids,
                zombie_thread_ids,
            });
        }

        // Check if there are interrupted threads.
        if interrupted_count > 0 {
            proof {
                assert(interrupted_thread_ids@.len() >= 1);
            }
            let ip: InterruptedProcess = InterruptedProcess {
                pid,
                interrupted_thread_ids,
                sleeping_thread_ids,
                zombie_thread_ids,
            };
            let rp: RunnableProcess = interrupted_resume(ip);
            return SleepResult::Runnable(rp);
        }

        // No ready or interrupted threads — become sleeping.
        SleepResult::Sleeping(SleepingProcess {
            pid,
            sleeping_thread_ids,
            zombie_thread_ids,
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
    pub fn exit(self, status: u64) -> (result: ExitResult)
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
                    // Zombie threads in result: original zombie + running + ready (matches original ordering).
                    && rp.zombie_thread_ids@.len() ==
                        1 + self.spec_ready_count() + self.spec_zombie_count()
                    && rp.zombie_thread_ids@ ==
                        self.zombie_thread_ids@.push(self.running_thread_id).add(
                            self.ready_thread_ids@)
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
                    && zp.spec_status() == status
                    // Branch: no interrupted or sleeping threads.
                    && self.spec_interrupted_count() == 0
                    && self.spec_sleeping_count() == 0
                    // Zombie list: original zombie + running + all ready (matches original ordering).
                    && zp.zombie_thread_ids@.len() ==
                        1 + self.spec_ready_count() + self.spec_zombie_count()
                    && zp.zombie_thread_ids@ ==
                        self.zombie_thread_ids@.push(self.running_thread_id).add(
                            self.ready_thread_ids@)
                },
            },
    {
        let RunningProcess {
            pid, running_thread_id, ready_thread_ids, mut interrupted_thread_ids,
            sleeping_thread_ids, mut zombie_thread_ids, ready_count, interrupted_count,
            sleeping_count, zombie_count,
        } = self;

        // Running thread becomes zombie. Original: push_back onto existing zombies.
        zombie_thread_ids.push(running_thread_id);
        // All ready threads become zombies. Original: append ready zombies after.
        vec_push_all(&mut zombie_thread_ids, &ready_thread_ids);

        proof {
            assert(zombie_thread_ids@.len() >= 1);
        }

        // Sleeping threads become interrupted.
        vec_push_all(&mut interrupted_thread_ids, &sleeping_thread_ids);

        if interrupted_count > 0 || sleeping_count > 0 {
            proof {
                assert(interrupted_thread_ids@.len() >= 1);
            }
            // In the original, self.sleeping_threads was already taken (line 208),
            // so InterruptedProcess::from_sleeping gets None for sleeping. We model
            // this faithfully with empty sleeping_thread_ids.
            let ip: InterruptedProcess = InterruptedProcess {
                pid,
                interrupted_thread_ids,
                sleeping_thread_ids: Vec::new(),
                zombie_thread_ids,
            };
            let rp: RunnableProcess = interrupted_resume(ip);
            ExitResult::Runnable(rp)
        } else {
            proof {
                assert(interrupted_thread_ids@.len() == 0);
            }
            ExitResult::Zombie(ZombieProcess {
                pid,
                zombie_thread_ids,
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
    pub fn exit_thread(self, status: u64) -> (result: ExitThreadResult)
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
                        self.zombie_thread_ids@.push(self.running_thread_id)
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
                        self.zombie_thread_ids@.push(self.running_thread_id)
                },
                ExitThreadResult::Zombie(zp) => {
                    zp.spec_pid() == self.spec_pid()
                    && zp.wf()
                    && zp.spec_status() == status
                    && self.spec_ready_count() == 0
                    && self.spec_interrupted_count() == 0
                    && self.spec_sleeping_count() == 0
                    // Zombie list includes the running thread + original zombie.
                    && zp.zombie_thread_ids@ ==
                        self.zombie_thread_ids@.push(self.running_thread_id)
                    && zp.zombie_thread_ids@.len() == 1 + self.spec_zombie_count()
                },
            },
    {
        let RunningProcess {
            pid, running_thread_id, ready_thread_ids, interrupted_thread_ids,
            sleeping_thread_ids, mut zombie_thread_ids, ready_count, interrupted_count,
            sleeping_count, zombie_count,
        } = self;

        // Running thread becomes zombie.
        zombie_thread_ids.push(running_thread_id);

        proof {
            assert(zombie_thread_ids@.len() >= 1);
        }

        if ready_count > 0 {
            return ExitThreadResult::Runnable(RunnableProcess {
                pid,
                ready_thread_ids,
                interrupted_thread_ids,
                sleeping_thread_ids,
                zombie_thread_ids,
            });
        }

        if interrupted_count > 0 {
            proof {
                assert(interrupted_thread_ids@.len() >= 1);
            }
            // Historical: original passed self.zombie.take() (=None) here. Now fixed in source.
            // We correctly pass zombie_thread_ids (includes exited thread).
            let ip: InterruptedProcess = InterruptedProcess {
                pid,
                interrupted_thread_ids,
                sleeping_thread_ids,
                zombie_thread_ids,
            };
            let rp: RunnableProcess = interrupted_resume(ip);
            return ExitThreadResult::Runnable(rp);
        }

        if sleeping_count > 0 {
            return ExitThreadResult::Sleeping(SleepingProcess {
                pid,
                sleeping_thread_ids,
                zombie_thread_ids,
            });
        }

        ExitThreadResult::Zombie(ZombieProcess {
            pid,
            zombie_thread_ids,
            status,
        })
    }

    /// Wakes up a sleeping thread and moves it to the ready queue.
    ///
    /// Models the original `RunningProcess::wakeup(tid)`.
    /// Returns Ok(RunningProcess) on success, Err(RunningProcess) if not found.
    ///
    /// ## Oracle Parameter
    ///
    /// The `found` parameter is required because this function performs exec-level
    /// mutation (ready_count increment, sleeping_count decrement) that requires
    /// an exec-level branch decision. The precondition `found == spec_seq_contains(...)`
    /// is verified by Verus at every call site, ensuring callers cannot pass
    /// inconsistent values. In the original code, the search is performed by
    /// `NonEmptyVecDeque::remove_if()`.
    ///
    /// # Parameters
    ///
    /// - `tid`: Thread ID to wake up.
    /// - `found`: Oracle — whether the thread is in the sleeping list.
    ///   Must equal `spec_seq_contains(sleeping_thread_ids, tid)`.
    ///
    /// # Returns
    ///
    /// Ok with updated state if found, Err with unchanged state if not found.
    pub fn wakeup(self, tid: u64, found: bool) -> (result: Result<RunningProcess, RunningProcess>)
        requires
            self.wf(),
            found == Self::spec_seq_contains(self.sleeping_thread_ids@, tid),
            self.ready_count < u64::MAX,
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
                    && r.ready_thread_ids@ == self.ready_thread_ids@.push(tid)
                    // Sleeping list has the found thread removed.
                    && (exists|idx: int| 0 <= idx < self.sleeping_thread_ids@.len()
                        && self.sleeping_thread_ids@[idx] == tid
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
            return Err(self);
        }

        let RunningProcess {
            pid, running_thread_id, mut ready_thread_ids, interrupted_thread_ids,
            sleeping_thread_ids, zombie_thread_ids, ready_count, interrupted_count,
            sleeping_count, zombie_count,
        } = self;

        proof {
            // Derive sleeping_count > 0 from wf() and found == spec_seq_contains.
            assert(sleeping_thread_ids@.len() > 0);
            assert(sleeping_count > 0);
        }

        // Find the first occurrence of tid in sleeping_thread_ids.
        let slen: usize = sleeping_thread_ids.len();
        let mut idx: usize = 0;
        let mut found_it: bool = false;
        while idx < slen && !found_it
            invariant
                0 <= idx <= slen,
                slen == sleeping_thread_ids@.len(),
                !found_it ==> forall|j: int| 0 <= j < idx as int
                    ==> sleeping_thread_ids@[j] != tid,
                found_it ==> (
                    idx < slen
                    && sleeping_thread_ids@[idx as int] == tid
                    && forall|j: int| 0 <= j < idx as int
                        ==> sleeping_thread_ids@[j] != tid
                ),
                Self::spec_seq_contains(sleeping_thread_ids@, tid),
            decreases slen - idx, if found_it { 0int } else { 1int },
        {
            if sleeping_thread_ids[idx] == tid {
                found_it = true;
            } else {
                idx = idx + 1;
            }
        }

        proof {
            // After loop: found_it must be true.
            if !found_it {
                assert(idx >= slen);
                assert(forall|j: int| 0 <= j < slen as int
                    ==> sleeping_thread_ids@[j] != tid);
                assert(false);
            }
            assert(found_it);
            assert(idx < slen);
            assert(sleeping_thread_ids@[idx as int] == tid);
        }

        // Remove tid from sleeping list.
        let new_sleeping: Vec<u64> = vec_remove_at(&sleeping_thread_ids, idx);

        proof {
            // Prove new sleeping length.
            let s: Seq<u64> = sleeping_thread_ids@;
            let left: Seq<u64> = s.subrange(0, idx as int);
            let right: Seq<u64> = s.subrange(idx as int + 1, s.len() as int);
            assert(left.len() == idx as nat);
            assert(right.len() == (s.len() - idx as nat - 1) as nat);
            assert(left.add(right).len() == (s.len() - 1) as nat);
        }

        // Push tid onto ready.
        ready_thread_ids.push(tid);

        proof {
            assert(ready_thread_ids@.len() == ready_count as int + 1);
        }

        Ok(RunningProcess {
            pid,
            running_thread_id,
            ready_thread_ids,
            interrupted_thread_ids,
            sleeping_thread_ids: new_sleeping,
            zombie_thread_ids,
            ready_count: ready_count + 1,
            interrupted_count,
            sleeping_count: sleeping_count - 1,
            zombie_count,
        })
    }
}

} // verus!
