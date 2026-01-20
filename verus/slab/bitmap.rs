// Copyright(c) The Maintainers of Nanvix.
// Licensed under the MIT License.

//==================================================================================================
// Bitmap Allocator (Trusted Dependency)
//==================================================================================================

use vstd::prelude::*;
use crate::error::Error;
use crate::raw_array::RawArray;

verus! {

/// Bitmap view for specification.
#[verifier::ext_equal]
pub ghost struct BitmapView {
    pub bits: Seq<bool>,
}

impl BitmapView {
    pub open spec fn number_of_bits(&self) -> int {
        self.bits.len() as int
    }

    /// Returns the number of bytes required to store the bitmap.
    /// This is ceil(number_of_bits / 8).
    pub open spec fn number_of_bytes(&self) -> int {
        (self.number_of_bits() + 7) / 8
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
}

/// Bitmap allocator for managing block allocation.
#[derive(Debug)]
#[verifier::ext_equal]
pub struct Bitmap {
    /// Number of bits in the bitmap.
    number_of_bits: usize,
    /// Usage count.
    usage: usize,
    /// Storage for bitmap bytes.
    bits: RawArray<u8>,
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
        // number_of_bits is stored as usize, so it's bounded.
        &&& self@.number_of_bits() <= (usize::MAX as int)
    }

    /// Lemma: Bitmap invariant implies number_of_bits is bounded by usize::MAX.
    pub proof fn lemma_number_of_bits_bounded(&self)
        requires
            self.inv(),
        ensures
            self@.number_of_bits() <= (usize::MAX as int),
    {
        // Follows directly from the definition of inv().
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
            },
            result is Err ==> self@ == old(self)@,
            // Issue 3 FIX: Liveness - if there's a free bit, alloc succeeds.
            old(self)@.has_free_bit() ==> result is Ok,
            // Conversely, if alloc fails, bitmap was full.
            result is Err ==> old(self)@.is_full(),
    {
        unimplemented!()
    }

    #[verifier::external_body]
    pub fn set(&mut self, index: usize) -> (result: Result<(), Error>)
        requires 
            old(self).inv(),
            (index as int) < old(self)@.number_of_bits(),
        ensures
            self.inv(),
            result is Ok ==> {
                &&& self@.number_of_bits() == old(self)@.number_of_bits()
                &&& self.is_bit_set(index as int)
                &&& forall|i: int| 0 <= i < self@.number_of_bits() && i != index as int ==>
                    self.is_bit_set(i) == old(self).is_bit_set(i)
            },
            result is Err ==> self@ == old(self)@,
    {
        unimplemented!()
    }

    #[verifier::external_body]
    pub fn clear(&mut self, index: usize) -> (result: Result<(), Error>)
        requires 
            old(self).inv(),
            (index as int) < old(self)@.number_of_bits(),
            old(self).is_bit_set(index as int),
        ensures
            self.inv(),
            result is Ok ==> {
                &&& !self.is_bit_set(index as int)
                &&& self@.number_of_bits() == old(self)@.number_of_bits()
                &&& forall|i: int| 0 <= i < self@.number_of_bits() && i != index as int ==>
                    self.is_bit_set(i) == old(self).is_bit_set(i)
            },
            result is Err ==> self@ == old(self)@,
    {
        unimplemented!()
    }

    #[verifier::external_body]
    pub fn test(&self, index: usize) -> (result: Result<bool, Error>)
        requires
            self.inv(),
            (index as int) < self@.number_of_bits(),
        ensures
            result is Ok ==> result->Ok_0 == self.is_bit_set(index as int),
    {
        unimplemented!()
    }

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

    #[verifier::external_body]
    pub fn from_raw_array(storage: RawArray<u8>, number_of_bits: usize) -> (result: Bitmap)
        requires 
            storage@.len() > 0,
            number_of_bits > 0,
            number_of_bits <= storage@.len() * 8,
            // Storage is zeroed.
            forall|i: int| 0 <= i < storage@.len() ==> storage@[i] == 0u8,
        ensures
            result.inv(),
            result@.number_of_bits() == number_of_bits as int,
            // All bits are unset (since storage was zeroed).
            forall|i: int| 0 <= i < number_of_bits as int ==> !result.is_bit_set(i),
    {
        Bitmap {
            number_of_bits,
            usage: 0,
            bits: storage,
        }
    }
}

} // verus!
