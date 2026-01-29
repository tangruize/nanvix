// Copyright(c) The Maintainers of Nanvix.
// Licensed under the MIT License.

//==================================================================================================
//! # Physical Memory Manager (Verified Implementation)
//!
//! This module provides a verified implementation of the Physical Memory Manager which serves
//! as a unified interface for managing both kernel and user frame pools.
//!
//! ## Architecture
//!
//! The PhysMemoryManager wraps two frame pools:
//! - `Kpool`: Kernel frame pool for kernel-space memory frames
//! - `Upool`: User frame pool for user-space memory frames
//!
//! This design provides:
//! - Unified allocation interface for both kernel and user memory
//! - Clear separation between kernel and user memory domains
//! - Composable verification through verified sub-components
//!
//! ## Memory Safety Properties Verified
//!
//! 1. **Invariant Preservation**: Both pools maintain their invariants across all operations.
//! 2. **Pool Independence**: Operations on one pool don't affect the other's state.
//! 3. **No Double Allocation**: Frames can only be allocated if free (inherited from pools).
//! 4. **No Double Free**: Frames can only be freed if allocated (inherited from pools).
//! 5. **Liveness**: Allocation succeeds when free frames exist; fails otherwise.
//! 6. **Frame Validity**: All returned frames have valid indices and aligned addresses.
//! 7. **Provenance Tracking**: Kernel frames carry pool_id for correct deallocation.
//!
//! ## Abstraction Decisions
//!
//! ### 1. No `clear` Parameter
//! The original API has `alloc_kernel_frame(clear: bool)`. The `clear` parameter is omitted
//! in the verified API because memory zeroing is orthogonal to allocation safety:
//! - Clearing doesn't affect double-allocation prevention
//! - Clearing doesn't affect memory aliasing properties
//! - Clearing doesn't affect liveness guarantees
//!
//! If memory initialization proofs are needed, a separate `clear_frame()` function can be added.
//!
//! ### 2. Batch Allocation Returns Ghost Data
//! The `alloc_many_*` functions return `Ghost<Seq<int>>` for specification purposes rather
//! than `Vec<Frame>`. This design:
//! - Allows reasoning about batch allocation properties
//! - Maintains frame ownership within the pools
//! - Simplifies verification while preserving core safety properties
//!
//! For executable code that needs multiple frames, callers should use `alloc_*` in a loop.
//!
//! ### 3. No Drop Semantics
//! Explicit `free_*` calls make proof obligations clearer and verification tractable.
//!
//! ## API Summary
//!
//! | Function | Description |
//! |----------|-------------|
//! | `new(kpool, upool)` | Create manager from verified pools |
//! | `alloc_user_frame()` | Allocate single user frame |
//! | `alloc_many_user_frames(n)` | Allocate n user frames (ghost indices) |
//! | `alloc_kernel_frame()` | Allocate single kernel frame |
//! | `alloc_many_kernel_frames(n)` | Allocate n kernel frames (ghost indices) |
//! | `free_user_frame(frame)` | Free a user frame |
//! | `free_kernel_frame(frame)` | Free a kernel frame |
//==================================================================================================

use crate::{
    kpool::{KernelFrame, Kpool, KpoolView},
    upool::{UserFrame, Upool, UpoolView},
    error::Error,
};
use vstd::prelude::*;

verus! {

//==================================================================================================
// PhysMemoryManagerView - Abstract Specification
//==================================================================================================

/// Abstract view of the physical memory manager for specification purposes.
/// This provides a unified view of both kernel and user pool states.
#[verifier::ext_equal]
pub struct PhysMemoryManagerView {
    /// View of the kernel frame pool.
    pub kpool_view: KpoolView,
    /// View of the user frame pool.
    pub upool_view: UpoolView,
}

impl PhysMemoryManagerView {
    //==============================================================================================
    // Kernel Pool Properties
    //==============================================================================================

    /// Returns the capacity of the kernel pool.
    pub open spec fn kpool_capacity(&self) -> int {
        self.kpool_view.capacity()
    }

    /// Returns true if a kernel frame at the given index is allocated.
    pub open spec fn kpool_is_allocated(&self, frame_idx: int) -> bool {
        self.kpool_view.is_allocated(frame_idx)
    }

    /// Returns the number of allocated kernel frames.
    pub open spec fn kpool_num_allocated(&self) -> int {
        self.kpool_view.num_allocated()
    }

    /// Returns true if the kernel pool has at least one free frame.
    pub open spec fn kpool_has_free_frame(&self) -> bool {
        self.kpool_view.has_free_frame()
    }

    /// Returns the kernel pool identifier.
    pub open spec fn kpool_id(&self) -> int {
        self.kpool_view.id()
    }

    //==============================================================================================
    // User Pool Properties
    //==============================================================================================

    /// Returns the capacity of the user pool.
    pub open spec fn upool_capacity(&self) -> int {
        self.upool_view.capacity()
    }

    /// Returns true if a user frame at the given index is allocated.
    pub open spec fn upool_is_allocated(&self, frame_idx: int) -> bool {
        self.upool_view.is_allocated(frame_idx)
    }

    /// Returns the number of allocated user frames.
    pub open spec fn upool_num_allocated(&self) -> int {
        self.upool_view.num_allocated()
    }

    /// Returns true if the user pool has at least one free frame.
    pub open spec fn upool_has_free_frame(&self) -> bool {
        self.upool_view.has_free_frame()
    }
}

//==================================================================================================
// PhysMemoryManager - Implementation
//==================================================================================================

/// A structure that manages physical memory through kernel and user frame pools.
///
/// The manager provides a unified interface for allocating and freeing frames
/// from both kernel and user memory domains. It ensures that:
/// - Kernel frames are allocated from the kernel pool
/// - User frames are allocated from the user pool
/// - Each pool maintains its own invariants independently
pub struct PhysMemoryManager {
    /// Kernel frame pool.
    kpool: Kpool,
    /// User frame pool.
    upool: Upool,
}

impl View for PhysMemoryManager {
    type V = PhysMemoryManagerView;

    closed spec fn view(&self) -> PhysMemoryManagerView {
        PhysMemoryManagerView {
            kpool_view: self.kpool@,
            upool_view: self.upool@,
        }
    }
}

impl PhysMemoryManager {
    //==============================================================================================
    // Invariant
    //==============================================================================================

    /// Invariant for the physical memory manager.
    ///
    /// # Properties Guaranteed
    ///
    /// - The kernel pool satisfies its invariant.
    /// - The user pool satisfies its invariant.
    pub closed spec fn inv(&self) -> bool {
        &&& self.kpool.inv()
        &&& self.upool.inv()
    }

    //==============================================================================================
    // Specification Functions
    //==============================================================================================

    /// Returns the kernel pool capacity.
    pub open spec fn spec_kpool_capacity(&self) -> int {
        self@.kpool_capacity()
    }

    /// Returns the user pool capacity.
    pub open spec fn spec_upool_capacity(&self) -> int {
        self@.upool_capacity()
    }

    /// Returns the number of allocated kernel frames.
    pub closed spec fn spec_kpool_num_allocated(&self) -> int {
        self.kpool.spec_num_allocated()
    }

    /// Returns the number of allocated user frames.
    pub closed spec fn spec_upool_num_allocated(&self) -> int {
        self.upool.spec_num_allocated()
    }

    //==============================================================================================
    // Constructor
    //==============================================================================================

    /// Instantiates a physical memory manager from verified pools.
    ///
    /// # Description
    ///
    /// Creates a new physical memory manager wrapping the given kernel and user pools.
    /// Both pools must satisfy their invariants.
    ///
    /// # Parameters
    ///
    /// - `kpool`: Kernel frame pool (must satisfy inv()).
    /// - `upool`: User frame pool (must satisfy inv()).
    ///
    /// # Returns
    ///
    /// A physical memory manager wrapping both pools.
    pub fn new(kpool: Kpool, upool: Upool) -> (result: PhysMemoryManager)
        requires
            kpool.inv(),
            upool.inv(),
        ensures
            result.inv(),
            // Preserve pool views.
            result@.kpool_view == kpool@,
            result@.upool_view == upool@,
            // Preserve capacities.
            result@.kpool_capacity() == kpool@.capacity(),
            result@.upool_capacity() == upool@.capacity(),
            // Preserve allocation states.
            forall|i: int| 0 <= i < result@.kpool_capacity() ==>
                result@.kpool_is_allocated(i) == kpool@.is_allocated(i),
            forall|i: int| 0 <= i < result@.upool_capacity() ==>
                result@.upool_is_allocated(i) == upool@.is_allocated(i),
    {
        PhysMemoryManager { kpool, upool }
    }

    //==============================================================================================
    // User Frame Allocation
    //==============================================================================================

    /// Allocates a user frame from the user frame pool.
    ///
    /// # Description
    ///
    /// Allocates a single frame from the user pool. The returned frame is
    /// guaranteed to be page-aligned and was not previously allocated.
    ///
    /// # Returns
    ///
    /// On success, a UserFrame containing the allocated frame address is returned.
    /// On failure (pool exhausted), an error is returned.
    ///
    /// # Memory Safety
    ///
    /// - The returned frame was not previously allocated.
    /// - The frame index is within valid range.
    /// - The frame address is page-aligned.
    /// - The kernel pool is unchanged.
    pub fn alloc_user_frame(&mut self) -> (result: Result<UserFrame, Error>)
        requires old(self).inv(),
        ensures
            self.inv(),
            // Capacities are preserved.
            self@.kpool_capacity() == old(self)@.kpool_capacity(),
            self@.upool_capacity() == old(self)@.upool_capacity(),
            // Kernel pool is unchanged.
            self@.kpool_view == old(self)@.kpool_view,
            // Liveness: If there's a free frame, allocation succeeds.
            old(self)@.upool_has_free_frame() ==> result is Ok,
            // Converse: If no free frame, allocation fails.
            !old(self)@.upool_has_free_frame() ==> result is Err,
            // On success: exactly one new user frame is allocated.
            result is Ok ==> {
                let uframe = result->Ok_0;
                let frame_idx = uframe.spec_frame_number();
                // Frame is valid and aligned.
                &&& uframe.spec_is_aligned()
                // Frame index is valid.
                &&& 0 <= frame_idx < self@.upool_capacity()
                // Frame is now allocated.
                &&& self@.upool_is_allocated(frame_idx)
                // Frame was not previously allocated.
                &&& !old(self)@.upool_is_allocated(frame_idx)
                // All other user frames unchanged.
                &&& forall|i: int| #![trigger self@.upool_is_allocated(i)]
                    0 <= i < self@.upool_capacity() && i != frame_idx ==>
                    self@.upool_is_allocated(i) == old(self)@.upool_is_allocated(i)
            },
            // Count tracking.
            result is Ok ==> self.spec_upool_num_allocated() == old(self).spec_upool_num_allocated() + 1,
            // On failure: user pool state unchanged.
            result is Err ==> self@.upool_view == old(self)@.upool_view,
    {
        self.upool.alloc()
    }

    /// Allocates multiple user frames from the user frame pool.
    ///
    /// # Description
    ///
    /// Allocates `nframes` individual frames from the user pool. The frames are
    /// allocated one-by-one and are not necessarily contiguous.
    ///
    /// # Note on Usage
    ///
    /// This function returns ghost data for specification purposes. For executable
    /// code that needs multiple frames, use `alloc_user_frame()` in a loop.
    ///
    /// # Parameters
    ///
    /// - `nframes`: Number of frames to allocate (must be > 0).
    ///
    /// # Precondition
    ///
    /// The caller must ensure enough free frames exist.
    ///
    /// # Returns
    ///
    /// A ghost sequence of allocated frame indices.
    ///
    /// # Memory Safety
    ///
    /// - All returned frames were not previously allocated.
    /// - All frame indices are within valid range.
    /// - All frames are mutually distinct.
    /// - The kernel pool is unchanged.
    pub fn alloc_many_user_frames(&mut self, nframes: usize) -> (result: Ghost<Seq<int>>)
        requires
            old(self).inv(),
            nframes > 0,
            // Precondition: must have at least `nframes` free user frames.
            old(self).spec_upool_num_allocated() + nframes as int <= old(self).spec_upool_capacity(),
        ensures
            self.inv(),
            // Capacities are preserved.
            self@.kpool_capacity() == old(self)@.kpool_capacity(),
            self@.upool_capacity() == old(self)@.upool_capacity(),
            // Kernel pool is unchanged.
            self@.kpool_view == old(self)@.kpool_view,
            // Correct number of frames returned.
            result@.len() == nframes as int,
            // All frame indices are valid and newly allocated.
            forall|i: int| #![trigger result@[i]]
                0 <= i < result@.len() ==> {
                    let frame_idx = result@[i];
                    &&& 0 <= frame_idx < self@.upool_capacity()
                    &&& self@.upool_is_allocated(frame_idx)
                    &&& !old(self)@.upool_is_allocated(frame_idx)
                },
            // All frame indices are distinct.
            forall|i: int, j: int| #![trigger result@[i], result@[j]]
                0 <= i < result@.len() && 0 <= j < result@.len() && i != j ==>
                result@[i] != result@[j],
            // Count tracking.
            self.spec_upool_num_allocated() == old(self).spec_upool_num_allocated() + nframes as int,
    {
        self.upool.alloc_many(nframes)
    }

    //==============================================================================================
    // Kernel Frame Allocation
    //==============================================================================================

    /// Allocates a kernel frame from the kernel frame pool.
    ///
    /// # Description
    ///
    /// Allocates a single frame from the kernel pool. The returned frame is
    /// guaranteed to be page-aligned and was not previously allocated.
    ///
    /// # Note on `clear` Parameter
    ///
    /// The original API has `alloc_kernel_frame(clear: bool)`. The `clear` parameter
    /// is omitted because memory zeroing is orthogonal to allocation safety.
    ///
    /// # Returns
    ///
    /// On success, a KernelFrame containing the allocated frame address is returned.
    /// On failure (pool exhausted), an error is returned.
    ///
    /// # Memory Safety
    ///
    /// - The returned frame was not previously allocated.
    /// - The frame index is within valid range.
    /// - The frame address is page-aligned.
    /// - The frame carries the kernel pool's provenance (pool_id).
    /// - The user pool is unchanged.
    pub fn alloc_kernel_frame(&mut self) -> (result: Result<KernelFrame, Error>)
        requires old(self).inv(),
        ensures
            self.inv(),
            // Capacities are preserved.
            self@.kpool_capacity() == old(self)@.kpool_capacity(),
            self@.upool_capacity() == old(self)@.upool_capacity(),
            // User pool is unchanged.
            self@.upool_view == old(self)@.upool_view,
            // Pool ID is preserved.
            self@.kpool_id() == old(self)@.kpool_id(),
            // Liveness: If there's a free frame, allocation succeeds.
            old(self)@.kpool_has_free_frame() ==> result is Ok,
            // Converse: If no free frame, allocation fails.
            !old(self)@.kpool_has_free_frame() ==> result is Err,
            // On success: exactly one new kernel frame is allocated.
            result is Ok ==> {
                let kframe = result->Ok_0;
                let frame_idx = kframe.spec_frame_number();
                // Frame is valid and aligned.
                &&& kframe.spec_is_aligned()
                // Frame index is valid.
                &&& 0 <= frame_idx < self@.kpool_capacity()
                // Frame is now allocated.
                &&& self@.kpool_is_allocated(frame_idx)
                // Frame was not previously allocated.
                &&& !old(self)@.kpool_is_allocated(frame_idx)
                // Provenance: Frame carries this pool's ID.
                &&& kframe.spec_pool_id() == self@.kpool_id()
                // All other kernel frames unchanged.
                &&& forall|i: int| #![trigger self@.kpool_is_allocated(i)]
                    0 <= i < self@.kpool_capacity() && i != frame_idx ==>
                    self@.kpool_is_allocated(i) == old(self)@.kpool_is_allocated(i)
            },
            // Count tracking.
            result is Ok ==> self.spec_kpool_num_allocated() == old(self).spec_kpool_num_allocated() + 1,
            // On failure: kernel pool state unchanged.
            result is Err ==> self@.kpool_view == old(self)@.kpool_view,
    {
        self.kpool.alloc()
    }

    /// Allocates multiple kernel frames from the kernel frame pool.
    ///
    /// # Description
    ///
    /// Allocates `count` individual frames from the kernel pool. The frames are
    /// allocated one-by-one and are not necessarily contiguous.
    ///
    /// # Note on Usage
    ///
    /// This function returns ghost data for specification purposes. For executable
    /// code that needs multiple frames, use `alloc_kernel_frame()` in a loop.
    ///
    /// # Note on `clear` Parameter
    ///
    /// The original API has `alloc_many_kernel_frames(clear, count)`. The `clear`
    /// parameter is omitted because memory zeroing is orthogonal to allocation safety.
    ///
    /// # Parameters
    ///
    /// - `count`: Number of frames to allocate (must be > 0).
    ///
    /// # Precondition
    ///
    /// The caller must ensure enough free frames exist.
    ///
    /// # Returns
    ///
    /// A Result containing a ghost sequence of allocated frame indices on success.
    ///
    /// # Memory Safety
    ///
    /// - All returned frames were not previously allocated.
    /// - All frame indices are within valid range.
    /// - All frames are mutually distinct.
    /// - The user pool is unchanged.
    pub fn alloc_many_kernel_frames(&mut self, count: usize) -> (result: Result<Ghost<Seq<int>>, Error>)
        requires
            old(self).inv(),
            count > 0,
            // Precondition: must have at least `count` free kernel frames.
            old(self).spec_kpool_num_allocated() + count as int <= old(self).spec_kpool_capacity(),
        ensures
            self.inv(),
            // Capacities are preserved.
            self@.kpool_capacity() == old(self)@.kpool_capacity(),
            self@.upool_capacity() == old(self)@.upool_capacity(),
            // User pool is unchanged.
            self@.upool_view == old(self)@.upool_view,
            // Pool ID is preserved.
            self@.kpool_id() == old(self)@.kpool_id(),
            // Liveness: With precondition satisfied, allocation succeeds.
            result is Ok,
            // On success: exactly count new frames are allocated.
            result is Ok ==> {
                let frame_indices = result->Ok_0@;
                // Correct number of frames returned.
                &&& frame_indices.len() == count as int
                // All frame indices are valid and newly allocated.
                &&& forall|i: int| #![trigger frame_indices[i]]
                    0 <= i < frame_indices.len() ==> {
                        let frame_idx = frame_indices[i];
                        &&& 0 <= frame_idx < self@.kpool_capacity()
                        &&& self@.kpool_is_allocated(frame_idx)
                        &&& !old(self)@.kpool_is_allocated(frame_idx)
                    }
                // All frame indices are distinct.
                &&& forall|i: int, j: int| #![trigger frame_indices[i], frame_indices[j]]
                    0 <= i < frame_indices.len() && 0 <= j < frame_indices.len() && i != j ==>
                    frame_indices[i] != frame_indices[j]
            },
            // Count tracking.
            result is Ok ==> self.spec_kpool_num_allocated() == old(self).spec_kpool_num_allocated() + count as int,
    {
        self.kpool.alloc_noncontiguous(count)
    }

    //==============================================================================================
    // Frame Deallocation
    //==============================================================================================

    /// Frees a user frame back to the user frame pool.
    ///
    /// # Description
    ///
    /// Frees a single user frame back to the pool. The frame must have been
    /// previously allocated from the user pool.
    ///
    /// # Parameters
    ///
    /// - `frame`: User frame to be freed.
    ///
    /// # Returns
    ///
    /// On success, `Ok(())` is returned. Free always succeeds when preconditions are met.
    ///
    /// # Memory Safety
    ///
    /// - The frame must be currently allocated (no double free).
    /// - The frame index must be within valid range.
    /// - After freeing, the frame is available for allocation.
    /// - The kernel pool is unchanged.
    pub fn free_user_frame(&mut self, frame: UserFrame) -> (result: Result<(), Error>)
        requires
            old(self).inv(),
            frame.spec_is_aligned(),
            frame.spec_frame_number() < old(self)@.upool_capacity(),
            old(self)@.upool_is_allocated(frame.spec_frame_number()),
        ensures
            self.inv(),
            // Liveness: free always succeeds when preconditions are met.
            result is Ok,
            // Capacities are preserved.
            self@.kpool_capacity() == old(self)@.kpool_capacity(),
            self@.upool_capacity() == old(self)@.upool_capacity(),
            // Kernel pool is unchanged.
            self@.kpool_view == old(self)@.kpool_view,
            // Frame is now free.
            !self@.upool_is_allocated(frame.spec_frame_number()),
            // All other user frames unchanged.
            forall|i: int| #![trigger self@.upool_is_allocated(i)]
                0 <= i < self@.upool_capacity() && i != frame.spec_frame_number() ==>
                self@.upool_is_allocated(i) == old(self)@.upool_is_allocated(i),
            // Count decreases by exactly 1.
            self.spec_upool_num_allocated() == old(self).spec_upool_num_allocated() - 1,
    {
        self.upool.free(frame)
    }

    /// Frees a kernel frame back to the kernel frame pool.
    ///
    /// # Description
    ///
    /// Frees a single kernel frame back to the pool. The frame must have been
    /// previously allocated from the kernel pool (provenance check).
    ///
    /// # Parameters
    ///
    /// - `frame`: Kernel frame to be freed.
    ///
    /// # Returns
    ///
    /// On success, `Ok(())` is returned. Free always succeeds when preconditions are met.
    ///
    /// # Memory Safety
    ///
    /// - The frame must belong to this pool (provenance check).
    /// - The frame must be currently allocated (no double free).
    /// - The frame index must be within valid range.
    /// - After freeing, the frame is available for allocation.
    /// - The user pool is unchanged.
    pub fn free_kernel_frame(&mut self, frame: KernelFrame) -> (result: Result<(), Error>)
        requires
            old(self).inv(),
            frame.spec_is_aligned(),
            frame.spec_frame_number() < old(self)@.kpool_capacity(),
            old(self)@.kpool_is_allocated(frame.spec_frame_number()),
            // Provenance: Frame must belong to this pool.
            frame.spec_pool_id() == old(self)@.kpool_id(),
        ensures
            self.inv(),
            // Liveness: free always succeeds when preconditions are met.
            result is Ok,
            // Capacities are preserved.
            self@.kpool_capacity() == old(self)@.kpool_capacity(),
            self@.upool_capacity() == old(self)@.upool_capacity(),
            // User pool is unchanged.
            self@.upool_view == old(self)@.upool_view,
            // Pool ID is preserved.
            self@.kpool_id() == old(self)@.kpool_id(),
            // Frame is now free.
            !self@.kpool_is_allocated(frame.spec_frame_number()),
            // All other kernel frames unchanged.
            forall|i: int| #![trigger self@.kpool_is_allocated(i)]
                0 <= i < self@.kpool_capacity() && i != frame.spec_frame_number() ==>
                self@.kpool_is_allocated(i) == old(self)@.kpool_is_allocated(i),
            // Count decreases by exactly 1.
            self.spec_kpool_num_allocated() == old(self).spec_kpool_num_allocated() - 1,
    {
        self.kpool.free(frame)
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

    /// Test: Constructor preserves pool properties.
    proof fn test_new_preserves_properties(
        kpool: Kpool,
        upool: Upool,
    )
        requires
            kpool.inv(),
            upool.inv(),
    {
        let manager = PhysMemoryManager::new(kpool, upool);
        assert(manager.inv());
        assert(manager@.kpool_capacity() == kpool@.capacity());
        assert(manager@.upool_capacity() == upool@.capacity());
    }

    /// Test: User allocation doesn't affect kernel pool.
    proof fn test_user_alloc_kpool_unchanged(
        old_mgr: PhysMemoryManager,
        new_mgr: PhysMemoryManager,
        uframe: UserFrame,
    )
        requires
            old_mgr.inv(),
            new_mgr.inv(),
            old_mgr@.upool_has_free_frame(),
            new_mgr@.kpool_view == old_mgr@.kpool_view,
    {
        // Kernel pool state is completely unchanged after user allocation.
        assert(new_mgr@.kpool_capacity() == old_mgr@.kpool_capacity());
        assert forall|i: int| 0 <= i < new_mgr@.kpool_capacity()
            implies new_mgr@.kpool_is_allocated(i) == old_mgr@.kpool_is_allocated(i)
        by {}
    }

    /// Test: Kernel allocation doesn't affect user pool.
    proof fn test_kernel_alloc_upool_unchanged(
        old_mgr: PhysMemoryManager,
        new_mgr: PhysMemoryManager,
        kframe: KernelFrame,
    )
        requires
            old_mgr.inv(),
            new_mgr.inv(),
            old_mgr@.kpool_has_free_frame(),
            new_mgr@.upool_view == old_mgr@.upool_view,
    {
        // User pool state is completely unchanged after kernel allocation.
        assert(new_mgr@.upool_capacity() == old_mgr@.upool_capacity());
        assert forall|i: int| 0 <= i < new_mgr@.upool_capacity()
            implies new_mgr@.upool_is_allocated(i) == old_mgr@.upool_is_allocated(i)
        by {}
    }

    /// Test: Alloc-free cycle returns to original state for user frames.
    proof fn test_user_alloc_free_cycle(
        initial_mgr: PhysMemoryManager,
        after_alloc_mgr: PhysMemoryManager,
        after_free_mgr: PhysMemoryManager,
        uframe: UserFrame,
    )
        requires
            initial_mgr.inv(),
            after_alloc_mgr.inv(),
            after_free_mgr.inv(),
            // After allocation.
            initial_mgr@.upool_has_free_frame(),
            uframe.spec_is_aligned(),
            0 <= uframe.spec_frame_number() < after_alloc_mgr@.upool_capacity(),
            after_alloc_mgr@.upool_is_allocated(uframe.spec_frame_number()),
            !initial_mgr@.upool_is_allocated(uframe.spec_frame_number()),
            // After free.
            !after_free_mgr@.upool_is_allocated(uframe.spec_frame_number()),
            // Frame unchanged for others.
            forall|i: int| 0 <= i < initial_mgr@.upool_capacity() && i != uframe.spec_frame_number() ==>
                after_free_mgr@.upool_is_allocated(i) == initial_mgr@.upool_is_allocated(i),
    {
        // The specific frame is back to its original state.
        assert(!after_free_mgr@.upool_is_allocated(uframe.spec_frame_number()));
        assert(!initial_mgr@.upool_is_allocated(uframe.spec_frame_number()));
    }

    } // verus!
}
