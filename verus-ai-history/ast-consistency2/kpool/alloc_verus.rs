    pub fn alloc(&mut self) -> (result: Result<KernelFrame, Error>)
        requires old(self).inv(),
        ensures
            self.inv(),
            // Capacity is preserved.
            self@.capacity() == old(self)@.capacity(),
            // Pool ID is preserved.
            self@.id() == old(self)@.id(),
            // Base address is preserved.
            self@.base() == old(self)@.base(),
            // Liveness: If there's a free frame, allocation succeeds.
            old(self)@.has_free_frame() ==> result is Ok,
            // Converse: If no free frame, allocation fails.
            !old(self)@.has_free_frame() ==> result is Err,
            // On success: exactly one new frame is allocated.
            result is Ok ==> {
                let kframe = result->Ok_0;
                let frame_idx: int = kframe@.frame_number;
                // The frame satisfies its invariant.
                &&& kframe.inv()
                // The frame address is page-aligned (exposed for external modules).
                &&& kframe.spec_is_aligned()
                // The frame index is valid.
                &&& 0 <= frame_idx < self@.capacity()
                // The frame is now allocated.
                &&& self@.is_allocated(frame_idx)
                // The frame was not previously allocated.
                &&& !old(self)@.is_allocated(frame_idx)
                // PROVENANCE: Frame carries this pool's ID.
                &&& kframe@.pool_id == self@.id()
                // All other frames unchanged.
                &&& forall|i: int| #![trigger self@.is_allocated(i)]
                    0 <= i < self@.capacity() && i != frame_idx ==>
                    self@.is_allocated(i) == old(self)@.is_allocated(i)
            },
            // EXPLICIT COUNT: exactly one more frame allocated.
            result is Ok ==> self@.num_allocated() == old(self)@.num_allocated() + 1,
            // On failure: allocation state unchanged.
            result is Err ==> {
                &&& self@.capacity() == old(self)@.capacity()
                &&& self@.id() == old(self)@.id()
                &&& self@.base() == old(self)@.base()
                &&& forall|i: int| 0 <= i < self@.capacity() ==>
                    self@.is_allocated(i) == old(self)@.is_allocated(i)
            },
    {
        match self.frame_allocator.alloc() {
            Ok(addr) => {
                let kframe: KernelFrame = KernelFrame::new_internal(addr, self.pool_id);
                Ok(kframe)
            },
            Err(error) => Err(error),
        }
    }
