    pub fn capacity(&self) -> (result: usize)
        requires self.inv(),
        ensures
            result as int == self@.capacity(),
            // Capacity is always positive (from invariant).
            result > 0,
    {
        self.frame_allocator.capacity()
    }
