// Copyright(c) The Maintainers of Nanvix.
// Licensed under the MIT License.

// Specifications for the virtual memory init module.

verus! {

//==================================================================================================
// Spec Functions for Address Alignment
//==================================================================================================

/// Spec function for aligning an address down to a boundary.
///
/// # Description
///
/// Computes the largest multiple of `alignment` that is <= `addr`.
/// Equivalent to `addr & !(alignment - 1)` when alignment is a power of 2.
pub open spec fn spec_align_down(addr: int, alignment: int) -> int
    recommends alignment > 0
{
    (addr / alignment) * alignment
}


/// Spec function for computing the page table base address.
///
/// # Description
///
/// Returns the page-table-aligned base address for a given virtual address.
/// This determines which page table a virtual address belongs to.
pub open spec fn spec_pgtab_base(vaddr: int) -> int {
    spec_align_down(vaddr, INIT_PGTAB_ALIGNMENT as int)
}


/// Spec function for computing the number of pages in a region.
///
/// # Description
///
/// Returns the number of pages that fit in a region of the given size.
/// Requires page-aligned size.
pub open spec fn spec_page_count(size: int) -> int
    recommends
        size >= 0,
        size % INIT_PAGE_SIZE as int == 0,
{
    size / INIT_PAGE_SIZE as int
}


/// Spec function for computing the i-th page address in a region.
pub open spec fn spec_nth_page_addr(region_start: int, index: int) -> int {
    region_start + index * INIT_PAGE_SIZE as int
}

//==================================================================================================
// MemRegion Spec Functions
//==================================================================================================

impl MemRegion {
    /// Returns the start address of the region.
    pub open spec fn spec_start(&self) -> int {
        self.start as int
    }

    /// Returns the size of the region in bytes.
    pub open spec fn spec_size(&self) -> int {
        self.size as int
    }

    /// Returns whether this is an MMIO region.
    pub open spec fn spec_is_mmio(&self) -> bool {
        self.is_mmio
    }

    /// Checks if the region start and size are page-aligned.
    pub open spec fn spec_is_page_aligned(&self) -> bool {
        self.start as int % INIT_PAGE_SIZE as int == 0
        && self.size as int % INIT_PAGE_SIZE as int == 0
    }

    /// Returns the number of pages in the region.
    pub open spec fn spec_page_count(&self) -> int {
        self.size as int / INIT_PAGE_SIZE as int
    }

    /// Returns the exclusive end address of the region.
    pub open spec fn spec_end(&self) -> int {
        self.start as int + self.size as int
    }

    /// Checks if the region is valid for processing by init.
    pub open spec fn spec_is_valid(&self) -> bool {
        &&& self.size > 0
        &&& self.spec_is_page_aligned()
        &&& self.spec_end() <= usize::MAX as int
    }
}

//==================================================================================================
// View Type for Init Result
//==================================================================================================

/// Abstract view of the virtual memory initialization result.
///
/// # Description
///
/// Captures the essential properties of the init function's output:
/// - A sequence of page table base addresses (ordered, aligned).
/// - The regions that were processed.
#[verifier::ext_equal]
pub struct VirtInitView {
    /// The input regions processed by init.
    pub regions: Seq<MemRegion>,
    /// The page table base addresses in the output.
    pub page_table_bases: Seq<int>,
}

impl VirtInitView {
    /// Property: all input regions are sorted by start address.
    pub open spec fn regions_sorted(&self) -> bool {
        forall|i: int, j: int|
            #![trigger self.regions[i], self.regions[j]]
            0 <= i < j < self.regions.len() as int ==>
            self.regions[i].spec_start() <= self.regions[j].spec_start()
    }

    /// Property: all input regions are valid (positive size, aligned).
    pub open spec fn regions_valid(&self) -> bool {
        forall|i: int|
            #![trigger self.regions[i]]
            0 <= i < self.regions.len() as int ==>
            self.regions[i].spec_is_valid()
    }

    /// Property: output page table bases are in non-decreasing order.
    pub open spec fn page_tables_ordered(&self) -> bool {
        forall|i: int, j: int|
            #![trigger self.page_table_bases[i], self.page_table_bases[j]]
            0 <= i < j < self.page_table_bases.len() as int ==>
            self.page_table_bases[i] <= self.page_table_bases[j]
    }

    /// Property: all page table bases are properly aligned.
    pub open spec fn page_tables_aligned(&self) -> bool {
        forall|i: int|
            #![trigger self.page_table_bases[i]]
            0 <= i < self.page_table_bases.len() as int ==>
            self.page_table_bases[i] % INIT_PGTAB_ALIGNMENT as int == 0
    }

    /// Property: no duplicate page table bases.
    pub open spec fn page_tables_unique(&self) -> bool {
        forall|i: int, j: int|
            #![trigger self.page_table_bases[i], self.page_table_bases[j]]
            0 <= i < j < self.page_table_bases.len() as int ==>
            self.page_table_bases[i] < self.page_table_bases[j]
    }
}

//==================================================================================================
// PageTableStorage Spec Functions
//==================================================================================================

impl PageTableStorage {
    /// Spec function for the number of entries in the page table storage.
    pub open spec fn spec_entry_count(&self) -> int {
        (INIT_PAGE_SIZE as int) / 4  // sizeof::<u32>() == 4
    }
}

} // verus!
