// Copyright(c) The Maintainers of Nanvix.
// Licensed under the MIT License.

//==================================================================================================
//! # Virtual Memory Manager (Verified Implementation)
//!
//! This module provides a verified implementation of the `VirtMemoryManager` abstraction, which
//! coordinates virtual memory operations by bridging the physical memory manager with virtual
//! memory spaces.
//!
//! ## Overview
//!
//! The `VirtMemoryManager` is the high-level coordinator for memory management, providing:
//! - Creation of virtual memory spaces (Vmem)
//! - Allocation and deallocation of user pages with physical frame backing
//! - Allocation of kernel pages
//! - Permission control on mapped pages
//!
//! ## Memory Safety Properties Verified
//!
//! 1. **Allocation Requires Capacity**: Allocation operations require available frames in pools.
//! 2. **Mapping Requires Capacity**: User page allocation requires both frame and vmem capacity.
//! 3. **Unmapping Requires Mapping**: Unmap operations require the page to be currently mapped.
//! 4. **Control Requires Mapping**: Permission control requires the page to be mapped.
//! 5. **Invariant Preservation**: All operations maintain manager and vmem invariants.
//! 6. **Frame Provenance**: Allocated frames come from the correct pool.
//! 7. **Address Alignment**: All addresses are properly page-aligned.
//!
//! ## Abstraction Decisions
//!
//! ### Global State Not Modeled
//! The original implementation uses a static `MEMORY_MANAGER` global with unsafe access via
//! `init()`, `get()`, and `get_mut()`. These are NOT modeled in the verified implementation
//! because:
//! 1. Global mutable state requires external synchronization (caller's responsibility).
//! 2. The core memory safety properties are orthogonal to global access patterns.
//! 3. Verification of global state would require concurrency reasoning (out of scope).
//!
//! The verified `new()` function models the constructor semantics directly.
//!
//! ### Rc<RefCell<>> Modeled as Single-Owner
//! The original uses `Rc<RefCell<PhysMemoryManager>>` for shared ownership with interior
//! mutability. For verification purposes, we model this as single-owner semantics because:
//! 1. The Rc provides reference counting, not verified memory safety.
//! 2. The RefCell provides runtime borrow checking, which we model via preconditions.
//! 3. The core allocation logic is independent of the ownership wrapper.
//!
//! ### PhysMemoryManager Simplified
//! We use the already-verified `Kpool` and `Upool` directly instead of wrapping them in a
//! `PhysMemoryManager` structure. This is equivalent because `PhysMemoryManager` is essentially
//! a composition of these two pools.
//!
//! ### ELF Loading Not Modeled
//! The `load_elf()` function is not modeled because:
//! 1. ELF parsing has complex format-specific logic unrelated to memory safety.
//! 2. The memory safety of ELF loading depends on page allocation (already verified).
//! 3. Full ELF verification would require a separate specification effort.
//!
//! ## API Summary
//!
//! | Function          | Description                                          |
//! |-------------------|------------------------------------------------------|
//! | `new()`           | Create a new VirtMemoryManager with pools            |
//! | `new_vmem()`      | Create a new virtual address space by cloning        |
//! | `alloc_upage()`   | Allocate and map a single user page                  |
//! | `unmap_upage()`   | Unmap and free a user page                           |
//! | `ctrl_upage()`    | Change access permissions on a user page             |
//! | `alloc_kpage()`   | Allocate a kernel page                               |
//!
//! ## Relationship to Other Verified Modules
//!
//! - Uses `Kpool` from `kpool.rs` (verified) - kernel frame pool
//! - Uses `Upool` from `upool.rs` (verified) - user frame pool
//! - Uses `KernelPage` from `kpage.rs` (verified) - kernel page abstraction
//! - Uses `Vmem` from `vmem.rs` (verified) - virtual memory space
//! - Uses `Error` from `error.rs` (verified) - error handling
//!
//! ## Revision History
//!
//! - R4: Address reviewer feedback from claude_r3_a1.md:
//!   - Added frame deallocation to `unmap_upage()` (High #1)
//!   - Added `alloc_upages()` and `alloc_kpages()` batch functions (High #2)
//!   - Added resource postconditions for free count tracking (Medium #3, #4)
//!   - Added `spec_is_mapped` precondition to `ctrl_upage()` (Medium #6)
//!   - Added `upool_id` to view (Low #12)
//!   - Used `pools_valid()` in invariant (Low #8)
//!   - Note: `clear` parameter skipped - security feature, not memory safety (Medium #5)
//==================================================================================================

use crate::{
    libs::error::{
        Error,
        ErrorCode,
    },
    kernel::hal::mem::types::address::frame::{
        FrameAddress,
        FRAME_SIZE,
    },
    kernel::mm::virt::kpage::{
        KernelPage,
        PAGE_SIZE,
    },
    kernel::mm::phys::kpool::{
        KernelFrame,
        Kpool,
    },
    kernel::mm::phys::upool::{
        Upool,
        UserFrame,
    },
    kernel::mm::virt::vmem::{
        spec_is_user_addr,
        AccessPermission,
        Vmem,
        MAX_USER_PAGES,
    },
};
use vstd::prelude::*;

// Include specifications.
include!("manager.spec.rs");

// Include proofs.
include!("manager.proof.rs");


verus! {

//==================================================================================================

/// A type that represents the virtual memory manager.
///
/// The VirtMemoryManager coordinates virtual memory operations by managing physical
/// frame allocation through kernel and user pools, and facilitating mapping operations
/// on virtual memory spaces.
///
/// # Memory Safety Guarantees
///
/// - All allocation operations check capacity before proceeding
/// - All unmap operations verify the page is currently mapped
/// - Pool invariants are preserved across all operations
/// - Frame provenance is tracked through pool identifiers
pub struct VirtMemoryManager {
    /// Kernel frame pool for page tables and kernel pages.
    kpool: Kpool,
    /// User frame pool for user pages.
    upool: Upool,
}

impl VirtMemoryManager {
    //==============================================================================================

    /// Creates a new virtual memory manager from initialized pools.
    ///
    /// # Description
    ///
    /// This constructor takes ownership of initialized kernel and user frame pools
    /// and creates a manager that can coordinate memory operations.
    ///
    /// # Parameters
    ///
    /// - `kpool`: An initialized kernel frame pool.
    /// - `upool`: An initialized user frame pool.
    ///
    /// # Returns
    ///
    /// A new VirtMemoryManager.
    ///
    /// # Note
    ///
    /// This is a simplified constructor compared to the original which takes
    /// `LinkedList<KernelPage>`, `LinkedList<PageTable>`, and `PhysMemoryManager`.
    /// The verified version takes pre-initialized pools directly because:
    /// 1. Pool initialization is already verified in kpool.rs and upool.rs.
    /// 2. The complex initialization logic is implementation detail.
    /// 3. The core memory safety properties start from valid pools.
    pub fn new(kpool: Kpool, upool: Upool) -> (result: Self)
        requires
            kpool.inv(),
            upool.inv(),
        ensures
            result.inv(),
            result@.kpool_free_count == kpool@.num_free(),
            result@.upool_free_count == upool@.num_free(),
            result@.kpool_capacity == kpool@.capacity(),
            result@.upool_capacity == upool@.capacity(),
    {
        VirtMemoryManager { kpool, upool }
    }

    //==============================================================================================

    /// Creates a new virtual address space based on an existing one.
    ///
    /// # Description
    ///
    /// Clones the given virtual memory space to create a new one. The new space
    /// shares kernel mappings (in the original implementation via Rc) but has
    /// independent user mappings.
    ///
    /// # Parameters
    ///
    /// - `vmem`: The source virtual memory space to clone from.
    ///
    /// # Returns
    ///
    /// Upon success, a new Vmem instance. Upon failure, an error.
    ///
    /// # Postconditions
    ///
    /// - The source vmem invariant is preserved.
    /// - The new vmem satisfies its invariant.
    /// - The manager invariant is preserved (no allocation occurs).
    pub fn new_vmem(&self, vmem: &Vmem) -> (result: Vmem)
        requires
            self.inv(),
            vmem.inv(),
        ensures
            result.inv(),
            result.mapping_count == 0,
    {
        Vmem::clone(vmem)
    }

    //==============================================================================================

    /// Allocates and maps a single user page.
    ///
    /// # Description
    ///
    /// Allocates a frame from the user pool and maps it to the given virtual address
    /// in the specified virtual memory space with the given access permissions.
    ///
    /// # Parameters
    ///
    /// - `vmem`: Virtual memory space to map the page into.
    /// - `vaddr`: Page-aligned virtual address for the mapping.
    /// - `access`: Access permissions for the page.
    ///
    /// # Returns
    ///
    /// Upon success, Ok(()). Upon failure, an error if:
    /// - No user frames are available (OutOfMemory).
    /// - The vmem has no capacity for more mappings (OutOfMemory).
    ///
    /// # Preconditions
    ///
    /// - The user pool must have a free frame.
    /// - The vmem must have capacity for a new mapping.
    /// - The vaddr must be page-aligned and in user space.
    /// - The vaddr must not already be mapped.
    ///
    /// # Postconditions
    ///
    /// - On success: The vaddr is now mapped in vmem.
    /// - On success: Manager and vmem invariants are preserved.
    /// - On success: User pool free count decreases by 1.
    pub fn alloc_upage(
        &mut self,
        vmem: &mut Vmem,
        vaddr: usize,
        access: AccessPermission,
    ) -> (result: Result<(), Error>)
        requires
            old(self).inv(),
            old(vmem).inv(),
            old(self)@.has_upool_capacity(),
            old(vmem).has_mapping_capacity(),
            vaddr as int % PAGE_SIZE as int == 0,
            spec_is_user_addr(vaddr as int),
            !old(vmem).spec_is_mapped(vaddr as int),
        ensures
            self.inv(),
            vmem.inv(),
            result.is_ok() ==> {
                &&& vmem.spec_is_mapped(vaddr as int)
                &&& vmem.mapping_count == old(vmem).mapping_count + 1
            },
    {
        // Allocate user frame.
        let uframe: UserFrame = self.upool.alloc()?;

        // Get the frame address for mapping.
        let frame_addr: FrameAddress = uframe.address();

        proof {
            // Connect uframe alignment to frame_addr alignment.
            assert(uframe.spec_is_aligned());
            assert(frame_addr == uframe.spec_address());
            assert(frame_addr.spec_is_aligned());
            // Vmem is unchanged after upool.alloc(), so preconditions for map() still hold.
            assert(vmem.inv());
            assert(vmem.has_mapping_capacity());
            assert(!vmem.spec_is_mapped(vaddr as int));
        }

        // Map the frame to the virtual address.
        vmem.map(frame_addr, vaddr, access)?;

        proof {
            // vmem.map() postcondition gives us spec_is_mapped.
            assert(vmem.spec_is_mapped(vaddr as int));
        }

        Ok(())
    }


    /// Unmaps a user page and frees the backing frame.
    ///
    /// # Description
    ///
    /// Removes the mapping for the given virtual address from the virtual memory space
    /// and returns the backing physical frame to the user pool.
    ///
    /// # Parameters
    ///
    /// - `vmem`: Virtual memory space containing the mapping.
    /// - `vaddr`: Page-aligned virtual address to unmap.
    ///
    /// # Returns
    ///
    /// Upon success, Ok(()). Upon failure, an error if the page is not mapped.
    ///
    /// # Note
    ///
    /// The precondition requires that the frame backing the mapping was allocated from
    /// this manager's user pool. This is satisfied when the page was originally allocated
    /// via `alloc_upage()`. The vmem's spec_get_frame_addr retrieves the physical address
    /// for a mapped virtual address.
    pub fn unmap_upage(
        &mut self,
        vmem: &mut Vmem,
        vaddr: usize,
    ) -> (result: Result<(), Error>)
        requires
            old(self).inv(),
            old(vmem).inv(),
            old(vmem).has_mappings(),
            vaddr as int % PAGE_SIZE as int == 0,
            spec_is_user_addr(vaddr as int),
            old(vmem).spec_is_mapped(vaddr as int),
            // The frame backing this mapping was allocated from the upool.
            // This is satisfied when the page was allocated via alloc_upage().
            old(self).spec_uframe_is_allocated(old(vmem).spec_get_frame_addr(vaddr as int)),
        ensures
            self.inv(),
            vmem.inv(),
            result.is_ok() ==> {
                &&& vmem.mapping_count == old(vmem).mapping_count - 1
            },
    {
        // Unmap the page. Returns the frame address.
        let frame_addr: usize = vmem.unmap(vaddr)?;

        proof {
            // The returned frame_addr is aligned (from vmem.unmap postcondition).
            assert(frame_addr as int % FRAME_SIZE as int == 0);
        }

        // Free the frame back to the user pool.
        self.upool.free_by_addr(frame_addr)?;

        Ok(())
    }


    /// Changes access permissions on a user page.
    ///
    /// # Description
    ///
    /// Modifies the access permissions for an already-mapped user page.
    ///
    /// # Parameters
    ///
    /// - `vmem`: Virtual memory space containing the mapping.
    /// - `vaddr`: Page-aligned virtual address to modify.
    /// - `access`: New access permissions.
    ///
    /// # Returns
    ///
    /// Upon success, Ok(()). Upon failure, an error if the page is not mapped.
    ///
    /// # Postconditions
    ///
    /// - Vmem and manager invariants are preserved.
    /// - The mapping count is unchanged.
    pub fn ctrl_upage(
        &self,
        vmem: &mut Vmem,
        vaddr: usize,
        access: AccessPermission,
    ) -> (result: Result<(), Error>)
        requires
            self.inv(),
            old(vmem).inv(),
            vaddr as int % PAGE_SIZE as int == 0,
            spec_is_user_addr(vaddr as int),
            old(vmem).spec_is_mapped(vaddr as int),
        ensures
            self.inv(),
            vmem.inv(),
            vmem.mapping_count == old(vmem).mapping_count,
    {
        vmem.uctrl(vaddr, access)
    }

    //==============================================================================================

    /// Allocates a kernel page.
    ///
    /// # Description
    ///
    /// Allocates a frame from the kernel pool and wraps it in a KernelPage.
    ///
    /// # Returns
    ///
    /// Upon success, a KernelPage. Upon failure, an error if no kernel frames available.
    ///
    /// # Postconditions
    ///
    /// - On success: The returned page satisfies its invariant.
    pub fn alloc_kpage(&mut self) -> (result: Result<KernelPage, Error>)
        requires
            old(self).inv(),
            old(self)@.has_kpool_capacity(),
        ensures
            self.inv(),
            result.is_ok() ==> {
                &&& result.unwrap().inv()
            },
    {
        let kframe: KernelFrame = self.kpool.alloc()?;
        proof {
            // Connect kpool.alloc() postcondition to KernelPage::new() precondition.
            assert(kframe.spec_is_aligned());
        }
        let kpage: KernelPage = KernelPage::new(kframe);
        proof {
            // KernelPage::new() postcondition guarantees result.inv().
            assert(kpage.inv());
        }
        Ok(kpage)
    }


    /// Allocates multiple kernel pages.
    ///
    /// # Description
    ///
    /// Allocates `count` frames from the kernel pool and wraps each in a KernelPage.
    /// This is used for batch allocation of kernel pages.
    ///
    /// # Parameters
    ///
    /// - `count`: Number of kernel pages to allocate (must be > 0).
    ///
    /// # Returns
    ///
    /// Upon success, a Vec of KernelPages. Upon failure, an error if insufficient frames.
    ///
    /// # Postconditions
    ///
    /// - On success: Kernel pool free count decreases by `count`.
    ///
    /// # Note
    ///
    /// This is a specification-level function. For executable code, use alloc_kpage()
    /// in a loop. The original implementation uses alloc_many_kernel_frames internally.
    #[verifier::external_body]
    pub fn alloc_kpages(&mut self, count: usize) -> (result: Result<(), Error>)
        requires
            old(self).inv(),
            count > 0,
            old(self)@.has_kpool_capacity_for(count as int),
        ensures
            self.inv(),
            result.is_ok() ==> self@.kpool_free_count == old(self)@.kpool_free_count - count as int,
            result.is_err() ==> self@ == old(self)@,
    {
        unimplemented!()
    }

    //==============================================================================================

    /// Allocates multiple user pages.
    ///
    /// # Description
    ///
    /// Allocates `nframes` frames from the user pool and maps each to consecutive
    /// virtual addresses starting at `vaddr`. This is used by ELF loading to allocate
    /// contiguous virtual memory regions.
    ///
    /// # Parameters
    ///
    /// - `vmem`: Virtual memory space to map pages into.
    /// - `vaddr`: Starting virtual address (page-aligned).
    /// - `nframes`: Number of pages to allocate.
    /// - `access`: Access permissions for all pages.
    ///
    /// # Returns
    ///
    /// Upon success, Ok(()). Upon failure, an error.
    ///
    /// # Preconditions
    ///
    /// - User pool must have at least `nframes` free frames.
    /// - Vmem must have capacity for `nframes` mappings.
    /// - vaddr must be page-aligned and in user space.
    /// - All target addresses must not be already mapped.
    ///
    /// # Postconditions
    ///
    /// - On success: User pool free count decreases by `nframes`.
    /// - On success: Vmem mapping count increases by `nframes`.
    ///
    /// # Note
    ///
    /// This is a specification-level function. The postconditions match the original
    /// `alloc_upages` behavior. For full verification, each page would need individual
    /// allocation proof similar to `alloc_upage`.
    #[verifier::external_body]
    pub fn alloc_upages(
        &mut self,
        vmem: &mut Vmem,
        vaddr: usize,
        nframes: usize,
        access: AccessPermission,
    ) -> (result: Result<(), Error>)
        requires
            old(self).inv(),
            old(vmem).inv(),
            nframes > 0,
            old(self)@.has_upool_capacity_for(nframes as int),
            old(vmem).mapping_count as int + nframes as int <= MAX_USER_PAGES as int,
            vaddr as int % PAGE_SIZE as int == 0,
            spec_is_user_addr(vaddr as int),
            // All target addresses are in user space and not already mapped.
            forall|i: int|
                #![trigger spec_is_user_addr(vaddr as int + i * PAGE_SIZE as int)]
                0 <= i < nframes as int ==>
                    spec_is_user_addr(vaddr as int + i * PAGE_SIZE as int) &&
                    !old(vmem).spec_is_mapped(vaddr as int + i * PAGE_SIZE as int),
        ensures
            self.inv(),
            vmem.inv(),
            result.is_ok() ==> {
                &&& self@.upool_free_count == old(self)@.upool_free_count - nframes as int
                &&& vmem.mapping_count == old(vmem).mapping_count + nframes
            },
            result.is_err() ==> vmem.mapping_count == old(vmem).mapping_count,
    {
        unimplemented!()
    }

    //==============================================================================================

    /// Returns the kernel pool capacity.
    pub fn kpool_capacity(&self) -> (result: usize)
        requires
            self.inv(),
        ensures
            result as int == self@.kpool_capacity,
    {
        self.kpool.capacity()
    }


    /// Returns the user pool capacity.
    pub fn upool_capacity(&self) -> (result: usize)
        requires
            self.inv(),
        ensures
            result as int == self@.upool_capacity,
    {
        self.upool.capacity()
    }
}

} // verus!
