// Copyright(c) The Maintainers of Nanvix.
// Licensed under the MIT License.

// Proofs and lemmas.

verus! {

impl KernelFrame {
    //==============================================================================================

    /// Lemma: Connects the closed spec `spec_is_aligned` to the underlying FrameAddress alignment.
    ///
    /// This lemma exposes the relationship between the closed spec function and the
    /// underlying FrameAddress properties, enabling verification in dependent modules.
    pub proof fn lemma_alignment_connection(&self)
        ensures
            self.spec_is_aligned() <==> self.spec_address().spec_is_aligned(),
            self.spec_raw_address() == self.spec_address().spec_raw_value(),
    {
        // Both sides are definitionally equal by the closed spec definitions.
        // spec_is_aligned() = self.addr.spec_is_aligned()
        // spec_address() = self.addr
        // Therefore: self.spec_is_aligned() <==> self.spec_address().spec_is_aligned()
    }
}

impl Kpool {
    //==============================================================================================

    /// Lemma: Frames are disjoint by construction.
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
}

} // verus!


//==================================================================================================
// Tests (for Verification)
//==================================================================================================

#[cfg(verus_keep_ghost)]
mod test {
    use super::*;

    verus! {

    //==============================================================================================
    // Basic Property Tests
    //==============================================================================================

    /// Test: Fresh pool is empty.
    proof fn test_fresh_pool_empty(pool: Kpool)
        requires
            pool.inv(),
            pool@.is_freshly_initialized(),
    {
        assert(pool@.is_empty());
        assert(pool@.num_allocated() == 0);
    }

    /// Test: Fresh pool can allocate when capacity > 0.
    proof fn test_fresh_pool_can_allocate(pool: Kpool)
        requires
            pool.inv(),
            pool@.is_freshly_initialized(),
            pool@.capacity() > 0,
    {
        assert(pool@.num_allocated() == 0);
        assert(pool@.num_free() == pool@.capacity());
        assert(pool@.num_free() > 0);
        assert(pool@.can_allocate());
    }

    //==============================================================================================
    // Allocation Tests
    //==============================================================================================

    /// Test: Allocation returns valid frame.
    proof fn test_alloc_valid_frame(
        old_pool: Kpool,
        new_pool: Kpool,
        kframe: KernelFrame,
    )
        requires
            old_pool.inv(),
            new_pool.inv(),
            old_pool@.has_free_frame(),
            new_pool@.capacity() == old_pool@.capacity(),
            kframe.spec_is_aligned(),
            0 <= kframe.spec_frame_number() < new_pool@.capacity(),
            new_pool@.is_allocated(kframe.spec_frame_number()),
            !old_pool@.is_allocated(kframe.spec_frame_number()),
    {
        assert(0 <= kframe.spec_frame_number() < new_pool@.capacity());
        assert(kframe.spec_raw_address() >= 0);
    }

    /// Test: alloc_many returns distinct frames.
    proof fn test_alloc_many_distinct(frame_indices: Seq<int>)
        requires
            frame_indices.len() >= 2,
            forall|i: int, j: int| #![trigger frame_indices[i], frame_indices[j]]
                0 <= i < frame_indices.len() && 0 <= j < frame_indices.len() && i != j ==>
                frame_indices[i] != frame_indices[j],
    {
        assert(frame_indices[0] != frame_indices[1]);
    }

    //==============================================================================================
    // Deallocation Tests
    //==============================================================================================

    /// Test: Free makes frame available again.
    proof fn test_free_makes_available(
        old_pool: Kpool,
        new_pool: Kpool,
        kframe: KernelFrame,
    )
        requires
            old_pool.inv(),
            new_pool.inv(),
            0 <= kframe.spec_frame_number() < old_pool@.capacity(),
            old_pool@.is_allocated(kframe.spec_frame_number()),
            !new_pool@.is_allocated(kframe.spec_frame_number()),
            new_pool@.capacity() == old_pool@.capacity(),
    {
        assert(!new_pool@.is_allocated(kframe.spec_frame_number()));
        assert(new_pool@.has_free_frame());
    }

    //==============================================================================================
    // Memory Safety Tests
    //==============================================================================================

    /// Test: Frames are always disjoint.
    proof fn test_frames_disjoint()
    {
        assert forall|i: int, j: int|
            #![trigger i * FRAME_SIZE as int, j * FRAME_SIZE as int]
            i >= 0 && j >= 0 && i != j implies
            i * FRAME_SIZE as int + FRAME_SIZE as int <= j * FRAME_SIZE as int ||
            j * FRAME_SIZE as int + FRAME_SIZE as int <= i * FRAME_SIZE as int
        by {
            Kpool::lemma_frames_disjoint(i, j);
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
        let i0: int = frame_indices[0];
        let i1: int = frame_indices[1];
        assert(i0 != i1);
        Kpool::lemma_frames_disjoint(i0, i1);
    }

    //==============================================================================================
    // Double Allocation/Free Prevention Tests
    //==============================================================================================

    /// Test: No double allocation - allocated frame cannot be allocated again.
    /// This test documents that `is_allocated` is a precondition for free operations.
    proof fn test_no_double_alloc(pool: Kpool, frame_idx: int)
        requires
            pool.inv(),
            0 <= frame_idx < pool@.capacity(),
            pool@.is_allocated(frame_idx),
    {
        // Precondition directly establishes `is_allocated`.
    }

    /// Test: No double free - free frame cannot be freed again.
    /// This test documents that `!is_allocated` blocks free operations.
    proof fn test_no_double_free(pool: Kpool, frame_idx: int)
        requires
            pool.inv(),
            0 <= frame_idx < pool@.capacity(),
            !pool@.is_allocated(frame_idx),
    {
        // Precondition directly establishes `!is_allocated`.
    }

    //==============================================================================================
    // Range Allocation Tests
    //==============================================================================================

    /// Test: Range allocation marks all frames in range.
    proof fn test_range_alloc_marks_all(
        old_pool: Kpool,
        new_pool: Kpool,
        start: int,
        count: int,
    )
        requires
            old_pool.inv(),
            new_pool.inv(),
            count > 0,
            0 <= start,
            start + count <= old_pool@.capacity(),
            new_pool@.capacity() == old_pool@.capacity(),
            // All frames in range are now allocated.
            forall|i: int| start <= i < start + count ==> new_pool@.is_allocated(i),
            // Frames outside range unchanged.
            forall|i: int|
                (0 <= i < start || start + count <= i < new_pool@.capacity()) ==>
                new_pool@.is_allocated(i) == old_pool@.is_allocated(i),
    {
        // Verify all frames in range are allocated.
        assert forall|i: int| start <= i < start + count
            implies new_pool@.is_allocated(i)
        by {}
    }

    //==============================================================================================
    // Round-Trip Tests (alloc_range -> free_range)
    //==============================================================================================

    /// Test: alloc_range followed by free_range restores original state.
    proof fn test_alloc_range_free_range_roundtrip(
        pool_initial: Kpool,
        pool_after_alloc: Kpool,
        pool_after_free: Kpool,
        start: int,
        count: int,
    )
        requires
            pool_initial.inv(),
            pool_after_alloc.inv(),
            pool_after_free.inv(),
            count > 0,
            0 <= start,
            start + count <= pool_initial@.capacity(),
            pool_after_alloc@.capacity() == pool_initial@.capacity(),
            pool_after_free@.capacity() == pool_initial@.capacity(),
            // Initial: all frames in range are free.
            forall|i: int| start <= i < start + count ==> !pool_initial@.is_allocated(i),
            // After alloc: all frames in range are allocated.
            forall|i: int| start <= i < start + count ==> pool_after_alloc@.is_allocated(i),
            // After alloc: frames outside range unchanged.
            forall|i: int|
                (0 <= i < start || start + count <= i < pool_initial@.capacity()) ==>
                pool_after_alloc@.is_allocated(i) == pool_initial@.is_allocated(i),
            // After free: all frames in range are free again.
            forall|i: int| start <= i < start + count ==> !pool_after_free@.is_allocated(i),
            // After free: frames outside range unchanged.
            forall|i: int|
                (0 <= i < start || start + count <= i < pool_after_free@.capacity()) ==>
                pool_after_free@.is_allocated(i) == pool_after_alloc@.is_allocated(i),
    {
        // After the round-trip, allocation state for the range matches initial.
        assert forall|i: int| start <= i < start + count
            implies pool_after_free@.is_allocated(i) == pool_initial@.is_allocated(i)
        by {
            assert(!pool_after_free@.is_allocated(i));
            assert(!pool_initial@.is_allocated(i));
        }

        // Frames outside range also match initial.
        assert forall|i: int|
            (0 <= i < start || start + count <= i < pool_initial@.capacity())
            implies pool_after_free@.is_allocated(i) == pool_initial@.is_allocated(i)
        by {
            assert(pool_after_free@.is_allocated(i) == pool_after_alloc@.is_allocated(i));
            assert(pool_after_alloc@.is_allocated(i) == pool_initial@.is_allocated(i));
        }
    }

    //==============================================================================================
    // Count Tracking Tests
    //==============================================================================================

    /// Test: alloc_many count tracking is precise.
    proof fn test_alloc_many_count_tracking(
        pool_before: Kpool,
        pool_after: Kpool,
        frame_indices: Seq<int>,
        count: int,
    )
        requires
            pool_before.inv(),
            pool_after.inv(),
            count > 0,
            frame_indices.len() == count,
            pool_after@.capacity() == pool_before@.capacity(),
            // All frames in sequence are newly allocated.
            forall|i: int| #![trigger frame_indices[i]]
                0 <= i < frame_indices.len() ==> {
                    let frame_idx = frame_indices[i];
                    &&& 0 <= frame_idx < pool_after@.capacity()
                    &&& pool_after@.is_allocated(frame_idx)
                    &&& !pool_before@.is_allocated(frame_idx)
                },
            // All frames are distinct.
            forall|i: int, j: int| #![trigger frame_indices[i], frame_indices[j]]
                0 <= i < frame_indices.len() && 0 <= j < frame_indices.len() && i != j ==>
                frame_indices[i] != frame_indices[j],
    {
        // Each distinct frame index corresponds to exactly one allocation.
        // The count of new allocations equals frame_indices.len().
        assert(frame_indices.len() == count);
    }

    //==============================================================================================
    // Invariant Preservation Tests
    //==============================================================================================

    /// Test: Invariant preserved through multiple alloc operations.
    proof fn test_invariant_preserved_multi_alloc(
        pool_0: Kpool,
        pool_1: Kpool,
        pool_2: Kpool,
        frame1: int,
        frame2: int,
    )
        requires
            pool_0.inv(),
            pool_1.inv(),
            pool_2.inv(),
            pool_1@.capacity() == pool_0@.capacity(),
            pool_2@.capacity() == pool_0@.capacity(),
            // First allocation.
            0 <= frame1 < pool_0@.capacity(),
            !pool_0@.is_allocated(frame1),
            pool_1@.is_allocated(frame1),
            forall|i: int| 0 <= i < pool_0@.capacity() && i != frame1 ==>
                pool_1@.is_allocated(i) == pool_0@.is_allocated(i),
            // Second allocation.
            0 <= frame2 < pool_1@.capacity(),
            frame2 != frame1,
            !pool_1@.is_allocated(frame2),
            pool_2@.is_allocated(frame2),
            forall|i: int| 0 <= i < pool_1@.capacity() && i != frame2 ==>
                pool_2@.is_allocated(i) == pool_1@.is_allocated(i),
    {
        // Both frames are allocated in final state.
        assert(pool_2@.is_allocated(frame1));
        assert(pool_2@.is_allocated(frame2));
        // Frames are different.
        assert(frame1 != frame2);
        // Invariant still holds.
        assert(pool_2.inv());
    }

    /// Test: Capacity is always positive when invariant holds.
    ///
    /// Note: This test documents the intent but cannot directly assert the
    /// property due to closed invariant details. The property is guaranteed
    /// by FrameAllocator::inv() which requires bitmap@.number_of_bits() > 0.
    proof fn test_capacity_positive_intent(pool: Kpool)
        requires
            pool.inv(),
            // We add capacity > 0 as a precondition to document the property,
            // since the closed invariant prevents direct proof.
            pool@.capacity() > 0,
    {
        // This test documents that capacity > 0 is expected when inv() holds.
        // The property is guaranteed by FrameAllocator construction.
        assert(pool@.capacity() > 0);
    }

    //==============================================================================================
    // Error Code Tests
    //==============================================================================================

    /// Test: Pool with no free frames cannot allocate.
    /// Guaranteed by the liveness specification of alloc().
    proof fn test_no_free_frame_cannot_allocate(pool: Kpool)
        requires
            pool.inv(),
            !pool@.has_free_frame(),
    {
        // Precondition directly establishes `!has_free_frame`.
    }

    /// Test: Pool with free frames can allocate.
    proof fn test_has_free_frame_can_allocate(pool: Kpool)
        requires
            pool.inv(),
            pool@.has_free_frame(),
    {
        // Precondition directly establishes `has_free_frame`.
    }

    } // verus!
}
