// Copyright(c) The Maintainers of Nanvix.
// Licensed under the MIT License.

// SleepingProcess Specification.
// This file contains spec functions and View types for the SleepingProcess type.
//
// ## Abstract State Transition Functions (View Level)
//
// Added `impl SleepingProcessView` with abstract transition functions so that
// downstream modules can write postconditions like:
//     ensures result@ =~= old(self)@.spec_terminate()
// instead of listing every field change individually.
//
// Transition functions:
// - `spec_new(pid, sleeping, zombie)` — constructor.
// - `spec_terminate(self)` — all sleeping → interrupted.
// - `spec_wakeup(self, tid)` — remove sleeping thread, produce RunnableProcessView.
// - `spec_wakeup_alarm_expired(self, interrupted, remaining)` — partition sleeping.
// - `spec_wakeup_alarm_none(self)` — identity (no alarm expired).
// - `spec_add_thread(self, ready_tid)` — add ready thread, produce RunnableProcessView.
//
// Bridging lemmas in sleeping.proof.rs prove that exec postconditions imply
// the View-level transition equalities.
//
// ## Verification Model
//
// SleepingProcess has at least one sleeping thread and optional zombie threads.
// For verification:
// - Thread lists are modeled as concrete `Vec<u64>` of thread IDs.
// - `NonEmptyVecDeque<SleepingThread>` is modeled as `Vec<u64>` with `len() >= 1`.
// - `Option<NonEmptyVecDeque<ZombieThread>>` is modeled as `Vec<u64>`:
//   - Empty Vec represents `None`.
//   - Vec with `len() >= 1` represents `Some(non_empty_deque)`.
// - `Box<ProcessState>` is transparent (modeled as PID only, concrete `u64`).
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
//   they return reference types that Verus cannot express. Importantly,
//   `find_thread_mut()` yields `&mut` access; any mutation through it is
//   **outside** the scope of this verification model. Callers must ensure
//   that mutations preserve wf() as a caller-side proof obligation.
// - `state()` / `state_mut()` are elided (ProcessState access modeled via PID).

use vstd::prelude::*;

verus! {

//==================================================================================================
// View Types
//==================================================================================================
// These View types provide the abstract state representation for cross-module
// composition. They are exercised by lemma_view_equality and serve as the
// canonical interface for downstream modules that consume process state transitions.

/// Abstract view of a SleepingProcess (this module's primary type).
#[verifier::ext_equal]
pub struct SleepingProcessView {
    /// Process identifier value.
    pub pid: u64,
    /// Sleeping thread IDs (non-empty).
    pub sleeping_thread_ids: Seq<u64>,
    /// Zombie thread IDs (may be empty).
    pub zombie_thread_ids: Seq<u64>,
}

/// Abstract view of a RunnableProcess (boundary type).
#[verifier::ext_equal]
pub struct RunnableProcessView {
    /// Process identifier value.
    pub pid: u64,
    /// Ready thread IDs (non-empty).
    pub ready_thread_ids: Seq<u64>,
    /// Interrupted thread IDs (may be empty).
    pub interrupted_thread_ids: Seq<u64>,
    /// Sleeping thread IDs (may be empty).
    pub sleeping_thread_ids: Seq<u64>,
    /// Zombie thread IDs (may be empty).
    pub zombie_thread_ids: Seq<u64>,
}

/// Abstract view of an InterruptedProcess (boundary type).
#[verifier::ext_equal]
pub struct InterruptedProcessView {
    /// Process identifier value.
    pub pid: u64,
    /// Interrupted thread IDs (non-empty).
    pub interrupted_thread_ids: Seq<u64>,
    /// Sleeping thread IDs (may be empty).
    pub sleeping_thread_ids: Seq<u64>,
    /// Zombie thread IDs (may be empty).
    pub zombie_thread_ids: Seq<u64>,
}

//==================================================================================================
// Spec Functions: SleepingProcess
//==================================================================================================

impl SleepingProcess {
    /// Spec function: returns the process identifier value.
    pub open spec fn spec_pid(&self) -> u64 {
        self.pid
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
    pub open spec fn spec_has_sleeping_thread(&self, tid: u64) -> bool {
        exists|i: int| 0 <= i < self.sleeping_thread_ids@.len()
            && self.sleeping_thread_ids@[i] == tid
    }

    /// Spec function: checks if a thread ID is in the zombie list.
    pub open spec fn spec_has_zombie_thread(&self, tid: u64) -> bool {
        exists|i: int| 0 <= i < self.zombie_thread_ids@.len()
            && self.zombie_thread_ids@[i] == tid
    }

    /// Spec function: checks if a thread ID is in any list.
    pub open spec fn spec_has_thread(&self, tid: u64) -> bool {
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
    pub open spec fn spec_find_thread(&self, tid: u64) -> Option<int> {
        if self.spec_has_sleeping_thread(tid) {
            Some(0int)
        } else if self.spec_has_zombie_thread(tid) {
            Some(1int)
        } else {
            None
        }
    }

    /// Spec helper: checks if a sequence contains a given value.
    pub open spec fn spec_seq_contains(s: Seq<u64>, tid: u64) -> bool {
        exists|i: int| 0 <= i < s.len() && s[i] == tid
    }

    /// Spec helper: computes the sequence resulting from removing index `idx`.
    pub open spec fn spec_remove_at(s: Seq<u64>, idx: int) -> Seq<u64>
        recommends 0 <= idx < s.len()
    {
        s.subrange(0, idx).add(s.subrange(idx + 1, s.len() as int))
    }

    /// Spec helper: checks whether a sequence has no duplicate elements.
    pub open spec fn spec_no_duplicates(s: Seq<u64>) -> bool {
        forall|i: int, j: int| 0 <= i < j < s.len()
            ==> s[i] != s[j]
    }

    /// Spec helper: checks whether two sequences share no common elements.
    pub open spec fn spec_seqs_disjoint(a: Seq<u64>, b: Seq<u64>) -> bool {
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
    pub open spec fn spec_is_subsequence(sub: Seq<u64>, full: Seq<u64>) -> bool {
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
    /// - There is at least one sleeping thread (NonEmptyVecDeque invariant).
    /// - No duplicate thread IDs within either list.
    /// - Sleeping and zombie thread IDs are disjoint.
    pub open spec fn wf(&self) -> bool {
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
    }
}

//==================================================================================================
// Spec Functions: Boundary Types
//==================================================================================================

impl RunnableProcess {
    /// Spec function: returns the process identifier value.
    pub open spec fn spec_pid(&self) -> u64 {
        self.pid
    }

    /// Spec function: well-formedness predicate.
    pub open spec fn wf(&self) -> bool {
        self.ready_thread_ids@.len() >= 1
    }
}

impl InterruptedProcess {
    /// Spec function: returns the process identifier value.
    pub open spec fn spec_pid(&self) -> u64 {
        self.pid
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
            pid: self.pid,
            sleeping_thread_ids: self.sleeping_thread_ids@,
            zombie_thread_ids: self.zombie_thread_ids@,
        }
    }
}

impl View for RunnableProcess {
    type V = RunnableProcessView;

    open spec fn view(&self) -> RunnableProcessView {
        RunnableProcessView {
            pid: self.pid,
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
            sleeping_thread_ids: self.sleeping_thread_ids@,
            zombie_thread_ids: self.zombie_thread_ids@,
        }
    }
}

//==================================================================================================
// Abstract State Transition Functions (View Level)
//==================================================================================================
// These functions model exec-level operations at the View level, enabling
// downstream modules to write postconditions like:
//     ensures result@ =~= old(self)@.spec_terminate()
// instead of listing every field change individually.

impl SleepingProcessView {
    /// View-level well-formedness predicate, equivalent to SleepingProcess::wf().
    pub open spec fn wf(&self) -> bool {
        &&& self.sleeping_thread_ids.len() >= 1
        &&& SleepingProcess::spec_no_duplicates(self.sleeping_thread_ids)
        &&& SleepingProcess::spec_no_duplicates(self.zombie_thread_ids)
        &&& SleepingProcess::spec_seqs_disjoint(self.sleeping_thread_ids, self.zombie_thread_ids)
    }

    /// View-level helper: removes element at index from a sequence.
    pub open spec fn spec_remove_at_seq(s: Seq<u64>, idx: int) -> Seq<u64> {
        s.subrange(0, idx).add(s.subrange(idx + 1, s.len() as int))
    }

    /// Abstract constructor: models `SleepingProcess::new()`.
    pub open spec fn spec_new(
        pid: u64,
        sleeping_thread_ids: Seq<u64>,
        zombie_thread_ids: Seq<u64>,
    ) -> SleepingProcessView {
        SleepingProcessView { pid, sleeping_thread_ids, zombie_thread_ids }
    }

    /// Abstract transition: models `SleepingProcess::terminate()`.
    ///
    /// All sleeping threads become interrupted; no sleeping threads remain.
    /// Zombie threads are preserved.
    pub open spec fn spec_terminate(self) -> InterruptedProcessView {
        InterruptedProcessView {
            pid: self.pid,
            interrupted_thread_ids: self.sleeping_thread_ids,
            sleeping_thread_ids: Seq::<u64>::empty(),
            zombie_thread_ids: self.zombie_thread_ids,
        }
    }

    /// Abstract transition: models `SleepingProcess::wakeup()` (success case).
    ///
    /// Removes the thread with ID `tid` from the sleeping list and makes it
    /// the sole ready thread in the resulting RunnableProcess. Under `wf()`
    /// no-duplicates, the index is uniquely determined by `choose`.
    pub open spec fn spec_wakeup(self, tid: u64) -> RunnableProcessView {
        let idx = choose|i: int| 0 <= i < self.sleeping_thread_ids.len()
            && self.sleeping_thread_ids[i] == tid;
        RunnableProcessView {
            pid: self.pid,
            ready_thread_ids: Seq::<u64>::empty().push(tid),
            interrupted_thread_ids: Seq::<u64>::empty(),
            sleeping_thread_ids: Self::spec_remove_at_seq(self.sleeping_thread_ids, idx),
            zombie_thread_ids: self.zombie_thread_ids,
        }
    }

    /// Abstract transition: models `SleepingProcess::wakeup_alarm()` (expired case).
    ///
    /// Partitions sleeping threads into expired (→interrupted) and remaining
    /// (→sleeping). Zombie threads are preserved.
    pub open spec fn spec_wakeup_alarm_expired(
        self,
        interrupted_ids: Seq<u64>,
        remaining_ids: Seq<u64>,
    ) -> InterruptedProcessView {
        InterruptedProcessView {
            pid: self.pid,
            interrupted_thread_ids: interrupted_ids,
            sleeping_thread_ids: remaining_ids,
            zombie_thread_ids: self.zombie_thread_ids,
        }
    }

    /// Abstract transition: models `SleepingProcess::wakeup_alarm()` (no-expiry case).
    ///
    /// No alarm expired; process remains sleeping with all state unchanged.
    pub open spec fn spec_wakeup_alarm_none(self) -> SleepingProcessView {
        self
    }

    /// Abstract transition: models `SleepingProcess::add_thread()`.
    ///
    /// Adds a ready thread and transitions to RunnableProcess. Sleeping and
    /// zombie threads are preserved.
    pub open spec fn spec_add_thread(self, ready_tid: u64) -> RunnableProcessView {
        RunnableProcessView {
            pid: self.pid,
            ready_thread_ids: Seq::<u64>::empty().push(ready_tid),
            interrupted_thread_ids: Seq::<u64>::empty(),
            sleeping_thread_ids: self.sleeping_thread_ids,
            zombie_thread_ids: self.zombie_thread_ids,
        }
    }
}

} // verus!
