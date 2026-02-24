    pub fn allocate(&mut self) -> Result<*mut u8, Error> {
        let block: usize = self.index.alloc()?;
        // Safety: the start and resulting addresses are valid.
        let block_addr: *mut u8 = unsafe {
            self.data_addr
                .add((block - self.num_index_blocks) * self.block_size)
        };
        Ok(block_addr)
    }
