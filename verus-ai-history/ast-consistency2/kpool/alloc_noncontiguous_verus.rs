    pub fn alloc_noncontiguous(&mut self, count: usize) -> (result: Result<Ghost<Seq<int>>, Error>)
        requires
            old(self).inv(),
            count > 0,
            // Precondition: must have at least `count` free frames.
            old(self)@.num_allocated() + count as int <= old(self)@.capacity(),
        ensures
            self.inv(),
            // Capacity and region are preserved.
            self@.capacity() == old(self)@.capacity(),
            self@.base() == old(self)@.base(),
            self@.id() == old(self)@.id(),
            // Liveness: With precondition satisfied, allocation succeeds.
            result is Ok,
            // On success: exactly count new frames are allocated.
            result is Ok ==> {
                let frame_indices = result->Ok_0@;
                // Correct number of frames returned.
                &&& frame_indices.len() == count as int
                // All frame indices are valid and newly allocated.
                &&& forall|i: int| #![trigger frame_indices[i]]
                    0 <= i < frame_indices.len() ==> {
                        let frame_idx = frame_indices[i];
                        &&& 0 <= frame_idx < self@.capacity()
                        &&& self@.is_allocated(frame_idx)
                        &&& !old(self)@.is_allocated(frame_idx)
                    }
                // All frame indices are distinct (but NOT necessarily contiguous).
                &&& forall|i: int, j: int| #![trigger frame_indices[i], frame_indices[j]]
                    0 <= i < frame_indices.len() && 0 <= j < frame_indices.len() && i != j ==>
                    frame_indices[i] != frame_indices[j]
            },
            // EXPLICIT COUNT: exactly count more frames allocated.
            result is Ok ==> self@.num_allocated() == old(self)@.num_allocated() + count as int,
            // MONOTONICITY: previously allocated frames remain allocated.
            forall|i: int| #![trigger self@.is_allocated(i)]
                0 <= i < self@.capacity() && old(self)@.is_allocated(i) ==>
                self@.is_allocated(i),
    {
        let ghost original_self: Kpool = *self;
        let ghost original_capacity: int = self@.capacity();
        let ghost original_num_allocated: int = self.spec_num_allocated();

        let ghost mut frame_indices: Seq<int> = Seq::empty();
        let mut allocated_count: usize = 0;

        while allocated_count < count
            invariant
                self.inv(),
                original_self.inv(),
                self@.capacity() == original_self@.capacity(),
                self@.capacity() == original_capacity,
                // Pool ID is preserved.
                self@.id() == original_self@.id(),
                self@.base() == original_self@.base(),
                // Precondition: enough capacity for all remaining allocations.
                original_num_allocated + count as int <= original_capacity,
                // Loop bounds.
                0 <= allocated_count <= count,
                // Frame indices collected so far.
                frame_indices.len() == allocated_count as int,
                // Allocation count tracking.
                self.spec_num_allocated() == original_num_allocated + allocated_count as int,
                // Remaining room for more allocations.
                self.spec_num_allocated() + (count - allocated_count) as int <= self@.capacity(),
                // MONOTONICITY: frames allocated in original_self remain allocated.
                forall|i: int| #![trigger self@.is_allocated(i)]
                    0 <= i < self@.capacity() && original_self@.is_allocated(i) ==>
                    self@.is_allocated(i),
                // All collected frame indices are valid.
                forall|i: int| #![trigger frame_indices[i]]
                    0 <= i < frame_indices.len() ==> {
                        let frame_idx = frame_indices[i];
                        &&& 0 <= frame_idx < self@.capacity()
                        &&& self@.is_allocated(frame_idx)
                        &&& !original_self@.is_allocated(frame_idx)
                    },
                // All frame indices are distinct.
                forall|i: int, j: int| #![trigger frame_indices[i], frame_indices[j]]
                    0 <= i < frame_indices.len() && 0 <= j < frame_indices.len() && i != j ==>
                    frame_indices[i] != frame_indices[j],
            decreases
                count - allocated_count,
        {
            let ghost prev_self: Kpool = *self;
            let ghost prev_frame_indices: Seq<int> = frame_indices;
            let ghost prev_len: int = prev_frame_indices.len() as int;

            // We need has_free_frame() for alloc to succeed.
            proof {
                assert(self.spec_num_allocated() < self@.capacity());
                self.frame_allocator.lemma_can_allocate_implies_has_free_frame();
            }

            let kframe: KernelFrame = match self.alloc() {
                Ok(f) => f,
                Err(_) => {
                    proof { assert(false); }
                    return Err(Error::new(ErrorCode::OutOfMemory, "unexpected"));
                },
            };

            let ghost new_frame_idx: int = kframe@.frame_number;

            proof {
                // The new frame is distinct from all previously allocated frames.
                assert forall|k: int| 0 <= k < prev_len
                    implies prev_frame_indices[k] != new_frame_idx
                by {
                    assert(prev_self@.is_allocated(prev_frame_indices[k]));
                    assert(!prev_self@.is_allocated(new_frame_idx));
                }

                // Update frame_indices.
                frame_indices = prev_frame_indices.push(new_frame_idx);

                // For old elements (i < prev_len):
                assert forall|i: int| #![trigger frame_indices[i]]
                    0 <= i < prev_len
                    implies {
                        let idx: int = frame_indices[i];
                        &&& 0 <= idx < self@.capacity()
                        &&& self@.is_allocated(idx)
                        &&& !original_self@.is_allocated(idx)
                    }
                by {
                    let idx: int = frame_indices[i];
                    assert(idx == prev_frame_indices[i]);
                    assert(prev_self@.is_allocated(idx));
                }

                // For the new element (i == prev_len):
                assert({
                    let idx: int = frame_indices[prev_len as int];
                    &&& 0 <= idx < self@.capacity()
                    &&& self@.is_allocated(idx)
                    &&& !original_self@.is_allocated(idx)
                }) by {
                    assert(frame_indices[prev_len as int] == new_frame_idx);
                    assert(!prev_self@.is_allocated(new_frame_idx));
                }
            }
            allocated_count = allocated_count + 1;
        }

        Ok(Ghost(frame_indices))
    }
