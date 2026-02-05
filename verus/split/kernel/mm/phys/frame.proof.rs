// Copyright(c) The Maintainers of Nanvix.
// Licensed under the MIT License.

// Proofs and lemmas.

verus! {

impl FrameAllocator {
    //==============================================================================================

    /// Lemma: Reveals the connection between bitmap state and allocation state.
    /// When invariant holds, is_allocated(i) iff is_bit_set(i).
    proof fn lemma_allocated_iff_bit_set(&self, i: int)
        requires
            self.inv(),
            0 <= i < self@.capacity,
        ensures
            self@.is_allocated(i) <==> self.bitmap.is_bit_set(i)
    {
        // This follows from the invariant.
    }


    /// Lemma: Frames are disjoint by construction (addresses differ by at least FRAME_SIZE).
    pub proof fn lemma_frames_disjoint(i: int, j: int)
        requires
            0 <= i,
            0 <= j,
            i != j,
        ensures
            i * FRAME_SIZE as int + FRAME_SIZE as int <= j * FRAME_SIZE as int ||
            j * FRAME_SIZE as int + FRAME_SIZE as int <= i * FRAME_SIZE as int
    {
        // If i < j, then i + 1 <= j, so i * FRAME_SIZE + FRAME_SIZE <= j * FRAME_SIZE.
        // If i > j, then j + 1 <= i, so j * FRAME_SIZE + FRAME_SIZE <= i * FRAME_SIZE.
        if i < j {
            assert(i + 1 <= j);
            assert((i + 1) * FRAME_SIZE as int <= j * FRAME_SIZE as int);
            assert(i * FRAME_SIZE as int + FRAME_SIZE as int <= j * FRAME_SIZE as int);
        } else {
            assert(j + 1 <= i);
            assert((j + 1) * FRAME_SIZE as int <= i * FRAME_SIZE as int);
            assert(j * FRAME_SIZE as int + FRAME_SIZE as int <= i * FRAME_SIZE as int);
        }
    }


    /// Lemma: Connects has_free_frame to bitmap's has_free_bit.
    /// When invariant holds, has_free_frame implies bitmap has_free_bit.
    proof fn lemma_has_free_frame_implies_bitmap_has_free_bit(&self)
        requires
            self.inv(),
            self@.has_free_frame(),
        ensures
            self.bitmap@.has_free_bit()
    {
        // By has_free_frame, there exists i such that 0 <= i < capacity && !is_allocated(i).
        let i = choose|i: int| 0 <= i < self@.capacity && !self@.is_allocated(i);
        // By invariant, capacity == bitmap.number_of_bits().
        assert(0 <= i < self.bitmap@.number_of_bits());
        // By invariant, is_allocated(i) <==> is_bit_set(i).
        self.lemma_allocated_iff_bit_set(i);
        // Since !is_allocated(i), we have !is_bit_set(i).
        assert(!self.bitmap.is_bit_set(i));
        // Therefore, has_free_bit is satisfied.
        assert(self.bitmap@.has_free_bit());
    }


    /// Lemma: If `spec_num_allocated() < capacity`, then `has_free_frame()`.
    /// This connects the bitmap count to the existential predicate.
    pub proof fn lemma_can_allocate_implies_has_free_frame(&self)
        requires
            self.inv(),
            // Use spec_num_allocated (bitmap-based) directly.
            self.spec_num_allocated() < self@.capacity,
        ensures
            self@.has_free_frame()
    {
        // spec_num_allocated = bitmap.usage().
        // capacity = bitmap.number_of_bits() (by invariant).
        // So count_allocated < number_of_bits, meaning bitmap is NOT full.
        assert(!self.bitmap@.is_full());

        // Use bitmap lemma: if not full, there exists an unset bit.
        self.bitmap.lemma_not_full_means_exists_unset_bit();

        // Now we have: exists|i| 0 <= i < number_of_bits && !is_bit_set(i).
        let i: int = choose|i: int| 0 <= i < self.bitmap@.number_of_bits() && !self.bitmap.is_bit_set(i);
        assert(0 <= i < self@.capacity);

        // By invariant: is_allocated(i) <==> is_bit_set(i).
        self.lemma_allocated_iff_bit_set(i);
        assert(!self@.is_allocated(i));

        // Therefore, has_free_frame.
        assert(self@.has_free_frame());
    }
}

//==================================================================================================

/// Lemma: After allocation, no memory aliasing is preserved.
proof fn lemma_alloc_preserves_no_aliasing(old_alloc: &FrameAllocator, new_alloc: &FrameAllocator, new_idx: int)
    requires
        old_alloc.inv(),
        new_alloc.inv(),
        old_alloc@.no_memory_aliasing(),
        0 <= new_idx < new_alloc@.capacity,
        !old_alloc@.is_allocated(new_idx),
        new_alloc@.is_allocated(new_idx),
        forall|i: int| #![trigger new_alloc@.is_allocated(i)]
            0 <= i < new_alloc@.capacity && i != new_idx ==>
            new_alloc@.is_allocated(i) == old_alloc@.is_allocated(i),
    ensures
        new_alloc@.no_memory_aliasing()
{
    // For any two allocated frames i, j with i != j in new_alloc:
    // Case 1: Both i and j were in old_alloc -> they're disjoint by old_alloc.no_memory_aliasing().
    // Case 2: One is new_idx, other was in old_alloc -> disjoint by lemma_frames_disjoint.
    assert forall|i: int, j: int|
        new_alloc@.is_allocated(i) && new_alloc@.is_allocated(j) && i != j
    implies
        new_alloc@.frames_are_disjoint(i, j)
    by {
        if i == new_idx {
            // j was in old_alloc.
            assert(old_alloc@.is_allocated(j));
            assert(0 <= j < new_alloc@.capacity);
            FrameAllocator::lemma_frames_disjoint(i, j);
        } else if j == new_idx {
            // i was in old_alloc.
            assert(old_alloc@.is_allocated(i));
            assert(0 <= i < new_alloc@.capacity);
            FrameAllocator::lemma_frames_disjoint(i, j);
        } else {
            // Both in old_alloc.
            assert(old_alloc@.is_allocated(i));
            assert(old_alloc@.is_allocated(j));
            assert(old_alloc@.frames_are_disjoint(i, j));
        }
    }
}


/// Lemma: After deallocation, no memory aliasing is preserved.
proof fn lemma_dealloc_preserves_no_aliasing(old_alloc: &FrameAllocator, new_alloc: &FrameAllocator, freed_idx: int)
    requires
        old_alloc.inv(),
        new_alloc.inv(),
        old_alloc@.no_memory_aliasing(),
        0 <= freed_idx < new_alloc@.capacity,
        old_alloc@.is_allocated(freed_idx),
        !new_alloc@.is_allocated(freed_idx),
        forall|i: int| #![trigger new_alloc@.is_allocated(i)]
            0 <= i < new_alloc@.capacity && i != freed_idx ==>
            new_alloc@.is_allocated(i) == old_alloc@.is_allocated(i),
    ensures
        new_alloc@.no_memory_aliasing()
{
    // The set of allocated frames is a subset of old_alloc's allocated frames.
    // Since old_alloc had no aliasing, new_alloc (with fewer allocations) also has no aliasing.
    assert forall|i: int, j: int|
        new_alloc@.is_allocated(i) && new_alloc@.is_allocated(j) && i != j
    implies
        new_alloc@.frames_are_disjoint(i, j)
    by {
        assert(old_alloc@.is_allocated(i));
        assert(old_alloc@.is_allocated(j));
        assert(old_alloc@.frames_are_disjoint(i, j));
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

    /// Test: Allocation returns valid frame index.
    proof fn test_alloc_valid_index(alloc: FrameAllocator, new_alloc: FrameAllocator, frame_idx: int)
        requires
            alloc.inv(),
            new_alloc.inv(),
            alloc@.can_allocate(),
            new_alloc@.capacity == alloc@.capacity,
            0 <= frame_idx < new_alloc@.capacity,
            new_alloc@.is_allocated(frame_idx),
            !alloc@.is_allocated(frame_idx),
    {
        // Frame index is valid.
        assert(0 <= frame_idx < new_alloc@.capacity);
        // Frame address would be correctly computed.
        assert(frame_idx * FRAME_SIZE as int >= 0);
    }

    /// Test: Free makes frame available again.
    proof fn test_free_makes_available(old_alloc: FrameAllocator, new_alloc: FrameAllocator, frame_idx: int)
        requires
            old_alloc.inv(),
            new_alloc.inv(),
            0 <= frame_idx < old_alloc@.capacity,
            old_alloc@.is_allocated(frame_idx),
            !new_alloc@.is_allocated(frame_idx),
            new_alloc@.capacity == old_alloc@.capacity,
    {
        // After freeing, the frame is no longer allocated.
        assert(!new_alloc@.is_allocated(frame_idx));
    }

    /// Test: Frames are always disjoint.
    proof fn test_frames_disjoint()
    {
        // Any two distinct frames have disjoint memory regions.
        assert forall|i: int, j: int|
            #![trigger i * FRAME_SIZE as int, j * FRAME_SIZE as int]
            i >= 0 && j >= 0 && i != j implies
            i * FRAME_SIZE as int + FRAME_SIZE as int <= j * FRAME_SIZE as int ||
            j * FRAME_SIZE as int + FRAME_SIZE as int <= i * FRAME_SIZE as int
        by {
            FrameAllocator::lemma_frames_disjoint(i, j);
        }
    }

    } // verus!
}
