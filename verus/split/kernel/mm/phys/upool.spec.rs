// Copyright(c) The Maintainers of Nanvix.
// Licensed under the MIT License.

// Specifications.

verus! {

impl UserFrame {
    /// Spec function to get the frame address.
    pub open spec fn spec_address(&self) -> FrameAddress {
        self.addr
    }


    /// Spec function to get the frame number (index).
    pub open spec fn spec_frame_number(&self) -> int {
        self.addr.spec_frame_number()
    }


    /// Spec function to check if the address is page-aligned.
    pub open spec fn spec_is_aligned(&self) -> bool {
        self.addr.spec_is_aligned()
    }


    /// Spec function to get the raw address value.
    pub open spec fn spec_raw_address(&self) -> int {
        self.addr.spec_raw_value()
    }


    /// Spec function to check if the frame was allocated from a specific pool.
    /// This ties the frame to its originating pool for ownership tracking.
    ///
    /// Note: In the verified model, this is established by the alloc() postcondition
    /// which guarantees the returned frame's index is within the pool's capacity
    /// and is marked as allocated.
    pub open spec fn spec_is_from_pool(&self, pool: Upool) -> bool {
        &&& self.spec_is_aligned()  // Frame must be aligned.
        &&& pool.inv()
        &&& 0 <= self.spec_frame_number() < pool@.capacity()
        &&& pool@.is_allocated(self.spec_frame_number())
    }


    /// Spec function to get the permission level of the frame from a pool.
    /// Per kernel contract, frames allocated from the pool have read-only permissions.
    /// The permission is only meaningful for frames that are from the pool.
    pub open spec fn spec_permission_from_pool(&self, pool: Upool) -> FramePermission
        recommends self.spec_is_from_pool(pool)
    {
        FramePermission::ReadOnly
    }
}


impl UpoolView {
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

    /// Returns the pool identifier (uses base address as ID).
    pub open spec fn id(&self) -> int {
        self.base_addr
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


impl View for Upool {
    type V = UpoolView;

    closed spec fn view(&self) -> UpoolView {
        UpoolView {
            allocator_view: self.frame_allocator@,
            // Base address is abstract (default 0).
            base_addr: 0,
        }
    }
}

impl Upool {
    //==============================================================================================

    /// Invariant for the user frame pool.
    /// Ensures internal consistency and memory safety guarantees.
    pub closed spec fn inv(&self) -> bool {
        // The underlying frame allocator must satisfy its invariant.
        &&& self.frame_allocator.inv()
        // View consistency.
        &&& self@.allocator_view == self.frame_allocator@
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
