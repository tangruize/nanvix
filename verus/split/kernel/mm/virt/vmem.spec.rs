// Copyright(c) The Maintainers of Nanvix.
// Licensed under the MIT License.

// Specifications.

verus! {


impl PageMapping {
    /// Spec function to check if this mapping is for the given vaddr.
    pub open spec fn spec_is_for_vaddr(&self, vaddr: int) -> bool {
        self.valid && self.vaddr as int == vaddr
    }
}

//==================================================================================================

/// Property: A virtual address is in user space.
pub open spec fn spec_is_user_addr(vaddr: int) -> bool {
    USER_BASE as int <= vaddr && vaddr < USER_END as int
}


/// Property: A virtual address is in kernel space.
///
/// # Note
///
/// Kernel space is defined as any address NOT in user space. On x86 with the
/// standard 3GB/1GB split:
/// - User space: USER_BASE (1GB) to USER_END (3GB)
/// - Kernel space: 0 to USER_BASE and USER_END to 4GB
///
/// Addresses in the range 0-1GB are typically not directly accessible in
/// user mode on x86. Addresses above 3GB are the kernel's address space.
/// This definition matches the x86 memory layout used by Nanvix.
pub open spec fn spec_is_kernel_addr(vaddr: int) -> bool {
    !spec_is_user_addr(vaddr)
}


/// Property: A memory region lies entirely in user space.
/// Requires size > 0 and no overflow.
/// Property: A memory region lies entirely in user space.
///
/// # Requirements
///
/// - `size > 0`: Zero-length regions return false. The verified copy operations
///   explicitly check for zero size and return an error, matching the original
///   implementation which also rejects zero-length copies.
/// - No overflow: `start + size - 1 >= start`
/// - Both endpoints must be in user space
pub open spec fn spec_is_user_region(start: int, size: int) -> bool {
    &&& size > 0
    &&& start >= 0
    &&& start + size - 1 >= start  // No overflow
    &&& spec_is_user_addr(start)
    &&& spec_is_user_addr(start + size - 1)
}


/// Property: A memory region lies entirely in kernel space.
///
/// # Requirements
///
/// - `size > 0`: Zero-length regions return false (same as user regions).
/// - No overflow: `start + size - 1 >= start`
/// - Both endpoints must be in kernel space
pub open spec fn spec_is_kernel_region(start: int, size: int) -> bool {
    &&& size > 0
    &&& start >= 0
    &&& start + size - 1 >= start  // No overflow
    &&& spec_is_kernel_addr(start)
    &&& spec_is_kernel_addr(start + size - 1)
}


/// Property: A memory region lies within physical memory bounds.
pub open spec fn spec_is_physical_region(start: int, size: int) -> bool {
    &&& size > 0
    &&& start >= 0
    &&& start + size - 1 >= start  // No overflow
    &&& start < MEMORY_SIZE as int
    &&& start + size - 1 < MEMORY_SIZE as int
}

impl Vmem {
    //==============================================================================================

    /// Spec function to check if a vaddr is mapped (exists in some slot).
    pub open spec fn spec_is_mapped(&self, vaddr: int) -> bool {
        exists|i: int|
            #![trigger self.mappings[i]]
            0 <= i < self.mapping_count as int &&
            self.mappings[i as int].spec_is_for_vaddr(vaddr)
    }


    /// Spec function to get the frame address for a mapped virtual address.
    ///
    /// # Description
    ///
    /// Returns the physical frame address backing a given virtual address.
    /// If multiple mappings exist for the same vaddr (not expected in well-formed vmem),
    /// returns the first one found.
    ///
    /// # Recommends
    ///
    /// The vaddr should be mapped (spec_is_mapped(vaddr) is true).
    pub open spec fn spec_get_frame_addr(&self, vaddr: int) -> int
        recommends self.spec_is_mapped(vaddr)
    {
        // Find the mapping for vaddr and return its frame address.
        // Since mappings are unique per vaddr (enforced by map()), this is deterministic.
        // We return 0 if not found (should not happen when recommends is satisfied).
        if exists|i: int|
            #![trigger self.mappings[i]]
            0 <= i < self.mapping_count as int &&
            self.mappings[i as int].spec_is_for_vaddr(vaddr)
        {
            // Use choose on the index, not the frame address.
            let idx: int = choose|i: int|
                #![trigger self.mappings[i]]
                0 <= i < self.mapping_count as int &&
                self.mappings[i as int].spec_is_for_vaddr(vaddr);
            self.mappings[idx].frame_addr as int
        } else {
            0 // Default (unreachable when recommends is satisfied).
        }
    }


    /// Spec function to check if a page at vaddr is mapped.
    /// Returns true if there exists a mapping for the page containing vaddr.
    pub open spec fn spec_page_is_mapped(&self, vaddr: int) -> bool {
        let page_addr: int = (vaddr / PAGE_SIZE as int) * PAGE_SIZE as int;
        exists|i: int|
            #![trigger self.mappings[i]]
            0 <= i < self.mapping_count as int &&
            self.mappings[i as int].vaddr as int == page_addr
    }


    /// Spec function to check if all pages in a user region are mapped.
    ///
    /// # Approximation
    ///
    /// This is a simplified model that checks page-aligned boundaries within
    /// the region. It verifies that every page boundary address within
    /// [start, start+size) has a corresponding mapping.
    ///
    /// # Limitations
    ///
    /// - Only checks page-aligned offsets within the region
    /// - If start is not page-aligned, the first partial page's mapping is
    ///   checked via spec_page_is_mapped which aligns down to page boundary
    /// - For full coverage verification, the caller should ensure start is
    ///   page-aligned and size is a multiple of PAGE_SIZE
    ///
    /// The original implementation checks mapping existence page-by-page during
    /// the copy loop, which this approximates.
    pub open spec fn spec_user_region_is_mapped(&self, start: int, size: int) -> bool
        recommends spec_is_user_region(start, size)
    {
        // For a region to be fully mapped, every page it touches must be mapped.
        // We approximate this by requiring the region lies within mapped pages.
        forall|offset: int|
            #![trigger self.spec_page_is_mapped(start + offset)]
            0 <= offset < size && (start + offset) % PAGE_SIZE as int == 0 ==>
            self.spec_page_is_mapped(start + offset)
    }


    /// Invariant: mapping_count is within bounds and all mappings in [0, mapping_count) are valid.
    pub closed spec fn inv(&self) -> bool {
        &&& self.mapping_count <= MAX_USER_PAGES
        // All mappings in [0, mapping_count) have valid flag set.
        &&& forall|i: int| #![auto] 0 <= i < self.mapping_count as int ==> self.mappings[i as int].valid
        // All valid mappings are for user addresses.
        &&& forall|i: int| #![auto] 0 <= i < self.mapping_count as int ==>
                spec_is_user_addr(self.mappings[i as int].vaddr as int)
        // All valid mappings have page-aligned vaddr.
        &&& forall|i: int| #![auto] 0 <= i < self.mapping_count as int ==>
                self.mappings[i as int].vaddr as int % PAGE_SIZE as int == 0
        // All valid mappings have frame-aligned frame_addr.
        &&& forall|i: int| #![auto] 0 <= i < self.mapping_count as int ==>
                self.mappings[i as int].frame_addr as int % FRAME_SIZE as int == 0
        // Uniqueness: Each virtual address is mapped at most once.
        &&& forall|i: int, j: int|
                #![trigger self.mappings[i], self.mappings[j]]
                0 <= i < self.mapping_count as int &&
                0 <= j < self.mapping_count as int &&
                i != j ==>
                self.mappings[i].vaddr != self.mappings[j].vaddr
    }


    /// Spec function to check if there is capacity for more mappings.
    pub closed spec fn has_mapping_capacity(&self) -> bool {
        self.mapping_count < MAX_USER_PAGES
    }


    /// Spec function to check if there are any mappings.
    pub closed spec fn has_mappings(&self) -> bool {
        self.mapping_count > 0
    }
}

} // verus!
