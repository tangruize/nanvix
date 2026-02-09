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
// can only appear once. We model this explicitly via `spec_no_duplicates`.
//
// ## Trust Assumptions
//
// - `find_thread()` and `find_thread_mut()` are modeled spec-only because
//   they return reference types (`ThreadRef`, `ThreadRefMut`) that Verus
//   cannot express. The spec model `spec_find_thread()` captures the search
//   semantics over the zombie list.
// - `state()` / `state_mut()` return references to ProcessState in the original.
//   `state_mut()` permits arbitrary mutation; callers must preserve PID
//   immutability. Modeled as external_body with frame conditions.

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

    /// Spec function: checks if a thread ID is in the zombie list.
    pub open spec fn spec_has_zombie_thread(&self, tid: int) -> bool {
        exists|i: int| 0 <= i < self.zombie_thread_ids@.len()
            && self.zombie_thread_ids@[i] == tid
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

    /// Spec helper: checks if a sequence contains a given value.
    pub open spec fn spec_seq_contains(s: Seq<int>, tid: int) -> bool {
        exists|i: int| 0 <= i < s.len() && s[i] == tid
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
    /// An integration proof must verify:
    /// 1. The search predicate `t.id() == tid` matches `spec_has_zombie_thread`.
    /// 2. The first-match semantics of `Iterator::find` match the
    ///    existential quantifier in `spec_has_zombie_thread` under the
    ///    no-duplicates invariant from `wf()`.
    pub open spec fn spec_find_thread_integration_obligation(
        &self, tid: int, real_result: Option<int>,
    ) -> bool {
        real_result == self.spec_find_thread(tid)
    }

    /// Integration obligation for ProcessState PID linking.
    ///
    /// The ghost `pid` field in `ZombieProcess` is assumed to match the
    /// real `ProcessState::pid()` inside `Box<ProcessState>`. An integration
    /// proof must establish: `ghost_pid == real_process_state.pid()`.
    pub open spec fn spec_process_state_pid_integration_obligation(
        ghost_pid: int, real_pid: int,
    ) -> bool {
        ghost_pid == real_pid
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
