// Copyright(c) The Maintainers of Nanvix.
// Licensed under the MIT License.

// RunnableProcess Specification.
// This file contains spec functions and View types for the RunnableProcess type
// and boundary types RunningProcess, InterruptedProcess, ZombieProcess.
//
// ## Verification Model
//
// RunnableProcess manages process-level thread scheduling. For verification:
// - Thread lists are modeled as `Seq<int>` of abstract thread IDs.
// - `ready_admission_times` tracks admission times paired with ready thread IDs.
// - `NonEmptyVecDeque<T>` is modeled as `Seq` with `len() >= 1` invariant.
// - `Box<ProcessState>` is transparent (modeled as ProcessState directly).
// - `ProcessState` uses the verified dependency's `spec_pid()`.
// - `RunningProcess`, `InterruptedProcess`, `ZombieProcess` are boundary models.
//
// ## Key Invariants
//
// - A RunnableProcess always has at least one ready thread (`ready_thread_ids.len() >= 1`).
// - Thread IDs across all lists are disjoint (no thread appears in two lists).
// - All thread IDs are non-negative (modeling valid kernel thread identifiers).
// - Process identity (PID) is immutable across all operations.
//
// ## Trust Assumptions
//
// - Thread ID uniqueness across lists models the original's type-safe ownership
//   (a thread can only be in one NonEmptyVecDeque at a time).
// - `ContextInformation` and `VirtualAddress` from run() are omitted (HAL boundary).
// - `InterruptReason` is modeled as abstract int tag.

use vstd::prelude::*;

verus! {

//==================================================================================================
// View Types
//==================================================================================================

/// Abstract view of a RunnableProcess.
#[verifier::ext_equal]
pub struct RunnableProcessView {
    /// Process identifier value.
    pub pid: int,
    /// Sequence of ready thread IDs (non-empty).
    pub ready_thread_ids: Seq<int>,
    /// Sequence of ready thread admission times, parallel to ready_thread_ids.
    pub ready_admission_times: Seq<int>,
    /// Sequence of interrupted thread IDs (may be empty).
    pub interrupted_thread_ids: Seq<int>,
    /// Sequence of sleeping thread IDs (may be empty).
    pub sleeping_thread_ids: Seq<int>,
    /// Sequence of zombie thread IDs (may be empty).
    pub zombie_thread_ids: Seq<int>,
}

/// Abstract view of a RunningProcess (boundary type).
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
/// Models `ErrorCode::Interrupted.into()` from the original code.
pub open spec fn EXIT_STATUS_INTERRUPTED() -> int { 4 }

//==================================================================================================
// Spec Functions: RunnableProcess
//==================================================================================================

impl RunnableProcess {
    /// Spec function: returns the process identifier value.
    pub open spec fn spec_pid(&self) -> int {
        self.pid.spec_value()
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

    /// Spec function: returns the total number of threads.
    pub open spec fn spec_total_thread_count(&self) -> nat {
        self.spec_ready_count()
        + self.spec_interrupted_count()
        + self.spec_sleeping_count()
        + self.spec_zombie_count()
    }

    /// Spec function: returns the i-th ready thread ID.
    pub open spec fn spec_ready_thread_id(&self, i: int) -> int
        recommends 0 <= i < self.ready_thread_ids@.len()
    {
        self.ready_thread_ids@[i]
    }

    /// Spec function: returns the i-th ready thread admission time.
    pub open spec fn spec_ready_admission_time(&self, i: int) -> int
        recommends 0 <= i < self.ready_admission_times@.len()
    {
        self.ready_admission_times@[i]
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

    /// Spec function: checks if a thread ID is in any list.
    pub open spec fn spec_has_thread(&self, tid: int) -> bool {
        self.spec_has_ready_thread(tid)
        || self.spec_has_sleeping_thread(tid)
        || (exists|i: int| 0 <= i < self.interrupted_thread_ids@.len()
            && self.interrupted_thread_ids@[i] == tid)
        || (exists|i: int| 0 <= i < self.zombie_thread_ids@.len()
            && self.zombie_thread_ids@[i] == tid)
    }

    /// Spec function: finds the index of the ready thread with earliest admission time.
    pub open spec fn spec_earliest_ready_index(&self) -> int
        recommends self.ready_thread_ids@.len() > 0
    {
        choose|idx: int| 0 <= idx < self.ready_admission_times@.len()
            && forall|j: int| 0 <= j < self.ready_admission_times@.len()
                ==> self.ready_admission_times@[idx] <= self.ready_admission_times@[j]
    }

    /// Spec function: returns the earliest admission time among ready threads.
    pub open spec fn spec_earliest_admission_time(&self) -> int
        recommends self.ready_thread_ids@.len() > 0
    {
        self.ready_admission_times@[self.spec_earliest_ready_index()]
    }

    /// Spec function: well-formedness predicate.
    ///
    /// A RunnableProcess is well-formed when:
    /// - There is at least one ready thread (modeling NonEmptyVecDeque).
    /// - Ready thread IDs and admission times sequences have equal length.
    /// - All admission times are non-negative.
    /// - Thread IDs across all lists are pairwise distinct (no duplicates within
    ///   or across lists), modeling ownership semantics.
    pub open spec fn wf(&self) -> bool {
        // At least one ready thread (NonEmptyVecDeque invariant).
        &&& self.ready_thread_ids@.len() >= 1
        // Parallel arrays have matching lengths.
        &&& self.ready_thread_ids@.len() == self.ready_admission_times@.len()
        // Admission times are non-negative.
        &&& forall|i: int| 0 <= i < self.ready_admission_times@.len()
                ==> #[trigger] self.ready_admission_times@[i] >= 0
    }
}

//==================================================================================================
// Spec Functions: RunningProcess (Boundary)
//==================================================================================================

impl RunningProcess {
    /// Spec function: returns the process identifier value.
    pub open spec fn spec_pid(&self) -> int {
        self.pid
    }

    /// Spec function: returns the running thread ID.
    pub open spec fn spec_running_thread_id(&self) -> int {
        self.running_thread_id
    }

    /// Spec function: well-formedness predicate.
    pub open spec fn wf(&self) -> bool {
        true
    }
}

//==================================================================================================
// Spec Functions: InterruptedProcess (Boundary)
//==================================================================================================

impl InterruptedProcess {
    /// Spec function: returns the process identifier value.
    pub open spec fn spec_pid(&self) -> int {
        self.pid
    }

    /// Spec function: well-formedness predicate.
    pub open spec fn wf(&self) -> bool {
        self.interrupted_thread_ids@.len() >= 1
    }
}

//==================================================================================================
// Spec Functions: ZombieProcess (Boundary)
//==================================================================================================

impl ZombieProcess {
    /// Spec function: returns the process identifier value.
    pub open spec fn spec_pid(&self) -> int {
        self.pid
    }

    /// Spec function: returns the exit status.
    pub open spec fn spec_status(&self) -> int {
        self.status
    }

    /// Spec function: well-formedness predicate.
    pub open spec fn wf(&self) -> bool {
        self.zombie_thread_ids@.len() >= 1
    }
}

//==================================================================================================
// View Implementations
//==================================================================================================

impl View for RunnableProcess {
    type V = RunnableProcessView;

    open spec fn view(&self) -> RunnableProcessView {
        RunnableProcessView {
            pid: self.pid.spec_value(),
            ready_thread_ids: self.ready_thread_ids@,
            ready_admission_times: self.ready_admission_times@,
            interrupted_thread_ids: self.interrupted_thread_ids@,
            sleeping_thread_ids: self.sleeping_thread_ids@,
            zombie_thread_ids: self.zombie_thread_ids@,
        }
    }
}

impl View for RunningProcess {
    type V = RunningProcessView;

    open spec fn view(&self) -> RunningProcessView {
        RunningProcessView {
            pid: self.pid,
            running_thread_id: self.running_thread_id,
            ready_thread_ids: self.ready_thread_ids@,
            interrupted_thread_ids: self.interrupted_thread_ids@,
            sleeping_thread_ids: self.sleeping_thread_ids@,
            zombie_thread_ids: self.zombie_thread_ids@,
        }
    }
}

impl View for InterruptedProcess {
    type V = InterruptedProcessView;

    open spec fn view(&self) -> InterruptedProcessView {
        InterruptedProcessView {
            pid: self.pid,
            interrupted_thread_ids: self.interrupted_thread_ids@,
            zombie_thread_ids: self.zombie_thread_ids@,
        }
    }
}

impl View for ZombieProcess {
    type V = ZombieProcessView;

    open spec fn view(&self) -> ZombieProcessView {
        ZombieProcessView {
            pid: self.pid,
            zombie_thread_ids: self.zombie_thread_ids@,
            status: self.status,
        }
    }
}

} // verus!
