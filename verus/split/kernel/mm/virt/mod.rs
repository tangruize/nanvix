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
//!    - Map the page to its frame
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
//! 5. **Identity Mapping**: For non-MMIO regions, paddr == vaddr.
//! 6. **Idempotence**: Already-aligned addresses are fixed points of align_down.
//!
//! ## Abstraction Decisions
//!
//! ### Simplified Memory Region Model
//! The original uses `TruncatedMemoryRegion<VirtualAddress>` with complex type
//! wrappers. We model regions as simple `(start, size, is_mmio)` tuples, capturing
//! the essential properties needed for verification.
//!
//! ### PageTableStorage as External
//! `PageTableStorage` involves unsafe raw pointer operations for the `KernelPage`
//! variant. The `Deref`/`DerefMut` implementations use `core::slice::from_raw_parts`
//! which cannot be verified without a hardware memory model. These are marked
//! `external_body` as they are HAL-level operations.
//!
//! ### LinkedList/Vec Abstraction
//! The original uses `LinkedList` and `Vec` for region management. We model the
//! core algorithmic properties (sorting, ordering) using spec-level sequences
//! and verify them against exec-level functions operating on individual addresses.
//!
//! ### Init Function Decomposition
//! Rather than verifying the entire `init()` function monolithically, we decompose
//! it into verified components:
//! - `virt_align_down`: Address alignment (core arithmetic)
//! - `compute_pgtab_base`: Page table assignment
//! - `check_pgtab_monotonicity`: Ordering verification
//! - `compute_region_page_count`: Page enumeration
//! - `get_nth_page_addr`: Page address computation
//! The composition of these components models the init loop behavior.
//!
//! ## Relationship to Other Verified Modules
//!
//! - Uses page size constants consistent with `kpage.rs` (verified)
//! - Uses frame address concepts from `frame_address.rs` (verified)
//! - Page table management connects to `vmem.rs` (verified)
//!
//! ## Verification-Only Additions
//!
//! - `MemRegion`: Simplified memory region model for verification
//! - `VirtInitView`: Abstract view type for init result properties
//! - Proof lemmas for alignment, monotonicity, and coverage properties
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
/// a `[u32]` slice of page table entries.
///
/// The `Deref`/`DerefMut` implementations involve unsafe raw pointer operations
/// for the `KernelPage` variant and are marked `external_body`.
pub enum PageTableStorage {
    /// Heap-allocated storage (Box<[u32; PAGE_SIZE / 4]>).
    Heap,
    /// Kernel page-backed storage.
    KernelPage,
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
        addr >= 0,
    ensures
        result as int == spec_align_down(addr as int, alignment as int),
        result <= addr,
        result as int % alignment as int == 0,
{
    proof {
        lemma_align_down_le(addr as int, alignment as int);
        lemma_align_down_aligned(addr as int, alignment as int);
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
    requires
        true,
    ensures
        result as int == spec_pgtab_base(vaddr as int),
        result <= vaddr,
        result as int % INIT_PGTAB_ALIGNMENT as int == 0,
{
    proof {
        lemma_align_down_le(vaddr as int, INIT_PGTAB_ALIGNMENT as int);
        lemma_align_down_aligned(vaddr as int, INIT_PGTAB_ALIGNMENT as int);
    }
    virt_align_down(vaddr, INIT_PGTAB_ALIGNMENT)
}


/// Verifies that processing addresses in non-decreasing order produces
/// non-decreasing page table bases.
///
/// # Description
///
/// This function encodes the key safety property of the init algorithm:
/// if virtual addresses are processed in sorted order, the page table
/// base addresses never decrease. This means the `Ordering::Less` error
/// branch in the original init function is unreachable when regions are
/// properly sorted and non-overlapping.
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
        lemma_sorted_addrs_sorted_pgtab_bases(prev_vaddr as int, curr_vaddr as int);
    }
    let prev_base: usize = compute_pgtab_base(prev_vaddr);
    let curr_base: usize = compute_pgtab_base(curr_vaddr);
    curr_base >= prev_base
}


/// Computes the number of pages in a page-aligned region.
///
/// # Description
///
/// Returns how many pages fit in a region of the given size. The size
/// must be page-aligned and positive.
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
        size <= usize::MAX,
    ensures
        result as int == spec_page_count(size as int),
        result > 0,
{
    size / INIT_PAGE_SIZE
}


/// Computes the virtual address of the i-th page in a region.
///
/// # Description
///
/// Returns `region_start + index * PAGE_SIZE`, which is the virtual
/// address of the page at position `index` within the region.
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
        index >= 0,
    ensures
        result as int == spec_nth_page_addr(region_start as int, index as int),
        result as int % INIT_PAGE_SIZE as int == 0,
{
    region_start + index * INIT_PAGE_SIZE
}


/// Computes the identity-mapped physical address for a non-MMIO page.
///
/// # Description
///
/// For non-MMIO kernel memory, the physical address equals the virtual
/// address (identity mapping). This function models that relationship.
///
/// # Parameters
///
/// - `vaddr`: The virtual address (page-aligned).
///
/// # Returns
///
/// The physical address (equal to vaddr for identity mapping).
pub fn get_identity_mapped_paddr(vaddr: usize) -> (result: usize)
    requires
        vaddr as int % INIT_PAGE_SIZE as int == 0,
    ensures
        result == vaddr,
        result as int % INIT_PAGE_SIZE as int == 0,
{
    vaddr
}


/// Checks if a virtual address is within kernel memory bounds.
///
/// # Description
///
/// Returns true if the address is within the kernel's memory region
/// [0, INIT_MEMORY_SIZE). The original init function uses this check
/// as a loop termination condition.
///
/// # Parameters
///
/// - `vaddr`: The virtual address to check.
///
/// # Returns
///
/// `true` if the address is within kernel memory bounds.
pub fn is_within_memory_bounds(vaddr: usize) -> (result: bool)
    ensures
        result == (vaddr < INIT_MEMORY_SIZE),
{
    vaddr < INIT_MEMORY_SIZE
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

} // verus!
