// Copyright(c) The Maintainers of Nanvix.
// Licensed under the MIT License.

// Bitmap - Proofs
//
// This file contains lemmas and proof functions for Bitmap.
// Uses Set<int> as the primary abstraction.

use vstd::prelude::*;

verus! {

impl Bitmap {
    //==================================================================================================
    // Lemmas: Finiteness
    //==================================================================================================

    /// Lemma: A subset of a finite set is finite.
    /// set_bits ⊆ [0, num_bits) and [0, num_bits) is finite, so set_bits is finite.
    pub proof fn lemma_set_bits_finite(&self)
        requires
            self@.wf(),
            self@.num_bits >= 0,
        ensures
            self@.set_bits.finite(),
    {
        // The set [0, num_bits) is finite.
        // set_bits is a subset of [0, num_bits) by wf().
        // A subset of a finite set is finite.
        
        // Use set_int_range to get a finite set.
        let full_range: Set<int> = vstd::set_lib::set_int_range(0, self@.num_bits);
        
        // Prove full_range is finite using vstd lemma.
        vstd::set_lib::lemma_int_range(0, self@.num_bits);
        
        // Prove set_bits ⊆ full_range.
        assert(self@.set_bits.subset_of(full_range)) by {
            assert forall|i: int| self@.set_bits.contains(i) implies full_range.contains(i) by {
                // From wf(): set_bits.contains(i) ==> 0 <= i < num_bits
            }
        }
        
        // A subset of a finite set is finite.
        vstd::set_lib::lemma_set_subset_finite(full_range, self@.set_bits);
    }

    /// Lemma: Extensional equality preserves finiteness.
    /// If s1 =~= s2 and s2 is finite, then s1 is finite.
    pub proof fn lemma_ext_equal_finite(s1: Set<int>, s2: Set<int>)
        requires
            s1 =~= s2,
            s2.finite(),
        ensures
            s1.finite(),
    {
        // s1 =~= s2 means forall|a| s1.contains(a) == s2.contains(a)
        // So s1 ⊆ s2, and by lemma_set_subset_finite, s1.finite()
        assert(s1.subset_of(s2)) by {
            assert forall|a: int| s1.contains(a) implies s2.contains(a) by {}
        }
        vstd::set_lib::lemma_set_subset_finite(s2, s1);
    }

    /// Lemma: range_set(lo, hi) is finite when lo <= hi.
    pub proof fn lemma_range_set_finite(lo: int, hi: int)
        requires
            lo <= hi,
        ensures
            BitmapView::range_set(lo, hi).finite(),
    {
        // range_set(lo, hi) = { i | lo <= i < hi } = set_int_range(lo, hi)
        vstd::set_lib::lemma_int_range(lo, hi);
        // Prove equality.
        assert(BitmapView::range_set(lo, hi) =~= vstd::set_lib::set_int_range(lo, hi)) by {
            assert forall|i: int| BitmapView::range_set(lo, hi).contains(i) ==
                vstd::set_lib::set_int_range(lo, hi).contains(i) by {}
        }
        Self::lemma_ext_equal_finite(BitmapView::range_set(lo, hi), vstd::set_lib::set_int_range(lo, hi));
    }

    /// Lemma: Empty set is finite.
    pub proof fn lemma_empty_set_finite()
        ensures
            Set::<int>::empty().finite(),
    {
        // Axiom from vstd.
    }

    /// Lemma: Inserting into a finite set produces a finite set.
    pub proof fn lemma_insert_finite(s: Set<int>, x: int)
        requires
            s.finite(),
        ensures
            s.insert(x).finite(),
    {
        // Follows from vstd axiom_set_insert_finite.
    }

    /// Lemma: Removing from a finite set produces a finite set.
    pub proof fn lemma_remove_finite(s: Set<int>, x: int)
        requires
            s.finite(),
        ensures
            s.remove(x).finite(),
    {
        // Follows from vstd axiom_set_remove_finite.
    }

    /// Lemma: Union of two finite sets is finite.
    pub proof fn lemma_union_finite(s1: Set<int>, s2: Set<int>)
        requires
            s1.finite(),
            s2.finite(),
        ensures
            s1.union(s2).finite(),
    {
        // Follows from vstd axiom_set_union_finite.
    }

    /// Lemma: Difference of a finite set and any set is finite.
    pub proof fn lemma_difference_finite(s1: Set<int>, s2: Set<int>)
        requires
            s1.finite(),
        ensures
            s1.difference(s2).finite(),
    {
        // Follows from vstd axiom_set_difference_finite.
    }

    //==================================================================================================
    // Lemmas: Cardinality
    //==================================================================================================

    /// Lemma: Inserting a new element increases cardinality by 1.
    pub proof fn lemma_insert_len(s: Set<int>, x: int)
        requires
            s.finite(),
            !s.contains(x),
        ensures
            s.insert(x).len() == s.len() + 1,
    {
        // Follows from vstd axiom_set_insert_len.
    }

    /// Lemma: Inserting an existing element doesn't change cardinality.
    pub proof fn lemma_insert_same_len(s: Set<int>, x: int)
        requires
            s.finite(),
            s.contains(x),
        ensures
            s.insert(x).len() == s.len(),
    {
        assert(s.insert(x) =~= s);
    }

    /// Lemma: Removing an existing element decreases cardinality by 1.
    pub proof fn lemma_remove_len(s: Set<int>, x: int)
        requires
            s.finite(),
            s.contains(x),
        ensures
            s.remove(x).len() == s.len() - 1,
    {
        // Follows from vstd axiom_set_remove_len.
    }

    /// Lemma: Removing a non-existing element doesn't change cardinality.
    pub proof fn lemma_remove_same_len(s: Set<int>, x: int)
        requires
            s.finite(),
            !s.contains(x),
        ensures
            s.remove(x).len() == s.len(),
    {
        assert(s.remove(x) =~= s);
    }

    /// Lemma: Empty set has cardinality 0.
    pub proof fn lemma_empty_len()
        ensures
            Set::<int>::empty().len() == 0,
    {
        // Follows from vstd axiom_set_empty_len.
    }

    //==================================================================================================
    // Lemmas: Free Range Properties
    //==================================================================================================

    /// Lemma: if a free range of size n exists starting at p, then usage <= number_of_bits - n
    proof fn lemma_free_range_implies_usage_bound(&self, p: int, n: int)
        requires
            self.inv(),
            self.has_free_range_at(p, n),
            n > 0,
        ensures
            self@.usage() <= self@.number_of_bits() - n,
    {
        // has_free_range_at(p, n) means all bits in [p, p+n) are unset.
        // That means set_bits does not contain any index in [p, p+n).
        // So set_bits ⊆ [0, p) ∪ [p+n, num_bits).
        // |[0, p) ∪ [p+n, num_bits)| = p + (num_bits - p - n) = num_bits - n.
        // Therefore |set_bits| ≤ num_bits - n.
        
        // TODO: prove using cardinality reasoning.
        assume(self@.usage() <= self@.number_of_bits() - n);
    }

    /// Lemma: has_free_bit implies exists_contiguous_free_range(1)
    pub proof fn lemma_has_free_bit_implies_exists_free_range_1(&self)
        requires
            self.inv(),
            self@.has_free_bit(),
        ensures
            self.exists_contiguous_free_range(1),
    {
        // has_free_bit means exists i: 0 <= i < num_bits && !set_bits.contains(i).
        let i = choose|i: int| 0 <= i < self@.number_of_bits() && !self@.set_bits.contains(i);
        assert(!self.is_bit_set(i));
        assert(self.all_bits_unset_in_range(i, i + 1));
        assert(self.has_free_range_at(i, 1));
    }

    /// Lemma: if set_bits are equal, has_free_range_at returns the same result
    pub proof fn lemma_set_bits_equal_has_free_range_at_equal(&self, other: &Self, p: int, n: int)
        requires
            self.inv(),
            other.inv(),
            self@.set_bits =~= other@.set_bits,
            self@.number_of_bits() == other@.number_of_bits(),
        ensures
            self.has_free_range_at(p, n) == other.has_free_range_at(p, n),
    {
        assert forall|i: int| 0 <= i < self@.number_of_bits() implies
            self.is_bit_set(i) == other.is_bit_set(i)
        by {
            assert(self@.set_bits.contains(i) == other@.set_bits.contains(i));
        }
    }

    /// Lemma: if set_bits are equal, exists_contiguous_free_range returns the same result
    pub proof fn lemma_set_bits_equal_exists_free_range_equal(&self, other: &Self, n: int)
        requires
            self.inv(),
            other.inv(),
            self@.set_bits =~= other@.set_bits,
            self@.number_of_bits() == other@.number_of_bits(),
        ensures
            self.exists_contiguous_free_range(n) == other.exists_contiguous_free_range(n),
    {
        assert forall|p: int| #![trigger self.has_free_range_at(p, n)]
            self.has_free_range_at(p, n) == other.has_free_range_at(p, n)
        by {
            self.lemma_set_bits_equal_has_free_range_at_equal(other, p, n);
        }

        if self.exists_contiguous_free_range(n) {
            let p = choose|p: int| #[trigger] self.has_free_range_at(p, n);
            assert(other.has_free_range_at(p, n));
        }
        if other.exists_contiguous_free_range(n) {
            let p = choose|p: int| #[trigger] other.has_free_range_at(p, n);
            assert(self.has_free_range_at(p, n));
        }
    }

    //==================================================================================================
    // Lemmas: View Synchronization
    //==================================================================================================

    /// Lemma: if bitmap is empty, no bits are set
    pub proof fn lemma_is_empty_means_no_bits_set(&self)
        requires
            self.inv(),
            self@.is_empty(),
        ensures
            forall|i: int| 0 <= i < self@.number_of_bits() ==> !self.is_bit_set(i),
    {
        // is_empty means set_bits =~= Set::empty().
    }

    /// Lemma: if bitmap is full, all bits are set
    pub proof fn lemma_is_full_means_all_bits_set(&self)
        requires
            self.inv(),
            self@.is_full(),
        ensures
            forall|i: int| 0 <= i < self@.number_of_bits() ==> self.is_bit_set(i),
    {
        // is_full means forall|i| 0 <= i < num_bits ==> set_bits.contains(i).
    }

    /// Lemma: if bitmap is full, there are no free bits
    pub proof fn lemma_is_full_implies_no_free_bit(&self)
        requires
            self.inv(),
            self@.is_full(),
        ensures
            !self@.has_free_bit(),
    {
        self.lemma_is_full_means_all_bits_set();
    }

    /// Lemma: if bitmap is not full, there exists at least one unset bit
    pub proof fn lemma_not_full_means_exists_unset_bit(&self)
        requires
            self.inv(),
            !self@.is_full(),
        ensures
            exists|i: int| 0 <= i < self@.number_of_bits() && !self.is_bit_set(i),
    {
        // not is_full means: exists|i| 0 <= i < num_bits && !set_bits.contains(i).
        assert(exists|i: int| 0 <= i < self@.num_bits && !self@.set_bits.contains(i));
    }

    /// Lemma: If a specific bit is unset, then has_free_bit() is true.
    pub proof fn lemma_unset_bit_implies_has_free_bit(&self, i: int)
        requires
            self.inv(),
            0 <= i < self@.number_of_bits(),
            !self.is_bit_set(i),
        ensures
            self@.has_free_bit(),
    {
        assert(!self@.set_bits.contains(i));
    }

    /// Lemma: if all bits are set, bitmap is full
    pub proof fn lemma_all_bits_set_means_full(&self)
        requires
            self.inv(),
            forall|i: int| 0 <= i < self@.number_of_bits() ==> self.is_bit_set(i),
        ensures
            self@.is_full(),
    {
        assert forall|i: int| 0 <= i < self@.num_bits implies self@.set_bits.contains(i)
        by {
            assert(self.is_bit_set(i));
        };
    }

    //==================================================================================================
    // Lemmas: Bit-level Operations
    //==================================================================================================

    /// Lemma: Helper for proving bit operations on bytes
    proof fn lemma_bit_or_effects(old_byte: u8, bit_pos: int, new_byte: u8)
        requires
            0 <= bit_pos < 8,
            new_byte == (old_byte | (1u8 << bit_pos)),
        ensures
            (new_byte & (1u8 << bit_pos)) != 0,
            forall|other_pos: int| #![auto] 0 <= other_pos < 8 && other_pos != bit_pos ==>
                (new_byte & (1u8 << other_pos)) == (old_byte & (1u8 << other_pos)),
    {
        let shift: u8 = bit_pos as u8;
        assert((new_byte & (1u8 << shift)) != 0) by (bit_vector)
            requires
                new_byte == (old_byte | (1u8 << shift)),
                0 <= shift < 8,
        ;
        assert forall|other_pos: int| #![auto] 0 <= other_pos < 8 && other_pos != bit_pos implies
            (new_byte & (1u8 << other_pos)) == (old_byte & (1u8 << other_pos))
        by {
            let other_shift: u8 = other_pos as u8;
            assert((new_byte & (1u8 << other_shift)) == (old_byte & (1u8 << other_shift))) by (bit_vector)
                requires
                    new_byte == (old_byte | (1u8 << shift)),
                    0 <= shift < 8,
                    0 <= other_shift < 8,
                    shift != other_shift,
            ;
        }
    }

    /// Lemma: Helper for proving bit clear operations on bytes
    proof fn lemma_bit_and_not_effects(old_byte: u8, bit_pos: int, new_byte: u8)
        requires
            0 <= bit_pos < 8,
            new_byte == (old_byte & !(1u8 << bit_pos)),
        ensures
            (new_byte & (1u8 << bit_pos)) == 0,
            forall|other_pos: int| #![auto] 0 <= other_pos < 8 && other_pos != bit_pos ==>
                (new_byte & (1u8 << other_pos)) == (old_byte & (1u8 << other_pos)),
    {
        let shift: u8 = bit_pos as u8;
        assert((new_byte & (1u8 << shift)) == 0) by (bit_vector)
            requires
                new_byte == (old_byte & !(1u8 << shift)),
                0 <= shift < 8,
        ;
        assert forall|other_pos: int| #![auto] 0 <= other_pos < 8 && other_pos != bit_pos implies
            (new_byte & (1u8 << other_pos)) == (old_byte & (1u8 << other_pos))
        by {
            let other_shift: u8 = other_pos as u8;
            assert((new_byte & (1u8 << other_shift)) == (old_byte & (1u8 << other_shift))) by (bit_vector)
                requires
                    new_byte == (old_byte & !(1u8 << shift)),
                    0 <= shift < 8,
                    0 <= other_shift < 8,
                    shift != other_shift,
            ;
        }
    }

    /// Lemma: setting a byte bit reflects in set_bits
    proof fn lemma_byte_or_reflects_in_view(&self, new_self: &Self, word: int, bit: int)
        requires
            self.inv(),
            0 <= word < self.bits@.len(),
            0 <= bit < (u8::BITS as int),
            new_self.bits@.len() == self.bits@.len(),
            new_self.bits@[word] == (self.bits@[word] | (1u8 << bit)),
            forall|i: int| 0 <= i < self.bits@.len() && i != word ==> self.bits@[i] == new_self.bits@[i],
            self.number_of_bits == new_self.number_of_bits,
        ensures
            new_self@.set_bits =~= self@.set_bits.insert(word * (u8::BITS as int) + bit),
    {
        Self::lemma_bit_or_effects(self.bits@[word], bit, new_self.bits@[word]);
        let idx: int = word * (u8::BITS as int) + bit;
        
        assert forall|i: int| new_self@.set_bits.contains(i) == self@.set_bits.insert(idx).contains(i) by {
            if i == idx {
                assert(Self::bit_at(new_self.bits@, idx));
            } else if 0 <= i < self@.number_of_bits() {
                let i_word: int = i / (u8::BITS as int);
                let i_bit: int = i % (u8::BITS as int);
                if i_word == word {
                    assert((new_self.bits@[word] & (1u8 << i_bit)) == (self.bits@[word] & (1u8 << i_bit)));
                } else {
                    assert(self.bits@[i_word] == new_self.bits@[i_word]);
                }
                assert(Self::bit_at(self.bits@, i) == Self::bit_at(new_self.bits@, i));
            }
        }
    }

    /// Lemma: clearing a byte bit reflects in set_bits
    proof fn lemma_byte_and_not_reflects_in_view(&self, new_self: &Self, word: int, bit: int)
        requires
            self.inv(),
            0 <= word < self.bits@.len(),
            0 <= bit < (u8::BITS as int),
            new_self.bits@.len() == self.bits@.len(),
            new_self.bits@[word] == (self.bits@[word] & !(1u8 << bit)),
            forall|i: int| 0 <= i < self.bits@.len() && i != word ==> self.bits@[i] == new_self.bits@[i],
            self.number_of_bits == new_self.number_of_bits,
        ensures
            new_self@.set_bits =~= self@.set_bits.remove(word * (u8::BITS as int) + bit),
    {
        Self::lemma_bit_and_not_effects(self.bits@[word], bit, new_self.bits@[word]);
        let idx: int = word * (u8::BITS as int) + bit;
        
        assert forall|i: int| new_self@.set_bits.contains(i) == self@.set_bits.remove(idx).contains(i) by {
            if i == idx {
                assert(!Self::bit_at(new_self.bits@, idx));
            } else if 0 <= i < self@.number_of_bits() {
                let i_word: int = i / (u8::BITS as int);
                let i_bit: int = i % (u8::BITS as int);
                if i_word == word {
                    assert((new_self.bits@[word] & (1u8 << i_bit)) == (self.bits@[word] & (1u8 << i_bit)));
                } else {
                    assert(self.bits@[i_word] == new_self.bits@[i_word]);
                }
                assert(Self::bit_at(self.bits@, i) == Self::bit_at(new_self.bits@, i));
            }
        }
    }

    /// Lemma: when all raw bytes are zero, set_bits is empty
    proof fn lemma_zero_bytes_means_empty_set(&self)
        requires
            self@.number_of_bits() == self.bits@.len() * (u8::BITS as int),
            forall|i: int| 0 <= i < self.bits@.len() ==> self.bits@[i] == 0,
        ensures
            self@.set_bits =~= Set::<int>::empty(),
    {
        assert forall|i: int| !self@.set_bits.contains(i) by {
            if 0 <= i < self@.number_of_bits() {
                let byte_idx: int = i / (u8::BITS as int);
                let bit_idx: int = i % (u8::BITS as int);
                let bit_idx_u8: u8 = bit_idx as u8;
                assert((0u8 & (1u8 << bit_idx_u8)) == 0) by (bit_vector)
                    requires 0 <= bit_idx_u8 < 8;
                assert(!Self::bit_at(self.bits@, i));
            }
        }
    }

    /// Lemma: Connects closed `is_bit_set` to open `BitmapView.is_bit_set`.
    pub proof fn lemma_is_bit_set_equals_view(&self, i: int)
        requires
            self.inv(),
            0 <= i < self@.number_of_bits(),
        ensures
            self.is_bit_set(i) == self@.is_bit_set(i),
    {
        // Both reduce to self@.set_bits.contains(i).
    }

    /// Proves that number_of_bits is bounded by usize::MAX.
    pub proof fn lemma_number_of_bits_bounded(&self)
        requires
            self.inv(),
        ensures
            self@.number_of_bits() <= usize::MAX as int,
    {
        // From inv(): number_of_bits < u32::MAX, and u32::MAX <= usize::MAX.
    }

} // impl Bitmap

} // verus!
