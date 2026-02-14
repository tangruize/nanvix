// Copyright(c) The Maintainers of Nanvix.
// Licensed under the MIT License.

// InterruptedProcess Specification.
// This file contains spec functions and View types for the InterruptedProcess type.
//
// ## View-Level Abstract State Transitions (added)
//
// `InterruptedProcessView` now exposes abstract state transition spec functions
// so downstream modules can write postconditions like:
//     ensures result@ =~= old(self)@.spec_resume(admission_time)
// instead of listing every field change individually.
//
// Added spec functions on `InterruptedProcessView`:
// - `wf()`: view-level well-formedness predicate.
// - `spec_new()`: models `InterruptedProcess::new()`.
// - `spec_from_sleeping()`: models `InterruptedProcess::from_sleeping()`.
// - `spec_resume()`: models `InterruptedProcess::resume()`.
// - `spec_state_mut()`: identity transition for `state_mut()`.
// - `spec_find_thread_mut()`: identity transition for `find_thread_mut()`.
//
// Bridging lemmas in `interrupted.proof.rs` connect exec postconditions to
// these view-level transitions (e.g., `lemma_resume_refines_spec`).
//
// ## Verification Model
//
// InterruptedProcess has at least one interrupted thread, optional sleeping
// threads, and optional zombie threads. For verification:
// - Thread lists are modeled as concrete `Vec<u64>` of thread IDs.
// - `NonEmptyVecDeque<InterruptedThread>` is modeled as `Vec<u64>` with `len() >= 1`.
// - `Option<NonEmptyVecDeque<SleepingThread>>` is modeled as `Vec<u64>`:
//   - Empty Vec represents `None`.
//   - Vec with `len() >= 1` represents `Some(non_empty_deque)`.
// - `Option<NonEmptyVecDeque<ZombieThread>>` is similarly modeled.
// - `Box<ProcessState>` is transparent (modeled as PID only, concrete `u64`).
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
//   as ID-preserving operations. **Per-thread state mutation trust gap:**
//   The original `InterruptedThread::resume()` calls
//   `self.state.set_interrupt_reason(self.reason)`, storing the interrupt
//   reason into the thread's `ThreadState` before conversion to `ReadyThread`.
//   This is NOT modeled because threads are abstracted to integer IDs.
//   Verification of `interrupt_reason` propagation is deferred to the
//   thread module (`src/kernel/src/pm/thread/interrupted.rs`).
// - `RunnableProcess` is a boundary model from the sibling module.
//   Note: The `InterruptedProcess` boundary model in `runnable.spec.rs`
//   omits `sleeping_thread_ids`. A projection lemma
//   (`lemma_project_to_runnable_boundary`) in the proof file extracts
//   `(pid, interrupted_ids, zombie_ids)` matching the runnable module's
//   boundary shape with invariants (non-empty, no-duplicates, disjointness)
//   proven. Integration proofs must construct the runnable module's boundary
//   type from this tuple.
// - `find_thread()` / `find_thread_mut()` are spec-level models that compute
//   `spec_find_thread()` directly. They do NOT model the executable search.
//   The original performs linear searches through `iter().find(...)` across
//   three collections with priority order (interrupted → sleeping → zombie).
//   The spec captures this search order but the executable iterator-based
//   logic is NOT verified. `lemma_find_thread_refinement_assumption`
//   documents the semantic equivalence assumption. Trust scope: the search
//   predicate (`thread.id() == tid`) and collection ordering must match
//   `spec_find_thread`. Tagged for trust-boundary inventory. If Verus adds
//   reference-typed return support or executable ghost iteration, replace
//   with a verified implementation.
// - `state()` / `state_mut()` return references to ProcessState in the original.
//   In the verification model, ProcessState is abstracted to PID and all fields
//   are concrete, so these are implemented as external_body returns.
//   The original `state_mut()` allows mutation of inner ProcessState fields
//   (e.g., capabilities); since our model only tracks PID, the frame condition
//   holds trivially.
// - The standalone `interrupt()` function is modeled as ID-preserving with
//   an explicit `InterruptReason::Killed` tag (spec constant
//   `INTERRUPT_REASON_KILLED`).
// - `resume()` takes `admission_time` as an oracle parameter. In the original,
//   this is `clock::now()` inside `ReadyThread::from_state()`. The clock is a
//   HAL boundary; callers must satisfy `spec_admission_time_valid()` to
//   establish equivalence with `clock::now()`. The link is advisory at this
//   module level and must be enforced at the integration proof level.

use vstd::prelude::*;

verus! {

//==================================================================================================
// View Types
//==================================================================================================

/// Abstract view of an InterruptedProcess (this module's primary type).
#[verifier::ext_equal]
pub struct InterruptedProcessView {
    /// Process identifier value.
    pub pid: u64,
    /// Sleeping thread IDs (may be empty).
    pub sleeping_thread_ids: Seq<u64>,
    /// Interrupted thread IDs (non-empty).
    pub interrupted_thread_ids: Seq<u64>,
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
    /// Ready thread admission times, parallel to ready_thread_ids.
    pub ready_admission_times: Seq<u64>,
    /// Interrupted thread IDs (may be empty).
    pub interrupted_thread_ids: Seq<u64>,
    /// Sleeping thread IDs (may be empty).
    pub sleeping_thread_ids: Seq<u64>,
    /// Zombie thread IDs (may be empty).
    pub zombie_thread_ids: Seq<u64>,
}

//==================================================================================================
// Spec Functions: InterruptedProcess
//==================================================================================================

impl InterruptedProcess {
    /// Spec function: returns the process identifier value.
    pub open spec fn spec_pid(&self) -> u64 {
        self.pid
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
    pub open spec fn spec_has_interrupted_thread(&self, tid: u64) -> bool {
        exists|i: int| 0 <= i < self.interrupted_thread_ids@.len()
            && self.interrupted_thread_ids@[i] == tid
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
    pub open spec fn spec_find_thread(&self, tid: u64) -> Option<int> {
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
    pub open spec fn spec_seq_contains(s: Seq<u64>, tid: u64) -> bool {
        exists|i: int| 0 <= i < s.len() && s[i] == tid
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

    /// Spec constant: interrupt reason for `Killed`.
    ///
    /// Models `InterruptReason::Killed` from the original source. The standalone
    /// `interrupt()` function always uses this reason. Value 0 is an abstract tag;
    /// the actual enum discriminant in the kernel is not relied upon.
    pub open spec fn INTERRUPT_REASON_KILLED() -> int { 0 }

    /// Spec function: models a clock reading for admission time.
    ///
    /// In the original code, `resume()` calls `clock::now()` to obtain the
    /// admission time for the newly ready thread. This spec function serves as
    /// a cross-module contract point: callers of `resume()` must provide an
    /// `admission_time` oracle satisfying:
    ///   `admission_time == Self::spec_clock_now(clock_state)`
    /// where `clock_state` is the abstract clock state at the call site.
    ///
    /// The clock model is a HAL boundary — this module does not define it.
    /// This spec is provided so that callers can express the obligation.
    /// The `clock_state` parameter is an opaque abstract value representing
    /// the current time context.
    pub open spec fn spec_clock_now(clock_state: int) -> int {
        clock_state
    }

    /// Spec function: validates that an admission time oracle was obtained
    /// from a valid clock reading.
    ///
    /// Callers of `resume()` should satisfy this predicate to establish
    /// semantic equivalence with the original `clock::now()` call.
    /// This is an **integration obligation**: `resume()` intentionally does
    /// NOT require this in its precondition because the clock is a HAL
    /// boundary outside this module's scope. Integration proofs must
    /// discharge this obligation at the call site.
    pub open spec fn spec_admission_time_valid(admission_time: int, clock_state: int) -> bool {
        admission_time == Self::spec_clock_now(clock_state)
        && admission_time >= 0
    }

    //==============================================================================================
    // Integration Obligations
    //==============================================================================================
    // The following spec functions define formal contracts that integration
    // proofs must discharge. They are NOT required by any precondition in
    // this module — they exist as machine-readable trust boundary markers
    // so that cross-module verification can reference and satisfy them.

    /// Integration obligation for `find_thread()` / `find_thread_mut()`.
    ///
    /// The spec model (`spec_find_thread`) assumes that the real executable
    /// `iter().find(|t| t.id() == tid)` search across three collections
    /// (interrupted → sleeping → zombie) produces identical results. An
    /// integration proof must verify:
    /// 1. The search predicate `t.id() == tid` matches `spec_has_*_thread`.
    /// 2. The collection ordering (interrupted first, then sleeping, then
    ///    zombie) matches `spec_find_thread`.
    /// 3. The first-match semantics of `Iterator::find` match the
    ///    existential quantifier in `spec_has_*_thread` under the
    ///    no-duplicates and disjointness invariants from `wf()`.
    ///
    /// This obligation cannot be discharged within this module because
    /// Verus cannot express the reference-typed return value (`ThreadRef`)
    /// or iterate ghost sequences.
    pub open spec fn spec_find_thread_integration_obligation(
        &self, tid: u64, real_result: Option<int>,
    ) -> bool {
        real_result == self.spec_find_thread(tid)
    }

    /// Integration obligation for interrupt reason propagation in `resume()`.
    ///
    /// The original `InterruptedThread::resume()` calls
    /// `self.state.set_interrupt_reason(self.reason)` before converting to
    /// `ReadyThread`. This module abstracts threads to integer IDs and does
    /// NOT model per-thread state. An integration proof must verify that the
    /// thread module's `resume()` establishes:
    ///   `ready_thread.state().interrupt_reason() == interrupted_thread.reason()`
    /// The `reason_tag` parameter represents the `InterruptReason` value
    /// (modeled as `INTERRUPT_REASON_KILLED` for the `Killed` variant).
    pub open spec fn spec_resume_reason_integration_obligation(
        thread_id: u64, reason_tag: int, ready_thread_reason: int,
    ) -> bool {
        ready_thread_reason == reason_tag
    }

    /// Integration obligation for ProcessState PID linking.
    ///
    /// The `pid` field in `InterruptedProcess` is assumed to match the
    /// real `ProcessState::pid()` inside `Box<ProcessState>`. An integration
    /// proof (or the ProcessState module's verification) must establish:
    ///   `pid == real_process_state.pid()`
    /// at construction time and show that no operation in this module
    /// invalidates this link. Since this module never mutates the PID
    /// (proven by PID-preservation postconditions on all functions), the
    /// obligation reduces to verifying the link at `new()` / `from_sleeping()`.
    pub open spec fn spec_process_state_pid_integration_obligation(
        model_pid: u64, real_pid: u64,
    ) -> bool {
        model_pid == real_pid
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
    ///
    /// A boundary RunnableProcess is well-formed when:
    /// - There is at least one ready thread (NonEmptyVecDeque invariant).
    /// - Ready thread IDs and admission times have matching lengths.
    /// - No duplicate thread IDs within any list.
    /// - All thread lists are pairwise disjoint.
    pub open spec fn wf(&self) -> bool {
        &&& self.ready_thread_ids@.len() >= 1
        &&& self.ready_thread_ids@.len() == self.ready_admission_times@.len()
        &&& Self::spec_no_duplicates(self.ready_thread_ids@)
        &&& Self::spec_no_duplicates(self.interrupted_thread_ids@)
        &&& Self::spec_no_duplicates(self.sleeping_thread_ids@)
        &&& Self::spec_no_duplicates(self.zombie_thread_ids@)
        &&& Self::spec_seqs_disjoint(self.ready_thread_ids@, self.interrupted_thread_ids@)
        &&& Self::spec_seqs_disjoint(self.ready_thread_ids@, self.sleeping_thread_ids@)
        &&& Self::spec_seqs_disjoint(self.ready_thread_ids@, self.zombie_thread_ids@)
        &&& Self::spec_seqs_disjoint(self.interrupted_thread_ids@, self.sleeping_thread_ids@)
        &&& Self::spec_seqs_disjoint(self.interrupted_thread_ids@, self.zombie_thread_ids@)
        &&& Self::spec_seqs_disjoint(self.sleeping_thread_ids@, self.zombie_thread_ids@)
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
}

//==================================================================================================
// View-Level Abstract State Transition Functions
//==================================================================================================

impl InterruptedProcessView {
    /// View-level well-formedness predicate.
    ///
    /// Mirrors `InterruptedProcess::wf()` but operates directly on
    /// abstract `Seq<u64>` fields.
    pub open spec fn wf(self) -> bool {
        &&& self.interrupted_thread_ids.len() >= 1
        &&& InterruptedProcess::spec_no_duplicates(self.interrupted_thread_ids)
        &&& InterruptedProcess::spec_no_duplicates(self.sleeping_thread_ids)
        &&& InterruptedProcess::spec_no_duplicates(self.zombie_thread_ids)
        &&& InterruptedProcess::spec_seqs_disjoint(
            self.interrupted_thread_ids, self.sleeping_thread_ids)
        &&& InterruptedProcess::spec_seqs_disjoint(
            self.interrupted_thread_ids, self.zombie_thread_ids)
        &&& InterruptedProcess::spec_seqs_disjoint(
            self.sleeping_thread_ids, self.zombie_thread_ids)
    }

    /// Abstract state transition: models `InterruptedProcess::new()`.
    ///
    /// Returns the expected view after constructing with no sleeping threads.
    pub open spec fn spec_new(
        pid: u64, interrupted_ids: Seq<u64>, zombie_ids: Seq<u64>,
    ) -> InterruptedProcessView {
        InterruptedProcessView {
            pid: pid,
            sleeping_thread_ids: Seq::empty(),
            interrupted_thread_ids: interrupted_ids,
            zombie_thread_ids: zombie_ids,
        }
    }

    /// Abstract state transition: models `InterruptedProcess::from_sleeping()`.
    ///
    /// Returns the expected view after constructing with sleeping threads.
    pub open spec fn spec_from_sleeping(
        pid: u64,
        sleeping_ids: Seq<u64>,
        interrupted_ids: Seq<u64>,
        zombie_ids: Seq<u64>,
    ) -> InterruptedProcessView {
        InterruptedProcessView {
            pid: pid,
            sleeping_thread_ids: sleeping_ids,
            interrupted_thread_ids: interrupted_ids,
            zombie_thread_ids: zombie_ids,
        }
    }

    /// Abstract state transition: models `InterruptedProcess::resume()`.
    ///
    /// Pops the front interrupted thread and produces a `RunnableProcessView`
    /// with that thread as the single ready thread at `admission_time`.
    /// Remaining interrupted, sleeping, and zombie threads are preserved.
    pub open spec fn spec_resume(self, admission_time: u64) -> RunnableProcessView {
        RunnableProcessView {
            pid: self.pid,
            ready_thread_ids: seq![self.interrupted_thread_ids[0]],
            ready_admission_times: seq![admission_time],
            interrupted_thread_ids: self.interrupted_thread_ids.subrange(
                1, self.interrupted_thread_ids.len() as int),
            sleeping_thread_ids: self.sleeping_thread_ids,
            zombie_thread_ids: self.zombie_thread_ids,
        }
    }

    /// Abstract state transition: models `InterruptedProcess::state_mut()`.
    ///
    /// Identity transition — mutable accessor preserves all fields.
    pub open spec fn spec_state_mut(self) -> InterruptedProcessView {
        self
    }

    /// Abstract state transition: models `InterruptedProcess::find_thread_mut()`.
    ///
    /// Identity transition — mutable search preserves all fields.
    pub open spec fn spec_find_thread_mut(self) -> InterruptedProcessView {
        self
    }
}

//==================================================================================================
// View Implementations
//==================================================================================================

impl View for InterruptedProcess {
    type V = InterruptedProcessView;

    open spec fn view(&self) -> InterruptedProcessView {
        InterruptedProcessView {
            pid: self.pid,
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
            pid: self.pid,
            ready_thread_ids: self.ready_thread_ids@,
            ready_admission_times: self.ready_admission_times@,
            interrupted_thread_ids: self.interrupted_thread_ids@,
            sleeping_thread_ids: self.sleeping_thread_ids@,
            zombie_thread_ids: self.zombie_thread_ids@,
        }
    }
}

} // verus!
