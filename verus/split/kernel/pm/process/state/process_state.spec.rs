// Copyright(c) The Maintainers of Nanvix.
// Licensed under the MIT License.

// ProcessState Specification.
// This file contains spec functions and View types for the ProcessState type.
//
// ## Verification Model
//
// ProcessState manages per-process state in the kernel. For verification we model:
// - `pid` as a ProcessIdentifier (verified dependency).
// - `capabilities` as a Capabilities (verified dependency).
// - `mutexes` as a ghost `Map<int, nat>` with runtime `mutex_count` counter,
//   modeling `BTreeMap<MutexAddress, Mutex>` with capacity bound `MUTEX_MAX`.
//   The `nat` value represents the Arc strong reference count of the Mutex.
// - `conditions` as a ghost `Map<int, nat>` with runtime `cond_count` counter,
//   modeling `BTreeMap<ConditionAddress, Condvar>` with capacity bound `COND_MAX`.
//   The `nat` value represents the Arc strong reference count of the Condvar.
// - `pmio` as a ghost `Seq<int>` of port numbers, modeling `LinkedList<AnyIoPort>`.
// - `events`, `mailbox`, `mmio`, `vmem` as abstract ghost tokens (opaque boundary types).

use vstd::prelude::*;

verus! {

//==================================================================================================
// View Type
//==================================================================================================

/// Abstract view of a ProcessState.
///
/// Models the logical state of a process: its identity, capabilities,
/// mutex/condvar maps (with reference counts), and I/O port list.
#[verifier::ext_equal]
pub struct ProcessStateView {
    /// Process identifier value.
    pub pid: int,
    /// Capabilities bitfield value.
    pub capabilities_bits: u8,
    /// Number of mutexes in the map.
    pub mutex_count: nat,
    /// Ghost map of mutex addresses to reference counts.
    pub mutex_map: Map<int, nat>,
    /// Number of condition variables in the map.
    pub cond_count: nat,
    /// Ghost map of condvar addresses to reference counts.
    pub cond_map: Map<int, nat>,
    /// Ghost sequence of I/O port numbers.
    pub pmio_ports: Seq<int>,
}

//==================================================================================================
// Spec Functions
//==================================================================================================

impl ProcessState {
    /// Spec function: returns the process identifier value.
    pub open spec fn spec_pid(&self) -> int {
        self.pid.spec_value()
    }

    /// Spec function: returns the capabilities bitfield value.
    pub open spec fn spec_capabilities_bits(&self) -> u8 {
        self.capabilities.spec_bits()
    }

    /// Spec function: returns the number of mutexes.
    pub open spec fn spec_mutex_count(&self) -> nat {
        self.mutex_count as nat
    }

    /// Spec function: returns the number of condition variables.
    pub open spec fn spec_cond_count(&self) -> nat {
        self.cond_count as nat
    }

    /// Spec function: checks whether a mutex address is present.
    pub open spec fn spec_has_mutex(&self, addr: int) -> bool {
        self.ghost_mutexes@.contains_key(addr)
    }

    /// Spec function: returns the reference count for a mutex address.
    pub open spec fn spec_mutex_ref_count(&self, addr: int) -> nat
        recommends self.spec_has_mutex(addr)
    {
        self.ghost_mutexes@[addr]
    }

    /// Spec function: checks whether a condvar address is present.
    pub open spec fn spec_has_cond(&self, addr: int) -> bool {
        self.ghost_conditions@.contains_key(addr)
    }

    /// Spec function: returns the reference count for a condvar address.
    pub open spec fn spec_cond_ref_count(&self, addr: int) -> nat
        recommends self.spec_has_cond(addr)
    {
        self.ghost_conditions@[addr]
    }

    /// Spec function: returns the ghost PMIO port sequence.
    pub open spec fn spec_pmio_ports(&self) -> Seq<int> {
        self.ghost_pmio@
    }

    /// Spec function: checks whether a PMIO port number is present.
    pub open spec fn spec_has_pmio(&self, port_number: int) -> bool {
        exists|i: int| 0 <= i < self.ghost_pmio@.len() && self.ghost_pmio@[i] == port_number
    }

    /// Spec function: returns the number of PMIO ports.
    pub open spec fn spec_pmio_count(&self) -> nat {
        self.ghost_pmio@.len()
    }

    /// Spec function: well-formedness predicate.
    ///
    /// A ProcessState is well-formed when:
    /// - The ghost mutex map is finite and its domain size equals the runtime counter.
    /// - The ghost condvar map is finite and its domain size equals the runtime counter.
    /// - The mutex count does not exceed MUTEX_MAX.
    /// - The condvar count does not exceed COND_MAX.
    /// - The capabilities are well-formed.
    /// - All mutex reference counts are positive.
    /// - All condvar reference counts are positive.
    /// - All PMIO port numbers are valid u16 values (0..=0xFFFF).
    ///
    /// Note: PMIO port uniqueness is NOT enforced, matching the original's
    /// `LinkedList` semantics which allows duplicate port numbers. Adding the
    /// same port twice creates two entries; removing it removes only the first.
    pub open spec fn wf(&self) -> bool {
        &&& self.ghost_mutexes@.dom().finite()
        &&& self.ghost_mutexes@.dom().len() == self.mutex_count as nat
        &&& self.ghost_conditions@.dom().finite()
        &&& self.ghost_conditions@.dom().len() == self.cond_count as nat
        &&& self.mutex_count as nat <= Self::MUTEX_MAX() as nat
        &&& self.cond_count as nat <= Self::COND_MAX() as nat
        &&& self.capabilities.wf()
        &&& forall|addr: int| #![auto] self.ghost_mutexes@.contains_key(addr) ==>
                self.ghost_mutexes@[addr] > 0
        &&& forall|addr: int| #![auto] self.ghost_conditions@.contains_key(addr) ==>
                self.ghost_conditions@[addr] > 0
        // PMIO port numbers must be valid u16 values (0..=0xFFFF),
        // matching the original's `u16` port number type.
        &&& forall|i: int| #![auto] 0 <= i < self.ghost_pmio@.len() ==>
                0 <= self.ghost_pmio@[i] <= 0xFFFF
    }

    /// Spec function: checks if the mutex map is at capacity.
    pub open spec fn spec_mutexes_full(&self) -> bool {
        self.mutex_count as nat >= Self::MUTEX_MAX() as nat
    }

    /// Spec function: checks if the condvar map is at capacity.
    pub open spec fn spec_conditions_full(&self) -> bool {
        self.cond_count as nat >= Self::COND_MAX() as nat
    }

    /// Spec constant: maximum number of mutexes per process.
    /// Matches `mutex_open_max = 32` at `build/kernel_config.toml:40`.
    pub open spec fn MUTEX_MAX() -> usize {
        32usize
    }

    /// Spec constant: maximum number of condition variables per process.
    /// Matches `cond_open_max = 32` at `build/kernel_config.toml:45`.
    pub open spec fn COND_MAX() -> usize {
        32usize
    }

    /// Spec constant: mutex Arc strong count threshold for removal.
    /// In the original, `extract_if` removes when `mutex.reference_count() <= 2`.
    /// The ghost ref count directly models `Arc::strong_count()`: new entries start
    /// at 2 (BTreeMap entry + returned clone), and each `get_mutex` on an existing
    /// entry increments by 1 (modeling `clone()`). When `ref_count <= 2`, only the
    /// BTreeMap entry and the caller's single clone exist (no external holders),
    /// so the entry can be safely removed.
    pub open spec fn MUTEX_REMOVE_THRESHOLD() -> nat {
        2
    }

    /// Spec constant: condvar Arc strong count threshold for removal.
    /// In the original, `extract_if` removes when `cond.reference_count() <= 1`.
    /// The ghost ref count directly models `Arc::strong_count()`: new entries start
    /// at 2 (BTreeMap entry + returned clone). When `ref_count <= 1`, only the
    /// BTreeMap entry's reference exists (the caller has dropped its clone),
    /// so the entry can be safely removed.
    pub open spec fn COND_REMOVE_THRESHOLD() -> nat {
        1
    }
}

//==================================================================================================
// View Implementation
//==================================================================================================

impl View for ProcessState {
    type V = ProcessStateView;

    open spec fn view(&self) -> ProcessStateView {
        ProcessStateView {
            pid: self.pid.spec_value(),
            capabilities_bits: self.capabilities.spec_bits(),
            mutex_count: self.mutex_count as nat,
            mutex_map: self.ghost_mutexes@,
            cond_count: self.cond_count as nat,
            cond_map: self.ghost_conditions@,
            pmio_ports: self.ghost_pmio@,
        }
    }
}

} // verus!
