    fn alloc_range_inner(
        &mut self,
        start_frame: usize,
        end_frame: usize,
        Ghost(original_self): Ghost<FrameAllocator>,
        Ghost(original_capacity): Ghost<int>,
    ) -> (result: Result<(), Error>)
        requires
            old(self).inv(),
            original_self.inv(),
            start_frame < end_frame,
            end_frame as int <= old(self)@.capacity,
            old(self)@.capacity == original_capacity,
            original_self@.capacity == original_capacity,
            // All frames in range are initially free (bit not set).
            forall|i: int| start_frame as int <= i < end_frame as int ==>
                !old(self).bitmap.is_bit_set(i),
            // Frames outside range match original.
            forall|i: int|
                (0 <= i < start_frame as int || end_frame as int <= i < old(self)@.capacity) ==>
                old(self).bitmap.is_bit_set(i) == original_self.bitmap.is_bit_set(i),
        ensures
            self.inv(),
            self@.capacity == original_capacity,
            // LIVENESS: always succeeds (bitmap.set always succeeds when preconditions met).
            result is Ok,
            forall|i: int| start_frame as int <= i < end_frame as int ==>
                self.bitmap.is_bit_set(i),
            forall|i: int|
                (0 <= i < start_frame as int || end_frame as int <= i < self@.capacity) ==>
                self.bitmap.is_bit_set(i) == original_self.bitmap.is_bit_set(i),
            // Count increases by exactly (end_frame - start_frame).
            self.spec_num_allocated() == old(self).spec_num_allocated() + (end_frame - start_frame) as int,
    {
        let mut idx: usize = start_frame;

        while idx < end_frame
            invariant
                // Core invariant preserved.
                self.inv(),
                // Original invariant held.
                original_self.inv(),
                // Capacity unchanged.
                self@.capacity == original_capacity,
                original_self@.capacity == original_capacity,
                // Loop bounds.
                start_frame <= idx <= end_frame,
                end_frame as int <= self@.capacity,
                // Bits in [start_frame, idx) are set.
                forall|i: int| start_frame as int <= i < idx as int ==>
                    self.bitmap.is_bit_set(i),
                // Bits in [idx, end_frame) are unchanged from old.
                forall|i: int| idx as int <= i < end_frame as int ==>
                    !self.bitmap.is_bit_set(i),
                // Bits outside [start_frame, end_frame) are unchanged from original.
                forall|i: int|
                    (0 <= i < start_frame as int || end_frame as int <= i < self@.capacity) ==>
                    self.bitmap.is_bit_set(i) == original_self.bitmap.is_bit_set(i),
                // Count tracking: we've added (idx - start_frame) bits so far.
                self.spec_num_allocated() == old(self).spec_num_allocated() + (idx - start_frame) as int,
            decreases
                end_frame - idx,
        {
            // VERIFIED: bitmap.set always succeeds when preconditions are met.
            // NOTE: Original Nanvix code has: error!("{error:?} (region={region:?})");
            // This error path is proven unreachable by Verus, so no logging is needed here.
            // The preconditions guarantee idx is in range and the bit is currently unset.
            let _ = self.bitmap.set(idx);
            idx = idx + 1;
        }

        Ok(())
    }
