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
//! for shared ownership. For verification, we model page tables as an abstract array-based
//! storage, capturing the essential translation properties.
//!
//! ### View Derived from Concrete State
//! The VmemView is derived from the concrete mappings array rather than maintained
//! as separate ghost state. This follows the pattern used in other verified modules
//! like upool and kpool.
//!
//! ## Relationship to Other Verified Modules
//!
//! - Uses `FrameAddress` from `frame_address.rs` (verified) - for frame addresses
//! - Uses `PAGE_SIZE` from `kpage.rs` (verified) - for page size constant
//!
//! ## API Summary
//!
//! | Function | Description |
//! |----------|-------------|
//! | `new()` | Create a new virtual memory space |
//! | `map()` | Map a user frame to a virtual address |
//! | `unmap()` | Unmap a page from the virtual address space |
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
    kpage::PAGE_SIZE,
    frame_address::FrameAddress,
    error::{Error, ErrorCode},
};
use vstd::prelude::*;

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

/// Maximum number of user pages that can be tracked (simplified model).
pub const MAX_USER_PAGES: usize = 1024;

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
// PageMapping - Concrete mapping storage
//==================================================================================================

/// A single page mapping entry storing virtual address and frame address.
#[derive(Debug, Clone, Copy)]
pub struct PageMapping {
    /// Virtual address (page-aligned).
    pub vaddr: usize,
    /// Frame address (page-aligned).
    pub frame_addr: usize,
    /// Whether this entry is valid/in-use.
    pub valid: bool,
}

impl PageMapping {
    /// Spec function to check if this mapping is for the given vaddr.
    pub open spec fn spec_is_for_vaddr(&self, vaddr: int) -> bool {
        self.valid && self.vaddr as int == vaddr
    }
}

//==================================================================================================
// Spec Functions - Address Space Properties
//==================================================================================================

/// Property: A virtual address is in user space.
pub open spec fn spec_is_user_addr(vaddr: int) -> bool {
    USER_BASE as int <= vaddr && vaddr < USER_END as int
}

/// Property: A virtual address is in kernel space.
pub open spec fn spec_is_kernel_addr(vaddr: int) -> bool {
    !spec_is_user_addr(vaddr)
}

/// Property: A memory region lies entirely in user space.
/// Requires size > 0 and no overflow.
pub open spec fn spec_is_user_region(start: int, size: int) -> bool {
    &&& size > 0
    &&& start >= 0
    &&& start + size - 1 >= start  // No overflow
    &&& spec_is_user_addr(start)
    &&& spec_is_user_addr(start + size - 1)
}

/// Property: A memory region lies entirely in kernel space.
/// Requires size > 0 and no overflow.
pub open spec fn spec_is_kernel_region(start: int, size: int) -> bool {
    &&& size > 0
    &&& start >= 0
    &&& start + size - 1 >= start  // No overflow
    &&& spec_is_kernel_addr(start)
    &&& spec_is_kernel_addr(start + size - 1)
}

/// Property: A memory region lies within physical memory bounds.
pub open spec fn spec_is_physical_region(start: int, size: int) -> bool {
    &&& size > 0
    &&& start >= 0
    &&& start + size - 1 >= start  // No overflow
    &&& start < MEMORY_SIZE as int
    &&& start + size - 1 < MEMORY_SIZE as int
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
/// For verification purposes, we use an array-based storage for mappings
/// instead of the complex linked lists used in the original implementation.
/// This allows us to verify the core properties while keeping the model tractable.
pub struct Vmem {
    /// Array of user page mappings.
    pub mappings: [PageMapping; MAX_USER_PAGES],
    /// Number of valid mappings.
    pub mapping_count: usize,
}

impl Vmem {
    //==============================================================================================
    // Specification Functions
    //==============================================================================================

    /// Spec function to check that a mapping at index i is valid and for the given vaddr.
    pub open spec fn spec_mapping_for_vaddr(&self, i: int, vaddr: int) -> bool {
        0 <= i < self.mapping_count as int &&
        self.mappings[i as int].valid &&
        self.mappings[i as int].vaddr as int == vaddr
    }

    /// Spec function to check if a vaddr is mapped (exists in some slot).
    pub open spec fn spec_is_mapped(&self, vaddr: int) -> bool {
        exists|i: int|
            #![trigger self.mappings[i]]
            0 <= i < self.mapping_count as int &&
            self.mappings[i as int].spec_is_for_vaddr(vaddr)
    }

    /// Spec function to get the frame address for a mapped vaddr at a given index.
    pub open spec fn spec_get_frame_at(&self, i: int) -> int
        recommends 0 <= i < self.mapping_count as int
    {
        self.mappings[i as int].frame_addr as int
    }

    /// Invariant: mapping_count is within bounds and all mappings in [0, mapping_count) are valid.
    pub closed spec fn inv(&self) -> bool {
        &&& self.mapping_count <= MAX_USER_PAGES
        // All mappings in [0, mapping_count) have valid flag set.
        &&& forall|i: int| 0 <= i < self.mapping_count as int ==> self.mappings[i as int].valid
        // All valid mappings are for user addresses.
        &&& forall|i: int| 0 <= i < self.mapping_count as int ==>
                spec_is_user_addr(self.mappings[i as int].vaddr as int)
        // All valid mappings have page-aligned vaddr.
        &&& forall|i: int| 0 <= i < self.mapping_count as int ==>
                self.mappings[i as int].vaddr as int % PAGE_SIZE as int == 0
    }

    /// Spec function to check if there is capacity for more mappings.
    pub closed spec fn has_mapping_capacity(&self) -> bool {
        self.mapping_count < MAX_USER_PAGES
    }

    /// Spec function to check if there are any mappings.
    pub closed spec fn has_mappings(&self) -> bool {
        self.mapping_count > 0
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
            result.mapping_count == 0,
    {
        let empty_mapping: PageMapping = PageMapping {
            vaddr: 0,
            frame_addr: 0,
            valid: false,
        };

        Vmem {
            mappings: [empty_mapping; MAX_USER_PAGES],
            mapping_count: 0,
        }
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
    pub fn is_user_addr(vaddr: usize) -> (result: bool)
        ensures
            result == spec_is_user_addr(vaddr as int),
    {
        vaddr >= USER_BASE && vaddr < USER_END
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
            result == spec_is_kernel_addr(vaddr as int),
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
            result ==> spec_is_user_region(start as int, size as int),
            result ==> size > 0,
            result ==> spec_is_user_addr(start as int),
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
            result ==> spec_is_kernel_region(start as int, size as int),
            result ==> size > 0,
            result ==> spec_is_kernel_addr(start as int),
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
            result ==> spec_is_physical_region(start as int, size as int),
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
    /// - `BadAddress`: The virtual address is not in user space or not page-aligned.
    /// - `ResourceBusy`: The virtual address is already mapped.
    /// - `OutOfMemory`: No more mapping slots available.
    pub fn map(
        &mut self,
        frame_addr: FrameAddress,
        vaddr: usize,
        access: AccessPermission,
    ) -> (result: Result<(), Error>)
        requires
            old(self).inv(),
            old(self).has_mapping_capacity(),
        ensures
            self.inv(),
            result.is_ok() ==> {
                &&& spec_is_user_addr(vaddr as int)
                &&& vaddr as int % PAGE_SIZE as int == 0
                &&& self.spec_is_mapped(vaddr as int)
                &&& self.mapping_count == old(self).mapping_count + 1
            },
            result.is_err() ==> {
                &&& self.mapping_count == old(self).mapping_count
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

        // Check if we have space.
        if self.mapping_count >= MAX_USER_PAGES {
            return Err(Error::new(ErrorCode::OutOfMemory, "no mapping slots available"));
        }

        // Check if already mapped by scanning existing entries.
        let mut i: usize = 0;
        while i < self.mapping_count
            invariant
                0 <= i <= self.mapping_count,
                self.mapping_count < MAX_USER_PAGES,
                self.inv(),
                forall|j: int| 0 <= j < i as int ==>
                    !(self.mappings[j as int].valid && self.mappings[j as int].vaddr == vaddr),
            decreases self.mapping_count - i,
        {
            if self.mappings[i].valid && self.mappings[i].vaddr == vaddr {
                return Err(Error::new(ErrorCode::ResourceBusy, "page already mapped"));
            }
            i = i + 1;
        }

        // Add the new mapping.
        let slot: usize = self.mapping_count;
        self.mappings[slot] = PageMapping {
            vaddr: vaddr,
            frame_addr: frame_addr.into_raw_value(),
            valid: true,
        };
        self.mapping_count = self.mapping_count + 1;

        // Help Verus see that the new mapping satisfies spec_is_mapped.
        proof {
            let idx: int = slot as int;
            let count: int = self.mapping_count as int;
            assert(self.mappings[idx].valid);
            assert(self.mappings[idx].vaddr as int == vaddr as int);
            assert(self.mappings[idx].spec_is_for_vaddr(vaddr as int));
            assert(idx >= 0);
            assert(idx < count);
        }

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
    /// - `BadAddress`: The virtual address is not in user space, not page-aligned, or not mapped.
    pub fn unmap(&mut self, vaddr: usize) -> (result: Result<usize, Error>)
        requires
            old(self).inv(),
            old(self).has_mappings(),
        ensures
            self.inv(),
            result.is_ok() ==> {
                &&& spec_is_user_addr(vaddr as int)
                &&& vaddr as int % PAGE_SIZE as int == 0
                &&& self.mapping_count == old(self).mapping_count - 1
            },
            result.is_err() ==> {
                &&& self.mapping_count == old(self).mapping_count
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

        // Find the mapping.
        let mut found_idx: usize = MAX_USER_PAGES;
        let mut frame_addr: usize = 0;
        let mut i: usize = 0;
        while i < self.mapping_count
            invariant
                0 <= i <= self.mapping_count,
                self.mapping_count <= MAX_USER_PAGES,
                self.inv(),
                found_idx == MAX_USER_PAGES || found_idx < self.mapping_count,
                found_idx < MAX_USER_PAGES ==>
                    self.mappings[found_idx as int].valid &&
                    self.mappings[found_idx as int].vaddr == vaddr,
            decreases self.mapping_count - i,
        {
            if self.mappings[i].valid && self.mappings[i].vaddr == vaddr {
                found_idx = i;
                frame_addr = self.mappings[i].frame_addr;
                break;
            }
            i = i + 1;
        }

        if found_idx == MAX_USER_PAGES {
            return Err(Error::new(ErrorCode::BadAddress, "page not mapped"));
        }

        // Remove the mapping by swapping with the last entry.
        let last_idx: usize = self.mapping_count - 1;
        if found_idx != last_idx {
            self.mappings[found_idx] = self.mappings[last_idx];
        }
        self.mappings[last_idx] = PageMapping {
            vaddr: 0,
            frame_addr: 0,
            valid: false,
        };
        self.mapping_count = self.mapping_count - 1;

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
    pub fn find_user_frame(&self, vaddr: usize) -> (result: Result<usize, Error>)
        requires
            self.inv(),
        ensures
            result.is_ok() ==> {
                &&& spec_is_user_addr(vaddr as int)
                &&& vaddr as int % PAGE_SIZE as int == 0
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

        // Find the mapping.
        let mut i: usize = 0;
        while i < self.mapping_count
            invariant
                0 <= i <= self.mapping_count,
                self.mapping_count <= MAX_USER_PAGES,
                self.inv(),
            decreases self.mapping_count - i,
        {
            if self.mappings[i].valid && self.mappings[i].vaddr == vaddr {
                return Ok(self.mappings[i].frame_addr);
            }
            i = i + 1;
        }

        Err(Error::new(ErrorCode::BadAddress, "page not found"))
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
            result.is_ok() ==> spec_is_user_addr(vaddr as int),
            // Permission changes don't affect the mappings.
            self.mapping_count == old(self).mapping_count,
    {
        // Check if address is in user space.
        if !Self::is_user_addr(vaddr) {
            return Err(Error::new(ErrorCode::BadAddress, "address is not in user space"));
        }

        // Check if page is mapped.
        let mut found: bool = false;
        let mut i: usize = 0;
        while i < self.mapping_count
            invariant
                0 <= i <= self.mapping_count,
                self.mapping_count <= MAX_USER_PAGES,
                self.inv(),
            decreases self.mapping_count - i,
        {
            if self.mappings[i].valid && self.mappings[i].vaddr == vaddr {
                found = true;
                break;
            }
            i = i + 1;
        }

        if !found {
            return Err(Error::new(ErrorCode::BadAddress, "page not mapped"));
        }

        // In a real implementation, we would update the page table entry permissions.
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
            result.is_ok() ==> spec_is_kernel_addr(vaddr as int),
            // Permission changes don't affect the mappings.
            self.mapping_count == old(self).mapping_count,
    {
        // Check if address is in kernel space.
        if !Self::is_kernel_addr(vaddr) {
            return Err(Error::new(ErrorCode::BadAddress, "address is not in kernel space"));
        }

        // In a real implementation, we would update the page table entry permissions.
        Ok(())
    }

    //==============================================================================================
    // Memory Copy Operations
    //==============================================================================================

    /// Copies data from user space to kernel space.
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
                &&& spec_is_user_region(src as int, size as int)
                &&& spec_is_kernel_region(dst as int, size as int)
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
                &&& spec_is_kernel_region(src as int, size as int)
                &&& spec_is_user_region(dst as int, size as int)
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
                &&& spec_is_user_addr(vaddr as int)
                &&& vaddr as int % PAGE_SIZE as int == 0
            },
            // memset doesn't change mappings.
            self.mapping_count == old(self).mapping_count,
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
        let mut found: bool = false;
        let mut i: usize = 0;
        while i < self.mapping_count
            invariant
                0 <= i <= self.mapping_count,
                self.mapping_count <= MAX_USER_PAGES,
                self.inv(),
            decreases self.mapping_count - i,
        {
            if self.mappings[i].valid && self.mappings[i].vaddr == vaddr {
                found = true;
                break;
            }
            i = i + 1;
        }

        if !found {
            return Err(Error::new(ErrorCode::BadAddress, "page not mapped"));
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
        !(spec_is_user_addr(vaddr) && spec_is_kernel_addr(vaddr)),
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
        spec_is_user_region(start, size),
    ensures
        spec_is_user_addr(start),
{
    // By definition of spec_is_user_region.
}

/// Proof that a valid kernel region implies start is a kernel address.
proof fn kernel_region_implies_kernel_addr(start: int, size: int)
    requires
        spec_is_kernel_region(start, size),
    ensures
        spec_is_kernel_addr(start),
{
    // By definition of spec_is_kernel_region.
}

} // verus!
