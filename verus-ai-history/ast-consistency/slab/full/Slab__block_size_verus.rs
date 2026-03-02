    pub fn block_size(&self) -> (result: usize)
        requires self.inv(),
        ensures result as int == self@.block_size,
    {
        self.block_size
    }
