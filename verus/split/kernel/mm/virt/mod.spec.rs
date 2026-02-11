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
pub uninterp spec fn spec_mmio_paddr(region_start: int) -> int;


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
// Page Mapping Record (Ghost State)
//==================================================================================================

/// Record of a single page mapping produced by init.
///
/// # Description
///
/// Captures the mapping parameters for each page mapped during initialization.
/// Exposed as a ghost postcondition of init() so callers can reason about
/// the complete mapping table including permission attributes.
///
/// The permission fields model the original's `page_table.map()` arguments:
/// `present=true, writable=true, user=false` with `AccessPermission::RDWR`.
/// These are fixed for all init mappings (the original has `FIXME: do not
/// be so open about permissions and caching`).
pub struct PageMapping {
    /// Virtual address of the mapped page (page-aligned).
    pub vaddr: int,
    /// Physical address the page is mapped to.
    pub paddr: int,
    /// Start address of the region this page belongs to.
    pub region_start: int,
    /// Whether this page belongs to an MMIO region.
    pub is_mmio: bool,
    /// Page is present in the page table.
    pub present: bool,
    /// Page is writable.
    pub writable: bool,
    /// Page is user-accessible (false for kernel init mappings).
    pub user_accessible: bool,
}

/// Spec: the default permission attributes for init mappings.
///
/// # Description
///
/// The original init() maps all pages with:
/// `present=true, writable=true, user=false, AccessPermission::RDWR`.
/// This spec function checks that a mapping has these fixed attributes.
///
/// The original has `FIXME: do not be so open about permissions and caching`,
/// indicating that these permissions may be tightened in the future. If the
/// original is updated, this spec should be parameterized over a permission
/// policy rather than hardcoded. The `PageMapping` struct already has
/// individual permission fields to support such an extension.
pub open spec fn spec_has_init_permissions(m: PageMapping) -> bool {
    m.present && m.writable && !m.user_accessible
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
// Init Result Type
//==================================================================================================

/// Result type for checked initialization.
///
/// # Description
///
/// Models the `Result<LinkedList<...>, Error>` return type from the original
/// `init()`. The `Ok` variant carries the page table bases and ghost mappings.
/// The `OverlapError` variant models the `Ordering::Less` error path.
pub enum InitResult {
    /// Successful initialization with page table bases and ghost mappings.
    Ok { bases: Vec<usize>, mappings: Ghost<Seq<PageMapping>> },
    /// Error: overlapping memory regions detected at runtime.
    OverlapError,
}

impl InitResult {
    /// Spec: the result is successful.
    pub open spec fn spec_is_ok(&self) -> bool {
        matches!(*self, InitResult::Ok { .. })
    }
}

//==================================================================================================
// Init Mapping Coverage Spec
//==================================================================================================

/// Recursive spec function computing the total number of pages across regions.
///
/// # Description
///
/// Computes the sum of page counts for regions[0..n). Used to prove that
/// init() visits exactly the right number of pages (functional completeness).
pub open spec fn spec_total_pages(regions: Seq<MemRegion>, n: int) -> int
    decreases n,
{
    if n <= 0 {
        0
    } else {
        spec_total_pages(regions, n - 1) + spec_page_count(regions[n - 1].spec_size())
    }
}

} // verus!
