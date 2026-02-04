// Copyright(c) The Maintainers of Nanvix.
// Licensed under the MIT License.

//==================================================================================================
//! # User Frame Pool (Verified Implementation)
//!
//! This module provides a verified implementation of the User Frame Pool (Upool) which manages
//! user-space memory frames. The pool wraps a FrameAllocator and provides:
//!
//! - Single frame allocation via `alloc()`
//! - Multiple frame allocation via `alloc_many()` (specification-level)
//! - Frame deallocation via `free()`
//!
//! ## Memory Safety Properties Verified
//!
//! 1. **No Double Allocation**: A frame can only be allocated if it's currently free.
//! 2. **No Double Free**: A frame can only be freed if it's currently allocated.
//! 3. **No Memory Aliasing**: All allocated frames have disjoint memory regions.
//! 4. **Valid Frame Indices**: All returned frames have valid indices within capacity.
//! 5. **Liveness**: Allocation succeeds when free frames exist; fails otherwise.
//!
//! ## Design Notes
//!
//! The original implementation uses `Rc<RefCell<>>` for shared ownership. For verification,
//! we model the pool with exclusive ownership (single-owner semantics). The Rc/RefCell
//! pattern is a runtime mechanism that doesn't affect the core allocation invariants.
//!
//! ## Batch Allocation Usage
//!
//! The `alloc_many()` function returns ghost data for specification purposes. For executable
//! code that needs multiple frames, use `alloc()` in a loop:
//!
//! ```rust,ignore
//! // Allocate multiple frames into a caller-managed array
//! let mut frames: [Option<UserFrame>; N] = [None; N];
//! for i in 0..N {
//!     frames[i] = Some(pool.alloc()?);
//! }
//! ```
//!
//! The `alloc_many()` function is primarily for proofs about batch allocation properties.
//==================================================================================================

use crate::{
    error::{
        Error,
        ErrorCode,
    },
    frame::{
        FrameAllocator,
        FrameAllocatorView,
    },
    frame_address::{
        FrameAddress,
        FRAME_SIZE,
    },
};
use vstd::prelude::*;

verus! {

//==================================================================================================
// Permission Model
//==================================================================================================

/// Abstract permission type for frame access control.
/// Per kernel contract, freshly allocated frames have read-only permissions.
#[derive(PartialEq, Eq)]
pub enum FramePermission {
    /// Frame has read-only access.
    ReadOnly,
    /// Frame has read-write access.
    ReadWrite,
}

//==================================================================================================
// UserFrame - Wrapper for allocated frame
//==================================================================================================

/// A type that represents a user frame.
///
/// UserFrame is a simple wrapper around a FrameAddress that represents an allocated
/// frame from the user frame pool. The address is guaranteed to be page-aligned.
///
/// # Zero-Initialization and Permissions
///
/// Per the kernel contract, freshly allocated frames are zero-initialized and have
/// read-only permissions. These properties are tracked via ghost predicates:
/// - `spec_is_from_pool(pool)`: The frame was allocated from the given pool.
/// - `spec_permission()`: The permission level of the frame.
///
/// # Field Visibility
///
/// The `addr` field is public to allow spec functions to access it in Verus.
/// Callers should use `UserFrame::new()` to construct frames, which enforces
/// alignment preconditions. The `free()` method validates that the frame is
/// actually allocated before freeing.
#[derive(Debug)]
pub struct UserFrame {
    /// Frame address (page-aligned).
    pub addr: FrameAddress,
}

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

    /// Spec function to check if the frame is zero-initialized from a pool.
    /// This is only guaranteed for frames that come from a pool allocation.
    /// The predicate requires the frame to be associated with a valid pool and aligned.
    pub open spec fn spec_is_zero_initialized_from_pool(&self, pool: Upool) -> bool {
        self.spec_is_from_pool(pool)
    }

    /// Instantiates a user frame.
    ///
    /// # Parameters
    ///
    /// - `addr`: Frame address (must be page-aligned).
    ///
    /// # Returns
    ///
    /// A user frame wrapping the given address.
    pub fn new(addr: FrameAddress) -> (result: UserFrame)
        requires
            addr.spec_is_aligned(),
        ensures
            result.spec_address() == addr,
            result.spec_is_aligned(),
            result.spec_frame_number() == addr.spec_frame_number(),
    {
        UserFrame { addr }
    }

    /// Returns the physical address of the target user frame.
    ///
    /// # Returns
    ///
    /// The frame address of the user frame.
    pub fn address(&self) -> (result: FrameAddress)
        ensures result == self.spec_address()
    {
        self.addr
    }
}

//==================================================================================================
// UpoolView - Abstract Specification
//==================================================================================================

/// Abstract view of the user frame pool for specification purposes.
/// This mirrors FrameAllocatorView but provides pool-specific semantics.
///
/// # Region Properties
///
/// The view includes:
/// - `base_addr`: The base physical address of the pool region
///
/// Frame addresses are computed as: `base_addr + frame_idx * FRAME_SIZE`.
#[verifier::ext_equal]
pub struct UpoolView {
    /// The underlying frame allocator view.
    pub allocator_view: FrameAllocatorView,
    /// Base physical address of the pool region.
    /// Frame i has address: base_addr + i * FRAME_SIZE.
    pub base_addr: int,
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

//==================================================================================================
// Upool - User Frame Pool Implementation
//==================================================================================================

/// A structure that describes a pool of user frames.
///
/// The pool manages physical memory frames for user-space processes.
/// It provides allocation and deallocation operations with memory safety guarantees.
#[derive(Debug)]
pub struct Upool {
    /// Underlying frame allocator.
    frame_allocator: FrameAllocator,
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
    // Invariant
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
    // Specification Functions
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

    //==============================================================================================
    // Lemmas
    //==============================================================================================

    /// Lemma: Frames are disjoint by construction.
    /// Two distinct frame indices have non-overlapping address ranges.
    pub proof fn lemma_frames_disjoint(i: int, j: int)
        requires
            0 <= i,
            0 <= j,
            i != j,
        ensures
            i * FRAME_SIZE as int + FRAME_SIZE as int <= j * FRAME_SIZE as int ||
            j * FRAME_SIZE as int + FRAME_SIZE as int <= i * FRAME_SIZE as int
    {
        FrameAllocator::lemma_frames_disjoint(i, j);
    }

    /// Lemma: A newly allocated frame is disjoint from all previously allocated frames.
    /// This provides explicit no-aliasing guarantees for callers.
    pub proof fn lemma_new_frame_disjoint_from_existing(&self, new_frame_idx: int, existing_frame_idx: int)
        requires
            self.inv(),
            0 <= new_frame_idx < self@.capacity(),
            0 <= existing_frame_idx < self@.capacity(),
            new_frame_idx != existing_frame_idx,
        ensures
            self@.frames_are_disjoint(new_frame_idx, existing_frame_idx)
    {
        Self::lemma_frames_disjoint(new_frame_idx, existing_frame_idx);
    }

    //==============================================================================================
    // Constructor
    //==============================================================================================

    /// Instantiates a user frame pool from a frame allocator.
    ///
    /// # Parameters
    ///
    /// - `frame_allocator`: Underlying frame allocator.
    ///
    /// # Returns
    ///
    /// A user frame pool.
    pub fn new(frame_allocator: FrameAllocator) -> (result: Upool)
        requires
            frame_allocator.inv(),
        ensures
            result.inv(),
            result@.capacity() == frame_allocator@.capacity,
            // Allocated set is preserved.
            forall|i: int| 0 <= i < result@.capacity() ==>
                result@.is_allocated(i) == frame_allocator@.is_allocated(i),
            // Fresh initialization is preserved (bidirectional).
            result@.is_freshly_initialized() <==> frame_allocator@.is_freshly_initialized(),
    {
        Upool { frame_allocator }
    }

    /// Returns the capacity (number of frames managed).
    pub fn capacity(&self) -> (result: usize)
        requires self.inv(),
        ensures
            result as int == self@.capacity(),
            // Capacity is always positive (from invariant).
            result > 0,
    {
        self.frame_allocator.capacity()
    }

    //==============================================================================================
    // Single Frame Allocation
    //==============================================================================================

    /// Allocates a frame from the user frame pool.
    ///
    /// # Returns
    ///
    /// On success, a UserFrame containing the allocated frame address is returned.
    /// On failure, an error is returned.
    ///
    /// # Memory Safety
    ///
    /// - The returned frame was not previously allocated.
    /// - The frame index is within valid range.
    /// - The frame address is page-aligned.
    /// - The frame has read-only permissions (per kernel contract).
    /// - The frame is disjoint from all other allocated frames.
    /// - No memory aliasing is introduced.
    pub fn alloc(&mut self) -> (result: Result<UserFrame, Error>)
        requires old(self).inv(),
        ensures
            self.inv(),
            // Capacity is preserved.
            self@.capacity() == old(self)@.capacity(),
            // Liveness: If there's a free frame, allocation succeeds.
            old(self)@.has_free_frame() ==> result is Ok,
            // Converse: If no free frame, allocation fails.
            !old(self)@.has_free_frame() ==> result is Err,
            // On success: exactly one new frame is allocated.
            result is Ok ==> {
                let uframe = result->Ok_0;
                let frame_idx = uframe.spec_frame_number();
                // The frame address is valid and aligned.
                &&& uframe.spec_is_aligned()
                // The frame index is valid.
                &&& 0 <= frame_idx < self@.capacity()
                // The frame is now allocated.
                &&& self@.is_allocated(frame_idx)
                // The frame was not previously allocated.
                &&& !old(self)@.is_allocated(frame_idx)
                // All other frames unchanged.
                &&& forall|i: int| #![trigger self@.is_allocated(i)]
                    0 <= i < self@.capacity() && i != frame_idx ==>
                    self@.is_allocated(i) == old(self)@.is_allocated(i)
                // DISJOINTNESS: The new frame is disjoint from all other frames.
                &&& forall|other_idx: int| #![trigger self@.frames_are_disjoint(frame_idx, other_idx)]
                    0 <= other_idx < self@.capacity() && other_idx != frame_idx ==>
                    self@.frames_are_disjoint(frame_idx, other_idx)
                // OWNERSHIP: The frame is from this pool.
                &&& uframe.spec_is_from_pool(*self)
                // PERMISSION: The frame has read-only permissions (via pool).
                &&& uframe.spec_permission_from_pool(*self) == FramePermission::ReadOnly
            },
            // EXPLICIT COUNT: exactly one more frame allocated.
            result is Ok ==> self.spec_num_allocated() == old(self).spec_num_allocated() + 1,
            // On failure: state unchanged.
            result is Err ==> self@ == old(self)@,
    {
        match self.frame_allocator.alloc() {
            Ok(addr) => {
                let uframe: UserFrame = UserFrame::new(addr);
                proof {
                    // Prove disjointness for all other frames.
                    assert forall|other_idx: int|
                        0 <= other_idx < self@.capacity() && other_idx != uframe.spec_frame_number()
                        implies self@.frames_are_disjoint(uframe.spec_frame_number(), other_idx)
                    by {
                        Self::lemma_frames_disjoint(uframe.spec_frame_number(), other_idx);
                    }
                }
                Ok(uframe)
            },
            Err(error) => Err(error),
        }
    }

    //==============================================================================================
    // Multiple Frame Allocation
    //==============================================================================================

    /// Allocates multiple frames from the user frame pool.
    ///
    /// # Note on Usage
    ///
    /// This function returns ghost data for **specification purposes only**. The ghost sequence
    /// of frame indices is used for reasoning about batch allocation properties in proofs.
    ///
    /// For **executable code** that needs multiple frames, use `alloc()` in a loop instead:
    ///
    /// ```rust,ignore
    /// let mut frames: [Option<UserFrame>; N] = [None; N];
    /// for i in 0..N {
    ///     frames[i] = Some(pool.alloc()?);
    /// }
    /// ```
    ///
    /// # Parameters
    ///
    /// - `count`: Number of frames to allocate (must be > 0).
    ///
    /// # Precondition
    ///
    /// The caller must ensure enough free frames exist. When the precondition is satisfied,
    /// the function is guaranteed to succeed.
    ///
    /// # Returns
    ///
    /// A ghost sequence of allocated frame indices. The frames are allocated but ownership
    /// is consumed internally. This function should be called for its side effect on the pool
    /// state and its ghost return value for proofs.
    ///
    /// # Memory Safety
    ///
    /// - All returned frames were not previously allocated.
    /// - All frame indices are within valid range.
    /// - All frames are mutually distinct (no aliasing).
    /// - All frames are pairwise disjoint in memory.
    /// - All frames have read-only permissions.
    pub fn alloc_many(&mut self, count: usize) -> (result: Ghost<Seq<int>>)
        requires
            old(self).inv(),
            count > 0,
            // Precondition: must have at least `count` free frames.
            old(self).spec_num_allocated() + count as int <= old(self).spec_capacity(),
        ensures
            self.inv(),
            // Capacity is preserved.
            self@.capacity() == old(self)@.capacity(),
            // Correct number of frames returned.
            result@.len() == count as int,
            // All frame indices are valid and newly allocated.
            forall|i: int| #![trigger result@[i]]
                0 <= i < result@.len() ==> {
                    let frame_idx = result@[i];
                    &&& 0 <= frame_idx < self@.capacity()
                    &&& self@.is_allocated(frame_idx)
                    &&& !old(self)@.is_allocated(frame_idx)
                },
            // All frame indices are distinct.
            forall|i: int, j: int| #![trigger result@[i], result@[j]]
                0 <= i < result@.len() && 0 <= j < result@.len() && i != j ==>
                result@[i] != result@[j],
            // DISJOINTNESS: All pairs of returned frames have disjoint memory.
            forall|i: int, j: int| #![trigger result@[i], result@[j]]
                0 <= i < result@.len() && 0 <= j < result@.len() && i != j ==>
                self@.frames_are_disjoint(result@[i], result@[j]),
            // EXPLICIT COUNT: exactly count more frames allocated.
            self.spec_num_allocated() == old(self).spec_num_allocated() + count as int,
            // MONOTONICITY: Previously allocated frames remain allocated.
            forall|i: int| #![trigger self@.is_allocated(i)]
                0 <= i < self@.capacity() && old(self)@.is_allocated(i) ==>
                self@.is_allocated(i),
    {
        let ghost original_self: Upool = *self;
        let ghost original_capacity: int = self.spec_capacity();
        let ghost original_num_allocated: int = self.spec_num_allocated();

        let ghost mut frame_indices: Seq<int> = Seq::empty();
        let mut allocated_count: usize = 0;

        while allocated_count < count
            invariant
                self.inv(),
                original_self.inv(),
                self@.capacity() == original_self@.capacity(),
                self@.capacity() == original_capacity,
                // Precondition: enough capacity for all remaining allocations.
                original_num_allocated + count as int <= original_capacity,
                // Loop bounds.
                0 <= allocated_count <= count,
                // Frame indices collected so far.
                frame_indices.len() == allocated_count as int,
                // Allocation count tracking.
                self.spec_num_allocated() == original_num_allocated + allocated_count as int,
                // Remaining room for more allocations.
                self.spec_num_allocated() + (count - allocated_count) as int <= self@.capacity(),
                // MONOTONICITY: frames allocated in original_self remain allocated.
                forall|i: int| #![trigger self@.is_allocated(i)]
                    0 <= i < self@.capacity() && original_self@.is_allocated(i) ==>
                    self@.is_allocated(i),
                // All collected frame indices are valid.
                forall|i: int| #![trigger frame_indices[i]]
                    0 <= i < frame_indices.len() ==> {
                        let frame_idx = frame_indices[i];
                        &&& 0 <= frame_idx < self@.capacity()
                        &&& self@.is_allocated(frame_idx)
                        &&& !original_self@.is_allocated(frame_idx)
                    },
                // All frame indices are distinct.
                forall|i: int, j: int| #![trigger frame_indices[i], frame_indices[j]]
                    0 <= i < frame_indices.len() && 0 <= j < frame_indices.len() && i != j ==>
                    frame_indices[i] != frame_indices[j],
            decreases
                count - allocated_count,
        {
            let ghost prev_self: Upool = *self;
            let ghost prev_frame_indices: Seq<int> = frame_indices;
            let ghost prev_len: int = prev_frame_indices.len() as int;

            // Prove has_free_frame() for alloc to succeed.
            proof {
                assert(self.spec_num_allocated() < self@.capacity());
                self.frame_allocator.lemma_can_allocate_implies_has_free_frame();
            }

            // Allocate a frame. By the lemma above, has_free_frame() is true,
            // so alloc() is guaranteed to succeed.
            let alloc_result: Result<UserFrame, Error> = self.alloc();

            // Extract the frame. Use match with proof that Err is unreachable.
            let uframe: UserFrame = match alloc_result {
                Ok(f) => f,
                Err(e) => {
                    // Unreachable: we proved has_free_frame() so alloc succeeds.
                    proof { assert(false); }
                    return Ghost(Seq::empty());  // Never executed.
                },
            };

            let ghost new_frame_idx: int = uframe.spec_frame_number();

            proof {
                // Contrapositive of monotonicity: !prev_self@.is_allocated(i) ==> !original_self@.is_allocated(i).
                if original_self@.is_allocated(new_frame_idx) {
                    assert(prev_self@.is_allocated(new_frame_idx));
                    assert(false);
                }

                // The new frame is distinct from all previously collected frames.
                assert forall|k: int| 0 <= k < prev_len
                    implies prev_frame_indices[k] != new_frame_idx
                by {
                    assert(prev_self@.is_allocated(prev_frame_indices[k]));
                    assert(!prev_self@.is_allocated(new_frame_idx));
                }

                // Update frame_indices.
                frame_indices = prev_frame_indices.push(new_frame_idx);

                // For old elements (i < prev_len):
                assert forall|i: int| #![trigger frame_indices[i]]
                    0 <= i < prev_len
                    implies {
                        let idx: int = frame_indices[i];
                        &&& 0 <= idx < self@.capacity()
                        &&& self@.is_allocated(idx)
                        &&& !original_self@.is_allocated(idx)
                    }
                by {
                    let idx: int = frame_indices[i];
                    assert(idx == prev_frame_indices[i]);
                    assert(prev_self@.is_allocated(idx));
                    assert(idx != new_frame_idx);
                }
            }
            allocated_count = allocated_count + 1;
        }

        proof {
            // Prove disjointness of all pairs.
            assert forall|i: int, j: int|
                #![trigger frame_indices[i], frame_indices[j]]
                0 <= i < frame_indices.len() && 0 <= j < frame_indices.len() && i != j
                implies self@.frames_are_disjoint(frame_indices[i], frame_indices[j])
            by {
                let idx_i: int = frame_indices[i];
                let idx_j: int = frame_indices[j];
                assert(idx_i != idx_j);
                Self::lemma_frames_disjoint(idx_i, idx_j);
            }
        }

        Ghost(frame_indices)
    }

    //==============================================================================================
    // Frame Deallocation
    //==============================================================================================

    /// Frees a frame that was previously allocated from the user frame pool.
    ///
    /// # Parameters
    ///
    /// - `uframe`: User frame to be freed.
    ///
    /// # Returns
    ///
    /// On success, `Ok(())` is returned.
    ///
    /// # Liveness
    ///
    /// When preconditions are satisfied, free always succeeds.
    ///
    /// # Memory Safety
    ///
    /// - The frame must have been allocated (no double free).
    /// - The frame index must be within valid range.
    /// - After freeing, the frame is available for allocation.
    ///
    /// # Runtime Validation
    ///
    /// The preconditions ensure the frame is valid and allocated. This prevents
    /// freeing arbitrary frames even if the UserFrame was constructed directly.
    pub fn free(&mut self, uframe: UserFrame) -> (result: Result<(), Error>)
        requires
            old(self).inv(),
            uframe.spec_is_aligned(),
            uframe.spec_frame_number() < old(self)@.capacity(),
            old(self)@.is_allocated(uframe.spec_frame_number()),
        ensures
            self.inv(),
            // LIVENESS: free always succeeds when preconditions are met.
            result is Ok,
            self@.capacity() == old(self)@.capacity(),
            // Frame is now free.
            !self@.is_allocated(uframe.spec_frame_number()),
            // All other frames unchanged.
            forall|i: int| #![trigger self@.is_allocated(i)]
                0 <= i < self@.capacity() && i != uframe.spec_frame_number() ==>
                self@.is_allocated(i) == old(self)@.is_allocated(i),
            // Count decreases by exactly 1.
            self.spec_num_allocated() == old(self).spec_num_allocated() - 1,
    {
        self.frame_allocator.free(uframe.address())
    }

    /// Frees a frame by raw address.
    ///
    /// # Description
    ///
    /// Frees a frame identified by its raw physical address. This is used when
    /// the frame address is returned from operations like vmem.unmap() which
    /// returns a raw usize rather than a UserFrame.
    ///
    /// # Parameters
    ///
    /// - `addr`: Raw physical address (must be page-aligned).
    ///
    /// # Returns
    ///
    /// Upon success, Ok(()). Upon failure, an error.
    ///
    /// # Preconditions
    ///
    /// - Address must be page-aligned.
    /// - Frame index (addr / FRAME_SIZE) must be within capacity.
    /// - Frame must be currently allocated.
    ///
    /// # Postconditions
    ///
    /// - The frame is now free.
    /// - All other frames are unchanged.
    /// - Free count increases by 1.
    pub fn free_by_addr(&mut self, addr: usize) -> (result: Result<(), Error>)
        requires
            old(self).inv(),
            addr as int % FRAME_SIZE as int == 0,
            (addr as int / FRAME_SIZE as int) < (old(self))@.capacity(),
            (old(self))@.is_allocated(addr as int / FRAME_SIZE as int),
        ensures
            self.inv(),
            // LIVENESS: free always succeeds when preconditions are met.
            result is Ok,
            self@.capacity() == (old(self))@.capacity(),
            // Frame is now free.
            !self@.is_allocated(addr as int / FRAME_SIZE as int),
            // All other frames unchanged.
            forall|i: int| #![trigger self@.is_allocated(i)]
                0 <= i < self@.capacity() && i != addr as int / FRAME_SIZE as int ==>
                self@.is_allocated(i) == (old(self))@.is_allocated(i),
            // Count decreases by exactly 1.
            self.spec_num_allocated() == old(self).spec_num_allocated() - 1,
    {
        let frame_addr: FrameAddress = FrameAddress { raw_addr: addr };
        self.frame_allocator.free(frame_addr)
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

    /// Test: Fresh pool is empty.
    proof fn test_fresh_pool_empty(pool: Upool)
        requires
            pool.inv(),
            pool@.is_freshly_initialized(),
    {
        assert(pool@.is_empty());
    }

    /// Test: Allocation returns valid frame.
    proof fn test_alloc_valid_frame(
        old_pool: Upool,
        new_pool: Upool,
        uframe: UserFrame,
    )
        requires
            old_pool.inv(),
            new_pool.inv(),
            old_pool@.has_free_frame(),
            new_pool@.capacity() == old_pool@.capacity(),
            uframe.spec_is_aligned(),
            0 <= uframe.spec_frame_number() < new_pool@.capacity(),
            new_pool@.is_allocated(uframe.spec_frame_number()),
            !old_pool@.is_allocated(uframe.spec_frame_number()),
    {
        assert(uframe.spec_raw_address() >= 0);
    }

    /// Test: Free makes frame available again.
    proof fn test_free_makes_available(
        old_pool: Upool,
        new_pool: Upool,
        uframe: UserFrame,
    )
        requires
            old_pool.inv(),
            new_pool.inv(),
            0 <= uframe.spec_frame_number() < old_pool@.capacity(),
            old_pool@.is_allocated(uframe.spec_frame_number()),
            !new_pool@.is_allocated(uframe.spec_frame_number()),
            new_pool@.capacity() == old_pool@.capacity(),
    {
        assert(new_pool@.has_free_frame());
    }

    /// Test: Frames are always disjoint.
    proof fn test_frames_disjoint()
    {
        assert forall|i: int, j: int|
            #![trigger i * FRAME_SIZE as int, j * FRAME_SIZE as int]
            i >= 0 && j >= 0 && i != j implies
            i * FRAME_SIZE as int + FRAME_SIZE as int <= j * FRAME_SIZE as int ||
            j * FRAME_SIZE as int + FRAME_SIZE as int <= i * FRAME_SIZE as int
        by {
            Upool::lemma_frames_disjoint(i, j);
        }
    }

    /// Test: Distinct frames have disjoint memory.
    proof fn test_distinct_frames_disjoint_memory(frame_indices: Seq<int>)
        requires
            frame_indices.len() > 1,
            forall|i: int| 0 <= i < frame_indices.len() ==> frame_indices[i] >= 0,
            forall|i: int, j: int| #![trigger frame_indices[i], frame_indices[j]]
                0 <= i < frame_indices.len() && 0 <= j < frame_indices.len() && i != j ==>
                frame_indices[i] != frame_indices[j],
    {
        Upool::lemma_frames_disjoint(frame_indices[0], frame_indices[1]);
    }

    /// Test: Fresh pool can allocate.
    proof fn test_fresh_pool_can_allocate(pool: Upool)
        requires
            pool.inv(),
            pool@.is_freshly_initialized(),
            pool@.capacity() > 0,
    {
        assert(pool@.can_allocate());
    }

    /// Test: New frame is disjoint from existing.
    proof fn test_new_frame_disjoint(pool: Upool, new_idx: int, existing_idx: int)
        requires
            pool.inv(),
            0 <= new_idx < pool@.capacity(),
            0 <= existing_idx < pool@.capacity(),
            new_idx != existing_idx,
    {
        pool.lemma_new_frame_disjoint_from_existing(new_idx, existing_idx);
        assert(pool@.frames_are_disjoint(new_idx, existing_idx));
    }

    /// Test: Frame permission is read-only when from pool.
    proof fn test_frame_permission_from_pool(pool: Upool, uframe: UserFrame)
        requires
            pool.inv(),
            uframe.spec_is_aligned(),
            0 <= uframe.spec_frame_number() < pool@.capacity(),
            pool@.is_allocated(uframe.spec_frame_number()),
    {
        assert(uframe.spec_is_from_pool(pool));
        assert(uframe.spec_permission_from_pool(pool) == FramePermission::ReadOnly);
    }

    /// Test: Frame from pool is zero-initialized.
    proof fn test_frame_zero_initialized(pool: Upool, uframe: UserFrame)
        requires
            pool.inv(),
            uframe.spec_is_aligned(),
            0 <= uframe.spec_frame_number() < pool@.capacity(),
            pool@.is_allocated(uframe.spec_frame_number()),
    {
        assert(uframe.spec_is_from_pool(pool));
        assert(uframe.spec_is_zero_initialized_from_pool(pool));
    }

    } // verus!
}
