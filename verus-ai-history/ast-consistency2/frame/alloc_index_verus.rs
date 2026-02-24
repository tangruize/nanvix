    pub fn alloc_index(&mut self) -> (result: Result<usize, Error>)
        requires old(self).inv(),
        ensures
            self.inv(),
            // Capacity is preserved.
            self@.capacity == old(self)@.capacity,
            // Liveness: If there's a free frame, allocation succeeds.
            old(self)@.has_free_frame() ==> result is Ok,
            // On success: exactly one new frame is allocated.
            result is Ok ==> {
                let frame_idx = result->Ok_0 as int;
                // The frame index is valid.
                &&& 0 <= frame_idx < self@.capacity
                // The frame is now allocated.
                &&& self@.is_allocated(frame_idx)
                // The frame was not previously allocated.
                &&& !old(self)@.is_allocated(frame_idx)
                // All other frames unchanged.
                &&& forall|i: int| #![trigger self@.is_allocated(i)]
                    0 <= i < self@.capacity && i != frame_idx ==>
                    self@.is_allocated(i) == old(self)@.is_allocated(i)
            },
            // EXPLICIT COUNT: exactly one more frame allocated.
            result is Ok ==> self.spec_num_allocated() == old(self).spec_num_allocated() + 1,
            // On failure: state unchanged AND no free frame existed (contrapositive of liveness).
            result is Err ==> {
                &&& self@ == old(self)@
                &&& !old(self)@.has_free_frame()
            },
    {
        // Use lemma to connect has_free_frame to bitmap's has_free_bit.
        proof {
            if self@.has_free_frame() {
                self.lemma_has_free_frame_implies_bitmap_has_free_bit();
            }
        }
        // Allocate a bit from the bitmap.
        match self.bitmap.alloc() {
            Ok(frame_number) => Ok(frame_number),
            Err(error) => {
                // NOTE: Original Nanvix code has: error!("{error:?}");
                error.log();
                Err(error)
            },
        }
    }
