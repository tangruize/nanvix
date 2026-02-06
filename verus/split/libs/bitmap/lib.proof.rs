// Copyright(c) The Maintainers of Nanvix.
// Licensed under the MIT License.

// Bitmap - Proofs
//
// This file contains lemmas and proof functions for Bitmap.

use vstd::prelude::*;

verus! {

impl Bitmap {
    //==================================================================================================
    // Lemmas: Basic Properties
    //==================================================================================================

    /// Lemma: count in a sequence range is bounded by the range size
    proof fn lemma_count_set_bits_in_seq_bounded(bits: Seq<bool>, start: int, end: int)
        requires
            start <= end,
        ensures
            Self::count_set_bits_in_seq(bits, start, end) >= 0,
            Self::count_set_bits_in_seq(bits, start, end) <= end - start,
        decreases end - start
    {
        if start >= end {
        } else {
            Self::lemma_count_set_bits_in_seq_bounded(bits, start + 1, end);
        }
    }

    //==================================================================================================
    // Lemmas: Bit Set/Unset Properties
    //==================================================================================================

    /// Lemma: if a bit in sequence is set, count in range >= 1
    proof fn lemma_bit_set_in_seq_implies_count_geq_1(bits: Seq<bool>, start: int, end: int, index: int)
        requires
            start <= index < end,
            0 <= index < bits.len(),
            bits[index],
        ensures
            Self::count_set_bits_in_seq(bits, start, end) >= 1,
        decreases end - start
    {
        if start >= end {
        } else if start == index {
            Self::lemma_count_set_bits_in_seq_bounded(bits, start + 1, end);
        } else {
            Self::lemma_bit_set_in_seq_implies_count_geq_1(bits, start + 1, end, index);
        }
    }

    /// Lemma: if a bit in sequence is not set, count < range size
    proof fn lemma_bit_unset_in_seq_implies_count_lt_size(bits: Seq<bool>, start: int, end: int, index: int)
        requires
            start <= index < end,
            0 <= index < bits.len(),
            !bits[index],
        ensures
            Self::count_set_bits_in_seq(bits, start, end) < end - start,
        decreases end - start
    {
        if start >= end {
        } else if start == index {
            Self::lemma_count_set_bits_in_seq_bounded(bits, start + 1, end);
        } else {
            Self::lemma_bit_unset_in_seq_implies_count_lt_size(bits, start + 1, end, index);
        }
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
        // Therefore, at least n bits are unset.
        // So usage (count of set bits) <= number_of_bits - n.

        // We prove this by showing that count_free >= n.
        // count_free = number_of_bits - usage.
        // If forall i in [p, p+n): !is_bit_set(i), then there are at least n unset bits.

        Self::lemma_unset_range_implies_count_free_geq(&self, p, n);
    }

    /// Lemma: if a range [p, p+n) is all unset, then count_free >= n
    proof fn lemma_unset_range_implies_count_free_geq(&self, p: int, n: int)
        requires
            self.inv(),
            0 <= p,
            p + n <= self@.number_of_bits(),
            n > 0,
            self.all_bits_unset_in_range(p, p + n),
        ensures
            self@.count_free() >= n,
    {
        // count_free = number_of_bits - usage.
        // usage = count_set_bits_in_seq(bits, 0, number_of_bits).
        // We need to show: number_of_bits - usage >= n.
        // Equivalently: usage <= number_of_bits - n.

        // Split the range [0, number_of_bits) into three parts:
        // [0, p), [p, p+n), [p+n, number_of_bits).
        // In [p, p+n), all bits are unset, so contribution to usage is 0.
        // usage = count in [0, p) + count in [p, p+n) + count in [p+n, number_of_bits).
        //       = count in [0, p) + 0 + count in [p+n, number_of_bits).
        //       <= p + (number_of_bits - (p+n))
        //       = number_of_bits - n.

        Self::lemma_count_split(self@.bits, 0, p, self@.number_of_bits());
        Self::lemma_count_split(self@.bits, p, p + n, self@.number_of_bits());

        // Count in [p, p+n) is 0 because all bits are unset.
        assert forall|i: int| p <= i < p + n implies !self@.bits[i]
        by {
            assert(self.all_bits_unset_in_range(p, p + n));
            assert(!self.is_bit_set(i));
        }
        Self::lemma_all_zero_in_seq_implies_count_zero(self@.bits, p, p + n);

        // Count in [0, p) is at most p.
        Self::lemma_count_set_bits_in_seq_bounded(self@.bits, 0, p);

        // Count in [p+n, number_of_bits) is at most number_of_bits - (p+n).
        Self::lemma_count_set_bits_in_seq_bounded(self@.bits, p + n, self@.number_of_bits());
    }

    /// Lemma: count can be split across ranges
    proof fn lemma_count_split(bits: Seq<bool>, start: int, mid: int, end: int)
        requires
            0 <= start <= mid <= end,
            end <= bits.len(),
        ensures
            Self::count_set_bits_in_seq(bits, start, end) ==
                Self::count_set_bits_in_seq(bits, start, mid) +
                Self::count_set_bits_in_seq(bits, mid, end),
        decreases mid - start
    {
        if start >= mid {
            // Empty first range.
        } else {
            Self::lemma_count_split(bits, start + 1, mid, end);
        }
    }

    /// Lemma: has_free_bit implies exists_contiguous_free_range(1)
    pub proof fn lemma_has_free_bit_implies_exists_free_range_1(&self)
        requires
            self.inv(),
            self@.has_free_bit(),
        ensures
            self.exists_contiguous_free_range(1),
    {
        // has_free_bit means exists i: 0 <= i < number_of_bits && !bits[i].
        // This means !is_bit_set(i).
        // For exists_contiguous_free_range(1), we need exists start: has_free_range_at(start, 1).
        // has_free_range_at(i, 1) requires:
        //   - 0 <= i
        //   - i + 1 <= number_of_bits
        //   - all_bits_unset_in_range(i, i+1)
        // Since !is_bit_set(i), all_bits_unset_in_range(i, i+1) holds.

        let i = choose|i: int| 0 <= i < self@.number_of_bits() && !self@.bits[i];
        assert(!self.is_bit_set(i));
        assert(self.all_bits_unset_in_range(i, i + 1));
        assert(self.has_free_range_at(i, 1));
    }

    /// Lemma: if bits are equal, has_free_range_at returns the same result
    pub proof fn lemma_bits_equal_has_free_range_at_equal(&self, other: &Self, p: int, n: int)
        requires
            self.inv(),
            other.inv(),
            self@.bits =~= other@.bits,
            self@.number_of_bits() == other@.number_of_bits(),
        ensures
            self.has_free_range_at(p, n) == other.has_free_range_at(p, n),
    {
        // has_free_range_at depends on number_of_bits and all_bits_unset_in_range.
        // all_bits_unset_in_range depends on is_bit_set.
        // is_bit_set depends on bits[i].
        // Since bits are equal, is_bit_set returns the same result for both.
        assert forall|i: int| 0 <= i < self@.number_of_bits() implies
            self.is_bit_set(i) == other.is_bit_set(i)
        by {
            assert(self@.bits[i] == other@.bits[i]);
        }
        // Therefore all_bits_unset_in_range returns the same result.
        // Therefore has_free_range_at returns the same result.
    }

    /// Lemma: if bits are equal, exists_contiguous_free_range returns the same result
    pub proof fn lemma_bits_equal_exists_free_range_equal(&self, other: &Self, n: int)
        requires
            self.inv(),
            other.inv(),
            self@.bits =~= other@.bits,
            self@.number_of_bits() == other@.number_of_bits(),
        ensures
            self.exists_contiguous_free_range(n) == other.exists_contiguous_free_range(n),
    {
        // exists_contiguous_free_range(n) = exists|start| has_free_range_at(start, n).
        // By the previous lemma, has_free_range_at returns the same result for both.
        assert forall|p: int| #![trigger self.has_free_range_at(p, n)]
            self.has_free_range_at(p, n) == other.has_free_range_at(p, n)
        by {
            self.lemma_bits_equal_has_free_range_at_equal(other, p, n);
        }

        // Now prove the existentials are equal.
        // If self.exists_contiguous_free_range(n), then there exists a p such that self.has_free_range_at(p, n).
        // By the above, other.has_free_range_at(p, n) is also true, so other.exists_contiguous_free_range(n).
        // Symmetrically for the other direction.
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
    // Lemmas: Bit Mutation Effects
    //==================================================================================================

    /// Lemma: setting a bit increases the count by 1
    proof fn lemma_set_bit_increases_count(&self, new_self: &Self, index: int)
        requires
            0 <= index < self@.number_of_bits(),
            !self.is_bit_set(index),
            new_self.is_bit_set(index),
            forall|i: int| 0 <= i < self@.number_of_bits() && i != index ==>
                self.is_bit_set(i) == new_self.is_bit_set(i),
            self@.number_of_bits() == new_self@.number_of_bits(),
            self@.bits.len() == new_self@.bits.len(),
        ensures
            new_self@.usage() == self@.usage() + 1,
    {
        assert forall|i: int| 0 <= i < self@.number_of_bits() && i != index
        implies self@.bits[i] == new_self@.bits[i]
        by {
            assert(self.is_bit_set(i) == new_self.is_bit_set(i));
        };

        Self::lemma_set_bit_increases_count_in_seq(self@.bits, new_self@.bits, 0, self@.number_of_bits(), index);
    }

    /// Lemma: setting a bit in a sequence increases the count by 1
    proof fn lemma_set_bit_increases_count_in_seq(old_bits: Seq<bool>, new_bits: Seq<bool>, start: int, end: int, index: int)
        requires
            start <= index < end,
            0 <= index < old_bits.len(),
            0 <= index < new_bits.len(),
            old_bits.len() == new_bits.len(),
            !old_bits[index],
            new_bits[index],
            forall|i: int| start <= i < end && i != index && 0 <= i < old_bits.len() ==>
                old_bits[i] == new_bits[i],
        ensures
            Self::count_set_bits_in_seq(new_bits, start, end) == Self::count_set_bits_in_seq(old_bits, start, end) + 1,
        decreases end - start
    {
        if start >= end {
        } else if start == index {
            Self::lemma_bits_equal_in_seq_implies_count_equal(old_bits, new_bits, start + 1, end);
        } else {
            Self::lemma_set_bit_increases_count_in_seq(old_bits, new_bits, start + 1, end, index);
        }
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

    /// Lemma: clearing a bit decreases the count by 1
    proof fn lemma_clear_bit_decreases_count(&self, new_self: &Self, index: int)
        requires
            0 <= index < self@.number_of_bits(),
            self.is_bit_set(index),
            !new_self.is_bit_set(index),
            forall|i: int| 0 <= i < self@.number_of_bits() && i != index ==>
                self.is_bit_set(i) == new_self.is_bit_set(i),
            self@.number_of_bits() == new_self@.number_of_bits(),
            self@.bits.len() == new_self@.bits.len(),
        ensures
            new_self@.usage() == self@.usage() - 1,
    {
        assert forall|i: int| 0 <= i < self@.number_of_bits() && i != index
        implies self@.bits[i] == new_self@.bits[i]
        by {
            assert(self.is_bit_set(i) == new_self.is_bit_set(i));
        };

        Self::lemma_clear_bit_decreases_count_in_seq(self@.bits, new_self@.bits, 0, self@.number_of_bits(), index);
    }

    /// Lemma: clearing a bit in a sequence decreases the count by 1
    proof fn lemma_clear_bit_decreases_count_in_seq(old_bits: Seq<bool>, new_bits: Seq<bool>, start: int, end: int, index: int)
        requires
            start <= index < end,
            0 <= index < old_bits.len(),
            0 <= index < new_bits.len(),
            old_bits.len() == new_bits.len(),
            old_bits[index],
            !new_bits[index],
            forall|i: int| start <= i < end && i != index && 0 <= i < old_bits.len() ==>
                old_bits[i] == new_bits[i],
        ensures
            Self::count_set_bits_in_seq(new_bits, start, end) == Self::count_set_bits_in_seq(old_bits, start, end) - 1,
        decreases end - start
    {
        if start >= end {
        } else if start == index {
            Self::lemma_bits_equal_in_seq_implies_count_equal(old_bits, new_bits, start + 1, end);
        } else {
            Self::lemma_clear_bit_decreases_count_in_seq(old_bits, new_bits, start + 1, end, index);
        }
    }

    /// Lemma: if bits in sequences are equal in a range, counts are equal
    proof fn lemma_bits_equal_in_seq_implies_count_equal(bits1: Seq<bool>, bits2: Seq<bool>, start: int, end: int)
        requires
            start <= end,
            bits1.len() == bits2.len(),
            forall|i: int| start <= i < end && 0 <= i < bits1.len() ==>
                bits1[i] == bits2[i],
        ensures
            Self::count_set_bits_in_seq(bits1, start, end) == Self::count_set_bits_in_seq(bits2, start, end),
        decreases end - start
    {
        if start >= end {
        } else {
            Self::lemma_bits_equal_in_seq_implies_count_equal(bits1, bits2, start + 1, end);
        }
    }

    /// Lemma: if all bits in sequence are false, count == 0
    proof fn lemma_all_zero_in_seq_implies_count_zero(bits: Seq<bool>, start: int, end: int)
        requires
            0 <= start <= end,
            forall|i: int| start <= i < end && 0 <= i < bits.len() ==> !bits[i],
        ensures
            Self::count_set_bits_in_seq(bits, start, end) == 0,
        decreases end - start
    {
        if start >= end {
        } else {
            Self::lemma_all_zero_in_seq_implies_count_zero(bits, start + 1, end);
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
        if exists|i: int| 0 <= i < self@.number_of_bits() && self.is_bit_set(i) {
            let i = choose|i: int| 0 <= i < self@.number_of_bits() && self.is_bit_set(i);
            Self::lemma_bit_set_in_seq_implies_count_geq_1(self@.bits, 0, self@.number_of_bits(), i);
        }
    }

    /// Lemma: if bitmap is full, all bits are set
    pub proof fn lemma_is_full_means_all_bits_set(&self)
        requires
            self.inv(),
            self@.is_full(),
        ensures
            forall|i: int| 0 <= i < self@.number_of_bits() ==> self.is_bit_set(i),
    {
        if exists|i: int| 0 <= i < self@.number_of_bits() && !self.is_bit_set(i) {
            let i = choose|i: int| 0 <= i < self@.number_of_bits() && !self.is_bit_set(i);
            Self::lemma_bit_unset_in_seq_implies_count_lt_size(self@.bits, 0, self@.number_of_bits(), i);
        }
    }

    /// Lemma: if bitmap is full, there are no free bits
    pub proof fn lemma_is_full_implies_no_free_bit(&self)
        requires
            self.inv(),
            self@.is_full(),
        ensures
            !self@.has_free_bit(),
    {
        // Prove all bits are true, which contradicts has_free_bit().
        self.lemma_is_full_means_all_bits_set();
        assert forall|i: int| 0 <= i < self@.number_of_bits() implies self@.bits[i] by {
            assert(self.is_bit_set(i));
        }
    }

    /// Lemma: if bitmap is not full, there exists at least one unset bit
    pub proof fn lemma_not_full_means_exists_unset_bit(&self)
        requires
            self.inv(),
            !self@.is_full(),
        ensures
            exists|i: int| 0 <= i < self@.number_of_bits() && !self.is_bit_set(i),
    {
        if forall|i: int| 0 <= i < self@.number_of_bits() ==> self.is_bit_set(i) {
            Self::lemma_all_bits_set_means_full(self);
        }
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
        // Witness i satisfies the existential in has_free_bit().
        assert(!self@.bits[i]);
    }

    /// Lemma: if all bits are set, bitmap is full
    pub proof fn lemma_all_bits_set_means_full(&self)
        requires
            self.inv(),
            forall|i: int| 0 <= i < self@.number_of_bits() ==> self.is_bit_set(i),
        ensures
            self@.is_full(),
        decreases self@.number_of_bits()
    {
        assert forall|i: int| 0 <= i < self@.number_of_bits() implies self@.bits[i]
        by {
            assert(self.is_bit_set(i));
        };
        Self::lemma_all_set_means_count_equals_size(self@.bits, 0, self@.number_of_bits());
    }

    /// Lemma: if all bits in range are set, count equals range size
    proof fn lemma_all_set_means_count_equals_size(bits: Seq<bool>, start: int, end: int)
        requires
            0 <= start <= end,
            end <= bits.len(),
            forall|i: int| start <= i < end ==> bits[i],
        ensures
            Self::count_set_bits_in_seq(bits, start, end) == end - start,
        decreases end - start
    {
        if start >= end {
        } else {
            Self::lemma_all_set_means_count_equals_size(bits, start + 1, end);
        }
    }

    /// Lemma: setting a byte bit reflects in the boolean sequence and set_bits
    proof fn lemma_byte_or_reflects_in_view(&self, new_self: &Self, word: int, bit: int)
        requires
            0 <= word < self.bits@.len(),
            0 <= bit < (u8::BITS as int),
            new_self.bits@.len() == self.bits@.len(),
            new_self.bits@[word] == (self.bits@[word] | (1u8 << bit)),
            forall|i: int| 0 <= i < self.bits@.len() && i != word ==> self.bits@[i] == new_self.bits@[i],
            self.number_of_bits == new_self.number_of_bits,
            self@.number_of_bits() == self.bits@.len() * (u8::BITS as int),
        ensures
            forall|i: int| 0 <= i < self@.number_of_bits() ==>
                self@.bits[i] == new_self@.bits[i] || i == word * (u8::BITS as int) + bit,
            new_self@.bits[word * (u8::BITS as int) + bit],
            // Set-based: new set_bits = old set_bits + the new bit index.
            new_self@.set_bits =~= self@.set_bits.insert(word * (u8::BITS as int) + bit),
    {
        Self::lemma_bit_or_effects(self.bits@[word], bit, new_self.bits@[word]);
        let idx: int = word * (u8::BITS as int) + bit;
        // Prove set_bits equality
        assert forall|i: int| new_self@.set_bits.contains(i) == self@.set_bits.insert(idx).contains(i) by {
            if i == idx {
                assert(Self::bit_at(new_self.bits@, idx));
            } else if 0 <= i < self@.number_of_bits() {
                assert(Self::bit_at(self.bits@, i) == Self::bit_at(new_self.bits@, i));
            }
        }
    }

    /// Lemma: clearing a byte bit reflects in the boolean sequence and set_bits
    proof fn lemma_byte_and_not_reflects_in_view(&self, new_self: &Self, word: int, bit: int)
        requires
            0 <= word < self.bits@.len(),
            0 <= bit < (u8::BITS as int),
            new_self.bits@.len() == self.bits@.len(),
            new_self.bits@[word] == (self.bits@[word] & !(1u8 << bit)),
            forall|i: int| 0 <= i < self.bits@.len() && i != word ==> self.bits@[i] == new_self.bits@[i],
            self.number_of_bits == new_self.number_of_bits,
            self@.number_of_bits() == self.bits@.len() * (u8::BITS as int),
        ensures
            forall|i: int| 0 <= i < self@.number_of_bits() ==>
                self@.bits[i] == new_self@.bits[i] || i == word * (u8::BITS as int) + bit,
            !new_self@.bits[word * (u8::BITS as int) + bit],
            // Set-based: new set_bits = old set_bits - the cleared bit index.
            new_self@.set_bits =~= self@.set_bits.remove(word * (u8::BITS as int) + bit),
    {
        Self::lemma_bit_and_not_effects(self.bits@[word], bit, new_self.bits@[word]);
        let idx: int = word * (u8::BITS as int) + bit;
        // Prove set_bits equality
        assert forall|i: int| new_self@.set_bits.contains(i) == self@.set_bits.remove(idx).contains(i) by {
            if i == idx {
                assert(!Self::bit_at(new_self.bits@, idx));
            } else if 0 <= i < self@.number_of_bits() {
                assert(Self::bit_at(self.bits@, i) == Self::bit_at(new_self.bits@, i));
            }
        }
    }

    /// Lemma: when all raw bytes are zero, all boolean bits are false and set_bits is empty
    proof fn lemma_zero_bytes_means_false_bits(&self)
        requires
            self@.number_of_bits() == self.bits@.len() * (u8::BITS as int),
            forall|i: int| 0 <= i < self.bits@.len() ==> self.bits@[i] == 0,
        ensures
            forall|i: int| 0 <= i < self@.number_of_bits() ==> !self@.bits[i],
            self@.set_bits =~= Set::<int>::empty(),
    {
        assert forall|i: int| 0 <= i < self@.number_of_bits() implies !self@.bits[i] by {
            let byte_idx = i / (u8::BITS as int);
            let bit_idx = i % (u8::BITS as int);

            let bit_idx_u8 = bit_idx as u8;
            assert((0u8 & (1u8 << bit_idx_u8)) == 0) by (bit_vector)
                requires 0 <= bit_idx_u8 < 8;
        };
        // Prove set_bits is empty: no bit is set, so no index is in set_bits
        // The key insight is: set_bits = Set::new(|i| 0 <= i < num_bits && bit_at(bits, i))
        // and self@.bits[i] == bit_at(self.bits@, i) by definition of bits_to_seq
        assert forall|i: int| !self@.set_bits.contains(i) by {
            if 0 <= i < self@.number_of_bits() {
                // self@.bits[i] == bit_at(self.bits@, i) by definition (via bits_to_seq)
                // We already proved !self@.bits[i] above.
                // So bit_at(self.bits@, i) == false.
                assert(!self@.bits[i]);
                // set_bits.contains(i) requires bit_at(self.bits@, i) which is false.
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
        // Both reduce to self@.bits[i].
    }

    /// Lemma: If bits sequences are equal, then is_bit_set returns the same result.
    pub proof fn lemma_bits_equal_implies_is_bit_set_equal(&self, other: &Self, i: int)
        requires
            self.inv(),
            other.inv(),
            self@.bits =~= other@.bits,
            0 <= i < self@.number_of_bits(),
        ensures
            self.is_bit_set(i) == other.is_bit_set(i),
    {
        // Both is_bit_set definitions reduce to bits[i].
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
