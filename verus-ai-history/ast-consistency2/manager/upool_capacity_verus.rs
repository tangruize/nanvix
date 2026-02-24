    pub fn upool_capacity(&self) -> (result: usize)
        requires
            self.inv(),
        ensures
            result as int == self@.upool_capacity,
    {
        self.upool.capacity()
    }
