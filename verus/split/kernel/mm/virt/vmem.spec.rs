// Copyright(c) The Maintainers of Nanvix.
// Licensed under the MIT License.

// Specifications.

verus! {

//==================================================================================================
// View Types
//==================================================================================================

/// Abstract view of a single page mapping entry.
///
/// # Description
///
/// Uses abstract types (`int` instead of `usize`) per the spec methodology guidelines.
/// This decouples the specification from implementation-specific type sizes.
#[verifier::ext_equal]
pub ghost struct PageMappingView {
    /// Virtual address (page-aligned).
    pub vaddr: int,
    /// Frame address (page-aligned).
    pub frame_addr: int,
    /// Whether this entry is valid/in-use.
    pub valid: bool,
}

/// Abstract view of a virtual memory space.
///
/// # Description
///
/// Provides an abstract representation of `Vmem` using abstract types.
/// `mappings` is a `Seq<PageMappingView>` abstracting the fixed-size array,
/// and `mapping_count` is `int` abstracting `usize`.
#[verifier::ext_equal]
pub ghost struct VmemView {
    /// Number of valid mappings.
    pub mapping_count: int,
    /// Abstract sequence of page mappings.
    pub mappings: Seq<PageMappingView>,
}

//==================================================================================================
// View Implementations
//==================================================================================================

impl View for PageMapping {
    type V = PageMappingView;

    /// View maps the concrete PageMapping to the abstract PageMappingView.
    closed spec fn view(&self) -> PageMappingView {
        PageMappingView {
            vaddr: self.vaddr as int,
            frame_addr: self.frame_addr as int,
            valid: self.valid,
        }
    }
}

impl View for Vmem {
    type V = VmemView;

    /// View maps the concrete Vmem to the abstract VmemView.
    closed spec fn view(&self) -> VmemView {
        VmemView {
            mapping_count: self.mapping_count as int,
            mappings: Seq::new(MAX_USER_PAGES as nat, |i: int| self.mappings[i]@),
        }
    }
}

//==================================================================================================
// PageMappingView Specifications
//==================================================================================================

impl PageMappingView {
    /// Spec function to check if this mapping is for the given vaddr.
    pub open spec fn spec_is_for_vaddr(&self, vaddr: int) -> bool {
        self.valid && self.vaddr == vaddr
    }
}

//==================================================================================================
// VmemView Specifications
//==================================================================================================

impl VmemView {
    /// Spec function to check if a vaddr is mapped (exists in some slot).
    pub open spec fn spec_is_mapped(&self, vaddr: int) -> bool {
        exists|i: int|
            #![trigger self.mappings[i]]
            0 <= i < self.mapping_count &&
            self.mappings[i].spec_is_for_vaddr(vaddr)
    }


    /// Spec function to get the frame address for a mapped virtual address.
    ///
    /// # Description
    ///
    /// Returns the physical frame address backing a given virtual address.
    /// Returns 0 if not found (unreachable when recommends is satisfied).
    pub open spec fn spec_get_frame_addr(&self, vaddr: int) -> int
        recommends self.spec_is_mapped(vaddr)
    {
        if exists|i: int|
            #![trigger self.mappings[i]]
            0 <= i < self.mapping_count &&
            self.mappings[i].spec_is_for_vaddr(vaddr)
        {
            let idx: int = choose|i: int|
                #![trigger self.mappings[i]]
                0 <= i < self.mapping_count &&
                self.mappings[i].spec_is_for_vaddr(vaddr);
            self.mappings[idx].frame_addr
        } else {
            0
        }
    }


    /// Spec function to check if a page at vaddr is mapped.
    pub open spec fn spec_page_is_mapped(&self, vaddr: int) -> bool {
        let page_addr: int = (vaddr / PAGE_SIZE as int) * PAGE_SIZE as int;
        exists|i: int|
            #![trigger self.mappings[i]]
            0 <= i < self.mapping_count &&
            self.mappings[i].vaddr == page_addr
    }


    /// Spec function to check if all pages in a user region are mapped.
    ///
    /// # Approximation
    ///
    /// Checks page-aligned boundaries within [start, start+size).
    pub open spec fn spec_user_region_is_mapped(&self, start: int, size: int) -> bool
        recommends spec_is_user_region(start, size)
    {
        forall|offset: int|
            #![trigger self.spec_page_is_mapped(start + offset)]
            0 <= offset < size && (start + offset) % PAGE_SIZE as int == 0 ==>
            self.spec_page_is_mapped(start + offset)
    }


    /// Spec function to check if there is capacity for more mappings.
    pub open spec fn has_mapping_capacity(&self) -> bool {
        self.mapping_count < MAX_USER_PAGES as int
    }


    /// Spec function to check if there are any mappings.
    pub open spec fn has_mappings(&self) -> bool {
        self.mapping_count > 0
    }
}

//==================================================================================================
// PageMapping Specifications
//==================================================================================================

impl PageMapping {
    /// Invariant for a single page mapping entry.
    ///
    /// # Description
    ///
    /// When the entry is valid, both addresses must be page-aligned and the
    /// virtual address must lie in user space. Invalid entries are trivially
    /// well-formed.
    pub closed spec fn inv(&self) -> bool {
        self.valid ==> {
            &&& self.vaddr as int % PAGE_SIZE as int == 0
            &&& self.frame_addr as int % FRAME_SIZE as int == 0
            &&& spec_is_user_addr(self.vaddr as int)
        }
    }

    /// Private helper to check if this mapping is for the given vaddr.
    /// Used in internal loop invariants. The public version is on `PageMappingView`.
    spec fn spec_is_for_vaddr(&self, vaddr: int) -> bool {
        self.valid && self.vaddr as int == vaddr
    }
}

//==================================================================================================
// Free-Standing Spec Functions
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
pub open spec fn spec_is_kernel_addr(vaddr: int) -> bool {
    !spec_is_user_addr(vaddr)
}


/// Property: A memory region lies entirely in user space.
///
/// # Requirements
///
/// - `size > 0`: Zero-length regions return false.
/// - No overflow: `start + size - 1 >= start`.
/// - Both endpoints must be in user space.
pub open spec fn spec_is_user_region(start: int, size: int) -> bool {
    &&& size > 0
    &&& start >= 0
    &&& start + size - 1 >= start  // No overflow.
    &&& spec_is_user_addr(start)
    &&& spec_is_user_addr(start + size - 1)
}


/// Property: A memory region lies entirely in kernel space.
///
/// # Requirements
///
/// - `size > 0`: Zero-length regions return false.
/// - No overflow: `start + size - 1 >= start`.
/// - Both endpoints must be in kernel space.
pub open spec fn spec_is_kernel_region(start: int, size: int) -> bool {
    &&& size > 0
    &&& start >= 0
    &&& start + size - 1 >= start  // No overflow.
    &&& spec_is_kernel_addr(start)
    &&& spec_is_kernel_addr(start + size - 1)
}


/// Property: A memory region lies within physical memory bounds.
pub open spec fn spec_is_physical_region(start: int, size: int) -> bool {
    &&& size > 0
    &&& start >= 0
    &&& start + size - 1 >= start  // No overflow.
    &&& start < MEMORY_SIZE as int
    &&& start + size - 1 < MEMORY_SIZE as int
}

//==================================================================================================
// Vmem Specifications (internal helpers and invariant)
//==================================================================================================

impl Vmem {
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


    /// Private helper: check if a vaddr is mapped.
    /// Used in internal loop invariants. The public version is on `VmemView`.
    spec fn spec_is_mapped(&self, vaddr: int) -> bool {
        exists|i: int|
            #![trigger self.mappings[i]]
            0 <= i < self.mapping_count as int &&
            self.mappings[i as int].spec_is_for_vaddr(vaddr)
    }


    /// Private helper: get the frame address for a mapped vaddr.
    /// Used in internal proofs. The public version is on `VmemView`.
    spec fn spec_get_frame_addr(&self, vaddr: int) -> int
        recommends self.spec_is_mapped(vaddr)
    {
        if exists|i: int|
            #![trigger self.mappings[i]]
            0 <= i < self.mapping_count as int &&
            self.mappings[i as int].spec_is_for_vaddr(vaddr)
        {
            let idx: int = choose|i: int|
                #![trigger self.mappings[i]]
                0 <= i < self.mapping_count as int &&
                self.mappings[i as int].spec_is_for_vaddr(vaddr);
            self.mappings[idx].frame_addr as int
        } else {
            0
        }
    }
}

} // verus!
