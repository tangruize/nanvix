    pub fn free(&mut self, frame: FrameAddress) -> (result: Result<(), Error>)
        requires
            old(self).inv(),
            // Frame address must be aligned.
            frame.spec_is_aligned(),
            // Frame index must be within capacity.
            frame.spec_frame_number() < old(self)@.capacity,
            // Use can_deallocate for clearer specification.
            old(self)@.can_deallocate(frame.spec_frame_number()),
        ensures
            self.inv(),
            // LIVENESS: free always succeeds when preconditions are met.
            result is Ok,
            // Capacity is preserved.
            self@.capacity == old(self)@.capacity,
            // Unconditional guarantees (since success is guaranteed by liveness above).
            // The frame is now free.
            !self@.is_allocated(frame.spec_frame_number()),
            // All other frames unchanged.
            forall|i: int| #![trigger self@.is_allocated(i)]
                0 <= i < self@.capacity && i != frame.spec_frame_number() ==>
                self@.is_allocated(i) == old(self)@.is_allocated(i),
            // EXPLICIT COUNT: exactly one fewer frame allocated.
            self.spec_num_allocated() == old(self).spec_num_allocated() - 1,
    {
        proof {
            frame.lemma_inv_from_aligned();
        }
        let frame_number: usize = frame.into_frame_number().into_raw_value();

        // Explicitly prove the bitmap precondition is satisfied.
        // The invariant connects is_allocated to is_bit_set.
        proof {
            self.lemma_allocated_iff_bit_set(frame_number as int);
        }

        match self.bitmap.clear(frame_number) {
            Ok(()) => Ok(()),
            Err(error) => {
                // NOTE: Original Nanvix code has: error!("{error:?} (frame={frame:?})");
                // The frame parameter provides context about which frame failed.
                error.log();
                Err(error)
            },
        }
    }
