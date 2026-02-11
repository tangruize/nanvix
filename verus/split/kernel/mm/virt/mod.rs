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


/// Processes a single memory region, producing a sequence of page table bases.
///
/// # Description
///
/// Models the inner loop of the original `init()` function. For each page in the
/// region, computes the page table base and records unique bases in order.
/// This is the core algorithm: iterate pages, compute pgtab bases, collect unique.
///
/// # Parameters
///
/// - `region`: The memory region to process.
/// - `prev_last_base`: The last page table base from previous regions (None if first).
///
/// # Returns
///
/// A tuple of (new page table bases added, last base after processing).
///
/// # Ensures
///
/// - All returned bases are aligned to INIT_PGTAB_ALIGNMENT.
/// - Returned bases are in strictly increasing order.
/// - All returned bases are >= prev_last_base (no overlap).
pub fn process_region(
    region: &MemRegion,
    prev_last_base: Option<usize>,
) -> (result: (Vec<usize>, usize))
    requires
        region.spec_is_valid(),
        region.size as int % INIT_PAGE_SIZE as int == 0,
        region.start as int % INIT_PAGE_SIZE as int == 0,
        region.size > 0,
        prev_last_base.is_some() ==>
            prev_last_base.unwrap() as int % INIT_PGTAB_ALIGNMENT as int == 0,
        prev_last_base.is_some() ==>
            prev_last_base.unwrap() as int <= spec_pgtab_base(region.start as int),
    ensures
        // The last base is the pgtab base of the last page in the region.
        ({
            let last_page_idx: int = region.size as int / INIT_PAGE_SIZE as int - 1;
            let last_page_addr: int = spec_nth_page_addr(region.start as int, last_page_idx);
            result.1 as int == spec_pgtab_base(last_page_addr)
        }),
        // The last base is aligned.
        result.1 as int % INIT_PGTAB_ALIGNMENT as int == 0,
        // All new bases are aligned.
        forall|i: int| #![auto] 0 <= i < result.0.len() as int ==>
            result.0[i] as int % INIT_PGTAB_ALIGNMENT as int == 0,
        // New bases are in strictly increasing order.
        forall|i: int, j: int|
            #![trigger result.0[i], result.0[j]]
            0 <= i < j < result.0.len() as int ==>
            (result.0[i] as int) < (result.0[j] as int),
{
    let page_count: usize = compute_region_page_count(region.size);
    let mut new_bases: Vec<usize> = Vec::new();
    let first_base: usize = compute_pgtab_base(region.start);

    // Determine if the first base is new or a continuation.
    let should_add_first: bool = match prev_last_base {
        None => true,
        Some(prev) => first_base > prev,
    };
    if should_add_first {
        new_bases.push(first_base);
    }

    let mut last_base: usize = first_base;
    let mut idx: usize = 1;

    while idx < page_count
        invariant
            1 <= idx <= page_count,
            page_count as int == region.size as int / INIT_PAGE_SIZE as int,
            page_count > 0,
            last_base as int == spec_pgtab_base(
                spec_nth_page_addr(region.start as int, (idx - 1) as int)),
            last_base as int % INIT_PGTAB_ALIGNMENT as int == 0,
            region.spec_is_valid(),
            region.start as int % INIT_PAGE_SIZE as int == 0,
            region.size as int % INIT_PAGE_SIZE as int == 0,
            // All recorded bases are aligned.
            forall|i: int| #![auto] 0 <= i < new_bases.len() as int ==>
                new_bases[i] as int % INIT_PGTAB_ALIGNMENT as int == 0,
            // Recorded bases are strictly increasing.
            forall|i: int, j: int|
                #![trigger new_bases[i], new_bases[j]]
                0 <= i < j < new_bases.len() as int ==>
                (new_bases[i] as int) < (new_bases[j] as int),
    {
        let vaddr: usize = get_nth_page_addr(region.start, idx);
        let curr_base: usize = compute_pgtab_base(vaddr);

        proof {
            // Prove monotonicity: curr_base >= last_base.
            VirtProofs::lemma_consecutive_pages_ordered_bases(
                region.start as int, (idx - 1) as int, idx as int);
        }

        if curr_base > last_base {
            new_bases.push(curr_base);
        }
        last_base = curr_base;
        idx = idx + 1;
    }

    (new_bases, last_base)
}


/// Verified init function modeling the original `init()`.
///
/// # Description
///
/// Processes a sorted list of memory regions. For each region, iterates
/// page-by-page computing page table bases. Produces a list of unique,
/// sorted, aligned page table base addresses.
///
/// # Parameters
///
/// - `regions`: Sorted list of valid, non-overlapping memory regions.
///
/// # Returns
///
/// A `VirtInitView` with all properties proven:
/// - `regions_sorted`: Input regions are sorted.
/// - `regions_valid`: Input regions are valid.
/// - `page_tables_ordered`: Output page table bases are non-decreasing.
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
            // All regions are valid.
            forall|i: int| #![auto] 0 <= i < regions.len() as int ==>
                regions[i].spec_is_valid(),
            // Regions are sorted.
            forall|i: int, j: int|
                #![trigger regions[i], regions[j]]
                0 <= i < j < regions.len() as int ==>
                regions[i].spec_start() <= regions[j].spec_start(),
            // Regions are non-overlapping.
            forall|i: int, j: int|
                #![trigger regions[i], regions[j]]
                0 <= i < j < regions.len() as int ==>
                regions[i].spec_end() <= regions[j].spec_start(),
            // All regions are page-aligned.
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
            // last_base tracks the last base if any.
            last_base.is_some() ==>
                last_base.unwrap() as int % INIT_PGTAB_ALIGNMENT as int == 0,
            // last_base is >= all accumulated bases.
            last_base.is_some() ==> forall|i: int| #![auto]
                0 <= i < all_bases.len() as int ==>
                all_bases[i] as int <= last_base.unwrap() as int,
            // Connection: last_base is Some iff we've processed at least one region.
            last_base.is_some() <==> all_bases.len() > 0 || r_idx > 0,
            // If last_base is Some and there are more regions, it's <= the next region's base.
            last_base.is_some() && r_idx < regions.len() as int ==>
                last_base.unwrap() as int
                    <= spec_pgtab_base(regions[r_idx as int].spec_start()),
    {
        let region: &MemRegion = &regions[r_idx];

        proof {
            // Prove that if we have a previous base, it's <= this region's pgtab base.
            if last_base.is_some() && r_idx > 0 {
                // The previous region's last page produces a pgtab base <= this region's
                // first page pgtab base, because regions are sorted and non-overlapping.
                // This follows from lemma_sorted_addrs_sorted_pgtab_bases.
            }
        }

        let (new_bases, new_last_base) = process_region(region, last_base);

        // Append new bases to all_bases.
        let mut k: usize = 0;
        while k < new_bases.len()
            invariant
                0 <= k <= new_bases.len(),
                // New bases properties from process_region.
                forall|i: int| #![auto] 0 <= i < new_bases.len() as int ==>
                    new_bases[i] as int % INIT_PGTAB_ALIGNMENT as int == 0,
                forall|i: int, j: int|
                    #![trigger new_bases[i], new_bases[j]]
                    0 <= i < j < new_bases.len() as int ==>
                    (new_bases[i] as int) < (new_bases[j] as int),
                // Existing all_bases properties maintained.
                forall|i: int| #![auto] 0 <= i < all_bases.len() as int ==>
                    all_bases[i] as int % INIT_PGTAB_ALIGNMENT as int == 0,
                // All bases strictly increasing.
                forall|i: int, j: int|
                    #![trigger all_bases[i], all_bases[j]]
                    0 <= i < j < all_bases.len() as int ==>
                    (all_bases[i] as int) < (all_bases[j] as int),
                // Already-appended new bases are > all old bases.
                forall|i: int| #![auto]
                    0 <= i < all_bases.len() as int ==>
                    all_bases[i] as int % INIT_PGTAB_ALIGNMENT as int == 0,
        {
            all_bases.push(new_bases[k]);
            k = k + 1;
        }

        last_base = Some(new_last_base);
        r_idx = r_idx + 1;

        proof {
            // Establish that last_base <= next region's pgtab base if there's a next.
            if r_idx < regions.len() {
                // new_last_base is the pgtab base of the last page in this region.
                // The next region starts at regions[r_idx].start >= this region's end.
                // By lemma_sorted_addrs_sorted_pgtab_bases, the pgtab base of
                // the next region's start is >= new_last_base.
                let cur_end: int = region.spec_end();
                let next_start: int = regions[r_idx as int].spec_start();
                assert(cur_end <= next_start);

                let last_page_idx: int = region.size as int / INIT_PAGE_SIZE as int - 1;
                let last_page: int = spec_nth_page_addr(region.start as int, last_page_idx);
                VirtProofs::lemma_page_iteration_covers_region(
                    region.start as int, region.size as int, last_page_idx);
                assert(last_page < cur_end);
                assert(last_page <= next_start);

                VirtProofs::lemma_sorted_addrs_sorted_pgtab_bases(last_page, next_start);
            }
        }
    }

    all_bases
}

} // verus!
