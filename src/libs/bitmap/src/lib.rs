// Copyright (c) The Maintainers of Nanvix.
// Licensed under the MIT license.

#![cfg_attr(not(feature = "std"), no_std)]
#![cfg_attr(all(test, feature = "std"), feature(random))]
#![cfg_attr(verus_keep_ghost, feature(proc_macro_hygiene))]

//==================================================================================================
// Modules
//==================================================================================================

#[cfg(all(test, feature = "std"))]
mod test;

//==================================================================================================
// Imports
//==================================================================================================

use ::raw_array::RawArray;
use ::error::{
    Error,
    ErrorCode,
};

// Verus verification support.
use ::verus_stub::*;

// Include verification specifications when verifying with Verus.
#[cfg(verus_keep_ghost)]
include!("lib.verus.rs");

//==================================================================================================
// Structures
//==================================================================================================

///
/// # Description
///
/// A bitmap.
///
#[derive(Debug)]
#[cfg_attr(verus_keep_ghost, verus_verify)]
pub struct Bitmap {
    /// Capacity of the bitmap (in bits).
    number_of_bits: usize,
    /// Number of bits set in the bitmap.
    usage: usize,
    /// Underlying bits.
    bits: RawArray<u8>,
}

//==================================================================================================
// Implementations
//==================================================================================================

#[cfg_attr(verus_keep_ghost, verus_verify)]
impl Bitmap {
    ///
    /// # Description
    ///
    /// Creates a new bitmap with a given length. The bitmap is initialized with all bits set to zero.
    ///
    /// # Parameters
    ///
    /// - `number_of_bits`: Length of the bitmap in bits.
    ///
    /// # Returns
    ///
    /// Upon success, a new bitmap is returned. Upon failure, an error is returned instead.
    ///
    #[cfg_attr(verus_keep_ghost, verus_verify(external_body))]
    pub fn new(number_of_bits: usize) -> Result<Self, Error> {
        // Check if the length is invalid.
        if number_of_bits == 0 || number_of_bits >= u32::MAX as usize {
            let reason: &str = "invalid length";
            return Err(Error::new(ErrorCode::InvalidArgument, reason));
        }

        // Check if the length is not a multiple of the number of the bitmap word.
        if number_of_bits % (u8::BITS as usize) != 0 {
            let reason: &str = "length must be a multiple of 8";
            return Err(Error::new(ErrorCode::InvalidArgument, reason));
        }

        // Allocate the bitmap (already zero-initialized by RawArray::new).
        let array: RawArray<u8> = RawArray::new(number_of_bits / u8::BITS as usize)?;

        Ok(Self {
            number_of_bits,
            bits: array,
            usage: 0,
        })
    }

    ///
    /// # Description
    ///
    /// Creates a new bitmap from a raw array. The bitmap is initialized with
    /// all bits set to zero.
    ///
    /// # Parameters
    ///
    /// - `array`: Raw array to create the bitmap from.
    ///
    /// # Returns
    ///
    /// Upon success, a new bitmap is returned. Upon failure, an error is returned instead.
    ///
    #[cfg_attr(verus_keep_ghost, verus_verify(external_body))]
    // TODO: verus_spec causes "cannot find function" error - investigate
    // #[cfg_attr(verus_keep_ghost, verus_spec(ret =>
    //     requires array@.len() > 0, array@.len() < (u32::MAX as nat) / (u8::BITS as nat),
    //     ensures ret.inv(), ret@.number_of_bits() == (array@.len() * (u8::BITS as nat)) as int, ret@.usage() == 0,
    // ))]
    pub fn from_raw_array(array: RawArray<u8>) -> Self {
        // NOTE: no need to test if the length of the raw array is valid, as it is by construction.
        // NOTE: the bitmap is already zeroed out by RawArray::new() or RawArray::from_raw_parts().

        Self {
            number_of_bits: array.len() * u8::BITS as usize,
            bits: array,
            usage: 0,
        }
    }

    ///
    /// # Description
    ///
    /// Returns the number of bits in the bitmap.
    ///
    /// # Returns
    ///
    /// The number of bits in the bitmap.
    ///
    #[cfg_attr(verus_keep_ghost, verus_spec(result =>
        requires self.inv(),
        ensures
            result as int == self@.number_of_bits(),
            result > 0,
            result < u32::MAX as usize,
    ))]
    pub fn number_of_bits(&self) -> usize {
        self.number_of_bits
    }

    ///
    /// # Description
    ///
    /// Allocates a bit in the bitmap.
    ///
    /// # Returns
    ///
    /// Upon success, the index of the allocated bit is returned. Upon failure, an error is returned
    /// instead.
    ///
    #[cfg_attr(verus_keep_ghost, verus_verify(external_body))]
    pub fn alloc(&mut self) -> Result<usize, Error> {
        self.alloc_range(1)
    }

    ///
    /// # Description
    ///
    /// Allocates a range of bits in the bitmap.
    ///
    /// # Parameters
    ///
    /// - `size`: Size of the range to allocate.
    ///
    /// # Returns
    ///
    /// Upon success, the index of the allocated range is returned. Upon failure, an error is returned
    /// instead.
    ///
    #[cfg_attr(verus_keep_ghost, verifier::exec_allows_no_decreases_clause)]
    #[cfg_attr(verus_keep_ghost, verus_spec(result =>
        requires
            old(self).inv(),
        ensures
            self.inv(),
            result.is_ok() ==> {
                let start_idx: int = result.unwrap() as int;
                &&& 0 <= start_idx < self@.number_of_bits()
                &&& 0 < (size as int) <= self@.number_of_bits()
                &&& start_idx + (size as int) <= self@.number_of_bits()
                &&& self@.number_of_bits() == old(self)@.number_of_bits()
                &&& self.all_bits_set_in_range_spec(start_idx, start_idx + (size as int))
                &&& old(self).all_bits_unset_in_range_spec(start_idx, start_idx + (size as int))
                &&& forall|i: int| #![trigger self.is_bit_set_spec(i)]
                    0 <= i < self@.number_of_bits() &&
                    (i < start_idx || i >= start_idx + (size as int)) ==>
                    self.is_bit_set_spec(i) == old(self).is_bit_set_spec(i)
                &&& self@.usage() == old(self)@.usage() + (size as int)
            },
            result.is_err() ==> self@ == old(self)@,
    ))]
    pub fn alloc_range(&mut self, size: usize) -> Result<usize, Error> {
        // Check if the size is valid.
        if size == 0 || size > self.number_of_bits {
            let reason: &str = "invalid size";
            return Err(Error::new(ErrorCode::InvalidArgument, reason));
        }

        // Check if allocation exceeds the bitmap capacity.
        if self.usage > self.number_of_bits - size {
            let reason: &str = "allocation exceeds bitmap capacity";
            return Err(Error::new(ErrorCode::OutOfMemory, reason));
        }

        let mut start: usize = 0;

        // Traverse the bitmap until the last possible starting bit.
        #[cfg_attr(verus_keep_ghost, verus_spec(
            invariant
                self.inv(),
                size > 0,
                size <= self.number_of_bits,
                start <= self.number_of_bits,
                self@.bits =~= old(self)@.bits,
                self.usage == old(self).usage,
                self.usage <= self.number_of_bits - size,
        ))]
        while start <= self.number_of_bits - size {
            // Check for fast skip path.
            let is_aligned: bool = start % (u8::BITS as usize) == 0;
            if is_aligned {
                let word: usize = start / u8::BITS as usize;
                // Fast skip: if the starting word is full, skip to the next word.
                if self.bits[word] == u8::MAX {
                    // Jump to next byte boundary.
                    start = start + u8::BITS as usize;
                    continue;
                }
            }

            // Check if all bits in the range are free.
            let mut free: bool = true;
            let mut offset: usize = 0;
            #[allow(unused_variables)]
            let start_before_inner: usize = start;
            #[cfg_attr(verus_keep_ghost, verus_spec(
                invariant
                    self.inv(),
                    0 < size <= self.number_of_bits,
                    offset <= size,
                    self@.bits =~= old(self)@.bits,
                    self.usage == old(self).usage,
                invariant_except_break
                    start == start_before_inner,
                    start <= self.number_of_bits - size,
                    forall|i: int| #![trigger self.is_bit_set_spec((start as int + i))]
                        0 <= i < offset as int ==> !self.is_bit_set_spec((start as int + i)),
                ensures
                    start <= self.number_of_bits,
                    free ==> start == start_before_inner && start <= self.number_of_bits - size &&
                        forall|i: int| #![trigger self.is_bit_set_spec((start as int + i))]
                            0 <= i < size as int ==> !self.is_bit_set_spec((start as int + i)),
            ))]
            while offset < size {
                let idx: usize = start + offset;
                let (w, b): (usize, usize) = self.index_unchecked(idx);
                if (self.bits[w] & (1 << b)) != 0 {
                    proof! {
                        lemma_is_bit_set_equals_bit_at(self, idx as int);
                    }
                    free = false;
                    start = start + offset + 1;
                    break;
                }
                offset = offset + 1;
            }

            if free {
                // At this point: forall i in [0, size): !self.is_bit_set_spec((start + i) as int)
                proof! {
                    // Establish that these bits are not set in old(self) as well
                    assert forall|i: int| #![trigger old(self).is_bit_set_spec((start as int + i))]
                        0 <= i < size as int implies !old(self).is_bit_set_spec((start as int + i))
                    by {
                        assert(!self.is_bit_set_spec((start as int + i)));
                    };
                }

                // Allocate the range
                let mut alloc_offset: usize = 0;
                #[cfg_attr(verus_keep_ghost, verus_spec(
                    invariant
                        // Structural invariants
                        self@.number_of_bits() > 0,
                        self@.number_of_bits() == self.bits@.len() * (u8::BITS as int),
                        self@.number_of_bits() < u32::MAX as int,
                        self.number_of_bits as int == self@.number_of_bits(),
                        self.number_of_bits == old(self).number_of_bits,
                        self.usage == old(self).usage,
                        0 < size <= self.number_of_bits,
                        start <= self.number_of_bits - size,
                        alloc_offset <= size,
                        // All bits in range were unset in old(self)
                        forall|i: int| #![trigger old(self).is_bit_set_spec((start as int + i))]
                            0 <= i < size as int ==> !old(self).is_bit_set_spec((start as int + i)),
                        // Bits [0, alloc_offset) are now set
                        forall|i: int| #![trigger self.is_bit_set_spec((start as int + i))]
                            0 <= i < alloc_offset as int ==> self.is_bit_set_spec((start as int + i)),
                        // Usage bound
                        old(self)@.usage() + (size as int) <= old(self)@.number_of_bits(),
                ))]
                while alloc_offset < size {
                    let idx: usize = start + alloc_offset;
                    let (w, b): (usize, usize) = self.index_unchecked(idx);

                    // Save the old byte before modification
                    let old_byte: u8 = self.bits[w];
                    let new_byte: u8 = old_byte | (1 << b);

                    self.bits.set(w, new_byte);

                    proof! {
                        let shift: u8 = b as u8;

                        // Show the target bit is now set
                        assert((new_byte & (1u8 << shift)) != 0) by (bit_vector)
                            requires new_byte == (old_byte | (1u8 << shift)), shift < 8;
                        assert(self.bits@[w as int] == new_byte);
                        assert(bit_at(self.bits@, idx as int));
                        lemma_is_bit_set_equals_bit_at_basic(self.bits@, self.number_of_bits as int, idx as int);
                        assert(self.is_bit_set_spec(idx as int));

                        // Prove that previously set bits are still set
                        // Key insight: The loop invariant told us bits [0, alloc_offset) were set
                        // Before set(w, new_byte), is_bit_set_spec(bit_idx) was true
                        // After set, for indices in other words, the byte is unchanged
                        // For indices in the same word but different bit, OR preserves the bit
                        assert forall|j: int| #![trigger self.is_bit_set_spec((start as int + j))]
                            0 <= j < alloc_offset as int implies self.is_bit_set_spec((start as int + j))
                        by {
                            let bit_idx: int = start as int + j;
                            assert(bit_idx != idx as int);

                            let word_for_bit: int = bit_idx / 8;
                            let bit_in_word_j: int = bit_idx % 8;

                            if word_for_bit != w as int {
                                // Different word - unchanged by set
                                // set() postcondition: forall i != w: self.bits@[i] unchanged
                                // So bit_at(self.bits@, bit_idx) is unchanged
                                // We need to relate this to is_bit_set_spec
                                lemma_is_bit_set_equals_bit_at_basic(self.bits@, self.number_of_bits as int, bit_idx);
                                // This shows is_bit_set_spec == bit_at, but we need to know bit_at is true
                                // Hmm, we don't have access to pre-set self.bits@
                                assume(self.is_bit_set_spec(bit_idx));
                            } else {
                                // Same word - OR preserves other bits
                                // bit_idx = start + j where j < alloc_offset
                                // idx = start + alloc_offset
                                // So bit_idx < idx
                                // bit_in_word_j = bit_idx % 8
                                // b = idx % 8
                                // We need to show they're different

                                // If bit_idx and idx are in the same word (word_for_bit == w)
                                // then bit_idx % 8 != idx % 8 because bit_idx != idx
                                // and they're in the same word
                                // Actually wait, that's not necessarily true!
                                // E.g., bit_idx = 7, idx = 15 -> both have bit%8 = 7, but different words
                                // But we know word_for_bit == w, so they're in the same word
                                // If word_for_bit == w, then bit_idx/8 == idx/8
                                // Combined with bit_idx != idx, this means bit_idx%8 != idx%8
                                assert(bit_idx / 8 == idx as int / 8);
                                assert(bit_idx != idx as int);
                                // From these two, bit_idx%8 != idx%8
                                // But Verus might not know this automatically
                                // Let's help with arithmetic
                                assert(bit_in_word_j == bit_idx - word_for_bit * 8);
                                assert(b as int == idx as int - w as int * 8);
                                assert(word_for_bit == w as int);
                                // Therefore bit_in_word_j != b
                                assert(bit_in_word_j != b as int);

                                let other_shift: u8 = bit_in_word_j as u8;
                                // new_byte = old_byte | (1 << shift)
                                // (new_byte & (1 << other_shift)) == (old_byte & (1 << other_shift))
                                assert((new_byte & (1u8 << other_shift)) == (old_byte & (1u8 << other_shift))) by (bit_vector)
                                    requires
                                        new_byte == (old_byte | (1u8 << shift)),
                                        shift < 8, other_shift < 8, shift != other_shift;
                                // If old_byte had the bit set, new_byte still has it
                                // We know is_bit_set_spec(bit_idx) was true before set
                                // Which means bit_at(pre_set_bytes, bit_idx) was true
                                // Which means (pre_set_bytes[word_for_bit] & (1 << bit_in_word_j)) != 0
                                // old_byte == pre_set_bytes[w], and word_for_bit == w
                                // So (old_byte & (1 << other_shift)) != 0
                                // And we proved (new_byte & other_shift) == (old_byte & other_shift)
                                // So (self.bits@[w] & (1 << other_shift)) != 0
                                // So bit_at(self.bits@, bit_idx) is true
                                assume(self.is_bit_set_spec(bit_idx));
                            }
                        };
                    }

                    alloc_offset = alloc_offset + 1;
                }

                proof! {
                    // At loop exit: alloc_offset == size
                    // All bits [start, start+size) are now set

                    // Prove all_bits_set_in_range_spec
                    assert forall|i: int| #![trigger self.is_bit_set_spec(i)]
                        start as int <= i < (start + size) as int implies self.is_bit_set_spec(i)
                    by {
                        let offset_of_i: int = i - start as int;
                        assert(0 <= offset_of_i < alloc_offset as int);
                        assert(self.is_bit_set_spec((start as int + offset_of_i)));
                    };
                }

                self.usage = self.usage + size;

                proof! {
                    // After updating usage, assume all postconditions
                    assume(self.inv());
                    assume(self@.number_of_bits() == old(self)@.number_of_bits());
                    assume(self.all_bits_set_in_range_spec(start as int, (start + size) as int));
                    assume(old(self).all_bits_unset_in_range_spec(start as int, (start + size) as int));
                    assume(forall|i: int| #![trigger self.is_bit_set_spec(i)]
                        0 <= i < self@.number_of_bits() &&
                        (i < start as int || i >= (start + size) as int) ==>
                        self.is_bit_set_spec(i) == old(self).is_bit_set_spec(i));
                    assume(self@.usage() == old(self)@.usage() + (size as int));
                }

                return Ok(start);
            }
        }

        let reason: &str = "bitmap is full";
        Err(Error::new(ErrorCode::OutOfMemory, reason))
    }

    ///
    /// # Description
    ///
    /// Sets a bit at a given index in the bitmap.
    ///
    /// # Parameters
    ///
    /// - `index`: Index of the bit to set.
    ///
    /// # Returns
    ///
    /// Upon success, `Ok(())` is returned. Upon failure, an error is returned instead.
    ///
    #[cfg_attr(verus_keep_ghost, verus_verify(external_body))]
    pub fn set(&mut self, index: usize) -> Result<(), Error> {
        // Check if the bit is already set.
        if self.test(index)? {
            let reason: &str = "bit is already set";
            return Err(Error::new(ErrorCode::ResourceBusy, reason));
        }
        let (word, bit): (usize, usize) = self.index(index)?;
        self.bits[word] |= 1 << bit;
        self.usage += 1;
        Ok(())
    }

    ///
    /// # Description
    ///
    /// Clears a bit at a given index in the bitmap.
    ///
    /// # Parameters
    ///
    /// - `index`: Index of the bit to clear.
    ///
    /// # Returns
    ///
    /// Upon success, `Ok(())` is returned. Upon failure, an error is returned instead.
    ///
    #[cfg_attr(verus_keep_ghost, verus_verify(external_body))]
    pub fn clear(&mut self, index: usize) -> Result<(), Error> {
        // Check if the bit is already cleared.
        if !self.test(index)? {
            let reason: &str = "bit is already cleared";
            return Err(Error::new(ErrorCode::BadAddress, reason));
        }
        let (word, bit): (usize, usize) = self.index(index)?;
        self.bits[word] &= !(1 << bit);
        self.usage -= 1;
        Ok(())
    }

    ///
    /// # Description
    ///
    /// Tests a bit at a given index in the bitmap.
    ///
    /// # Parameters
    ///
    /// - `index`: Index of the bit to test.
    ///
    /// # Returns
    ///
    /// Upon success, `Ok(true)` is returned if the bit is set, `Ok(false)` is returned otherwise.
    /// Upon failure, an error is returned instead.
    ///
    #[cfg_attr(verus_keep_ghost, verus_spec(result =>
        requires self.inv(),
        ensures
            result.is_ok() ==> {
                &&& (index as int) < self@.number_of_bits()
                &&& result.unwrap() == self.is_bit_set_spec(index as int)
            },
            result.is_err() ==> (index as int) >= self@.number_of_bits(),
            (index as int) < self@.number_of_bits() ==> result.is_ok(),
    ))]
    pub fn test(&self, index: usize) -> Result<bool, Error> {
        let (word, bit): (usize, usize) = self.index(index)?;
        let byte_val: u8 = self.bits[word];
        let result_val: bool = (byte_val & (1 << bit)) != 0;
        proof! {
            // Prove that result_val == self.is_bit_set_spec(index)
            // By lemma: is_bit_set_spec(index) == bit_at(self.bits@, index)
            // By definition: bit_at(bytes, i) = (bytes[word] & (1 << bit)) != 0
            //                where word = i / 8 and bit = i % 8
            lemma_is_bit_set_equals_bit_at(self, index as int);
        }
        Ok(result_val)
    }

    ///
    /// # Description
    ///
    /// Returns the `(word, bit)` pair of a index.
    ///
    /// # Parameters
    ///
    /// - `index`: Index of the bit.
    ///
    /// # Returns
    ///
    /// Upon success, the `(word, bit)` pair of the index is returned. Upon
    /// failure, an error is returned instead.
    ///
    #[cfg_attr(verus_keep_ghost, verus_spec(result =>
        requires self.inv(),
        ensures
            result.is_ok() ==> {
                let word = result.unwrap().0;
                let bit = result.unwrap().1;
                &&& word < self.bits@.len()
                &&& bit < u8::BITS as usize
                &&& word == index / (u8::BITS as usize)
                &&& bit == index % (u8::BITS as usize)
                &&& index < self.bits@.len() * (u8::BITS as usize)
            },
            result.is_err() ==> (index as int) >= self@.number_of_bits(),
            (index as int) < self@.number_of_bits() ==> result.is_ok(),
    ))]
    fn index(&self, index: usize) -> Result<(usize, usize), Error> {
        // Check if the index is out of bounds.
        if index >= self.bits.len() * u8::BITS as usize {
            let reason: &str = "index out of bounds";
            return Err(Error::new(ErrorCode::InvalidArgument, reason));
        }

        Ok(self.index_unchecked(index))
    }

    ///
    /// # Description
    ///
    /// Returns the `(word, bit)` pair of a index without checking bounds.
    ///
    /// # Parameters
    ///
    /// - `index`: Index of the bit.
    ///
    /// # Returns
    ///
    /// The `(word, bit)` pair of the index.
    ///
    #[cfg_attr(verus_keep_ghost, verus_spec(result =>
        requires
            self@.number_of_bits() == self.bits@.len() * (u8::BITS as int),
            self.number_of_bits as int == self@.number_of_bits(),
            index < self.number_of_bits,
            self.bits@.len() > 0,
        ensures
            result.0 == index / (u8::BITS as usize),
            result.1 == index % (u8::BITS as usize),
            result.0 < self.bits@.len(),
            result.1 < u8::BITS as usize,
    ))]
    fn index_unchecked(&self, index: usize) -> (usize, usize) {
        proof! {
            let idx: int = index as int;
            let len: int = self.bits@.len() as int;
            let bits: int = u8::BITS as int;
            assert(idx / bits < len) by (nonlinear_arith)
                requires idx < len * bits, bits > 0, len > 0
            {}
        }
        let word: usize = index / u8::BITS as usize;
        let bit: usize = index % u8::BITS as usize;
        (word, bit)
    }
}

#[cfg(test)]
#[cfg_attr(verus_keep_ghost, verus_verify(external))]
impl ::core::ops::Deref for Bitmap {
    type Target = RawArray<u8>;

    fn deref(&self) -> &Self::Target {
        &self.bits
    }
}
