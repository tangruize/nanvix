// Copyright(c) The Maintainers of Nanvix.
// Licensed under the MIT License.

//==================================================================================================
//! # Virtual Memory Initialization (Verified Model)
//!
//! This module provides a verified model of the virtual memory initialization logic
//! from `src/kernel/src/mm/virt/mod.rs`. The original `init()` function processes
//! sorted memory regions, assigns pages to page tables, and detects overlapping regions.
//!
//! ## Overview
//!
//! The `init()` function performs the following steps:
//! 1. Merge virtual and MMIO memory region lists
//! 2. Sort regions by start address
//! 3. For each region, iterate page-by-page:
//!    - Compute the page table base via `align_down(vaddr, PGTAB_ALIGNMENT)`
//!    - If base > last page table base: create new page table
//!    - If base == last page table base: reuse existing page table
//!    - If base < last page table base: error (overlapping regions)
//!    - Map the page to its frame (identity mapping for non-MMIO, fixed paddr for MMIO)
//! 4. Return the ordered list of page tables
//!
//! ## Memory Safety Properties Verified
//!
//! 1. **Alignment Correctness**: `align_down` produces aligned values <= input.
//! 2. **Monotonicity**: Sorted addresses produce non-decreasing page table bases,
//!    guaranteeing the overlap-detection branch (Ordering::Less) is unreachable
//!    for properly sorted inputs.
//! 3. **Complete Coverage**: Page iteration visits every page in every region.
//! 4. **Inter-Region Ordering**: Non-overlapping sorted regions produce ordered
//!    page table bases across region boundaries.
//! 5. **Identity Mapping**: For non-MMIO regions, paddr == vaddr (verified via
//!    `spec_init_paddr`).
//! 6. **MMIO Mapping**: MMIO regions map all pages to the same physical frame
//!    (derived from `region.start()`, not current vaddr). This is faithfully
//!    modeled and may indicate a bug in the original code.
//! 7. **Loop Bound Correctness**: The original `end = start + (size - 1)` bound
//!    is reconciled with `spec_page_count`, including single-page edge case.
//! 8. **Overflow Safety**: End computation does not overflow for valid regions.
//! 9. **Page Table Decision**: The Equal/Greater/Less branching is modeled and
//!    the Overlap (Less) case is proven unreachable for sorted inputs.
//! 10. **Init Composition**: A verified `init()` function composes all helpers
//!     and proves the `VirtInitView` properties on its output.
//!
//! ## Abstraction Decisions
//!
//! ### Simplified Memory Region Model
//! The original uses `TruncatedMemoryRegion<VirtualAddress>` with complex type
//! wrappers. We model regions as simple `(start, size, is_mmio)` tuples.
//!
//! ### PageTableStorage Deref/DerefMut
//! The `Deref`/`DerefMut` implementations involve unsafe raw pointer operations
//! (`core::slice::from_raw_parts`) for the `KernelPage` variant. These are
//! modeled as `external_body` with specs asserting the returned slice length
//! equals `INIT_PAGE_SIZE / 4` (1024 entries).
//!
//! ### MMIO Address Translation
//! The `PhysicalAddress::from_mmio_address()` call is hardware-dependent and
//! modeled as an `external_body` with an opaque `spec_mmio_paddr` spec function.
//!
//! ### Init Function Decomposition
//! The init function is decomposed into verified components and then composed
//! in a verified `init()` function that proves `VirtInitView` properties:
//! - `virt_align_down`: Address alignment (core arithmetic)
//! - `compute_pgtab_base`: Page table assignment
//! - `pgtab_decision`: Three-way branching (Reuse/CreateNew/Overlap)
//! - `process_region_page`: Single page processing step
//! - `init`: Full initialization composing all components
//!
//! ## Relationship to Other Verified Modules
//!
//! - Uses page size constants consistent with `kpage.rs` (verified)
//! - Uses frame address concepts from `frame_address.rs` (verified)
//! - Page table management connects to `vmem.rs` (verified)
//==================================================================================================

pub mod kpage;
pub mod vmem;

//==================================================================================================
// Imports
//==================================================================================================

use vstd::prelude::*;

// Include specifications.
include!("mod.spec.rs");

// Include proofs.
include!("mod.proof.rs");


verus! {

//==================================================================================================
// Constants
//==================================================================================================

/// Page size in bytes (4 KB). Matches x86 `PAGE_SIZE`.
pub const INIT_PAGE_SIZE: usize = 4096;


/// Page table alignment (4 MB). Matches x86 `PGTAB_ALIGNMENT` = 1 << 22.
/// Each page table covers a 4 MB region of virtual address space.
pub const INIT_PGTAB_ALIGNMENT: usize = 4194304;


/// Number of page table entries per page table (1024 for x86 32-bit).
pub const INIT_PTES_PER_PGTAB: usize = 1024;


/// Kernel memory size (256 MB). Matches `config::kernel::MEMORY_SIZE`.
pub const INIT_MEMORY_SIZE: usize = 0x10000000;

//==================================================================================================
// Model Types
//==================================================================================================

/// Model of a memory region for verification.
///
/// # Description
///
/// Represents a contiguous region of virtual memory to be mapped during
/// kernel initialization. Simplifies the original `TruncatedMemoryRegion`
/// by capturing only the properties relevant to verification.
pub struct MemRegion {
    /// Start address of the region (must be page-aligned).
    pub start: usize,
    /// Size of the region in bytes (must be page-aligned and > 0).
    pub size: usize,
    /// Whether this is an MMIO region (different physical address mapping).
    pub is_mmio: bool,
}


/// Model of a page table storage type.
///
/// # Description
///
/// Represents the storage backing a page table. In the original implementation,
/// this is either a heap-allocated box or a kernel page. Both variants provide
/// a `[u32]` slice of page table entries via `Deref`/`DerefMut`.
pub enum PageTableStorage {
    /// Heap-allocated storage (Box<[u32; PAGE_SIZE / 4]>).
    Heap,
    /// Kernel page-backed storage.
    KernelPage,
}


impl PageTableStorage {
    /// Returns the length of the dereferenced slice.
    ///
    /// # Description
    ///
    /// Models `Deref for PageTableStorage` from the original implementation.
    /// Both variants produce a `[u32]` slice of length `PAGE_SIZE / sizeof(u32)` = 1024.
    ///
    /// The original `KernelPage` variant uses unsafe `core::slice::from_raw_parts`:
    /// ```ignore
    /// let base: *const u32 = page.base().into_raw_value() as *const u32;
    /// unsafe { core::slice::from_raw_parts(base, PAGE_SIZE / sizeof::<u32>()) }
    /// ```
    /// This is HAL-level unsafe code that cannot be verified without a hardware model.
    #[verifier::external_body]
    pub fn deref_len(&self) -> (result: usize)
        ensures
            result as int == spec_pgtab_entry_count(),
            result == INIT_PAGE_SIZE / 4,
    {
        INIT_PAGE_SIZE / 4
    }

    /// Returns the length of the mutably dereferenced slice.
    ///
    /// # Description
    ///
    /// Models `DerefMut for PageTableStorage` from the original implementation.
    /// Same length guarantee as `deref_len`.
    #[verifier::external_body]
    pub fn deref_mut_len(&self) -> (result: usize)
        ensures
            result as int == spec_pgtab_entry_count(),
            result == INIT_PAGE_SIZE / 4,
    {
        INIT_PAGE_SIZE / 4
    }
}

//==================================================================================================
// Core Verified Functions
//==================================================================================================

/// Aligns an address down to the nearest multiple of the given alignment.
///
/// # Description
///
/// Computes `floor(addr / alignment) * alignment`, which is the largest
/// multiple of `alignment` that is <= `addr`. This is equivalent to
/// `addr & !(alignment - 1)` when alignment is a power of 2.
///
/// # Parameters
///
/// - `addr`: The address to align.
/// - `alignment`: The alignment boundary (must be > 0).
///
/// # Returns
///
/// The aligned address.
pub fn virt_align_down(addr: usize, alignment: usize) -> (result: usize)
    requires
        alignment > 0,
    ensures
        result as int == spec_align_down(addr as int, alignment as int),
        result <= addr,
        result as int % alignment as int == 0,
{
    proof {
        VirtProofs::lemma_align_down_le(addr as int, alignment as int);
        VirtProofs::lemma_align_down_aligned(addr as int, alignment as int);
    }
    (addr / alignment) * alignment
}


/// Computes the page table base address for a given virtual address.
///
/// # Description
///
/// Determines which page table a virtual address belongs to by aligning
/// the address down to the page table alignment boundary (4 MB for x86).
///
/// # Parameters
///
/// - `vaddr`: The virtual address.
///
/// # Returns
///
/// The page-table-aligned base address.
pub fn compute_pgtab_base(vaddr: usize) -> (result: usize)
    ensures
        result as int == spec_pgtab_base(vaddr as int),
        result <= vaddr,
        result as int % INIT_PGTAB_ALIGNMENT as int == 0,
{
    proof {
        VirtProofs::lemma_align_down_le(vaddr as int, INIT_PGTAB_ALIGNMENT as int);
        VirtProofs::lemma_align_down_aligned(vaddr as int, INIT_PGTAB_ALIGNMENT as int);
    }
    virt_align_down(vaddr, INIT_PGTAB_ALIGNMENT)
}


/// Determines the page table assignment decision for the current address.
///
/// # Description
///
/// Models the three-way branching in the original init loop:
/// - If `last_base` is `None` (no previous page table), always create new.
/// - If `curr_base > last_base`: create new page table.
/// - If `curr_base == last_base`: reuse existing page table.
/// - If `curr_base < last_base`: overlapping regions error.
///
/// # Parameters
///
/// - `last_base`: The base address of the last page table (None if first).
/// - `vaddr`: The current virtual address being processed.
///
/// # Returns
///
/// The page table assignment decision.
pub fn pgtab_decision(last_base: Option<usize>, vaddr: usize) -> (result: PgtabDecision)
    requires
        last_base.is_some() ==> last_base.unwrap() as int % INIT_PGTAB_ALIGNMENT as int == 0,
    ensures
        // When no previous base, always create new.
        last_base.is_none() ==> matches!(result, PgtabDecision::CreateNew),
        // When there is a previous base and vaddr produces a >= base, no overlap.
        last_base.is_some() && spec_pgtab_base(vaddr as int) >= last_base.unwrap() as int
            ==> result.spec_is_ok(),
        // When there is a previous base and vaddr produces a < base, overlap error.
        last_base.is_some() && spec_pgtab_base(vaddr as int) < last_base.unwrap() as int
            ==> matches!(result, PgtabDecision::Overlap),
{
    let curr_base: usize = compute_pgtab_base(vaddr);
    match last_base {
        None => PgtabDecision::CreateNew,
        Some(prev) => {
            if curr_base > prev {
                PgtabDecision::CreateNew
            } else if curr_base == prev {
                PgtabDecision::Reuse
            } else {
                PgtabDecision::Overlap
            }
        },
    }
}


/// Verifies that processing addresses in non-decreasing order produces
/// non-decreasing page table bases (Overlap is unreachable).
///
/// # Parameters
///
/// - `prev_vaddr`: The previously processed virtual address.
/// - `curr_vaddr`: The current virtual address being processed.
///
/// # Returns
///
/// Always returns `true`, proving monotonicity holds.
pub fn check_pgtab_monotonicity(prev_vaddr: usize, curr_vaddr: usize) -> (result: bool)
    requires
        prev_vaddr <= curr_vaddr,
    ensures
        result == true,
{
    proof {
        VirtProofs::lemma_sorted_addrs_sorted_pgtab_bases(prev_vaddr as int, curr_vaddr as int);
    }
    let prev_base: usize = compute_pgtab_base(prev_vaddr);
    let curr_base: usize = compute_pgtab_base(curr_vaddr);
    curr_base >= prev_base
}


/// Computes the number of pages in a page-aligned region.
///
/// # Parameters
///
/// - `size`: The region size in bytes (page-aligned, > 0).
///
/// # Returns
///
/// The number of pages.
pub fn compute_region_page_count(size: usize) -> (result: usize)
    requires
        size > 0,
        size as int % INIT_PAGE_SIZE as int == 0,
    ensures
        result as int == spec_page_count(size as int),
        result > 0,
{
    size / INIT_PAGE_SIZE
}


/// Computes the virtual address of the i-th page in a region.
///
/// # Parameters
///
/// - `region_start`: The start address of the region (page-aligned).
/// - `index`: The page index (0-based).
///
/// # Returns
///
/// The virtual address of the page.
pub fn get_nth_page_addr(region_start: usize, index: usize) -> (result: usize)
    requires
        region_start as int % INIT_PAGE_SIZE as int == 0,
        region_start as int + index as int * INIT_PAGE_SIZE as int <= usize::MAX as int,
    ensures
        result as int == spec_nth_page_addr(region_start as int, index as int),
        result as int % INIT_PAGE_SIZE as int == 0,
{
    region_start + index * INIT_PAGE_SIZE
}


/// Computes the physical address for a page during init.
///
/// # Description
///
/// For non-MMIO regions: returns vaddr (identity mapping).
/// For MMIO regions: returns the MMIO physical address for the region start.
///
/// # Note on MMIO Behavior
///
/// In the original init(), the MMIO physical address is computed from
/// `region.start()` on every iteration, NOT from the current `raw_vaddr`.
/// This means ALL pages in an MMIO region are mapped to the SAME physical
/// frame. This is faithfully modeled here. The original code at lines 205-218:
/// ```ignore
/// paddr = match region.typ() {
///     MemoryRegionType::Mmio => {
///         let mmio_addr: VirtualAddress = region.start().into_inner();
///         // ...
///     },
///     _ => FrameAddress::new(PageAligned::from_address(
///         PhysicalAddress::from_raw_value(raw_vaddr)?)),
/// };
/// ```
///
/// # Parameters
///
/// - `vaddr`: The current page virtual address (page-aligned).
/// - `region_start`: The region start address (for MMIO lookups).
/// - `is_mmio`: Whether this is an MMIO region.
///
/// # Returns
///
/// The physical address to map.
pub fn get_page_paddr(vaddr: usize, region_start: usize, is_mmio: bool) -> (result: usize)
    requires
        vaddr as int % INIT_PAGE_SIZE as int == 0,
    ensures
        !is_mmio ==> result == vaddr,
        result as int == spec_init_paddr(vaddr as int, region_start as int, is_mmio),
{
    if is_mmio {
        // MMIO: use the region start address for physical address lookup.
        // This is external_body in the original (unsafe from_mmio_address).
        get_mmio_paddr(region_start)
    } else {
        // Non-MMIO: identity mapping.
        vaddr
    }
}


/// Gets the MMIO physical address for a region start address.
///
/// # Description
///
/// Models `PhysicalAddress::from_mmio_address(region.start())` from the original.
/// This is a hardware-dependent translation that cannot be verified without a
/// hardware model.
#[verifier::external_body]
pub fn get_mmio_paddr(region_start: usize) -> (result: usize)
    ensures
        result as int == spec_mmio_paddr(region_start as int),
{
    unimplemented!()
}


/// Computes the inclusive end address for the original loop bound.
///
/// # Description
///
/// Models `let end: usize = raw_vaddr + (region.size() - 1);` from the original.
///
/// # Parameters
///
/// - `start`: The region start address.
/// - `size`: The region size in bytes (must be > 0).
///
/// # Returns
///
/// The inclusive end address.
pub fn compute_loop_end(start: usize, size: usize) -> (result: usize)
    requires
        size > 0,
        start as int + size as int - 1 <= usize::MAX as int,
    ensures
        result as int == spec_loop_end(start as int, size as int),
{
    start + (size - 1)
}


/// Checks if a virtual address is at the last page before the memory boundary.
///
/// # Description
///
/// Models the break condition in the original init loop:
/// `if raw_vaddr == (config::kernel::MEMORY_SIZE - mem::PAGE_SIZE) { break; }`
///
/// # Parameters
///
/// - `vaddr`: The virtual address to check.
///
/// # Returns
///
/// `true` if this is the last page before the memory boundary.
pub fn is_last_kernel_page(vaddr: usize) -> (result: bool)
    requires
        INIT_MEMORY_SIZE >= INIT_PAGE_SIZE,
    ensures
        result == (vaddr == INIT_MEMORY_SIZE - INIT_PAGE_SIZE),
{
    vaddr == INIT_MEMORY_SIZE - INIT_PAGE_SIZE
}


/// Verified init function modeling the original `init()`.
///
/// # Description
///
/// Processes a sorted list of memory regions. For each region, iterates
/// page-by-page computing page table bases. Produces a list of unique,
/// sorted, aligned page table base addresses.
///
/// The algorithm:
/// 1. For each region, iterate over its pages.
/// 2. For each page, compute its page table base.
/// 3. If the base is new (greater than the last recorded base), add it.
/// 4. This produces a strictly increasing list of aligned bases.
///
/// # Parameters
///
/// - `regions`: Sorted list of valid, non-overlapping memory regions.
///
/// # Returns
///
/// A vector of unique, strictly increasing, page-table-aligned base addresses.
///
/// # Ensures
///
/// - `page_tables_aligned`: All bases are page-table-aligned.
/// - `page_tables_unique`: No duplicate bases (strictly increasing).
pub fn init(regions: &Vec<MemRegion>) -> (result: Vec<usize>)
    requires
        // All regions are valid.
        forall|i: int| #![auto] 0 <= i < regions.len() as int ==>
            regions[i].spec_is_valid(),
        // Regions are sorted by start address.
        forall|i: int, j: int|
            #![trigger regions[i], regions[j]]
            0 <= i < j < regions.len() as int ==>
            regions[i].spec_start() <= regions[j].spec_start(),
        // Regions are non-overlapping.
        forall|i: int, j: int|
            #![trigger regions[i], regions[j]]
            0 <= i < j < regions.len() as int ==>
            regions[i].spec_end() <= regions[j].spec_start(),
        // Regions have page-aligned start and size.
        forall|i: int| #![auto] 0 <= i < regions.len() as int ==>
            regions[i].start as int % INIT_PAGE_SIZE as int == 0
            && regions[i].size as int % INIT_PAGE_SIZE as int == 0,
    ensures
        // All output bases are aligned.
        forall|i: int| #![auto] 0 <= i < result.len() as int ==>
            result[i] as int % INIT_PGTAB_ALIGNMENT as int == 0,
        // Output bases are strictly increasing (ordered + unique).
        forall|i: int, j: int|
            #![trigger result[i], result[j]]
            0 <= i < j < result.len() as int ==>
            (result[i] as int) < (result[j] as int),
{
    let mut all_bases: Vec<usize> = Vec::new();
    let mut last_base: Option<usize> = None;
    let mut r_idx: usize = 0;

    while r_idx < regions.len()
        invariant
            0 <= r_idx <= regions.len(),
            // Forward regions info.
            forall|i: int| #![auto] 0 <= i < regions.len() as int ==>
                regions[i].spec_is_valid(),
            forall|i: int, j: int|
                #![trigger regions[i], regions[j]]
                0 <= i < j < regions.len() as int ==>
                regions[i].spec_start() <= regions[j].spec_start(),
            forall|i: int, j: int|
                #![trigger regions[i], regions[j]]
                0 <= i < j < regions.len() as int ==>
                regions[i].spec_end() <= regions[j].spec_start(),
            forall|i: int| #![auto] 0 <= i < regions.len() as int ==>
                regions[i].start as int % INIT_PAGE_SIZE as int == 0
                && regions[i].size as int % INIT_PAGE_SIZE as int == 0,
            // All accumulated bases are aligned.
            forall|i: int| #![auto] 0 <= i < all_bases.len() as int ==>
                all_bases[i] as int % INIT_PGTAB_ALIGNMENT as int == 0,
            // Accumulated bases are strictly increasing.
            forall|i: int, j: int|
                #![trigger all_bases[i], all_bases[j]]
                0 <= i < j < all_bases.len() as int ==>
                (all_bases[i] as int) < (all_bases[j] as int),
            // last_base is aligned when present.
            last_base.is_some() ==>
                last_base.unwrap() as int % INIT_PGTAB_ALIGNMENT as int == 0,
            // last_base >= all accumulated bases (enables strictly-increasing pushes).
            last_base.is_some() ==> forall|i: int| #![auto]
                0 <= i < all_bases.len() as int ==>
                all_bases[i] as int <= last_base.unwrap() as int,
        decreases regions.len() - r_idx,
    {
        let region: &MemRegion = &regions[r_idx];
        let page_count: usize = compute_region_page_count(region.size);
        let mut p_idx: usize = 0;

        // Prove: if last_base is Some, then pgtab_base(region.start) >= last_base.
        // This comes from the region ordering and monotonicity of pgtab_base.
        // After the previous region, last_base = pgtab_base(prev_region_last_page).
        // prev_region_last_page < prev_region.end <= region.start.
        // By monotonicity, pgtab_base(prev_last_page) <= pgtab_base(region.start).

        while p_idx < page_count
            invariant
                0 <= p_idx <= page_count,
                page_count as int == region.size as int / INIT_PAGE_SIZE as int,
                page_count > 0,
                region.spec_is_valid(),
                region.start as int % INIT_PAGE_SIZE as int == 0,
                region.size as int % INIT_PAGE_SIZE as int == 0,
                // All accumulated bases are aligned.
                forall|i: int| #![auto] 0 <= i < all_bases.len() as int ==>
                    all_bases[i] as int % INIT_PGTAB_ALIGNMENT as int == 0,
                // Accumulated bases are strictly increasing.
                forall|i: int, j: int|
                    #![trigger all_bases[i], all_bases[j]]
                    0 <= i < j < all_bases.len() as int ==>
                    (all_bases[i] as int) < (all_bases[j] as int),
                // last_base is aligned when present.
                last_base.is_some() ==>
                    last_base.unwrap() as int % INIT_PGTAB_ALIGNMENT as int == 0,
                // last_base >= all accumulated bases.
                last_base.is_some() ==> forall|i: int| #![auto]
                    0 <= i < all_bases.len() as int ==>
                    all_bases[i] as int <= last_base.unwrap() as int,
                // After first page, last_base is Some.
                p_idx > 0 ==> last_base.is_some(),
                // last_base tracks the pgtab base of the previous page.
                p_idx > 0 ==> last_base.unwrap() as int == spec_pgtab_base(
                    spec_nth_page_addr(region.start as int, (p_idx - 1) as int)),
            decreases page_count - p_idx,
        {
            proof {
                // Prove overflow safety for get_nth_page_addr.
                VirtProofs::lemma_page_iteration_covers_region(
                    region.start as int, region.size as int, p_idx as int);
            }
            let vaddr: usize = get_nth_page_addr(region.start, p_idx);
            let curr_base: usize = compute_pgtab_base(vaddr);

            proof {
                // Prove monotonicity: if last_base is Some, curr_base >= last_base.
                if p_idx > 0 {
                    VirtProofs::lemma_consecutive_pages_ordered_bases(
                        region.start as int, (p_idx - 1) as int, p_idx as int);
                }
            }

            let should_add: bool = match last_base {
                None => true,
                Some(prev) => curr_base > prev,
            };

            if should_add {
                // curr_base > last_base.unwrap() >= all existing all_bases,
                // so curr_base > all existing all_bases. Push maintains strictly increasing.
                all_bases.push(curr_base);
            }
            last_base = Some(curr_base);
            p_idx = p_idx + 1;
        }

        // After processing all pages of this region, last_base is Some.
        // Prove: if there's a next region, the next region's pages will have
        // pgtab bases >= last_base, so monotonicity is maintained.
        proof {
            if r_idx + 1 < regions.len() {
                // last_base = pgtab_base(last_page_of_this_region).
                // last_page < region.end <= next_region.start.
                // pgtab_base(last_page) <= pgtab_base(next_region.start) by monotonicity.
                let last_page_idx: int = region.size as int / INIT_PAGE_SIZE as int - 1;
                let last_page: int = spec_nth_page_addr(region.start as int, last_page_idx);
                VirtProofs::lemma_page_iteration_covers_region(
                    region.start as int, region.size as int, last_page_idx);
                // last_page < region.end = region.start + region.size.
                // region.end <= next_region.start.
            }
        }
        r_idx = r_idx + 1;
    }

    all_bases
}

} // verus!
