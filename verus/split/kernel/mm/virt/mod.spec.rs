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


/// Spec function for computing the inclusive end address used in the original loop.
///
/// # Description
///
/// The original init loop computes `end = raw_vaddr + (region.size() - 1)` and loops
/// `while raw_vaddr < end`. This function models that inclusive bound.
///
/// # Loop Bound Semantics
///
/// For a region of size S starting at addr A:
/// - `end = A + (S - 1)` (inclusive last byte)
/// - The loop `while raw_vaddr < end` iterates for pages at A, A+PAGE_SIZE, ..., up to
///   but NOT including the page containing `end`.
/// - For a single-page region (S == PAGE_SIZE): `end = A + PAGE_SIZE - 1 = A + 4095`.
///   Since `A < A + 4095`, the loop body executes once (mapping the single page).
/// - For S == 0: would underflow, but size > 0 is a precondition.
///
/// The spec's `spec_page_count(S) = S / PAGE_SIZE` matches the iteration count because:
/// - The loop maps pages at indices 0, 1, ..., (S/PAGE_SIZE - 1).
/// - The last page addr is `A + (S/PAGE_SIZE - 1) * PAGE_SIZE = A + S - PAGE_SIZE`.
/// - The loop condition `A + S - PAGE_SIZE < A + S - 1` holds when PAGE_SIZE >= 2 (true).
pub open spec fn spec_loop_end(start: int, size: int) -> int {
    start + (size - 1)
}


/// Spec function modeling MMIO physical address translation.
///
/// # Description
///
/// For MMIO regions, the physical address is obtained from
/// `PhysicalAddress::from_mmio_address(region.start())`. This function is called
/// with the region's START address on every iteration (not the current vaddr),
/// meaning all pages in an MMIO region map to the SAME physical frame.
///
/// This is an opaque hardware-dependent translation. The spec models
/// it as an uninterpreted function from region start to physical address.
pub open spec fn spec_mmio_paddr(region_start: int) -> int;


/// Spec function for the physical address of a page during init.
///
/// # Description
///
/// For non-MMIO regions: paddr == vaddr (identity mapping).
/// For MMIO regions: paddr == spec_mmio_paddr(region_start) for ALL pages.
pub open spec fn spec_init_paddr(vaddr: int, region_start: int, is_mmio: bool) -> int {
    if is_mmio {
        spec_mmio_paddr(region_start)
    } else {
        vaddr
    }
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
        // No overflow: start + size - 1 fits in usize.
        &&& self.start as int + self.size as int - 1 <= usize::MAX as int
    }

    /// Checks if two regions are non-overlapping with self before other.
    pub open spec fn spec_before(&self, other: &MemRegion) -> bool {
        self.spec_end() <= other.spec_start()
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
    /// The page table base addresses in the output (unique, sorted).
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

    /// Property: no duplicate page table bases (strictly increasing).
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

/// Number of entries in a page table (PAGE_SIZE / sizeof(u32) = 1024).
pub open spec fn spec_pgtab_entry_count() -> int {
    (INIT_PAGE_SIZE as int) / 4  // sizeof::<u32>() == 4
}


impl PageTableStorage {
    /// Spec function for the number of entries in the page table storage.
    pub open spec fn spec_entry_count(&self) -> int {
        spec_pgtab_entry_count()
    }

    /// Spec function for the length of the deref'd slice.
    pub open spec fn spec_deref_len(&self) -> int {
        spec_pgtab_entry_count()
    }
}

//==================================================================================================
// Page Table Decision Spec
//==================================================================================================

/// Enumeration of page table assignment decisions.
///
/// # Description
///
/// Models the three-way branching in the original init loop:
/// - `Reuse`: current pgtab_base == last pgtab_base (Ordering::Equal)
/// - `CreateNew`: current pgtab_base > last pgtab_base (Ordering::Greater)
/// - `Error`: current pgtab_base < last pgtab_base (Ordering::Less, overlapping regions)
pub enum PgtabDecision {
    /// Reuse the existing page table (bases are equal).
    Reuse,
    /// Create a new page table (current base is greater).
    CreateNew,
    /// Error: overlapping memory regions (current base is less).
    Overlap,
}

impl PgtabDecision {
    /// Spec: the decision is valid (not an overlap error).
    pub open spec fn spec_is_ok(&self) -> bool {
        !matches!(*self, PgtabDecision::Overlap)
    }
}

//==================================================================================================
// Init Loop Invariant Spec
//==================================================================================================

/// Spec for the init inner loop invariant.
///
/// # Description
///
/// Captures the state maintained across iterations of the inner page-mapping loop.
/// The invariant ensures that the page table list remains sorted and aligned,
/// and that the last base address is consistent with the current virtual address.
pub open spec fn spec_init_loop_inv(
    pgtab_bases: Seq<int>,
    last_base: int,
    vaddr: int,
) -> bool {
    // The last base is the pgtab base for the current vaddr.
    &&& last_base == spec_pgtab_base(vaddr)
    // The last base is aligned.
    &&& last_base % INIT_PGTAB_ALIGNMENT as int == 0
    // All existing bases are aligned.
    &&& forall|i: int|
            #![trigger pgtab_bases[i]]
            0 <= i < pgtab_bases.len() ==>
            pgtab_bases[i] % INIT_PGTAB_ALIGNMENT as int == 0
    // All existing bases are <= the last base.
    &&& forall|i: int|
            #![trigger pgtab_bases[i]]
            0 <= i < pgtab_bases.len() ==>
            pgtab_bases[i] <= last_base
}

} // verus!
