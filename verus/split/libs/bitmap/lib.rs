// Copyright(c) The Maintainers of Nanvix.
// Licensed under the MIT License.

//! # Bitmap Allocator - Implementation
//!
//! This file contains the implementation code for bitmap allocator.
//! Specification functions are in `lib.spec.rs` and proofs are in `lib.proof.rs`.

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
    pub fn new(number_of_bits: usize) -> (result: Result<Self, Error>)
        ensures
            result is Ok ==> {
                let bitmap = result->Ok_0;
                &&& bitmap.inv()
                &&& bitmap@.number_of_bits() == number_of_bits as int
                &&& bitmap@.is_empty()
                // Set-based: no bits are set initially.
                &&& bitmap@.set_bits =~= Set::<int>::empty()
            },
            // Error case: at least one of these conditions must hold.
            // Note: RawArray::new may also fail, so we cannot fully enumerate error causes.
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
        if number_of_bits % (u8::BITS as usize) != 0 {
            let reason: &str = "length must be a multiple of 8";
            return Err(Error::new(ErrorCode::InvalidArgument, reason));
        }

        // Allocate the bitmap.
        let array: RawArray<u8> = RawArray::new(number_of_bits / u8::BITS as usize)?;

        // NOTE: the bitmap is already zeroed out by RawArray::new()

        let result = Self {
            number_of_bits,
            bits: array,
            usage: 0,
        };

        proof {
            assert forall|i: int| 0 <= i < result.bits@.len() implies (result.bits@[i] == 0) by {
                axiom_u8_zero_is_0(result.bits@[i]);
            };
            result.lemma_zero_bytes_means_false_bits();
            Self::lemma_all_zero_in_seq_implies_count_zero(result@.bits, 0, result@.number_of_bits());
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
                // Set-based: no bits are set initially.
                &&& bmp@.set_bits =~= Set::<int>::empty()
            },
    {
        Self::new(number_of_bits)
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
    pub fn from_raw_array(array: RawArray<u8>) -> (result: Self)
        requires
            array@.len() > 0, // Required since inv() requires number_of_bits > 0.
            array@.len() <= usize::MAX / (u8::BITS as usize),
            array@.len() * (u8::BITS as usize) < u32::MAX as usize,
            forall|i: int| 0 <= i < array@.len() ==> array@[i] == 0,
        ensures
            result.inv(),
            result@.number_of_bits() == array@.len() * (u8::BITS as int),
            result@.is_empty(),
            // Forall-based: no bits are set initially (for backward compatibility).
            forall|i: int| 0 <= i < result@.number_of_bits() ==> !result.is_bit_set(i),
            // Set-based: no bits are set initially.
            result@.set_bits =~= Set::<int>::empty(),
    {
        // NOTE: no need to test if the length of the raw array is valid, as it is by construction.
        // NOTE: the bitmap is already zeroed out by RawArray::new() or RawArray::from_raw_parts()

        let result = Self {
            number_of_bits: array.len() * u8::BITS as usize,
            bits: array,
            usage: 0,
        };
        proof {
            result.lemma_zero_bytes_means_false_bits();
            Self::lemma_all_zero_in_seq_implies_count_zero(result@.bits, 0, result@.number_of_bits());
        }
        result
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
    pub fn number_of_bits(&self) -> (result: usize)
        requires
            self.inv(),
        ensures
            result as int == self@.number_of_bits(),
            result > 0, // Follows from inv() which requires number_of_bits > 0.
            result < u32::MAX as usize,
    {
        self.number_of_bits
    }

    ///
    /// # Description
    ///
    /// Returns the number of bits set (usage count) in the bitmap.
    ///
    /// # Returns
    ///
    /// The number of bits set in the bitmap.
    ///
    pub fn usage(&self) -> (result: usize)
        requires
            self.inv(),
        ensures
            result as int == self@.usage(),
    {
        self.usage
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
                // Forall-based frame (for backward compatibility with callers).
                &&& forall|i: int| 0 <= i < self@.number_of_bits() && i != index ==>
                    self.is_bit_set(i) == old(self).is_bit_set(i)
                // Set-based frame: set_bits is old plus the new index.
                &&& self@.set_bits =~= old(self)@.set_bits.insert(index)
                &&& self@.usage() == old(self)@.usage() + 1
            },
            result is Err ==> self@ == old(self)@,
            // Liveness: if there's a free bit, allocation succeeds.
            old(self)@.has_free_bit() ==> result is Ok,
    {
        proof {
            // Prove that has_free_bit implies exists_contiguous_free_range(1).
            if old(self)@.has_free_bit() {
                old(self).lemma_has_free_bit_implies_exists_free_range_1();
            }
        }
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
                // Forall-based frame (for backward compatibility with callers).
                &&& forall|i: int| 0 <= i < self@.number_of_bits() &&
                    (i < start || i >= start + (size as int)) ==>
                    self.is_bit_set(i) == old(self).is_bit_set(i)
                // Set-based frame: set_bits is old union the allocated range.
                &&& self@.set_bits =~= old(self)@.set_bits.union(BitmapView::range_set(start, start + (size as int)))
                &&& self@.usage() == old(self)@.usage() + (size as int)
            },
            result is Err ==> self@ == old(self)@,
            // Liveness: if a contiguous free range of the requested size exists (and size > 0), allocation succeeds.
            // Note: size must be > 0 because exists_contiguous_free_range(0) is trivially true for any bitmap.
            (size > 0 && old(self).exists_contiguous_free_range(size as int)) ==> result is Ok,
    {
        let ghost old_self = *self;

        // Check if the size is valid.
        if size == 0 || size > self.number_of_bits {
            proof {
                // Case 1: size == 0
                // The postcondition is: (size > 0 && exists_free_range) ==> Ok.
                // Since size == 0, the antecedent (size > 0 && ...) is false.
                // So the postcondition is vacuously true.

                // Case 2: size > number_of_bits
                // has_free_range_at(start, size) requires start + size <= number_of_bits.
                // For any start >= 0: start + size >= size > number_of_bits.
                // So start + size > number_of_bits, violating the condition.
                // Therefore !has_free_range_at(start, size) for all start >= 0.
                // Hence !exists_contiguous_free_range(size).
                if size > self.number_of_bits {
                    assert forall|start: int| #![trigger old_self.has_free_range_at(start, size as int)]
                        0 <= start implies !old_self.has_free_range_at(start, size as int)
                    by {
                        // start + size >= size > number_of_bits
                        assert(start + (size as int) >= size as int);
                        assert((size as int) > self@.number_of_bits());
                        // Therefore start + size > number_of_bits.
                    }
                    assert(!old_self.exists_contiguous_free_range(size as int));
                    assert(!old(self).exists_contiguous_free_range(size as int));
                }
                // In either case, the postcondition (size > 0 && exists_free_range ==> Ok) is satisfied:
                // - For size == 0: antecedent is false.
                // - For size > number_of_bits: !exists_free_range, so antecedent is false.
            }
            let reason: &str = "invalid size";
            return Err(Error::new(ErrorCode::InvalidArgument, reason));
        }

        // Check if allocation exceeds the bitmap capacity.
        if self.usage > self.number_of_bits - size {
            proof {
                // If usage > number_of_bits - size, then there are fewer than `size` free bits.
                // Therefore, no contiguous range of `size` free bits can exist.
                // count_free = number_of_bits - usage < size.
                assert(self@.count_free() < size as int);
                // old_self == old(self) at this point (no mutations yet).
                assert(old_self@ == self@);

                // Any contiguous free range of size `size` would require at least `size` free bits.
                // Since count_free < size, no such range exists.
                // We need to prove: !exists_contiguous_free_range(size).
                // This is implied by count_free < size.

                // To prove this formally, assume exists a free range [p, p+size).
                // Then all size bits in that range are unset (free).
                // This means usage() would be at most number_of_bits - size.
                // But we have usage > number_of_bits - size, contradiction.
                assert forall|p: int| #![trigger old_self.has_free_range_at(p, size as int)]
                    0 <= p <= old_self@.number_of_bits() - (size as int) implies !old_self.has_free_range_at(p, size as int)
                by {
                    // Suppose has_free_range_at(p, size) were true.
                    // Then all_bits_unset_in_range(p, p+size) would be true.
                    // This means forall i in [p, p+size): !is_bit_set(i).
                    // So at least `size` bits are unset, meaning count_free >= size.
                    // But count_free < size, contradiction.
                    if old_self.has_free_range_at(p, size as int) {
                        // has_free_range_at implies all_bits_unset_in_range.
                        assert(old_self.all_bits_unset_in_range(p, p + (size as int)));
                        // Use the lemma on old_self.
                        old_self.lemma_free_range_implies_usage_bound(p, size as int);
                        // This gives: old_self@.usage() <= old_self@.number_of_bits() - size.
                        // But old_self@.usage() == self.usage > self.number_of_bits - size.
                        assert(false); // Contradiction.
                    }
                }

                assert(!old_self.exists_contiguous_free_range(size as int));

                // old_self == old(self).
                assert(!old(self).exists_contiguous_free_range(size as int));

                // For size=1: also prove the has_free_bit implication.
                if size == 1 {
                    assert(self.usage as int > self.number_of_bits as int - 1);
                    assert(self.usage as int >= self.number_of_bits as int);
                    // From inv: usage <= number_of_bits.
                    assert(self.usage as int == self.number_of_bits as int);
                    // is_full() = usage() == number_of_bits().
                    assert(self@.is_full());
                    // is_full() implies !has_free_bit() by our lemma.
                    self.lemma_is_full_implies_no_free_bit();
                    assert(!self@.has_free_bit());
                }
            }
            let reason: &str = "allocation exceeds bitmap capacity";
            return Err(Error::new(ErrorCode::OutOfMemory, reason));
        }

        let mut start: usize = 0;

        // Traverse the bitmap until the last possible starting bit.
        while start <= self.number_of_bits - size
            invariant
                self.inv(),
                old_self.inv(),
                old_self == old(self),
                size > 0,
                size <= self.number_of_bits,
                start <= self.number_of_bits,
                self@.bits == old(self)@.bits,
                self@.set_bits =~= old(self)@.set_bits,
                self.usage <= self.number_of_bits - size,
                // For size=1: all bits before start are set (checked and found occupied).
                size == 1 ==> forall|i: int| 0 <= i < start as int ==> self.is_bit_set(i),
                // General invariant: no contiguous free range of size `size` starts before `start`.
                forall|p: int| #![trigger self.has_free_range_at(p, size as int)]
                    0 <= p < start as int ==> !self.has_free_range_at(p, size as int),
        {
            // Check for fast skip/ path.
            let is_aligned: bool = start % (u8::BITS as usize) == 0;
            if is_aligned {
                let word: usize = start / u8::BITS as usize;
                // Fast skip: if the starting word is full, skip to the next word.
                if self.bits[word] == u8::MAX {
                    proof {
                        // When a byte is 0xFF (u8::MAX), all 8 bits are set.
                        // Prove that bits start..start+8 are all set.
                        assert forall|i: int| start as int <= i < start as int + 8 implies
                            self.is_bit_set(i)
                        by {
                            let bit_pos: int = i % 8;
                            let bit_pos_u8: u8 = bit_pos as u8;
                            // u8::MAX = 0xFF. For any bit position 0-7, (0xFF & (1 << b)) != 0.
                            assert((0xFFu8 & (1u8 << bit_pos_u8)) != 0) by (bit_vector)
                                requires 0 <= bit_pos_u8 < 8;
                        }

                        // Prove: no contiguous free range of size `size` starts in [start, start+8).
                        // For any p in [start, start+8), the bit at p is set, so the range [p, p+size) contains a set bit.
                        assert forall|p: int| #![trigger self.has_free_range_at(p, size as int)]
                            start as int <= p < (start + 8) as int implies !self.has_free_range_at(p, size as int)
                        by {
                            // p is in [start, start+8), and we know is_bit_set(p).
                            assert(self.is_bit_set(p));
                            // A free range at p requires all bits in [p, p+size) to be unset.
                            // But bit p is set, so it's not a free range.
                        }

                        // Combined with the loop invariant, no free range starts in [0, start+8).
                        assert forall|p: int| #![trigger self.has_free_range_at(p, size as int)]
                            0 <= p < (start + 8) as int implies !self.has_free_range_at(p, size as int)
                        by {
                            if p < start as int {
                                // From loop invariant.
                            } else {
                                // From above assertion.
                            }
                        }
                    }
                    // Jump to next byte boundary.
                    start = start + u8::BITS as usize;
                    continue;
                }
            }

            // Check if all bits in the range are free.
            let mut free: bool = true;
            let mut offset: usize = 0;
            let ghost start_before_inner = start;
            while offset < size
                invariant_except_break
                    start <= self.number_of_bits - size,
                    start == start_before_inner,
                    forall|i: int| 0 <= i < offset ==>
                        !#[trigger] self.is_bit_set((start + i) as int),
                invariant
                    self.inv(),
                    old_self == old(self),
                    0 < size <= self.number_of_bits,
                    offset <= size,
                    self@.bits == old(self)@.bits,
                    // For size=1: all bits before start_before_inner are set.
                    size == 1 ==> forall|i: int| 0 <= i < start_before_inner as int ==> self.is_bit_set(i),
                    // General invariant: no contiguous free range of size `size` starts before start_before_inner.
                    forall|p: int| #![trigger self.has_free_range_at(p, size as int)]
                        0 <= p < start_before_inner as int ==> !self.has_free_range_at(p, size as int),
                ensures
                    start <= self.number_of_bits,
                    free ==> start <= self.number_of_bits - size &&
                        forall|i: int| 0 <= i < size ==>
                            !#[trigger] self.is_bit_set((start + i) as int),
                    // On break (!free): all bits before new start are set (for size=1).
                    (size == 1 && !free) ==> forall|i: int| 0 <= i < start as int ==> self.is_bit_set(i),
                    // On break (!free): no contiguous free range of size `size` starts before new start.
                    !free ==> forall|p: int| #![trigger self.has_free_range_at(p, size as int)]
                        0 <= p < start as int ==> !self.has_free_range_at(p, size as int),
            {
                let idx: usize = start + offset;
                let (w, b): (usize, usize) = self.index_unchecked(idx);
                if (self.bits[w] & (1 << b)) != 0 {
                    proof {
                        // The bit at position idx = start + offset is set.
                        let bit_pos: u8 = b as u8;
                        assert((self.bits@[w as int] & (1u8 << bit_pos)) != 0);
                        assert(self.is_bit_set(idx as int));

                        // For size=1: offset=0, so idx = start.
                        // After break, start = start + 1.
                        // Invariant gives: forall|i| 0 <= i < start ==> is_bit_set(i).
                        // We just showed is_bit_set(start), so forall|i| 0 <= i < start+1 ==> is_bit_set(i).
                        if size == 1 {
                            assert(offset == 0);
                            assert(idx as int == start as int);
                            assert(self.is_bit_set(start as int));
                        }

                        // General case: The bit at idx = start + offset is set.
                        // Any range [p, p+size) where start_before_inner <= p <= start + offset
                        // must include idx (since p <= start + offset < p + size),
                        // hence cannot be a free range.
                        // After break, start = start + offset + 1.
                        // We need to prove: forall p in [0, start + offset + 1): !has_free_range_at(p, size).
                        let new_start = start + offset + 1;
                        assert forall|p: int| #![trigger self.has_free_range_at(p, size as int)]
                            0 <= p < new_start as int implies !self.has_free_range_at(p, size as int)
                        by {
                            if p < start_before_inner as int {
                                // From loop invariant: no free range before start_before_inner.
                            } else {
                                // p is in [start_before_inner, start + offset].
                                // The range [p, p+size) would include idx = start + offset if p <= idx < p + size.
                                // Since p <= start + offset and size >= 1, we have p + size > start + offset.
                                // Thus idx is in the range [p, p+size), which means the range includes the set bit at idx.
                                let idx_int: int = idx as int;
                                let size_int: int = size as int;
                                assert(p <= idx_int);
                                assert(idx_int < p + size_int);
                                // Therefore the range [p, p+size) contains a set bit and is not free.
                            }
                        }
                    }
                    free = false;
                    start = start + offset + 1;
                    break;
                }
                offset = offset + 1;
            }
            if free {
                // At this point: forall i in [0, size): !self.is_bit_set((start + i) as int)
                proof {
                    // Establish that these bits are not set in old_self as well
                    assert forall|i: int| 0 <= i < size implies
                        !#[trigger] old_self.is_bit_set((start + i) as int)
                    by {
                        assert(!self.is_bit_set((start + i) as int));
                    };
                }

                // Allocate the range
                let ghost pre_alloc_self = *self;

                proof {
                    // At this point, self hasn't been modified since old_self, so set_bits are equal.
                    // range_set(start, start + 0) is empty, so union with empty = old_self@.set_bits.
                    assert(BitmapView::range_set(start as int, start as int) =~= Set::<int>::empty());
                    assert(old_self@.set_bits.union(Set::<int>::empty()) =~= old_self@.set_bits);
                }

                let mut offset: usize = 0;
                while offset < size
                    invariant
                        self@.number_of_bits() == self.bits@.len() * (u8::BITS as int),
                        old_self.inv(),
                        pre_alloc_self.inv(),
                        old_self == old(self),
                        self.number_of_bits == pre_alloc_self.number_of_bits,
                        self.usage == pre_alloc_self.usage,
                        0 < size <= self.number_of_bits,
                        start <= self.number_of_bits - size,
                        offset <= size,
                        forall|i: int| 0 <= i < size ==>
                            !#[trigger] pre_alloc_self.is_bit_set((start + i) as int),
                        forall|i: int| 0 <= i < size ==>
                            !#[trigger] old_self.is_bit_set((start + i) as int),
                        forall|i: int| 0 <= i < offset ==>
                            #[trigger] self.is_bit_set((start + i) as int),
                        forall|i: int| (0 <= i < self@.number_of_bits() &&
                            (i < start as int || i >= (start + offset) as int)) ==>
                            #[trigger] self.is_bit_set(i) == #[trigger] pre_alloc_self.is_bit_set(i),
                        forall|i: int| (0 <= i < self@.number_of_bits() &&
                            (i < start as int || i >= (start + offset) as int)) ==>
                            #[trigger] self.is_bit_set(i) == #[trigger] old_self.is_bit_set(i),
                        self@.usage() == pre_alloc_self@.usage() + offset,
                        // Set-based invariant: set_bits == old union range [start, start+offset).
                        self@.set_bits =~= old_self@.set_bits.union(BitmapView::range_set(start as int, start as int + (offset as int))),
                {
                    let idx: usize = start + offset;
                    let (w, b): (usize, usize) = self.index_unchecked(idx);
                    let ghost loop_old_self = *self;
                    self.bits.set(w, self.bits[w] | (1 << b));

                    proof {
                        assert(loop_old_self.is_bit_set(idx as int) == pre_alloc_self.is_bit_set(idx as int));
                        loop_old_self.lemma_byte_or_reflects_in_view(self, w as int, b as int);
                        loop_old_self.lemma_set_bit_increases_count(self, idx as int);

                        assert forall|i: int| 0 <= i < offset + 1 implies
                            #[trigger] self.is_bit_set((start + i) as int)
                        by {
                            if i < offset {
                                assert(loop_old_self.is_bit_set((start + i) as int));
                            }
                        };

                        assert forall|i: int| (0 <= i < self@.number_of_bits() &&
                            (i < start as int || i >= (start + offset + 1) as int)) implies
                            #[trigger] self.is_bit_set(i) == #[trigger] pre_alloc_self.is_bit_set(i)
                        by {
                            if i < start as int || i >= (start + offset) as int {
                                assert(loop_old_self.is_bit_set(i) == pre_alloc_self.is_bit_set(i));
                            }
                        };

                        assert forall|i: int| (0 <= i < self@.number_of_bits() &&
                            (i < start as int || i >= (start + offset + 1) as int)) implies
                            #[trigger] self.is_bit_set(i) == #[trigger] old_self.is_bit_set(i)
                        by {
                            if i < start as int || i >= (start + offset) as int {
                                assert(loop_old_self.is_bit_set(i) == old_self.is_bit_set(i));
                            }
                        };

                        // Prove set_bits invariant update.
                        // We have: self@.set_bits =~= loop_old_self@.set_bits.insert(idx)
                        // And: loop_old_self@.set_bits =~= old_self@.set_bits.union(range_set(start, start+offset))
                        // Need: self@.set_bits =~= old_self@.set_bits.union(range_set(start, start+offset+1))
                        // This follows because: insert(idx) where idx == start+offset
                        //   union(range_set(start, start+offset)).insert(start+offset)
                        //   == union(range_set(start, start+offset+1))
                        assert forall|i: int| self@.set_bits.contains(i) ==
                            old_self@.set_bits.union(BitmapView::range_set(start as int, start as int + (offset as int + 1))).contains(i)
                        by {
                            // Case analysis
                            if i == idx as int {
                                // i is the newly inserted bit
                            } else if start as int <= i < start as int + (offset as int) {
                                // i is in the range we already allocated
                                assert(loop_old_self@.set_bits.contains(i));
                            } else if 0 <= i < self@.number_of_bits() {
                                // i is outside the allocation range
                            }
                        }
                    }

                    offset = offset + 1;
                }

                let ghost pre_assignment_bits_set_in_range: Set<int> = Set::new(|i: int| start as int <= i < start as int + (size as int) && self.is_bit_set(i));

                proof {
                    // At loop exit: offset == size, so all bits [start, start+size) are set
                    assert forall|i: int| start as int <= i < (start + size) as int implies
                        self.is_bit_set(i)
                    by {
                        let offset_of_i = (i - start as int) as int;
                        assert(self.is_bit_set((start as int + offset_of_i) as int));
                    };

                    // Record that all bits in the range are set
                    assert forall|i: int| start as int <= i < start as int + (size as int) implies
                        pre_assignment_bits_set_in_range.contains(i)
                    by {
                        assert(self.is_bit_set(i));
                    };

                    // All bits outside [start, start+size) remain unchanged from old(self)
                    assert forall|i: int| (0 <= i < self@.number_of_bits() &&
                        (i < start as int || i >= (start + size) as int)) implies
                        self.is_bit_set(i) == old(self).is_bit_set(i)
                    by {
                        assert(self.is_bit_set(i) == pre_alloc_self.is_bit_set(i));
                    };

                    // Establish all bits [start, start+size) were not set in old(self)
                    assert forall|i: int| start as int <= i < (start + size) as int implies
                        !old(self).is_bit_set(i)
                    by {
                        let offset_of_i = (i - start as int);
                        assert(!old_self.is_bit_set((start + offset_of_i) as int));
                    };
                }

                self.usage = self.usage + size;

                proof {
                    // After assignment to self.usage, the bits array hasn't changed
                    // Since bits haven't changed, all the bit properties remain true
                    assert forall|i: int| start as int <= i < start as int + (size as int) implies
                        self.is_bit_set(i)
                    by {
                        assert(pre_assignment_bits_set_in_range.contains(i));
                    };
                }

                return Ok(start);
            }
        }

        // For size=1: If we reach here, the loop invariant tells us all bits in [0, start) are set.
        // After loop exit: start > number_of_bits - size, so start >= number_of_bits for size=1.
        // Combined with start <= number_of_bits (invariant), we have start == number_of_bits.
        // So all bits in [0, number_of_bits) are set, meaning !has_free_bit().
        proof {
            // General case: Loop exit condition is start > number_of_bits - size.
            // Any valid starting position p for a contiguous free range of size `size`
            // must satisfy 0 <= p <= number_of_bits - size.
            // But loop invariant tells us: forall p in [0, start): !has_free_range_at(p, size).
            // Since start > number_of_bits - size, all valid starting positions have been checked.
            // Therefore, there is no contiguous free range of size `size` in old(self).
            assert forall|p: int| #![trigger self.has_free_range_at(p, size as int)]
                0 <= p <= self@.number_of_bits() - (size as int) implies !self.has_free_range_at(p, size as int)
            by {
                // p < start (since start > number_of_bits - size >= p), so covered by invariant.
                assert(p < start as int);
            }

            // Prove !exists_contiguous_free_range(size).
            // exists_contiguous_free_range(n) = exists|start| has_free_range_at(start, n).
            // has_free_range_at requires start + n <= number_of_bits, so start <= number_of_bits - n.
            // We just proved all such positions don't have a free range.
            assert(!self.exists_contiguous_free_range(size as int));

            // Since bits are unchanged from old_self (which equals old(self)), transfer the result.
            assert(self@.bits =~= old_self@.bits);
            assert(self@.number_of_bits() == old_self@.number_of_bits());
            // Also prove set_bits unchanged (follows from bits unchanged).
            assert(self@.set_bits =~= old_self@.set_bits);
            // Now prove the full view equality.
            assert(self@ == old_self@);
            assert(self@ == old(self)@);

            // Use the lemma to connect self and old_self.
            self.lemma_bits_equal_exists_free_range_equal(&old_self, size as int);
            assert(!old_self.exists_contiguous_free_range(size as int));

            // old_self == old(self), so the postcondition is satisfied.
            assert(!old(self).exists_contiguous_free_range(size as int));

            // For size=1, also prove the old postcondition for compatibility.
            if size == 1 {
                // Loop exit condition: start > number_of_bits - 1, so start >= number_of_bits.
                // Loop invariant: start <= number_of_bits.
                // Therefore: start == number_of_bits.
                assert(start as int == self.number_of_bits as int);
                // Loop invariant: forall|i| 0 <= i < start ==> is_bit_set(i).
                // So all bits [0, number_of_bits) are set.
                assert forall|i: int| 0 <= i < self@.number_of_bits() implies self@.bits[i] by {
                    assert(self.is_bit_set(i));
                }
                // This means !has_free_bit().
                assert(!self@.has_free_bit());
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
                // Forall-based frame (for backward compatibility with callers).
                &&& forall|i: int| 0 <= i < self@.number_of_bits() && i != (index as int) ==>
                    self.is_bit_set(i) == old(self).is_bit_set(i)
                // Set-based frame: set_bits is old plus the new index.
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

        proof {
            Self::lemma_bit_unset_in_seq_implies_count_lt_size(self@.bits, 0, self@.number_of_bits(), index as int);
        }

        let (word, bit): (usize, usize) = self.index(index)?;
        let ghost old_self = *self;

        self.bits.set(word, self.bits[word] | (1 << bit));

        proof {
            old_self.lemma_byte_or_reflects_in_view(self, word as int, bit as int);
            old_self.lemma_set_bit_increases_count(self, index as int);
        }

        self.usage = self.usage + 1;

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
                // Forall-based frame (for backward compatibility with callers).
                &&& forall|i: int| 0 <= i < self@.number_of_bits() && i != (index as int) ==>
                    self.is_bit_set(i) == old(self).is_bit_set(i)
                // Set-based frame: set_bits is old minus the cleared index.
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

        proof {
            Self::lemma_bit_set_in_seq_implies_count_geq_1(self@.bits, 0, self@.number_of_bits(), index as int);
        }

        let (word, bit): (usize, usize) = self.index(index)?;
        let ghost old_self = *self;

        self.bits.set(word, self.bits[word] & !(1 << bit));

        proof {
            old_self.lemma_byte_and_not_reflects_in_view(self, word as int, bit as int);
            old_self.lemma_clear_bit_decreases_count(self, index as int);
        }

        self.usage = self.usage - 1;

        Ok(())
    }

    ///
    /// # Description
    ///
    /// Clears a range of bits in the bitmap.
    ///
    /// # Parameters
    ///
    /// - `start`: Starting index of the range to clear.
    /// - `size`: Size of the range to clear.
    ///
    /// # Returns
    ///
    /// Upon success, `Ok(())` is returned. Upon failure, an error is returned instead.
    ///
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
                // Forall-based frame (for backward compatibility with callers).
                &&& forall|i: int| 0 <= i < self@.number_of_bits() &&
                    (i < start as int || i >= start as int + (size as int)) ==>
                    self.is_bit_set(i) == old(self).is_bit_set(i)
                // Set-based frame: set_bits is old minus the cleared range.
                &&& self@.set_bits =~= old(self)@.set_bits.difference(BitmapView::range_set(start as int, start as int + (size as int)))
                &&& self@.usage() == old(self)@.usage() - (size as int)
            },
            result is Err ==> self@ == old(self)@,
            // Liveness: clearing always succeeds when preconditions are met.
            result is Ok,
    {
        let ghost old_self = *self;

        // Check if the range is within bounds.
        if start > self.number_of_bits - size {
            proof {
                // This should be unreachable due to preconditions.
                assert(false);
            }
            let reason: &str = "range out of bounds";
            return Err(Error::new(ErrorCode::InvalidArgument, reason));
        }

        // Clear the range one bit at a time using the verified clear function.
        let ghost pre_clear_self = *self;
        let mut offset: usize = 0;

        while offset < size
            invariant
                self.inv(),
                old_self.inv(),
                pre_clear_self.inv(),
                old_self == old(self),
                pre_clear_self == old(self),
                self.number_of_bits == pre_clear_self.number_of_bits,
                self@.number_of_bits() == old(self)@.number_of_bits(),
                0 < size <= self.number_of_bits,
                start <= self.number_of_bits - size,
                offset <= size,
                // Bits in [start, start+offset) are cleared.
                forall|i: int| start as int <= i < (start + offset) as int ==>
                    !#[trigger] self.is_bit_set(i),
                // Bits in [start+offset, start+size) are still set.
                forall|i: int| (start + offset) as int <= i < (start + size) as int ==>
                    #[trigger] self.is_bit_set(i),
                // Bits outside [start, start+size) are unchanged from old_self.
                forall|i: int| (0 <= i < self@.number_of_bits() &&
                    (i < start as int || i >= (start + size) as int)) ==>
                    #[trigger] self.is_bit_set(i) == #[trigger] old_self.is_bit_set(i),
                // Usage tracking: decreased by offset so far.
                self@.usage() == old_self@.usage() - offset,
                // Set-based invariant: set_bits == old minus range [start, start+offset).
                self@.set_bits =~= old_self@.set_bits.difference(BitmapView::range_set(start as int, start as int + (offset as int))),
        {
            let idx: usize = start + offset;
            let ghost loop_old_self = *self;

            proof {
                // The bit at idx is still set (from invariant: bits in [start+offset, start+size) are set).
                assert(self.is_bit_set(idx as int));
            }

            // Use the verified clear function.
            let clear_result: Result<(), Error> = self.clear(idx);

            proof {
                // clear succeeds because the bit is set and in bounds.
                match clear_result {
                    Ok(_) => {},
                    Err(_) => { assert(false); }
                }

                // After clearing, update our knowledge.
                // Bits in [start, start+offset+1) are now cleared.
                assert(!self.is_bit_set(idx as int));

                // Bits in [start+offset+1, start+size) are still set.
                assert forall|i: int| (start + offset + 1) as int <= i < (start + size) as int
                    implies #[trigger] self.is_bit_set(i)
                by {
                    // i != idx, so unchanged by clear.
                };

                // Bits outside [start, start+size) are unchanged.
                assert forall|i: int| (0 <= i < self@.number_of_bits() &&
                    (i < start as int || i >= (start + size) as int))
                    implies #[trigger] self.is_bit_set(i) == #[trigger] old_self.is_bit_set(i)
                by {
                    // i != idx, so unchanged by clear.
                };

                // Bits in [start, start+offset+1) are cleared.
                assert forall|i: int| start as int <= i < (start + offset + 1) as int
                    implies !#[trigger] self.is_bit_set(i)
                by {
                    if i < (start + offset) as int {
                        // From loop invariant.
                    } else {
                        // i == start + offset == idx, which we just cleared.
                        assert(i == idx as int);
                    }
                };

                // Prove set_bits invariant update.
                // We have: self@.set_bits =~= loop_old_self@.set_bits.remove(idx)
                // And: loop_old_self@.set_bits =~= old_self@.set_bits.difference(range_set(start, start+offset))
                // Need: self@.set_bits =~= old_self@.set_bits.difference(range_set(start, start+offset+1))
                assert forall|i: int| self@.set_bits.contains(i) ==
                    old_self@.set_bits.difference(BitmapView::range_set(start as int, start as int + (offset as int + 1))).contains(i)
                by {
                    // Case analysis
                    if i == idx as int {
                        // i is the newly removed bit
                    } else if start as int <= i < start as int + (offset as int) {
                        // i is in the range we already cleared
                    } else if 0 <= i < self@.number_of_bits() {
                        // i is outside the clearing range
                    }
                }
            }

            offset = offset + 1;
        }

        proof {
            // At loop exit: offset == size, so all bits [start, start+size) are cleared.
            assert forall|i: int| start as int <= i < (start + size) as int implies
                !self.is_bit_set(i)
            by {
                // From invariant with offset == size.
            };
        }

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
    pub fn test(&self, index: usize) -> (result: Result<bool, Error>)
        requires
            self.inv(),
        ensures
            result is Ok ==> {
                &&& (index as int) < self@.number_of_bits()
                &&& result->Ok_0 == self.is_bit_set(index as int)
            },
            // Error case: out of bounds index produces error.
            result is Err ==> (index as int) >= self@.number_of_bits(),
            // Liveness: valid index always succeeds.
            (index as int) < self@.number_of_bits() ==> result is Ok,
            // test is read-only - no state changes
    {
        let (word, bit): (usize, usize) = self.index(index)?;
        let byte_val = self.bits[word];
        let result_val = (byte_val & (1 << bit)) != 0;

        Ok(result_val)
    }

    //==================================================================================================
    // Private Helper Methods
    //==================================================================================================

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
    fn index(&self, index: usize) -> (result: Result<(usize, usize), Error>)
        requires
            self.inv(),
        ensures
            result is Ok ==> {
                let (word, bit) = result->Ok_0;
                &&& word < self.bits@.len()
                &&& bit < u8::BITS as usize
                &&& word == index / (u8::BITS as usize)
                &&& bit == index % (u8::BITS as usize)
                &&& index < self.bits@.len() * (u8::BITS as usize)
            },
            // Error case: out of bounds index produces error.
            result is Err ==> (index as int) >= self@.number_of_bits(),
            // Liveness: valid index always succeeds.
            (index as int) < self@.number_of_bits() ==> result is Ok,
    {
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
    fn index_unchecked(&self, index: usize) -> (result: (usize, usize))
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
    {
        proof {
            // index < number_of_bits = bits.len() * 8
            // We need: index / 8 < bits.len()
            // This follows from: if a < b * c and c > 0, then a / c < b
            let idx = index as int;
            let len = self.bits@.len() as int;
            let bits = u8::BITS as int;
            assert(idx < len * bits);
            assert(bits > 0);
            // Use nonlinear arithmetic
            assert(idx / bits < len) by (nonlinear_arith)
                requires idx < len * bits, bits > 0, len > 0
            {}
        }
        let word: usize = index / u8::BITS as usize;
        let bit: usize = index % u8::BITS as usize;
        (word, bit)
    }
}

//==================================================================================================
// Verified Tests
//==================================================================================================

/// Verifiable test: creating a new bitmap should set number_of_bits correctly.
fn test_bitmap_new_verified(number_of_bits: usize)
    requires
        number_of_bits > 0,
        number_of_bits < u32::MAX as usize,
        number_of_bits % (u8::BITS as usize) == 0,
{
    let result = Bitmap::new(number_of_bits);
    if let Ok(bitmap) = result {
        assert(bitmap@.number_of_bits() == number_of_bits as int);
    }
}

/// Verifiable test: allocating a bit should return a valid index and set the bit.
fn test_bitmap_alloc_verified(number_of_bits: usize)
    requires
        number_of_bits > 0,
        number_of_bits < u32::MAX as usize,
        number_of_bits % (u8::BITS as usize) == 0,
{
    let result = Bitmap::new(number_of_bits);
    if let Ok(mut bitmap) = result {
        let alloc_result = bitmap.alloc();
        if let Ok(index) = alloc_result {
            // The allocated index should be within bounds.
            assert(index < number_of_bits);
            // The bit at the allocated index should be set.
            assert(bitmap.is_bit_set(index as int));
        }
    }
}

/// Verifiable test: setting and clearing a bit should work correctly.
fn test_bitmap_set_clear_verified(number_of_bits: usize, index: usize)
    requires
        number_of_bits > 0,
        number_of_bits < u32::MAX as usize,
        number_of_bits % (u8::BITS as usize) == 0,
        index < number_of_bits,
{
    let result = Bitmap::new(number_of_bits);
    if let Ok(mut bitmap) = result {
        // Set the bit.
        let set_result = bitmap.set(index);
        if let Ok(()) = set_result {
            // The bit should be set.
            assert(bitmap.is_bit_set(index as int));

            // Clear the bit.
            let clear_result = bitmap.clear(index);
            if let Ok(()) = clear_result {
                // The bit should be cleared.
                assert(!bitmap.is_bit_set(index as int));
            }
        }
    }
}

/// Verifiable test: allocating a range should allocate contiguous bits.
fn test_bitmap_alloc_range_verified(number_of_bits: usize, size: usize)
    requires
        number_of_bits > 0,
        number_of_bits < u32::MAX as usize,
        number_of_bits % (u8::BITS as usize) == 0,
        size > 0,
        size <= number_of_bits,
{
    let result = Bitmap::new(number_of_bits);
    if let Ok(mut bitmap) = result {
        let alloc_result = bitmap.alloc_range(size);
        if let Ok(start_index) = alloc_result {
            // The start index should be within valid range.
            assert(start_index + size <= number_of_bits);

            // All bits in the range should be set.
            assert(bitmap.all_bits_set_in_range(start_index as int, (start_index + size) as int));
        }
    }
}

/// Verifiable test: multiple allocations should not overlap.
fn test_bitmap_multiple_alloc_verified(number_of_bits: usize)
    requires
        number_of_bits >= 16,  // Need at least 2 bits.
        number_of_bits < u32::MAX as usize,
        number_of_bits % (u8::BITS as usize) == 0,
{
    let result = Bitmap::new(number_of_bits);
    if let Ok(mut bitmap) = result {
        let alloc1 = bitmap.alloc();
        if let Ok(index1) = alloc1 {
            let alloc2 = bitmap.alloc();
            if let Ok(index2) = alloc2 {
                // The two allocated indices should be different.
                assert(index1 != index2);
                // Both bits should be set.
                assert(bitmap.is_bit_set(index1 as int));
                assert(bitmap.is_bit_set(index2 as int));
            }
        }
    }
}

/// Verifiable test: clearing and re-allocating should work.
fn test_bitmap_clear_and_realloc_verified(number_of_bits: usize, index: usize)
    requires
        number_of_bits > 0,
        number_of_bits < u32::MAX as usize,
        number_of_bits % (u8::BITS as usize) == 0,
        index < number_of_bits,
{
    let result = Bitmap::new(number_of_bits);
    if let Ok(mut bitmap) = result {
        // Set a bit.
        let set_result = bitmap.set(index);
        if let Ok(()) = set_result {
            // Clear the bit.
            let clear_result = bitmap.clear(index);
            if let Ok(()) = clear_result {
                // The bit should be cleared.
                assert(!bitmap.is_bit_set(index as int));

                // Usage should be back to 0.
                assert(bitmap@.usage() == 0);
            }
        }
    }
}

/// Verifiable test: usage tracking is correct.
fn test_bitmap_usage_tracking_verified(number_of_bits: usize)
    requires
        number_of_bits >= 24,  // Need at least 3 bits.
        number_of_bits < u32::MAX as usize,
        number_of_bits % (u8::BITS as usize) == 0,
{
    let result = Bitmap::new(number_of_bits);
    if let Ok(mut bitmap) = result {
        // Initially empty.
        assert(bitmap@.is_empty());
        assert(bitmap@.usage() == 0);

        // Allocate first bit.
        let alloc1 = bitmap.alloc();
        if let Ok(_) = alloc1 {
            assert(bitmap@.usage() == 1);

            // Allocate second bit.
            let alloc2 = bitmap.alloc();
            if let Ok(_) = alloc2 {
                assert(bitmap@.usage() == 2);

                // Allocate third bit.
                let alloc3 = bitmap.alloc();
                if let Ok(index3) = alloc3 {
                    assert(bitmap@.usage() == 3);

                    // Clear one bit.
                    let clear_result = bitmap.clear(index3);
                    if let Ok(()) = clear_result {
                        assert(bitmap@.usage() == 2);
                    }
                }
            }
        }
    }
}

/// Verifiable test: alloc_range preserves bits outside the allocated range.
fn test_bitmap_alloc_range_preserves_others_verified(number_of_bits: usize, size: usize, test_index: usize)
    requires
        number_of_bits >= 16,
        number_of_bits < u32::MAX as usize,
        number_of_bits % (u8::BITS as usize) == 0,
        size > 0,
        size < number_of_bits,
        test_index < number_of_bits,
{
    let result = Bitmap::new(number_of_bits);
    if let Ok(mut bitmap) = result {
        // Set a bit first.
        let set_result = bitmap.set(test_index);
        if let Ok(()) = set_result {
            // Allocate a range.
            let alloc_result = bitmap.alloc_range(size);
            if let Ok(start_index) = alloc_result {
                // If test_index is outside the allocated range, it should still be set.
                if test_index < start_index || test_index >= start_index + size {
                    assert(bitmap.is_bit_set(test_index as int));
                }
            }
        }
    }
}

/// Verifiable test: number_of_bits remains constant across operations.
fn test_bitmap_number_of_bits_constant_verified(number_of_bits: usize, index: usize)
    requires
        number_of_bits > 0,
        number_of_bits < u32::MAX as usize,
        number_of_bits % (u8::BITS as usize) == 0,
        index < number_of_bits,
{
    let result = Bitmap::new(number_of_bits);
    if let Ok(mut bitmap) = result {
        let ghost initial_bits = bitmap@.number_of_bits();
        assert(initial_bits == number_of_bits as int);

        // After allocation.
        let alloc_result = bitmap.alloc();
        if let Ok(_) = alloc_result {
            assert(bitmap@.number_of_bits() == initial_bits);

            // After setting a bit.
            let set_result = bitmap.set(index);
            match set_result {
                Ok(()) => {
                    assert(bitmap@.number_of_bits() == initial_bits);
                },
                Err(_) => {
                    // If set failed (bit already set), number_of_bits should still be the same.
                    assert(bitmap@.number_of_bits() == initial_bits);
                }
            }
        }
    }
}

/// Verifiable test: setting an already-set bit should fail.
fn test_bitmap_double_set_fails_verified(number_of_bits: usize, index: usize)
    requires
        number_of_bits > 0,
        number_of_bits < u32::MAX as usize,
        number_of_bits % (u8::BITS as usize) == 0,
        index < number_of_bits,
{
    let result = Bitmap::new(number_of_bits);
    if let Ok(mut bitmap) = result {
        // Set the bit.
        let set_result1 = bitmap.set(index);
        if let Ok(()) = set_result1 {
            // Try to set the same bit again.
            let set_result2 = bitmap.set(index);
            // This should fail because the bit is already set.
            assert(set_result2 is Err);
            // The bit should still be set.
            assert(bitmap.is_bit_set(index as int));
        }
    }
}

/// Verifiable test: clearing an already-clear bit should fail.
fn test_bitmap_double_clear_fails_verified(number_of_bits: usize, index: usize)
    requires
        number_of_bits > 0,
        number_of_bits < u32::MAX as usize,
        number_of_bits % (u8::BITS as usize) == 0,
        index < number_of_bits,
{
    let result = Bitmap::new(number_of_bits);
    if let Ok(mut bitmap) = result {
        // The bit is initially clear, try to clear it.
        let clear_result = bitmap.clear(index);
        // This should fail because the bit is already clear.
        assert(clear_result is Err);
        // The bit should still be clear.
        assert(!bitmap.is_bit_set(index as int));
    }
}

/// Verifiable test: setting all bits and then clearing all bits.
fn test_set_and_clear_all_bits_verified(number_of_bits: usize)
    requires
        number_of_bits > 0,
        number_of_bits < u32::MAX as usize,
        number_of_bits % (u8::BITS as usize) == 0,
{
    let result = Bitmap::new(number_of_bits);
    if let Ok(mut bitmap) = result {
        // Set all bits.
        let mut i: usize = 0;
        while i < number_of_bits
            invariant
                0 <= i <= number_of_bits,
                bitmap.inv(),
                bitmap@.number_of_bits() == number_of_bits as int,
                forall|j: int| 0 <= j < i ==> bitmap.is_bit_set(j),
            decreases number_of_bits - i,
        {
            let set_result = bitmap.set(i);
            if let Ok(()) = set_result {
                i = i + 1;
            } else {
                break;
            }
        }

        // If we set all bits successfully.
        if i == number_of_bits {
            // All bits should be set.
            assert(bitmap.all_bits_set_in_range(0, number_of_bits as int));

            // Clear all bits.
            let mut j: usize = 0;
            while j < number_of_bits
                invariant
                    0 <= j <= number_of_bits,
                    bitmap.inv(),
                    bitmap@.number_of_bits() == number_of_bits as int,
                    forall|k: int| 0 <= k < j ==> !bitmap.is_bit_set(k),
                decreases number_of_bits - j,
            {
                let clear_result = bitmap.clear(j);
                if let Ok(()) = clear_result {
                    j = j + 1;
                } else {
                    break;
                }
            }

            // If we cleared all bits successfully.
            if j == number_of_bits {
                // All bits should be cleared.
                assert(bitmap.all_bits_unset_in_range(0, number_of_bits as int));
            }
        }
    }
}

/// Verifiable test: allocating all bits and then clearing all bits.
fn test_alloc_and_clear_all_bits_verified(number_of_bits: usize)
    requires
        number_of_bits > 0,
        number_of_bits < u32::MAX as usize,
        number_of_bits % (u8::BITS as usize) == 0,
{
    let result = Bitmap::new(number_of_bits);
    if let Ok(mut bitmap) = result {
        // Allocate all bits.
        let mut count: usize = 0;
        while count < number_of_bits
            invariant
                0 <= count <= number_of_bits,
                bitmap.inv(),
                bitmap@.number_of_bits() == number_of_bits as int,
                bitmap@.usage() == count as int,
            decreases number_of_bits - count,
        {
            let alloc_result = bitmap.alloc();
            if let Ok(_) = alloc_result {
                count = count + 1;
            } else {
                break;
            }
        }

        // If we allocated all bits successfully.
        if count == number_of_bits {
            // Usage should equal number_of_bits, so bitmap is full.
            assert(bitmap@.usage() == number_of_bits as int);
            assert(bitmap@.is_full());

            // By lemma, all bits should be set.
            proof {
                bitmap.lemma_is_full_means_all_bits_set();
            }
            assert(bitmap.all_bits_set_in_range(0, number_of_bits as int));

            // Clear all bits.
            let mut j: usize = 0;
            while j < number_of_bits
                invariant
                    0 <= j <= number_of_bits,
                    bitmap.inv(),
                    bitmap@.number_of_bits() == number_of_bits as int,
                    forall|k: int| 0 <= k < j ==> !bitmap.is_bit_set(k),
                decreases number_of_bits - j,
            {
                let clear_result = bitmap.clear(j);
                if let Ok(()) = clear_result {
                    j = j + 1;
                } else {
                    break;
                }
            }

            // If we cleared all bits successfully.
            if j == number_of_bits {
                // All bits should be cleared.
                assert(bitmap.all_bits_unset_in_range(0, number_of_bits as int));
            }
        }
    }
}

/// Verifiable test: allocating a range larger than the bitmap should fail.
fn test_alloc_range_too_large_verified(number_of_bits: usize)
    requires
        number_of_bits > 0,
        number_of_bits < u32::MAX as usize,
        number_of_bits % (u8::BITS as usize) == 0,
{
    let result = Bitmap::new(number_of_bits);
    if let Ok(mut bitmap) = result {
        let size = number_of_bits + 1;
        let alloc_result = bitmap.alloc_range(size);
        // Allocating more than number_of_bits should fail.
        assert(alloc_result is Err);
    }
}

/// Verifiable test: allocating a range of size 0 should fail.
fn test_alloc_range_zero_verified(number_of_bits: usize)
    requires
        number_of_bits > 0,
        number_of_bits < u32::MAX as usize,
        number_of_bits % (u8::BITS as usize) == 0,
{
    let result = Bitmap::new(number_of_bits);
    if let Ok(mut bitmap) = result {
        let alloc_result = bitmap.alloc_range(0);
        // Allocating 0 bits should fail (size must be > 0).
        assert(alloc_result is Err);
    }
}

/// Verifiable test: allocating a range, verifying it's allocated, then clearing it.
fn test_alloc_range_and_clear_verified(number_of_bits: usize, size: usize)
    requires
        number_of_bits > 0,
        number_of_bits < u32::MAX as usize,
        number_of_bits % (u8::BITS as usize) == 0,
        size > 0,
        size <= number_of_bits,
{
    let result = Bitmap::new(number_of_bits);
    if let Ok(mut bitmap) = result {
        let alloc_result = bitmap.alloc_range(size);
        if let Ok(start) = alloc_result {
            // Verify the range is allocated.
            assert(bitmap.all_bits_set_in_range(start as int, (start + size) as int));

            // Clear the range.
            let mut i: usize = start;
            let end = start + size;
            while i < end
                invariant
                    start <= i <= end,
                    end == start + size,
                    bitmap.inv(),
                    bitmap@.number_of_bits() == number_of_bits as int,
                    forall|j: int| start <= j < i ==> !bitmap.is_bit_set(j),
                decreases end - i,
            {
                let clear_result = bitmap.clear(i);
                if let Ok(()) = clear_result {
                    i = i + 1;
                } else {
                    break;
                }
            }

            // If we cleared all bits successfully.
            if i == end {
                // All bits in the range should be cleared.
                assert(bitmap.all_bits_unset_in_range(start as int, end as int));
            }
        }
    }
}

/// Verifiable test: allocating a bit in a partially filled bitmap.
fn test_alloc_in_partial_bitmap_verified(number_of_bits: usize, set_index: usize)
    requires
        number_of_bits >= 16,  // Need at least 2 bits.
        number_of_bits < u32::MAX as usize,
        number_of_bits % (u8::BITS as usize) == 0,
        set_index < number_of_bits,
{
    let result = Bitmap::new(number_of_bits);
    if let Ok(mut bitmap) = result {
        // Set one bit to partially fill the bitmap.
        let set_result = bitmap.set(set_index);
        if let Ok(()) = set_result {
            // The set bit should be marked as set.
            assert(bitmap.is_bit_set(set_index as int));

            // Allocate a new bit.
            let alloc_result = bitmap.alloc();
            if let Ok(index) = alloc_result {
                // The allocated bit should be set.
                assert(bitmap.is_bit_set(index as int));

                // Clear the allocated bit.
                let clear_result = bitmap.clear(index);
                if let Ok(()) = clear_result {
                    assert(!bitmap.is_bit_set(index as int));
                    // The originally set bit should still be set (if it wasn't the one we allocated).
                    if index != set_index {
                        assert(bitmap.is_bit_set(set_index as int));
                    }
                }
            }
        }
    }
}

//==================================================================================================

} // verus!
