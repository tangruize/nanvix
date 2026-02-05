// Copyright(c) The Maintainers of Nanvix.
// Licensed under the MIT License.

//==================================================================================================
// Frame Address and Frame Number (Trusted Dependencies)
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

/// Frame size in bytes (4 KB).
pub const FRAME_SIZE: usize = 4096;

/// Maximum frame number (for a 32-bit address space with 4KB frames).
/// This represents the upper bound of frame numbers.
pub const MAX_FRAME_NUMBER: usize = 0xFFFF_FFFF / FRAME_SIZE;

//==================================================================================================
// FrameNumber
//==================================================================================================

/// A type that represents a frame number.
/// A frame number is in the range from `0` to `MAX_FRAME_NUMBER` (inclusive).
#[derive(Debug, Clone, Copy)]
pub struct FrameNumber {
    pub value: usize,
}

impl FrameNumber {
    /// Spec function to get the raw value.
    pub open spec fn spec_raw_value(&self) -> int {
        self.value as int
    }

    /// Constructs a FrameNumber from a raw value.
    /// Returns None if the value exceeds MAX_FRAME_NUMBER.
    pub fn from_raw_value(value: usize) -> (result: Option<FrameNumber>)
        ensures
            result is Some ==> {
                let frame = result->Some_0;
                &&& frame.spec_raw_value() == value as int
                &&& value <= MAX_FRAME_NUMBER
            },
            result is None ==> value > MAX_FRAME_NUMBER,
    {
        if value > MAX_FRAME_NUMBER {
            return None;
        }
        Some(FrameNumber { value })
    }

    /// Converts a FrameNumber into a raw value.
    pub fn into_raw_value(self) -> (result: usize)
        ensures result as int == self.spec_raw_value()
    {
        self.value
    }
}

//==================================================================================================
// FrameAddress
//==================================================================================================

/// A type that represents a frame address.
/// A frame address is page-aligned (multiple of FRAME_SIZE).
#[derive(Debug, Clone, Copy)]
pub struct FrameAddress {
    pub raw_addr: usize,
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

    /// Constructs a FrameAddress from a frame number.
    /// Precondition: frame_number.value <= MAX_FRAME_NUMBER ensures no overflow.
    pub fn from_frame_number(frame_number: FrameNumber) -> (result: Result<FrameAddress, Error>)
        requires
            frame_number.spec_raw_value() <= MAX_FRAME_NUMBER as int,
        ensures
            // Always succeeds when precondition is met.
            result is Ok,
            result is Ok ==> {
                let addr = result->Ok_0;
                &&& addr.spec_frame_number() == frame_number.spec_raw_value()
                &&& addr.spec_is_aligned()
                &&& addr.spec_raw_value() == frame_number.spec_raw_value() * FRAME_SIZE as int
            },
    {
        let raw_value: usize = frame_number.into_raw_value();
        // Check for overflow: raw_value * FRAME_SIZE must fit in usize.
        // Since raw_value <= MAX_FRAME_NUMBER = usize::MAX / FRAME_SIZE (on 32-bit),
        // this check always passes when precondition is satisfied.
        if raw_value > usize::MAX / FRAME_SIZE {
            return Err(Error::new(ErrorCode::InvalidArgument, "frame number overflow"));
        }
        let addr: usize = raw_value * FRAME_SIZE;
        Ok(FrameAddress { raw_addr: addr })
    }

    /// Converts a FrameAddress into a frame number.
    pub fn into_frame_number(self) -> (result: FrameNumber)
        requires self.spec_is_aligned(),
        ensures result.spec_raw_value() == self.spec_frame_number()
    {
        FrameNumber { value: self.raw_addr / FRAME_SIZE }
    }

    /// Gets the raw address value.
    pub fn into_raw_value(self) -> (result: usize)
        ensures result as int == self.spec_raw_value()
    {
        self.raw_addr
    }
}

//==================================================================================================
// PageAligned PhysicalAddress (Simplified Abstraction)
//==================================================================================================

/// A page-aligned physical address (simplified for verification).
#[derive(Debug, Clone, Copy)]
pub struct PageAlignedPhysAddr {
    pub raw_addr: usize,
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

    /// Constructs from a raw address value.
    /// Returns error if not page-aligned.
    pub fn from_raw_value(addr: usize) -> (result: Result<PageAlignedPhysAddr, Error>)
        ensures
            result is Ok ==> {
                let pa = result->Ok_0;
                &&& pa.spec_raw_value() == addr as int
                &&& addr % FRAME_SIZE == 0
            },
            result is Err ==> addr % FRAME_SIZE != 0,
    {
        if addr % FRAME_SIZE != 0 {
            return Err(Error::new(ErrorCode::InvalidArgument, "address not page-aligned"));
        }
        Ok(PageAlignedPhysAddr { raw_addr: addr })
    }

    /// Gets the frame number.
    pub fn into_frame_number(self) -> (result: FrameNumber)
        ensures result.spec_raw_value() == self.spec_frame_number()
    {
        FrameNumber { value: self.raw_addr / FRAME_SIZE }
    }
}

//==================================================================================================
// TruncatedMemoryRegion (Simplified for Verification)
//==================================================================================================

/// A simplified memory region where both start and size are page-aligned.
///
/// # Description
///
/// This is a simplified version of the Nanvix `TruncatedMemoryRegion<PhysicalAddress>` type.
/// The key invariants are:
/// - `start` is page-aligned (multiple of FRAME_SIZE).
/// - `size` is page-aligned (multiple of FRAME_SIZE).
/// - `size > 0` (non-empty region).
///
/// For verification purposes, we only track the start address and size,
/// omitting name, type, and permissions which are not relevant to allocation.
#[derive(Debug, Clone, Copy)]
pub struct TruncatedMemoryRegion {
    /// The page-aligned start address.
    start: PageAlignedPhysAddr,
    /// The page-aligned size in bytes.
    size: usize,
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

    /// Creates a new TruncatedMemoryRegion.
    ///
    /// # Parameters
    ///
    /// - `start`: Page-aligned start address.
    /// - `size`: Size in bytes (will be validated to be page-aligned and positive).
    ///
    /// # Returns
    ///
    /// Upon success, the region is returned. Upon failure, an error is returned.
    pub fn new(start: PageAlignedPhysAddr, size: usize) -> (result: Result<TruncatedMemoryRegion, Error>)
        ensures
            result is Ok ==> {
                let region = result->Ok_0;
                &&& region.inv()
                &&& region.spec_start() == start.spec_raw_value()
                &&& region.spec_size() == size as int
                &&& region.spec_start_frame() == start.spec_frame_number()
                &&& region.spec_frame_count() == size as int / FRAME_SIZE as int
                // Explicit guarantees for callers (since inv() is closed).
                &&& region.spec_frame_count() > 0
                &&& region.spec_size() > 0
            },
            result is Err ==> (size == 0 || size % FRAME_SIZE != 0),
    {
        if size == 0 {
            return Err(Error::new(ErrorCode::InvalidArgument, "size must be positive"));
        }
        if size % FRAME_SIZE != 0 {
            return Err(Error::new(ErrorCode::InvalidArgument, "size must be page-aligned"));
        }
        Ok(TruncatedMemoryRegion { start, size })
    }

    /// Lemma: If inv() holds, then frame_count > 0.
    ///
    /// # Description
    ///
    /// This lemma reveals the key property that inv() implies frame_count > 0,
    /// which is needed by callers since inv() is closed.
    pub proof fn lemma_inv_implies_frame_count_positive(&self)
        requires self.inv(),
        ensures
            self.spec_frame_count() > 0,
            self.spec_size() > 0,
    {
        // From inv(): size > 0 and size % FRAME_SIZE == 0.
        // Therefore size >= FRAME_SIZE, so size / FRAME_SIZE >= 1.
    }

    /// Returns the page-aligned start address.
    pub fn start(&self) -> (result: PageAlignedPhysAddr)
        ensures
            result.spec_raw_value() == self.spec_start(),
            result.spec_frame_number() == self.spec_start_frame(),
    {
        proof {
            // Reveal that spec_start_frame() == start.spec_frame_number().
            // This is trivially true by definition.
        }
        self.start
    }

    /// Returns the size of the region in bytes.
    pub fn size(&self) -> (result: usize)
        ensures result as int == self.spec_size()
    {
        self.size
    }

    /// Returns the number of frames in this region.
    pub fn frame_count(&self) -> (result: usize)
        requires self.inv(),
        ensures result as int == self.spec_frame_count()
    {
        self.size / FRAME_SIZE
    }
}

} // verus!
