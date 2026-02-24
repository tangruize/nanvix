    pub fn number_of_bits(&self) -> (result: usize)
        requires
            self.inv(),
        ensures
            result as int == self@.number_of_bits(),
            result > 0,
            result < u32::MAX as usize,
    {
        self.number_of_bits
    }
