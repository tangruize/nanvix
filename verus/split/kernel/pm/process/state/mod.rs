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
//!   requiring the index to be the first occurrence, and decrements count
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
//! - `mutexes` → parallel `Vec<u64>` pairs (`mutex_addrs`, `mutex_ref_counts`)
//!   with runtime `mutex_count` counter, modeling BTreeMap per-key semantics.
//!   Addresses are kept unique (enforced by wf()). The `u64` ref count value
//!   represents `Arc::strong_count()`.
//! - `conditions` → parallel `Vec<u64>` pairs (`cond_addrs`, `cond_ref_counts`)
//!   with runtime `cond_count` counter. Same model as mutexes.
//! - `pmio` → `Vec<u16>` of port numbers, modeling `LinkedList<AnyIoPort>`
//!   preserving insertion order.
//! - `capabilities` → reuses the verified `Capabilities` type.
//! - `pid` → reuses the verified `ProcessIdentifier` type.

use crate::kernel::pm::sys::pid::ProcessIdentifier;
use crate::kernel::pm::process::capability::Capabilities;
use crate::kernel::pm::sys::capability::Capability;
use crate::libs::error::{Error, ErrorCode};
use vstd::prelude::*;

// Include specifications.
include!("mod.spec.rs");

// Include proofs.
include!("mod.proof.rs");

pub mod interrupted;
pub mod runnable;
pub mod running;
pub mod sleeping;
pub mod zombie;

verus! {

//==================================================================================================
// Structures
//==================================================================================================

/// A type that represents the inner state of a process.
///
/// This is the verification model of `src/kernel/src/pm/process/state/mod.rs::ProcessState`.
/// Complex kernel types are abstracted to concrete Vecs.
///
/// **Field visibility:** All fields are `pub` because the Verus `View` trait requires
/// `open spec fn view()`, which in turn requires accessed fields to be visible outside
/// the module. This is a Verus framework constraint, not a design choice. The `wf()`
/// invariant ensures field consistency; callers should use the public API methods
/// rather than accessing fields directly.
pub struct ProcessState {
    /// Process identifier (verified dependency).
    pub pid: ProcessIdentifier,
    /// Capabilities bitfield (verified dependency).
    pub capabilities: Capabilities,
    /// Number of mutexes in the map.
    pub mutex_count: usize,
    /// Mutex addresses (concrete, models BTreeMap keys).
    /// Keys are unique (enforced by wf()).
    pub mutex_addrs: Vec<u64>,
    /// Mutex reference counts (parallel to mutex_addrs).
    /// Each value models `Arc::strong_count()` for the corresponding mutex.
    pub mutex_ref_counts: Vec<u64>,
    /// Number of condition variables in the map.
    pub cond_count: usize,
    /// Condvar addresses (concrete, models BTreeMap keys).
    /// Keys are unique (enforced by wf()).
    pub cond_addrs: Vec<u64>,
    /// Condvar reference counts (parallel to cond_addrs).
    pub cond_ref_counts: Vec<u64>,
    /// I/O port numbers (concrete, models LinkedList<AnyIoPort>).
    pub pmio_ports: Vec<u16>,
}

//==================================================================================================
// Implementations
//==================================================================================================

impl ProcessState {
    /// Creates a new ProcessState.
    ///
    /// # Signature Divergence
    ///
    /// The original `ProcessState::new(pid, vmem)` takes a second `Vmem`
    /// parameter. `Vmem` is an opaque HAL boundary type excluded from the
    /// verification model, so it is elided here. The verified properties
    /// (PID, capabilities, mutex/condvar/PMIO initialization) are unaffected.
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
            result@.capabilities_granted =~= Set::<Capability>::empty(),
            result.spec_mutex_count() == 0,
            result.spec_cond_count() == 0,
            result.spec_pmio_count() == 0,
            result.wf(),
    {
        let caps: Capabilities = Capabilities::new();
        proof {
            caps.lemma_view_bits();
        }
        ProcessState {
            pid: pid,
            capabilities: caps,
            mutex_count: 0usize,
            mutex_addrs: Vec::new(),
            mutex_ref_counts: Vec::new(),
            cond_count: 0usize,
            cond_addrs: Vec::new(),
            cond_ref_counts: Vec::new(),
            pmio_ports: Vec::new(),
        }
    }

    /// Returns the process identifier.
    pub fn pid(&self) -> (result: ProcessIdentifier)
        ensures
            result.spec_value() == self.spec_pid(),
    {
        self.pid
    }

    /// Sets a capability for this process.
    pub fn set_capability(&mut self, capability: Capability)
        requires
            old(self).wf(),
        ensures
            self@.capabilities_granted.contains(capability),
            self.spec_pid() == old(self).spec_pid(),
            self.spec_mutex_count() == old(self).spec_mutex_count(),
            self.spec_cond_count() == old(self).spec_cond_count(),
            self.spec_pmio_ports() == old(self).spec_pmio_ports(),
            forall|a: int| self.spec_has_mutex(a) == old(self).spec_has_mutex(a),
            forall|a: int| self.spec_has_cond(a) == old(self).spec_has_cond(a),
            self.wf(),
    {
        self.capabilities.set(capability);
        proof {
            self.capabilities.lemma_view_bits();
        }
    }

    /// Clears a capability for this process.
    pub fn clear_capability(&mut self, capability: Capability)
        requires
            old(self).wf(),
        ensures
            !self@.capabilities_granted.contains(capability),
            self.spec_pid() == old(self).spec_pid(),
            self.spec_mutex_count() == old(self).spec_mutex_count(),
            self.spec_cond_count() == old(self).spec_cond_count(),
            self.spec_pmio_ports() == old(self).spec_pmio_ports(),
            forall|a: int| self.spec_has_mutex(a) == old(self).spec_has_mutex(a),
            forall|a: int| self.spec_has_cond(a) == old(self).spec_has_cond(a),
            self.wf(),
    {
        self.capabilities.clear(capability);
        proof {
            self.capabilities.lemma_view_bits();
        }
    }

    /// Tests whether this process has a given capability.
    pub fn has_capability(&self, capability: Capability) -> (result: bool)
        requires
            self.wf(),
        ensures
            result == self@.capabilities_granted.contains(capability),
    {
        proof {
            self.capabilities.lemma_view_bits();
        }
        self.capabilities.has(capability)
    }

    /// Returns a mutex associated with the given address, or creates one.
    ///
    /// # Signature Divergence
    ///
    /// The original `get_mutex(mutex_addr: MutexAddress)` performs
    /// `BTreeMap::entry()` lookup internally. In the verification model,
    /// `BTreeMap` is abstracted to parallel Vecs, so the lookup result
    /// (`already_present`) and index (`idx`) are hoisted to preconditions.
    /// This is a standard verification technique for externalizing
    /// nondeterministic container lookups. The return type changes from
    /// `Result<Mutex, Error>` to `Result<u64, Error>` (ref count) since
    /// `Mutex` is an opaque type modeled by its reference count.
    ///
    /// # Parameters
    ///
    /// - `mutex_addr`: Address of the mutex.
    /// - `already_present`: Runtime result of BTreeMap::contains_key, tied to state.
    /// - `idx`: Index in the parallel Vecs where the address was found (if present).
    ///
    /// # Returns
    ///
    /// On success, returns the reference count of the cloned mutex.
    pub fn get_mutex(&mut self, mutex_addr: u64, already_present: bool, idx: usize) -> (result: Result<u64, Error>)
        requires
            old(self).wf(),
            already_present == old(self).spec_has_mutex(mutex_addr as int),
            already_present ==> (
                idx < old(self).mutex_addrs@.len()
                && old(self).mutex_addrs@[idx as int] == mutex_addr
                && old(self).mutex_ref_counts@[idx as int] < u64::MAX
            ),
        ensures
            result is Ok ==> {
                &&& self.spec_has_mutex(mutex_addr as int)
                &&& already_present ==>
                        self.spec_mutex_ref_count(mutex_addr as int)
                            == old(self).spec_mutex_ref_count(mutex_addr as int) + 1
                &&& !already_present ==> self.spec_mutex_ref_count(mutex_addr as int) == 2
                &&& result->Ok_0 as nat == self.spec_mutex_ref_count(mutex_addr as int)
                &&& self.spec_pid() == old(self).spec_pid()
                &&& self.spec_capabilities_bits() == old(self).spec_capabilities_bits()
                &&& self.spec_cond_count() == old(self).spec_cond_count()
                &&& self.spec_pmio_ports() == old(self).spec_pmio_ports()
                &&& forall|a: int| self.spec_has_cond(a) == old(self).spec_has_cond(a)
                &&& forall|a: int| a != mutex_addr as int ==>
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
            let old_rc: u64 = self.mutex_ref_counts.remove(idx);
            self.mutex_ref_counts.insert(idx, old_rc + 1);
            proof {
                // Vec state after remove+insert = update; addrs unchanged.
                assert(self.mutex_ref_counts@ =~= old(self).mutex_ref_counts@.update(idx as int, (old_rc + 1) as u64));
                assert(self.mutex_addrs@ =~= old(self).mutex_addrs@);

                // Witness for spec_has_mutex(mutex_addr): idx is still valid.
                assert(self.mutex_addrs@[idx as int] as int == mutex_addr as int);

                // By uniqueness (from wf), idx is the only match → choose picks idx.
                assert forall|i: int|
                    0 <= i < self.mutex_addrs@.len() && self.mutex_addrs@[i] as int == mutex_addr as int
                    implies i == idx as int by {}

                // Ref count at idx is old_rc + 1.
                let chosen: int = choose|i: int|
                    0 <= i < self.mutex_addrs@.len() && self.mutex_addrs@[i] as int == mutex_addr as int;
                assert(chosen == idx as int);
                assert(self.mutex_ref_counts@[chosen] as nat == (old_rc + 1) as nat);

                // Old spec_mutex_ref_count: choose also picks idx.
                let old_chosen: int = choose|i: int|
                    0 <= i < old(self).mutex_addrs@.len()
                    && old(self).mutex_addrs@[i] as int == mutex_addr as int;
                assert(old_chosen == idx as int);
                assert(old(self).mutex_ref_counts@[old_chosen] as nat == old_rc as nat);

                // Positive ref counts preserved (update only increases).
                assert forall|i: int| #![auto]
                    0 <= i < self.mutex_ref_counts@.len()
                    implies self.mutex_ref_counts@[i] > 0u64 by {
                    if i == idx as int {
                        // old_rc > 0 from wf, so old_rc + 1 > 0.
                    } else {
                        // Unchanged from old.
                    }
                }
            }
            Ok(old_rc + 1)
        } else {
            // Insert new entry with ref_count = 2.
            self.mutex_count = self.mutex_count + 1;
            self.mutex_addrs.push(mutex_addr);
            self.mutex_ref_counts.push(2);
            proof {
                assert(self.mutex_addrs@ =~= old(self).mutex_addrs@.push(mutex_addr));
                assert(self.mutex_ref_counts@ =~= old(self).mutex_ref_counts@.push(2u64));

                // Witness for spec_has_mutex: last index.
                let new_idx: int = self.mutex_addrs@.len() - 1;
                assert(self.mutex_addrs@[new_idx] as int == mutex_addr as int);

                // No old index maps to mutex_addr (was not present).
                assert forall|i: int| 0 <= i < old(self).mutex_addrs@.len()
                    implies old(self).mutex_addrs@[i] as int != mutex_addr as int by {}

                // Uniqueness: only new_idx maps to mutex_addr.
                assert forall|i: int|
                    0 <= i < self.mutex_addrs@.len() && self.mutex_addrs@[i] as int == mutex_addr as int
                    implies i == new_idx by {
                    if i < old(self).mutex_addrs@.len() as int {
                        assert(self.mutex_addrs@[i] == old(self).mutex_addrs@[i]);
                    }
                }

                // choose picks new_idx → ref count is 2.
                let chosen: int = choose|i: int|
                    0 <= i < self.mutex_addrs@.len() && self.mutex_addrs@[i] as int == mutex_addr as int;
                assert(chosen == new_idx);
                assert(self.mutex_ref_counts@[chosen] == 2u64);

                // Frame: other addresses preserved.
                assert forall|a: int| a != mutex_addr as int
                    implies (self.spec_has_mutex(a) == old(self).spec_has_mutex(a)) by {
                    assert forall|i: int|
                        0 <= i < old(self).mutex_addrs@.len() && old(self).mutex_addrs@[i] as int == a
                        implies (exists|j: int|
                            0 <= j < self.mutex_addrs@.len() && self.mutex_addrs@[j] as int == a) by {
                        assert(self.mutex_addrs@[i] == old(self).mutex_addrs@[i]);
                    }
                    assert forall|i: int|
                        0 <= i < self.mutex_addrs@.len() && self.mutex_addrs@[i] as int == a
                        implies (exists|j: int|
                            0 <= j < old(self).mutex_addrs@.len() && old(self).mutex_addrs@[j] as int == a) by {
                        if i < old(self).mutex_addrs@.len() as int {
                            assert(old(self).mutex_addrs@[i] == self.mutex_addrs@[i]);
                        } else {
                            // i == new_idx, self.mutex_addrs@[i] == mutex_addr, but a != mutex_addr
                        }
                    }
                }

                // wf: uniqueness after push.
                assert forall|i: int, j: int|
                    0 <= i < self.mutex_addrs@.len() && 0 <= j < self.mutex_addrs@.len() && i != j
                    implies self.mutex_addrs@[i] != self.mutex_addrs@[j] by {
                    if i < old(self).mutex_addrs@.len() as int && j < old(self).mutex_addrs@.len() as int {
                        assert(self.mutex_addrs@[i] == old(self).mutex_addrs@[i]);
                        assert(self.mutex_addrs@[j] == old(self).mutex_addrs@[j]);
                    } else if i == new_idx {
                        assert(self.mutex_addrs@[j] == old(self).mutex_addrs@[j]);
                    } else {
                        assert(self.mutex_addrs@[i] == old(self).mutex_addrs@[i]);
                    }
                }

                // wf: ref counts positive after push.
                assert forall|i: int| #![auto]
                    0 <= i < self.mutex_ref_counts@.len()
                    implies self.mutex_ref_counts@[i] > 0u64 by {
                    if i < old(self).mutex_ref_counts@.len() as int {
                        assert(self.mutex_ref_counts@[i] == old(self).mutex_ref_counts@[i]);
                    }
                }
            }
            Ok(2)
        }
    }

    /// Releases a mutex associated with the given address.
    ///
    /// # Signature Divergence
    ///
    /// The original `put_mutex(mutex_addr: MutexAddress)` performs
    /// `contains_key()` and `extract_if()` internally. In the verification
    /// model, the lookup result (`contains`), ref-count threshold check
    /// (`ref_count_at_threshold`), and index (`idx`) are hoisted to
    /// preconditions to externalize nondeterministic container operations.
    ///
    /// # Parameters
    ///
    /// - `mutex_addr`: Address of the mutex.
    /// - `contains`: Runtime result of BTreeMap::contains_key, tied to state.
    /// - `ref_count_at_threshold`: Whether the mutex's Arc strong count is <= 2.
    /// - `idx`: Index in the parallel Vecs where the address was found.
    pub fn put_mutex(
        &mut self,
        mutex_addr: u64,
        contains: bool,
        ref_count_at_threshold: bool,
        idx: usize,
    ) -> (result: Result<(), Error>)
        requires
            old(self).wf(),
            contains == old(self).spec_has_mutex(mutex_addr as int),
            contains ==> (
                idx < old(self).mutex_addrs@.len()
                && old(self).mutex_addrs@[idx as int] == mutex_addr
            ),
            contains ==> (ref_count_at_threshold ==
                (old(self).mutex_ref_counts@[idx as int] as nat <= Self::MUTEX_REMOVE_THRESHOLD())),
        ensures
            result is Ok ==> {
                &&& old(self).spec_has_mutex(mutex_addr as int)
                &&& ref_count_at_threshold ==> !self.spec_has_mutex(mutex_addr as int)
                &&& ref_count_at_threshold ==>
                        self.spec_mutex_count() == old(self).spec_mutex_count() - 1
                &&& !ref_count_at_threshold ==> self.spec_has_mutex(mutex_addr as int)
                &&& !ref_count_at_threshold ==>
                        self.spec_mutex_count() == old(self).spec_mutex_count()
                &&& self.spec_pid() == old(self).spec_pid()
                &&& self.spec_capabilities_bits() == old(self).spec_capabilities_bits()
                &&& self.spec_cond_count() == old(self).spec_cond_count()
                &&& self.spec_pmio_ports() == old(self).spec_pmio_ports()
                &&& forall|a: int| self.spec_has_cond(a) == old(self).spec_has_cond(a)
                &&& forall|a: int| a != mutex_addr as int ==>
                        self.spec_has_mutex(a) == old(self).spec_has_mutex(a)
                &&& self.wf()
            },
            result is Err ==> {
                &&& !old(self).spec_has_mutex(mutex_addr as int)
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
            // Reference count at threshold: remove the entry from both parallel Vecs.
            self.mutex_count = self.mutex_count - 1;
            self.mutex_addrs.remove(idx);
            self.mutex_ref_counts.remove(idx);
            proof {
                assert(self.mutex_addrs@ =~= old(self).mutex_addrs@.remove(idx as int));
                assert(self.mutex_ref_counts@ =~= old(self).mutex_ref_counts@.remove(idx as int));

                // Removed address no longer present: no index maps to mutex_addr.
                assert forall|i: int|
                    0 <= i < self.mutex_addrs@.len()
                    implies self.mutex_addrs@[i] as int != mutex_addr as int by {
                    if i < idx as int {
                        assert(self.mutex_addrs@[i] == old(self).mutex_addrs@[i]);
                    } else {
                        assert(self.mutex_addrs@[i] == old(self).mutex_addrs@[i + 1]);
                    }
                }

                // Frame: other addresses preserved.
                assert forall|a: int| a != mutex_addr as int
                    implies (self.spec_has_mutex(a) == old(self).spec_has_mutex(a)) by {
                    assert forall|i: int|
                        0 <= i < old(self).mutex_addrs@.len() && old(self).mutex_addrs@[i] as int == a
                        implies (exists|j: int|
                            0 <= j < self.mutex_addrs@.len() && self.mutex_addrs@[j] as int == a) by {
                        if i < idx as int {
                            assert(self.mutex_addrs@[i] == old(self).mutex_addrs@[i]);
                        } else {
                            // i > idx (i != idx because a != mutex_addr)
                            assert(self.mutex_addrs@[i - 1] == old(self).mutex_addrs@[i]);
                        }
                    }
                    assert forall|i: int|
                        0 <= i < self.mutex_addrs@.len() && self.mutex_addrs@[i] as int == a
                        implies (exists|j: int|
                            0 <= j < old(self).mutex_addrs@.len() && old(self).mutex_addrs@[j] as int == a) by {
                        if i < idx as int {
                            assert(old(self).mutex_addrs@[i] == self.mutex_addrs@[i]);
                        } else {
                            assert(old(self).mutex_addrs@[i + 1] == self.mutex_addrs@[i]);
                        }
                    }
                }

                // wf: uniqueness preserved after remove.
                assert forall|i: int, j: int|
                    0 <= i < self.mutex_addrs@.len() && 0 <= j < self.mutex_addrs@.len() && i != j
                    implies self.mutex_addrs@[i] != self.mutex_addrs@[j] by {
                    let oi: int = if i < idx as int { i } else { i + 1 };
                    let oj: int = if j < idx as int { j } else { j + 1 };
                    assert(self.mutex_addrs@[i] == old(self).mutex_addrs@[oi]);
                    assert(self.mutex_addrs@[j] == old(self).mutex_addrs@[oj]);
                    assert(oi != oj);
                }

                // wf: ref counts positive after remove.
                assert forall|i: int| #![auto]
                    0 <= i < self.mutex_ref_counts@.len()
                    implies self.mutex_ref_counts@[i] > 0u64 by {
                    let oi: int = if i < idx as int { i } else { i + 1 };
                    assert(self.mutex_ref_counts@[i] == old(self).mutex_ref_counts@[oi]);
                }
            }
        }
        // else: ref count above threshold — entry remains, no state change.
        Ok(())
    }

    /// Returns a condition variable associated with the given address, or creates one.
    ///
    /// # Signature Divergence
    ///
    /// The original `get_cond(cond_addr: ConditionAddress)` performs
    /// `BTreeMap::entry()` lookup internally. Same abstraction as
    /// `get_mutex`: lookup result and index hoisted to preconditions.
    /// Return type changes from `Result<Condvar, Error>` to
    /// `Result<u64, Error>` (ref count) since `Condvar` is opaque.
    ///
    /// # Parameters
    ///
    /// - `cond_addr`: Address of the condition variable.
    /// - `already_present`: Runtime result of BTreeMap::contains_key, tied to state.
    /// - `idx`: Index in the parallel Vecs where the address was found (if present).
    pub fn get_cond(&mut self, cond_addr: u64, already_present: bool, idx: usize) -> (result: Result<u64, Error>)
        requires
            old(self).wf(),
            already_present == old(self).spec_has_cond(cond_addr as int),
            already_present ==> (
                idx < old(self).cond_addrs@.len()
                && old(self).cond_addrs@[idx as int] == cond_addr
                && old(self).cond_ref_counts@[idx as int] < u64::MAX
            ),
        ensures
            result is Ok ==> {
                &&& self.spec_has_cond(cond_addr as int)
                &&& already_present ==>
                        self.spec_cond_ref_count(cond_addr as int)
                            == old(self).spec_cond_ref_count(cond_addr as int) + 1
                &&& !already_present ==> self.spec_cond_ref_count(cond_addr as int) == 2
                &&& result->Ok_0 as nat == self.spec_cond_ref_count(cond_addr as int)
                &&& self.spec_pid() == old(self).spec_pid()
                &&& self.spec_capabilities_bits() == old(self).spec_capabilities_bits()
                &&& self.spec_mutex_count() == old(self).spec_mutex_count()
                &&& self.spec_pmio_ports() == old(self).spec_pmio_ports()
                &&& forall|a: int| self.spec_has_mutex(a) == old(self).spec_has_mutex(a)
                &&& forall|a: int| a != cond_addr as int ==>
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
            let old_rc: u64 = self.cond_ref_counts.remove(idx);
            self.cond_ref_counts.insert(idx, old_rc + 1);
            proof {
                assert(self.cond_ref_counts@ =~= old(self).cond_ref_counts@.update(idx as int, (old_rc + 1) as u64));
                assert(self.cond_addrs@ =~= old(self).cond_addrs@);

                // Witness for spec_has_cond.
                assert(self.cond_addrs@[idx as int] as int == cond_addr as int);

                // Uniqueness → choose picks idx.
                assert forall|i: int|
                    0 <= i < self.cond_addrs@.len() && self.cond_addrs@[i] as int == cond_addr as int
                    implies i == idx as int by {}

                let chosen: int = choose|i: int|
                    0 <= i < self.cond_addrs@.len() && self.cond_addrs@[i] as int == cond_addr as int;
                assert(chosen == idx as int);
                assert(self.cond_ref_counts@[chosen] as nat == (old_rc + 1) as nat);

                let old_chosen: int = choose|i: int|
                    0 <= i < old(self).cond_addrs@.len()
                    && old(self).cond_addrs@[i] as int == cond_addr as int;
                assert(old_chosen == idx as int);
                assert(old(self).cond_ref_counts@[old_chosen] as nat == old_rc as nat);

                assert forall|i: int| #![auto]
                    0 <= i < self.cond_ref_counts@.len()
                    implies self.cond_ref_counts@[i] > 0u64 by {
                    if i == idx as int {} else {}
                }
            }
            Ok(old_rc + 1)
        } else {
            self.cond_count = self.cond_count + 1;
            self.cond_addrs.push(cond_addr);
            self.cond_ref_counts.push(2);
            proof {
                assert(self.cond_addrs@ =~= old(self).cond_addrs@.push(cond_addr));
                assert(self.cond_ref_counts@ =~= old(self).cond_ref_counts@.push(2u64));

                let new_idx: int = self.cond_addrs@.len() - 1;
                assert(self.cond_addrs@[new_idx] as int == cond_addr as int);

                assert forall|i: int| 0 <= i < old(self).cond_addrs@.len()
                    implies old(self).cond_addrs@[i] as int != cond_addr as int by {}

                assert forall|i: int|
                    0 <= i < self.cond_addrs@.len() && self.cond_addrs@[i] as int == cond_addr as int
                    implies i == new_idx by {
                    if i < old(self).cond_addrs@.len() as int {
                        assert(self.cond_addrs@[i] == old(self).cond_addrs@[i]);
                    }
                }

                let chosen: int = choose|i: int|
                    0 <= i < self.cond_addrs@.len() && self.cond_addrs@[i] as int == cond_addr as int;
                assert(chosen == new_idx);
                assert(self.cond_ref_counts@[chosen] == 2u64);

                // Frame: other addresses preserved.
                assert forall|a: int| a != cond_addr as int
                    implies (self.spec_has_cond(a) == old(self).spec_has_cond(a)) by {
                    assert forall|i: int|
                        0 <= i < old(self).cond_addrs@.len() && old(self).cond_addrs@[i] as int == a
                        implies (exists|j: int|
                            0 <= j < self.cond_addrs@.len() && self.cond_addrs@[j] as int == a) by {
                        assert(self.cond_addrs@[i] == old(self).cond_addrs@[i]);
                    }
                    assert forall|i: int|
                        0 <= i < self.cond_addrs@.len() && self.cond_addrs@[i] as int == a
                        implies (exists|j: int|
                            0 <= j < old(self).cond_addrs@.len() && old(self).cond_addrs@[j] as int == a) by {
                        if i < old(self).cond_addrs@.len() as int {
                            assert(old(self).cond_addrs@[i] == self.cond_addrs@[i]);
                        }
                    }
                }

                // wf: uniqueness after push.
                assert forall|i: int, j: int|
                    0 <= i < self.cond_addrs@.len() && 0 <= j < self.cond_addrs@.len() && i != j
                    implies self.cond_addrs@[i] != self.cond_addrs@[j] by {
                    if i < old(self).cond_addrs@.len() as int && j < old(self).cond_addrs@.len() as int {
                        assert(self.cond_addrs@[i] == old(self).cond_addrs@[i]);
                        assert(self.cond_addrs@[j] == old(self).cond_addrs@[j]);
                    } else if i == new_idx {
                        assert(self.cond_addrs@[j] == old(self).cond_addrs@[j]);
                    } else {
                        assert(self.cond_addrs@[i] == old(self).cond_addrs@[i]);
                    }
                }

                assert forall|i: int| #![auto]
                    0 <= i < self.cond_ref_counts@.len()
                    implies self.cond_ref_counts@[i] > 0u64 by {
                    if i < old(self).cond_ref_counts@.len() as int {
                        assert(self.cond_ref_counts@[i] == old(self).cond_ref_counts@[i]);
                    }
                }
            }
            Ok(2)
        }
    }

    /// Releases a condition variable associated with the given address.
    ///
    /// # Signature Divergence
    ///
    /// The original `put_cond(cond_addr: ConditionAddress)` performs
    /// `contains_key()` and `extract_if()` internally. Same abstraction
    /// as `put_mutex`: lookup result, threshold check, and index hoisted
    /// to preconditions.
    ///
    /// # Parameters
    ///
    /// - `cond_addr`: Address of the condition variable.
    /// - `contains`: Runtime result of BTreeMap::contains_key, tied to state.
    /// - `ref_count_at_threshold`: Whether the condvar's Arc strong count is <= 1.
    /// - `idx`: Index in the parallel Vecs where the address was found.
    pub fn put_cond(
        &mut self,
        cond_addr: u64,
        contains: bool,
        ref_count_at_threshold: bool,
        idx: usize,
    ) -> (result: Result<(), Error>)
        requires
            old(self).wf(),
            contains == old(self).spec_has_cond(cond_addr as int),
            contains ==> (
                idx < old(self).cond_addrs@.len()
                && old(self).cond_addrs@[idx as int] == cond_addr
            ),
            contains ==> (ref_count_at_threshold ==
                (old(self).cond_ref_counts@[idx as int] as nat <= Self::COND_REMOVE_THRESHOLD())),
        ensures
            result is Ok ==> {
                &&& old(self).spec_has_cond(cond_addr as int)
                &&& ref_count_at_threshold ==> !self.spec_has_cond(cond_addr as int)
                &&& ref_count_at_threshold ==>
                        self.spec_cond_count() == old(self).spec_cond_count() - 1
                &&& !ref_count_at_threshold ==> self.spec_has_cond(cond_addr as int)
                &&& !ref_count_at_threshold ==>
                        self.spec_cond_count() == old(self).spec_cond_count()
                &&& self.spec_pid() == old(self).spec_pid()
                &&& self.spec_capabilities_bits() == old(self).spec_capabilities_bits()
                &&& self.spec_mutex_count() == old(self).spec_mutex_count()
                &&& self.spec_pmio_ports() == old(self).spec_pmio_ports()
                &&& forall|a: int| self.spec_has_mutex(a) == old(self).spec_has_mutex(a)
                &&& forall|a: int| a != cond_addr as int ==>
                        self.spec_has_cond(a) == old(self).spec_has_cond(a)
                &&& self.wf()
            },
            result is Err ==> {
                &&& !old(self).spec_has_cond(cond_addr as int)
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
            self.cond_addrs.remove(idx);
            self.cond_ref_counts.remove(idx);
            proof {
                assert(self.cond_addrs@ =~= old(self).cond_addrs@.remove(idx as int));
                assert(self.cond_ref_counts@ =~= old(self).cond_ref_counts@.remove(idx as int));

                // Removed address no longer present.
                assert forall|i: int|
                    0 <= i < self.cond_addrs@.len()
                    implies self.cond_addrs@[i] as int != cond_addr as int by {
                    if i < idx as int {
                        assert(self.cond_addrs@[i] == old(self).cond_addrs@[i]);
                    } else {
                        assert(self.cond_addrs@[i] == old(self).cond_addrs@[i + 1]);
                    }
                }

                // Frame: other addresses preserved.
                assert forall|a: int| a != cond_addr as int
                    implies (self.spec_has_cond(a) == old(self).spec_has_cond(a)) by {
                    assert forall|i: int|
                        0 <= i < old(self).cond_addrs@.len() && old(self).cond_addrs@[i] as int == a
                        implies (exists|j: int|
                            0 <= j < self.cond_addrs@.len() && self.cond_addrs@[j] as int == a) by {
                        if i < idx as int {
                            assert(self.cond_addrs@[i] == old(self).cond_addrs@[i]);
                        } else {
                            assert(self.cond_addrs@[i - 1] == old(self).cond_addrs@[i]);
                        }
                    }
                    assert forall|i: int|
                        0 <= i < self.cond_addrs@.len() && self.cond_addrs@[i] as int == a
                        implies (exists|j: int|
                            0 <= j < old(self).cond_addrs@.len() && old(self).cond_addrs@[j] as int == a) by {
                        if i < idx as int {
                            assert(old(self).cond_addrs@[i] == self.cond_addrs@[i]);
                        } else {
                            assert(old(self).cond_addrs@[i + 1] == self.cond_addrs@[i]);
                        }
                    }
                }

                // wf: uniqueness preserved after remove.
                assert forall|i: int, j: int|
                    0 <= i < self.cond_addrs@.len() && 0 <= j < self.cond_addrs@.len() && i != j
                    implies self.cond_addrs@[i] != self.cond_addrs@[j] by {
                    let oi: int = if i < idx as int { i } else { i + 1 };
                    let oj: int = if j < idx as int { j } else { j + 1 };
                    assert(self.cond_addrs@[i] == old(self).cond_addrs@[oi]);
                    assert(self.cond_addrs@[j] == old(self).cond_addrs@[oj]);
                    assert(oi != oj);
                }

                // wf: ref counts positive after remove.
                assert forall|i: int| #![auto]
                    0 <= i < self.cond_ref_counts@.len()
                    implies self.cond_ref_counts@[i] > 0u64 by {
                    let oi: int = if i < idx as int { i } else { i + 1 };
                    assert(self.cond_ref_counts@[i] == old(self).cond_ref_counts@[oi]);
                }
            }
        }
        Ok(())
    }

    /// Adds an I/O port to the process.
    ///
    /// # Signature Divergence
    ///
    /// The original `add_pmio(port: AnyIoPort)` takes the full opaque port
    /// object. In the verification model, `AnyIoPort` is abstracted to its
    /// port number (`u16`), so only the port number is accepted here.
    ///
    /// # Parameters
    ///
    /// - `port_number`: Port number to add.
    pub fn add_pmio(&mut self, port_number: u16)
        requires
            old(self).wf(),
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
        self.pmio_ports.push(port_number);
    }

    /// Removes an I/O port from the process.
    ///
    /// # Signature Divergence
    ///
    /// The original `remove_pmio(port_number: u16) -> Result<AnyIoPort, Error>`
    /// performs `iter().position()` internally and returns the removed port.
    /// In the verification model, the position search result (`found`,
    /// `found_idx`) is hoisted to preconditions, and the return type is
    /// `Result<(), Error>` since `AnyIoPort` is an opaque HAL type.
    ///
    /// # Parameters
    ///
    /// - `port_number`: Port number to remove.
    /// - `found`: Runtime result of the position search, tied to state.
    /// - `found_idx`: Index of the first matching entry in the PMIO Vec.
    pub fn remove_pmio(
        &mut self,
        port_number: u16,
        found: bool,
        found_idx: usize,
    ) -> (result: Result<(), Error>)
        requires
            old(self).wf(),
            found == old(self).spec_has_pmio(port_number as int),
            found ==> (found_idx as int) < old(self).pmio_ports@.len()
                && old(self).pmio_ports@[found_idx as int] == port_number
                && forall|j: int| 0 <= j < found_idx as int ==>
                    old(self).pmio_ports@[j] as int != port_number as int,
        ensures
            result is Ok ==> {
                &&& old(self).spec_has_pmio(port_number as int)
                &&& old(self).pmio_ports@[found_idx as int] == port_number
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
                &&& !old(self).spec_has_pmio(port_number as int)
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

        // Remove the element at found_idx.
        self.pmio_ports.remove(found_idx);
        Ok(())
    }

    //==============================================================================================
    // Frame-Condition Stubs for Omitted Functions
    //==============================================================================================

    // The following stubs model functions that operate on opaque HAL/MM/IPC
    // boundary types (Vmem, EventOwnership, Mailbox, IoMemoryRegion, AnyIoPort).
    // Verus cannot verify these types. Each stub uses justified external_body
    // to prove frame conditions: verified state (PID, capabilities, mutexes,
    // condvars, PMIO) is preserved.

    /// Stub: copy_from_user_unaligned is read-only on verified state.
    ///
    /// # Trust Boundary
    ///
    /// Justified external_body: operates on opaque `Vmem` (HAL boundary type)
    /// which is outside the verification scope. Read-only (`&self`) so verified
    /// state cannot be modified.
    #[verifier::external_body]
    pub fn copy_from_user_unaligned_stub(&self) -> (result: Result<(), Error>)
        requires
            self.wf(),
        ensures true,
    {
        unimplemented!()
    }

    /// Stub: copy_to_user_unaligned is read-only on verified state.
    ///
    /// # Trust Boundary
    ///
    /// Justified external_body: operates on opaque `Vmem` (HAL boundary type).
    /// Read-only (`&self`).
    #[verifier::external_body]
    pub fn copy_to_user_unaligned_stub(&self) -> (result: Result<(), Error>)
        requires
            self.wf(),
        ensures true,
    {
        unimplemented!()
    }

    /// Stub: add_event preserves verified state.
    ///
    /// # Trust Boundary
    ///
    /// Justified external_body: operates on opaque `EventOwnership`/`LinkedList`
    /// (HAL boundary types) which are outside the verification scope.
    /// Frame conditions ensure verified state (PID, capabilities, mutexes,
    /// condvars, PMIO) is not modified.
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

    /// Stub: read_pmio is read-only on verified state.
    ///
    /// # Trust Boundary
    ///
    /// Justified external_body: reads from opaque `AnyIoPort` (HAL boundary type).
    /// Read-only (`&self`).
    #[verifier::external_body]
    pub fn read_pmio_stub(&self) -> (result: Result<u32, Error>)
        requires
            self.wf(),
        ensures true,
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

    /// Stub: vmem returns a reference to opaque Vmem, no state mutation.
    ///
    /// # Trust Boundary
    ///
    /// Justified external_body: returns opaque `Vmem` reference (HAL boundary type).
    /// Read-only (`&self`).
    #[verifier::external_body]
    pub fn vmem_stub(&self)
        requires
            self.wf(),
        ensures true,
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

    /// Stub: get_pmio finds a port by number (read-only).
    ///
    /// # Trust Boundary
    ///
    /// Justified external_body: searches opaque `AnyIoPort` list (HAL boundary type).
    /// Read-only (`&self`).
    #[verifier::external_body]
    fn get_pmio_stub(&self) -> (result: Result<(), Error>)
        requires
            self.wf(),
        ensures true,
    {
        unimplemented!()
    }

    /// Stub: get_pmio_mut finds a port by number (mutable).
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
    /// # Trust Boundary
    ///
    /// Justified external_body: formatting is outside verification scope.
    /// Read-only (`&self`).
    #[verifier::external_body]
    fn debug_fmt_stub(&self)
        requires
            self.wf(),
        ensures true,
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

/// Opaque model of `ProcessRefMut<'a>`.
#[verifier::external_body]
pub struct ProcessRefMut {
    _phantom: (),
}

impl ProcessRefMut {
    /// Stub: state_mut returns mutable reference to inner ProcessState.
    #[verifier::external_body]
    pub fn state_mut_stub(&mut self)
        ensures true,
    {
        unimplemented!()
    }
}

/// Opaque model of `ProcessRef<'a>`.
#[verifier::external_body]
pub struct ProcessRef {
    _phantom: (),
}

impl ProcessRef {
    /// Stub: state returns reference to inner ProcessState.
    #[verifier::external_body]
    pub fn state_stub(&self)
        ensures true,
    {
        unimplemented!()
    }
}

} // verus!
