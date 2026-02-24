    pub fn alloc_contiguous(&mut self, count: usize) -> (result: Result<usize, Error>)
        requires
            old(self).inv(),
            count > 0,
            count as int <= old(self)@.capacity(),
        ensures
            self.inv(),
            // Capacity and pool ID are preserved.
            self@.capacity() == old(self)@.capacity(),
            self@.id() == old(self)@.id(),
            // Base address is preserved.
            self@.base() == old(self)@.base(),
            // On success: a valid contiguous range is allocated.
            result is Ok ==> {
                let start = result->Ok_0 as int;
                &&& 0 <= start < self@.capacity()
                &&& start + count as int <= self@.capacity()
                // All frames in range were previously free.
                &&& forall|i: int| start <= i < start + count as int ==>
                    !old(self)@.is_allocated(i)
                // All frames in range are now allocated.
                &&& forall|i: int| start <= i < start + count as int ==>
                    self@.is_allocated(i)
                // All frames outside range unchanged.
                &&& forall|i: int| #![trigger self@.is_allocated(i)]
                    (0 <= i < start || start + count as int <= i < self@.capacity()) ==>
                    self@.is_allocated(i) == old(self)@.is_allocated(i)
            },
            // Count tracking on success.
            result is Ok ==> self@.num_allocated() == old(self)@.num_allocated() + count as int,
            // On failure: allocation state unchanged.
            result is Err ==> {
                &&& self@.capacity() == old(self)@.capacity()
                &&& self@.id() == old(self)@.id()
                &&& self@.base() == old(self)@.base()
                &&& forall|i: int| 0 <= i < self@.capacity() ==>
                    self@.is_allocated(i) == old(self)@.is_allocated(i)
            },
            // Liveness for count=1.
            (count == 1 && old(self)@.has_free_frame()) ==> result is Ok,
    {
        match self.frame_allocator.alloc_contiguous_range(count) {
            Ok(start) => {
                proof {
                    // Connect FrameAllocator view to Kpool view.
                    assert forall|i: int| start as int <= i < start as int + count as int
                        implies !old(self)@.is_allocated(i)
                    by {
                        assert(!old(self).frame_allocator@.is_allocated(i));
                    }

                    assert forall|i: int| start as int <= i < start as int + count as int
                        implies self@.is_allocated(i)
                    by {
                        assert(self.frame_allocator@.is_allocated(i));
                    }

                    assert forall|i: int|
                        (0 <= i < start as int || start as int + count as int <= i < self@.capacity())
                        implies self@.is_allocated(i) == old(self)@.is_allocated(i)
                    by {
                        assert(self.frame_allocator@.is_allocated(i) == old(self).frame_allocator@.is_allocated(i));
                    }
                }
                Ok(start)
            },
            Err(e) => Err(e),
        }
    }
