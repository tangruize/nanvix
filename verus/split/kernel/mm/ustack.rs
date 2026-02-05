// Copyright(c) The Maintainers of Nanvix.
// Licensed under the MIT License.

//==================================================================================================
//! # User Stack (Verified Implementation)
//!
//! This module provides a verified implementation of the User Stack data structure.
//! The user stack manages a contiguous region of virtual memory used for user-mode
//! execution stacks.
//!
//! ## Overview
//!
//! A user stack is a fixed-size memory region defined by:
//! - A **base address**: the lowest virtual address of the stack (page-aligned)
//! - A **top address**: the first address past the end of the stack (base + size)
//! - A **size**: the constant `USER_STACK_SIZE` (512KB = 128 pages)
//!
//! Since stacks grow downward on x86, the stack pointer starts near the top and grows
//! toward the base.
//!
//! ## Memory Safety Properties Verified
//!
//! 1. **Page Alignment**: Base address is always page-aligned.
//! 2. **Constant Size**: Stack size is the fixed `USER_STACK_SIZE` constant.
//! 3. **Size Alignment**: Stack size is always a multiple of PAGE_SIZE.
//! 4. **Address Ordering**: top = base + size, so top > base (assuming size > 0).
//! 5. **No Overflow**: Address arithmetic does not overflow.
//!
//! ## Liveness Properties Verified
//!
//! 1. **Construction Success**: If base is page-aligned and no overflow, construction succeeds.
//! 2. **Accessor Availability**: base(), top(), size() always return valid values for
//!    a well-formed stack.
//!
//! ## Verification Scope and Abstraction Decisions
//!
//! ### Return Type Abstraction
//!
//! The original returns `PageAligned<VirtualAddress>` from `base()` and `top()`.
//! This verified version returns raw `usize` with alignment proven in postconditions:
//!
//! - **Type-level vs proof-level**: The original enforces alignment via the type system;
//!   the verification proves the same property via postconditions.
//! - **Equivalent guarantees**: Both approaches ensure callers receive aligned addresses.
//! - **Why not use PageAligned<T>?**: Verus modules cannot import kernel types directly.
//!   The postcondition `spec_is_page_aligned(result as int)` provides an equivalent
//!   guarantee that is machine-checked.
//!
//! ### Constructor Signature
//!
//! The original `new()` takes `PageAligned<VirtualAddress>` (infallible, type-level guarantee).
//! This version takes `usize` with preconditions (alignment must hold for caller). The
//! runtime checks are redundant when preconditions hold, but kept for defense-in-depth.
//!
//! - **Equivalence**: A caller providing `PageAligned<VirtualAddress>` satisfies the
//!   precondition `spec_is_page_aligned(base_addr as int)` by construction.
//! - **Postcondition**: We prove `result.is_ok()` when preconditions hold, matching
//!   the original's infallibility.
//!
//! ### Constant Size
//!
//! The user stack always uses `USER_STACK_SIZE`. We represent this as a constant
//! and verify all properties assuming this fixed size.
//!
//! ### Documentation Correction (base/top semantics) - ORIGINAL CODE BUG
//!
//! **IMPORTANT**: The original kernel code has contradictory documentation:
//! - Comments state: base = "highest address", top = "lowest address" (lines 59, 76)
//! - Implementation computes: `top = base + size` (line 79)
//!
//! If `top = base + size` and `size > 0`, then `top > base`, meaning top is HIGHER.
//! The comments contradict the implementation. This is a documentation bug in the
//! original code, not in this verified module.
//!
//! This verified version documents the **implementation-consistent** semantics:
//! - **base**: lowest address of the stack region (where stack is full)
//! - **top**: highest address, first byte past end (where stack pointer starts)
//!
//! The stack pointer starts at `top` and grows downward toward `base`.
//! This matches the implementation: `top() = base.into_raw_value() + size()`.
//!
//! ### Extended API
//!
//! This module includes helper methods (`contains`, `page_index`, `initial_sp`, `has_room`)
//! beyond the original API. These demonstrate additional verified properties and are
//! clearly marked as extensions.
//!
//! ### Type-Safe API (PageAlignedAddr)
//!
//! To mirror the original `PageAligned<VirtualAddress>` return types, this module provides:
//! - `PageAlignedAddr`: A newtype wrapper with an alignment invariant
//! - `base()`: Returns base as PageAlignedAddr (mirrors original `base()`)
//! - `top()`: Returns top as PageAlignedAddr (mirrors original `top()`)
//! - `from_aligned()`: Infallible constructor taking PageAlignedAddr (mirrors original `new()`)
//!
//! These preserve the type-level alignment guarantees from the original API.
//!
//! ## API Summary
//!
//! | Function | Description |
//! |----------|-------------|
//! | `new(base_addr)` | Create a new user stack (fallible, takes raw usize) |
//! | `from_aligned(base)` | Create a new user stack (infallible, takes PageAlignedAddr) |
//! | `size()` | Returns the size in bytes (constant USER_STACK_SIZE) |
//! | `base()` | Returns the base address as PageAlignedAddr |
//! | `top()` | Returns the top address as PageAlignedAddr |
//! | `base_raw()` | Returns the base address as raw usize |
//! | `top_raw()` | Returns the top address as raw usize |
//!
//==================================================================================================

use crate::libs::error::{
    Error,
    ErrorCode,
};
use vstd::prelude::*;

verus! {

//==================================================================================================
// Constants
//==================================================================================================
//
// IMPORTANT: Configuration Linkage
//
// These constants MUST match the kernel configuration in:
//   - config::memory_layout::USER_STACK_SIZE (512 * KILOBYTE = 524288)
//   - arch::PAGE_SIZE (4096)
//
// The verification is only valid when these values match the actual kernel build.
// If the kernel configuration changes, these constants must be updated accordingly.
//
// Verus modules cannot directly import kernel crates, so we duplicate these values.
//
// CI INTEGRATION:
// Run `scripts/verify-verus-constants.sh` to verify these constants match the kernel.
// This script is included in the CI pipeline to prevent configuration drift.
//
// VERIFICATION: The lemma `lemma_constants_valid` below proves internal consistency
// of these constants. External linkage is verified by the CI script above.
//==================================================================================================

/// Page size in bytes (4 KiB).
/// Must match: arch::PAGE_SIZE = 4096
pub const PAGE_SIZE: usize = 4096;

/// Page alignment requirement.
/// Must match: PAGE_SIZE (pages are aligned to their size)
pub const PAGE_ALIGNMENT: usize = 4096;

/// User stack size in bytes (512 KiB = 128 pages).
/// Must match: config::memory_layout::USER_STACK_SIZE = 512 * KILOBYTE = 524288
pub const USER_STACK_SIZE: usize = 524288;

/// User stack size in pages (512 KiB / 4 KiB = 128 pages).
/// Derived: USER_STACK_SIZE / PAGE_SIZE = 128
pub const USER_STACK_PAGES: usize = 128;

/// Kilobyte constant for documentation (1024 bytes).
pub const KILOBYTE: usize = 1024;

/// Lemma: Verify internal consistency of constants.
///
/// This proves that our constant definitions are internally consistent.
/// External linkage (matching kernel config) must be verified separately.
proof fn lemma_constants_valid()
    ensures
        PAGE_SIZE == 4096,
        PAGE_ALIGNMENT == PAGE_SIZE,
        USER_STACK_SIZE == 512 * KILOBYTE,
        USER_STACK_PAGES == USER_STACK_SIZE / PAGE_SIZE,
        USER_STACK_SIZE % PAGE_SIZE == 0,
        KILOBYTE == 1024,
{
    assert(PAGE_SIZE == 4096usize);
    assert(PAGE_ALIGNMENT == 4096usize);
    assert(KILOBYTE == 1024usize);
    assert(512usize * 1024usize == 524288usize);
    assert(USER_STACK_SIZE == 524288usize);
    assert(524288usize / 4096usize == 128usize);
    assert(USER_STACK_PAGES == 128usize);
    assert(524288usize % 4096usize == 0usize);
}

//==================================================================================================
// Specification Helper Functions
//==================================================================================================

/// Checks if an address is page-aligned.
pub open spec fn spec_is_page_aligned(addr: int) -> bool {
    addr % (PAGE_SIZE as int) == 0
}

/// Checks if a size is page-aligned.
pub open spec fn spec_is_size_aligned(size: int) -> bool {
    size % (PAGE_SIZE as int) == 0
}

/// Computes the top address given base and size.
pub open spec fn spec_compute_top(base: int, size: int) -> int {
    base + size
}

//==================================================================================================
// API Equivalence Model
//==================================================================================================
//
// This section provides a formal model of the original API types and proves equivalence
// between the verified module's guarantees and the original kernel API.
//
// Original API:
//   - PageAligned<VirtualAddress>: A newtype wrapper ensuring alignment at construction
//   - new(base: PageAligned<VirtualAddress>) -> Self: Infallible constructor
//   - base() -> PageAligned<VirtualAddress>: Returns aligned base
//   - top() -> PageAligned<VirtualAddress>: Returns aligned top (base + size)
//
// Verified API equivalence:
//   - spec_is_page_aligned(addr) == true <==> addr could be wrapped in PageAligned
//   - Precondition spec_is_page_aligned(base) <==> caller has PageAligned<VirtualAddress>
//   - Postcondition spec_is_page_aligned(result) <==> result could be PageAligned
//==================================================================================================

//==================================================================================================
// PageAlignedAddr - Type-Level Alignment Wrapper
//==================================================================================================
//
// This type mirrors the kernel's `PageAligned<VirtualAddress>` to provide type-level
// alignment guarantees. The invariant ensures the wrapped address is always page-aligned.
//==================================================================================================

/// A page-aligned virtual address.
///
/// This type mirrors `PageAligned<VirtualAddress>` from the kernel, providing
/// type-level guarantees that the wrapped address is page-aligned.
///
/// # Invariant
///
/// The wrapped address is always page-aligned: `addr % PAGE_SIZE == 0`.
pub struct PageAlignedAddr {
    addr: usize,
}

impl PageAlignedAddr {
    /// Invariant: The address is page-aligned.
    pub closed spec fn inv(&self) -> bool {
        spec_is_page_aligned(self.addr as int)
    }

    /// Spec function to get the raw address value.
    pub closed spec fn spec_addr(&self) -> int {
        self.addr as int
    }

    /// Creates a new PageAlignedAddr from a raw address.
    ///
    /// # Preconditions
    ///
    /// The address must be page-aligned.
    ///
    /// # Returns
    ///
    /// A PageAlignedAddr wrapping the given address.
    pub fn from_raw(addr: usize) -> (result: Option<Self>)
        ensures
            // Liveness: alignment implies success.
            spec_is_page_aligned(addr as int) ==> result.is_some(),
            // Safety: success implies well-formed result.
            result.is_some() ==> {
                let pa = result.unwrap();
                &&& pa.inv()
                &&& pa.spec_addr() == addr as int
            },
            // Error case: failure implies misalignment.
            result.is_none() ==> !spec_is_page_aligned(addr as int),
    {
        if addr % PAGE_ALIGNMENT == 0 {
            Some(PageAlignedAddr { addr })
        } else {
            None
        }
    }

    /// Creates a new PageAlignedAddr from a raw address (unchecked).
    ///
    /// # Preconditions
    ///
    /// The address must be page-aligned.
    pub fn from_raw_unchecked(addr: usize) -> (result: Self)
        requires
            spec_is_page_aligned(addr as int),
        ensures
            result.inv(),
            result.spec_addr() == addr as int,
    {
        PageAlignedAddr { addr }
    }

    /// Returns the raw address value.
    pub fn into_raw(&self) -> (result: usize)
        requires
            self.inv(),
        ensures
            result as int == self.spec_addr(),
            spec_is_page_aligned(result as int),
    {
        self.addr
    }
}

//==================================================================================================
// UserStackView - Abstract Specification
//==================================================================================================

/// Abstract view of a user stack for specification purposes.
///
/// This view captures the essential properties of a user stack without
/// exposing implementation details.
#[verifier::ext_equal]
pub struct UserStackView {
    /// Base virtual address of the stack (lowest address).
    pub base_addr: int,
}

impl UserStackView {
    //==============================================================================================
    // Basic Properties
    //==============================================================================================

    /// Returns the size of the stack in bytes (constant).
    pub open spec fn size(&self) -> int {
        USER_STACK_SIZE as int
    }

    /// Returns the number of pages in the stack.
    pub open spec fn num_pages(&self) -> int {
        USER_STACK_PAGES as int
    }

    /// Returns the top address (first byte past the end).
    pub open spec fn top(&self) -> int {
        spec_compute_top(self.base_addr, self.size())
    }

    /// Returns true if the base address is page-aligned.
    pub open spec fn is_base_aligned(&self) -> bool {
        spec_is_page_aligned(self.base_addr)
    }

    //==============================================================================================
    // Memory Safety Properties
    //==============================================================================================

    /// Property: The stack size is page-aligned.
    pub open spec fn size_is_aligned(&self) -> bool {
        spec_is_size_aligned(self.size())
    }

    /// Property: The top address is page-aligned (follows from base and size alignment).
    pub open spec fn top_is_aligned(&self) -> bool {
        spec_is_page_aligned(self.top())
    }

    /// Property: Top is greater than base (stack has positive size).
    pub open spec fn top_greater_than_base(&self) -> bool {
        self.top() > self.base_addr
    }

    /// Property: Address arithmetic does not overflow.
    pub open spec fn no_overflow(&self) -> bool {
        self.base_addr >= 0 &&
        self.base_addr + self.size() <= usize::MAX as int
    }

    /// Property: All pages in the stack are contiguous.
    /// Page i ends exactly where page i+1 starts (no gaps, no overlaps).
    pub open spec fn pages_are_contiguous(&self) -> bool {
        forall|i: int|
            0 <= i < self.num_pages() - 1 ==>
            self.page_end(i) == self.page_start(i + 1)
    }

    /// Returns the start address of page i.
    pub open spec fn page_start(&self, i: int) -> int {
        self.base_addr + i * (PAGE_SIZE as int)
    }

    /// Returns the end address of page i (exclusive).
    pub open spec fn page_end(&self, i: int) -> int {
        self.base_addr + (i + 1) * (PAGE_SIZE as int)
    }

    /// Property: Address is within page bounds.
    pub open spec fn addr_in_page(&self, addr: int, page_idx: int) -> bool {
        self.page_start(page_idx) <= addr && addr < self.page_end(page_idx)
    }

    /// Property: A given address is within the stack bounds.
    pub open spec fn contains_addr(&self, addr: int) -> bool {
        self.base_addr <= addr && addr < self.top()
    }

    //==============================================================================================
    // Well-formedness
    //==============================================================================================

    /// Returns true if the view represents a well-formed user stack.
    pub open spec fn is_well_formed(&self) -> bool {
        &&& self.is_base_aligned()
        &&& self.no_overflow()
    }
}

//==================================================================================================
// UserStack - Concrete Implementation
//==================================================================================================

/// A type that represents a user stack.
///
/// The user stack is a contiguous region of virtual memory used for user-mode
/// execution. It has a fixed size of USER_STACK_SIZE bytes.
///
/// # Debug Formatting Note
///
/// This type uses `#[derive(Debug)]` for simplicity. The original kernel implementation
/// has a custom `fmt::Debug` that formats as `UserStack { base: ..., top: ..., size=... }`.
/// The derived Debug produces a different format: `UserStack { base_addr: ... }`.
/// This difference is cosmetic and does not affect correctness properties.
#[derive(Debug)]
pub struct UserStack {
    /// Base virtual address of the stack.
    base_addr: usize,
}

impl View for UserStack {
    type V = UserStackView;

    closed spec fn view(&self) -> UserStackView {
        UserStackView {
            base_addr: self.base_addr as int,
        }
    }
}

impl UserStack {
    //==============================================================================================
    // Invariant
    //==============================================================================================

    /// Invariant for the user stack.
    ///
    /// Ensures internal consistency and memory safety guarantees.
    pub closed spec fn inv(&self) -> bool {
        // The view must be well-formed.
        &&& self@.is_well_formed()
        // Size is the constant USER_STACK_SIZE.
        &&& self@.size() == USER_STACK_SIZE as int
        // Top must be correctly computed.
        &&& self@.top() == self.base_addr as int + self@.size()
        // Size is page-aligned.
        &&& self@.size_is_aligned()
        // Top is page-aligned.
        &&& self@.top_is_aligned()
        // Top is greater than base.
        &&& self@.top_greater_than_base()
        // Pages are contiguous.
        &&& self@.pages_are_contiguous()
    }

    //==============================================================================================
    // Specification Functions
    //==============================================================================================

    /// Spec function to get the base address.
    pub closed spec fn spec_base(&self) -> int {
        self.base_addr as int
    }

    /// Spec function to get the top address.
    pub closed spec fn spec_top(&self) -> int {
        self@.top()
    }

    /// Spec function to get the size in bytes.
    pub closed spec fn spec_size(&self) -> int {
        self@.size()
    }

    //==============================================================================================
    // Constructor
    //==============================================================================================

    /// Instantiates a new user stack.
    ///
    /// # Description
    ///
    /// Creates a new user stack with the given base address.
    /// The base address must be page-aligned and the address arithmetic
    /// must not overflow.
    ///
    /// # Parameters
    ///
    /// - `base_addr`: The base virtual address of the stack (must be page-aligned).
    ///
    /// # Returns
    ///
    /// Upon success, returns the new user stack. Upon failure, returns an error.
    ///
    /// # Errors
    ///
    /// - `InvalidArgument`: If base_addr is not page-aligned.
    /// - `OutOfMemory`: If the address arithmetic would overflow.
    ///
    pub fn new(base_addr: usize) -> (result: Result<Self, Error>)
        requires
            spec_is_page_aligned(base_addr as int),
            base_addr as int + (USER_STACK_SIZE as int) <= usize::MAX as int,
        ensures
            result.is_ok(),
            result.is_ok() ==> {
                let stack = result.unwrap();
                &&& stack.inv()
                &&& stack.spec_base() == base_addr as int
                &&& stack.spec_size() == USER_STACK_SIZE as int
                &&& stack.spec_top() == base_addr as int + (USER_STACK_SIZE as int)
            },
    {
        // Validate page alignment.
        if base_addr % PAGE_ALIGNMENT != 0 {
            return Err(Error::new(ErrorCode::InvalidArgument, "base address not page-aligned"));
        }

        // Check for overflow.
        if base_addr > usize::MAX - USER_STACK_SIZE {
            return Err(Error::new(ErrorCode::OutOfMemory, "address overflow"));
        }

        // Prove that USER_STACK_SIZE is page-aligned (128 * 4096 = 524288).
        proof {
            assert(USER_STACK_SIZE as int == USER_STACK_PAGES as int * (PAGE_SIZE as int));
            assert(spec_is_size_aligned(USER_STACK_SIZE as int));
        }

        // Prove page contiguity (consecutive pages are adjacent).
        proof {
            assert forall|i: int|
                0 <= i < USER_STACK_PAGES as int - 1
            implies
                #[trigger] (base_addr as int + (i + 1) * (PAGE_SIZE as int)) ==
                base_addr as int + i * (PAGE_SIZE as int) + (PAGE_SIZE as int)
            by {
                // page_end(i) = base_addr + (i+1) * PAGE_SIZE
                // page_start(i+1) = base_addr + (i+1) * PAGE_SIZE
                // They are equal by definition.
            }
        }

        let stack = UserStack { base_addr };

        // Prove the invariant holds.
        proof {
            // Prove size alignment.
            assert(spec_is_size_aligned(stack@.size())) by {
                assert(stack@.size() == USER_STACK_SIZE as int);
            }

            // Prove top alignment.
            assert(spec_is_page_aligned(stack@.top())) by {
                assert(stack@.top() == base_addr as int + (USER_STACK_SIZE as int));
                // base_addr is page-aligned and USER_STACK_SIZE is a multiple of PAGE_SIZE.
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

    /// Instantiates a new user stack from a PageAlignedAddr (infallible).
    ///
    /// # Description
    ///
    /// This constructor mirrors the original `new(base: PageAligned<VirtualAddress>) -> Self`
    /// which is infallible because alignment is guaranteed by the type.
    ///
    /// # Parameters
    ///
    /// - `base`: The page-aligned base address of the stack.
    ///
    /// # Returns
    ///
    /// A new user stack.
    ///
    pub fn from_aligned(base: PageAlignedAddr) -> (result: Self)
        requires
            base.inv(),
            base.spec_addr() + (USER_STACK_SIZE as int) <= usize::MAX as int,
        ensures
            result.inv(),
            result.spec_base() == base.spec_addr(),
            result.spec_size() == USER_STACK_SIZE as int,
            result.spec_top() == base.spec_addr() + (USER_STACK_SIZE as int),
    {
        let base_addr = base.into_raw();

        // Prove that USER_STACK_SIZE is page-aligned.
        proof {
            assert(USER_STACK_SIZE as int == USER_STACK_PAGES as int * (PAGE_SIZE as int));
            assert(spec_is_size_aligned(USER_STACK_SIZE as int));
        }

        let stack = UserStack { base_addr };

        // Prove the invariant holds.
        proof {
            // Prove size alignment.
            assert(spec_is_size_aligned(stack@.size())) by {
                assert(stack@.size() == USER_STACK_SIZE as int);
            }

            // Prove top alignment.
            assert(spec_is_page_aligned(stack@.top())) by {
                assert(stack@.top() == base_addr as int + (USER_STACK_SIZE as int));
            }

            // Prove top > base.
            assert(stack@.top_greater_than_base()) by {
                assert(stack@.size() > 0);
            }

            // Prove pages are contiguous.
            assert(stack@.pages_are_contiguous());
        }

        stack
    }

    //==============================================================================================
    // Accessors
    //==============================================================================================

    /// Returns the size of the user stack in bytes.
    ///
    /// # Description
    ///
    /// The size is always the constant USER_STACK_SIZE.
    ///
    /// # Returns
    ///
    /// The size of the user stack.
    ///
    pub fn size(&self) -> (result: usize)
        requires
            self.inv(),
        ensures
            result as int == self.spec_size(),
            result == USER_STACK_SIZE,
            result % PAGE_SIZE == 0,
    {
        USER_STACK_SIZE
    }

    /// Returns the base address of the user stack as raw usize.
    ///
    /// # Description
    ///
    /// The base address is the lowest virtual address of the stack.
    /// Since stacks grow downward, this is where the stack ends when full.
    ///
    /// # Returns
    ///
    /// The base address of the user stack (page-aligned).
    ///
    /// # Notes
    ///
    /// As stacks grow downwards, the base address is the lowest address of the stack.
    /// For type-safe access, use `base()` which returns `PageAlignedAddr`.
    ///
    pub fn base_raw(&self) -> (result: usize)
        requires
            self.inv(),
        ensures
            result as int == self.spec_base(),
            spec_is_page_aligned(result as int),
    {
        self.base_addr
    }

    /// Returns the top address of the user stack as raw usize.
    ///
    /// # Description
    ///
    /// The top address is the first byte past the end of the stack.
    /// Since stacks grow downward, the stack pointer typically starts at top - 1
    /// (or at a word-aligned address just below top).
    ///
    /// # Returns
    ///
    /// The top address of the user stack (page-aligned).
    ///
    /// # Notes
    ///
    /// As stacks grow downwards, the top address is the highest address of the stack.
    /// For type-safe access, use `top()` which returns `PageAlignedAddr`.
    ///
    pub fn top_raw(&self) -> (result: usize)
        requires
            self.inv(),
        ensures
            result as int == self.spec_top(),
            spec_is_page_aligned(result as int),
            result as int > self.spec_base(),
    {
        self.base_addr + USER_STACK_SIZE
    }

    //==============================================================================================
    // Type-Safe Accessors (mirrors original PageAligned<VirtualAddress> API)
    //==============================================================================================

    /// Returns the base address as a PageAlignedAddr.
    ///
    /// This method mirrors the original API that returns `PageAligned<VirtualAddress>`.
    /// It is the primary accessor for base address, providing type-level alignment guarantees.
    pub fn base(&self) -> (result: PageAlignedAddr)
        requires
            self.inv(),
        ensures
            result.inv(),
            result.spec_addr() == self.spec_base(),
    {
        PageAlignedAddr::from_raw_unchecked(self.base_addr)
    }

    /// Returns the top address as a PageAlignedAddr.
    ///
    /// This method mirrors the original API that returns `PageAligned<VirtualAddress>`.
    /// It is the primary accessor for top address, providing type-level alignment guarantees.
    pub fn top(&self) -> (result: PageAlignedAddr)
        requires
            self.inv(),
        ensures
            result.inv(),
            result.spec_addr() == self.spec_top(),
    {
        PageAlignedAddr::from_raw_unchecked(self.base_addr + USER_STACK_SIZE)
    }

    //==============================================================================================
    // Address Queries
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
        addr >= self.base_addr && addr < self.base_addr + USER_STACK_SIZE
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
            // Precise computation: result equals the floor division of offset by page size.
            result as int == (addr as int - self.spec_base()) / (PAGE_SIZE as int),
            // Upper bound: result is within valid page range.
            (result as int) < USER_STACK_PAGES as int,
            // Semantic property: the address is within the computed page.
            self@.addr_in_page(addr as int, result as int),
    {
        (addr - self.base_addr) / PAGE_SIZE
    }

    //==============================================================================================
    // Stack Pointer Helpers
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
            result as int > self.spec_base(),
    {
        self.top_raw()
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
        current_sp - self.base_addr >= growth
    }
}

//==================================================================================================
// Proof Helpers
//==================================================================================================

/// Lemma: Page alignment is preserved under addition of page-aligned values.
///
/// Uses modular arithmetic: (a + b) % p = ((a % p) + (b % p)) % p = (0 + 0) % p = 0.
proof fn lemma_page_aligned_add(a: int, b: int)
    requires
        spec_is_page_aligned(a),
        spec_is_page_aligned(b),
    ensures
        spec_is_page_aligned(a + b),
{
    let p = PAGE_SIZE as int;
    // By preconditions: a % p == 0 and b % p == 0.
    // By modular arithmetic: (a + b) % p == ((a % p) + (b % p)) % p == (0 + 0) % p == 0.
    assert(a % p == 0);
    assert(b % p == 0);
    assert((a + b) % p == 0);
}

/// Lemma: Page multiplication produces page-aligned results.
proof fn lemma_page_mult_aligned(n: int)
    requires
        n >= 0,
    ensures
        spec_is_page_aligned(n * (PAGE_SIZE as int)),
{
    assert((n * (PAGE_SIZE as int)) % (PAGE_SIZE as int) == 0);
}

/// Lemma: USER_STACK_SIZE is page-aligned.
proof fn lemma_user_stack_size_aligned()
    ensures
        spec_is_size_aligned(USER_STACK_SIZE as int),
        USER_STACK_SIZE as int == USER_STACK_PAGES as int * (PAGE_SIZE as int),
{
    assert(USER_STACK_SIZE == 524288);
    assert(PAGE_SIZE == 4096);
    assert(USER_STACK_PAGES == 128);
    assert(128 * 4096 == 524288);
}

/// Lemma: A well-formed stack has aligned top.
proof fn lemma_well_formed_has_aligned_top(view: UserStackView)
    requires
        view.is_well_formed(),
    ensures
        view.top_is_aligned(),
{
    lemma_user_stack_size_aligned();
    lemma_page_aligned_add(view.base_addr, view.size());
}

/// Lemma: Stack pages are disjoint from each other.
proof fn lemma_pages_disjoint(view: UserStackView, i: int, j: int)
    requires
        view.is_well_formed(),
        0 <= i < view.num_pages(),
        0 <= j < view.num_pages(),
        i != j,
    ensures
        view.page_end(i) <= view.page_start(j) || view.page_end(j) <= view.page_start(i),
{
    // Pages are laid out sequentially, so if i < j, page i ends before page j starts.
    if i < j {
        assert(view.page_end(i) == view.base_addr + (i + 1) * (PAGE_SIZE as int));
        assert(view.page_start(j) == view.base_addr + j * (PAGE_SIZE as int));
        assert(i + 1 <= j);
        assert(view.page_end(i) <= view.page_start(j));
    } else {
        // j < i
        assert(view.page_end(j) == view.base_addr + (j + 1) * (PAGE_SIZE as int));
        assert(view.page_start(i) == view.base_addr + i * (PAGE_SIZE as int));
        assert(j + 1 <= i);
        assert(view.page_end(j) <= view.page_start(i));
    }
}

/// Lemma: All addresses in a page are within stack bounds.
proof fn lemma_page_in_bounds(view: UserStackView, page_idx: int, offset: int)
    requires
        view.is_well_formed(),
        0 <= page_idx < view.num_pages(),
        0 <= offset < PAGE_SIZE as int,
    ensures
        view.contains_addr(view.page_start(page_idx) + offset),
{
    let addr = view.page_start(page_idx) + offset;
    assert(addr >= view.base_addr);
    assert(addr < view.page_end(page_idx));
    assert(view.page_end(page_idx) <= view.top());
    assert(addr < view.top());
}

} // verus!
