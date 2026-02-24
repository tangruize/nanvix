    pub fn alloc_contiguous_range(&mut self, count: usize) -> (result: Result<usize, Error>)
        requires
            old(self).inv(),
            count > 0,
            count as int <= old(self)@.capacity,
        ensures
            self.inv(),
            // Capacity is preserved.
            self@.capacity == old(self)@.capacity,
            // On success: a valid contiguous range is allocated.
            result is Ok ==> {
                let start = result->Ok_0 as int;
                &&& 0 <= start < self@.capacity
                &&& start + count as int <= self@.capacity
                // All frames in range were previously free.
                &&& forall|i: int| start <= i < start + count as int ==>
                    !old(self)@.is_allocated(i)
                // All frames in range are now allocated.
                &&& forall|i: int| start <= i < start + count as int ==>
                    self@.is_allocated(i)
                // All frames outside range unchanged.
                &&& forall|i: int| #![trigger self@.is_allocated(i)]
                    (0 <= i < start || start + count as int <= i < self@.capacity) ==>
                    self@.is_allocated(i) == old(self)@.is_allocated(i)
            },
            // Count tracking on success.
            result is Ok ==> self.spec_num_allocated() == old(self).spec_num_allocated() + count as int,
            // On failure: state unchanged.
            result is Err ==> self@ == old(self)@,
            // Liveness for count=1 (has_free_frame implies a contiguous range of size 1 exists).
            (count == 1 && old(self)@.has_free_frame()) ==> result is Ok,
    {
        proof {
            // For liveness: if count == 1 && has_free_frame, then bitmap.exists_contiguous_free_range(1).
            if count == 1 && old(self)@.has_free_frame() {
                // has_free_frame implies bitmap.has_free_bit by existing lemma.
                old(self).lemma_has_free_frame_implies_bitmap_has_free_bit();
                // bitmap.has_free_bit implies bitmap.exists_contiguous_free_range(1).
                old(self).bitmap.lemma_has_free_bit_implies_exists_free_range_1();
            }
        }

        // Use bitmap's alloc_range which searches for and allocates a contiguous range.
        match self.bitmap.alloc_range(count) {
            Ok(start) => {
                proof {
                    // Connect bitmap.is_bit_set to is_allocated.
                    assert forall|i: int| start as int <= i < start as int + count as int
                        implies !old(self)@.is_allocated(i)
                    by {
                        // old(self).bitmap.all_bits_unset_in_range(start, start + count).
                        assert(!old(self).bitmap.is_bit_set(i));
                        old(self).lemma_allocated_iff_bit_set(i);
                    }

                    assert forall|i: int| start as int <= i < start as int + count as int
                        implies self@.is_allocated(i)
                    by {
                        // self.bitmap.all_bits_set_in_range(start, start + count).
                        assert(self.bitmap.is_bit_set(i));
                        self.lemma_allocated_iff_bit_set(i);
                    }

                    assert forall|i: int|
                        (0 <= i < start as int || start as int + count as int <= i < self@.capacity)
                        implies self@.is_allocated(i) == old(self)@.is_allocated(i)
                    by {
                        assert(self.bitmap.is_bit_set(i) == old(self).bitmap.is_bit_set(i));
                        self.lemma_allocated_iff_bit_set(i);
                        old(self).lemma_allocated_iff_bit_set(i);
                    }
                }
                Ok(start)
            },
            Err(e) => Err(e),
        }
    }
