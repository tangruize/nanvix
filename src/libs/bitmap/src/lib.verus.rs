// Copyright(c) The Maintainers of Nanvix.
// Licensed under the MIT License.

// Bitmap Verification Specifications
//
// This file contains complete Verus verification specifications, proofs, and lemmas
// for the Bitmap module. All specs and proofs are copied from verus/bitmap.rs.

use vstd::prelude::*;

verus! {

//==================================================================================================
// BitmapView - The Abstract Specification Model
//==================================================================================================

/// A view of the Bitmap as a sequence of booleans.
#[verifier::ext_equal]
pub ghost struct BitmapView {
    pub bits: Seq<bool>,
}

impl BitmapView {
    /// Returns the number of bits in the bitmap view.
    pub open spec fn number_of_bits(&self) -> int {
        self.bits.len() as int
    }

    /// Returns the usage (count of set bits) in the bitmap view.
    pub open spec fn usage(&self) -> int {
        count_set_bits_in_seq(self.bits, 0, self.bits.len() as int)
    }

    /// Alias for usage() - returns the count of allocated (set) bits.
    pub open spec fn count_allocated(&self) -> int {
        self.usage()
    }

    /// Returns the count of free (unset) bits.
    pub open spec fn count_free(&self) -> int {
        self.number_of_bits() - self.count_allocated()
    }

    /// Returns true if there exists at least one unset bit.
    pub open spec fn has_free_bit(&self) -> bool {
        exists|i: int| 0 <= i < self.number_of_bits() && !self.bits[i]
    }

    /// Returns true if the bitmap is full (all bits set).
    pub open spec fn is_full(&self) -> bool {
        self.usage() == self.number_of_bits()
    }

    /// Returns true if the bitmap is empty (no bits set).
    pub open spec fn is_empty(&self) -> bool {
        self.usage() == 0
    }

    /// Returns true if a specific bit is set.
    pub open spec fn is_bit_set(&self, index: int) -> bool {
        self.bits[index]
    }
}

//==================================================================================================
// Specification Functions
//==================================================================================================

/// Helper spec function: get the bit value at a specific index from raw bytes.
pub open spec fn bit_at(bytes: Seq<u8>, bit_index: int) -> bool {
    let word: int = bit_index / (u8::BITS as int);
    let bit: int = bit_index % (u8::BITS as int);
    if 0 <= bit_index && word < bytes.len() {
        (bytes[word] & (1u8 << bit)) != 0
    } else {
        false
    }
}

/// Helper spec function: convert Seq<u8> to Seq<bool>.
pub open spec fn bits_to_seq(bytes: Seq<u8>, num_bits: int) -> Seq<bool>
    decreases num_bits
{
    Seq::new(num_bits as nat, |i: int| bit_at(bytes, i))
}

/// Helper spec function: check if a bit at the given index is set.
pub open spec fn is_bit_set_at(view: &BitmapView, bit_index: int) -> bool {
    &&& 0 <= bit_index < view.number_of_bits()
    &&& view.bits[bit_index]
}

/// Helper spec function: check if all bits in range [start, end) are set.
pub open spec fn all_bits_set_in_range(view: &BitmapView, start: int, end: int) -> bool {
    forall|i: int| start <= i < end ==> is_bit_set_at(view, i)
}

/// Helper spec function: check if all bits in range [start, end) are not set.
pub open spec fn all_bits_unset_in_range(view: &BitmapView, start: int, end: int) -> bool {
    forall|i: int| start <= i < end ==> !is_bit_set_at(view, i)
}

/// Helper spec function: check if there exists a contiguous range of n free bits starting at start.
pub open spec fn has_free_range_at(view: &BitmapView, start: int, n: int) -> bool {
    &&& 0 <= start
    &&& start + n <= view.number_of_bits()
    &&& all_bits_unset_in_range(view, start, start + n)
}

/// Helper spec function: check if there exists a contiguous range of n free bits.
pub open spec fn exists_contiguous_free_range(view: &BitmapView, n: int) -> bool {
    exists|start: int| #![trigger has_free_range_at(view, start, n)]
        has_free_range_at(view, start, n)
}

/// Helper spec function: count set bits in a range [start, end) of a sequence.
pub closed spec fn count_set_bits_in_seq(bits: Seq<bool>, start: int, end: int) -> int
    decreases end - start when end >= start
{
    if start >= end {
        0
    } else {
        let rest = count_set_bits_in_seq(bits, start + 1, end);
        if 0 <= start < bits.len() && bits[start] { rest + 1 } else { rest }
    }
}

/// Bitmap invariant.
pub open spec fn bitmap_inv(num_bits: int, usage: int, bytes_len: int) -> bool {
    &&& num_bits > 0
    &&& num_bits == bytes_len * (u8::BITS as int)
    &&& num_bits < u32::MAX as int
    &&& usage >= 0
    &&& usage <= num_bits
}

//==================================================================================================
// Lemmas: Basic Properties
//==================================================================================================

/// Lemma: count in a sequence range is bounded by the range size.
pub proof fn lemma_count_set_bits_in_seq_bounded(bits: Seq<bool>, start: int, end: int)
    requires
        start <= end,
    ensures
        count_set_bits_in_seq(bits, start, end) >= 0,
        count_set_bits_in_seq(bits, start, end) <= end - start,
    decreases end - start
{
    if start >= end {
    } else {
        lemma_count_set_bits_in_seq_bounded(bits, start + 1, end);
    }
}

//==================================================================================================
// Lemmas: Bit Set/Unset Properties
//==================================================================================================

/// Lemma: if a bit in sequence is set, count in range >= 1.
pub proof fn lemma_bit_set_in_seq_implies_count_geq_1(bits: Seq<bool>, start: int, end: int, index: int)
    requires
        start <= index < end,
        0 <= index < bits.len(),
        bits[index],
    ensures
        count_set_bits_in_seq(bits, start, end) >= 1,
    decreases end - start
{
    if start >= end {
    } else if start == index {
        lemma_count_set_bits_in_seq_bounded(bits, start + 1, end);
    } else {
        lemma_bit_set_in_seq_implies_count_geq_1(bits, start + 1, end, index);
    }
}

/// Lemma: if a bit in sequence is not set, count < range size.
pub proof fn lemma_bit_unset_in_seq_implies_count_lt_size(bits: Seq<bool>, start: int, end: int, index: int)
    requires
        start <= index < end,
        0 <= index < bits.len(),
        !bits[index],
    ensures
        count_set_bits_in_seq(bits, start, end) < end - start,
    decreases end - start
{
    if start >= end {
    } else if start == index {
        lemma_count_set_bits_in_seq_bounded(bits, start + 1, end);
    } else {
        lemma_bit_unset_in_seq_implies_count_lt_size(bits, start + 1, end, index);
    }
}

//==================================================================================================
// Lemmas: Bit Mutation Effects
//==================================================================================================

/// Lemma: setting a bit in a sequence increases the count by 1.
pub proof fn lemma_set_bit_increases_count_in_seq(old_bits: Seq<bool>, new_bits: Seq<bool>, start: int, end: int, index: int)
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
        count_set_bits_in_seq(new_bits, start, end) == count_set_bits_in_seq(old_bits, start, end) + 1,
    decreases end - start
{
    if start >= end {
    } else if start == index {
        lemma_bits_equal_in_seq_implies_count_equal(old_bits, new_bits, start + 1, end);
    } else {
        lemma_set_bit_increases_count_in_seq(old_bits, new_bits, start + 1, end, index);
    }
}

/// Lemma: clearing a bit in a sequence decreases the count by 1.
pub proof fn lemma_clear_bit_decreases_count_in_seq(old_bits: Seq<bool>, new_bits: Seq<bool>, start: int, end: int, index: int)
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
        count_set_bits_in_seq(new_bits, start, end) == count_set_bits_in_seq(old_bits, start, end) - 1,
    decreases end - start
{
    if start >= end {
    } else if start == index {
        lemma_bits_equal_in_seq_implies_count_equal(old_bits, new_bits, start + 1, end);
    } else {
        lemma_clear_bit_decreases_count_in_seq(old_bits, new_bits, start + 1, end, index);
    }
}

/// Lemma: if bits in sequences are equal in a range, counts are equal.
pub proof fn lemma_bits_equal_in_seq_implies_count_equal(bits1: Seq<bool>, bits2: Seq<bool>, start: int, end: int)
    requires
        start <= end,
        bits1.len() == bits2.len(),
        forall|i: int| start <= i < end && 0 <= i < bits1.len() ==>
            bits1[i] == bits2[i],
    ensures
        count_set_bits_in_seq(bits1, start, end) == count_set_bits_in_seq(bits2, start, end),
    decreases end - start
{
    if start >= end {
    } else {
        lemma_bits_equal_in_seq_implies_count_equal(bits1, bits2, start + 1, end);
    }
}

/// Lemma: if all bits in sequence are false, count == 0.
pub proof fn lemma_all_zero_in_seq_implies_count_zero(bits: Seq<bool>, start: int, end: int)
    requires
        0 <= start <= end,
        forall|i: int| start <= i < end && 0 <= i < bits.len() ==> !bits[i],
    ensures
        count_set_bits_in_seq(bits, start, end) == 0,
    decreases end - start
{
    if start >= end {
    } else {
        lemma_all_zero_in_seq_implies_count_zero(bits, start + 1, end);
    }
}

/// Lemma: if all bits in range are set, count equals range size.
pub proof fn lemma_all_set_means_count_equals_size(bits: Seq<bool>, start: int, end: int)
    requires
        0 <= start <= end,
        end <= bits.len(),
        forall|i: int| start <= i < end ==> bits[i],
    ensures
        count_set_bits_in_seq(bits, start, end) == end - start,
    decreases end - start
{
    if start >= end {
    } else {
        lemma_all_set_means_count_equals_size(bits, start + 1, end);
    }
}

//==================================================================================================
// Lemmas: Bit-level Operations
//==================================================================================================

/// Lemma: Helper for proving bit operations on bytes (OR sets bit).
#[verifier(bit_vector)]
pub proof fn lemma_bit_or_sets_bit(old_byte: u8, shift: u8)
    requires
        shift < 8,
    ensures
        ((old_byte | (1u8 << shift)) & (1u8 << shift)) != 0,
{
}

/// Lemma: Helper for proving bit operations on bytes (OR preserves other bits).
#[verifier(bit_vector)]
pub proof fn lemma_bit_or_preserves_others(old_byte: u8, new_byte: u8, shift: u8, other_shift: u8)
    requires
        new_byte == (old_byte | (1u8 << shift)),
        shift < 8,
        other_shift < 8,
        shift != other_shift,
    ensures
        (new_byte & (1u8 << other_shift)) == (old_byte & (1u8 << other_shift)),
{
}

/// Lemma: Helper for proving bit clear operations on bytes (AND NOT clears bit).
#[verifier(bit_vector)]
pub proof fn lemma_bit_and_not_clears_bit(old_byte: u8, shift: u8)
    requires
        shift < 8,
    ensures
        ((old_byte & !(1u8 << shift)) & (1u8 << shift)) == 0,
{
}

/// Lemma: Helper for proving bit clear operations on bytes (AND NOT preserves other bits).
#[verifier(bit_vector)]
pub proof fn lemma_bit_and_not_preserves_others(old_byte: u8, new_byte: u8, shift: u8, other_shift: u8)
    requires
        new_byte == (old_byte & !(1u8 << shift)),
        shift < 8,
        other_shift < 8,
        shift != other_shift,
    ensures
        (new_byte & (1u8 << other_shift)) == (old_byte & (1u8 << other_shift)),
{
}

/// Lemma: when all raw bytes are zero, the corresponding bit is false.
#[verifier(bit_vector)]
pub proof fn lemma_zero_byte_has_no_bits(bit_idx: u8)
    requires
        bit_idx < 8,
    ensures
        (0u8 & (1u8 << bit_idx)) == 0,
{
}

//==================================================================================================
// Lemmas: View Properties
//==================================================================================================

/// Lemma: if view is empty, no bits are set.
pub proof fn lemma_is_empty_means_no_bits_set(view: &BitmapView)
    requires
        view.is_empty(),
    ensures
        forall|i: int| 0 <= i < view.number_of_bits() ==> !view.bits[i],
{
    if exists|i: int| 0 <= i < view.number_of_bits() && view.bits[i] {
        let i = choose|i: int| 0 <= i < view.number_of_bits() && view.bits[i];
        lemma_bit_set_in_seq_implies_count_geq_1(view.bits, 0, view.number_of_bits(), i);
    }
}

/// Lemma: if view is full, all bits are set.
pub proof fn lemma_is_full_means_all_bits_set(view: &BitmapView)
    requires
        view.is_full(),
    ensures
        forall|i: int| 0 <= i < view.number_of_bits() ==> view.bits[i],
{
    if exists|i: int| 0 <= i < view.number_of_bits() && !view.bits[i] {
        let i = choose|i: int| 0 <= i < view.number_of_bits() && !view.bits[i];
        lemma_bit_unset_in_seq_implies_count_lt_size(view.bits, 0, view.number_of_bits(), i);
    }
}

/// Lemma: if view is full, there are no free bits.
pub proof fn lemma_is_full_implies_no_free_bit(view: &BitmapView)
    requires
        view.is_full(),
    ensures
        !view.has_free_bit(),
{
    lemma_is_full_means_all_bits_set(view);
}

/// Lemma: if a bit is unset, then has_free_bit() is true.
pub proof fn lemma_unset_bit_implies_has_free_bit(view: &BitmapView, i: int)
    requires
        0 <= i < view.number_of_bits(),
        !view.bits[i],
    ensures
        view.has_free_bit(),
{
}

/// Lemma: if all bits are set, view is full.
pub proof fn lemma_all_bits_set_means_full(view: &BitmapView)
    requires
        forall|i: int| 0 <= i < view.number_of_bits() ==> view.bits[i],
    ensures
        view.is_full(),
{
    lemma_all_set_means_count_equals_size(view.bits, 0, view.number_of_bits());
}

//==================================================================================================
// Lemmas: Index Calculation
//==================================================================================================

/// Lemma: byte_index is bounded by number of bytes.
pub proof fn lemma_byte_index_bounded(bit_idx: int, num_bits: int, bytes_len: int)
    requires
        0 <= bit_idx < num_bits,
        num_bits == bytes_len * (u8::BITS as int),
        bytes_len > 0,
    ensures
        bit_idx / (u8::BITS as int) < bytes_len,
{
    let idx = bit_idx;
    let len = bytes_len;
    let bits = u8::BITS as int;
    assert(idx / bits < len) by (nonlinear_arith)
        requires idx < len * bits, bits > 0, len > 0
    {}
}

/// Lemma: bit_offset is always less than 8.
pub proof fn lemma_bit_offset_bounded(bit_idx: int)
    ensures
        0 <= bit_idx % (u8::BITS as int) < (u8::BITS as int),
{
}

/// Lemma: reconstructing bit index from byte_index and bit_offset.
pub proof fn lemma_index_reconstruction(bit_idx: int)
    requires
        bit_idx >= 0,
    ensures
        (bit_idx / (u8::BITS as int)) * (u8::BITS as int) + bit_idx % (u8::BITS as int) == bit_idx,
{
}

} // verus!
