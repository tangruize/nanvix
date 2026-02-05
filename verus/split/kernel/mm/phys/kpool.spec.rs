// Copyright(c) The Maintainers of Nanvix.
// Licensed under the MIT License.

// Specifications.

verus! {

impl KernelFrame {
    //==============================================================================================

    /// Spec function to get the frame address.
    /// Closed because addr is private.
    pub closed spec fn spec_address(&self) -> FrameAddress {
        self.addr
    }


    /// Spec function to get the frame number (index within the pool).
    /// Closed because it accesses private addr field.
    pub closed spec fn spec_frame_number(&self) -> int {
        self.addr.spec_frame_number()
    }


    /// Spec function to check if the address is page-aligned.
    /// Closed because it accesses private addr field.
    pub closed spec fn spec_is_aligned(&self) -> bool {
        self.addr.spec_is_aligned()
    }


    /// Spec function to get the raw address value.
    /// Closed because it accesses private addr field.
    pub closed spec fn spec_raw_address(&self) -> int {
        self.addr.spec_raw_value()
    }


    /// Spec function to get the pool identifier for provenance tracking.
    /// Closed because it accesses private pool_id field.
    pub closed spec fn spec_pool_id(&self) -> int {
        self.pool_id as int
    }
}


impl KpoolView {
    //==============================================================================================
    // Basic Properties
    //==============================================================================================

    /// Returns the capacity (total number of frames in the pool).
    pub open spec fn capacity(&self) -> int {
        self.allocator_view.capacity
    }

    /// Returns true if a frame at the given index is allocated.
    pub open spec fn is_allocated(&self, frame_idx: int) -> bool {
        self.allocator_view.is_allocated(frame_idx)
    }

    /// Returns the number of allocated frames.
    pub open spec fn num_allocated(&self) -> int {
        self.allocator_view.num_allocated()
    }

    /// Returns the number of free frames.
    pub open spec fn num_free(&self) -> int {
        self.allocator_view.num_free()
    }

    /// Returns true if the pool has at least one free frame (existential).
    pub open spec fn has_free_frame(&self) -> bool {
        self.allocator_view.has_free_frame()
    }

    /// Returns true if the pool can allocate (num_free > 0).
    pub open spec fn can_allocate(&self) -> bool {
        self.allocator_view.can_allocate()
    }

    /// Returns true if the pool is empty (no allocated frames).
    pub open spec fn is_empty(&self) -> bool {
        self.allocator_view.is_empty()
    }

    /// Returns true if the pool is full (all frames allocated).
    pub open spec fn is_full(&self) -> bool {
        self.allocator_view.is_full()
    }

    //==============================================================================================
    // Region Properties
    //==============================================================================================

    /// Returns the base address of the pool region.
    pub open spec fn base(&self) -> int {
        self.base_addr
    }

    /// Returns the pool identifier for provenance tracking.
    pub open spec fn id(&self) -> int {
        self.pool_id
    }

    /// Computes the physical address of a frame given its index.
    pub open spec fn frame_addr(&self, frame_idx: int) -> int {
        self.base_addr + frame_idx * FRAME_SIZE as int
    }

    /// Returns the limit address (one past the last valid address).
    pub open spec fn limit(&self) -> int {
        self.base_addr + self.capacity() * FRAME_SIZE as int
    }

    //==============================================================================================
    // Memory Safety Properties
    //==============================================================================================

    /// Property: All allocated frame indices are within valid range [0, capacity).
    pub open spec fn allocated_frames_in_range(&self) -> bool {
        self.allocator_view.allocated_frames_in_range()
    }

    /// Property: Memory regions of different frames are disjoint.
    pub open spec fn frames_are_disjoint(&self, i: int, j: int) -> bool {
        self.allocator_view.frames_are_disjoint(i, j)
    }

    /// Property: All allocated frames have disjoint memory regions (no aliasing).
    pub open spec fn no_memory_aliasing(&self) -> bool {
        self.allocator_view.no_memory_aliasing()
    }

    //==============================================================================================
    // Initialization Properties
    //==============================================================================================

    /// Property: A freshly initialized pool has no allocated frames.
    pub open spec fn is_freshly_initialized(&self) -> bool {
        self.allocator_view.is_freshly_initialized()
    }
}


impl View for Kpool {
    type V = KpoolView;

    closed spec fn view(&self) -> KpoolView {
        KpoolView {
            allocator_view: self.frame_allocator@,
            // Base address is abstract (default 0).
            base_addr: 0,
            // Pool ID from the struct.
            pool_id: self.pool_id as int,
        }
    }
}

impl Kpool {
    //==============================================================================================

    /// Invariant for the kernel frame pool.
    /// Ensures internal consistency and memory safety guarantees.
    ///
    /// # Properties Guaranteed
    ///
    /// - The underlying frame allocator satisfies its invariant
    /// - Capacity is positive (inherited from FrameAllocator::inv())
    ///
    /// Note: Capacity positivity is guaranteed by FrameAllocator::inv() which requires
    /// `bitmap@.number_of_bits() > 0`. This is implicitly available through inv().
    pub closed spec fn inv(&self) -> bool {
        // The underlying frame allocator must satisfy its invariant.
        // Note: FrameAllocator::inv() includes capacity > 0.
        self.frame_allocator.inv()
    }

    //==============================================================================================

    /// Returns the capacity (total number of frames).
    pub open spec fn spec_capacity(&self) -> int {
        self@.capacity()
    }


    /// Returns the number of allocated frames (bitmap-based, closed).
    /// This is used for counting postconditions.
    pub closed spec fn spec_num_allocated(&self) -> int {
        self.frame_allocator.spec_num_allocated()
    }
}

} // verus!
