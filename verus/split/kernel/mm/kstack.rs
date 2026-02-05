// Copyright(c) The Maintainers of Nanvix.
// Licensed under the MIT License.

//==================================================================================================
//! # Kernel Stack (Verified Implementation)
//!
//! This module provides a verified implementation of the Kernel Stack data structure.
//! The kernel stack manages a contiguous region of virtual memory used for kernel-mode
//! execution stacks.
//!
//! ## Overview
//!
//! A kernel stack consists of one or more contiguous kernel pages. The stack has:
//! - A **base address**: the lowest virtual address of the allocated pages
//! - A **top address**: the first address past the end of the stack (base + size)
//! - A **size**: the total size in bytes (num_pages * PAGE_SIZE)
//!
//! Since stacks grow downward on x86, the stack pointer starts at the top and grows
//! toward the base.
//!
//! ## Memory Safety Properties Verified
//!
//! 1. **Non-empty Invariant**: A kernel stack always has at least one page.
//! 2. **Page Alignment**: Base address is always page-aligned.
//! 3. **Size Alignment**: Stack size is always a multiple of PAGE_SIZE.
//! 4. **Address Ordering**: top = base + size, so top > base (assuming size > 0).
//! 5. **Contiguity**: All pages in the stack form a contiguous memory region.
//! 6. **No Overflow**: Address arithmetic does not overflow.
//!
//! ## Liveness Properties Verified
//!
//! 1. **Allocation Success**: If sufficient pages are available, allocation succeeds.
//! 2. **Accessor Availability**: base(), top(), size() always return valid values for
//!    a well-formed stack.
//!
//! ## Verification Scope and Abstraction Decisions
//!
//! This verification focuses on the **address arithmetic and invariant safety** of the
//! kernel stack, not on resource lifecycle management. The following are explicitly
//! **out of scope**:
//!
//! ### Out of Scope: Allocator Interaction
//!
//! The original implementation takes `&mut VirtMemoryManager` and calls `mm.alloc_kpages()`
//! to allocate pages. This verified version abstracts the allocator by taking
//! `(base_addr, num_pages)` directly. The rationale:
//!
//! - **Separation of concerns**: The allocator (`VirtMemoryManager`) should be verified
//!   separately with its own invariants (contiguity, alignment, no double-allocation).
//! - **Preconditions**: The `new()` preconditions encode what a correct allocator must
//!   provide: page-aligned base, valid page count, no overflow.
//! - **Composability**: When both modules are verified, their guarantees compose.
//!
//! ### Out of Scope: Drop / Resource Cleanup
//!
//! The original implementation has a `Drop` trait that pops and drops each `KernelPage`.
//! This verified version does not model `Drop` because:
//!
//! - **Verus limitations**: Verus does not yet fully support verifying `Drop` traits.
//! - **Linear types**: Proper deallocation verification requires linear/affine types
//!   which would track page ownership through the type system.
//! - **Focus**: This verification targets the data structure invariants, not RAII.
//!
//! ### Out of Scope: Fixed Stack Size Configuration
//!
//! The original uses `config::kernel::KSTACK_SIZE` (typically 32768 bytes = 8 pages).
//! This verified version parameterizes by `num_pages` for generality:
//!
//! - **Flexibility**: Verifies the general case; any specific size is a specialization.
//! - **Configuration binding**: To verify config-specific behavior, instantiate with
//!   `num_pages = KSTACK_SIZE / PAGE_SIZE` and the proofs apply.
//!
//! ### Return Type Abstraction
//!
//! The original returns `PageAligned<VirtualAddress>` from `base()` and `top()`.
//! This verified version returns raw `usize` with alignment proven in postconditions:
//!
//! - **Type-level vs proof-level**: The original enforces alignment via the type system;
//!   the verification proves the same property via postconditions.
//! - **Equivalent guarantees**: Both approaches ensure callers receive aligned addresses.
//!
//! ### Verification Helper Methods
//!
//! The methods `contains()`, `page_index()`, `initial_sp()`, and `has_room()` are
//! added for verification purposes. They enable reasoning about:
//!
//! - Stack bounds checking (for stack overflow protection)
//! - Page membership (for page fault handling)
//! - Stack pointer validity
//!
//! These could be added to the original implementation if useful, or kept as
//! verification-only helpers.
//!
//! ## API Summary
//!
//! | Function | Description |
//! |----------|-------------|
//! | `new(base_addr, num_pages)` | Create a new kernel stack |
//! | `size()` | Returns the size in bytes |
//! | `base()` | Returns the base address |
//! | `top()` | Returns the top address (first byte past end) |
//! | `num_pages()` | Returns the number of pages |
//!
//==================================================================================================

use crate::libs::error::{
    Error,
    ErrorCode,
};
use vstd::prelude::*;

// Include specifications.
include!("kstack.spec.rs");

// Include proofs.
include!("kstack.proof.rs");


verus! {

//==================================================================================================

/// Page size in bytes (4 KiB).
pub const PAGE_SIZE: usize = 4096;


/// Page alignment requirement.
pub const PAGE_ALIGNMENT: usize = 4096;


/// Maximum stack size in pages (to prevent overflow).
pub const MAX_STACK_PAGES: usize = 256;


/// Default kernel stack size in pages (matches config::kernel::KSTACK_SIZE / PAGE_SIZE = 32768 / 4096 = 8).
pub const DEFAULT_KSTACK_PAGES: usize = 8;

//==================================================================================================

/// Abstract view of a kernel stack for specification purposes.
///
/// This view captures the essential properties of a kernel stack without
/// exposing implementation details.
#[verifier::ext_equal]
pub struct KernelStackView {
    /// Base virtual address of the stack (lowest address).
    pub base_addr: int,
    /// Number of pages in the stack.
    pub num_pages: int,
}

//==================================================================================================

/// A type that represents a kernel stack.
///
/// The kernel stack is a contiguous region of virtual memory used for kernel-mode
/// execution. It consists of one or more pages and provides accessors for the
/// base address, top address, and size.
#[derive(Debug)]
pub struct KernelStack {
    /// Base virtual address of the stack.
    base_addr: usize,
    /// Number of pages in the stack.
    num_pages: usize,
}

impl KernelStack {
    //==============================================================================================

    /// Instantiates a new kernel stack.
    ///
    /// # Description
    ///
    /// Creates a new kernel stack with the given base address and number of pages.
    /// The base address must be page-aligned and the number of pages must be positive
    /// and within bounds.
    ///
    /// # Parameters
    ///
    /// - `base_addr`: The base virtual address of the stack (must be page-aligned).
    /// - `num_pages`: The number of pages to allocate (must be > 0 and <= MAX_STACK_PAGES).
    ///
    /// # Returns
    ///
    /// Upon success, returns the new kernel stack. Upon failure, returns an error.
    ///
    /// # Errors
    ///
    /// - `InvalidArgument`: If base_addr is not page-aligned, num_pages is 0, or
    ///   num_pages exceeds MAX_STACK_PAGES.
    /// - `OutOfMemory`: If the address arithmetic would overflow.
    ///
    pub fn new(base_addr: usize, num_pages: usize) -> (result: Result<Self, Error>)
        requires
            spec_is_page_aligned(base_addr as int),
            0 < num_pages <= MAX_STACK_PAGES,
            base_addr as int + spec_pages_to_bytes(num_pages as int) <= usize::MAX as int,
        ensures
            // Liveness: When preconditions are met, allocation always succeeds.
            result.is_ok(),
            result.is_ok() ==> {
                let stack = result.unwrap();
                &&& stack.inv()
                &&& stack.spec_base() == base_addr as int
                &&& stack.spec_num_pages() == num_pages as int
                &&& stack.spec_size() == spec_pages_to_bytes(num_pages as int)
                &&& stack.spec_top() == base_addr as int + stack.spec_size()
            },
    {
        // Validate page alignment.
        if base_addr % PAGE_ALIGNMENT != 0 {
            return Err(Error::new(ErrorCode::InvalidArgument, "base address not page-aligned"));
        }

        // Validate page count.
        if num_pages == 0 {
            return Err(Error::new(ErrorCode::InvalidArgument, "num_pages must be positive"));
        }

        if num_pages > MAX_STACK_PAGES {
            return Err(Error::new(ErrorCode::InvalidArgument, "num_pages exceeds maximum"));
        }

        // Check for overflow.
        let size: usize = num_pages * PAGE_SIZE;
        if base_addr > usize::MAX - size {
            return Err(Error::new(ErrorCode::OutOfMemory, "address overflow"));
        }

        let stack = KernelStack { base_addr, num_pages };

        // Prove the invariant holds.
        proof {
            // Prove size alignment.
            assert(spec_is_size_aligned(stack@.size())) by {
                assert(stack@.size() == num_pages as int * (PAGE_SIZE as int));
            }

            // Prove top alignment.
            assert(spec_is_page_aligned(stack@.top())) by {
                assert(stack@.top() == base_addr as int + num_pages as int * (PAGE_SIZE as int));
                // base_addr is page-aligned and size is a multiple of PAGE_SIZE.
                // So their sum is also page-aligned.
            }

            // Prove top > base.
            assert(stack@.top_greater_than_base()) by {
                assert(stack@.size() > 0);
            }

            // Prove pages are contiguous.
            assert(stack@.pages_are_contiguous());
        }

        Ok(stack)
    }

    //==============================================================================================

    /// Returns the size of the kernel stack in bytes.
    ///
    /// # Returns
    ///
    /// The size of the kernel stack.
    ///
    pub fn size(&self) -> (result: usize)
        requires
            self.inv(),
        ensures
            result as int == self.spec_size(),
            // Strengthened: Size is always positive and page-aligned.
            result > 0,
            result % PAGE_SIZE == 0,
    {
        self.num_pages * PAGE_SIZE
    }


    /// Returns the base address of the kernel stack.
    ///
    /// # Description
    ///
    /// The base address is the lowest virtual address of the stack.
    /// Since stacks grow downward, this is where the stack ends when full.
    ///
    /// # Returns
    ///
    /// The base address of the kernel stack (page-aligned).
    ///
    pub fn base(&self) -> (result: usize)
        requires
            self.inv(),
        ensures
            result as int == self.spec_base(),
            spec_is_page_aligned(result as int),
    {
        self.base_addr
    }


    /// Returns the top address of the kernel stack.
    ///
    /// # Description
    ///
    /// The top address is the first byte past the end of the stack.
    /// Since stacks grow downward, the stack pointer typically starts at top - 1
    /// (or at a word-aligned address just below top).
    ///
    /// # Returns
    ///
    /// The top address of the kernel stack (page-aligned).
    ///
    pub fn top(&self) -> (result: usize)
        requires
            self.inv(),
        ensures
            result as int == self.spec_top(),
            spec_is_page_aligned(result as int),
            // Strengthened: top is strictly greater than base.
            result > self.spec_base(),
    {
        self.base_addr + self.num_pages * PAGE_SIZE
    }


    /// Returns the number of pages in the kernel stack.
    ///
    /// # Returns
    ///
    /// The number of pages.
    ///
    pub fn num_pages(&self) -> (result: usize)
        requires
            self.inv(),
        ensures
            result as int == self.spec_num_pages(),
            // Strengthened: Guarantee positive count and bounded.
            result > 0,
            result <= MAX_STACK_PAGES,
    {
        self.num_pages
    }

    //==============================================================================================

    /// Checks if a given address is within the stack bounds.
    ///
    /// # Parameters
    ///
    /// - `addr`: The address to check.
    ///
    /// # Returns
    ///
    /// True if the address is within [base, top), false otherwise.
    ///
    pub fn contains(&self, addr: usize) -> (result: bool)
        requires
            self.inv(),
        ensures
            result == self@.contains_addr(addr as int),
    {
        addr >= self.base_addr && addr < self.base_addr + self.num_pages * PAGE_SIZE
    }


    /// Returns the page index for a given address within the stack.
    ///
    /// # Parameters
    ///
    /// - `addr`: The address to query (must be within stack bounds).
    ///
    /// # Returns
    ///
    /// The page index (0-based) containing the address.
    ///
    pub fn page_index(&self, addr: usize) -> (result: usize)
        requires
            self.inv(),
            self@.contains_addr(addr as int),
        ensures
            // Strengthened: Removed trivial `0 <= result as int` (always true for usize).
            // Added tighter bounds with spec functions.
            (result as int) < self.spec_num_pages(),
            result <= MAX_STACK_PAGES,
            self@.addr_in_page(addr as int, result as int),
    {
        (addr - self.base_addr) / PAGE_SIZE
    }

    //==============================================================================================

    /// Returns the initial stack pointer value.
    ///
    /// # Description
    ///
    /// Returns the top address, which is where the stack pointer should be
    /// initialized. The stack will grow downward from this address.
    ///
    /// # Returns
    ///
    /// The initial stack pointer (same as top()).
    ///
    pub fn initial_sp(&self) -> (result: usize)
        requires
            self.inv(),
        ensures
            result as int == self.spec_top(),
            // Strengthened: initial_sp is page-aligned and greater than base.
            spec_is_page_aligned(result as int),
            result > self.spec_base(),
    {
        self.top()
    }


    /// Checks if the stack has room to grow by the given amount.
    ///
    /// # Parameters
    ///
    /// - `current_sp`: The current stack pointer.
    /// - `growth`: The number of bytes the stack needs to grow.
    ///
    /// # Returns
    ///
    /// True if there is room, false if it would overflow past base.
    ///
    pub fn has_room(&self, current_sp: usize, growth: usize) -> (result: bool)
        requires
            self.inv(),
            self@.contains_addr(current_sp as int) || current_sp as int == self.spec_top(),
        ensures
            result == (current_sp as int - growth as int >= self.spec_base()),
    {
        // Use subtraction instead of addition to avoid overflow.
        // If current_sp >= base_addr, and growth <= current_sp - base_addr,
        // then there is room.
        current_sp - self.base_addr >= growth
    }
}

} // verus!
