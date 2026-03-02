    pub fn allocate(&mut self) -> Result<usize, Error>
    {
        let alloc_result = self.index.alloc();

        // Handle error case explicitly.
        let block: usize = match alloc_result {
            Ok(b) => b,
            Err(e) => {
                return Err(e);
            }
        };

        let block_idx: usize = block - self.num_index_blocks;

        // Prove bounds for safe multiplication.
        let product: usize = block_idx * self.block_size;
        let block_addr: usize = self.data_addr + product;

        Ok(block_addr)
    }
