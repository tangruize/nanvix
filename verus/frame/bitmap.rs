// Copyright(c) The Maintainers of Nanvix.
// Licensed under the MIT License.

//==================================================================================================
// Bitmap Allocator (Trusted Dependency)
//==================================================================================================

use vstd::prelude::*;
use crate::error::Error;
use crate::frame_address::MAX_FRAME_NUMBER;

verus! {

/// Raw array stub for external storage.
/// This is a trusted type representing raw byte storage.
#[verifier::external_body]
pub struct RawArray {
    capacity: usize,
}

impl RawArray {
    /// Spec function to get capacity.
    pub uninterp spec fn spec_capacity(&self) -> int;
}

/// Bitmap view for specification.
#[verifier::ext_equal]
pub ghost struct BitmapView {
    pub bits: Seq<bool>,
}

impl BitmapView {
    pub open spec fn number_of_bits(&self) -> int {
        self.bits.len() as int
    }

    pub open spec fn is_bit_set(&self, index: int) -> bool {
        self.bits[index]
    }

    /// Returns true if there exists at least one unset bit.
    pub open spec fn has_free_bit(&self) -> bool {
        exists|i: int| 0 <= i < self.number_of_bits() && !self.is_bit_set(i)
    }

    /// Returns true if all bits are set (no free bits).
    pub open spec fn is_full(&self) -> bool {
        forall|i: int| 0 <= i < self.number_of_bits() ==> self.is_bit_set(i)
    }

    /// Count of allocated (set) bits.
    pub open spec fn count_allocated(&self) -> int {
        self.bits.filter(|b: bool| b).len() as int
    }

    /// Count of free (unset) bits.
    pub open spec fn count_free(&self) -> int {
        self.number_of_bits() - self.count_allocated()
    }
}

/// Bitmap allocator for managing frame allocation.
#[derive(Debug)]
#[verifier::ext_equal]
pub struct Bitmap {
    /// Number of bits in the bitmap.
    number_of_bits: usize,
    /// Usage count.
    usage: usize,
}

impl View for Bitmap {
    type V = BitmapView;
    uninterp spec fn view(&self) -> BitmapView;
}

impl Bitmap {
    /// Invariant for Bitmap.
    pub closed spec fn inv(&self) -> bool {
        &&& self.number_of_bits > 0
        &&& self@.number_of_bits() == self.number_of_bits as int
        &&& self@.number_of_bits() <= (usize::MAX as int)
    }

    /// Spec function: is bit at index set?
    pub open spec fn is_bit_set(&self, index: int) -> bool {
        self@.is_bit_set(index)
    }

    #[verifier::external_body]
    pub fn number_of_bits(&self) -> (result: usize)
        requires self.inv(),
        ensures result as int == self@.number_of_bits()
    {
        self.number_of_bits
    }

    /// Allocate the first free bit, returning its index.
    #[verifier::external_body]
    pub fn alloc(&mut self) -> (result: Result<usize, Error>)
        requires old(self).inv(),
        ensures
            self.inv(),
            result is Ok ==> {
                let index = result->Ok_0 as int;
                &&& 0 <= index < self@.number_of_bits()
                &&& self@.number_of_bits() == old(self)@.number_of_bits()
                &&& self.is_bit_set(index)
                &&& !old(self).is_bit_set(index)
                &&& forall|i: int| 0 <= i < self@.number_of_bits() && i != index ==>
                    self.is_bit_set(i) == old(self).is_bit_set(i)
                // Count change: exactly one more bit is set.
                &&& self@.count_allocated() == old(self)@.count_allocated() + 1
            },
            result is Err ==> self@ == old(self)@,
            // Liveness: if there's a free bit, alloc succeeds.
            old(self)@.has_free_bit() ==> result is Ok,
            // If alloc fails, bitmap was full.
            result is Err ==> old(self)@.is_full(),
    {
        unimplemented!()
    }

    /// Set a specific bit (mark as allocated).
    ///
    /// # Liveness
    ///
    /// This operation always succeeds when preconditions are met (index in range).
    #[verifier::external_body]
    pub fn set(&mut self, index: usize) -> (result: Result<(), Error>)
        requires
            old(self).inv(),
            (index as int) < old(self)@.number_of_bits(),
        ensures
            self.inv(),
            // Liveness: operation always succeeds when preconditions are met.
            result is Ok,
            // Postcondition for success (always true given liveness).
            self@.number_of_bits() == old(self)@.number_of_bits(),
            self.is_bit_set(index as int),
            forall|i: int| 0 <= i < self@.number_of_bits() && i != index as int ==>
                self.is_bit_set(i) == old(self).is_bit_set(i),
            // Count change: if bit was not set, count increases by 1.
            !old(self).is_bit_set(index as int) ==>
                self@.count_allocated() == old(self)@.count_allocated() + 1,
            // If bit was already set, count unchanged.
            old(self).is_bit_set(index as int) ==>
                self@.count_allocated() == old(self)@.count_allocated(),
    {
        unimplemented!()
    }

    /// Clear a specific bit (mark as free).
    ///
    /// # Liveness
    ///
    /// This operation always succeeds when preconditions are met (index in range and bit set).
    #[verifier::external_body]
    pub fn clear(&mut self, index: usize) -> (result: Result<(), Error>)
        requires
            old(self).inv(),
            (index as int) < old(self)@.number_of_bits(),
            old(self).is_bit_set(index as int),
        ensures
            self.inv(),
            // Liveness: operation always succeeds when preconditions are met.
            result is Ok,
            // Postcondition for success (always true given liveness).
            !self.is_bit_set(index as int),
            self@.number_of_bits() == old(self)@.number_of_bits(),
            forall|i: int| 0 <= i < self@.number_of_bits() && i != index as int ==>
                self.is_bit_set(i) == old(self).is_bit_set(i),
            // Count change: count decreases by 1 (since bit was set and is now clear).
            self@.count_allocated() == old(self)@.count_allocated() - 1,
    {
        unimplemented!()
    }

    /// Test if a specific bit is set.
    ///
    /// # Liveness
    ///
    /// This operation always succeeds when preconditions are met (index in range).
    #[verifier::external_body]
    pub fn test(&self, index: usize) -> (result: Result<bool, Error>)
        requires
            self.inv(),
            (index as int) < self@.number_of_bits(),
        ensures
            // Liveness: always succeeds when preconditions are met.
            result is Ok,
            result->Ok_0 == self.is_bit_set(index as int),
    {
        unimplemented!()
    }

    /// Create a new bitmap with managed storage.
    #[verifier::external_body]
    pub fn new_managed(number_of_bits: usize) -> (result: Result<Bitmap, Error>)
        requires
            number_of_bits > 0,
            number_of_bits < i32::MAX as usize,
        ensures
            result is Ok ==> {
                let bmp = result->Ok_0;
                &&& bmp.inv()
                &&& bmp@.number_of_bits() == number_of_bits as int
                &&& forall|i: int| 0 <= i < number_of_bits as int ==> !bmp.is_bit_set(i)
            },
    {
        unimplemented!()
    }

    /// Create a bitmap from raw storage.
    /// The raw storage provides the backing memory for the bitmap.
    /// The bitmap state depends on the initial content of the storage.
    #[verifier::external_body]
    pub fn from_raw_array(storage: RawArray, number_of_bits: usize) -> (result: Result<Bitmap, Error>)
        requires
            number_of_bits > 0,
            number_of_bits as int <= MAX_FRAME_NUMBER as int + 1,
            // Storage must have enough capacity for the bits.
            storage.spec_capacity() >= ((number_of_bits + 7) / 8) as int,
        ensures
            result is Ok ==> {
                let bmp = result->Ok_0;
                &&& bmp.inv()
                &&& bmp@.number_of_bits() == number_of_bits as int
                // Note: bit values depend on storage content; not specified here.
            },
    {
        unimplemented!()
    }
}

} // verus!
