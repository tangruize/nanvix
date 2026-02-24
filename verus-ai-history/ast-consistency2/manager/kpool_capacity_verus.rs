    pub fn kpool_capacity(&self) -> (result: usize)
        requires
            self.inv(),
        ensures
            result as int == self@.kpool_capacity,
    {
        self.kpool.capacity()
    }
