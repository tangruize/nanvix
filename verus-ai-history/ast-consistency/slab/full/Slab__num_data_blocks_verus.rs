    pub fn num_data_blocks(&self) -> (result: usize)
        requires self.inv(),
        ensures result as int == self@.num_data_blocks,
    {
        self.num_data_blocks
    }
