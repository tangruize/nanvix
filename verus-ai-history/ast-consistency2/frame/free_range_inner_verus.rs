    fn free_range_inner(
        &mut self,
        start_frame: usize,
        end_frame: usize,
        Ghost(original_self): Ghost<FrameAllocator>,
    ) -> (result: Result<(), Error>)
        requires
            old(self).inv(),
            original_self.inv(),
            start_frame < end_frame,
            end_frame as int <= old(self)@.capacity,
            old(self)@.capacity == original_self@.capacity,
            // All frames in range are currently allocated.
            forall|i: int| start_frame as int <= i < end_frame as int ==>
                old(self).bitmap.is_bit_set(i),
            // Frames outside range match original.
            forall|i: int|
                (0 <= i < start_frame as int || end_frame as int <= i < old(self)@.capacity) ==>
                old(self).bitmap.is_bit_set(i) == original_self.bitmap.is_bit_set(i),
        ensures
            self.inv(),
            self@.capacity == original_self@.capacity,
            // LIVENESS: always succeeds.
            result is Ok,
            // All frames in range are now free (bit not set).
            forall|i: int| start_frame as int <= i < end_frame as int ==>
                !self.bitmap.is_bit_set(i),
            // Frames outside range unchanged.
            forall|i: int|
                (0 <= i < start_frame as int || end_frame as int <= i < self@.capacity) ==>
                self.bitmap.is_bit_set(i) == original_self.bitmap.is_bit_set(i),
            // Count decreases by exactly (end_frame - start_frame).
            self.spec_num_allocated() == old(self).spec_num_allocated() - (end_frame - start_frame) as int,
    {
        let mut idx: usize = start_frame;

        while idx < end_frame
            invariant
                // Core invariant preserved.
                self.inv(),
                // Original invariant held.
                original_self.inv(),
                // Capacity unchanged.
                self@.capacity == original_self@.capacity,
                // Loop bounds.
                start_frame <= idx <= end_frame,
                end_frame as int <= self@.capacity,
                // Bits in [start_frame, idx) are now unset.
                forall|i: int| start_frame as int <= i < idx as int ==>
                    !self.bitmap.is_bit_set(i),
                // Bits in [idx, end_frame) are still set.
                forall|i: int| idx as int <= i < end_frame as int ==>
                    self.bitmap.is_bit_set(i),
                // Bits outside [start_frame, end_frame) are unchanged from original.
                forall|i: int|
                    (0 <= i < start_frame as int || end_frame as int <= i < self@.capacity) ==>
                    self.bitmap.is_bit_set(i) == original_self.bitmap.is_bit_set(i),
                // Count tracking: we've removed (idx - start_frame) bits so far.
                self.spec_num_allocated() == old(self).spec_num_allocated() - (idx - start_frame) as int,
            decreases
                end_frame - idx,
        {
            // bitmap.clear always succeeds when preconditions are met.
            let _ = self.bitmap.clear(idx);
            idx = idx + 1;
        }

        Ok(())
    }
