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
        is_zero,
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
        // Check if the length is invalid.
        if number_of_bits == 0 || number_of_bits >= u32::MAX as usize {
            let reason: &str = "invalid length";
            return Err(Error::new(ErrorCode::InvalidArgument, reason));
        }

        // Check if the length is not a multiple of the number of the bitmap word.
        if !number_of_bits.is_multiple_of(u8::BITS as usize) {
            let reason: &str = "length must be a multiple of 8";
            return Err(Error::new(ErrorCode::InvalidArgument, reason));
        }

        // Allocate the bitmap.
        // Note: RawArray::new() guarantees zero-initialization of the backing storage.
        let array: RawArray<u8> = RawArray::new(number_of_bits / u8::BITS as usize)?;

        let result = Self {
            number_of_bits,
            bits: array,
            usage: 0,
        };

        proof {
            // RawArray::new ensures is_zero(array@[i]) for all i.
            // Convert is_zero to == 0 via axiom.
            assert forall|i: int| 0 <= i < result.bits@.len() implies (result.bits@[i] == 0) by {
                axiom_u8_zero_is_0(result.bits@[i]);
            };
            result.lemma_zero_bytes_means_empty_set();
            Self::lemma_empty_set_finite();
        }

        Ok(result)
    }

    /// Creates a new bitmap from a raw array.
    ///
    /// # Note
    ///
    /// RawArray guarantees zero-initialization of the backing storage.
    /// The caller must ensure the array is zero-initialized.
    ///
    /// # Errors
    ///
    /// - `InvalidArgument` if the array length multiplied by 8 overflows `usize`.
    pub fn from_raw_array(array: RawArray<u8>) -> (result: Result<Self, Error>)
        requires
            array@.len() > 0,
            array@.len() * (u8::BITS as usize) < u32::MAX as usize,
            forall|i: int| 0 <= i < array@.len() ==> array@[i] == 0,
        ensures
            result is Ok ==> {
                let bitmap = result->Ok_0;
                &&& bitmap.inv()
                &&& bitmap@.number_of_bits() == array@.len() * (u8::BITS as int)
                &&& bitmap@.is_empty()
                &&& forall|i: int| 0 <= i < bitmap@.number_of_bits() ==> !bitmap.is_bit_set(i)
            },
            // Liveness: given preconditions, always succeeds.
            result is Ok,
    {
        // Check for overflow: array.len() * u8::BITS would overflow usize.
        // Verus note: checked_mul and closures are not supported in Verus,
        // so we use a manual overflow check. Semantically equivalent to
        // source's array.len().checked_mul(u8::BITS as usize).ok_or_else(...).
        if array.len() > usize::MAX / (u8::BITS as usize) {
            let reason: &str = "bitmap size overflow: array too large";
            return Err(Error::new(ErrorCode::InvalidArgument, reason));
        }
        let number_of_bits: usize = array.len() * u8::BITS as usize;

        let result = Self {
            number_of_bits,
            bits: array,
            usage: 0,
        };
        proof {
            result.lemma_zero_bytes_means_empty_set();
            Self::lemma_empty_set_finite();
        }
        Ok(result)
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

        // Note: debug_assert_eq! is not supported by Verus, so we guard it
        // with cfg. The invariant self.inv() already proves this property.
        #[cfg(not(verus_keep_ghost))]
        debug_assert_eq!(
            self.bits.len() * u8::BITS as usize,
            self.number_of_bits,
            "bitmap length must match the number of bits"
        );

        let mut start: usize = 0;

        // Search for a contiguous free range.
        while start <= self.number_of_bits - size
            invariant
                self.inv(),
                old_self.inv(),
                old_self == *old(self),
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
            decreases
                self.number_of_bits - start as int,
        {
            // Fast skip: if the starting word is full, skip 8 bits.
            let is_aligned: bool = start.is_multiple_of(u8::BITS as usize);
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

            // Check if all bits in the range are free.
            let ghost start_before_inner: usize = start;
            let mut offset: usize = 0;
            let mut free: bool = true;

            while offset < size
                invariant_except_break
                    start == start_before_inner,  // start doesn't change until break
                    free,  // free remains true unless we break
                invariant
                    self.inv(),
                    old_self.inv(),
                    old_self == *old(self),
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
                    !free ==> start > start_before_inner,
                    !free ==> forall|p: int| #![trigger self.has_free_range_at(p, size as int)]
                        0 <= p < start as int ==> !self.has_free_range_at(p, size as int),
                decreases
                    size - offset,
            {
                let idx: usize = start + offset;
                let (w, b): (usize, usize) = self.index_unchecked(idx);
                if (self.bits[w] & (1u8 << b)) != 0 {
                    free = false;
                    start += offset + 1;
                    proof {
                        // Bit at idx is set, so no free range can include idx.
                        assert forall|p: int| #![trigger self.has_free_range_at(p, size as int)]
                            start_before_inner as int <= p <= idx as int implies !self.has_free_range_at(p, size as int)
                        by {
                            if self.has_free_range_at(p, size as int) {
                                assert(p <= idx as int);
                                assert((idx as int) - p <= (offset as int));
                                assert((offset as int) < (size as int));
                                assert((idx as int) < p + (size as int));
                                assert(self.is_bit_set(idx as int));
                                assert(self.all_bits_unset_in_range(p, p + (size as int)));
                                assert(!self.is_bit_set(idx as int));
                            }
                        }
                    }
                    break;
                }
                offset += 1;
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
                    assert(self@.number_of_bits() == old_self@.number_of_bits());
                    assert(pre_alloc_self@.number_of_bits() == old_self@.number_of_bits());
                }

                // Verus note: `for offset in 0..size` is not supported;
                // `self.bits[w] |= 1 << b` is not supported for mutable index.
                while alloc_offset < size
                    invariant
                        // Basic structure preservation.
                        self.bits@.len() == pre_alloc_self.bits@.len(),
                        self.bits@.len() == old_self.bits@.len(),
                        self@.number_of_bits() > 0,
                        self@.number_of_bits() == self.bits@.len() * (u8::BITS as int),
                        self.number_of_bits == pre_alloc_self.number_of_bits,
                        self.number_of_bits as int == self@.number_of_bits(),
                        // Usage unchanged during this loop (updated after).
                        self.usage == pre_alloc_self.usage,
                        // Ghost state.
                        old_self.inv(),
                        pre_alloc_self.inv(),
                        old_self == *old(self),
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
                        self@.set_bits.finite(),
                        // The range [start, start+size) was free in old_self.
                        old_self.all_bits_unset_in_range(start as int, start as int + (size as int)),
                    decreases
                        size - alloc_offset,
                {
                    let idx: usize = start + alloc_offset;
                    let (w, b): (usize, usize) = self.index_unchecked(idx);
                    let ghost loop_old_self = *self;

                    self.bits.set(w, self.bits[w] | (1u8 << b));

                    proof {
                        loop_old_self.lemma_byte_or_reflects_in_view(self, w as int, b as int);

                        // Prove set_bits invariant update.
                        assert forall|i: int| self@.set_bits.contains(i) ==
                            old_self@.set_bits.union(BitmapView::range_set(start as int, start as int + (alloc_offset as int + 1))).contains(i)
                        by {}

                        // Prove wf() preserved.
                        assert(self@.wf()) by {
                            assert forall|i: int| self@.set_bits.contains(i) implies (0 <= i < self@.num_bits) by {
                                if loop_old_self@.set_bits.contains(i) {}
                            }
                        }
                        Self::lemma_insert_finite(loop_old_self@.set_bits, idx as int);
                        assert(self@.set_bits.finite());
                    }

                    alloc_offset += 1;
                }
                // Verus note: compound assignment on struct fields not supported.
                // Equivalent to source's `self.usage += size`.
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
                            } else {
                                assert(old_self@.set_bits.contains(i));
                            }
                        }
                    }

                    // Prove the sets are disjoint: old_self.set_bits ∩ range_set = ∅.
                    let range: Set<int> = BitmapView::range_set(start as int, start as int + (size as int));
                    assert(old_self@.set_bits.disjoint(range)) by {
                        assert forall|i: int| #![auto] !(old_self@.set_bits.contains(i) && range.contains(i)) by {
                            if range.contains(i) {
                                assert(old(self).all_bits_unset_in_range(start as int, start as int + (size as int)));
                                assert(!old(self).is_bit_set(i));
                                assert(!old_self.is_bit_set(i));
                                assert(!old_self@.set_bits.contains(i));
                            }
                        }
                    }

                    // Use disjoint union cardinality lemma.
                    Self::lemma_disjoint_union_len(old_self@.set_bits, range);
                    Self::lemma_range_set_len(start as int, start as int + (size as int));
                    assert(self@.set_bits.len() == old_self@.set_bits.len() + (size as int));

                    // Prove usage matches set_bits.len().
                    assert(self.usage as int == self@.set_bits.len());
                    assert(self@.usage() == old(self)@.usage() + (size as int));

                    // Prove usage bound.
                    let full_range: Set<int> = vstd::set_lib::set_int_range(0, self@.num_bits);
                    vstd::set_lib::lemma_int_range(0, self@.num_bits);
                    assert(self@.set_bits.subset_of(full_range)) by {
                        assert forall|i: int| #![auto] self@.set_bits.contains(i) implies full_range.contains(i) by {}
                    }
                    vstd::set_lib::lemma_len_subset(self@.set_bits, full_range);

                    assert(self.inv());
                }

                return Ok(start);
            }
            // !free: start was advanced past the blocked position.
            proof {
                assert(start > start_before_inner);
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
            result is Err ==> *self == *old(self),
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
            result is Err ==> *self == *old(self),
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
        let (word, bit): (usize, usize) = self.index(index)?;
        let byte_val: u8 = self.bits[word];
        let result_val: bool = (byte_val & (1 << bit)) != 0;

        Ok(result_val)
    }

    //==================================================================================================
    // Private Methods
    //==================================================================================================

    /// Converts a bit index to (word_index, bit_position) without bounds checking.
    fn index_unchecked(&self, bit_index: usize) -> (result: (usize, usize))
        requires
            bit_index < self.bits@.len() * u8::BITS as usize,
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
        // Check if the index is out of bounds.
        if bit_index >= self.bits.len() * u8::BITS as usize {
            let reason: &str = "index out of bounds";
            return Err(Error::new(ErrorCode::InvalidArgument, reason));
        }
        Ok(self.index_unchecked(bit_index))
    }
}

} // verus!

// Include verified tests.
include!("lib.test.rs");

// Deref implementation for test support (external to verification).
#[cfg(test)]
#[verifier::external]
impl ::core::ops::Deref for Bitmap {
    type Target = RawArray<u8>;

    fn deref(&self) -> &Self::Target {
        &self.bits
    }
}
