// Copyright(c) The Maintainers of Nanvix.
// Licensed under the MIT License.

//==================================================================================================
// Slab Allocator (Trusted Dependency)
//==================================================================================================
//!
//! This module provides a trusted abstraction of the Slab allocator for use in Kheap verification.
//! The Slab implementation is verified separately in ../slab/slab_core.rs.
//! Here we provide the minimal spec needed for Kheap verification.

use vstd::prelude::*;
use crate::error::Error;

verus! {

/// Abstract view of a Slab allocator.
#[verifier::ext_equal]
pub ghost struct SlabView {
    /// Set of allocated block indices.
    pub allocated_blocks: Set<int>,
    /// Total number of data blocks.
    pub num_data_blocks: int,
    /// Block size in bytes.
    pub block_size: int,
    /// Base address of data region.
    pub data_addr: int,
}

impl SlabView {
    /// Returns the number of allocated blocks.
    pub open spec fn num_allocated(&self) -> int {
        self.allocated_blocks.len() as int
    }

    /// Returns the number of used blocks.
    pub open spec fn used(&self) -> int {
        self.num_allocated()
    }

    /// Returns the capacity (total number of data blocks).
    pub open spec fn capacity(&self) -> int {
        self.num_data_blocks
    }

    /// Returns the number of free blocks.
    pub open spec fn free(&self) -> int {
        self.capacity() - self.used()
    }

    /// Returns true if block at given index is allocated.
    pub open spec fn is_allocated(&self, block_idx: int) -> bool {
        self.allocated_blocks.contains(block_idx)
    }

    /// Returns true if the slab is full.
    pub open spec fn is_full(&self) -> bool {
        self.num_allocated() == self.num_data_blocks
    }

    /// Returns true if the slab is empty.
    pub open spec fn is_empty(&self) -> bool {
        self.allocated_blocks.len() == 0
    }

    /// Returns the address of a block given its index.
    pub open spec fn block_addr(&self, block_idx: int) -> int {
        self.data_addr + block_idx * self.block_size
    }

    /// Returns true if there's free capacity for allocation.
    pub open spec fn can_allocate(&self) -> bool {
        self.free() > 0
    }

    /// Returns true if the address is valid for this slab.
    pub open spec fn is_valid_addr(&self, addr: int) -> bool {
        &&& addr >= self.data_addr
        &&& addr < self.data_addr + self.num_data_blocks * self.block_size
        &&& (addr - self.data_addr) % self.block_size == 0
    }

    /// Returns true if the slab's data region is properly aligned.
    ///
    /// # Description
    ///
    /// When data_addr is aligned to block_size, every block is naturally aligned
    /// because block_addr(i) = data_addr + i * block_size.
    pub open spec fn is_aligned(&self) -> bool {
        self.data_addr % self.block_size == 0
    }
}

/// Slab allocator for managing fixed-size block allocation.
#[derive(Debug)]
pub struct Slab {
    /// Block size in bytes.
    block_size: usize,
    /// Number of data blocks.
    num_data_blocks: usize,
    /// Base address of data region.
    data_addr: usize,
    /// Dummy field for external body.
    _phantom: u8,
}

impl View for Slab {
    type V = SlabView;
    uninterp spec fn view(&self) -> SlabView;
}

impl Slab {
    /// Invariant for the Slab allocator.
    ///
    /// # Properties
    ///
    /// - Block size and data address are positive
    /// - Abstract view matches concrete fields
    /// - Data address is aligned to block size (alignment guarantee)
    pub closed spec fn inv(&self) -> bool {
        &&& self.block_size > 0
        &&& self.num_data_blocks > 0
        &&& self.data_addr > 0
        &&& self@.block_size == self.block_size as int
        &&& self@.num_data_blocks == self.num_data_blocks as int
        &&& self@.data_addr == self.data_addr as int
        // Alignment: data_addr is aligned to block_size.
        // This ensures all blocks are naturally aligned.
        &&& self.data_addr as int % self.block_size as int == 0
    }

    /// Lemma: Invariant implies positive capacity.
    ///
    /// This lemma reveals the `num_data_blocks > 0` property that is
    /// part of the closed `inv()` spec. Useful for clients that need
    /// to reason about capacity without knowing inv() internals.
    #[verifier::external_body]
    pub proof fn lemma_inv_implies_positive_capacity(&self)
        requires
            self.inv(),
        ensures
            self@.num_data_blocks > 0,
    {
        // Follows from inv() definition: self.num_data_blocks > 0
        // and self@.num_data_blocks == self.num_data_blocks as int.
    }

    /// Returns the block size.
    pub closed spec fn spec_block_size(&self) -> int {
        self.block_size as int
    }

    /// Returns the data address.
    pub closed spec fn spec_data_addr(&self) -> int {
        self.data_addr as int
    }

    /// Creates a new Slab from raw memory at a specific offset.
    ///
    /// # Description
    ///
    /// Creates a slab at address `base_addr + offset * slab_size`.
    /// This is used by Kheap to create 8 slabs at consecutive memory regions.
    ///
    /// # Safety
    ///
    /// - The memory region must be valid and writable.
    /// - Memory must be properly aligned for block_size.
    ///
    /// # Postcondition
    ///
    /// The slab's data region is within [base_addr + offset * slab_size, base_addr + (offset+1) * slab_size).
    #[verifier::external_body]
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
            // No overflow.
            (base_addr as int) + ((offset as int) + 1) * (slab_size as int) <= (usize::MAX as int),
        ensures
            result is Ok ==> {
                let slab = result->Ok_0;
                &&& slab.inv()
                &&& slab@.block_size == block_size as int
                &&& slab@.num_data_blocks > 0
                &&& forall|i: int| 0 <= i < slab@.num_data_blocks ==> !slab@.is_allocated(i)
                // The slab starts empty.
                &&& slab@.is_empty()
                // Critical: data region is within the assigned slice.
                &&& slab@.data_addr >= (base_addr as int) + (offset as int) * (slab_size as int)
                &&& slab@.data_addr + slab@.num_data_blocks * slab@.block_size
                    <= (base_addr as int) + ((offset as int) + 1) * (slab_size as int)
                // Alignment: data_addr is aligned to block_size.
                &&& slab@.is_aligned()
            },
    {
        unimplemented!()
    }

    /// Creates a new Slab from raw memory.
    ///
    /// # Safety
    ///
    /// - `addr` must point to valid writable memory of at least `len` bytes.
    /// - Memory must be properly aligned for block_size.
    #[verifier::external_body]
    pub unsafe fn from_raw_parts(addr: *mut u8, len: usize, block_size: usize) -> (result: Result<Slab, Error>)
        requires
            len > 0,
            len < i32::MAX as usize,
            block_size > 0,
            block_size < i32::MAX as usize,
            block_size <= len,
            addr as usize > 0,
            (addr as usize as int) + (len as int) <= (usize::MAX as int),
        ensures
            result is Ok ==> {
                let slab = result->Ok_0;
                &&& slab.inv()
                &&& slab@.block_size == block_size as int
                &&& slab@.num_data_blocks > 0
                &&& forall|i: int| 0 <= i < slab@.num_data_blocks ==> !slab@.is_allocated(i)
            },
    {
        unimplemented!()
    }

    /// Allocates a block from the slab.
    #[verifier::external_body]
    pub unsafe fn allocate(&mut self) -> (result: Result<*mut u8, Error>)
        requires
            old(self).inv(),
        ensures
            self.inv(),
            result is Ok ==> {
                let ptr = result->Ok_0;
                let addr = ptr as usize as int;
                let block_idx = self@.data_addr_to_block_idx(addr);
                // Address is valid in the post-state slab.
                &&& self@.is_valid_addr(addr)
                // Block index is valid.
                &&& 0 <= block_idx < self@.num_data_blocks
                // Block was not allocated before.
                &&& !old(self)@.is_allocated(block_idx)
                // Block is now allocated.
                &&& self@.is_allocated(block_idx)
                // Slab metadata is unchanged.
                &&& self@.num_data_blocks == old(self)@.num_data_blocks
                &&& self@.block_size == old(self)@.block_size
                &&& self@.data_addr == old(self)@.data_addr
                // Frame: other blocks unchanged.
                &&& forall|i: int| 0 <= i < self@.num_data_blocks && i != block_idx ==>
                    self@.is_allocated(i) == old(self)@.is_allocated(i)
                // Alignment: address is aligned to block_size.
                &&& addr % self@.block_size == 0
            },
            result is Err ==> self@ == old(self)@,
            old(self)@.can_allocate() ==> result is Ok,
            result is Err ==> old(self)@.is_full(),
    {
        unimplemented!()
    }

    /// Deallocates a block from the slab.
    #[verifier::external_body]
    pub unsafe fn deallocate(&mut self, ptr: *mut u8) -> (result: Result<(), Error>)
        requires
            old(self).inv(),
            ptr as usize > 0,
            old(self)@.is_valid_addr(ptr as usize as int),
            old(self)@.is_allocated(old(self)@.data_addr_to_block_idx(ptr as usize as int)),
        ensures
            self.inv(),
            result is Ok ==> {
                let addr = ptr as usize as int;
                let block_idx = old(self)@.data_addr_to_block_idx(addr);
                &&& !self@.is_allocated(block_idx)
                &&& self@.num_data_blocks == old(self)@.num_data_blocks
                &&& self@.block_size == old(self)@.block_size
                &&& self@.data_addr == old(self)@.data_addr
                &&& forall|i: int| 0 <= i < self@.num_data_blocks && i != block_idx ==>
                    self@.is_allocated(i) == old(self)@.is_allocated(i)
            },
            result is Err ==> self@ == old(self)@,
    {
        unimplemented!()
    }
}

impl SlabView {
    /// Helper: convert data address to block index.
    pub open spec fn data_addr_to_block_idx(&self, addr: int) -> int {
        (addr - self.data_addr) / self.block_size
    }
}

} // verus!
