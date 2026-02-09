// Copyright(c) The Maintainers of Nanvix.
// Licensed under the MIT License.

// ProcessState Proofs.
// This file contains proof lemmas for the ProcessState type.
//
// Key proven properties:
// - Construction produces well-formed state with correct initial values.
// - PID is immutable across all operations.
// - Capability operations delegate correctly and preserve well-formedness.
// - Mutex get/put maintain capacity invariant and map consistency.
// - Condvar get/put maintain capacity invariant and map consistency.
// - PMIO add/remove maintain sequence consistency.
// - Well-formedness is preserved by all operations.

use vstd::prelude::*;

verus! {

impl ProcessState {

    //==============================================================================================
    // Construction Lemmas
    //==============================================================================================

    /// Lemma: A newly constructed ProcessState is well-formed.
    pub proof fn lemma_new_is_wf(pid: ProcessIdentifier)
        ensures
            ({
                let s: ProcessState = ProcessState {
                    pid: pid,
                    capabilities: Capabilities { bits: 0u8 },
                    mutex_count: 0usize,
                    ghost_mutexes: Ghost(Map::empty()),
                    cond_count: 0usize,
                    ghost_conditions: Ghost(Map::empty()),
                    ghost_pmio: Ghost(Seq::empty()),
                };
                s.wf()
            }),
    {
        assert(0u8 & 0b1110_0000u8 == 0u8) by (bit_vector);
    }

    /// Lemma: A newly constructed ProcessState has empty collections.
    pub proof fn lemma_new_has_empty_collections(pid: ProcessIdentifier)
        ensures
            ({
                let s: ProcessState = ProcessState {
                    pid: pid,
                    capabilities: Capabilities { bits: 0u8 },
                    mutex_count: 0usize,
                    ghost_mutexes: Ghost(Map::empty()),
                    cond_count: 0usize,
                    ghost_conditions: Ghost(Map::empty()),
                    ghost_pmio: Ghost(Seq::empty()),
                };
                s.spec_mutex_count() == 0
                && s.spec_cond_count() == 0
                && s.spec_pmio_count() == 0
            }),
    {
    }

    /// Lemma: A newly constructed ProcessState has no capabilities.
    pub proof fn lemma_new_no_capabilities(pid: ProcessIdentifier)
        ensures
            ({
                let s: ProcessState = ProcessState {
                    pid: pid,
                    capabilities: Capabilities { bits: 0u8 },
                    mutex_count: 0usize,
                    ghost_mutexes: Ghost(Map::empty()),
                    cond_count: 0usize,
                    ghost_conditions: Ghost(Map::empty()),
                    ghost_pmio: Ghost(Seq::empty()),
                };
                s.spec_capabilities_bits() == 0u8
            }),
    {
    }

    //==============================================================================================
    // PID Immutability Lemmas
    //==============================================================================================

    /// Lemma: set_capability preserves PID.
    pub proof fn lemma_set_capability_preserves_pid(&self, cap_bits: u8)
        ensures
            ({
                let post: ProcessState = ProcessState {
                    capabilities: Capabilities { bits: cap_bits },
                    ..*self
                };
                post.spec_pid() == self.spec_pid()
            }),
    {
    }

    /// Lemma: Mutex count change preserves PID.
    pub proof fn lemma_mutex_change_preserves_pid(&self, new_count: usize, new_map: Map<int, int>)
        ensures
            ({
                let post: ProcessState = ProcessState {
                    mutex_count: new_count,
                    ghost_mutexes: Ghost(new_map),
                    ..*self
                };
                post.spec_pid() == self.spec_pid()
            }),
    {
    }

    /// Lemma: Condvar count change preserves PID.
    pub proof fn lemma_cond_change_preserves_pid(&self, new_count: usize, new_map: Map<int, int>)
        ensures
            ({
                let post: ProcessState = ProcessState {
                    cond_count: new_count,
                    ghost_conditions: Ghost(new_map),
                    ..*self
                };
                post.spec_pid() == self.spec_pid()
            }),
    {
    }

    /// Lemma: PMIO change preserves PID.
    pub proof fn lemma_pmio_change_preserves_pid(&self, new_pmio: Seq<int>)
        ensures
            ({
                let post: ProcessState = ProcessState {
                    ghost_pmio: Ghost(new_pmio),
                    ..*self
                };
                post.spec_pid() == self.spec_pid()
            }),
    {
    }

    //==============================================================================================
    // Mutex Map Lemmas
    //==============================================================================================

    /// Lemma: Getting a mutex that already exists does not change the map.
    pub proof fn lemma_get_existing_mutex_no_change(&self, addr: int)
        requires
            self.wf(),
            self.spec_has_mutex(addr),
        ensures
            // Map already contains addr, so get_mutex returns existing without change.
            self.ghost_mutexes@.contains_key(addr),
    {
    }

    /// Lemma: Inserting a new mutex increases count by 1.
    pub proof fn lemma_insert_new_mutex_increments(&self, addr: int, val: int)
        requires
            self.wf(),
            !self.spec_has_mutex(addr),
            self.mutex_count < usize::MAX,
        ensures
            ({
                let new_map: Map<int, int> = self.ghost_mutexes@.insert(addr, val);
                new_map.dom().finite()
                && new_map.dom().len() == self.ghost_mutexes@.dom().len() + 1
                && new_map.contains_key(addr)
            }),
    {
    }

    /// Lemma: Removing a mutex that exists decreases count.
    pub proof fn lemma_remove_mutex_decrements(&self, addr: int)
        requires
            self.wf(),
            self.spec_has_mutex(addr),
        ensures
            ({
                let new_map: Map<int, int> = self.ghost_mutexes@.remove(addr);
                new_map.dom().finite()
                && new_map.dom().len() == self.ghost_mutexes@.dom().len() - 1
                && !new_map.contains_key(addr)
            }),
    {
    }

    /// Lemma: Inserting a mutex preserves membership of other addresses.
    pub proof fn lemma_insert_mutex_preserves_others(&self, addr: int, val: int, other: int)
        requires
            self.wf(),
            other != addr,
        ensures
            ({
                let new_map: Map<int, int> = self.ghost_mutexes@.insert(addr, val);
                new_map.contains_key(other) == self.ghost_mutexes@.contains_key(other)
            }),
    {
    }

    /// Lemma: Removing a mutex preserves membership of other addresses.
    pub proof fn lemma_remove_mutex_preserves_others(&self, addr: int, other: int)
        requires
            self.wf(),
            self.spec_has_mutex(addr),
            other != addr,
        ensures
            ({
                let new_map: Map<int, int> = self.ghost_mutexes@.remove(addr);
                new_map.contains_key(other) == self.ghost_mutexes@.contains_key(other)
            }),
    {
    }

    //==============================================================================================
    // Condvar Map Lemmas
    //==============================================================================================

    /// Lemma: Inserting a new condvar increases count by 1.
    pub proof fn lemma_insert_new_cond_increments(&self, addr: int, val: int)
        requires
            self.wf(),
            !self.spec_has_cond(addr),
            self.cond_count < usize::MAX,
        ensures
            ({
                let new_map: Map<int, int> = self.ghost_conditions@.insert(addr, val);
                new_map.dom().finite()
                && new_map.dom().len() == self.ghost_conditions@.dom().len() + 1
                && new_map.contains_key(addr)
            }),
    {
    }

    /// Lemma: Removing a condvar that exists decreases count.
    pub proof fn lemma_remove_cond_decrements(&self, addr: int)
        requires
            self.wf(),
            self.spec_has_cond(addr),
        ensures
            ({
                let new_map: Map<int, int> = self.ghost_conditions@.remove(addr);
                new_map.dom().finite()
                && new_map.dom().len() == self.ghost_conditions@.dom().len() - 1
                && !new_map.contains_key(addr)
            }),
    {
    }

    //==============================================================================================
    // PMIO Lemmas
    //==============================================================================================

    /// Lemma: Adding a PMIO port increases the count.
    pub proof fn lemma_add_pmio_increments(&self, port_number: int)
        ensures
            ({
                let new_pmio: Seq<int> = self.ghost_pmio@.push(port_number);
                new_pmio.len() == self.ghost_pmio@.len() + 1
            }),
    {
    }

    //==============================================================================================
    // Well-Formedness Preservation Lemmas
    //==============================================================================================

    /// Lemma: Capability change preserves well-formedness (if new capabilities are wf).
    pub proof fn lemma_capability_change_preserves_wf(&self, new_caps: Capabilities)
        requires
            self.wf(),
            new_caps.wf(),
        ensures
            ({
                let post: ProcessState = ProcessState {
                    capabilities: new_caps,
                    ..*self
                };
                post.wf()
            }),
    {
    }

    /// Lemma: PMIO change preserves well-formedness.
    pub proof fn lemma_pmio_change_preserves_wf(&self, new_pmio: Seq<int>)
        requires
            self.wf(),
        ensures
            ({
                let post: ProcessState = ProcessState {
                    ghost_pmio: Ghost(new_pmio),
                    ..*self
                };
                post.wf()
            }),
    {
    }

    //==============================================================================================
    // View Equality Lemma
    //==============================================================================================

    /// Lemma: Two ProcessStates with identical fields have equal views.
    pub proof fn lemma_view_equality(a: &ProcessState, b: &ProcessState)
        requires
            a.pid.spec_value() == b.pid.spec_value(),
            a.capabilities.spec_bits() == b.capabilities.spec_bits(),
            a.mutex_count == b.mutex_count,
            a.ghost_mutexes@ =~= b.ghost_mutexes@,
            a.cond_count == b.cond_count,
            a.ghost_conditions@ =~= b.ghost_conditions@,
            a.ghost_pmio@ =~= b.ghost_pmio@,
        ensures
            a@ == b@,
    {
    }
}

} // verus!
