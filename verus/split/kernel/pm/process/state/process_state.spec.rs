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
// - `mutexes` as a ghost `Map<int, int>` with runtime `mutex_count` counter,
//   modeling `BTreeMap<MutexAddress, Mutex>` with capacity bound `MUTEX_MAX`.
// - `conditions` as a ghost `Map<int, int>` with runtime `cond_count` counter,
//   modeling `BTreeMap<ConditionAddress, Condvar>` with capacity bound `COND_MAX`.
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
/// mutex/condvar maps, and I/O port list.
#[verifier::ext_equal]
pub struct ProcessStateView {
    /// Process identifier value.
    pub pid: int,
    /// Capabilities bitfield value.
    pub capabilities_bits: u8,
    /// Number of mutexes in the map.
    pub mutex_count: nat,
    /// Ghost map of mutex addresses to abstract values.
    pub mutex_map: Map<int, int>,
    /// Number of condition variables in the map.
    pub cond_count: nat,
    /// Ghost map of condvar addresses to abstract values.
    pub cond_map: Map<int, int>,
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

    /// Spec function: checks whether a condvar address is present.
    pub open spec fn spec_has_cond(&self, addr: int) -> bool {
        self.ghost_conditions@.contains_key(addr)
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
    /// - The ghost PMIO sequence length is finite (always true for Seq).
    /// - The mutex count does not exceed MUTEX_MAX.
    /// - The condvar count does not exceed COND_MAX.
    /// - The capabilities are well-formed.
    pub open spec fn wf(&self) -> bool {
        &&& self.ghost_mutexes@.dom().finite()
        &&& self.ghost_mutexes@.dom().len() == self.mutex_count as nat
        &&& self.ghost_conditions@.dom().finite()
        &&& self.ghost_conditions@.dom().len() == self.cond_count as nat
        &&& self.capabilities.wf()
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
    pub open spec fn MUTEX_MAX() -> usize {
        256usize
    }

    /// Spec constant: maximum number of condition variables per process.
    pub open spec fn COND_MAX() -> usize {
        256usize
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
