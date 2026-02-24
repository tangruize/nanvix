    pub fn alloc_range(&mut self, region: &TruncatedMemoryRegion) -> (result: Result<(), Error>)
        requires
            old(self).inv(),
            region.inv(),
            // Region must be within allocator capacity.
            region.spec_start_frame() >= 0,
            region.spec_start_frame() + region.spec_frame_count() <= old(self)@.capacity,
        ensures
            self.inv(),
            // Capacity is preserved.
            self@.capacity == old(self)@.capacity,
            // On success: all frames in region are now allocated.
            result is Ok ==> {
                let start = region.spec_start_frame();
                let count = region.spec_frame_count();
                // All frames in range are now allocated.
                &&& forall|i: int| start <= i < start + count ==>
                    self@.is_allocated(i)
                // All frames outside range unchanged.
                &&& forall|i: int| #![trigger self@.is_allocated(i)]
                    (0 <= i < start || start + count <= i < self@.capacity) ==>
                    self@.is_allocated(i) == old(self)@.is_allocated(i)
                // COUNT: allocated count increases by frame_count.
                &&& self.spec_num_allocated() == old(self).spec_num_allocated() + count
            },
            // On failure: state unchanged and some frame was already allocated.
            result is Err ==> {
                &&& self@ == old(self)@
                &&& exists|i: int| region.spec_start_frame() <= i < region.spec_start_frame() + region.spec_frame_count()
                    && old(self)@.is_allocated(i)
            },
            // Liveness: if all frames in range are free, allocation succeeds.
            (forall|i: int| region.spec_start_frame() <= i < region.spec_start_frame() + region.spec_frame_count() ==>
                !old(self)@.is_allocated(i)) ==> result is Ok,
    {
        // Use lemma to reveal that inv() implies frame_count > 0.
        proof {
            region.lemma_inv_implies_frame_count_positive();
        }

        let start_frame: usize = region.start().into_frame_number().into_raw_value();
        let count: usize = region.frame_count();

        // Delegate to alloc_range_checked which implements the check-then-set logic.
        self.alloc_range_checked(start_frame, count)
    }
