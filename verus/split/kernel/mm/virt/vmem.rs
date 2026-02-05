// Copyright(c) The Maintainers of Nanvix.
// Licensed under the MIT License.

//==================================================================================================
//! # Virtual Memory Space (Verified Specification Model)
//!
//! This module provides a **verified specification model** of the `Vmem` abstraction for
//! formal verification purposes. It is designed to verify the correctness of memory
//! management logic and preconditions, NOT to replace the actual implementation.
//!
//! ## Purpose and Scope
//!
//! This verification module serves as:
//! - A **formal specification** of virtual memory space properties
//! - A **proof** that precondition/postcondition contracts are satisfiable
//! - A **reference model** for the actual implementation
//!
//! It is NOT:
//! - A drop-in replacement for the original implementation
//! - A complete model of hardware page table operations
//! - Executable code for production use
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
//! 7. **Physical Memory Bounds**: Copy operations verify physical address ranges.
//! 8. **Mapping Existence**: Copy operations require user pages to be mapped.
//! 9. **Mapping Uniqueness**: Each virtual address is mapped at most once.
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
//! ### Kernel Mappings Not Modeled
//! Kernel page tables and pages are shared across address spaces via Rc<RefCell<>>
//! in the original implementation. We do not model kernel mappings because:
//! 1. They are shared state managed separately from user mappings
//! 2. Verification of shared ownership would require linear types or separation logic
//! 3. The core safety properties (user/kernel separation) are captured without modeling kernel internals
//!
//! **Private vs Shared Kernel Pages**: The original has `private_kernel_pages` distinct
//! from shared kernel pages. Private kernel pages belong to a single address space while
//! shared pages are reference-counted across all address spaces. This distinction is not
//! modeled because: (a) both are kernel mappings which are out of scope, and (b) the
//! ownership model would require linear types for proper verification.
//!
//! ### Capacity Simplification
//! The verified model uses a fixed-size array (MAX_USER_PAGES=65536) while the original
//! uses unbounded linked lists. This is sufficient for verification of core properties
//! and typical workloads. Production systems should validate this bound.
//!
//! ### Hardware Effects Not Modeled
//! TLB flush effects, CR3 loading, and cache coherency are hardware-specific behaviors
//! that are not modeled in this verification. Functions affecting hardware state are
//! marked external_body with appropriate precondition/postcondition contracts.
//!
//! ### Internal Helpers Abstracted
//! Functions like `lookup_page_table`, `lookup_kernel_page_table`, and `Drop` are
//! implementation details that are abstracted away. The essential behaviors they
//! provide (translation lookup, resource cleanup) are captured in the high-level
//! specifications of the public API.
//!
//! **Rationale for omitting `lookup_page_table`/`lookup_kernel_page_table`**:
//! These functions iterate over linked lists to find page tables by address.
//! The array model abstracts this by maintaining mappings directly. The safety
//! of the lookup is captured by the uniqueness invariant - if a vaddr is mapped,
//! there is exactly one entry for it. The linked list iteration correctness
//! would need to be verified separately if refinement proofs are implemented.
//!
//! **Rationale for omitting `Drop` implementation**:
//! The `Drop` trait deallocates page tables and releases kernel pages. This is
//! resource management that doesn't affect memory safety properties verified here
//! (address space separation, bounds checking). Resource leak verification would
//! require tracking allocation/deallocation pairs, which is out of scope for this
//! memory safety specification model.
//!
//! ### Memory Operations as Specifications
//! Functions like `copy_from_user_unaligned`, `copy_to_user_unaligned`, `memset`,
//! and `uctrl` verify preconditions and postconditions but do not perform
//! actual memory operations. They serve as specifications that the actual
//! implementation must satisfy. The implementation would perform unsafe hardware
//! operations that cannot be verified without a full hardware model.
//!
//! ### Relationship to Implementation
//! This verification module should be linked to the actual implementation via
//! refinement or used to generate proof obligations. The verified contracts can
//! be attached to the original implementation as pre/postconditions that the
//! implementation must satisfy at runtime or through separate proof.
//!
//! **Future Work**: Implement refinement proofs that connect this specification
//! model to the actual page table implementation. This would require:
//! 1. A verified model of x86 page directory/table structures
//! 2. Proof that the implementation's linked list operations refine the array model
//! 3. Connection to a hardware memory model for physical operations
//!
//! ### Verification Value
//! While this model abstracts page table complexity, it provides value by:
//! 1. **Contract Verification**: Proves that the API contracts are internally consistent
//! 2. **Invariant Preservation**: Verifies that all operations maintain key invariants
//! 3. **Safety Properties**: Proves user/kernel separation, bounds checking, uniqueness
//! 4. **Precondition Discovery**: Identifies necessary preconditions for safe operation
//! 5. **Reference Specification**: Serves as a formal reference for implementation
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
//! | `clone()` | Clone a virtual memory space for process forking |
//! | `load()` | Load the page directory into CR3 register |
//! | `pgdir()` | Get the page directory physical address |
//! | `map()` | Map a user frame to a virtual address |
//! | `map_kpage()` | Map a kernel page to a virtual address |
//! | `unmap()` | Unmap a page from the virtual address space |
//! | `is_user_addr()` | Check if address is in user space |
//! | `is_user_region()` | Check if region is entirely in user space |
//! | `is_physical_region()` | Check if region is within physical memory |
//! | `copy_from_user_unaligned()` | Copy from user space to kernel space |
//! | `copy_to_user_unaligned()` | Copy from kernel space to user space |
//! | `copy_to_user_unaligned_unchecked()` | Unchecked copy with dry-run support |
//! | `memset()` | Fill a page with a value |
//! | `uctrl()` | Change access permissions on a user page |
//! | `kctrl()` | Change access permissions on a kernel page |
//!
//! Private helper functions (not exposed):
//! - `is_kernel_addr()` - Check if address is in kernel space
//! - `is_kernel_region()` - Check if region is entirely in kernel space
//! - `find_user_frame()` - Look up physical frame for user address
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
    kernel::mm::virt::kpage::PAGE_SIZE,
};
use vstd::prelude::*;

// Include specifications.
include!("vmem.spec.rs");

// Include proofs.
include!("vmem.proof.rs");


verus! {

//==================================================================================================

/// User space base address.
/// Must be page-aligned and mark the start of user-accessible virtual memory.
///
/// # Configuration Note
///
/// This value matches `config::memory_layout::USER_BASE` from the system configuration.
/// The value 0x40000000 (1 GB) is the standard user space start address for x86.
pub const USER_BASE: usize = 0x40000000; // 1 GB


/// User space end address (exclusive).
/// Must be page-aligned and mark the end of user-accessible virtual memory.
///
/// # Configuration Note
///
/// This value matches `config::memory_layout::USER_END` from the system configuration.
/// The value 0xC0000000 (3 GB) is the standard kernel/user split point for x86.
pub const USER_END: usize = 0xC0000000; // 3 GB


/// Total physical memory size (for bounds checking).
///
/// # Configuration Note
///
/// This value matches `config::kernel::MEMORY_SIZE` from the system configuration.
/// The value 0x10000000 (256 MB) is the default physical memory size.
pub const MEMORY_SIZE: usize = 0x10000000; // 256 MB


/// Maximum number of user pages that can be tracked (simplified model).
///
/// # Abstraction Note
///
/// This is a verification simplification. The original implementation uses
/// linked lists with dynamic allocation. For verification tractability, we
/// use a fixed-size array.
///
/// # Capacity
///
/// With PAGE_SIZE=4096 bytes and MAX_USER_PAGES=65536, this model supports
/// up to 256 MB of user virtual memory, which covers the full user address
/// space for typical embedded/microkernel configurations.
pub const MAX_USER_PAGES: usize = 65536;

//==================================================================================================

/// Access permission flags for memory pages.
///
/// # Abstraction Note
///
/// The original implementation uses bit flags for permissions combined with
/// caching flags. This simplified enum captures the essential permission
/// categories for verification purposes. The mapping is:
/// - `ReadOnly` -> original's read-only mode
/// - `ReadWrite` -> original's read-write mode
/// - `Execute` -> original's read-execute mode
///
/// # Scope Limitation
///
/// Permission enforcement at the PTE level is not verified. The map/ctrl
/// operations accept permissions as parameters but the verified model does
/// not track per-page permissions. This is an intentional scope limitation:
/// verifying permission enforcement would require modeling the full PTE
/// structure and MMU behavior.
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
    ///
    /// # Signature Difference
    ///
    /// The original signature is:
    /// ```ignore
    /// fn new(kernel_pages: LinkedList<KernelPage>,
    ///        kernel_page_tables: LinkedList<(PageTableAddress, PageTable<...>)>)
    ///        -> Result<Self, Error>
    /// ```
    /// This verified version takes no parameters because:
    /// 1. Kernel mappings are abstracted (shared state not tracked in verified model)
    /// 2. The original's error path (from pgdir.map) is not modeled
    ///
    /// For refinement proofs, this would need to be wrapped with an external_body
    /// function matching the original signature.
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


    /// Clones a virtual memory space for process forking.
    ///
    /// Creates a new Vmem that shares kernel page tables/pages (reference counted)
    /// but has independent (empty) user page tables.
    ///
    /// # Parameters
    ///
    /// - `from`: The source virtual memory space to clone from. The source must
    ///   satisfy its invariant. Kernel mappings sharing is not explicitly modeled
    ///   in the verified abstraction - the `from` parameter is used to require
    ///   source validity and to match the original API signature.
    ///
    /// # Returns
    ///
    /// A new Vmem instance with kernel mappings shared and empty user mappings.
    ///
    /// # Postconditions
    ///
    /// - The returned Vmem satisfies its invariant.
    /// - The returned Vmem has no user pages mapped (user space is empty).
    ///
    /// # Fork Semantics
    ///
    /// This models POSIX fork() semantics where:
    /// - Kernel mappings are shared (via reference counting in the original)
    /// - User mappings start empty in the child and are populated via copy-on-write
    ///   at page fault time, NOT via deep copy at clone time
    ///
    /// # Signature Difference
    ///
    /// The original returns `Result<Vmem, Error>` because `physical_address()`
    /// can fail during page directory setup. This verified version returns `Self`
    /// (infallible) because the error path requires modeling PageDirectory
    /// internals which is out of scope. For refinement proofs, this would need
    /// an external_body wrapper with the original signature.
    ///
    /// This is why `result.mapping_count == 0` - the cloned Vmem has no user
    /// mappings initially. The actual user page data is copied lazily when
    /// the child process writes to a page.
    ///
    /// # Note
    ///
    /// In the verified model, we only track user mappings. Kernel mappings are
    /// abstracted as shared state that is not explicitly modeled. The original
    /// implementation copies kernel_page_tables and kernel_pages via Rc::clone(),
    /// establishing shared ownership. This is sound because:
    /// 1. Kernel mappings are read-only from user process perspective.
    /// 2. User mappings start empty in both original and verified versions.
    ///
    /// # Model Limitation
    ///
    /// The empty user mappings in the cloned Vmem mean that any `copy_from_user`
    /// call on the child process would fail immediately in this model due to
    /// "region not mapped". In the actual implementation, the page fault handler
    /// would allocate and populate pages on-demand (COW). This page fault handling
    /// is NOT modeled. Verified code that needs to reason about child process
    /// memory access after clone would need to explicitly call `map()` to establish
    /// the expected mappings in the model.
    pub fn clone(from: &Self) -> (result: Self)
        requires
            from.inv(),
        ensures
            result.inv(),
            result.mapping_count == 0,
            // Note: Source preservation is guaranteed by Rust's borrow checker.
            // The `from: &Self` parameter is an immutable borrow, so Rust ensures
            // the source is unchanged. Verus's `old()` requires `&mut` so we cannot
            // express this directly in the postcondition, but the type system
            // provides the guarantee.
    {
        // Use from in a proof block to document that we require source validity.
        // Kernel mappings sharing is abstracted - we only verify user mapping properties.
        proof {
            assert(from.inv());
        }

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


    /// Loads the page directory into the CR3 register.
    ///
    /// This activates the virtual memory space for the current CPU.
    ///
    /// # Returns
    ///
    /// Upon success, Ok(()). Upon failure, an error if the page directory
    /// physical address cannot be obtained.
    ///
    /// # Note
    ///
    /// This is modeled as external_body since we don't model hardware state
    /// (CR3 register). The original implementation can fail if
    /// `self.pgdir.physical_address()` fails, so we don't guarantee success.
    #[verifier::external_body]
    pub fn load(&self) -> (result: Result<(), Error>)
        requires
            self.inv(),
    {
        unimplemented!()
    }


    /// Returns the page directory physical address as a raw value.
    ///
    /// # Returns
    ///
    /// The physical address of the page directory as a usize.
    ///
    /// # Note
    ///
    /// This is modeled as external_body since the page directory is not
    /// explicitly modeled in the verified abstraction. The return type
    /// differs from the original (which returns `&PageDirectory`) - this
    /// simplification returns the raw physical address instead of a
    /// reference to the complex PageDirectory structure. For verification
    /// purposes, we only need to know that a valid Vmem has a page directory.
    #[verifier::external_body]
    pub fn pgdir(&self) -> (result: usize)
        requires
            self.inv(),
    {
        unimplemented!()
    }


    /// Maps a kernel page to the target virtual address space.
    ///
    /// # Parameters
    ///
    /// - `frame_addr`: Physical frame address of the kernel page.
    /// - `vaddr`: Virtual address to map to (must be page-aligned and in kernel space).
    /// - `access`: Access permissions for the mapping.
    ///
    /// # Returns
    ///
    /// Upon success, Ok(()). Upon failure, an error describing the issue.
    ///
    /// # Errors
    ///
    /// - `BadAddress`: The virtual address is not in kernel space or not page-aligned.
    /// - `TryAgain`: Failed to read page directory entry.
    /// - `NoSuchEntry`: Page table not found.
    ///
    /// # Note
    ///
    /// Kernel mappings are not tracked in the verified model's mapping array,
    /// as they are shared across address spaces. This function is marked
    /// external_body because:
    /// 1. Kernel mappings use complex linked list structures with Rc<RefCell<>>
    /// 2. Page table allocation requires a callback allocator
    /// 3. TLB flush effects are hardware-specific
    ///
    /// The specification captures:
    /// - Preconditions: valid invariant, kernel address, page alignment
    /// - Postconditions: invariant preserved, user mapping count unchanged
    /// - May fail: returns Result to model failure modes
    #[verifier::external_body]
    pub fn map_kpage(
        &mut self,
        frame_addr: FrameAddress,
        vaddr: usize,
        access: AccessPermission,
    ) -> (result: Result<(), Error>)
        requires
            old(self).inv(),
            spec_is_kernel_addr(vaddr as int),
            vaddr as int % PAGE_SIZE as int == 0,
        ensures
            self.inv(),
            self.mapping_count == old(self).mapping_count,
    {
        unimplemented!()
    }

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
    fn is_kernel_addr(vaddr: usize) -> (result: bool)
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
    fn is_kernel_region(start: usize, size: usize) -> (result: bool)
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
    ///
    /// # Frame Physical Bounds
    ///
    /// The `FrameAddress` type represents a page-aligned physical address. The verified
    /// model does not explicitly require frame_addr < MEMORY_SIZE as a precondition
    /// because:
    /// 1. In the original implementation, frames come from UserFrame which is allocated
    ///    by the physical memory allocator (upool). The allocator only provides frames
    ///    within the physical memory range.
    /// 2. Adding this precondition would require modeling the allocator invariants,
    ///    which is out of scope for this address space specification.
    /// 3. The safety property is established at allocation time, not at map time.
    ///
    /// For refinement proofs, a precondition like `frame_addr.spec_raw_value() < MEMORY_SIZE`
    /// could be added if the allocator's postconditions are connected to the vmem model.
    ///
    /// # Type Mapping
    ///
    /// The original signature is:
    /// ```ignore
    /// fn map(&mut self, uframe: UserFrame, vaddr: PageAligned<VirtualAddress>,
    ///        access: AccessPermission, page_table_allocator: T) -> Result<(), Error>
    /// ```
    /// Type correspondences:
    /// - `UserFrame.frame_address() -> FrameAddress`
    /// - `PageAligned<VirtualAddress>.into_raw_value() -> usize`
    /// - The `page_table_allocator` callback is not modeled (page table allocation is abstracted)
    pub fn map(
        &mut self,
        frame_addr: FrameAddress,
        vaddr: usize,
        access: AccessPermission,
    ) -> (result: Result<(), Error>)
        requires
            old(self).inv(),
            old(self).has_mapping_capacity(),
            frame_addr.spec_is_aligned(),
            // Liveness: require valid parameters for guaranteed success.
            spec_is_user_addr(vaddr as int),
            vaddr as int % PAGE_SIZE as int == 0,
            !old(self).spec_is_mapped(vaddr as int),
        ensures
            self.inv(),
            // LIVENESS: With all preconditions met, map always succeeds.
            result.is_ok(),
            result.is_ok() ==> {
                &&& self.spec_is_mapped(vaddr as int)
                &&& self.mapping_count == old(self).mapping_count + 1
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
        // With precondition !old(self).spec_is_mapped(vaddr), this loop won't find a match.
        let mut i: usize = 0;
        let ghost old_mapping_count: usize = self.mapping_count;
        while i < self.mapping_count
            invariant
                0 <= i <= self.mapping_count,
                self.mapping_count < MAX_USER_PAGES,
                self.inv(),
                self.mapping_count == old_mapping_count,
                // No modification to mappings during scan.
                !self.spec_is_mapped(vaddr as int),
                forall|j: int| #![auto] 0 <= j < i as int ==>
                    !(self.mappings[j as int].valid && self.mappings[j as int].vaddr == vaddr),
            decreases self.mapping_count - i,
        {
            if self.mappings[i].valid && self.mappings[i].vaddr == vaddr {
                proof {
                    // This branch contradicts the invariant !self.spec_is_mapped(vaddr).
                    assert(self.mappings[i as int].spec_is_for_vaddr(vaddr as int));
                    assert(self.spec_is_mapped(vaddr as int));
                    assert(false);
                }
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
    ///
    /// # Type Mapping
    ///
    /// The original returns `Result<UserFrame, Error>`. This verified version returns
    /// `Result<usize, Error>` where the `usize` is the raw physical frame address.
    /// The correspondence is: `UserFrame.frame_address().into_raw_value() -> usize`.
    ///
    /// # Ownership Semantics
    ///
    /// The original `UserFrame` type carries RAII ownership semantics: when dropped,
    /// it deallocates the underlying physical frame. This ownership transfer is NOT
    /// captured in this verified model. Resource leak prevention and double-free
    /// safety would require ghost resource tracking which is out of scope. The verified
    /// model only ensures the mapping is correctly removed from the address space.
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
                // The returned value was a previously-mapped frame address.
                &&& old(self).spec_is_mapped(vaddr as int)
                // The returned frame address is frame-aligned.
                &&& result.unwrap() as int % FRAME_SIZE as int == 0
                // The returned frame address equals the one that was mapped.
                &&& result.unwrap() as int == old(self).spec_get_frame_addr(vaddr as int)
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

        // Capture old self for proving postcondition about spec_get_frame_addr.
        let ghost entry_self: Vmem = *self;

        // Find the mapping.
        let mut found_idx: usize = MAX_USER_PAGES;
        let mut frame_addr: usize = 0;
        let mut i: usize = 0;
        while i < self.mapping_count
            invariant
                0 <= i <= self.mapping_count,
                self.mapping_count <= MAX_USER_PAGES,
                self.inv(),
                // Entry state is unchanged during search.
                self.mapping_count == entry_self.mapping_count,
                forall|j: int| #![auto] 0 <= j < self.mapping_count as int ==>
                    self.mappings[j as int] == entry_self.mappings[j as int],
                found_idx == MAX_USER_PAGES || found_idx < self.mapping_count,
                found_idx < MAX_USER_PAGES ==> {
                    &&& self.mappings[found_idx as int].valid
                    &&& self.mappings[found_idx as int].vaddr == vaddr
                    &&& frame_addr as int % FRAME_SIZE as int == 0
                    &&& frame_addr as int == self.mappings[found_idx as int].frame_addr as int
                },
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

        proof {
            // entry_self has the same mappings as old(self) at function entry.
            // The found mapping has frame_addr == entry_self.spec_get_frame_addr(vaddr).
            assert(entry_self.mappings[found_idx as int].spec_is_for_vaddr(vaddr as int));
            assert(entry_self.spec_is_mapped(vaddr as int));
            assert(frame_addr as int == entry_self.mappings[found_idx as int].frame_addr as int);
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
    ///
    /// # Visibility
    ///
    /// This is a private helper function matching the original implementation's
    /// visibility. It is used internally by copy operations to look up frame
    /// addresses.
    fn find_user_frame(&self, vaddr: usize) -> (result: Result<usize, Error>)
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
                &&& spec_is_user_addr(vaddr as int)
                &&& vaddr as int % PAGE_SIZE as int == 0
            },
            // Permission changes don't affect the mappings.
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
                found ==> exists|j: int|
                    #![trigger self.mappings[j]]
                    0 <= j < self.mapping_count as int &&
                    self.mappings[j].valid &&
                    self.mappings[j].vaddr == vaddr,
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
    ///
    /// # Note
    ///
    /// This is marked `external_body` because the verified model does not track
    /// kernel mappings. The original implementation would fail with `NoSuchEntry`
    /// if the PDE/PTE is absent. The specification captures the precondition that
    /// the Vmem invariant holds and the address must be in kernel space.
    ///
    /// The implementation must verify that the kernel page exists before
    /// modifying permissions.
    #[verifier::external_body]
    pub fn kctrl(&mut self, vaddr: usize, access: AccessPermission) -> (result: Result<(), Error>)
        requires
            old(self).inv(),
            spec_is_kernel_addr(vaddr as int),
            vaddr as int % PAGE_SIZE as int == 0,
        ensures
            self.inv(),
            self.mapping_count == old(self).mapping_count,
    {
        unimplemented!()
    }

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
    ///
    /// # Preconditions
    ///
    /// - The source user pages must be mapped. The original implementation
    ///   panics if `find_user_frame` fails during the copy loop.
    ///
    /// # Note
    ///
    /// The original implementation checks that each source page is mapped
    /// and that the source frame physical addresses are within bounds during
    /// the copy loop. The destination is a kernel virtual address that is
    /// identity-mapped to physical memory, so no separate physical bounds
    /// check is performed on it.
    ///
    /// # Physical Bounds
    ///
    /// Physical frame addresses are looked up internally via `find_user_frame`
    /// but not returned by this function. The physical bounds are implicitly
    /// satisfied because frames come from the physical memory allocator which
    /// only allocates within MEMORY_SIZE. This constraint is established at
    /// map() time where frame_addr must come from a valid FrameAddress.
    pub fn copy_from_user_unaligned(
        &self,
        dst: usize,
        src: usize,
        size: usize,
    ) -> (result: Result<(), Error>)
        requires
            self.inv(),
            // Source user pages must be mapped (original panics if not).
            size > 0 ==> self.spec_user_region_is_mapped(src as int, size as int),
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
        // This includes looking up user frames and copying page-by-page.
        // The precondition ensures all source pages are mapped.
        Ok(())
    }


    /// Copies data from kernel space to user space.
    ///
    /// # Parameters
    ///
    /// - `dst`: Destination address in user space.
    /// - `src`: Source address in kernel space (must be within physical memory bounds).
    /// - `size`: Number of bytes to copy.
    ///
    /// # Returns
    ///
    /// Upon success, Ok(()). Upon failure, an error describing the issue.
    ///
    /// # Errors
    ///
    /// - `InvalidArgument`: The size is zero.
    /// - `BadAddress`: Source region not in kernel space.
    /// - `BadAddress`: Destination region not in user space.
    /// - `BadAddress`: Source region not within physical memory bounds.
    ///
    /// # Preconditions
    ///
    /// - The destination user pages must be mapped. The original implementation
    ///   panics if `find_user_frame` fails during the copy loop.
    ///
    /// # Physical Memory Requirement
    ///
    /// The source address must be within physical memory bounds (< MEMORY_SIZE).
    /// This is NOT overly restrictive - Nanvix uses identity mapping where kernel
    /// accesses physical memory directly via low addresses. The kernel space check
    /// (`is_kernel_region`) verifies the address is not in user space [USER_BASE,
    /// USER_END), while the physical region check (`is_physical_region`) verifies
    /// the address can be used for physical memory operations. These checks mirror
    /// the original implementation exactly (lines 732-739 and 774-789).
    ///
    /// # Note
    ///
    /// The original implementation performs a dry-run first to check for errors
    /// before the actual copy. This verified version matches that control flow
    /// by calling `copy_to_user_unaligned_unchecked` twice: once for dry-run
    /// validation and once for the actual copy.
    pub fn copy_to_user_unaligned(
        &self,
        dst: usize,
        src: usize,
        size: usize,
    ) -> (result: Result<(), Error>)
        requires
            self.inv(),
            // Destination user pages must be mapped (original panics if not).
            size > 0 ==> self.spec_user_region_is_mapped(dst as int, size as int),
        ensures
            result.is_ok() ==> {
                &&& size > 0
                &&& spec_is_kernel_region(src as int, size as int)
                &&& spec_is_user_region(dst as int, size as int)
                &&& spec_is_physical_region(src as int, size as int)
            },
    {
        // Perform a dry run first to check for errors (matches original control flow).
        self.copy_to_user_unaligned_unchecked(dst, src, size, true)?;
        // Perform the actual copy (this will panic on irrecoverable errors).
        self.copy_to_user_unaligned_unchecked(dst, src, size, false)
    }


    /// Copies data from kernel space to user space without dry-run validation.
    ///
    /// # Parameters
    ///
    /// - `dst`: Destination address in user space.
    /// - `src`: Source address in kernel space.
    /// - `size`: Number of bytes to copy.
    /// - `dry_run`: If true, only validate without performing the copy.
    ///
    /// # Returns
    ///
    /// Upon success, Ok(()). Upon failure, an error describing the issue.
    ///
    /// # Safety
    ///
    /// When not running in dry-run mode, this function performs a physical memory
    /// copy. Any errors that occur while copying data will cause this function to
    /// panic. The caller must ensure all preconditions are satisfied.
    ///
    /// # Note
    ///
    /// This is the unchecked variant that the original implementation uses
    /// internally. It is marked external_body because it performs unsafe
    /// physical memory operations that require hardware access.
    ///
    /// # Destination Physical Bounds
    ///
    /// The original implementation checks destination frame physical bounds and panics
    /// if violated (lines 806-813). This is a runtime safety assertion, not a return
    /// error. The postcondition does not include destination physical bounds because:
    /// 1. The check only applies when dry_run=false (inside the copy loop)
    /// 2. Violation causes a panic, not an Err return
    /// 3. Frames come from the allocator which only provides addresses within MEMORY_SIZE
    /// The safety relies on the allocator invariant, not per-operation validation.
    #[verifier::external_body]
    pub fn copy_to_user_unaligned_unchecked(
        &self,
        dst: usize,
        src: usize,
        size: usize,
        dry_run: bool,
    ) -> (result: Result<(), Error>)
        requires
            self.inv(),
            size > 0 ==> self.spec_user_region_is_mapped(dst as int, size as int),
        ensures
            result.is_ok() ==> {
                &&& size > 0
                &&& spec_is_kernel_region(src as int, size as int)
                &&& spec_is_user_region(dst as int, size as int)
                &&& spec_is_physical_region(src as int, size as int)
            },
    {
        unimplemented!()
    }


    /// Fills a user page with a given value.
    ///
    /// # Parameters
    ///
    /// - `vaddr`: Virtual address of the page (must be page-aligned and in user space).
    /// - `value`: Value to fill the page with. Note: the underlying implementation
    ///   truncates this to u8 when calling __phys_memset. Only the lowest 8 bits
    ///   are used for the fill pattern.
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

} // verus!
