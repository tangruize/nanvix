// Copyright(c) The Maintainers of Nanvix.
// Licensed under the MIT License.

// Specifications.
// Methodology: Step 1 (abstraction), Step 2 (invariant), Step 3 (public specs).

verus! {

//==================================================================================================
// View Types (Step 1)
//==================================================================================================

/// Abstract view of a FrameNumber.
///
/// # Description
///
/// Uses `int` instead of `usize` for abstract reasoning (methodology Step 1).
pub struct FrameNumberView {
    /// The abstract value of the frame number.
    pub value: int,
}

/// Abstract view of a FrameAddress.
///
/// # Description
///
/// Uses `int` instead of `usize` for abstract reasoning (methodology Step 1).
pub struct FrameAddressView {
    /// The abstract raw address value.
    pub raw_value: int,
}

/// Abstract view of a PageAlignedPhysAddr.
///
/// # Description
///
/// Uses `int` instead of `usize` for abstract reasoning (methodology Step 1).
pub struct PageAlignedPhysAddrView {
    /// The abstract raw address value.
    pub raw_value: int,
}

/// Abstract view of a TruncatedMemoryRegion.
///
/// # Description
///
/// Uses `int` instead of `usize` for abstract reasoning (methodology Step 1).
pub struct TruncatedMemoryRegionView {
    /// The abstract start address value.
    pub start: int,
    /// The abstract size in bytes.
    pub size: int,
}

//==================================================================================================
// View Type Helpers
//==================================================================================================

impl FrameAddressView {
    /// The frame number derived from this address.
    pub open spec fn frame_number(&self) -> int {
        self.raw_value / FRAME_SIZE as int
    }

    /// Whether this address is page-aligned.
    pub open spec fn is_aligned(&self) -> bool {
        self.raw_value % FRAME_SIZE as int == 0
    }
}

impl PageAlignedPhysAddrView {
    /// The frame number derived from this address.
    pub open spec fn frame_number(&self) -> int {
        self.raw_value / FRAME_SIZE as int
    }
}

impl TruncatedMemoryRegionView {
    /// The start frame number.
    pub open spec fn start_frame(&self) -> int {
        self.start / FRAME_SIZE as int
    }

    /// The number of frames in this region.
    pub open spec fn frame_count(&self) -> int {
        self.size / FRAME_SIZE as int
    }
}

//==================================================================================================
// View Implementations (closed per Step 1)
//==================================================================================================

impl View for FrameNumber {
    type V = FrameNumberView;

    // Closed per methodology Step 1: hides implementation internals from users.
    closed spec fn view(&self) -> FrameNumberView {
        FrameNumberView { value: self.value as int }
    }
}

impl View for FrameAddress {
    type V = FrameAddressView;

    // Closed per methodology Step 1: hides implementation internals from users.
    closed spec fn view(&self) -> FrameAddressView {
        FrameAddressView { raw_value: self.raw_addr as int }
    }
}

impl View for PageAlignedPhysAddr {
    type V = PageAlignedPhysAddrView;

    // Closed per methodology Step 1: hides implementation internals from users.
    closed spec fn view(&self) -> PageAlignedPhysAddrView {
        PageAlignedPhysAddrView { raw_value: self.raw_addr as int }
    }
}

impl View for TruncatedMemoryRegion {
    type V = TruncatedMemoryRegionView;

    // Closed per methodology Step 1: hides implementation internals from users.
    closed spec fn view(&self) -> TruncatedMemoryRegionView {
        TruncatedMemoryRegionView {
            start: self.start.raw_addr as int,
            size: self.size as int,
        }
    }
}

//==================================================================================================
// Invariants (Step 2)
//==================================================================================================

impl FrameNumber {
    /// Invariant for FrameNumber (methodology Step 2).
    ///
    /// # Description
    ///
    /// A FrameNumber must be within the valid range [0, MAX_FRAME_NUMBER].
    /// This matches the original `FrameNumber::from_raw_value` which rejects
    /// values greater than `FrameNumber::MAX`.
    ///
    /// # Note on pub fields
    ///
    /// The `value` field is `pub` due to a Verus limitation: `pub open spec fn`
    /// definitions require field expressions to be visible at all scopes, and
    /// Verus does not support `pub(crate)` or private fields in this context.
    /// Direct struct literal construction can bypass this invariant. Callers
    /// must use `from_raw_value()` to ensure the invariant is established.
    pub closed spec fn inv(&self) -> bool {
        self.value as int <= MAX_FRAME_NUMBER as int
    }

    // NOTE: spec_raw_value is kept as a pub open backward-compatible helper
    // for cross-module callers. Per methodology Step 3, new callers should
    // prefer self@.value instead.
    /// Spec function to get the raw value.
    pub open spec fn spec_raw_value(&self) -> int {
        self.value as int
    }
}

impl FrameAddress {
    /// Invariant for FrameAddress (methodology Step 2).
    ///
    /// # Description
    ///
    /// A FrameAddress must be page-aligned (a multiple of FRAME_SIZE).
    /// This is the structural invariant maintained by all constructors.
    ///
    /// # Note on pub fields
    ///
    /// The `raw_addr` field is `pub` due to a Verus limitation (same as
    /// FrameNumber). Direct struct literal construction can bypass this
    /// invariant. Callers must use constructors (`new`, `from_frame_number`,
    /// `from_raw_value`) to ensure the invariant is established.
    pub closed spec fn inv(&self) -> bool {
        self.raw_addr as int % FRAME_SIZE as int == 0
    }

    // NOTE: The following spec functions are kept as pub open backward-compatible
    // helpers for cross-module callers. Per methodology Step 3, new callers
    // should prefer self@.raw_value, self@.frame_number(), self@.is_aligned().

    /// Spec function to get the raw address value.
    pub open spec fn spec_raw_value(&self) -> int {
        self.raw_addr as int
    }


    /// Spec function to get the frame number.
    pub open spec fn spec_frame_number(&self) -> int {
        self.raw_addr as int / FRAME_SIZE as int
    }


    /// Spec function to check if address is page-aligned.
    pub open spec fn spec_is_aligned(&self) -> bool {
        self.raw_addr as int % FRAME_SIZE as int == 0
    }
}

impl PageAlignedPhysAddr {
    /// Invariant for PageAlignedPhysAddr (methodology Step 2).
    ///
    /// # Description
    ///
    /// A PageAlignedPhysAddr must be page-aligned (a multiple of FRAME_SIZE).
    /// This is the structural invariant maintained by all constructors.
    ///
    /// # Note on pub fields
    ///
    /// The `raw_addr` field is `pub` due to a Verus limitation (same as
    /// FrameNumber). Direct struct literal construction can bypass this
    /// invariant. Callers must use `from_raw_value()` to ensure the invariant
    /// is established.
    pub closed spec fn inv(&self) -> bool {
        self.raw_addr as int % FRAME_SIZE as int == 0
    }

    // NOTE: The following spec functions are kept as pub open backward-compatible
    // helpers for cross-module callers. Per methodology Step 3, new callers
    // should prefer self@.raw_value, self@.frame_number().

    /// Spec function to get the raw address value.
    pub open spec fn spec_raw_value(&self) -> int {
        self.raw_addr as int
    }


    /// Spec function to get the frame number.
    pub open spec fn spec_frame_number(&self) -> int {
        self.raw_addr as int / FRAME_SIZE as int
    }
}

impl TruncatedMemoryRegion {
    /// Spec function to get the start address.
    pub closed spec fn spec_start(&self) -> int {
        self.start.spec_raw_value()
    }


    /// Spec function to get the size in bytes.
    pub closed spec fn spec_size(&self) -> int {
        self.size as int
    }


    /// Spec function to get the start frame number.
    pub closed spec fn spec_start_frame(&self) -> int {
        self.start.spec_frame_number()
    }


    /// Spec function to get the number of frames in this region.
    pub closed spec fn spec_frame_count(&self) -> int {
        self.size as int / FRAME_SIZE as int
    }


    /// Invariant for TruncatedMemoryRegion (methodology Step 2).
    ///
    /// # Description
    ///
    /// The size must be page-aligned and positive, and the start address
    /// must satisfy its own invariant (page-aligned).
    pub closed spec fn inv(&self) -> bool {
        &&& self.start.inv()
        &&& self.size as int % FRAME_SIZE as int == 0
        &&& self.size > 0
    }
}

} // verus!
