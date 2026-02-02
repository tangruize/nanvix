// Copyright(c) The Maintainers of Nanvix.
// Licensed under the MIT License.

// Bitmap Verification Specifications
//
// This file contains Verus verification specifications, proofs, and lemmas
// for the Bitmap module.

use vstd::prelude::*;

verus! {

//==================================================================================================
// Constants
//==================================================================================================

pub open spec const BITS_PER_BYTE: nat = 8;

//==================================================================================================
// BitmapView - The Abstract Specification Model
//==================================================================================================

/// Abstract view of a Bitmap as a sequence of booleans.
/// Each boolean represents whether a bit is set (true) or clear (false).
pub ghost struct BitmapView {
    /// The logical contents of the bitmap as a sequence of bits.
    pub bits: Seq<bool>,
    /// Number of bits that are set (true).
    pub usage: nat,
}

impl BitmapView {
    /// Returns the number of bits in the bitmap.
    pub open spec fn len(&self) -> nat {
        self.bits.len()
    }

    /// Returns the bit at index i.
    pub open spec fn get(&self, i: int) -> bool {
        self.bits[i]
    }

    /// Returns a new view with the bit at index i set to value.
    pub open spec fn set_bit(&self, i: int, value: bool) -> BitmapView {
        BitmapView {
            bits: self.bits.update(i, value),
            usage: if value && !self.bits[i] {
                self.usage + 1
            } else if !value && self.bits[i] {
                (self.usage - 1) as nat
            } else {
                self.usage
            },
        }
    }

    /// Invariant: usage equals the count of set bits.
    pub open spec fn usage_inv(&self) -> bool {
        self.usage == self.count_set_bits()
    }

    /// Counts the number of set bits in the bitmap.
    pub open spec fn count_set_bits(&self) -> nat {
        self.count_set_bits_range(0, self.bits.len() as int)
    }

    /// Counts the number of set bits in the range [start, end).
    pub open spec fn count_set_bits_range(&self, start: int, end: int) -> nat
        decreases end - start
    {
        if start >= end {
            0
        } else if self.bits[start] {
            1 + self.count_set_bits_range(start + 1, end)
        } else {
            self.count_set_bits_range(start + 1, end)
        }
    }
}

//==================================================================================================
// Bit Index Specifications
//==================================================================================================

/// Computes the byte index for a given bit index.
pub open spec fn byte_index(bit_idx: nat) -> nat {
    bit_idx / BITS_PER_BYTE
}

/// Computes the bit offset within a byte for a given bit index.
pub open spec fn bit_offset(bit_idx: nat) -> nat {
    bit_idx % BITS_PER_BYTE
}

/// Specification: index_unchecked returns the correct (word, bit) pair.
pub open spec fn spec_index_unchecked(index: nat) -> (nat, nat) {
    (byte_index(index), bit_offset(index))
}

//==================================================================================================
// Lemmas about Bit Indexing
//==================================================================================================

/// Lemma: byte_index is bounded by number of bytes.
pub proof fn lemma_byte_index_bounded(bit_idx: nat, num_bits: nat)
    requires
        bit_idx < num_bits,
        num_bits % BITS_PER_BYTE == 0,
    ensures
        byte_index(bit_idx) < num_bits / BITS_PER_BYTE,
{
}

/// Lemma: bit_offset is always less than 8.
pub proof fn lemma_bit_offset_bounded(bit_idx: nat)
    ensures
        bit_offset(bit_idx) < BITS_PER_BYTE,
{
}

/// Lemma: reconstructing bit index from byte_index and bit_offset.
pub proof fn lemma_index_reconstruction(bit_idx: nat)
    ensures
        byte_index(bit_idx) * BITS_PER_BYTE + bit_offset(bit_idx) == bit_idx,
{
}

//==================================================================================================
// Lemmas about BitmapView
//==================================================================================================

/// Lemma: Setting a bit preserves the length.
pub proof fn lemma_set_preserves_len(view: BitmapView, i: int, value: bool)
    requires
        0 <= i < view.len() as int,
    ensures
        view.set_bit(i, value).len() == view.len(),
{
}

/// Lemma: Setting a bit only changes that bit.
pub proof fn lemma_set_only_changes_target(view: BitmapView, i: int, j: int, value: bool)
    requires
        0 <= i < view.len() as int,
        0 <= j < view.len() as int,
        i != j,
    ensures
        view.set_bit(i, value).get(j) == view.get(j),
{
}

/// Lemma: Setting a bit sets it to the new value.
pub proof fn lemma_set_changes_target(view: BitmapView, i: int, value: bool)
    requires
        0 <= i < view.len() as int,
    ensures
        view.set_bit(i, value).get(i) == value,
{
}

//==================================================================================================
// Allocation Specifications
//==================================================================================================

/// Predicate: A range [start, start+size) is all clear (available for allocation).
pub open spec fn range_is_clear(view: &BitmapView, start: int, size: int) -> bool {
    forall|i: int| start <= i < start + size ==> !view.get(i)
}

/// Predicate: A range [start, start+size) is all set.
pub open spec fn range_is_set(view: &BitmapView, start: int, size: int) -> bool {
    forall|i: int| start <= i < start + size ==> view.get(i)
}

//==================================================================================================
// Bitmap Invariant
//==================================================================================================

/// The invariant that all Bitmap instances must satisfy.
pub open spec fn bitmap_inv(num_bits: nat, usage: nat, bytes_len: nat) -> bool {
    &&& num_bits > 0
    &&& num_bits < u32::MAX as nat
    &&& num_bits % BITS_PER_BYTE == 0
    &&& bytes_len == num_bits / BITS_PER_BYTE
    &&& usage <= num_bits
}

//==================================================================================================
// Bit Manipulation Lemmas (using bit_vector solver)
//==================================================================================================

/// Lemma: Setting bit b in byte w using OR.
#[verifier(bit_vector)]
pub proof fn lemma_bit_set(w: u8, b: u8)
    requires
        b < 8,
    ensures
        ((w | (1u8 << b)) >> b) & 1u8 == 1u8,
{
}

/// Lemma: Clearing bit b in byte w using AND NOT.
#[verifier(bit_vector)]
pub proof fn lemma_bit_clear(w: u8, b: u8)
    requires
        b < 8,
    ensures
        ((w & sub(0xffu8, 1u8 << b)) >> b) & 1u8 == 0u8,
{
}

/// Lemma: OR is idempotent when bit is already set.
#[verifier(bit_vector)]
pub proof fn lemma_or_idempotent(a: u8, b: u8)
    ensures
        a | a == a,
        a | b == b | a,
{
}

/// Lemma: AND is idempotent.
#[verifier(bit_vector)]
pub proof fn lemma_and_idempotent(a: u8, b: u8)
    ensures
        a & a == a,
        a & b == b & a,
{
}

/// Lemma: OR with zero is identity.
#[verifier(bit_vector)]
pub proof fn lemma_or_zero(a: u8)
    ensures
        a | 0u8 == a,
        0u8 | a == a,
{
}

/// Lemma: AND with all ones is identity.
#[verifier(bit_vector)]
pub proof fn lemma_and_ones(a: u8)
    ensures
        a & 0xffu8 == a,
        0xffu8 & a == a,
{
}

/// Lemma: OR with all ones is all ones.
#[verifier(bit_vector)]
pub proof fn lemma_or_ones(a: u8)
    ensures
        a | 0xffu8 == 0xffu8,
        0xffu8 | a == 0xffu8,
{
}

/// Lemma: AND with zero is zero.
#[verifier(bit_vector)]
pub proof fn lemma_and_zero(a: u8)
    ensures
        a & 0u8 == 0u8,
        0u8 & a == 0u8,
{
}

} // verus!
