// Copyright(c) The Maintainers of Nanvix.
// Licensed under the MIT License.

//==================================================================================================
// Frame Permission - Ghost Ownership Tracking
//==================================================================================================
//!
//! This module provides ghost (tracked) permissions for frame ownership.
//! A `FramePermission` is a proof-level token that proves exclusive ownership
//! of a specific frame. It is consumed when freeing a frame, ensuring that only
//! the entity that allocated a frame can free it.
//!
//! ## Design
//!
//! - `FramePermission` is a `tracked` struct, meaning it exists only during verification
//!   and has zero runtime cost.
//! - Each `FramePermission` is uniquely tied to a specific frame index.
//! - Allocation returns a `Tracked<FramePermission>` along with the frame index.
//! - Freeing requires consuming the corresponding `Tracked<FramePermission>`.
//! - This enables clients to prove they have exclusive access to allocated frames.
//!
//! ## Memory Safety Guarantee
//!
//! The permission system enforces that:
//! 1. Only the allocator can create permissions (via alloc).
//! 2. Each frame has at most one outstanding permission.
//! 3. Freeing requires the permission, so double-free is impossible.
//! 4. Clients can use permissions to prove exclusive access for downstream verification.

use vstd::prelude::*;

verus! {

//==================================================================================================
// FramePermission - Tracked Ghost Type
//==================================================================================================

/// A ghost permission that proves exclusive ownership of a specific frame.
///
/// This type exists only during verification and has zero runtime cost.
/// It is created by `alloc_tracked` and consumed by `free_tracked`.
///
/// The `frame_idx` field specifies which frame this permission grants ownership of.
/// The `allocator_id` field ties the permission to a specific allocator instance
/// (for cases where multiple allocators might exist).
#[verifier::ext_equal]
pub tracked struct FramePermission {
    /// The frame index this permission is for.
    pub ghost frame_idx: int,
    /// An identifier for the allocator that issued this permission.
    /// This prevents cross-allocator permission confusion.
    pub ghost allocator_id: int,
}

impl FramePermission {
    /// Specification function to get the frame index.
    pub open spec fn spec_frame_idx(&self) -> int {
        self.frame_idx
    }

    /// Specification function to get the allocator ID.
    pub open spec fn spec_allocator_id(&self) -> int {
        self.allocator_id
    }

    /// Creates a new frame permission (proof-only constructor).
    /// This is used internally by the allocator.
    pub proof fn new(frame_idx: int, allocator_id: int) -> (tracked perm: Self)
        ensures
            perm.frame_idx == frame_idx,
            perm.allocator_id == allocator_id,
    {
        FramePermission {
            frame_idx: frame_idx,
            allocator_id: allocator_id,
        }
    }
}

//==================================================================================================
// FramePermissionSet - Collection of Permissions (Specification Only)
//==================================================================================================

/// A specification-level set of frame permissions for range allocations.
///
/// This is used to specify properties about collections of permissions
/// without requiring a concrete tracked implementation.
pub ghost struct FramePermissionSet {
    /// Set of frame indices covered by this permission set.
    pub frame_indices: Set<int>,
    /// The allocator ID these permissions are for.
    pub allocator_id: int,
}

impl FramePermissionSet {
    /// Returns true if this set contains a permission for the given frame index.
    pub open spec fn contains(&self, frame_idx: int) -> bool {
        self.frame_indices.contains(frame_idx)
    }

    /// Returns the number of permissions in the set.
    pub open spec fn len(&self) -> int {
        self.frame_indices.len() as int
    }

    /// Returns true if this set contains permissions for all frames in [start, end).
    pub open spec fn covers_range(&self, start: int, end: int) -> bool {
        forall|i: int| start <= i < end ==> self.contains(i)
    }

    /// Creates an empty permission set specification.
    pub open spec fn empty(allocator_id: int) -> Self {
        FramePermissionSet {
            frame_indices: Set::empty(),
            allocator_id: allocator_id,
        }
    }

    /// Adds a frame index to the set specification.
    pub open spec fn insert(&self, frame_idx: int) -> Self {
        FramePermissionSet {
            frame_indices: self.frame_indices.insert(frame_idx),
            allocator_id: self.allocator_id,
        }
    }
}

//==================================================================================================
// RangePermission - Tracked Ownership of Contiguous Frames
//==================================================================================================

/// A tracked permission that proves exclusive ownership of a contiguous range of frames.
///
/// This type exists only during verification and has zero runtime cost.
/// It is created by `alloc_range_tracked` and consumed by `free_range_tracked`.
///
/// Unlike `FramePermission` which represents a single frame, `RangePermission` represents
/// ownership of a contiguous range [start_frame, start_frame + count).
#[verifier::ext_equal]
pub tracked struct RangePermission {
    /// The starting frame index of the range.
    pub ghost start_frame: int,
    /// The number of frames in the range.
    pub ghost count: int,
    /// The allocator that issued this permission.
    pub ghost allocator_id: int,
}

impl RangePermission {
    /// Specification: get the start frame index.
    pub open spec fn spec_start(&self) -> int {
        self.start_frame
    }

    /// Specification: get the count.
    pub open spec fn spec_count(&self) -> int {
        self.count
    }

    /// Specification: get the end frame index (exclusive).
    pub open spec fn spec_end(&self) -> int {
        self.start_frame + self.count
    }

    /// Specification: get the allocator ID.
    pub open spec fn spec_allocator_id(&self) -> int {
        self.allocator_id
    }

    /// Specification: check if a frame index is covered by this range permission.
    pub open spec fn covers(&self, frame_idx: int) -> bool {
        self.start_frame <= frame_idx < self.start_frame + self.count
    }

    /// Specification: check if this range permission covers all frames in [start, end).
    pub open spec fn covers_range(&self, start: int, end: int) -> bool {
        self.start_frame <= start && end <= self.start_frame + self.count
    }

    /// Creates a new range permission (proof-only constructor).
    /// This is used internally by the allocator.
    pub proof fn new(start_frame: int, count: int, allocator_id: int) -> (tracked perm: Self)
        requires
            count > 0,
        ensures
            perm.start_frame == start_frame,
            perm.count == count,
            perm.allocator_id == allocator_id,
    {
        RangePermission {
            start_frame: start_frame,
            count: count,
            allocator_id: allocator_id,
        }
    }
}

} // verus!
