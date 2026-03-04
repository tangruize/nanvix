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
    raw_array::RawArray,
};
#[cfg(verus_keep_ghost)]
use crate::libs::raw_array::{
    axiom_u8_zero_is_0,
    is_zero,
};
use vstd::{
    prelude::*,
    arithmetic::power2::is_pow2,
    raw_ptr::{
        PointsToRaw,
        Provenance,
    },
    set::*,
    set_lib::{
        lemma_int_range,
        lemma_len_subset,
        lemma_set_subset_finite,
        set_int_range,
    },
};

// Include specifications.
#[cfg(verus_keep_ghost)]
include!("lib.spec.rs");

// Include proofs.
#[cfg(verus_keep_ghost)]
include!("lib.proof.rs");

// Include verified tests.
#[cfg(verus_keep_ghost)]
include!("lib.test.rs");

//==================================================================================================
// Structures
//==================================================================================================

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
    /// Base address of data blocks.
    data_addr: *mut u8,
    /// Number of index blocks in the slab.
    num_index_blocks: usize,
    /// Number of data blocks in the slab.
    num_data_blocks: usize,
    /// Size of blocks in the slab.
    block_size: usize,
}

//==================================================================================================
// View Implementation for Slab
//==================================================================================================

#[cfg(verus_keep_ghost)]
impl View for Slab {
    type V = SlabView;

    closed spec fn view(&self) -> SlabView {
        let offset: int = self.num_index_blocks as int;
        SlabView {
            allocated_blocks: Set::new(|i: int|
                0 <= i < self.num_data_blocks as int &&
                self.index@.set_bits.contains(offset + i)
            ),
            num_data_blocks: self.num_data_blocks as int,
            block_size: self.block_size as int,
            data_addr: self.data_addr as int,
        }
    }
}

#[cfg(verus_keep_ghost)]
impl Slab {
    /// Invariant for the slab allocator.
    pub closed spec fn inv(&self) -> bool {
        &&& self.index.inv()
        &&& self.block_size > 0
        &&& self.num_data_blocks > 0
        &&& self.num_index_blocks > 0
        &&& self.num_index_blocks + self.num_data_blocks == self.index@.number_of_bits()
        &&& forall|i: int| #![trigger self.index@.set_bits.contains(i)]
            0 <= i < self.num_index_blocks as int ==> self.index@.set_bits.contains(i)
        &&& self.data_addr as int > 0
        &&& (self.num_data_blocks as int) * (self.block_size as int) <= usize::MAX as int
        &&& (self.data_addr as int) + (self.num_data_blocks as int) * (self.block_size as int) <= usize::MAX as int
        &&& is_pow2(self.block_size as int)
        &&& self.data_addr as int % self.block_size as int == 0
        &&& self.data_addr as int >= self.num_index_blocks as int * self.block_size as int
        &&& self@.num_data_blocks == self.num_data_blocks as int
        &&& self@.block_size == self.block_size as int
        &&& self@.data_addr == self.data_addr as int
        &&& self@.allocated_blocks_in_range()
    }
}

//==================================================================================================
// Implementations
//==================================================================================================

impl Slab {

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
            is_pow2(block_size as int),
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
        if block_size & (block_size - 1) != 0 {
            return Err(Error::new(ErrorCode::InvalidArgument, "block size is not a power of two"));
        }

        proof {
            Self::lemma_bitwise_implies_is_pow2(block_size);
        }

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

        // Prove layout bounds for slab construction.
        proof {
            Self::lemma_from_raw_parts_layout_bounds(
                len as int, block_size as int, total_num_blocks as int,
                index_len as int, num_index_blocks as int, num_data_blocks as int,
                addr as int,
            );
        }
        let data_addr: *mut u8 = addr.with_addr(addr.addr() + num_index_blocks * block_size);

        // Check if `data_addr` is aligned to `block_size`.
        if !(data_addr as usize).is_multiple_of(block_size) {
            return Err(Error::new(ErrorCode::InvalidArgument, "unaligned data address"));
        }

        // Instantiate index.
        let storage: RawArray<u8> = RawArray::from_raw_parts(addr, index_len)?;

        // Prove that all bytes in storage are zero (required by Bitmap::from_raw_array).
        proof {
            Self::lemma_raw_array_storage_zeroed(storage@);
        }

        let mut index: Bitmap = Bitmap::from_raw_array(storage)?;

        // Prove key invariants before the loop.
        proof {
            Self::lemma_from_raw_parts_pre_loop(
                index@.number_of_bits(), index_len as int, total_num_blocks as int,
                num_index_blocks as int, num_data_blocks as int,
            );
        }

        // NOTE: The index is initialized with all blocks free, thus if we fail beyond this point
        // the memory region is left in a modified state.

        // Initialize index.
        for i in 0..num_index_blocks
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
        {
            index.set(i)?;
        }

        // After the loop, all index blocks are set.
        let result_slab: Slab = Slab {
            num_index_blocks,
            num_data_blocks,
            block_size,
            data_addr,
            index,
        };

        proof {
            Self::lemma_from_raw_parts_post_loop(
                &result_slab, addr as int, len as int, total_num_blocks as int,
            );
        }

        Ok(result_slab)
    }

    ///
    /// # Description
    ///
    /// Allocates a block of memory from the slab allocator.
    ///
    /// # Returns
    ///
    /// Upon success, a pointer to the allocated block is returned. Upon failure, an error is
    /// returned instead.
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
                // Returned address is non-null (derivable from data_addr as int > 0 and is_valid_addr).
                &&& addr > 0
            },
            // Error case: state unchanged and slab was full (no capacity).
            result is Err ==> (self@ == old(self)@ && !old(self)@.can_allocate()),
            // Liveness: if there's free capacity, allocation succeeds.
            old(self)@.can_allocate() ==> result is Ok,
    {
        let block: usize = match self.index.alloc() {
            Ok(b) => b,
            Err(e) => {
                proof {
                    Self::lemma_alloc_error_preserves_state(self, old(self));
                }
                return Err(e);
            }
        };

        proof {
            Self::lemma_alloc_block_is_data_block_with_bounds(
                self, old(self), block as int,
            );
        }

        // Safety: the start and resulting addresses are valid.
        let block_addr: *mut u8 = {
            let block_idx: usize = block - self.num_index_blocks;

            proof {
                Self::lemma_alloc_product_in_bounds(self, block_idx as int);
            }

            self.data_addr.with_addr(self.data_addr.addr() + block_idx * self.block_size)
        };

        proof {
            let block_idx: int = block as int - self.num_index_blocks as int;
            Self::lemma_alloc_establishes_postconditions(
                self, old(self), block as int,
                block_idx, block_addr as int,
            );
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
        // Safety: the start and resulting addresses are valid.
        proof {
            assert((self.data_addr as int) + (self.num_data_blocks as int) * (self.block_size as int) <= usize::MAX as int);
        }
        if (ptr as usize) < (self.data_addr as usize)
            || (ptr as usize) >= (self.data_addr as usize) + self.num_data_blocks * self.block_size
        {
            return Err(Error::new(ErrorCode::BadAddress, "pointer out of bounds"));
        }

        // Compute the block index.
        // Safety: we have already checked that ptr is within the bounds of the slab.

        proof {
            Self::lemma_dealloc_offset_bounds(self, ptr as int);
        }

        let index: usize = self.num_index_blocks + ((ptr as usize) - (self.data_addr as usize)) / self.block_size;

        proof {
            Self::lemma_dealloc_index_is_allocated(self, ptr as int, index as int);
        }

        // Check if the block is already free.
        if !self.index.test(index)? {
            return Err(Error::new(ErrorCode::BadAddress, "block is already free"));
        }

        // Free the block.
        match self.index.clear(index) {
            Ok(()) => {
                proof {
                    Self::lemma_dealloc_clear_ok_postconditions(
                        self, old(self), index as int, ptr as int,
                    );
                }
                Ok(())
            },
            Err(e) => {
                proof {
                    Self::lemma_dealloc_clear_err_preserves_inv(self, old(self));
                }
                Err(e)
            }
        }
    }
}

} // verus!
