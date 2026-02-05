// Copyright(c) The Maintainers of Nanvix.
// Licensed under the MIT License.

// Specifications.

verus! {


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

    /// Returns the number of allocated frames (delegated to bitmap's count).
    pub closed spec fn spec_num_allocated(&self) -> int {
        self.bitmap@.usage()
    }
}

} // verus!
