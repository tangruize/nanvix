// Copyright(c) The Maintainers of Nanvix.
// Licensed under the MIT License.

// Specifications.

verus! {

//==================================================================================================
// KernelFrame Specifications
//==================================================================================================

/// Abstract view of a kernel frame for specification purposes.
///
/// # Description
///
/// Uses abstract types (`int`) instead of concrete types (`usize`, `FrameAddress`)
/// to decouple specifications from implementation details.
#[verifier::ext_equal]
pub struct KernelFrameView {
    /// Frame number (index within the pool).
    pub frame_number: int,
    /// Pool identifier for provenance tracking.
    pub pool_id: int,
}

impl View for KernelFrame {
    type V = KernelFrameView;

    /// View maps the concrete KernelFrame to the abstract KernelFrameView.
    closed spec fn view(&self) -> KernelFrameView {
        KernelFrameView {
            frame_number: self.addr.spec_frame_number(),
            pool_id: self.pool_id as int,
        }
    }
}

impl KernelFrame {
    //==============================================================================================

    /// Invariant for kernel frames.
    ///
    /// # Properties Guaranteed
    ///
    /// - The frame address is page-aligned.
    pub closed spec fn inv(&self) -> bool {
        self.addr.spec_is_aligned()
    }

    //==============================================================================================

    // NOTE: The following spec accessors are kept for backward compatibility with
    // external modules (e.g., kpage.rs). Within kpool, public method specs use
    // view-based `self@.frame_number` and `self@.pool_id` instead.

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


//==================================================================================================
// KpoolView Specifications
//==================================================================================================

impl KpoolView {
    //==============================================================================================
    // Basic Properties
    //==============================================================================================

    /// Returns the capacity (total number of frames in the pool).
    pub open spec fn capacity(&self) -> int {
        self.capacity
    }

    /// Returns true if a frame at the given index is allocated.
    pub open spec fn is_allocated(&self, frame_idx: int) -> bool {
        self.allocated_frames.contains(frame_idx)
    }

    /// Returns the number of allocated frames (concrete count).
    pub open spec fn num_allocated(&self) -> int {
        self.num_allocated_count
    }

    /// Returns the number of free frames.
    pub open spec fn num_free(&self) -> int {
        self.capacity - self.num_allocated_count
    }

    /// Returns true if the pool has at least one free frame (existential).
    pub open spec fn has_free_frame(&self) -> bool {
        exists|i: int| 0 <= i < self.capacity && !self.allocated_frames.contains(i)
    }

    /// Returns true if the pool can allocate (num_free > 0).
    pub open spec fn can_allocate(&self) -> bool {
        self.num_free() > 0
    }

    /// Returns true if the pool is empty (no allocated frames).
    pub open spec fn is_empty(&self) -> bool {
        self.num_allocated_count == 0
    }

    /// Returns true if the pool is full (all frames allocated).
    pub open spec fn is_full(&self) -> bool {
        self.num_allocated_count == self.capacity
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
        self.base_addr + self.capacity * FRAME_SIZE as int
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
    pub open spec fn frames_are_disjoint(&self, i: int, j: int) -> bool {
        let addr_i: int = self.frame_addr(i);
        let addr_j: int = self.frame_addr(j);
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
    // Initialization Properties
    //==============================================================================================

    /// Property: A freshly initialized pool has no allocated frames.
    pub open spec fn is_freshly_initialized(&self) -> bool {
        &&& self.allocated_frames =~= Set::<int>::empty()
        &&& self.num_allocated_count == 0
    }
}


//==================================================================================================
// Kpool View and Invariant
//==================================================================================================

impl View for Kpool {
    type V = KpoolView;

    // NOTE: The `pub` keyword cannot be used on trait impl methods in Rust.
    // The View trait is public, so view() inherits its visibility.
    closed spec fn view(&self) -> KpoolView {
        KpoolView {
            allocated_frames: self.frame_allocator@.allocated_frames,
            capacity: self.frame_allocator@.capacity,
            num_allocated_count: self.frame_allocator.spec_num_allocated(),
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
    /// - The underlying frame allocator satisfies its invariant.
    /// - Capacity is positive (inherited from FrameAllocator::inv()).
    pub closed spec fn inv(&self) -> bool {
        self.frame_allocator.inv()
    }

    //==============================================================================================

    /// Returns the number of allocated frames (bitmap-based).
    /// Private: used only in internal loop invariants and proof blocks.
    /// Public method specs use `self@.num_allocated()` instead.
    spec fn spec_num_allocated(&self) -> int {
        self.frame_allocator.spec_num_allocated()
    }
}

} // verus!
