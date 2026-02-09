// Copyright(c) The Maintainers of Nanvix.
// Licensed under the MIT License.

// InterruptedProcess Specification.
// This file contains spec functions and View types for the InterruptedProcess type.
//
// ## Verification Model
//
// InterruptedProcess has at least one interrupted thread, optional sleeping
// threads, and optional zombie threads. For verification:
// - Thread lists are modeled as `Seq<int>` of abstract thread IDs.
// - `NonEmptyVecDeque<InterruptedThread>` is modeled as `Seq<int>` with `len() >= 1`.
// - `Option<NonEmptyVecDeque<SleepingThread>>` is modeled as `Seq<int>`:
//   - `Seq::empty()` represents `None`.
//   - `Seq` with `len() >= 1` represents `Some(non_empty_deque)`.
// - `Option<NonEmptyVecDeque<ZombieThread>>` is similarly modeled.
// - `Box<ProcessState>` is transparent (modeled as PID only).
//
// ## Key Invariants
//
// - An InterruptedProcess always has at least one interrupted thread.
// - Process identity (PID) is immutable across all operations.
// - `resume()` pops the front interrupted thread, resumes it to ready,
//   and transitions to RunnableProcess. PID and all other thread lists
//   are preserved.
// - Thread IDs are unique within and disjoint across lists.
//
// ## Ownership Semantics
//
// Thread ID uniqueness within and across lists IS enforced in `wf()`.
// In the original code, Rust's ownership model ensures a thread struct
// can only appear in one `NonEmptyVecDeque` at a time. We model this
// explicitly via `spec_no_duplicates` and pairwise `spec_seqs_disjoint`.
//
// ## Trust Assumptions
//
// - Thread state transitions (InterruptedThread::resume()) are modeled
//   as ID-preserving operations.
// - `RunnableProcess` is a boundary model from the sibling module.
// - `find_thread()` / `find_thread_mut()` return reference types that
//   Verus cannot express; modeled spec-only.
// - `state()` / `state_mut()` return references to ProcessState; modeled
//   as external_body with frame conditions.
// - The standalone `interrupt()` function is modeled as ID-preserving.

use vstd::prelude::*;

verus! {

//==================================================================================================
// View Types
//==================================================================================================

/// Abstract view of an InterruptedProcess (this module's primary type).
#[verifier::ext_equal]
pub struct InterruptedProcessView {
    /// Process identifier value.
    pub pid: int,
    /// Sleeping thread IDs (may be empty).
    pub sleeping_thread_ids: Seq<int>,
    /// Interrupted thread IDs (non-empty).
    pub interrupted_thread_ids: Seq<int>,
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

//==================================================================================================
// Spec Functions: InterruptedProcess
//==================================================================================================

impl InterruptedProcess {
    /// Spec function: returns the process identifier value.
    pub open spec fn spec_pid(&self) -> int {
        self.pid@
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
        self.spec_interrupted_count()
        + self.spec_sleeping_count()
        + self.spec_zombie_count()
    }

    /// Spec function: checks if a thread ID is in the interrupted list.
    pub open spec fn spec_has_interrupted_thread(&self, tid: int) -> bool {
        exists|i: int| 0 <= i < self.interrupted_thread_ids@.len()
            && self.interrupted_thread_ids@[i] == tid
    }

    /// Spec function: checks if a thread ID is in the sleeping list.
    pub open spec fn spec_has_sleeping_thread(&self, tid: int) -> bool {
        exists|i: int| 0 <= i < self.sleeping_thread_ids@.len()
            && self.sleeping_thread_ids@[i] == tid
    }

    /// Spec function: checks if a thread ID is in the zombie list.
    pub open spec fn spec_has_zombie_thread(&self, tid: int) -> bool {
        exists|i: int| 0 <= i < self.zombie_thread_ids@.len()
            && self.zombie_thread_ids@[i] == tid
    }

    /// Spec function: checks if a thread ID is in any list.
    pub open spec fn spec_has_thread(&self, tid: int) -> bool {
        self.spec_has_interrupted_thread(tid)
        || self.spec_has_sleeping_thread(tid)
        || self.spec_has_zombie_thread(tid)
    }

    /// Spec function: models `find_thread()` — returns which list a thread is in.
    ///
    /// - `Some(0)` if found in interrupted threads.
    /// - `Some(1)` if found in sleeping threads.
    /// - `Some(2)` if found in zombie threads.
    /// - `None` if not found.
    ///
    /// Search order matches original: interrupted → sleeping → zombie.
    pub open spec fn spec_find_thread(&self, tid: int) -> Option<int> {
        if self.spec_has_interrupted_thread(tid) {
            Some(0int)
        } else if self.spec_has_sleeping_thread(tid) {
            Some(1int)
        } else if self.spec_has_zombie_thread(tid) {
            Some(2int)
        } else {
            None
        }
    }

    /// Spec helper: checks if a sequence contains a given value.
    pub open spec fn spec_seq_contains(s: Seq<int>, tid: int) -> bool {
        exists|i: int| 0 <= i < s.len() && s[i] == tid
    }

    /// Spec helper: checks whether a sequence has no duplicate elements.
    pub open spec fn spec_no_duplicates(s: Seq<int>) -> bool {
        forall|i: int, j: int| 0 <= i < j < s.len()
            ==> s[i] != s[j]
    }

    /// Spec helper: checks whether two sequences share no common elements.
    pub open spec fn spec_seqs_disjoint(a: Seq<int>, b: Seq<int>) -> bool {
        forall|i: int, j: int|
            0 <= i < a.len() && 0 <= j < b.len()
            ==> a[i] != b[j]
    }

    /// Spec function: well-formedness predicate.
    ///
    /// An InterruptedProcess is well-formed when:
    /// - There is at least one interrupted thread (NonEmptyVecDeque invariant).
    /// - No duplicate thread IDs within any list.
    /// - All thread lists are pairwise disjoint.
    pub open spec fn wf(&self) -> bool {
        &&& self.interrupted_thread_ids@.len() >= 1
        &&& Self::spec_no_duplicates(self.interrupted_thread_ids@)
        &&& Self::spec_no_duplicates(self.sleeping_thread_ids@)
        &&& Self::spec_no_duplicates(self.zombie_thread_ids@)
        &&& Self::spec_seqs_disjoint(self.interrupted_thread_ids@, self.sleeping_thread_ids@)
        &&& Self::spec_seqs_disjoint(self.interrupted_thread_ids@, self.zombie_thread_ids@)
        &&& Self::spec_seqs_disjoint(self.sleeping_thread_ids@, self.zombie_thread_ids@)
    }

    /// Spec function: frame condition for mutable accessor (`state_mut()`).
    pub open spec fn mutation_frame_preserved(old_self: &Self, new_self: &Self) -> bool {
        &&& new_self.spec_pid() == old_self.spec_pid()
        &&& new_self.interrupted_thread_ids@ == old_self.interrupted_thread_ids@
        &&& new_self.sleeping_thread_ids@ == old_self.sleeping_thread_ids@
        &&& new_self.zombie_thread_ids@ == old_self.zombie_thread_ids@
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

//==================================================================================================
// View Implementations
//==================================================================================================

impl View for InterruptedProcess {
    type V = InterruptedProcessView;

    open spec fn view(&self) -> InterruptedProcessView {
        InterruptedProcessView {
            pid: self.pid@,
            sleeping_thread_ids: self.sleeping_thread_ids@,
            interrupted_thread_ids: self.interrupted_thread_ids@,
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

} // verus!
