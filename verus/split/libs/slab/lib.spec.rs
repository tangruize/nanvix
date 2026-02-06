// Copyright(c) The Maintainers of Nanvix.
// Licensed under the MIT License.

// Specifications.

verus! {


impl SlabView {
    /// Returns the number of allocated blocks.
    pub open spec fn used(&self) -> int {
        self.allocated_blocks.len() as int
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
        self.used() == self.num_data_blocks
    }

    /// Returns true if the slab is empty.
    pub open spec fn is_empty(&self) -> bool {
        self.allocated_blocks.len() == 0
    }

    /// Returns the address of a block given its index.
    pub open spec fn block_addr(&self, block_idx: int) -> int {
        self.data_addr + block_idx * self.block_size
    }

    /// Returns the block index for a given address.
    pub open spec fn addr_to_block_idx(&self, addr: int) -> int {
        (addr - self.data_addr) / self.block_size
    }

    /// Returns true if the data address is aligned to block size.
    pub open spec fn is_aligned(&self) -> bool {
        self.data_addr % self.block_size == 0
    }

    /// Returns true if the address is valid for this slab.
    pub open spec fn is_valid_addr(&self, addr: int) -> bool {
        &&& addr >= self.data_addr
        &&& addr < self.data_addr + self.num_data_blocks * self.block_size
        &&& (addr - self.data_addr) % self.block_size == 0
    }

    /// Returns true if an address is within the overall buffer.
    pub open spec fn is_within_buffer(&self, addr: int) -> bool {
        addr >= self.base_addr && addr < self.base_addr + self.total_len
    }

    //==============================================================================================
    // High-Level Memory Management Properties
    //==============================================================================================

    /// Property: All allocated block indices are within valid range.
    /// Issue 9 FIX: Added explicit trigger for quantifier.
    pub open spec fn allocated_blocks_in_range(&self) -> bool {
        forall|i: int|
            #![trigger self.is_allocated(i)]
            self.is_allocated(i) ==> (0 <= i < self.num_data_blocks)
    }

    /// Property: Memory regions of different blocks are disjoint.
    /// Two blocks with different indices have non-overlapping memory regions.
    pub open spec fn blocks_are_disjoint(&self, i: int, j: int) -> bool
        recommends 0 <= i < self.num_data_blocks, 0 <= j < self.num_data_blocks, i != j
    {
        let addr_i = self.block_addr(i);
        let addr_j = self.block_addr(j);
        // Block i's region [addr_i, addr_i + block_size) does not overlap with block j's region.
        addr_i + self.block_size <= addr_j || addr_j + self.block_size <= addr_i
    }

    /// Property: All allocated blocks have disjoint memory regions (no aliasing).
    /// Issue 9 FIX: Added explicit trigger for quantifier.
    pub open spec fn no_memory_aliasing(&self) -> bool {
        forall|i: int, j: int|
            #![trigger self.is_allocated(i), self.is_allocated(j)]
            (self.is_allocated(i) && self.is_allocated(j) && i != j) ==>
            self.blocks_are_disjoint(i, j)
    }

    /// Property: Inverse relationship - addr_to_block_idx(block_addr(i)) == i for valid i.
    pub open spec fn addr_block_idx_inverse(&self, i: int) -> bool
        recommends 0 <= i < self.num_data_blocks, self.block_size > 0
    {
        self.addr_to_block_idx(self.block_addr(i)) == i
    }

    /// Property: Inverse relationship - block_addr(addr_to_block_idx(a)) == a for valid addresses.
    pub open spec fn block_addr_inverse(&self, addr: int) -> bool
        recommends self.is_valid_addr(addr), self.block_size > 0
    {
        self.block_addr(self.addr_to_block_idx(addr)) == addr
    }

    //==============================================================================================
    // Liveness Properties
    //==============================================================================================

    /// Property (Liveness): If there's free capacity, allocation can succeed.
    /// This ensures the allocator is "live" - it can make progress when resources are available.
    pub open spec fn can_allocate(&self) -> bool {
        self.free() > 0
    }

    /// Property (Liveness): If a block is allocated, it can be deallocated.
    pub open spec fn can_deallocate(&self, block_idx: int) -> bool {
        self.is_allocated(block_idx) && 0 <= block_idx < self.num_data_blocks
    }

    /// Property (Liveness): After deallocation, allocation becomes possible.
    /// This is crucial for allocator liveness - freed resources become available.
    pub open spec fn dealloc_enables_alloc(&self, freed_view: &SlabView) -> bool
        recommends self.used() == self.capacity()  // self is full
    {
        // If we were full and deallocated one block, now we can allocate.
        freed_view.used() < freed_view.capacity()
    }

    //==============================================================================================
    // Memory Initialization Properties
    //==============================================================================================

    /// Property: A freshly initialized slab has no allocated data blocks.
    /// This is the initial condition for a new slab.
    pub open spec fn is_freshly_initialized(&self) -> bool {
        self.allocated_blocks =~= Set::<int>::empty()
    }

    //==============================================================================================
}


impl View for Slab {
    type V = SlabView;

    closed spec fn view(&self) -> SlabView {
        // Performance optimization: directly use bitmap's set_bits with offset.
        // Instead of is_bit_set(num_index_blocks + i), we use:
        // set_bits.contains(num_index_blocks + i).
        // The bounds check is implicit: by slab invariant, all set bits in the
        // data block range [num_index_blocks, num_index_blocks + num_data_blocks)
        // correspond to allocated data blocks.
        let offset: int = self.num_index_blocks as int;
        SlabView {
            allocated_blocks: Set::new(|i: int|
                0 <= i < self.num_data_blocks as int &&
                self.index@.set_bits.contains(offset + i)
            ),
            num_data_blocks: self.num_data_blocks as int,
            block_size: self.block_size as int,
            data_addr: self.data_addr as int,
            base_addr: self.base_addr as int,
            total_len: self.total_len as int,
        }
    }
}

impl Slab {
    //==============================================================================================

    /// Invariant for the slab allocator.
    pub closed spec fn inv(&self) -> bool {
        &&& self.index.inv()
        &&& self.block_size > 0
        &&& self.num_data_blocks > 0
        &&& self.num_index_blocks > 0
        &&& self.num_index_blocks + self.num_data_blocks == self.index@.number_of_bits()
        // Index blocks are always marked as allocated in the bitmap.
        // Performance: use set_bits.contains directly instead of is_bit_set.
        &&& forall|i: int| #![trigger self.index@.set_bits.contains(i)]
            0 <= i < self.num_index_blocks as int ==> self.index@.set_bits.contains(i)
        // Data block indices start after index blocks.
        &&& self.data_addr > 0
        // Memory region bounds - ensures no overflow in address calculations.
        // Total size of data region fits in usize.
        &&& (self.num_data_blocks as int) * (self.block_size as int) <= usize::MAX as int
        // data_addr + total data size fits in usize (no overflow when computing addresses).
        &&& (self.data_addr as int) + (self.num_data_blocks as int) * (self.block_size as int) <= usize::MAX as int
        // Issue 2 FIX: Metadata/Data disjointness - index region ends before data region starts.
        // index_end = base_addr + num_index_blocks * block_size (where index is stored).
        // data_addr >= index_end (data starts at or after index region).
        // Since data_addr = base_addr + num_index_blocks * block_size in from_raw_parts,
        // this is ensured by construction. The invariant captures that data_addr is correctly computed.
        &&& self.data_addr as int >= self.num_index_blocks as int * self.block_size as int
        // Issue 3 FIX: Power-of-two and alignment requirements.
        // Block size must be a power of two (required for correct address arithmetic).
        &&& Self::spec_is_power_of_two(self.block_size as int)
        // Data address must be aligned to block size (required for aligned allocations).
        &&& self.data_addr as int % self.block_size as int == 0
        // Issue 6 FIX: Buffer bounds - record and validate the overall slab buffer.
        &&& self.base_addr > 0
        &&& self.total_len > 0
        // base_addr + total_len fits in usize (no overflow).
        &&& (self.base_addr as int) + (self.total_len as int) <= usize::MAX as int
        // data_addr is within the buffer: base_addr <= data_addr.
        &&& self.base_addr as int <= self.data_addr as int
        // All data blocks are within the buffer.
        &&& (self.data_addr as int) + (self.num_data_blocks as int) * (self.block_size as int)
            <= (self.base_addr as int) + (self.total_len as int)
        // Metadata/data disjointness with explicit base_addr.
        &&& self.data_addr as int == self.base_addr as int + self.num_index_blocks as int * self.block_size as int
        // View consistency: connect concrete fields to the view.
        // This is needed because view() is closed.
        &&& self@.num_data_blocks == self.num_data_blocks as int
        &&& self@.block_size == self.block_size as int
        &&& self@.data_addr == self.data_addr as int
        &&& self@.base_addr == self.base_addr as int
        &&& self@.total_len == self.total_len as int
        // Allocated blocks are in range: all allocated indices are valid data block indices.
        // This is true by definition of allocated_blocks in view(), but needs to be explicit
        // because view() is closed.
        &&& self@.allocated_blocks_in_range()
    }


    /// Specification function: checks if a value is a power of two.
    /// Uses recursive definition: 1, 2, 4, 8, 16, ... are powers of two.
    pub open spec fn spec_is_power_of_two(n: int) -> bool
        decreases n,
    {
        if n <= 0 {
            false
        } else if n == 1 {
            true
        } else if n % 2 != 0 {
            false
        } else {
            Self::spec_is_power_of_two(n / 2)
        }
    }

    //==============================================================================================

    /// Property: The index (metadata) region and data region are disjoint.
    /// The index uses bytes [base_addr, base_addr + index_bytes_used).
    /// The data region starts at data_addr = base_addr + num_index_blocks * block_size.
    /// Since num_index_blocks * block_size >= index_bytes_used (rounded up), they don't overlap.
    pub closed spec fn metadata_data_disjoint(&self, base_addr: int, index_bytes: int) -> bool {
        let index_region_end = base_addr + index_bytes;
        let data_region_start = self.data_addr as int;
        // Data region starts at or after index region ends.
        data_region_start >= index_region_end
    }
}

} // verus!
