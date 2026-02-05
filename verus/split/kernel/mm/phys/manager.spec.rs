// Copyright(c) The Maintainers of Nanvix.
// Licensed under the MIT License.

// Specifications.

verus! {

//==================================================================================================

/// Abstract view of the virtual memory manager for specification purposes.
///
/// The view captures:
/// - Kernel pool state (available kernel frames)
/// - User pool state (available user frames)
/// - Pool identifiers for provenance tracking
#[verifier::ext_equal]
pub ghost struct VirtMemoryManagerView {
    /// Number of free kernel frames.
    pub kpool_free_count: int,
    /// Total kernel pool capacity.
    pub kpool_capacity: int,
    /// Number of free user frames.
    pub upool_free_count: int,
    /// Total user pool capacity.
    pub upool_capacity: int,
    /// Kernel pool identifier.
    pub kpool_id: int,
    /// User pool identifier.
    pub upool_id: int,
}


impl VirtMemoryManagerView {
    //==============================================================================================
    // Capacity Properties
    //==============================================================================================

    /// Returns true if the kernel pool has at least one free frame.
    pub open spec fn has_kpool_capacity(&self) -> bool {
        self.kpool_free_count > 0
    }

    /// Returns true if the user pool has at least one free frame.
    pub open spec fn has_upool_capacity(&self) -> bool {
        self.upool_free_count > 0
    }

    /// Returns true if the kernel pool has at least `count` free frames.
    pub open spec fn has_kpool_capacity_for(&self, count: int) -> bool {
        self.kpool_free_count >= count && count > 0
    }

    /// Returns true if the user pool has at least `count` free frames.
    pub open spec fn has_upool_capacity_for(&self, count: int) -> bool {
        self.upool_free_count >= count && count > 0
    }

    //==============================================================================================
    // Invariant Properties
    //==============================================================================================

    /// Returns true if the pool counts are within valid bounds.
    pub open spec fn pools_valid(&self) -> bool {
        &&& 0 <= self.kpool_free_count <= self.kpool_capacity
        &&& 0 <= self.upool_free_count <= self.upool_capacity
    }
}


impl View for VirtMemoryManager {
    type V = VirtMemoryManagerView;

    closed spec fn view(&self) -> VirtMemoryManagerView {
        VirtMemoryManagerView {
            kpool_free_count: self.kpool@.num_free(),
            kpool_capacity: self.kpool@.capacity(),
            upool_free_count: self.upool@.num_free(),
            upool_capacity: self.upool@.capacity(),
            kpool_id: self.kpool@.id(),
            upool_id: self.upool@.id(),
        }
    }
}

impl VirtMemoryManager {
    //==============================================================================================

    /// Invariant for the virtual memory manager.
    ///
    /// Ensures:
    /// - Both pools satisfy their invariants
    ///
    /// The underlying pool invariants guarantee:
    /// - Capacity > 0
    /// - num_free() >= 0
    /// - num_allocated() <= capacity
    pub closed spec fn inv(&self) -> bool {
        &&& self.kpool.inv()
        &&& self.upool.inv()
    }

    //==============================================================================================

    /// Spec function to check if a frame is allocated from this manager's upool.
    ///
    /// # Parameters
    ///
    /// - `frame_addr`: Physical address of the frame.
    ///
    /// # Returns
    ///
    /// True if the frame is within the upool's range and is currently allocated.
    pub closed spec fn spec_uframe_is_allocated(&self, frame_addr: int) -> bool {
        let frame_idx: int = frame_addr / FRAME_SIZE as int;
        &&& frame_addr % FRAME_SIZE as int == 0
        &&& 0 <= frame_idx < self.upool@.capacity()
        &&& self.upool@.is_allocated(frame_idx)
    }
}

} // verus!
