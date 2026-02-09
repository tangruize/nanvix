// Copyright(c) The Maintainers of Nanvix.
// Licensed under the MIT License.

//! # ProcessState Implementation
//!
//! Manages per-process state in the Nanvix kernel: identity, capabilities,
//! virtual memory, events, mailbox, I/O resources, mutexes, and condition variables.
//!
//! ## Verified Properties
//!
//! - Construction produces well-formed state with correct initial values.
//! - Process identifier (`pid`) is immutable: all operations preserve it.
//! - `set_capability` / `clear_capability` / `has_capability` delegate correctly
//!   to the verified Capabilities type with well-formedness preservation.
//! - `get_mutex` enforces capacity bound (MUTEX_MAX = 32): returns `OutOfMemory`
//!   error **only** when the map is at capacity (spec_mutexes_full), inserts new
//!   entry with ref_count=2 when absent (BTreeMap entry + returned clone),
//!   increments ref_count when present (modeling `clone()`).
//! - `put_mutex` checks existence: returns error when not found. Conditionally
//!   removes entry only when `ref_count <= MUTEX_REMOVE_THRESHOLD` (2), matching
//!   the original `extract_if` with `reference_count() <= 2`.
//! - `get_cond` enforces capacity bound (COND_MAX = 32): same pattern as `get_mutex`,
//!   with `OutOfMemory` error only when `spec_conditions_full()`.
//! - `put_cond` conditionally removes when `ref_count <= COND_REMOVE_THRESHOLD` (1),
//!   matching the original `extract_if` with `reference_count() <= 1`.
//! - `add_pmio` / `remove_pmio` maintain I/O port sequence consistency.
//!   `remove_pmio` removes only the **first** matching entry (matching
//!   `LinkedList::remove` after `iter().position()`), enforced by a precondition
//!   requiring the ghost index to be the first occurrence, and decrements count
//!   by exactly 1.
//! - Well-formedness (`wf()`) is preserved by all operations, including capacity
//!   bounds (`mutex_count <= MUTEX_MAX`, `cond_count <= COND_MAX`).
//! - Frame-condition stubs for omitted HAL/IPC functions prove they do not
//!   modify verified state (PID, capabilities, mutexes, condvars, PMIO).
//!
//! ## Verification Model
//!
//! This is a **protocol-level verification model**, not a drop-in replacement
//! for the original `ProcessState`. The verification proves that the state
//! management *protocol* (capacity enforcement, ref-count-based cleanup,
//! PID immutability, error reporting) is correct by construction.
//!
//! The original `ProcessState` contains complex kernel types (`Vmem`,
//! `EventOwnership`, `Mailbox`, `IoMemoryRegion`, `AnyIoPort`, `Mutex`,
//! `Condvar`, `BTreeMap`, `LinkedList`) from HAL, MM, IPC, and sync
//! subsystems. Verus cannot verify the Rust standard library or kernel
//! HAL types. For verification we abstract these away:
//! - `vmem`, `events`, `mailbox`, `mmio` → elided (opaque HAL/MM/IPC boundary types).
//!   Frame-condition stubs prove functions on these fields preserve verified state.
//! - `mutexes` → `mutex_count: usize` (runtime counter) paired with
//!   ghost `ghost_mutexes: Map<int, nat>` modeling BTreeMap per-key semantics.
//!   The `nat` value represents the Arc strong reference count: `get_mutex`
//!   increments it (modeling `clone()`), `put_mutex` conditionally removes
//!   when it drops to the threshold.
//! - `conditions` → `cond_count: usize` (runtime counter) paired with
//!   ghost `ghost_conditions: Map<int, nat>` modeling BTreeMap per-key semantics.
//!   Same reference-counting model as mutexes.
//! - `pmio` → ghost `ghost_pmio: Seq<int>` modeling LinkedList of port numbers.
//!   Removal uses ghost index (matching `LinkedList::remove(index)` after
//!   `iter().position()`), not filter.
//! - `capabilities` → reuses the verified `Capabilities` type.
//! - `pid` → reuses the verified `ProcessIdentifier` type.
//!
//! ## Trust Assumptions
//!
//! - **T1: Capacity constants.** `MUTEX_MAX` (32) and `COND_MAX` (32) match
//!   `MUTEX_OPEN_MAX` and `COND_OPEN_MAX` from `build/kernel_config.toml`.
//! - **T2: BTreeMap semantics.** The ghost Map model faithfully represents
//!   BTreeMap insert/remove/contains_key behavior.
//! - **T3: Arc reference counting.** The ghost `nat` value directly models
//!   `Arc::strong_count()`. New entries start at 2 (one Arc in the BTreeMap,
//!   one returned clone). Each subsequent `get_mutex`/`get_cond` increments by
//!   1 (modeling `clone()`), and dropping the caller's clone decrements it.
//!   The original thresholds (2 for mutexes, 1 for condvars)
//!   correctly identify entries with no external references.
//!   Note: The model does not include explicit decrement operations because
//!   clone drops occur outside ProcessState (in calling code). The threshold
//!   check in `put_mutex`/`put_cond` observes the current count at the decision
//!   point, which is faithful to the original `extract_if` predicate.
//! - **T4: Frame conditions for omitted functions.** Functions marked
//!   `external_body` that interact with opaque types (Vmem, EventOwnership,
//!   Mailbox, IoMemoryRegion, AnyIoPort) do not modify the verified fields
//!   (PID, capabilities, mutexes, condvars, PMIO).
//! - **T5: Oracle parameters.** Functions like `get_mutex`, `put_mutex`, etc.
//!   take runtime boolean parameters (`already_present`, `contains`,
//!   `ref_count_at_threshold`, `found`) tied to ghost state via preconditions.
//!   This is Verus's standard "oracle parameter" pattern for bridging the
//!   ghost/exec boundary: ghost map operations return spec-level bools that
//!   cannot be used in exec-level `if` conditions. The preconditions constrain
//!   these parameters to exactly match the ghost state, so any caller that
//!   satisfies the precondition must have computed the correct value.
//!   **Caller burden:** This shifts correctness responsibility to call sites —
//!   callers must derive oracle values from runtime data structures (e.g.,
//!   `BTreeMap::contains_key()`) and the BTreeMap↔ghost map correspondence
//!   (T2) is the critical link ensuring the oracle values are correct.
//!
//! ## Verification Scope
//!
//! This verification proves the **state management protocol** is correct:
//! PID immutability, capability delegation, bounded collection management
//! with proper error reporting and reference-counted cleanup, and I/O port
//! tracking consistency.
//!
//! **ABI note:** The verified `ProcessState` struct is not ABI-compatible
//! with the original. It replaces runtime containers with ghost state and
//! counters. Method signatures use ghost parameters and oracle booleans.
//! This is inherent to the protocol-model approach: the verified model
//! establishes correctness of the protocol logic, which the original
//! implementation follows. The `_stub` suffix on frame-condition methods
//! distinguishes them from the original API to avoid confusion.
//!
//! The following are out of scope for deep verification but have frame-condition
//! stubs or opaque type models:
//! - Virtual memory operations (Vmem) — frame stubs provided.
//! - Event ownership management — frame stubs provided.
//! - Mailbox send/receive — frame stubs provided.
//! - Memory-mapped I/O management — frame stubs provided.
//! - Raw I/O port read/write operations — frame stubs provided.
//! - `ProcessRefMut`/`ProcessRef` enum dispatch — modeled as opaque
//!   `external_body` types with accessor stubs. The originals are thin
//!   wrappers over external process lifecycle types (`RunnableProcess`,
//!   `RunningProcess`, `SleepingProcess`, `InterruptedProcess`,
//!   `ZombieProcess`); their `state_mut()`/`state()` are pure dispatches.
//! - `get_pmio`/`get_pmio_mut` — private LinkedList traversal helpers
//!   modeled as frame-condition stubs; the LinkedList is abstracted to
//!   ghost `Seq<int>`.
//! - `Debug` impl — formatting trait stub; no logical effect on state.
//! - Return-value identity/ownership for `get_mutex`/`get_cond` — opaque
//!   `Arc<Mutex>`/`Arc<Condvar>` tokens cannot be modeled; ghost ref count
//!   captures the essential protocol information for cleanup decisions.

use crate::kernel::pm::sys::pid::ProcessIdentifier;
use crate::kernel::pm::process::capability::Capabilities;
use crate::kernel::pm::sys::capability::Capability;
use crate::libs::error::{Error, ErrorCode};
use vstd::prelude::*;

// Include specifications.
include!("process_state.spec.rs");

// Include proofs.
include!("process_state.proof.rs");

verus! {

//==================================================================================================
// Structures
//==================================================================================================

/// A type that represents the inner state of a process.
///
/// This is the verification model of `src/kernel/src/pm/process/state/mod.rs::ProcessState`.
/// Complex kernel types are abstracted to counters and ghost collections.
pub struct ProcessState {
    /// Process identifier (verified dependency).
    pub pid: ProcessIdentifier,
    /// Capabilities bitfield (verified dependency).
    pub capabilities: Capabilities,
    /// Number of mutexes in the map.
    pub mutex_count: usize,
    /// Ghost map of mutex addresses to reference counts (nat).
    /// Models `BTreeMap<MutexAddress, Mutex>` where each Mutex wraps an Arc.
    /// The nat value represents `Arc::strong_count()`.
    pub ghost_mutexes: Ghost<Map<int, nat>>,
    /// Number of condition variables in the map.
    pub cond_count: usize,
    /// Ghost map of condvar addresses to reference counts (nat).
    /// Models `BTreeMap<ConditionAddress, Condvar>` where each Condvar wraps an Arc.
    pub ghost_conditions: Ghost<Map<int, nat>>,
    /// Ghost sequence of I/O port numbers.
    /// Models `LinkedList<AnyIoPort>` preserving insertion order.
    pub ghost_pmio: Ghost<Seq<int>>,
}

//==================================================================================================
// Implementations
//==================================================================================================

impl ProcessState {
    /// Creates a new ProcessState.
    ///
    /// The original constructor takes `(pid, vmem)`. The `Vmem` parameter is
    /// omitted here because Vmem is an opaque HAL boundary type outside the
    /// verification scope (see Trust Assumption T4).
    ///
    /// # Parameters
    ///
    /// - `pid`: Process identifier.
    ///
    /// # Returns
    ///
    /// A new, well-formed ProcessState with no capabilities, empty mutex/condvar
    /// maps, and no I/O ports.
    pub fn new(pid: ProcessIdentifier) -> (result: ProcessState)
        ensures
            result.spec_pid() == pid.spec_value(),
            result.spec_capabilities_bits() == 0u8,
            result.spec_mutex_count() == 0,
            result.spec_cond_count() == 0,
            result.spec_pmio_count() == 0,
            result.wf(),
    {
        proof {
            assert(0u8 & 0b1110_0000u8 == 0u8) by (bit_vector);
        }
        ProcessState {
            pid: pid,
            capabilities: Capabilities { bits: 0u8 },
            mutex_count: 0usize,
            ghost_mutexes: Ghost(Map::empty()),
            cond_count: 0usize,
            ghost_conditions: Ghost(Map::empty()),
            ghost_pmio: Ghost(Seq::empty()),
        }
    }

    /// Returns the process identifier.
    ///
    /// # Returns
    ///
    /// The process identifier, unchanged from construction.
    pub fn pid(&self) -> (result: ProcessIdentifier)
        ensures
            result.spec_value() == self.spec_pid(),
    {
        self.pid
    }

    /// Sets a capability for this process.
    ///
    /// # Parameters
    ///
    /// - `capability`: The capability to grant.
    pub fn set_capability(&mut self, capability: Capability)
        requires
            old(self).wf(),
        ensures
            self.capabilities.spec_has(capability),
            self.spec_pid() == old(self).spec_pid(),
            self.spec_mutex_count() == old(self).spec_mutex_count(),
            self.spec_cond_count() == old(self).spec_cond_count(),
            self.spec_pmio_ports() == old(self).spec_pmio_ports(),
            forall|a: int| self.spec_has_mutex(a) == old(self).spec_has_mutex(a),
            forall|a: int| self.spec_has_cond(a) == old(self).spec_has_cond(a),
            self.wf(),
    {
        self.capabilities.set(capability);
    }

    /// Clears a capability for this process.
    ///
    /// # Parameters
    ///
    /// - `capability`: The capability to revoke.
    pub fn clear_capability(&mut self, capability: Capability)
        requires
            old(self).wf(),
        ensures
            !self.capabilities.spec_has(capability),
            self.spec_pid() == old(self).spec_pid(),
            self.spec_mutex_count() == old(self).spec_mutex_count(),
            self.spec_cond_count() == old(self).spec_cond_count(),
            self.spec_pmio_ports() == old(self).spec_pmio_ports(),
            forall|a: int| self.spec_has_mutex(a) == old(self).spec_has_mutex(a),
            forall|a: int| self.spec_has_cond(a) == old(self).spec_has_cond(a),
            self.wf(),
    {
        self.capabilities.clear(capability);
    }

    /// Tests whether this process has a given capability.
    ///
    /// # Parameters
    ///
    /// - `capability`: The capability to test.
    ///
    /// # Returns
    ///
    /// `true` if the capability is granted, `false` otherwise.
    pub fn has_capability(&self, capability: Capability) -> (result: bool)
        ensures
            result == self.capabilities.spec_has(capability),
    {
        self.capabilities.has(capability)
    }

    /// Returns a mutex associated with the given address, or creates one.
    ///
    /// Models the original `BTreeMap::entry(mutex_addr).or_insert_with(Mutex::new).clone()`.
    /// If the address already exists, the reference count is incremented (modeling `clone()`).
    /// If it does not exist, a new entry is inserted with ref_count = 2: one for the
    /// BTreeMap entry and one for the clone returned to the caller, matching
    /// `Arc::strong_count()` after `Mutex::new` + `.clone()`.
    /// Returns the reference count of the returned clone.
    ///
    /// **Known over-approximation (inherited from original):** The capacity check
    /// `self.mutexes.len() >= MUTEX_OPEN_MAX` runs before consulting the entry.
    /// At capacity, this rejects even existing keys whose `or_insert_with` would
    /// not grow the map. The spec faithfully models this behavior.
    ///
    /// # Parameters
    ///
    /// - `mutex_addr`: Abstract address of the mutex.
    /// - `already_present`: Runtime result of BTreeMap::contains_key, tied to ghost state.
    ///
    /// # Returns
    ///
    /// On success, returns the ghost reference count of the cloned mutex.
    /// If the map is full, returns an OutOfMemory error (even for existing keys).
    ///
    /// # Errors
    ///
    /// Returns `ErrorCode::OutOfMemory` when the map is at capacity.
    pub fn get_mutex(&mut self, mutex_addr: Ghost<int>, already_present: bool) -> (result: Result<Ghost<nat>, Error>)
        requires
            old(self).wf(),
            already_present == old(self).spec_has_mutex(mutex_addr@),
        ensures
            result is Ok ==> {
                &&& self.spec_has_mutex(mutex_addr@)
                // If was present: ref count incremented by 1 (modeling clone()).
                &&& already_present ==>
                        self.spec_mutex_ref_count(mutex_addr@)
                            == old(self).spec_mutex_ref_count(mutex_addr@) + 1
                // If was absent: new entry with ref_count = 2 (BTreeMap entry + returned clone).
                &&& !already_present ==> self.spec_mutex_ref_count(mutex_addr@) == 2
                // The returned ghost value is the new ref count.
                &&& result->Ok_0@ == self.spec_mutex_ref_count(mutex_addr@)
                &&& self.spec_pid() == old(self).spec_pid()
                &&& self.spec_capabilities_bits() == old(self).spec_capabilities_bits()
                &&& self.spec_cond_count() == old(self).spec_cond_count()
                &&& self.spec_pmio_ports() == old(self).spec_pmio_ports()
                &&& forall|a: int| self.spec_has_cond(a) == old(self).spec_has_cond(a)
                &&& forall|a: int| a != mutex_addr@ ==>
                        self.spec_has_mutex(a) == old(self).spec_has_mutex(a)
                &&& self.wf()
            },
            result is Err ==> {
                &&& result->Err_0.code == ErrorCode::OutOfMemory
                &&& old(self).spec_mutexes_full()
                &&& self.spec_pid() == old(self).spec_pid()
                &&& self.spec_capabilities_bits() == old(self).spec_capabilities_bits()
                &&& self.spec_mutex_count() == old(self).spec_mutex_count()
                &&& self.spec_cond_count() == old(self).spec_cond_count()
                &&& self.spec_pmio_ports() == old(self).spec_pmio_ports()
                &&& forall|a: int| self.spec_has_mutex(a) == old(self).spec_has_mutex(a)
                &&& forall|a: int| self.spec_has_cond(a) == old(self).spec_has_cond(a)
                &&& self.wf()
            },
    {
        if self.mutex_count >= Self::MUTEX_MAX_EXEC() {
            let reason: &'static str = "maximum number of mutexes reached";
            return Err(Error::new(ErrorCode::OutOfMemory, reason));
        }

        if already_present {
            // Already present: increment reference count (modeling clone()).
            let ghost old_rc: nat = self.ghost_mutexes@[mutex_addr@];
            self.ghost_mutexes = Ghost(
                self.ghost_mutexes@.insert(mutex_addr@, self.ghost_mutexes@[mutex_addr@] + 1)
            );
            let ghost new_rc: nat = self.ghost_mutexes@[mutex_addr@];
            Ok(Ghost(new_rc))
        } else {
            // Insert new entry with ref_count = 2 (one Arc in the BTreeMap + one clone returned).
            self.mutex_count = self.mutex_count + 1;
            self.ghost_mutexes = Ghost(self.ghost_mutexes@.insert(mutex_addr@, 2nat));
            Ok(Ghost(2nat))
        }
    }

    /// Releases a mutex associated with the given address.
    ///
    /// Models the original `extract_if` with predicate
    /// `mutex_addr == addr && mutex.reference_count() <= 2`. The mutex is only
    /// removed from the map when the reference count drops to the threshold (≤ 2),
    /// meaning only the BTreeMap entry and the caller's clone hold references.
    /// If the ref count is above the threshold, the entry remains (other holders exist).
    ///
    /// # Parameters
    ///
    /// - `mutex_addr`: Abstract address of the mutex.
    /// - `contains`: Runtime result of BTreeMap::contains_key, tied to ghost state.
    /// - `ref_count_at_threshold`: Whether the mutex's Arc strong count is ≤ 2.
    ///
    /// # Returns
    ///
    /// Upon success, empty result. Upon failure, an error.
    ///
    /// # Errors
    ///
    /// Returns `ErrorCode::NoSuchEntry` when the mutex is not found.
    pub fn put_mutex(
        &mut self,
        mutex_addr: Ghost<int>,
        contains: bool,
        ref_count_at_threshold: bool,
    ) -> (result: Result<(), Error>)
        requires
            old(self).wf(),
            contains == old(self).spec_has_mutex(mutex_addr@),
            contains ==> (ref_count_at_threshold ==
                (old(self).spec_mutex_ref_count(mutex_addr@) <= Self::MUTEX_REMOVE_THRESHOLD())),
        ensures
            result is Ok ==> {
                &&& old(self).spec_has_mutex(mutex_addr@)
                // If ref count was at threshold: entry removed.
                &&& ref_count_at_threshold ==> !self.spec_has_mutex(mutex_addr@)
                &&& ref_count_at_threshold ==>
                        self.spec_mutex_count() == old(self).spec_mutex_count() - 1
                // If ref count above threshold: entry remains, no removal.
                &&& !ref_count_at_threshold ==> self.spec_has_mutex(mutex_addr@)
                &&& !ref_count_at_threshold ==>
                        self.spec_mutex_count() == old(self).spec_mutex_count()
                &&& self.spec_pid() == old(self).spec_pid()
                &&& self.spec_capabilities_bits() == old(self).spec_capabilities_bits()
                &&& self.spec_cond_count() == old(self).spec_cond_count()
                &&& self.spec_pmio_ports() == old(self).spec_pmio_ports()
                &&& forall|a: int| self.spec_has_cond(a) == old(self).spec_has_cond(a)
                &&& forall|a: int| a != mutex_addr@ ==>
                        self.spec_has_mutex(a) == old(self).spec_has_mutex(a)
                &&& self.wf()
            },
            result is Err ==> {
                &&& !old(self).spec_has_mutex(mutex_addr@)
                &&& result->Err_0.code == ErrorCode::NoSuchEntry
                &&& self.spec_pid() == old(self).spec_pid()
                &&& self.spec_capabilities_bits() == old(self).spec_capabilities_bits()
                &&& self.spec_mutex_count() == old(self).spec_mutex_count()
                &&& self.spec_cond_count() == old(self).spec_cond_count()
                &&& self.spec_pmio_ports() == old(self).spec_pmio_ports()
                &&& forall|a: int| self.spec_has_mutex(a) == old(self).spec_has_mutex(a)
                &&& forall|a: int| self.spec_has_cond(a) == old(self).spec_has_cond(a)
                &&& self.wf()
            },
    {
        if !contains {
            let reason: &'static str = "mutex not found";
            return Err(Error::new(ErrorCode::NoSuchEntry, reason));
        }

        if ref_count_at_threshold {
            // Reference count at threshold: remove the entry.
            self.mutex_count = self.mutex_count - 1;
            self.ghost_mutexes = Ghost(self.ghost_mutexes@.remove(mutex_addr@));
        }
        // else: ref count above threshold — entry remains, no state change.
        Ok(())
    }

    /// Returns a condition variable associated with the given address, or creates one.
    ///
    /// Models the original `BTreeMap::entry(cond_addr).or_insert_with(Condvar::new).clone()`.
    /// Same reference-counting model as `get_mutex`: new entries get ref_count = 2
    /// (BTreeMap entry + returned clone), existing entries get ref_count incremented by 1.
    ///
    /// **Known over-approximation (inherited from original):** Same as `get_mutex` —
    /// capacity check runs before consulting the entry, rejecting existing keys at capacity.
    ///
    /// # Parameters
    ///
    /// - `cond_addr`: Abstract address of the condition variable.
    /// - `already_present`: Runtime result of BTreeMap::contains_key, tied to ghost state.
    ///
    /// # Returns
    ///
    /// On success, returns the ghost reference count of the cloned condvar.
    /// If the map is full, returns an OutOfMemory error (even for existing keys).
    ///
    /// # Errors
    ///
    /// Returns `ErrorCode::OutOfMemory` when the map is at capacity.
    pub fn get_cond(&mut self, cond_addr: Ghost<int>, already_present: bool) -> (result: Result<Ghost<nat>, Error>)
        requires
            old(self).wf(),
            already_present == old(self).spec_has_cond(cond_addr@),
        ensures
            result is Ok ==> {
                &&& self.spec_has_cond(cond_addr@)
                &&& already_present ==>
                        self.spec_cond_ref_count(cond_addr@)
                            == old(self).spec_cond_ref_count(cond_addr@) + 1
                &&& !already_present ==> self.spec_cond_ref_count(cond_addr@) == 2
                &&& result->Ok_0@ == self.spec_cond_ref_count(cond_addr@)
                &&& self.spec_pid() == old(self).spec_pid()
                &&& self.spec_capabilities_bits() == old(self).spec_capabilities_bits()
                &&& self.spec_mutex_count() == old(self).spec_mutex_count()
                &&& self.spec_pmio_ports() == old(self).spec_pmio_ports()
                &&& forall|a: int| self.spec_has_mutex(a) == old(self).spec_has_mutex(a)
                &&& forall|a: int| a != cond_addr@ ==>
                        self.spec_has_cond(a) == old(self).spec_has_cond(a)
                &&& self.wf()
            },
            result is Err ==> {
                &&& result->Err_0.code == ErrorCode::OutOfMemory
                &&& old(self).spec_conditions_full()
                &&& self.spec_pid() == old(self).spec_pid()
                &&& self.spec_capabilities_bits() == old(self).spec_capabilities_bits()
                &&& self.spec_mutex_count() == old(self).spec_mutex_count()
                &&& self.spec_cond_count() == old(self).spec_cond_count()
                &&& self.spec_pmio_ports() == old(self).spec_pmio_ports()
                &&& forall|a: int| self.spec_has_mutex(a) == old(self).spec_has_mutex(a)
                &&& forall|a: int| self.spec_has_cond(a) == old(self).spec_has_cond(a)
                &&& self.wf()
            },
    {
        if self.cond_count >= Self::COND_MAX_EXEC() {
            let reason: &'static str = "maximum number of condition variables reached";
            return Err(Error::new(ErrorCode::OutOfMemory, reason));
        }

        if already_present {
            let ghost old_rc: nat = self.ghost_conditions@[cond_addr@];
            self.ghost_conditions = Ghost(
                self.ghost_conditions@.insert(cond_addr@, self.ghost_conditions@[cond_addr@] + 1)
            );
            let ghost new_rc: nat = self.ghost_conditions@[cond_addr@];
            Ok(Ghost(new_rc))
        } else {
            self.cond_count = self.cond_count + 1;
            self.ghost_conditions = Ghost(self.ghost_conditions@.insert(cond_addr@, 2nat));
            Ok(Ghost(2nat))
        }
    }

    /// Releases a condition variable associated with the given address.
    ///
    /// Models the original `extract_if` with predicate
    /// `cond_addr == addr && cond.reference_count() <= 1`. The condvar is only
    /// removed from the map when the reference count drops to the threshold (≤ 1),
    /// meaning only the BTreeMap entry holds a reference.
    ///
    /// # Parameters
    ///
    /// - `cond_addr`: Abstract address of the condition variable.
    /// - `contains`: Runtime result of BTreeMap::contains_key, tied to ghost state.
    /// - `ref_count_at_threshold`: Whether the condvar's Arc strong count is ≤ 1.
    ///
    /// # Returns
    ///
    /// Upon success, empty result. Upon failure, an error.
    ///
    /// # Errors
    ///
    /// Returns `ErrorCode::NoSuchEntry` when the condition variable is not found.
    pub fn put_cond(
        &mut self,
        cond_addr: Ghost<int>,
        contains: bool,
        ref_count_at_threshold: bool,
    ) -> (result: Result<(), Error>)
        requires
            old(self).wf(),
            contains == old(self).spec_has_cond(cond_addr@),
            contains ==> (ref_count_at_threshold ==
                (old(self).spec_cond_ref_count(cond_addr@) <= Self::COND_REMOVE_THRESHOLD())),
        ensures
            result is Ok ==> {
                &&& old(self).spec_has_cond(cond_addr@)
                &&& ref_count_at_threshold ==> !self.spec_has_cond(cond_addr@)
                &&& ref_count_at_threshold ==>
                        self.spec_cond_count() == old(self).spec_cond_count() - 1
                &&& !ref_count_at_threshold ==> self.spec_has_cond(cond_addr@)
                &&& !ref_count_at_threshold ==>
                        self.spec_cond_count() == old(self).spec_cond_count()
                &&& self.spec_pid() == old(self).spec_pid()
                &&& self.spec_capabilities_bits() == old(self).spec_capabilities_bits()
                &&& self.spec_mutex_count() == old(self).spec_mutex_count()
                &&& self.spec_pmio_ports() == old(self).spec_pmio_ports()
                &&& forall|a: int| self.spec_has_mutex(a) == old(self).spec_has_mutex(a)
                &&& forall|a: int| a != cond_addr@ ==>
                        self.spec_has_cond(a) == old(self).spec_has_cond(a)
                &&& self.wf()
            },
            result is Err ==> {
                &&& !old(self).spec_has_cond(cond_addr@)
                &&& result->Err_0.code == ErrorCode::NoSuchEntry
                &&& self.spec_pid() == old(self).spec_pid()
                &&& self.spec_capabilities_bits() == old(self).spec_capabilities_bits()
                &&& self.spec_mutex_count() == old(self).spec_mutex_count()
                &&& self.spec_cond_count() == old(self).spec_cond_count()
                &&& self.spec_pmio_ports() == old(self).spec_pmio_ports()
                &&& forall|a: int| self.spec_has_mutex(a) == old(self).spec_has_mutex(a)
                &&& forall|a: int| self.spec_has_cond(a) == old(self).spec_has_cond(a)
                &&& self.wf()
            },
    {
        if !contains {
            let reason: &'static str = "condition variable not found";
            return Err(Error::new(ErrorCode::NoSuchEntry, reason));
        }

        if ref_count_at_threshold {
            self.cond_count = self.cond_count - 1;
            self.ghost_conditions = Ghost(self.ghost_conditions@.remove(cond_addr@));
        }
        Ok(())
    }

    /// Adds an I/O port to the process.
    ///
    /// # Parameters
    ///
    /// - `port_number`: Abstract port number to add (must be a valid u16 value).
    pub fn add_pmio(&mut self, port_number: Ghost<int>)
        requires
            old(self).wf(),
            0 <= port_number@ <= 0xFFFF,
        ensures
            self.spec_pmio_count() == old(self).spec_pmio_count() + 1,
            self.spec_pid() == old(self).spec_pid(),
            self.spec_capabilities_bits() == old(self).spec_capabilities_bits(),
            self.spec_mutex_count() == old(self).spec_mutex_count(),
            self.spec_cond_count() == old(self).spec_cond_count(),
            forall|a: int| self.spec_has_mutex(a) == old(self).spec_has_mutex(a),
            forall|a: int| self.spec_has_cond(a) == old(self).spec_has_cond(a),
            self.wf(),
    {
        self.ghost_pmio = Ghost(self.ghost_pmio@.push(port_number@));
    }

    /// Removes an I/O port from the process.
    ///
    /// Models the original `LinkedList::remove(index)` after `iter().position()`.
    /// Removes only the first matching entry, decrementing count by exactly 1.
    ///
    /// # Parameters
    ///
    /// - `port_number`: Abstract port number to remove.
    /// - `found`: Runtime result of the position search, tied to ghost state.
    /// - `found_idx`: Ghost index of the first matching entry in the PMIO sequence.
    ///   Must be the first occurrence (no earlier index has the same port number).
    ///
    /// # Returns
    ///
    /// On success, returns Ok. On failure, an error.
    ///
    /// # Errors
    ///
    /// Returns `ErrorCode::NoSuchEntry` when the port is not found.
    pub fn remove_pmio(
        &mut self,
        port_number: Ghost<int>,
        found: bool,
        found_idx: Ghost<int>,
    ) -> (result: Result<(), Error>)
        requires
            old(self).wf(),
            0 <= port_number@ <= 0xFFFF,
            found == old(self).spec_has_pmio(port_number@),
            // If found, the ghost index must be valid, point to the matching port,
            // and be the FIRST occurrence (matching `iter().position()` semantics).
            found ==> 0 <= found_idx@ < old(self).ghost_pmio@.len()
                && old(self).ghost_pmio@[found_idx@] == port_number@
                && forall|j: int| 0 <= j < found_idx@ ==>
                    old(self).ghost_pmio@[j] != port_number@,
        ensures
            result is Ok ==> {
                &&& old(self).spec_has_pmio(port_number@)
                // The removed entry had the requested port number.
                &&& old(self).ghost_pmio@[found_idx@] == port_number@
                &&& self.spec_pmio_count() == old(self).spec_pmio_count() - 1
                &&& self.spec_pid() == old(self).spec_pid()
                &&& self.spec_capabilities_bits() == old(self).spec_capabilities_bits()
                &&& self.spec_mutex_count() == old(self).spec_mutex_count()
                &&& self.spec_cond_count() == old(self).spec_cond_count()
                &&& forall|a: int| self.spec_has_mutex(a) == old(self).spec_has_mutex(a)
                &&& forall|a: int| self.spec_has_cond(a) == old(self).spec_has_cond(a)
                &&& self.wf()
            },
            result is Err ==> {
                &&& !old(self).spec_has_pmio(port_number@)
                &&& result->Err_0.code == ErrorCode::NoSuchEntry
                &&& self.spec_pid() == old(self).spec_pid()
                &&& self.spec_capabilities_bits() == old(self).spec_capabilities_bits()
                &&& self.spec_mutex_count() == old(self).spec_mutex_count()
                &&& self.spec_cond_count() == old(self).spec_cond_count()
                &&& self.spec_pmio_ports() == old(self).spec_pmio_ports()
                &&& forall|a: int| self.spec_has_mutex(a) == old(self).spec_has_mutex(a)
                &&& forall|a: int| self.spec_has_cond(a) == old(self).spec_has_cond(a)
                &&& self.wf()
            },
    {
        if !found {
            let reason: &'static str = "io port not found";
            return Err(Error::new(ErrorCode::NoSuchEntry, reason));
        }

        // Remove only the element at the found index (matching LinkedList::remove(index)).
        self.ghost_pmio = Ghost(
            self.ghost_pmio@.subrange(0, found_idx@)
                .add(self.ghost_pmio@.subrange(found_idx@ + 1, self.ghost_pmio@.len() as int))
        );
        Ok(())
    }

    //==============================================================================================
    // Frame-Condition Stubs for Omitted Functions
    //==============================================================================================
    //
    // The following functions interact with opaque HAL/MM/IPC boundary types.
    // They are marked `external_body` and specify frame conditions proving they
    // do not modify the verified state fields (PID, capabilities, mutexes,
    // condvars, PMIO). This increases verification coverage by formally
    // documenting the non-interference of these operations.
    //
    // NOTE: These stubs have simplified signatures compared to the originals.
    // They are not intended to be API-compatible replacements. Their sole purpose
    // is to assert frame conditions (non-interference with verified ghost state).
    // The original functions take additional parameters (buffer pointers, port
    // addresses, MMIO region descriptors, etc.) that are irrelevant to the
    // verified state model.
    //
    // SOUNDNESS NOTE on `&self` stubs: Stubs taking `&self` use `ensures true`.
    // In Verus's verification model, `&self` is truly immutable — Verus does not
    // support interior mutability patterns (Cell, RefCell, UnsafeCell). The
    // original ProcessState struct has no interior mutability fields: all fields
    // are plain data (ProcessIdentifier, Capabilities, BTreeMap, LinkedList,
    // Vmem). Therefore, `&self` in these stubs is a verified guarantee of
    // non-mutation, not merely a convention.

    /// Stub: copy_from_user_unaligned preserves verified state.
    /// Takes `&self` (immutable reference), so Rust's borrow checker prevents
    /// any mutation. No frame conditions needed beyond `true`.
    #[verifier::external_body]
    pub fn copy_from_user_unaligned_stub(&self) -> (result: Result<(), Error>)
        ensures
            // Frame: `&self` guarantees no state mutation (Rust ownership).
            true,
    {
        unimplemented!()
    }

    /// Stub: copy_to_user_unaligned preserves verified state.
    /// Takes `&self` (immutable reference), so Rust's borrow checker prevents
    /// any mutation. No frame conditions needed beyond `true`.
    #[verifier::external_body]
    pub fn copy_to_user_unaligned_stub(&self) -> (result: Result<(), Error>)
        ensures
            // Frame: `&self` guarantees no state mutation (Rust ownership).
            true,
    {
        unimplemented!()
    }

    /// Stub: add_event preserves verified state.
    #[verifier::external_body]
    pub fn add_event_stub(&mut self)
        requires
            old(self).wf(),
        ensures
            self.spec_pid() == old(self).spec_pid(),
            self.spec_capabilities_bits() == old(self).spec_capabilities_bits(),
            self.spec_mutex_count() == old(self).spec_mutex_count(),
            self.spec_cond_count() == old(self).spec_cond_count(),
            self.spec_pmio_ports() == old(self).spec_pmio_ports(),
            forall|a: int| self.spec_has_mutex(a) == old(self).spec_has_mutex(a),
            forall|a: int| self.spec_has_cond(a) == old(self).spec_has_cond(a),
            self.wf(),
    {
        unimplemented!()
    }

    /// Stub: remove_event preserves verified state.
    #[verifier::external_body]
    pub fn remove_event_stub(&mut self)
        requires
            old(self).wf(),
        ensures
            self.spec_pid() == old(self).spec_pid(),
            self.spec_capabilities_bits() == old(self).spec_capabilities_bits(),
            self.spec_mutex_count() == old(self).spec_mutex_count(),
            self.spec_cond_count() == old(self).spec_cond_count(),
            self.spec_pmio_ports() == old(self).spec_pmio_ports(),
            forall|a: int| self.spec_has_mutex(a) == old(self).spec_has_mutex(a),
            forall|a: int| self.spec_has_cond(a) == old(self).spec_has_cond(a),
            self.wf(),
    {
        unimplemented!()
    }

    /// Stub: post_message preserves verified state.
    ///
    /// Mailbox semantics (ordering, delivery guarantees) are intentionally
    /// out of scope. The original `post_message` enqueues a `Message` into the
    /// process mailbox. The stub only asserts frame conditions.
    #[verifier::external_body]
    pub fn post_message_stub(&mut self)
        requires
            old(self).wf(),
        ensures
            self.spec_pid() == old(self).spec_pid(),
            self.spec_capabilities_bits() == old(self).spec_capabilities_bits(),
            self.spec_mutex_count() == old(self).spec_mutex_count(),
            self.spec_cond_count() == old(self).spec_cond_count(),
            self.spec_pmio_ports() == old(self).spec_pmio_ports(),
            forall|a: int| self.spec_has_mutex(a) == old(self).spec_has_mutex(a),
            forall|a: int| self.spec_has_cond(a) == old(self).spec_has_cond(a),
            self.wf(),
    {
        unimplemented!()
    }

    /// Stub: receive_message preserves verified state.
    ///
    /// The original returns `Option<Message>` (dequeuing from the mailbox).
    /// Mailbox semantics are intentionally out of scope; the stub only
    /// asserts frame conditions on the verified state fields.
    // TODO: If mailbox semantics are brought into verification scope,
    // this stub would need a ghost message queue and dequeue postconditions.
    #[verifier::external_body]
    pub fn receive_message_stub(&mut self)
        requires
            old(self).wf(),
        ensures
            self.spec_pid() == old(self).spec_pid(),
            self.spec_capabilities_bits() == old(self).spec_capabilities_bits(),
            self.spec_mutex_count() == old(self).spec_mutex_count(),
            self.spec_cond_count() == old(self).spec_cond_count(),
            self.spec_pmio_ports() == old(self).spec_pmio_ports(),
            forall|a: int| self.spec_has_mutex(a) == old(self).spec_has_mutex(a),
            forall|a: int| self.spec_has_cond(a) == old(self).spec_has_cond(a),
            self.wf(),
    {
        unimplemented!()
    }

    /// Stub: add_mmio preserves verified state.
    #[verifier::external_body]
    pub fn add_mmio_stub(&mut self)
        requires
            old(self).wf(),
        ensures
            self.spec_pid() == old(self).spec_pid(),
            self.spec_capabilities_bits() == old(self).spec_capabilities_bits(),
            self.spec_mutex_count() == old(self).spec_mutex_count(),
            self.spec_cond_count() == old(self).spec_cond_count(),
            self.spec_pmio_ports() == old(self).spec_pmio_ports(),
            forall|a: int| self.spec_has_mutex(a) == old(self).spec_has_mutex(a),
            forall|a: int| self.spec_has_cond(a) == old(self).spec_has_cond(a),
            self.wf(),
    {
        unimplemented!()
    }

    /// Stub: remove_mmio preserves verified state.
    #[verifier::external_body]
    pub fn remove_mmio_stub(&mut self)
        requires
            old(self).wf(),
        ensures
            self.spec_pid() == old(self).spec_pid(),
            self.spec_capabilities_bits() == old(self).spec_capabilities_bits(),
            self.spec_mutex_count() == old(self).spec_mutex_count(),
            self.spec_cond_count() == old(self).spec_cond_count(),
            self.spec_pmio_ports() == old(self).spec_pmio_ports(),
            forall|a: int| self.spec_has_mutex(a) == old(self).spec_has_mutex(a),
            forall|a: int| self.spec_has_cond(a) == old(self).spec_has_cond(a),
            self.wf(),
    {
        unimplemented!()
    }

    /// Stub: read_pmio preserves verified state.
    /// Takes `&self` (immutable reference), so Rust's borrow checker prevents
    /// any mutation. No frame conditions needed beyond `true`.
    #[verifier::external_body]
    pub fn read_pmio_stub(&self) -> (result: Result<u32, Error>)
        ensures
            // Frame: `&self` guarantees no state mutation (Rust ownership).
            true,
    {
        unimplemented!()
    }

    /// Stub: write_pmio preserves verified state.
    #[verifier::external_body]
    pub fn write_pmio_stub(&mut self)
        requires
            old(self).wf(),
        ensures
            self.spec_pid() == old(self).spec_pid(),
            self.spec_capabilities_bits() == old(self).spec_capabilities_bits(),
            self.spec_mutex_count() == old(self).spec_mutex_count(),
            self.spec_cond_count() == old(self).spec_cond_count(),
            self.spec_pmio_ports() == old(self).spec_pmio_ports(),
            forall|a: int| self.spec_has_mutex(a) == old(self).spec_has_mutex(a),
            forall|a: int| self.spec_has_cond(a) == old(self).spec_has_cond(a),
            self.wf(),
    {
        unimplemented!()
    }

    /// Stub: vmem returns a reference, no state mutation.
    /// Takes `&self` (immutable reference), so Rust's borrow checker prevents
    /// any mutation. No frame conditions needed beyond `true`.
    #[verifier::external_body]
    pub fn vmem_stub(&self)
        ensures
            // Frame: `&self` guarantees no state mutation (Rust ownership).
            true,
    {
        unimplemented!()
    }

    /// Stub: vmem_mut preserves verified state.
    #[verifier::external_body]
    pub fn vmem_mut_stub(&mut self)
        requires
            old(self).wf(),
        ensures
            self.spec_pid() == old(self).spec_pid(),
            self.spec_capabilities_bits() == old(self).spec_capabilities_bits(),
            self.spec_mutex_count() == old(self).spec_mutex_count(),
            self.spec_cond_count() == old(self).spec_cond_count(),
            self.spec_pmio_ports() == old(self).spec_pmio_ports(),
            forall|a: int| self.spec_has_mutex(a) == old(self).spec_has_mutex(a),
            forall|a: int| self.spec_has_cond(a) == old(self).spec_has_cond(a),
            self.wf(),
    {
        unimplemented!()
    }

    //==============================================================================================
    // Private Helper Stubs
    //==============================================================================================

    /// Stub: get_pmio finds a port by number in the PMIO list (read-only).
    ///
    /// Original signature: `fn get_pmio(&self, port_number: u16) -> Result<&AnyIoPort, Error>`.
    /// This is a private LinkedList traversal helper used by `read_pmio` and `write_pmio`.
    /// Takes `&self`, so no state mutation is possible (Verus `&self` is truly
    /// immutable — no interior mutability in the verified model).
    #[verifier::external_body]
    fn get_pmio_stub(&self) -> (result: Result<(), Error>)
        ensures
            // Frame: `&self` guarantees no state mutation.
            true,
    {
        unimplemented!()
    }

    /// Stub: get_pmio_mut finds a port by number in the PMIO list (mutable).
    ///
    /// Original signature: `fn get_pmio_mut(&mut self, port_number: u16) -> Result<&mut AnyIoPort, Error>`.
    /// This is a private LinkedList traversal helper used by `write_pmio`.
    /// Returns a mutable reference to an `AnyIoPort` but does not modify
    /// verified state fields (PID, capabilities, mutexes, condvars, PMIO sequence).
    #[verifier::external_body]
    fn get_pmio_mut_stub(&mut self)
        requires
            old(self).wf(),
        ensures
            self.spec_pid() == old(self).spec_pid(),
            self.spec_capabilities_bits() == old(self).spec_capabilities_bits(),
            self.spec_mutex_count() == old(self).spec_mutex_count(),
            self.spec_cond_count() == old(self).spec_cond_count(),
            self.spec_pmio_ports() == old(self).spec_pmio_ports(),
            forall|a: int| self.spec_has_mutex(a) == old(self).spec_has_mutex(a),
            forall|a: int| self.spec_has_cond(a) == old(self).spec_has_cond(a),
            self.wf(),
    {
        unimplemented!()
    }

    /// Stub: Debug::fmt formats the ProcessState for display (read-only).
    ///
    /// Original: `impl Debug for ProcessState { fn fmt(&self, ...) }`.
    /// Takes `&self`, so no state mutation is possible.
    #[verifier::external_body]
    fn debug_fmt_stub(&self)
        ensures
            // Frame: `&self` guarantees no state mutation.
            true,
    {
        unimplemented!()
    }

    //==============================================================================================
    // Exec-level Constants
    //==============================================================================================

    /// Returns the capacity constant for mutexes (exec-level).
    fn MUTEX_MAX_EXEC() -> (result: usize)
        ensures
            result == Self::MUTEX_MAX(),
    {
        32usize
    }

    /// Returns the capacity constant for condition variables (exec-level).
    fn COND_MAX_EXEC() -> (result: usize)
        ensures
            result == Self::COND_MAX(),
    {
        32usize
    }
}

//==================================================================================================
// ProcessRefMut / ProcessRef — Lifecycle Accessor Wrappers
//==================================================================================================
//
// The original module defines `ProcessRefMut<'a>` and `ProcessRef<'a>` as enums
// wrapping mutable/immutable references to process lifecycle states (Runnable,
// Running, Sleeping, Interrupted, Zombie). Their sole purpose is to dispatch
// `state_mut()`/`state()` calls to the inner type's ProcessState accessor.
//
// These types depend on external process lifecycle types (RunnableProcess,
// RunningProcess, SleepingProcess, InterruptedProcess, ZombieProcess) that are
// defined in other modules and are not part of ProcessState's core state logic.
// We model them as opaque types with accessor specifications.

/// Opaque model of `ProcessRefMut<'a>`.
///
/// Original: an enum with variants Runnable, Running, Sleeping, Interrupted, Zombie,
/// each wrapping `&'a mut <LifecycleType>`. The `state_mut()` method dispatches
/// to the inner type's `state_mut()` returning `&mut ProcessState`.
#[verifier::external_body]
pub struct ProcessRefMut {
    _phantom: (),
}

impl ProcessRefMut {
    /// Stub: state_mut returns a mutable reference to the inner ProcessState.
    ///
    /// Original signature: `pub fn state_mut(&mut self) -> &mut ProcessState`.
    /// The accessor dispatches across enum variants but does not modify any
    /// ProcessState fields — it merely returns a reference.
    #[verifier::external_body]
    pub fn state_mut_stub(&mut self)
        ensures
            // The accessor is a pure dispatch; it does not modify state.
            // The returned reference allows the caller to modify ProcessState,
            // but that mutation is tracked by the caller's own verified code.
            true,
    {
        unimplemented!()
    }
}

/// Opaque model of `ProcessRef<'a>`.
///
/// Original: an enum with variants Runnable, Running, Sleeping, Interrupted, Zombie,
/// each wrapping `&'a <LifecycleType>`. The `state()` method dispatches
/// to the inner type's `state()` returning `&ProcessState`.
#[verifier::external_body]
pub struct ProcessRef {
    _phantom: (),
}

impl ProcessRef {
    /// Stub: state returns an immutable reference to the inner ProcessState.
    ///
    /// Original signature: `pub fn state(&self) -> &ProcessState`.
    /// The accessor dispatches across enum variants. Takes `&self`, so
    /// no state mutation is possible.
    #[verifier::external_body]
    pub fn state_stub(&self)
        ensures
            // Frame: `&self` guarantees no state mutation.
            true,
    {
        unimplemented!()
    }
}

} // verus!
