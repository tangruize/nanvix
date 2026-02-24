    pub fn free(&mut self, kframe: KernelFrame) -> (result: Result<(), Error>)
        requires
            old(self).inv(),
            kframe.inv(),
            kframe@.frame_number < old(self)@.capacity(),
            old(self)@.is_allocated(kframe@.frame_number),
            // PROVENANCE: Frame must belong to this pool.
            kframe@.pool_id == old(self)@.id(),
        ensures
            self.inv(),
            self@.capacity() == old(self)@.capacity(),
            self@.id() == old(self)@.id(),
            // Base address is preserved.
            self@.base() == old(self)@.base(),
            // LIVENESS: free always succeeds when preconditions are met.
            result is Ok,
            // On success: the frame is freed.
            !self@.is_allocated(kframe@.frame_number),
            // All other frames unchanged.
            forall|i: int| #![trigger self@.is_allocated(i)]
                0 <= i < self@.capacity() && i != kframe@.frame_number ==>
                self@.is_allocated(i) == old(self)@.is_allocated(i),
            // Count decremented by 1.
            self@.num_allocated() == old(self)@.num_allocated() - 1,
    {
        self.frame_allocator.free(kframe.address())
    }
