    pub fn usage(&self) -> (result: usize)
        requires
            self.inv(),
        ensures
            result as int == self@.usage(),
            result as int <= self@.number_of_bits(),
    {
        self.usage
    }
