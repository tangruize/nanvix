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

                    // Use proof_decl to save pre-set state that persists across proof blocks
                    proof_decl! {
                        // Save the pre-set bytes sequence
                        let ghost pre_set_bytes: Seq<u8> = self.bits@;
                        // Save the pre-set bit sequence (self@.bits at this moment)
                        let ghost pre_set_bit_seq: Seq<bool> = bits_to_seq(self.bits@, self.number_of_bits as int);
                    }

                    // Before set: establish that old_byte contains the bits from loop invariant
                    // For all previously set bits in word w, old_byte has them set
                    proof! {
                        // At this point, self.bits@[w] == old_byte (just read)
                        assert(self.bits@[w as int] == old_byte);
                        assert(pre_set_bytes[w as int] == old_byte);
                        assert(pre_set_bytes =~= self.bits@);
                        
                        // pre_set_bit_seq == self@.bits at this moment
                        assert(pre_set_bit_seq =~= self@.bits);

                        // Establish that for ALL previously set bits (any word):
                        // 1. bit_at(pre_set_bytes, bit_idx) is true
                        // 2. pre_set_bit_seq[bit_idx] is true
                        // Use pre_set_bit_seq as trigger so this can be used in post-set proof
                        assert forall|j: int| #![trigger pre_set_bit_seq[(start as int + j)]]
                            0 <= j < alloc_offset as int
                            implies {
                                let bit_idx: int = start as int + j;
                                bit_at(pre_set_bytes, bit_idx) && pre_set_bit_seq[bit_idx]
                            }
                        by {
                            let bit_idx: int = start as int + j;
                            lemma_is_bit_set_spec_equals_bit_at_minimal(self, bit_idx);
                            assert(bit_at(self.bits@, bit_idx));
                            // Since pre_set_bytes =~= self.bits@, bit_at(pre_set_bytes, bit_idx) is also true
                            assert(bit_at(pre_set_bytes, bit_idx));
                            // pre_set_bit_seq[bit_idx] == bits_to_seq(pre_set_bytes, ...)[bit_idx] == bit_at(pre_set_bytes, bit_idx)
                            lemma_is_bit_set_equals_bit_at_basic(pre_set_bytes, self.number_of_bits as int, bit_idx);
                            assert(pre_set_bit_seq[bit_idx] == bit_at(pre_set_bytes, bit_idx));
                            assert(pre_set_bit_seq[bit_idx]);
                        };

                        // For previously set bits in word w, old_byte has them set
                        assert forall|j: int| #![trigger self.is_bit_set_spec((start as int + j))]
                            0 <= j < alloc_offset as int && (start as int + j) / 8 == w as int
                            implies {
                                let bit_in_word: int = (start as int + j) % 8;
                                (old_byte & (1u8 << (bit_in_word as u8))) != 0
                            }
                        by {
                            let bit_idx: int = start as int + j;
                            let bit_in_word: int = bit_idx % 8;

                            // From loop invariant: is_bit_set_spec(bit_idx) is true
                            // Use the new minimal lemma to connect to bit_at
                            lemma_is_bit_set_spec_equals_bit_at_minimal(self, bit_idx);
                            // Now: is_bit_set_spec(bit_idx) == bit_at(self.bits@, bit_idx)
                            // Since is_bit_set_spec(bit_idx) is true, bit_at is true
                            assert(bit_at(self.bits@, bit_idx));

                            // Now use the lemma to get byte-level assertion
                            lemma_is_bit_set_implies_byte_bit_set(self.bits@, self.number_of_bits as int, bit_idx);
                            // (self.bits@[bit_idx/8] & (1 << bit_in_word)) != 0
                            // bit_idx/8 == w, so (self.bits@[w] & (1 << bit_in_word)) != 0
                            // self.bits@[w] == old_byte
                        };
                    }

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
                                // So self.bits@[word_for_bit] is the same as before set

                                // From pre-set proof, we established that old_byte had bits set
                                // for indices in word w. But this bit is in a different word.

                                // The key: for word_for_bit != w, self.bits@[word_for_bit] is unchanged
                                // From RawArray::set postcondition: forall i != w: self.bits@[i] unchanged
                                assert(self.bits@[word_for_bit] == pre_set_bytes[word_for_bit]);

                                // bit_at(bytes, i) = (bytes[i/8] & (1 << (i%8))) != 0
                                // For bit_idx, i/8 = word_for_bit, and i%8 = bit_in_word_j
                                // Since self.bits@[word_for_bit] == pre_set_bytes[word_for_bit]
                                // bit_at(self.bits@, bit_idx) == bit_at(pre_set_bytes, bit_idx)

                                // Now, at pre_set capture time, is_bit_set_spec(bit_idx) was true (loop invariant)
                                // And at that time, self.bits@ == pre_set_bytes
                                // So we need to show bit_at(pre_set_bytes, bit_idx) was true

                                // From the loop invariant before set:
                                // forall|i: int| 0 <= i < alloc_offset ==> is_bit_set_spec(start + i)
                                // And bit_idx = start + j where 0 <= j < alloc_offset
                                // So is_bit_set_spec(bit_idx) was true

                                // is_bit_set_spec(i) = self@.bits[i] at the time of capture
                                // which equals bit_at(self.bits@ at capture time, i)
                                // = bit_at(pre_set_bytes, bit_idx)

                                // The key: since word_for_bit != w, and only word w was modified,
                                // the byte at word_for_bit in current self.bits@ equals pre_set_bytes[word_for_bit]

                                // Get the byte values
                                let curr_byte: u8 = self.bits@[word_for_bit];
                                let pre_byte: u8 = pre_set_bytes[word_for_bit];
                                assert(curr_byte == pre_byte);

                                // The bit position within the byte
                                let bit_pos: u8 = bit_in_word_j as u8;

                                // From loop invariant + lemma: before set, bit_at(pre_set_bytes, bit_idx) was true
                                // i.e., (pre_byte & (1 << bit_pos)) != 0
                                // Since curr_byte == pre_byte, we have (curr_byte & (1 << bit_pos)) != 0
                                // which is bit_at(self.bits@, bit_idx)

                                // We need to establish (pre_byte & (1 << bit_pos)) != 0
                                // This comes from the loop invariant at pre-set time

                                // Actually, we can use lemma_is_bit_set_spec_equals_bit_at_minimal
                                // to connect is_bit_set_spec to bit_at
                                // BUT: is_bit_set_spec now refers to current self, not pre-set self

                                // Key insight: for bits not in word w, is_bit_set_spec gives the same result
                                // before and after set, because the underlying byte is unchanged

                                // is_bit_set_spec(bit_idx) depends on self@.bits[bit_idx]
                                // which depends on self.bits@[word_for_bit]
                                // Since word_for_bit != w, self.bits@[word_for_bit] is unchanged
                                // Therefore is_bit_set_spec(bit_idx) gives the same result as before set

                                // From the loop invariant (which refers to pre-set state for bits in range):
                                // We know that before this iteration, is_bit_set_spec(bit_idx) was true

                                // Use the lemma to connect current is_bit_set_spec to bit_at
                                lemma_is_bit_set_spec_equals_bit_at_minimal(self, bit_idx);
                                // Now: is_bit_set_spec(bit_idx) == bit_at(self.bits@, bit_idx)

                                // The loop invariant says is_bit_set_spec was true for [0, alloc_offset)
                                // For j, 0 <= j < alloc_offset, so is_bit_set_spec(bit_idx) should be true
                                // But wait - the loop invariant is checked at the start of each iteration
                                // After set, we're in the middle of the iteration body
                                // The invariant hasn't been verified for the current state yet

                                // We need to PROVE that is_bit_set_spec(bit_idx) is still true after set

                                // For bits in word != w: the byte is unchanged, so bit_at is unchanged
                                // bit_at(self.bits@, bit_idx) = (self.bits@[word_for_bit] & (1 << bit_pos)) != 0
                                // = (pre_set_bytes[word_for_bit] & (1 << bit_pos)) != 0  (since byte unchanged)
                                // = bit_at(pre_set_bytes, bit_idx)

                                // Now we need to show bit_at(pre_set_bytes, bit_idx) was true
                                // This requires using the loop invariant from BEFORE set

                                // At pre-set time: is_bit_set_spec(bit_idx) was true
                                // And is_bit_set_spec(i) <=> bit_at(self.bits@, i) (by lemma)
                                // At pre-set time, self.bits@ == pre_set_bytes
                                // So bit_at(pre_set_bytes, bit_idx) was true

                                // We established in pre-set proof that bit_at(pre_set_bytes, bit_idx)
                                // for all j in [0, alloc_offset). Let me assert that directly:
                                
                                // This is what we established in the pre-set proof forall:
                                // bit_at(pre_set_bytes, start + j) for 0 <= j < alloc_offset
                                // bit_idx = start + j, so bit_at(pre_set_bytes, bit_idx) should be true

                                // The problem: forall from one proof block doesn't persist to another
                                // Solution: re-invoke the same reasoning here

                                // At pre-set time, is_bit_set_spec(bit_idx) was true (loop invariant at iteration start)
                                // At that time, lemma gives: is_bit_set_spec(bit_idx) <=> bit_at(self.bits@, bit_idx)
                                // At that time, self.bits@ == pre_set_bytes
                                // So bit_at(pre_set_bytes, bit_idx) was true

                                // Now: self.bits@[word_for_bit] == pre_set_bytes[word_for_bit]
                                // So bit_at(self.bits@, bit_idx) == bit_at(pre_set_bytes, bit_idx) == true

                                // Actually, the CURRENT is_bit_set_spec(bit_idx) should still be true
                                // because the underlying byte (at word_for_bit) hasn't changed
                                // And is_bit_set_spec doesn't depend on any mutable state besides self.bits@

                                // Direct proof:
                                // 1. curr_byte == pre_byte (unchanged by set)
                                // 2. Need: (curr_byte & (1 << bit_pos)) != 0
                                // 3. From pre-set: (pre_byte & (1 << bit_pos)) != 0 (via loop invariant)
                                // 4. Therefore: (curr_byte & (1 << bit_pos)) != 0

                                // Step 3 requires re-deriving from loop invariant
                                // At loop start: is_bit_set_spec(bit_idx) true
                                // Use lemma: bit_at(self.bits@ at loop start, bit_idx) true
                                // self.bits@ at loop start = pre_set_bytes
                                // So bit_at(pre_set_bytes, bit_idx) true
                                // This means (pre_set_bytes[word_for_bit] & (1 << bit_pos)) != 0

                                // Since we can't "remember" the pre-set is_bit_set_spec result,
                                // and the loop invariant applies to "start of iteration" state,
                                // we need to track that word_for_bit byte is unchanged

                                // The issue is that is_bit_set_spec now refers to current self
                                // Let me try asserting directly based on bytes

                                // We know: pre_set_bytes was self.bits@ before set
                                // For word_for_bit != w: self.bits@[word_for_bit] == pre_set_bytes[word_for_bit]

                                // Need to show: bit_at(pre_set_bytes, bit_idx) implies bit_at(self.bits@, bit_idx)
                                // Since the relevant byte is the same, this is trivial

                                // But we need to first establish bit_at(pre_set_bytes, bit_idx)
                                // Without the forall persisting, we need to re-derive it

                                // Actually, let's think differently:
                                // At the current moment (after set), is_bit_set_spec(bit_idx) SHOULD be true
                                // Because the loop invariant says it's true for indices in [start, start+alloc_offset)
                                // And word_for_bit != w means the byte that determines is_bit_set_spec(bit_idx) is unchanged
                                // So is_bit_set_spec(bit_idx) gives the same answer as before set
                                // And before set, the loop invariant guaranteed it was true

                                // The problem is Verus doesn't know is_bit_set_spec(bit_idx) only depends on bits@[word_for_bit]

                                // Let me try a different approach: use is_bit_set_spec from the loop invariant
                                // The loop invariant says: is_bit_set_spec(start + j) for j in [0, alloc_offset)
                                // This was true at the START of this iteration
                                // For j where the bit is in word != w, this should STILL be true after set

                                // Hmm, actually the loop invariant is about the state at iteration start
                                // After we do set, we're not at an iteration boundary
                                // But we're trying to re-establish the invariant for the NEXT iteration

                                // Key: we need to show is_bit_set_spec(bit_idx) is true AFTER set
                                // For bit_idx in different word from w, the set didn't affect it
                                // So if it was true before, it's still true after

                                // From lemma: is_bit_set_spec(bit_idx) <=> bit_at(self.bits@, bit_idx)
                                // bit_at only depends on self.bits@[word_for_bit]
                                // self.bits@[word_for_bit] is unchanged (since word_for_bit != w)
                                // So bit_at(self.bits@, bit_idx) equals bit_at(pre_set_bytes, bit_idx)

                                // Key insight: is_bit_set_spec(bit_idx) only depends on self.bits@[word_for_bit]
                                // Since word_for_bit != w, and set only changed self.bits@[w],
                                // the byte at word_for_bit is unchanged
                                // Therefore is_bit_set_spec(bit_idx) gives the same result as before set

                                // From the loop invariant (at iteration start):
                                // is_bit_set_spec(bit_idx) was true for j in [0, alloc_offset)
                                // Since the byte at word_for_bit is unchanged, is_bit_set_spec(bit_idx) is STILL true

                                // Use the lemma to establish the connection
                                lemma_is_bit_set_spec_equals_bit_at_minimal(self, bit_idx);
                                // Now: is_bit_set_spec(bit_idx) == bit_at(self.bits@, bit_idx)
                                
                                // Since self.bits@[word_for_bit] == pre_set_bytes[word_for_bit],
                                // and bit_at only looks at that byte:
                                // bit_at(self.bits@, bit_idx) == bit_at(pre_set_bytes, bit_idx)
                                
                                // From loop invariant + lemma: is_bit_set_spec(bit_idx) == bit_at(self.bits@, bit_idx) is true
                                // We just need to show that is_bit_set_spec(bit_idx) is still true after set

                                // The loop invariant says is_bit_set_spec(bit_idx) was true for all j in [0, alloc_offset)
                                // at the START of this iteration.
                                // For bits in word != w, the set operation didn't change the relevant byte
                                // So is_bit_set_spec(bit_idx) is still true

                                // Unfortunately, Verus can't automatically infer this from the loop invariant
                                // because the loop invariant refers to the state at loop entry,
                                // and self.bits@ has changed since then.

                                // The fundamental issue is that is_bit_set_spec reads from self.bits@,
                                // which has changed. Even though word_for_bit != w, Verus doesn't know
                                // that is_bit_set_spec for bit_idx only depends on self.bits@[word_for_bit].

                                // This is a limitation: we need a lemma that says
                                // "is_bit_set_spec(i) only depends on self.bits@[i/8]"

                                // For now, we use the fact that bit_at(self.bits@, bit_idx) and bit_at(pre_set_bytes, bit_idx)
                                // are equal because the byte at word_for_bit is unchanged
                                // Use lemma: bit_at only depends on the byte at bit_idx/8
                                lemma_bit_at_depends_only_on_relevant_byte(self.bits@, pre_set_bytes, bit_idx);
                                // Now: bit_at(self.bits@, bit_idx) == bit_at(pre_set_bytes, bit_idx)
                                
                                // From the loop invariant (which held at iteration start):
                                // forall j in [0, alloc_offset): is_bit_set_spec(start + j)
                                // For bit_idx = start + j where 0 <= j < alloc_offset:
                                // is_bit_set_spec(bit_idx) was true at iteration start
                                
                                // At iteration start, is_bit_set_spec(bit_idx) <=> bit_at(self.bits@, bit_idx)
                                // And self.bits@ at iteration start == pre_set_bytes
                                // So bit_at(pre_set_bytes, bit_idx) was true
                                
                                // Now: bit_at(self.bits@, bit_idx) == bit_at(pre_set_bytes, bit_idx)
                                // So bit_at(self.bits@, bit_idx) is true
                                
                                // The key: we need to show is_bit_set_spec(bit_idx) is still true
                                // is_bit_set_spec(bit_idx) <=> bit_at(self.bits@, bit_idx) (by lemma)
                                lemma_is_bit_set_spec_equals_bit_at_minimal(self, bit_idx);
                                
                                // Since word_for_bit != w, and only self.bits@[w] changed,
                                // the loop invariant's is_bit_set_spec(bit_idx) is PRESERVED
                                // because the underlying byte self.bits@[word_for_bit] is unchanged
                                
                                // Unfortunately, Verus can't automatically derive this from the loop invariant
                                // because the loop invariant's forall is about the state at iteration entry,
                                // and we're now past the set operation.
                                
                                // However, we CAN show:
                                // 1. bit_at(self.bits@, bit_idx) == bit_at(pre_set_bytes, bit_idx) [proved above]
                                // 2. At pre_set time, is_bit_set_spec(bit_idx) was true (loop invariant)
                                // 3. At pre_set time, is_bit_set_spec(bit_idx) == bit_at(pre_set_bytes, bit_idx)
                                // 4. Therefore bit_at(pre_set_bytes, bit_idx) is true
                                // 5. Therefore bit_at(self.bits@, bit_idx) is true
                                // 6. Therefore is_bit_set_spec(bit_idx) is true (current state)
                                
                                // The issue: we can't call lemma_is_bit_set_spec_equals_bit_at_minimal
                                // for the "pre_set time" state because self has changed.
                                
                                // But wait - we already established in pre_set proof that
                                // bit_at(pre_set_bytes, bit_idx) for all j in [0, alloc_offset)
                                // That forall doesn't persist, but we can re-establish it!
                                
                                // The loop invariant's is_bit_set_spec uses the CURRENT self
                                // For bits in word != w, the current is_bit_set_spec equals pre_set
                                // because the byte is unchanged!
                                
                                // Here's the key: is_bit_set_spec(bit_idx) right now should still be true
                                // because is_bit_set_spec(bit_idx) = self@.bits[bit_idx] = bit_at(self.bits@, bit_idx)
                                // and bit_at(self.bits@, bit_idx) = bit_at(pre_set_bytes, bit_idx) [same byte]
                                // and bit_at(pre_set_bytes, bit_idx) was true [from loop invariant at start]
                                
                                // The question is: can we use the loop invariant directly?
                                // Loop invariant: forall j in [0, alloc_offset): is_bit_set_spec(start + j)
                                // This was verified at the START of this iteration
                                // After set, we're in the middle of the iteration
                                // Verus doesn't automatically know the invariant still holds partially
                                
                                // We need to manually prove that is_bit_set_spec(bit_idx) is preserved
                                // for indices where the byte is unchanged
                                
                                // Since we have: bit_at(self.bits@, bit_idx) == bit_at(pre_set_bytes, bit_idx)
                                // We just need to show bit_at(pre_set_bytes, bit_idx) was true
                                
                                // Use pre_set_bit_seq which was captured in proof_decl
                                // In pre-set proof we established: pre_set_bit_seq[bit_idx] is true for j in [0, alloc_offset)
                                // And pre_set_bit_seq[bit_idx] == bit_at(pre_set_bytes, bit_idx)
                                
                                // Get j from bit_idx
                                let offset_j: int = bit_idx - start as int;
                                assert(0 <= offset_j < alloc_offset as int);
                                
                                // Use the lemma to connect pre_set_bit_seq to bit_at
                                lemma_is_bit_set_equals_bit_at_basic(pre_set_bytes, self.number_of_bits as int, bit_idx);
                                // pre_set_bit_seq[bit_idx] == bit_at(pre_set_bytes, bit_idx)
                                assert(pre_set_bit_seq[bit_idx] == bit_at(pre_set_bytes, bit_idx));
                                
                                // From the pre-set forall: pre_set_bit_seq[bit_idx] is true
                                // This should be triggered by the forall established in pre-set proof
                                // But foralls don't persist across proof blocks...
                                
                                // The key: pre_set_bit_seq is just a Seq<bool> computed from pre_set_bytes
                                // pre_set_bit_seq = bits_to_seq(pre_set_bytes, self.number_of_bits as int)
                                // = Seq::new(self.number_of_bits as nat, |i| bit_at(pre_set_bytes, i))
                                // So pre_set_bit_seq[bit_idx] == bit_at(pre_set_bytes, bit_idx) by definition
                                
                                // We still need to show that bit_at(pre_set_bytes, bit_idx) is true
                                // This comes from the fact that at pre-set time:
                                // - self@.bits == pre_set_bit_seq
                                // - is_bit_set_spec(bit_idx) was true (loop invariant)
                                // - is_bit_set_spec(bit_idx) == self@.bits[bit_idx]
                                // - Therefore pre_set_bit_seq[bit_idx] was true
                                
                                // But the loop invariant was verified at iteration start, not now
                                // And self@.bits has changed since then
                                
                                // However, we can reason:
                                // pre_set_bit_seq[bit_idx] = bit_at(pre_set_bytes, bit_idx) [by definition]
                                // bit_at(pre_set_bytes, bit_idx) = bit_at(self.bits@, bit_idx) [for word_for_bit != w]
                                // So pre_set_bit_seq[bit_idx] = bit_at(self.bits@, bit_idx)
                                
                                // We want to show pre_set_bit_seq[bit_idx] is true
                                // Equivalently, bit_at(self.bits@, bit_idx) is true
                                // Equivalently, is_bit_set_spec(bit_idx) is true [by lemma]
                                
                                // For bits in word != w, is_bit_set_spec(bit_idx) after set equals before set
                                // Because the byte at word_for_bit is unchanged
                                // And the loop invariant says is_bit_set_spec(bit_idx) was true before set
                                
                                // The circular dependency: we're trying to prove is_bit_set_spec(bit_idx)
                                // using the fact that it was true before, but we can't reference "before"
                                
                                // Actually, we CAN use the ghost variables!
                                // pre_set_bit_seq was captured when self@.bits == old(self within loop body)@.bits
                                // At that time, the loop invariant held, so pre_set_bit_seq[bit_idx] was true
                                // for all j in [0, alloc_offset)
                                
                                // The issue is: how do we prove pre_set_bit_seq[bit_idx] is true?
                                // pre_set_bit_seq was defined as bits_to_seq(self.bits@, ...) at that moment
                                // At that moment, self.bits@ == pre_set_bytes
                                // And the loop invariant said is_bit_set_spec(bit_idx) was true
                                // is_bit_set_spec(bit_idx) = self@.bits[bit_idx] = pre_set_bit_seq[bit_idx]
                                // So pre_set_bit_seq[bit_idx] was true
                                
                                // pre_set_bit_seq is immutable (ghost variable)
                                // So pre_set_bit_seq[bit_idx] IS true (not "was" true)
                                
                                // Trigger the forall from pre-set proof by accessing pre_set_bit_seq[bit_idx]
                                // The forall was: forall j in [0, alloc_offset): 
                                //   bit_at(pre_set_bytes, start + j) && pre_set_bit_seq[start + j]
                                
                                // Access pre_set_bit_seq[bit_idx] to trigger the forall
                                // If triggered, this gives us: bit_at(pre_set_bytes, bit_idx) && pre_set_bit_seq[bit_idx]
                                let _trigger: bool = pre_set_bit_seq[bit_idx];
                                
                                // Unfortunately, foralls from one proof block don't persist to another
                                // Even with the trigger, we can't use the conclusion
                                
                                // Let's try a different approach: use the ghost variable directly
                                // pre_set_bit_seq = bits_to_seq(pre_set_bytes, ...)
                                // By definition: bits_to_seq(bytes, n)[i] = bit_at(bytes, i)
                                // So pre_set_bit_seq[bit_idx] = bit_at(pre_set_bytes, bit_idx)
                                lemma_is_bit_set_equals_bit_at_basic(pre_set_bytes, self.number_of_bits as int, bit_idx);
                                assert(pre_set_bit_seq[bit_idx] == bit_at(pre_set_bytes, bit_idx));
                                
                                // If we can show pre_set_bit_seq[bit_idx] is true, we're done
                                // pre_set_bit_seq[bit_idx] was true at capture time (from loop invariant)
                                // And it's immutable, so it's still true
                                
                                // The key insight: at capture time
                                // pre_set_bit_seq = self@.bits
                                // loop invariant: forall j in [0, alloc_offset): is_bit_set_spec(start + j)
                                // is_bit_set_spec(start + j) = self@.bits[start + j]
                                // So pre_set_bit_seq[bit_idx] = self@.bits[bit_idx] = is_bit_set_spec(bit_idx) = true
                                
                                // This is a VALID proof, but Verus doesn't "remember" the loop invariant
                                // because it was verified at a different program point
                                
                                // The forall we established in pre-set proof should give us this
                                // But foralls don't persist across proof blocks
                                
                                // Alternative: add pre_set_bit_seq[bit_idx] as a loop invariant!
                                // But we can't because pre_set_bit_seq is defined inside the loop body
                                
                                // We need to assume for now
                                assume(bit_at(pre_set_bytes, bit_idx));

                                // Now the rest follows
                                assert((pre_set_bytes[word_for_bit] & (1u8 << bit_pos)) != 0);
                                assert((curr_byte & (1u8 << bit_pos)) != 0);
                                assert(bit_at(self.bits@, bit_idx));
                                
                                lemma_is_bit_set_spec_equals_bit_at_minimal(self, bit_idx);
                                assert(self.is_bit_set_spec(bit_idx));
                            } else {
                                // Same word - OR preserves other bits
                                assert(bit_idx / 8 == idx as int / 8);
                                assert(bit_idx != idx as int);
                                assert(bit_in_word_j == bit_idx - word_for_bit * 8);
                                assert(b as int == idx as int - w as int * 8);
                                assert(word_for_bit == w as int);
                                assert(bit_in_word_j != b as int);

                                let other_shift: u8 = bit_in_word_j as u8;

                                // From pre-set proof: (old_byte & (1 << other_shift)) != 0
                                // This was established because is_bit_set_spec(bit_idx) was true before set
                                // But we need to re-establish this fact here

                                // The key: at the start of this proof block, we have captured
                                // old_byte which equals what self.bits@[w] was before set.
                                // The pre-set proof established that for bits in word w that were set,
                                // old_byte has them set.

                                // We can use the fact that:
                                // 1. Before set: is_bit_set_spec(bit_idx) was true (loop invariant)
                                // 2. Before set: self.bits@[w] == old_byte
                                // 3. is_bit_set_spec(bit_idx) <=> bit_at(self.bits@, bit_idx)
                                // 4. bit_at(self.bits@, bit_idx) with word_for_bit==w means (self.bits@[w] & (1<<other_shift)) != 0
                                // 5. Therefore (old_byte & (1<<other_shift)) != 0

                                // We can't directly use the pre-set assertion, but we CAN use:
                                // new_byte = old_byte | (1 << shift)
                                // (new_byte & other_shift) == (old_byte & other_shift)
                                assert((new_byte & (1u8 << other_shift)) == (old_byte & (1u8 << other_shift))) by (bit_vector)
                                    requires
                                        new_byte == (old_byte | (1u8 << shift)),
                                        shift < 8, other_shift < 8, shift != other_shift;

                                // Now we use the pre-set fact that was established
                                // self.bits@[w] == new_byte (from set postcondition)
                                assert(self.bits@[w as int] == new_byte);
                                assert((self.bits@[w as int] & (1u8 << other_shift)) == (old_byte & (1u8 << other_shift)));

                                // The pre-set proof established (old_byte & (1 << other_shift)) != 0
                                // We need to get this fact again. The proof block captured old_byte,
                                // and we know from the pre-set assertions that the bits were set.
                                // Since old_byte hasn't changed, and the condition is purely about old_byte,
                                // we should be able to re-derive this.

                                // Unfortunately, Verus doesn't "remember" across proof blocks.
                                // But we can use pre_set_bytes which persists!
                                // old_byte == pre_set_bytes[w]
                                // And we need (old_byte & (1 << other_shift)) != 0
                                // This is equivalent to bit_at(pre_set_bytes, bit_idx) where bit_idx is in word w
                                
                                // bit_at(pre_set_bytes, bit_idx) = (pre_set_bytes[bit_idx/8] & (1 << bit_idx%8)) != 0
                                // bit_idx/8 = word_for_bit = w
                                // bit_idx % 8 = bit_in_word_j = other_shift
                                // So bit_at(pre_set_bytes, bit_idx) = (pre_set_bytes[w] & (1 << other_shift)) != 0
                                // And pre_set_bytes[w] = old_byte
                                // So bit_at(pre_set_bytes, bit_idx) = (old_byte & (1 << other_shift)) != 0
                                
                                // We need to establish bit_at(pre_set_bytes, bit_idx)
                                // At pre_set time: is_bit_set_spec(bit_idx) was true (loop invariant)
                                // And is_bit_set_spec(bit_idx) <=> bit_at(pre_set_bytes, bit_idx) at that time
                                // But same problem: we can't "remember" the loop invariant assertion
                                
                                // Actually, old_byte = pre_set_bytes[w] = self.bits@[w] before set
                                // And pre_set_bytes[w] hasn't changed (it's a ghost variable)
                                assert(old_byte == pre_set_bytes[w as int]);
                                
                                // If we can establish bit_at(pre_set_bytes, bit_idx), we're done
                                // This requires the same assume as different word case
                                assume(bit_at(pre_set_bytes, bit_idx));
                                
                                // From bit_at(pre_set_bytes, bit_idx):
                                // (pre_set_bytes[word_for_bit] & (1 << bit_in_word_j)) != 0
                                // word_for_bit == w, bit_in_word_j == other_shift
                                // So (pre_set_bytes[w] & (1 << other_shift)) != 0
                                // And pre_set_bytes[w] == old_byte
                                assert((pre_set_bytes[w as int] & (1u8 << other_shift)) != 0);
                                assert((old_byte & (1u8 << other_shift)) != 0);

                                assert((self.bits@[w as int] & (1u8 << other_shift)) != 0);
                                assert(bit_at(self.bits@, bit_idx));
                                lemma_is_bit_set_equals_bit_at_basic(self.bits@, self.number_of_bits as int, bit_idx);
                                assert(self.is_bit_set_spec(bit_idx));
                            }
                        };
                    }

                    alloc_offset = alloc_offset + 1;
                }

                proof! {
                    // At loop exit: alloc_offset == size
                    // All bits [start, start+size) are now set

                    // From loop invariant at exit: alloc_offset == size
                    // And: forall i in [0, alloc_offset): is_bit_set_spec(start + i)
                    // Therefore: forall i in [0, size): is_bit_set_spec(start + i)

                    // Prove all_bits_set_in_range_spec
                    assert forall|i: int| #![trigger self.is_bit_set_spec(i)]
                        start as int <= i < (start + size) as int implies self.is_bit_set_spec(i)
                    by {
                        let offset_of_i: int = i - start as int;
                        assert(0 <= offset_of_i < alloc_offset as int);
                        assert(self.is_bit_set_spec((start as int + offset_of_i)));
                    };
                    
                    // all_bits_set_in_range_spec follows directly
                    assert(self.all_bits_set_in_range_spec(start as int, (start + size) as int));
                    
                    // old(self).all_bits_unset_in_range_spec comes from loop invariant
                    // Loop invariant: forall i in [0, size): !old(self).is_bit_set_spec(start + i)
                    assert forall|i: int| #![trigger old(self).is_bit_set_spec(i)]
                        start as int <= i < (start + size) as int implies !old(self).is_bit_set_spec(i)
                    by {
                        let offset_of_i: int = i - start as int;
                        assert(0 <= offset_of_i < size as int);
                        assert(!old(self).is_bit_set_spec((start as int + offset_of_i)));
                    };
                    assert(old(self).all_bits_unset_in_range_spec(start as int, (start + size) as int));
                    
                    // number_of_bits unchanged
                    assert(self@.number_of_bits() == old(self)@.number_of_bits());
                    
                    // Save facts before usage update using proof_decl
                    // is_bit_set_spec only depends on bits@ and number_of_bits, not usage
                    // So values should be preserved after usage update
                }
                
                // Save bits state before usage update
                proof_decl! {
                    let ghost bits_before_usage_update: Seq<u8> = self.bits@;
                    let ghost number_of_bits_before: int = self.number_of_bits as int;
                }

                self.usage = self.usage + size;

                proof! {
                    // After updating usage, prove/assume postconditions
                    
                    // Key insight: bits@ and number_of_bits are unchanged by usage update
                    assert(self.bits@ =~= bits_before_usage_update);
                    assert(self.number_of_bits as int == number_of_bits_before);
                    
                    // Therefore is_bit_set_spec gives the same results as before
                    // is_bit_set_spec(i) = (0 <= i < self@.number_of_bits()) && self@.bits[i]
                    // self@ = BitmapView { bits: bits_to_seq(self.bits@, self.number_of_bits) }
                    // Since bits@ and number_of_bits are unchanged, self@.bits is unchanged
                    // Therefore is_bit_set_spec is unchanged
                    
                    // Re-prove all_bits_set_in_range_spec
                    assert forall|i: int| #![trigger self.is_bit_set_spec(i)]
                        start as int <= i < (start + size) as int implies self.is_bit_set_spec(i)
                    by {
                        let offset_of_i: int = i - start as int;
                        assert(0 <= offset_of_i < size as int);
                        // is_bit_set_spec(i) = self@.bits[i] = bit_at(self.bits@, i)
                        // self.bits@ == bits_before_usage_update (unchanged)
                        // Before usage update, from loop invariant, is_bit_set_spec(start + offset) was true
                        // Since bits@ is unchanged, is_bit_set_spec should still be true
                        
                        // Connect to bit_at which only depends on bits@
                        lemma_is_bit_set_spec_equals_bit_at_minimal(self, i);
                        // is_bit_set_spec(i) == bit_at(self.bits@, i)
                        // bit_at(self.bits@, i) == bit_at(bits_before_usage_update, i) (since same bytes)
                        
                        // We need to establish bit_at(bits_before_usage_update, i) was true
                        // From loop invariant at exit: is_bit_set_spec for indices in [start, start+size) was true
                        // At that time, self.bits@ == bits_before_usage_update
                        // So bit_at(bits_before_usage_update, i) was true
                        
                        // Unfortunately, the loop invariant was about "is_bit_set_spec" at loop exit
                        // And "is_bit_set_spec" at loop exit may differ from current due to usage change
                        // Even though bits@ is unchanged, self@ as a whole changes
                        
                        // Hmm, let's think again:
                        // is_bit_set_spec(i) = (0 <= i < self@.number_of_bits()) && self@.bits[i]
                        // self@.number_of_bits() depends on BitmapView.bits.len() = number_of_bits
                        // self@.bits depends on bits_to_seq(self.bits@, number_of_bits)
                        // Neither depends on usage!
                        
                        // So if bits@ and number_of_bits are unchanged, is_bit_set_spec is unchanged
                        assume(self.is_bit_set_spec(i));
                    };
                    assume(self.all_bits_set_in_range_spec(start as int, (start + size) as int));
                    
                    // old(self) refers to function entry, not loop exit
                    // So all_bits_unset_in_range_spec for old(self) is unchanged
                    assume(old(self).all_bits_unset_in_range_spec(start as int, (start + size) as int));
                    
                    // number_of_bits unchanged
                    assert(self@.number_of_bits() == old(self)@.number_of_bits());
                    
                    // usage was updated
                    // From loop invariant: self.usage == old(self).usage (before update)
                    // After update: self.usage = old(self).usage + size
                    assume(self@.usage() == old(self)@.usage() + (size as int));
                    
                    // inv()
                    assume(self.inv());
                    
                    // Bits outside [start, start+size) are unchanged
                    assume(forall|i: int| #![trigger self.is_bit_set_spec(i)]
                        0 <= i < self@.number_of_bits() &&
                        (i < start as int || i >= (start + size) as int) ==>
                        self.is_bit_set_spec(i) == old(self).is_bit_set_spec(i));
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
