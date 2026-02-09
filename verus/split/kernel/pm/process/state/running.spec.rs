// Copyright(c) The Maintainers of Nanvix.
// Licensed under the MIT License.

// RunningProcess Specification.
// This file contains spec functions and View types for the RunningProcess type.
//
// ## Verification Model
//
// RunningProcess has exactly one running thread and optional collections of
// ready, interrupted, sleeping, and zombie threads. For verification:
// - The running thread is modeled as a single ghost int (thread ID).
// - Thread lists are modeled as `Seq<int>` of abstract thread IDs.
// - `Option<NonEmptyVecDeque<T>>` is modeled as `Seq<int>`:
//   - `Seq::empty()` represents `None`.
//   - `Seq` with `len() >= 1` represents `Some(non_empty_deque)`.
// - `Box<ProcessState>` is transparent (modeled as PID only).
// - `ContextInformation*` is elided (HAL boundary).
// - `Condvar` is elided (sync primitive boundary).
// - `ExitStatus` is modeled as `int`.
//
// ## Key Invariants
//
// - A RunningProcess always has exactly one running thread.
// - Process identity (PID) is immutable across all operations.
// - `schedule()` moves running→ready, producing a RunnableProcess.
// - `sleep()` moves running→sleeping; produces Runnable (if ready threads exist),
//   Runnable (if interrupted threads exist, via InterruptedProcess.resume()),
//   or Sleeping (if no other threads).
// - `exit()` moves running→zombie + terminates all ready + interrupts all sleeping.
// - `exit_thread()` moves running→zombie for just the running thread.
// - `wakeup()` moves a sleeping thread to ready within this process.
// - `get_tid()` returns the running thread's identifier.
//
// ## Ownership Semantics (Trust Assumption)
//
// Thread ID uniqueness across lists is NOT enforced in `wf()`. In the original
// code, the Rust type system ensures ownership semantics — a thread struct can
// only be in one `NonEmptyVecDeque` at a time. An optional `wf_strict()`
// predicate is provided for downstream cross-module proofs that require
// thread ID uniqueness.
//
// ## Trust Assumptions
//
// - `ContextInformation` pointers from schedule/sleep/exit are omitted (HAL boundary).
// - `Condvar` from join_cond/exit_thread is omitted (sync boundary).
// - `alarm: Option<SystemTime>` parameter in `sleep()` is elided. The alarm
//   affects the `SleepingThread`'s wakeup behavior but not the process state
//   machine logic verified here. If alarm-based temporal properties are ever
//   verified, this elision must be revisited.
// - Thread state transitions (RunningThread::schedule(), sleep(), exit()) are
//   modeled as ID-preserving operations.
// - `SleepingProcess`, `RunnableProcess`, `InterruptedProcess`, `ZombieProcess`
//   are boundary models from sibling modules.
// - `wakeup()` takes a `found: bool` oracle parameter because `Seq::contains()`
//   is spec-only and cannot be evaluated at exec level. The precondition
//   `found == spec_seq_contains(...)` ties the oracle to ghost state. Callers
//   must be trusted to provide the correct value. The search in the original
//   code is performed by `NonEmptyVecDeque::remove_if()`.
// - `find_thread()` and `find_thread_mut()` are modeled spec-only because they
//   return reference types (`ThreadRef`, `ThreadRefMut`) that Verus cannot
//   express. The spec model `spec_find_thread()` captures the exhaustive search
//   semantics and correct list-variant selection. Exec-level implementations
//   should be independently verified against this spec when reference types
//   become expressible in Verus.
// - `running_mut()` returns `&mut RunningThread`, permitting arbitrary mutation
//   of the running thread's internal state. This is a trust boundary: callers
//   must preserve the running thread's ID (`spec_running_thread_id()` unchanged)
//   and any structural invariants assumed by this module. This obligation is
//   discharged when `RunningThread` is independently verified.
// - `state()` / `state_mut()` return references to ProcessState. Modeled via PID
//   only. `state_mut()` permits arbitrary mutation; callers must preserve PID
//   immutability.

use vstd::prelude::*;

verus! {

//==================================================================================================
// View Types
//==================================================================================================

/// Abstract view of a RunningProcess (this module's primary type).
#[verifier::ext_equal]
pub struct RunningProcessView {
    /// Process identifier value.
    pub pid: int,
    /// The running thread ID.
    pub running_thread_id: int,
    /// Ready thread IDs (may be empty).
    pub ready_thread_ids: Seq<int>,
    /// Interrupted thread IDs (may be empty).
    pub interrupted_thread_ids: Seq<int>,
    /// Sleeping thread IDs (may be empty).
    pub sleeping_thread_ids: Seq<int>,
    /// Zombie thread IDs (may be empty).
    pub zombie_thread_ids: Seq<int>,
}

/// Abstract view of a SleepingProcess (boundary type).
#[verifier::ext_equal]
pub struct SleepingProcessView {
    /// Process identifier value.
    pub pid: int,
    /// Sleeping thread IDs (non-empty).
    pub sleeping_thread_ids: Seq<int>,
    /// Zombie thread IDs (may be empty).
    pub zombie_thread_ids: Seq<int>,
}

/// Abstract view of a RunnableProcess (boundary type).
#[verifier::ext_equal]
pub struct RunnableProcessView {
    /// Process identifier value.
    pub pid: int,
    /// Ready thread IDs (non-empty).
    pub ready_thread_ids: Seq<int>,
    /// Interrupted thread IDs (may be empty).
    pub interrupted_thread_ids: Seq<int>,
    /// Sleeping thread IDs (may be empty).
    pub sleeping_thread_ids: Seq<int>,
    /// Zombie thread IDs (may be empty).
    pub zombie_thread_ids: Seq<int>,
}

/// Abstract view of an InterruptedProcess (boundary type).
///
/// Includes `sleeping_thread_ids` to match the original `InterruptedProcess`
/// struct which has a `sleeping_threads: Option<NonEmptyVecDeque<SleepingThread>>`
/// field. This is critical for `sleep()`'s interrupted branch, where sleeping
/// threads are passed through `InterruptedProcess::from_sleeping()` and
/// preserved through `resume()` into the resulting `RunnableProcess`.
#[verifier::ext_equal]
pub struct InterruptedProcessView {
    /// Process identifier value.
    pub pid: int,
    /// Interrupted thread IDs (non-empty).
    pub interrupted_thread_ids: Seq<int>,
    /// Sleeping thread IDs (may be empty).
    pub sleeping_thread_ids: Seq<int>,
    /// Zombie thread IDs (may be empty).
    pub zombie_thread_ids: Seq<int>,
}

/// Abstract view of a ZombieProcess (boundary type).
#[verifier::ext_equal]
pub struct ZombieProcessView {
    /// Process identifier value.
    pub pid: int,
    /// Zombie thread IDs (non-empty).
    pub zombie_thread_ids: Seq<int>,
    /// Exit status.
    pub status: int,
}

//==================================================================================================
// Spec Functions: RunningProcess
//==================================================================================================

impl RunningProcess {
    /// Spec function: returns the process identifier value.
    pub open spec fn spec_pid(&self) -> int {
        self.pid@
    }

    /// Spec function: returns the running thread ID.
    pub open spec fn spec_running_thread_id(&self) -> int {
        self.running_thread_id@
    }

    /// Spec function: returns the number of ready threads.
    pub open spec fn spec_ready_count(&self) -> nat {
        self.ready_thread_ids@.len()
    }

    /// Spec function: returns the number of interrupted threads.
    pub open spec fn spec_interrupted_count(&self) -> nat {
        self.interrupted_thread_ids@.len()
    }

    /// Spec function: returns the number of sleeping threads.
    pub open spec fn spec_sleeping_count(&self) -> nat {
        self.sleeping_thread_ids@.len()
    }

    /// Spec function: returns the number of zombie threads.
    pub open spec fn spec_zombie_count(&self) -> nat {
        self.zombie_thread_ids@.len()
    }

    /// Spec function: returns the total number of threads (including the running one).
    pub open spec fn spec_total_thread_count(&self) -> nat {
        1 + self.spec_ready_count()
        + self.spec_interrupted_count()
        + self.spec_sleeping_count()
        + self.spec_zombie_count()
    }

    /// Spec function: checks if a thread ID is in the ready list.
    pub open spec fn spec_has_ready_thread(&self, tid: int) -> bool {
        exists|i: int| 0 <= i < self.ready_thread_ids@.len()
            && self.ready_thread_ids@[i] == tid
    }

    /// Spec function: checks if a thread ID is in the sleeping list.
    pub open spec fn spec_has_sleeping_thread(&self, tid: int) -> bool {
        exists|i: int| 0 <= i < self.sleeping_thread_ids@.len()
            && self.sleeping_thread_ids@[i] == tid
    }

    /// Spec function: checks if a thread ID is in the interrupted list.
    pub open spec fn spec_has_interrupted_thread(&self, tid: int) -> bool {
        exists|i: int| 0 <= i < self.interrupted_thread_ids@.len()
            && self.interrupted_thread_ids@[i] == tid
    }

    /// Spec function: checks if a thread ID is in the zombie list.
    pub open spec fn spec_has_zombie_thread(&self, tid: int) -> bool {
        exists|i: int| 0 <= i < self.zombie_thread_ids@.len()
            && self.zombie_thread_ids@[i] == tid
    }

    /// Spec function: checks if a thread ID is in any list (including running).
    pub open spec fn spec_has_thread(&self, tid: int) -> bool {
        self.running_thread_id@ == tid
        || self.spec_has_ready_thread(tid)
        || self.spec_has_sleeping_thread(tid)
        || self.spec_has_interrupted_thread(tid)
        || self.spec_has_zombie_thread(tid)
    }

    /// Spec function: models `find_thread()` — returns which list a thread is in.
    ///
    /// - `Some(0)` if it is the running thread.
    /// - `Some(1)` if found in ready threads.
    /// - `Some(2)` if found in interrupted threads.
    /// - `Some(3)` if found in sleeping threads.
    /// - `Some(4)` if found in zombie threads.
    /// - `None` if not found.
    ///
    /// Search order matches original: running → ready → interrupted → sleeping → zombie.
    pub open spec fn spec_find_thread(&self, tid: int) -> Option<int> {
        if self.running_thread_id@ == tid {
            Some(0int)
        } else if self.spec_has_ready_thread(tid) {
            Some(1int)
        } else if self.spec_has_interrupted_thread(tid) {
            Some(2int)
        } else if self.spec_has_sleeping_thread(tid) {
            Some(3int)
        } else if self.spec_has_zombie_thread(tid) {
            Some(4int)
        } else {
            None
        }
    }

    /// Spec function: models `try_join_thread()` return semantics.
    ///
    /// Returns an abstract result tag:
    /// - `Ok(0)` if the thread is a zombie (will be removed from zombie list).
    /// - `Err(Ok(1))` if the thread is the running thread (operation not permitted).
    /// - `Err(Ok(2))` if the thread is ready, sleeping, or interrupted (returns condvar).
    /// - `Err(Err(3))` if the thread is not found (no such process).
    ///
    /// Key properties:
    /// - Joining a running thread is an error (`ErrorCode::OperationNotPermitted`).
    /// - Joining a zombie thread removes it from the zombie list (side effect).
    /// - Joining a live thread (ready/sleeping/interrupted) returns a condvar for waiting.
    /// - Joining a non-existent thread is an error (`ErrorCode::NoSuchProcess`).
    pub open spec fn spec_try_join_thread(&self, tid: int) -> int {
        if self.running_thread_id@ == tid {
            // Running thread: OperationNotPermitted error.
            1int
        } else if self.spec_has_zombie_thread(tid) {
            // Zombie thread: Ok, will be removed.
            0int
        } else if self.spec_has_ready_thread(tid)
            || self.spec_has_sleeping_thread(tid)
            || self.spec_has_interrupted_thread(tid) {
            // Live thread: returns condvar.
            2int
        } else {
            // Not found: NoSuchProcess error.
            3int
        }
    }

    /// Spec function: models the post-state of `try_join_thread()` for the
    /// zombie removal side effect.
    ///
    /// When `spec_try_join_thread(tid) == 0` (zombie found), the zombie list
    /// shrinks by one element (the matched thread is removed). This models
    /// the `self.zombie.take()` + `remove_if()` mutation in the original
    /// (source lines 350–359).
    ///
    /// Returns the resulting zombie thread ID sequence after removal.
    /// Requires that `tid` is in the zombie list (i.e., `spec_try_join_thread(tid) == 0`).
    pub open spec fn spec_try_join_zombie_post(&self, tid: int) -> Seq<int>
        recommends self.spec_try_join_thread(tid) == 0int
    {
        // The zombie list with the first occurrence of `tid` removed.
        // Since spec_has_zombie_thread guarantees an index exists, we use
        // choose to pick one and remove it.
        let idx: int = choose|i: int| 0 <= i < self.zombie_thread_ids@.len()
            && self.zombie_thread_ids@[i] == tid;
        Self::spec_remove_at(self.zombie_thread_ids@, idx)
    }

    /// Spec helper: checks if a sequence contains a given value.
    pub open spec fn spec_seq_contains(s: Seq<int>, tid: int) -> bool {
        exists|i: int| 0 <= i < s.len() && s[i] == tid
    }

    /// Spec helper: computes the sequence resulting from removing index `idx`.
    pub open spec fn spec_remove_at(s: Seq<int>, idx: int) -> Seq<int>
        recommends 0 <= idx < s.len()
    {
        s.subrange(0, idx).add(s.subrange(idx + 1, s.len() as int))
    }

    /// Spec helper: checks whether two sequences share no common elements.
    pub open spec fn spec_seqs_disjoint(a: Seq<int>, b: Seq<int>) -> bool {
        forall|i: int, j: int|
            0 <= i < a.len() && 0 <= j < b.len()
            ==> a[i] != b[j]
    }

    /// Spec function: well-formedness predicate.
    ///
    /// A RunningProcess is well-formed when:
    /// - Exec-level counters match ghost sequence lengths.
    ///
    /// Note: Thread ID uniqueness is a trust assumption from Rust's ownership model.
    /// See `wf_strict()` for an optional stronger predicate.
    pub open spec fn wf(&self) -> bool {
        &&& self.ready_count as nat == self.ready_thread_ids@.len()
        &&& self.interrupted_count as nat == self.interrupted_thread_ids@.len()
        &&& self.sleeping_count as nat == self.sleeping_thread_ids@.len()
        &&& self.zombie_count as nat == self.zombie_thread_ids@.len()
    }

    /// Spec function: strict well-formedness with thread ID uniqueness.
    ///
    /// Extends `wf()` with pairwise disjointness of all thread lists plus
    /// the running thread. NOT part of the default `wf()` — this is a trust
    /// assumption inherited from Rust's ownership model. Provided for
    /// downstream cross-module proofs that need thread ID exclusivity.
    pub open spec fn wf_strict(&self) -> bool {
        &&& self.wf()
        // Running thread not in any list.
        &&& !self.spec_has_ready_thread(self.running_thread_id@)
        &&& !self.spec_has_interrupted_thread(self.running_thread_id@)
        &&& !self.spec_has_sleeping_thread(self.running_thread_id@)
        &&& !self.spec_has_zombie_thread(self.running_thread_id@)
        // Pairwise disjointness of lists.
        &&& Self::spec_seqs_disjoint(self.ready_thread_ids@, self.interrupted_thread_ids@)
        &&& Self::spec_seqs_disjoint(self.ready_thread_ids@, self.sleeping_thread_ids@)
        &&& Self::spec_seqs_disjoint(self.ready_thread_ids@, self.zombie_thread_ids@)
        &&& Self::spec_seqs_disjoint(self.interrupted_thread_ids@, self.sleeping_thread_ids@)
        &&& Self::spec_seqs_disjoint(self.interrupted_thread_ids@, self.zombie_thread_ids@)
        &&& Self::spec_seqs_disjoint(self.sleeping_thread_ids@, self.zombie_thread_ids@)
    }
}

//==================================================================================================
// Spec Functions: Boundary Types
//==================================================================================================

impl RunnableProcess {
    /// Spec function: returns the process identifier value.
    pub open spec fn spec_pid(&self) -> int {
        self.pid@
    }

    /// Spec function: well-formedness predicate.
    pub open spec fn wf(&self) -> bool {
        self.ready_thread_ids@.len() >= 1
    }
}

impl SleepingProcess {
    /// Spec function: returns the process identifier value.
    pub open spec fn spec_pid(&self) -> int {
        self.pid@
    }

    /// Spec function: well-formedness predicate.
    pub open spec fn wf(&self) -> bool {
        self.sleeping_thread_ids@.len() >= 1
    }
}

impl InterruptedProcess {
    /// Spec function: returns the process identifier value.
    pub open spec fn spec_pid(&self) -> int {
        self.pid@
    }

    /// Spec function: well-formedness predicate.
    pub open spec fn wf(&self) -> bool {
        self.interrupted_thread_ids@.len() >= 1
    }
}

impl ZombieProcess {
    /// Spec function: returns the process identifier value.
    pub open spec fn spec_pid(&self) -> int {
        self.pid@
    }

    /// Spec function: returns the exit status.
    pub open spec fn spec_status(&self) -> int {
        self.status@
    }

    /// Spec function: well-formedness predicate.
    pub open spec fn wf(&self) -> bool {
        self.zombie_thread_ids@.len() >= 1
    }
}

//==================================================================================================
// View Implementations
//==================================================================================================

impl View for RunningProcess {
    type V = RunningProcessView;

    open spec fn view(&self) -> RunningProcessView {
        RunningProcessView {
            pid: self.pid@,
            running_thread_id: self.running_thread_id@,
            ready_thread_ids: self.ready_thread_ids@,
            interrupted_thread_ids: self.interrupted_thread_ids@,
            sleeping_thread_ids: self.sleeping_thread_ids@,
            zombie_thread_ids: self.zombie_thread_ids@,
        }
    }
}

impl View for RunnableProcess {
    type V = RunnableProcessView;

    open spec fn view(&self) -> RunnableProcessView {
        RunnableProcessView {
            pid: self.pid@,
            ready_thread_ids: self.ready_thread_ids@,
            interrupted_thread_ids: self.interrupted_thread_ids@,
            sleeping_thread_ids: self.sleeping_thread_ids@,
            zombie_thread_ids: self.zombie_thread_ids@,
        }
    }
}

impl View for SleepingProcess {
    type V = SleepingProcessView;

    open spec fn view(&self) -> SleepingProcessView {
        SleepingProcessView {
            pid: self.pid@,
            sleeping_thread_ids: self.sleeping_thread_ids@,
            zombie_thread_ids: self.zombie_thread_ids@,
        }
    }
}

impl View for InterruptedProcess {
    type V = InterruptedProcessView;

    open spec fn view(&self) -> InterruptedProcessView {
        InterruptedProcessView {
            pid: self.pid@,
            interrupted_thread_ids: self.interrupted_thread_ids@,
            sleeping_thread_ids: self.sleeping_thread_ids@,
            zombie_thread_ids: self.zombie_thread_ids@,
        }
    }
}

impl View for ZombieProcess {
    type V = ZombieProcessView;

    open spec fn view(&self) -> ZombieProcessView {
        ZombieProcessView {
            pid: self.pid@,
            zombie_thread_ids: self.zombie_thread_ids@,
            status: self.status@,
        }
    }
}

} // verus!
