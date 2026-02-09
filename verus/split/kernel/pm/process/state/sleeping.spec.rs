// Copyright(c) The Maintainers of Nanvix.
// Licensed under the MIT License.

// SleepingProcess Specification.
// This file contains spec functions and View types for the SleepingProcess type.
//
// ## Verification Model
//
// SleepingProcess has at least one sleeping thread and optional zombie threads.
// For verification:
// - Thread lists are modeled as `Seq<int>` of abstract thread IDs.
// - `NonEmptyVecDeque<SleepingThread>` is modeled as `Seq<int>` with `len() >= 1`.
// - `Option<NonEmptyVecDeque<ZombieThread>>` is modeled as `Seq<int>`:
//   - `Seq::empty()` represents `None`.
//   - `Seq` with `len() >= 1` represents `Some(non_empty_deque)`.
// - `Box<ProcessState>` is transparent (modeled as PID only).
//
// ## Key Invariants
//
// - A SleepingProcess always has at least one sleeping thread.
// - Process identity (PID) is immutable across all operations.
// - `terminate()` converts all sleeping threads to interrupted.
// - `wakeup(tid)` removes one sleeping thread (by ID), makes it ready,
//   and transitions to RunnableProcess.
// - `wakeup_alarm()` partitions sleeping threads into expired (→interrupted)
//   and remaining (→sleeping). Returns InterruptedProcess if any expired,
//   SleepingProcess otherwise.
// - `add_thread()` adds a ready thread and transitions to RunnableProcess.
//
// ## Ownership Semantics
//
// Thread ID uniqueness within and across lists IS enforced in `wf()`. In the
// original code, the Rust type system ensures ownership semantics — a thread
// struct can only be in one `NonEmptyVecDeque` at a time. We model this
// explicitly via `spec_no_duplicates` and `spec_seqs_disjoint` predicates
// in `wf()`, enabling precise reasoning about wakeup/partition operations.
//
// ## Trust Assumptions
//
// - Thread state transitions (SleepingThread::interrupt(), wakeup()) are
//   modeled as ID-preserving operations.
// - `wakeup_alarm()` alarm checking is modeled via oracle parameters; the
//   per-thread alarm comparison logic is a trust boundary.
// - `RunnableProcess` and `InterruptedProcess` are boundary models from
//   sibling modules.
// - `find_thread()` and `find_thread_mut()` are modeled spec-only because
//   they return reference types that Verus cannot express.
// - `state()` / `state_mut()` are elided (ProcessState access modeled via PID).

use vstd::prelude::*;

verus! {

//==================================================================================================
// View Types
//==================================================================================================

/// Abstract view of a SleepingProcess (this module's primary type).
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
    /// Sleeping thread IDs (may be empty).
    pub sleeping_thread_ids: Seq<int>,
    /// Zombie thread IDs (may be empty).
    pub zombie_thread_ids: Seq<int>,
}

//==================================================================================================
// Spec Functions: SleepingProcess
//==================================================================================================

impl SleepingProcess {
    /// Spec function: returns the process identifier value.
    pub open spec fn spec_pid(&self) -> int {
        self.pid@
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
        self.spec_sleeping_count() + self.spec_zombie_count()
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
        self.spec_has_sleeping_thread(tid)
        || self.spec_has_zombie_thread(tid)
    }

    /// Spec function: models `find_thread()` — returns which list a thread is in.
    ///
    /// - `Some(0)` if found in sleeping threads.
    /// - `Some(1)` if found in zombie threads.
    /// - `None` if not found.
    ///
    /// Search order matches original: sleeping → zombie.
    pub open spec fn spec_find_thread(&self, tid: int) -> Option<int> {
        if self.spec_has_sleeping_thread(tid) {
            Some(0int)
        } else if self.spec_has_zombie_thread(tid) {
            Some(1int)
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

    /// Spec helper: checks whether `sub` is a subsequence of `full`.
    ///
    /// A sequence `sub` is a subsequence of `full` if every element of `sub`
    /// appears in `full` in the same relative order. This models the stable
    /// partition semantics of the original `wakeup_alarm()` implementation,
    /// which processes threads front-to-back and preserves their order.
    pub open spec fn spec_is_subsequence(sub: Seq<int>, full: Seq<int>) -> bool {
        exists|indices: Seq<int>|
            indices.len() == sub.len()
            && (forall|k: int| #![auto] 0 <= k < indices.len() ==>
                0 <= indices[k] < full.len() as int
                && full[indices[k]] == sub[k])
            && (forall|k: int, l: int| #![auto] 0 <= k < l < indices.len() ==>
                indices[k] < indices[l])
    }

    /// Spec function: well-formedness predicate.
    ///
    /// A SleepingProcess is well-formed when:
    /// - The sleeping thread count matches the ghost sequence length.
    /// - There is at least one sleeping thread (NonEmptyVecDeque invariant).
    /// - No duplicate thread IDs within either list.
    /// - Sleeping and zombie thread IDs are disjoint.
    pub open spec fn wf(&self) -> bool {
        &&& self.sleeping_count as nat == self.sleeping_thread_ids@.len()
        &&& self.sleeping_thread_ids@.len() >= 1
        &&& Self::spec_no_duplicates(self.sleeping_thread_ids@)
        &&& Self::spec_no_duplicates(self.zombie_thread_ids@)
        &&& Self::spec_seqs_disjoint(self.sleeping_thread_ids@, self.zombie_thread_ids@)
    }

    /// Spec function: frame condition for mutable accessor (`state_mut()`).
    pub open spec fn mutation_frame_preserved(old_self: &Self, new_self: &Self) -> bool {
        &&& new_self.spec_pid() == old_self.spec_pid()
        &&& new_self.sleeping_thread_ids@ == old_self.sleeping_thread_ids@
        &&& new_self.zombie_thread_ids@ == old_self.zombie_thread_ids@
        &&& new_self.sleeping_count == old_self.sleeping_count
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

//==================================================================================================
// View Implementations
//==================================================================================================

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

} // verus!
