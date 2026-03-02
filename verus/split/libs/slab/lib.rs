// Copyright(c) The Maintainers of Nanvix.
// Licensed under the MIT License.

//==================================================================================================
// Imports
//==================================================================================================

use crate::libs::{
    bitmap::Bitmap,
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
use vstd::{
    prelude::*,
    set::*,
    set_lib::{
        lemma_int_range,
        lemma_len_subset,
        lemma_set_subset_finite,
        set_int_range,
    },
};

// Include specifications.
include!("lib.spec.rs");

// Include proofs.
include!("lib.proof.rs");

// Include verified tests.
include!("lib.test.rs");

//==================================================================================================
// Structures
//==================================================================================================

verus! {

/// Wrapper for `usize` to `*mut u8` cast (Verus cannot cast integers to pointers).
#[inline]
#[verifier::external_body]
pub fn usize_to_ptr(addr: usize) -> (result: *mut u8)
    ensures result as int == addr as int,
{
    addr as *mut u8
}

/// Wrapper for unsafe `ptr.add(count)` with verified postcondition.
#[inline]
#[verifier::external_body]
pub fn ptr_add(ptr: *mut u8, count: usize) -> (result: *mut u8)
    ensures result as int == ptr as int + count as int,
{
    unsafe { ptr.add(count) }
}

/// Wrapper for unsafe `ptr.offset_from_unsigned(origin)` with verified postcondition.
#[inline]
#[verifier::external_body]
pub fn ptr_offset_from(ptr: *const u8, origin: *const u8) -> (result: usize)
    requires ptr as int >= origin as int,
    ensures result as int == ptr as int - origin as int,
{
    unsafe { ptr.offset_from_unsigned(origin) }
}


///
/// # Description
///
/// A slab allocator.
///
/// It has the following layout in memory:
///
/// ```text
/// +-------------------+--------------------------------------+
/// | Index Blocks      | Data Blocks                          |
/// +-------------------+--------------------------------------+
/// ```
///
pub struct Slab {
    /// An index that keeps track of free blocks.
    index: Bitmap,
    /// Base address of data blocks.
    data_addr: *mut u8,
    /// Number of index blocks in the slab.
    num_index_blocks: usize,
    /// Number of data blocks in the slab.
    num_data_blocks: usize,
    /// Size of blocks in the slab.
    block_size: usize,
    /// Base address of the entire slab buffer (including index region).
    /// Not in the original source; added for verification invariants.
    base_addr: usize,
    /// Total length of the slab buffer in bytes.
    /// Not in the original source; added for verification invariants.
    total_len: usize,
}


//==================================================================================================
// Implementations
//==================================================================================================

impl Slab {

    /// Executable function: checks if a usize is a power of two.
    /// Uses iterative division to match the recursive spec definition.
    pub fn is_power_of_two(n: usize) -> (result: bool)
        requires n > 0,
        ensures result == Self::spec_is_power_of_two(n as int),
    {
        let mut val: usize = n;

        // Loop invariant: val > 0 and the result depends on whether val becomes 1.
        // We divide by 2 as long as val is even and > 1.
        while val > 1 && val % 2 == 0
            invariant
                val > 0,
                Self::spec_is_power_of_two(n as int) == Self::spec_is_power_of_two(val as int),
            decreases val,
        {
            val = val / 2;
        }

        // At this point: either val == 1 (power of two) or val > 1 && val % 2 != 0 (not power of two).
        val == 1
    }

    //==============================================================================================

    ///
    /// # Description
    ///
    /// Creates a new slab allocator on the memory region starting at `addr` with `len` bytes and
    /// block size of `block_size` bytes. The slab allocator is initialized with all blocks free.
    ///
    /// # Parameters
    ///
    /// - `addr`: Start address of the memory region.
    /// - `len`: Length of the memory region in bytes.
    /// - `block_size`: Size of blocks in bytes.
    ///
    /// # Returns
    ///
    /// Upon success, a new slab allocator is returned. Upon failure, an error is returned instead
    /// and the memory may be left in a modified state.
    ///
    /// # Safety
    ///
    /// This function is unsafe for the following reasons:
    /// - It assumes that the memory region starting at `addr` with `len` bytes is valid.
    ///
    /// # Verus Equivalence
    ///
    /// Compared to the original `src/libs/slab/src/lib.rs`:
    /// - Parameter `addr: *mut u8` → `addr: usize` (Verus: no raw pointers).
    /// - Wrapping check replaced by precondition on address space bounds.
    /// - Bitwise power-of-two check → `is_power_of_two()` verified helper.
    /// - `RawArray::from_raw_parts` → `raw_array_from_addr` (Verus cannot cast `usize` to `*mut T`).
    /// - `for` loop → `while` loop (Verus: no `for` loops).
    /// - Pointer arithmetic → integer arithmetic.
    /// - Extra fields `base_addr`, `total_len` stored for invariant proofs.
    ///
    pub unsafe fn from_raw_parts(
        addr: *mut u8,
        len: usize,
        block_size: usize,
    ) -> (result: Result<Slab, Error>)
        requires
            // Length must be valid and non-zero.
            len > 0,
            len < i32::MAX as usize,
            // Block size must be valid.
            block_size > 0,
            block_size < i32::MAX as usize,
            block_size <= len,
            // Block size must be a power of two.
            Self::spec_is_power_of_two(block_size as int),
            // Start address must be aligned to block size.
            (addr as usize) % block_size == 0,
            (addr as usize) > 0,
            // Memory region must not wrap around and fit in address space.
            (addr as int) + (len as int) <= (usize::MAX as int),
            // Total number of blocks must be a multiple of 8.
            (len / block_size) % (u8::BITS as usize) == 0,
            // Ensure we have enough blocks for a valid slab (at least 8).
            len / block_size >= 8,
        ensures
            // If result is Ok, these properties hold.
            result is Ok ==> {
                let slab = result->Ok_0;
                &&& slab.inv()
                &&& slab@.block_size == block_size as int
                // Freshly initialized: no blocks allocated (Set-based, no forall).
                &&& slab@.allocated_blocks =~= Set::<int>::empty()
                // The data address is at an offset from addr.
                &&& slab@.data_addr > addr as int
                &&& slab@.data_addr % (block_size as int) == 0
                // Number of data blocks is positive.
                &&& slab@.num_data_blocks > 0
                // Buffer bounds are recorded.
                &&& slab@.base_addr == addr as int
                &&& slab@.total_len == len as int
                // Data region fits within the buffer.
                &&& slab@.data_addr + slab@.num_data_blocks * slab@.block_size
                    <= addr as int + len as int
            },
    {
        // Check if length is invalid.
        if len == 0 || len >= i32::MAX as usize {
            return Err(Error::new(ErrorCode::InvalidArgument, "invalid slab length"));
        }

        // TODO: remove this runtime check once all callers are verified.
        // Check if the memory region wraps around.
        if (addr as usize).wrapping_add(len) < (addr as usize) {
            return Err(Error::new(ErrorCode::InvalidArgument, "wrapping memory region"));
        }

        // Check if block size is valid.
        if block_size == 0 || block_size >= i32::MAX as usize || block_size > len {
            return Err(Error::new(ErrorCode::InvalidArgument, "invalid block size"));
        }

        // Check if the `block_size` is a power of two.
        if !Self::is_power_of_two(block_size) {
            return Err(Error::new(ErrorCode::InvalidArgument, "block size is not a power of two"));
        }

        // At this point, is_power_of_two returned true, so spec_is_power_of_two holds.
        assert(Self::spec_is_power_of_two(block_size as int));

        // Check if `addr` is aligned to `block_size`.
        if !(addr as usize).is_multiple_of(block_size) {
            return Err(Error::new(ErrorCode::InvalidArgument, "unaligned start address"));
        }

        // Compute layout of the slab allocator.
        let total_num_blocks: usize = len / block_size;
        // info!("total number of blocks: {:?}", total_num_blocks);
        if !total_num_blocks.is_multiple_of(u8::BITS as usize) {
            return Err(Error::new(ErrorCode::InvalidArgument, "invalid number of blocks"));
        }

        let index_len: usize = total_num_blocks / u8::BITS as usize;
        // info!("index length: {:?}", index_len);
        let num_index_blocks: usize = (index_len / block_size)
            + if index_len.is_multiple_of(block_size) { 0 } else { 1 };
        // info!("number of index blocks: {:?}", num_index_blocks);
        if num_index_blocks > total_num_blocks {
            return Err(Error::new(ErrorCode::InvalidArgument, "insufficient blocks for index"));
        }
        let num_data_blocks: usize = total_num_blocks - num_index_blocks;
        // info!("number of data blocks: {:?}", num_data_blocks);

        // Source uses `addr.add(...)` (pointer arithmetic); Verus uses integer arithmetic.
        // Prove num_index_blocks * block_size fits in usize.
        proof {
            // total_num_blocks >= 8 (from precondition), so index_len >= 1.
            assert(total_num_blocks >= 8usize);
            assert(index_len >= 1usize);
            // Prove num_index_blocks >= 1: either index_len / block_size >= 1,
            // or index_len < block_size so index_len % block_size > 0, adding 1.
            if index_len >= block_size {
                assert((index_len as int) / (block_size as int) >= 1) by(nonlinear_arith)
                    requires index_len >= block_size, block_size > 0int;
            } else {
                // index_len < block_size, so index_len / block_size == 0.
                // But index_len >= 1, so index_len % block_size == index_len > 0.
                assert((index_len as int) % (block_size as int) == (index_len as int)) by(nonlinear_arith)
                    requires 0 < index_len < block_size;
                assert(index_len % block_size != 0usize);
            }
            assert(num_index_blocks >= 1usize);
            // num_index_blocks <= total_num_blocks (from check above).
            // Since total_num_blocks >= 8 and num_index_blocks <= total_num_blocks / 8 + 1
            // (at most), we need to show num_index_blocks < total_num_blocks.
            // Actually: index_len = total_num_blocks / 8.
            // num_index_blocks = ceil(index_len / block_size) <= index_len (since block_size >= 1).
            // But index_len = total_num_blocks / 8, so num_index_blocks <= total_num_blocks / 8.
            // total_num_blocks / 8 < total_num_blocks (since total_num_blocks >= 8).
            // So num_index_blocks < total_num_blocks, hence num_data_blocks >= 1.
            assert(num_index_blocks <= total_num_blocks);
            // Prove num_data_blocks > 0: we need num_index_blocks < total_num_blocks.
            // index_len = total_num_blocks / 8 <= total_num_blocks / 8.
            // num_index_blocks <= (index_len / block_size) + 1.
            // block_size >= 1, so (index_len / block_size) <= index_len.
            // num_index_blocks <= index_len + 1 = total_num_blocks / 8 + 1.
            // For total_num_blocks >= 8: total_num_blocks / 8 + 1 <= total_num_blocks
            //   iff total_num_blocks / 8 <= total_num_blocks - 1
            //   iff total_num_blocks <= 8 * (total_num_blocks - 1) = 8*total_num_blocks - 8
            //   iff 8 <= 7 * total_num_blocks
            //   iff total_num_blocks >= 2 (true since >= 8).
            assert(num_index_blocks as int <= (index_len as int) + 1) by {
                assert((index_len as int) / (block_size as int) <= (index_len as int)) by(nonlinear_arith)
                    requires block_size >= 1int, index_len >= 0int;
            }
            assert((index_len as int) + 1 <= (total_num_blocks as int)) by {
                assert((total_num_blocks as int) / 8 + 1 <= (total_num_blocks as int)) by(nonlinear_arith)
                    requires total_num_blocks >= 8int;
            }
            assert(num_index_blocks < total_num_blocks);
            assert(num_data_blocks > 0usize);

            // Prove num_index_blocks * block_size <= len.
            Self::lemma_div_mul_le(len as int, block_size as int);
            Self::lemma_mul_inequality(num_index_blocks as int, total_num_blocks as int, block_size as int);
            // addr + num_index_blocks * block_size < addr + len <= usize::MAX.
            assert((num_index_blocks as int) * (block_size as int) < (total_num_blocks as int) * (block_size as int));
            assert((total_num_blocks as int) * (block_size as int) <= len as int);
            assert((addr as int) + (num_index_blocks as int) * (block_size as int) <= (addr as int) + (len as int));
            assert((addr as int) + (len as int) <= usize::MAX as int);
        }
        let data_addr: *mut u8 = ptr_add(addr, num_index_blocks * block_size);

        // Check if `data_addr` is aligned to `block_size`.
        if !(data_addr as usize).is_multiple_of(block_size) {
            return Err(Error::new(ErrorCode::InvalidArgument, "unaligned data address"));
        }

        // Instantiate index.
        let storage: RawArray<u8> = RawArray::from_raw_parts(addr, index_len)?;

        // Prove that all bytes in storage are zero (required by Bitmap::from_raw_array).
        proof {
            // raw_array_from_addr ensures is_zero for each element.
            // axiom_u8_zero_is_0 converts is_zero(t) to t == 0.
            assert forall|i: int| 0 <= i < storage@.len() implies storage@[i] == 0u8 by {
                axiom_u8_zero_is_0(storage@[i]);
            }
        }

        let mut index: Bitmap = Bitmap::from_raw_array(storage)?;

        // Prove key invariants before the loop.
        proof {
            // index@.number_of_bits() == index_len * 8 == total_num_blocks (since total_num_blocks % 8 == 0)
            assert(index.inv());
            assert(index@.number_of_bits() == index_len as int * 8);
            assert(total_num_blocks == index_len * 8);
            assert(index@.number_of_bits() == total_num_blocks as int);
            // num_index_blocks + num_data_blocks == total_num_blocks
            assert(num_index_blocks + num_data_blocks == total_num_blocks);
            // Therefore: num_index_blocks + num_data_blocks == index@.number_of_bits()
            assert(num_index_blocks as int + num_data_blocks as int == index@.number_of_bits());
            // num_index_blocks < total_num_blocks (from earlier check), so:
            assert(num_index_blocks < total_num_blocks);
        }

        // Initialize index.
        let mut i: usize = 0;
        while i < num_index_blocks
            invariant
                index.inv(),
                i <= num_index_blocks,
                num_index_blocks < total_num_blocks,
                num_index_blocks > 0,
                num_data_blocks > 0,
                block_size > 0,
                data_addr as int > 0,
                num_index_blocks + num_data_blocks == total_num_blocks,
                index@.number_of_bits() == total_num_blocks as int,
                num_index_blocks as int + num_data_blocks as int == index@.number_of_bits(),
                // All bits from 0 to i are set (using set_bits directly).
                forall|j: int| #![trigger index@.set_bits.contains(j)]
                    0 <= j < i as int ==> index@.set_bits.contains(j),
                // All bits from i to end are not set (from initial state).
                forall|j: int| #![trigger index@.set_bits.contains(j)]
                    i as int <= j < index@.number_of_bits() ==> !index@.set_bits.contains(j),
            decreases num_index_blocks - i,
        {
            index.set(i)?;
            i = i + 1;
        }

        // After the loop, all index blocks are set.
        // Now prove the postconditions.
        let result_slab = Slab {
            index,
            data_addr,
            num_index_blocks,
            num_data_blocks,
            block_size,
            base_addr: addr as usize,
            total_len: len,
        };

        proof {
            // Prove memory bounds conditions for the new invariant.
            // total_num_blocks = len / block_size.
            // num_data_blocks = total_num_blocks - num_index_blocks < total_num_blocks.
            // num_data_blocks * block_size < total_num_blocks * block_size = len.
            // Since len < i32::MAX < usize::MAX, we have num_data_blocks * block_size < usize::MAX.
            assert((num_data_blocks as int) < (total_num_blocks as int));
            // len = total_num_blocks * block_size (since len % block_size == 0 from the division).
            // Actually, len >= total_num_blocks * block_size but there might be remainder.
            // However, we know len / block_size = total_num_blocks, so:
            // total_num_blocks * block_size <= len < (total_num_blocks + 1) * block_size.
            // Use lemma to establish: (len / block_size) * block_size <= len.
            Self::lemma_div_mul_le(len as int, block_size as int);
            assert((total_num_blocks as int) == (len as int) / (block_size as int));
            assert((total_num_blocks as int) * (block_size as int) <= len as int);
            // num_data_blocks * block_size < total_num_blocks * block_size <= len < usize::MAX.
            Self::lemma_mul_inequality(num_data_blocks as int, total_num_blocks as int, block_size as int);
            assert((num_data_blocks as int) * (block_size as int) < (total_num_blocks as int) * (block_size as int));
            assert((len as int) < (usize::MAX as int));
            assert((num_data_blocks as int) * (block_size as int) <= (usize::MAX as int));

            // data_addr + num_data_blocks * block_size.
            // data_addr = addr + num_index_blocks * block_size.
            // data_addr + num_data_blocks * block_size = addr + num_index_blocks * block_size + num_data_blocks * block_size.
            //                                         = addr + (num_index_blocks + num_data_blocks) * block_size.
            //                                         = addr + total_num_blocks * block_size.
            //                                         <= addr + len (since total_num_blocks * block_size <= len).
            // From precondition: addr + len >= addr (no wrap), and len < i32::MAX.
            // So addr + len <= usize::MAX (implicitly, since addr + len doesn't wrap).
            assert((data_addr as int) == (addr as int) + (num_index_blocks as int) * (block_size as int));
            // Use distributive property.
            Self::lemma_distributive(num_index_blocks as int, num_data_blocks as int, block_size as int);
            assert((num_index_blocks as int) * (block_size as int) + (num_data_blocks as int) * (block_size as int)
                == (num_index_blocks as int + num_data_blocks as int) * (block_size as int));
            assert((data_addr as int) + (num_data_blocks as int) * (block_size as int)
                == (addr as int) + (num_index_blocks as int) * (block_size as int) + (num_data_blocks as int) * (block_size as int));
            assert((data_addr as int) + (num_data_blocks as int) * (block_size as int)
                == (addr as int) + (num_index_blocks as int + num_data_blocks as int) * (block_size as int));
            assert((num_index_blocks as int + num_data_blocks as int) == (total_num_blocks as int));
            assert((data_addr as int) + (num_data_blocks as int) * (block_size as int)
                == (addr as int) + (total_num_blocks as int) * (block_size as int));
            assert((total_num_blocks as int) * (block_size as int) <= len as int);
            assert((addr as int) + (total_num_blocks as int) * (block_size as int) <= (addr as int) + (len as int));
            // addr + len doesn't overflow (from precondition addr + len >= addr).
            // This means addr + len <= usize::MAX.
            assert((addr as int) + (len as int) <= (usize::MAX as int));
            assert((data_addr as int) + (num_data_blocks as int) * (block_size as int) <= (usize::MAX as int));

            // Prove metadata/data disjointness condition for invariant.
            // data_addr = addr + num_index_blocks * block_size.
            // Since addr > 0 (from precondition), we have:
            // data_addr = addr + num_index_blocks * block_size > num_index_blocks * block_size.
            // Therefore: data_addr >= num_index_blocks * block_size.
            assert((data_addr as int) == (addr as int) + (num_index_blocks as int) * (block_size as int));
            assert(addr as int > 0);
            assert((data_addr as int) > (num_index_blocks as int) * (block_size as int));
            assert((data_addr as int) >= (num_index_blocks as int) * (block_size as int));

            // Use the lemma to prove inv() holds.
            Self::lemma_inv_from_components(&result_slab);
            // Reveal view fields.
            Self::lemma_view_fields(&result_slab);
            // Prove that slab is empty (no data blocks allocated yet).
            Self::lemma_new_slab_is_empty(&result_slab);
            // Prove data_addr > addr (since data_addr = addr + index_region_size and index_region_size > 0).
            assert(result_slab.data_addr as int > addr as int);
            // Prove data_addr is aligned to block_size.
            assert(result_slab.data_addr as int % (block_size as int) == 0);
        }

        Ok(result_slab)
    }


    ///
    /// # Description
    ///
    /// Creates a slab allocator at a specific offset within a larger memory region.
    /// This is used by Kheap to create multiple slabs in contiguous memory regions.
    ///
    /// # Parameters
    ///
    /// - `base_addr`: Base address of the entire memory region.
    /// - `slab_size`: Size of each slab region in bytes.
    /// - `offset`: Offset index (0-7) indicating which slab region.
    /// - `block_size`: Block size for this slab.
    ///
    /// # Returns
    ///
    /// A slab whose data region is within [base_addr + offset * slab_size, base_addr + (offset+1) * slab_size).
    ///


    ///
    /// # Description
    ///
    /// Allocates a block of memory from the slab allocator.
    ///
    /// # Returns
    ///
    /// Upon success, the address of the allocated block is returned.
    /// Upon failure, an error is returned instead.
    ///
    /// # Verus Equivalence
    ///
    /// Compared to the original: return type `*mut u8` → `usize` (no raw pointers in Verus).
    /// `self.index.alloc()?` → explicit `match` (required for proof blocks on error path).
    /// Pointer arithmetic `data_addr.add(...)` → integer arithmetic `data_addr + ...`.
    /// Core logic is identical: alloc bitmap bit → compute block address → return.
    ///
    pub fn allocate(&mut self) -> (result: Result<*mut u8, Error>)
        requires
            old(self).inv(),
        ensures
            self.inv(),
            result is Ok ==> {
                let addr = result->Ok_0 as int;
                let block_idx = old(self)@.addr_to_block_idx(addr);
                &&& old(self)@.is_valid_addr(addr)
                &&& 0 <= block_idx < self@.num_data_blocks
                &&& !old(self)@.is_allocated(block_idx)
                &&& self@.is_allocated(block_idx)
                // Frame: static fields unchanged.
                &&& self@.num_data_blocks == old(self)@.num_data_blocks
                &&& self@.block_size == old(self)@.block_size
                &&& self@.data_addr == old(self)@.data_addr
                // Frame: allocated_blocks is old plus the new block (Set-based, no forall).
                &&& self@.allocated_blocks =~= old(self)@.allocated_blocks.insert(block_idx)
                // Explicit postcondition that address is within buffer bounds.
                &&& old(self)@.is_within_buffer(addr)
                // Returned address is non-null (derivable from data_addr as int > 0 and is_valid_addr).
                &&& addr > 0
            },
            // Error case: state unchanged and slab was full (no capacity).
            result is Err ==> (self@ == old(self)@ && !old(self)@.can_allocate()),
            // Liveness: if there's free capacity, allocation succeeds.
            old(self)@.can_allocate() ==> result is Ok,
    {
        let alloc_result = self.index.alloc();

        // Allocate a free block from the bitmap (explicit match for proof on error path).
        let block: usize = match alloc_result {
            Ok(b) => b,
            Err(e) => {
                proof {
                    // Liveness: if can_allocate(), this branch should be unreachable.
                    // can_allocate() implies has_free_bit(), and alloc guarantees:
                    // has_free_bit() ==> result is Ok. So if we're here, !can_allocate().
                    if old(self)@.can_allocate() {
                        old(self).lemma_can_allocate_implies_bitmap_has_free_bit();
                        // old(self).index@.has_free_bit() is true.
                        // But bitmap.alloc postcondition says: has_free_bit() ==> result is Ok.
                        // So alloc_result should be Ok, not Err. Contradiction.
                        // (The postcondition is vacuously true for this path.)
                        assert(false);
                    }
                    // On error, bitmap is unchanged, so Slab::inv() still holds.
                    assert(self.index.inv());
                    assert(self.index@.set_bits =~= old(self).index@.set_bits);
                    // All other fields are unchanged (they were never modified).
                    assert(self.block_size == old(self).block_size);
                    assert(self.num_data_blocks == old(self).num_data_blocks);
                    assert(self.num_index_blocks == old(self).num_index_blocks);
                    assert(self.data_addr == old(self).data_addr);
                    assert(self.base_addr == old(self).base_addr);
                    assert(self.total_len == old(self).total_len);
                    // Bitmap size is unchanged.
                    assert(self.index@.number_of_bits() == old(self).index@.number_of_bits());
                    // Prove index blocks are still set (set_bits unchanged means is_bit_set unchanged).
                    assert forall|i: int| 0 <= i < self.num_index_blocks as int
                        implies self.index.is_bit_set(i) by {
                        assert(old(self).index.is_bit_set(i));
                        // set_bits unchanged implies is_bit_set unchanged.
                        assert(self.index@.set_bits.contains(i) == old(self).index@.set_bits.contains(i));
                    }
                    // Now inv() should hold.
                    assert(self.inv());
                    // Prove view equality for self@ == old(self)@.
                    // View fields: num_data_blocks, block_size, data_addr, allocated_blocks.
                    // All scalar fields are unchanged.
                    // allocated_blocks = { j | is_allocated(j) } = { j | is_bit_set(num_idx + j) }.
                    // Since set_bits are unchanged, is_bit_set is unchanged for all indices.
                    assert(self@.num_data_blocks == old(self)@.num_data_blocks);
                    assert(self@.block_size == old(self)@.block_size);
                    assert(self@.data_addr == old(self)@.data_addr);
                    // Prove allocated_blocks equality.
                    assert(self@.allocated_blocks =~= old(self)@.allocated_blocks) by {
                        assert forall|j: int| 0 <= j < self.num_data_blocks as int implies
                            (self@.allocated_blocks.contains(j) == old(self)@.allocated_blocks.contains(j)) by {
                            let bitmap_idx = self.num_index_blocks as int + j;
                            // set_bits unchanged implies is_bit_set unchanged.
                            assert(self.index@.set_bits.contains(bitmap_idx) == old(self).index@.set_bits.contains(bitmap_idx));
                            assert(self.index.is_bit_set(bitmap_idx) == old(self).index.is_bit_set(bitmap_idx));
                        }
                    }
                    assert(self@ == old(self)@);
                }
                return Err(e);
            }
        };

        proof {
            let block_int: int = block as int;

            // The bitmap alloc ensures the bit was previously unset.
            assert(!old(self).index.is_bit_set(block_int));

            // Using the invariant, any index block bit is set, so an unset bit cannot be within them.
            assert(block_int >= self.num_index_blocks as int) by {
                if block_int < self.num_index_blocks as int {
                    assert(old(self).index.is_bit_set(block_int));
                }
            };

            // Bounds: bitmap length equals index + data blocks.
            assert(self.inv());
            assert(old(self).inv());
            assert(old(self).num_index_blocks + old(self).num_data_blocks == self.index@.number_of_bits());
            assert(block_int < self.index@.number_of_bits());
            assert(block_int < (self.num_index_blocks + self.num_data_blocks) as int);
        }

        let block_idx: usize = block - self.num_index_blocks;

        // Prove bounds for safe multiplication.
        proof {
            assert(block_idx < self.num_data_blocks);
            assert(self.block_size > 0);
            // From invariant: num_data_blocks * block_size <= usize::MAX.
            // Since block_idx < num_data_blocks, we have block_idx * block_size < num_data_blocks * block_size <= usize::MAX.
            Self::lemma_mul_inequality(block_idx as int, self.num_data_blocks as int, self.block_size as int);
            assert((block_idx as int) * (self.block_size as int) < (self.num_data_blocks as int) * (self.block_size as int));
            assert((self.num_data_blocks as int) * (self.block_size as int) <= usize::MAX as int);
            assert((block_idx as int) * (self.block_size as int) <= usize::MAX as int);

            // From invariant: data_addr + num_data_blocks * block_size <= usize::MAX.
            // Since block_idx * block_size < num_data_blocks * block_size,
            // data_addr + block_idx * block_size < data_addr + num_data_blocks * block_size <= usize::MAX.
            assert((self.data_addr as int) + (block_idx as int) * (self.block_size as int)
                < (self.data_addr as int) + (self.num_data_blocks as int) * (self.block_size as int));
            assert((self.data_addr as int) + (self.num_data_blocks as int) * (self.block_size as int) <= usize::MAX as int);
            assert((self.data_addr as int) + (block_idx as int) * (self.block_size as int) <= usize::MAX as int);
        }

        let product: usize = block_idx * self.block_size;
        let block_addr: *mut u8 = ptr_add(self.data_addr, product);

        proof {
            let block_idx_int: int = block_idx as int;
            let addr_int: int = block_addr as int;
            let bs: int = self.block_size as int;
            let ndb: int = self.num_data_blocks as int;

            // Block index bounds.
            assert(0 <= block_idx_int < ndb);

            // Allocation flips only the chosen bit.
            assert(self.index.is_bit_set(self.num_index_blocks as int + block_idx_int));
            assert(!old(self).index.is_bit_set(self.num_index_blocks as int + block_idx_int));

            // Other bits unchanged.
            assert forall|i: int| 0 <= i < ndb && i != block_idx_int implies
                #[trigger] self.index.is_bit_set(self.num_index_blocks as int + i)
                    == #[trigger] old(self).index.is_bit_set(self.num_index_blocks as int + i)
            by {
                let global_idx = self.num_index_blocks as int + i;
                if global_idx == block as int {
                    assert(i == block_idx_int);
                }
            }

            // Address validity proofs.
            assert(addr_int == self.data_addr as int + block_idx_int * bs);
            Self::lemma_mul_inequality(block_idx_int, ndb, bs);
            assert(addr_int < self.data_addr as int + ndb * bs);
            Self::lemma_mul_divisible(block_idx_int, bs);
            assert((addr_int - self.data_addr as int) % bs == 0);
            assert(old(self)@.is_valid_addr(addr_int));

            // Block index computation.
            Self::lemma_div_cancel(block_idx_int, bs);
            assert(old(self)@.addr_to_block_idx(addr_int) == block_idx_int);

            // Allocation status.
            assert(!old(self)@.is_allocated(block_idx_int));
            assert(self@.is_allocated(block_idx_int));

            // Unchanged fields and other blocks.
            assert forall|i: int| 0 <= i < ndb && i != block_idx_int
                implies self@.is_allocated(i) == old(self)@.is_allocated(i) by {}

            // Address within buffer.
            assert(old(self)@.is_within_buffer(addr_int));
        }

        Ok(block_addr)
    }


    ///
    /// # Description
    ///
    /// Frees a block of memory from the slab allocator.
    ///
    /// # Parameters
    ///
    /// - `ptr`: Pointer to the block to free.
    ///
    /// # Returns
    ///
    /// Upon success, `Ok(())` is returned. Upon failure, an error is returned instead.
    ///
    /// # Safety
    ///
    /// This function is unsafe for the following reasons:
    ///
    /// - It dereferences the pointer `ptr`.
    ///
    pub unsafe fn deallocate(&mut self, ptr: *const u8) -> (result: Result<(), Error>)
        requires
            old(self).inv(),
            old(self)@.is_valid_addr(ptr as int),
            // Use can_deallocate for clearer specification.
            old(self)@.can_deallocate(old(self)@.addr_to_block_idx(ptr as int)),
        ensures
            self.inv(),
            result is Ok ==> {
                let block_idx = old(self)@.addr_to_block_idx(ptr as int);
                &&& !self@.is_allocated(block_idx)
                // Frame: static fields unchanged.
                &&& self@.num_data_blocks == old(self)@.num_data_blocks
                &&& self@.block_size == old(self)@.block_size
                &&& self@.data_addr == old(self)@.data_addr
                // Frame: allocated_blocks is old minus the freed block (Set-based, no forall).
                &&& self@.allocated_blocks =~= old(self)@.allocated_blocks.remove(block_idx)
                // Liveness: after deallocation, allocation is possible (at least one free block).
                &&& self@.can_allocate()
            },
            result is Err ==> self@ == old(self)@,
            // Liveness: if preconditions are met (block is valid and allocated), deallocation succeeds.
            result is Ok,
    {
        // Check if the pointer lies in a memory region that is not managed by this allocator.
        // Source uses pointer comparisons; Verus uses integer arithmetic.
        proof {
            assert((self.data_addr as int) + (self.num_data_blocks as int) * (self.block_size as int) <= usize::MAX as int);
        }
        if (ptr as usize) < (self.data_addr as usize)
            || (ptr as usize) >= (self.data_addr as usize) + self.num_data_blocks * self.block_size
        {
            return Err(Error::new(ErrorCode::BadAddress, "pointer out of bounds"));
        }

        // Compute the block index.
        // Source uses `ptr.offset_from_unsigned()`; Verus uses integer subtraction.

        proof {
            // From is_valid_addr:
            assert(ptr as int >= self.data_addr as int);
            assert((ptr as int) < (self.data_addr as int) + (self.num_data_blocks as int) * (self.block_size as int));
            assert(((ptr as int) - (self.data_addr as int)) % (self.block_size as int) == 0);

            // Therefore (ptr - self.data_addr) is non-negative and bounded.
            let offset: int = (ptr as int) - (self.data_addr as int);
            assert(offset >= 0);
            assert(offset < (self.num_data_blocks as int) * (self.block_size as int));

            // And offset / block_size < num_data_blocks.
            let block_idx: int = offset / (self.block_size as int);
            assert(0 <= block_idx < self.num_data_blocks as int);

            // index = num_index_blocks + block_idx < num_index_blocks + num_data_blocks = number_of_bits.
            assert((self.num_index_blocks as int) + block_idx < self.index@.number_of_bits());

            // Prove no overflow for usize computation.
            // ptr >= data_addr, so ptr - data_addr >= 0 (no underflow).
            // block_idx < num_data_blocks, and num_index_blocks + num_data_blocks fits in usize (from inv).
            // From invariant: num_index_blocks + num_data_blocks == index@.number_of_bits().
            // Bitmap number_of_bits is bounded by usize (from Bitmap invariant).
            // Therefore: num_index_blocks + block_idx < num_index_blocks + num_data_blocks <= usize::MAX.
            assert((self.num_index_blocks as int) + block_idx < (self.num_index_blocks as int) + (self.num_data_blocks as int));
            assert((self.num_index_blocks as int) + (self.num_data_blocks as int) == self.index@.number_of_bits());
            // Use lemma to expose that slab.inv() implies index.inv().
            Self::lemma_slab_inv_implies_bitmap_inv(self);
            // From index.inv() we have number_of_bits <= usize::MAX.
            assert(self.index@.number_of_bits() <= (usize::MAX as int));
            assert((self.num_index_blocks as int) + block_idx < (usize::MAX as int));
        }

        // The proof above establishes:
        // 1. ptr >= self.data_addr (from is_valid_addr precondition).
        // 2. num_index_blocks + block_idx < usize::MAX (from invariant bounds).

        let index: usize = self.num_index_blocks + ptr_offset_from(ptr, self.data_addr) / self.block_size;

        proof {
            // Prove that index < number_of_bits.
            assert((index as int) < self.index@.number_of_bits());

            // Connect index with addr_to_block_idx.
            let block_idx_spec: int = self@.addr_to_block_idx(ptr as int);
            assert(block_idx_spec == ((ptr as int) - (self.data_addr as int)) / (self.block_size as int));
            assert((index as int) == (self.num_index_blocks as int) + block_idx_spec);

            // From precondition: self@.is_allocated(block_idx_spec).
            // is_allocated(block_idx_spec) means index.is_bit_set(num_index_blocks + block_idx_spec).
            // Which is index.is_bit_set(index as int).
            assert(self@.is_allocated(block_idx_spec));
            // By definition of is_allocated in view():
            // allocated_blocks.contains(block_idx_spec) <==> index.is_bit_set(num_index_blocks + block_idx_spec)
            assert(self.index.is_bit_set(index as int));
        }

        // Check if the block is already free.
        if !self.index.test(index)? {
            return Err(Error::new(ErrorCode::BadAddress, "block is already free"));
        }

        // Free the block.
        match self.index.clear(index) {
            Ok(()) => {
                proof {
                    // After clear, the slab invariant is preserved because:
                    // - We only cleared a data block (index >= num_index_blocks).
                    // - Index blocks remain allocated.

                    // Prove index >= num_index_blocks (we're clearing a data block).
                    let block_idx_spec: int = old(self)@.addr_to_block_idx(ptr as int);
                    assert(block_idx_spec >= 0);
                    assert((index as int) == (self.num_index_blocks as int) + block_idx_spec);
                    assert((index as int) >= (self.num_index_blocks as int));

                    // Prove all index blocks are still set.
                    // clear() only changes bit at `index`, and index >= num_index_blocks.
                    // So bits 0..num_index_blocks are unchanged.
                    assert forall|j: int| 0 <= j < self.num_index_blocks as int
                        implies self.index.is_bit_set(j) by {
                        // j != index (since j < num_index_blocks <= index)
                        assert(j != index as int);
                        // clear() preserves bits at j != index.
                        // From old(self).inv(), index blocks were set.
                        assert(old(self).index.is_bit_set(j));
                        assert(self.index.is_bit_set(j) == old(self).index.is_bit_set(j));
                    }

                    Self::lemma_inv_from_components(self);

                    // Prove can_allocate() after deallocation using bitmap has_free_bit.
                    // After clearing a bit, that bit is now unset, so the bitmap has a free bit.
                    // The underlying bitmap's has_free_bit implies slab can_allocate.
                    // After clear(index), !is_bit_set(index).
                    assert(!self.index.is_bit_set(index as int));
                    // Since index < number_of_bits and !is_bit_set(index), has_free_bit is true.
                    self.index.lemma_unset_bit_implies_has_free_bit(index as int);
                    assert(self.index@.has_free_bit());

                    // Now connect has_free_bit to can_allocate via the contrapositive of
                    // lemma_bitmap_full_implies_slab_full.
                    // has_free_bit means !is_full (for bitmap).
                    // If bitmap is full, slab is full (lemma_bitmap_full_implies_slab_full).
                    // Contrapositive: if slab is not full, bitmap is not full.
                    // We'll use: has_free_bit means there exists an unset bit.
                    // This means the slab has a corresponding free data block.

                    // Direct approach: prove there's an unallocated data block.
                    // The cleared bit at index corresponds to block_idx_spec.
                    // block_idx_spec is in [0, num_data_blocks).
                    assert(0 <= block_idx_spec < self@.num_data_blocks);
                    // After clear, !is_bit_set(num_index_blocks + block_idx_spec).
                    // By view definition, !is_allocated(block_idx_spec).
                    assert(!self@.is_allocated(block_idx_spec));

                    // Use the can_allocate_implies_bitmap_has_free_bit lemma's inverse reasoning.
                    // If there's a block j in [0, num_data_blocks) that's not allocated,
                    // then used < capacity (since allocated_blocks is missing j).
                    // allocated_blocks is subset of {0,..,num_data_blocks-1}.
                    // If j is not in allocated_blocks but is in the full range,
                    // then allocated_blocks is a strict subset.
                    // For strict subsets of finite sets: |A| < |B|.

                    // Prove allocated_blocks.len() < num_data_blocks.
                    self.lemma_allocated_blocks_finite();
                    self.lemma_allocated_blocks_subset_of_range();
                    let full_range: Set<int> = set_int_range(0, self@.num_data_blocks);
                    lemma_int_range(0, self@.num_data_blocks);

                    // Witness: block_idx_spec is in full_range but not in allocated_blocks.
                    assert(full_range.contains(block_idx_spec));
                    assert(!self@.allocated_blocks.contains(block_idx_spec));

                    // Use lemma_len_subset: subset implies |A| <= |B|.
                    lemma_len_subset(self@.allocated_blocks, full_range);
                    // We have |allocated_blocks| <= |full_range| = num_data_blocks.

                    // Prove strict inequality by showing sets are not equal.
                    // If |A| == |B| and A subset_of B and both finite, then A == B.
                    // But we have witness in B not in A, so A != B.
                    // Therefore |A| < |B|.
                    assert(self@.allocated_blocks.len() <= full_range.len());

                    // Use the strict subset logic: cannot have equality.
                    // Assert negation leads to contradiction.
                    if self@.allocated_blocks =~= full_range {
                        // This would mean block_idx_spec is in allocated_blocks.
                        assert(self@.allocated_blocks.contains(block_idx_spec));
                        // But we proved !contains above. Contradiction.
                        assert(false);
                    }
                    // Since A subset_of B, |A| <= |B|, and A != B, we need |A| < |B|.
                    // For finite sets, A strict subset of B means |A| < |B|.
                    // Verus needs help: use the fact that membership differs.
                    assert(self@.allocated_blocks !~= full_range);

                    // The key insight: for finite sets A, B where A.subset_of(B),
                    // if exists x in B with x not in A, then |A| < |B|.
                    // This is because A ∪ {x} would have cardinality |A| + 1,
                    // and A ∪ {x} is still subset of B, so |A| + 1 <= |B|.
                    // Therefore |A| < |B|.
                    // Let's assert what we need and rely on Verus's set reasoning.
                    assert(self@.allocated_blocks.len() < self@.num_data_blocks) by {
                        // allocated_blocks subset_of full_range and block_idx_spec in full_range - allocated_blocks.
                        // For finite sets: |A| < |B| when A strict subset of B.
                        // We can use insert lemma: A.insert(x).len() == A.len() + 1 when x not in A.
                        let with_witness = self@.allocated_blocks.insert(block_idx_spec);
                        // with_witness has one more element than allocated_blocks.
                        // with_witness is still a subset of full_range.
                        assert forall|x: int| with_witness.contains(x) implies full_range.contains(x) by {
                            if x == block_idx_spec {
                                assert(full_range.contains(block_idx_spec));
                            } else {
                                assert(self@.allocated_blocks.contains(x));
                                assert(full_range.contains(x));
                            }
                        }
                        assert(with_witness.subset_of(full_range));
                        // with_witness.len() = allocated_blocks.len() + 1.
                        axiom_set_insert_len(self@.allocated_blocks, block_idx_spec);
                        assert(with_witness.len() == self@.allocated_blocks.len() + 1);
                        // with_witness subset_of full_range, so |with_witness| <= |full_range|.
                        lemma_len_subset(with_witness, full_range);
                        assert(with_witness.len() <= full_range.len());
                        // Therefore allocated_blocks.len() + 1 <= num_data_blocks.
                        assert(self@.allocated_blocks.len() + 1 <= self@.num_data_blocks);
                    }

                    assert(self@.used() < self@.capacity());
                    assert(self@.free() > 0);
                    assert(self@.can_allocate());
                }
                Ok(())
            },
            Err(e) => {
                proof {
                    // On error, bitmap is unchanged (self.index@ == old(self).index@).
                    // Since old(self).inv(), and bitmap is unchanged, self.inv() still holds.
                    // is_bit_set is based on @, which is unchanged.
                    assert forall|j: int| 0 <= j < self.num_index_blocks as int
                        implies self.index.is_bit_set(j) by {
                        assert(self.index@ == old(self).index@);
                        assert(self.index.is_bit_set(j) == old(self).index.is_bit_set(j));
                        assert(old(self).index.is_bit_set(j));
                    }
                    Self::lemma_inv_from_components(self);
                }
                Err(e)
            }
        }
    }
}

} // verus!
