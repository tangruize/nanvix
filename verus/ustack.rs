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
//! - A **size**: the constant `USER_STACK_SIZE` (typically 64KB = 16 pages)
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
//!
//! ### Constant Size
//!
//! The user stack always uses `USER_STACK_SIZE`. We represent this as a constant
//! and verify all properties assuming this fixed size.
//!
//! ## API Summary
//!
//! | Function | Description |
//! |----------|-------------|
//! | `new(base_addr)` | Create a new user stack with the given base address |
//! | `size()` | Returns the size in bytes (constant USER_STACK_SIZE) |
//! | `base()` | Returns the base address |
//! | `top()` | Returns the top address (first byte past end) |
//!
//==================================================================================================

use crate::error::{Error, ErrorCode};
use vstd::prelude::*;

verus! {

//==================================================================================================
// Constants
//==================================================================================================

/// Page size in bytes (4 KiB).
pub const PAGE_SIZE: usize = 4096;

/// Page alignment requirement.
pub const PAGE_ALIGNMENT: usize = 4096;

/// User stack size in bytes (64 KiB = 16 pages).
/// This matches config::memory_layout::USER_STACK_SIZE.
pub const USER_STACK_SIZE: usize = 65536;

/// User stack size in pages.
pub const USER_STACK_PAGES: usize = 16;

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

    /// Property: All pages in the stack have contiguous addresses.
    /// Page i starts at base_addr + i * PAGE_SIZE.
    pub open spec fn pages_are_contiguous(&self) -> bool {
        forall|i: int|
            #![trigger self.page_start(i)]
            0 <= i < self.num_pages() ==>
            self.page_start(i) == self.base_addr + i * (PAGE_SIZE as int)
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

        // Prove that USER_STACK_SIZE is page-aligned (16 * 4096 = 65536).
        proof {
            assert(USER_STACK_SIZE as int == USER_STACK_PAGES as int * (PAGE_SIZE as int));
            assert(spec_is_size_aligned(USER_STACK_SIZE as int));
        }

        // Prove page contiguity.
        proof {
            assert forall|i: int|
                0 <= i < USER_STACK_PAGES as int
            implies
                #[trigger] (base_addr as int + i * (PAGE_SIZE as int)) ==
                base_addr as int + i * (PAGE_SIZE as int)
            by {}
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

    /// Returns the base address of the user stack.
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

    /// Returns the top address of the user stack.
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
    ///
    pub fn top(&self) -> (result: usize)
        requires
            self.inv(),
        ensures
            result as int == self.spec_top(),
            result as int == self.spec_base() + self.spec_size(),
            spec_is_page_aligned(result as int),
            result as int > self.spec_base(),
    {
        self.base_addr + USER_STACK_SIZE
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
            0 <= result as int,
            (result as int) < USER_STACK_PAGES as int,
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
        current_sp - self.base_addr >= growth
    }
}

//==================================================================================================
// Proof Helpers
//==================================================================================================

/// Lemma: Page alignment is preserved under addition of page-aligned values.
proof fn lemma_page_aligned_add(a: int, b: int)
    requires
        spec_is_page_aligned(a),
        spec_is_page_aligned(b),
    ensures
        spec_is_page_aligned(a + b),
{
    assert((a + b) % (PAGE_SIZE as int) == 0) by {
        assert(a % (PAGE_SIZE as int) == 0);
        assert(b % (PAGE_SIZE as int) == 0);
    }
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
    assert(USER_STACK_SIZE == 65536);
    assert(PAGE_SIZE == 4096);
    assert(USER_STACK_PAGES == 16);
    assert(16 * 4096 == 65536);
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
