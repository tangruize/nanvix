    pub unsafe fn deallocate(&mut self, ptr: *const u8) -> Result<(), Error> {
        // Check if the pointer lies in a memory region that is not managed by this allocator.
        // Safety: the start and resulting addresses are valid.
        if ptr < self.data_addr
            || ptr >= unsafe { self.data_addr.add(self.num_data_blocks * self.block_size) }
        {
            return Err(Error::new(ErrorCode::BadAddress, "pointer out of bounds"));
        }

        // Compute the block index.
        // Safety: we have already checked that ptr is within the bounds of the slab.
        let index: usize = self.num_index_blocks
            + unsafe { ptr.offset_from_unsigned(self.data_addr) } / self.block_size;

        // Check if the block is already free.
        if !self.index.test(index)? {
            return Err(Error::new(ErrorCode::BadAddress, "block is already free"));
        }

        // Free the block.
        self.index.clear(index)?;

        Ok(())
    }
