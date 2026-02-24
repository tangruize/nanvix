    pub fn alloc_range_checked(&mut self, start_frame: usize, count: usize) -> (result: Result<(), Error>)
        requires
            old(self).inv(),
            count > 0,
            start_frame as int + count as int <= old(self)@.capacity,
        ensures
            self.inv(),
            // Capacity is preserved.
            self@.capacity == old(self)@.capacity,
            // On success: all frames in range are now allocated.
            result is Ok ==> {
                // All frames in range were previously free.
                &&& forall|i: int| start_frame as int <= i < start_frame as int + count as int ==>
                    !old(self)@.is_allocated(i)
                // All frames in range are now allocated.
                &&& forall|i: int| start_frame as int <= i < start_frame as int + count as int ==>
                    self@.is_allocated(i)
                &&& forall|i: int| #![trigger self@.is_allocated(i)]
                    (0 <= i < start_frame as int || start_frame as int + count as int <= i < self@.capacity) ==>
                    self@.is_allocated(i) == old(self)@.is_allocated(i)
            },
            // On success: count increases by exactly `count`.
            result is Ok ==> self.spec_num_allocated() == old(self).spec_num_allocated() + count as int,
            // On failure: state unchanged and at least one frame was already allocated.
            result is Err ==> {
                &&& self@ == old(self)@
                &&& exists|i: int| start_frame as int <= i < start_frame as int + count as int &&
                    old(self)@.is_allocated(i)
            },
    {
        let end_frame: usize = start_frame + count;

        // Step 1: Check if all frames in the range are free (matches original).
        let mut idx: usize = start_frame;
        while idx < end_frame
            invariant
                self.inv(),
                self@ == old(self)@,
                self@.capacity == old(self)@.capacity,
                start_frame <= idx <= end_frame,
                end_frame as int <= self@.capacity,
                end_frame == start_frame + count,
                // All frames checked so far are free.
                forall|i: int| start_frame as int <= i < idx as int ==>
                    !self@.is_allocated(i),
            decreases
                end_frame - idx,
        {
            match self.bitmap.test(idx) {
                Ok(is_set) => {
                    if is_set {
                        // Frame is already allocated - return error (matches original).
                        // Prove the exists postcondition: idx is a witness.
                        proof {
                            self.lemma_allocated_iff_bit_set(idx as int);
                            // idx is in range and is_allocated(idx) is true.
                            assert(self@.is_allocated(idx as int));
                            // Since self@ == old(self)@, we have old(self)@.is_allocated(idx).
                            assert(old(self)@.is_allocated(idx as int));
                            // Prove the exists postcondition with idx as witness.
                            // Need to show: start_frame <= idx < start_frame + count && is_allocated(idx).
                            let witness_i: int = idx as int;
                            assert(start_frame as int <= witness_i);
                            assert(witness_i < start_frame as int + count as int);
                            assert(old(self)@.is_allocated(witness_i));
                        }
                        return Err(Error::new(ErrorCode::OutOfMemory, "frame is already allocated"));
                    }
                    // Frame is free, continue checking.
                    proof {
                        self.lemma_allocated_iff_bit_set(idx as int);
                    }
                },
                Err(err) => {
                    // VERIFIED: unreachable. bitmap.test() guarantees Ok when
                    // idx < number_of_bits, and idx < end_frame <= capacity == number_of_bits.
                    return Err(err);
                },
            }
            idx = idx + 1;
        }

        // Step 2: All frames are free, allocate them (matches original).
        // At this point we have proven all frames are free.
        proof {
            // Connect to the bitmap level.
            assert forall|i: int| start_frame as int <= i < end_frame as int
                implies !self.bitmap.is_bit_set(i)
            by {
                self.lemma_allocated_iff_bit_set(i);
            }
        }

        let ghost original_self: FrameAllocator = *self;
        let ghost original_capacity: int = self@.capacity;

        match self.alloc_range_inner(start_frame, end_frame, Ghost(original_self), Ghost(original_capacity)) {
            Ok(()) => {
                proof {
                    // All frames in range are now allocated.
                    assert forall|i: int| start_frame as int <= i < end_frame as int
                        implies self@.is_allocated(i)
                    by {
                        assert(self.bitmap.is_bit_set(i));
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
                // Unreachable: alloc_range_inner always succeeds.
                proof { assert(false); }
                Err(e)
            },
        }
    }
