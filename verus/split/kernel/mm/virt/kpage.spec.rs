// Copyright(c) The Maintainers of Nanvix.
// Licensed under the MIT License.

// Specifications.

verus! {

//==================================================================================================
// PageAddress Specifications
//==================================================================================================

impl PageAddressView {
    /// Returns the raw address value.
    pub open spec fn raw_value(&self) -> int {
        self.raw_value
    }

    /// Property: The address is page-aligned.
    pub open spec fn is_aligned(&self) -> bool {
        self.raw_value % PAGE_SIZE as int == 0
    }

    /// Returns the page table entry index.
    /// This extracts bits [12:21] of the address, giving a value 0-1023.
    /// The formula models: (addr & (PGTAB_MASK ^ PAGE_MASK)) >> PAGE_SHIFT
    /// Which is equivalent to: (addr / PAGE_SIZE) % 1024
    pub open spec fn pte_index(&self) -> int {
        (self.raw_value / PAGE_SIZE as int) % 1024
    }

    /// Compares two page address views.
    pub open spec fn cmp(&self, other: &Self) -> core::cmp::Ordering {
        if self.raw_value < other.raw_value {
            core::cmp::Ordering::Less
        } else if self.raw_value > other.raw_value {
            core::cmp::Ordering::Greater
        } else {
            core::cmp::Ordering::Equal
        }
    }
}

impl View for PageAddress {
    type V = PageAddressView;

    /// Converts a PageAddress to its abstract view.
    ///
    /// This is `closed` so users cannot see the internal representation.
    closed spec fn view(&self) -> PageAddressView {
        PageAddressView { raw_value: self.raw_addr as int }
    }
}

impl PageAddress {
    //==============================================================================================

    /// Invariant for the page address.
    ///
    /// Ensures that the raw address is page-aligned.
    pub closed spec fn inv(&self) -> bool {
        self@.is_aligned()
    }
}


/// The `PageAddressEqSpec` trait methods are `open spec fn` without explicit `pub` visibility.
/// This is a required workaround for vstd's PartialEq trait mechanism, which expects
/// `obeys_eq_spec()` and `eq_spec()` to be provided via an external trait extension.
/// The trait itself is `pub`, making these methods effectively public.
impl PageAddressEqSpec for PageAddress {
    /// PageAddress obeys the equality specification.
    open spec fn obeys_eq_spec() -> bool {
        true
    }

    /// Two page addresses are equal if their abstract raw values are equal.
    open spec fn eq_spec(&self, other: &Self) -> bool {
        self@.raw_value() == other@.raw_value()
    }
}

//==================================================================================================
// KernelPageView Specifications
//==================================================================================================

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
}

} // verus!
