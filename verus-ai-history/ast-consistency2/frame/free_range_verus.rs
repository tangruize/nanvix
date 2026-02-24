    pub fn free_range(&mut self, start_frame: usize, count: usize) -> (result: Result<(), Error>)
        requires
            old(self).inv(),
            count > 0,
            start_frame as int + count as int <= old(self)@.capacity,
            // All frames in range must be allocated.
            forall|i: int| start_frame as int <= i < start_frame as int + count as int ==>
                old(self)@.is_allocated(i),
        ensures
            self.inv(),
            // Capacity is preserved.
            self@.capacity == old(self)@.capacity,
            // LIVENESS: always succeeds when preconditions met.
            result is Ok,
            // All frames in range are now free.
            forall|i: int| start_frame as int <= i < start_frame as int + count as int ==>
                !self@.is_allocated(i),
            // All frames outside range unchanged.
            forall|i: int| #![trigger self@.is_allocated(i)]
                (0 <= i < start_frame as int || start_frame as int + count as int <= i < self@.capacity) ==>
                self@.is_allocated(i) == old(self)@.is_allocated(i),
            // COUNT: allocated count decreases by exactly `count`.
            self.spec_num_allocated() == old(self).spec_num_allocated() - count as int,
    {
        let end_frame: usize = start_frame + count;
        let ghost original_self: FrameAllocator = *self;

        // Prove preconditions for inner function: connect is_allocated to is_bit_set.
        proof {
            assert forall|i: int| start_frame as int <= i < end_frame as int
                implies self.bitmap.is_bit_set(i)
            by {
                self.lemma_allocated_iff_bit_set(i);
            }
        }

        match self.free_range_inner(start_frame, end_frame, Ghost(original_self)) {
            Ok(()) => {
                // Connect helper's bitmap postconditions to caller's is_allocated postconditions.
                proof {
                    // All frames in range are now free.
                    assert forall|i: int| start_frame as int <= i < end_frame as int
                        implies !self@.is_allocated(i)
                    by {
                        assert(!self.bitmap.is_bit_set(i));
                        self.lemma_allocated_iff_bit_set(i);
                    }

                    // All frames outside range unchanged.
                    assert forall|i: int|
                        (0 <= i < start_frame as int || end_frame as int <= i < self@.capacity)
                        implies self@.is_allocated(i) == original_self@.is_allocated(i)
                    by {
                        assert(self.bitmap.is_bit_set(i) == original_self.bitmap.is_bit_set(i));
                        self.lemma_allocated_iff_bit_set(i);
                        original_self.lemma_allocated_iff_bit_set(i);
                    }
                }
                Ok(())
            },
            Err(e) => {
                // Unreachable: free_range_inner always succeeds.
                proof { assert(false); }
                Err(e)
            },
        }
    }
