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
// - `mutexes` as parallel `Vec<u64>` pairs (addresses and ref counts) with
//   `mutex_count` counter, modeling `BTreeMap<MutexAddress, Mutex>` with
//   capacity bound `MUTEX_MAX`. Keys are unique (enforced by wf()).
// - `conditions` as parallel `Vec<u64>` pairs (addresses and ref counts) with
//   `cond_count` counter, modeling `BTreeMap<ConditionAddress, Condvar>` with
//   capacity bound `COND_MAX`. Keys are unique (enforced by wf()).
// - `pmio` as `Vec<u16>` of port numbers, modeling `LinkedList<AnyIoPort>`.
// - `events`, `mailbox`, `mmio`, `vmem` as abstract tokens (opaque boundary types).

verus! {

//==================================================================================================
// View Type
//==================================================================================================

/// Abstract view of a ProcessState.
///
/// Models the logical state of a process: its identity, capabilities,
/// mutex/condvar parallel arrays (with reference counts), and I/O port list.
#[verifier::ext_equal]
pub struct ProcessStateView {
    /// Process identifier value.
    pub pid: int,
    /// Capabilities bitfield value (implementation level).
    pub capabilities_bits: u8,
    /// Set of granted capabilities (abstract level).
    pub capabilities_granted: Set<Capability>,
    /// Number of mutexes in the map.
    pub mutex_count: nat,
    /// Mutex address keys.
    pub mutex_addrs: Seq<u64>,
    /// Mutex reference counts (parallel to mutex_addrs).
    pub mutex_ref_counts: Seq<u64>,
    /// Number of condition variables in the map.
    pub cond_count: nat,
    /// Condvar address keys.
    pub cond_addrs: Seq<u64>,
    /// Condvar reference counts (parallel to cond_addrs).
    pub cond_ref_counts: Seq<u64>,
    /// I/O port numbers.
    pub pmio_ports: Seq<u16>,
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

    /// Spec function: returns the set of granted capabilities (abstract level).
    pub open spec fn spec_capabilities_granted(&self) -> Set<Capability> {
        self.capabilities.spec_granted()
    }

    /// Spec function: checks whether a capability is granted (abstract level).
    ///
    /// # Description
    ///
    /// Downstream modules should prefer this over `spec_capabilities_bits`.
    pub open spec fn spec_has_capability(&self, cap: Capability) -> bool {
        self.capabilities.spec_set_contains(cap)
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
        exists|i: int| 0 <= i < self.mutex_addrs@.len() && self.mutex_addrs@[i] as int == addr
    }

    /// Spec function: returns the reference count for a mutex address.
    /// Requires uniqueness (from wf()) so `choose` is deterministic.
    pub open spec fn spec_mutex_ref_count(&self, addr: int) -> nat
        recommends self.spec_has_mutex(addr)
    {
        let i = choose|i: int| 0 <= i < self.mutex_addrs@.len() && self.mutex_addrs@[i] as int == addr;
        self.mutex_ref_counts@[i] as nat
    }

    /// Spec function: checks whether a condvar address is present.
    pub open spec fn spec_has_cond(&self, addr: int) -> bool {
        exists|i: int| 0 <= i < self.cond_addrs@.len() && self.cond_addrs@[i] as int == addr
    }

    /// Spec function: returns the reference count for a condvar address.
    pub open spec fn spec_cond_ref_count(&self, addr: int) -> nat
        recommends self.spec_has_cond(addr)
    {
        let i = choose|i: int| 0 <= i < self.cond_addrs@.len() && self.cond_addrs@[i] as int == addr;
        self.cond_ref_counts@[i] as nat
    }

    /// Spec function: returns the PMIO port sequence.
    pub open spec fn spec_pmio_ports(&self) -> Seq<u16> {
        self.pmio_ports@
    }

    /// Spec function: checks whether a PMIO port number is present.
    pub open spec fn spec_has_pmio(&self, port_number: int) -> bool {
        exists|i: int| 0 <= i < self.pmio_ports@.len() && self.pmio_ports@[i] as int == port_number
    }

    /// Spec function: returns the number of PMIO ports.
    pub open spec fn spec_pmio_count(&self) -> nat {
        self.pmio_ports@.len()
    }

    /// Spec function: well-formedness predicate.
    ///
    /// A ProcessState is well-formed when:
    /// - Parallel Vec lengths match and equal the runtime counter (mutexes, condvars).
    /// - Capacity bounds are respected.
    /// - Capabilities are well-formed.
    /// - Mutex and condvar keys are unique within their respective Vecs.
    /// - All reference counts are positive.
    ///
    /// Note: PMIO port uniqueness is NOT enforced, matching the original's
    /// `LinkedList` semantics which allows duplicate port numbers.
    ///
    /// NOTE: `wf()` is `open` rather than the guideline's `closed` because
    /// numerous proof lemmas and exec proof blocks rely on the SMT solver
    /// automatically unfolding the definition to establish individual
    /// conjuncts. Making it `closed` would require 20+ `reveal()` calls
    /// across exec code and proof lemmas with no semantic benefit for this
    /// internal kernel type that is not directly used by downstream modules.
    pub open spec fn wf(&self) -> bool {
        // Parallel Vec invariants for mutexes.
        &&& self.mutex_addrs@.len() == self.mutex_ref_counts@.len()
        &&& self.mutex_addrs@.len() == self.mutex_count as nat
        // Parallel Vec invariants for condvars.
        &&& self.cond_addrs@.len() == self.cond_ref_counts@.len()
        &&& self.cond_addrs@.len() == self.cond_count as nat
        // Capacity bounds.
        &&& self.mutex_count as nat <= Self::MUTEX_MAX() as nat
        &&& self.cond_count as nat <= Self::COND_MAX() as nat
        // Capabilities well-formedness.
        &&& self.capabilities.wf()
        // Mutex keys are unique.
        &&& forall|i: int, j: int|
                0 <= i < self.mutex_addrs@.len() && 0 <= j < self.mutex_addrs@.len() && i != j
                ==> self.mutex_addrs@[i] != self.mutex_addrs@[j]
        // Condvar keys are unique.
        &&& forall|i: int, j: int|
                0 <= i < self.cond_addrs@.len() && 0 <= j < self.cond_addrs@.len() && i != j
                ==> self.cond_addrs@[i] != self.cond_addrs@[j]
        // All mutex ref counts are positive.
        &&& forall|i: int| #![auto] 0 <= i < self.mutex_ref_counts@.len() ==>
                self.mutex_ref_counts@[i] > 0
        // All condvar ref counts are positive.
        &&& forall|i: int| #![auto] 0 <= i < self.cond_ref_counts@.len() ==>
                self.cond_ref_counts@[i] > 0
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
    pub open spec fn MUTEX_REMOVE_THRESHOLD() -> nat {
        2
    }

    /// Spec constant: condvar Arc strong count threshold for removal.
    pub open spec fn COND_REMOVE_THRESHOLD() -> nat {
        1
    }
}

//==================================================================================================
// View Implementation
//==================================================================================================

impl View for ProcessState {
    type V = ProcessStateView;

    // NOTE: view() must remain `open` because the Verus `View` trait requires
    // implementations to use `open spec fn`. This is a justified exception to
    // the guideline that view() should be `pub closed spec fn`.
    open spec fn view(&self) -> ProcessStateView {
        ProcessStateView {
            pid: self.pid.spec_value(),
            capabilities_bits: self.capabilities.spec_bits(),
            capabilities_granted: self.capabilities.spec_granted(),
            mutex_count: self.mutex_count as nat,
            mutex_addrs: self.mutex_addrs@,
            mutex_ref_counts: self.mutex_ref_counts@,
            cond_count: self.cond_count as nat,
            cond_addrs: self.cond_addrs@,
            cond_ref_counts: self.cond_ref_counts@,
            pmio_ports: self.pmio_ports@,
        }
    }
}

} // verus!
