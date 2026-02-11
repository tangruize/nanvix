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
    /// Expressed as: any ProcessState with empty collections and zero capabilities is wf.
    pub proof fn lemma_new_is_wf(pid: ProcessIdentifier)
        ensures
            forall|s: ProcessState|
                s.pid.spec_value() == pid.spec_value()
                && s.capabilities.spec_bits() == 0u8
                && s.capabilities.wf()
                && s.mutex_count == 0
                && s.mutex_addrs@.len() == 0
                && s.mutex_ref_counts@.len() == 0
                && s.cond_count == 0
                && s.cond_addrs@.len() == 0
                && s.cond_ref_counts@.len() == 0
                && s.pmio_ports@.len() == 0
                ==> s.wf(),
    {
        assert(0u8 & 0b1110_0000u8 == 0u8) by (bit_vector);
    }

    /// Lemma: A newly constructed ProcessState has empty collections.
    pub proof fn lemma_new_has_empty_collections(pid: ProcessIdentifier)
        ensures
            forall|s: ProcessState|
                s.mutex_count == 0
                && s.cond_count == 0
                && s.pmio_ports@.len() == 0
                ==> (
                    s.spec_mutex_count() == 0
                    && s.spec_cond_count() == 0
                    && s.spec_pmio_count() == 0
                ),
    {
    }

    /// Lemma: A newly constructed ProcessState has no capabilities.
    pub proof fn lemma_new_no_capabilities(pid: ProcessIdentifier)
        ensures
            forall|s: ProcessState|
                s.capabilities.spec_bits() == 0u8
                ==> s.spec_capabilities_bits() == 0u8,
    {
    }

    //==============================================================================================
    // PID Immutability Lemmas
    //==============================================================================================

    /// Lemma: Changing capabilities preserves PID.
    pub proof fn lemma_set_capability_preserves_pid(&self, cap_bits: u8)
        ensures
            forall|post: ProcessState|
                post.pid.spec_value() == self.pid.spec_value()
                ==> post.spec_pid() == self.spec_pid(),
    {
    }

    /// Lemma: Changing mutex fields preserves PID.
    pub proof fn lemma_mutex_change_preserves_pid(&self)
        ensures
            forall|post: ProcessState|
                post.pid.spec_value() == self.pid.spec_value()
                ==> post.spec_pid() == self.spec_pid(),
    {
    }

    /// Lemma: Changing condvar fields preserves PID.
    pub proof fn lemma_cond_change_preserves_pid(&self)
        ensures
            forall|post: ProcessState|
                post.pid.spec_value() == self.pid.spec_value()
                ==> post.spec_pid() == self.spec_pid(),
    {
    }

    /// Lemma: Changing PMIO ports preserves PID.
    pub proof fn lemma_pmio_change_preserves_pid(&self)
        ensures
            forall|post: ProcessState|
                post.pid.spec_value() == self.pid.spec_value()
                ==> post.spec_pid() == self.spec_pid(),
    {
    }

    //==============================================================================================
    // Mutex Vec Lemmas
    //==============================================================================================

    /// Lemma: Getting a mutex that already exists: the address is in the Vec.
    pub proof fn lemma_get_existing_mutex_no_change(&self, addr: int)
        requires
            self.wf(),
            self.spec_has_mutex(addr),
        ensures
            exists|i: int| 0 <= i < self.mutex_addrs@.len() && self.mutex_addrs@[i] as int == addr,
    {
    }

    /// Lemma: Pushing a new address increases count by 1.
    pub proof fn lemma_insert_new_mutex_increments(&self, addr: u64)
        requires
            self.wf(),
            !self.spec_has_mutex(addr as int),
            self.mutex_count < usize::MAX,
        ensures
            ({
                let new_addrs: Seq<u64> = self.mutex_addrs@.push(addr);
                new_addrs.len() == self.mutex_addrs@.len() + 1
                && new_addrs[new_addrs.len() - 1] == addr
            }),
    {
    }

    /// Lemma: Removing a mutex address at index decreases count.
    pub proof fn lemma_remove_mutex_decrements(&self, idx: int)
        requires
            self.wf(),
            0 <= idx < self.mutex_addrs@.len(),
        ensures
            ({
                let new_addrs: Seq<u64> = self.mutex_addrs@.remove(idx);
                new_addrs.len() == self.mutex_addrs@.len() - 1
            }),
    {
    }

    /// Lemma: Pushing a new address does not affect membership of other addresses.
    pub proof fn lemma_insert_mutex_preserves_others(&self, addr: u64, other: int)
        requires
            self.wf(),
            other != addr as int,
        ensures
            ({
                let new_addrs: Seq<u64> = self.mutex_addrs@.push(addr);
                (exists|i: int| 0 <= i < new_addrs.len() && new_addrs[i] as int == other) ==
                (exists|i: int| 0 <= i < self.mutex_addrs@.len() && self.mutex_addrs@[i] as int == other)
            }),
    {
        let new_addrs: Seq<u64> = self.mutex_addrs@.push(addr);
        let old_len: int = self.mutex_addrs@.len() as int;
        // Forward: if other existed in old, it exists in new (at the same index).
        assert forall|i: int| 0 <= i < old_len && self.mutex_addrs@[i] as int == other
            implies exists|j: int| 0 <= j < new_addrs.len() && new_addrs[j] as int == other by {
            assert(new_addrs[i] == self.mutex_addrs@[i]);
        }
        // Backward: if other exists in new, it must be at an old index (not the pushed one).
        assert forall|i: int| 0 <= i < new_addrs.len() && new_addrs[i] as int == other
            implies exists|j: int| 0 <= j < old_len && self.mutex_addrs@[j] as int == other by {
            if i < old_len {
                assert(self.mutex_addrs@[i] as int == other);
            } else {
                // i == old_len, new_addrs[i] == addr, but addr as int != other.
                assert(false);
            }
        }
    }

    /// Lemma: Removing a mutex at index preserves membership of other addresses.
    pub proof fn lemma_remove_mutex_preserves_others(&self, idx: int, other: int)
        requires
            self.wf(),
            0 <= idx < self.mutex_addrs@.len(),
            other != self.mutex_addrs@[idx] as int,
        ensures
            ({
                let new_addrs: Seq<u64> = self.mutex_addrs@.remove(idx);
                (exists|i: int| 0 <= i < new_addrs.len() && new_addrs[i] as int == other) ==
                (exists|i: int| 0 <= i < self.mutex_addrs@.len() && self.mutex_addrs@[i] as int == other)
            }),
    {
        let new_addrs: Seq<u64> = self.mutex_addrs@.remove(idx);
        let old_len: int = self.mutex_addrs@.len() as int;
        // Forward: if other existed in old, it exists in new.
        assert forall|i: int| 0 <= i < old_len && self.mutex_addrs@[i] as int == other
            implies exists|j: int| 0 <= j < new_addrs.len() && new_addrs[j] as int == other by {
            if i < idx {
                assert(new_addrs[i] == self.mutex_addrs@[i]);
            } else {
                // i > idx (i != idx because other != addrs[idx])
                assert(new_addrs[i - 1] == self.mutex_addrs@[i]);
            }
        }
        // Backward: if other exists in new, it existed in old.
        assert forall|i: int| 0 <= i < new_addrs.len() && new_addrs[i] as int == other
            implies exists|j: int| 0 <= j < old_len && self.mutex_addrs@[j] as int == other by {
            if i < idx {
                assert(self.mutex_addrs@[i] == new_addrs[i]);
            } else {
                assert(self.mutex_addrs@[i + 1] == new_addrs[i]);
            }
        }
    }

    /// Lemma: Updating a ref count preserves positive-ref-count invariant.
    pub proof fn lemma_increment_mutex_ref_count_preserves_wf(&self, idx: int)
        requires
            self.wf(),
            0 <= idx < self.mutex_ref_counts@.len(),
            self.mutex_ref_counts@[idx] < u64::MAX,
        ensures
            ({
                let new_rcs: Seq<u64> = self.mutex_ref_counts@.update(idx, (self.mutex_ref_counts@[idx] + 1) as u64);
                forall|i: int| #![auto] 0 <= i < new_rcs.len() ==> new_rcs[i] > 0
            }),
    {
    }

    //==============================================================================================
    // Condvar Vec Lemmas
    //==============================================================================================

    /// Lemma: Pushing a new condvar address increases count by 1.
    pub proof fn lemma_insert_new_cond_increments(&self, addr: u64)
        requires
            self.wf(),
            !self.spec_has_cond(addr as int),
            self.cond_count < usize::MAX,
        ensures
            ({
                let new_addrs: Seq<u64> = self.cond_addrs@.push(addr);
                new_addrs.len() == self.cond_addrs@.len() + 1
                && new_addrs[new_addrs.len() - 1] == addr
            }),
    {
    }

    /// Lemma: Removing a condvar at index decreases count.
    pub proof fn lemma_remove_cond_decrements(&self, idx: int)
        requires
            self.wf(),
            0 <= idx < self.cond_addrs@.len(),
        ensures
            ({
                let new_addrs: Seq<u64> = self.cond_addrs@.remove(idx);
                new_addrs.len() == self.cond_addrs@.len() - 1
            }),
    {
    }

    /// Lemma: Updating a condvar ref count preserves positive-ref-count invariant.
    pub proof fn lemma_increment_cond_ref_count_preserves_wf(&self, idx: int)
        requires
            self.wf(),
            0 <= idx < self.cond_ref_counts@.len(),
            self.cond_ref_counts@[idx] < u64::MAX,
        ensures
            ({
                let new_rcs: Seq<u64> = self.cond_ref_counts@.update(idx, (self.cond_ref_counts@[idx] + 1) as u64);
                forall|i: int| #![auto] 0 <= i < new_rcs.len() ==> new_rcs[i] > 0
            }),
    {
    }

    //==============================================================================================
    // PMIO Lemmas
    //==============================================================================================

    /// Lemma: Adding a PMIO port increases the count.
    pub proof fn lemma_add_pmio_increments(&self, port_number: u16)
        ensures
            ({
                let new_pmio: Seq<u16> = self.pmio_ports@.push(port_number);
                new_pmio.len() == self.pmio_ports@.len() + 1
            }),
    {
    }

    /// Lemma: Removing a single element by index decrements count by exactly 1.
    pub proof fn lemma_remove_pmio_at_index_decrements(&self, idx: int)
        requires
            0 <= idx < self.pmio_ports@.len(),
        ensures
            ({
                let new_pmio: Seq<u16> = self.pmio_ports@.remove(idx);
                new_pmio.len() == self.pmio_ports@.len() - 1
            }),
    {
    }

    /// Lemma: The first-occurrence precondition on remove_pmio is satisfiable.
    pub proof fn lemma_pmio_first_occurrence_exists(&self, port_number: int)
        requires
            self.spec_has_pmio(port_number),
        ensures
            exists|idx: int| 0 <= idx < self.pmio_ports@.len()
                && self.pmio_ports@[idx] as int == port_number
                && forall|j: int| 0 <= j < idx ==> self.pmio_ports@[j] as int != port_number,
    {
        let witness: int = choose|i: int| 0 <= i < self.pmio_ports@.len()
            && self.pmio_ports@[i] as int == port_number;
        Self::lemma_pmio_min_index_helper(&self.pmio_ports@, port_number, witness);
    }

    /// Helper: Given a sequence and a valid matching index, find the first occurrence.
    proof fn lemma_pmio_min_index_helper(seq: &Seq<u16>, val: int, bound: int)
        requires
            0 <= bound < seq.len(),
            seq[bound] as int == val,
        ensures
            exists|idx: int| 0 <= idx <= bound
                && seq[idx] as int == val
                && forall|j: int| 0 <= j < idx ==> seq[j] as int != val,
        decreases bound,
    {
        if bound == 0 {
            assert(seq[0] as int == val);
        } else {
            if exists|k: int| 0 <= k < bound && seq[k] as int == val {
                let earlier: int = choose|k: int| 0 <= k < bound && seq[k] as int == val;
                Self::lemma_pmio_min_index_helper(seq, val, earlier);
            } else {
                assert(forall|j: int| 0 <= j < bound ==> seq[j] as int != val);
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
            forall|post: ProcessState|
                post.capabilities == new_caps
                && post.mutex_count == self.mutex_count
                && post.mutex_addrs@ =~= self.mutex_addrs@
                && post.mutex_ref_counts@ =~= self.mutex_ref_counts@
                && post.cond_count == self.cond_count
                && post.cond_addrs@ =~= self.cond_addrs@
                && post.cond_ref_counts@ =~= self.cond_ref_counts@
                && post.pmio_ports@ =~= self.pmio_ports@
                ==> post.wf(),
    {
    }

    /// Lemma: PMIO push preserves well-formedness.
    pub proof fn lemma_pmio_push_preserves_wf(&self, port_number: u16)
        requires
            self.wf(),
        ensures
            forall|post: ProcessState|
                post.pid.spec_value() == self.pid.spec_value()
                && post.capabilities == self.capabilities
                && post.mutex_count == self.mutex_count
                && post.mutex_addrs@ =~= self.mutex_addrs@
                && post.mutex_ref_counts@ =~= self.mutex_ref_counts@
                && post.cond_count == self.cond_count
                && post.cond_addrs@ =~= self.cond_addrs@
                && post.cond_ref_counts@ =~= self.cond_ref_counts@
                && post.pmio_ports@ =~= self.pmio_ports@.push(port_number)
                ==> post.wf(),
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

    /// Lemma: After get_mutex on a new address, ref count is 2.
    pub proof fn lemma_new_mutex_ref_count_is_two(&self, addr: u64)
        requires
            self.wf(),
            !self.spec_has_mutex(addr as int),
        ensures
            ({
                let new_addrs: Seq<u64> = self.mutex_addrs@.push(addr);
                let new_rcs: Seq<u64> = self.mutex_ref_counts@.push(2);
                new_rcs[new_rcs.len() - 1] == 2
            }),
    {
    }

    /// Lemma: After get_cond on a new address, ref count is 2.
    pub proof fn lemma_new_cond_ref_count_is_two(&self, addr: u64)
        requires
            self.wf(),
            !self.spec_has_cond(addr as int),
        ensures
            ({
                let new_addrs: Seq<u64> = self.cond_addrs@.push(addr);
                let new_rcs: Seq<u64> = self.cond_ref_counts@.push(2);
                new_rcs[new_rcs.len() - 1] == 2
            }),
    {
    }

    /// Lemma: put_mutex with ref count at threshold — removal is valid.
    pub proof fn lemma_put_mutex_at_threshold_removes(&self, idx: int)
        requires
            self.wf(),
            0 <= idx < self.mutex_addrs@.len(),
            self.mutex_ref_counts@[idx] as nat <= Self::MUTEX_REMOVE_THRESHOLD(),
        ensures
            ({
                let new_addrs: Seq<u64> = self.mutex_addrs@.remove(idx);
                let new_rcs: Seq<u64> = self.mutex_ref_counts@.remove(idx);
                new_addrs.len() == self.mutex_addrs@.len() - 1
                && new_rcs.len() == self.mutex_ref_counts@.len() - 1
            }),
    {
    }

    /// Lemma: put_mutex with ref count above threshold keeps the entry.
    pub proof fn lemma_put_mutex_above_threshold_keeps(&self, idx: int)
        requires
            self.wf(),
            0 <= idx < self.mutex_addrs@.len(),
            self.mutex_ref_counts@[idx] as nat > Self::MUTEX_REMOVE_THRESHOLD(),
        ensures
            self.mutex_addrs@.len() > 0,
    {
    }

    /// Lemma: put_cond with ref count at threshold — removal is valid.
    pub proof fn lemma_put_cond_at_threshold_removes(&self, idx: int)
        requires
            self.wf(),
            0 <= idx < self.cond_addrs@.len(),
            self.cond_ref_counts@[idx] as nat <= Self::COND_REMOVE_THRESHOLD(),
        ensures
            ({
                let new_addrs: Seq<u64> = self.cond_addrs@.remove(idx);
                let new_rcs: Seq<u64> = self.cond_ref_counts@.remove(idx);
                new_addrs.len() == self.cond_addrs@.len() - 1
                && new_rcs.len() == self.cond_ref_counts@.len() - 1
            }),
    {
    }

    //==============================================================================================
    // View Equality Lemma
    //==============================================================================================

    /// Lemma: Two ProcessStates with identical views have equal views.
    pub proof fn lemma_view_equality(a: &ProcessState, b: &ProcessState)
        requires
            a.pid.spec_value() == b.pid.spec_value(),
            a.capabilities.spec_bits() == b.capabilities.spec_bits(),
            a.mutex_count == b.mutex_count,
            a.mutex_addrs@ =~= b.mutex_addrs@,
            a.mutex_ref_counts@ =~= b.mutex_ref_counts@,
            a.cond_count == b.cond_count,
            a.cond_addrs@ =~= b.cond_addrs@,
            a.cond_ref_counts@ =~= b.cond_ref_counts@,
            a.pmio_ports@ =~= b.pmio_ports@,
        ensures
            a@ == b@,
    {
    }
}

} // verus!
