    pub fn alloc_range(&mut self, start_frame: usize, count: usize) -> (result: Result<(), Error>)
        requires
            old(self).inv(),
            count > 0,
            start_frame as int + count as int <= old(self)@.capacity(),
            // All frames in range must be initially free.
            forall|i: int| start_frame as int <= i < start_frame as int + count as int ==>
                !old(self)@.is_allocated(i),
        ensures
            self.inv(),
            // Capacity and pool ID are preserved.
            self@.capacity() == old(self)@.capacity(),
            self@.id() == old(self)@.id(),
            // Base address is preserved.
            self@.base() == old(self)@.base(),
            // LIVENESS: alloc_range always succeeds when preconditions are met.
            result is Ok,
            // All frames in range are now allocated.
            forall|i: int| start_frame as int <= i < start_frame as int + count as int ==>
                self@.is_allocated(i),
            // All frames outside range unchanged.
            forall|i: int| #![trigger self@.is_allocated(i)]
                (0 <= i < start_frame as int || start_frame as int + count as int <= i < self@.capacity()) ==>
                self@.is_allocated(i) == old(self)@.is_allocated(i),
            // EXPLICIT COUNT: allocated count increases by exactly `count`.
            self@.num_allocated() == old(self)@.num_allocated() + count as int,
    {
        // Connect Kpool's view to FrameAllocator's view.
        proof {
            assert forall|i: int| start_frame as int <= i < start_frame as int + count as int
                implies !self.frame_allocator@.is_allocated(i)
            by {
                assert(!self@.is_allocated(i));
            }
        }
        self.frame_allocator.alloc_range_checked(start_frame, count)
    }
