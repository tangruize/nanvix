// Copyright(c) The Maintainers of Nanvix.
// Licensed under the MIT License.

//==================================================================================================
// Slab Allocator
//==================================================================================================

use crate::libs::{
    bitmap::Bitmap,
    error::{
        Error,
        ErrorCode,
    },
    raw_array::{
        axiom_u8_zero_is_0,
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


verus! {


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
#[cfg_attr(not(verus_keep_ghost), derive(Debug))]
pub struct Slab {
    /// An index that keeps track of free blocks.
    index: Bitmap,
    /// Base address of data blocks (as usize for verus compatibility).
    data_addr: usize,
    /// Number of index blocks in the slab.
    num_index_blocks: usize,
    /// Number of data blocks in the slab.
    num_data_blocks: usize,
    /// Size of blocks in the slab.
    block_size: usize,
    // Issue 6 FIX: Store buffer base address and length for bounds checking.
    /// Base address of the entire slab buffer (including index region).
    base_addr: usize,
    /// Total length of the slab buffer in bytes.
    total_len: usize,
}


/// A view of the Slab as an abstract specification.
#[verifier::ext_equal]
pub struct SlabView {
    /// Set of allocated block indices (relative to data blocks).
    pub allocated_blocks: Set<int>,
    /// Total number of data blocks.
    pub num_data_blocks: int,
    /// Block size in bytes.
    pub block_size: int,
    /// Base address of data region.
    pub data_addr: int,
    // Issue 6 FIX: Track buffer bounds in view for spec-level bounds checking.
    /// Base address of the entire slab buffer (including index region).
    pub base_addr: int,
    /// Total length of the slab buffer in bytes.
    pub total_len: int,
}

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
    pub unsafe fn from_raw_parts(
        addr: usize,
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
            addr % block_size == 0,
            addr > 0,
            // Memory region must not wrap around and fit in address space.
            (addr as int) + (len as int) <= (usize::MAX as int),
            // Total number of blocks must be a multiple of 8.
            (len / block_size) % (u8::BITS as usize) == 0,
            // Ensure we have enough blocks for a valid slab (at least 8).
            len / block_size >= 8,
            // Issue 1 FIX: Zero-initialization of the bitmap backing storage.
            // `RawArray::from_raw_addr` zeroes the region before returning and its
            // postcondition exposes `is_zero` for every byte, which we rely on when
            // constructing the bitmap. No caller-side zeroing precondition is required.
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
                // Issue 6 FIX: Buffer bounds are recorded.
                &&& slab@.base_addr == addr as int
                &&& slab@.total_len == len as int
            },
    {
        // Check if length is invalid.
        if len == 0 || len >= i32::MAX as usize {
            return Err(Error::new(ErrorCode::InvalidArgument, "invalid slab length"));
        }

        // Check if block size is valid.
        if block_size == 0 || block_size >= i32::MAX as usize || block_size > len {
            return Err(Error::new(ErrorCode::InvalidArgument, "invalid block size"));
        }

        // Check if the `block_size` is a power of two using the verified function.
        if !Self::is_power_of_two(block_size) {
            return Err(Error::new(ErrorCode::InvalidArgument, "block size is not a power of two"));
        }

        // At this point, is_power_of_two returned true, so spec_is_power_of_two holds.
        assert(Self::spec_is_power_of_two(block_size as int));

        // Check if `addr` is aligned to `block_size`.
        if addr % block_size != 0 {
            return Err(Error::new(ErrorCode::InvalidArgument, "unaligned start address"));
        }

        // Compute layout of the slab allocator.
        let total_num_blocks: usize = len / block_size;
        if total_num_blocks % (u8::BITS as usize) != 0 {
            return Err(Error::new(ErrorCode::InvalidArgument, "invalid number of blocks"));
        }

        // Need at least 8 blocks for valid slab.
        if total_num_blocks < 8 {
            return Err(Error::new(ErrorCode::InvalidArgument, "too few blocks"));
        }

        let index_len: usize = total_num_blocks / u8::BITS as usize;

        // Prove that index_len >= 1.
        assert(index_len >= 1);

        let num_index_blocks: usize = (index_len / block_size)
            + if index_len % block_size == 0 { 0 } else { 1 };

        // Check that num_index_blocks >= 1.
        if num_index_blocks == 0 {
            return Err(Error::new(ErrorCode::InvalidArgument, "no index blocks"));
        }

        // Check that num_index_blocks < total_num_blocks.
        if num_index_blocks >= total_num_blocks {
            return Err(Error::new(ErrorCode::InvalidArgument, "too many index blocks"));
        }

        let num_data_blocks: usize = total_num_blocks - num_index_blocks;

        // Check that we have at least one data block.
        if num_data_blocks == 0 {
            return Err(Error::new(ErrorCode::InvalidArgument, "no data blocks"));
        }

        // Check for overflow in address calculation.
        if block_size == 0 {
            return Err(Error::new(ErrorCode::InvalidArgument, "block size is zero"));
        }
        let max_blocks: usize = usize::MAX / block_size;
        if num_index_blocks > max_blocks {
            return Err(Error::new(ErrorCode::InvalidArgument, "address overflow"));
        }

        // Now we can safely multiply using checked_mul.
        let index_region_size: usize = match num_index_blocks.checked_mul(block_size) {
            Some(v) => v,
            None => return Err(Error::new(ErrorCode::InvalidArgument, "address overflow")),
        };

        // Check index_region_size > 0.
        if index_region_size == 0 {
            return Err(Error::new(ErrorCode::InvalidArgument, "index region size is zero"));
        }

        if addr > usize::MAX - index_region_size {
            return Err(Error::new(ErrorCode::InvalidArgument, "address overflow"));
        }
        let data_addr: usize = addr + index_region_size;

        // Check if `data_addr` is aligned to `block_size`.
        if data_addr % block_size != 0 {
            return Err(Error::new(ErrorCode::InvalidArgument, "unaligned data address"));
        }

        // Instantiate index.
        let storage: RawArray<u8> = RawArray::from_raw_addr(addr, index_len)?;

        // Prove that all bytes in storage are zero (required by Bitmap::from_raw_array).
        proof {
            // from_raw_addr ensures is_zero for each element.
            // axiom_u8_zero_is_0 converts is_zero(t) to t == 0.
            assert forall|i: int| 0 <= i < storage@.len() implies storage@[i] == 0u8 by {
                axiom_u8_zero_is_0(storage@[i]);
            }
        }

        let mut index: Bitmap = Bitmap::from_raw_array(storage);

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

        // Initialize index: mark index blocks as allocated.
        let mut i: usize = 0;
        while i < num_index_blocks
            invariant
                index.inv(),
                i <= num_index_blocks,
                num_index_blocks < total_num_blocks,
                num_index_blocks > 0,
                num_data_blocks > 0,
                block_size > 0,
                data_addr > 0,
                num_index_blocks + num_data_blocks == total_num_blocks,
                index@.number_of_bits() == total_num_blocks as int,
                num_index_blocks as int + num_data_blocks as int == index@.number_of_bits(),
                // All bits from 0 to i are set.
                forall|j: int| 0 <= j < i as int ==> index.is_bit_set(j),
                // All bits from i to end are not set (from initial state).
                forall|j: int| i as int <= j < index@.number_of_bits() ==> !index.is_bit_set(j),
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
            base_addr: addr,
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

            // Issue 2 FIX: Prove metadata/data disjointness condition for invariant.
            // data_addr = addr + num_index_blocks * block_size.
            // Since addr > 0 (from precondition), we have:
            // data_addr = addr + num_index_blocks * block_size > num_index_blocks * block_size.
            // Therefore: data_addr >= num_index_blocks * block_size.
            assert((data_addr as int) == (addr as int) + (num_index_blocks as int) * (block_size as int));
            assert(addr > 0);
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
    /// # Safety
    ///
    /// Caller must ensure the memory region is valid.
    #[verifier::rlimit(60)]
    pub unsafe fn from_raw_parts_at_offset(
        base_addr: usize,
        slab_size: usize,
        offset: usize,
        block_size: usize,
    ) -> (result: Result<Slab, Error>)
        requires
            base_addr > 0,
            slab_size > 0,
            slab_size < i32::MAX as usize,
            offset < 8,
            block_size > 0,
            block_size < i32::MAX as usize,
            block_size <= slab_size,
            Self::spec_is_power_of_two(block_size as int),
            // Alignment: base_addr + offset * slab_size must be aligned to block_size.
            ((base_addr as int) + (offset as int) * (slab_size as int)) % (block_size as int) == 0,
            // No overflow: the END of this slab region (base_addr + (offset+1) * slab_size) fits.
            (base_addr as int) + ((offset as int) + 1) * (slab_size as int) <= (usize::MAX as int),
            // Overflow check: offset * slab_size fits in usize.
            (offset as int) * (slab_size as int) <= (usize::MAX as int),
            // Overflow check: base_addr + offset * slab_size fits in usize.
            (base_addr as int) + (offset as int) * (slab_size as int) <= (usize::MAX as int),
            // Additional preconditions for from_raw_parts:
            // Total number of blocks must be a multiple of 8.
            (slab_size / block_size) % (u8::BITS as usize) == 0,
            // Ensure we have enough blocks for a valid slab (at least 8).
            slab_size / block_size >= 8,
        ensures
            result is Ok ==> {
                let slab = result->Ok_0;
                &&& slab.inv()
                &&& slab@.block_size == block_size as int
                &&& slab@.num_data_blocks > 0
                // Freshly initialized: no blocks allocated (Set-based, no forall).
                &&& slab@.allocated_blocks =~= Set::<int>::empty()
                // Critical: data region is within the assigned slice.
                &&& slab@.data_addr >= (base_addr as int) + (offset as int) * (slab_size as int)
                &&& slab@.data_addr + slab@.num_data_blocks * slab@.block_size
                    <= (base_addr as int) + ((offset as int) + 1) * (slab_size as int)
                // Alignment: data_addr is aligned to block_size.
                &&& slab@.is_aligned()
            },
    {
        // Calculate the address for this slab region.
        // Preconditions ensure no overflow.
        let offset_times_slab: usize = offset * slab_size;
        let addr: usize = base_addr + offset_times_slab;

        // Prove the precondition for from_raw_parts: addr + slab_size <= usize::MAX.
        proof {
            // addr = base_addr + offset * slab_size.
            assert((addr as int) == (base_addr as int) + (offset as int) * (slab_size as int));

            // Use the distributive lemma: (offset + 1) * slab_size = offset * slab_size + slab_size.
            Self::lemma_mul_distribute((offset as int), (slab_size as int));
            assert(((offset as int) + 1int) * (slab_size as int)
                   == (offset as int) * (slab_size as int) + (slab_size as int));

            // addr + slab_size = base_addr + offset * slab_size + slab_size
            //                  = base_addr + (offset + 1) * slab_size.
            assert((addr as int) + (slab_size as int)
                   == (base_addr as int) + (offset as int) * (slab_size as int) + (slab_size as int));
            assert((addr as int) + (slab_size as int)
                   == (base_addr as int) + ((offset as int) + 1int) * (slab_size as int));

            // From precondition: base_addr + (offset + 1) * slab_size <= usize::MAX.
            assert((base_addr as int) + ((offset as int) + 1int) * (slab_size as int) <= (usize::MAX as int));
            assert((addr as int) + (slab_size as int) <= (usize::MAX as int));
        }

        // Use the existing from_raw_parts to create the slab.
        Self::from_raw_parts(addr, slab_size, block_size)
    }


    ///
    /// # Description
    ///
    /// Returns the number of data blocks in the slab.
    ///
    pub fn num_data_blocks(&self) -> (result: usize)
        requires self.inv(),
        ensures result as int == self@.num_data_blocks,
    {
        self.num_data_blocks
    }


    ///
    /// # Description
    ///
    /// Returns the block size.
    ///
    pub fn block_size(&self) -> (result: usize)
        requires self.inv(),
        ensures result as int == self@.block_size,
    {
        self.block_size
    }


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
    pub fn allocate(&mut self) -> (result: Result<usize, Error>)
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
                // Returned address is non-null (derivable from data_addr > 0 and is_valid_addr).
                &&& addr > 0
            },
            // Error case: state unchanged and slab was full (no capacity).
            result is Err ==> (self@ == old(self)@ && !old(self)@.can_allocate()),
            // Liveness: if there's free capacity, allocation succeeds.
            old(self)@.can_allocate() ==> result is Ok,
    {
        let alloc_result = self.index.alloc();

        // Handle error case explicitly.
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
                    assert(self.index@.bits =~= old(self).index@.bits);
                    // All other fields are unchanged (they were never modified).
                    assert(self.block_size == old(self).block_size);
                    assert(self.num_data_blocks == old(self).num_data_blocks);
                    assert(self.num_index_blocks == old(self).num_index_blocks);
                    assert(self.data_addr == old(self).data_addr);
                    assert(self.base_addr == old(self).base_addr);
                    assert(self.total_len == old(self).total_len);
                    // Bitmap size is unchanged.
                    assert(self.index@.number_of_bits() == old(self).index@.number_of_bits());
                    // Prove index blocks are still set (bits unchanged means is_bit_set unchanged).
                    assert forall|i: int| 0 <= i < self.num_index_blocks as int
                        implies self.index.is_bit_set(i) by {
                        assert(old(self).index.is_bit_set(i));
                        // Use lemma: equal bits implies equal is_bit_set.
                        self.index.lemma_bits_equal_implies_is_bit_set_equal(&old(self).index, i);
                    }
                    // Now inv() should hold.
                    assert(self.inv());
                    // Prove view equality for self@ == old(self)@.
                    // View fields: num_data_blocks, block_size, data_addr, allocated_blocks.
                    // All scalar fields are unchanged.
                    // allocated_blocks = { j | is_allocated(j) } = { j | is_bit_set(num_idx + j) }.
                    // Since bits are unchanged, is_bit_set is unchanged for all indices.
                    assert(self@.num_data_blocks == old(self)@.num_data_blocks);
                    assert(self@.block_size == old(self)@.block_size);
                    assert(self@.data_addr == old(self)@.data_addr);
                    // Prove allocated_blocks equality.
                    assert(self@.allocated_blocks =~= old(self)@.allocated_blocks) by {
                        assert forall|j: int| 0 <= j < self.num_data_blocks as int implies
                            (self@.allocated_blocks.contains(j) == old(self)@.allocated_blocks.contains(j)) by {
                            let bitmap_idx = self.num_index_blocks as int + j;
                            self.index.lemma_bits_equal_implies_is_bit_set_equal(&old(self).index, bitmap_idx);
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
        let block_addr: usize = self.data_addr + product;

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
    /// - `addr`: Address of the block to free.
    ///
    /// # Returns
    ///
    /// Upon success, `Ok(())` is returned. Upon failure, an error is returned instead.
    ///
    pub fn deallocate(&mut self, addr: usize) -> (result: Result<(), Error>)
        requires
            old(self).inv(),
            old(self)@.is_valid_addr(addr as int),
            // Use can_deallocate for clearer specification.
            old(self)@.can_deallocate(old(self)@.addr_to_block_idx(addr as int)),
        ensures
            self.inv(),
            result is Ok ==> {
                let block_idx = old(self)@.addr_to_block_idx(addr as int);
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
        // Issue 3 FIX: Keep runtime bounds check for defensive programming.
        // This protects against unverified callers that may violate preconditions.
        // Check if the address is below the data region.
        if addr < self.data_addr {
            return Err(Error::new(ErrorCode::BadAddress, "pointer out of bounds (below data region)"));
        }

        // Check if the address is beyond the data region.
        // Compute end of data region carefully to avoid overflow.
        // From invariant: data_addr + num_data_blocks * block_size <= usize::MAX.
        proof {
            assert((self.data_addr as int) + (self.num_data_blocks as int) * (self.block_size as int) <= usize::MAX as int);
        }
        let data_region_size: usize = self.num_data_blocks * self.block_size;
        let data_region_end: usize = self.data_addr + data_region_size;
        if addr >= data_region_end {
            return Err(Error::new(ErrorCode::BadAddress, "pointer out of bounds (beyond data region)"));
        }

        // Check if the address is properly aligned to block size.
        if (addr - self.data_addr) % self.block_size != 0 {
            return Err(Error::new(ErrorCode::BadAddress, "unaligned block address"));
        }

        // Compute the bitmap index for this address.
        // Since precondition guarantees is_valid_addr, we know:
        // - addr >= data_addr
        // - addr < data_addr + num_data_blocks * block_size
        // - (addr - data_addr) % block_size == 0

        proof {
            // From is_valid_addr:
            assert(addr as int >= self.data_addr as int);
            assert((addr as int) < (self.data_addr as int) + (self.num_data_blocks as int) * (self.block_size as int));
            assert(((addr as int) - (self.data_addr as int)) % (self.block_size as int) == 0);

            // Therefore (addr - self.data_addr) is non-negative and bounded.
            let offset: int = (addr as int) - (self.data_addr as int);
            assert(offset >= 0);
            assert(offset < (self.num_data_blocks as int) * (self.block_size as int));

            // And offset / block_size < num_data_blocks.
            let block_idx: int = offset / (self.block_size as int);
            assert(0 <= block_idx < self.num_data_blocks as int);

            // index = num_index_blocks + block_idx < num_index_blocks + num_data_blocks = number_of_bits.
            assert((self.num_index_blocks as int) + block_idx < self.index@.number_of_bits());

            // Prove no overflow for usize computation.
            // addr >= data_addr, so addr - data_addr >= 0 (no underflow).
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
        // 1. addr >= self.data_addr (from is_valid_addr precondition).
        // 2. num_index_blocks + block_idx < usize::MAX (from invariant bounds).

        let index: usize = self.num_index_blocks + (addr - self.data_addr) / self.block_size;

        proof {
            // Prove that index < number_of_bits.
            assert((index as int) < self.index@.number_of_bits());

            // Connect index with addr_to_block_idx.
            let block_idx_spec: int = self@.addr_to_block_idx(addr as int);
            assert(block_idx_spec == ((addr as int) - (self.data_addr as int)) / (self.block_size as int));
            assert((index as int) == (self.num_index_blocks as int) + block_idx_spec);

            // From precondition: self@.is_allocated(block_idx_spec).
            // is_allocated(block_idx_spec) means index.is_bit_set(num_index_blocks + block_idx_spec).
            // Which is index.is_bit_set(index as int).
            assert(self@.is_allocated(block_idx_spec));
            // By definition of is_allocated in view():
            // allocated_blocks.contains(block_idx_spec) <==> index.is_bit_set(num_index_blocks + block_idx_spec)
            assert(self.index.is_bit_set(index as int));
        }

        // Since the block is allocated (precondition), test will return true.
        // We don't need this check given the precondition, but it matches original code.
        if !self.index.test(index)? {
            return Err(Error::new(ErrorCode::BadAddress, "block is already free"));
        }

        // Clear the bit to deallocate.
        match self.index.clear(index) {
            Ok(()) => {
                proof {
                    // After clear, the slab invariant is preserved because:
                    // - We only cleared a data block (index >= num_index_blocks).
                    // - Index blocks remain allocated.

                    // Prove index >= num_index_blocks (we're clearing a data block).
                    let block_idx_spec: int = old(self)@.addr_to_block_idx(addr as int);
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

//==================================================================================================

/// Test: from_raw_parts creates a valid slab with expected properties.
fn test_slab_from_raw_parts_verified(
    addr: usize,
    len: usize,
    block_size: usize,
)
    requires
        // Length must be valid and non-zero.
        len > 0,
        len < i32::MAX as usize,
        // Block size must be valid.
        block_size > 0,
        block_size < i32::MAX as usize,
        block_size <= len,
        // Block size must be a power of two.
        Slab::spec_is_power_of_two(block_size as int),
        // Start address must be aligned to block size.
        addr % block_size == 0,
        addr > 0,
        // Memory region fits in address space.
        (addr as int) + (len as int) <= (usize::MAX as int),
        // Total number of blocks must be a multiple of 8.
        (len / block_size) % (u8::BITS as usize) == 0,
        // Must have at least 8 blocks for a valid slab.
        len / block_size >= 8,
{
    let result = unsafe { Slab::from_raw_parts(addr, len, block_size) };
    if let Ok(slab) = result {
        // Slab should satisfy invariant.
        assert(slab.inv());
        // All data blocks are not allocated.
        assert(forall|i: int| 0 <= i < slab@.num_data_blocks ==> !slab@.is_allocated(i));
        // Block size should match.
        assert(slab@.block_size == block_size as int);
        // Data address should be properly aligned.
        assert(slab@.data_addr % (block_size as int) == 0);
        // Number of data blocks should be positive.
        assert(slab@.num_data_blocks > 0);
    }
}


/// Test: from_raw_parts followed by allocate/deallocate works correctly.
fn test_slab_from_raw_parts_allocate_verified(
    addr: usize,
    len: usize,
    block_size: usize,
)
    requires
        len > 0,
        len < i32::MAX as usize,
        block_size > 0,
        block_size < i32::MAX as usize,
        block_size <= len,
        Slab::spec_is_power_of_two(block_size as int),
        addr % block_size == 0,
        addr > 0,
        (addr as int) + (len as int) <= (usize::MAX as int),
        (len / block_size) % (u8::BITS as usize) == 0,
        // Must have at least 8 blocks for a valid slab.
        len / block_size >= 8,
{
    let result = unsafe { Slab::from_raw_parts(addr, len, block_size) };
    if let Ok(mut slab) = result {
        // Initially all data blocks are not allocated.
        assert(forall|i: int| 0 <= i < slab@.num_data_blocks ==> !slab@.is_allocated(i));

        let alloc_result = slab.allocate();
        if let Ok(alloc_addr) = alloc_result {
            proof {
                // Allocated address should be valid.
                assert(slab@.is_valid_addr(alloc_addr as int));
                // Block should be allocated.
                let block_idx = slab@.addr_to_block_idx(alloc_addr as int);
                assert(slab@.is_allocated(block_idx));
            }

            let dealloc_result = slab.deallocate(alloc_addr);
            if let Ok(()) = dealloc_result {
                proof {
                    // Block should be freed.
                    let block_idx = slab@.addr_to_block_idx(alloc_addr as int);
                    assert(!slab@.is_allocated(block_idx));
                }
            }
        }
    }
}

//==================================================================================================

/// Verified version of test_slab_creation from test.rs.
/// Tests that a slab can be created with valid parameters.
fn test_slab_creation_verified(addr: usize, len: usize, block_size: usize)
    requires
        // Simulating: vec![0u32; 1024] with block_size 4
        // len = 1024 * 4 = 4096 bytes, block_size = 4
        len > 0,
        len < i32::MAX as usize,
        block_size > 0,
        block_size < i32::MAX as usize,
        block_size <= len,
        Slab::spec_is_power_of_two(block_size as int),
        addr % block_size == 0,
        addr > 0,
        (addr as int) + (len as int) <= (usize::MAX as int),
        (len / block_size) % (u8::BITS as usize) == 0,
        // Must have at least 8 blocks for a valid slab.
        len / block_size >= 8,
{
    let slab = unsafe { Slab::from_raw_parts(addr, len, block_size) };
    if let Ok(s) = slab {
        assert(s.inv());
        assert(forall|i: int| 0 <= i < s@.num_data_blocks ==> !s@.is_allocated(i));
        assert(s@.block_size == block_size as int);
    }
}


/// Verified version of test_allocate_deallocate from test.rs.
/// Tests allocating a block and then deallocating it.
fn test_allocate_deallocate_verified(addr: usize, len: usize, block_size: usize)
    requires
        len > 0,
        len < i32::MAX as usize,
        block_size > 0,
        block_size < i32::MAX as usize,
        block_size <= len,
        Slab::spec_is_power_of_two(block_size as int),
        addr % block_size == 0,
        addr > 0,
        (addr as int) + (len as int) <= (usize::MAX as int),
        (len / block_size) % (u8::BITS as usize) == 0,
        // Must have at least 8 blocks for a valid slab.
        len / block_size >= 8,
{
    let result = unsafe { Slab::from_raw_parts(addr, len, block_size) };
    if let Ok(mut slab) = result {
        // Allocate a block.
        let block = slab.allocate();
        if let Ok(block_addr) = block {
            proof {
                // Block should be allocated.
                let block_idx = slab@.addr_to_block_idx(block_addr as int);
                assert(slab@.is_allocated(block_idx));
            }

            // Deallocate the block.
            let dealloc_result = slab.deallocate(block_addr);
            if let Ok(()) = dealloc_result {
                proof {
                    // Block should be freed.
                    let block_idx = slab@.addr_to_block_idx(block_addr as int);
                    assert(!slab@.is_allocated(block_idx));
                }
            }
        }
    }
}


/// Verified version of test_double_deallocate from test.rs.
/// Tests that double deallocation requires the block to be allocated.
/// In verus, this is expressed as a precondition on deallocate.
fn test_double_deallocate_verified(addr: usize, len: usize, block_size: usize)
    requires
        len > 0,
        len < i32::MAX as usize,
        block_size > 0,
        block_size < i32::MAX as usize,
        block_size <= len,
        Slab::spec_is_power_of_two(block_size as int),
        addr % block_size == 0,
        addr > 0,
        (addr as int) + (len as int) <= (usize::MAX as int),
        (len / block_size) % (u8::BITS as usize) == 0,
        // Must have at least 8 blocks for a valid slab.
        len / block_size >= 8,
{
    let result = unsafe { Slab::from_raw_parts(addr, len, block_size) };
    if let Ok(mut slab) = result {
        let block = slab.allocate();
        if let Ok(block_addr) = block {
            // First deallocation should succeed.
            let dealloc1 = slab.deallocate(block_addr);
            if let Ok(()) = dealloc1 {
                proof {
                    // After deallocation, block is NOT allocated.
                    let block_idx = slab@.addr_to_block_idx(block_addr as int);
                    assert(!slab@.is_allocated(block_idx));
                    // Therefore, a second deallocation would violate the precondition:
                    // old(self)@.is_allocated(old(self)@.addr_to_block_idx(addr as int))
                    // This is the verus way of expressing "double deallocate fails".
                }
            }
        }
    }
}


/// Verified version of test_allocate_out_of_bounds from test.rs.
/// Tests that deallocating an out-of-bounds address would violate preconditions.
/// In verus, this is expressed as: deallocate requires is_valid_addr(addr).
fn test_allocate_out_of_bounds_verified(addr: usize, len: usize, block_size: usize)
    requires
        len > 0,
        len < i32::MAX as usize,
        block_size > 0,
        block_size < i32::MAX as usize,
        block_size <= len,
        Slab::spec_is_power_of_two(block_size as int),
        addr % block_size == 0,
        addr > 0,
        (addr as int) + (len as int) <= (usize::MAX as int),
        (len / block_size) % (u8::BITS as usize) == 0,
        // Must have at least 8 blocks for a valid slab.
        len / block_size >= 8,
{
    let result = unsafe { Slab::from_raw_parts(addr, len, block_size) };
    if let Ok(slab) = result {
        proof {
            // An out-of-bounds address would NOT satisfy is_valid_addr.
            // For example, an address beyond the slab's data region:
            let invalid_addr = slab@.data_addr + slab@.num_data_blocks * slab@.block_size;
            // This address is NOT valid:
            assert(!slab@.is_valid_addr(invalid_addr));
            // Therefore, calling deallocate(invalid_addr) would violate the precondition.
            // This is the verus way of expressing "out of bounds deallocation fails".
        }
    }
}


/// Additional test: verify that multiple allocations exhaust the slab properly.
fn test_multiple_allocations_verified(addr: usize, len: usize, block_size: usize)
    requires
        len > 0,
        len < i32::MAX as usize,
        block_size > 0,
        block_size < i32::MAX as usize,
        block_size <= len,
        Slab::spec_is_power_of_two(block_size as int),
        addr % block_size == 0,
        addr > 0,
        (addr as int) + (len as int) <= (usize::MAX as int),
        (len / block_size) % (u8::BITS as usize) == 0,
        // Need at least 16 blocks for this test (enough for index + 2 data blocks).
        len / block_size >= 16,
{
    let result = unsafe { Slab::from_raw_parts(addr, len, block_size) };
    if let Ok(mut slab) = result {
        let alloc1 = slab.allocate();
        if let Ok(addr1) = alloc1 {
            let alloc2 = slab.allocate();
            if let Ok(addr2) = alloc2 {
                // Two allocations return different addresses.
                assert(addr1 != addr2);
                proof {
                    // Both blocks are allocated.
                    let idx1 = slab@.addr_to_block_idx(addr1 as int);
                    let idx2 = slab@.addr_to_block_idx(addr2 as int);
                    assert(slab@.is_allocated(idx1));
                    assert(slab@.is_allocated(idx2));
                    // Block indices are different.
                    assert(idx1 != idx2);
                }
            }
        }
    }
}


/// Test: address computation properties.
fn test_address_computation_verified(addr: usize, len: usize, block_size: usize)
    requires
        len > 0,
        len < i32::MAX as usize,
        block_size > 0,
        block_size < i32::MAX as usize,
        block_size <= len,
        Slab::spec_is_power_of_two(block_size as int),
        addr % block_size == 0,
        addr > 0,
        (addr as int) + (len as int) <= (usize::MAX as int),
        (len / block_size) % (u8::BITS as usize) == 0,
        // Must have at least 8 blocks for a valid slab.
        len / block_size >= 8,
{
    let result = unsafe { Slab::from_raw_parts(addr, len, block_size) };
    if let Ok(mut slab) = result {
        let alloc_result = slab.allocate();
        if let Ok(alloc_addr) = alloc_result {
            proof {
                // Verify is_valid_addr holds for allocated address.
                assert(slab@.is_valid_addr(alloc_addr as int));
                // Verify block index is within bounds.
                let block_idx = slab@.addr_to_block_idx(alloc_addr as int);
                assert(0 <= block_idx < slab@.num_data_blocks);
                // Verify the block is allocated.
                assert(slab@.is_allocated(block_idx));
            }
        }
    }
}

//==================================================================================================

/// Test: Allocation reuse - after deallocation, the same block can be reallocated.
fn test_allocation_reuse_verified(addr: usize, len: usize, block_size: usize)
    requires
        len > 0,
        len < i32::MAX as usize,
        block_size > 0,
        block_size < i32::MAX as usize,
        block_size <= len,
        Slab::spec_is_power_of_two(block_size as int),
        addr % block_size == 0,
        addr > 0,
        (addr as int) + (len as int) <= (usize::MAX as int),
        (len / block_size) % (u8::BITS as usize) == 0,
        len / block_size >= 8,
{
    let result = unsafe { Slab::from_raw_parts(addr, len, block_size) };
    if let Ok(mut slab) = result {
        // Allocate a block.
        let alloc1 = slab.allocate();
        if let Ok(addr1) = alloc1 {
            // Deallocate.
            let dealloc = slab.deallocate(addr1);
            if let Ok(()) = dealloc {
                // Allocate again - should succeed.
                let alloc2 = slab.allocate();
                if let Ok(addr2) = alloc2 {
                    proof {
                        // The second allocation should be valid.
                        assert(slab@.is_valid_addr(addr2 as int));
                        assert(slab@.is_allocated(slab@.addr_to_block_idx(addr2 as int)));
                    }
                }
            }
        }
    }
}


/// Test: Memory block alignment - all allocated addresses are aligned to block_size.
fn test_memory_block_alignment_verified(addr: usize, len: usize, block_size: usize)
    requires
        len > 0,
        len < i32::MAX as usize,
        block_size > 0,
        block_size < i32::MAX as usize,
        block_size <= len,
        Slab::spec_is_power_of_two(block_size as int),
        addr % block_size == 0,
        addr > 0,
        (addr as int) + (len as int) <= (usize::MAX as int),
        (len / block_size) % (u8::BITS as usize) == 0,
        len / block_size >= 16,
{
    let result = unsafe { Slab::from_raw_parts(addr, len, block_size) };
    if let Ok(mut slab) = result {
        let alloc1 = slab.allocate();
        if let Ok(addr1) = alloc1 {
            let alloc2 = slab.allocate();
            if let Ok(addr2) = alloc2 {
                proof {
                    // All allocated addresses should be aligned to block_size.
                    // This is a key property: addr = data_addr + block_idx * block_size.
                    // If data_addr is aligned and block_size is power of 2, result is aligned.
                    assert(slab@.is_valid_addr(addr1 as int));
                    assert(slab@.is_valid_addr(addr2 as int));
                    // Both addresses are within the data region.
                    assert(addr1 as int >= slab@.data_addr);
                    assert(addr2 as int >= slab@.data_addr);
                }
            }
        }
    }
}


/// Test: Deallocate doesn't affect other allocated blocks.
fn test_no_data_corruption_verified(addr: usize, len: usize, block_size: usize)
    requires
        len > 0,
        len < i32::MAX as usize,
        block_size > 0,
        block_size < i32::MAX as usize,
        block_size <= len,
        Slab::spec_is_power_of_two(block_size as int),
        addr % block_size == 0,
        addr > 0,
        (addr as int) + (len as int) <= (usize::MAX as int),
        (len / block_size) % (u8::BITS as usize) == 0,
        len / block_size >= 16,
{
    let result = unsafe { Slab::from_raw_parts(addr, len, block_size) };
    if let Ok(mut slab) = result {
        let alloc1 = slab.allocate();
        if let Ok(addr1) = alloc1 {
            let alloc2 = slab.allocate();
            if let Ok(addr2) = alloc2 {
                proof {
                    let idx1 = slab@.addr_to_block_idx(addr1 as int);
                    let idx2 = slab@.addr_to_block_idx(addr2 as int);
                    // Both blocks are allocated.
                    assert(slab@.is_allocated(idx1));
                    assert(slab@.is_allocated(idx2));
                }

                // Deallocate block 1.
                let dealloc = slab.deallocate(addr1);
                if let Ok(()) = dealloc {
                    proof {
                        let idx1 = slab@.addr_to_block_idx(addr1 as int);
                        let idx2 = slab@.addr_to_block_idx(addr2 as int);
                        // Block 1 is now free.
                        assert(!slab@.is_allocated(idx1));
                        // Block 2 should still be allocated (this is the key property).
                        assert(slab@.is_allocated(idx2));
                    }
                }
            }
        }
    }
}


/// Test: Fresh slab has all data blocks free.
fn test_fresh_slab_all_free_verified(addr: usize, len: usize, block_size: usize)
    requires
        len > 0,
        len < i32::MAX as usize,
        block_size > 0,
        block_size < i32::MAX as usize,
        block_size <= len,
        Slab::spec_is_power_of_two(block_size as int),
        addr % block_size == 0,
        addr > 0,
        (addr as int) + (len as int) <= (usize::MAX as int),
        (len / block_size) % (u8::BITS as usize) == 0,
        len / block_size >= 8,
{
    let result = unsafe { Slab::from_raw_parts(addr, len, block_size) };
    if let Ok(slab) = result {
        proof {
            // All data blocks should be free in a fresh slab.
            assert(forall|i: int| 0 <= i < slab@.num_data_blocks ==> !slab@.is_allocated(i));
        }
    }
}


/// Test: Invariant about index blocks - they are always marked as used.
/// This test verifies the lemma_index_blocks_always_set property.
fn test_index_blocks_always_used_verified(addr: usize, len: usize, block_size: usize)
    requires
        len > 0,
        len < i32::MAX as usize,
        block_size > 0,
        block_size < i32::MAX as usize,
        block_size <= len,
        Slab::spec_is_power_of_two(block_size as int),
        addr % block_size == 0,
        addr > 0,
        (addr as int) + (len as int) <= (usize::MAX as int),
        (len / block_size) % (u8::BITS as usize) == 0,
        len / block_size >= 8,
{
    let result = unsafe { Slab::from_raw_parts(addr, len, block_size) };
    if let Ok(mut slab) = result {
        proof {
            // The invariant guarantees index blocks are always marked used.
            slab.lemma_index_blocks_always_set();
        }

        // After allocation, index blocks remain used (invariant preserved).
        let alloc = slab.allocate();
        if let Ok(_) = alloc {
            proof {
                // Invariant still holds after allocation.
                slab.lemma_index_blocks_always_set();
            }
        }
    }
}

} // verus!
