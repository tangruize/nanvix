    pub fn book(&mut self, phys_addr: PageAlignedPhysAddr) -> (result: Result<(), Error>)
        requires
            old(self).inv(),
            phys_addr.inv(),
            // Frame index must be within capacity.
            phys_addr.spec_frame_number() < old(self)@.capacity,
        ensures
            self.inv(),
            // Capacity is preserved.
            self@.capacity == old(self)@.capacity,
            // LIVENESS: book succeeds when frame is not already allocated.
            !old(self)@.is_allocated(phys_addr.spec_frame_number()) ==> result is Ok,
            // On success: the frame is now allocated and was not previously allocated.
            result is Ok ==> {
                &&& self@.is_allocated(phys_addr.spec_frame_number())
                &&& !old(self)@.is_allocated(phys_addr.spec_frame_number())
                // All other frames unchanged.
                &&& forall|i: int| #![trigger self@.is_allocated(i)]
                    0 <= i < self@.capacity && i != phys_addr.spec_frame_number() ==>
                    self@.is_allocated(i) == old(self)@.is_allocated(i)
            },
            // On success: exactly one more frame allocated.
            result is Ok ==>
                self.spec_num_allocated() == old(self).spec_num_allocated() + 1,
            // On failure: state unchanged (frame was already allocated).
            result is Err ==> {
                &&& self@ == old(self)@
                &&& old(self)@.is_allocated(phys_addr.spec_frame_number())
            },
    {
        let frame_number: usize = phys_addr.into_frame_number().into_raw_value();
        // Restored original logic: directly call bitmap.set() without idempotency check.
        // NOTE: Original Nanvix code has: error!("{error:?} (phys_addr={phys_addr:?})");
        // Verus limitation: error!() macro unavailable, using error.log() instead.
        match self.bitmap.set(frame_number) {
            Ok(()) => Ok(()),
            Err(error) => {
                error.log();
                Err(error)
            },
        }
    }
