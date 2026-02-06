// Copyright(c) The Maintainers of Nanvix.
// Licensed under the MIT License.

//! # Bitmap Allocator - Implementation
//!
//! This file contains the implementation code for bitmap allocator.
//! Specification functions are in `lib.spec.rs` and proofs are in `lib.proof.rs`.
//! Uses Set<int> as the primary abstraction for efficient frame conditions.

use crate::libs::{
    error::{
        Error,
        ErrorCode,
    },
    raw_array::{
        axiom_u8_zero_is_0,
        RawArray,
    },
};
use vstd::prelude::*;

// Include specifications.
include!("lib.spec.rs");

// Include proofs (lemmas).
include!("lib.proof.rs");

verus! {

//==================================================================================================
// Structures
//==================================================================================================

/// A bitmap.
#[cfg_attr(not(verus_keep_ghost), derive(Debug))]
#[verifier::ext_equal]
pub struct Bitmap {
    /// Capacity of the bitmap (in bits).
    number_of_bits: usize,
    /// Number of bits set in the bitmap.
    usage: usize,
    /// Underlying bits.
    bits: RawArray<u8>,
}

//==================================================================================================
// Implementation
//==================================================================================================

impl Bitmap {
    //==================================================================================================
    // Public Methods
    //==================================================================================================

    /// Creates a new bitmap with a given length. All bits are initialized to zero.
    pub fn new(number_of_bits: usize) -> (result: Result<Self, Error>)
        ensures
            result is Ok ==> {
                let bitmap = result->Ok_0;
                &&& bitmap.inv()
                &&& bitmap@.number_of_bits() == number_of_bits as int
                &&& bitmap@.is_empty()
                &&& forall|i: int| 0 <= i < bitmap@.number_of_bits() ==> !bitmap.is_bit_set(i)
            },
            (number_of_bits == 0 ||
             number_of_bits >= u32::MAX as usize ||
             number_of_bits % (u8::BITS as usize) != 0) ==> result is Err,
    {
        // Check if the number of bits is valid.
        if number_of_bits == 0 || number_of_bits >= u32::MAX as usize {
            let reason: &str = "invalid number of bits";
            return Err(Error::new(ErrorCode::InvalidArgument, reason));
        }

        // Check if the number of bits is a multiple of 8.
        if number_of_bits % u8::BITS as usize != 0 {
            let reason: &str = "bitmap length must be a multiple of 8";
            return Err(Error::new(ErrorCode::InvalidArgument, reason));
        }

        // Allocate the underlying array.
        let num_bytes: usize = number_of_bits / u8::BITS as usize;
        let array: RawArray<u8> = RawArray::new(num_bytes)?;

        let result = Self {
            number_of_bits,
            bits: array,
            usage: 0,
        };

        proof {
            assert forall|i: int| 0 <= i < result.bits@.len() implies (result.bits@[i] == 0) by {
                axiom_u8_zero_is_0(result.bits@[i]);
            };
            result.lemma_zero_bytes_means_empty_set();
            Self::lemma_empty_set_finite();
        }

        Ok(result)
    }

    /// Alias for `new` for backward compatibility with slab/frame APIs.
    pub fn new_managed(number_of_bits: usize) -> (result: Result<Self, Error>)
        requires
            number_of_bits > 0,
            number_of_bits <= (usize::MAX as int - 7) / 8 * 8,
            number_of_bits % (u8::BITS as usize) == 0,
            number_of_bits < u32::MAX as usize,
        ensures
            result is Ok ==> {
                let bmp = result->Ok_0;
                &&& bmp.inv()
                &&& bmp@.number_of_bits() == number_of_bits as int
                &&& bmp@.is_empty()
                &&& forall|i: int| 0 <= i < number_of_bits as int ==> !bmp.is_bit_set(i)
            },
    {
        Self::new(number_of_bits)
    }

    /// Creates a new bitmap from a raw array. All bits must be initialized to zero.
    pub fn from_raw_array(array: RawArray<u8>) -> (result: Self)
        requires
            array@.len() > 0,
            array@.len() <= usize::MAX / (u8::BITS as usize),
            array@.len() * (u8::BITS as usize) < u32::MAX as usize,
            forall|i: int| 0 <= i < array@.len() ==> array@[i] == 0,
        ensures
            result.inv(),
            result@.number_of_bits() == array@.len() * (u8::BITS as int),
            result@.is_empty(),
            forall|i: int| 0 <= i < result@.number_of_bits() ==> !result.is_bit_set(i),
    {
        let result = Self {
            number_of_bits: array.len() * u8::BITS as usize,
            bits: array,
            usage: 0,
        };
        proof {
            result.lemma_zero_bytes_means_empty_set();
            Self::lemma_empty_set_finite();
        }
        result
    }

    /// Returns the number of bits in the bitmap.
    pub fn number_of_bits(&self) -> (result: usize)
        requires
            self.inv(),
        ensures
            result as int == self@.number_of_bits(),
            result > 0,
            result < u32::MAX as usize,
    {
        self.number_of_bits
    }

    /// Returns the number of bits set (usage count) in the bitmap.
    pub fn usage(&self) -> (result: usize)
        requires
            self.inv(),
        ensures
            result as int == self@.usage(),
            result as int <= self@.number_of_bits(),
    {
        self.usage
    }

    /// Allocates a single bit in the bitmap.
    pub fn alloc(&mut self) -> (result: Result<usize, Error>)
        requires
            old(self).inv(),
        ensures
            self.inv(),
            result is Ok ==> {
                let index = result->Ok_0 as int;
                &&& 0 <= index < self@.number_of_bits()
                &&& self@.number_of_bits() == old(self)@.number_of_bits()
                &&& self.is_bit_set(index)
                &&& !old(self).is_bit_set(index)
                &&& !old(self)@.is_full()
                // Frame: only the allocated bit changed.
                &&& forall|i: int| 0 <= i < self@.number_of_bits() && i != index ==>
                    self.is_bit_set(i) == old(self).is_bit_set(i)
                // Set-based frame.
                &&& self@.set_bits =~= old(self)@.set_bits.insert(index)
                &&& self@.usage() == old(self)@.usage() + 1
            },
            result is Err ==> self@ == old(self)@,
            old(self)@.has_free_bit() ==> result is Ok,
    {
        proof {
            if old(self)@.has_free_bit() {
                old(self).lemma_has_free_bit_implies_exists_free_range_1();
            }
        }
        self.alloc_range(1)
    }

    /// Allocates a contiguous range of bits in the bitmap.
    #[verifier::exec_allows_no_decreases_clause]
    pub fn alloc_range(&mut self, size: usize) -> (result: Result<usize, Error>)
        requires
            old(self).inv(),
        ensures
            self.inv(),
            result is Ok ==> {
                let start = result->Ok_0 as int;
                &&& 0 <= start < self@.number_of_bits()
                &&& 0 < size <= self@.number_of_bits()
                &&& start + (size as int) <= self@.number_of_bits()
                &&& self@.number_of_bits() == old(self)@.number_of_bits()
                &&& self.all_bits_set_in_range(start, start + (size as int))
                &&& old(self).all_bits_unset_in_range(start, start + (size as int))
                // Frame: only the allocated range changed.
                &&& forall|i: int| 0 <= i < self@.number_of_bits() &&
                    (i < start || i >= start + (size as int)) ==>
                    self.is_bit_set(i) == old(self).is_bit_set(i)
                // Set-based frame.
                &&& self@.set_bits =~= old(self)@.set_bits.union(BitmapView::range_set(start, start + (size as int)))
                &&& self@.usage() == old(self)@.usage() + (size as int)
            },
            result is Err ==> self@ == old(self)@,
            (size > 0 && old(self).exists_contiguous_free_range(size as int)) ==> result is Ok,
    {
        let ghost old_self = *self;

        // Check if the size is valid.
        if size == 0 || size > self.number_of_bits {
            proof {
                if size > self.number_of_bits {
                    assert forall|start: int| #![trigger old_self.has_free_range_at(start, size as int)]
                        0 <= start implies !old_self.has_free_range_at(start, size as int)
                    by {
                        assert(start + (size as int) > self@.number_of_bits());
                    }
                    assert(!old_self.exists_contiguous_free_range(size as int));
                }
            }
            let reason: &str = "invalid size";
            return Err(Error::new(ErrorCode::InvalidArgument, reason));
        }

        // Check if allocation exceeds the bitmap capacity.
        if self.usage > self.number_of_bits - size {
            proof {
                assert(self@.usage() > self@.number_of_bits() - (size as int));
                // If usage > number_of_bits - size, then there are fewer than `size` free bits.
                // Use lemma to prove no contiguous free range exists.
                assert forall|p: int| #![trigger old_self.has_free_range_at(p, size as int)]
                    0 <= p <= old_self@.number_of_bits() - (size as int) implies !old_self.has_free_range_at(p, size as int)
                by {
                    if old_self.has_free_range_at(p, size as int) {
                        old_self.lemma_free_range_implies_usage_bound(p, size as int);
                    }
                }
                assert(!old_self.exists_contiguous_free_range(size as int));
            }
            let reason: &str = "allocation exceeds bitmap capacity";
            return Err(Error::new(ErrorCode::OutOfMemory, reason));
        }

        let mut start: usize = 0;

        // Search for a contiguous free range.
        while start <= self.number_of_bits - size
            invariant
                self.inv(),
                old_self.inv(),
                old_self == old(self),
                size > 0,
                size <= self.number_of_bits,
                start <= self.number_of_bits,
                self@.set_bits =~= old(self)@.set_bits,
                self.usage <= self.number_of_bits - size,
                // number_of_bits is unchanged.
                self.number_of_bits == old_self.number_of_bits,
                // All positions before start don't have a free range.
                forall|p: int| #![trigger self.has_free_range_at(p, size as int)]
                    0 <= p < start as int ==> !self.has_free_range_at(p, size as int),
        {
            // Fast skip: if the starting word is full, skip 8 bits.
            let is_aligned: bool = start % (u8::BITS as usize) == 0;
            if is_aligned {
                let word: usize = start / u8::BITS as usize;
                if self.bits[word] == u8::MAX {
                    proof {
                        // When a byte is 0xFF, all 8 bits are set.
                        assert forall|i: int| start as int <= i < start as int + 8 implies
                            self.is_bit_set(i)
                        by {
                            let bit_pos: int = i % 8;
                            let bit_pos_u8: u8 = bit_pos as u8;
                            assert((0xFFu8 & (1u8 << bit_pos_u8)) != 0) by (bit_vector)
                                requires 0 <= bit_pos_u8 < 8;
                        }

                        // No contiguous free range starts in [start, start+8).
                        assert forall|p: int| #![trigger self.has_free_range_at(p, size as int)]
                            start as int <= p < (start + 8) as int implies !self.has_free_range_at(p, size as int)
                        by {
                            assert(self.is_bit_set(p));
                        }
                    }

                    start = start + u8::BITS as usize;
                    continue;
                }
            }

            // Check if we have a free range starting at `start`.
            let ghost start_before_inner: usize = start;
            let mut offset: usize = 0;
            let mut free: bool = true;

            while offset < size
                invariant_except_break
                    start == start_before_inner,  // start doesn't change until break
                invariant
                    self.inv(),
                    old_self.inv(),
                    old_self == old(self),
                    0 < size <= self.number_of_bits,
                    offset <= size,
                    start_before_inner <= self.number_of_bits - size,  // from outer loop
                    self@.set_bits =~= old(self)@.set_bits,
                    forall|p: int| #![trigger self.has_free_range_at(p, size as int)]
                        0 <= p < start_before_inner as int ==> !self.has_free_range_at(p, size as int),
                    // All bits checked so far are unset.
                    free ==> forall|i: int| 0 <= i < offset ==>
                        !#[trigger] self.is_bit_set((start_before_inner + i) as int),
                ensures
                    start <= self.number_of_bits,
                    free ==> start == start_before_inner && start <= self.number_of_bits - size &&
                        forall|i: int| 0 <= i < size ==>
                            !#[trigger] self.is_bit_set((start + i) as int),
                    !free ==> forall|p: int| #![trigger self.has_free_range_at(p, size as int)]
                        0 <= p < start as int ==> !self.has_free_range_at(p, size as int),
            {
                let idx: usize = start + offset;

                // Check bounds.
                if idx >= self.number_of_bits {
                    free = false;
                    start = self.number_of_bits;
                    proof {
                        // Prove: for all p in [start_before_inner, self.number_of_bits),
                        // p + size > number_of_bits, so no free range.
                        assert forall|p: int| #![trigger self.has_free_range_at(p, size as int)]
                            0 <= p < start as int implies !self.has_free_range_at(p, size as int)
                        by {
                            if p >= start_before_inner as int {
                                // We have: idx = start_before_inner + offset >= number_of_bits.
                                // And: offset < size.
                                // For p >= start_before_inner:
                                // p + size >= start_before_inner + size > start_before_inner + offset = idx >= number_of_bits.
                                // So p + size > number_of_bits.
                                assert(idx as int == (start_before_inner + offset) as int);
                                assert(idx as int >= self@.number_of_bits());
                                assert((offset as int) < (size as int));
                                assert(p >= start_before_inner as int);
                                // p + size >= start_before_inner + size > start_before_inner + offset >= number_of_bits.
                                assert(p + (size as int) > self@.number_of_bits());
                            }
                        }
                    }
                    break;
                }

                // Check if bit is set.
                let is_set: bool = self.test_unchecked(idx);
                if is_set {
                    free = false;
                    start = idx + 1;
                    proof {
                        // Bit at idx is set, so no free range can include idx.
                        assert forall|p: int| #![trigger self.has_free_range_at(p, size as int)]
                            start_before_inner as int <= p <= idx as int implies !self.has_free_range_at(p, size as int)
                        by {
                            if self.has_free_range_at(p, size as int) {
                                // Free range at p means all bits in [p, p+size) are unset.
                                // We have: start_before_inner <= p <= idx.
                                // And: idx = start_before_inner + offset, offset < size.
                                // So: idx - p <= idx - start_before_inner = offset < size.
                                // Therefore: p <= idx < p + size, meaning idx is in [p, p+size).
                                assert(p <= idx as int);
                                assert((idx as int) - p <= (offset as int));
                                assert((offset as int) < (size as int));
                                assert((idx as int) < p + (size as int));
                                // So idx is in the range [p, p+size).
                                // has_free_range_at(p, size) means all bits in [p, p+size) are unset.
                                // But idx is set (self.is_bit_set(idx)).
                                assert(self.is_bit_set(idx as int));
                                assert(self.all_bits_unset_in_range(p, p + (size as int)));
                                // This is a contradiction: idx in range, idx set, but range all unset.
                                assert(!self.is_bit_set(idx as int));
                            }
                        }
                    }
                    break;
                }

                offset = offset + 1;
            }

            if free {
                // Found a free range at [start, start + size).
                proof {
                    // From inner loop ensures:
                    // forall|j: int| 0 <= j < size ==> !self.is_bit_set((start + j) as int)
                    // This means all bits in [start, start+size) are unset.
                    // From outer loop invariant: self@.set_bits =~= old(self)@.set_bits
                    // And old_self == old(self).
                    // Therefore, bits unset in self are also unset in old_self = old(self).
                    
                    // Convert from j-indexed to i-indexed form.
                    assert forall|i: int| start as int <= i < start as int + (size as int) implies !#[trigger] old_self.is_bit_set(i)
                    by {
                        let j: int = i - (start as int);
                        // j = i - start, so i = start + j.
                        // 0 <= j < size follows from start <= i < start + size.
                        assert(0 <= j && j < size);
                        // From inner loop: !self.is_bit_set((start + j) as int) = !self.is_bit_set(i).
                        assert(!self.is_bit_set((start as int + j) as int));
                        // Since self@.set_bits =~= old(self)@.set_bits and old_self == old(self):
                        // !self@.set_bits.contains(i) <==> !old_self@.set_bits.contains(i).
                    };
                    
                    // old_self == old(self), so old_self.is_bit_set(i) == old(self).is_bit_set(i).
                    assert forall|i: int| start as int <= i < start as int + (size as int) implies !#[trigger] old(self).is_bit_set(i)
                    by {
                        assert(!old_self.is_bit_set(i));
                    };
                    
                    // This is the definition of all_bits_unset_in_range.
                    assert(old(self).all_bits_unset_in_range(start as int, start as int + (size as int)));
                }

                // Allocate the range.
                let ghost pre_alloc_self = *self;
                let mut alloc_offset: usize = 0;

                proof {
                    // At this point, self.inv() holds from the outer loop invariant.
                    // From outer loop invariant: self.number_of_bits == old_self.number_of_bits.
                    // self@.number_of_bits() = self.number_of_bits as int (from self.inv()).
                    // old_self@.number_of_bits() = old_self.number_of_bits as int (from old_self.inv()).
                    // Therefore self@.number_of_bits() == old_self@.number_of_bits().
                    assert(self@.number_of_bits() == old_self@.number_of_bits());
                    assert(pre_alloc_self@.number_of_bits() == old_self@.number_of_bits());
                }

                while alloc_offset < size
                    invariant
                        // Basic structure preservation.
                        self.bits@.len() == pre_alloc_self.bits@.len(),
                        self.bits@.len() == old_self.bits@.len(),
                        self@.number_of_bits() == self.bits@.len() * (u8::BITS as int),
                        self.number_of_bits == pre_alloc_self.number_of_bits,
                        self.usage == pre_alloc_self.usage,  // Not updated yet.
                        // Ghost state.
                        old_self.inv(),
                        pre_alloc_self.inv(),
                        old_self == old(self),
                        // Bounds.
                        0 < size <= self.number_of_bits,
                        start <= self.number_of_bits - size,
                        alloc_offset <= size,
                        // Bits [start, start+alloc_offset) are set.
                        forall|i: int| 0 <= i < alloc_offset ==>
                            #[trigger] self.is_bit_set((start + i) as int),
                        // Bits outside [start, start+alloc_offset) are unchanged.
                        forall|i: int| (0 <= i < self@.number_of_bits() &&
                            (i < start as int || i >= (start + alloc_offset) as int)) ==>
                            #[trigger] self.is_bit_set(i) == #[trigger] old_self.is_bit_set(i),
                        // Set-based invariant.
                        self@.set_bits =~= old_self@.set_bits.union(BitmapView::range_set(start as int, start as int + (alloc_offset as int))),
                {
                    let idx: usize = start + alloc_offset;
                    
                    proof {
                        // Prove idx is valid for index_unchecked.
                        assert(idx < self.number_of_bits);
                        assert(self.bits@.len() > 0);
                        // index_unchecked requires self.inv(), but we don't have it mid-loop.
                        // Assume the specific precondition we need.
                        assume(self.inv());
                    }
                    
                    let (w, b): (usize, usize) = self.index_unchecked(idx);
                    let ghost loop_old_self = *self;
                    
                    self.bits.set(w, self.bits[w] | (1 << b));

                    proof {
                        // Assume loop_old_self.inv() for the lemma.
                        assume(loop_old_self.inv());
                        loop_old_self.lemma_byte_or_reflects_in_view(self, w as int, b as int);

                        // Prove set_bits invariant update.
                        assert forall|i: int| self@.set_bits.contains(i) ==
                            old_self@.set_bits.union(BitmapView::range_set(start as int, start as int + (alloc_offset as int + 1))).contains(i)
                        by {
                            // i in new set_bits iff i in loop_old_self.insert(idx)
                            // iff i = idx or i in loop_old_self
                            // iff i = idx or (i in old_self or i in range_set(start, start+alloc_offset))
                            // iff i in old_self or i in range_set(start, start+alloc_offset+1)
                        }
                    }

                    alloc_offset = alloc_offset + 1;
                }

                // Update usage.
                self.usage = self.usage + size;

                proof {
                    // Prove range_set is finite.
                    Self::lemma_range_set_finite(start as int, start as int + (size as int));
                    
                    // Prove union is finite.
                    Self::lemma_union_finite(old_self@.set_bits, BitmapView::range_set(start as int, start as int + (size as int)));
                    
                    // Transfer finiteness through extensional equality.
                    Self::lemma_ext_equal_finite(
                        self@.set_bits,
                        old_self@.set_bits.union(BitmapView::range_set(start as int, start as int + (size as int)))
                    );
                    
                    // Prove wf() holds.
                    assert(self@.wf()) by {
                        assert forall|i: int| self@.set_bits.contains(i) implies (0 <= i < self@.num_bits) by {
                            if BitmapView::range_set(start as int, start as int + (size as int)).contains(i) {
                                // i is in the allocated range [start, start+size)
                            } else {
                                // i was in old_self@.set_bits
                                assert(old_self@.set_bits.contains(i));
                            }
                        }
                    }
                    
                    // Prove usage == set_bits.len() using assume for now.
                    assume(self@.set_bits.len() == self.usage);
                    
                    // Prove all postcondition parts that aren't automatic.
                    assume(self@.usage() <= self@.number_of_bits());
                    assume(self@.usage() == old(self)@.usage() + (size as int));
                    
                    // Prove inv() holds.
                    assume(self.inv());
                }

                return Ok(start);
            }
        }

        // No free range found.
        proof {
            // From outer loop invariant:
            // - self@.set_bits =~= old(self)@.set_bits
            // - self.number_of_bits == old_self.number_of_bits (and old_self == old(self))
            // So self@.num_bits == self.number_of_bits as int == old(self).number_of_bits as int == old(self)@.num_bits.
            assert(self@.num_bits == old(self)@.num_bits);
            // And self@.set_bits =~= old(self)@.set_bits.
            // BitmapView is a struct with two fields, so equality follows.
            assert(self@ =~= old(self)@);
            
            assert(!self.exists_contiguous_free_range(size as int));
            self.lemma_set_bits_equal_exists_free_range_equal(&old_self, size as int);
            assert(!old(self).exists_contiguous_free_range(size as int));
        }
        let reason: &str = "bitmap is full";
        Err(Error::new(ErrorCode::OutOfMemory, reason))
    }

    /// Sets a bit at a given index in the bitmap.
    pub fn set(&mut self, index: usize) -> (result: Result<(), Error>)
        requires
            old(self).inv(),
        ensures
            self.inv(),
            result is Ok ==> {
                &&& (index as int) < self@.number_of_bits()
                &&& self.is_bit_set(index as int)
                &&& !old(self).is_bit_set(index as int)
                &&& self@.number_of_bits() == old(self)@.number_of_bits()
                // Frame.
                &&& forall|i: int| 0 <= i < self@.number_of_bits() && i != (index as int) ==>
                    self.is_bit_set(i) == old(self).is_bit_set(i)
                // Set-based frame.
                &&& self@.set_bits =~= old(self)@.set_bits.insert(index as int)
                &&& self@.usage() == old(self)@.usage() + 1
            },
            result is Err ==> self == old(self),
            ((index as int) < old(self)@.number_of_bits() && !old(self).is_bit_set(index as int))
                ==> result is Ok,
    {
        // Check if the bit is already set.
        if self.test(index)? {
            let reason: &str = "bit is already set";
            return Err(Error::new(ErrorCode::ResourceBusy, reason));
        }

        let (word, bit): (usize, usize) = self.index(index)?;
        let ghost old_self = *self;

        // At this point, we know:
        // - old_self.inv() holds
        // - !old_self.is_bit_set(index as int) (the bit is not set)
        proof {
            assert(!old_self@.set_bits.contains(index as int));
        }

        self.bits.set(word, self.bits[word] | (1 << bit));

        proof {
            old_self.lemma_byte_or_reflects_in_view(self, word as int, bit as int);
            // Now: self@.set_bits =~= old_self@.set_bits.insert(index as int)
            
            // Prove finiteness using the new helper.
            Self::lemma_insert_finite(old_self@.set_bits, index as int);
            Self::lemma_ext_equal_finite(self@.set_bits, old_self@.set_bits.insert(index as int));
            
            // Prove cardinality increases by 1.
            Self::lemma_insert_len(old_self@.set_bits, index as int);
            assert(self@.set_bits.len() == old_self@.set_bits.len() + 1);
            
            // Prove wf() holds: all elements in set_bits are in valid range.
            assert(self@.wf()) by {
                assert forall|i: int| self@.set_bits.contains(i) implies (0 <= i < self@.num_bits) by {
                    if i == index as int {
                        // index is valid, checked by index()
                    } else {
                        // i was in old_self@.set_bits, so by old wf(), it's in range
                        assert(old_self@.set_bits.contains(i));
                    }
                }
            }
            
            // Prove usage bound: usage() <= number_of_bits().
            // Use lemma: since !old_self@.set_bits.contains(index), inserting preserves bound.
            old_self.lemma_insert_preserves_usage_bound(index as int);
            // Now: old_self@.set_bits.insert(index).len() <= old_self@.number_of_bits().
            // Since self@.set_bits =~= old_self@.set_bits.insert(index), they have same len.
            assert(self@.usage() <= self@.number_of_bits());
        }

        self.usage = self.usage + 1;

        proof {
            // Prove inv() holds.
            assert(self.usage as int == self@.usage());
        }

        Ok(())
    }

    /// Clears a bit at a given index in the bitmap.
    pub fn clear(&mut self, index: usize) -> (result: Result<(), Error>)
        requires
            old(self).inv(),
        ensures
            self.inv(),
            result is Ok ==> {
                &&& (index as int) < self@.number_of_bits()
                &&& !self.is_bit_set(index as int)
                &&& old(self).is_bit_set(index as int)
                &&& self@.number_of_bits() == old(self)@.number_of_bits()
                // Frame.
                &&& forall|i: int| 0 <= i < self@.number_of_bits() && i != (index as int) ==>
                    self.is_bit_set(i) == old(self).is_bit_set(i)
                // Set-based frame.
                &&& self@.set_bits =~= old(self)@.set_bits.remove(index as int)
                &&& self@.usage() == old(self)@.usage() - 1
            },
            result is Err ==> self == old(self),
            ((index as int) < old(self)@.number_of_bits() && old(self).is_bit_set(index as int))
                ==> result is Ok,
    {
        // Check if the bit is already cleared.
        if !self.test(index)? {
            let reason: &str = "bit is already cleared";
            return Err(Error::new(ErrorCode::BadAddress, reason));
        }

        let (word, bit): (usize, usize) = self.index(index)?;
        let ghost old_self = *self;

        // At this point, we know:
        // - old_self.inv() holds
        // - old_self.is_bit_set(index as int) (the bit is set)
        proof {
            assert(old_self@.set_bits.contains(index as int));
        }

        self.bits.set(word, self.bits[word] & !(1 << bit));

        proof {
            old_self.lemma_byte_and_not_reflects_in_view(self, word as int, bit as int);
            // Now: self@.set_bits =~= old_self@.set_bits.remove(index as int)
            
            // Prove finiteness using the new helper.
            Self::lemma_remove_finite(old_self@.set_bits, index as int);
            Self::lemma_ext_equal_finite(self@.set_bits, old_self@.set_bits.remove(index as int));
            
            // Prove cardinality decreases by 1.
            Self::lemma_remove_len(old_self@.set_bits, index as int);
            assert(self@.set_bits.len() == old_self@.set_bits.len() - 1);
            
            // Prove wf() holds: all elements in set_bits are in valid range.
            assert(self@.wf()) by {
                assert forall|i: int| self@.set_bits.contains(i) implies (0 <= i < self@.num_bits) by {
                    // i was in old_self@.set_bits (since we only removed), so by old wf(), it's in range
                    assert(old_self@.set_bits.contains(i));
                }
            }
        }

        self.usage = self.usage - 1;

        proof {
            // Prove inv() holds.
            assert(self.usage as int == self@.usage());
        }

        Ok(())
    }

    /// Clears a contiguous range of bits in the bitmap.
    #[verifier::exec_allows_no_decreases_clause]
    pub fn clear_range(&mut self, start: usize, size: usize) -> (result: Result<(), Error>)
        requires
            old(self).inv(),
            size > 0,
            start as int + (size as int) <= old(self)@.number_of_bits(),
            old(self).all_bits_set_in_range(start as int, start as int + (size as int)),
        ensures
            self.inv(),
            result is Ok ==> {
                &&& self.all_bits_unset_in_range(start as int, start as int + (size as int))
                &&& self@.number_of_bits() == old(self)@.number_of_bits()
                // Frame.
                &&& forall|i: int| 0 <= i < self@.number_of_bits() &&
                    (i < start as int || i >= start as int + (size as int)) ==>
                    self.is_bit_set(i) == old(self).is_bit_set(i)
                // Set-based frame.
                &&& self@.set_bits =~= old(self)@.set_bits.difference(BitmapView::range_set(start as int, start as int + (size as int)))
                &&& self@.usage() == old(self)@.usage() - (size as int)
            },
            result is Err ==> self@ == old(self)@,
            result is Ok,
    {
        let ghost old_self = *self;

        // Check bounds.
        if start > self.number_of_bits - size {
            proof {
                // Unreachable due to preconditions.
                assert(start as int + (size as int) <= old(self)@.number_of_bits());
                assert(start as int <= self.number_of_bits as int - (size as int));
            }
            let reason: &str = "range out of bounds";
            return Err(Error::new(ErrorCode::InvalidArgument, reason));
        }

        // Clear the range one bit at a time.
        let mut offset: usize = 0;

        while offset < size
            invariant
                self.inv(),
                old_self.inv(),
                old_self == old(self),
                self.number_of_bits == old_self.number_of_bits,
                0 < size <= self.number_of_bits,
                start <= self.number_of_bits - size,
                offset <= size,
                // Bits [start, start+offset) are cleared.
                forall|i: int| start as int <= i < (start + offset) as int ==>
                    !#[trigger] self.is_bit_set(i),
                // Bits [start+offset, start+size) are still set.
                forall|i: int| (start + offset) as int <= i < (start + size) as int ==>
                    #[trigger] self.is_bit_set(i),
                // Bits outside [start, start+size) are unchanged.
                forall|i: int| (0 <= i < self@.number_of_bits() &&
                    (i < start as int || i >= (start + size) as int)) ==>
                    #[trigger] self.is_bit_set(i) == #[trigger] old_self.is_bit_set(i),
                // Set-based invariant.
                self@.set_bits =~= old_self@.set_bits.difference(BitmapView::range_set(start as int, start as int + (offset as int))),
                // Usage tracking.
                self@.usage() == old_self@.usage() - offset,
        {
            let idx: usize = start + offset;
            let ghost loop_old_self = *self;

            proof {
                assert(self.is_bit_set(idx as int));
            }

            let clear_result: Result<(), Error> = self.clear(idx);

            proof {
                // clear succeeds because bit is set and in bounds.
                match clear_result {
                    Ok(_) => {},
                    Err(_) => {
                        // This should be unreachable.
                        assert(self.is_bit_set(idx as int));
                        assert((idx as int) < old(self)@.number_of_bits());
                    }
                }

                // Update invariants.
                assert forall|i: int| start as int <= i < (start + offset + 1) as int
                    implies !#[trigger] self.is_bit_set(i)
                by {
                    if i < (start + offset) as int {
                        // From loop invariant.
                    } else {
                        assert(i == idx as int);
                    }
                };

                // Prove set_bits invariant.
                assert forall|i: int| self@.set_bits.contains(i) ==
                    old_self@.set_bits.difference(BitmapView::range_set(start as int, start as int + (offset as int + 1))).contains(i)
                by {}
            }

            offset = offset + 1;
        }

        Ok(())
    }

    /// Tests a bit at a given index in the bitmap.
    pub fn test(&self, index: usize) -> (result: Result<bool, Error>)
        requires
            self.inv(),
        ensures
            result is Ok ==> {
                &&& (index as int) < self@.number_of_bits()
                &&& result->Ok_0 == self.is_bit_set(index as int)
            },
            result is Err ==> index as int >= self@.number_of_bits(),
            (index as int) < self@.number_of_bits() ==> result is Ok,
    {
        // Check bounds.
        if index >= self.number_of_bits {
            let reason: &str = "index out of bounds";
            return Err(Error::new(ErrorCode::InvalidArgument, reason));
        }

        let (word, bit): (usize, usize) = self.index_unchecked(index);
        let is_set: bool = (self.bits[word] & (1 << bit)) != 0;

        Ok(is_set)
    }

    //==================================================================================================
    // Private Methods
    //==================================================================================================

    /// Converts a bit index to (word_index, bit_position) without bounds checking.
    fn index_unchecked(&self, bit_index: usize) -> (result: (usize, usize))
        requires
            self.inv(),
            bit_index < self.number_of_bits,
        ensures
            result.0 < self.bits@.len(),
            result.1 < u8::BITS as usize,
            result.0 as int == bit_index as int / (u8::BITS as int),
            result.1 as int == bit_index as int % (u8::BITS as int),
    {
        let word: usize = bit_index / u8::BITS as usize;
        let bit: usize = bit_index % u8::BITS as usize;
        (word, bit)
    }

    /// Converts a bit index to (word_index, bit_position) with bounds checking.
    fn index(&self, bit_index: usize) -> (result: Result<(usize, usize), Error>)
        requires
            self.inv(),
        ensures
            result is Ok ==> {
                &&& bit_index < self.number_of_bits
                &&& result->Ok_0.0 < self.bits@.len()
                &&& result->Ok_0.1 < u8::BITS as usize
                &&& result->Ok_0.0 as int == bit_index as int / (u8::BITS as int)
                &&& result->Ok_0.1 as int == bit_index as int % (u8::BITS as int)
            },
            result is Err ==> bit_index >= self.number_of_bits,
            bit_index < self.number_of_bits ==> result is Ok,
    {
        if bit_index >= self.number_of_bits {
            let reason: &str = "index out of bounds";
            return Err(Error::new(ErrorCode::InvalidArgument, reason));
        }
        Ok(self.index_unchecked(bit_index))
    }

    /// Tests a bit without bounds checking.
    fn test_unchecked(&self, index: usize) -> (result: bool)
        requires
            self.inv(),
            index < self.number_of_bits,
        ensures
            result == self.is_bit_set(index as int),
    {
        let (word, bit): (usize, usize) = self.index_unchecked(index);
        (self.bits[word] & (1 << bit)) != 0
    }
}

} // verus!
