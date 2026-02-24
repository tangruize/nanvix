    pub fn alloc(&mut self) -> Result<usize, Error> {
        self.alloc_range(1)
    }
