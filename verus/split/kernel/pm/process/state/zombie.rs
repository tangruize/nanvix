// Copyright(c) The Maintainers of Nanvix.
// Licensed under the MIT License.

//! # ZombieProcess — Design Verification
//!
//! Represents a process that finished its execution and is waiting for its
//! parent to collect its exit status and release its resources in the
//! Nanvix kernel. A ZombieProcess has at least one zombie thread, a process
//! state (PID), and an exit status.
//!
//! ## Verification Scope
//!
//! This is a **design-level (ghost model) verification**, not an
//! implementation verification. All struct fields are `Ghost<...>` types
//! and functions operate on ghost sequences. The verification proves that
//! the *state transition logic* is correct — threads are not lost or
//! duplicated, well-formedness invariants are preserved, and process
//! identity and exit status are immutable — but does NOT verify the
//! executable Rust code in `src/kernel/src/pm/process/state/zombie.rs`
//! directly.
//!
//! ## Verified Properties
//!
//! - Construction (new) produces well-formed state with correct identity and status.
//! - Process identifier (PID) is immutable: all operations preserve it.
//! - Exit status is immutable: all operations preserve it.
//! - `bury()` decomposes the process, returning zombie thread IDs, PID, and
//!   exit status — all matching the original fields exactly.
//! - `find_thread()` / `find_thread_mut()` are modeled spec-only via
//!   `spec_find_thread()`.
//! - `state()` / `state_mut()` are modeled with frame conditions.
//! - Well-formedness (including thread ID uniqueness) is preserved by all
//!   operations.
//!
//! ## Verification Model
//!
//! The original `ZombieProcess` contains complex kernel types. For verification:
//! - `NonEmptyVecDeque<ZombieThread>` -> `Seq<int>` of thread IDs (ghost), `len() >= 1`.
//! - `Box<ProcessState>` -> PID (int, identity tracking only).
//! - `ExitStatus` -> int.
//!
//! ## Trust Boundary
//!
//! - **`state_mut()` PID immutability (TRUST ASSUMPTION):** `state_mut()` is
//!   `external_body` and its postcondition asserts PID preservation. The
//!   original returns `&mut ProcessState`, giving callers unrestricted write
//!   access to all `ProcessState` fields. PID immutability is enforced
//!   architecturally: `ProcessState` does NOT expose a public setter for its
//!   `pid` field — only the constructor sets it. This is a trust assumption
//!   on the `ProcessState` module's API surface. If `ProcessState` ever
//!   exposes a PID setter, this postcondition becomes unsound and must be
//!   revised. Integration proofs must verify this assumption against the
//!   `ProcessState` API when that module is independently verified.
//! - **`state()` abstraction boundary:** `state()` returns `&ProcessState`
//!   which contains ~9 fields (pid, capabilities, vmem, events, mailbox,
//!   mmio, pmio, mutexes, conditions). Modeling it as a single `int` (PID)
//!   is a deliberate abstraction — this module only reasons about process
//!   identity. If future verification needs to reason about capabilities,
//!   vmem, or other `ProcessState` fields through `ZombieProcess`, the model
//!   must be extended.
//! - **`find_thread()` / `find_thread_mut()` (UNVERIFIED SEARCH):** These
//!   are **spec-level models** that compute `spec_find_thread()` directly in
//!   ghost mode. They do NOT model the executable `iter().find(...)` search.
//!   The original performs a linear search through `NonEmptyVecDeque::iter()`
//!   on the zombie thread list. The spec captures the search semantics but
//!   the executable iterator-based logic is NOT verified. Any bug in the real
//!   search (wrong predicate, wrong iteration order) would not be caught.
//!   `lemma_ghost_search_correctness` proves the ghost-level search logic is
//!   sound; `spec_find_thread_integration_obligation` defines the formal
//!   contract integration proofs must discharge. Tagged for trust-boundary
//!   inventory.
//! - **`bury()` ownership transfer:** The original `bury()` returns actual
//!   ownership of `(NonEmptyVecDeque<ZombieThread>, Box<ProcessState>,
//!   ExitStatus)` — transferring resources for the parent to collect. The
//!   ghost model returns `(Ghost<Seq<int>>, Ghost<int>, Ghost<int>)` and
//!   does NOT verify resource transfer or ownership semantics. This is the
//!   key semantic purpose of `bury()` in the original and is outside the
//!   ghost model scope. Executable verification would require modeling
//!   ownership transfer through Verus's tracked types.
//!
//! ## Fields
//!
//! All struct fields are `pub` for Verus proof ergonomics. The original has
//! private fields with getter/setter methods.

use vstd::prelude::*;

// Include specifications.
include!("zombie.spec.rs");

// Include proofs.
include!("zombie.proof.rs");

verus! {

//==================================================================================================
// Structures
//==================================================================================================

/// A process that finished its execution and is waiting for its parent
/// to collect its exit status and release its resources.
///
/// Verification model of `src/kernel/src/pm/process/state/zombie.rs::ZombieProcess`.
pub struct ZombieProcess {
    /// Process identifier (from the inner ProcessState).
    pub pid: Ghost<int>,
    /// Ghost sequence of zombie thread IDs (non-empty).
    pub zombie_thread_ids: Ghost<Seq<int>>,
    /// Exit status.
    pub status: Ghost<int>,
    /// Exec-level count of zombie threads.
    pub zombie_count: u64,
}

//==================================================================================================
// ZombieProcess Implementation
//==================================================================================================

impl ZombieProcess {
    /// Creates a new ZombieProcess.
    ///
    /// Models the original `ZombieProcess::new(process, zombie_threads, status)`.
    /// The original constructor takes `NonEmptyVecDeque<ZombieThread>` which
    /// implicitly knows its own length. In the ghost model, `zombie_count` is
    /// a separate exec-level parameter and callers at integration boundaries
    /// must establish `zombie_count as nat == zombie_ids@.len()`.
    ///
    /// # Parameters
    ///
    /// - `pid`: Process identifier.
    /// - `zombie_ids`: Ghost zombie thread IDs (must be non-empty).
    /// - `status`: Exit status.
    /// - `zombie_count`: Exec-level count of zombie threads (must match ghost length).
    ///
    /// # Returns
    ///
    /// A new, well-formed ZombieProcess.
    pub fn new(
        pid: Ghost<int>,
        zombie_ids: Ghost<Seq<int>>,
        status: Ghost<int>,
        zombie_count: u64,
    ) -> (result: ZombieProcess)
        requires
            zombie_count as nat == zombie_ids@.len(),
            zombie_ids@.len() >= 1,
            Self::spec_no_duplicates(zombie_ids@),
        ensures
            result.spec_pid() == pid@,
            result.zombie_thread_ids@ == zombie_ids@,
            result.spec_status() == status@,
            result.spec_zombie_count() == zombie_ids@.len(),
            result.wf(),
    {
        ZombieProcess {
            pid,
            zombie_thread_ids: zombie_ids,
            status,
            zombie_count,
        }
    }

    /// Returns the process state (modeled as PID).
    ///
    /// Models the original `ZombieProcess::state()` which returns `&ProcessState`.
    /// The original `ProcessState` contains ~9 fields (pid, capabilities, vmem,
    /// events, mailbox, mmio, pmio, mutexes, conditions). This ghost model only
    /// extracts PID — a deliberate abstraction for identity-focused verification.
    /// If future verification needs to reason about other `ProcessState` fields,
    /// this model must be extended.
    ///
    /// # Returns
    ///
    /// The process identifier.
    #[verifier::external_body]
    pub fn state(&self) -> (result: Ghost<int>)
        ensures
            result@ == self.spec_pid(),
    {
        unimplemented!()
    }

    /// Returns a mutable reference to the process state.
    ///
    /// Models the original `ZombieProcess::state_mut()` which returns
    /// `&mut ProcessState`.
    ///
    /// ## Trust Assumption: PID Immutability
    ///
    /// This `external_body` function's postcondition asserts PID preservation
    /// (`self.spec_pid() == old(self).spec_pid()`). The original gives callers
    /// unrestricted `&mut ProcessState` access. PID immutability is enforced
    /// architecturally: `ProcessState` does NOT expose a public setter for
    /// `pid` — only the constructor sets it. If `ProcessState` ever exposes
    /// a PID setter, this postcondition becomes unsound.
    ///
    /// # Returns
    ///
    /// The process identifier (as a ghost value).
    #[verifier::external_body]
    pub fn state_mut(&mut self) -> (result: Ghost<int>)
        ensures
            result@ == self.spec_pid(),
            self.spec_pid() == old(self).spec_pid(),
            self.zombie_thread_ids@ == old(self).zombie_thread_ids@,
            self.spec_status() == old(self).spec_status(),
            self.zombie_count == old(self).zombie_count,
    {
        unimplemented!()
    }

    /// Decomposes the ZombieProcess, returning its components.
    ///
    /// Models the original `ZombieProcess::bury()` which returns
    /// `(NonEmptyVecDeque<ZombieThread>, Box<ProcessState>, ExitStatus)`.
    /// The original transfers actual ownership of thread objects and process
    /// state to the caller (parent process) for resource cleanup. The ghost
    /// model captures identity preservation (PID, thread IDs, status) but
    /// does NOT verify resource transfer or ownership semantics.
    ///
    /// # Returns
    ///
    /// A tuple of (zombie_thread_ids, pid, status).
    pub fn bury(self) -> (result: (Ghost<Seq<int>>, Ghost<int>, Ghost<int>))
        requires
            self.wf(),
        ensures
            result.0@ == self@.zombie_thread_ids,
            result.1@ == self@.pid,
            result.2@ == self@.status,
            result.0@.len() >= 1,
            result.0@.len() == self.spec_zombie_count(),
    {
        (
            Ghost(self.zombie_thread_ids@),
            Ghost(self.pid@),
            Ghost(self.status@),
        )
    }

    /// Finds a thread by its identifier and returns which list it belongs to.
    ///
    /// **Spec-level model (UNVERIFIED SEARCH)** of the original
    /// `ZombieProcess::find_thread(tid)`. This function computes
    /// `spec_find_thread()` directly in ghost mode. It does NOT model the
    /// executable `iter().find(|t| t.id() == tid)` search. Any bug in the
    /// real search predicate or iteration logic would not be caught by this
    /// verification. `spec_find_thread_integration_obligation` defines the
    /// **unproven** contract that integration proofs must discharge.
    /// `lemma_ghost_search_correctness` proves the ghost-level search logic
    /// is sound for the abstract model.
    ///
    /// Returns the abstract list variant:
    /// - `Some(0)`: zombie thread found.
    /// - `None`: not found.
    ///
    /// # Parameters
    ///
    /// - `tid`: Ghost thread identifier to search for.
    ///
    /// # Returns
    ///
    /// The ghost list variant.
    pub fn find_thread(&self, tid: Ghost<int>) -> (result: Ghost<Option<int>>)
        ensures
            result@ == self.spec_find_thread(tid@),
    {
        Ghost(self.spec_find_thread(tid@))
    }

    /// Finds a thread by its identifier (mutable variant).
    ///
    /// **Spec-level model (UNVERIFIED SEARCH)** of the original
    /// `ZombieProcess::find_thread_mut(tid)`. Same trust scope as
    /// `find_thread()` — see its documentation. Frame condition: self is
    /// unchanged. The mutable reference in the original allows in-place
    /// mutation of the found thread. Callers must preserve the thread's
    /// identity and list membership after such mutation.
    ///
    /// # Parameters
    ///
    /// - `tid`: Ghost thread identifier to search for.
    ///
    /// # Returns
    ///
    /// The ghost list variant.
    pub fn find_thread_mut(&mut self, tid: Ghost<int>) -> (result: Ghost<Option<int>>)
        requires
            old(self).wf(),
        ensures
            result@ == old(self).spec_find_thread(tid@),
            self.spec_pid() == old(self).spec_pid(),
            self.zombie_thread_ids@ == old(self).zombie_thread_ids@,
            self.spec_status() == old(self).spec_status(),
            self.zombie_count == old(self).zombie_count,
            self.wf(),
    {
        Ghost(old(self).spec_find_thread(tid@))
    }
}

} // verus!
