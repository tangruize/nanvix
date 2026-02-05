// Copyright(c) The Maintainers of Nanvix.
// Licensed under the MIT License.

// Specifications.

verus! {

impl PageAddress {
    //==============================================================================================

    /// Spec function to get the raw address value.
    pub open spec fn spec_raw_value(&self) -> int {
        self.raw_addr as int
    }


    /// Spec function to check if address is page-aligned.
    pub open spec fn spec_is_aligned(&self) -> bool {
        self.raw_addr as int % PAGE_SIZE as int == 0
    }


    /// Spec function to get the page table entry index.
    /// This extracts bits [12:21] of the address, giving a value 0-1023.
    /// The formula models: (addr & (PGTAB_MASK ^ PAGE_MASK)) >> PAGE_SHIFT
    /// Which is equivalent to: (addr / PAGE_SIZE) % 1024
    pub open spec fn spec_pte_index(&self) -> int {
        (self.raw_addr as int / PAGE_SIZE as int) % 1024
    }


    /// Spec function to compare two page addresses.
    pub open spec fn spec_cmp(&self, other: &Self) -> core::cmp::Ordering {
        if self.raw_addr < other.raw_addr {
            core::cmp::Ordering::Less
        } else if self.raw_addr > other.raw_addr {
            core::cmp::Ordering::Greater
        } else {
            core::cmp::Ordering::Equal
        }
    }
}


impl PageAddressEqSpec for PageAddress {
    /// PageAddress obeys the equality specification.
    open spec fn obeys_eq_spec() -> bool {
        true
    }

    /// Two page addresses are equal if their raw values are equal.
    open spec fn eq_spec(&self, other: &Self) -> bool {
        self.raw_addr == other.raw_addr
    }
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

    //==============================================================================================
    // Memory Safety Properties
    //==============================================================================================

    /// Property: The page address is non-negative (valid address space).
    pub open spec fn addr_is_valid(&self) -> bool {
        self.page_addr >= 0 && self.frame_addr >= 0
    }
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

    /// Spec function to get the pool ID of the underlying frame.
    pub closed spec fn spec_pool_id(&self) -> int {
        self@.pool_id()
    }
}

} // verus!
