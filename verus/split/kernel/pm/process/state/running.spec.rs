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
// - `SystemTime` (alarm) is modeled as `Option<int>`.
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
// only be in one `NonEmptyVecDeque` at a time.
//
// ## Trust Assumptions
//
// - `ContextInformation` pointers from schedule/sleep/exit are omitted (HAL boundary).
// - `Condvar` from join_cond/exit_thread is omitted (sync boundary).
// - Thread state transitions (RunningThread::schedule(), sleep(), exit()) are
//   modeled as ID-preserving operations.
// - `SleepingProcess`, `RunnableProcess`, `InterruptedProcess`, `ZombieProcess`
//   are boundary models from sibling modules.

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
#[verifier::ext_equal]
pub struct InterruptedProcessView {
    /// Process identifier value.
    pub pid: int,
    /// Interrupted thread IDs (non-empty).
    pub interrupted_thread_ids: Seq<int>,
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
// Spec Constants
//==================================================================================================

/// Abstract exit status for interrupted processes.
/// Value 4 corresponds to EINTR.
pub open spec fn EXIT_STATUS_INTERRUPTED() -> int { 4 }

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

    /// Spec function: well-formedness predicate.
    ///
    /// A RunningProcess is well-formed when:
    /// - Exec-level counters match ghost sequence lengths.
    ///
    /// Note: Thread ID uniqueness is a trust assumption from Rust's ownership model.
    pub open spec fn wf(&self) -> bool {
        &&& self.ready_count as nat == self.ready_thread_ids@.len()
        &&& self.interrupted_count as nat == self.interrupted_thread_ids@.len()
        &&& self.sleeping_count as nat == self.sleeping_thread_ids@.len()
        &&& self.zombie_count as nat == self.zombie_thread_ids@.len()
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

    /// Spec function: models `resume()` — transitions to RunnableProcess.
    /// In the original, `InterruptedProcess::resume()` picks an interrupted
    /// thread, makes it ready, and returns a RunnableProcess.
    /// We model the result's PID preservation and ready list non-emptiness.
    pub open spec fn spec_resume_pid(&self) -> int {
        self.pid@
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
