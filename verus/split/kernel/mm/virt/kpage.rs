// Copyright(c) The Maintainers of Nanvix.
// Licensed under the MIT License.

//==================================================================================================
//! # Kernel Page (Verified Implementation)
//!
//! This module provides a verified implementation of the `KernelPage` abstraction, which
//! represents a virtual memory page backed by a kernel physical frame. The `KernelPage`
//! provides the virtual-to-physical memory mapping abstraction for kernel-space memory.
//!
//! ## Overview
//!
//! A `KernelPage` is a thin wrapper around a `KernelFrame` that provides:
//! - Page address accessor (virtual address of the page)
//! - Frame address accessor (physical address of the underlying frame)
//!
//! The relationship between pages and frames is fundamental to virtual memory:
//! - A `KernelPage` represents a contiguous region of virtual address space (page-sized)
//! - A `KernelFrame` represents a contiguous region of physical memory (frame-sized)
//! - For kernel identity-mapped regions, page_address == frame_address
//!
//! ## Memory Safety Properties Verified
//!
//! 1. **Frame Ownership**: Each KernelPage owns exactly one valid KernelFrame.
//! 2. **Address Alignment**: Page and frame addresses are properly aligned.
//! 3. **Address Consistency**: The page address derives correctly from the frame address.
//! 4. **Provenance Preservation**: The underlying frame's pool provenance is preserved.
//! 5. **Invariant Preservation**: All operations maintain the KernelPage invariant.
//!
//! ## API Summary
//!
//! | Function         | Description                                    |
//! |-----------------|------------------------------------------------|
//! | `new(kframe)`   | Construct a KernelPage from a KernelFrame      |
//! | `base()`        | Get the page address (virtual address)         |
//! | `frame_address()`| Get the frame address (physical address)       |
//!
//! ## Abstraction Decisions
//!
//! ### PageAddress Simplification
//! In the original implementation, PageAddress wraps a PageAligned<VirtualAddress>.
//! For verification purposes, we model PageAddress as a simple wrapper around an
//! aligned address value, sufficient to capture the essential properties.
//!
//! ### Identity Mapping Assumption
//! For kernel pages, we assume identity mapping where the virtual page address
//! equals the physical frame address. This assumption is verified against the
//! original kernel implementation:
//!
//! **Evidence from original code:**
//! - `PhysicalAddress` wraps `VirtualAddress` directly (phys.rs:35)
//! - `PhysicalAddress::into_virtual_address()` returns `self.0` (phys.rs:76-78)
//! - The kernel explicitly uses "identity map memory regions" (virt/mod.rs:125)
//! - `PageAligned<PhysicalAddress>::into_virtual_address()` calls the identity
//!   conversion (aligned/page.rs:189-192)
//!
//! The translation path `frame.base().into_page_address().into_virtual_address()`
//! ultimately calls `PhysicalAddress::into_virtual_address()` which is identity.
//!
//! ## Relationship to Other Verified Modules
//!
//! - Uses `KernelFrame` from `kpool.rs` (verified)
//! - Uses `FrameAddress` from `frame_address.rs` (verified)
//! - Introduces `PageAddress` (simplified abstraction for verification)
//!
//! ## Verification-Only Additions
//!
//! The following methods are added for verification purposes but do not exist in the original:
//! - `KernelPage::pool_id()` - Exposes the underlying frame's pool ID for provenance tracking.
//!   This enables verification of memory allocation provenance properties.
//==================================================================================================

use crate::kernel::{
    hal::mem::types::address::frame::{
        FrameAddress,
        FRAME_SIZE,
    },
    mm::phys::kpool::KernelFrame,
};
use vstd::prelude::*;

// Include specifications.
include!("kpage.spec.rs");

// Include proofs.
include!("kpage.proof.rs");


verus! {

//==================================================================================================

/// Page size in bytes (4 KB). Same as frame size for x86.
pub const PAGE_SIZE: usize = 4096;


/// Page shift (log2 of PAGE_SIZE). Used for page table indexing.
pub const PAGE_SHIFT: usize = 12;


/// Page table shift (log2 of PGTAB_SIZE). For x86 32-bit, 22.
pub const PGTAB_SHIFT: usize = 22;


/// Number of page table entries per page table (1024 for x86 32-bit).
pub const PTES_PER_PGTAB: usize = 1024;

//==================================================================================================

/// A type that represents a page-aligned virtual address.
///
/// In the original implementation, PageAddress wraps `PageAligned<VirtualAddress>`.
/// For verification purposes, we model it as a simple wrapper around an aligned
/// address value, capturing the essential alignment and consistency properties.
#[derive(Debug, Clone, Copy)]
pub struct PageAddress {
    /// Raw virtual address (must be page-aligned).
    pub raw_addr: usize,
}

impl PageAddress {
    //==============================================================================================

    /// Creates a new PageAddress from a raw address value.
    ///
    /// # Parameters
    ///
    /// - `raw_addr`: The raw virtual address (must be page-aligned).
    ///
    /// # Returns
    ///
    /// A new PageAddress.
    pub fn new(raw_addr: usize) -> (result: PageAddress)
        requires
            raw_addr as int % PAGE_SIZE as int == 0,
        ensures
            result.spec_raw_value() == raw_addr as int,
            result.spec_is_aligned(),
    {
        PageAddress { raw_addr }
    }

    //==============================================================================================

    /// Gets the raw virtual address value.
    ///
    /// # Returns
    ///
    /// The raw virtual address.
    pub fn into_raw_value(self) -> (result: usize)
        ensures result as int == self.spec_raw_value()
    {
        self.raw_addr
    }


    /// Gets the page table entry index for this page address.
    ///
    /// # Description
    ///
    /// Computes the index into a page table for this address. This is used
    /// for page table management and virtual memory mapping operations.
    /// Equivalent to: (addr / PAGE_SIZE) % 1024
    ///
    /// # Returns
    ///
    /// The page table entry index (0 to 1023 for x86 32-bit).
    pub fn get_pte_index(&self) -> (result: usize)
        requires
            self.spec_is_aligned(),
        ensures
            result as int == self.spec_pte_index(),
            result < PTES_PER_PGTAB,
    {
        // Use arithmetic equivalent of bit extraction: (addr / PAGE_SIZE) % 1024.
        let page_num: usize = self.raw_addr / PAGE_SIZE;
        page_num % PTES_PER_PGTAB
    }
}

//==================================================================================================

/// Trait extension for PageAddress equality specification.
/// This connects the implementation to vstd's PartialEq specs.
pub trait PageAddressEqSpec {
    /// Specifies whether this type obeys the equality specification.
    spec fn obeys_eq_spec() -> bool;

    /// Specification for equality comparison.
    spec fn eq_spec(&self, other: &Self) -> bool;
}


/// Verified equality comparison for PageAddress.
///
/// # Description
///
/// Compares two page addresses for equality. This is a verified alternative
/// to the PartialEq trait implementation.
///
/// # Returns
///
/// True if both addresses have the same raw value.
pub fn page_address_eq(a: &PageAddress, b: &PageAddress) -> (result: bool)
    ensures
        result == (a.raw_addr == b.raw_addr),
        result == a.eq_spec(b),
        result == (a.spec_raw_value() == b.spec_raw_value()),
{
    a.raw_addr == b.raw_addr
}

impl PartialEq for PageAddress {
    /// Compares two page addresses for equality.
    ///
    /// # Description
    ///
    /// Two page addresses are equal if their raw address values are equal.
    /// The implementation is verified via the `page_address_eq` function and
    /// `lemma_page_address_eq_correct` lemma. The `external_body` marker is
    /// required because vstd's PartialEq trait specification requires implementing
    /// `obeys_eq_spec()` via an external trait extension mechanism that cannot
    /// be satisfied directly in user code.
    ///
    /// # Verification Justification
    ///
    /// - The implementation body (`self.raw_addr == other.raw_addr`) is trivially correct.
    /// - `page_address_eq()` provides a fully verified equivalent function.
    /// - `lemma_page_address_eq_correct()` proves the implementation matches `eq_spec()`.
    #[verifier::external_body]
    fn eq(&self, other: &Self) -> bool {
        self.raw_addr == other.raw_addr
    }
}


/// Abstract view of a kernel page for specification purposes.
///
/// The view captures the essential state:
/// - The virtual page address (aligned)
/// - The physical frame address (aligned)
/// - The underlying frame's properties (alignment, provenance)
///
/// For kernel pages with identity mapping, page_addr == frame_addr.
#[verifier::ext_equal]
pub struct KernelPageView {
    /// Virtual page address (page-aligned).
    pub page_addr: int,
    /// Physical frame address (frame-aligned).
    pub frame_addr: int,
    /// Pool ID from the underlying frame (for provenance tracking).
    pub pool_id: int,
}

//==================================================================================================

/// A type that represents a kernel page.
///
/// A kernel page wraps a kernel frame and provides the virtual memory abstraction.
/// For kernel-space memory with identity mapping, the virtual page address equals
/// the physical frame address.
///
/// # Memory Safety Guarantees
///
/// - The underlying frame is valid and page-aligned
/// - The page address is consistent with the frame address
/// - Frame provenance (pool_id) is preserved through the page abstraction
pub struct KernelPage {
    /// Underlying kernel frame.
    kframe: KernelFrame,
}

impl KernelPage {
    //==============================================================================================

    /// Instantiates a kernel page from a kernel frame.
    ///
    /// # Description
    ///
    /// Creates a new kernel page backed by the given kernel frame. The page
    /// address is derived from the frame address (identity mapping for kernel space).
    ///
    /// # Parameters
    ///
    /// - `kframe`: The underlying kernel frame (must be page-aligned).
    ///
    /// # Returns
    ///
    /// A new kernel page wrapping the given frame.
    ///
    /// # Ensures
    ///
    /// - The returned page satisfies its invariant
    /// - The page address equals the frame address (identity mapping)
    /// - The frame's provenance (pool_id) is preserved
    pub fn new(kframe: KernelFrame) -> (result: KernelPage)
        requires
            kframe.spec_is_aligned(),
        ensures
            result.inv(),
            result@.frame_address() == kframe.spec_raw_address(),
            result@.page_address() == kframe.spec_raw_address(),
            result@.pool_id() == kframe.spec_pool_id(),
            result@.is_identity_mapped(),
    {
        proof {
            // Use lemma to connect closed specs to FrameAddress properties.
            kframe.lemma_alignment_connection();
        }
        KernelPage { kframe }
    }

    //==============================================================================================

    /// Gets the base address (page address) of the kernel page.
    ///
    /// # Description
    ///
    /// Returns the virtual page address of this kernel page. For identity-mapped
    /// kernel memory, this equals the physical frame address.
    ///
    /// # Returns
    ///
    /// The page address of the kernel page.
    ///
    /// # Note
    ///
    /// This function is named `base()` for compatibility with the original API.
    /// Consider renaming to `page_address()` in the future.
    pub fn base(&self) -> (result: PageAddress)
        requires
            self.inv(),
        ensures
            result.spec_raw_value() == self@.page_address(),
            result.spec_is_aligned(),
    {
        proof {
            // Use lemma to connect closed specs to FrameAddress properties.
            self.kframe.lemma_alignment_connection();
        }
        // For identity mapping, use the frame address directly as the page address.
        let frame_addr: FrameAddress = self.kframe.base();
        PageAddress::new(frame_addr.into_raw_value())
    }


    /// Gets the frame address of the underlying kernel frame.
    ///
    /// # Description
    ///
    /// Returns the physical frame address of the underlying kernel frame.
    ///
    /// # Returns
    ///
    /// The frame address of the kernel page.
    pub fn frame_address(&self) -> (result: FrameAddress)
        requires
            self.inv(),
        ensures
            result.spec_raw_value() == self@.frame_address(),
            result.spec_is_aligned(),
    {
        proof {
            // Use lemma to connect closed specs to FrameAddress properties.
            self.kframe.lemma_alignment_connection();
        }
        self.kframe.base()
    }

    //==============================================================================================

    /// Gets the pool ID of the underlying frame.
    ///
    /// # Description
    ///
    /// Returns the pool identifier from which the underlying frame was allocated.
    /// This enables provenance tracking for memory safety.
    ///
    /// # Returns
    ///
    /// The pool ID of the underlying frame.
    pub fn pool_id(&self) -> (result: usize)
        requires
            self.inv(),
        ensures
            result as int == self@.pool_id(),
    {
        self.kframe.pool_id()
    }
}

} // verus!
