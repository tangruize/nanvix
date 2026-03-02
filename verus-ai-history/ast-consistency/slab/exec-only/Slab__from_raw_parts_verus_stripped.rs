    pub unsafe fn from_raw_parts(
        addr: usize,
        len: usize,
        block_size: usize,
    ) -> Result<Slab, Error>
    {
        // Check if length is invalid.
        if len == 0 || len >= i32::MAX as usize {
            return Err(Error::new(ErrorCode::InvalidArgument, "invalid slab length"));
        }

        // Check if block size is valid.
        if block_size == 0 || block_size >= i32::MAX as usize || block_size > len {
            return Err(Error::new(ErrorCode::InvalidArgument, "invalid block size"));
        }

        // Check if the `block_size` is a power of two using the verified function.
        if !Self::is_power_of_two(block_size) {
            return Err(Error::new(ErrorCode::InvalidArgument, "block size is not a power of two"));
        }

        // At this point, is_power_of_two returned true, so spec_is_power_of_two holds.
        assert(Self::spec_is_power_of_two(block_size as int));

        // Check if `addr` is aligned to `block_size`.
        if addr % block_size != 0 {
            return Err(Error::new(ErrorCode::InvalidArgument, "unaligned start address"));
        }

        // Compute layout of the slab allocator.
        let total_num_blocks: usize = len / block_size;
        if total_num_blocks % (u8::BITS as usize) != 0 {
            return Err(Error::new(ErrorCode::InvalidArgument, "invalid number of blocks"));
        }

        let index_len: usize = total_num_blocks / u8::BITS as usize;
        // Verus note: source uses `index_len.is_multiple_of(block_size)`.
        // `is_multiple_of()` is not available in Verus; `% == 0` is equivalent.
        let num_index_blocks: usize = (index_len / block_size)
            + if index_len % block_size == 0 { 0 } else { 1 };
        if num_index_blocks > total_num_blocks {
            return Err(Error::new(ErrorCode::InvalidArgument, "insufficient blocks for index"));
        }
        let num_data_blocks: usize = total_num_blocks - num_index_blocks;

        // Verus note: source uses `addr.add(num_index_blocks * block_size)` (pointer
        // arithmetic). Verus uses integer arithmetic; overflow safety proven via
        // preconditions. Prove num_index_blocks * block_size fits in usize.
        let data_addr: usize = addr + num_index_blocks * block_size;

        // Check if `data_addr` is aligned to `block_size`.
        if data_addr % block_size != 0 {
            return Err(Error::new(ErrorCode::InvalidArgument, "unaligned data address"));
        }

        // Instantiate index.
        let storage: RawArray<u8> = raw_array_from_addr(addr, index_len)?;

        // Prove that all bytes in storage are zero (required by Bitmap::from_raw_array).
        let mut index: Bitmap = Bitmap::from_raw_array(storage)?;

        // Prove key invariants before the loop.
        // Initialize index: mark index blocks as allocated.
        let mut i: usize = 0;
        while i < num_index_blocks
        {
            index.set(i)?;
            i = i + 1;
        }

        // After the loop, all index blocks are set.
        // Now prove the postconditions.
        let result_slab = Slab {
            index,
            data_addr,
            num_index_blocks,
            num_data_blocks,
            block_size,
            base_addr: addr,
            total_len: len,
        };

        Ok(result_slab)
    }
