    pub fn free_contiguous(&mut self, start_frame: usize, frame_indices: Ghost<Seq<int>>, count: usize) -> (result: Result<(), Error>)
        requires
            old(self).inv(),
            count > 0,
            frame_indices@.len() == count as int,
            start_frame as int + count as int <= old(self)@.capacity(),
            // Frame indices must be the contiguous range [start_frame, start_frame + count).
            forall|i: int| #![trigger frame_indices@[i]]
                0 <= i < frame_indices@.len() ==>
                frame_indices@[i] == start_frame as int + i,
            // All frames in range must be allocated.
            forall|i: int| start_frame as int <= i < start_frame as int + count as int ==>
                old(self)@.is_allocated(i),
        ensures
            self.inv(),
            // Capacity and pool ID are preserved.
            self@.capacity() == old(self)@.capacity(),
            self@.id() == old(self)@.id(),
            // Base address is preserved.
            self@.base() == old(self)@.base(),
            // LIVENESS: always succeeds when preconditions met.
            result is Ok,
            // All frames in the sequence are now free.
            forall|i: int| #![trigger frame_indices@[i]]
                0 <= i < frame_indices@.len() ==>
                !self@.is_allocated(frame_indices@[i]),
            // All frames in range are now free.
            forall|i: int| start_frame as int <= i < start_frame as int + count as int ==>
                !self@.is_allocated(i),
            // Frames outside range unchanged.
            forall|i: int| #![trigger self@.is_allocated(i)]
                (0 <= i < start_frame as int || start_frame as int + count as int <= i < self@.capacity()) ==>
                self@.is_allocated(i) == old(self)@.is_allocated(i),
            // COUNT: allocated count decreases by exactly `count`.
            result is Ok ==> self@.num_allocated() == old(self)@.num_allocated() - count as int,
    {
        // Delegate to free_range since the frame indices are contiguous.
        self.free_range(start_frame, count)
    }
