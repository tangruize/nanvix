    pub fn alloc(&mut self) -> (result: Result<FrameAddress, Error>)
        requires old(self).inv(),
        ensures
            self.inv(),
            // Capacity is preserved.
            self@.capacity == old(self)@.capacity,
            // Liveness: If there's a free frame, allocation succeeds.
            old(self)@.has_free_frame() ==> result is Ok,
            // On success: exactly one new frame is allocated.
            result is Ok ==> {
                let frame = result->Ok_0;
                let frame_idx = frame.spec_frame_number();
                // The frame address is valid and aligned.
                &&& frame.spec_is_aligned()
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
            Ok(frame_idx) => {
                // VERIFIED OPTIMIZATION: The original Nanvix code uses:
                //   let frame_number: FrameNumber = match FrameNumber::from_raw_value(frame_number) { ... }
                // which includes a runtime bounds check (frame_number <= MAX_FRAME_NUMBER).
                // Here we bypass that check by directly constructing FrameNumber, because we have
                // proven that frame_idx <= MAX_FRAME_NUMBER via the invariant:
                //   capacity <= MAX_FRAME_NUMBER + 1  AND  frame_idx < capacity
                // This is safe and eliminates an unnecessary runtime check.
                proof {
                    // By invariant: capacity <= MAX_FRAME_NUMBER + 1.
                    // By bitmap postcondition: frame_idx < capacity.
                    // Therefore: frame_idx <= MAX_FRAME_NUMBER.
                    assert(frame_idx as int <= MAX_FRAME_NUMBER as int);
                }
                let frame_number = FrameNumber { value: frame_idx };
                proof {
                    frame_number.lemma_inv_from_bound();
                }
                let frame_addr = FrameAddress::from_frame_number(frame_number);
                frame_addr
            },
            Err(error) => {
                // NOTE: Original Nanvix code has: error!("{error:?}");
                error.log();
                Err(error)
            },
        }
    }
