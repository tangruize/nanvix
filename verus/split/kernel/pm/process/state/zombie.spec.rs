// Copyright(c) The Maintainers of Nanvix.
// Licensed under the MIT License.

// ZombieProcess Specification (Design-Level Verification).
// This file contains spec functions and View types for the ZombieProcess type.
//
// ## Verification Scope
//
// This is a design-level (ghost model) specification. The spec functions define
// abstract properties over ghost sequences of thread IDs. Structural equivalence
// with the real executable code is assumed (see zombie.rs header).
//
// ## Verification Model
//
// ZombieProcess has at least one zombie thread, a process state (PID), and an
// exit status. For verification:
// - `NonEmptyVecDeque<ZombieThread>` is modeled as `Seq<int>` with `len() >= 1`.
// - `Box<ProcessState>` is transparent (modeled as PID only).
// - `ExitStatus` is modeled as `int`.
//
// ## Key Invariants
//
// - A ZombieProcess always has at least one zombie thread.
// - Process identity (PID) is immutable across all operations.
// - Exit status is immutable across all operations.
// - `bury()` decomposes the process, returning all components.
// - `find_thread()` searches only the zombie thread list.
//
// ## Ownership Semantics
//
// Thread ID uniqueness within the zombie list IS enforced in `wf()`.
// In the original code, Rust's ownership model ensures a thread struct
// can only appear once: `NonEmptyVecDeque<ZombieThread>` owns its elements,
// and `ZombieThread` is a move-only type. Thread IDs are allocated by the
// global thread subsystem (via `ThreadIdentifier`) which guarantees uniqueness.
// We model this ownership invariant explicitly via `spec_no_duplicates`.
// The original constructor does NOT check uniqueness at runtime — it is a
// pre-existing invariant from the thread subsystem, not a constructor-enforced
// constraint. See `wf()` documentation for full justification.
//
// ## Trust Assumptions
//
// - `find_thread()` and `find_thread_mut()` are modeled spec-only because
//   they return reference types (`ThreadRef`, `ThreadRefMut`) that Verus
//   cannot express. The spec model `spec_find_thread()` captures the search
//   semantics over the zombie list. The executable iterator-based search is
//   NOT verified — three integration obligations decompose the proof:
//   (1) `spec_find_thread_integration_obligation` (overall result match),
//   (2) `spec_find_thread_search_predicate_obligation` (per-element predicate
//   equivalence between `ZombieThread::id()` and ghost ID), and
//   (3) `spec_find_thread_mut_caller_obligation` (caller discipline for
//   mutable access). `lemma_ghost_search_correctness` proves the ghost-level
//   search logic. All three obligations remain **unproven** until integration
//   proofs discharge them.
// - `state()` / `state_mut()` return references to ProcessState in the original.
//   `state_mut()` permits arbitrary mutation; callers must preserve PID
//   immutability. PID immutability is enforced architecturally: `ProcessState`
//   has private `pid` field with no public setter (verified by inspection of
//   `src/kernel/src/pm/process/state/mod.rs`). This is a TRUST ASSUMPTION on
//   the ProcessState module's API surface. `spec_state_mut_pid_stability_obligation`
//   formalizes this requirement. Modeled as external_body with frame conditions.
// - `bury()` ownership transfer is not verified in the ghost model.
//   `spec_bury_ownership_integration_obligation` formalizes the identity part
//   of the transfer obligation. Full ownership transfer requires Verus tracked
//   types which are outside the ghost model's scope.

use vstd::prelude::*;

verus! {

//==================================================================================================
// View Types
//==================================================================================================

/// Abstract view of a ZombieProcess (this module's primary type).
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
// Spec Functions: ZombieProcess
//==================================================================================================

impl ZombieProcess {
    /// Spec function: returns the process identifier value.
    pub open spec fn spec_pid(&self) -> int {
        self.pid@
    }

    /// Spec function: returns the exit status.
    pub open spec fn spec_status(&self) -> int {
        self.status@
    }

    /// Spec function: returns the number of zombie threads.
    pub open spec fn spec_zombie_count(&self) -> nat {
        self.zombie_thread_ids@.len()
    }

    /// Spec helper: checks if a sequence contains a given value.
    pub open spec fn spec_seq_contains(s: Seq<int>, tid: int) -> bool {
        exists|i: int| 0 <= i < s.len() && s[i] == tid
    }

    /// Spec function: checks if a thread ID is in the zombie list.
    /// Defined in terms of `spec_seq_contains` for consistency.
    pub open spec fn spec_has_zombie_thread(&self, tid: int) -> bool {
        Self::spec_seq_contains(self.zombie_thread_ids@, tid)
    }

    /// Spec function: models `find_thread()` — returns whether a thread is found.
    ///
    /// - `Some(0)` if found in zombie threads.
    /// - `None` if not found.
    ///
    /// ZombieProcess only has one thread list, so the search is straightforward.
    pub open spec fn spec_find_thread(&self, tid: int) -> Option<int> {
        if self.spec_has_zombie_thread(tid) {
            Some(0int)
        } else {
            None
        }
    }

    /// Spec helper: checks whether a sequence has no duplicate elements.
    pub open spec fn spec_no_duplicates(s: Seq<int>) -> bool {
        forall|i: int, j: int| 0 <= i < j < s.len()
            ==> s[i] != s[j]
    }

    /// Spec function: well-formedness predicate.
    ///
    /// A ZombieProcess is well-formed when:
    /// - The zombie thread count matches the ghost sequence length.
    /// - There is at least one zombie thread (NonEmptyVecDeque invariant).
    /// - No duplicate thread IDs within the zombie list.
    ///
    /// ## Uniqueness Justification
    ///
    /// The `spec_no_duplicates` constraint is NOT checked by the original
    /// constructor — the original `new()` simply accepts a
    /// `NonEmptyVecDeque<ZombieThread>`. However, uniqueness is guaranteed
    /// by Rust's ownership model: each `ZombieThread` is a move-only struct
    /// that can exist in exactly one location. The `NonEmptyVecDeque` owns
    /// its elements, and thread IDs are assigned by the global thread
    /// subsystem (via `ThreadIdentifier`) which guarantees uniqueness at
    /// allocation time. The `no_duplicates` predicate models this ownership
    /// invariant explicitly for ghost-level reasoning.
    ///
    /// If a higher-level proof requires this invariant, it should link to
    /// the thread subsystem's uniqueness guarantee. Within this module,
    /// `no_duplicates` is a precondition on `new()` that callers must
    /// discharge by establishing the ownership invariant at the integration
    /// boundary.
    ///
    /// Note: Thread ID validity (e.g., non-negative, within valid range) is
    /// NOT enforced here. The original `ThreadIdentifier` is a structured type
    /// with constraints, but thread IDs are modeled as unbounded `int` in this
    /// ghost model. Thread ID validity is outside this module's verification
    /// scope and must be established by the thread module's verification.
    pub open spec fn wf(&self) -> bool {
        &&& self.zombie_count as nat == self.zombie_thread_ids@.len()
        &&& self.zombie_thread_ids@.len() >= 1
        &&& Self::spec_no_duplicates(self.zombie_thread_ids@)
    }

    /// Spec function: frame condition for mutable accessor (`state_mut()`).
    ///
    /// Mutations via `state_mut()` must not change any modeled fields.
    /// In the original code, `state_mut()` allows changing ProcessState
    /// fields (e.g., capabilities) but must not change the process identity
    /// (PID), the zombie thread list, or the exit status.
    pub open spec fn mutation_frame_preserved(old_self: &Self, new_self: &Self) -> bool {
        &&& new_self.spec_pid() == old_self.spec_pid()
        &&& new_self.zombie_thread_ids@ == old_self.zombie_thread_ids@
        &&& new_self.spec_status() == old_self.spec_status()
        &&& new_self.zombie_count == old_self.zombie_count
    }

    /// Integration obligation for `find_thread()` / `find_thread_mut()`.
    ///
    /// The spec model (`spec_find_thread`) assumes that the real executable
    /// `iter().find(|t| t.id() == tid)` search produces identical results.
    /// This obligation is **unproven** at this module level and must be
    /// discharged by integration proofs. Specifically, the integration proof
    /// must establish `spec_find_thread_search_predicate_obligation` (predicate
    /// equivalence) and that `Iterator::find`'s first-match semantics match
    /// the existential quantifier in `spec_has_zombie_thread` under `wf()`.
    pub open spec fn spec_find_thread_integration_obligation(
        &self, tid: int, real_result: Option<int>,
    ) -> bool {
        real_result == self.spec_find_thread(tid)
    }

    /// Integration obligation: search predicate equivalence.
    ///
    /// The real `find_thread()` uses `|thread| thread.id() == tid` as
    /// the search predicate. This obligation requires that for every
    /// zombie thread in the list, `thread.id()` produces the same integer
    /// as the corresponding element in `zombie_thread_ids@`. An integration
    /// proof must establish that:
    /// 1. `ZombieThread::id()` returns a `ThreadIdentifier`.
    /// 2. The `ThreadIdentifier` equality (`==`) matches integer equality
    ///    in the ghost model.
    /// 3. The ghost sequence `zombie_thread_ids@` is indexed in the same
    ///    order as the real `NonEmptyVecDeque<ZombieThread>` iteration.
    ///
    /// `ghost_id` is the ghost model's integer ID for a thread at some index.
    /// `real_id` is the value returned by `ZombieThread::id()` for the same
    /// thread. The obligation requires these are equal.
    pub open spec fn spec_find_thread_search_predicate_obligation(
        ghost_id: int, real_id: int,
    ) -> bool {
        ghost_id == real_id
    }

    /// Integration obligation: `find_thread_mut()` caller discipline.
    ///
    /// The original `find_thread_mut()` returns `Option<ThreadRefMut<'_>>`,
    /// giving callers mutable access to a `ZombieThread`. Callers that
    /// mutate through this reference must preserve:
    /// 1. The thread's identity (`thread.id()` is unchanged).
    /// 2. The thread remains in the zombie list (list membership unchanged).
    /// 3. The overall `ZombieProcess` well-formedness (`wf()`).
    ///
    /// This obligation cannot be enforced at this module level because Verus
    /// cannot model mutable borrow lifetimes. It must be discharged at each
    /// call site. `old_tid` is the thread's ID before mutation; `new_tid` is
    /// after. The obligation requires identity preservation.
    pub open spec fn spec_find_thread_mut_caller_obligation(
        old_tid: int, new_tid: int,
    ) -> bool {
        old_tid == new_tid
    }

    /// Integration obligation: `state_mut()` PID stability.
    ///
    /// The `state_mut()` `external_body` postcondition asserts PID
    /// preservation. This is justified by `ProcessState`'s API: `pid` is a
    /// private field (verified in `src/kernel/src/pm/process/state/mod.rs`)
    /// and no public setter exists — only `ProcessState::new()` sets `pid`,
    /// and only `ProcessState::pid()` reads it. The available mutators
    /// (`set_capability`, `clear_capability`, etc.) do not touch `pid`.
    ///
    /// This obligation formalizes the requirement: after any sequence of
    /// public method calls on `&mut ProcessState`, the PID is unchanged.
    /// Integration proofs must verify this against the ProcessState module's
    /// public API surface.
    pub open spec fn spec_state_mut_pid_stability_obligation(
        pid_before: int, pid_after: int,
    ) -> bool {
        pid_before == pid_after
    }

    /// Integration obligation for ProcessState PID linking.
    ///
    /// The ghost `pid` field in `ZombieProcess` is assumed to match the
    /// real `ProcessState::pid()` inside `Box<ProcessState>`. An integration
    /// proof must establish at construction time:
    ///   `ghost_pid == real_process_state.pid()`
    /// This link is preserved by all operations (proven by PID-preservation
    /// postconditions). The only construction site is `ZombieProcess::new()`
    /// which receives `Box<ProcessState>` — the integration proof must show
    /// that the ghost `pid` parameter equals the real `process.pid()`.
    pub open spec fn spec_process_state_pid_integration_obligation(
        ghost_pid: int, real_pid: int,
    ) -> bool {
        ghost_pid == real_pid
    }

    /// Integration obligation: `bury()` ownership transfer.
    ///
    /// The original `bury()` moves `self` and returns
    /// `(NonEmptyVecDeque<ZombieThread>, Box<ProcessState>, ExitStatus)`,
    /// transferring ownership of thread objects and process state to the
    /// caller (parent process) for resource cleanup. The ghost model only
    /// verifies identity preservation (PID, thread IDs, status match).
    ///
    /// An integration proof must establish that:
    /// 1. The returned `NonEmptyVecDeque<ZombieThread>` contains the same
    ///    thread objects (not just IDs) as the original `zombie_threads`.
    /// 2. The returned `Box<ProcessState>` is the same object as the original
    ///    `process` (pointer identity / ownership transfer).
    /// 3. The returned `ExitStatus` equals the original `status`.
    /// 4. The original `ZombieProcess` is fully consumed (no residual state).
    ///
    /// Properties 1-3 follow trivially from `bury()`'s implementation
    /// (field destructuring), but executable verification requires Verus
    /// tracked types which are outside this ghost model's scope.
    ///
    /// `ghost_ids` and `real_ids` are the ghost and real thread ID sequences.
    /// The obligation requires they match.
    pub open spec fn spec_bury_ownership_integration_obligation(
        ghost_ids: Seq<int>, real_ids: Seq<int>,
        ghost_pid: int, real_pid: int,
        ghost_status: int, real_status: int,
    ) -> bool {
        &&& ghost_ids =~= real_ids
        &&& ghost_pid == real_pid
        &&& ghost_status == real_status
    }
}

//==================================================================================================
// View Implementation
//==================================================================================================

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
