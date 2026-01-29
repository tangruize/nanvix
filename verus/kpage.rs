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
//! equals the physical frame address. This is a common kernel memory model.
//!
//! ## Relationship to Other Verified Modules
//!
//! - Uses `KernelFrame` from `kpool.rs` (verified)
//! - Uses `FrameAddress` from `frame_address.rs` (verified)
//! - Introduces `PageAddress` (simplified abstraction for verification)
//==================================================================================================

use crate::{
    kpool::KernelFrame,
    frame_address::{FrameAddress, FRAME_SIZE},
};
use vstd::prelude::*;

verus! {

//==================================================================================================
// Constants
//==================================================================================================

/// Page size in bytes (4 KB). Same as frame size for x86.
pub const PAGE_SIZE: usize = 4096;

//==================================================================================================
// PageAddress - Virtual Page Address Abstraction
//==================================================================================================

/// A type that represents a page-aligned virtual address.
///
/// In the original implementation, PageAddress wraps `PageAligned<VirtualAddress>`.
/// For verification purposes, we model it as a simple wrapper around an aligned
/// address value, capturing the essential alignment and consistency properties.
#[derive(Debug, Clone, Copy)]
pub struct PageAddress {
    /// Raw virtual address (must be page-aligned).
    raw_addr: usize,
}

impl PageAddress {
    //==============================================================================================
    // Specification Functions
    //==============================================================================================

    /// Spec function to get the raw address value.
    pub open spec fn spec_raw_value(&self) -> int {
        self.raw_addr as int
    }

    /// Spec function to check if address is page-aligned.
    pub open spec fn spec_is_aligned(&self) -> bool {
        self.raw_addr as int % PAGE_SIZE as int == 0
    }

    /// Spec function to get the page number.
    pub open spec fn spec_page_number(&self) -> int {
        self.raw_addr as int / PAGE_SIZE as int
    }

    //==============================================================================================
    // Constructor
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
    // Accessors
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
}

//==================================================================================================
// KernelPageView - Abstract Specification
//==================================================================================================

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

impl KernelPageView {
    //==============================================================================================
    // Basic Properties
    //==============================================================================================

    /// Returns the page address.
    pub open spec fn page_address(&self) -> int {
        self.page_addr
    }

    /// Returns the frame address.
    pub open spec fn frame_address(&self) -> int {
        self.frame_addr
    }

    /// Returns the pool ID of the underlying frame.
    pub open spec fn pool_id(&self) -> int {
        self.pool_id
    }

    /// Returns the page number (index).
    pub open spec fn page_number(&self) -> int {
        self.page_addr / PAGE_SIZE as int
    }

    /// Returns the frame number (index).
    pub open spec fn frame_number(&self) -> int {
        self.frame_addr / FRAME_SIZE as int
    }

    //==============================================================================================
    // Alignment Properties
    //==============================================================================================

    /// Property: The page address is page-aligned.
    pub open spec fn page_is_aligned(&self) -> bool {
        self.page_addr % PAGE_SIZE as int == 0
    }

    /// Property: The frame address is frame-aligned.
    pub open spec fn frame_is_aligned(&self) -> bool {
        self.frame_addr % FRAME_SIZE as int == 0
    }

    //==============================================================================================
    // Consistency Properties
    //==============================================================================================

    /// Property: For identity-mapped kernel pages, page address equals frame address.
    /// This is the fundamental invariant for kernel memory.
    pub open spec fn is_identity_mapped(&self) -> bool {
        self.page_addr == self.frame_addr
    }

    /// Property: Page and frame numbers are equal for identity-mapped pages.
    pub open spec fn consistent_numbering(&self) -> bool {
        self.page_number() == self.frame_number()
    }

    //==============================================================================================
    // Memory Safety Properties
    //==============================================================================================

    /// Property: The page address is non-negative (valid address space).
    pub open spec fn addr_is_valid(&self) -> bool {
        self.page_addr >= 0 && self.frame_addr >= 0
    }
}

//==================================================================================================
// KernelPage - Verified Implementation
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

impl View for KernelPage {
    type V = KernelPageView;

    closed spec fn view(&self) -> KernelPageView {
        KernelPageView {
            // For identity mapping, page address == frame address.
            page_addr: self.kframe.spec_raw_address(),
            frame_addr: self.kframe.spec_raw_address(),
            pool_id: self.kframe.spec_pool_id(),
        }
    }
}

impl KernelPage {
    //==============================================================================================
    // Invariant
    //==============================================================================================

    /// Invariant for the kernel page.
    ///
    /// Ensures internal consistency and memory safety guarantees:
    /// - The underlying frame is page-aligned
    /// - Address values are consistent between page and frame
    /// - Identity mapping holds (page_addr == frame_addr)
    pub closed spec fn inv(&self) -> bool {
        // The underlying frame is page-aligned.
        &&& self.kframe.spec_is_aligned()
        // View consistency: page address equals frame address (identity mapping).
        &&& self@.is_identity_mapped()
        // View consistency: both addresses are aligned.
        &&& self@.page_is_aligned()
        &&& self@.frame_is_aligned()
        // View consistency: page address matches frame's raw address.
        &&& self@.page_addr == self.kframe.spec_raw_address()
        &&& self@.frame_addr == self.kframe.spec_raw_address()
        // Pool ID is preserved from the underlying frame.
        &&& self@.pool_id == self.kframe.spec_pool_id()
        // Address validity.
        &&& self@.addr_is_valid()
    }

    //==============================================================================================
    // Specification Functions
    //==============================================================================================

    /// Spec function to get the underlying frame.
    pub closed spec fn spec_kframe(&self) -> KernelFrame {
        self.kframe
    }

    /// Spec function to get the page address.
    pub closed spec fn spec_page_address(&self) -> int {
        self@.page_address()
    }

    /// Spec function to get the frame address.
    pub closed spec fn spec_frame_address(&self) -> int {
        self@.frame_address()
    }

    /// Spec function to get the pool ID of the underlying frame.
    pub closed spec fn spec_pool_id(&self) -> int {
        self@.pool_id()
    }

    //==============================================================================================
    // Constructor
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
        KernelPage { kframe }
    }

    //==============================================================================================
    // Accessors
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
        self.kframe.base()
    }

    //==============================================================================================
    // Additional Verified Accessors
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

//==================================================================================================
// Proof: PAGE_SIZE equals FRAME_SIZE
//==================================================================================================

/// Proof that PAGE_SIZE equals FRAME_SIZE (both are 4KB on x86).
/// This is essential for identity mapping to work correctly.
proof fn proof_page_frame_size_equality()
    ensures PAGE_SIZE as int == FRAME_SIZE as int
{
    // Both are compile-time constants equal to 4096.
}

} // verus!
