// Copyright(c) The Maintainers of Nanvix.
// Licensed under the MIT License.

// Bitmap Verification Specifications
//
// This file contains complete Verus verification specifications for Bitmap.
// Includes BitmapView specs, proof lemmas, and tests.

use vstd::prelude::*;

verus! {

//==================================================================================================
// Bitmap View Implementation
//==================================================================================================

/// Implement View for Bitmap to map it to BitmapView.
impl View for Bitmap {
    type V = BitmapView;

    closed spec fn view(&self) -> BitmapView {
        BitmapView {
            bits: bits_to_seq(self.bits@, self.number_of_bits as int),
        }
    }
}

impl Bitmap {
    /// Invariant: the bitmap's number_of_bits must equal bits.len() * 8
    /// and must be less than u32::MAX.
    pub closed spec fn inv(&self) -> bool {
        &&& self@.number_of_bits() > 0
        &&& self@.number_of_bits() == self.bits@.len() * (u8::BITS as int)
        &&& self@.number_of_bits() < u32::MAX as int
        &&& self@.usage() <= self@.number_of_bits()
        &&& self.number_of_bits as int == self@.number_of_bits()
        &&& self.usage as int == self@.usage()
    }

    /// Weak invariant for use during allocation loop (usage field not yet updated).
    pub closed spec fn inv_weak(&self) -> bool {
        &&& self@.number_of_bits() > 0
        &&& self@.number_of_bits() == self.bits@.len() * (u8::BITS as int)
        &&& self@.number_of_bits() < u32::MAX as int
        &&& self@.usage() <= self@.number_of_bits()
        &&& self.number_of_bits as int == self@.number_of_bits()
    }

    /// Spec function: check if a bit at the given bit index is set.
    pub open spec fn is_bit_set_spec(&self, bit_index: int) -> bool {
        &&& 0 <= bit_index < self@.number_of_bits()
        &&& self@.bits[bit_index]
    }

    /// Spec function: check if all bits in range [start, end) are set.
    pub open spec fn all_bits_set_in_range_spec(&self, start: int, end: int) -> bool {
        forall|i: int| start <= i < end ==> self.is_bit_set_spec(i)
    }

    /// Spec function: check if all bits in range [start, end) are not set.
    pub open spec fn all_bits_unset_in_range_spec(&self, start: int, end: int) -> bool {
        forall|i: int| start <= i < end ==> !self.is_bit_set_spec(i)
    }

    /// Lemma: is_full implies no free bit.
    pub proof fn lemma_is_full_implies_no_free_bit_on_bitmap(&self)
        requires
            self.inv(),
            self@.is_full(),
        ensures
            !self@.has_free_bit(),
    {
        lemma_is_full_implies_no_free_bit(&self@);
    }

    /// Lemma: setting a byte bit reflects in the boolean view.
    pub(crate) proof fn lemma_byte_or_reflects_in_view(&self, new_self: &Self, word: int, bit: int)
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
    {
        lemma_bit_or_effects(self.bits@[word], bit, new_self.bits@[word]);
    }

    /// Lemma: setting a bit increases the count by 1.
    pub(crate) proof fn lemma_set_bit_increases_count(&self, new_self: &Self, index: int)
        requires
            0 <= index < self@.number_of_bits(),
            !self.is_bit_set_spec(index),
            new_self.is_bit_set_spec(index),
            forall|i: int| 0 <= i < self@.number_of_bits() && i != index ==>
                self.is_bit_set_spec(i) == new_self.is_bit_set_spec(i),
            self@.number_of_bits() == new_self@.number_of_bits(),
            self@.bits.len() == new_self@.bits.len(),
        ensures
            new_self@.usage() == self@.usage() + 1,
    {
        assert forall|i: int| 0 <= i < self@.number_of_bits() && i != index
        implies self@.bits[i] == new_self@.bits[i]
        by {
            assert(self.is_bit_set_spec(i) == new_self.is_bit_set_spec(i));
        };

        lemma_set_bit_increases_count_in_seq(self@.bits, new_self@.bits, 0, self@.number_of_bits(), index);
    }
}

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
pub open spec fn bits_to_seq(bytes: Seq<u8>, num_bits: int) -> Seq<bool> {
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
        let rest: int = count_set_bits_in_seq(bits, start + 1, end);
        if 0 <= start < bits.len() && bits[start] { rest + 1 } else { rest }
    }
}

/// Bitmap invariant specification.
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

/// Lemma: view bits equals bit_at for valid indices.
/// This connects the abstract view with the concrete byte representation.
pub(crate) proof fn lemma_view_bits_equals_bit_at(bm: &Bitmap, index: int)
    requires
        bm.inv(),
        0 <= index < bm@.number_of_bits(),
    ensures
        bm@.bits[index] == bit_at(bm.bits@, index),
{
    // By definition of bits_to_seq: bits_to_seq(bytes, n) = Seq::new(n, |i| bit_at(bytes, i))
    // So bm@.bits[index] = bits_to_seq(bm.bits@, bm.number_of_bits)[index] = bit_at(bm.bits@, index)
}

/// Lemma: view bits equals bit_at for valid indices (weak invariant version).
/// This connects the abstract view with the concrete byte representation.
pub(crate) proof fn lemma_view_bits_equals_bit_at_weak(bm: &Bitmap, index: int)
    requires
        bm.inv_weak(),
        0 <= index < bm@.number_of_bits(),
    ensures
        bm@.bits[index] == bit_at(bm.bits@, index),
{
    // By definition of bits_to_seq: bits_to_seq(bytes, n) = Seq::new(n, |i| bit_at(bytes, i))
    // So bm@.bits[index] = bits_to_seq(bm.bits@, bm.number_of_bits)[index] = bit_at(bm.bits@, index)
}

/// Lemma: is_bit_set_spec equals bit_at for valid indices.
pub(crate) proof fn lemma_is_bit_set_equals_bit_at(bm: &Bitmap, index: int)
    requires
        bm.inv(),
        0 <= index < bm@.number_of_bits(),
    ensures
        bm.is_bit_set_spec(index) == bit_at(bm.bits@, index),
{
    lemma_view_bits_equals_bit_at(bm, index);
}

/// Lemma: is_bit_set_spec equals bit_at for valid indices (weak invariant version).
pub(crate) proof fn lemma_is_bit_set_equals_bit_at_weak(bm: &Bitmap, index: int)
    requires
        bm.inv_weak(),
        0 <= index < bm@.number_of_bits(),
    ensures
        bm.is_bit_set_spec(index) == bit_at(bm.bits@, index),
{
    lemma_view_bits_equals_bit_at_weak(bm, index);
}

/// Lemma: is_bit_set_spec equals bit_at without requiring invariant.
/// Only requires structural properties that hold in the allocation loop.
pub(crate) proof fn lemma_is_bit_set_spec_equals_bit_at_minimal(bm: &Bitmap, index: int)
    requires
        bm@.number_of_bits() > 0,
        bm@.number_of_bits() == bm.bits@.len() * 8,
        bm.number_of_bits as int == bm@.number_of_bits(),
        0 <= index < bm@.number_of_bits(),
    ensures
        bm.is_bit_set_spec(index) == bit_at(bm.bits@, index),
{
    // is_bit_set_spec(index) = (0 <= index < number_of_bits) && self@.bits[index]
    // self@.bits = bits_to_seq(self.bits@, number_of_bits)
    // bits_to_seq[index] = bit_at(self.bits@, index)
    // So is_bit_set_spec(index) <=> bit_at(self.bits@, index) (given index in bounds)
}

/// Lemma: is_bit_set_spec equals bit_at for valid indices (minimal version).
/// This only requires basic structural properties, not the full invariant.
pub(crate) proof fn lemma_is_bit_set_equals_bit_at_basic(
    bits_seq: Seq<u8>,
    number_of_bits: int,
    index: int,
)
    requires
        number_of_bits > 0,
        number_of_bits == bits_seq.len() * 8,
        0 <= index < number_of_bits,
    ensures
        // bits_to_seq(bits_seq, number_of_bits)[index] == bit_at(bits_seq, index)
        bits_to_seq(bits_seq, number_of_bits)[index] == bit_at(bits_seq, index),
{
    // By definition: bits_to_seq(bytes, n) = Seq::new(n, |i| bit_at(bytes, i))
    // So bits_to_seq[index] = bit_at(bytes, index) by Seq::new property.
}

/// Lemma: If bit_at is true, then the raw byte has that bit set.
/// This connects the abstract bit_at to the concrete byte representation.
pub(crate) proof fn lemma_is_bit_set_implies_byte_bit_set(
    bits_seq: Seq<u8>,
    number_of_bits: int,
    bit_index: int,
)
    requires
        number_of_bits > 0,
        number_of_bits == bits_seq.len() * 8,
        0 <= bit_index < number_of_bits,
        bit_at(bits_seq, bit_index),
    ensures
        ({
            let word: int = bit_index / 8;
            let bit: int = bit_index % 8;
            (bits_seq[word] & (1u8 << (bit as u8))) != 0
        }),
{
    // By definition of bit_at:
    // bit_at(bytes, i) = (bytes[i/8] & (1 << (i%8))) != 0
    // Since bit_at is true, the byte has the bit set.
}

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

/// Lemma: bit_at only depends on the byte at index i/8.
/// If two byte sequences have the same byte at position i/8, then bit_at gives the same result.
pub proof fn lemma_bit_at_depends_only_on_relevant_byte(
    bytes1: Seq<u8>,
    bytes2: Seq<u8>,
    bit_index: int,
)
    requires
        bit_index >= 0,
        bit_index / 8 < bytes1.len() as int,
        bit_index / 8 < bytes2.len() as int,
        bytes1[bit_index / 8] == bytes2[bit_index / 8],
    ensures
        bit_at(bytes1, bit_index) == bit_at(bytes2, bit_index),
{
    // By definition: bit_at(bytes, i) = (bytes[i/8] & (1 << (i%8))) != 0
    // Since bytes1[i/8] == bytes2[i/8], the expressions are identical.
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

/// Lemma: Helper for proving bit operations on bytes (OR effects).
pub proof fn lemma_bit_or_effects(old_byte: u8, bit_pos: int, new_byte: u8)
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

/// Lemma: Helper for proving bit clear operations on bytes (AND NOT effects).
pub proof fn lemma_bit_and_not_effects(old_byte: u8, bit_pos: int, new_byte: u8)
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

/// Lemma: After setting a bit in a byte array, bit_at reflects the change.
/// This connects RawArray::set to the bit-level view.
pub proof fn lemma_set_byte_updates_bit_at(
    old_bytes: Seq<u8>,
    new_bytes: Seq<u8>,
    word_idx: int,
    bit_in_word: int,
    num_bits: int,
)
    requires
        0 <= word_idx < old_bytes.len(),
        0 <= bit_in_word < 8,
        new_bytes.len() == old_bytes.len(),
        new_bytes[word_idx] == (old_bytes[word_idx] | (1u8 << bit_in_word)),
        forall|i: int| 0 <= i < old_bytes.len() && i != word_idx ==> new_bytes[i] == old_bytes[i],
        num_bits > 0,
        num_bits <= old_bytes.len() * 8,
    ensures
        // The target bit is now set
        bit_at(new_bytes, word_idx * 8 + bit_in_word),
        // All other bits unchanged
        forall|i: int| #![trigger bit_at(new_bytes, i), bit_at(old_bytes, i)]
            0 <= i < num_bits && i != word_idx * 8 + bit_in_word ==>
            bit_at(new_bytes, i) == bit_at(old_bytes, i),
{
    let target_idx: int = word_idx * 8 + bit_in_word;

    // Prove the target bit is set
    let shift: u8 = bit_in_word as u8;
    let old_byte: u8 = old_bytes[word_idx];
    let new_byte: u8 = new_bytes[word_idx];
    assert((new_byte & (1u8 << shift)) != 0) by (bit_vector)
        requires
            new_byte == (old_byte | (1u8 << shift)),
            shift < 8,
    ;

    // Prove other bits unchanged
    assert forall|i: int| #![trigger bit_at(new_bytes, i), bit_at(old_bytes, i)]
        0 <= i < num_bits && i != target_idx implies
        bit_at(new_bytes, i) == bit_at(old_bytes, i)
    by {
        let w: int = i / 8;
        let b: int = i % 8;
        let other_shift: u8 = b as u8;

        if w == word_idx {
            // Same word, different bit
            assert(b != bit_in_word);
            assert(other_shift != shift);
            assert((new_byte & (1u8 << other_shift)) == (old_byte & (1u8 << other_shift))) by (bit_vector)
                requires
                    new_byte == (old_byte | (1u8 << shift)),
                    shift < 8,
                    other_shift < 8,
                    shift != other_shift,
            ;
        } else {
            // Different word, unchanged
            assert(new_bytes[w] == old_bytes[w]);
        }
    }
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
        let i: int = choose|i: int| 0 <= i < view.number_of_bits() && view.bits[i];
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
        let i: int = choose|i: int| 0 <= i < view.number_of_bits() && !view.bits[i];
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

/// Lemma: if view is not full, there exists at least one unset bit.
pub proof fn lemma_not_full_means_exists_unset_bit(view: &BitmapView)
    requires
        !view.is_full(),
    ensures
        exists|i: int| 0 <= i < view.number_of_bits() && !view.bits[i],
{
    if forall|i: int| 0 <= i < view.number_of_bits() ==> view.bits[i] {
        lemma_all_bits_set_means_full(view);
    }
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
    let idx: int = bit_idx;
    let len: int = bytes_len;
    let bits: int = u8::BITS as int;
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

//==================================================================================================
// Tests
//==================================================================================================

#[cfg(verus_keep_ghost)]
mod test {
    use super::*;

    /// Test: BitmapView usage calculation.
    proof fn test_bitmap_view_usage() {
        let view: BitmapView = BitmapView {
            bits: seq![false, true, false, true, true, false, false, false],
        };

        assert(view.number_of_bits() == 8);
        // 3 bits set at indices 1, 3, 4.
        lemma_count_set_bits_in_seq_bounded(view.bits, 0, 8);
    }

    /// Test: Empty view has no bits set.
    proof fn test_empty_view() {
        let view: BitmapView = BitmapView {
            bits: seq![false, false, false, false, false, false, false, false],
        };

        lemma_all_zero_in_seq_implies_count_zero(view.bits, 0, 8);
        assert(view.is_empty());
        lemma_is_empty_means_no_bits_set(&view);
    }

    /// Test: Full view has all bits set.
    proof fn test_full_view() {
        let view: BitmapView = BitmapView {
            bits: seq![true, true, true, true, true, true, true, true],
        };

        lemma_all_set_means_count_equals_size(view.bits, 0, 8);
        assert(view.is_full());
        lemma_is_full_means_all_bits_set(&view);
    }

    /// Test: Full view has no free bit.
    proof fn test_full_no_free_bit() {
        let view: BitmapView = BitmapView {
            bits: seq![true, true, true, true, true, true, true, true],
        };

        lemma_all_set_means_count_equals_size(view.bits, 0, 8);
        lemma_is_full_implies_no_free_bit(&view);
        assert(!view.has_free_bit());
    }

    /// Test: Setting a bit increases count.
    proof fn test_set_bit_increases_count() {
        let old_bits: Seq<bool> = seq![false, false, false, false];
        let new_bits: Seq<bool> = seq![false, true, false, false];

        lemma_set_bit_increases_count_in_seq(old_bits, new_bits, 0, 4, 1);
        assert(count_set_bits_in_seq(new_bits, 0, 4) == count_set_bits_in_seq(old_bits, 0, 4) + 1);
    }

    /// Test: Clearing a bit decreases count.
    proof fn test_clear_bit_decreases_count() {
        let old_bits: Seq<bool> = seq![false, true, false, false];
        let new_bits: Seq<bool> = seq![false, false, false, false];

        lemma_clear_bit_decreases_count_in_seq(old_bits, new_bits, 0, 4, 1);
        assert(count_set_bits_in_seq(new_bits, 0, 4) == count_set_bits_in_seq(old_bits, 0, 4) - 1);
    }

    /// Test: bit_at function.
    proof fn test_bit_at() {
        let bytes: Seq<u8> = seq![0b00000010u8];  // Bit 1 is set.

        // bit_at extracts the correct bit value.
        // Bit 0 is not set (0b00000010 & 0b00000001 == 0).
        assert((0b00000010u8 & (1u8 << 0u8)) == 0) by (bit_vector);
        assert(bit_at(bytes, 0) == false);

        // Bit 1 is set (0b00000010 & 0b00000010 != 0).
        assert((0b00000010u8 & (1u8 << 1u8)) != 0) by (bit_vector);
        assert(bit_at(bytes, 1) == true);
    }

    /// Test: Byte index bounded.
    proof fn test_byte_index_bounded() {
        lemma_byte_index_bounded(7, 8, 1);
        lemma_byte_index_bounded(15, 16, 2);
        lemma_byte_index_bounded(63, 64, 8);
    }

    /// Test: Bit offset bounded.
    proof fn test_bit_offset_bounded() {
        lemma_bit_offset_bounded(0);
        lemma_bit_offset_bounded(7);
        lemma_bit_offset_bounded(8);
        lemma_bit_offset_bounded(15);
    }

    /// Test: Index reconstruction.
    proof fn test_index_reconstruction() {
        lemma_index_reconstruction(0);
        lemma_index_reconstruction(7);
        lemma_index_reconstruction(8);
        lemma_index_reconstruction(15);
        lemma_index_reconstruction(100);
    }

    /// Test: Unset bit implies has_free_bit.
    proof fn test_unset_bit_implies_free() {
        let view: BitmapView = BitmapView {
            bits: seq![true, true, false, true, true, true, true, true],
        };

        lemma_unset_bit_implies_has_free_bit(&view, 2);
        assert(view.has_free_bit());
    }

    /// Test: All bits set means full.
    proof fn test_all_bits_set_means_full() {
        let view: BitmapView = BitmapView {
            bits: seq![true, true, true, true],
        };

        lemma_all_bits_set_means_full(&view);
        assert(view.is_full());
    }

    /// Test: bits_to_seq creates correct sequence.
    proof fn test_bits_to_seq() {
        let bytes: Seq<u8> = seq![0b00000000u8];
        let bits: Seq<bool> = bits_to_seq(bytes, 8);

        assert(bits.len() == 8);
        // All bits should be false since byte is 0.
        assert forall|i: int| 0 <= i < 8 implies !bits[i]
        by {
            let idx: u8 = i as u8;
            assert((0u8 & (1u8 << idx)) == 0) by (bit_vector)
                requires 0 <= idx < 8;
        }
    }

    /// Test: bit_or_effects lemma.
    proof fn test_bit_or_effects() {
        let old_byte: u8 = 0b00000000u8;
        let shift: u8 = 2u8;
        // new_byte = old_byte | (1 << 2) = 0 | 4 = 4 = 0b00000100.
        assert((0b00000000u8 | (1u8 << 2u8)) == 0b00000100u8) by (bit_vector);
        let new_byte: u8 = (old_byte | (1u8 << shift));
        lemma_bit_or_effects(old_byte, 2, new_byte);
    }

    /// Test: bit_and_not_effects lemma.
    proof fn test_bit_and_not_effects() {
        let old_byte: u8 = 0b11111111u8;
        let shift: u8 = 2u8;
        // new_byte = old_byte & !(1 << 2) = 255 & ~4 = 255 & 251 = 251 = 0b11111011.
        assert((0b11111111u8 & !(1u8 << 2u8)) == 0b11111011u8) by (bit_vector);
        let new_byte: u8 = (old_byte & !(1u8 << shift));
        lemma_bit_and_not_effects(old_byte, 2, new_byte);
    }

    /// Test: not_full_means_exists_unset_bit lemma.
    proof fn test_not_full_exists_unset() {
        let view: BitmapView = BitmapView {
            bits: seq![true, true, false, true],  // One bit unset.
        };

        // First prove that the view is not full.
        // Count is 3, size is 4.
        lemma_bit_unset_in_seq_implies_count_lt_size(view.bits, 0, 4, 2);
        assert(!view.is_full());
        lemma_not_full_means_exists_unset_bit(&view);
    }

    /// Test: Multiple bits set.
    proof fn test_multiple_bits_set() {
        let bits: Seq<bool> = seq![true, false, true, false, true, false, true, false];

        // Count should be 4 (bits 0, 2, 4, 6 are set).
        lemma_count_set_bits_in_seq_bounded(bits, 0, 8);
        lemma_bit_set_in_seq_implies_count_geq_1(bits, 0, 8, 0);
    }

    /// Test: Count in subrange.
    proof fn test_count_subrange() {
        let bits: Seq<bool> = seq![true, true, true, true, false, false, false, false];

        // Count in [0, 4) should be 4.
        lemma_all_set_means_count_equals_size(bits, 0, 4);
        assert(count_set_bits_in_seq(bits, 0, 4) == 4);

        // Count in [4, 8) should be 0.
        lemma_all_zero_in_seq_implies_count_zero(bits, 4, 8);
        assert(count_set_bits_in_seq(bits, 4, 8) == 0);
    }
}

} // verus!
