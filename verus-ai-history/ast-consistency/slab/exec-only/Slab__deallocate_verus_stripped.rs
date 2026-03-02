    pub fn deallocate(&mut self, addr: usize) -> Result<(), Error>
    {
        // Issue 3 FIX: Keep runtime bounds check for defensive programming.
        // This protects against unverified callers that may violate preconditions.
        // Check if the address is below the data region.
        if addr < self.data_addr {
            return Err(Error::new(ErrorCode::BadAddress, "pointer out of bounds (below data region)"));
        }

        // Check if the address is beyond the data region.
        // Compute end of data region carefully to avoid overflow.
        // From invariant: data_addr + num_data_blocks * block_size <= usize::MAX.
        let data_region_size: usize = self.num_data_blocks * self.block_size;
        let data_region_end: usize = self.data_addr + data_region_size;
        if addr >= data_region_end {
            return Err(Error::new(ErrorCode::BadAddress, "pointer out of bounds (beyond data region)"));
        }

        // Check if the address is properly aligned to block size.
        if (addr - self.data_addr) % self.block_size != 0 {
            return Err(Error::new(ErrorCode::BadAddress, "unaligned block address"));
        }

        // Compute the bitmap index for this address.
        // Since precondition guarantees is_valid_addr, we know:
        // - addr >= data_addr
        // - addr < data_addr + num_data_blocks * block_size
        // - (addr - data_addr) % block_size == 0

        // The proof above establishes:
        // 1. addr >= self.data_addr (from is_valid_addr precondition).
        // 2. num_index_blocks + block_idx < usize::MAX (from invariant bounds).

        let index: usize = self.num_index_blocks + (addr - self.data_addr) / self.block_size;

        // Since the block is allocated (precondition), test will return true.
        // We don't need this check given the precondition, but it matches original code.
        if !self.index.test(index)? {
            return Err(Error::new(ErrorCode::BadAddress, "block is already free"));
        }

        // Clear the bit to deallocate.
        match self.index.clear(index) {
            Ok(()) => {
                Ok(())
            },
            Err(e) => {
                Err(e)
            }
        }
    }
