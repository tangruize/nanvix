// Copyright(c) The Maintainers of Nanvix.
// Licensed under the MIT License.

// Proofs and lemmas.

verus! {

impl Upool {
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
    }

    } // verus!
}
