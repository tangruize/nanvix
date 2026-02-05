// Copyright(c) The Maintainers of Nanvix.
// Licensed under the MIT License.

// Specifications.

verus! {

impl FrameNumber {
    /// Spec function to get the raw value.
    pub open spec fn spec_raw_value(&self) -> int {
        self.value as int
    }
}

impl FrameAddress {
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


    /// Spec function: invariant that size is page-aligned and positive.
    pub closed spec fn inv(&self) -> bool {
        &&& self.size as int % FRAME_SIZE as int == 0
        &&& self.size > 0
    }
}

} // verus!
