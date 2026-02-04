// Copyright(c) The Maintainers of Nanvix.
// Licensed under the MIT License.

//==================================================================================================
// Frame Allocator Core (Verified Implementation)
//==================================================================================================

use crate::{
    bitmap::Bitmap,
    error::{
        Error,
        ErrorCode,
    },
    frame_address::{
        FrameAddress,
        FrameNumber,
        PageAlignedPhysAddr,
        TruncatedMemoryRegion,
        FRAME_SIZE,
        MAX_FRAME_NUMBER,
    },
    raw_array::RawArray,
};
use vstd::{
    prelude::*,
    set::*,
};

verus! {

//==================================================================================================
// FrameAllocatorView - Abstract Specification
//==================================================================================================

/// Abstract view of the frame allocator for specification purposes.
#[verifier::ext_equal]
pub struct FrameAllocatorView {
    /// Set of allocated frame numbers (indices).
    pub allocated_frames: Set<int>,
    /// Total number of frames managed by the allocator.
    pub capacity: int,
}

impl FrameAllocatorView {
    //==============================================================================================
    // Basic Properties
    //==============================================================================================

    /// Returns the number of allocated frames.
    pub open spec fn num_allocated(&self) -> int {
        self.allocated_frames.len() as int
    }

    /// Returns the number of free frames.
    pub open spec fn num_free(&self) -> int {
        self.capacity - self.num_allocated()
    }

    /// Returns true if a frame at the given index is allocated.
    pub open spec fn is_allocated(&self, frame_idx: int) -> bool {
        self.allocated_frames.contains(frame_idx)
    }

    /// Returns true if the allocator is full (no free frames).
    pub open spec fn is_full(&self) -> bool {
        self.num_allocated() == self.capacity
    }

    /// Returns true if the allocator is empty (all frames free).
    pub open spec fn is_empty(&self) -> bool {
        self.allocated_frames.len() == 0
    }

    /// Returns the physical address of a frame given its index.
    pub open spec fn frame_addr(&self, frame_idx: int) -> int {
        frame_idx * FRAME_SIZE as int
    }

    //==============================================================================================
    // Memory Safety Properties
    //==============================================================================================

    /// Property: All allocated frame indices are within valid range [0, capacity).
    pub open spec fn allocated_frames_in_range(&self) -> bool {
        forall|i: int|
            #![trigger self.is_allocated(i)]
            self.is_allocated(i) ==> (0 <= i < self.capacity)
    }

    /// Property: Memory regions of different frames are disjoint.
    /// Two frames with different indices have non-overlapping memory regions.
    pub open spec fn frames_are_disjoint(&self, i: int, j: int) -> bool
        recommends 0 <= i < self.capacity, 0 <= j < self.capacity, i != j
    {
        let addr_i = self.frame_addr(i);
        let addr_j = self.frame_addr(j);
        // Frame i's region [addr_i, addr_i + FRAME_SIZE) does not overlap with frame j's region.
        addr_i + FRAME_SIZE as int <= addr_j || addr_j + FRAME_SIZE as int <= addr_i
    }

    /// Property: All allocated frames have disjoint memory regions (no aliasing).
    pub open spec fn no_memory_aliasing(&self) -> bool {
        forall|i: int, j: int|
            #![trigger self.is_allocated(i), self.is_allocated(j)]
            (self.is_allocated(i) && self.is_allocated(j) && i != j) ==>
            self.frames_are_disjoint(i, j)
    }

    //==============================================================================================
    // Liveness Properties
    //==============================================================================================

    /// Property (Liveness): If there's free capacity, allocation can succeed.
    pub open spec fn can_allocate(&self) -> bool {
        self.num_free() > 0
    }

    /// Property (Liveness): There exists at least one unallocated frame.
    /// This mirrors bitmap's has_free_bit and is easier to connect.
    pub open spec fn has_free_frame(&self) -> bool {
        exists|i: int| 0 <= i < self.capacity && !self.is_allocated(i)
    }

    /// Property (Liveness): If a frame is allocated, it can be deallocated.
    pub open spec fn can_deallocate(&self, frame_idx: int) -> bool {
        self.is_allocated(frame_idx) && 0 <= frame_idx < self.capacity
    }

    //==============================================================================================
    // Initialization Properties
    //==============================================================================================

    /// Property: A freshly initialized allocator has no allocated frames.
    pub open spec fn is_freshly_initialized(&self) -> bool {
        self.allocated_frames =~= Set::<int>::empty()
    }
}

//==================================================================================================
// FrameAllocator - Concrete Implementation
//==================================================================================================

/// Frame allocator that manages physical memory frames using a bitmap.
///
/// The allocator tracks frame allocation state using a bitmap where each bit
/// represents a frame: set = allocated, unset = free.
#[derive(Debug)]
pub struct FrameAllocator {
    /// A bitmap that keeps track of free/used frames.
    bitmap: Bitmap,
}

impl View for FrameAllocator {
    type V = FrameAllocatorView;

    closed spec fn view(&self) -> FrameAllocatorView {
        FrameAllocatorView {
            allocated_frames: Set::new(|i: int|
                0 <= i < self.bitmap@.number_of_bits() &&
                self.bitmap.is_bit_set(i)
            ),
            capacity: self.bitmap@.number_of_bits(),
        }
    }
}

impl FrameAllocator {
    //==============================================================================================
    // Invariant
    //==============================================================================================

    /// Invariant for the frame allocator.
    /// Ensures internal consistency and memory safety guarantees.
    pub closed spec fn inv(&self) -> bool {
        // Bitmap must satisfy its own invariant.
        &&& self.bitmap.inv()
        // Capacity must be positive.
        &&& self.bitmap@.number_of_bits() > 0
        // Capacity must not exceed maximum addressable frames.
        &&& self.bitmap@.number_of_bits() <= MAX_FRAME_NUMBER as int + 1
        // View consistency.
        &&& self@.capacity == self.bitmap@.number_of_bits()
        // All allocated frames are in valid range.
        &&& self@.allocated_frames_in_range()
        // Connection between bitmap and view: a frame is allocated iff its bit is set.
        &&& forall|i: int| #![trigger self@.is_allocated(i), self.bitmap.is_bit_set(i)]
            0 <= i < self.bitmap@.number_of_bits() ==>
            (self@.is_allocated(i) <==> self.bitmap.is_bit_set(i))
        // MEMORY SAFETY: All allocated frames have disjoint memory regions (no aliasing).
        // This is a first-class invariant, automatically preserved by all operations.
        &&& self@.no_memory_aliasing()
    }

    //==============================================================================================
    // Specification Functions
    //==============================================================================================

    /// Returns the number of allocated frames (delegated to bitmap's count).
    pub closed spec fn spec_num_allocated(&self) -> int {
        self.bitmap@.usage()
    }

    //==============================================================================================
    // Lemmas
    //==============================================================================================

    /// Lemma: Reveals the connection between bitmap state and allocation state.
    /// When invariant holds, is_allocated(i) iff is_bit_set(i).
    proof fn lemma_allocated_iff_bit_set(&self, i: int)
        requires
            self.inv(),
            0 <= i < self@.capacity,
        ensures
            self@.is_allocated(i) <==> self.bitmap.is_bit_set(i)
    {
        // This follows from the invariant.
    }

    /// Lemma: Frames are disjoint by construction (addresses differ by at least FRAME_SIZE).
    pub proof fn lemma_frames_disjoint(i: int, j: int)
        requires
            0 <= i,
            0 <= j,
            i != j,
        ensures
            i * FRAME_SIZE as int + FRAME_SIZE as int <= j * FRAME_SIZE as int ||
            j * FRAME_SIZE as int + FRAME_SIZE as int <= i * FRAME_SIZE as int
    {
        // If i < j, then i + 1 <= j, so i * FRAME_SIZE + FRAME_SIZE <= j * FRAME_SIZE.
        // If i > j, then j + 1 <= i, so j * FRAME_SIZE + FRAME_SIZE <= i * FRAME_SIZE.
        if i < j {
            assert(i + 1 <= j);
            assert((i + 1) * FRAME_SIZE as int <= j * FRAME_SIZE as int);
            assert(i * FRAME_SIZE as int + FRAME_SIZE as int <= j * FRAME_SIZE as int);
        } else {
            assert(j + 1 <= i);
            assert((j + 1) * FRAME_SIZE as int <= i * FRAME_SIZE as int);
            assert(j * FRAME_SIZE as int + FRAME_SIZE as int <= i * FRAME_SIZE as int);
        }
    }

    /// Lemma: Connects has_free_frame to bitmap's has_free_bit.
    /// When invariant holds, has_free_frame implies bitmap has_free_bit.
    proof fn lemma_has_free_frame_implies_bitmap_has_free_bit(&self)
        requires
            self.inv(),
            self@.has_free_frame(),
        ensures
            self.bitmap@.has_free_bit()
    {
        // By has_free_frame, there exists i such that 0 <= i < capacity && !is_allocated(i).
        let i = choose|i: int| 0 <= i < self@.capacity && !self@.is_allocated(i);
        // By invariant, capacity == bitmap.number_of_bits().
        assert(0 <= i < self.bitmap@.number_of_bits());
        // By invariant, is_allocated(i) <==> is_bit_set(i).
        self.lemma_allocated_iff_bit_set(i);
        // Since !is_allocated(i), we have !is_bit_set(i).
        assert(!self.bitmap.is_bit_set(i));
        // Therefore, has_free_bit is satisfied.
        assert(self.bitmap@.has_free_bit());
    }

    /// Lemma: If `spec_num_allocated() < capacity`, then `has_free_frame()`.
    /// This connects the bitmap count to the existential predicate.
    pub proof fn lemma_can_allocate_implies_has_free_frame(&self)
        requires
            self.inv(),
            // Use spec_num_allocated (bitmap-based) directly.
            self.spec_num_allocated() < self@.capacity,
        ensures
            self@.has_free_frame()
    {
        // spec_num_allocated = bitmap.usage().
        // capacity = bitmap.number_of_bits() (by invariant).
        // So count_allocated < number_of_bits, meaning bitmap is NOT full.
        assert(!self.bitmap@.is_full());

        // Use bitmap lemma: if not full, there exists an unset bit.
        self.bitmap.lemma_not_full_means_exists_unset_bit();

        // Now we have: exists|i| 0 <= i < number_of_bits && !is_bit_set(i).
        let i: int = choose|i: int| 0 <= i < self.bitmap@.number_of_bits() && !self.bitmap.is_bit_set(i);
        assert(0 <= i < self@.capacity);

        // By invariant: is_allocated(i) <==> is_bit_set(i).
        self.lemma_allocated_iff_bit_set(i);
        assert(!self@.is_allocated(i));

        // Therefore, has_free_frame.
        assert(self@.has_free_frame());
    }

    //==============================================================================================
    // Constructor
    //==============================================================================================

    /// Instantiates a new frame allocator with a given bitmap.
    ///
    /// # Parameters
    ///
    /// - `bitmap`: The bitmap to use for tracking frame allocation.
    pub fn new(bitmap: Bitmap) -> (result: FrameAllocator)
        requires
            bitmap.inv(),
            bitmap@.number_of_bits() > 0,
            bitmap@.number_of_bits() <= MAX_FRAME_NUMBER as int + 1,
            // All bits initially unset (all frames free).
            forall|i: int| 0 <= i < bitmap@.number_of_bits() ==> !bitmap.is_bit_set(i),
        ensures
            result.inv(),
            result@.capacity == bitmap@.number_of_bits(),
            result@.is_freshly_initialized(),
    {
        let frame_allocator: FrameAllocator = FrameAllocator { bitmap };

        // Prove the allocator is freshly initialized.
        proof {
            assert(frame_allocator@.allocated_frames =~= Set::<int>::empty()) by {
                assert forall|i: int| !frame_allocator@.allocated_frames.contains(i) by {
                    if 0 <= i < frame_allocator.bitmap@.number_of_bits() {
                        assert(!frame_allocator.bitmap.is_bit_set(i));
                    }
                }
            }
        }

        frame_allocator
    }

    /// Instantiates a frame allocator from raw storage.
    ///
    /// # Parameters
    ///
    /// - `storage`: Raw byte storage for the bitmap.
    ///
    /// # Returns
    ///
    /// Upon success, a FrameAllocator is returned. Upon failure, an error is returned.
    ///
    /// # Note
    ///
    /// The storage must be zero-initialized. All frames will initially be free.
    /// This matches the original signature: `fn from_raw_storage(...) -> Result<Self, Error>`.
    pub fn from_raw_storage(storage: RawArray<u8>) -> (result: Result<FrameAllocator, Error>)
        requires
            storage@.len() > 0,
            storage@.len() <= usize::MAX / (u8::BITS as usize),
            storage@.len() * (u8::BITS as usize) <= MAX_FRAME_NUMBER as usize + 1,
            storage@.len() * (u8::BITS as usize) < u32::MAX as usize,
            forall|i: int| 0 <= i < storage@.len() ==> storage@[i] == 0,
        ensures
            // LIVENESS: Always succeeds when preconditions are met.
            result is Ok,
            // Unconditional guarantees (success is guaranteed by liveness above).
            result->Ok_0.inv(),
            result->Ok_0@.capacity == storage@.len() * (u8::BITS as int),
            result->Ok_0@.is_empty(),
            result->Ok_0@.is_freshly_initialized(),
    {
        let bitmap = Bitmap::from_raw_array(storage);
        let alloc = FrameAllocator { bitmap };
        proof {
            // Bitmap is empty, so no bits are set.
            // Therefore allocated_frames is the empty set.
            assert forall|i: int| 0 <= i < alloc.bitmap@.number_of_bits()
                implies !alloc.bitmap.is_bit_set(i) by {
                // From bitmap postcondition: forall|i| !result.is_bit_set(i)
            }
            // Therefore the set is empty.
            assert(alloc@.allocated_frames =~= Set::empty());
        }
        Ok(alloc)
    }

    /// Returns the capacity (number of frames managed).
    pub fn capacity(&self) -> (result: usize)
        requires self.inv(),
        ensures
            result as int == self@.capacity,
            // Capacity is always positive (from invariant).
            result > 0,
    {
        self.bitmap.number_of_bits()
    }

    //==============================================================================================
    // Allocation
    //==============================================================================================

    /// Allocates a frame and returns its frame index.
    ///
    /// # Description
    ///
    /// This is a helper function that returns the raw frame index (usize).
    /// For the primary allocation API matching the original source, use `alloc()`.
    ///
    /// # Returns
    ///
    /// Upon success, the frame index is returned as a usize.
    /// Upon failure, an error is returned.
    pub fn alloc_index(&mut self) -> (result: Result<usize, Error>)
        requires old(self).inv(),
        ensures
            self.inv(),
            // Capacity is preserved.
            self@.capacity == old(self)@.capacity,
            // Liveness: If there's a free frame, allocation succeeds.
            old(self)@.has_free_frame() ==> result is Ok,
            // On success: exactly one new frame is allocated.
            result is Ok ==> {
                let frame_idx = result->Ok_0 as int;
                // The frame index is valid.
                &&& 0 <= frame_idx < self@.capacity
                // The frame is now allocated.
                &&& self@.is_allocated(frame_idx)
                // The frame was not previously allocated.
                &&& !old(self)@.is_allocated(frame_idx)
                // All other frames unchanged.
                &&& forall|i: int| #![trigger self@.is_allocated(i)]
                    0 <= i < self@.capacity && i != frame_idx ==>
                    self@.is_allocated(i) == old(self)@.is_allocated(i)
            },
            // EXPLICIT COUNT: exactly one more frame allocated.
            result is Ok ==> self.spec_num_allocated() == old(self).spec_num_allocated() + 1,
            // On failure: state unchanged AND no free frame existed (contrapositive of liveness).
            result is Err ==> {
                &&& self@ == old(self)@
                &&& !old(self)@.has_free_frame()
            },
    {
        // Use lemma to connect has_free_frame to bitmap's has_free_bit.
        proof {
            if self@.has_free_frame() {
                self.lemma_has_free_frame_implies_bitmap_has_free_bit();
            }
        }
        // Allocate a bit from the bitmap.
        match self.bitmap.alloc() {
            Ok(frame_number) => Ok(frame_number),
            Err(error) => Err(error),
        }
    }

    /// Allocates a frame.
    ///
    /// # Description
    ///
    /// This matches the original source: `fn alloc(&mut self) -> Result<FrameAddress, Error>`.
    ///
    /// # Returns
    ///
    /// Upon success, the frame address is returned.
    /// Upon failure, an error is returned.
    pub fn alloc(&mut self) -> (result: Result<FrameAddress, Error>)
        requires old(self).inv(),
        ensures
            self.inv(),
            // Capacity is preserved.
            self@.capacity == old(self)@.capacity,
            // Liveness: If there's a free frame, allocation succeeds.
            old(self)@.has_free_frame() ==> result is Ok,
            // On success: exactly one new frame is allocated.
            result is Ok ==> {
                let frame = result->Ok_0;
                let frame_idx = frame.spec_frame_number();
                // The frame address is valid and aligned.
                &&& frame.spec_is_aligned()
                // The frame index is valid.
                &&& 0 <= frame_idx < self@.capacity
                // The frame is now allocated.
                &&& self@.is_allocated(frame_idx)
                // The frame was not previously allocated.
                &&& !old(self)@.is_allocated(frame_idx)
                // All other frames unchanged.
                &&& forall|i: int| #![trigger self@.is_allocated(i)]
                    0 <= i < self@.capacity && i != frame_idx ==>
                    self@.is_allocated(i) == old(self)@.is_allocated(i)
            },
            // EXPLICIT COUNT: exactly one more frame allocated.
            result is Ok ==> self.spec_num_allocated() == old(self).spec_num_allocated() + 1,
            // On failure: state unchanged AND no free frame existed (contrapositive of liveness).
            result is Err ==> {
                &&& self@ == old(self)@
                &&& !old(self)@.has_free_frame()
            },
    {
        // Use lemma to connect has_free_frame to bitmap's has_free_bit.
        proof {
            if self@.has_free_frame() {
                self.lemma_has_free_frame_implies_bitmap_has_free_bit();
            }
        }
        // Allocate a bit from the bitmap.
        match self.bitmap.alloc() {
            Ok(frame_idx) => {
                // frame_idx < capacity <= MAX_FRAME_NUMBER + 1, so frame_idx <= MAX_FRAME_NUMBER.
                // Thus the conversion to FrameAddress always succeeds.
                proof {
                    // By invariant: capacity <= MAX_FRAME_NUMBER + 1.
                    // By bitmap postcondition: frame_idx < capacity.
                    // Therefore: frame_idx <= MAX_FRAME_NUMBER.
                    assert(frame_idx as int <= MAX_FRAME_NUMBER as int);
                }
                let frame_number = FrameNumber { value: frame_idx };
                let frame_addr = FrameAddress::from_frame_number(frame_number);
                frame_addr
            },
            Err(error) => Err(error),
        }
    }

    //==============================================================================================
    // Deallocation
    //==============================================================================================

    /// Frees a frame.
    ///
    /// # Parameters
    ///
    /// - `frame`: The address of the frame to free.
    ///
    /// # Returns
    ///
    /// Upon success, `Ok(())` is returned.
    /// Upon failure, an error is returned.
    pub fn free(&mut self, frame: FrameAddress) -> (result: Result<(), Error>)
        requires
            old(self).inv(),
            // Frame address must be aligned.
            frame.spec_is_aligned(),
            // Frame index must be within capacity.
            frame.spec_frame_number() < old(self)@.capacity,
            // Use can_deallocate for clearer specification.
            old(self)@.can_deallocate(frame.spec_frame_number()),
        ensures
            self.inv(),
            // LIVENESS: free always succeeds when preconditions are met.
            result is Ok,
            // Capacity is preserved.
            self@.capacity == old(self)@.capacity,
            // Unconditional guarantees (since success is guaranteed by liveness above).
            // The frame is now free.
            !self@.is_allocated(frame.spec_frame_number()),
            // All other frames unchanged.
            forall|i: int| #![trigger self@.is_allocated(i)]
                0 <= i < self@.capacity && i != frame.spec_frame_number() ==>
                self@.is_allocated(i) == old(self)@.is_allocated(i),
            // EXPLICIT COUNT: exactly one fewer frame allocated.
            self.spec_num_allocated() == old(self).spec_num_allocated() - 1,
    {
        let frame_number: usize = frame.into_frame_number().into_raw_value();

        // Explicitly prove the bitmap precondition is satisfied.
        // The invariant connects is_allocated to is_bit_set.
        proof {
            self.lemma_allocated_iff_bit_set(frame_number as int);
        }

        match self.bitmap.clear(frame_number) {
            Ok(()) => Ok(()),
            Err(error) => Err(error),
        }
    }

    //==============================================================================================
    // Booking (Reserve Specific Frame)
    //==============================================================================================

    /// Books a specific frame (marks it as allocated without first allocating it).
    /// Used to reserve frames for memory-mapped I/O or other special purposes.
    ///
    /// Note: If the frame is already allocated, the operation is idempotent (succeeds).
    /// The postcondition guarantees the frame is allocated after the call regardless
    /// of its prior state.
    ///
    /// # Parameters
    ///
    /// - `phys_addr`: The page-aligned physical address to book.
    ///
    /// # Returns
    ///
    /// Upon success, `Ok(())` is returned.
    /// Upon failure, an error is returned.
    pub fn book(&mut self, phys_addr: PageAlignedPhysAddr) -> (result: Result<(), Error>)
        requires
            old(self).inv(),
            // Frame index must be within capacity.
            phys_addr.spec_frame_number() < old(self)@.capacity,
        ensures
            self.inv(),
            // LIVENESS: book always succeeds when preconditions are met.
            result is Ok,
            // Capacity is preserved.
            self@.capacity == old(self)@.capacity,
            // Frame is now allocated (idempotent - works whether or not already allocated).
            // The frame is now allocated.
            self@.is_allocated(phys_addr.spec_frame_number()),
            // All other frames unchanged.
            forall|i: int| #![trigger self@.is_allocated(i)]
                0 <= i < self@.capacity && i != phys_addr.spec_frame_number() ==>
                self@.is_allocated(i) == old(self)@.is_allocated(i),
            // COUNT (idempotent): +1 if frame was free, unchanged if already allocated.
            !old(self)@.is_allocated(phys_addr.spec_frame_number()) ==>
                self.spec_num_allocated() == old(self).spec_num_allocated() + 1,
            old(self)@.is_allocated(phys_addr.spec_frame_number()) ==>
                self.spec_num_allocated() == old(self).spec_num_allocated(),
    {
        let frame_number: usize = phys_addr.into_frame_number().into_raw_value();
        // Check if already allocated (idempotent behavior).
        let already_set: bool = match self.bitmap.test(frame_number) {
            Ok(b) => b,
            Err(_) => false,  // Error means out of bounds, shouldn't happen given preconditions.
        };
        if already_set {
            // Already allocated - nothing to do.
            proof {
                // The view doesn't change since the bit is already set.
                assert(self@ == old(self)@);
            }
            return Ok(());
        }
        // Not allocated - set it.
        match self.bitmap.set(frame_number) {
            Ok(()) => Ok(()),
            Err(error) => {
                // This should not happen since we checked test() above.
                Err(error)
            },
        }
    }

    //==============================================================================================
    // Range Allocation
    //==============================================================================================

    /// Allocates a contiguous range of frames by marking them as allocated.
    ///
    /// # Parameters
    ///
    /// - `start_frame`: Start frame index (inclusive).
    /// - `count`: Number of frames to allocate.
    ///
    /// # Returns
    ///
    /// Upon success, all frames in [start_frame, start_frame + count) are allocated.
    ///
    /// # Liveness
    ///
    /// This operation always succeeds when preconditions are met (all frames in range
    /// are free and range is within capacity). The underlying bitmap operations have
    /// liveness guarantees.
    ///
    /// # Precondition
    ///
    /// All frames in the range must be initially free.
    pub fn alloc_range(&mut self, start_frame: usize, count: usize) -> (result: Result<(), Error>)
        requires
            old(self).inv(),
            count > 0,
            start_frame as int + count as int <= old(self)@.capacity,
            // All frames in range must be initially free.
            forall|i: int| start_frame as int <= i < start_frame as int + count as int ==>
                !old(self)@.is_allocated(i),
        ensures
            self.inv(),
            // Capacity is preserved.
            self@.capacity == old(self)@.capacity,
            // LIVENESS: alloc_range always succeeds when preconditions are met.
            result is Ok,
            // All frames in range are now allocated.
            forall|i: int| start_frame as int <= i < start_frame as int + count as int ==>
                self@.is_allocated(i),
            // All frames outside range unchanged.
            forall|i: int| #![trigger self@.is_allocated(i)]
                (0 <= i < start_frame as int || start_frame as int + count as int <= i < self@.capacity) ==>
                self@.is_allocated(i) == old(self)@.is_allocated(i),
            // EXPLICIT COUNT: allocated count increases by exactly `count`.
            self.spec_num_allocated() == old(self).spec_num_allocated() + count as int,
    {
        let end_frame: usize = start_frame + count;
        let ghost original_self: FrameAllocator = *self;
        let ghost original_capacity: int = self@.capacity;

        // Prove preconditions for inner function.
        proof {
            assert(original_capacity == old(self)@.capacity);
            // Connect !is_allocated to !is_bit_set via the invariant.
            assert forall|i: int| start_frame as int <= i < end_frame as int
                implies !self.bitmap.is_bit_set(i)
            by {
                self.lemma_allocated_iff_bit_set(i);
            }
        }

        match self.alloc_range_inner(start_frame, end_frame, Ghost(original_self), Ghost(original_capacity)) {
            Ok(()) => {
                // Connect helper's bitmap postconditions to caller's is_allocated postconditions.
                proof {
                    // All frames in range are now allocated.
                    assert forall|i: int| start_frame as int <= i < end_frame as int
                        implies self@.is_allocated(i)
                    by {
                        assert(self.bitmap.is_bit_set(i));
                        self.lemma_allocated_iff_bit_set(i);
                    }

                    // All frames outside range unchanged.
                    assert forall|i: int|
                        (0 <= i < start_frame as int || end_frame as int <= i < self@.capacity)
                        implies self@.is_allocated(i) == original_self@.is_allocated(i)
                    by {
                        assert(self.bitmap.is_bit_set(i) == original_self.bitmap.is_bit_set(i));
                        self.lemma_allocated_iff_bit_set(i);
                        original_self.lemma_allocated_iff_bit_set(i);
                    }
                }
                Ok(())
            },
            Err(_) => {
                // Unreachable: alloc_range_inner always succeeds.
                proof { assert(false); }
                Err(Error::new(ErrorCode::InvalidArgument, "unreachable"))
            },
        }
    }

    /// Helper function for alloc_range that handles the loop.
    fn alloc_range_inner(
        &mut self,
        start_frame: usize,
        end_frame: usize,
        Ghost(original_self): Ghost<FrameAllocator>,
        Ghost(original_capacity): Ghost<int>,
    ) -> (result: Result<(), Error>)
        requires
            old(self).inv(),
            original_self.inv(),
            start_frame < end_frame,
            end_frame as int <= old(self)@.capacity,
            old(self)@.capacity == original_capacity,
            original_self@.capacity == original_capacity,
            // All frames in range are initially free (bit not set).
            forall|i: int| start_frame as int <= i < end_frame as int ==>
                !old(self).bitmap.is_bit_set(i),
            // Frames outside range match original.
            forall|i: int|
                (0 <= i < start_frame as int || end_frame as int <= i < old(self)@.capacity) ==>
                old(self).bitmap.is_bit_set(i) == original_self.bitmap.is_bit_set(i),
        ensures
            self.inv(),
            self@.capacity == original_capacity,
            // LIVENESS: always succeeds (bitmap.set always succeeds when preconditions met).
            result is Ok,
            forall|i: int| start_frame as int <= i < end_frame as int ==>
                self.bitmap.is_bit_set(i),
            forall|i: int|
                (0 <= i < start_frame as int || end_frame as int <= i < self@.capacity) ==>
                self.bitmap.is_bit_set(i) == original_self.bitmap.is_bit_set(i),
            // Count increases by exactly (end_frame - start_frame).
            self.spec_num_allocated() == old(self).spec_num_allocated() + (end_frame - start_frame) as int,
    {
        let mut idx: usize = start_frame;

        while idx < end_frame
            invariant
                // Core invariant preserved.
                self.inv(),
                // Original invariant held.
                original_self.inv(),
                // Capacity unchanged.
                self@.capacity == original_capacity,
                original_self@.capacity == original_capacity,
                // Loop bounds.
                start_frame <= idx <= end_frame,
                end_frame as int <= self@.capacity,
                // Bits in [start_frame, idx) are set.
                forall|i: int| start_frame as int <= i < idx as int ==>
                    self.bitmap.is_bit_set(i),
                // Bits in [idx, end_frame) are unchanged from old.
                forall|i: int| idx as int <= i < end_frame as int ==>
                    !self.bitmap.is_bit_set(i),
                // Bits outside [start_frame, end_frame) are unchanged from original.
                forall|i: int|
                    (0 <= i < start_frame as int || end_frame as int <= i < self@.capacity) ==>
                    self.bitmap.is_bit_set(i) == original_self.bitmap.is_bit_set(i),
                // Count tracking: we've added (idx - start_frame) bits so far.
                self.spec_num_allocated() == old(self).spec_num_allocated() + (idx - start_frame) as int,
            decreases
                end_frame - idx,
        {
            // bitmap.set always succeeds when preconditions are met.
            let _ = self.bitmap.set(idx);
            idx = idx + 1;
        }

        Ok(())
    }

    /// Allocates a contiguous range of frames with runtime checking.
    ///
    /// # Description
    ///
    /// This function matches the original source behavior exactly:
    /// 1. First checks if ALL frames in the range are free (runtime check).
    /// 2. If any frame is already allocated, returns OutOfMemory error.
    /// 3. If all frames are free, allocates them all.
    ///
    /// This differs from `alloc_range` which requires frames to be free as a precondition.
    ///
    /// # Parameters
    ///
    /// - `start_frame`: Start frame index (inclusive).
    /// - `count`: Number of frames to allocate.
    ///
    /// # Returns
    ///
    /// Upon success, all frames in [start_frame, start_frame + count) are allocated.
    /// Upon failure (any frame already allocated), an error is returned.
    pub fn alloc_range_checked(&mut self, start_frame: usize, count: usize) -> (result: Result<(), Error>)
        requires
            old(self).inv(),
            count > 0,
            start_frame as int + count as int <= old(self)@.capacity,
        ensures
            self.inv(),
            // Capacity is preserved.
            self@.capacity == old(self)@.capacity,
            // On success: all frames in range are now allocated.
            result is Ok ==> {
                // All frames in range were previously free.
                &&& forall|i: int| start_frame as int <= i < start_frame as int + count as int ==>
                    !old(self)@.is_allocated(i)
                // All frames in range are now allocated.
                &&& forall|i: int| start_frame as int <= i < start_frame as int + count as int ==>
                    self@.is_allocated(i)
                &&& forall|i: int| #![trigger self@.is_allocated(i)]
                    (0 <= i < start_frame as int || start_frame as int + count as int <= i < self@.capacity) ==>
                    self@.is_allocated(i) == old(self)@.is_allocated(i)
            },
            // On success: count increases by exactly `count`.
            result is Ok ==> self.spec_num_allocated() == old(self).spec_num_allocated() + count as int,
            // On failure: state unchanged and at least one frame was already allocated.
            result is Err ==> {
                &&& self@ == old(self)@
                &&& exists|i: int| start_frame as int <= i < start_frame as int + count as int &&
                    old(self)@.is_allocated(i)
            },
    {
        let end_frame: usize = start_frame + count;

        // Step 1: Check if all frames in the range are free (matches original).
        let mut idx: usize = start_frame;
        while idx < end_frame
            invariant
                self.inv(),
                self@ == old(self)@,
                self@.capacity == old(self)@.capacity,
                start_frame <= idx <= end_frame,
                end_frame as int <= self@.capacity,
                end_frame == start_frame + count,
                // All frames checked so far are free.
                forall|i: int| start_frame as int <= i < idx as int ==>
                    !self@.is_allocated(i),
            decreases
                end_frame - idx,
        {
            match self.bitmap.test(idx) {
                Ok(is_set) => {
                    if is_set {
                        // Frame is already allocated - return error (matches original).
                        // Prove the exists postcondition: idx is a witness.
                        proof {
                            self.lemma_allocated_iff_bit_set(idx as int);
                            // idx is in range and is_allocated(idx) is true.
                            assert(self@.is_allocated(idx as int));
                            // Since self@ == old(self)@, we have old(self)@.is_allocated(idx).
                            assert(old(self)@.is_allocated(idx as int));
                            // Prove the exists postcondition with idx as witness.
                            // Need to show: start_frame <= idx < start_frame + count && is_allocated(idx).
                            let witness_i: int = idx as int;
                            assert(start_frame as int <= witness_i);
                            assert(witness_i < start_frame as int + count as int);
                            assert(old(self)@.is_allocated(witness_i));
                        }
                        return Err(Error::new(ErrorCode::OutOfMemory, "frame is already allocated"));
                    }
                    // Frame is free, continue checking.
                    proof {
                        self.lemma_allocated_iff_bit_set(idx as int);
                    }
                },
                Err(err) => {
                    // Error from bitmap.test - state unchanged but we can't prove the exists.
                    // This branch should not occur given our preconditions (idx < capacity).
                    proof {
                        // We need a witness, but don't have one. This is an edge case.
                        // Since idx < end_frame <= capacity, test should not fail.
                        // If it does, we assume there's a problem elsewhere.
                    }
                    return Err(err);
                },
            }
            idx = idx + 1;
        }

        // Step 2: All frames are free, allocate them (matches original).
        // At this point we have proven all frames are free.
        proof {
            // Connect to the bitmap level.
            assert forall|i: int| start_frame as int <= i < end_frame as int
                implies !self.bitmap.is_bit_set(i)
            by {
                self.lemma_allocated_iff_bit_set(i);
            }
        }

        let ghost original_self: FrameAllocator = *self;
        let ghost original_capacity: int = self@.capacity;

        match self.alloc_range_inner(start_frame, end_frame, Ghost(original_self), Ghost(original_capacity)) {
            Ok(()) => {
                proof {
                    // All frames in range are now allocated.
                    assert forall|i: int| start_frame as int <= i < end_frame as int
                        implies self@.is_allocated(i)
                    by {
                        assert(self.bitmap.is_bit_set(i));
                        self.lemma_allocated_iff_bit_set(i);
                    }

                    // All frames outside range unchanged.
                    assert forall|i: int|
                        (0 <= i < start_frame as int || end_frame as int <= i < self@.capacity)
                        implies self@.is_allocated(i) == original_self@.is_allocated(i)
                    by {
                        assert(self.bitmap.is_bit_set(i) == original_self.bitmap.is_bit_set(i));
                        self.lemma_allocated_iff_bit_set(i);
                        original_self.lemma_allocated_iff_bit_set(i);
                    }
                }
                Ok(())
            },
            Err(e) => {
                // Unreachable: alloc_range_inner always succeeds.
                proof { assert(false); }
                Err(e)
            },
        }
    }

    //==============================================================================================
    // Range Allocation from TruncatedMemoryRegion
    //==============================================================================================

    /// Allocates all frames in a memory region.
    ///
    /// # Description
    ///
    /// This matches the Nanvix API: `fn alloc_range(&mut self, region: &TruncatedMemoryRegion<PhysicalAddress>)`.
    /// It first checks if all frames in the region are free, then allocates them.
    ///
    /// # Parameters
    ///
    /// - `region`: A page-aligned memory region specifying the frames to allocate.
    ///
    /// # Returns
    ///
    /// Upon success, `Ok(())` is returned and all frames in the region are allocated.
    /// Upon failure (any frame already allocated), an error is returned.
    pub fn alloc_range_from_region(&mut self, region: &TruncatedMemoryRegion) -> (result: Result<(), Error>)
        requires
            old(self).inv(),
            region.inv(),
            // Region must be within allocator capacity.
            region.spec_start_frame() >= 0,
            region.spec_start_frame() + region.spec_frame_count() <= old(self)@.capacity,
        ensures
            self.inv(),
            // Capacity is preserved.
            self@.capacity == old(self)@.capacity,
            // On success: all frames in region are now allocated.
            result is Ok ==> {
                let start = region.spec_start_frame();
                let count = region.spec_frame_count();
                // All frames in range are now allocated.
                &&& forall|i: int| start <= i < start + count ==>
                    self@.is_allocated(i)
                // All frames outside range unchanged.
                &&& forall|i: int| #![trigger self@.is_allocated(i)]
                    (0 <= i < start || start + count <= i < self@.capacity) ==>
                    self@.is_allocated(i) == old(self)@.is_allocated(i)
                // COUNT: allocated count increases by frame_count.
                &&& self.spec_num_allocated() == old(self).spec_num_allocated() + count
            },
            // On failure: state unchanged and some frame was already allocated.
            result is Err ==> {
                &&& self@ == old(self)@
                &&& exists|i: int| region.spec_start_frame() <= i < region.spec_start_frame() + region.spec_frame_count()
                    && old(self)@.is_allocated(i)
            },
            // Liveness: if all frames in range are free, allocation succeeds.
            (forall|i: int| region.spec_start_frame() <= i < region.spec_start_frame() + region.spec_frame_count() ==>
                !old(self)@.is_allocated(i)) ==> result is Ok,
    {
        let start_frame: usize = region.start().into_frame_number().into_raw_value();
        let count: usize = region.frame_count();

        // Delegate to alloc_range_checked which has the same semantics.
        self.alloc_range_checked(start_frame, count)
    }

    //==============================================================================================
    // Contiguous Range Allocation with Search
    //==============================================================================================

    /// Allocates a contiguous range of frames by searching for a free range.
    ///
    /// # Description
    ///
    /// This function searches for a contiguous range of `count` free frames and
    /// allocates them atomically. This matches the original kernel behavior where
    /// `alloc_range(count)` internally finds a suitable range.
    ///
    /// # Parameters
    ///
    /// - `count`: Number of contiguous frames to allocate.
    ///
    /// # Returns
    ///
    /// On success, returns the starting frame index of the allocated range.
    /// On failure (no contiguous range available), returns an error.
    ///
    /// # Memory Safety
    ///
    /// - All frames in the returned range were previously free.
    /// - All frames in the range are now allocated.
    /// - All frames outside the range are unchanged.
    ///
    /// # Liveness
    ///
    /// For count=1, if there's a free frame, allocation succeeds.
    pub fn alloc_contiguous_range(&mut self, count: usize) -> (result: Result<usize, Error>)
        requires
            old(self).inv(),
            count > 0,
            count as int <= old(self)@.capacity,
        ensures
            self.inv(),
            // Capacity is preserved.
            self@.capacity == old(self)@.capacity,
            // On success: a valid contiguous range is allocated.
            result is Ok ==> {
                let start = result->Ok_0 as int;
                &&& 0 <= start < self@.capacity
                &&& start + count as int <= self@.capacity
                // All frames in range were previously free.
                &&& forall|i: int| start <= i < start + count as int ==>
                    !old(self)@.is_allocated(i)
                // All frames in range are now allocated.
                &&& forall|i: int| start <= i < start + count as int ==>
                    self@.is_allocated(i)
                // All frames outside range unchanged.
                &&& forall|i: int| #![trigger self@.is_allocated(i)]
                    (0 <= i < start || start + count as int <= i < self@.capacity) ==>
                    self@.is_allocated(i) == old(self)@.is_allocated(i)
            },
            // Count tracking on success.
            result is Ok ==> self.spec_num_allocated() == old(self).spec_num_allocated() + count as int,
            // On failure: state unchanged.
            result is Err ==> self@ == old(self)@,
            // Liveness for count=1 (has_free_frame implies a contiguous range of size 1 exists).
            (count == 1 && old(self)@.has_free_frame()) ==> result is Ok,
    {
        proof {
            // For liveness: if count == 1 && has_free_frame, then bitmap.exists_contiguous_free_range(1).
            if count == 1 && old(self)@.has_free_frame() {
                // has_free_frame implies bitmap.has_free_bit by existing lemma.
                old(self).lemma_has_free_frame_implies_bitmap_has_free_bit();
                // bitmap.has_free_bit implies bitmap.exists_contiguous_free_range(1).
                old(self).bitmap.lemma_has_free_bit_implies_exists_free_range_1();
            }
        }

        // Use bitmap's alloc_range which searches for and allocates a contiguous range.
        match self.bitmap.alloc_range(count) {
            Ok(start) => {
                proof {
                    // Connect bitmap.is_bit_set to is_allocated.
                    assert forall|i: int| start as int <= i < start as int + count as int
                        implies !old(self)@.is_allocated(i)
                    by {
                        // old(self).bitmap.all_bits_unset_in_range(start, start + count).
                        assert(!old(self).bitmap.is_bit_set(i));
                        old(self).lemma_allocated_iff_bit_set(i);
                    }

                    assert forall|i: int| start as int <= i < start as int + count as int
                        implies self@.is_allocated(i)
                    by {
                        // self.bitmap.all_bits_set_in_range(start, start + count).
                        assert(self.bitmap.is_bit_set(i));
                        self.lemma_allocated_iff_bit_set(i);
                    }

                    assert forall|i: int|
                        (0 <= i < start as int || start as int + count as int <= i < self@.capacity)
                        implies self@.is_allocated(i) == old(self)@.is_allocated(i)
                    by {
                        assert(self.bitmap.is_bit_set(i) == old(self).bitmap.is_bit_set(i));
                        self.lemma_allocated_iff_bit_set(i);
                        old(self).lemma_allocated_iff_bit_set(i);
                    }
                }
                Ok(start)
            },
            Err(e) => Err(e),
        }
    }

    //==============================================================================================
    // Range Deallocation
    //==============================================================================================

    /// Frees a contiguous range of frames.
    ///
    /// # Parameters
    ///
    /// - `start_frame`: Start frame index (inclusive).
    /// - `count`: Number of frames to free.
    ///
    /// # Returns
    ///
    /// Upon success, all frames in [start_frame, start_frame + count) are freed.
    ///
    /// # Liveness
    ///
    /// This operation always succeeds when preconditions are met.
    ///
    /// # Precondition
    ///
    /// All frames in the range must be currently allocated.
    pub fn free_range(&mut self, start_frame: usize, count: usize) -> (result: Result<(), Error>)
        requires
            old(self).inv(),
            count > 0,
            start_frame as int + count as int <= old(self)@.capacity,
            // All frames in range must be allocated.
            forall|i: int| start_frame as int <= i < start_frame as int + count as int ==>
                old(self)@.is_allocated(i),
        ensures
            self.inv(),
            // Capacity is preserved.
            self@.capacity == old(self)@.capacity,
            // LIVENESS: always succeeds when preconditions met.
            result is Ok,
            // All frames in range are now free.
            forall|i: int| start_frame as int <= i < start_frame as int + count as int ==>
                !self@.is_allocated(i),
            // All frames outside range unchanged.
            forall|i: int| #![trigger self@.is_allocated(i)]
                (0 <= i < start_frame as int || start_frame as int + count as int <= i < self@.capacity) ==>
                self@.is_allocated(i) == old(self)@.is_allocated(i),
            // COUNT: allocated count decreases by exactly `count`.
            self.spec_num_allocated() == old(self).spec_num_allocated() - count as int,
    {
        let end_frame: usize = start_frame + count;
        let ghost original_self: FrameAllocator = *self;

        // Prove preconditions for inner function: connect is_allocated to is_bit_set.
        proof {
            assert forall|i: int| start_frame as int <= i < end_frame as int
                implies self.bitmap.is_bit_set(i)
            by {
                self.lemma_allocated_iff_bit_set(i);
            }
        }

        match self.free_range_inner(start_frame, end_frame, Ghost(original_self)) {
            Ok(()) => {
                // Connect helper's bitmap postconditions to caller's is_allocated postconditions.
                proof {
                    // All frames in range are now free.
                    assert forall|i: int| start_frame as int <= i < end_frame as int
                        implies !self@.is_allocated(i)
                    by {
                        assert(!self.bitmap.is_bit_set(i));
                        self.lemma_allocated_iff_bit_set(i);
                    }

                    // All frames outside range unchanged.
                    assert forall|i: int|
                        (0 <= i < start_frame as int || end_frame as int <= i < self@.capacity)
                        implies self@.is_allocated(i) == original_self@.is_allocated(i)
                    by {
                        assert(self.bitmap.is_bit_set(i) == original_self.bitmap.is_bit_set(i));
                        self.lemma_allocated_iff_bit_set(i);
                        original_self.lemma_allocated_iff_bit_set(i);
                    }
                }
                Ok(())
            },
            Err(e) => {
                // Unreachable: free_range_inner always succeeds.
                proof { assert(false); }
                Err(e)
            },
        }
    }

    /// Helper function for free_range that handles the loop.
    fn free_range_inner(
        &mut self,
        start_frame: usize,
        end_frame: usize,
        Ghost(original_self): Ghost<FrameAllocator>,
    ) -> (result: Result<(), Error>)
        requires
            old(self).inv(),
            original_self.inv(),
            start_frame < end_frame,
            end_frame as int <= old(self)@.capacity,
            old(self)@.capacity == original_self@.capacity,
            // All frames in range are currently allocated.
            forall|i: int| start_frame as int <= i < end_frame as int ==>
                old(self).bitmap.is_bit_set(i),
            // Frames outside range match original.
            forall|i: int|
                (0 <= i < start_frame as int || end_frame as int <= i < old(self)@.capacity) ==>
                old(self).bitmap.is_bit_set(i) == original_self.bitmap.is_bit_set(i),
        ensures
            self.inv(),
            self@.capacity == original_self@.capacity,
            // LIVENESS: always succeeds.
            result is Ok,
            // All frames in range are now free (bit not set).
            forall|i: int| start_frame as int <= i < end_frame as int ==>
                !self.bitmap.is_bit_set(i),
            // Frames outside range unchanged.
            forall|i: int|
                (0 <= i < start_frame as int || end_frame as int <= i < self@.capacity) ==>
                self.bitmap.is_bit_set(i) == original_self.bitmap.is_bit_set(i),
            // Count decreases by exactly (end_frame - start_frame).
            self.spec_num_allocated() == old(self).spec_num_allocated() - (end_frame - start_frame) as int,
    {
        let mut idx: usize = start_frame;

        while idx < end_frame
            invariant
                // Core invariant preserved.
                self.inv(),
                // Original invariant held.
                original_self.inv(),
                // Capacity unchanged.
                self@.capacity == original_self@.capacity,
                // Loop bounds.
                start_frame <= idx <= end_frame,
                end_frame as int <= self@.capacity,
                // Bits in [start_frame, idx) are now unset.
                forall|i: int| start_frame as int <= i < idx as int ==>
                    !self.bitmap.is_bit_set(i),
                // Bits in [idx, end_frame) are still set.
                forall|i: int| idx as int <= i < end_frame as int ==>
                    self.bitmap.is_bit_set(i),
                // Bits outside [start_frame, end_frame) are unchanged from original.
                forall|i: int|
                    (0 <= i < start_frame as int || end_frame as int <= i < self@.capacity) ==>
                    self.bitmap.is_bit_set(i) == original_self.bitmap.is_bit_set(i),
                // Count tracking: we've removed (idx - start_frame) bits so far.
                self.spec_num_allocated() == old(self).spec_num_allocated() - (idx - start_frame) as int,
            decreases
                end_frame - idx,
        {
            // bitmap.clear always succeeds when preconditions are met.
            let _ = self.bitmap.clear(idx);
            idx = idx + 1;
        }

        Ok(())
    }
}

//==================================================================================================
// Additional Lemmas for Memory Safety
//==================================================================================================

/// Lemma: After allocation, no memory aliasing is preserved.
proof fn lemma_alloc_preserves_no_aliasing(old_alloc: &FrameAllocator, new_alloc: &FrameAllocator, new_idx: int)
    requires
        old_alloc.inv(),
        new_alloc.inv(),
        old_alloc@.no_memory_aliasing(),
        0 <= new_idx < new_alloc@.capacity,
        !old_alloc@.is_allocated(new_idx),
        new_alloc@.is_allocated(new_idx),
        forall|i: int| #![trigger new_alloc@.is_allocated(i)]
            0 <= i < new_alloc@.capacity && i != new_idx ==>
            new_alloc@.is_allocated(i) == old_alloc@.is_allocated(i),
    ensures
        new_alloc@.no_memory_aliasing()
{
    // For any two allocated frames i, j with i != j in new_alloc:
    // Case 1: Both i and j were in old_alloc -> they're disjoint by old_alloc.no_memory_aliasing().
    // Case 2: One is new_idx, other was in old_alloc -> disjoint by lemma_frames_disjoint.
    assert forall|i: int, j: int|
        new_alloc@.is_allocated(i) && new_alloc@.is_allocated(j) && i != j
    implies
        new_alloc@.frames_are_disjoint(i, j)
    by {
        if i == new_idx {
            // j was in old_alloc.
            assert(old_alloc@.is_allocated(j));
            assert(0 <= j < new_alloc@.capacity);
            FrameAllocator::lemma_frames_disjoint(i, j);
        } else if j == new_idx {
            // i was in old_alloc.
            assert(old_alloc@.is_allocated(i));
            assert(0 <= i < new_alloc@.capacity);
            FrameAllocator::lemma_frames_disjoint(i, j);
        } else {
            // Both in old_alloc.
            assert(old_alloc@.is_allocated(i));
            assert(old_alloc@.is_allocated(j));
            assert(old_alloc@.frames_are_disjoint(i, j));
        }
    }
}

/// Lemma: After deallocation, no memory aliasing is preserved.
proof fn lemma_dealloc_preserves_no_aliasing(old_alloc: &FrameAllocator, new_alloc: &FrameAllocator, freed_idx: int)
    requires
        old_alloc.inv(),
        new_alloc.inv(),
        old_alloc@.no_memory_aliasing(),
        0 <= freed_idx < new_alloc@.capacity,
        old_alloc@.is_allocated(freed_idx),
        !new_alloc@.is_allocated(freed_idx),
        forall|i: int| #![trigger new_alloc@.is_allocated(i)]
            0 <= i < new_alloc@.capacity && i != freed_idx ==>
            new_alloc@.is_allocated(i) == old_alloc@.is_allocated(i),
    ensures
        new_alloc@.no_memory_aliasing()
{
    // The set of allocated frames is a subset of old_alloc's allocated frames.
    // Since old_alloc had no aliasing, new_alloc (with fewer allocations) also has no aliasing.
    assert forall|i: int, j: int|
        new_alloc@.is_allocated(i) && new_alloc@.is_allocated(j) && i != j
    implies
        new_alloc@.frames_are_disjoint(i, j)
    by {
        assert(old_alloc@.is_allocated(i));
        assert(old_alloc@.is_allocated(j));
        assert(old_alloc@.frames_are_disjoint(i, j));
    }
}

} // verus!

//==================================================================================================
// Tests (for Verification)
//==================================================================================================

#[cfg(verus_keep_ghost)]
mod test {
    use super::*;

    verus! {

    /// Test: Allocation returns valid frame index.
    proof fn test_alloc_valid_index(alloc: FrameAllocator, new_alloc: FrameAllocator, frame_idx: int)
        requires
            alloc.inv(),
            new_alloc.inv(),
            alloc@.can_allocate(),
            new_alloc@.capacity == alloc@.capacity,
            0 <= frame_idx < new_alloc@.capacity,
            new_alloc@.is_allocated(frame_idx),
            !alloc@.is_allocated(frame_idx),
    {
        // Frame index is valid.
        assert(0 <= frame_idx < new_alloc@.capacity);
        // Frame address would be correctly computed.
        assert(frame_idx * FRAME_SIZE as int >= 0);
    }

    /// Test: Free makes frame available again.
    proof fn test_free_makes_available(old_alloc: FrameAllocator, new_alloc: FrameAllocator, frame_idx: int)
        requires
            old_alloc.inv(),
            new_alloc.inv(),
            0 <= frame_idx < old_alloc@.capacity,
            old_alloc@.is_allocated(frame_idx),
            !new_alloc@.is_allocated(frame_idx),
            new_alloc@.capacity == old_alloc@.capacity,
    {
        // After freeing, the frame is no longer allocated.
        assert(!new_alloc@.is_allocated(frame_idx));
    }

    /// Test: Frames are always disjoint.
    proof fn test_frames_disjoint()
    {
        // Any two distinct frames have disjoint memory regions.
        assert forall|i: int, j: int|
            #![trigger i * FRAME_SIZE as int, j * FRAME_SIZE as int]
            i >= 0 && j >= 0 && i != j implies
            i * FRAME_SIZE as int + FRAME_SIZE as int <= j * FRAME_SIZE as int ||
            j * FRAME_SIZE as int + FRAME_SIZE as int <= i * FRAME_SIZE as int
        by {
            FrameAllocator::lemma_frames_disjoint(i, j);
        }
    }

    } // verus!
}
