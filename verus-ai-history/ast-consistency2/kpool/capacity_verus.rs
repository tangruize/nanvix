    pub fn capacity(&self) -> (result: usize)
        requires self.inv(),
        ensures result as int == self@.capacity()
    {
        self.frame_allocator.capacity()
    }
