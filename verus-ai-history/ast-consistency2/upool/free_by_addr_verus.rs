    pub fn free_by_addr(&mut self, addr: usize) -> (result: Result<(), Error>)
        requires
            old(self).inv(),
            addr as int % FRAME_SIZE as int == 0,
            (addr as int / FRAME_SIZE as int) < (old(self))@.capacity(),
            (old(self))@.is_allocated(addr as int / FRAME_SIZE as int),
        ensures
            self.inv(),
            // LIVENESS: free always succeeds when preconditions are met.
            result is Ok,
            self@.capacity() == (old(self))@.capacity(),
            // Frame is now free.
            !self@.is_allocated(addr as int / FRAME_SIZE as int),
            // All other frames unchanged.
            forall|i: int| #![trigger self@.is_allocated(i)]
                0 <= i < self@.capacity() && i != addr as int / FRAME_SIZE as int ==>
                self@.is_allocated(i) == (old(self))@.is_allocated(i),
            // Count decreases by exactly 1.
            self@.num_allocated() == old(self)@.num_allocated() - 1,
    {
        let frame_addr: FrameAddress = FrameAddress { raw_addr: addr };
        self.frame_allocator.free(frame_addr)
    }
