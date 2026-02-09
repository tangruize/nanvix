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
//! - `find_thread()` / `find_thread_mut()` are **spec-level models** — they
//!   compute `spec_find_thread()` directly and do not model the executable
//!   search implementation. The original code performs a linear search through
//!   `iter().find(...)` on the zombie thread list. The spec captures this
//!   search semantics. The executable iterator-based search is NOT verified.
//!   Tagged for trust-boundary inventory.
//! - `state()` / `state_mut()` return references to ProcessState. Modeled
//!   as external_body with frame conditions. `state_mut()` permits arbitrary
//!   mutation; callers must preserve PID immutability.
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
    ///
    /// # Parameters
    ///
    /// - `pid`: Process identifier.
    /// - `zombie_ids`: Ghost zombie thread IDs (must be non-empty).
    /// - `status`: Exit status.
    /// - `zombie_count`: Exec-level count of zombie threads.
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
    /// Models the original `ZombieProcess::state()`.
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
    /// Models the original `ZombieProcess::state_mut()`.
    /// Callers must ensure PID immutability after mutation.
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
    /// Models the original `ZombieProcess::bury()`.
    /// Returns the zombie thread IDs, PID, and exit status.
    ///
    /// # Returns
    ///
    /// A tuple of (zombie_thread_ids, pid, status).
    pub fn bury(self) -> (result: (Ghost<Seq<int>>, Ghost<int>, Ghost<int>))
        requires
            self.wf(),
        ensures
            result.0@ == self.zombie_thread_ids@,
            result.1@ == self.spec_pid(),
            result.2@ == self.spec_status(),
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
    /// **Spec-level model** of the original `ZombieProcess::find_thread(tid)`.
    /// This function computes `spec_find_thread()` directly in ghost mode.
    /// It does NOT model the executable `iter().find(...)` search.
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
    /// **Spec-level model** of the original `ZombieProcess::find_thread_mut(tid)`.
    /// Same semantics as `find_thread()`. Frame condition: self is unchanged.
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
