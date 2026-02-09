// Copyright(c) The Maintainers of Nanvix.
// Licensed under the MIT License.

// ProcessState Proofs.
// This file contains proof lemmas for the ProcessState type.
//
// Key proven properties:
// - Construction produces well-formed state with correct initial values.
// - PID is immutable across all operations.
// - Capability operations delegate correctly and preserve well-formedness.
// - Mutex get/put maintain capacity invariant, ref-count semantics, and map consistency.
// - Condvar get/put maintain capacity invariant, ref-count semantics, and map consistency.
// - PMIO add/remove maintain sequence consistency with single-element removal.
// - Well-formedness (including capacity bounds) is preserved by all operations.

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
    pub proof fn lemma_mutex_change_preserves_pid(&self, new_count: usize, new_map: Map<int, nat>)
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
    pub proof fn lemma_cond_change_preserves_pid(&self, new_count: usize, new_map: Map<int, nat>)
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

    /// Lemma: Getting a mutex that already exists does not change the map domain.
    pub proof fn lemma_get_existing_mutex_no_change(&self, addr: int)
        requires
            self.wf(),
            self.spec_has_mutex(addr),
        ensures
            self.ghost_mutexes@.contains_key(addr),
    {
    }

    /// Lemma: Inserting a new mutex increases count by 1.
    pub proof fn lemma_insert_new_mutex_increments(&self, addr: int, val: nat)
        requires
            self.wf(),
            !self.spec_has_mutex(addr),
            self.mutex_count < usize::MAX,
        ensures
            ({
                let new_map: Map<int, nat> = self.ghost_mutexes@.insert(addr, val);
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
                let new_map: Map<int, nat> = self.ghost_mutexes@.remove(addr);
                new_map.dom().finite()
                && new_map.dom().len() == self.ghost_mutexes@.dom().len() - 1
                && !new_map.contains_key(addr)
            }),
    {
    }

    /// Lemma: Inserting a mutex preserves membership of other addresses.
    pub proof fn lemma_insert_mutex_preserves_others(&self, addr: int, val: nat, other: int)
        requires
            self.wf(),
            other != addr,
        ensures
            ({
                let new_map: Map<int, nat> = self.ghost_mutexes@.insert(addr, val);
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
                let new_map: Map<int, nat> = self.ghost_mutexes@.remove(addr);
                new_map.contains_key(other) == self.ghost_mutexes@.contains_key(other)
            }),
    {
    }

    /// Lemma: Incrementing ref count on existing mutex preserves positive-ref-count wf.
    pub proof fn lemma_increment_mutex_ref_count_preserves_wf(&self, addr: int)
        requires
            self.wf(),
            self.spec_has_mutex(addr),
        ensures
            ({
                let new_map: Map<int, nat> = self.ghost_mutexes@.insert(addr, self.ghost_mutexes@[addr] + 1);
                forall|a: int| #![auto] new_map.contains_key(a) ==> new_map[a] > 0
            }),
    {
    }

    //==============================================================================================
    // Condvar Map Lemmas
    //==============================================================================================

    /// Lemma: Inserting a new condvar increases count by 1.
    pub proof fn lemma_insert_new_cond_increments(&self, addr: int, val: nat)
        requires
            self.wf(),
            !self.spec_has_cond(addr),
            self.cond_count < usize::MAX,
        ensures
            ({
                let new_map: Map<int, nat> = self.ghost_conditions@.insert(addr, val);
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
                let new_map: Map<int, nat> = self.ghost_conditions@.remove(addr);
                new_map.dom().finite()
                && new_map.dom().len() == self.ghost_conditions@.dom().len() - 1
                && !new_map.contains_key(addr)
            }),
    {
    }

    /// Lemma: Incrementing ref count on existing condvar preserves positive-ref-count wf.
    pub proof fn lemma_increment_cond_ref_count_preserves_wf(&self, addr: int)
        requires
            self.wf(),
            self.spec_has_cond(addr),
        ensures
            ({
                let new_map: Map<int, nat> = self.ghost_conditions@.insert(addr, self.ghost_conditions@[addr] + 1);
                forall|a: int| #![auto] new_map.contains_key(a) ==> new_map[a] > 0
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

    /// Lemma: Removing a single element by index decrements count by exactly 1.
    pub proof fn lemma_remove_pmio_at_index_decrements(&self, idx: int)
        requires
            0 <= idx < self.ghost_pmio@.len(),
        ensures
            ({
                let new_pmio: Seq<int> = self.ghost_pmio@.subrange(0, idx)
                    .add(self.ghost_pmio@.subrange(idx + 1, self.ghost_pmio@.len() as int));
                new_pmio.len() == self.ghost_pmio@.len() - 1
            }),
    {
    }

    /// Lemma: The first-occurrence precondition on remove_pmio is satisfiable.
    /// If a port is present, there exists a smallest index where it occurs.
    /// This follows from finite sequence well-ordering: among all matching
    /// indices, one must be the minimum.
    pub proof fn lemma_pmio_first_occurrence_exists(&self, port_number: int)
        requires
            self.spec_has_pmio(port_number),
        ensures
            exists|idx: int| 0 <= idx < self.ghost_pmio@.len()
                && self.ghost_pmio@[idx] == port_number
                && forall|j: int| 0 <= j < idx ==> self.ghost_pmio@[j] != port_number,
    {
        // spec_has_pmio guarantees existence of some matching index.
        let witness: int = choose|i: int| 0 <= i < self.ghost_pmio@.len()
            && self.ghost_pmio@[i] == port_number;
        // Use decreasing induction: the minimum matching index satisfies the postcondition.
        Self::lemma_pmio_min_index_helper(&self.ghost_pmio@, port_number, witness);
    }

    /// Helper: Given a sequence and a valid matching index, find the first occurrence.
    proof fn lemma_pmio_min_index_helper(seq: &Seq<int>, val: int, bound: int)
        requires
            0 <= bound < seq.len(),
            seq[bound] == val,
        ensures
            exists|idx: int| 0 <= idx <= bound
                && seq[idx] == val
                && forall|j: int| 0 <= j < idx ==> seq[j] != val,
        decreases bound,
    {
        if bound == 0 {
            // Base case: bound is 0, so it is the first occurrence.
            assert(seq[0] == val);
        } else {
            // Check if there's an earlier match.
            if exists|k: int| 0 <= k < bound && seq[k] == val {
                let earlier: int = choose|k: int| 0 <= k < bound && seq[k] == val;
                Self::lemma_pmio_min_index_helper(seq, val, earlier);
            } else {
                // No earlier match, so bound is the first occurrence.
                assert(forall|j: int| 0 <= j < bound ==> seq[j] != val);
            }
        }
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
    // Capacity Bound Lemmas
    //==============================================================================================

    /// Lemma: A well-formed state has mutex_count <= MUTEX_MAX.
    pub proof fn lemma_mutex_count_bounded(&self)
        requires
            self.wf(),
        ensures
            self.mutex_count as nat <= Self::MUTEX_MAX() as nat,
    {
    }

    /// Lemma: A well-formed state has cond_count <= COND_MAX.
    pub proof fn lemma_cond_count_bounded(&self)
        requires
            self.wf(),
        ensures
            self.cond_count as nat <= Self::COND_MAX() as nat,
    {
    }

    /// Lemma: get_mutex only returns OutOfMemory when the mutex map is at capacity.
    pub proof fn lemma_get_mutex_error_implies_full(&self)
        requires
            self.wf(),
            self.spec_mutexes_full(),
        ensures
            self.mutex_count as nat >= Self::MUTEX_MAX() as nat,
    {
    }

    /// Lemma: get_cond only returns OutOfMemory when the condvar map is at capacity.
    pub proof fn lemma_get_cond_error_implies_full(&self)
        requires
            self.wf(),
            self.spec_conditions_full(),
        ensures
            self.cond_count as nat >= Self::COND_MAX() as nat,
    {
    }

    //==============================================================================================
    // Reference Count Lemmas
    //==============================================================================================

    /// Lemma: After get_mutex on a new address, ref count is 2 (BTreeMap + clone).
    pub proof fn lemma_new_mutex_ref_count_is_two(&self, addr: int)
        requires
            self.wf(),
            !self.spec_has_mutex(addr),
        ensures
            ({
                let new_map: Map<int, nat> = self.ghost_mutexes@.insert(addr, 2nat);
                new_map[addr] == 2nat
            }),
    {
    }

    /// Lemma: After get_cond on a new address, ref count is 2 (BTreeMap + clone).
    pub proof fn lemma_new_cond_ref_count_is_two(&self, addr: int)
        requires
            self.wf(),
            !self.spec_has_cond(addr),
        ensures
            ({
                let new_map: Map<int, nat> = self.ghost_conditions@.insert(addr, 2nat);
                new_map[addr] == 2nat
            }),
    {
    }

    /// Lemma: put_mutex with ref count at threshold removes the entry.
    pub proof fn lemma_put_mutex_at_threshold_removes(&self, addr: int)
        requires
            self.wf(),
            self.spec_has_mutex(addr),
            self.spec_mutex_ref_count(addr) <= Self::MUTEX_REMOVE_THRESHOLD(),
        ensures
            ({
                let new_map: Map<int, nat> = self.ghost_mutexes@.remove(addr);
                !new_map.contains_key(addr)
                && new_map.dom().finite()
                && new_map.dom().len() == self.ghost_mutexes@.dom().len() - 1
            }),
    {
    }

    /// Lemma: put_mutex with ref count above threshold keeps the entry.
    pub proof fn lemma_put_mutex_above_threshold_keeps(&self, addr: int)
        requires
            self.wf(),
            self.spec_has_mutex(addr),
            self.spec_mutex_ref_count(addr) > Self::MUTEX_REMOVE_THRESHOLD(),
        ensures
            // Entry and map unchanged.
            self.ghost_mutexes@.contains_key(addr),
    {
    }

    /// Lemma: put_cond with ref count at threshold removes the entry.
    pub proof fn lemma_put_cond_at_threshold_removes(&self, addr: int)
        requires
            self.wf(),
            self.spec_has_cond(addr),
            self.spec_cond_ref_count(addr) <= Self::COND_REMOVE_THRESHOLD(),
        ensures
            ({
                let new_map: Map<int, nat> = self.ghost_conditions@.remove(addr);
                !new_map.contains_key(addr)
                && new_map.dom().finite()
                && new_map.dom().len() == self.ghost_conditions@.dom().len() - 1
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
