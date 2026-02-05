// Copyright(c) The Maintainers of Nanvix.
// Licensed under the MIT License.

// Specifications.

verus! {

//==================================================================================================

/// Checks if an address is page-aligned.
pub open spec fn spec_is_page_aligned(addr: int) -> bool {
    addr % (PAGE_SIZE as int) == 0
}


/// Checks if a size is page-aligned.
pub open spec fn spec_is_size_aligned(size: int) -> bool {
    size % (PAGE_SIZE as int) == 0
}


/// Computes the size in bytes for a given number of pages.
pub open spec fn spec_pages_to_bytes(num_pages: int) -> int {
    num_pages * (PAGE_SIZE as int)
}


/// Computes the top address given base and size.
pub open spec fn spec_compute_top(base: int, size: int) -> int {
    base + size
}


impl KernelStackView {
    //==============================================================================================
    // Basic Properties
    //==============================================================================================

    /// Returns the size of the stack in bytes.
    pub open spec fn size(&self) -> int {
        spec_pages_to_bytes(self.num_pages)
    }

    /// Returns the top address (first byte past the end).
    pub open spec fn top(&self) -> int {
        spec_compute_top(self.base_addr, self.size())
    }

    /// Returns true if the base address is page-aligned.
    pub open spec fn is_base_aligned(&self) -> bool {
        spec_is_page_aligned(self.base_addr)
    }

    /// Returns true if the number of pages is within valid bounds.
    pub open spec fn has_valid_page_count(&self) -> bool {
        0 < self.num_pages <= MAX_STACK_PAGES as int
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
            0 <= i < self.num_pages ==>
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

    /// Returns true if the view represents a well-formed kernel stack.
    pub open spec fn is_well_formed(&self) -> bool {
        &&& self.is_base_aligned()
        &&& self.has_valid_page_count()
        &&& self.no_overflow()
    }
}


impl View for KernelStack {
    type V = KernelStackView;

    closed spec fn view(&self) -> KernelStackView {
        KernelStackView {
            base_addr: self.base_addr as int,
            num_pages: self.num_pages as int,
        }
    }
}

impl KernelStack {
    //==============================================================================================

    /// Invariant for the kernel stack.
    ///
    /// Ensures internal consistency and memory safety guarantees.
    pub closed spec fn inv(&self) -> bool {
        // The view must be well-formed.
        &&& self@.is_well_formed()
        // Size must be correctly computed.
        &&& self@.size() == self.num_pages as int * (PAGE_SIZE as int)
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


    /// Spec function to get the number of pages.
    pub closed spec fn spec_num_pages(&self) -> int {
        self.num_pages as int
    }
}

} // verus!
