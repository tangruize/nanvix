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
            result is Ok ==> self@.num_allocated() == old(self)@.num_allocated() + 1,
            // On failure: allocation state unchanged.
            result is Err ==> {
                &&& self@.capacity() == old(self)@.capacity()
                &&& self@.base() == old(self)@.base()
                &&& forall|i: int| 0 <= i < self@.capacity() ==>
                    self@.is_allocated(i) == old(self)@.is_allocated(i)
            },
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
