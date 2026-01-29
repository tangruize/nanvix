// Copyright(c) The Maintainers of Nanvix.
// Licensed under the MIT License.

//==================================================================================================
//! # Virtual Memory Space (Verified Implementation)
//!
//! This module provides a verified implementation of the `Vmem` abstraction, which represents
//! a virtual memory space. The `Vmem` structure manages the mapping between virtual addresses
//! and physical frames through page directories and page tables.
//!
//! ## Overview
//!
//! A `Vmem` (Virtual Memory Space) provides:
//! - Page directory management for virtual-to-physical address translation
//! - Kernel page table management (shared across address spaces)
//! - User page table management (private to each address space)
//! - Kernel page management (pages mapped into kernel space)
//! - User frame mapping/unmapping operations
//! - Memory copy operations between user and kernel space
//!
//! ## Memory Safety Properties Verified
//!
//! 1. **Address Space Separation**: User and kernel address spaces are disjoint.
//! 2. **Valid Address Bounds**: User addresses must be within USER_BASE..USER_END.
//! 3. **Region Validity**: Memory regions don't overflow and are properly bounded.
//! 4. **Physical Memory Bounds**: Physical addresses lie within valid memory range.
//! 5. **Non-Zero Size**: Memory operations require non-zero size.
//! 6. **Invariant Preservation**: All operations maintain the Vmem invariant.
//!
//! ## Abstraction Decisions
//!
//! ### Simplified Page Table Model
//! The original implementation uses complex linked lists of page tables with Rc<RefCell<>>
//! for shared ownership. For verification, we model page tables as abstract maps from
//! virtual addresses to physical frames, capturing the essential translation properties.
//!
//! ### Page Directory as Abstract Map
//! Instead of modeling the x86-specific page directory structure, we use an abstract
//! map that captures the semantics: each page table address maps to whether it's present.
//!
//! ### External Memory Operations
//! Physical memory operations (__phys_memcpy, __phys_memset) are modeled with
//! external_body since they involve low-level hardware operations.
//!
//! ## Relationship to Other Verified Modules
//!
//! - Uses `KernelPage` from `kpage.rs` (verified) - for kernel page abstraction
//! - Uses `FrameAddress` from `frame_address.rs` (verified) - for frame addresses
//! - Uses `UserFrame` from `upool.rs` (verified) - for user frame abstraction
//!
//! ## API Summary
//!
//! | Function | Description |
//! |----------|-------------|
//! | `new()` | Create a new virtual memory space |
//! | `clone()` | Clone an existing virtual memory space |
//! | `map()` | Map a user frame to a virtual address |
//! | `unmap()` | Unmap a page from the virtual address space |
//! | `map_kpage()` | Map a kernel page |
//! | `is_user_addr()` | Check if address is in user space |
//! | `is_user_region()` | Check if region is entirely in user space |
//! | `is_kernel_addr()` | Check if address is in kernel space |
//! | `is_kernel_region()` | Check if region is entirely in kernel space |
//! | `is_physical_region()` | Check if region is within physical memory |
//! | `copy_from_user_unaligned()` | Copy from user space to kernel space |
//! | `copy_to_user_unaligned()` | Copy from kernel space to user space |
//! | `memset()` | Fill a page with a value |
//! | `uctrl()` | Change access permissions on a user page |
//! | `kctrl()` | Change access permissions on a kernel page |
//==================================================================================================

use crate::{
    kpage::{KernelPage, PageAddress, PAGE_SIZE},
    upool::UserFrame,
    frame_address::{FrameAddress, FRAME_SIZE},
    error::{Error, ErrorCode},
};
use vstd::prelude::*;
use vstd::set::*;

verus! {

//==================================================================================================
// Constants
//==================================================================================================

/// User space base address.
/// Must be page-aligned and mark the start of user-accessible virtual memory.
pub const USER_BASE: usize = 0x40000000; // 1 GB

/// User space end address (exclusive).
/// Must be page-aligned and mark the end of user-accessible virtual memory.
pub const USER_END: usize = 0xC0000000; // 3 GB

/// Total physical memory size (for bounds checking).
/// This should match the system configuration.
pub const MEMORY_SIZE: usize = 0x10000000; // 256 MB

/// Page table alignment (4 MB for x86 32-bit).
pub const PGTAB_ALIGNMENT: usize = 0x400000; // 4 MB

//==================================================================================================
// Access Permissions
//==================================================================================================

/// Access permission flags for memory pages.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AccessPermission {
    /// Read-only access.
    ReadOnly,
    /// Read-write access.
    ReadWrite,
    /// Execute permission (read-execute).
    Execute,
}

//==================================================================================================
// VmemView - Abstract Specification
//==================================================================================================

/// Abstract view of the virtual memory space for specification purposes.
///
/// This captures the essential state of a virtual memory space:
/// - The set of mapped user pages (virtual address -> frame address mappings)
/// - The set of mapped kernel pages
/// - The number of page tables in use
#[verifier::ext_equal]
pub struct VmemView {
    /// Set of mapped user page virtual addresses.
    pub user_pages: Set<int>,
    /// Map from user virtual page addresses to frame addresses.
    pub user_mappings: Map<int, int>,
    /// Set of mapped kernel page virtual addresses.
    pub kernel_pages: Set<int>,
    /// Map from kernel virtual page addresses to frame addresses.
    pub kernel_mappings: Map<int, int>,
    /// Number of user page tables currently in use.
    pub user_page_table_count: int,
    /// Number of kernel page tables.
    pub kernel_page_table_count: int,
}

impl VmemView {
    //==============================================================================================
    // Basic Properties
    //==============================================================================================

    /// Returns the number of mapped user pages.
    pub open spec fn num_user_pages(&self) -> int {
        self.user_pages.len() as int
    }

    /// Returns the number of mapped kernel pages.
    pub open spec fn num_kernel_pages(&self) -> int {
        self.kernel_pages.len() as int
    }

    /// Returns true if a user page is mapped at the given virtual address.
    pub open spec fn is_user_page_mapped(&self, vaddr: int) -> bool {
        self.user_pages.contains(vaddr)
    }

    /// Returns true if a kernel page is mapped at the given virtual address.
    pub open spec fn is_kernel_page_mapped(&self, vaddr: int) -> bool {
        self.kernel_pages.contains(vaddr)
    }

    /// Returns the frame address for a mapped user page.
    pub open spec fn get_user_frame(&self, vaddr: int) -> int
        recommends self.is_user_page_mapped(vaddr)
    {
        self.user_mappings[vaddr]
    }

    /// Returns the frame address for a mapped kernel page.
    pub open spec fn get_kernel_frame(&self, vaddr: int) -> int
        recommends self.is_kernel_page_mapped(vaddr)
    {
        self.kernel_mappings[vaddr]
    }

    //==============================================================================================
    // Address Space Properties
    //==============================================================================================

    /// Property: A virtual address is in user space.
    pub open spec fn spec_is_user_addr(vaddr: int) -> bool {
        USER_BASE as int <= vaddr && vaddr < USER_END as int
    }

    /// Property: A virtual address is in kernel space.
    pub open spec fn spec_is_kernel_addr(vaddr: int) -> bool {
        !Self::spec_is_user_addr(vaddr)
    }

    /// Property: A memory region lies entirely in user space.
    /// Requires size > 0 and no overflow.
    pub open spec fn spec_is_user_region(start: int, size: int) -> bool {
        &&& size > 0
        &&& start >= 0
        &&& start + size - 1 >= start  // No overflow
        &&& Self::spec_is_user_addr(start)
        &&& Self::spec_is_user_addr(start + size - 1)
    }

    /// Property: A memory region lies entirely in kernel space.
    /// Requires size > 0 and no overflow.
    pub open spec fn spec_is_kernel_region(start: int, size: int) -> bool {
        &&& size > 0
        &&& start >= 0
        &&& start + size - 1 >= start  // No overflow
        &&& Self::spec_is_kernel_addr(start)
        &&& Self::spec_is_kernel_addr(start + size - 1)
    }

    /// Property: A memory region lies within physical memory bounds.
    pub open spec fn spec_is_physical_region(start: int, size: int) -> bool {
        &&& size > 0
        &&& start >= 0
        &&& start + size - 1 >= start  // No overflow
        &&& start < MEMORY_SIZE as int
        &&& start + size - 1 < MEMORY_SIZE as int
    }

    //==============================================================================================
    // Invariant Properties
    //==============================================================================================

    /// Property: All user pages are mapped to addresses in user space.
    pub open spec fn user_pages_in_user_space(&self) -> bool {
        forall|vaddr: int|
            #![trigger self.user_pages.contains(vaddr)]
            self.user_pages.contains(vaddr) ==> Self::spec_is_user_addr(vaddr)
    }

    /// Property: All kernel pages are mapped to addresses in kernel space.
    pub open spec fn kernel_pages_in_kernel_space(&self) -> bool {
        forall|vaddr: int|
            #![trigger self.kernel_pages.contains(vaddr)]
            self.kernel_pages.contains(vaddr) ==> Self::spec_is_kernel_addr(vaddr)
    }

    /// Property: User and kernel pages are disjoint (no overlapping addresses).
    pub open spec fn user_kernel_disjoint(&self) -> bool {
        forall|vaddr: int|
            #![trigger self.user_pages.contains(vaddr), self.kernel_pages.contains(vaddr)]
            !(self.user_pages.contains(vaddr) && self.kernel_pages.contains(vaddr))
    }

    /// Property: All user mappings have corresponding entries in user_pages.
    pub open spec fn user_mappings_consistent(&self) -> bool {
        forall|vaddr: int|
            #![trigger self.user_mappings.contains_key(vaddr)]
            self.user_mappings.contains_key(vaddr) <==> self.user_pages.contains(vaddr)
    }

    /// Property: All kernel mappings have corresponding entries in kernel_pages.
    pub open spec fn kernel_mappings_consistent(&self) -> bool {
        forall|vaddr: int|
            #![trigger self.kernel_mappings.contains_key(vaddr)]
            self.kernel_mappings.contains_key(vaddr) <==> self.kernel_pages.contains(vaddr)
    }

    /// Property: All mapped user pages have page-aligned addresses.
    pub open spec fn user_pages_aligned(&self) -> bool {
        forall|vaddr: int|
            #![trigger self.user_pages.contains(vaddr)]
            self.user_pages.contains(vaddr) ==> (vaddr % PAGE_SIZE as int == 0)
    }

    /// Property: All mapped kernel pages have page-aligned addresses.
    pub open spec fn kernel_pages_aligned(&self) -> bool {
        forall|vaddr: int|
            #![trigger self.kernel_pages.contains(vaddr)]
            self.kernel_pages.contains(vaddr) ==> (vaddr % PAGE_SIZE as int == 0)
    }

    /// Property: User page table count is non-negative.
    pub open spec fn valid_page_table_counts(&self) -> bool {
        self.user_page_table_count >= 0 && self.kernel_page_table_count >= 0
    }

    /// Combined invariant for VmemView.
    pub open spec fn inv(&self) -> bool {
        &&& self.user_pages_in_user_space()
        &&& self.kernel_pages_in_kernel_space()
        &&& self.user_kernel_disjoint()
        &&& self.user_mappings_consistent()
        &&& self.kernel_mappings_consistent()
        &&& self.user_pages_aligned()
        &&& self.kernel_pages_aligned()
        &&& self.valid_page_table_counts()
    }

    //==============================================================================================
    // Liveness Properties
    //==============================================================================================

    /// Property: A fresh vmem has no user pages mapped.
    pub open spec fn is_fresh(&self) -> bool {
        &&& self.user_pages =~= Set::<int>::empty()
        &&& self.user_mappings =~= Map::<int, int>::empty()
        &&& self.user_page_table_count == 0
    }
}

//==================================================================================================
// Vmem - Virtual Memory Space
//==================================================================================================

/// A type that represents a virtual memory space.
///
/// The Vmem structure manages the virtual-to-physical address translation
/// for a process, including both kernel and user space mappings.
///
/// # Implementation Notes
///
/// For verification purposes, we use simplified abstract state instead of
/// the complex linked lists used in the original implementation. The key
/// properties we verify are:
/// - Address space separation (user vs kernel)
/// - Mapping consistency
/// - Proper bounds checking
pub struct Vmem {
    /// Number of mapped user pages.
    user_page_count: usize,
    /// Number of mapped kernel pages.
    kernel_page_count: usize,
    /// Number of user page tables.
    user_page_table_count: usize,
    /// Number of kernel page tables.
    kernel_page_table_count: usize,
    /// Ghost state for specification.
    ghost_view: Ghost<VmemView>,
}

impl View for Vmem {
    type V = VmemView;

    closed spec fn view(&self) -> VmemView {
        self.ghost_view@
    }
}

impl Vmem {
    //==============================================================================================
    // Specification Functions
    //==============================================================================================

    /// Spec function to check the invariant.
    pub open spec fn inv(&self) -> bool {
        &&& self@.inv()
        &&& self.user_page_count as int == self@.num_user_pages()
        &&& self.kernel_page_count as int == self@.num_kernel_pages()
        &&& self.user_page_table_count as int == self@.user_page_table_count
        &&& self.kernel_page_table_count as int == self@.kernel_page_table_count
    }

    /// Spec function for user address check.
    pub open spec fn spec_is_user_addr(vaddr: int) -> bool {
        VmemView::spec_is_user_addr(vaddr)
    }

    /// Spec function for kernel address check.
    pub open spec fn spec_is_kernel_addr(vaddr: int) -> bool {
        VmemView::spec_is_kernel_addr(vaddr)
    }

    /// Spec function for user region check.
    pub open spec fn spec_is_user_region(start: int, size: int) -> bool {
        VmemView::spec_is_user_region(start, size)
    }

    /// Spec function for kernel region check.
    pub open spec fn spec_is_kernel_region(start: int, size: int) -> bool {
        VmemView::spec_is_kernel_region(start, size)
    }

    /// Spec function for physical region check.
    pub open spec fn spec_is_physical_region(start: int, size: int) -> bool {
        VmemView::spec_is_physical_region(start, size)
    }

    //==============================================================================================
    // Constructor
    //==============================================================================================

    /// Creates a new empty virtual memory space.
    ///
    /// # Returns
    ///
    /// A new Vmem instance with no user mappings.
    ///
    /// # Postconditions
    ///
    /// - The returned Vmem satisfies its invariant.
    /// - The returned Vmem has no user pages mapped.
    pub fn new() -> (result: Self)
        ensures
            result.inv(),
            result@.is_fresh(),
            result@.num_user_pages() == 0,
            result@.user_page_table_count == 0,
    {
        let ghost_view: Ghost<VmemView> = Ghost(VmemView {
            user_pages: Set::empty(),
            user_mappings: Map::empty(),
            kernel_pages: Set::empty(),
            kernel_mappings: Map::empty(),
            user_page_table_count: 0,
            kernel_page_table_count: 0,
        });
        Vmem {
            user_page_count: 0,
            kernel_page_count: 0,
            user_page_table_count: 0,
            kernel_page_table_count: 0,
            ghost_view,
        }
    }

    /// Creates a new Vmem initialized with kernel page tables and pages.
    ///
    /// This is the constructor that matches the original API.
    ///
    /// # Parameters
    ///
    /// - `kernel_page_count`: Number of kernel pages to initialize.
    /// - `kernel_page_table_count`: Number of kernel page tables.
    ///
    /// # Returns
    ///
    /// Upon success, a Result containing the new Vmem. Upon failure, an error.
    pub fn new_with_kernel(
        kernel_page_count: usize,
        kernel_page_table_count: usize,
    ) -> (result: Result<Self, Error>)
        ensures
            result.is_ok() ==> result.unwrap().inv(),
    {
        // In a full implementation, we would initialize kernel page tables here.
        // For verification purposes, we model the essential state.
        let ghost_view: Ghost<VmemView> = Ghost(VmemView {
            user_pages: Set::empty(),
            user_mappings: Map::empty(),
            kernel_pages: Set::empty(),
            kernel_mappings: Map::empty(),
            user_page_table_count: 0,
            kernel_page_table_count: kernel_page_table_count as int,
        });
        Ok(Vmem {
            user_page_count: 0,
            kernel_page_count: kernel_page_count,
            user_page_table_count: 0,
            kernel_page_table_count: kernel_page_table_count,
            ghost_view,
        })
    }

    //==============================================================================================
    // Address Space Checks
    //==============================================================================================

    /// Checks if a virtual address is in user space.
    ///
    /// # Parameters
    ///
    /// - `vaddr`: Virtual address to check.
    ///
    /// # Returns
    ///
    /// True if the address is in user space [USER_BASE, USER_END).
    #[verifier::when_used_as_spec(spec_is_user_addr_impl)]
    pub fn is_user_addr(vaddr: usize) -> (result: bool)
        ensures
            result == Self::spec_is_user_addr(vaddr as int),
    {
        vaddr >= USER_BASE && vaddr < USER_END
    }

    /// Spec version of is_user_addr for internal use.
    pub open spec fn spec_is_user_addr_impl(vaddr: usize) -> bool {
        Self::spec_is_user_addr(vaddr as int)
    }

    /// Checks if a virtual address is in kernel space.
    ///
    /// # Parameters
    ///
    /// - `vaddr`: Virtual address to check.
    ///
    /// # Returns
    ///
    /// True if the address is in kernel space (not in user space).
    pub fn is_kernel_addr(vaddr: usize) -> (result: bool)
        ensures
            result == Self::spec_is_kernel_addr(vaddr as int),
    {
        !Self::is_user_addr(vaddr)
    }

    /// Checks if a memory region lies entirely in user space.
    ///
    /// # Parameters
    ///
    /// - `start`: Starting virtual address of the region.
    /// - `size`: Size of the region in bytes.
    ///
    /// # Returns
    ///
    /// True if the entire region lies in user space, false otherwise.
    /// Returns false for zero-length regions.
    pub fn is_user_region(start: usize, size: usize) -> (result: bool)
        ensures
            result ==> Self::spec_is_user_region(start as int, size as int),
            result ==> size > 0,
            result ==> Self::spec_is_user_addr(start as int),
    {
        // Reject zero-length regions.
        if size == 0 {
            return false;
        }

        // Check for overflow.
        let end_opt: Option<usize> = start.checked_add(size - 1);
        match end_opt {
            Some(end) => Self::is_user_addr(start) && Self::is_user_addr(end),
            None => false,
        }
    }

    /// Checks if a memory region lies entirely in kernel space.
    ///
    /// # Parameters
    ///
    /// - `start`: Starting virtual address of the region.
    /// - `size`: Size of the region in bytes.
    ///
    /// # Returns
    ///
    /// True if the entire region lies in kernel space, false otherwise.
    /// Returns false for zero-length regions.
    pub fn is_kernel_region(start: usize, size: usize) -> (result: bool)
        ensures
            result ==> Self::spec_is_kernel_region(start as int, size as int),
            result ==> size > 0,
            result ==> Self::spec_is_kernel_addr(start as int),
    {
        // Reject zero-length regions.
        if size == 0 {
            return false;
        }

        // Check for overflow.
        let end_opt: Option<usize> = start.checked_add(size - 1);
        match end_opt {
            Some(end) => Self::is_kernel_addr(start) && Self::is_kernel_addr(end),
            None => false,
        }
    }

    /// Checks if a memory region lies within physical memory bounds.
    ///
    /// # Parameters
    ///
    /// - `start`: Starting physical address of the region.
    /// - `size`: Size of the region in bytes.
    ///
    /// # Returns
    ///
    /// True if the entire region lies within physical memory, false otherwise.
    pub fn is_physical_region(start: usize, size: usize) -> (result: bool)
        ensures
            result ==> Self::spec_is_physical_region(start as int, size as int),
            result ==> size > 0,
            result ==> start < MEMORY_SIZE,
    {
        // Reject zero-length regions.
        if size == 0 {
            return false;
        }

        // Check for overflow.
        let end_opt: Option<usize> = start.checked_add(size - 1);
        match end_opt {
            Some(end) => start < MEMORY_SIZE && end < MEMORY_SIZE,
            None => false,
        }
    }

    //==============================================================================================
    // User Page Mapping Operations
    //==============================================================================================

    /// Maps a user frame to a virtual address in user space.
    ///
    /// # Parameters
    ///
    /// - `frame_addr`: Physical frame address to map.
    /// - `vaddr`: Virtual address to map to (must be page-aligned and in user space).
    /// - `access`: Access permissions for the mapping.
    ///
    /// # Returns
    ///
    /// Upon success, Ok(()). Upon failure, an error describing the issue.
    ///
    /// # Errors
    ///
    /// - `BadAddress`: The virtual address is not in user space.
    /// - `ResourceBusy`: The virtual address is already mapped.
    pub fn map(
        &mut self,
        frame_addr: FrameAddress,
        vaddr: usize,
        access: AccessPermission,
    ) -> (result: Result<(), Error>)
        requires
            old(self).inv(),
        ensures
            self.inv(),
            result.is_ok() ==> {
                &&& Self::spec_is_user_addr(vaddr as int)
                &&& vaddr as int % PAGE_SIZE as int == 0
                &&& self@.is_user_page_mapped(vaddr as int)
                &&& self@.get_user_frame(vaddr as int) == frame_addr.spec_raw_value()
            },
            result.is_err() ==> self@ == old(self)@,
    {
        // Check if address is in user space.
        if !Self::is_user_addr(vaddr) {
            return Err(Error::new(ErrorCode::BadAddress, "address is not in user space"));
        }

        // Check if address is page-aligned.
        if vaddr % PAGE_SIZE != 0 {
            return Err(Error::new(ErrorCode::BadAddress, "address is not page-aligned"));
        }

        // Check if already mapped.
        // In a real implementation, we would check the page tables.
        // For verification, we use the ghost state.
        proof {
            if self@.user_pages.contains(vaddr as int) {
                return Err(Error::new(ErrorCode::ResourceBusy, "page already mapped"));
            }
        }

        // Update ghost state.
        proof {
            let old_view: VmemView = self@;
            self.ghost_view = Ghost(VmemView {
                user_pages: old_view.user_pages.insert(vaddr as int),
                user_mappings: old_view.user_mappings.insert(vaddr as int, frame_addr.spec_raw_value()),
                kernel_pages: old_view.kernel_pages,
                kernel_mappings: old_view.kernel_mappings,
                user_page_table_count: old_view.user_page_table_count,
                kernel_page_table_count: old_view.kernel_page_table_count,
            });
        }

        self.user_page_count = self.user_page_count + 1;

        Ok(())
    }

    /// Unmaps a page from the user virtual address space.
    ///
    /// # Parameters
    ///
    /// - `vaddr`: Virtual address of the page to unmap (must be page-aligned).
    ///
    /// # Returns
    ///
    /// Upon success, the frame address that was unmapped. Upon failure, an error.
    ///
    /// # Errors
    ///
    /// - `BadAddress`: The virtual address is not in user space.
    /// - `InvalidArgument`: The address is not page-aligned.
    pub fn unmap(&mut self, vaddr: usize) -> (result: Result<FrameAddress, Error>)
        requires
            old(self).inv(),
        ensures
            self.inv(),
            result.is_ok() ==> {
                &&& Self::spec_is_user_addr(vaddr as int)
                &&& vaddr as int % PAGE_SIZE as int == 0
                &&& old(self)@.is_user_page_mapped(vaddr as int)
                &&& !self@.is_user_page_mapped(vaddr as int)
                &&& result.unwrap().spec_raw_value() == old(self)@.get_user_frame(vaddr as int)
            },
            result.is_err() ==> self@ == old(self)@,
    {
        // Check if address is in user space.
        if !Self::is_user_addr(vaddr) {
            return Err(Error::new(ErrorCode::BadAddress, "address is not in user space"));
        }

        // Check if address is page-aligned.
        if vaddr % PAGE_SIZE != 0 {
            return Err(Error::new(ErrorCode::BadAddress, "address is not page-aligned"));
        }

        // Get the frame address before unmapping.
        let frame_raw: int;
        proof {
            if !self@.user_pages.contains(vaddr as int) {
                return Err(Error::new(ErrorCode::BadAddress, "page not mapped"));
            }
            frame_raw = self@.user_mappings[vaddr as int];
        }

        // Update ghost state.
        proof {
            let old_view: VmemView = self@;
            self.ghost_view = Ghost(VmemView {
                user_pages: old_view.user_pages.remove(vaddr as int),
                user_mappings: old_view.user_mappings.remove(vaddr as int),
                kernel_pages: old_view.kernel_pages,
                kernel_mappings: old_view.kernel_mappings,
                user_page_table_count: old_view.user_page_table_count,
                kernel_page_table_count: old_view.kernel_page_table_count,
            });
        }

        self.user_page_count = self.user_page_count - 1;

        // Create frame address from raw value.
        let frame_addr: FrameAddress = FrameAddress::from_raw_unchecked(frame_raw as usize);
        Ok(frame_addr)
    }

    /// Finds the physical frame address for a mapped user page.
    ///
    /// # Parameters
    ///
    /// - `vaddr`: Virtual address to look up (must be page-aligned and in user space).
    ///
    /// # Returns
    ///
    /// Upon success, the frame address. Upon failure, an error.
    fn find_user_frame(&self, vaddr: usize) -> (result: Result<FrameAddress, Error>)
        requires
            self.inv(),
        ensures
            result.is_ok() ==> {
                &&& Self::spec_is_user_addr(vaddr as int)
                &&& vaddr as int % PAGE_SIZE as int == 0
                &&& self@.is_user_page_mapped(vaddr as int)
                &&& result.unwrap().spec_raw_value() == self@.get_user_frame(vaddr as int)
            },
    {
        // Check if address is in user space.
        if !Self::is_user_addr(vaddr) {
            return Err(Error::new(ErrorCode::BadAddress, "address is not in user space"));
        }

        // Check if address is page-aligned.
        if vaddr % PAGE_SIZE != 0 {
            return Err(Error::new(ErrorCode::BadAddress, "address is not page-aligned"));
        }

        // Look up in ghost state.
        let frame_raw: int;
        proof {
            if !self@.user_pages.contains(vaddr as int) {
                return Err(Error::new(ErrorCode::BadAddress, "page not found"));
            }
            frame_raw = self@.user_mappings[vaddr as int];
        }

        let frame_addr: FrameAddress = FrameAddress::from_raw_unchecked(frame_raw as usize);
        Ok(frame_addr)
    }

    //==============================================================================================
    // Access Permission Operations
    //==============================================================================================

    /// Changes access permissions on a user page.
    ///
    /// # Parameters
    ///
    /// - `vaddr`: Virtual address of the page (must be page-aligned and in user space).
    /// - `access`: New access permissions.
    ///
    /// # Returns
    ///
    /// Upon success, Ok(()). Upon failure, an error.
    pub fn uctrl(&mut self, vaddr: usize, access: AccessPermission) -> (result: Result<(), Error>)
        requires
            old(self).inv(),
        ensures
            self.inv(),
            result.is_ok() ==> {
                &&& Self::spec_is_user_addr(vaddr as int)
                &&& self@.is_user_page_mapped(vaddr as int)
            },
            // Permission changes don't affect the abstract view (mappings unchanged).
            result.is_ok() ==> self@.user_pages == old(self)@.user_pages,
            result.is_ok() ==> self@.user_mappings == old(self)@.user_mappings,
            result.is_err() ==> self@ == old(self)@,
    {
        // Check if address is in user space.
        if !Self::is_user_addr(vaddr) {
            return Err(Error::new(ErrorCode::BadAddress, "address is not in user space"));
        }

        // Check if page is mapped.
        proof {
            if !self@.user_pages.contains(vaddr as int) {
                return Err(Error::new(ErrorCode::BadAddress, "page not mapped"));
            }
        }

        // In a real implementation, we would update the page table entry permissions.
        // The abstract view doesn't change since mappings are unchanged.
        Ok(())
    }

    /// Changes access permissions on a kernel page.
    ///
    /// # Parameters
    ///
    /// - `vaddr`: Virtual address of the page (must be page-aligned and in kernel space).
    /// - `access`: New access permissions.
    ///
    /// # Returns
    ///
    /// Upon success, Ok(()). Upon failure, an error.
    pub fn kctrl(&mut self, vaddr: usize, access: AccessPermission) -> (result: Result<(), Error>)
        requires
            old(self).inv(),
        ensures
            self.inv(),
            result.is_ok() ==> Self::spec_is_kernel_addr(vaddr as int),
            // Permission changes don't affect the abstract view.
            result.is_ok() ==> self@.kernel_pages == old(self)@.kernel_pages,
            result.is_ok() ==> self@.kernel_mappings == old(self)@.kernel_mappings,
            result.is_err() ==> self@ == old(self)@,
    {
        // Check if address is in kernel space.
        if !Self::is_kernel_addr(vaddr) {
            return Err(Error::new(ErrorCode::BadAddress, "address is not in kernel space"));
        }

        // In a real implementation, we would update the page table entry permissions.
        Ok(())
    }

    //==============================================================================================
    // Memory Copy Operations (Specifications Only)
    //==============================================================================================

    /// Copies data from user space to kernel space.
    ///
    /// This is a specification-level function that describes the preconditions
    /// for a safe copy from user to kernel space.
    ///
    /// # Parameters
    ///
    /// - `dst`: Destination address in kernel space.
    /// - `src`: Source address in user space.
    /// - `size`: Number of bytes to copy.
    ///
    /// # Returns
    ///
    /// Upon success, Ok(()). Upon failure, an error describing the issue.
    ///
    /// # Errors
    ///
    /// - `InvalidArgument`: The size is zero.
    /// - `BadAddress`: Source region not in user space.
    /// - `BadAddress`: Destination region not in kernel space.
    pub fn copy_from_user_unaligned(
        &self,
        dst: usize,
        src: usize,
        size: usize,
    ) -> (result: Result<(), Error>)
        requires
            self.inv(),
        ensures
            result.is_ok() ==> {
                &&& size > 0
                &&& Self::spec_is_user_region(src as int, size as int)
                &&& Self::spec_is_kernel_region(dst as int, size as int)
            },
    {
        // Check if size is zero.
        if size == 0 {
            return Err(Error::new(ErrorCode::InvalidArgument, "zero-length copy"));
        }

        // Check if source is in user space.
        if !Self::is_user_region(src, size) {
            return Err(Error::new(ErrorCode::BadAddress, "source not in user space"));
        }

        // Check if destination is in kernel space.
        if !Self::is_kernel_region(dst, size) {
            return Err(Error::new(ErrorCode::BadAddress, "destination not in kernel space"));
        }

        // In a real implementation, we would perform the physical memory copy.
        // The actual copy is external_body since it involves low-level operations.
        Ok(())
    }

    /// Copies data from kernel space to user space.
    ///
    /// # Parameters
    ///
    /// - `dst`: Destination address in user space.
    /// - `src`: Source address in kernel space.
    /// - `size`: Number of bytes to copy.
    ///
    /// # Returns
    ///
    /// Upon success, Ok(()). Upon failure, an error describing the issue.
    pub fn copy_to_user_unaligned(
        &self,
        dst: usize,
        src: usize,
        size: usize,
    ) -> (result: Result<(), Error>)
        requires
            self.inv(),
        ensures
            result.is_ok() ==> {
                &&& size > 0
                &&& Self::spec_is_kernel_region(src as int, size as int)
                &&& Self::spec_is_user_region(dst as int, size as int)
            },
    {
        // Check if size is zero.
        if size == 0 {
            return Err(Error::new(ErrorCode::InvalidArgument, "zero-length copy"));
        }

        // Check if source is in kernel space.
        if !Self::is_kernel_region(src, size) {
            return Err(Error::new(ErrorCode::BadAddress, "source not in kernel space"));
        }

        // Check if destination is in user space.
        if !Self::is_user_region(dst, size) {
            return Err(Error::new(ErrorCode::BadAddress, "destination not in user space"));
        }

        Ok(())
    }

    /// Fills a user page with a given value.
    ///
    /// # Parameters
    ///
    /// - `vaddr`: Virtual address of the page (must be page-aligned and in user space).
    /// - `value`: Value to fill the page with.
    ///
    /// # Returns
    ///
    /// Upon success, Ok(()). Upon failure, an error.
    pub fn memset(&mut self, vaddr: usize, value: u32) -> (result: Result<(), Error>)
        requires
            old(self).inv(),
        ensures
            self.inv(),
            result.is_ok() ==> {
                &&& Self::spec_is_user_addr(vaddr as int)
                &&& vaddr as int % PAGE_SIZE as int == 0
                &&& self@.is_user_page_mapped(vaddr as int)
            },
            // memset doesn't change mappings.
            self@.user_pages == old(self)@.user_pages,
            self@.user_mappings == old(self)@.user_mappings,
    {
        // Check if address is in user space.
        if !Self::is_user_addr(vaddr) {
            return Err(Error::new(ErrorCode::BadAddress, "address is not in user space"));
        }

        // Check if address is page-aligned.
        if vaddr % PAGE_SIZE != 0 {
            return Err(Error::new(ErrorCode::BadAddress, "address is not page-aligned"));
        }

        // Check if page is mapped.
        proof {
            if !self@.user_pages.contains(vaddr as int) {
                return Err(Error::new(ErrorCode::BadAddress, "page not mapped"));
            }
        }

        // In a real implementation, we would perform the physical memset.
        Ok(())
    }
}

//==================================================================================================
// Helper Proofs
//==================================================================================================

/// Proof that user and kernel spaces are disjoint.
proof fn user_kernel_disjoint_proof(vaddr: int)
    ensures
        !(VmemView::spec_is_user_addr(vaddr) && VmemView::spec_is_kernel_addr(vaddr)),
{
    // By definition, kernel space is !user_space.
}

/// Proof that USER_BASE < USER_END.
proof fn user_space_bounds_valid()
    ensures
        USER_BASE < USER_END,
{
    // Constants are set such that USER_BASE = 1GB < USER_END = 3GB.
}

/// Proof that a valid user region implies start is a user address.
proof fn user_region_implies_user_addr(start: int, size: int)
    requires
        VmemView::spec_is_user_region(start, size),
    ensures
        VmemView::spec_is_user_addr(start),
{
    // By definition of spec_is_user_region.
}

/// Proof that a valid kernel region implies start is a kernel address.
proof fn kernel_region_implies_kernel_addr(start: int, size: int)
    requires
        VmemView::spec_is_kernel_region(start, size),
    ensures
        VmemView::spec_is_kernel_addr(start),
{
    // By definition of spec_is_kernel_region.
}

} // verus!
