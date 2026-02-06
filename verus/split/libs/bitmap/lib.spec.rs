// Copyright(c) The Maintainers of Nanvix.
// Licensed under the MIT License.

// Bitmap - Specifications
//
// This file contains specification functions, BitmapView, and View trait for Bitmap.

use vstd::prelude::*;

verus! {

//==================================================================================================
// BitmapView - Abstract Specification Model
//==================================================================================================

/// A view of the Bitmap as a sequence of booleans.
#[verifier::ext_equal]
pub struct BitmapView {
    pub bits: Seq<bool>,
    /// Set of indices where bits are set. Used for efficient frame conditions.
    pub set_bits: Set<int>,
}

impl BitmapView {
    /// Returns the number of bits in the bitmap view.
    pub open spec fn number_of_bits(&self) -> int {
        self.bits.len() as int
    }

    /// Returns the usage (count of set bits) in the bitmap view.
    pub open spec fn usage(&self) -> int {
        Bitmap::count_set_bits_in_seq(self.bits, 0, self.bits.len() as int)
    }

    /// Returns the count of free (unset) bits.
    pub open spec fn count_free(&self) -> int {
        self.number_of_bits() - self.usage()
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

    /// Helper: Create a set of indices in range [start, end).
    pub open spec fn range_set(start: int, end: int) -> Set<int> {
        Set::new(|i: int| start <= i < end)
    }
}

//==================================================================================================
// View Implementation for Bitmap
//==================================================================================================

impl View for Bitmap {
    type V = BitmapView;

    closed spec fn view(&self) -> BitmapView {
        BitmapView {
            bits: Self::bits_to_seq(self.bits@, self.number_of_bits as int),
            set_bits: Set::new(|i: int| 0 <= i < self.number_of_bits as int && Self::bit_at(self.bits@, i)),
        }
    }
}

//==================================================================================================
// Bitmap Specification Functions
//==================================================================================================

impl Bitmap {
    /// Helper spec function: convert RawArray<u8> to Seq<bool>.
    pub open spec fn bits_to_seq(bytes: Seq<u8>, num_bits: int) -> Seq<bool>
        decreases num_bits
    {
        Seq::new(num_bits as nat, |i: int| Self::bit_at(bytes, i))
    }

    /// Helper spec function: get the bit value at a specific index.
    pub open spec fn bit_at(bytes: Seq<u8>, bit_index: int) -> bool {
        let word: int = bit_index / (u8::BITS as int);
        let bit: int = bit_index % (u8::BITS as int);
        if 0 <= bit_index && word < bytes.len() {
            (bytes[word] & (1u8 << bit)) != 0
        } else {
            false
        }
    }

    /// Helper spec function: check if a bit at the given bit index is set.
    pub open spec fn is_bit_set(&self, bit_index: int) -> bool {
        &&& 0 <= bit_index < self@.number_of_bits()
        &&& self@.bits[bit_index]
    }

    /// Helper spec function: check if all bits in range [start, end) are set.
    pub open spec fn all_bits_set_in_range(&self, start: int, end: int) -> bool {
        forall|i: int| start <= i < end ==> self.is_bit_set(i)
    }

    /// Helper spec function: check if all bits in range [start, end) are not set.
    pub open spec fn all_bits_unset_in_range(&self, start: int, end: int) -> bool {
        forall|i: int| start <= i < end ==> !self.is_bit_set(i)
    }

    /// Helper spec function: check if there exists a contiguous range of n free bits starting at start.
    pub open spec fn has_free_range_at(&self, start: int, n: int) -> bool {
        &&& 0 <= start
        &&& start + n <= self@.number_of_bits()
        &&& self.all_bits_unset_in_range(start, start + n)
    }

    /// Helper spec function: check if there exists a contiguous range of n free bits.
    pub open spec fn exists_contiguous_free_range(&self, n: int) -> bool {
        exists|start: int| #![trigger self.has_free_range_at(start, n)]
            self.has_free_range_at(start, n)
    }

    /// Helper spec function: count set bits in a range [start, end) of a sequence.
    pub closed spec fn count_set_bits_in_seq(bits: Seq<bool>, start: int, end: int) -> int
        decreases end - start when end >= start
    {
        if start >= end {
            0
        } else {
            let rest: int = Self::count_set_bits_in_seq(bits, start + 1, end);
            if 0 <= start < bits.len() && bits[start] { rest + 1 } else { rest }
        }
    }

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
}

} // verus!
